# AIHack Design Decisions v2

> Archive chain
> - Latest: `.archive/DESIGN_DECISIONS_archive_260715.md`
> - Previous: first archive
>
> ADR-0001..ADR-0020과 과거 중복 번호는 아카이브에 보존한다. 이 파일은 v0.3.0 활성 결정만 포함한다.

문서 상태: active decisions
작성일: 2026-07-15
기준: `spec.md`

Accepted는 계획 승인을 뜻하며 구현 완료를 뜻하지 않는다. 아카이브의 과거 결정과 충돌하면 이 파일과 `spec.md`를 적용한다.

## ADR-P12-01: Stage 6 (P12) 마법 체계 심화 · 대규모 도감 투입 · build.rs 데이터 주도 전환 계약 (Accepted)

- **Decision Date**: 2026-11-28
- **Context**: `Stage 5 (P11)` 완료 시점에서 AIHack은 55종 아이템·30종 몬스터·3종 주문(`ForceBolt`/`Heal`/`Detect`)·2종 완드(`MagicMissile`/`Fire`)·3종 반지·1종 아뮬렛을 보유하고 있으며, 신규 아이템/몬스터 추가 시 여러 크레이트에 수동 `match`가 분산되어 있었다. `docs/CODER_HANDOFF_P7_TO_P13.md` Stage 6 (`P12`) 및 `ADR-P10-02` 6항의 스케일업 결정에 따라 (1) `build.rs` 정적 카탈로그 코드 생성과 선언적 매크로 단일 테이블로 수동 `match` 복제를 제거하고 `unidentified_pool`·`damage_small`/`damage_large`·`hp_dice`를 런타임 소비하며, (2) 마법서(`+`) 학습·20,000턴 기억 감퇴·실패율 계산·7학파 28종 주문을 구현하고, (3) 18종 완드(`Polymorph`/`Death`/`Digging`/`Teleport`/`Cancellation`/`Wishing` 포함)·12종 반지·6종 아뮬렛(`LifeSaving`/`Reflection`/`MagicalBreathing`/`Unchanging` 포함)·5종 확장 저항을 구현하며, (4) **130종 아이템 · 85종 몬스터** 도감을 투입해야 한다. 동시에 레거시 2층 모드(`campaign.is_none()`, `SAVE_SCHEMA_VERSION_V1 = 1`), `tests/long_run.rs` R9 causal 해시, `MAX_XP = 180`, `CommandIntent: Copy`, `SAVE_SCHEMA_VERSION_V3 = 3`, `LlmObservationView < 32,768B`, `ActionSpace <= 64`, 무추가 외부 의존성(`Cargo.lock` 불변)을 보존한다.
- **Decision**:
  1. **하이브리드 `build.rs` 정적 카탈로그 + 선언적 매크로 단일 테이블 (`Cargo.lock` 불변)**:
     - `crates/aihack-content/build.rs`(표준 라이브러리만 사용)가 `items.toml`/`monsters.toml`을 빌드 시 파싱하여 `$OUT_DIR/generated_catalog.rs`에 `GENERATED_ITEM_COUNT`, `GENERATED_MONSTER_COUNT`, `GENERATED_CANONICAL_ITEM_KINDS`, `GENERATED_ITEM_APPEARANCE_POOLS`, `GENERATED_MONSTER_IDS`를 생성한다.
     - `crates/aihack-core`는 `define_item_kinds!` / `define_monster_kinds!` 선언적 매크로 단일 테이블로 `ItemKind`(130종)·`MonsterKind`(85종)·`toml_id`/`from_toml_id`·`is_stackable`·`has_randomized_appearance`·`is_legacy_v1_allowed`·`ai_kind`·`difficulty`·`natural_attack_name`을 일괄 생성하여 모든 수동 ID/명칭 `match`를 제거하고 `Copy` + 기존 serde variant 이름을 보존한다.
     - `unidentified_pool`(외형 풀 매핑), `damage_small`/`damage_large`(몬스터 크기별 무기 피해), `hp_dice`(변이/소환 시 몬스터 HP 결정론적 롤)를 런타임에서 소비한다.
  2. **마법서(`ItemClass::Spellbook`, `'+'`) 학습 · `SPELL_MEMORY_TURNS = 20_000` 기억 감퇴 · 실패율**:
     - `SpellSchool` 7학파(`Attack`, `Healing`, `Divination`, `Enchantment`, `Clerical`, `Escape`, `Matter`)와 `SpellKind` 28종을 정의한다.
     - `CommandIntent::Read`로 마법서를 읽으면(Blessed 확정 습득/갱신, Uncursed `Int`·레벨·`Spellcasting` 숙련도 판정, Cursed 혼란/실명/피해 역풍) `world.learned_spells`에 `LearnedSpell { spell, learned_at_turn }`을 기록·갱신하며, `charges == 0` 시 책이 소멸한다.
     - 비고유(non-innate) 습득 주문은 `turn - learned_at_turn < 20_000` 동안만 기억되며 20,000턴 경과 후 시전 시 기억 만료로 거부되고 마법서 재독서로 갱신된다.
     - `spell_failure_percent(world, spell)`: 장착된 금속 방어구/방패(`iron`/`steel`/`mithril`) 패널티, `Int`, `campaign.level()`, `Spellcasting` 숙련도, `spell_level(1..=7)`로 실패율(`0..=100%`)을 산출한다(단, 기존 직업 고유 시작 주문 + 비금속 무장 상태는 `0%`를 유지해 P7~P11 호환성을 보존한다). 시전 실패 시 MP 절반(최소 1) 소비·소량 숙련도 획득 후 주문이 불발한다.
  3. **18종 완드 · 소원(`CommandIntent::Wish: Copy`) · 12종 반지 · 6종 아뮬렛 · 5종 확장 저항**:
     - `Resistances` 비트셋(기존 `u16`)에 `REFLECTION`(bit 8), `MAGIC_RESISTANCE`(bit 9), `CONFLICT`(bit 10), `TELEPORT_CONTROL`(bit 11), `SEARCHING`(bit 12)을 추가한다.
     - `CommandIntent::Wish { item, wish: WishChoice }` (`CURATED_WISHES` 고정 enum으로 `CommandIntent: Copy` 보존)를 추가하며 `WandWishing` 충전을 소비해 아이템(축복/인챈트 포함) 또는 미생성 고유 아티팩트를 소환하고 `AmuletAscension`/`WandWishing`/`|enchant| > 3` 금지 소원을 거부한다.
     - `WandPolymorph`는 살아 있는 비고유 NPC 몬스터를 다른 `MonsterKind`로 변이(`EntityStore::polymorph_monster` + `hp_dice`)시키고 `AmuletUnchanging`은 자가 변이를 차단한다.
     - `AmuletLifeSaving`은 치명 피해 직전 자동 발동해 아뮬렛을 소멸시키고 플레이어 HP를 `max_hp`까지 회복·상태이상을 해제하며, `AmuletReflection`/`ArmorShieldReflection`/`ArmorSilverDragonScale`의 `REFLECTION`은 브레스·시선(`Medusa`/`FloatingEye`)·공격 완드를 공격자에게 반사한다. `RingConflict`는 주변 몬스터 간 상호 공격을 유발한다.
  4. **대규모 도감(130 아이템 · 85 몬스터) · 고난도 스폰 격리 · SaveV3/LLM/TUI/Headless**:
     - `items.toml` 130종(기존 55 + P12 75종) 및 `monsters.toml` 85종(기존 30 + P12 55종)을 투입한다. 기존 55+30 TOML 라인과 기존 `WandMagicMissile`/`WandFire` 비외형 계약, `MAX_XP = 180`을 그대로 보존하며, `difficulty >= 6` 고난도 몬스터는 `spawn_depth_range = [6, 20]`으로 배치해 `Main 2..=5` 기존 경로 호환성을 지킨다.
     - `save.rs`는 `is_legacy_v1_allowed()` 및 `Spellbook`/`Gem` 거부를 통해 레거시 V1과 R9 causal 해시를 불변으로 격리한다. `CampaignObservation`에 `learned_spells`/`spell_failure_rates`를 추가하되 `LlmObservationView` 32,768B 및 `ActionSpace` Tier 1~3 `<= 64` 상한을 유지한다.
- **Alternatives Considered**:
  - `Cargo.lock`을 변경하여 외부 `toml` 크레이트를 `build-dependencies`에 추가하는 안 → `Cargo.lock` SHA/license 감사 범위를 불필요하게 건드리므로 `build.rs`는 `std`만 사용해 무의존성 정적 코드 생성을 구현한다.
  - `CommandIntent::Wish`에 임의 `String`을 넣는 안 → `CommandIntent: Copy` 불변조건이 깨지므로 `WishChoice` enum(`Copy`) + `CURATED_WISHES`를 사용한다.
- **Outcome**: 7학파 28주문·마법서 학습/감퇴/실패율·18완드/소원/변이·12반지·6아뮬렛·130아이템·85몬스터 도감이 `build.rs`+매크로 데이터 주도 계층과 기존 V1/V3/LLM/TUI/Headless 계약 안에서 통합 완성된다.

## ADR-P11-01: Stage 5 (P11) 살아 있는 던전 — 펫·NPC·상점 채무·고급 몬스터 AI 계약 (Accepted)

- **Decision Date**: 2026-11-27
- **Context**: `Stage 4 (P10)` 및 `P10-Audit-Fix` 완료 후, 던전 내 비플레이어 행위자는 여전히 단순 추적/원거리 적대 몬스터와 타일 메뉴 상점에 머물러 있다. `docs/CODER_HANDOFF_P7_TO_P13.md` Stage 5 (`P11`) 요구사항에 따라 반려동물(펫) 동반·길들이기·성장·진화·저주 회피·상점 절도, 실제 NPC(`Shopkeeper`, `TemplePriest`, `Oracle`, `Watchman`)와 바닥 진열 상품·채무·도둑질 적대화, 고급 몬스터 AI(`ai_flags` 소비: 문 열기, 아이템 줍기/장착/사용/사망 드롭, 훔치기+텔레포트, 금속 부식, 분열, 소환, 실명 방어 시선 공격, 오크 vs 드워프 파벌전)를 구현해야 한다. 동시에 레거시 2층 모드(`campaign.is_none()`, `SAVE_SCHEMA_VERSION_V1 = 1`, `tests/long_run.rs` R9 causal 해시), `CommandIntent: Copy`, `SAVE_SCHEMA_VERSION_V3 = 3`, `LlmObservationView < 32,768B`, `ActionSpace <= 64`, 기존 `Buy`/`Sell` 호환성을 보존해야 한다.
- **Decision**:
  1. **P11 몬스터·동료·NPC 17종 및 `MonsterPassive` 6종 추가 (`monsters.toml`, 총 30종)**:
     - 동료/진화 6종: `Kitten -> Housecat`, `LittleDog -> Dog`, `Pony -> Warhorse` (`spawn_weight = 0`, `pet_capable`).
     - 평화 NPC 4종: `Shopkeeper` (`Main 2`, `Mines 1`), `TemplePriest` (`Main 3`), `Oracle` (`Main 4`), `Watchman` (`Mines 1`).
     - 특수·파벌 몬스터 7종: `Leprechaun`(`StealGold`), `Nymph`(`StealItem`), `RustMonster`(`RustMetal`), `BlackPudding`(`SplitOnHit`), `Dwarf`(`hates_orcs`), `Medusa`(`GazeParalyze`), `OrcCaptain`(`SummonAllies`). 기존 인간형(`Goblin`/`Kobold`/`Orc`/`Ogre`/`GnollArcher`)에도 `ai_flags`(`opens_doors`, `picks_up`, `uses_items`, `orc`/`hates_dwarves`)를 부여한다.
     - 레거시 2층 모드(`campaign.is_none()`)는 P11 전용 몬스터·비적대 `Faction`·P11 AI 분기를 일체 실행하지 않고 세이브 로드 시 거부한다.
  2. **확장 명령 (`CommandIntent: Copy` 유지) 및 Optional Serde 스키마 (`SaveV3` 유지)**:
     - `CommandIntent::Chat(Direction)`(인접 NPC·펫 대화, TUI `'m'`)과 `CommandIntent::Pay`(상점 채무 정산·사제 기부·오라클 상담, TUI `'D'` 및 `'$'` 상점 메뉴 병행) 2종을 추가한다. `playing_actions()` Tier 1/2에 배치해 `ActionSpace <= 64` 예산 내 접근성을 보장한다.
     - `EntityPayload::Actor`에 optional serde 필드 `tameness: u8`, `pet_xp: u32`, `pet_hunger: i16`, `EntityPayload::Item`에 `unpaid: bool`, `WorldState`/`SavedWorldV1`에 `shop_debt: u32`, `PlayerObservation`에 `shop_debt: u32`, `EntityObservation`/`LlmVisibleEntity`에 `faction: Option<Faction>`, `ItemObservation`/`LlmInventoryItem`에 `unpaid: bool`을 추가한다(모두 기본값 시 직렬화 생략).
  3. **동료(Pet) 시스템 및 우호 자리바꿈**:
     - 캠페인 시작 시 `Main 1` 플레이어 인접 빈 바닥에 역할/종족 기반 시작 펫(`Knight`/`Valkyrie`/`Samurai`→`Pony`, `Mage`/`Wizard`/`Rogue`/`Archeologist`/`Elf`→`Kitten`, 그 외→`LittleDog`; `Faction::Pet`, `tameness = 5`, `pet_hunger = 600`)을 배치한다. 기존 캠페인 호환 테스트의 엔티티 순서를 보존하기 위해 시작 펫과 NPC는 던전 적대 몬스터 배치 이후에 스폰한다.
     - 플레이어가 인접 `Pet`/`Peaceful`/`Shopkeeper` 타일로 `Move`하면 공격 대신 위치를 맞바꾸는 우호 자리바꿈(friendly displacement)을 수행해 통로 막힘을 방지한다.
     - 펫은 매 턴 허기 감산(0 도달 시 친밀도 하락 후 야생화 `Hostile`), 바닥/투척 음식 섭취 및 야생 `pet_capable` 동물 길들이기(`Throw` 음식은 피해를 주지 않고 먹이기/길들이기), 적대 몬스터 교전 및 `pet_xp >= 20` 진화(`Kitten->Housecat`, `LittleDog->Dog`, `Pony->Warhorse`), 바닥 저주 아이템 진입 회피 및 `buc_known = true` 경고, 상점 `unpaid` 상품 운반 후 상점 밖 플레이어 발밑 드롭(`unpaid = false` 도둑질), 계단/분기(`Descend`/`Ascend`/`EnterBranch`) 동반 이동(체비셰프 거리 `<= 2`)을 수행한다.
  4. **실제 NPC·상점 채무 경제 및 기존 `Buy`/`Sell` 호환**:
     - `Main 2`와 `Mines 1` 상점에 `Shopkeeper` NPC와 `unpaid = true` 바닥 진열 상품 3종을 배치한다(`tour` 테스트의 물약 수집과 충돌하지 않는 비-물약 유용 품목 위주). `unpaid` 아이템 `Pickup` 시 가격만큼 `world.shop_debt`가 누적되고(유료 스택과 비병합), 상점 내 `Drop` 시 차감되며, `Pay`로 정산하면 `shop_debt = 0` 및 `unpaid = false`가 된다. `shop_debt > 0` 상태로 상점 구역(거리 > 4)을 벗어나거나 계단을 이용하거나 평화 NPC/상점주인을 공격하면 상점주인 및 같은 층 `Watchman`이 `Faction::Hostile`로 전환된다.
     - 기존 `TileKind::Shop` 위 `Buy`/`Sell` 명령은 같은 층에 적대화된 `Shopkeeper`가 없는 한 그대로 호환 동작한다.
     - `TemplePriest`(`Main 3`) `Pay` 기부는 신의 분노 감소·상태이상 치료·행운 증가·아이템 감정/치유를 제공하고, `Oracle`(`Main 4`) `Chat`/`Pay`는 결정론적 힌트와 주요 상담(감정/탐지)을 제공한다.
  5. **고급 몬스터 AI·특수 패턴·파벌전 (`ADR-P10-02` `ai_flags` 소비)**:
     - `opens_doors`: 추적 경로의 닫힌 문(`DoorState::Closed`)을 연다.
     - `picks_up`/`uses_items`: 적대/경비 몬스터가 바닥 무기·방어구·치유 물약·공격 완드를 `monster_inventory`(최대 4개, 아뮬렛/컨테이너/`unpaid` 제외)에 주워 공격/AC 보너스를 얻고, 체력 절반 이하 시 치유 물약 음용·시야선 공격 완드 발사를 수행하며, 사망 시 `monster_inventory` 전량을 바닥에 드롭한다.
     - 특수 패턴: `Leprechaun`(금화 절도+안전 바닥 텔레포트), `Nymph`(비-아뮬렛 아이템 절도→`monster_inventory` 보관+텔레포트, 처치 시 회수), `RustMonster`(비-아티팩트 철/강 장비 `erosion +1`), `BlackPudding`(피격 시 HP `>= 6`이면 체력 반분 클론 분열), `Medusa`/`FloatingEye` 시선 공격(`status_timers.blind > 0` 또는 착용 `Blindfold`로 완전 방어), `OrcCaptain`(인접 빈 바닥에 `Orc` 소환, 층당 상한 4).
     - 파벌전: `orc`/`hates_dwarves`와 `dwarf`/`hates_orcs` 몬스터가 서로 교전하며, `Watchman`은 같은 층 `Hostile` 몬스터를 단속하고 `Pet`은 적대 몬스터와 교전한다.
  6. **`SaveV3` 검증·TUI·Headless 정렬**:
     - `save.rs`에서 `tameness <= 20`, `pet_xp <= 1000`, `pet_hunger`(`0..=1500`), `shop_debt <= 1_000_000`, 플레이어/비플레이어 `Faction` 정합성, `unpaid` 금지 위치(`AmuletAscension`/`Consumed`/`InContainer`), 레거시 모드 P11 메타/몬스터 거부를 검증한다.
     - TUI는 `EntityObservation.faction` 기반 색상/Inspect 표시, HUD `DEBT:` 및 펫/NPC 상태, 충돌 없는 `'m'`(Chat)/`'D'`(Pay) 키를 제공하고, Headless `survival-v1`은 캠페인에서 `Faction::Hostile`만 공격하며 `unpaid` 무단 픽업을 피하고 채무 발생 시 `Pay`를 우선한다.
- **Alternatives Considered**:
  - 기존 `Buy`/`Sell`을 제거하고 바닥 픽업+`Pay`만 남기는 안 → `tests/campaign_p6.rs`, `tests/stage2_p8.rs`, `tests/audit35_remediation.rs` 등 기존 상점 계약이 불필요하게 깨지므로, 평화 상점주인/상점 타일에서 `Buy`/`Sell`을 유지하면서 바닥 진열+`shop_debt`+`Pay`+도둑질 시스템을 통합한다.
  - 별도 `CommandIntent::Feed { item, direction }`를 추가하는 안 → `ActionSpace <= 64` 조합을 늘리지 않고 기존 `Throw`/`Drop` 식량 상호작용으로 먹이기·길들이기를 통합하는 편이 NetHack 관례와 예산 제약 모두에 부합한다.
- **Outcome**: 펫·평화 NPC·상점 채무/도둑질·고급 몬스터 AI 및 파벌 생태계가 `GameSession::submit` 단일 결정론 트랜잭션·`SaveV3`·TUI·Headless·LLM 예산 불변조건 안에서 완성된다.

## ADR-P10-02: 감사 보고서 35 시정(P10-Audit-Fix) — 트랜잭션 롤백·ActionSpace 정렬·미식별 은닉·데이터 주도 전환·P11~P13 이월 범위 (Accepted)

- **Decision Date**: 2026-11-27
- **Context**: `docs/audit/audit_report_35.md` 중간 감사에서 `Stage 1 (P7-Fix)` ~ `Stage 4 (P10)` 사이 차단 결함 3건(`CRIT-01`~`CRIT-03`), 주요 결함 6건(`MAJ-01`~`MAJ-06`), 경미 결함 7건(`MIN-01`~`MIN-07`), 단계 간 이월 범위 명시 항목(`GAP-01`)이 확인됐다.
- **Decision**:
  1. **`CRIT-01` (`ActionSpace` 우선순위 정렬 및 TUI `legal_actions` 전환)**: `observation::playing_actions()`는 공간·생존·핵심 단발 명령(Tier 1: `Move`/`Wait`/`Search`/`Pickup`/`Descend`/`Ascend`/`EnterBranch`/`Drink`/`Pray`/`ShowInventory`/`Open`/`Close`/`Kick`/`Enhance`/`Invoke`, Tier 2: `Quaff`/`Eat`/`Read`/`Wield`/`Wear`/`PutOn`/`RemoveRing`/`TakeOff`/비방향 `Apply`/`Cast`/`Buy`/`Sell`/`Offer`/손가락 `Elbereth`)을 앞쪽에 배치하고 조합 폭발 명령(Tier 3: 방향성 `Cast`/`Apply`, `Throw`, `Zap`, `Drop`, 비-`Elbereth` 각인, `ContainerPut`/`Take`, `Dip`, `Identify`)을 후순위로 둔다. LLM 요청 예산용 `action_space.commands` 64개 상한은 유지하되, TUI `input.rs` (`key_to_candidate`, `command_panel_ctas`, `inventory_panel_ctas`)는 전체 `observation.legal_actions`를 기준으로 동작한다.
  2. **`CRIT-02` (트랜잭션 롤백 일원화 및 선검증 후변이)**: `GameSession::submit()`은 `transaction.was_aborted()`뿐 아니라 `!outcome.accepted`인 모든 거부 명령에서 working copy를 롤백한다. `items::read`, `containers::container_take`, `dip::dip_into_tile`/`dip_into_potion`, `engrave::engrave`는 모두 대상/인벤토리 여유/각인 상한을 먼저 검증한 뒤 소비·함정·MP·충전을 변경한다(저주 안대 탈착/소비 거부 시 `Err` 전 사전 `buc_known` 노출 부수효과는 제거하고, 장착 시점에 `buc_known = true`를 기록한다).
  3. **`CRIT-03` (P9 지형 근접 공격 및 추격 스폰 안전화)**: `movement::is_walkable_or_attackable_destination`은 `TileKind::is_movement_passable()`과 동일 기준을 적용하고, `ascension::find_pursuit_spawn_pos`는 `Water | Lava`를 제외한다.
  4. **`MAJ-03` (미식별 아이템 정보 유출 차단)**: `appearance::color_for_appearance`를 도입해 캠페인 미식별 아이템의 맵/인벤토리 `color`를 시드 외형 문자열 기반 색상으로 마스킹한다. `containers`/`dip`/`terrain`/`shop` 로그는 실제 이름/`Debug` 대신 `appearance::display_name_for_*`를 출력하며(연금술 결과 역시 미식별 상태면 외형명 출력), `LlmObservationView`는 미식별 캠페인 엔티티·인벤토리의 `kind`를 `None`으로 마스킹(`skip_serializing_if = "Option::is_none"`)한다. `HeadlessPolicy::SurvivalV1`은 캠페인 모드에서 `identified || appearance.is_none()`인 아이템만 약물/반지 효과를 전제로 사용한다(레거시 2층 모드 불변).
  5. **`MAJ-04` (`60x24` 호환 13역할 TUI 생성 화면)**: `character_creation_lines_with_selection`을 최대 폭 58열·높이 22줄 이내로 재구성하고 충돌 없는 역할 키(`1..=9`, `b` Barbarian, `g` Ranger, `m` Monk, `k` Archeologist)와 `[R]` Race·`[A]` Align·`[Tab]` Role 순환, 동적 HP/MP/AC/능력치 프리뷰, 13역할/옵션 전체 마우스 CTA 및 역할 전환 시 호환 종족·성향 자동 스냅을 구현한다.
  6. **`MAJ-05` (`monster_resistances`/`con_regen_reduction`/`spawn_weight` 즉시 연결 및 `P12` 데이터 주도 전환 전략)**: 즉시 `MonsterTemplate.resistances`와 `ActorStats.resistances`에 `monster_resistances()`를 주입해 `WandFire` 등 원소 저항 면역/경감을 연결하고, `session.rs` 재생 주기를 `con_regen_reduction()`으로 교정하며, `weighted_monster_kinds_for_depth`로 `spawn_depth_range`/`spawn_weight`를 스폰에 연결한다. **향후 `P12` 스케일업(`80~150+` 몬스터, `120~200+` 아이템, `25~40` 주문) 대비 아키텍처**: `P12` 진입 전 TOML → `build.rs` 정적 데이터 테이블 코드 생성 + 데이터 기반 디스패치 하이브리드 구조를 채택해 수동 enum `match` 복제를 제거하되 `CommandIntent: Copy` 불변조건은 유지한다(`ai_flags`/`attacks[]`는 `P11`, `damage_small`/`damage_large`/`hp_dice`/`unidentified_pool`은 `P12`에서 런타임 소비).
  7. **`MAJ-06` (`SaveV3` 엄격 검증 및 엣지 케이스)**: `god_anger`(`0..=100`), `prayer_timeout`(`0..=2000`), `artifact_invoke_cooldown`(`0..=25`), `learned_spells`(중복 금지·상한 `32`), `ascension_altars`(중복 금지·상한 `8`·유효 층/좌표), `monster_inventory`(플레이어 비허용·중복/외부참조/비아이템/아뮬렛/`InContainer`/`Consumed` 차단), 비시체 `corpse_monster` 차단, 아티팩트 `erosion > 0` 차단을 검증한다. `TrapKind::Hole` 사망 시 층 이동 전에 즉시 종료하고, 고속 몬스터 `Wait` 시 잔여 에너지를 `ENERGY_THRESHOLD - effective_gain` 이하로 제한하며, 비이동 턴 또는 `Levitation` 만료 직후 `Water`/`Lava` 정지 피해를 적용한다.
  8. **`GAP-01` (이월 범위 명시)**: 현재 캠페인은 8층(`Main 1..6` + `Mines 1..2`), `MAX_XP = 180` (Lv 10), 간소화 `Polymorph`(상태/STR 감소), 비호환 `SaveV2`(`ADR-P8-01`)를 유지하며, `Sokoban`/`Quest` 확장은 `P11`, `MAX_XP`/변신 확장은 `P12`, `Gehennom`/`Main 1..20+`/승천은 `P13`에서 구현한다.
- **Outcome**: 감사 보고서 35의 `CRIT-01`~`CRIT-03`, `MAJ-01`~`MAJ-06`, `MIN-01`~`MIN-07`, `GAP-01` 시정 계약과 `P11~P13` 스케일업·이월 경로가 문서와 런타임에 일관되게 고정된다.

## ADR-P10-01: Stage 4 (P10) 성향·종족·신앙·고유 아티팩트·13역할·#enhance 숙련도 계약 (Accepted)

- **Decision Date**: 2026-11-27
- **Context**: Stage 3 (`P9`)까지 4종 역할(`Knight`, `Scout`, `Mage`, `Cleric`), 단일 성향 제단, 기본 기도/헌납, 제한된 무기군 위주로 동작해 NetHack 스타일의 성향·종족 빌드 다양성, 제단/신앙 리스크-보상, 고유 아티팩트 획득과 `#invoke` 발동, 13종 역할, `#enhance` 숙련도 성장 루프가 부재하다.
- **Decision**:
  1. **3 성향(`Lawful`/`Neutral`/`Chaotic`) · 5 종족(`Human`/`Elf`/`Dwarf`/`Gnome`/`Orc`) · 13 역할(`Role`)**: `CampaignState`(`Copy` 유지)에 `race: Race`, `god_anger: i16`, `prayer_timeout: u16`, `artifact_invoke_cooldown: u16`, `unarmed_bonus` 연계 필드를 확장한다. `Role`을 13종(`Knight`, `Scout`, `Mage`, `Cleric` + `Valkyrie`, `Wizard`, `Rogue`, `Priest`, `Samurai`, `Barbarian`, `Ranger`, `Monk`, `Archeologist`)으로 확장하되 기존 4역할 × `Human` 기본 성향의 시작 능력치/장비/주문/AC 계약은 회귀 없이 보존한다. `Role × Race × Alignment` 유효성 검증(`Role::is_valid_combo`)과 종족 능력치/HP/MP 보정·내재 저항(`Elf`=Sleep, `Orc`=Poison)·암시(`Elf`/`Dwarf`/`Gnome`/`Orc`, 어두운 방 내 거리 8 시야선 내 살아 있는 몬스터 감지)를 적용한다.
  2. **신앙·제단 심화**: `GameLevel::altar_alignments: Vec<(Pos, Alignment)>`로 제단별 성향을 분리하고(`Main 3`=플레이어 시작 성향, `Mines 2`=교차 성향 `cross_alignment`), 비정렬(`cross-aligned`) 제단 기도 시 `god_anger` 누적과 신벌(`DeathCause::DivineWrath`), 기도 재사용 대기시간(`prayer_timeout`) 위반 처벌, 비정렬 제단 시체 `#offer` 시 제단 개종, 동정렬 제단 시체 `#offer` 시 분노 해소 및 고헌신(`luck >= 2 || difficulty >= 2`) 조건부 고유 아티팩트 하사, 비정렬 제단 `#dip` 저주/오염을 구현한다.
  3. **고유 아티팩트 6종 & `#invoke`**: 무기 5종(`LongSword`, `Katana`, `BattleAxe`, `Quarterstaff`, `BroadSword`)을 `items.toml`에 추가하고 6종 고유 아티팩트(`excalibur`, `sunsword`, `magicbane`, `cleaver`, `stormbringer`, `sting`)를 전역 1회 생성(`WorldState::generated_artifacts`), `Lawful` 레벨 5+ 분수 `#dip` `Excalibur` 변환, 비정렬 아티팩트 반발 피해(`DeathCause::ArtifactBlast`), 내구/부식 면역(`erosion_proof`), 패시브 저항/광원(`Sunsword`), `CommandIntent::Invoke { item }`(`'V'`, 25턴 쿨다운)을 구현한다.
  4. **무기/주문 숙련도 `#enhance` (`'X'`)**: 8개 `SkillCategory`(`Dagger`, `Sword`, `Axe`, `Blunt`, `Staff`, `Thrown`, `BareHands`, `Spellcasting`)와 6단계 `SkillRank`(`Unskilled=0..GrandMaster=5`)를 도입해 역할별 초기 숙련/상한, 전투/투척/주문 사용 시 `skill_practice` 누적, 명중/피해/주문 보너스, `CommandIntent::Enhance { skill }` 승급을 구현한다.
  5. **기존/저장 호환성**: 레거시(`campaign = None`, `SaveDataV1` V1) 및 `P7`/`P8`/`P9` 시드·해시·`LlmObservationView < 32 KiB`·`ActionSpace <= 64`를 보존하며, `SaveV3` 검증에 제단 성향 범위, 아티팩트 유일성·등록 정합성, 숙련도/연습치 유효성을 추가한다.
- **Alternatives Considered**:
  - `CampaignState`에 `BTreeMap`을 넣어 `Copy`를 깨뜨리는 안 → `GameSnapshot`/`CampaignObservation`/`WorldState` 복사와 직렬화 호환 비용이 크므로 가변 맵은 `WorldState`/`GameLevel`에 두고 `CampaignState`는 `Copy`를 유지했다.
  - 무기별 개별 숙련도 20종을 전부 `ActionIntent`로 노출하는 안 → `ActionSpace <= 64` 예산을 압박하므로 8개 대표 `SkillCategory`로 통합하고 승급 가능한 숙련만 `legal_actions`에 노출한다.

## ADR-P9-01: Stage 3 (P9) 도구·컨테이너·#dip/#engrave·지형/함정 생태계 및 SaveV3 확장 계약

- **상태**: Accepted (2026-11-27)
- **맥락**: Stage 2(`P8`)에서 `80×21` 절차적 생성기 v3, 구조적 `validate_persisted_map`, 10 슬롯 장비, `SAVE_SCHEMA_VERSION_V3 = 3` 및 P9 예약 훅(`contained_items`, `engravings`, `lit_tiles`, `tool_action`, `container_capacity`)을 구축했다. 이제 `docs/CODER_HANDOFF_P7_TO_P13.md` Stage 3(`P9`)의 도구(`(`), 컨테이너(가방/상자), `#dip`/성수 경제, `#engrave` `Elbereth`, 분수/싱크/물/용암 및 12종 함정 생태계를 연결해야 한다. 동시에 레거시 2층 모드(`campaign.is_none()`)의 V1 저장·R9 causal 해시(`tests/long_run.rs`)·`observation_v1.json`과 `CommandIntent: Copy`, `LlmObservationView < 32 KiB`, `ActionSpace <= 64` 불변조건은 반드시 유지해야 한다.
- **결정**:
  1. **Save Schema v3 유지 (`SAVE_SCHEMA_VERSION_V3 = 3`) 및 V1 격리**: 캠페인은 `SAVE_SCHEMA_VERSION_V3 = 3`을 유지하고 신규 P9 필드(`EntityPayload::Item`의 `tool_active`, `container_locked`, `container_trapped`, `erosion`, `EntityData`의 `tool_action`, `container_capacity`, `EntitySnapshot`/`Observation` P9 필드)에 모두 `#[serde(default, skip_serializing_if = ...)]`를 적용한다. `campaign.is_none()` 레거시 모드에서는 `EntityLocation::InContainer`, P9 타일/함정(`Fountain`, `DryFountain`, `Sink`, `Water`, `Lava`, 신규 9종 함정) 및 P9 아이템/메타데이터를 저장 검증에서 거부해 V1 직렬화와 해시를 불변으로 유지한다.
  2. **`CommandIntent: Copy` 보존을 위한 고정 타입 액션 설계**: `ActionIntent`/`CommandIntent`가 `Copy`를 유지하도록 `#engrave`와 `#dip`은 임의 `String` 대신 `DipTarget { Item(EntityId), Tile }`, `EngravingMethod { Fingers, Item(EntityId) }`, `EngravingText { Elbereth, Warning, Safe, XMark }`(`as_str()`로 `"Elbereth"`, `"Beware!"`, `"Safe"`, `"X"` 산출)를 사용한다. ActionSpace `.take(64)` 상한 안에는 `Drink`/`Apply`/`Engrave` 등 핵심 명령을 `Drop`보다 앞에 배치해 LLM/Headless/TUI가 항상 접근할 수 있게 한다.
  3. **컨테이너(`Sack`, `BagOfHolding`, `Chest`) 및 `BagOfHolding` 불변조건**: 컨테이너 내부 아이템은 `EntityLocation::InContainer { container }`와 컨테이너의 `contained_items: Vec<EntityId>`가 양방향 일치해야 하며 깊이 상한(4단계)·순환 금지·비음수 raw 무게 `<= container_capacity`를 `save.rs`에서 검증한다. `AmuletAscension`은 어떤 컨테이너에도 넣을 수 없어 목표 추적 및 승천 불변조건을 단순·안전하게 보존한다. `BagOfHolding`은 BUC별 내용물 무게 배율(`Blessed /4`, `Uncursed /2`, `Cursed *2`)을 적용하고, `BagOfHolding`끼리 중첩 삽입 시 2d8 폭발(`DeathCause::Explosion`)과 함께 내용물을 소멸시킨다.
  4. **12종 함정·지형(`Fountain`, `DryFountain`, `Sink`, `Water`, `Lava`) 및 `validate_persisted_map` 수용**: 신규 지형/함정은 `campaign_map.rs`에서 주로 Room 4/5에 배치해 `rooms[0..3]` 주 경로와 기존 3종 함정 좌표를 보존하며, `all_targets_reachable`는 `Wall | Water | Lava`를 비통과로 판정한다. `validate_persisted_map`은 `Fountain -> DryFountain` 고갈과 12종 `HiddenTrap -> Trap` 공개를 허용하고 임의 지형/계단/상점/제단 변조를 차단한다.
  5. **충돌 없는 TUI 조작 키 매핑**: 기존 키맵(`q`, `r`, `p`, `P`, `T`, `x`, `Z`, `O`, `$`, `f`, `s` 등)과 충돌하지 않도록 `'a'`(Apply tool), `'C'`(Container/Loot), `'v'`(Dip), `'E'`(Engrave), `'g'`(Drink, `'q'` 메뉴에도 병행)를 할당한다.
- **대안 및 기각**: (a) `SAVE_SCHEMA_VERSION_V4`를 신설하면 불필요하게 기존 V3 캠페인 호환성을 깨므로 optional serde 훅을 갖춘 `V3`을 유지한다; (b) `CommandIntent::Engrave`에 `String`을 넣으면 `Copy`가 깨져 `ActionIntent`/`PlayMenu`/LLM 전 계층을 비-Copy로 고쳐야 하므로 `EngravingText` enum을 채택한다; (c) `AmuletAscension`을 컨테이너에 넣을 수 있게 허용하면 귀환 추격·승리·강제 변조 검사가 복잡해지므로 NetHack 3.6.7 스타일 반발 규칙과 함께 차단한다.
- **결과**: 도구·컨테이너·성수/연금술·Elbereth·물/용암/분수/싱크·12종 함정이 `GameSession::submit` 단일 트랜잭션 경계 안에서 결정론적으로 동작하고 save/TUI/Headless/LLM 예산 불변조건을 모두 만족한다.

## ADR-P8-01: Stage 2 (P8) 데이터 주도 콘텐츠·확장 RPG 코어·80x21 생성기 v3·SaveV3 및 P9~P13 스키마 훅

- **상태**: Accepted (2026-11-27)
- **맥락**: P7과 Stage 1(`P7-Fix`)까지는 40×20 고정 4방 생성기와 셀 단위 맵 일치 검증, 무기/몸통/반지 1개(총 3슬롯), 정수 나눗셈 턴 스케줄러(`speed` 0..=12), 역할별 고정 HP/hit/dmg, 화염/독 2종 저항, 잭칼 전용 시체만 지원했다. `P9`~`P13`의 대량 몬스터·아이템·특수 레벨·종교/아티팩트·마법/동료·승천 탑 확장을 수용하려면 엔진 코어와 세이브/콘텐츠 스키마를 먼저 구조적으로 업그레이드해야 한다. 동시에 레거시 2층 모드(`campaign.is_none()`)의 V1 세이브·`observation_v1.json`·R9 causal baseline 해시(`tests/long_run.rs`)는 100% 보존해야 한다.
- **결정**:
  1. **Save Schema v3 (`SAVE_SCHEMA_VERSION_V3 = 3`) 및 V1 격리**: 레거시 모드(`campaign.is_none()`)는 `schema_version = 1`과 V1 JSON 형태를 그대로 유지하며 모든 신규 P8 및 P9~P13 예약 필드에 `#[serde(default, skip_serializing_if = ...)]`를 적용해 레거시 직렬화와 R9 causal baseline을 불변으로 유지한다. 캠페인 모드(`campaign.is_some()`)는 `80×21` 맵 크기·6대 능력치·10 장비 슬롯 등 비호환 구조 확장에 맞춰 `schema_version = 3`으로 승격하며, V1/V2 캠페인 위조/구형 세이브는 로드 경계에서 명시적으로 거부한다.
  2. **데이터 주도 콘텐츠 레지스트리 (`P8-A`)**: `CONTENT_SCHEMA_VERSION = 1` 하위 호환을 유지한 채 `items.toml`/`monsters.toml`에 `name`, `color`, `unidentified_pool`, `material`, `stackable`, `damage_small`/`damage_large`, `effects[]`, `tags[]`, `spawn_depth_range`, `spawn_weight`, `base_level`, `hp_dice`, `attacks[]`, `resistances[]`, `corpse_chance`, `corpse_nutrition`, `corpse_effects[]`, `ai_flags[]` 및 P9~P13 훅(`tool_action`, `container_capacity`, `artifact_id`)을 추가한다. `monster.speed` 검증 상한을 `0..=36`으로 확장하고, `armor` 슬롯(`body|shield|helm|cloak|gloves|boots`)과 `amulet` 슬롯·신규 P8 아이템 13종·`monster.bat`(`speed = 24`)을 추가한다. `ItemData.armor_slot`은 `Body` 방어구에 대해 `None`을 유지해 기존 V1 `ItemData` 직렬화를 보존하고, 정적 표시 메타데이터(`glyph`, `name`, `color`)는 `ContentRegistry` 및 캠페인 전용 optional `Observation` 필드로 조회해 TUI 하드코딩 `match`를 제거한다.
  3. **확장 RPG 코어 (`P8-B`)**:
     - **10 장비 슬롯**: `Melee`, `Shield`, `Helm`, `Body`, `Cloak`, `Gloves`, `Boots`, `LeftRing`(`#[serde(alias = "Ring")]` 호환), `RightRing`, `Amulet`을 `Inventory`에 저장하고 통합 하강 AC 공식(`10 - sum(ac_bonus + enchant)`)과 슬롯별 저주 가드·`CommandIntent::TakeOff { slot }`을 적용한다.
     - **에너지 버킷 턴 스케줄러**: `ActorStats.energy` 누적기(`ENERGY_THRESHOLD = 12`, 턴당 최대 `4` 행동)로 `speed = 24`는 턴당 2행동, `speed = 6`은 2턴당 1행동, `Fast` 상태는 몬스터 에너지 획득을 절반으로 줄이며 `speed == 12` 레거시 케이던스와 해시를 보존한다.
     - **6대 능력치 (`Str`, `Dex`, `Con`, `Int`, `Wis`, `Cha`, 3..=25)**: 역할별 시작 능력치(`Knight 16/11/15/9/10/12`, `Scout 12/16/12/11/10/12`, `Mage 9/12/10/17/14/11`, `Cleric 12/10/14/11/16/14`)를 `CampaignState`에 보관하고, `Str`(무게 한도 `60 + Str*5`, 근접 피해 `(Str-12)/3`), `Dex`(명중 `(Dex-12)/2`, `Dex>=15` 함정 피해 `-1`), `Con`(레벨당 `max_hp` 보정 `((Con-12)/3).max(0)`, 재생 주기 단축), `Int`/`Wis`(주문 위력·치유·기도 쿨다운 `20 - ((Wis-12).max(0))` 단축), `Cha`(상점 구매/판매 배율: `<=8` 120%/80%, `9..=14` 100%, `>=15` 85%/115%)를 런타임 전체에 연결한다.
     - **저항 비트셋과 상태이상**: 8종 `Resistances`(`Fire`, `Cold`, `Shock`, `Poison`, `Sleep`, `Disintegration`, `Telepathy`, `SeeInvisible`)의 내재(`intrinsic_resistances`)+장비 외재 합성 및 7종 `StatusTimers`(`Blind`, `Confused`, `Stunned`, `Hallucinating`, `Fast`, `Levitation`, `Invisible`) 매 턴 감산/효과(시야 0+텔레파시, 결정론적 혼란 이동, 공중부양 함정 회피, 투명화 탐지 차단, 가속)를 구현한다.
     - **일반화된 몬스터 시체**: 캠페인 처치는 레지스트리의 `corpse_chance`·`corpse_nutrition`·`corpse_effects`로 임의 몬스터 시체(`corpse_monster`, `corpse_age`)를 드롭하며(레거시 잭칼 전용 드롭 유지), `corpse_age > 50` 부패 식중독, 시체 섭취 시 확률적 내재 저항 획득, `Troll` 시체 `corpse_age >= 25` 부활을 지원한다.
  4. **Generator v3 (`80×21`)·분기 그래프·구조 검증기 (`P8-C`)**: `campaign_map`을 `80×21` 가변 방·복도·루프·어둠/조명(`lit_tiles`)·특수방(`Shop`, `Temple`, `Barracks`, `Vault`)과 BFS 연결 보장 구조로 교체하고 `BranchGraph`(`Main 1..=6`, `Mines 1..=2`, 예약 `Sokoban`/`Quest`/`Gehennom`)로 계단/분기 전환을 일반화한다. `validate_persisted_map`은 `80×21` 크기·외곽 `Wall` 테두리·계단 수/좌표·`Shop`/`Altar` 위조/삭제 차단·BFS 도달성·`lit_tiles`/`engravings` 불변조건을 검증하되 내부 `Wall -> Floor` 전이(P9 곡괭이 굴착 대비)는 허용한다.
  5. **P9~P13 스키마 훅 선반영**: `contained_items`, `artifact_id`, `engravings`, `special_level_id`, `alignment`, `race`, `god_anger`, `prayer_timeout`, `weapon_skills`, `spell_skill`, `learned_spells`, `generated_artifacts`, `ascension_altars`, `Faction::{Pet, Peaceful, Shopkeeper}`, `monster_inventory`를 optional serde 생략 필드로 선반영한다(단 `CampaignState: Copy` 관례를 보존하기 위해 컬렉션 훅은 `WorldState`/`SavedWorldV1`에 배치).
- **대안 및 기각**: (a) 40×20 셀 단위 맵 동일성 검증과 V2 save를 그대로 유지하면 절차적 다중 방과 P9 굴착이 막히므로 `SaveV3` + 구조적 검증으로 전환한다; (b) `ItemData`에 `String` 메타데이터를 직접 복사하면 `Copy` 및 V1 `ItemData` 직렬화가 깨지므로 `ContentRegistry` 조회와 `Observation` optional 필드를 사용한다.
- **결과**: `P9`~`P13` 콘텐츠와 특수 레벨을 세이브/엔진 대수술 없이 추가할 수 있는 기반이 확립되며, `80×21` 맵에서도 `LlmObservationView`(시야 타일 필터)는 `32,768B` 이하를 유지하고 TUI `60×24` 뷰포트는 플레이어 중심 카메라 스크롤로 동작한다.

## ADR-P7-02: Stage 1 (P7-Fix) 하강 AC 통일·52슬롯·안개·미식별 은닉·허기/회복·두루마리 BUC 및 호환성

- **상태**: Accepted (2026-11-27)
- **맥락**: P7 확장 워킹트리에서 방어구·반지 AC 부호 역전(`DEF-01`), 인벤토리 `a-z` 단조 소진(`DEF-02`), `ScrollLevelTeleport` 아뮬렛 추격 우회(`DEF-03`), 허기/자연회복 미연결(`DEF-04`~`05`), 미식별 BUC·상점 외형 누출(`DEF-06`), Fog of War 미구현(`DEF-07`), `aihack-content` 레벨 로더 누락(`DEF-08`), 생성 화면 정적 스탯/미연결 상태(`DEF-09`), 두루마리 BUC 분기·대상 지정 감정 부재(`DEF-10`)가 확인됐다. 동시에 V1 저장·`observation_v1.json` 호환·`Entity::item()` 5-튜플·`LlmObservationView` 32 KiB 예산·64개 `action_space` 상한을 유지해야 한다.
- **결정**:
  1. **전면 하강 AC 통일(`DEF-01`)**: `adventurer_template().ac = 10`, `combat::attack_roll_value` `defense = 20 - defender.ac`, `monsters.toml` 12종 AC(`10 - old_ac`), `PlayerObservation.ac`(`default = 10`, `skip_serializing_if = "is_default_player_ac"`), TUI `AC:<val>` HUD로 통일한다.
  2. **52슬롯 인벤토리 재사용(`DEF-02`)**: `Inventory`는 `a..=z`, `A..=Z` 중 `entries`에 없는 가장 앞 슬롯을 재할당하고 `next_letter_index` 불변조건(`<= 52`, `>= entries.len()`)을 유지하며, `Pickup`/`Buy` 스택 병합은 슬롯 한도 전에 먼저 검사한다.
  3. **감정 은닉·안개·예산 격리(`DEF-06`, `DEF-07`)**: `EntityPayload::Item`과 `ItemObservation`에 optional serde `buc_known`/`enchant_known`을 두고 캠페인 `Observation`에서 미감정 항목의 `buc`/`enchantment` 값을 마스킹한다. `GameLevel.explored`를 optional serde로 저장하고 `Observation.visible_tiles`에 `visible: false` 타일을 포함하되, `LlmObservationView::from(observation)`은 `tile.visible == true`만 투영해 32,768B 제한을 보존한다.
  4. **허기·자연 회복·아뮬렛 순간이동·두루마리 BUC(`DEF-03`~`05`, `DEF-10`)**: `DeathCause::Starvation`·`Weak`/`Fainting`/아사 효과 및 `survival-v1` 식량 섭취(`hunger <= 150`)를 연결하고, 캠페인 전용 자연 HP 회복(`(15 - level).max(5)`턴, `poison_turns == 0 && nutrition > 50`)과 `has_amulet` 레벨 텔레포트 차단을 적용한다. `CommandIntent::Identify { scroll, target }`는 식별된 `ScrollIdentify`에 대해 `legal_actions`와 TUI 메뉴가 미감정/미확인 대상 전부를 노출하고(혼잡 인벤토리에서도 대상 선택이 굶지 않음), LLM 전송용 `Observation.action_space.commands`는 64개 상한(`.take(64)`)을 유지한다.
- **대안 및 기각**: (a) legacy 모드에만 상승 AC를 남기면 동일 엔진 내 방어 공식 이원화와 향후 콘텐츠 충돌이 남으므로 전면 하강 AC 통일을 택했다; (b) `LlmObservationView`에 전체 `explored` 타일(최대 800개, ~56 KB)을 그대로 실으면 `REQUEST_BODY_LIMIT`(32,768B) 초과로 LLM 요청이 실패하므로 LLM 뷰는 현재 시야 타일만 유지했다.
- **결과**: 모든 전투/장비/상점/탐험/생존 규칙이 단일 일관 계약을 따르며, `DEF-01`·`DEF-04`·`DEF-07` 반영 후 `tests/long_run.rs` R9 causal baseline(9개 witness 및 3회 결정론)을 새 기준 해시로 재검증·고정한다.

## ADR-P7-01: P7 제단·마나/주문·시드 외형 식별·아이템 중첩·최종 보스/승천 추격 및 V1 호환성

- **상태**: Accepted (2026-11-26)
- **맥락**: P6(반지 슬롯·저항·상점/골드 경제) 이후 핵심 로그라이크 루프를 완성하기 위해 제단 봉헌/기도 보상, 직업별 MP 풀과 주문 시전, 시드 결정론적 외형 미식별/자동 식별, 소모품/투척물 중첩(stacking), 후반 보스(`Minotaur`, `Dragon`) 및 승천 추격 메커니즘이 필요하다. 동시에 기존 레거시(`campaign.is_none()`) 시뮬레이션 결정론 해시(`tests/long_run.rs`), `observation_v1` JSON 호환성, V1 세이브 호환, `Entity::item()` 시그니처가 유지되어야 한다.
- **결정**:
  1. **직업 MP와 상태 저장 분리**: `CampaignState` 구조체 필드(`role`, `xp`, `amulet`)를 유지하고 `mp`, `max_mp`, `ascension_pursuit_stage`는 `WorldState` 및 `SavedWorldV1`에 `#[serde(default, skip_serializing_if = "...")]`로 추가한다. 레거시 모드는 `mp == 0 && max_mp == 0`과 `ascension_pursuit_stage == 0`을 강제하며 캠페인 검증기에서 불변조건(`max_mp == campaign.max_mp()`, `0 <= mp <= max_mp`)을 검증한다.
  2. **외형 미식별 및 중첩 게이트**: `appearance_for_kind(seed, kind)`는 전투 RNG와 분리된 SplitMix64 기반 Fisher-Yates 셔플로 포션(6색)·스크롤(10라벨)·반지(6재질)를 매핑한다. `ItemObservation.appearance`와 바닥 `EntityObservation.appearance`는 캠페인에서 미식별일 때만 채우고(`None`이면 직렬화 생략), `quantity`는 캠페인 소모품(`Potion`/`Scroll`/`Food`/`Rock`, 최대 `MAX_ITEM_STACK = 99`)에만 `> 1`을 허용하며 레거시는 항상 `1`(`1`일 때 직렬화 생략)으로 고정한다.
  3. **제단·주문·보스/승천 추격**: `TileKind::Altar`(이동·시야 통과 가능)는 `Main 3`과 `Mines 2`의 고정 결정론 좌표에 배치하고 맵 검증기에서 강제한다. `SpellKind::{ForceBolt, Heal, Detect}`는 `CommandIntent::Cast`로 실행하며 `Offer`와 함께 LLM/TUI/헤드리스 계약에 포함한다. `Minotaur`(`Mines 2`), `Dragon`(`Main 6`, `FireBreath`는 `RingFireResistance`로 방어) 및 `AmuletAscension` 보유 상승 시 단계별 추격 스폰과 허기 부담을 적용한다.
- **대안 및 기각**: `CampaignState`에 MP 필드를 직접 추가하거나 `Entity::item()` 5-튜플 시그니처를 변경하면 기존 테스트와 스냅샷 직렬화 호환성이 광범위하게 깨지므로 기각했다.
- **결과**: 레거시 결정론(`long_run`)과 기존 V1 세이브·스냅샷 계약을 훼손하지 않으면서 P7 캠페인 확장을 안전하게 계층적으로 구현할 수 있다.

## ADR-P6-01: P6 반지 슬롯·저항·화염 개미·상점/골드 경제 및 V1/스냅샷 호환성

Status: Accepted (2026-11-26); 구현 시작. 기준 계약: `docs/campaign_spec.md` P6.

Context:

P5까지 BUC·인챈트·잠긴 문·독·특수 몬스터를 추가했으나, 장신구(`Ring`) 슬롯, 화염/독 저항, 누적 골드의 소비처가 없었다. 이를 추가하면서도 기존 V1 저장, `campaign = None` 해시·골드 공식(`difficulty`), `observation_v1.json` 직렬화 및 `Entity::item()` 5-튜플 시그니처를 깨지 않아야 한다.

Decision:

- `Inventory`/`InventorySnapshot`에 `equipped_ring: Option<EntityId>`를, `ItemData`에 `ring_effect: Option<RingEffect>`를, `PlayerObservation`에 `gold`/`fire_resistant`/`poison_resistant`를 모두 `#[serde(default, skip_serializing_if = ...)]`로 추가하여 기본값이 V1 저장·스냅샷·관찰 JSON에서 생략되게 한다.
- AC 계산은 `items::recompute_player_ac` 단일 헬퍼로 통합하여 `adventurer_template().ac - (body_ac + body_enchant + ring_ac + ring_enchant)`를 일관되게 유지하고 `save::validate_inventory`에서도 동일 공식을 검증한다.
- 캠페인 전용 초기 `50`골드와 처치 골드 `difficulty * 25`는 `world.campaign.is_some()`에만 적용하여 legacy/R9(`difficulty`) 점수와 해시를 보존한다.
- `TileKind::Shop`은 `Main 2`·`Mines 1` 생성 위치에만 배치하고 `validate_persisted_map`으로 임의 위치 위조를 거부한다.

Alternatives:

- legacy 모드에서도 처치 골드를 `difficulty * 25`로 변경: R9 `GoldScore` witness와 기존 장기 테스트를 깨므로 기각한다.
- `Ring` AC를 `worn` 별도 필드에 저장: `body`와 이중 진실원이 생기므로 단일 `recompute_player_ac` 공식을 선택한다.

Consequences:

`ContentRegistry::content_hash()`와 `docs/provenance/r7-content.sha256`가 P6 콘텐츠(신규 반지 3종·`FireAnt`) 기준으로 갱신되며, 기존 V1 저장/해시 및 `campaign = None` 경로는 불변으로 유지된다.

## ADR-P5-01: P5 BUC·인챈트·잠긴 문·독·특수 몬스터 및 V1/스냅샷 호환성

Status: Accepted (2026-11-26); 구현 시작. 기준 계약: `docs/campaign_spec.md` P5.

Context:

P4까지 캠페인 던전의 문/비밀문/함정·4역할·16아이템·6몬스터를 구축했으나, 아이템 축복/저주(BUC)와 강화 수치(`-3..=+5`), 잠긴 문(`Locked`), 독 상태이상과 `DeathCause::Poison`, 원거리 사격/재생 몬스터 AI가 없었다. 이를 추가하면서도 기존 V1 저장, `campaign = None` 해시, `tests/ai_api_schema.rs`, `Entity::item()` 5-튜플 시그니처와 기존 회귀 계약을 깨지 않아야 한다.

Decision:

- `EntityPayload::Item`에 `buc: BucStatus`(기본 `Uncursed`)·`enchantment: i8`(기본 `0`)를, `WorldState`/`SavedWorldV1`/`GameSnapshot`/`PlayerObservation`/`ItemObservation`에 각각 `poison_turns`/`buc`/`enchantment`를 `#[serde(default, skip_serializing_if = ...)]`로 추가하여 기본값이 V1 저장/스냅샷 JSON과 해시에서 완전히 생략되게 한다.
- `Entity::item()` 5-튜플(`ItemView`)을 유지하고 `item_buc`/`item_enchantment`/`set_item_buc`/`set_item_enchantment` 별도 접근자를 둔다.
- 무기 BUC/인챈트 피해 보너스는 `AttackProfile` wire 구조를 바꾸지 않고 `resolve_attack_with_profile_and_bonus(..., extra_damage_bonus)`로 전달한다.
- `DoorState::Locked`는 `Kick`으로만 개방하며, `campaign_map`은 `Mines 2` 입구 문을 `Locked`로 생성하고 `validate_persisted_map`은 `Locked`로 생성된 타일에 한해 `Locked -> Open <-> Closed` 전이만 허용한다.

Alternatives:

- `Entity::item()` 튜플을 7-튜플로 변경: 기존 모든 호출처와 테스트를 불필요하게 깨므로 기각한다.
- `AttackProfile` struct에 `damage_bonus` 직렬화 필드 추가: `ItemData` 직렬화 해시/형태를 불필요하게 바꾸므로 기각한다.
- V3 save schema 도입: 기존 V1/V2 호환 읽기만으로 무기본값 필드 확장이 가능하므로 기각한다.

Consequences:

`ContentRegistry::content_hash()`와 `docs/provenance/r7-content.sha256`가 P5 콘텐츠(신규 5아이템·3몬스터) 기준으로 갱신되며, 기존 V1 저장/해시 및 `campaign = None` 경로는 불변으로 유지된다.

## ADR-P4-01: 생성 던전 문·함정 전이 검증 및 P4 역할·콘텐츠 확장

Status: Accepted (2026-11-26); 구현 및 전체 로컬 게이트 검증 진행 중. 기준 계약: `docs/campaign_spec.md` P4.

Context:

P1–P3 첫 캠페인은 생성 맵의 문과 함정을 비워 둔 채 `level.map == expected.map` 완전 일치로 V2 저장을 검증했고, 역할 3종과 초기 몬스터/장비 3·11종에 한정됐다. P4에서는 생성 던전에 닫힌 문·비밀문·다종 함정 상호작용을 도입하고 4번째 역할(`Cleric`)과 중·후반 아이템/몬스터를 추가한다.

Decision:

- `TrapKind`에 `Dart`(피해 2)와 `Bear`(피해 2 + 1턴 마비)를 추가하고, `submit_move`의 함정 사망 원인이 실제 발동한 `TrapKind`를 기록하도록 연결한다.
- `campaign_map::generate`는 1칸 폭 출입구 문(홀수 층 닫힌 문, 짝수 층 `HiddenDoor`)과 방 내부 회피 가능 숨겨진 함정(`Pit`/`Dart`/`Bear`)을 결정론적으로 배치한다.
- V2 저장 검증은 `campaign_map::validate_persisted_map`으로 변경하여 정당한 런타임 전이(`Door(Closed)<->Door(Open)`, `HiddenDoor->Door(*)`, `HiddenTrap(k)->Trap(k)`)만 허용하고 지형·계단·함정·문 위조를 거부한다.
- `Role::Cleric`, 신규 5개 아이템(`ShortSword`, `Mace`, `ArmorChainMail`, `PotionExtraHealing`, `ScrollCharging`), 신규 3종 몬스터(`Kobold`, `Orc`, `Ogre`)를 추가하되 기존 11개 아이템·3종 몬스터 정의와 V1 저장·2층 호환 경로는 변경하지 않는다.

Alternatives:

- 생성 맵 검증을 생략하고 V2 맵 전체를 신뢰: 임의 벽 제거·계단 위조가 통과하므로 기각한다.
- 기존 `items.toml`의 `Dagger`/`ArmorLeather` 수치를 수정: R7/R9 및 `P8-G01..G20` 회귀 기준과 충돌하므로 신규 아이템 추가만 허용한다.

Consequences:

`ContentRegistry::content_hash()`와 `docs/provenance/r7-content.sha256`가 P4 콘텐츠 기준으로 갱신되며 `PROVENANCE.md`에 변경 기록을 남긴다. V1 저장과 `campaign = None` 해시는 불변이다.

## ADR-MA2-02: P1–P3 캠페인과 명시적 save 버전

Status: Implemented with local verification; independent re-audit required (2026-09-05). 증거: `docs/campaign_implementation.md`.

`docs/campaign_spec.md`의 세 단계를 실제 완주 경로로 연결한다. 기존 save/replay를 보존하기 위해 Creation Wait는 유지하고 StartCampaign(role)을 추가한다. CampaignState는 optional typed state, campaign save만 wire schema 2를 사용한다. world를 재진입마다 재생성하거나 구형 게임에 승리 플래그만 붙이는 대안은 기각한다. 목표는 실제 고유 item이며 성공은 별도 Victory다. atomic submit/save validator 경계를 공유한다.

## ADR-MA2-01: 플레이 선택과 저장 수명

Status: Accepted, 구현·검증 결과는 `docs/multi_audit/2/remediation.md` 참조 (2026-09-05)

선택은 core persisted state를 새로 생산하지 않고 TUI 임시 메뉴로 관리한다. 방향·물건이 확정되면 기존 concrete intent를 submit한다. 선택·취소는 core turn/RNG를 보존하며 페이지/마우스 hit test는 표시된 CTA를 기준으로 한다. q는 Quaff, Q는 명시적 종료 확인이다.

production 저장은 `--save-dir` 또는 현재 디렉터리 기준 `runtime/tui`의 capability ArtifactStore를 사용한다. 임시 저장은 테스트 기본값으로만 남긴다. 저장 schema나 public core mutation visibility를 변경하지 않는다. 공개 TUI builder와 CLI 저장 경로 옵션은 추가적 인터페이스다. 더 넓은 게임 목표는 별도 P1~P3 계약으로 단계별 구현한다.

## ADR-0042: report 32 current-HEAD modification evidence와 final-SHA CI

Status: Implemented with full local gate and candidate clean Windows actual bundle verified; exact-headSha CI is external verification authority; independent re-audit required (2026-08-25)

Local verification: candidate `57d8108a51db08f942aba3218eafd2a94cc011d3` date `2026-08-25`에서 453 named tests, fmt/clippy, release all-target build, RustSec/cargo-deny 0.19.4, R7/R8와 381-entry source ZIP·9-entry clean Windows actual bundle이 PASS했다. final acceptance는 이 문서를 포함한 current final `headSha`의 clean Windows 재검증과 GitHub Actions Ubuntu/Linux·Windows completed-success record로 판정하며 별도 evidence commit을 만들지 않는다.
Date: 2026-08-25
Decision ID: DEC-AUDIT-R32-01

Context:

Report 32는 Report 31 lifecycle finding과 FIN-F012를 독립 종결했다. 그러나 final evidence commit `8045249`의 commit date는 `2026-08-25`이고 bundled modification period와 Notice ID는 `2026-08-24`에 머물러 clean Windows bundle이 source identity 검증 뒤 candidate-date 범위 오류로 실패했다. 기존 `license_compliance`는 실제 HEAD 날짜 대신 literal 2026-08-24만 확인해 이 불일치를 actual bundle 전에는 검출하지 못했다.

Decision:

- 단일 current authority는 `docs/audit/audit_report_32.md`다.
- Report 31 lifecycle remediation은 historical independent closure로 보존하고 R32-DBG-F001/FIN-F015 및 PROGRAM/PUBLICATION HOLD를 current 상태로 둔다.
- 배포 modification evidence를 Notice ID `AIHACK-MODIFICATIONS-2026-08-25-01`, covered period `2025-05-20..2026-08-25`로 갱신하고 metadata, build scripts, 양 verifier, R8 checkpoint와 tests에 exact value로 원자 전파한다.
- candidate date의 source of truth는 wall-clock이 아니라 exact final commit의 `git show -s --format=%cs`다. `license_compliance`는 current HEAD 날짜와 `MODIFICATIONS.md`의 단일 period를 직접 파싱해 `start <= candidate <= end`를 조기 강제한다.
- Linux/Windows verifier의 strict calendar와 out-of-period fail-closed 경계는 완화하지 않는다. fixture는 period 이전·종료일·다음 날에서 종료일까지 두 값만 허용한다.
- CI 결과를 기록하려고 final SHA 뒤에 evidence-only commit을 만들지 않는다. remediation과 검증 절차를 포함한 final commit을 먼저 만들고, 그 exact `headSha`의 GitHub Actions completed-success run을 외부 불변 evidence로 사용한다.
- 전체 local gate, clean Windows bundle과 Ubuntu runner의 clean Linux bundle, 동일 final SHA Ubuntu/Windows CI를 통과해도 후속 독립 감사와 별도 게시 승인 전까지 HOLD를 유지한다.

Alternatives:

- verifier가 manifest 종료일을 자동으로 현재 시각까지 늘림: 배포 증거를 build가 사후 조작하고 stale manifest를 은폐하므로 기각한다.
- out-of-period를 warning으로 낮춤: complete corresponding source의 modification evidence를 fail-open하므로 기각한다.
- 기존 2026-08-24 Notice ID만 유지하고 기간만 변경: ID가 가리키는 evidence revision을 숨기므로 새 날짜 ID로 함께 갱신한다.
- CI 성공 후 결과 문서 commit을 추가하고 predecessor CI를 인용: final HEAD와 검증 SHA가 다시 갈라지는 Report 32 재현을 반복하므로 기각한다.

Consequences:

2026-08-25에 만들어지는 final commit은 bundled manifest 범위에 포함된다. 이후 날짜의 commit은 새 modification evidence revision 없이는 조기 test와 actual verifier에서 실패한다. source/test/document 변경과 RED/GREEN, 전체 gate 및 final-SHA CI 절차는 `docs/audit/audit_report_32_remediation.md`에 기록한다.

## ADR-0041: report 31 summary lifecycle과 generic predecessor gate

Status: Superseded in active authority by ADR-0042; independently closed by Report 32 (2026-08-25)
Date: 2026-08-24
Decision ID: DEC-AUDIT-R31-01

Context:

Report 31은 ADR-0040 public Rust visibility와 report 29 기술 회귀, `ed02dbf/32733235414` 양 OS actual bundle을 Verified했다. 그러나 `IMPLEMENTATION_SUMMARY.md` 1절은 완료 증거를 기록하면서 10절은 완료 이전 실행 계획을 유지했고, 11절은 report 29를 현재 문서로 잘못 가리켰다. 기존 `r8_documentation`은 특정 report 27/28 문구와 선택된 header만 검사해 이 active 후반 회귀를 놓쳤다.

Decision:

- 단일 current authority는 `docs/audit/audit_report_31.md`다. implementation summary 1·10·11절을 각각 독립 active section으로 검사하며 모두 현재 report marker를 정확히 한 번 포함해야 한다.
- current-authority 검사는 report 29/30 같은 개별 stale 문자열을 계속 나열하지 않는다. active line에서 current/현재 권위를 선언한 모든 `audit_report_N.md` 또는 `report N`을 추출하고, N이 현재 report 번호와 다르면 공통 규칙으로 거부한다.
- 완료된 predecessor의 lifecycle 재개방도 공통 규칙으로 거부한다. predecessor report를 `다음 단계`, `시정 중`, `구현·검증 진행 중`, 구현/local gate/same-SHA/CI `pending`과 결합한 active line은 실패한다. successor가 Verified라는 역사 사실은 허용한다.
- 기존 README 5개 locale, spec, ADR, summary, documentation audit, roadmap, build/gap, designs, compatibility 및 remediation header 검사는 유지하고 report 31 remediation header와 summary 10·11절을 coverage에 추가한다.
- report 30 public visibility와 report 29 기술 회귀, 전체 local gate 및 새 clean same-SHA Ubuntu/Windows actual bundle을 다시 통과해도 report 31 후속 독립 PASS와 별도 게시 승인 전까지 PROGRAM/PUBLICATION HOLD를 유지한다.

Alternatives:

- report 29 current와 report 30 pending 문구만 exact string으로 추가: 표현이 조금만 달라져도 같은 false-green이 반복되므로 기각한다.
- implementation summary 전체 파일에서 predecessor 문자열을 전면 금지: 11절의 검증된 기술 이력까지 지워 감사 계보를 잃으므로 active section별 의미 검사로 한정한다.
- 상단 1절만 current marker로 검사: 10·11절의 실행 지시가 다시 부패한 이번 재현을 막지 못하므로 기각한다.

Consequences:

문서 gate는 현재 report 번호와 predecessor lifecycle 상태를 구조적으로 해석한다. summary의 현재 기준, 다음 작업과 R9 이력은 서로 다른 문장이어도 하나의 lifecycle을 가리키며, 완료된 Report 30 기술 증거는 역사적 Verified로만 남는다. RED/GREEN, 전체 gate와 CI 증거는 `docs/audit/audit_report_31_remediation.md`에 기록한다.

Local verification: candidate SHA `f3a7aa662d8820b361c674b37264f3246cc2b7ac`에서 453 named tests, fmt/clippy, release all-target build, cargo-audit/deny 0.19.4, R7/R8와 379-entry source ZIP·9-entry clean Windows actual bundle이 PASS했다. docs evidence successor의 same-SHA 양 OS 결과는 다음 CI verification으로 확정했다.

CI verification: docs evidence successor SHA `8c042d48df57621e23a9c2a3406cc6fa68bea0af`의 Actions `32741917348`에서 Ubuntu job `97478142640`과 Windows job `97478143152`이 각 19 success step과 actual platform bundle을 완료했다. cargo-audit, cargo-deny 0.19.4와 lockfile 불변도 모두 success이며 후속 독립 감사와 외부 게시 승인은 별도다.

## ADR-0040: report 30 active authority와 submit-only visibility 강제

Status: Superseded in active authority by ADR-0041; historical technical implementation and independent verification complete (2026-08-24)
Date: 2026-08-24
Decision ID: DEC-AUDIT-R30-01

Context:

`docs/audit/audit_report_30.md`는 report 29의 content, allocator, TUI gesture, archive identity와 projectile/monster 축소를 Verified했지만 designs/compatibility/remediation/roadmap active header가 과거 lifecycle을 유지하고, default public `GameWorld` mutator와 combat/death/doors/items/movement/stairs/traps가 master submit-only 계약을 우회함을 재현했다.

Decision:

- current authority는 report 30 하나다. designs, compatibility index, report 29 remediation top status와 roadmap current paragraph를 포함한 모든 active header/section을 common exact-one/negative mutation gate로 검사한다. predecessor는 명시적 historical section에서만 허용한다.
- default `aihack-runtime` public mutation entry는 atomic `GameSession::submit` 하나다. `GameWorld` mutator·fixture constructor와 combat/death/doors/items/movement/monster/projectile/stairs/traps module은 crate-private로 축소한다.
- read-only world/entity query, validated save/bootstrap와 순수 score/vision API는 public으로 유지한다.
- C010 depleted-death처럼 session 이전 상태를 구성해야 하는 호환 fixture는 opt-in `testing` feature 아래의 명시적 비원자 test helper로만 제공한다. root compatibility test host만 이 feature를 사용하며 TUI/headless dependency와 production source에는 `testing` feature/import를 금지한다.
- external temporary crate regression은 default public read query compile-pass와 World/system mutation compile-fail을 실제 `cargo check --offline`으로 검증한다. 문자열 visibility 검사는 보조 근거로만 사용한다.
- report 29 기술 회귀와 새 clean same-SHA 양 OS actual bundle을 모두 재실행해도 report 30 후속 독립 감사와 별도 게시 승인 전까지 PROGRAM/PUBLICATION HOLD를 유지한다.

Alternatives:

- 모든 low-level mutator를 public compatibility로 문서화: shipped consumer가 turn/RNG/event/monster/invariant commit을 우회하므로 master 계약과 제품 안전성이 약해져 기각한다.
- mutator 이름에 `unchecked`만 추가: compiler visibility가 그대로여서 잘못 쓰기 쉬우므로 기각한다.
- test helper를 default public에 유지: Hyrum's Law상 production contract가 되므로 opt-in feature 격리를 선택한다.

Consequences:

일부 pre-release Rust public path는 제거된다. 외부 production consumer는 `GameSession`과 read-only query로 이동해야 하며, test-only direct world scenarios는 feature-gated helper 또는 persisted `SessionBuilder`로 이동한다. source/test/document 변경과 RED/GREEN, 전체 gate 및 CI는 `docs/audit/audit_report_30_remediation.md`에 기록한다.

Verification update: implementation SHA `59c88720924d28b892e66f732eb4007825eb76d5`에서 Windows named tests 452개, fmt/clippy, release all-target build, cargo-audit/deny 0.19.4, R7/R8와 clean Windows 9-entry actual bundle이 PASS했다. 새 same-SHA Ubuntu/Windows CI와 후속 독립 감사는 pending이다.

CI verification: docs/test successor SHA `ed02dbff3911194e1c4aaaf9b989e5bd41c1b80a`의 Actions `32733235414`에서 Ubuntu job `97450032431`과 Windows job `97450032767`이 각 19 success step과 actual platform bundle을 완료했다. cargo-audit, cargo-deny 0.19.4와 lockfile 불변도 모두 success이며 후속 독립 감사와 외부 게시 승인은 별도다.

## ADR-0039: report 29 transition·source identity·public boundary 폐쇄

Status: Superseded in active authority by ADR-0040; historical technical implementation verified (2026-08-24)
Date: 2026-08-24
Decision ID: DEC-AUDIT-R29-01

Context:

`docs/audit/audit_report_29.md`는 report 28 시정의 exact allocator, custom HP/bootstrap, equipment removal, F9 실제 경로와 calendar 구현을 유지했지만 key-code 중심 Repeat 방어, archive 표시 이름 중심 검사, metadata 문자열 중심 `ExpectedCommit`, active 문서의 current-authority false-green을 재현했다. item ID가 정하는 `ItemKind`와 declared class, public low-level system 오류 원자성도 서로 다른 consumer 기대를 만들었다.

Decision:

- transition/control candidate는 raw key code가 아니라 gesture lifecycle로 보호한다. modifier를 제외하고 Enter/CR/LF, Esc/ESC, Backspace/DEL을 정규화한 논리 key identity를 사용한다. ConPTY의 byte별 합성 Release를 독립 gesture authority로 신뢰하지 않고, transition candidate 뒤 같은 논리 key 또는 새 state의 다른 transition/control candidate를 최소 500ms quiet window와 production 50ms poll 2회 연속 idle이 모두 충족될 때까지 억제한다. 억제 대상이 다시 관찰되면 window/count를 초기화하고, 다른 논리 key의 repeat-safe movement/text candidate는 즉시 새 gesture로 허용한다.
- source archive는 양 OS가 공유하는 format-aware validator로 raw name/type/link/prefix/Unicode·Windows extraction 의미를 검사하고, 검증 후 임시 추출 path/content manifest를 대조한다. 최종 archive는 `ExpectedCommit`에서 같은 format으로 독립 재생성한 `git archive`와 byte-identical해야 한다.
- active documentation의 current authority는 report 29 하나이며 predecessor report와 SHA/run은 명시된 historical section에서만 허용한다. README, ADR, roadmap, summary와 build/gap 문서의 active section을 section-scoped positive/negative regression으로 검사한다.
- 알려진 item ID는 canonical `ItemKind`와 declared kind/class의 고정 pair를 강제한다. v0.3.0 custom registry는 기존 ID의 class-changing override를 지원하지 않으며 모든 adapter가 canonical pair를 소비한다. item glyph는 정확히 한 Unicode scalar만 수용한다.
- 외부 production mutation API는 atomic `GameSession::submit` 하나다. mutation 뒤 allocation 오류가 날 수 있는 projectile/monster low-level system은 crate 내부 transaction primitive로 축소한다. ordinary rejection의 invariant-valid state transition은 보존하고 `transaction_aborted` 또는 invariant failure에만 working world/RNG/turn/log/state 전체 rollback을 적용한다.
- exact-successor valid save의 마지막 ID commit과 다음 exhaustion, Throw/Zap의 item·charge·RNG·full-save/hash rollback을 영구 integration fixture로 보존한다.
- implementation/local gate/clean same-SHA CI가 완료돼도 report 29 후속 독립 재감사와 별도 게시 승인 전까지 PROGRAM/PUBLICATION HOLD를 유지한다.

Alternatives:

- 모든 key Repeat 금지: text editing과 정상 movement hold를 손상하므로 repeat-safe allowlist와 gesture barrier를 선택한다.
- timestamp debounce만 사용: runner 부하와 사용자 입력 속도에 따라 달라지므로 matching Release 또는 bounded 500ms quiet window와 production poll 2회 연속 empty를 함께 요구하는 drain boundary를 선택한다.
- metadata commit 문자열과 checksum만 비교: complete corresponding source를 증명하지 못하므로 independently regenerated archive identity를 선택한다.
- class-changing mod 허용 후 모든 consumer 재설계: v0.3.0 wire와 `ItemKind` 기반 UI/행동을 넓게 바꾸므로 canonical ID-kind reject를 선택한다.
- public low-level API에 clone rollback을 각각 추가: mutation ownership이 중복되므로 existing session transaction에 캡슐화한다.

Consequences:

TUI production dispatcher는 mutable gesture state를 소유하고 event-loop idle 신호를 받는다. release 검증에는 Python 3 표준 라이브러리 기반 공통 archive validator가 추가되며 양 script가 같은 정책을 호출한다. custom registry의 이전 ID/class mismatch와 multi-scalar glyph는 시작 단계에서 typed reject된다. `systems::projectiles`와 `systems::monster_ai`의 외부 직접 호출은 더 이상 public contract가 아니다. 수정 전 RED, 구현, 전체 local gate와 양 OS clean same-SHA evidence는 `docs/audit/audit_report_29_remediation.md`에 순차 기록한다.

Verification update: implementation SHA `1fa6d903ea09170014154c0c64e0fdaf673fcb6c`에서 Windows named tests 455개, fmt/clippy, release all-target build, cargo-audit/deny 0.19.4, R7/R8와 clean Windows 9-entry actual bundle이 PASS했다. 새 same-SHA Ubuntu/Windows CI와 후속 독립 감사는 pending이다.

CI verification: Unix fixture identity 수정 successor SHA `a91a9c70523288bf2d5289bb35c9d1f1e5565a33`의 Actions `32706869079`에서 Ubuntu job `97369721441`과 Windows job `97369721295`가 각 19 success step, platform 반대편 1 skip으로 완료됐다. actual Linux/Windows bundle, cargo-audit, cargo-deny 0.19.4와 lockfile 불변이 모두 success다. 이 evidence는 후속 독립 감사나 외부 게시 승인을 대신하지 않는다.

## ADR-0038: report 28 allocator·registry·input·archive 동등 경계

Status: Superseded in active authority by ADR-0039; historical implementation verified (2026-08-24)
Date: 2026-08-24
Decision ID: DEC-AUDIT-R28-01

Context:

`docs/audit/audit_report_28.md`는 report 27 구현/문서 SHA의 양 OS success를 인정했지만 `next_id=u32::MAX-1`, hp 0 custom monster, damage armor의 Wear→Throw, Windows archive component alias를 production/adversarial 경계에서 재현했다. Esc/Enter control-key Repeat 정책, Linux year 0000, F9 실제 toggle 증거와 implementation summary 후반 상태도 열려 있다.

Decision:

- persisted allocator는 `next_id == max_entity_id.checked_add(1)`을 요구하고 core/runtime spawn은 fallible allocation을 사용한다. allocation failure처럼 `transaction_aborted`로 표시한 partial-mutation 오류는 working transaction을 commit하지 않고 원본 RNG/hash/world를 보존하며, ordinary invariant-valid rejection의 state transition은 유지한다.
- content registry는 live monster hp와 item kind별 complete required/forbidden field table을 검증한다. custom registry session은 반환 전 save/persisted invariant를 통과해야 한다.
- armor는 damage/hit bonus를 가질 수 없다. Drop, Throw, consume/read를 포함한 inventory removal은 공통 fallible unequip helper를 거쳐 equipment pointer와 derived AC를 원자적으로 복원한다.
- TUI soft-input의 문자/Backspace Repeat는 허용하지만 Esc, Enter, F9와 Quit `q/Q`는 Press-only로 시작했다. report 29가 동등 transition과 Release 없는 연속 Press를 재개방했으므로 활성 gesture 계약은 ADR-0039가 대체한다.
- source archive의 모든 component는 Windows-compatible fail-closed rule을 따른다. ASCII case-insensitive 이름, trailing dot/space, reserved device basename과 extraction collision을 거부하고 excluded root도 같은 canonical first component로 비교한다.
- candidate/period 날짜는 양 OS 공통 year `0001..9999`와 Gregorian round-trip을 요구한다.
- F9 regression은 실제 Press candidate와 handler return/flag/revision/hash를 검사하고 active implementation summary의 다음 단계는 report 28 시정과 독립 재감사로 갱신한다.
- report 28은 이 ADR 구현 당시 authority였으며 report 29가 활성 authority를 대체한다. 별도 게시 승인 전까지 PROGRAM/PUBLICATION HOLD를 유지한다.

Alternatives:

- save에서 `MAX-1`만 추가 차단: live allocator의 unchecked panic 경계를 남겨 기각한다.
- schema reject만 하고 Throw removal을 유지: future equipment-capable item이 같은 우회를 재발시켜 공통 lifecycle을 선택한다.
- 모든 key Repeat 전역 차단: movement와 text editing의 정상 hold 입력까지 잃으므로 destructive transition/control key만 Press-only로 분리한다.
- Windows verifier만 alias 강화: cross-platform source archive의 추출 의미가 달라지므로 양 verifier 공통 rule을 선택한다.

Consequences:

wire schema v1은 유지하지만 allocator gap, invalid live monster와 forbidden item shape save/custom registry는 더 일찍 typed reject된다. 공개 spawn API는 fallible해지고 caller가 오류를 처리해야 한다. TUI control-key Repeat와 F9 evidence, archive/calendar parity가 양 OS regression matrix에 추가된다. 수정 전 RED와 전체 검증은 `docs/audit/audit_report_28_remediation.md`에 기록한다.

Historical verification: implementation SHA `9725c37896a8d149be5c500cdd26da154ab0a3fa`의 Actions `32694375654`에서 Ubuntu/Windows 각 19개 step, 전체 tests, R7/R8, actual platform bundle, cargo-audit, cargo-deny 0.19.4와 lockfile 불변이 모두 success였다. report 29가 이 ADR의 인접 경계를 재개방했고 ADR-0039가 활성 계약을 대체한다.

## ADR-0037: report 27 field-only causal 및 consumer canonical 경계

Status: Superseded in active authority by ADR-0038/ADR-0039; historical implementation verified (2026-08-24)
Date: 2026-08-24
Decision ID: DEC-AUDIT-R27-01

Context:

`docs/audit/audit_report_27.md`는 report 26 최종 verifier 시정 SHA `1e84a94aa0623b5cee5349b5832992a4682e93a8`와 Actions `32660514315`의 양 OS success를 확인했지만, save allocator/level/charge, unsafe injected armor, omission-by-skipping causal harness, archive path alias, Linux calendar, debug panel mouse, Judge text repeat와 repository-root local action recursion을 production entrypoint에서 재현했다. broad green은 열거하지 않은 consumer와 verifier alias를 증명하지 않는다.

Decision:

- SaveDataV1은 다음 spawn이 가능한 allocator headroom, active registry와 동일한 level ID 집합, checked stairs target과 dynamic/max charge optional shape를 load 전에 검증한다.
- ContentRegistry는 item kind별 numeric/field shape를 consumer-safe 범위로 제한한다. armor AC는 adventurer base에서 직접 derive하고 unequip/drop도 base로 재계산하여 accepted registry의 장비 lifecycle을 가역적으로 만든다.
- 9종 causal isolation은 active/control 양쪽에서 같은 producer·consumer command와 observer를 실행하고 대상 field/state 하나만 neutralize한다. difficulty 양쪽은 같은 kill을 수행하며 omission마다 나머지 8개 full attribution record가 complete run과 같아야 한다.
- source archive entry는 raw prefix가 아니라 canonical component로 검사한다. absolute, dot/parent, 빈 component, backslash와 excluded first component를 Linux/Windows 모두 fail-closed로 거부한다. candidate/period 날짜는 strict Gregorian calendar와 ordered containment를 만족해야 한다.
- F9 debug panel은 비모달이지만 visible rect의 mouse authority를 소유한다. Judge editor에서는 character Repeat를 허용하되 일반 Playing LLM request Repeat는 계속 차단한다.
- action pin gate는 repository-root local action을 resolve해 `action.yml` 또는 `action.yaml`의 transitive `uses`를 cycle-safe 재귀 검사하며 missing, root escape, mutable remote와 invalid Docker ref를 거부한다.
- report 27은 이 ADR 구현 당시 독립 authority였고 report 28/29가 차례로 활성 authority를 대체했다. 별도 게시 승인 전까지 PROGRAM/PUBLICATION HOLD를 유지한다.

Alternatives:

- allocator/stairs에서 panic만 catch: 성공 load의 consumer-safe 계약을 충족하지 않아 기각한다.
- saturating Wear/Drop 유지: inverse가 아니며 정상 session을 unsaveable하게 만들 수 있어 base-derived 재계산을 선택한다.
- omission branch에서 command/observer 생략: field 인과성을 증명하지 못하므로 동일 flow field-only A/B를 선택한다.
- archive entry raw prefix만 확장: 새로운 lexical alias를 계속 놓치므로 component canonical validation을 선택한다.

Consequences:

wire schema v1은 유지하지만 이전에 수용되던 malformed allocator/level/charge save와 unsafe custom content는 typed error가 된다. debug panel 아래 map click은 더 이상 turn을 만들지 않고 Judge text repeat는 정상 입력된다. local composite action chain과 release archive/date fixture가 양 OS gate에 추가된다. 수정 전 실패 fixture, 전체 local gate와 새 clean same-SHA CI를 `docs/audit/audit_report_27_remediation.md`에 보존한다.

Historical verification: implementation SHA `ea7822a5b32b3bb9ee8224176381c44871037bc4`의 Actions `32683076204`에서 Ubuntu/Windows 각 19개 step, 437 tests, R7/R8, actual platform bundle, cargo-audit, cargo-deny 0.19.4와 lockfile 불변이 모두 success였다. 후속 report 28/29가 활성 경계를 대체한다.

## ADR-0036: report 26 consumer-safe artifact와 presentation/release authority 경계

Status: Superseded in part by ADR-0037; final report 26 evidence retained (2026-08-24)
Date: 2026-08-24
Decision ID: DEC-AUDIT-R26-01

Context:

`docs/audit/audit_report_26.md`는 report 25 시정 SHA의 clean same-SHA 양 OS PASS를 인정하면서도, 열거되지 않은 malformed scalar, Win32 trailing-name alias, modal mouse, hidden Inspect CTA, summary-label deletion, preplaced release root/inode와 stale modification date를 production probe로 재현했다. helper와 self-consistent metadata만으로는 다음 consumer와 실제 filesystem/UI 의미가 안전하다고 증명되지 않았다.

Decision:

- SaveDataV1 복원은 unequipped player base AC, consumer-safe turn/score 조합과 전달된 registry에 대한 persisted `ItemData` equality를 검사한다. runtime 좁은 정수 consumer도 widening/saturating 정책으로 wraparound를 만들지 않는다.
- Windows artifact component는 trailing dot/space, ADS·금지 문자와 reserved device name을 거부한다. lexical compare, handle identity와 atomic replace가 같은 Win32 이름 의미를 사용한다.
- causal negative는 `CausalSummary::without` 같은 사후 evidence 삭제를 사용하지 않는다. 각 producer command/content/pair를 실행 전에 하나씩 제거한 full production run으로 exactly-one missing witness를 확인한다.
- TUI는 modal/overlay guard를 event kind보다 먼저 적용한다. Inspect renderer와 mouse hit-test는 같은 `InspectPresentation`을 소비하며 LLM request key는 Press만 허용한다.
- release build는 workspace 내부 random fresh directory에서 create-new staging하고 검증된 directory rename으로 승격한다. verifier는 root reparse/symlink와 expected file hard link를 거부한다. Windows link count는 열린 handle의 `GetFileInformationByHandle` 결과를 사용한다.
- release metadata는 exact candidate commit에서 파생한 `candidate_date`를 포함하고 modification manifest 기간과 자동 교차 검증한다. dependency exception은 미래 approval을 거부하며 action pin gate는 `.github/**/*.yml|yaml`을 YAML node로 구조 순회한다.
- 이 결정 당시 report 26을 authority로 두었다. 12건의 로컬 RED→GREEN, 전체 gate와 새 clean same-SHA 양 OS bundle 뒤에도 새 독립 감사와 별도 게시 승인 전까지 program/publication HOLD를 유지한다. 현재 authority는 ADR-0037과 report 27이 대체한다.

Alternatives:

- validator 뒤 consumer만 saturating 처리: forged state를 정상 save로 승인하므로 기각한다.
- mouse click을 마지막에 취소: 이미 생성된 hidden candidate와 renderer 불일치를 유지하므로 기각한다.
- 기존 `output/` 안에서 destination별 replace: root junction과 preplaced inode authority를 계속 신뢰하므로 fresh directory promotion을 선택한다.
- modification ID 문자열만 날짜와 함께 변경: exact commit이 범위 안인지 자동 증명하지 못하므로 `candidate_date` 필드를 추가한다.

Consequences:

wire schema version은 1로 유지하지만 registry와 소비 산술에 맞지 않는 과거 malformed save 및 Windows 비정규 artifact name은 typed error가 된다. release metadata에 새 필수 key가 추가되고 build/verifier 호출은 candidate date를 함께 전달한다. 각 경계는 수정 전 실패 fixture를 보존하고 새 clean commit의 Ubuntu/Windows actual bundle까지 통과해야 Verified가 된다.

Verification update: `fc01ec12bac522e601bc56bced06b0908f5873b0`/Actions `32658658526`은 후속 pipefail 재현 전의 부분 evidence이고, `a9a39d87235109c0fb1d1ea7a31ea3751fd37a30`/Actions `32660221745`는 Ubuntu failure evidence다. 최종 verifier fix SHA `1e84a94aa0623b5cee5349b5832992a4682e93a8`의 Actions `32660514315`에서 Ubuntu/Windows 각 19개 step, actual platform bundle, cargo-audit, cargo-deny 0.19.4와 lockfile 불변이 모두 success다. 이 evidence는 report 26 구현을 Verified로 올리지만 report 27 독립 finding이나 외부 게시 승인을 대신하지 않는다.

## ADR-0035: report 25 production 경계와 exact-set 재시정

Status: Superseded in part by ADR-0036; report 25 evidence retained (2026-08-24)
Date: 2026-08-23
Decision ID: DEC-AUDIT-R25-01

Context:

`docs/audit/audit_report_25.md`는 final multi-audit report 1의 첫 시정본을 실제 production entrypoint와 변조 fixture로 재검증했다. broad workspace green과 helper-level test는 inverse save relation, write-side budget, replay alias identity, production GoldScore pair, TUI event-loop 우선순위/geometry, terminal failure lifecycle, actual release output 집합을 닫지 못했다. 문서도 report 23/24, final report 1과 report 25의 authority를 동시에 current처럼 표현했다.

Decision:

- save read/write는 동일 semantic/byte 예산을 사용한다. actor `alive/hp/max_hp`, inventory 양방향 relation과 armor checked arithmetic을 복원 전에 검증하고, writer도 capped serialization 성공 전에 destination을 열거나 교체하지 않는다.
- headless target 1,000,000은 실행 범위일 뿐 save 성공 보장이 아니다. budget 초과 save는 typed 실패와 no-clobber로 닫으며 v0.3.0에서 history를 자동 폐기하지 않는다.
- artifact relative path는 `.`을 제거한 canonical lexical form을 사용한다. replay input/output은 lexical form, Windows case와 열린 file identity를 함께 비교하고 ambient path helper를 public API에서 제거한다.
- GoldScore는 동일 world/turn의 active/control clone에서 gold만 0으로 바꾸고 양쪽 모두 production `death_score`를 호출한 결과로만 witness를 기록한다.
- TUI는 raw event에서 candidate를 만드는 state-aware dispatcher 하나를 production loop와 tests가 공유한다. blocking state는 LLM dismiss/focus보다 우선하고, reset 이전 response ID는 새 outstanding 존재 여부와 무관하게 먼저 폐기한다.
- blocking prompt는 최소 60x24/80x24에서 별도 modal 높이를 보장한다. command/inspect mouse hit box는 renderer와 같은 CTA label model에서 파생한다.
- release `output/` 전체를 게시 bundle로 정의하고 platform별 actual top-level entry exact set을 검사한다. extra file/directory/link/reparse는 checksum 미포함 여부와 관계없이 실패다.
- dependency exception은 parsed TOML AST, exact trigger set과 valid calendar date를 검사한다. 모든 CI `uses:`는 40-hex ref여야 하며 provenance comment는 실제 tag/SHA와 일치해야 한다.
- 현재 authority는 report 25 HOLD다. report 23/24는 historical closed, final report 1과 첫 remediation은 partial historical evidence이며 gap child/aggregate는 동일 lifecycle을 따른다.

Alternatives:

- loader, checksum inventory, mapper helper만 보강: production writer/output/event loop가 계속 다른 계약을 실행하므로 기각한다.
- 1,000,000턴 save를 항상 보장하도록 즉시 event compaction 도입: wire/evidence retention 정책을 새로 만들고 범위를 확대하므로 v0.3.0에서는 기각한다.
- release publish 대상을 checksum에 적힌 파일만으로 간주: 현재 build와 문서가 `output/` directory를 bundle로 전달하므로 기각한다.

Consequences:

기존 malformed save와 stale release directory 일부는 새 gate에서 거부된다. public ambient path helper 제거는 pre-release internal API 정리이며 production caller는 이미 `ArtifactStore`를 사용한다. 각 수정은 report 25의 수정 전 fixture를 이름 붙인 RED 기록으로 보존하고, 로컬 전체 gate 뒤 clean 동일 SHA의 Ubuntu/Windows release verifier까지 통과해야 `Verified`로 승격한다.

2026-08-24 same-SHA verification update: 첫 clean run `32648979651`의 Ubuntu가 O_PATH `Dir` clone에 대한 parent `sync_all`을 EBADF로 거부했다. capability root를 ambient path로 되돌리지 않고 parent 아래 `.`을 read-only directory `File`로 다시 열어 sync 가능한 descriptor를 얻는 방식으로 시정한다. 첫 run은 실패 증거로 보존하며 후속 same-SHA run만 closure evidence로 사용한다.

Final implementation verification: SHA `b732c42d62f295f4d8be64480c1d0a5a440fe738`의 Actions `32650404618`에서 Ubuntu/Windows tests, R7/R8, actual platform bundle, cargo-audit/deny와 lockfile 불변이 모두 success다. 이 증거는 구현을 `Verified`로 올리지만 독립 재감사나 외부 게시 승인을 대신하지 않는다.

## ADR-0034: SaveDataV1 fail-closed 복원과 self-verifying replay

Status: Superseded in part by ADR-0035 and ADR-0036
Date: 2026-08-23
Decision ID: DEC-SAVE-03

Context:

최종 다중 감사 `FIN-F001..F007`은 v1 save가 schema 외 semantic 관계와 자원 상한을 검사하지 않고, replay metadata를 검증하지 않으며, public `DerefMut`와 registry/equipment lifecycle가 transaction truth를 우회한다고 확인했다. 기존 valid-fixture test와 deterministic hash는 손상 artifact와 외부 mutation을 증명하지 못한다.

Decision:

- SaveDataV1 wire shape와 schema version 1은 유지하되 16 MiB, event/entity 각 100,000개, RNG 1,000,000 draw, persisted text 512 UTF-8 byte의 fail-closed 경계를 둔다.
- 복원 전에 entity, player, map, inventory, equipment, actor stat과 text 관계를 typed validator로 검사한다. invalid artifact는 `GameError::InvalidSave`이며 live session이나 RNG를 만들지 않는다.
- ReplayLineV1은 self-verifying artifact다. 소비한 line의 turn, full outcome, outer hash를 working clone에서 비교하고 전체 성공 후 한 번만 commit한다.
- GameSession/GameWorld/runtime EntityStore의 외부 `DerefMut`를 제거하고 crate-private typed mutation surface만 사용한다.
- GameWorld는 immutable registry context를 runtime-only로 보존한다. injected save 복원은 동일 registry를 인자로 받으며, runtime-created corpse/equipment도 그 context와 공통 lifecycle helper를 사용한다.
- replay append는 bounded read 후 atomic rewrite로 바꿔 외부 hard-link inode에 열린 append handle로 쓰지 않는다.
- atomic replace 뒤 Unix parent directory를 fsync한다. Windows directory handle flush의 portable 보장은 현재 dependency에서 제공하지 않으므로 file sync + atomic replace 이후 전원 손실 metadata durability는 명시적 platform 잔여 위험이다.

Alternatives:

- SaveDataV2로 즉시 변경: v0.3.0 wire 호환을 불필요하게 깨므로 기각한다.
- replay를 command-only로 축소: 기존 integrity 필드와 감사/재현 용도를 무의미하게 만들어 기각한다.
- invalid state를 deserialize 뒤 runtime `expect`에 맡김: panic과 부분 상태를 허용하므로 기각한다.
- 같은 계정의 악성 concurrent writer까지 애플리케이션 lock으로 격리: OS sandbox 없이 완전한 경계를 보장할 수 없고 현재 single-writer 제품 모델을 넘으므로 비목표로 둔다. 사전 배치 link와 외부 inode write는 계속 차단한다.

Consequences:

valid v1 save/replay wire는 유지되지만 과거에 우연히 수용된 malformed artifact는 typed error가 된다. 테스트는 boundary-1/boundary/boundary+1, malformed 관계 matrix, replay field별 tamper/no-partial-commit, 외부 compile-fail, custom registry continuation, armor wear/drop/rewear를 직접 검증한다.

## ADR-0033: winx 0.36.4 LLVM exception 한정 허용

Status: Implemented with machine expiry/graph gate (2026-08-23)
Date: 2026-08-18
Decision ID: DEC-DEP-02

Context:

ADR-0032의 capability filesystem은 Windows에서 `cap-primitives -> winx 0.36.4`를 shipped dependency로 추가한다. `winx`의 유일한 SPDX 식은 `Apache-2.0 WITH LLVM-exception`이며 기존 `deny.toml`의 일반 `Apache-2.0` 허용만으로는 cargo-deny license gate를 통과하지 못한다.

Decision:

- 일반 allowlist를 넓히지 않고 `winx 0.36.4`에만 `Apache-2.0 WITH LLVM-exception`을 허용한다.
- exception owner는 Dependency owner / Release manager다.
- exception은 2026-10-31에 만료하며, 그 전에 `winx`, `cap-primitives`, `cap-std`, `cap-fs-ext`, `cap-tempfile` 중 하나의 version이 바뀌면 즉시 재검토한다.
- `dependency-exceptions.json`의 `DEP-EXC-0001`을 예외 owner, 사유, 승인일, 만료일과 trigger version의 단일 ledger로 사용한다. CI test는 오늘 날짜가 만료일을 지났거나 deny 설정·Cargo graph가 ledger와 다르면 실패한다.
- cargo-deny 0.19.4의 `licenses`, `bans`, `sources` 실제 PASS를 R1/R8 필수 증거로 유지한다.

Alternatives:

- `Apache-2.0 WITH LLVM-exception`을 일반 allowlist에 추가: 향후 무관한 crate까지 자동 허용하므로 기각한다.
- capability dependency 제거: SEC-F001의 portable path sandbox와 atomic replace 경계를 후퇴시키므로 기각한다.
- cargo-deny gate 생략: shipped dependency policy를 검증하지 못하므로 기각한다.

Consequences:

`deny.toml` exception은 crate와 version을 함께 고정한다. `dependency-exceptions.json` checker가 만료, unrelated crate 확장, dependency version 변경을 fail-closed한다. 실패 시 exception 유지, dependency 교체, 일반 정책 변경 중 하나를 다시 결정하고 `BUILD_GUIDE.md`와 감사 기록을 갱신한다.

전역 `multiple-versions = "allow"`는 무제한 허용이 아니다. `dependency-duplicate-budget.json`의 정확한 all-target family/version 집합과 최대 23개 family를 별도 machine gate로 고정하며 drift는 review 전까지 실패한다.

Verification update: implementation SHA `2519bc8e0ede81c39f46b5778e62a41d4ca66901`의 Actions run `32107862171`에서 Ubuntu/Windows cargo-deny 0.19.4와 전체 quality gate가 success다. report 23/24 시정은 후속 독립 재감사와 외부 게시 승인을 대체하지 않는다.

## ADR-0032: capability 기반 save/replay 파일 경계

Status: Superseded in part by ADR-0034 (2026-08-23)
Date: 2026-08-18
Decision ID: DEC-SAVE-02

Context:

기존 headless 경로는 문자열 경로를 canonicalize한 뒤 실제 파일을 다시 열었고, save는 예측 가능한 `.tmp`를 truncate mode로 생성했다. 이 구조는 경로 검사와 open 사이의 교체 경쟁을 남기며, 미리 배치한 symbolic link 또는 hard link가 root 밖 파일을 열거나 truncate하게 할 수 있다. TUI quick-save도 모든 프로세스가 같은 OS temp 경로를 공유했다.

Decision:

- headless의 save/load/replay/report I/O는 열린 runtime root directory capability와 검증된 상대 경로를 함께 보유하는 `ArtifactStore`를 통한다. absolute path와 root 탈출은 구조적으로 거부하고, 각 open/rename은 root handle 기준으로 수행한다.
- 쓰기 대상의 마지막 경로 요소는 symbolic link를 따라가지 않는다. 열린 handle이 일반 파일이며 hard-link count가 1인지 쓰기 전에 검증한다.
- save는 대상과 같은 directory에 충돌하지 않는 임시 파일을 `create_new`로 생성하고 payload와 file metadata를 동기화한 뒤 capability-relative rename으로 교체한다. Unix는 mode `0600`을 강제하고 Windows는 parent directory DACL을 상속하므로, Windows에서 기밀성이 필요한 runtime root는 사용자 전용 directory여야 한다. 실패 시 새 임시 파일만 정리하고 기존 save는 보존한다.
- replay 기록은 기존 payload를 bounded read한 뒤 같은 directory의 temporary file로 atomic rewrite한다. 열린 외부 inode에 append하지 않는다.
- TUI quick-save는 프로세스별 임시 directory를 사용해 다른 실행과 경로를 공유하지 않는다.
- `ArtifactStore::open`은 root의 마지막 component를 no-follow로 열고 symbolic link와 Windows junction/reparse root를 거절한다. TUI는 store와 relative quick-save path를 소유한다.

Alternatives:

- canonicalize 후 일반 `File::open`/`File::create` 유지: 검사와 사용 사이 경쟁을 닫지 못해 기각한다.
- 고정 `.tmp` 이름에 `create_new`만 추가: 사전 배치 공격의 truncate는 막지만 동시 실행 충돌과 parent 교체 경계를 해결하지 못해 기각한다.
- 플랫폼별 raw descriptor API를 프로젝트에서 직접 구현: `unsafe`와 OS별 유지보수 표면을 늘리므로 기각하고, 검증된 capability filesystem dependency를 사용한다.
- Windows DACL을 runtime에서 직접 재작성: `unsafe` Windows ACL API와 권한 상속 파괴 위험이 현재 save 데이터 민감도보다 크므로 기각한다. owner-only가 필수인 배포는 사용자 전용 application directory를 root로 제공한다.

Consequences:

`aihack-runtime`은 capability filesystem dependency를 추가한다. path 기반 compatibility helper는 신뢰된 테스트 경로에서만 유지하고 production CLI는 `ArtifactStore`를 사용한다. `tests/headless_paths.rs`는 Windows와 Unix에서 temp hard-link/symlink, replay link, parent escape, 기존 save 보존과 platform permission contract를 직접 회귀한다. Windows inherited DACL은 owner-only hard boundary로 표현하지 않는다.

## ADR-0031: 콘텐츠 인과 폐쇄와 상태-delta 검증

Status: Accepted (2026-08-17)
Date: 2026-08-17
Decision ID: DEC-CAUSE-01

Context:

기존 content registry와 snapshot은 많은 값을 저장하지만 일부 값은 kind 기반 상수에 가려지거나 production producer/consumer가 없다. 기존 1000턴 테스트는 생존과 최종 hash 반복성만 확인하므로 orphan content가 남아 있어도 통과한다.

Decision:

- 주요 콘텐츠는 `producer -> semantic state delta -> consumer -> downstream delta` 경로를 가져야 한다.
- PASS 판정에서 turn, event count, last event만의 변화는 제외한다.
- item nutrition은 Eat 행동과 hunger 전이에 연결한다.
- armor, monster behavior처럼 schema가 지원한다고 선언한 값은 typed runtime data에 투영한다. 현재 milestone에서 의미 있게 지원하지 않을 값은 거짓 계약으로 남기지 않고 명시적으로 제거 또는 후속 비목표로 분류한다.
- 경제 값은 가격이 후속 score 또는 경제 상태를 바꾸는 최소 루프로 연결한다.
- luck과 hallucination은 production producer와 downstream consumer를 함께 제공할 때만 active state로 인정한다.
- seed 42, 7, 1234의 장기 regression은 필수 causal witness와 semantic delta를 집계하고 반복 hash와 함께 검증한다.

Alternatives:

- 이벤트 존재만 검증: 구현 호출은 증명하지만 실제 세계 변화는 증명하지 못해 기각한다.
- 모든 orphan 필드 삭제: save v1 호환성과 이미 노출된 관찰 계약을 불필요하게 파괴하므로 기각한다.
- 한 번에 범용 event bus 도입: 현재 규모보다 복잡하고 인과 증거를 자동으로 보장하지 않아 기각한다.

Consequences:

R9는 테스트 기반을 먼저 만들고 음식/영양, content behavior, 경제/점수, 상태 orphan 순서로 작은 수직 슬라이스를 적용한다. 의도된 snapshot hash 변화는 witness와 ADR 근거 없이 baseline만 갱신할 수 없다.

2026-08-18 verification update: `CausalProjection`과 9종 필수 `CausalWitness`가 seed 42/7/1234의 1000턴 fixture에서 각 1회 이상 발생하고 witness multiset/final hash가 3회 반복 일치한다. `hallucinating` SaveDataV1 compatibility risk owner는 Project owner/runtime maintainer이며 SaveDataV2·v0.4.0 scope 승인 또는 2026-10-31 중 먼저 도래하는 시점에 제거 migration과 실제 producer feature 중 하나를 재결정한다.

## ADR-0021: NetHack 3.6.7 행동 호환 clean reimplementation

Status: Implemented (2026-07-15)
Date: 2026-07-15
Decision ID: DEC-PRODUCT-01

Context:

사용자의 제품 목표는 NetHack 3.6.7을 Rust로 재구성하고 local LLM을 메시지와 판정에 사용하는 것이다. ADR-0005의 “NetHack-inspired 독립 게임”은 이 목표를 정확히 표현하지 않으며, 줄 단위 포트는 기존 legacy 구조 문제와 출처 위험을 되살린다.

Decision:

AIHack은 NetHack 3.6.7의 관찰 가능한 행동을 compatibility scenario로 재구현한다. 구현 단위는 source locator, precondition, typed command, expected event/state, Rust test를 갖는다. 원본 C control flow, symbol layout, 문자열, 데이터 테이블을 그대로 번역하거나 복사하지 않는다. v0.3.0은 NH367-C001..C010만 release scope로 고정한다.

Alternatives Considered:

- NetHack-inspired 독립 게임 유지: 사용자의 3.6.7 변환 목표와 어긋나므로 기각
- C 소스 자동 변환: Rust 경계·안전성·유지보수성과 provenance가 악화되어 기각
- full 3.6.7 parity를 v0.3.0에 포함: 검증 가능한 종료 조건이 없어 기각

Consequences:

- ADR-0005의 제품 범위는 이 ADR로 대체된다.
- compatibility ID 없는 NetHack 규칙 변경은 release에 포함하지 않는다.
- full parity는 이후 milestone로 남으며 v0.3.0 완료 조건이 아니다.
- R7 provenance와 compatibility gate가 필수가 된다.
- R7 source 대조에서 기존 hunger 상태 경계가 3.6.7과 다름을 발견해 C008 RED test 후 `newuhs`의 다섯 상태 경계로 수정했다. 기존 `Oversatiated` enum variant는 호환 목적으로 남지만 새 projection에서는 생성하지 않는다.

## ADR-0022: Rust 1.94.1과 단일 UI dependency 계열 고정

Status: Accepted; UI dependency 선택은 ADR-0028로 대체됨
Date: 2026-07-15
Decision IDs: DEC-RUST-01, DEC-UI-DEP-01

Context:

R1 시작 전에는 빌드가 통과하지만 repository toolchain과 MSRV가 없고, ratatui 0.30이 crossterm 0.29를 추가하여 direct crossterm 0.28과 공존했다. 두 binary에 default-run도 없었다.

Decision:

`rust-toolchain.toml` channel을 1.94.1로, Cargo rust-version을 1.94로 고정한다. Cargo 자동화에는 `--locked`를 사용하고 default-run은 `aihack`이다. UI dependency 선택은 ADR-0028을 따른다.

Alternatives Considered:

- ratatui 0.30과 crossterm 0.29로 동시 상향: 당시에는 회귀 범위가 불명확하여 보류했으나 RustSec advisory 확인 후 ADR-0028에서 채택
- dependency duplicate 허용: event/key type 혼선과 build drift가 남아 기각
- stable 최신을 매 실행 사용: 재현 불가능하여 기각

Consequences:

- R1에서 lockfile 변경과 UI compile regression을 한 번 검증한다.
- toolchain upgrade는 별도 ADR과 CI matrix 변경이 필요하다.
- R1 이후 quick start는 `default-run = "aihack"`을 사용한다.

## ADR-0028: RustSec 경고 없는 ratatui 0.30/crossterm 0.29 계열

Status: Accepted; R1/report 21 verified, RUSTSEC-2026-0253 remediation updated 2026-08-18
Date: 2026-07-15
Decision ID: DEC-UI-DEP-01

Context:

ADR-0022의 ratatui 0.29 계열은 crossterm 중복을 피했지만, 현재 RustSec advisory에서 필수 dependency `lru 0.12.5`의 memory-corruption unsound 문제와 `paste 1.0.15`의 unmaintained 상태가 확인됐다. `lru`는 ratatui 0.29의 비선택 dependency이며 수정된 버전은 0.16.3 이상이다.

Decision:

UI dependency를 `ratatui = "0.30"`과 `crossterm = "0.29"`로 함께 유지한다. lockfile은 ratatui 0.30.2, crossterm 0.29.0을 고정하고 전이 의존성 `lru`는 `RUSTSEC-2026-0253`이 수정된 0.18.2 이상을 요구한다. `cargo audit`, `cargo deny check licenses bans sources`, crossterm 단일 버전 검증을 R1 gate에 포함한다.

Alternatives Considered:

- ratatui 0.29 유지와 RustSec 예외: memory-corruption advisory와 유지보수 중단 dependency를 release baseline에 남기므로 기각
- lru만 단독 상향 또는 feature 제거: ratatui 0.29가 요구하는 API/의존성 제약과 맞지 않아 기각
- ratatui 0.29 fork/patch: 보안 수정의 장기 유지 책임을 프로젝트가 떠안으므로 기각

Consequences:

- R1 lockfile diff와 full test/UI compile regression을 검증한다.
- 기존 UI API가 바뀔 경우 최소 호환 수정과 회귀 테스트를 같은 Task에 포함한다.
- R1의 SC-BUILD-02는 Linux/Windows 원격 CI가 green일 때만 PASS다.

## ADR-0023: private state와 transaction/invariant commit

Status: Implemented (2026-07-15)
Date: 2026-07-15
Decision IDs: DEC-STATE-01, DEC-RUNTIME-01, DEC-RNG-01

Context:

`GameSession`과 `GameWorld`의 mutable field가 공개되어 UI, LLM, test가 command validation과 turn semantics를 우회할 수 있다. submit 내부 mutation 중 오류가 나면 부분 state와 RNG draw rollback 계약도 없다.

Decision:

`GameSession`을 유일한 mutable owner로 유지하되 모든 field를 private으로 만든다. 외부 read는 session getter, `Observation`, snapshot query로 제한한다. accepted command는 `TurnTransaction`의 prepare → apply → 6 invariant validate → atomic commit 순서를 따른다. 거절과 invariant failure는 `accepted=false` outcome으로 표현하며 world, event log, turn, RNG draw를 모두 보존한다.

Alternatives Considered:

- public field + coding convention: compiler가 우회 경로를 차단하지 못해 기각
- interior mutability를 광범위하게 사용: borrow 오류를 runtime으로 이동시켜 기각
- ECS로 전환: 상태 캡슐화와 behavior preservation보다 범위가 커 기각

Consequences:

- integration test는 `tests/support/session_builder.rs`를 사용한다.
- R2에서는 hash field order와 게임 공식 변경을 허용하지 않는다.
- invariant 오류는 no-commit `accepted=false` result가 된다.
- `GameClient`, revision, typed submit error는 R5 workspace boundary와 R6 stale-response gate에서 함께 도입한다.

## ADR-0024: embedded TOML ContentRegistry가 runtime 데이터 원천

Status: Implemented (Re-audit #2, 2026-07-16)
Date: 2026-07-15
Decision ID: DEC-CONTENT-01

Context:

기존 TOML loader는 data test에서만 사용되고 runtime factory와 level 생성은 hardcoded 값에 의존했으며, invalid data의 일부는 `expect` 또는 panic으로 끝났다.

Decision:

items, monsters, levels TOML을 build에 embed하고 process 시작 시 한 번 parse/validate하여 immutable `ContentRegistry`를 만든다. runtime factory는 ID로 registry를 조회한다. duplicate ID, unknown reference, invalid dice/coordinate, unpaired stairs, unsupported schema는 `ContentError`로 반환한다. canonical content hash는 정렬된 schema v1 데이터의 FNV-1a 64-bit다.

Alternatives Considered:

- hardcoded Rust 상수 유지: 데이터와 test의 이중 원천이 남아 기각
- 매 access마다 TOML parse: 비용과 오류 시점이 불안정해 기각
- 외부 mutable data directory: v0.3.0 배포·replay 재현성을 낮춰 기각

Consequences:

- content schema/version과 hash가 save/replay compatibility metadata가 된다.
- invalid embedded data는 게임 시작 실패이며 fallback hardcoded data를 쓰지 않는다. TUI/headless production bootstrap은 R3-4에서 fallible `ContentError` 경계로 전환됐고, injected missing level/item regression test가 이를 고정한다. legacy infallible fixture adapter는 production startup 경계가 아니다.
- R3에서 현재 TOML 값의 provenance도 함께 조사한다.

## ADR-0025: core/content/AI/adapter workspace 경계

Status: Implemented and verified by `audit_report_6.md`
Date: 2026-07-15
Decision ID: DEC-WORKSPACE-01

Context:

단일 package에 core, TOML, TUI, LLM scaffold, 두 binary가 있어 dependency upgrade와 compile failure의 영향 범위가 넓다. mutable core type의 노출도 adapter 경계를 약화한다.

Decision:

R1~R4 behavior gate가 통과한 뒤 `aihack-core`, `aihack-content`, `aihack-ai-contract`, `aihack-llm`, `aihack-runtime`, `aihack-tui`, `aihack-headless` workspace로 분리한다. runtime은 core와 content의 조합, content bootstrap, command 실행 및 저장 경계를 소유하고 `GameClient`만 adapter에 노출한다. core는 serde, thiserror, rand만 허용하며 UI와 HTTP dependency를 갖지 않는다. binary 이름과 CLI는 유지한다.

Alternatives Considered:

- 단일 package 유지: dependency와 public API 경계가 compile-time에 보장되지 않아 기각
- workspace를 먼저 수행: behavior bug와 file move regression을 구분하기 어려워 기각
- plugin/dynamic library: 배포 복잡도와 ABI 위험이 커 기각

Consequences:

- R5는 mechanical move만 수행하며 hash 변경을 허용하지 않는다.
- crate public API는 `GameClient`, DTO, registry constructor 중심으로 최소화한다. runtime을 두어 adapter가 core/session 구현에 직접 의존하지 않게 한다.
- file move는 Task당 5개 이하로 나눈다.

## ADR-0026: local LLM은 loopback presentation adapter

Status: Accepted; audit report 11 independent R6 PASS
Date: 2026-07-15
Decision IDs: DEC-AI-01, DEC-LLM-01, DEC-LLM-02

Context:

R6 시작 시 narrative/decision module은 provider trait와 mock만 있었고 실제 transport, 강제 timeout, request와 current session의 correlation이 없었다. 사용자는 local LLM을 메시지 생성과 판정에 쓰려 하지만 core 결정론을 잃으면 빌드·재현 문제를 더 악화시킨다.

Decision:

기본 endpoint는 loopback OpenAI-compatible HTTP이며 기본 enabled는 false다. `reqwest 0.13.4` blocking/json client를 전용 worker thread 1개와 capacity 16 bounded channel 안에 격리한다. connect 500ms, narrative 2000ms, decision 1500ms를 transport가 강제한다. redirect와 system proxy는 끈다. request는 request ID와 `SessionRevision { turn, snapshot_hash }`를 포함한다. narrative와 soft verdict는 presentation-only다. suggestion은 current `ActionSpace`와 revision을 재검증하고 사용자의 `Y` 승인 뒤 normal submit path를 사용한다.

Alternatives Considered:

- LLM이 자유 텍스트로 state patch 반환: 무결성·보안·replay 문제로 기각
- LLM 응답을 core turn에서 동기 대기: provider 장애가 gameplay를 중단시켜 기각
- remote endpoint 기본 허용: privacy와 운영 의존성이 늘어 기각
- soft verdict가 능력치 modifier를 반환: 사실상 core 판정권이 되어 기각

Consequences:

- provider가 없어도 전체 core 게임이 동작한다.
- timeout/invalid/stale response는 hash를 바꾸지 않는다.
- prompt와 response body를 save/replay에 기록하지 않는다.
- remote provider는 v0.3.0 비목표다.
- R6-1은 연결 직전 resolve 결과를 재검사하고 검증된 loopback 주소를 client에 고정한다. R6-2는 opaque request ID, current revision/ActionSpace, submit 직전 revision을 연속 검증해 response-validation 사이의 stale gap도 막는다.
- R6-3은 strict soft payload와 `Neutral / LLM_UNAVAILABLE` fallback을 UI-only state로 보관하고, terminal 복원 뒤 worker를 최대 250ms만 정리한다.
- R6 통합은 G/A/J 요청과 Y/N/R 안전 경로, 상태·modal, 동일 종류 outstanding·250ms cooldown, capacity 16 oldest-drop 표시 큐를 실제 TUI loop에 연결한다. 자동 failure matrix와 live PTY/loopback fixture matrix를 통과했고 `audit_report_11.md`가 checkpoint를 PASS로 종결했다.
- public request는 `schema_version = 1`, `SessionRevision`, `LlmObservationView`, 독립 `ActionSpace`, `LlmRequestKind`를 소유하고 enqueue 전에 version·bounds·canonical size를 검증한다. response envelope도 TUI payload 수용 전에 version을 거부한다.
- timeout의 단일 기본값은 connect 500ms, narrative 2000ms, decision/soft-adjudication 1500ms이며 helper와 env config가 같은 상수를 사용한다. decision rationale은 trim 후 1..=160자다.
- v0.3.0 built-in runtime locale은 English로 한정한다. 다국어 README와 provider Unicode pass-through는 runtime 5-locale catalog 완료 증거가 아니다.
- deterministic loopback fixture와 PTY script는 success/timeout/stale/down 및 pending-exit 복원 순서를 저장소에서 재현한다. `audit_report_11.md` 독립 재감사에서도 같은 evidence가 통과했다.
- 실제 모델 smoke는 R6 필수 gate가 아니다. 최종 통합에서 별도 호환성 증거가 필요할 때만 localhost OpenAI-compatible 임시 adapter가 Google AI Studio Gemini 같은 원격 API를 대리 호출한다. AIHack은 계속 loopback만 호출하며 API key는 adapter 환경변수에만 주입하고 model ID는 실행 시점에 확인한다.

## ADR-0027: provenance approval이 runtime 포함의 선행 조건

Status: Implemented; owner license decision superseded by ADR-0030
Date: 2026-07-15
Decision ID: DEC-LICENSE-01

Context:

legacy tree에는 Apache-2.0 text와 NGPL text가 함께 있고 적용 범위가 명시되지 않았다. `LICENSE.NGPL` 33..35행은 반복 단어로 손상되어 있다. root package는 UNLICENSED이며 현재 TOML 값의 출처도 완전히 기록되지 않았다.

Decision:

`PROVENANCE.md`의 상태를 Unknown, Reviewed, Approved, Blocked로 고정한다. runtime에는 Approved 자산만 포함한다. legacy code/data/string은 기본 Blocked이며 path import와 복사를 금지한다. 공식 NetHack 3.6.7 archive는 공식 SHA-256을 확인한 뒤 source locator로만 사용한다. 배포 라이선스 결정은 프로젝트 소유자 또는 적격 검토자의 승인을 요구한다.

Alternatives Considered:

- legacy의 Apache file을 전체 tree license로 간주: 적용 notice 증거가 없어 기각
- 손상된 NGPL을 그대로 배포 notice로 사용: 원문 신뢰성이 없어 기각
- 코드만 새로 쓰면 provenance 생략: 데이터·문구·규칙 표현의 출처 위험이 남아 기각

Consequences:

- R7 전에는 release artifact 외부 게시를 중단한다.
- 공식 source metadata 확인은 재사용 승인과 동일하지 않다.
- compatibility record마다 source locator와 provenance status가 필요하다.
- 이 ADR과 inventory는 법률 자문을 대체하지 않는다.
- R7 engineering review는 공식 archive와 `dat/license` checksum을 검증하고 legacy direct import 0건을 자동화한다. 내부 build/test inclusion approval은 외부 배포 승인과 분리하며, content data와 scenario는 project owner 또는 적격 검토자가 범위를 승인할 때까지 `Reviewed`다.
- `audit_report_12.md` 이후 approval gate는 상태 문자열을 신뢰하지 않는다. runtime coverage, reviewer/date/license/scope/notice/evidence, content checksum, scenario schema/function과 Blocked reference를 모두 machine validation한 경우에만 PASS한다.
- `audit_report_13.md`의 phase-cycle 시정에 따라 R7은 asset/scenario provenance를, R8은 root distribution license/version/packaging을 소유한다. R7 PASS만으로 외부 배포를 허용하지 않는다.
- release checkpoint는 caller environment가 지정한 root를 신뢰하지 않고 script-relative canonical repository만 검사한다.
- release bundle 무결성은 Linux Bash와 Windows PowerShell verifier가 동일한 required file, excluded path, metadata/record equality, checksum exact-set와 tamper negative matrix를 갖는다. 한 OS의 positive build만으로 다른 OS의 fail-closed parity를 대신하지 않는다.

## ADR-0029: 미승인 provenance를 R8 실제 런칭 게이트로 이관

Status: Superseded for licensing by ADR-0030; R7 pass with known risks remains historical
Date: 2026-07-18
Decision ID: DEC-LICENSE-02

Context:

`audit_report_14.md`까지 R7의 checksum inventory, legacy 격리, compatibility trace, fail-closed validator와 전체 회귀는 검증됐지만 PROV-0004와 NH367 scenario 10개의 actual license approval는 남아 있다. 사용자는 실제 런칭이 개시될 때 이 범위를 명확히 검토하고, 현재 개발 단계에서는 최종 검토사항으로만 유지하기로 결정했다.

Decision:

R7 engineering 단계는 `PASS WITH KNOWN RISKS`로 종결한다. SC-LICENSE-01, content/scenario actual approval, root distribution license와 notice는 R8 실제 런칭 전 필수 게이트로 이관한다. 이 결정 당시에는 `Reviewed`, `UNLICENSED`, `publish = false`와 외부 배포 차단을 유지했다. 이후 상태는 ADR-0030이 대체한다.

Alternatives Considered:

- actual approval 없이 `Approved`로 변경: 근거를 조작하므로 기각
- R7 전체를 계속 HOLD: 구현·테스트 진행과 외부 배포 위험을 불필요하게 같은 gate로 묶어 기각
- 라이선스 검토를 완전히 면제: 런칭 시 법적·배포 위험이 남아 기각

Consequences:

- R8 비배포 준비와 후속 개발은 진행할 수 있다.
- R7 PASS는 라이선스 적합성 보장이나 외부 배포 허가가 아니다.
- 당시 `scripts/r7_checkpoint.sh`의 HOLD는 실패가 아니라 R8에 남은 승인 evidence를 표시했다. ADR-0030 반영 후에는 PASS한다.
- 외부 배포 전 project owner 또는 적격 검토자가 approval authority/evidence를 기록해야 한다.
- 승인 불가 자산은 Blocked 처리하고 독립 작성 자산으로 교체한다.

## ADR-0030: NetHack 3.6.7 파생물 분류와 whole-work NGPL 배포

Status: Implemented; report 21/24/25/26 historical closure retained, report 27 current HOLD
Date: 2026-07-20
Decision ID: DEC-LICENSE-03

Context:

AIHack은 NetHack 3.6.7 원본 소스를 AI 추론에 제공해 의도와 관찰 가능한 규칙을 추출한 뒤 Rust 구조로 재작성했다. 결과 소스의 표현과 아키텍처가 크게 다르더라도 프로젝트 소유자는 원 저작권을 존중해야 하는 파생물로 취급하기로 명시했다. 따라서 clean-room 독립 저작물 전제를 유지하거나 permissive license를 선택하는 것은 실제 생성 과정과 소유자의 위험 판단에 맞지 않는다.

Decision:

AIHack 전체를 NetHack 3.6.7의 AI-assisted semantic rewrite 파생물로 분류하고 SPDX `NGPL`을 workspace 전체에 적용한다. 이 project-owner 결정의 범위와 직접 지시는 `AIHACK-OWNER-2026-07-20-NGPL-01`로 기록한다. root `LICENSE`는 공식 3.6.7 archive의 `dat/license`와 byte-for-byte 동일한 원문을 사용하고 SHA-256 `93a3ae2cb8dee482daddfaebe53bcffe5b114b603def19b4dca21621cbc5a747`로 고정한다. `NOTICE`에는 원 저작권, 파생·수정 사실과 AIHack 기여를 명시한다. 바이너리 배포에는 `LICENSE`, `NOTICE`, `MODIFICATIONS.md`, `PROJECT_OWNER_LICENSE_APPROVAL.md`, commit이 확장된 `RELEASE-METADATA`, `SHA256SUMS`와 해당 바이너리를 만든 커밋의 complete corresponding source archive를 동반한다. metadata의 필수 key는 각각 정확히 한 번 존재하고 owner/modification ID 전체 값은 함께 배포되는 record ID와 일치해야 한다. 검증되지 않은 `legacy_nethack_port_reference/`는 release archive에서 제외한다.

Alternatives Considered:

- MIT/Apache-2.0 등 permissive license 적용: 소유자의 파생물 분류와 NGPL whole-work 조건에 맞지 않아 기각
- 코드 표현이 다르다는 이유로 독립 저작물로 선언: 원본 소스 기반 의도 추론이라는 실제 생성 과정을 축소하므로 기각
- 손상된 legacy `LICENSE.NGPL` 복구본 사용: 공식 원문이 아니며 보존 증거를 훼손하므로 기각
- 라이선스 결정을 외부 게시 직전까지 다시 연기: 이미 소유자 판단이 확정되어 문서·manifest 불일치를 지속할 이유가 없어 기각

Consequences:

- 모든 workspace package는 version 0.3.0, `license = "NGPL"`, `publish = false`를 유지한다.
- PROV-0001..0012와 NH367-C001..C010은 소유자 승인 authority, 날짜, scope, notice와 `AIHACK-OWNER-2026-07-20-NGPL-01` evidence를 기록한다.
- R8 checkpoint는 NGPL 정확성, 공식 LICENSE checksum, NOTICE, modification manifest, release commit metadata와 source archive 계약을 fail-closed로 검사한다.
- source archive는 `.git/` history에 의존하지 않고 `MODIFICATIONS.md`의 scope/date와 `RELEASE-METADATA`의 commit을 수신자에게 전달한다.
- 라이선스 정비 완료는 독립 R8 기술 감사 `PASS`나 외부 게시 실행을 자동 승인하지 않는다.
- 이 결정과 프로젝트 기록은 qualified legal opinion이나 법률 자문을 대체하지 않는다.
- Release verification update (2026-08-23): `audit_report_21.md`가 report 20을, `audit_report_24.md`와 implementation SHA `2519bc8e0ede81c39f46b5778e62a41d4ca66901`의 Actions run `32107862171`이 report 23/24를 historical closed했다. final multi-audit report 1의 첫 시정은 `docs/audit/audit_report_25.md`가 production 결함을 재현해 partial evidence로 강등했으며, report 25 시정·same-SHA CI·독립 재감사 전까지 전체 PASS나 외부 게시를 승인하지 않는다.
