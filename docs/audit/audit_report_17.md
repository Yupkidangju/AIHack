# AIHack R8 Remediation Re-audit Report 17

감사 기준: `AI_AUDIT_DOC_STANDARD.md`

감사 유형: `audit_report_16.md` 시정에 대한 독립 3-pass 재감사

감사 일자: 2026-07-20 (Asia/Seoul)

기준 commit: `04775c189fba59146835f6a9055f5606714b90f4` (`main`, `origin/main`) + 현재 R8 remediation working tree

환경: Linux 7.0.0-28-generic x86_64, rustc/cargo 1.94.1

코드·설정 수정: 없음

감사 산출물: 이 보고서만 추가

## 1. Audit Summary

최종 판정: **HOLD — report 16 remediation is substantially verified; release bundle authority trace and same-commit evidence remain**

보고서 16의 시정 방향은 대부분 정확히 반영됐다.

- `IMP-F014`: 배포되지 않는 Git history 주장을 제거하고 `MODIFICATIONS.md`, commit-expanded `RELEASE-METADATA`, `SHA256SUMS`와 source archive verifier로 교체했다. 원 finding의 객관적 문서·bundle 모순은 **Verified**다.
- `IMP-F015`: compatibility 인덱스 10건과 개별 record가 모두 `Approved`로 정렬되고 파싱 회귀가 추가돼 **Verified**다.
- `IMP-F012`: direct user instruction, 승인 범위와 boundary가 `AIHACK-OWNER-2026-07-20-NGPL-01`로 기록되고 PROV/scenario에 연결됐다. working-tree 내용은 **Partially Verified**지만 immutable commit reference와 release bundle 내 resolvability가 남았다.
- `DBG-F006`: clean fixture 기반 실제 tar archive 검증과 Linux/Windows CI release step은 구현됐다. 그러나 공식 R8 clean commit과 같은 commit의 원격 CI는 아직 없어 **Hold**다.

재감사에서 새 `DBG-F007`을 확인했다. 현재 positive release fixture에는 `PROJECT_OWNER_LICENSE_APPROVAL.md`가 없고 output `RELEASE-METADATA`에도 `owner_approval`과 `modification_notice`가 없지만 verifier가 PASS한다. 따라서 승인 ID와 modification ID가 실제 배포 source에서 해석 불가능해지는 회귀를 final release gate가 놓칠 수 있다.

로컬 기술 검증은 R7/R8 checkpoint, 표적 37개, 전체 338개 테스트, fmt, Clippy, release build, 최신 RustSec, cargo-deny와 PTY 3종이 모두 PASS했다. 하지만 Major release finding과 same-commit CI가 남아 R8 전체와 외부 배포는 HOLD다.

## 2. Audit Scope

### 2.1 재감사 대상

- 원 finding: `IMP-F012`, `IMP-F014`, `IMP-F015`, `DBG-F006`, `XPF-F010`
- 승인 evidence: `PROJECT_OWNER_LICENSE_APPROVAL.md`, `PROVENANCE.md`, NH367-C001..C010
- notice/source evidence: `NOTICE`, `MODIFICATIONS.md`, `RELEASE-METADATA`, `.gitattributes`
- packaging: `build.sh`, `build.bat`, `scripts/verify_release_bundle.sh`
- release tests: `tests/license_compliance.rs`, `tests/release_gate.rs`, `tests/release_bundle.rs`, `tests/r8_documentation.rs`, `tests/build_contract.rs`
- CI: `.github/workflows/ci.yml`
- 연결 문서: `spec.md`, `DESIGN_DECISIONS.md`, `BUILD_GUIDE.md`, `audit_roadmap.md`, `DOCUMENTATION_AUDIT_REPORT.md`, README/CHANGELOG/roadmap/summary
- regression/security: 전체 workspace와 기존 R6/R7/R8 gate

### 2.2 현재 repository 상태

- `HEAD`와 `origin/main`: `04775c1`, R7 commit
- tracked 변경: 38 files
- R8/remediation 신규 파일과 기존 `audit_report_16.md`는 untracked
- 실제 R8 clean commit: 없음
- same-commit Linux/Windows CI: 없음

## 3. Excluded Scope

- 실제 외부 게시·배포·릴리스
- Windows host에서의 수동 `build.bat` 실행
- GitHub Actions 원격 결과: 해당 R8 commit이 아직 없음
- 실제 유료/원격 LLM provider 호출
- NGPL 의무의 최종 법률 판단; 프로젝트가 승인한 engineering distribution contract의 구현 정합성만 감사
- legacy reference tree의 기능·법률 내용; release/runtime 격리만 검사
- `.git`, `target`, `output`, editor swap 내용

## 4. Remediation Inventory

| 보고서 16 요구 | 현재 시정 |
| --- | --- |
| owner approval 원본/scope 연결 | `PROJECT_OWNER_LICENSE_APPROVAL.md`, approval ID, direct instruction 인용, PROV/scenario 연결 |
| distributed Git history 모순 제거 | `NOTICE` 문구 제거, `MODIFICATIONS.md`와 `RELEASE-METADATA`로 교체 |
| actual archive verification | `scripts/verify_release_bundle.sh`, `tests/release_bundle.rs` |
| checksums/commit binding | `SHA256SUMS`, export-subst metadata, build scripts |
| compatibility index sync | 10행 `Approved`, parser regression |
| clean commit 양 OS CI | workflow step 구현, 실제 commit/remote result는 pending |

## 5. Verification Evidence

### 5.1 PASS

| 명령/검사 | 결과 |
| --- | --- |
| `scripts/r7_checkpoint.sh` | PASS |
| `scripts/r8_checkpoint.sh` | PASS |
| R8 표적 7개 test target | PASS, 37 tests |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 338 tests |
| `cargo build --workspace --release --locked` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS |
| `cargo deny check licenses bans sources` | PASS |
| 최신 `cargo audit` | PASS, 1166 advisories / 267 dependencies |
| `scripts/r8_tui_core_flow.sh` | PASS |
| `scripts/r6_pty_matrix.sh` | PASS, success/timeout/stale/down |
| `scripts/r6_pending_exit_smoke.sh` | PASS, 291ms |
| `git diff --check` | PASS |

### 5.2 HOLD / NOT RUN

| 검사 | 결과 |
| --- | --- |
| `./build.sh --release` | expected fail-closed, exit 1: dirty working tree |
| 공식 R8 clean commit package | NOT RUN |
| 동일 commit Ubuntu/Windows CI | NOT RUN |
| immutable approval commit reference | pending clean commit |
| bundle approval-ID resolvability | FAIL by inspection/positive fixture gap, `DBG-F007` |

## 6. Pass 1: Implementation Compliance Findings

### [IMP-F012] project-owner 승인 authority 추적 — Re-audit #5

- Pass: Implementation
- Pattern: IMP-001, SPEC-GAP-001
- Area: approval authority, SC-LICENSE-01
- Severity: **Major**
- Status: **Partially Verified / Hold**
- Summary: 승인 결정의 내용·범위·경계는 독립 파일과 고유 ID로 복구됐지만, record 자체가 아직 uncommitted이고 release verifier가 그 record의 archive 포함을 강제하지 않는다.
- Evidence:
  - `PROJECT_OWNER_LICENSE_APPROVAL.md`는 direct user instruction 두 건을 인용하고 whole-work NGPL, PROV-0001..0012, NH367-C001..C010, notice/source 범위를 명시한다.
  - record는 `qualified legal opinion: not claimed`와 외부 게시 별도 승인을 분리한다.
  - PROVENANCE와 scenario 10개가 `AIHACK-OWNER-2026-07-20-NGPL-01`을 참조한다.
  - record status 자체가 `immutable Git reference pending commit`이라고 명시한다.
  - `DBG-F007` 때문에 source archive에 approval record가 없어도 bundle verifier가 PASS한다.
- Expected: 승인 내용이 고유 ID로 기록되고, clean commit과 release archive에서 그 ID가 실제 approval record로 해석 가능해야 한다.
- Actual: working-tree 기록은 적절하지만 immutable commit과 bundle-level reference integrity가 아직 없다.
- Impact: local documentation closure는 확인되지만 release recipient 관점의 authority trace는 아직 final gate가 아니다.
- Suggested Fix: `DBG-F007` 수정 후 clean R8 commit에 approval record를 포함하고, archive의 ID/file/content 연결을 검증한다.
- Re-audit Method: source archive에서 `PROJECT_OWNER_LICENSE_APPROVAL.md`를 추출해 approval ID/scope를 metadata·PROVENANCE와 대조하고 commit SHA를 확인한다.
- Owner: Project owner / Coder / Release manager
- Notes: 승인 내용 자체를 새로 창작할 필요는 없다. 현재 기록의 immutable·bundle 연결만 닫으면 된다.

### [IMP-F014] NGPL modification notice와 source bundle 모순 — Re-audit #1

- Pass: Implementation
- Pattern: IMP-001, BUILD-001
- Area: modification notice, source distribution
- Severity: **Major**
- Status: **Verified for the approved engineering contract**
- Evidence:
  - `NOTICE`에서 배포된 Git history 주장이 제거됐다.
  - `MODIFICATIONS.md`가 path scope, 기간, 변경 내용을 제공하고 `.git` 의존이 없음을 명시한다.
  - `RELEASE-METADATA`가 export-subst로 commit을 확장한다.
  - source archive positive fixture는 commit expansion·required files·checksums를 PASS하고 blocked legacy 포함 fixture는 FAIL한다.
  - owner decision record가 이 notice/source 방식을 프로젝트 engineering distribution contract로 승인한다.
- Expected / Actual: report 16에서 요구한 bundle-carried modification evidence와 실제 archive 정합성이 구현됐다.
- Remaining Boundary: 최종 법률 해석은 이번 기술 감사 범위 밖이며 qualified legal opinion을 주장하지 않는다.
- Re-audit Method: `DBG-F007` 수정과 clean commit 뒤 실제 archive에서 동일 evidence를 재확인한다.
- Owner: Coder / Project owner

### [IMP-F015] compatibility 인덱스와 개별 record 불일치 — Re-audit #1

- Pass: Implementation
- Pattern: IMP-002
- Area: documentation sync
- Severity: **Minor**
- Status: **Verified**
- Evidence: 인덱스 10행이 모두 `Approved`; 각 record 10개도 `provenance_status: Approved`; `compatibility_index_matches_all_ten_approved_records` PASS.
- Expected / Actual: 10개 index/record가 1:1로 동일 상태를 표시한다.
- Re-audit Method: 전체 문서 회귀에 계속 포함한다.
- Owner: Coder

## 7. Pass 2: Debug / Engineering Quality Findings

### [DBG-F006] clean release commit·실제 bundle·same-commit CI — Re-audit #1

- Pass: Debug
- Pattern: BUILD-001, TEST-001
- Area: release reproducibility, CI
- Severity: **Major**
- Status: **Partially Verified / Hold**
- Evidence:
  - `tests/release_bundle.rs`가 임시 clean Git commit과 실제 `git archive`를 사용한다.
  - Linux verifier가 required files, expanded commit, checksum, legacy exclusion을 검사한다.
  - CI가 Ubuntu `./build.sh --release`와 Windows `cmd /c build.bat --release` step을 포함한다.
  - 현재 공식 tree는 여전히 `04775c1` + dirty R8 changes이며 `build.sh --release`는 exit 1이다.
- Expected: 최종 R8 clean commit에서 양 OS CI와 platform bundle이 PASS해야 한다.
- Actual: 구현/fixture는 green이지만 공식 commit과 remote evidence는 없다.
- Impact: SC-BUILD-02와 R8 final release는 계속 미충족이다.
- Suggested Fix: `DBG-F007` 수정 후 R8 commit을 만들고 같은 SHA의 Ubuntu/Windows CI 및 실제 bundle checksum을 제출한다.
- Re-audit Method: commit SHA, CI jobs, Linux/Windows artifact tree와 checksum을 대조한다.
- Owner: Coder / Release manager

### [DBG-F007] release verifier가 owner/modification reference가 끊긴 bundle을 허용함

- Pass: Debug
- Pattern: BUILD-001, TEST-001
- Area: bundle reference integrity, release provenance
- Severity: **Major**
- Status: **Needs Fix**
- Summary: positive fixture가 approval record와 두 metadata reference를 누락한 상태로 PASS하므로, release gate는 승인·modification trace가 실제 bundle에서 해석 가능한지 보장하지 않는다.
- Evidence:
  - `tests/release_bundle.rs:33-35`의 source fixture에는 `LICENSE`, `NOTICE`, `MODIFICATIONS.md`, `RELEASE-METADATA`만 있고 `PROJECT_OWNER_LICENSE_APPROVAL.md`가 없다.
  - 같은 fixture의 output metadata(`tests/release_bundle.rs:86-90`)는 `product`, `version`, `commit`, `source_license`만 포함하고 `owner_approval`, `modification_notice`가 없다.
  - `scripts/verify_release_bundle.sh:24-26`은 archive required file 목록에 approval record를 포함하지 않는다.
  - verifier는 archive metadata에서 version/commit만, output metadata에서 commit만 검사한다. approval/modification ID와 실제 문서 ID의 일치 검사가 없다.
  - 이 불완전한 positive fixture의 `verifier_accepts_commit_bound_bundle_with_notices_and_checksums`가 PASS했다.
- Expected:
  - source archive가 `PROJECT_OWNER_LICENSE_APPROVAL.md`를 반드시 포함한다.
  - archive와 output `RELEASE-METADATA` 모두 `owner_approval=AIHACK-OWNER-2026-07-20-NGPL-01`과 `modification_notice=AIHACK-MODIFICATIONS-2026-07-20-01`을 포함한다.
  - 각 ID가 대응 문서 내부 ID와 일치해야 한다.
  - approval record/ID 또는 modification record/ID 누락·불일치 negative fixture가 FAIL해야 한다.
- Actual: checksum과 commit이 맞으면 authority/modification reference가 끊겨도 PASS한다.
- Impact: 외부 bundle이 프로젝트가 승인한 license evidence를 제공하지 않으면서도 release verification PASS를 받을 수 있다. `IMP-F012`의 최종 closure도 방해한다.
- Suggested Fix:
  1. verifier의 archive required files에 `PROJECT_OWNER_LICENSE_APPROVAL.md`를 추가한다.
  2. archive/output metadata의 owner/modification ID를 모두 강제한다.
  3. approval 문서의 Approval ID와 MODIFICATIONS의 Notice ID를 추출해 metadata와 대조한다.
  4. positive fixture를 실제 완전 bundle로 고치고 missing/mismatched approval/modification negative tests를 추가한다.
  5. Windows release 검증도 같은 reference-integrity 조건을 검사한다.
- Re-audit Method: 4종 누락/불일치 fixture와 완전 fixture를 실행하고 clean source archive 내용을 직접 검사한다.
- Owner: Coder

## 8. Pass 3: Security Findings

새 Critical/Major runtime security finding은 발견되지 않았다.

- 최신 RustSec scan PASS
- cargo-deny licenses/bans/sources PASS
- loopback·schema·payload·stale/timeout/down 회귀 포함 전체 338 tests PASS
- blocked legacy archive 포함 negative fixture PASS
- R7/R8 canonical-root와 dirty-tree fail-closed 유지

`DBG-F007`은 악성 코드 취약점이 아니라 release provenance hard-boundary 누락이므로 Debug finding으로 분류했다.

## 9. Cross-Pass Conflicts

### [XPF-F010] local green preflight와 final release evidence — Re-audit #1

- Related Findings: IMP-F012, DBG-F006, DBG-F007
- Conflict: checkpoint와 전체 로컬 회귀는 green이지만 approval reference가 끊긴 bundle도 verifier를 통과하며 공식 clean commit/CI가 없다.
- Resolution: IMP-F014/015의 remediation scope는 Verified하되 R8 전체는 HOLD한다.
- Gate Impact: `DBG-F007` 수정 및 `DBG-F006` same-commit evidence 전에는 PASS 불가.

## 10. Required Fixes Before PASS

1. `DBG-F007`의 approval/modification reference-integrity 검증과 negative fixtures를 추가한다.
2. 완전한 positive fixture에 approval record와 두 ID를 포함한다.
3. 시정 tree를 clean R8 commit으로 만들고 immutable approval reference를 확정한다.
4. 같은 commit에서 Ubuntu/Windows CI와 실제 release bundle을 PASS한다.
5. R8 전체 gate와 수동 PTY 3종을 재실행한다.

## 11. Accepted Risks

새 release risk acceptance는 없다.

- 실제 remote LLM provider smoke는 spec상 비차단이며 이번에도 호출하지 않았다.
- qualified legal opinion 미주장은 감사 범위 경계다. owner가 승인한 engineering contract 구현 여부만 판정했다.
- same-commit CI와 bundle reference 무결성은 Accepted Risk가 아니라 release blocker다.

## 12. Needs Spec Clarification

없음. 남은 기술 수정과 release evidence 조건은 현재 문서로 충분히 명확하다.

## 13. Re-audit Checklist

- [ ] archive에 `PROJECT_OWNER_LICENSE_APPROVAL.md` 필수
- [ ] archive/output metadata owner/modification ID 필수
- [ ] metadata ID와 실제 Approval ID/Notice ID 일치
- [ ] 누락/불일치 negative fixtures FAIL
- [ ] 완전 positive fixture PASS
- [ ] R7/R8 checkpoint PASS
- [ ] 표적 및 전체 338+ tests PASS
- [ ] fmt / clippy / release build / audit / deny PASS
- [ ] clean R8 commit package PASS
- [ ] same-commit Ubuntu/Windows CI PASS
- [ ] PTY core/degraded/pending-exit PASS
- [ ] `git diff --check` PASS

## 14. Final Decision

**HOLD — report 16 remediation is substantially verified; release bundle authority trace and same-commit evidence remain**

| Gate/Finding | 상태 |
| --- | --- |
| `IMP-F012` owner approval trace | Partially Verified / Hold |
| `IMP-F014` modification evidence | **Verified** |
| `IMP-F015` compatibility sync | **Verified** |
| `DBG-F006` clean package / same-commit CI | Partially Verified / Hold |
| `DBG-F007` bundle reference integrity | **Needs Fix** |
| R7/R8 checkpoint | PASS |
| Targeted regression | PASS, 37 tests |
| Full workspace regression | PASS, 338 tests |
| fmt / clippy / release build | PASS |
| RustSec / cargo-deny | PASS |
| Manual PTY matrix | PASS |
| Official clean R8 release package | NOT RUN / HOLD |
| Same-commit Linux/Windows CI | NOT RUN / HOLD |
| External distribution | **BLOCKED** |
| R8/final release | **HOLD** |

이번 재감사에서 소스 코드, 설정, 기존 문서는 수정하지 않았다. `audit_report_17.md`만 추가했다.
