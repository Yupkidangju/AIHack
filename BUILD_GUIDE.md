# 빌드 가이드 (BUILD_GUIDE)

이 문서는 AIHack 프로젝트를 빌드하고 실행하기 위한 절차를 설명합니다.

## 1. 사전 요구 사항

| 도구 | 최소 버전 | 용도 |
|-----|----------|-----|
| **Rust** | 1.84+ (Stable) | 핵심 개발 언어 |
| **Cargo** | Rust 번들 | 패키지 관리자 |
| **MSVC Build Tools** | VS 2022+ | Windows C 링커 (egui 의존성) |
| **Git** | 2.40+ | 버전 관리 |

### 1.1 Rust 설치 확인
```powershell
rustc --version   # 1.84.0 이상 확인
cargo --version
```

### 1.2 Windows 빌드 도구
`egui`(`eframe`)는 내부적으로 C 라이브러리를 사용합니다. Windows에서 빌드하려면 MSVC Build Tools가 필요합니다.
- Visual Studio Installer에서 "C++ 빌드 도구" 워크로드 설치

---

## 2. 빌드 절차

### 2.1 디버그 빌드 (개발용)
```powershell
# 프로젝트 루트에서
cargo build
```

### 2.2 실행
```powershell
cargo run
```

### 2.3 릴리즈 빌드 (배포용)
```powershell
cargo build --release
```
릴리즈 바이너리 경로: `target/release/nethack-rs.exe`

### 2.4 빌드 스크립트 (자동화)
```powershell
# PowerShell 빌드 스크립트
./build.ps1
```

---

## 3. 의존성 목록

| 크레이트 | 버전 | 용도 |
|---------|-----|-----|
| `ratatui` | 0.26 | TUI 맵 렌더링 |
| `eframe` | 0.27 | egui 네이티브 윈도우 |
| `crossterm` | 0.27 | 키보드/마우스 이벤트 |
| `legion` | 0.4 | ECS 엔처 |
| `serde` | 1.0 | 직렬화 |
| `serde_json` | 1.0 | JSON 직렬화 |
| `toml` | 0.8 | 설정 파일 파싱 |
| `rand` | 0.8 | 난수 생성 |
| `bitflags` | 2.4 | 비트플래그 |
| `serde_repr` | 0.1 | 열거형 직렬화 |
| `lazy_static` | 1.4 | 정적 초기화 |

---

## 4. 프로젝트 구조

```
root/
├── src/              # Rust 소스 (192개 파일, 114,280라인)
│   ├── main.rs       # 앱 엔트리포인트 + AppState 화면 분기
│   ├── app.rs        # NetHackApp 구조체 + 초기화
│   ├── game_loop.rs  # 게임 턴 처리
│   ├── game_ui.rs    # UI 렌더링
│   ├── input_handler.rs # 입력 처리
│   ├── core/         # 게임 로직
│   ├── ui/           # UI 레이어 (screens/, layout/, widgets/)
│   ├── assets/       # 에셋 로더
│   └── util/         # 유틸리티
├── assets/           # 게임 데이터 (TOML)
│   └── data/
├── nethack_original/ # 원본 C 소스 (참조용)
├── Cargo.toml        # 의존성 정의
└── options.toml      # 게임 옵션
```

---

## 5. 환경 변수 설정

현재 별도의 환경 변수는 필요하지 않습니다. 게임 옵션은 `options.toml`에서 관리합니다.

---

## 6. 문제 해결 (Troubleshooting)

### 6.1 eframe 빌드 실패
```
error: linker `link.exe` not found
```
→ MSVC Build Tools 설치 후 재시도

### 6.2 에셋 로드 실패
```
Failed to load assets/data/monsters.toml
```
→ 프로젝트 루트에서 `cargo run` 실행 확인 (작업 디렉토리가 프로젝트 루트여야 함)

### 6.3 윈도우가 뜨지 않는 경우
- GPU 드라이버 업데이트 확인 (egui는 GPU 가속 사용)
- `RUST_LOG=debug cargo run`으로 디버그 로그 확인

---

**문서 버전**: v2.19.0
**최종 업데이트**: 2026-02-21
