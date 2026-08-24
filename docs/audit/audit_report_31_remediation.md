# AIHack 감사 보고서 31 시정 기록

- 기준 독립 감사: Report 31
- 기준 HEAD: `b8c20c23d106797ad36b41d817635b27553afe6a`
- 작업 브랜치: `codex/audit-report-31-remediation`
- 작업 일자: 2026-08-24
- predecessor 기술 판정: **Report 30/29 표적 회귀 PASS**
- 현재 판정: **successor `8c042d48/32741917348` clean same-SHA 양 OS actual Verified / independent re-audit pending / PROGRAM·PUBLICATION HOLD**

독립 감사 원문은 수정하지 않는다. Report 30 public visibility와 technical successor `ed02dbf/32733235414`, Report 29의 content/allocator/TUI/archive 기술 증거는 역사적 Verified로 보존한다.

현재 권위는 `docs/audit/audit_report_31.md`이며 이번 시정은 implementation summary lifecycle과 document regression source에 한정한다.

## 1. Finding 대조와 결정

| Finding | current implementation summary | document regression source | 결정 |
| --- | --- | --- | --- |
| R29-DOC-F002 Re-audit #2 / FIN-F012 | 1절은 Report 30 기술 완료를 기록하지만 10절은 ADR-0040 구현·local gate·CI를 next로 재개방하고 11절은 Report 29를 current로 선언 | `tests/r8_documentation.rs`는 summary 1절만 exact-one 검사하고 10·11절에서는 report 27/28 exact phrase만 거부 | Report 31 exact-one을 1·10·11절 각각에 적용하고 predecessor current 및 완료 작업 재개방을 report 번호 기반 공통 negative gate로 거부 |

Report 31의 나머지 판정은 재대조 결과 그대로 유지한다. R30-IMP-F001과 FIN-F001~F011/F013~F018은 production/API 또는 기술 회귀 finding이 아니라 Verified다. 새 Debug/Security finding과 spec clarification은 없다.

## 2. 문서 선행 계약

- `spec.md`: active section의 current exact-one, predecessor current 및 completed-work 재개방 금지 계약
- `DESIGN_DECISIONS.md`: ADR-0041과 ADR-0040 historical 전환
- `IMPLEMENTATION_SUMMARY.md`: 구현 후 1·10·11절이 가리켜야 할 Report 31 current/next lifecycle
- `CHANGELOG.md`, gap/audit roadmap과 active 문서 header: Report 31 시정 계보

## 3. 수정 전 RED

문서 계약 갱신 후 production summary와 regression test를 수정하기 전에 다음 fixture를 실행해 기록한다.

```text
report 29 current-authority mutation                         -> RED 예상
completed report 30 ADR-0040 implementation next mutation   -> RED 예상
completed report 30 local gate/same-SHA CI pending mutation -> RED 예상
current IMPLEMENTATION_SUMMARY 1/10/11 lifecycle            -> RED 예상
```

실제 RED:

```text
lifecycle_gate_rejects_predecessor_current_and_completed_work_reopen ... FAILED
generic lifecycle mutation was accepted: report 29가 현재 authority이며 ADR-0039 시정 전이다.

active_document_sections_have_one_report_31_authority_and_reject_predecessors ... FAILED
README ### 현재 상태: current authority marker ... audit_report_31.md, count=0
```

첫 실패는 기존 helper가 predecessor current 문장을 허용했음을, 두 번째 실패는 Report 31 contract로 검사 범위를 올렸을 때 실제 active 문서가 Report 30에 머물렀음을 각각 보존한다.

## 4. 구현과 표적 GREEN

- `report_numbers`가 `audit_report_N.md`와 `report N` 표기에서 번호를 추출한다.
- 선언형 current-authority line이 현재 report가 아닌 번호를 가리키면 실패한다. 한국어/영어/일본어/중국어 번체·간체 선언 변형을 mutation으로 검증한다.
- 현재 번호보다 작은 report가 다음 단계·진행 중·pending과 구현/remediation/verification/local gate/same-SHA/CI/bundle을 결합하면 완료 작업 재개방으로 실패한다.
- `no longer`, `아니다`, `않는다`, historical state 같은 명시적 부정·역사 표현은 false positive로 보지 않는다.
- README 5 locale, spec/ADR, summary 1·10·11, documentation audit, roadmap, build/gap, designs, compatibility와 report 29/30/31 remediation active section을 Report 31 exact-one으로 정렬했다.
- 전체 workspace 첫 실행에서 이 시정 문서 header가 predecessor PASS와 current local/CI pending을 한 줄에 결합해 generic gate에 걸렸다. 기술 이력과 현재 상태를 별도 줄로 분리한 뒤 문서 gate를 복구했다.

```text
lifecycle_gate_rejects_predecessor_current_and_completed_work_reopen ... ok
active_document_sections_have_one_report_31_authority_and_reject_predecessors ... ok
r8_documentation ... 11 passed
```

## 5. Report 30 public visibility와 Report 29 기술 회귀

| 회귀 묶음 | 결과 |
| --- | --- |
| Report 30 external public visibility | `public_mutation_boundary` 2 PASS; default read query compile-pass, World/system/testing compile-fail |
| Report 29 content/custom registry/item ID-kind/glyph | `content_validation` 13 PASS |
| allocator/Throw/Zap atomicity | `transaction` 6 PASS |
| archive raw type/extraction/ExpectedCommit | `release_archive_security` 2 PASS |
| Windows staging root/hardlink/calendar/alias | `release_bundle_windows` 7 PASS |
| NH367 compatibility | `nethack_367_compat` 10 PASS |
| TUI lib/main/ConPTY/gesture/F9/modal mouse | `aihack-tui --all-targets` 27 PASS |

Report 30 public visibility와 Report 29 기술 회귀는 변경 없이 유지됐다.

## 6. 전체 로컬 quality gate와 clean Windows actual bundle

clean-worktree actual bundle을 제외한 전체 gate의 최종 실행 결과다. 첫 workspace 실행에서 새 gate가 이 문서의 혼합 lifecycle line을 차단했고, section 4의 구조 수정 뒤 전체 suite를 처음부터 재실행해 PASS했다.

| 명령 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked -- --list` | Windows named test **453개** |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| `cargo metadata --locked --format-version 1` | packages/nodes **318/318**, registry 310, path 8, git 0 |
| `cargo audit` | PASS, 318 dependencies, vulnerabilities 0 |
| `cargo deny --version` | `cargo-deny 0.19.4` |
| `cargo deny check licenses bans sources` | PASS |
| Git Bash `scripts/r7_checkpoint.sh` | PASS |
| Git Bash `scripts/r8_checkpoint.sh` | PASS |

candidate commit `f3a7aa662d8820b361c674b37264f3246cc2b7ac`의 clean worktree에서 `cmd /d /c build.bat --release`를 실행했다.

```text
PASS source archive: format=zip entries=379
PASS Windows release bundle: version=0.3.0 commit=f3a7aa662d8820b361c674b37264f3246cc2b7ac
output exact entries=9
```

output은 binary 2, license/notice/modification/approval/metadata 문서 5, source ZIP, `SHA256SUMS`의 정확히 9개 entry다. docs evidence successor `8c042d48...`의 clean Windows 재검증과 same-SHA 양 OS CI도 아래 7절에서 Verified됐다.

## 7. clean same-SHA Ubuntu/Windows actual bundle

최종 기술 evidence successor는 `8c042d48df57621e23a9c2a3406cc6fa68bea0af`이며 [Actions `32741917348`](https://github.com/Yupkidangju/AIHack/actions/runs/32741917348)가 completed/success다.

| Job | ID | 결과 | 시간 | step |
| --- | ---: | --- | ---: | --- |
| `ubuntu-latest quality gate` | `97478142640` | PASS | 9m58s | 19 success, Windows bundle 1 skip |
| `windows-latest quality gate` | `97478143152` | PASS | 23m07s | 19 success, Linux bundle 1 skip |

양 job은 checkout SHA `8c042d48...`에서 metadata/fmt/clippy/all-target tests, external compile contracts, dependency exception/duplicate, R7/R8, release all-target build, actual platform bundle, cargo-audit, cargo-deny 0.19.4와 lockfile 불변을 모두 통과했다.

이 결과로 report 31 시정 lifecycle은 `Verified`다. 다만 `Closed`, 전체 PROGRAM PASS와 외부 게시 허가는 새 독립 재감사 및 별도 사용자 승인 전까지 HOLD다.

## 8. 잔여 경계

- report 31 후속 독립 재감사 전에는 finding을 `Closed` 또는 전체 PROGRAM PASS로 올리지 않는다.
- actual provider, Windows Terminal GUI, physical key-hold, signing/attestation과 외부 게시 승인은 이번 문서 시정 범위 밖이다.
- CI Info: pinned `actions/checkout`이 Node.js 20 deprecation annotation을 냈지만 runner가 Node.js 24로 강제 실행했고 양 job은 success였다. workflow pin 갱신은 이번 finding과 분리된 유지보수 항목이다.
