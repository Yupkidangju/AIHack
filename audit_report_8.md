# AIHack D3D Re-audit Report 8

감사 기준: `AI_AUDIT_DOC_STANDARD.md`

감사 유형: `audit_report_7.md`의 IMP-F007, DBG-F002, DBG-F003 시정 후 독립 재감사

감사 일자: 2026-07-17 (Asia/Seoul)

감사 대상: 현재 working tree의 R1~R5 구현·문서·빌드·보안 경계 및 보고서 7 시정 범위

기준 commit: `49e3de8` (`main`)

환경: Linux 7.0.0-27-generic x86_64, rustc 1.94.1, cargo 1.94.1

감사 중 소스·기존 문서 수정: 없음

이번 감사가 생성한 파일: `audit_report_8.md`

## 1. 감사 요약

최종 판정: **HOLD — 이전 3개 finding은 Verified, 새 문서 수량 drift 1건 수정 필요**

`audit_report_7.md`에서 지적한 IMP-F007, DBG-F002, DBG-F003은 모두 시정됐다. 활성 문서의 Cargo 명령은 현재 workspace/package를 올바르게 선택하며, 완료 Task의 과거 monolith 경로는 현재 crate/app owner로 교체됐고, `/output/`은 Git ignore 정책에 포함됐다. 이를 직접 재실행한 결과 targeted test, 전체 workspace 244개 test, fmt, clippy, debug/release build, RustSec, cargo-deny, dependency tree, TUI/headless CLI와 seed 42 결정론 hash가 모두 통과했다.

다만 IMP-F007 시정 과정에서 `IMPLEMENTATION_SUMMARY.md`의 현재 파일 목록은 갱신됐지만 바로 아래 `범위` 파일 수 5곳이 목록과 일치하지 않는다. 구현·빌드 실패는 아니며 severity는 Minor지만, 활성 implementation authority 내부의 정량 정보가 서로 모순되고 아직 Known Issue, 후속 Phase 또는 Accepted Risk로 추적되지 않아 문서 재감사 closure는 HOLD로 유지한다.

| 구분 | 결과 |
| --- | --- |
| IMP-F007 Re-audit #1 | Verified |
| DBG-F002 Re-audit #1 | Verified |
| DBG-F003 Re-audit #1 | Verified |
| Full workspace test | PASS, 244 tests |
| Build/lint/supply-chain | PASS |
| 새 finding | Minor 1건 (`IMP-F008`) |
| Critical / Major | 0건 / 0건 |
| 신규 Security finding | 0건 |

이 HOLD는 R1~R5 runtime 또는 보고서 7의 핵심 시정을 기각하지 않는다. 남은 작업은 완료 Task 5개의 파일 수를 실제 현재 목록과 맞추고 해당 drift를 계약 테스트로 방지하는 국소 문서 수정이다.

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
- 감사 계보: `audit_report_1.md`~`audit_report_7.md`

### 2.2 `audit_report_7.md` 이후 시정 범위

보고서 7 이후 감사와 직접 관련된 변경 파일은 다음과 같다.

- `.gitignore`
- `spec.md`
- `BUILD_GUIDE.md`
- `README.md`
- `CHANGELOG.md`
- `GAP_CLOSURE_ROADMAP.md`
- `IMPLEMENTATION_SUMMARY.md`
- `audit_roadmap.md`
- `tests/build_contract.rs`

runtime source, dependency manifest, lockfile 및 CI workflow에는 보고서 7 이후 시정 변경이 없다. 따라서 재감사는 문서-구현 정합성 Pass 1, 명령/테스트/빌드 Pass 2, artifact 및 공급망 경계 Pass 3을 연결 범위로 수행하고 전체 workspace 회귀로 비연결 영역의 부작용도 확인했다.

### 2.3 검사한 케이스

- IMP-F007의 현재 파일 owner 경로와 과거 migration 이력 분리
- DBG-F002의 root integration test package selector 및 전체 workspace selector
- DBG-F002를 방지하는 build contract test의 범위와 실행 결과
- DBG-F003의 `/output/` ignore 및 working-tree 노출 상태
- 시정된 문서의 신규 drift 여부
- 전체 workspace test, fmt, clippy, debug/release build
- core dependency 순도, crossterm 중복, RustSec, license/bans/sources
- TUI/headless CLI surface 및 seed 42 accepted-turn/hash 결정론
- R6~R8 및 원격 CI 상태의 과대표현 여부

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

### 4.1 통과한 검증

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS, 8 workspace members, default member `aihack-tui` |
| `cargo test -p aihack --locked --test build_contract --test workspace_boundaries` | PASS, 8 tests |
| `cargo test -p aihack --locked --test world_invariants --test transaction` | PASS, 6 tests |
| `cargo test -p aihack --locked --test content_validation --test content_runtime` | PASS, 10 tests |
| `cargo test -p aihack --locked --test data_loading --test items --test monster_ai --test levels` | PASS, 42 tests |
| `cargo test --workspace --all-targets --locked` | PASS, 244 tests, 실패 0 |
| `cargo check --workspace --all-targets --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo build --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --locked` | PASS |
| `cargo tree -p aihack-core --locked` | PASS, UI/terminal/network dependency 없음 |
| `cargo tree -i crossterm --locked` | PASS, crossterm 0.29.0 단일 버전 |
| `cargo audit --no-fetch` | PASS, 1160 advisories로 214 dependencies scan |
| `cargo deny check licenses bans sources` | PASS, bans/licenses/sources 모두 ok |
| `git check-ignore -v output/aihack output/aihack-headless` | PASS, `.gitignore:5:/output/` |
| `git status --short -- output .gitignore` | PASS, `output/` 미노출; `.gitignore` 수정만 표시 |
| TUI `--help` | PASS, binary `aihack`, option `--seed` |
| headless `--help` | PASS, 기존 8개 flag 유지 |
| seed 42, survival-v1, target 10 | PASS, accepted 10, hash `e7d30d72027a39c0` |
| `git diff --check` | PASS |

`cargo audit --no-fetch`는 crates.io package-cache lock warning을 출력했지만 로컬 advisory DB scan을 완료하고 exit 0이었다.

### 4.2 감사자 명령 보정 기록

처음 실행한 `cargo run -p aihack -- --help`는 production binary가 아닌 root compatibility facade package를 명시해 exit 101이었고, headless에 존재하지 않는 `--target-level`을 준 명령은 exit 2였다. 문서 계약을 재확인한 뒤 아래 canonical 명령으로 보정해 모두 통과했다.

```bash
cargo run -q --locked --bin aihack -- --help
cargo run -q --locked -p aihack-headless --bin aihack-headless -- --help
cargo run -q --locked -p aihack-headless --bin aihack-headless -- \
  --seed 42 --turns 10 --policy survival-v1
```

위 초기 실패는 문서 또는 제품 명령의 실패가 아니라 감사자가 잘못 구성한 invocation이며 finding으로 분류하지 않는다.

## 5. Pass 1: Implementation Compliance Findings

### [IMP-F007 Re-audit #1] 완료 Task의 파일 책임 경로가 현재 workspace와 불일치

- Pass: Implementation Compliance
- Pattern: IMP-003, IMP-004, DOC-BACKFILL-001
- Area: `IMPLEMENTATION_SUMMARY.md` R2~R4 완료 Task 책임표
- Severity: Minor
- Status: **Verified**
- Summary: 보고서 7에서 지적한 삭제된 monolith 경로가 활성 Task의 현재 파일 목록에서 제거되고 현재 crate/app owner로 정렬됐다.
- Evidence:
  - R2-1~R2-3 현재 파일은 `crates/aihack-runtime`, `crates/aihack-core`, `tests/` owner를 사용한다 (`IMPLEMENTATION_SUMMARY.md:297`, `321`, `346`).
  - R3-1~R3-4는 `crates/aihack-content`, `crates/aihack-runtime`, 현재 test owner를 사용한다 (`IMPLEMENTATION_SUMMARY.md:377`, `401`, `425`, `448`).
  - R4-1~R4-2는 `apps/aihack-headless`, runtime 및 현재 root test owner를 사용한다 (`IMPLEMENTATION_SUMMARY.md:481`, `503`).
  - 이전 `src/data/schema.rs`, `src/data/levels/main_2.toml`, `src/bin/aihack-headless.rs`, `src/ui/tui/` 검색 결과는 `IMPLEMENTATION_SUMMARY.md:548-585`의 명시적 `old -> new` migration 표와 현재 경로 문자열에만 남는다.
  - `tests/build_contract.rs:103-125`가 활성 Task owner와 `/output/` 정책을 회귀 검사한다.
- Expected: active Task는 현재 owner를 가리키고 과거 경로는 migration/history 문맥에만 남는다.
- Actual: 기대와 일치한다.
- Impact: 원 finding의 잘못된 수정 경로 유도 위험이 해소됐다.
- Suggested Fix: 없음. 새로 확인한 파일 수 불일치는 IMP-F008로 분리한다.
- Re-audit Method: stale monolith 경로 검색, 현재 파일 존재/owner 대조, build contract test 재실행.
- Owner: Auditor verified
- Notes: 원 finding은 종결한다. 시정 과정의 별도 정량 drift는 새 finding으로 추적한다.

### [IMP-F008] 완료 Task의 현재 파일 목록과 `범위` 파일 수가 불일치

- Pass: Implementation Compliance
- Pattern: IMP-003, IMP-004
- Area: `IMPLEMENTATION_SUMMARY.md` R3-2~R4-2 완료 Task 범위 메타데이터
- Severity: Minor
- Status: **Needs Fix**
- Summary: 현재 owner 경로 목록은 고쳐졌지만 인접한 `범위`의 파일 수가 5개 Task에서 실제 나열 수와 다르다.
- Evidence:
  - R3-2는 5개 파일을 나열하지만 `M, 4개`로 기록한다 (`IMPLEMENTATION_SUMMARY.md:401-402`).
  - R3-3은 4개 파일을 나열하지만 `M, 5개`로 기록한다 (`IMPLEMENTATION_SUMMARY.md:425-426`).
  - R3-4는 8개 파일을 나열하지만 `M, 9개`로 기록한다 (`IMPLEMENTATION_SUMMARY.md:448-449`).
  - R4-1은 5개 파일을 나열하지만 `M, 6개`로 기록한다 (`IMPLEMENTATION_SUMMARY.md:481-482`).
  - R4-2는 4개 파일을 나열하지만 `M, 3개`로 기록한다 (`IMPLEMENTATION_SUMMARY.md:503-504`).
  - R2-1, R2-2, R2-3, R3-1은 각각 4/4, 4/4, 5/5, 4/4로 일치한다.
- Expected: 활성 완료 Task의 정량 범위는 바로 위 현재 파일 목록의 실제 개수와 일치하거나, 다른 산정 기준이면 그 기준을 명시한다.
- Actual: 5개 Task에서 목록과 수량이 ±1씩 어긋나며 별도 산정 기준 설명이 없다.
- Impact: runtime 동작에는 영향이 없지만 작업 범위, 책임표, 변경 영향 분석을 기계 또는 사람이 신뢰하기 어렵고 IMP-F007 시정 완료를 과대평가할 수 있다.
- Suggested Fix: 위 5개 수량을 각각 5, 4, 8, 5, 4개로 정렬한다. 파일 수 메타데이터를 유지한다면 build contract test에서 목록과 수량을 함께 검증한다.
- Re-audit Method: R1~R5 완료 Task의 `현재 파일`/`파일` 항목을 파싱 또는 수동 계수해 모든 `범위` 수량과 일치하는지 확인하고 build contract test를 재실행한다.
- Owner: Coder, Auditor verification
- Notes: 요구가 명확하고 국소 문서 수정으로 해소 가능하므로 Needs Documentation Recovery가 아니라 Needs Fix다.

### 5.3 Verified implementation evidence

- R5 workspace 8 members와 one-way dependency 경계 유지
- root compatibility facade와 app-owned production binaries 유지
- R2 transaction/invariant, R3 content/bootstrap, R4 accepted-turn/hash/save/replay 회귀 통과
- R6~R8은 `NOT RUN`/Open으로 남아 scope 과대주장 없음

## 6. Pass 2: Debug / Engineering Quality Findings

### [DBG-F002 Re-audit #1] R2/R3 canonical audit 명령이 workspace default member에서 실패

- Pass: Debug / Engineering Quality
- Pattern: BUILD-001, IMP-004, TEST-001
- Area: active control/build documents와 `tests/build_contract.rs`
- Severity: Major
- Status: **Verified**
- Summary: root integration test에 `-p aihack`, 전체 범위 명령에 `--workspace`가 적용되어 보고서 7의 실패와 과소 선택 위험이 해소됐다.
- Evidence:
  - `spec.md:51`의 build gate가 `cargo build --workspace --all-targets --locked`를 사용한다.
  - `audit_roadmap.md:187-190`, `210-212`의 R2/R3 명령이 `cargo test -p aihack`을 사용한다.
  - `GAP_CLOSURE_ROADMAP.md:93-95`, `BUILD_GUIDE.md:129`, `141-143`이 workspace 범위와 실제 script 동작을 일치시킨다.
  - `IMPLEMENTATION_SUMMARY.md`의 root integration test 명령도 `-p aihack`을 사용한다.
  - `tests/build_contract.rs:77-101`은 6개 active 문서에서 보고서 7의 under-scoped 명령 재등장을 금지한다.
  - build contract/workspace boundary 8 tests, 보고서 7의 R2/R3 targeted commands, 전체 workspace 244 tests가 모두 통과했다.
  - `audit_roadmap.md`는 자체 PASS를 선언하지 않고 document re-audit pending 및 보고서 8 대기를 명시한다.
- Expected: canonical 명령이 현재 workspace에서 그대로 실행되고 주장한 package 범위를 검사한다.
- Actual: 기대와 일치한다.
- Impact: 독립 감사 재현성과 전체 workspace gate 신뢰가 복구됐다.
- Suggested Fix: 없음.
- Re-audit Method: 문서 명령의 selector 정적 검사, targeted/full workspace 명령 직접 실행, contract test 실행.
- Owner: Auditor verified
- Notes: 원 Major finding을 종결한다.

### [DBG-F003 Re-audit #1] 생성 binary 디렉터리 `output/`이 Git ignore 정책 밖에 있음

- Pass: Debug / Engineering Quality
- Pattern: BUILD-001, DEP-001
- Area: `.gitignore`, generated artifacts, build contract
- Severity: Minor
- Status: **Verified**
- Summary: `/output/`이 repository ignore 정책과 회귀 테스트에 포함됐다.
- Evidence:
  - `.gitignore:5`에 `/output/`이 존재한다.
  - `git check-ignore -v output/aihack output/aihack-headless`가 두 파일 모두 `.gitignore:5:/output/`로 성공한다.
  - `git status --short -- output .gitignore`는 `output/`을 표시하지 않고 의도된 `.gitignore` 수정만 표시한다.
  - `tests/build_contract.rs:103-125`가 generated output policy를 검사한다.
- Expected: 추적하지 않는 generated binary가 working tree 후보로 노출되지 않는다.
- Actual: 기대와 일치한다.
- Impact: 대용량 생성 바이너리의 실수 커밋 및 상태 잡음 위험이 해소됐다.
- Suggested Fix: 없음.
- Re-audit Method: `git check-ignore`, scoped `git status`, build contract test 재실행.
- Owner: Auditor verified
- Notes: 원 finding을 종결한다.

## 7. Pass 3: Security Findings

새 Critical/Major/Minor security finding 없음.

Verified evidence:

- `cargo audit --no-fetch` exit 0, 214 dependencies scan
- cargo-deny bans/licenses/sources 모두 PASS
- `aihack-core` dependency는 rand, serde, thiserror와 transitive dependencies로 제한
- crossterm 0.29.0 단일 버전이며 TUI 경로에만 존재
- save/report path traversal 및 symlink escape 관련 전체 회귀 통과
- `/output/` ignore 정책으로 generated binary의 우발적 commit 노출 감소
- R6 network/LLM transport는 아직 `NOT RUN` 범위로 유지

## 8. Cross-Pass Conflicts

### [XPF-F006 Re-audit #1] R1~R5 PASS authority와 실행 불가능한 재감사 절차의 충돌

- Pass: Cross-Pass
- Pattern: IMP-004, BUILD-001
- Area: R1~R5 문서 authority와 실행 evidence
- Severity: Major
- Status: **Verified**
- Summary: DBG-F002의 canonical 명령이 복구되고 IMP-F007의 현재 owner 경로가 정렬돼 원래의 문서-실행 충돌은 해소됐다.
- Evidence: active document selector contract, targeted commands, 전체 244 tests 및 build/lint gate가 같은 PASS 결론을 지지한다.
- Expected: 문서와 실행 evidence가 동일한 범위와 결과를 가리킨다.
- Actual: 원 conflict에 대해서는 기대와 일치한다.
- Impact: 보고서 7의 Major cross-pass HOLD 사유는 제거됐다.
- Suggested Fix: 없음. IMP-F008은 새 Minor implementation-document finding으로 별도 처리한다.
- Re-audit Method: DBG-F002/IMP-F007 evidence와 full gate를 함께 재확인.
- Owner: Auditor verified
- Notes: 원 cross-pass conflict는 종결한다.

현재 활성 cross-pass conflict는 없다. IMP-F008은 문서 내부의 국소 정량 불일치이며 runtime/build/security 결과와 상충하지 않는다.

## 9. Required Fixes Before PASS

1. `IMPLEMENTATION_SUMMARY.md`의 R3-2, R3-3, R3-4, R4-1, R4-2 파일 수를 실제 현재 파일 목록과 각각 5, 4, 8, 5, 4개로 정렬한다.
2. 파일 수 메타데이터를 유지할 경우 `tests/build_contract.rs`에 목록-수량 일치 회귀 검사를 추가한다.
3. 수정 후 build contract test, full workspace test 및 문서 정적 검사를 재실행한다.

## 10. Accepted Risks

없음.

IMP-F008은 owner/사유/만료/재검토 조건을 갖춘 Accepted Risk로 기록되지 않았으므로 자동 면제하지 않는다. SC-BUILD-02 remote CI pending과 R6~R8 NOT RUN도 Accepted Risk가 아니라 별도 pending/deferred gate다.

## 11. Needs Spec Clarification

없음. 현재 파일 목록 바로 아래의 파일 수라는 문맥이 명확하며, 다른 산정 기준은 문서에 정의되어 있지 않다.

## 12. Re-audit Checklist

```bash
cargo fmt --all -- --check
cargo test -p aihack --locked --test build_contract --test workspace_boundaries
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo audit --no-fetch
cargo deny check licenses bans sources
git check-ignore -v output/aihack output/aihack-headless
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

완료 기준:

- R1~R5의 모든 `현재 파일`/`파일` 나열 수와 인접 `범위` 수량 일치
- 과거 monolith 경로는 명시적 migration/history 문맥에만 존재
- active command의 package/workspace selector contract test 유지
- 전체 regression과 supply-chain gate 재통과

## 13. Remaining Risks

- SC-BUILD-02 Linux/Windows 원격 CI evidence pending
- R6 local LLM, R7 provenance/compatibility, R8 release NOT RUN
- 실제 terminal의 장시간 TUI UX/restore 수동 검수 미수행
- working tree는 기존 대규모 이동·미추적 파일을 포함하며 commit readiness와 change ownership은 이번 감사 범위가 아님
- 설치된 advisory DB만 사용했으므로 최신 원격 RustSec DB 상태는 별도 CI/온라인 감사에서 확인 필요
- 이번 단일 감사는 최종 release의 인간 또는 복수 모델 교차감사를 대체하지 않음

## 14. Final Decision

**HOLD — report 7 remediation verified; IMP-F008 documentation count drift remains**

| Gate | 판정 |
| --- | --- |
| 보고서 7 IMP-F007 | Verified; stale current paths 해소 |
| 보고서 7 DBG-F002 | Verified; canonical commands와 contract tests PASS |
| 보고서 7 DBG-F003 | Verified; `/output/` ignore PASS |
| R1 build | local full-workspace PASS, SC-BUILD-02 remote CI pending |
| R2 state/transaction | PASS |
| R3 content/bootstrap | PASS; Task 범위 수량 drift 3곳 |
| R4 long-run | PASS; Task 범위 수량 drift 2곳 |
| R5 workspace | runtime/architecture/document command evidence PASS |
| R6 local LLM | NOT RUN |
| R7 provenance/compatibility | NOT RUN |
| R8 release | NOT RUN |

코드 수정 없이 감사 보고서만 생성했다. 별도 코더는 IMP-F008의 5개 수량과 회귀 검사를 시정한 뒤 다음 순번 보고서로 재감사해야 한다.
