# Campaign P1–P3 계약

2026-09-05. 사용자 승인 범위는 역할·성장·탐험·승천을 연결한 P3까지의 구현이다. 다음 수치는 첫 캠페인의 설계값이며 NetHack 공식 balance 복제를 주장하지 않는다.

## P1 roles-growth

`CommandIntent::StartCampaign { role: Role }`는 CharacterCreation에서만 허용, non-turn이다. `Role={Knight,Scout,Mage}`. Creation Wait는 기존 2층 Adventurer 호환 run이다. UI는 1/2/3 및 mouse CTA로 새 역할을 선택한다.

| Role | HP | hit | damage | 공통 5개 외 소지품 |
| --- | --- | --- | --- | --- |
| Knight | 28 | 4 | 2 | leather armor, healing potion |
| Scout | 22 | 5 | 1 | food ration 2개 |
| Mage | 18 | 3 | 0 | magic missile wand, healing potion 2개 |

기본 AC=0, 장착은 기존 Wield/Wear를 사용한다. player가 직접 처치한 살아 있는 monster만 `max(difficulty,1)*10` XP를 정확히 한 번 준다. `level=min(10,1+xp/20)`, XP cap=180. 레벨당 max_hp/현재 hp +4, hit +1, damage +1. 장비/기도 보정은 독립이다. CampaignState는 role/xp/고유 amulet EntityId를 저장하고 level은 계산한다. 모든 변경은 기존 transaction submit 경계를 따른다.

## P2 exploration

캠페인은 시작 kit와 원정 자원에 맞게 운반 한도 120을 사용한다(legacy 80 유지). 한도 초과 시 Move는 legal에서 제외하고 실제 submit도 거부한다. HUD에 무게/한도를 표시한다. 한도를 넘는 물건을 계속 줍는 완주 스크립트는 사용·버리기 등의 실제 행동으로 짐을 줄여야 한다.

이 제한은 위치를 바꾸는 이동에 적용하며 인접 적을 향한 bump attack은 기존 전투 경로를 유지한다. campaign observation은 carried_weight/carrying_capacity도 포함한다. HUD의 A는 아뮬렛 보유 표시다. Headless는 목표 턴에 도달한 Victory를 성공 결과로 보존하며, 성공 종료 뒤 더 높은 턴을 요구하면 `VictoryBeforeTarget` 오류로 추가 submit 없이 종료한다.

generator v1은 seed와 LevelId에서 결정되며 combat RNG를 소비하지 않는다. 40×20 연결된 방/통로, Main 1..6과 Mines 1..2를 bootstrap에서 한 번 생성하고 map/monster/item을 저장한다. 재진입 시 재생성하지 않는다. 현존 monster와 food/potion을 연결된 위치에 배치한다.

현재 생성 맵에는 변경 가능한 문/함정이 없으므로 V2 load는 seed/LevelId의 generator v1 map과 일치해야 한다. 순간이동 scroll을 소지한 확장 저장에서도 고정 fixture 좌표 대신 실제 생성 map의 계단 landing을 사용한다. 해당 소비 경로 역시 회귀 대상이다.

Main 3 위 계단의 `EnterBranch`(B/명령 메뉴)는 Mines 1 위 계단으로 이동한다. Mines 1 Ascend는 Main 3 위 계단으로 복귀한다. 나머지 계단은 같은 branch 이웃 층이다. Main 1 위 계단은 지상 출구, Main 6/Mines 2에는 아래 계단이 없다. 광산은 선택 탐험 경로다. seed별 BFS와 fixture 주입 없는 실제 명령 완주로 도달성을 검증한다.

## P3 ascension

`AmuletAscension`: glyph `"`, class Quest, weight=1, price=0, 장비/소모/충전 효과 없음. campaign마다 단 하나 Main 6 목표 위치에 생성한다. 기존 Pickup/Drop/Inventory를 사용하며 ID와 kind를 함께 검사한다. 소지한 살아 있는 player가 Main 1 위 계단에서 Ascend하면 `RunState::Victory { final_score }`로 종료한다. 잘못된 위치/목표 없음/죽음은 승리 불가다. 성공 명령은 1턴 소비하며 monster 반격 없이 종료, 추가 게임 mutation은 거부한다. 점수는 기존 계산+10,000. 결과는 사망/자진 종료와 구분하고 N 새 게임/Q 종료를 제공한다.

## 저장·API

Headless의 `--role knight|scout|mage`는 해당 campaign을 생성한 뒤 기존 policy/replay를 실행한다. 생략은 legacy, `--load`와 동시 사용은 오류다. 재현 명령은 `cargo run -p aihack-headless --locked -- --seed 42 --role knight --turns 10 --policy wait-v1`이다.

기존 Rust save envelope에 optional campaign을 추가한다. None은 serialize 생략하고 wire schema_version=1, Some이면 schema_version=2이다. reader는 V1 legacy/V2 campaign 조합만 허용한다. 구형 reader는 V2를 거부하므로 성장/목표를 조용히 유실하지 않는다. 기존 run의 자동 campaign 변환이나 저장 덮어쓰기는 없다. V2는 정확한 8개 level 집합, XP cap/role 성장 stats, 고유 목표 ID-kind-location, Victory의 소지/출구/alive/score를 기존 validator와 함께 검증한다.

snapshot은 campaign=None을 생략해 기존 hash를 유지한다. observation의 optional campaign 요약은 role/xp/level/목표 보유를 제공한다. 새 enum variant에 따라 Rust exhaustive consumer 수정은 필요하다. 미게시 workspace 소스 확장으로 관리하고 자동 버전/릴리스 승격은 하지 않는다.

## 검증 및 경계

`cargo test -p aihack --test campaign --locked`, TUI dispatcher tests, `cargo test --workspace --all-targets --locked`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo build --workspace --all-targets --release --locked`.

역할별 차이·성장 전후 전투·3 seed BFS와 완주·층 재진입·저장/RNG 연속성·위조 목표/성장/topology·기존 478 회귀를 검사한다. NetHack 전체 규칙 동등성/외부 독립 감사/CI/게시 승인은 별개다. 기존 Rust 구조와 한국어 경계 주석, native cargo integration test 관례를 유지한다.

## P4 캠페인 확장 (2026-11-26)

P1–P3 첫 캠페인 위에 생성 던전 문·비밀문·다종 함정 상호작용과 4번째 역할·신규 장비/소모품·신규 몬스터를 확장한다. 기존 2층 Adventurer run, V1 저장/해시, 기존 11개 아이템·3종 몬스터 정의와 `NH367-C001..C010`/`P8-G01..G20` 호환성은 그대로 보존한다.

### P4a 문·비밀문·함정·저장 검증

- `TrapKind`: 기존 `Pit`(3 피해) 외에 `Dart`(2 피해), `Bear`(2 피해 + `paralysis_turns >= 1`)를 추가한다. `submit_move`의 치사 함정 사망 원인 `DeathCause::Trap { trap }`은 실제 발동한 `TrapKind`를 기록한다. 함정 타일 위의 적대 몬스터도 인접 bump attack 대상이 된다.
- 생성 던전(`campaign_map`) v2: 40×20 4방+L자 통로 결정론 구조를 유지하되 모든 층에 1칸 폭 출입구 문(`x=7, y=centers[0].y` 닫힌 문, `x=17, y=centers[1].y` 홀수 층 닫힌 문 / 짝수 층 `HiddenDoor`)과 방 내부 회피 가능 숨겨진 함정(`centers[1]-(1,0)` Pit/Dart, `Main 3/5/6, Mines 2`의 `centers[2]-(1,0)` Bear)을 배치한다.
- V2 저장 지도 검증(`validate_persisted_map`): seed/LevelId로 생성한 기준 맵 대비 허용된 런타임 상태 전이(`Door(Closed)<->Door(Open)`, `HiddenDoor->Door(*)`, `HiddenTrap(k)->Trap(k)`)만 통과시키고 지형·계단 위조나 함정/문 임의 삭제·생성을 거부한다.

### P4b 4번째 역할·신규 아이템 5종·신규 몬스터 3종

- `Role::Cleric`: `HP 24`, `hit +3`, `damage +1`, 기본 5종 + `Mace`·`PotionExtraHealing`·`ScrollIdentify` (초기 무게 97/120). Creation `[4]` 키와 mouse CTA 및 headless `--role cleric`을 지원한다.
- 신규 아이템 5종(`crates/aihack-content/src/data/items.toml`):
  - `item.weapon.short_sword` (`ShortSword`, `)`, weight 15, hit +1, `1d6`, price 10)
  - `item.weapon.mace` (`Mace`, `)`, weight 20, hit +1, `1d6`, price 8)
  - `item.armor.chain_mail` (`ArmorChainMail`, `[`, weight 35, `ac_bonus=3`, price 75)
  - `item.potion.extra_healing` (`PotionExtraHealing`, `!`, weight 20, `heal_2d8_plus_8`, price 100)
  - `item.scroll.charging` (`ScrollCharging`, `?`, weight 5, `charging`, price 60) — 인벤토리 첫 미충전(또는 첫) 완드의 `charges`를 `max_charges`까지 회복(`ConsumableEffect::RechargeWand`)
- 신규 몬스터 3종(`crates/aihack-content/src/data/monsters.toml`):
  - `monster.kobold` (`Kobold`, `k`, HP 5, AC 1, hit +1, `1d4`, `chase_on_sight`, speed 12, difficulty 2)
  - `monster.orc` (`Orc`, `o`, HP 10, AC 2, hit +2, `1d6`, `chase_on_sight`, speed 12, difficulty 3)
  - `monster.ogre` (`Ogre`, `O`, HP 16, AC 3, hit +3, `1d8`, `chase_on_sight`, speed 10, difficulty 5)
- 캠페인 배치(`bootstrap::campaign_world`): `Main 1..2` Jackal, `Main 3` Goblin, `Main 4` Orc + FloatingEye, `Main 5` Orc, `Main 6` Ogre(아뮬렛 수호), `Mines 1` Kobold + FloatingEye, `Mines 2` Orc + Kobold, 그리고 층별 보너스 장비·물약·충전 두루마리를 배치한다.

### P4c TUI/검증 계약

- TUI 맵은 닫힌 문 `+`·열린 문 `-`을 구분 렌더링하고, hover 패널은 `closed door`/`open door` 및 `pit trap`/`dart trap`/`bear trap`, 신규 아이템/몬스터 라벨을 표시한다.
- 수정된 `items.toml`/`monsters.toml`의 LF SHA-256을 `docs/provenance/r7-content.sha256`과 `PROVENANCE.md`에 기록하고, `aihack-content` registry golden hash를 동기화한다.

## P5 캠페인 확장 (2026-11-26)

P1–P4 캠페인 위에 NetHack식 BUC(Blessed/Uncursed/Cursed)·인챈트, 잠긴 문(`Locked`), 독 상태이상, 원거리/재생 몬스터, 저주 해제/치료/화염 지팡이를 확장한다. 기존 V1 저장/해시, `Entity::item()` 5-튜플 시그니처, legacy 2층 모드, 기존 16종 아이템·6종 몬스터 및 `NH367-C001..C010`/`P8-G01..G20` 호환성은 그대로 보존한다.

### P5a BUC·인챈트·잠긴 문

- 아이템 BUC(`Blessed`/`Uncursed`/`Cursed`, 기본 `Uncursed`)와 인챈트 수치(`-3..=+5`, 기본 `0`)를 `EntityPayload::Item`의 optional serde 필드로 저장한다(기본값은 직렬화 생략).
- 장착 무기 인챈트는 근접 명중 보너스(`+enchant + if Blessed { 1 }`)와 피해 보너스(`+enchant`)에 반영한다. 장착 방어구 인챈트는 플레이어 AC(`adventurer_template().ac - (ac_bonus + enchant)`)에 반영하며 V2/V1 저장 검증기도 동일 공식으로 검증한다.
- `Cursed` 장착 무기·방어구는 `ScrollRemoveCurse` 또는 인챈트 두루마리로 저주를 풀기 전까지 교체(`Wield`/`Wear`)·버리기(`Drop`)·던지기(`Throw`)를 거부한다.
- 신규 두루마리 3종:
  - `item.scroll.enchant_weapon` (`ScrollEnchantWeapon`, `enchant_weapon`, price 60): 장착 무기를 `+1`(최대 `+5`) 강화하고 `Cursed`이면 `Uncursed`로 전환한다.
  - `item.scroll.enchant_armor` (`ScrollEnchantArmor`, `enchant_armor`, price 80): 장착 방어구를 `+1`(최대 `+5`) 강화·해저주하고 AC를 즉시 재계산한다.
  - `item.scroll.remove_curse` (`ScrollRemoveCurse`, `remove_curse`, price 80): 인벤토리 내 모든 `Cursed` 아이템을 `Uncursed`로 전환한다.
- 기존 포션/두루마리 BUC 효과: 회복 물약(`Heal`)은 `Blessed +4` / `Cursed -4`(최소 1), `ScrollCharging`은 `Blessed` 시 완드 해저주+충전 / `Cursed` 시 완드 충전량 0 소진.
- `DoorState::Locked`: `Open`/`Move` 및 투사체는 통과할 수 없으며 `Kick`으로만 부숴 `Door(Open)`으로 열 수 있다. `campaign_map`은 `Mines 2`의 `(7, centers[0].y)`에 `Door(Locked)`를 생성하고, `validate_persisted_map`은 `Locked`로 생성된 위치에서만 `Locked -> Open <-> Closed` 전이를 허용한다(일반 `Closed` 문 위치를 `Locked`로 변조 시 거부).

### P5b 독 상태이상·특수 몬스터 3종·신규 물약/완드

- 독 상태이상(`poison_turns`, 기본 `0`, 직렬화 생략)과 `DeathCause::Poison`: 매 턴 종료 시 `poison_turns > 0`이면 HP 1 피해, 카운터 1 감소, 경고 메시지 출력 및 0 HP 도달 시 `DeathCause::Poison`으로 게임 오버 처리한다.
- 신규 아이템 2종:
  - `item.potion.cure` (`PotionCure`, `!`, weight 20, `cure_status`, price 90): `poison_turns`/`paralysis_turns`/`hallucinating`을 즉시 해제하고 `1d6 + 4`(`Blessed +4`, `Cursed`는 1 고정) HP를 회복한다.
  - `item.wand.fire` (`WandFire`, `/`, weight 7, `wand_effect="fire"`, `max_charges=4`, price 150): 방향 투사체로 `2d6 + 2` 화염 피해를 입힌다.
- 신규 몬스터 3종 및 AI/패시브:
  - `monster.giant_spider` (`GiantSpider`, `s`, HP 8, AC 2, hit +2, `1d4`, `chase_on_sight`, passive `poison_on_melee`, speed 12, diff 3): 근접 적중 시 `poison_turns >= 3` 부여.
  - `monster.gnoll_archer` (`GnollArcher`, `G`, HP 11, AC 2, hit +2, `1d6`, `ranged_on_sight`, speed 12, diff 4): 인접 시 근접 공격, 직교 직선 거리 5 이내 장애물(`Wall`/`Door(Closed)`/`Door(Locked)`/`HiddenDoor`/중간 몬스터) 없으면 원거리 사격, 시야 내면 추적.
  - `monster.troll` (`Troll`, `T`, HP 22, AC 3, hit +3, `2d6`, `chase_on_sight`, passive `regenerate_hp`, speed 10, diff 6): 살아 있는 동안 턴당 HP 1 재생(최대 `max_hp`).
- 캠페인 배치: `Mines 1`에 `GiantSpider` 및 `Cursed -1 Dagger`·`PotionCure`, `Main 4`에 `GnollArcher`·`PotionCure`, `Main 5`에 `Troll`·`WandFire`, `Main 1/2/3`에 `ScrollEnchantWeapon`/`ScrollRemoveCurse`/`ScrollEnchantArmor`와 `Blessed +1 ShortSword`, `Mines 2`에 `+1 Mace`를 배치한다.

### P5c TUI·Headless·검증

- TUI는 `DoorState::Locked`를 `+` / `"locked door"`, `POIS:<n>` HUD, 비기본 BUC/인챈트 라벨(`+1 blessed ...`, `-1 cursed ...`), 신규 몬스터/아이템 글리프 및 `DeathCause::Poison`("Killed by poison")을 표시한다.
- Headless `survival-v1`은 중독 시 `PotionCure`를 우선 음용하고 `Door(Locked)`를 발로 차 여는 후보를 지원한다.

## P6 캠페인 확장 (2026-11-26)

P1–P5 캠페인 위에 반지 슬롯·화염/독 저항·화염 개미(`FireAnt`)와 상점·골드 경제를 확장한다. 기존 legacy V1 저장/해시, `Entity::item()` 5-튜플, `observation_v1.json` 기본 직렬화 형태 및 `NH367-C001..C010`/`P8-G01..G20` 호환성은 그대로 보존한다.

### P6a 반지 슬롯·화염/독 저항·신규 몬스터

- 반지 슬롯(`ItemClass::Ring`, `EquipmentSlot::Ring`, `Inventory::equipped_ring` optional serde 생략) 및 `RingEffect::{PoisonResistance, FireResistance, Protection}`(`ItemData::ring_effect` optional serde 생략)을 추가한다.
- 신규 반지 3종(glyph `=`, weight `3`, slot `"ring"`):
  - `item.ring.poison_resistance` (`RingPoisonResistance`, `poison_resistance`, price `150`): `GiantSpider`의 `PoisonOnMelee` 및 매 턴 독 틱 피해를 막는다(`poison_turns`만 감소, HP 피해 0).
  - `item.ring.fire_resistance` (`RingFireResistance`, `fire_resistance`, price `150`): `FireAnt`의 `FireOnMelee` 화염 추가 피해를 무효화한다.
  - `item.ring.protection` (`RingProtection`, `protection`, `ac_bonus=1`, price `120`): 착용 방어구(`equipped_body`)와 동일한 통합 AC 공식(`adventurer_template().ac - (body_ac + body_enchant + ring_ac + ring_enchant)`)에 합산된다.
- 장착/해제 명령: `CommandIntent::PutOn { item }` / `CommandIntent::RemoveRing` (및 `InventoryAction::PutOn`). `Cursed` 반지는 `ScrollRemoveCurse` 해저주 전까지 `RemoveRing`/`PutOn` 교체/`Drop`/`Sell`을 거부한다.
- 신규 몬스터 `monster.fire_ant` (`FireAnt`, `a`, HP `12`, AC `3`, hit `+2`, `1d6`, `chase_on_sight`, passive `fire_on_melee` -> `MonsterPassive::FireOnMelee`, speed `12`, diff `5`): 근접 적중 시 비저항 플레이어에게 추가 `1d6` 화염 피해(`PassiveAttackTriggered` + 경고 메시지)를 입힌다.

### P6b 상점 타일·골드 경제

- `TileKind::Shop`을 추가해 통과·시야 투과를 허용하고, `campaign_map`이 `Main 2` `(centers[1].x, centers[1].y - 1)`와 `Mines 1` `(centers[1].x + 1, centers[1].y - 1)`에 배치한다. `validate_persisted_map`은 생성 위치 외 임의 `TileKind::Shop` 변조를 거부한다.
- 캠페인 전용 골드 스케일링: 캠페인은 초기 소지금 `50`골드 및 처치당 `difficulty * 25` 골드를 지급하고, legacy 2층 모드는 기존 `difficulty` 골드 규칙을 보존한다.
- 상점 카탈로그 9종(`FoodRation`, `PotionHealing`, `PotionCure`, `ScrollEnchantWeapon`, `ScrollEnchantArmor`, `ScrollRemoveCurse`, `RingProtection`, `RingPoisonResistance`, `RingFireResistance`) 구매(`CommandIntent::Buy { kind }`, 가격 `base_price.max(10)`)와 판매(`CommandIntent::Sell { item }`/`InventoryAction::Sell`, 가격 `(base_price / 2).max(1)`, `AmuletAscension` 및 `Cursed` 장착 아이템 판매 거부)를 구현한다.

### P6c TUI·Headless·캠페인 배치

- 캠페인 배치: `Mines 1`에 `RingPoisonResistance`, `Main 4`에 `RingFireResistance`, `Mines 2`에 `+1 RingProtection`, `Main 5`·`Main 6`에 `FireAnt`를 배치한다.
- TUI는 `TileKind::Shop`을 `$`/`"shop"`, `FireAnt`를 `a`/`"fire ant"`, 반지를 `=`/`"ring of ..."`, 캠페인 상태 줄 `G<gold>`, 저항 배지 `RES:F`/`RES:P`/`RES:FP`로 표시하며 `'P'`(착용)·`'T'`(반지 해제)·`'$'`(상점 메뉴)를 제공한다.
- Headless `survival-v1`은 반지 슬롯이 비어 있을 때 소지 반지를 우선 착용(`PutOn`)한다.

## P7 캠페인 확장 (2026-11-26)

P1–P6 캠페인 위에 제단·마나(MP)/주문, 시드별 미식별 외형·자동 식별 및 소모품/투사체 수량 스택, 심층 보스(`Minotaur`, `Dragon`) 및 아뮬렛 귀환 추격전을 확장한다. 기존 legacy V1 저장/해시, `Entity::item()` 5-튜플 시그니처, `observation_v1.json` 기본 직렬화 형태, `tests/long_run.rs` R9 causal baseline 및 `NH367-C001..C010`/`P8-G01..G20` 호환성은 그대로 보존한다.

### P7a 제단·마나(MP)/주문

- `TileKind::Altar`를 추가해 이동·시야 투과를 허용하며 `campaign_map`이 `Main 3` `(centers[1].x, centers[1].y - 1)`와 `Mines 2` `(centers[1].x + 1, centers[1].y - 1)`에 결정적으로 배치한다(`validate_persisted_map`으로 위조 제단 거부).
- 제단 상호작용:
  - `CommandIntent::Offer { item }` (`InventoryAction::Offer`): 제단 위에서 소지품(아뮬렛·저주 장착 아이템 제외) 1단위를 바친다. 시체·식량 공물은 `+1 luck`(최대 3), `prayer_cooldown -10`, `+4 MP`(최대 `max_mp`), 미저주 장착 근접 무기 축복(`Blessed`)을 부여하고, 기타 아이템 공물은 해당 `ItemKind` 식별·`+2 MP`·`prayer_cooldown -5`를 부여한다.
  - `TileKind::Altar` 위에서 `Pray`: 모든 저주(`Cursed`) 소지품 해저주, MP 전량 회복(`max_mp`), HP `+6` 회복(최대 `max_hp`)을 함께 수행한다.
- 역할별 마나(`WorldState::{mp, max_mp}`, legacy 0 생략)와 주문 `SpellKind::{ForceBolt, Heal, Detect}` (`CommandIntent::Cast { spell, direction }`, `GameEvent::SpellCast`):
  - `Mage`: 초기 MP 14 + 레벨당 +3, 주문 `[ForceBolt, Detect]`
  - `Cleric`: 초기 MP 10 + 레벨당 +2, 주문 `[Heal, Detect]`
  - `Scout`: 초기 MP 6 + 레벨당 +1, 주문 `[Detect]`
  - `Knight`: 초기 MP 4 + 레벨당 +1, 주문 `[Heal]`
  - 캠페인 턴 진행 시 5턴마다 `MP +1` 자연 회복(`max_mp` 상한).
  - `ForceBolt`(MP 4, 방향 지정): 사거리 5 이내 적에게 `2d6` (`hit +4`, 피해 보정 `level / 2`) 역장 공격.
  - `Heal`(MP 5, 자가 시전): HP `2d6 + 4 + level` 회복(`Cleric`은 `poison_turns = 0` 동시 해제).
  - `Detect`(MP 3, 자가 시전): 현재 층 숨겨진 문/함정 공개 및 생존 적대 몬스터 수 통지.

### P7b 미식별 외형 학습·아이템 스택

- `appearance_for_kind(seed, kind)`: 캠페인에서 combat RNG와 독립된 SplitMix64 셔플로 물약(6색상 `"<color> potion"`), 두루마리(10라벨 `"scroll labeled <LABEL>"`), 반지(6재질 `"<material> ring"`)의 미식별 외형을 부여한다.
- 캠페인 자동 식별: `Quaff`, `Read`, `PutOn` 사용 또는 제단 비식량 공물·`ScrollIdentify`(미식별 아이템 우선) 시 해당 `ItemKind`가 즉시 식별되며, `ItemObservation`/`EntityObservation`의 `appearance`가 `None`으로 전환된다. Legacy 모드는 기존 `identified_items` 규칙을 유지한다.
- 아이템 수량 스택(`EntityPayload::Item::quantity: u16`, 기본 `1` serde 생략, `MAX_ITEM_STACK = 99`): 캠페인에서 물약·두루마리·식량·`Rock`은 동일 `(kind, buc, enchantment)` 스택과 병합(`Pickup`, `Buy`)되며, `Quaff`/`Read`/`Eat`/`Sell`/`Offer`/`Throw` 시 1단위씩 소비·투척된다(`Throw`는 1단위 개체 분리). `carried_weight()`와 점수 계산은 수량을 반영한다.

### P7c 심층 보스·아뮬렛 귀환 추격·TUI/Headless

- 신규 몬스터 2종:
  - `monster.minotaur` (`Minotaur`, `H`, HP 26, AC 4, hit `+4`, `2d6`, `chase_on_sight`, speed 12, diff 7): `Mines 2` 배치.
  - `monster.dragon` (`Dragon`, `D`, HP 30, AC 5, hit `+4`, `2d6`, `ranged_on_sight`, `passive="fire_breath"` -> `MonsterPassive::FireBreath`, speed 10, diff 8): `Main 6` 배치, 직교 시선 거리 5 이내에서 `2d6` 화염 브레스(화염 저항 시 무효).
- 귀환 추격 (`ascension_pursuit_stage: u8`, 기본 `0` serde 생략): 아뮬렛을 소지한 채 상승할 때 `Main 4`(1단계 `FireAnt`), `Main 2`(2단계 `Troll`), `Main 1`(3단계 `Minotaur`)에 각 1회 추격 몬스터가 생성되며 `Main 1..=5`에서 15턴마다 포만도 -1 부하를 받는다.
- TUI는 `Altar`(`_`/`"altar"`), `Minotaur`(`H`/`"minotaur"`), `Dragon`(`D`/`"dragon"`), `mp <cur>/<max>`, `<qty>x <label>` 스택 접두사, 미식별 외형 라벨, `'Z'`(주문 메뉴)·`'O'`(공물 메뉴)를 제공한다. Headless `survival-v1`은 부상 시 `Heal`, 시선/인접 적에게 `ForceBolt`를 활용한다.

## Stage 1 (P7-Fix) 결함 일괄 시정 (2026-11-27)

`docs/CODER_HANDOFF_P7_TO_P13.md` 감사에서 식별된 `DEF-01` ~ `DEF-11` 결함을 시정하며, 기준 결정은 `ADR-P7-02`에 기록한다.

- **DEF-01 (하강 AC 전면 통일)**: `adventurer_template().ac = 10`, `combat::attack_roll_value` 방어력 공식 `defense = 20 - defender.ac`, `monsters.toml` 12종 몬스터 AC(`10 - old_ac`), `PlayerObservation.ac`, TUI 상태 줄 `AC:<val>` 및 생성 화면 `AC: 10`으로 통일한다(`PROVENANCE.md`·`r7-content.sha256`·`registry.rs` 동기화 포함).
- **DEF-02 (인벤토리 52슬롯 `a-zA-Z`·빈 문자 재할당)**: `Inventory`가 `a..=z`, `A..=Z` 총 52슬롯에서 현재 미사용인 가장 앞 문자를 재할당하고 `Pickup`/`Buy`의 기존 스택 병합은 슬롯 포화 시에도 허용한다(`save.rs` `is_ascii_alphabetic()`·52슬롯 검증 포함).
- **DEF-03 (`ScrollLevelTeleport` 아뮬렛 우회 차단·분기 목적지)**: 캠페인에서 `has_amulet`이면 층 이동을 차단하고(두루마리 소비, 경고 메시지) 비소지 시 현재 분기 내 다른 유효 층으로 결정론적으로 이동한다(legacy 2층 `Main 1 <-> Main 2` 유지).
- **DEF-04 (허기 런타임 효과)**: `DeathCause::Starvation`과 TUI `"Died of starvation"`, `Weak`(`1..=50`) 명중 `-2`·경고, `Fainting`(`<= 0`) 주기적 마비, `nutrition <= -150` 아사 처리를 연결하고, Headless `survival-v1`이 `nutrition <= 150`일 때 식량(`FoodRation`, `CorpseJackal`)을 섭취하도록 보강한다.
- **DEF-05 (캠페인 자연 HP 회복)**: 캠페인 생존 중 `poison_turns == 0`이고 `nutrition > 50`일 때 `(15 - level).max(5)`턴마다 HP `+1`(`max_hp` 상한)을 회복한다(legacy 비적용).
- **DEF-06 (미식별 BUC/인챈트·상점 누출 차단)**: `EntityPayload::Item`/`ItemObservation`의 `buc_known`·`enchant_known`(optional serde 생략)으로 미감정 아이템의 BUC/인챈트를 `Observation`과 TUI에서 은닉하고, `CampaignObservation.identified_items` 및 상점 구매가 헬퍼로 TUI `'$'` 상점 메뉴의 미식별 품목이 시드 외형·구매 가격을 표시하도록 한다.
- **DEF-07 (탐험 기억 / Fog of War)**: `GameLevel.explored`(`serde(default, skip_serializing_if = "Vec::is_empty")`)에 탐험 타일을 영속 저장하고 `Observation.visible_tiles`에 기억 타일(`visible: false`)을 포함해 `render_map.rs`가 `theme.muted`로 어둡게 그리며, `LlmObservationView`는 `tile.visible`만 투영해 32 KiB 요청 한도를 보장한다.
- **DEF-08 (`aihack-content` 레벨 로더 보완)**: `item_id`/`item_kind_from_id` 단일 매핑으로 24종 아이템을 공유해 P5/P6 8종 누락을 제거하고 `level_tile_overrides()`에 `"locked"` 문 및 `"dart"`/`"bear"` 함정을 추가한다.
- **DEF-09 (Dead state 정리)**: `character_creation_lines()`의 정적 `Strength: 10`/`Dexterity: 10`·`AC: 0`을 실제 스탯·역할·`AC: 10`으로 정리하고 `log_lines()`에 `hallucination_message` 출력을 연결한다(`hallucinating` 유발원과 `damage_reduction` 확장은 `P8/P9` 보류 명시).
- **DEF-10 (두루마리 BUC 분기·대상 지정 감정)**: `EnchantWeapon`/`EnchantArmor`(`Cursed` `-1`·저주, `Uncursed` `+1`·해저주, `Blessed` `+1..=+2` 안전 상한 `+5`), `RemoveCurse`(`Blessed` 전체·`Uncursed` 장착만·`Cursed` 무효), `CommandIntent::Identify { scroll, target }` 및 TUI 식별된 감정 두루마리 대상 메뉴를 구현한다.
- **DEF-11 (마감 검증·문서 동기화)**: 전체 게이트 통과 및 `tasks/todo.md`, `README.md`, `docs/campaign_implementation.md`, `IMPLEMENTATION_SUMMARY.md`, `CHANGELOG.md` 동기화 후 미커밋 상태로 마감한다.

## Stage 2 (P8) 엔진 기반 업그레이드 (2026-11-27)

기준 결정: `DESIGN_DECISIONS.md` `ADR-P8-01`. 레거시 2층 모드(`campaign.is_none()`)의 V1 저장(`schema_version = 1`), `observation_v1.json` 직렬화 및 `tests/long_run.rs` R9 baseline 해시는 그대로 보존한다.

### P8-A 데이터 주도 콘텐츠 레지스트리
- `items.toml` / `monsters.toml`: 기존 24아이템·12몬스터의 레거시 필드를 보존한 채 `name`, `color`, `unidentified_pool`, `material`, `stackable`, `damage_small`/`damage_large`, `effects`, `tags`, `spawn_depth_range`, `spawn_weight`, `base_level`, `hp_dice`, `attacks`, `resistances`, `corpse_chance`, `corpse_nutrition`, `corpse_effects`, `ai_flags`와 P9~P13 훅(`tool_action`, `container_capacity`, `artifact_id`)을 추가한다.
- `monster.speed` 검증 범위는 `0..=36`으로 확장하고 `monster.bat`(`B`, `speed = 24`) 및 방어구 5슬롯·`AmuletEsp`·상태 물약 6종·일반 시체(`Corpse`)를 추가한다.
- `Observation`의 캠페인 전용 optional 필드와 `ContentRegistry` 메타데이터 조회로 TUI(`render_map`, `render_panels`, `input`, `play_menu`)를 데이터 주도 방식으로 전환한다.

### P8-B 확장 RPG 코어
- **10 장비 슬롯**: `Melee`, `Shield`, `Helm`, `Body`, `Cloak`, `Gloves`, `Boots`, `LeftRing`(기존 `Ring` 직렬화 alias 포함), `RightRing`, `Amulet`. 모든 방어/장신구 슬롯이 통합 하강 AC(`10 - sum(ac_bonus + enchant)`) 및 외재 저항에 합산되며 `CommandIntent::TakeOff { slot }`을 추가한다.
- **에너지 버킷 턴 스케줄러**: `ActorStats.energy` 누적기(`ENERGY_THRESHOLD = 12`, 턴당 최대 4회 행동)로 `speed`에 비례한 다중/분수 턴 행동과 `Fast` 감속을 처리한다.
- **6대 능력치 (`Str`, `Dex`, `Con`, `Int`, `Wis`, `Cha`, `3..=25`)**: 역할별 초기 능력치(`Knight 16/11/15/9/10/12`, `Scout 12/16/12/11/10/12`, `Mage 9/12/10/17/14/11`, `Cleric 12/10/14/11/16/14`)를 부여하고 무게 한도(`60 + Str*5`), 근접 명중/피해, 함정 회피, 레벨당 HP/재생 속도, 주문 위력/치유/기도 쿨다운, 상점 거래 가격에 반영한다.
- **저항 비트셋 & 상태이상**: `Resistances`(Fire, Cold, Shock, Poison, Sleep, Disintegration, Telepathy, SeeInvisible) 내재+외재 합성과 `StatusTimers`(Blind, Confused, Stunned, Hallucinating, Fast, Levitation, Invisible) 턴 감산/효과를 구현한다.
- **일반화된 몬스터 시체**: 캠페인에서 `corpse_chance` 기반 임의 몬스터 시체(`corpse_monster`, `corpse_age`)를 드롭하며, 50턴 초과 부패 식중독, `corpse_effects` 저항 획득, `Troll` 시체 `corpse_age >= 25` 부활을 처리한다.

### P8-C Generator v3 (`80×21`), 분기 그래프, SaveV3 & P9~P13 스키마 훅
- **Generator v3**: `80×21` 크기 가변 방·복도·루프·어둠/조명(`lit_tiles`)·특수방(`Shop`, `Temple`, `Vault`, `Barracks`)·BFS 연결 보장 던전 생성기.
- **BranchGraph**: `Main(1..=6)`·`Mines(1..=2)`(및 예약 `Sokoban`, `Quest`, `Gehennom`) 연결 그래프 테이블로 계단/분기 진입을 구동한다.
- **SaveV3 & 구조적 맵 검증**: 캠페인 저장은 `SAVE_SCHEMA_VERSION_V3 = 3`을 사용하며 `validate_persisted_map`이 `80×21` 경계, 외곽 벽, 계단 수/위치, `Shop`/`Altar` 비변조, BFS 연결성, `lit_tiles`/`engravings` 범위를 검증한다(내부 `Wall -> Floor` 굴착 전이 허용).
- **P9~P13 스키마 훅**: 아이템 컨테이너(`container_capacity`, `contained_items`)·도구/아티팩트(`tool_action`, `artifact_id`), 레벨(`engravings`, `special_room`, `special_level_id`), 캠페인/월드(`alignment`, `race`, `god_anger`, `prayer_timeout`, `weapon_skills`, `spell_skill`, `learned_spells`, `generated_artifacts`, `ascension_altars`), 액터(`Faction::{Pet, Peaceful, Shopkeeper}`, `monster_inventory`)를 optional serde 생략 필드로 선반영한다.

## Stage 3 (P9) 도구 · 컨테이너 · #dip / #engrave · 지형 생태계 (2026-11-27)

기준 결정: `DESIGN_DECISIONS.md` `ADR-P9-01`. 레거시 2층 모드(`campaign.is_none()`)의 V1 저장(`schema_version = 1`), `observation_v1.json`, `Entity::item()` 5-튜플 및 `tests/long_run.rs` R9 baseline 해시는 그대로 보존하며, 모든 P9 신규 저장/관찰 필드는 optional serde 생략으로 유지한다.

### P9-1 도구(`ItemClass::Tool`) 및 `CommandIntent::Apply { item, direction }`
- 신규 도구 8종(glyph `(`):
  - `SkeletonKey` (`tool_action="unlock"`): 인접 `Door(Locked)` 해제 또는 `Door(Closed)` 잠금; `direction=None` 시 현재 위치/소지 `Chest` 잠금 토글.
  - `LockPick` (`tool_action="unlock"`): `Door(Locked)` 해제 및 `Chest` `container_locked = false`·`container_trapped = false`(비저주 시).
  - `PickAxe` / `DwarvishMattock` (`tool_action="dig"`): 내부(`1..width-1`, `1..height-1`) `TileKind::Wall`만 `TileKind::Floor`로 굴착(`Cursed` `PickAxe`는 50% 미끄러짐, 외곽 테두리 거부).
  - `TinWhistle` (`tool_action="music"`): 현재 층 `Faction::Pet` 동료를 플레이어 쪽으로 1칸 호출.
  - `OilLamp` (`tool_action="light"`, `charges=150`): `tool_active` 점등/소등 토글; 어두운 방에서도 점등+연료 잔여 시 `DEFAULT_VISION_RADIUS (6)` 시야 제공, 매 턴 연료 1 소모 후 0 소진 시 자동 소등.
  - `Blindfold` (`tool_action="blindfold"`): `tool_active` 착용/해제 토글(착용 시 시야를 현재 좌표 1칸으로 축소하며 `Telepathy`와 결합 가능; `Cursed` 착용 상태는 해제 차단 및 `buc_known = true`).
  - `UnicornHorn` (`tool_action="magic"`): 비저주 시 `blind`/`confused`/`stunned`/`hallucinating`/`poison_turns`/`paralysis_turns` 해제 및 역할 기준 미만의 능력치 손실 복구; `Cursed` 적용 시 `confused`/`stunned` 부여.

### P9-2 컨테이너(`Sack`, `BagOfHolding`, `Chest`) 및 `CommandIntent::{ContainerPut, ContainerTake}`
- `Sack`(cap 200), `BagOfHolding`(cap 400), `Chest`(cap 500, `Main 2`/`Mines 2` 등에 잠김/함정 상태로 생성 가능)를 추가한다.
- `EntityLocation::InContainer { container }` ↔ `contained_items` 양방향 참조·acyclicity(깊이 `<= 4`)·수용량(`<= container_capacity`)을 런타임 및 `save.rs`에서 강제한다.
- `BagOfHolding` 내용물 무게는 `Blessed` `1/4`, `Uncursed` `1/2`, `Cursed` `2x`로 계산되며 점수 계산(`inventory_value`)은 내부 아이템 가치를 재귀 포함한다.
- `AmuletAscension` 수납은 거부하며, `BagOfHolding` 내부에 또 다른 `BagOfHolding`을 넣으면 `2d8` 폭발(`DeathCause::Explosion`)과 함께 양쪽 컨테이너 및 내용물을 소멸시킨다.
- 잠긴 컨테이너에서 꺼내기는 거부되고, 함정(`container_trapped`)이 있는 컨테이너에서 첫 꺼내기 시 바늘 함정(`2d4` + 독)이 발동한 뒤 해제된다.

### P9-3 `#dip` (`CommandIntent::Dip`) 및 성수·녹(`erosion`)·연금술
- `PotionWater`(물약 `!`, `effect="water"`, stackable)와 `ScrollBlank`(두루마리 `?`, `effect="blank"`, stackable)를 추가한다.
- 제단 위에서 `Uncursed PotionWater` 담그기는 `3 MP`를 소비해 `Blessed PotionWater`(성수)로 축복하고(제단 `Pray` 역시 소지 `Uncursed PotionWater`를 축복), `Blessed`/`Cursed` 물에 다른 아이템을 담그면 1병을 소비해 BUC를 1단계 정화/저주하고 `buc_known = true`를 설정한다.
- `Fountain`/`Sink`/`Water` 또는 `Uncursed` 물에 담그면 일반 물약은 `PotionWater`로 희석, 일반 두루마리는 `ScrollBlank`로 세척, 점등된 `OilLamp`는 소등, 철/강 재질(`iron`/`steel`) 장비는 `erosion`(+1, 최대 `3`)이 증가한다(무기 피해 `-erosion`, 방어/장신구 AC 기여 `-erosion`).
- 두 비-물 물약 혼합(`DipTarget::Item`)은 연금술 조합 결과 물약 또는 `PotionWater`를 생성한다.

### P9-4 `#engrave` (`CommandIntent::Engrave`) 및 `Elbereth`
- `EngravingMethod::{Fingers, Item(EntityId)}`와 `EngravingText::{Elbereth, Warning, Safe, XMark}`로 `Dust`(손가락)·`Engraved`(무기/굴착 도구)·`Burned`(`WandFire` 1충전 소비) 바닥 각인을 남긴다(`Water`/`Lava` 위 거부, 혼란/기절 시 문구 훼손).
- 플레이어가 유효 `"Elbereth"` 각인 위에서 공격하면 `Dust`는 즉시 지워지고 `Engraved`/`Burned`는 1글자 마모되며, 매 턴 종료 시 `Dust` 각인은 확률적으로 마모된다.
- 유효 `"Elbereth"` 위에 선 플레이어를 상대로 `Minotaur`를 제외한 적대 몬스터 AI는 공격·추적 대신 1칸 도주(또는 대기)한다.

### P9-5 지형(`Fountain`, `DryFountain`, `Sink`, `Water`, `Lava`), `#drink`, 12종 함정
- `Fountain`(`{`), `DryFountain`(`{`), `Sink`(`;`), `Water`(`~`), `Lava`(`~`) 5종 지형을 추가하고 `CommandIntent::Drink`로 `Fountain`/`Sink` 음용 효과를 처리한다(`Fountain` 고갈 시 `DryFountain` 전이 및 `Bat` 출현 가능).
- `Water` 진입 시 `Levitation`은 무사 통과, 여유 무게(`carried_weight * 2 <= carrying_capacity`) 시 수영(철제 장비 1개 `erosion +1` 및 점등 램프 소등), 초과 무게 시 `2d6` 익사 피해(`DeathCause::Drowning`)를 입는다. `Lava`는 `Levitation` 무사 통과, 화염 저항 시 `1d6`, 비저항 시 `3d10` 화염 피해(`DeathCause::Lava`)를 입는다.
- 기존 3종(`Pit`, `Dart`, `Bear`)에 `Teleport`, `SleepingGas`, `Rust`, `Polymorph`, `Hole`, `AntiMagic`, `FallingRock`, `Magic`, `Fire` 9종을 더해 총 12종 함정을 `campaign_map` Room 4/5 비-경로 좌표 및 `level_tile_overrides()`에 연결한다.
- TUI 키(`'a'` Apply, `'C'` Container, `'v'` Dip, `'E'` Engrave, `'g'` Drink), Headless `survival-v1` 캠페인 전용 `UnicornHorn` 치료, `SAVE_SCHEMA_VERSION_V3 = 3` 양방향/범위 저장 검증을 적용한다.
