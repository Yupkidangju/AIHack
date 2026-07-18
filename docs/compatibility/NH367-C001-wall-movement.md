# NH367-C001 벽 이동

```yaml
id: NH367-C001
title: 벽 방향 이동의 위치와 turn 정책
status: Implemented
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: src/hack.c:test_move@713,domove@1352
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
  player: { position: [1, 1] }
  world: { north_tile: wall, monsters: cleared }
commands: [Move(North)]
expected:
  accepted_turns: 0
  events: [CommandRejected]
  state: { position: [1, 1], turn: 0 }
  hash_fields: [turn, player_position, current_level]
implementation:
  task: R7-2A
  module: crates/aihack-runtime/src/systems/movement.rs
test:
  file: tests/nethack_367_compat.rs
  function: nh367_c001_wall_movement_preserves_position_turn_and_hash
  command: cargo test -p aihack --locked --test nethack_367_compat
review:
  reviewer: Codex engineering trace
  reviewed_at: 2026-07-18
```

관찰 규칙은 통과 불가능한 타일로의 이동이 위치와 turn을 바꾸지 않는다는 것이다. 원문 제어 흐름이나 메시지는 복사하지 않고 AIHack의 typed rejection과 snapshot field로 재서술한다.
