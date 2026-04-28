# Test Spec: AIHack Phase 3 Combat and Death

문서 상태: approved-test-spec
작성일: 2026-04-28
대상 PRD: `.omx/plans/prd-phase3-combat-death.md`
범위: Phase 3 combat/death 검증만 해당

## 1. 테스트 목표

Phase 3 구현이 다음 성질을 만족하는지 검증한다.

1. `EntityStore`가 stable `EntityId`를 생성하고 player/monster를 저장한다.
2. player/jackal/goblin stat이 `spec.md` 실데이터와 일치한다.
3. `Move(dir)` 대상에 hostile monster가 있으면 movement 대신 bump attack이 실행된다.
4. hit formula와 damage formula가 deterministic하게 계산된다.
5. `AttackResolved`와 `EntityDied` event가 고정 순서로 생성된다.
6. monster death는 tombstone/alive=false를 남기고 movement blocker에서 제외된다.
7. player death는 `RunState::GameOver`를 만든다.
8. Phase 2 map/movement/doors/vision/observation 검증은 계속 통과한다.
9. item/inventory/monster AI/TUI scope creep이 없다.

## 2. 테스트 매트릭스

| ID | 종류 | 대상 | 검증 내용 |
| --- | --- | --- | --- |
| T1 | unit | `EntityStore` | first real id = `EntityId(1)`, stable lookup |
| T2 | unit | `EntityStore` | alive=false tombstone does not compact ids |
| T3 | unit | player factory | player hp/ac/hit_bonus/damage profile |
| T4 | unit | monster factory | jackal/goblin hp/ac/hit_bonus/damage profile |
| T5 | unit | hit formula | seeded d20 + bonuses vs defense |
| T6 | unit | damage formula | dice + bonus - reduction, min 1 |
| T7 | integration | bump attack | player does not move into occupied hostile tile |
| T8 | integration | attack event | `AttackResolved` contains attacker/defender/hit/damage |
| T9 | integration | monster death | hp <= 0 creates `EntityDied`, alive=false |
| T10 | integration | dead monster movement | dead monster no longer blocks movement |
| T11 | integration | player death | `RunState::GameOver`, `EntityDied` for player |
| T12 | integration | snapshot hash | entity hp/pos/alive state affects stable hash |
| T13 | regression | Phase 2 | map/movement/doors/vision/observation tests still pass |
| T14 | audit | dependency boundary | legacy direct reference 없음 |
| T15 | audit | scope boundary | item/inventory/monster AI/TUI 구현 없음 |
| T16 | quality | cargo | fmt/clippy/test 통과 |

## 3. Unit Test 상세

### T1: stable entity IDs

Given:

```rust
let mut store = EntityStore::new();
let player = store.spawn_player(Pos { x: 5, y: 5 });
let jackal = store.spawn_monster(MonsterKind::Jackal, Pos { x: 6, y: 5 });
```

Expected:

```text
player = EntityId(1)
jackal = EntityId(2)
store.get(player).kind = Player
store.get(jackal).kind = Monster(Jackal)
EntityId(0) is never assigned
```

### T2: tombstone does not compact IDs

Given:

- player `EntityId(1)`
- jackal `EntityId(2)`
- goblin `EntityId(3)`

When:

- jackal dies.

Expected:

```text
store.get(EntityId(2)).alive = false
store.get(EntityId(3)).id still EntityId(3)
next spawned id, if any, is EntityId(4)
```

Phase 3 does not need spawning after death, but the invariant should be testable if helper exists.

### T3: player factory matches spec

Expected:

```text
hp = 16
max_hp = 16
ac = 0
hit_bonus = 2
damage_bonus = 0
built-in attack profile = dagger
weapon_hit_bonus = 1
damage = 1d4
```

### T4: monster factories match spec

Expected:

```text
jackal: hp=4, ac=0, hit_bonus=0, damage=1d2
goblin: hp=6, ac=1, hit_bonus=1, damage=1d4
floating_eye: hp=8, ac=2, hit_bonus=0, damage=0
```

Phase 3 combat fixture must include jackal/goblin. floating eye passive is not implemented.

### T5: hit formula

Formula:

```text
attack_roll = d20 + attacker.hit_bonus + weapon.hit_bonus
defense = 10 + defender.ac
hit = attack_roll >= defense
```

Test strategy:

- Use `GameRng::new(42)` or deterministic fake roller helper.
- Assert exact d20 value if the API exposes it, or assert stable hit/miss result for known seed and attacker/defender pair.
- Include at least one forced hit and one forced miss by using fixed roller or controlled stats.

### T6: damage formula

Formula:

```text
damage = max(1, damage_roll + attacker.damage_bonus - defender.damage_reduction)
```

Expected:

- `1d4` damage with seed/fake roller stays in `1..=4`.
- negative reduction result clamps to 1.
- Phase 3 `damage_reduction = 0` unless a test-specific defender override is used.

## 4. Combat Integration Tests

### T7: bump attack does not move player

Given:

```text
player at (5,5)
jackal at (6,5)
```

When:

```rust
session.submit(CommandIntent::Move(Direction::East))
```

Expected:

```text
accepted = true
turn_advanced = true
player_pos remains (5,5)
jackal hp may decrease if hit
events contains AttackResolved
```

### T8: attack event shape

Expected event order for accepted bump attack:

```text
TurnStarted
AttackResolved { attacker: player_id, defender: jackal_id, hit, damage }
optional EntityDied if defender hp <= 0
```

Rules:

- `damage = 0` is allowed only when `hit = false`.
- `damage >= 1` when `hit = true`.
- `attacker` and `defender` are stable `EntityId`s.

### T9: monster death creates event and tombstone

Test strategy:

- Use low-hp synthetic jackal or repeated bump attacks with deterministic seed until hp <= 0.

Expected:

```text
EntityDied { entity: jackal_id, cause: DeathCause::Combat { attacker: player_id } }
store.get(jackal_id).alive = false
jackal tile no longer blocks movement
```

### T10: dead monster no longer blocks movement

Given:

- jackal at `(6,5)` dead/alive=false.

When:

```rust
session.submit(CommandIntent::Move(Direction::East))
```

Expected:

```text
accepted = true
player_pos = (6,5)
```

### T11: player death enters GameOver

Test strategy:

- Use direct system helper `apply_damage_and_check_death` or synthetic goblin attack helper.
- Do not implement monster AI just to kill player.

Expected:

```text
player hp <= 0
session.state = RunState::GameOver { ... } or current GameOver variant
TurnOutcome.next_state = GameOver
EntityDied event for player
```

If current `RunState::GameOver` has no cause/score fields, Phase 3 may keep the existing enum shape but must document that score/cause expansion is deferred.

### T12: snapshot hash includes entity state

Given:

- two seed 42 sessions.
- same combat command sequence.

Expected:

- final hash identical.

Then:

- alter one accepted attack/movement command or entity hp.

Expected:

- final hash differs.

## 5. Phase 2 Regression Tests

The following commands must continue to pass:

```bash
cargo test map
cargo test movement
cargo test doors
cargo test vision
cargo test observation
```

Expected:

- Existing movement/door/LOS semantics remain true.
- If adjacent jackal in default fixture changes east movement behavior, regression tests must use no-monster fixture helper or set player/monster positions explicitly.

## 6. Audit Commands

품질 게이트:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

필터형 targeted tests:

```bash
cargo test entity
cargo test combat
cargo test death
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

- `inventory`, `item`, `shop`, `pray`, `spell`, `zap`, `throw`, `ui`, `save` 구현 파일 없음.
- `domain/entity`, `domain/player`, `domain/monster`, `domain/combat`, `systems/combat`, `systems/death`는 허용.

## 7. 실패 시 수정 루프

1. 실패한 테스트 이름과 명령을 기록한다.
2. 원인을 Phase 3 범위 안에서만 수정한다.
3. 수정이 item/inventory/monster AI/TUI/save를 요구하면 중단하고 PRD 갱신 필요 여부를 보고한다.
4. `cargo fmt --check`, `cargo clippy`, `cargo test`, targeted tests를 다시 실행한다.
5. snapshot hash 기준이 바뀌면 `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, `CHANGELOG.md`, 필요 시 `DESIGN_DECISIONS.md`를 동기화한다.

## 8. 완료 증거 형식

Ralph 완료 보고는 다음 형식을 포함해야 한다.

```text
변경 파일:
- ...

검증:
- cargo fmt --check: pass
- cargo clippy --all-targets -- -D warnings: pass
- cargo test: pass
- cargo test entity: pass
- cargo test combat: pass
- cargo test death: pass
- cargo test map/movement/doors/vision/observation: pass
- cargo run --bin aihack-headless -- --seed 42 --turns 100: pass, final_hash=<값>
- rg legacy direct refs: pass
- scope creep audit: pass

남은 리스크:
- ...
```
