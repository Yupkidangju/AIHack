# PRD: AIHack Phase 1 Headless Core

문서 상태: approved-plan
작성일: 2026-04-28
기준 문서: `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, `BUILD_GUIDE.md`, `DESIGN_DECISIONS.md`
컨텍스트 스냅샷: `.omx/context/phase1-headless-core-20260428T063401Z.md`
범위: Phase 1만 해당. 구현은 이 문서 승인 이후 별도 실행 모드에서 수행한다.

## 1. 목표

루트에 새 Rust 패키지를 만들고, seed 기반 deterministic `GameSession`이 `Wait` 명령만으로 턴을 진행하며, headless runner가 최종 snapshot hash를 안정적으로 출력하게 한다.

Phase 1의 완료 상태는 다음 질문에 `예`로 답할 수 있어야 한다.

- 같은 seed와 같은 turns를 두 번 실행하면 같은 final hash가 나오는가?
- 루트 새 코드가 레거시 코드를 직접 import하지 않는가?
- Phase 2 이상의 map/movement/combat/item/TUI 기능이 끼어들지 않았는가?
- `cargo test`와 headless runner가 통과하는가?

## 2. RALPLAN-DR 요약

### 2.1 원칙

1. Phase 1은 첫 deterministic runner만 만든다.
2. 모든 상태 변경은 `GameSession::submit()` 경계 안에서만 일어난다.
3. RNG는 `GameRng` wrapper를 통해서만 사용한다.
4. Headless 검증이 UI/TUI보다 먼저다.
5. 레거시는 참조만 가능하고 직접 import/workspace 편입은 금지한다.

### 2.2 결정 동인

| 순위 | 동인 | 의미 |
| --- | --- | --- |
| 1 | 재현성 | 같은 seed와 같은 명령열은 같은 final hash를 만들어야 한다. |
| 2 | 범위 잠금 | Phase 2 이상의 map/movement/combat/item은 만들지 않는다. |
| 3 | 후속 확장성 | Phase 2+가 붙을 수 있도록 타입/모듈 경계는 문서명과 맞춘다. |

### 2.3 대안 검토

| 옵션 | 설명 | 장점 | 단점 | 판정 |
| --- | --- | --- | --- | --- |
| A. 최소 CLI scaffold | `BUILD_GUIDE.md` 수준의 Cargo와 echo runner만 구현 | 가장 빠름 | Phase 1 audit의 `GameRng`, `GameSession`, `TurnOutcome`, hash를 만족하지 못함 | 기각 |
| B. Phase 1 core skeleton | Cargo + ids/rng/action/event/turn/session/snapshot/hash + Wait runner | 문서 요구 충족, 후속 Phase 기반 제공 | 타입 파일 수가 조금 늘어남 | 선택 |
| C. Phase 2 선행 scaffold 포함 | map/pos/movement stub까지 같이 생성 | 후속 작업이 쉬워 보임 | scope creep, 문서보다 코드가 앞섬 | 기각 |

선택: **옵션 B. Phase 1 core skeleton**.

## 3. 포함 범위

### 3.1 파일

| 파일 | 책임 |
| --- | --- |
| `Cargo.toml` | 새 루트 패키지 정의. 레거시 workspace member 금지 |
| `src/main.rs` | UI adapter 미구현 안내만 출력 |
| `src/bin/aihack-headless.rs` | seed/turns CLI, wait-only simulation 실행, final hash 출력 |
| `src/core/mod.rs` | core module export |
| `src/core/ids.rs` | `EntityId`, `LevelId`, `BranchId` 최소 타입 |
| `src/core/rng.rs` | `GameRng` seed wrapper와 deterministic API |
| `src/core/error.rs` | `GameError`, `GameResult` 최소 타입 |
| `src/core/action.rs` | `CommandIntent::Wait`, `CommandIntent::Quit` 최소 명령 |
| `src/core/event.rs` | `GameEvent::TurnStarted`, `GameEvent::Waited`, `GameEvent::CommandRejected` 최소 이벤트 |
| `src/core/turn.rs` | `TurnOutcome`, `SnapshotHash` |
| `src/core/session.rs` | `GameSession::new(seed)`, `submit()` wait loop |
| `src/core/snapshot.rs` | Phase 1 snapshot과 deterministic hash 입력 |

### 3.2 타입 계약

```rust
pub struct EntityId(pub u32);

pub struct LevelId {
    pub branch: BranchId,
    pub depth: i16,
}

pub enum BranchId {
    Main,
}

pub enum CommandIntent {
    Wait,
    Quit,
}

pub enum RunState {
    Playing,
    GameOver,
}

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

Phase 1에서는 `world` 전체 구조를 만들지 않는다. `GameSession`의 최종 v0.1 계약과 다르게 보일 수 있는 항목은 후속 Phase에서 확장 가능하도록 최소 필드만 둔다.

## 4. 제외 범위 / 비목표

- TUI, 마우스, ASCII 애니메이션, 현대 UX 구현.
- map fixture, movement, doors, vision.
- combat, monster AI, items, inventory.
- save/load JSON persistence.
- AI `Observation`/`ActionSpace` 전체 구현.
- 레거시 source direct import.
- Legion/ECS 도입.
- Phase 2 이상을 위한 빈 스텁 남발.

## 5. 구현 순서

### Step 1: Cargo 스캐폴딩

- `BUILD_GUIDE.md`의 초기 패키지 구성을 따른다.
- `serde`, `serde_json`, `thiserror`, `rand`, `clap`만 초기 후보로 허용한다.
- 레거시 폴더를 workspace member로 넣지 않는다.

### Step 2: Core 타입 뼈대

- `ids`, `rng`, `error`, `action`, `event`, `turn`, `snapshot`, `session` 순서로 생성한다.
- 공개 타입명은 `spec.md`와 충돌하지 않게 둔다.
- 코드 주석은 한국어로 작성한다.

### Step 3: Deterministic RNG

- `GameRng::new(seed)`를 제공한다.
- 외부 시스템은 `rand`를 직접 쓰지 않고 `GameRng`를 통해서만 난수를 얻는다.
- Phase 1에서는 hash 안정성 테스트를 위해 최소 1개 RNG snapshot test를 둔다.

### Step 4: Wait-only submit loop

- `GameSession::new(seed)`는 `turn = 0`, `RunState::Playing`으로 시작한다.
- `submit(CommandIntent::Wait)`은 `accepted=true`, `turn_advanced=true`, `turn += 1`을 보장한다.
- `submit(CommandIntent::Quit)`은 필요 시 `RunState::GameOver`로 전환하되, Phase 1 runner는 기본적으로 사용하지 않는다.

### Step 5: Snapshot hash

- final hash는 seed, turn, run_state, event summary를 포함한다.
- Rust `DefaultHasher`처럼 버전/플랫폼 안정성을 보장하기 어려운 해시는 피한다.
- 권장: stable string/serde JSON 입력 + 자체 FNV-1a 64-bit 함수.

### Step 6: Headless runner

- `--seed <u64>` 기본값은 42.
- `--turns <u64>` 기본값은 1000 또는 `BUILD_GUIDE.md`와 일치시킨다.
- runner는 turns만큼 `Wait`을 제출한다.
- stdout에는 최소 `seed`, `turns`, `final_turn`, `final_hash`가 포함된다.

### Step 7: 문서 동기화

- Phase 1 구현 완료 시 `CHANGELOG.md`에 구현 항목을 추가한다.
- 코드가 문서 계약과 달라질 경우 구현보다 문서 동기화를 우선한다.

## 6. 수용 기준

| ID | 기준 | 검증 |
| --- | --- | --- |
| AC-1 | `cargo test` 통과 | `cargo test` |
| AC-2 | seed 42, turns 0 실행 성공 | `cargo run --bin aihack-headless -- --seed 42 --turns 0` |
| AC-3 | seed 42, turns 100 deterministic | 같은 명령 2회 final_hash 비교 |
| AC-4 | seed 차이가 hash에 반영 | seed 42와 43의 turns 100 final_hash 비교 |
| AC-5 | 레거시 직접 의존 없음 | `rg "legacy_nethack_port_reference" src Cargo.toml` 결과 없음 |
| AC-6 | 레거시 workspace 편입 없음 | `Cargo.toml`에 workspace member로 legacy 없음 |
| AC-7 | Phase 2 이상 구현 없음 | map/movement/combat/item/TUI 모듈 생성 없음 또는 빈 선행 스텁 없음 |
| AC-8 | Rust 품질 게이트 통과 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` |

## 7. 리스크와 완화

| 리스크 | 영향 | 완화 |
| --- | --- | --- |
| Phase 1에서 Phase 2 스텁이 늘어남 | 범위 오염 | 파일 목록을 본 PRD 3.1로 제한 |
| hash가 플랫폼/버전에 따라 흔들림 | replay 검증 실패 | 자체 FNV-1a 같은 고정 알고리즘 사용 |
| `rand` 직접 사용 확산 | 결정론 경계 붕괴 | `GameRng` 외부 난수 접근 금지 |
| runner가 echo 수준에 머묾 | Phase 1 audit 미충족 | `GameSession::submit(Wait)` 실제 호출 필수 |
| 레거시 직접 import 유혹 | 라이선스/구조 리스크 | `rg` 감사와 workspace member 금지 |

## 8. ADR

### 결정

Phase 1은 `BUILD_GUIDE.md`의 단순 scaffold보다 조금 넓은 **wait-only deterministic core skeleton**으로 구현한다.

### Drivers

- `audit_roadmap.md`의 Phase 1 항목을 모두 만족해야 한다.
- 후속 Phase의 map/movement/combat를 위한 안정적 `GameSession` 경계가 필요하다.
- headless runner가 첫 실행 대상이라는 ADR-0006을 따라야 한다.

### Alternatives considered

- Echo runner only: Phase 1 감사 기준 미달.
- Phase 2 타입까지 선행 생성: scope creep.

### Why chosen

옵션 B는 deterministic replay/hash 기반을 만들면서도 gameplay 범위는 `Wait`에 잠근다.

### Consequences

- Phase 1부터 core 파일 수는 생기지만, 각 파일 책임은 작고 테스트 가능하다.
- Phase 2는 map/movement를 추가할 때 기존 `GameSession::submit()` 경계를 확장하면 된다.

### Follow-ups

- Phase 1 완료 후 Phase 2 PRD/test-spec을 별도로 생성한다.
- `SaveDataV1`, `Observation`, TUI는 각각 문서상 후속 Phase에서만 다룬다.

## 9. Ralph/Team 후속 실행 지침

### 권장 실행 방식

- 권장: `$ralph .omx/plans/prd-phase1-headless-core.md`
- 이유: Phase 1은 단일 순차 구현이 적합하고 병렬 팀 오버헤드가 크다.

### 사용 가능한 agent type roster

| 역할 | 용도 | 권장 reasoning |
| --- | --- | --- |
| `executor` | Phase 1 구현 | medium |
| `test-engineer` | deterministic/hash/test 기준 보강 | medium |
| `build-fixer` | cargo/clippy 실패 수정 | high |
| `verifier` | 완료 주장 검증 | high |
| `code-reviewer` | 구현 후 품질 검토 | high |

### Ralph 실행 중 금지

- Phase 2로 임의 진행 금지.
- TUI 또는 현대 UX 구현 금지.
- 레거시 직접 import 금지.
- 테스트 실패 상태에서 완료 보고 금지.

## 10. Consensus 결과

- Planner: 승인. 옵션 B가 Phase 1 요구를 가장 잘 만족한다.
- Architect: 승인. `GameSession`/`GameRng`/stable hash 경계가 후속 확장에 적합하다.
- Critic: 승인. acceptance criteria와 검증 명령이 구체적이며 scope creep 방지가 명확하다.
