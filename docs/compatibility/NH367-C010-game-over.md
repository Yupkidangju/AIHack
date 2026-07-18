# NH367-C010 사망과 GameOver

```yaml
id: NH367-C010
title: death cause와 GameOver final state
status: Implemented
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: src/end.c:done_in_by@415,done@1099
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
  player: { hp: 0 }
  world: { attacker: entity_3 }
commands: [ResolveDeath]
expected:
  accepted_turns: 0
  events: [EntityDied(player,Combat(entity_3))]
  state: { run_state: GameOver, cause: Combat(entity_3), final_score: 0 }
  hash_fields: [run_state, player_alive, death_cause, score]
implementation:
  task: R7-2E
  module: crates/aihack-runtime/src/systems/death.rs; crates/aihack-core/src/death.rs
test:
  file: tests/nethack_367_compat.rs
  function: nh367_c010_player_death_records_cause_and_game_over_state
  command: cargo test -p aihack --locked --test nethack_367_compat
review:
  reviewer: Codex engineering trace
  reviewed_at: 2026-07-18
```

사망 원인은 typed state와 event로 보존한다. NetHack의 종료 화면 문구나 scoring 전체를 복제하지 않는다.
