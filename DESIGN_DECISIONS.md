# AIHack Design Decisions

문서 상태: active
작성일: 2026-04-28

## ADR-0001: 기존 NetHack Rust 포트를 레거시 참조 베이스로 격리한다

배경:

이전 코드베이스는 NetHack 3.6.7 C 구조를 Rust로 옮기는 과정에서 큰 진척을 만들었지만, 실행 경로가 `game_loop.rs`, Legion `World/Resources`, `ActionQueue`, `EventQueue`, Grid 복사/역동기화에 분산되었다. 문서와 코드 모두 런타임 borrow conflict, 미연결 섬 코드, Grid 이중화 리스크를 기록하고 있다.

결정:

이전 포트 전체를 `legacy_nethack_port_reference/`로 이동하고 새 런타임은 루트에서 새로 설계한다.

기각:

- 기존 구조를 계속 보수: 같은 종류의 런타임 충돌과 연결 부채가 반복될 가능성이 높다.
- 기존 코드를 삭제: 테스트, 데이터, 규칙 지식 손실이 크다.

결과:

레거시는 참조 자산이고 새 엔진의 빌드 대상이 아니다.

## ADR-0002: 새 런타임은 Legion ECS를 초기 스택에서 제외한다

배경:

Legion은 대량 엔티티 처리에는 유용하지만, 이전 프로젝트에서 런타임 borrow conflict가 반복되었다. 현재 목표는 최대 성능보다 AI 연결과 결정론적 디버깅이다.

결정:

v0.1은 `GameSession`이 소유하는 explicit entity store를 사용한다. ECS 재도입은 v0.3 이후 performance audit에서만 검토한다.

기각:

- Legion 유지: 이전 구조적 문제를 새 엔진으로 다시 가져올 위험이 크다.
- 즉시 Bevy/Specs 도입: UI와 시뮬레이션 경계가 복잡해지고 초기 목표를 흐린다.

결과:

엔티티는 `EntityId`와 typed component-like structs로 관리한다.

## ADR-0003: 단일 상태 원천은 `GameSession`이다

배경:

이전 포트는 `self.game.grid`와 `resources.Grid`가 분리되어 렌더링과 시스템 상태를 동기화해야 했다. 이 구조는 AI observation과 replay 검증에 부적합하다.

결정:

모든 게임 상태는 `GameSession` 아래에 둔다. UI와 AI는 snapshot/observation만 읽는다.

기각:

- UI별 상태 복사본 유지: 동기화 버그를 만든다.
- 시스템별 mutable singleton: 테스트와 replay를 어렵게 만든다.

결과:

상태 변경은 `GameSession::submit(CommandIntent)`로만 발생한다.

## ADR-0004: AI는 `Observation`과 `ActionSpace`를 통해서만 연결한다

배경:

AI가 자유 텍스트로 게임을 조작하거나 내부 상태를 직접 읽으면 문제 원인 분리가 어렵다.

결정:

AI는 매 턴 `Observation`을 받고, `ActionIntent`를 제출한다. 엔진은 legal action validator를 통과한 명령만 실행한다.

기각:

- LLM이 직접 Rust API 호출: 안정성/보안/재현성 부족
- 화면 OCR 기반 AI: typed state가 있는데 일부러 불안정한 경로를 만들 필요가 없다.

결과:

LLM은 초기에는 narrative only이며, decision support는 Phase 13에서 제한적으로 도입한다.

## ADR-0005: NetHack 1:1 복제보다 안정적 AIHack을 우선한다

배경:

NetHack의 전체 규칙은 방대하다. 이전 포트 문서에도 `cmd.c`, `invent.c`, `sp_lev.c`, `trap.c`, `zap.c`, `shk.c` 등의 미완/부분 구현 항목이 남아 있다.

결정:

AIHack은 NetHack-inspired 독립 런타임으로 시작한다. 핵심 재미를 살리는 규칙은 golden test로 선별 흡수한다.

기각:

- 전체 NetHack parity를 v0.1 목표로 설정: 구현 착수와 검증을 막는다.
- NetHack 요소 전부 제거: 기존 프로젝트 정체성과 축적된 지식 손실이 크다.

결과:

문서와 코드에서 "NetHack 100% 포트"라는 목표를 새 루트 프로젝트의 성공 기준으로 사용하지 않는다.

## ADR-0006: Headless runner가 첫 번째 실행 대상이다

배경:

이전 프로젝트는 GUI/TUI와 core가 강하게 얽혀 실제 버그 원인 분리가 어려웠다. AI 연결도 UI 없이 상태를 검증할 수 있어야 한다.

결정:

첫 실행 대상은 `aihack-headless`이다. UI는 headless core가 deterministic 검증을 통과한 뒤 adapter로 붙인다.

기각:

- UI 먼저 구현: 빠르게 보이지만 core 계약이 흔들린다.
- 테스트만 구현하고 runner 생략: replay와 AI 실험 경로가 부족하다.

결과:

`cargo run --bin aihack-headless -- --seed 42 --turns 1000`는 모든 주요 Phase의 기본 검증 명령이다.

## ADR-0007: 현대 TUI는 단계형 혼합 로드맵으로 선반영한다

배경:

프로젝트는 headless deterministic core를 우선하지만, 최종 플레이 경험은 ASCII 우선 TUI다. 외부 그라운딩에서 Cogmind는 전체 키보드/마우스 접근, drag/drop, 자동 라벨, ASCII 효과를 통해 전통 ASCII를 현대 UX로 확장했고, Brogue는 단순한 ASCII에서도 키보드/마우스 혼합 조작과 도움말로 접근성을 확보한다. AIHack은 두 방향을 모두 참고하되 core 안정성과 AI 관찰 계약을 훼손하면 안 된다.

결정:

TUI 리팩토링은 단계형 혼합 로드맵으로 문서에 선반영한다. v0.1은 정보 가독성과 안전한 기본 TUI, v0.2는 Brogue식 접근성, v0.3은 Cogmind식 고급 ASCII 효과와 마우스 UX를 목표로 한다. 모든 애니메이션과 효과는 `UiEffectEvent` presentation layer로 제한하고 replay hash에 포함하지 않는다.

기각:

- Cogmind급 전체 UX를 v0.1 완료 기준으로 설정: headless core 우선 원칙과 초기 구현 안정성을 해친다.
- 그래픽 타일셋을 병행 도입: ASCII 우선 정체성과 현재 문서 범위를 흐린다.
- UI effect를 core event/state에 포함: deterministic replay와 AI observation 경계를 오염시킨다.

결과:

`spec.md`, `designs.md`, `implementation_summary.md`, `audit_roadmap.md`는 현대 TUI 계획을 같은 단계와 검증 기준으로 참조한다. 후속 구현자는 UI 편의 기능을 추가하더라도 `GameSession` 단일 상태 원천과 `CommandIntent` 경계를 유지해야 한다.

추가 메모 (2026-05-18):

Phase 15에서는 v0.2 범위를 넓히지 않기 위해 inspect panel이 hover read-only inspect와 inventory primary-action surface를 함께 맡는다. 즉, 별도 drag/drop inventory나 동적 레이아웃 저장을 도입하지 않고도 layout/hit-test contract를 유지한 채 keyboard baseline과 mouse mixed-input parity를 맞춘다.

## ADR-0008: Phase 1 replay hash는 고정 FNV-1a로 계산하고 RNG 업그레이드를 replay 영향 변경으로 취급한다

배경:

Phase 1 Headless Core는 같은 seed와 같은 명령열이 같은 final hash를 만드는지를 첫 검증 기준으로 삼는다. Rust `DefaultHasher`는 버전/구현 안정성을 장기 replay 계약으로 쓰기에 부적합하고, `StdRng` sequence는 `rand` crate 버전에 묶인다.

결정:

Phase 1 snapshot hash는 `GameSnapshot`을 안정적인 JSON 문자열로 직렬화한 뒤 자체 FNV-1a 64-bit 함수로 계산한다. RNG는 `GameRng` wrapper가 `StdRng::seed_from_u64(seed)`를 소유하며, `Cargo.lock`에 고정된 `rand` 버전을 replay 계약의 일부로 본다.

기각:

- `DefaultHasher` 사용: 플랫폼/버전별 안정 계약이 약하다.
- 암호학적 hash crate 추가: Phase 1 범위와 의존성 최소화 원칙에 비해 과하다.
- 각 시스템에서 `rand` 직접 사용: deterministic 경계를 흩뜨린다.

결과:

`seed=42`, `turns=100`의 Phase 1 기준 final hash는 `f827bc2d4155ef66`이다. 향후 `rand` 업그레이드 또는 snapshot hash 입력 변경은 replay/hash 영향 변경으로 취급하고, 변경 전후 기준 hash를 문서와 테스트에 함께 갱신해야 한다.

## ADR-0009: Phase 2는 전체 entity store 대신 Minimal World로 map/movement/vision을 완성한다

배경:

Phase 2는 40x20 fixture map, movement blocker, open/close door, LOS radius 8, `Observation.visible_tiles`를 완료해야 한다. 하지만 full entity store, combat, item, monster AI는 후속 Phase 범위다.

결정:

`GameSession`에 `GameWorld { map, player_pos }`를 추가하고, player position만 최소 상태로 보관한다. `GameMap`은 row-major tile vector를 소유하고, movement/doors/vision은 모두 `GameSession::submit()` 경계에서만 상태를 바꾼다. Snapshot hash에는 player position과 map tile state를 포함한다.

기각:

- Map-only 구현: Phase 2 movement/doors/vision 완료 기준을 만족하지 못한다.
- Full entity store 선행 도입: Phase 3+ 범위를 끌어와 scope creep을 만든다.
- UI/TUI에서 map을 직접 조작: `GameSession` 단일 상태 원천 원칙을 위반한다.

결과:

Phase 2 기준 `seed=42`, `turns=100`의 final hash는 `1aad6f4049778b0e`이다. Phase 1 hash와 달라진 이유는 snapshot hash 입력에 map/player/door state가 포함되었기 때문이다. 후속 entity store 도입 시 `GameWorld.player_pos`의 이전 경계를 별도 PRD/ADR에 기록해야 한다.


## ADR-0010: Phase 3는 Minimal EntityStore로 전투/사망만 완성한다

배경:

Phase 3는 Phase 2 map/movement/doors/vision 위에 jackal/goblin 전투와 사망 event를 추가해야 한다. 동시에 item/inventory, monster AI, TUI를 끌어오면 deterministic core 검증 범위가 흐려진다.

결정:

`GameWorld`를 `map + EntityStore + player_id` 구조로 확장하고, player position은 player entity의 `pos`를 단일 원천으로 삼는다. `Move(Direction)` 대상에 살아있는 hostile entity가 있으면 movement 대신 bump attack으로 분기한다. Combat 공식은 PRD의 d20 명중식과 damage clamp를 그대로 사용하고, 사망은 `EntityDied` event와 `alive=false` tombstone으로 기록한다.

기각:

- Full ECS/Legion 도입: Phase 3 범위보다 크고 현재 문서의 의존성 최소화 원칙에 맞지 않는다.
- Monster AI/반격 동시 구현: player death 검증은 가능하지만 턴 루프 복잡도가 Phase 6 범위를 침범한다.
- Item/equipment entity 선행 도입: Phase 4 범위를 끌어와 전투 공식 검증을 불안정하게 만든다.

결과:

Phase 3 기준 `seed=42`, `turns=100`의 final hash는 `8b20a23301eea977`이다. Phase 2 hash와 달라진 이유는 snapshot hash 입력에 entity id/kind/position/hp/alive 상태가 포함되었기 때문이다. 후속 Phase 4에서 장비/아이템을 추가할 때 player 내장 dagger profile을 실제 item/equipment 계약으로 마이그레이션해야 한다.

## ADR-0011: Phase 4는 EntityPayload 분리와 Inventory.equipped_melee로 item/equipment를 통합한다

배경:

Phase 4는 item pickup, stable inventory letter, dagger wield, healing potion quaff를 구현해야 한다. Phase 3의 `Entity`는 actor-only 구조였기 때문에 item이 actor stats나 alive flag를 잘못 갖지 않도록 payload 분리가 필요했다.

결정:

`EntityPayload::Actor | EntityPayload::Item`으로 actor와 item을 분리하고, item location은 `OnMap`, `Inventory`, `Consumed`로 고정한다. Inventory letter는 item payload의 `assigned_letter`로도 보존하며, consumed item은 inventory entry에서 제거하되 `assigned_letter`를 유지한다. 장비 상태의 단일 원천은 `Inventory.equipped_melee`다.

기각:

- 별도 ItemStore: EntityId/event/snapshot 통합성이 약하다.
- InventoryEntry별 equipped flag: 장비 상태가 중복되어 불일치 위험이 있다.
- File save/load 동시 구현: Phase 9 범위 침범이다.

결과:

Phase 4 기준 `seed=42`, `turns=100`의 final hash는 `00ba578d933177f2`이다. 후속 Phase 5+에서 level 이동 또는 save/load를 구현할 때 `EntityLocation`과 `assigned_letter` 계약을 유지해야 한다.


## ADR-0012: Phase 5는 fixed LevelRegistry와 level-aware EntityLocation으로 계단 왕복을 구현한다

배경:

Phase 5는 1층-2층 왕복, 1층 상태 보존, current level snapshot hash 반영을 구현해야 했다. 단일 `GameWorld.map` 백업 방식은 item/actor 위치와 후속 Phase 6 monster AI 경계를 모호하게 만들 수 있었다.

결정:

`GameWorld`는 `LevelRegistry { levels: Vec<GameLevel> }`, `current_level`, 단일 `EntityStore`, `Inventory`를 가진다. `LevelRegistry`는 Phase 5에서 fixed `main:1 -> main:2` 순서를 유지한다. actor/item 위치는 `EntityLocation::OnMap { level, pos }`로 통일하고, player transition은 `set_player_location(level, pos)`로 current level과 actor location을 atomic하게 갱신한다.

기각:

- 단일 map 백업: snapshot/entity 위치 계약이 불명확하다.
- level별 `EntityStore`: `EntityId` 재사용과 event/snapshot 통합이 복잡해진다.
- procedural generation 또는 save/load 동시 구현: Phase 5 범위 밖이다.

결과:

`Descend`/`Ascend`는 stairs tile에서만 accepted되고 `TurnStarted -> LevelChanged` event order를 만든다. Phase 5 기준 `seed=42`, `turns=100` final hash는 `88886c28698a1730`이다. Phase 6 monster AI는 반드시 level-aware query helper를 사용해야 한다.

## ADR-0009: Phase 6 monster AI는 current-level deterministic 후처리로 제한한다

Context:

Phase 5에서 level registry와 current-level invariant가 도입되면서 monster AI를 바로 all-level simulation으로 확장하면 stairs 추적, persistence, pathfinding이 한 번에 들어온다. 하지만 Phase 6 목표는 최소 monster turn loop와 deterministic replay 유지다.

Decision:

monster AI는 `GameSession`의 accepted-turn 공용 후처리에서만 실행한다. 대상은 `world.current_level()`의 살아있는 hostile monster이며, 순서는 `EntityId` 오름차순이다. jackal=`Wander`, goblin=`ChaseVisiblePlayer`, floating eye=`Stationary`로 고정한다. `EntityMoved`는 player/monster 모두 `entity`를 포함하도록 확장한다.

Consequences:

- `ShowInventory` 같은 no-turn command와 rejected command는 monster phase를 트리거하지 않는다.
- player side resolution으로 `GameOver`가 되면 남은 monster action은 중단된다.
- stairs traversal, door manipulation, memory/pathfinding, ranged/passive ability는 Phase 7+로 미룬다.
- Phase 6 기준 `seed=42`, `turns=100` final hash는 `2fb549b5d2e1e67f`이며 `seed=43`, `turns=100` final hash는 `ec98b802759e109c`다.


## ADR-0010: Phase 7 상호작용은 fixture-driven 최소 typed contract로 제한한다

Context:

Phase 6까지의 deterministic core 위에 NetHack flavor를 추가해야 하지만, generalized projectile/trap/identify framework를 먼저 만들면 snapshot/event/item taxonomy 변경이 너무 넓어진다.

Decision:

Phase 7은 `Search`, `Throw`, `Zap`, `Read`, `Trap(Pit)`만 current-level fixture 계약으로 구현한다. hidden tile은 `HiddenDoor`/`HiddenTrap(Pit)`로 표현하고, trap 피해는 fixed `3`으로 고정한다. `WandMagicMissile`은 charge를 갖고 `ScrollReveal`은 current level hidden tile 전체 reveal만 수행한다.

Consequences:

- replay/hash는 hidden/revealed tile, item charge, thrown item location까지 포함해야 한다.
- identify, reflection, bounce, hunger, curse, save/load, TUI는 Phase 8+로 미룬다.
- Phase 7 기준 `seed=42`, `turns=100` final hash는 `5aecd83cf284cb25`, `seed=43`, `turns=100` final hash는 `5f5d5b89faa9a834`다.


## ADR-0011: Phase 8 규칙 흡수는 golden scenario 중심 wave 실행으로 제한한다

Context:

레거시 규칙 20개를 한 번에 넓게 이식하면 deterministic closure와 regression 관리가 무너진다. 현재 repo는 phase-gated verification 문화가 강하므로 wave + golden scenario 접근이 더 적합하다.

Decision:

Phase 8은 20개 규칙을 4개 wave로 나누고, 각 규칙마다 최소 1개 golden scenario를 구현한다. 구현은 direct import가 아니라 typed helper/command/state 재구성으로 제한한다.

Consequences:

- golden tests가 문서/제품 계약 역할을 겸한다.
- Phase 8 기준 `seed=42`, `turns=100` final hash는 `4c77dafb19dd2226`, `seed=43`, `turns=100` final hash는 `f8324eacbce50087`이다.
- Phase 9 save/load는 identified_items, prayer_cooldown, nutrition, gold, kill_count 등 신규 persistent state를 모두 serialization 대상으로 재검토해야 한다.


## ADR-0012: Phase 9 persistence는 explicit schema와 replay artifact를 분리한다

Context:

Phase 8까지 state가 커지면서 raw session dump는 schema control과 auditability가 약해졌다. 동시에 replay는 debugging artifact로서 save file과 역할이 다르다.

Decision:

Phase 9는 `SaveDataV1`/`SavedWorldV1` explicit schema와 `ReplayLineV1` JSONL line schema를 분리한다. RNG는 `RngStateV1 { seed, draws }`로 continuation을 복원한다.

Consequences:

- save/load correctness는 snapshot hash equality와 continuation equality로 검증된다.
- replay JSONL은 command/outcome trace용 artifact로 유지된다.
- Phase 9 기준 `seed=42`, `turns=100` final hash는 `4c77dafb19dd2226`, `seed=43`, `turns=100` final hash는 `f8324eacbce50087`이다.


## ADR-0013: Phase 10 TUI는 adapter boundary와 layout-driven hit test를 고정한다

Context:

headless core와 persistence가 완료된 상태에서 실제 플레이 가능한 terminal UI가 필요하지만, UI가 core state를 직접 만지면 지금까지의 deterministic architecture가 무너진다.

Decision:

Phase 10은 `src/ui/tui/*` 분리형 adapter를 도입하고, layout/input/effect를 별도 모듈로 유지한다. UI는 `Observation`/`GameSnapshot`을 읽고 `CommandIntent`만 제출한다. mouse hit-test는 layout/viewport contract를 렌더와 공유한다.

Consequences:

- 80x28 degraded layout과 keyboard-only parity가 Phase 10의 최소 closure가 된다.
- replay hash는 UI effect projection과 무관해야 한다.
- Phase 11 AI API freeze는 TUI와 별개로 observation schema 안정화를 다룬다.


## ADR-0014: Phase 11은 explicit AI DTO와 fixture-locked schema freeze를 사용한다

Context:

Observation/legal_actions가 내부 구현과 너무 가깝게 붙어 있으면 future refactor가 public breakage로 이어진다. save/load와 TUI도 같은 AI-facing contract를 공유해야 한다.

Decision:

Phase 11은 `Observation`, `PlayerObservation`, `EntityObservation`, `ActionSpace`, `ActionIntent`를 explicit AI DTO로 고정하고 canonical schema tests로 freeze한다. `legal_actions`는 compatibility alias로 유지하되 public future surface는 `action_space`가 주도한다.

Consequences:

- TUI와 headless/save-load는 같은 schema를 소비해야 한다.
- future breaking change는 version bump와 migration note가 필요하다.
- Phase 12/13는 이 frozen AI API 위에서만 확장해야 한다.


## ADR-0015: Phase 12 narrative layer는 provider-agnostic timeout/fallback adapter로 제한한다

Context:

AI API freeze 이후에도 LLM integration은 core determinism과 분리되어야 한다. narrative text는 가치가 있지만 provider failure가 gameplay를 막으면 안 된다.

Decision:

Phase 12는 provider-agnostic narrative adapter와 2초 timeout, deterministic fallback policy를 도입한다. narrative response는 presentation artifact이며 snapshot hash, save/load, replay에 포함되지 않는다.

Consequences:

- no-provider mode와 provider failure는 모두 fallback text로 degrade 된다.
- Phase 13 decision support는 narrative layer와 별도 safety policy가 필요하다.
- TUI/log consumer는 same narrative response envelope를 읽는다.


## ADR-0016: Phase 13 decision support는 suggestion envelope와 validator gate를 분리한다

Context:

narrative-only layer 이후 user utility를 높이기 위해 command suggestion이 필요하지만, LLM이 core command를 직접 실행하면 boundary가 무너진다.

Decision:

Phase 13은 suggestion envelope와 validator-gated execution을 분리한다. provider는 legal/illegal suggestion을 생성할 수 있지만, 실제 실행은 `ActionSpace` membership와 기존 submit path를 통과한 경우에만 허용된다.

Consequences:

- suggestion metadata는 persistence truth가 아니다.
- timeout/failure는 fallback/disabled suggestion으로 degrade 된다.
- autonomous play는 별도 safety layer 없이는 허용되지 않는다.


## ADR-0017: Phase 16에서 RunState를 spec.md 8.2 계약과 일치시킨다

Context:

Phase 15 완료 시점에서 실제 코드의 `RunState`는 `Playing`과 `GameOver` 2개 변이체만 존재했다. 하지만 `spec.md` 8.2에는 `Title`, `CharacterCreation`, `AwaitingDirection`, `AwaitingInventorySelection`, `MorePrompt`, `GameOver { cause, final_score }`가 명시되어 있었다. 이 불일치는 문서-구현 gap closure의 핵심 대상이었다.

Decision:

Phase 16에서 `RunState`를 spec.md 8.2 계약과 완전히 일치시킨다. `GameOver`는 bare variant에서 `GameOver { cause: DeathCause, final_score: i32 }`로 확장하고, 나머지 5개 변이체도 모두 추가한다. `CommandIntent`에는 `AcknowledgeMore`를 추가하고, `DirectionalAction`과 `InventoryAction`을 신규 정의한다.

기각:

- `RunState`를 현재 2개로 유지하고 TUI에서만 별도 상태 관리: spec.md "spec is law" 원칙 위반
- `GameOver`에 필드 추가 없이 TUI에서만 사망 정보 표시: core-UI 경계 원칙 위반, observation에 사망 정보 누락
- `GameSession::new()`를 Playing으로 유지: Title 화면 구현 시 core 상태와 UI 상태 불일치

Consequences:

- `GameSession::new()`는 `Title` 상태로 시작하므로 기존 테스트/headless runner에 영향이 있다. 호환성을 위해 `new_for_playing()`을 추가한다.
- `submit()`이 상태별 분기를 처리하므로 기존 단일 match arm에서 7개 상태 분기로 확장되었다.
- snapshot hash가 변경되었다. Phase 16 기준 `seed=42 turns=1000` final hash는 `569bc36895258349`이다.
- `GameEvent::Message { priority, text }`와 `MessagePriority` enum이 추가되어 TUI 메시지 로그의 중요도 표시 계약이 완성되었다.


## ADR-0018: Phase 16에서 snapshot hash 변경을 수용한다

Context:

Phase 16의 `RunState` 확장과 `MessagePriority` enum 추가로 인해 `GameSnapshot`의 serde_json 직렬화 결과가 변경되었다. 이로 인해 `seed=42 turns=1000` 기준 hash가 `4c77dafb19dd2226`에서 `569bc36895258349`로 변경되었다.

Decision:

snapshot hash 변경을 수용하고, 새로운 기준 hash를 문서와 테스트에 갱신한다. hash 변경의 직접적인 원인은 `RunState` enum 확장으로 인한 serde 처리 변경과 `GameEvent` enum에 `Message` variant 추가로 인한 직렬화 결과 변화이다.

기각:

- hash 변경을 방지하기 위해 RunState/MessagePriority를 별도 모듈로 분리: serde_json이 enum variant 이름을 사용하므로, 동일 모듈 내 variant 추가/확장은 여전히 직렬화 결과에 영향을 줄 수 있다.
- 이전 hash를 강제로 유지하기 위해 GameSnapshot 직렬화를 커스텀으로 고정: 유지보수성 저하, serde_json의 안정성 보장을 무시함.

Consequences:

- `tests/release_candidate.rs`의 기준 hash 3종을 모두 갱신해야 한다.
- `cargo run --bin aihack-headless -- --seed 42 --turns 1000`의 출력 hash가 변경된다.
- 향후 Phase 17~20에서 추가 UI-only 변경은 hash에 영향을 주지 않아야 한다 (GameSnapshot에 UI 상태 불포함 원칙 유지).


## ADR-0019: Phase 19에서 자동 라벨은 UI-only 상태로 수집하고 렌더링한다

Context:

Phase 18 완료 후 TUI는 headless core와 격리되어 있지만, 플레이어가 시야 내 새로운 위협/아이템을 즉시 인지하는 UX가 필요하다. spec.md 15.4에 자동 라벨 우선순위와 최대 3개 제한이 명시되어 있다.

Decision:

자동 라벨은 `TuiApp.active_labels` Vec으로 UI-only 상태를 관리하고, `collect_auto_labels()`가 매 턴 `Observation`을 읽어 라벨을 생성한다. 라벨 수집은 턴이 진행된 경우에만 실행되며, `MapWidget` 렌더링 시 overlay로 표시한다. 라벨 정보는 snapshot hash, save, replay에 포함되지 않는다.

기각:

- 라벨 정보를 `GameWorld`에 포함: core-UI 경계 원칙 위반, deterministic hash에 영향
- 라벨을 `GameEvent`로 생성: presentation layer와 core event의 분리 원칙 위반
- 라벨 수집을 매 프레임이 아니라 매 턴에만 실행: 프레임 누락은 gameplay 누락이 아니므로, 턴 기준 수집으로 충분

Consequences:

- `tests/ui_labels.rs`가 라벨 수집/우선순위/만료를 검증한다.
- headless baseline hash에 영향이 없다.
- `UiEffectKind::NewEntityLabel`은 effect projection 용도만 사용된다.


## ADR-0020: Phase 20에서 상태 필드는 getter/setter로 분리하고 외부 데이터는 TOML로 이동한다

Context:

spec.md 7의 디렉터리 구조에는 `src/domain/status.rs`와 `src/data/`가 예정되어 있었다. 현재 `GameWorld`에는 nutrition, luck, prayer_cooldown 등 상태 필드가 산재해 있고, 아이템/몬스터/레벨 데이터는 하드코딩되어 있다.

Decision:

`Status` struct와 `HungerState` enum을 `src/domain/status.rs`에 생성하고, `GameWorld`에는 기존 개별 필드를 유지하되 `status()`/`set_status()` getter/setter를 추가한다. 외부 데이터는 `items.toml`, `monsters.toml`, `levels/main_1.toml`로 이동하고, `src/data/mod.rs`에 `include_str!` 기반 로더를 구현한다.

기각:

- `GameWorld` 필드를 `Status`로 완전 대체: snapshot hash 입력 변경으로 인해 Phase 16 기준값이 깨진다
- 외부 데이터를 별도 크레이트로 분리: Phase 20 범위를 벗어나며, 현재는 단일 바이너리가 모든 데이터를 포함해야 한다
- 동적 파일 IO 대신 `include_str!`: TOML 데이터는 빌드 시점에 고정되며, 런타임 파일 IO는 불필요한 복잡도를 추가한다

Consequences:

- `Status`는 getter/setter를 통해서만 접근하고, 개별 필드는 snapshot/hash 호환성을 위해 유지된다.
- TOML 데이터는 `include_str!`로 빌드에 임베드되어 별도 배포 파일 없이 동작한다.
- `tests/data_loading.rs`가 TOML 파싱과 Status/HungerState 계산을 검증한다.
- headless baseline hash에 영향이 없다 (`GameWorld` 필드 구조 유지).
