# NH367-C007 투척·완드·스크롤

```yaml
id: NH367-C007
title: throw, zap, read의 resource와 projectile stop
status: Implemented
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: src/dothrow.c:dothrow@260,throwit@1095; src/zap.c:weffects@3029,bhit@3218; src/read.c:doread@187,seffects@999
  reference_seen: true
provenance_status: Approved
approval:
  approval_reviewer: Project owner
  approval_reviewed_at: 2026-07-20
  license_id: NGPL
  license_scope: whole AIHack derivative distribution
  notice_required: true
  modification_notice_required: true
  evidence: Project owner derivative classification; AIHACK-OWNER-2026-07-20-NGPL-01; AI-assisted semantic rewrite from NetHack 3.6.7 source
preconditions:
  seed: 42
  level: main:1
  player: { position: [5, 5] }
  world: { throwable: rock, wand_charges: 3, readable: reveal_scroll, monsters: cleared }
commands: [Throw(rock,East), Zap(wand,East), Read(reveal_scroll)]
expected:
  accepted_turns: 3
  events: [ItemThrown, WandZapped(charges_after=2), ScrollRead]
  state: { rock_position: [9,5], wand_charges: 2, scroll: consumed, hidden_door_and_trap: revealed }
  hash_fields: [turn, item_locations, item_charges, current_map_tiles, rng_state]
implementation:
  task: R7-2D
  module: crates/aihack-runtime/src/systems/projectiles.rs; crates/aihack-runtime/src/systems/items.rs
test:
  file: tests/nethack_367_compat.rs
  function: nh367_c007_throw_zap_and_read_consume_bounded_resources
  command: cargo test -p aihack --locked --test nethack_367_compat
review:
  reviewer: Codex engineering trace
  reviewed_at: 2026-07-18
```

세 명령은 서로 독립된 session에서 실행해 각각 turn 1회를 소비한다. 연결 test가 rock 위치, wand charge, scroll 소비, reveal 전후 map tile과 target이 없는 경로의 RNG 불변을 직접 검증한다. 원본 아이템 명칭표나 메시지는 포함하지 않는다.
