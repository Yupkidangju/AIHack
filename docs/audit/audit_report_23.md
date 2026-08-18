# D3D 감사 리포트 23: R8/R9 통합 재감사

- 감사일: 2026-08-18T11:33:37+09:00
- 감사 유형: 활성 R8 상태, R9 콘텐츠 인과 폐쇄, 현재 작업 트리의 독립 3-pass 재감사
- 감사 기준: `AI_AUDIT_DOC_STANDARD.md`, `audit_roadmap.md`, `spec.md`
- 기준 commit: `41a1b63f11a57a671b0f705883431dab24298b5a`
- 원격 기준: `origin/main` = `41a1b63f11a57a671b0f705883431dab24298b5a`
- 환경: Windows, rustc/cargo 1.94.1
- 변경 정책: 감사 중 소스·테스트·설정·기존 구현 문서는 수정하지 않았고 본 보고서만 추가했다.

## 1. 감사 요약

최종 판정은 **HOLD**다.

핵심 런타임은 강하다. 별도 Cargo target에서 fmt, Clippy `-D warnings`, 전체 349개 테스트와 release build가 PASS했고, R9의 9개 표적 인과 테스트와 3개 seed 장기 결정론도 재현됐다. 현재 commit의 GitHub Actions run `32034295607`은 Ubuntu와 Windows quality gate 모두 success다.

그러나 다음 네 가지 Major gate가 남는다.

1. `audit_report_21.md`가 report 20 시정을 PASS로 종결했는데도 활성 문서와 문서 회귀 테스트는 아직 report 20 재감사 대기를 현재 상태로 강제한다.
2. R9 장기 테스트는 모든 필수 causal witness를 세지 않으며, 현재 `survival-v1`은 Eat, Wear, Pray를 선택하지 않아 `SC-CAUSE-05`를 증명할 수 없다.
3. Windows 실제 체크아웃에서 CRLF checksum manifest 때문에 `scripts/r7_checkpoint.sh`와 `scripts/r8_checkpoint.sh`가 구조적 FAIL 2로 종료한다. fixture 테스트는 입력을 LF로 정규화해 이 실패를 숨긴다.
4. 저장 함수는 예측 가능한 `.tmp`를 `File::create`로 열어 사전 배치된 hard link 또는 symlink를 따라갈 수 있다. 로컬 hard-link probe에서 보호 대상 파일이 실제로 truncate됐다.

따라서 개별 R9 함수 연결과 전체 테스트 PASS를 전체 프로그램 PASS 또는 외부 게시 가능 상태로 확대할 수 없다.

## 2. 감사 범위

### 2.1 확인한 문서와 계보

- `AGENTS.md`
- `AI_AUDIT_DOC_STANDARD.md`
- `AI_IMPLEMENTATION_DOC_STANDARD.md`
- `AI_CODING_STANDARD.md`
- `spec.md`, `designs.md`, `DESIGN_DECISIONS.md`
- `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`
- `README.md`, `BUILD_GUIDE.md`, `CHANGELOG.md`, `LESSONS_LEARNED.md`
- `DOCUMENTATION_AUDIT_REPORT.md`, `audit_roadmap.md`
- `audit_report_20.md`, `audit_report_21.md`, `docs/audit/audit_report_22.md`
- `PROVENANCE.md`, `docs/compatibility/**`, `docs/provenance/r7-content.sha256`
- `PROJECT_OWNER_LICENSE_APPROVAL.md`, `MODIFICATIONS.md`, `NOTICE`, `RELEASE-METADATA`

### 2.2 확인한 구현과 테스트

- `crates/aihack-core/**`, `crates/aihack-content/**`, `crates/aihack-runtime/**`
- `crates/aihack-llm/**`, `crates/aihack-ai-contract/**`
- `apps/aihack-tui/**`, `apps/aihack-headless/**`
- root와 workspace integration test 전체
- `scripts/r7_checkpoint.sh`, `scripts/r8_checkpoint.sh`, build/CI 설정
- 현재 미커밋 변경: `AGENTS.md`, `AI_AUDIT_DOC_STANDARD.md`, `AI_IMPLEMENTATION_DOC_STANDARD.md`, `tests/license_compliance.rs`, `AI_CODING_STANDARD.md`

### 2.3 프로젝트 유형과 주요 경계

- Rust workspace 기반 TUI/headless 게임
- deterministic core, embedded content registry, JSON save/replay v1
- 기본 비활성화된 loopback-only local LLM HTTP client
- NGPL whole-work 배포 및 provenance/release bundle gate

## 3. 제외 범위

- 외부 게시, 배포, tag, release 실행
- 실제 원격 LLM provider 호출
- 대화형 PTY/TUI 수동 시각 검수 재실행
- qualified legal opinion
- `legacy_nethack_port_reference/`, `.git/`, 기존 `target/`, 생성 `output/`
- 로컬에 설치되지 않은 `cargo-deny`의 신규 실행. 현재 commit의 원격 CI에서 동일 단계가 success이고 Cargo manifests/lock의 미커밋 변경은 없지만, 본 로컬 환경에서 명령은 `no such command: deny`로 실행되지 않았다.

## 4. 검증 증거

| 검증 | 결과 |
| --- | --- |
| `rustc --version`, `cargo --version` | 1.94.1 |
| R0 필수 파일·기존 SC/DEC/R0~R8 ID·archive 정적 검사 | PASS |
| 표준 금지 표현 검사 | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 349 tests |
| `cargo build --workspace --release --locked` | PASS |
| R9 `causal_content` | PASS, 9 tests |
| release `headless_policy` + `long_run` | PASS, 7 tests |
| seed 42/7/1234 release runner | 1000 accepted, 1000 submitted, Playing |
| seed 42/7/1234 hash | `c734eeafedc77c82`, `de24bb6e33a8c43f`, `c6f5e6ca9498ef35` |
| `cargo run --locked -- --help` | PASS, 단 사용자 설명은 여전히 v0.1.0 |
| 현재 SHA 원격 CI | [run 32034295607](https://github.com/Yupkidangju/AIHack/actions/runs/32034295607), Ubuntu/Windows success |
| `cargo audit` | exit 0, `RUSTSEC-2026-0253` allowed warning 1건 |
| `cargo deny check licenses bans sources` | 미실행, 로컬 subcommand 없음 |
| `git diff --check` | FAIL, `AGENTS.md:4` trailing whitespace |
| `scripts/r7_checkpoint.sh` | FAIL 2, CRLF checksum manifest |
| `scripts/r8_checkpoint.sh` | FAIL 2, R7 structural failure 전파 |
| Windows `release_bundle` test | 0 tests, 파일 전체가 `#![cfg(unix)]` |
| Windows `headless_paths` test | 1 test, symlink test는 `#[cfg(unix)]`로 제외 |

R9 이후 실제 release hash는 `audit_roadmap.md:261`의 과거 값 세 개와 다르다. ADR-0031이 의도된 semantic hash 변경 근거를 제공하지만, 활성 감사 로드맵에는 새 baseline과 causal witness evidence가 반영되지 않았다.

## 5. Pass 1: Implementation Compliance Findings

### [IMP-F016] 활성 R8 권한 문서가 report 21과 현재 commit을 반영하지 않는다 — Re-audit #3

- Pass: Implementation
- Pattern: IMP-003, IMP-004, SPEC-GAP-001
- Area: active release authority, cross-document synchronization
- Severity: **Major**
- Status: **Needs Fix / Hold**
- Summary: report 21이 report 20의 `IMP-F016`, `DBG-F008`, `XPF-F011`을 Verified/Resolved했지만 활성 문서와 테스트는 report 20 재감사 대기를 현재 상태로 유지한다.
- Evidence:
  - `audit_report_21.md:246` 이후 최종 판정은 report 20 시정 PASS다.
  - `README.md:20,44,60`, `IMPLEMENTATION_SUMMARY.md:18,861,887,922`, `GAP_CLOSURE_ROADMAP.md:55,252`, `audit_roadmap.md:378,433-434`, `BUILD_GUIDE.md:443`, `DOCUMENTATION_AUDIT_REPORT.md:208-212`, `DESIGN_DECISIONS.md:326`은 여전히 report 20 재감사 대기를 current state로 기록한다.
  - `tests/r8_documentation.rs:109-145,149-178`은 `audit_report_20.md`와 `report 20 ... 재감사 대기`를 기대값으로 강제해 후속 PASS를 stale 상태로 오인한다.
  - 현재 기준 commit `41a1b63...`은 R9 runtime 변경을 포함하고 [Actions run 32034295607](https://github.com/Yupkidangju/AIHack/actions/runs/32034295607)에서 양 OS success지만 활성 문서는 과거 `b9bd680...`/run `29886410221`만 current baseline으로 둔다.
  - `audit_roadmap.md:247,261`의 `submitted_commands=1017` 예시와 과거 hash 세 개는 현재 release runner의 1000 submitted 및 새 hash와 다르다.
- Expected: report 21이 닫은 과거 R8 remediation, R9 변경, 현재 commit/CI, 본 report 23의 새 HOLD를 시간 순서와 권한 범위에 맞게 분리해야 한다.
- Actual: 현재 상태 문서와 regression이 종결된 report 20 HOLD를 계속 현재 권한으로 취급한다.
- Impact: R0/SC-DOC-01과 R8/R9 진행 상태가 false-green이며, 다음 작업자가 이미 끝난 재감사를 반복하거나 현재 release evidence를 잘못 해석할 수 있다.
- Suggested Fix:
  1. report 21을 report 20 문서 시정의 종결 권한으로 연결한다.
  2. R9 runtime 변경 후 기준 commit, run ID, release hash를 활성 상태 절에 반영한다.
  3. report 23의 새 blocker와 외부 게시 HOLD를 이전 R8 remediation과 구분한다.
  4. `r8_documentation`을 특정 report 번호의 영구 대기가 아니라 최신 권한 상태와 stale predecessor 부재를 검사하도록 갱신한다.
- Re-audit Method: 위 활성 문서의 current-state 절·행을 대조하고 report 20 pending 문장을 되돌린 negative fixture가 실패하는지 확인한다.
- Owner: Coder / Documentation owner

### [IMP-F017] report 22가 수정 전 finding과 수정 후 PASS를 동시에 현재형으로 주장한다

- Pass: Implementation
- Pattern: IMP-003, IMP-004
- Area: R9 audit authority, accepted-risk governance
- Severity: **Major**
- Status: **Needs Documentation Recovery / Hold**
- Summary: report 22의 인과 표와 최종 판정은 R9 연결 완료를 말하지만 finding과 handoff는 같은 기능이 아직 미구현이라고 말한다.
- Evidence:
  - `docs/audit/audit_report_22.md:27-53`은 R9 항목 대부분을 `연결됨`으로 분류한다.
  - 같은 파일 `:57-95`는 speed/difficulty/passive/ac_bonus, Eat, 경제, luck과 장기 인과 검증이 현재 누락됐다고 서술한다.
  - `:108`은 PASS WITH ACCEPTED COMPATIBILITY RISK지만 `:112`는 코더에게 모든 finding을 수정하라고 요청한다.
  - `spec.md:752,776`과 `IMPLEMENTATION_SUMMARY.md:926`은 report 22를 완료 evidence와 상세 근거로 참조한다.
  - `hallucinating` risk에는 영향과 후속 선택은 있으나 owner, 만료 또는 재검토 시점이 없어 `AI_AUDIT_DOC_STANDARD.md`의 Accepted Risk 요건을 충족하지 않는다.
- Expected: 수정 전 finding은 `Initial Finding` 또는 원 ID의 재감사 계보로 표시하고, 각 항목의 현재 Status와 검증 파일을 연결하며, 남은 호환성 risk는 owner와 재검토 조건을 가져야 한다.
- Actual: 단일 active report 안에서 pre-fix와 post-fix 상태가 구분되지 않고 handoff도 최종 판정과 충돌한다.
- Impact: R9 완료 여부와 남은 작업을 보고서만으로 판정할 수 없고, 유효하지 않은 Accepted Risk가 phase gate를 우회한다.
- Suggested Fix: finding별 `Verified`/`Needs Fix` 상태와 재검증 근거를 추가하고, coder handoff는 실제 잔여 항목만 가리키며, `hallucinating` 호환성 risk에 owner와 SaveV2/feature-spec 재검토 trigger를 기록한다.
- Re-audit Method: report 22만 읽어도 각 finding의 과거/현재 상태, 남은 수정, risk owner와 재검토 조건이 하나로 결정되는지 확인한다.
- Owner: Auditor / Documentation owner / Product owner

### [IMP-F018] 사용자 CLI help가 v0.1.0 Phase 설명을 노출한다

- Pass: Implementation
- Pattern: IMP-001, IMP-004
- Area: user-facing runtime metadata
- Severity: **Minor**
- Status: **Needs Fix**
- Evidence: `apps/aihack-tui/src/main.rs:3`과 `apps/aihack-headless/src/main.rs:14`의 doc comment가 Clap help에 각각 `[v0.1.0] Phase 10`, `[v0.1.0] Phase 1`로 출력되지만 Cargo/README/CHANGELOG 현재 버전은 0.3.0이다.
- Expected: help 설명은 장기 유지 가능한 제품 설명을 사용하거나 현재 공개 버전과 일치해야 한다.
- Actual: 내부 과거 Phase 주석이 사용자 CLI 설명으로 노출된다.
- Impact: 설치된 binary의 버전과 성숙도를 잘못 전달한다.
- Suggested Fix: Clap `about`을 현재 제품 설명으로 교체하고 `--help` snapshot/contract test를 추가한다.
- Re-audit Method: 두 binary의 `--help`에서 v0.1.0/과거 Phase 문구가 0건인지 확인한다.
- Owner: Coder

## 6. Pass 2: Debug / Engineering Quality Findings

### [TEST-F001] 장기 테스트가 필수 인과 witness 전체를 증명하지 않는다 — Re-audit #1

- Pass: Debug
- Pattern: DBG-002, TEST-001
- Area: R9 causal coverage, deterministic long run
- Severity: **Major**
- Status: **Needs Fix / Hold**
- Summary: 표적 A/B 테스트는 9개 루프를 각각 증명하지만 3-seed 장기 테스트는 `SC-CAUSE-05`의 필수 witness 집합을 수집하거나 검사하지 않는다.
- Evidence:
  - `spec.md:786-788`은 각 seed 1000턴에서 모든 필수 causal witness 1회 이상과 event-only/turn-only negative gate를 요구한다.
  - `IMPLEMENTATION_SUMMARY.md:948-950`도 필수 witness와 반복 hash를 R9-6 완료 조건으로 둔다.
  - `tests/long_run.rs:9-38`은 seed/turn/event_count/last_event를 제거한 snapshot이 초기값과 다르고 nutrition이 감소했는지만 검사한다.
  - `tests/long_run.rs:43-57`은 최종 hash 반복성만 검사한다.
  - `apps/aihack-headless/src/lib.rs:193-234`의 `survival-v1` 후보에는 Quaff, Move, Pickup, Wait만 있고 Eat, Wear, Pray가 없다. 따라서 음식 섭취, armor, luck witness를 이 policy가 모두 생성할 수 없다.
  - source/test 전체에서 `witness` 또는 `SC-CAUSE`를 구현한 코드는 없으며 문서에만 존재한다.
- Expected: seed별 required witness set과 semantic delta를 명시적으로 집계하고, 각 required loop가 1회 이상 발생하며 event/turn-only 가짜 구현에서 실패해야 한다.
- Actual: 임의 이동이나 매 턴 nutrition 감소 하나만 있어도 장기 테스트가 PASS한다.
- Impact: R9 핵심 성공 기준과 report 22 PASS가 false-green이며 orphan 또는 단절된 루프의 회귀를 잡지 못한다.
- Suggested Fix:
  1. typed `CausalWitness`/summary 또는 동등한 semantic projection을 정의한다.
  2. 기존 `survival-v1`을 바꾸지 않으려면 별도 deterministic causal policy/command fixture를 추가한다.
  3. seed별 Eat/corpse, armor, monster speed/AI/passive/difficulty, gold/score, prayer/luck 등 문서상 필수 witness를 직접 assert한다.
  4. event-only, turn-only, witness 누락 fixture가 각각 실패하는 negative 회귀를 추가한다.
  5. 새 hash와 witness summary를 audit roadmap에 기록한다.
- Re-audit Method: 세 seed를 3회 실행해 witness multiset과 final hash가 반복 일치하고, 필수 witness 하나를 제거한 mutation/fixture에서 실패하는지 확인한다.
- Owner: Coder / Test owner

### [DBG-F009] Windows 실제 R7/R8 checkpoint가 CRLF manifest로 실패하지만 fixture는 PASS한다

- Pass: Debug
- Pattern: BUILD-001, TEST-001
- Area: cross-platform checkpoint reproducibility
- Severity: **Major**
- Status: **Needs Fix / Hold**
- Summary: Git Bash에서 실제 저장소 checkpoint는 FAIL하지만 테스트 fixture는 checksum manifest를 LF로 바꿔 실행해 false-green이다.
- Evidence:
  - `docs/provenance/r7-content.sha256`은 현재 Windows checkout에서 `0D 0A` 줄바꿈이다.
  - `.gitattributes:1-7`은 `LICENSE`와 `*.sh`만 LF로 고정하고 `*.sha256`은 고정하지 않는다.
  - `scripts/r7_checkpoint.sh:124-143`은 CR을 제거하지 않고 정규식, path 비교, `sha256sum --check --strict`에 전달한다.
  - 실제 실행은 각 TOML path 끝에 `$'\r'`가 붙어 파일을 찾지 못하고 `R7 CHECKPOINT: FAIL` exit 2를 반환했다.
  - `scripts/r8_checkpoint.sh`는 이를 `R7 approval checkpoint failed structurally`로 전파해 FAIL 2다.
  - `tests/provenance_manifest.rs:14-17`의 `project_file`은 모든 CRLF를 LF로 교체하고 fixture를 만들므로 실제 checkout 문제를 재현하지 않는다.
  - Windows에서는 `tests/release_bundle.rs`도 전체 `#![cfg(unix)]`로 0개 실행된다.
- Expected: 지원 OS의 실제 checkout에서 canonical checkpoint가 동작하거나, platform별 authoritative 명령과 동일 검증 범위가 명시·테스트되어야 한다.
- Actual: Linux/정규화 fixture는 green이지만 Windows 실제 mandatory checkpoint는 red다.
- Impact: Windows 로컬 감사 재현성이 깨지고 checksum/provenance 경계의 false-green 가능성이 있다.
- Suggested Fix:
  1. `docs/provenance/*.sha256 text eol=lf`를 `.gitattributes`에 추가하거나 script가 CRLF를 안전하게 정규화한 임시 입력을 사용한다.
  2. fixture 생성 시 source line ending을 무조건 정규화하지 않는 회귀를 추가한다.
  3. Windows CI에서 실제 checkout을 대상으로 R7/R8 checkpoint 또는 완전 동등한 build.bat 계약을 실행한다.
  4. platform별 release-bundle coverage 차이를 문서화하고 누락된 Windows negative case를 추가한다.
- Re-audit Method: Windows 실제 checkout에서 R7/R8 script exit 0, CRLF fixture도 예상 정책대로 처리, checksum drift fixture는 exit 2인지 확인한다.
- Owner: Coder / CI owner

### [DBG-F010] 현재 미커밋 표준·테스트 변경이 diff 품질 게이트를 통과하지 않는다

- Pass: Debug
- Pattern: TEST-001
- Area: working-tree hygiene, project convention
- Severity: **Minor**
- Status: **Needs Fix**
- Evidence:
  - `git diff --check`가 `AGENTS.md:4` trailing whitespace로 실패한다.
  - `tests/license_compliance.rs:43,64`의 새 설명 주석은 영어로 작성되어 프로젝트 기본 한국어 주석 정책과 다르다.
  - `AI_IMPLEMENTATION_DOC_STANDARD.md`는 내용 diff 없이 line-ending 상태만 modified로 보고된다.
- Expected: 의도한 content diff만 남고 whitespace/line-ending gate와 프로젝트 주석 관례를 통과해야 한다.
- Actual: 기능 테스트는 PASS하지만 version-control 품질 gate는 FAIL한다.
- Impact: R8 `git diff --check` 필수 명령을 통과할 수 없고 불필요한 line-ending churn이 생길 수 있다.
- Suggested Fix: trailing whitespace를 제거하고 주석을 한국어로 정렬하며 line-ending-only 변경을 의도한 정책에 맞게 정리한다.
- Re-audit Method: `git diff --check`, `git diff --numstat`, fmt/Clippy를 다시 실행한다.
- Owner: Coder

## 7. Pass 3: Security Findings

### [SEC-F001] 예측 가능한 save temp path가 hard-link/symlink 대상 파일을 truncate할 수 있다

- Pass: Security
- Pattern: SEC-004, BUILD-001
- Area: filesystem path boundary, atomic save
- Severity: **Major**
- Status: **Needs Fix / Hold**
- Summary: path resolver가 최종 candidate를 사전 검사해도 실제 save는 별도의 예측 가능한 `.tmp`를 일반 create로 열어 link race와 사전 배치 공격을 막지 못한다.
- Evidence:
  - `crates/aihack-runtime/src/save.rs:16-46`은 candidate와 기존 parent를 canonicalize한 뒤 path를 반환하지만 file descriptor 기반 경계는 유지하지 않는다.
  - `:78-87`은 `path.with_extension("tmp")`와 `File::create`를 사용한다.
  - `append_replay_line`도 `create(true).append(true)`로 resolved path를 다시 연다.
  - `apps/aihack-tui/src/tui/mod.rs:692-693`은 모든 실행에서 같은 OS temp 경로 `aihack-tui-save.json`을 사용한다.
  - Windows 로컬 probe에서 `save.tmp`를 `victim.txt`의 hard link로 미리 만들고 같은 `File::create` 동작을 적용하자 victim 내용이 빈 문자열로 truncate됐다.
  - 공식 Rust 문서는 [`File::create`](https://doc.rust-lang.org/std/fs/struct.File.html)가 기존 파일을 truncate한다고 설명하고, `create_new`는 사전 확인과 생성 사이 TOCTOU를 피하는 원자적 선택이라고 설명한다.
  - `tests/headless_paths.rs:17-37`의 link test는 Unix 전용이고 최종 candidate의 기존 symlink만 검사한다. Windows에서는 이 파일의 테스트가 1개만 실행됐으며 `.tmp` link case가 없다.
- Expected: root-bound path 검증과 실제 open/write가 link 교체로 분리되지 않고, temp 파일은 예측 불가하거나 `create_new`/no-follow 성격으로 생성되며 기존 save 교체도 crash-safe해야 한다.
- Actual: 공격자가 predictable `.tmp`를 link로 준비하면 root 밖 또는 다른 동일 사용자 파일을 truncate할 수 있다.
- Impact: shared temp/workspace 환경에서 로컬 임의 파일 손상과 save data loss가 가능하다.
- Suggested Fix:
  1. per-user application data directory와 run별 또는 random temp name을 사용한다.
  2. temp는 `create_new(true)`와 제한된 권한으로 생성하고 link/regular-file 상태를 handle 기준으로 확인한다.
  3. root resolver와 open/write를 하나의 안전한 API로 합쳐 검증 후 교체 race를 줄인다.
  4. destination 보존, sync, replace의 Windows/Unix 동작을 별도 회귀로 잠근다.
  5. candidate와 temp 각각의 symlink/hard-link/parent-swap negative test를 지원 OS에서 실행한다.
- Re-audit Method: 사전 배치 hard link/symlink와 검증 후 교체 fixture에서 외부 victim이 불변이고 save가 fail-closed하는지 확인한다.
- Owner: Coder / Security reviewer

### [SEC-F002] 전이 의존성 `lru 0.18.1`에 패치 가능한 RustSec unsound 경고가 있다

- Pass: Security
- Pattern: SEC-006, DEP-001
- Area: dependency audit, memory safety
- Severity: **Minor**
- Status: **Needs Fix or Time-bounded Accepted Risk**
- Summary: 최신 advisory DB에서 `RUSTSEC-2026-0253`이 탐지됐고 patched version 0.18.2가 존재한다.
- Evidence:
  - `cargo audit`는 exit 0과 함께 `lru 0.18.1`, `RUSTSEC-2026-0253`, `1 allowed warning found`를 출력했다.
  - local RustSec record는 `LruCache::pop()`에서 panicking Drop과 `catch_unwind`가 결합될 때 use-after-free/double-free가 가능하고 `>=0.18.2`에서 수정됐다고 기록한다.
  - dependency tree는 `lru -> ratatui-core -> ratatui -> aihack-tui`다.
  - inspected `ratatui-core 0.1.2`는 `LruCache<(Rect, Layout), ...>`를 사용하지만 `.pop()`이나 `catch_unwind` 호출은 관찰되지 않아 현재 shipped path의 직접 도달 가능성은 낮다.
- Expected: 패치 버전으로 lock을 갱신하거나 owner, reachability 근거, 만료일을 가진 accepted risk로 추적해야 한다.
- Actual: advisory는 release 문서에 없고 기존 `cargo audit vulnerability 0` 문구만 있다.
- Impact: 현재 exploitability는 낮지만 memory-safety advisory가 무기한 추적되지 않을 수 있다.
- Suggested Fix: `lru` 0.18.2 이상이 dependency constraints와 호환되는지 확인해 lock을 갱신하고 전체 gate를 재실행한다. 즉시 갱신하지 않으면 다음 dependency update 또는 30일 이내 재검토 조건을 기록한다.
- Re-audit Method: `cargo tree -i lru`, `cargo audit`, 전체 TUI 테스트와 release build를 재실행한다.
- Owner: Dependency owner

## 8. Cross-Pass Conflicts

### [XPF-F012] R9 완료 문서와 실제 장기 causal evidence가 충돌한다

- Related Findings: IMP-F017, TEST-F001
- Conflict: spec/summary/report 22는 R9-1..R9-6 완료를 선언하지만 장기 테스트는 필수 witness 집합을 구현하지 않는다.
- Resolution: 개별 9개 A/B causality test는 Verified로 유지하되 `SC-CAUSE-05..07`과 R9 전체는 TEST-F001 시정 전까지 HOLD한다.
- Gate Impact: R9 최종 PASS 불가.
- Required Fix Before PASS: typed witness summary, seed별 필수 coverage, negative false-green 회귀.

### [XPF-F013] 원격 양 OS green과 Windows 로컬 canonical checkpoint가 충돌한다

- Related Findings: IMP-F016, DBG-F009
- Conflict: current commit CI는 양 OS success지만 Windows 실제 Git Bash R7/R8 checkpoint는 CRLF manifest로 실패한다. Windows CI는 build.bat을 검증하고 fixture test는 LF로 정규화한다.
- Resolution: CI success는 build/test evidence로 유지하되 canonical checkpoint의 platform parity를 증명할 때까지 R7/R8 local reproducibility는 FAIL/HOLD로 둔다.
- Gate Impact: R8 final PASS 및 외부 게시 불가.
- Required Fix Before PASS: 실제 checkout line-ending 회귀와 platform authority 정렬.

## 9. 필수 수정 우선순위

1. **P0 — SEC-F001:** save/replay open 경계를 link-safe API로 바꾸고 candidate/temp race 회귀를 추가한다.
2. **P0 — TEST-F001:** R9 causal witness 집계와 seed별 필수 coverage/negative gate를 구현한다.
3. **P0 — DBG-F009:** checksum manifest line-ending 정책과 Windows 실제 checkpoint를 복구한다.
4. **P1 — IMP-F016/IMP-F017:** report 21/22/23 계보, current SHA/CI/hash, R9 risk 상태를 활성 문서와 regression에 동기화한다.
5. **P2 — SEC-F002:** `lru` patch 또는 time-bounded accepted risk를 처리한다.
6. **P2 — IMP-F018/DBG-F010:** CLI metadata, whitespace, 주석 언어와 line-ending hygiene를 정리한다.

문서 우선 정책에 따라 수정 시 `spec.md`의 보안 경계·R9 gate, `audit_roadmap.md`, 관련 ADR/BUILD_GUIDE를 먼저 또는 같은 작업 단위에서 갱신한다. 단, report 23 finding을 검증 없이 새 절대 권위로 복사하지 않고 실제 코드와 테스트에 대조한다.

## 10. Accepted Risks

- 실제 remote LLM provider smoke는 현재 spec상 비차단이다.
- qualified legal opinion은 본 기술 감사 범위 밖이다.
- `hallucinating` save v1 호환성 orphan은 후속 방향은 기록됐지만 owner/재검토 조건이 없어 아직 유효한 Accepted Risk로 인정하지 않았다.
- `RUSTSEC-2026-0253`은 현재 ratatui 호출 경로의 직접 도달 가능성이 낮지만, owner/만료가 기록되기 전에는 정식 Accepted Risk가 아니다.

## 11. Needs Spec Clarification

1. `hallucinating`을 SaveV2에서 제거할지 별도 producer feature로 승격할지 결정하는 owner와 재검토 trigger가 필요하다.
2. Windows에서 `scripts/r8_checkpoint.sh` 자체가 canonical인지, `build.bat --release`가 완전 동등한 platform authority인지 활성 BUILD/audit 문서에 명시해야 한다. 현재 문서는 전자를 필수 명령으로 적고 후자는 별도 bundle 경로로 취급한다.

## 12. 재감사 체크리스트

- [ ] SEC-F001 hard-link/symlink/temp-race 회귀가 Windows/Unix에서 fail-closed
- [ ] SC-CAUSE-05..07 typed witness summary와 seed 42/7/1234 전체 coverage PASS
- [ ] event-only/turn-only/witness 누락 negative fixture FAIL
- [ ] Windows 실제 `scripts/r7_checkpoint.sh`, `scripts/r8_checkpoint.sh` exit 0
- [ ] CRLF checksum fixture와 checksum drift negative case가 구분됨
- [ ] report 21/22/23과 current SHA/CI/hash가 활성 문서에 동기화됨
- [ ] stale report 20 pending regression이 FAIL
- [ ] report 22 finding status와 handoff가 최종 판정과 일치
- [ ] `cargo audit` warning 처리 또는 time-bounded risk 기록
- [ ] `git diff --check` PASS
- [ ] fmt, Clippy, 전체 workspace test, release build PASS
- [ ] `cargo deny check licenses bans sources` PASS
- [ ] 같은 clean commit의 Ubuntu/Windows CI와 release bundle PASS
- [ ] 외부 게시 전 별도 사용자 승인

## 13. 최종 판정

| Gate | 판정 |
| --- | --- |
| R0 Documentation | **FAIL** — IMP-F016/017 |
| R1 Build | **PASS WITH KNOWN RISK** — local build green, cargo-deny 미설치, RustSec warning 1 |
| R2 State | PASS |
| R3 Content registry | PASS |
| R4 Determinism | PASS, 단 활성 hash 문서 stale |
| R5 Workspace | PASS |
| R6 Local LLM isolation | PASS |
| R7 Provenance/Compatibility | **FAIL on current Windows checkpoint** |
| R8 Release | **HOLD** |
| R9 Causal closure | **FAIL** — TEST-F001 |
| Security | **HOLD** — SEC-F001 |
| 최종 | **HOLD** |

판단 근거: 자동 테스트와 현재 SHA 원격 CI는 제품 구현의 상당 부분을 강하게 지지한다. 그러나 문서 권한, R9 핵심 장기 acceptance, Windows provenance checkpoint, filesystem write boundary에 Major finding이 남아 있어 `AI_AUDIT_DOC_STANDARD.md`의 PASS 조건을 충족하지 못한다.

## 14. Coder Handoff

```text
`C:\LocalDev\rust\AIHack\docs\audit\audit_report_23.md`의 최신 감사 결과를 확인하고,
각 finding을 관련 프로젝트 문서와 실제 코드에 대조하여 검토한 후 필요한 수정을 수행하세요.
계약 변경이 필요한 경우 관련 문서를 먼저 갱신하고, 수정 후 관련 테스트와 검증을 실행하여 결과를 기록하세요.
우선순위는 SEC-F001, TEST-F001, DBG-F009, IMP-F016/017 순서입니다.
```
