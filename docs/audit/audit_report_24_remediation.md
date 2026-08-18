# 감사 리포트 24 코더 시정 기록

- 작성일: 2026-08-18 (Asia/Seoul)
- 기준 finding: `docs/audit/audit_report_24.md`
- 기준 HEAD: `41a1b63f11a57a671b0f705883431dab24298b5a` + report 23 remediation working tree
- 문서 성격: 코더 구현·검증 기록. 독립 재감사 또는 외부 게시 승인 아님

## 1. 시정 결과

| Finding | 코더 상태 | 시정 내용 | 로컬 증거 |
| --- | --- | --- | --- |
| DBG-F011 | Remediated / CI pending | `winx 0.36.4`에만 `Apache-2.0 WITH LLVM-exception` 허용, owner/2026-10-31 만료/version 변경 trigger 기록 | cargo-deny 0.19.4 `bans ok, licenses ok, sources ok` |
| SEC-F003 | Remediated by contract narrowing / CI pending | Unix mode 0600과 Windows parent DACL 상속을 문서·함수 이름·platform test에 정렬 | Windows writable/replace test PASS, Unix mode test는 Unix CI 대상 |
| IMP-F019 | Remediated / CI pending | SC-CAUSE-01..07 각각을 production 심볼과 정확한 테스트 함수에 중앙 매핑 | `sc_cause_contract_ids_map_to_code_and_tests` PASS |

## 2. Dependency exception 통제

- Crate/version: `winx 0.36.4`
- SPDX: `Apache-2.0 WITH LLVM-exception`
- 사용 경로: `winx -> cap-primitives -> cap-std/cap-fs-ext -> aihack-runtime`
- Owner: Dependency owner / Release manager
- 만료일: 2026-10-31
- 조기 재검토 trigger: `winx`, `cap-primitives`, `cap-std`, `cap-fs-ext`, `cap-tempfile` version 변경
- 정책: 일반 allowlist에 추가하지 않고 version-scoped exception으로만 허용

## 3. Windows save 권한 경계

`ArtifactStore`의 path sandbox, no-follow, single-hard-link, atomic replace 경계는 유지한다. Unix save/temp는 mode 0600을 강제한다. Windows는 runtime이 DACL을 직접 재작성하지 않고 parent directory DACL을 상속하므로, 다른 principal의 읽기를 막아야 하는 배포는 사용자 전용 ACL의 runtime root를 제공해야 한다. 일반 workspace root를 owner-only hard boundary로 주장하지 않는다.

## 4. SC-CAUSE 개별 매핑

`audit_roadmap.md`와 `IMPLEMENTATION_SUMMARY.md`의 중앙 표가 SC-CAUSE-01부터 SC-CAUSE-07까지 각각 다음을 연결한다.

1. `spec.md` 성공 기준
2. production 코드 책임 심볼
3. 실제 test 파일과 함수
4. `tests/r8_documentation.rs`의 mapping regression

range 표기만 존재하거나 test 함수가 제거되면 문서 회귀가 실패한다.

## 5. 검증 상태

| 검증 | 결과 |
| --- | --- |
| cargo-deny 0.19.4 `check licenses bans sources` | PASS |
| `build_contract` winx exception regression | PASS |
| `headless_paths` platform permission regression | PASS on Windows |
| `r8_documentation` SC-CAUSE/permission regression | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --locked` | PASS |
| `cargo audit` | PASS, vulnerability/warning 0 |
| Windows Git Bash R7/R8 checkpoint | PASS / PASS |
| `git diff --check` | PASS |
| clean same-SHA Ubuntu/Windows CI | commit/push 대기 |

첫 원격 실행 `32106910778`의 Ubuntu test는 Linux `O_TMPFILE`의 정상 link count 0을 기존-file validator가 거부해 실패했다. destination/read file은 nlink 1을 계속 요구하고, 원자적으로 신규 생성된 temp validator만 nlink 0 또는 1을 허용하도록 분리했다. `unix_save_file_uses_mode_0600`과 기존 save/load 회귀가 이 platform 경로를 검증한다.

두 번째 원격 실행 `32107476736`에서 save와 Unix 0600 회귀는 PASS했다. 이후 Unix 전용 release-bundle negative fixture가 환경별 Git ignore 상태 때문에 legacy probe를 archive에 넣지 못해 실패했다. `IncludedLegacy` case는 probe를 `git add -f`로 추적하고 archive listing에 차단 경로가 실제 포함됐는지 verifier 전에 확인하도록 수정했다.

## 6. 남은 gate

- final diff review와 secret check
- clean commit/push
- 동일 commit SHA의 Ubuntu/Windows quality gate, R7/R8 checkpoint, release bundle, cargo-audit/cargo-deny 확인
- 외부 게시에는 별도 사용자 승인 필요
