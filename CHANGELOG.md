# AIHack Changelog

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
