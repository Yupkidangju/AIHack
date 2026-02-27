# 🎮 로그라이크 알고리즘 레퍼런스 (ROGUELIKE_LOGIC_REFERENCE)

**프로젝트**: AIHack v2.41.0
**원본**: NetHack 3.6.7 (177,232줄 C → 177,229줄 Rust)
**아키텍처**: Pure Result Pattern + ECS (Legion)
**테스트**: 4,177개 전량 통과

> 이 문서는 AIHack 코드베이스를 **"범용 로그라이크 알고리즘 라이브러리"**로 재사용하기 위한 기능 맵(Feature Map)이다.
> 모든 순수 알고리즘은 ECS 의존 없이 독립 호출 가능하며, 다른 게임 프로젝트에서 함수 단위로 추출하여 사용할 수 있다.

---

## 목차

1. [시야 및 맵 탐색 (Vision & Exploration)](#1-시야-및-맵-탐색)
2. [절차적 맵 생성 (Procedural Generation)](#2-절차적-맵-생성)
3. [전투 및 데미지 공식 (Combat Mechanics)](#3-전투-및-데미지-공식)
4. [몬스터 AI 및 경로 탐색 (AI & Pathfinding)](#4-몬스터-ai-및-경로-탐색)
5. [아이템 및 인벤토리 관리 (Item & Inventory)](#5-아이템-및-인벤토리-관리)
6. [상태 변화 및 진화 (Status & Evolution)](#6-상태-변화-및-진화)
7. [사회적 상호작용 (Social Interactions)](#7-사회적-상호작용)
8. [마법 시스템 (Magic & Spells)](#8-마법-시스템)
9. [생물/캐릭터 시스템 (Creature & Character)](#9-생물캐릭터-시스템)
10. [던전 구조 및 환경 (Dungeon & Environment)](#10-던전-구조-및-환경)
11. [유틸리티 (Utility)](#11-유틸리티)
12. [재사용성 가이드 (How to Reuse)](#12-재사용성-가이드)

---

## 1. 시야 및 맵 탐색

> 📁 `src/core/systems/world/` — 시야, 감지, 탐색 관련

### 핵심 알고리즘

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| `vision.rs` | `VisionSystem::recalc()` | **FOV(Field of View) 계산** — Bresenham LOS 기반, 반경 내 가시 타일 계산 |
| `vision.rs` | `VisionSystem::has_line_of_sight()` | **시선 차단 판정** — 두 좌표 사이 직선상 장애물 체크 (Bresenham 알고리즘) |
| `vision.rs` | `detect_monster()` | **몬스터 감지 우선순위** — 일반시야 > 투명감지 > 적외선 > 텔레파시 > 경고 |
| `vision.rs` | `effective_vision_radius()` | **유효 시야 반경** — 어둠/야시경/광원 보정 계산 |
| `vision.rs` | `VisionSystem::clairvoyance()` | **천리안** — 범위 내 모든 타일 즉시 공개 |
| `vision.rs` | `VisionSystem::infravision_detect()` | **적외선 감지** — 거리 내 온혈 몬스터 탐지 |
| `vision.rs` | `VisionSystem::telepathy_detect()` | **텔레파시** — 마음이 있는 몬스터 원거리 감지 |
| `vision_ext.rs` | 확장 시야 유틸리티 | FOV 엣지 케이스 처리 |
| `vision_phase3_ext.rs` | 고급 시야 통합 | 조명/어둠 레벨과 시야 통합 |
| `vision_phase96_ext.rs` | 시야 확장 계산 | 추가 시야 모디파이어 |
| `vision_system.rs` | `vision_update_system()` | ECS 시스템 래퍼 (Legion schedule 등록용) |
| `detect.rs` | 탐지 시스템 기반 | 함정/몬스터/아이템 탐지 기본 |
| `detect_ext.rs` | 고급 탐지 | 마법 탐지, 저주 탐지 |
| `detect_phase91_ext.rs` | 탐지 확장 | 추가 탐지 메커니즘 |
| `search.rs` | 수색 판정 | 비밀문/함정 수색 확률 |
| `trap_detect_ext.rs` | 함정 탐지 | 함정 종류별 탐지 확률 계산 |
| `lighting_phase102_ext.rs` | 조명 시스템 | 광원 범위, 영구/임시 조명 |

### 재사용 포인트
- `has_line_of_sight()` — **어떤 2D 타일 기반 게임에서든** 시선 차단 판정에 사용 가능
- `detect_monster()` — 감지 우선순위 체계가 깔끔한 match 기반, 쉽게 커스터마이징
- `effective_vision_radius()` — 순수 함수, 시야 반경 보정 공식만 추출 가능

---

## 2. 절차적 맵 생성

> 📁 `src/core/dungeon/` — 던전 레벨, 방, 미로, 특수 레벨 생성

### 핵심 알고리즘

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| `gen.rs` | 레벨 생성기 | **메인 던전 생성** — 방 배치 + 복도 연결 + 문/계단 배치 |
| `mkroom.rs` | `courtmon()`, `squadmon()` | **특수 방 몬스터** — 보병대/근위실/묘지/오크소굴 편성 확률 테이블 |
| `mkroom_ext.rs` | 방 유형 생성기 | 상점/사원/동물원/묘지 등 특수 방 생성 |
| `mkroom_phase95_ext.rs` | 방 생성 확장 | 추가 특수 방 타입, 방 내 아이템/몬스터 배치 |
| `mklev_ext.rs` | 레벨 레이아웃 | 층 전체 레이아웃 결정 (방 수, 복도 패턴) |
| `mklev_ext2.rs` | 레벨 생성 확장 | 추가 레벨 생성 알고리즘 |
| `mklev_phase93_ext.rs` | 레벨 생성 Phase 93 | 세부 타일 배치 알고리즘 |
| `mkmaze_ext.rs` | **미로 생성** | 미로형 레벨 생성 알고리즘 (게헨나, 미노타우로스 미궁) |
| `sp_lev.rs` | 특수 레벨 파서 | `.des` 파일 기반 특수 레벨 정의 |
| `sp_lev_phase95_ext.rs` | 특수 레벨 확장 | 추가 특수 레벨 구현 |
| `special_level_phase103_ext.rs` | 특수 레벨 데이터 | 메두사/오라클/빅룸 등 특수 레벨 메타데이터 |
| `rect.rs` | `Rect` 구조체 | **사각형 교차/포함 판정** — 방 vs 방 겹침 검사 |
| `tile.rs` | `TileType` (43종) | 타일 타입 정의 (벽/바닥/문/계단/물/용암 등) |
| `dungeon.rs` | `DungeonManager` | 던전 분기(Branches) 관리 — 게헨나/미네스/소반 등 |
| `dungeon_ext.rs` | 던전 확장 | 던전 깊이 계산, 분기 접근 조건 |
| `branch_phase98_ext.rs` | 던전 분기 확장 | 분기 접근 가능성/난이도 계산 |
| `mapgen_phase100_ext.rs` | 맵 생성 통합 | 단계별 맵 생성 파이프라인 |
| `stairs_ext.rs` | 계단 배치 | 상/하행 계단 배치 알고리즘 |
| `boundary_ext.rs` | 경계 처리 | 레벨 경계 벽 생성 |

### 재사용 포인트
- `gen.rs`의 방+복도 생성 — 어떤 타일 기반 던전크롤러에서든 기본 레벨 생성기로 활용
- `mkmaze_ext.rs` — 미로 생성 전용 알고리즘
- `mkroom.rs`의 확률 테이블 기반 몬스터 편성 — 난이도별 적 배치 시스템

---

## 3. 전투 및 데미지 공식

> 📁 `src/core/systems/combat/` — 명중/회피/데미지/특수공격 전체

### 핵심 알고리즘

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| **combat_formula_phase95_ext.rs** | `attack_roll()` | **명중 판정 공식** — 공격력 vs AC, d20 롤, 펌블/치명타 판정 |
| | `calculate_damage()` | **데미지 계산** — 기본 다이스 + 무기보너스 + 강화치 + 속성배율 |
| | `counter_check()` | **반격/반사 판정** — 반사/가시/회피/패리 4종 |
| | `calculate_exp_gain()` | **경험치 공식** — 몬스터 레벨/HP vs 플레이어 레벨 |
| `engine.rs` | `CombatEngine` | **전투 엔진 코어** — 플레이어→몬스터 공격 총괄 |
| `uhitm.rs` | 플레이어→몬스터 | 근접 공격 처리 (hit/miss/kill), 무기 효과 |
| `uhitm_ext.rs` | 확장 공격 | 다중 공격, 양손 공격, 기회 공격 |
| `uhitm_phase90_ext.rs` | 고급 공격 | 추가 명중 보정, 방어 관통 |
| `mhitu.rs` | 몬스터→플레이어 | **특수 공격 16종** — 절도/질병/라이칸스로피/녹/삼키기/마비/석화 등 |
| `mhitu_ext.rs` | 몬스터 공격 확장 | 드레인/질식/폭발 추가 공격 |
| `mhitu_phase90_ext.rs` | 고급 몬스터 공격 | 추가 특수 공격 확장 |
| `mhitm.rs` | 몬스터↔몬스터 | 몬스터 간 전투 판정 |
| `mhitm_ext.rs` | 몬스터 간 확장 | 추가 몬스터 간 상호작용 |
| `mhitm_phase91_ext.rs` | 고급 몬스터 전투 | 확장 몬스터 간 전투 공식 |
| `weapon.rs` | 무기 데이터 | **무기 보너스 테이블** — 대소형 데미지, 숙련도, 재질 |
| `weapon_ext.rs` | 무기 확장 | 무기 특수효과, 은 무기, 축복 무기 |
| `weapon_class_ext.rs` | 무기 분류 | 무기 카테고리(검/둔기/폴암/활 등) 분류 체계 |
| `hit_calc_ext.rs` | 명중률 세부 | AC 보정, DEX 보너스, 장비 보정 세부 계산 |
| `throw.rs` | 투척 기본 | 투척물 궤적 계산, 명중/빗나감 |
| `dothrow_ext.rs` | 투척 확장 | 투척 보너스, 부메랑 귀환, 보석 수락 |
| `dothrow_phase92_ext.rs` | 투척 고급 | 다중 투척, 투척 숙련도 |
| `mthrowu_ext.rs` | 몬스터 투척 | 몬스터의 아이템 투척 AI |
| `kick.rs` | 발차기 기본 | 문/상자/몬스터 차기 데미지 |
| `dokick_ext.rs` | 발차기 확장 | 벽 차기 자해, 강화 차기 |
| `explode.rs` | 폭발 기본 | 폭발 범위 내 데미지 분배 |
| `explode_ext.rs` | 폭발 확장 | 원소 폭발, 연쇄 폭발 |
| `area_attack_ext.rs` | 범위 공격 | 브레스, 광역 주문 범위 판정 |
| `elemental_ext.rs` | 속성 상성 | **원소 데미지 상성** — 화/냉/전/독/산 저항/약점 |
| `artifact_combat_ext.rs` | 아티팩트 전투 | 아티팩트 무기 특수 효과 (파멸/축복/흡수 등) |
| `artifact_ext.rs` | 아티팩트 시스템 | 아티팩트 인식/기능/소유권 |
| `artifact_table_ext.rs` | 아티팩트 테이블 | 전체 아티팩트 데이터 |
| `artifact_phase99_ext.rs` | 아티팩트 통합 | 최종 아티팩트 시스템 |
| `music_combat_ext.rs` | 음악 전투 | 오르페우스 류트 등 악기 전투 효과 |
| `theft_ext.rs` | 절도 판정 | 님프/원숭이 아이템 탈취 확률 |
| `skill_phase104_ext.rs` | **스킬/숙련도** | 6단계 숙련도, 16카테고리, 역할별 제한 |
| `destroy_phase98_ext.rs` | 장비 파괴 | 산/화염에 의한 장비 파괴 확률 |
| `do_wear_phase90_ext.rs` | 장비 전투 보정 | 착용 장비의 전투 보너스 |
| `mcastu_phase91_ext.rs` | 몬스터 마법 전투 | 몬스터의 전투 주문 사용 |
| `final_combat_ext.rs` | **투사체 궤적** | Bresenham 기반 투사체 경로, 부위별 방어, 치명타, 무기 내구도 |

### 재사용 포인트
- `combat_formula_phase95_ext.rs` — **완전히 독립적인 전투 공식 모듈**. 입력 구조체→결과 enum 패턴으로 어떤 게임에든 이식 가능
- `elemental_ext.rs` — 원소 상성 시스템을 독립적으로 추출 가능
- `mhitu.rs`의 특수 공격 16종 — 로그라이크 특유의 상태이상 공격 패턴 레퍼런스

---

## 4. 몬스터 AI 및 경로 탐색

> 📁 `src/core/systems/ai/` — 몬스터 행동 결정, 펫 AI, 마법 사용

### 핵심 알고리즘

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| `core.rs` | `monster_ai_system()` | **메인 AI 루프** — 매 턴 모든 몬스터의 행동 결정 |
| `monmove.rs` | `m_move()` | **몬스터 이동 판정** — 추적/도주/방황/순찰 4모드 |
| `monmove_ext.rs` | 이동 확장 | 지형 통과(비행/수영/터널링), 문 열기/부수기 |
| `monmove_phase94_ext.rs` | 고급 이동 | 탈출 경로 탐색, 아이템 탐지 이동 |
| `muse.rs` | `find_offensive/defensive/misc()` | **몬스터 아이템 사용 AI** — 공격/방어/기타 아이템 선택 |
| `muse_ext.rs` | 아이템 사용 확장 | 포션 투척, 완드 사용, 두루마리 읽기 판단 |
| `mcastu.rs` | 몬스터 주문 기본 | **마법 사용 판정** — Mage vs Cleric 주문군 분리 |
| `mcastu_ext.rs` | 주문 확장 | 주문 성공률, 주문 선택 우선순위 |
| `mcastu_phase96_ext.rs` | 고급 주문 | 추가 몬스터 주문, 쿨다운 시스템 |
| `dog.rs` | 펫 AI 기본 | **펫 행동** — 먹기/따라가기/아이템 줍기 |
| `dog_ext.rs` | 펫 확장 | 펫 배고픔, 만족도, 충성도 계산 |
| `dogmove_ext.rs` | 펫 이동 | 주인 추적, 도움 공격, 위험 회피 |
| `pet_ai_ext.rs` | 펫 AI 통합 | 펫 행동 결정 트리 확장 |
| `wizard.rs` | **위저드 AI** | 중간보스(Wizard of Yendor) 특수 AI — 텔레포트/소환/도주 |
| `ai_brain_ext.rs` | AI 두뇌 | 전술 판단 레이어 |
| `ai_helper.rs` | AI 헬퍼 | 몬스터 능력/상태 확인 유틸리티 |
| `ai_tactic_ext.rs` | 전술 판단 | 공격/도주/대기 전술 선택 |
| `track_ext.rs` | 추적 시스템 | 냄새 추적, 소리 추적 알고리즘 |
| `strategy_phase101_ext.rs` | 전략 계층 | 상위 전략 판단 (목표 설정) |

### 경로 탐색 (별도)
| 파일 | 알고리즘 |
|------|---------|
| `src/util/path.rs` | **A* 경로 탐색** — 2D 그리드 기반, 장애물 회피 |

### 재사용 포인트
- `monmove.rs`의 4모드 AI — 추적/도주/방황/순찰 패턴이 대부분의 로그라이크에 적용 가능
- `muse.rs`의 아이템 사용 판단 — "몬스터가 아이템을 지능적으로 사용"하는 로직
- `path.rs`의 A* — 범용 2D 경로 탐색

---

## 5. 아이템 및 인벤토리 관리

> 📁 `src/core/systems/item/` — 아이템 사용/제작/관리 전체

### 핵심 알고리즘

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| `item_use.rs` | `item_use_system()` | **아이템 사용 총괄** — 종류별 효과 분기 |
| `potion.rs` | 포션 시스템 | 포션 타입별 효과 (치유/독/변이/투명 등) |
| `potion_ext.rs` | 포션 확장 | 포션 투척, 스플래시 피해 |
| `potion_mix_ext.rs` | **포션 혼합** | 두 포션 조합 → 새 포션 생성 (화학 반응 시스템) |
| `potion_quaff_ext.rs` | 포션 음용 | 음용 효과 상세 |
| `potion_phase92_ext.rs` | 포션 확장 | 추가 포션 효과 |
| `read.rs` | 두루마리 | 두루마리 읽기 효과 (텔레포트/식별/마법지도 등) |
| `read_ext.rs` | 두루마리 확장 | 추가 두루마리 효과 |
| `read_phase91_ext.rs` | 두루마리 고급 | 축복/저주시 효과 변동 |
| `scroll_effect_ext.rs` | 스크롤 효과 전산 | 효과별 순수 결과 계산 |
| `zap.rs` | 완드 시스템 | **완드 발사** — 16종 즉시 효과, 빔/볼트/광선 |
| `zap_ext.rs` | 완드 확장 | 반사, 흡수, 면역 처리 |
| `apply.rs` | 도구 사용 | 곡괭이/호루라기/열쇠/거울 등 도구 적용 |
| `apply_ext.rs` | 도구 확장 | 추가 도구 효과 |
| `apply_phase93_ext.rs` | 도구 고급 | 확장 도구 사용 |
| `eat.rs` | **음식 섭취** | 영양값 계산, 시체 효과 (저항 획득/중독/알레르기) |
| `eat_ext.rs` | 음식 확장 | 채식주의, 식인, 통조림 |
| `eat_phase90_ext.rs` | 음식 고급 | 소비 속도, 과식 |
| `eat_phase97_ext.rs` | 음식 확장 | 추가 음식 효과 |
| `food_spoil_ext.rs` | **음식 부패** | 시간 경과에 따른 시체 부패 알고리즘 |
| `corpse_ext.rs` | 시체 시스템 | 시체 드롭/영양/저항 획득 판정 |
| `pickup.rs` | 아이템 줍기 | 자동 줍기, 무게 체크 |
| `pickup_ext.rs` | 줍기 확장 | 자동줍기 필터, 더미 합치기 |
| `weight.rs` | **무게 계산** | 아이템별 무게, 하중 단계 (5단계 부하) |
| `container_ext.rs` | **컨테이너** | 가방/상자 재귀 탐색, Bag of Holding 무게 공식 |
| `loot.rs` | 전리품 | 드롭 테이블, 전리품 생성 |
| `mkobj.rs` | 아이템 생성 | 아이템 생성기 — 종류/난이도/BUC 결정 |
| `mkobj_ext.rs` | 아이템 생성 확장 | 추가 생성 규칙 |
| `objnam.rs` | 아이템 명명 | 아이템 이름 포맷팅 (미식별/저주/강화 반영) |
| `objnam_ext.rs` | 명명 확장 | 복수형, 관사, 재질명 |
| `objnam_phase96_ext.rs` | 명명 고급 | 추가 명명 규칙 |
| `identify_ext.rs` | **식별 시스템** | 아이템 식별 상태 (미식별/시도/완전식별) |
| `buc_phase101_ext.rs` | **BUC 시스템** | 축복(Blessed)/무저주(Uncursed)/저주(Cursed) 판정 |
| `buc_spread_ext.rs` | BUC 전파 | 인접 아이템에 BUC 상태 전파 |
| `curse_system_ext.rs` | 저주 시스템 | 저주 해제/적용 조건 |
| `gem_ext.rs` | 보석 시스템 | 보석 감정, 가치 판정 |
| `wish_ext.rs` | **소원 시스템** | 소원으로 아이템 생성 시 파싱/제한 규칙 |
| `write_ext.rs` | 두루마리 작성 | 빈 두루마리에 주문 작성 |
| `write_scroll_ext.rs` | 작성 확장 | 작성 성공률, 잉크 소비 |
| `o_init_ext.rs` | 아이템 초기화 | 아이템 외관 랜덤화 (포션 색/두루마리 라벨) |
| `item_damage.rs` | 아이템 피해 | 산/화재/전기에 의한 아이템 파괴 |
| `item_tick.rs` | 아이템 틱 | 매 턴 아이템 상태 갱신 (타이머/부식/충전) |
| `invent_sort_ext.rs` | 인벤토리 정렬 | 아이템 정렬 알고리즘 (카테고리/이름/무게) |

### 재사용 포인트
- `container_ext.rs` — Bag of Holding의 재귀 무게 공식은 로그라이크의 고전적 알고리즘
- `identify_ext.rs` + `o_init_ext.rs` — 아이템 미식별/외관 랜덤화 시스템 전체를 독립 사용 가능
- `potion_mix_ext.rs` — 포션 혼합 조합 테이블
- `eat.rs` — 시체 식사를 통한 저항 획득 시스템

---

## 6. 상태 변화 및 진화

> 📁 `src/core/systems/creature/` — 상태이상, 변이, 능력치, 배고픔

### 핵심 알고리즘

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| `status.rs` | `StatusBundle` | **상태 관리** — 41종 상태이상 (u64 비트필드), 시한부 틱 기반 해소 |
| `status_phase98_ext.rs` | 상태 확장 | 추가 상태 이상 메커니즘 |
| `status_timer_ext.rs` | 상태 타이머 | 시한부 상태 틱 처리 |
| `evolution.rs` | **폴리모프 엔진** | 변이 시스템 — 몬스터 형태 변환, 능력 변화, 장비 해제 |
| `polymorph_ext.rs` | 변이 확장 | 변이 지속시간, 역변이 |
| `polymorph_rule_ext.rs` | **변이 규칙** | 변이 가능 대상, 불가 대상, 변이 제한 |
| `polymorph_phase94_ext.rs` | 고급 변이 | 추가 변이 효과 |
| `polyself_ext.rs` | 자기 변이 | 플레이어의 자발적 변이 |
| `were_ext.rs` | **라이칸스로피** | 늑대인간 변신 — 만월 트리거, 변신/역변신 |
| `hunger_ext.rs` | **배고픔** | 6단계 배고픔 (포만→배부름→정상→공복→약함→실신→기아) |
| `nutrition_phase104_ext.rs` | 영양 확장 | 영양소비율, 채식주의, 독/과식 |
| `attrib.rs` | 능력치 기본 | STR/DEX/CON/INT/WIS/CHA 6대 능력치 |
| `attrib_ext.rs` | 능력치 확장 | 능력치 변동, exercise 시스템 |
| `attrib_ext2.rs` | 능력치 고급 | 능력치 최대/최소, 역할별 보정 |
| `attribute_phase103_ext.rs` | 능력치 통합 | 최종 능력치 통합 시스템 |
| `stat_change_ext.rs` | 스탯 변화 | 능력치 일시적/영구적 변화 |
| `prop_calc_ext.rs` | 속성 계산 | 장비/상태에 의한 속성 합산 |
| `resist_calc_ext.rs` | **저항 계산** | 원소/마법/상태 저항력 합산 (장비+내재+일시) |
| `regeneration.rs` | HP/MP 회복 | 턴 기반 자연 회복, CON 보정 |
| `ecology_phase102_ext.rs` | 생태계 | 몬스터 먹이사슬, 공생, 포식 관계 |

### 재사용 포인트
- `StatusBundle` — 비트필드 기반 상태 관리, 시한부 자동 해소가 포함된 완성형 시스템
- `evolution.rs` — 폴리모프 시스템 전체를 독립 모듈로 추출 가능
- `hunger_ext.rs` — 6단계 배고픔 상태머신, 생존 게임에 그대로 적용 가능
- `resist_calc_ext.rs` — 다중 소스(장비/내재/일시) 저항력 합산 공식

---

## 7. 사회적 상호작용

> 📁 `src/core/systems/social/` — 상점, 종교, 퀘스트, 도둑질

### 핵심 알고리즘

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| `shop.rs` | 상점 기본 | **가격 계산** — 카리스마 보정, 관광세, 도난 감지 |
| `shop_ext.rs` | 상점 확장 | 상점 재고 생성, 가격 흥정 |
| `shop_stock_ext.rs` | 재고 관리 | 상점 종류별 재고 테이블 (무기점/방어구점/서점/포션점 등) |
| `shk_ext.rs` | 상인 AI | 상인 행동 — 추적, 분노, 가격 청구 |
| `shk_ai_ext.rs` | 상인 AI 확장 | 상인의 전투/도주 판단 |
| `shk_price_ext.rs` | **가격 공식** | 아이템 가격 결정 — 기본가×카리스마 보정×상점 배율 |
| `shk_phase3_ext.rs` | 상인 통합 | 상인 시스템 통합 |
| `shk_phase93_ext.rs` | 상인 고급 | 추가 상인 행동 |
| `economy_phase99_ext.rs` | 경제 시스템 | 금화 관리, 거래 내역 |
| `pray.rs` | **기도 시스템** | 기도 시스템 — 신앙심/쿨다운/축복/저주해제/분노 |
| `pray_ext.rs` | 기도 확장 | 기도 효과 상세, 제물 바치기 |
| `pray_phase91_ext.rs` | 기도 고급 | 추가 기도 효과 |
| `prayer_calc_ext.rs` | **기도 계산** | 기도 성공률/효과 판정 공식 |
| `religion_phase104_ext.rs` | 종교 시스템 | 3성향(Lawful/Neutral/Chaotic), 5단계 신과의 관계 |
| `priest_ext.rs` | 사제 시스템 | 사제 행동, 성수 제작 |
| `priest_temple_ext.rs` | 사원 시스템 | 제단 기능, 봉헌 |
| `altar_phase94_ext.rs` | 제단 확장 | 제단 위 아이템 효과, 제단 변환 |
| `alignment_ext.rs` | 성향 시스템 | 성향 변동, 행위별 보정 |
| `steal.rs` | 도둑질 기본 | 님프/원숭이의 아이템 탈취 |
| `steal_ext.rs` | 도둑질 확장 | 탈취 확률, 탈취 대상 선택 |
| `steal_phase91_ext.rs` | 도둑질 고급 | 추가 도둑질 메커니즘 |
| `quest_ext.rs` | 퀘스트 기본 | 퀘스트 진행 조건, 목표 확인 |
| `quest_branch_ext.rs` | 퀘스트 분기 | 역할별 퀘스트 분기 |
| `questpgr_ext.rs` | 퀘스트 대화 | 퀘스트 NPC 대화 스크립트 |
| `talk.rs` | 대화 시스템 | 몬스터 대화 판정 |
| `vault_ext.rs` | 금고 시스템 | 금고 경비원 AI |
| `minion_ext.rs` | 시종 시스템 | 신의 시종 소환/행동 |
| `interaction.rs` | InteractionProvider | **텍스트 생성 추상화** — LLM 교체 가능 인터페이스 |

### 재사용 포인트
- `shk_price_ext.rs` — 상점 가격 공식 (카리스마 보정)을 그대로 다른 RPG에 적용
- `prayer_calc_ext.rs` — 신앙/기도 시스템의 수학적 모델
- `interaction.rs` — LLM 교체 가능한 텍스트 인터페이스 패턴

---

## 8. 마법 시스템

> 📁 `src/core/systems/magic/` — 주문, 완드, 감지, 텔레포트

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| `spell_ext.rs` | 주문 시전 | **주문 성공률** — 레벨+지능+숙련도 vs 주문 난이도 |
| `spell_ext2.rs` | 주문 확장 | 주문 망각, 주문 학습 |
| `spell_school_ext.rs` | **주문 학파** | 7학파 분류 (공격/치유/점지/탈출/물질/부여/차원) |
| `spell_phase93_ext.rs` | 고급 주문 | 추가 주문 메커니즘 |
| `spellbook_phase102_ext.rs` | 마법서 | 마법서 읽기/학습/오독(뮤트) |
| `wand_effect_ext.rs` | 완드 효과 | 완드 종류별 효과 (죽음/염동력/화염볼 등) |
| `wand_phase97_ext.rs` | 완드 확장 | 충전, 폭발, 반사 |
| `recharge_ext.rs` | **재충전** | 완드 재충전 성공/과충전 폭발 확률 |
| `teleport_ext.rs` | 텔레포트 | 텔레포트 목적지 결정, 레벨 텔레포트 |
| `detect_ext.rs` | 마법 감지 | 몬스터/아이템/함정 마법 감지 |
| `detect_map_ext.rs` | 감지 확장 | 마법 지도, 물체 감지 |
| `read_phase97_ext.rs` | 두루마리 마법 | 두루마리-마법 통합 |

---

## 9. 생물/캐릭터 시스템

> 📁 `src/core/systems/creature/` + `spawn/` + `misc/`

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| `movement.rs` | **이동 엔진** | 8방향 이동, 지형 통과, 얼음 미끄러짐, 수중 이동 |
| `do_wear.rs` | 장비 착탈 | **장비 슬롯 관리** — 12부위 (투구/갑옷/방패/장갑/부츠/망토/셔츠/반지×2/아뮬렛/무기/보조무기) |
| `do_wear_ext.rs` | 장비 확장 | 장비 특수효과, 드래곤 갑옷, 저주된 장비 |
| `equipment.rs` | ECS 장비 | 장비 시스템 ECS 래퍼 |
| `worn.rs` | 착용 상태 | WornSlots 비트필드 (12슬롯) |
| `worn_ext.rs` | 착용 확장 | 착용 제한, 자동 해제 |
| `worn_phase94_ext.rs` | 착용 고급 | 추가 착용 규칙 |
| `death.rs` | **사망 처리** | 사망 원인 기록, 생명구원 판정 |
| `death_check_ext.rs` | 사망 확인 | 사망 조건 세부 확인 |
| `end.rs` | **게임 종료** | 사망/승천/탈출 엔딩, 최종 점수 계산 |
| `end_ext.rs` | 종료 확장 | 업적, 행동강령(conduct) |
| `end_phase93_ext.rs` | 종료 고급 | 추가 엔딩 조건 |
| `exper.rs` | 경험치 | 경험치 획득, 레벨업 판정 |
| `exper_ext.rs` | 경험치 확장 | 경험치 보정, 레벨 드레인 |
| `experience_ext.rs` | 고급 경험치 | 추가 경험치 메커니즘 |
| `levelup_phase101_ext.rs` | 레벨업 | 레벨업 시 HP/MP/스킬 증가, 능력치 상승 |
| `rip_ext.rs` | 묘비 | 사망 시 묘비 생성 |
| `steed_ext.rs` | **기마** | 탈것 시스템 — 기마공격, 기마이동, 안장 |
| `mount_ext.rs` | 기마 확장 | 탈것 행동 |
| `worm_ext.rs` | 벌레/지렁이 | 다분절 몬스터 (긴 벌레) |
| `unique_mon_ext.rs` | 고유 몬스터 | 유일 몬스터 관리 |
| `sounds_ext.rs` | 소리 시스템 | 몬스터 소리, 플레이어 청각 |
| `skill_tree_ext.rs` | 스킬 트리 | 스킬 포인트 배분 |
| `armor_data_ext.rs` | 방어구 데이터 | 방어구 AC/무게/재질 테이블 |
| `armor_enhance_ext.rs` | 방어구 강화 | 방어구 인챈트 |
| `accessory_ext.rs` | 장신구 | 반지/아뮬렛 효과 |
| `wield_ext.rs` | 무기 장착 | 무기 장착/해제, 양손무기 |
| `makemon.rs` | **몬스터 생성** | 난이도 기반 몬스터 선택, 그룹 스폰 |
| `makemon_ext.rs` | 생성 확장 | 추가 생성 규칙 |
| `makemon_phase93_ext.rs` | 생성 고급 | 세부 생성 알고리즘 |
| `spawn_manager.rs` | **스폰 매니저** | 턴 기반 리스폰, 스폰 요청 큐 |
| `spawn_rule_ext.rs` | 스폰 규칙 | 깊이/시간/역할별 스폰 규칙 |
| `summon_ext.rs` | 소환 | 마법 소환, 신의 시종 소환 |
| `mplayer_ext.rs` | 플레이어 몬스터 | 다른 플레이어 유령/분신 |
| `role.rs` | **역할(직업)** | 13종 역할 (전사/마법사/도적/기사/성직자 등) |
| `role_ext.rs` | 역할 확장 | 역할별 능력치/장비/제한 |
| `role_phase92_ext.rs` | 역할 고급 | 역할+종족 조합 보정 |
| `role_selection_ext.rs` | 역할 선택 | 캐릭터 생성 시 역할/종족/성향 선택 |
| `u_init_ext.rs` | 초기 장비 | 역할별 시작 장비/스킬 |

---

## 10. 던전 구조 및 환경

> 📁 `src/core/systems/world/` — 함정, 문, 분수, 좌석, 조각, 환경

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| `trap.rs` | **함정 시스템** | 30종 함정 (낙석/텔레포트/불/구덩이/다트/석화 등) |
| `trap_ext.rs` | 함정 확장 | 함정 효과 상세 |
| `trap_ext2.rs` | 함정 고급 | 추가 함정 유형 |
| `trap_phase91_ext.rs` | 함정 확장 | 추가 확장 |
| `trap_phase96_ext.rs` | 함정 고급 | 고급 함정 로직 |
| `hazard_phase104_ext.rs` | 위험 통합 | 15종 함정 작동/회피/감지/해제 |
| `fountain.rs` | **분수** | 분수 음용/담금 효과 (16+15+15 = 46종 결과) |
| `fountain_ext.rs` | 분수 확장 | 추가 분수 효과 |
| `fountain_effect_ext.rs` | 분수 효과 | 분수 효과 상세 계산 |
| `sit.rs` | 앉기 | 의자/왕좌 효과, 왕좌의 소원 |
| `sit_ext.rs` | 앉기 확장 | 추가 좌석 효과 |
| `sink.rs` | 싱크대 | 싱크대 사용 효과 |
| `dig.rs` | **채굴** | 곡괭이/마법 채굴, 터널링 |
| `dig_ext.rs` | 채굴 확장 | 리코셰, 돌 생성 |
| `dig_calc_ext.rs` | 채굴 계산 | 채굴 성공 확률/시간 |
| `dig_phase50_ext.rs` | 채굴 통합 | 채굴 완성 모듈 |
| `terrain_destroy_ext.rs` | 지형 파괴 | 터널/붕괴/폭발에 의한 지형 변화 |
| `liquid_damage_ext.rs` | 액체 피해 | 용암/물 피해, 익사, 부유 |
| `door_ext.rs` | 문 시스템 | 문 열기/닫기/잠금/부수기 |
| `lock.rs` | 자물쇠 | 열쇠/곡괭이/마법으로 잠금 해제 |
| `lock_ext.rs` | 잠금 확장 | 잠금 해제 확률, 금고 |
| `engrave.rs` | **각인** | 바닥에 글씨 새기기 (Elbereth 보호막) |
| `engrave_ext.rs` | 각인 확장 | 각인 도구별 효과, 마모 |
| `engrave_calc_ext.rs` | 각인 계산 | 각인 지속시간, 재질별 차이 |
| `stairs.rs` | 계단 | 상/하행 계단, 구멍 |
| `dbridge_ext.rs` | 도개교 | 도개교 열기/닫기 |
| `ball_ext.rs` | 철구 | 철구+사슬 족쇄 이동 제한 |
| `bones_ext.rs` | **본즈** | 이전 사망 지점 유령/아이템 로딩 |
| `music_ext.rs` | 음악 | 악기 연주 효과 (오르페우스 류트 등) |
| `region_ext.rs` | 지역 | 가스 구름, 독무 등 영역 효과 |
| `region_phase94_ext.rs` | 지역 확장 | 추가 영역 효과 |
| `environ_phase101_ext.rs` | 환경 통합 | 지형/환경 통합 시스템 |
| `light_ext.rs` | 조명 아이템 | 등불/양초/마법 조명 |
| `light_source_ext.rs` | 광원 | 광원 반경, 연료 소비 |
| `timeout_ext.rs` | 타이머 | 물약/주문/상태 타이머 |
| `timeout_phase91_ext.rs` | 타이머 확장 | 추가 타이머 이벤트 |
| `teleport.rs` | 텔레포트 | 텔레포트 목적지 결정, 불가 구역 |
| `teleport_phase91_ext.rs` | 텔레포트 확장 | 레벨 텔레포트, 제어 텔레포트 |

---

## 11. 유틸리티

> 📁 `src/util/` — 범용 유틸리티

| 파일 | 핵심 함수/구조체 | 알고리즘 설명 |
|------|-----------------|-------------|
| `rng.rs` | `NetHackRng` | **시드 기반 RNG** — d(n,s) 다이스 롤, rn1/rn2/rne/rnl (행운 보정 포함) |
| `rng_ext.rs` | RNG 확장 | 추가 RNG 함수 |
| `path.rs` | `a_star()` | **A* 경로 탐색** — 2D 그리드, 장애물 회피, 이동 비용 |
| `hacklib_ext.rs` | 유틸리티 | 문자열 처리, 방향 계산 |

---

## 12. 재사용성 가이드

### 12.1 아키텍처 패턴: Pure Result Pattern

이 코드베이스의 핵심 설계 원칙:

```rust
// ❌ 부작용 함수 (C 원본 스타일)
fn attack(player: &mut Player, monster: &mut Monster) {
    monster.hp -= damage; // 직접 수정
    player.exp += xp;     // 전역 상태 변경
}

// ✅ 순수 결과 함수 (AIHack 스타일)
fn attack_roll(input: &AttackRollInput, rng: &mut NetHackRng) -> AttackRollResult {
    // 입력만 받아서 결과만 반환. 부작용 없음.
    AttackRollResult::Hit { margin: 5 }
}
```

**장점**: 
- ECS 없이도 함수 단독 호출 가능
- 시드 RNG로 결정론적 테스트 보장
- 어떤 게임 엔진에든 이식 가능 (Unity/Godot/Bevy 등)

### 12.2 독립 추출 가이드

특정 시스템만 추출하여 사용하는 방법:

1. **대상 파일 복사** (예: `combat_formula_phase95_ext.rs`)
2. **의존성 확인** — 대부분 `NetHackRng` (시드 RNG)만 의존
3. **RNG 대체** — 자체 RNG로 교체하거나 `NetHackRng` 함께 복사
4. **입력/출력 구조체 조정** — 필드명/타입을 자체 프로젝트에 맞게 수정

### 12.3 의존성 계층

```
가장 독립적 (바로 추출 가능)
├── combat_formula_phase95_ext.rs (전투 공식)
├── fountain.rs (분수 효과)
├── sit.rs (앉기 효과)
├── rng.rs (RNG)
├── path.rs (A* 경로 탐색)
├── hunger_ext.rs (배고픔)
├── potion_mix_ext.rs (포션 혼합)
├── resist_calc_ext.rs (저항 계산)
└── identify_ext.rs (식별 시스템)

중간 의존성 (2~3개 파일 함께 추출)
├── vision.rs + tile.rs (시야 + 타일)
├── evolution.rs + status.rs (변이 + 상태)
├── shop.rs + shk_price_ext.rs (상점 + 가격)
└── makemon.rs + spawn_rule_ext.rs (몬스터 생성)

높은 의존성 (시스템 전체 필요)
├── game_loop.rs (ECS 통합)
├── app.rs (앱 상태머신)
└── death.rs + end.rs (사망/종료)
```

### 12.4 Gather-Apply 패턴

ECS 환경에서 borrow checker 충돌을 피하는 핵심 패턴:

```rust
// Gather 단계: 데이터 읽기 (불변 참조)
let input = AttackRollInput {
    attacker_level: player.level,
    defender_ac: monster.ac,
    // ...
};

// Apply 단계: 순수 계산 (소유권 문제 없음)
let result = attack_roll(&input, &mut rng);

// Store 단계: 결과 적용 (가변 참조)
match result {
    AttackRollResult::Hit { margin } => monster.hp -= damage,
    AttackRollResult::Miss { .. } => { /* nothing */ },
}
```

### 12.5 Quick Start: "내 게임에 전투 시스템 붙이기"

```
1. 복사: combat_formula_phase95_ext.rs + rng.rs
2. 제거: #[cfg(test)] 블록 (선택)
3. 수정: AttackRollInput 필드를 자체 캐릭터 구조체에 맞게 조정
4. 호출: attack_roll() / calculate_damage() / counter_check()
5. 완료!
```

---

**문서 버전**: v1.0
**작성일**: 2026-02-28
**총 참조 파일**: 438개 (.rs)
**총 참조 줄수**: 177,229줄
