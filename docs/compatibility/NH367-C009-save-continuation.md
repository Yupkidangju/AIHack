# NH367-C009 저장 연속성

```yaml
id: NH367-C009
title: save/load 후 command와 RNG continuation
status: Implemented
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: src/save.c:dosave@78,savegamestate@279; src/restore.c:dorecover@801
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
  player: { after_commands: [Wait] }
  world: { schema_version: 1 }
commands: [Save, Load, Search, Move(East)]
expected:
  accepted_turns: 3
  events: [direct_equals_loaded]
  state: { direct_snapshot_hash: loaded_snapshot_hash }
  hash_fields: [turn, run_state, rng_state, world, event_log]
implementation:
  task: R7-2E
  module: crates/aihack-runtime/src/save.rs
test:
  file: tests/nethack_367_compat.rs
  function: nh367_c009_save_load_preserves_rng_command_continuation
  command: cargo test -p aihack --locked --test nethack_367_compat
review:
  reviewer: Codex engineering trace
  reviewed_at: 2026-07-18
```

저장 형식 자체의 NetHack 호환을 주장하지 않는다. 관찰 규칙은 저장 전후 동일 명령열의 event와 deterministic snapshot이 이어지는지다.
