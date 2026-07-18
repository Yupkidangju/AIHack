# AIHack D3D R6 Re-audit Report 11

감사 기준: `AI_AUDIT_DOC_STANDARD.md`

감사 유형: `audit_report_10.md`의 IMP-F009/010/011, DBG-F004 시정 후 독립 재감사

감사 일자: 2026-07-18 (Asia/Seoul)

감사 대상: 현재 working tree의 R6-6 public/schema contract 및 재현 evidence 시정, 연결 문서·테스트·빌드·공급망·보안 경계, R1~R6 전체 회귀

기준 commit: `bc3363d` (`main`, `origin/main`) + R6/R6-6 remediation working tree

환경: Linux 7.0.0-28-generic x86_64, rustc 1.94.1, cargo 1.94.1

감사 중 소스·설정·기존 문서 수정: 없음

이번 감사가 생성한 파일: `audit_report_11.md`

## 1. 감사 요약

최종 판정: **PASS — audit_report_10 remediation and R6 checkpoint closed**

보고서 10의 IMP-F009, IMP-F010, IMP-F011, DBG-F004가 모두 시정됐다. public `LlmRequestInput`은 schema version, projection, 독립 ActionSpace, request kind를 명시하고 request schema 0/2와 bounds/canonical oversize를 외부 work 전에 typed error로 거부한다. response envelope schema 0/2도 TUI payload 수용 전에 거부된다. public error/command enum의 non-exhaustive 정책과 G-LLM lifecycle 상태가 정렬됐다.

일회성 PTY 기록은 저장소 보존 deterministic loopback fixture와 재실행 script로 대체됐다. 독립 실행 결과 success, timeout, stale, connection-refused 및 pending-request exit가 모두 통과했고 terminal restore가 worker wait보다 먼저 관찰됐다.

R6 표적 75개, TUI package 6개, 전체 workspace 300개 테스트와 fmt, metadata, check, clippy, debug/release build, RustSec, cargo-deny, dependency tree, CLI/hash smoke가 모두 통과했다. 새 implementation/debug/security finding은 없으며 XPF-F007의 green runtime과 public-contract 충돌도 해소됐다.

| 구분 | 결과 |
| --- | --- |
| IMP-F009 Re-audit #1 | Verified |
| IMP-F010 Re-audit #1 | Verified |
| IMP-F011 Re-audit #1 | Verified |
| DBG-F004 Re-audit #1 | Verified |
| XPF-F007 Re-audit #1 | Resolved / Verified |
| R6 표적 test | PASS, 75 tests |
| Full workspace test | PASS, 300 tests |
| PTY/loopback scripts | PASS, 5 scenarios |
| Critical / Major / Minor open | 0 / 0 / 0 |
| 신규 Security finding | 0건 |
| R6 checkpoint | **PASS** |

이 PASS는 보고서 10 remediation과 R6 checkpoint 범위에 대한 판정이다. R7 provenance/compatibility, R8 release, SC-BUILD-02 원격 CI는 pending/NOT RUN이므로 전체 프로그램 또는 release PASS를 의미하지 않는다.

## 2. Audit Scope

### 2.1 시정 범위

- `crates/aihack-llm/src/service.rs`: versioned public request/projection, synchronous request validation, response schema gate
- `config.rs`, `worker.rs`, `transport.rs`, `decision.rs`, `narrative.rs`: typed schema/input error 및 public enum stability
- `apps/aihack-tui/src/tui/mod.rs`: response schema 선검증과 non-exhaustive consumer
- `tests/llm_transport.rs`, `tests/llm_tui_integration.rs`: public shape, schema 0/2, bounds/oversize, TUI rejection 회귀
- `scripts/r6_loopback_fixture.py`, `scripts/r6_pty_matrix.sh`, `scripts/r6_pending_exit_smoke.sh`: deterministic loopback 및 실제 PTY 재현 자산
- `docs/R6_MANUAL_MATRIX.md`, `BUILD_GUIDE.md`: exact 재현 명령과 결과
- `GAP_CLOSURE_ROADMAP.md`, `IMPLEMENTATION_SUMMARY.md`, `audit_roadmap.md`, README/CHANGELOG/ADR/lessons: remediation authority와 audit-pending 상태

### 2.2 확인한 케이스

- public DTO field shape와 observation schema 보존
- request schema 0/1/2 및 response envelope schema 0/1/2
- action space 64개 초과와 canonical request 32,768 bytes 초과의 synchronous rejection
- provider external work 전에 typed input failure
- TUI가 unsupported envelope payload를 표시하거나 core에 반영하지 않음
- public error/command enum의 `#[non_exhaustive]`와 downstream wildcard
- G-LLM-001~004의 단일 `Implemented / Audit HOLD` pre-verdict 상태
- success, timeout, stale, connection refused PTY semantic output
- pending request 중 terminal restore 선행, canonical/echo 복원, bounded process exit
- loopback-only fixture, secret/external network 부재, 고유 tmux/mktemp 및 cleanup trap
- R1~R5 deterministic/runtime regression과 R6 전체 회귀

## 3. Excluded Scope

- 실제 언어 모델 추론 smoke: R6 필수 gate가 아닌 비차단 선택 검증
- R7 NH367 provenance/compatibility 및 법적 승인: NOT RUN
- R8 version/release/packaging: NOT RUN
- SC-BUILD-02 Linux/Windows 원격 CI 실제 green evidence: pending
- 외부 advisory DB 최신 fetch: 설치된 DB를 `cargo audit --no-fetch`로 검사
- 배포, Git commit/push readiness, 법률 자문
- 복수 terminal emulator와 장시간 사용자 UX 수동 검수

## 4. 실행 명령과 결과

### 4.1 표적 계약·PTY 검증

| 명령/검사 | 결과 |
| --- | --- |
| R6 LLM/UI 표적 9개 test target | PASS, 75 tests |
| `cargo test -p aihack-tui --locked --bin aihack --test tui_contract` | PASS, 6 tests |
| `scripts/r6_pty_matrix.sh` | PASS: success, timeout, stale, down |
| `scripts/r6_pending_exit_smoke.sh` | PASS: restore-before-worker-wait, 289ms |
| `bash -n` on shell scripts | PASS |
| Python fixture `--help`/argument boundary | PASS |
| public enum/static document state search | PASS |

### 4.2 전체 회귀·품질·공급망 검증

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS, 8 workspace members |
| `cargo check --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 300 tests, 실패 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo build --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --locked` | PASS |
| `cargo tree -p aihack-core --locked` | PASS, UI/terminal/network dependency 없음 |
| `cargo tree -p aihack-llm --locked` | PASS, network dependency가 LLM adapter에 격리 |
| `cargo audit --no-fetch` | PASS, 1160 advisories로 267 dependencies scan |
| `cargo deny check licenses bans sources` | PASS |
| `git diff --check` | PASS |
| TUI/headless `--help` | PASS |
| headless seed 42, survival-v1, target 10 | PASS, accepted 10, hash `e7d30d72027a39c0` |

build/package/advisory DB lock 대기 메시지는 모든 명령이 정상 완료하고 exit 0이므로 제품 실패로 분류하지 않았다.

## 5. Pass 1: Implementation Compliance Findings

### [IMP-F009 Re-audit #1] R6 public request와 schema-version hard boundary가 확정 명세와 다름

- Pass: Implementation Compliance
- Pattern: IMP-001, IMP-003
- Area: R6 public DTO, request/response schema validation, typed input error
- Severity: Major
- Status: **Verified**
- Summary: public DTO와 schema/version/error timing이 `spec.md` 계약에 맞게 시정됐다.
- Evidence:
  - `LlmObservationView`와 `LlmRequestInput`은 명세가 요구한 public projection, `schema_version`, `SessionRevision`, 독립 `ActionSpace`, `LlmRequestKind`를 제공한다 (`crates/aihack-llm/src/service.rs:25-80`).
  - `from_observation`은 source observation의 schema version을 보존하며 projection 과정에서 v1로 재표기하지 않는다 (`service.rs:66-79`).
  - enqueue는 schema mismatch, projection/action bounds, soft text, canonical request size를 request ID 생성과 channel send 전에 검증한다 (`service.rs:168-200`, `250-282`).
  - request schema 0/2와 observation schema 2가 `UnsupportedSchema`, action/canonical oversize가 synchronous `InvalidInput { PayloadTooLarge }`로 실패한다 (`tests/llm_transport.rs:401-484`).
  - TUI는 response envelope schema를 request correlation/payload보다 먼저 검사하고 invalid version이면 outstanding을 제거한 뒤 payload를 수용하지 않는다 (`apps/aihack-tui/src/tui/mod.rs:241-265`).
  - schema 2 narrative가 render되지 않고 revision도 유지되는 회귀 테스트가 통과했다 (`tests/llm_tui_integration.rs:254-287`).
- Expected: 명시된 public DTO와 request/response version hard boundary가 external work 및 payload acceptance 전에 동작한다.
- Actual: 기대와 일치한다.
- Impact: v0.3.x public compatibility와 unsupported-schema fail-closed 경계가 복구됐다.
- Suggested Fix: 없음.
- Re-audit Method: public shape, schema 0/1/2, synchronous bounds/oversize, TUI rejection tests와 full regression 재실행.
- Owner: Auditor verified
- Notes: IMP-F009를 종결한다.

### [IMP-F010 Re-audit #1] public error와 command enum의 non-exhaustive 정책이 부분 적용됨

- Pass: Implementation Compliance
- Pattern: IMP-001
- Area: `aihack-llm` public API stability
- Severity: Minor
- Status: **Verified**
- Summary: 명세가 지정한 public error/command enum에 non-exhaustive 정책이 일관되게 적용됐다.
- Evidence:
  - `LlmRequestKind`, `LlmConfigError`, `LlmInputCode`, `DecisionGateError`, `DecisionError`, `NarrativeError`, `LlmEnqueueError`, `LlmResponseError`, `LlmValidationCode` 모두 선언 직전에 `#[non_exhaustive]`가 있다.
  - TUI와 transport consumer는 새 variant를 허용하는 wildcard branch를 유지한다 (`apps/aihack-tui/src/tui/mod.rs:208-231`, `transport.rs:379-391`).
  - 전체 workspace clippy와 compile/test가 통과했다.
- Expected: 공개 error/command enum 확장이 downstream exhaustive match를 깨지 않는다.
- Actual: 기대와 일치한다.
- Impact: public API forward compatibility 정책이 복구됐다.
- Suggested Fix: 없음.
- Re-audit Method: public enum 정적 대조와 downstream compile/clippy/full test.
- Owner: Auditor verified
- Notes: IMP-F010을 종결한다.

### [IMP-F011 Re-audit #1] G-LLM 현재 상태가 동일 control 문서에서 Implemented/Closed/Open으로 갈림

- Pass: Implementation Compliance
- Pattern: IMP-004
- Area: G-LLM lifecycle authority
- Severity: Minor
- Status: **Verified**
- Summary: 독립 재감사 전 lifecycle 상태가 `Implemented / Audit HOLD`로 통일되고 과거 Closed/Open 충돌이 제거됐다.
- Evidence:
  - G-LLM-001~004 table이 모두 `Implemented / Audit HOLD`와 R6-6/re-audit 대기를 사용한다 (`GAP_CLOSURE_ROADMAP.md:50-53`).
  - 상세 문단은 `Closed`를 독립 재감사 PASS 이후에만 사용한다고 명시한다 (`GAP_CLOSURE_ROADMAP.md:189`).
  - 현재 완료 범위도 동일 상태와 남은 remote CI/R7/R8을 구분한다 (`GAP_CLOSURE_ROADMAP.md:244-246`).
  - implementation summary, audit roadmap, README, ADR, changelog가 coder-local PASS와 independent verdict를 분리한다.
- Expected: 구현 완료, local evidence, independent audit closure가 하나의 상태 모델로 표현된다.
- Actual: 기대와 일치한다.
- Impact: 후속 작업자가 local remediation과 독립 PASS를 혼동하지 않는다.
- Suggested Fix: 없음. 본 `audit_report_11.md`가 R6 independent PASS의 closure authority이며, 다음 정상 문서 동기화에서 pending label을 Closed/PASS로 소비할 수 있다.
- Re-audit Method: table, detail, current scope, implementation summary, audit roadmap, README/ADR의 authority 대조.
- Owner: Auditor verified
- Notes: IMP-F011을 종결한다. 감사 전 `Audit HOLD` 표기는 자체 PASS 선점을 막기 위한 올바른 pre-verdict 상태였다.

### 5.4 Verified implementation evidence

- 보고서 10에서 확인한 loopback/DNS/proxy/redirect/timeout/size/queue/stale/approval/soft-presentation 안전 경계 유지
- R6 public DTO와 version error가 한 schema version source를 사용
- request validation이 worker/network 이전에 완료
- response validation이 TUI rendering/core effect 이전에 완료
- R1~R5 전체 회귀와 보고서 9 Verified 상태 유지

## 6. Pass 2: Debug / Engineering Quality Findings

### [DBG-F004 Re-audit #1] prescribed pending-request exit smoke와 PTY fixture 재현 자산이 없음

- Pass: Debug / Engineering Quality
- Pattern: TEST-001, BUILD-001
- Area: deterministic loopback fixture, live PTY matrix, terminal cleanup
- Severity: Minor
- Status: **Verified**
- Summary: 재현 불가능했던 일회성 evidence가 저장소 보존 fixture와 exact scripts로 대체됐다.
- Evidence:
  - Python fixture는 `127.0.0.1`에만 bind하고 request kind별 deterministic OpenAI-compatible JSON과 configurable delay/request count를 제공한다 (`scripts/r6_loopback_fixture.py:13-82`).
  - PTY matrix는 고유 PID 기반 tmux session과 `mktemp`를 사용하며 success/timeout/stale/down semantic text를 검사하고 trap에서 자신이 만든 자원만 정리한다 (`scripts/r6_pty_matrix.sh:4-30`, `70-103`).
  - 독립 실행 결과 `[N] Dismiss`, `LLM: TIMEOUT`, `LLM: STALE`, `LLM: DOWN` 네 경로가 모두 PASS했다.
  - pending-exit smoke는 실제 WAIT 상태에서 Q를 입력하고 exit status 생성 전 화면 복원, `icanon`/`echo`, bounded exit를 검사한다 (`scripts/r6_pending_exit_smoke.sh:44-90`).
  - TUI 구현은 cursor/raw/alternate-screen 복원을 먼저 수행한 뒤 `shutdown_with_grace(250ms)`를 호출한다 (`apps/aihack-tui/src/tui/mod.rs:799-810`).
  - 독립 실행은 restore-before-worker-wait와 289ms process exit를 PASS했다.
  - `docs/R6_MANUAL_MATRIX.md`와 `BUILD_GUIDE.md`에 exact 재현 명령, fixture 제한, semantic 결과가 기록됐다.
- Expected: 다음 감사자가 외부 provider/secret 없이 terminal·timing failure matrix와 pending exit를 재현한다.
- Actual: 기대와 일치한다.
- Impact: PTY 및 terminal recovery 회귀가 반복 가능한 evidence로 전환됐다.
- Suggested Fix: 없음.
- Re-audit Method: 두 script 독립 실행, shell syntax, fixture loopback/cleanup 정적 검토.
- Owner: Auditor verified
- Notes: DBG-F004를 종결한다. 실제 model smoke는 이 finding의 요구가 아니다.

### 6.2 Verified engineering evidence

- 표적 tests가 보고서 10의 실제 실패 모드를 직접 이름 붙이고 검증한다.
- 전체 workspace 300 tests와 deterministic long-run이 통과했다.
- fmt, check, clippy `-D warnings`, debug/release build가 통과했다.
- PTY scripts는 raw screen 전체가 아니라 안정된 semantic 상태를 검사한다.
- 새 dependency 없이 Python 표준 라이브러리와 기존 tmux/build 환경만 사용한다.

## 7. Pass 3: Security Findings

새 Security finding 없음.

Verified evidence:

- 기존 default-disabled, exact loopback host, 모든 resolved IP loopback 재검사, pinned address, redirect/proxy off 경계 유지
- request/response version, size, text/control, action, confidence, reason code 검증 유지
- fixture는 `127.0.0.1`만 bind하며 external network/API key/Authorization 처리가 없음
- scripts는 PID가 포함된 고유 session과 `mktemp`를 쓰고 trap에서 정확한 session/process/temp directory만 정리
- secret 정적 검색 결과는 credential/query rejection test와 일반 용어만 존재
- `cargo audit --no-fetch`, cargo-deny licenses/bans/sources PASS
- `aihack-core`에 network/UI dependency 없음

## 8. Cross-Pass Conflicts

### [XPF-F007 Re-audit #1] green runtime evidence와 R6 public-contract 완료 주장이 충돌함

- Pass: Cross-Pass
- Pattern: IMP-003, TEST-001
- Area: R6 checkpoint authority
- Severity: Major
- Status: **Verified / Resolved**
- Summary: runtime safety, public/schema contract, 재현 evidence와 문서 authority가 같은 PASS 결론을 지지한다.
- Evidence: IMP-F009/010/011과 DBG-F004가 Verified됐고, 표적/전체/PTY/보안 gate가 모두 통과했다.
- Expected: R6 PASS가 runtime, public compatibility, evidence reproducibility의 결합 결과다.
- Actual: 기대와 일치한다.
- Impact: 보고서 10의 R6 HOLD 사유가 제거됐다.
- Suggested Fix: 없음.
- Re-audit Method: 연결 finding과 전체 gate 동시 검증.
- Owner: Auditor verified
- Notes: XPF-F007을 종결한다.

## 9. Required Fixes Before PASS

보고서 10 remediation 및 R6 checkpoint 범위에는 없음.

`audit_report_11.md`가 independent closure authority다. active 문서의 pre-verdict `Audit HOLD` 표기는 다음 계획된 문서 동기화에서 R6 PASS/Closed로 전환할 수 있으며 추가 코드 재감사 조건이 아니다.

## 10. Accepted Risks

없음.

실제 model smoke 비수행, 원격 CI pending, R7/R8 NOT RUN은 숨은 면제가 아니라 명시된 후속/제외 범위다.

## 11. Needs Spec Clarification

없음.

## 12. Re-audit Checklist

IMP-F009/010/011, DBG-F004 및 XPF-F007은 이번 보고서에서 종결됐다. 이후 R6 public/schema/PTY 경계를 변경하면 다음을 재실행한다.

```bash
cargo test -p aihack --locked --test llm_transport --test llm_tui_integration
cargo test -p aihack-tui --locked --bin aihack --test tui_contract
scripts/r6_pty_matrix.sh
scripts/r6_pending_exit_smoke.sh
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
cargo audit --no-fetch
cargo deny check licenses bans sources
git diff --check
```

정적 확인:

```bash
rg -n -B2 '^pub enum' crates/aihack-llm/src
rg -n 'UnsupportedSchema|LLM_SCHEMA_VERSION|PayloadTooLarge' crates/aihack-llm/src tests
rg -n 'G-LLM-00[1-4]|R6.*(HOLD|PASS|Closed|Implemented)' \
  GAP_CLOSURE_ROADMAP.md IMPLEMENTATION_SUMMARY.md audit_roadmap.md README.md
```

## 13. Remaining Risks

- SC-BUILD-02 Linux/Windows 원격 CI evidence pending
- R7 provenance/compatibility와 R8 release NOT RUN
- 실제 model provider 호환성은 비필수 선택 검증
- 설치된 advisory DB만 사용했으므로 최신 원격 RustSec 상태는 CI/온라인 감사에서 확인 필요
- working tree는 R6/R6-6 coder 수정과 감사 보고서를 포함하며 commit readiness는 이번 감사 범위가 아님
- 최종 release 전 복수 모델 또는 인간 교차감사 필요

## 14. Final Decision

**PASS — audit_report_10 remediation and R6 checkpoint closed**

| Gate | 판정 |
| --- | --- |
| 보고서 9 IMP-F008 / R1~R5 | 기존 Verified/PASS 유지 |
| IMP-F009 public/schema boundary | Verified |
| IMP-F010 public enum stability | Verified |
| IMP-F011 lifecycle authority | Verified |
| DBG-F004 PTY evidence | Verified |
| XPF-F007 | Resolved |
| R6 local transport/failure safety | PASS |
| R6 stale/action/explicit approval | PASS |
| R6 soft presentation-only | PASS |
| R6 public/schema compatibility | PASS |
| R6 independent checkpoint | **PASS** |
| R7/R8/remote CI | pending / NOT RUN |
| 전체 프로그램/release | 아직 PASS 대상 아님 |

보고서 10에서 요구한 추가 코드 재수정은 더 이상 필요하지 않다. 다음 단계는 본 PASS를 active 상태 문서에 소비한 뒤 프로젝트 roadmap에 따라 R7 또는 승인된 후속 Phase를 진행하는 것이다.

코드·설정·기존 문서는 수정하지 않았고 감사 보고서만 생성했다.
