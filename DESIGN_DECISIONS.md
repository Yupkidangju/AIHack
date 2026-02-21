# 디자인 의사결정 (DESIGN_DECISIONS)

이 문서는 프로젝트 진행 중 선택한 주요 아키텍처 및 기술적 결정에 대한 근거를 기록합니다.
모든 신규 아키텍처 결정은 반드시 이 문서에 추가되어야 하며, 결정의 배경/근거/대안을 포함해야 합니다.

---

## [2026-02-15] - v2.3.4~v2.3.5: 대규모 시스템 확장 전략

### EXT-1. Statistics 구조체 패턴 도입
- **결정**: 모든 시스템 모듈에 `XxxStatistics` 구조체를 추가하여 게임 내 통계 추적을 표준화.
- **배경**: NetHack 원본의 `botl.c`에서 표시하는 다양한 통계를 체계적으로 수집하기 위해 각 시스템에 전용 통계 구조체가 필요.
- **근거**: (1) 게임 오버 화면에서 상세 통계 표시, (2) 차후 업적 시스템 연동, (3) 디버깅 시 시스템별 동작 추적 용이.
- **적용**: PotionStatistics, EquipmentStatistics, ShopStatistics, EatStatistics, DoWearStatistics, MhituStatistics, DigStatistics, KickExtendedStats 등 8종 추가.

### EXT-2. Enum 기반 세분화 (Material/Erosion/Allergy/Security 등)
- **결정**: 각 시스템의 하위 개념을 독립 enum으로 세분화하여 타입 안전성 확보.
- **배경**: 원본 C에서 정수/문자열로 관리하던 재료, 부식, 알레르기, 보안 등급 등을 Rust enum으로 전환.
- **근거**: (1) 컴파일 타임 검증, (2) match 식 exhaustive 체크로 누락 방지, (3) 문서화 효과.
- **적용**: ArmorMaterial(10종), ErosionType(5종), FoodAllergy(5종), ShopSecurity(5종), MixResult(9종), PreservationMethod(5종) 등.

### EXT-3. 테스트 모듈 네이밍 규칙
- **결정**: 확장 시 테스트 모듈명을 `xxx_extended_tests`, `xxx_advanced_tests` 등으로 분리하여 기존 테스트와 충돌 방지.
- **배경**: v2.3.4에서 `mcastu_extended_tests`, `botl_extended_tests` 등 중복 모듈명으로 컴파일 에러 발생.
- **근거**: Rust는 동일 모듈 내 같은 이름의 `mod`를 허용하지 않으므로, 확장 차수별로 고유한 접미사 사용 필수.

### EXT-4. 함수명 중복 방지 전략
- **결정**: 기존 함수와 유사한 기능의 확장 함수 추가 시 `_strength_`, `_advanced_` 등 구별 접미사 사용.
- **배경**: `wall_kick_self_damage`(기존 서명)과 충돌하여 `wall_kick_strength_damage`로 변경, `hp_color` → `hp_danger_color` 변경 등.
- **근거**: 원본 C에서 같은 이름으로 오버로딩되는 함수들이 Rust에서는 불가능하므로 명확한 이름 구분 필수.

---


## [2026-02-15] - 이식 아키텍처 원칙 명문화 (Transplant Architecture Rule)

### ARCH-1. C 구조 직역 금지 원칙
- **결정**: 원본 NetHack C 코드를 이식할 때, C의 구조(전역 변수, goto, 매크로, 포인터 연산)를 그대로 번역하는 것을 명시적으로 금지한다.
- **배경**: 이 프로젝트의 "100% 이식"은 **동작(Behavior)의 충실한 재현**이지, **코드 구조(Structure)의 1:1 번역**이 아니다. 리팩토링(R1~R6)을 거쳐 Rust 고유의 안전한 아키텍처가 확립되었으므로, 신규 이식 코드가 이를 역행해서는 안 된다.
- **근거**: (1) 전역 변수 회귀 시 Legion ECS 무의미화, (2) goto 패턴은 Rust에서 unsafe를 강제하여 안전성 저하, (3) 매크로 직역은 타입 안전을 파괴, (4) R2에서 enum 전환 완료된 코드를 다시 문자열 비교로 작성하면 일관성 상실.
- **적용 범위**: 모든 신규 코드, 모든 이식 코드, AI 에이전트가 생성하는 모든 코드에 적용.
- **구체적 변환 규칙**:
  - `전역 변수` → Legion Resources / Component
  - `struct you (God Object)` → Player 컴포넌트 + 뷰 타입
  - `#define 매크로` → `const` / `bitflags!` / `enum`
  - `포인터 링크드 리스트` → `Vec<Entity>`
  - `문자열 비교` → `ItemKind`/`MonsterKind` enum 직접 비교
  - `goto` → `match` / `loop` + `break 'label`
  - `함수 포인터 테이블` → `match` 구문 / Trait 디스패치
- **참조**: spec.md 섹션 0 (이식 아키텍처 원칙), designs.md 섹션 0 (전역 문서 규칙)

---

## [2026-02-15] - M2: 현대적 게임 레이아웃 완성

### M2-1. ECS 기반 장비 요약 연동
- **결정**: 우측 Stats Panel의 장비 요약을 하드코딩 default 대신 ECS Equipment 컴포넌트에서 실시간 조회.
- **근거**: ECS 아키텍처 원칙에 따라 모든 게임 데이터는 ECS를 통해 접근해야 하며, UI가 직접 데이터를 보유하면 안 된다.

### M2-2. 상태 아이콘 확장 (7→21종) + HungerState 별도 처리
- **결정**: StatusFlags 비트플래그와 Player.hunger(HungerState enum)를 분리하여 상태 아이콘을 표시.
- **근거**: 배고픔은 StatusFlags가 아닌 별도 enum으로 관리되므로, 두 시스템을 각각 조회하여 합산.

### M2-3. Settings 윈도우 구현
- **결정**: View > Settings 체크박스로 토글하는 egui::Window 기반 설정 패널.
- **근거**: options.toml 저장 연동으로 영속성 보장, 인게임에서 즉시 변경 가능.

---

## [2026-02-15] - M1: AppState::GameOver 사망 화면 시스템

### M1-1. AppState와 GameState의 이중 레이어 구조
- **결정**: 기존 `GameState::GameOver`(인게임 상태머신)를 유지하면서, `AppState::GameOver`(앱 전체 흐름)를 추가.
- **배경**: `GameState`는 인게임 턴 처리 중 사용되고, `AppState`는 타이틀/생성/플레이/사망의 상위 흐름을 제어.
- **근거**: Playing 상태에서 GameState::GameOver을 감지 → AppState::GameOver로 승격시키는 패턴으로 기존 게임 로직 무변경.
- **대안 고려**: GameState::GameOver를 제거하고 AppState만 사용 — game_loop.rs/death.rs 등 많은 사이트 변경 필요, 위험도 높아 기각.

### M1-2. 풀스크린 사망 화면 (game_over.rs)
- **결정**: game_ui.rs의 기존 오버레이 팝업 대신, AppState::GameOver 전환 시 전용 풀스크린 화면을 표시.
- **근거**: (1) 사망은 게임의 핵심 이벤트로 전용 화면이 적합, (2) 통계/점수를 충분히 보여줄 공간 확보, (3) 타이틀 화면과 일관된 UX.
- **내용**: ASCII 묘비 아트 + 사망 메시지 + 점수/턴/던전깊이 통계 + New Game/Quit 버튼.

---

## [2026-02-15] - R2 최적화: Enum 기반 타입 안전성 완성

### R2-5. 문자열 리터럴 비교 → Enum 패턴 매칭 전환
- **결정**: `kind.as_str() == "문자열"` 비교를 전부 `ItemKind::Xxx` / `MonsterKind::Xxx` 직접 비교로 전환.
- **배경**: R2-4에서 구조체 필드를 enum으로 전환했지만, 비교 시 `as_str()`로 다시 문자열화하는 비효율이 존재.
- **근거**: enum 직접 비교는 (1) 오타 시 컴파일 에러, (2) 문자열 해싱 불필요, (3) 리팩터링 시 IDE 지원 가능.
- **예외**: `artifact.rs`의 동적 변수 비교(`base`)는 enum 전환 불가 — 의도적 유지.

### R2-6. ItemManager/MonsterManager 병렬 인덱스 전략
- **결정**: `HashMap<String, Template>` 유지 + `HashMap<Kind, String>` 병렬 인덱스 추가. `get_by_kind()` API 제공.
- **배경**: 기존 `get_template(name: &str)` API를 사용하는 20+ 개소를 한번에 전환하면 리스크가 높음.
- **근거**: (1) 기존 API 100% 호환 유지, (2) 새 코드는 `get_by_kind()` 사용, (3) 향후 `EnumMap` 전환 시 API 변경 없이 내부 구현만 교체 가능.
- **대안 고려**: 완전 EnumMap 전환 (영향 범위가 너무 넓어 점진적 접근 채택).
- **인덱스 구축**: `AssetManager.load_defaults()` 완료 후 `build_kind_index()` 자동 호출.

---

## [2026-02-06] - 프로젝트 초기 설계

### 1. 전역 상태 관리 방식: ECS 선정
- **결정**: Legion (ECS - Entity Component System) 라이브러리 사용.
- **배경**: NetHack의 수많은 몬스터, 아이템, 함정의 상태를 효율적으로 관리하고, Rust의 소유권(Ownership) 갈등을 최소화하기 위함.
- **근거**: 각 엔티티의 컴포넌트화를 통해 객체 간 의존성을 분리하고, 시스템 단위의 독립적 처리가 가능함.
- **대안 고려**: Specs (성능은 유사하나 API가 불편), 자체 구조체 (유연성 부족)
- **결론**: Legion의 간결한 API와 높은 쿼리 성능이 이 프로젝트에 최적.

### 2. 하이브리드 UI (Ratatui + egui)
- **결정**: `egui`를 메인 호스트로 사용하고 중앙 캔버스에 `Ratatui`를 렌더링.
- **배경**: NetHack 특유의 TUI 미학을 유지하면서도, 인벤토리 관리 및 설정 등 복잡한 상호작용은 현대적인 GUI를 통해 사용자 편의성을 극대화하기 위함.
- **근거**: 순수 TUI는 마우스 지원이 약하고, 순수 GUI는 NetHack의 콘솔 느낌을 잃음. 하이브리드 방식이 양쪽 장점을 극대화.
- **대안 고려**: 순수 Ratatui (마우스 미지원), 순수 egui (아스키 미학 손실)

### 3. 입력 처리 프레임워크
- **결정**: `crossterm` 기반 이벤트 후킹 → 이후 egui 상태 폴링 방식으로 전환 (Phase 12).
- **배경**: Windows 환경에서의 네이티브 콘솔 제어 및 키보드/마우스 이벤트 처리.
- **근거**: egui 이벤트 루프와의 통합에서 상태 폴링(`ctx.input()`)이 더 안정적임이 확인됨.

---

## [2026-02-07] - 아이템 및 전투 엔진 고도화

### 4. Gather-Apply 패턴 도입
- **결정**: 사용자 입력(`Command`)을 즉시 처리하지 않고 `ItemAction` 리소스를 거쳐 시스템에서 소비하는 구조.
- **배경**: Legion ECS에서 동일 SubWorld에 대한 동시 가변 대여가 불가능한 제약.
- **근거**: 데이터를 읽는 과정(Gather)과 수정하는 과정(Apply)을 엄격히 분리하여 Borrow Checker 충돌 방지.
- **영향 범위**: 모든 시스템(equipment, combat, ai, item_use)에 적용됨.

### 5. 시야 기반 AI (LOS Pursuit)
- **결정**: Bresenham's Line Algorithm 기반 LOS 체크.
- **배경**: 벽 뒤의 플레이어를 투시하여 추격하는 불합리함 제거.
- **근거**: 원본 NetHack의 `vision.c`도 LOS를 사용하며, 전략적 플레이(잠입, 유인)를 가능하게 함.

---

## [2026-02-08] - 차세대 확장성

### 6. LLM-Ready 데이터 구조 (Future Vision)
- **결정**: 추후 1B급 초경량 Local LLM 연동을 고려하여 ECS 데이터를 JSON으로 직렬화 가능한 구조 유지.
- **배경**: 턴제 시스템의 이점을 활용하여 매 턴의 상황을 LLM이 분석할 수 있는 환경 구축.
- **근거**: `serde` 기반의 모든 컴포넌트 직렬화 지원으로 기반 확보.
- **주의**: 현재까지는 준비적 설계일 뿐, 실제 LLM 연동은 100% 이식 완료 후 진행.

---

## [2026-02-09] - 식사 엔진 및 능력치 시스템

### 7. 영양 기반 식사 엔진
- **결정**: `eat.c`의 영양가 계산 및 포만감 상태 머신을 `item_use.rs`에 통합.
- **배경**: NetHack의 핵심 생존 요소인 '배고픔'과 '질식'을 원본 공식 그대로 이식.
- **근거**: 시체 식사를 통한 내성 획득이 게임 전략의 핵심 요소이므로 충실한 이식 필요.

### 8. 가변 속성 구조 (base/max/exercise)
- **결정**: 플레이어 속성(Str, Int 등)을 `base`/`max`/`exercise` 구조로 개편.
- **배경**: 행동(발차기 등)을 통한 능력치 성장 및 일시적 상태(중독 등)에 의한 능력치 하락 관리.
- **근거**: `timeout` 시스템과 연동하여 주기적인 환경 체크 로직 필요.

---

## [2026-02-11] - 몬스터 AI 및 특수 레벨

### 9. 몬스터 주문 선택 로직 (Mage/Cleric)
- **결정**: `msound` 속성을 기반으로 Mage와 Cleric 주문군 분리, 난이도에 따른 확률적 주문 선택.
- **배경**: NetHack의 고전적인 몬스터 마법 시스템 유지.
- **근거**: Rust의 매치 구문을 통해 안전하고 확장성 있게 새 주문 추가 가능.

### 10. 특수 레벨 하드코딩 전략
- **결정**: `.des` 파일 파서 대신 하드코딩된 맵 데이터로 특수 레벨 생성.
- **배경**: `sp_lev.c`(5,441줄)의 `.des` 파일 파서 이식은 비용 대비 효과가 낮음.
- **근거**: Oracle, Minetown 등 핵심 레벨을 고정 좌표 기반으로 먼저 구현하고, 추후 파서 이식.
- **리스크**: `.des` 파일 기반의 다양한 레이aout 변형을 놓칠 수 있음.
- **향후 계획**: Phase 50+ 에서 `sp_lev.c` 이식 검토.

---

## [2026-02-12] - 전투/방어 고도화

### 11. StatusFlags u32→u64 확장
- **결정**: `StatusFlags` 비트필드를 `u32`에서 `u64`로 확장.
- **배경**: 기존 32비트로는 NetHack의 모든 상태 이상(Stoning, Sliming, Food Poisoning 등)을 표현 불가.
- **근거**: 향후 추가될 상태 플래그를 여유롭게 수용하기 위해 64비트로 미리 확장.

### 12. 파벌(Faction) 시스템 도입
- **결정**: 몬스터 간 적대/동맹 관계를 `Faction` 열거형으로 관리.
- **배경**: 원본 NetHack에서 엘프-오크 적대, 드워프-노움 우호 등의 종족 관계 구현.
- **근거**: `is_hostile_to()` 함수로 중앙 집중식 관계 판단, 추후 확장 용이.

---

## [2026-02-13] - UI 현대화 (GnollHack 참조)

### 13. AppState 기반 앱 상태 머신 도입
- **결정**: 기존의 즉시 게임 시작 방식 대신, Title → CharCreation → Playing → GameOver의 앱 상태 머신 도입.
- **배경**: 원본 NetHack도 캐릭터 생성(종족/직업 선택) 과정이 있으며, GnollHack은 이를 그래피컬하게 구현.
- **근거**: 게임 경험의 완성도를 위해 캐릭터 생성 과정이 필수적이며, 향후 멀티 세이브/하이스코어 기능 확장에도 필요.
- **참고**: `MODERNIZATION_PLAN.md` M1 참조

### 14. 하단 커맨드 바 (GnollHack Simple/Advanced)
- **결정**: 화면 하단에 마우스 클릭 가능한 커맨드 버튼 바를 배치. Simple(1줄)/Advanced(2줄) 전환 가능.
- **배경**: 키보드 단축키를 모르는 사용자를 위한 접근성 향상. GnollHack의 핵심 현대화 요소.
- **근거**: 원본 NetHack 경험을 해치지 않으면서(키보드 우선). 마우스 사용자에게 편의성 제공.
- **대안 고려**: 상단 리본 메뉴 (Office 스타일) → 게임 화면 침해가 심함. 하단 바가 더 적합.

### 15. 마우스 클릭 이동 (Travel 모드)
- **결정**: 맵 위 좌클릭으로 이동/공격. 인접 → 즉시, 원거리 → A* 자동이동.
- **배경**: GnollHack의 핵심 마우스 기능. 원본 NetHack의 `_` (Travel) 명령어와 동일한 로직.
- **근거**: A* 경로 탐색은 이미 `util/path.rs`에 구현되어 있어 재사용 가능.
- **주의**: 자동 이동 중 몬스터 감지/트랩 근접 시 반드시 중단해야 함 (원본 동작 보존).

### 16. 우측 Stats/Equipment 상시 패널
- **결정**: egui RightPanel에 능력치 + 장비 요약을 상시 표시.
- **배경**: GnollHack의 Desktop Buttons 기능. 별도 팝업 없이 현재 상태를 한눈에 확인.
- **대안 고려**: 플로팅 윈도우 (현재 인벤토리 방식) → 맵 가림. 고정 패널이 더 적합.

### 17. ASCII 미학 유지 (타일 미도입)
- **결정**: GnollHack의 2D 타일 그래픽은 채택하지 않음. ASCII 기반 Ratatui 렌더링 유지.
- **배경**: 이 프로젝트의 정체성은 "Rust 기반 현대적 ASCII NetHack"임.
- **근거**: 타일 에셋 제작/관리의 비용 대비, ASCII의 미학적 가치와 개발 효율성이 더 높음.
- **향후**: 100% 이식 완료 후 옵션으로 타일 모드 추가 검토 가능.

---

## [2026-02-14] - C-to-Rustic 아키텍처 전환

### 18. 원본 파일 1:1 매핑 → 도메인별 모듈 트리
- **결정**: 원본 `.c` 파일 경계를 따르지 않고, Rust 도메인 모듈(combat/, ai/, item/, creature/)로 재편.
- **배경**: 70개 파일이 `systems/`에 플랫하게 나열되어 있어 지식 탐색 비용이 극심.
- **근거**: 원본 C의 파일 경계는 "단일 컴파일 단위" 기준이었지, "기능 단위"가 아니었음. Rust에서는 모듈 시스템이 이 역할을 수행.
- **참조**: `REFACTORING_PLAN.md` Phase R3

### 19. 문자열 기반 템플릿 → 열거형 전환
- **결정**: `Monster.template: String` → `Monster.kind: MonsterKind` 등 모든 템플릿 참조를 타입 안전한 enum으로 전환.
- **배경**: `monster.template == "grid bug"` 같은 문자열 비교가 전체 코드에 ~500곳 산재. 오타 시 런타임 버그.
- **근거**: Rust의 열거형 + exhaustive match를 활용하면 컴파일 타임에 누락을 잡을 수 있음.
- **참조**: `REFACTORING_PLAN.md` Phase R2

### 20. main.rs 해체 (174KB → ~5KB)
- **결정**: `main.rs`의 게임 루프, 입력 처리, UI 렌더링을 `app.rs`, `world/turn.rs`, `input/handler.rs`로 분리.
- **배경**: 단일 파일 174KB는 IDE 성능, 컴파일 속도, 가독성 모두에 악영향.
- **근거**: 단일 책임 원칙(SRP) 적용. 각 모듈이 하나의 역할만 담당.
- **참조**: `REFACTORING_PLAN.md` Phase R1

### 21. Creature 트레이트 도입
- **결정**: 플레이어와 몬스터의 공통 인터페이스를 `Creature` 트레이트로 추출.
- **배경**: `combat.rs`와 `mhitu.rs`에서 거의 동일한 전투 로직이 반복됨.
- **근거**: Rust의 트레이트 시스템으로 코드 중복을 제거하고 새 몬스터 타입 추가를 용이하게 함.
- **참조**: `REFACTORING_PLAN.md` Phase R4

### 22. 이벤트 큐 도입 (시스템 간 느슨한 결합)
- **결정**: 시스템 간 통신을 `GameEvent` 열거형 기반 이벤트 큐로 전환.
- **배경**: `Player.equip_hunger_bonus`, `DeathResults` 등 시스템 간 중간 필드/브릿지 리소스가 난립.
- **근거**: 이벤트 기반으로 전환하면 시스템 추가/제거가 다른 시스템에 영향을 주지 않음.
- **참조**: `REFACTORING_PLAN.md` Phase R5

---

## 📋 향후 필요한 결정 사항

> 아래 항목은 향후 Phase에서 결정이 필요한 아키텍처 이슈입니다.

1. **`sp_lev.c` 이식 전략**: `.des` 파일 파서를 직접 이식할 것인가, 대안 포맷(TOML/JSON)을 사용할 것인가?
2. **반려동물 AI 수준**: `dog.c`/`dogmove.c`의 복잡한 배고픔/만족도/아이템 선호 로직을 어느 수준까지 이식할 것인가?
3. **세이브 파일 포맷**: 현재 JSON 기반이나, 원본 NetHack의 바이너리 세이브와의 호환성을 고려할 것인가?
4. **옵션 시스템 범위**: `options.c`(6,473줄)의 방대한 옵션 중 어느 범위까지 지원할 것인가?
5. **테스트 전략**: 원본 C와의 동작 동일성을 어떻게 체계적으로 검증할 것인가?

---

## [2026-02-14] - Phase R1: main.rs 해체 1단계

### 12. main.rs 분리 전략: impl 분산 패턴
- **결정**: `main.rs`의 `update()` 메서드(2,871줄)를 `game_loop.rs`와 `game_ui.rs`로 분리하되, Rust의 `impl` 블록 분산 기능을 활용하여 같은 `NetHackApp`에 대한 메서드를 여러 파일에 정의.
- **배경**: 174KB의 단일 파일은 개발 효율성과 IDE 성능을 심각하게 저해. 하지만 `update()` 내부의 로직이 `NetHackApp`의 필드에 광범위하게 접근하므로, 완전한 구조체 분리는 대규모 리팩토링 필요.
- **근거**: `impl SuperModule::Struct`를 서브모듈에서 정의하는 Rust 표준 패턴을 활용하면, 기존 코드를 최소한으로 변경하면서 파일 수준 분리 가능.
- **대안 고려**: (1) 완전 분리(`GameLoop` 별도 구조체) — `self` 참조 문제로 대규모 수정 필요, (2) 메서드 단위 분리만 — 파일 크기 감소 효과 미미
- **결과**: `main.rs` 174KB → 37.5KB (78% 축소), 에러 0개, 경고만 35개(기존과 동일)

### 13. last_cmd 필드 승격
- **결정**: `update()` 내 로컬 변수 `last_cmd: Command`와 `spell_key_input: Option<char>`를 `NetHackApp` 구조체 필드로 승격.
- **배경**: 원래 코드에서 `last_cmd`는 `update()` 안에서 선언되어 입력 처리(game_loop)와 UI 렌더링(game_ui) 양쪽에서 동일 프레임 내에 읽기/쓰기됨. 파일 분리 후 양쪽 메서드에서 접근 불가 문제 발생.
- **근거**: `self.last_cmd` 필드로 승격하면 양쪽 메서드에서 자연스럽게 접근 가능하고, 프레임 시작 시 `poll_input()`으로 초기화되므로 상태 누수 위험 없음.
- **대안 고려**: `render_game_ui()`가 `Command`를 반환하는 방식 — 하지만 현재 코드에서 UI가 `last_cmd`를 즉시 같은 프레임에서 수정(메뉴/커맨드바 클릭)하므로 호출 순서 변경 필요.
### 14. app.rs 분리 + 필드 가시성 결정
- **결정**: `NetHackApp` 구조체 정의 + `new()` + `restart_game()` + `initialize_game_with_choices()`를 `app.rs`로 분리. 모든 필드를 `pub(crate)`로 설정.
- **배경**: `main.rs`에서 구조체 정의를 분리하되, `game_loop.rs`, `game_ui.rs`, `input_handler.rs`에서 `self.필드`를 직접 접근해야 하므로 모듈 간 가시성 필요.
- **근거**: `pub(crate)`는 크레이트 내부에서만 접근 가능하게 하므로, 외부 노출 없이 내부 모듈 간 자유로운 필드 접근을 보장. `pub`보다 보수적이고, getter/setter 패턴은 현 단계에서 과도한 추상화.
- **결과**: `main.rs` 5.7KB(118줄)로 최종 축소, 순수 진입점(mod 선언 + update + main) 역할만 수행.

### 15. input_handler.rs 분리
- **결정**: `poll_input()` 메서드(228줄)를 `input_handler.rs`로 독립 분리.
- **배경**: `poll_input()`은 NetHack cmd.c 기반의 복잡한 키바인딩 매핑으로, 순수 입력→Command 변환 로직. 게임 상태 변경이나 UI 렌더링과 독립적.
- **근거**: 단일 책임 원칙 적용. 향후 키바인딩 커스터마이징, 매크로 지원 등 확장 시 독립된 파일에서 관리하기 용이.

### 23. Item.template: String → Item.kind: ItemKind 전환 (Phase R2)
- **결정**: `Item` 구조체의 `template: String` 필드를 `kind: crate::generated::ItemKind` 열거형으로 전환.
- **배경**: Monster.template → Monster.kind 전환에 이어, 아이템에도 동일한 타입 안전성 패턴을 적용. `item.template == "oil lamp"` 같은 문자열 비교가 ~200곳 이상 산재.
- **근거**: 
  - `ItemKind`는 `Copy` + `Clone` + `PartialEq` + `Hash` 구현으로 String보다 효율적
  - `build.rs`에서 `items.toml`로부터 자동 생성되어 데이터 일관성 보장
  - `as_str()` 메서드로 기존 문자열 비교 코드와의 호환성 유지
- **전환 전략**: 
  1. 필드명 `template` → `kind`로 변경
  2. 문자열 할당 → `ItemKind::from_str()` 사용
  3. 문자열 비교 → `item.kind.as_str() == "xxx"` (단계적 전환)
  4. HashMap 조회 → `get_template(item.kind.as_str())` (기존 API 유지)
- **시체(corpse) 처리**: 동적 이름 시체(`{monster} corpse`)는 `ItemKind::from_str()`로 `UnknownItem`에 매핑되며, 몬스터 종류는 `corpsenm` 필드로 구분.
- **대안 고려**: 시체 전용 `ItemKind::Corpse` variant 추가 — 향후 개선 대상.
- **결과**: ~30개 파일 수정, `cargo check` 에러 0개 달성.
### 24. 시스템 모듈 트리 재구성 (Phase R3)
- **결정**: `systems/` 아래 70개 플랫 `.rs` 파일을 9개 도메인별 서브디렉토리로 재구성.
- **배경**: 원본 NetHack C 파일을 1:1 매핑하여 `systems/` 아래 70개 파일이 계층 없이 나열되어 있었음.
- **근거**:
  - 기능별 그룹핑으로 코드 탐색 시간 단축 (IDE 파일 탐색, 코드 리뷰)
  - 새 시스템 이식 시 배치 위치 즉시 명확
  - 향후 Phase R4 트레이트 도입의 기반 (도메인별 `mod.rs`에 트레이트 정의 가능)
- **분류 기준**: NetHack의 원본 C 파일 역할과 게임 도메인 기반
  - `combat/`: 전투 (uhitm.c + mhitu.c + mhitm.c 계열)
  - `ai/`: 몬스터 AI (monmove.c + dog.c + mcastu.c 계열)
  - `item/`: 아이템 사용/관리 (eat.c + read.c + potion.c 계열)
  - `creature/`: 생물 공통 (status + equipment + movement)
  - `world/`: 월드/환경 (stairs + trap + vision + dig)
  - `social/`: 사회적 상호작용 (shop + talk + pray)
  - `spawn/`: 생성 (makemon + spawn_manager)
  - `identity/`: 명명/식별 (do_name + pager + botl)
  - `misc/`: 기타 (artifact + luck + timeout + spell)
- **호환성 전략**: `systems/mod.rs`에서 `pub use` re-export로 기존 `crate::core::systems::xxx` 경로 100% 유지
- **이름 충돌 처리**: `combat/combat.rs` → `combat/engine.rs`, `ai/ai.rs` → `ai/core.rs`로 rename
- **부산물**: `ai_part1~3.rs` 미사용 잔존 파일 발견 → 비활성화
- **결과**: 69개 파일 이동, 9개 서브모듈 생성, `cargo check` 에러 0개 / 경고 0개

### 25. Creature/UseEffect/Behavior 트레이트 도입 (Phase R4)
- **결정**: 3개 도메인(전투, 아이템, AI)에 통합 트레이트 인터페이스 도입.
- **배경**: 
  - 전투 시스템: 플레이어 공격(uhitm.rs)과 몬스터 공격(mhitu.rs)에 거의 동일한 로직이 중복
  - 아이템 시스템: `item_use.rs` 800줄+ match 분기에 모든 아이템 효과가 밀집
  - AI 시스템: 모든 몬스터가 `monster_ai()` 하나의 함수에서 분기 처리
- **ECS 호환 설계**:
  - 전통적 OOP 트레이트 대신 "컴포넌트 조합 → 스냅샷 구조체" 패턴 채택
  - `CreatureSnapshot`: ECS 컴포넌트(Health, CombatStats, StatusBundle 등)에서 추출한 전투 데이터
  - `Combatant`/`DamageReceiver`: 전투 시스템이 스냅샷을 통해 통일적으로 처리
- **트레이트 설계**:
  - `combat::CombatResult` — 전투 결과 구조체 (hit/miss/kill + 상태 이상)
  - `item::UseEffect` — 아이템 효과 (`apply()` → `UseResult`)
  - `item::UseResult` — 결과 (Success/NoEffect/Failure + 소비 여부)
  - `ai::Behavior` — AI 행동 결정 (`decide()` → `AiAction`)
  - `ai::Conversable` — 대화 인터페이스 (Phase R4.5 LLM 연동 대비)
- **기본 구현**:
  - `RuleBasedAi` — NetHack monmove.c 규칙 기반 (매 턴 0ms 보장)
  - `PetAi` — dogmove.c 펫 전용 AI
  - `ScriptedDialogue` — 고정 대사 기반 대화
- **레이턴시 경계**:
  - `Behavior::decide()` → 매 턴 × N마리, 0ms 필수 (규칙 기반 전용)
  - `Conversable::respond()` → 대화 개시 시에만, ~0.5초 허용 (LLM 가능)
- **결과**: 트레이트 정의만 추가, 기존 코드 무변경. `cargo check` 에러 0개.

### 26. GameEvent 이벤트 큐 시스템 도입 (Phase R5)
- **결정**: `core/events.rs`에 `GameEvent` enum + `EventQueue` + `EventHistory` 정의.
- **배경**:
  - 시스템 간 통신이 `DeathResults`, `equip_hunger_bonus` 등 브릿지 리소스/필드에 의존
  - 시스템 추가/제거 시 연쇄 수정 발생 (암묵적 의존)
  - Phase R4.5 LLM 연동 시 AI에 "최근 사건 문맥"을 전달할 표준 포맷 필요
- **설계 핵심**:
  - `GameEvent` — 20+ variant, 발생 사실의 기록(Command가 아님)
  - `EventQueue` — 매 턴 clear, 다수 소비자가 같은 이벤트 읽기 가능
  - `EventHistory` — 링 버퍼 200개, `recent_narrative()` for LLM 문맥
  - `category()` — 이벤트 분류 (combat/item/status/movement/social 등)
  - `to_narrative()` — 자연어 요약 (LLM 피딩 + 디버깅)
- **전환 전략**: 4단계 점진적 전환
  1. ✅ 이벤트 타입/큐 정의 (현재)
  2. ⬜ 기존 시스템에서 이벤트 발행 코드 병행 추가
  3. ⬜ 소비자 시스템을 이벤트 기반으로 전환
  4. ⬜ 브릿지 리소스(DeathResults 등) 최종 제거
- **결과**: 인프라 추가만, 기존 코드 무변경. `cargo check` 에러 0개.

### 27. 비트플래그 래퍼 + God Object 뷰 타입 도입 (Phase R6)
- **결정**: 의미적 열거형 래퍼 + 논리적 뷰 타입으로 단계적 정리.
- **배경**:
  - `MonsterFlags1/2/3` — C의 `permonst.geno/flags` 비트플래그 직이식. 60+ 개 비트가 3개 u32/u16에 밀집
  - `Player` — C의 `struct you` 직이식. 30+ 필드가 단일 구조체에 밀집 (God Object)
  - `StatusFlags` — 상시 저항력과 일시적 상태 이상이 같은 비트 공간에 혼재
- **MonsterCapability 설계**:
  - 60+ 비트를 도메인별(이동/신체/감각/특수/식성/종족/행동/퀘스트) 열거형으로 분류
  - `is_set_in(template)` → 내부에서 `has_flag1/2/3` 호출 (변환 비용 0)
  - `MonsterTemplate::has_capability()` — 새 API. 기존 `has_flag1()` 유지
- **StatusCategory 설계**:
  - 6종 분류: 일시적 디버프, 일시적 버프, 내재 저항력, 이동 모드, 하중, 치명적 상태
  - `is_dangerous()`, `is_timed()` — UI/시스템에서 상태 처리 방식 결정
- **Player 뷰 타입 설계**:
  - 4종 뷰: CombatView, SurvivalView, ProgressView, AttributeView
  - `from_player()` 팩토리 — Player에서 도메인별 스냅샷 추출
  - 향후 Player 분해 시 뷰 타입이 실제 ECS 컴포넌트로 승격 가능
- **JSON 호환성**: MonsterTemplate.flags1/2/3 필드 유지, 기존 YAML/JSON 데이터 무변경
- **결과**: 래퍼/뷰 타입 추가만, 기존 코드 무변경. `cargo check` 에러 0개.

---

### 28. 전투/사망 시스템 이벤트 실제 발행 (Phase R5 확장)
- **결정**: 사망 시스템에 직접 이벤트 발행 + 전투 시스템은 변환 API 제공.
- **death.rs 이벤트 발행 (직접 방식)**:
  - `#[resource] event_queue: &mut EventQueue` 를 Legion 시스템 파라미터로 추가
  - `MonsterDied` — 기존 `DeathResults`와 병행 발행. 점진적 전환 지원
  - `ExperienceGained` — 경험치 반영 직전에 발행
  - `PlayerDied` — GameOver 전환 직전에 발행
- **uhitm.rs 이벤트 변환 (간접 방식)**:
  - `player_attack()`은 순수 함수 (리소스 미접근) → EventQueue를 직접 받을 수 없음
  - 대신 `PlayerAttackResult::to_events(target_name, weapon_name)` 메서드 제공
  - 호출자가 결과를 받아 `event_queue.push()` 수행 → 순수 함수 아키텍처 유지
- **MonsterDied 필드 확장**:
  - `killer: String` — 처치자 (player 또는 몬스터 이름). LLM 문맥에 필수
  - `dropped_corpse: bool` — 시체 드롭 여부. 식단/저항 획득 연동
  - 기존 `x, y, xp_gained` 유지
- **결과**: 사망/전투 이벤트 5종 발행. `cargo check` 에러 0개.

### 29. 호출부(movement.rs, ai/core.rs)에서 이벤트 실제 push (Phase R5 확장)
- **결정**: 이벤트를 실제 계산 함수가 아닌 **호출부** 시스템에서 push.
- **movement.rs (플레이어→몬스터)**:
  - `#[resource] event_queue: &mut EventQueue` 추가
  - 명중 시 `GameEvent::DamageDealt` push (weapon_name 포함)
  - 빗나감 시 `GameEvent::AttackMissed` push
  - 매 공격마다 개별 이벤트 생성 (쌍수 공격 시 2개)
- **ai/core.rs (몬스터→플레이어)**:
  - `#[resource] event_queue: &mut EventQueue` 추가
  - 피해 적용 시 `GameEvent::DamageDealt` push
  - `source`에 `DamageType` Debug 포맷 사용 (Fire, Cold 등)
- **설계 근거**:
  - CombatEngine의 함수들은 순수 계산 함수 → EventQueue 접근 불가
  - 호출부(Legion `#[system]`)만 리소스 접근 가능 → 자연스러운 분리
  - `to_events()` API는 uhitm.rs의 결과를 변환하는 대안으로 제공 (Engine 호출 시)
- **결과**: 전투 이벤트 양방향 발행. `cargo check` 에러 0개.

### 30. 장비 시스템 이벤트 발행 (Phase R5 확장)
- **결정**: `equipment.rs`의 `equip_item()` / `unequip_item()` 헬퍼에서 이벤트 발행.
- **이벤트 전략**:
  - 장착 시: `ItemEquipped { item_name, slot }` + `EquipmentChanged`
  - 해제 시: `ItemUnequipped { item_name, slot }` + `EquipmentChanged`
  - `EquipmentChanged`는 하위 시스템(배고픔 보너스 재계산 등) 트리거용
- **설계 근거**:
  - `equip_item()`에 `&mut EventQueue` 파라미터 추가 (일반 함수이므로 직접 전달)
  - `equipment()` Legion 시스템이 `#[resource] event_queue` 접근 후 헬퍼에 전달
  - 이벤트 2개 쌍 발행: 세부 정보(`ItemEquipped`) + 변경 신호(`EquipmentChanged`)
- **TODO**: `unequip_item()`에서 실제 아이템 이름 전달 (현재 `"item"` 고정)
- **결과**: 장비 이벤트 3종 발행. `cargo check` 에러 0개.

### 31. 상태 이벤트 발행 (Phase R5 확장)
- **결정**: 상태 적용/만료 시 이벤트 발행.
- **StatusExpired (만료)**:
  - `status_tick` 시스템에서 `tick()` 반환 벡터 순회 시 자동 발행
  - 모든 만료 상태에 대해 일괄 적용 (코드 1곳)
- **StatusApplied (적용)**:
  - 각 `add()` 호출부에서 개별 발행 (8개 지점)
  - `item_use.rs`: 혼란, 실명, 속도, 마비, 질병, 질식, 두루마리 혼란
  - `ai/core.rs`: 독 (몬스터 공격)
  - `status.rs`: 수면 (기아 기절)
- **설계 근거**:
  - `StatusBundle::add()`에 이벤트를 내장할 수 없음 (리소스 접근 불가)
  - 호출부에서 직접 push — 문맥 정보(원인, 대상)를 알 수 있음
  - `engine.rs`의 `passive()` 내 BLIND는 향후 리팩토링 대상
- **부수 수정**: `item_use`에서 미사용 `_level_req` 리소스 제거 (Legion 8-tuple 한계)
- **결과**: 상태 이벤트 2종 발행 (8개 적용 지점 + 자동 만료). `cargo check` 에러 0개.

### 32. 이벤트 소비자 구현 및 브릿지 리소스 분석 (Phase R5 완성)
- **결정**: `game_loop.rs`의 턴 후처리에서 이벤트 라이프사이클 완성.
- **이벤트 라이프사이클**:
  1. 시스템 실행 중: `EventQueue::push()` — 각 시스템이 이벤트 발행
  2. 턴 끝: `EventQueue → EventHistory` 기록 — `record()` 호출
  3. 턴 끝: `EventQueue::clear(next_turn)` — 다음 턴 준비
- **설계 근거**:
  - 별도 Legion 시스템이 아닌 `game_loop.rs` 후처리에서 실행
  - Legion Schedule 실행 후 + DeathResults 처리 후 배치 (모든 이벤트 수집 완료 보장)
  - borrow 규칙 준수를 위해 EventQueue → Vec 복사 → EventHistory 기록 패턴
- **브릿지 리소스 대체 분석**:
  - `DeathResults`: SubWorld에서 `World::push()` 불가 → 구조적 필요 (대체 불가)
  - 현재 **병행 모드** 유지 (DeathResults + MonsterDied 이벤트 동시 발행)
  - 완전 대체는 ECS 아키텍처 변경 또는 CommandBuffer 확장 필요
- **결과**: 이벤트 라이프사이클 완성. 14번 항목은 구조적 한계로 보류. `cargo check` 에러 0개.

### 33. MonsterFlags → has_capability() 점진적 교체 (Phase R6)
- **결정**: `has_flag1/2/3()` 직접 호출을 `has_capability()` 래퍼로 전환.
- **전환 대상 (1차)**:
  - `mon.rs`: 22개 헬퍼 함수 (is_flyer, is_swimmer, tunnels, amorphous, passes_walls 등)
  - `monster.rs`: MonsterTemplate 메서드 30개 (is_flyer, is_undead, is_demon 등)
  - `spawn.rs`: faction 결정 로직 7개 (Orc, Elf, Dwarf, Gnome, Demon, Undead, Animal)
- **설계 근거**:
  - `has_capability()`는 내부에서 `has_flag1/2/3()`를 호출하므로 동작 변경 없음
  - 호출부에서 `MonsterFlags1::FLY` 같은 비트 상수 대신 `MonsterCapability::Fly` 사용
  - 의미적 가독성 향상 + JSON 데이터/직렬화 무변경
- **주의사항**: `has_flag1/2/3()` 메서드 자체는 유지 (capability.rs에서 내부 사용)
- **1차 결과**: 59개 호출 전환. `mon.rs`에서 `MonsterFlags1/2/3` import 제거.
- **2차 전환 (전체 코드베이스 완료)**:
  - `spawn.rs`(+1), `uhitm.rs`(2), `ai_helper.rs`(6), `regeneration.rs`(1)
  - `monmove.rs`(13), `ai_part3.rs`(1), `core.rs`(2), `engine.rs`(3), `throw.rs`(1)
  - 미사용 import 7개 파일 정리: `object_data.rs`, `gen.rs`, `botl.rs`, `mkobj.rs`, `pickup.rs`, `detect.rs`, `inventory.rs`
- **최종 결과**: **총 89개 호출 전환**. 외부 코드에서 `has_flag1/2/3` 직접 호출 0개. `cargo check` 에러 0개.

### 34. WornSlots 비트플래그 → WornSlot enum 도입 (Phase R6-7)
- **결정**: `WornSlots`의 `u32` 비트 상수(ARM, ARMC 등)를 `WornSlot` enum으로 래핑.
- **설계 방법**:
  - `WornSlot` enum: 12개 variant (Armor~Saddle), `to_bit()`/`from_bit()` 양방향 변환
  - `WornSlots`에 enum 기반 메서드: `has()`/`wear()`/`unwear()`/`worn_slots()`/`empty()`
  - `do_wear.rs`에 enum 기반 함수 5개: `item_to_worn_slot`, `can_wear_slot`, `can_remove_slot`, `wear_slot_message`, `remove_slot_message`
  - 기존 `u32` API는 하위 호환용으로 유지 (점진적 전환 가능)
- **설계 근거**:
  - 비트 상수는 `slot: u32`로 전달되어 잘못된 값(0, 복합 비트마스크) 전달 시 런타임 버그 유발
  - enum은 컴파일 타임에 유효한 슬롯만 허용 → 타입 안전성 확보
  - `is_armor()`, `is_accessory()`, `is_ring()` 등 카테고리 분류 가능
- **결과**: 테스트 11개 통과. `cargo check` 에러 0개. 기존 코드 무변경.

### 35. StatusFlags → StatusEffect enum 도입 (Phase R6-6)
- **결정**: `StatusFlags`의 44개 `u64` 비트 상수를 `StatusEffect` enum으로 래핑하고, 6개 카테고리로 컴파일 타임 분류.
- **설계 방법**:
  - `StatusEffect` enum: 44개 variant (Blind~Choking), 카테고리별 논리적 그룹화
  - `EffectCategory` enum: TemporaryDebuff, TemporaryBuff, IntrinsicResistance, MovementMode, Encumbrance, LethalCondition
  - `to_flag()`/`from_flag()` 양방향 변환, `category()` const fn으로 컴파일 타임 결정
  - `StatusBundle`에 enum API 7개: `has_effect`, `add_effect`, `remove_effect`, `set_permanent`, `clear_permanent`, `active_effects`, `effects_by_category`, `tick_effects`
  - 기존 `StatusFlags` u64 API는 하위 호환용으로 유지
- **설계 근거**:
  - `StatusFlags::BLIND`와 `StatusFlags::FIRE_RES`가 동일 타입(u64)으로 저항력과 상태이상이 혼재 → 논리적 오류 가능
  - `capability.rs`의 `StatusCategory::of()`는 런타임 판별 → 성능/정확성 이슈
  - enum의 `category()` const fn으로 컴파일 타임에 분류 확정 → 명확한 의도 표현
  - `effects_by_category()`로 "현재 치명적 상태만" 등 도메인 쿼리 가능
- **결과**: 테스트 6개 통과. `cargo check` 에러 0개. 기존 코드 무변경.

### 36. Player God Object 뷰 타입 완전 구현 (Phase R6-5)
- **결정**: Player 구조체(30+ 필드)를 즉시 분해하지 않고, 4개 뷰 타입에 양방향(추출 + 적용) 메서드를 완비.
- **설계 방법**:
  - `PlayerCombatView`, `PlayerSurvivalView`, `PlayerProgressView`, `PlayerAttributeView` — 이전 Step 1에서 정의 완료
  - `from_player()` + `apply_to()` 양방향 메서드: 뷰에서 변경한 데이터를 Player에 역으로 기록
  - `Player`에 `as_combat_view()`, `as_survival_view()`, `as_progress_view()`, `as_attribute_view()` 팩토리 메서드 추가
  - 뷰 독립성 보장 — 각 뷰의 변경이 다른 뷰에 영향 없음 (테스트 검증)
- **설계 근거**:
  - Player 전체가 ECS 컴포넌트로 사용되면 시스템 의존성이 불명확 → 뷰로 필요 범위 명시
  - `apply_to()` 패턴: 시스템이 뷰를 받아 로직 처리 후 결과를 Player에 기록 → 향후 Player 분해 시 뷰 자체가 컴포넌트로 승격
  - 기존 시스템 코드 변경 없이 새 시스템에서 뷰 패턴 선택적 채택 가능
- **결과**: 테스트 8개 통과. `cargo check` 에러 0개. 기존 코드 무변경.

---

### 결정 #37: 프로젝트 최종 비전 — AI Roguelike (2026-02-15)

> ⚠️ **최상위 의사결정**: 이 결정은 프로젝트의 존재 이유를 정의한다.

- **결정 내용**: AIHack의 100% Rust 이식은 **최종 목표가 아니라 기반 인프라**이다.
  - **Phase 1** (현재): NetHack 3.6.7의 100% 완전 이식 — 동작 완전 재현, Rust 아키텍처 확립
  - **Phase 2** (이식 후): 로컬 LLM 이식 — 게임 엔진 내부에 경량 LLM을 통합
  - **Phase 3** (최종): **세계 최초 AI 추론 기반 정통 로그라이크** 장르 개척
    - LLM이 던전 서술, NPC 대화, 몬스터 행동, 이벤트 생성을 실시간 추론으로 수행
    - 클라우드 의존 없이 **로컬 추론**으로 동작하는 완전 오프라인 AI Roguelike
    - 기존에 존재하지 않는 **신규 장르**
- **구조가 중요한 이유**:
  - LLM을 게임 엔진에 이식하려면 **깨끗한 인터페이스**가 필수
  - ECS 아키텍처 → LLM이 게임 상태(엔티티, 컴포넌트, 리소스)를 **구조화된 데이터**로 읽을 수 있음
  - enum 기반 타입 시스템 → LLM의 출력을 **타입 안전하게 파싱** 가능
  - 이벤트 큐(GameEvent) → LLM이 이벤트를 **발행/구독**하여 게임에 개입 가능
  - 이것이 C 직역 금지, ECS 일관성, enum 전환을 강제하는 **근본적 이유**
- **아키텍처적 준비사항** (Phase 1에서 확보해야 할 것):
  1. 모든 게임 상태가 직렬화 가능한 구조(ECS Component/Resource)로 존재
  2. 게임 이벤트가 구조화된 enum으로 표현 (문자열 파싱 불필요)
  3. 몬스터 AI 시스템이 독립 모듈로 분리 (LLM으로 교체 가능)
  4. NPC 대화 시스템이 데이터 구동 (LLM 생성 텍스트로 대체 가능)
  5. 던전 서술이 템플릿 기반 (LLM 서술로 확장 가능)
- **설계 근거**: 
  - AI로 동작하는 게임을 만들려면, AI로 만드는(이식하는) 과정에서부터 AI-friendly한 구조가 필요
  - "나중에 LLM을 붙이자"가 아니라, "LLM이 붙을 수 있는 구조로 지금 이식하자"
  - 이것이 단순 포팅 프로젝트와 이 프로젝트의 본질적 차이

---

### 결정 #38: AI 에이전트 협업 방법론 (2026-02-15)

- **결정 내용**: AI 모델의 특성에 따라 작업을 분담하고, 코드베이스 모호성을 관리하는 체계를 확립한다.
- **배경**:
  - 이 프로젝트는 시작부터 AI 에이전트로 개발됨 (인간 + AI 페어 프로그래밍)
  - 초기(~20%): Gemini 3 Flash로 기계적 이식 — 패턴을 따라가는 작업에 적합
  - 중반(20%~): 리팩토링(R1~R6) 수행하여 아키텍처 레일 확립
  - **AI는 기존 패턴을 따라가는 특성이 강함** → 20%에서 올바른 구조를 잡으면, 이후 이식도 그 구조를 따름
  - 현재(37%~): 코드베이스 증가로 **모호성 증가** — 미구현 함수가 "있는 것처럼" 보이는 현상 발생
- **모델별 역할 분담**:
  | 작업 유형 | 적합 모델 | 이유 |
  |-----------|----------|------|
  | 기계적 이식 (패턴 반복) | Flash급 (빠름, 저비용) | 확립된 패턴을 따라가는 작업 |
  | 모호성 감지 + 검증 | Thinking급 (opus 등) | "이것이 진짜 완성인가?" 자문 능력 |
  | 아키텍처 결정 | Thinking급 | 시스템 간 상호작용 분석 |
  | 원본 C 대조 이식 | Thinking급 | 대용량 컨텍스트 + 추론 체인 |
  | 문서 동기화 | 어느 모델이든 | 규칙 기반 반복 작업 |
- **모호성 관리 규칙**:
  1. **미사용 파라미터 = 미구현 로직의 증거**: `unused_variable` 경고가 있는 함수는 "완성"이 아님
  2. **이식 완성도 어노테이션**: 주석에 `@port_status: PARTIAL/COMPLETE` 명시
  3. **Thinking 모델 진입 시 필수 점검**: 이식률 실측, 미구현 목록 대조, 미사용 파라미터 확인
  4. **컨텍스트 전달**: 새 세션 시작 시 `audit_roadmap.md` → `designs.md` → 해당 Phase 소스 순으로 읽기
- **결과**: 37% 시점에서 `unsafe` 0개, `static mut` 0개, `todo!()` 0개 — 초기 구조화가 성공적이었음을 실측으로 확인

---

## [2026-02-17] - v2.9.0~v2.9.3: 40% 이식률 돌파 및 합동 감사

### PORT-1. 호출부 우선 원칙 (Caller-First Principle) 도입
- **결정**: 유틸 함수 이식 시 반드시 호출하는 시스템/함수도 함께 구현한다.
- **배경**: v2.9.2까지 이식된 유틸 함수 약 30여개가 테스트만 통과하고 실제 게임 루프에서 호출되지 않는 "섬 코드(Island Code)" 상태로 방치됨.
- **근거**: (1) 테스트 통과는 동작 보장이 아님, (2) ECS 래퍼 없이는 게임에서 실행 불가, (3) 섬 코드 누적은 실질 이식률을 왜곡함.
- **적용**: `monster_ai` 시스템에 `steal_check`, `rust_attack_effect`, `lycanthropy_attack`, `disease_attack` 5종 연결 완료.

### PORT-2. ECS 래퍼 의무화
- **결정**: 순수 함수만 이식하고 ECS 래퍼를 구현하지 않는 것을 미완성으로 간주한다.
- **배경**: 순수 함수 설계는 테스트 용이성과 모듈성에 좋지만, 실제 `SubWorld`→컴포넌트 접근→결과 기록 체인이 없으면 게임에서 동작하지 않음.
- **대안 검토**: (1) 순수 함수 + `#[system]` 래퍼 분리, (2) 시스템 내 인라인 로직. → (1) 채택.

### PORT-3. 매직넘버 상수화 규칙
- **결정**: NetHack 원본의 하드코딩 수치를 `const`로 분리하고 원본 참조 주석을 필수로 한다.
- **배경**: `steal_check` 내 `70`, `50`, `60` 등 확률값이 의미 불명 상태로 방치됨.
- **근거**: 원본 C에서도 하드코딩이지만, Rust 포팅의 이점을 살려 유지보수성 향상.

### PORT-4. 감사 체크리스트 제도화
- **결정**: 이식 완료 시 6항목 체크리스트(빌드/테스트/호출확인/상수화/주석/문서동기화) 점검 후 커밋한다.
- **배경**: v2.9.2까지 이식 후 문서 동기화 누락, 경고 방치 등이 반복됨.
- **적용 위치**: `audit_roadmap.md` 하단 "이식 가이드라인" 섹션에 명시.

---

## [2026-02-18] - v2.9.6~v2.10.0: 소중형 모듈 집중 완전 이식

### PORT-5. 순수 결과 패턴(Pure Result Pattern) 표준화
- **결정**: 원본 C의 부수효과(side-effect) 없는 판정 로직을 `XxxResult` enum + `xxx_result()` 순수 함수로 이식하는 패턴을 표준 이식 패턴으로 공식화.
- **배경**: sit.rs, fountain.rs, wizard.rs, mkroom.rs 4개 모듈에서 일관되게 적용, 모두 100%+ 이식률 달성.
- **근거**: (1) ECS 의존 없이 독립 테스트 가능, (2) match exhaustive로 누락 방지, (3) 원본 C의 거대 switch-case와 1:1 대응.
- **구체적 사례**:
  - `DrinkFountainEffect` (16종) — fountain.c drinkfountain의 31개 분기를 enum으로 완전 매핑
  - `CourtMonsterClass` (9종) — mkroom.c courtmon의 9단계 if-else를 enum으로 변환
  - `SquadType` (4종) — mkroom.c squadmon의 확률 테이블 기반 선택
  - `MorgueMonType` (5종) — mkroom.c morguemon의 난이도/지역 기반 선택

### PORT-6. cmap_to_type 심볼→타일 완전 매핑
- **결정**: 원본 mkroom.c의 `cmap_to_type` 함수를 41개 심볼 인덱스에 대해 TileType으로 완전 매핑 구현.
- **배경**: 기존에는 미구현으로, 미믹이 가구로 위장할 때 기억된 터레인을 설정하는 로직이 누락.
- **근거**: 각 심볼을 정확한 TileType variant(Stone, VWall, HWall, TlCorner, ..., Cloud, Water)에 매핑하여 원본 완전 일치.

### PORT-7. rnl 행운 조정 랜덤 함수 추가
- **결정**: rng.rs에 `rnl(x, luck)` 함수를 추가하여 rnd.c의 마지막 미구현 함수를 완료.
- **배경**: 행운이 좋으면 0 쪽, 나쁘면 (x-1) 쪽으로 결과가 편향되는 핵심 RNG 함수로, 저주/축복 판정 등에 광범위하게 사용.
- **근거**: 작은 범위(x≤15)에서는 luck/3 보정, 큰 범위에서는 luck 직접 보정이라는 원본의 이중 보정 체계를 정확히 구현.

### PORT-8. _ext 모듈 분리 패턴 도입 (8개 소형 파일 완전 이식)
- **결정**: 원본 C 파일이 500줄 이하인 소형 모듈을 `xxx_ext.rs`라는 독립 확장 파일로 이식하는 패턴을 표준화.
- **배경**: v2.10.1에서 lock.c, steal.c, light.c, bones.c, were.c, rip.c, write.c, minion.c 8개 파일을 일괄 이식하면서 기존 모듈과의 충돌 없이 확장하는 방법이 필요.
- **근거**: (1) 기존 `steal.rs`/`lock.rs` 등에는 이미 확장 로직이 있으므로, 원본 핵심 로직을 `_ext.rs`로 분리하면 관심사 분리 달성, (2) 순수 결과 패턴(PORT-5)과 결합하여 독립 테스트 가능, (3) 소형 파일이므로 단일 세션에서 100% 완료 가능.
- **적용 모듈**: lock_ext.rs(12함수/18테스트), steal_ext.rs(8함수/12테스트), light_ext.rs(15함수/14테스트), bones_ext.rs(10함수/19테스트), were_ext.rs(10함수/9테스트), rip_ext.rs(5함수/8테스트), write_ext.rs(7함수/12테스트), minion_ext.rs(10함수/12테스트).
- **결과**: 단일 세션에서 +7,037줄, +181테스트 추가. 이식률 48.1%→52.1% (50% 돌파).

---

**문서 버전**: v2.19.0
**최종 업데이트**: 2026-02-21

### 39. ActionQueue �ý��� ���� (Phase R7-1/2/3)
- **����**: ActionQueue �ý����� ���� ������ �����ϰ� ������ ��� ���� ������ ť ��� �������� ��ȯ.
- **���**: AI�� �÷��̾��� �ൿ ������ �и��ϰ� Ȯ�强 �ִ� ���� ä��.
- **���**: main.rs, pp.rs � ť ���� �ݿ� �Ϸ�.
