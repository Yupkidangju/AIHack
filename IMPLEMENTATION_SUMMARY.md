# AIHack 구현 현황 (Implementation Summary)

- **프로젝트명**: AIHack (NetHack 3.6.7 → Rust 100% Port)
- **현재 버전**: v2.20.0 (Phase R7 Architecture Modernization)
- **최근 업데이트**: 2026-02-22
- **원본 소스**: 177,232 라인 (C 소스) + 20,097 라인 (헤더) = **197,329 라인**
- **현재 Rust**: **114,280 라인** (192개 파일) [2026-02-22 실측치]
- **이식률**: **64.5%** (C 소스 기준)
- **테스트**: **2,189개** 전체 통과 [2026-02-22 실측치]
- **목표**: **100%**

---

## 모듈별 구현 현황 (2026-02-22 v2.20.0 실측치)

### 주요 파일 라인수(Rust) 상위 50개

| 파일 | 라인 수 | 대응 원본 | 원본 라인 | 이식률 |
|-----|---------|----------|----------|--------|
| `mon.rs` (entity) | 4,746 | `mon.c`+`monst.c` | 8,252 | 57.5% |
| `monmove.rs` | 2,494 | `monmove.c` | 1,578 | 158.1% |
| `dog.rs` | 2,140 | `dog.c`+`dogmove.c` | 2,497 | 85.7% |
| `objnam.rs` | 2,065 | `objnam.c` | 4,031 | 51.2% |
| `zap.rs` | 1,980 | `zap.c` | 5,016 | 39.5% |
| `equipment.rs` | 1,850 | `do_wear.c`+`worn.c` | 3,633 | 50.9% |
| `trap.rs` | 1,810 | `trap.c` | 5,112 | 35.4% |
| `game_ui.rs` | 1,782 | `cmd.c`+`display.c` | ~8,081 | ~22.0% |
| `hack.rs` | 1,675 | `hack.c` | 2,939 | 57.0% |
| `eat.rs` | 1,619 | `eat.c` | 3,110 | 52.1% |
| `mcastu.rs` | 1,424 | `mcastu.c` | 1,623 | 87.7% |
| `inventory.rs` | 1,307 | `invent.c` | 4,161 | 31.4% |
| `mkobj.rs` | 1,246 | `mkobj.c` | 2,739 | 45.5% |
| `pickup.rs` | 1,236 | `pickup.c` | 3,008 | 41.1% |
| `apply.rs` | 1,196 | `apply.c` | 3,527 | 33.9% |
| `movement.rs` | 1,186 | `hack.c` | 2,939 | 40.4% |
| `end.rs` | 1,183 | `end.c` | 2,092 | 56.5% |
| `mhitu.rs` | 1,179 | `mhitu.c` | 2,819 | 41.8% |
| `game_loop.rs` | 1,165 | `allmain.c` | 536 | 217.4% |
| `muse.rs` | 1,154 | `muse.c` | 3,052 | 37.8% |
| `gen.rs` | 1,119 | `mklev.c`+`mkmaze.c` | 3,476 | 32.2% |
| `do_wear.rs` | 1,114 | `do_wear.c` | 2,816 | 39.6% |
| `evolution.rs` | 1,091 | `polyself.c` | 1,776 | 61.4% |
| `do_name.rs` | 1,032 | `do_name.c` | 1,394 | 74.0% |
| `weapon.rs`+`weapon_ext.rs` | 2,912 | `weapon.c` | 1,532 | ~100% |
| `wield_ext.rs` | 846 | `wield.c` | 893 | ~80% |
| `explode.rs`+`explode_ext.rs` | 1,306 | `explode.c` | 820 | ~90% |
| `dothrow_ext.rs` | 885 | `dothrow.c` | 2,189 | ~40% |
| `priest_ext.rs` | 824 | `priest.c` | 1,100 | ~75% |
| `music_ext.rs` | 560 | `music.c` | 990 | ~60% |
| `potion_ext.rs` | 661 | `potion.c` | 2,413 | ~27% |
| `read_ext.rs` | 895 | `read.c` | 2,653 | ~34% |
| `mthrowu_ext.rs` | 801 | `mthrowu.c` | 1,216 | ~66% |
| `spell_ext.rs` | 1,011 | `spell.c` | 1,898 | ~53% |
| `detect_ext.rs` | 694 | `detect.c` | 2,033 | ~34% |
| `teleport_ext.rs` | 664 | `teleport.c` | 1,585 | ~42% |
| `dokick_ext.rs` | 430 | `dokick.c` | 1,812 | ~24% |
| `steed_ext.rs` | 375 | `steed.c` | 781 | ~48% |
| `sounds_ext.rs` | 365 | `sounds.c` | 1,183 | ~31% |
| `fountain_ext.rs` | 750 | `fountain.c`+`sit.c` | 1,121 | ~67% |
| `dig_ext.rs` | 350 | `dig.c` | 2,153 | ~16% |
| `artifact_ext.rs` | 380 | `artifact.c` | 2,206 | ~17% |
| `trap_ext.rs` | 1,137 | `trap.c` | 5,477 | ~21% |
| `pray_ext.rs` | 530 | `pray.c` | 2,303 | ~23% |
| `eat_ext.rs` | 530 | `eat.c` | 3,353 | ~16% |
| `dbridge_ext.rs` | 340 | `dbridge.c` | 1,005 | ~34% |
| `timeout_ext.rs` | 390 | `timeout.c` | 2,429 | ~16% |
| `region_ext.rs` | 320 | `region.c` | 1,129 | ~28% |
| `ball_ext.rs` | 380 | `ball.c` | 1,115 | ~34% |
| `vault_ext.rs` | 350 | `vault.c` | 1,152 | ~30% |
| `worm_ext.rs` | 350 | `worm.c` | 875 | ~40% |
| **[v2.19.0 추가]** | | | | |
| `uhitm_ext.rs` | 700 | `uhitm.c` | 2,975 | ~24% |
| `invent_ext.rs` | 550 | `invent.c` | 4,161 | ~13% |
| `shk_ext.rs` | 700 | `shk.c` | 4,542 | ~15% |
| `mhitu_ext.rs` | 770 | `mhitu.c` | 2,819 | ~27% |
| `apply_ext.rs` | 590 | `apply.c` | 3,527 | ~17% |
| `do_wear_ext.rs` | 540 | `do_wear.c` | 2,816 | ~19% |

---

## 완료된 Phase 목록

### 기반 시스템 (Phase 1-11)
- **Phase 1-5**: 핵심 기반 — ECS 설정, 맵 생성, 이동, 전투, FOV
- **Phase 6**: 데이터 통합 — MonsterTemplate/ItemTemplate 확장, TOML 데이터베이스
- **Phase 7**: 몬스터 AI — LOS 추적, 다중 공격, 시야 기반 추격 (부분 완료)
- **Phase 8**: 마법 시스템 — 마법서 학습, 에너지 시스템, 직선 궤적 발전
- **Phase 9**: 세계 상호작용 — 트랩, 제단/기도, 분수/싱크/왕좌, 상점
- **Phase 10**: 상태 진화 — 상태 이상 관리, 변이, 레벨 드레인
- **Phase 11**: 식사 시스템 — 영양소, 부패, 독, 시스템, 아티팩트

### 긴급 수정 및 고도화 (Phase 12-19)
- **Phase 12**: 긴급 버그 4건 수정 (몬스터 스폰, 맵 생성, 출력, UI)
- **Phase 13**: 전투 공식 정밀화 (명중/회피/데미지)
- **Phase 14**: 몬스터 AI 확장 (A*, MUSE, 특수 방)
- **Phase 15**: 메시지/인벤토리 UI 혁신
- **Phase 16**: 세이브/로드 + 옵션 시스템
- **Phase 17**: 수동형 반격 (Passive Attacks)
- **Phase 18**: 지팡이 물리 정밀 (반사/폭탄)
- **Phase 19**: 텔레포트/층간 이동

### 아키텍처 현대화 및 확장 (Phase R1-R7)
- **Phase R1-R6**: 기반 모듈/아키텍처 스캐폴딩 변경 적용 및 뷰어 통합.
- **Phase R7 (v2.20.0)**: **[현재 완료 단계]** ActionQueue 방식 액션 처리 통합, 안전성(`unwrap` 제거 및 `thiserror` 연동) 개선, LLM 상호작용(`InteractionProvider`) 구조 추상화 도입.

### 콘텐츠 이식 (Phase 20-49) — 대수 완료
- **Phase 20-49**: 가변 출력치, 식사, 도구, 용기, 기도, 명칭, 스킬, 아티팩트, 특수 전술, 던전 분기, 감정, 방어, 상점 AI, 지팡이, 상태 복구, 특수 공격, 몬스터 마법, 특수 레벨, 장기전, 지형, 조명, 트랩, 상태 이상, 아이템 보상, 경제, 컨테이너, 고급 AI

### 리팩토링 Phase (R1-R6) — 완료 ✅
- **Phase R1**: 거대 파일 분리 — main.rs(174KB) → 5개 파일
- **Phase R2**: 문자열→Enum 전환 — MonsterKind/ItemKind enum + get_by_kind() API
- **Phase R3**: 시스템 모듈 재구조화 — systems/ 하위 7개 서브디렉토리
- **Phase R4**: Creature/UseEffect/Behavior 트레이트 도입
- **Phase R5**: GameEvent 이벤트 큐 시스템 (20+ variant)
- **Phase R6**: 비트플래그 래퍼 + Player 뷰 타입

### UI 현대화 (M1) — 완료 ✅
- **M1**: AppState 상태 머신 — Title/CharCreation/Playing/GameOver

---

## 버전별 라인 수 변화

| 버전 | 라인 수 | 테스트 수 | 주요 변경 |
|------|---------|----------|----------|
| v2.1.0 | 40,489 | 230 | 기반 시스템 완료 |
| v2.2.0 | 49,745 | 280 | 콘텐츠 이식 |
| v2.3.0 | 53,247 | 299 | 대규모 이식 1차 |
| v2.3.1 | 54,514 | 319 | 대규모 이식 2차 |
| v2.3.2 | 56,020 | 330 | 모듈 확장 |
| v2.3.3 | 56,920 | 351 | 10개 파일 확장 |
| v2.3.4 | 59,326 | 426 | 10개 파일 확장 2차 |
| v2.3.5 | 65,862 | 470 | 8개 파일 확장 |
| v2.5.0 | 65,862 | 618 | inventory.rs + objnam.rs + do_name.rs 이식 |
| v2.7.0 | 67,448 | ~690 | pickup.rs + mkobj.rs 이식 |
| v2.8.0 | 70,302 | ~720 | dungeon.rs 대량 이식 + eat.rs 실측 반영 |
| v2.9.4 | ~78,000 | ~1,000 | muse.rs 신규 + zap.rs/mon.rs/mcastu.rs 대규모 이식 |
| v2.9.5 | 82,267 | 1,066 | dog.rs 2차 이식 (2,140줄) |
| v2.9.6 | 83,012 | 1,087 | dog.rs 3차 이식 완결 |
| v2.9.7 | 83,655 | 1,108 | wizard.rs 전량 이식 |
| v2.9.8 | 83,936 | 1,124 | sit.rs 전량 이식 |
| v2.9.9 | 84,805 | 1,149 | fountain.rs + rng.rs 전량 이식 |
| v2.10.0 | 85,259 | 1,168 | mkroom.rs 전량 이식 |
| **v2.10.1** | **92,296** | **1,349** | **8개 _ext 모듈 이식 (lock/steal/light/bones/were/rip/write/minion) — 50% 돌파** |
| v2.11.0 | 97,137 | 1,510 | weapon_ext/wield_ext/explode_ext/dothrow_ext/priest_ext/music_ext 대량 이식 |
| v2.14.0 | 103,085 | 1,756 | v2.12.0~v2.14.0: potion_ext~sounds_ext 9개 모듈 이식 (58.2%) |
| v2.16.0 | 106,157 | 1,873 | trap_ext/pray_ext/eat_ext 이식 (60% 돌파) |
| v2.17.0 | 107,556 | 1,936 | dbridge_ext/timeout_ext/region_ext 이식 |
| **v2.18.0** | **108,900** | **1,996** | **ball_ext/vault_ext/worm_ext 이식 — 총 27개 _ext 모듈, 61.4%** |
| **v2.19.0** | **114,280** | **2,186** | **uhitm_ext/invent_ext/shk_ext/trap_ext확장/mhitu_ext/apply_ext/do_wear_ext (75함수, 190테스트) — 총 33개 _ext 모듈, 64.5%** |

---

## 현재 미구현 핵심 시스템 (향후 작업 필수)

> ⚠️ 다음 시스템들은 NetHack 100% 이식을 위해 반드시 구현해야 하며,
> 원본 C 파일의 모든 조건문/분기/예외를 포함해야 합니다.

### 우선순위 높음 (게임 플레이에 필수)
1. **`cmd.c` 완전 이식** (5,661줄) — 100+ 명령어 디스패처, 확장 명령
2. **`invent.c` 나머지 이식** (4,161줄, 현재 ~28.6%) — 인벤토리 전체 관리 문자 슬롯
3. **`sp_lev.c` 이식** (5,441줄) — 특수 레벨 파서
4. **`trap.c` 나머지 이식** (5,112줄, 현재 ~59.7%) — 트랩 전체 로직
5. **`zap.c` 나머지 이식** (5,016줄, 현재 39.5%) — 마법봉/레이/빔 전체
6. **`shk.c` 나머지 이식** (4,542줄, 현재 ~35.4%) — 상점 전체 거래

### 우선순위 중간 (게임 완성도에 중요)
1. `potion.c` 나머지, `read.c` 나머지, `eat.c` 나머지 (현재 52.1%)
2. `apply.c` 나머지 (현재 ~52.9% — apply_ext 추가)
3. `dog.c`+`dogmove.c` 나머지 (현재 **99%+** — 3차 이식 완결)
4. `muse.c` 나머지 (현재 37.8%)
5. `detect.c`, `dig.c`, `lock.c` 나머지
6. `mhitm.c` 나머지, `weapon.c` 나머지

### 우선순위 낮음 (최종 완성 단계)
1. `options.c` (6,473줄), `display.c` (2,420줄)
2. `sounds.c` (1,108줄), `music.c` (918줄)
3. `region.c` (1,024줄), `vault.c` (1,074줄)
4. ~~`bones.c` (628줄)~~ → ✅ `bones_ext.rs` 이식 완료, `topten.c` (1,156줄)
5. ~~`were.c` (222줄)~~ → ✅ `were_ext.rs` 이식 완료
6. ~~`rip.c` (170줄)~~ → ✅ `rip_ext.rs` 이식 완료
7. ~~`write.c` (391줄)~~ → ✅ `write_ext.rs` 이식 완료
8. ~~`minion.c` (522줄)~~ → ✅ `minion_ext.rs` 이식 완료
9. 기타 유틸리티 및 윈도우 시스템

---

## 핵심 기술적 특징

- **ECS 기반**: `legion` 라이브러리를 사용한 Entity-Component-System 아키텍처
- **하이브리드 UI**: `egui` (프론트 윈도우) + `ratatui` (TUI 렌더링)
- **원본 공식 1:1 이식**: NetHack 3.6.7 C 소스의 계치/공식/분기를 Rust로 완전하게 재구현
- **다국어 지원**: 모든 코드 주석은 한국어, 게임 내 메시지는 다국어 대응
- **Gather-Apply 패턴**: ECS Borrow Checker 충돌을 방지하는 수집-적용 분리 아키텍처
- **AppState 상태 머신**: Title → CharCreation → Playing → GameOver → Title 전환
- **Enum 기반 타입 안전**: MonsterKind/ItemKind enum + get_by_kind() API로 문자열 비교 최소화
- **이벤트 큐**: GameEvent 20+ variant + EventQueue + EventHistory 링 버퍼 (200건)

---

## 현재 다음 작업

- muse.rs 확장 이식: `m_dowear`, `meatmetal` 연동 (원본 37.8%)
- zap.rs 확장 이식: `buzz`, `bhitpile`, `effect`, `weffects` (원본 39.5%)
- 핵심 시스템 파일의 지속 확장 (이식률 64.5%, 60% 돌파 완료)
- 테스트 커버리지 증가 (현재 2,186개)

### v2.19.0 이식 완료 모듈 (6개 신규 _ext 모듈 + 1개 확장)
- **uhitm_ext.rs**: uhitm.c 핵심 이식 — 녹슬기/기사도/마상창/그림자/변신공격 데미지 (10함수, 32테스트)
- **invent_ext.rs**: invent.c 핵심 이식 — 전리품 분류/정렬/병합/특수아이템/무게 (7함수, 23테스트)
- **shk_ext.rs**: shk.c 핵심 이식 — 가격계산/크레딧/청구서/도둑질/분노/전문품목 (10함수, 31테스트)
- **trap_ext.rs 확장**: trap.c 추가 — 마법함정/거미줄/지뢰/바위/변이/레벨텔레/탈것 (+540줄, 25테스트)
- **mhitu_ext.rs**: mhitu.c 핵심 이식 — 공격대체/사전조건/악마소환/수인/삼킴/패시브 (9함수, 33테스트)
- **apply_ext.rs**: apply.c 핵심 이식 — 카메라/수건/점프/통조림/채찍/장창/갈고리 (10함수, 32테스트)
- **do_wear_ext.rs**: do_wear.c 핵심 이식 — 착용판정/방어구효과/반지15종/목걸이9종 (6함수, 16테스트)

### v2.10.1 이식 완료 모듈 (8개 신규 _ext 모듈)
- **lock_ext.rs**: lock.c 핵심 이식 — 자물쇠 따기/부수기/상자/문 (12함수, 18테스트)
- **steal_ext.rs**: steal.c 핵심 이식 — 도적 절도/골드/장비 타겟팅 (8함수, 12테스트)
- **light_ext.rs**: light.c 완전 이식 — 광원 관리/이동/분할/병합/범위 (15함수, 14테스트)
- **bones_ext.rs**: bones.c 완전 이식 — 유골 파일 생성/로드/정화 (10함수, 19테스트)
- **were_ext.rs**: were.c 완전 이식 — 수인 변신/확률/소환/치유 (10함수, 9테스트)
- **rip_ext.rs**: rip.c 완전 이식 — 묘비 ASCII 아트/중앙배치/사인분할 (5함수, 8테스트)
- **write_ext.rs**: write.c 완전 이식 — 마법 마커 작성/비용/판정 (7함수, 12테스트)
- **minion_ext.rs**: minion.c 완전 이식 — 악마 협상/소환/수호천사 (10함수, 12테스트)

### v2.9.6~v2.10.0 이식 완료 모듈 (이전 세션)
- **sit.rs**: take_gold/rndcurse/attrcurse 전량 이식 (139.4%)
- **fountain.rs**: 분수/싱크대/담그기/고갈 전량 이식 (132.4%)
- **rng.rs**: rnl 추가로 rnd.c 100% 완료 (114.6%)
- **mkroom.rs**: courtmon/squadmon/morguemon/cmap_to_type 등 13함수 (113.6%)
- **wizard.rs**: 전략/전술/소환/특수이벤트 14함수 (141.3%)
- **dog.rs**: 3차 이식 완결 (99%+)

---

## AI 모듈 구조 노트 (컨텍스트 인수인계용)

> ⚠️ 이 섹션은 잡업 인수인계 시 반드시 참조해야 합니다.

### 파일 구조 (`src/core/systems/ai/`)

| 파일 | 라인 | 상태 | 설명 |
|------|------|------|------|
| `mod.rs` | 215 | ✅ 사용 | Behavior/Conversable 트레이트, RuleBasedAi/PetAi 기본 구현 |
| `core.rs` | 675 | ✅ 사용 | **ECS 시스템 함수** — `monster_ai`, `pet_hunger`, `move_towards`, `move_random`, `move_away` (SubWorld 직접 조작) |
| `dog.rs` | 2,140 | ✅ 사용 | **결과 구조체 패턴** — Phase1(PetContext/행동결정) + Phase2(InitEdog/DogFoodInput/Hunger/Tame/Wary/Abuse/Catchup/ScoreTarget) |
| `monmove.rs` | 2,494 | ✅ 사용 | **결과 구조체 패턴** — MoveResult/MonsterGoal/SpecialMovement + mfndpos 확장 + 대규모 이식 |
| `mcastu.rs` | 1,424 | ✅ 사용 | **결과 구조체 패턴** — castmu/buzzmu 마법 시전 |
| `muse.rs` | 1,154 | ✅ 사용 | **결과 구조체 패턴** — DefenseUse/OffenseUse/MiscUse, find_defensive/use_defensive |
| `wizard.rs` | 884 | ✅ 사용 | **결과 구조체 패턴** — WizardState/WizardAttack/WizardActionResult + WantsFlag/StrategyGoal/TacticsResult 14함수 |
| `ai_helper.rs` | ~240 | ✅ 사용 | AiHelper::mfndpos, MoveFlags — monmove 등에서 사용 |
| `ai_part1.rs` | 38 | ⚠️ 미사용 | mod.rs에서 주석 처리됨. 기존 분리 시도 잔존. 내용은 core.rs에 병합 완료 |
| `ai_part2.rs` | 318 | ⚠️ 미사용 | mod.rs에서 주석 처리됨. 기존 분리 시도 잔존. 내용은 core.rs에 병합 완료 |
| `ai_part3.rs` | 174 | ⚠️ 미사용 | mod.rs에서 주석 처리됨. 기존 분리 시도 잔존. 내용은 core.rs에 병합 완료 |

### 아키텍처 패턴

1. **ECS 시스템 계층 (core.rs)**: `#[system]` 매크로 기반 legion ECS 함수. SubWorld를 직접 조작. 게임 루프에서 호출.
2. **결과 구조체 패턴 (dog.rs/monmove.rs/muse.rs 등)**: `XxxResult` 구조체 + `xxx_result()` 순수 함수. ECS 없이 독립 테스트 가능. 원본 C 로직의 1:1 변환.
3. **Behavior 트레이트 (mod.rs)**: `decide(&self, obs: &Observation) -> AiAction` — 향후 교체 가능한 AI 전략 패턴.

### dog.rs 내부 구조 (Phase1 + Phase2)

- **Phase1 (L1-735)**: PetBehavior enum, PetContext, dog_move, determine_behavior, follow_owner, guard_owner, hunt_target, flee_from_enemies, classify_food(DogFood enum), try_tame, abuse_pet, feed_pet, pet_turn_update, christen_pet, pet_follows_player, PetStatistics 등
- **Phase2 (L736-2140)**: InitEdogResult, DogFoodInput/FoodItemInfo/dogfood_extended(확장판 21종), DogNutritionResult, DogEatResult, HungerResult, TameDogResult, WaryDogResult, AbuseDogResult, CatchupResult, could_reach_item, can_reach_location_simple, score_target
- **테스트**: tests(3개) + dog_extended_tests(7개) + dog_phase2_tests(31개) = 총 41개

### dog.rs 미이식 원본 함수 (잔여 ~15%)

| 원본 함수 | 원본 위치 | 우선순위 | 설명 |
|-----------|-----------|----------|------|
| `dog_invent` | dogmove.c L82-138 | 높음 | 펫 인벤토리 관리 (아이템 드롭/교환) |
| `dog_goal` | dogmove.c L399-520 | 높음 | 이동 목표 결정 (음식/적/주인 우선순위) |
| `best_target` | dogmove.c L809-1066 | 중간 | 최적 전투 대상 선정 (score_target 활용) |
| `dog_move` (전체판) | dogmove.c L1068-1267 | 높음 | 메인 이동 루프 (mfndpos 연동) |
| `can_reach_location` (전체판) | dogmove.c L1288-1321 | 낮음 | 완전한 경로 탐색 (현재 간소화 버전 존재) |
| `pet_type` | dog.c L9-30 | 낮음 | 시작 펫 종류 결정 |
| `find_omon` | dog.c L56-95 | 낮음 | 같은 종 위치 검색 |
| `update_mon_intrinsics` | dog.c L427-461 | 중간 | 시체 섭취 내성 부여 |

---

**문서 버전**: v2.19.0
**최종 업데이트**: 2026-02-21
