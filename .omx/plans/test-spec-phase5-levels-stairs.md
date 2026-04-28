# Test Spec: AIHack Phase 5 Levels and Stairs

문서 상태: approved-test-spec
작성일: 2026-04-29
대상 PRD: `.omx/plans/prd-phase5-levels-stairs.md`
범위: Phase 5 level registry/stairs/current level snapshot 검증만 해당

## 1. 테스트 목표

Phase 5 구현이 다음 성질을 만족하는지 검증한다.

1. `LevelRegistry`가 fixed `main:1`, `main:2` level을 보유한다.
2. `GameWorld.current_level`과 player `EntityLocation::OnMap.level`이 항상 일치하며 atomic helper로만 갱신된다.
3. `Descend`/`Ascend`는 stairs tile에서만 accepted + turn advance 된다.
4. Level transition은 `TurnStarted -> LevelChanged` event order를 가진다.
5. 1층 door/item/player state는 2층 왕복 후 보존된다.
6. Snapshot hash는 current level과 level별 map/entity location state를 포함한다.
7. Observation은 current level과 stairs legal actions를 노출한다.
8. Phase 2/3/4 regression은 계속 통과한다.
9. procedural generation, monster AI, file save/load, TUI scope creep이 없다.

## 2. 테스트 매트릭스

| ID | 종류 | 대상 | 검증 내용 |
| --- | --- | --- | --- |
| T1 | unit | `LevelRegistry` | exactly main:1/main:2 |
| T2 | unit | fixed fixture | 1층 stairs down, 2층 stairs up |
| T3 | unit | actor/item `EntityLocation` | OnMap includes LevelId and actor uses location contract |
| T4 | integration | descend reject | non-stairs descend rejected/no-turn |
| T5 | integration | descend accept | stairs down accepted/turn/current level changed |
| T6 | integration | ascend reject | non-stairs/up-less ascend rejected/no-turn |
| T7 | integration | ascend accept | stairs up accepted/turn/current level changed |
| T8 | integration | event order | `TurnStarted -> LevelChanged` |
| T9 | integration | landing | descend lands `(5,5)`, ascend lands `(34,15)` |
| T10 | integration | state preservation | 1층 door/item state survives round trip |
| T11 | integration | snapshot | current level affects hash |
| T12 | integration | snapshot | level map state affects hash |
| T13 | integration | observation | current level and stairs actions |
| T14 | regression | Phase 2-4 | movement/doors/vision/combat/death/items/inventory pass |
| T15 | audit | dependency boundary | legacy direct reference 없음 |
| T16 | audit | scope boundary | procgen/AI/save/TUI 없음 |
| T17 | quality | cargo | fmt/clippy/test 통과 |

## 3. Unit Test 상세

### T1: level registry contains two fixed levels

Given:

```rust
let world = GameWorld::fixture_phase5();
```

Expected:

```text
world.levels.len() = 2
world.levels contains LevelId { branch: Main, depth: 1 }
world.levels contains LevelId { branch: Main, depth: 2 }
world.current_level = main:1
```

### T2: fixed level stairs match spec

Expected:

```text
main:1 map tile at (34,15) = StairsDown
main:2 map tile at (5,5) = StairsUp
main:2 has no StairsDown tile in Phase 5
jackal EntityId(2) remains OnMap { level: main:1, pos: (6,5) }
goblin EntityId(3) remains OnMap { level: main:1, pos: (20,12) }
```

### T3: actor/item entity location includes level

Expected:

```text
player actor payload location = OnMap { level: main:1, pos: (5,5) }
actor payload no longer exposes standalone pos without level
potion item payload location = OnMap { level: main:1, pos: (8,5) }
inventory item location has no level
consumed item location has no level
level-aware helpers include alive_actor_at(level,pos), alive_hostile_at(level,pos), items_at(level,pos)
compile-facing tests or source audit prove standalone actor pos field is gone or not publicly mutable
player_location()/set_player_location(level,pos) or equivalent atomic helper exists
```

## 4. Stairs Integration Tests

### T4: descend rejected away from stairs

Setup:

- player at `(5,5)` on main:1.

When:

```rust
session.submit(CommandIntent::Descend)
```

Expected:

```text
accepted = false
turn_advanced = false
turn unchanged
current_level = main:1
snapshot_hash unchanged
CommandRejected reason contains "stairs down"
```

### T5: descend accepted on stairs down

Setup:

- player at main:1 `(34,15)`.

When:

```rust
session.submit(CommandIntent::Descend)
```

Expected:

```text
accepted = true
turn_advanced = true
current_level = main:2
player_pos = (5,5)
player location = OnMap { level: main:2, pos: (5,5) }
current_level == player.location.level
```

### T6: ascend rejected away from stairs up

Setup:

- player on main:2 but not `(5,5)`.

Expected:

```text
accepted = false
turn_advanced = false
current_level remains main:2
snapshot_hash unchanged
CommandRejected reason contains "stairs up"
```

### T7: ascend accepted on stairs up

Setup:

- after T5 descend, player on main:2 `(5,5)`.

When:

```rust
session.submit(CommandIntent::Ascend)
```

Expected:

```text
accepted = true
turn_advanced = true
current_level = main:1
player_pos = (34,15)
player location = OnMap { level: main:1, pos: (34,15) }
current_level == player.location.level
```

### T8: level change event order

Expected for accepted transition:

```text
events[0] = TurnStarted { turn }
events[1] = LevelChanged { entity: player_id, from, to }
```

Rules:

- No `EntityMoved` event is required for level transition.
- Rejected transitions produce only `CommandRejected` and do not append to event log.

### T9: landing positions are stable

Expected:

```text
main:1 -> main:2 landing = main:2 StairsUp (5,5)
main:2 -> main:1 landing = main:1 StairsDown (34,15)
```

## 5. State Preservation Tests

### T10: level1 state survives round trip

Setup:

1. Open 1층 door at `(10,5)`.
2. Pick up potion at `(8,5)` or verify potion consumed/location change after Phase 4 command.
3. Move player to stairs down `(34,15)`.
4. Descend to main:2.
5. Ascend back to main:1.

Expected:

```text
1층 door at (10,5) remains Open
jackal/goblin remain on main:1 at their Phase 3 positions unless changed by explicit combat tests
potion location/inventory/consumed state remains exactly as before descend
inventory letters remain stable
player returns to (34,15)
current_level = main:1
```

Test helper may set player position directly to reduce command length. This is allowed because Phase 5 tests are level transition tests, not pathfinding tests.

### T11: current level affects snapshot hash

Given two sessions with same seed and identical level maps/entities except current level:

Expected:

```text
hash(main:1) != hash(main:2)
```

### T12: level map state affects snapshot hash

Given:

- session A: main:2 door `(8,5)` closed.
- session B: same but main:2 door `(8,5)` open.

Expected:

```text
hash(A) != hash(B)
level serialization order is deterministic main:1 -> main:2
```

## 6. Observation Tests

### T13: observation includes current level and stairs actions

Expected:

```text
at main:1 (34,15), legal_actions contains Descend
at main:1 (5,5), legal_actions does not contain Descend
at main:2 (5,5), legal_actions contains Ascend
Observation.current_level equals GameWorld.current_level
visible_tiles come from current level only
```

## 7. Regression Tests

Required commands:

```bash
cargo test --test movement
cargo test --test doors
cargo test --test vision
cargo test --test combat
cargo test --test death
cargo test --test items
cargo test --test inventory
cargo test --test levels
cargo test --test stairs
cargo test
```

Regression expectations:

- Phase 2 movement/door/vision semantics remain unchanged on current level.
- Phase 3 bump attack/death semantics remain unchanged on current level.
- Phase 4 pickup/wield/quaff semantics remain unchanged on main:1.
- Item/inventory letter and consumed tombstone policies survive level transitions.

## 8. Audit / Scope Tests

### T15: legacy boundary

Command:

```bash
rg "legacy_nethack_port_reference" src Cargo.toml tests
```

Expected:

```text
no matches
```

### T16: scope boundary

Manual/rg audit:

```bash
rg "monster AI|pathfind|procedural|save/load|ratatui|crossterm|teleport" src Cargo.toml Cargo.lock tests
rg "rand::|thread_rng|random" src
```

Allowed matches:

- `rand::` imports are allowed only in `src/core/rng.rs`.
- `Stairs`/`Level` terms are expected in Phase 5 files.
- No procedural generation, AI/pathfinding, file save/load, TUI, teleport implementation should appear in `src`.

## 9. Quality Gate

Required final command set:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --test levels
cargo test --test stairs
cargo run --bin aihack-headless -- --seed 42 --turns 100
rg "legacy_nethack_port_reference" src Cargo.toml tests
rg "monster AI|pathfind|procedural|save/load|ratatui|crossterm|teleport" src Cargo.toml Cargo.lock tests
rg "rand::|thread_rng|random" src
git diff --check -- src tests Cargo.toml Cargo.lock README.md spec.md implementation_summary.md DESIGN_DECISIONS.md BUILD_GUIDE.md CHANGELOG.md audit_roadmap.md designs.md
```

Pass criteria:

- All commands exit 0, except scope `rg` commands may exit 1 for no matches.
- Headless final hash is deterministic across two identical runs.
- New final hash is recorded in docs after implementation.
- No Phase 6+ or save/TUI/procgen scope creep is introduced.

## 10. Failure Triage

1. If Phase 2/3/4 regression fails, first restore current-level map/entity helper semantics.
2. If level state preservation fails, check actor/item `EntityLocation::OnMap.level`, level-aware query helpers, and `LevelRegistry` mutation path before changing tests.
3. If stairs transitions happen via `Move`, reject that design and keep transition explicit through `Descend`/`Ascend`.
4. If implementation requires procedural generation, stop and revise PRD because Phase 5 uses fixed fixtures only.
5. If implementation requires file save/load, stop and revise PRD because Phase 9 owns file persistence.
