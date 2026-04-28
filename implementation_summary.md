# AIHack Implementation Summary

문서 상태: active
작성일: 2026-04-28

## 1. 전체 런타임 흐름

새 AIHack은 다음 흐름을 고정한다.

```text
Input/UI/AI
  -> CommandIntent
  -> GameSession::submit()
  -> validate_command()
  -> apply_turn()
  -> systems run in fixed order
  -> TurnOutcome
  -> GameSnapshot + Observation
  -> UI render / replay write / AI read
```

모든 상태 변경은 `GameSession::submit()` 안에서만 일어난다.

## 2. 시스템 순서

한 턴 처리 순서는 고정이다.

1. `TurnStarted`
2. player command validation
3. player action apply
4. tile interaction
5. monster intent collection
6. monster action apply
7. delayed effects
8. status tick
9. death check
10. snapshot hash calculation
11. observation build

이 순서는 `systems/turn_pipeline.rs`에 상수로 둔다.

## 3. 파일 책임

| 파일 | 책임 |
| --- | --- |
| `src/core/session.rs` | `GameSession`, submit, snapshot |
| `src/core/turn.rs` | 턴 파이프라인과 `TurnOutcome` |
| `src/core/action.rs` | `CommandIntent`, validation result |
| `src/core/observation.rs` | AI 관찰 스키마 |
| `src/core/event.rs` | `GameEvent`, formatter |
| `src/core/save.rs` | Phase 9 예정: `SaveDataV1`, load/write |
| `src/domain/map.rs` | `GameMap`, tile 접근 |
| `src/domain/entity.rs` | entity store, `EntityId` |
| `src/domain/player.rs` | player component data |
| `src/domain/monster.rs` | monster data and AI state |
| `src/domain/item.rs` | item data and inventory letter |
| `src/systems/movement.rs` | 이동 검증/적용 |
| `src/systems/doors.rs` | 문 열기/닫기/차단 |
| `src/systems/vision.rs` | LOS와 visible tile |
| `src/systems/combat.rs` | 명중/피해/사망 후보 |
| `src/systems/monster_ai.rs` | Phase 6 예정: 몬스터 의도 수집 |
| `src/systems/items.rs` | Phase 4 완료: pickup/wield/quaff, wear는 후속 범위 |
| `src/systems/stairs.rs` | Phase 5 완료: explicit descend/ascend level transition |
| `src/bin/aihack-headless.rs` | deterministic simulation runner |

## 4. 레거시 자산 사용 규칙

레거시 폴더:

```text
legacy_nethack_port_reference/
```

사용 가능:

- 규칙을 읽고 새 코드로 재작성
- 테스트 케이스의 입력/출력만 참고
- TOML 데이터를 새 스키마로 변환
- 문서의 리스크/결정 기록 참고

사용 금지:

- `#[path = "../legacy..."]` 방식 import
- workspace member로 레거시 Cargo 추가
- 레거시 `GameState`, `ActionQueue`, `EventQueue` 타입 재사용
- 레거시 UI 파일을 새 UI에 직접 복사

## 5. 첫 플레이어블 최소 범위

아래 v0.1 scope는 장기 목표를 포함한 목표 범위다. 2026-04-29 현재 구현 완료 기준은 Phase 5이며, Phase 6+ 항목(몬스터 AI 고도화/저장/TUI)은 아직 구현 완료가 아니다.

v0.1 scope:

- 40x20 fixture map
- 플레이어 1명
- 몬스터 3종: jackal, goblin, floating eye
- 아이템 3종: dagger, food ration, potion of healing
- 명령 목표: move, wait, open, close, pickup, show inventory, wield, quaff, descend, ascend는 Phase 5까지 구현. search/wear는 후속 Phase 범위
- 이벤트 목표: turn, reject, move, door, pickup, equip, consume, heal, attack, death, level changed는 Phase 5까지 구현. message formatter는 후속 UI/API 범위
- 저장 1종: JSON save v1은 Phase 9 범위
- headless runner 1개: 구현 완료
- TUI play 화면 1개: Phase 10 범위

v0.1에서 제외:

- 상점
- 기도
- 복잡한 polymorph
- NetHack 특수 레벨
- LLM decision support
- 마법봉/투척/스크롤

## 5.1 Phase 1 Headless Core 완료 요약

2026-04-28에 Phase 1은 `.omx/plans/prd-phase1-headless-core.md`와 `.omx/plans/test-spec-phase1-headless-core.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `Cargo.toml` | root package와 `aihack`, `aihack-headless` bin 정의 완료 |
| `src/main.rs` | UI adapter 미구현 안내 출력 완료 |
| `src/lib.rs` | library export 진입점 추가 완료 |
| `src/bin/aihack-headless.rs` | seed/turns wait-only simulation runner 완료 |
| `src/core/rng.rs` | `GameRng` wrapper와 deterministic unit test 완료 |
| `src/core/session.rs` | `GameSession::new`, `submit(Wait)`, snapshot 생성 완료 |
| `src/core/snapshot.rs` | FNV-1a stable hash 완료 |
| `src/core/turn.rs` | `SnapshotHash`, `TurnOutcome` 완료 |
| `src/core/action.rs` | `Wait`, `Quit` 최소 명령 완료 |
| `src/core/event.rs` | Phase 1 최소 이벤트 완료 |
| `src/core/ids.rs` | 최소 ID 타입 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: 7 passed
cargo run --bin aihack-headless -- --seed 42 --turns 0: 66595593fabacdf4
cargo run --bin aihack-headless -- --seed 42 --turns 100: f827bc2d4155ef66
cargo run --bin aihack-headless -- --seed 43 --turns 100: 3ed5b4db4d5e7157
```

Phase 1에서 의도적으로 제외된 항목:

- map/movement/doors/vision
- combat/death
- item/inventory
- monster AI
- save/load persistence
- TUI/modern UX
- AI `Observation`/`ActionSpace` 전체 구현

## 5.2 Phase 2 Map, Movement, Doors, Vision 완료 요약

2026-04-28에 Phase 2는 `.omx/plans/prd-phase2-map-movement-doors-vision.md`와 `.omx/plans/test-spec-phase2-map-movement-doors-vision.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/core/position.rs` | `Pos`, `Delta`, `Direction`와 8방향 delta 완료 |
| `src/domain/tile.rs` | `TileKind`, `DoorState`, movement/LOS blocker helper 완료 |
| `src/domain/map.rs` | 40x20 Phase 2 fixture와 bounds-safe tile access 완료 |
| `src/core/world.rs` | Phase 2 `GameWorld { map, player_pos }` 완료, Phase 3 이후 `EntityStore` 기반 player 위치로 확장 |
| `src/systems/movement.rs` | movement validation/apply와 diagonal corner-cutting 금지 완료 |
| `src/systems/doors.rs` | adjacent open/close door 완료 |
| `src/systems/vision.rs` | radius 8 LOS와 blocker 규칙 완료 |
| `src/core/observation.rs` | 최소 `Observation.visible_tiles`와 `legal_actions` 완료 |
| `src/core/session.rs` | `Move`, `Open`, `Close` submit routing 완료 |
| `src/core/snapshot.rs` | map/player/door state hash 입력 확장 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test map: pass
cargo test movement: pass
cargo test doors: pass
cargo test vision: pass
cargo test observation: pass
cargo run --bin aihack-headless -- --seed 42 --turns 100: 1aad6f4049778b0e
```

Phase 2에서 의도적으로 제외된 항목:

- full entity store
- combat/death
- item/inventory
- monster AI
- stairs level transition
- save/load persistence
- TUI/modern UX


## 5.3 Phase 3 Combat and Death 완료 요약

2026-04-28에 Phase 3는 `.omx/plans/prd-phase3-combat-death.md`와 `.omx/plans/test-spec-phase3-combat-death.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/domain/entity.rs` | `EntityStore`, `Entity`, `EntityKind`, `Faction`, `ActorStats`, stable `EntityId`, tombstone policy 완료 |
| `src/domain/player.rs` | player adventurer 기본 stat과 내장 dagger profile 완료 |
| `src/domain/monster.rs` | jackal/goblin/floating eye 기본 stat factory 완료 |
| `src/domain/combat.rs` | `DamageRoll`, `AttackProfile`, `DeathCause` 계약 완료 |
| `src/systems/combat.rs` | d20 명중 판정, damage roll, `AttackResolved` 생성 완료 |
| `src/systems/death.rs` | `EntityDied`, tombstone, player `GameOver` 판정 완료 |
| `src/core/world.rs` | map + entity store + player id 기반 world 완료 |
| `src/core/session.rs` | `Move`의 빈 타일 이동/bump attack 분기와 shared legality gate 완료 |
| `src/core/snapshot.rs` | entity id/kind/pos/hp/alive snapshot hash 입력 완료 |
| `tests/combat.rs` | entity, factory, formula, legal bump action, jackal/goblin bump attack, event order 검증 완료 |
| `tests/death.rs` | monster death, dead monster movement, player death, hash 변경 검증 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo run --bin aihack-headless -- --seed 42 --turns 100: 8b20a23301eea977
```

Phase 3에서 의도적으로 제외된 항목:

- item/inventory/equipment entity
- monster AI와 반격 루프
- ranged/throw/zap/spell
- XP/score/corpse/drop
- save/load persistence
- TUI/modern UX/effect


## 5.4 Phase 4 Items and Inventory 완료 요약

2026-04-28에 Phase 4는 `.omx/plans/prd-phase4-items-inventory.md`와 `.omx/plans/test-spec-phase4-items-inventory.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/domain/item.rs` | `ItemKind`, `ItemClass`, `EquipmentSlot`, `ConsumableEffect`, `ItemData` 실데이터 완료 |
| `src/domain/inventory.rs` | `Inventory`, `InventoryEntry`, `InventoryLetter`, stable letter, `equipped_melee` 완료 |
| `src/domain/entity.rs` | `EntityPayload::Actor | Item`, `EntityLocation`, item query helper 완료 |
| `src/systems/items.rs` | pickup, wield, quaff, effective healing event 완료 |
| `src/systems/combat.rs` | player equipped melee profile / unarmed fallback 완료 |
| `src/core/action.rs` | `Pickup`, `ShowInventory`, `Wield`, `Quaff` 완료 |
| `src/core/event.rs` | `ItemPickedUp`, `ItemEquipped`, `ItemConsumed`, `EntityHealed` 완료 |
| `src/core/observation.rs` | `Observation.inventory`와 item legal actions 완료 |
| `src/core/snapshot.rs` | item/inventory/equipment snapshot hash 입력 완료 |
| `tests/items.rs` | item fixture, pickup, quaff, consumed policy 검증 완료 |
| `tests/inventory.rs` | letter, wield, combat profile, serde roundtrip 검증 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test --test items: pass
cargo test --test inventory: pass
cargo run --bin aihack-headless -- --seed 42 --turns 100: 00ba578d933177f2
```

Phase 4에서 의도적으로 제외된 항목:

- file save/load
- drop/read/zap/throw
- food eating/nutrition/hunger
- item identification/stack/quantity/BUC/enchantment
- TUI inventory screen/mouse/drag-drop
- monster item use
- multi-level item persistence

## 6. 구현 순서

### Task 1: 새 Cargo 스캐폴딩

Acceptance:

- `Cargo.toml` 생성
- `src/main.rs`와 `src/bin/aihack-headless.rs` 생성
- `cargo test` 성공
- 레거시 코드는 workspace member가 아님

Verification:

```bash
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 0
```

### Task 2: Core IDs, RNG, Error

Acceptance:

- `EntityId`, `LevelId`, `Pos`, `Direction`
- deterministic `GameRng`
- `GameError`
- seed 42 RNG snapshot test

### Task 3: GameMap fixture

Acceptance:

- 40x20 map fixture load
- wall/floor/door/stairs tile
- bounds check returns error, not panic

### Task 4: Entity store

Acceptance:

- player, monster, item spawn
- stable `EntityId`
- remove leaves tombstone or generation check

### Task 5: GameSession submit loop

Acceptance:

- `GameSession::new(seed)`
- `submit(CommandIntent::Wait)` advances turn
- `TurnOutcome.snapshot_hash` stable

### Task 6: Movement and doors

Acceptance:

- movement blocked by wall/closed door
- open/close updates `DoorState`
- rejected command does not advance turn

### Task 7: Vision and observation

Acceptance:

- visible tiles generated from player
- closed door blocks LOS
- `Observation.legal_actions` generated

### Task 8: Combat and death

Acceptance:

- bump attack on hostile monster
- deterministic hit/damage tests
- death event and removal policy

### Task 9: Inventory and item actions

Acceptance:

- pickup assigns inventory letter
- wield dagger changes melee weapon
- quaff healing potion heals and consumes item

### Task 10: Monster AI

Acceptance:

- jackal wanders until LOS
- goblin chases on sight
- floating eye stationary passive effect exists as event placeholder

### Task 11: Stairs and second level

Status: 완료.

Acceptance:

- `LevelRegistry`가 fixed `main:1`, `main:2`를 deterministic order로 보유한다.
- `Descend`는 1층 stairs down `(34,15)`에서만 accepted되며 2층 stairs up `(5,5)`로 이동한다.
- `Ascend`는 2층 stairs up `(5,5)`에서만 accepted되며 1층 stairs down `(34,15)`로 복귀한다.
- 1층 door/item/inventory/monster state는 2층 왕복 후 보존된다.
- `GameSnapshot`은 current level, level map state, level-aware entity location을 hash 입력에 포함한다.

### Task 12: Save/load/replay

Acceptance:

- save after 10 turns
- load and continue same hash path
- replay JSONL writes all command/outcome pairs

### Task 13: TUI adapter

Acceptance:

- reads `GameSnapshot`
- sends `CommandIntent`
- no direct mutation of core state

### Task 14: AI adapter

Acceptance:

- emits `Observation` JSON
- accepts `ActionIntent`
- invalid actions are rejected with event

## 7. 유지보수 규칙

- 각 task는 `cargo test` 통과 상태로 끝낸다.
- 새 시스템은 panic보다 `GameError`를 반환한다.
- `unwrap()`은 tests 또는 static invariant에만 허용한다.
- 새 문서에 나온 ID는 data fixture에 존재해야 한다.
- AI 관련 필드는 schema version을 올리지 않고 의미를 바꾸지 않는다.

## 8. 레거시 기능 흡수 우선순위

| 순위 | 기능 | 레거시 참고 |
| --- | --- | --- |
| 1 | RNG/dice | `legacy.../src/util/rng.rs` |
| 2 | item/monster kind 목록 | `legacy.../src/generated/kinds.rs` |
| 3 | combat formulas | `legacy.../src/core/systems/combat/` |
| 4 | dungeon room gen | `legacy.../src/core/dungeon/gen.rs` |
| 5 | inventory letters | `legacy.../src/core/entity/` and item systems |
| 6 | trap rules | `legacy.../src/core/systems/world/trap*.rs` |
| 7 | wand/beam | `legacy.../src/core/systems/item/zap*.rs` |
| 8 | shop/pray/social | `legacy.../src/core/systems/social/` |

흡수는 복사가 아니라 새 계약에 맞춘 재작성이다.

## 9. 현대 TUI 리팩토링 구현 순서

이 섹션은 `spec.md`의 현대 TUI/UX 계획을 구현 요약으로 분해한다. core 안정화 전에는 타입/경계만 고려하고, 실제 TUI 구현은 Task 13 이후에 진행한다.

### 9.1 파일 책임 추가

| 파일 | 책임 |
| --- | --- |
| `src/ui/tui/mod.rs` | TUI adapter 진입점, terminal lifecycle |
| `src/ui/tui/layout.rs` | 80x28/100x32/120x36 layout 계산과 snapshot test |
| `src/ui/tui/input.rs` | keyboard/mouse `UiInputEvent` 수집과 `UiCommandCandidate` 변환 |
| `src/ui/tui/viewport.rs` | screen 좌표와 dungeon `Pos` 변환 |
| `src/ui/tui/render_map.rs` | map/entity glyph rendering |
| `src/ui/tui/render_panels.rs` | status/inspect/log/command bar rendering |
| `src/ui/tui/effects.rs` | `GameEvent -> UiEffectEvent` projection과 frame lifecycle |
| `src/ui/tui/theme.rs` | color profile, reduced motion, high contrast token |
| `src/ui/tui/config.rs` | `UiRuntimeConfig` 기본값과 로드 정책 |
| `tests/ui_layout.rs` | layout regression |
| `tests/ui_input_mapping.rs` | mouse/key 좌표와 command candidate 검증 |
| `tests/ui_effect_projection.rs` | core event에서 UI effect 생성 검증 |

### 9.2 Task 13 세분화

#### Task 13A: TUI shell와 layout

Acceptance:

- terminal raw mode/alternate screen 진입과 복구 경로 정의
- 80x28 layout snapshot 통과
- `GameSnapshot` read-only render
- `CommandIntent` submit 경로만 core로 연결

#### Task 13B: 정보 가독성

Acceptance:

- hover/inspect data model 구현
- priority message style 구현
- command bar가 legal action 또는 고정 command hint를 표시
- low HP/danger 상태가 색상 외 텍스트 채널로도 표시

#### Task 13C: 마우스 입력

Acceptance:

- mouse capture 옵션화
- map click -> `InspectMap` 또는 adjacent `Move` 후보 변환
- inventory row click -> item selection 후보 변환
- hover/click은 비진행 이벤트와 진행 명령을 명확히 분리

#### Task 13D: ASCII effect projection

Acceptance:

- `GameEvent`에서 `UiEffectEvent` 생성
- effect duration 기본값 준수
- `reduced_motion`에서 effect가 단일 frame 또는 정적 표시로 축소
- 같은 seed의 headless replay hash가 effect on/off와 무관하게 동일

### 9.3 검증 순서

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test ui_layout
cargo test ui_input_mapping
cargo test ui_effect_projection
cargo test replay_hash_ignores_ui_effects
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 1000
```

수동 검증은 `designs.md` 15.7 기준을 따른다.


## Phase 5 구현 요약

- `src/domain/level.rs`를 추가하여 `GameLevel`, `LevelRegistry`, `main:1/main:2` fixed fixture를 정의했다.
- `GameWorld`를 단일 map에서 `levels + current_level + EntityStore + Inventory` 구조로 전환했다.
- actor/item 위치 계약을 `EntityLocation::OnMap { level, pos }`로 확장하고 player location helper로 current level invariant를 유지한다.
- `CommandIntent::Descend`, `CommandIntent::Ascend`, `GameEvent::LevelChanged`를 추가했다.
- `tests/levels.rs`, `tests/stairs.rs`를 추가했고 전체 `cargo test`를 통과했다.
- Phase 5 기준 hash: `seed=42 turns=100 final_hash=88886c28698a1730`.
