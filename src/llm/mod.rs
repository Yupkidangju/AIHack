// ============================================================================
// AIHack - LLM 엔진 모듈
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// smaLLM Rust Engine (Vibe-LLM v0.2.25)에서 이식
// llama.cpp b8192 Sidecar 기반 로컬 LLM 추론 엔진
// ============================================================================
// [v3.0.0 E4] Phase E4: LLM Engine Integration
// - ProcessManager: llama-server.exe 프로세스 관리 (시작/종료/헬스체크)
// - AcceleratorInfo: GPU 자동 감지 (CUDA/Vulkan/CPU)
// - LlmEngine: 텍스트 생성 공개 API
// ============================================================================

pub mod accelerator;
pub mod process;

use std::path::Path;
use std::sync::OnceLock;

// [v3.0.0 E4] 전역 HTTP 클라이언트 싱글턴
// smaLLM 패턴: 소켓 고갈 방지를 위해 Client를 재사용
static LLM_CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();

fn get_llm_client() -> reqwest::blocking::Client {
    LLM_CLIENT
        .get_or_init(|| {
            reqwest::blocking::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(30))
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default()
        })
        .clone()
}

/// [v3.0.0 E4] AIHack LLM 엔진
/// smaLLM의 ProcessManager + EngineState를 게임용으로 통합한 구조체
/// 
/// 사용법:
/// ```no_run
/// let engine = LlmEngine::start("models/qwen3-4b.gguf", Path::new("binaries"))?;
/// let text = engine.generate("묘비명을 지어줘", 60)?;
/// engine.stop();
/// ```
pub struct LlmEngine {
    /// llama-server 프로세스 관리자
    process: process::ProcessManager,
    /// 엔진 API 기본 URL (예: http://127.0.0.1:8080)
    base_url: String,
}

impl LlmEngine {
    /// LLM 엔진 시작
    /// 
    /// GPU를 자동 감지하여 최적 백엔드(Vulkan/CPU)를 선택하고
    /// llama-server.exe를 Sidecar 프로세스로 시작합니다.
    ///
    /// # 인자
    /// * `model_path` - GGUF 모델 파일 경로
    /// * `binary_dir` - llama-server.exe가 있는 디렉토리 경로
    ///
    /// # 반환
    /// 성공 시 `LlmEngine` 인스턴스, 실패 시 에러 메시지
    pub fn start(model_path: &str, binary_dir: &Path) -> Result<Self, String> {
        // 1. GPU 자동 감지
        let detection = accelerator::detect_hardware();
        println!(
            "[LLM] 하드웨어 감지 완료: {} GPU(s), 추천 백엔드: {}, 총 VRAM: {}MB",
            detection.gpu_count, detection.recommended_backend, detection.total_vram_mb
        );

        // 2. 컨텍스트 크기 결정 (4B 모델 기준 2048이면 충분)
        let ctx_size: u32 = 2048;
        
        // 3. GPU 레이어 결정
        // Vulkan/CUDA: 모든 레이어 GPU 오프로드 (999 = 전부)
        // CPU: 0
        let ngl: u32 = if detection.recommended_backend == "cpu" { 0 } else { 999 };

        // 4. 프로세스 매니저 생성 및 시작
        let pm = process::ProcessManager::new();
        let (pid, port) = pm.start(
            binary_dir,
            model_path.to_string(),
            ngl,
            8080, // 기본 포트 (충돌 시 자동 호핑)
            ctx_size,
            &detection.recommended_backend,
        )?;

        println!("[LLM] 엔진 시작 완료 — PID: {}, Port: {}", pid, port);

        Ok(Self {
            process: pm,
            base_url: format!("http://127.0.0.1:{}", port),
        })
    }

    /// 텍스트 생성 (동기, 비스트리밍)
    /// 
    /// OpenAI v1 chat/completions API를 사용하여 텍스트를 생성합니다.
    /// 턴 기반 게임에 적합한 동기 호출 방식입니다.
    ///
    /// # 인자
    /// * `prompt` - 시스템/사용자 프롬프트
    /// * `max_tokens` - 최대 생성 토큰 수
    ///
    /// # 반환
    /// 생성된 텍스트 또는 에러 메시지
    pub fn generate(&self, prompt: &str, max_tokens: u32) -> Result<String, String> {
        let client = get_llm_client();

        // [v3.0.0 E4] 게임용 LLM 파라미터 (포맷 이탈 방지 + 자연스러운 텍스트 균형)
        // temp=0.5: 구조화 데이터 형식 유지하면서 적당한 다양성
        // top_p=0.85, top_k=25: 무의미한 토큰 컷오프
        // repetition_penalty=1.15: 반복 문구 억제
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.5,
            "top_p": 0.85,
            "top_k": 25,
            "repetition_penalty": 1.15,
            "max_tokens": max_tokens,
            "stream": false,
        });

        let resp = client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&body)
            .send()
            .map_err(|e| format!("LLM 요청 실패: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().unwrap_or_default();
            return Err(format!("LLM API 에러 ({}): {}", status, err_text));
        }

        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("JSON 파싱 실패: {}", e))?;

        // [v3.0.0 E4] smaLLM 패턴: 안전한 옵셔널 체이닝
        json.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("LLM 응답에 content 없음: {:?}", json))
    }

    /// 시스템 프롬프트를 포함한 텍스트 생성
    /// 
    /// 게임 맥락에 맞는 시스템 프롬프트를 설정하여 더 정확한 응답을 유도합니다.
    pub fn generate_with_system(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: u32,
    ) -> Result<String, String> {
        let client = get_llm_client();

        let body = serde_json::json!({
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.5,
            "top_p": 0.85,
            "top_k": 25,
            "repetition_penalty": 1.15,
            "max_tokens": max_tokens,
            "stream": false,
        });

        let resp = client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&body)
            .send()
            .map_err(|e| format!("LLM 요청 실패: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().unwrap_or_default();
            return Err(format!("LLM API 에러 ({}): {}", status, err_text));
        }

        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("JSON 파싱 실패: {}", e))?;

        json.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("LLM 응답에 content 없음: {:?}", json))
    }

    /// 엔진 상태 확인
    pub fn is_alive(&self) -> bool {
        self.process.is_alive()
    }

    /// 엔진 종료
    pub fn stop(&self) {
        self.process.stop();
        println!("[LLM] 엔진 종료 완료");
    }
}

impl Drop for LlmEngine {
    fn drop(&mut self) {
        self.stop();
    }
}
