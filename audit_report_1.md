# D3D Audit Report 1

감사일: 2026-07-16 (Asia/Seoul)

감사 기준: `AI_AUDIT_DOC_STANDARD.md`, `AI_IMPLEMENTATION_DOC_STANDARD.md`

감사 범위: 현재 working tree의 R1~R3 구현, 활성 문서 세트, 빌드·테스트·의존성 게이트
코드 수정: 없음

## 1. Audit Scope

| 항목 | 확인 대상 |
| --- | --- |
| 문서 authority | `spec.md`, `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `designs.md`, `DESIGN_DECISIONS.md`, `BUILD_GUIDE.md`, `audit_roadmap.md`, `README.md`, `CHANGELOG.md`, `DOCUMENTATION_AUDIT_REPORT.md` |
| 구현 | `src/core`, `src/data`, `src/domain`, `src/ui`, `src/llm` |
| 테스트 | `src` unit test 및 `tests/` integration test 207개 |
| 설정·공급망 | `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `deny.toml`, `.github/workflows/ci.yml` |
| R1~R3 증거 | build contract, transaction/invariant, content validation/runtime test |

### 실행 증거

새 target directory (`CARGO_TARGET_DIR=/tmp/aihack-audit-target-20260716`)에서 아래 명령은 모두 exit 0이었다.

```bash
rustc --version                 # 1.94.1
cargo --version                 # 1.94.1
cargo metadata --locked --no-deps --format-version 1
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo audit
cargo deny check licenses bans sources
git diff --check
```

`cargo test`는 207개 `#[test]` 선언을 실행해 통과했다. R0 문서 게이트의 필수 파일, ID closure, 금지 표현, archive 존재 검사도 통과했다.

## 2. Excluded Scope

- GitHub Actions의 실제 Linux/Windows 원격 실행 결과: 로컬 tree에는 workflow만 있으며 이 감사에서 원격 CI를 실행하지 않았다.
- R4~R8 구현: 계획과 checkpoint 상태만 대조했으며, 미구현 local LLM transport·workspace 분리·provenance 승인·NetHack compatibility는 완료 여부를 주장하지 않는다.
- `legacy_nethack_port_reference/`: reference-only tree이며 active runtime 구현 감사에서 제외했다.
- `target/`, `.git/`, `.archive/`의 과거 snapshot 내용: 생성물 또는 불변 이력으로서 현재 코드의 source of truth가 아니다.

## 3. Pass 1: Implementation Compliance Findings

## [IMP-F001] 활성 사용자 문서가 R1~R3 완료 상태를 부정함

- Pass: Implementation Compliance
- Pattern: DOC-BACKFILL-001
- Area: `README.md`, `BUILD_GUIDE.md`, `IMPLEMENTATION_SUMMARY.md`
- Severity: Major
- Status: Verified (Re-audit #1, 2026-07-16)
- Summary: R1~R3는 구현·로컬 검증 완료로 표시되지만, 활성 사용자 문서와 다음 작업 지시가 구현 전 상태를 계속 설명한다.
- Evidence:
  - `README.md:13,72`는 public mutable state와 hardcoded runtime content가 남았다고 한다. 실제로 R2 checkpoint는 private state 완료(`IMPLEMENTATION_SUMMARY.md:356-359`)이고 R3 checkpoint는 registry runtime 연결 완료(`IMPLEMENTATION_SUMMARY.md:361-435`)다.
  - `README.md:39-41`은 R1→R2→R3를 다음 구현 순서로 제시한다.
  - `BUILD_GUIDE.md:90,132`는 `rust-toolchain.toml` 생성과 lockfile 갱신을 미래 R1 작업으로 설명하지만 해당 파일과 R1 local gate는 현재 존재·통과한다.
  - `IMPLEMENTATION_SUMMARY.md:761`는 다음 세션을 R1-1에서 시작하라고 지시한다.
- Expected: active 문서는 R1~R3의 local PASS와 R4가 다음 구현 단계임을 일관되게 설명해야 한다.
- Actual: source/test 증거와 문서의 현재 상태·작업 순서가 모순된다.
- Impact: 사용자와 다음 구현 AI가 이미 완료한 state/content 작업을 재수행하거나, 현재 API·위험을 잘못 판단할 수 있다.
- Suggested Fix: README의 다국어 Current status와 순서를 R1~R3 완료/R4 다음 단계로 갱신하고, BUILD_GUIDE의 R1 설명을 완료 증거와 historical note로 전환한다. `IMPLEMENTATION_SUMMARY.md`의 구현 시작 순서는 R4-1로 바꾼다.
- Re-audit Method: `rg -n -i 'public mutable|hardcoded runtime|Task R1-1|R1-1에서 생성|R1 구현 세션' README.md BUILD_GUIDE.md IMPLEMENTATION_SUMMARY.md` 결과가 historical archive 외 활성 현재 상태 문맥에서 0건인지 확인한다.
- Re-audit #1 Evidence: README의 다국어 current status와 구현 순서는 R1/R2 local PASS 및 R3-4를 반영하고, BUILD_GUIDE의 R1 절은 적용 결과로 전환됐으며, IMPLEMENTATION_SUMMARY의 다음 작업은 R3-4로 갱신됐다.
- Owner: Coder / Architect

## [IMP-F002] 최상위 문서 감사 결과가 현재 구현 범위와 충돌함

- Pass: Implementation Compliance
- Pattern: DOC-BACKFILL-001
- Area: `DOCUMENTATION_AUDIT_REPORT.md`, `audit_roadmap.md`
- Severity: Major
- Status: Verified (Re-audit #1, 2026-07-16)
- Summary: 현재 README가 링크하는 감사 결과는 코드 구현을 명시적으로 제외한 R0 전용 리포트인데, 활성 audit roadmap도 전체 구현을 `NOT RUN`으로 표시한다.
- Evidence:
  - `DOCUMENTATION_AUDIT_REPORT.md:3-12,118`은 감사 범위를 계획 문서만으로 제한하고 R1~R8 미구현 및 다음 R1-1 시작을 주장한다.
  - `audit_roadmap.md:202,223`은 R2/R3 local PASS를 기록한다.
  - `audit_roadmap.md:400`은 세분화 없이 "현재 전체 구현 판정: NOT RUN"이라고 기록한다.
- Expected: 과거 R0 계획 감사는 historical record로 보존하되, 현재 상태를 대표하는 문서는 R1~R3의 검증 범위와 R4~R8의 미완료 범위를 분리해야 한다.
- Actual: 현재 README의 Documentation audit 링크가 현재 구현 상태와 반대되는 리포트로 연결된다.
- Impact: SC-DOC-01과 R1~R3의 실증 상태를 잘못 해석하고, R8 final audit와 구분할 수 없다.
- Suggested Fix: 기존 리포트를 R0 계획 감사로 명시적으로 보존하고, 이 리포트를 최신 audit index에 추가한다. `audit_roadmap.md`의 전체 상태는 "R1~R3 local PASS; R4~R8 NOT RUN"으로 세분화한다.
- Re-audit Method: README에서 최신 감사 리포트를 링크하고, R0/R1~R3/R4~R8의 상태가 단일 표에서 모순 없이 표시되는지 확인한다.
- Re-audit #1 Evidence: README는 R0 계획 감사와 current audit을 분리해 링크하고, 기존 R0 리포트에는 historical scope notice를 추가했다. audit roadmap은 R1/R2 local PASS, R3 HOLD, R4~R8 NOT RUN으로 세분화됐다.
- Owner: Architect / Coder

## [IMP-F003] ContentRegistry의 공개·실패 계약이 master spec과 다름

- Pass: Implementation Compliance
- Pattern: IMP-002
- Area: `spec.md`, `src/data/schema.rs`, `src/domain/item.rs`, `src/domain/level.rs`, `src/core/world.rs`
- Severity: Major
- Status: Needs Fix
- Summary: spec은 typed `LevelId` error와 다섯 read-only query만 공개하고 content 오류는 시작 실패로 반환한다고 정의하지만, 구현은 String ID와 추가 public constructor/iterator를 노출하며 production bootstrap 경로에서 `expect`로 panic한다.
- Evidence:
  - `spec.md:279-298`은 `BTreeMap<...Id, ...Definition>`, `InvalidCoordinate { level: LevelId }`, `MissingStairsPair { level: LevelId }`, 그리고 다섯 query만 public이라고 정의한다.
  - `src/data/schema.rs:100-177`은 `BTreeMap<String, ...>`, public `from_toml_sources`, `items`, `monsters`, `levels` iterator를 노출한다.
  - `src/core/world.rs:62-81`, `src/domain/level.rs:26-30`, `src/domain/item.rs:74-75`는 registry/변환 오류를 `expect`로 panic 처리한다.
- Expected: (a) spec을 실제 의도된 public testing/import boundary에 맞춰 갱신하거나, (b) 구현을 spec의 typed read-only boundary와 fallible session/world bootstrap으로 맞춘다. 어느 쪽이든 invalid embedded content가 process panic이 아닌 typed startup error가 되어야 한다.
- Actual: R3 테스트는 `ContentRegistry::from_toml_sources`에서 typed validation을 증명하지만, 실제 `GameSession::new_for_playing` 경로의 registry failure는 typed result가 아니다.
- Impact: SC-DATA-01과 G-DATA-002의 "panic fallback 없음" 완료 주장을 완전히 뒷받침하지 못하며, public API authority가 불명확하다.
- Suggested Fix: `try_new_for_playing`/`try_fixture_phase5` 같은 fallible bootstrap을 도입하고 CLI가 `ContentError`를 사용자 오류로 표시하게 한다. test-only source injection은 `#[cfg(test)]` 또는 명시적인 test-support API로 축소한다. ID newtype 전환을 지금 하지 않을 경우에는 spec에 String ID 선택과 공개 iterator 이유를 기록한다.
- Re-audit Method: malformed embedded-source startup regression이 `Err(ContentError)`를 반환하고 `catch_unwind` 없이 통과하는지 테스트한다. public API snapshot 또는 compile-fail test로 permitted methods를 고정한다.
- Owner: Coder / Architect

## 4. Pass 2: Debug / Engineering Quality Findings

## [DBG-F001] R3의 data-only panic scan은 통과하지만 runtime bootstrap panic을 놓침

- Pass: Debug / Engineering Quality
- Pattern: TEST-001
- Area: `audit_roadmap.md`, R3 runtime construction
- Severity: Minor
- Status: Needs Fix
- Summary: R3 checkpoint는 invalid content test에서 panic 0건을 요구하지만, 현재 검증 명령은 data constructor만 exercise한다.
- Evidence: `audit_roadmap.md:217-223`의 R3 PASS 조건과 `src/core/world.rs:62-81`의 panic adapter가 공존한다.
- Expected: checkpoint test는 registry validation뿐 아니라 game bootstrap boundary도 cover해야 한다.
- Actual: `tests/content_validation.rs`는 injected TOML validation을 검사하나, session creation의 fallible error contract가 없다.
- Impact: malformed future embedded content가 startup panic으로 바뀌어도 현재 R3 test suite가 잡지 못한다.
- Suggested Fix: IMP-F003의 fallible bootstrap과 malformed-content fixture test를 R3 gate에 추가한다.
- Re-audit Method: R3 gate에 bootstrap error test를 포함해 실행한다.
- Owner: Coder

### Verified engineering evidence

- Formatting, Clippy `-D warnings`, 207 tests, debug/release build, `git diff --check`가 통과했다.
- `cargo audit`는 207 lockfile dependency를 스캔했고 취약점 보고 없이 종료했다.
- `cargo deny check licenses bans sources`는 `bans ok, licenses ok, sources ok`였다.
- R1 toolchain/config/CI files는 존재한다.

## 5. Pass 3: Security Findings

Critical 또는 Major security finding은 확인되지 않았다.

- embedded content는 `include_str!`로 compile-time 포함되며 runtime arbitrary path load가 없다.
- local LLM live transport는 아직 R6 scope라서 외부 endpoint exposure는 관찰되지 않았다.
- RustSec advisory, dependency ban/license/source gate는 로컬에서 통과했다.

보안 검토 제한: R6의 실제 HTTP transport, R7 provenance 승인, GitHub Actions 원격 실행은 아직 scope 밖 또는 미완료다. 이들은 security PASS가 아니라 후속 checkpoint 조건이다.

## 6. Cross-Pass Conflicts

| ID | 관련 finding | 충돌 |
| --- | --- | --- |
| XPF-F001 | IMP-F001, IMP-F002 | 코드/테스트는 R1~R3 완료 증거를 제공하지만 active user/audit 문서는 이를 구현 전 또는 전체 NOT RUN으로 나타낸다. |
| XPF-F002 | IMP-F003, DBG-F001 | 문서는 ContentError/panic-free startup을 완료로 표현하지만 runtime bootstrap에는 `expect`가 남아 있다. |

## 7. Required Fixes Before PASS

1. IMP-F003: ContentRegistry의 API/typed-ID 문서를 구현과 맞추거나 구현을 spec에 맞추고, bootstrap의 panic fallback을 제거한다.

## 8. Accepted Risks

없음. R4~R8은 명시적으로 미구현 Phase이며 accepted risk가 아니다.

## 9. Needs Spec Clarification

없음. ContentRegistry의 public surface와 startup error behavior는 `spec.md`에 이미 명시되어 있어 구현 또는 문서 중 하나를 정렬하면 된다.

## 10. Re-audit Checklist

```bash
rg -n -i 'public mutable|hardcoded runtime|Task R1-1|R1-1에서 생성|R1 구현 세션' \
  README.md BUILD_GUIDE.md IMPLEMENTATION_SUMMARY.md
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo audit
cargo deny check licenses bans sources
git diff --check
```

- README와 audit index가 이 리포트를 최신 결과로 링크하는지 확인한다.
- malformed embedded content가 `GameSession` bootstrap에서 `ContentError`가 되는 regression test를 확인한다.
- `ContentRegistry` public API와 `spec.md` 9.3 계약이 type/signature 단위로 같은지 확인한다.

## 11. Final Decision

**HOLD (Re-audit #1: documentation synchronization complete)**

R1~R3의 build·test·supply-chain 로컬 증거는 강하다. IMP-F001과 IMP-F002의 문서 동기화는 Re-audit #1에서 해소됐다. 그러나 ContentRegistry startup/public-contract 불일치(IMP-F003)가 남아 있어 전체 PASS는 아니다. 다음 구현은 `IMPLEMENTATION_SUMMARY.md`의 R3-4이며, 완료 후 이 리포트의 finding ID를 유지하여 재감사한다.
