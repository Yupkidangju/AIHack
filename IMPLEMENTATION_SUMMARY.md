# AIHack Implementation Summary v2

> Archive chain
> - Latest: `.archive/IMPLEMENTATION_SUMMARY_archive_260715.md`
> - Previous: first archive
>
> Phase 1~20 구현 이력은 아카이브에 있다. 이 문서는 v0.3.0 구현 시작점과 작업 순서만 정의한다.

문서 상태: active implementation plan
작성일: 2026-07-15
기준 스펙: `spec.md`
기준 계획: `GAP_CLOSURE_ROADMAP.md`

## 1. 현재 기준과 목표

R1~R8 기존 시정과 report 23/24 finding은 `docs/audit/audit_report_24.md`, 후속 remediation, implementation SHA `2519bc8e0ede81c39f46b5778e62a41d4ca66901`의 Actions run `32107862171`에서 역사적으로 종결됐다. report 23의 독립 재감사 대기는 현재 gate가 아니다.

현재 권위는 report 25 same-SHA 구현을 독립 production probe로 다시 HOLD한 `docs/audit/audit_report_26.md`다. report 25 SHA `b732c42d62f295f4d8be64480c1d0a5a440fe738`와 Actions `32650404618`은 역사적 positive/partial evidence로 보존한다. report 26 시정은 consumer-safe save/산술, Windows alias, actual causal producer pair/removal, modal mouse·Inspect presentation, fresh release staging/link count/candidate date와 dependency/action gate를 ADR-0036 계약에 따라 정렬한다.

report 26 finding별 수정 전 RED와 표적 GREEN은 `docs/audit/audit_report_26_remediation.md`에 기록한다. 현재 표적 회귀는 GREEN으로 전환됐지만 전체 로컬 gate와 새 clean same-SHA Ubuntu/Windows actual bundle은 아직 실행 전이므로 program 또는 외부 게시 가능 상태로 올리지 않는다.

실제 model provider smoke는 필수 release gate가 아니다. 최종 통합에서 추가 호환성 증거가 반드시 필요할 때만 localhost OpenAI-compatible 임시 adapter를 통해 원격 provider를 선택 검증한다.

## 2. 전체 런타임 흐름

```text
TUI / Headless / LLM adapter
        |
        v
Observation + ActionSpace
        |
        v
CommandIntent
        |
        v
GameSession::submit()
  -> validate revision/state/action
  -> create TurnTransaction
  -> apply player/tile/monster/status/death systems
  -> validate 6 world invariants
  -> commit events
  -> hash snapshot
        |
        +--> Observation
        +--> ReplayLineV1
        +--> presentation-only LLM request
```

LLM은 마지막 presentation branch에서만 호출한다. core turn은 LLM 응답을 기다리지 않는다.

## 3. 시스템 분해와 파일 책임

### 3.1 R5 현재 구조와 compatibility facade

| 경로 | 현재 책임 |
| --- | --- |
| `crates/aihack-core/src/*` | 순수 domain, 결정론 규칙, generic state/save/transaction primitive |
| `crates/aihack-content/src/*` | embedded schema, TOML asset, registry 검증 |
| `crates/aihack-ai-contract/src/*` | read-only Observation/ActionSpace/ClientRevision 계약 |
| `crates/aihack-llm/src/*` | loopback transport, strict response validation, bounded worker, provider fallback와 R6 decision scaffold |
| `crates/aihack-runtime/src/*` | content-aware world/session/system, production score pair, projection, bounded save/replay `ArtifactStore` I/O |
| `apps/aihack-tui/src/*` | Observation render, 단일 state-aware event dispatcher, render-derived CTA geometry, terminal lifecycle, `aihack` binary |
| `apps/aihack-headless/src/*` | policy 실행, normalized/file-identity replay guard, report/replay CLI, `aihack-headless` binary |
| `src/*` | 기존 public module path를 유지하는 root compatibility facade |
| `tests/support/*` | 공개 필드 대입을 대체하는 fixture builder |

### 3.2 R5 구현 workspace

| crate/app | 허용 의존 |
| --- | --- |
| root `aihack` facade | core, content, AI contract, LLM/TUI는 test compatibility에만 re-export |
| `aihack-core` | serde, thiserror, rand |
| `aihack-content` | aihack-core ID/domain contract, serde, toml |
| `aihack-ai-contract` | aihack-core read-only DTO |
| `aihack-llm` | aihack-ai-contract, reqwest 0.13.4 blocking/json |
| `aihack-runtime` | core, content, AI contract; `GameClient` 구현과 save/bootstrap |
| `aihack-tui` | runtime, AI contract, LLM, ratatui, crossterm |
| `aihack-headless` | runtime, AI contract |

core Cargo manifest에 ratatui, crossterm, HTTP client가 나타나면 R5 실패다.

## 4. 경계 계약 요약

- mutable entry: `GameSession::submit(CommandIntent) -> TurnOutcome` 한 개
- read entries: snapshot, observation (내부 action_space 포함), seed, turn, run_state
- content entry: `ContentRegistry::from_embedded() -> Result<ContentRegistry, ContentError>`
- LLM entry: `LocalLlmService::request(context, kind)`
- replay truth: command, deterministic outcome, snapshot hash
- presentation truth: narrative, rationale, soft verdict
- stale response: request revision과 current revision이 다르면 폐기

자세한 타입은 `spec.md` 9절을 그대로 사용한다.

## 5. 알고리즘 메모

### 5.1 Turn transaction

1. session revision을 캡처한다.
2. 현재 action space에 command가 있는지 검사한다.
3. working state에 시스템을 순서대로 적용한다.
4. invariant 6개를 검사한다.
5. 오류가 있으면 working state와 RNG draw를 폐기한다.
6. 오류가 없으면 event log, world, RNG, turn을 한 번에 commit한다.

R2 구현에서는 behavior와 snapshot field order를 바꾸지 않는다. hash 변경이 발생하면 R2 gate는 실패다.

### 5.2 Content hash

- items, monsters, levels를 ID 오름차순으로 정렬한다.
- canonical JSON으로 serialize한다.
- FNV-1a 64-bit를 적용해 16자리 lowercase hex를 만든다.
- 같은 embedded content는 platform과 실행 순서에 관계없이 같은 hash를 가져야 한다.

### 5.3 LLM revision validation

```text
response.request_id == request.request_id
response.revision.turn == session.turn()
response.revision.snapshot_hash == session.snapshot().stable_hash()
proposal.action in session.action_space()
```

네 조건 중 하나라도 false면 status는 Invalid 또는 Stale이며 `submit`을 호출하지 않는다.

### 5.4 1000-turn run

`survival-v1`은 `spec.md` 12절 우선순위를 사용한다. submitted command가 reject되면 같은 turn에서 다음 legal candidate를 시도하며, 한 turn 최대 시도는 16회다. 16회 안에 accepted command가 없으면 report는 `NoAcceptedAction`으로 실패한다.

## 6. 동결된 공식

```text
attack_roll = d20 + attacker.hit_bonus + weapon.hit_bonus
defense = 10 + defender.ac
hit = attack_roll >= defense
damage = max(1, damage_roll + attacker.damage_bonus - defender.damage_reduction)
vision_radius = 8
trap_pit_damage = 3
```

리팩터링 Task는 이 공식을 변경하지 않는다.

## 7. 구현 Task

각 Task는 단독 세션에서 완료 가능한 크기다. Task 종료 시 지정 검증이 통과하지 않으면 다음 Task로 진행하지 않는다.

### Task R0-1: 제품·gap·실행 계약

**설명:** v0.3.0 목표, typed contract, gap과 구현 Task를 폐쇄한다.

**수용 기준:**

- [x] 목표·성공·비목표와 DEC ID가 동결됨
- [x] 모든 active gap이 Task와 SC ID에 매핑됨
- [x] Task별 파일, 선행, 수용 기준, 검증 명령이 정의됨

**검증:** `audit_roadmap.md` R0 3.2의 SC/Task ID 검사
**선행:** 없음
**파일:** `spec.md`, `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`
**범위:** M, 문서 3개

### Task R0-2: 결정·UI·빌드 계약

**설명:** architecture decision, LLM UI state, 현재/target build 명령을 동기화한다.

**수용 기준:**

- [x] ADR-0021..ADR-0027에 context/decision/alternatives/consequences가 있음
- [x] CTA ID, 활성 조건, 결과, degraded state가 정의됨
- [x] 현재와 target 명령·artifact·CI가 분리됨

**검증:** `audit_roadmap.md` R0 3.4의 항목 2, 3, 7, 9, 10
**선행:** R0-1
**파일:** `DESIGN_DECISIONS.md`, `designs.md`, `BUILD_GUIDE.md`
**범위:** M, 문서 3개

### Task R0-3: 출처·탐색·감사 계약

**설명:** README, changelog, provenance, compatibility template, final audit를 동기화한다.

**수용 기준:**

- [x] 공식 3.6.7 source URL과 SHA-256이 기록됨
- [x] Unknown/Reviewed/Approved/Blocked의 runtime 규칙이 정의됨
- [x] README와 changelog가 구현 미완료를 명시함
- [x] R0 자동·수동 문서 감사가 PASS

**검증:** `audit_roadmap.md` R0 3.1..3.4 전체, `DOCUMENTATION_AUDIT_REPORT.md`
**선행:** R0-2
**파일:** `PROVENANCE.md`, `docs/compatibility/README.md`, `README.md`, `CHANGELOG.md`, `audit_roadmap.md`
**범위:** M, 문서 5개

### Task R1-1: Rust와 dependency baseline 고정

**설명:** toolchain과 UI dependency를 검증된 한 계열로 고정한다.

**수용 기준:**

- [x] `rust-toolchain.toml`은 channel 1.94.1
- [x] Cargo package에 `rust-version = "1.94"`와 `default-run = "aihack"`
- [x] ratatui 0.30, crossterm 0.29
- [x] `cargo tree -d`에 crossterm 중복 0건

**검증:**

```bash
cargo metadata --locked --no-deps --format-version 1
cargo tree -d
cargo check --workspace --all-targets --locked
```

**선행:** R0-3
**파일:** `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`
**범위:** M, 3개

### Task R1-2: 빌드 스크립트 fail-fast

**설명:** test와 artifact 검증이 없는 성공 출력을 제거한다.

**수용 기준:**

- [x] `--test`는 locked test 실행
- [x] debug/release build는 `--locked` 사용
- [x] 두 binary 중 하나라도 없으면 exit code 1
- [x] Linux/Windows script 동작 계약 동일

**검증:**

```bash
./build.sh --test
test -x output/aihack
test -x output/aihack-headless
```

Windows는 `build.bat --test` 후 두 exe 존재를 확인한다.

**선행:** R1-1
**파일:** `build.sh`, `build.bat`, `tests/build_contract.rs`
**범위:** M, 3개

### Task R1-3: CI quality gate

**설명:** Linux와 Windows에서 동일한 locked gate를 자동 실행한다.

**수용 기준:**

- [x] push와 pull request에서 실행하도록 구성
- [x] fmt, clippy -D warnings, all-target tests, release build 포함
- [x] Ubuntu/Windows에서 각 플랫폼 release packaging script를 실제 실행
- [x] lockfile 변경 여부를 job 종료 시 확인
- [x] cargo-audit 0.22.1, cargo-deny 0.19.4를 pinned install
- [x] vulnerability, license, source, crossterm duplicate gate 포함
- [x] Linux/Windows 원격 CI green 및 실패 단계 로그 확인 — `b9bd680200d82b20d7c9ba961a2758caa3d49e16`, Actions run `29886410221`

**검증:**

- workflow YAML parse
- 로컬에서 workflow와 동일한 5개 명령 통과

**선행:** R1-2
**파일:** `.github/workflows/ci.yml`, `deny.toml`, `BUILD_GUIDE.md`
**범위:** S, 3개

### Checkpoint R1

- [x] SC-BUILD-01 PASS (R1 local audit 통과)
- [x] SC-BUILD-02 PASS — 동일 SHA Linux/Windows quality gate와 release bundle PASS (`29886410221`)
- [x] README quick start를 default-run 계약에 맞춤
- [x] package/version 상태가 문서와 일치

### Task R2-1: GameSession 캡슐화

**설명:** session 필드를 private으로 만들고 query API와 test builder를 도입한다.

**진행 상태 (2026-07-15):** 완료. `meta`, `rng`, `turn`, `state`, `world`, `event_log`은 crate 외부에서 보이지 않게 전환했고, `seed()`, `turn()`, `run_state()`, `event_log()`, `world()` 읽기 API를 제공한다. headless/TUI 생산 경로와 integration test 조회는 이 API를 사용하며, 특수 상태 구성은 저장 스키마 기반 `aihack::testing::SessionBuilder` fixture 경계에서만 수행한다.

**수용 기준:**

- [x] 외부 module의 session field 직접 대입 0건
- [x] seed, turn, state, snapshot, observation getter 존재
- [x] 기존 save/load와 TUI 동작 유지
- [x] test fixture는 builder로만 상태 구성

**검증:**

```bash
rg -n "session\.(meta|rng|turn|state|world|event_log)\s*=" src tests
cargo test -p aihack --locked --test save_load --test ui_runtime_smoke
```

첫 명령 결과는 `tests/support` 내부 허용 목록 외 0건이다.

**선행:** R1 checkpoint
**현재 파일:** `crates/aihack-runtime/src/session.rs`, `crates/aihack-runtime/src/save.rs`, `tests/support/session_builder.rs`, `tests/support/mod.rs`
**범위:** M, 4개

### Task R2-2: GameWorld 캡슐화와 invariant

**설명:** world public field 변경을 typed API로 교체하고 6개 invariant를 검사한다.

**진행 상태 (2026-07-15):** 완료. `WorldInvariantError` 6종과 `InvariantReport`를 도입했고, 정상 fixture와 save-schema로 구성한 각 위반 상태를 `tests/world_invariants.rs`로 검증한다. `GameWorld.levels`, `entities`, `inventory`와 status·score·식별·사망원인·player identity 필드는 crate 외부 비공개이며, 조회는 `levels()`, `entities()`, `inventory()` 및 typed getter로 제공한다. integration test의 상태 변경은 `aihack::testing::SessionBuilder`를 통해 저장 스키마를 재구성한다. 모든 submit은 transaction validation을 거쳐 invariant 오류 시 원본 world/turn/RNG/snapshot을 유지한다.

**수용 기준:**

- [x] `WorldInvariantError` 6종 구현
- [x] accepted turn마다 `InvariantReport.checked == 6`
- [x] UI/LLM/test에서 world field 직접 대입 0건
- [x] invariant failure는 no-commit/no-turn

**검증:**

```bash
cargo test -p aihack --locked --test world_invariants
cargo test -p aihack --locked --test levels --test inventory --test save_load
```

**선행:** R2-1
**현재 파일:** `crates/aihack-runtime/src/world.rs`, `crates/aihack-core/src/world.rs`, `crates/aihack-core/src/invariant.rs`, `tests/world_invariants.rs`
**범위:** M, 4개

### Task R2-3: submit transaction 분리

**설명:** `accept_turn`의 mutation을 prepare, validate, commit 단계로 나눈다.

**진행 상태 (2026-07-15):** 구현 및 동작 검증 완료. `TurnTransaction`이 cloned working copy에서 명령을 적용하고, 6개 invariant를 검증한 뒤에만 원본 session을 교체한다. invariant 오류는 `accepted=false` outcome으로 projection하며 turn, snapshot hash, RNG state를 보존한다. 기존 AwaitingDirection의 reject 후 Playing 복귀 계약도 유지한다.

**수용 기준:**

- [x] `TurnTransaction` 구현
- [x] player/monster/status/death 순서 유지
- [x] 기존 golden hash 유지
- [x] reject와 invariant error에서 RNG draws 유지
- [x] accepted-bool `TurnOutcome` replay wire shape와 save/load continuation 유지

**검증:**

```bash
cargo test -p aihack --locked --test combat --test monster_ai --test replay
cargo test -p aihack --locked --test golden_phase8_rules
```

**선행:** R2-2
**현재 파일:** `crates/aihack-runtime/src/session.rs`, `crates/aihack-runtime/src/transaction.rs`, `crates/aihack-core/src/rng.rs`, `tests/transaction.rs`, `tests/replay.rs`
**범위:** M, 5개

### Checkpoint R2

- [x] SC-CORE-01 PASS (R2 local gate: UI/LLM/integration test의 직접 대입 0건)
- [x] SC-CORE-02 PASS (accepted command transaction validation, invariant 오류 no-commit)
- [x] Phase 20 baseline hash 유지 (P8-G01..G20, replay/save regression 통과)
- [x] public mutable state type-level 완전 은닉

### Task R3-1: Content schema와 validator

**설명:** panic 기반 TOML parsing을 typed error와 registry validation으로 교체한다.

**상태:** 구현 완료. `ContentRegistry`는 schema v1 embedded TOML을 `OnceLock`으로 한 번 parse·검증하며 canonical FNV-1a hash를 제공한다. malformed/injected content의 session bootstrap은 R3-4에서 `ContentError`로 반환한다.

**수용 기준:**

- [x] ContentError 6종 구현
- [x] duplicate ID, unknown reference, invalid dice/coordinate 테스트
- [x] schema_version 1
- [x] canonical content hash 안정

**검증:**

```bash
cargo test -p aihack --locked --test content_validation
cargo test -p aihack --locked --test data_loading
```

**선행:** R2 checkpoint
**현재 파일:** `crates/aihack-content/src/schema.rs`, `crates/aihack-content/src/lib.rs`, `crates/aihack-core/src/error.rs`, `tests/content_validation.rs`
**범위:** M, 4개

### Task R3-2: Item과 monster registry 실연결

**설명:** hardcoded factory가 registry definition을 사용하게 한다.

**상태:** 구현 완료. `item_data`와 `monster_template`은 registry ID 조회 및 typed conversion을 사용한다. R3-4의 injected registry 경로는 초기 entity 생성에도 같은 registry를 사용한다.

**수용 기준:**

- [x] dagger/jackal 값이 TOML에서 생성
- [x] unknown ID는 ContentError
- [x] 기존 combat/item golden test 유지
- [x] runtime에서 `load_items/load_monsters` 중복 parsing 0회

**검증:**

```bash
cargo test -p aihack --locked --test items --test combat --test monster_ai
cargo test -p aihack --locked --test content_runtime
```

**선행:** R3-1
**현재 파일:** `crates/aihack-content/src/schema.rs`, `crates/aihack-runtime/src/domain/item.rs`, `crates/aihack-runtime/src/domain/monster.rs`, `crates/aihack-runtime/src/domain/entity.rs`, `tests/content_runtime.rs`
**범위:** M, 5개

### Task R3-3: Level registry 실연결

**설명:** main:1과 main:2를 TOML level definition에서 생성한다.

**상태:** 구현 완료. level map, player start, stairs 및 TOML에 선언된 monster/item 배치는 registry definition에서 생성하며, bootstrap 오류는 R3-4에서 fallible result로 전환됐다.

**수용 기준:**

- [x] 시작 위치, stairs, door, monster, item이 registry에서 생성
- [x] map bounds와 stairs pair 검증
- [x] 왕복 후 level state 유지
- [x] hardcoded fixture는 test builder로 이동

**검증:**

```bash
cargo test -p aihack --locked --test map --test levels --test stairs
cargo test -p aihack --locked --test content_runtime
```

**선행:** R3-2
**현재 파일:** `crates/aihack-content/src/schema.rs`, `crates/aihack-content/src/data/levels/main_2.toml`, `crates/aihack-runtime/src/world.rs`, `tests/content_runtime.rs`
**범위:** M, 4개

### Task R3-4: Content bootstrap 오류 경계 정렬

**설명:** embedded content 검증 실패가 `GameSession`/world bootstrap에서 panic이 아니라 `ContentError`로 반환되도록 전환하고, public registry API를 `spec.md` 9.3 계약과 정렬한다.

**수용 기준:**

- [x] malformed/injected content에서 fallible session/world bootstrap이 `Err(ContentError)`를 반환
- [x] TUI/headless production registry 생성 경로에 `expect`/`panic!` 0건
- [x] `from_toml_sources`의 test/import public boundary 이유를 `spec.md`에 기록
- [x] `ContentRegistry` String ID와 query surface를 `spec.md` 9.3 계약에 일치

**검증:**

```bash
cargo test -p aihack --locked --test content_validation --test content_runtime
rg -n 'registry\(\).*expect|try_item_data\(kind\)\.expect|try_monster_template\(kind\)\.expect' \
  src/core src/data src/domain
```

**선행:** R3-1..R3-3
**현재 파일:** `crates/aihack-content/src/schema.rs`, `crates/aihack-content/src/lib.rs`, `crates/aihack-runtime/src/session.rs`, `crates/aihack-runtime/src/world.rs`, `crates/aihack-runtime/src/domain/item.rs`, `crates/aihack-runtime/src/domain/monster.rs`, `tests/content_validation.rs`, `tests/content_runtime.rs`
**범위:** M, 8개

### Checkpoint R3

- [x] SC-DATA-01 PASS (R3-4 완료)
- [x] registry parse는 process당 1회
- [x] content hash 3회 동일
- [x] injected invalid content 검증 path panic 0건
- [x] production bootstrap invalid content path panic 0건

### Task R4-1: Headless policy와 report

**설명:** wait-only loop를 policy 기반 accepted-turn runner로 교체한다.

**수용 기준:**

- [x] `wait-v1`, `survival-v1`, `replay-file` 지원
- [x] replay-file은 `--replay-in` 필수이며 input/output canonical path 충돌 거부
- [x] report에 requested/accepted/submitted/final state/hash 포함
- [x] `--turns`는 absolute target turn, load run accepted 수는 target-current
- [x] save/load/replay/report path traversal와 symlink escape 거부
- [x] 16회 안에 accepted action이 없으면 명시적 실패
- [x] GameOver 조기 종료를 성공으로 출력하지 않음

**검증:**

```bash
cargo test -p aihack --locked --test headless_policy
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

**선행:** R3 checkpoint
**현재 파일:** `apps/aihack-headless/src/main.rs`, `apps/aihack-headless/src/lib.rs`, `crates/aihack-runtime/src/save.rs`, `tests/headless_policy.rs`, `tests/headless_paths.rs`
**범위:** M, 5개

### Task R4-2: 실제 장기 결정론 테스트

**설명:** 세 seed에서 실제 accepted turn 1000개와 반복 hash를 검증한다.

**수용 기준:**

- [x] seed 42, 7, 1234 모두 accepted_turns 1000
- [x] 각 seed 3회 hash 동일
- [x] 실패 report에 seed, turn, command index 포함
- [x] save/load continuation이 direct run과 동일

**검증:**

```bash
cargo test -p aihack --locked --test long_run --release
cargo test -p aihack --locked --test save_load --test replay
```

**선행:** R4-1
**파일:** `tests/long_run.rs`, `tests/release_candidate.rs`, `tests/save_load.rs`, `tests/replay.rs`
**범위:** M, 4개

### Checkpoint R4

- [x] SC-TEST-01 PASS (`tests/long_run.rs`: three seeds × three deterministic runs)
- [x] SC-TEST-02 PASS (`tests/save_load.rs`, `tests/replay.rs`)
- [x] 기존 release_candidate의 조기 사망 성공 기준 제거
- [x] long-run report 3개 생성 (`runtime/reports/r4-seed-{42,7,1234}.json`, ignored runtime artifacts)

### Task R5-1: Core와 content crate 추출

**설명:** behavior 변경 없이 core와 content를 workspace crate로 이동한다.

**수용 기준:**

- [ ] `aihack-core`에 UI/HTTP dependency 0건
- [ ] `aihack-content`가 registry를 소유
- [ ] public API fixture와 hash 유지
- [ ] 기존 integration tests 경로만 조정

**검증:**

```bash
cargo tree -p aihack-core
cargo test --workspace --locked
```

**선행:** R4 checkpoint

이동 slice는 순서를 바꾸지 않는다. `source -> target`은 한 파일 이동으로 계산하며 각 slice 뒤 `cargo check --workspace --all-targets --locked`를 실행한다.

**진행 상태 (R5-1):** 완료. R5-1A workspace skeleton과 `action`/`ids`/`position`/`rng`/`GameMeta`/`RunState`/`SnapshotHash`/`TurnOutcome`, `tile`/`combat`/`player`/`status`, `inventory`, item 및 monster 데이터 계약, `error`, `event`, `map`, `level`, `EntityStore` 저장·조회·변이 본체, world invariant 계약·검증, score·luck·hallucination 규칙, map 기반 LOS·가시 타일 계산, door 상태 전이, hidden tile reveal, combat 주사위·피해 계산을 `aihack-core`로 옮겼다. `WorldState<E>`, generic save DTO, `SessionState<W>`, cloned working-copy transaction과 death run-state 결정도 core가 소유한다. root의 `EntityStore`는 embedded content를 조회하는 기본 item/monster 생성 API만 호환 wrapper로 유지하고, root `GameWorld`는 content bootstrap 및 runtime adapter를 제공한다. content의 `LevelData`는 core `MapLayout` trait을 구현하고 world는 content layout iterator를 adapter로 전달한다. `aihack-content`가 schema Rust 파일, embedded registry, TOML asset을 물리적으로 단독 소유하고 item/monster 변환, level tile override, typed spawn plan을 제공하며 contract test로 schema v1, 필수 ID, hash `c491b83c6f499a62`를 검증한다. root `data`와 item/monster factory는 compatibility facade다. 기본 world fixture도 fallible production bootstrap으로 단일화해 content 검증을 우회하지 않는다. `tests/workspace_boundaries.rs`가 core의 UI/HTTP 무의존, content→core 단방향, content physical ownership 및 entity 저장소의 content-factory 무의존을 회귀 검사한다. core dependency tree, content/save/hash regression, headless CLI smoke를 통과했다.

| Slice | 파일, 최대 5개 |
| --- | --- |
| R5-1A | `Cargo.toml`, `crates/aihack-core/Cargo.toml`, `crates/aihack-core/src/lib.rs`, `crates/aihack-content/Cargo.toml`, `crates/aihack-content/src/lib.rs` |
| R5-1B | `src/core/action.rs -> crates/aihack-core/src/action.rs`, `src/core/error.rs -> crates/aihack-core/src/error.rs`, `src/core/event.rs -> crates/aihack-core/src/event.rs`, `src/core/ids.rs -> crates/aihack-core/src/ids.rs`, `src/core/position.rs -> crates/aihack-core/src/position.rs` |
| R5-1C | `src/core/rng.rs -> crates/aihack-core/src/rng.rs`, `src/core/save.rs -> crates/aihack-core/src/save.rs`, `src/core/session.rs -> crates/aihack-core/src/session.rs`, `src/core/snapshot.rs -> crates/aihack-core/src/snapshot.rs`, `src/core/turn.rs -> crates/aihack-core/src/turn.rs` |
| R5-1D | `src/core/world.rs -> crates/aihack-core/src/world.rs`, `src/core/invariant.rs -> crates/aihack-core/src/invariant.rs`, `src/core/transaction.rs -> crates/aihack-core/src/transaction.rs`, `src/core/policy.rs -> crates/aihack-core/src/policy.rs`, `src/core/mod.rs -> crates/aihack-core/src/core.rs` |
| R5-1E | `src/domain/combat.rs -> crates/aihack-core/src/domain/combat.rs`, `src/domain/entity.rs -> crates/aihack-core/src/domain/entity.rs`, `src/domain/inventory.rs -> crates/aihack-core/src/domain/inventory.rs`, `src/domain/item.rs -> crates/aihack-core/src/domain/item.rs`, `src/domain/level.rs -> crates/aihack-core/src/domain/level.rs` |
| R5-1F | `src/domain/map.rs -> crates/aihack-core/src/domain/map.rs`, `src/domain/mod.rs -> crates/aihack-core/src/domain/mod.rs`, `src/domain/monster.rs -> crates/aihack-core/src/domain/monster.rs`, `src/domain/player.rs -> crates/aihack-core/src/domain/player.rs`, `src/domain/status.rs -> crates/aihack-core/src/domain/status.rs` |
| R5-1G | `src/domain/tile.rs -> crates/aihack-core/src/domain/tile.rs`, `src/systems/combat.rs -> crates/aihack-core/src/systems/combat.rs`, `src/systems/death.rs -> crates/aihack-core/src/systems/death.rs`, `src/systems/doors.rs -> crates/aihack-core/src/systems/doors.rs`, `src/systems/items.rs -> crates/aihack-core/src/systems/items.rs` |
| R5-1H | `src/systems/mod.rs -> crates/aihack-core/src/systems/mod.rs`, `src/systems/monster_ai.rs -> crates/aihack-core/src/systems/monster_ai.rs`, `src/systems/movement.rs -> crates/aihack-core/src/systems/movement.rs`, `src/systems/projectiles.rs -> crates/aihack-core/src/systems/projectiles.rs`, `src/systems/score.rs -> crates/aihack-core/src/systems/score.rs` |
| R5-1I | `src/systems/stairs.rs -> crates/aihack-core/src/systems/stairs.rs`, `src/systems/traps.rs -> crates/aihack-core/src/systems/traps.rs`, `src/systems/vision.rs -> crates/aihack-core/src/systems/vision.rs` |
| R5-1J | `src/data/mod.rs -> crates/aihack-content/src/registry.rs`, `src/data/schema.rs -> crates/aihack-content/src/schema.rs`, `src/data/items.toml -> crates/aihack-content/src/data/items.toml`, `src/data/monsters.toml -> crates/aihack-content/src/data/monsters.toml` |
| R5-1K | `src/data/levels/main_1.toml -> crates/aihack-content/src/data/levels/main_1.toml`, `src/data/levels/main_2.toml -> crates/aihack-content/src/data/levels/main_2.toml` |

**범위:** 11개 sequential slice, 각 S 또는 M, slice당 5개 이하

### Task R5-2: AI contract와 adapter app 추출

**설명:** Observation/ActionSpace를 AI contract crate로, TUI/headless를 app으로 이동한다.

**진행 상태 (R5-2A~H):** 완료. `aihack-ai-contract`, `aihack-llm`, `aihack-runtime`, `apps/aihack-tui`, `apps/aihack-headless` workspace package를 생성했다. contract는 현재 turn과 snapshot hash를 담는 `ClientRevision`, `Observation`/`ActionSpace` 및 관련 DTO·read-only action type을 공개하며 mutable world/session은 노출하지 않는다. LLM의 decision/narrative request·fallback·provider 정책은 `aihack-llm`으로 이동했고, `GameSession::submit`을 호출하는 `execute_suggestion`만 compatibility adapter에 남긴다. `aihack-runtime`은 content-aware `EntityStore`, item/monster factory, content bootstrap, `GameWorld`, 모든 system adapter, observation projection, snapshot/hash, cloned transaction orchestration, 저장·replay I/O와 구체 `GameSession`을 소유한다. TUI는 `GameClient`를 확장한 저장·새 게임용 `TuiClient` trait object를 보유하고, headless policy는 generic `GameClient`만 소비한다. 두 app manifest는 root facade와 `aihack-core`를 직접 의존하지 않는다. production binary 소유권은 각 app package로 이동했고 root package는 기존 module path와 integration test를 유지하는 compatibility facade로 축소했다. transport/config의 실제 local provider 동작은 R6-1 범위이므로 아직 구현하지 않았다.

**수용 기준:**

- [x] AI contract는 mutable core type export 금지
- [x] TUI와 headless가 `GameClient`만 사용
- [x] binary 이름과 CLI 유지
- [x] save/replay 경로 유지

**검증:**

```bash
cargo test --workspace --all-targets --locked
cargo run --locked --bin aihack -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

**선행:** R5-1

| Slice | 파일, 최대 5개 |
| --- | --- |
| R5-2A | `crates/aihack-ai-contract/Cargo.toml`, `src/lib.rs`, `crates/aihack-llm/Cargo.toml`, `src/lib.rs`, `apps/aihack-tui/Cargo.toml` |
| R5-2B | `src/core/observation.rs -> crates/aihack-ai-contract/src/observation.rs`, `crates/aihack-ai-contract/src/action_space.rs`, `crates/aihack-ai-contract/src/llm.rs` |
| R5-2C | `src/llm/mod.rs -> crates/aihack-llm/src/contract.rs`, `src/llm/decision.rs -> crates/aihack-llm/src/decision.rs`, `src/llm/narrative.rs -> crates/aihack-llm/src/narrative.rs`, `crates/aihack-llm/src/config.rs`, `crates/aihack-llm/src/transport.rs` |
| R5-2D0 | `crates/aihack-runtime/Cargo.toml`, `src/lib.rs`, `src/client.rs`, `src/domain/{entity,item,monster}.rs`, `tests/{game_client_contract,entity_store_contract}.rs`, root workspace manifest |
| R5-2D | `src/main.rs -> apps/aihack-tui/src/main.rs`, `apps/aihack-tui/src/lib.rs`, `src/ui/tui/mod.rs -> apps/aihack-tui/src/app.rs`, `src/ui/mod.rs -> apps/aihack-tui/src/ui.rs` |
| R5-2E | `src/ui/tui/config.rs -> apps/aihack-tui/src/config.rs`, `src/ui/tui/effects.rs -> apps/aihack-tui/src/effects.rs`, `src/ui/tui/input.rs -> apps/aihack-tui/src/input.rs`, `src/ui/tui/labels.rs -> apps/aihack-tui/src/labels.rs`, `src/ui/tui/layout.rs -> apps/aihack-tui/src/layout.rs` |
| R5-2F | `src/ui/tui/render_map.rs -> apps/aihack-tui/src/render_map.rs`, `src/ui/tui/render_panels.rs -> apps/aihack-tui/src/render_panels.rs`, `src/ui/tui/theme.rs -> apps/aihack-tui/src/theme.rs`, `src/ui/tui/viewport.rs -> apps/aihack-tui/src/viewport.rs` |
| R5-2G | `apps/aihack-headless/Cargo.toml`, `apps/aihack-headless/src/lib.rs`, `src/bin/aihack-headless.rs -> apps/aihack-headless/src/main.rs` |
| R5-2H | root `src/lib.rs` compatibility facade, root `Cargo.toml`에서 default-run 제거/default-members 설정, `apps/aihack-tui/Cargo.toml`에 default-run 이동 |

각 slice 뒤 compile, 마지막 slice 뒤 전체 workspace test와 R4 hash 비교를 실행한다. root `tests/**`는 facade를 통해 기존 test target으로 유지하며 R5에서 test 의미를 바꾸지 않는다.

**범위:** 8개 sequential slice, 각 S 또는 M, slice당 5개 이하

### Checkpoint R5

**2026-07-17 종료 노트:** R5 구현 slice는 완료됐다. core는 순수 도메인 계약·결정론 규칙·generic state/save/transaction primitive를, content는 embedded schema와 registry를, runtime은 content-aware entity/world/bootstrap·system adapter·projection·저장 I/O·구체 `GameSession`을 소유한다. `audit_report_9.md`가 보고서 8의 IMP-F008 시정과 R1~R5 전체 회귀를 PASS로 종결했다. R6~R8은 시작하지 않았다.

- [x] SC-ARCH-01 PASS (`audit_report_6.md`)
- [x] workspace dependency 방향 PASS
- [x] binary CLI compatibility PASS
- [x] core dependency tree에 TUI/HTTP 0건
- [x] R4 hash 유지

### Task R6-1: Local LLM transport와 narrative

**설명:** loopback OpenAI-compatible endpoint를 reqwest blocking worker 1개로 호출하고 강제 timeout/fallback을 구현한다.

**진행 상태 (2026-07-17):** 구현 및 local gate 완료. endpoint·resolve 결과를 loopback으로 제한하고 연결 주소를 client에 고정했으며 redirect/system proxy를 비활성화했다. narrative 요청은 전용 worker 1개와 capacity 16 request/response channel을 사용한다. request/response 크기, timeout, strict JSON, Unicode scalar 길이, C0/C1/ANSI control, soft user text 경계를 검증하고 실패 시 결정론적 fallback을 반환한다. R6 전체 checkpoint는 R6-2와 R6-3 완료 전까지 진행 중이다.

**수용 기준:**

- [x] disabled, success, connect failure, timeout, invalid JSON, empty text 처리
- [x] connect 500ms, narrative 2000ms
- [x] output 1..=240 Unicode scalar
- [x] bounded request/response channel capacity 16, queue full은 즉시 typed error
- [x] redirect/proxy 비활성, response body 최대 65,536 bytes
- [x] endpoint resolve 결과 loopback 재검사 및 연결 주소 고정, request JSON 최대 32,768 bytes
- [x] user text/control character/unknown JSON field boundary validation
- [x] provider 결과가 snapshot/save/replay를 변경하지 않음

**검증:**

```bash
cargo test -p aihack --locked --test llm_transport
cargo test -p aihack --locked --test llm_narrative
```

**선행:** R5 checkpoint
**파일:** `crates/aihack-llm/Cargo.toml`, `Cargo.lock`, `crates/aihack-llm/src/config.rs`, `crates/aihack-llm/src/transport.rs`, `tests/llm_transport.rs`
**범위:** R6-1A S, 5개

**파일:** `crates/aihack-llm/src/lib.rs`, `crates/aihack-llm/src/worker.rs`, `crates/aihack-llm/src/narrative.rs`, `tests/llm_narrative.rs`, `tests/ui_runtime_smoke.rs`
**범위:** R6-1B S, 5개

### Task R6-2: Decision revision gate

**설명:** action proposal에 request/revision correlation을 적용한다.

**진행 상태 (2026-07-17):** 구현 및 local gate 완료. `DecisionGate`가 kind별 outstanding 1개를 opaque UUID와 request revision으로 추적한다. unknown ID는 정상 outstanding을 소비하지 않으며 matching response만 소진한다. strict decision JSON은 현재 request ActionSpace의 canonical wire action으로만 변환되고, transport와 TUI에서 current revision·current ActionSpace·confidence·rationale를 검증한 뒤 private `ValidatedDecision`을 만든다. submit 직전 revision도 다시 확인하므로 validation과 실행 사이의 stale 응답은 core를 변경하지 않는다.

**수용 기준:**

- [x] stale turn/hash는 `LlmResponseError::Stale`
- [x] unknown request_id는 `InvalidSchema { code: UnknownRequestId }`
- [x] current action space에 없는 action은 `InvalidSchema { code: InvalidAction }`
- [x] valid action만 normal submit path 사용

**검증:**

```bash
cargo test -p aihack --locked --test llm_decision_support
cargo test -p aihack --locked --test llm_revision_gate
```

**선행:** R6-1
**파일:** `crates/aihack-ai-contract/src/llm.rs`, `crates/aihack-ai-contract/src/lib.rs`, `crates/aihack-llm/src/decision.rs`, `crates/aihack-llm/src/transport.rs`, `crates/aihack-llm/src/worker.rs`
**범위:** R6-2A S, 5개

**파일:** `src/llm/decision.rs`, `tests/llm_revision_gate.rs`, `tests/llm_decision_support.rs`, `tests/ui_runtime_smoke.rs`
**범위:** R6-2B S, 4개

### Task R6-3: Soft adjudication UI

**설명:** LLM 판정을 Favorable/Neutral/Unfavorable presentation으로 표시한다.

**진행 상태 (2026-07-17):** 구현 및 local gate 완료. strict `SOFT_ADJUDICATION` JSON은 UPPER_SNAKE verdict와 camelCase field만 허용하고 reason code·message·control 경계를 render 전에 검증한다. provider 실패는 `Neutral / LLM_UNAVAILABLE`로 표시하며 활성 결과는 인벤토리보다 우선해 INSPECT 패널에 나타난다. 결과 설정과 N/Esc dismiss는 core revision·save/replay truth를 바꾸지 않는다. TUI 종료 경로는 terminal 복원 뒤 sender를 닫고 worker를 최대 250ms 기다린 후 detach한다.

**수용 기준:**

- [x] reason_code와 message 표시
- [x] core effect 생성 0건
- [x] save/replay 포함 0건
- [x] reduced-motion/high-contrast에서도 텍스트 판독 가능
- [x] TUI exit에서 terminal restore 후 worker shutdown grace 250ms

**검증:**

```bash
cargo test -p aihack --locked --test llm_soft_adjudication
cargo test -p aihack --locked --test ui_runtime_smoke
```

**선행:** R6-2
**파일:** `crates/aihack-ai-contract/src/llm.rs`, `crates/aihack-llm/src/lib.rs`, `crates/aihack-llm/src/soft_adjudication.rs`, `crates/aihack-llm/src/transport.rs`, `src/llm/soft_adjudication.rs`
**범위:** R6-3A S, 5개

**파일:** `apps/aihack-tui/src/lib.rs`, `apps/aihack-tui/src/tui/input.rs`, `apps/aihack-tui/src/tui/mod.rs`, `apps/aihack-tui/src/tui/render_panels.rs`, `tests/llm_soft_adjudication.rs`
**범위:** R6-3B S, 5개

**파일:** `crates/aihack-llm/src/worker.rs`, `tests/llm_transport.rs`
**범위:** R6-3C S, 2개

### Task R6-4: Integrated TUI CTA and failure matrix

**설명:** local provider worker를 실제 TUI input/render loop에 연결하고 모든 결과를 core 권한 경계 안에서 처리한다.

**진행 상태 (2026-07-18):** local automated gate와 live PTY/loopback fixture matrix 완료. G/A/J가 narrative/decision/soft request를 enqueue하고 textual status와 Judge 240자 modal을 표시한다. suggestion은 current revision과 ActionSpace를 재검증한 뒤 Y 승인에서만 normal submit을 사용한다. N/Esc는 UI-only 결과를 제거하고 R은 새 request ID로 마지막 실패를 재시도한다. 동일 종류 outstanding 1개, 250ms cooldown, capacity 16 response oldest-drop, keyboard/mouse 동일 CTA candidate가 테스트로 고정됐다. `audit_report_10.md`의 HOLD 사유는 R6-6에서 시정했고 `audit_report_11.md` 독립 재감사가 R6 checkpoint를 PASS로 종결했다.

**수용 기준:**

- [x] G/A/J enqueue와 Disabled/Ready/Pending/Busy/Timeout/Unavailable/Invalid/Stale 텍스트 상태
- [x] Judge input trim·control 거부·Unicode 240자 경계
- [x] Y만 decision submit, N/Esc와 soft/narrative는 core effect 0건
- [x] stale/invalid/unavailable fallback에서 snapshot hash 불변
- [x] 표시 footer의 keyboard/mouse CTA candidate 일치
- [x] live PTY + loopback compatible fixture 수동 matrix
- [x] `audit_report_11.md` 시정 독립 재감사 PASS

**고려 대상:** 최종 통합에서 실제 provider 호환성 증거가 반드시 필요할 때만 재사용 가능한 localhost OpenAI-compatible 임시 adapter를 만들고 원격 provider smoke를 수행한다. Google AI Studio Gemini는 후보 provider이며 API key는 adapter 환경변수로만 전달한다.

**검증:**

```bash
cargo test -p aihack --locked --test llm_transport
cargo test -p aihack --locked --test llm_tui_integration
cargo test -p aihack --locked --test llm_revision_gate --test llm_soft_adjudication
```

**파일:** `crates/aihack-llm/src/service.rs`, `crates/aihack-llm/src/transport.rs`, `apps/aihack-tui/src/tui/mod.rs`, `apps/aihack-tui/src/tui/render_panels.rs`, `tests/llm_tui_integration.rs`
**범위:** R6-4A S, 5개

**파일:** `apps/aihack-tui/src/tui/input.rs`, `tests/llm_transport.rs`
**범위:** R6-4B S, 2개

### Task R6-5: Live PTY contract closure

**설명:** 승인된 terminal 크기·접근성·failure matrix를 실제 TUI binary에서 실행하고 event/render 경계 회귀를 닫는다.

**진행 상태 (2026-07-18):** 구현 및 live PTY/loopback fixture matrix 완료. 120x36, 80x24, 60x24는 full TUI를 유지하고 59x23은 안내 화면에서 gameplay 입력을 무시한 채 Q/Esc로 clean exit한다. runtime Enter와 `.` Wait가 표시 계약과 일치하며 TIMEOUT/DOWN은 Retry를 보존한다. Judge modal은 빈 입력 행을 포함해 underlying panel을 지운다. `--high-contrast`와 `--reduced-motion`으로 수동 접근성 상태를 선택할 수 있다.

**수용 기준:**

- [x] 120x36 disabled에서 OFF badge와 core play
- [x] 80x24 success fixture에서 G/A/J/Y/N CTA 판독
- [x] 60x24 timeout과 connection-refused에서 Retry/Dismiss 판독
- [x] 59x23 안내와 Q/Esc clean exit
- [x] delayed suggestion + Wait에서 STALE, Y CTA와 provider submit 0건
- [x] 실제 KeyCode, footer mapping, modal clear 회귀 테스트

**증거:** `docs/R6_MANUAL_MATRIX.md`

**파일:** `apps/aihack-tui/src/main.rs`, `apps/aihack-tui/src/tui/config.rs`, `apps/aihack-tui/src/tui/mod.rs`, `apps/aihack-tui/src/tui/render_panels.rs`, `apps/aihack-tui/tests/tui_contract.rs`
**범위:** R6-5A S, 5개

**파일:** `apps/aihack-tui/src/tui/input.rs`, `tests/ui_input_mapping.rs`, `tests/ui_layout.rs`, `tests/llm_tui_integration.rs`
**범위:** R6-5B S, 4개

### Task R6-6: Audit report 10 public contract and evidence remediation

**설명:** IMP-F009/010/011과 DBG-F004를 연결해 versioned public request/response boundary, enum 안정성, 재현 fixture와 control 문서 상태를 정렬한다.

**진행 상태 (2026-07-18):** 구현 및 local 표적 검증 완료, `audit_report_11.md` 독립 재감사 PASS. public request가 `schema_version`, `SessionRevision`, 소유형 `LlmObservationView`, 독립 `ActionSpace`, `LlmRequestKind`를 포함한다. enqueue는 schema 0/2, projection/action bound, canonical 32,768 bytes를 외부 work 전에 typed error로 거부하고 TUI는 response schema 0/2를 payload 수용 전에 거부한다. public error/command enum은 non-exhaustive이며 저장소 fixture와 PTY script가 success/timeout/stale/down 및 pending-exit를 재현한다.

**수용 기준:**

- [x] public DTO shape와 request schema 0/1/2 contract
- [x] response schema 0/2 TUI acceptance 0건
- [x] observation/action bound와 canonical oversize synchronous failure
- [x] public error/command enum non-exhaustive와 downstream wildcard
- [x] 저장소 fixture로 success/timeout/stale/down 재현
- [x] pending exit에서 terminal restore 선행, bounded shutdown
- [x] 감사 전 G-LLM 상태 `Implemented / Audit HOLD` 통일
- [x] IMP-F009/010/011, DBG-F004 독립 재감사 Verified

**검증:**

```bash
cargo test -p aihack --locked --test llm_transport --test llm_tui_integration
cargo clippy -p aihack-llm -p aihack-tui -p aihack --all-targets --locked -- -D warnings
scripts/r6_pty_matrix.sh
scripts/r6_pending_exit_smoke.sh
```

**파일:** `crates/aihack-llm/src/service.rs`, `crates/aihack-llm/src/transport.rs`, `crates/aihack-llm/src/worker.rs`, `crates/aihack-llm/src/config.rs`, `tests/llm_transport.rs`
**범위:** R6-6A S, 5개

**파일:** `crates/aihack-llm/src/decision.rs`, `crates/aihack-llm/src/narrative.rs`, `apps/aihack-tui/src/tui/mod.rs`, `tests/llm_tui_integration.rs`
**범위:** R6-6B S, 4개

**파일:** `scripts/r6_loopback_fixture.py`, `scripts/r6_pty_matrix.sh`, `scripts/r6_pending_exit_smoke.sh`, `docs/R6_MANUAL_MATRIX.md`, `BUILD_GUIDE.md`
**범위:** R6-6C S, 5개

**파일:** `GAP_CLOSURE_ROADMAP.md`, `IMPLEMENTATION_SUMMARY.md`, `audit_roadmap.md`, `README.md`, `CHANGELOG.md`
**범위:** R6-6D S, 5개

**파일:** `spec.md`, `DESIGN_DECISIONS.md`, `LESSONS_LEARNED.md`
**범위:** R6-6E S, 3개

### Checkpoint R6

- [x] SC-LLM-01 local evidence PASS
- [x] SC-LLM-02 local evidence PASS
- [x] SC-LLM-03 local evidence PASS
- [x] provider 없는 실행 PASS
- [x] stale/invalid response submit 호출 0건

### Task R7-1: Provenance inventory와 license scope

**설명:** 레거시 자산과 새 구현의 출처 상태를 파일 단위로 기록한다.

**진행 상태 (2026-07-20):** engineering inventory와 자동 차단 검증 완료. 공식 archive와 `dat/license` checksum을 확인했고, legacy code/data/license는 runtime 밖 `Blocked/Reviewed`로 격리했다. `audit_report_12.md`의 SEC-F002 시정으로 validator는 runtime file의 최구체 coverage, Approved reviewer/date/license/scope/notice/modification-notice/evidence, content full checksum, scenario schema/ID/function과 Blocked reference를 검사한다. `audit_report_13.md`의 IMP-F013/SEC-F003 시정으로 R7/R8 책임을 분리하고 검사 root를 script-relative repository로 고정했다. 프로젝트 소유자의 파생물 분류와 NGPL 승인 후 content/scenario provenance를 근거 포함 `Approved`로 전환했으며 R7 checkpoint는 PASS한다. R8 독립 기술 감사 전 외부 게시는 계속 보류한다.

**수용 기준:**

- [x] `PROVENANCE.md`에 상태 enum과 초기 inventory
- [x] 손상된 NGPL 33..35행과 local checksum 기록
- [x] Apache/NGPL 적용 범위 미확정 항목 격리
- [x] Unknown/Blocked 자산의 runtime import 0건
- [x] status-only·필수 field 누락·checksum drift·coverage ambiguity 우회 차단

**검증:**

- `audit_roadmap.md` provenance search 통과
- 공식 3.6.7 source checksum과 기록 일치

**선행:** R0-3
**파일:** `PROVENANCE.md`, `docs/compatibility/README.md`, `tests/provenance_manifest.rs`
**범위:** M, 3개

### Task R7-2: Compatibility scenario trace

**설명:** NH367-C001..C010의 출처와 기대 결과를 고정한다.

**진행 상태 (2026-07-18):** 10개 record와 10개 integration test 구현 완료. 각 record는 공식 archive checksum, C file/symbol locator, 관찰 규칙, typed command, expected event/state/hash field를 연결한다. C008 hunger drift를 수정했고, `audit_report_12.md`의 DBG-F005 시정으로 C003 hit/damage/HP/death/RNG와 C007 turn/item/charge/map/RNG를 직접 assert한다. schema gate는 ID 유일성, non-empty locator/command/event/hash/module과 실제 test function 연결을 검증한다. engineering tests는 PASS지만 provenance는 `Reviewed`이므로 승인 전 release compatibility count에는 포함하지 않는다.

**수용 기준:**

- [x] 10개 scenario 문서
- [x] 각 문서에 source, observation, command, expected event/hash fields
- [x] 각 scenario에 integration test
- [x] P8-G01..G20 regression 유지
- [x] record schema와 C003/C007 expected outcome 직접 assertion

**검증:**

```bash
cargo test -p aihack --locked --test nethack_367_compat
cargo test -p aihack --locked --test golden_phase8_rules
```

**선행:** R3 checkpoint, R7-1

| Slice | 파일 |
| --- | --- |
| R7-2A | `docs/compatibility/NH367-C001-wall-movement.md`, `docs/compatibility/NH367-C002-closed-door.md`, `tests/nethack_367_compat.rs` |
| R7-2B | `docs/compatibility/NH367-C003-bump-attack.md`, `docs/compatibility/NH367-C004-item-actions.md`, `tests/nethack_367_compat.rs` |
| R7-2C | `docs/compatibility/NH367-C005-stairs.md`, `docs/compatibility/NH367-C006-search.md`, `tests/nethack_367_compat.rs` |
| R7-2D | `docs/compatibility/NH367-C007-projectiles.md`, `docs/compatibility/NH367-C008-hunger-status.md`, `tests/nethack_367_compat.rs` |
| R7-2E | `docs/compatibility/NH367-C009-save-continuation.md`, `docs/compatibility/NH367-C010-game-over.md`, `tests/nethack_367_compat.rs` |

각 slice는 record 2개와 동일 test file의 case 2개만 수정한다.
**범위:** 5회 M Tasks, slice당 3개

### Checkpoint R7

- [x] SC-COMPAT-01 engineering evidence PASS
- [x] provenance inventory/checksum/legacy 격리/fail-closed validator PASS
- [x] SC-LICENSE-01 — project-owner NGPL approval와 machine validation 완료; `audit_report_19.md` 기술 evidence Verified
- [x] runtime included provenance Unknown/Blocked 0건
- [x] compatibility report 생성
- [x] source 직접 import 0건

**현재 판정:** `PASS WITH KNOWN RISKS`. 표적 42개와 전체 322개, build/security gate는 `audit_report_14.md`까지 검증됐다. 이후 2026-07-20 project-owner NGPL approval를 기록해 R7 checkpoint의 actual approval 조건도 로컬 PASS했다. 독립 R8 기술 감사와 외부 게시 승인은 별개다.

### Task R8-1: 통합 릴리즈 감사

**설명:** 모든 성공 기준과 문서 동기화를 검증한다.

**진행 상태 (2026-08-18):** R8-0 fail-closed preflight와 R8-1A/B/C를 구현했고 `audit_report_21.md`가 report 20의 active-state/false-green 시정을 PASS로 종결했다. `scripts/r8_checkpoint.sh`는 script-relative canonical root에서 R7 승인 상태, workspace 0.3.0/NGPL, 공식 LICENSE checksum, NOTICE, source packaging, 문서 archive target을 검사한다. R9 기준 commit `41a1b63f11a57a671b0f705883431dab24298b5a`의 Actions run `32034295607`은 Linux/Windows success다. report 23의 후속 HOLD는 R8 과거 시정 재발이 아니라 R9 witness·filesystem·Windows checkpoint·권한 문서의 새 finding이다.

**수용 기준:**

- [x] R1~R7 checkpoint/evidence 및 SC-BUILD-02 same-SHA Linux/Windows CI PASS
- [x] 승인된 whole-work NGPL과 notice를 workspace manifest 및 release 문서에 반영
- [x] Cargo/README/CHANGELOG 버전 0.3.0
- [x] archive chain 무결성 PASS
- [x] AI 구현 문서 표준 12개 self-check와 report 20 시정 독립 재감사 PASS (`audit_report_21.md`)

**검증:**

- `audit_roadmap.md` R8 명령 전체 실행
- Linux/Windows CI green
- 수동 TUI/LLM degraded flow 확인

**수동 로컬 상태 (2026-07-20):** `scripts/r8_tui_core_flow.sh`가 high contrast/reduced motion의 Title → Character Creation → Playing → Inventory → Game Over → New Run과 59x23 clean exit를 PASS했다. 기존 deterministic loopback PTY matrix와 pending-exit smoke도 success/timeout/stale/down 및 terminal restore를 PASS했다. 실제 LLM/API 호출은 없었다.

**보고서 16 HOLD 시정 (2026-07-20):** `IMP-F015`는 compatibility index 10행과 개별 Approved record를 동기화하고 파싱 회귀 테스트로 고정했다. `IMP-F012`는 프로젝트 소유자의 기존 직접 지시와 승인 범위를 `AIHACK-OWNER-2026-07-20-NGPL-01` record로 전사해 PROV/scenario evidence에 연결했으며 qualified legal opinion은 주장하지 않는다. `IMP-F014`의 배포되지 않는 Git history 의존은 제거하고 `MODIFICATIONS.md`, export-substitution `RELEASE-METADATA`, `SHA256SUMS`와 실제 tar archive verifier로 대체했다. `DBG-F006`의 clean R8 commit, 실제 release bundle 및 same-commit 양 OS CI는 commit/push 전까지 계속 HOLD다.

**보고서 17 HOLD 시정 (2026-07-20):** `DBG-F007`은 `PROJECT_OWNER_LICENSE_APPROVAL.md`를 output/source archive와 checksum의 필수 항목으로 승격하고, archive/output metadata의 owner/modification ID가 실제 Approval ID/Notice ID와 일치하도록 Linux verifier와 Windows release script를 강화했다. 완전 bundle은 PASS하고 문서 누락, metadata ID 누락, owner/modification record ID 불일치와 legacy 포함은 각각 FAIL하는 실제 Git archive fixture로 회귀를 고정했다. 이 tree를 포함하는 release commit과 해당 SHA의 양 OS CI는 별도 release gate다.

**보고서 18 HOLD 시정 (2026-07-22):** Linux verifier의 부분 문자열 비교를 key/value parser로 교체해 product, version, commit, source license, owner approval와 modification notice가 archive/output에 각각 정확히 한 번 존재하고 전체 값이 일치하도록 했다. Windows PowerShell gate도 동일한 단일-key·완전 값 계약을 사용한다. owner/modification 각각 wrong, suffix, duplicate를 archive와 output에 주입하는 12개 actual-archive case가 모두 fail-closed한다.

**보고서 19 HOLD 시정 (2026-07-22):** `audit_report_19.md`는 보고서 18의 기술 시정, clean bundle과 same-SHA CI를 모두 Verified하고 활성 문서 상태만 `IMP-F016`/`XPF-F011`로 HOLD했다. 2026-07-22 기준 evidence는 commit `b9bd680200d82b20d7c9ba961a2758caa3d49e16`의 [Actions run `29886410221`](https://github.com/Yupkidangju/AIHack/actions/runs/29886410221)이며 `ubuntu-latest quality gate`와 `windows-latest quality gate`가 success다. 활성 문서는 SC-BUILD-02 PASS와 report 19의 documentation-sync HOLD를 구분해 기록하며, 최종 R8 PASS는 이 문서 diff의 독립 재감사 전까지 선언하지 않는다.

**보고서 20 HOLD 시정 (2026-07-22):** 최상단 current baseline, `G-LICENSE-001`, BUILD_GUIDE 전체 테스트 표현을 현재 evidence와 정렬했다. 문서 회귀는 implementation summary 절, gap별 행과 checklist 행을 직접 식별하고 알려진 stale 상태의 부재까지 검사한다. `IMP-F016`/`DBG-F008` 시정의 독립 재감사 전까지 R8 전체와 외부 게시는 HOLD다.

**보고서 21 종결 (2026-07-22):** report 20의 `IMP-F016`, `DBG-F008`, `XPF-F011`을 Verified/Resolved하고 R8 문서 remediation을 PASS로 종결했다. 이후 문서에서 report 20 재감사 대기를 현재 상태로 사용하지 않는다.

**선행:** R6 checkpoint, R7 checkpoint

| Slice | 파일, 최대 5개 |
| --- | --- |
| R8-0 | `scripts/r8_checkpoint.sh`, `tests/release_gate.rs`, `IMPLEMENTATION_SUMMARY.md`, `audit_roadmap.md`, `BUILD_GUIDE.md` |
| R8-1A | `Cargo.toml`, `Cargo.lock`, `README.md`, `CHANGELOG.md`, `spec.md` |
| R8-1B | `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `BUILD_GUIDE.md`, `audit_roadmap.md`, `PROVENANCE.md` |
| R8-1C | `designs.md`, `DESIGN_DECISIONS.md`, `docs/compatibility/README.md`, `DOCUMENTATION_AUDIT_REPORT.md` |
| R8-1D | owner approval record, modification manifest/metadata, release bundle verifier와 report 16 회귀 시정 |

**범위:** R8-0 완료 후 3개 sequential slice, 각 S 또는 M, slice당 5개 이하

## 8. 병렬화와 순서

- R1, R2, R3, R4, R5, R6은 순차다.
- R7-1은 R1~R6과 병렬 가능하다.
- R7-2는 R3 완료 뒤 시작한다.
- 같은 파일을 수정하는 Task는 병렬 실행하지 않는다.
- R8은 모든 checkpoint 뒤에만 실행한다.

## 9. 유지보수 규칙

- refactor와 behavior change를 같은 Task에 넣지 않는다.
- Task당 변경 파일은 5개 이하로 유지한다.
- hash 변경은 오류로 취급하고 원인과 ADR이 없으면 baseline을 갱신하지 않는다.
- 새 dependency는 도입 이유, license, duplicate tree를 기록한다.
- 테스트 이름은 성공 조건을 표현한다.
- 문서의 `완료` 표시는 검증 로그와 함께 변경한다.
- LLM prompt와 response는 core type에 저장하지 않는다.
- compatibility ID 없는 NetHack 규칙 변경은 merge하지 않는다.

## 10. 구현 시작 순서

다음 단계는 `docs/audit/audit_report_26.md` 시정의 전체 로컬 quality gate와 새 clean same-SHA Ubuntu/Windows actual bundle이다. report 24까지의 종결 evidence, report 25의 부분 positive evidence와 project-owner NGPL approval은 유지되며, 외부 게시는 새 독립 PASS와 별도 사용자 승인까지 HOLD다. 실제 model provider smoke는 비차단 고려 대상이다.

2026-08-18 `audit_report_23.md` 시정은 문서 우선으로 시작한다. 첫 slice인 SEC-F001은 ADR-0032와 `spec.md` 14·16절을 구현 권한으로 삼아 `ArtifactStore` capability root, no-follow/single-link open, `create_new` save temp, 실행별 TUI quick-save 경계를 구현한다. 회귀 기준은 사전 배치 temp hard link와 replay symlink/hard link가 외부 victim을 바꾸지 않고, 기존 save 보존과 정상 save/load/replay가 함께 통과하는 것이다. 완료 표시는 표적 테스트와 workspace 검증 후에만 갱신한다.

DBG-F009의 platform authority는 두 OS 모두 Git Bash `scripts/r7_checkpoint.sh`와 `scripts/r8_checkpoint.sh`로 확정한다. `.sha256` checkout은 LF로 고정하되 스크립트는 CRLF fixture도 정규화하며, Windows CI가 실제 checkout checkpoint를 bundle gate와 별도로 실행한다. `tests/provenance_manifest.rs`의 CRLF positive와 checksum drift negative가 두 조건을 분리한다.

## 11. R9 콘텐츠 인과 폐쇄 Task

역사 기록 (2026-08-18~24): R9-1..R9-5의 표적 인과 루프 이후 report 23이 R9-6 witness를 HOLD했고 report 24가 당시 coder remediation을 검증했다. report 25의 production GoldScore pair는 Actions `32650404618`에서 양 OS Verified됐다. report 26은 사후 summary label 삭제 negative를 재개방했으며, 현재는 speed/AI/difficulty를 독립 production pair로 실행하고 9종 producer/content/pair를 실행 전에 하나씩 제거하는 full-run matrix로 교체했다. 전체 gate와 새 same-SHA CI 전에는 R9 최종 PASS를 선언하지 않는다.

### Task R9-1: semantic delta와 causal witness 기반

snapshot의 자동 메타 변화와 게임 의미 상태 변화를 구분한다. event-only와 turn-only 변화는 FAIL이고 위치, HP, AC, nutrition, gold, score, run state, entity lifecycle 변화만 witness가 된다.

### Task R9-2: 음식·영양·시체 루프

Food/Corpse item의 content nutrition을 Eat 명령으로 소비해 world nutrition과 hunger state에 연결한다. ration과 jackal corpse 모두 content 값만큼 nutrition을 바꾸고 consumed tombstone으로 전이해야 한다.

### Task R9-3: content behavior projection

`ac_bonus`, monster `ai`/`passive` 등 지원 필드를 typed runtime data에 보존하고 실제 규칙 소비자가 사용한다. `speed`/`difficulty`는 실제 소비 경로를 제공하거나 schema 비목표로 닫는다. injected registry A/B는 해당 값 하나의 차이로 예상한 semantic delta 차이를 생성해야 한다.

### Task R9-4: 경제·점수 루프

`base_price`와 production에서 생성 가능한 경제 상태를 후속 score에 연결한다. 가격이 다른 loot를 획득한 동일 seed scenario의 후속 score 또는 gold가 달라야 한다.

### Task R9-5: luck·hallucination 폐쇄

두 상태에 production producer와 별도 downstream consumer를 제공하거나 호환 경계만 남기는 비목표 결정을 문서화한다.

### Task R9-6: seed 기반 장기 인과 회귀

seed 42, 7, 1234 각각 absolute turn 1000 이상 실행하며 9종 `CausalWitnessRecord`와 final hash를 3회 반복 비교한다. record는 scenario, producer, content field/source value와 consumer before/after를 보유한다. MonsterSpeed/MonsterAi는 독립 A/B pair에서만, GoldScore는 동일 world/turn의 gold/no-gold clone이 모두 production `death_score`를 통과한 exact pair에서만 기록한다. 일반 `survival-v1`과 분리된 테스트 전용 fixture가 production `GameSession::submit`을 사용하며 event-only·turn-only·witness별 누락 negative validator가 false-green을 차단한다. SC-CAUSE-01..07과 전체 workspace quality gate 통과가 완료 조건이다.

### R9 SC-CAUSE 개별 책임 매핑

| ID | 구현 책임 | 테스트 책임 |
| --- | --- | --- |
| SC-CAUSE-01 | `CausalProjection`, `CausalSummary`, `CausalWitnessRecord` attribution | `sc_cause_contract_ids_map_to_code_and_tests` |
| SC-CAUSE-02 | content-backed armor·monster typed behavior, `observe_monster_speed_pair`, `observe_monster_ai_pair`, `observe_monster_difficulty_pair` | `monster_speed_content_changes_actual_turn_movement`, `monster_ai_content_changes_actual_turn_intent`, `monster_passive_content_changes_player_status`, `armor_content_bonus_changes_player_defense_state` |
| SC-CAUSE-03 | Food/Corpse Eat와 death corpse producer | `eating_food_changes_nutrition_hunger_and_item_lifecycle`, `jackal_death_creates_an_edible_corpse_that_changes_hunger` |
| SC-CAUSE-04 | production gold/no-gold exact score pair, Pray/luck와 hallucinating 비목표 risk | `item_base_price_changes_actual_game_over_score`, `gold_score_witness_uses_a_paired_production_score`, `prayer_created_luck_changes_the_next_attack_roll`, `sc_cause_contract_ids_map_to_code_and_tests` |
| SC-CAUSE-05 | `REQUIRED_CAUSAL_WITNESSES`, attributed record required-set validator | `causal_fixture_covers_every_required_witness_for_each_seed` |
| SC-CAUSE-06 | attributed witness record multiset와 final hash 결정론 | `causal_witness_multiset_and_final_hash_are_stable_across_three_runs` |
| SC-CAUSE-07 | event/turn-only, speed/AI 원인 공유와 사후 label 삭제 false-green 거부 | `causal_validator_rejects_event_only_and_turn_only_changes`, `causal_actual_producer_removal_loses_exactly_one_required_witness`, `monster_speed_content_changes_actual_turn_movement`, `monster_ai_content_changes_actual_turn_intent` |
