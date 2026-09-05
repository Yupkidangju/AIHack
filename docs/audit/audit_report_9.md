# AIHack D3D Re-audit Report 9

감사 기준: `AI_AUDIT_DOC_STANDARD.md`

감사 유형: `audit_report_8.md`의 IMP-F008 시정 후 독립 재감사

감사 일자: 2026-07-17 (Asia/Seoul)

감사 대상: 현재 working tree의 IMP-F008 시정, 연결 문서·테스트·빌드·보안 경계, R1~R5 전체 회귀

기준 commit: `49e3de8` (`main`)

환경: Linux 7.0.0-27-generic x86_64, rustc 1.94.1, cargo 1.94.1

감사 중 소스·기존 문서 수정: 없음

이번 감사가 생성한 파일: `audit_report_9.md`

## 1. 감사 요약

최종 판정: **PASS — audit_report_8 remediation scope closed**

`audit_report_8.md`의 IMP-F008은 시정됐다. `IMPLEMENTATION_SUMMARY.md`의 R3-2, R3-3, R3-4, R4-1, R4-2 파일 수가 실제 현재 owner 목록과 각각 5, 4, 8, 5, 4개로 일치하고, R6 시작 전 완료 Task 구간의 파일 목록과 선언 수를 비교하는 회귀 테스트가 추가됐다.

독립 계수 결과 R0~R4의 활성 파일 목록 15개가 모두 선언 수와 일치했다. 새 회귀 테스트를 포함한 전체 workspace 245개 test, fmt, metadata, check, clippy, debug/release build, RustSec, cargo-deny, dependency tree, TUI/headless CLI 및 seed 42 결정론 hash도 모두 통과했다. 보고서 8 이후 runtime source, manifest, lockfile, CI workflow 변경은 없으며 새 implementation/debug/security finding이나 cross-pass conflict는 발견되지 않았다.

| 구분 | 결과 |
| --- | --- |
| IMP-F008 Re-audit #1 | Verified |
| Full workspace test | PASS, 245 tests |
| Build/lint/supply-chain | PASS |
| Critical / Major / Minor | 0 / 0 / 0 |
| 신규 Security finding | 0건 |
| 보고서 8 시정 범위 | PASS |

이 PASS는 **보고서 8의 문서 시정 범위**에 대한 판정이다. SC-BUILD-02 원격 CI와 R6~R8은 여전히 pending/NOT RUN이므로 전체 프로그램 또는 release PASS를 의미하지 않는다.

## 2. Audit Scope

### 2.1 프로젝트 인벤토리

- 프로젝트 경로: `/mnt/Projects_SSD/rust/AIHack`
- 유형: Rust Cargo workspace 기반 CLI/TUI 턴제 로그라이크
- workspace: root compatibility facade, `crates/` 5개, `apps/` 2개
- source: `src/`, `crates/`, `apps/`
- tests: root `tests/`, crate/app별 `tests/`
- dependency/policy: `Cargo.toml`, `Cargo.lock`, member manifests, `deny.toml`, `rust-toolchain.toml`
- CI/CD: `.github/workflows/ci.yml`, `build.sh`, `build.bat`
- build/run 문서: `README.md`, `BUILD_GUIDE.md`
- control 문서: `spec.md`, `IMPLEMENTATION_SUMMARY.md`, `DESIGN_DECISIONS.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`
- 보조 문서: `designs.md`, `CHANGELOG.md`, `LESSONS_LEARNED.md`, `PROVENANCE.md`, `docs/compatibility/README.md`
- 감사 계보: `audit_report_1.md`~`audit_report_8.md`

### 2.2 `audit_report_8.md` 이후 변경 범위

보고서 8보다 modification time이 늦고 `target/` 및 Git/tool state를 제외한 변경 파일은 다음 6개다.

- `IMPLEMENTATION_SUMMARY.md`: IMP-F008의 5개 수량 정렬 및 재감사 대기 상태 기록
- `tests/build_contract.rs`: 완료 Task 목록-수량 일치 회귀 테스트 추가
- `GAP_CLOSURE_ROADMAP.md`: 시정 반영 및 독립 재감사 대기 상태 기록
- `audit_roadmap.md`: R5 document re-audit pending 유지
- `README.md`: IMP-F008 시정 후 독립 재감사 대기 상태 동기화
- `CHANGELOG.md`: IMP-F008 시정과 회귀 테스트 추가 기록

runtime source, dependency manifests, `Cargo.lock`, build scripts, CI workflow 및 보안 경계에는 보고서 8 이후 변경이 없다.

### 2.3 확인한 문서·파일과 검사 케이스

- `audit_report_8.md`의 IMP-F008 evidence, expected, suggested fix, re-audit method
- `IMPLEMENTATION_SUMMARY.md` R0~R5 완료 Task 파일 owner와 범위 수량
- `tests/build_contract.rs`의 신규 목록-수량 회귀 테스트와 기존 path/command/output 정책
- `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`, `README.md`, `CHANGELOG.md`의 시정 상태와 authority 과대주장 여부
- active 문서의 package/workspace selector와 삭제된 monolith 경로 잔존 여부
- 전체 workspace test와 long-run 3 seed × 3회 결정론 회귀
- fmt, metadata, check, clippy, debug/release build
- RustSec, licenses/bans/sources, core dependency 순도, crossterm 중복
- TUI/headless CLI 및 seed 42 accepted-turn/hash 스모크
- `/output/` ignore와 diff whitespace 상태

## 3. Excluded Scope

- R6 live local LLM transport, 강제 timeout, stale correlation, soft adjudication: `NOT RUN`
- R7 NH367-C001..C010 호환성 구현 및 법적 provenance 승인: `NOT RUN`
- R8 v0.3.0 release/version/packaging: `NOT RUN`
- SC-BUILD-02 Linux/Windows 원격 CI 실제 green evidence: pending
- 장시간 실제 terminal TUI 입력·복원·시각·접근성 수동 검수
- `legacy_nethack_port_reference/`: reference-only, shipped scope 아님
- `target/`, `runtime/`, `.archive/`, `.omx/`, `.antigravitycli/`: generated/archive/tool state
- `output/` 바이너리의 내부 provenance 및 재현 빌드 비교: ignore 동작만 검사
- 외부 advisory DB 갱신: 설치된 DB를 `cargo audit --no-fetch`로 검사
- 법률 자문, 원격 CI 실행, Git commit readiness

## 4. 실행 명령과 결과

### 4.1 IMP-F008 표적 검증

| 검사 | 결과 |
| --- | --- |
| R0~R5 완료 구간의 활성 목록-수량 독립 계수 | PASS, 목록 15개 모두 일치 |
| `cargo test -p aihack --locked --test build_contract --test workspace_boundaries` | PASS, 9 tests |
| 삭제된 monolith 경로 정적 검색 | PASS, 현재 경로 substring 또는 명시적 `old -> new` 이력만 존재 |
| under-scoped `cargo test/build/check --locked` 정적 검색 | PASS, 결과 0건 |
| `git check-ignore -v output/aihack output/aihack-headless` | PASS, `.gitignore:5:/output/` |
| `git diff --check` | PASS |

### 4.2 전체 회귀·품질·공급망 검증

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS, 8 workspace members, default member `aihack-tui` |
| `cargo check --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 245 tests, 실패 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo build --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --locked` | PASS |
| `cargo tree -p aihack-core --locked` | PASS, UI/terminal/network dependency 없음 |
| `cargo tree -i crossterm --locked` | PASS, crossterm 0.29.0 단일 버전 |
| `cargo audit --no-fetch` | PASS, 1160 advisories로 214 dependencies scan |
| `cargo deny check licenses bans sources` | PASS, bans/licenses/sources 모두 ok |
| TUI `--help` | PASS, binary `aihack`, option `--seed` |
| headless `--help` | PASS, 기존 8개 flag 유지 |
| seed 42, survival-v1, target 10 | PASS, accepted 10, hash `e7d30d72027a39c0` |

`cargo audit --no-fetch`는 crates.io package-cache lock warning을 출력했지만 로컬 advisory DB scan을 완료하고 exit 0이었다. 전체 test는 artifact directory lock을 잠시 대기했으나 정상 완료해 제품 실패로 분류하지 않는다.

## 5. Pass 1: Implementation Compliance Findings

### [IMP-F008 Re-audit #1] 완료 Task의 현재 파일 목록과 `범위` 파일 수가 불일치

- Pass: Implementation Compliance
- Pattern: IMP-003, IMP-004
- Area: `IMPLEMENTATION_SUMMARY.md` 완료 Task 범위 메타데이터, `tests/build_contract.rs`
- Severity: Minor
- Status: **Verified**
- Summary: 보고서 8에서 지적한 5개 목록-수량 불일치가 모두 시정되고 회귀 테스트로 고정됐다.
- Evidence:
  - R3-2는 5개/5개로 일치한다 (`IMPLEMENTATION_SUMMARY.md:401-402`).
  - R3-3은 4개/4개로 일치한다 (`IMPLEMENTATION_SUMMARY.md:425-426`).
  - R3-4는 8개/8개로 일치한다 (`IMPLEMENTATION_SUMMARY.md:448-449`).
  - R4-1은 5개/5개로 일치한다 (`IMPLEMENTATION_SUMMARY.md:481-482`).
  - R4-2는 4개/4개로 일치한다 (`IMPLEMENTATION_SUMMARY.md:503-504`).
  - 독립 계수는 R6 시작 전 활성 파일 목록 15개 전부의 선언 수 일치를 확인했다.
  - `tests/build_contract.rs:127-169`의 `completed_task_file_counts_match_their_active_owner_lists`가 R6 시작 전 완료 Task 구간을 파싱해 backtick owner 수와 `범위` 선언 수를 비교한다.
  - 신규 테스트를 포함한 build contract 8개와 workspace boundary 1개가 통과했다.
  - 관련 문서는 시정 반영 후에도 독립 재감사 대기 상태를 유지해 자체 PASS를 선점하지 않았다.
- Expected: 활성 완료 Task의 정량 범위가 실제 현재 owner 목록과 일치하고 향후 drift를 자동 검출한다.
- Actual: 기대와 일치한다.
- Impact: 작업 범위·책임표의 정량 신뢰가 복구되고 동일 drift의 재발을 build contract gate가 차단한다.
- Suggested Fix: 없음.
- Re-audit Method: 독립 목록 계수, build contract test, full workspace regression 및 관련 문서 상태 대조.
- Owner: Auditor verified
- Notes: IMP-F008을 종결한다. 후속 R6~R8의 계획 Task는 완료 범위가 아니며 이번 finding의 대상이 아니다.

### 5.2 Verified implementation evidence

- 보고서 7의 IMP-F007, DBG-F002, DBG-F003 closure 유지
- R5 workspace 8 members, one-way dependency, root compatibility facade 및 app-owned production binaries 유지
- R2 transaction/invariant, R3 content/bootstrap, R4 long-run/save/replay 회귀 통과
- R6~R8은 `NOT RUN`/Open으로 유지돼 scope 과대주장 없음

## 6. Pass 2: Debug / Engineering Quality Findings

새 Debug/Engineering Quality finding 없음.

Verified evidence:

- 신규 회귀 테스트가 IMP-F008의 실제 실패 모드인 owner 목록과 선언 수 불일치를 직접 이름 붙이고 비교한다.
- targeted 9 tests와 전체 workspace 245 tests가 통과했다.
- fmt, check, clippy `-D warnings`, debug/release build가 모두 통과했다.
- active 문서의 root integration test와 workspace selector 계약이 유지된다.
- seed 42 CLI 스모크와 장기 결정론 테스트가 기존 hash/accepted-turn 계약을 유지한다.

## 7. Pass 3: Security Findings

새 Security finding 없음.

Verified evidence:

- 보고서 8 이후 runtime, dependency, CI, filesystem 처리 코드 변경 없음
- `cargo audit --no-fetch` exit 0, 214 dependencies scan
- cargo-deny bans/licenses/sources 모두 PASS
- `aihack-core`에 UI/terminal/network dependency 없음
- crossterm 0.29.0 단일 버전이며 TUI 경로에만 존재
- path traversal/symlink escape를 포함한 전체 회귀 통과
- `/output/` generated artifact ignore 유지

## 8. Cross-Pass Conflicts

없음.

문서의 시정·대기 상태, contract test, 전체 실행 결과가 모두 IMP-F008 해소를 지지한다. Pass 1의 remediation PASS를 전체 프로그램 PASS로 확장하지 않으므로 pending Phase와의 authority 충돌도 없다.

## 9. Required Fixes Before PASS

보고서 8 remediation scope에는 없음.

R6~R8 구현과 SC-BUILD-02 원격 CI는 이번 시정의 required fix가 아니라 기존 후속 gate다.

## 10. Accepted Risks

없음.

설치된 advisory DB 사용, 원격 CI pending, R6~R8 NOT RUN은 숨은 면제가 아니라 아래 Remaining Risks 및 기존 roadmap에서 명시적으로 추적한다.

## 11. Needs Spec Clarification

없음.

## 12. Re-audit Checklist

IMP-F008은 Verified로 종결됐다. 향후 `IMPLEMENTATION_SUMMARY.md`의 완료 Task owner 또는 수량을 변경하면 다음을 재실행한다.

```bash
cargo test -p aihack --locked --test build_contract --test workspace_boundaries
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --release --locked
cargo audit --no-fetch
cargo deny check licenses bans sources
git diff --check
```

정적 확인:

```bash
rg -n '^\*\*(현재 파일|파일|범위):' IMPLEMENTATION_SUMMARY.md
rg -n 'src/data/schema.rs|src/data/levels/main_2.toml|src/bin/aihack-headless.rs|src/ui/tui/' \
  IMPLEMENTATION_SUMMARY.md
rg -n '^cargo (test|build|check) --locked' \
  spec.md IMPLEMENTATION_SUMMARY.md audit_roadmap.md \
  GAP_CLOSURE_ROADMAP.md BUILD_GUIDE.md README.md
```

## 13. Remaining Risks

- SC-BUILD-02 Linux/Windows 원격 CI evidence pending
- R6 local LLM, R7 provenance/compatibility, R8 release NOT RUN
- 실제 terminal의 장시간 TUI UX/restore 수동 검수 미수행
- working tree는 기존 대규모 이동·미추적 파일을 포함하며 commit readiness와 change ownership은 이번 감사 범위가 아님
- 설치된 advisory DB만 사용했으므로 최신 원격 RustSec DB 상태는 별도 CI/온라인 감사에서 확인 필요
- 이번 단일 감사는 최종 release의 인간 또는 복수 모델 교차감사를 대체하지 않음

## 14. Final Decision

**PASS — audit_report_8 remediation scope closed**

| Gate | 판정 |
| --- | --- |
| IMP-F008 | Verified; 5개 수량 정렬 및 회귀 테스트 PASS |
| 보고서 7 IMP-F007/DBG-F002/DBG-F003 | 기존 Verified 상태 유지 |
| R1 build | local full-workspace PASS, SC-BUILD-02 remote CI pending |
| R2 state/transaction | local PASS |
| R3 content/bootstrap | local PASS |
| R4 long-run | local PASS |
| R5 workspace | runtime/architecture/document remediation PASS |
| R6 local LLM | NOT RUN |
| R7 provenance/compatibility | NOT RUN |
| R8 release | NOT RUN |
| 전체 프로그램/release | 아직 PASS 대상 아님 |

코드와 기존 문서는 수정하지 않고 감사 보고서만 생성했다. 다음 구현 단계는 프로젝트 roadmap과 승인 순서에 따라 별도 코더가 진행한다.
