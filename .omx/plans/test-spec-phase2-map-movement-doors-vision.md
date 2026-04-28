# Test Spec: AIHack Phase 2 Map, Movement, Doors, Vision

문서 상태: approved-test-spec
작성일: 2026-04-28
대상 PRD: `.omx/plans/prd-phase2-map-movement-doors-vision.md`
범위: Phase 2 map/movement/doors/vision 검증만 해당

## 1. 테스트 목표

Phase 2 구현이 다음 성질을 만족하는지 검증한다.

1. 40x20 fixture map이 deterministic하게 생성된다.
2. bounds 밖 접근은 panic하지 않고 오류를 반환한다.
3. player는 `(5,5)`에서 시작한다.
4. floor/open door/stairs 이동은 accepted이고 turn을 진행한다.
5. wall/closed door/out-of-bounds 이동은 rejected이고 turn을 진행하지 않는다.
6. open/close door는 adjacent door 상태만 변경하고 event를 생성한다.
7. LOS radius 8은 벽/닫힌 문에서 차단되고 열린 문에서 통과한다.
8. `Observation.visible_tiles`는 player 기준 상대 좌표와 tile 정보를 포함한다.
9. legacy direct import와 Phase 3+ scope creep이 없다.

## 2. 테스트 매트릭스

| ID | 종류 | 대상 | 검증 내용 |
| --- | --- | --- | --- |
| T1 | unit | `GameMap` | fixture width=40, height=20 |
| T2 | unit | `GameMap` | bounds 밖 tile 접근 error |
| T3 | unit | `TileKind` | wall/closed door/open door movement blocker 판정 |
| T4 | unit | `Direction` | 8방향 delta 변환 |
| T5 | integration | `GameSession::new` | player_start `(5,5)` |
| T6 | integration | movement | floor 이동 accepted, turn +1 |
| T7 | integration | movement | wall/closed door/out-of-bounds rejected, turn 유지 |
| T8 | integration | diagonal movement | corner-cutting 금지 |
| T9 | integration | doors | closed door open accepted, `DoorChanged` event |
| T10 | integration | doors | open door close accepted, `DoorChanged` event |
| T11 | integration | doors | door 없는 방향 open/close rejected |
| T12 | unit/integration | vision | radius 8 범위 밖 tile 제외 |
| T13 | unit/integration | vision | wall blocks LOS |
| T14 | unit/integration | vision | closed door blocks LOS |
| T15 | unit/integration | vision | open door does not block LOS |
| T16 | integration | observation | `visible_tiles` contains player tile and relative delta |
| T17 | integration | legal actions | 최소 wait/move/open/close legal action 후보 생성 |
| T18 | integration | snapshot hash | movement/door change가 final hash에 반영되고 재현 가능 |
| T19 | audit | dependency boundary | legacy direct reference 없음 |
| T20 | audit | scope boundary | combat/item/monster/TUI 구현 없음 |
| T21 | quality | cargo | fmt/clippy/test 통과 |

## 3. Unit Test 상세

### T1: fixture map size

Expected:

```text
GameMap::fixture_phase2().width = 40
GameMap::fixture_phase2().height = 20
tile_count = 800
```

### T2: bounds 밖 접근

Cases:

```text
(-1, 0)
(0, -1)
(40, 0)
(0, 20)
```

Expected:

- `GameMap::tile(pos)` 또는 동등 API가 `Err(GameError::...)`를 반환한다.
- panic 금지.

### T3: movement/LOS blocker helper

Expected:

| Tile | movement passable | los transparent |
| --- | --- | --- |
| Wall | false | false |
| Floor | true | true |
| Door(Closed) | false | false |
| Door(Open) | true | true |
| StairsDown | true | true |
| StairsUp | true | true |

### T4: direction delta

Expected:

```text
North=(0,-1)
South=(0,1)
West=(-1,0)
East=(1,0)
NorthWest=(-1,-1)
NorthEast=(1,-1)
SouthWest=(-1,1)
SouthEast=(1,1)
```

## 4. Movement/Doors Integration Tests

### T5: player start

Given:

```rust
let session = GameSession::new(42);
```

Expected:

```text
session.world.player_pos = Pos { x: 5, y: 5 }
session.turn = 0
```

### T6: floor movement advances turn

Given:

- player at `(5,5)`.
- east tile `(6,5)` is floor.

When:

```rust
session.submit(CommandIntent::Move(Direction::East))
```

Expected:

```text
accepted = true
turn_advanced = true
player_pos = (6,5)
turn = 1
events contains EntityMoved or equivalent movement event
```

### T7: blockers do not advance turn

Cases:

- move into boundary wall.
- move into closed door.
- move out of bounds from near boundary in a fixture or synthetic map.

Expected:

```text
accepted = false
turn_advanced = false
player_pos unchanged
turn unchanged
events contains CommandRejected
```

### T8: diagonal corner-cutting 금지

Given:

- diagonal target is floor.
- one of the orthogonal side tiles is wall or closed door.

Expected:

- diagonal move rejected.
- turn unchanged.

### T9/T10: open/close door

Given:

- player adjacent to closed door.

When:

```rust
session.submit(CommandIntent::Open(Direction::East))
```

Expected:

```text
accepted = true
turn_advanced = true
door state: Closed -> Open
events contains DoorChanged { from: Closed, to: Open }
```

Then:

```rust
session.submit(CommandIntent::Close(Direction::East))
```

Expected:

```text
door state: Open -> Closed
events contains DoorChanged { from: Open, to: Closed }
```

### T11: invalid open/close rejected

Cases:

- open floor.
- open already open door.
- close closed door.
- close wall.

Expected:

- rejected, turn unchanged.

## 5. Vision/Observation Tests

### T12: radius 8

Given:

- player at `(5,5)`.

Expected:

- Chebyshev distance `<= 8` 후보만 visible result에 포함된다.
- distance `> 8` tile은 제외된다.

### T13: wall blocks LOS

Given:

- wall between player and target tile.

Expected:

- wall tile 자체는 visible일 수 있다.
- wall 뒤 target tile은 not visible 또는 `visible_tiles`에서 제외된다.

### T14: closed door blocks LOS

Given:

- closed door between player and target.

Expected:

- closed door tile 자체는 visible일 수 있다.
- closed door 뒤 target tile은 제외된다.

### T15: open door does not block LOS

Given:

- same setup as T14 but door opened.

Expected:

- tile beyond door becomes visible if radius 조건을 만족한다.

### T16: visible_tiles shape

Expected first principles:

```text
schema_version = 1
seed = session.meta.seed
turn = session.turn
player_pos = current player position
visible_tiles contains player tile
player tile rel = Delta { dx: 0, dy: 0 }
```

### T17: legal actions 최소 후보

Expected:

- `Wait` is always legal while Playing.
- passable adjacent direction yields `Move(dir)`.
- adjacent closed door yields `Open(dir)`.
- adjacent open door yields `Close(dir)`.

## 6. Snapshot/Headless Determinism Tests

### T18: movement/door hash deterministic

Test strategy:

- Run a fixed command sequence directly through `GameSession` twice:

```text
Move(East)
Move(East)
Open(East or fixture-specific door direction)
Wait
```

Expected:

- final snapshot hash is identical across two runs with seed 42.
- changing at least one accepted command changes final hash.

주의:

- Phase 2에서 snapshot hash 입력이 확장되므로 Phase 1 기준 hash 변경은 허용된다.
- 변경된 기준 hash는 구현 완료 시 문서에 기록해야 한다.

## 7. Audit Commands

품질 게이트:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

필터형 targeted tests:

```bash
cargo test map
cargo test movement
cargo test doors
cargo test vision
cargo test observation
```

Headless smoke:

```bash
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

레거시 직접 의존 검사:

```bash
rg "legacy_nethack_port_reference" src Cargo.toml
```

Expected:

- 결과 없음.

Scope creep 검사:

```bash
find src -maxdepth 4 -type f | sort
```

Expected:

- `combat`, `item`, `monster`, `ui`, `save` 구현 파일 없음.
- `domain/map`, `domain/tile`, `systems/movement`, `systems/doors`, `systems/vision`, `core/observation`, `core/world`, `core/position`은 허용.

## 8. 실패 시 수정 루프

1. 실패한 테스트 이름과 명령을 기록한다.
2. 원인을 Phase 2 범위 안에서만 수정한다.
3. 수정이 combat/item/monster/TUI/entity-store 전체 구현을 요구하면 중단하고 PRD 갱신 필요 여부를 보고한다.
4. `cargo fmt --check`, `cargo clippy`, `cargo test`, targeted tests를 다시 실행한다.
5. snapshot hash 기준이 바뀌면 문서를 먼저 동기화한다.

## 9. 완료 증거 형식

Ralph 완료 보고는 다음 형식을 포함해야 한다.

```text
변경 파일:
- ...

검증:
- cargo fmt --check: pass
- cargo clippy --all-targets -- -D warnings: pass
- cargo test: pass
- cargo test map: pass
- cargo test movement: pass
- cargo test doors: pass
- cargo test vision: pass
- cargo test observation: pass
- cargo run --bin aihack-headless -- --seed 42 --turns 100: pass, final_hash=<값>
- rg legacy direct refs: pass
- scope creep audit: pass

남은 리스크:
- ...
```
