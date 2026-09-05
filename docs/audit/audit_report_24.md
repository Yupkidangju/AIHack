# D3D 재감사 리포트 24: report 23 시정 검증

- 감사일: 2026-08-18T13:47:19+09:00
- 감사 유형: `docs/audit/audit_report_23.md` 전체 finding 시정 독립 재감사
- 감사 기준: `AI_AUDIT_DOC_STANDARD.md`, `audit_roadmap.md`, `spec.md`
- 기준 HEAD: `41a1b63f11a57a671b0f705883431dab24298b5a` + 현재 미커밋 remediation working tree
- 환경: Windows, rustc/cargo 1.94.1
- 변경 정책: 감사 중 기존 소스·테스트·설정·구현 문서는 수정하지 않았고 본 보고서만 추가했다.

## 1. 감사 요약

최종 판정은 **HOLD**다.

report 23의 기존 finding은 로컬 범위에서 모두 시정됐다. 문서 권한 계보, 9종 causal witness, Windows CRLF checkpoint, save/replay hard-link 경계, CLI help, diff hygiene와 `lru` advisory 시정은 코드·테스트·실행 결과가 같은 결론을 지지한다. 별도 Cargo target에서 fmt, Clippy, 전체 359개 테스트와 release build가 PASS했고, Windows Git Bash의 실제 R7/R8 checkpoint도 exit 0이다.

그러나 새 capability 의존성이 추가한 `winx 0.36.4`의 라이선스는 `Apache-2.0 WITH LLVM-exception`이고 현재 `deny.toml`은 이 SPDX exception을 허용하지 않는다. cargo-deny는 exception이 없는 `Apache-2.0`과 이를 서로 다른 라이선스로 취급하므로 필수 release dependency gate가 닫히지 않았다. 또한 remediation working tree는 아직 commit/push되지 않아 새 same-SHA Linux/Windows CI evidence가 없다.

따라서 report 23 remediation 자체는 대부분 Verified지만, R1/R8 전체와 외부 게시 상태는 HOLD다.

## 2. 감사 범위

### 2.1 확인한 문서

- `docs/audit/audit_report_23.md`
- `docs/audit/audit_report_23_remediation.md`
- `docs/audit/audit_report_22.md`
- `spec.md`, `designs.md`, `DESIGN_DECISIONS.md`
- `README.md`, `BUILD_GUIDE.md`, `CHANGELOG.md`
- `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`
- `DOCUMENTATION_AUDIT_REPORT.md`, `LESSONS_LEARNED.md`
- `AI_AUDIT_DOC_STANDARD.md`, `AI_IMPLEMENTATION_DOC_STANDARD.md`, `AI_CODING_STANDARD.md`

### 2.2 확인한 구현·테스트·설정

- `crates/aihack-runtime/src/save.rs`, `crates/aihack-runtime/src/causal.rs`
- `apps/aihack-headless/src/main.rs`
- `apps/aihack-tui/src/main.rs`, `apps/aihack-tui/src/tui/mod.rs`
- `tests/long_run.rs`, `tests/headless_paths.rs`
- `tests/provenance_manifest.rs`, `tests/r8_documentation.rs`
- headless/TUI CLI contract test
- `.gitattributes`, `.github/workflows/ci.yml`, `scripts/r7_checkpoint.sh`
- `Cargo.lock`, runtime/TUI manifests, `deny.toml`
- 전체 workspace source/test target

## 3. 제외 범위

- Git commit, push, tag, release 및 외부 게시
- remediation diff를 포함하는 새 원격 CI 실행: 아직 commit SHA가 없음
- 실제 원격 LLM provider 호출
- 대화형 PTY/TUI 수동 시각 검수
- qualified legal opinion
- 로컬 `cargo-deny` 실행: subcommand가 설치되어 있지 않음. 대신 Cargo metadata와 공식 cargo-deny SPDX matching 규칙으로 현재 구성의 불충족을 판정했다.
- `legacy_nethack_port_reference/`, `.git/`, 기존 build/output artifact

## 4. 검증 증거

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS |
| report 23 표적 테스트 | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 359 tests |
| `cargo build --workspace --release --locked` | PASS |
| release `headless_policy` + `long_run` | PASS, 10 tests |
| seed 42/7/1234 survival hash | `c734eeafedc77c82`, `de24bb6e33a8c43f`, `c6f5e6ca9498ef35` |
| seed 42/7/1234 causal hash | `5cde4a5f145ff3af`, `942403c665e19ad9`, `01a8631d0ad95d96` |
| Windows 실제 `scripts/r7_checkpoint.sh` | PASS, exit 0 |
| Windows 실제 `scripts/r8_checkpoint.sh` | PASS, exit 0 |
| `cargo audit` | PASS, vulnerability/warning 0 |
| `cargo tree -i lru` | `lru 0.18.2` |
| `cargo tree -i winx` | `winx -> cap-primitives -> aihack-runtime` shipped path 확인 |
| `cargo deny check licenses bans sources` | 미실행, 로컬 subcommand 없음; config 불충족 정적 확인 |
| 원격 CI | base commit run `32034295607`만 존재; remediation diff 미포함 |

Windows에서는 `tests/release_bundle.rs`가 계속 Unix 전용이라 0개 테스트지만, 실제 R7/R8 checkpoint와 Windows `build.bat --release`를 CI에 별도 실행하도록 workflow가 보강됐다. 이 workflow 변경의 실제 원격 성공은 새 commit 뒤에만 검증할 수 있다.

## 5. report 23 Finding 재감사 결과

| Finding | 재감사 상태 | 핵심 증거 |
| --- | --- | --- |
| IMP-F016 | **Verified** | report 21 종결, R9 SHA/run, report 23 HOLD가 활성 문서와 section/row regression에 정렬됨 |
| IMP-F017 | **Verified** | report 22가 Initial/Post-fix/Current 상태를 분리하고 risk owner·trigger를 기록함 |
| IMP-F018 | **Verified** | 두 CLI help에서 v0.1.0/Phase 제거, contract test PASS |
| TEST-F001 | **Verified** | 9종 witness, seed별 3회 multiset/hash, event/turn-only 및 누락 negative gate PASS |
| DBG-F009 | **Verified locally** | LF checkout, CRLF positive, checksum drift negative, 실제 Windows R7/R8 exit 0 |
| DBG-F010 | **Verified** | `git diff --check` PASS, 한국어 주석과 line-ending-only 상태 정리 |
| SEC-F001 | **Verified for reported exploit** | capability-relative I/O, random create-new temp, no-follow/single-link 검증, hard-link victim 불변 회귀 PASS |
| SEC-F002 | **Verified** | `lru 0.18.2`, 최신 RustSec audit warning 0 |

DBG-F009의 새 Windows CI step과 전체 remediation의 same-SHA evidence는 commit/push 뒤 최종 release precondition으로 남는다. 이는 로컬 시정 실패가 아니라 immutable release evidence 대기다.

## 6. Pass 1: Implementation Compliance Findings

### [IMP-F019] SC-CAUSE 계약 ID가 실행 계획과 테스트로 개별 폐쇄되지 않는다

- Pass: Implementation
- Pattern: IMP-003, DOC-BACKFILL-001
- Area: R9 contract traceability
- Severity: **Minor**
- Status: **Needs Documentation Recovery**
- Summary: `SC-CAUSE-01..07`의 의미와 구현은 존재하지만 대부분의 ID는 `spec.md`에만 개별 문자열로 나타난다.
- Evidence:
  - `spec.md:787-793`은 7개 ID를 각각 정의한다.
  - `IMPLEMENTATION_SUMMARY.md:958`은 `SC-CAUSE-01..07` 범위 표기만 사용해 02~07 개별 ID 검색이 닫히지 않는다.
  - `GAP_CLOSURE_ROADMAP.md:50`은 `SC-CAUSE-05..07` 범위 표기만 사용한다.
  - `audit_roadmap.md`, source와 test에는 개별 `SC-CAUSE-02`, `03`, `04`, `06`, `07` 추적 표지가 없다.
- Expected: 성공 기준 ID가 적어도 실행 계획/감사 gate와 대응 테스트 또는 책임 파일에서 개별 추적 가능해야 한다.
- Actual: 자연어와 range 표기만 있어 기존 R0 ID closure 방식으로는 대부분의 R9 계약을 검증할 수 없다.
- Impact: 구현은 동작하지만 후속 문서 회귀가 특정 R9 계약을 누락해도 자동으로 탐지하기 어렵다.
- Suggested Fix: `GAP_CLOSURE_ROADMAP.md` 또는 `audit_roadmap.md`에 SC-CAUSE-01..07 개별 행/매핑을 추가하고, 관련 테스트 이름 또는 주석 대신 중앙 contract map에서 책임 테스트를 연결한다.
- Re-audit Method: 각 ID가 spec과 실행/감사 문서 양쪽에 존재하고 실제 테스트 파일·함수로 연결되는지 검사한다.
- Owner: Documentation owner / Test owner

## 7. Pass 2: Debug / Engineering Quality Findings

### [DBG-F011] 새 `winx` 라이선스가 cargo-deny allowlist와 맞지 않는다

- Pass: Debug
- Pattern: DEP-001, BUILD-001
- Area: dependency policy, release gate
- Severity: **Major**
- Status: **Needs Fix / Hold**
- Summary: capability filesystem 도입으로 shipped dependency `winx 0.36.4`가 추가됐지만 해당 SPDX exception이 `deny.toml`에 없다.
- Evidence:
  - `crates/aihack-runtime/Cargo.toml:13-15`가 `cap-fs-ext`, `cap-std`, `cap-tempfile 4.0.2`를 추가한다.
  - `cargo tree -i winx`는 `winx -> cap-primitives -> cap-std/cap-fs-ext -> aihack-runtime` 경로를 확인한다.
  - Cargo metadata의 `winx 0.36.4` license는 정확히 `Apache-2.0 WITH LLVM-exception`이다.
  - `deny.toml:5`의 허용 목록은 `Apache-2.0`, `MIT`, `Unicode-3.0`, `Zlib`뿐이다.
  - [cargo-deny 공식 license 설정 문서](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html)는 exception이 붙은 license를 exception 없는 동일 license와 별개로 취급한다고 명시한다.
  - 로컬 `cargo-deny`가 없어 코더 기록과 본 감사 모두 필수 명령을 실제 실행하지 못했다.
- Expected: 모든 shipped dependency license가 `deny.toml` 정책을 만족하고 고정 버전 cargo-deny 명령이 PASS해야 한다.
- Actual: 현재 allowlist는 `winx`의 유일한 license expression을 만족하지 못한다.
- Impact: CI의 `cargo deny check licenses bans sources`가 실패할 가능성이 아니라, 현 구성 규칙상 license gate를 통과할 수 없다. R1/R8 PASS와 새 same-SHA CI 생성이 차단된다.
- Suggested Fix:
  1. 프로젝트 정책이 LLVM exception을 수용하면 `Apache-2.0 WITH LLVM-exception`을 일반 allowlist 또는 `winx 0.36.4`에 한정된 exception으로 기록한다.
  2. 한정 exception을 사용하면 BUILD_GUIDE가 요구하는 이유, owner, 만료/재검토 조건을 함께 남긴다.
  3. `cargo deny check licenses bans sources`를 실제 실행하고 lockfile 불변 및 전체 gate를 재검증한다.
- Re-audit Method: cargo-deny 0.19.4 명령 exit 0, `winx` license diagnostic 0건, 새 dependency source/duplicate 정책 확인.
- Owner: Dependency owner / Release manager

## 8. Pass 3: Security Findings

### [SEC-F003] Windows에서 “소유자 전용” save temp 권한을 강제하지 않는다

- Pass: Security
- Pattern: SEC-004, SEC-005
- Area: filesystem permissions, documentation boundary
- Severity: **Minor**
- Status: **Needs Fix or Documentation Narrowing**
- Summary: Unix는 mode 0600을 적용하지만 Windows는 read-only 속성만 해제하고 DACL/owner-only 접근을 설정하지 않는다.
- Evidence:
  - `spec.md:696`, `BUILD_GUIDE.md:249`, `DESIGN_DECISIONS.md:29`는 save temp를 소유자 전용으로 선언한다.
  - `crates/aihack-runtime/src/save.rs:266-274`는 모든 OS에서 `set_readonly(false)`만 수행하고, `set_mode(0o600)`은 `#[cfg(unix)]` 안에 있다.
  - Windows용 ACL 설정이나 owner-only permission regression은 없다.
  - [`cap-tempfile::TempFile` 공식 문서](https://docs.rs/cap-tempfile/latest/cap_tempfile/struct.TempFile.html)는 기본 권한이 `File::create_new`와 같고 private file이 필요한 경우 별도 처리가 필요하다고 설명한다.
- Expected: 지원 OS 모두에서 owner-only가 강제되거나, 문서가 Unix mode 0600과 Windows parent ACL inheritance를 정확히 구분해야 한다.
- Actual: Windows 권한은 runtime directory의 inherited ACL에 의존하면서 문서는 owner-only hard boundary로 표현한다.
- Impact: 공유되거나 느슨한 ACL의 workspace에서 임시·최종 save가 다른 OS principal에게 읽힐 수 있다. 현재 save에는 API credential이 없으므로 영향은 제한적이다.
- Suggested Fix: Windows DACL을 명시적으로 제한하고 회귀 테스트를 추가하거나, per-user application directory를 강제하거나, 문서의 owner-only 주장을 platform별 실제 보장으로 좁힌다.
- Re-audit Method: Windows ACL/Unix 0600 검사 또는 수정된 platform-specific security contract와 실제 파일 metadata 대조.
- Owner: Coder / Security reviewer

## 9. Cross-Pass Conflicts

### [XPF-F014] 전체 Rust gate green과 dependency release policy가 충돌한다

- Related Findings: DBG-F011
- Conflict: fmt, Clippy, 359 tests, release build와 RustSec audit는 PASS하지만 cargo-deny license 정책은 새 shipped dependency를 허용하지 않는다.
- Resolution: 구현·테스트 PASS는 유지하되 dependency policy가 실제 명령으로 PASS할 때까지 R1/R8과 전체 판정을 HOLD한다.
- Gate Impact: release candidate와 same-SHA CI 생성 차단.
- Required Fix Before PASS: cargo-deny license allow/exception 정렬 및 실제 명령 PASS.

## 10. Required Fixes Before PASS

1. `winx 0.36.4`의 `Apache-2.0 WITH LLVM-exception`을 프로젝트 license policy에 명시적으로 처리한다.
2. `cargo deny check licenses bans sources`를 실제 실행해 PASS evidence를 남긴다.
3. SEC-F003의 Windows 권한 보장을 구현하거나 문서 경계를 정확히 좁힌다.
4. SC-CAUSE-01..07의 개별 문서-테스트 매핑을 닫는다.
5. 최종 intended diff를 commit/push한 뒤 같은 SHA의 Ubuntu/Windows CI, canonical R7/R8와 양 OS bundle을 확인한다.
6. 외부 게시는 별도 사용자 승인 뒤에만 수행한다.

## 11. Accepted Risks

- `hallucinating` SaveDataV1 호환성 orphan은 owner가 Project owner/runtime maintainer로 지정됐고, SaveDataV2·v0.4.0 승인 또는 2026-10-31 중 먼저 도래하는 시점에 재검토한다. 본 재감사는 이를 time-bounded Accepted Compatibility Risk로 인정한다.
- 실제 remote LLM provider smoke는 spec상 비차단이다.
- qualified legal opinion은 본 기술 감사 범위 밖이다.

## 12. Needs Spec Clarification

없음. 새 finding의 기대 상태와 수정 경계는 현재 문서와 dependency metadata로 충분히 결정할 수 있다.

## 13. 재감사 체크리스트

- [ ] cargo-deny 0.19.4 license/bans/sources PASS
- [ ] `winx` SPDX exception policy와 owner/review 조건 확인
- [ ] Windows owner-only ACL 구현 또는 문서 경계 축소
- [ ] SC-CAUSE-01..07 개별 문서·테스트 매핑
- [ ] fmt / Clippy / 전체 359개 이상 테스트 / release build PASS
- [ ] cargo audit vulnerability/warning 0
- [ ] Windows 실제 R7/R8 exit 0
- [ ] `git diff --check` PASS
- [ ] clean final commit의 Ubuntu/Windows same-SHA CI 및 bundle PASS
- [ ] 외부 게시 별도 승인

## 14. 최종 판정

| Gate | 판정 |
| --- | --- |
| R0 Documentation | PASS WITH MINOR DOCUMENTATION GAP |
| R1 Build/Dependency | **FAIL — DBG-F011** |
| R2 State | PASS |
| R3 Content | PASS |
| R4 Determinism | PASS |
| R5 Workspace | PASS |
| R6 Local LLM | PASS |
| R7 Provenance/Compatibility | PASS locally |
| R8 Release | **HOLD** |
| R9 Causal closure | PASS locally |
| Security | PASS WITH MINOR RISK — SEC-F003 |
| report 23 remediation | **PASS locally except new dependency gate** |
| 최종 | **HOLD** |

판단 근거: report 23의 원래 blocker는 코드·테스트·문서·실행 증거로 시정됐다. 그러나 dependency policy는 필수 build/release gate이며 `winx` license와 현 allowlist가 명시적으로 충돌한다. Major finding이 남아 있으므로 전체 PASS를 선언할 수 없다.

## 15. Coder Handoff

```text
`C:\LocalDev\rust\AIHack\docs\audit\audit_report_24.md`의 재감사 결과를 확인하고,
DBG-F011을 최우선으로 실제 cargo-deny 0.19.4 PASS까지 시정하세요.
이후 SEC-F003의 Windows 권한 경계와 IMP-F019의 SC-CAUSE ID 매핑을 문서·코드·테스트에 대조해 정리하고,
전체 quality gate 및 clean same-SHA 양 OS CI 결과를 기록하세요.
```
