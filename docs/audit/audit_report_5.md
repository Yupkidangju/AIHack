# AIHack D3D R5 중간감사 보고서 5

감사 기준: `AI_AUDIT_DOC_STANDARD.md`
감사 유형: R4 이후 변경 및 R5 workspace 종료 중간감사
감사 일자: 2026-07-17 (Asia/Seoul)
감사 대상: 현재 working tree
기준 commit: `49e3de8` (`main`)
환경: Linux 7.0.0-27-generic x86_64, rustc 1.94.1, cargo 1.94.1
감사 중 소스·기존 문서 수정: 없음
이번 감사가 생성한 파일: `audit_report_5.md`

## 1. 감사 요약

최종 판정: **HOLD — R5 구현은 동작하지만 중간감사 gate는 미통과**

R5의 물리적 workspace 분리, `GameClient` adapter 경계, 두 production binary, R4 결정론 회귀는 구현 및 테스트 수준에서 확인됐다. 전체 workspace test, clippy, debug/release build, 두 artifact 생성, CLI help, core dependency tree와 seed 42의 10-turn 기준 hash가 통과했다.

그러나 다음 3개 Major finding 때문에 R5를 독립 closure 또는 다음 Phase 진입 가능 상태로 판정할 수 없다.

| Finding | Pass | Severity | Status | Gate 영향 |
| --- | --- | --- | --- | --- |
| IMP-F006 | Implementation | Major | Needs Fix | R4/R5 control-document authority 충돌 |
| DBG-F001 | Debug | Major | Needs Fix | 문서에 고정된 R5 감사 명령 2개가 실행 불가 |
| SEC-F001 | Security | Major | Needs Fix | `cargo deny` 공급망 gate 실패 |

R6는 시작하지 않는다. 아래 finding을 수정한 뒤 새 순번의 재감사가 필요하다.

## 2. Audit Scope

### 2.1 프로젝트 인벤토리

- 프로젝트 경로: `/mnt/Projects_SSD/rust/AIHack`
- 유형: Rust workspace 기반 CLI/TUI 로그라이크 게임
- source: root compatibility facade `src/`, libraries `crates/`, production apps `apps/`
- tests: root `tests/`, crate/app별 `tests/`
- dependency/policy: `Cargo.toml`, 하위 package manifests, `Cargo.lock`, `deny.toml`
- CI: `.github/workflows/ci.yml`
- build/run: `build.sh`, `build.bat`, `BUILD_GUIDE.md`, `README.md`
- control docs: `spec.md`, `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`, `DESIGN_DECISIONS.md`
- 감사 계보: `audit_report_1.md`~`audit_report_4.md`

### 2.2 이번 감사의 중심 범위

- R4 accepted-turn runner와 결정론 회귀가 workspace 이동 후 유지되는지
- SC-ARCH-01과 G-ARCH-001의 구현·문서 상태
- core/content/AI contract/LLM/runtime/TUI/headless dependency 방향
- AI contract의 mutable runtime type 비노출
- TUI/headless의 `GameClient` 경계 사용
- binary 이름, CLI flag, save/replay v1 경로
- Linux/Windows build script의 전체 workspace artifact 생성
- 새 dependency graph의 RustSec 및 cargo-deny 정책
- 문서-소스 양방향 동기화

## 3. Excluded Scope

- R6 live local LLM transport, timeout, stale-response 및 soft adjudication: NOT RUN
- R7 provenance/license 법적 판단 및 NH367 compatibility: NOT RUN
- R8 release version·통합 출시 gate: NOT RUN
- Linux/Windows 원격 CI 실제 green 결과: pending
- 실제 터미널에서의 장시간 수동 TUI 플레이와 시각 검수: 비대화형 감사 환경에서 제외
- `target/`, `.git/`, `.archive/`, `.antigravitycli/`, legacy/reference corpus와 generated output
- cross-model 독립성: 이번 보고서는 구현을 수행한 동일 에이전트의 중간감사이며, 최종 출시 독립 교차감사를 대체하지 않는다.

## 4. 실행 명령과 결과

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, test annotation 240개, 실패 0 |
| `cargo build --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --locked` | PASS |
| `./build.sh --test` | PASS, `output/aihack`, `output/aihack-headless` 생성·실행권한 확인 |
| `cargo run --locked --bin aihack -- --help` | PASS, binary `aihack`, `--seed` 유지 |
| `cargo run --locked -p aihack-headless --bin aihack-headless -- --help` | PASS, 기존 headless flags 유지 |
| seed 42, survival-v1, target 10 CLI | PASS, hash `e7d30d72027a39c0` |
| `cargo tree -p aihack-core --locked` | PASS, ratatui/crossterm/HTTP client 0건 |
| `cargo metadata --locked --no-deps --format-version 1` | PASS, 8 workspace package와 두 app binary 확인 |
| `cargo metadata --workspace --locked --format-version 1` | **FAIL**, cargo metadata는 `--workspace`를 받지 않음 |
| `cargo run --locked --bin aihack-headless ...` | **FAIL 계약**, default member가 TUI뿐이므로 `-p aihack-headless` 필요 |
| `cargo audit --no-fetch` | PASS, advisory 1160건으로 Cargo.lock 214 dependencies scan, exit 0 |
| `cargo deny check licenses bans sources` | **FAIL**, internal path wildcard dependency 18건 |
| `git diff --check` | PASS |

`cargo audit --no-fetch`는 crates.io package-cache lock warning을 출력했지만 설치된 advisory DB를 사용해 scan을 완료하고 exit 0을 반환했다.

## 5. Pass 1: Implementation Compliance Findings

### 5.1 Verified evidence

- root manifest는 `autobins = false`인 compatibility facade이며 workspace default member는 `apps/aihack-tui`다.
- `apps/aihack-tui`가 `aihack`, `apps/aihack-headless`가 `aihack-headless` binary를 각각 소유한다.
- 두 app manifest는 `aihack-core`와 root facade를 직접 의존하지 않고 `aihack-runtime`과 `aihack-ai-contract`를 사용한다.
- `aihack-ai-contract`는 `Observation`, action/ID/value enum과 `ClientRevision`을 공개하지만 `GameWorld`, `GameSession`, `EntityStore`를 공개하지 않는다.
- `GameClient`의 adapter-facing mutation entry는 `submit(CommandIntent)` 하나이며 TUI는 이를 확장한 `TuiClient`, headless policy는 generic `GameClient`를 소비한다.
- `IMPLEMENTATION_SUMMARY.md:563-606`의 R5-2 구현 설명과 acceptance checkbox는 현재 source 구조와 대체로 일치한다.
- workspace boundary, TUI/headless contract, save/replay, long-run, release-candidate 회귀 테스트가 모두 통과했다.

### [IMP-F006] R4/R5 control document와 실제 구현 상태 불일치

- Pass: Implementation Compliance
- Pattern: IMP-003, IMP-004, DOC-BACKFILL-001
- Area: `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`, `DESIGN_DECISIONS.md`, `IMPLEMENTATION_SUMMARY.md`
- Severity: Major
- Status: **Needs Fix**
- Summary: R5 완료 주장과 활성 lifecycle/ADR/감사 문서가 서로 다른 상태와 파일 구조를 가리킨다.
- Evidence:
  - `IMPLEMENTATION_SUMMARY.md:563-606`은 R5-2 완료와 SC-ARCH-01 PASS를 선언한다.
  - `GAP_CLOSURE_ROADMAP.md:47-49`는 이미 구현·검증된 G-TEST-001/002와 G-ARCH-001을 모두 `Open`으로 유지하며, G-ARCH-001의 현재 증거도 `단일 Cargo package`로 남아 있다.
  - `audit_roadmap.md:284,400-401`은 R5를 `IN PROGRESS`로 두고 R4 이후 변경의 감사 필요성을 과거형으로 남긴다.
  - `DESIGN_DECISIONS.md:147-170`의 ADR-0025는 여전히 `implementation pending`이다.
  - `IMPLEMENTATION_SUMMARY.md:60-89,487`은 현재 책임표와 R4 파일 목록에서 삭제·이동된 `src/ui/tui/*`, `src/bin/aihack-headless.rs` 등을 현재 경로처럼 가리킨다.
- Expected: 구현 완료 주장, gap lifecycle, audit checkpoint, ADR status, 파일 책임표가 같은 R4/R5 상태와 실제 소유 경로를 사용해야 한다.
- Actual: 구현 요약만 완료/PASS이고 gap·audit·ADR 문서는 Open/IN PROGRESS/pending이며 일부 파일 책임표는 이전 monolith 경로다.
- Impact: 다음 작업자가 R6 진입 가능 여부와 현재 authority를 상반되게 해석하며, 독립 감사 없이 G-ARCH-001을 닫거나 반대로 완료 구현을 다시 수행할 수 있다.
- Suggested Fix:
  1. G-TEST-001/002와 G-ARCH-001의 현재 증거를 실제 test/hash/workspace evidence로 갱신하고 재감사 전 상태를 `Verified`로 정렬한다.
  2. `audit_roadmap.md`는 이번 HOLD와 올바른 R5 명령을 기록한다.
  3. ADR-0025 상태와 implementation summary의 현재 책임 경로를 실제 app/crate 소유권에 맞춘다.
  4. 새 독립 재감사에서만 관련 gap을 `Closed`로 승격한다.
- Re-audit Method: active 문서 전체의 R4/R5 상태·이전 경로 검색, 실제 manifests/source 대조, 새 report에서 lifecycle 전환 검증.
- Owner: Coder, Auditor verification

## 6. Pass 2: Debug / Engineering Quality Findings

### 6.1 Verified evidence

- full workspace tests, clippy `-D warnings`, debug/release build가 통과했다.
- 3 seeds × 1000 accepted-turn 및 seed별 3회 hash 회귀가 workspace 이동 후 통과했다.
- build scripts는 `--workspace`를 사용하며 debug build에서 두 production artifact를 fail-fast 검증한다.
- `tests/workspace_boundaries.rs`는 core UI/HTTP 무의존, app의 core 직접 의존 금지와 content physical ownership을 잠근다.
- CLI help와 10-turn headless smoke는 binary 이름·flag·결정론 hash를 보존했다.

### [DBG-F001] R5 감사 문서의 고정 명령이 현재 Cargo workspace에서 실패

- Pass: Debug / Engineering Quality
- Pattern: BUILD-001, IMP-004
- Area: `audit_roadmap.md` R5 gate
- Severity: Major
- Status: **Needs Fix**
- Summary: R5의 canonical audit command 중 metadata와 headless 명령이 현재 Cargo 구조에서 실행되지 않는다.
- Evidence:
  - `audit_roadmap.md:270`의 `cargo metadata --workspace ...`는 `unexpected argument '--workspace'`로 exit 1이다.
  - `audit_roadmap.md:274`의 headless 명령은 workspace default member가 TUI만이므로 package를 찾지 못한다.
  - 실제 통과 명령은 `cargo metadata --locked --no-deps --format-version 1` 및 `cargo run --locked -p aihack-headless --bin aihack-headless -- ...`다.
  - `spec.md`, README, BUILD_GUIDE와 implementation summary의 최신 headless 명령은 이미 package selector를 사용한다.
- Expected: audit roadmap에 복사 가능한 명령만 존재하고 같은 명령이 현재 tree에서 R5 gate를 재현해야 한다.
- Actual: 두 명령이 실행 단계에서 실패해 문서만으로는 R5를 재감사할 수 없다.
- Impact: 자동 또는 신규 감사자가 제품 실패로 오판하거나 R5 검증을 중단한다.
- Suggested Fix: metadata에서 `--workspace`를 제거하고 필요한 경우 `--no-deps`를 추가하며, 모든 headless workspace 명령에 `-p aihack-headless`를 추가한다. 해당 문자열을 build/audit contract test로 고정한다.
- Re-audit Method: 수정된 audit roadmap의 R5 code block을 그대로 실행하고 exit code와 package/binary 목록을 기록한다.
- Owner: Coder, Auditor verification

## 7. Pass 3: Security Findings

### 7.1 Verified evidence

- `cargo audit --no-fetch`는 설치된 advisory DB로 현재 214 dependencies를 scan하고 exit 0이다.
- `aihack-core` dependency tree에는 UI/HTTP dependency가 없다.
- R5 source에 새 `unsafe`, shell command 실행, network listener, HTTP client, secret/token 저장 표면이 없다.
- save/replay/report runtime-root traversal 및 symlink escape 회귀 테스트가 통과했다.
- R6 network/LLM security surface는 아직 호출 가능 구현이 아니며 후속 Phase로 명시돼 있다.

### [SEC-F001] workspace path dependency가 wildcard 금지 정책을 위반

- Pass: Security
- Pattern: SEC-006, DEP-001
- Area: workspace manifests, `deny.toml`, CI supply-chain gate
- Severity: Major
- Status: **Needs Fix**
- Summary: R5에서 추가한 내부 path dependency에 version requirement가 없어 repository의 `wildcards = "deny"` 정책과 충돌한다.
- Evidence:
  - `deny.toml:8-11`은 wildcard dependency를 deny한다.
  - root `Cargo.toml:25-31`과 각 crate/app manifest의 내부 path dependency가 `{ path = ... }`만 사용한다.
  - `cargo deny check licenses bans sources`는 root 7건을 포함해 총 18개 wildcard dependency를 보고하고 `bans FAILED`로 exit 1이다.
  - `.github/workflows/ci.yml`은 같은 cargo-deny 명령을 필수 gate로 실행한다.
- Expected: workspace 분리 후에도 R1의 license/bans/sources gate가 통과하고 CI가 dependency policy 단계에서 실패하지 않아야 한다.
- Actual: licenses와 sources는 통과하지만 bans가 새 내부 wildcard dependency 때문에 실패한다.
- Impact: 현재 tree는 로컬 R5 공급망 gate와 향후 Linux/Windows CI를 통과할 수 없다. SC-ARCH-01 구현 사실과 별개로 Phase gate PASS 선언이 불가능하다.
- Suggested Fix: 모든 내부 path dependency에 현재 package version requirement를 함께 명시한다. 예: `{ path = "...", version = "0.1.0" }`. workspace dependency 상속을 도입한다면 모든 member가 같은 canonical declaration을 사용하도록 한다. build contract test에 내부 path dependency version 정책 또는 cargo-deny 실행을 연결한다.
- Re-audit Method: 모든 manifests에서 path-only dependency 0건 확인, `cargo metadata --locked`, `cargo deny check licenses bans sources`, full workspace test/build와 CI command parity 재실행.
- Owner: Coder, Auditor verification

## 8. Cross-Pass Conflicts

### [XPF-F004] 구현 acceptance PASS와 공급망 gate 실패

- Related Findings: SEC-F001
- Conflict: SC-ARCH-01의 구조·CLI·hash 기준은 통과했지만, 같은 workspace 변경이 필수 cargo-deny gate를 실패시킨다.
- Resolution: 구조 구현은 verified evidence로 보존하되 R5 전체 checkpoint는 cargo-deny 복구 전 HOLD로 둔다.
- Gate Impact: **HOLD**
- Required Fix Before PASS: SEC-F001 수정 및 재감사.

### [XPF-F005] R5 완료 주장과 lifecycle authority 충돌

- Related Findings: IMP-F006, DBG-F001
- Conflict: implementation summary는 완료/PASS지만 gap register와 audit roadmap은 Open/IN PROGRESS이며 canonical 명령도 일부 실패한다.
- Resolution: 구현 사실을 `Verified`로 정렬하고, 수정 후 새 감사 보고서가 PASS일 때만 `Closed`로 승격한다.
- Gate Impact: **HOLD**
- Required Fix Before PASS: IMP-F006, DBG-F001 수정 및 재감사.

## 9. Required Fixes Before PASS

1. 모든 workspace 내부 path dependency에 version requirement를 추가해 cargo-deny bans gate를 복구한다.
2. R5 audit roadmap의 metadata/headless 명령을 현재 Cargo package selection 규칙에 맞춘다.
3. R4/R5 gap, audit checkpoint, ADR 상태와 현재 파일 책임표를 실제 구현에 동기화한다.
4. 관련 contract test를 보강하고 full fmt/clippy/test/release build/cargo-audit/cargo-deny를 다시 실행한다.
5. 새 순번 재감사에서 finding 3개와 cross-pass conflict 2개를 해소한 뒤 R5 closure를 판정한다.

## 10. Accepted Risks

없음.

SC-BUILD-02 원격 CI pending과 R6~R8 NOT RUN은 Accepted Risk가 아니라 별도 pending/deferred gate다.

## 11. Needs Spec Clarification

없음. 요구사항은 충분히 명확하며 발견된 문제는 구현·문서·정책의 수정 가능한 불일치다.

## 12. Re-audit Checklist

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo metadata --locked --no-deps --format-version 1
cargo tree -p aihack-core --locked
cargo audit --no-fetch
cargo deny check licenses bans sources
cargo run --locked --bin aihack -- --help
cargo run --locked -p aihack-headless --bin aihack-headless -- --help
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 10 --policy survival-v1
git diff --check
```

정적 확인:

```bash
rg -n 'aihack_core|aihack-core' apps
rg -n 'R5 IN PROGRESS|G-ARCH-001.*Open|implementation pending' \
  GAP_CLOSURE_ROADMAP.md audit_roadmap.md DESIGN_DECISIONS.md
rg -n 'src/bin/aihack-headless.rs|src/ui/tui/\*' IMPLEMENTATION_SUMMARY.md
```

기대 결과는 app의 core 직접 의존 0건, path-only workspace dependency 0건, stale R5 상태·현재 경로 오표기 0건이다. 이동 이력을 설명하는 `old -> new` 표는 허용한다.

## 13. Remaining Risks

- SC-BUILD-02 Linux/Windows 원격 CI evidence는 pending이다.
- R6 live LLM, R7 provenance/compatibility, R8 release는 NOT RUN이다.
- 실제 terminal restore와 장시간 TUI UX는 이번 비대화형 감사에서 수동 검증하지 않았다.
- working tree는 감사 전부터 많은 수정·이동·미추적 파일을 포함한다. 본 보고서는 현재 tree의 내용과 실행 결과를 감사했으며 commit readiness나 변경 소유권을 판정하지 않는다.
- 동일 에이전트가 구현과 이번 감사를 수행했으므로 최종 release 단계에서는 별도 모델 또는 인간의 독립 교차감사가 필요하다.

## 14. Final Decision

**HOLD — R5 중간감사 미통과**

R5의 핵심 구조 목표는 구현됐고 full workspace behavior regression도 통과했다. 그러나 `cargo deny` 필수 gate 실패, 실행 불가능한 canonical audit 명령, 상충하는 lifecycle 문서가 남아 있어 현재 상태를 R5 PASS 또는 G-ARCH-001 Closed로 선언할 수 없다.

| Gate | 판정 |
| --- | --- |
| R1 build | 기존 LOCAL PASS에서 **cargo-deny regression 발생**, remote CI pending |
| R2 state/transaction | LOCAL PASS 유지 |
| R3 content/bootstrap | LOCAL PASS 유지 |
| R4 long-run | 구현·결정론 tests PASS, lifecycle 문서 정렬 필요 |
| R5 workspace | 구현 evidence PASS, **중간감사 HOLD** |
| R6 local LLM | NOT RUN |
| R7 provenance/compatibility | NOT RUN |
| R8 release | NOT RUN |

R6를 시작하기 전에 IMP-F006, DBG-F001, SEC-F001을 수정하고 새 순번 재감사를 수행해야 한다.
