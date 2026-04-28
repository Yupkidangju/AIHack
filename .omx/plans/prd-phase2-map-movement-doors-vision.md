# PRD: AIHack Phase 2 Map, Movement, Doors, Vision

문서 상태: approved-plan
작성일: 2026-04-28
기준 문서: `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, `BUILD_GUIDE.md`, `DESIGN_DECISIONS.md`
컨텍스트 스냅샷: `.omx/context/phase2-map-movement-doors-vision-20260428T070214Z.md`
이전 단계: Phase 1 Headless Core 완료
범위: Phase 2만 해당. 이 문서는 계획 산출물이며 구현은 별도 실행 모드에서 수행한다.

## 1. 목표

40x20 fixture map에서 플레이어가 deterministic하게 움직이고, 벽/닫힌 문/열린 문이 movement와 LOS에 일관되게 반영되며, 최소 `Observation.visible_tiles`를 생성한다.

Phase 2 완료 상태는 다음 질문에 `예`로 답할 수 있어야 한다.

- `GameSession`이 Phase 1 wait-only 상태에서 `world`를 소유하는 구조로 확장되었는가?
- fixture level 40x20에서 player_start `(5,5)`가 고정되는가?
- 벽과 닫힌 문은 movement와 LOS를 막고, 열린 문은 movement와 LOS를 통과시키는가?
- rejected command는 turn을 진행하지 않고 final hash도 deterministic하게 유지되는가?
- `Observation.visible_tiles`가 반경 8과 LOS 차단 규칙에 따라 생성되는가?

## 2. RALPLAN-DR 요약

### 2.1 원칙

1. Phase 2는 map/movement/doors/vision만 만든다.
2. `GameSession`은 계속 단일 상태 원천이어야 한다.
3. UI/TUI, combat, item, monster AI, save/load는 구현하지 않는다.
4. LOS와 movement는 같은 tile/door blocker 규칙을 공유하되, 시야와 이동의 결과 타입은 분리한다.
5. 모든 fixture와 테스트는 seed 없이도 deterministic하거나, seed 42와 결합되어 stable hash를 만든다.

### 2.2 결정 동인

| 순위 | 동인 | 의미 |
| --- | --- | --- |
| 1 | 경계 일관성 | movement, doors, vision이 모두 `GameSession::submit()` 경계에서 변경/조회되어야 한다. |
| 2 | 범위 잠금 | entity store, combat, items, TUI를 끌어오지 않고 player position만 최소 상태로 둔다. |
| 3 | 검증 가능성 | 각 규칙은 unit test와 headless hash 검증으로 증명 가능해야 한다. |

### 2.3 대안 검토

| 옵션 | 설명 | 장점 | 단점 | 판정 |
| --- | --- | --- | --- | --- |
| A. Map만 구현 | `GameMap` fixture와 tile 접근만 추가 | 작고 안전함 | Phase 2의 movement/doors/vision 완료 기준 미달 | 기각 |
| B. Minimal world + player_pos | `GameWorld { map, player_pos }`와 movement/doors/vision 구현 | Phase 2 충족, entity store 없이 범위 유지 | Phase 3 이후 entity store로 확장 필요 | 선택 |
| C. Entity store까지 선행 | player entity store를 지금 도입 | 후속 Phase와 형태가 가까움 | Task 4/Phase 3+ 범위까지 확장되어 scope creep | 기각 |

선택: **옵션 B. Minimal world + player_pos**.

## 3. 포함 범위

### 3.1 신규/변경 파일

| 파일 | 책임 |
| --- | --- |
| `src/core/position.rs` | `Pos`, `Delta`, `Direction`와 좌표 연산 |
| `src/domain/mod.rs` | domain module export |
| `src/domain/tile.rs` | `TileKind`, `DoorState`, movement/LOS blocker 판정 |
| `src/domain/map.rs` | `GameMap`, 40x20 fixture, bounds/tile access |
| `src/core/world.rs` | `GameWorld { map, player_pos }` 최소 world |
| `src/core/observation.rs` | `Observation`, `TileObservation`, `visible_tiles` 최소 스키마 |
| `src/systems/mod.rs` | systems module export |
| `src/systems/movement.rs` | movement validation/apply |
| `src/systems/doors.rs` | open/close validation/apply |
| `src/systems/vision.rs` | LOS radius 8 visible tile 계산 |
| `src/core/action.rs` | `Move`, `Open`, `Close` 명령 추가 |
| `src/core/event.rs` | `EntityMoved`, `DoorChanged`, movement/door reject reason 확장 |
| `src/core/session.rs` | `world` 소유, submit routing 확장 |
| `src/core/snapshot.rs` | map/player/door state hash 입력 확장 |
| `src/bin/aihack-headless.rs` | Phase 2 smoke command 옵션은 선택. 기본 wait runner는 유지 |
| `tests/map.rs` | fixture/bounds/tile tests |
| `tests/movement.rs` | movement blocker/accepted/rejected tests |
| `tests/doors.rs` | open/close and turn behavior tests |
| `tests/vision.rs` | LOS radius/blocker tests |

### 3.2 타입 계약

```rust
pub struct Pos {
    pub x: i16,
    pub y: i16,
}

pub struct Delta {
    pub dx: i16,
    pub dy: i16,
}

pub enum Direction {
    North,
    South,
    West,
    East,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

pub enum TileKind {
    Wall,
    Floor,
    Door(DoorState),
    StairsDown,
    StairsUp,
}

pub enum DoorState {
    Closed,
    Open,
}

pub struct GameMap {
    pub width: i16,
    pub height: i16,
    tiles: Vec<TileKind>,
}

pub struct GameWorld {
    pub map: GameMap,
    pub player_pos: Pos,
}

pub struct Observation {
    pub schema_version: u16,
    pub seed: u64,
    pub turn: u64,
    pub player_pos: Pos,
    pub visible_tiles: Vec<TileObservation>,
    pub legal_actions: Vec<CommandIntent>,
}

pub struct TileObservation {
    pub pos: Pos,
    pub rel: Delta,
    pub tile: TileKind,
    pub visible: bool,
}
```

계약:

- `GameMap`은 row-major `tiles[y * width + x]`로 저장한다.
- `GameMap::tile(pos)`는 bounds 밖에서 panic하지 않고 `GameError`를 반환한다.
- Phase 2에서 diagonal corner-cutting은 금지한다. 대각 이동은 양쪽 orthogonal neighbor 중 하나라도 blocker이면 rejected 처리한다.
- `CommandIntent::Move(dir)`는 이동 가능하면 turn을 진행한다.
- `CommandIntent::Open(dir)`/`Close(dir)`는 adjacent door만 대상으로 한다.
- rejected movement/open/close는 `accepted=false`, `turn_advanced=false`다.
- LOS는 radius 8, Chebyshev distance 기준 후보 tile을 대상으로 하며 Bresenham line 또는 동등한 grid ray를 사용한다.
- LOS에서 wall과 closed door는 blocker이고 open door는 blocker가 아니다.

## 4. Fixture 데이터

Phase 2 fixture는 `GameMap::fixture_phase2()` 또는 동등 API로 제공한다.

```text
size = 40x20
player_start = (5, 5)
stairs_down = (34, 15)
closed_door = (10, 5)
open_door_candidate = (14, 5)
wall_line = x=0..39 at y=0 and y=19, y=0..19 at x=0 and x=39
interior_wall = (12, 4)..(12, 8)
```

권장 ASCII 예시:

```text
########################################
#......................................#
#......................................#
#......................................#
#...........#..........................#
#....@....+.#.+...................>....#
#...........#..........................#
#...........#..........................#
#...........#..........................#
#......................................#
#......................................#
#......................................#
#......................................#
#......................................#
#......................................#
#.................................>....#
#......................................#
#......................................#
#......................................#
########################################
```

주의:

- `@`는 저장 tile이 아니라 `player_pos` 표시 예시다.
- `+`는 closed door로 로드한다. open door test는 open command 후 상태 변화로 만든다.
- `>`는 stairs down tile이며 Phase 2에서는 이동/LOS 통과 가능한 floor-like tile로 취급한다.

## 5. 제외 범위 / 비목표

- 몬스터, 아이템, entity store 전체 구현.
- combat, bump attack, death.
- stairs level transition.
- save/load/replay file persistence.
- TUI, 마우스, ASCII effect.
- AI adapter 전체. 단, `Observation.visible_tiles` 최소 스키마는 Phase 2 완료 기준이라 포함한다.
- pathfinding/autotravel.
- darkness/light source/fog of war memory. 문서의 “어둠/광원 시스템은 v0.2” 문구는 후속 별도 Phase에서 다룬다. Phase 2는 radius+LOS만 구현한다.

## 6. 구현 순서

### Step 1: Position/Direction

- `Pos`, `Delta`, `Direction`을 추가한다.
- 8방향 delta 변환을 테스트한다.
- bounds와 좌표 연산은 signed integer overflow를 피한다.

### Step 2: Tile/Map fixture

- `TileKind`, `DoorState`, `GameMap`을 추가한다.
- 40x20 fixture를 코드 fixture 또는 `tests/fixtures` 데이터로 만든다.
- bounds 밖 접근은 `GameError`로 실패한다.

### Step 3: GameWorld와 snapshot 확장

- `GameSession`에 `world: GameWorld`를 추가한다.
- `GameSession::new(seed)`는 Phase 2 fixture world를 생성한다.
- snapshot hash에는 player_pos와 map/door summary를 포함한다.
- Phase 1 wait hash 값 변경은 허용하되, `DESIGN_DECISIONS.md`/`CHANGELOG.md`에 hash 입력 변경으로 기록해야 한다.

### Step 4: Movement system

- `Move(dir)` validation/apply를 구현한다.
- floor/open door/stairs는 통과 가능.
- wall/closed door/out-of-bounds는 rejected.
- accepted move는 turn을 진행하고 `EntityMoved` event를 생성한다.

### Step 5: Doors system

- `Open(dir)`은 adjacent closed door를 open으로 바꾼다.
- `Close(dir)`은 adjacent open door를 closed로 바꾼다.
- adjacent door가 없거나 이미 원하는 상태이면 rejected.
- accepted door change는 turn을 진행하고 `DoorChanged` event를 생성한다.

### Step 6: Vision/Observation

- `visible_tiles`를 radius 8과 LOS blocker 규칙으로 생성한다.
- player tile은 항상 visible이다.
- wall/closed door tile 자체는 보이지만 그 너머는 막힌다.
- `legal_actions`는 Phase 2 최소 범위에서 wait, valid adjacent move/open/close 후보를 생성한다.

### Step 7: Headless smoke 확장

- 기본 wait-only runner는 유지한다.
- 선택적으로 `--script "wait,move:e,open:e"` 같은 command script는 Phase 2 범위에 포함하지 않는다. 필요하면 후속 Phase에서 계획한다.
- Phase 2 검증은 주로 unit/integration tests로 수행한다.

### Step 8: 문서 동기화

- 구현 후 `CHANGELOG.md`, `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, 필요 시 `DESIGN_DECISIONS.md`를 갱신한다.
- snapshot hash 입력 변경이 있으면 기존 Phase 1 기준 hash와 새 기준 hash를 모두 문서에 기록한다.

## 7. 수용 기준

| ID | 기준 | 검증 |
| --- | --- | --- |
| AC-1 | 40x20 fixture map 생성 | `cargo test map_fixture_is_40x20` |
| AC-2 | bounds 밖 tile 접근은 panic이 아니라 error | `cargo test map_bounds_returns_error` |
| AC-3 | player_start는 `(5,5)` | `cargo test session_starts_player_at_fixture_start` |
| AC-4 | floor 이동 accepted, turn +1 | `cargo test movement_to_floor_advances_turn` |
| AC-5 | wall/closed door/out-of-bounds 이동 rejected, turn 유지 | `cargo test movement_blockers_do_not_advance_turn` |
| AC-6 | closed door open 후 movement/LOS 통과 | `cargo test open_door_allows_movement_and_los` |
| AC-7 | open door close 후 movement/LOS 차단 | `cargo test close_door_blocks_movement_and_los` |
| AC-8 | LOS radius 8 적용 | `cargo test vision_respects_radius_8` |
| AC-9 | wall/closed door는 LOS 차단, open door는 통과 | `cargo test vision_respects_door_blockers` |
| AC-10 | `Observation.visible_tiles` 생성 | `cargo test observation_contains_visible_tiles` |
| AC-11 | rejected command는 `accepted=false`, `turn_advanced=false` | `cargo test rejected_commands_do_not_advance_turn` |
| AC-12 | legacy direct import 없음 | `rg "legacy_nethack_port_reference" src Cargo.toml` 결과 없음 |
| AC-13 | 품질 게이트 통과 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |

## 8. 리스크와 완화

| 리스크 | 영향 | 완화 |
| --- | --- | --- |
| Entity store를 조기 도입 | Phase 2 범위 확장 | `GameWorld.player_pos` 최소 상태만 허용 |
| LOS와 movement blocker 규칙 불일치 | 문/벽 동작 버그 | `TileKind` helper를 공유하고 door/movement/vision 교차 테스트 추가 |
| 대각 이동 corner-cutting 버그 | 벽 통과처럼 보이는 이동 | 대각 이동 blocker 정책을 테스트로 고정 |
| Snapshot hash 기준 변경 혼란 | Phase 1 hash와 비교 어려움 | hash 입력 변경을 문서/CHANGELOG/ADR에 명시 |
| Observation 전체 AI API로 확장 | Phase 11 범위 침범 | `visible_tiles`와 최소 legal_actions만 포함 |

## 9. ADR

### 결정

Phase 2는 전체 entity store가 아니라 `GameWorld { map, player_pos }` 최소 world로 구현한다.

### Drivers

- `audit_roadmap.md` Phase 2는 map/movement/doors/vision 완료가 목표다.
- 현재 Phase 1에는 world가 없으므로 map과 player 위치를 보관할 최소 상태가 필요하다.
- entity store, monster, item은 후속 Phase 범위다.

### Alternatives considered

- Map-only: movement/doors/vision 완료 기준을 만족하지 못한다.
- Full entity store: 후속 Phase 작업을 끌어와 scope creep을 만든다.

### Why chosen

`GameWorld { map, player_pos }`는 Phase 2 완료 기준을 만족하면서도 후속 entity store 도입을 막지 않는 가장 작은 상태 확장이다.

### Consequences

- Phase 2는 player를 `EntityId` 기반 entity로 취급하지 않는다.
- Phase 3+에서 entity store가 도입될 때 `player_pos`는 player entity position component로 이전될 수 있다.
- snapshot hash 입력이 map/player/door state를 포함하도록 바뀌므로 기준 hash 문서 갱신이 필요하다.

### Follow-ups

- Phase 3/4 계획에서 entity store 도입 시 migration boundary를 별도 PRD에 기록한다.
- TUI는 Phase 10 전까지 이 world에 직접 접근하지 않는다.

## 10. Ralph/Team 후속 실행 지침

### 권장 실행 방식

- 권장: `$ralph .omx/plans/prd-phase2-map-movement-doors-vision.md`
- 이유: Phase 2는 여러 모듈을 추가하지만 경계가 강하게 연결되어 있어 단일 persistence loop가 안전하다.

### Team이 필요한 경우

`$team`은 다음 조건에서만 고려한다.

- movement/doors/vision 테스트가 반복 실패하여 독립 진단 lane이 필요할 때.
- LOS 알고리즘과 map fixture 설계를 병렬 검토해야 할 때.

### 사용 가능한 agent type roster

| 역할 | 용도 | 권장 reasoning |
| --- | --- | --- |
| `executor` | Phase 2 구현 | medium |
| `test-engineer` | map/movement/doors/vision regression test 설계 | medium |
| `debugger` | LOS 또는 rejected command failure 분석 | high |
| `build-fixer` | cargo/clippy 실패 수정 | high |
| `architect` | world/entity-store 경계 검증 | high |
| `verifier` | PRD/test-spec 충족 증거 확인 | high |
| `code-reviewer` | 구현 후 범위/품질 검토 | high |

### Ralph 실행 중 금지

- Phase 3 combat/death로 진행 금지.
- item/inventory/monster AI/TUI 구현 금지.
- 레거시 직접 import 금지.
- 테스트 실패 상태에서 완료 보고 금지.

## 11. Consensus 결과

- Planner: 승인. Minimal world + player_pos가 Phase 2 범위와 완료 기준을 가장 작게 만족한다.
- Architect: 승인. entity store를 미루면서도 `GameSession` 단일 상태 원천을 유지한다.
- Critic: 승인. blocker, LOS, rejected command, hash 변경, scope creep에 대한 테스트 기준이 명확하다.
