# PRD: AIHack Phase 3 Combat and Death

문서 상태: approved-plan
작성일: 2026-04-28
기준 문서: `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, `DESIGN_DECISIONS.md`
컨텍스트 스냅샷: `.omx/context/phase3-combat-death-20260428T112628Z.md`
이전 단계: Phase 2 Map, Movement, Doors, Vision 완료
범위: Phase 3만 해당. 이 문서는 계획 산출물이며 구현은 별도 실행 모드에서 수행한다.

## 1. 목표

Phase 2의 map/movement/vision 위에 최소 entity store와 deterministic combat/death를 추가한다. 플레이어가 jackal/goblin을 bump attack으로 공격할 수 있고, 몬스터 사망은 `EntityDied` event로 기록되며, 플레이어 사망은 `RunState::GameOver`로 전환된다.

Phase 3 완료 상태는 다음 질문에 `예`로 답할 수 있어야 한다.

- `EntityId` 기반 player/monster 최소 entity store가 `GameSession` 단일 상태 원천 아래 존재하는가?
- `Move(dir)`가 빈 tile이면 이동, hostile monster tile이면 bump attack으로 분기하는가?
- hit formula와 damage formula가 `spec.md` 기준대로 deterministic하게 동작하는가?
- jackal/goblin combat와 death event가 seed 고정 테스트로 검증되는가?
- player HP가 0 이하가 되면 `RunState::GameOver`와 `EntityDied`가 생성되는가?
- Phase 4 item/inventory, Phase 6 monster AI, TUI를 구현하지 않았는가?

## 2. RALPLAN-DR 요약

### 2.1 원칙

1. Phase 3는 combat/death만 추가한다.
2. Entity store는 player와 monster 최소 데이터만 포함한다.
3. Bump attack은 movement system의 자연스러운 확장으로 구현하되, combat 판정은 `systems/combat.rs`로 분리한다.
4. 모든 난수는 `GameRng`를 통해서만 사용한다.
5. 사망 처리는 event-first로 기록하고 snapshot hash에 entity state를 포함한다.

### 2.2 결정 동인

| 순위 | 동인 | 의미 |
| --- | --- | --- |
| 1 | Deterministic combat | seed와 command sequence로 hit/damage/death를 재현할 수 있어야 한다. |
| 2 | Scope containment | item, monster AI, TUI, save/load를 끌어오지 않는다. |
| 3 | Future migration | Phase 4+ item/entity 확장과 충돌하지 않는 최소 entity store가 필요하다. |

### 2.3 대안 검토

| 옵션 | 설명 | 장점 | 단점 | 판정 |
| --- | --- | --- | --- | --- |
| A. Player/monster를 `GameWorld` 필드로 직접 보관 | `player_hp`, `monsters: Vec<Monster>`만 추가 | 작고 빠름 | `EntityId`/event 계약과 맞지 않고 Phase 4+ 확장성 부족 | 기각 |
| B. Minimal `EntityStore` 도입 | player+monster entity만 저장하고 item component는 제외 | `EntityId` 계약 충족, 후속 확장 가능 | 약간의 타입 수 증가 | 선택 |
| C. ECS/Legion 도입 | full ECS로 combat 구현 | 일반화 쉬움 | 문서상 금지, 이전 레거시 리스크 반복 | 기각 |

선택: **옵션 B. Minimal `EntityStore` 도입**.

## 3. 포함 범위

### 3.1 신규/변경 파일

| 파일 | 책임 |
| --- | --- |
| `src/domain/entity.rs` | `EntityStore`, `Entity`, `EntityKind`, `ActorStats`, `Faction`, position/hp helpers |
| `src/domain/monster.rs` | `MonsterKind`, jackal/goblin/floating_eye 기본 stat factory. Phase 3 combat는 jackal/goblin 중심 |
| `src/domain/player.rs` | player 기본 stat factory |
| `src/domain/combat.rs` | `AttackProfile`, `DamageRoll`, `DeathCause` 등 combat domain 타입 |
| `src/core/world.rs` | `entities: EntityStore`, player id 보유. 기존 `player_pos`는 제거 또는 compatibility getter로 축소 |
| `src/core/action.rs` | 기존 `Move` 유지. 별도 `Attack` command는 Phase 3 범위 밖 |
| `src/core/event.rs` | `AttackResolved`, `EntityDied` event 추가 |
| `src/core/session.rs` | `Move` submit에서 bump attack 분기와 death check 연결 |
| `src/core/snapshot.rs` | entity hp/position/alive state를 hash 입력에 포함 |
| `src/core/observation.rs` | player pos는 entity store에서 읽고, visible hostile entity 최소 정보는 선택적으로 포함 가능 |
| `src/systems/movement.rs` | occupied hostile tile이면 movement blocker 대신 combat handoff |
| `src/systems/combat.rs` | hit/damage formula, bump attack resolve |
| `src/systems/death.rs` | death event 생성, monster removal/tombstone, player GameOver 전환 |
| `tests/combat.rs` | hit/damage/bump/death deterministic tests |
| `tests/death.rs` | player/monster death and GameOver tests |

### 3.2 핵심 타입 계약

```rust
pub enum EntityKind {
    Player,
    Monster(MonsterKind),
}

pub enum MonsterKind {
    Jackal,
    Goblin,
    FloatingEye,
}

pub enum Faction {
    Player,
    Hostile,
    Neutral,
}

pub struct ActorStats {
    pub hp: i16,
    pub max_hp: i16,
    pub ac: i16,
    pub hit_bonus: i16,
    pub damage_bonus: i16,
    pub damage: DamageRoll,
}

pub struct Entity {
    pub id: EntityId,
    pub kind: EntityKind,
    pub faction: Faction,
    pub pos: Pos,
    pub stats: ActorStats,
    pub alive: bool,
}

pub struct EntityStore {
    entities: Vec<Entity>,
    next_id: u32,
}

pub struct DamageRoll {
    pub dice: i16,
    pub sides: i16,
}

pub enum DeathCause {
    Combat { attacker: EntityId },
}
```

계약:

- `EntityId(0)`은 invalid sentinel이다. 첫 실제 entity는 `EntityId(1)`이다.
- Phase 3 fixture는 player, jackal, goblin을 spawn한다. floating eye 데이터는 type/factory 수준까지만 허용하고 combat test 필수 대상은 아니다.
- `GameWorld.player_id`는 player entity를 가리키며, player position은 entity store에서 읽는다.
- tile occupancy는 alive entity만 고려한다.
- monster death는 `alive=false` tombstone으로 시작한다. Vec index 안정성을 위해 즉시 compact remove는 하지 않는다.
- player death는 `RunState::GameOver`로 전환한다.

## 4. Combat 공식

`spec.md` 기준을 그대로 사용한다.

```text
attack_roll = d20 + attacker.hit_bonus + weapon.hit_bonus
defense = 10 + defender.ac
hit = attack_roll >= defense
```

Phase 3 weapon policy:

- item/inventory는 Phase 4 범위이므로 실제 weapon entity는 만들지 않는다.
- player는 기본 dagger profile을 내장 공격 프로필로 사용한다.
- monster는 monster data의 damage roll을 사용한다.

기본값:

| Actor | hp | ac | hit_bonus | damage | weapon_hit_bonus | damage_bonus |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| player.adventurer | 16 | 0 | 2 | 1d4 dagger | 1 | 0 |
| monster.jackal | 4 | 0 | 0 | 1d2 | 0 | 0 |
| monster.goblin | 6 | 1 | 1 | 1d4 | 0 | 0 |
| monster.floating_eye | 8 | 2 | 0 | 0 | 0 | 0 |

피해:

```text
damage = max(1, damage_roll + attacker.damage_bonus - defender.damage_reduction)
```

Phase 3에서 `damage_reduction = 0`으로 고정한다.

## 5. Fixture 데이터

Phase 3는 Phase 2 map fixture를 유지하고 entity만 추가한다.

```text
player = { id = EntityId(1), kind = Player, pos = (5,5), hp = 16 }
jackal = { id = EntityId(2), kind = Monster(Jackal), pos = (6,5), hp = 4 }
goblin = { id = EntityId(3), kind = Monster(Goblin), pos = (20,12), hp = 6 }
```

주의:

- Phase 2 문/시야 tests와 충돌하지 않도록 tests는 필요 시 synthetic fixture 또는 entity-cleared world helper를 사용한다.
- Phase 3 runtime 기본 fixture에 jackal이 adjacent라면 첫 east move가 movement가 아니라 bump attack이 된다. 이 변경은 문서와 tests에 명시한다.

## 6. 제외 범위 / 비목표

- item/inventory/equipment entity 구현.
- weapon pickup/wield/wear/quaff.
- monster AI chase/wander/stationary behavior. 몬스터 반격도 Phase 3 필수 범위가 아니며, player death tests는 직접 damage helper 또는 synthetic command sequence로 검증한다.
- ranged attack, throw, zap, spell.
- XP/score/death score 정식 계산. `GameOver` final score는 0 또는 placeholder로 고정 가능.
- corpse/drop generation.
- save/load persistence.
- TUI/animation/effect.

## 7. 구현 순서

### Step 1: Entity domain

- `EntityStore`, `Entity`, `EntityKind`, `Faction`, `ActorStats`를 추가한다.
- stable `EntityId` 생성과 tombstone policy를 테스트한다.
- player/monster position lookup helper를 만든다.

### Step 2: Player/monster data factory

- player adventurer, jackal, goblin, floating eye 기본 데이터를 코드 factory로 둔다.
- Phase 3 tests는 player/jackal/goblin을 필수로 검증한다.

### Step 3: World migration

- `GameWorld`가 `EntityStore`와 `player_id`를 소유한다.
- 기존 movement/vision/observation이 player position을 entity store에서 읽도록 변경한다.
- Phase 2 tests가 필요한 경우 helper로 no-monster fixture를 제공한다.

### Step 4: Combat formulas

- `roll_d20`, `roll_damage`, `resolve_attack`을 `systems/combat.rs`에 둔다.
- RNG는 `GameSession.rng`를 mutable로 받아 사용한다.
- hit/miss와 damage는 `AttackResolved` event로 기록한다.

### Step 5: Bump attack integration

- `Move(dir)` target에 hostile alive monster가 있으면 이동하지 않고 attack을 resolve한다.
- attack accepted이면 turn을 진행한다.
- miss도 accepted + turn advance다.
- target이 죽으면 `EntityDied` event를 같은 turn outcome에 포함한다.

### Step 6: Death system

- monster hp <= 0이면 `alive=false` tombstone 처리.
- player hp <= 0이면 `RunState::GameOver`로 전환하고 `EntityDied` event를 생성한다.
- dead entity는 movement blocker/visible hostile/legal action 대상에서 제외한다.

### Step 7: Snapshot/Observation update

- snapshot hash에 entity id/kind/pos/hp/alive를 포함한다.
- `Observation.visible_tiles`는 유지한다.
- 선택적으로 `visible_entities` 최소 스키마를 추가할 수 있지만, Phase 3 필수 완료 기준이 아니므로 scope가 커지면 제외한다.

### Step 8: 문서 동기화

- 구현 완료 후 `CHANGELOG.md`, `spec.md`, `implementation_summary.md`, `audit_roadmap.md`, 필요 시 `DESIGN_DECISIONS.md`를 갱신한다.
- snapshot hash 기준이 바뀌면 Phase 2 기준 hash와 Phase 3 기준 hash를 모두 기록한다.

## 8. 수용 기준

| ID | 기준 | 검증 |
| --- | --- | --- |
| AC-1 | `EntityStore`가 `EntityId(1)`부터 stable id를 생성 | `cargo test entity_store_stable_ids` |
| AC-2 | player/jackal/goblin 기본 stat 생성 | `cargo test actor_factories_match_spec_data` |
| AC-3 | `Move(dir)` target에 monster가 있으면 이동 대신 bump attack | `cargo test bump_attack_does_not_move_player` |
| AC-4 | hit formula deterministic | `cargo test combat_hit_formula_seeded` |
| AC-5 | damage formula deterministic and min 1 | `cargo test combat_damage_formula_seeded` |
| AC-6 | jackal/goblin hp 감소와 `AttackResolved` event | `cargo test jackal_goblin_combat_events` |
| AC-7 | monster hp <= 0이면 `EntityDied`와 alive=false | `cargo test monster_death_creates_event_and_tombstone` |
| AC-8 | player hp <= 0이면 `RunState::GameOver` | `cargo test player_death_enters_game_over` |
| AC-9 | dead monster는 movement blocker가 아님 | `cargo test dead_monster_no_longer_blocks_movement` |
| AC-10 | rejected command no-turn 계약 유지 | `cargo test rejected_commands_do_not_advance_turn` |
| AC-11 | Phase 2 map/movement/doors/vision tests 유지 | `cargo test map movement doors vision observation` |
| AC-12 | legacy direct import 없음 | `rg "legacy_nethack_port_reference" src Cargo.toml` 결과 없음 |
| AC-13 | 품질 게이트 통과 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |

## 9. 리스크와 완화

| 리스크 | 영향 | 완화 |
| --- | --- | --- |
| Entity store가 Phase 4 item까지 확장됨 | scope creep | Phase 3 entity kind는 Player/Monster만 허용 |
| Adjacent jackal 때문에 기존 movement tests 실패 | regression | no-monster fixture helper 또는 player 위치 조정으로 Phase 2 semantics 보존 |
| RNG 호출 순서가 불안정 | hash/test flake | combat RNG 호출 순서를 `resolve_attack` 내부로 고정하고 tests로 seed sequence 검증 |
| Death 처리와 event 순서 혼란 | replay/hash 불안정 | `AttackResolved` 후 `EntityDied` 순서 고정 |
| Player death test를 위해 monster AI를 구현하게 됨 | scope creep | direct damage helper 또는 synthetic attack test로 검증 |

## 10. ADR

### 결정

Phase 3는 `EntityId` 기반 Minimal `EntityStore`를 도입하고, item/equipment/monster AI 없이 bump attack combat/death만 구현한다.

### Drivers

- `spec.md`는 초기 엔티티를 `EntityId` 기반 arena/vector 저장소로 시작한다고 명시한다.
- Phase 3 완료 기준은 jackal/goblin combat와 death event다.
- Phase 4 item/inventory와 Phase 6 monster AI를 침범하면 검증 범위가 과도해진다.

### Alternatives considered

- `GameWorld`에 hp/monster vec 직접 추가: 빠르지만 `EntityId` event 계약과 충돌한다.
- Full ECS/Legion: 문서상 금지이며 이전 레거시 문제를 반복한다.
- item/equipment를 같이 구현: weapon formula가 명확해지지만 Phase 4 범위 침범이다.

### Why chosen

Minimal `EntityStore`는 `EntityId`, combat event, death event 계약을 만족하면서도 후속 item/AI 구현을 미룰 수 있는 가장 작은 구조다.

### Consequences

- `GameWorld.player_pos`는 player entity position helper로 이전된다.
- Snapshot hash는 entity state를 포함하므로 Phase 2 기준 hash가 바뀐다.
- Phase 4에서는 item entity를 같은 store에 추가하되, 이 PRD의 player/monster invariant를 깨지 않아야 한다.

### Follow-ups

- Phase 4 PRD에서 item/inventory를 `EntityStore`에 통합하는 정책을 별도 결정한다.
- Phase 6 PRD에서 monster AI가 combat과 turn order에 어떻게 연결되는지 결정한다.

## 11. Ralph/Team 후속 실행 지침

### 권장 실행 방식

- 권장: `$ralph .omx/plans/prd-phase3-combat-death.md`
- 이유: Entity store migration과 combat/death는 경계가 강하게 연결되어 있어 단일 persistence loop가 안전하다.

### Team이 필요한 경우

`$team`은 다음 조건에서만 고려한다.

- Entity store migration과 combat formula tests가 서로 독립적으로 오래 걸릴 때.
- death/game-over regression이 반복되어 debugger lane이 필요할 때.

### 사용 가능한 agent type roster

| 역할 | 용도 | 권장 reasoning |
| --- | --- | --- |
| `executor` | Phase 3 구현 | medium |
| `test-engineer` | combat/death deterministic tests 설계 | medium |
| `debugger` | RNG/order/death regression 분석 | high |
| `build-fixer` | cargo/clippy 실패 수정 | high |
| `architect` | entity store와 future item/AI 경계 검증 | high |
| `verifier` | PRD/test-spec 충족 증거 확인 | high |
| `code-reviewer` | 구현 후 범위/품질 검토 | high |

### Ralph 실행 중 금지

- Phase 4 item/inventory로 진행 금지.
- Phase 6 monster AI로 진행 금지.
- TUI/animation/effect 구현 금지.
- 레거시 직접 import 금지.
- 테스트 실패 상태에서 완료 보고 금지.

## 12. Consensus 결과

- Planner: 승인. Minimal `EntityStore`가 Phase 3 combat/death 목표와 후속 확장성을 가장 작게 만족한다.
- Architect: 승인. `GameSession` 단일 상태 원천과 `EntityId` event 계약을 강화한다.
- Critic: 승인. hit/damage/death, scope creep, RNG determinism, Phase 2 regression에 대한 수용 기준이 명확하다.
