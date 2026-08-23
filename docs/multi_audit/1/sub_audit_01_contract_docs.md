# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: contract_docs (목표·계약·문서·구현·기능 정합성)
- User Goal: 현재 프로젝트의 문서와 구현을 대조해 모순·문제점을 진단하고 해결 가능한 감사 결과를 제시한다.
- Audit Basis: Standard-backed
- Standard Path: `C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md`
- Report Contract Path: `C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md`
- Repository: `C:\LocalDev\rust\AIHack`
- Audited HEAD: `80d959af94cb08c5d9b2f2601f5e63f3827a1210` (`codex/audit-report-24-remediation`)
- Audit Date: 2026-08-23 (Asia/Seoul)
- Other sub-audit reports: 읽지 않음. 부모 대화의 결론도 가정하지 않음.

## 2. Assigned Scope

- 루트 활성 문서: `spec.md`, `README.md`, `BUILD_GUIDE.md`, `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`, `DESIGN_DECISIONS.md`, `designs.md`, `CHANGELOG.md`, `DOCUMENTATION_AUDIT_REPORT.md`, `LESSONS_LEARNED.md`, provenance/license 문서.
- 활성 audit chain: `audit_report_20.md`, `audit_report_21.md`, `docs/audit/audit_report_22.md`~`audit_report_24.md` 및 두 remediation 기록.
- `docs/compatibility/**`, R6/R7 보조 보고서, 루트 `Cargo.toml`/`Cargo.lock`/`deny.toml`/toolchain/CI/build script.
- 주요 구현·테스트 대조: headless CLI와 policy, runtime `GameClient`/session/save 경계, causal witness 계약, `tests/r8_documentation.rs`, `tests/long_run.rs`, 관련 crate manifest.
- 문서 주장과 현재 source/test/manifest의 양방향 정합성, 완료·pending·release evidence 권한, 계약 ID 추적, 로컬 링크 존재 여부를 검사했다.

## 3. Excluded and Uninspected Scope

- 사용자가 지정한 `legacy_nethack_port_reference/**` 본문 구현, `.git/**`, `target/**`, runtime 생성물은 제외했다. active 코드·문서가 해당 레거시 경로를 참조하는지는 정적 검색으로 확인했다.
- 외부 게시·release·tag·push는 수행하지 않았다. GitHub Actions 링크의 내용을 재실행하지 않고 저장소에 기록된 run/SHA evidence와 현재 Git 이력을 대조했다.
- 다른 관점의 보안·성능·전체 코드 품질을 독립적으로 판정하지 않는다. 이 보고서의 보안 관련 언급은 문서 권한과 contract-docs 정합성에 한정한다.
- source 전체를 모든 줄 단위로 재검토한 것이 아니라, 문서가 주장하는 public/runtime/CLI 경계와 관련 테스트·entrypoint를 표적 대조했다.

## 4. Evidence Examined

### 문서·계보

- `docs/audit/audit_report_24.md:81-90` — report 23의 `IMP-F016`, `IMP-F017`, `IMP-F018`, `TEST-F001`, `DBG-F009`, `DBG-F010`, `SEC-F001`, `SEC-F002`가 report 24 재감사에서 검증된 표.
- `docs/audit/audit_report_24_remediation.md:64-68` — report 24 시정의 남은 gate가 후속 독립 재감사와 사용자 승인임을 명시.
- `README.md:84-87`, `BUILD_GUIDE.md:446-450`, `IMPLEMENTATION_SUMMARY.md:16-24`, `audit_roadmap.md:380-386,463-464`, `DESIGN_DECISIONS.md:15-42`, `DOCUMENTATION_AUDIT_REPORT.md:222-233`, `docs/compatibility/README.md:3`, `docs/audit/audit_report_22.md:1-4,110,128,132`.
- `GAP_CLOSURE_ROADMAP.md:16,41-42,51,59-62`의 상태 규칙과 현재 gap 행.

### 구현·테스트·명령

- `apps/aihack-headless/src/main.rs:14-19` — `turns`가 범위 validator 없는 `u64`이며 `policy` 기본값이 `wait-v1`.
- `BUILD_GUIDE.md:238-242` — `--turns`가 `1..=1,000,000`, `--policy` 기본값이 `survival-v1`이라고 선언.
- `cargo run --locked -p aihack-headless --bin aihack-headless -- --help` — 실제 help가 policy 기본값 `[default: wait-v1]`을 출력.
- `cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 0 --policy wait-v1 --report reports/contract-probe.json` — `exit 0`, `requested_turns=0`, `accepted_turns=0`으로 문서의 최소 1 제한과 불일치. 생성한 probe는 감사 후 삭제했다.
- `cargo test -p aihack --locked --test r8_documentation -- --nocapture` — 7 tests PASS. 아래 stale expectation을 포함해 PASS하므로 false-green finding의 증거로 사용했다.
- `cargo metadata --locked --no-deps --format-version 1` — 8개 workspace member, 모두 version `0.3.0`, license `NGPL`.
- `cargo run --locked --release -p aihack-headless --bin aihack-headless -- --seed {42,7,1234} --turns 1000 --policy survival-v1` — 세 run 모두 1000 accepted, 1000 submitted, `Playing`; hashes는 `c734eeafedc77c82`, `de24bb6e33a8c43f`, `c6f5e6ca9498ef35`로 `audit_roadmap.md:261`과 일치.
- Markdown local-link 검사: 활성 markdown 22개 local link 검사, broken link 0건.
- `git diff 2519bc8e0ede81c39f46b5778e62a41d4ca66901..HEAD` — current HEAD와 report 24가 인용한 same-SHA 사이에 문서 8개와 `tests/r8_documentation.rs` 변경이 존재하며 source/manifest 변경은 없음.

## 5. Findings

### [A01-F001] report 24 이후에도 활성 문서가 report 23 재감사를 현재 pending으로 유지함

- Pass: Implementation
- Pattern: IMP-004, IMP-003, DOC-BACKFILL-001
- Area: active audit authority, current-state documentation, audit-chain synchronization
- Severity: Major
- Status: Confirmed
- Standard Status: Needs Fix / Hold
- Summary: report 24는 report 23의 기존 finding을 독립 재감사해 Verified로 기록했지만, 여러 활성 문서와 문서 회귀 테스트가 여전히 report 23 시정의 독립 재감사를 현재 미완료 gate로 표현한다. report 24 remediation의 실제 후속 gate는 report 24 시정 자체의 독립 재감사다.
- Evidence:
  - `docs/audit/audit_report_24.md:81-90`은 report 23 관련 finding을 `Verified` 또는 `Verified locally`로 판정한다.
  - `docs/audit/audit_report_24_remediation.md:64-68`은 report 24 시정의 남은 gate를 `report 24 시정의 후속 독립 재감사`와 사용자 승인으로 둔다.
  - `README.md:85`(영문 및 일본어·번체·간체 대응 문장 포함), `IMPLEMENTATION_SUMMARY.md:18-20`, `BUILD_GUIDE.md:447,450`, `GAP_CLOSURE_ROADMAP.md:41,51,59,61`, `audit_roadmap.md:384,463-464`, `docs/compatibility/README.md:3`, `docs/audit/audit_report_22.md:4,110,128,132`가 report 23 재감사를 pending으로 남긴다.
  - `designs.md:9`는 이미 `audit_report_21.md`가 닫은 report 20 remediation을 아직 pending으로 표현한다.
  - `DOCUMENTATION_AUDIT_REPORT.md:222-225`도 report 23 finding을 독립 재감사 전에는 Verified로 올리지 않는다고 기록한다.
  - `tests/r8_documentation.rs:46,138-142,158-159,191`은 이 stale 문구를 기대값으로 검사해 현재 모순을 PASS로 고정한다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md` Sections 6, 9, 13, 14, 16 및 IMP-004/DOC-BACKFILL-001. 독립 재감사 report 24의 finding 상태와 최신 remediation 기록이 활성 current-state authority에 반영되어야 하며, 과거 report의 초기 상태는 historical scope로 표시되어야 한다.
- Expected: 활성 문서는 report 23 기존 finding을 Verified로, report 24 시정의 후속 독립 재감사를 유일한 현재 pending gate로 표시해야 한다.
- Actual: report 23의 “재감사 대기”가 현재 gate처럼 남고, report 24의 새 finding 시정에 대한 후속 재감사와 구분되지 않는다. 같은 문서 세트 안에서 “report 23 pending”과 “report 23 verified/report 24 remediation”이 동시에 current처럼 읽힌다.
- Impact: SC-DOC-01과 R8/R9 release authority가 단일 결론으로 결정되지 않는다. 작업자가 완료된 시정을 반복하거나, 반대로 report 24 시정의 미검증 상태를 report 23 시정 완료로 오인할 수 있다. 최종 PASS 또는 외부 게시를 이 상태에서 판정하면 안 된다.
- Suggested Action: 활성 문서의 pending gate를 “report 24 시정 후속 독립 재감사”로 통일하고, README 5개 언어·summary·build guide·gap/audit roadmap·design/compatibility 문서를 갱신한다. report 22/23은 초기 상태와 후속 report 24 판정을 분리한 historical record로 남긴다. `tests/r8_documentation.rs`는 report 23 pending 토큰을 긍정 기대값으로 사용하지 말고 current section/row의 단일 pending authority와 predecessor stale-token 부재를 검사해야 한다.
- Suggested Fix: 위 Suggested Action을 적용해 predecessor report와 successor remediation의 authority를 분리한다.
- Re-audit Method: (1) 활성 문서에서 report 23 pending/current 토큰을 section-aware 검색해 0건인지 확인한다. (2) report 24 verified finding과 report 24 remediation 후속 gate가 각각 한 번만 current로 나타나는지 확인한다. (3) 문서 회귀 테스트에 stale predecessor fixture와 report 24 follow-up fixture를 넣고 각각 실패/통과를 확인한다. (4) `cargo test -p aihack --locked --test r8_documentation` 및 관련 전체 workspace test를 새 문서 상태에서 재실행한다.
- Confidence: High
- Owner: Documentation owner / Auditor
- Notes: report 24 자체의 historical finding을 수정하라는 뜻이 아니라, active authority와 후속 remediation 문서의 계보 표현을 정렬하라는 finding이다.

### [A01-F002] Gap register가 독립 재감사 전의 report 24 시정을 `Closed`로 표기함

- Pass: Implementation
- Pattern: IMP-003, IMP-004
- Area: gap lifecycle and release gate state
- Severity: Major
- Status: Confirmed
- Standard Status: Needs Fix / Hold
- Summary: `GAP_CLOSURE_ROADMAP.md`의 상태 문법과 report 24 remediation 권한이 서로 충돌한다. `Closed / re-audit pending`은 문서가 허용한 상태 전이가 아니며, 독립 재감사가 아직 남은 시정은 Closed로 표시할 수 없다.
- Evidence:
  - `GAP_CLOSURE_ROADMAP.md:16`은 상태를 `Open -> Implemented -> Verified -> Closed`로 제한하고 `Closed`를 독립 audit 또는 동등 re-audit evidence가 있을 때만 허용한다.
  - `GAP_CLOSURE_ROADMAP.md:42,60,62`의 `G-BUILD-006`, `G-DOC-005`, `G-SEC-002`는 각각 `Closed / re-audit pending`이다.
  - 같은 표의 `G-BUILD-005`, `G-TEST-003`, `G-DOC-004`, `G-SEC-001`은 report 24에서 관련 report 23 finding이 검증됐는데도 `Implemented / report 23 re-audit pending`으로 남아 predecessor와 successor 상태가 모두 낡았다.
  - `docs/audit/audit_report_24_remediation.md:64-68`은 report 24 시정의 후속 독립 재감사가 남았다고 명시한다.
- Expected Basis: `GAP_CLOSURE_ROADMAP.md` 자체의 lifecycle/Closed 정의와 AI audit Section 7/9. report 23 원 finding은 report 24 evidence에 따라 Closed로 정리할 수 있고, report 24 새 finding 시정은 적어도 `Implemented / report 24 follow-up re-audit pending`으로 남겨야 한다.
- Expected: 모든 gap 상태는 정의된 전이와 독립 evidence를 따르고, report 24 시정 후속 재감사 전에는 `Closed`를 사용하지 않아야 한다.
- Actual: “Closed”와 “re-audit pending”을 동시에 사용해 closure 권한과 pending 증거를 한 행에서 충돌시킨다. 일부 P0 gap은 report 23 predecessor 대기로 잘못 남는다.
- Impact: gap 기반 phase gate를 읽는 사람이 검증되지 않은 dependency/permission/contract 문서를 Closed로 오인하거나 완료된 시정을 다시 pending으로 분류한다. R8/R9 PASS 판단과 handoff 우선순위가 왜곡된다.
- Suggested Action: report 23 관련 G-BUILD-005/G-TEST-003/G-DOC-004/G-SEC-001의 report 24 Verified evidence를 연결해 상태를 `Closed`로 정리하고, report 24 관련 G-BUILD-006/G-DOC-005/G-SEC-002를 `Implemented / report 24 follow-up re-audit pending`으로 정규화한다. 상태 파서/회귀 테스트가 허용되지 않은 조합을 거부하게 한다.
- Suggested Fix: gap 행의 composite 상태를 제거하고 predecessor/successor report ID와 독립 evidence를 각 행에 연결한다.
- Re-audit Method: 표의 모든 상태가 허용 enum 또는 명시된 후속 상태만 사용하는지 정적 검사하고, 각 `Closed` 행에 독립 report/evidence ID가 연결되는지 검증한다. `G-*` row-specific documentation test와 R8/R9 gate를 재실행한다.
- Confidence: High
- Owner: Documentation owner / Release manager
- Notes: report 24의 same-SHA CI가 존재한다는 사실은 독립 re-audit 전의 `Closed` 권한을 자동 부여하지 않는다.

### [A01-F003] Headless `--policy` 기본값이 BUILD_GUIDE 계약과 구현에서 다름

- Pass: Implementation
- Pattern: IMP-001, BUILD-001
- Area: CLI contract, current run behavior
- Severity: Major
- Status: Confirmed
- Standard Status: Needs Fix
- Summary: 활성 build guide는 `--policy` 기본값을 `survival-v1`로 선언하지만 실제 headless binary는 `wait-v1`을 기본값으로 사용한다.
- Evidence:
  - `BUILD_GUIDE.md:242` — `wait-v1, survival-v1, replay-file; default survival-v1`.
  - `apps/aihack-headless/src/main.rs:18` — `#[arg(long, default_value = "wait-v1")]`.
  - 실제 `cargo run --locked -p aihack-headless --bin aihack-headless -- --help` 출력이 `[default: wait-v1]`을 표시한다.
  - `BUILD_GUIDE.md:24`는 current working tree의 wait-only와 v0.3.0 target의 survival-v1을 구분하지만, 같은 문서의 active flag contract와 현재 package `0.3.0` 실행 표면에는 어느 값이 authoritative인지 명시되지 않는다.
- Expected Basis: `BUILD_GUIDE.md:238-242`의 명시적 CLI flag contract와 `spec.md:12`의 headless policy/1000 accepted-turn 기준. 문서와 실제 default가 같아야 하며 current/target을 구분한다면 실행 명령에 그 구분을 명시해야 한다.
- Expected: CLI help와 실행 기본값이 BUILD_GUIDE의 canonical policy와 동일해야 한다.
- Actual: 인자 없이 실행하면 wait-only 정책이 선택되고, 문서가 선언한 survival-v1 기본 동작과 다르다. 사용자가 `--turns 1000`만 사용하면 long-run 성공 계약을 자동으로 얻지 못한다.
- Impact: 사용자 실행과 release verification command가 서로 다른 policy를 실행한다. accepted-turn/early-game-over 결과와 report policy가 문서 예상과 달라질 수 있다.
- Suggested Action: v0.3.0 계약을 survival-v1 default로 확정하면 Clap default와 help/contract test를 바꾸고, wait-only를 명시적 `--policy wait-v1`로 남긴다. wait-v1이 current 의도라면 BUILD_GUIDE/README/spec의 default와 target 표기를 wait-v1로 좁히고 survival-v1을 항상 명시한다. 어느 방향인지 명세에 한 줄로 고정한다.
- Suggested Fix: source 또는 문서 중 한 쪽을 canonical default에 맞추고 help/contract test를 동기화한다.
- Re-audit Method: `--help`의 default, 인자 없는 `--turns 1` 실행 report, 명시적 wait/survival 실행을 비교하고 `BUILD_GUIDE.md`, README 5개 언어, `tests/build_contract.rs`의 기대값을 함께 재검증한다.
- Confidence: High
- Owner: Coder / Documentation owner

### [A01-F004] Headless `--turns` 문서 범위가 구현에서 강제되지 않음

- Pass: Implementation
- Pattern: IMP-001, BUILD-001
- Area: CLI input validation
- Severity: Minor
- Status: Confirmed
- Standard Status: Needs Fix
- Summary: BUILD_GUIDE가 `--turns`를 `1..=1,000,000`으로 문서화하지만 source는 범위가 없는 `u64` parser를 사용해 0을 성공 입력으로 허용한다.
- Evidence:
  - `BUILD_GUIDE.md:241` — `absolute target turn, 1..=1,000,000`.
  - `apps/aihack-headless/src/main.rs:16-17` — `default_value_t = 1000`인 plain `turns: u64`; `value_parser` 또는 범위 검사가 없다.
  - `cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 0 --policy wait-v1 --report reports/contract-probe.json`가 exit 0 및 `requested_turns=0`, `accepted_turns=0`을 출력했다.
- Expected Basis: `BUILD_GUIDE.md:241`의 명시적 CLI validation contract. 범위가 target-only라면 current/target 표를 분리하고 현재 parser가 계약을 제공하지 않는 사실을 문서화해야 한다.
- Expected: `--turns`는 1 이상 1,000,000 이하만 허용하거나, 그 범위를 실제 계약에서 제거해야 한다.
- Actual: 0이 유효한 no-op 성공으로 처리된다. 상한 1,000,000도 source 경계에서 확인되지 않는다.
- Impact: 잘못된 입력을 release/automation 성공으로 오인할 수 있고 `--turns 1000` acceptance의 하한 계약이 CLI에서 닫히지 않는다.
- Suggested Action: `clap` value parser 또는 명시적 초기 검증으로 1..=1,000,000을 강제하고 하한/상한/초과 회귀를 추가한다. 범위를 실제로 허용하지 않을 계획이면 BUILD_GUIDE와 spec의 수치를 수정한다.
- Suggested Fix: parser에 범위를 연결하고 경계 회귀 테스트를 추가하거나 문서의 validation 수치를 현재 구현에 맞게 수정한다.
- Re-audit Method: 0, 1, 1,000,000, 1,000,001에 대해 exit code/report 생성 여부를 확인하고 valid 1000 run과 전체 headless contract test를 재실행한다.
- Confidence: High
- Owner: Coder / Documentation owner

### [A01-F005] 현재 문서·문서 회귀 테스트가 인용된 same-SHA CI 범위 밖에 있음

- Pass: Implementation
- Pattern: IMP-003, IMP-004, BUILD-001
- Area: immutable evidence and release-document scope
- Severity: Major
- Status: Confirmed
- Standard Status: Needs Fix / Hold
- Summary: 활성 문서와 `tests/r8_documentation.rs`는 report 24 CI가 실행된 implementation SHA `2519bc8e` 이후 변경됐지만, 현재 문서가 이를 current release evidence처럼 연결한다. source/manifest는 바뀌지 않았다는 점은 완화 요소지만, 문서 gate와 회귀 테스트 자체는 같은 SHA evidence로 검증되지 않았다.
- Evidence:
  - `git rev-parse HEAD`는 `80d959af...`이다. report 24 remediation과 `README.md`, `BUILD_GUIDE.md`, `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`, `DOCUMENTATION_AUDIT_REPORT.md`, `DESIGN_DECISIONS.md`, `tests/r8_documentation.rs`는 `2519bc8...` 이후 commit `0fba087`, `80d959a`에서 변경됐다.
  - `docs/audit/audit_report_24_remediation.md:56,62`와 `BUILD_GUIDE.md:448`은 Actions run `32107862171`이 implementation SHA `2519bc8...`에서 PASS했다고 기록한다.
  - `git diff 2519bc8..HEAD`는 9개 문서와 `tests/r8_documentation.rs` 변경을 보여주며, 이 current documentation test의 7-test PASS는 로컬 재실행일 뿐 run `32107862171`의 same-SHA evidence가 아니다.
- Expected Basis: AI audit Section 3.1/9/16 및 report contract의 immutable source evidence 규칙. 최종 release authority를 current HEAD로 주장하려면 해당 HEAD의 문서·테스트·bundle·CI가 함께 검증되어야 하며, implementation-only SHA를 사용할 때 그 범위를 명시해야 한다.
- Expected: release/current-state 문서와 문서 회귀 테스트를 포함한 최종 intended SHA가 CI·bundle evidence와 동일해야 한다.
- Actual: 문서는 `2519bc8` CI를 current status로 사용하면서 HEAD 후속 문서/test 변경을 별도 pending으로 표시하지 않는다.
- Impact: report24 remediation evidence와 현재 active documentation gate의 provenance가 분리된다. 특히 현재 `tests/r8_documentation.rs`가 stale status를 긍정 기대값으로 검사하는데 same-SHA CI에는 그 test revision이 없으므로 false-green을 release evidence로 확대할 수 있다.
- Suggested Action: intended final commit을 하나로 확정하고 그 SHA에서 full tests, R7/R8 checkpoint, bundle, cargo-audit/cargo-deny를 재실행한다. 또는 `2519bc8`을 implementation-only baseline으로 명확히 라벨링하고 후속 문서/test commit은 독립 re-audit 전까지 release authority에서 제외한다.
- Suggested Fix: current HEAD를 새 immutable CI/bundle 기준으로 만들고 report manifest에 SHA를 갱신한다.
- Re-audit Method: `git diff <audited_sha>..HEAD`가 비어 있거나 각 변경이 새 CI evidence에 포함되는지 확인하고, source archive에 current docs/test가 들어가는지 inspect한다. report manifest와 same-SHA run ID를 다시 연결한다.
- Confidence: High
- Owner: Release manager / Auditor
- Notes: 이 finding은 `2519bc8`의 실제 CI 성공을 부정하지 않는다. 현재 HEAD 전체를 그 CI가 검증했다는 확장을 막는 finding이다.

### [A01-F006] CHANGELOG의 Unreleased audit-chain 기록이 report 24를 누락함

- Pass: Implementation
- Pattern: IMP-004, DOC-BACKFILL-001
- Area: release history and audit traceability
- Severity: Minor
- Status: Confirmed
- Standard Status: Needs Documentation Recovery
- Summary: 현재 Unreleased 변경 요약은 report 20~23의 history/state를 언급하지만 report 24 시정·same-SHA evidence·후속 re-audit gate를 명시하지 않는다.
- Evidence:
  - `CHANGELOG.md:19-21`은 report 20/21/22/23과 SC-CAUSE/permission 변경을 기록한다.
  - `docs/audit/audit_report_24.md` 및 `docs/audit/audit_report_24_remediation.md`는 winx exception, Windows DACL 계약, SC-CAUSE mapping과 후속 독립 재감사를 별도 권한으로 기록한다.
- Expected Basis: AGENTS 문서 동기화/Changelog 정책 및 AI audit Section 2/14. 릴리스 가치가 있는 report24 remediation은 Unreleased history에서 현재 상태와 evidence로 추적 가능해야 한다.
- Expected: Unreleased history에서 report24의 시정 항목·evidence·후속 gate를 report23 predecessor와 구분해 추적할 수 있어야 한다.
- Actual: 기능 변경 항목은 일부 있지만 report24 권한 계보와 “report24 follow-up re-audit pending”이 changelog에서 직접 추적되지 않는다.
- Impact: release history만 읽는 운영자가 report24의 Major gate와 후속 재감사 필요성을 놓칠 수 있다.
- Suggested Action: Unreleased Changed/Fixed에 report24 finding별 시정과 independent follow-up gate를 추가하고, report23 predecessor와 report24 successor를 명확히 구분한다.
- Suggested Fix: CHANGELOG Unreleased에 report24 remediation과 follow-up gate를 추가한다.
- Re-audit Method: CHANGELOG에서 report20~24의 state/authority가 현재 active docs와 일치하는지 ID 기반 검색하고 문서 regression을 재실행한다.
- Confidence: Medium
- Owner: Documentation owner

### [A01-F007] ADR-0033 내부 verification 상태가 서로 모순됨

- Pass: Implementation
- Pattern: IMP-004
- Area: ADR status and dependency release authority
- Severity: Minor
- Status: Confirmed
- Standard Status: Needs Fix
- Summary: 같은 ADR이 cargo-deny 검증을 “대기”라고 표시하면서 아래 verification update에서는 동일 검증이 PASS했다고 기록한다.
- Evidence:
  - `DESIGN_DECISIONS.md:15-17` — `Status: Accepted (2026-08-18), cargo-deny 0.19.4 검증 대기`.
  - `DESIGN_DECISIONS.md:30,42` — cargo-deny를 필수 evidence로 유지하고 `2519bc8`/Actions `32107862171`에서 Ubuntu/Windows PASS라고 기록한다.
- Expected Basis: AI audit IMP-004와 ADR authority 규칙. Accepted(결정 승인)와 Verification PASS(증거 상태)는 분리할 수 있지만 한 상태 줄이 이미 통과한 검증을 pending으로 표현해서는 안 된다.
- Expected: ADR의 결정 승인 상태와 cargo-deny verification 상태가 별도 필드로 표현되고 서로 모순되지 않아야 한다.
- Actual: ADR-0033을 단독으로 읽으면 dependency release gate가 아직 검증되지 않은 것으로 보인다.
- Impact: cargo-deny exception의 expiry/owner는 이해할 수 있어도 R1/R8 dependency gate의 현재 상태가 불명확해진다.
- Suggested Action: `Status: Accepted; cargo-deny verification recorded in ...`처럼 결정 상태와 증거 상태를 별도 필드로 정리하고, follow-up independent audit와 local/same-SHA CI를 구분한다.
- Suggested Fix: ADR-0033의 pending 문구를 실제 verification evidence와 일치하는 `Verified`/`Recorded` 상태로 정리한다.
- Re-audit Method: ADR-0033, `deny.toml`, BUILD_GUIDE, report24 remediation의 owner/version/expiry/verification 문장을 대조한다.
- Confidence: High
- Owner: Documentation owner / Release manager

### [A01-F008] 활성 source 주석이 현재 v0.3.0 phase와 과거 버전을 구분하지 않음

- Pass: Implementation
- Pattern: DOC-BACKFILL-001, IMP-004
- Area: source-to-document maintenance context
- Severity: Minor
- Status: Probable
- Standard Status: Needs Documentation Recovery
- Summary: 동작을 직접 깨뜨리지는 않지만 active runtime/UI source의 phase/version 주석이 v0.1.0/v0.2.0 또는 Phase 4/5/16~19로 남아 있어 현재 v0.3.0 workspace 책임과 historical implementation note가 구분되지 않는다.
- Evidence:
  - `crates/aihack-runtime/src/session.rs:28-29,97-98,496`, `crates/aihack-runtime/src/world.rs:124-143`에 Phase 4/5/16 및 v0.2.0 주석이 남아 있다.
  - `apps/aihack-tui/src/tui/theme.rs:3`, `apps/aihack-tui/src/tui/input.rs:30-39`, `apps/aihack-tui/src/tui/mod.rs:88,133-139` 등에도 v0.1/v0.2/Phase 17~19 표기가 남아 있다.
  - `Cargo.toml:3`, `README.md:3`, `spec.md:목표 버전` 및 활성 summary는 현재 package/계약을 v0.3.0으로 설명한다.
- Expected Basis: AI audit reverse-documentation 규칙. 과거 phase 주석을 보존할 수는 있으나 현재 동작의 authority인지 historical rationale인지 구분되어야 한다.
- Expected: 과거 phase/version 주석은 historical rationale로 명시하거나 현재 v0.3.0 책임·불변조건을 설명해야 한다.
- Actual: 주석만 읽으면 일부 runtime/UI가 과거 버전 구현으로 보이며, source ownership/current contract와 archive history의 경계가 없다.
- Impact: 유지보수자가 이미 이동·추출된 책임을 잘못 추적하거나 stale phase gate를 재개할 수 있다. 기능/보안 gate 자체의 즉시 실패 증거는 아니다.
- Suggested Action: 동작 불변조건과 선택 이유만 현재 용어로 정리하고, 남겨야 할 migration/history 주석에는 `historical phase note`와 현재 문서 링크를 붙인다. 매 변경마다 버전 주석을 새로 넣는 방식은 피한다.
- Suggested Fix: stale version marker를 current/historical로 분류하고 필요한 주석만 갱신한다.
- Re-audit Method: active source의 `[v0.*]`/`Phase` 주석을 inventory하고 각 항목이 현재 책임·역사 메모 중 하나로 분류됐는지 확인한다.
- Confidence: Medium
- Owner: Coder / Documentation owner

## 6. Uncertainties and Clarifications Needed

- `BUILD_GUIDE.md`의 current working tree와 v0.3.0 target 표를 기준으로 `survival-v1`을 기본값으로 확정할지, 실제 `wait-v1`을 current default로 유지할지 사용자/아키텍처 결정이 필요하다. 이 선택에 따라 source 수정 또는 문서 수정 범위가 달라진다.
- 최종 release authority가 implementation-only SHA `2519bc8e`인지 현재 HEAD `80d959a`인지 명시해야 한다. 문서/test 변경을 release bundle에 포함한다면 HEAD 기준 same-SHA CI가 필요하다.
- `hallucinating` compatibility risk의 owner/trigger는 report24 계보에 기록되어 있으므로 본 관점에서 재판정하지 않았다.
- Security pass의 filesystem/network 전체 gate는 본 보고서의 배정 범위가 아니며, report24의 SEC-F003 문서 경계가 현재 active docs에 반영됐는지만 확인했다.

## 7. Perspective Decision

**HOLD — contract/document authority is not currently PASSable.**

문서 링크 자체는 22개 검사에서 모두 유효하고, workspace metadata·표적 문서 테스트·장기 survival 실행은 통과했다. 그러나 report24 이후 active audit chain의 current-state가 단일 권위로 닫히지 않고, gap register에 invalid composite closure 상태가 있으며, headless CLI 계약 두 항목이 문서와 구현에서 다르다. 위 Major finding을 정리하고 current final SHA에 대한 재감사를 수행하기 전에는 전체 PASS 또는 외부 게시 가능 상태로 해석할 수 없다.

### Pass별 범위 판정

- Pass 1 Implementation Compliance: A01-F001~A01-F008에서 문서·CLI·구현 정합성 문제를 확인했다.
- Pass 2 Debug / Engineering Quality: CLI 실행 증거는 A01-F003/A01-F004에 반영했지만 전체 빌드·dependency 품질을 독립 판정하지 않았다.
- Pass 3 Security: 전체 공격 표면은 배정 외 범위다. 문서가 선언하는 save permission/current audit authority의 정합성만 확인했으며 보안 PASS를 주장하지 않는다.

### Coverage note

| Work surface / question | Evidence | Coverage | Follow-up |
| --- | --- | --- | --- |
| Active audit-chain authority | report 22~24, README/BUILD/summary/gap/audit/design/compat docs, r8 regression | Covered with findings | A01-F001/F002/F005 시정 후 재감사 |
| Headless CLI contract | source, BUILD_GUIDE, `--help`, `--turns 0` run | Covered with findings | A01-F003/F004 |
| Markdown local links | 22 local links, 0 broken | Covered | None |
| Core/game behavior | targeted public entrypoints and long-run command only | Partially Covered | 다른 구현/테스트 관점 감사 |
| Security boundary | report24 docs and save contract references only | Partially Covered | 독립 security pass |

## 8. Coder Handoff

`C:\LocalDev\rust\AIHack\docs\multi_audit\1\sub_audit_01_contract_docs.md`를 먼저 읽고, A01-F001~F008을 현재 `spec.md`, report24 계보, 실제 source/test와 대조하세요. 계약을 바꿀 경우 관련 문서를 먼저 정렬한 뒤 source/test를 수정하고, 최종 intended SHA에서 문서 회귀·headless boundary·전체 quality gate 및 독립 재감사 증거를 기록하세요.
