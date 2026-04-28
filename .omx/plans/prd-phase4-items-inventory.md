# PRD: AIHack Phase 4 Items and Inventory

문서 상태: approved-plan
작성일: 2026-04-28
기준 문서: `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, `DESIGN_DECISIONS.md`, `designs.md`
컨텍스트 스냅샷: `.omx/context/phase4-items-inventory-20260428T130145Z.md`
이전 단계: Phase 3 Combat and Death 완료
범위: Phase 4만 해당. 이 문서는 계획 산출물이며 구현은 별도 실행 모드에서 수행한다.

## 1. 목표

Phase 3의 `GameWorld { map, EntityStore, player_id }` 위에 최소 item/inventory/equipment 계약을 추가한다. 플레이어는 fixture potion을 줍고, inventory letter가 안정적으로 할당되며, dagger를 wield하고, potion of healing을 quaff하여 HP를 회복하고 potion item을 소비할 수 있어야 한다.

Phase 4 완료 상태는 다음 질문에 `예`로 답할 수 있어야 한다.

- `EntityId` 기반 item entity가 player/monster와 같은 `GameSession` 단일 상태 원천 아래 존재하는가?
- `Pickup` 명령이 player 위치의 item을 inventory로 이동시키고 stable inventory letter를 할당하는가?
- `ShowInventory` 또는 `Observation.inventory`가 inventory item과 letter/equipped/consumed 상태를 typed data로 노출하는가?
- `Wield { item }`가 inventory 안의 dagger를 melee equipment로 지정하고 combat damage/hit profile에 반영하는가?
- `Quaff { item }`가 potion of healing을 소비하고 `1d8+4`만큼 player HP를 `max_hp` 이하로 회복하는가?
- `ItemPickedUp`, `ItemEquipped`, `ItemConsumed`, `EntityHealed` event가 결정적 순서로 생성되는가? `ShowInventory`는 상태 변경과 event 없이 accepted/no-turn으로 처리되는가?
- Phase 5 level/stairs, Phase 6 monster AI, TUI, file save/load를 구현하지 않았는가?

## 2. RALPLAN-DR 요약

### 2.1 원칙

1. Phase 4는 item/inventory/equipment 최소 루프만 추가한다.
2. Item은 `EntityId`를 가지며 replay/hash 입력에 포함한다.
3. Inventory letter는 획득 시 고정되고 drop/reorder가 없는 Phase 4에서는 재사용하지 않는다.
4. Combat는 player 내장 dagger profile에서 equipped item profile로 마이그레이션하되 monster combat은 그대로 유지한다.
5. Healing potion 난수는 반드시 `GameRng`를 통해 `1d8+4`로 계산한다.
6. 실제 파일 save/load는 Phase 9 범위로 유예하고, Phase 4는 serde/snapshot 안정성까지만 검증한다.

### 2.2 결정 동인

| 순위 | 동인 | 의미 |
| --- | --- | --- |
| 1 | Deterministic inventory | pickup/wield/quaff 결과와 letter가 seed/command sequence로 재현되어야 한다. |
| 2 | Entity continuity | item도 `EntityId`/snapshot/event 계약에 포함되어 후속 Phase와 충돌하지 않아야 한다. |
| 3 | Scope containment | drop/read/zap/throw/save/load/TUI/AI를 끌어오지 않는다. |

### 2.3 대안 검토

| 옵션 | 설명 | 장점 | 단점 | 판정 |
| --- | --- | --- | --- | --- |
| A. 별도 `ItemStore` + inventory item ids | actor entity store와 item store 분리 | Actor 구조 변경이 작다 | entity id 공간이 분리되어 `ItemPickedUp { item: EntityId }` 계약과 충돌 가능 | 기각 |
| B. `EntityKind::Item(ItemKind)`로 기존 `EntityStore` 확장 | player/monster/item이 같은 id 공간 공유 | event/snapshot/fixture 확장 자연스러움 | actor stats가 item에는 불필요하므로 entity 구조 리팩터 필요 | 선택 |
| C. Inventory를 문자열/enum 목록만으로 구현 | 구현량 최소 | map 위 item, pickup event, future drop/stairs와 연결 어려움 | 기각 |
| D. Full equipment/inventory/save system 구현 | 후속 기능 확장 완성도 높음 | Phase 4 범위를 크게 초과 | 기각 |

선택: **옵션 B. `EntityStore`에 Item entity 통합 + 별도 `Inventory` 상태 도입**.

## 3. 포함 범위

### 3.1 신규/변경 파일

| 파일 | 책임 |
| --- | --- |
| `src/domain/item.rs` | `ItemKind`, `ItemClass`, `EquipmentSlot`, `ConsumableEffect`, `ItemData`, fixture factory |
| `src/domain/inventory.rs` | `Inventory`, `InventoryEntry`, `InventoryLetter`, letter assignment, equipped melee slot |
| `src/domain/entity.rs` | `EntityKind::Item(ItemKind)`, actor/item payload 분리 또는 item data 연결 |
| `src/core/action.rs` | `Pickup`, `ShowInventory`, `Wield { item }`, `Quaff { item }` 추가 |
| `src/core/event.rs` | `ItemPickedUp`, `ItemEquipped`, `ItemConsumed`, `EntityHealed` 추가 |
| `src/core/world.rs` | fixture item spawn, player inventory 보유 또는 inventory accessor |
| `src/core/session.rs` | pickup/show inventory/wield/quaff routing |
| `src/core/snapshot.rs` | item position/inventory/equipped/consumed state hash 포함 |
| `src/core/observation.rs` | 최소 `inventory: Vec<ItemObservation>`와 item 관련 legal actions 추가 |
| `src/systems/items.rs` | pickup/wield/quaff/heal resolve |
| `src/systems/combat.rs` | player equipped melee item profile 사용 |
| `src/systems/mod.rs` | `items` module export |
| `tests/items.rs` | item fixture, pickup, quaff, event 검증 |
| `tests/inventory.rs` | letter stability, wield, snapshot/serde 검증 |

### 3.2 핵심 타입 계약

```rust
pub enum EntityKind {
    Player,
    Monster(MonsterKind),
    Item(ItemKind),
}

pub enum ItemKind {
    Dagger,
    FoodRation,
    PotionHealing,
}

pub enum ItemClass {
    Weapon,
    Food,
    Potion,
}

pub enum EquipmentSlot {
    Melee,
}

pub enum ConsumableEffect {
    Heal { dice: i16, sides: i16, bonus: i16 },
}

pub struct ItemData {
    pub kind: ItemKind,
    pub class: ItemClass,
    pub glyph: char,
    pub weight: i16,
    pub attack_profile: Option<AttackProfile>,
    pub consumable_effect: Option<ConsumableEffect>,
}

pub struct InventoryLetter(pub char);

pub struct InventoryEntry {
    pub item: EntityId,
    pub letter: InventoryLetter,
}

pub struct Inventory {
    pub owner: EntityId,
    pub entries: Vec<InventoryEntry>,
    pub equipped_melee: Option<EntityId>,
    pub next_letter_index: u8,
}

pub enum EntityLocation {
    OnMap(Pos),
    Inventory { owner: EntityId },
    Consumed,
}
```

계약:

- `InventoryLetter`는 `a..=z` 순서로 할당한다.
- Phase 4는 drop이 없으므로 letter 재사용을 하지 않는다.
- `Inventory.entries` 순서는 획득 순서를 보존한다.
- Equipment의 단일 상태 원천은 `Inventory.equipped_melee: Option<EntityId>`이다. `InventoryEntry`에는 `equipped_slot`을 두지 않는다.
- Item entity는 위치 상태를 반드시 가진다. 고정 위치 표현:
  - `EntityLocation::OnMap(Pos)`
  - `EntityLocation::Inventory { owner: EntityId }`
  - `EntityLocation::Consumed`
- 살아있는 actor occupancy와 item occupancy는 분리한다. Item은 movement blocker가 아니다.
- `EntityKind::Item`은 combat `ActorStats`를 갖지 않는다. 구현자는 `EntityPayload::Actor { ... } | EntityPayload::Item { ... }` 형태로 리팩터링한다. `Option<ActorStats>`/`Option<ItemData>` 조합형은 invalid state가 생기므로 Phase 4에서 금지한다.
- Store query는 actor/item을 분리한다. 필수 helper: `alive_actor_at(pos)`, `alive_hostile_at(pos)`, `item_at(pos)`, `items_at(pos)`, `inventory_items(owner)`.
- Movement blocker는 `alive_actor_at`만 사용한다. Item은 movement blocker가 아니다.
- Snapshot은 actor entity에만 hp/alive를 기록하고 item entity에는 kind/location/assigned_letter/consumed를 기록한다. Equipped state는 `Inventory.equipped_melee`에서만 기록한다.

## 4. Item 실데이터

`spec.md`의 v0.1 값을 그대로 사용한다.

| kind | class | glyph | weight | slot/effect | combat/effect 값 |
| --- | --- | ---: | ---: | --- | --- |
| `Dagger` | Weapon | `)` | 10 | `Melee` | hit_bonus `1`, damage `1d4` |
| `FoodRation` | Food | `%` | 20 | 없음 | nutrition `800`, Phase 4 사용 명령 없음 |
| `PotionHealing` | Potion | `!` | 20 | quaff | heal `1d8+4` |

Phase 4 command support:

- `Pickup`: player 위치의 pickup 가능한 item 중 deterministic 정렬 기준 `(entity_id)`이 가장 작은 첫 item만 줍는다. Phase 4 fixture는 같은 tile 다중 item을 만들지 않지만, helper는 다중 item이 있어도 가장 작은 `EntityId` 하나만 선택해야 한다.
- `Wield { item }`: inventory 안의 weapon만 허용한다.
- `Quaff { item }`: inventory 안의 potion만 허용한다. 성공 시 inventory entry를 제거하고 item entity는 `EntityLocation::Consumed` tombstone으로 남긴다. item payload의 `assigned_letter`는 유지한다.
- `ShowInventory`: 상태 변경 없이 `accepted=true`, `turn_advanced=false`, `events=[]`로 처리한다. `InventoryShown` event는 만들지 않는다. UI/AI는 `Observation.inventory`를 읽어 표시한다.

## 5. Fixture 데이터

Phase 4는 Phase 3 fixture를 유지하고 item만 추가한다.

```text
player = { id = EntityId(1), kind = Player, pos = (5,5), hp = 16 }
jackal = { id = EntityId(2), kind = Monster(Jackal), pos = (6,5), hp = 4 }
goblin = { id = EntityId(3), kind = Monster(Goblin), pos = (20,12), hp = 6 }
potion_healing = { id = EntityId(4), kind = Item(PotionHealing), location = OnMap(8,5), assigned_letter = None }
dagger = { id = EntityId(5), kind = Item(Dagger), location = Inventory(player), assigned_letter = 'a' }
food_ration = { id = EntityId(6), kind = Item(FoodRation), location = Inventory(player), assigned_letter = 'b' }
```

주의:

- Phase 3에서는 player 기본 공격이 내장 dagger였다. Phase 4부터 내장 dagger profile은 제거하고 player 시작 inventory의 dagger를 `Wield { item }`로 장착해야 dagger `AttackProfile`이 전투에 반영된다. 장착 전 fallback은 unarmed `hit_bonus=0`, damage `1d2`로 문서화한다.
- 시작 inventory에 dagger/food ration이 있으므로 `Pickup` 첫 테스트의 potion letter는 `c`가 된다. 시작 inventory를 비우거나 dagger를 map에 두는 대안은 Phase 4에서 금지한다.
- Potion `(8,5)`에 도달하려면 jackal `(6,5)`가 막고 있으므로 테스트는 몬스터를 제거하거나 player 위치를 potion tile로 직접 세팅하는 helper를 사용한다. Phase 4는 monster AI/전투 밸런스를 검증하지 않는다.

## 6. 제외 범위 / 비목표

- Drop command와 map에 item 다시 내려놓기.
- Wear armor, read scroll, zap wand, throw item.
- Food eating/nutrition/hunger.
- Item identification, stack, quantity, BUC, enchantment.
- Shop/price/ownership.
- Full save/load 파일 I/O. Phase 4는 serde/snapshot roundtrip까지만 검증한다.
- TUI inventory screen, mouse click, drag/drop.
- Monster pickup/use item.
- Corpse/drop generation on death.
- Multi-level item persistence. Phase 5+ 범위.

## 7. 구현 순서

### Step 1: Entity payload migration

- `EntityKind::Item(ItemKind)`를 추가한다.
- Actor와 Item이 서로 잘못된 payload를 갖지 않도록 entity 구조를 정리한다.
- 기존 Phase 3 combat/death tests가 계속 통과하도록 `actor()`/`item()` helper를 둔다.

### Step 2: Item domain factory

- `src/domain/item.rs`에 dagger/food ration/potion healing 실데이터를 정의한다.
- `AttackProfile`은 dagger에서 재사용한다.
- `ConsumableEffect::Heal { dice: 1, sides: 8, bonus: 4 }`를 고정한다.

### Step 3: Inventory domain

- `Inventory`와 `InventoryEntry`를 추가한다.
- `assign_next_letter()`는 `a..=z`를 순서대로 할당하고, 27번째 아이템은 rejected error로 처리한다.
- `equipped_melee`는 최대 1개만 허용하며, 현재 장착 상태의 유일한 source of truth다.

### Step 4: Fixture migration

- Phase 4 world fixture에 start dagger/food ration inventory와 map potion `(8,5)`를 추가한다.
- Existing Phase 2/3 tests는 필요 시 `clear_items()` 또는 synthetic helper를 사용한다.

### Step 5: Pickup command

- `CommandIntent::Pickup`, `ShowInventory`, `Wield { item }`, `Quaff { item }` 추가.
- player 위치에 pickup 가능한 item이 없으면 rejected + no turn advance.
- 성공 시 item location을 inventory로 옮기고 `assigned_letter`를 할당하며 accepted + turn advance.
- event order: `TurnStarted`, `ItemPickedUp`.

### Step 6: Wield command

- `CommandIntent::Wield { item }` 추가.
- item이 owner inventory 안의 weapon이 아니면 rejected + no turn advance.
- item이 현재 장착되어 있지 않은 weapon이면 equipped melee를 갱신하고 accepted + turn advance.
- 이미 장착된 같은 item을 다시 wield하면 idempotent accepted + no turn advance + events=[]로 처리한다.
- 신규 장착 event order: `TurnStarted`, `ItemEquipped`.

### Step 7: Combat integration

- `systems/combat.rs`가 player 공격 시 equipped melee item의 `AttackProfile`을 사용한다.
- Equipped melee가 없으면 Phase 4 fallback unarmed profile `hit_bonus=0`, damage `1d2`를 사용한다.
- Combat regression 중 dagger 기반 공격은 테스트에서 먼저 `Wield { dagger_id }`를 실행한 뒤 검증한다.
- Phase 3 jackal/goblin combat tests는 hash 변경 외 의미가 유지되어야 한다.

### Step 8: Quaff command

- `CommandIntent::Quaff { item }` 추가.
- potion이 아니거나 inventory 밖이면 rejected + no turn advance.
- 성공 시 `heal = roll(1d8) + 4`, `hp = min(max_hp, hp + heal)`.
- potion entity location은 `Consumed`로 변경하고 inventory entry에서 제거한다. 단, item payload의 `assigned_letter`는 유지하여 snapshot/serde roundtrip에서 consumed item의 과거 letter를 추적할 수 있게 한다.
- event order: `TurnStarted`, `ItemConsumed`, `EntityHealed`.

### Step 9: Observation/Snapshot

- `Observation.inventory: Vec<ItemObservation>` 추가.
- `legal_actions`에 `Pickup`, `ShowInventory`, wield/quaff 후보를 추가한다.
- Snapshot hash에 item kind/location/assigned_letter/consumed state와 `Inventory.equipped_melee`를 포함한다.

### Step 0: 문서 충돌 선해결

- 코드 구현 전에 `audit_roadmap.md`의 Phase 4 완료 기준을 “save/load 후 inventory letter 유지”에서 “serde/snapshot roundtrip 후 inventory letter 유지”로 갱신한다. 이 PRD가 Phase 4 범위에서는 기존 audit 문구보다 우선한다. 실제 file save/load는 Phase 9로 유예한다.

### Step 10: 문서 동기화

- 구현 완료 후 `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, `DESIGN_DECISIONS.md`, `CHANGELOG.md`, `README.md`, `designs.md`, `BUILD_GUIDE.md`를 갱신한다. 특히 `audit_roadmap.md`의 Phase 4 완료 기준은 기존 “save/load 후 inventory letter 유지”에서 “serde/snapshot roundtrip 후 inventory letter 유지”로 바꿔 Phase 9 file save/load와 충돌하지 않게 한다.
- Phase 4 기준 `seed=42 turns=100` hash를 새로 기록한다.

## 8. 수용 기준

| ID | 기준 | 검증 |
| --- | --- | --- |
| AC-1 | `EntityKind::Item(ItemKind)`와 item fixture 생성 | `cargo test --test items item_factories_match_spec_data` |
| AC-2 | 시작 inventory dagger=`a`, food ration=`b` 안정 할당 | `cargo test --test inventory starting_inventory_letters_are_stable` |
| AC-3 | map potion `(8,5)` pickup 시 inventory letter=`c` | `cargo test --test items pickup_assigns_next_letter` |
| AC-4 | pickup 성공 시 `TurnStarted -> ItemPickedUp` event | `cargo test --test items pickup_event_order_is_stable` |
| AC-5 | item 없는 tile pickup은 rejected + no turn advance | `cargo test --test items pickup_without_item_is_rejected` |
| AC-6 | `Wield { dagger }`가 melee equipment를 지정하고 재호출은 idempotent no-turn | `cargo test --test inventory wield_dagger_sets_melee_slot` |
| AC-7 | non-weapon `Wield`는 rejected | `cargo test --test inventory wield_non_weapon_is_rejected` |
| AC-8 | equipped dagger combat가 `hit_bonus=1`, `1d4`를 사용 | `cargo test --test combat equipped_dagger_drives_player_attack_profile` |
| AC-9 | `Quaff { potion }`는 `1d8+4` 회복 후 potion consumed, 같은 seed/command sequence 반복 시 같은 heal amount | `cargo test --test items quaff_healing_potion_heals_and_consumes` |
| AC-10 | healing은 `max_hp`를 초과하지 않음 | `cargo test --test items healing_clamps_to_max_hp` |
| AC-11 | consumed potion은 inventory와 legal actions에서 제거 | `cargo test --test items consumed_potion_is_not_legal_action` |
| AC-12 | snapshot hash가 item/inventory/equipment 상태 변화를 반영 | `cargo test --test inventory inventory_state_affects_snapshot_hash` |
| AC-13 | serde_json roundtrip 후 inventory letter/assigned_letter/equipped_melee 유지 | `cargo test --test inventory inventory_roundtrip_preserves_letters` |
| AC-14 | Phase 2/3 regression 통과 | `cargo test --test movement --test doors --test vision --test combat --test death` |
| AC-15 | legacy direct import 없음 | `rg "legacy_nethack_port_reference" src Cargo.toml tests` 결과 없음 |
| AC-16 | scope boundary 및 RNG boundary 준수 | `rg "Drop|Read|Zap|Throw|monster AI|ratatui|crossterm|save/load" src`; `rg "rand::|thread_rng|random" src`에서 `core/rng.rs` 외 직접 난수 없음 |
| AC-17 | 품질 게이트 통과 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |

## 9. 위험과 완화

| 위험 | 영향 | 완화 |
| --- | --- | --- |
| Entity 구조가 actor/item Option 조합으로 느슨해짐 | invalid state 발생 | enum payload 또는 constructor-only 생성 + tests로 잘못된 조합 차단 |
| 시작 dagger를 실제 item으로 바꾸며 combat hash가 바뀜 | replay 기준 변경 | Phase 4 hash를 새 기준으로 문서화하고 Phase 3 hash는 과거 기준으로 유지 |
| inventory letter와 consumed tombstone 정책 불명확 | save/replay 불안정 | `assigned_letter`는 item payload에 유지, inventory entry는 consumed 시 제거, letter never-reuse 정책 고정 |
| save/load 완료 기준이 Phase 9와 충돌 | scope creep | Phase 4는 serde roundtrip만 검증하고 파일 save/load는 Phase 9로 명시 유예 |
| legal_actions가 item id를 과도하게 노출 | AI action ambiguity | inventory 내 valid item만 후보로 생성하고 consumed/map item은 제외 |

## 10. RALPLAN ADR

결정:

Phase 4는 `EntityStore`를 item까지 확장하고, `Inventory`를 별도 domain state로 추가한다. Equipment source of truth는 `Inventory.equipped_melee` 하나로 고정한다. 시작 dagger/food ration은 player inventory에 있지만 dagger는 아직 equipped 상태가 아니다. Phase 4의 `Wield { item }`가 실제 melee slot을 설정한다. Map potion은 `(8,5)`에 놓이며 pickup 후 letter `c`를 받는다. Combat는 player 내장 dagger profile에서 equipped item profile로 이전한다.

결정 동인:

- 동일 `EntityId` 공간을 사용해야 `ItemPickedUp { item: EntityId }`와 future drop/stairs/save가 자연스럽다.
- Inventory letter는 UI와 replay 모두에 필요한 안정 계약이다.
- Full save/load와 TUI를 Phase 4에 끌어오지 않아야 계획과 검증이 작게 유지된다. `audit_roadmap.md`의 Phase 4 완료 문구는 file save/load가 아니라 serde/snapshot roundtrip으로 갱신한다.

대안:

- 별도 `ItemStore`: actor store 변경량은 작지만 id/event/snapshot 통합성이 약하다.
- Inventory-only item: pickup/drop/visible item 확장이 어렵다.
- Full inventory/equipment/save: 후속 Phase를 침범한다.

결과:

- Phase 4 구현은 `domain/entity.rs` 리팩터를 포함한다.
- Phase 3 combat tests는 equipped dagger migration 때문에 `Wield { dagger }` 선행 또는 unarmed fallback 기대값으로 조정될 수 있으며, 일부 기대 hash는 갱신될 수 있다.
- Phase 9 save/load 구현 시 Phase 4의 serde roundtrip contract를 실제 save schema로 승격한다.

## 11. 실행 핸드오프

### Ralph 권장 지시

```text
$ralph .omx/plans/prd-phase4-items-inventory.md

반드시 .omx/plans/test-spec-phase4-items-inventory.md의 검증 기준을 만족시키고,
Phase 4 완료 후 멈추세요. Phase 5로 진행하지 마세요.
```

### 사용 가능한 agent roster

| 역할 | 책임 | 권장 reasoning |
| --- | --- | --- |
| `executor` | item/inventory 구현 | medium |
| `test-engineer` | item/inventory/combat regression 테스트 보강 | medium |
| `architect` | EntityStore item 통합과 save/load 경계 검토 | high |
| `critic` | scope creep, typed contract, acceptance criteria 검토 | high |
| `verifier` | 최종 증거와 테스트 충분성 확인 | high |

### Team 사용 시 lane

- Lane A: domain/entity/item/inventory 타입과 fixture migration
- Lane B: action/event/session/items system 구현
- Lane C: tests/items.rs, tests/inventory.rs, combat regression
- Lane D: docs sync and final verification

Team verification path:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --test items
cargo test --test inventory
cargo run --bin aihack-headless -- --seed 42 --turns 100
rg "legacy_nethack_port_reference" src Cargo.toml tests
```

## 12. Consensus 상태

- Planner: 승인. Phase 4 item/inventory 목표를 가장 작은 typed contract로 분리했다.
- Architect: APPROVE. Wield idempotency, EntityPayload 구체화, save/load 문구, ShowInventory 정책을 보완했다.
- Critic: APPROVE. Equipment source, consumed letter persistence, healing determinism, stale audit handling, pickup/quaff 정책 고정이 충분하다.
