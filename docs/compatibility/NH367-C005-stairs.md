# NH367-C005 계단 왕복

```yaml
id: NH367-C005
title: stairs 왕복의 level과 landing state
status: Implemented
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: src/do.c:dodown@940,doup@1095; src/dungeon.c
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
  player: { position: level1_stairs_down }
  world: { paired_stairs: main_1_to_main_2, monsters: cleared }
commands: [Descend, Ascend]
expected:
  accepted_turns: 2
  events: [LevelChanged(main:1,main:2), LevelChanged(main:2,main:1)]
  state: { level: main:1, position: level1_stairs_down }
  hash_fields: [turn, current_level, player_location, level_maps]
implementation:
  task: R7-2C
  module: crates/aihack-runtime/src/systems/stairs.rs
test:
  file: tests/nethack_367_compat.rs
  function: nh367_c005_stairs_roundtrip_preserves_level_landing_contract
  command: cargo test -p aihack --locked --test nethack_367_compat
review:
  reviewer: Codex engineering trace
  reviewed_at: 2026-07-18
```

하강과 상승은 쌍을 이루는 landing 위치를 사용하며 current level과 actor location이 함께 바뀐다.
