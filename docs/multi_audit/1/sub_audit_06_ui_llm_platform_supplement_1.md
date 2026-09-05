# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1, Supplement: 1
- Perspective: TUI·입력·접근성·LLM·플랫폼 통합 — Windows interactive coverage
- Supplement Of: `C:\LocalDev\rust\AIHack\docs\multi_audit\1\sub_audit_06_ui_llm_platform.md`
- User Goal: 원 범위 중 Windows interactive terminal/ConPTY, raw+alternate restore, mouse capture, 실제 Title/Creation/Playing/Inventory/GameOver 키 흐름의 coverage gap을 직접 검증하거나 남은 위험과 harness를 명확히 기록한다.
- Audit Basis: Standard-backed
- Standard Path: `C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md`; `C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md`
- Environment: Windows 11 build `10.0.26200.0`, PowerShell runtime, Rust target/debug binary, native `tmux 3.3.2` (`psmux`)
- Remote/paid provider call: 수행하지 않음. 로컬 loopback fixture만 사용함.

## 2. Assigned Scope

- Windows native PTY/console tool availability와 실제 TUI 실행
- raw mode/alternate screen 정상 종료 및 pending LLM worker shutdown
- mouse capture source와 실제 mouse event 전달 가능성
- Title→CharacterCreation→Playing→Inventory→GameOver→New Run의 실제 키 입력 coverage
- 59x23 minimum terminal 안내 및 clean exit
- 원본 보고서 `A06-F001`, `A06-F003`, `A06-F009`, `A06-F010`, `A06-F011`의 Windows/interactive 증거 보완

## 3. Excluded and Uninspected Scope

- 원본 보고서 수정, 다른 원본/통합 multi-audit 보고서 읽기
- `legacy_nethack_port_reference/` 본문, 외부 provider, 실제 모델 추론, 배포
- Computer Use UI automation: 지침상 Windows Terminal/PowerShell 같은 terminal application 자동화가 금지되어 사용하지 않음. 대신 `exec_command`의 직접 PTY와 native `tmux`만 사용함.
- 실제 Windows Terminal 창 또는 별도 ConPTY host: 현재 세션에서 targetable Windows Terminal window/ConPTY harness가 제공되지 않아 직접 확인하지 못함.
- initialization failure를 유도하는 destructive/OS-level console manipulation: terminal 상태를 강제로 망가뜨리지 않고 소스와 정상 경로 증거만 확인함.

## 4. Evidence Examined

### 문서/소스

- `designs.md:64-84,88-120,133-154,270-283,295-296`
- `spec.md:180-208,499-510,695-699,724-729`
- `docs/R6_MANUAL_MATRIX.md:1-55`, `BUILD_GUIDE.md:332-347`, `.github/workflows/ci.yml:13-46`
- `apps/aihack-tui/src/tui/mod.rs:686-813,1017-1089`, `input.rs:54-339`, `config.rs:7-27`, `render_panels.rs:316-397`
- `tests/ui_input_mapping.rs`, `tests/ui_screens.rs`, `tests/ui_runtime_smoke.rs`, `apps/aihack-tui/tests/tui_contract.rs`

### Windows tool inventory

Read-only `Get-Command` 결과:

- `wt.exe`: `C:\Users\temp\AppData\Local\Microsoft\WindowsApps\wt.exe`
- `conhost.exe`: `C:\Windows\system32\conhost.exe`
- `powershell.exe`, `pwsh.exe`, `cmd.exe`: 사용 가능
- `tmux.exe`: `C:\Users\temp\AppData\Local\Microsoft\WinGet\Packages\...\tmux.exe`, `tmux 3.3.2`
- `winpty`: 없음
- `wsl.exe`: 존재하지만 `wsl --list --verbose`에서 이 작업에 사용할 수 있는 정상 Linux distribution 항목을 확보하지 못함(출력은 Windows 인코딩으로 깨졌고 install hint만 관찰됨)
- 현재 `exec_command` console은 `[Console]::IsOutputRedirected=True`, `[Console]::IsInputRedirected=True`이며 `Console.WindowWidth/Height`를 읽을 수 없다. `WT_SESSION`도 설정되지 않아 현재 세션을 Windows Terminal 창으로 간주할 수 없다.

### 직접 실행 명령과 결과

- `\.\target\debug\aihack.exe --seed 42` with `exec_command(tty=true)`:
  - 시작 출력에 `ESC [?1049h` alternate screen 진입과 cursor hide가 관찰됨.
  - `q` 종료 출력에 `ESC [?1049l` alternate screen 복구와 cursor show가 관찰됨.
- native psmux: `tmux new-session -d -s <temporary> -x 80 -y 24 -- target\\debug\\aihack.exe --seed 42`:
  - Title 화면이 직접 capture됨.
  - `Enter, Enter, i` 후 capture에 `STATUS`, `INSPECT`, `COMMANDS`와 inventory lines가 남고 전용 Inventory overlay/focus는 없음.
  - 59x23 session에서 `terminal requires 60x24; resize or press Q/Esc to exit`가 capture되고 `q`로 session이 종료됨.
  - `.` 반복으로 `Killed by entity`, `Turn: 20`, `Score: 383`, `[N] New Run [Q] Quit`을 capture하고 `N` 후 `Press Enter to Start` Title로 복귀함.
- Windows native psmux + `scripts/r6_loopback_fixture.py` local fixture:
  - fixture `127.0.0.1`, delay 5000ms, `AIHACK_LLM_NARRATIVE_TIMEOUT_MS=10000`으로 시작.
  - `Enter, Enter, G` 후 `Narrative request ... [N] Dismiss`/`LLM: WAIT` capture.
  - pending 상태에서 `q` 입력 후 native tmux session disappearance를 322ms에 관찰함(`has_session_gone=True elapsed_ms=322`).
- `cargo test --workspace --locked --test ui_input_mapping --test ui_screens --test ui_runtime_smoke --test ui_layout --test llm_tui_integration && cargo test -p aihack-tui --locked --bin aihack --test tui_contract` — exit 0. UI/input/screen/LLM integration 관련 40개와 package TUI contract 6개 통과.

## 5. Coverage Matrix

| Surface | Direct Windows evidence | Coverage | Conclusion |
| --- | --- | --- | --- |
| Normal alternate-screen entry/exit | `tty=true` ANSI `?1049h`/`?1049l`, cursor hide/show | Covered for normal exit | Q path restores alternate screen visibly |
| Pending worker shutdown | native psmux + local 5s fixture, Q, 322ms | Covered for normal pending exit | bounded exit observed; not an init-failure proof |
| Raw mode restoration | crossterm source calls `disable_raw_mode`; parent shell remains usable after exit | Partially Covered | exact Windows console mode bits not introspected |
| Windows Terminal/ConPTY | `wt`/`conhost` binaries exist; current session is redirected PowerShell PTY, no targetable WT/ConPTY | Not Covered | dedicated harness required |
| Mouse capture | source has no `EnableMouseCapture`/`DisableMouseCapture`; no mouse tracking escape in app output | Not Covered / implementation gap | synthetic mapper tests do not prove event source |
| Title initial screen | native psmux capture | Covered | Title renders |
| Title→CharacterCreation one key | one injected Enter reaches final Playing in psmux; direct PTY output briefly contains Character Creation then Playing | Partially Covered | input injector likely emits CR/LF or duplicate Enter; stable single-step not proven |
| CharacterCreation→Playing one key | source/unit core tests only; no faithful native one-step capture | Partially Covered | requires in-process TuiApp harness or real console key event |
| Playing→Inventory | native psmux `i` capture | Covered for current behavior | confirms no overlay/focus transition |
| GameOver→New Run | native psmux capture and N | Covered | reaches Title and resets visible gameplay screen |
| 59x23 minimum | native psmux capture and Q | Covered | 안내 화면 and clean exit |

## 6. Findings

### [A06-S1-F001] Windows Terminal/ConPTY coverage remains unavailable

- Pass: Debug
- Pattern: `TEST-001`, `BUILD-001`
- Area: Windows interactive terminal, ConPTY, platform gate
- Severity: Major
- Status: Needs Clarification
- Summary: Windows tool binaries exist, but the available execution surface is a redirected PowerShell PTY/native psmux rather than a Windows Terminal or independently controlled ConPTY host.
- Evidence:
  - `wt.exe` and `conhost.exe` are discoverable, but `WT_SESSION` is absent and redirected console properties cannot expose a real window size.
  - `docs/R6_MANUAL_MATRIX.md:1-10` identifies only Linux/tmux; `.github/workflows/ci.yml:13-46` runs Windows cargo/checkpoint/release gates but no TUI ConPTY smoke.
  - Native psmux supports pane capture and key injection, but one Enter was not a reliable one-key state step (A06-S1-F002).
- Expected Basis: User explicitly requested Windows interactive terminal/ConPTY evidence; `designs.md:295-296` requires terminal restore-before-worker wait without narrowing to Linux.
- Actual: Normal Windows native PTY evidence is available, but Windows Terminal/ConPTY-specific behavior is not established.
- Impact: Windows-only differences in input encoding, raw mode, resize, mouse tracking, and alternate-screen restoration remain unresolved. This blocks a full platform PASS.
- Suggested Action: Add a Windows CI/local harness using `CreatePseudoConsole`/ConPTY or a maintained Windows PTY library, launch the actual binary, capture screen bytes, inject one key event at a time, and assert restore/exit. Alternatively revise the product gate to explicitly promise only native psmux/Linux PTY behavior.
- Re-audit Method: Run the same fixture matrix in Windows Terminal/ConPTY and Linux PTY; compare screen state, key count, resize, mouse event, alternate/raw restore, pending Q exit, and exit code.
- Confidence: High for coverage gap; no claim that `wt.exe` itself fails.
- Owner: Platform owner / CI maintainer

### [A06-S1-F002] Current Windows PTY key injection cannot isolate CharacterCreation

- Pass: Implementation / Debug
- Pattern: `IMP-001`, `TEST-001`
- Area: Title, CharacterCreation, Enter key semantics
- Severity: Major
- Status: Needs Clarification
- Summary: Native psmux and direct redirected PTY show a single injected Enter advancing to the final Playing frame; direct output can contain a transient Character Creation frame, indicating input duplication/CRLF translation in the harness. The real physical-key behavior is therefore not proven by this harness.
- Evidence:
  - Fresh native psmux 80x24 session starts at Title. After exactly one `tmux send-keys Enter`, pane capture already contains `STATUS`/`turn0`, not a stable Character Creation frame.
  - Direct `exec_command(tty=true)` with one `write_stdin` `\r` produced output containing `Character Creation` followed in the same refresh burst by `STATUS`, consistent with two Enter-like events or redraws before capture.
  - `tmux send-keys -l` literal CR/LF produced no transition, while key names `C-m`/`Enter` produced the transition; this demonstrates psmux key injection is not a transparent byte-level console harness.
  - `apps/aihack-tui/src/tui/mod.rs:1025-1037` has distinct Title/CharacterCreation mappings, but `apps/aihack-tui/tests/tui_contract.rs:19-25` checks only candidate creation and `tests/ui_screens.rs:11-32` submits directly to `GameSession`, not the live event loop.
- Expected Basis: `designs.md:64-84`, `spec.md:180-208` require observable Title→CharacterCreation→Playing state flow and keyboard behavior.
- Actual: The Windows harness cannot establish that one physical Enter produces exactly one transition or that CharacterCreation can be observed before the next key.
- Impact: A regression that skips, repeats, or mishandles CharacterCreation could pass the current tests and this psmux smoke. The supplement cannot certify the actual per-key Title/Creation contract.
- Suggested Action: Add an in-process `TuiApp` event harness that injects one `KeyCode::Enter` and asserts state after each event, plus a Windows ConPTY test that writes one canonical keyboard input event and captures a fresh frame before the next event.
- Re-audit Method: Assert Title + one Enter = CharacterCreation, second Enter = Playing, no turn advance; assert Creation Esc behavior separately. Repeat with physical Windows Terminal/ConPTY input.
- Confidence: High for harness limitation; Medium for physical-console implementation behavior
- Owner: Coder / Platform test owner

### [A06-S1-F003] Mouse capture is not enabled in the actual Windows terminal path

- Pass: Debug / Security boundary
- Pattern: `DBG-001`, `TEST-001`
- Area: crossterm mouse capture, CTA click source
- Severity: Major
- Status: Confirmed
- Summary: The binary consumes `Event::Mouse` only if the terminal emits it, but never enables crossterm mouse tracking. Native Windows capture therefore cannot establish the documented mouse path.
- Evidence:
  - `apps/aihack-tui/src/tui/config.rs:7-24` defaults `enable_mouse=true`; `apps/aihack-tui/src/tui/mod.rs:770-785` handles `Event::Mouse`.
  - `rg` over `apps/aihack-tui/src` finds no `EnableMouseCapture` or `DisableMouseCapture`. Terminal setup at `mod.rs:686-691` enables alternate/raw only.
  - Direct PTY startup output contains host focus/cursor modes but no application mouse-tracking sequence such as `?1000h`, `?1002h`, or `?1006h`.
  - `tests/ui_input_mapping.rs:58-101` and `tests/llm_tui_integration.rs:134-165` pass synthetic coordinates/strings directly to mapping functions; no physical/synthetic terminal mouse event reaches `event::read`.
- Expected Basis: `designs.md:145-147` requires mouse click to produce the same CTA ID as keyboard, and `UiRuntimeConfig.enable_mouse` implies a runtime capture boundary.
- Actual: No Windows mouse event source is enabled; the direct Windows harness cannot prove click/hover behavior because the app does not request tracking.
- Impact: Mouse users cannot reliably activate map, focus, Inventory, or LLM CTA actions. `enable_mouse=false` also has no corresponding disable lifecycle.
- Suggested Action: Add conditional `EnableMouseCapture` at startup and `DisableMouseCapture` in the same restore guard, then test mouse enabled/disabled in ConPTY/PTY with a real event injection harness.
- Re-audit Method: Verify terminal mode escape/API state, inject a map click and LLM footer click, and assert the exact same `UiCommandCandidate`/turn behavior as keyboard.
- Confidence: High
- Owner: Coder / Platform maintainer

### [A06-S1-F004] Initialization/error-path terminal restore is still unverified and structurally unguarded

- Pass: Debug
- Pattern: `DBG-001`, `TEST-001`
- Area: raw mode, alternate screen, failure cleanup, worker shutdown
- Severity: Major
- Status: Confirmed
- Summary: Normal Windows PTY exit and pending worker shutdown were directly observed, but setup/error paths remain outside an RAII restore guard.
- Evidence:
  - Direct `exec_command(tty=true)` observed alternate enter `?1049h` and Q exit `?1049l`/cursor show. Native pending fixture observed `LLM: WAIT` followed by session disappearance in 322ms while the fixture was delayed 5000ms.
  - `apps/aihack-tui/src/tui/mod.rs:686-691` performs `EnterAlternateScreen`, `enable_raw_mode`, `Terminal::new`, `GameSession::try_new`, and tempdir creation before the normal restore closure at `mod.rs:800-808`. Any early `?` can skip cleanup.
  - The restore closure itself uses `terminal::disable_raw_mode()?` before `LeaveAlternateScreen`; if disable fails, leave-alternate is skipped.
  - No Windows failure-injection harness is present in `.github/workflows/ci.yml` or `scripts/`.
- Expected Basis: `designs.md:295-296` requires terminal restore before worker grace wait; platform failure paths must not leave raw/alternate state.
- Actual: Normal path is good evidence, but raw mode bit restoration and early setup failures are not directly observed and are not exception-safe by structure.
- Impact: A startup/raw/terminal construction error could strand the user in alternate/raw mode even though the worker path is bounded.
- Suggested Action: Introduce a scope guard owning alternate/raw/mouse state, perform best-effort all-step restore while preserving the first error, then start the 250ms worker grace wait. Add Windows failure injection around raw enable, Terminal construction, draw/size, and restore.
- Re-audit Method: Force each setup/restore operation to fail in a controlled test double and verify leave-alternate, raw/canonical/echo mode, mouse disable, process exit, and bounded worker shutdown on Windows and Linux.
- Confidence: High
- Owner: Coder / Platform maintainer

### [A06-S1-F005] Direct Windows flow confirms missing Inventory overlay while GameOver/New Run normal path works

- Pass: Implementation
- Pattern: `IMP-001`, `TEST-001`
- Area: Playing→Inventory and GameOver→New Run
- Severity: Major
- Status: Confirmed
- Summary: Native Windows capture independently confirms that `i` does not open the documented Inventory overlay, while the normal GameOver→N→Title path does work.
- Evidence:
  - After native psmux `Enter, Enter, i`, capture shows `STATUS`, `INSPECT`, inventory rows (`a dagger`, `b food ration`, etc.), `focus Map`, and the normal `COMMANDS` footer. No Inventory overlay title or Inventory focus appears.
  - After 260 native psmux `.` inputs, capture shows `Killed by entity`, `Turn: 20`, `Score: 383`, `Seed: 42`, `[N] New Run [Q] Quit`; N capture shows `Press Enter to Start` Title.
  - This is consistent with `input.rs:90-92` mapping `i` to `ShowInventory`, `mod.rs:515-532` submitting it without UI state, and no `UiPanel::Inventory` construction.
- Expected Basis: `designs.md:64-84` requires `I → Inventory overlay → Esc → Playing`; GameOver N→Title is also specified at `designs.md:133-135`.
- Actual: GameOver/New Run is covered on the native Windows path; Inventory overlay/focus is not implemented and the direct capture makes the gap observable.
- Impact: Inventory user flow remains a Major TUI contract failure even though GameOver/New Run itself is not a Windows platform blocker.
- Suggested Action: Implement explicit Inventory overlay/focus and a state-aware Esc close path, then preserve the direct native smoke as a regression case.
- Re-audit Method: Native ConPTY/PTY capture after one `i` must show overlay/focus, Esc must return Playing without turn change, and GameOver N must still reach Title in one key.
- Confidence: High
- Owner: Coder / TUI maintainer

## 7. Uncertainties and Clarifications Needed

1. Native `tmux 3.3.2` is a useful Windows PTY smoke host but is not evidence for Windows Terminal/ConPTY. The product gate must state whether psmux is an accepted platform harness.
2. The single Enter duplication/CRLF behavior may belong to the psmux/exec harness rather than a physical Windows keyboard. It prevents a fair single-step Creation verdict; an in-process event harness plus ConPTY is required.
3. Exact Windows raw mode bits were not read from a real console handle. Alternate-screen escape restoration and bounded process exit were observed; raw restoration remains source-plus-normal-path evidence.
4. Mouse behavior cannot be tested until the binary enables mouse capture. Synthetic mapper tests are not a substitute for terminal event source coverage.
5. Initialization failure cleanup was not forced because doing so would require a controlled test seam; source structure is sufficient to keep the original risk open.

## 8. Perspective Decision

**HOLD / INCONCLUSIVE for full Windows interactive coverage.**

Direct evidence upgrades the normal-path status:

- Windows native psmux starts the TUI, enters alternate screen, renders Title, handles a Playing loop, reaches GameOver, accepts N to Title, shows the 59x23 minimum message, and exits cleanly.
- A pending loopback request remains visible as `LLM: WAIT`; Q exits the native psmux session in 322ms despite a 5000ms fixture delay, consistent with terminal-first restore and bounded worker shutdown.
- The direct Windows flow confirms the Inventory overlay gap.

The supplement cannot close the coverage gate because Windows Terminal/ConPTY is not exercised, mouse capture is absent, setup/error restore is not exception-safe, and one injected Enter cannot be isolated to CharacterCreation in the available PTY harness. The required next harness is a Windows ConPTY or in-process TuiApp event test with one-event/one-frame assertions, plus real mouse event injection after capture is implemented.

## 9. Coder Handoff

`C:\LocalDev\rust\AIHack\docs\multi_audit\1\sub_audit_06_ui_llm_platform_supplement_1.md`를 먼저 읽고, 원본 `sub_audit_06_ui_llm_platform.md`와 실제 코드에 각 coverage finding을 대조하세요. 계약 변경이 필요하면 `spec.md`/`designs.md`를 먼저 갱신하고, Windows ConPTY/in-process event harness 및 관련 테스트를 추가한 뒤 재감사 증거를 기록하세요.

