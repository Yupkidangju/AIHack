# AIHack D3D Current-State Audit Report 7

감사 기준: `AI_AUDIT_DOC_STANDARD.md`

감사 유형: `audit_report_6.md` 이후 현재 working tree 독립 감사

감사 일자: 2026-07-17 (Asia/Seoul)

감사 대상: 현재 working tree의 R1~R5 구현·문서·빌드·보안 경계

기준 commit: `49e3de8` (`main`)

환경: Linux 7.0.0-27-generic x86_64, rustc 1.94.1, cargo 1.94.1

감사 중 소스·기존 문서 수정: 없음

이번 감사가 생성한 파일: `audit_report_7.md`

## 1. 감사 요약

최종 판정: **HOLD — runtime 회귀 없음, 감사 재현성 문서 수정 필요**

현재 코드와 실제 전체 workspace gate는 정상이다. 242개 테스트, fmt, clippy, debug/release build, metadata, RustSec, cargo-deny, dependency tree, TUI/headless CLI와 결정론 hash가 모두 통과했다. `audit_report_6.md` 이후 변경된 6개 파일도 R5 PASS를 control 문서에 반영한 post-audit bookkeeping이며 source 변경은 없었다.

그러나 R5 workspace 전환 이후에도 활성 문서 여러 곳에 단일-package 시절의 Cargo 명령과 파일 경로가 남아 있다. 특히 `audit_roadmap.md`의 R2/R3 명령은 현재 tree에서 exit 101로 실패하므로 새 감사자가 checkpoint evidence를 문서 그대로 재현할 수 없다. 이는 코드 실패가 아니라 `BUILD-001`/`IMP-004` 문서-실행 drift지만, canonical audit procedure가 실행 불가능한 Major finding이므로 이번 감사는 HOLD다.

| 구분 | 결과 |
| --- | --- |
| Runtime/source regression | 없음 |
| Full workspace test | PASS, 242 tests |
| Build/lint/supply-chain | PASS |
| Major | 1건 (`DBG-F002`) |
| Minor | 2건 (`IMP-F007`, `DBG-F003`) |
| Critical | 0건 |
| Security finding | 신규 0건 |

이 HOLD는 R1~R5 구현 결과를 기각하지 않는다. 문서 명령과 책임표를 현재 workspace에 맞춘 뒤 재감사해야 한다. R6~R8 및 SC-BUILD-02 원격 CI는 이번 finding이 아니라 기존 pending/deferred scope다.

## 2. Audit Scope

### 2.1 프로젝트 인벤토리

- 프로젝트 경로: `/mnt/Projects_SSD/rust/AIHack`
- 유형: Rust Cargo workspace 기반 CLI/TUI 턴제 로그라이크
- workspace: root compatibility facade, `crates/` 5개, `apps/` 2개
- source: `src/`, `crates/`, `apps/`
- tests: root `tests/`, crate/app별 `tests/`
- dependency/policy: `Cargo.toml`, `Cargo.lock`, member manifests, `deny.toml`, `rust-toolchain.toml`
- CI: `.github/workflows/ci.yml`
- build/run: `README.md`, `BUILD_GUIDE.md`, `build.sh`, `build.bat`
- control 문서: `spec.md`, `IMPLEMENTATION_SUMMARY.md`, `DESIGN_DECISIONS.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`
- 보조 문서: `designs.md`, `CHANGELOG.md`, `LESSONS_LEARNED.md`, `PROVENANCE.md`, `docs/compatibility/README.md`
- 감사 계보: `audit_report_1.md`~`audit_report_6.md`

### 2.2 `audit_report_6.md` 이후 변경 범위

파일 modification time 기준 다음 6개 문서만 `audit_report_6.md` 이후 변경됐다.

- `GAP_CLOSURE_ROADMAP.md`
- `DESIGN_DECISIONS.md`
- `audit_roadmap.md`
- `IMPLEMENTATION_SUMMARY.md`
- `README.md`
- `CHANGELOG.md`

변경 내용은 G-TEST-001/002와 G-ARCH-001 closure, R5 PASS checkpoint, 최신 감사 링크를 `audit_report_6.md`에 연결하는 bookkeeping이다. source, tests, manifests, lockfile, CI에는 보고서 6 이후 변경이 없다.

### 2.3 검사한 케이스

- R1~R5 문서 주장과 실제 Cargo workspace/package selection 대조
- 전체 workspace의 242개 test와 장기 결정론 3 seed × 3회
- core/content/runtime/app dependency 방향과 path dependency version 정책
- fmt, clippy `-D warnings`, debug/release build, metadata, CLI 호환성
- save/report path traversal 및 symlink escape 회귀
- RustSec advisory, license, bans, sources
- 생성 artifact의 Git 추적/ignore 상태
- R6~R8이 호출 가능 완료 상태로 과대표현되는지 여부

## 3. Excluded Scope

- R6 live local LLM transport, 강제 timeout, stale correlation, soft adjudication: `NOT RUN`
- R7 NH367-C001..C010 호환성 구현 및 법적 provenance 승인: `NOT RUN`
- R8 v0.3.0 release/version/packaging: `NOT RUN`
- SC-BUILD-02 Linux/Windows 원격 CI 실제 green evidence: pending
- 장시간 실제 terminal TUI 입력·복원·시각·접근성 수동 검수
- `legacy_nethack_port_reference/`: reference-only, shipped scope 아님
- `target/`, `runtime/`, `.archive/`, `.omx/`, `.antigravitycli/`: generated/archive/tool state
- `output/` 바이너리의 내부 내용 및 provenance 검증: 생성 artifact 상태와 크기만 검사
- 외부 advisory DB 갱신: 설치된 DB를 `cargo audit --no-fetch`로 검사
- standalone `SECURITY.md`, `kanban_board.md`: 존재하지 않음. 현재 보안/phase authority는 `spec.md`, `DESIGN_DECISIONS.md`, gap/audit roadmap에서 확인
- 법률 자문 및 원격 CI 실행

## 4. 실행 명령과 결과

### 4.1 통과한 검증

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS, 8 workspace members, default member는 `aihack-tui` |
| `cargo test -p aihack --test build_contract --test workspace_boundaries --locked` | PASS, 6 tests |
| `cargo test --workspace --all-targets --locked` | PASS, 242 tests, 실패 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo build --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --locked` | PASS |
| `cargo tree -p aihack-core --locked` | PASS, TUI/HTTP dependency 0건 |
| `cargo tree -i crossterm --locked` | PASS, crossterm 0.29.0 단일 버전 |
| `cargo audit --no-fetch` | PASS, 1160 advisories로 214 dependencies scan |
| `cargo deny check licenses bans sources` | PASS, bans/licenses/sources 모두 ok |
| TUI `--help` | PASS, `aihack`, `--seed` 유지 |
| headless `--help` | PASS, 8개 기존 flag 유지 |
| seed 42, survival-v1, target 10 | PASS, accepted 10, hash `e7d30d72027a39c0` |
| `git diff --check` | PASS |

`cargo audit --no-fetch`는 crates.io package-cache lock warning을 출력했지만 설치된 advisory DB scan을 완료하고 exit 0이었다.

### 4.2 실패한 문서 명령

| 문서 명령 | 결과 |
| --- | --- |
| `cargo test --locked --test world_invariants` | FAIL, exit 101: default-run packages에 target 없음 |
| `cargo test --locked --test content_validation` | FAIL, exit 101: default-run packages에 target 없음 |
| `cargo test --locked --test data_loading --test items --test monster_ai --test levels` | FAIL, exit 101: default-run packages에 target 없음 |

Cargo는 각 오류에서 해당 test가 root `aihack` package에 있다고 안내했다. 동일 테스트는 `-p aihack` 또는 전체 workspace 명령으로 실행했을 때 통과했다.

## 5. Pass 1: Implementation Compliance Findings

### [IMP-F007] 완료 Task의 파일 책임 경로가 현재 workspace와 불일치

- Pass: Implementation Compliance
- Pattern: IMP-003, IMP-004, DOC-BACKFILL-001
- Area: `IMPLEMENTATION_SUMMARY.md` R2~R4 완료 Task 책임표
- Severity: Minor
- Status: **Needs Fix**
- Summary: 상단의 현재 책임표는 올바르지만, 완료된 Task 본문의 `파일` 항목은 R5 이전 monolith 경로를 현재 경로처럼 유지한다.
- Evidence:
  - `IMPLEMENTATION_SUMMARY.md:377`은 존재하지 않는 `src/data/schema.rs`를 R3-1 파일로 제시한다.
  - `IMPLEMENTATION_SUMMARY.md:425`은 존재하지 않는 `src/data/levels/main_2.toml`을 R3-3 파일로 제시한다.
  - `IMPLEMENTATION_SUMMARY.md:448`은 이동 전 `src/data/schema.rs`와 root wrapper 경로를 현재 R3-4 책임 파일로 나열한다.
  - 실제 구현은 `crates/aihack-content/src/schema.rs`, `crates/aihack-content/src/data/levels/main_2.toml`, `crates/aihack-runtime/`에 있고 R5 이동표는 이를 별도로 기록한다.
  - `audit_report_6.md`는 이전 monolith 경로가 이동 이력 표에만 남았다고 기술했지만, 현재 검색 결과는 위 활성 Task 본문에도 남아 있음을 보여준다.
- Expected: active implementation summary의 완료 Task는 현재 책임 경로를 사용하거나 `R5 이전 완료 당시 경로`라는 snapshot 표기를 가져야 한다.
- Actual: 존재하지 않거나 compatibility wrapper만 남은 과거 경로가 현재 책임 파일처럼 읽힌다.
- Impact: R6 작업자가 잘못된 모듈을 수정하거나 root facade에 runtime 구현을 다시 넣어 workspace 경계를 역행할 수 있다.
- Suggested Fix: R2~R4 `파일` 항목을 현재 crate/app owner로 갱신한다. 이력 보존이 필요하면 과거 경로와 현재 경로를 `old -> new`로 표시하고 상단 현재 책임표를 authority로 명시한다.
- Re-audit Method: active docs에서 삭제된 monolith 경로를 검색하고, 허용된 결과가 명시적 migration/history 문맥에만 존재하는지 확인한다.
- Owner: Coder, Auditor verification
- Notes: runtime 구현과 상단 현재 책임표는 정상이라 Minor로 분류한다.

### 5.2 Verified implementation evidence

- R5 workspace 8 members와 one-way dependency 경계 유지
- root compatibility facade와 app-owned production binaries 유지
- R4 accepted-turn/hash, save/replay, content/bootstrap, transaction/invariant 회귀 통과
- R6~R8은 `NOT RUN`/Open으로 명시되어 scope 과대주장 없음

## 6. Pass 2: Debug / Engineering Quality Findings

### [DBG-F002] R2/R3 canonical audit 명령이 workspace default member에서 실패

- Pass: Debug / Engineering Quality
- Pattern: BUILD-001, IMP-004, TEST-001
- Area: `audit_roadmap.md`, `spec.md`, `GAP_CLOSURE_ROADMAP.md`, `BUILD_GUIDE.md`, `IMPLEMENTATION_SUMMARY.md`, build contract tests
- Severity: Major
- Status: **Needs Fix**
- Summary: R5에서 `default-members = ["apps/aihack-tui"]`로 전환했지만 R2/R3와 과거 완료 Task의 root integration-test 명령은 `-p aihack`을 지정하지 않아 실행 즉시 실패한다. 일부 build/check 명령은 성공하더라도 default member만 선택해 전체 workspace 증거가 되지 못한다.
- Evidence:
  - `Cargo.toml:21`은 workspace default member를 `apps/aihack-tui`로 제한한다.
  - `audit_roadmap.md:187-190`, `210-212`의 R2/R3 명령은 `-p aihack` 없이 root integration test를 지정한다.
  - 위 대표 명령 3개를 그대로 실행한 결과 모두 exit 101이며 Cargo가 root `aihack` package를 명시하라고 안내했다.
  - `spec.md:51`의 SC-BUILD-01은 `cargo build --locked --all-targets`만 요구해 default member 및 그 dependency만 선택한다.
  - `GAP_CLOSURE_ROADMAP.md:93-95`의 Closed evidence와 `BUILD_GUIDE.md:141-143`의 build script 설명도 `--workspace`가 없다. 실제 `build.sh:34,38,41`은 `--workspace`를 사용한다.
  - `IMPLEMENTATION_SUMMARY.md:291,316-317,341-342,372-373,396-397,420-442,476,498-499`에도 같은 pre-workspace test 명령이 남아 있다.
  - `tests/build_contract.rs:65-74`는 audit roadmap의 metadata/headless 명령만 검사하므로 R2/R3 package selector 및 다른 활성 문서 drift를 감지하지 못한다.
- Expected: canonical audit/build 문서의 모든 명령이 현재 workspace에서 복사 실행 가능하고 주장한 package 범위를 실제로 검사해야 한다.
- Actual: R2/R3 명령은 실패하고 일부 R1/SC-BUILD-01 명령은 전체 workspace보다 좁은 범위만 검사한다.
- Impact: 독립 재감사가 중단되거나, default TUI member만 검사한 성공을 전체 workspace PASS로 오판할 수 있다. 문서 기반 Phase closure와 자동화 재현성이 손상된다.
- Suggested Fix:
  - root integration test 명령에 `-p aihack`을 추가한다.
  - 전체 workspace를 의미하는 check/test/build 명령에는 `--workspace`를 추가한다.
  - `BUILD_GUIDE.md`의 script 설명을 실제 `build.sh`/`build.bat`과 동일하게 만든다.
  - `tests/build_contract.rs`가 R1~R5 active command blocks와 `spec.md`/gap/build guide의 workspace selector를 함께 검증하도록 보강한다.
- Re-audit Method: 수정된 R1~R5 code block을 문서 그대로 실행하고, root tests가 `-p aihack`으로 통과하며 workspace build가 8 members를 대상으로 하는지 기록한다.
- Owner: Coder, Auditor verification
- Notes: 실제 full workspace 명령은 모두 통과했으므로 runtime 회귀가 아니라 audit procedure failure다.

### [DBG-F003] 생성 binary 디렉터리 `output/`이 Git ignore 정책 밖에 있음

- Pass: Debug / Engineering Quality
- Pattern: BUILD-001, DEP-001
- Area: `.gitignore`, `build.sh`, `build.bat`, generated artifacts
- Severity: Minor
- Status: **Needs Fix**
- Summary: build script가 생성하는 `output/`이 추적되지도 ignore되지도 않아 항상 untracked working-tree noise로 남는다.
- Evidence:
  - `build.sh:50-60`과 `build.bat:48-56`은 binary를 `output/`에 생성/복사한다.
  - `.gitignore:1-22`는 `/target`, `/runtime`을 제외하지만 `/output`은 제외하지 않는다.
  - `git status --short -- output .gitignore`는 `?? output/`을 반환한다.
  - 현재 `output/aihack-headless`는 38,566,560 bytes, `output/aihack`은 49,083,552 bytes다.
- Expected: 의도적으로 추적하지 않는 generated binary 디렉터리는 repository ignore 정책에 포함된다.
- Actual: 약 87.7 MB의 생성 바이너리가 매번 untracked 후보로 노출된다.
- Impact: 대용량 바이너리의 실수 커밋과 감사/리뷰 상태 잡음 가능성이 있다.
- Suggested Fix: `.gitignore`에 `/output/`을 추가하고 build contract test 또는 `git check-ignore output/aihack` 검증을 추가한다.
- Re-audit Method: build 후 `git check-ignore -v output/aihack output/aihack-headless`가 성공하고 `git status --short -- output`이 비어 있는지 확인한다.
- Owner: Coder, Auditor verification

## 7. Pass 3: Security Findings

새 Critical/Major/Minor security finding 없음.

Verified evidence:

- `cargo audit --no-fetch` exit 0, 214 dependencies scan
- cargo-deny bans/licenses/sources 모두 PASS
- 내부 path dependency 18건의 version requirement 유지
- `aihack-core`에 UI/HTTP/network dependency 없음
- 현재 shipped source에 `unsafe`, shell execution, network listener, secret/token 저장 표면 없음
- save/report runtime-root traversal 및 symlink escape 회귀 통과
- R6 network/LLM transport는 아직 dependency와 호출 경로가 없는 scaffold이며 문서도 `NOT RUN`으로 분리

## 8. Cross-Pass Conflicts

### [XPF-F006] R1~R5 PASS authority와 실행 불가능한 재감사 절차의 충돌

- Related Findings: IMP-F007, DBG-F002
- Conflict: 실제 전체 workspace 구현과 테스트는 PASS지만, active implementation/audit 문서 일부가 이전 경로와 실패하는 Cargo 명령을 제공한다.
- Resolution: R1~R5 runtime evidence와 `audit_report_6.md`의 구현 closure는 보존한다. 그러나 새 독립 감사가 동일 evidence를 문서만으로 재현할 수 없으므로 documentation/reproducibility gate는 HOLD로 둔다.
- Gate Impact: **HOLD**
- Required Fix Before PASS: DBG-F002 수정과 active Task 책임 경로 정렬 후 새 순번 재감사.

## 9. Required Fixes Before PASS

1. `audit_roadmap.md` R2/R3 root integration-test 명령에 `-p aihack`을 추가한다.
2. `spec.md`, gap roadmap, build guide, implementation summary의 전체-workspace 명령에 `--workspace`를 적용하고 실제 script와 정렬한다.
3. build contract test가 모든 active R1~R5 명령과 주요 control 문서의 package/workspace selector drift를 검출하도록 보강한다.
4. implementation summary의 완료 Task 파일 경로를 현재 owner에 맞추거나 명시적 migration snapshot으로 표시한다.
5. `/output/`을 generated artifact ignore 정책에 포함한다.
6. 수정 후 full fmt/test/clippy/release build/RustSec/cargo-deny와 문서 명령을 그대로 재실행한다.

## 10. Accepted Risks

없음.

SC-BUILD-02 remote CI pending과 R6~R8 NOT RUN은 Accepted Risk가 아니라 별도 pending/deferred gate다.

## 11. Needs Spec Clarification

없음. 현재 요구와 workspace 구조는 충분히 명확하며 finding은 문서/명령/저장소 정책의 수정 가능한 불일치다.

## 12. Re-audit Checklist

```bash
cargo fmt --all -- --check
cargo metadata --locked --no-deps --format-version 1
cargo test -p aihack --locked --test world_invariants --test transaction
cargo test -p aihack --locked --test content_validation --test content_runtime
cargo test -p aihack --locked --test data_loading --test items --test monster_ai --test levels
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo tree -p aihack-core --locked
cargo audit --no-fetch
cargo deny check licenses bans sources
git check-ignore -v output/aihack output/aihack-headless
git diff --check
```

정적 확인:

```bash
rg -n '^cargo (test|build|check) --locked' \
  spec.md IMPLEMENTATION_SUMMARY.md audit_roadmap.md GAP_CLOSURE_ROADMAP.md BUILD_GUIDE.md README.md
rg -n 'src/data/schema.rs|src/data/levels/main_2.toml|src/bin/aihack-headless.rs|src/ui/tui/' \
  IMPLEMENTATION_SUMMARY.md
```

과거 경로 검색 결과는 명시적 `old -> new` 또는 snapshot 문맥에만 존재해야 한다.

## 13. Remaining Risks

- SC-BUILD-02 Linux/Windows 원격 CI evidence pending
- R6 local LLM, R7 provenance/compatibility, R8 release NOT RUN
- 실제 terminal의 장시간 TUI UX/restore 수동 검수 미수행
- working tree는 기존 대규모 이동·미추적 파일을 포함하며 commit readiness와 change ownership은 이번 감사 범위가 아님
- 설치된 advisory DB만 사용했으므로 최신 원격 RustSec DB 상태는 별도 CI/온라인 감사에서 확인 필요
- 이번 단일 감사는 최종 release의 인간 또는 복수 모델 교차감사를 대체하지 않음

## 14. Final Decision

**HOLD — audit documentation/reproducibility remediation required**

| Gate | 판정 |
| --- | --- |
| R1 build | 실제 local full-workspace gate PASS, SC-BUILD-02 remote CI pending, 문서 명령 정렬 필요 |
| R2 state/transaction | 실제 local tests PASS, canonical audit 명령 FAIL |
| R3 content/bootstrap | 실제 local tests PASS, canonical audit 명령 FAIL |
| R4 long-run | 실제 local PASS, 완료 Task 명령/경로 정렬 필요 |
| R5 workspace | runtime/architecture evidence PASS 유지, 문서 재현성 HOLD |
| R6 local LLM | NOT RUN |
| R7 provenance/compatibility | NOT RUN |
| R8 release | NOT RUN |

코드 수정 없이 감사 보고서만 생성했다. 별도 코더가 DBG-F002, IMP-F007, DBG-F003을 수정한 뒤 `audit_report_8.md` 순번으로 독립 재감사해야 한다.
