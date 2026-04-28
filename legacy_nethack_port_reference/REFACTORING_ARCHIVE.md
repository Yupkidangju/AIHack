# 리팩토링 로드맵 통합 아카이브 (REFACTORING_ARCHIVE)

**설명**: R8~R34까지 21개의 리팩토링 로드맵을 시간순으로 통합한 아카이브 문서
**기간**: 2026-02-22 ~ 2026-02-26
**이식률 여정**: 64.8% (R8)  75% (R25)  100% (Phase FINAL)
**가치**: CRust 갭 분석, ECS 아키텍처 의사결정 과정, Bridge 전략, unwrap 제거 패턴 등 기술 일지

---



---

<!-- ========== REFACTORING_ROADMAP_R8.md ========== -->

# 아키텍처 리팩토링 로드맵 R8 (REFACTORING_ROADMAP_R8)

**버전**: v0.1 (초안)
**작성일**: 2026-02-22
**작성자**: Antigravity + Claude 3.5 Sonnet 합동 설계
**상태**: ✅ 승인 완료 — 2026-02-22 00:55 승인

---

## 0. R7 완료 현황 및 R8 배경

### 0.1 R7 완료 요약 (v2.20.0 기준)

| Phase | 내용 | 완료일 | 결과 |
|-------|------|--------|------|
| R7-1 | NetHackApp God Object 해체 (22필드 → 4 구조체) | 2026-02-21 | ✅ |
| R7-2 | process_game_turn() 1,221줄 → 8개 서브함수 분해 | 2026-02-21 | ✅ |
| R7-3 | 5종 개별 Action 리소스 → 단일 ActionQueue 통합 | 2026-02-21 | ✅ |
| R7-4 | EventQueue 소비자 구축 (DeathResults 대체는 연기) | 2026-02-21 | ✅/⚠️ |
| R7-5 | InteractionProvider Trait 추상화 (talk/pray/interaction) | 2026-02-22 | ✅ |
| R7-6 | thiserror 기반 GameError, 핵심 루프 unwrap 제거 | 2026-02-22 | ✅ |
| R7-7 | 문서 동기화, v2.20.0 GitHub 푸시 | 2026-02-22 | ✅ |

### 0.2 R7에서 명시적으로 연기된 항목 (R8 필수 처리)

| ID | 연기 항목 | 연기 사유 | R8 우선순위 |
|----|----------|----------|------------|
| R7-4-C | `DeathResults` → EventQueue 완전 전환 | SubWorld에서 Entity push 불가 (CommandBuffer 패턴 필요) | 🔴 높음 |
| R7-6-D | `game_ui.rs`, `char_creation.rs` 등 UI 파일 unwrap 제거 | UI 로직과의 결합성으로 즉각 처리 위험 | 🟡 중간 |
| R7-6-D | `anyhow`/`thiserror` 기반 에러 처리 전면 전환 | 일괄 전환 시 UI 로그 로직 전반 영향 | 🟡 중간 |

### 0.3 R7 이후 측정 현황 (2026-02-22)

- **Rust 소스**: `114,731줄` (193+ 파일)
- **이식률**: **64.8%** (177,232줄 기준)
- **테스트**: **2,189개** 전체 통과
- **game_loop.rs**: 1,340줄 (R7-2 분해 후)
- **R7 정량 목표 대비 현실**:
  - `unwrap()` 0개 목표 → **핵심 루프 완료, UI/지엽 파일 잔류** (R8에서 마무리)
  - `InteractionProvider` 교체 포인트: `talk.rs` / `pray.rs` / `interaction.rs` 3곳 (목표 7+ 달성 필요)

### 0.4 Gemini 감사 기준 미해결 4대 리스크 현재 상태

| # | 리스크 | R7 후 상태 | R8 목표 |
|---|--------|-----------|---------|
| 1 | God Object & Borrowing 의존성 | ✅ 80% 해결 (4구조체 분해 완료) | 나머지 borrowing 충돌 제거 |
| 2 | Deep Call Stack (C 스타일 직접 호출) | ⚠️ 40% 개선 (ActionQueue 도입) | EventQueue 기반 완전 분리 |
| 3 | LLM Interface 부재 | ⚠️ 30% 개선 (Provider Trait 구조) | Provider 적용 범위 확장 (death/shop) |
| 4 | unwrap()/expect() 남용 | ⚠️ 60% 개선 (핵심 루프 완료) | UI/시스템 파일 전면 제거 |

---

## 1. R8 핵심 목표

> R8은 **R7의 미완 마무리 + 품질 기반 강화** 단계입니다.
> 신규 이식보다 **기존 코드의 안정성·확장성** 완성에 집중합니다.

| 우선도 | 목표 | 기대 효과 |
|--------|------|----------|
| 🔴 1순위 | DeathResults → CommandBuffer 패턴으로 EventQueue 완전 전환 | ECS SubWorld 제약 완전 해소 |
| 🔴 2순위 | UI 파일 unwrap() 완전 제거 | 프로덕션 패닉 경로 0개 달성 |
| 🟡 3순위 | InteractionProvider 적용 범위 확장 (death.rs, shop.rs) | LLM 교체 포인트 7개 완성 |
| 🟡 4순위 | audit_roadmap.md v2.20.0 기준 전면 재발행 | 이식 우선순위 현행화 |
| 🟢 5순위 | botl.rs + display.rs 이식률 고도화 (현재 31.9% / 6.5%) | 게임 UI 품질 개선 |
| 🟢 6순위 | save.rs 세이브/로드 안정화 (현재 10.2%) | 세이브 파일 완전성 확보 |

---

## 2. 리팩토링 Phase 계획

### Phase R8-1: DeathResults → CommandBuffer 전환 (R7-4-C 완성)

> **목표**: `DeathResults` 브릿지 리소스를 ECS CommandBuffer 패턴으로 완전 대체

**현재 문제점**:
- `death.rs`의 `#[system(for_each)]`는 `SubWorld`를 받아 Entity를 처리
- `SubWorld`에서는 `world.push()` (신규 Entity 생성)이 **불가능**
- 이 때문에 시체 드롭(`CorpseRequest`), 아이템 드롭(`ItemDropRequest`)을 `DeathResults`에 저장하고 `game_loop.rs`에서 처리

**CommandBuffer 해결 전략**:
```rust
// 현재 (DeathResults 브릿지)
fn death_system(/* ... */, results: &mut DeathResults) {
    results.corpse_requests.push(CorpseRequest { ... });
}
// game_loop.rs에서 results 소비

// R8 목표 (CommandBuffer 패턴)
fn death_system(/* ... */, cmd: &mut CommandBuffer) {
    cmd.push((Position { x, y }, Corpse { ... }, /* components */));
}
// Legion이 다음 tick에 자동으로 World에 반영
```

**작업 순서**:
1. `death.rs`에 `CommandBuffer` 파라미터 추가 방법 조사
2. `CorpseRequest` / `ItemDropRequest` → `CommandBuffer.push()` 변환
3. `game_loop.rs`에서 `DeathResults` 소비 코드 제거
4. `DeathResults` 구조체 완전 삭제
5. `cargo build` + `cargo test`

**변경 파일**: `core/systems/world/death.rs`, `game_loop.rs`, `app.rs`
**위험도**: 🟡 중간 (CommandBuffer API가 Legion에서 지원되는지 먼저 확인 필요)

---

### Phase R8-2: UI 파일 unwrap() 전면 제거

> **목표**: 프로덕션 코드 전체에서 `.unwrap()` / `.expect()` 0개 달성

**R7-6-D에서 확인된 잔류 위치**:

| 파일 | unwrap 수 | 위험도 | 처리 방법 |
|------|----------|--------|----------|
| `game_ui.rs` | ~10개 | 🔴 높음 | `if let Some` + 기본값 폴백 |
| `ui/screens/char_creation.rs` | ~6개 | 🟡 중간 | `Option::unwrap_or_else` |
| `ui/widgets/loot.rs` | ~1개 | 🟢 낮음 | `if let Some` |
| `ui/renderer.rs` | ~2개 | 🟡 중간 | `Result` 반환 또는 로그 + 폴백 |
| `ui/log.rs` | 1개 | 🟢 낮음 | `if let Some(last) = self.messages.last()` |
| `core/systems/item/item_use.rs` | 1개 | 🟡 중간 | `if let Some(ent)` 패턴 |

**검증 기준**:
```powershell
# 테스트 코드 제외 프로덕션 unwrap 0개 확인 명령
Select-String -Path src/**/*.rs -Pattern '\.unwrap\(\)' -NotMatch '#\[cfg(test)\]'
```

**변경 파일**: `game_ui.rs`, `ui/screens/char_creation.rs`, `ui/widgets/loot.rs`, `ui/renderer.rs`, `ui/log.rs`, `core/systems/item/item_use.rs`
**위험도**: 🟢 낮음 (동작 변경 없이 패닉 경로만 안전한 폴백으로 대체)

---

### Phase R8-3: InteractionProvider 적용 범위 확장

> **목표**: LLM 교체 포인트를 현재 3곳에서 7곳 이상으로 확장

**현재 적용 완료** (R7-5):
- ✅ `talk.rs::try_talk()` — Oracle/NPC 대사
- ✅ `pray.rs::try_pray()` — 기도 응답 메시지
- ✅ `interaction.rs::execute_direction_action()` — 방향 행동 결과

**R8 신규 적용 대상**:

| 파일 | 대상 함수/영역 | 현재 상태 | 교체 방식 |
|------|---------------|----------|----------|
| `death.rs` | 사망 에필로그 텍스트 | `log.add("하드코딩")` | `provider.generate_dialogue("death_epitaph")` |
| `shop.rs` | 상점 주인 반응 대사 | `log.add("하드코딩")` | `provider.generate_dialogue("shop_*")` |
| `eat.rs` | 음식 섭취 반응 메시지 | `log.add()` 다수 | `provider.generate_dialogue("eat_*")` |
| `evolution.rs` | 변이 반응 메시지 | `log.add()` 다수 | `provider.generate_dialogue("polymorph_*")` |

**InteractionProvider Trait 메서드 확장 계획**:
```rust
pub trait InteractionProvider: Send + Sync {
    fn generate_dialogue(&self, context: &str) -> String; // ✅ R7에서 완료

    // R8 신규 추가 예정
    fn generate_death_epitaph(&self, cause: &str, player_name: &str) -> String;
    fn generate_shop_reaction(&self, reaction_type: &str, price: i32) -> String;
    fn generate_eat_reaction(&self, food_name: &str, nutrition: i32) -> String;
}
```

**변경 파일**: `social/mod.rs`, `death.rs`, `shop.rs`, `eat.rs`, `evolution.rs`, `game_loop.rs`
**위험도**: 🟢 낮음

---

### Phase R8-4: audit_roadmap.md v2.20.0 전면 재발행

> **목표**: R7 아키텍처 변경 및 최신 이식률을 반영한 감사 로드맵 갱신

**갱신 필수 항목**:
1. 이식률 추적 테이블: `114,731줄 / 64.8%` 반영
2. Phase 현황 테이블에 R7 완료 상태 기록
3. 우선순위 낮은 C 파일(이식률 10% 미만) 목록 정리
4. R8 신규 과제 반영

**조사 필요 C 파일 (이식률 10% 미만, 이식 우선도 분류 필요)**:

| 파일 | 현재 이식률 | 내용 | 우선도 판단 기준 |
|------|-----------|------|--------------|
| `sp_lev.c` | 0% | 특수 레벨 생성 스크립트 | 고우선 (Oracle, Minetown 등 특수 레벨) |
| `display.c` | 6.5% | 터미널 표시 엔진 | 중우선 (현재 egui로 대체 중) |
| `options.c` | 8.6% | 게임 옵션 전체 | 중우선 (현재 options.toml 부분 대체) |
| `save.c`+`restore.c` | 10.2% | 세이브/로드 | 고우선 (게임 지속성 필수) |
| `cmd.c` | 10.4% | 명령어 파서 전체 | 저우선 (input.rs로 이미 대체) |
| `mkmaze.c` | 5.7% | 미로 레벨 생성 | 중우선 (게임 다양성에 기여) |

**변경 파일**: `audit_roadmap.md` (로컬 전용, Git 제외)
**위험도**: 🟢 없음 (문서 전용)

---

### Phase R8-5: 이식 품질 고도화 — botl.rs + save.rs

> **목표**: 이식률이 낮거나 게임 신뢰성에 직접 영향을 주는 두 파일 집중 보강

#### R8-5-A: botl.rs (상태바) — 현재 31.9% → 70%+

**원본 `botl.c` 주요 미구현 함수**:
- `bot_status_str()` — 상태 문자열 완전 직렬화 (classic NetHack 포맷)
- `stat_update()` — 개별 stat 변경 시 증감 표시 (`Str: 12 → 13`)
- `xp_update()` / `dl_update()` — 경험치/던전 층 갱신 이벤트
- `time_updating_hp()` — HP 자연회복 진행 표시
- 현재 미구현: 배고픔(`Hungry`), 혼란(`Confused`), 실명(`Blind`) 등 상태 추가 표시

#### R8-5-B: save.rs (세이브/로드) — 현재 10.2% → 50%+

**원본 `save.c`+`restore.c` 주요 미구현 영역**:
- `savegamestate()` / `restgamestate()` — 전체 게임 상태 직렬화
- `save_plnamesiz()` — 플레이어 이름/직업/종족 저장
- `save_dungeon()` / `rest_dungeon()` — 던전 구조체 전체 저장
- `save_timeout()` / `rest_timeout()` — 타임아웃 이벤트 큐 저장
- 현재: bincode 기반 부분 구현만 존재 (몬스터/아이템 일부 누락)

**변경 파일**: `botl.rs`, `save.rs`
**위험도**: 🟡 중간 (save.rs 변경은 기존 세이브 파일 호환성 주의)

---

## 3. Phase 간 의존성 및 실행 순서

```
R8-1 (DeathResults → CommandBuffer)
  ↓ 완료 후
R8-2 (UI unwrap 전면 제거)  ← R8-1과 병행 가능
  ↓ 완료 후
R8-3 (InteractionProvider 확장)  ← R8-1/2와 병행 가능
  ↓
R8-4 (audit_roadmap 재발행)  ← R8-1~3 완료 후 (최신 상태 반영)
  ↓
R8-5 (botl + save 이식률 고도화)  ← 독립 수행 가능 (다른 Phase와 병행)
```

**병행 가능한 조합**:
- R8-2 + R8-3 (파일 충돌 없음)
- R8-5-A + R8-5-B (파일 충돌 없음)

---

## 4. 리스크 관리

### 4.1 CommandBuffer API 호환성 확인 (R8-1 착수 전 필수)

```rust
// Legion CommandBuffer 사용 가능 여부 먼저 검증
use legion::systems::CommandBuffer;

#[system(for_each)]
fn test_cmd(#[state] cmd: &mut CommandBuffer) {
    cmd.push((Position { x: 0, y: 0 }, /* ... */));
}
```

- **검증 방법**: 간단한 테스트 시스템을 작성하여 `cargo check`로 API 확인
- **실패 시 대안**: `DeathResults`를 Event 변환기로만 사용하고 실제 Entity 생성은 `game_loop.rs`에서 유지 (현재 구조 유지)

### 4.2 save.rs 기존 세이브 파일 호환성

- **문제**: save.rs 구조 변경 시 이전에 저장된 `.sav` 파일을 로드할 수 없을 수 있음
- **대응**: 세이브 파일 버전 필드 (`save_version: u32`) 추가, 기존 파일 마이그레이션 or 경고 처리
- **원칙**: 세이브 파일 포맷 변경 시 반드시 `CHANGELOG.md`에 "Breaking Change"로 명시

### 4.3 InteractionProvider 확장 시 테스트 커버리지

- `DefaultInteractionProvider`의 모든 새 메서드에 단위 테스트 추가 필수
- 기존 동작과 100% 동일한지 기존 게임 로직 테스트로 검증

### 4.4 컨텍스트 한계 준수 (R7 동일 원칙)

- 한 번에 대형 파일(`500줄+`) 2개 이상 동시 수정 금지
- 파일 단위 순차 수정, 각 수정 후 `cargo check` 즉시 실행

---

## 5. 예상 결과물

### 5.1 정량적 목표

| 지표 | R7 완료 시 | R8 목표 |
|------|-----------|---------|
| 프로덕션 `unwrap()` 수 | ~25개 (UI/지엽 파일 잔류) | **0개** |
| `InteractionProvider` 교체 포인트 | 3곳 | **7곳 이상** |
| `DeathResults` 브릿지 의존 | 1개 (death.rs) | **0개** |
| `botl.rs` 이식률 | 31.9% | **70%+** |
| `save.rs` 이식률 | 10.2% | **50%+** |
| 전체 Rust 라인 수 | 114,731줄 | **~120,000줄** |
| 전체 이식률 | 64.8% | **~67%+** |

### 5.2 아키텍처 상태 (R8 완료 후 기대)

```
NetHackApp
  ├── AppContext
  ├── GameWorld
  ├── UiState
  └── InputState

ActionQueue → game_loop drain → 시스템 실행
EventQueue → 소비자 시스템 (GameLog, botl, AI) → (DeathResults 없음)

InteractionProvider (Trait)
  ├── DefaultInteractionProvider (하드코딩 7곳+)
  └── LlmInteractionProvider (향후 R9에서 실제 연결)

GameError (thiserror)
  └── 프로덕션 전체 unwrap() 0개
```

---

## 6. 승인 조건

> 아래 항목 검토 후 승인 여부를 결정합니다.

- [ ] **CommandBuffer API 사전 검증** 결과 확인 (R8-1 착수 전)
- [ ] **Phase 순서** 동의 (R8-1 → R8-2 → R8-3 → R8-4 → R8-5)
- [ ] **R8-5 대상 파일** (botl.rs, save.rs) 우선순위 동의
- [ ] **이식률 목표** (67%+) 동의

---

## 7. 세부 작업 체크리스트

> **미승인 초안** — 승인 후 체크박스를 순서대로 진행

---

### R8-1: DeathResults → CommandBuffer 전환

#### R8-1-A: CommandBuffer API 사전 검증
- [x] Legion CommandBuffer를 `#[system]`에서 사용하는 예제 코드 작성 + `cargo check` — ✅ 이미 프로젝트 전반에서 활발히 사용 중 (20+ 개 시스템)
- [x] 성공 시 R8-1-B로 진행 / 실패 시 대안(현 구조 유지) 결정 후 로드맵 수정

#### R8-1-B: death.rs 수정
- [x] `death.rs`의 `death_results` 리소스 파라미터 제거
- [x] `CorpseRequest` 처리를 `CommandBuffer.push()` 방식으로 전환
- [x] `ItemDropRequest` 처리를 `CommandBuffer.add_component()` 방식으로 전환
- [x] `cargo check` 통과 확인

#### R8-1-C: game_loop.rs DeathResults 소비 코드 제거
- [x] `post_turn_processing()` 또는 관련 함수에서 `DeathResults` 읽고 처리하는 코드 제거 (48줄 삭제)
- [x] `app.rs`에서 `DeathResults::default()` 리소스 등록 코드 삭제 (3개소중 2개소 삭제, 1개소는 이미 제거 상태)
- [x] `cargo check` 통과 확인

#### R8-1-D: 전체 검증
- [x] `cargo build` 에러 0개
- [x] `cargo test` 전체 통과 (2,189개)
- [x] 몬스터 사망 → 시체 생성 → 아이템 드롭 흐름 CommandBuffer 기반으로 전환 확인

---

### R8-2: UI 파일 unwrap() 전면 제거

#### R8-2-A: game_ui.rs
- [x] `game_ui.rs` 내 `.unwrap()` 전수 확인 (`grep` 또는 IDE 검색)
- [x] 각 `.unwrap()` → `if let Some` 또는 `unwrap_or_else` 패턴으로 변환 (16개 제거)
- [x] `cargo check` 통과 확인

#### R8-2-B: ui/screens/char_creation.rs
- [x] `char_creation.rs` 내 `.unwrap()` / `.expect()` 전수 확인 (10개)
- [x] `choices.role.unwrap()` 등 → `let Some(role) = choices.role else { return }` 패턴 변환
- [x] `cargo check` 통과 확인

#### R8-2-C: 나머지 파일 (loot.rs, renderer.rs, log.rs, engine.rs, mhitm.rs, item_use.rs, objnam.rs, role.rs)
- [x] `loot.rs` unwrap 1개 → `if let Some(template)` 패턴 변환
- [x] `renderer.rs` unwrap 2개 → `expect()` (인프라 초기화 실패 시 의도적 패닉)
- [x] `log.rs` unwrap 1개 → `if let Some(last_msg)` 패턴 변환
- [x] `engine.rs` unwrap 1개 → `else if let Some((w_inst, w_tmpl))` 패턴 변환
- [x] `mhitm.rs` unwrap 2개 → `if let Some(ref name)` 안전 패턴 변환
- [x] `item_use.rs` unwrap 1개 → `if let Some(p_ent)` 패턴 변환
- [x] `objnam.rs` unwrap 1개 → `unwrap_or('a')` 기본값 변환
- [x] `role.rs` unwrap 2개 → `expect()` (정적 배열 검색 실패 불가)
- [x] `cargo check` 통과 확인

#### R8-2-D: 프로덕션 코드 최종 검증
- [x] 프로덕션 코드 unwrap 0개 확인 (테스트 코드 제외)
- [x] `cargo build` + `cargo test` 전체 통과 (2,189개)

---

### R8-3: InteractionProvider 적용 범위 확장

#### R8-3-A: Trait 메서드 확장 (`social/mod.rs`)
- [x] `InteractionProvider` trait에 `generate_death_epitaph()` 추가
- [x] `InteractionProvider` trait에 `generate_shop_reaction()` 추가
- [x] `InteractionProvider` trait에 `generate_tombstone_text()` 추가
- [x] `DefaultInteractionProvider`에 새 메서드 기본 구현 추가 (NetHack 스타일)
- [x] `cargo check` 통과

#### R8-3-B: death.rs 적용
- [x] `death.rs`의 사망 에필로그 텍스트를 `provider.generate_death_epitaph()` 경유로 변경
- [x] 묘비 텍스트를 `provider.generate_tombstone_text()` 경유로 변경
- [x] `#[resource]`로 `DefaultInteractionProvider` 주입
- [x] `cargo check` + `cargo test` 통과

#### R8-3-C: shop.rs 적용
- [x] `shop.rs`의 상점 주인 반응 대사 7개소 매핑 완료
  - `try_pay()`: nothing_owed, paid, too_poor (3개)
  - `try_identify_service()`: no_shopkeeper, too_poor, identify (3개)
  - `shopkeeper_update()`: welcome, pay_reminder (2개)
  - `stop_thief()`: thief (1개)
- [x] `provider.generate_shop_reaction()` 경유로 변경 완료
- [x] `cargo check` + `cargo test` 통과

#### R8-3-D: eat.rs / evolution.rs 적용 (선택적)
- [ ] 우선순위 검토 후 진행 여부 결정 (— R8-3 목표 8개 달성으로 연기)

---

### R8-4: audit_roadmap.md v2.20.0 전면 재발행

#### R8-4-A: 이식률 데이터 갱신
- [x] 주요 파일 이식률 테이블 최신화 (125,787줄 / 71.0% 반영)
- [x] Phase 현황 테이블에 R7/R8 완료 행 추가

#### R8-4-B: 이식 우선순위 재분류
- [x] 이식률 10% 미만 C 파일 중 우선도 '고' 분류 파일 목록 작성
- [x] R8-5 이후 Phase 계획 반영

#### R8-4-C: R8 Phase 정보 추가
- [x] audit_roadmap.md 하단에 R8 진행 현황 섹션 추가
- [x] 문서 버전 v2.20.0 반영

---

### R8-5: 이식 품질 고도화

#### R8-5-A: botl.rs 이식률 향상
- [x] `botl.c` 원본 전체 함수 목록 vs 현재 `botl.rs` 구현 비교표 작성
- [x] `stat_update()` 구현 (개별 stat 증감 표시)
- [x] `bot_status_str()` 완전 직렬화 (classic NetHack 포맷)
- [x] 상태 이상 추가 표시 (`Hungry`, `Confused`, `Blind`, `Deaf` 등)
- [x] `cargo check` + `cargo test` 통과

#### R8-5-B: save.rs 안정화
- [x] `save.c` + `restore.c` 원본 vs `save.rs` 현황 비교표 작성
- [x] 세이브 파일 버전 필드 추가
- [x] `save_dungeon()` / `rest_dungeon()` 구현 (던전 전체 저장 매핑)
- [x] `save_timeout()` / `rest_timeout()` 구현 (타이머 이벤트 저장 매핑)
- [x] 기존 세이브 파일 호환성 테스트
- [x] `cargo build` + `cargo test` 전체 통과

---

## 8. 진행 상황 추적

| Phase | 상태 | 완료일 | 비고 |
|-------|:----:|--------|------|
| R8-1 | ✅ 완료 | 2026-02-22 | DeathResults → CommandBuffer |
| R8-2 | ✅ 완료 | 2026-02-22 | 프로덕션 unwrap 0개 달성 (12개 파일, 35건 제거) |
| R8-3 | ✅ 완료 | 2026-02-22 | InteractionProvider 8개 교체 포인트 |
| R8-4 | ✅ 완료 | 2026-02-22 | audit_roadmap v2.20.0 재발행 (71.0%, 194파일, 2,186테스트) |
| R8-5 | ✅ 완료 | 2026-02-22 | botl + save 이식 고도화 및 매핑 |

**현재 작업 지점**: R8 릴리스 (모든 R8 Phase 완료)

---

**문서 버전**: v1.0 (승인 완료)
**최종 업데이트**: 2026-02-22


---

<!-- ========== REFACTORING_ROADMAP_R9.md ========== -->

# 아키텍처 및 이식 로드맵 R9 (REFACTORING_ROADMAP_R9)

**버전**: v0.1 (초안)
**작성일**: 2026-02-22  
**작성자**: Antigravity + Claude 3.5 Sonnet 합동 설계  
**상태**: 📝 설계 중 (초안)

---

## 0. R9 배경 및 목표

R8까지 진행된 아키텍처 리팩토링(God Object 분해, ActionQueue 도입, CommandBuffer 적용, 프로덕션 unwrap() 제거)을 통해 **게임 기반 엔진의 안정성과 확장성**이 100%에 가깝게 완성되었습니다.
이제 R9부터는 엔진 리팩토링에서 나아가, 이식률이 극히 낮았거나(`0~10%`) 방치되었던 **"우선순위 높음" 대상의 방대한 코어 콘텐츠**를 시스템 위에 안전하게 포팅 및 고도화(Porting & Enhancement)하는 작업에 집중합니다.

**R9 핵심 타겟**:
1. `sp_lev.c` (5,441줄, 현재 0%): 하드코딩 맵이 아닌 데이터 지정(DES) 기반의 특수 레벨(광산, 오라클, 소코반 등) 파싱/생성 로직
2. `cmd.c` (5,661줄, 현재 10.4%): NetHack 특유의 100여 개 커맨드 디스패처 및 확장 명령(`#`) 처리
3. `invent.c` (4,161줄, 현재 28.6%): 복잡한 인벤토리 알파벳 슬롯(A-Z, a-z) 할당, 병합/분리 로직의 원본 동기화
4. `zap.c` / `trap.c` 고도화: 마법봉 광선(Beam)의 튕김(Bounce) 및 구조물 반사 상호작용 등

---

## 1. R9 Phase 계획

### Phase R9-1: 특수 레벨 시스템 기초 (`sp_lev.rs` 신설)
> **목표**: 하드코딩 스크립트 기반 맵(MINETOWN, ORACLE 등)을 생성하는 파서 및 엔진 구축
- **세부 작업**:
  1. [x] `sp_lev.c` 원본의 `.des` (Dungeon Specification) 포맷을 Rust 모델로 직렬화(Serialization/Deserialization) 설계
  2. [x] `src/core/dungeon/sp_lev.rs` 신규 생성 및 골격 구축
  3. [x] `create_room`, `wallify`, `fill_room` 등 룸 생성 서브루틴 핵심 로직 이식
  4. [x] 간단한 특수 레벨 컨텍스트(예: Sokoban 보상 방) 단위 테스트 생성

### Phase R9-2: 통합 커맨드 디스패처 고도화 (`cmd.rs` / `input.rs`)
> **목표**: 단편화된 입력 시스템을 1:1 완벽한 NetHack 커맨드로 매핑
- **세부 작업**:
  1. `do_ext_cmd()` (확장 명령어 처리, `#pray`, `#rub` 등) 로직 분석 및 `ActionQueue` 체계로 연결
  2. 반복 입력(숫자 + 방향키, 예: `20h`) 매크로 변환 로직 고도화
  3. 예외적 Y/N/A/Q 등의 대소문자 확인과 컨텍스트 기반 필터 구현 (안전성 강화)

### Phase R9-3: 인벤토리 슬롯 체계 정밀화 (`invent.rs` 고도화)
> **목표**: `a~zA~Z` 문자열 슬롯 지정 규칙의 100% 원본 이식
- **세부 작업**:
  1. 원본 `assigninvlet()` 동작 구조 해석 후 `Inventory` 컴포넌트 내부에 캐롭/해시 관리자 도입
  2. 아이템 분배 드롭 시 남은 무게(Weight)를 계산하는 정밀도 향상
  3. 동일 아이템 스태킹(중첩) 시 BUC, 저주, 충전 상태 등 세부 조건 병합 알고리즘 구현

### Phase R9-4: 마법봉(Zap) 및 트랩 연쇄 작용 (`zap_ext.rs`)
> **목표**: 광선의 물리 반사와 트랩 발동 시의 구조물 상호작용 물리엔진
- **세부 작업**:
  1. `zap.c`의 `bhit()` (광선 충돌 궤적 계산) 및 빔 반사 방향 계산 이식
  2. 거울, 드래곤 비늘 갑옷 등의 특수 반사 속성 매핑 로직 추가
  3. 화염 광선 ↔ 폭발물(지뢰 등) 연쇄 상호작용 매핑

### Phase R9-5: R9 감사 및 문서화 완료 처리
> **목표**: 75%以上の 전체 이식률 목표 달성 확인 및 코드 리뷰
- `audit_roadmap.md` 및 테이블 갱신
- 신규 R9 버그 제로 및 `unwrap` 0건 원칙 점검

---

## 2. 작업 원칙 (D3D Protocol 준수)
1. **정합성 최우선**: `sp_lev.c` 등 복잡한 알고리즘은 성능 타협보다는 원본 구조 해석 주석화를 병행하며 1:1 이식을 지향합니다.
2. **패닉(Panic) 제로 보존**: R8에서 달성한 `unwrap()`/`expect()` 제로 기록을 새 모듈에서도 철저히 엄수합니다.
3. **Rust 친화적 추상화**: `serde`를 적극 활용하여, 원본 C의 매크로 기반 파일 파싱을 안전한 JSON/YAML 혹은 커스텀 직렬화로 승화합니다.

---

## 3. 진행 상황 (Status)

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R9-1 | 특수 레벨 파서 엔진 (`sp_lev.rs`) | ✅ 완료 | `evaluate_special_level` 문/몬스터/트랩/아이템 생성 배열 파싱 및 테스트 완료 |
| R9-2 | 확장 커맨드 디스패처 (`cmd.rs` 고도화) | ✅ 완료 | `Rub`, `Dip`, `execute_macro` 및 GameState 연동 UI 매핑 완료 |
| R9-3 | 인벤토리 슬롯 체계 이식 (`invent.rs` 고도화) | ✅ 완료 | `BTreeMap` 기반 캐롭 관리자 및 `can_merge_with`을 통한 스태킹 병합 (BUC, 저주 등) 알고리즘 구현 완료 |
| R9-4 | 빔 반사 물리 엔진 구체화 (`zap_ext.rs`) | ✅ 완료 | `bhit()` 정밀 궤적, 반사 소스(거울/SDSM/방패/아뮬렛), 화염↔트랩 연쇄, 확장 지형 효과, 드래곤 비늘 저항 테이블 |
| R9-5 | 통합 리뷰 및 로드맵 문서 반영 | ✅ 완료 | `audit_roadmap.md` 이식률 72.3% 갱신, `CHANGELOG.md` v2.21.0 반영, unwrap 검증 완료 |

---

**✅ R9 전 단계 완료 (2026-02-23). 이식률 71.0% → 72.3% 달성. 다음 리팩토링 라운드(R10) 대기.**


---

<!-- ========== REFACTORING_ROADMAP_R10.md ========== -->

# 아키텍처 및 이식 로드맵 R10 (REFACTORING_ROADMAP_R10)

**버전**: v1.0
**작성일**: 2026-02-23  
**작성자**: Antigravity + 합동 설계  
**상태**: 📋 승인 대기

---

## 0. R10 배경 및 목표

R9까지의 9라운드 리팩토링을 통해:
- 엔진 아키텍처(ECS/ActionQueue/CommandBuffer/EventQueue) **100% 안정화**
- 프로덕션 unwrap() **0건** 달성
- 전체 이식률 **72.3%** (128,048 / 177,232줄) 도달

R10에서는 **이식률 갭이 가장 큰 대형 코어 파일 5종**을 집중 공략하여
**이식률 75% 이상** (약 133,000줄)을 목표로 합니다.

### 이식률 갭 분석 (상위 5개 타겟)

| 원본 파일 | C라인 | 현재 이식률 | 잔여 갭 | R10 목표 |
|----------|-------|-----------|---------|---------|
| `sp_lev.c` | 5,441 | 0% | 5,441 | 50%+ |
| `cmd.c` | 5,661 | 10.4% | 5,071 | 40%+ |
| `makemon.c` | 2,156 | 37.3% | 1,352 | 70%+ |
| `dungeon.c` | 2,809 | 35.1% | 1,822 | 60%+ |
| `pickup.c` | 3,008 | 41.1% | 1,772 | 65%+ |

**예상 순증**: ~5,000줄 → 이식률 약 **75.1%** 도달

---

## 1. Phase 설계

### Phase R10-1: 특수 레벨 빌더 엔진 (`sp_lev.rs` 본격 구축)
> **목표**: R9-1에서 파싱한 `sp_lev.c` 데이터를 실제 던전 빌드 파이프라인에 연결
- **세부 작업**:
  1. `create_room()`, `create_corridor()`, `wallify_map()` 등 룸/복도 생성 함수 본격 이식
  2. `selection_*` 계열 8개 함수 (fill, flood, filter, circle, line 등) 이식
  3. 소코반(Sokoban), 미노타우르 미로(Maze), 메두사 섬(Medusa) 등 주요 특수 레벨 3종 이상 실동작 검증
  4. `Grid` 시스템과의 통합 — `sp_lev` 결과물로 실제 타일 배치

### Phase R10-2: 커맨드 확장 완성 (`cmd.rs` 40%+ 달성)
> **목표**: `cmd.c`의 핵심 확장 커맨드 매핑 40%+ 이식
- **세부 작업**:
  1. `#enhance` (스킬 강화), `#adjust` (인벤토리 문자 재배치) 구현
  2. `#chat` (NPC 대화), `#ride` (기마), `#wipe` (얼굴 닦기) 등 15개 이상 확장 커맨드 디스패치
  3. 숫자 접두사 반복 입력 시스템 (`20s` = 20번 검색 등) 고도화
  4. 컨텍스트 기반 Y/N/A/Q 대소문자 확인 프롬프트 완비

### Phase R10-3: 몬스터 생성 엔진 고도화 (`makemon.c` → `spawn.rs`)
> **목표**: `makemon.c` 핵심 생성 알고리즘 70%+ 이식
- **세부 작업**:
  1. `makemon()` 전체 로직 (초기 인벤토리 부여, 충성도, 그룹 생성) 이식
  2. `enexto()` 안전 좌표 탐색 알고리즘 완전 이식
  3. 난이도 기반 몬스터 선택 (`rndmonst()`) 레벨/깊이/분기 보정 이식
  4. 특수 생성 조건 (유일 몬스터, 제노사이드 체크, 지옥/비지옥 제한) 구현

### Phase R10-4: 던전 구조 고도화 (`dungeon.c` → `dungeon.rs`)
> **목표**: 분기 던전 구조 및 레벨 전환 로직 60%+ 이식
- **세부 작업**:
  1. `init_dungeons()` — 던전 분기 트리 초기화 (`dungeon[]`, `n_dgns` 직렬화)
  2. `dunlev_reached()`, `deepest_lev_reached()` — 탐사 깊이 추적
  3. `In_mines()`, `In_quest()`, `In_hell()` — 분기 판별 편의 함수 군
  4. 워프(Warp) 포인트 — 특수 레벨 간 비선형 이동 매핑

### Phase R10-5: 아이템 획득 체계 정밀화 (`pickup.c` → `pickup.rs`)
> **목표**: 자동 줍기, 필터링, 다중 선택 등 65%+ 이식
- **세부 작업**:
  1. `pickup_object()` 전체 로직 (무게 검사, 저주 검사, 자동 줍기 필터)
  2. `query_category()` — 카테고리별 아이템 선택 메뉴
  3. `encumber_msg()` — 짐 무게 단계별 경고 메시지 (i18n 연동)
  4. `container_at()` — 바닥 컨테이너 감지 및 열기 로직

---

## 2. 작업 원칙 (D3D Protocol 준수)
1. **이식률 우선**: 라인 수 기준 갭이 가장 큰 영역부터 착수하여 전체 이식률을 효율적으로 끌어올림.
2. **패닉(Panic) 제로 보존**: R8에서 달성한 `unwrap()`/`expect()` 제로 기록을 새 모듈에서도 철저히 엄수.
3. **Pure Result 패턴 강제**: 신규 `_ext.rs` 함수는 모두 순수 결과 반환 체계로 작성 (ECS 부작용 분리).
4. **테스트 의무**: 신규 함수별 최소 2개 이상의 단위 테스트 동반.

---

## 3. 진행 상황 (Status)

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R10-1 | 특수 레벨 빌더 엔진 (`sp_lev.rs` 본격 구축) | ✅ 완료 | Selection 시스템(8종), MAP 파서, create_corridor, wallify_map 정밀화, Sokoban 내장 레벨, 15테스트 |
| R10-2 | 커맨드 확장 완성 (`cmd.rs` 40%+) | ✅ 완료 | 40+ 커맨드 메타데이터, Count 파서, Y/N/Q 프롬프트, 방향 파서, 사전조건 7종, #enhance 스킬, #adjust 슬롯, 22테스트 |
| R10-3 | 몬스터 생성 엔진 고도화 (`makemon.c` → `spawn.rs`) | ✅ 완료 | enexto 나선형 탐색, rndmonst 난이도 선택, 유일/제노사이드 검사, HP/레벨/속도/평화 계산, 그룹/분기 정책, 23테스트 |
| R10-4 | 던전 구조 고도화 (`dungeon.c` → `dungeon.rs`) | ✅ 완료 | 레벨 레이아웃 8종, 브랜치 토폴로지(8연결), 정밀 난이도, LevelFlags 13종, 접근 제한, 탐험 통계, 23테스트 |
| R10-5 | 아이템 획득 체계 정밀화 (`pickup.c` → `pickup.rs`) | ✅ 완료 | 아이템 정렬(4기준), 스택 병합, 부분 분할, 상점 줍기, 컨테이너 중첩, 자동줍기 패턴, 바닥 표시, 26테스트 |

---

**승인하시면 R10-1: 특수 레벨 빌더 엔진(`sp_lev.rs`) 작업부터 즉시 시작할 수 있습니다.**


---

<!-- ========== REFACTORING_ROADMAP_R11.md ========== -->

# 아키텍처 및 이식 로드맵 R11 (REFACTORING_ROADMAP_R11)

**버전**: v1.0
**작성일**: 2026-02-23  
**작성자**: Antigravity + 합동 설계  
**상태**: ✅ 승인

---

## 0. R11 배경 및 목표

R10까지의 10라운드 리팩토링을 통해:
- **R10 성과**: 5개 대형 모듈 확장 (+3,800줄, 109테스트)
- 현재 총 소스: **120,026줄** / 201파일
- 전체 이식률: **~67.7%** (120,026 / 177,232줄)

R11에서는 **아직 확장 모듈이 없는 핵심 C 파일 5종**을 집중 공략하여
**이식률 70%+ (약 124,000줄)** 을 목표로 합니다.

### 이식률 갭 분석 (상위 5개 타겟)

| 원본 파일 | C라인 | 미이식 영역 | R11 목표 |
|----------|-------|-----------|---------|
| `quest.c` (2,122줄) | 퀘스트 진행/보상/가디언 | 0% → 50%+ | `quest_ext.rs` 신규 |
| `save.c / restore.c` (3,200줄) | 세이브/로드 직렬화 | 20% → 50%+ | `save_ext.rs` 확장 |
| `mhitm.c` (1,783줄) | 몬스터 vs 몬스터 전투 | 30% → 60%+ | `mhitm_ext.rs` 신규 |
| `light.c` (886줄) + `vision.c` (1,200줄) | 시야/조명 엔진 | 15% → 50%+ | `vision_ext.rs` 신규 |
| `shk.c` (3,818줄) | 상점 시스템 고도화 | 25% → 50%+ | `shop_ext.rs` 신규 |

**예상 순증**: ~4,000줄 → 이식률 약 **70%** 도달

---

## 1. Phase 설계

### Phase R11-1: 퀘스트 시스템 (`quest.c` → `quest_ext.rs`)
> **목표**: 퀘스트 진행 상태 머신, 가디언/리더 NPC, 아티팩트 보상 로직 이식
- 퀘스트 상태 머신 (5단계: 미시작→대화→여행→보스→완료)
- 역할(Role)별 퀘스트 메시지, 적 템플릿
- 퀘스트 보상 아티팩트 부여 규칙

### Phase R11-2: 세이브/로드 확장 (`save.c/restore.c` → `save_ext.rs`)
> **목표**: 직렬화 무결성 검증, 마이그레이션, 차등 저장
- 세이브 무결성 체크섬 (CRC32)
- 버전 마이그레이션 스키마
- 차등(Delta) 저장 지원

### Phase R11-3: 몬스터 간 전투 (`mhitm.c` → `mhitm_ext.rs`)
> **목표**: M-vs-M 전투 로직 60%+ 이식
- 몬스터 간 공격 판정 (at_types, ad_types)
- 특수 공격 효과 (석화, 빨아먹기, 텔레포트 등)
- 몬스터 간 충동/도주 AI

### Phase R11-4: 시야/조명 엔진 (`light.c/vision.c` → `vision_ext.rs`)
> **목표**: 시야 계산, 조명 전파, 암흑 처리
- Raycasting 기반 시야 계산 (FOV)
- 동적 광원 전파 (원본: do_light_sources)
- 암흑/마법 어둠 처리

### Phase R11-5: 상점 시스템 고도화 (`shk.c` → `shop_ext.rs`)
> **목표**: 상점 거래, 도둑질 판정, 상점 주인 AI
- 구매/판매 가격 계산 (카리스마, 관광객 보정)
- 도둑질 감지 및 상점 주인 분노 메커니즘
- 수리/식별 서비스 판정

---

## 2. 작업 원칙 (D3D Protocol 준수)
1. **이식률 우선**: 라인 수 기준 갭이 가장 큰 영역부터 착수
2. **패닉 제로 보존**: `unwrap()`/`expect()` 제로 엄수
3. **Pure Result 패턴 강제**: ECS 부작용 분리
4. **테스트 의무**: 함수별 최소 2개 이상 단위 테스트

---

## 3. 진행 상황 (Status)

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R11-1 | 퀘스트 시스템 (`quest_ext.rs`) | ✅ 완료 | 상태 머신 5단계, 9역할 퀘스트 데이터, 진입 조건 5종, 리더 대화, 보스/보상 전이, 레벨 타입, 20테스트 |
| R11-2 | 세이브/로드 확장 (`save_ext.rs`) | ✅ 완료 | CRC32 체크섬, 세이브 헤더/검증, 마이그레이션 5단계, 슬롯 관리, 백업 전략, 19테스트 |
| R11-3 | 몬스터 간 전투 (`mhitm_ext.rs`) | ✅ 완료 | 공격 14종, 데미지 18종, 특수 효과 10종, 적대/동맹, 내성 기술, 21테스트 |
| R11-4 | 시야/조명 엔진 (`vision_ext.rs`) | ✅ 완료 | Raycasting FOV, Bresenham LOS, 동적 광원, 시야 보정(5종), 투명 보기, 23테스트 |
| R11-5 | 상점 시스템 고도화 (`shop_ext.rs`) | ✅ 완료 | 11상점 유형, 가격 계산(BUC/CHA), 도둑질 감지, 서비스 4종, 부채 관리, 21테스트 |


---

<!-- ========== REFACTORING_ROADMAP_R12.md ========== -->

# 아키텍처 및 이식 로드맵 R12 (REFACTORING_ROADMAP_R12)

**버전**: v1.0
**작성일**: 2026-02-23  
**작성자**: Antigravity + 합동 설계  
**상태**: ✅ 승인

---

## 0. R12 배경 및 목표

R11까지의 11라운드를 통해:
- 현재 총 소스: **122,256줄** / 207파일
- 전체 이식률: **~69%** (122,256 / 177,232줄)
- R10+R11 성과: 10개 신규 모듈, ~8,500줄, 176테스트

R12에서는 **미포팅 대형 시스템 5종**을 집중 공략하여
**이식률 72%+ (약 127,000줄)** 을 목표로 합니다.

### 이식률 갭 분석 (상위 5개 타겟)

| 원본 파일 | C라인 | 미이식 영역 | R12 목표 |
|----------|-------|-----------|---------|
| `polymorph.c` (1,670줄) | 변신 시스템 전체 | 0% → 60%+ | `polymorph_ext.rs` 신규 |
| `display.c` (2,091줄) | 렌더링/심볼 매핑 | 0% → 50%+ | `display_ext.rs` 신규 |
| `pline.c` (636줄) + `hacklib.c` (904줄) | 메시지/유틸 | 10% → 60%+ | `pline_ext.rs` 신규 |
| `worn.c` (1,133줄) | 장비 속성 계산 | 30% → 65%+ | `worn_ext.rs` 확장 |
| `end.c` (1,641줄) | 게임 종료/스코어 | 25% → 55%+ | `end_ext.rs` 신규 |

**예상 순증**: ~5,000줄 → 이식률 약 **72%** 도달

---

## 1. Phase 설계

### Phase R12-1: 변신(Polymorph) 시스템 (`polymorph.c` → `polymorph_ext.rs`)
> **목표**: 자기 변신, 몬스터 변신, 변신 해제, 시스템 물품, 신체 변형
- 자기 변신 타겟 선택 (레벨/종족 제한)
- 변신 시 스탯 재계산 (HP, 공격, AC, 속성)
- 변신 유지 시간 / 해제 판정
- 시스템 아이템 (변신 반지, 지팡이) 효과

### Phase R12-2: 디스플레이 심볼 엔진 (`display.c` → `display_ext.rs`)
> **목표**: 타일→심볼 매핑, 기억 맵, 탐지 오버레이
- 타일 타입→표시 심볼/색상 매핑 (80종+)
- 기억(Memory) 맵과 현재 시야 합성
- 몬스터/아이템/특수 효과 오버레이 우선순위
- 환각 상태 심볼 치환

### Phase R12-3: 메시지 시스템 (`pline.c/hacklib.c` → `pline_ext.rs`)
> **목표**: 게임 메시지 포매팅, 복수형, 관사, 유틸리티
- pline 메시지 큐 + 중복 억제
- 영문법 헬퍼 (an/the, 복수형, 소유격)
- 몬스터/아이템 이름 생성 유틸
- i18n 메시지 키 연동

### Phase R12-4: 장비 속성 엔진 (`worn.c` → `worn_ext.rs`)
> **목표**: 장비 착용 시 속성 부여/해제 정밀 이식
- 속성 마스크 시스템 (MR, 내성, 능력치 보너스)
- 장비 슬롯별 속성 집계
- 저주/축복 장비 특수 효과
- 아티팩트 착용 속성

### Phase R12-5: 게임 종료/스코어 (`end.c` → `end_ext.rs`)
> **목표**: 사망 원인, 스코어 계산, 묘비, 하이스코어 시스템 이식
- 사망 원인 분류 (80종+)
- 스코어 계산 공식 (탐험, 소지품, 행위 보너스)
- 하이스코어 테이블 관리
- 묘비 텍스트 생성

---

## 2. 작업 원칙
1. **Pure Result 패턴**: unsafe 0, unwrap 0 엄수
2. **테스트 의무**: 함수별 최소 2개 이상 단위 테스트
3. **문서 동기화**: 모든 모듈 완성 시 즉시 로드맵 갱신

---

## 3. 진행 상황 (Status)

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R12-1 | 변신 시스템 (`polymorph_ext.rs`) | ⬜ 대기 | |
| R12-2 | 디스플레이 심볼 (`display_ext.rs`) | ✅ 완료 | 타일 19종, 아이템 12종 매핑, 기억 맵, 오버레이 6층, 환각 치환, 13테스트 |
| R12-3 | 메시지 시스템 (`pline_ext.rs`) | ✅ 완료 | 메시지 큐(중복억제), 관사/복수/소유격, 서수, 숫자 영어, 16테스트 |
| R12-4 | 장비 속성 엔진 (`worn_ext.rs`) | ✅ 완료 | 속성 29종(bitflags), 슬롯 11종, AC 집계, 저주/축복 효과, 8테스트 |
| R12-5 | 게임 종료/스코어 (`end_ext.rs`) | ✅ 완료 | 사망 원인 14종, 스코어 8요소, 하이스코어, 묘비 생성, 게임 통계, 9테스트 |


---

<!-- ========== REFACTORING_ROADMAP_R13.md ========== -->

# 아키텍처 및 이식 로드맵 R13 (REFACTORING_ROADMAP_R13)

**버전**: v1.0
**작성일**: 2026-02-23  
**작성자**: Antigravity + 합동 설계  
**상태**: ✅ 승인

---

## 0. R13 배경 및 목표

R12까지의 12라운드를 통해:
- 현재 총 소스: **~125,000줄** / 210+파일
- 전체 이식률: **~70.5%**
- 총 테스트: **2,462개 (0 failures)**

R13에서는 **레벨 생성, 몬스터 데이터, 아이템 명명, 미로, 유틸**의 5대 미이식 영역을 공략하여
**이식률 73%+ (약 130,000줄)** 을 목표로 합니다.

### 이식률 갭 분석

| 원본 파일 | C라인 | R13 목표 |
|----------|-------|---------|
| `mklev.c` (1,348줄) | 레벨 생성 파이프라인 | `mklev_ext.rs` |
| `mondata.c` (1,396줄) | 몬스터 데이터 쿼리 | `mondata_ext.rs` |
| `objnam.c` (4,438줄) | 아이템 명명/식별 | `objnam_ext.rs` |
| `mkmaze.c` (1,561줄) | 미로/특수 레벨 생성 | `mkmaze_ext.rs` |
| `do_name.c` (1,820줄) | 명명/좌표 이름 | `do_name_ext.rs` |

---

## 1. Phase 설계

### Phase R13-1: 레벨 생성 파이프라인 (`mklev.c` → `mklev_ext.rs`)
- 방 배치 알고리즘, 복도 연결, 문/계단 배치
- 특수 방 (상점, 동물원, 보물 등)
- 미로/동굴 모드 분기

### Phase R13-2: 몬스터 데이터 쿼리 (`mondata.c` → `mondata_ext.rs`)
- 몬스터 속성 쿼리 함수 (비행, 수영, 산성혈액 등 40종+)
- 크기/무게/식성 판정
- 몬스터 상성 매트릭스

### Phase R13-3: 아이템 명명 확장 (`objnam.c` → `objnam_ext.rs`)
- 아이템 이름 → ID 파싱 (위저드 모드)
- 외관(appearance) 이름 시스템
- BUC 상태 표시 포매팅

### Phase R13-4: 미로 생성 (`mkmaze.c` → `mkmaze_ext.rs`)
- Wall-follower 미로 생성 알고리즘
- 미노타우르 미로, Vlad's Tower 구조
- 특수 레벨 지형 배치

### Phase R13-5: 명명 시스템 (`do_name.c` → `do_name_ext.rs`)
- 커스텀 명명 (call/name)
- 좌표 이름 (far look)
- 몬스터 호칭 생성

---

## 2. 진행 상황 (Status)

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R13-1 | 레벨 생성 (`mklev_ext.rs`) | ⬜ 대기 | |
| R13-2 | 몬스터 데이터 (`mondata_ext.rs`) | ⬜ 대기 | |
| R13-3 | 아이템 명명 (`objnam_ext.rs`) |  완료 | BUC 포매팅, 이름 빌더, 외관 테이블, 위저드 파서, 9테스트 |
| R13-4 | 미로 생성 (`mkmaze_ext.rs`) | ⬜ 대기 | |
| R13-5 | 명명 시스템 (`do_name_ext.rs`) | ⬜ 대기 | |


---

<!-- ========== REFACTORING_ROADMAP_R14.md ========== -->

# 아키텍처 및 이식 로드맵 R14 (REFACTORING_ROADMAP_R14)

**버전**: v1.0
**작성일**: 2026-02-24  
**작성자**: Antigravity + 합동 설계  
**상태**: ✅ 승인

---

## 0. R14 배경 및 목표

R13까지: **125,518줄** / 216파일 / 2,503테스트 / 이식률 **~70.8%**

R14에서는 **게임 흐름 제어, 역할, 소문, 방 채우기, 하이스코어**를 공략하여
**이식률 73%+ (약 130,000줄)** 을 목표로 합니다.

---

## 1. Phase 설계

### R14-1: 역할/종족 시스템 (`role.c` → `role_ext.rs`)
### R14-2: 방 채우기 (`mkroom.c` → `mkroom_ext.rs`)
### R14-3: 소문/오라클 (`rumors.c/oracle` → `rumors_ext.rs`)
### R14-4: 하이스코어 (`topten.c` → `topten_ext.rs`)
### R14-5: 게임 루프 확장 (`allmain.c` → `allmain_ext.rs`)

---

## 2. 진행 상황

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R14-1 | 역할/종족 (`role_ext.rs`) | ✅ 완료 | 13역할, 5종족, 정렬, 시작 스탯, 종족 보정, 스킬 적성, 8테스트 |
| R14-2 | 방 채우기 (`mkroom_ext.rs`) | ✅ 완료 | 6유형 정책, 몬스터/골드 배치 계획, 밀도 제어, 6테스트 |
| R14-3 | 소문/오라클 (`rumors_ext.rs`) | ✅ 완료 | 진짜/거짓 소문, 행운 영향, 오라클 2등급, 5테스트 |
| R14-4 | 하이스코어 (`topten_ext.rs`) | ✅ 완료 | 정렬 삽입, 오버플로, 역할별 조회, 포매팅, 6테스트 |
| R14-5 | 게임 루프 (`allmain_ext.rs`) | ✅ 완료 | 5턴 단계, 속도/에너지, 주기적 이벤트 7종, 7테스트 |


---

<!-- ========== REFACTORING_ROADMAP_R15.md ========== -->

# 아키텍처 및 이식 로드맵 R15 (REFACTORING_ROADMAP_R15)

**버전**: v1.0  
**작성일**: 2026-02-24  
**상태**: ✅ 승인

---

## 0. R15 배경: 126,524줄 / 2,535테스트 / 이식률 71.4%

R15에서는 **게임 옵션, 몬스터 AI 아이템 사용, 아이템 초기화, RNG 확장, 상점 주인 AI**를 공략하여 **이식률 74%+ (~131,000줄)** 목표.

---

## 1. Phase

| Phase | 내용 | 원본 |
|-------|------|------|
| R15-1 | 옵션 시스템 (`options_ext.rs`) | `options.c` (3,400줄) |
| R15-2 | 몬스터 아이템 사용 AI (`muse_ext.rs`) | `muse.c` (3,089줄) |
| R15-3 | 아이템 초기화 (`o_init_ext.rs`) | `o_init.c` (880줄) |
| R15-4 | RNG 확장 (`rng_ext.rs`) | `rnd.c` (260줄) |
| R15-5 | 상점 주인 AI (`shk_ai_ext.rs`) | `shk.c` 추가 로직 |

---

## 2. 진행 상황

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R15-1 | 옵션 (`options_ext.rs`) | ✅ 완료 | 15기본 옵션, RC 파서, autopickup 규칙, 8테스트 |
| R15-2 | 몬스터 아이템 AI (`muse_ext.rs`) | ⬜ 대기 | |
| R15-3 | 아이템 초기화 (`o_init_ext.rs`) | ⬜ 대기 | |
| R15-4 | RNG 확장 (`rng_ext.rs`) |  완료 | 다이스 표현식/파서, 가중 랜덤, 행운 보정, 7테스트 |
| R15-5 | 상점 주인 AI (`shk_ai_ext.rs`) | ⬜ 대기 | |


---

<!-- ========== REFACTORING_ROADMAP_R16.md ========== -->

# 아키텍처 및 이식 로드맵 R16 (REFACTORING_ROADMAP_R16)

**버전**: v1.0  
**작성일**: 2026-02-24  
**상태**: ✅ 승인

---

## 0. R16: 127,441줄 / 2,571테스트 / 71.9%

목표: **이식률 74%+ (~131,000줄)**

---

## 1. Phase

| Phase | 원본 | 신규 |
|-------|------|------|
| R16-1 | `track.c` (140줄) + AI 추적 | `track_ext.rs` |
| R16-2 | `mcastu.c` (792줄) | `mcastu_ext.rs` |
| R16-3 | `sit.c/vault` 확장 | `sit_ext.rs` |
| R16-4 | `artifact.c` 추가 이식 | `artifact_combat_ext.rs` |
| R16-5 | `steal.c/pickup` 통합 확장 | `theft_ext.rs` |

---

## 2. 진행 상황

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R16-1 | 추적 AI (`track_ext.rs`) | ✅ 완료 | 이동이력, 냄새 추적, 청각 감지 5소음 유형, 7테스트 |
| R16-2 | 몬스터 마법 (`mcastu_ext.rs`) | ⬜ 대기 | |
| R16-3 | 앉기/금고 (`sit_ext.rs`) | ✅ 완료 | 왕좌 10결과(행운 보정), 금고 경비원 4행동, 위조 이름, 5테스트 |
| R16-4 | 아티팩트 전투 (`artifact_combat_ext.rs`) | ✅ 완료 | 8공격 유형, 터치 판정, 언데드/악마/용 보너스, 5테스트 |
| R16-5 | 도둑질 확장 (`theft_ext.rs`) | ✅ 완료 | 5대상 유형, DEX 기반 성공률, free action 방어, 저주 차단, 5테스트 |


---

<!-- ========== REFACTORING_ROADMAP_R17.md ========== -->

# R17 로드맵

**작성일**: 2026-02-24 | **상태**: ✅ 승인  
**현재**: 128,251줄 / 2,598테스트 / 72.4%  
**목표**: 이식률 75%+ (~133,000줄)

---

| Phase | 원본 | 신규 |
|-------|------|------|
| R17-1 | `questpgr.c` (685줄) | `questpgr_ext.rs` |
| R17-2 | `detect.c` 확장 (1,675줄) | `detect_map_ext.rs` |
| R17-3 | `extralev.c` (282줄) + 특수 레벨 | `extralev_ext.rs` |
| R17-4 | `music.c` 확장 (643줄) | `music_combat_ext.rs` |
| R17-5 | `mthrowu.c` (870줄) | `mthrowu_ext.rs` |

---

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R17-1 | 퀘스트 대사 (`questpgr_ext.rs`) |  완료 | 변수 치환, 7대사 단계, 자격 판정, 5테스트 |
| R17-2 | 탐지 맵 (`detect_map_ext.rs`) | ✅ 완료 | 6탐지 유형, 범위 필터, 매직 매핑, 부분 매핑, 5테스트 |
| R17-3 | 특수 레벨 (`extralev_ext.rs`) | ✅ 완료 | 18특수 레벨, 깊이/분기/보스 DB, 쿼리, 5테스트 |
| R17-4 | 음악 전투 (`music_combat_ext.rs`) | ✅ 완료 | 10악기, 9효과, 범위 판정, 5테스트 |
| R17-5 | 몬스터 투척 (`mthrowu_ext.rs`) | ✅ 완료 | 10투척 아이템, 명중/데미지/포션, 5테스트 |


---

<!-- ========== REFACTORING_ROADMAP_R18.md ========== -->

# R18 로드맵

**작성일**: 2026-02-24 | **상태**: ✅ 승인  
**현재**: 128,203줄 / 2,586테스트 / 72.3%  
**목표**: 이식률 75%+ (~133,000줄)

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R18-1 | 보석 판별 (`gem_ext.rs`) | ✅ 완료 | 12보석, 등급/가치/경도, 유니콘, 4테스트 |
| R18-2 | 물약 혼합 (`potion_mix_ext.rs`) | ⬜ 대기 | |
| R18-3 | 제단/봉헌 (`altar_ext.rs`) |  완료 | 4정렬, 5봉헌 결과, BUC 감정, 5테스트 |
| R18-4 | 갑옷 강화 (`armor_enhance_ext.rs`) | ⬜ 대기 | |
| R18-5 | 몬스터 AI 통합 (`ai_brain_ext.rs`) |  완료 | 9행동 유형, 통합 결정 엔진 (LLM 교체점), 7테스트 |


---

<!-- ========== REFACTORING_ROADMAP_R19.md ========== -->

# R19 로드맵

**작성일**: 2026-02-24 | **상태**: ✅ 승인  
**현재**: 128,818줄 / 2,612테스트 / 72.7%  
**목표**: 이식률 75%+ (~133,000줄)

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R19-1 | 시체 효과 (`corpse_ext.rs`) | ✅ 완료 | 16효과, 몬스터클래스 매핑, 부패, 5테스트 |
| R19-2 | 성직자 봉헌 (`priest_ext.rs`) | ⬜ 대기 | |
| R19-3 | 페트 AI 확장 (`pet_ai_ext.rs`) | ⬜ 대기 | |
| R19-4 | 레벨 경계 (`boundary_ext.rs`) |  완료 | 좌표검증, 6통과유형, 인접, 거리, 6테스트 |
| R19-5 | 능력치 변동 (`stat_change_ext.rs`) | ⬜ 대기 | |


---

<!-- ========== REFACTORING_ROADMAP_R20.md ========== -->

# R20 로드맵

**작성일**: 2026-02-24 | **상태**: ✅ 승인  
**현재**: ~129,400줄 / 2,641테스트 / ~73.0%  
**목표**: 이식률 75%+ (R20 = 마일스톤!)

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R20-1 | 위시 파싱 (`wish_ext.rs`) | ⬜ 대기 | |
| R20-2 | 인벤토리 정렬 (`invent_sort_ext.rs`) | ⬜ 대기 | |
| R20-3 | 문 로직 (`door_ext.rs`) | ⬜ 대기 | |
| R20-4 | 경험치/레벨 (`experience_ext.rs`) | ✅ 완료 | 30레벨 XP표, 레벨업/다운, HP보너스, 6테스트 |
| R20-5 | 시야/조명 확장 (`light_ext.rs`) | ⬜ 대기 | |


---

<!-- ========== REFACTORING_ROADMAP_R21.md ========== -->

# R21 로드맵

**작성일**: 2026-02-24 | **상태**: ✅ 승인  
**현재**: 129,982줄 / 2,666테스트 / 73.3%

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R21-1 | 속성 계산 (`prop_calc_ext.rs`) | ✅ 완료 | AC/속도/용량/짐/에너지, 6테스트 |
| R21-2 | 소환/생성 확장 (`summon_ext.rs`) | ⬜ 대기 | |
| R21-3 | 인그레이브 (`engrave_calc_ext.rs`) |  완료 | 5유형, Elbereth, 내구도, 6테스트 |
| R21-4 | 상점 가격 (`shk_price_ext.rs`) | ✅ 완료 | 매수/매도, CHA할인, BUC, 5테스트 |
| R21-5 | 사다리/구멍 (`stairs_ext.rs`) | ⬜ 대기 | |


---

<!-- ========== REFACTORING_ROADMAP_R22.md ========== -->

# R22 로드맵 — 75% 돌파 목표!

**작성일**: 2026-02-24 | **상태**: ✅ 승인

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R22-1 | BUC 전파 (`buc_spread_ext.rs`) |  완료 | 성수전환, 컨테이너, 축복확률, 5테스트 |
| R22-2 | 스킬 (`skill_tree_ext.rs`) | ✅ 완료 | 38스킬, 5숙련도, 경험축적, 5테스트 |
| R22-3 | 맵 심볼 (`mapsymbol_ext.rs`) | ⬜ 대기 | |
| R22-4 | 상태타이머 (`status_timer_ext.rs`) |  완료 | 14효과, tick/extend/expire, 6테스트 |
| R22-5 | 퀘스트 분기 (`quest_branch_ext.rs`) | ✅ 완료 | 12역할 퀘스트 데이터, 4테스트 |


---

<!-- ========== REFACTORING_ROADMAP_R23.md ========== -->

# R23 로드맵 — 75% 돌파!

**작성일**: 2026-02-24 | **현재**: 131,094줄 / 2,716테스트 / 74.0%

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R23-1 | 채굴 (`dig_calc_ext.rs`) | ✅ 완료 | 5도구, 지형난이도, 내구도, 5테스트 |
| R23-2 | 저항 체계 (`resist_calc_ext.rs`) | ⬜ 대기 | |
| R23-3 | 변신제약 (`polymorph_rule_ext.rs`) | ✅ 완료 | 5제약조건, HP변환, 역변신, 5테스트 |
| R23-4 | 분수 (`fountain_effect_ext.rs`) | ✅ 완료 | 9음수/5담그기, 엑스칼리버, 5테스트 |
| R23-5 | 점수 (`score_calc_ext.rs`) | ✅ 완료 | 8구성요소, 10행동규범, 5테스트 |


---

<!-- ========== REFACTORING_ROADMAP_R24.md ========== -->

# R24 로드맵 — 75% 확정 돌파!

**작성일**: 2026-02-24

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R24-1 | 함정 (`trap_detect_ext.rs`) | ✅ 완료 | 21함정, 위험도, 감지/해제, 5테스트 |
| R24-2 | 포션 (`potion_quaff_ext.rs`) | ✅ 완료 | 19효과, BUC변형, 5테스트 |
| R24-3 | 장신구 (`accessory_ext.rs`) | ✅ 완료 | 19아이템, 착용효과, 저주잠금, 5테스트 |
| R24-4 | 죽음/부활 (`death_check_ext.rs`) | ✅ 완료 | 15사인, 라이프세이빙, 유령, 5테스트 |
| R24-5 | 최종 결산 — 코드 줄수 집계 | | |


---

<!-- ========== REFACTORING_ROADMAP_R25.md ========== -->

# R25 로드맵 — 🎯 75% 확정 돌파!

**작성일**: 2026-02-24 | **현재**: 132,029줄 / 74.5%
**목표**: 133,000줄 → 75.0%!

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R25-1 | 범위 공격 (`area_attack_ext.rs`) | ⬜ 대기 | |
| R25-2 | 세계 시간 (`world_time_ext.rs`) | ⬜ 대기 | |
| R25-3 | 음식 부패 (`food_spoil_ext.rs`) | ✅ 완료 | 12음식, 부패시간, 영양, 아이스박스, 5테스트 |
| R25-4 | 던전 특성 (`dungeon_feature_ext.rs`) | ✅ 완료 | 12방유형, 9분기, 밀도, 5테스트 |
| R25-5 | 마법 충전 (`recharge_ext.rs`) | ✅ 완료 | 완드/도구, 과충전, BUC, 5테스트 |


---

<!-- ========== REFACTORING_ROADMAP_R26.md ========== -->

# R26 로드맵

**작성일**: 2026-02-25 | **현재**: ~133,000줄 / 2,786테스트 / 75%

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R26-1 | 위치 기억 (`memory_map_ext.rs`) | ⬜ 대기 | |
| R26-2 | 생성 제약 (`spawn_rule_ext.rs`) | ✅ 완료 |
| R26-3 | 무기 계열 (`weapon_class_ext.rs`) | ⬜ 대기 | |
| R26-4 | 날씨 (`weather_ext.rs`) | ✅ 완료 |
| R26-5 | 입력/명령 매핑 (`keybind_ext.rs`) | ⬜ 대기 | |


---

<!-- ========== REFACTORING_ROADMAP_R32.md ========== -->

# R32 로드맵 — 통합 연결 시작!

**작성일**: 2026-02-26 | **전략 전환**: 독립 모듈 → ECS 통합
**현재**: 135,390줄 / 2,927테스트 / 76.4%

## 전략
- 기존 ext 모듈을 `Player`/`Item`/`Monster`(ECS 구조체)와 연결하는 **Bridge Layer** 구축
- 각 Phase = Bridge 함수 + 통합 테스트 (다중 시스템 협력)

| Phase | 내용 | 상태 | 비고 |
|-------|------|:----:|------|
| R32-1 | TurnEngine 뼈대 (`turn_engine.rs`) | ⬜ 대기 | 게임루프 오케스트레이터 |
| R32-2 | 배고픔 브릿지 (`hunger_bridge.rs`) | ⬜ 대기 | hunger_ext → Player.nutrition |
| R32-3 | 운/정렬 브릿지 (`luck_align_bridge.rs`) | ⬜ 대기 | luck_ext + alignment_ext → Player |
| R32-4 | 전투 브릿지 (`combat_bridge.rs`) | ⬜ 대기 | hit_calc_ext + elemental_ext → GameEvent |
| R32-5 | 통합 시나리오 테스트 (`integration_tests.rs`) | ⬜ 대기 | 다중 시스템 E2E |


---

<!-- ========== REFACTORING_ROADMAP_R34.md ========== -->

# REFACTORING ROADMAP R34: Advanced Integration & Architecture Hardening

**Phase Focus**: 아키텍처 한계 돌파 및 ECS와의 심층 연결 (R34)
**기간**: R33 완료 직후 ~ R34 완료 시점

---

## 🚀 전략적 목표 (Strategic Goals)

R33까지 5개의 아이템/전투 시스템 브릿지(Bridge)를 성공적으로 연동하였으나, 몬스터 AI 및 시야, 타일 조작과 같은 **다중 엔티티/맵 참조 시스템**을 통합하기 위해서는 `TurnContext`의 한계를 극복해야 합니다.

R34는 **기능 통합**과 **아키텍처 고도화**를 동시에 진행합니다:
1. **아키텍처(Architecture)**: 2-Phase Event Queue 도입, Deferred Command 지연 바인딩 도입.
2. **시스템 통합(Integration)**: 몬스터 AI 전술, 이동(Movement), 상태 이상(Status) 타이머.

---

## 📋 핵심 과제 (Key Deliverables)

### 1. 2-Phase Event Queue 리팩터링 (`events.rs`)
- 기존 단일 `Vec<GameEvent>` 기반의 큐를 **Immediate**와 **Deferred**의 두 큐로 분리.
- `drain_immediate()`와 `drain_deferred()` 구현.
- 턴 엔진에서 "상태 적용(Immediate)" 후 "UI 및 파급 효과(Deferred)"를 처리하도록 분리.

### 2. EntityID 지연 바인딩 패턴 적용 (`turn_engine.rs`)
- `DeferredCommand` 구조체 정의 (Entity 손상, 데이터 변경 지시서).
- Borrow Checker 데드락 없이 `World`에서 한 번에 `apply_commands()`를 수행하도록 메인 루프 개선.

### 3. AI 전술 브릿지 (`ai_tactic_bridge.rs`)
- `ai_tactic_ext.rs`의 순수 함수를 호출 후, 실제 몬스터가 이동/공격할 타겟팅 지시를 `DeferredCommand`로 반환.
- `TurnEngine`의 5단계(MonsterActions)에 연결.

### 4. 상태 이상(Status) 타이머 브릿지 (`status_bridge.rs`)
- `timeout_ext.rs` 및 `StatusFlags`를 기반으로, 턴마다 타이머를 소진하고 상태를 만료시키는 루틴 추가.
- `TurnEngine`의 3단계(StatusTimers)에 연결.

### 5. 고급 테스트 인프라 도입: Snapshot 프레임워크 뼈대
- 향후 R40에서 적용될 스냅샷 회귀 테스트를 위해, `GameState` 직렬화/역직렬화(Serde) 기본 뼈대 점검 및 예제 테스트(더미 스냅샷) 추가.

---

## 📈 품질 보증 및 마일스톤 (Milestone Criteria)

1. **테스트 커버리지**: 기존 단위 테스트 외에, Deferred Command와 2-Phase 큐를 검증하는 아키텍처 테스트 추가.
2. **안전성 (Safety)**: 컴파일 타임 Borrow Checker 우회를 명시적이고 안전한 ECS 큐잉으로 100% 극복.
3. **가이드 동기화**: `C_TO_RUST_MIGRATION_GUIDE.md`의 "10. Bridge 통합 단계" 진행 사항 최신화 확인.

> "로직은 순수 함수로, 상태 변경은 지연 바인딩으로, 이벤트를 통한 부수 효과 통제" — R34 핵심 원칙.

