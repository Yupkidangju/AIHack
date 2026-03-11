// ============================================================================
// AIHack - LLM 프로세스 관리자
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// smaLLM engine/process.rs에서 이식 (v0.2.25, 974줄 → ~350줄)
// Tauri AppHandle 의존성 완전 제거, 15→6 파라미터 축소
// ============================================================================
// [v3.0.0 E4] Phase E4: LLM Engine Integration
//
// 핵심 유지 로직 (smaLLM 검증 완료):
// - spawn_lock: 동시 실행 차단 (좀비 프로세스 원천 방지)
// - 포트 호핑: TOCTOU 최소화
// - Windows 경로 안전 처리: GetShortPathNameW (한글 경로 대응)
// - wait_for_ready: Raw TCP /health 폴링 (90초 타임아웃)
// - CPU Only 3중 방어: --device none + -ngl 0 + 환경변수
// ============================================================================

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// [v3.0.0 E4] llama-server.exe 프로세스 관리자
/// smaLLM ProcessManager에서 Tauri 의존성을 제거한 게임 전용 버전
#[derive(Clone)]
pub struct ProcessManager {
    /// 관리 중인 자식 프로세스
    child: Arc<Mutex<Option<Child>>>,
    /// 서버 준비 완료 여부
    is_ready: Arc<Mutex<bool>>,
    /// [smaLLM 패턴] 다중 스레드 좀비 방지: start 동시 실행 차단
    spawn_lock: Arc<Mutex<()>>,
    /// 현재 사용 중인 포트
    current_port: Arc<Mutex<u16>>,
}

/// [smaLLM 패턴] Windows 한글 경로 → 8.3 짧은 경로 변환
/// llama-server.exe(C++)가 UTF-8 한글 경로를 읽지 못하는 문제 해결
#[cfg(windows)]
fn safe_windows_path(path: &str) -> String {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::fileapi::GetShortPathNameW;

    let canonical = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => return path.to_string(),
    };

    let wide: Vec<u16> = OsStr::new(&canonical)
        .encode_wide()
        .chain(Some(0))
        .collect();

    unsafe {
        let mut buffer = vec![0u16; 1024];
        let len = GetShortPathNameW(wide.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32);

        if len > 0 && len < buffer.len() as u32 {
            return String::from_utf16_lossy(&buffer[..len as usize]);
        }
    }
    path.to_string()
}

#[cfg(not(windows))]
fn safe_windows_path(path: &str) -> String {
    path.to_string()
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            is_ready: Arc::new(Mutex::new(false)),
            spawn_lock: Arc::new(Mutex::new(())),
            current_port: Arc::new(Mutex::new(8080)),
        }
    }

    /// llama-server 시작
    ///
    /// # 인자 (smaLLM 15개 → 6개로 축소)
    /// * `binary_dir` - llama-server.exe가 있는 디렉토리
    /// * `model_path` - GGUF 모델 파일 경로
    /// * `ngl` - GPU 오프로드 레이어 수 (0=CPU, 999=전부)
    /// * `port` - 기본 포트 (충돌 시 자동 호핑)
    /// * `ctx_size` - 컨텍스트 크기 (토큰 수)
    /// * `backend` - "cuda", "vulkan", "cpu"
    pub fn start(
        &self,
        binary_dir: &Path,
        model_path: String,
        ngl: u32,
        port: u16,
        ctx_size: u32,
        backend: &str,
    ) -> Result<(u32, u16), String> {
        // [smaLLM 패턴] spawn_lock으로 동시 실행 차단
        let _spawn_guard = match self.spawn_lock.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::WouldBlock) => {
                return Err("엔진이 이미 시작 중입니다. 완료될 때까지 기다려주세요.".to_string());
            }
            Err(std::sync::TryLockError::Poisoned(poisoned)) => {
                println!("[Process] WARNING: spawn_lock poisoned, force-acquiring");
                poisoned.into_inner()
            }
        };

        // Windows 경로 안전 처리
        let safe_model = safe_windows_path(&model_path);

        // 기존 프로세스 종료
        self.stop();
        *self.is_ready.lock().unwrap_or_else(|e| e.into_inner()) = false;

        // 바이너리 탐색
        let cmd_path = self.find_binary(binary_dir)?;

        println!("=== AIHack LLM 엔진 시작 ===");
        println!("바이너리: {:?}", cmd_path);
        println!("모델: {}", model_path);
        println!("GPU 레이어: {}, 컨텍스트: {}, 백엔드: {}", ngl, ctx_size, backend);

        // [smaLLM 패턴] 포트 호핑 (TOCTOU 최소화: args 빌드 직전)
        let mut actual_port = port;
        for p in port..=port.saturating_add(100).min(65535) {
            if std::net::TcpListener::bind(("127.0.0.1", p)).is_ok() {
                actual_port = p;
                break;
            }
        }
        if actual_port != port {
            println!("[Port Hop] {} -> {} (원래 포트가 사용 중)", port, actual_port);
        }

        *self.current_port.lock().unwrap_or_else(|e| e.into_inner()) = actual_port;

        // 커맨드 인자 생성
        let mut args = vec![
            "-m".to_string(),
            safe_model.clone(),
            "--port".to_string(),
            actual_port.to_string(),
            "--ctx-size".to_string(),
            ctx_size.to_string(),
            "--host".to_string(),
            "127.0.0.1".to_string(),
            // [v0.2.25] llama.cpp b8192: --embeddings (복수형)
            "--embeddings".to_string(),
        ];

        // [smaLLM 패턴] CPU Only 3중 방어
        let is_cpu_only = backend == "cpu";

        if is_cpu_only {
            args.push("-ngl".to_string());
            args.push("0".to_string());
            args.push("--device".to_string());
            args.push("none".to_string());
            args.push("--no-kv-offload".to_string());
            println!("[Process] CPU Only 모드: --device none + -ngl 0 + --no-kv-offload");
        } else if ngl > 0 {
            args.push("-ngl".to_string());
            args.push(ngl.to_string());
        }

        // 게임용: 병렬 요청 1개 (턴 기반이므로 동시 요청 없음)
        args.push("--parallel".to_string());
        args.push("1".to_string());

        // Flash Attention 활성화 (VRAM 절약)
        args.push("--flash-attn".to_string());
        args.push("auto".to_string());

        println!("Args: {:?}", args);

        // Command 생성
        let mut cmd = Command::new(&cmd_path);
        cmd.args(&args);

        // DLL 경로 주입 (바이너리 디렉토리)
        if let Some(dir) = cmd_path.parent() {
            cmd.current_dir(dir);

            // [smaLLM 패턴] DLL PATH 주입
            if let Some(current_paths) = std::env::var_os("PATH") {
                let mut all_paths = vec![dir.to_path_buf()];
                all_paths.extend(std::env::split_paths(&current_paths));
                if let Ok(new_path) = std::env::join_paths(all_paths) {
                    cmd.env("PATH", new_path);
                }
            }
        }

        // [smaLLM 패턴] CPU Only: GPU 환경변수 비활성화
        if is_cpu_only {
            cmd.env("VK_ICD_FILENAMES", "");
            cmd.env("VK_DRIVER_FILES", "");
            cmd.env("VK_LOADER_LAYERS_DISABLE", "~implicit~");
            cmd.env("CUDA_VISIBLE_DEVICES", "");
        }

        // 로그 파일 (Append 모드)
        let log_path = cmd_path
            .parent()
            .map(|d| d.join("llama-server.log"))
            .unwrap_or_else(|| PathBuf::from("llama-server.log"));

        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();

        cmd.stdout(Stdio::null());
        if let Some(file) = log_file {
            cmd.stderr(Stdio::from(file));
        } else {
            cmd.stderr(Stdio::null());
        }

        // Windows: 콘솔 창 숨김
        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        // 프로세스 실행
        let child = cmd.spawn().map_err(|e| {
            format!(
                "llama-server 실행 실패: {}. 경로: {:?}",
                e, cmd_path
            )
        })?;

        let pid = child.id();
        println!("llama-server 시작됨, PID: {}", pid);

        {
            let mut store = self.child.lock().unwrap_or_else(|e| e.into_inner());
            *store = Some(child);
        }

        // [smaLLM 패턴] 서버 준비 대기 (Raw TCP /health, 90초)
        let ready = self.wait_for_ready(actual_port, 90)?;

        if ready {
            *self.is_ready.lock().unwrap_or_else(|e| e.into_inner()) = true;
            println!("=== LLM 엔진 준비 완료! 포트: {} ===", actual_port);
            Ok((pid, actual_port))
        } else {
            self.stop();
            Err("서버가 타임아웃 내에 시작되지 않았습니다. 모델 경로와 메모리를 확인하세요."
                .to_string())
        }
    }

    /// llama-server 바이너리 탐색
    fn find_binary(&self, binary_dir: &Path) -> Result<PathBuf, String> {
        let binary_names = [
            "llama-server-x86_64-pc-windows-msvc.exe",
            "llama-server.exe",
        ];

        let mut searched: Vec<String> = Vec::new();

        // 1. 지정된 binary_dir에서 탐색
        for name in &binary_names {
            let p = binary_dir.join(name);
            searched.push(p.display().to_string());
            if p.exists() {
                return Ok(p);
            }
        }

        // 2. 실행 파일 옆에서 탐색
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                for name in &binary_names {
                    let p = exe_dir.join(name);
                    searched.push(p.display().to_string());
                    if p.exists() {
                        return Ok(p);
                    }
                }
            }
        }

        Err(format!(
            "llama-server를 찾을 수 없습니다! 탐색 경로:\n{}",
            searched.join("\n")
        ))
    }

    /// [smaLLM 패턴] 서버 /health 엔드포인트 폴링
    /// reqwest::blocking 대신 Raw TCP 사용 (tokio 런타임 충돌 방지)
    fn wait_for_ready(&self, port: u16, timeout_secs: u32) -> Result<bool, String> {
        let url = format!("http://127.0.0.1:{}/health", port);
        println!("서버 대기 중: {}...", url);

        for i in 0..timeout_secs * 2 {
            // 프로세스 생존 확인
            {
                let mut child_guard = self.child.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(ref mut child) = *child_guard {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            return Err(format!(
                                "llama-server 조기 종료. 종료 코드: {}",
                                status
                            ));
                        }
                        Ok(None) => {} // 실행 중, OK
                        Err(e) => {
                            return Err(format!("프로세스 상태 확인 에러: {}", e));
                        }
                    }
                }
            }

            // [smaLLM 패턴] Raw TCP HTTP GET /health
            if let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) {
                use std::io::{Read, Write};
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(100)));
                let _ = stream.write_all(b"GET /health HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n");
                let mut response = vec![0u8; 512];
                if let Ok(n) = stream.read(&mut response) {
                    let resp_str = String::from_utf8_lossy(&response[..n]);
                    if resp_str.contains("200 OK") || resp_str.contains("200 ok") {
                        println!("서버 /health 200 OK — ~{:.1}초 경과", (i as f32) / 2.0);
                        return Ok(true);
                    } else if i % 4 == 0 {
                        println!("소켓 열림, 모델 로딩 중...");
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(500));

            if i % 4 == 0 && i > 0 {
                println!("대기 중... ({}초)", i / 2);
            }
        }

        Ok(false)
    }

    /// 프로세스 생존 확인
    pub fn is_alive(&self) -> bool {
        let mut child = self.child.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref mut c) = *child {
            matches!(c.try_wait(), Ok(None))
        } else {
            false
        }
    }

    /// 프로세스 종료
    pub fn stop(&self) {
        let mut child = self.child.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(mut c) = child.take() {
            let _ = c.kill();
            let _ = c.wait();
            println!("[Process] llama-server 프로세스 종료됨");
        }
        *self.is_ready.lock().unwrap_or_else(|e| e.into_inner()) = false;
    }

    /// 현재 포트 조회
    pub fn get_port(&self) -> u16 {
        *self.current_port.lock().unwrap_or_else(|e| e.into_inner())
    }
}
