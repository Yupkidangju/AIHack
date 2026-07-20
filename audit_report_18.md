# AIHack R8 Remediation Re-audit Report 18

감사 기준: `AI_AUDIT_DOC_STANDARD.md`

감사 유형: `audit_report_17.md` 시정에 대한 독립 3-pass 재감사

감사 일자: 2026-07-20 (Asia/Seoul)

기준 commit: `04775c189fba59146835f6a9055f5606714b90f4` (`main`, `origin/main`) + 현재 R8 remediation working tree

환경: Linux 7.0.0-28-generic x86_64, rustc/cargo 1.94.1

코드·설정 수정: 없음

감사 산출물: 이 보고서만 추가

## 1. Audit Summary

최종 판정: **HOLD — bundle reference remediation is partial; exact-value validation and same-commit release evidence remain**

`audit_report_17.md`의 `DBG-F007` 시정은 실질적으로 진전됐다. Linux/Windows release 경로와 positive fixture가 `PROJECT_OWNER_LICENSE_APPROVAL.md`, `owner_approval`, `modification_notice`를 포함하며, 승인 record 누락·metadata 누락·record ID 불일치·legacy tree 포함 negative case가 모두 fail-closed한다.

그러나 Linux verifier의 `require_text`는 metadata 값을 완전한 key/value 행으로 비교하지 않고 `grep -Fq` 부분 문자열로 검사한다. 그 결과 다음과 같은 잘못된 값도 현재 matcher를 통과한다.

- `owner_approval=AIHACK-OWNER-2026-07-20-NGPL-01-TAMPERED`
- `modification_notice=AIHACK-MODIFICATIONS-2026-07-20-01-TAMPERED`

`tests/release_bundle.rs`에도 mismatched metadata fixture는 없고, missing metadata와 mismatched record만 있다. 따라서 `DBG-F007`은 **Partially Verified / Needs Fix**이며 release authority reference hard-boundary는 아직 완결되지 않았다.

또한 공식 저장소는 여전히 R7 commit `04775c1` 위의 dirty R8 tree다. **이번 재감사는 감사 지적사항의 재수정 전 working tree를 대상으로 했으며, 코더는 해당 수정이 끝나기 전 단계이므로 R8 변경의 commit·push를 아직 진행하지 않았다.** 이는 별도의 신규 코드 결함이 아니라 현재 작업 순서와 evidence 상태를 명시한 것이다. `./build.sh --release`는 의도대로 fail-closed하며, 재수정 후 clean R8 commit package와 그 동일 SHA의 Ubuntu/Windows CI가 제출되기 전까지 `DBG-F006`과 immutable approval reference는 Hold다.

로컬 R7/R8 checkpoint, 표적 39개, 전체 340개 테스트, fmt, Clippy, release build, cargo-deny, 최신 RustSec 및 PTY 3종은 PASS했다. 기술 회귀는 발견되지 않았지만 Major release finding이 남아 R8 전체와 외부 배포는 HOLD다.

## 2. Audit Scope

### 2.1 확인한 문서

- `AI_AUDIT_DOC_STANDARD.md`
- `audit_report_16.md`, `audit_report_17.md`
- `spec.md`, `designs.md`, `DESIGN_DECISIONS.md`
- `IMPLEMENTATION_SUMMARY.md`, `BUILD_GUIDE.md`, `audit_roadmap.md`
- `DOCUMENTATION_AUDIT_REPORT.md`, `GAP_CLOSURE_ROADMAP.md`
- `README.md`, `CHANGELOG.md`, `LESSONS_LEARNED.md`
- `PROVENANCE.md`, `PROJECT_OWNER_LICENSE_APPROVAL.md`
- `NOTICE`, `MODIFICATIONS.md`, `RELEASE-METADATA`
- `docs/R7_COMPATIBILITY_REPORT.md`, `docs/compatibility/**`

### 2.2 확인한 구현·설정·테스트

- `build.sh`, `build.bat`, `.gitattributes`
- `scripts/verify_release_bundle.sh`, `scripts/r8_checkpoint.sh`
- `.github/workflows/ci.yml`
- `tests/release_bundle.rs`, `tests/release_gate.rs`
- `tests/license_compliance.rs`, `tests/r8_documentation.rs`
- `tests/build_contract.rs`, `tests/provenance_manifest.rs`
- 전체 workspace source, manifest와 lockfile의 회귀 범위

### 2.3 검사한 핵심 케이스

- 완전한 commit-bound source bundle
- approval record 누락
- owner metadata 누락
- owner record ID 불일치
- modification metadata 누락
- modification record ID 불일치
- blocked legacy tree 포함
- metadata expected ID 뒤에 임의 접미사가 붙는 부분 문자열 우회
- dirty worktree release fail-closed
- R7/R8 checkpoint, 전체 test/build/lint/security/supply-chain gate
- TUI core flow, LLM success/timeout/stale/down, pending-exit PTY

### 2.4 현재 repository 상태

- `HEAD`와 `origin/main`: `04775c1`, R7 commit
- R8/remediation tree: 다수 tracked 변경과 신규 untracked 파일
- 감사 지적사항 재수정 전 상태이므로 코더의 R8 commit·push는 아직 진행되지 않음
- 공식 R8 clean commit: 없음
- same-commit Ubuntu/Windows CI: 없음
- release command: dirty tree에서 의도대로 exit 1

## 3. Excluded Scope

- 실제 외부 게시·배포·릴리스
- Windows host에서의 수동 `build.bat --release` 실행
- GitHub Actions 원격 실행 결과: 해당 R8 commit이 아직 없음
- clean official R8 commit의 실제 Linux/Windows artifact 비교
- 실제 유료/원격 LLM provider 호출
- NGPL 의무의 최종 법률 판단; 프로젝트가 승인한 engineering distribution contract의 구현 정합성만 감사
- legacy reference tree의 기능·법률 내용; release/runtime 격리만 검사
- `.git`, `target`, `output`, editor swap 내용

## 4. Remediation Inventory

| 보고서 17 요구 | 현재 시정 | 판정 |
| --- | --- | --- |
| archive에 approval record 필수 | Linux/Windows와 fixture에 추가 | Verified |
| archive/output metadata에 두 ID 필수 | 생성 경로와 verifier에 추가 | 구조적 반영 |
| metadata와 record ID 대조 | hard-coded ID 및 record phrase 검사 추가 | 부분 문자열 허용으로 미완결 |
| 누락·불일치 negative fixtures | 5개 case 추가 | metadata 불일치 fixture 누락 |
| clean R8 commit | 없음 | Hold |
| same-commit Ubuntu/Windows CI | workflow step만 존재 | Hold |

## 5. Verification Evidence

### 5.1 PASS

| 명령/검사 | 결과 |
| --- | --- |
| `scripts/r7_checkpoint.sh` | PASS |
| `scripts/r8_checkpoint.sh` | PASS |
| R8 표적 7개 test target | PASS, 39 tests |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 340 tests |
| `cargo build --workspace --release --locked` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS |
| `cargo deny check licenses bans sources` | PASS |
| 최신 `cargo audit` | PASS, 1166 advisories / 267 dependencies |
| `scripts/r8_tui_core_flow.sh` | PASS |
| `scripts/r6_pty_matrix.sh` | PASS, success/timeout/stale/down |
| `scripts/r6_pending_exit_smoke.sh` | PASS, 289ms |
| `git diff --check` | PASS |

### 5.2 환경 분리 기록

- 기본 sandbox의 전체 test는 loopback bind 권한 차단으로 `llm_transport` 6건이 `Operation not permitted`를 반환했다.
- 같은 명령을 loopback이 허용된 확장 환경에서 재실행해 전체 340개 PASS를 확인했다.
- 기본 sandbox의 RustSec DB 갱신은 네트워크 차단으로 실패했고, 확장 환경에서 최신 DB를 가져와 PASS했다.
- 위 두 초기 실패는 저장소 결함이 아니라 실행 환경 제약으로 분류한다.

### 5.3 HOLD / FAIL-CLOSED

| 검사 | 결과 |
| --- | --- |
| `./build.sh --release` | expected fail-closed, exit 1: dirty working tree |
| 공식 R8 clean commit package | NOT RUN |
| 동일 commit Ubuntu/Windows CI | NOT RUN |
| immutable approval commit reference | pending clean commit |
| exact metadata ID validation | FAIL by matcher inspection/demonstration |

## 6. Pass 1: Implementation Compliance Findings

### [IMP-F012] project-owner 승인 authority 추적 — Re-audit #6

- Pass: Implementation
- Pattern: IMP-001, SPEC-GAP-001
- Area: approval authority, SC-LICENSE-01
- Severity: **Major**
- Status: **Partially Verified / Hold**
- Summary: 승인 내용·범위·고유 ID와 bundle 포함 계약은 구현됐지만 record가 아직 uncommitted이고 Linux verifier의 metadata ID 비교가 exact하지 않다.
- Evidence:
  - `PROJECT_OWNER_LICENSE_APPROVAL.md`는 direct user instruction, whole-work NGPL 범위, PROV/scenario 연결, 법률 의견 비주장을 명시한다.
  - Linux/Windows package 경로 모두 approval record와 owner/modification ID를 포함한다.
  - 현재 record status는 `immutable Git reference pending commit`이다.
  - `HEAD`/`origin/main`은 계속 `04775c1`이며 approval record는 untracked다.
  - `DBG-F007`의 suffix 우회 때문에 Linux final verifier가 정확한 metadata reference를 보장하지 못한다.
- Expected: approval record가 clean release commit과 source archive에 포함되고 metadata 값이 대응 record의 ID와 정확히 일치해야 한다.
- Actual: working-tree 내용과 기본 bundle chain은 존재하지만 immutable commit과 exact Linux validation이 없다.
- Impact: 수신자 관점의 authority trace가 아직 final release gate로 확정되지 않았다.
- Suggested Fix: `DBG-F007` exact validation을 닫은 뒤 clean R8 commit에 record를 포함하고 같은 commit의 실제 archive를 검증한다.
- Re-audit Method: clean source archive에서 record를 추출해 metadata의 파싱된 exact 값, record header와 commit SHA를 대조한다.
- Owner: Project owner / Coder / Release manager

### [IMP-F014] NGPL modification notice와 source bundle 모순 — Re-audit #2

- Pass: Implementation
- Pattern: IMP-001, BUILD-001
- Area: modification notice, source distribution
- Severity: **Major**
- Status: **Verified for the approved engineering contract**
- Evidence: `NOTICE`, `MODIFICATIONS.md`, commit-expanded metadata, source archive, checksum과 legacy exclusion 계약이 계속 일치하며 전체 회귀가 PASS했다.
- Remaining Boundary: final legal opinion과 실제 외부 배포는 감사 범위 밖이다.
- Re-audit Method: clean commit artifact에서 같은 evidence를 재확인한다.
- Owner: Coder / Project owner

### [IMP-F015] compatibility 인덱스와 개별 record 불일치 — Re-audit #2

- Pass: Implementation
- Pattern: IMP-002
- Area: documentation sync
- Severity: **Minor**
- Status: **Verified**
- Evidence: index 10행과 record 10개가 모두 `Approved`이며 parser regression과 전체 회귀가 PASS했다.
- Re-audit Method: 전체 문서 회귀에 계속 포함한다.
- Owner: Coder

## 7. Pass 2: Debug / Engineering Quality Findings

### [DBG-F006] clean release commit·실제 bundle·same-commit CI — Re-audit #2

- Pass: Debug
- Pattern: BUILD-001, TEST-001
- Area: release reproducibility, CI
- Severity: **Major**
- Status: **Partially Verified / Hold**
- Evidence:
  - clean fixture archive와 Linux/Windows CI release step은 구현돼 있다.
  - local release build, 340 tests, lint와 supply-chain gates는 green이다.
  - 공식 tree는 `04775c1` + dirty R8 changes이고 `./build.sh --release`는 exit 1이다.
  - 이번 감사 시점은 지적사항 재수정 전이므로 코더가 R8 변경을 아직 commit·push하지 않은 상태다.
  - 실제 R8 SHA, 동일 SHA의 Ubuntu/Windows CI 및 platform bundle evidence가 없다.
- Expected: 최종 clean R8 commit에서 두 OS release job과 실제 bundle 검증이 PASS해야 한다.
- Actual: local implementation은 준비됐지만 감사 지적사항 재수정과 그 후속 commit·push가 아직 수행되지 않아 공식 same-commit evidence는 없다. 이 미제출 상태는 현재 단계의 명시적 선후관계이며, clean commit 이후에만 최종 검증할 수 있다.
- Impact: SC-BUILD-02와 R8 final release는 계속 미충족이다.
- Suggested Fix: `DBG-F007`을 닫고 clean R8 commit을 만든 뒤 동일 SHA의 Ubuntu/Windows CI 결과와 artifact checksum을 제출한다.
- Re-audit Method: commit SHA, 두 CI job, archive tree, metadata와 checksum을 동일 SHA로 대조한다.
- Owner: Coder / Release manager

### [DBG-F007] release bundle authority reference exactness — Re-audit #1

- Pass: Debug
- Pattern: BUILD-001, TEST-001
- Area: bundle reference integrity, release provenance
- Severity: **Major**
- Status: **Partially Verified / Needs Fix**
- Summary: 보고서 17의 누락 문제는 대부분 수정됐지만 Linux verifier가 key/value exact match 대신 부분 문자열을 허용해 잘못된 metadata ID도 PASS할 수 있다.
- Evidence:
  - `scripts/verify_release_bundle.sh:16-18`의 `require_text`는 `grep -Fq "$needle"`을 사용한다.
  - `scripts/verify_release_bundle.sh:55-59`의 owner/modification 검증은 이 helper를 그대로 사용한다.
  - 실제 matcher demonstration에서 expected ID 뒤에 `-TAMPERED`를 붙인 두 candidate 모두 `ACCEPTED_BY_CURRENT_MATCHER`가 됐다.
  - `tests/release_bundle.rs:24-31`에는 `MismatchedOwnerMetadata`, `MismatchedModificationMetadata` case가 없다.
  - `tests/release_bundle.rs:223-248`은 missing metadata와 mismatched record를 검증하지만 metadata의 wrong/suffix value는 검증하지 않는다.
  - 반면 Windows 경로는 `findstr /x`로 metadata 행 전체를 검사해 OS 간 gate 강도가 다르다.
- Expected:
  - metadata를 key/value로 파싱하거나 전체 행 exact match로 검증한다.
  - 각 required key가 정확히 한 번 존재하고 값 전체가 expected ID와 일치해야 한다.
  - archive와 output에 wrong value, suffix value, duplicate key를 넣은 negative fixture가 FAIL해야 한다.
  - Linux와 Windows가 같은 validation strength를 가져야 한다.
- Actual: substring을 포함하면 값 전체가 달라도 Linux verifier가 reference를 수락할 수 있다.
- Impact: 변형되거나 모호한 approval/modification reference가 final Linux release verifier를 통과할 수 있어 authority chain을 hard-boundary로 신뢰할 수 없다.
- Suggested Fix:
  1. metadata를 `key=value` 단위로 파싱하고 exact value·단일 key를 강제한다.
  2. archive/output 각각 owner/modification의 wrong, suffix, duplicate negative fixture를 추가한다.
  3. commit/version도 같은 exact parser를 사용해 verifier 전체의 비교 강도를 통일한다.
  4. Linux/Windows gate가 동일 fixture contract를 공유하도록 한다.
- Re-audit Method: wrong/suffix/duplicate metadata bundle을 생성해 두 platform verifier가 모두 non-zero인지 확인하고 완전 bundle만 PASS하는지 확인한다.
- Owner: Coder

## 8. Pass 3: Security Findings

새 Critical/Major runtime security finding은 발견되지 않았다.

- 최신 RustSec scan PASS
- cargo-deny licenses/bans/sources PASS
- loopback·schema·payload·stale/timeout/down 회귀 포함 전체 340 tests PASS
- PTY core/degraded/pending-exit PASS
- legacy archive exclusion negative fixture PASS
- dirty-tree release fail-closed 유지

`DBG-F007`은 runtime 침해 취약점이 아니라 release provenance hard-boundary 결함이므로 Debug finding으로 유지한다.

## 9. Cross-Pass Conflicts

### [XPF-F010] local green preflight와 final release evidence — Re-audit #2

- Related Findings: IMP-F012, DBG-F006, DBG-F007
- Conflict: checkpoint와 전체 로컬 회귀는 green이지만 Linux reference validation은 exact하지 않고 공식 clean commit/CI가 없다.
- Resolution: 구현 회귀와 `IMP-F014`/`IMP-F015` closure는 Verified하되 R8 전체는 HOLD한다.
- Gate Impact: `DBG-F007` exactness와 `DBG-F006` same-commit evidence 전에는 PASS 불가다.

## 10. Required Fixes Before PASS

1. Linux verifier의 owner/modification metadata를 exact key/value 및 단일 key로 검증한다.
2. wrong/suffix/duplicate metadata negative fixtures를 archive/output에 추가한다.
3. 가능하면 commit/version도 동일 exact parser로 통일한다.
4. 완전 positive fixture와 모든 negative fixture를 재실행한다.
5. 시정 tree를 clean R8 commit으로 만들고 immutable approval reference를 확정한다.
6. 같은 commit에서 Ubuntu/Windows CI와 실제 release bundle을 PASS한다.
7. R8 전체 gate와 수동 PTY 3종을 재실행한다.

## 11. Accepted Risks

새 release risk acceptance는 없다.

- 실제 remote LLM provider smoke는 spec상 비차단이며 호출하지 않았다.
- qualified legal opinion 미주장은 감사 범위 경계다.
- exact metadata validation과 same-commit CI는 Accepted Risk가 아니라 release blocker다.

## 12. Needs Spec Clarification

없음. 남은 exact validation과 release evidence 조건은 현재 문서와 finding으로 충분히 명확하다.

## 13. Re-audit Checklist

- [ ] Linux metadata exact key/value 및 단일 key 검증
- [ ] wrong/suffix/duplicate owner metadata fixture FAIL
- [ ] wrong/suffix/duplicate modification metadata fixture FAIL
- [ ] archive/output 양쪽 동일 contract
- [ ] Linux/Windows validation strength 일치
- [ ] 완전 positive fixture PASS
- [ ] R7/R8 checkpoint PASS
- [ ] 표적 39+ 및 전체 340+ tests PASS
- [ ] fmt / clippy / release build / audit / deny PASS
- [ ] clean R8 commit package PASS
- [ ] same-commit Ubuntu/Windows CI PASS
- [ ] PTY core/degraded/pending-exit PASS
- [ ] `git diff --check` PASS

## 14. Final Decision

**HOLD — bundle reference remediation is partial; exact-value validation and same-commit release evidence remain**

| Gate/Finding | 상태 |
| --- | --- |
| `IMP-F012` owner approval trace | Partially Verified / Hold |
| `IMP-F014` modification evidence | **Verified** |
| `IMP-F015` compatibility sync | **Verified** |
| `DBG-F006` clean package / same-commit CI | Partially Verified / Hold |
| `DBG-F007` bundle reference integrity | **Partially Verified / Needs Fix** |
| R7/R8 checkpoint | PASS |
| Targeted regression | PASS, 39 tests |
| Full workspace regression | PASS, 340 tests |
| fmt / clippy / release build | PASS |
| RustSec / cargo-deny | PASS |
| PTY 3종 | PASS |
| clean official R8 release | NOT RUN / HOLD |

판단 근거: 보고서 17에서 지적한 record·metadata 누락은 대부분 해소됐으나, release authority reference를 완전 일치로 강제하지 않는 Linux verifier 결함과 공식 same-commit evidence 부재는 `AI_AUDIT_DOC_STANDARD.md`의 Major/Phase gate 규칙상 전체 PASS를 허용하지 않는다.

코더 재수정 후에는 `DBG-F007` exact metadata negative cases를 먼저 재감사하고, 그 결과가 green일 때에만 clean commit·양 OS CI의 `DBG-F006` 최종 evidence로 진행한다.
