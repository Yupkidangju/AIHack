# NH367-C003 Bump attack

```yaml
id: NH367-C003
title: bump attack의 전투 event와 위치 보존
status: Implemented
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: src/uhitm.c:attack@329,hmon@649; src/hack.c:domove@1352
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
  world: { east_entity: jackal }
commands: [Move(East)]
expected:
  accepted_turns: 1
  events: [AttackResolved(attacker,defender,hit,damage), EntityDied(if_lethal)]
  state: { player_position: [5, 5], defender_hp: hp_before-damage, defender_alive: inverse_of_death_event }
  hash_fields: [turn, player_position, entity_hp, rng_state]
implementation:
  task: R7-2B
  module: crates/aihack-runtime/src/systems/combat.rs
test:
  file: tests/nethack_367_compat.rs
  function: nh367_c003_bump_attack_emits_combat_without_player_movement
  command: cargo test -p aihack --locked --test nethack_367_compat
review:
  reviewer: Codex engineering trace
  reviewed_at: 2026-07-18
```

적이 있는 방향의 이동 의도는 이동 대신 전투로 해석되며 플레이어 위치는 유지된다. 이 record의 연결 test가 hit/damage, defender HP, lethal death event와 RNG draw 증가를 직접 검증한다.
