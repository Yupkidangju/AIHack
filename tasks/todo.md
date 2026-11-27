# P1–P3 체크리스트

- [x] P1a 타입: Role/CampaignState + world/save/snapshot. 기존 V1/hash 및 성장 공식 검증.
- [x] P1b runtime: StartCampaign(role), 역할 장비/stat, kill XP. 역할 차이·중복 보상·저장 검증.
- [x] P1c UI: 생성 역할 선택·HUD. keyboard/mouse dispatcher 검증.
- [x] P2a 생성: seed별 connected 6+2층. BFS·동일/다른 seed 검사.
- [x] P2b 이동: 계단·분기·재진입 저장. 모든 왕복·replay 검증.
- [x] P3a 목표: 고유 AmuletAscension item. pickup/drop/recover와 forged save 거부.
- [x] P3b 승천: Ascend→Victory→결과→새 게임. 조기/중복 성공 거부와 실제 명령 완주.
- [x] 최종: 기존 478 회귀 포함 총 491 tests, fmt/clippy/release PASS. 증거: `docs/campaign_implementation.md`.

# P4 체크리스트 (2026-11-26)

- [x] P4-doc: `docs/campaign_spec.md`, `spec.md`, `DESIGN_DECISIONS.md` (`ADR-P4-01`), `tasks/plan.md`, `tasks/todo.md` 문서 선행 확정.
- [x] P4a: `TrapKind::Dart/Bear`, `campaign_map` 문/비밀문/함정 배치, `validate_persisted_map` 전이 검증.
- [x] P4b: `Role::Cleric`, 신규 5아이템(`RechargeWand` 포함), 신규 3몬스터, `PROVENANCE.md`·`r7-content.sha256`·`registry.rs` hash 동기화.
- [x] P4c: TUI(`render_map`, `render_panels`, `input`, `labels`, `mod`) 및 Headless(`--role cleric`, `PotionExtraHealing`) 연동.
- [x] P4d: `tests/campaign.rs` 4역할 완주 및 문/비밀문/함정/신규 아이템·몬스터/위조 저장 회귀 테스트.
- [x] P4-verify: `fmt`, `clippy`, `test -j 4`, `release`, `cargo-audit`, `cargo-deny`, `r7_checkpoint`, `r8_checkpoint`, headless smoke & 최종 문서 마감.

# P5 체크리스트 (2026-11-26)

- [x] P5-doc: `docs/campaign_spec.md`, `spec.md`, `DESIGN_DECISIONS.md` (`ADR-P5-01`), `tasks/plan.md`, `tasks/todo.md` 문서 선행 확정.
- [x] P5a: `BucStatus`/`enchantment`, `Entity` 별도 접근자, `DoorState::Locked` 및 `kick_door`/전이 검증, 인챈트·저주 해제 두루마리 3종.
- [x] P5b: `poison_turns`/`DeathCause::Poison`, `PotionCure`, `WandFire`, `GiantSpider`/`GnollArcher`/`Troll`, `PROVENANCE.md`·`r7-content.sha256`·`registry.rs` hash 동기화.
- [x] P5c: TUI(잠긴 문, `POIS:`, BUC/인챈트 라벨, 신규 몬스터/아이템, `DeathCause::Poison`) 및 Headless `survival-v1` 연동.
- [x] P5d: `tests/campaign.rs`, `tests/content_validation.rs`, `tests/campaign_p5.rs` 회귀·통합 테스트 및 전체 검증(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`)·최종 문서 마감.

# P6 체크리스트 (2026-11-26)

- [x] P6-doc: `docs/campaign_spec.md`, `spec.md`, `DESIGN_DECISIONS.md` (`ADR-P6-01`), `tasks/plan.md`, `tasks/todo.md` 문서 선행 확정.
- [x] P6a: `Ring` 슬롯·`RingEffect`·반지 3종·`FireAnt`(`FireOnMelee`), 통합 AC 계산·독/화염 저항·`Cursed` 반지 가드.
- [x] P6b: `TileKind::Shop`, `Main 2`/`Mines 1` 상점, 캠페인 골드(초기 50 + 처치 `diff*25`), `systems/shop.rs` 구매·판매 및 content/provenance 동기화.
- [x] P6c: TUI(`$`/`shop`, `G<gold>`, `RES:F|P|FP`, `a`/`fire ant`, `=`, `'P'`/`'T'`/`'$'` 메뉴) 및 Headless `survival-v1` 연동.
- [x] P6d: `tests/content_validation.rs`, `tests/campaign.rs`, `tests/campaign_p6.rs` 테스트 및 전체 검증(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`)·최종 문서 마감.

# P7 체크리스트 (2026-11-26)

- [x] P7-doc: `docs/campaign_spec.md`, `spec.md`, `DESIGN_DECISIONS.md` (`ADR-P7-01`), `tasks/plan.md`, `tasks/todo.md` 문서 선행 확정.
- [x] P7a: `TileKind::Altar`, `mp`/`max_mp`, `SpellKind::{ForceBolt, Heal, Detect}`, `Offer`/제단 `Pray`/`Cast`.
- [x] P7b: 시드별 미식별 외형(`appearance_for_kind`)·자동 식별 및 `quantity` 스택(`1..=99`)·병합·분리 투척.
- [x] P7c: `Minotaur`/`Dragon`(`FireBreath`), `ascension_pursuit_stage` 귀환 추격, content/provenance/LLM 동기화.
- [x] P7d: TUI/Headless, `tests/campaign.rs`/`tests/campaign_p7.rs`, 전체 gate(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`)·최종 문서 마감.

# Stage 1 (P7-Fix) 체크리스트 (2026-11-27)

- [x] S1-doc: `spec.md`, `docs/campaign_spec.md`, `DESIGN_DECISIONS.md` (`ADR-P7-02`), `tasks/plan.md`, `tasks/todo.md` 문서 선행 확정.
- [x] DEF-01: 방어구·반지 AC 부호 역전 해결 및 NetHack식 하강 AC(기본 10, `defense = 20 - ac`, 12종 몬스터·HUD·생성 화면·provenance·테스트) 전면 통일.
- [x] DEF-02: 인벤토리 52슬롯(`a-zA-Z`) 확장, 빈 문자 재할당, `Buy`/`Pickup` 스택 병합 허용 및 저장 검증기 52슬롯 정렬.
- [x] DEF-03: `ScrollLevelTeleport` 아뮬렛 소지 시 차단 및 비소지 시 분기 내 다른 유효 층 이동(legacy 2층 유지).
- [x] DEF-04: 허기(`nutrition`) 런타임 효과(`Weak` `-2` 명중, `Fainting` 마비, `nutrition <= -150` `DeathCause::Starvation`) 및 Headless `survival-v1` 식량 섭취 연결.
- [x] DEF-05: 캠페인 자연 HP 회복(`poison_turns == 0 && nutrition > 50`, `(15 - level).max(5)`턴 주기 `+1 HP`) 구현.
- [x] DEF-06: `buc_known`/`enchant_known` 미식별 정보 은닉 및 TUI 상점 메뉴(`'$'`) 미식별 외형·구매 가격 표시.
- [x] DEF-07: 탐험 기억(`GameLevel.explored`, `Observation` 기억 타일, TUI `muted` Fog of War, `LlmObservationView` 32 KiB 안전 필터) 구현.
- [x] DEF-08: `aihack-content` 단일 `item_id` 매핑 및 `"locked"` 문·`"dart"`/`"bear"` 함정 레벨 로더 보완·단위 테스트.
- [x] DEF-09: `character_creation_lines()` 정적 스탯 제거/정합화, `hallucination_message` TUI 로그 연결 및 `P8/P9` 보류 항목 `spec.md` 명시.
- [x] DEF-10: 두루마리 BUC 분기(`EnchantWeapon`/`EnchantArmor`/`RemoveCurse`) 및 `CommandIntent::Identify { scroll, target }` 대상 지정 감정 구현.
- [x] DEF-11: `P7` + Stage 1 전체 gate(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`) PASS 및 문서(`README.md`, `docs/campaign_implementation.md`, `IMPLEMENTATION_SUMMARY.md`, `CHANGELOG.md`, `tasks/todo.md`) 동기화 마감.

# Stage 2 (P8) 체크리스트 (2026-11-27)

- [x] S2-doc: `DESIGN_DECISIONS.md` (`ADR-P8-01`), `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` 문서 기준 선행 확정.
- [x] S2-core: `BranchId`, `SAVE_SCHEMA_VERSION_V3`, `Resistances`/`StatusTimers`, 10 슬롯(`EquipmentSlot`/`Inventory`), 6대 능력치(`Attributes`), `GameLevel` 조명/특수방/각인, `Faction`/인벤토리/아티팩트 훅, `TakeOff` 및 `Observation` 확장.
- [x] S2-content: `items.toml`/`monsters.toml` P8-A 메타데이터 및 신규 장비/물약/`Bat` 추가, `speed 0..=36` 및 슬롯 검증 확장, `PROVENANCE.md`·`r7-content.sha256`·`registry.rs` (`cf2b7230608c64d2`) 동기화.
- [x] S2-runtime: `80×21` Generator v3 + 구조적 `validate_persisted_map` + `BranchGraph`, 10 슬롯/AC/저항, 에너지 버킷 스케줄러, 6대 능력치, 상태이상 효과, 일반화 시체(부패/저항/트롤 부활), `SaveV3` 검증.
- [x] S2-tui: `render_map`/`render_panels`/`input`/`play_menu` 데이터 주도 전환 및 6대 능력치·저항·상태·장비 해제(`'x'` 메뉴) HUD/메뉴 연동.
- [x] S2-verify: 기존 캠페인/콘텐츠 테스트 동기화 및 `tests/stage2_p8.rs` (**7 PASS**) 작성, 전체 gate(`fmt`, `clippy`, `test -j 2`: **532 PASS / 0 FAIL / 0 ignored**, 96 test binaries, `release`, `audit`, `deny`, `r7/r8`) PASS 및 최종 문서 마감.

# Stage 3 (P9) 체크리스트 (2026-11-27)

- [x] S3-doc: `DESIGN_DECISIONS.md` (`ADR-P9-01`), `spec.md`, `docs/campaign_spec.md`, `tasks/plan.md`, `tasks/todo.md` 문서 기준 선행 확정.
- [x] S3-core: `TileKind` 5지형·`TrapKind` 12함정·`DeathCause` 3종·`ItemKind` 13종·`InContainer`·`CommandIntent` 6명령·`Observation`/LLM 직렬화 확장.
- [x] S3-content: `items.toml` P9 13아이템 추가(`b45ed709b09768670536e1d6a65e171c1787104ecd3078ee1e696e6470b340a3`), `schema.rs`/`lib.rs` 확장 및 `PROVENANCE.md`·`r7-content.sha256`·`registry.rs`(`055379afe655af12`)·`content_validation.rs` 동기화.
- [x] S3-runtime: `tools`/`containers`/`dip`/`engrave`/`terrain` 시스템 구현, `traps`(12종)/`vision`/`monster_ai`(Elbereth/수면회피)/`combat`/`items`/`altar`/`shop`/`campaign_map`/`bootstrap`/`save`(양방향 컨테이너/녹/P9 검증) 통합.
- [x] S3-adapters: TUI(`a`/`C`/`v`/`E`/`g`, 지형/함정/각인/녹 라벨) 및 Headless(`UnicornHorn` 치료, `Lava` 회피) 연동.
- [x] S3-verify: `tests/stage3_p9.rs`(**5 PASS**) 및 `tests/campaign.rs`(`route()`) 정렬, 전체 gate(`fmt`, `clippy`, `test -j 2`: **537 PASS / 0 FAIL / 0 ignored**, 97 test binaries, `release`, `audit`, `deny`, `r7/r8`) PASS 및 최종 문서 마감.
