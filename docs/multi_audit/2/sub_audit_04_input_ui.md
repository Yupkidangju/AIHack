# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 2
- Perspective: 키보드/마우스 전체 조작 접근성. `CommandIntent`/`UiCommandCandidate` 전수, state-aware key/event mapper, mouse hit-test, rendered affordance, handler 소비 경로를 추적했다.
- User Goal: 게임 루프와 `Title -> 진행 -> 끝`이 NetHack 3.6.7에 준하는지, 인터페이스의 모든 기능을 키보드와 마우스로 정상 플레이할 수 있는지 감사한다.
- Audit Basis: Standard-backed / Goal-driven
- Standard Path: `C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md`; `C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md`
- Snapshot: `HEAD 899660167d59c4b06d27a59c0d75fcccda0cce33`
- Audit Date: 2026-09-05 (Asia/Seoul)
- 판정 가정: 사용자 목표의 “키보드와 마우스”는 각 사용자 기능이 두 입력 방식으로 도달 가능하다는 뜻으로 해석했다. 제품 설계가 마우스 geometry가 없는 modal의 mouse를 의도적으로 무시하는 경우에는 그 사실을 사용자 목표와의 gap으로 별도 표시했다. 이 보고서는 NetHack 전체 구현 적합성이 아니라 TUI 입력·표시 표면을 판정한다.

## 2. Assigned Scope

- `apps/aihack-tui/src/tui/input.rs`: baseline key map, candidate map, mouse hit-test, CTA model
- `apps/aihack-tui/src/tui/mod.rs`: state-aware dispatcher, transition gesture gate, candidate handler, event loop, screen dispatch
- `apps/aihack-tui/src/tui/render_map.rs`, `render_panels.rs`, `layout.rs`, `viewport.rs`, `labels.rs`, `config.rs`: rendered affordance, map/HUD/log/prompt geometry
- `crates/aihack-ai-contract/src/{lib.rs,observation.rs}`, `crates/aihack-core/src/{action.rs,run_state.rs}`: command/state contract
- `crates/aihack-runtime/src/{observation.rs,session.rs,client.rs}`, relevant systems: legal action generation and handler consumer
- `crates/aihack-content/src/data/levels/{main_1.toml,main_2.toml}` and `bootstrap.rs`: visible enemy/ground-item fixture evidence
- Project authority documents: `spec.md`, `designs.md`, `README.md`, `docs/compatibility/NH367-C001..C010.md` (UI-relevant compatibility claims only)
- Targeted verification only; no full workspace suite was run.

## 3. Excluded and Uninspected Scope

- 배포 문서/CI/license 감사, 네트워크/LLM transport 자체, core 전투·AI·저장 규칙의 전체 정합성은 제외했다.
- `legacy_nethack_port_reference/`의 구현 전체, `.git`, `target`, generated/vendor 산출물은 읽지 않았다.
- 다른 감사 보고서, supplement, 부모/다른 agent의 결론은 읽거나 근거로 사용하지 않았다.
- Windows Terminal GUI 자체는 프로젝트 설계가 ConPTY와 구분하므로 자동 판정에 넣지 않았다.
- 실제 모든 `ActionSpace` 조합을 GUI에서 클릭하는 신규 probe/test는 만들지 않았다. 아래 결론은 소스 trace와 기존 대상 테스트/ConPTY 실행에 근거한다.

## 4. Evidence Examined

### 4.1 Input-to-handler trace

| Layer | Evidence | Observed contract |
| --- | --- | --- |
| Core command surface | `crates/aihack-core/src/action.rs:28-68` | `Wait`, `Quit`, `Move`, `Search`, `Kick`, `Open`, `Close`, `Pickup`, item actions, `Pray`, stairs, `AcknowledgeMore`가 `CommandIntent`로 정의된다. |
| Legal action producer | `crates/aihack-runtime/src/observation.rs:173-303` | Playing은 방향별 이동/문/킥/투사체와 item별 명령을 여러 방향으로 생성한다. blocking state도 별도 legal action을 생성한다. |
| Keyboard baseline | `apps/aihack-tui/src/tui/input.rs:151-196` | 이동 8방향, wait/search/open/close/kick/pickup/inventory/stairs/pray/save/load/quit의 일부 단축키를 정의한다. open/close/kick은 East, projectile도 East로 고정된다. |
| Candidate mapper | `apps/aihack-tui/src/tui/input.rs:198-291`; `apps/aihack-tui/src/tui/mod.rs:1539-1645` | LLM 및 state별 후보를 먼저 만들고 `ActionSpace`/현재 inventory와 대조한다. `AwaitingDirection`, inventory selection, MorePrompt, GameOver는 별도 분기한다. |
| Mouse mapper | `apps/aihack-tui/src/tui/input.rs:293-335,338-405`; `apps/aihack-tui/src/tui/mod.rs:1649-1827` | map 인접 click은 Move, 비인접 map은 Inspect, status/inspect/command 일부는 Focus/CTA다. overlay·soft input·blocking state 및 Title/Creation/GameOver의 mouse는 차단된다. |
| Handler | `apps/aihack-tui/src/tui/mod.rs:769-960` | `Command`는 `GameClient::submit`, Save/Load는 quick store, NewRun/BackToTitle/overlay/LLM/focus 후보는 UI/runtime 상태로 소비된다. |
| Event loop | `apps/aihack-tui/src/tui/mod.rs:1223-1309` | draw 후 하나의 `runtime_event_to_candidate`를 호출하고 하나의 handler만 실행한다. |

### 4.2 Screen/state trace

| State/layer | Rendered affordance | Keyboard path → candidate → consumer | Mouse path → hit-test → consumer | Independent result |
| --- | --- | --- | --- | --- |
| Title | `render_title_screen` → `title_lines` (`apps/aihack-tui/src/tui/mod.rs:1313-1322`; `render_panels.rs:329-342`) | Enter → `Command(Wait)` → `GameSession::submit` → CharacterCreation; L → Load; Q/Esc → Quit | `runtime_event_to_candidate`가 Title mouse를 `None`으로 끝낸다 (`mod.rs:1667-1680,1796-1806`). | Enter 기반 시작은 닫혀 있지만 mouse 시작/종료는 없다. 설계의 N alias도 실제 mapper에는 없다. |
| CharacterCreation | `render_character_creation_screen` → `character_creation_lines` (`mod.rs:1324-1333`; `render_panels.rs:344-360`) | Enter → Wait → Playing; Esc → BackToTitle; Q → Quit | Title과 같이 mouse 후보 없음 | 키보드 state 전이는 동작하고 pointer 전이는 없다. |
| Playing | map + STATUS + COMMANDS + LOG + INSPECT + optional DEBUG (`mod.rs:1335-1485`) | baseline key → `Command`; F9/focus/LLM 별도 후보; handler가 `submit` 또는 UI 상태 변경 | map hover/click, status/inspect/command text span, LLM footer만 hit-test (`input.rs:293-362`; `mod.rs:1807-1826`) | mixed input의 일부만 닫혀 있다. |
| Inventory overlay | `render_global_overlay` → 최대 4개 item row + Esc 안내 (`mod.rs:1752-1780`; `render_panels.rs:248-255`) | Esc/I → CloseOverlay; letter → InventoryLetter; handler가 제한된 item-kind/action으로 변환 (`mod.rs:1713-1720,709-746`) | 모든 mouse event가 overlay에서 underlying candidate를 만들지 않는다 (`mod.rs:1667-1680`). 별도 overlay geometry 없음 | 열기(I 또는 COMMANDS click)는 가능하지만 overlay close/select는 keyboard-only다. |
| AwaitingDirection | `awaiting_direction_lines` 중앙 STATE modal (`mod.rs:1347-1356`; `render_panels.rs:385-392`) | h/j/k/l/y/u/b/n → Move → action-specific open/close/kick; Esc → AcknowledgeMore cancel | blocking state라 mouse 무시 | persisted fixture의 keyboard path는 닫혔지만 normal Playing 진입 경로가 없다. |
| AwaitingInventorySelection | `awaiting_inventory_lines` 중앙 STATE modal (`mod.rs:1357-1367`; `render_panels.rs:394-401`) | item letter → InventoryLetter → typed action; Esc → AcknowledgeMore cancel | blocking state라 mouse 무시 | persisted fixture에서만 확인되며 modal item click geometry가 없다. |
| MorePrompt | `more_prompt_lines` 중앙 `--More--` modal (`mod.rs:1368-1369`; `render_panels.rs:403-409`) | 모든 key → AcknowledgeMore → Playing, release/repeat는 gesture gate 적용 | blocking state라 mouse 무시 | keyboard acknowledgement만 있고 normal production transition도 없다. |
| LLM result/footer | COMMANDS row 2에 status/result CTA; `llm_footer_line` (`mod.rs:1395-1408`; `render_panels.rs:158-180`) | G/A/J/Y/N/R → LLM 후보; Judge editor는 character/backspace/Enter/Esc; handler queue/validate/submit/dismiss | footer row의 실제 label span만 `llm_footer_click_candidate`로 동일 후보 생성 (`input.rs:338-362`; `mod.rs:1807-1817`) | 일반 Playing footer CTA는 keyboard/mouse parity가 있으나 soft-input layer에는 mouse geometry가 없다. |
| GameOver | `render_game_over_screen` → cause/turn/depth/score/seed + N/Q (`mod.rs:1505-1536`; `render_panels.rs:362-383`) | N → NewRun → Title; Q/Esc → Quit → process break | GameOver mouse 후보 없음 | keyboard 종료/restart는 닫혀 있으나 pointer end/restart는 없다. |

### 4.3 `CommandIntent` 전수 coverage

| `CommandIntent` | Keyboard mapping | Mouse mapping / rendered affordance | Handler 소비 |
| --- | --- | --- | --- |
| `Wait` | `.`; state action-space gate | `[. ] Wait` command row text span | `client.submit(Wait)` |
| `Quit` | Playing/Title/Creation/GameOver의 Q 또는 Esc | Q 텍스트는 있지만 screen click geometry 없음 | `handle_candidate_owned`의 Quit은 종료; core Quit은 GameOver |
| `Move(Direction)` | `hjklyubn` 8방향 | map 인접 cell click만 방향으로 변환 | `client.submit(Move)`; bump attack는 core가 해석 |
| `Search` | `s` | CTA/지도 hit 없음 | `client.submit(Search)` |
| `Kick(Direction)` | `K` → East; persisted direction modal만 8방향 | map click은 Kick이 아니라 Move | `client.submit(Kick)` |
| `Open(Direction)` | `o` → East; command row는 East legality일 때만 활성 | `[o] Open` text span은 East만; map click은 Move | `client.submit(Open)` |
| `Close(Direction)` | `c` → East | 별도 CTA/지도 방향 click 없음 | `client.submit(Close)` |
| `Pickup` | `,` | 바닥 item click은 Move; pickup CTA 없음 | `client.submit(Pickup)` |
| `Drop { item }` | `d` → inventory 첫 item | inspect row primary command에는 Drop 없음; overlay mouse 없음 | `client.submit(Drop)` |
| `Throw { item, direction }` | `t` → 첫 dagger/rock + East | mouse item/direction CTA 없음 | `client.submit(Throw)` |
| `ShowInventory` | `i` | COMMANDS `[i] Inventory` text span click은 가능 | core no-turn + UI Inventory overlay |
| `Wield { item }` | `w` → 첫 dagger 또는 overlay letter | Inventory presentation의 dagger row만 primary Wield | `client.submit(Wield)` |
| `Wear { item }` | `e` → 첫 leather armor 또는 overlay letter | Inventory presentation의 armor row만 primary Wear | `client.submit(Wear)` |
| `Quaff { item }` | 의도된 `q` branch가 Quit baseline에 가려져 normal key path 없음 | Inventory presentation potion row는 가능하지만 overlay가 keyboard-only | `client.submit(Quaff)` |
| `Eat { item }` | `f` → 첫 food/corpse | inventory row에는 primary Eat 후보가 없음 | `client.submit(Eat)` |
| `Zap { item, direction }` | `z` → 첫 wand + East | inventory row primary/지도 CTA 없음 | `client.submit(Zap)` |
| `Read { item }` | `r` → 첫 scroll | Inventory presentation scroll row primary Read | `client.submit(Read)` |
| `Pray` | `p` | CTA/지도 hit 없음 | `client.submit(Pray)` |
| `Descend` | `>` | stair click은 Inspect/Move이며 Descend CTA 없음 | `client.submit(Descend)` |
| `Ascend` | `<` | stair click은 Inspect/Move이며 Ascend CTA 없음 | `client.submit(Ascend)` |
| `AcknowledgeMore` | MorePrompt의 모든 key, blocking modal Esc | MorePrompt mouse 무시 | `client.submit(AcknowledgeMore)` → Playing |

### 4.4 Non-command `UiCommandCandidate` coverage

| Candidate family | Producer | Consumer/result |
| --- | --- | --- |
| `Command(intent)` | baseline key, state key, map Move, command/LLM CTA | `client.submit`; accepted turn이면 labels 갱신 |
| `Inspect(pos)` | map hover 또는 비인접 map click | hovered position/focus만 변경; non-turn |
| `Focus(panel)`, `FocusNext`, `FocusPrevious` | status/inspect/command click, Tab/BackTab | UI focus만 변경; Log/Inventory 직접 mouse hit는 없음 |
| `Save`, `Load` | S/L key 또는 direct `UiInputEvent` bridge | quick-save/load; 실패는 StorageError overlay |
| `Quit` | Q/Esc/undersized quit path | true를 반환해 event loop 종료 |
| `NewRun`, `BackToTitle` | GameOver N, Creation Esc | session reset; NewRun은 Title로 이동 |
| `CloseOverlay`, `InventoryLetter` | Inventory/Storage keyboard | overlay 닫기 또는 typed item action |
| `ToggleDebug` | F9 | UI-only debug flag; map 위 debug rect가 mouse를 소비 |
| `DismissLlmResult` | N/Esc 또는 footer | presentation result/validated decision 제거 |
| `LlmNarrative`, `LlmSuggest`, `LlmJudge`, `LlmApply`, `LlmRetry` | G/A/J/Y/R key 또는 footer label | queue/modal/validated submit/retry |
| `LlmInput`, `LlmBackspace`, `LlmSubmitInput`, `LlmCancelInput` | soft-input keyboard | 240-char editor/queue/cancel; mouse geometry 없음 |

### 4.5 Targeted verification

| Command | Result |
| --- | --- |
| `cargo test --locked -p aihack --test ui_input_mapping` | 6 passed |
| `cargo test --locked -p aihack --test ui_layout` | 5 passed |
| `cargo test --locked -p aihack --test ui_screens` | 9 passed |
| `cargo test --locked -p aihack --test ui_runtime_smoke` | 9 passed |
| `cargo test --locked -p aihack-tui --test tui_contract` | 20 passed |
| `cargo test --locked -p aihack-tui --test conpty_contract -- --nocapture` | 2 passed on Windows; one-event Title→Creation→Playing, map mouse, Inventory/Esc, quit and repeated Enter boundary를 실행 |

## 5. Findings

### [A04-F001] ActionSpace 전체에 비해 targeted command와 mouse command 표면이 닫히지 않음

- Pass: Implementation Compliance
- Pattern: `IMP-001` (도메인 계약·사용자 UI 레이어 분리 정합성)
- Area: `CommandIntent` → `UiCommandCandidate` → key/mouse mapper → rendered CTA → `GameClient::submit`
- Severity: Major
- Status: Confirmed (user-goal gap; design의 좁은 mouse-CTA 계약과 충돌)
- Summary: runtime `Playing`의 `ActionSpace`는 모든 방향의 문/킥/투사체와 Search, Pickup, Pray, stairs, Drop 등을 노출하지만 TUI는 일부 단축키와 East 고정 targeted key만 제공한다. mouse는 map 인접 Move/Inspect와 일부 panel/footer text span만 만들며 전체 command surface를 만들지 않는다.
- Evidence: `crates/aihack-runtime/src/observation.rs:229-303`은 8방향 Move/Open/Close/Kick/Throw/Zap와 item별 action을 생성한다. `apps/aihack-tui/src/tui/input.rs:151-196`은 Open/Close/Kick을 East로, `t`/`z`도 East로 고정한다. `input.rs:293-335`의 map click은 인접 Move 또는 Inspect뿐이고, `input.rs:86-117`의 command CTA는 Inventory/Wait/Open/Inspect/Focus만 포함한다.
- Expected Basis: 사용자 목표의 “모든 기능을 키보드와 마우스로 플레이”; `designs.md:164-180`의 CTA/동일 candidate 계약 및 `designs.md:302-309`의 mouse CTA keyboard equivalent. NetHack 호환 판단은 적어도 현재 구현된 action을 사용자가 도달할 수 있어야 한다는 UI 조건으로 한정한다.
- Actual: Search, Pickup, Pray, Descend/Ascend, Save/Load, Drop, 일반 방향의 Open/Close/Kick/Throw/Zap에는 mouse command candidate가 없다. 키보드도 `Open/Close/Kick/Throw/Zap`의 방향 선택이 East 또는 persisted blocking-state에 한정된다.
- Impact: legal action이 존재해도 사용자가 해당 방향/행동을 선택하지 못한다. 문이 서쪽/북쪽에 있거나 projectile 목표가 East가 아니면 UI 입력만으로 해당 command를 재현할 수 없다. mouse-only 플레이는 진행·stairs·아이템·종료 기능에서 중단된다.
- Suggested Action: 모든 legal command를 대상으로 방향 선택과 item 선택의 명시적 UI geometry를 만들고, 동일 CTA 모델에서 keyboard/mouse candidate를 파생한다. 두 입력 방식 지원을 제품 목표로 유지하지 않을 경우 `designs.md`/acceptance를 keyboard-equivalent 범위로 명시적으로 좁힌다.
- Re-audit Method: representative `Observation`마다 `legal_actions`의 각 `CommandIntent`에 대해 key path와 실제 화면 cell/CTA click path가 하나씩 존재하는지 표로 대조하고, candidate가 handler에서 실제 accepted/rejected 결과를 만드는지 실행한다.
- Owner: Coder / Architect
- Confidence: High
- Notes: core의 command legality 자체를 반증하는 finding이 아니다. UI 접근 경로의 finding이다.

### [A04-F002] `q`가 Quaff를 가리고, inventory item 선택도 전수 접근되지 않음

- Pass: Implementation Compliance
- Pattern: `IMP-003` (완료 주장과 결정적 검증 기준), backward input contract drift
- Area: keyboard key collision, inventory CTA/hit-test coverage
- Severity: Major
- Status: Confirmed
- Summary: `key_to_candidate`는 baseline candidate를 item shortcut보다 먼저 반환한다. baseline의 `q`는 Quit이므로 뒤의 Quaff branch는 도달 불가하다. 동시에 inventory panel은 첫 4개만 렌더하고 primary command도 일부 kind만 생성한다.
- Evidence: `apps/aihack-tui/src/tui/input.rs:181-195`는 `q -> UiInputEvent::Quit`; `input.rs:220-229`는 baseline이 있으면 즉시 return; `input.rs:247-248`의 `q -> Quaff` branch는 따라서 unreachable이다. `input.rs:120-148`의 `inventory_panel_ctas`는 `.take(4)`와 `primary_inventory_command`를 사용하고, `input.rs:407-421`은 Dagger/Armor/Potion/Scroll만 primary CTA로 만든다. 시작 inventory 5개는 `crates/aihack-runtime/src/bootstrap.rs:76-96`에 있다.
- Expected Basis: 사용자 목표의 모든 keyboard/mouse 기능; `designs.md:99-100`의 inventory letter typed command와 `designs.md:179-180`의 rendered label 기반 click contract.
- Actual: Playing에서 `q` 입력은 Quaff가 아니라 `Command(Quit)`가 되어 core `submit_quit`로 GameOver를 만든다 (`crates/aihack-runtime/src/session.rs:209-227,537-550`). Food/corpse는 inventory row가 보여도 mouse primary action이 없고, 시작 inventory의 다섯 번째 Rock은 overlay/inspect row에 표시되지 않는다. Drop/Throw/Zap도 inventory row click candidate가 없다.
- Impact: potion을 마시려는 정상 입력이 run 종료로 해석될 수 있다. 일부 item은 키로만 indirect first-item shortcut을 써야 하며, item letter와 mouse click으로 원하는 legal item action을 선택할 수 없다.
- Suggested Action: Quit과 Quaff의 key contract를 충돌 없이 분리하고, inventory를 전체 row/scroll 또는 충분한 geometry로 렌더한다. action×item별 명시 CTA/letter selection을 제공해 Drop/Throw/Zap/Eat도 같은 handler 경로로 연결한다.
- Re-audit Method: potion을 inventory에 둔 Playing fixture에서 `q` candidate/handler가 `Quaff`인지 확인하고, 5개 이상 item과 각 item action을 overlay/inspect mouse click 및 letter로 실행해 core turn/state/event를 비교한다.
- Owner: Coder / Architect
- Confidence: High
- Notes: 기존 `ui_input_mapping` 테스트는 `q` collision을 검사하지 않고 `w/t/z/r`만 검사한다 (`tests/ui_input_mapping.rs:36-55`).

### [A04-F003] Title/Creation/Blocking/Overlay/GameOver mouse 경로가 없다

- Pass: Implementation Compliance
- Pattern: `IMP-001`; accessibility contract gap
- Area: state-aware event gating, modal hit-test, start/end flow accessibility
- Severity: Major
- Status: Confirmed (design-intent conflict; user-goal gap)
- Summary: production dispatcher는 Title, CharacterCreation, AwaitingDirection, AwaitingInventorySelection, MorePrompt, GameOver와 모든 overlay/soft-input에서 mouse event를 버린다. 설계는 geometry가 없는 layer의 mouse를 무시한다고 명시하지만, 그 결과 “모든 기능을 키보드와 마우스”라는 사용자 목표에는 미달한다.
- Evidence: `apps/aihack-tui/src/tui/mod.rs:1667-1680`은 overlay/soft input/blocking state의 모든 mouse를 early return한다. `mod.rs:1796-1806`은 Title/Creation/GameOver에서 `None`을 반환한다. `designs.md:299-300`도 Inventory/storage error/soft input/core blocking state의 underlying mouse 후보를 금지한다. 실제 overlay 소비는 `mod.rs:1713-1727`에서 key만 처리한다.
- Expected Basis: 사용자 목표; `designs.md:67-85`의 Title→Creation→Playing→GameOver 흐름, `designs.md:99-107`의 overlay/cancel 계약, `designs.md:302-309`의 접근성 원칙. 설계상 modal mouse 무시는 별도 명세 의도다.
- Actual: mouse로 Title Enter/quit, Creation confirm/back, MorePrompt acknowledge, direction/item selection/cancel, Inventory close/select, StorageError close, Judge submit/cancel, GameOver New Run/Quit을 할 수 없다. Playing의 `[i] Inventory`/LLM footer는 예외적으로 click 가능하지만 overlay에 들어가면 pointer 조작이 끊긴다.
- Impact: mouse-only 사용자에게 start, modal recovery, restart, quit가 불가능하다. blocking state에 잘못 들어가면 keyboard 없이는 진행을 재개할 수 없다.
- Suggested Action: 두 modality를 목표로 유지한다면 각 screen/modal에 실제 rendered button/row geometry와 동일 candidate hit-test를 추가한다. keyboard-only modal이 의도된 제품 범위라면 사용자 목표와 README/acceptance를 그 범위로 명확히 수정하고 mouse 지원 주장을 제한한다.
- Re-audit Method: 80x24 ConPTY/실제 화면에서 Title→Creation→Playing→Inventory→More/selection→GameOver 각 단계의 mouse click을 독립 실행하고, 모든 click이 동일 keyboard candidate와 handler 결과를 만드는지 확인한다.
- Owner: Architect / Coder / Product owner (범위 충돌)
- Confidence: High
- Notes: 기존 `tui_contract`는 modal mouse가 underlying command를 제출하지 않는 음성 조건을 검증한다 (`apps/aihack-tui/tests/tui_contract.rs:194-225`). 이는 안전성 증거이지 modal mouse 지원 증거는 아니다.

### [A04-F004] map renderer가 visible 적과 바닥 item을 그리지 않음

- Pass: Implementation Compliance
- Pattern: `IMP-001` (Observation domain과 rendered UI의 forward sync)
- Area: Observation → MapWidget render → hover inspect context
- Severity: Major
- Status: Confirmed
- Summary: `MapWidget`는 `visible_tiles`와 player만 순회하며 `visible_entities` 또는 map-location item을 렌더하지 않는다. runtime Observation도 `visible_entities`를 actor만 수집하고 item은 inventory observation으로만 노출한다.
- Evidence: `apps/aihack-tui/src/tui/render_map.rs:15-42`에는 tile glyph와 `@`만 있고 entity/item 처리 분기가 없다. labels는 `render_map.rs:44-58`에서 이미 수집된 label의 첫 문자만 오른쪽 셀에 쓰며, 초기 labels는 비어 있고 턴 후 최대 3개만 수집된다 (`apps/aihack-tui/src/tui/mod.rs:312-316,782-795`). `crates/aihack-runtime/src/observation.rs:114-140`의 `visible_entities`는 `entity.actor()`만 통과시키고, `:142-170`의 `inventory_observations`는 inventory location만 반환한다. ground item fixture는 `crates/aihack-content/src/data/levels/main_1.toml:15-29` 및 `crates/aihack-runtime/src/bootstrap.rs:98-111`에 있다.
- Expected Basis: 사용자 추가 감사 범위의 “map renderer가 observation의 적/바닥 아이템을 화면에 실제 표시하는지”; map이 관찰 가능한 play state를 전달해야 한다는 `designs.md:109-154`의 map/HUD 우선순위.
- Actual: seed 42 fixture의 인접 jackal `(6,5)`과 바닥 potion `(8,5)` 같은 entity/item은 map에 glyph가 없다. hover inspect도 `visible_entities`에 없는 ground item을 `entity none`/tile 정보만으로 보여준다 (`render_panels.rs:220-238`). hostile label이 나중에 표시되어도 첫 문자 하나의 transient annotation일 뿐 안정적인 enemy/item glyph가 아니다.
- Impact: 사용자는 bump attack 대상, pickup 대상, projectile target 및 floor item 위치를 map에서 식별할 수 없다. click-to-Move는 동작해도 무엇을 향하는지 알 수 없어 정상적인 플레이/검증이 끊긴다.
- Suggested Action: Observation에 visible map-location item을 포함시키고 MapWidget에 deterministic z-order(terrain → item → actor → player → label)를 구현한다. inspect/hit-test도 같은 projection을 공유하고, label은 보조 affordance로만 남긴다.
- Re-audit Method: visible enemy와 ground item을 포함한 Observation을 buffer에 렌더해 해당 world cell에 glyph가 있는지, hover inspect가 kind/location을 표시하는지, adjacent click이 의도된 action과 결합되는지 확인한다.
- Owner: Coder / Architect
- Confidence: High
- Notes: core entity/item state가 없다는 finding이 아니다. runtime에 상태가 있으나 TUI projection이 소비하지 않는 단절이다.

### [A04-F005] 80x24/60x24에서 LOG 본문이 사라지고 일반 HUD에 hunger가 없음

- Pass: Implementation Compliance
- Pattern: `IMP-001`; `DBG-002` (결정적 상태가 사용자 진단 표면에 닫히지 않음)
- Area: minimum layout → panel content consumption → HP/hunger/log visibility
- Severity: Major
- Status: Confirmed
- Summary: 80x24와 60x24 모두 LOG rect 높이가 1이라 `ThemedTextPanel`의 title만 그려지고 `log_lines` 본문은 표시되지 않는다. STATUS에는 HP는 있지만 hunger field가 없으며 hunger는 F9 debug lines에서만 보인다.
- Evidence: `apps/aihack-tui/src/tui/layout.rs:47-106` 및 58-73은 60x24 degraded, 80x24 standard를 계산한다. 계산 결과 `80x24: status=(52,0,28,10), inspect=(52,10,28,10), log=(0,20,80,1)` 및 `60x24: status=(40,0,20,6), inspect=(40,6,20,6), log=(0,20,60,1)`이다. `render_panels.rs:42-72`는 title을 area.y에 그리고 content를 area.y+1부터 그리므로 height 1에서는 본문이 0행이다. `status_lines` (`render_panels.rs:75-103`)는 turn/hp/level/pos만 반환하고, `log_lines` (`:192-198`)는 priority/event/narrative 본문을 생성한다. hunger는 `debug_observation_lines` (`render_panels.rs:417-447`, 특히 `:435`)에만 있다.
- Expected Basis: 사용자 추가 범위의 log/HP/hunger 최소 80x24 및 60x24 소비; `designs.md:141-154`의 최소 panel 계약과 정보 우선순위(HP/immediate danger, core message, map).
- Actual: 두 최소 크기 모두 HP 문자열은 STATUS content row에 들어가지만 hunger가 일반 HUD에 없고, LOG는 border/title만 남는다. `ui_layout`는 rect bounds와 accessible text function만 확인할 뿐 실제 1행 buffer content를 검사하지 않는다 (`tests/ui_layout.rs:17-25,52-68`).
- Impact: core event/rejection/damage/pickup 메시지와 hunger 상태를 사용자가 확인할 수 없다. 특히 hunger rule이 진행에 영향을 주어도 사용자가 행동을 선택할 feedback이 없다.
- Suggested Action: 최소 layout에서 LOG에 적어도 한 본문 row를 보장하거나 priority message를 STATUS/LOG에 합성하고, regular STATUS에 hunger state/value를 compact하게 추가한다. panel text clipping과 blocking modal 우선순위를 함께 검증한다.
- Re-audit Method: 80x24와 60x24 buffer snapshot에 hit/damage/pickup/low-hunger event를 넣어 LOG 본문, `hp`, `hunger`가 실제 visible row에 있는지 확인하고, modal 표시와 겹치지 않는지 재실행한다.
- Owner: Coder / Architect
- Confidence: High
- Notes: layout geometry test는 PASS지만 content-consumption 판정은 별개다.

### [A04-F006] 명세에 있는 AwaitingDirection/InventorySelection/MorePrompt의 정상 진입 인과가 없음

- Pass: Implementation Compliance
- Pattern: `IMP-002` (현재 Phase 호출 경로와 state 범위 분류)
- Area: Playing → blocking state transition → state-specific mapper/handler
- Severity: Major
- Status: Confirmed
- Summary: `RunState`와 handler는 blocking state를 지원하지만 production `Playing` command 처리에는 해당 state로 들어가는 assignment가 없다. `Awaiting...` 및 `MorePrompt`는 저장 fixture/테스트에서만 구성되는 경로로 보이며 normal play에서 prompt가 나타나지 않는다.
- Evidence: `crates/aihack-runtime/src/session.rs:203-233`의 `submit_in_playing`은 모든 command를 직접 처리하고 blocking state를 설정하지 않는다. `:236-255`와 `:257-287`은 이미 Awaiting 상태일 때만 Playing으로 복귀/재설정한다. `:289-297`은 이미 MorePrompt일 때만 acknowledge를 처리한다. 프로젝트 소스 검색에서 이 상태를 설정하는 production assignment는 이 경로와 테스트 fixture 외에 없다. 반면 `spec.md:185-192`와 `designs.md:67-84,101-104`는 Playing에서 needs direction/item/message overflow transition을 계약한다.
- Expected Basis: `spec.md`의 Title→Creation→Playing, needs direction/item, message overflow, GameOver state graph 및 `designs.md`의 해당 blocking prompt UX.
- Actual: 일반 Playing에서 `o/c/K`가 필요한 direction prompt를 열지 않고, item command가 필요한 selection prompt를 열지 않으며, message overflow도 MorePrompt로 전환시키지 않는다. 따라서 state-specific keyboard path와 blocking-state mouse restriction은 persisted/test state에서만 검증된다.
- Impact: 사용자는 prompt가 제공하는 방향/아이템 선택 UX를 normal game loop에서 사용할 수 없다. F001의 East/first-item shortcut 단절을 복구할 fallback도 없다. “시작→진행→끝” 중 진행 단계의 documented state graph가 실제 실행 인과로 닫히지 않는다.
- Suggested Action: normal command flow에서 direction/item/message overflow를 state transition으로 만들거나, 해당 variants를 현재 Phase 비목표로 명시하고 문서·ActionSpace·UI를 축소한다. 어느 선택이든 unreachable state와 direct shortcut을 같은 contract에 남겨두지 않는다.
- Re-audit Method: clean session에서 각 trigger command를 제출하여 state가 `AwaitingDirection`, `AwaitingInventorySelection`, `MorePrompt`로 변하는지, prompt key/cancel/handler가 다시 Playing으로 복귀하는지 end-to-end로 확인한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: core state transition 구현의 별도 범위 판단은 필요하지만, 현재 문서상 흐름과 production call graph의 단절은 확인된다.

### [A04-F007] Title의 문서상 `N or Enter` 시작 alias가 실제 mapper/render에 없음

- Pass: Implementation Compliance
- Pattern: `IMP-004` (secondary UI description drift)
- Area: Title affordance ↔ state-aware key mapper
- Severity: Minor
- Status: Confirmed
- Summary: active design은 Title에서 `N or Enter`를 CharacterCreation으로 보낸다고 쓰지만, 실제 `title_lines`와 `key_to_candidate_for_state`는 Enter만 시작으로 제공한다.
- Evidence: `designs.md:69-74`는 `N or Enter -> CharacterCreation`; `apps/aihack-tui/src/tui/render_panels.rs:329-342`는 “Press Enter to Start”만 표시한다. `apps/aihack-tui/src/tui/mod.rs:1540-1553`의 Title branch는 Enter, Q, L만 처리한다.
- Expected Basis: active `designs.md` state flow와 표시 affordance의 forward/backward sync.
- Actual: Title에서 N은 candidate가 없고 Enter만 동작한다. Enter-based ConPTY/target test는 통과하지만 N alias는 접근 불가다.
- Impact: 낮은 영향의 문서/입력 drift이며 keyboard Enter 사용자는 정상 시작한다. 사용자에게 표시되지 않은 key를 기대하면 시작이 되지 않는다.
- Suggested Action: N alias를 실제 mapper와 label에 추가하거나 active design에서 alias를 삭제해 단일 contract로 맞춘다.
- Re-audit Method: Title에서 N/Enter 각각의 candidate와 state transition을 실행하고 screen hint와 동일한지 확인한다.
- Owner: Coder / Architect
- Confidence: High
- Notes: mouse start 부재는 A04-F003에서 별도 판정한다.

### [A04-F008] 비-click mouse event가 Map focus 후보로 합성됨

- Pass: Debug / Engineering Quality
- Pattern: `DBG-002` (event-to-candidate determinism)
- Area: crossterm mouse kind normalization
- Severity: Info
- Status: Confirmed
- Summary: `MouseEventKind::Moved`와 `Down` 외 모든 mouse kind가 `FocusPanel(Map)`으로 합성된다. Release/drag/wheel을 no-op으로 다루려는 의도라면 현재 구현은 예상하지 않은 focus side effect를 만든다.
- Evidence: `apps/aihack-tui/src/tui/mod.rs:1695-1707`의 match가 `Moved`, `Down(_)` 외를 `UiInputEvent::FocusPanel(UiPanel::Map)`으로 보낸다. `input.rs:330-335`는 FocusPanel을 항상 Focus candidate로 만든다.
- Expected Basis: 일반 mouse release/drag/wheel은 click command를 중복 제출하지 않고, 실제 click/hover와 의미가 분리되어야 한다는 event lifecycle 불변조건.
- Actual: Up/Drag 등도 Map focus를 변경한다. core turn은 변하지 않지만 focus order와 screen-reader/keyboard context가 비의도적으로 바뀔 수 있다.
- Impact: gate 차단 없는 국소 UX 불안정성. modal/overlay에서는 앞서 early return되어 영향 범위가 제한된다.
- Suggested Action: 지원하지 않는 mouse kind를 `None`으로 명시하고, 실제 focus 변경은 Down 또는 별도 documented gesture만 허용한다.
- Re-audit Method: Down/Up/Drag/Scroll/Move 이벤트를 각 panel에서 dispatch해 candidate와 `focused_panel` 변화가 contract와 일치하는지 확인한다.
- Owner: Coder
- Confidence: Medium
- Notes: 프로젝트가 Up/Drag를 Map focus gesture로 의도했다는 문서 근거는 확인하지 못했다. 의도라면 `designs.md`에 명시할 필요가 있다.

## 6. Uncertainties and Clarifications Needed

- `designs.md:299-300`은 modal/overlay mouse 무시를 명시하고 `:307`은 “모든 mouse CTA에 keyboard equivalent”만 말한다. 이것이 사용자 목표의 “모든 기능 양방식”보다 우선하는지 product owner clarification이 필요하다. 우선순위를 좁은 CTA 계약으로 확정하면 A04-F001/A04-F003의 severity/status를 재분류해야 한다.
- ConPTY 실행은 `apps/aihack-tui/tests/conpty_contract.rs:31-138`의 one-event path(Title/Creation/Playing/map mouse/Inventory/Esc/Quit)와 repeated Enter 경계를 확인한다. 이것은 Title/GameOver/MorePrompt/soft-input의 mouse 지원을 증명하지 않는다.
- 기존 대상 테스트는 state-aware keyboard와 안전한 modal mouse blocking을 주로 검증한다. `q` collision, visible enemy/item glyph, minimum-size LOG body/hunger, 각 legal action의 mouse path에는 회귀 assertion이 없다.
- 이 보고서는 프로젝트가 기록한 NH367 scenario 계약과 TUI contract를 사용했지만 NetHack 3.6.7 원본 전체 UI/command equivalence를 주장하지 않는다. full compatibility 판단은 별도 core/runtime 범위다.

## 7. Perspective Decision

`HOLD` for the stated both-modality goal. Keyboard Enter→Creation→Playing, tested movement/Inventory/Esc, and GameOver keyboard restart/quit paths are present, and all targeted tests passed. 그러나 Major findings A04-F001~F006은 legal command 접근성, mouse-only flow, map observability, minimum HUD feedback, documented blocking-state progression을 막으므로 “모든 기능을 키보드와 마우스로 정상 플레이” 또는 UI 수준의 NetHack-like playable loop를 PASS로 판정할 수 없다. A04-F007은 문서 drift, A04-F008은 비차단 정보 finding이다.

## 8. Coder Handoff

`C:\LocalDev\rust\AIHack\docs\multi_audit\2\sub_audit_04_input_ui.md`를 먼저 읽고, 각 finding을 `spec.md`/`designs.md`와 실제 코드에 대조하여 우선순위대로 수정하세요. 양방식 입력 범위를 바꾸는 경우 관련 문서를 먼저 갱신하고, 수정 후 `ui_input_mapping`, `ui_layout`, `ui_screens`, `ui_runtime_smoke`, `tui_contract`, Windows ConPTY 및 모든 변경된 render/input 경로의 재감사 증거를 기록하세요.
