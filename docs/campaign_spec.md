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
