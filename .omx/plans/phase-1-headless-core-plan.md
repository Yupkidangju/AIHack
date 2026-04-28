# Phase 1 계획: Headless Core deterministic runner

문서 상태: RALPLAN 초안  
작성일: 2026-04-28  
근거 문서: `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, `BUILD_GUIDE.md`, `DESIGN_DECISIONS.md`  
범위: Phase 1 전용. 구현 금지 상태의 실행 준비 계획.

## 1. 범위 고정

### 목표

문서만 존재하는 루트 프로젝트를 Rust Cargo 패키지로 전환하고, `CommandIntent::Wait`만 처리하는 deterministic headless core를 세운다.

### 포함 항목

1. Cargo 스캐폴딩: `Cargo.toml`, `src/main.rs`, `src/bin/aihack-headless.rs`, 최소 `src/core/*` 모듈.
2. `GameRng`: seed 기반 deterministic RNG wrapper.
3. `GameSession::new(seed: u64) -> GameSession`.
4. `CommandIntent::Wait` 처리 경로.
5. `TurnOutcome` 생성: `accepted`, `turn_advanced`, `events`, `snapshot_hash`, `next_state`.
6. replay JSONL 기록과 final hash 재현성 검증.

### 제외 항목

- TUI/GUI, UI adapter, 마우스/키 입력, 렌더링.
- map, movement, doors, vision, combat, items, monster AI, AI adapter/Observation 구현.
- 레거시 코드 직접 import 또는 Cargo workspace member 등록.
- ECS/Legion 도입.

## 2. RALPLAN-DR 요약

### 원칙

1. **결정론 우선**: 같은 seed와 같은 command stream은 byte-level로 동일한 replay hash를 만들어야 한다.
2. **단일 상태 원천**: 상태 변경은 `GameSession::submit(CommandIntent)` 내부에서만 일어난다.
3. **Phase 경계 엄수**: Phase 1은 `Wait`와 headless runner만 다루며 게임 규칙 확장은 Phase 2 이후로 넘긴다.
4. **레거시 격리**: `legacy_nethack_port_reference/`는 참조만 가능하고 새 Cargo 빌드 그래프에 포함하지 않는다.
5. **검증 가능한 최소 실행물**: `cargo test`와 `aihack-headless --seed 42 --turns 100`이 완료 기준이다.

### Top 3 decision drivers

1. **재현성**: replay hash가 AI 실험과 디버깅의 기준선이다.
2. **작은 PR 가능성**: 첫 PR은 scaffolding + deterministic loop에 집중해야 후속 Phase 충돌을 줄인다.
3. **아키텍처 부채 차단**: UI/ECS/레거시 직접 연결을 초기에 막아 이전 포트의 구조적 문제를 반복하지 않는다.

### viable options

| 옵션 | 설명 | 장점 | 단점 | 판단 |
| --- | --- | --- | --- | --- |
| A. 최소 headless core | `Wait`만 처리하는 `GameSession`, replay hash, CLI runner 구현 | 범위가 작고 Phase 1 검증에 정확히 부합, Phase 2가 확장하기 쉬움 | 실제 게임성은 아직 없음 | **채택** |
| B. Phase 2 선행 포함 | map fixture와 movement까지 함께 구현 | 눈에 보이는 동작이 빠름 | Phase 1 경계 위반, 테스트/문서 동기화 범위 증가 | 기각 |
| C. 레거시 포트 재사용 | 기존 loop/RNG 일부를 import해 빠르게 runner 구성 | 단기 구현량 감소 가능 | 직접 import 금지와 Legion/ECS 제외 원칙 위반 위험 | 기각 |

## 3. 구현 설계 초안

### 예상 파일 경계

- `Cargo.toml`: package/bin/dependency 정의. 레거시 workspace 제외.
- `src/main.rs`: UI 미구현 안내만 출력.
- `src/bin/aihack-headless.rs`: `--seed u64`, `--turns u64` 파싱, 반복 제출, replay 저장, final hash 출력.
- `src/core/mod.rs`: core module export.
- `src/core/rng.rs`: `GameRng`.
- `src/core/action.rs`: Phase 1 `CommandIntent::Wait` 중심 enum. 후속 variant는 문서 계약에 맞춰 선언 가능하나 미구현 처리.
- `src/core/session.rs`: `GameSession`, `GameMeta`, `RunState`, `submit`.
- `src/core/turn.rs`: `TurnOutcome`, `SnapshotHash`.
- `src/core/event.rs`: `GameEvent::TurnStarted`, `GameEvent::Message` 또는 Phase 1 최소 이벤트.
- `src/core/replay.rs`: replay line 직렬화와 final hash 계산.
- `tests/phase1_headless.rs`: deterministic contract 테스트.

### 최소 타입 계약

```rust
pub struct GameRng { seed: u64, /* deterministic inner rng */ }

pub enum CommandIntent { Wait }

pub enum RunState { Playing }

pub struct GameSession {
    pub meta: GameMeta,
    pub rng: GameRng,
    pub turn: u64,
    pub state: RunState,
    pub event_log: Vec<GameEvent>,
}

pub struct TurnOutcome {
    pub accepted: bool,
    pub turn_advanced: bool,
    pub events: Vec<GameEvent>,
    pub snapshot_hash: SnapshotHash,
    pub next_state: RunState,
}
```

Phase 1에서 `world`는 map/entity 의미를 갖지 않는다. `spec.md`의 최종 계약과 충돌하지 않도록 후속 확장을 위한 빈/최소 snapshot 필드만 허용한다.

### replay line 예시

```json
{"seed":42,"turn_before":0,"command":"Wait","accepted":true,"turn_after":1,"events":[{"TurnStarted":{"turn":1}}],"snapshot_hash":"..."}
```

## 4. 실행 순서

1. **스캐폴딩**
   - `Cargo.toml`과 기본 bin 2개를 만든다.
   - dependencies는 `serde`, `serde_json`, `thiserror`, `rand`, `clap`까지만 사용한다.
   - `.gitignore`에 `target/`, `runtime/` 유지 여부를 확인한다.

2. **core 최소 타입 추가**
   - `GameRng`, `CommandIntent::Wait`, `RunState::Playing`, `GameEvent`, `TurnOutcome`을 serde 가능한 타입으로 고정한다.
   - 모든 주석은 한국어로 작성한다.

3. **`GameSession::new(seed)`와 `submit(Wait)` 구현**
   - `new(42)`는 `turn=0`, `state=Playing`, 빈 event log로 시작한다.
   - `Wait`은 항상 `accepted=true`, `turn_advanced=true`, `turn += 1`.
   - 매 턴 deterministic event를 생성한다.

4. **snapshot/replay hash 구현**
   - hash 입력은 seed, turn, state, event log 또는 안정 snapshot JSON으로 제한한다.
   - wall-clock time, 파일 경로, stdout 문자열, UI 효과는 hash에 넣지 않는다.

5. **headless runner 연결**
   - `--seed 42 --turns 100` 실행 시 `runtime/replays/42-100.jsonl` 생성.
   - stdout에 seed, turns, final hash, replay path를 출력한다.

6. **문서 동기화 준비**
   - 구현 PR에서는 `CHANGELOG.md`, `implementation_summary.md`, 필요 시 `BUILD_GUIDE.md`에 Phase 1 완료 사실을 한국어로 반영한다.
   - 본 계획 단계에서는 문서 수정 범위를 정의만 하고 코드/문서 구현은 하지 않는다.

## 5. 테스트/검증 명세 초안

### 단위 테스트

- `game_rng_same_seed_same_sequence`: seed 42의 첫 N개 값이 두 인스턴스에서 동일.
- `game_session_new_sets_seed_and_turn_zero`: `GameSession::new(42)` 초기 상태 검증.
- `wait_advances_one_turn`: `submit(Wait)` 후 `turn=1`, `accepted=true`, `turn_advanced=true`.
- `snapshot_hash_stable_for_same_seed_and_turns`: seed 42, 100 waits hash 동일.

### 통합 테스트/명령

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 100
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

완료 기준:

- 두 번 실행한 final hash가 동일하다.
- `runtime/replays/42-100.jsonl`이 생성된다.
- JSONL line count는 `100`이다.
- 레거시 경로가 새 `Cargo.toml` 또는 `src/`에서 import되지 않는다.

추가 금지 검증:

```bash
rg "legacy_nethack_port_reference|legion" Cargo.toml src tests
```

결과는 match 없음이어야 한다.

## 6. PRD/test-spec 변환 포인트

### PRD story 후보

1. 개발자로서 새 Cargo 프로젝트를 빌드/테스트할 수 있다.
2. 개발자로서 seed 기반 `GameSession`을 만들고 `Wait` 턴을 진행할 수 있다.
3. QA로서 같은 seed/headless replay의 final hash가 동일함을 검증할 수 있다.

### acceptance criteria

- `GameSession::new(42)`는 panic 없이 deterministic 초기 상태를 반환한다.
- `CommandIntent::Wait` 100회 제출은 정확히 100턴을 진행한다.
- `TurnOutcome.snapshot_hash`는 같은 seed/turn stream에서 동일하다.
- CLI replay 파일은 deterministic JSONL로 기록된다.
- TUI, map/movement/combat/item/AI adapter, ECS/Legion, 레거시 직접 import는 포함되지 않는다.

## 7. ADR 초안

Decision: Phase 1은 `Wait` 전용 최소 headless core로 구현한다.  
Drivers: 재현성, 작은 PR, 레거시/컴포넌트 경계 차단.  
Alternatives considered: Phase 2 선행 포함, 레거시 포트 재사용.  
Why chosen: `audit_roadmap.md`의 Phase 1 완료 기준을 가장 작은 변경으로 만족한다.  
Consequences: 게임성은 아직 없지만, 이후 map/movement/combat가 붙을 deterministic baseline이 생긴다.  
Follow-ups: Phase 2에서 `GameMap`, movement, doors, vision을 별도 PRD/test-spec으로 작성한다.

## 8. 후속 실행 핸드오프 가이드

- 단일 순차 실행 권장: `$ralph` 또는 executor 1명.
- 병렬 팀이 필요하다면 2 lane까지만 사용한다.
  - Lane A: Cargo/CLI/replay 파일 경계.
  - Lane B: core 타입/session/tests.
- 검증 담당은 마지막에 `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, headless double-run hash 비교를 수행한다.
