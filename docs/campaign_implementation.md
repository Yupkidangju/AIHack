# P1–P3 구현·검증 기록

2026-09-05. 기준 작업은 미커밋 P0 위의 사용자 P3까지 구현 요청이다. 계약은 `docs/campaign_spec.md`. 봉인 감사 원문은 변경하지 않았다.

## 구현

- P1: core `campaign.rs` Role/CampaignState, runtime bootstrap/XP 소비와 save validator, TUI 역할 keyboard/mouse 및 HUD.
- P2: runtime `campaign_map.rs`의 combat RNG와 독립인 40×20 연결 방/통로 생성, Main 1..6/Mines 1..2, Main 3 branch 입출구와 실제 생성 좌표 teleport.
- P3: content `item.quest.ascension` → typed Quest item → Pickup/Drop/Inventory → Main 1 Ascend → Victory/score → 결과 화면/새 게임. Headless는 Victory와 사망을 구분한다.
- Save: legacy None은 schema 1/생략 field로 보존, campaign은 schema 2. 구형 reader는 새 저장을 거부한다. 기존 runtime capability/atomic 파일 쓰기 및 mutable API 경계는 유지한다.

## 재현 증거

- 신규 타입/StartCampaign/Amulet/Victory 부재의 compile RED, 역할 성장 및 위조 XP 승인 behavioral RED를 확인하고 수정했다.
- 첫 정상 완주가 legacy 무게 80에서 막혔다. campaign 120 계약을 문서화하고 actual movement/observation/HUD에 연결했다. 과다 loot는 테스트도 실제 Quaff/Drop으로 관리하며 guard를 우회하지 않는다.
- `tests/campaign.rs`: 3 seed/3 role 실제 명령 완주, 광산 왕복, 목표 drop/pickup, 저장·RNG 연속성, 마지막 Ascend replay, 반복 최종 hash, UI 최소 크기 60×24 role/새 게임, 성장 후 실제 attack roll +1, 위조 schema/XP/stat/amulet/topology/Victory, teleport landing 검증.
- Windows ConPTY: legacy와 campaign 각각 서로 다른 두 실제 CLI 프로세스 저장→종료→재시작→추가 Wait의 최종 hash 일치.
- content 체크섬 gate가 신규 quest 데이터의 미반영 manifest를 검출했다. 원래 10개 item 정의는 보존하고 신규 데이터 출처·이전/현재 SHA를 PROVENANCE에 기록한 뒤 manifest를 갱신했다. checksum negative tests는 유지한다.
- content crate의 registry golden hash도 신규 item 포함에 따라 `c491b83c6f499a62` → `f106d044fee3e340`으로 변경된다. 기존 item/monster/level 조회와 정확한 hash assertion을 유지하고 기대값만 새 콘텐츠 기준으로 갱신한다. legacy world snapshot hash와 별개의 콘텐츠 registry hash다.

## Gate 결과

- `cargo test --workspace --all-targets --locked --no-fail-fast`: **491 PASS / 0 FAIL / 0 ignored**, 89개 test binary, exit 0. 기존 P0 478개와 캠페인 12개·V2 ConPTY 1개를 포함한다.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS.
- `cargo build --workspace --all-targets --release --locked`: PASS.
- 표적 campaign 12 tests 및 legacy/V2 ConPTY 4 tests: PASS.
- 전체 로그: `runtime/campaign-evidence/workspace-tests.log`, SHA256 `464a753f8561c0d5ba3564d325c8434b96f41a304d3620cf6e9594b15fa35317`. 최종 검증 OS는 Windows다.

P1–P3의 명시된 첫 캠페인 범위는 local verified로 완료했다. 독립 감사 PASS나 NetHack 전체 구현 완료 선언과는 구분한다.

실제 headless CLI: seed 42/Knight/wait-v1 10턴 저장 후 별도 실행에서 12턴까지 복원 진행(2 accepted/2 submitted)과 새 게임에서 직접 12턴 진행(12 accepted/12 submitted)의 최종 hash는 모두 `8236e9774f4b6d40`이다. report는 `runtime/campaign-evidence/headless-start.json`, `headless-resume.json`, `headless-direct.json`에 생성했다. runtime 증거는 Git 추적 대상이 아니다.

리뷰: generator는 새 모듈, 성장·semantic 검증은 `campaign.rs`로 분리했다. 기존 10개 item 값, 기존 2층 생성, V1 field/hash, legacy 운반 80은 유지한다. 신규 XP 예산 누적은 saturating arithmetic, 승리 score 검사는 기존 범위 검증 이후 수행하여 malformed save가 overflow/panic을 만들지 않도록 했다. 운영 계정·네트워크·인증 설정은 변경하지 않았다.

## 제한과 인수인계

첫 bounded campaign이며 NetHack 전체 직업/마법/종족/분기/신앙/원소계 구현을 뜻하지 않는다. 생성 map에는 현재 문·함정이 없고 V2는 generator v1 map을 검증한다. save의 semantic validator는 로컬 사용자의 완전한 파일 위조를 막는 암호학적 인증 수단이 아니다. XP cap은 180이며 실제 8층 콘텐츠의 보상 총량에 따라 도달 레벨은 더 낮을 수 있다.

release/게시 승인, modification notice와 final-SHA bundle 갱신, 새 양 OS CI는 별도 작업이며 이번 기능 검증으로 승인하지 않는다. 커밋·푸시는 실행하지 않았다.

## P4 확장 구현·검증 기록 (2026-11-26)

### 구현

- **P4a (문/비밀문·함정·전이 검증)**:
  - `aihack-core::domain::tile::TrapKind`에 `Dart`(2 피해)·`Bear`(2 피해 + `paralysis_turns = 2`, 턴 종료 감소 후 다음 1턴 완전 마비)를 추가하고, `session::submit_move`가 발동한 실제 `TrapKind`를 `DeathCause::Trap { trap }`에 기록하도록 연결했다.
  - `systems/movement.rs`가 `TileKind::Trap(_) | TileKind::HiddenTrap(_)` 위 적 몬스터에 대한 근접 bump attack을 허용한다.
  - `campaign_map::generate`(generator v2)가 모든 캠페인 층 `(7, centers[0].y)`에 `Door(Closed)`, 홀수 층 `(17, centers[1].y)`에 `Door(Closed)`, 짝수 층에 `HiddenDoor`, `centers[1]-(1,0)`에 `Pit`(홀수 층)/`Dart`(짝수 층), Main 3/5/6 및 Mines 2 `centers[2]-(1,0)`에 `Bear` `HiddenTrap`을 결정적으로 배치한다.
  - `save.rs`의 엄격 동일 맵 비교를 `campaign_map::validate_persisted_map`으로 교체해 정당한 문 개폐·비밀문 공개·함정 공개 전이만 허용하고 임의 변조(벽 뚫기, 계단 삭제/추가, 함정 제거/변형, 임의 문 위조)를 거부한다.
- **P4b (성직자·신규 5아이템·신규 3몬스터)**:
  - 제4역할 `Role::Cleric`(`HP 24, hit +3, dmg +1`, 추가 장비 `[Mace, PotionExtraHealing, ScrollIdentify]`, 초기 무게 `97/120`, TUI `[4] Cleric` 및 키 `'4'`, headless `--role cleric`)을 추가했다.
  - `items.toml`에 `short_sword`, `mace`, `chain_mail`(`ac_bonus=3`), `potion.extra_healing`(`2d8+8`), `scroll.charging`(`ConsumableEffect::RechargeWand`) 5종을 추가했다.
  - `monsters.toml`에 `kobold`(HP 5, 1d4, diff 2), `orc`(HP 10, 1d6, diff 3), `ogre`(HP 16, 1d8, diff 5) 3종을 추가하고, `entity::natural_attack_profile` 및 캠페인 8층 생성 테이블에 연결했다.
  - 기존 11종 아이템·3종 몬스터·`main_1.toml`·`main_2.toml` 및 legacy V1 저장/해시는 변경하지 않았다.
- **P4c (TUI·Headless·Provenance)**:
  - `aihack-ai-contract`에서 `DoorState`를 재노출하고 `render_map.rs`/`render_panels.rs`가 닫힌 문(`+`, `"closed door"`)·열린 문(`-`, `"open door"`)·함정 라벨(`"pit trap"`, `"dart trap"`, `"bear trap"`)·신규 몬스터/아이템 및 60×24 `[4] Cleric`을 표시하도록 갱신했다.
  - `docs/provenance/r7-content.sha256`와 `PROVENANCE.md`를 LF SHA-256으로 동기화했고, `crates/aihack-content/tests/registry.rs` golden content hash를 `f3767a7764223506`으로 갱신했다.

### 검증 증거

- `tests/campaign.rs`: 4역할(`Knight`, `Scout`, `Mage`, `Cleric`) 실제 명령 8층 왕복·승천 완주 및 60×24 UI/키보드 선택(**12 PASS / 0 FAIL**).
- `tests/campaign_p4.rs`: 캠페인 문/비밀문(`Open`/`Close`/`Kick`/`Search`/V2 roundtrip), `Pit`/`Dart`/`Bear` 함정·마비·`ScrollReveal`·치명적 `DeathCause`, 전이 검증 허용 및 변조 거부, `Cleric` 초기 키트 및 실제 몬스터 처치 레벨 10 성장, 신규 5아이템(`ScrollCharging` 완드 충전/상한·`ArmorChainMail` AC `-3`)·3몬스터·함정 위 근접 공격, TUI 렌더링(**6 PASS / 0 FAIL**).
- `apps/aihack-headless/tests/headless_contract.rs`: `--role cleric` V2 저장·리포트 생성 포함 **7 PASS / 0 FAIL**.
- `tests/content_validation.rs`(신규 5종 ID 포함 **13 PASS / 0 FAIL**) 및 `tests/traps.rs`(**6 PASS / 0 FAIL**).
- headless CLI smoke (`--seed 42 --role cleric --turns 10 --policy wait-v1`): `accepted_turns=10`, `final_state=Playing`, `final_hash=8e58029000b64cbb`.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo build -j 4 --workspace --all-targets --release --locked`, `cargo deny check licenses bans sources`, `cargo audit`(0 vulnerabilities), `scripts/r7_checkpoint.sh`(PASS), `scripts/r8_checkpoint.sh`(PASS).
- 커밋·푸시는 실행하지 않았다.

## P5 확장 구현·검증 기록 (2026-11-26)

### 구현

- **P5a (BUC·인챈트·잠긴 문·강화/해저주 두루마리)**:
  - `aihack-core::domain::item`에 `BucStatus`(`Blessed`/`Uncursed`/`Cursed`, 기본 `Uncursed`), `MIN_ITEM_ENCHANTMENT = -3`, `MAX_ITEM_ENCHANTMENT = 5`, `ConsumableEffect::{EnchantWeapon, EnchantArmor, RemoveCurse}`, `EntityPayload::Item`의 serde-omitted optional 필드 `buc`·`enchantment` 및 별도 접근자(`Entity::item_buc`, `Entity::item_enchantment`, `EntityStore::set_item_buc`, `EntityStore::set_item_enchantment`)를 추가했다. `Entity::item()` 5-튜플 시그니처와 V1 기본 JSON 직렬화 생략 계약을 그대로 보존했다.
  - `systems/combat.rs`·`systems/items.rs`·`save.rs`에서 장착 무기 인챈트(`+enchant` 명중/피해, `Blessed` 명중 `+1`), 장착 방어구 인챈트(`adventurer_template().ac - (ac_bonus + enchant)`), 물약/충전 주문서 BUC 효과(`Heal` Blessed `+4` / Cursed `-4` 최소 1, `ScrollCharging` Blessed 해저주+충전 / Cursed 0 소진), `Cursed` 장착 아이템 `Wield`/`Wear`/`Drop`/`Throw` 금지, 강화/해저주 두루마리 3종(`ScrollEnchantWeapon`, `ScrollEnchantArmor`, `ScrollRemoveCurse`), 저장 범위/AC 검증을 구현했다.
  - `DoorState::Locked` 및 `doors::kick_door` 해제를 추가하고 `campaign_map`이 `Mines 2` `(7, centers[0].y)`에 `Door(Locked)`를 생성하도록 연결했으며, `validate_persisted_map`이 생성된 `Locked` 위치에서만 `Locked -> Open <-> Closed` 전이를 허용하고 임의 `Closed`/바닥 타일의 `Locked` 변조를 거부하도록 강화했다.
- **P5b (독 상태이상·치료 물약·화염 지팡이·신규 몬스터 3종·Provenance)**:
  - `WorldState`/`SavedWorldV1`/`PlayerObservation`/`GameSnapshot`에 serde-omitted `poison_turns: u8`과 `DeathCause::Poison`을 추가하고 `session::accept_turn`에서 중독 틱(-1 HP, 경고 메시지, 0 HP 시 `DeathCause::Poison`)을 처리했다.
  - `items.toml`에 `PotionCure`(`cure_status`, `1d6+4`·BUC 보정, 독/마비/환각 해제), `WandFire`(`fire`, `charges=4`, `2d6+2`), `ScrollEnchantWeapon`, `ScrollEnchantArmor`, `ScrollRemoveCurse` 5종을, `monsters.toml`에 `GiantSpider`(`s`, HP 8, `poison_on_melee`), `GnollArcher`(`G`, HP 11, `ranged_on_sight` 거리 5 직교 투사), `Troll`(`T`, HP 22, `regenerate_hp`) 3종을 추가하고 `docs/provenance/r7-content.sha256`, `PROVENANCE.md`, `crates/aihack-content/tests/registry.rs` golden hash(`5b9ee4946ebf05d7`)를 동기화했다.
  - `bootstrap.rs` `campaign_world`가 `Mines 1`(`GiantSpider`, `PotionCure`, `Cursed -1 Dagger`), `Main 4`(`GnollArcher`, `PotionCure`), `Main 5`(`Troll`, `WandFire`), `Main 1/2/3`(`ScrollEnchantWeapon`, `ScrollRemoveCurse`, `Blessed +1 ShortSword`, `ScrollEnchantArmor`), `Mines 2`(`+1 Mace`)를 배치한다.
- **P5c (TUI·Headless `survival-v1`)**:
  - `aihack-ai-contract`에서 `BucStatus`를 재노출하고 `apps/aihack-tui`(`labels.rs`, `render_map.rs`, `render_panels.rs`, `input.rs`, `play_menu.rs`, `mod.rs`)가 잠긴 문(`+`, `"locked door"`), `POIS:<n>` HUD, 비기본 BUC/인챈트 라벨(`+1 blessed ...`, `-1 cursed ...`), 신규 몬스터/아이템, `"Killed by poison"` 사인을 표시하도록 연결했다.
  - `apps/aihack-headless` `survival-v1`은 `poison_turns > 0` 시 `PotionCure` 우선 음용과 `Door(Locked)` 대상 `Kick` 후보를 지원한다.

### 검증 증거

- `tests/campaign_p5.rs`: BUC/인챈트 전투/AC/저주 해제/강화 상한/물약·주문서 BUC/범위·AC 변조 거부, `Mines 2` 잠긴 문 생성/`Open`·`Move` 거부/`Kick` 해제/변조 거부, `GiantSpider` 중독·독 틱·`PotionCure` 해독·`DeathCause::Poison` 저장·`GnollArcher` 직교 사격/차단·`Troll` 재생·`WandFire`, TUI(`POIS:3`, `"locked door"`, BUC/인챈트 라벨, 글리프, `"Killed by poison"`)·Headless `survival-v1`·V1 기본 JSON 생략 보존(**4 PASS / 0 FAIL**).
- `tests/campaign.rs`(**12 PASS / 0 FAIL**, 4역할 8층 왕복·승천 완주 포함), `tests/campaign_p4.rs`(**6 PASS / 0 FAIL**, `Mines 2` 입구 `Door(Locked)` 정렬), `tests/content_validation.rs`(**13 PASS / 0 FAIL**), `tests/r8_documentation.rs`(**PASS**).
- 전체 게이트: `cargo fmt --all -- --check` PASS, `cargo clippy --workspace --all-targets --locked -- -D warnings` PASS, `cargo test -j 4 --workspace --all-targets --locked --no-fail-fast` (**505 PASS / 0 FAIL / 0 ignored**, 92 test results), `cargo build -j 4 --workspace --all-targets --release --locked` PASS, `cargo deny check licenses bans sources` PASS, `cargo audit` PASS(0 vulnerabilities, 기존 허용 `rustix`/`tokio` warning 2건), `scripts/r7_checkpoint.sh` PASS, `scripts/r8_checkpoint.sh` PASS.
- 커밋·푸시는 실행하지 않았다.

## P6 확장 구현·검증 기록 (2026-11-26)

### 구현

- **P6a (반지 슬롯·독/화염 저항·`FireAnt`)**:
  - `aihack-core::domain::item`에 `ItemClass::Ring`, `EquipmentSlot::Ring`, `RingEffect::{PoisonResistance, FireResistance, Protection}`, `ItemKind::{RingPoisonResistance, RingFireResistance, RingProtection}`, `ItemData::ring_effect`(optional serde omission), `Inventory::equipped_ring`(optional serde omission), `CommandIntent::{PutOn { item }, RemoveRing}`, `InventoryAction::PutOn`을 추가했다. `Entity::item()` 5-튜플과 legacy V1 JSON 호환성은 그대로 보존했다.
  - `systems/items.rs`의 `recompute_player_ac`와 `save::validate_inventory`가 통합 방어력 공식 `adventurer_template().ac - (body_ac + body_enchant + ring_ac + ring_enchant)`을 적용·검증하며, `Cursed` 반지는 `ScrollRemoveCurse` 해제 전까지 `RemoveRing`·`PutOn` 교체·`Drop`·`Sell`을 차단한다.
  - `monsters.toml`/`domain/monster.rs`에 `MonsterKind::FireAnt`(`a`, HP 12, AC 3, hit `+2`, `1d6`, `MonsterPassive::FireOnMelee`, speed 12, diff 5)를 추가하고 `Main 5`/`Main 6`에 배치했으며, `Main 4`(`RingFireResistance`), `Mines 1`(`RingPoisonResistance`), `Mines 2`(`+1 RingProtection`)에 반지를 스폰한다. `FireOnMelee`는 화염 저항이 없으면 추가 `1d6` 화염 피해(`PassiveAttackTriggered` + 경고)를 입히고, 독 저항은 `GiantSpider`의 `PoisonOnMelee`를 차단하며 기존 `poison_turns` 틱 피해를 0으로 줄인다.
- **P6b (`TileKind::Shop`·골드 경제·상점 구매/판매·Provenance)**:
  - `TileKind::Shop`(이동/시야 통과 가능)을 추가해 `campaign_map`이 `Main 2` `(centers[1].x, centers[1].y - 1)`와 `Mines 1` `(centers[1].x + 1, centers[1].y - 1)`에 결정적으로 배치하고 `validate_persisted_map`이 비인가 `Shop` 변조를 거부하도록 유지했다.
  - 캠페인 전용 골드 스케일링을 적용해 `campaign_world` 시작 골드 `50` 및 처치 골드 `difficulty * 25`를 지급한다(`campaign.is_none()` legacy 모드는 초기 `0` 및 `difficulty` 유지).
  - `systems/shop.rs`가 9종 `SHOP_CATALOG`, `Buy { kind }` (`base_price.max(10)`), `Sell { item }` / `InventoryAction::Sell` (`(base_price / 2).max(1)`, `AmuletAscension`·저주 장착 아이템 거부)을 처리하며, `crates/aihack-content/tests/registry.rs` golden hash(`aa720d339c7f0c77`)·`docs/provenance/r7-content.sha256`·`PROVENANCE.md`를 동기화했다.
- **P6c (TUI·Headless `survival-v1`·AI 계약/LLM 어댑터)**:
  - `PlayerObservation`에 serde-omitted `gold`, `fire_resistant`, `poison_resistant`를 추가하고 `crates/aihack-llm/src/decision.rs` `wire_command_value`에 `PutOn`·`RemoveRing`·`Buy`·`Sell` 직렬화를 연결했다.
  - `apps/aihack-tui`가 `Shop` (`'$'`, `"shop"`), `FireAnt` (`'a'`, `"fire ant"`), 반지 (`'='`, `"ring of ..."`), 상태 HUD(`G<gold>`, `RES:F|P|FP`), `'P'`(착용)·`'T'`(해제)·`'$'`(상점 구매/판매 `PlayMenu`)를 렌더링·처리한다.
  - `apps/aihack-headless` `survival-v1`은 반지 슬롯이 비어 있을 때 소지 반지를 우선 `PutOn`하도록 확장했다.

### 검증 증거

- `tests/campaign_p6.rs`: 반지 AC 스택/해제/저장 검증·저주 가드, `FireAnt` `FireOnMelee`/독 저항·화염 저항, `Main 2`/`Mines 1` `Shop` 배치/위조 거부·캠페인/legacy 처치 골드 차등·9종 카탈로그 구매/판매(자금 부족/인벤토리 포화/아뮬렛/저주 거부), TUI(`$`/`a`/`=`/HUD/상점 메뉴 구매)·Headless `PutOn`·legacy V1 `PlayerObservation` serde 생략 보존(**4 PASS / 0 FAIL**).
- `tests/campaign.rs`(**12 PASS / 0 FAIL**, `Main 4` `RingFireResistance` 장착 포함 4역할 8층 완주), `tests/content_validation.rs`(**13 PASS / 0 FAIL**, 반지 3종 ID 포함), `crates/aihack-content/tests/registry.rs`(**1 PASS / 0 FAIL**, `aa720d339c7f0c77`), `tests/r8_documentation.rs`(**PASS**).
- 전체 게이트: `cargo fmt --all -- --check` PASS, `cargo clippy --workspace --all-targets --locked -- -D warnings` PASS, `cargo test -j 4 --workspace --all-targets --locked --no-fail-fast` (**509 PASS / 0 FAIL / 0 ignored**, 93 test results), `cargo build -j 4 --workspace --all-targets --release --locked` PASS, `cargo deny check licenses bans sources` PASS, `cargo audit` PASS(0 vulnerabilities, 기존 허용 `rustix`/`tokio` warning 2건), `scripts/r7_checkpoint.sh` PASS, `scripts/r8_checkpoint.sh` PASS.
- 커밋·푸시는 실행하지 않았으며 `feat/p5-gameplay-expansion` 브랜치도 삭제하지 않았다.
