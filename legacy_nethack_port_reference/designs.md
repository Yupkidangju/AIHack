# AIHack 디자인 구조 (designs.md)

**프로젝트명**: AIHack (NetHack 3.6.7 → Rust 100% Port)
**버전**: v2.41.1
**최종 업데이트**: 2026-03-30

---

## 0. 전역 문서 규칙

- 이 문서는 spec.md(마스터플랜)에 종속된다.
- C 구조 직역 금지 원칙(ARCH-1) 적용
- 모든 신규 코드는 Pure Result 패턴을 준수한다.

---

## 1. 전체 앱 구조도

```
+----------------------- AIHack App -----------------------+
|                                                           |
|  +----------------- AppState FSM -------------------+    |
|  |  Title -> CharCreation -> Playing -> GameOver     |    |
|  +---------------------------------------------------+    |
|                                                           |
|  +-------------- UI Layer (egui Host) ---------------+   |
|  |  +---------+  +----------+  +------------------+  |   |
|  |  | MenuBar |  |RightPanel|  |CommandBar(하단)   |  |   |
|  |  | (상단)  |  | (Stats)  |  |Simple/Advanced   |  |   |
|  |  +---------+  +----------+  +------------------+  |   |
|  |  +--------------------------------------------+    |   |
|  |  |            Center Canvas                    |   |   |
|  |  |         +-------------------+               |   |   |
|  |  |         | Ratatui (TUI Map) |               |   |   |
|  |  |         |  ASCII Rendering  |               |   |   |
|  |  |         +-------------------+               |   |   |
|  |  |         +-------------------+               |   |   |
|  |  |         | Message Log (하단)|               |   |   |
|  |  |         +-------------------+               |   |   |
|  |  +--------------------------------------------+    |   |
|  +----------------------------------------------------+   |
|                                                           |
|  +------------ Game Engine (ECS) --------------------+   |
|  |  ActionQueue -> Systems -> EventQueue -> Render    |   |
|  +----------------------------------------------------+   |
+-----------------------------------------------------------+
```

### 1.1 기능 설명
- **AppState FSM**: Title/CharCreation/Playing/GameOver 4상태 전환
- **MenuBar**: File(세이브/로드), View(Settings), Help
- **RightPanel**: 능력치 + 장비 요약 (ECS 실시간 조회)
- **CommandBar**: 마우스 사용자를 위한 커맨드 버튼
- **Center Canvas**: Ratatui ASCII 맵 + 메시지 로그
- **Game Engine**: ActionQueue 기반 턴 처리

### 1.2 구현 시 주의사항
- egui를 메인 호스트로 사용, 중앙 캔버스에 Ratatui 렌더링
- 마우스 클릭 이동: 인접->즉시, 원거리->A* 자동이동
- 자동 이동 중 몬스터/트랩 감지 시 반드시 중단

---

## 2. 게임 루프 구조도

```
+------------------ Turn Cycle -------------------+
|                                                   |
|  1. Input Phase                                   |
|     poll_input() -> Command enum                  |
|     (keyboard / mouse / menu)                     |
|                    |                              |
|  2. ActionQueue Phase                             |
|     Command -> ActionQueue.push()                 |
|     drain() -> 시스템별 분배                      |
|                    |                              |
|  3. Systems Phase (ECS)                           |
|     movement -> combat -> ai -> item_use          |
|     -> status_tick -> death -> spawn              |
|     (Gather-Apply 패턴)                           |
|                    |                              |
|  4. Event Phase                                   |
|     EventQueue -> EventHistory 기록               |
|     -> 소비자 시스템 (GameLog, botl, AI)          |
|     -> EventQueue.clear(next_turn)                |
|                    |                              |
|  5. Render Phase                                  |
|     vision_update -> tile_render                  |
|     -> message_log -> status_bar -> egui panels   |
+---------------------------------------------------+
```

### 2.1 기능 설명
- 턴제 게임 루프: 플레이어 입력 1회 = 1턴
- ActionQueue: 모든 행동을 계획-실행 분리
- EventQueue: 시스템 간 느슨한 결합 (20+ variant)
- EventHistory: 링 버퍼 200건, LLM 문맥 전달용

---

## 3. ECS 시스템 모듈 구조도

```
src/core/systems/
+-- combat/          <- 전투 시스템
|   +-- engine.rs        uhitm.c (플레이어->몬스터)
|   +-- mhitu.rs         mhitu.c (몬스터->플레이어)
|   +-- mhitm.rs         mhitm.c (몬스터<->몬스터)
|   +-- throw.rs         dothrow.c (투척)
|   +-- weapon.rs        weapon.c (무기/숙련도)
|   +-- kick.rs          dokick.c (발차기)
|
+-- ai/              <- 몬스터 AI
|   +-- core.rs          ECS 시스템 함수
|   +-- dog.rs           dog.c + dogmove.c (펫 AI)
|   +-- monmove.rs       monmove.c (이동 판정)
|   +-- mcastu.rs        mcastu.c (마법 시전)
|   +-- muse.rs          muse.c (아이템 사용 AI)
|   +-- wizard.rs        wizard.c (위저드 AI)
|
+-- item/            <- 아이템
|   +-- eat.rs           eat.c (식사)
|   +-- potion.rs        potion.c (포션)
|   +-- zap.rs           zap.c (완드/빔)
|
+-- creature/        <- 생물 공통
|   +-- status.rs        상태이상 (41종 u64 bitflags)
|   +-- equipment.rs     장비 시스템
|   +-- movement.rs      hack.c (이동)
|   +-- evolution.rs     polyself.c (변신)
|
+-- world/           <- 월드/환경
|   +-- trap.rs          trap.c (함정 30종)
|   +-- vision.rs        vision.c (시야)
|   +-- death.rs         사망/CommandBuffer
|
+-- social/          <- 사회적 상호작용
|   +-- shop.rs          shk.c (상점)
|   +-- pray.rs          pray.c (기도)
|   +-- interaction.rs   InteractionProvider
|
+-- spawn/           <- 생성
|   +-- makemon.rs       makemon.c
|
+-- identity/        <- 명명/식별
|   +-- botl.rs          botl.c (상태바)
|
+-- misc/            <- 기타
    +-- artifact.rs      artifact.c
    +-- spell.rs         spell.c
```

### 3.1 구현 시 주의사항
- 모든 _ext 모듈(33개+)은 위 기본 모듈의 Pure Result 확장
- pub use re-export로 crate::core::systems::xxx 경로 유지

---

## 4. 데이터 흐름

```
assets/data/*.toml
    | (빌드 시)
build.rs -> src/generated/ (MonsterKind/ItemKind enum)
    | (런타임 로드)
AssetManager.load_defaults()
    | (HashMap + 병렬 인덱스)
MonsterManager / ItemManager
    | (ECS Entity 생성)
World.push((Monster { kind, .. }, Health, ..))
```

- **build.rs**: TOML에서 enum 자동 생성
- **SaveState**: JSON 기반, 버전 필드로 하위 호환

---

## 5. 화면 레이아웃

### 5.1 Playing 화면
- 좌측: Ratatui ASCII 맵 + 메시지 로그
- 우측: Stats Panel (능력치 6종 + 장비 요약)
- 하단: CommandBar (Simple/Advanced 전환)
- 최하단: 상태바 (Dlvl/Gold/HP/Pw/AC/XL)

### 5.2 GameOver 화면
- 풀스크린 ASCII 묘비 아트
- 사망 메시지 + 점수/턴/던전 깊이 통계
- New Game / Quit 버튼

---

## 6. LLM 통합 구조 (Phase 2 준비)

```
InteractionProvider (Trait)
+-- DefaultInteractionProvider (하드코딩, 8개 교체점)
+-- LlmInteractionProvider (향후 로컬 LLM)

EventHistory (링 버퍼 200건)
+-- recent_narrative() -> LLM 피딩용 문맥
+-- GameEvent.to_narrative() -> 자연어 요약

Delta 직렬화
+-- DirtyMarker -> 변경 엔티티만 추출
+-- 체크포인트(10턴)에서만 전체 스냅샷
```

### 6.1 구현 시 주의사항
- LLM 교체 시 InteractionProvider 트레이트만 구현
- Behavior::decide() -> 매 턴 0ms (규칙 기반)
- Conversable::respond() -> 대화 개시 시 ~0.5초 허용

---

## 7. 게임 플레이 안정화 진행 상태

> 이식률 100%이나, 게임 루프(`game_loop.rs`)에 모든 커맨드가 연결되지 않은 상태.
> 아래 순서로 점진적으로 연결. 상세: `STABILIZATION_ROADMAP.md` 참조.

### 7.1 구조적 문제 (진단 완료)

```
┌─────────────── 게임 루프 커맨드 처리 ──────────────┐
│                                                      │
│  Command::Open → request_direction(Open)             │
│                      │                               │
│  ┌──── 방향 입력 ────┘                               │
│  │                                                   │
│  ├─ Throw/Cast/Zap → 개별 처리 (✅ Grid 동기화O)    │
│  ├─ Apply/Loot     → 개별 처리 (✅ Grid 동기화O)    │
│  └─ Open/Close/Kick/Talk → _ => 분기                 │
│         │                                            │
│         └─ execute_direction_action() 호출            │
│              └─ Grid 수정됨... 하지만                 │
│                 ❌ resources.insert(grid) 누락!       │
│                 → 렌더링에 반영 안 됨                 │
└──────────────────────────────────────────────────────┘
```

### 7.2 연결 우선순위

| 순서 | 단계 | 대상 | 상태 |
|------|------|------|------|
| 1 | S5a | Grid 역동기화 + split_for_query 제거 | ✅ 완료 |
| 2 | S5b | Open/Close/Kick/Pickup/Stairs/Death | ✅ 완료 |
| 3 | S5c | Wear/Wield/Quaff/Read/Cast/Zap/Throw/Eat/Apply/Pray | ⬜ 대기 |
| 4 | S6 | Save/Load, 레벨 변경, 상점, Edge Cases | ⬜ 대기 |
| 5 | S7 | LLM 최소 주입 (InteractionProvider 교체) | ⬜ 대기 |

### 7.3 구현 시 주의사항

- **Grid 역동기화 필수**: `execute_turn_systems()` 직후 `resources.Grid → self.game.grid` 복원 (이중화 방지)
- Grid를 직접 수정하는 경우(game_loop 내부) `resources.insert(self.game.grid.clone())` 호출
- `split_for_query` 사용 금지 → 직접 World 접근으로 교체 (AccessDenied 방지)
- 각 커맨드에서 `_action_executed = true` 설정 확인 (턴 소비 여부)
- UI(인벤토리 창, 프롬프트)와 GameState 상태 머신의 정합성 유지
- 시작 장비 자동 장착 시 `Equipment.slots.insert()` + `Inventory.remove_item()` 쌍 호출

---

**문서 버전**: v2.41.1
**최종 업데이트**: 2026-03-30
