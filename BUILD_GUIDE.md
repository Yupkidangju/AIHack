# AIHack Build Guide

문서 상태: active
작성일: 2026-04-28

## 1. 현재 루트 상태

현재 루트에는 새 엔진 코드가 없다. 이전 Rust 포트는 `legacy_nethack_port_reference/` 아래에 보존되어 있다.

새 코드 작성 전 기준 파일:

- `AGENTS.md`
- `AI_IMPLEMENTATION_DOC_STANDARD.md`
- `README.md`
- `spec.md`
- `designs.md`
- `implementation_summary.md`
- `DESIGN_DECISIONS.md`
- `audit_roadmap.md`
- `CHANGELOG.md`

## 2. 사전 준비

필수:

- Rust stable
- Cargo

확인:

```bash
rustc --version
cargo --version
```

## 3. 새 Cargo 스캐폴딩 절차

루트에서 새 Cargo 패키지를 만든다. 레거시 폴더는 workspace member로 넣지 않는다.

필수 `Cargo.toml` 초안:

```toml
[package]
name = "aihack"
version = "0.1.0"
edition = "2021"
license = "UNLICENSED"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
rand = "0.8"
clap = { version = "4", features = ["derive"] }

[[bin]]
name = "aihack"
path = "src/main.rs"

[[bin]]
name = "aihack-headless"
path = "src/bin/aihack-headless.rs"
```

`license = "UNLICENSED"`는 초기 개발 중 임시값이다. NetHack 파생 여부가 결정되면 별도 라이선스 결정을 `DESIGN_DECISIONS.md`에 추가한다.

## 4. 필수 엔트리와 현재 구현 상태

`src/main.rs`:

```rust
fn main() {
    println!("AIHack UI adapter is not implemented yet. Use aihack-headless.");
}
```

`src/bin/aihack-headless.rs`:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 1000)]
    turns: u64,
}

fn main() {
    let args = Args::parse();
    println!("seed={} turns={}", args.seed, args.turns);
}
```

이 엔트리는 Phase 1에서 실제 `GameSession::new(seed)`와 `CommandIntent::Wait` 실행으로 교체되었다. 현재 runner는 `seed`, `turns`, `final_turn`, `final_hash`를 출력한다.

## 5. 런타임 출력 경로

초기 구현은 다음 경로를 사용한다.

```text
runtime/
  save/
    dev_save.json
  replays/
    {seed}-{turns}.jsonl
  snapshots/
    {seed}-{turn}.json
  logs/
    headless.log
```

`runtime/`은 git 추적 대상이 아니다. 필요 시 `.gitignore`에 추가한다.

## 6. 검증 명령

기본:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --test levels
cargo test --test stairs
cargo run --bin aihack-headless -- --seed 42 --turns 0
```

Phase 1 이후:

```bash
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

Phase 5 이후:

```bash
cargo run --bin aihack-headless -- --seed 42 --turns 1000
```

품질 게이트:

```bash
cargo clippy --all-targets -- -D warnings
```

## 7. 첫 실행 산출물

Phase 5 현재 기준 출력 예시:

```text
seed=42 turns=0 final_turn=0 final_hash=821520dc302c9ea2
seed=42 turns=100 final_turn=100 final_hash=88886c28698a1730
seed=43 turns=100 final_turn=100 final_hash=948c5ec460bebb99
```


`--turns 0`:

- stdout에 seed/turns 표시
- exit code 0

`--turns 1000`:

- `runtime/replays/42-1000.jsonl` 생성
- 마지막 줄에 final snapshot hash 포함
- 같은 명령을 두 번 실행하면 final hash 동일

## 8. 레거시 빌드

이전 포트를 빌드해야 할 경우:

```bash
cd legacy_nethack_port_reference
cargo test
```

이 명령은 새 엔진 검증이 아니다. 레거시 상태 확인용이다.

## 9. 금지 사항

- `legacy_nethack_port_reference`를 workspace member로 추가하지 않는다.
- 새 `src/`에서 `legacy_nethack_port_reference/src`를 path import하지 않는다.
- UI 구현 전 core 없이 빈 화면만 만드는 작업을 완료로 보지 않는다.
- `cargo test` 없이 Phase 완료를 주장하지 않는다.

## 10. 배포 전 체크리스트

v0.1 release candidate:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- headless 1000턴 seed 42/7/1234 통과
- save/load hash 일치
- replay 재생 hash 일치
- `Observation` schema snapshot 통과
- 루트 문서와 구현 타입 이름 일치
