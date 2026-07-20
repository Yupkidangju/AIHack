# NH367-C006 탐색

```yaml
id: NH367-C006
title: hidden door와 trap의 search reveal
status: Implemented
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: src/detect.c:findit@1574,dosearch@1784; src/trap.c
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
  player: { positions: [[11,5], [15,5]] }
  world: { hidden_door: [12,5], hidden_pit: [16,5], monsters: cleared }
commands: [Search, Search]
expected:
  accepted_turns: 2
  events: [TileRevealed(hidden_door), TileRevealed(hidden_pit)]
  state: { hidden_door: closed_door, hidden_pit: pit }
  hash_fields: [turn, current_map_tiles]
implementation:
  task: R7-2C
  module: crates/aihack-runtime/src/systems/traps.rs; crates/aihack-core/src/doors.rs
test:
  file: tests/nethack_367_compat.rs
  function: nh367_c006_search_reveals_hidden_door_and_trap
  command: cargo test -p aihack --locked --test nethack_367_compat
review:
  reviewer: Codex engineering trace
  reviewed_at: 2026-07-18
```

각 precondition은 별도 결정적 session에서 실행한다. reveal event와 map state만 관찰하며 원본 확률식은 범위에 포함하지 않는다.
