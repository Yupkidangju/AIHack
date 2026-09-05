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
