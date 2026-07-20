# NH367-C002 닫힌 문

```yaml
id: NH367-C002
title: 닫힌 문의 이동 차단과 open 전이
status: Implemented
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: src/lock.c:doopen@626,doorlock@921; src/hack.c:test_move@713
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
  player: { position: [9, 5] }
  world: { east_tile: closed_door, monsters: cleared }
commands: [Move(East), Open(East)]
expected:
  accepted_turns: 1
  events: [CommandRejected, DoorChanged(Closed,Open)]
  state: { east_tile: open_door, position: [9, 5] }
  hash_fields: [turn, player_position, current_map_tiles]
implementation:
  task: R7-2A
  module: crates/aihack-runtime/src/systems/doors.rs
test:
  file: tests/nethack_367_compat.rs
  function: nh367_c002_closed_door_blocks_then_open_transitions_state
  command: cargo test -p aihack --locked --test nethack_367_compat
review:
  reviewer: Codex engineering trace
  reviewed_at: 2026-07-18
```

닫힌 문은 이동을 막고 명시적 open 명령만 문 상태를 전이시킨다. 장문 게임 메시지나 원본 조건 분기는 포함하지 않는다.
