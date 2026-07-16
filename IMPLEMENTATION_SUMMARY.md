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

현재 코드는 fmt, clippy, 전체 245개 test, release build를 통과한다. 그러나 다음은 완료되지 않았다.

- 고정 toolchain과 CI
- 단일 crossterm dependency
- private session/world state
- runtime에 연결된 TOML registry
- 실제 1000 accepted-turn 검증
- 실제 local LLM transport와 강제 timeout
- stale LLM response 차단
- NetHack 출처·호환성 trace

v0.3.0은 새 기능 추가보다 이 8개 기반을 먼저 닫는다.

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
| `crates/aihack-llm/src/*` | R6 전까지 provider-independent contract scaffold |
| `crates/aihack-runtime/src/*` | content-aware world/session/system, projection, save/replay I/O |
| `apps/aihack-tui/src/*` | Observation render, CommandIntent 생성, `aihack` binary |
| `apps/aihack-headless/src/*` | policy 실행, report/replay CLI, `aihack-headless` binary |
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
- [x] lockfile 변경 여부를 job 종료 시 확인
- [x] cargo-audit 0.22.1, cargo-deny 0.19.4를 pinned install
- [x] vulnerability, license, source, crossterm duplicate gate 포함
- [ ] Linux/Windows 원격 CI green 및 실패 단계 로그 확인

**검증:**

- workflow YAML parse
- 로컬에서 workflow와 동일한 5개 명령 통과

**선행:** R1-2
**파일:** `.github/workflows/ci.yml`, `deny.toml`, `BUILD_GUIDE.md`
**범위:** S, 3개

### Checkpoint R1

- [x] SC-BUILD-01 PASS (R1 local audit 통과)
- [ ] SC-BUILD-02 PASS (원격 CI 실행 대기)
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

**수용 기준:**

- [ ] disabled, success, connect failure, timeout, invalid JSON, empty text 처리
- [ ] connect 500ms, narrative 2000ms
- [ ] output 1..=240 chars
- [ ] bounded request/response channel capacity 16, queue full은 즉시 typed error
- [ ] redirect/proxy 비활성, response body 최대 65,536 bytes
- [ ] endpoint resolve 결과 loopback 재검사, request JSON 최대 32,768 bytes
- [ ] user text/control character/unknown JSON field boundary validation
- [ ] provider 결과가 snapshot/save/replay를 변경하지 않음

**검증:**

```bash
cargo test -p aihack --locked --test llm_transport
cargo test -p aihack --locked --test llm_narrative
```

**선행:** R5 checkpoint
**파일:** `crates/aihack-llm/src/config.rs`, `transport.rs`, `worker.rs`, `narrative.rs`, `tests/llm_transport.rs`
**범위:** M, 5개

### Task R6-2: Decision revision gate

**설명:** action proposal에 request/revision correlation을 적용한다.

**수용 기준:**

- [ ] stale turn/hash는 `LlmResponseError::Stale`
- [ ] unknown request_id는 `InvalidSchema { code: UnknownRequestId }`
- [ ] current action space에 없는 action은 `InvalidSchema { code: InvalidAction }`
- [ ] valid action만 normal submit path 사용

**검증:**

```bash
cargo test -p aihack --locked --test llm_decision_support
cargo test -p aihack --locked --test llm_revision_gate
```

**선행:** R6-1
**파일:** `crates/aihack-llm/src/decision.rs`, `crates/aihack-ai-contract/src/llm.rs`, `tests/llm_revision_gate.rs`
**범위:** M, 3개

### Task R6-3: Soft adjudication UI

**설명:** LLM 판정을 Favorable/Neutral/Unfavorable presentation으로 표시한다.

**수용 기준:**

- [ ] reason_code와 message 표시
- [ ] core effect 생성 0건
- [ ] save/replay 포함 0건
- [ ] reduced-motion/high-contrast에서도 텍스트 판독 가능
- [ ] TUI exit에서 terminal restore 후 worker shutdown grace 250ms

**검증:**

```bash
cargo test -p aihack --locked --test llm_soft_adjudication
cargo test -p aihack --locked --test ui_runtime_smoke
```

**선행:** R6-2
**파일:** `crates/aihack-ai-contract/src/llm.rs`, `apps/aihack-tui/src/render_panels.rs`, `apps/aihack-tui/src/app.rs`, `tests/llm_soft_adjudication.rs`
**범위:** M, 4개

### Checkpoint R6

- [ ] SC-LLM-01 PASS
- [ ] SC-LLM-02 PASS
- [ ] SC-LLM-03 PASS
- [ ] provider 없는 실행 PASS
- [ ] stale/invalid response submit 호출 0건

### Task R7-1: Provenance inventory와 license scope

**설명:** 레거시 자산과 새 구현의 출처 상태를 파일 단위로 기록한다.

**수용 기준:**

- [ ] `PROVENANCE.md`에 상태 enum과 초기 inventory
- [ ] 손상된 NGPL 33..35행과 local checksum 기록
- [ ] Apache/NGPL 적용 범위 미확정 항목 격리
- [ ] Unknown/Blocked 자산의 runtime import 0건

**검증:**

- `audit_roadmap.md` provenance search 통과
- 공식 3.6.7 source checksum과 기록 일치

**선행:** R0-3
**파일:** `PROVENANCE.md`, `docs/compatibility/README.md`
**범위:** S, 2개

### Task R7-2: Compatibility scenario trace

**설명:** NH367-C001..C010의 출처와 기대 결과를 고정한다.

**수용 기준:**

- [ ] 10개 scenario 문서
- [ ] 각 문서에 source, observation, command, expected event/hash fields
- [ ] 각 scenario에 integration test
- [ ] P8-G01..G20 regression 유지

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

- [ ] SC-COMPAT-01 PASS
- [ ] SC-LICENSE-01 PASS
- [ ] runtime included provenance Unknown 0건
- [ ] compatibility report 생성
- [ ] source 직접 import 0건

### Task R8-1: 통합 릴리즈 감사

**설명:** 모든 성공 기준과 문서 동기화를 검증한다.

**수용 기준:**

- [ ] R1~R7 checkpoint 전부 PASS
- [ ] Cargo/README/CHANGELOG 버전 0.3.0
- [ ] archive chain 무결성 PASS
- [ ] AI 구현 문서 표준 12개 checklist PASS

**검증:**

- `audit_roadmap.md` R8 명령 전체 실행
- Linux/Windows CI green
- 수동 TUI/LLM degraded flow 확인

**선행:** R6 checkpoint, R7 checkpoint

| Slice | 파일, 최대 5개 |
| --- | --- |
| R8-1A | `Cargo.toml`, `Cargo.lock`, `README.md`, `CHANGELOG.md`, `spec.md` |
| R8-1B | `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `BUILD_GUIDE.md`, `audit_roadmap.md`, `PROVENANCE.md` |
| R8-1C | `designs.md`, `DESIGN_DECISIONS.md`, `docs/compatibility/README.md`, `DOCUMENTATION_AUDIT_REPORT.md` |

**범위:** 3개 sequential slice, 각 S 또는 M, slice당 5개 이하

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

다음 구현 단계는 `Task R6-1`이다. `audit_report_9.md`가 R5 문서 시정과 전체 회귀를 PASS했고 G-TEST-001/002와 G-ARCH-001의 closure도 유지된다. 전체 program PASS와 R8 release gate는 아직 열려 있다.
