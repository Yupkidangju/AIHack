# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: TUI·입력·접근성·LLM·플랫폼 통합 (`ui_llm_platform`)
- User Goal: 현재 프로젝트의 문서와 구현을 대조하고 모순·문제점을 찾아 수정 가능한 감사 결과를 만든다.
- Audit Basis: Standard-backed
- Standard Path: `C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md`; `C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md`
- Audit Date/Environment: 2026-08-23, Windows PowerShell, Rust workspace `0.3.0`
- Remote/paid provider call: 수행하지 않음. loopback/fixture 호출도 이 감사에서는 정적 코드와 기존 결정적 테스트로 대체함.

## 2. Assigned Scope

- `apps/aihack-tui`: 화면 상태, 레이아웃, 키보드·마우스 입력, modal, resize, accessibility flag, terminal lifecycle, quick-save/load
- `crates/aihack-llm`: config, loopback transport, schema validation, worker, timeout/busy/stale/fallback, decision approval boundary
- `crates/aihack-ai-contract`: `Observation`, `ActionSpace`, `ClientRevision`, LLM payload DTO
- `crates/aihack-runtime`: run-state transition, save/load adapter, `ArtifactStore`, observation/action projection
- 관련 테스트, PTY/fixture script, `spec.md`, `designs.md`, `IMPLEMENTATION_SUMMARY.md`, `DESIGN_DECISIONS.md`, `BUILD_GUIDE.md`, `audit_roadmap.md`, `docs/R6_MANUAL_MATRIX.md`, `README.md`
- 핵심 대조 항목: Title→CharacterCreation→Playing→Inventory→GameOver→New Run, keyboard/mouse/save/load, resize/modal/minimum terminal, high contrast/reduced motion/text semantics, LLM disabled/busy/timeout/stale/schema/approval, worker shutdown과 파일 경계

## 3. Excluded and Uninspected Scope

- `legacy_nethack_port_reference/` 본문과 과거 구현: 사용자가 명시한 legacy body 제외
- 외부 provider, 실제 모델 추론, 유료/원격 네트워크 호출, 배포·게시·release 승인
- 다른 `multi_audit` 보고서: 독립 판정을 위해 읽지 않음. 감사 표준과 report contract만 읽음.
- Linux `tmux` PTY script의 이번 실행: 현재 감사 호스트가 Windows이며 script가 `bash`/`tmux`를 요구한다. 저장소가 기록한 Linux 결과는 문서 증거로만 확인했다.
- Windows ConPTY/실제 Windows terminal interactive matrix: 실행 환경·fixture가 없어 직접 검증하지 못했다.
- 전체 게임 규칙·콘텐츠 인과·headless 정책: 배정 관점과 직접 연결되는 runtime state/save 경계만 확인했다.

## 4. Evidence Examined

### 권위 문서

- `spec.md`: §3 성공 기준, §4 비목표, §8 상태 전이, §9.4~9.6 LLM 계약, §12 headless, §14 저장·replay, §16 보안 경계, §18 완료 조건
- `designs.md`: §2 release contract, §4 화면 흐름, §5.3 New run reset, §6 CTA/입력, §7 LLM UI 상태, §8 결과 표현, §10 오류 처리, §11 접근성, §12 구현 제약, §13 수동 matrix
- `IMPLEMENTATION_SUMMARY.md`: R6-1~R6-6 구현 주장과 검증 명령
- `DESIGN_DECISIONS.md`: ADR-0026 LLM adapter, ADR-0032 capability save/replay 경계
- `BUILD_GUIDE.md`, `audit_roadmap.md`, `docs/R6_MANUAL_MATRIX.md`: PTY/worker shutdown 및 Linux/Windows 실행 범위
- `README.md`: 현재 상태 및 다국어 문서 정책

### 소스 및 테스트

- TUI: `apps/aihack-tui/src/tui/mod.rs`, `input.rs`, `layout.rs`, `config.rs`, `theme.rs`, `effects.rs`, `render_panels.rs`, `render_map.rs`, `main.rs`
- LLM/contract: `crates/aihack-llm/src/{config,service,worker,transport,decision,narrative,soft_adjudication}.rs`, `crates/aihack-ai-contract/src/{lib,llm,observation}.rs`
- Runtime: `crates/aihack-runtime/src/session.rs`, `observation.rs`, `save.rs`; `apps/aihack-headless/src/main.rs`
- 표적 테스트: `tests/{llm_transport,llm_tui_integration,llm_revision_gate,llm_soft_adjudication,ui_layout,ui_input_mapping,ui_runtime_smoke,ui_screens}.rs`, `apps/aihack-tui/tests/tui_contract.rs`, `tests/headless_paths.rs`, `crates/aihack-runtime/tests/save_paths.rs`
- script: `scripts/r6_pty_matrix.sh`, `scripts/r6_pending_exit_smoke.sh`, `scripts/r8_tui_core_flow.sh`, `scripts/r6_loopback_fixture.py`

### 실행한 명령

- `cargo test --workspace --locked --test ui_layout --test ui_input_mapping --test ui_runtime_smoke --test ui_screens --test llm_tui_integration --test llm_revision_gate --test llm_soft_adjudication --test llm_transport` — exit 0. LLM revision 9개, soft adjudication 5개, transport 22개, TUI integration 10개, input 5개, layout 4개, runtime smoke 7개, screen 8개 전부 통과.
- `cargo test --workspace --locked --test headless_paths --test save_paths` — exit 0. path/save/link/Windows permission 관련 7개 통과.
- `cargo test -p aihack-tui --locked --bin aihack --test tui_contract` — exit 0. TUI contract 6개 통과.
- `cargo run -p aihack-tui --locked --bin aihack -- --help` — exit 0. `--seed`, `--high-contrast`, `--reduced-motion`만 노출되며 save/load 옵션은 없음.
- `cargo tree -d --locked` — crossterm 중복은 보이지 않았고 `aihack-core`는 TUI/HTTP 직접 의존을 갖지 않음.
- 정적 `rg`: `EnableMouseCapture`/`DisableMouseCapture`, TUI `theme()` 소비자, `UiPanel::Inventory` 생성 지점, locale/i18n 구조를 확인. 해당 소비 경로가 없거나 비어 있음.

## 5. Findings

### Pass 1: Implementation Compliance

### [A06-F001] Inventory overlay와 중간 상태 입력 경로가 문서 흐름과 연결되지 않음

- Pass: Implementation
- Pattern: `IMP-001`, `IMP-002`, `TEST-001`
- Area: TUI state machine, inventory, modal/prompt input
- Severity: Major
- Status: Confirmed
- Summary: `I → Inventory overlay → Esc → Playing`과 item-selection/MorePrompt affordance가 표시·core 상태·입력 매퍼 사이에 연결되지 않았다.
- Evidence:
  - `designs.md:64-84`는 `I`가 Inventory overlay를 열고 `Esc`로 닫는 흐름을 정의한다.
  - `apps/aihack-tui/src/tui/input.rs:90-92`의 `i`는 `CommandIntent::ShowInventory`만 생성한다. `apps/aihack-tui/src/tui/mod.rs:515-532`는 이 command를 core에 submit할 뿐 overlay, `focused_panel`, `UiPanel::Inventory`를 설정하지 않는다. `UiPanel::Inventory` 선언은 `input.rs:9-17`에만 있고 생성/전환 호출은 없다.
  - `render_panels.rs:381-397`는 AwaitingInventory에 “Enter letter or Esc”, MorePrompt에 “Press any key to continue”를 표시하지만 `mod.rs:1044-1047`은 두 상태를 일반 `key_to_candidate`로만 보낸다. `input.rs:101-193`에는 item letter를 현재 `InventoryAction`과 연결하거나 `AcknowledgeMore`를 생성하는 경로가 없다.
  - `mod.rs:751-758`의 Esc는 LLM 결과가 없으면 곧바로 `Quit`이다. 따라서 Inventory/awaiting cancel의 표시와 실제 동작이 다르다.
  - `tests/ui_input_mapping.rs`와 `tests/ui_screens.rs`는 순수 mapping 또는 runtime 직접 submit만 확인하고, 실제 `I`, item letter, Esc, MorePrompt 입력의 화면 후속 상태를 잠그지 않는다.
- Expected Basis: `designs.md:64-84`, `designs.md:136-154`, `spec.md:180-208`의 상태/CTA 계약과 표시된 modal instruction.
- Actual: Inventory는 항상 Inspect 패널의 일부 텍스트로만 보이며 전용 overlay/focus 상태가 없다. AwaitingInventorySelection과 MorePrompt에서 사용자가 보는 키를 눌러도 후보가 생성되지 않거나, Esc가 취소 대신 process quit가 된다.
- Impact: 사용자는 요구된 Inventory 흐름을 재현할 수 없고 item 선택 또는 MorePrompt에서 멈추거나 잘못 종료할 수 있다. 이는 core가 안전하더라도 TUI shipped behavior의 기능 손실이다.
- Suggested Action: Inventory overlay와 `UiPanel::Inventory` focus를 명시적인 UI-only state로 구현하거나 문서/문구를 실제 Inspect 동작으로 수정한다. `InventoryAction + InventoryLetter → CommandIntent` 매퍼, `AcknowledgeMore`, 각 상태의 UI-only cancel을 추가하고 Esc/quit 의미를 분리한다.
- Re-audit Method: Title/Creation/Playing에서 `I`, 각 inventory letter, Esc를 실제 binary와 headless UI harness로 실행하고, AwaitingDirection/Inventory/MorePrompt의 transition 및 turn/hash 불변을 assertion한다.
- Confidence: High
- Owner: Coder / TUI maintainer
- Notes: 이것은 LLM 경계와 독립된 TUI 입력 계약 finding이다.

### [A06-F002] Title/CharacterCreation의 표시 CTA와 실제 키·load 경로가 불일치

- Pass: Implementation
- Pattern: `IMP-001`, `IMP-002`
- Area: Title, CharacterCreation, save/load affordance
- Severity: Major
- Status: Confirmed
- Summary: Title의 `L - Load Game`, CharacterCreation의 `Esc - Back to Title`, direction prompt의 `Esc to cancel`이 실제 runtime mapping에서 각각 load/back/cancel로 동작하지 않는다.
- Evidence:
  - `apps/aihack-tui/src/tui/render_panels.rs:316-346`은 Title에 `L - Load Game`, CharacterCreation에 `Esc - Back to Title`을 표시한다.
  - `apps/aihack-tui/src/tui/mod.rs:1025-1037`의 Title/CharacterCreation 매퍼는 Enter→Wait와 q/Q→Quit만 허용한다. L은 `key_to_candidate_for_state`에 도달해도 후보가 없으며, CharacterCreation Esc는 `mod.rs:751-758`에서 Quit가 된다.
  - `render_panels.rs:372-379`는 AwaitingDirection에서 Esc 취소를 안내하지만 같은 main mapping은 Esc를 quit/dismiss로만 처리한다.
  - `apps/aihack-tui/src/tui/input.rs:95-97`의 S/L baseline은 Playing 계열에서만 쓰이고, `--help`에는 save/load 경로가 노출되지 않는다. Title의 L 문구와 실제 state/action-space는 닫혀 있지 않다.
- Expected Basis: 표시된 UI 계약 자체, `designs.md:64-84`, `spec.md:180-208`의 Title→Creation→Playing state graph. 제품이 Title load/back을 의도하지 않는다면 문구를 제거하고 명세를 좁혀야 한다.
- Actual: L은 dead affordance이고 Esc는 back/cancel이 아니라 quit이다. 사용자는 화면 안내만 보고 기대한 경로를 실행할 수 없다.
- Impact: 시작/취소/복귀 흐름의 discoverability와 안전성이 깨지고, 잘못된 Esc 입력이 세션 종료를 유발한다. Title load는 process별 temp quick-save 정책과도 충돌한다.
- Suggested Action: back/cancel을 별도의 UI state transition으로 구현하고 Title load의 source of truth와 지속 범위를 결정한다. 구현하지 않을 기능은 화면 문구에서 제거하고 keyboard baseline/ActionSpace/문서를 단일 계약으로 맞춘다.
- Re-audit Method: 실제 TUI에서 Title L, Creation Esc, AwaitingDirection Esc를 각각 실행하고 예상 상태·exit code·turn/hash를 기록한다. save/load 정책이 실행 단위라면 Title L을 제거했는지 확인한다.
- Confidence: High
- Owner: Architect / Coder
- Notes: GameOver의 N/Q는 별도 A06-F003에서 다룬다.

### [A06-F003] New Run이 transient UI/LLM 상태를 완전히 초기화하지 않고 GameOver N 입력 우선순위가 뒤집힘

- Pass: Implementation
- Pattern: `IMP-002`, `IMP-003`
- Area: GameOver/New Run, stale LLM lifecycle, UI transient state
- Severity: Major
- Status: Confirmed
- Summary: 문서가 요구하는 New Run reset 목록과 실제 reset 범위가 다르며, GameOver에서 결과가 남아 있으면 N이 New Run 대신 LLM dismiss로 소비된다.
- Evidence:
  - `designs.md:133-135`는 New Run에서 world/turn/RNG/event log, outstanding request/result, hover/focus/modal/debug를 reset하고 theme와 accessibility 설정만 유지하도록 요구한다.
  - `apps/aihack-tui/src/tui/mod.rs:552-557`의 `NewRun`은 client 재생성, `dismiss_llm_result`, `soft_input=None`, `queued_llm_request=None`만 수행한다. `dismiss_llm_result`(`mod.rs:462-475`)는 `outstanding_llm_request`를 제거하지 않고 `ignored=true`만 설정한다. `hovered_pos`, `focused_panel`, `debug_observation_visible`, `active_labels`, `last_label_update_turn`, `next_effect_id`도 reset하지 않는다.
  - `mod.rs:455-460`에서 outstanding request 자체가 `has_llm_result()`다. `mod.rs:751-755`의 N/Esc 우선 분기가 `runtime_key_to_candidate`보다 먼저 실행되므로 GameOver에서 결과가 남아 있으면 첫 N은 Dismiss다. GameOver 매퍼의 N→NewRun은 `mod.rs:1039-1042`까지 도달하지 않는다.
  - `queue_llm_request`는 `mod.rs:358-360`에서 outstanding이 남아 있으면 새 요청을 enqueue하지 않는다. 늦은 응답이 오기 전 새 run의 같은 kind 요청이 막힌다.
- Expected Basis: `designs.md:133-135`, `spec.md:191`의 New Run reset 계약, `designs.md:64-84`의 GameOver N→Title 흐름.
- Actual: 결과/ignored request와 UI overlay/focus/labels가 새 run으로 새어 나갈 수 있고, GameOver N을 한 번 더 눌러야 할 수 있다. pending response가 사라지면 LLM kind가 영구적으로 막힐 가능성도 있다.
- Impact: 사용자 입력이 state-aware하지 않으며 old-session presentation이 새 session에 표시되거나 요청 lifecycle이 교착된다. core hash를 직접 바꾸지는 않지만 UI·LLM 계약을 위반한다.
- Suggested Action: New Run용 전용 `reset_transient_state`를 만들어 outstanding를 ignored registry로 분리한 뒤 response를 drain/discard하고, hover/focus/modal/debug/effects/labels를 명시적으로 초기화한다. main loop에서 GameOver/Title state를 먼저 판정해 N/Q를 LLM dismiss보다 우선시킨다.
- Re-audit Method: pending narrative/decision을 만든 뒤 사망시키고 N 1회로 Title 전환, old response 도착 전 새 G/A/J 요청, labels/focus/debug/hash reset을 확인한다.
- Confidence: High
- Owner: Coder / TUI maintainer

### [A06-F004] Responsive layout tier가 designs/spec의 minimum terminal 계약과 drift

- Pass: Implementation
- Pattern: `IMP-001`, `BUILD-001`
- Area: layout, resize, minimum terminal
- Severity: Major
- Status: Confirmed
- Summary: 문서의 120x36/80..119/60..79 layout policy와 실제 fixed-width layout이 다르다.
- Evidence:
  - `designs.md:88-120`은 120x36에서 map 70%/side 30%, 80..119에서 65%/35%, 60..79에서 map 위·HUD 아래 수직 배치를 정의한다. 60 미만 또는 높이 24 미만은 core status와 안내만 표시하도록 한다.
  - `apps/aihack-tui/src/tui/layout.rs:47-55`는 120x36만 Roomy, 100x32 이상만 Standard, 나머지를 Degraded로 분류한다. `layout.rs:58-73`의 Degraded는 map 40x20과 우측 status/inspect를 고정하며, `layout.rs:76-110`의 Standard/Roomy도 map width 60 고정이다. 80..119용 vertical branch가 없다.
  - `tests/ui_layout.rs:7-13`은 80x28에서 map 40 고정을 정답으로 고정하지만 designs의 80..119 policy를 검증하지 않는다.
- Expected Basis: `designs.md:88-120`, `designs.md:260-274`의 resize/core-turn 불변 계약, `spec.md:180-208`의 UI state 보호.
- Actual: 80x24와 60x24에서 map/side panel이 문서의 vertical/split 비율과 다르고, 20-column side panel과 3-line command panel에 긴 status/CTA가 clip될 수 있다. resize 자체는 다음 frame에서 재계산되지만 문서 layout 의미는 보존되지 않는다.
- Impact: minimum terminal에서 core status, prompt, CTA의 가독성과 mouse hit geometry가 예측과 달라진다. 현재 geometry test PASS는 문서 정합성 PASS가 아니다.
- Suggested Action: 문서의 breakpoint/비율을 구현하거나, 실제 fixed layout을 authoritative spec으로 승격해 designs와 test를 함께 수정한다. 각 matrix 크기에서 모든 core status와 CTA가 온전히 들어가는 assertion을 추가한다.
- Re-audit Method: 120x36, 100x32, 80x24, 60x24, 59x23에서 panel rect·문자 clipping·mouse hit target을 snapshot/PTY로 확인한다.
- Confidence: High
- Owner: Architect / Coder

### [A06-F005] High-contrast/reduced-motion/focus 설정이 실제 렌더러와 연결되지 않음

- Pass: Implementation
- Pattern: `IMP-001`, `TEST-001`
- Area: accessibility, theme, reduced motion, focus semantics
- Severity: Major
- Status: Confirmed
- Summary: 접근성 flag와 theme token은 계산되지만 render path에 소비되지 않는다. 텍스트 badge 일부는 존재하지만 high-contrast의 7:1 목표와 focus order는 검증·강제되지 않는다.
- Evidence:
  - `designs.md:280-283`은 색 외 텍스트 상태, high contrast 7:1, reduced motion에서 spinner 대신 고정 `...`, mouse CTA의 keyboard equivalent, `map→HUD→inventory/inspect→LLM result→footer` focus order를 요구한다.
  - `apps/aihack-tui/src/tui/config.rs:7-24`에는 `enable_mouse`, `enable_animations`, `reduced_motion`, `high_contrast`가 있지만 `rg`상 `enable_mouse`/`enable_animations` 소비자가 없다. `mod.rs:485-487`의 `theme()`도 호출자가 없다.
  - `render_panels.rs:16-41`의 `TextPanel`은 buffer style을 reset하고 문자를 쓸 뿐 `UiTheme`의 foreground/background를 적용하지 않는다. `render_map.rs:14-37`도 glyph만 쓰며 색/contrast style을 적용하지 않는다.
  - reduced motion은 `effects.rs:43-57`에서 TTL 숫자만 줄이지만 `mod.rs:944-951`은 effect를 debug count로만 표시하고 실제 animation renderer가 없다. keyboard focus 순환이나 Tab mapping도 `mod.rs:1051-1059`의 Enter/Char 외에는 없다.
  - `tests/ui_effect_projection.rs:55-62`는 theme token 색 값이 서로 다른지만 실제 rendered buffer style을 검사하지 않는다. `tests/llm_soft_adjudication.rs:96-124`도 textual lines와 core 불변만 확인한다.
- Expected Basis: `designs.md:280-283`, `spec.md:16`의 UI/LLM read-only 경계, 일반적인 접근성 불변조건(색 대비와 keyboard-only 사용 가능성).
- Actual: `--high-contrast`는 실제 화면에 색 대비 변화를 보장하지 않으며 terminal 기본 색에 의존한다. reduced-motion은 표시되는 animation을 제어하지 않고, focus 순서는 mouse click 외에는 존재하지 않는다.
- Impact: 색각·운동·screen-reader/keyboard 사용자에게 문서가 약속한 접근성 보장이 없다. LLM 상태가 텍스트로 구분되는 부분은 PASS지만 전체 accessibility gate는 PASS할 수 없다.
- Suggested Action: 모든 panel/widget에 theme style을 주입하고 high-contrast foreground/background를 buffer assertion으로 잠근다. focusable 영역과 Tab/Shift-Tab 순서를 구현하거나 명세에서 focus 계약을 제거한다. reduced-motion은 실제 effect renderer와 status indicator에 적용하고 dead flag는 제거한다.
- Re-audit Method: default/high-contrast buffer snapshot의 실제 fg/bg와 상태 badge를 비교하고, keyboard-only focus traversal 및 reduced-motion frame output을 검증한다.
- Confidence: High
- Owner: Coder / Accessibility owner

### [A06-F006] Save/load 후 이전 revision의 LLM presentation과 UI transient가 무효화되지 않음

- Pass: Implementation
- Pattern: `IMP-001`, `DBG-002`
- Area: save/load lifecycle, stale response, presentation state
- Severity: Major
- Status: Confirmed
- Summary: load는 `GameClient`만 교체하고 이전 narrative/suggestion/verdict, validated decision, pending request, hover/focus를 invalidation하지 않는다.
- Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:376-382`의 `load_from_path`는 `self.client.load_from_path(path)`만 호출한다.
  - `TuiApp` 필드와 `mod.rs:397-475`의 setter/reader는 loaded revision을 기존 `latest_narrative`, `latest_decision`, `latest_soft_adjudication`, `validated_decision`, `outstanding_llm_request`, `hovered_pos`, `focused_panel`과 대조하거나 초기화하지 않는다.
  - `designs.md:198-200`은 narrative를 `session_revision`이 일치할 때만 표시하도록 하며, `designs.md:271-272`는 typed save/load error panel을 요구한다. `mod.rs:241-268`의 revision 검사는 새 response 수신 시에만 수행된다.
  - `tests/ui_runtime_smoke.rs:15-23`, `43-51`은 같은 valid path의 save/load bridge만 확인한다. 서로 다른 turn/hash를 저장한 뒤 load했을 때 old presentation이 숨겨지는지 검사하지 않는다.
- Expected Basis: `designs.md:198-200`, `spec.md:499-510`의 current revision/stale 규칙, New Run reset과 동일한 transient isolation 원칙.
- Actual: 다른 session을 load해도 이전 결과가 Inspect/Log에 남고, old validated suggestion이 footer Apply 후보로 남을 수 있다. pending request는 old revision으로 남아 응답 시점까지 Busy/Pending을 차지한다.
- Impact: 사용자에게 현재 save와 무관한 LLM 설명/추천이 표시되며, stale action을 누를 때까지 잘못된 CTA가 보인다. core submit은 재검증으로 보호되지만 UI truth가 오염된다.
- Suggested Action: load 성공 직후 `invalidate_for_revision`을 호출해 presentation/result/validated decision/hover/focus/modal/effect를 지우고 pending request를 ignored+drain 상태로 전환한다. load 실패는 TUI error state에 typed 요약과 제한된 path만 표시한다.
- Re-audit Method: turn/hash가 다른 save를 load한 후 old text/Apply가 0건인지, pending response가 discard되고 새 요청이 가능해지는지, invalid path가 process 종료 없이 panel에 표시되는지 확인한다.
- Confidence: High
- Owner: Coder / TUI maintainer

### [A06-F007] Decision timeout과 rationale 최소 길이 계약이 public path·문서·구현에서 갈라짐

- Pass: Implementation
- Pattern: `IMP-003`, `DBG-002`
- Area: LLM timeout/schema contract
- Severity: Minor
- Status: Confirmed
- Summary: production worker config의 decision timeout은 1500ms지만 public helper `request_decision()`은 2000ms이며, rationale 최소 길이에 designs와 spec이 서로 다르다.
- Evidence:
  - `crates/aihack-llm/src/config.rs:5-9`의 `DEFAULT_DECISION_TIMEOUT_MS`는 1500이고 `config.rs:146-151`은 Decision/SoftAdjudication에 이를 사용한다.
  - `crates/aihack-llm/src/decision.rs:17`, `decision.rs:305-313`의 `DECISION_TIMEOUT_MS`와 `request_decision()` 기본값은 2000이다. TUI의 `LocalLlmService`와 별개의 public decision path가 다른 deadline을 갖는다.
  - `designs.md:209-212`는 rationale 1..=160자를 말하지만 `spec.md:503-505`와 `decision.rs:149-155`는 rationale 0..=160자를 허용한다. Timeout UI `render_panels.rs:114-121`도 실제 1500/2000ms 수치를 표시하지 않는다.
- Expected Basis: `spec.md:393`, `spec.md:503-505`, `designs.md:209-212`의 단일 LLM contract.
- Actual: caller/path에 따라 동일 Decision이 1.5초 또는 2초 경계를 갖고 빈 rationale 허용 여부도 다르게 해석된다.
- Impact: failure matrix와 provider integration의 timeout/schema 기대가 갈라지고, public API consumer가 TUI와 다른 behavior를 본다. 현재 TUI core isolation은 유지되지만 contract drift다.
- Suggested Action: timeout source를 `LocalLlmConfig` 또는 단일 상수로 통합하고 Decision/Soft UI에 실제 deadline을 표시한다. rationale 최소 길이를 spec/designs/code/test 중 하나로 확정한다.
- Re-audit Method: public helper, worker transport, UI timeout을 같은 fixture에서 측정하고 empty rationale/timeout boundary test를 양 경로에 추가한다.
- Confidence: High
- Owner: Architect / LLM maintainer

### [A06-F008] UI·LLM user-facing 문자열의 i18n authority가 없고 fallback 언어가 혼재

- Pass: Implementation
- Pattern: `SPEC-GAP-001`, `DOC-BACKFILL-001`
- Area: i18n, UI/LLM strings
- Severity: Major
- Status: Needs Clarification
- Summary: README와 전역 작업 규칙은 5개 언어를 요구하지만 product `spec.md`/`designs.md`에는 UI locale source가 없고, TUI 문자열은 영어 고정이며 LLM fallback narrative만 한국어다.
- Evidence:
  - `README.md:5-68`은 한국어·영어·일본어·중국어 번체·간체 문서 섹션을 제공한다. 프로젝트 `AGENTS.md` §8은 사용자 노출 코드의 5개 언어 i18n을 요구한다.
  - `apps/aihack-tui/src/tui/render_panels.rs:95-167`, `render_panels.rs:316-397`은 LLM badge, CTA, Title/Creation/GameOver/상태 오류를 영어 literal로 직접 생성한다. locale 선택기, 번역 테이블, locale 환경변수는 `rg`에서 확인되지 않는다.
  - `crates/aihack-llm/src/narrative.rs:94-101`의 deterministic fallback은 `턴 ... 층에서 HP ...` 한국어이고, `soft_adjudication.rs:66-68` 및 TUI fallback은 영어다.
  - `spec.md`와 `designs.md`에는 locale/i18n 계약이나 지원 언어 범위가 없다. README 다국어는 문서 정책이지 TUI locale source를 정의하지 않는다.
- Expected Basis: 전역 i18n 정책과 사용자의 “UI/LLM 문자열·i18n 정책 모순 여부” 질문. 제품 요구가 English-only인지 5개 UI locale인지 명세가 결정해야 한다.
- Actual: 현재 화면은 언어를 선택·상속할 수 없고 동일 failure flow에서 영어와 한국어가 섞인다.
- Impact: 지원 언어를 제품 약속으로 해석하면 모든 UI 상태/CTA가 미완료이며, English-only로 해석해도 fallback 언어 혼재가 일관성을 깨뜨린다. 요구 authority가 없으므로 전체 PASS를 확정할 수 없다.
- Suggested Action: `spec.md`에 TUI/LLM presentation 지원 locale, default/fallback, wire enum과 표시 문자열의 분리를 확정한다. 그 결정에 따라 strings를 externalize하고 locale별 snapshot을 추가하거나 English-only 범위를 명시한다.
- Re-audit Method: 다섯 locale 또는 승인된 locale matrix에서 Title/Creation/Play/Inventory/GameOver/LLM failure CTA를 모두 렌더하고 untranslated/mixed fallback을 검사한다.
- Confidence: High (actual drift), Medium (required locale scope)
- Owner: Architect / Product owner

### Pass 2: Debug / Engineering Quality

### [A06-F009] Mouse capture가 활성화되지 않아 실제 mouse flow가 pure mapper에만 머묾

- Pass: Debug
- Pattern: `DBG-001`, `TEST-001`
- Area: crossterm input, mouse/CTA integration
- Severity: Major
- Status: Confirmed
- Summary: `enable_mouse=true` 기본 설정과 mouse mapper는 있지만 terminal에 `EnableMouseCapture`를 보내는 코드가 없어 일반 실행에서 mouse event가 발생한다는 보장이 없다.
- Evidence:
  - `apps/aihack-tui/src/tui/config.rs:7-24`는 `enable_mouse` 기본값을 true로 둔다.
  - `apps/aihack-tui/src/tui/mod.rs:770-785`는 `Event::Mouse`를 읽고 `map_mouse_event_for_state`로 전달하지만 `rg`상 `EnableMouseCapture`/`DisableMouseCapture` 호출이 없다. `run_tui_with_service`의 terminal setup은 `mod.rs:686-691`에서 alternate/raw만 설정한다.
  - `designs.md:145-147`은 mouse click이 keyboard와 같은 CTA ID를 생성해야 한다. 현재 `tests/ui_input_mapping.rs:58-101`, `tests/llm_tui_integration.rs:134-165`은 synthetic coordinates/string mapper만 검사한다.
- Expected Basis: `designs.md:145-147`, `UiRuntimeConfig.enable_mouse`, crossterm terminal lifecycle 불변조건.
- Actual: OS/terminal이 capture를 기본 제공하지 않으면 click/hover/LLM footer CTA가 전혀 전달되지 않는다. pure function PASS는 event source가 연결됐다는 증거가 아니다.
- Impact: keyboard-only는 일부 동작하지만 mouse 사용자와 documented mouse CTA가 실패한다. `enable_mouse=false`도 실제 capture disable path가 없어 설정이 무의미하다.
- Suggested Action: terminal setup/teardown에 조건부 `EnableMouseCapture`/`DisableMouseCapture`를 RAII guard로 연결하고, restore 오류에도 capture disable을 시도한다. 실제 PTY/ConPTY에서 click/hover/CTA를 검증한다.
- Re-audit Method: mouse enabled/disabled 각각에서 crossterm event capture 여부와 footer click→same candidate를 실제 terminal에서 확인한다.
- Confidence: High
- Owner: Coder / Platform maintainer

### [A06-F010] Terminal restore가 initialization/error path에서 exception-safe하지 않음

- Pass: Debug
- Pattern: `DBG-001`, `TEST-001`
- Area: Linux/Windows terminal restore, worker shutdown lifecycle
- Severity: Major
- Status: Confirmed
- Summary: normal loop exit에서는 terminal restore 후 worker grace wait 순서가 맞지만, raw/alternate setup 중 오류와 restore 단계의 첫 오류에서는 alternate/raw 상태가 남을 수 있다.
- Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:686-691`은 `EnterAlternateScreen` 후 `enable_raw_mode`, `Terminal::new`, `GameSession::try_new`를 모두 restore guard 밖에서 `?`로 반환한다. 중간 실패 시 `LeaveAlternateScreen`/raw disable가 실행되지 않는다.
  - 정상 loop 이후의 restore는 `mod.rs:800-808`에만 있고 `disable_raw_mode()?`가 실패하면 `LeaveAlternateScreen` 호출도 건너뛴다. worker는 그 뒤 `shutdown_with_grace(250ms)`로 기다린다.
  - `tests/llm_transport.rs`의 worker grace test와 `docs/R6_MANUAL_MATRIX.md:14-55`의 Linux tmux pending-exit 기록은 정상 pending 경로만 증명한다. setup failure/restore failure injection은 없다.
- Expected Basis: `designs.md:295-296`, `spec.md:393`의 terminal restore-before-worker wait와 플랫폼 lifecycle 안전성.
- Actual: 정상 Q 종료는 의도한 순서를 따르지만 setup/restore error branch에는 복구 보장이 없다.
- Impact: Windows/Linux에서 startup 또는 terminal I/O 오류가 나면 사용자의 shell이 raw mode/alternate screen에 남을 수 있다. 이는 LLM worker가 종료되어도 복구되지 않는 플랫폼 장애다.
- Suggested Action: alternate/raw/mouse state를 scope guard로 소유하고 모든 단계에서 best-effort restore를 수행하며 첫 오류를 보존한다. restore 이후 worker grace를 시작하는 구조를 유지한다.
- Re-audit Method: raw enable, `Terminal::new`, draw/size, disable raw 각각의 failure를 주입하고 shell mode/alternate screen 복구 및 250ms bounded shutdown을 Linux와 Windows에서 확인한다.
- Confidence: High
- Owner: Coder / Platform maintainer

### [A06-F011] Windows interactive terminal evidence가 없어 플랫폼 PASS를 확정할 수 없음

- Pass: Debug
- Pattern: `TEST-001`, `BUILD-001`
- Area: Windows/Linux platform coverage
- Severity: Major
- Status: Needs Clarification
- Summary: 코드가 crossterm cross-platform API를 사용하지만 실제 PTY evidence는 Linux tmux뿐이며 Windows CI는 cargo test/release/checkpoint만 실행한다.
- Evidence:
  - `docs/R6_MANUAL_MATRIX.md:1-10`이 환경을 명시적으로 `Linux, tmux PTY`로 기록한다. `scripts/r6_pty_matrix.sh:1`, `r6_pending_exit_smoke.sh:1`도 Bash/tmux 전제다.
  - `.github/workflows/ci.yml:13-46`의 Windows job은 test, checkpoint, release bundle, audit/deny를 실행하지만 TUI ConPTY, mouse capture, raw/alternate restore smoke를 실행하지 않는다.
  - `BUILD_GUIDE.md:332-347`은 PTY 명령을 공통 재현 절차처럼 적지만 Windows terminal interactive 결과나 대체 harness를 제공하지 않는다.
- Expected Basis: 사용자 핵심 질문의 “Windows/Linux terminal restore와 worker shutdown” 및 `designs.md:295-296`의 platform-independent behavior 목표.
- Actual: Linux normal pending-exit의 문서 증거만 있고 Windows interactive restore/mouse/resize는 미검증이다.
- Impact: Windows에서만 나타나는 ConPTY/raw/mouse/alternate-screen 차이를 배제하지 못해 이 관점의 전체 PASS가 불가능하다.
- Suggested Action: Windows runner에 ConPTY/Windows Terminal 또는 자동화 가능한 console harness를 추가해 Q/pending, resize, mouse capture, initialization failure를 검증하거나, 문서의 보장 범위를 Linux PTY로 명시적으로 축소한다.
- Re-audit Method: 동일 fixture를 Linux PTY와 Windows ConPTY에서 실행하고 terminal restore-before-worker wait, exit code, mode restoration, CTA click 결과를 비교한다.
- Confidence: High for coverage gap; implementation status unresolved
- Owner: Platform owner / CI maintainer
- Notes: 이 finding은 “Windows 코드가 반드시 실패한다”는 주장이 아니라 사용자 핵심 목표의 증거 공백이다.

### [A06-F012] TUI save/load 오류가 화면 상태로 처리되지 않고 process error로 전파됨

- Pass: Debug
- Pattern: `DBG-001`, `TEST-001`
- Area: save/load UX, error recovery
- Severity: Major
- Status: Confirmed
- Summary: 문서는 typed save/load error를 TUI panel에 표시하도록 하지만 runtime loop는 오류를 `?`로 전파해 종료한다. 성공/실패 feedback도 별도 UI 상태가 없다.
- Evidence:
  - `designs.md:270-272`는 invariant/error와 save/load error를 최상위 typed summary panel로 표시하고 secret/path detail을 숨기도록 한다.
  - `apps/aihack-tui/src/tui/mod.rs:543-549`의 Save/Load arm은 `self.save_to_path?()`/`self.load_from_path?()`를 호출한다. main loop `mod.rs:789-794`의 `app.handle_candidate(...)?`가 이를 `run_result`로 전파하고 종료 cleanup 뒤 process error가 된다.
  - `crates/aihack-core/src/error.rs:26-35`의 `GameError` Display는 `Io(String)` 및 `InvalidRuntimePath(String)` 원문을 보유한다. 화면용 redacted summary/경로 정책을 구현한 field가 없다.
  - valid path bridge만 `tests/ui_runtime_smoke.rs:15-23,43-51`에서 PASS한다. invalid schema/path/permission의 non-exit UI 회귀가 없다.
- Expected Basis: `designs.md:270-272`, `spec.md:724-729`의 redacted save/path error boundary.
- Actual: save/load 오류는 TUI에서 재시도·dismiss할 수 없고 stderr/process exit로 끝난다. raw `GameError` 문자열이 terminal cleanup 후 출력될 수 있다.
- Impact: 디스크 오류·손상 save·path rejection이 사용자 세션을 종료시키며, 오류 세부 정보가 UI 경계 밖으로 노출될 수 있다.
- Suggested Action: `UiState`에 redacted storage error와 Retry/Dismiss CTA를 추가하고 command handler가 `Ok(false)`로 복구하도록 한다. `GameError` 원문과 UI summary를 분리한다.
- Re-audit Method: 손상 save, read-only/permission failure, invalid path, atomic replace failure fixture에서 TUI가 살아 있고 제한된 error text만 표시되는지 확인한다.
- Confidence: High
- Owner: Coder / TUI maintainer

### Pass 3: Security / High-risk boundary

### [A06-F013] TUI/public save helper가 canonical runtime root 경계를 우회할 수 있음

- Pass: Security
- Pattern: `SEC-004`, `SEC-007`
- Area: file path, save/load capability boundary
- Severity: Major
- Status: Confirmed
- Summary: headless는 `ArtifactStore`와 검증된 runtime-relative path를 사용하지만 TUI의 public save/load adapter는 임의 `Path`를 path-based compatibility helper에 직접 넘긴다. helper는 전체 입력을 검증하지 않고 `path.parent()`를 새 artifact root로 연다.
- Evidence:
  - 보안 계약 `spec.md:695-699,727-729`와 `DESIGN_DECISIONS.md:56-71`은 production artifact I/O를 열린 runtime root capability + relative path로 제한하고 path helper는 trusted test path로만 남기도록 한다.
  - `apps/aihack-tui/src/tui/mod.rs:89-103`의 `TuiClient` production impl은 `save::save_session_to_path`/`load_session_from_path`를 호출한다.
  - `crates/aihack-runtime/src/save.rs:169-176`은 path helper를 public으로 노출하고, `save.rs:228-234`의 `store_for_path`는 `path.file_name()`만 분리한 뒤 `ArtifactStore::open(parent)`를 호출한다. `../outside/file.json` 또는 임의 absolute parent를 runtime-relative rejection 없이 해당 parent를 root로 삼을 수 있다.
  - 반대로 headless `apps/aihack-headless/src/main.rs:45-56,187-200`은 먼저 runtime-root `ArtifactStore`를 열고 `validate_path` 후 relative path를 사용한다. `tests/headless_paths.rs` 6개와 `crates/aihack-runtime/tests/save_paths.rs` 1개는 이 canonical store 경계를 통과시킨다.
  - 현재 binary main은 `mod.rs:692-694`에서 자체 `tempdir/quick-save.json`을 생성하므로 즉시 untrusted user path를 받지는 않는다. 그러나 `TuiApp::save_to_path`, `load_from_path`와 runtime helper는 public workspace API다.
- Expected Basis: `spec.md:695-699,727-729`, ADR-0032의 production/trusted-test 경계, SEC-004 path/workspace separation.
- Actual: TUI adapter와 public helper의 호출 가능 경계가 headless와 달라 absolute/parent path를 임의 root로 취급할 수 있다. no-follow/atomic replace는 열린 root 내부에는 적용되지만 root 자체 선택을 제한하지 않는다.
- Impact: 향후 file picker, CLI option, plugin, 외부 adapter가 이 API에 사용자 입력을 연결하면 runtime 밖 임의 파일 read/write 경계가 생긴다. 현재 내부 tempdir 호출은 위험을 일부 줄이지만 public contract가 hard boundary를 보장하지 않는다.
- Suggested Action: TUI가 tempdir `ArtifactStore`를 열고 상대 `quick-save.json`만 사용하게 하며 path-based helper를 `pub(crate)`/test-only로 축소한다. absolute, `..`, root symlink 및 parent replacement 회귀를 TUI API에도 추가한다.
- Re-audit Method: TUI/public API에 absolute, `..`, root-outside symlink, hard-link destination을 주입해 fail-closed인지 확인하고, headless와 동일한 capability root source를 사용하는지 대조한다.
- Confidence: High for callable boundary; Medium for current exploitability because binary path is internally generated
- Owner: Architect / Security owner
- Notes: 이 finding은 parent file-security 관점과 독립적으로 TUI→runtime adapter call chain을 증거로 제시한다.

## 6. Uncertainties and Clarifications Needed

1. **UI locale 범위:** `AGENTS.md`의 5개 언어 i18n 규칙을 TUI/LLM presentation에도 적용할지, 아니면 README 문서만 다국어이고 TUI는 English-only인지 `spec.md`가 결정해야 한다. 현재는 어느 해석도 전체 구현과 일치하지 않는다.
2. **Title L / CharacterCreation Esc:** 화면 문구가 역사적 잔재인지 active contract인지 authority가 불명확하다. 구현을 추가할지 문구를 제거할지 제품 결정이 필요하다.
3. **Windows interactive gate:** R8/CI의 Windows release PASS가 Windows terminal restore/mouse/resize PASS를 의미하는지 문서가 정의하지 않는다. user goal에는 포함되므로 미검증을 자동으로 Excluded할 수 없다.
4. **TUI path helper trust:** `save_session_to_path`가 trusted test compatibility용인지 production TUI가 사용해도 되는지 ADR-0032와 현재 public API가 충돌한다.
5. **Rationale minimum:** `designs.md`의 1자 이상과 `spec.md`/code의 0자 이상 중 어느 계약이 canonical인지 확정해야 한다.
6. Linux `tmux` manual matrix와 실제 remote model/provider smoke는 각각 문서에 명시된 선택/비필수 범위이므로 이번 감사에서 실행하지 않았다. 이는 Windows interactive gate나 UI state findings를 해소하지 않는다.

## 7. Perspective Decision

**HOLD for this perspective.**

LLM adapter 자체의 핵심 격리 경계는 표적 테스트에서 양호했다. disabled/loopback restriction, response schema rejection, request/action bounds, C0/C1/ANSI rejection, timeout/unavailable classification, stale revision rejection, explicit `Y` approval, narrative/soft presentation-only 효과, bounded worker queue와 250ms shutdown grace를 다음 명령으로 재현했다.

그러나 A06-F001~F006, A06-F009~F010, A06-F012~F013의 Major 문제가 남아 있고, A06-F008/A06-F011은 사용자 핵심 Windows/platform 목표의 coverage gap이다. 따라서 LLM core hash 불변 테스트의 PASS를 TUI shipped-flow, accessibility, file-boundary 또는 cross-platform release PASS로 승격할 수 없다.

### Verified controls (finding이 아님)

- `tests/llm_revision_gate.rs` 9개: unknown request ID 보존, same-kind outstanding, stale revision, invalid action/confidence/rationale, validated command만 normal submit.
- `tests/llm_tui_integration.rs` 10개: disabled no-op, Unicode 240자 modal, status text, Y-only apply, soft verdict no submit, stale response, schema 2 rejection, unavailable fallback.
- `tests/llm_transport.rs` 22개: loopback endpoint, no redirect/proxy, body/request limits, bounded worker, timeout/unavailable, strict payload, 250ms grace.
- `tests/headless_paths.rs` 및 runtime `save_paths.rs`: canonical headless ArtifactStore의 parent escape, symlink/hard-link, atomic replace, Unix/Windows permission behavior.

### Accepted/explicitly deferred scope

- 실제 provider smoke는 `spec.md`가 R6 필수 조건이 아니라고 명시하므로 이번 판단에서 차단 사유로 삼지 않았다.
- Linux-only PTY evidence는 Windows 증거로 대체하지 않았고, 미확정 상태로 유지했다.
- 외부 배포·license/provenance·legacy body는 이 perspective 범위 밖으로 유지했다.

### Recommended remediation order

1. A06-F001~F003: state-aware input/Inventory/MorePrompt/New Run ordering과 transient reset.
2. A06-F004~F005, A06-F009: layout/accessibility/theme와 real mouse capture 연결.
3. A06-F010~F011: RAII terminal restore와 Windows ConPTY evidence.
4. A06-F006, A06-F012~F013: load/error state invalidation, TUI storage capability boundary.
5. A06-F007~F008: canonical timeout/rationale/i18n authority를 문서와 코드에 동기화.
