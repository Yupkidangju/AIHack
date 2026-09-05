# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 2
- Perspective: 플레이 중 입력 신뢰경계 및 오류 복원력
- User Goal: 게임 루프와 Title → 진행 → 종료 흐름이 NetHack 3.6.7에 준하는지, 그리고 키보드·마우스로 정상 플레이 가능한지 감사한다.
- Audit Basis: Standard-backed / Goal-driven
- Standard Path: `C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md`, `C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md`
- Baseline: `HEAD 899660167d59c4b06d27a59c0d75fcccda0cce33`

## 2. Assigned Scope

다음 production call path를 입력 신뢰경계와 복원성 관점에서 대조했다.

`crossterm Event → runtime_event_to_candidate → filter/상태 우선순위 → TuiApp::handle_candidate_owned → GameClient/GameSession`,
`TuiStorage::ephemeral → ArtifactStore save/load`, LLM pending/reset/late response, terminal setup/restore, 그리고 persisted blocking state의 direction/item/More 취소.

판정 기준은 `spec.md`의 상태 전이·save/replay 계약과 `designs.md`의 TUI/gesture 계약이다. 사용자의 재시작 후 이어하기 기대가 현재 문서의 실행별 임시 quick-save 계약과 일치하는지는 별도 분리했다.

## 3. Excluded and Uninspected Scope

- 소스, 테스트, 설정, 제품 문서와 새 probe/test 파일은 수정하지 않았다.
- `aihack-tui` package test와 Windows 실제 ConPTY 실행은 배정상 중복 실행하지 않았다. 따라서 physical key-hold, Windows Terminal GUI, 실제 마우스 버튼 반복 발생률은 직접 관찰하지 않았다.
- 배포/CI/전수 보안, 유료 provider, 이전 감사보고서와 다른 agent 결론은 범위에서 제외했다.
- 마우스 hold가 플랫폼별로 `Down`을 반복 생성하는지, 사용자가 원하는 double-click/hold semantics는 현재 문서에 닫혀 있지 않다.
- TUI process를 종료한 뒤 새 process에서 `L`로 복원하는 수동 재시작 실험은 실행하지 않았다.

## 4. Evidence Examined

### Documents

- `spec.md:184-212`: Title → CharacterCreation → Playing → blocking state/GameOver 전이와 transaction/turn 계약.
- `spec.md:528-532`: transition candidate의 Press-only/repeat-safe/quarantine 계약.
- `spec.md:618-661`, `spec.md:729-748`: SaveDataV1 semantic validation, save/load continuation, TUI quick-save capability 경계.
- `designs.md:67-107`: 화면 흐름, blocking state 우선순위, cancel/load 오류 복구.
- `designs.md:156-181`: new-run/reset, quick-save의 실행별 임시 directory, LLM late-response 폐기.
- `docs/compatibility/NH367-C009-save-continuation.md`, `NH367-C010-game-over.md`: save continuation과 GameOver의 observable contract.

### Source and call paths

- `apps/aihack-tui/src/tui/mod.rs:1540-1647,1649-1750`: state-aware keyboard dispatcher와 production event dispatcher.
- `apps/aihack-tui/src/tui/mod.rs:769-960`: candidate handler, save/load/new-run/quit/LLM 처리.
- `apps/aihack-tui/src/tui/mod.rs:205-220,525-543`: process-scoped quick-save store.
- `apps/aihack-tui/src/tui/mod.rs:1137-1310`: terminal lifecycle와 restore-before-worker-shutdown 순서.
- `apps/aihack-tui/src/tui/input.rs:151-290,293-335`: keyboard baseline, item shortcut, mouse mapping.
- `crates/aihack-runtime/src/session.rs:175-304,537-594`: core state transition과 `submit_quit`/turn semantics.
- `crates/aihack-runtime/src/observation.rs:173-304`: state별 legal action과 Playing action space.
- `crates/aihack-runtime/src/save.rs:151-243,342-417,484-772`: atomic ArtifactStore save/load 및 persisted-state validator.
- `crates/aihack-llm/src/service.rs:157-270`, `worker.rs:125-168`: bounded worker shutdown과 response lifecycle.

### Commands and results

- `cargo test --package aihack --locked --test ui_runtime_smoke --test llm_tui_integration --test save_load --test replay --test death`: 33 tests passed.
- `cargo test --package aihack --locked --test ui_input_mapping --test ui_screens`: 15 tests passed.
- `cargo test --package aihack --locked --test nethack_367_compat`: 10 tests passed, including C009 save continuation and C010 GameOver.
- `cargo test --package aihack-headless --locked --test headless_contract`: 6 tests passed.
- `cargo test --package aihack-runtime --locked`: 5 unit tests and 9 integration tests passed.

이 결과는 core save/RNG continuation, headless artifact rejection, LLM stale/reset, persisted blocking-state handling의 표적 계약이 현재 실행 환경에서 통과했음을 뜻한다. 아래 TUI production dispatcher의 미검증 경로를 통과시킨 증거로 확장하지 않았다.

## 5. Findings

### [A03-F001] Playing의 Q/Esc가 core GameOver와 저장 경계를 우회해 즉시 종료

- Pass: Implementation Compliance
- Pattern: IMP-001
- Area: quit lifecycle, unsaved-progress recovery, state transition
- Severity: Major
- Status: Confirmed
- Summary: Playing 상태의 Esc와 q는 `GameClient::submit(Quit)` 또는 종료 확인/GameOver 화면을 거치지 않고 `TuiApp`가 즉시 process loop를 빠져나가는 `Quit` candidate로 처리한다.
- Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:1627-1645`의 `runtime_key_to_candidate`는 Playing의 `Esc`를 `UiCommandCandidate::Quit`으로 반환한다.
  - `apps/aihack-tui/src/tui/mod.rs:1540-1579`는 Playing 문자를 `key_to_candidate`로 넘기며, q는 `apps/aihack-tui/src/tui/input.rs:151-195,220-229`의 baseline Quit으로 먼저 해석된다.
  - `apps/aihack-tui/src/tui/mod.rs:769-797,826`의 `handle_candidate_owned`는 `Quit`에서 `Ok(true)`만 반환하고 `self.client.submit(CommandIntent::Quit)`를 호출하지 않는다.
  - `apps/aihack-tui/src/tui/mod.rs:1287-1309`는 true를 받은 즉시 loop를 break한 뒤 terminal restore와 worker shutdown만 수행한다.
  - 반대로 `crates/aihack-runtime/src/session.rs:537-550`의 core `submit_quit`는 session state를 `GameOver`로 만들고 event를 남기는 별도 경로다. `tests/ui_screens.rs:34-40`은 이 core 경로만 검증한다.
  - `apps/aihack-tui/src/tui/mod.rs:205-220`의 `_directory: TempDir`가 TUI quick-save를 process-scoped로 소유한다.
- Expected Basis: `spec.md:184-195`의 `Playing --death/quit--> GameOver`, `docs/compatibility/NH367-C010-game-over.md`의 GameOver state contract, `designs.md:82-85`의 GameOver 후 N/Q 흐름. 현재 design은 Q를 GameOver 이후 Exit로 보여 주므로 Playing에서의 즉시 Q/Esc 종료 의도도 명시해야 한다.
- Actual: 생산 TUI는 Playing 중 Q/Esc에서 session state/final score/event를 갱신하지 않고 종료한다. 정상 Drop에서는 process-scoped quick-save도 제거되므로 저장하지 않은 진행과 TUI quick-save 모두 재시작에서 회수할 수 없다.
- Impact: 사용자가 모달 취소로 기대하기 쉬운 Esc 또는 일반 q 입력 한 번으로 진행 상태가 복구 불가능하게 사라질 수 있다. LLM pending 중에도 같은 경로가 적용되어 pending 결과를 폐기하거나 현재 run을 GameOver로 남길 기회가 없다. terminal은 복원되지만 게임 상태는 복원되지 않는다.
- Suggested Action: Q/Esc의 product semantics를 먼저 하나로 닫는다. Game contract를 따라야 한다면 Q/Esc를 explicit quit-confirmation 또는 `submit(Quit)` → GameOver 경로로 보내고 GameOver에서만 process exit를 허용한다. 즉시 종료가 의도라면 Playing의 Esc를 no-op/cancel로 분리하고 Q의 unsaved-progress 정책과 사용자-facing 경고를 명세에 기록하며 quick-save persistence 여부를 함께 결정한다.
- Re-audit Method: 실제 production dispatcher에서 Playing의 Q/Esc, LLM pending 중 Q/Esc, 저장 후 Q/Esc를 각각 한 번씩 주입한다. `run_state`, `turn`, `event_log`, save artifact 존재 여부, exit code와 terminal restore sequence를 확인하고 `spec.md`/`designs.md` 한 방향과 일치하는지 재검증한다.
- Owner: Coder / Architect
- Confidence: High (source call path와 core contrast가 직접 확인됨)
- Notes: 이는 core quit 구현이 없다는 finding이 아니라 TUI adapter가 core quit contract를 bypass하는 finding이다.

### [A03-F002] q의 Quaff shortcut은 baseline Quit에 가려져 도달 불가

- Pass: Implementation Compliance
- Pattern: IMP-002
- Area: keyboard command routing, accidental quit, item action reachability
- Severity: Major
- Status: Confirmed
- Summary: 입력 모듈은 q를 Quaff로 처리하려는 branch를 갖지만, 같은 함수의 baseline lookup이 항상 먼저 q→Quit을 반환하므로 Quaff candidate가 생성되지 않는다.
- Evidence:
  - `apps/aihack-tui/src/tui/input.rs:151-195`의 `keyboard_baseline`은 `('q', UiInputEvent::Quit)`을 선언한다.
  - `apps/aihack-tui/src/tui/input.rs:220-230`은 baseline을 먼저 찾고 `base.is_some()`이면 즉시 반환한다.
  - `apps/aihack-tui/src/tui/input.rs:242-256`의 후속 q branch만 `PotionHealing`을 찾아 `CommandIntent::Quaff`를 만들지만 baseline 때문에 실행되지 않는다.
  - `crates/aihack-runtime/src/observation.rs:248-263`은 potion이 있는 경우 Quaff를 legal action으로 만들고, `crates/aihack-runtime/src/session.rs:220-224`는 Quaff를 실제 turn command로 처리한다.
  - 현재 initial inventory에는 potion이 없지만 `crates/aihack-content/src/data/levels/main_1.toml:30-32`가 map에 healing potion을 spawn하고 `crates/aihack-runtime/src/bootstrap.rs:52-72`가 이를 production world에 반영한다. 따라서 기본 fixture에서도 pickup 이후 q collision이 도달 가능하다.
- Expected Basis: 코드에 존재하는 Quaff shortcut과 core `CommandIntent::Quaff` contract를 같은 keyboard candidate path에서 소비해야 한다. `AI_AUDIT_DOC_STANDARD.md`의 backward-sync 분류상 후속 Quaff branch는 현재 baseline에 의해 orphaned/inaccessible implementation이다.
- Actual: Playing에서 q는 항상 Quit candidate가 되고 Quaff는 keyboard direct path에서 생성되지 않는다. F001의 즉시 종료와 결합되어 potion 사용을 시도하는 입력이 process exit로 바뀔 수 있다.
- Impact: potion-bearing save/content를 플레이할 때 핵심 item action 하나가 direct keyboard에서 사라지고, 사용자가 q를 Quaff로 인식하는 경우 run을 잃는다. Inventory overlay의 letter 선택 경로는 별도라 이 충돌을 완전히 상쇄하지 못한다.
- Suggested Action: Quit과 Quaff의 key ownership을 분리한다. q를 Quaff로 유지하려면 process quit을 별도 확인키/메뉴로 이동하고, q를 Quit으로 유지하려면 Quaff branch와 문서/CTA를 제거하거나 명시적 item-selection sequence로 교체한다. 어느 쪽이든 potion-bearing fixture에서 candidate와 core turn effect를 직접 회귀 고정한다.
- Re-audit Method: potion이 포함된 validated save 또는 deterministic fixture에서 `runtime_event_to_candidate(KeyCode::Char('q'))`가 의도한 단일 candidate를 만드는지 확인하고, 그 candidate를 handler에 전달했을 때 process exit 없이 Quaff turn/state/hash 결과가 나오는지 검증한다. Quit 경로는 별도 key와 confirmation/GameOver로 확인한다.
- Owner: Coder / Architect
- Confidence: High (branch ordering과 call path가 결정적임)
- Notes: 기본 bootstrap이 potion을 spawn하지 않는 사실은 현재 smoke의 재현성을 낮출 뿐 collision 자체를 해소하지 않는다.

### [A03-F003] Mouse Down은 repeat/button quarantine 없이 매 이벤트를 command로 소비

- Pass: Debug / Engineering Quality
- Pattern: TEST-001
- Area: mouse gesture lifecycle, turn side effects
- Severity: Minor
- Status: Probable
- Summary: production dispatcher는 `MouseEventKind::Down(_)`을 버튼 종류와 무관하게 `MouseClick`으로 변환하고 keyboard repeat gate를 거치지 않는다. 따라서 동일 pointer gesture가 여러 Down 이벤트로 전달되면 legal map/CTA command가 이벤트마다 제출된다.
- Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:1682-1707`은 `MouseEventKind::Down(_)`을 `UiInputEvent::MouseClick`으로 만들고 `map_mouse_event_for_state` 결과를 즉시 반환한다. keyboard 전용 `filter_keyboard_candidate`는 이 return 이전에 호출되지 않는다.
  - `apps/aihack-tui/src/tui/input.rs:304-311`은 map 인접 tile click을 `CommandIntent::Move(direction)`으로 변환한다.
  - `apps/aihack-tui/src/tui/mod.rs:773-797`은 그 candidate마다 `client.submit`을 호출하고, `crates/aihack-runtime/src/session.rs:552-582`의 accepted move는 turn을 증가시킨다.
  - `Down(_)` wildcard라 Left뿐 아니라 Right/Middle도 같은 click/action path에 들어간다.
  - 실행한 `tests/ui_input_mapping.rs`는 단일 left-click만 검증했고, `apps/aihack-tui/tests/conpty_contract.rs`도 단일 mouse sequence만 보유한다. repeated Down, hold, Right/Middle negative case는 이 감사 실행에서 직접 검증하지 않았다.
- Expected Basis: `designs.md:173-181`은 mouse click이 keyboard와 같은 CTA ID를 만든다고만 정의하며 hold/double-click/button별 의미를 닫지 않았다. 사용자의 입력 신뢰경계 목표상 하나의 의도하지 않은 반복 gesture가 여러 accepted turn으로 증폭되지 않아야 한다는 안전 불변조건을 확인할 필요가 있다.
- Actual: 여러 Down이 실제 backend에서 발생하면 각 이벤트가 독립 click으로 간주된다. 한 번의 Down이 중복되었다고 판정하거나 버튼을 Left로 제한하는 보호가 없다. 다만 platform이 hold를 어떤 event sequence로 내보내는지는 직접 확인하지 못했으므로 반복 부작용은 Probable로 한정한다.
- Impact: 이동/Wait 등 turn-consuming CTA를 누르고 있는 동안 backend가 Down을 반복하면 의도보다 많은 turn이 생길 수 있다. 인접 hostile/위험 tile에서는 사용자 입력 하나의 오해가 사망으로 이어질 수 있다.
- Suggested Action: mouse contract를 먼저 정한다. 단일-click semantics라면 Left `Down` 한 번을 gesture token으로 소비하고 matching `Up` 전 중복 Down/Drag를 억제하며 Right/Middle은 no-op 또는 inspect로 분리한다. hold-repeat를 허용한다면 명시적 repeat rate와 action별 safe/unsafe 정책을 keyboard contract와 함께 정의하고 양 OS 실제 event stream을 고정한다.
- Re-audit Method: Windows ConPTY와 대표 Unix terminal에서 같은 좌표의 Left Down×2, Down/Up/Down, hold/Drag, Right/Middle을 주입한다. 각 sequence의 candidate 수, `turn` delta, core hash, modal overlay 차단을 기록하고 의도한 policy와 비교한다.
- Owner: Coder / Auditor
- Confidence: Medium (source behavior High, backend event generation Low)
- Notes: 단일 deliberate double-click을 두 command로 인정할지 여부는 명세 결정이며, 이 finding은 현재 문서가 그 경계를 말하지 않는 점까지 포함한다.

### [A03-F004] TUI quick-save는 spec상 process-scoped이지만 재시작 이어하기 요구는 미확정

- Pass: Implementation Compliance
- Pattern: SPEC-GAP-001
- Area: save lifecycle, process restart recovery
- Severity: Major (재시작 이어하기가 요구될 경우)
- Status: Needs Clarification
- Summary: 같은 process 안의 save/load와 RNG continuation은 통과하지만, TUI quick-save는 `TempDir` 아래에 있어 정상 종료 후 새 process에서 복원할 수 없다. 이는 현재 spec의 실행별 임시 저장 계약과는 일치하므로 즉시 Needs Fix로 단정하지 않고 사용자 기대와 분리한다.
- Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:205-220`은 `TuiStorage::ephemeral`에서 `tempfile::tempdir()`를 만들고 relative `quick-save.json`만 소유한다.
  - `apps/aihack-tui/src/tui/mod.rs:525-543`의 quick save/load는 그 process의 `ArtifactStore`만 사용한다.
  - `apps/aihack-tui/src/tui/mod.rs:1248-1250`은 process 시작마다 새 `TuiApp`/새 ephemeral store를 만든다.
  - `designs.md:156-160`, `spec.md:745-746`은 TUI quick-save를 실행별 temporary directory로 명시한다.
  - 실행한 `ui_runtime_smoke`, `save_load`, `replay`, C009 테스트는 save 직후 load와 direct continuation을 검증했지만 process 종료→재시작 복원은 검증하지 않았다.
- Expected Basis: 현재 authority만 따르면 same-process quick-save 및 atomic load 경계가 기대 상태다. NetHack-like user expectation 또는 사용자의 “진행 상태 손실” 질문이 process restart까지 포함한다면 durable user-scoped save와 명시적 crash/restart contract가 추가로 필요하다. 기대를 임의로 확정하지 않는다.
- Actual: same-process load 성공 시 `TuiApp::quick_load`가 transient와 late LLM request를 reset한다. 그러나 process가 종료되면 TempDir가 사라지고 다음 TUI의 Title `L`은 이전 quick-save를 찾을 수 없다.
- Impact: F001의 Q/Esc 즉시 종료와 결합하면 사용자는 저장했다고 생각한 TUI quick-save까지 재시작에서 잃을 수 있다. 반대로 process-local semantics가 의도라면 현재 구현은 spec 준수이며 durable persistence를 요구하는 수정은 scope expansion이다.
- Suggested Action: 제품 결정을 문서화한다. (a) process-local을 유지하면 화면/README에 “session quick-save; restart recovery 없음”을 명시하고 종료 전 경고를 검토한다. (b) 재시작 이어하기가 요구되면 사용자 전용 durable root, atomic replace/lock/crash recovery, restart integration test를 새 contract로 승인한 뒤 구현한다.
- Re-audit Method: 동일 seed에서 한 turn 진행→S→Q/정상 종료→새 process 시작→Title L을 실제로 실행한다. 요구 정책에 따라 save 존재/복원 state/hash와 error overlay를 확인하고 `spec.md`, `designs.md`, README의 설명을 동기화한다.
- Owner: Architect / Human
- Confidence: High (storage lifetime), Low on unstated product requirement
- Notes: 이 finding의 current-spec 판정은 `Verified`(same-process)이고, user-goal 판정은 `Needs Spec Clarification`이다.

## 6. Uncertainties and Clarifications Needed

- Playing에서 Q가 core GameOver를 만들어야 하는지, 아니면 GameOver 이후에만 허용되는 process exit인지 결정이 필요하다. Esc는 normal Playing에서 cancel/no-op인지 quit인지도 닫아야 한다.
- `q`를 Quit으로 예약할지 Quaff로 예약할지 결정이 필요하다. 두 command를 동시에 candidate로 둘 수는 없다.
- Mouse Left/Right/Middle, single-click/double-click/hold/drag의 turn semantics가 명세에 없다. F003은 이 미확정 경계를 이유로 Probable로 제한했다.
- `AwaitingDirection`, `AwaitingInventorySelection`, `MorePrompt`의 persisted-state handling은 source와 표적 core/UI test에서 확인했지만, `Playing`에서 이 상태들을 만드는 production producer는 확인하지 못했다. `crates/aihack-runtime/src/session.rs:203-233`의 Playing dispatch는 direct command만 처리하며, 저장/테스트 외 production assignment가 없다. 따라서 live modal entry와 live cancel까지 검증되었다고 확장하지 않는다.
- LLM pending/reset/late response, save/load 오류의 typed overlay, terminal restore-before-worker shutdown은 source와 실행한 표적 테스트 범위에서 추가 finding을 만들지 않았다. 이는 process restart persistence 또는 physical mouse hold까지 PASS했다는 뜻은 아니다.

## 7. Perspective Decision

`HOLD` — A03-F001과 A03-F002는 현재 production TUI에서 확인된 사용자-visible quit/state-loss 및 command-routing 문제로, 명시적 Accepted Risk 없이는 입력 신뢰경계 관점의 PASS를 줄 수 없다. A03-F003은 실제 backend event stream 재검증이 필요하고, A03-F004는 현재 spec 준수와 재시작 이어하기 요구를 먼저 분리 결정해야 한다.
