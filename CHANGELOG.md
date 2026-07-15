# AIHack Changelog

## [Unreleased] - 2026-07-15

### Added

- `AI_IMPLEMENTATION_DOC_STANDARD.md`에 맞춘 v0.3.0 리팩터링 구현 계획을 `spec.md`, `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`에 작성했다.
- build reproducibility, private state transaction, runtime content registry, accepted-turn 1000 검증, workspace 경계, local LLM transport/revision gate, provenance/compatibility의 R0~R8 Task와 종료 게이트를 정의했다.
- `PROVENANCE.md`와 `docs/compatibility/README.md`를 추가해 NetHack 3.6.7 공식 source checksum, 자산 상태, runtime 포함 차단 조건, NH367-C001..C010 record schema를 정의했다.
- `DOCUMENTATION_AUDIT_REPORT.md`를 추가해 AI 구현 문서 표준 12항목과 R0 자동 검증의 PASS 증거를 기록했다.
- ADR-0021~ADR-0027을 추가해 제품 범위, toolchain, state, content, workspace, LLM 권한, provenance 결정을 동결했다.
- 성장한 계획 문서 원문을 `.archive/*_archive_260715.md`에 immutable snapshot으로 보존했다.
- R1을 위해 `rust-toolchain.toml`, `deny.toml`, Linux/Windows CI workflow, build contract test를 추가했다.
- R2-1의 첫 세로 슬라이스로 `GameSession` 읽기 API(`seed`, `turn`, `run_state`, `event_log`)와 접근자 회귀 테스트를 추가했다.
- `tests/support/session_builder.rs`를 추가해 UI 상태 전환 테스트가 세션 필드 직접 대입 없이 fixture를 구성하도록 했다.
- `WorldInvariantError` 6종, `InvariantReport`, `tests/world_invariants.rs`를 추가해 현재 level/player/inventory 소유 관계를 명시적으로 검증하기 시작했다.
- `GameWorld`의 status·score·식별·사망원인 상태를 crate 외부 비공개로 전환하고 `Status` 및 score accessor로 테스트 fixture를 구성하도록 했다.
- `GameWorld::player_id()`를 추가하고 player identity 필드를 crate 외부 비공개로 전환했다.
- `TurnTransaction` working-copy 경로를 추가해 submit이 invariant 검증을 통과할 때만 world/turn/RNG/event log를 commit하도록 전환했다.
- `GameSession.world`와 `GameWorld.levels/entities/inventory`를 crate 외부 비공개로 전환하고, 읽기 accessor 및 저장 기반 `SessionBuilder` fixture로 integration test 경계를 정리했다.
- embedded TOML `ContentRegistry`와 `ContentError` 6종을 추가하고, item/monster factory와 main:1/main:2 map·초기 배치를 registry runtime으로 전환했다.
- `audit_report_1.md`를 추가하고, R1/R2 local PASS와 R3 bootstrap `ContentError` 경계가 아직 Hold라는 현재 상태를 활성 문서에 동기화했다.

### Changed

- README와 BUILD_GUIDE의 현재 실행 명령을 두 binary 환경에 맞는 `cargo run --locked --bin ...` 형태로 수정했다.
- `designs.md`를 local LLM CTA, 상태, timeout/stale/error, 접근성, core 무결성 계약 중심의 v0.3.0 target 문서로 재구성했다.
- 현재 구현과 v0.3.0 target을 분리하고, 과거 Phase 완료 기록은 archive와 기존 changelog 이력으로 이동했다.
- UI dependency를 RustSec 경고가 없는 ratatui 0.30/crossterm 0.29 단일 계열로 올리고 ADR-0028에 근거를 기록했다.
- build script를 locked command와 artifact fail-fast 계약으로 전환하고 TUI default-run을 `aihack`으로 고정했다.
- headless runner와 TUI가 세션 메타데이터·turn·상태·event log를 공개 field가 아니라 `GameSession` 읽기 API로 소비하도록 전환했다.
- `GameSession`의 meta/RNG/turn/run-state/event-log 저장 필드를 crate 외부 비공개로 전환했다.
- invariant 오류는 원본 session을 보존한 reject로 처리하고, AwaitingDirection의 실패 입력 후 Playing 복귀 동작은 유지했다.

### Security

- LLM은 loopback endpoint와 presentation-only 권한을 기본으로 하며, stale/invalid response와 자유 텍스트 state mutation을 차단하는 계획을 명시했다.
- 출처 상태가 `Approved`가 아닌 legacy 코드·데이터·문자열은 runtime 포함 및 배포를 금지하는 gate를 명시했다.

### Verification

- R0 문서 gate는 PASS다.
- R1의 로컬 fmt, clippy, test, release build, cargo audit, cargo deny gate는 통과했다. Linux/Windows 원격 CI는 workflow push 후에만 PASS로 기록한다.

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
  - `IMPLEMENTATION_SUMMARY.md`
  - `DESIGN_DECISIONS.md`
  - `BUILD_GUIDE.md`
  - `audit_roadmap.md`
  - `CHANGELOG.md`
- Added `legacy_nethack_port_reference/REFERENCE_INDEX.md` to document how the old codebase should be used as a reference.

### Decision

The old codebase is preserved as a reference and test knowledge base. New implementation work must not directly import legacy source files. The new runtime will be built around `GameSession`, deterministic turns, typed `Observation`, and validated `ActionSpace`.
