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
| `src/core/session.rs` | Phase 16 완료: `GameSession`, RunState 확장, submit 상태별 분기, snapshot |
| `src/core/turn.rs` | 턴 파이프라인과 `TurnOutcome` |
| `src/core/action.rs` | Phase 16 완료: `CommandIntent`, `DirectionalAction`, `InventoryAction`, `AcknowledgeMore`, `Pray` |
| `src/core/observation.rs` | Phase 16 완료: AI 관찰 스키마, RunStateSummary 확장, 상태별 legal_actions |
| `src/core/event.rs` | Phase 16 완료: `GameEvent`, `MessagePriority`, formatter |
| `src/core/save.rs` | Phase 9 완료: `SaveDataV1`, load/write, replay JSONL |
| `src/core/world.rs` | Phase 16/20: `GameWorld`, `last_death_cause`, `status()`/`set_status()` |
| `src/domain/map.rs` | `GameMap`, tile 접근 |
| `src/domain/entity.rs` | entity store, `EntityId` |
| `src/domain/player.rs` | player component data |
| `src/domain/monster.rs` | monster data and AI state |
| `src/domain/item.rs` | item data and inventory letter |
| `src/domain/status.rs` | Phase 20 완료: `Status`, `HungerState` |
| `src/systems/movement.rs` | 이동 검증/적용 |
| `src/systems/doors.rs` | 문 열기/닫기/차단 |
| `src/systems/vision.rs` | LOS와 visible tile |
| `src/systems/combat.rs` | 명중/피해/사망 후보 |
| `src/systems/monster_ai.rs` | Phase 6 완료: 몬스터 의도 수집/적용, current-level deterministic AI |
| `src/systems/items.rs` | Phase 4/8 완료: pickup/wield/quaff/wear/drop/read |
| `src/systems/stairs.rs` | Phase 5 완료: explicit descend/ascend level transition |
| `src/systems/death.rs` | Phase 16: `state_after_deaths()`가 `GameOver { cause, final_score }` 반환 |
| `src/bin/aihack-headless.rs` | deterministic simulation runner |
| `src/ui/tui/mod.rs` | Phase 17/18: TUI runtime, RunState별 화면 분기, F9 debug toggle, 라벨 수집 |
| `src/ui/tui/input.rs` | Phase 17: 키/마우스 입력, `UiCommandCandidate::NewRun` |
| `src/ui/tui/render_panels.rs` | Phase 17/18: 화면별 lines 함수(Title/CharacterCreation/GameOver/DebugObservation) |
| `src/ui/tui/render_map.rs` | Phase 19: `MapWidget`에 라벨 오버레이 렌더링 |
| `src/ui/tui/labels.rs` | Phase 19: `AutoLabel`, `LabelKind`, `collect_auto_labels()`, `filter_expired_labels()` |
| `src/ui/tui/effects.rs` | Phase 19: `UiEffectKind::NewEntityLabel` |
| `src/data/mod.rs` | Phase 20: TOML 로더(`load_items`, `load_monsters`, `load_level`) |
| `src/data/items.toml` | Phase 20: 아이템 외부 데이터 |
| `src/data/monsters.toml` | Phase 20: 몬스터 외부 데이터 |
| `src/data/levels/` | Phase 20: 레벨 외부 데이터 |

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

아래 v0.1 scope는 장기 목표를 포함한 목표 범위다. 2026-05-18 현재 구현 완료 기준은 Phase 15이며, 이후 planned execution phase는 없다.

v0.1 scope:

- 40x20 fixture map
- 플레이어 1명
- 몬스터 3종: jackal, goblin, floating eye
- 아이템 핵심군: dagger, food ration, potion of healing, wand/scroll/rock/armor/corpse extension
- 명령 목표: move, wait, open, close, pickup, show inventory, wield, wear, quaff, drop, search, throw, zap, read, pray, descend, ascend 구현 완료
- 이벤트 목표: turn, reject, move, door, trap, projectile, pickup/drop, equip, consume, heal, attack, death, level changed, narrative/decision presentation hooks까지 구현 완료
- 저장 1종: JSON save v1 구현 완료
- headless runner 1개: 구현 완료
- TUI play 화면 1개: 구현 완료

v0.1에서 제외:

- 상점 full economy
- 복잡한 polymorph
- NetHack 특수 레벨
- transport/network AI API
- autonomous self-play

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


## 5.5 Phase 5 Levels and Stairs 완료 요약

2026-05-16 기준 Phase 5는 fixed `main:1`/`main:2` level registry, explicit `Descend`/`Ascend`, level-aware `EntityLocation`을 유지한다. Phase 5 기준 hash는 `seed=42 turns=100 final_turn=100 final_hash=88886c28698a1730`이다.

## 5.6 Phase 6 Monster AI 완료 요약

2026-05-16에 Phase 6은 `.omx/plans/prd-phase6-monster-ai.md`와 `.omx/plans/test-spec-phase6-monster-ai.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/domain/monster.rs` | `MonsterAiKind`, jackal/goblin/floating eye policy 완료 |
| `src/systems/monster_ai.rs` | current-level intent collect/apply 완료 |
| `src/core/session.rs` | accepted-turn monster phase hook 완료 |
| `tests/monster_ai.rs` | turn gate, chase, wander, stationary, off-level freeze 검증 완료 |

검증 결과:

```text
cargo test --test monster_ai: pass
seed=42 turns=100 final_turn=20 final_hash=2fb549b5d2e1e67f
seed=43 turns=100 final_turn=21 final_hash=ec98b802759e109c
```

## 5.7 Phase 7 NetHack Interaction Set 1 완료 요약

2026-05-16에 Phase 7은 `.omx/plans/prd-phase7-nethack-interaction-set-1.md`와 `.omx/plans/test-spec-phase7-nethack-interaction-set-1.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/domain/tile.rs` | `HiddenDoor`, `Trap(Pit)`, `HiddenTrap(Pit)` 계약 완료 |
| `src/domain/item.rs` | `WandMagicMissile`, `ScrollReveal`, `Rock`, charge/effect 데이터 완료 |
| `src/systems/traps.rs` | search reveal, trap trigger, scroll reveal 완료 |
| `src/systems/projectiles.rs` | throw/zap directional traversal 완료 |
| `src/core/action.rs` | `Search`, `Throw`, `Zap`, `Read` 완료 |
| `src/core/event.rs` | `TileRevealed`, `TrapTriggered`, `ItemThrown`, `WandZapped`, `ScrollRead` 완료 |
| `tests/traps.rs` | search/trap/reveal/snapshot 검증 완료 |
| `tests/projectiles.rs` | throw/zap/charge/wall stop 검증 완료 |

검증 결과:

```text
cargo test: pass
cargo clippy --all-targets -- -D warnings: pass
seed=42 turns=0 final_turn=0 final_hash=dc24b554ed6401aa
seed=42 turns=100 final_turn=20 final_hash=5aecd83cf284cb25
seed=43 turns=100 final_turn=21 final_hash=5f5d5b89faa9a834
```

Phase 7에서 의도적으로 제외된 항목:

- random trap table / teleport / web / trapdoor
- identify metagame / curse / hunger / encumbrance
- multi-wand / reflection / bounce / engraving
- save/load persistence
- TUI interaction UX


## 5.8 Phase 8 Legacy Rule Absorption 완료 요약

2026-05-16에 Phase 8은 `.omx/plans/prd-phase8-legacy-rule-absorption.md`와 `.omx/plans/test-spec-phase8-legacy-rule-absorption.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/core/action.rs` | `Kick`, `Drop`, `Wear`, `Pray`와 Phase 8 command 확장 완료 |
| `src/core/world.rs` / `src/core/snapshot.rs` | nutrition/luck/prayer_cooldown/paralysis/gold/kill_count/identified_items state 완료 |
| `src/domain/item.rs` | scroll identify/level teleport, armor, corpse, base price metadata 완료 |
| `src/systems/items.rs` | drop/wear/read identify/level teleport 완료 |
| `src/systems/doors.rs` | kick door 완료 |
| `src/systems/score.rs` | death score / luck / hallucination message helper 완료 |
| `tests/golden_phase8_rules.rs` | 20개 golden scenario 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test --test golden_phase8_rules: 20 passed
seed=42 turns=0 final_turn=0 final_hash=53435bb29a2e69ee
seed=42 turns=100 final_turn=20 final_hash=4c77dafb19dd2226
seed=43 turns=100 final_turn=21 final_hash=f8324eacbce50087
```

Phase 8에서 의도적으로 제외된 항목:

- save/load persistence
- TUI affordance/message UX polish
- AI schema freeze
- full NetHack parity


## 5.9 Phase 9 Save Load Replay 완료 요약

2026-05-17에 Phase 9는 `.omx/plans/prd-phase9-save-load-replay.md`와 `.omx/plans/test-spec-phase9-save-load-replay.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/core/save.rs` | `SaveDataV1`, `SavedWorldV1`, `ReplayLineV1`, file IO 완료 |
| `src/core/rng.rs` | `RngStateV1`, seed+draw continuation restore 완료 |
| `src/core/session.rs` | `to_save_data`, `from_save_data`, load-resume integration 완료 |
| `src/bin/aihack-headless.rs` | `--save`, `--load`, `--replay-out` CLI 완료 |
| `tests/save_load.rs` | schema/hash/continuation/invalid schema 검증 완료 |
| `tests/replay.rs` | replay JSONL schema/load-resume equivalence 검증 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test --test save_load --test replay: pass
seed=42 turns=100 final_turn=20 final_hash=4c77dafb19dd2226
seed=43 turns=100 final_turn=21 final_hash=f8324eacbce50087
```

Phase 9에서 의도적으로 제외된 항목:

- binary/compressed save
- autosave slot UX
- replay visualizer UI
- TUI save/load command surface
- backward compatibility beyond v1


## 5.10 Phase 10 TUI Adapter 완료 요약

2026-05-17에 Phase 10은 `.omx/plans/prd-phase10-tui-adapter.md`와 `.omx/plans/test-spec-phase10-tui-adapter.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/ui/tui/mod.rs` | TUI runtime shell, terminal lifecycle, single-frame render 완료 |
| `src/ui/tui/layout.rs` | 80x28/100x32/120x36 layout contract 완료 |
| `src/ui/tui/input.rs` | keyboard/mouse/save-load request mapping 완료 |
| `src/ui/tui/viewport.rs` | terminal cell <-> dungeon pos 변환 완료 |
| `src/ui/tui/render_map.rs` | ASCII map/entity rendering 완료 |
| `src/ui/tui/render_panels.rs` | status/log/command/inspect panel rendering 완료 |
| `src/ui/tui/effects.rs` | `GameEvent -> UiEffectEvent` projection 완료 |
| `src/ui/tui/theme.rs`, `config.rs` | runtime config/theme token 완료 |
| `tests/ui_layout.rs` | layout regression 완료 |
| `tests/ui_input_mapping.rs` | keyboard/mouse/save-load bridge 검증 완료 |
| `tests/ui_effect_projection.rs` | projection non-hash 검증 완료 |
| `tests/ui_runtime_smoke.rs` | runtime smoke 검증 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test --test ui_layout --test ui_input_mapping --test ui_effect_projection --test ui_runtime_smoke: pass
cargo run --bin aihack -- --seed 42: pass (small-terminal fallback render + clean exit)
```

Phase 10에서 의도적으로 제외된 항목:

- advanced drag/drop inventory UX
- v0.2/v0.3 animation-heavy polish
- AI prompt UI / narrative UI
- multiplayer/network UI


## 5.11 Phase 11 AI API Freeze 완료 요약

2026-05-17에 Phase 11은 `.omx/plans/prd-phase11-ai-api-freeze.md`와 `.omx/plans/test-spec-phase11-ai-api-freeze.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/core/action.rs` | `ActionIntent`, `NarrativeTopic` 추가 완료 |
| `src/core/observation.rs` | canonical `Observation`, `PlayerObservation`, `EntityObservation`, `ActionSpace`, `RunStateSummary` 완료 |
| `src/core/mod.rs` | AI-facing export surface 정리 완료 |
| `tests/ai_api_schema.rs` | observation/action_space fixture/compatibility 검증 완료 |
| `tests/action_space.rs` | `legal_actions` -> `action_space` mapping consistency 검증 완료 |
| `tests/observation.rs` | action space / run state regression 보강 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test --test ai_api_schema --test action_space: pass
phase9 headless baseline unchanged: seed=42 turns=100 final_hash=4c77dafb19dd2226
```

Phase 11에서 의도적으로 제외된 항목:

- network/OpenAPI/gRPC transport
- auth/tenant/session model
- phase12 narrative semantics
- phase13 decision-support execution semantics


## 5.12 Phase 12 LLM Narrative 완료 요약

2026-05-17에 Phase 12는 `.omx/plans/prd-phase12-llm-narrative.md`와 `.omx/plans/test-spec-phase12-llm-narrative.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/llm/narrative.rs` | provider trait, timeout/fallback envelope, deterministic fallback 완료 |
| `src/llm/mod.rs` | narrative module export 완료 |
| `src/ui/tui/mod.rs` | narrative consumer lines bridge 완료 |
| `tests/llm_narrative.rs` | success/timeout/failure/empty/non-hash 검증 완료 |
| `tests/ui_runtime_smoke.rs` | narrative consumer smoke 검증 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test --test llm_narrative --test ui_runtime_smoke --test ai_api_schema --test action_space: pass
Phase 9 headless baseline unchanged: seed=42 turns=100 final_hash=4c77dafb19dd2226
```

Phase 12에서 의도적으로 제외된 항목:

- decision support / command recommendation
- remote auth/network model
- narrative persistence in save files
- prompt editing UI


## 5.13 Phase 13 LLM Decision Support 완료 요약

2026-05-17에 Phase 13은 `.omx/plans/prd-phase13-llm-decision-support.md`와 `.omx/plans/test-spec-phase13-llm-decision-support.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/llm/decision.rs` | suggestion envelope, provider trait, fallback/disabled policy 완료 |
| `src/llm/mod.rs` | narrative + decision export 완료 |
| `tests/llm_decision_support.rs` | legal/illegal/timeout/non-hash/submit-path 검증 완료 |
| `tests/ui_runtime_smoke.rs` | decision support consumer smoke 검증 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test --test llm_decision_support --test llm_narrative --test ui_runtime_smoke: pass
Phase 9 headless baseline unchanged: seed=42 turns=100 final_hash=4c77dafb19dd2226
```

Phase 13에서 의도적으로 제외된 항목:

- autonomous self-play
- tool use / remote orchestration
- command auto-execution without validator gate
- chain-of-thought exposure UI


## 5.14 Phase 14 Release Candidate Hardening 완료 요약

2026-05-18에 Phase 14는 `.omx/plans/prd-phase14-release-candidate-hardening.md`와 `.omx/plans/test-spec-phase14-release-candidate-hardening.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `tests/release_candidate.rs` | multi-seed RC baseline smoke 완료 |
| `BUILD_GUIDE.md` | RC checklist, evidence commands, triage guidance 완료 |
| `README.md` | current release status summary 정렬 완료 |
| `CHANGELOG.md` | RC hardening note 추가 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
seed=42 turns=1000 final_turn=20 final_hash=4c77dafb19dd2226
seed=7 turns=1000 final_turn=28 final_hash=6eb33e9470d41b66
seed=1234 turns=1000 final_turn=18 final_hash=b88149ab5bb10bda
```

Known debt triage:

- release blocker: none
- non-blocking known issue: small-terminal fallback only shows message, not interactive degraded UI
- post-RC follow-up: v0.2/v0.3 UX polish, provider-backed live integrations

## 5.15 Phase 15 v0.2 Accessibility and UX Polish 완료 요약

2026-05-18에 Phase 15는 `.omx/plans/prd-phase15-v0-2-accessibility-ux.md`와 `.omx/plans/test-spec-phase15-v0-2-accessibility-ux.md` 기준으로 완료되었다.

구현 책임:

| 파일 | 구현 상태 |
| --- | --- |
| `src/ui/tui/input.rs` | hover inspect, inspect panel inventory primary-action click, command hint click 추가 완료 |
| `src/ui/tui/mod.rs` | `hovered_pos`, `focused_panel`, `UiTheme` selection, reduced-motion projection 연결 완료 |
| `src/ui/tui/render_panels.rs` | read-only inspect, priority message, command hint, inventory summary 텍스트 렌더 완료 |
| `src/ui/tui/effects.rs` | reduced-motion TTL 축약 path 추가 완료 |
| `tests/ui_input_mapping.rs` | non-turn inspect, inventory click equivalence 검증 추가 완료 |
| `tests/ui_layout.rs` | accessible text priority/hint 검증 추가 완료 |
| `tests/ui_effect_projection.rs` | reduced motion / high contrast non-hash 검증 추가 완료 |
| `tests/ui_runtime_smoke.rs` | hovered inspect panel smoke 추가 완료 |

검증 결과:

```text
cargo fmt --check: pass
cargo test --test save_load --test replay --test ui_input_mapping --test ui_layout --test ui_effect_projection --test ui_runtime_smoke: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
```

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

Status: 완료.

Acceptance:

- accepted + turn advance 된 player command 뒤에만 monster phase가 실행된다.
- jackal은 deterministic wander를 수행한다.
- goblin은 LOS 안에서만 player를 추적한다.
- floating eye는 stationary를 유지한다.
- `EntityMoved`는 actor identity를 포함한다.

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


## Phase 16~20 Gap Closure 구현 요약

### Phase 16: RunState & CommandIntent 계약 정렬

- `src/core/session.rs`: `RunState`를 spec.md 8.2 계약과 일치시켜 6개 변이체를 추가했다. `submit()`을 상태별 분기 처리하도록 재구성했다.
- `src/core/action.rs`: `AcknowledgeMore`, `DirectionalAction`, `InventoryAction`를 추가했다.
- `src/core/observation.rs`: `RunStateSummary`를 7개 상태로 확장하고, `legal_actions()`을 각 상태별로 분기했다.
- `src/core/event.rs`: `GameEvent::Message { priority, text }`와 `MessagePriority` enum을 추가했다.
- `src/core/world.rs`: `last_death_cause` 필드를 추가했다.
- `src/systems/death.rs`: `state_after_deaths()`가 `GameOver { cause, final_score }`를 반환하도록 수정했다.
- `src/bin/aihack-headless.rs`: `new_for_playing()`을 사용하여 기존 headless runner 호환성을 유지했다.
- Phase 16 기준 hash: `seed=42 turns=1000 final_turn=20 final_hash=569bc36895258349`.

### Phase 17: Game Flow Screens

- `src/ui/tui/mod.rs`: `run_tui()`를 `RunState`별로 4개 화면으로 분기했다. Title/CharacterCreation/Playing/GameOver 렌더러를 구현했다.
- `src/ui/tui/render_panels.rs`: `title_lines()`, `character_creation_lines()`, `game_over_lines()`, `awaiting_direction_lines()`, `awaiting_inventory_lines()`, `more_prompt_lines()`를 추가했다.
- `src/ui/tui/input.rs`: `UiCommandCandidate::NewRun`을 추가했다.
- `tests/ui_screens.rs`: 8개 테스트로 화면 전환과 입력 처리를 검증했다.

### Phase 18: Debug Observation Panel (F9 Toggle)

- `src/ui/tui/mod.rs`: `TuiApp.debug_observation_visible` 상태를 추가하고, F9 키 입력 시 토글하도록 구현했다.
- `src/ui/tui/render_panels.rs`: `debug_observation_lines()`를 추가하여 schema_version, seed, turn, run_state, player 상태, visible tile/entity 수, inventory 수, action_space 수, last_events, legal_actions를 표시한다.
- `tests/ui_debug.rs`: 3개 테스트로 debug observation lines 생성, 필수 항목 포함, hash 무영향을 검증했다.

### Phase 19: Auto-Label Priority System

- `src/ui/tui/labels.rs`: `LabelKind`, `AutoLabel`, `collect_auto_labels()`, `filter_expired_labels()`를 구현했다.
- `src/ui/tui/render_map.rs`: `MapWidget`에 라벨 오버레이 렌더링을 추가했다.
- `src/ui/tui/effects.rs`: `UiEffectKind::NewEntityLabel`을 추가했다.
- `tests/ui_labels.rs`: 7개 테스트로 라벨 수집, 우선순위, 최대 3개 제한, 만료 필터링을 검증했다.

### Phase 20: 데이터 외부화 및 모듈 분리

- `src/domain/status.rs`: `Status`, `HungerState`를 구현하고, `GameWorld`에 `status()`/`set_status()`/`hunger_state()` 메서드를 추가했다.
- `src/data/mod.rs`: TOML 파싱 로더(`load_items`, `load_monsters`, `load_level`)를 구현했다.
- `src/data/items.toml`, `monsters.toml`, `levels/main_1.toml`: 외부 데이터 파일을 생성했다.
- `tests/data_loading.rs`: 9개 테스트로 TOML 파일 로딩, Status 생성, HungerState 계산을 검증했다.
- `Cargo.toml`에 `toml` crate 의존성을 추가했다.


## Phase 5 구현 요약

- `src/domain/level.rs`를 추가하여 `GameLevel`, `LevelRegistry`, `main:1/main:2` fixed fixture를 정의했다.
- `GameWorld`를 단일 map에서 `levels + current_level + EntityStore + Inventory` 구조로 전환했다.
- actor/item 위치 계약을 `EntityLocation::OnMap { level, pos }`로 확장하고 player location helper로 current level invariant를 유지한다.
- `CommandIntent::Descend`, `CommandIntent::Ascend`, `GameEvent::LevelChanged`를 추가했다.
- `tests/levels.rs`, `tests/stairs.rs`를 추가했고 전체 `cargo test`를 통과했다.
- Phase 7 기준 hash: `seed=42 turns=100 final_turn=20 final_hash=5aecd83cf284cb25`.
