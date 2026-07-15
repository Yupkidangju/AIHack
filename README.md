# AIHack

> Current code: Cargo 0.1.0 · Refactoring target: 0.3.0 · Plan date: 2026-07-15

## 한국어

AIHack은 NetHack 3.6.7의 관찰 가능한 규칙을 시나리오와 테스트로 재구현하는 Rust 로그라이크다. 줄 단위 C-to-Rust 번역이 아니라 결정론적 core, 출처가 추적되는 호환 규칙, 격리된 local LLM adapter를 목표로 한다.

### 현재 상태

- fmt, clippy, 전체 test, release build는 현재 환경에서 통과한 기준선이다.
- Rust 1.94.1과 ratatui 0.30/crossterm 0.29 계열은 repository에 고정됐다. Linux/Windows CI의 첫 원격 결과는 대기 중이다.
- R1 build 재현성과 R2 private state/transaction은 local gate를 통과했다.
- R3는 runtime `ContentRegistry`까지 연결됐으나, malformed embedded content가 session bootstrap에서 `ContentError`가 아니라 panic으로 끝나는 경로가 남아 있다. 이 문제는 R3-4에서 닫는다.
- `--turns 1000`은 조기 사망 때문에 실제 accepted turn 1000개를 보장하지 않는다.
- LLM 코드는 trait/mock scaffold이며 실제 local provider, 강제 timeout, stale response gate는 아직 없다.
- NetHack 참조 코드·데이터는 provenance와 license scope 검토 전 runtime에 포함하지 않는다.

### v0.3.0 목표

- Rust 1.94.1, locked dependency, Linux/Windows CI
- private state와 transaction/invariant 기반 turn commit
- embedded TOML `ContentRegistry` 실연결
- accepted turn 1000개의 multi-seed deterministic 검증
- core/content/AI contract/LLM/TUI/headless workspace 분리
- loopback local LLM narrative, legal-action suggestion, presentation-only soft judgment
- NetHack 3.6.7 출처와 NH367 compatibility scenario 추적

### 현재 실행

```bash
cargo run --locked -- --seed 42
cargo run --locked --bin aihack-headless -- --seed 42 --turns 100
```

TUI는 `default-run = "aihack"`로 선택된다. headless 실행에는 계속 `--bin aihack-headless`를 명시한다.

### 구현 순서

1. R3-4 fallible content bootstrap
2. R4 true 1000-turn runner
3. R5 workspace boundaries
4. R6 local LLM adapter
5. R7 provenance and compatibility
6. R8 integrated release audit

### 문서

1. [Master specification](spec.md)
2. [Implementation tasks](IMPLEMENTATION_SUMMARY.md)
3. [Gap register](GAP_CLOSURE_ROADMAP.md)
4. [Runtime and UI design](designs.md)
5. [Architecture decisions](DESIGN_DECISIONS.md)
6. [Build guide](BUILD_GUIDE.md)
7. [Audit gates](audit_roadmap.md)
8. [Provenance policy](PROVENANCE.md)
9. [Compatibility template](docs/compatibility/README.md)
10. [Change history](CHANGELOG.md)
11. [R0 documentation audit](DOCUMENTATION_AUDIT_REPORT.md)
12. [Current implementation/document audit](audit_report_1.md)

과거 Phase 계획은 `.archive/`의 불변 snapshot으로 보존한다.

## English

AIHack is a Rust roguelike that reimplements observable NetHack 3.6.7 behavior through traced scenarios and tests. It targets a deterministic core, provenance-backed compatibility rules, and an isolated local-LLM adapter—not a line-by-line C-to-Rust translation.

### Current status

- The current baseline passes formatting, clippy, the full test suite, and a release build in the audited environment.
- Rust 1.94.1 and the ratatui 0.30/crossterm 0.29 family are pinned; the first Linux/Windows CI result is pending.
- R1 build reproducibility and R2 private-state transaction gates pass locally.
- R3 now uses the runtime `ContentRegistry`, but malformed embedded content can still panic during session bootstrap rather than return `ContentError`; R3-4 tracks this remaining issue.
- `--turns 1000` does not yet prove 1,000 accepted turns because the wait-only runner can die early.
- LLM modules are trait/mock scaffolding; live transport, enforced timeout, and stale-response rejection are planned for v0.3.0.
- NetHack-derived code or data remains excluded from runtime until provenance and license scope are approved.

### v0.3.0 target

- Rust 1.94.1, locked dependencies, Linux/Windows CI
- Private state and transaction/invariant turn commits
- Runtime-backed embedded TOML `ContentRegistry`
- Deterministic 1,000-accepted-turn multi-seed verification
- Core/content/AI-contract/LLM/TUI/headless workspace boundaries
- Loopback local-LLM narrative, legal-action suggestions, and presentation-only soft judgments
- Traceable NetHack 3.6.7 sources and NH367 compatibility scenarios

### Run the current code

```bash
cargo run --locked -- --seed 42
cargo run --locked --bin aihack-headless -- --seed 42 --turns 100
```

The TUI is selected through `default-run = "aihack"`; keep `--bin aihack-headless` for the headless binary. See the Korean section for the ordered document set.

## 日本語

AIHack は、NetHack 3.6.7 の観察可能な挙動を、出典付きシナリオとテストで再実装する Rust ローグライクです。C ソースの行単位変換ではなく、決定論的 core、追跡可能な互換ルール、隔離された local LLM adapter を目標にします。

### 現在の状態

- 現在の基準では format、clippy、全 test、release build が通過しています。
- Rust 1.94.1 と ratatui 0.30/crossterm 0.29 は固定済みで、R1 build と R2 private state/transaction、R3 runtime TOML registry はローカル検証済みです。R3 の malformed content bootstrap、1000 accepted turns、live local LLM、provenance gate は未完了です。
- LLM は core state を直接変更せず、v0.3.0 でも narrative、legal-action suggestion、effect なしの soft judgment に限定します。

### 現在の実行

```bash
cargo run --locked -- --seed 42
cargo run --locked --bin aihack-headless -- --seed 42 --turns 100
```

TUI は `default-run = "aihack"` で選択されます。headless binary には引き続き `--bin aihack-headless` を指定します。実装順序と文書一覧は韓国語セクションを参照してください。

## 繁體中文

AIHack 是以具來源追蹤的情境與測試，重新實作 NetHack 3.6.7 可觀察行為的 Rust roguelike。目標不是逐行翻譯 C，而是 deterministic core、可追蹤的相容規則，以及隔離的 local LLM adapter。

### 目前狀態

- 目前基準已通過 format、clippy、全部 test 與 release build。
- Rust 1.94.1 與 ratatui 0.30/crossterm 0.29 已固定；R1 build、R2 private state/transaction 與 R3 runtime TOML registry 已在本機驗證。R3 malformed content bootstrap、真正的 1000 accepted turns、live local LLM 與 provenance gate 尚未完成。
- v0.3.0 的 LLM 只提供 narrative、合法行動建議與不產生 core effect 的 soft judgment。

### 目前執行

```bash
cargo run --locked -- --seed 42
cargo run --locked --bin aihack-headless -- --seed 42 --turns 100
```

TUI 由 `default-run = "aihack"` 選取；headless binary 仍需指定 `--bin aihack-headless`。實作順序與文件列表請參閱韓文段落。

## 简体中文

AIHack 是通过带来源追踪的场景和测试，重新实现 NetHack 3.6.7 可观察行为的 Rust roguelike。目标不是逐行翻译 C，而是 deterministic core、可追踪的兼容规则，以及隔离的 local LLM adapter。

### 当前状态

- 当前基线已通过 format、clippy、全部 test 和 release build。
- Rust 1.94.1 和 ratatui 0.30/crossterm 0.29 已固定；R1 build、R2 private state/transaction 与 R3 runtime TOML registry 已在本地验证。R3 malformed content bootstrap、真正的 1000 accepted turns、live local LLM 和 provenance gate 尚未完成。
- v0.3.0 的 LLM 仅提供 narrative、合法行动建议和不产生 core effect 的 soft judgment。

### 当前运行

```bash
cargo run --locked -- --seed 42
cargo run --locked --bin aihack-headless -- --seed 42 --turns 100
```

TUI 由 `default-run = "aihack"` 选择；headless binary 仍需指定 `--bin aihack-headless`。实现顺序和文档列表请参阅韩文部分。
