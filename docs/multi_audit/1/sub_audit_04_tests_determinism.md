# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: 테스트 품질·결정론·인과 coverage·false-green (`tests_determinism`)
- User Goal: 현재 프로젝트의 문서와 구현을 대조하여 모순·문제점을 진단하고 해결 가능한 감사 결과를 만든다.
- Audit Basis: Standard-backed
- Standard Path: `C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md`
- Report Contract: `C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md`
- Audit Environment: Windows, Rust 1.94.1, workspace version 0.3.0
- Independence: 다른 `docs/multi_audit` 보고서는 읽지 않고 소스·테스트·통제 문서만으로 판정했다.

## 2. Assigned Scope

- root `tests/**/*.rs`, `tests/fixtures/**`, `tests/support/**`
- `crates/aihack-core`, `crates/aihack-runtime`, `crates/aihack-content`, `crates/aihack-llm`의 테스트와 테스트에 연결된 구현
- `apps/aihack-headless`, `apps/aihack-tui`의 contract/UI/LLM 테스트
- 장기 실행·causal witness·save/load·replay·hash·release/provenance·UI·LLM 테스트
- fixture builder와 저장·replay capability 경계
- 결정론, semantic delta, negative case, OS `cfg`로 인한 coverage 손실, 테스트 간 환경 충돌 가능성

판정 기준은 `AI_AUDIT_DOC_STANDARD.md`의 `DBG-002`, `TEST-001`, `BUILD-001`과 `spec.md`의 SC-TEST-02 및 R9 SC-CAUSE-01..07이다.

## 3. Excluded and Uninspected Scope

- `legacy_nethack_port_reference/**`의 legacy body는 사용자 범위에서 제외했다. 현재 runtime이 해당 트리를 import하지 않는다는 파일 경계만 확인했다.
- `target/**`, `.git/**`, 생성된 release output과 reference corpus는 산출물 자체의 테스트 대상이 아니므로 내용 검사를 하지 않았다.
- 외부 LLM provider 호출은 하지 않았다. LLM 테스트는 저장소의 loopback/mock provider 범위만 실행했다.
- 수동 TUI/PTY 스크립트(`scripts/r6_pty_matrix.sh`, `scripts/r8_tui_core_flow.sh`)와 실제 release build/package 실행은 이 관점에서 실행하지 않았다.
- `cargo test --release` 및 `cargo build --workspace --release --locked`는 실행하지 않았다. debug workspace 실행 결과와 표적 테스트 결과만 아래에 기록한다.
- Linux 전용 경로는 현재 Windows에서 실행하지 못했다. 특히 `tests/release_bundle.rs`는 소스와 Windows test listing을 검사했지만 Windows runtime behavior는 미실행으로 분류했다.

## 4. Evidence Examined

### 확인한 문서

- `spec.md`의 저장/replay 정책, SC-TEST-02, R9-1..R9-6와 SC-CAUSE-01..07
- `BUILD_GUIDE.md`, `README.md`, `IMPLEMENTATION_SUMMARY.md`의 workspace/full-test/release/Windows gate 주장
- `AI_AUDIT_DOC_STANDARD.md`와 report contract

### 확인한 구현·테스트

- `crates/aihack-runtime/src/causal.rs`, `crates/aihack-runtime/src/save.rs`, `crates/aihack-runtime/src/snapshot.rs`
- `crates/aihack-core/src/score.rs`, `apps/aihack-headless/src/lib.rs`
- `tests/long_run.rs`, `tests/causal_content.rs`, `tests/save_load.rs`, `tests/replay.rs`, `tests/headless_policy.rs`, `tests/headless_paths.rs`
- `tests/release_bundle.rs`, `tests/release_gate.rs`, `tests/provenance_manifest.rs`, `tests/r8_documentation.rs`
- `tests/llm_transport.rs`, `tests/llm_tui_integration.rs`, `tests/llm_narrative.rs`, `tests/llm_decision_support.rs`, `tests/llm_revision_gate.rs`, `tests/llm_soft_adjudication.rs`
- runtime/core/app package contract tests와 `src/testing.rs`, `tests/support/session_builder.rs`

### 실행 증거

| Command | Result | Timing / Note |
| --- | --- | --- |
| `cargo test --workspace --all-targets --locked` | PASS for all executed tests | Windows debug run. `-- --list`에서 363개 test entry를 확인했으며 full run은 실패 없이 종료했다. `release_bundle`는 Windows에서 0 tests였다. |
| `cargo test -p aihack --locked --test save_load invalid_save_schema_is_rejected -- --exact --nocapture` | PASS (1 passed) | 테스트가 통과하지만 F001의 malformed JSON false-green을 재현하는 실행 증거다. |
| `cargo test -p aihack --locked --test long_run causal_validator_rejects_event_only_turn_only_and_missing_witnesses -- --exact` | PASS | test 0.84s, 명령 약 1.4s |
| `cargo test -p aihack --locked --test long_run causal_fixture_covers_every_required_witness_for_each_seed -- --exact` | PASS | test 2.40s, 명령 약 2.8s |
| `cargo test -p aihack --locked --test long_run causal_witness_multiset_and_final_hash_are_stable_across_three_runs -- --exact` | PASS | test 7.13s, 명령 약 7.6s |
| `cargo test -p aihack --locked --test long_run survival_policy_hash_is_stable_across_three_runs_per_seed -- --exact` | PASS | test 32.75s, 명령 약 33.3s |
| `cargo test -p aihack --locked --test release_bundle -- --list` | 0 tests | Windows에서 `tests/release_bundle.rs`의 `#![cfg(unix)]`가 전체 fixture matrix를 제거한다. |

전체 debug 실행에서 `long_run` 5개는 51.66초, `provenance_manifest` 14개는 252.60초가 소요되었다. 후자는 Git/Bash fixture를 반복 생성하는 테스트가 많아 CI 시간·timeout 위험도 함께 관찰했다.

## 5. Findings

### [A04-F001] Save schema negative test가 JSON 문법 오류를 schema mismatch로 오인한다

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`, `DBG-002`
- Area: SaveDataV1 schema regression
- Severity: Major
- Status: Confirmed
- Summary: 테스트 이름은 schema version 999를 거부하는지 확인한다고 선언하지만 실제 fixture는 유효 JSON이 아니어서 deserialization syntax error만 검증한다.
- Evidence:
  - `tests/save_load.rs:123-132`의 raw string은 `\"schema_version\"`처럼 backslash를 보존하므로 JSON object가 아니라 escape가 섞인 잘못된 JSON이다.
  - 동일 테스트를 `cargo test -p aihack --locked --test save_load invalid_save_schema_is_rejected -- --exact --nocapture`로 실행하면 1 passed가 되지만, `assert!(...is_err())`만 있어 오류 원인을 구분하지 않는다.
  - 실제 schema 분기는 `crates/aihack-runtime/src/save.rs:193-199`의 `SaveSchemaVersionMismatch`다. 해당 variant를 직접 검증하는 다른 테스트는 검색되지 않았다.
- Expected Basis: `spec.md` §14의 `SaveDataV1 schema_version = 1` 및 schema 호환성 회귀 기준. 유효한 JSON에서 버전 불일치가 typed error로 거부되어야 한다.
- Actual: malformed JSON이면 schema field 파싱 전에도 테스트가 통과한다. schema check가 제거되거나 깨져도 이 테스트는 계속 통과할 수 있다.
- Impact: 저장 파일의 버전 호환성 경계가 false-green 상태이며, SaveDataV2 전환·손상 save 회귀를 조기에 발견하지 못한다.
- Suggested Action: backslash를 제거한 유효 JSON 또는 정상 save를 `serde_json::Value`로 읽어 `schema_version`만 999로 바꾸는 fixture를 사용하고, `GameError::SaveSchemaVersionMismatch { expected: 1, actual: 999 }`를 정확히 assert한다. malformed JSON rejection은 별도 테스트로 분리한다.
- Re-audit Method: 위 targeted command를 재실행하고 두 테스트가 각각 syntax error와 typed schema mismatch를 구분하는지 확인한다. `cargo test -p aihack --locked --test save_load`와 save/load continuation도 함께 실행한다.
- Confidence: High
- Notes: 현재 구현의 schema branch 존재 여부와 테스트의 주장된 검증 대상이 불일치한다.

### [A04-F002] MonsterSpeed와 MonsterAi causal witness가 같은 이동 한 번으로 동시에 충족된다

- Pass: Debug / Engineering Quality
- Pattern: `DBG-002`, `TEST-001`
- Area: R9 semantic causal witness independence
- Severity: Major
- Status: Confirmed
- Summary: 장기 causal summary가 speed와 AI를 별도 downstream 인과로 세지 않고, 하나의 `moved` predicate가 참이면 두 witness를 모두 기록한다.
- Evidence:
  - `crates/aihack-runtime/src/causal.rs:250-263`는 살아 있고 speed가 양수이며 stationary가 아닌 monster의 위치가 바뀌면 `MonsterSpeed`와 `MonsterAi`를 연속으로 기록한다. speed schedule 차이와 AI intent 차이를 각각 검증하지 않는다.
  - `tests/long_run.rs:154-160`은 두 count가 모두 1 이상이면 speed/AI 단계의 fixture를 즉시 종료한다. 두 이름이 같은 이동 이벤트에서 나온다는 사실을 가리는 구조다.
  - `tests/causal_content.rs:34-145`에는 별도 A/B registry 테스트가 있지만, 장기 `REQUIRED_CAUSAL_WITNESSES`가 해당 독립 A/B 결과를 요구하거나 witness별 필드 차분을 보존하지는 않는다.
- Expected Basis: `spec.md` R9-3 및 SC-CAUSE-05..07의 typed witness와 semantic world-state delta. 각 witness는 자기 원인과 downstream 결과를 분리해 증명해야 한다.
- Actual: AI 소비자가 no-op이거나 speed만으로 이동이 발생해도 두 witness가 함께 증가한다. 동일 사건의 라벨 복제이며 인과 closure가 아니다.
- Impact: R9 장기 gate와 witness multiset 결정론 테스트가 통과해도 AI 또는 speed 한 경로가 끊긴 상태를 놓칠 수 있다. `summary.validate_required()`가 false-green이 된다.
- Suggested Action: speed witness는 speed A/B 또는 speed schedule의 turn/movement delta를, AI witness는 동일 위치에서 AI 종류만 바꾼 intent/target/position delta를 각각 기록한다. witness record에 before/after semantic field와 scenario ID를 포함하고 하나의 move event가 두 witness를 자동 충족하지 못하게 한다. speed/AI 라벨 교환·한 소비자 no-op negative도 추가한다.
- Re-audit Method: 3개 seed causal fixture를 다시 실행하고 witness별 evidence map을 확인한다. speed 값만 0으로 바꾼 registry와 AI만 stationary로 바꾼 registry에서 각각 기대 witness가 하나만 생성되는 A/B 테스트를 추가·실행한다.
- Confidence: High
- Notes: `causal_content`의 독립 테스트가 존재한다는 점은 긍정적이나, 현재 장기 witness 구현의 독립성을 보장하지 않는다.

### [A04-F003] GoldScore witness는 gold가 score에 미치는 차분을 검증하지 않는다

- Pass: Debug / Engineering Quality
- Pattern: `DBG-002`, `TEST-001`
- Area: R9 economy-to-score causal proof
- Severity: Major
- Status: Confirmed
- Summary: GoldScore 판정은 score가 gold와 실제로 연결되는지 보지 않고, before gold가 양수이고 최종 score가 그보다 크거나 같은지만 확인한다.
- Evidence:
  - `crates/aihack-runtime/src/causal.rs:182-190`의 조건은 `before.gold > 0`과 `final_score >= before.gold`뿐이며 `after.gold - before.gold`나 score delta를 비교하지 않는다.
  - 실제 score는 `crates/aihack-core/src/score.rs:12-16`에서 gold, kill count, depth, inventory value, turn penalty를 합산한다. gold 항을 제거해도 다른 항이 충분히 크면 현재 predicate는 참일 수 있다.
  - `tests/long_run.rs:286-293`은 monsters를 제거한 후 1000턴까지 진행하고 Quit한다. 이전에 만든 kill/inventory/depth 항이 남아 있어 GoldScore 조건을 단독으로 증명하지 않는다.
  - `tests/causal_content.rs:194-222`의 A/B는 item `base_price`를 바꾸어 inventory value 차이를 확인할 뿐, gold만 바꾼 score A/B가 아니다.
- Expected Basis: `spec.md` R9-4, SC-CAUSE-04와 R9-6의 gold/score semantic delta. 원인 상태 하나만 바꾼 paired run에서 score downstream 차이가 보여야 한다.
- Actual: unrelated score terms만으로 `final_score >= before.gold`를 만족할 수 있고, gold 항이 사라져도 witness가 기록될 가능성이 있다.
- Impact: 경제·점수 causal closure가 false-green이 되어 gold reward와 final score 연결 회귀를 놓친다.
- Suggested Action: 동일 session state를 복제한 gold-only A/B에서 `final_score_high - final_score_low == gold_high - gold_low`를 assert한다. observer는 최소한 `after.gold > before.gold`와 score delta가 해당 gold delta를 반영하는지 함께 확인해야 한다. inventory/kill/depth 기여가 분리된 fixture를 사용한다.
- Re-audit Method: gold-only score A/B 및 kill→gold→Quit sequence를 실행하고 모든 비-gold 항을 동일하게 유지했는지 snapshot 비교로 확인한다. 이후 3개 seed causal witness count와 hash를 갱신·재실행한다.
- Confidence: High
- Notes: 현재 score 함수에 gold 항 자체는 있으나, 장기 witness predicate가 그 사실을 증명하지 않는다.

### [A04-F004] SC-CAUSE-07의 turn-only negative fixture가 별도로 존재하지 않는다

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`, `DBG-002`
- Area: causal validator negative coverage
- Severity: Major
- Status: Confirmed
- Summary: 테스트 이름은 event-only·turn-only·missing witness를 모두 검사한다고 쓰지만, 실제로는 한 개의 combined fixture만 사용한다.
- Evidence:
  - `tests/long_run.rs:104-128`은 `PrayerOffered` event가 있는 `TurnOutcome`과 동일한 `projection`을 before/after로 전달한 한 사례만 만든다. 별도의 turn-only(`events=[]`, turn만 증가) 사례가 없다.
  - `CausalProjection`의 필드(`crates/aihack-runtime/src/causal.rs:62-75`)에는 turn/event_count/last_event가 없으므로 실제로 semantic projection이 동일한 turn-only 입력과 event-only 입력을 구별하는 테스트 구조가 필요하다.
  - `CausalSummary::observe`의 `before == after` early return(`causal.rs:161-163`)이 결과를 0으로 만들지만, 두 실패 모드를 각각 이름 붙여 잠근 증거는 없다.
- Expected Basis: `spec.md` SC-CAUSE-07과 R9-6의 “event-only 또는 turn-only 변화에서 실패” 조건.
- Actual: 한 fixture에 `turn_advanced=true`와 event를 함께 넣고 `event-only`로만 이름 붙였다. turn-only regression이 따로 깨지는지 알 수 없다.
- Impact: validator가 event presence는 막아도 turn-only metadata drift를 잘못 집계하는 경로를 놓칠 수 있고, SC-CAUSE-07 coverage가 과장된다.
- Suggested Action: `event_only_projection_is_rejected`, `turn_only_projection_is_rejected`, `missing_witness_is_rejected`를 분리한다. turn-only의 표현을 projection 외부 turn 입력으로 둘지 명세에 명시하고, 실제 `submit` 결과의 hash·turn·events 조합과 함께 negative assertion을 둔다.
- Re-audit Method: 세 테스트를 각각 단독 실행하고 `total_count()==0`, `validate_required().is_err()`를 모두 확인한다. R8 documentation mapping의 SC-CAUSE-07 test function 목록도 새 함수와 동기화한다.
- Confidence: High
- Notes: 현재 combined test는 일부 방어 가치를 가지지만, 요구된 세 negative case의 독립 coverage를 증명하지 않는다.

### [A04-F005] Windows에서는 release bundle integration test 전체가 `cfg(unix)`로 제거된다

- Pass: Debug / Engineering Quality
- Pattern: `BUILD-001`, `DBG-002`, `TEST-001`
- Area: cross-platform release/package verification
- Severity: Major
- Status: Confirmed
- Summary: release bundle의 complete/negative fixture matrix가 파일 첫 줄의 `#![cfg(unix)]` 때문에 Windows에서 컴파일·실행되지 않는다.
- Evidence:
  - `tests/release_bundle.rs:1`이 파일 전체를 Unix로 제한한다. 이 파일은 375줄의 archive, metadata exactness, legacy include negative matrix를 포함한다.
  - 현재 Windows에서 `cargo test -p aihack --locked --test release_bundle -- --list` 결과는 `0 tests, 0 benchmarks`였고, 전체 workspace run에서도 `release_bundle`는 `running 0 tests`로 나타났다.
  - `tests/headless_paths.rs:115-139`의 mode 0600과 `:200-223`의 symlink escape도 Unix 전용이며, `:141-163` Windows 대체 테스트는 writable/read-only와 regular replacement만 확인한다.
- Expected Basis: `spec.md` SC-BUILD-02와 `BUILD_GUIDE.md`의 Linux/Windows quality gate·release bundle contract. 양 OS에서 release artifact와 저장/replay 경계를 실제로 회귀 검증해야 한다.
- Actual: Windows quality gate가 통과해도 bundle verifier fixture의 Windows 실행 경로, reparse/symlink escape, Unix/Windows artifact 차이를 이 테스트 세트가 확인하지 않는다.
- Impact: `build.bat`/Windows packaging drift가 full workspace PASS에 숨어 배포 artifact가 Linux에서만 검증된 상태가 된다. save/replay link boundary도 Windows에서 negative coverage가 빈약하다.
- Suggested Action: release fixture를 cross-platform helper로 분리하고 Windows에서도 Git Bash/PowerShell로 archive와 verifier를 실행하거나 build.bat 산출물을 검증한다. Windows는 가능한 환경에서 junction/reparse-point 또는 capability-specific no-follow negative를 추가하고, 링크 생성 권한이 없는 runner의 skip 조건과 잔여 coverage를 명시한다.
- Re-audit Method: Windows clean checkout에서 `cargo test --workspace --all-targets --locked` 후 `release_bundle`가 0이 아닌지 확인하고, `build.bat --release`와 output verifier를 같은 commit으로 실행한다. Unix symlink와 Windows reparse fixture 결과를 별도 기록한다.
- Confidence: High
- Notes: `release_gate`/`provenance_manifest`에는 Windows Bash 경로 처리가 있지만, Unix 전용 release bundle matrix의 공백을 대체하지 않는다.

### [A04-F006] Replay runner가 기록된 turn/outcome/hash를 검증하지 않아 replay integrity 기준이 불명확하다

- Pass: Debug / Engineering Quality
- Pattern: `DBG-002`, `TEST-001`
- Area: replay determinism and tamper/negative coverage
- Severity: Major
- Status: Needs Clarification
- Summary: `ReplayLineV1`은 turn_before, outcome, snapshot_hash_after를 저장하지만 replay 실행기는 command만 재제출한다. 이 메타데이터를 검증하지 않는 것이 의도된 trusted-command replay인지, integrity-preserving replay 누락인지 명세가 닫혀 있지 않다.
- Evidence:
  - `apps/aihack-headless/src/lib.rs:59-95`의 `run_replay_to_turn`은 각 line의 `line.command`만 `session.submit`하고 `turn_before`, `line.outcome`, `line.snapshot_hash_after`와 실제 outcome을 비교하지 않는다.
  - `tests/replay.rs:31-80`은 정상적으로 생성한 4개 Wait sequence의 JSONL roundtrip과 direct continuation만 확인한다.
  - `tests/headless_policy.rs:42-54`도 1-turn 정상 trace의 최종 hash만 비교하며, tampered hash/turn, rejected command, wrong command 또는 early exhaustion negative가 없다(실제 `ReplayExhausted` branch도 직접 잠기지 않음).
  - `spec.md` §14는 line field와 schema를 정의하지만 line metadata를 신뢰할지 검증할지 명시하지 않는다.
- Expected Basis: spec §14의 ReplayLineV1 field contract와 사용자 요청의 replay/hash 결정론. 최소한 trusted replay인지 self-verifying artifact인지 결정되어야 하며, 후자라면 매 line의 causal/hash chain을 검증해야 한다.
- Actual: 잘못된 metadata가 들어간 line도 command가 target turn까지 진행되면 성공 report가 될 수 있다. 현재 happy-path test는 이 동작을 탐지하지 않는다.
- Impact: replay 파일을 재현 증거·감사 artifact로 사용할 때 손상/변조를 조용히 수용할 수 있다. 반대로 trusted command log가 목적이라면 불필요한 필드가 검증되지 않은 채 계약 혼란을 만든다.
- Suggested Action: 먼저 spec에서 trust model을 결정한다. integrity replay라면 `turn_before == session.turn`, actual outcome equality, actual hash equality, accepted/turn advancement와 line outcome의 일치를 검증하고 tamper/duplicate/rejected/exhausted fixtures를 추가한다. trusted command log라면 ignored fields를 제거하거나 “검증하지 않음”을 문서·테스트에 명시한다.
- Re-audit Method: tampered replay matrix(해시 변경, turn 변경, outcome 변경, command 변경, target보다 짧은 trace)를 실행해 기대 오류/성공을 확인하고 save→replay continuation의 per-line hash chain을 검증한다.
- Confidence: High for observed behavior; Medium for gate severity because replay trust model is unspecified.
- Notes: 이 finding은 요구사항 창작을 피하기 위해 `Needs Clarification`으로 분류했지만, clarification 전에는 replay integrity PASS를 주장할 수 없다.

### [A04-F007] Content A/B fixture가 ID가 아닌 첫 문자열 일치 항목을 바꾼다

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`
- Area: content-backed causal fixtures
- Severity: Minor
- Status: Confirmed
- Summary: causal content tests가 typed registry record를 선택하지 않고 TOML raw string의 첫 `speed=12`, `ai="wander"`, `difficulty=1`, `base_price=4`를 `replacen`한다.
- Evidence:
  - `tests/causal_content.rs:43-50`은 `speed=12` 첫 occurrence를 0으로 바꾼다.
  - `:98-105`는 첫 `ai="wander"`를 stationary로, `:156-167`은 첫 `difficulty=1`에 passive를 삽입한다.
  - `:202-209`는 첫 `base_price=4`를 404로 바꾼다. 현재 TOML 순서에서는 의도한 jackal/dagger와 일치하지만, replacement 횟수·ID·변경된 typed field를 assert하지 않는다.
- Expected Basis: R9-3의 injected registry A/B가 한 content field의 차이로 의도된 entity semantic delta를 만들어야 한다.
- Actual: data file에 새 record를 앞에 추가하거나 field 순서를 바꾸면 다른 entity가 바뀌어도 fixture가 parse되고, 결과가 우연히 통과하거나 잘못된 원인을 검증할 수 있다.
- Impact: content behavior 회귀의 원인 attribution이 fixture layout에 결합되고 false-green/false-fail 위험이 생긴다.
- Suggested Action: ID 기반 TOML fixture helper 또는 parsed `ContentRegistry` mutation seam을 사용하고, 정확히 한 record/field가 바뀌었는지와 typed value를 assert한다. 현재 data ordering 의존성을 제거한다.
- Re-audit Method: record 순서를 바꾼 별도 fixture에서도 intended ID만 변하는지 확인하고 `cargo test -p aihack --locked --test causal_content`를 재실행한다.
- Confidence: High
- Notes: 현재 데이터에서 즉시 실패하는 문제는 아니지만, 장기 유지보수 시 결정론과 인과 귀속을 약화시키는 brittle baseline이다.

### [A04-F008] LLM loopback 테스트가 semantic 결과와 함께 wall-clock sleep/deadline에 의존한다

- Pass: Debug / Engineering Quality
- Pattern: `DBG-002`, `TEST-001`
- Area: asynchronous LLM worker determinism
- Severity: Minor
- Status: Probable
- Summary: LLM core path는 외부 호출 없이 mock/loopback으로 검증되어 좋지만, readiness·timeout·cooldown을 실제 scheduler timing으로 판정하는 테스트가 있어 느린 CI에서 flaky할 수 있다.
- Evidence:
  - `tests/llm_transport.rs:394-400`, `:662-668`은 1초 wall-clock deadline과 `thread::yield_now()` polling을 사용한다.
  - `:512-527`은 100ms client timeout과 250ms server sleep 차이에 의존하고, `:688-690`은 10ms grace 후 실제 elapsed가 250ms 미만이어야 한다고 가정한다.
  - `tests/llm_tui_integration.rs:325`는 250ms cooldown을 넘기기 위해 `thread::sleep(260ms)`를 호출한다.
  - 현재 Windows debug 실행에서는 관련 테스트가 PASS했지만, 이 구조는 실행 부하에 따라 timing margin이 달라진다.
- Expected Basis: `DBG-002`의 반복 가능한 headless/fixture 검증. external boundary의 semantic state transition은 scheduler timing과 분리하는 것이 바람직하다.
- Actual: mock response/error 분류 자체는 deterministic하지만, readiness와 cooldown 완료를 wall-clock으로만 관찰한다.
- Impact: 병렬 CI 또는 저사양 runner에서 false failure·과도한 252초급 fixture suite 시간이 생길 수 있다. 기능 false-green보다 재현성/운영 비용 위험이다.
- Suggested Action: response channel barrier와 explicit test clock/scheduler seam을 주입하고, timeout test는 server-side gate로 “response를 보내지 않음/보냄”을 제어한다. 최소한 deadline margin과 cleanup timeout을 명시적으로 분리한다.
- Re-audit Method: 고병렬 Windows/Linux runner에서 반복 실행하고, sleep 제거 후 동일 status/error/hash 결과와 bounded shutdown을 확인한다.
- Confidence: Medium
- Notes: 현재 관찰된 실패는 없으므로 `Probable`로 분류한다.

### [A04-F009] runtime package contract 테스트 일부가 callability/smoke만 확인한다

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`, `DBG-002`
- Area: package-level semantic coverage
- Severity: Minor
- Status: Confirmed
- Summary: root integration tests의 semantic coverage는 넓지만, runtime package 자체의 여러 contract test는 API가 존재하거나 panic하지 않는지만 확인한다.
- Evidence:
  - `crates/aihack-runtime/tests/entity_store_contract.rs:5-11`은 spawn 후 `is_some()`만 확인한다.
  - `environment_systems_contract.rs:8-17`은 시스템 호출 가능성과 한 `None`/`Err` 결과만 확인한다.
  - `projection_contract.rs:6-15`은 player position과 snapshot 자기 자신 hash 안정성만 확인하고 semantic state delta를 확인하지 않는다.
  - `game_client_contract.rs:5-31`의 adapter 구현은 모든 메서드를 `unreachable!()`로 둔 compile-only contract다.
- Expected Basis: runtime 경계의 구현 완료 주장은 호출 존재가 아니라 구체적 상태·event·hash·negative behavior로 회귀되어야 한다(`TEST-001`, `DBG-002`).
- Actual: 해당 package 내부에서 entity/system/projection/client behavior가 깨져도 이 contract test들은 계속 통과할 수 있다. root tests가 일부를 보완하지만 모든 package-local contract를 대체하지는 않는다.
- Impact: workspace facade 재배치나 adapter refactor 때 semantic regression의 검출 지연이 생긴다.
- Suggested Action: 각 contract test에 최소 semantic assertion을 추가한다(예: movement 위치/turn/hash, projectile resource/lifecycle, projection field, invalid environment command의 no-commit). compile-only trait test는 compile contract로 명시하고 runtime behavior test와 분리한다.
- Re-audit Method: runtime package test를 단독 실행하고 각 test가 production `GameSession::submit` 또는 실제 world state delta를 관찰하는지 확인한다.
- Confidence: High
- Notes: root `tests/monster_ai.rs`, `tests/items.rs`, `tests/nethack_367_compat.rs` 등은 상대적으로 강한 보완 증거로 인정한다. 따라서 이 finding은 전체 coverage 부재가 아니라 package-local contract의 국소 공백이다.

## 6. Uncertainties and Clarifications Needed

1. **Replay trust model:** `ReplayLineV1`의 outcome/hash를 검증하지 않는 것이 의도된 trusted command log인지, self-verifying replay artifact 누락인지 `spec.md`에서 결정해야 한다. 이 결정 전에는 replay integrity에 대해 PASS 계열 판정을 내리지 않는다.
2. **Windows release test authority:** `release_bundle.rs` Unix fixture를 Windows CI의 `build.bat --release`/verifier가 완전히 대체한다고 볼지, 동일 negative fixture를 양 OS에서 실행해야 하는지 문서에 명시해야 한다. 현재 SC-BUILD-02와 BUILD_GUIDE의 표현은 양 OS gate를 요구하는 쪽으로 읽힌다.
3. **GoldScore witness contract:** 단순 `final_score >= before.gold`가 제품 acceptance인지, gold-only score delta가 필요한지 R9-4의 semantic delta 문장에서 명확히 닫아야 한다. 현재 사용자 목표와 R9 문맥에서는 후자가 더 강한 기준이다.

## 7. Perspective Decision

**HOLD — 테스트 결정론·인과 coverage 관점에서 R9/release PASS를 인증할 수 없다.**

현재 Windows debug workspace의 363개 실행 대상은 모두 통과했고, root integration 테스트와 loopback LLM 테스트의 폭은 양호하다. 그러나 F001의 save schema false-green, F002/F003의 causal witness false-green, F004의 누락된 turn-only negative, F005의 Windows release bundle 0-test 공백은 모두 테스트가 성공했다는 사실만으로 해소되지 않는다. F006은 명세 결정 없이는 replay integrity를 판정할 수 없고, F007~F009는 유지보수·flakiness·package-local semantic coverage 위험이다.

`CausalSummary` witness predicate를 독립 semantic delta로 분리하고 save/schema 및 negative replay/turn-only fixture를 보강한 뒤, Windows release bundle·reparse path coverage를 실제 실행하고 관련 문서 mapping을 재감사해야 한다.

