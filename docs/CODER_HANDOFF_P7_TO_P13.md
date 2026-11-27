# AIHack 코더 핸드오프 및 마스터 개발 로드맵 (P7-Fix ~ P13)

- **문서 상태**: Active Coder Handoff & Master Roadmap (`P7-Fix` ~ `P13`)
- **작성일**: 2026-11-26 (Asia/Seoul)
- **대상 범위**: 감사 결함 시정 및 P7 마감(`Stage 1`) → 데이터 주도 엔진 기반 구축(`Stage 2 / P8`) → NetHack 3.6.7 스타일 시스템·콘텐츠 단계적 대량 흡수(`Stage 3 ~ 7 / P9 ~ P13`)
- **출발 기준선**: `P1 ~ P6` 커밋 반영본 + 미커밋 `P7` 워킹트리(54개 변경/신규 파일, 체크리스트 미마감 상태)
- **운영 기준**: [AGENTS.md](../AGENTS.md), [spec.md](../spec.md), [campaign_spec.md](campaign_spec.md), [DESIGN_DECISIONS.md](../DESIGN_DECISIONS.md)

> [!IMPORTANT]
> **D3D Document-First 및 작업 경계**
> 1. 현재 워킹트리의 미커밋 `P7` 변경분(54개 파일)을 유실·되돌림 없이 보존한 상태에서 `Stage 1`부터 순차 진행한다.
> 2. 각 단계 착수 전 반드시 [spec.md](../spec.md), [campaign_spec.md](campaign_spec.md), [DESIGN_DECISIONS.md](../DESIGN_DECISIONS.md)(해당 단계 `ADR`), [tasks/plan.md](../tasks/plan.md), [tasks/todo.md](../tasks/todo.md)를 먼저 갱신한 뒤 코드를 수정한다.
> 3. **Git 커밋·푸시·태그·외부 게시**는 사용자의 명시적 승인 전까지 수행하지 않는다.
> 4. [legacy_nethack_port_reference/](../legacy_nethack_port_reference/AIHACK_REWRITE_VS_REPAIR_REPORT.md)는 [PROVENANCE.md](../PROVENANCE.md)상 `Blocked` 격리 자산이므로 코드·데이터·문자열을 직접 복사하거나 `Cargo.toml`/`crates`/`apps`/`src`에서 참조·import하지 않는다.

---

## 1. 확정된 제품 정체성 · 라이선스 · 핵심 정책

2026-11-26 감사/토의 세션에서 확정된 최상위 방향은 다음과 같다.

1. **제품 방향 (방향 B + 최대 흡수)**:
   - AIHack은 **NetHack 3.6.7 파생 독자 캠페인 로그라이크**다.
   - 원본 3.6.7의 1:1 완전 클론이나 바이너리·밸런스 완벽 일치를 표방하지 않되, **콘텐츠(직업·종족·몬스터·아이템·던전·특수 분기)와 핵심 로그라이크 규칙·상호작용은 단계적으로 최대한 흡수**한다.
2. **라이선스 및 출처 정책 (안 1 확정)**:
   - 배포 라이선스는 기존 **whole-work NGPL**(`AIHACK-OWNER-2026-07-20-NGPL-01`)을 유지한다.
   - 루트 [LICENSE](../LICENSE), [NOTICE](../NOTICE), [MODIFICATIONS.md](../MODIFICATIONS.md), [PROJECT_OWNER_LICENSE_APPROVAL.md](../PROJECT_OWNER_LICENSE_APPROVAL.md), [PROVENANCE.md](../PROVENANCE.md) 및 `NH367-C001..C010` 추적 체계를 그대로 유지한다.
   - "원본 참조만 했다"는 식으로 라이선스를 완화하지 않으며, 앞으로 NetHack 규칙과 콘텐츠를 흡수할 때도 AI-assisted semantic rewrite 규약과 R7/R8 checksum/provenance 게이트를 준수한다.
3. **방어력(AC) 정책 (권장안 확정)**:
   - 엔진 전체를 **NetHack식 하강 AC(Descending AC — 낮을수록 강함, 비무장 기본 `AC = 10`)**로 전면 통일한다.
   - 명중 판정 공식은 `defense = 20 - defender.ac`, `hit = attack_roll >= defense`를 사용한다.
   - 기존 `P1 ~ P7`의 모든 몬스터·플레이어·장비·저장 검증·테스트·인과 검증(`causal`)을 이 단일 기준으로 정렬한다.
4. **단계적 실행 순서**:
   - **Stage 1 (`P7-Fix`)**에서 P7 마감과 현재 확인된 결함(`DEF-01` ~ `DEF-11`)을 모두 시정하고 전체 게이트를 통과시킨 뒤,
   - **Stage 2 (`P8`)**에서 대량 콘텐츠 확장을 수용할 데이터 주도 엔진·장비·스탯·속도·던전 생성기 v3 기반을 세우고,
   - **Stage 3 ~ 7 (`P9` ~ `P13`)**을 순차 실행한다.

---

## 2. 현재 개발 자산 및 감사 기준선 요약 (P1 ~ P7)

### 2.1 보존해야 할 아키텍처 불변조건

현재 활성 Rust 코드베이스(`crates/`, `apps/`, `src/`, 153개 `.rs` 파일)가 보유한 핵심 공학 강점은 이후 모든 단계에서도 반드시 유지해야 한다.

- **단방향 워크스페이스 의존성**: `aihack-core` ← `aihack-content` / `aihack-ai-contract` ← `aihack-llm` / `aihack-runtime` ← `apps/aihack-tui`, `apps/aihack-headless` (`aihack-core`에 TUI/HTTP 의존성 금지).
- **단일 가변 상태 소유자 및 트랜잭션**: [session.rs](../crates/aihack-runtime/src/session.rs)의 `GameSession::submit`만 상태를 변경하며, 작업 사본에서 실패/불변조건 오류 시 롤백한다.
- **결정론적 RNG**: 모든 시뮬레이션 난수는 [rng.rs](../crates/aihack-core/src/rng.rs)의 `GameRng`만 통과한다(외형 셔플·맵 생성처럼 전투 RNG와 분리해야 하는 결정론 연산은 시드 기반 독립 SplitMix64 유지).
- **격리된 로컬 LLM 어댑터**: loopback 전용, bounded channel, `SessionRevision` 기반 stale 응답 폐기, presentation-only soft verdict.
- **원자적·경로방어 저장소**: [save.rs](../crates/aihack-runtime/src/save.rs)의 `ArtifactStore`(`cap_std`, 심볼릭 링크 차단, 리소스 상한, fail-closed semantic/map-transition 검증).

### 2.2 현재 구현 기준선 (`P1 ~ P7`) vs NetHack 3.6.7 정량 격차

| 영역 | 현재 AIHack (`P1 ~ P7` 워킹트리) | 주요 코드 위치 | NetHack 3.6.7 목표 격차 및 병목 |
| --- | --- | --- | --- |
| 직업 / 역할 | 4종(`Knight`, `Scout`, `Mage`, `Cleric`) + 레거시 `Adventurer` | [campaign.rs](../crates/aihack-core/src/campaign.rs) | 13직업·5종족·3성향(L/N/C)·신앙/유물 부재 |
| 능력치 / 성장 | `hp`, `max_hp`, `ac`, `hit_bonus`, `damage_bonus`, `speed` (`MAX_XP=180`, Lv 1..10) | [entity.rs](../crates/aihack-core/src/domain/entity.rs) | Str(`18/xx`)/Dex/Con/Int/Wis/Cha, Lv 1..30, 무기 숙련 부재 |
| 몬스터 | 12종(`Jackal` ~ `Dragon`) | [monsters.toml](../crates/aihack-content/src/data/monsters.toml) | `MonsterKind` enum 고정, 다중 공격·속도 > 12·몬스터 인벤토리·파벌 AI 부재 |
| 아이템 | 24종 (`ItemKind` enum 고정) | [items.toml](../crates/aihack-content/src/data/items.toml) | 도구(`(`), 마법서(`+`), 컨테이너, 유물, 보석, 3슬롯(`Melee`/`Body`/`Ring`) 한계 |
| 던전 / 타일 | 8층(`Main 1..6`, `Mines 1..2`), 40×20 고정 4방+L자 통로, 타일 10종, 함정 3종 | [campaign_map.rs](../crates/aihack-runtime/src/campaign_map.rs), [tile.rs](../crates/aihack-core/src/domain/tile.rs) | `80×21` 절차적 방/복도·특수방·분수/싱크/물/용암·20+종 함정·특수 분기 부재 |
| 호환성 테스트 | 공식 대조 10개 시나리오(`NH367-C001..C010`) | [compatibility/README.md](compatibility/README.md) | Stage 2 이후 신규 시스템마다 회귀·호환 테스트 지속 확장 필요 |

> **구조적 핵심 교훈**: 현재 아이템/몬스터 1종을 추가할 때마다 Rust enum, `aihack-content`, `schema.rs`, `entity.rs`, TUI 라벨/렌더러, provenance 해시까지 10곳 이상을 수동 수정해야 한다. 따라서 **Stage 1 결함 시정 직후 Stage 2(`P8`)에서 데이터 주도 구조와 던전 생성기 v3를 먼저 구축**한 뒤에야 `P9 ~ P13`의 대량 콘텐츠 흡수가 가능하다.

---

## 3. 전체 순차 로드맵 개요 (`Stage 1` ~ `Stage 7`)

| 단계 | Phase | ADR ID | 핵심 주제 | 선행 조건 | 주요 목표 산출물 |
| --- | --- | --- | --- | --- | --- |
| **Stage 1** | `P7-Fix` | `ADR-P7-02` | 감사 결함 11종(`DEF-01..11`) 시정 & `P7` 마감 | 현재 워킹트리 | 하강 AC 통일, 52슬롯 재사용, 텔레포트 차단, 허기·자연회복, 미식별 누출 차단, Fog of War, P7 문서/게이트 PASS |
| **Stage 2** | `P8` | `ADR-P8-01` | 데이터 주도 엔진·확장 코어·던전 생성기 v3 | `Stage 1` PASS | TOML 주도 `ContentRegistry`, 10슬롯 장비, 에너지 스케줄러, 6능력치/저항/시체, `80×21` 절차적 던전, P9..P13 확장 훅 |
| **Stage 3** | `P9` | `ADR-P9-01` | 도구 · 컨테이너 · `#dip` / `#engrave` · 지형 생태계 | `Stage 2` PASS | 도구 클래스(`(`), 가방/상자, `#dip` 성수/물약 혼합, `#engrave` Elbereth, 분수/싱크/물/용암/굴착, 함정 12+종 |
| **Stage 4** | `P10` | `ADR-P10-01` | 성향 · 종족 · 신앙/희생 · 유물 · 직업군 확장 | `Stage 3` PASS | L/N/C 3성향, 5종족, 신의 분노/제단 개종/희생 선물, 고유 유물, 7~13직업군, `#enhance` 무기 숙련 |
| **Stage 5** | `P11` | `ADR-P11-01` | 살아 있는 던전: 펫 · NPC · 고급 몬스터 AI | `Stage 4` PASS | 시작 펫/길들이기, NPC 상점주인/경비대/사제/오라클, 몬스터 문 열기/장비 착용/도둑질/소환/파벌전 |
| **Stage 6** | `P12` | `ADR-P12-01` | 마법 체계 심화 & 대규모 도감 투입 | `Stage 5` PASS | 마법서(`+`) 학습/기억감퇴/실패율, 7학파 25~40주문, 완드 15~20종(변이/죽음/소원), 몬스터 80~150+·아이템 120~200+ |
| **Stage 7** | `P13` | `ADR-P13-01` | 풀 엔드게임 · 특수 분기 · 로컬 LLM 심화 통합 | `Stage 6` PASS | 소코반, 직업 퀘스트 분기, 게헤놈/보스전, 4원소계+아스트랄계 다중 제단 승천, LLM 전술 어드바이저 완전 연동 |

### 각 단계 공통 실행 프로토콜

1. **문서 선행**: 해당 단계의 `ADR`을 [DESIGN_DECISIONS.md](../DESIGN_DECISIONS.md)에 추가하고, [spec.md](../spec.md), [campaign_spec.md](campaign_spec.md), [tasks/plan.md](../tasks/plan.md), [tasks/todo.md](../tasks/todo.md)에 목표·계약·검증 기준을 확정한다.
2. **RED 테스트 작성**: 핵심 규칙·경계 조건·부정 저장(위조/오버플로) 테스트를 먼저 작성해 실패를 확인한다.
3. **점진적 구현 및 하위 호환 검증**: legacy `campaign.is_none()` 2층 모드와 `NH367-C001..C010` 계약을 훼손하지 않고 구현한다(단, `Stage 1`의 `DEF-01` 하강 AC 전환은 정책 결정에 따라 전 모드 통합 갱신).
4. **전체 게이트 통과 및 마감**:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets --locked -- -D warnings`
   - `cargo test --workspace --all-targets --locked`
   - `cargo build --workspace --all-targets --release --locked`
   - `cargo audit` / `cargo deny check licenses bans sources`
   - `scripts/r7_checkpoint.sh` / `scripts/r8_checkpoint.sh` (해당 시 `docs/provenance/r7-content.sha256` LF 해시 및 `crates/aihack-content/tests/registry.rs` golden hash 동기화)
   - [tasks/todo.md](../tasks/todo.md), [campaign_implementation.md](campaign_implementation.md), [IMPLEMENTATION_SUMMARY.md](../IMPLEMENTATION_SUMMARY.md), [README.md](../README.md), [CHANGELOG.md](../CHANGELOG.md) 갱신.

---

## 4. Stage 1 (최우선 착수): `P7` 마감 및 감사 결함 일괄 시정 (`DEF-01` ~ `DEF-11`)

> 목표: 현재 `P7` 워킹트리의 결함 11건을 모두 수정하고 테스트·문서·체크포인트를 완전히 종결해 `Stage 2 (P8)`의 안전한 베이스라인을 확보한다.

### 4.1 P0 — 즉시 시정 (치명적 규칙 파손)

#### [DEF-01] 방어구·반지 AC 부호 역전 해결 및 NetHack식 하강 AC(기본 10) 전면 통일
- **발견 근거**:
  - [items.rs](../crates/aihack-runtime/src/systems/items.rs)의 `compute_ac_with`는 `adventurer_template().ac - total_bonus`(하강 AC)로 계산한다.
  - 반면 [player.rs](../crates/aihack-core/src/domain/player.rs)의 `adventurer_template().ac`는 `0`, [combat.rs](../crates/aihack-core/src/domain/combat.rs)의 `attack_roll_value`는 `defense = 10 + defender_ac`(상승 AC), [monsters.toml](../crates/aihack-content/src/data/monsters.toml)도 상승 AC(`0..5`)를 사용한다.
  - 그 결과 플레이어가 `ArmorChainMail`(`ac = -3`)을 입으면 `defense`가 `10`에서 `7`로 떨어져 **방어구를 입을수록 적이 더 잘 맞추는 치명적 역전 버그**가 발생한다.
- **수정 명세**:
  1. **문서 선행**: [spec.md](../spec.md) §10(`defense = 20 - defender.ac`) 및 §11.1/11.2, [campaign_spec.md](campaign_spec.md), [DESIGN_DECISIONS.md](../DESIGN_DECISIONS.md)(`ADR-P7-02`)에 하강 AC 기준을 명시한다.
  2. **코어/전투/플레이어**:
     - [player.rs](../crates/aihack-core/src/domain/player.rs): `adventurer_template().ac = 10`.
     - [combat.rs](../crates/aihack-core/src/domain/combat.rs): `attack_roll_value`의 `defense` 계산을 `20 - defender_ac`로 변경한다(`i32` 포화/클램프 유지).
  3. **몬스터 데이터 변환 (`10 - old_ac`)**: [monsters.toml](../crates/aihack-content/src/data/monsters.toml)의 기존 12종 AC를 아래와 같이 변환하여 기존 `defense = 10..15` 난이도를 유지한다.
     - `Jackal`: `10`, `Goblin`: `9`, `FloatingEye`: `8`, `Kobold`: `9`, `Orc`: `8`, `Ogre`: `7`, `GiantSpider`: `8`, `GnollArcher`: `8`, `Troll`: `7`, `FireAnt`: `7`, `Minotaur`: `6`, `Dragon`: `5`.
  4. **장비/저장/검증기**: [items.rs](../crates/aihack-runtime/src/systems/items.rs) 및 [save.rs](../crates/aihack-runtime/src/save.rs)는 `adventurer_template().ac - total_bonus`(`10 - bonus`) 공식을 유지하되, 저장 검증 메시지와 테스트를 `AC 10` 기준으로 정렬한다.
  5. **관찰·HUD·생성화면**:
     - [observation.rs](../crates/aihack-ai-contract/src/observation.rs) `PlayerObservation`에 `ac: i16` 필드를 추가한다(V1 JSON 호환이 필요하면 `#[serde(default = "default_ac_ten")]` 적용 여부 결정 및 `tests/ai_api_schema.rs`·`tests/fixtures/observation_v1.json` 영향 확인).
     - [runtime/observation.rs](../crates/aihack-runtime/src/observation.rs)에서 `stats.ac`를 전달하고, [render_panels.rs](../apps/aihack-tui/src/tui/render_panels.rs) 상태 패널에 `AC:<val>` 표시 및 생성 화면의 정적 `AC: 0`을 `AC: 10`으로 수정한다.
  6. **인과 검증 및 연쇄 테스트 동기화**:
     - [causal.rs](../crates/aihack-runtime/src/causal.rs) `ArmorDefense` witness가 `player_ac` 감소뿐 아니라 실제 전투 `defense` 상승과 정합됨을 확인한다.
     - [docs/provenance/r7-content.sha256](provenance/r7-content.sha256), [PROVENANCE.md](../PROVENANCE.md), [crates/aihack-content/tests/registry.rs](../crates/aihack-content/tests/registry.rs) golden hash를 갱신한다.
     - [tests/combat.rs](../tests/combat.rs), [tests/causal_content.rs](../tests/causal_content.rs), [tests/long_run.rs](../tests/long_run.rs)(`EXPECTED_CAUSAL_HASHES`), [tests/campaign_p4.rs](../tests/campaign_p4.rs), [tests/campaign_p5.rs](../tests/campaign_p5.rs), [tests/campaign_p6.rs](../tests/campaign_p6.rs), [tests/save_load.rs](../tests/save_load.rs), [tests/inventory.rs](../tests/inventory.rs)를 하강 AC 기준으로 동기화한다.
- **완료 기준(검증)**:
  - 동일 시드·공격자 조건에서 비무장(`AC 10`) 대비 `ArmorLeather`(`AC 9`), `ArmorChainMail`(`AC 7`), `+2 ChainMail + +1 RingProtection`(`AC 4`) 장착 시 `AttackResolved.defense`가 `10 → 11 → 13 → 16`으로 증가하고 명중률이 실제로 감소함을 테스트로 증명한다.

#### [DEF-02] 인벤토리 52슬롯(`a-zA-Z`) 확장 및 빈 문자 재할당
- **발견 근거**:
  - [inventory.rs](../crates/aihack-core/src/domain/inventory.rs)의 `next_letter_index`가 `0..26`(`a..z`)으로 단조 증가만 하여, 아이템을 버리거나 소비해도 슬롯이 반환되지 않고 누적 26회 획득 시 빈 가방에서도 줍기/구매가 영구 실패한다.
  - [shop.rs](../crates/aihack-runtime/src/systems/shop.rs)는 스택 병합 가능 여부 확인 전에 `next_letter_index >= 26` 검사를 먼저 수행해 기존 스택 구매까지 차단한다.
- **수정 명세**:
  - [inventory.rs](../crates/aihack-core/src/domain/inventory.rs): `a..=z`(0..26) + `A..=Z`(26..52) 총 52슬롯에서 **현재 `entries`에 없는 가장 앞선 문자**를 할당하도록 변경한다(단일 루프/헬퍼로 구현하며, `next_letter_index` 필드는 기존 스냅샷/저장 호환을 고려해 현재 사용/최대 인덱스 또는 재활용 가능한 규칙으로 정리).
  - [shop.rs](../crates/aihack-runtime/src/systems/shop.rs) 및 [runtime/observation.rs](../crates/aihack-runtime/src/observation.rs): 스택 병합(`Pickup`, `Buy`)이 가능한 경우 슬롯 한도 검사를 통과시키고, 신규 슬롯이 필요한 경우에만 52슬롯 상한을 검사한다.
  - [save.rs](../crates/aihack-runtime/src/save.rs) 및 [world.rs](../crates/aihack-core/src/world.rs): `is_ascii_alphabetic()` 문자 허용 및 52개 슬롯 상한에 맞춰 `validate_inventory`를 정렬한다(기존 `next_letter_index <= 26` 및 `is_ascii_lowercase()` 하드코딩 해제).
- **완료 기준(검증)**:
  - 30회 이상 연속 줍기↔버리기 반복 성공 테스트, 52칸 가득 찬 상태에서 기존 스택 품목 구매 성공 / 비스택 신규 품목 거절 테스트 통과.

#### [DEF-03] `ScrollLevelTeleport` 아뮬렛 귀환 추격 우회 차단 및 목적지 정상화
- **발견 근거**:
  - [items.rs](../crates/aihack-runtime/src/systems/items.rs)의 `ConsumableEffect::LevelTeleport`가 `Main 1`이면 `Main 2`로, 그 외 모든 층(`Main 2..6`, `Mines 1..2`)에서는 무조건 `Main 1` 위 계단으로 이동시켜, `Main 6`에서 아뮬렛을 줍자마자 읽으면 추격 1·2단계를 건너뛰고 즉시 승천할 수 있다.
- **수정 명세**:
  - 캠페인 모드에서 `crate::campaign::has_amulet(world)`가 `true`이면 `LevelTeleport` 이동을 차단하고(스크롤은 소비, 메시지 `"A mysterious force prevents you from teleporting with the Amulet!"` 출력), 층/추격 단계를 변경하지 않는다.
  - 캠페인 비소지 시에는 브랜치 내 다른 결정론적 층(예: 현재 `Main` depth와 다른 유효 층)으로 이동하게 하며, 레거시 2층 모드(`world.campaign.is_none()`)는 기존 `Main 1 <-> Main 2` 호환 동작을 그대로 유지한다.
- **완료 기준(검증)**:
  - `Main 6` 아뮬렛 소지 상태에서 `ScrollLevelTeleport` 사용 시 위치가 `Main 6`으로 유지됨을 검증하고, 기존 legacy/비소지 텔레포트 테스트가 계속 통과함을 확인한다.

---

### 4.2 P1 — 높음 (핵심 생존 · UX · 콘텐츠 로더 결함)

#### [DEF-04] 허기(`nutrition`) 런타임 효과 연결 (`Weak` · `Fainting` · `Starvation`)
- **발견 근거**: [session.rs](../crates/aihack-runtime/src/session.rs)에서 턴마다 `nutrition -= 1`만 수행하고 [status.rs](../crates/aihack-core/src/domain/status.rs)의 `hunger_state()`를 소비하지 않아 `nutrition <= 0`이어도 기절·아사가 일어나지 않는다.
- **수정 명세**:
  - [combat.rs](../crates/aihack-core/src/domain/combat.rs) `DeathCause`에 `Starvation`을 추가하고 TUI 사망 화면(`"Died of starvation"`)을 연결한다.
  - 캠페인/코어 턴 처리에서 `HungerState::Weak`(`1..=50`)은 명중 페널티(`-2`) 및 경고 메시지를 적용하고, `HungerState::Fainting`(`<= 0`)은 주기적 기절(`paralysis_turns`)을 부여하며, 임계치(`nutrition <= -150`) 도달 시 `DeathCause::Starvation`으로 게임 오버 처리한다.
  - **필수 확인**: [apps/aihack-headless/src/lib.rs](../apps/aihack-headless/src/lib.rs)의 `survival_candidates`에 `Hungry`/`Weak`/`Fainting` 상태일 때 인벤토리의 식량(`FoodRation`, `CorpseJackal`)을 우선 섭취(`CommandIntent::Eat`)하는 규칙을 추가하여 [tests/long_run.rs](../tests/long_run.rs)의 1,000턴 생존 테스트(`nutrition` 900 → 1000턴 진행 시 식량 1회 이상 섭취)가 실패하지 않게 한다.

#### [DEF-05] 캠페인 자연 HP 회복 추가
- **발견 근거**: [session.rs](../crates/aihack-runtime/src/session.rs)의 `accept_turn`에 5턴마다 `MP +1` 자연 회복은 있으나 HP 자연 회복이 전혀 없다.
- **수정 명세**:
  - 캠페인 생존 중 `poison_turns == 0`이고 `nutrition > 50`일 때 일정 주기(예: `12`턴 또는 레벨 비례 주기 `(15 - level).max(5)`)마다 HP `+1`(`max_hp` 상한)을 회복한다.
  - legacy 모드(`campaign.is_none()`)에서는 기존 해시 보존 여부를 검토해 캠페인 전용으로 격리한다.

#### [DEF-06] 미식별 아이템 정보 누출 차단 (BUC/인챈트 및 상점 카탈로그)
- **발견 근거**:
  - [input.rs](../apps/aihack-tui/src/tui/input.rs)의 `item_observation_label`이 미식별 아이템(`!item.identified`)에 대해서도 `blessed`/`cursed` 및 `+1`/`-1` 인챈트를 그대로 출력한다.
  - [play_menu.rs](../apps/aihack-tui/src/tui/play_menu.rs)의 상점 구매 목록(`key == '$'`)이 미식별 물약/두루마리/반지에 대해서도 실제 `item_label(*kind)` 이름을 그대로 노출한다.
- **수정 명세**:
  - 캠페인에서 개별 아이템의 `buc_known` / `enchant_known` 상태(또는 미식별 아이템은 BUC·인챈트 비노출, `ScrollIdentify`·제단·장착 테스트·상점 감정 시 공개)를 도입한다(`serde(default, skip_serializing_if = ...)`로 V1 저장/JSON 호환 유지).
  - [play_menu.rs](../apps/aihack-tui/src/tui/play_menu.rs)의 상점 구매 메뉴에서 미식별 품목은 `appearance_for_kind(seed, *kind)` 외형과 구매 가격을 표시한다.

#### [DEF-07] 탐험 기억(Fog of War / Explored Map) 구현
- **발견 근거**: [render_map.rs](../apps/aihack-tui/src/tui/render_map.rs)와 [runtime/observation.rs](../crates/aihack-runtime/src/observation.rs)가 현재 `visible_tiles`(반경 8 LOS)만 전달·렌더링해 지나온 방과 계단 위치가 시야에서 벗어나면 즉시 지워진다.
- **수정 명세**:
  - 레벨별 탐험 완료 셀 집합(`explored`)을 월드/세이브 상태에 추가(legacy V1 호환 시 기본 생략, `serde(default)`).
  - `Observation`에 기억된 시야 밖 타일(`visible: false`)을 포함하고, [render_map.rs](../apps/aihack-tui/src/tui/render_map.rs)에서 `visible == false` 타일은 어두운 스타일(예: `fg`를 회색/어두운 톤으로 구분)로 지형만 그리며 몬스터는 현재 시야(`visible == true`) 내에만 표시한다.

#### [DEF-08] `aihack-content` TOML 레벨 로더의 P4 ~ P6 누락 항목 보완
- **발견 근거**: [crates/aihack-content/src/lib.rs](../crates/aihack-content/src/lib.rs)의 `level_tile_overrides()`가 `"locked"` 문과 `"dart"`/`"bear"` 함정을 거부하고, `level_spawns()`에 `P5`/`P6` 아이템 8종(`PotionCure`, `WandFire`, `ScrollEnchantWeapon`, `ScrollEnchantArmor`, `ScrollRemoveCurse`, `RingPoisonResistance`, `RingFireResistance`, `RingProtection`)이 누락되어 있다.
- **수정 명세**:
  - `item_data_from_registry`와 동일한 `id <-> ItemKind` 단일 매핑 헬퍼를 공유해 중복 매칭과 누락을 영구 제거한다.
  - `level_tile_overrides()`에 `"locked"` 문 및 `"dart"`, `"bear"` 함정 파싱을 추가하고 단위 테스트로 검증한다.

---

### 4.3 P2 — 중간 (미연결 상태 정리 · `P7` 워킹트리 마감)

#### [DEF-09] Dead state 정리 (`hallucinating`, `damage_reduction`, 정적 생성 화면 스탯)
- **발견 근거**: `hallucinating`은 해제 경로만 있고 유발원이 `0`건이며, `damage_reduction`은 항상 `0`이고, [render_panels.rs](../apps/aihack-tui/src/tui/render_panels.rs) `character_creation_lines()`에 실제 엔진에 없는 `Strength: 10 / Dexterity: 10`이 정적으로 출력된다.
- **수정 명세**:
  - `character_creation_lines()`의 정적 `Strength: 10 / Dexterity: 10` 표기를 현재 실제 캠페인 스탯/역할 안내와 일치시킨다(`Stage 2 P8-B`에서 실제 6능력치 도입 시 재연결).
  - `hallucinating`은 `Stage 1`에서 환각 유발원(또는 `PotionHallucination`) + `hallucination_message` TUI 로그 출력을 연결하거나, 명시적으로 `P8/P9` 보류 항목으로 [spec.md](../spec.md)에 표기한다.

#### [DEF-10] 두루마리 BUC 분기 및 `ScrollIdentify` 대상 선택 경로 확장
- **발견 근거**: [items.rs](../crates/aihack-runtime/src/systems/items.rs)에서 `ScrollEnchantWeapon`/`ScrollEnchantArmor`/`ScrollRemoveCurse`가 두루마리의 `BucStatus`에 따른 분기가 없고, `ScrollIdentify`도 첫 미식별 아이템 자동 선택만 지원한다.
- **수정 명세**:
  - `EnchantWeapon` / `EnchantArmor`: `Cursed`이면 `-1` 강화 및 장착품 저주화, `Uncursed`이면 `+1` 및 해저주, `Blessed`이면 `+1..=+2` 또는 상한 내 보너스 강화(상한 `+5` 초과 과인챈트 안전 처리 포함).
  - `RemoveCurse`: `Blessed`이면 인벤토리 전체 해저주, `Uncursed`이면 장착 아이템만 해저주, `Cursed`이면 효과 없음(또는 메시지).
  - `ScrollIdentify`: 기존 미식별 우선 자동 선택 호환을 유지하면서, TUI/명령에서 원하는 인벤토리 대상을 지정해 감정할 수 있는 확장 경로(또는 `Read` 후 선택 상태)를 검토·구현한다.

#### [DEF-11] `P7` 미커밋 작업 검증 및 문서 동기화 마감
- [tasks/todo.md](../tasks/todo.md)의 `P7` 미체크 5개 항목과 `Stage 1` 결함 시정 항목을 모두 검증 후 `[x]` 마감한다.
- [README.md](../README.md)(한국어·EN·JA·繁體·简体 5개 섹션), [campaign_implementation.md](campaign_implementation.md), [IMPLEMENTATION_SUMMARY.md](../IMPLEMENTATION_SUMMARY.md), [CHANGELOG.md](../CHANGELOG.md)에 `P7` 및 `DEF-01..11` 시정 내역과 검증 수치를 동기화한다.

### 4.4 Stage 1 종료 게이트 체크리스트

- [ ] `cargo fmt --all -- --check` PASS
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` PASS
- [ ] `cargo test --workspace --all-targets --locked` 전 테스트 PASS (0 FAIL, 0 ignored)
- [ ] `cargo build --workspace --all-targets --release --locked` PASS
- [ ] `cargo audit` & `cargo deny check licenses bans sources` PASS
- [ ] `scripts/r7_checkpoint.sh` & `scripts/r8_checkpoint.sh` PASS

---

## 5. Stage 2 (`P8`, `ADR-P8-01`): 대량 콘텐츠 흡수를 위한 엔진 기반 업그레이드

> **핵심 목적**: `P9 ~ P13`에서 몬스터 100+종, 아이템 150+종, 13직업, 30+층 던전과 특수 상호작용을 빠르게 투입해도 코드 수정 병목이나 재작업이 생기지 않도록 엔진 아키텍처를 업그레이드한다.

### 5.1 `P8-A`: 데이터 주도 콘텐츠 레지스트리 전환

- **현재 병목 제거**:
  - 기존 legacy/호환 핵심 ID(`ItemKind`, `MonsterKind`)의 직렬화·테스트 호환 어댑터는 유지하되, 캠페인 신규 아이템·몬스터는 TOML에 정의하는 즉시 런타임과 TUI에서 자동 인식되도록 **TOML-driven ID + 속성/컴포넌트 구조**로 전환한다.
- **확장 아이템 스키마**:
  - `id`, `name`, `unidentified_pool`(색상/재질/라벨 풀), `class`, `glyph`, `color`, `weight`, `base_price`, `slot`, `material`(Iron/Silver/Wood/Leather/Cloth/Glass/Paper/Stone 등), `stackable`, `damage_small`/`damage_large`, `hit_bonus`, `ac_bonus`, `effects[]`, `tags[]`, `spawn_depth_range`, `spawn_weight`.
- **확장 몬스터 스키마**:
  - `id`, `name`, `glyph`, `color`, `base_level`, `hp_dice`, `ac`(하강), `speed`(기존 `0..=12` 상한 해제 → `1..=36`), `attacks[]`(다중 공격 슬롯: 물리/독/화염/냉기/전기/마비/훔치기/부식/시선 등), `resistances[]`, `corpse_chance`, `corpse_nutrition`, `corpse_effects[]`, `ai_flags[]`(문 열기, 줍기, 원거리, 소환 등), `spawn_depth_range`, `spawn_weight`, `difficulty`.
- **TUI 레지스트리 직결**: [render_map.rs](../apps/aihack-tui/src/tui/render_map.rs), [render_panels.rs](../apps/aihack-tui/src/tui/render_panels.rs), [input.rs](../apps/aihack-tui/src/tui/input.rs)가 `match` 하드코딩 대신 `Observation`/레지스트리 메타데이터에서 glyph·이름·색상을 직접 읽도록 전환한다.

### 5.2 `P8-B`: 확장 RPG 코어 (장비 슬롯 · 에너지 스케줄러 · 능력치 · 저항 · 시체)

1. **장비 슬롯 10종 확장**:
   - `Melee`, `Shield`, `Helm`, `Body`, `Cloak`, `Gloves`, `Boots`, `LeftRing`, `RightRing`, `Amulet`.
   - 통합 하강 AC: `base_ac(10) - sum(equipped_ac_bonus + equipped_enchant)`.
2. **에너지 누적 턴 스케줄러**:
   - 기존 `turn * speed / 12` 정수 나눗셈 방식을 `energy` 버킷 누적 방식으로 교체하여, `speed = 12`를 표준 1행동으로 두고 `speed = 24`(2배속)·`speed = 6`(반속) 및 가속/감속 버프를 자연스럽게 처리한다.
3. **6대 능력치 및 보조 스탯**:
   - `Str`, `Dex`, `Con`, `Int`, `Wis`, `Cha` 도입:
     - `Str`: 최대 운반 한도 및 근접 명중/피해 보정
     - `Dex`: 명중/회피·함정 회피 보정
     - `Con`: 레벨업 HP 보정 및 자연 회복 속도
     - `Int`/`Wis`: 주문 성공률·MP 성장·기도/신앙 보정
     - `Cha`: 상점 매매 가격 보정
4. **저항 비트셋 & 상태이상 체계**:
   - 저항: `Fire`, `Cold`, `Shock`, `Poison`, `Sleep`, `Disintegration`, `Telepathy`, `SeeInvisible`.
   - 상태이상: `Blind`, `Confused`, `Stunned`, `Hallucinating`, `Fast`, `Levitation`, `Invisible` + 통합 턴 틱 처리.
5. **전 몬스터 시체 일반화**:
   - 모든 몬스터 사망 시 `corpse_chance`에 따라 해당 몬스터 시체 생성(부패 타이머 `corpse_age` 포함), 섭취 시 식중독(부패/독성 시체) 또는 확률적 영구 고유능력(Intrinsic 저항) 획득, 트롤 시체 부활 규칙 구현.

### 5.3 `P8-C`: 절차적 던전 생성기 v3 및 구조적 맵 검증기

1. **절차적 던전 생성기 v3 (`80×21`)**:
   - 기존 `40×20` 4방 직렬 고정 구조를 대체하는 결정론적 방·복도·루프 생성기(방 크기/개수 가변, 교차 복도, 어두운 방/밝은 방, 특수방 슬롯: 상점방·신전·금고·군영/동물원).
   - 시드 기반 BFS/Union-Find로 모든 계단·분기 입구 간 도달 가능성 자동 보장.
2. **던전 브랜치 그래프 일반화**:
   - `Main 1..20+`, `Mines 1..5+`, 특수 분기(`Sokoban`, `Quest`, `Gehennom` 등)를 데이터/그래프 테이블로 정의하여 `stairs.rs`의 하드코딩 브랜치 전이를 대체한다.
3. **저장 맵 검증기 진화**:
   - `campaign_map::validate_persisted_map`을 타일 1:1 diff 비교에서 **구조적 불변조건 검증**(맵 크기·경계·필수 계단/분기 포탈 보존·허용 타일 집합 무결성·리소스 상한)으로 진화시켜 이후 `P9`의 벽 파기(`Dig`), 문 파괴, 함정 생성, 지형 변화를 수용한다.

### 5.4 `P9 ~ P13`을 위해 `P8` 스키마에 미리 열어둘 확장 훅

`P8` 데이터 모델 및 저장 스키마(`SaveV3` 전환 시 `ADR-P8-01`로 V1/V2 읽기 호환 정책 확정) 설계 시 아래 필드를 처음부터 포함해 이후 단계의 대수술을 방지한다.

| 예약 필드 / 훅 | 대상 구조 | 실제 소비 단계 |
| --- | --- | --- |
| `material`, `tool_action`, `container_capacity`, `contained_items[]` | 아이템 정의 / `EntityPayload::Item` | `P9` (도구·가방/상자·녹/부식) |
| `tile_feature` (분수/싱크/물/용암/조각문 `engravings[]`) | `GameLevel` / `TileKind` | `P9` (`#dip`, `#engrave`, 지형 생태계) |
| `alignment`(L/N/C), `race`, `god_anger`, `prayer_timeout`, `weapon_skills` | `CampaignState` / 플레이어 | `P10` (성향·종족·신앙·숙련) |
| `artifact_id`, `generated_artifacts: BTreeSet<String>` | 아이템 정의 / `WorldState` | `P10` (고유 유물 1회 생성 보장) |
| `Faction` (`Player`, `Pet`, `Peaceful`, `Hostile`, `Shopkeeper`), `monster_inventory` | `EntityPayload::Actor` | `P11` (펫·상점주인·몬스터 소지품/훔치기) |
| `known_spells` (`learned_at_turn`), `spell_skill`, `effect_scripts[]` | 플레이어 / 아이템·주문 스키마 | `P12` (마법서 학습·기억 감퇴·대규모 효과) |
| `branch_graph`, `special_level_id`, `ascension_altars[]` | `LevelRegistry` / 캠페인 규칙 | `P13` (소코반·퀘스트·원소계·아스트랄계) |

---

## 6. Stage 3 ~ Stage 7 (`P9` ~ `P13`) 순차 확장 마스터 로드맵

> **진행 원칙**: 각 Phase는 직전 Phase의 전체 게이트 PASS 이후에만 착수한다.

### Stage 3 (`P9`, `ADR-P9-01`): 도구 · 컨테이너 · `#dip` / `#engrave` · 지형 생태계

- **선행 조건**: `Stage 2 (P8)` 완료 및 전체 게이트 PASS.
- **핵심 구현 항목**:
  1. **도구 클래스(`(`) 및 `#apply` 명령**: 열쇠/자물쇠따개(잠긴 문·상자 열기), 곡괭이/드워프 맷돌(벽 파기 `Dig`), 호루라기(펫 호출), 등불/기름 램프(어두운 방 시야 확장), 눈가리개(시각 차단·텔레파시 활용), 유니콘 뿔(상태이상·능력치 손실 회복).
  2. **컨테이너(가방·상자) 및 `#loot`**: 무게 감경 가방(Bag of Holding, 소모품/전리품 보관), 잠긴/함정 상자 열기 및 넣기·꺼내기(가방 중첩 폭발 등 NetHack 대표 상호작용 포함).
  3. **`#dip` 및 성수(Holy Water) 경제**: 물약에 아이템 담그기, 빈 병/물에 담가 씻기, 제단 축복을 통한 성수 제작 및 성수를 통한 BUC 확인/해저주/축복.
  4. **`#engrave` 및 Elbereth**: 바닥에 손가락·무기·완드로 글씨 새기기, `Elbereth` 각인 타일 위에서 대부분의 비이성 몬스터 회피(마모·지워짐 포함).
  5. **지형 타일 및 함정 12+종 확장**: 분수(물 마시기·요정/물 정령 이벤트), 싱크, 물/용암(수영/부유/익사), 기존 3종 함정(`Pit`, `Dart`, `Bear`) + 텔레포트, 수면 가스, 녹, 변이, 구멍(낙하), 반마법, 바위 낙하, 마법 함정 등 확장.
- **종료 기준**: 도구/컨테이너/성수/각인/굴착/함정 E2E 시나리오 및 저장 무결성 테스트, 전체 워크스페이스 게이트 PASS.

### Stage 4 (`P10`, `ADR-P10-01`): 성향 · 종족 · 신앙/희생 · 유물 · 직업군 확장

- **선행 조건**: `Stage 3 (P9)` 완료.
- **핵심 구현 항목**:
  1. **3성향(`Lawful`, `Neutral`, `Chaotic`) & 5종족(`Human`, `Elf`, `Dwarf`, `Gnome`, `Orc`)**: 종족별 기본 스탯·저항(인프라비전, 독/수면 저항 등)·선호/허용 직업 조합.
  2. **직업군 대폭 확장**: 기존 4직업 체계를 NetHack 핵심 직업군으로 확장(1차: `Valkyrie`, `Wizard`, `Rogue`, `Priest`, `Samurai`, `Barbarian`, `Ranger`, `Knight` → 최종 13직업 목표 완성) + 직업별 초기 장비/주문/고유 레벨업 특성.
  3. **신앙·제단·희생 심화**: 성향 일치 제단 vs 타 성향 제단 구분, 타 성향 제단에서 기도시 신의 분노 또는 개종, 시체 희생(`#offer`)에 따른 신의 은총·행운(`-13..=+13`)·성수 축복·최초 희생 유물 선물(`Excalibur`, `Stormbringer`, `Magicbane` 등 고유 유물).
  4. **고유 유물(Artifacts) 시스템**: 게임 전체에서 각 유물 1회만 생성 보장, 성향 불일치 유물 착용/공격 시 블래스트 피해, 특수 발동 효과(`#invoke`) 및 속성 추가 피해.
  5. **무기·주문 숙련(`#enhance`)**: 전투/시전 사용량 기반 숙련도 슬롯(`Unskilled` → `Basic` → `Skilled` → `Expert` → `Master` → `Grand Master`) 및 명중·피해 보정.
- **종료 기준**: 역할×종족×성향 조합 시작·제단 개종/신의 분노/유물 1회 생성 불변조건·숙련 성장 회귀 테스트 및 전체 게이트 PASS.

### Stage 5 (`P11`, `ADR-P11-01`): 살아 있는 던전 — 펫 · NPC · 고급 몬스터 AI

- **선행 조건**: `Stage 4 (P10)` 완료.
- **핵심 구현 항목**:
  1. **동료(Pet) 시스템**: 시작 시 직업/종족에 맞는 반려동물(고양이·개·조랑말) 동반, 먹이 주기로 친밀도 유지 및 길들이기, 처치 경험을 통한 펫 성장·변이, 바닥 저주 감별(펫이 저주 아이템 기피), 펫의 상점 물건 훔치기 등 대표 상호작용.
  2. **실제 NPC 상점주인 및 평화적 NPC**:
     - 기존 타일 위 메뉴 상점을 **상점 방 내부의 실제 `Shopkeeper` NPC + 바닥 진열 상품 + 문 앞 정산/채무 시스템**으로 업그레이드(도둑질 시 경비대/분노 상점주인 적대 전환).
     - 제단 사제(기부·감정·축복), 오라클(소문/힌트), 미네스타운 경비병(무법 행위 단속) 추가.
  3. **고급 몬스터 AI 및 생태계**:
     - 몬스터의 문 열기, 바닥 무기/방어구/완드/물약 줍기 및 실제 착용·사용.
     - 특수 몬스터 패턴: 금화/아이템 훔치기(님프/레프러콘 — 텔레포트 도주), 녹/부식(러스트 몬스터), 분열, 소환, 시선 공격(블라인드 상태 시 방어), 몬스터 간 파벌전(오크 vs 드워프 등).
- **종료 기준**: 펫 동반/길들이기/층간 이동, 상점주인 채무·도둑질, 몬스터 아이템 사용/훔치기 E2E 테스트 및 전체 게이트 PASS.

### Stage 6 (`P12`, `ADR-P12-01`): 마법 체계 심화 & 대규모 도감 투입

- **선행 조건**: `Stage 5 (P11)` 완료.
- **핵심 구현 항목**:
  1. **마법서(`+`) 학습 & 기억 감퇴**: 마법서를 읽어 주문 습득(실패 시 저주/부작용), 20,000턴(또는 기준 턴) 기억 유지 및 재학습 필요, `Int`·주문 난이도·금속 갑옷/방패 착용에 따른 주문 실패율 계산.
  2. **7학파 주문 & 확장 완드·반지·아뮬렛**:
     - 주문 25~40종(치유, 식별, 매직 미사일, 파이어볼/냉기 광선, 해저주, 매혹, 텔레포트, 순간이동 제어, 소환 등).
     - 완드 15~20종(변이 `Polymorph`, 죽음 `Death`, 굴착 `Digging`, 텔레포트, 취소 `Cancellation`, 소원 `Wishing` — 제한된 소원 문법·밸런스 포함).
     - 반지·아뮬렛 확장(생명보존 `Life Saving`, 반사 `Reflection`, 투명, 공중부양, 갈등 `Conflict`, 탐색 등).
  3. **대규모 도감 투입**:
     - **몬스터 80 ~ 150+종** 및 **아이템 120 ~ 200+종**을 `crates/aihack-content/src/data/` TOML 도감으로 투입(다중 파일 분할 허용 시 체크포인트 스크립트 동기화 필수).
     - 전체 도감 자동 검증: 모든 TOML 엔트리가 파싱·스폰·관찰·TUI 렌더링·시체/전리품 생성까지 무오류임을 테스트한다.
- **종료 기준**: 마법서 학습/기억감퇴/실패율, 소원/변이/반사/생명보존 회귀 테스트, 전 도감 엔트리 스폰/렌더 스모크 테스트 및 전체 게이트 PASS.

### Stage 7 (`P13`, `ADR-P13-01`): 풀 엔드게임 · 특수 분기 · 로컬 LLM 심화 통합

- **선행 조건**: `Stage 6 (P12)` 완료.
- **핵심 구현 항목**:
  1. **특수 레벨 및 분기 완성**:
     - **소코반(Sokoban)**: 바위 밀기 퍼즐 4층 및 완주 보상(반사 아뮬렛 또는 가방 오브 홀딩).
     - **직업별 퀘스트 분기**: 중간 분기 탐험 → 퀘스트 보스 처치 → 직업 퀘스트 유물 회수.
     - **게헤놈(Gehennom) 심층**: 화염/미로 구역, 주요 네임드 보스전, **실제 옌도르의 아뮬렛(Amulet of Yendor)** 회수 및 마법사의 방해·탈출 추격전.
  2. **4원소계(Earth, Air, Fire, Water) + 아스트랄계(Astral Plane) 최종 승천**:
     - 아뮬렛 소지 후 `Main 1` 상향 계단 통과 시 4원소계 포탈 순차 돌파 → 아스트랄계 진입 → 3개 제단(L/N/C) 중 본인 성향과 일치하는 고위 제단에 아뮬렛 봉헌 시 **진정한 승천(`Victory`)** 달성(4기사 컨셉 최종 위협 포함).
  3. **로컬 LLM 어드바이저 심화 통합**:
     - 확장된 `Observation`(능력치·저항·장비 10슬롯·도구·신앙·특수 층 정보 포함)을 `LlmObservationView` 예산(`32,768` bytes) 내로 구조화해 전술 추천(`Decision`), 상황 내러티브(`Narrative`), 보조 판정(`SoftAdjudication`)이 심층 캠페인에서도 결정론 코어를 해치지 않고 동작하도록 완성한다.
- **종료 기준**: 4원소계→아스트랄계→성향 제단 승천 E2E 완주 테스트, LLM payload 예산·stale-revision·결정론 해시 불변 테스트, 전체 워크스페이스 및 R7/R8 게이트 PASS.

---

## 7. 필수 주의사항 (Pitfalls) 및 공통 게이트 규칙

구현 중 반복해서 발생하기 쉬운 저장소 고유의 자동 게이트 함정들을 미리 정리한다.

1. **문서 상대 링크 전수 검사 ([r8_documentation.rs](../tests/r8_documentation.rs))**:
   - `active_markdown_relative_links_resolve_after_audit_report_relocation` 테스트는 `.archive`, `target`, `legacy_nethack_port_reference`, `.git`을 제외한 **저장소 내 모든 `.md` 파일의 모든 줄**(코드 블록 내부 포함!)에서 마크다운 링크 문법과 참조 링크(`]:`)를 정규식으로 스캔한다.
   - 따라서 코드 예시나 주석 안에서도 존재하지 않는 상대 경로 링크나 `[이름]: 값` 형태의 줄을 절대 쓰지 않는다.
   - 루트 문서(`README.md`, `spec.md`, `DESIGN_DECISIONS.md`, `IMPLEMENTATION_SUMMARY.md`, `BUILD_GUIDE.md`, `audit_roadmap.md`, `GAP_CLOSURE_ROADMAP.md`, `designs.md`)의 `audit_report_32.md` 단일 권위 문자열 검증(`active_document_sections_have_one_report_32_authority_and_reject_predecessors`)을 건드리지 않도록 주의한다.
2. **콘텐츠 체크섬 및 R7/R8 체크포인트 ([r7_checkpoint.sh](../scripts/r7_checkpoint.sh), [provenance_manifest.rs](../tests/provenance_manifest.rs))**:
   - [items.toml](../crates/aihack-content/src/data/items.toml), [monsters.toml](../crates/aihack-content/src/data/monsters.toml), 레벨 TOML 수정 시 반드시 **LF 기준 SHA-256**을 계산해 [docs/provenance/r7-content.sha256](provenance/r7-content.sha256)와 [PROVENANCE.md](../PROVENANCE.md)에 반영한다.
   - `Stage 2 (P8)` 또는 `Stage 6 (P12)`에서 `crates/aihack-content/src/data/` 아래에 새로운 하위 디렉터리(예: `special_levels/`, `roles/`)나 분할 TOML 파일을 추가할 경우, [r7_checkpoint.sh](../scripts/r7_checkpoint.sh)의 `r7-content.sha256` 경로 정규식 허용 패턴(현재 최상위 `.toml`과 `levels/*.toml`만 허용)과 [r8_checkpoint.sh](../scripts/r8_checkpoint.sh)를 함께 갱신해야 한다.
3. **1,000턴 장기 실행 및 인과 해시 ([tests/long_run.rs](../tests/long_run.rs))**:
   - `Stage 1`의 `DEF-01`(하강 AC), `DEF-04`(허기 식량 소비), `DEF-05`(자연 회복)는 `survival-v1`과 `EXPECTED_CAUSAL_HASHES`에 직접 영향을 준다.
   - 먼저 `survival-v1`가 3개 시드(`42, 7, 1234`)에서 1,000턴 동안 굶어 죽거나 멈추지 않고 **3회 동일 해시**를 내는지 확인한 뒤, 인과 검증 9종(`REQUIRED_CAUSAL_WITNESSES`)을 모두 통과한 새 결정론 해시로 `EXPECTED_CAUSAL_HASHES`를 동기화한다.
4. **세이브 스키마 버전 관리 (`V1` / `V2` / `V3`)**:
   - `Stage 1`까지는 기존 `V1`(legacy) / `V2`(campaign) 저장 계약을 유지하고 신규 필드는 `#[serde(default, skip_serializing_if = ...)]`로 처리한다.
   - `Stage 2 (P8)`에서 10슬롯 장비·6능력치·가변 데이터 주도 스키마가 들어올 때는 `ADR-P8-01`에서 `schema_version = 3` 도입 여부와 `V1`/`V2` 호환·거부 경계를 명시적으로 결정한다.
5. **`LlmObservationView` 32 KiB 페이로드 상한 ([spec.md](../spec.md) §9.4)**:
   - `P8 ~ P13`에서 인벤토리(52슬롯)·몬스터/타일 관찰 필드가 늘어나도 LLM 요청 JSON이 `32,768` bytes를 초과하지 않도록 `LlmObservationView` 프로젝션 절단·요약 정책을 유지한다.
