# D3D Audit Report 2

감사일: 2026-07-16 (Asia/Seoul)

감사 기준: AI_AUDIT_DOC_STANDARD.md

감사 대상: 현재 working tree의 활성 문서, Rust source/test, build·dependency·security 설정

감사 방식: Implementation Compliance, Debug / Engineering Quality, Security의 독립 3-pass

최초 감사 시 코드 수정: 없음. 이후 사용자 요청에 따른 remediation과 Re-audit #1은 13절에 기록한다.

최초 판정: **HOLD**. 현재 판정은 13절 Re-audit #1을 따른다.

## 감사 요약

- clean target에서 209개 테스트가 모두 통과했다.
- fmt, Clippy -D warnings, debug all-target build, release build, locked metadata, git diff check가 통과했다.
- cargo audit은 207개 lockfile dependency를 검사해 exit 0이었고, cargo deny의 licenses/bans/sources도 통과했다.
- R3 ContentRegistry의 fallible production bootstrap과 injected invalid-content 회귀 테스트는 확인됐다.
- 그러나 R2 완료 문서가 선언하는 GameClient, Result<TurnOutcome, SubmitError>, SessionRevision, ReplayTurnOutcomeV1 계약은 source와 tests에 없다.
- R3 완료 후에도 DESIGN_DECISIONS.md와 GAP_CLOSURE_ROADMAP.md는 bootstrap expect 경계가 남았다고 기록한다.
- Major finding 2건이 미해결이므로 AI_AUDIT_DOC_STANDARD.md 8~9절에 따라 PASS할 수 없다.

## 1. Audit Scope

### 1.1 프로젝트 인벤토리

| 항목 | 확인 결과 |
| --- | --- |
| 프로젝트 유형 | Rust 단일-package TUI/headless 턴제 게임 |
| package | aihack 0.1.0, edition 2021, rust-version 1.94 |
| toolchain | rustc/cargo 1.94.1 |
| source | src/ 62개 파일, 그중 Rust 58개 |
| tests | tests/ 40개 파일, 그중 Rust 38개, test 선언 209개 |
| dependency manifest | Cargo.toml, Cargo.lock, deny.toml |
| CI/CD | .github/workflows/ci.yml |
| build/run | build.sh, build.bat, BUILD_GUIDE.md, README.md |
| security 경계 | spec.md 16절, DESIGN_DECISIONS.md ADR-0026/0027, deny.toml, CI supply-chain gate |

### 1.2 확인한 문서

- AGENTS.md
- AI_AUDIT_DOC_STANDARD.md
- spec.md
- designs.md
- IMPLEMENTATION_SUMMARY.md
- DESIGN_DECISIONS.md
- BUILD_GUIDE.md
- audit_roadmap.md
- GAP_CLOSURE_ROADMAP.md
- README.md
- CHANGELOG.md
- LESSONS_LEARNED.md
- PROVENANCE.md
- DOCUMENTATION_AUDIT_REPORT.md
- audit_report_1.md
- docs/compatibility/README.md

### 1.3 확인한 구현

- src/core, src/data, src/domain, src/systems
- src/ui, src/llm
- src/main.rs, src/bin/aihack-headless.rs
- tests 전체 및 tests/support
- Cargo.toml, Cargo.lock, rust-toolchain.toml, deny.toml
- .github/workflows/ci.yml, build.sh, build.bat

### 1.4 실행한 주요 명령

~~~bash
rustc --version
cargo --version
cargo metadata --locked --no-deps --format-version 1
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo audit
cargo deny check licenses bans sources
cargo tree -d
git diff --check
~~~

clean-room 전체 테스트는 다음 저용량 debug 설정으로 별도 target에서 재실행해 통과했다.

~~~bash
CARGO_TARGET_DIR=target/audit-clean \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=0 \
CARGO_PROFILE_TEST_DEBUG=0 \
cargo test --workspace --all-targets --locked
~~~

R3와 R4 상태를 구분하기 위해 다음도 실행했다.

~~~bash
cargo test --locked \
  --test content_validation --test content_runtime \
  --test data_loading --test items --test monster_ai --test levels

target/release/aihack-headless --seed 42 --turns 1000
target/release/aihack-headless --seed 7 --turns 1000
target/release/aihack-headless --seed 1234 --turns 1000

cargo test --locked --release --test headless_policy
target/release/aihack-headless \
  --seed 42 --turns 1000 --policy survival-v1
~~~

## 2. Excluded Scope

- legacy_nethack_port_reference/: reference-only/vendor 성격의 격리 tree이므로 active runtime 구현에서 제외했다.
- target/, output/, .git/: 생성물·VCS metadata이며 source of truth가 아니다.
- GitHub Actions의 실제 Ubuntu/Windows 원격 실행: workflow는 확인했으나 원격 CI run 결과는 이 로컬 감사에서 실행하지 않았다.
- TUI 전체 수동 상호작용: non-interactive audit 환경이므로 자동 UI smoke/layout/input test로 제한했다.
- R4~R8 완료 검증: 문서가 NOT RUN 또는 구현 예정으로 명시하며, 현재 존재 여부와 gate 상태만 대조했다.
- 법률 자문: provenance 및 license gate의 구조만 확인했으며 라이선스 적합성의 법률 판단은 하지 않았다.

첫 /tmp clean target 시도는 user tmpfs quota 초과로 중단됐다. 이는 source 실패가 아니다. 저장소의 ignored target/audit-clean에서 저용량 clean build/test를 재실행해 209개 테스트 통과를 확인했고, 해당 임시 target은 감사 후 정리했다.

## 3. Pass 1: Implementation Compliance Findings

## [IMP-F001] R2 완료 문서의 경계 타입 계약이 실제 public API와 다름

- Pass: Implementation Compliance
- Pattern: IMP-003, IMP-004
- Area: spec.md 9.1·9.2·14절, IMPLEMENTATION_SUMMARY.md R2, DESIGN_DECISIONS.md ADR-0023, src/core
- Severity: Major
- Status: Needs Fix
- Summary: R2가 완료됐다고 표시됐지만 master spec과 완료 문서가 선언한 typed submit/revision/replay 계약은 구현되어 있지 않다.
- Evidence:
  - spec.md:219-244는 GameClient, SessionRevision, Result<TurnOutcome, SubmitError>를 active contract로 정의한다.
  - spec.md:247은 action_space getter와 Err(SubmitError)의 무변경 계약을 요구한다.
  - spec.md:259-270의 WorldInvariantError 6종과 InvariantReport.checked 타입은 실제 구현과 다르다.
  - spec.md:700-719는 internal Result를 ReplayTurnOutcomeV1로 projection한다고 명시한다.
  - IMPLEMENTATION_SUMMARY.md:329-359는 R2-3과 SC-CORE-01/02를 완료 처리하고, line 341에서 internal Result API와 ReplayTurnOutcomeV1 projection을 체크 완료했다.
  - DESIGN_DECISIONS.md:94-119는 ADR-0023을 Implemented로 표시하고 GameClient read boundary 및 typed error를 결정으로 기록한다.
  - src/core/session.rs:132-143의 submit은 Result가 아닌 TurnOutcome을 직접 반환한다.
  - src/core/turn.rs:11-16의 TurnOutcome은 accepted bool, snapshot_hash, next_state를 직접 보유한다.
  - src/core/invariant.rs:14-41의 실제 enum variant와 checked: u8은 spec의 variant와 checked: u16과 다르다.
  - source/test 검색 결과 GameClient, SessionRevision, SubmitError, ReplayTurnOutcomeV1, action_space method는 0건이다.
  - IMPLEMENTATION_SUMMARY.md:351이 책임 파일로 지목한 tests/fixtures/replay_v1.json도 존재하지 않는다.
  - tests/transaction.rs와 tests/replay.rs는 현재 accepted bool 계약을 검증하므로, 전체 테스트 PASS가 spec 계약의 구현 증거가 되지 않는다.
- Expected: R2 완료 표시는 spec 9.1·9.2·14의 타입, signature, projection fixture와 실제 source/test가 일치할 때만 유지되어야 한다.
- Actual: transaction의 working-copy/no-commit 동작은 존재하지만 public API와 replay boundary는 이전 accepted-bool wire를 그대로 사용한다.
- Impact: R4 runner, R5 crate 경계, R6 revision/stale gate가 존재하지 않는 API를 선행 계약으로 삼게 된다. 이후 구현자가 문서를 따를지 현재 source를 따를지 결정할 수 없고, public compatibility 변경 시점도 흐려진다.
- Suggested Fix:
  1. spec을 authority로 유지한다면 GameClient, action_space, SessionRevision, SubmitError, Result submit, ReplayTurnOutcomeV1 projection과 fixture를 구현한다.
  2. 현재 API가 의도라면 spec, ADR-0023, IMPLEMENTATION_SUMMARY의 완료 기준을 실제 타입에 맞춰 수정하고 Result/revision 계약을 명시적 후속 Task로 이관한다.
  3. 두 선택 모두 R2 checkpoint를 재감사하기 전까지 완료/PASS 표시는 보류한다.
- Re-audit Method:
  - source/test에서 위 5개 contract type과 action_space 경계를 검색한다.
  - spec code block과 rustdoc/public signature를 type 단위로 대조한다.
  - transaction, replay, save/load, AI schema test를 다시 실행한다.
  - tests/fixtures/replay_v1.json 또는 동등한 canonical fixture의 실제 존재와 roundtrip assertion을 확인한다.
- Owner: Architect / Coder
- Notes: working-copy transaction과 invariant no-commit 테스트의 유효성은 인정한다. finding은 그 부분이 아니라 완료 범위와 public contract의 과대주장이다.

## [IMP-F002] R3 완료 증거가 활성 ADR·gap·audit 상태에 동기화되지 않음

- Pass: Implementation Compliance
- Pattern: IMP-004, DOC-BACKFILL-001
- Area: DESIGN_DECISIONS.md, GAP_CLOSURE_ROADMAP.md, audit_roadmap.md, audit_report_1.md
- Severity: Major
- Status: Needs Documentation Recovery
- Classification: Intentional but Undocumented / stale secondary authority
- Summary: source/test와 여러 활성 문서는 R3 fallible bootstrap 완료를 지지하지만, active decision/gap 문서는 이전 Hold 원인을 현재 사실로 유지한다.
- Evidence:
  - audit_report_1.md:182-184는 Re-audit #2에서 R3 LOCAL PASS와 fallible bootstrap 완료를 판정한다.
  - audit_roadmap.md:223과 400은 R3 local PASS를 기록한다.
  - IMPLEMENTATION_SUMMARY.md:433-460은 R3-4 수용 기준과 SC-DATA-01을 모두 체크 완료했다.
  - src/bin/aihack-headless.rs와 src/ui/tui/mod.rs는 GameSession::try_new 계열 production path를 사용한다.
  - tests/content_validation.rs의 missing main level/starting item 두 회귀 테스트와 R3 targeted 52개 test가 통과했다.
  - 반면 DESIGN_DECISIONS.md:123-144는 ADR-0024를 Partially implemented로 두고 current bootstrap expect 경계가 남았다고 기록한다.
  - GAP_CLOSURE_ROADMAP.md:45는 G-DATA-002를 Implemented로만 표시하고 production expect가 남았다는 과거 증거를 유지한다.
  - audit_roadmap.md:401은 현재 문서 감사를 HOLD(audit_report_1.md)로 기록하지만, 해당 보고서의 최종 판정은 R3 LOCAL PASS다.
- Expected: active ADR, gap register, audit roadmap, latest audit report가 R3의 같은 상태와 남은 범위를 설명해야 한다.
- Actual: 구현·테스트는 R3 완료를 지지하지만 결정 이력과 gap 상태는 R3-4 이전 상태를 유지한다.
- Impact: GAP_CLOSURE_ROADMAP의 규칙상 P1 gap은 R4 전 Closed여야 하므로 다음 구현이 R4라는 지시와 G-DATA-002 상태가 충돌한다. 감사자가 R3를 재작업할지 R4로 진행할지 일관되게 판단할 수 없다.
- Suggested Fix:
  1. ADR-0024의 status와 consequence를 현재 fallible production bootstrap에 맞춘다.
  2. G-DATA-002의 증거와 상태를 실제 재감사 결과에 맞춰 Verified/Closed로 전환한다.
  3. audit_roadmap의 current document audit 문구를 audit_report_1의 checkpoint 판정과 전체 R0~R8 미완료 판정으로 분리한다.
  4. historical Hold 문구는 날짜와 superseded report를 명시해 보존한다.
- Re-audit Method:
  - active docs에서 bootstrap error boundary pending, current expect path, G-DATA-002 상태를 다시 검색한다.
  - production entrypoint call site와 두 invalid-registry 회귀 테스트를 재확인한다.
  - R3 상태가 spec, summary, ADR, gap, audit roadmap, latest report에서 동일한지 대조한다.
- Owner: Architect / Coder
- Notes: 두 개 이상의 독립 근거(source, tests, audit_report_1, summary)가 같은 의도를 지지하므로 새 요구사항 창작이 아닌 문서 복구 대상이다.

## [IMP-F003] Lessons Learned가 존재하지 않는 감사 보고서 계보를 사실로 기록함

- Pass: Implementation Compliance
- Pattern: IMP-004, DOC-BACKFILL-001
- Area: LESSONS_LEARNED.md
- Severity: Minor
- Status: Needs Documentation Recovery
- Summary: 복구 문서가 현재 저장소에 없는 aihack_audit_report_1.md와 aihack_audit_report_2.md의 판정을 사실로 서술한다.
- Evidence:
  - LESSONS_LEARNED.md:204는 두 파일에서 HOLD와 PASS WITH KNOWN RISKS를 받았다고 기록한다.
  - 감사 시작 시 repository의 numbered report는 audit_report_1.md 하나뿐이었다.
  - audit_report_1.md의 현재 최종 판정은 PASS WITH KNOWN RISKS가 아니라 R3 LOCAL PASS다.
- Expected: 복구 문서는 실제 파일명과 현재/과거 판정, supersede 계보를 재구성 가능하게 기록해야 한다.
- Actual: 존재하지 않는 파일명과 확인할 수 없는 판정이 기록되어 있다.
- Impact: 향후 복구 시 잘못된 audit lineage를 따라가거나 누락된 보고서를 찾는 비용이 발생한다.
- Suggested Fix: 실제 보고서명과 판정으로 교정하거나, 외부/과거 artifact라면 경로·시점·현재 저장소에 없는 이유를 명시한다.
- Re-audit Method: repository 전체에서 언급된 audit report 경로가 모두 존재하고 각 Final Decision과 서술이 일치하는지 확인한다.
- Owner: Architect / Coder
- Notes: release gate를 단독 차단하지 않지만 R8의 복구 가능성·문서 폐쇄성에는 포함해야 한다.

## 4. Pass 2: Debug / Engineering Quality Findings

새로운 독립 code correctness finding은 확인되지 않았다. 다만 IMP-F001 때문에 green test suite를 문서 계약 PASS로 해석할 수 없다.

### 4.1 Verified engineering evidence

- clean target 전체 209 tests PASS
- cargo fmt --all -- --check PASS
- cargo clippy --workspace --all-targets --locked -- -D warnings PASS
- cargo build --workspace --all-targets --locked PASS
- cargo build --workspace --release --locked PASS
- cargo metadata --locked PASS
- git diff --check PASS
- R3 targeted content/runtime/data/items/monster/levels 52 tests PASS
- 동일 seed의 기존 release-candidate hash baseline test PASS

### 4.2 R4 gate evidence

R4는 문서대로 NOT RUN/미구현이다.

| seed | requested | actual final_turn | final_hash |
| ---: | ---: | ---: | --- |
| 42 | 1000 | 20 | 569bc36895258349 |
| 7 | 1000 | 28 | f1ee87dc33c32533 |
| 1234 | 1000 | 18 | 58762b2adea01615 |

- headless_policy test target은 존재하지 않아 exit 101이었다.
- --policy survival-v1은 현재 CLI에서 unexpected argument로 exit 2였다.
- 이는 README, BUILD_GUIDE, audit_roadmap이 명시한 현재 wait-only runner와 R4 NOT RUN 상태에 부합한다.
- 따라서 R4 미완료를 이번 감사에서 새 regression으로 분류하지 않는다. 전체 프로그램 PASS를 막는 명시적 후속 checkpoint다.

## 5. Pass 3: Security Findings

Critical 또는 Major security finding은 확인되지 않았다.

### 5.1 Verified security evidence

- cargo audit: advisory DB 1160건 로드, Cargo.lock 207 dependency scan, exit 0
- cargo deny check licenses bans sources: bans/licenses/sources 모두 PASS
- dependency source는 crates.io registry로 제한되며 unknown registry/git은 deny다.
- embedded content는 include_str 기반이며 current production bootstrap에서 임의 runtime content path를 받지 않는다.
- unsafe 검색 결과 active source에서 별도 도입은 확인되지 않았다.
- live HTTP LLM transport와 remote bind surface는 R6 미구현 상태다.

### 5.2 Deferred security surfaces

- R4의 runtime root path normalization, atomic save, replay/report path 경계
- R6의 loopback 재검증, redirect/proxy 차단, timeout, body limit, stale response gate
- R7의 provenance 승인과 배포 license scope

이 항목들은 문서에 후속 Phase로 명시되어 있어 현재 finding 또는 Accepted Risk로 오인하지 않는다. 해당 Phase 구현 후에는 독립 Pass 3 재감사가 필요하다.

## 6. Cross-Pass Conflicts

## [XPF-F001] Green verification과 R2 contract PASS 주장이 충돌함

- Related Findings: IMP-F001
- Conflict: Pass 2의 build/test는 모두 통과하지만, 테스트가 spec의 Result/revision contract가 아니라 현재 accepted-bool contract를 검증한다.
- Resolution: green suite는 현재 구현의 내부 일관성 증거로만 인정한다. spec contract 및 R2 checkpoint PASS 증거로 승격하지 않는다.
- Gate Impact: R2 documentation/contract gate HOLD
- Required Fix Before PASS: contract 구현 또는 문서상 명시적 deferral 후 해당 contract 회귀 테스트 추가

## [XPF-F002] R3 runtime PASS와 control-document 상태가 충돌함

- Related Findings: IMP-F002
- Conflict: source/test와 audit_report_1은 R3 완료를 지지하지만 ADR/gap/audit current status는 이전 Hold 상태를 유지한다.
- Resolution: R3 implementation evidence는 Verified로 보존하되 Phase documentation closure는 Needs Documentation Recovery로 둔다.
- Gate Impact: R4 진입 문서 authority HOLD
- Required Fix Before PASS: active decision/gap/audit status 동기화

## 7. Required Fixes Before PASS

1. IMP-F001: R2 typed boundary 계약을 구현하거나, 현재 API를 authority로 확정해 spec·ADR·summary를 일치시킨다.
2. IMP-F002: ADR-0024, G-DATA-002, audit_roadmap current status를 R3 재감사 증거와 동기화한다.
3. IMP-F003: LESSONS_LEARNED의 감사 보고서 계보를 실제 artifact와 맞춘다.
4. 전체 v0.3.0 PASS를 주장하려면 R4~R8 checkpoint를 각각 실행·통과한다.
5. SC-BUILD-02는 동일 commit의 Linux/Windows 원격 CI green 증거가 있어야 PASS다.

## 8. Accepted Risks

없음.

R4~R8 미구현과 원격 CI pending은 Accepted Risk가 아니라 명시적 미완료/후속 checkpoint다.

## 9. Needs Spec Clarification

없음.

IMP-F001의 현재 문서에는 Result/revision/public boundary가 구체적으로 정의되어 있고 IMPLEMENTATION_SUMMARY가 완료까지 주장하므로, 요구 불명확이 아니라 산출물 불일치로 판정했다. Architect가 현재 accepted-bool API를 의도한 것이라면 그 결정을 spec 변경과 ADR로 명시해야 한다.

## 10. Re-audit Checklist

~~~bash
rg -n "GameClient|SessionRevision|SubmitError|ReplayTurnOutcomeV1|action_space" \
  src tests spec.md IMPLEMENTATION_SUMMARY.md DESIGN_DECISIONS.md

rg -n "Partially implemented|bootstrap error boundary pending|expect 경로가 남음|G-DATA-002" \
  DESIGN_DECISIONS.md GAP_CLOSURE_ROADMAP.md audit_roadmap.md

rg -n "aihack_audit_report_[12]\.md" \
  LESSONS_LEARNED.md README.md *.md

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo audit
cargo deny check licenses bans sources
git diff --check
~~~

수동 대조:

- spec 9.1/9.2/14와 실제 public Rust signature가 type 단위로 일치하는지 확인
- R2 checkpoint의 각 체크 항목에 실제 source/test/fixture가 연결되는지 확인
- R3 상태가 spec, summary, ADR, gap, audit roadmap, latest report에서 같은지 확인
- historical report는 current authority와 supersede 관계를 명시하는지 확인

## 11. Remaining Risks

- R2 contract drift가 해소되지 않은 상태에서 R4/R5/R6를 구현하면 API 재작업 범위가 확대된다.
- R4 runner가 아직 requested loop와 accepted turn을 구분하지 않아 1000-turn 성공을 주장할 수 없다.
- R5 workspace, R6 live LLM, R7 provenance/compatibility, R8 release gate는 아직 검증되지 않았다.
- 원격 Linux/Windows CI 결과가 없어 SC-BUILD-02는 pending이다.
- working tree는 감사 전부터 다수의 수정·미추적 파일을 포함했다. 본 감사는 해당 현재 tree를 대상으로 했으며 baseline commit과의 변경 소유권은 판정하지 않았다.

## 12. Final Decision

**HOLD**

현재 source는 clean-room 209-test, build, lint, R3 content regression, supply-chain gate를 통과해 내부 실행 증거가 강하다. 그러나 R2의 active typed contract가 구현되지 않은 채 완료로 표시됐고, R3 완료 상태도 active ADR/gap/audit 문서에 동기화되지 않았다. 두 Major finding이 해소되기 전에는 R2/R3 문서 gate와 다음 Phase 진입을 PASS로 판정할 수 없다.

부분 판정:

| Gate | 판정 |
| --- | --- |
| R0 documentation | HOLD: active contract/state drift |
| R1 build | LOCAL PASS, remote CI pending |
| R2 state/transaction | HOLD: implementation subset verified, typed public contract mismatch |
| R3 content | implementation Verified, documentation recovery required |
| R4 long-run | NOT RUN |
| R5 workspace | NOT RUN |
| R6 local LLM | NOT RUN |
| R7 provenance/compatibility | NOT RUN |
| R8 release | NOT RUN |

재감사는 IMP-F001과 IMP-F002의 변경 영역에 대해 Pass 1을 필수 수행하고, public API/test 변경이 있으면 Pass 2와 replay/save/AI boundary를 함께 다시 실행해야 한다.

## 13. Coder remediation claim: R2/R3 문서 수정 (2026-07-16)

이 절은 최초 HOLD finding에 대응한 코더의 수정 범위와 자체 검증 설명이다. 독립 재감사 판정이 아니며, 이 claim의 검증과 최신 authority는 `audit_report_3.md`가 담당한다.

### 13.1 변경 범위

- IMP-F001: `spec.md` 9.1·9.2·14, `IMPLEMENTATION_SUMMARY.md` R2, ADR-0023을 실제 accepted-bool `TurnOutcome`/`InvariantReport`/replay wire와 일치시켰다. `GameClient`, `SessionRevision`, typed `SubmitError`, replay projection은 R5/R6 target으로 명시적으로 이관했다.
- IMP-F002: ADR-0024를 Implemented로 전환하고, G-DATA-002의 증거·상태를 fallible TUI/headless bootstrap 및 injected missing level/item regression에 맞춰 Verified로 전환했다. `audit_roadmap.md`는 최신 audit lineage와 R2/R3 remediation 상태를 분리해 기록한다.
- IMP-F003: `LESSONS_LEARNED.md`의 존재하지 않는 `aihack_audit_report_*.md` 참조를 실제 `audit_report_1.md`/`audit_report_2.md` lineage와 판정으로 교정했다. README의 current audit link도 `audit_report_2.md`로 갱신했다.

### 13.2 Coder-provided verification evidence

- `rg -n "GameClient|SessionRevision|SubmitError|ReplayTurnOutcomeV1|action_space" src tests spec.md IMPLEMENTATION_SUMMARY.md DESIGN_DECISIONS.md` 결과에서 R2 완료 계약으로 남은 `GameClient`/`SubmitError`/`ReplayTurnOutcomeV1` 선언은 0건이다. `SessionRevision`은 spec 9.4의 R6 target과 LLM target 문맥에만 남는다.
- `rg -n "Partially implemented|bootstrap error boundary pending|expect 경로가 남음" DESIGN_DECISIONS.md GAP_CLOSURE_ROADMAP.md audit_roadmap.md` 결과에서 active R3 pending claim은 0건이다. `G-DATA-002`는 Verified row로 남아 재감사 근거를 보존한다.
- `rg -n "aihack_audit_report_[12]\\.md" LESSONS_LEARNED.md README.md` 결과는 0건이다. 이 report의 최초 감사 본문은 historical evidence로 옛 filename을 인용하며, numbered audit link와 Final Decision lineage는 실제 파일과 일치한다.
- source/test boundary는 기존 R3 regression (`tests/content_validation.rs`)과 R2 transaction/replay/save-load test를 재실행해 확인한다.

### 13.3 Claimed finding disposition

| Finding | Re-audit status | 근거 |
| --- | --- | --- |
| IMP-F001 | Fixed (documentation authority aligned) | 현재 public contract와 R5/R6 target의 시점이 분리됨 |
| IMP-F002 | Fixed | ADR, gap register, audit index가 R3 LOCAL PASS evidence와 일치 |
| IMP-F003 | Fixed | audit report filename과 verdict lineage가 실제 artifact에 일치 |
| XPF-F001 | Resolved | green suite와 R2 contract claim의 충돌을 제거 |
| XPF-F002 | Resolved | R3 runtime evidence와 control-document status를 동기화 |

### 13.4 Coder claim conclusion

**R2/R3 DOCUMENTATION REMEDIATION CLAIM — independent audit pending at time of writing**

이 claim은 R2/R3 contract/state drift의 수정 범위에 한정한다. 전체 v0.3.0 program은 R4~R8이 NOT RUN이고 SC-BUILD-02의 원격 Linux/Windows CI evidence도 pending이므로 release/program PASS를 의미하지 않는다. 독립 재감사 결과와 다음 implementation gate는 `audit_report_3.md` 및 최신 audit roadmap을 따른다.
