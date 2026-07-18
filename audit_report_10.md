# AIHack D3D R6 Comprehensive Audit Report 10

감사 기준: `AI_AUDIT_DOC_STANDARD.md`

감사 유형: R6 구현 완료 주장 후 독립 종합감사

감사 일자: 2026-07-18 (Asia/Seoul)

감사 대상: 현재 working tree의 R6 local LLM transport·worker·revision/action gate·soft adjudication·TUI 통합·PTY 보정, 연결 문서·테스트·빌드·공급망·보안 경계, R1~R5 전체 회귀

기준 commit: `bc3363d` (`main`, `origin/main`) + R6 terminal 보정 working tree

환경: Linux 7.0.0-28-generic x86_64, rustc 1.94.1, cargo 1.94.1

감사 중 소스·설정·기존 문서 수정: 없음

이번 감사가 생성한 파일: `audit_report_10.md`

## 1. 감사 요약

최종 판정: **HOLD — R6 runtime safety passes, public contract closure incomplete**

R6의 핵심 실행 경로는 기능적으로 건전하다. loopback-only endpoint, DNS 재검사, pinned address, redirect/system proxy 차단, request/response 크기 제한, timeout, bounded worker, opaque request ID, stale revision·ActionSpace 재검증, Y 명시 승인, soft verdict의 presentation-only 경계가 코드와 구체적 테스트에서 확인됐다. R6 표적 69개와 전체 workspace 294개 테스트, fmt, metadata, check, clippy, debug/release build, RustSec, cargo-deny가 모두 통과했다. 독립 120x36 tmux PTY에서도 disabled 상태의 `Enter`, `Enter`, `G/A/J`, turn 0, `LLM: OFF`, Q clean exit를 확인했다.

그러나 `spec.md`가 확정한 R6 public request/schema contract와 실제 `aihack-llm` public API가 다르다. 실제 서비스 입력에는 `schema_version`과 독립 `action_space`가 없고 public `LlmObservationView` 대신 전체 `Observation`을 받는다. 입력 schema version을 거부하는 경로가 없으며 response envelope의 version도 TUI가 검사하지 않는다. oversized request 역시 문서의 synchronous `LlmEnqueueError::InvalidInput { PayloadTooLarge }`가 아니라 worker의 asynchronous response error로 바뀌었다. public error/command enum의 `#[non_exhaustive]` 정책도 일부만 적용됐다. 이 상태는 green test와 무관하게 명시된 public compatibility hard boundary를 충족하지 않으므로 Major finding이 해소되기 전 R6 PASS로 전환할 수 없다.

| 구분 | 결과 |
| --- | --- |
| R6 표적 test | PASS, 69 tests |
| Full workspace test | PASS, 294 tests |
| Build/lint/supply-chain | PASS |
| 독립 실제 PTY | 부분 PASS, disabled/clean exit 재현 |
| Critical / Major / Minor | 0 / 1 / 3 |
| Security finding | 0건 |
| R6 독립 감사 | **HOLD** |

`audit_report_9.md`의 IMP-F008 및 R1~R5 remediation PASS는 유지된다. 이번 HOLD는 R6 checkpoint와 그 연결 계약에 대한 판정이며, 과거 Verified finding을 되돌리지 않는다.

## 2. Audit Scope

### 2.1 프로젝트 인벤토리

- 프로젝트 경로: `/mnt/Projects_SSD/rust/AIHack`
- 유형: Rust Cargo workspace 기반 CLI/TUI 턴제 로그라이크
- workspace: root compatibility facade, `crates/` 5개, `apps/` 2개, 총 8 members
- R6 핵심: `crates/aihack-llm`, `crates/aihack-ai-contract`, `apps/aihack-tui`, root compatibility facade
- tests: root `tests/llm_*.rs`, UI integration tests, crate/app contract tests, 기존 R1~R5 회귀
- dependency/policy: `Cargo.toml`, `Cargo.lock`, member manifests, `deny.toml`, `rust-toolchain.toml`
- control 문서: `spec.md`, `IMPLEMENTATION_SUMMARY.md`, `DESIGN_DECISIONS.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`
- 실행 문서: `README.md`, `BUILD_GUIDE.md`, `docs/R6_MANUAL_MATRIX.md`
- 감사 계보: `audit_report_1.md`~`audit_report_9.md`

### 2.2 R6 변경 범위

- `crates/aihack-llm`: config, loopback HTTP transport, bounded service/worker, narrative, decision, soft adjudication
- `crates/aihack-ai-contract`: LLM payload DTO
- `apps/aihack-tui`: G/A/J/Y/N/R CTA, status/fallback, Judge input, stale/approval 처리, terminal restore와 accessibility flags
- root tests: transport, narrative, decision, revision, soft adjudication, TUI integration, layout/input/runtime
- current working tree: 18개 수정 파일과 `docs/R6_MANUAL_MATRIX.md` 1개 미추적 파일
- 문서 상태: local gate 완료, independent R6 audit pending/READY FOR AUDIT

### 2.3 확인한 문서·파일·검사 케이스

- `AI_AUDIT_DOC_STANDARD.md`의 3-pass, finding, severity, phase gate, 재감사 규칙
- `audit_report_9.md`의 이전 PASS 범위와 R6 NOT RUN 경계
- `spec.md` 9.4~9.6의 public DTO, endpoint, queue, timeout, schema/version, validation 계약
- R6 Task R6-1~R6-5와 checkpoint, SC-LLM-01~03, G-LLM-001~004
- loopback endpoint 및 resolve 결과, redirect/proxy, timeout, request/response bound
- disabled, busy, unavailable, timeout, invalid JSON, oversized input/output, stale, invalid action, unknown request ID
- narrative/decision/soft payload strict parsing과 control/ANSI 차단
- TUI G/A/J/Y/N/R, retry cooldown, explicit approval, fallback, mouse/keyboard text, accessibility, terminal minimum
- public request/response DTO와 error/command enum stability
- 전체 workspace regression, deterministic long-run, build, lint, dependency/supply-chain
- 독립 120x36 tmux PTY disabled/turn 0/status/clean exit

## 3. Excluded Scope

- 실제 언어 모델 추론 smoke: `spec.md`가 R6 필수 gate에서 명시적으로 제외하므로 NOT REQUIRED
- 저장소에 보존되지 않은 일회성 success/timeout/stale PTY fixture의 독립 재실행: NOT REPRODUCIBLE
- R7 NH367-C001..C010 provenance/compatibility와 법적 승인: NOT RUN
- R8 v0.3.0 version/release/packaging: NOT RUN
- SC-BUILD-02 Linux/Windows 원격 CI 실제 green evidence: pending
- 외부 advisory DB 갱신: 설치된 DB를 `cargo audit --no-fetch`로만 검사
- 법률 자문, 배포, Git commit/push readiness
- 장시간 실제 사용자 UX와 복수 terminal/emulator 시각 검수

## 4. 실행 명령과 결과

### 4.1 R6 표적 검증

| 검사 | 결과 |
| --- | --- |
| R6 LLM + UI 표적 9개 test target | PASS, 69 tests |
| `llm_transport` | PASS, 17 tests |
| `llm_revision_gate` | PASS, 9 tests |
| `llm_soft_adjudication` | PASS, 5 tests |
| `llm_tui_integration` | PASS, 9 tests |
| `aihack-tui` binary + `tui_contract` | 전체 workspace에서 PASS, 6 tests |
| 독립 120x36 tmux PTY, LLM disabled | PASS: Playing, turn 0, `LLM: OFF`, G/A/J 후 turn 유지, Q clean exit |

첫 sandbox 실행에서 `llm_transport`의 local `TcpListener::bind` 6건이 `Operation not permitted`로 실패했다. 동일 명령을 loopback socket 권한이 있는 환경에서 재실행하자 17건 전부 통과했으므로 저장소 실패가 아닌 감사 환경 제한으로 분류했다.

### 4.2 전체 회귀·품질·공급망 검증

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS, 8 workspace members |
| `cargo check --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 294 tests, 실패 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo build --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --locked` | PASS |
| `cargo tree -p aihack-core --locked` | PASS, UI/terminal/network dependency 없음 |
| `cargo tree -p aihack-llm --locked` | PASS, network dependency가 LLM adapter에 격리 |
| `cargo audit --no-fetch` | PASS, 1160 advisories로 267 dependencies scan |
| `cargo deny check licenses bans sources` | PASS |
| `git diff --check` | PASS |
| TUI/headless `--help` | PASS, R6 accessibility flags와 기존 headless flags 유지 |

`cargo audit --no-fetch`는 crates.io package-cache lock warning을 냈지만 설치된 advisory DB로 scan을 완료하고 exit 0이었다.

## 5. Pass 1: Implementation Compliance Findings

### [IMP-F009] R6 public request와 schema-version hard boundary가 확정 명세와 다름

- Pass: Implementation Compliance
- Pattern: IMP-001, IMP-003
- Area: `spec.md` 9.4~9.6, `crates/aihack-llm/src/service.rs`, `config.rs`, `transport.rs`, TUI response consumer
- Severity: **Major**
- Status: **Needs Fix**
- Summary: public service DTO, input error 시점, schema-version rejection이 승인된 R6 contract와 일치하지 않는다.
- Evidence:
  - 명세는 `LlmRequestInput`에 `schema_version`, `SessionRevision`, public `LlmObservationView`, 독립 `ActionSpace`, `LlmRequestKind`를 요구한다 (`spec.md:285-341`).
  - 실제 public type은 `ClientRevision`, 전체 `Observation`, `LlmRequestKind` 3개 field만 갖는다. `schema_version`과 독립 `action_space`가 없고 public `LlmObservationView`도 없다 (`crates/aihack-llm/src/service.rs:24-29`).
  - 명세의 `LlmEnqueueError`에는 `InvalidEndpoint`, `InvalidModel`이 있고 `LlmInputCode`에는 `PayloadTooLarge`가 있다 (`spec.md:354-368`). 실제 `LlmInputCode`에는 세 text 오류만 있고 `PayloadTooLarge`가 없다 (`crates/aihack-llm/src/config.rs:37-42`).
  - `enqueue`는 soft text만 검증하고 observation schema/bounds/canonical size는 검증하지 않는다 (`crates/aihack-llm/src/service.rs:117-150`). size/bounds 오류는 worker transport까지 진행한 뒤 `LlmResponseError::InvalidSchema { PayloadTooLarge }`로 비동기 반환된다 (`crates/aihack-llm/src/transport.rs:179-190`, `297-305`).
  - internal wire는 항상 `schema_version: 1`을 새로 기록하고 projection에서 input `Observation.schema_version`을 제외한다 (`crates/aihack-llm/src/transport.rs:179-187`, `378-401`). 따라서 caller가 unsupported observation version을 전달해도 version mismatch를 거부하지 않고 v1 wire로 다시 표기할 수 있다.
  - 명세는 version mismatch를 `UnsupportedSchema { expected, actual }`로 실패하도록 요구한다 (`spec.md:509-518`). R6 crate·TUI·R6 tests에는 `UnsupportedSchema` 경로가 없고 TUI는 public `LlmResponseEnvelope.schema_version`도 검사하지 않는다 (`apps/aihack-tui/src/tui/mod.rs:233-255`).
  - 현재 tests는 축소된 실제 DTO를 직접 구성하므로 문서의 public shape 및 unsupported-version behavior를 검증하지 않는다 (`tests/llm_transport.rs:343-397`).
- Expected: public DTO와 typed error가 `spec.md`의 확정 계약을 구현하고 request/envelope version mismatch가 external work 또는 UI acceptance 전에 typed failure가 된다.
- Actual: transport 내부 wire만 v1로 고정되며 public input과 TUI consumer에는 version gate가 없다. public API shape와 oversized-input error 시점도 명세와 다르다.
- Impact: public consumer가 승인된 DTO를 사용할 수 없고, 향후 versioned producer/adapter가 unsupported data를 v1로 오인할 수 있다. green runtime test가 public compatibility를 증명하지 못하므로 R6 완료 및 v0.3.x 안정성 주장을 신뢰할 수 없다.
- Suggested Fix: `spec.md` shape에 맞춰 validated public request projection과 explicit `schema_version`/`action_space`를 구현하고, enqueue 및 response acceptance 양쪽에 `UnsupportedSchema` typed gate를 추가한다. `PayloadTooLarge`를 문서화된 input error로 반환하고 compile-time public contract fixture, request/envelope version 0/2 negative tests, oversize enqueue timing test를 추가한다. 현재 축소 API가 의도된 새 결정이라면 코드에 맞추기 전에 `spec.md`와 ADR을 승인된 변경으로 먼저 갱신해야 한다.
- Re-audit Method: public API compile fixture, schema 0/1/2 request·response tests, oversized request enqueue test, R6 표적 69개 및 full workspace 재실행, TUI mismatch response가 core/UI payload를 수용하지 않는지 확인.
- Owner: Coder
- Notes: 실제 provider response payload에 provider-owned request ID/revision을 추가하라는 finding이 아니다. 명세대로 application-owned request/envelope version boundary를 검증하라는 finding이다.

### [IMP-F010] public error와 command enum의 non-exhaustive 정책이 부분 적용됨

- Pass: Implementation Compliance
- Pattern: IMP-001
- Area: `aihack-llm` public API stability
- Severity: Minor
- Status: **Needs Fix**
- Summary: 명세의 `#[non_exhaustive]` 정책이 일부 R6 enum에만 적용됐다.
- Evidence:
  - 정책은 public error와 command enum에 `#[non_exhaustive]`를 요구한다 (`spec.md:509-515`).
  - `LlmEnqueueError`, `LlmResponseError`, `LlmValidationCode`에는 attribute가 있다 (`worker.rs:33-40`, `transport.rs:31-57`).
  - command 역할의 `LlmRequestKind`와 public error인 `LlmConfigError`, `LlmInputCode`, `DecisionGateError`, `DecisionError`, `NarrativeError`에는 attribute가 없다 (`config.rs:22-42`, `decision.rs:26-29`, `287-292`, `narrative.rs:29-35`).
- Expected: 공개 error/command enum이 모두 forward-compatible하고 외부 match가 wildcard branch를 가진다.
- Actual: 일부 enum에 새 variant가 추가되면 downstream exhaustive match가 깨지는 공개 계약이다.
- Impact: 현재 동작 오류는 아니지만 v0.3.x에서 error/command 확장이 source-breaking change가 될 수 있다.
- Suggested Fix: 대상 enum에 `#[non_exhaustive]`를 일관되게 적용하고 root/TUI consumer match와 public contract compile test를 갱신한다.
- Re-audit Method: `rg -n -B2 '^pub enum'` 정적 대조, downstream wildcard compile test, clippy/full workspace 재실행.
- Owner: Coder
- Notes: source/status 표현 enum 전체에 무차별 적용하라는 뜻이 아니라 명세가 지정한 error와 command 범위를 우선 닫는다.

### [IMP-F011] G-LLM 현재 상태가 동일 control 문서에서 Implemented/Closed/Open으로 갈림

- Pass: Implementation Compliance
- Pattern: IMP-004
- Area: `GAP_CLOSURE_ROADMAP.md`
- Severity: Minor
- Status: **Needs Documentation Recovery**
- Summary: G-LLM-001~004의 lifecycle 상태가 같은 active 문서 안에서 세 표현으로 갈린다.
- Evidence:
  - gap table은 네 항목을 `Implemented`와 독립 재감사 대기로 표시한다 (`GAP_CLOSURE_ROADMAP.md:50-53`).
  - 상세 closure 문단은 local integration 기준 `Closed`라고 표현한다 (`GAP_CLOSURE_ROADMAP.md:177-189`).
  - 현재 완료 범위는 G-LLM을 포함한 gap이 구현·검증 전까지 `Open`이라고 남겨 둔다 (`GAP_CLOSURE_ROADMAP.md:244-246`).
  - `audit_roadmap.md:328`과 `IMPLEMENTATION_SUMMARY.md:858`은 R6를 READY FOR AUDIT/open checkpoint로 일관되게 취급한다.
- Expected: 구현 완료, local evidence, independent audit closure가 구분된 하나의 상태 모델을 모든 active summary가 사용한다.
- Actual: 세 상태가 병존해 후속 코더가 R6를 닫힌 gap, 구현 완료 대기, 또는 미구현 open 중 어느 것으로 해석할지 문단에 따라 달라진다.
- Impact: runtime에는 영향이 없지만 phase authority와 다음 작업 선택이 잘못될 수 있다.
- Suggested Fix: 이번 감사 결과를 반영해 G-LLM-001~004를 `Implemented / Audit HOLD` 같은 단일 상태로 정렬하고, `Closed`는 독립 감사 PASS 이후에만 사용한다. §7 현재 완료 범위를 실제 R1~R6 상태로 갱신한다.
- Re-audit Method: table, gap detail, current scope, implementation summary, audit roadmap의 G-LLM 상태를 상호 대조한다.
- Owner: Coder
- Notes: 과거 R1~R5 Closed 상태는 변경 대상이 아니다.

### 5.4 Verified implementation evidence

- loopback host allowlist와 모든 resolved IP loopback 검사, connect address pinning
- `reqwest 0.13.4` default feature off, `blocking/json`만 사용, redirect 0회, system proxy off
- request 32,768 bytes, response 65,536 bytes, text/control/ANSI validation
- worker 1개, capacity 16, same-kind outstanding, bounded shutdown
- opaque request ID, stale revision/current ActionSpace gate, submit 직전 재검증
- decision은 Y 승인 전 core submit 0건, N/Esc는 UI-only
- soft adjudication과 provider failure fallback은 core/save/replay truth에 영향 없음
- R1~R5 전체 회귀와 보고서 9 Verified 상태 유지

## 6. Pass 2: Debug / Engineering Quality Findings

### [DBG-F004] prescribed pending-request exit smoke와 PTY fixture 재현 자산이 없음

- Pass: Debug / Engineering Quality
- Pattern: TEST-001, BUILD-001
- Area: `audit_roadmap.md` R6 exit smoke, `docs/R6_MANUAL_MATRIX.md`, TUI cleanup integration evidence
- Severity: Minor
- Status: **Needs Fix**
- Summary: 구현 순서는 안전해 보이고 disabled clean exit는 독립 재현했지만, roadmap이 지정한 pending-request exit smoke와 success/timeout/stale PTY fixture는 재실행 가능한 형태로 보존되지 않았다.
- Evidence:
  - roadmap은 pending request 중 terminal restore가 먼저 수행되고 worker wait가 250ms를 넘지 않는 exit smoke를 요구한다 (`audit_roadmap.md:324-326`).
  - TUI 코드는 정상 loop 종료 후 cursor/raw/alternate-screen을 먼저 복원하고 `shutdown_with_grace(250ms)`를 호출한다 (`apps/aihack-tui/src/tui/mod.rs:780-792`).
  - worker 단위 test는 grace timeout을 확인하고 LLM TUI tests는 service shutdown을 호출하지만 실제 terminal restore 순서와 pending request를 함께 실행하는 exit smoke는 없다 (`tests/llm_transport.rs:566-585`).
  - manual matrix는 success/timeout/stale provider를 `일회성 fixture`로 명시하며 결과 표만 제공한다. fixture source, 실행 명령, deterministic response/timing 입력 또는 raw transcript가 저장소에 없다 (`docs/R6_MANUAL_MATRIX.md:3-20`).
  - 이번 감사는 120x36 disabled G/A/J와 Q clean exit를 독립 재현했지만 저장소에 없는 success/timeout/stale fixture는 독립 재실행하지 못했다.
- Expected: R6 hard recovery path와 manual claims를 다음 감사자가 같은 명령으로 재현할 수 있다.
- Actual: 코드 inspection과 분리된 worker test는 존재하지만 required end-to-end exit smoke 및 fixture provenance가 없다.
- Impact: 현재 기능 실패가 증명된 것은 아니나 terminal corruption/slow-exit 회귀와 PTY timing/render 회귀를 자동 또는 반복 감사에서 검출하기 어렵다.
- Suggested Fix: deterministic loopback fixture를 `tests/support` 또는 감사용 script로 보존하고 exact command/input/timing을 문서화한다. pending request 상태에서 실제 TUI exit를 유도해 terminal restore가 먼저 완료되고 총 bounded wait가 요구 범위에 드는 smoke를 추가한다. raw screen 전체가 아니라 안정된 semantic assertion만 고정한다.
- Re-audit Method: 보존된 fixture로 disabled/success/timeout/stale/connection-refused 및 pending-exit를 재실행하고, 60x24/80x24/120x36 semantic output과 Q/Esc cleanup을 확인한다.
- Owner: Coder
- Notes: 실제 model smoke는 이 finding의 요구가 아니다. deterministic loopback fixture면 충분하다.

### 6.2 Verified engineering evidence

- R6 tests가 timeout, redirect, invalid/unknown fields, body limit, action/revision gate 등 구체 실패 모드를 이름 붙여 검증한다.
- 전체 294 tests와 deterministic long-run이 통과했다.
- fmt, check, clippy `-D warnings`, debug/release build가 통과했다.
- runtime response queue overflow는 oldest presentation drop으로 bounded된다.
- 실제 disabled PTY에서 core play와 clean exit를 독립 확인했다.

## 7. Pass 3: Security Findings

새 Security finding 없음.

Verified evidence:

- LLM은 default disabled이며 explicit `enabled=true`와 model이 필요하다.
- endpoint는 `http` + explicit port + exact `127.0.0.1`/`localhost`/`::1`만 허용하고 userinfo/query/fragment를 거부한다.
- 최초와 각 request 직전 DNS 결과의 모든 IP가 loopback인지 검사한다.
- redirect, system proxy, referer를 비활성화하고 최초 resolve 주소를 client에 고정한다.
- response body, canonical request, output text, queue, worker, timeout이 bounded다.
- unknown JSON field, invalid action/confidence/reason code, control/ANSI text를 boundary에서 거부한다.
- API key/Authorization/Bearer 처리 경로가 shipped R6 code에 없고 문서는 실제 provider smoke를 비필수·adapter 환경변수 경계로 제한한다.
- `cargo audit --no-fetch`와 cargo-deny licenses/bans/sources가 통과했다.
- `aihack-core` dependency tree에 network/UI dependency가 없다.

보안 경계가 구현됐다는 사실은 IMP-F009의 public schema compatibility finding을 해소하지 않는다.

## 8. Cross-Pass Conflicts

### [XPF-F007] green runtime evidence와 R6 public-contract 완료 주장이 충돌함

- Pass: Cross-Pass
- Pattern: IMP-003, TEST-001
- Area: R6 checkpoint authority
- Severity: Major
- Status: **Hold**
- Summary: Pass 2/3의 실행·보안 evidence는 R6 runtime safety를 지지하지만 Pass 1의 public/schema contract는 미충족이다.
- Evidence: 69개 R6 test와 294개 전체 test는 PASS했으나 IMP-F009의 exact public shape, unsupported version, input error timing을 검증하지 않는다. control 문서는 local acceptance를 완료로 표현하면서 independent audit을 open으로 둔다.
- Expected: R6 PASS는 runtime, public compatibility, evidence reproducibility가 같은 결론을 지지할 때만 선언한다.
- Actual: runtime safety는 PASS지만 public hard boundary는 Needs Fix다.
- Impact: green suite만 보고 R6를 Closed로 전환하면 v0.3.x public compatibility 결함을 다음 Phase로 넘기게 된다.
- Suggested Fix: IMP-F009를 우선 시정하고 IMP-F010/IMP-F011/DBG-F004를 같은 R6 재감사 범위에서 닫는다.
- Re-audit Method: 네 finding의 증거와 targeted/full gates를 함께 재검증한 뒤에만 R6 checkpoint를 PASS/Closed로 전환한다.
- Owner: Coder, Auditor re-verification
- Notes: 구조적 rewrite가 아니라 public boundary와 검증 자산의 국소 정렬로 해결 가능하므로 `REWORK REQUIRED` 대신 `HOLD`다.

## 9. Required Fixes Before PASS

1. IMP-F009: public request DTO, schema-version rejection, `PayloadTooLarge` input error와 response envelope version gate를 확정 명세에 맞춘다.
2. IMP-F010: R6 public error/command enum의 `#[non_exhaustive]` 적용 범위를 정렬한다.
3. IMP-F011: G-LLM 상태를 active control 문서 전체에서 하나로 동기화한다.
4. DBG-F004: pending-request exit smoke와 재현 가능한 deterministic PTY/loopback fixture를 보존한다.
5. 수정 후 R6 표적, public contract/version negative tests, full workspace 294+ tests, clippy/release/supply-chain을 재실행한다.

## 10. Accepted Risks

없음.

실제 model smoke 비수행은 `spec.md`가 허용한 R6 비차단 항목이며 Accepted Risk 면제가 아니다. 원격 CI와 R7/R8도 아래 Remaining Risks에서 계속 추적한다.

## 11. Needs Spec Clarification

없음.

IMP-F009의 정확한 public DTO와 `schema_version = 1`, mismatch failure 요구는 `spec.md`에 충분히 명시되어 있다. 구현자가 현재 축소 API를 유지하려면 감사자가 추정으로 문서를 바꾸는 것이 아니라 승인된 spec/ADR 변경 절차를 먼저 거쳐야 한다.

## 12. Re-audit Checklist

표적 계약 확인:

```bash
cargo test -p aihack --locked --test llm_transport
cargo test -p aihack --locked --test llm_revision_gate
cargo test -p aihack --locked --test llm_soft_adjudication
cargo test -p aihack --locked --test llm_tui_integration
cargo test -p aihack-tui --locked --bin aihack --test tui_contract
```

추가 필수 evidence:

- public `LlmRequestInput` compile fixture가 `spec.md` shape와 일치
- request schema 0/2가 enqueue 전에 typed `UnsupportedSchema`로 실패
- response envelope schema 0/2가 TUI payload acceptance 전에 실패
- observation/action bounds와 canonical 32,768 bytes 초과가 문서화된 input error로 실패
- public error/command enum consumer가 wildcard branch로 compile
- pending request exit smoke가 terminal restore 선행과 bounded wait를 검증
- 보존된 loopback fixture로 success/timeout/stale/connection-refused PTY matrix 재현
- G-LLM status가 roadmap/summary/table에서 동일

전체 gate:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
cargo audit --no-fetch
cargo deny check licenses bans sources
git diff --check
```

## 13. Remaining Risks

- SC-BUILD-02 Linux/Windows 원격 CI evidence pending
- R7 provenance/compatibility와 R8 release NOT RUN
- 실제 model provider 호환성은 비필수 선택 검증으로 남음
- success/timeout/stale live PTY 주장은 현재 일회성 fixture 결과 문서에 의존
- 설치된 advisory DB만 사용했으므로 최신 원격 RustSec 상태는 CI/온라인 감사에서 확인 필요
- working tree의 R6 terminal 보정과 문서는 아직 commit되지 않았으며 change ownership/commit readiness는 이번 감사 범위가 아님
- 최종 release 전 복수 모델 또는 인간 교차감사가 별도로 필요함

## 14. Final Decision

**HOLD — R6 runtime safety passes, public contract closure incomplete**

| Gate | 판정 |
| --- | --- |
| 보고서 9 IMP-F008 / R1~R5 | 기존 Verified/PASS 유지 |
| R6 local transport·failure safety | Runtime PASS |
| R6 stale/action/explicit approval | Runtime PASS |
| R6 soft presentation-only | Runtime PASS |
| R6 public/schema compatibility | **HOLD, IMP-F009 Major** |
| R6 public enum stability | Needs Fix |
| R6 PTY evidence reproducibility | Needs Fix |
| R6 independent checkpoint | **HOLD** |
| R7/R8/remote CI | pending / NOT RUN |
| 전체 프로그램/release | 아직 PASS 대상 아님 |

추가 수정은 필요하다. 다만 transport나 TUI 전체를 다시 쓰는 문제가 아니라, 확정된 public/schema boundary와 그 회귀 테스트·재현 자산·상태 문서를 R6 안에서 정렬하는 국소 시정이다.

코드·설정·기존 문서는 수정하지 않았고 감사 보고서만 생성했다. 시정은 별도 코더가 진행하고, 완료 후 IMP-F009/010/011 및 DBG-F004를 연결한 재감사를 요청한다.
