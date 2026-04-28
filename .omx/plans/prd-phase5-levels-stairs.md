# PRD: AIHack Phase 5 Levels and Stairs

문서 상태: approved-plan
작성일: 2026-04-29
기준 문서: `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, `DESIGN_DECISIONS.md`, `designs.md`
컨텍스트 스냅샷: `.omx/context/phase5-levels-stairs-20260428T171951Z.md`
이전 단계: Phase 4 Items and Inventory 완료
범위: Phase 5만 해당. 이 문서는 계획 산출물이며 구현은 별도 실행 모드에서 수행한다.

## 1. 목표

Phase 4의 단일 level world를 최소 multi-level world로 확장한다. 플레이어는 1층 stairs down에서 `Descend`로 2층에 내려가고, 2층 stairs up에서 `Ascend`로 1층에 돌아올 수 있어야 한다. 왕복 후 1층의 map/entity/item/inventory 상태는 보존되어야 하며, snapshot hash는 현재 level과 level별 state를 포함해야 한다.

Phase 5 완료 상태는 다음 질문에 `예`로 답할 수 있어야 한다.

- `LevelId { branch: Main, depth }`가 실제 current level과 level registry에 사용되는가?
- `GameWorld`가 1층과 2층의 `GameMap` 및 level-local state를 보존하는가?
- `CommandIntent::Descend`와 `CommandIntent::Ascend`가 stairs tile에서만 accepted + turn advance 되는가?
- `LevelChanged { entity, from, to }` event가 deterministic order로 생성되는가?
- 1층에서 potion pickup/door 상태 변경 후 2층 왕복 시 1층 상태가 유지되는가?
- Phase 6 monster AI, procedural generation, file save/load, TUI를 구현하지 않았는가?

## 2. RALPLAN-DR 요약

### 2.1 원칙

1. Phase 5는 fixed 2-level registry와 stairs transition만 추가한다.
2. `EntityId`는 전체 run에서 단일 namespace를 유지한다.
3. `EntityLocation::OnMap`은 level-aware location으로 확장한다.
4. Level transition은 `GameSession::submit()` 내부에서만 상태를 바꾼다.
5. File save/load와 procedural generation은 구현하지 않는다.

### 2.2 결정 동인

| 순위 | 동인 | 의미 |
| --- | --- | --- |
| 1 | State preservation | 1층 상태가 2층 왕복 후 그대로 유지되어야 한다. |
| 2 | Replay determinism | level transition event와 snapshot hash가 seed/command sequence로 재현되어야 한다. |
| 3 | Scope containment | monster AI/procgen/save/TUI를 끌어오지 않는다. |

### 2.3 대안 검토

| 옵션 | 설명 | 장점 | 단점 | 판정 |
| --- | --- | --- | --- | --- |
| A. `GameWorld { current_map, previous_map_backup }` | 단순히 1층 map을 백업 | 구현량 작음 | 3+ level 확장과 entity/item 위치 계약이 불안정 | 기각 |
| B. `GameWorld { levels: Vec<GameLevel>, current_level }` + 단일 `EntityStore` | level별 map 보존, entity id는 run-wide 유지 | Phase 5 요구와 후속 확장 균형 | `EntityLocation` 리팩터 필요 | 선택 |
| C. level별 `EntityStore` 분리 | level-local query 단순 | EntityId 중복/이동/이벤트 계약 복잡 | 기각 |
| D. procedural dungeon generator 도입 | 장기 roguelike 확장성 높음 | Phase 5 scope 초과, deterministic fixture 검증 어려움 | 기각 |

선택: **옵션 B. Level registry + 단일 EntityStore + level-aware EntityLocation**.

## 3. 포함 범위

### 3.1 신규/변경 파일

| 파일 | 책임 |
| --- | --- |
| `src/domain/level.rs` | `GameLevel`, `LevelRegistry`, fixed level fixture factory |
| `src/domain/map.rs` | level 1/2 fixture 생성과 stairs up/down 위치 상수 |
| `src/domain/entity.rs` | Actor/Item 위치를 level-aware `EntityLocation::OnMap { level, pos }` 계약으로 통일 |
| `src/core/action.rs` | `Descend`, `Ascend` 추가 |
| `src/core/event.rs` | `LevelChanged` 추가 |
| `src/core/world.rs` | `levels`, `current_level`, current map accessors |
| `src/core/session.rs` | descend/ascend submit routing |
| `src/core/snapshot.rs` | current level과 level registry state hash 입력 포함 |
| `src/core/observation.rs` | observation에 current level과 stairs legal actions 추가 |
| `src/systems/stairs.rs` | stairs transition validation/apply |
| `src/systems/movement.rs` | current level map accessor 사용 |
| `src/systems/vision.rs` | current level map accessor 사용 |
| `src/systems/doors.rs` | current level map accessor 사용 |
| `src/systems/items.rs` | level-aware item pickup |
| `tests/levels.rs` | level registry/snapshot/state preservation 검증 |
| `tests/stairs.rs` | descend/ascend/event/legal action 검증 |

### 3.2 핵심 타입 계약

```rust
pub struct GameWorld {
    pub levels: LevelRegistry,
    pub current_level: LevelId,
    pub entities: EntityStore,
    pub player_id: EntityId,
    pub inventory: Inventory,
}

pub struct LevelRegistry {
    pub levels: Vec<GameLevel>,
}

pub struct GameLevel {
    pub id: LevelId,
    pub map: GameMap,
}

pub enum EntityLocation {
    OnMap { level: LevelId, pos: Pos },
    Inventory { owner: EntityId },
    Consumed,
}

pub enum EntityPayload {
    Actor { /* existing actor fields */, location: EntityLocation },
    Item { /* existing item fields */, location: EntityLocation },
}

pub enum CommandIntent {
    // existing Phase 4 variants...
    Descend,
    Ascend,
}

pub enum GameEvent {
    // existing events...
    LevelChanged { entity: EntityId, from: LevelId, to: LevelId },
}
```

계약:

- `LevelId { branch: BranchId::Main, depth: 1 }`은 1층, `depth: 2`는 2층이다.
- `LevelRegistry`는 Phase 5에서 exactly 2 levels를 가진다.
- `LevelRegistry.levels`의 deterministic ordering은 `(branch, depth)` 정렬 또는 fixed insertion order `main:1 -> main:2`로 고정하며 snapshot 직렬화도 같은 순서를 사용한다.
- `GameWorld.current_level`은 player actor의 `EntityLocation::OnMap.level`과 항상 같아야 한다.
- Actor payload의 기존 `pos: Pos` 단독 계약은 `location: EntityLocation`으로 전환하며, actor는 Phase 5에서 `OnMap`만 허용한다.
- Item payload도 동일한 `EntityLocation`을 사용하되 inventory/consumed 상태는 기존 Phase 4 의미를 유지한다.
- Actor/item map queries는 `alive_actor_at(level, pos)`, `alive_hostile_at(level, pos)`, `items_at(level, pos)`처럼 level을 명시하거나 `current_level`을 기본으로 필터링한다.
- Player 위치 변경은 `player_location()`/`set_player_location(level, pos)` 또는 동등한 atomic helper를 통해 수행하여 `current_level`과 actor location이 분리 갱신되지 않도록 한다.
- Inventory/Consumed item은 level을 갖지 않는다.
- `EntityId`는 level을 넘어 재사용하지 않는다.
- `Descend`/`Ascend`는 현재 tile이 각각 `StairsDown`/`StairsUp`일 때만 accepted다.
- Transition landing position은 반대 stairs tile이다.
- Phase 5는 cross-level pathfinding, monster follow, level generation을 구현하지 않는다.

## 4. Fixture 데이터

### 4.1 Level 1

Phase 4 fixture를 유지한다.

```text
level = main:1
size = 40x20
player_start = EntityId(1), OnMap { level=main:1, pos=(5,5) }
jackal = EntityId(2), OnMap { level=main:1, pos=(6,5) }
goblin = EntityId(3), OnMap { level=main:1, pos=(20,12) }
stairs_down = (34,15)
closed_doors = [(10,5), (14,5)]
interior_wall = x=12, y=4..8
potion_healing = EntityId(4), OnMap { level=main:1, pos=(8,5) }
dagger = EntityId(5), Inventory(player), letter='a'
food_ration = EntityId(6), Inventory(player), letter='b'
```

### 4.2 Level 2

Phase 5 fixed fixture다.

```text
level = main:2
size = 40x20
stairs_up = (5,5)
stairs_down = none
player_landing_from_level1 = (5,5)
closed_doors = [(8,5)]
interior_wall = x=18, y=3..10
items = []
monsters = []
```

주의:

- 2층은 Phase 5 검증용 fixed map이다. Procedural map generation은 후속 Phase다.
- 2층에는 몬스터/아이템을 기본 배치하지 않는다. 1층 상태 보존과 stairs transition만 검증한다.
- 1층으로 ascend할 때 landing position은 1층 stairs down `(34,15)`이다.

## 5. Command 정책

### 5.1 Descend

- 현재 player position tile이 `TileKind::StairsDown`이면 accepted + turn advance.
- target level은 `LevelId { branch: Main, depth: current.depth + 1 }`이다.
- target level이 registry에 없으면 rejected + no turn advance.
- player location은 target level의 `StairsUp` position으로 이동한다.
- event order: `TurnStarted`, `LevelChanged { entity: player_id, from, to }`.

### 5.2 Ascend

- 현재 player position tile이 `TileKind::StairsUp`이면 accepted + turn advance.
- target level은 `LevelId { branch: Main, depth: current.depth - 1 }`이다.
- depth 1에서 ascend하면 rejected + no turn advance.
- player location은 target level의 `StairsDown` position으로 이동한다.
- event order: `TurnStarted`, `LevelChanged { entity: player_id, from, to }`.

### 5.3 Legal actions

- `Observation.legal_actions`는 현재 tile에 따라 `Descend` 또는 `Ascend`를 포함한다.
- Stairs tile은 기존 movement passable/LOS transparent 계약을 유지한다.
- `Move`로 stairs tile에 올라서는 것은 level transition이 아니다. Transition은 `Descend`/`Ascend` 명령에서만 발생한다.

## 6. Snapshot / Observation 정책

Snapshot 추가 입력:

- `current_level: LevelId`
- 모든 level의 `id`, `map_width`, `map_height`, `map_tiles`를 deterministic level order(`main:1 -> main:2`)로 직렬화
- entity item/actor locations의 `level`
- inventory/equipped/consumed state는 Phase 4와 동일

Observation 추가 입력:

```rust
pub struct Observation {
    // existing Phase 4 fields...
    pub current_level: LevelId,
}
```

정책:

- `visible_tiles`는 current level map에서만 생성한다.
- `inventory`는 level과 무관하게 유지된다.
- `legal_actions`는 current tile의 stairs state를 반영한다.

## 7. 제외 범위 / 비목표

- Procedural dungeon generation.
- 3층 이상 level 생성.
- Branch 전환(Main 외 Mines/Quest/Gehennom/Endgame).
- Monster AI, monster follow stairs, monster level persistence behavior.
- Item drop and cross-level item movement beyond existing OnMap/Inventory/Consumed location preservation.
- File save/load.
- TUI level transition animation/effect.
- Trap/teleport/level teleport.
- Stairs discovery/fog-of-war memory.

## 8. 구현 순서

### Step 0: 문서 충돌 선해결

- 코드 구현 전에 `audit_roadmap.md` Phase 5 완료 기준이 “1층 아이템 상태가 2층 왕복 후 유지”임을 확인하고, file save/load나 procedural generation을 요구하지 않도록 문구를 유지/보강한다.

### Step 1: Level domain

- `src/domain/level.rs` 추가.
- `GameLevel`, `LevelRegistry`와 `get/get_mut/current` helper 작성.
- `main:1`, `main:2` fixture 생성.

### Step 2: EntityLocation migration

- `EntityLocation::OnMap(Pos)`를 `OnMap { level, pos }`로 확장한다.
- Actor payload의 `pos: Pos` 직접 보관을 `location: EntityLocation`으로 전환하고 actor는 `OnMap`만 허용한다.
- Phase 4 item tests를 level-aware location으로 갱신한다.
- Actor/item query helpers는 level-aware로 전환하고 player/current-level invariant를 검증한다.
- `player_location()`/`set_player_location(level, pos)` 또는 동등한 atomic helper를 추가해 stairs transition이 current level과 player actor location을 한 경로로만 갱신하게 한다.

### Step 3: World migration

- `GameWorld.map`을 `GameWorld.levels + current_level`로 전환한다.
- `current_map()` / `current_map_mut()` helper를 제공한다.
- 기존 map/movement/doors/vision/items systems는 helper를 통해 current map만 다룬다.

### Step 4: Stairs commands/events

- `CommandIntent::Descend`, `Ascend` 추가.
- `GameEvent::LevelChanged` 추가.
- `src/systems/stairs.rs`에서 validation/apply 수행.

### Step 5: Snapshot/Observation update

- `GameSnapshot`에 current level과 level registry state를 포함한다.
- `Observation.current_level`과 stairs legal action을 추가한다.

### Step 6: Tests and regression

- `tests/levels.rs`, `tests/stairs.rs` 추가.
- 기존 Phase 2/3/4 tests를 current level accessors에 맞게 조정한다.

### Step 7: 문서 동기화

- 구현 완료 후 `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, `DESIGN_DECISIONS.md`, `CHANGELOG.md`, `README.md`, `designs.md`, `BUILD_GUIDE.md`를 갱신한다.
- Phase 5 기준 `seed=42 turns=100` hash를 새로 기록한다.

## 9. 수용 기준

| ID | 기준 | 검증 |
| --- | --- | --- |
| AC-1 | `LevelRegistry`가 main:1/main:2 exactly 2 levels 보유 | `cargo test --test levels level_registry_contains_two_fixed_levels` |
| AC-2 | 1층 stairs down `(34,15)`, 2층 stairs up `(5,5)`, 기존 jackal/goblin은 main:1 유지 | `cargo test --test levels fixed_level_stairs_match_spec` |
| AC-3 | `Descend`는 stairs down에서만 accepted + turn advance | `cargo test --test stairs descend_requires_stairs_down` |
| AC-4 | `Ascend`는 stairs up에서만 accepted + turn advance | `cargo test --test stairs ascend_requires_stairs_up` |
| AC-5 | Descend landing은 2층 stairs up `(5,5)` | `cargo test --test stairs descend_lands_on_level2_stairs_up` |
| AC-6 | Ascend landing은 1층 stairs down `(34,15)` | `cargo test --test stairs ascend_returns_to_level1_stairs_down` |
| AC-7 | event order는 `TurnStarted -> LevelChanged` | `cargo test --test stairs level_change_event_order_is_stable` |
| AC-8 | 1층 door/item/player state는 2층 왕복 후 보존 | `cargo test --test levels level1_state_survives_round_trip` |
| AC-9 | current level이 snapshot hash에 반영 | `cargo test --test levels current_level_affects_snapshot_hash` |
| AC-10 | deterministic level order와 level map state가 snapshot hash에 반영 | `cargo test --test levels level_map_state_affects_snapshot_hash` |
| AC-11 | Observation.current_level과 stairs legal actions 노출 | `cargo test --test stairs observation_includes_stairs_actions` |
| AC-12 | Phase 2/3/4 regression 통과 | `cargo test --test movement --test doors --test vision --test combat --test death --test items --test inventory` |
| AC-13 | legacy direct import 없음 | `rg "legacy_nethack_port_reference" src Cargo.toml tests` 결과 없음 |
| AC-14 | scope boundary 준수 | `rg "monster AI|pathfind|procedural|save/load|ratatui|crossterm|teleport" src Cargo.toml Cargo.lock tests` 수동 검토 |
| AC-15 | 품질 게이트 통과 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |

## 10. 위험과 완화

| 위험 | 영향 | 완화 |
| --- | --- | --- |
| `GameWorld.map` 단일 필드 제거로 기존 systems가 광범위하게 깨짐 | 회귀 위험 | `current_map()`/`current_map_mut()` adapter를 먼저 만들고 systems를 순차 전환 |
| `EntityLocation` level 확장으로 actor/item tests 불안정 | Phase 2-4 회귀 위험 | actor/item 위치 계약을 한 번에 통일하고 Phase 2-4 tests를 level-aware expected로 갱신해 same behavior 보존 |
| 2층 fixture가 과도해짐 | scope creep | 2층은 stairs up + 벽/문 최소 데이터만 허용 |
| Ascend/Descend와 Move가 혼동됨 | gameplay ambiguity | Move는 stairs tile에 올라설 뿐이고 transition은 explicit command로만 수행 |
| save/load 요구로 번짐 | Phase 9 침범 | snapshot/serde-ready state까지만 검증하고 file I/O 금지 |

## 11. RALPLAN ADR

결정:

Phase 5는 fixed `LevelRegistry`에 `main:1`, `main:2` 두 level을 보관하고, `GameWorld.current_level`과 actor/item 공용 level-aware `EntityLocation::OnMap { level, pos }`로 현재 level을 추적한다. `Descend`/`Ascend`는 stairs tile에서만 accepted되며, `LevelChanged` event를 생성한다.

결정 동인:

- 1층 상태 보존은 level registry 없이는 안정적으로 검증하기 어렵다.
- EntityId를 run-wide로 유지해야 Phase 4 item/inventory/event 계약이 깨지지 않는다.
- Fixed fixture는 deterministic replay와 작은 Phase 5 범위에 적합하다.

대안:

- 단일 map 백업: 작지만 후속 확장성이 낮고 item/entity location이 모호하다.
- level별 entity store: id 중복/이벤트 복잡도가 크다.
- procedural generation: Phase 5 범위 초과다.

결과:

- Phase 5 구현은 map access 경계를 `world.current_map()`로 바꾸는 리팩터를 포함한다.
- Phase 4 hash는 level state 추가로 변경될 수 있으며, 새 Phase 5 기준 hash를 문서화해야 한다.
- Phase 6 monster AI가 level-aware query를 사용해야 하므로, Phase 5 helper naming과 player location atomic helper를 안정적으로 유지해야 한다.

## 12. 실행 핸드오프

### Ralph 권장 지시

```text
$ralph .omx/plans/prd-phase5-levels-stairs.md

반드시 .omx/plans/test-spec-phase5-levels-stairs.md의 검증 기준을 만족시키고,
Phase 5 완료 후 멈추세요. Phase 6로 진행하지 마세요.
```

### Available-Agent-Types Roster

| 역할 | 책임 | 권장 reasoning |
| --- | --- | --- |
| `executor` | level registry, stairs commands, world migration 구현 | medium |
| `test-engineer` | levels/stairs/regression 테스트 작성 | medium |
| `architect` | world/entity location boundary 검토 | high |
| `critic` | scope creep와 acceptance criteria 검토 | high |
| `verifier` | 최종 증거와 테스트 충분성 확인 | high |

### Launch Hints

Ralph 단일 소유 실행:

```bash
# Codex/OMX 세션에서 실행
$ralph .omx/plans/prd-phase5-levels-stairs.md
```

Team 병렬 실행이 필요할 때:

```bash
# 문서/테스트 기준을 팀 브리프에 포함
$team .omx/plans/prd-phase5-levels-stairs.md .omx/plans/test-spec-phase5-levels-stairs.md
```

### Team 사용 시 lane

- Lane A: `domain/level.rs`, `GameWorld` level registry migration
- Lane B: `CommandIntent`/`GameEvent`/`systems/stairs.rs`/session routing
- Lane C: snapshot/observation/systems current map migration
- Lane D: `tests/levels.rs`, `tests/stairs.rs`, Phase 2-4 regression
- Lane E: docs sync and verification evidence

Team verification path:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --test levels
cargo test --test stairs
cargo run --bin aihack-headless -- --seed 42 --turns 100
rg "legacy_nethack_port_reference" src Cargo.toml tests
rg "monster AI|pathfind|procedural|save/load|ratatui|crossterm|teleport" src Cargo.toml Cargo.lock tests
```

## 13. Consensus 상태

- Planner: 승인. Fixed 2-level registry가 Phase 5 목표를 가장 작게 만족한다.
- Architect: 승인. 최초 ITERATE 후 1층 몬스터 보존, atomic player location helper, deterministic snapshot order, T3 실행성, scope audit 범위 보강을 반영하고 재검토 APPROVE를 받았다.
- Critic: 승인. AC-1~AC-15/T1~T17, RALPLAN-DR, Ralph handoff safety가 충분하다는 APPROVE를 받았다.


## 14. RALPLAN 개선 반영 로그

- Architect ITERATE 의견을 반영하여 1층 jackal/goblin 보존을 명시했다.
- `player_location()`/`set_player_location(level, pos)` 계열 atomic helper 요구를 추가했다.
- `LevelRegistry`와 snapshot의 deterministic level order 요구를 추가했다.
- T3 실행 가능성 강화를 위해 compile-facing/source-audit 기준을 test spec에 연결했다.
- scope audit 범위를 `src`에서 `src Cargo.toml Cargo.lock tests`로 확장했다.
