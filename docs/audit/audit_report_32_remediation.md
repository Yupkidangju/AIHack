# AIHack 감사 보고서 32 시정 기록

- 기준 독립 감사: Report 32
- 기준 HEAD: `80452498861e7acce2416821255b329372a8004f`
- 작업 브랜치: `codex/audit-report-32-remediation`
- 작업 일자: 2026-08-25
- Report 31 판정: **R29-DOC-F002 Re-audit #3 / FIN-F012 independently Closed**
- 현재 판정: **R32-DBG-F001·FIN-F015 full local·candidate clean Windows actual PASS / exact-final-headSha Actions external verification / PROGRAM·PUBLICATION HOLD**

독립 감사 원문은 수정하지 않는다. Report 31 successor `8c042d48/32741917348`의 lifecycle/public visibility/Report 29 기술 증거는 historical Verified로 보존한다.

현재 권위는 `docs/audit/audit_report_32.md`다. 이번 시정은 exact final commit date, modification Notice ID/period, release metadata와 양 verifier의 포함 관계를 닫는다.

## 1. Finding 대조와 결정

| Finding | current HEAD/document | production release path | 결정 |
| --- | --- | --- | --- |
| R29-DOC-F002 Re-audit #3 / FIN-F012 | Report 31 summary 1·10·11과 generic gate independent closure | `r8_documentation` 11 PASS, `8c042d48/32741917348` | 추가 구현 없이 historical Closed로 보존 |
| R32-DBG-F001 / FIN-F015 | HEAD `8045249` date `2026-08-25`, manifest end/ID `2026-08-24` | `build.bat`/`build.sh` → metadata → PowerShell/Bash verifier | Notice ID/period를 2026-08-25 revision으로 원자 갱신하고 actual HEAD-date 조기 회귀 추가 |

## 2. 문서 선행 계약

- `spec.md`: exact final `%cs`, Notice ID/period, actual HEAD-date early gate와 final-SHA CI evidence 방식
- `DESIGN_DECISIONS.md`: ADR-0042와 ADR-0041 independent closure
- active docs: Report 32 current authority, FIN-F015 Needs Fix, PROGRAM/PUBLICATION HOLD
- `MODIFICATIONS.md`와 구현 상수는 RED 확인 뒤 갱신한다.

## 3. 수정 전 RED

감사 재현 기준:

```text
git show -s --format=%cs HEAD = 2026-08-25
MODIFICATIONS.md period end    = 2026-08-24
build.bat --release            = FAIL: candidate date falls outside the modification period
license_compliance literal test = false-green PASS
```

actual HEAD 날짜를 읽는 새 regression의 실패 출력은 test-first 단계에서 추가한다.

실제 RED:

```text
release_metadata_and_manifest_cover_the_candidate_commit_date ... FAILED
current HEAD candidate date가 modification period 밖입니다:
2025-05-20 <= 2026-08-25 <= 2026-08-24
```

기존 literal self-check가 놓친 동일 불일치를 actual HEAD 기반 unit boundary에서 actual bundle 전에 재현했다.

## 4. 구현과 표적 GREEN

- `MODIFICATIONS.md` Notice ID/period와 모든 scope row를 `AIHACK-MODIFICATIONS-2026-08-25-01` / `2025-05-20..2026-08-25`로 갱신했다.
- `RELEASE-METADATA`, `build.sh`, `build.bat`, Bash/PowerShell verifier, R8 checkpoint와 Linux/Windows fixtures에 같은 exact Notice ID/period를 원자 전파했다.
- `license_compliance::release_metadata_and_manifest_cover_the_candidate_commit_date`가 실제 `git show -s --format=%cs HEAD`를 읽고 단일 manifest period의 `start <= candidate <= end`를 검사한다.
- 양 verifier fixture는 candidate가 period 종료일 전과 종료일이면 허용하고 다음 날이면 거부한다. Windows 표적 matrix는 local PASS했고 Unix executable fixture는 final Ubuntu CI에서 실행한다.
- Report 32 current authority, Report 31/FIN-F012 independent closure, R32-DBG-F001/FIN-F015 HOLD를 active docs와 generic document gate에 동기화했다.

```text
license_compliance                           6 PASS
r8_documentation                            11 PASS
release_bundle_windows                      7 PASS
release_gate                                7 PASS
```

## 5. 전체 로컬 quality gate

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
| Git Bash `bash -n` build/verifier/checkpoint | PASS |
| Git Bash `scripts/r7_checkpoint.sh` | PASS |
| Git Bash `scripts/r8_checkpoint.sh` | PASS |

Windows host에는 WSL distro와 Docker가 없어 local Linux binary를 권위 있게 생성할 수 없다. `build.sh`를 Windows Git Bash에서 실행한 결과는 Linux actual로 사용하지 않으며, final SHA의 `ubuntu-latest` `build.sh --release`를 clean Linux actual bundle evidence로 사용한다.

candidate `57d8108a51db08f942aba3218eafd2a94cc011d3`의 commit date는 `2026-08-25`이며 clean worktree `cmd /d /c build.bat --release` 결과는 다음과 같다.

```text
PASS source archive: format=zip entries=381
PASS Windows release bundle: version=0.3.0 commit=57d8108a51db08f942aba3218eafd2a94cc011d3
output exact entries=9
```

final document-bearing commit에서도 clean Windows bundle을 다시 실행하고 동일 SHA의 Ubuntu/Windows CI를 사용한다. 이 repository 문서는 미래 run ID를 예측하거나 결과 기록용 successor를 만들지 않으며, current final `headSha`와 일치하는 completed-success Actions record가 canonical result다.

## 6. clean Windows/Linux actual bundle과 final-SHA CI

CI run ID는 final commit 이후에만 생성되므로 저장소 문서는 특정 미래 ID를 예측하지 않는다. 이 문서를 포함한 final commit의 `headSha`와 일치하는 Ubuntu/Windows completed-success run이 canonical external evidence다. 결과는 이 thread의 최종 보고와 GitHub Actions run record에 남기며, 결과 기록용 후속 commit으로 final SHA를 바꾸지 않는다.

## 7. 잔여 경계

- 새 독립 재감사 전에는 R32-DBG-F001/FIN-F015를 `Closed` 또는 전체 PROGRAM PASS로 올리지 않는다.
- actual provider, physical key-hold, Windows Terminal GUI, signing/attestation과 외부 게시 승인은 이번 시정 범위 밖이다.
