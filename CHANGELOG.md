# AIHack Changelog

## 2026-05-18

### Added

- 문서-구현 Gap Closure 계획 문서(`GAP_CLOSURE_ROADMAP.md`)를 작성하여 spec.md 8.2/8.3, designs.md와 실제 코드 간의 25개 미구현 항목을 식별하고 Phase 16~20 구현 로드맵을 수립했다.
- Phase 16 RunState & CommandIntent 계약 정렬을 완료하여 Title, CharacterCreation, AwaitingDirection, AwaitingInventorySelection, MorePrompt, GameOver { cause, final_score } 상태를 추가하고, AcknowledgeMore 명령과 DirectionalAction, InventoryAction 타입을 추가했다.
- Phase 16 GameEvent::Message와 MessagePriority를 추가하여 TUI 메시지 로그의 중요도 표시 계약을 구현했다.
- Phase 16 GameWorld.last_death_cause를 추가하여 사망 원인 기록과 GameOver 상태 생성의 정합성을 확보했다.
- Phase 17 Game Flow Screens를 구현하여 Title, Character Creation, Game Over 화면을 TUI에 추가했다.
- Phase 17 `render_panels.rs`에 title_lines, character_creation_lines, game_over_lines, awaiting_direction_lines, awaiting_inventory_lines, more_prompt_lines 함수를 추가했다.
- Phase 17 `UiCommandCandidate::NewRun`을 추가하여 Game Over 화면에서 새 게임 시작을 지원했다.
- Phase 17 TUI 키 입력을 RunState별로 분기 처리하여 각 화면에 맞는 입력 매핑을 구현했다.
- Phase 17 `tests/ui_screens.rs`를 추가하여 Title/CharacterCreation/GameOver/ MorePrompt/AwaitingDirection 상태 전환과 입력 처리를 검증했다.
- Phase 18 F9 Debug Observation 토글을 구현하여 F9 키 입력 시 Observation 데이터 패널을 표시/숨김한다.
- Phase 18 `render_panels::debug_observation_lines()`를 추가하여 schema_version, seed, turn, run_state, player 상태, visible tile/entity 수, inventory 수, action_space 수, last_events, legal_actions를 표시한다.
- Phase 18 `TuiApp.debug_observation_visible` 상태를 추가하여 UI-only 토글 기능을 구현했다.
- Phase 18 `tests/ui_debug.rs`를 추가하여 debug observation lines 생성, 필수 항목 포함, hash 무영향을 검증했다.
- Phase 19 Auto-Label Priority System을 구현하여 hostile adjacent, low HP warning, stairs, unidentified item, passive monster 라벨을 자동 수집하고 우선순위별로 최대 3개 표시한다.
- Phase 19 `src/ui/tui/labels.rs`를 추가하여 LabelKind, AutoLabel, collect_auto_labels, filter_expired_labels를 구현했다.
- Phase 19 `MapWidget`에 라벨 오버레이 렌더링을 추가하여 맵 위에 자동 라벨 텍스트를 표시한다.
- Phase 19 `TuiApp`에 `active_labels` 상태를 추가하고, 턴 진행 시 새 라벨을 수집하도록 구현했다.
- Phase 19 `UiEffectKind::NewEntityLabel`을 추가하여 자동 라벨 관련 UI effect를 확장했다.
- Phase 19 `tests/ui_labels.rs`를 추가하여 라벨 수집, 우선순위 정렬, 최대 3개 제한, 만료 필터링을 검증했다.
- Phase 20 `src/domain/status.rs`를 생성하여 Status, HungerState를 구현하고 GameWorld에 `status()`/`set_status()`/`hunger_state()` 메서드를 추가했다.
- Phase 20 `src/data/items.toml`, `monsters.toml`, `levels/main_1.toml`을 생성하여 외부 데이터 파일 구조를 도입했다.
- Phase 20 `src/data/mod.rs`를 생성하여 TOML 파싱 로더(`load_items`, `load_monsters`, `load_level`)를 구현했다.
- Phase 20 `Cargo.toml`에 `toml` crate 의존성을 추가했다.
- Phase 20 `tests/data_loading.rs`를 추가하여 TOML 파일 로딩, Status 생성, HungerState 계산을 검증했다.

### Changed

- `RunState`를 spec.md 8.2 계약과 일치시켜 6개 변이체를 추가하고, `submit()`을 상태별 분기 처리하도록 재구성했다.
- `GameSession::new()`를 Title 상태로 시작하되, 기존 테스트 호환성을 위해 `new_for_playing()`를 추가했다.
- headless runner와 release candidate 테스트의 기준 hash를 Phase 16 변경사항에 맞게 갱신했다.
- Phase 15 v0.2 Accessibility and UX Polish를 완료하여 hover read-only inspect, inspect-panel inventory click, priority message, command hint, reduced motion/high contrast presentation tests를 추가했다.
- Phase 14 Release Candidate Hardening을 완료하여 multi-seed RC baseline smoke와 release gate triage를 추가했다.

### Changed

- TUI runtime이 `hovered_pos`, `focused_panel`, `UiTheme` selection을 presentation-only 상태로 유지하면서 mixed-input UX를 확장하도록 정렬했다.
- release candidate 문서/체크리스트를 현재 구현과 정렬하고 known debt를 blocker/non-blocking/deferred로 분류했다.


## 2026-05-17

### Added

- Phase 13 LLM Decision Support 구현을 완료하여 suggestion envelope, validator-gated execution, decision support smoke tests를 추가했다.

### Changed

- decision support를 persistence truth와 분리하고 fallback/disabled policy를 고정했다.


## 2026-05-17

### Added

- Phase 12 LLM Narrative 구현을 완료하여 provider-agnostic narrative adapter, timeout/fallback policy, narrative consumer smoke tests를 추가했다.

### Changed

- narrative output을 presentation-only artifact로 고정하고 core hash/save/load/replay와 분리했다.


## 2026-05-17

### Added

- Phase 11 AI API Freeze 구현을 완료하여 `ActionIntent`, canonical `Observation` DTO, `ActionSpace`, AI schema compatibility tests를 추가했다.

### Changed

- `Observation`을 AI-facing contract로 고정하고 `legal_actions`는 compatibility alias로 유지했다. save/load와 TUI가 same AI schema를 소비하도록 정렬했다.


## 2026-05-17

### Added

- Phase 10 TUI Adapter 구현을 완료하여 `src/ui/tui/*` runtime shell, layout/input/effect modules, layout/mouse/effect/smoke tests를 추가했다.

### Changed

- `src/main.rs`를 ASCII TUI adapter 진입점으로 전환했다. small-terminal 환경에서는 fallback 메시지를 렌더하고 clean exit 하도록 했다.


## 2026-05-17

### Added

- Phase 9 Save Load Replay 구현을 완료하여 `SaveDataV1`, `RngStateV1`, replay JSONL, `--save/--load/--replay-out` CLI를 추가했다.
- `tests/save_load.rs`와 `tests/replay.rs`를 추가하여 save/load hash equality, continuation equality, replay JSONL schema를 검증했다.

### Changed

- snapshot 기반 persistent state를 explicit save schema로 고정했다. Phase 9 기준 `seed=42 turns=100` final hash는 `4c77dafb19dd2226`, `seed=43 turns=100` final hash는 `f8324eacbce50087`이다.


## 2026-05-16

### Added

- Phase 8 Legacy Rule Absorption 구현을 완료하여 20개 golden scenario(P8-G01~P8-G20)와 kick/drop/wear/pray, identify/teleport, hunger/luck/score/meta state를 추가했다.

### Changed

- snapshot hash 입력에 nutrition/luck/prayer cooldown/paralysis/gold/kill_count/identified item state를 포함했다. Phase 8 기준 `seed=42 turns=100` final hash는 `4c77dafb19dd2226`, `seed=43 turns=100` final hash는 `f8324eacbce50087`이다.


## 2026-05-16

### Added

- Phase 7 NetHack Interaction Set 1 구현을 완료하여 `Search`, hidden door/trap reveal, `Throw`, `Zap`, `Read` 상호작용을 추가했다.
- `tests/traps.rs`와 `tests/projectiles.rs`를 추가하여 trap/search/reveal, throw/zap/charge/wall stop golden scenario를 검증했다.

### Changed

- starting inventory를 wand/scroll/rock까지 확장하고 hidden tile/charge/item location을 snapshot hash 입력에 포함했다. Phase 7 기준 `seed=42 turns=100` final hash는 `5aecd83cf284cb25`, `seed=43 turns=100` final hash는 `5f5d5b89faa9a834`이다.


## 2026-05-16

### Added

- Phase 6 Monster AI 구현을 완료하여 `MonsterAiKind`, current-level hostile monster turn loop, goblin chase, jackal wander, floating eye stationary policy를 추가했다.
- `tests/monster_ai.rs`를 추가하여 turn gate, actor order, chase/wander/stationary, off-level freeze, player death stop을 검증했다.

### Changed

- `GameEvent::EntityMoved`를 `entity` id 포함 형태로 확장하고 player/monster movement event shape를 통일했다.
- headless deterministic baseline을 monster phase 포함 기준으로 갱신했다. Phase 6 기준 `seed=42 turns=100` final hash는 `2fb549b5d2e1e67f`이고 `seed=43 turns=100` final hash는 `ec98b802759e109c`이다.

## 2026-04-28

### Added

- Phase 5 Levels and Stairs 구현을 완료하여 fixed `main:1/main:2` level registry, `Descend`, `Ascend`, `LevelChanged` event를 추가했다.
- actor/item 위치를 `EntityLocation::OnMap { level, pos }`로 확장하고 `tests/levels.rs`, `tests/stairs.rs`를 추가했다.

### Changed

- `GameWorld`를 단일 map에서 `levels + current_level + EntityStore + Inventory` 구조로 전환했다.
- Snapshot hash 입력에 current level, deterministic level map state, level-aware actor/item location을 포함했다. Phase 5 기준 `seed=42 turns=100` final hash는 `88886c28698a1730`이다.

### Added

- Phase 4 Items and Inventory 구현을 완료하여 item entity, stable inventory letter, pickup, show inventory, dagger wield, healing potion quaff를 추가했다.
- `ItemPickedUp`, `ItemEquipped`, `ItemConsumed`, `EntityHealed` event와 `Observation.inventory`를 추가했다.
- `tests/items.rs`, `tests/inventory.rs`로 item fixture, pickup, wield, quaff, consumed tombstone, serde/snapshot roundtrip을 검증했다.

### Changed

- `Entity`를 `EntityPayload::Actor | EntityPayload::Item` 구조로 리팩터링했다.
- Snapshot hash 입력에 item location, assigned letter, inventory entries, equipped melee 상태를 포함했다. Phase 4 기준 `seed=42 turns=100` final hash는 `00ba578d933177f2`이다.


### Added

- Phase 3 Combat and Death 구현을 완료하여 `EntityStore`, player/monster entity, jackal/goblin/floating eye factory, bump attack, `AttackResolved`, `EntityDied`, player `RunState::GameOver`를 추가했다.
- `tests/combat.rs`와 `tests/death.rs`를 추가하여 stable `EntityId`, tombstone, hit/damage formula, monster death, dead monster movement, player death, snapshot hash 변경을 검증했다.

### Changed

- `GameWorld`를 map + `EntityStore` + `player_id` 구조로 확장하고 player position 원천을 player entity로 이전했다.
- Snapshot hash 입력에 entity id/kind/position/hp/alive 상태를 포함하도록 확장했다. Phase 3 기준 `seed=42 turns=100` final hash는 `8b20a23301eea977`이다.


### Added

- Phase 2 Map, Movement, Doors, Vision 구현을 완료하여 `GameMap`, `TileKind`, `DoorState`, `GameWorld`, `Pos/Direction`, movement/doors/vision systems, 최소 `Observation.visible_tiles`를 추가했다.
- 40x20 fixture map, player start `(5,5)`, radius 8 LOS, wall/closed-door blocker, open-door transparency, rejected command non-advance 검증을 추가했다.

### Changed

- Snapshot hash 입력에 `player_pos`, map size, map tile state를 포함하도록 확장했다. Phase 2 기준 `seed=42 turns=100` final hash는 `1aad6f4049778b0e`이다.

### Added

- Phase 1 Headless Core 구현을 완료하여 루트 `Cargo.toml`, `src/main.rs`, `src/bin/aihack-headless.rs`, `src/lib.rs`, `src/core/*` 최소 런타임을 추가했다.
- `GameRng`, `GameSession::new(seed)`, `CommandIntent::Wait`, `TurnOutcome`, stable FNV-1a snapshot hash를 추가했다.
- seed/turns 기반 `aihack-headless` runner를 추가하여 같은 seed와 turns의 final hash를 재현 가능하게 했다.
- Phase 1 검증 결과로 `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, seed 42/43 headless deterministic 검증을 통과했다.

### Added

- Added a modern TUI/UX refactoring plan grounded in Cogmind/Brogue/Ratatui references.
- Documented phased ASCII UI modernization: readability-first v0.1, mouse-accessible v0.2, presentation-only ASCII effects v0.3.
- Added UI-only contracts for `UiRuntimeConfig`, `UiInputEvent`, `UiCommandCandidate`, and `UiEffectEvent`.

### Changed

- Expanded TUI implementation and audit plans with layout, input mapping, effect projection, reduced-motion, and replay-hash verification criteria.


### Changed

- Moved the previous NetHack Rust port into `legacy_nethack_port_reference/`.
- Removed the `.gitignore` rule that ignored all Markdown documents.
- Reframed the root project as a Rust-native AIHack runtime rebuild.
- Added a new reference-grade root document set:
  - `README.md`
  - `spec.md`
  - `designs.md`
  - `implementation_summary.md`
  - `DESIGN_DECISIONS.md`
  - `BUILD_GUIDE.md`
  - `audit_roadmap.md`
  - `CHANGELOG.md`
- Added `legacy_nethack_port_reference/REFERENCE_INDEX.md` to document how the old codebase should be used as a reference.

### Decision

The old codebase is preserved as a reference and test knowledge base. New implementation work must not directly import legacy source files. The new runtime will be built around `GameSession`, deterministic turns, typed `Observation`, and validated `ActionSpace`.
