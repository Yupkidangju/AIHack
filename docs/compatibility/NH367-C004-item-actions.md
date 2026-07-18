# NH367-C004 아이템 행동

```yaml
id: NH367-C004
title: pickup, wield, quaff의 inventory와 item state
status: Implemented
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: src/pickup.c:pickup@488; src/wield.c:dowield@259; src/potion.c:dodrink@488,peffects@590
  reference_seen: false
provenance_status: Reviewed
approval:
  approval_reviewer: ""
  approval_reviewed_at: ""
  license_id: pending
  license_scope: pending
  notice_required: pending
  modification_notice_required: pending
  evidence: ""
preconditions:
  seed: 42
  level: main:1
  player: { position: [8, 5], hp: 5 }
  world: { ground_item: healing_potion, inventory_weapon: dagger, monsters: cleared }
commands: [Pickup, Wield(dagger), Quaff(healing_potion)]
expected:
  accepted_turns: 3
  events: [ItemPickedUp, ItemEquipped, ItemConsumed, EntityHealed]
  state: { potion: consumed, dagger: equipped_melee, hp: increased }
  hash_fields: [turn, inventory, entity_item_locations, equipped_melee, player_hp, rng_state]
implementation:
  task: R7-2B
  module: crates/aihack-runtime/src/systems/items.rs
test:
  file: tests/nethack_367_compat.rs
  function: nh367_c004_pickup_wield_and_quaff_update_owned_item_state
  command: cargo test -p aihack --locked --test nethack_367_compat
review:
  reviewer: Codex engineering trace
  reviewed_at: 2026-07-18
```

이 record는 행동 순서와 소유 상태만 추적한다. legacy 데이터는 provenance 위험 비교에만 사용했고 값이나 문자열을 scenario 구현에 복사하지 않았다.
