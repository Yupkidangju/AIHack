# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 2
- Perspective: runtime_loop (시작·진행·종료 경로, 명령 소비, 완주 증거, 저장/복원 무결성)
- User Goal: 게임 루프와 시작→진행→끝이 NetHack 3.6.7에 준하는지, 키보드와 마우스로 모든 기능을 정상 플레이할 수 있는지 감사
- Audit Basis: Standard-backed / Goal-driven
- Standard Path: `C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md`
- Report Contract: `C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md`
- Repository HEAD: `899660167d59c4b06d27a59c0d75fcccda0cce33`

## 2. Assigned Scope

- `GameSession::try_new`/`try_new_for_playing`의 Title→CharacterCreation→Playing 시작 경로
- `GameSession::submit`의 accepted turn, rejection, monster/death, GameOver 경로
- `RunState` 중간 상태, stairs/level 전환, quit/new-run 경로
- headless `run_to_turn*`/replay target semantics와 실제 명령 소비
- 저장/복원 후 snapshot·RNG·명령 연속성 및 transaction rollback
- 기본 월드 bootstrap과 long-run/release-candidate fixture의 차이
- 키보드 mapping이 runtime command를 실제로 소비하는지에 대한 런타임 관점 확인
- 현재 `spec.md` 및 `designs.md`의 상태 전이·headless·NH367-C005/C009/C010 계약 대비

## 3. Excluded and Uninspected Scope

- 배포 산출물, CI workflow, 전수 의존성 감사, provenance/license 감사
- 전체 workspace test suite 및 수동 장시간/실제 사용자 physical key-hold 검증
- 다른 감사보고서, 부모 에이전트 결론 및 다른 에이전트 결과는 참조하지 않음
- 소스·테스트·설정·제품 문서는 수정하지 않았고, 신규 probe/test 파일은 생성하지 않음
- GUI 터미널 자체의 동작은 제외하고, Windows ConPTY가 증명하는 byte-level 범위만 확인

## 4. Evidence Examined

### 문서·코드

- `spec.md:184-213` 상태 전이와 accepted-turn pipeline, `spec.md:627-704` SaveDataV1/headless contract, `spec.md:713-728` NH367 compatibility scope
- `designs.md:65-107` 화면 흐름 및 persisted blocking state, `designs.md:156-160` new-run/load reset, `designs.md:333-334` ConPTY 범위
- `crates/aihack-runtime/src/session.rs:45-104,158-303,537-598` 생성·상태 dispatch·명령 처리·quit·turn commit
- `crates/aihack-runtime/src/observation.rs:173-303` run-state/action-space projection
- `crates/aihack-runtime/src/systems/stairs.rs`, `systems/death.rs`, `src/save.rs` 및 `crates/aihack-runtime/src/save.rs` 저장/복원 경계
- `apps/aihack-headless/src/lib.rs:80-157,182-238` replay 및 target runner, `apps/aihack-headless/src/main.rs:80-178` production CLI wiring
- `apps/aihack-tui/src/tui/mod.rs:769-850,1223-1301,1540-1749` candidate handler·실제 loop·state-aware input
- `apps/aihack-tui/src/tui/input.rs:151-285` keyboard baseline와 item/action mapping
- `crates/aihack-runtime/src/world.rs:86-105`, `tests/world_bootstrap.rs` fixture/bootstrap 동등성

### 실행한 표적 명령과 결과

| Command | Result |
| --- | --- |
| `cargo test -p aihack --locked --test ui_screens --test stairs --test world_bootstrap --test long_run --test release_candidate` | 25 passed, 0 failed. long-run 7, release-candidate 1, stairs 7, ui-screens 9, world-bootstrap 1 |
| `cargo test -p aihack --locked --test save_load --test replay --test death --test action_space --test nethack_367_compat --test ui_input_mapping --test ui_runtime_smoke` | 39 passed, 0 failed |
| `cargo test -p aihack-tui --locked --test tui_contract --test conpty_contract` | 22 passed, 0 failed (TUI 20, Windows ConPTY 2) |
| `cargo test -p aihack --locked --test golden_phase8_rules --test monster_ai --test transaction` | 39 passed, 0 failed |
| `cargo test -p aihack --locked --test headless_paths --test save_validation --test world_invariants` | 24 passed, 0 failed |
| `cargo test -p aihack-headless --locked --test headless_contract` | 6 passed, 0 failed |

추가로 저장/복원 target 경계를 확인하기 위해 프로젝트 밖의 일회성 임시 runtime에서 실제 `target\debug\aihack-headless.exe`를 실행했다. `--seed 42 --turns 2 --policy wait-v1 --save saves/higher.json`은 exit 0, `accepted_turns=2`였고, 이어 `--load saves/higher.json --turns 1 --policy wait-v1`도 exit 0으로 `accepted_turns=0`, `submitted_commands=0`, `final_state=Playing`, 동일 final hash를 출력했다. 임시 산출물은 확인 후 제거했다.

## 5. Findings

### [A02-F001] load된 turn이 target보다 큰 경우 성공 no-op으로 보고됨

- Pass: Debug / Implementation
- Pattern: `TEST-001`, `IMP-003`
- Area: headless target semantics, save continuation, 종료 판정
- Severity: Major
- Status: Confirmed
- Summary: load session의 현재 turn이 requested target보다 이미 큰 경우에도 runner와 CLI가 오류가 아닌 성공 report를 반환한다.
- Evidence:
  - `apps/aihack-headless/src/lib.rs:187-225`의 `run_to_turn_with_trace`는 `while session.revision().turn < target_turn`만 실행하고 현재 turn 초과를 검사하지 않는다.
  - 같은 파일 `:88-91,139-155`의 `run_replay_to_turn`도 현재 turn이 target 이상이면 replay를 소비하지 않고 성공 report를 만든다.
  - `apps/aihack-headless/src/main.rs:143-158`은 runner의 `Ok`를 그대로 성공 경로로 처리한다.
  - `BUILD_GUIDE.md:263`은 load turn이 target보다 크면 exit code 2라고 명시하고, `:291-297`은 exit 0을 target 달성으로 정의한다.
  - 실제 실행: save turn 2를 load한 뒤 `--turns 1`을 요청했는데 exit 0, `accepted_turns=0`, `submitted_commands=0`이었다.
- Expected Basis: `BUILD_GUIDE.md`의 명시적 load/target 계약, `spec.md:698-704`의 accepted-turn target 및 failure report 계약, 사용자 목표의 실제 진행·종료 판정
- Expected: current turn > target이면 typed CLI/runner 오류(문서 계약상 exit 2)를 반환하고 성공 report 또는 success stdout를 만들지 않아야 한다.
- Actual: 현재 상태를 그대로 clone/commit하고 requested target보다 높은 turn을 성공으로 보고한다. replay path도 동일한 guard 누락을 가진다.
- Impact: 저장된 진행을 더 이른 target으로 재개할 때 게임이 한 턴도 진행되지 않았는데 성공처럼 보인다. 자동 검증과 호출자는 `accepted_turns=0` 성공을 정상 완료로 오인할 수 있다.
- Suggested Action: runner 진입부에서 `current_turn > target_turn`을 명시적으로 거절하고 CLI exit 2/failure semantics를 고정한다. wait/replay 양쪽 API와 실제 binary에 lower/equal/higher target 회귀를 추가한다.
- Re-audit Method: turn 0/target, turn == target, turn > target의 `--load`와 replay를 각각 실행해 exit code, report 생성 여부, 원 session 보존을 확인한다.
- Confidence: 0.99
- Notes: 이는 기능을 전혀 실행하지 않는 성공 판정 버그이며, 불충분한 long-run 테스트 문제와 분리된 실제 CLI contract 위반이다.

### [A02-F002] 선언된 Awaiting/MorePrompt 상태가 실제 gameplay 경로에서 생성되지 않음

- Pass: Implementation / Debug
- Pattern: `IMP-002`, `DBG-002`, `TEST-001`
- Area: session state machine, directional/item command consumption, message flow
- Severity: Major
- Status: Confirmed
- Summary: `RunState::AwaitingDirection`, `RunState::AwaitingInventorySelection`, `RunState::MorePrompt` handler는 존재하지만 production `Playing` submit/event path가 이 상태로 전이하지 않는다. 표적 테스트는 저장 fixture로 상태를 주입해 handler만 확인한다.
- Evidence:
  - `crates/aihack-runtime/src/session.rs:203-233`의 `submit_in_playing`은 명령을 즉시 `submit_open/close/kick/throw/zap/...`으로 전달하며 세 중간 상태를 설정하지 않는다.
  - 같은 파일에서 `:251`과 `:283`의 assignment는 이미 해당 Awaiting 상태인 handler가 invalid selection 뒤 같은 상태로 복귀하는 경우뿐이다. `MorePrompt` assignment는 production source에 없다.
  - `tests/ui_screens.rs:73-135`와 `tests/support/session_builder.rs`는 `run_state`를 SaveData fixture에 직접 설정해 `MorePrompt`/Awaiting handler를 검사한다.
  - `apps/aihack-tui/src/tui/input.rs:181-186,261-279`는 Open/Close/Kick/Throw/Zap을 각각 고정 `East` 방향 명령으로 만들며, 실제 two-step direction entry를 시작하지 않는다.
- Expected Basis: `spec.md:187-192`의 `Playing --needs direction/item/message overflow-->` 상태 전이, `designs.md:101-104`의 persisted blocking state 의미와 취소 계약, 사용자 목표의 실제 명령 소비
- Expected: 선언된 blocking 상태가 실제 명령/메시지 조건에서 도달 가능하고, 그 상태의 방향·item 선택·ack/cancel 입력이 production flow를 완성해야 한다. direct one-key semantics를 의도했다면 상태 다이어그램과 shipped 범위를 그에 맞게 닫아야 한다.
- Actual: 정상 새 게임에서 이 세 상태로 갈 수 없고, MorePrompt도 overflow producer가 없다. 따라서 관련 테스트는 persisted fixture의 상태 handler만 증명한다. 일부 command는 API에서 직접 실행되지만 UI는 East 고정 또는 다른 직접 mapping에 의존한다.
- Impact: 상태 전이 문서가 주장하는 경로와 실제 start→progress 경로가 다르며, 임의 방향/아이템을 고르는 blocking interaction과 message pagination의 production 도달성이 확인되지 않는다. 테스트가 전이 handler의 존재를 실제 완주 증거처럼 보이게 한다.
- Suggested Action: direct command model과 two-step model 중 하나를 명세에서 확정한다. two-step을 유지하면 action initiation→Awaiting→selection/cancel→commit을 실제 `submit`/TUI 경로에 연결하고, MorePrompt producer와 overflow 기준을 추가한다. direct model이면 unused state와 테스트를 shipped contract에서 분리한다.
- Re-audit Method: fresh `try_new(42)`에서 Title→Playing 후 Open/Close/Kick/Throw/Zap 및 긴 message sequence를 실제 key/mouse candidate로 수행해 각 상태 도달과 복귀를 trace한다.
- Confidence: 0.97
- Notes: direct action 자체가 전부 고장났다고 단정하지 않는다. 확인된 문제는 spec에 선언된 중간 상태의 production reachability와 테스트 증거 수준이다.

### [A02-F003] `legal_actions`가 거절되는 명령을 노출해 paralysis에서 survival runner가 dead end가 됨

- Pass: Implementation / Debug
- Pattern: `IMP-001`, `DBG-002`, `TEST-001`
- Area: action-space truth, status gating, headless policy progress
- Severity: Major
- Status: Confirmed
- Summary: action-space가 submit의 실제 legality와 일치하지 않는다. 특히 지원되는 paralysis 상태에서 survival policy는 거절될 Move 하나만 반복해 `NoAcceptedAction`으로 끝날 수 있다.
- Evidence:
  - `crates/aihack-runtime/src/observation.rs:229-235`는 `Pray`를 cooldown과 무관하게 항상 추가하고, `:287-299`는 인접 문이 없어도 모든 방향의 `Kick`을 추가한다.
  - `crates/aihack-runtime/src/session.rs:203-207`은 `paralysis_turns > 0`일 때 Wait/Quit 외 모든 명령을 거절한다. `:214`의 Kick은 문이 없으면 `NoDoor`를 반환하고, `submit_pray`도 cooldown이면 거절한다.
  - `apps/aihack-headless/src/lib.rs:267-292`의 survival policy는 observation action-space에서 첫 통과 방향 하나만 반환한다. `:196-224` runner는 그 후보를 최대 16회 재시도한 뒤 Wait로 바꾸지 않고 `NoAcceptedAction`으로 실패한다.
  - `tests/golden_phase8_rules.rs:203-223`은 지원 콘텐츠인 FloatingEye passive가 실제 명령 후 `paralysis_turns=1`을 만드는 것을 통과시킨다. 반면 required-seed long-run은 embedded main levels에 FloatingEye가 없어 이 경계를 실행하지 않는다.
- Expected Basis: `spec.md:699-704`의 survival-v1 규칙과 accepted-turn 성공 계약, `Observation.legal_actions`/`ActionSpace`의 명칭 및 `GameSession::submit` legality, 지원된 status/monster passive
- Expected: action-space에는 현재 submit이 수용할 수 있는 command만 포함하거나 status-gated 명령을 제외해야 한다. paralysis 상태의 survival policy는 Wait로 진행 가능해야 하며, no-door/cooldown 명령은 legal로 표시되지 않아야 한다.
- Actual: 거절될 Move/Pray/Kick이 legal로 노출된다. FloatingEye passive 직후에는 first passable Move가 action-space에 남아 policy가 그 명령 하나만 16회 제출한다. 한편 사용자가 직접 `.`/Wait를 누르면 core는 진행할 수 있다.
- Impact: supported save/status 또는 passive fixture에서 deterministic headless completion이 실패하며, action-space를 신뢰하는 LLM/입력 adapter가 반복 거절에 빠진다. 현재 기본 월드의 1000-turn 통과는 이 status branch를 커버하지 않는다.
- Suggested Action: `legal_actions`를 submit guard와 단일 함수로 공유하고 cooldown/no-door/paralysis를 반영한다. survival policy에는 legal Wait fallback을 포함하고, paralysis·cooldown·empty-door 상태에서 one-target progress 회귀를 추가한다.
- Re-audit Method: validated save 또는 test-only fixture로 paralysis=1, prayer cooldown>0, 문 없는 위치를 만들고 observation action-space와 `survival-v1` target 1 실행을 대조한다. Wait가 accepted turn을 만들고 거절 명령이 action-space에서 빠지는지 확인한다.
- Confidence: 0.98
- Notes: required seeds의 long-run PASS를 반증하는 finding이 아니라, 현재 기본 월드 밖이지만 registry와 테스트가 지원하는 status 경계의 미검증/실패 경로다.

### [A02-F004] TUI Quit은 core GameOver를 거치지 않고 process loop를 바로 종료함

- Pass: Implementation / Debug
- Pattern: `IMP-001`, `DBG-001`
- Area: voluntary quit, end-state semantics, score persistence
- Severity: Minor
- Status: Needs Clarification
- Summary: core의 `CommandIntent::Quit`은 GameOver sentinel과 score를 만들지만, 실제 TUI의 `UiCommandCandidate::Quit`은 `GameSession::submit(Quit)`을 호출하지 않고 즉시 loop 종료를 지시한다.
- Evidence:
  - `crates/aihack-runtime/src/session.rs:209-212,537-549`는 core Quit을 GameOver(cause sentinel, final_score)로 만들고 `CommandRejected("quit requested")` event를 append한다.
  - `apps/aihack-tui/src/tui/mod.rs:769-827`에서 `UiCommandCandidate::Quit`은 `Ok(true)`만 반환한다. `:1288-1295`의 production loop는 true를 받으면 바로 break한다.
  - `spec.md:191-192`는 `Playing --death/quit--> GameOver`를 적지만, `designs.md:82`는 GameOver 이후 `Q -> Exit`을 UI 흐름으로 적는다.
  - `apps/aihack-tui/tests/conpty_contract.rs:91-121,207-224`의 실제 q 검증은 child clean exit와 terminal restore만 확인하며 GameSession의 GameOver/event/score를 검사하지 않는다.
- Expected Basis: 위 spec/design 두 문서의 상충하는 종료 semantics와 사용자 목표의 quit/new-run 경로
- Expected: 제품이 Q를 즉시 process exit로 의도한다면 그 예외와 score/state 보존 범위를 명시해야 한다. spec의 quit→GameOver가 shipped 경로라면 TUI도 core Quit을 소비해 GameOver 화면/저장을 거쳐야 한다.
- Actual: TUI Q/Esc는 core final score를 계산하지 않고 process를 종료한다. core Quit은 별도의 direct API 경로에서만 관찰된다.
- Impact: voluntary quit을 GameOver/score/replay state로 재개할 수 없고, TUI의 quit semantics를 기준으로 end-to-end GameOver를 주장할 수 없다. 다만 designs의 즉시 Exit 의도라면 기능적 결함이 아닐 수 있다.
- Suggested Action: Q/Esc를 process exit와 run-state quit 중 하나로 명확히 확정하고 문서·candidate·test를 일치시킨다. 두 semantics가 모두 필요하면 UI exit와 core quit을 별도 명령으로 명명한다.
- Re-audit Method: 실제 TUI Playing에서 q/Esc를 보내 GameOver 화면, final score/event, terminal exit 중 명세가 정한 결과를 확인하고 ConPTY assertion을 그 결과에 맞춘다.
- Confidence: 0.97
- Notes: 문서 충돌 때문에 현재는 Needs Clarification으로 분류한다. 사망에 의한 GameOver core 경로 자체는 표적 death tests에서 통과했다.

### [A02-F005] long-run/release 테스트는 library fixture 완주를 증명하지만 production binary E2E는 닫지 않음

- Pass: Debug / Implementation
- Pattern: `TEST-001`, `IMP-003`, `DBG-001`
- Area: completion evidence, fixture/production boundary
- Severity: Minor
- Status: Needs Clarification
- Summary: 1000-turn PASS는 결정론적 runner와 기본 fixture에 대한 강한 증거지만, 실제 headless binary startup/CLI와 TUI의 death→GameOver→NewRun까지 한 흐름으로 증명하지 않는다.
- Evidence:
  - `tests/long_run.rs:33-72`와 `tests/release_candidate.rs:7-15`은 `GameSession::new_for_playing(seed)`와 in-process `run_to_turn`을 사용한다.
  - `crates/aihack-runtime/src/session.rs:86-103`의 test-facing `new_for_playing`은 `GameWorld::fixture_phase4()`를 통해 infallible fixture 경계를 사용한다. `crates/aihack-runtime/src/world.rs:94-105`와 `tests/world_bootstrap.rs`로 이 fixture의 embedded content 값은 `try_fixture_phase5`와 동등함을 확인했다.
  - 실제 production 경로는 `apps/aihack-headless/src/main.rs:83-89`의 `try_new_for_playing`, TUI는 `apps/aihack-tui/src/tui/mod.rs:1248-1249`의 `try_new`이다. 따라서 월드 데이터 차이는 확인되지 않았지만 fallible bootstrap·CLI argument·process/exit 경계는 long-run tests가 우회한다.
  - Windows ConPTY test는 `apps/aihack-tui/tests/conpty_contract.rs`에서 Enter→Creation→Playing, playing mouse, Inventory/Esc, Wait, q clean exit을 확인하지만 stairs navigation, real death/GameOver/NewRun, TUI save/load는 수행하지 않는다.
  - `tests/ui_screens.rs:35-66`와 `tests/stairs.rs`의 death/stairs 사례는 player 위치를 `SessionBuilder`로 재배치한 fixture이고, `tests/ui_runtime_smoke.rs`의 save/load는 in-process TuiApp bridge다.
- Expected Basis: 사용자 목표의 시작→진행→끝 완주성, `spec.md:39-70`의 SC-TEST-01/02, `designs.md:333-334`가 정의한 실제 ConPTY 범위
- Expected: library/fixture evidence와 별도로 production binary의 startup, target completion, stairs/death/new-run 및 필요한 save/load를 실제 adapter 경계에서 증명하거나, 검증 범위를 그 수준으로 한정해야 한다.
- Actual: embedded 기본 월드의 내용 동등성·library runner 1000-turn·scoped NH367 C001-C010·save/load hash/RNG는 통과했다. 그러나 하나의 production E2E start→progress→end trace와 binary 1000-turn success evidence는 없다.
- Impact: 현재 evidence만으로는 “게임 core가 결정적으로 1000턴 진행된다”는 scoped claim은 가능하지만, “실제 TUI/headless 사용자가 시작해서 stairs/death/new-run/quit까지 완주한다”는 전체 목표 PASS는 할 수 없다.
- Suggested Action: production headless binary를 clean temp runtime에서 target 1000으로 실행하고 stdout/report/exit를 assert한다. TUI ConPTY에는 deterministic death fixture 또는 reachable command sequence로 GameOver→N→Title→Creation→Playing을 추가하되, fixture-only 결과와 실제 기본 월드 결과를 별도 표기한다.
- Re-audit Method: `cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1`에 준하는 실제 binary 실행과 ConPTY state trace를 재실행하고, failure path는 exit/report semantics까지 검사한다.
- Confidence: 0.94
- Notes: 이는 기능 버그 단정이 아니라 completion evidence gap이다. `tests/world_bootstrap.rs` 때문에 기본 fixture가 production content와 다르다고 주장하지 않는다.

### [A02-F006] Playing에서 lowercase `q`의 Quaff mapping은 Quit에 가려짐

- Pass: Implementation
- Pattern: `IMP-001`, `TEST-001`
- Area: keyboard command consumption, item action reachability
- Severity: Minor
- Status: Confirmed
- Summary: `key_to_candidate`가 keyboard baseline의 lowercase `q` Quit을 먼저 반환하므로 뒤의 potion Quaff branch는 실제 Playing key path에서 도달하지 않는다.
- Evidence:
  - `apps/aihack-tui/src/tui/input.rs:181-194`에서 baseline은 `q -> UiInputEvent::Quit`이다.
  - 같은 파일 `:220-231`에서 baseline hit가 먼저 반환되고, `:247-248`의 `q -> PotionHealing -> Quaff` branch는 그 뒤에 있어 shadowed다.
  - `apps/aihack-tui/src/tui/mod.rs:1578`은 Playing에서 `key_to_candidate`를 사용한다. 따라서 lowercase q는 Quit candidate이고, uppercase Q는 Playing baseline/branch에 없어 no-op이다.
  - `tests/ui_input_mapping.rs`는 q 충돌이나 potion의 direct keyboard consumption을 assertion하지 않으며, inventory letter를 통한 item action만 별도 경로로 남아 있다.
- Expected Basis: 사용자 목표의 모든 기능 키보드 소비 및 코드가 선언한 Quaff mapping; inventory overlay는 보조 경로이지 shadowed direct mapping의 대체 근거가 아니다.
- Expected: Quit과 Quaff의 키 의미가 충돌하지 않고, healing potion을 명시된 keyboard flow에서 직접 선택할 수 있어야 한다.
- Actual: potion은 Inventory를 열고 item letter를 고르는 경로로는 사용할 수 있으나, direct lowercase q mapping은 항상 Quit으로 해석된다. Playing의 uppercase Q는 종료하지 않는다.
- Impact: potion action이 advertised direct key에서 dead end이며, 실수로 q를 눌러 run을 종료할 수 있다. 기능 전체가 막히지는 않지만 keyboard parity와 NetHack-like command expectation이 흔들린다.
- Suggested Action: Quit을 uppercase Q/별도 control로 고정하거나 Quaff를 다른 명확한 key로 분리하고, key→candidate→submit의 direct potion test와 actual TUI trace를 추가한다.
- Re-audit Method: potion을 inventory에 둔 Playing session에서 lowercase/uppercase quit·quaff candidates와 resulting core state/turn을 각각 확인한다.
- Confidence: 0.99
- Notes: mouse/inventory-letter 보조 경로가 존재하므로 Severity를 Minor로 제한했다.

## 6. Uncertainties and Clarifications Needed

- NetHack 3.6.7 “준함”은 현재 `NH367-C001..C010`의 행동 계약으로만 측정 가능하다. 표적 `nethack_367_compat` 10개와 stairs/save/death 관련 검사는 모두 통과했지만, 전체 NetHack의 character creation, 승리/Ascension, monster/item 집합을 주장할 수 있는 범위는 아니다.
- CharacterCreation은 현재 fixed Adventurer 확인 화면이다(`session.rs:189-200`, `render_panels.rs:346-359`). 현재 spec/design은 고정 플레이어를 정의하므로 full race/class/name customization의 필요 여부는 별도 제품 결정이다.
- `RunState`에는 Victory/Ascension 상태가 없고 정상 종료는 spec의 GameOver/quit 흐름으로만 표현된다. 사용자 목표의 “끝”이 death/quit을 뜻하는지, 성공적인 dungeon completion까지 포함하는지 명세 확인이 필요하다.
- TUI mouse dispatcher는 Title/CharacterCreation/GameOver에서 mouse candidate를 만들지 않는다(`apps/aihack-tui/src/tui/mod.rs:1797-1819`). 본 보고서는 runtime command consumption을 중심으로 했고, mouse CTA completeness/접근성의 최종 판정은 이 관점에서 확정하지 않는다.
- 저장/복원 무결성은 현재 evidence에서 양호하다. `save_load`, `replay`, `headless_paths`, `save_validation`, `world_invariants`, `transaction` 및 NH367-C009가 hash/RNG/atomic path/semantic validation을 통과했다. 다만 F001은 저장 데이터가 아니라 loaded target의 runner 판정 오류다.

## 7. Perspective Decision

- User Goal Decision: `HOLD — PARTIALLY VERIFIED`
  - 확인된 범위: 실제 TUI의 Title→CharacterCreation→Playing one-key path, basic playing mouse/Inventory/Esc/Wait/clean exit, core combat death→GameOver, stairs roundtrip, save/load hash·RNG continuation, scoped NH367-C001..C010, required seed fixture long-run 1000 accepted turns.
  - HOLD 사유: F001의 실제 headless success 판정 오류와 F002/F003의 production reachability/action-space 문제. F004/F005는 quit semantics 및 production E2E completion 증거 공백이다.
- Current Spec Decision: `HOLD`
  - spec의 load target exit 계약(F001), 상태 전이 다이어그램(F002), action-space/accepted-turn semantics(F003)가 현재 코드·테스트와 닫히지 않는다.
  - F004는 `spec.md`와 `designs.md`가 서로 다른 quit semantics를 제시하므로 요구사항 확정 전에는 기능 결함으로 확정하지 않는다.
- NetHack Decision: `PASS WITHIN SCOPED NH367-C001..C010 EVIDENCE ONLY`
  - 이는 full NetHack 3.6.7 parity나 production E2E 완료를 의미하지 않는다.
- Re-audit gate: F001~F003 수정 및 F004 종료 semantics 결정 후, actual production CLI/TUI trace와 status-gated action-space/replay lower-target 회귀를 재실행해야 한다.

## 8. Coder Handoff

`C:\LocalDev\rust\AIHack\docs\multi_audit\2\sub_audit_02_runtime_loop.md`를 먼저 읽고, 각 finding을 `spec.md`/`designs.md`와 실제 호출 경로·테스트에 대조한 뒤 우선순위대로 수정하세요. 계약 변경이 필요하면 관련 문서를 먼저 갱신하고, 수정 후 target runner, action-space/status, TUI transition 및 save/replay 테스트와 production 실행 증거를 기록하세요.
