# AIHack Master Spec

문서 상태: active
작성일: 2026-04-28
기준 문서: `AI_IMPLEMENTATION_DOC_STANDARD.md`

## 1. 문서 운영 규칙

이 문서는 새 AIHack 런타임의 최상위 계약이다. 구현자가 다른 결정을 해야 할 때는 이 문서를 먼저 수정하고, `DESIGN_DECISIONS.md`에 이유를 남긴다.

금지:

- 레거시 `legacy_nethack_port_reference/src` 직접 import
- UI에서 게임 상태 직접 수정
- LLM이 `GameSession`을 직접 변경
- RNG seed 없이 재현 불가능한 테스트 작성
- 구현되지 않은 기능을 문서상 완료로 표시

## 2. 프로젝트 정체성

AIHack은 NetHack-inspired 턴제 로그라이크다. 원본 NetHack의 방대한 규칙과 긴장감을 참고하지만, 목표는 1:1 C 포트가 아니라 **AI가 안정적으로 관찰하고 행동할 수 있는 결정론적 Rust 게임 엔진**이다.

## 3. 목표

### 3.1 제품 목표

- 단일 플레이어 로컬 로그라이크
- ASCII 우선 UI
- headless simulation과 GUI/TUI가 같은 엔진 사용
- AI 관찰/행동 인터페이스 1급 지원
- NetHack 핵심 루프 보존: 탐험, 전투, 아이템, 위험한 상호작용, 죽음

### 3.2 성공 기준

첫 플레이어블 v0.1 성공 기준:

- 같은 seed에서 1000턴 headless replay의 event log hash가 항상 동일
- 플레이어가 1층에서 이동, 문 열기/닫기, 아이템 줍기, 장비 착용, 근접 공격, 계단 이동 가능
- 몬스터 3종이 시야 기반 추적 또는 배회 행동 수행
- 사망 시 `GameOver` 상태와 death event 생성
- `Observation` JSON이 매 턴 생성되고 schema test 통과
- `cargo test`와 `cargo run --bin aihack-headless -- --seed 42 --turns 1000` 통과

## 4. 비목표

v0.1 비목표:

- NetHack 3.6.7 전체 규칙 1:1 복제
- Legion ECS 사용
- 실시간 멀티스레드 게임 로직
- 네트워크 멀티플레이
- LLM이 직접 게임 명령을 자유 텍스트로 실행
- 그래픽 타일셋
- 세이브 파일 하위 호환성

## 5. 동결된 핵심 결정

| 결정 | 값 |
| --- | --- |
| 언어 | Rust 2021 또는 최신 stable compatible edition |
| 런타임 모델 | 단일 스레드 턴 트랜잭션 |
| 상태 소유자 | `GameSession` 단일 원천 |
| UI 경계 | UI는 `GameSnapshot` 읽기, `CommandIntent` 쓰기만 가능 |
| AI 경계 | AI는 `Observation` 읽기, `ActionIntent` 쓰기만 가능 |
| RNG | seed 기반 deterministic RNG |
| 레거시 코드 | 참조만, 직접 import 금지 |
| 저장 | JSON snapshot v1, 이후 binary optional |
| LLM 1차 역할 | narrative only |

## 6. 기술 스택

초기 스택:

- Rust
- `serde`, `serde_json`
- `thiserror`
- `rand` 또는 자체 deterministic RNG wrapper
- CLI headless runner
- UI는 core 안정화 후 선택: `ratatui` 우선, `egui`는 별도 adapter

초기에는 ECS를 사용하지 않는다. 엔티티는 `EntityId` 기반 arena/vector 저장소로 시작한다. ECS 재도입은 v0.3 이후 performance audit에서만 검토한다.

## 7. 디렉터리 구조

예정 구조:

```text
src/
  main.rs
  bin/
    aihack-headless.rs
  core/
    mod.rs
    session.rs
    ids.rs
    rng.rs
    error.rs
    turn.rs
    snapshot.rs
    observation.rs
    action.rs
    event.rs
    save.rs
  domain/
    map.rs
    tile.rs
    entity.rs
    player.rs
    monster.rs
    item.rs
    combat.rs
    inventory.rs
    status.rs
  systems/
    movement.rs
    doors.rs
    vision.rs
    combat.rs
    monster_ai.rs
    items.rs
    stairs.rs
    death.rs
  data/
    items.toml
    monsters.toml
    levels/
  ui/
    tui/
    debug/
  tests/
    fixtures/
```

## 8. 핵심 타입 계약

### 8.1 식별자

```rust
pub struct EntityId(pub u32);
pub struct LevelId { pub branch: BranchId, pub depth: i16 }
pub enum BranchId { Main, Mines, Quest, Gehennom, Endgame }
```

`EntityId(0)`은 invalid sentinel로 예약한다. 실제 엔티티는 `1..=u32::MAX`를 사용한다.

### 8.2 GameSession

```rust
pub struct GameSession {
    pub meta: GameMeta,
    pub rng: GameRng,
    pub turn: u64,
    pub state: RunState,
    pub world: GameWorld,
    pub event_log: Vec<GameEvent>,
}
```

```rust
pub enum RunState {
    Title,
    CharacterCreation,
    Playing,
    AwaitingDirection { action: DirectionalAction },
    AwaitingInventorySelection { action: InventoryAction },
    MorePrompt,
    GameOver { cause: DeathCause, final_score: i32 },
}
```

### 8.3 CommandIntent

```rust
pub enum CommandIntent {
    Move(Direction),
    Wait,
    Open(Direction),
    Close(Direction),
    Kick(Direction),
    Search,
    Pickup,
    Drop { item: EntityId },
    Wear { item: EntityId },
    Wield { item: EntityId },
    Quaff { item: EntityId },
    Read { item: EntityId },
    Zap { item: EntityId, direction: Direction },
    Throw { item: EntityId, direction: Direction },
    Descend,
    Ascend,
    ShowInventory,
    AcknowledgeMore,
    Quit,
}
```

### 8.4 TurnOutcome

```rust
pub struct TurnOutcome {
    pub accepted: bool,
    pub turn_advanced: bool,
    pub events: Vec<GameEvent>,
    pub snapshot_hash: SnapshotHash,
    pub next_state: RunState,
}
```

`accepted=false`인 명령은 게임 턴을 진행하지 않는다. 예: 벽 방향으로 닫을 문이 없는 `Close`.

### 8.5 Observation

```rust
pub struct Observation {
    pub schema_version: u16,
    pub seed: u64,
    pub turn: u64,
    pub run_state: RunStateSummary,
    pub player: PlayerObservation,
    pub visible_tiles: Vec<TileObservation>,
    pub visible_entities: Vec<EntityObservation>,
    pub inventory: Vec<ItemObservation>,
    pub last_events: Vec<GameEvent>,
    pub legal_actions: Vec<ActionIntent>,
}
```

AI는 `Observation` 밖의 상태를 읽지 않는다.

### 8.6 ActionIntent

```rust
pub enum ActionIntent {
    Command(CommandIntent),
    NarrativeRequest { topic: NarrativeTopic },
    Noop,
}
```

LLM decision support는 `ActionIntent::Command` 후보를 제안할 수 있지만, 엔진은 항상 validator를 통과한 명령만 실행한다.

### 8.7 GameEvent

```rust
pub enum GameEvent {
    TurnStarted { turn: u64 },
    CommandRejected { reason: RejectReason },
    EntityMoved { entity: EntityId, from: Pos, to: Pos },
    DoorChanged { pos: Pos, from: DoorState, to: DoorState },
    ItemPickedUp { entity: EntityId, item: EntityId },
    ItemEquipped { entity: EntityId, item: EntityId, slot: EquipmentSlot },
    AttackResolved { attacker: EntityId, defender: EntityId, hit: bool, damage: i16 },
    EntityDied { entity: EntityId, cause: DeathCause },
    LevelChanged { entity: EntityId, from: LevelId, to: LevelId },
    Message { priority: MessagePriority, text: String },
}
```

## 9. 핵심 공식

초기 공식은 단순하고 고정한다. NetHack 정밀 공식은 Phase 8 이후 선별 흡수한다.

### 9.1 명중

```text
attack_roll = d20 + attacker.hit_bonus + weapon.hit_bonus
defense = 10 + defender.ac
hit = attack_roll >= defense
```

기본값:

- 플레이어 기본 `hit_bonus = 2`
- goblin 기본 `hit_bonus = 1`
- jackal 기본 `hit_bonus = 0`
- dagger `hit_bonus = 1`
- bare hands `hit_bonus = 0`

### 9.2 피해

```text
damage = max(1, weapon.damage_roll + attacker.damage_bonus - defender.damage_reduction)
```

기본 무기:

- bare hands: `1d2`
- dagger: `1d4`
- short sword: `1d6`

### 9.3 시야

초기 시야:

- 기본 반경: 8 tiles
- 벽과 닫힌 문은 line of sight를 차단
- 열린 문은 차단하지 않음
- 어둠/광원 시스템은 v0.2에서 추가

## 10. 실데이터 기준표

### 10.1 플레이어 기본값

```toml
[player]
id = "player.adventurer"
hp = 16
max_hp = 16
energy = 6
strength = 10
dexterity = 10
ac = 0
hit_bonus = 2
damage_bonus = 0
vision_radius = 8
start_items = ["item.weapon.dagger", "item.food.ration"]
```

### 10.2 몬스터 v0.1

```toml
[[monster]]
id = "monster.jackal"
glyph = "d"
hp = 4
ac = 0
hit_bonus = 0
damage = "1d2"
ai = "wander_then_chase"
speed = 12

[[monster]]
id = "monster.goblin"
glyph = "g"
hp = 6
ac = 1
hit_bonus = 1
damage = "1d4"
ai = "chase_on_sight"
speed = 12

[[monster]]
id = "monster.floating_eye"
glyph = "e"
hp = 8
ac = 2
hit_bonus = 0
damage = "0"
ai = "stationary"
passive = "paralyze_on_melee"
speed = 0
```

### 10.3 아이템 v0.1

```toml
[[item]]
id = "item.weapon.dagger"
kind = "weapon"
glyph = ")"
weight = 10
slot = "melee"
hit_bonus = 1
damage = "1d4"

[[item]]
id = "item.food.ration"
kind = "food"
glyph = "%"
weight = 20
nutrition = 800

[[item]]
id = "item.potion.healing"
kind = "potion"
glyph = "!"
weight = 20
effect = "heal_1d8_plus_4"
```

### 10.4 첫 레벨 fixture

```text
level_id = "main:1"
size = 40x20
player_start = (5, 5)
stairs_down = (34, 15)
monsters = [
  { id = "monster.jackal", pos = (12, 5) },
  { id = "monster.goblin", pos = (20, 12) }
]
items = [
  { id = "item.potion.healing", pos = (8, 5) }
]
```

## 11. 저장 정책

초기 저장은 JSON이다.

```rust
pub struct SaveDataV1 {
    pub schema_version: u16,
    pub created_at_unix: i64,
    pub game_meta: GameMeta,
    pub session: GameSessionSnapshot,
}
```

경로:

- 개발 저장: `runtime/save/dev_save.json`
- headless replay: `runtime/replays/{seed}-{turns}.jsonl`
- snapshot: `runtime/snapshots/{seed}-{turn}.json`

## 12.1 Phase 1 구현 완료 상태

2026-04-28 기준 Phase 1 Headless Core는 완료되었다.

구현된 파일:

```text
Cargo.toml
Cargo.lock
src/main.rs
src/lib.rs
src/bin/aihack-headless.rs
src/core/action.rs
src/core/error.rs
src/core/event.rs
src/core/ids.rs
src/core/mod.rs
src/core/rng.rs
src/core/session.rs
src/core/snapshot.rs
src/core/turn.rs
```

완료된 계약:

- `GameRng::new(seed)`는 같은 seed에서 같은 sequence를 만든다.
- `GameSession::new(seed)`는 `turn = 0`, `RunState::Playing`, 빈 event log로 시작한다.
- `CommandIntent::Wait`은 `accepted = true`, `turn_advanced = true`, `turn += 1`을 보장한다.
- `TurnOutcome.snapshot_hash`는 seed, turn, run_state, event summary를 포함한 `GameSnapshot`에서 FNV-1a 64-bit로 생성한다.
- `aihack-headless --seed <u64> --turns <u64>`는 `seed`, `turns`, `final_turn`, `final_hash`를 stdout에 출력한다.

검증된 기준값:

```text
seed=42 turns=0   final_turn=0   final_hash=66595593fabacdf4
seed=42 turns=100 final_turn=100 final_hash=f827bc2d4155ef66
seed=43 turns=100 final_turn=100 final_hash=3ed5b4db4d5e7157
```

주의:

- `StdRng` sequence는 `Cargo.lock`에 고정된 `rand` 버전에 묶인다. `rand` 업그레이드는 replay/hash 영향 변경으로 취급하고 `DESIGN_DECISIONS.md`에 결정 기록을 남긴다.
- Phase 1은 map, movement, combat, item, TUI를 구현하지 않는다.

## 12.2 Phase 2 구현 완료 상태

2026-04-28 기준 Phase 2 Map, Movement, Doors, Vision은 완료되었다.

구현된 파일:

```text
src/core/position.rs
src/core/world.rs
src/core/observation.rs
src/domain/mod.rs
src/domain/map.rs
src/domain/tile.rs
src/systems/mod.rs
src/systems/movement.rs
src/systems/doors.rs
src/systems/vision.rs
tests/map.rs
tests/movement.rs
tests/doors.rs
tests/vision.rs
tests/observation.rs
```

완료된 계약:

- Phase 2 완료 당시 `GameWorld { map, player_pos }`가 `GameSession`의 단일 상태 원천 아래 추가되었다. Phase 3 완료 후 player position은 `EntityStore`의 player entity가 원천이며 `GameWorld::player_pos()` getter로 조회한다.
- 40x20 fixture map은 `player_start = (5,5)`, `stairs_down = (34,15)`, closed door `(10,5)`와 `(14,5)`, interior wall `x=12, y=4..8`을 가진다.
- `Move(Direction)`은 floor/open door/stairs에서 accepted + turn advance, wall/closed door/out-of-bounds에서 rejected + no turn advance를 보장한다.
- diagonal corner-cutting은 금지된다.
- `Open(Direction)`/`Close(Direction)`은 adjacent door 상태만 변경하고 `DoorChanged` event를 생성한다.
- LOS radius는 8이며 wall/closed door는 차단하고 open door는 통과한다.
- `Observation.visible_tiles`와 최소 `legal_actions`가 생성된다.
- Snapshot hash는 `player_pos`, map size, map tile state를 포함한다.

검증된 기준값:

```text
seed=42 turns=100 final_turn=100 final_hash=1aad6f4049778b0e
```

주의:

- Phase 2에서 snapshot hash 입력이 확장되었으므로 Phase 1의 `seed=42 turns=100` 기준 hash `f827bc2d4155ef66`은 더 이상 현재 런타임 기준값이 아니다.
- Item, inventory, monster AI, TUI는 아직 구현되지 않았다. Entity store와 combat/death는 Phase 3에서 구현 완료되었다.

## 12.3 Phase 3 구현 완료 상태

2026-04-28 기준 Phase 3 Combat and Death는 `.omx/plans/prd-phase3-combat-death.md`와 `.omx/plans/test-spec-phase3-combat-death.md` 기준으로 완료되었다.

구현된 파일:

```text
src/domain/combat.rs
src/domain/entity.rs
src/domain/monster.rs
src/domain/player.rs
src/core/world.rs
src/core/event.rs
src/core/session.rs
src/core/snapshot.rs
src/systems/combat.rs
src/systems/death.rs
tests/combat.rs
tests/death.rs
```

완료된 계약:

- `EntityId(0)`은 invalid sentinel이며 첫 실제 entity는 `EntityId(1)`이다.
- `EntityStore`는 player, jackal, goblin을 보관하고 사망 엔티티를 즉시 제거하지 않는 tombstone 정책을 사용한다.
- 기본 fixture는 player `(5,5)`, jackal `(6,5)`, goblin `(20,12)`를 생성한다.
- `Move(Direction)` 대상에 살아있는 hostile monster가 있으면 이동 대신 bump attack이 실행되며 player 위치는 유지된다. 이 bump action은 `Observation.legal_actions`에도 노출된다.
- 명중 공식은 `d20 + attacker.hit_bonus + weapon.hit_bonus >= 10 + defender.ac`이다.
- 피해 공식은 `max(1, damage_roll + attacker.damage_bonus - defender.damage_reduction)`이며 Phase 3 `damage_reduction`은 0이다.
- Player는 장비/아이템 엔티티 없이 내장 dagger profile(`hit_bonus=1`, `1d4`)을 사용한다.
- `AttackResolved`와 `EntityDied` event가 snapshot/replay 입력에 포함된다.
- Monster death는 `alive=false` tombstone으로 기록되고 movement blocker에서 제외된다.
- Player death는 `RunState::GameOver`로 전환된다.
- Snapshot hash는 entity id/kind/position/hp/alive 상태를 포함한다.

검증된 기준값:

```text
seed=42 turns=100 final_turn=100 final_hash=8b20a23301eea977
```

주의:

- Phase 3에서 snapshot hash 입력이 entity state까지 확장되었으므로 Phase 2의 `seed=42 turns=100` 기준 hash `1aad6f4049778b0e`은 더 이상 현재 런타임 기준값이 아니다.
- Monster AI, 반격, item/inventory/equipment, ranged/throw/zap/spell, XP/score, corpse/drop, save/load, TUI/effect는 아직 구현하지 않았다.

## 12.4 Phase 4 구현 완료 상태

2026-04-28 기준 Phase 4 Items and Inventory는 `.omx/plans/prd-phase4-items-inventory.md`와 `.omx/plans/test-spec-phase4-items-inventory.md` 기준으로 완료되었다.

구현된 파일:

```text
src/domain/item.rs
src/domain/inventory.rs
src/domain/entity.rs
src/core/action.rs
src/core/event.rs
src/core/world.rs
src/core/session.rs
src/core/snapshot.rs
src/core/observation.rs
src/systems/items.rs
src/systems/combat.rs
tests/items.rs
tests/inventory.rs
```

완료된 계약:

- `EntityPayload::Actor | EntityPayload::Item` 구조로 actor/item invalid state를 분리했다.
- `EntityKind::Item(ItemKind)`가 같은 `EntityId` 공간에 통합되었다.
- 시작 fixture는 potion healing `EntityId(4)` at `(8,5)`, dagger `EntityId(5)` letter `a`, food ration `EntityId(6)` letter `b`를 가진다.
- `Pickup`은 player 위치의 가장 작은 `EntityId` item 하나만 줍고 stable inventory letter를 할당한다.
- Equipment source of truth는 `Inventory.equipped_melee` 하나다.
- `Wield { item }`은 dagger를 melee slot에 장착하고, 이미 장착된 같은 item 재호출은 accepted/no-turn/eventless다.
- `Quaff { item }`은 potion healing을 `1d8+4`로 회복하고 item entity를 `EntityLocation::Consumed` tombstone으로 남기며 `assigned_letter`를 유지한다.
- `ShowInventory`는 accepted/no-turn/eventless이며 UI/AI는 `Observation.inventory`를 읽는다.
- Snapshot hash는 item location, assigned letter, inventory entries, equipped melee 상태를 포함한다.

검증된 기준값:

```text
seed=42 turns=0 final_turn=0 final_hash=821520dc302c9ea2
seed=42 turns=100 final_turn=100 final_hash=88886c28698a1730
seed=43 turns=100 final_turn=100 final_hash=948c5ec460bebb99
```

주의:

- Phase 5에서 snapshot hash 입력이 current level, deterministic level registry, level-aware actor/item location까지 확장되었으므로 Phase 4의 `seed=42 turns=100` 기준 hash `00ba578d933177f2`는 더 이상 현재 런타임 기준값이 아니다.
- File save/load, drop/read/zap/throw, TUI inventory screen, monster item use, procedural generation은 아직 구현하지 않았다.

### 11.5 Phase 5 완료 기준

Phase 5는 fixed `main:1`/`main:2` level registry, 명시적 `Descend`/`Ascend` 명령, `LevelChanged` event, level-aware actor/item `EntityLocation::OnMap { level, pos }`를 구현했다. `GameWorld.current_level`은 player actor location level과 같은 값이어야 하며, player 위치 변경은 `set_player_location(level, pos)` 경로로 atomic하게 수행한다. `Move`는 stairs tile에 올라서는 동작만 수행하고 level transition은 발생시키지 않는다.

검증 범위:

- `tests/levels.rs`: fixed level registry, 1층 몬스터/아이템 보존, level snapshot hash
- `tests/stairs.rs`: descend/ascend reject/accept, landing, event order, legal actions
- `cargo test --test movement --test doors --test vision --test combat --test death --test items --test inventory` 회귀 통과

## 12. 단계별 로드맵

| Phase | 이름 | 완료 기준 |
| --- | --- | --- |
| 0 | 문서/레거시 경계 | 루트 문서세트와 레거시 인덱스 완료 |
| 1 | Headless core | seed 42, 100턴 replay deterministic |
| 2 | 맵/이동/문/시야 | fixture level에서 이동/문/시야 테스트 통과 |
| 3 | 전투/사망 | jackal/goblin 전투와 death event 통과 |
| 4 | 아이템/인벤토리 | pickup, wield, wear, quaff 통과 |
| 5 | 던전/계단 | 완료: 1층-2층 왕복과 level snapshot 통과 |
| 6 | 몬스터 AI | 추적/배회/정지 AI 테스트 통과 |
| 7 | NetHack 상호작용 | trap, wand, throw, read 최소 구현 |
| 8 | 레거시 규칙 흡수 | 핵심 규칙 20개 golden test 통과 |
| 9 | 저장/replay | save/load 후 hash 일치 |
| 10 | UI | TUI에서 v0.1 핵심 루프 플레이 |
| 11 | AI API | Observation/ActionSpace schema freeze |
| 12 | LLM narrative | timeout/fallback 포함 narrative only |
| 13 | LLM decision support | validator 통과 명령만 제한 실행 |

## 13. 검증 명령

초기 스캐폴딩 후 고정 명령:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 1000
```

## 14. 잔여 리스크

| 리스크 | 영향 | 완화 |
| --- | --- | --- |
| NetHack 라이선스 경계 | 높음 | 레거시 코드 직접 import 금지, 복사 전 검토 |
| 문서 과대 범위 | 중간 | Phase별 완료 기준 고정 |
| AI 자유 행동 폭주 | 높음 | `ActionSpace` validator 강제 |
| 새 엔진 재복잡화 | 높음 | UI/AI/core 경계 테스트 |
| 원본 재미 손실 | 중간 | Phase 8 golden rules로 흡수 |

## 15. 현대 TUI/UX 리팩토링 계획

이 섹션은 2026-04-28 deep-interview 결과와 외부 TUI 로그라이크 그라운딩을 반영한 UI/UX 선반영 계약이다. 구현은 headless core 안정화 이후 진행하되, core/UI 경계 타입은 지금부터 이 계획을 고려해 설계한다.

### 15.1 목표

목표는 **단계형 혼합 로드맵**이다.

- v0.1: 기존 Phase 10의 안전한 TUI를 유지하되, 정보 가독성 중심의 기본 현대화와 최소 마우스 입력을 포함한다.
- v0.2: Brogue식 접근성 수준을 목표로, 키보드/마우스 혼합 플레이와 hover/inspect, 클릭 가능한 패널을 완성한다.
- v0.3: Cogmind식 고급 ASCII 경험을 장기 목표로, 자동 라벨, ASCII 피드백 애니메이션, 상태 강조, 드래그 기반 인벤토리 UX를 presentation layer로 확장한다.

### 15.2 외부 그라운딩 기준

| 기준 게임/도구 | 확인된 UX 수준 | AIHack 반영 방식 |
| --- | --- | --- |
| Cogmind | ASCII 기반 현대 인터페이스, 전체 명령 키보드/마우스 접근, 드래그 앤 드롭 인벤토리, 자동 라벨, 다채널 피드백, ASCII particle/effect | v0.3 장기 목표. 단, gameplay state가 아니라 `UiEffectEvent`로만 표현 |
| Brogue | 키보드/마우스 단독 또는 혼합 플레이, 명령 도움말, 단순하지만 읽기 쉬운 ASCII UX | v0.2 접근성 목표. 클릭 이동/검사/도움말 우선 |
| Ratatui + Crossterm | immediate rendering, backend 기반 raw mode/alternate screen/mouse capture, tick/frame rate event loop | TUI adapter 후보. 단일 Crossterm major version만 허용 |

### 15.3 동결된 비목표

- 그래픽 타일셋 렌더러는 이 계획의 범위가 아니다.
- ASCII 애니메이션과 효과는 deterministic core replay hash에 영향을 주면 안 된다.
- UI는 여전히 `GameSnapshot`/`Observation`을 읽고 `CommandIntent`만 제출한다.
- UI 편의 기능 때문에 `GameSession` 내부 상태, RNG, entity store를 직접 수정하지 않는다.
- v0.1은 Cogmind급 전체 효과 parity를 완료 조건으로 삼지 않는다. 고급 효과는 v0.2 이후 단계별 확장이다.

### 15.4 성공 기준 우선순위

사용자 체감 개선의 1순위는 **정보 가독성**이다. 마우스 조작성과 타격감은 정보 가독성을 해치지 않는 범위에서 뒤따른다.

정량 기준:

| 항목 | v0.1 기준 | v0.2 기준 | v0.3 기준 |
| --- | --- | --- | --- |
| 최소 터미널 | 80x28 | 100x32 권장, 80x28 degraded | 120x36 권장, 80x28 degraded |
| 지도 viewport | 40x20 고정 | 60x24까지 adaptive | terminal 크기 기반 adaptive |
| 메시지 표시 | 최근 5줄 + 중복 압축 | priority별 색/아이콘 + 필터 | event category tab + hover origin |
| 자동 라벨 | 없음 | 새로 보인 hostile/item 최대 3개, 1200ms | 설정 가능, 위험도 기반 우선순위 |
| hover/inspect | 없음 | map/entity/status hover read-only | 비교 tooltip과 command hint 연동 |
| 마우스 | 선택 사항 | 지도 클릭 이동/검사, 패널 클릭 | drag/drop inventory 후보 |
| 애니메이션 | 없음 또는 off | 80~160ms 상태 flash | 30fps presentation loop에서 80~400ms effect |

### 15.5 UI 전용 타입 계약

Core event와 UI effect를 분리한다.

```rust
pub struct UiRuntimeConfig {
    pub enable_mouse: bool,
    pub enable_animations: bool,
    pub frame_rate: f32,
    pub tick_rate: f32,
    pub reduced_motion: bool,
    pub color_profile: UiColorProfile,
}

pub enum UiColorProfile {
    Default,
    HighContrast,
    ColorBlindSafe,
    Monochrome,
}

pub enum UiInputEvent {
    Key(CommandKey),
    MouseClick { x: u16, y: u16, button: MouseButton },
    MouseDrag { from: UiPoint, to: UiPoint, button: MouseButton },
    MouseHover { x: u16, y: u16 },
    Resize { width: u16, height: u16 },
    Tick,
}

pub enum UiCommandCandidate {
    Submit(CommandIntent),
    InspectMap { pos: Pos },
    FocusPanel { panel: UiPanelId },
    ToggleDebugObservation,
    Noop,
}

pub struct UiEffectEvent {
    pub effect_id: UiEffectId,
    pub anchor: UiEffectAnchor,
    pub kind: UiEffectKind,
    pub started_at_ms: u64,
    pub duration_ms: u16,
}

pub enum UiEffectKind {
    DamageFlash { amount: i16 },
    HealPulse { amount: i16 },
    MissWisp,
    DoorToggle,
    ItemPickup,
    DangerAlert,
    NewEntityLabel,
}
```

계약:

- `UiEffectEvent`는 `GameEvent`에서 파생되지만 `TurnOutcome.snapshot_hash`에 포함하지 않는다.
- `UiInputEvent::MouseHover`, `FocusPanel`, `InspectMap`은 턴을 진행하지 않는다.
- `UiCommandCandidate::Submit`만 core로 전달된다.
- `reduced_motion=true`이면 모든 effect duration은 `0ms` 또는 단일 frame flash로 축소한다.

### 15.6 실제 UI 기본값

```toml
[ui]
enable_mouse = true
enable_animations = true
frame_rate = 30.0
tick_rate = 8.0
reduced_motion = false
color_profile = "Default"
min_width = 80
min_height = 28
recommended_width = 100
recommended_height = 32

[ui.labels]
max_simultaneous_labels = 3
new_entity_label_ms = 1200
danger_label_ms = 1600

[ui.effects]
damage_flash_ms = 120
heal_pulse_ms = 160
miss_wisp_ms = 80
door_toggle_ms = 100
item_pickup_ms = 120
danger_alert_ms = 400
```

### 15.7 단계별 구현 로드맵

| Phase | 이름 | 완료 기준 |
| --- | --- | --- |
| 10A | 기본 TUI 안정화 | 80x28에서 map/status/log/debug 표시, 키보드 명령 연결, snapshot test 통과 |
| 10B | 정보 가독성 개선 | hover/inspect 모델, priority message, 위험/상태 강조, command hint 표시 |
| 10C | 마우스 입력 | map click to inspect/move, inventory click select, panel focus, mouse event 좌표 매핑 테스트 |
| 10D | ASCII 효과 | core event 기반 flash/pulse/label effect, reduced motion, replay hash 무영향 검증 |
| 10E | 고급 UX 후보 | drag/drop inventory와 자동 라벨 우선순위, 설정 파일 기반 UI 옵션 |

### 15.8 검증 기준

```bash
cargo test ui_layout
cargo test ui_input_mapping
cargo test ui_effect_projection
cargo test replay_hash_ignores_ui_effects
cargo run --bin aihack-headless -- --seed 42 --turns 1000
```

수동 검증:

- 80x28, 100x32, 120x36에서 텍스트 겹침 없음.
- mouse disabled 환경에서도 모든 필수 명령을 키보드로 실행 가능.
- reduced motion 설정 시 애니메이션 없이도 damage/heal/danger 정보를 색상 또는 텍스트로 확인 가능.
- hover/inspect와 panel focus는 턴을 진행하지 않음.
