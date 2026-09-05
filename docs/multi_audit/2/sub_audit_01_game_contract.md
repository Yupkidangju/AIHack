# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 2
- Audit Date: 2026-09-05 (Asia/Seoul)
- Repository: `C:\LocalDev\rust\AIHack`
- Audited HEAD: `899660167d59c4b06d27a59c0d75fcccda0cce33`
- Perspective: A01 — 목표·명세·게임 완결성
- User Goal: 게임 루프와 시작→진행→끝이 NetHack 3.6.7에 준하는지, 그리고 모든 기능을 키보드와 마우스로 정상 플레이할 수 있는지 감사
- Audit Basis: Standard-backed / Goal-driven
- Standard Path: `AI_AUDIT_DOC_STANDARD.md`; `C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md`
- Scope Interpretation: 현재 v0.3.0의 의도적 호환성 slice와 사용자가 요청한 NetHack 3.6.7에 준하는 전체 게임 계약을 분리해 판정했다. 아래의 `Major` finding은 전체 목표 대비 계약 공백이며, v0.3.0 slice에서 의도적으로 줄인 부분은 별도의 Notes로 구분했다.

## 2. Assigned Scope

- `spec.md`, `designs.md`, `IMPLEMENTATION_SUMMARY.md`, `README.md`의 시작·진행·종료·입력 계약
- `crates/aihack-content`의 content registry와 embedded item/monster/level 데이터
- `crates/aihack-runtime`의 bootstrap, `GameSession`, `GameWorld`, transaction, observation/action space, stairs, death, score, item/status 시스템
- `crates/aihack-core`의 `RunState`, `CommandIntent`, player/world/event/save 타입
- `apps/aihack-tui`의 화면 흐름, 키보드 dispatcher, 마우스 hit-test, overlay/modal, command CTA
- 기존 호환성·게임 루프·UI 관련 테스트와 테스트 문서
- NetHack 3.6.7 공식 Guidebook과 `NetHack-3.6.7_Released` tag의 역할/던전/종료 자료

## 3. Excluded and Uninspected Scope

- 과거 감사 보고서(`docs/audit/**`, `docs/multi_audit/1/**`)와 다른 에이전트 보고서는 읽지 않았다.
- `legacy_nethack_port_reference/**` 원문 corpus, release/HOLD 문서의 재감사, 게시·CI·전수 dependency·보안 감사는 제외했다.
- 전체 workspace/full suite와 실제 Windows Terminal GUI는 실행하지 않았다. 이 관점에서 필요한 기존 표적 테스트만 실행했다.
- 소스, 테스트, 설정, 제품 문서는 수정하지 않았다. 새 probe/test 파일도 만들지 않았다. 이 보고서 파일만 생성했다.
- NetHack 3.6.7의 모든 몬스터·아이템·특수 레벨, 주문·상점·펫·저주·유물 등의 상세 규칙은 개별 적합성까지 판정하지 않고 게임 완결성의 범위 증거로만 확인했다.

## 4. Evidence Examined

### 4.1 Project documents and records

- `spec.md`: §3 목표/성공 기준, §4 v0.3.0 비목표, §8 상태 전이/turn pipeline, §9 경계 타입, §10 공식, §11 실데이터, §13 NH367-C001..C010, §18 완료 조건, §19 R9 인과 목표
- `designs.md`: §2~§6 화면 흐름/CTA/입력, §10 상태·오류, §11 접근성, §12 구현 제약, §13 검증
- `IMPLEMENTATION_SUMMARY.md`: 전체 runtime 흐름, R3 registry, R6 TUI/PTY 계약, R7 compatibility records, R9 causal scope
- `docs/compatibility/README.md`, `docs/compatibility/NH367-C001..C010-*.md`; 특히 `NH367-C010-game-over.md`

### 4.2 Local source and symbols

- Start: `crates/aihack-core/src/meta.rs:3-7 (GameMeta)`, `crates/aihack-core/src/domain/player.rs:1-20 (PlayerTemplate/adventurer_template)`, `crates/aihack-core/src/domain/entity.rs:314-335 (EntityStore::spawn_player)`, `crates/aihack-runtime/src/bootstrap.rs:33-126 (initial_world)`, `crates/aihack-runtime/src/session.rs:175-200 (Title/CharacterCreation)`
- State/end: `crates/aihack-core/src/run_state.rs:3-16 (RunState)`, `crates/aihack-core/src/action.rs:28-68 (CommandIntent)`, `crates/aihack-core/src/event.rs:14-121 (GameEvent)`, `crates/aihack-runtime/src/systems/stairs.rs:5-71`, `crates/aihack-runtime/src/systems/death.rs:15-85`, `crates/aihack-runtime/src/session.rs:523-583,537-550`
- Registry/world: `crates/aihack-content/src/schema.rs:99-318`, `crates/aihack-content/src/data/levels/main_1.toml`, `crates/aihack-content/src/data/levels/main_2.toml`, `crates/aihack-runtime/src/observation.rs:173-304`, `crates/aihack-runtime/src/world.rs:197-317`
- TUI/input: `apps/aihack-tui/src/tui/render_panels.rs:329-359`, `apps/aihack-tui/src/tui/input.rs:86-149,151-290,293-423`, `apps/aihack-tui/src/tui/mod.rs:769-961,984-1040,1540-1827`
- Search checks (source only): `rg -n -i "ascend|ascending|escape|escaped|win|victory|amulet|yendor" crates/aihack-core/src crates/aihack-runtime/src apps/aihack-headless/src apps/aihack-tui/src`; `rg -n -i "role|race|gender|alignment|experience|experience_level|xp|skill|enhance" crates/aihack-core/src crates/aihack-runtime/src apps/aihack-tui/src apps/aihack-headless/src`

### 4.3 Existing commands and results

- `cargo test --locked -p aihack --test ui_input_mapping --test ui_screens --test nethack_367_compat`: 25 passed, 0 failed.
- `cargo test --locked -p aihack-tui --test tui_contract`: 20 passed, 0 failed.
- These tests verify the bounded movement/door/combat/item/stairs/search/projectile/hunger/save/death slice and selected TUI state/geometry contracts. They do not establish full NetHack start choices, XP/skill growth, Amulet/escape/ascension, or complete keyboard↔mouse action parity.

### 4.4 NetHack 3.6.7 official basis

- [NetHack 3.6.7 Guidebook](https://www.nethack.org/v367/Guidebook.html), §2 (“What is going on here?”) states the goal as treasure, the Amulet of Yendor, and escaping alive; the same section describes roles and races. §3.1 describes alignment, dungeon depth, HP, power, armor class, experience level, hunger and encumbrance. §4 documents one/two-key command entry and prompts. §5.3 documents stairs, generated/visited levels and branches. §7 documents inventory/weight, weapons, proficiency, food/corpses and gold. §9.4 documents `role`, `race`, `gender`, `align`, `mouse_support` and travel/click behavior. §10 documents scoring differences for quit versus death.
- [Official NetHack release tag `NetHack-3.6.7_Released`](https://github.com/NetHack/NetHack/releases/tag/NetHack-3.6.7_Released), release commit `ed600d9` (3.6.7 only; no 3.7 source used).
- [3.6.7 `src/role.c`](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/role.c#L24-L50) contains the role table with allowed race/gender/alignment flags and role-specific starting/stat data.
- [3.6.7 `src/end.c`](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/end.c#L2220-L2250) tracks Amulet-class valuables; its [end-status table](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/end.c#L2681-L2715) distinguishes `quit`, `escaped`, and `ascended` from deaths.
- [3.6.7 `src/dungeon.c`](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/dungeon.c#L1411-L1436) contains the Gehennom/hell branch and level transition support.

### 4.5 Feature comparison

| 기능 계약 | NetHack 3.6.7 기준 | AIHack 실제 상태 | 판정 |
| --- | --- | --- | --- |
| 시작 선택 | 역할·종족·성별·정렬을 조합해 character를 시작하고 role별 통계/장비를 적용 | seed만 `GameMeta`에 있고 player는 고정 `Adventurer`; creation 화면도 고정 문자열 | A01-F001 |
| 탐험/턴 | 새 dungeon, 방/복도/시야/검색, 한 명령당 턴 진행 | deterministic map/LOS/search/turn transaction이 존재하고 C001/C002/C006이 통과 | 제한된 slice에서 Verified |
| 성장 | 경험치/experience level, role별 무기·주문 proficiency와 `#enhance` | XP/level/skill 필드·명령이 없고 player template가 고정 | A01-F003 |
| 자원·아이템 | 무게/허기/금/상점, 다양한 아이템·corpse·장비가 탐험과 종료 점수에 연결 | inventory, food/corpse nutrition, potion/wand/scroll/armor, kill gold/score 일부가 연결; 상점·유물·저주 등은 없음 | 부분 Verified; 전체는 scope gap |
| 던전 진행 | stairs/ladder로 깊어지고 branches·special levels·Gehennom으로 진행 | embedded registry가 `main:1`, `main:2` 두 level만 제공; main:2에는 down stair가 없음 | A01-F002 (v0.3.0 의도적 축소) |
| 목표/승리 | Amulet of Yendor 획득 후 신에게 바치고 살아서 escape/ascend | Amulet/Yendor/offer/success action·state/event 없음 | A01-F002 |
| 패배 | combat 외에도 starvation/choking/poison/drowning/burning/stone/slime 등 여러 end reason | combat/trap HP depletion만 typed death path; `GameOver`는 확인됨 | death slice Verified; 나머지는 v0.3.0 범위 밖 |
| 저장/계속 | 저장 후 상태·깊이·자원·RNG를 이어서 진행 | C009가 save/load 뒤 command/RNG continuation을 검증 | 제한된 slice에서 Verified |
| 키보드 | 모든 command가 key/extended command와 필요한 방향·물건 prompt를 통해 접근 | movement/basic items 일부만 직접 매핑; q 충돌, east 하드코딩, live prompt 진입 없음 | A01-F005 |
| 마우스 | 3.6.7 windowport의 mouse input/travel은 옵션으로 제공 | 지도 인접 이동/inspect/status/일부 footer/CTA만 매핑; modal·상태 전환·대부분 command는 마우스 no-op | A01-F005 |

## 5. Findings

### [A01-F001] CharacterCreation은 실제 역할·종족·성별·정렬 선택을 제공하지 않음

- Pass: Implementation Compliance
- Pattern: `IMP-001`, `SPEC-GAP-001`
- Area: 시작 계약 / character creation / player identity
- Severity: Major
- Status: Confirmed (전체 사용자 목표 대비 Needs Fix; v0.3.0 범위는 Needs Spec Clarification)
- Summary: Title에서 CharacterCreation으로 이동하지만, 생성 단계는 선택 상태를 갖지 않고 고정 Adventurer를 확인하는 화면일 뿐이다.
- Evidence:
  - `crates/aihack-core/src/meta.rs:3-7`의 `GameMeta`는 `seed`만 가진다. role/race/gender/sex/alignment 타입이나 저장 필드가 없다.
  - `crates/aihack-core/src/domain/player.rs:1-20`의 `PlayerTemplate`와 `adventurer_template()`는 HP/AC/hit/damage/단검 프로필 하나만 반환한다.
  - `crates/aihack-core/src/domain/entity.rs:314-335`의 `EntityStore::spawn_player`는 항상 `adventurer_template()`을 사용한다.
  - `crates/aihack-runtime/src/bootstrap.rs:44-77`의 `initial_world`는 선택값을 읽지 않고 고정 player와 초기 inventory를 생성한다.
  - `apps/aihack-tui/src/tui/render_panels.rs:344-359`는 `Class: Adventurer`, `HP: 16/16`, `Strength: 10`, `Dexterity: 10`, `AC: 0`을 상수 문자열로 렌더링한다. 실제 경로인 `apps/aihack-tui/src/tui/mod.rs:1324-1332`도 이 고정 renderer만 호출한다.
  - 범위 검색 `rg -n -i "role|race|gender|alignment|experience|experience_level|xp|skill|enhance" crates/aihack-core/src crates/aihack-runtime/src apps/aihack-tui/src apps/aihack-headless/src`에서 player 선택/성장 계약은 발견되지 않았다.
- Expected Basis: 사용자가 명시한 “역할/종족/성별/정렬” 비교 목표와 [Guidebook §2](https://www.nethack.org/v367/Guidebook.html), [Guidebook의 role/race/gender/align 옵션](https://www.nethack.org/v367/Guidebook.html#Options); 공식 [3.6.7 `role.c` role table](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/role.c#L24-L50)은 role별 허용 조합과 시작 통계를 가진다.
- Actual: 모든 production run은 선택 가능한 캐릭터 정체성 없이 동일한 Adventurer template와 초기 장비로 시작한다. `CharacterCreation`의 Enter는 선택을 확인하지 않고 `RunState::Playing`만 설정한다(`crates/aihack-runtime/src/session.rs:189-200`).
- Impact: 시작 선택이 전략과 상태에 영향을 주는 NetHack식 계약을 재현할 수 없으며, 같은 seed에서 다른 role/race run을 만들거나 save/replay로 identity를 보존할 수 없다.
- Suggested Action: 전체 목표를 유지하려면 구현 전에 `spec.md`에 role/race/gender/alignment의 허용 조합, 기본값·RNG, role별 stats/starting kit, save/replay 및 keyboard/mouse 선택 UI를 명시한 뒤 typed character-selection state를 추가한다. 현재 v0.3.0이 단일 Adventurer compatibility fixture라면 그 사실을 완료 주장과 화면 명칭에 명시해 CharacterCreation을 full NetHack start와 혼동하지 않게 한다.
- Re-audit Method: 각 role·race·gender·alignment 조합을 UI와 headless에서 생성하고, 허용/거부 조합, start stats/kit, snapshot/save/load/replay identity, 두 입력 방식의 동일 결과를 검증한다.
- Owner: Architect/Coder (제품 범위 결정은 Human)
- Confidence: High
- Notes: `spec.md:72-82`는 “모든 몬스터·아이템·특수 레벨”을 v0.3.0 비목표로 명시하지만 role/race/gender/alignment 자체를 비목표로 적지는 않는다. 따라서 이 finding은 확정된 코드 사실이면서, 현재 release가 compatibility slice인지 full game인지에 대한 명세 명확화가 필요하다.

### [A01-F002] Amulet 목표와 살아서 끝나는 escape/ascension 경로가 없음

- Pass: Implementation Compliance
- Pattern: `IMP-002`, `IMP-003`, `SPEC-GAP-001`
- Area: 던전 진행 / 목표 / 승리·종료 state machine
- Severity: Major
- Status: Confirmed (전체 사용자 목표 대비 Needs Fix; 현재 v0.3.0 slice에서는 Deferred/Needs Spec Clarification)
- Summary: 현재 구현은 두 고정 level을 왕복하고 죽거나 quit하는 루프만 닫혀 있다. 살아서 완료하는 목적지, Amulet 획득/제공, escape/ascension outcome이 구현되어 있지 않다.
- Evidence:
  - `crates/aihack-content/src/schema.rs:99-128`의 embedded registry는 `levels/main_1.toml`, `levels/main_2.toml`만 로드한다. 실제 data인 `main_1.toml`은 `stairs_down=[34,15]`, `main_2.toml`은 `stairs_up=[5,5]`만 가진다.
  - `crates/aihack-runtime/src/systems/stairs.rs:5-35`는 다음 depth의 `stairs_up` landing을 요구하고, `:38-71`은 `main:1` 위로 올라갈 수 없도록 한다. main:2에서 더 내려가는 target은 존재하지 않는다.
  - `crates/aihack-core/src/run_state.rs:3-16`에는 `Title`, `CharacterCreation`, `Playing`, 대기 상태와 `GameOver`만 있고 `Won`, `Escaped`, `Ascended`가 없다.
  - `crates/aihack-core/src/action.rs:28-68`의 `CommandIntent`와 `crates/aihack-core/src/event.rs:14-121`의 `GameEvent`에는 Amulet/offer/escape/ascension 계약이 없다.
  - `crates/aihack-runtime/src/systems/death.rs:75-85`의 player end projection은 생존이면 `Playing`, 사망이면 `GameOver`만 반환한다.
  - 검색 범위 `rg -n -i "ascend|ascending|escape|escaped|win|victory|amulet|yendor" crates/aihack-core/src crates/aihack-runtime/src apps/aihack-headless/src apps/aihack-tui/src`에서 `Ascend`(stairs 명령) 외 성공/Amulet/escape symbol은 확인되지 않았다.
  - `docs/compatibility/NH367-C010-game-over.md:46`는 “NetHack의 종료 화면 문구나 scoring 전체를 복제하지 않는다”고 명시한다. 이는 C010 death fixture의 의도적 범위이지 full game ending의 구현 증거가 아니다.
- Expected Basis: [Guidebook §2](https://www.nethack.org/v367/Guidebook.html)에서 treasure·Amulet of Yendor·살아서 escape를 게임 목표로 명시하고, [Guidebook §5.3](https://www.nethack.org/v367/Guidebook.html)에서 깊어지는 stairs와 Gnomish Mines branch를 설명한다. 공식 [3.6.7 `end.c`](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/end.c#L2220-L2250)에는 Amulet valuables가 있고, [3.6.7 `end.c` end-status table](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/end.c#L2681-L2715)에는 `escaped`와 `ascended`가 `quit`·죽음과 구분되어 있다. [3.6.7 `dungeon.c`](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/dungeon.c#L1411-L1436)는 Gehennom branch를 포함한다.
- Actual: `Descend`는 main:1→main:2 한 번만 유효하고 `Ascend`는 왕복용이다. Amulet을 생성·소지·offer할 수 없고, `RunState`/event/CLI/TUI에 성공 종료가 없다. `run_to_turn`의 1000-turn 성공은 `final_state=Playing`을 요구할 뿐(`apps/aihack-headless/src/lib.rs:191-238`, `tests/long_run.rs:39-120`), 게임 승리를 의미하지 않는다.
- Impact: 사용자는 목표를 달성해 run을 완료할 수 없고, “시작→탐험→성장→목표→끝”이라는 NetHack식 game loop가 성립하지 않는다. 장기 결정론 테스트가 통과해도 끝 조건의 부재를 탐지하지 못한다.
- Suggested Action: full target이면 먼저 spec/phase를 확정하고 Amulet item/위치, deeper main/branch/special levels, offer/escape/ascension legality, success state/event/score, end disclosure와 both-input UI를 설계한다. v0.3.0의 C001..C010 slice만 shipping하려면 README/spec/compatibility records에서 “death/stairs compatibility slice; full winning loop deferred”를 명시하고 full-NetHack 준수 표현을 제한한다.
- Re-audit Method: fixture 또는 실제 seed에서 Amulet 획득→필요한 offer/escape/ascend sequence를 keyboard와 mouse로 실행하고, `RunState`/event/score/save/replay가 성공을 구분하는지 확인한다. 목표 없이 main:2에서 더 진행할 때는 typed rejection이 명시되는지 확인한다.
- Owner: Architect/Coder (제품 목표의 defer 여부는 Human)
- Confidence: High
- Notes: `spec.md:72-82`의 full content 비목표와 `spec.md:706-728`의 10개 compatibility scenario는 이 부재가 현재 구현 범위에서 의도적일 수 있음을 보여준다. 그러나 사용자 요청의 full game-contract 판정에서는 Major gap으로 남으며, 현재 문서만으로 “full goal을 포기해도 되는지”는 확정할 수 없다.

### [A01-F003] 탐험 중 성장(경험치·레벨·skill) 계약이 없고 자원 루프만 부분적으로 닫힘

- Pass: Implementation Compliance
- Pattern: `IMP-001`, `IMP-003`
- Area: progression / player stats / resources
- Severity: Major
- Status: Confirmed (전체 사용자 목표 대비 Needs Fix; v0.3.0 범위에서는 Needs Spec Clarification)
- Summary: 전투와 1000-turn survival은 존재하지만 player progression이 없다. kill/gold와 hunger/inventory 일부는 semantic delta를 만들지만 경험치·experience level·attribute/weapon skill 성장으로 이어지지 않는다.
- Evidence:
  - `crates/aihack-core/src/world.rs:8-26`의 `WorldState`는 nutrition/luck/prayer/paralysis/hallucinating/kill_count/gold를 가지지만 XP, experience level, attributes, weapon/spell skill을 가지지 않는다.
  - `crates/aihack-ai-contract/src/observation.rs:28-40`의 `PlayerObservation`도 HP/max HP/current level/hunger/luck/cooldown/paralysis/hallucinating만 노출한다.
  - `crates/aihack-core/src/domain/player.rs:3-20`은 고정 player stats만 제공하며 level-up/experience 함수가 없다. `rg` growth 범위에서도 `experience`, `xp`, `skill`, `enhance` player contract가 없다.
  - `crates/aihack-runtime/src/systems/death.rs:59-65`는 monster death를 `kill_count += 1`, `gold += difficulty`로만 반영한다. `crates/aihack-runtime/src/score.rs`(실제 `crates/aihack-core/src/score.rs:3-32`)의 score도 gold/kill/depth/inventory/turn을 읽을 뿐 XP를 읽지 않는다.
  - `tests/long_run.rs:39-120`의 1000-turn gate는 `final_state=Playing`, semantic state 변화, nutrition 감소와 hash 안정성을 확인하지만 player experience/level-up 또는 end goal을 확인하지 않는다.
- Expected Basis: [Guidebook §3.1](https://www.nethack.org/v367/Guidebook.html)에서 experience points와 experience level이 전투·생존 능력을 높인다고 설명하고, [Guidebook §7.2.2](https://www.nethack.org/v367/Guidebook.html)에서 role/experience/use에 따른 weapon proficiency와 `#enhance`를 설명한다. role-specific stat table도 공식 [3.6.7 `role.c`](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/role.c#L24-L50)에 있다.
- Actual: player는 시작 이후 HP/AC/공격 프로필이 고정이다. 자원은 food/corpse→nutrition, potion→HP, wand charge, kill→gold/score, prayer→luck 정도로 부분 구현되어 있고, 성장·숙련으로 이어지지 않는다.
- Impact: 탐험의 장기 의사결정이 “고정 캐릭터로 turn을 버티기”에 머물며 NetHack의 성장/역할 전략과 다른 게임이 된다. 목표/승리 경로(F002)와 결합되면 진행 동기도 닫히지 않는다.
- Suggested Action: 전체 호환 목표면 spec에 XP source, level curve, max HP/attributes, role/race modifiers, skill/proficiency 및 save/replay projection을 먼저 정하고 producer→consumer regression을 추가한다. 축소 slice면 “progression = resource/kill score only; XP/skill deferred”를 명시하고 `survival-v1`을 full progression evidence로 표현하지 않는다.
- Re-audit Method: 동일 seed에서 XP를 발생시키는 kill/탐험 sequence와 control sequence를 실행해 player level/stats/skill 및 후속 legality/damage가 변하는지, save/load/replay와 both-input flow가 일치하는지 확인한다.
- Owner: Architect/Coder (범위 결정은 Human)
- Confidence: High
- Notes: 허기 threshold 자체는 `spec.md:727`의 NH367-C008 계약과 테스트에서 확인했다. `FAINTED/STARVED` death/message 전이가 v0.3.0 범위 밖이라는 명시(`spec.md:727`)는 별도 finding으로 올리지 않았고, 이 finding은 XP/level/skill 부재에 한정한다.

### [A01-F004] Core Quit이 전투 사망으로 기록되어 종료 의미가 어긋남

- Pass: Implementation Compliance
- Pattern: `IMP-001`
- Area: death/quit end semantics / adapter consistency
- Severity: Major
- Status: Confirmed (Needs Fix)
- Summary: public `GameSession::submit(CommandIntent::Quit)`은 quit을 별도 종료 원인이 아니라 `DeathCause::Combat { attacker: EntityId(0) }`인 `GameOver`로 만들고 `CommandRejected` 이벤트를 남긴다. 반면 production TUI의 Q는 core submit 없이 즉시 process exit 후보를 반환한다.
- Evidence:
  - `crates/aihack-runtime/src/session.rs:537-550 (submit_quit)`에서 non-GameOver 상태를 `RunState::GameOver { cause: DeathCause::Combat { attacker: EntityId(0) }, final_score: ... }`로 만들고 `GameEvent::CommandRejected { reason: "quit requested" }`를 `accepted_without_turn`으로 commit한다.
  - `crates/aihack-core/src/domain/combat.rs:49-53 (DeathCause)`에는 Combat/Trap만 있어 quit/escaped/ascended cause가 없다.
  - `apps/aihack-tui/src/tui/mod.rs:826`의 `UiCommandCandidate::Quit => Ok(true)`는 `client.submit(Quit)`을 호출하지 않고 loop를 끝낸다. 따라서 direct session/headless path와 production TUI path의 end semantics가 다르다.
  - `apps/aihack-tui/src/tui/mod.rs:1505-1517`는 Combat cause를 `Killed by entity {attacker}`로 렌더링한다. core quit을 화면에 렌더링하면 `Killed by entity 0`이 된다.
- Expected Basis: [Guidebook §10](https://www.nethack.org/v367/Guidebook.html)에서 quit 시 gold를 보존하는 종료와 사망 시 점수 처리를 구분한다. 공식 [3.6.7 `end.c` end-status table](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/end.c#L2681-L2715)도 `quit`, `escaped`, `ascended`를 death reason과 별도로 유지한다.
- Actual: TUI Q는 즉시 exit하므로 사용자에게 score/end disclosure를 보여주지 않으며, core/adapter API의 Quit는 combat death처럼 보이는 synthetic cause와 rejection event를 만든다. `spec.md:184-212`가 `Playing --death/quit--> GameOver`를 같은 state로 그린 것은 의도일 수 있지만, 현재 cause/event 계약은 quit 의미를 보존하지 않는다.
- Impact: replay/analytics/score consumers가 quit을 사망으로 분류하고, direct API와 TUI에서 같은 command의 결과가 달라진다. 사용자-facing end screen도 실제 원인을 오표시할 수 있다.
- Suggested Action: 제품 계약을 먼저 정해 `Quit`, `Escaped`, `Ascended`, death reason을 별도 typed end reason/state로 표현하고 score/disclosure 규칙을 분리한다. TUI Q가 즉시 process exit인지 GameOver disclosure인지 결정한 뒤 core/client/TUI/replay tests를 동일 경로로 정렬한다. `CommandRejected`를 successful quit outcome의 사건으로 재사용하지 않는다.
- Re-audit Method: direct `GameSession::submit(Quit)`, headless replay의 Quit, TUI Q를 각각 실행해 state/event/score/terminal behavior를 비교하고, combat/trap death와 quit/escape/ascend가 distinct end records를 만드는지 확인한다.
- Owner: Architect/Coder
- Confidence: High
- Notes: TUI의 즉시 Q exit 자체는 NetHack의 `#quit`처럼 process 종료가 될 수 있다. finding의 확정 대상은 별도 quit reason이 없는 core path와 두 adapter path의 불일치이며, 원하는 disclosure semantics는 명세 확인이 필요하다.

### [A01-F005] 기능 전부를 키보드와 마우스로 접근할 수 없고 키보드에도 명령 충돌/방향 손실이 있음

- Pass: Implementation Compliance
- Pattern: `IMP-001`, `DBG-002`
- Area: TUI keyboard/mouse coverage / state-aware dispatch / command reachability
- Severity: Major
- Status: Confirmed (사용자 목표 대비 Needs Fix; 현재 designs 계약은 더 약한 범위를 선언)
- Summary: production dispatcher는 안정성 때문에 modal/blocking state의 mouse를 모두 버리지만 그 modal 안의 유효 CTA도 구현하지 않았다. 일반 Playing mouse는 지도 인접 이동·inspect·focus와 일부 footer/command CTA만 제공한다. 동시에 keyboard mapping은 potion `q`를 Quit으로 가리고, 방향성 command를 East로 고정하며, `Awaiting*` prompt를 live command에서 진입시키지 않는다.
- Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:1650-1681 (runtime_event_to_candidate)`는 mouse가 `UiOverlay != None`, soft input, Title/CharacterCreation/AwaitingDirection/AwaitingInventorySelection/MorePrompt/GameOver일 때 즉시 `None`을 반환한다. inventory overlay의 실제 mouse selection/close, soft judgment submit/cancel, GameOver NewRun, Title start, CharacterCreation back은 mouse로 수행되지 않는다.
  - `apps/aihack-tui/src/tui/mod.rs:1797-1827 (map_mouse_event_for_state)`도 Title/CharacterCreation/GameOver에서 모든 mouse를 `None`으로 만들고, 나머지는 `input::map_mouse_event`와 LLM footer만 연결한다.
  - `apps/aihack-tui/src/tui/input.rs:293-335 (map_mouse_event)`는 지도에서 인접 8방향 Move 또는 Inspect, Inspect/Status/Command panel focus를 만들 뿐이다. `apps/aihack-tui/src/tui/input.rs:86-117 (command_panel_ctas)`는 Inventory/Wait/Open/Inspect/Focus 다섯 CTA만 렌더·hit-test한다.
  - `apps/aihack-tui/src/tui/input.rs:120-149`의 `inventory_panel_ctas`는 inventory를 `.take(4)`로 자르고, `:407-423 (primary_inventory_command)`는 food/corpse, wand, rock, drop에 mouse command candidate를 주지 않는다. 따라서 초기 inventory에서 potion을 주워도 letter `f`가 다섯 번째 entry가 되어 mouse inspect CTA로 보이지 않는다.
  - `apps/aihack-tui/src/tui/input.rs:151-195`에서 `q`는 `UiInputEvent::Quit`으로 baseline에 먼저 등록되고, `:198-290`의 `q => Quaff` arm은 baseline lookup(`:220-229`)에 가려져 도달하지 않는다. 일반 Playing에서 potion `Quaff`의 keyboard candidate가 없다.
  - 같은 baseline의 `o/c/K`는 `Direction::East`로 고정(`apps/aihack-tui/src/tui/input.rs:181-186`)되고, `t/z`도 East payload로 고정(`:261-278`)된다. `crates/aihack-runtime/src/observation.rs:180-187,267-299`는 8방향 legal action을 생성하지만 UI는 그 전체를 노출하지 않는다.
  - `crates/aihack-runtime/src/session.rs:203-234`의 live `Playing` dispatch에는 `AwaitingDirection`/`AwaitingInventorySelection` 진입 command가 없고, `:236-287`은 이미 persisted/fixture 상태에서만 선택을 처리한다. `rg -n "AwaitingDirection|AwaitingInventorySelection" crates apps src tests`에서도 production 진입은 이 fallback 처리와 test/save fixture뿐이다.
  - 기존 `apps/aihack-tui/tests/tui_contract.rs:194-226`의 `modal_and_overlay_mouse_clicks_never_submit_underlying_core_commands`는 modal click이 안전하게 underlying command를 제출하지 않음만 검증한다. 유효한 modal mouse action을 검증하지 않는다. `tests/ui_input_mapping.rs:9-55`도 q/quaff 및 다방향 command를 검증하지 않는다.
- Expected Basis: 사용자의 “모든 기능을 키보드와 마우스로”라는 명시 목표와 [Guidebook §4](https://www.nethack.org/v367/Guidebook.html)의 command/prompt 모델, [Guidebook §9.4 `mouse_support`](https://www.nethack.org/v367/Guidebook.html)의 mouse input/travel 옵션을 기준으로 했다. 프로젝트 자체 [designs.md §6/§10/§11](../designs.md)은 mouse CTA와 keyboard equivalent, render-derived hit rectangle 및 modal 경계를 선언하지만, 이는 현재 사용자 목표(각 기능의 양 입력 접근)보다 약한 계약이다.
- Actual: Map 이동/검사와 일부 CTA만 두 입력으로 연결된다. title/creation/game-over 시작·전환, inventory/soft-input/selection/more prompt, save/load CTA, 다수 item/door/projectile 방향은 mouse path가 없고, keyboard도 potion과 East 외 방향을 완전히 표현하지 못한다.
- Impact: 사용자는 mouse만으로 run을 시작·진행·종료하거나 inventory/selection/soft judgment를 완료할 수 없다. keyboard-only 사용자도 potion quaff와 일부 방향성 command를 정상 수행할 수 없으며, core의 legal `ActionSpace`와 실제 TUI command surface가 달라진다.
- Suggested Action: 모든 UI state에 canonical action/CTA model을 두고 renderer/hit-test/keyboard dispatcher가 공유하게 한다. Title/CharacterCreation/GameOver CTA, inventory modal pagination/letter/action, direction selection, MorePrompt, save/load/error, soft-input submit/cancel에 mouse geometry와 keyboard equivalent를 추가한다. `q` Quit/Quaff를 분리하고, 방향/item을 즉시 East/첫 item으로 고정하지 말고 pending selection 또는 법적으로 검증된 target picker를 사용한다. 각 legal `CommandIntent`를 keyboard와 mouse 모두에서 실행하는 matrix를 작성한다.
- Re-audit Method: 8방향 Move/Open/Close/Kick/Throw/Zap, 각 inventory class의 Wield/Wear/Quaff/Eat/Read/Drop, Search/Pickup/Pray/Descend/Ascend/Save/Load, 모든 Title→Creation→Playing→GameOver→NewRun/Exit 및 modal cancel/submit을 실제 dispatcher→handler에서 양 입력으로 실행한다. 각 command의 accepted/turn/hash/state를 core direct submit과 비교한다.
- Owner: Coder (input architecture는 Architect)
- Confidence: High
- Notes: `designs.md:299-307`의 “underlying mouse를 막는다”와 “모든 mouse CTA에 keyboard equivalent”는 안전한 no-op 경계를 의도한다. 그러나 no-op만 있고 modal CTA가 없으므로 user goal 기준으로는 기능 부재다. `tui_contract` 20개와 `ui_input_mapping` 6개 통과는 이 누락을 반증하지 않는다.

### [A01-F006] 기존 compatibility/long-run 테스트는 full game contract를 입증하지 못함

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`, `IMP-003`
- Area: compatibility evidence / acceptance coverage / false-green risk
- Severity: Major
- Status: Confirmed (전체 목표 대비 Needs Fix 또는 명세 축소 명시)
- Summary: NH367-C001..C010과 1000-turn deterministic gate는 좁은 observable slice를 잘 검증하지만, 시작 선택·성장·승리·escape/ascension·양 입력의 complete playability를 검증하지 않는다. 따라서 현재 compatibility test PASS를 “NetHack 3.6.7에 준하는 완결 게임”으로 해석할 수 없다.
- Evidence:
  - `tests/nethack_367_compat.rs:6-449`에는 정확히 10개 test가 있고 `docs/compatibility/README.md:15-30`도 벽/문/bump/item/stairs/search/projectile/hunger/save/death만 나열한다.
  - `spec.md:66`의 `SC-COMPAT-01`은 P8-G01..G20와 NH367-C001..C010 통과만 요구한다. `spec.md:706-728`에도 Amulet, role/race/gender/alignment, XP/skill, victory/escape/ascension scenario가 없다.
  - `tests/long_run.rs:39-120`은 seed 42/7/1234의 1000 accepted turns와 `RunState::Playing`, semantic change, nutrition decrease, hash stability를 확인한다. 목표 달성 또는 end state를 확인하지 않는다.
  - `tests/ui_input_mapping.rs:9-223`은 baseline 일부, map/status/focus, synthetic Save/Load request와 inspect click을 검사하지만 q/quaff, all legal directions, modal mouse action, Title/Creation/GameOver mouse, all item commands를 검사하지 않는다.
  - `apps/aihack-tui/tests/tui_contract.rs:15-632`의 20개 test는 selected state-aware keyboard, repeat/quarantine, overlay safety, F9, layout/terminal lifecycle를 검사한다. modal mouse가 valid action을 만들지 않는 사실(`:194-226`)을 오히려 고정한다.
  - 표적 실행 결과는 25개 core/compat/UI test와 20개 TUI contract가 모두 통과했지만, 위 누락 영역은 실행되지 않았다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md`의 `IMP-003`/`TEST-001`에 따라 완료 주장에는 목표별 결정적 검증 기준이 있어야 한다. [Guidebook §2](https://www.nethack.org/v367/Guidebook.html), [§3.1](https://www.nethack.org/v367/Guidebook.html), [§5.3](https://www.nethack.org/v367/Guidebook.html), [§10](https://www.nethack.org/v367/Guidebook.html)와 official 3.6.7 [`role.c`](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/role.c), [`end.c`](https://github.com/NetHack/NetHack/blob/NetHack-3.6.7_Released/src/end.c)에서 요구되는 시작/성장/진행/종료 축이 테스트 matrix에 연결되어야 한다.
- Actual: 테스트는 bounded compatibility records와 survival loop에 대해 강한 증거를 제공하지만 full-game claim이나 keyboard↔mouse parity의 acceptance gate가 아니다.
- Impact: 모든 선택된 테스트가 green이어도 F001~F005가 남은 상태를 탐지하지 못하는 false-green이 발생한다. 통합 감사가 이 테스트 결과만으로 PASS하면 사용자의 핵심 목표 coverage가 과대평가된다.
- Suggested Action: 제품 목표를 full game으로 유지할 경우 start-choice, progression, Amulet/offer/escape/ascension, quit semantics, keyboard/mouse command reachability를 별도 scenario ID와 deterministic fixture로 추가하고 release gate에 연결한다. v0.3.0 slice를 유지할 경우 테스트/README/compatibility 문서의 claim을 slice 수준으로 제한하고 full-game PASS 문구를 사용하지 않는다.
- Re-audit Method: 새 acceptance matrix에서 각 행에 source locator, command sequence, expected state/event/hash, keyboard path, mouse path를 연결하고, 성공 run과 intentional out-of-scope row를 구분한 뒤 표적 test를 재실행한다.
- Owner: Architect/Coder (acceptance 범위는 Human)
- Confidence: High
- Notes: 이는 기존 10개 compatibility test의 품질을 부정하는 finding이 아니다. 해당 test들은 각자의 bounded scenario에서는 유효하며, 문제는 그 coverage를 전체 NetHack game-contract evidence로 확장해석하는 것이다.

## 6. Uncertainties and Clarifications Needed

- Product scope: v0.3.0을 C001..C010 compatibility slice로 종료할지, 사용자가 요청한 역할/종족/성별/정렬·성장·Amulet 승리까지 포함하는 full NetHack-like game으로 확장할지 확인이 필요하다. `spec.md:72-82`의 full content 비목표는 존재하지만 start-choice와 success-state를 명시적으로 제외하지 않는다.
- Input parity: “모든 기능을 키보드와 마우스로”가 각 `CommandIntent`의 양 입력 경로를 뜻하는지, 아니면 현재 `designs.md:307`처럼 mouse CTA마다 keyboard equivalent만 뜻하는지 확인이 필요하다. 본 감사는 사용자의 더 강한 표현을 기준으로 F005를 유지했다.
- Quit semantics: Q가 process를 즉시 끝내는 UI-only action인지, `GameOver` disclosure/score를 보여주는 core end outcome인지 문서가 일관되게 정해야 한다. 두 경로가 모두 필요하다면 typed `Quit` end reason을 보존해야 한다.
- Mouse/terminal evidence: 실제 Windows Terminal GUI는 이 관점에서 실행하지 않았다. 현재 판단은 production dispatcher/hit-test code와 기존 synthetic TUI tests에 기반하며, GUI-specific behavior를 추가로 주장하지 않는다.

## 7. Perspective Decision

- Decision: **HOLD for the full user goal; REWORK REQUIRED for a NetHack-3.6.7-like complete game contract.**
- Bounded evidence: deterministic turn transaction, content-backed two-level bootstrap, map/LOS/search, door/combat/trap, inventory/item resource actions, stairs round-trip, hunger thresholds, save/RNG continuation, and combat/trap death→GameOver are supported by the existing 25+20 targeted tests.
- Blocking risks: no selectable role/race/gender/alignment, no XP/level/skill growth, no Amulet/escape/ascension/win path, core quit/death semantic conflation, and incomplete keyboard/mouse command reachability (including the q/quaff collision and modal mouse no-op).
- Scope-safe interpretation: full NetHack content, special branches, and starvation death are documented v0.3.0 limitations and were not misclassified as accidental regressions. They remain gaps against the user’s full comparison goal unless explicitly accepted as deferred scope.
- Re-audit gate: resolve the scope clarifications, then rerun the F001–F006 methods and require a scenario matrix that proves both bounded compatibility and the declared full/limited end-to-end claim.
