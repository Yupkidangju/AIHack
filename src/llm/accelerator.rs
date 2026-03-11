// ============================================================================
// AIHack - GPU/CPU 하드웨어 자동 감지
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// smaLLM core/accelerator.rs에서 이식 (v0.2.25)
// nvidia-smi 우선 → PowerShell Win32_VideoController 폴백
// CUDA/Vulkan/CPU 후보 자동 선택
// ============================================================================
// [v3.0.0 E4] Phase E4: LLM Engine Integration

use serde::Serialize;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// GPU 하드웨어 정보
#[derive(Serialize, Clone, Debug)]
pub struct AcceleratorInfo {
    /// GPU 인덱스 (멀티 GPU 식별용)
    pub gpu_index: usize,
    /// GPU 이름 (예: "NVIDIA GeForce RTX 4070")
    pub name: String,
    /// 제조사 ("NVIDIA", "AMD", "Intel", "Unknown")
    pub vendor: String,
    /// VRAM 용량 (MB)
    pub vram_mb: u64,
    /// CUDA 지원 여부
    pub has_cuda: bool,
    /// Vulkan 지원 여부
    pub has_vulkan: bool,
}

/// 하드웨어 감지 결과
#[derive(Serialize, Clone, Debug)]
pub struct DetectionResult {
    /// 감지된 GPU 목록
    pub accelerators: Vec<AcceleratorInfo>,
    /// 추천 백엔드 ("cuda", "vulkan", "cpu")
    pub recommended_backend: String,
    /// 전체 GPU VRAM 합계 (MB)
    pub total_vram_mb: u64,
    /// GPU 개수
    pub gpu_count: usize,
}

/// 하드웨어 자동 감지 수행
/// 
/// smaLLM 패턴: nvidia-smi 우선 시도 → PowerShell 폴백 (AMD/Intel)
pub fn detect_hardware() -> DetectionResult {
    let mut accelerators = Vec::new();
    let mut nvidia_found = false;
    let mut gpu_index: usize = 0;

    // 1단계: nvidia-smi로 NVIDIA GPU 감지 (가장 정확)
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let output = Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=name,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let name = parts[0].trim().to_string();
                    let vram_mb = parts[1].trim().parse::<u64>().unwrap_or(0);

                    if vram_mb > 0 {
                        accelerators.push(AcceleratorInfo {
                            gpu_index,
                            name,
                            vendor: "NVIDIA".to_string(),
                            vram_mb,
                            has_cuda: true,
                            has_vulkan: true,
                        });
                        gpu_index += 1;
                        nvidia_found = true;
                    }
                }
            }
        }
    }

    // 2단계: nvidia-smi 실패 시 PowerShell 폴백 (AMD/Intel)
    #[cfg(target_os = "windows")]
    if !nvidia_found {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let ps_script = r#"
            Get-CimInstance Win32_VideoController | ForEach-Object {
                $name = $_.Name
                $ram = $_.AdapterRAM
                $compat = $_.AdapterCompatibility
                "$name|$ram|$compat"
            }
        "#;

        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", ps_script])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.trim().is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split('|').collect();
                let name = parts.first().unwrap_or(&"Unknown GPU").trim().to_string();
                let ram_str = parts.get(1).unwrap_or(&"0").trim();
                let compat = parts.get(2).unwrap_or(&"").trim().to_lowercase();

                let lower_name = name.to_lowercase();
                let vendor =
                    if lower_name.contains("nvidia") || compat.contains("nvidia") {
                        "NVIDIA"
                    } else if lower_name.contains("amd")
                        || lower_name.contains("radeon")
                        || compat.contains("amd")
                    {
                        "AMD"
                    } else if lower_name.contains("intel") || compat.contains("intel") {
                        "Intel"
                    } else {
                        "Unknown"
                    };

                // VRAM 파싱 (AdapterRAM은 바이트 단위, i32 오버플로우로 음수 가능)
                let vram_mb = ram_str.parse::<i64>().unwrap_or(0).unsigned_abs() / (1024 * 1024);
                let vram_mb = if vram_mb == 0 {
                    match vendor {
                        "NVIDIA" => 8192,
                        "AMD" => 8192,
                        "Intel" => 2048,
                        _ => 0,
                    }
                } else {
                    vram_mb
                };

                let has_cuda = vendor == "NVIDIA";
                let has_vulkan = vendor == "NVIDIA"
                    || vendor == "AMD"
                    || (vendor == "Intel"
                        && (lower_name.contains("arc") || lower_name.contains("iris")));

                // 기본 디스플레이 어댑터 제외
                if vendor != "Unknown" && !lower_name.contains("microsoft basic") {
                    accelerators.push(AcceleratorInfo {
                        gpu_index,
                        name,
                        vendor: vendor.to_string(),
                        vram_mb,
                        has_cuda,
                        has_vulkan,
                    });
                    gpu_index += 1;
                }
            }
        }
    }

    // 3단계: 백엔드 추천 (cuda > vulkan > cpu)
    let recommended_backend = if accelerators.iter().any(|a| a.has_cuda) {
        "cuda".to_string()
    } else if accelerators.iter().any(|a| a.has_vulkan) {
        "vulkan".to_string()
    } else {
        "cpu".to_string()
    };

    let total_vram_mb: u64 = accelerators.iter().map(|a| a.vram_mb).sum();
    let gpu_count = accelerators.len();

    DetectionResult {
        accelerators,
        recommended_backend,
        total_vram_mb,
        gpu_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_hardware_no_panic() {
        // GPU가 없는 CI 환경에서도 패닉 없이 실행되어야 함
        let result = detect_hardware();
        assert!(
            result.recommended_backend == "cuda"
                || result.recommended_backend == "vulkan"
                || result.recommended_backend == "cpu"
        );
    }
}
