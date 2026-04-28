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
