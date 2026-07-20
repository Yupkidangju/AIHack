# AIHack R8 Integrated Release Audit Report 16

감사 기준: `AI_AUDIT_DOC_STANDARD.md`

감사 유형: R8 완료 주장에 대한 독립 3-pass 통합 감사

감사 일자: 2026-07-20 (Asia/Seoul)

기준 commit: `04775c189fba59146835f6a9055f5606714b90f4` (`main`, `origin/main`) + 현재 R8 working tree

환경: Linux 7.0.0-28-generic x86_64, rustc/cargo 1.94.1

코드·설정 수정: 없음

감사 산출물: 이 보고서만 추가

## 1. Audit Summary

최종 판정: **HOLD — R8 local engineering gates pass, but release authority and same-commit release evidence are incomplete**

R8에서 추가한 version 0.3.0 동기화, workspace `NGPL`, 공식 `LICENSE` checksum, `NOTICE`, R7/R8 checkpoint, license/release/document 회귀 테스트와 PTY 흐름은 로컬 기술 검증을 통과했다. 포맷, Clippy, 전체 333개 테스트, release build, 최신 RustSec DB 기반 `cargo audit`, `cargo deny`, R8 수동 TUI/LLM matrix도 PASS다.

그러나 이것만으로 R8 최종 릴리스 PASS를 선언할 수 없다.

1. `IMP-F012`를 닫는 project-owner 승인은 현재 working tree 문서가 자기 자신을 근거로 주장할 뿐, 이 변경 밖의 권한 있는 승인 기록으로 추적되지 않는다.
2. NGPL 2(a)의 modified-file notice 요구에 대해 root `NOTICE` 하나로 충족한다고 선언했지만, `NOTICE`가 근거로 삼는 “distributed Git history”는 `git archive HEAD` 산출물에 포함되지 않는다.
3. R8 변경은 아직 commit되지 않았다. 현재 `HEAD`는 R7 commit이고 `build.sh --release`는 의도대로 dirty worktree를 거부한다. 따라서 실제 R8 source bundle과 같은 commit의 Linux/Windows CI evidence가 없다.
4. compatibility 인덱스의 10개 provenance 상태가 개별 record의 `Approved`와 달리 여전히 `Reviewed`다.

현재 finding은 국소적으로 시정 가능하므로 `REWORK REQUIRED`는 아니다. 다만 Major release blocker가 남아 있으므로 외부 artifact 게시와 R8 최종 PASS는 보류한다.

## 2. Audit Scope

### 2.1 프로젝트 및 범위

- 프로젝트: Rust 2021 workspace 기반 TUI/headless roguelike
- release target: AIHack v0.3.0
- source: `src/**`, `crates/**`, `apps/**`
- tests: root `tests/**`, package-local `tests/**`
- manifests/config: 8개 `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `deny.toml`, `.github/workflows/ci.yml`
- release/security: `LICENSE`, `NOTICE`, `.gitattributes`, `PROVENANCE.md`, `build.sh`, `build.bat`, `scripts/r7_checkpoint.sh`, `scripts/r8_checkpoint.sh`
- R8 runtime evidence: `scripts/r8_tui_core_flow.sh`, `scripts/r6_pty_matrix.sh`, `scripts/r6_pending_exit_smoke.sh`
- governing docs: `spec.md`, `designs.md`, `DESIGN_DECISIONS.md`, `BUILD_GUIDE.md`, `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`, `README.md`, `CHANGELOG.md`, `DOCUMENTATION_AUDIT_REPORT.md`
- lineage: `audit_report_12.md`~`audit_report_15.md`, `docs/R7_COMPATIBILITY_REPORT.md`, `docs/compatibility/**`

### 2.2 변경 인벤토리

- tracked 변경: 37 files
- untracked: R8 산출물 8개와 기존 editor swap 1개
- source `.rs`: 152 files
- test `.rs`: 73 files
- CI workflow: 1 file, Ubuntu/Windows matrix
- 현재 `HEAD`와 `origin/main`: 모두 `04775c1`, R8 변경은 미커밋

### 2.3 감사 해석

- 사용자의 “R8 완료”는 완료 주장에 대한 감사 개시로 해석했다. 외부 게시·배포 실행 권한으로 해석하지 않았다.
- `audit_report_15.md`가 R8 전까지 수용한 license defer는 R8에서 만료된다. 따라서 SC-LICENSE-01과 distribution notice/source 조건은 이번 감사의 hard gate다.
- 법률 자문이나 파생물 여부의 법적 결론은 제공하지 않는다. 대신 프로젝트가 선택한 NGPL 계약과 실제 산출물의 정합성, 승인 권한의 추적성, fail-closed 동작을 감사한다.

## 3. Excluded Scope

- 실제 외부 게시, 배포, 릴리스 생성 및 운영 배포
- Windows host에서의 `build.bat` 실행과 GitHub Actions 원격 결과
- 실제 유료/원격 LLM provider 호출
- 법률 자문, 저작권 소유권 또는 파생물 여부에 대한 최종 법적 판단
- `legacy_nethack_port_reference/**`의 코드 품질과 기능 정확성; direct import/배포 격리 경계만 검사
- `.git/**`, `target/**`, `output/**`, editor swap 파일 내용
- clean R8 commit 기반 실제 source archive 검사: 현재 R8 tree가 미커밋이므로 생성 자체가 fail-closed로 차단됨

## 4. Documents and Files Examined

### 4.1 문서

- `AI_AUDIT_DOC_STANDARD.md` 전체
- 위 2.1의 governing docs와 R7/R8 lineage
- `PROVENANCE.md`, `docs/compatibility/README.md`, NH367-C001..C010 10개 record
- `LICENSE`, `NOTICE`

### 4.2 구현·설정·테스트

- R8 변경 전체 diff
- 모든 workspace manifest와 lockfile
- Linux/Windows build script, R7/R8 checkpoint, PTY scripts
- `tests/license_compliance.rs`
- `tests/release_gate.rs`
- `tests/r8_documentation.rs`
- `tests/provenance_manifest.rs`
- `tests/build_contract.rs`
- `.github/workflows/ci.yml`, `deny.toml`, `.gitattributes`

### 4.3 외부 기준

- SPDX NGPL 식별자 및 라이선스 원문: <https://spdx.org/licenses/NGPL.html>
- Cargo `license`/`license-file` manifest 규칙: <https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields>
- NetHack 3.6.7 공식 source 배포: <https://www.nethack.org/v367/download-src.html>

`NGPL`은 유효한 SPDX short identifier다. 따라서 manifest의 `license = "NGPL"` 자체는 finding이 아니다.

## 5. Verification Evidence

### 5.1 PASS

| 명령/검사 | 결과 |
| --- | --- |
| `scripts/r7_checkpoint.sh` | PASS, exit 0 |
| `scripts/r8_checkpoint.sh` | PASS, exit 0 |
| R8 표적 6개 test target | PASS, 32 tests |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 333 tests |
| `cargo build --workspace --release --locked` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS, 8 packages 모두 0.3.0/NGPL/publish false |
| `cargo deny check licenses bans sources` | PASS |
| `cargo audit --db /tmp/aihack-advisory-db` | PASS, 최신 fetch 후 1166 advisories/267 dependencies scan |
| `sha256sum LICENSE` | PASS, `93a3ae2c...5a747` |
| blocked legacy import scan | PASS, runtime manifest/source match 0건 |
| `scripts/r8_tui_core_flow.sh` | PASS |
| `scripts/r6_pty_matrix.sh` | PASS, success/timeout/stale/down |
| `scripts/r6_pending_exit_smoke.sh` | PASS, restore-before-worker-wait, 288ms |
| `git diff --check` | PASS |

전체 테스트의 첫 실행에서는 sandbox가 loopback bind를 금지해 `llm_transport` 6개가 `Operation not permitted`로 실패했다. 동일 command를 실제 loopback 권한으로 재실행해 333개가 모두 통과했으므로 이는 repository failure가 아닌 환경 제약으로 분류한다.

### 5.2 HOLD / NOT RUN

| 명령/검사 | 결과 |
| --- | --- |
| `./build.sh --release` | 예상된 fail-closed, exit 1: dirty worktree |
| R8 commit의 source archive 내용 검사 | NOT RUN, R8 변경 미커밋 |
| 동일 R8 commit Linux/Windows CI | NOT RUN, 해당 commit 없음 |
| Windows `build.bat --release` 실제 실행 | NOT RUN |
| 권한 있는 project-owner 승인 원본 대조 | HOLD, repository 내부 자기기술 외 독립 근거 없음 |

### 5.3 실패했으나 대체 검증한 명령

| 명령 | 최초 결과 | 처리 |
| --- | --- | --- |
| `cargo audit --locked` | CLI 0.22.1이 `--locked` 미지원, exit 2 | 문서화된 실제 gate인 `cargo audit`로 전환 |
| `cargo audit` | 기본 DB lock path가 read-only | `/tmp` DB에 최신 RustSec fetch 후 PASS |
| 전체 test 최초 실행 | sandbox loopback bind 거부 | 권한 있는 재실행에서 PASS |
| PTY script 최초 실행 | sandbox tmux socket 거부 | 권한 있는 재실행에서 모두 PASS |

## 6. Prior Finding Lineage

| Finding | 보고서 15 이후 상태 | 이번 판정 |
| --- | --- | --- |
| `IMP-F012` actual license/provenance approval | R8에서 closure 필요 | **Hold / Needs Spec Clarification** |
| `IMP-F013` R7/R8 gate cycle | report 14 Verified | **Verified 유지** |
| `DBG-F005` compatibility trace density | report 13 Verified | **Verified 유지** |
| `SEC-F002` R7 status-only fail-open | report 13 Verified | **Verified 유지** |
| `SEC-F003` caller-controlled checkpoint root | report 14 Verified | **Verified 유지** |
| `XPF-F008` green engineering vs authority boundary | R8로 defer | **Hold, authority evidence 미종결** |
| `XPF-F009` phase sequence conflict | report 14 Verified | **Verified 유지** |

## 7. Pass 1: Implementation Compliance Findings

### [IMP-F012] 실제 project-owner 승인 authority가 독립 evidence로 추적되지 않음 — Re-audit #4

- Pass: Implementation
- Pattern: IMP-001, SPEC-GAP-001
- Area: license authority, provenance approval, SC-LICENSE-01
- Severity: **Major**
- Status: **Needs Spec Clarification / Hold**
- Summary: R8 문서와 record는 project owner가 whole-work NGPL과 10개 scenario를 승인했다고 일괄 선언하지만, 감사자가 이 선언을 실제 승인 원본 또는 권한 있는 주체에게 귀속할 수 있는 근거가 없다.
- Evidence:
  - `audit_report_15.md`는 실제 런칭 시 project owner/qualified reviewer 결정을 요구했고, 당시 허용 범위는 비배포 준비뿐이었다.
  - 현재 ADR-0030, `PROVENANCE.md`, NH367 10개 record는 모두 같은 working-tree 문구인 `Project owner derivative classification`을 evidence로 사용한다.
  - `tests/license_compliance.rs`는 reviewer/date/license/evidence 문자열의 존재를 검증하지만 승인 주체의 실제 권한이나 승인 원본은 검증하지 않는다.
  - R8 변경은 아직 commit되지 않아 Git author/commit message로도 결정을 귀속할 수 없다.
- Expected: project owner 또는 적격 검토자가 정확한 대상(PROV-0001..0012, NH367-C001..C010, whole-work NGPL, NOTICE/source 제공 방식), 결정, 날짜와 권한을 직접 승인한 추적 가능한 기록이 존재해야 한다.
- Actual: coder가 작성한 여러 문서가 동일한 자기기술 문구를 상호 참조한다. 이 문구들이 실제 승인을 반영했는지는 현재 repository evidence만으로 판정할 수 없다.
- Impact: SC-LICENSE-01과 R8 release authority를 독립 감사로 Verified할 수 없으며, 근거 없는 `Approved` 전환 위험이 남는다.
- Suggested Fix:
  1. project owner/qualified reviewer가 직접 승인한 decision record를 추가하거나 기존 ADR에 승인 원본 reference를 연결한다.
  2. 승인 기록에는 reviewer 식별/역할, 승인 범위, 결정 내용, 날짜, notice/source 의무와 서명 또는 변경 불가능한 승인 reference를 포함한다.
  3. 승인이 실제로 없었다면 record를 `Reviewed`로 되돌리고 R8은 HOLD를 유지한다.
- Re-audit Method: 승인 원본과 ADR/PROVENANCE/scenario의 scope를 대조하고, 승인 없는 fixture가 machine gate와 human gate 모두를 통과하지 못하는지 확인한다.
- Owner: Human project owner / qualified license reviewer; coder는 기록 연결과 gate만 구현
- Notes: 이는 “NGPL 선택이 틀렸다”는 법률 판단이 아니다. 승인 authority가 감사 가능한 evidence로 닫혔는지를 판정한 것이다.

### [IMP-F014] NGPL modification notice 계약과 실제 source bundle 설계가 일치하지 않음

- Pass: Implementation
- Pattern: IMP-001, BUILD-001
- Area: NGPL paragraph 2(a), modification notice, source distribution
- Severity: **Major**
- Status: **Needs Spec Clarification / Hold**
- Summary: 프로젝트는 root `NOTICE`의 whole-tree 문단과 “distributed Git history”로 변경 파일·날짜 고지를 충족한다고 선언하지만, release source는 `git archive HEAD`라 Git history를 포함하지 않는다. modified-file notice 방식이 선택한 NGPL 계약을 충족하는지도 qualified review가 없다.
- Evidence:
  - `LICENSE:52-53`은 modified files가 변경 사실과 변경일을 나타내는 prominent notice를 carry해야 한다고 규정한다.
  - `NOTICE:26-31`은 모든 project-owned file의 정밀 변경일/작성자가 distributed Git history에 있고 root 문단이 whole-tree modification notice라고 선언한다.
  - `build.sh:73-79`, `build.bat:76-82`는 `git archive ... HEAD`만 생성한다.
  - `git archive HEAD` tree에는 `.git/` entry가 0건이다. 즉 NOTICE가 약속한 distributed Git history가 source archive에 없다.
  - `src/**`, `crates/**`, `apps/**`의 Rust source에서 file-level NetHack/NGPL 변경일 notice는 검색되지 않았다.
- Expected: qualified reviewer가 승인한 modification-notice 방식과 실제 배포 archive가 동일해야 하며, NOTICE가 가리키는 날짜/저자 근거가 recipient에게 실제 제공돼야 한다.
- Actual: root NOTICE는 배포되지 않는 Git history를 근거로 들고, file-level notice 요구는 whole-tree 문단으로 대체된다고 자체 해석한다.
- Impact: 외부 배포 시 선택한 NGPL 의무를 충족했다는 문서 주장을 입증할 수 없고, recipient에게 제공되는 notice가 실제 bundle과 다르다.
- Suggested Fix:
  1. qualified license reviewer에게 root whole-tree notice가 2(a)를 충족하는지 명시적으로 확인받는다.
  2. 승인되지 않으면 modified files에 변경 사실/날짜 notice를 추가하거나, source archive에 적격한 file-level modification manifest를 포함하고 그 적합성을 승인받는다.
  3. `NOTICE`의 “distributed Git history” 주장을 실제 배포 방식과 일치시키거나, source archive에 추적 가능한 history/manifest를 포함한다.
  4. release test가 실제 archive를 열어 LICENSE, NOTICE, modification evidence와 source file 집합을 검증하게 한다.
- Re-audit Method: clean commit에서 release archive를 생성한 뒤 archive tree와 notice evidence를 대조하고 qualified approval record를 확인한다.
- Owner: Human project owner / qualified license reviewer + coder
- Notes: 감사자는 2(a)의 최종 법적 해석을 대신하지 않는다. 현재 구현과 자체 문서 사이의 객관적 모순을 release blocker로 판정한다.

### [IMP-F015] compatibility 인덱스의 provenance 상태가 개별 승인 record와 불일치

- Pass: Implementation
- Pattern: IMP-002
- Area: documentation forward/backward sync
- Severity: **Minor**
- Status: **Needs Fix**
- Summary: `docs/compatibility/README.md`의 scenario 표는 NH367-C001..C010을 모두 `Reviewed`로 표시하지만 각 record와 본문은 `Approved`라고 선언한다.
- Evidence: `docs/compatibility/README.md` 2절 표 10행은 `Reviewed`; 각 `NH367-C*.md`는 `provenance_status: Approved`; 같은 README 본문은 approval 완료를 선언한다.
- Expected: 인덱스, record, PROVENANCE, checkpoint 결과가 같은 상태를 표시한다.
- Actual: 인덱스 표만 R7 시점 상태로 남아 있다.
- Impact: release reviewer가 어떤 상태가 authority인지 혼동하고 문서 self-check PASS의 신뢰도가 낮아진다.
- Suggested Fix: IMP-F012의 실제 승인 판정에 따라 표 10건과 각 record를 한 방향으로 동기화하고, `tests/r8_documentation.rs`에 표 상태 대조를 추가한다.
- Re-audit Method: 인덱스 10행과 record 10개의 status를 파싱해 1:1 일치하는지 확인한다.
- Owner: Coder

## 8. Pass 2: Debug / Engineering Quality Findings

### [DBG-F006] clean release commit·실제 source bundle·동일 commit CI evidence가 없음

- Pass: Debug
- Pattern: BUILD-001, TEST-001
- Area: release reproducibility, artifact provenance, CI
- Severity: **Major**
- Status: **Hold**
- Summary: 로컬 Cargo release build는 통과하지만 R8이 정의한 실제 release packaging 경로는 아직 실행할 수 없고, 동일 R8 commit의 Linux/Windows CI 결과도 없다.
- Evidence:
  - 현재 `HEAD`/`origin/main`은 R7 commit `04775c1`; R8 변경은 tracked 37개와 untracked R8 파일 8개다.
  - `./build.sh --release`는 dirty worktree를 감지해 exit 1로 올바르게 차단했다.
  - `spec.md` SC-BUILD-02와 18절은 Linux/Windows CI 및 전체 SC PASS를 R8 완료 조건으로 둔다.
  - `audit_roadmap.md`와 `DOCUMENTATION_AUDIT_REPORT.md`도 SC-BUILD-02 remote CI를 pending으로 기록한다.
  - `tests/license_compliance.rs`는 build script 내 문자열만 확인하며 실제 archive 내용이나 binary/source commit 일치를 실행 검증하지 않는다.
- Expected: 모든 R8 수정이 반영된 clean commit에서 Linux/Windows CI가 green이고, Linux/Windows release script가 생성한 binary, LICENSE, NOTICE, source archive의 내용과 commit identity가 검증돼야 한다.
- Actual: fail-closed script 구현과 fixture는 PASS지만 actual release artifact는 NOT RUN이다.
- Impact: binary와 corresponding source의 동일성, Windows packaging, remote CI 재현성을 증명할 수 없어 SC-BUILD-02 및 R8 final completion이 미충족이다.
- Suggested Fix:
  1. IMP-F012/F014/F015 시정 후 R8 변경을 하나의 검토 가능한 commit으로 만든다.
  2. 같은 commit을 push해 Ubuntu/Windows CI 전체 green evidence를 남긴다.
  3. clean commit에서 `build.sh --release`와 Windows `build.bat --release`를 실행한다.
  4. archive를 열어 commit/version, `LICENSE`, `NOTICE`, source/build inputs, export-ignore 결과를 검증하는 회귀 테스트 또는 release verification script를 추가한다.
  5. binary hash, source archive hash와 commit SHA를 release evidence에 기록한다.
- Re-audit Method: same commit SHA의 CI URL/결과, 실제 package tree, checksum, binary 실행 smoke를 재검증한다.
- Owner: Coder / Release manager
- Notes: dirty-tree 차단 자체는 올바른 fail-closed 동작이며 수정 대상이 아니다. 누락된 것은 final release evidence다.

## 9. Pass 3: Security Findings

새 Critical/Major security code finding은 발견되지 않았다.

Verified evidence:

- 최신 RustSec advisory DB 1166건으로 267 dependencies scan PASS
- `cargo deny check licenses bans sources` PASS
- loopback-only transport, credentialed/non-loopback endpoint 거부, redirect/timeout/stale/down, payload/schema/action bounds 전체 회귀 PASS
- blocked legacy import 0건
- R7/R8 checkpoint root는 script-relative이며 inherited override에 영향받지 않음
- release script는 dirty tree에서 fail-closed

단, license authority와 notice는 문서화만으로 hard boundary를 통과할 수 없으므로 `IMP-F012`, `IMP-F014`가 해소되기 전 보안/공급망 PASS를 외부 배포 승인으로 확장하지 않는다.

## 10. Cross-Pass Conflicts

### [XPF-F010] green local preflight와 final release authority/evidence의 충돌

- Related Findings: IMP-F012, IMP-F014, DBG-F006
- Conflict: `scripts/r8_checkpoint.sh`, 모든 test, release build와 supply-chain scan은 PASS지만, 실제 승인 원본·notice 적합성·clean package·same-commit CI는 없다.
- Resolution: local checkpoint PASS를 “R8 구현 preflight PASS”로만 인정한다. R8 전체와 외부 배포는 HOLD다.
- Gate Impact: Major finding 3건이 닫히기 전 R8 PASS 불가.
- Required Fix Before PASS: 권한 있는 approval와 notice 결정을 기록하고, clean commit package 및 같은 commit 양 OS CI evidence를 제출한다.

## 11. Required Fixes Before PASS

1. `IMP-F012`: project owner/qualified reviewer의 실제 승인 원본과 scope를 추적 가능하게 연결한다.
2. `IMP-F014`: NGPL 2(a) notice 방식을 qualified review로 확정하고, `git archive` bundle과 NOTICE의 history 주장을 일치시킨다.
3. `IMP-F015`: compatibility 인덱스와 개별 record 상태를 동기화하고 회귀 테스트를 보강한다.
4. `DBG-F006`: 시정된 R8 clean commit에서 실제 package와 Ubuntu/Windows CI를 검증한다.
5. 위 evidence로 R8 전체 명령과 수동 PTY 3종을 재실행한다.

## 12. Accepted Risks

이번 R8 최종 감사에서 새로 수용한 release risk는 없다.

- report 15의 `AR-F001`은 R8 pre-launch review 시작으로 만료됐다.
- 실제 remote LLM provider smoke는 spec상 비차단이며 이번에도 호출하지 않았다.
- 법률 판단 제외는 risk acceptance가 아니라 감사 권한 경계다.

## 13. Needs Spec Clarification

1. project owner가 2026-07-20 whole-work NGPL과 PROV/NH367 범위를 실제로 승인했는가? 승인했다면 원본 authority/evidence는 무엇인가?
2. NGPL 2(a)의 modified-file notice를 root whole-tree NOTICE 하나로 충족한다고 qualified reviewer가 승인했는가?
3. source archive에 Git history를 넣지 않을 경우 `NOTICE`의 “distributed Git history”를 어떤 실제 artifact로 대체할 것인가?

이 세 질문은 coder가 임의로 답할 수 없다. project owner 또는 적격 license reviewer의 결정이 필요하다.

## 14. Re-audit Checklist

- [ ] IMP-F012 승인 원본과 reviewer authority 확인
- [ ] PROV-0001..0012와 NH367-C001..C010 scope/date/license/notice/evidence 대조
- [ ] IMP-F014 qualified notice 결정 확인
- [ ] 실제 source archive에 modification evidence가 포함되는지 검사
- [ ] `NOTICE`가 실제 bundle 내용을 정확히 설명하는지 검사
- [ ] compatibility 인덱스 10행과 record 10개 상태 일치
- [ ] R7/R8 checkpoint PASS
- [ ] R8 표적 tests PASS
- [ ] fmt / clippy / full 333+ tests / release build PASS
- [ ] latest `cargo audit` / `cargo deny` PASS
- [ ] clean `build.sh --release` artifact tree/checksum/commit 검증
- [ ] Windows `build.bat --release` artifact tree 검증
- [ ] 동일 commit Ubuntu/Windows CI green
- [ ] R8 TUI core flow, degraded matrix, pending-exit PASS
- [ ] `git diff --check` PASS

## 15. Final Decision

**HOLD — R8 local engineering gates pass, but release authority and same-commit release evidence are incomplete**

| Gate | 상태 |
| --- | --- |
| R1 local build / R2~R6 | PASS, 기존 계보 유지 |
| R7 engineering | PASS WITH KNOWN RISKS, 기존 계보 유지 |
| R8 checkpoint | PASS, local preflight only |
| R8 targeted regression | PASS, 32 tests |
| Full workspace regression | PASS, 333 tests |
| Local release build | PASS |
| RustSec / cargo-deny | PASS |
| Manual PTY matrix | PASS |
| `IMP-F012` approval authority | **HOLD** |
| `IMP-F014` modification notice/source evidence | **HOLD** |
| `IMP-F015` compatibility index sync | Needs Fix |
| Clean R8 release package | **NOT RUN / HOLD** |
| Same-commit Linux/Windows CI | **NOT RUN / HOLD** |
| External distribution | **BLOCKED** |
| R8/final release | **HOLD** |

이번 감사에서 소스 코드, 설정, 기존 문서는 수정하지 않았다. 코더는 위 finding만 시정하고, project owner/qualified reviewer가 맡아야 할 승인 판단을 대신 작성하지 않아야 한다.
