# Context Snapshot: Phase 5 Levels and Stairs

작성일(UTC): 2026-04-28T17:19:51Z
작업: `$ralplan phase 5 계획을 문서로 작성합니다.`
상태: 계획 전용. 구현 금지.

## task statement

Phase 4 Items and Inventory 완료 상태 위에서 Phase 5 Levels and Stairs 구현을 위한 PRD와 Test Spec을 작성한다.

## desired outcome

- `.omx/plans/prd-phase5-levels-stairs.md` 생성
- `.omx/plans/test-spec-phase5-levels-stairs.md` 생성
- Phase 5 범위는 `LevelId`, level registry, stairs down/up, current level snapshot, 1층-2층 왕복과 level별 상태 보존으로 제한
- Phase 6 monster AI, Phase 9 file save/load, Phase 10 TUI, procedural dungeon generation은 제외

## known facts/evidence

- Phase 4 최종 hash: `seed=42 turns=100 final_hash=00ba578d933177f2`
- 현재 `LevelId { branch: BranchId, depth: i16 }`와 `BranchId::Main` 타입은 존재한다.
- 현재 `GameWorld`는 단일 `map + EntityStore + player_id + Inventory`만 보유한다.
- 현재 `GameMap::fixture_phase2()`는 40x20, stairs down `(34,15)`만 가진다.
- 현재 `TileKind`는 `StairsDown`, `StairsUp`을 이미 포함하고 movement/LOS passable로 취급한다.
- 현재 `CommandIntent`에는 `Descend`/`Ascend`가 없다.
- 현재 `GameEvent`에는 `LevelChanged`가 없다.
- Phase 4 item location은 `EntityLocation::OnMap(Pos) | Inventory { owner } | Consumed`이며 level 정보를 포함하지 않는다.

## constraints

- 모든 문서는 README를 제외하고 한국어로 작성
- `AI_IMPLEMENTATION_DOC_STANDARD.md` 기준: typed contract, concrete number, real data sample, verification path, scope closure 포함
- 새 의존성 추가 금지
- 모든 난수는 `GameRng` 경유. Phase 5는 가능하면 새 난수 사용 없이 fixed fixture level 2를 생성
- `GameSession::submit()` 단일 상태 변경 경계 유지
- Phase 5는 file save/load를 구현하지 않는다. level registry serialization-ready/snapshot 검증만 수행
- Phase 5는 monster AI/pathfinding/procedural map generation을 구현하지 않는다

## unknowns/open questions

- multi-level entity 위치를 `EntityLocation::OnMap { level, pos }`로 확장할지, level별 `GameLevel.entities` 분리로 둘지 결정 필요. 권장: Phase 5에서는 `GameWorld.levels: Vec<GameLevel>`와 `current_level`을 추가하고, `EntityLocation::OnMap { level, pos }`로 확장한다.
- Entity id는 level 간 공유 단일 store를 유지할지 level별 store로 분리할지 결정 필요. 권장: Phase 4의 stable `EntityId` 계약을 유지하기 위해 단일 `EntityStore`를 유지한다.
- 2층 fixture 데이터는 얼마나 포함할지 결정 필요. 권장: 40x20 fixed level 2, stairs up `(5,5)`, stairs down 없음, 소량의 벽/문/아이템만 포함하고 몬스터 AI 없음.

## likely codebase touchpoints

- `src/core/action.rs`
- `src/core/event.rs`
- `src/core/world.rs`
- `src/core/session.rs`
- `src/core/snapshot.rs`
- `src/core/observation.rs`
- `src/domain/map.rs`
- `src/domain/entity.rs`
- 신규 `src/domain/level.rs`
- 신규 `src/systems/stairs.rs`
- `src/systems/movement.rs`
- `src/systems/vision.rs`
- `tests/levels.rs`
- `tests/stairs.rs`
- 기존 regression: `tests/items.rs`, `tests/inventory.rs`, `tests/combat.rs`, `tests/death.rs`, `tests/movement.rs`, `tests/vision.rs`
