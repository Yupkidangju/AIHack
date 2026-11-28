# P1–P3 구현 계획

2026-09-05. 사용자 요청: P3까지 구현. 기존 미커밋 P0 및 봉인 원본을 보존한다. 기준: `spec.md`, `docs/campaign_spec.md`.

| 기능 ID | 책임 | 선행 |
| --- | --- | --- |
| roles-growth | 역할·XP·저장·관찰 | 기존 submit |
| exploration | 6층+광산 2층 생성·왕복 | roles-growth |
| ascension | 목표 물건·귀환·성공 결과 | exploration |

타입→runtime→UI의 작은 단위마다 표적 테스트, 마지막에 전체 테스트/fmt/clippy/release를 실행한다. 구형 save/hash는 campaign=None 생략으로 보존하고 새로운 저장은 wire schema 2로 구분한다. 위조 성장/목표와 단절된 dungeon은 validator/BFS/실제 명령 완주 테스트로 방어한다. 커밋·푸시·배포는 이번 범위가 아니다. 체크리스트는 `tasks/todo.md`, 결과는 `docs/campaign_implementation.md`.

# P4 구현 계획 (2026-11-26)

기준: `spec.md`, `docs/campaign_spec.md` (P4), `DESIGN_DECISIONS.md` (`ADR-P4-01`). 커밋·푸시는 수행하지 않는다.

| 단계 | 책임 | 핵심 대상 |
| --- | --- | --- |
| P4a | `TrapKind::Dart/Bear`, 캠페인 문/비밀문/함정 생성, 전이-인식 V2 맵 검증 | `core/tile.rs`, `runtime/campaign_map.rs`, `runtime/save.rs`, `runtime/systems/traps.rs`, `runtime/systems/movement.rs`, `runtime/session.rs` |
| P4b | `Role::Cleric`, 신규 아이템 5종(`RechargeWand` 포함), 몬스터 3종, 캠페인 배치 | `core/campaign.rs`, `core/domain/*`, `aihack-content/*`, `runtime/bootstrap.rs`, `runtime/systems/items.rs`, `runtime/observation.rs`, `PROVENANCE.md`, `r7-content.sha256` |
| P4c | TUI 문/함정/아이템/몬스터/Cleric 렌더·입력 및 Headless `--role cleric` | `apps/aihack-tui/*`, `apps/aihack-headless/*` |
| P4d | 통합/부정 테스트, 전체 gate(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`) 및 문서 동기화 | `tests/campaign.rs`, `docs/campaign_implementation.md`, `IMPLEMENTATION_SUMMARY.md`, `README.md`, `CHANGELOG.md`, `tasks/todo.md` |

# P5 구현 계획 (2026-11-26)

기준: `spec.md`, `docs/campaign_spec.md` (P5), `DESIGN_DECISIONS.md` (`ADR-P5-01`). 커밋·푸시는 수행하지 않는다.

| 단계 | 책임 | 핵심 대상 |
| --- | --- | --- |
| P5a | `BucStatus`/`enchantment`, 별도 `Entity` 접근자, `DoorState::Locked` 및 `kick_door`·전투/AC/저장 검증 | `core/domain/*`, `core/doors.rs`, `runtime/systems/{combat,items,projectiles}.rs`, `runtime/campaign_map.rs`, `runtime/save.rs` |
| P5b | `poison_turns`/`DeathCause::Poison`, 신규 5아이템(인챈트/해저주 두루마리 3종·`PotionCure`·`WandFire`), 신규 3몬스터(`GiantSpider`·`GnollArcher`·`Troll`), content/provenance 동기화 | `core/world.rs`, `core/save.rs`, `aihack-content/*`, `runtime/systems/monster_ai.rs`, `runtime/session.rs`, `runtime/bootstrap.rs`, `PROVENANCE.md`, `r7-content.sha256` |
| P5c | TUI(`locked door`, `POIS:`, BUC/인챈트 라벨, 몬스터/아이템, `DeathCause::Poison`) 및 Headless `survival-v1` 연동 | `aihack-ai-contract/*`, `apps/aihack-tui/*`, `apps/aihack-headless/*` |
| P5d | 회귀/통합 테스트, 전체 gate(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`) 및 문서 동기화 | `tests/campaign.rs`, `tests/content_validation.rs`, `tests/campaign_p5.rs`, `docs/campaign_implementation.md`, `IMPLEMENTATION_SUMMARY.md`, `README.md`, `CHANGELOG.md`, `tasks/todo.md` |

# P6 구현 계획 (2026-11-26)

기준: `spec.md`, `docs/campaign_spec.md` (P6), `DESIGN_DECISIONS.md` (`ADR-P6-01`). 커밋·푸시는 수행하지 않는다.

| 단계 | 책임 | 핵심 대상 |
| --- | --- | --- |
| P6a | `Ring` 슬롯/`RingEffect`/반지 3종, `FireAnt`/`FireOnMelee`, 통합 AC 및 화염·독 저항, `PutOn`/`RemoveRing` | `core/domain/*`, `core/action.rs`, `aihack-content/*`, `runtime/systems/{items,monster_ai}.rs`, `runtime/session.rs`, `runtime/save.rs` |
| P6b | `TileKind::Shop`, `Main 2`/`Mines 1` 상점 배치, 캠페인 50g/`difficulty * 25`, `Buy`/`Sell` 카탈로그·거래 | `core/domain/tile.rs`, `runtime/systems/{shop,death,movement}.rs`, `runtime/campaign_map.rs`, `runtime/bootstrap.rs`, `runtime/observation.rs` |
| P6c | TUI(`$`/`shop`, `a`/`fire ant`, `=`, `G<gold>`, `RES:F|P|FP`, `'P'`/`'T'`/`'$'`) 및 Headless `survival-v1` 반지 착용 | `aihack-ai-contract/*`, `apps/aihack-tui/*`, `apps/aihack-headless/*` |
| P6d | 회귀/통합 테스트, 전체 gate(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`) 및 문서 동기화 | `tests/campaign.rs`, `tests/content_validation.rs`, `tests/campaign_p6.rs`, `PROVENANCE.md`, `r7-content.sha256`, 문서 일체 |

# P7 구현 계획 (2026-11-26)

기준: `spec.md`, `docs/campaign_spec.md` (P7), `DESIGN_DECISIONS.md` (`ADR-P7-01`). 커밋·푸시는 수행하지 않는다.

| 단계 | 책임 | 핵심 대상 |
| --- | --- | --- |
| P7a | `TileKind::Altar`, `Main 3`/`Mines 2` 제단 배치, `mp`/`max_mp`, `SpellKind::{ForceBolt, Heal, Detect}`, `Offer`/제단 `Pray`/`Cast` | `core/{campaign,action,event,world,save}.rs`, `core/domain/tile.rs`, `runtime/systems/{altar,spells}.rs`, `runtime/{campaign_map,campaign,session,observation}.rs` |
| P7b | 시드별 미식별 외형(`appearance_for_kind`)·사용/공물/감정 식별 및 `quantity: u16` 스택(1..=99)·병합·분리 투척·무게/점수 반영 | `core/domain/{item,entity}.rs`, `runtime/appearance.rs`, `runtime/{world,save,snapshot,observation}.rs`, `runtime/systems/{items,shop,projectiles}.rs` |
| P7c | `Minotaur`/`Dragon`(`FireBreath`) 추가, `Mines 2`/`Main 6` 배치, 아뮬렛 귀환 추격 단계(`1..=3`)·부하, content/provenance/LLM 동기화 | `core/domain/{monster,entity}.rs`, `aihack-content/*`, `aihack-llm/src/decision.rs`, `runtime/systems/{ascension,monster_ai}.rs`, `runtime/bootstrap.rs`, `PROVENANCE.md`, `r7-content.sha256` |
| P7d | TUI/Headless, `campaign.rs` 4역할 완주·`tests/campaign_p7.rs`, 전체 gate(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`) 및 문서 마감 | `aihack-ai-contract/*`, `apps/aihack-tui/*`, `apps/aihack-headless/*`, `tests/{campaign,campaign_p7}.rs`, 문서 일체 |

# Stage 1 (P7-Fix) 구현 계획 (2026-11-27)

기준: `spec.md`, `docs/campaign_spec.md`, `docs/CODER_HANDOFF_P7_TO_P13.md` §4 (`DEF-01` ~ `DEF-11`), `DESIGN_DECISIONS.md` (`ADR-P7-02`). 커밋·푸시는 수행하지 않는다.

| 단계 | 책임 | 핵심 대상 |
| --- | --- | --- |
| S1-doc | `spec.md` §9.4/10/11/12, `docs/campaign_spec.md`, `DESIGN_DECISIONS.md`(`ADR-P7-02`), `tasks/plan.md`, `tasks/todo.md` 문서 선행 확정 | 루트 및 `docs/`, `tasks/` 문서 |
| S1-core | 하강 AC(`player.rs`, `combat.rs`, `monsters.toml`), 52슬롯(`inventory.rs`), `DeathCause::Starvation`/`STARVATION_THRESHOLD`, `buc_known`/`enchant_known`(`entity.rs`), `GameLevel.explored`(`level.rs`), `CommandIntent::Identify`(`action.rs`), `aihack-ai-contract`, `aihack-content` 로더/프로비넌스 동기화 | `crates/aihack-core/*`, `crates/aihack-ai-contract/*`, `crates/aihack-content/*`, `PROVENANCE.md`, `r7-content.sha256` |
| S1-runtime | `world.rs`, `bootstrap.rs`, `campaign_map.rs`, `snapshot.rs`, `save.rs`, `combat.rs`, `shop.rs`, `items.rs`, `altar.rs`, `session.rs`, `observation.rs` 런타임 규칙·불변조건 구현 | `crates/aihack-runtime/*` |
| S1-adapters | `LlmObservationView` LOS 필터·`IDENTIFY` 직렬화(`aihack-llm`), `survival-v1` 허기 섭취(`aihack-headless`), TUI `AC:`/`explored`/`muted`/상점 외형·가격/은닉/사망 원인/대상 지정 감정/생성 화면 정리(`aihack-tui`) | `crates/aihack-llm/*`, `apps/aihack-headless/*`, `apps/aihack-tui/*` |
| S1-verify | 기존 회귀 테스트·`long_run.rs` 해시 동기화, `tests/stage1_fixes.rs` 추가, 전체 gate(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`) 및 최종 문서 마감 (`DEF-11`) | `tests/*`, `crates/*/tests/*`, `README.md`, `docs/campaign_implementation.md`, `IMPLEMENTATION_SUMMARY.md`, `CHANGELOG.md`, `tasks/todo.md` |

# Stage 2 (P8) 구현 계획 (2026-11-27)

기준: `spec.md`, `docs/campaign_spec.md` (Stage 2 `P8`), `DESIGN_DECISIONS.md` (`ADR-P8-01`). 커밋·푸시는 수행하지 않는다.

| 단계 | 책임 | 핵심 대상 |
| --- | --- | --- |
| S2-doc | `DESIGN_DECISIONS.md` (`ADR-P8-01`), `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` 문서 기준 선행 확정 | 루트 및 `docs/`, `tasks/` 문서 |
| S2-core | `BranchId` 확장, `SAVE_SCHEMA_VERSION_V3`, `Resistances`/`StatusTimers`, 10 슬롯(`EquipmentSlot`/`Inventory`), `ItemKind`/`MonsterKind`/`Faction`/`Attributes`/`GameLevel` 및 P9~P13 훅, `Observation` optional 메타데이터·`TakeOff` | `crates/aihack-core/*`, `crates/aihack-ai-contract/*`, `crates/aihack-llm/*` |
| S2-content | `items.toml`/`monsters.toml` 메타데이터·신규 장비/물약/`Bat` 추가, `speed` `0..=36` 및 다슬롯/아뮬렛/상태 스키마 확장, LF 해시 및 `PROVENANCE.md` 동기화 | `crates/aihack-content/*`, `docs/provenance/r7-content.sha256`, `PROVENANCE.md` |
| S2-runtime | `80×21` Generator v3 + 구조적 `validate_persisted_map`, `BranchGraph` 계단, 10 슬롯/AC/저주/저항, 에너지 스케줄러, 6대 능력치, 상태이상 틱/시야/이동/함정, 일반화 시체(부패·저항·트롤 부활), `SaveV3` 검증 | `crates/aihack-runtime/*` |
| S2-tui | 데이터 주도 `render_map`/`render_panels`/`input`/`play_menu`(색상/글리프/이름 메타데이터, 6대 능력치/다중 저항/상태 배지/다분기 이름/장비 해제) | `apps/aihack-tui/*` |
| S2-verify | 영향 기존 캠페인/콘텐츠 테스트 정렬, `tests/stage2_p8.rs` 신설, 전체 gate(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`) 및 최종 문서 마감 | `tests/*`, `crates/*/tests/*`, `README.md`, `docs/campaign_implementation.md`, `IMPLEMENTATION_SUMMARY.md`, `CHANGELOG.md`, `tasks/todo.md` |

# Stage 3 (P9) 구현 계획 (2026-11-27)

기준: `spec.md`, `docs/campaign_spec.md` (`## Stage 3 (P9)`), `DESIGN_DECISIONS.md` (`ADR-P9-01`). 커밋·푸시는 수행하지 않는다.

| 단계 | 책임 | 핵심 대상 |
| --- | --- | --- |
| S3-doc | `DESIGN_DECISIONS.md` (`ADR-P9-01`), `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` 문서 기준 선행 확정 | 루트 및 `docs/`, `tasks/` 문서 |
| S3-core | `TileKind` 5지형·`TrapKind` 12함정·`DeathCause` 3종·`ItemKind` 13종·`ItemClass::Tool`·`ToolAction::Blindfold`·`EntityLocation::InContainer`·`CommandIntent` 6명령(`Apply`/`ContainerPut`/`ContainerTake`/`Dip`/`Engrave`/`Drink`)·`Observation`/`aihack-llm` wire 직렬화 | `crates/aihack-core/*`, `crates/aihack-ai-contract/*`, `crates/aihack-llm/*` |
| S3-content | `items.toml` P9 13아이템(도구 8종·컨테이너 3종·`PotionWater`·`ScrollBlank`) 추가, `schema.rs` `"tool"`/`"water"`/`"blank"` 검증, `lib.rs` 13종 ID 및 12함정 매핑, LF 해시·`PROVENANCE.md`·`registry.rs`·`content_validation.rs` 동기화 | `crates/aihack-content/*`, `docs/provenance/r7-content.sha256`, `PROVENANCE.md`, `tests/content_validation.rs` |
| S3-runtime | `systems/{tools,containers,dip,engrave,terrain}.rs` 신설, `traps`/`vision`/`monster_ai`/`combat`/`items`/`altar`/`shop`/`campaign_map`/`bootstrap`/`world`/`snapshot`/`save`/`session`/`observation` 통합 | `crates/aihack-runtime/*` |
| S3-adapters | `apps/aihack-tui`(`render_map`/`render_panels`/`input`/`play_menu`/`mod`, `a`/`C`/`v`/`E`/`g`) 및 `apps/aihack-headless`(캠페인 `UnicornHorn` 치료) 연동 | `apps/aihack-tui/*`, `apps/aihack-headless/*` |
| S3-verify | `tests/campaign.rs` 지형 경로 정렬, `tests/stage3_p9.rs` E2E/저장 변조/TUI/LLM 예산/레거시 보존 테스트 작성, 전체 gate(`fmt`, `clippy`, `test -j 2`, `release`, `audit`, `deny`, `r7/r8`) 및 완료 문서 마감 | `tests/*`, `crates/*/tests/*`, `README.md`, `docs/campaign_implementation.md`, `IMPLEMENTATION_SUMMARY.md`, `CHANGELOG.md`, `tasks/todo.md` |

## Stage 4 (P10) 실행 계획 (2026-11-27)

| 단계 | 작업 요약 | 대상 파일 |
| --- | --- | --- |
| S4-doc | `DESIGN_DECISIONS.md` (`ADR-P10-01`), `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md`에 Stage 4 (`P10`) 계약 및 체크리스트 선행 기록 | `DESIGN_DECISIONS.md`, `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` |
| S4-core | `campaign.rs`(3 성향·5 종족·13 역할·조합 검증·`SkillCategory`/`SkillRank`·`ARTIFACTS`), `domain/{item,combat,level}`, `world`, `save`, `action`, `aihack-ai-contract`, `aihack-llm` 확장 (`Copy`/`CampaignState`·레거시 V1 보존) | `crates/aihack-core/*`, `crates/aihack-ai-contract/*`, `crates/aihack-llm/*` |
| S4-content | `items.toml` 무기 5종(`LongSword`, `Katana`, `BattleAxe`, `Quarterstaff`, `BroadSword`) 추가, `schema.rs`/`lib.rs` 매핑, `tests/content_validation.rs`, LF SHA-256/`PROVENANCE.md`/`registry.rs` 동기화 | `crates/aihack-content/*`, `docs/provenance/r7-content.sha256`, `PROVENANCE.md`, `tests/content_validation.rs` |
| S4-runtime | `systems/{artifacts,skills}.rs` 신설, `altar`/`dip`/`combat`/`spells`/`items`/`vision`/`terrain`/`traps`/`projectiles`/`world`/`campaign`/`campaign_map`/`bootstrap`/`save`/`snapshot`/`session`/`observation` 통합 | `crates/aihack-runtime/*` |
| S4-adapters | `apps/aihack-tui`(13 역할·5 종족·3 성향 생성 UI, `'V'` Invoke, `'X'` Enhance, 제단 정렬 색·분노/종족/성향 HUD) 및 `apps/aihack-headless` (`--role`/`--race`/`--alignment`) 확장 | `apps/aihack-tui/*`, `apps/aihack-headless/*` |
| S4-verify | `tests/stage4_p10.rs` 작성 및 기존 테스트(`stage1_fixes.rs` 등) 호환 검증, 전체 gate(`fmt`, `clippy`, `test -j 2`, `release`, `audit`, `deny`, `r7/r8`) PASS 및 완료 문서 마감 | `tests/*`, `crates/*/tests/*`, `README.md`, `docs/campaign_implementation.md`, `IMPLEMENTATION_SUMMARY.md`, `CHANGELOG.md`, `tasks/todo.md` |

## P10-Audit-Fix 실행 계획 (`audit_report_35.md` 시정, 2026-11-27)

기준: `docs/audit/audit_report_35.md`, `DESIGN_DECISIONS.md` (`ADR-P10-02`), `spec.md`, `docs/campaign_spec.md` (`Stage 4.5`). 미커밋 `Stage 4` 55개 변경분을 보존하고 커밋/푸시는 하지 않는다.

| 단계 | 작업 요약 | 대상 파일 |
| --- | --- | --- |
| AF-W0 | `DESIGN_DECISIONS.md`(`MIN-07` `i16` + `ADR-P10-02` + `GAP-01`/`P12` 아키텍처), `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` D3D 선행 확정 | `DESIGN_DECISIONS.md`, `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` |
| AF-W1 | `CRIT-01`(`playing_actions` 정렬+TUI `legal_actions`), `CRIT-02`(`submit` 롤백+선검증 후변이), `CRIT-03`(지형 공격/`Water|Lava` 제외), `MAJ-01`(`UnicornHorn` 종족), `MAJ-02`(`MAX_LUCK`) | `crates/aihack-runtime/*`, `apps/aihack-tui/src/tui/input.rs` |
| AF-W2 | `MAJ-03`(미식별 색·로그·`LlmObservationView` 마스킹·Headless), `MAJ-04`(60x24 13역할 생성 UI·키/마우스·동적 스탯), `MAJ-06`(`SaveV3` P10 검증·`Hole`·대기 에너지 클램프·`Water`/`Lava` 대기/부유 만료) | `crates/aihack-{core,llm,runtime}/*`, `apps/aihack-{tui,headless}/*` |
| AF-W3 | `MAJ-05`(`monster_resistances` 주입+`WandFire` 면역, `con_regen_reduction`, `weighted_monster_kinds_for_depth` 소비) | `crates/aihack-{core,content,runtime}/*` |
| AF-W4 | `MIN-01`~`MIN-07`(`PutOn` 아뮬렛, TUI 색상/`Apply`/`'V'`/`event_line`, `Stormbringer` LOS, 상자 `ArtifactBlast`, `Throw` 보너스, `FloatingEye=5`, `FallingRock`/`Bat` 에러 전파, Mines 1/2 스폰 분리, `items.toml` 정리, LF SHA-256/`PROVENANCE.md`/`registry.rs`/`campaign_implementation.md` 동기화) | `crates/*`, `apps/*`, `docs/provenance/r7-content.sha256`, `PROVENANCE.md`, `docs/campaign_implementation.md` |
| AF-verify | `tests/audit35_remediation.rs` 작성, 전체 gate(`fmt`, `clippy`, `test -j 2`, `release`, `audit`, `deny`, `r7/r8`, legacy `rg`) PASS 및 완료 문서 마감 | `tests/*`, `docs/*`, `IMPLEMENTATION_SUMMARY.md`, `CHANGELOG.md`, `tasks/todo.md` |

## Stage 5 (P11) 실행 계획 (2026-11-27)

기준: `docs/CODER_HANDOFF_P7_TO_P13.md` Stage 5 (`P11`), `DESIGN_DECISIONS.md` (`ADR-P11-01`), `spec.md`, `docs/campaign_spec.md` (`Stage 5 (P11)`). 레거시 2층 모드·R9 causal 해시·`CommandIntent: Copy`·`SaveV3`·`LlmObservationView < 32 KiB`·`ActionSpace <= 64`를 보존하며 커밋/푸시는 하지 않는다.

| 단계 | 작업 요약 | 대상 파일 |
| --- | --- | --- |
| S5-doc | `DESIGN_DECISIONS.md` (`ADR-P11-01`), `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` D3D 기준 선행 확정 | `DESIGN_DECISIONS.md`, `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` |
| S5-core | `MonsterKind` 17종·`MonsterPassive` 6종·`CommandIntent::{Chat, Pay}`·`EntityPayload::Actor`(`tameness`/`pet_xp`/`pet_hunger`)·`EntityPayload::Item`(`unpaid`)·`WorldState`/`SavedWorldV1`(`shop_debt`)·`EntityStore` 헬퍼·`aihack-ai-contract`/`aihack-llm` 확장 | `crates/aihack-core/*`, `crates/aihack-ai-contract/*`, `crates/aihack-llm/*` |
| S5-content | `monsters.toml` P11 17종 추가 및 기존 인간형 `ai_flags` 보강, `schema.rs`/`lib.rs`(`monster_ai_flags` 포함) 확장, LF SHA-256/`PROVENANCE.md`/`registry.rs`/`content_validation.rs` 동기화 | `crates/aihack-content/*`, `docs/provenance/r7-content.sha256`, `PROVENANCE.md`, `tests/content_validation.rs` |
| S5-runtime | `systems/{pets,npc}.rs` 신설, `movement`(우호 자리바꿈)·`stairs`(펫 동반)·`items`(미결제 청구/반납)·`projectiles`(음식 투척 길들이기)·`shop`(호환+적대 거부)·`monster_ai`(문 열기·아이템 줍기/사용·시선·소환·파벌전)·`combat`(장비 보너스·훔치기+텔레포트·부식·분열·적대화)·`death`(`monster_inventory` 드롭·상점주인 사망 정산)·`bootstrap`·`session`·`observation`·`save`/`campaign` 통합 | `crates/aihack-runtime/*` |
| S5-adapters | `apps/aihack-tui`(`faction` 색상/Inspect, `DEBT:` HUD, `'m'` Chat / `'D'` Pay / `'$'` 메뉴) 및 `apps/aihack-headless`(비적대 비공격·미결제 회피·`Pay` 우선) 갱신 | `apps/aihack-tui/*`, `apps/aihack-headless/*` |
| S5-verify | `tests/stage5_p11.rs` 작성(펫/길들이기/진화/저주회피/상점절도/층이동, 상점주인 채무/도둑질/사제/오라클/경비병, 몬스터 문열기/아이템사용/훔치기/녹/분열/소환/시선+실명방어/오크vs드워프 파벌전, SaveV3 변조/V1 격리/LLM 32KiB/TUI/Headless), 전체 gate PASS 및 완료 문서 마감 | `tests/*`, `docs/*`, `IMPLEMENTATION_SUMMARY.md`, `CHANGELOG.md`, `README.md`, `tasks/todo.md` |

## Stage 6 (P12) 실행 계획 (2026-11-28)

기준: `docs/CODER_HANDOFF_P7_TO_P13.md` Stage 6 (`P12`), `DESIGN_DECISIONS.md` (`ADR-P12-01`), `spec.md`, `docs/campaign_spec.md` (`Stage 6 (P12)`). 레거시 2층 모드·R9 causal 해시·`MAX_XP = 180`·`CommandIntent: Copy`·`SaveV3`·`LlmObservationView < 32 KiB`·`ActionSpace <= 64`·`Cargo.lock` 불변을 유지하며 커밋/푸시는 하지 않는다.

| 단계 | 작업 요약 | 대상 파일 |
| --- | --- | --- |
| S6-doc | `DESIGN_DECISIONS.md` (`ADR-P12-01`), `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` D3D 기준 선행 확정 | `DESIGN_DECISIONS.md`, `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` |
| S6-core | `Resistances` 5플래그·`SpellSchool` 7학파·`SpellKind` 28종·`SPELL_MEMORY_TURNS = 20_000`·`define_item_kinds!`(130종)·`define_monster_kinds!`(85종)·`AmuletEffect` 6종·`RingEffect` 12종·`WandEffect` 18종·`MonsterPassive` 15종·`WishChoice`/`CommandIntent::Wish`·`EntityStore::polymorph_monster`·`aihack-ai-contract`/`aihack-llm` 확장 | `crates/aihack-core/*`, `crates/aihack-ai-contract/*`, `crates/aihack-llm/*` |
| S6-content | `crates/aihack-content/build.rs` 정적 카탈로그 코드 생성, `items.toml`(130종)·`monsters.toml`(85종) 확장, `schema.rs`/`lib.rs`(`damage_small`/`damage_large`·`hp_dice`·`unidentified_pool`) 통합, LF SHA-256/`PROVENANCE.md`/`registry.rs` 동기화 | `crates/aihack-content/*`, `docs/provenance/r7-content.sha256`, `PROVENANCE.md` |
| S6-runtime | `appearance`(풀 확장)·`spells`(마법서 학습/20,000턴 감퇴/실패율/28주문)·`items`(마법서 독서·레벨업/자기변이/매직매핑·텔레포트 제어)·`projectiles`(18완드·소원·몬스터 변이)·`death`(생명보존 부적)·`combat`(크기별 피해·은무기)·`monster_ai`(반사·갈등·신규 패시브·투명/가속)·`session`/`bootstrap`/`observation`/`save` 통합 | `crates/aihack-runtime/*` |
| S6-adapters | `apps/aihack-tui`(동적 28주문 메뉴+실패율, 소원 메뉴, 마법서 `r`) 및 `apps/aihack-headless`(생명보존/반사 아뮬렛·상급 치유 연동) 갱신 | `apps/aihack-tui/*`, `apps/aihack-headless/*` |
| S6-verify | `tests/stage6_p12.rs` 작성(마법서/기억감퇴/실패율/7학파 28주문/18완드/소원/변이/반사/생명보존/갈등/전 도감 130·85 스폰·관찰·TUI·시체 스모크/SaveV3/V1 격리/LLM 32KiB/ActionSpace <= 64), 전체 gate PASS 및 완료 문서 마감 | `tests/*`, `docs/*`, `IMPLEMENTATION_SUMMARY.md`, `CHANGELOG.md`, `README.md`, `tasks/todo.md` |
