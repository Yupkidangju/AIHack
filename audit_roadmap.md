# AIHack Audit Roadmap v2

> Archive chain
> - Latest: `.archive/audit_roadmap_archive_260715.md`
> - Previous: first archive
>
> 과거 Phase 0~20 감사 이력은 아카이브에 있다. 이 문서는 v0.3.0 리팩터링의 활성 검증 게이트만 정의한다.

문서 상태: active verification contract
작성일: 2026-07-15
목표 버전: 0.3.0
기준: `spec.md`, `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `AI_IMPLEMENTATION_DOC_STANDARD.md`

## 1. 판정 규칙

### 1.1 상태

| 상태 | 의미 |
| --- | --- |
| NOT RUN | 구현 또는 검증을 시작하지 않음 |
| FAIL | 필수 명령, 수동 시나리오, 산출물 중 하나 이상 실패 |
| BLOCKED | 외부 승인·환경 때문에 검증 불가하며 우회 증거도 없음 |
| PASS | 이 문서에 명시한 자동·수동 증거가 모두 충족 |
| PASS WITH KNOWN RISKS | 선택 게이트만 남고 필수 성공 기준은 모두 충족 |

### 1.2 공통 원칙

- 구현 존재 여부가 아니라 재현 가능한 명령과 산출물로 판정한다.
- 한 필수 항목이 FAIL이면 해당 checkpoint 전체는 FAIL이다.
- baseline hash는 예상값 불일치를 숨기기 위해 갱신하지 않는다.
- `--locked` 없는 Cargo 결과는 릴리즈 증거로 사용하지 않는다.
- 조기 `GameOver`는 1000 accepted-turn 성공이 아니다.
- LLM 응답 내용은 core 정합성 증거가 아니다.
- `Unknown` 또는 `Blocked` provenance 자산은 런타임 포함 즉시 FAIL이다.
- 문서가 target 구현을 현재 완료 상태로 표현하면 R0 FAIL이다.

## 2. 감사 환경과 증거 보존

검증 루트는 repository root다. clean target을 사용하여 기존 산출물에 의한 오판을 막는다.

```bash
export CARGO_TARGET_DIR=/tmp/aihack-audit-target
rustc --version
cargo --version
git status --short
```

필수 기록:

- 실행 일시와 OS
- `rustc --version`, `cargo --version`
- 명령, exit code, 실패한 test 이름
- long-run seed별 accepted/submitted/final hash
- LLM mock 요청의 request ID, revision, status
- provenance report와 compatibility report
- 최종 Git diff의 변경 파일 목록

`/tmp/aihack-audit-target`은 검증 캐시이며 저장소 산출물이 아니다.

## 3. R0 문서 구현 가능성 게이트

현재 판정 대상은 문서만이다. 이 게이트 PASS는 R1~R8 코드 구현 완료를 뜻하지 않는다.

### 3.1 필수 파일

```bash
test -s spec.md
test -s IMPLEMENTATION_SUMMARY.md
test -s GAP_CLOSURE_ROADMAP.md
test -s designs.md
test -s DESIGN_DECISIONS.md
test -s BUILD_GUIDE.md
test -s audit_roadmap.md
test -s CHANGELOG.md
test -s PROVENANCE.md
test -s docs/compatibility/README.md
test -s DOCUMENTATION_AUDIT_REPORT.md
```

### 3.2 계약 ID 폐쇄성

다음 ID는 정의 문서와 실행 계획·감사 문서에 모두 나타나야 한다.

```bash
for id in \
  SC-BUILD-01 SC-BUILD-02 SC-CORE-01 SC-CORE-02 SC-DATA-01 \
  SC-TEST-01 SC-TEST-02 SC-ARCH-01 SC-LLM-01 SC-LLM-02 SC-LLM-03 \
  SC-COMPAT-01 SC-LICENSE-01 SC-DOC-01; do
  rg -q "$id" spec.md
  rg -q "$id" GAP_CLOSURE_ROADMAP.md audit_roadmap.md
done

for id in DEC-PRODUCT-01 DEC-RUST-01 DEC-STATE-01 DEC-LLM-01 \
  DEC-LLM-02 DEC-CONTENT-01 DEC-WORKSPACE-01 DEC-LICENSE-01; do
  rg -q "$id" spec.md
  rg -q "$id" DESIGN_DECISIONS.md
done

for id in R0-1 R0-2 R0-3 R1-1 R1-2 R1-3 R2-1 R2-2 R2-3 \
  R3-1 R3-2 R3-3 R4-1 R4-2 R5-1 R5-2 R6-1 R6-2 R6-3 R7-1 R7-2 R8-1; do
  rg -q "$id" IMPLEMENTATION_SUMMARY.md
  rg -q "$id" GAP_CLOSURE_ROADMAP.md audit_roadmap.md
done
```

### 3.3 표준 금지 표현과 링크

활성 계획 문서에서 아래 표현은 구체적 수치·주체·파일로 교체한다.

```bash
bad_terms='적당''히|필요''시|원하''면|추후'' 고려|게임답''게|자연스럽''게|유연하''게|대충'' 이 정도|적절''히|알''아서|충분''히|일반적인'' 방식|적당한'' 값|나중에'' 결정'
! rg -n "$bad_terms" \
  spec.md IMPLEMENTATION_SUMMARY.md GAP_CLOSURE_ROADMAP.md designs.md \
  DESIGN_DECISIONS.md BUILD_GUIDE.md audit_roadmap.md PROVENANCE.md \
  README.md docs/compatibility/README.md
```

상대 경로로 참조한 필수 문서와 archive가 실제로 존재해야 한다.

```bash
test -s .archive/spec_archive_260715.md
test -s .archive/IMPLEMENTATION_SUMMARY_archive_260715.md
test -s .archive/GAP_CLOSURE_ROADMAP_archive_260715.md
test -s .archive/audit_roadmap_archive_260715.md
test -s .archive/designs_archive_260715.md
test -s .archive/BUILD_GUIDE_archive_260715.md
test -s .archive/DESIGN_DECISIONS_archive_260715.md
```

### 3.4 AI 구현 문서 표준 12항목

| # | 검사 | PASS 증거 |
| --- | --- | --- |
| 1 | 목표·성공·비목표 폐쇄 | `spec.md` 2~4절 |
| 2 | 현재와 target 분리 | 모든 활성 문서의 상태 표 |
| 3 | architecture와 의존 방향 | `spec.md` 6절, `designs.md` 3절 |
| 4 | typed contract | `spec.md` 9절 |
| 5 | concrete number/default | `spec.md` 10~12절 |
| 6 | real data sample | `spec.md` 11절, compatibility template |
| 7 | error/degraded behavior | `spec.md` 9.5, `designs.md` 7절 |
| 8 | task file/dependency/acceptance | `IMPLEMENTATION_SUMMARY.md` 7절 |
| 9 | command/artifact path | `BUILD_GUIDE.md`, 이 문서 R1~R8 |
| 10 | decision/alternative/consequence | ADR-0021~ADR-0027 |
| 11 | cross-document ID closure | R0 3.2 명령 |
| 12 | final completion gate | R8-1, `DOCUMENTATION_AUDIT_REPORT.md` |

R0는 위 12항목과 3.1~3.3 명령이 모두 통과할 때만 PASS다.

## 4. R1 재현 빌드 게이트

연결 gap: G-BUILD-001..004, G-RUN-001
성공 기준: SC-BUILD-01, SC-BUILD-02

```bash
test "$(rustc --version | awk '{print $2}')" = "1.94.1"
cargo metadata --locked --no-deps --format-version 1
cargo tree -d
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo audit
cargo deny check licenses bans sources
# default-run이 TUI binary를 선택하는지 확인한다. TUI event loop은 CI에서 열지 않는다.
cargo run --locked -- --help
./build.sh --test
test -x output/aihack
test -x output/aihack-headless
```

추가 판정:

- `Cargo.lock`은 검증 전후 byte-identical이다.
- `cargo tree -d`에 crossterm이 한 버전만 있다.
- Linux와 Windows CI가 동일 commit에서 green이다.
- `build.sh`와 `build.bat`은 artifact 누락을 exit code 0으로 숨기지 않는다.

Checkpoint R1 현재 상태: SC-BUILD-01 local PASS; SC-BUILD-02 remote CI pending.

## 5. R2 상태 무결성 게이트

연결 gap: G-CORE-001..003
성공 기준: SC-CORE-01, SC-CORE-02

```bash
cargo test -p aihack --locked --test world_invariants
cargo test -p aihack --locked --test transaction
cargo test -p aihack --locked --test save_load --test replay
cargo test -p aihack --locked --test golden_phase8_rules
! rg -n "session\.(meta|rng|turn|state|world|event_log)\s*=" src tests
! rg -n "world\.(levels|current_level|entities|inventory|status)\s*=" src/ui src/llm tests
```

PASS 조건:

- accepted turn당 검사한 invariant 수는 6이다.
- reject와 invariant error에서 turn, RNG draw, snapshot hash가 모두 불변이다.
- transaction 전후 시스템 순서가 player, tile/item, monster, status, death, commit 순서다.
- 기존 P8-G01..P8-G20 결과가 바뀌지 않는다.

Checkpoint R2 현재 상태: local command PASS (2026-07-15). `world/levels/entities/inventory`는 crate 외부 비공개이며, integration test는 읽기 accessor 및 저장 기반 fixture 경계만 사용한다.

## 6. R3 콘텐츠 레지스트리 게이트

연결 gap: G-DATA-001..002
성공 기준: SC-DATA-01

```bash
cargo test -p aihack --locked --test content_validation
cargo test -p aihack --locked --test content_runtime
cargo test -p aihack --locked --test data_loading --test items --test monster_ai --test levels
```

PASS 조건:

- embedded TOML은 process 시작 시 1회 파싱된다.
- duplicate ID, unknown reference, invalid dice, out-of-bounds position, unpaired stairs, unsupported schema가 typed error로 반환된다.
- dagger, jackal, main:1, main:2가 registry 정의로 생성된다.
- 동일 embedded content hash를 3회 생성했을 때 같은 16자리 lowercase hex다.
- invalid content 테스트에서 panic 0건이다.

Checkpoint R3 현재 상태: LOCAL PASS (2026-07-16). `ContentRegistry`의 OnceLock parse·validation, runtime item/monster/level factory 및 main:1/main:2 초기 배치는 local test를 통과했다. TUI/headless는 fallible `GameSession` bootstrap을 사용하며, injected malformed content와 누락된 시작 아이템은 `ContentError`로 반환하는 regression test로 고정했다.

## 7. R4 장기 결정론 게이트

연결 gap: G-TEST-001..002
성공 기준: SC-TEST-01, SC-TEST-02

```bash
cargo test -p aihack --locked --release --test headless_policy
cargo test -p aihack --locked --release --test long_run
cargo test -p aihack --locked --test save_load --test replay
for seed in 42 7 1234; do
  cargo run --locked --release -p aihack-headless --bin aihack-headless -- \
    --seed "$seed" --turns 1000 --policy survival-v1
done
```

각 run의 필수 report:

```json
{
  "policy": "survival-v1",
  "requested_turns": 1000,
  "accepted_turns": 1000,
  "submitted_commands": 1017,
  "final_state": "Playing",
  "final_hash": "16-lowercase-hex"
}
```

PASS 조건:

- seed 42, 7, 1234 각각 accepted_turns가 정확히 1000이다.
- 각 성공 report는 `accepted_turns <= submitted_commands <= accepted_turns * 16`을 만족한다.
- 각 seed의 동일 command sequence 3회 hash가 일치한다.
- policy가 한 turn에 16개 legal candidate를 모두 거부당하면 성공이 아니라 `NoAcceptedAction`이다.
- save/load continuation hash가 direct run hash와 같다.

Checkpoint R4 현재 상태: LOCAL PASS. `tests/long_run.rs`는 seed 42, 7, 1234 각각의 1000 accepted turn과 seed별 3회 hash 일치를 검증한다. release runner report hash는 각각 `7dc03ca706e350df`, `360a7c07904c78e2`, `0c73bd75ff8cd540`이다.

## 8. R5 workspace 경계 게이트

연결 gap: G-ARCH-001

성공 기준: SC-ARCH-01

```bash
cargo metadata --locked --no-deps --format-version 1
cargo tree -p aihack-core
cargo test --workspace --all-targets --locked
cargo run --locked --bin aihack -- --seed 42
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

PASS 조건:

- `aihack-core` dependency tree에 ratatui, crossterm, HTTP client가 없다.
- `aihack-ai-contract`은 mutable core type을 export하지 않는다.
- TUI/headless binary 이름, CLI flag, save/replay v1 경로가 유지된다.
- R4의 command sequence와 hash가 workspace 이동 전후 동일하다.

Checkpoint R5 현재 상태: **PASS**. core/content/AI contract/LLM/runtime/TUI/headless 분리, root compatibility facade, binary CLI, R4 hash, cargo-deny와 문서 시정을 `audit_report_9.md`가 재검증했다. 다음 구현 단계는 R6다.

## 9. R6 local LLM 격리 게이트

연결 gap: G-LLM-001..004
성공 기준: SC-LLM-01, SC-LLM-02, SC-LLM-03

```bash
cargo test -p aihack --locked --test llm_transport
cargo test -p aihack --locked --test llm_narrative
cargo test -p aihack --locked --test llm_decision_support
cargo test -p aihack --locked --test llm_revision_gate
cargo test -p aihack --locked --test llm_soft_adjudication
```

필수 failure matrix:

| case | 기대 status | core hash |
| --- | --- | --- |
| disabled | Disabled | 불변 |
| queue 16개 사용 중 | Busy | 불변 |
| endpoint가 non-loopback으로 resolve | Invalid | 불변 |
| request JSON 32,768 bytes 초과 | Invalid | 불변 |
| connection refused | Unavailable | 불변 |
| connect 500ms 초과 | Timeout | 불변 |
| narrative 2000ms 초과 | Timeout | 불변 |
| invalid JSON | Invalid | 불변 |
| empty text | Invalid | 불변 |
| unknown request ID | Invalid | 불변 |
| stale turn/hash | Stale | 불변 |
| action outside current ActionSpace | Invalid | 불변 |

수동 PASS:

1. provider 없이 TUI 시작
2. `G`, `A`, `J` CTA를 각각 실행
3. fallback/status 텍스트 확인
4. core snapshot hash 비교
5. suggestion은 `Y` 명시 승인 전 submit되지 않음을 확인

transport test는 redirect 0회, system proxy 미사용, response body 65,536 bytes 제한, request body 32,768 bytes 제한, queue capacity 16, C0/C1/ANSI 제거를 추가로 검증한다.

exit smoke는 pending request 중 terminal restore가 먼저 수행되고 worker 종료 대기가 250ms를 넘지 않는지 검증한다.

Checkpoint R6 현재 상태: **PASS / CLOSED**. `audit_report_10.md`의 IMP-F009/010/011과 DBG-F004를 시정해 versioned public boundary, synchronous input error, response schema gate, non-exhaustive enum, 저장소 fixture와 pending-exit smoke를 추가했다. `audit_report_11.md`가 모든 연결 finding과 XPF-F007을 Verified하고 R6 checkpoint를 종결했다. 실제 model provider smoke는 비차단 고려 대상이다.

## 10. R7 출처·호환성 게이트

연결 gap: G-LICENSE-001, G-COMPAT-001
성공 기준: SC-COMPAT-01과 provenance inventory/checksum/legacy 격리/fail-closed validator. SC-LICENSE-01의 owner approval와 whole-work NGPL 결정은 2026-07-20 R8 준비에서 반영했다.

```bash
test -s PROVENANCE.md
test -s docs/compatibility/README.md
rg -q "98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2" PROVENANCE.md
! rg -n "legacy_nethack_port_reference" Cargo.toml crates apps src --glob '*.toml' --glob '*.rs'
cargo test -p aihack --locked --test nethack_367_compat
cargo test -p aihack --locked --test golden_phase8_rules
scripts/r7_checkpoint.sh
```

PASS 조건:

- runtime 포함 파일의 provenance status가 모두 `Approved`다.
- 공식 source archive checksum이 `PROVENANCE.md`와 일치한다.
- NH367-C001..C010 각각 source locator, 관찰, 명령, 기대 event, test function이 있다.
- 레거시 경로 직접 import와 path dependency가 0건이다.
- 라이선스 적용 범위는 담당자 검토 결과와 배포 결정을 기록한다. 본 감사는 법률 자문을 대체하지 않는다.

Checkpoint R7 현재 상태: **PASS WITH KNOWN RISKS**. `audit_report_14.md`까지 기술 finding과 표적 42개·전체 322개 검증은 완료됐다. 2026-07-20 프로젝트 소유자의 파생물 분류·NGPL 승인 evidence를 PROV-0004와 scenario 10개에 기록해 validator는 PASS한다. R7 통과만으로 외부 배포할 수 없으며 R8 기술 감사가 남아 있다.

## 11. R8 통합 릴리즈 게이트

연결 gap: G-DOC-001
선행: R1~R7 모두 PASS

R8은 승인된 whole-work NGPL, version 0.3.0, 공식 LICENSE checksum, packaging/notice/source archive와 외부 배포 가능 여부를 함께 판정한다. workspace package가 NGPL이 아니거나 R7/R8 중 하나라도 HOLD면 release artifact 외부 게시를 중단한다.

R8 최종 PASS 전에 SC-LICENSE-01을 재실행해 PROV-0004와 NH367-C001..C010의 project-owner approval authority/evidence를 확인한다. 공식 LICENSE, NOTICE와 complete corresponding source packaging도 함께 검증한다.

2026-07-20 R8-0에서 `scripts/r8_checkpoint.sh`와 fixture 회귀 테스트 5개를 추가했다. 이어 프로젝트 소유자의 파생물 분류에 따라 workspace 0.3.0/NGPL, 공식 `LICENSE`, `NOTICE`, release source archive 계약과 provenance approval를 반영하고 licensing 회귀 테스트 4개를 추가했다. checkpoint의 로컬 PASS는 R8 독립 감사 또는 외부 게시 승인이 아니다.

같은 날 R8-1C에서 `designs.md`의 current v0.3.0/R8 release contract를 동기화하고, 역사적 R0 문서 감사 판정을 보존한 별도 R8 12항목 self-check와 `tests/r8_documentation.rs` 회귀 계약을 추가했다. 문서 self-check와 R8 checkpoint는 로컬 PASS이며, 수동 TUI matrix·SC-BUILD-02 원격 CI·독립 R8 감사는 계속 pending이다.

`audit_report_16.md` HOLD 시정에서 compatibility index를 Approved record와 동기화하고, project-owner 직접 지시를 `AIHACK-OWNER-2026-07-20-NGPL-01`로 추적했다. 배포되지 않는 Git history 주장은 `MODIFICATIONS.md`와 commit-expanded `RELEASE-METADATA`로 대체했고 `scripts/verify_release_bundle.sh` 및 실제 임시 Git archive 회귀 테스트가 필수 문서, commit, checksum과 legacy 제외를 검증한다. 이는 qualified legal opinion을 주장하지 않으며 clean R8 commit/same-commit CI gate를 면제하지 않는다.

동일 commit이 push되면 CI는 Ubuntu의 `./build.sh --release`와 Windows의 `cmd /c build.bat --release`를 각각 실행해 실제 platform bundle을 검증한다. 로컬 dirty worktree에서는 release script가 의도대로 중단되므로, 이 원격 결과가 DBG-F006의 clean-tree evidence가 된다.

`audit_report_17.md`의 `DBG-F007` 시정은 approval record를 output/source archive와 checksum의 필수 항목으로 추가하고, archive/output metadata의 owner/modification ID를 각 bundled record ID와 대조한다. positive actual-archive fixture와 문서 누락·metadata 누락·record ID 불일치 negative cases가 이 reference-integrity boundary를 검증한다. 이 로컬 시정도 clean commit/same-commit CI evidence를 대체하지 않는다.

`audit_report_18.md`의 exactness 시정은 metadata를 부분 문자열로 찾지 않고 key/value로 파싱해 필수 key가 정확히 한 번만 존재하며 전체 값이 expected와 같은지 검사한다. archive/output 양쪽의 wrong, suffixed, duplicate owner/modification values를 actual archive fixture로 거부하고 Windows release gate에도 같은 단일-key 계약을 적용한다.

```bash
scripts/r8_checkpoint.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo metadata --locked --no-deps --format-version 1
cargo audit
cargo deny check licenses bans sources
git diff --check
cargo test -p aihack --locked --test release_bundle
```

`scripts/r8_checkpoint.sh` exit code는 PASS 0, 승인·release 준비 대기 HOLD 1, version dependency drift·archive 손상 등 구조적 FAIL 2다. R8 최종 감사에서는 먼저 checkpoint PASS를 확인한 뒤 나머지 전체 명령을 실행한다.

최종 수동 확인:

```bash
scripts/r8_tui_core_flow.sh
scripts/r6_pty_matrix.sh
scripts/r6_pending_exit_smoke.sh
```

- Cargo, README, CHANGELOG의 release version이 0.3.0이다.
- Title → character creation → play → inventory → game over → new run 흐름이 동작한다.
- reduced motion과 high contrast에서 LLM 상태와 CTA를 텍스트만으로 구분한다.
- provider disabled/timeout/stale flow에서 core hash가 변하지 않는다.
- archive chain의 모든 경로가 존재하며 이전 파일을 덮어쓰지 않았다.
- `AI_IMPLEMENTATION_DOC_STANDARD.md` 12항목을 다시 PASS한다.

2026-07-20 로컬 PTY 결과: core flow는 high contrast/reduced motion에서 Title → Character Creation → Playing → Inventory → Game Over → New Run과 59x23 최소 크기 clean exit를 PASS했다. deterministic loopback degraded matrix는 success/timeout/stale/down을, pending-exit smoke는 terminal restore-before-worker-wait를 PASS했다. 실제 LLM/API는 호출하지 않았다.

## 12. 최종 판정 템플릿

```text
AIHack v0.3.0 Audit
Commit:
Date/OS/Rust:
R0 Documentation: PASS|FAIL
R1 Build: PASS|FAIL
R2 State: PASS|FAIL
R3 Content: PASS|FAIL
R4 Long-run: PASS|FAIL
R5 Workspace: PASS|FAIL
R6 Local LLM: PASS|FAIL
R7 Provenance/Compatibility: PASS|FAIL
R8 Release: PASS|FAIL
Mandatory failures:
Known risks:
Evidence paths:
Verdict: PASS|FAIL|PASS WITH KNOWN RISKS
```

현재 구현 판정: R1 local PASS (SC-BUILD-02 remote CI pending), R2~R6 PASS, R7 `PASS WITH KNOWN RISKS`, R8 implementation/local verification 진행 중이며 independent audit NOT RUN. R6는 `audit_report_11.md` 독립 재감사로 종결됐으며 실제 model provider smoke는 비차단 고려 대상이다.
현재 문서 감사 판정: `audit_report_9.md`가 IMP-F008과 R1~R5를, `audit_report_11.md`가 보고서 10 시정과 R6 checkpoint를 PASS로 종결했다. 전체 program PASS는 R7~R8 및 SC-BUILD-02 원격 CI evidence가 완료된 뒤에만 선언한다.
