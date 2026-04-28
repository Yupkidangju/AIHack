# Test Spec: AIHack Phase 4 Items and Inventory

문서 상태: approved-test-spec
작성일: 2026-04-28
대상 PRD: `.omx/plans/prd-phase4-items-inventory.md`
범위: Phase 4 item/inventory/equipment/consumable 검증만 해당

## 1. 테스트 목표

Phase 4 구현이 다음 성질을 만족하는지 검증한다.

1. Item entity가 `EntityId`와 stable snapshot 계약에 포함된다.
2. 시작 inventory는 dagger letter `a`, food ration letter `b`를 안정적으로 가진다.
3. Map potion `(8,5)` pickup은 letter `c`를 할당한다.
4. Pickup/wield/quaff는 accepted/rejected와 turn advance 계약을 지킨다.
5. Dagger wield는 player combat profile에 반영된다.
6. Healing potion은 `1d8+4`로 deterministic하게 회복하고 `max_hp`를 넘지 않는다.
7. Consumed potion은 inventory/legal_actions/snapshot에서 일관된 consumed state를 가진다.
8. Phase 2 movement/doors/vision과 Phase 3 combat/death regression은 계속 통과한다.
9. drop/read/zap/throw/save/load/TUI/monster AI scope creep이 없다.

## 2. 테스트 매트릭스

| ID | 종류 | 대상 | 검증 내용 |
| --- | --- | --- | --- |
| T1 | unit | item factory | dagger/food/potion 실데이터 |
| T2 | unit | entity store | item entity id와 location |
| T3 | unit | inventory | starting letters `a`, `b` |
| T4 | unit | inventory | next letter assignment `c` |
| T5 | integration | pickup | potion pickup event/order/turn |
| T6 | integration | pickup reject | empty tile pickup rejected/no turn |
| T7 | integration | observation | inventory와 legal actions 노출 |
| T8 | integration | wield | dagger equipped melee |
| T9 | integration | wield reject | non-weapon rejected/no turn |
| T10 | integration | combat | equipped dagger attack profile 사용 |
| T11 | integration | quaff | heal `1d8+4`, consumed state |
| T12 | integration | quaff clamp | hp <= max_hp |
| T13 | integration | quaff reject | non-potion/not-owned rejected/no turn |
| T14 | integration | snapshot | item/inventory/equipment state affects hash |
| T15 | integration | serde | inventory roundtrip preserves letters/equipped |
| T16 | regression | Phase 2/3 | movement/doors/vision/combat/death tests pass |
| T17 | audit | dependency boundary | legacy direct reference 없음 |
| T18 | audit | scope boundary | drop/read/zap/throw/save/TUI/AI 없음 |
| T19 | quality | cargo | fmt/clippy/test 통과 |

## 3. Unit Test 상세

### T1: item factories match spec data

Expected:

```text
dagger: class=Weapon, glyph=')', weight=10, slot=Melee, hit_bonus=1, damage=1d4
food_ration: class=Food, glyph='%', weight=20, nutrition=800
potion_healing: class=Potion, glyph='!', weight=20, effect=Heal(1d8+4)
```

### T2: item entity id and location

Given:

```rust
let world = GameWorld::fixture_phase4();
```

Expected:

```text
EntityId(4) = Item(PotionHealing), location=OnMap(Pos { x: 8, y: 5 })
EntityId(5) = Item(Dagger), location=Inventory { owner: EntityId(1) }
EntityId(6) = Item(FoodRation), location=Inventory { owner: EntityId(1) }
```

### T3: starting inventory letters

Expected:

```text
inventory.owner = EntityId(1)
entries[0] = { item: EntityId(5), letter: 'a' }
entries[1] = { item: EntityId(6), letter: 'b' }
equipped_melee = None
item_payload(EntityId(5)).assigned_letter = Some('a')
item_payload(EntityId(6)).assigned_letter = Some('b')
next_letter = 'c'
```

### T4: next letter assignment

When potion is picked up:

Expected:

```text
potion letter = 'c'
next_letter = 'd'
letters remain ['a', 'b', 'c']
```

## 4. Integration Test 상세

### T5: pickup potion event/order/turn

Setup:

- player at `(8,5)`
- potion healing entity on `(8,5)`

When:

```rust
session.submit(CommandIntent::Pickup)
```

Expected:

```text
accepted = true
turn_advanced = true
events[0] = TurnStarted { turn: 1 }
events[1] = ItemPickedUp { entity: player_id, item: potion_id, letter: 'c' }
potion location = Inventory { owner: player_id }
inventory contains potion letter 'c'
```

### T6: pickup without item rejected

Setup:

- player at `(5,5)` and no item at `(5,5)`

Expected:

```text
accepted = false
turn_advanced = false
turn unchanged
snapshot_hash unchanged
CommandRejected reason contains "no item"
```

### T7: observation inventory and legal actions

Expected at run start:

```text
Observation.inventory includes dagger 'a' not equipped, food ration 'b'
Observation.legal_actions includes ShowInventory
Observation.legal_actions includes ShowInventory and Wield { item: dagger_id }
Observation.legal_actions does not include Quaff { item: food_ration_id }
```

After potion pickup:

```text
Observation.inventory includes potion 'c'
Observation.legal_actions includes Quaff { item: potion_id }
```


### T7.5: show inventory is no-turn and eventless

When:

```rust
session.submit(CommandIntent::ShowInventory)
```

Expected:

```text
accepted = true
turn_advanced = false
events = []
snapshot_hash unchanged
```

### T8: wield dagger

Given:

```text
dagger is in inventory letter a and inventory.equipped_melee=None
```

When:

```rust
session.submit(CommandIntent::Wield { item: dagger_id })
```

Expected:

```text
accepted = true
turn_advanced = true
equipped_melee = dagger_id
events = [TurnStarted, ItemEquipped { entity: player_id, item: dagger_id, slot: Melee }]
```

Already wielded policy:

```text
second Wield { dagger_id } returns accepted=true, turn_advanced=false, events=[]
equipped_melee remains dagger_id
```

### T9: wield non-weapon rejected

When:

```rust
session.submit(CommandIntent::Wield { item: food_ration_id })
```

Expected:

```text
accepted = false
turn_advanced = false
equipped_melee remains dagger_id
```

### T10: equipped dagger drives combat profile

Setup:

- run `Wield { dagger_id }` first
- player bump attacks jackal

Expected:

```text
AttackResolved.attack_roll includes player hit_bonus 2 + dagger hit_bonus 1
Damage on hit uses 1d4
```

If combat event does not expose weapon id, test may compare deterministic damage range and attack_roll value. Optional event field `weapon: Option<EntityId>` is allowed and recommended if it does not widen scope.

### T11: quaff healing potion

Setup:

- player hp reduced to `5 / 16`
- potion healing in inventory letter `c`
- seed fixed at 42

When:

```rust
session.submit(CommandIntent::Quaff { item: potion_id })
```

Expected:

```text
accepted = true
turn_advanced = true
heal_amount in 5..=12
player hp = min(16, 5 + heal_amount)
repeat the same seed + same setup + same command sequence twice and assert equal heal_amount/hp_after
events[0] = TurnStarted
events[1] = ItemConsumed { entity: player_id, item: potion_id }
events[2] = EntityHealed { entity: player_id, amount: heal_amount, hp_after }
potion location = Consumed
potion assigned_letter remains Some('c')
inventory no longer contains potion
```

### T12: healing clamps to max hp

Setup:

- player hp = 15 / 16
- potion healing in inventory

Expected:

```text
hp_after = 16
EntityHealed.amount = effective healing amount, not raw rolled amount
```

정책: `EntityHealed.amount`는 실제로 증가한 effective healing amount다. raw rolled amount는 Phase 4 event에 넣지 않는다.

### T13: invalid quaff rejected

Cases:

```text
Quaff(food_ration) rejected
Quaff(map_potion_not_owned) rejected
Quaff(consumed_potion) rejected
Quaff(EntityId(0)) rejected
```

Expected each:

```text
accepted = false
turn_advanced = false
inventory unchanged
snapshot_hash unchanged
```

## 5. Snapshot / Serialization Tests

### T14: snapshot hash changes on inventory state

Cases:

- before vs after pickup
- before vs after wield different weapon if test fixture adds second weapon
- before vs after quaff consumed potion

Expected:

```text
hash_before != hash_after
same seed + same command sequence produces same final hash
```

### T15: inventory roundtrip preserves letters

Phase 4 does not implement full file save/load. Instead validate serialization-ready state:

```rust
let json = serde_json::to_string(&session.snapshot()).unwrap();
let decoded: GameSnapshot = serde_json::from_str(&json).unwrap();
```

Expected:

```text
decoded inventory letters == original inventory letters
decoded item assigned_letter values == original assigned_letter values
decoded equipped_melee == original equipped_melee
```

If `GameWorld` is made serializable, an additional `GameWorld` roundtrip test is allowed.

## 6. Regression Tests

Required commands:

```bash
cargo test --test movement
cargo test --test doors
cargo test --test vision
cargo test --test combat
cargo test --test death
cargo test --test items
cargo test --test inventory
cargo test
```

Regression expectations:

- Phase 2 movement/door/vision semantics remain unchanged except legal_actions may include item commands.
- Phase 3 bump attack/death semantics remain unchanged except player attack profile now comes from equipped dagger.
- Dead monster still does not block movement.
- Consumed item does not block movement or appear in pickup target list.

## 7. Audit / Scope Tests

### T17: legacy boundary

Command:

```bash
rg "legacy_nethack_port_reference" src Cargo.toml tests
```

Expected:

```text
no matches
```

### T18: scope boundary

Manual/rg audit:

```bash
rg "Drop|Read|Zap|Throw|Descend|Ascend|save/load|ratatui|crossterm|monster AI|pathfind" src
rg "rand::|thread_rng|random" src
```

Allowed matches:

- `CommandIntent` comments or future spec docs are not allowed in `src` unless the variant is intentionally implemented in Phase 4. Phase 4 implementation should not add `Drop`, `Read`, `Zap`, `Throw`, `Descend`, `Ascend` variants.
- Test names may mention rejected non-scope only if no implementation exists.
- `rand::` imports are allowed only in `src/core/rng.rs`; item healing must use `GameRng`.

## 8. Quality Gate

Required final command set:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --test items
cargo test --test inventory
cargo run --bin aihack-headless -- --seed 42 --turns 100
rg "legacy_nethack_port_reference" src Cargo.toml tests
git diff --check -- src tests Cargo.toml Cargo.lock README.md spec.md implementation_summary.md DESIGN_DECISIONS.md BUILD_GUIDE.md CHANGELOG.md audit_roadmap.md designs.md
```

Pass criteria:

- All commands exit 0.
- Headless final hash is deterministic across two identical runs.
- New final hash is recorded in docs after implementation.
- No scope creep files/modules are introduced.

## 9. Failure Triage

1. If Phase 2/3 regression fails, stop item work and restore previous movement/combat semantics first.
2. If inventory letter tests fail, do not patch tests around it; fix assignment policy.
3. If quaff healing is nondeterministic, verify all dice use `GameRng` and no direct `rand` use exists.
4. If snapshot hash does not change after item state change, add missing item/inventory field to `GameSnapshot` rather than changing hash algorithm.
5. If implementing this spec requires save/load file I/O, stop and revise PRD because that is Phase 9 scope.
