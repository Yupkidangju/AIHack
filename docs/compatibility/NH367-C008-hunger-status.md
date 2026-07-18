# NH367-C008 허기 상태

```yaml
id: NH367-C008
title: nutrition threshold의 hunger state 전이
status: Implemented
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: src/eat.c:gethungry@2790,newuhs@2928
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
  level: none
  player: { nutrition_boundaries: [-1, 0, 1, 50, 51, 150, 151, 1000, 1001] }
  world: { pure_status_projection: true }
commands: [ProjectHungerState]
expected:
  accepted_turns: 0
  events: []
  state: { le_0: Fainting, 1_to_50: Weak, 51_to_150: Hungry, 151_to_1000: NotHungry, gt_1000: Satiated }
  hash_fields: [nutrition]
implementation:
  task: R7-2D
  module: crates/aihack-core/src/domain/status.rs
test:
  file: tests/nethack_367_compat.rs
  function: nh367_c008_hunger_thresholds_map_to_stable_status_states
  command: cargo test -p aihack --locked --test nethack_367_compat
review:
  reviewer: Codex engineering trace
  reviewed_at: 2026-07-18
```

AIHack v0.3 범위는 `newuhs`의 다섯 threshold 상태를 경계값으로 고정한다. FAINTED/STARVED 전이, 전체 nutrition 소모 계산과 메시지 문구는 포함하지 않는다.
