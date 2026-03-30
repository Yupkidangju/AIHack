# 🛡️ AIHack 안정화 로드맵 (STABILIZATION_ROADMAP)

**버전**: v1.1
**작성일**: 2026-02-28
**최종 업데이트**: 2026-03-30 (Phase S5a 근본 수정 완료, S5b 검증 진행 중)
**대상 브랜치**: `stabilize/e2e-playable` (main에서 분기)
**전제**: v2.41.0 main 브랜치 = 100% 순수 번역본 (177,229줄, 4,177 테스트)
**목표**: cargo run → Title → CharCreation → Playing (N턴 생존) → GameOver 전체 E2E 동작

---

## 0. 외부감사 제안 내부 검토

### ✅ 수용하는 항목
| 제안 | 판단 | 이유 |
|------|------|------|
| LLM 연동 보류 | **완전 수용** | 로직 데드락 vs LLM 오류 구분 불가. 결정론적 동작 우선 |
| AppState 상태머신 E2E 검증 | **수용** | 현재 Title→Playing 흐름의 패닉 여부 미검증 |
| ActionQueue→EventQueue 동기화 검증 | **수용** | post_turn_processing의 borrow 패턴이 런타임에서 안전한지 미확인 |
| NetHack 특화 Edge Case 테스트 | **부분 수용** | 중요하나 E2E가 먼저. Phase 2로 배치 |
| 점진적 LLM 주입 전략 | **수용** | death.rs 묘비명부터 시작하는 전략이 합리적 |
| 커스텀 패닉 훅(Panic Hook) 설정 | **수용** | eframe/ratatui 혼용 환경에서 패닉 시 터미널 꼬임 방지 및 트레이스 확보 필수 |
| 큐 연쇄 액션(Cascade) 무한 루프 방지 | **수용** | 이벤트 큐 아키텍처 다발성 데드락 방지를 위한 Safe Limit(상한선) 도입 필수 |
| 세이브/로드(Save/Load) 엣지 케이스 | **수용** | ECS 직렬화/역직렬화 패닉은 가장 치명적이므로 최우선 엣지 케이스로 포함 |

### ❌ 수정/보완하는 항목
| 제안 | 수정 | 이유 |
|------|------|------|
| "스트레스 테스트 방안" | **E2E 동작이 먼저** | 폴리모프+장비해제 같은 엣지케이스는 기본 동작 후에 의미 있음 |
| "즉시 로드맵 문서 작성 후 코드 수정" | **동의하되 단계 세분화** | 외부감사의 4단계는 너무 거칠다. 내부적으로 8단계로 세분화 |

### 🔍 외부감사가 놓친 핵심 사항

1. **`cargo run` 자체가 안 될 가능성이 높다**
   - 177,229줄 중 상당수는 순수 결과 패턴(Pure Result)으로 이식되어 ECS와 **연결되지 않은 섬 코드**
   - `execute_turn_systems()`에 등록된 시스템은 약 25개뿐. 나머지 수백개 모듈은 테스트만 통과
   - **우선순위**: 게임이 "돌아가는" 것 = 이 25개 시스템이 패닉 없이 실행되는 것

2. **에셋 로딩 실패 가능성**
   - `assets/data/*.toml` 파일의 스키마와 현재 코드의 `serde` 역직렬화 구조체 사이 불일치 가능
   - `AssetManager::load_defaults()`가 성공해야 게임이 시작됨

3. **Legion ECS 런타임 Borrow Conflict**
   - 테스트에서는 단일 시스템만 실행. schedule.execute()에서 25개 시스템이 동시에 World/Resources에 접근할 때 Legion의 런타임 borrow checker가 패닉 발생 가능
   - 이것은 **컴파일 타임에 잡히지 않는** 런타임 이슈

4. **Grid/DungeonManager 초기화 누락**
   - `initialize_game_with_choices()`에서 Grid, DungeonLevel 등이 올바르게 생성되는지 미검증
   - Grid가 비어있으면 movement_system, vision_system 등이 즉시 패닉

---

## 1. 안정화 Phase 구조 (8단계)

```
Phase S0: 브랜치 생성 + 빌드 검증               ✅ 완료
Phase S1: 앱 기동 (cargo run → 창 뜨기)          ✅ 완료
Phase S2: 상태머신 관통 (Title → Playing 진입)    ✅ 완료
Phase S3: 첫 턴 생존 (Playing에서 1턴 실행)       ✅ 완료
Phase S4: N턴 루프 (1000턴 이상 패닉 없음)        ✅ 완료
Phase S5a: 게임 루프-ECS 상태 동기화 파이프라인   ✅ 완료 (Grid 역동기화 포함)
Phase S5b: 기본 상호작용 명령어 안정화            🔧 진행 중 ⬅️
Phase S5c: 인벤토리/전투/마법 명령어 연결         ⬜ 대기
Phase S6: Edge Case 방어                          ⬜ 대기
Phase S7: LLM 최소 주입                           ⬜ 대기
```

---

## 2. Phase 상세

### Phase S0: 브랜치 생성 + 빌드 검증

**목표**: 안정화 전용 브랜치에서 깨끗한 빌드 확인

- [x] `git checkout -b stabilize/e2e-playable` 생성
- [x] `cargo build` 에러 0개 확인
- [x] `cargo test` 4,177개 전량 통과 확인
- [x] 경고(warning) 목록 캡처 → 약 140개 (unused variables 등, 기능 영향 없음)

**판정 기준**: `cargo build` 성공, 테스트 전량 통과 — ✅ **달성**

---

### Phase S1: 앱 기동 (창 띄우기)

**목표**: `cargo run` → eframe 윈도우가 뜨고 Title 화면 렌더링

**예상 장애물**:
1. eframe/egui 의존성 버전 충돌
2. `NetHackApp::new()`에서 `AssetManager::load_defaults()` 실패 (TOML 파싱 에러)
3. ECS Resources 초기화 순서 문제

**작업**:
- [x] `main.rs` 기동 시 가장 먼저 `std::panic::set_hook` 설정
- [x] `cargo run` 실행 → 패닉 메시지 수집
- [x] AssetManager TOML 로딩 경로 확인/수정 (monsters.toml flags3 u16 초과 수정)
- [x] NetHackApp::new() 내 Resources 등록 순서 검증
- [x] Title 화면 렌더링 확인

**판정 기준**: 윈도우가 뜨고 Title 화면의 "New Game" 버튼이 보임 — ✅ **달성**

---

### Phase S2: 상태머신 관통

**목표**: Title → CharCreation → Playing 전체 흐름 무패닉

**예상 장애물**:
1. CharCreation → `initialize_game_with_choices()` 에서 패닉
   - 몬스터/아이템 템플릿 로딩 실패
   - Grid 생성 실패 (dungeon 초기화)
   - Player 엔티티 생성 시 컴포넌트 누락
2. AppState::Playing 전환 후 첫 `process_game_turn()` 호출 시 패닉

**작업**:
- [x] CharCreation 화면에서 역할/종족/이름 선택 → Done 클릭
- [x] `initialize_game_with_choices()` 디버깅
  - Grid 생성 (DungeonManager) — gen.rs 오버플로우 수정
  - Player 엔티티 + 초기 장비/인벤토리
  - 몬스터 스포닝 — 원본 NetHack 방식으로 조정 완료
  - Resources 등록 (TeleportAction 등 누락분 추가)
- [x] Playing 상태 진입 확인 (맵이 렌더링되는가?)

**판정 기준**: 맵+플레이어 '@'가 화면에 보임 — ✅ **달성**

---

### Phase S3: 첫 턴 생존

**목표**: 키 입력 1회 → `process_game_turn()` → `execute_turn_systems()` → 패닉 없음

**이 Phase가 가장 위험하다.**

**예상 장애물**:
1. `execute_turn_systems()`의 25개 시스템 중 하나가 패닉
   - *특히*: `movement_system` (Grid 타일 접근), `monster_ai_system` (몬스터 쿼리), `vision_update_system` (시야 계산)
2. Legion SubWorld borrow conflict (서로 다른 시스템이 같은 컴포넌트에 동시 접근)
3. `post_turn_processing()`에서 EventQueue/GameLog borrow 충돌

**전략**: 
```
execute_turn_systems() 내 시스템을 1개씩 활성화하며 디버깅
0개 → 1개(movement) → 2개(+ai) → ... → 25개(전체)
```

**작업**:
- [x] `execute_turn_systems()` 내 시스템 점진적 활성화
- [x] movement_system 활성화 → 방향키 이동 테스트 — ✅
- [x] +monster_ai_system → AccessDenied 발생 → write_component(Position) 추가로 수정
- [x] +death_system → 전투 사망 처리 확인
- [x] +trap_trigger → Resource not exist → AssetManager 구조 변경으로 수정
- [x] 25개 전체 활성화 → 1턴 무패닉

**판정 기준**: 방향키 입력 → '@' 이동 → 화면 갱신 → 패닉 없음 — ✅ **달성**

---

### Phase S4: N턴 루프

**목표**: 10턴 연속 + 몬스터와 조우 1회 이상 + 패닉 없음

**예상 장애물**:
1. 턴 카운터 오버플로우 또는 미증가
2. 몬스터 스폰 후 AI가 이동할 때 경계 체크 실패
3. EventQueue 누적 (clear 시점 오류)

**작업**:
- [x] `drain_action_queue()` 내 처리 횟수 카운터 도입
- [x] 1000턴 자동 실행 (테스트 스크립트) — 패닉 없이 완주
- [x] 턴 카운터 증가 확인
- [x] 몬스터 AI 이동 확인
- [x] GameLog 메시지 출력 확인
- [x] EventQueue→EventHistory 기록/클리어 확인

**판정 기준**: 1000턴 연속 패닉 없음, GameLog에 메시지 출력됨 — ✅ **달성**

**추가 수정 사항 (2026-03-29)**:
- ✅ 맵 생성: 몬스터 수 원본 공식(방당 33%) 적용, 특수 방 깊이 기반 확률로 변경
- ✅ 몬스터 스포닝: HP를 d(m_lev,8)로 안정화, 그룹 크기 rnd(n)으로 제한
- ✅ 난이도 필터: depth+5 제한 + 거리 기반 가중치
- ✅ AIHack 전용 스폰(beast_horde, CommBase, SupplyDepot) 비활성화
- ✅ dodoor: 벽 타일 조건으로 수정

---

### Phase S5: 핵심 상호작용 (3단계로 세분화)

> ⚠️ **2026-03-29 진단 결과**: 이식된 로직 함수는 대부분 존재하지만, 게임 루프(`game_loop.rs`)와의 연결이 
> 불완전하여 실제 게임에서 동작하지 않는 커맨드가 다수 존재함. 구조적으로 세 가지 문제가 확인됨:
> 1. **Grid 상태 동기화 부재**: 커맨드가 `self.game.grid`를 수정하지만 `resources`에 반영되지 않음
> 2. **분산된 로직**: 시스템(Schedule) vs 직접 Grid 조작이 혼재하여 버그 추적 어려움
> 3. **Legion ECS AccessDenied**: `split_for_query`에서 컴포넌트 접근 선언 불일치로 런타임 패닉

---

#### Phase S5a: 게임 루프 - ECS 상태 동기화 파이프라인 확립 ⬅️ **최우선**

**목표**: 어떤 커맨드가 Grid/엔티티를 수정하더라도, 그 변경이 즉각 렌더링에 반영되는 "단일화된 반영 통로" 구축

**현재 문제**:
- `Open/Close/Kick` 등 방향 액션의 `_ =>` 분기에서 Grid 수정 후 `resources.insert(grid.clone())` 누락
- `split_for_query` 사용 시 선언되지 않은 컴포넌트 접근으로 AccessDenied 패닉 (Search, Pray, Sit, Talk에서 확인)

**작업**:
- [x] `execute_direction_action` 의 `_ =>` 분기에서 Grid 변경 후 resources 동기화 추가 (2026-03-30)
- [x] `split_for_query` 사용 지점(Pray, Sit, Talk) 전부 직접 World 접근으로 교체 (2026-03-30)
- [x] Grid 동기화: `_ =>` 분기에서 `resources.insert(grid.clone())` 추가, borrow conflict을 remove/insert 패턴으로 해결

**판정 기준**: `o` (Open) + 방향키 → 실제로 문이 열림, `c` (Close) → 문이 닫힘, `K` (Kick) → 발차기 동작

**이전 수정 이력** (본 세션에서 완료):
- ✅ `Search (s)`: split_for_query → 직접 World 접근으로 수정 (AccessDenied 해결)
- ✅ `monster_ai`: `#[write_component(Position)]` 추가 (AccessDenied 해결)
- ✅ `Trap` 시스템: AssetManager를 통한 리소스 접근으로 구조 변경

**근본 수정 (2026-03-30)**: Grid 이중화 (Dual Grid) 문제 해결
- **문제**: `self.game.grid` (렌더러 소스)와 `resources.Grid` (시스템 소스)가 분리되어 있었음
  - 89행에서 `resources.insert(self.game.grid.clone())` → 시스템은 복사본을 수정
  - 시스템 실행 후 수정된 Grid를 `self.game.grid`로 역복원하지 않음
  - **결과**: 모든 시스템의 Grid 변경(문 열기, 전투, 함정 등)이 화면에 반영되지 않음
- **수정**: `execute_turn_systems()` 직후 `resources.Grid → self.game.grid` 역동기화 추가
- **추가 수정**: 시스템 실행 조건을 `last_cmd != Unknown || _action_executed`로 확장
- **추가 수정**: 방향 입력 소비(`last_cmd = Unknown`) 추가로 이동+액션 중복 실행 방지
- **추가 수정**: 몬스터 원거리 사망 시 불필요한 경험치/메시지 필터링 (거리 기반)

---

#### Phase S5b: 기본 상호작용 명령어 안정화

**목표**: NetHack의 기본 동사(verb) 중 '방향 입력 + 타일/아이템 상호작용' 그룹이 정상 동작

**대상 커맨드**:
- [x] **이동**: 8방향 + 대기(.) — ✅ 동작 확인됨
- [x] **공격**: 몬스터 인접 이동 → 전투 메시지 — ✅ 시각적 검증 완료 (Grid 역동기화)
- [x] **문 열기 (o)**: 방향 → Door → OpenDoor 변경 — ✅ 시각적 검증 완료 (Grid 역동기화)
- [x] **문 닫기 (c)**: 방향 → OpenDoor → Door 변경 — ✅ 코드 연결 완료
- [x] **발차기 (K)**: 방향 → 문 파괴/밀치기 — ✅ 코드 연결 완료
- [x] **아이템 줍기 (,)**: 바닥 아이템 → 인벤토리 — ✅ 코드 연결 완료
- [x] **계단 (> / <)**: 레벨 이동 — 기존 LevelChange 시스템으로 연결됨
- [x] **사망**: HP 0 → GameOver 전환 — 기존 death_system으로 연결됨
- [x] **탐색 (s)**: 숨겨진 문/함정 발견 — ✅ 동작 확인됨

**판정 기준**: 위 9개 동사가 패닉 없이 실행되고 실제 게임 상태가 변경됨

---

#### Phase S5c: 인벤토리 및 전투/마법 명령어 연결

**목표**: 인벤토리를 열어 타겟을 선택하는 복합 커맨드 그룹이 동작

**대상 커맨드**:
- [ ] **인벤토리 (i)**: 인벤토리 팝업 표시
- [ ] **장비 착용 (W/P)**: Wear(갑옷) / Put on(악세서리) 선택 → 장비
- [ ] **무기 장착 (w)**: Wield → 무기 교체
- [ ] **포션 마시기 (q)**: Quaff → 포션 효과 적용
- [ ] **스크롤 읽기 (r)**: Read → 스크롤 효과 적용
- [ ] **마법 시전 (Z)**: Cast → 방향/타겟 → 마법 효과
- [ ] **지팡이 (z)**: Zap → 방향 → 빔/볼트 효과
- [ ] **던지기 (t)**: Throw → 아이템 선택 → 방향 → 투사체 이동
- [ ] **식사 (e)**: Eat → 음식 선택 → 영양/효과 적용
- [ ] **도구 사용 (a)**: Apply → 도구 선택 → 효과
- [ ] **기도 (#pray)**: 신앙 시스템 → 보상/벌칙

**구현 시 주의사항**:
- UI(인벤토리 창, 프롬프트)와 게임 로직 간의 `GameState` 상태 머신이 꼬이지 않도록 주의
- 각 커맨드에서 `_action_executed = true` 설정 확인 (턴 소비 여부)

**판정 기준**: 위 커맨드가 패닉/크래시 없이 실행되고, 장비/스탯 변경이 즉각 UI에 반영됨

---

> **전체 진행 순서**: S5a (파이프라인) → S5b (기본) → S5c (복합)
> S5a를 완료해야 S5b/S5c가 의미가 있음 (Grid 동기화 없이는 어떤 커맨드도 제대로 안됨)

**판정 기준**: 위 전체 커맨드가 패닉 없이 실행

---

### Phase S6: Edge Case 방어

**목표**: 복합 상호작용에서 패닉/데드락 없음

- [ ] **세이브/로드 (중요)**: 10턴 진행 후 Save & Quit → 앱 완전 종료 → 재시작 후 Load → 1턴 생존 (ECS Entity 직렬화 검증)
- [ ] 사망 → GameOver 화면 → New Game → 재시작
- [ ] 레벨 변경 (계단 내려가기 → 2층 생성 → 올라가기)
- [ ] 상점 진입 (상점 타일 위 이동 시 메시지)
- [ ] 포션 사용 (인벤토리 → 포션 선택 → 효과)
- [ ] 마법 주문(비상 존: zap 계열)
- [ ] 다중 상태이상 (독+혼란+실명 중첩)

**판정 기준**: 위 항목에서 패닉(특히 Save/Load 관련) 0건

---

### Phase S7: LLM 최소 주입 (최종)

> ⚠️ **Phase S0~S6 전체 완료 후에만 진입 가능**

**목표**: 결정론적 게임 루프 위에 LLM 텍스트 생성을 최소 범위로 주입

**주입 순서** (의존성 낮은 것부터):

| 순서 | 대상 | LLM 역할 | 실패 시 폴백 |
|------|------|----------|-------------|
| 1 | `death.rs` 묘비명 | 사망 메시지 꾸밈 | 기존 하드코딩 텍스트 |
| 2 | `GameLog` 메시지 | 전투/이벤트 서술 | `to_narrative()` 기본값 |
| 3 | NPC 대화 | 오라클/상점주인 대사 | `ScriptedDialogue` 기본값 |
| 4 | 상점 가격 흥정 | 자연어 가격 제안 | 고정 공식 |
| 5 | 던전 서술 | 레벨 진입 시 분위기 | 고정 텍스트 |

**원칙**:
- 모든 LLM 호출에 **타임아웃(2초)** 설정
- 타임아웃 시 **폴백 텍스트** 반환 (게임 멈춤 절대 없음)
- LLM 호출은 **게임 로직과 완전 분리** (순수 텍스트 생성만)
- `InteractionProvider` 트레이트의 `DefaultInteractionProvider` → `LlmInteractionProvider` 전환

---

## 3. 디버깅 전략

### 3.1 결정론적 재현 보장

```rust
// RNG 시드 고정 → 항상 같은 던전/몬스터 배치
let rng = NetHackRng::new(42);
```

- 모든 디버깅은 **시드 42로 고정**하여 동일 상황 재현 보장
- Phase S3~S4에서 발견된 버그는 시드+턴 수로 재현 가능

### 3.2 시스템 격리 디버깅

```
[전략: 점진적 시스템 활성화]

Step 1: 빈 스케줄 (시스템 0개) → 입력만 확인
Step 2: movement_system만 → 이동 확인
Step 3: +vision_system → 시야 확인
Step 4: +monster_ai → AI 확인
Step 5: +death_system → 전투/사망 확인
...
Step 25: 전체 활성화
```

각 Step에서 패닉 발생 시 **해당 시스템만 디버깅** → 원인 격리 용이

### 3.3 ActionQueue/EventQueue 추적 방어

```rust
// 무한 연쇄 방지 (Cascade Limit)
const MAX_QUEUE_DEPTH: usize = 100;
let mut processed_count = 0;
while let Some(action) = queue.pop() {
    processed_count += 1;
    if processed_count > MAX_QUEUE_DEPTH {
        panic!("Cascade Limit Exceeded! Possible infinite loop detected in ActionQueue.");
    }
    // ...
}

// 디버그 모드에서 큐 상태 로깅
println!("[T{}] ActionQueue: {} / EventQueue: {}", 
    turn, action_queue.len(), event_queue.len());
```

- 한 턴 안의 연쇄 상호작용(예: 피격 -> 산성 반응 -> 장비파괴 -> 피격 로그)의 무한루프 방지벽 설치
- 매 턴 큐 크기 출력 → 누적/누수 감지
- EventQueue가 clear되지 않으면 메모리 누수 경고

---

## 4. 성공 기준 요약

| Phase | 기준 | 예상 소요 |
|-------|------|-----------|
| S0 | 빌드 + 테스트 통과 | 30분 |
| S1 | 윈도우 뜨기 | 2~4시간 |
| S2 | Playing 진입 (맵 보기) | 4~8시간 |
| S3 | 첫 턴 (이동 1회) | 8~16시간 ⚠️ **최대 난관** |
| S4 | 10턴 연속 | 4~8시간 |
| S5 | 7개 동사 동작 | 8~16시간 |
| S6 | Edge Case 방어 | 8~16시간 |
| S7 | LLM 최소 주입 | 별도 계획 |

**총 예상**: S0~S6까지 약 **35~68시간** (집중 작업 기준)

---

## 5. 브랜치 전략

```
main (v2.41.0 = 100% 순수 번역본, 불변)
 └─ stabilize/e2e-playable
      ├─ S0: build-verified
      ├─ S1: window-launch
      ├─ S2: state-machine-flow
      ├─ S3: first-turn
      ├─ S4: ten-turn-loop
      ├─ S5: core-interactions
      ├─ S6: edge-cases
      └─ S7: llm-minimal (별도 브랜치 가능)
```

- main은 **절대 오염시키지 않는다** (순수 번역본 보존)
- 각 Phase 완료 시 **태그 + 커밋 메시지**로 마일스톤 기록
- S6 완료 시 main으로 PR/머지 검토

---

**문서 버전**: v1.0
**최종 업데이트**: 2026-02-28
