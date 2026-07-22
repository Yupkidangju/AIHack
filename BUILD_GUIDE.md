# AIHack Build Guide v2

> Archive chain
> - Latest: `.archive/BUILD_GUIDE_archive_260715.md`
> - Previous: first archive
>
> 과거 Phase 빌드 절차와 hash 이력은 아카이브에 있다. 이 문서는 현재 실행법과 v0.3.0 target build contract를 구분한다.

문서 상태: active build contract
작성일: 2026-07-15
관련 Task: R1-1..R1-3, R4-1..R4-2, R5-1..R5-2, R8-1

## 1. 현재 상태

| 항목 | 현재 working tree | v0.3.0 target |
| --- | --- | --- |
| Rust | `rust-toolchain.toml` 1.94.1 고정 | `rust-toolchain.toml` 1.94.1 |
| package | 7개 library/app package와 root compatibility facade | workspace, release 0.3.0 |
| edition/MSRV | edition 2021, rust-version 1.94 | edition 2021, rust-version 1.94 |
| UI | ratatui 0.30.x + crossterm 0.29 단일 계열 | 같은 계열 유지 |
| binary 선택 | TUI default-run `aihack`, headless는 `-p aihack-headless --bin` | 같은 이름 + default-run aihack |
| CI | 2026-07-22: `b9bd680200d82b20d7c9ba961a2758caa3d49e16`, [run `29886410221`](https://github.com/Yupkidangju/AIHack/actions/runs/29886410221), `ubuntu-latest quality gate`/`windows-latest quality gate` success | Linux/Windows green |
| script | locked, artifact fail-fast | locked, artifact fail-fast |
| long run | wait-only, 조기 사망도 exit 0 | survival-v1, accepted turn 1000 |

현재 `cargo run --locked -- --seed 42`는 TUI binary를 선택한다.

## 2. 사전 준비

현재 검증:

```bash
rustc --version
cargo --version
cargo metadata --locked --no-deps --format-version 1
```

v0.3.0 구현 후 기대:

```text
rustc 1.94.1 (...)
cargo 1.94.1 (...)
```

필수 도구:

- Rustup 또는 Rust 1.94.1 toolchain
- Cargo
- Git
- Bash: Linux script 및 로컬 audit
- PowerShell 또는 cmd: Windows script
- `rg`: 문서·경계 audit
- `cargo-audit 0.22.1`: RustSec dependency advisory gate
- `cargo-deny 0.19.4`: license, source, duplicate dependency gate
- `sha256sum`, `grep`: R7 runtime content integrity와 Blocked reference gate

local LLM은 core build와 test의 필수 조건이 아니다. LLM integration test는 loopback mock server를 사용하고 외부 네트워크를 사용하지 않는다.

R6 dependency는 `reqwest = { version = "0.13.4", default-features = false, features = ["blocking", "json"] }`로 고정한다. HTTP loopback만 허용하므로 TLS feature를 넣지 않는다. `ClientBuilder::no_proxy()`, connect timeout 500ms, request별 total timeout을 사용한다.

## 3. 현재 빠른 실행

TUI:

```bash
cargo run --locked -- --seed 42
```

Headless:

```bash
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 100
```

headless의 `--turns`는 absolute accepted-turn target이다. long-run 품질 증거에는 `survival-v1`과 report의 `accepted_turns`, `final_hash`를 함께 사용한다.

현재 기본 검증:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --all-targets --locked
cargo build --workspace --release --locked
```

## 4. R1 toolchain과 dependency 고정 결과

### 4.1 `rust-toolchain.toml`

R1-1에서 적용·검증한 값:

```toml
[toolchain]
channel = "1.94.1"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

### 4.2 root package release baseline

R8 라이선스 결정과 version 동기화 후 baseline:

```toml
[package]
name = "aihack"
version = "0.3.0"
edition = "2021"
rust-version = "1.94"
default-run = "aihack"
license = "NGPL"
publish = false

[dependencies]
ratatui = "0.30"
crossterm = "0.29"
```

- 다른 dependency version은 R1에서 기능 변경 없이 lockfile 결과를 검증했다.
- crossterm duplicate 0건을 local gate로 확인했다.
- 2026-07-20 프로젝트 소유자의 파생물 분류에 따라 모든 workspace package에 `license = "NGPL"`과 version 0.3.0을 같은 변경 단위로 적용한다.
- `publish = false`는 crates.io 개별 배포로 whole-work source/notice 의무가 분산되지 않도록 유지한다.

검증:

```bash
cargo update -p ratatui --precise 0.30.2
cargo metadata --locked --no-deps --format-version 1
cargo tree -d
cargo check --workspace --all-targets --locked
```

첫 번째 명령은 이미 R1 구현에서 lockfile을 의도적으로 갱신할 때 사용했다. 이후 감사 세션에서는 실행하지 않는다.

## 5. R1 build script 계약

### 5.1 Linux

`./build.sh [--release] [--test]`:

1. `set -euo pipefail`
2. `--test`이면 `cargo test --workspace --all-targets --locked`
3. release면 untracked file을 포함한 Git worktree가 clean인지 확인하고 아니면 중단
4. debug면 `cargo build --workspace --all-targets --locked`
5. release면 `cargo build --workspace --release --locked`
6. host suffix를 계산해 두 binary를 `output/`에 복사
7. `LICENSE`, `NOTICE`, `MODIFICATIONS.md`, `PROJECT_OWNER_LICENSE_APPROVAL.md`를 `output/`에 복사
8. release commit이 적힌 `RELEASE-METADATA`와 `git archive` source를 생성
9. binary, notice, source archive의 `SHA256SUMS`를 생성
10. `scripts/verify_release_bundle.sh`로 archive 필수 파일, commit expansion, metadata key의 단일성·완전 값, owner/modification ID와 실제 record의 일치, checksum과 legacy 제외를 확인
11. 하나라도 실패하면 exit code 1, 모두 통과하면 정확한 binary 경로 출력

`cp ... || true`와 stderr 폐기는 금지한다.

### 5.2 Windows

`build.bat [--release] [--test]`은 같은 계약을 수행하며 release source는 ZIP으로 생성한다. 필수 artifact:

```text
output\aihack.exe
output\aihack-headless.exe
output\LICENSE
output\NOTICE
output\MODIFICATIONS.md
output\PROJECT_OWNER_LICENSE_APPROVAL.md
output\RELEASE-METADATA
output\SHA256SUMS
output\aihack-0.3.0-source.zip
```

copy 실패 뒤 성공 메시지를 출력하면 R1 실패다.

CI는 Ubuntu에서 `./build.sh --release`, Windows에서 `cmd /c build.bat --release`를 실행한다. 따라서 두 runner 모두 clean checkout의 동일 commit에서 실제 배포 bundle과 대응 source archive를 생성·검증한다. 양쪽 script는 approval/modification ID가 metadata와 bundled record에서 일치하는지 검사하며, Linux 전용 shell verifier 회귀 테스트는 Unix target에서만 컴파일한다.

### 5.3 현재와 target artifact

| 모드 | Cargo artifact | 배포 staging |
| --- | --- | --- |
| current debug | `target/debug/aihack[.exe]` | `output/aihack[.exe]` |
| current headless | `target/debug/aihack-headless[.exe]` | `output/aihack-headless[.exe]` |
| target workspace TUI | `target/debug/aihack[.exe]` | 동일 |
| target workspace headless | `target/debug/aihack-headless[.exe]` | 동일 |
| release | `target/release/*` | `output/*` |
| release source | release commit `HEAD` | `output/aihack-0.3.0-source.tar.gz` 또는 `.zip` |
| release evidence | commit-bound metadata와 owner/modification scope | `output/RELEASE-METADATA`, `PROJECT_OWNER_LICENSE_APPROVAL.md`, `MODIFICATIONS.md`, `SHA256SUMS` |

사용자 CLI와 artifact 이름은 workspace 추출 뒤에도 바꾸지 않는다.

## 6. R1 CI 계약

생성 파일: `.github/workflows/ci.yml`

trigger:

- `push`
- `pull_request`

matrix:

- `ubuntu-latest`
- `windows-latest`

job 순서:

```text
checkout
-> install Rust from rust-toolchain.toml
-> cargo metadata --locked
-> cargo fmt --all -- --check
-> cargo clippy --workspace --all-targets --locked -- -D warnings
-> cargo test --workspace --all-targets --locked
-> cargo build --workspace --release --locked
-> cargo audit
-> cargo deny check licenses bans sources
-> assert Cargo.lock unchanged
```

Linux와 Windows에서 command 의미가 같아야 한다. OS별 shell 차이 때문에 lockfile 검사 구현은 달라도 결과는 동일해야 한다.

CI tool 설치는 `cargo install --locked cargo-audit --version 0.22.1`과 `cargo install --locked cargo-deny --version 0.19.4`로 고정한다. `deny.toml`은 crates.io만 허용하고 license allowlist와 crossterm duplicate deny를 정의한다. exception은 crate, version, 이유, owner, 만료일을 가져야 하며 최대 90일이다.

## 7. R4 headless contract

target 명령:

```bash
cargo run --locked --release -p aihack-headless --bin aihack-headless -- \
  --seed 42 \
  --turns 1000 \
  --policy survival-v1 \
  --report runtime/reports/seed-42.json
```

flag contract:

| flag | type/default | validation and effect |
| --- | --- | --- |
| `--seed` | u64, default 42 | `--load`와 동시 사용 금지 |
| `--turns` | u64, default 1000 | absolute target turn, 1..=1,000,000 |
| `--policy` | wait-v1, survival-v1, replay-file; default survival-v1 | replay-file은 `--replay-in`이 있어야 함 |
| `--save` | optional relative path | 성공 종료 시 SaveDataV1 atomic replace |
| `--load` | optional relative path | SaveDataV1 load, seed 대신 save seed 사용 |
| `--replay-in` | optional relative path | replay-file policy의 CommandIntent JSONL source |
| `--replay-out` | optional relative path | 이번 invocation의 ReplayLineV1 JSONL 기록 |
| `--report` | optional relative path | 기본 `runtime/reports/long-run-<seed>.json` |

`--turns`는 현재 CLI와 같이 final target turn이다. 새 session의 turn 0에서 `--turns 1000`이면 1000번의 `turn_advanced=true`가 필요하다. load turn이 400이면 target 1000까지 600번을 수행하며 report의 `accepted_turns`는 600이다. load turn이 target보다 크면 exit code 2다.

path flag는 repository `runtime/`을 canonical root로 사용한다. absolute path, `..` 탈출, symlink로 root 밖을 가리키는 path를 거부한다. save는 같은 directory의 temp file에 write/flush한 뒤 rename하고, 실패 시 기존 save를 보존한다. `--replay-in`과 `--replay-out`은 같은 canonical path일 수 없다.

필수 stdout 한 줄:

```text
seed=42 policy=survival-v1 requested_turns=1000 accepted_turns=1000 submitted_commands=1000 final_state=Playing final_hash=0123456789abcdef
```

필수 report schema:

```json
{
  "seed": 42,
  "policy": "survival-v1",
  "requested_turns": 1000,
  "accepted_turns": 1000,
  "submitted_commands": 1017,
  "final_state": "Playing",
  "final_hash": "0123456789abcdef"
}
```

`policy` 값은 CLI ID와 같은 `wait-v1`, `survival-v1`, `replay-file`이다. runner 실패 report는 위 공통 필드에 `error`를 추가하며, error에는 실패 turn과 `submitted_commands`가 포함된다.

exit code:

- 0: accepted_turns가 requested_turns와 같음
- 1: accepted action 탐색 실패, replay 부족 또는 조기 GameOver
- 2: CLI/policy/path/save/replay/report 입출력 오류

`accepted_turns <= submitted_commands <= accepted_turns * 16`이어야 한다. reject 후 다음 legal candidate를 시도하므로 submitted 값은 accepted보다 클 수 있다.

report write 실패를 stdout 성공으로 숨기지 않는다.

## 8. R5 workspace build

target members:

```text
crates/aihack-core
crates/aihack-content
crates/aihack-ai-contract
crates/aihack-llm
crates/aihack-runtime
apps/aihack-tui
apps/aihack-headless
```

root `aihack` package는 `publish = false` compatibility facade와 `tests/**` host로 남는다. workspace `default-members`는 `apps/aihack-tui`이고 `default-run = "aihack"`은 TUI app manifest가 소유한다. 따라서 root의 `cargo run --locked -- --seed 42`와 명시적 `cargo run --locked --bin aihack -- --seed 42`는 같은 binary를 선택한다. Headless는 default member가 아니므로 package selector `-p aihack-headless`를 함께 지정한다.

root 명령:

```bash
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo run --locked --bin aihack -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

dependency 경계:

```bash
cargo tree -p aihack-core
cargo tree -p aihack-content
cargo tree -p aihack-llm
```

`aihack-core` tree에 ratatui, crossterm, reqwest 또는 다른 HTTP client가 나타나면 실패다.

## 9. Local LLM 개발 실행

local LLM transport의 기본값은 disabled다. 아래처럼 provider를 활성화하면 TUI에서 `G` narrative, `A` suggestion, `J` soft judgment를 요청하고, suggestion은 `Y`로만 적용하며 `N` dismiss와 `R` retry를 사용할 수 있다.

```bash
AIHACK_LLM_ENABLED=true \
AIHACK_LLM_ENDPOINT=http://127.0.0.1:11434/v1 \
AIHACK_LLM_MODEL=local-model \
cargo run --locked --bin aihack -- --seed 42
```

접근성 수동 matrix는 다음 실행 플래그를 사용한다. 기본 최소 terminal 계약은 60x24다.

```bash
cargo run --locked --bin aihack -- --seed 42 --high-contrast --reduced-motion
```

R6 PTY 실행 증거와 fixture/실제-model 구분은 `docs/R6_MANUAL_MATRIX.md`에 기록한다.

감사 재현용 fixture와 PTY smoke는 외부 provider 없이 다음 명령으로 실행한다.

```bash
scripts/r6_pty_matrix.sh
scripts/r6_pending_exit_smoke.sh
```

첫 명령은 80x24 success, 60x24 timeout/down, 120x36 stale 흐름을 검사한다. 두 번째 명령은 pending request 중 Q 종료에서 alternate/raw terminal 복원이 worker grace wait보다 먼저 수행되고 전체 종료가 bounded인지 검사한다. Python 표준 라이브러리, tmux, 현재 debug TUI binary만 사용한다.

허용 환경변수:

| 이름 | 기본값 | 검사 |
| --- | --- | --- |
| AIHACK_LLM_ENABLED | false | true/false |
| AIHACK_LLM_ENDPOINT | http://127.0.0.1:11434/v1 | host가 127.0.0.1, localhost, [::1] 중 하나 |
| AIHACK_LLM_MODEL | empty | enabled=true면 1..=128자 |
| AIHACK_LLM_CONNECT_TIMEOUT_MS | 500 | 100..=5000 |
| AIHACK_LLM_NARRATIVE_TIMEOUT_MS | 2000 | 100..=10000 |
| AIHACK_LLM_DECISION_TIMEOUT_MS | 1500 | 100..=10000 |
| AIHACK_LLM_MAX_CHARS | 240 | 1..=240 |

API key는 기본 local flow에서 요구하지 않는다. endpoint URL에 credential을 넣지 않는다.

## 10. Clean-room 감사

기존 `target/`을 지우지 않고 별도 target을 사용한다.

```bash
CARGO_TARGET_DIR=/tmp/aihack-audit-target cargo check --workspace --all-targets --locked
CARGO_TARGET_DIR=/tmp/aihack-audit-target cargo test --workspace --all-targets --locked
CARGO_TARGET_DIR=/tmp/aihack-audit-target cargo build --workspace --release --locked
```

이 명령은 기존 `target/`과 분리된 clean-room target에서 현재 전체 workspace를 검사한다.

## 11. 런타임 산출물

```text
runtime/
  saves/
    *.json
  replays/
    *.jsonl
  reports/
    seed-*.json
  logs/
    aihack.log
```

- `runtime/`은 Git 추적 대상이 아니다.
- save/replay schema v1은 R8까지 유지한다.
- LLM prompt, response body, API credential은 save/replay/report에 기록하지 않는다.
- deterministic test는 고정 fixture path를 사용하고 사용자 runtime을 덮어쓰지 않는다.

## 12. 실패 진단 순서

1. 실패 명령을 그대로 한 번 재현
2. `rustc --version`, `cargo metadata --locked`, `cargo tree -d` 확인
3. 실패 test 하나만 `--exact --nocapture`로 실행
4. clean target에서 재실행
5. 첫 표적 수정이 실패하거나 version behavior가 의심될 때 공식 Rust/crate 문서 확인
6. 같은 실패를 세 번 반복하지 않고 오류, 변경, 남은 가설을 기록

## 13. 릴리즈 체크리스트

R7 engineering gate:

```bash
cargo test -p aihack --locked --test provenance_manifest
cargo test -p aihack --locked --test nethack_367_compat
cargo test -p aihack --locked --test golden_phase8_rules
scripts/r7_checkpoint.sh
! rg -n "legacy_nethack_port_reference" Cargo.toml crates apps src \
  --glob '*.toml' --glob '*.rs'
```

앞의 세 Cargo test PASS는 R7 engineering evidence다. `scripts/r7_checkpoint.sh`는 `PROVENANCE.md`의 모든 runtime record와 NH367 record 10개가 승인 근거를 갖춘 `Approved`인지 확인한다. 2026-07-20 프로젝트 소유자의 approval authority/evidence 반영 후 이 checkpoint는 PASS해야 한다.

R7 validator는 script가 위치한 repository만 검사하며 inherited environment로 root를 바꿀 수 없다. root distribution license와 packaging은 R8 소유이므로 R7 결과만으로 외부 배포할 수 없다.

R8 fail-closed preflight:

```bash
cargo test -p aihack --locked --test release_gate
scripts/r8_checkpoint.sh
```

R8 checkpoint도 script-relative canonical repository만 검사한다. 승인된 완전 fixture는 PASS(exit 0), R7 approval·0.3.0 version·whole-work NGPL 또는 release metadata가 빠지면 HOLD(exit 1), LICENSE checksum·NOTICE/source packaging 계약·dependency version·archive chain이 손상되면 FAIL(exit 2)다. HOLD/FAIL에서는 release artifact를 게시하지 않는다.

- [x] R1~R7 engineering 단계 완료, R7은 license review가 이관된 `PASS WITH KNOWN RISKS`
- [x] R8 fail-closed preflight와 canonical-root 회귀 테스트
- [x] R8 런칭 전 SC-LICENSE-01과 distribution license 로컬 gate PASS
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [x] `cargo test --workspace --all-targets --locked` — 전체 PASS (테스트 수는 회귀 추가에 따라 변동)
- [x] `cargo build --workspace --release --locked`
- [x] seed 42, 7, 1234 accepted turn 1000
- [x] save/load/replay v1 hash equality
- [x] provider disabled/timeout/stale에서 core hash 불변
- [x] provenance Unknown/Blocked runtime 자산 0건
- [x] Cargo/README/CHANGELOG 0.3.0 동기화
- [x] deterministic PTY core flow와 success/timeout/stale/down/pending-exit matrix PASS
- [x] Linux/Windows CI green — Actions run `29886410221`, commit `b9bd680200d82b20d7c9ba961a2758caa3d49e16`
- [x] `cargo audit` vulnerability 0건
- [x] `cargo deny check licenses bans sources` PASS
- [ ] 독립 R8 최종 PASS — `audit_report_19.md`의 technical evidence는 Verified, `audit_report_20.md`의 잔여 active-state/false-green HOLD 시정 후 재감사 대기

위 완료 항목은 `audit_report_19.md`의 독립 검증과 동일 SHA CI evidence에 근거한다. 외부 게시는 마지막 미완료 항목과 별도 사용자 승인이 모두 충족되기 전까지 수행하지 않는다.
