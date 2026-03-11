# AIHack 기술 명세서 (spec.md)

**버전**: v3.0.0-alpha.2
**최종 업데이트**: 2026-03-11
**상태**: 이식률 💯 **100.0%** (177,229 / 177,232 라인) | 4,179개 테스트 통과 | 엔진 전환 30/30 시스템 완료 (100%) | LLM 엔진 통합 (Phase E4)

---

## 1. 프로젝트 정의

### 1.1 목적
NetHack 3.6.7 (C 소스 177,232줄 + 헤더 20,097줄 = 197,329줄)을 Rust로 **100% 이식**하는 프로젝트.
"대략적 구현"이나 "핵심만 이식"은 **절대 금지**된다.

### 1.2 원본 소스
- **NetHack 3.6.7**: Stichting Mathematisch Centrum, Amsterdam, 1985
- **라이선스**: NetHack General Public License (NGPL)
- **소스 파일 수**: 약 90개 C 파일 + 60개 헤더 파일

### 1.3 대상 환경
- **언어**: Rust (1.84+ Stable)
- **ECS**: Legion (데이터 모델만 사용, Schedule/SubWorld 제거 예정 → v3.0.0)
- **TUI**: Ratatui (터미널 기반 렌더링)
- **GUI**: egui/eframe (하이브리드 윈도우 UI)
- **빌드**: Cargo (MSVC Build Tools on Windows)
- **LLM**: llama.cpp b8192 Sidecar (Qwen3-4B Q4_K_M) — Vulkan/CPU, 비동기 generate_async()

### 1.4 엔진 전환 방향 (v3.0.0 — 진행 중)

> **결정 #41 (DESIGN_DECISIONS.md)**: Legion의 `#[system]` 매크로 + SubWorld를 제거하고,
> 모든 시스템을 `fn system_name(ctx: &mut GameContext)` 시그니처의 일반 함수로 전환한다.
> ECS 데이터 모델(Entity/Component/Query)은 100% 유지하며, 실행 모델만 순차 호출로 교체한다.
> 이를 통해 AccessDenied 패닉을 구조적으로 영구 제거하고, LLM 통합 포인트(AIProvider)를 확보한다.
>
> **현황 (v3.0.0-alpha.2, 2026-03-11)**:
> - Phase E1 ✅ 완료: `GameContext` + `TurnRunner` 구현
> - Phase E2 ✅ 완료: **30/30 시스템 전환 완료 (100%)**
> - Phase E3 ✅ 완료: Panic Hook 강화, dead code 정리, 레거시 테스트 마이그레이션, 1000턴 퍼징 통과
> - Phase E4 ✅ 완료: **LLM 엔진 통합 (smaLLM v0.2.25 이식)**
>   - `src/llm/` 모듈 (785줄): ProcessManager + LlmEngine + AcceleratorInfo
>   - 비동기 API: `generate_async()` → `LlmRequest` 폴링 (턴 블로킹 방지)
>   - 기능 연결: death.rs (AI 묘비명) + stairs.rs (던전 분위기 묘사)
>   - 바이너리: llama.cpp b8192 Vulkan+CPU (84.8MB)
>   - 모델: Qwen3-4B-Instruct-2507-Q4_K_M (2.33GB)
> - GameContext 확장: VisionSystem, LevelChange, Dungeon, **LlmEngine** 필드
> - 빌드 ✅ + 테스트 4,179개 전량 통과
>
> 상세: `STABILIZATION_ROADMAP.md` Phase E1~E5 참조.

---

## 2. 아키텍처 개요

### 2.1 계층 구조
```
┌─────────────────────────────────────────────┐
│                UI Layer                       │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ Ratatui  │  │  egui    │  │ GameLog   │  │
│  │ (TUI)    │  │ (GUI)    │  │           │  │
│  └──────────┘  └──────────┘  └───────────┘  │
├─────────────────────────────────────────────┤
│              Game Loop                        │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ Input    │  │ Dispatch │  │ Render    │  │
│  └──────────┘  └──────────┘  └───────────┘  │
├─────────────────────────────────────────────┤
│            Core Systems (ECS)                 │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌───────┐ │
│  │Combat  │ │Creature│ │Social  │ │ Misc  │ │
│  │engine  │ │movement│ │pray    │ │artifact│ │
│  │mhitu   │ │do_wear │ │shop    │ │potion │ │
│  │mhitm   │ │evolve  │ │quest   │ │spell  │ │
│  │throw   │ │end     │ │        │ │dig    │ │
│  │weapon  │ │eat     │ │        │ │fountain│ │
│  │kick    │ │status  │ │        │ │zap    │ │
│  └────────┘ └────────┘ └────────┘ └───────┘ │
├─────────────────────────────────────────────┤
│          LLM Engine (Phase E4)                │
│  ┌────────────┐ ┌──────────┐ ┌────────────┐ │
│  │ProcessMgr  │ │LlmEngine │ │Accelerator │ │
│  │(Sidecar)   │ │(async)   │ │(GPU detect)│ │
│  └─────┬──────┘ └────┬─────┘ └────────────┘ │
│        │ HTTP        │ generate_async()      │
│  ┌─────▼──────────────▼──────┐               │
│  │ llama-server (llama.cpp)  │               │
│  │ Qwen3-4B Q4_K_M (2.3GB)  │               │
│  └───────────────────────────┘               │
├─────────────────────────────────────────────┤
│            Data Layer                         │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌───────┐ │
│  │Dungeon │ │Entity  │ │Assets  │ │ Save  │ │
│  │Grid    │ │Monster │ │JSON    │ │ Load  │ │
│  │Tile    │ │Item    │ │Templates│ │       │ │
│  │Vision  │ │Player  │ │        │ │       │ │
│  └────────┘ └────────┘ └────────┘ └───────┘ │
└─────────────────────────────────────────────┘
```

### 2.2 C→Rust 변환 규칙
| C 패턴 | Rust 대응 |
|--------|-----------|
| `struct rm` (전역 맵 타일) | `Tile` 구조체 |
| `struct monst` (몬스터 연결 리스트) | `Monster` Entity + Components |
| `struct obj` (아이템 연결 리스트) | `Item` Entity + Components |
| `struct you` (전역 플레이어) | `Player` ECS Resource |
| `dlevel_t` (던전 레벨 전역) | `DungeonLevel` Resource |
| 전역 변수 (`level`, `u`, `fmon`) | ECS Resources로 캡슐화 |
| `goto` | `match` / `loop` + `break` |
| 매크로 (`#define`) | `enum` / `const` |
| 포인터 / NULL | `Option<T>` / `Vec<T>` |

### 2.3 모듈-원본 매핑 (주요 파일)
| Rust 모듈 | 원본 C 파일 | 원본 줄수 | 현재 줄수 | 이식률 |
|-----------|------------|----------|----------|--------|
| `engine.rs` | `uhitm.c` | 2,975 | 955 | 32.1% |
| `mhitu.rs` | `mhitu.c` | 2,819 | 1,131 | 40.1% |
| `mhitm.rs` | `mhitm.c` | 1,663 | 1,017 | 61.2% |
| `movement.rs` | `hack.c` | 2,939 | 1,186 | 40.4% |
| `do_wear.rs` | `do_wear.c` | 2,816 | 1,114 | 39.6% |
| `pray.rs` | `pray.c` | 2,162 | 978 | 45.2% |
| `throw.rs` | `dothrow.c` | 2,025 | 812 | 40.1% |
| `artifact.rs` | `artifact.c` | 2,005 | 819 | 40.8% |
| `evolution.rs` | `polyself.c` | 1,776 | 773 | 43.5% |
| `end.rs` | `end.c` | 2,092 | 1,183 | 56.5% |

---

## 3. 핵심 시스템 명세

### 3.1 전투 시스템 (Combat)
- **플레이어→몬스터**: `uhitm.c` → `engine.rs` — 무기별 보너스, 명중 판정, AC 계산
- **몬스터→플레이어**: `mhitu.c` → `mhitu.rs` — 특수공격(절도/질병/라이칸/녹/삼키기)
- **몬스터↔몬스터**: `mhitm.c` → `mhitm.rs` — 몬스터 간 전투
- **투척**: `dothrow.c` → `throw.rs` — 투척물 궤적, 연발, 보석 수락
- **무기**: `weapon.c` → `weapon.rs` — 숙련도, 데미지, 보너스
- **발차기**: `dokick.c` → `kick.rs` — 문/상자/몬스터 차기

### 3.2 생물 시스템 (Creature)
- **이동**: `hack.c` → `movement.rs` — BFS 경로탐색, 지형 통과, 얼음 미끄러짐
- **장비**: `do_wear.c` → `do_wear.rs` — 착/탈 효과, 장비 파괴, 드래곤 갑옷
- **변신**: `polyself.c` → `evolution.rs` — 폴리모프, 신체 부위, 능력 변화
- **종료**: `end.c` → `end.rs` — 사망 처리, 생명 구원, 공시
- **식사**: `eat.c` → `eat.rs` — 영양, 시체 효과, 알레르기

### 3.3 사회 시스템 (Social)
- **기도**: `pray.c` → `pray.rs` — 신앙심, 축복/저주 해제, 분노
- **상점**: `shk.c` → `shop.rs` — 거래, 절도, 가격 계산
- **퀘스트**: `quest.c` → `quest.rs` — 퀘스트 진행, 조건 확인

### 3.4 던전 시스템 (Dungeon)
- **레벨 생성**: `mklev.c` → 미구현 (개별 레벨)
- **방 생성**: `mkroom.c` → 부분 구현
- **타일**: `rm.h` → `tile.rs` — TileType 43종
- **시야**: `vision.c` → `vision.rs` — 가시성 계산

### 3.5 상태 시스템 (Status)
- **StatusFlags**: 41종 (u64 bitflags) — 디버프/버프/저항/이동/부하/치명
- **StatusBundle**: 영구 + 시한부 상태 관리, tick 기반 해소
- **StatusEffect enum**: 41종 → StatusFlags로 양방향 변환

---

## 4. 데이터 형식

### 4.1 몬스터 데이터 (`monsters.json`)
```json
{
  "name": "grid bug",
  "symbol": "x",
  "level": 0,
  "movement": 12,
  "ac": 9,
  "mr": 0,
  "attacks": [{"atype": 2, "adtype": 6, "dice": 1, "sides": 1}],
  "weight": 15,
  "nutrition": 10,
  "flags1": 65537,
  "flags2": 0
}
```

### 4.2 아이템 데이터 (`items.json`)
```json
{
  "name": "long sword",
  "class": "weapon",
  "weight": 40,
  "price": 15,
  "damage_small": "1d8",
  "damage_large": "1d12",
  "material": "iron"
}
```

---

## 5. 이식 규칙 (v2.9.3 감사에서 추가)

### 5.1 호출부 우선 원칙
유틸 함수 이식 시, **반드시 호출하는 시스템도 함께 구현**한다.
`pub fn`으로 공개만 하고 호출되지 않는 "섬 코드" 금지.

### 5.2 ECS 래퍼 의무화
순수 함수(로직)만 이식하고 ECS 래퍼(데이터 접근)를 구현하지 않으면 미완성.
`monster_ai`, `player_action`, `game_loop` 등에서의 연결 코드를 함께 작성.

### 5.3 매직넘버 상수화
원본 NetHack의 하드코딩 확률값은 `const`로 분리하고, 주석에 원본 값을 명시.

### 5.4 감사 체크리스트
- `cargo build` 에러 0개
- `cargo test` 전체 통과
- 새로운 `pub fn`이 최소 1곳에서 호출 확인
- 매직넘버 상수 분리 확인
- 주석 전체 한국어 확인
- `audit_roadmap.md`, `CHANGELOG.md`, `IMPLEMENTATION_SUMMARY.md` 동기화

---

## 6. 버전 이력

| 버전 | 날짜 | 이식률 | 주요 변경 |
|------|------|--------|-----------|
| v2.0.0 | 2026-02-07 | ~25% | ECS 전면 전환, Legion 채택 |
| v2.3.5 | 2026-02-15 | 35.99% | 대규모 시스템 확장 (8종 Statistics) |
| v2.9.0 | 2026-02-17 | 39.67% | end.rs, evolution.rs 대량 이식 |
| v2.9.3 | 2026-02-17 | 41.56% | mhitu/movement 이식, ECS 통합, 이식 가이드라인 |
| v2.9.5 | 2026-02-17 | 46.4% | mon.c 100% 완료, monmove.c 완료, dog.rs 2차, muse.rs/zap.rs 대량 이식 |
| v2.9.6 | 2026-02-17 | 47.1% | dog.rs 3차 이식 완결 (13개 함수/타입 추가, 99%+ 이식률) |
| v2.9.7 | 2026-02-17 | 47.4% | wizard.rs 전량 이식 (14개 함수, 141.3% 이식률) |
| v2.9.8 | 2026-02-17 | 47.5% | sit.rs 전량 이식 (take_gold/rndcurse/attrcurse, 139.4% 이식률) |
| v2.9.9 | 2026-02-17 | 47.8% | fountain.rs 전량 이식 + rng.rs 100% 완료 (rnl 추가) |
| v2.10.0 | 2026-02-18 | 48.1% | mkroom.rs 전량 이식 (courtmon/squadmon/morguemon/cmap_to_type 등 13함수) |
| **v2.10.1** | **2026-02-18** | **52.1%** | **8개 _ext 모듈 이식 (lock/steal/light/bones/were/rip/write/minion) — 50% 돌파** |
| v2.11.0 | 2026-02-19 | 54.8% | weapon_ext/wield_ext/explode_ext/dothrow_ext/priest_ext/music_ext 대량 이식 |
| v2.12.0~v2.13.0 | 2026-02-19 | 54.8%→56.3% | potion_ext/read_ext/mthrowu_ext/spell_ext/detect_ext/teleport_ext 이식 |
| v2.14.0~v2.15.0 | 2026-02-19 | 58.2%→59.1% | dokick_ext/steed_ext/sounds_ext/fountain_ext/dig_ext/artifact_ext 이식 |
| v2.16.0~v2.17.0 | 2026-02-19 | 60.0%→60.8% | trap_ext/pray_ext/eat_ext/dbridge_ext/timeout_ext/region_ext 이식, 60% 돌파 |
| **v2.18.0** | **2026-02-19** | **61.4%** | **ball_ext/vault_ext/worm_ext 이식 — 총 27개 _ext 모듈 완료** |
| **v2.19.0** | **2026-02-21** | **64.5%** | **uhitm_ext/invent_ext/shk_ext/trap_ext확장/mhitu_ext/apply_ext/do_wear_ext — 총 33개 _ext 모듈** |
| **v2.20.0** | **2026-02-22** | **64.5%** | **Phase R7 아키텍처 개편: ActionQueue / EventQueue / InteractionProvider 적용, 에러 통합 (unwrap 제거)** |
| **v2.22.0** | **2026-02-26** | **85.2%** | **R34 대량 이식: 22개 _ext 모듈, 338파일, 3,288테스트** |
| **v2.23.0** | **2026-02-26** | **85.8%** | **Phase 2 기능 연동: 5개 Bridge (status/terrain/spell/monster_ai/social) + 통합 시나리오, 344파일, 3,318테스트** |
| **v2.24.0** | **2026-02-26** | **84.9%** | **Phase 50: 구조물 파괴/지형 변화 — dig_phase50_ext(채굴 완성)/terrain_destroy_ext(터널/붕괴)/liquid_damage_ext(용암/물 피해), 347파일, 3,376테스트** |
| **v2.25.0** | **2026-02-26** | **86.0%** | **Phase 3: 핵심 시스템 고도화 — vision/invent/botl/shk 4대 모듈 확장, 351파일, 3,441테스트** |
| **v2.26.0** | **2026-02-27** | **86.9%** | **Phase 1: 최하위 모듈 공략 — display/cmd/options/save 4대 모듈 확장, 355파일, 3,498테스트** |
| **v2.27.0** | **2026-02-27** | **87.9%** | **Phase 90: 전투/음식/장비/마법 5대 모듈 확장, 360파일, 3,546테스트** |
| **v2.28.0** | **2026-02-27** | **89.4%** | **Phase 91: 10대 모듈 대규모 확장, 370파일, 3,624테스트** |
| **v2.29.0** | **2026-02-27** | **🎉 90.1%** | **Phase 92: 포션/던지기/직업·종족 3대 모듈 확장, 373파일, 3,653테스트 — 90% 돌파!** |
| **v2.30.0** | **2026-02-27** | **91.3%** | **Phase 93: 7대 모듈 대규모 확장, 380파일, 3,710테스트** |
| **v2.31.0** | **2026-02-27** | **92.1%** | **Phase 94: 5대 모듈 확장, 385파일, 3,752테스트** |
| **v2.32.0** | **2026-02-27** | **93.0%** | **Phase 95: 5대 모듈 확장, 390파일, 3,789테스트** |
| **v2.33.0** | **2026-02-27** | **93.8%** | **Phase 96: 5대 모듈 확장, 395파일, 3,830테스트** |
| **v2.34.0** | **2026-02-27** | **94.6%** | **Phase 97: 5대 모듈 확장, 🎉 400파일 돌파, 3,869테스트** |
| **v2.35.0** | **2026-02-27** | **🎉 95.4%** | **Phase 98: 5대 모듈 확장, 405파일, 3,909테스트 — 95% 돌파!** |
| **v2.36.0** | **2026-02-27** | **96.1%** | **Phase 99: 5대 모듈 확장, 410파일, 3,948테스트** |
| **v2.37.0** | **2026-02-27** | **🏆 96.9%** | **Phase 100: 최종 통합! 415파일, 3,989테스트** |
| **v2.38.0** | **2026-02-27** | **97.5%** | **Phase 101: 420파일, 🎉 4,029테스트 (4K 돌파!)** |
| **v2.39.0** | **2026-02-27** | **98.2%** | **Phase 102: 425파일, 4,074테스트** |
| **v2.40.0** | **2026-02-27** | **98.8%** | **Phase 103: 430파일, 4,115테스트** |
| **v2.41.0** | **2026-02-27** | **💯 100%** | **🏆 Phase FINAL: 438파일, 4,177테스트 — 이식 완료!** |
| **v3.0.0-α1** | **2026-03-07** | **💯 100%** | **Phase E1~E2: GameContext + TurnRunner, 30/30 시스템 전환 완료** |
| **v3.0.0-α2** | **2026-03-11** | **💯 100%** | **Phase E3~E4: Panic Hook 강화, LLM 엔진 통합 (smaLLM 이식), 비동기 API, death.rs+stairs.rs 연결, 4,179테스트** |

---

> 이 문서의 핵심 내용은 `designs.md`에 통합 반영되어 있으며, 두 문서 간 정합성을 유지해야 합니다.
