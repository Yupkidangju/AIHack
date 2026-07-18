# AIHack D3D R7 Remediation Re-audit Report 14

감사 기준: `AI_AUDIT_DOC_STANDARD.md`

감사 유형: `audit_report_13.md` 지적사항 시정 후 독립 재감사

감사 일자: 2026-07-18 (Asia/Seoul)

감사 대상: 현재 working tree의 `IMP-F013`, `SEC-F003`, `XPF-F009` 시정과 미종결 `IMP-F012`, R7/R8 gate 책임, 전체 회귀·공급망·보안 경계

기준 commit: `eb62984` (`main`, `origin/main`) + R7 remediation working tree

환경: Linux 7.0.0-28-generic x86_64, rustc 1.94.1, cargo 1.94.1, cargo-audit 0.22.1, cargo-deny 0.19.4

감사 중 소스·설정·기존 문서 수정: 없음

이번 감사가 생성한 파일: `audit_report_14.md`

## 1. 감사 요약

최종 판정: **HOLD — report 13 technical findings verified; actual provenance approval remains**

`audit_report_13.md`에서 새로 지적한 기술 문제는 모두 시정됐다.

- `IMP-F013`: R7은 runtime asset과 NH367 scenario provenance만 판정하고, root `Cargo.toml` distribution license·version·packaging은 R7 PASS 뒤 R8 release gate가 소유하도록 master contract와 활성 문서가 정렬됐다.
- `SEC-F003`: checkpoint root가 script-relative canonical repository로 고정됐으며 inherited `AIHACK_R7_ROOT`는 판정에 영향을 주지 않는다.
- `XPF-F009`: R7 asset approval와 R8 외부 배포 조건이 분리돼 순환 의존이 제거됐다. R7 PASS만으로 외부 배포는 허용되지 않는다.

이를 실제 fixture와 실행으로 확인했다. root가 `UNLICENSED`인 complete approval fixture는 R7 checkpoint를 통과했고, 같은 fixture에 잘못된 `AIHACK_R7_ROOT`를 주입해도 결과가 바뀌지 않았다. 현재 실제 tree의 checkpoint는 root license가 아니라 `PROV-0004`와 NH367 10개가 `Reviewed`인 사유만으로 HOLD/exit 1을 반환한다. 표적 42개와 전체 workspace 322개 테스트, fmt/check/clippy, debug/release build, RustSec, cargo-deny와 headless smoke도 모두 통과했다.

남은 유일한 R7 blocker는 `IMP-F012`다. project owner 또는 적격 검토자의 실제 content/scenario license·scope·notice·evidence 승인이 아직 없으므로 SC-LICENSE-01과 Checkpoint R7은 계속 HOLD다. 이는 코더가 근거 없이 상태를 바꿔 닫을 수 있는 코드 결함이 아니다.

| 구분 | 결과 |
| --- | --- |
| `IMP-F013` R7/R8 gate cycle | **Verified** |
| `SEC-F003` caller-controlled audit root | **Verified** |
| `XPF-F009` phase/security conflict | **Verified** |
| `IMP-F012` actual approval | **HOLD**, 변경 없음 |
| 이전 `DBG-F005`, `SEC-F002` | 기존 Verified 유지 |
| R7 표적 test | PASS, 42 tests |
| Full workspace test | PASS, 322 tests |
| Build/lint/supply-chain | PASS |
| `scripts/r7_checkpoint.sh` | 예상된 **HOLD**, exit 1 |
| Critical / Major / Minor open | 0 / 1 / 0 |
| 코더 기술 시정 | **완료** |
| Checkpoint R7 | **HOLD — human approval pending** |

`audit_report_11.md`의 R6 PASS, `audit_report_12.md`의 공식 NetHack 3.6.7 checksum/locator 증거, `audit_report_13.md`의 `DBG-F005`/`SEC-F002` Verified는 유지된다. 이번 판정은 R8 또는 전체 release PASS가 아니다.

## 2. Audit Scope

### 2.1 확인한 문서와 파일

- 기준·계보: `AI_AUDIT_DOC_STANDARD.md`, `audit_report_13.md`
- master contract: `spec.md`, `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`
- 설계·운영: `DESIGN_DECISIONS.md`, `BUILD_GUIDE.md`, `README.md`, `CHANGELOG.md`, `LESSONS_LEARNED.md`
- provenance: `PROVENANCE.md`, `docs/provenance/r7-content.sha256`, `docs/R7_COMPATIBILITY_REPORT.md`
- compatibility: `docs/compatibility/README.md`, `docs/compatibility/NH367-C001..C010*.md`
- gate와 tests: `scripts/r7_checkpoint.sh`, `tests/provenance_manifest.rs`, `tests/nethack_367_compat.rs`, `tests/golden_phase8_rules.rs`
- 연결 회귀: 전체 workspace source, manifests, `Cargo.lock`, `deny.toml`, 전체 tests

### 2.2 확인한 케이스

- `UNLICENSED` root에서 complete R7 asset/scenario approval fixture PASS
- R7 PASS와 R8 root distribution license 변경의 비순환 단계 순서
- R7 PASS 뒤에도 R8 전 외부 배포 차단 유지
- inherited `AIHACK_R7_ROOT`가 checkpoint 검사 tree를 변경하지 못함
- status-only와 missing-field/checksum/schema/coverage/Blocked include negative fixture 유지
- 현재 Reviewed 상태에서 정확한 HOLD 사유와 exit code
- 활성 문서의 R7/R8 책임, 상태, 수용 기준과 실행 명령 정합성
- 전체 workspace 회귀, dependency boundary, supply-chain, secret scan, deterministic headless smoke

## 3. Excluded Scope

- content/scenario의 실제 저작권·파생물 판단과 배포 범위 결정: project owner/적격 검토자 권한이며 본 기술 감사는 법률 자문이 아님
- `Reviewed -> Approved` 실전환: 승인 evidence가 없어 수행하지 않음
- R8 root distribution license 선택·manifest 변경, version 0.3.0, packaging과 final release audit: NOT RUN
- 외부 배포, artifact 게시, Git commit/push
- SC-BUILD-02 Linux/Windows 원격 CI 실제 green evidence: pending
- advisory DB 최신 fetch: 설치된 DB를 `cargo audit --no-fetch`로 검사
- 공식 NetHack archive 재다운로드: 관련 checksum/locator 입력은 보고서 12 이후 변경되지 않아 이전 Verified 증거를 계승

제외 tree: `.git`, `target`, generated runtime output, 외부 reference corpus. 비밀 파일은 열지 않고 정적 credential 패턴만 검사했다.

## 4. 실행 명령과 결과

### 4.1 R7 표적·단계·root integrity

| 명령/검사 | 결과 |
| --- | --- |
| `bash -n scripts/r7_checkpoint.sh` | PASS |
| `cargo test -p aihack --locked --test provenance_manifest --test nethack_367_compat --test golden_phase8_rules` | PASS, 42 tests |
| `scripts/r7_checkpoint.sh` | 예상된 HOLD, exit 1 |
| 현재 checkpoint 사유 | PROV-0004 Reviewed, scenario 10개 Reviewed |
| checkpoint의 root `UNLICENSED` 사유 | 없음, R8 책임으로 분리됨 |
| complete approval + `UNLICENSED` root fixture | PASS |
| complete approval + invalid inherited root override | PASS, script-relative tree 판정 유지 |
| status-only/7 fields/checksum/schema/function/coverage/Blocked include | 전부 fail-closed PASS |
| `AIHACK_R7_ROOT=/definitely/not/aihack scripts/r7_checkpoint.sh` | 현재 tree와 동일 HOLD 사유·exit 1 |

### 4.2 전체 회귀·품질·공급망

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS, 8 workspace members |
| `cargo check --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 322 tests, 실패 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo build --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --locked` | PASS |
| `cargo tree -p aihack-core --locked` | PASS, UI/terminal/network dependency 없음 |
| `cargo tree -i crossterm --locked` | PASS, crossterm 0.29 단일 계열 |
| `cargo audit --no-fetch` | PASS, 1160 advisories / 267 dependencies |
| `cargo deny check licenses bans sources` | PASS |
| `git diff --check` | PASS |
| release headless seed 42, target 10, survival-v1 | PASS, accepted 10, hash `e7d30d72027a39c0` |
| secret/credential 정적 패턴 검색 | hit 0 |

## 5. Re-audit Lineage

| Finding | 보고서 13 상태 | 보고서 14 상태 | 근거 |
| --- | --- | --- | --- |
| IMP-F012 | Major / Hold | **Hold — unchanged** | actual owner/qualified approval 없음 |
| IMP-F013 | Major / Needs Fix | **Verified** | R7 provenance와 R8 distribution 책임 분리, UNLICENSED positive fixture PASS |
| SEC-F003 | Minor / Needs Fix | **Verified** | script-relative root, inherited override 무효 fixture/실행 PASS |
| XPF-F009 | Needs Fix | **Verified** | 단계 순환 제거, R8 전 외부 배포 차단 유지 |
| XPF-F008 | Partially Resolved / Hold | **Hold, technical parts resolved** | 기술 결함은 모두 Verified, actual approval만 남음 |

`Verified`는 지적된 수정이 실제 tree에 반영되고 재감사 증거로 검증됐다는 뜻이다. `IMP-F013`과 `SEC-F003`에 대한 추가 코더 수정은 필요 없다. R7 전체 `PASS`는 별도의 실제 승인 후에만 가능하다.

## 6. Pass 1: Implementation Compliance Findings

### [IMP-F012] 실제 license/provenance approval 미완료

- Pass: Implementation Compliance
- Pattern: IMP-003, IMP-004
- Area: SC-LICENSE-01, R7 checkpoint authority
- Severity: **Major**
- Status: **Hold — Re-audit #2, unchanged**
- Summary: 기술 gate는 준비됐으나 권한 있는 검토자의 실제 content/scenario approval가 없다.
- Evidence:
  - SC-LICENSE-01은 runtime 포함 자산 전부의 machine-validated Approved와 reviewer/date/license/scope/notice/evidence/checksum을 요구한다 (`spec.md:63`).
  - R7 완료 gate는 SC-COMPAT-01과 SC-LICENSE-01이다 (`spec.md:705`).
  - PROV-0004는 `Reviewed`이며 distribution scope와 notice가 pending이다 (`PROVENANCE.md:48`, `59-63`).
  - NH367-C001..C010도 `provenance_status: Reviewed`, 빈 approval evidence를 유지한다.
  - checkpoint는 PROV-0004와 scenario 10개만 사유로 HOLD/exit 1을 반환했다.
- Expected: project owner 또는 적격 검토자의 실제 결정이 필수 field와 evidence에 기록되고, 승인 불가 자산은 Blocked/교체된다.
- Actual: engineering trace와 validator는 완료됐지만 권한 있는 승인 결정은 없다.
- Impact: SC-LICENSE-01, R7 Closed와 R8 선행조건을 통과할 수 없다.
- Suggested Fix: human approval을 얻은 뒤 코더는 그 evidence만 구조화해 반영한다. 근거 없이 status 문자열을 변경하지 않는다.
- Re-audit Method: 승인 주체·근거, PROV-0004, scenario 10개, checksum/coverage, checkpoint와 전체 회귀를 독립 검증한다.
- Owner: Human project owner / qualified reviewer; coder for approved evidence integration

### [IMP-F013] R7/R8 license gate 순환 의존

- Pass: Implementation Compliance
- Severity: **Major**
- Status: **Verified — Re-audit #1**
- Evidence:
  - master contract는 R7을 asset/scenario provenance, R8을 root distribution license/version/packaging으로 분리한다 (`spec.md:705-710`).
  - R8 task에 승인된 root distribution license와 notice 반영이 명시됐다 (`IMPLEMENTATION_SUMMARY.md:859-877`).
  - checkpoint에서 root `Cargo.toml` license HOLD 조건이 제거됐다 (`scripts/r7_checkpoint.sh:31-155`).
  - `UNLICENSED` root의 complete R7 approval fixture가 PASS한다 (`tests/provenance_manifest.rs:287-299`).
  - R8 전 `UNLICENSED`, `publish = false`, distribution BLOCKED가 별도 test로 유지된다 (`tests/provenance_manifest.rs:259-269`).
- Closure: R7은 문서상 허용된 승인 변경만으로 PASS 가능하며, 외부 배포 권한은 R8 전까지 생기지 않는다.

### 6.3 Verified implementation evidence

- `spec.md`, implementation summary, provenance, ADR, roadmap, build guide와 R7 report가 동일한 R7/R8 책임을 선언
- R7 current state는 active 문서 전반에서 `Implemented / Approval HOLD`
- root license와 외부 배포는 R8 owner로 명시되고 R7 PASS만으로 허용되지 않음
- R1~R6 이전 PASS와 전체 322 test 회귀 유지

## 7. Pass 2: Debug / Engineering Quality Findings

새로운 correctness, test quality, architecture 또는 performance finding 없음.

### 7.1 Verified engineering evidence

- report 13의 단계/root 수정에 각각 독립 regression test가 존재
- positive fixture와 negative fixture가 구현 세부가 아니라 checkpoint 결과를 검증
- fmt, metadata, check, clippy `-D warnings`, debug/release build 모두 PASS
- full workspace 322 tests, 실패·ignored 0
- deterministic headless smoke hash 유지
- 새 dependency 없음, core boundary와 crossterm 단일 계열 유지

## 8. Pass 3: Security Findings

### [SEC-F003] caller-controlled checkpoint root

- Pass: Security
- Pattern: SEC-005, SEC-006
- Severity: **Minor**
- Status: **Verified — Re-audit #1**
- Evidence:
  - root는 script 위치의 상위 디렉터리로만 계산된다 (`scripts/r7_checkpoint.sh:4`).
  - script는 `AIHACK_R7_ROOT`를 읽지 않는다.
  - copied fixture에 잘못된 override를 주입해도 complete approval fixture가 PASS한다 (`tests/provenance_manifest.rs:301-313`).
  - 실제 tree에서 동일 override를 주입해도 현재 tree의 PROV-0004/scenario HOLD 사유와 exit 1이 그대로 재현됐다.
  - active build/ADR 문서도 script-relative repository를 hard boundary로 기록한다 (`BUILD_GUIDE.md:400-402`, `DESIGN_DECISIONS.md:234-237`).
- Closure: inherited environment가 release checkpoint의 검사 대상을 바꾸는 경로는 제거됐다.

### 8.2 Verified security evidence

- report 12의 status-only approval 우회 차단 계속 PASS
- R7 provenance와 R8 distribution boundary 분리 후에도 외부 배포 fail-closed 유지
- Blocked legacy direct import/path reference 0건
- cargo-audit, cargo-deny 통과, 정적 secret pattern hit 0

## 9. Cross-Pass Conflicts

### [XPF-F008] green engineering evidence와 actual approval hard boundary

- Status: **Hold — technical findings resolved**
- Related Finding: IMP-F012
- Resolution: 모든 코더 기술 finding은 Verified됐지만 test와 자동 gate는 실제 human approval을 대체하지 않는다. 따라서 R7은 approval 전까지 HOLD다.

### [XPF-F009] fail-closed distribution 요구와 phase 순서 충돌

- Status: **Verified — Re-audit #1**
- Related Finding: IMP-F013
- Evidence: R7 asset approval와 R8 root distribution 조건이 분리됐고, 각각 positive fixture와 R8 fail-closed test로 고정됐다.
- Closure: 단계 순환 없이 R7을 닫을 수 있으며 R8 전 외부 배포는 계속 차단된다.

## 10. Required Conditions Before R7 PASS

1. project owner 또는 적격 검토자가 PROV-0004 content와 NH367 scenario 10개의 license/scope/notice/modification/evidence를 실제로 결정한다.
2. 승인 시 필수 field와 evidence를 기록하고 `Approved`로 전환한다. 승인 불가 시 `Blocked` 처리하고 독립 자산으로 교체한다.
3. checkpoint PASS, 표적 42+, 전체 322+, clippy/release/supply-chain을 다시 실행한다.
4. 독립 재감사에서 승인 authority와 evidence를 확인한다.

현재 코더가 추가로 수정할 기술 finding은 없다. 다음 단계는 human/qualified approval이며, 코더는 승인 결과의 통합만 담당한다.

## 11. Accepted Risks

없음.

actual approval pending은 Accepted Risk가 아니라 R7의 명시적 Hold다. 원격 CI와 R8은 후속 범위이며 현재 R7 finding을 면제하지 않는다.

## 12. Needs Spec Clarification

없음.

보고서 13의 `NSC-F002`는 R7 asset/scenario provenance와 R8 root distribution license로 책임을 분리해 해소됐다.

## 13. Re-audit Checklist

```bash
bash -n scripts/r7_checkpoint.sh
cargo test -p aihack --locked --test provenance_manifest
cargo test -p aihack --locked --test nethack_367_compat
cargo test -p aihack --locked --test golden_phase8_rules
scripts/r7_checkpoint.sh
! rg -n "legacy_nethack_port_reference" Cargo.toml crates apps src \
  --glob '*.toml' --glob '*.rs'
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
cargo audit --no-fetch
cargo deny check licenses bans sources
git diff --check
```

승인 재감사에서 추가 확인:

- 승인 authority와 evidence의 실재
- PROV-0004와 scenario 10개의 field 일관성
- content checksum과 runtime coverage 유지
- R7 PASS 뒤에도 root `UNLICENSED`와 external distribution BLOCKED 유지
- R8에서만 root license/version/packaging 변경

## 14. Remaining Risks

- project owner/적격 검토자의 content/scenario license approval pending
- SC-BUILD-02 Linux/Windows 원격 CI evidence pending
- R8 root license/version/packaging, SC-DOC-01과 final release audit NOT RUN
- 설치된 advisory DB만 사용했으므로 최신 원격 RustSec 상태는 release CI에서 재확인 필요
- full NetHack parity, 법률 자문, 외부 배포 가능성은 이번 범위 밖
- 최종 release 전 인간 또는 복수 모델 교차감사 필요

## 15. Final Decision

**HOLD — report 13 technical findings verified; actual provenance approval remains**

| Gate | 판정 |
| --- | --- |
| R1~R6 | 기존 Verified/PASS 유지 |
| `DBG-F005`, `SEC-F002` | 기존 Verified 유지 |
| `IMP-F013` | **Verified** |
| `SEC-F003` | **Verified** |
| `XPF-F009`, `NSC-F002` | **Resolved / Verified** |
| `IMP-F012` actual approval | **HOLD** |
| R7 target | PASS, 42 tests |
| Full workspace | PASS, 322 tests |
| SC-COMPAT-01 engineering evidence | PASS |
| SC-LICENSE-01 | **HOLD** |
| Checkpoint R7 | **HOLD** |
| R8/remote CI | pending / NOT RUN |
| 전체 프로그램/release | 아직 PASS 대상 아님 |

보고서 13에서 요구한 코더 시정은 완료돼 추가 재수정이 필요 없다. R7을 PASS로 전환하려면 project owner 또는 적격 검토자의 실제 승인과 그 evidence 통합이 필요하다.

코드·설정·기존 문서는 수정하지 않았고 감사 보고서만 생성했다.
