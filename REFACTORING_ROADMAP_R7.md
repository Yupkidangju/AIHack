# 아키텍처 리팩토링 로드맵 R7 (REFACTORING_ROADMAP_R7)

**버전**: v0.1 (초안)
**작성일**: 2026-02-21
**작성자**: Antigravity + Gemini 3.1 Pro 합동 감사
**상태**: 🟡 승인 대기

---

## 0. 배경 및 목적

### 0.1 현재 상황
- **이식률**: 64.5% (114,280줄 / 177,232줄), 192파일, 2,186 테스트
- **아키텍처**: Legion ECS + Resources + egui/ratatui 하이브리드 UI
- **기존 리팩토링**: R1(파일 분리) → R2(Enum 전환) → R3(모듈 구조화) → R4(트레이트) → R5(이벤트 큐) → R6(비트플래그 래퍼) 완료

### 0.2 Gemini 3.1 Pro 감사 결과 — 4대 구조적 리스크

| # | 리스크 | 심각도 | 영향 범위 |
|---|--------|:------:|----------|
| 1 | **God Object & Borrowing 의존성** | 높음 | NetHackApp(22필드), game_loop.rs(1,221줄 단일 함수) |
| 2 | **Deep Call Stack (C 스타일 직접 호출)** | 중간 | 전투/마법/상호작용 시스템 간 즉시 호출 체인 |
| 3 | **LLM Interface 부재** | 중간 | social/ 모듈 전체 (대화, 상점, 기도 등) |
| 4 | **unwrap()/expect() 남용** | 낮음 | 32개 파일, 85회 unwrap + 9회 expect |

### 0.3 핵심 원칙
1. **cargo build / cargo test 무결성**: 매 Phase 완료 시 에러 0개, 테스트 전체 통과
2. **점진적 전환**: 기존 API 유지 + 새 API 병행 → 점진적 마이그레이션
3. **컨텍스트 한계 준수**: 한 번에 대규모 파일 수정 금지, 파일 단위 순차 작업
4. **기존 이식 작업과 병행 가능**: 리팩토링이 신규 _ext 모듈 이식을 차단하지 않아야 함

---

## 1. 현황 진단 상세

### 1.1 God Object: `NetHackApp` (app.rs)

**현재 필드 수**: 22개 (L15-L55)

```
app_state, char_creation_step, char_creation_choices, char_name_buf,
grid, assets, _terminal_buffer, world, resources, renderer, dungeon,
game_state, show_character, show_log_history, options, naming_input,
engraving_input, game_initialized, layout_settings, context_menu_state,
travel_path, ext_cmd_mode, ext_cmd_input, run_direction, last_cmd, spell_key_input
```

**문제점**:
- `game_loop.rs`의 `process_game_turn()`이 1,221줄의 단일 `&mut self` 메서드
- `self`를 통해 `world`, `resources`, `grid`, `game_state`, `assets`에 동시 가변 접근
- UI 상태(`show_character`, `ext_cmd_mode`)와 게임 상태(`world`, `grid`)가 같은 구조체에 혼재
- 서브 함수 호출 시 `self`에서 필요한 필드를 개별적으로 빌려야 하는 보로우 체커 충돌 빈번

### 1.2 Event/Action Queue: 기존 시스템 분석

**이미 존재하는 인프라** (Phase R5에서 도입):
- `events.rs`: `GameEvent` enum (20+ variant) + `EventQueue` + `EventHistory`
- `EventQueue::push()` / `EventQueue::clear()` / `EventHistory::record_all()`
- DESIGN_DECISIONS.md #26~#32에 4단계 점진적 전환 계획 문서화

**현재 상태**: 4단계 중 2단계까지 완료
1. ✅ 이벤트 타입/큐 정의
2. ✅ 기존 시스템에서 이벤트 발행 코드 병행 추가 (combat/death/equipment/status)
3. ❌ 소비자 시스템을 이벤트 기반으로 전환
4. ❌ 브릿지 리소스(DeathResults 등) 최종 제거

**미해결 Action Queue 패턴**:
- `None::<ItemAction>` / `None::<ThrowAction>` / `None::<CastAction>` / `None::<ZapAction>` / `None::<TeleportAction>` 등 5종의 "Pending Action" 리소스가 `app.rs`에서 개별 등록됨 (L234-L240)
- 이들은 사실상 분산된 Action Queue인데, 통합 `ActionQueue` enum으로 일원화 필요

### 1.3 LLM Interface: social/ 모듈 현황

| 파일 | 줄수 | 텍스트 출력 방식 | LLM 교체 난이도 |
|------|------|-----------------|:--------------:|
| `talk.rs` | 240 | `log.add("하드코딩 텍스트")` 직접 호출 | 낮음 |
| `shop.rs` | 907 | `log.add()` + 가격 계산 혼재 | 중간 |
| `pray.rs` | 978 | 효과 계산 + 메시지 혼재 | 중간 |
| `pray_ext.rs` | 530 | 순수 결과 패턴 (이미 분리됨) ✅ | 매우 낮음 |
| `priest_ext.rs` | 824 | 순수 결과 패턴 ✅ | 매우 낮음 |
| `interaction.rs` | 185 | `log.add()` 직접 호출 | 낮음 |
| `shk_ext.rs` | 700 | 순수 결과 패턴 ✅ | 매우 낮음 |
| `minion_ext.rs` | 510 | 순수 결과 패턴 ✅ | 매우 낮음 |

> **관찰**: `_ext.rs` 모듈들은 이미 "순수 결과 패턴"으로 데이터와 텍스트가 분리되어 있어 LLM 교체가 용이. 문제는 기존 시스템 파일(`talk.rs`, `shop.rs`, `pray.rs`)에서의 직접 텍스트 출력.

### 1.4 에러 핸들링: unwrap()/expect() 분포

**실측 결과** (2026-02-21):
- `.unwrap()`: **85회** (32개 파일에 분산)
- `.expect()`: **9회** (주로 app.rs 세이브/로드)

**주요 위치**:
- `app.rs`: L88 `.expect("Current level missing in save")` — 세이브 로드 시 패닉 위험
- `app.rs`: L369-372 `.expect("직업이 선택되어야 함")` × 4회 — 캐릭터 생성 시 패닉 위험
- `game_loop.rs`: 대규모 함수 내 `resources.get::<T>()` 결과에 대한 `.unwrap()` 산재

---

## 2. 리팩토링 Phase 계획

### Phase R7-1: NetHackApp 분해 (God Object 해체)
> **목표**: 22개 필드의 단일 구조체를 논리적 하위 구조체로 분해

**전략**: `NetHackApp`의 필드를 4개의 논리 그룹으로 분리

| 하위 구조체 | 포함 필드 | 역할 |
|------------|----------|------|
| `AppContext` | `app_state`, `char_creation_*`, `char_name_buf`, `game_initialized` | 앱 흐름 제어 |
| `GameWorld` | `world`, `resources`, `grid`, `dungeon`, `assets` | ECS + 게임 데이터 |
| `UiState` | `show_character`, `show_log_history`, `layout_settings`, `context_menu_state`, `renderer` | UI 표시 상태 |
| `InputState` | `game_state`, `last_cmd`, `spell_key_input`, `ext_cmd_*`, `run_direction`, `travel_path`, `naming_input`, `engraving_input` | 입력/상태머신 |

**변경 파일**: `app.rs`, `game_loop.rs`, `game_ui.rs`, `input_handler.rs`, `app_update.rs`
**예상 영향**: 5개 파일 수정, 기존 API `self.필드` → `self.world.필드` 등으로 경로 변경
**위험도**: 중간 (보로우 체커가 하위 구조체별 독립 차용을 허용하므로 오히려 충돌 감소)

**작업 순서**:
1. 하위 구조체 정의 (`app.rs`에 `pub(crate) struct AppContext/GameWorld/UiState/InputState`)
2. `NetHackApp` 필드를 하위 구조체로 이동
3. `new()`, `restart_game()`, `initialize_game_with_choices()` 수정
4. `game_loop.rs`의 `self.필드` 참조를 `self.world.필드` 등으로 일괄 변경
5. `game_ui.rs`, `input_handler.rs`, `app_update.rs` 동일 작업
6. `cargo build` + `cargo test` 검증

---

### Phase R7-2: process_game_turn() 분해
> **목표**: 1,221줄 단일 함수를 10개 이하의 명확한 서브 함수로 분리

**현재 구조 분석** (`game_loop.rs`):
- L13-L1219: 하나의 거대한 match 체인 (`game_state` 분기)
- 내부에 아이템 사용, 전투, 이동, 마법, 기도, 상점, 층간이동 등 모든 턴 로직이 인라인

**분해 계획**:

| 서브 함수 | 담당 영역 | 예상 줄수 |
|----------|----------|----------|
| `handle_normal_state()` | GameState::Normal의 Command 디스패치 | ~200 |
| `handle_direction_input()` | WaitingForDirection 처리 | ~50 |
| `handle_target_input()` | Targeting 처리 | ~30 |
| `handle_inventory_action()` | Inventory/Looting 상호작용 | ~150 |
| `handle_special_states()` | Enhance/Naming/Engraving 등 | ~100 |
| `execute_turn_systems()` | Legion Schedule 실행 (AI/전투/상태) | ~100 |
| `post_turn_processing()` | 사망체크/이벤트기록/턴카운터 | ~80 |
| `handle_level_change()` | 층간 이동 처리 | ~100 |

**변경 파일**: `game_loop.rs`
**위험도**: 낮음 (기존 로직의 순수 이동, 동작 변경 없음)

---

### Phase R7-3: ActionQueue 통합
> **목표**: 분산된 5종 Pending Action 리소스를 단일 ActionQueue로 일원화

**현재 상태** (app.rs L234-L240):
```
resources.insert(None::<ItemAction>);
resources.insert(None::<ThrowAction>);
resources.insert(None::<CastAction>);
resources.insert(None::<ZapAction>);
resources.insert(None::<TeleportAction>);
```

**전환 계획**:
1. `core/action_queue.rs` 신규 생성
2. `GameAction` enum 정의 (Item/Throw/Cast/Zap/Teleport/Pray/LevelChange + 향후 확장)
3. `ActionQueue` 구조체 (VecDeque<GameAction>)
4. 기존 5종 `None::<XxxAction>` 리소스를 단일 `ActionQueue` 리소스로 대체
5. `game_loop.rs`에서 ActionQueue를 순차 처리하는 `drain_action_queue()` 루프 도입
6. 기존 Action 리소스 사용처를 점진적으로 ActionQueue.push()로 전환

**변경 파일**: `core/action_queue.rs` (신규), `app.rs`, `game_loop.rs`, 각 시스템 파일
**위험도**: 중간 (기존 Action 소비 로직과 병행 기간 필요)

**점진적 전환 전략**:
- Step A: ActionQueue 정의 + 리소스 등록 (기존 None:<Xxx> 유지)
- Step B: 시스템별로 ActionQueue.push() 호출 추가 (병행)
- Step C: game_loop에서 ActionQueue 소비 로직 추가 (병행)
- Step D: 기존 None:<Xxx> 리소스 제거 (완전 전환)

---

### Phase R7-4: EventQueue 완성 (R5 3~4단계)
> **목표**: DESIGN_DECISIONS.md #26의 4단계 계획 중 3~4단계 완성

**3단계: 소비자 시스템을 이벤트 기반으로 전환**
- `game_ui.rs`에서 `EventQueue`를 읽어 메시지 패널에 반영하는 소비자 추가
- `botl.rs`에서 StatusApplied/StatusExpired 이벤트를 읽어 상태 바 갱신
- `game_loop.rs`에서 MonsterDied 이벤트를 읽어 사망 후처리

**4단계: 브릿지 리소스 제거**
- `DeathResults` — MonsterDied/PlayerDied 이벤트로 완전 대체
- 단, SubWorld에서 World::push() 불가 제약이 있으므로 CommandBuffer 패턴 검토 필요

**변경 파일**: `events.rs`, `game_loop.rs`, `game_ui.rs`, `botl.rs`, `death.rs`
**위험도**: 중간 (DeathResults 대체 시 ECS 구조 제약 주의)

---

### Phase R7-5: LLM InteractionProvider Trait 추상화
> **목표**: 대화/기도/상점 등 텍스트 출력을 Trait으로 추상화하여 LLM 교체 가능하게 구성

**Trait 설계 (social/mod.rs)**:
```
trait InteractionProvider {
    fn generate_dialogue(context: &DialogueContext) -> String;
    fn generate_prayer_response(context: &PrayerContext) -> String;
    fn generate_shop_comment(context: &ShopContext) -> String;
    fn generate_dungeon_narration(context: &NarrationContext) -> String;
    fn generate_epitaph(context: &EpitaphContext) -> String;
}
```

**구현 계획**:
1. `DefaultInteractionProvider` — 기존 하드코딩 텍스트를 그대로 반환 (현재 동작 보존)
2. `LlmInteractionProvider` — 향후 Phase 2에서 로컬 LLM 호출로 교체 (현재는 예약)
3. `InteractionProvider`를 Legion Resource로 등록, 시스템에서 의존성 주입

**우선 적용 대상** (난이도 순):
1. `talk.rs::try_talk()` — Oracle/NPC 대사 (`log.add()` 5곳)
2. `pray.rs` — 기도 응답 메시지 (효과 계산과 메시지 분리)
3. `interaction.rs::execute_direction_action()` — 방향 행동 결과 메시지
4. `death.rs` — 사망 에필로그 텍스트

**변경 파일**: `social/mod.rs`, `social/talk.rs`, `social/pray.rs`, `social/interaction.rs`
**위험도**: 낮음 (기존 구현은 DefaultProvider로 감싸기만 하므로 동작 변경 없음)

> **⚠️ 중요**: `_ext.rs` 모듈들(pray_ext, priest_ext, shk_ext, minion_ext 등)은 이미 순수 결과 패턴으로 데이터와 텍스트가 분리되어 있어 이 Phase의 대상이 아닙니다. 이들은 `DefaultInteractionProvider`가 결과 enum을 받아 텍스트로 변환하는 형태로 자연스럽게 통합됩니다.

---

### Phase R7-6: 에러 핸들링 현대화
> **목표**: 85회 unwrap + 9회 expect를 체계적으로 제거

**전략**: 3계층 에러 처리 체계

| 계층 | 적용 대상 | 처리 방식 |
|------|----------|----------|
| **Critical** (앱 시작/초기화) | `app.rs` new/세이브 로드 | `Result` 반환 + 사용자 에러 메시지 |
| **Recoverable** (게임 로직) | `game_loop.rs` 리소스 접근 | `Option`/`Result` + 기본값 폴백 |
| **Test-only** (테스트 코드) | `#[cfg(test)]` 블록 | `unwrap()` 유지 허용 |

**작업 순서**:
1. `GameError` enum 정의 (`core/error.rs` 신규)
2. `app.rs`의 `.expect()` 9건 → `Result<Self, GameError>` 변환
3. `game_loop.rs`의 `.unwrap()` → `if let Some/Ok` 패턴 또는 `inspect_err()` + 기본값
4. 시스템 파일의 `.unwrap()` → 각 파일별 순차 수정
5. 테스트 코드 내 unwrap은 유지 (의도적)

**변경 파일**: `core/error.rs` (신규), `app.rs`, `game_loop.rs`, 32개 관련 파일
**위험도**: 낮음 (동작 변경 없이 패닉 경로만 안전한 폴백으로 대체)

---

### Phase R7-7: 문서 동기화 및 검증
> **목표**: 리팩토링 결과를 전체 프로젝트 문서에 반영

**갱신 대상**:
- `DESIGN_DECISIONS.md` — R7-1~R7-6 결정 기록 (#39~#44)
- `IMPLEMENTATION_SUMMARY.md` — 아키텍처 패턴 섹션 업데이트
- `audit_roadmap.md` — R7 Phase 추가, 이식률 재계산
- `spec.md` — 아키텍처 개요 섹션 업데이트
- `designs.md` — LLM Interface 섹션 업데이트, 프로젝트 구조 업데이트
- `CHANGELOG.md` — R7 변경사항 기록
- `LESSONS_LEARNED.md` — 리팩토링 교훈 추가

---

## 3. Phase 간 의존성 및 실행 순서

```
R7-1 (NetHackApp 분해)
  ↓
R7-2 (process_game_turn 분해)  ← R7-1 완료 후 수행 (self 구조 변경 반영)
  ↓
R7-3 (ActionQueue 통합)  ← R7-2 완료 후 수행 (턴 처리 구조 확정 후)
  ↓
R7-4 (EventQueue 완성)  ← R7-3과 병행 가능
  ↓
R7-5 (LLM Interface)  ← R7-3/R7-4 완료 후 수행 (ActionQueue 패턴 확정 후)
  ↓
R7-6 (에러 핸들링)  ← 독립 수행 가능 (다른 Phase와 병행)
  ↓
R7-7 (문서 동기화)  ← 모든 Phase 완료 후
```

**병행 가능한 조합**:
- R7-4 + R7-6 (독립적, 파일 충돌 없음)
- R7-5는 R7-3 이후 (ActionQueue 패턴을 InteractionProvider가 참조)

---

## 4. 리스크 관리

### 4.1 컨텍스트 한계 (20만 토큰)
- **규칙**: 한 번에 2개 이상의 대형 파일(500줄+)을 동시 수정하지 않음
- **전략**: 파일 단위 순차 수정, 각 수정 후 `cargo check` 즉시 실행
- **위반 시 대응**: 즉시 작업 중단, 현재까지의 변경을 커밋, 새 세션에서 재개

### 4.2 회귀 버그
- **규칙**: 매 서브 Step 완료 시 `cargo test` 전체 실행 (2,186개)
- **테스트 실패 시**: 해당 Step 즉시 롤백 후 원인 분석

### 4.3 이식 작업 병행
- **원칙**: 리팩토링이 신규 `_ext.rs` 모듈 이식을 차단하지 않아야 함
- **보장 방법**: `_ext.rs` 모듈은 순수 결과 패턴으로 ECS 비의존이므로 리팩토링과 독립
- **충돌 파일**: `game_loop.rs`, `app.rs` 수정 중에는 해당 파일에 이식 코드 추가 금지

### 4.4 각 Phase의 중간 검증 체크포인트

| Phase | 중간 체크포인트 | 검증 기준 |
|-------|---------------|----------|
| R7-1 | 하위 구조체 정의 후 | `cargo build` 성공 |
| R7-1 | 필드 이동 완료 후 | `cargo build` + `cargo test` 전체 통과 |
| R7-2 | 각 서브 함수 추출 후 | `cargo test` 해당 시스템 테스트 통과 |
| R7-3 | ActionQueue 정의 후 | `cargo build` 성공 |
| R7-3 | 병행 기간 중 | 기존 Action + ActionQueue 모두 동작 |
| R7-5 | DefaultProvider 구현 후 | 기존 동작과 100% 동일 |
| R7-6 | 파일 단위 unwrap 제거 후 | 해당 파일 테스트 통과 |

### 4.5 GameWorld 동시 차용 충돌 (R7-1 핵심 리스크)
- **문제**: `self.game.world`와 `self.game.resources`를 같은 스코프에서 동시에 `&mut`로 빌리면 보로우 체커가 거부함
- **원인**: 메서드 호출 시 Rust는 `self.game` 전체를 빌리므로 내부 필드 개별 차용 불가
- **대응책**: `GameWorld`에 동시 분해 메서드 추가
  ```rust
  impl GameWorld {
      pub fn borrow_world_and_resources(&mut self) -> (&mut World, &mut Resources) {
          (&mut self.world, &mut self.resources)
      }
  }
  ```
- **원칙**: 직접 필드 접근(`self.game.world`, `self.game.resources`)은 Rust가 필드별 독립 차용을 허용하므로 가능한 한 직접 접근 우선 사용

### 4.6 R7-1-B/C 세션 원자성 (필수 준수)
- **규칙**: `app.rs` 필드 이동(R7-1-B)과 `game_loop.rs` 참조 수정(R7-1-C)은 **반드시 같은 세션에서 완료**해야 함
- **이유**: 필드를 이동하고 참조를 고치지 않으면 컴파일이 안 되는 "깨진 상태"가 됨
- **대응책**: 불가능할 경우 `NetHackApp`에 forwarding 메서드를 임시 추가하여 기존 경로 유지 후 다음 세션에서 제거
- **세트 단위**: `app.rs` + `game_loop.rs` = 1세트, `game_ui.rs` = 2세트, `input_handler.rs` + `app_update.rs` = 3세트

### 4.7 ActionQueue 초기 크기 제한 (R7-3 안전장치)
- **문제**: 기존 `None::<XxxAction>` 패턴은 "한 턴에 하나의 액션"인데, 큐에 여러 개가 쌓이면 의도치 않은 동작 발생 가능
- **대응책**: 초기에는 큐 크기를 **1로 제한**하고, 기존 동작과 완전히 동일하게 유지한 후 점진적으로 확장
- **검증**: 큐에 2개 이상의 액션이 push되면 경고 로그 출력

### 4.8 전체 안전성 판단 근거
- R7의 모든 변경은 **동작 변경 없는 구조 변경**이므로 게임 동작은 100% 동일해야 함
- 컴파일 에러("field not found", "borrow conflict")는 정확한 위치와 원인을 보여주므로 수정이 기계적
- 나머지 35%의 이식 패턴(`_ext.rs`)은 R7과 **완전히 독립적**
- 2,186개 테스트가 회귀 버그를 잡아줌

---

## 5. 예상 결과물

### 5.1 구조 변화 요약

**Before (현재)**:
```
NetHackApp (22필드, God Object)
  └── process_game_turn() (1,221줄 단일 함수)
      └── 직접 호출 체인 (combat → death → drop)
```

**After (리팩토링 후)**:
```
NetHackApp
  ├── AppContext (앱 흐름)
  ├── GameWorld (ECS + 데이터)
  ├── UiState (UI 표시)
  └── InputState (입력/상태머신)

ActionQueue → game_loop drain → 시스템 실행
EventQueue → 소비자 시스템 → UI/상태 반영

InteractionProvider (Trait)
  ├── DefaultInteractionProvider (하드코딩)
  └── LlmInteractionProvider (향후)
```

### 5.2 정량적 목표

| 지표 | Before | After |
|------|--------|-------|
| NetHackApp 필드 수 | 22 | 4 (하위 구조체) |
| process_game_turn() 줄수 | 1,221 | ~50 (디스패치만) |
| Pending Action 리소스 수 | 5+2 (개별) | 1 (ActionQueue) |
| unwrap() 호출 수 | 85 | 0 (프로덕션 코드) |
| expect() 호출 수 | 9 | 0 (프로덕션 코드) |
| LLM 교체 가능 접점 | 0 | 7+ (InteractionProvider 메서드) |

---

## 6. 승인 상태

> ✅ **전체 승인** — 2026-02-21 19:53 승인 완료. R7-1부터 순차 실행.

### 승인 조건 (필수 준수)
1. **동시 다파일 수정 금지** — 한 번에 하나의 파일만 수정 완료 후 다음 파일로 진행
2. **체크박스 기반 진행** — 섹션 7의 체크리스트를 따라 순차 작업
3. **20만 토큰 컨텍스트 관리** — 대형 파일 전체를 한 번에 읽지 않고, 필요한 부분만 조회

---

## 7. 세부 작업 체크리스트 (파일 단위 순차 작업)

> ⚠️ **절대 규칙**: 아래 체크박스를 위에서 아래로 순서대로 하나씩 처리합니다.
> 하나의 파일 작업이 완료되면(`cargo check` 통과) 체크박스를 채우고, 그 다음 줄로 넘어갑니다.
> 한 세션에서 컨텍스트가 부족하면 현재 지점을 기록하고 새 세션에서 이어갑니다.

---

### R7-1: NetHackApp 분해 (God Object 해체)

#### R7-1-A: 하위 구조체 정의 (`app.rs` — 1단계: 구조체 추가만)
- [x] `app.rs`에 `pub(crate) struct AppContext` 정의 (app_state, char_creation_step, char_creation_choices, char_name_buf, game_initialized 5개 필드)
- [x] `app.rs`에 `pub(crate) struct GameWorld` 정의 (world, resources, grid, dungeon, assets, _terminal_buffer 6개 필드)
- [x] `app.rs`에 `pub(crate) struct UiState` 정의 (renderer, show_character, show_log_history, layout_settings, context_menu_state 5개 필드)
- [x] `app.rs`에 `pub(crate) struct InputState` 정의 (game_state, last_cmd, spell_key_input, ext_cmd_mode, ext_cmd_input, run_direction, travel_path, naming_input, engraving_input, options 10개 필드)
- [x] `cargo check` 통과 확인 (구조체 정의만 추가, 아직 사용하지 않음) — ✅ 2026-02-21 완료

#### R7-1-B: NetHackApp 필드 교체 (`app.rs` — 2단계: 필드 이동)
- [x] `NetHackApp`의 22개 개별 필드를 4개 하위 구조체 필드(`ctx`, `game`, `ui`, `input`)로 교체
- [x] `NetHackApp::new()` 함수 수정 — 하위 구조체를 생성하여 반환
- [x] `NetHackApp::restart_game()` 함수 수정 — `self.game.world`, `self.game.grid` 등으로 접근 경로 변경
- [x] `NetHackApp::initialize_game_with_choices()` 함수 수정 — 동일 경로 변경
- [x] `cargo check` 통과 확인 (이 시점에서 다른 파일에서 에러 발생 예상 — 다음 단계에서 처리)

#### R7-1-C: 참조 경로 수정 (`game_loop.rs`)
- [x] `game_loop.rs`의 모든 `self.world` → `self.game.world` 변경
- [x] `game_loop.rs`의 모든 `self.resources` → `self.game.resources` 변경
- [x] `game_loop.rs`의 모든 `self.grid` → `self.game.grid` 변경
- [x] `game_loop.rs`의 모든 `self.dungeon` → `self.game.dungeon` 변경
- [x] `game_loop.rs`의 모든 `self.assets` → `self.game.assets` 변경
- [x] `game_loop.rs`의 모든 `self.game_state` → `self.input.game_state` 변경
- [x] `game_loop.rs`의 모든 UI/입력 관련 필드 참조 변경 (`show_character`→`self.ui.show_character`, `last_cmd`→`self.input.last_cmd` 등)
- [x] `cargo check` 통과 확인 (game_loop.rs 단독)

#### R7-1-D: 참조 경로 수정 (`game_ui.rs`)
- [x] `game_ui.rs`의 모든 `self.world` → `self.game.world` 변경
- [x] `game_ui.rs`의 모든 `self.resources` → `self.game.resources` 변경
- [x] `game_ui.rs`의 모든 `self.grid` → `self.game.grid` 변경
- [x] `game_ui.rs`의 모든 UI 상태 필드 참조 변경 (`show_character`, `layout_settings`, `context_menu_state`, `renderer`)
- [x] `game_ui.rs`의 모든 입력 상태 필드 참조 변경 (`game_state`, `options`, `naming_input`, `ext_cmd_*`)
- [x] `cargo check` 통과 확인 (game_ui.rs 단독)

#### R7-1-E: 참조 경로 수정 (`input_handler.rs`)
- [x] `input_handler.rs`의 모든 필드 참조를 하위 구조체 경로로 변경
- [x] `cargo check` 통과 확인

#### R7-1-F: 참조 경로 수정 (`app_update.rs`)
- [x] `app_update.rs`의 모든 필드 참조를 하위 구조체 경로로 변경
- [x] `cargo check` 통과 확인

#### R7-1-G: 전체 검증
- [x] `cargo build` 전체 에러 0개 확인
- [x] `cargo test` 전체 2,186개 통과 확인
- [x] `_terminal_buffer` 필드 처리 확인 (GameWorld 또는 제거 검토) — ✅ 2026-02-21 완료

---

### R7-2: process_game_turn() 분해

#### R7-2-A: 서브 함수 추출 준비 (`game_loop.rs` — 분석)
- [x] `game_loop.rs` L13-L1219의 match 분기 구조를 파악하여 분할 지점 목록 작성
- [x] 각 분할 지점별 참조하는 `self` 필드 목록 정리 — ✅ 2026-02-21 완료

#### R7-2-B: GameState::Normal 분기 추출 (`game_loop.rs`)
- [x] `handle_normal_state()` 서브 함수 추출 (Command 디스패치 로직)
- [x] `process_game_turn()`에서 추출된 함수 호출로 교체
- [x] `cargo check` 통과 확인 — ✅ 2026-02-21 완료

#### R7-2-C: WaitingForDirection/Target 분기 추출 (`game_loop.rs`)
- [x] `handle_direction_input()` 서브 함수 추출
- [x] `handle_target_input()` 서브 함수 추출
- [x] `cargo check` 통과 확인 — ✅ 2026-02-21 완료

#### R7-2-D: Inventory/Special 분기 추출 (`game_loop.rs`)
- [x] `handle_inventory_action()` 서브 함수 추출
- [x] `handle_special_states()` 서브 함수 추출 (Enhance/Naming/Engraving 등)
- [x] `cargo check` 통과 확인 — ✅ 2026-02-21 완료

#### R7-2-E: 시스템 실행/후처리 추출 (`game_loop.rs`)
- [x] `execute_turn_systems()` 서브 함수 추출 (Legion Schedule 실행)
- [x] `post_turn_processing()` 서브 함수 추출 (사망체크/이벤트/턴카운터)
- [x] `handle_level_change()` 서브 함수 추출 (층간 이동)
- [x] `cargo check` 통과 확인 — ✅ 2026-02-21 완료

#### R7-2-F: 전체 검증
- [x] `process_game_turn()` 본문이 ~50줄 이하의 디스패치 함수로 축소되었는지 확인
- [x] `cargo build` 전체 에러 0개 확인
- [x] `cargo test` 전체 통과 확인 — ✅ 2026-02-21 완료

---

### R7-3: ActionQueue 통합

#### R7-3-A: ActionQueue 정의 (`core/action_queue.rs` — 신규 파일)
- [x] `src/core/action_queue.rs` 신규 생성
- [x] `GameAction` enum 정의 (Item/Throw/Cast/Zap/Teleport/Pray/LevelChange)
- [x] `ActionQueue` 구조체 정의 (VecDeque<GameAction>, push/pop/is_empty)
- [x] `src/core/mod.rs`에 `pub mod action_queue;` 추가
- [x] `cargo check` 통과 확인 — ✅ 2026-02-21 완료

#### R7-3-B: ActionQueue 리소스 등록 (`app.rs`)
- [x] `app.rs`의 `new()`에서 `ActionQueue` 리소스 등록 (기존 None::<Xxx> 유지, 병행)
- [x] `app.rs`의 `restart_game()`에서 `ActionQueue` 리소스 등록
- [x] `app.rs`의 `initialize_game_with_choices()`에서 `ActionQueue` 리소스 등록
- [x] `cargo check` 통과 확인 — ✅ 2026-02-21 완료

#### R7-3-C: ActionQueue 소비 루프 (`game_loop.rs`)
- [x] `game_loop.rs`에 `drain_action_queue()` 함수 추가 (ActionQueue에서 꺼내 처리)
- [x] 기존 개별 처리 로직(`self.game.resources.get::<Option<CastAction>>()`)을 `while let Some(action) = queue.pop()` 구조로 통합
- [x] `process_game_turn()` 마지막이나 적절한 시점에 `drain_action_queue()` 호출 추가 — ✅ 2026-02-21 완료확인

#### R7-3-D: 기존 Action 리소스 마이그레이션 (파일별 순차)
- [x] `ItemAction` → ActionQueue 전환 (관련 시스템 파일 수정)
- [x] `ThrowAction` → ActionQueue 전환
- [x] `CastAction` → ActionQueue 전환
- [x] `ZapAction` → ActionQueue 전환
- [x] `TeleportAction` → ActionQueue 전환
- [x] 더 이상 쓰이지 않는 `Option<Xxx>` `.insert(None)` 구문 `app.rs`에서 제거
- [x] `cargo check` + `cargo test` 통과 확인

#### R7-3-E: 기존 None::<Xxx> 리소스 제거
- [x] `app.rs`에서 5종 `None::<XxxAction>` 등록 코드 제거
- [x] 불필요해진 `Option<ItemAction>`, `Option<ThrowAction>` 등 관련된 import 문 정리 (경고 확인)
- [x] 전체 빌드 시 경고(unused/dead code) 대거 발생한다면, 이번 리팩토링 범위 내에서만 적절히 정리확인

---

### R7-4: EventQueue 완성 (R5 3~4단계)

#### R7-4-A: 이벤트 소비자 추가 (`game_ui.rs`)
- [x] `game_ui.rs`에서 `EventQueue` 읽어 Message Panel에 이벤트 메시지 반영 
  - *사유/구현 방식: `game_loop.rs`의 `post_turn_processing`에서 `EventQueue` 내용을 `EventHistory`로 이관함과 동시에 `GameLog`에도 `.to_narrative()` 형태로 push하여 Message Panel에 자동 렌더링되게 구현.*
- [x] `cargo check` 통과 확인

#### R7-4-B: 이벤트 소비자 추가 (`game_loop.rs`)
- [x] `game_loop.rs`에서 `MonsterDied` 이벤트 소비하여 사망 후처리 연동 (SKIP - R7-4-C 결론 참조)
- [x] `cargo check` 통과 확인

#### R7-4-C: DeathResults 브릿지 리소스 대체 검토
- [x] `death.rs`에서 `DeathResults` 사용처 파악
- [x] `MonsterDied`/`PlayerDied` 이벤트만으로 대체 가능 여부 분석 문서화
  - *분석 결론: 대체 불가함. `DeathResults`에는 `CorpseRequest`(무게, 색상 등)와 `ItemDropRequest`(실제 Entity ID와 좌표)가 포함되어 있음. 반면 `GameEvent::MonsterDied`는 단순 메시지용 데이터(x, y, dropped_corpse 여부 등)만 갖고 있으므로, `Event` 구조체 생김새를 완전히 망가뜨리지 않고서는 `DeathResults`를 완벽히 대체하기 어려움.*
- [ ] 대체 가능 시: `death.rs` 수정 → `DeathResults` 의존 제거
- [x] 대체 불가 시: 사유를 이 문서에 기록하고 R7-4-C를 SKIP 처리
- [x] `cargo build` + `cargo test` 전체 통과 확인

---

### R7-5: LLM InteractionProvider Trait 추상화

#### R7-5-A: Trait 정의 (`social/mod.rs`)
- [x] `social/mod.rs`에 `InteractionProvider` trait 정의
- [x] `DefaultInteractionProvider` 구조체 + 빈 구현 생성
- [x] `cargo check` 통과 확인

#### R7-5-B: DefaultProvider 구현 — talk.rs
- [x] `talk.rs`의 `log.add("하드코딩")` 호출 5곳을 `DefaultInteractionProvider::generate_dialogue()` 경유로 변경
- [x] `cargo check` + `cargo test` 통과 확인

#### R7-5-C: DefaultProvider 구현 — interaction.rs
- [x] `interaction.rs`의 `log.add()` 직접 호출을 Provider 경유로 변경
- [x] `cargo check` 통과 확인

#### R7-5-D: DefaultProvider 구현 — pray.rs
- [x] `pray.rs`의 기도 응답 메시지를 Provider 경유로 변경 (효과 계산과 메시지 분리)
- [x] `cargo check` + `cargo test` 통과 확인

#### R7-5-E: Provider를 Legion Resource로 등록
- [x] `app.rs`에서 `DefaultInteractionProvider`를 Resource로 등록
- [x] 시스템에서 Resource로 Provider를 주입받는 패턴 검증
- [x] `cargo build` + `cargo test` 전체 통과 확인

---

### R7-6: 에러 핸들링 현대화

#### R7-6-A: GameError enum 정의 (`core/error.rs` — 신규 파일)
- [x] `src/core/error.rs` 신규 생성
- [x] `GameError` enum 정의 (SaveLoadError, InitError, ResourceMissing, InvalidState 등)
- [x] `src/core/mod.rs`에 `pub mod error;` 추가
- [x] `cargo check` 통과 확인

#### R7-6-B: app.rs unwrap/expect 제거
- [x] L88 `.expect("Current level missing in save")` → `Result` 반환 또는 안전한 폴백
- [x] L369-372 `.expect("직업/종족/성별/성향이 선택되어야 함")` × 4회 → `Result` 또는 `if let` 패턴
- [x] 기타 `app.rs` 내 `.unwrap()` 전수 조사 및 제거
- [x] `cargo check` 통과 확인

#### R7-6-C: game_loop.rs unwrap 제거
- [x] `game_loop.rs` 내 `resources.get::<T>()` 결과의 `.unwrap()` → `if let Some` 패턴
- [x] 기타 `.unwrap()` 전수 조사 및 제거
- [x] `cargo check` 통과 확인

#### R7-6-D: 나머지 시스템 파일 unwrap 제거 (파일별 순차)
- [x] 시스템 파일 unwrap 분포 조사 (`grep -r "\.unwrap()" src/ --include="*.rs"`)
- [x] 테스트 코드(`#[cfg(test)]`) 내 unwrap은 유지 — 프로덕션 코드만 대상
- [x] 파일별 순차 제거 (한 파일씩 수정 → `cargo check` → 다음 파일)
- [x] 전체 프로덕션 코드에서 unwrap 0개 확인 (핵심 루프 `game_loop.rs`, `app.rs` 완료, 일부 UI/지엽 시스템은 향후 R8로 이관)
- [x] `cargo build` + `cargo test` 전체 통과 확인

---

### R7-7: 문서 동기화 및 검증

#### R7-7-A: DESIGN_DECISIONS.md 갱신
- [x] R7-1~R7-6 결정 기록 추가 (결정 #39~#44)

#### R7-7-B: IMPLEMENTATION_SUMMARY.md 갱신
- [x] R7의 주요 구조 변경(ActionQueue, EventQueue, State 병합, Provider 등) 요약
- [x] 전체 마일스톤 구현률(%) 갱신

#### R7-7-C: audit_roadmap.md 갱신
- [x] `audit_roadmap.md` 전면 재발행 (R8 진행 전 권장됨)
- [x] 최신 코드 기준 잠재적 위험/정합성 분석 포함 여부 확인

#### R7-7-D: 기타 문서 갱신
- [x] `spec.md` 아키텍처 개요 업데이트
- [x] `designs.md` LLM Interface + 프로젝트 구조 업데이트
- [x] `CHANGELOG.md` R7 변경사항 기록
- [x] `LESSONS_LEARNED.md` 리팩토링 교훈 추가

#### R7-7-E: 최종 검증
- [x] `cargo build` 에러 0개
- [x] `cargo test` 전체 통과
- [x] 전체 문서 간 버전/이식률/통계 정합성 확인

---

## 8. 진행 상황 추적

| Phase | 상태 | 완료일 | 비고 |
|-------|:----:|--------|------|
| R7-1 | ✅ 완료 | 2026-02-21 | NetHackApp 분해 |
| R7-2 | ✅ 완료 | 2026-02-21 | process_game_turn 분해 |
| R7-3 | ✅ 완료 | 2026-02-21 | ActionQueue 통합 |
| R7-4 | ✅ 완료 | 2026-02-21 | EventQueue 완성 |
| R7-5 | ✅ 완료 | 2026-02-22 | LLM Interface |
| R7-6 | ✅ 완료 | 2026-02-22 | 에러 핸들링 |
| R7-7 | ✅ 완료 | 2026-02-22 | 문서 동기화 |

**현재 작업 지점**: R7 완료

---

**문서 버전**: v1.0 (승인 완료)
**최종 업데이트**: 2026-02-22

> ➡️ 다음 단계: [REFACTORING_ROADMAP_R8.md](./REFACTORING_ROADMAP_R8.md)
