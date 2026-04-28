# Context Snapshot: Phase 4 Items and Inventory

작성일(UTC): 2026-04-28T13:01:45Z
작업: `$ralplan phase 4 계획 문서 작성`
상태: 계획 전용. 구현 금지.

## task statement

Phase 3 Combat/Death 완료 상태 위에서 Phase 4 Items and Inventory 구현을 위한 PRD와 Test Spec을 작성한다.

## desired outcome

- `.omx/plans/prd-phase4-items-inventory.md` 생성
- `.omx/plans/test-spec-phase4-items-inventory.md` 생성
- Phase 4 범위는 item store, inventory letter, pickup, wield dagger, quaff healing, item consumed event로 제한
- Phase 5+ level/stairs, Phase 6 monster AI, TUI/effect/save/load 구현은 계획 범위 밖으로 명시

## known facts/evidence

- Phase 3 최종 hash: `seed=42 turns=100 final_hash=8b20a23301eea977`
- 현재 core 명령: `Wait`, `Quit`, `Move`, `Open`, `Close`
- 현재 event: `TurnStarted`, `Waited`, `EntityMoved`, `DoorChanged`, `AttackResolved`, `EntityDied`, `CommandRejected`
- 현재 `GameWorld`: `map + EntityStore + player_id`
- 현재 `EntityKind`: `Player`, `Monster(MonsterKind)` only
- `spec.md` item v0.1 실데이터:
  - dagger: weapon, glyph `)`, slot melee, hit_bonus 1, damage 1d4
  - food ration: food, glyph `%`, nutrition 800
  - potion of healing: potion, glyph `!`, effect heal_1d8_plus_4
- `audit_roadmap.md` Phase 4 목표: item store, inventory letter map, pickup, wield dagger, quaff healing, item consumed event

## constraints

- 모든 문서는 README를 제외하고 한국어로 작성
- `AI_IMPLEMENTATION_DOC_STANDARD.md` 기준: typed contract, concrete number, real data sample, verification path, scope closure 포함
- 새 의존성 추가 금지
- 모든 난수는 `GameRng` 경유
- UI는 상태를 직접 변경하지 않고 `CommandIntent`만 제출한다는 기존 경계 유지
- Phase 4 구현 계획은 save/load를 구현하지 않되, inventory letter 안정성은 snapshot/replay와 테스트로 검증

## unknowns/open questions

- full save/load가 Phase 9이므로 Phase 4 완료 기준의 “save/load 후 inventory letter 유지”는 즉시 구현 대신 stable serialization/snapshot contract와 synthetic roundtrip 테스트로 대체해야 하는가? 권장: Phase 4에서는 `serde_json` roundtrip of inventory state까지만 검증하고 실제 save/load 파일 시스템은 Phase 9로 유예한다.
- 장비 슬롯을 player entity 내부에 둘지 별도 inventory/equipment 도메인에 둘지 결정 필요. 권장: `Inventory`가 `equipped_melee: Option<EntityId>`를 보유하고 combat가 이를 읽는다.
- item을 기존 `EntityStore`에 통합할지 별도 `ItemStore`를 둘지 결정 필요. 권장: Phase 4에서는 `EntityKind::Item(ItemKind)`로 같은 store에 통합하되 actor stats는 Option 분리 또는 enum payload로 리팩터링한다.

## likely codebase touchpoints

- `src/core/action.rs`
- `src/core/event.rs`
- `src/core/session.rs`
- `src/core/snapshot.rs`
- `src/core/observation.rs`
- `src/core/world.rs`
- `src/domain/entity.rs`
- 신규 `src/domain/item.rs`
- 신규 `src/domain/inventory.rs`
- `src/systems/combat.rs`
- 신규 `src/systems/items.rs`
- 신규 `tests/items.rs`
- 신규 `tests/inventory.rs`
- 기존 regression: `tests/combat.rs`, `tests/death.rs`, `tests/movement.rs`, `tests/vision.rs`
