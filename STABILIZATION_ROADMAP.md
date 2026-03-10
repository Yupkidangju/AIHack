# 🛡️ AIHack 안정화 로드맵 v2.0 (STABILIZATION_ROADMAP)

**버전**: v2.3
**갱신일**: 2026-03-10
**이전 버전**: v2.2 (2026-03-07) — Phase E2 23/30 시스템 전환 완료
**이전 버전**: v2.1 (2026-03-06) — Phase E2a/E2b 13/30 시스템 전환 완료
**이전 버전**: v1.0 (2026-02-28) — Phase S0~S6 완료, S7 대기 상태
**대상 브랜치**: `stabilize/e2e-playable`
**전제**: v2.42.1 = Phase S0~S6 완료. E2E 핵심 동사 및 Edge Case 전량 통과.
**목표**: Legion `#[system]` 매크로 제거 → `&mut World` 직접 접근 엔진으로 전환 + LLM 통합 아키텍처 확립

---

## 0. v2.0 로드맵 전면 개편 사유

### 0.1 발견된 구조적 문제

v1.0 로드맵의 Phase S3(첫 턴 생존)에서 바이너리 서치 방식으로 시스템을 하나씩 활성화하며 `AccessDenied` 패닉을 수정하는 접근이 **구조적으로 한계에 도달**했다.

**근본 원인**: Legion ECS의 `entry_ref(entity)` / `entry_mut(entity)` 호출 시, 해당 엔티티의 **아키타입 전체**에 대한 읽기/쓰기 권한이 `#[read_component]` / `#[write_component]`로 선언되어 있어야 함. 엔티티에 부착된 컴포넌트가 10~15개인 상황에서 하나라도 누락하면 런타임 패닉이 발생.

**한계**:
1. **비결정적**: RNG 시드에 따라 다른 코드 경로가 실행되어 같은 시스템 수에서도 패닉 발생 여부가 다름
2. **조합 폭발**: 시스템 A+B 개별 통과해도 동시 실행 시 아키타입 잠금 충돌 가능
3. **수정해도 재발**: 같은 패턴(entry_ref/entry_mut 사용 + 컴포넌트 선언 누락)이 25개+ 파일에 반복
4. **턴 기반 게임에 불필요한 병렬화 오버헤드**: 순차 실행이면 충분한 게임에 병렬 스케줄링 적용 중

### 0.2 핵심 통찰

NetHack은 **턴 기반 게임**이다. 병렬 실행이 불필요하다.
Legion의 Schedule + SubWorld + 컴포넌트 권한 선언은 **병렬 실행을 위한 안전 장치**이다.
우리에게는 이 장치가 필요 없으며, 오히려 **AccessDenied 패닉의 원인**이 되고 있다.

### 0.3 전략 전환

**기존 (v1.0)**: 각 시스템의 컴포넌트 선언을 일일이 수정하여 AccessDenied 제거
**신규 (v2.0)**: `#[system]` 매크로 자체를 제거하고 `&mut World` 직접 접근으로 전환

`&mut World`에서는 `entry_ref`/`entry_mut`에 **권한 검사가 없다**.
따라서 **AccessDenied가 구조적으로 영원히 불가능**해진다.

### 0.4 이미 존재하는 성공 사례

`src/core/systems/social/interaction.rs`는 **이미 `&mut World` 직접 접근 + InteractionProvider(LLM 교체 포인트) 패턴**을 사용 중:

```rust
pub fn try_open_door(
    world: &mut World,                        // ← SubWorld가 아닌 World 직접
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
    direction: Direction,
    provider: &dyn super::InteractionProvider, // ← LLM 교체 포인트
) -> bool { ... }
```

이 패턴을 **31개 전체 시스템에 확산**하는 것이 본 로드맵의 핵심.

---

## 1. 신규 Phase 구조 (5단계)

```
Phase E1: GameContext 구조체 정의 + 턴 엔진 골격 (기반 구축)
Phase E2: 시스템 전환 — #[system] → 일반 함수 (31개, 점진적)
Phase E3: 전체 통합 테스트 + 안정성 검증
Phase E4: LLM 통합 아키텍처 확립 (AIProvider trait)
Phase E5: LLM 실제 주입 + 대화/내러티브/적응형 AI
```

> **E = Engine Redesign**

---

## 2. Phase 상세

### Phase E1: GameContext 구조체 + 턴 엔진 골격 ✅ **완료 (2026-03-05)**

**목표**: 모든 시스템이 공유할 통합 컨텍스트 구조체를 정의하고, 수동 시스템 호출 루프를 구축

**완료 사항**:
- [x] `GameContext` 구조체 정의 (`src/core/context.rs`)
  ```rust
  pub struct GameContext<'a> {
      pub world: &'a mut World,
      pub grid: &'a mut Grid,
      pub log: &'a mut GameLog,
      pub rng: &'a mut NetHackRng,
      pub turn: u64,
      pub cmd: Command,
      pub assets: &'a AssetManager,
      pub event_queue: &'a mut EventQueue,
      pub action_queue: &'a mut ActionQueue,
      pub vision: &'a mut VisionSystem,      // [E2c] 추가
      pub level_req: &'a mut Option<LevelChange>, // [E2c] 추가
      pub dungeon: &'a Dungeon,              // [E2c] 추가
  }
  ```
- [x] `TurnRunner` 구조체 정의 (Schedule 대체)
- [x] `game_loop.rs`에서 Schedule 실행 후 GameContext 순차 호출부 구축
- [ ] `AIProvider` trait 정의 (→ Phase E4로 이동)
- [ ] `DefaultAIProvider` 구현 (→ Phase E4로 이동)

**판정 결과**: ✅ GameContext + TurnRunner 정의 완료 + 컴파일/테스트 성공

---

### Phase E2: 시스템 전환 (30개 → 일반 함수) — ✅ **30/30 완료 (100%) (2026-03-10)**

**목표**: 모든 `#[system]` / `#[legion::system]` 매크로 시스템을 `fn system_name(ctx: &mut GameContext)` 시그니처로 전환

**전환 상태 (2026-03-10 최종)**:

| # | 파일 | 시스템 | 상태 | 비고 |
|---|------|--------|------|------|
| 1 | timeout.rs | timeout_dialogue | ✅ | E2a |
| 2 | luck.rs | luck_maintenance | ✅ | E2a |
| 3 | status.rs | status_tick | ✅ | E2a |
| 4 | attrib.rs | attrib_maintenance | ✅ | E2a |
| 5 | item_tick.rs | item_tick | ✅ | E2a, Gather-Apply |
| 6 | regeneration.rs | regeneration | ✅ | E2a |
| 7 | regeneration.rs | monster_regeneration | ✅ | E2a |
| 8 | engrave.rs | engrave_tick | ✅ | E2b |
| 9 | inventory.rs | autopickup_tick | ✅ | E2b |
| 10 | inventory.rs | inventory_action | ✅ | E2b |
| 11 | evolution.rs | evolution_tick | ✅ | E2b, Gather-Apply |
| 12 | evolution.rs | lycanthropy_tick | ✅ | E2b, Gather-Apply |
| 13 | weight.rs | update_encumbrance | ✅ | E2b, EntityStore 제네릭 |
| 14 | ai/core.rs | pet_hunger | ✅ | E2c, 간단 전환 |
| 15 | equipment.rs | update_player_stats | ✅ | E2c, Gather-Apply |
| 16 | spell.rs | spell_cast | ✅ | E2c, 헬퍼 SubWorld→World |
| 17 | vision_system.rs | vision_update | ✅ | E2c, VisionSystem 추가 |
| 18 | vision_system.rs | magic_map_effect | ✅ | E2c, world.entry() |
| 19 | item_use.rs | item_input | ✅ | E2c, Gather 패턴 |
| 20 | equipment.rs | equipment | ✅ | E2c, **필드 분해 패턴** |
| 21 | throw.rs | throw | ✅ | E2c, **command_buffer 제거** |
| 22 | teleport.rs | teleport | ✅ | E2c, LevelChange |
| 23 | stairs.rs | stairs | ✅ | E2c, SystemBuilder 제거 |
| 24 | trap.rs | trap_trigger | ✅ | E2d |
| 25 | death.rs | death | ✅ | E2d, GameState |
| 26 | shop.rs | shopkeeper_update | ✅ | E2e, InteractionProvider |
| 27 | zap.rs | zap | ✅ | E2f, CommandBuffer→World |
| 28 | item_use.rs | item_use | ✅ | E2f, **deferred ops 패턴** |
| 29 | ai/core.rs | monster_ai | ✅ | E2f, 680줄 대형 전환 |
| 30 | movement.rs | movement | ✅ | E2f, 1310줄 대형 전환 |

**전환 절차 (각 시스템)**:
1. `#[system]` / `#[legion::system]` 매크로 제거
2. `#[read_component]` / `#[write_component]` 전체 제거
3. `world: &mut SubWorld` → `ctx.world` (GameContext 경유)
4. `#[resource] xxx: &Type` → `ctx.xxx` 직접 접근
5. `command_buffer` → `world.entry(ent).add_component()` 또는 `world.remove(ent)` 직접 조작
6. Schedule에서 제거 → GameContext 순차 호출부에 등록

**핵심 패턴 (2026-03-07)**:
- **Gather-Apply**: query borrow와 entry_mut borrow 충돌 → 데이터를 Vec에 수집 후 적용
- **필드 분해(destructure)**: `let GameContext { world, log, .. } = ctx;` → 각 필드를 독립 borrow
- **command_buffer 제거**: `add_component` → `world.entry(ent).add_component()`, `remove` → `world.remove(ent)`
- **Entry vs EntryMut**: `world.entry()` → `remove_component` ✅ / `world.entry_mut()` → `remove_component` ❌

**교훈 (2026-03-06~07)**:
- ⚠️ regex 대량 치환은 헬퍼 함수까지 오염시킴 → **한 시스템씩 수동 전환이 안전**
- `let world = &mut *ctx.world;` 패턴은 ctx 전체 borrow를 잡으므로, `ctx.필드` 직접 참조가 바람직
- Rust struct 다른 필드 동시 mutable borrow는 허용되므로 활용 가능

**판정 결과**: ✅ 30개 전체 전환 + `cargo check` 성공 (0 에러) + 4,178개 테스트 전체 통과

**실제 소요**: 약 15시간 (E2a~E2f, 5세션에 걸쳐 완료)

**추가 교훈 (2026-03-09~10)**:
- ⚠️ `CommandBuffer::flush`는 legion 0.4에서 `(world, resources)` 2개 인수 — `resources` 없이 불가 → 직접 제거
- ⚠️ deferred ops 패턴: 쿼리 `iter_mut` 빌림 중 `world.entry()` 호출 불가 → Vec에 수집 후 적용
- ⚠️ Schedule 완전 제거 시 `schedule.execute()` 호출도 함께 제거해야 함
- ✅ `ctx.cmd` 필드를 활용하면 action_queue에서 Move를 추출할 필요 없음

---

### Phase E3: 전체 통합 테스트 + 안정성 검증

**목표**: 전환 완료 후 모든 기존 테스트 + 퍼징 테스트 통과

**작업**:
- [ ] `e2e_stabilize.rs` 수정: `run_schedule_safe` → `TurnRunner::execute()` 호출
- [ ] `cargo test` 4,177개 전량 통과 확인
- [ ] `audit_command_fuzzing_100_turns` **전체 31개 시스템 활성화**로 통과
- [ ] `s3_incremental_system_activation` 테스트 업데이트
- [ ] 새 퍼징: 500턴, 1000턴 테스트 추가
- [ ] Panic Hook 설치: seed, turn, last_command, backtrace 기록
- [ ] 불변식 검사 추가: 턴 종료 시 HP >= 0, Entity 유효성, Position 범위 확인

**판정 기준**: 모든 테스트 통과 + 1000턴 퍼징 패닉 0건

**예상 소요**: 4~8시간

---

### Phase E4: LLM 통합 아키텍처 확립

**목표**: `GameContext`에 LLM 교체 포인트를 확장하여, 모든 시스템에서 LLM을 선택적으로 사용 가능한 구조 확립

**작업**:
- [ ] `AIProvider` trait 확장 (7개 교체 포인트):
  1. **대화/내러티브**: 몬스터 대사, NPC 대화 → `generate_monster_dialogue()`
  2. **몰입형 설명**: 전투 묘사, 던전 분위기 → `narrate_combat()`, `describe_atmosphere()`
  3. **적응형 AI**: 몬스터 전술 결정 → `decide_monster_action()` (선택적 오버라이드)
  4. **동적 이벤트**: 퀘스트/이벤트 자동 생성 → `generate_random_event()`
  5. **분위기 설명**: 새 층 진입, 특별한 방 → `describe_dungeon_level()`
  6. **아이템 설명**: 미확인 아이템 힌트 → `describe_unidentified_item()`
  7. **함정 대화**: 상점 흥정, 기도 응답 → 기존 `InteractionProvider` 확장
- [ ] `Snapshot` 구조체 정의: `PlayerSnapshot`, `MonsterSnapshot`, `GameStateSnapshot`
  - LLM에 전달할 읽기 전용 스냅샷 (World에서 추출, 비동기 안전)
- [ ] `ResponseCache` 구현: 비동기 LLM 호출 대비 캐시 패턴
- [ ] 기존 `InteractionProvider` 7곳을 `GameContext.provider` 경유로 통합

**핵심 설계 원칙**:
1. LLM은 항상 **선택적** (None이면 기본 NetHack 텍스트)
2. LLM 실패 시 **Fallback 보장** (게임 멈춤 절대 없음)
3. LLM 호출은 **게임 로직과 완전 분리** (순수 텍스트 생성만)
4. 모든 LLM 호출에 **타임아웃(2초)** 설정

**판정 기준**: `AIProvider` trait 완성 + 기존 테스트 무영향 + 컴파일 성공

**예상 소요**: 6~10시간

---

### Phase E5: LLM 실제 주입

> ⚠️ Phase E1~E4 전체 완료 후에만 진입 가능

**목표**: 실제 LLM 모델을 연결하여 게임 경험 향상

**주입 순서** (의존성 낮은 것부터):

| 순서 | 대상 | LLM 역할 | 실패 시 폴백 |
|------|------|----------|-------------|
| 1 | `death.rs` 묘비명 | 극적 사망 에필로그 | 기존 하드코딩 텍스트 |
| 2 | 전투 메시지 | 몰입형 전투 묘사 | `"You hit the {monster}."` |
| 3 | NPC 대화 | 오라클/상점주인 대사 | `ScriptedDialogue` |
| 4 | 던전 서술 | 레벨 진입 시 분위기 | 고정 텍스트 |
| 5 | 몬스터 AI | 전술적 AI 결정 | 기존 RuleBasedAi |
| 6 | 동적 이벤트 | 퀘스트/이벤트 자동 생성 | 생성 안함 |

**모델 전략**:
- **로컬 우선**: GGUF/ONNX 형태의 경량 LLM (1~7B)
- **오프라인 동작**: 클라우드 의존 없이 완전 로컬 추론
- **API 호환**: OpenAI-compatible API 지원 (llama.cpp, ollama 등)

**예상 소요**: 별도 계획 (E4 완료 후 수립)

---

## 3. 아키텍처 변경 요약

### 3.1 변경 전 (Legion Schedule 기반)

```
Schedule::builder()
  .add_system(movement_system())      ← #[system] 매크로
  .flush()                            ← 병렬화 경계
  .add_system(death_system())         ← SubWorld 사용
  .flush()
  ...
  .build()
  .execute(&mut world, &mut resources) ← Legion 스케줄러 실행
```

**약점**: 각 시스템이 SubWorld를 통해 제한된 뷰로 접근 → AccessDenied 위험

### 3.2 변경 후 (GameContext 기반)

```
let mut ctx = GameContext::new(&mut world, &mut resources, ...);
TurnRunner::execute(&mut ctx)
  → movement_system(&mut ctx)         ← 일반 함수
  → death_system(&mut ctx)            ← &mut World 직접 접근
  → vision_system(&mut ctx)           ← AccessDenied 불가능
  → ...
```

**장점**: 전체 World 접근 → 권한 검사 없음 → 패닉 원천 차단

### 3.3 ECS 유지 범위

| 유지 | 제거 |
|------|------|
| Entity / Component 데이터 모델 | `#[system]` 매크로 |
| `World::push()` 엔티티 생성 | `#[read/write_component]` 선언 |
| `<A, B>::query()` 컴포넌트 쿼리 | `SubWorld` 제한된 뷰 |
| `entry_ref()` / `entry_mut()` 직접 접근 | `Schedule` 병렬 스케줄러 |
| `CommandBuffer` (필요 시) | Legion Resources (`#[resource]`) |
| `serde` 직렬화 | |

**핵심**: "ECS 데이터 모델은 유지, 실행 모델만 교체"

---

## 4. v1.0 로드맵 진행 상태 (보존)

> 이하는 v1.0 로드맵에서 완료된 Phase의 기록. 신규 Phase E1~E5의 전제 조건.

| Phase | 상태 | 비고 |
|-------|------|------|
| S0: 빌드 검증 | ✅ 완료 | cargo build 성공, 4,177 테스트 통과 |
| S1: 앱 기동 | ✅ 완료 | eframe 윈도우 + Title 렌더링 |
| S2: 상태머신 관통 | ✅ 완료 | Title → CharCreation → Playing |
| S3: 첫 턴 생존 | ⚠️ 부분 | 29/31 시스템 통과, 30~31번째에서 AccessDenied → **v2.0에서 근본 해결** |
| S4: N턴 루프 | ✅ 완료 | 10턴 연속 패닉 없음 (29개 시스템 기준) |
| S5: 핵심 상호작용 | ✅ 완료 | 7/7 동사 통과 |
| S6: Edge Case | ✅ 완료 | 8/8 Edge Case 통과 |
| S7: LLM 주입 | → E4/E5 | 엔진 전환 후 Phase E4~E5로 재편 |

---

## 5. 성공 기준 요약

| Phase | 기준 | 예상 소요 |
|-------|------|-----------| 
| E1 | GameContext + TurnRunner 정의 + 컴파일 | 4~6시간 |
| E2a | 저난이도 12개 전환 + 빌드 | 4~6시간 |
| E2b | 중난이도 10개 전환 + 빌드 | 4~8시간 |
| E2c | 고난이도 9개 전환 + 기존 테스트 통과 | 4~8시간 |
| E3 | 전체 테스트 + 1000턴 퍼징 | 4~8시간 |
| E4 | LLM 아키텍처 확립 + 컴파일 | 6~10시간 |
| E5 | LLM 실제 주입 | 별도 계획 |

**E1~E3 총 예상**: **16~36시간** (집중 작업 기준)
**E1~E4 총 예상**: **22~46시간**

---

## 6. 브랜치 전략

```
main (v2.41.0 = 100% 순수 번역본, 불변)
 └─ stabilize/e2e-playable (v2.42.x = S0~S6 완료)
      └─ engine/gamecontext (v3.0.0 = 엔진 전환)
           ├─ E1: gamecontext-foundation
           ├─ E2: system-migration
           ├─ E3: integration-verified
           ├─ E4: llm-architecture
           └─ E5: llm-integration
```

- **v3.0.0**: 엔진 전환은 MAJOR 버전 변경 (Breaking Change: Schedule 제거)
- 각 Phase 완료 시 **태그 + 커밋 메시지**로 마일스톤 기록
- E3 완료 시 `stabilize/e2e-playable`로 머지 검토

---

**문서 버전**: v2.0
**최종 업데이트**: 2026-03-04
