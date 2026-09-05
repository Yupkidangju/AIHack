# AIHack v0.3.0 최종 다중감사 시정 재감사 보고서 25

- 감사 대상: `docs/multi_audit/1/final_audit_report_1_remediation.md`
- 원 감사: `docs/multi_audit/1/final_audit_report_1.md`
- 프로젝트: `C:\LocalDev\rust\AIHack`
- 감사 일자: 2026-08-23
- 기준 HEAD: `80d959af94cb08c5d9b2f2601f5e63f3827a1210`
- 기준 브랜치: `codex/audit-report-24-remediation`
- 작업 트리 상태: 시정 변경이 아직 커밋되지 않은 dirty working tree
- 실행 환경: Windows 11 Pro, `x86_64-pc-windows-msvc`
- Rust/Cargo: 1.94.1
- 보안 도구: `cargo-audit 0.22.1`, `cargo-deny 0.19.4`
- 적용 기준: `AI_AUDIT_DOC_STANDARD.md`, `audit_roadmap.md`, `spec.md`, `designs.md`, `AGENTS.md`
- 감사 원칙: 구현·테스트·설정은 수정하지 않고 감사 보고서만 추가한다.

## 0. 최종 판정

**HOLD — LOCAL REMEDIATION PASS 기각 / REWORK REQUIRED 아님**

현재 시정본은 replay self-verification, 저장 입력 상한, mutable API 축소, registry/equipment lifecycle, 일부 TUI 상태와 terminal RAII, dependency ledger, Windows verifier, immutable CI action pin 등 다수의 핵심 결함을 실질적으로 개선했다. 전체 workspace 품질 게이트도 로컬에서 모두 통과했다.

그러나 광범위한 녹색 테스트와 별개로 실제 production 경로를 직접 대조·변조한 결과 다음 차단급 문제가 남아 있다.

1. malformed save의 역방향 inventory 관계가 수용되고 armor 검증 산술에 debug panic 경로가 남아 있다.
2. save writer가 16 MiB를 초과하는 파일을 성공 기록한 뒤 같은 API의 loader가 거부한다.
3. replay input/output 동일 파일 금지를 `./` alias로 우회해 입력 trace를 증폭할 수 있다.
4. GoldScore witness가 명세의 gold-zero paired production score가 아니라 복제한 점수식에 의존한다.
5. 실제 TUI event loop의 입력 우선순위·late response reset이 직접 mapper 테스트와 다르다.
6. 지원 최소 화면에서 blocking prompt가 보이지 않고, mouse hit target이 표시 CTA와 다르다.
7. Windows release verifier가 checksum에 없는 추가 실행 파일을 포함한 output을 PASS한다.
8. active 문서의 CLI·audit authority·gap lifecycle이 서로 충돌한다.
9. terminal cleanup 검증은 실제 ConPTY test 이름이 주장하는 범위를 모두 assert하지 않는다.

Finding 집계는 **Critical 0, Major 9, Minor 4**다. Major가 남아 있으므로 `PASS`나 `PASS WITH KNOWN RISKS`로 전환할 수 없다. remediation 문서의 `PROGRAM HOLD` 자체는 정확하지만, `FIN-F001..F018 로컬 Verified`와 `LOCAL REMEDIATION PASS` 주장은 철회되어야 한다.

## 1. 감사 범위

### 1.1 인벤토리

생성물과 비제품 reference tree를 제외한 현재 제품 표면을 다음과 같이 확인했다.

| 구분 | 수량 |
| --- | ---: |
| 제품 파일 | 315 |
| Markdown 문서 | 66 |
| Rust 소스·테스트 | 213 |
| TOML/JSON/YAML/lock 설정 | 22 |
| Bash/PowerShell/Batch 스크립트 | 9 |

감사 시작 시 working tree는 tracked 변경 100개, untracked 항목 30개였고 tracked diff는 `3423 insertions, 7134 deletions`였다. 대규모 삭제 수에는 루트의 과거 감사 보고서를 `docs/audit/`로 옮기는 작업이 포함된다.

### 1.2 확인한 통제 문서

- `spec.md`
- `designs.md`
- `README.md`
- `BUILD_GUIDE.md`
- `IMPLEMENTATION_SUMMARY.md`
- `DESIGN_DECISIONS.md`
- `GAP_CLOSURE_ROADMAP.md`
- `CHANGELOG.md`
- `LESSONS_LEARNED.md`
- `PROVENANCE.md`
- `MODIFICATIONS.md`
- `PROJECT_OWNER_LICENSE_APPROVAL.md`
- `RELEASE-METADATA`
- `audit_roadmap.md`
- `AI_AUDIT_DOC_STANDARD.md`
- `docs/multi_audit/1/final_audit_report_1.md`
- `docs/multi_audit/1/final_audit_report_1_remediation.md`
- `docs/audit/audit_report_22.md`부터 `audit_report_24_remediation.md`까지의 연결 이력

### 1.3 집중 확인한 구현·검증 표면

- 저장·replay·경로: `crates/aihack-runtime/src/save.rs`, `apps/aihack-headless/src/lib.rs`, `apps/aihack-headless/src/main.rs`
- session/world/content: `session.rs`, `world.rs`, `domain/entity.rs`, `systems/*`, `causal.rs`
- TUI: `apps/aihack-tui/src/tui/*`, package-local TUI/ConPTY tests
- LLM: config/decision/narrative 계약과 transport/TUI integration tests
- 공급망·릴리스: `deny.toml`, exception/duplicate ledger, CI workflow, `build.sh`, `build.bat`, 양 OS verifier와 release tests
- 문서 gate: `tests/r8_documentation.rs`, build/license/provenance contract tests
- 전체 workspace의 source, test, manifest와 변경 diff

### 1.4 제외·제한 범위

- `target/`, `runtime/`, `output/`, `.git/`은 생성물 또는 저장소 내부 상태이므로 제품 소스 감사에서 제외했다.
- `legacy_nethack_port_reference/` 466개 파일의 본문은 제품 코드가 아니므로 재감사하지 않았다. 대신 runtime direct import/path dependency 0건과 R7 gate를 확인했다.
- 실제 외부 LLM provider smoke는 명시된 비목표이므로 제외했다.
- Windows Terminal GUI의 픽셀·폰트 렌더링은 제외했다. Windows ConPTY는 자동 검증했다.
- 현재 설치 target은 Windows MSVC 하나뿐이므로 `#[cfg(unix)]` branch는 정적 검토만 수행했다.
- dirty working tree이므로 clean release bundle 생성과 현재 시정본의 same-SHA Ubuntu/Windows CI는 실행할 수 없었다.
- commit, push, tag, release, 외부 게시는 수행하지 않았다.

## 2. 전체 및 표적 검증 결과

### 2.1 전체 품질 게이트

| 명령 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked -- --list` | named test 397개 확인 |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| `cargo audit` | PASS, vulnerability 0 |
| `cargo deny check licenses bans sources` | PASS, bans/licenses/sources ok |
| Git Bash `scripts/r7_checkpoint.sh` | PASS |
| Git Bash `scripts/r8_checkpoint.sh` | PASS |
| `cmd /c build.bat --release` | 기대대로 exit 1, dirty release 차단 |

### 2.2 주요 표적 회귀

다음 표적도 모두 녹색이었다.

- save validation, save/load, replay, headless path/policy, transaction, world invariant
- causal content와 3-seed long run
- external consumer compile-fail accessor contract
- TUI package contract, UI layout/input/runtime smoke, 실제 Windows ConPTY
- LLM transport/revision/TUI integration
- dependency exception/duplicate gate
- provenance/license/release bundle과 Windows negative matrix
- R8 문서 authority tests

이 결과는 구현된 positive/negative case가 동작한다는 증거다. 다만 아래 재현은 현재 test matrix가 누락한 inverse relation, write-side budget, path alias, actual output set, production dispatch와 render-content 경계를 직접 확인했다.

### 2.3 독립 동적 probe

감사용 probe는 ignored `target/` 또는 임시 fixture에서만 실행했고 프로젝트 구현 파일은 수정하지 않았다.

| Probe | 관찰값 | 판정 |
| --- | --- | --- |
| OnMap item을 `Inventory { owner: 999 }`로 변경 후 `GameSession::from_save_data` | `ORPHAN_INVENTORY_OWNER_ACCEPTED=true`, `ORPHAN_WORLD_INVARIANTS_VALID=true` | 역방향 관계 검증 누락 |
| 512-byte Message 40,000개 session 저장 후 즉시 load | `SAVE_BYTES_WRITTEN=24009611`, limit `16777216`, reload reject | writer/loader 계약 모순 |
| replay input `replays/run.jsonl`, output `./replays/run.jsonl` | exit 0, line 1개에서 2개로 증가 | 동일 파일 alias guard 우회 |
| 정상 Windows release fixture에 `UNTRACKED-UNSIGNED-PAYLOAD.exe` 추가 | verifier exit 0, `PASS Windows release bundle` | unsigned extra artifact 수용 |
| 모든 Markdown 상대 링크 검사 | broken 1개 | `README.md:63`의 이동 전 경로 |

## 3. FIN-F001~F018 재감사 상태

| 원 Finding | 재감사 상태 | 요약 |
| --- | --- | --- |
| FIN-F001 | **Needs Fix** | typed validator는 추가됐지만 inverse inventory relation, `hp <= max_hp`, armor 산술 경계 누락 |
| FIN-F002 | **Needs Fix** | read 상한은 유효하나 writer가 16 MiB 초과 self-unloadable save를 기록 |
| FIN-F003 | **Verified** | 7개 replay field mismatch와 consumed-prefix no-partial-commit 확인 |
| FIN-F004 | **Needs Fix** | root no-follow는 개선됐지만 same-file alias guard와 public ambient helper가 남음 |
| FIN-F005 | **Verified** | production `DerefMut` 제거와 external compile-fail 확인 |
| FIN-F006 | **Needs Fix** | speed/AI pair는 개선됐지만 GoldScore paired production oracle과 독립 negative가 없음 |
| FIN-F007 | **Verified** | immutable registry, corpse continuation, armor lifecycle 회귀 확인 |
| FIN-F008 | **Needs Fix** | 기본 mapper/overlay는 개선됐지만 production dispatch priority와 late response reset이 불완전 |
| FIN-F009 | **Needs Fix** | theme/layout tier/mouse capture는 연결됐지만 prompt clipping과 CTA hit mismatch가 남음 |
| FIN-F010 | **Needs Fix / Verification Hold** | RAII는 개선됐지만 ConPTY·failure injection evidence가 원 요구 범위를 닫지 못함 |
| FIN-F011 | **Needs Documentation Recovery** | parser/default는 일치하나 active BUILD 표가 반대 계약을 유지 |
| FIN-F012 | **Needs Fix** | predecessor/current authority와 child/aggregate gap lifecycle이 여전히 충돌 |
| FIN-F013 | **Needs Fix (Minor)** | 현재 ledger/graph는 정렬됐지만 checker가 TOML 구조와 exact trigger set을 fail-closed로 잠그지 않음 |
| FIN-F014 | **Needs Fix** | Windows verifier가 checksum에 없는 추가 output artifact를 허용 |
| FIN-F015 | **Needs Documentation Recovery / Remote Pending** | full SHA pin은 유효하나 action 주석이 틀리고 현재 시정 SHA/CI가 없음 |
| FIN-F016 | **Needs Fix (Minor)** | 값·locale·cooldown seam은 정렬됐지만 실제 wall-clock timing tests가 남음 |
| FIN-F017 | **Needs Fix (Minor)** | package-local tautology/callability tests, stale phase 주석, Unix branch 미검증 |
| FIN-F018 | **Verified within frozen threat model** | bounded read + atomic batch rewrite와 late hard-link 거부 확인 |

## 4. Pass 1 — 구현·문서 정합성 Findings

### [R25-IMP-F001] Save semantic validator가 inverse relation과 산술 경계를 놓침

- Related: FIN-F001
- Pattern: `IMP-001`, `TEST-001`, `SEC-004`
- Severity: **Major**
- Status: **Needs Fix**
- Evidence:
  - `crates/aihack-runtime/src/save.rs:436-451`은 item이 `OnMap`일 때만 location을 검증한다.
  - `save.rs:562-569`는 `Inventory` owner가 player인 item만 inventory index와 대조한다.
  - 따라서 `Inventory { owner: 999 }` item은 존재하지 않는 owner와 index 부재를 모두 우회했고 direct probe에서 load와 world invariant가 모두 성공했다.
  - `save.rs:481-510`은 `max_hp > 0`과 alive actor의 `hp > 0`은 검사하지만 `hp <= max_hp`를 검사하지 않는다.
  - `save.rs:573-585`는 persisted `ac_bonus: i16`을 별도 범위 확인 없이 `adventurer_template().ac - bonus`에 사용한다. `i16::MIN`은 debug overflow가 가능한 결정적 산술 경계다.
  - `tests/save_validation.rs`의 dangling fixture는 inventory entry가 missing item을 가리키는 정방향만 검사한다.
- Expected: `spec.md:642`의 full saved-world relation과 stat/equipment invariant가 live session 생성 전 typed `InvalidSave`로 닫혀야 한다.
- Actual: schema-valid malformed world가 수용되거나 validator 자체가 debug panic할 수 있다.
- Impact: inventory orphan, UI/score/lifecycle 모호성, malformed save에 의한 process crash.
- Suggested Fix:
  1. 모든 `Inventory { owner }` location은 owner 존재, player/inventory owner 일치와 양방향 index를 검사한다.
  2. alive/dead와 `0 < hp <= max_hp` 및 run state 관계를 명시적으로 검증한다.
  3. persisted item stat에 범위를 두고 derived AC는 `checked_sub` 또는 넓은 정수형에서 검증한 뒤 변환한다.
  4. 정방향·역방향 relation, `hp=max/max+1`, `ac_bonus=min/max`를 debug/release에서 typed reject하는 회귀를 추가한다.
- Re-audit: inverse owner probe가 `InvalidSave`이고 panic 0건인지 확인한다.

### [R25-IMP-F002] GoldScore witness가 production paired oracle을 사용하지 않음

- Related: FIN-F006
- Pattern: `IMP-003`, `DBG-002`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Evidence:
  - `spec.md:822`는 동일 world/turn에서 gold만 제거한 paired score와 실제 final score의 차이가 gold와 정확히 같아야 한다고 정의한다.
  - `crates/aihack-runtime/src/causal.rs:196-206`은 production score 함수를 호출하지 않고 kill/depth/inventory/turn 식을 `score_without_gold`로 재구현한다.
  - `causal.rs:277-291`은 한 session의 Quit 결과를 그 복제식과 비교한다.
  - `tests/long_run.rs:106`의 negative fixture는 `turn_advanced=true`와 event를 한 번에 넣어 event-only와 turn-only를 독립 격리하지 않는다.
  - missing-witness negative는 한 witness만 제거하며 각 원인의 독립 removal matrix가 아니다.
- Expected: 실제 production score path를 사용하는 gold/no-gold A/B pair와 독립 event-only, turn-only, witness별 removal negative.
- Actual: 구현과 별도의 복제 oracle이 같은 개념 오류를 공유하거나 drift할 수 있다.
- Impact: 3-seed deterministic hash가 잘못된 causal attribution을 반복 재현하는 false-green이 될 수 있다.
- Suggested Fix:
  1. 같은 saved world/turn에서 gold만 0인 control session을 만들고 양쪽 모두 production score path로 종료한다.
  2. 두 final score의 차이가 원 gold와 정확히 같은 경우에만 witness를 기록한다.
  3. event-only, turn-only, 각 required witness 제거를 독립 fixture로 분리한다.
  4. raw TOML `replacen`은 ID 기반 typed registry fixture로 바꾼다.
- Re-audit: GoldScore producer 하나를 제거했을 때 GoldScore만 사라지고 다른 8개 witness/hash 계약은 유지되는지 확인한다.

### [R25-IMP-F003] TUI production dispatch와 reset이 state contract를 우회함

- Related: FIN-F008
- Pattern: `IMP-001`, `DBG-001`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:1134-1151`은 state mapper 전에 LLM result의 uppercase `N`과 `Esc`를 먼저 소비한다. GameOver에서 명시된 `N -> NewRun`과 state cancel이 우회될 수 있다.
  - `mod.rs:1451-1474`는 `Tab/BackTab`을 state보다 먼저 focus 이동으로 처리한다. 반면 MorePrompt 화면은 `Press any key to continue`라고 표시한다.
  - `mod.rs:609-625`는 reset 시 old outstanding request ID를 ignored 목록에 넣는다.
  - 그러나 `mod.rs:342-361`은 current outstanding request가 없을 때만 ignored ID를 확인한다. reset 뒤 새 요청이 outstanding인 동안 old response가 도착하면 silent discard가 아니라 `Invalid`가 된다.
  - package tests는 mapper나 `handle_candidate`를 직접 호출하고 위 production event dispatch 우선순위를 통과하지 않는다.
- Expected: 단일 state-aware dispatcher가 화면 CTA, keyboard event source, late response lifecycle에 동일하게 사용되어야 한다.
- Actual: unit-level mapper는 green이지만 실제 event loop의 guard 순서가 다른 의미를 만든다.
- Impact: NewRun/cancel/MorePrompt 입력 불일치, 새 session에 이전 response 오류 표시, shipped TUI 핵심 흐름 신뢰 저하.
- Suggested Fix:
  1. raw event에서 candidate를 만드는 순서를 하나의 순수 dispatcher로 추출하고 production loop와 tests가 같이 사용한다.
  2. blocking state의 키 의미를 LLM dismiss/focus보다 먼저 결정한다.
  3. ignored ID는 current outstanding 유무와 무관하게 가장 먼저 제거·폐기한다.
  4. old response와 new outstanding이 교차하는 load/new-run 회귀, GameOver+LLM result의 N, MorePrompt의 Tab/BackTab을 추가한다.
- Re-audit: one-event/one-frame 및 ConPTY에서 각 조합을 직접 실행하고 core revision/hash와 표시 상태를 확인한다.

### [R25-IMP-F004] 최소 화면 blocking prompt와 mouse CTA geometry가 실제 렌더와 불일치

- Related: FIN-F009
- Pattern: `IMP-001`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Evidence:
  - `apps/aihack-tui/src/tui/layout.rs:58-63`의 60x24 degraded layout은 log 높이가 1이다.
  - `layout.rs:76-95`의 80x24 standard layout도 body height 20, command 3으로 계산되어 log 높이가 1이다.
  - `mod.rs:1354-1369`는 3개 line과 border가 필요한 Awaiting/MorePrompt overlay를 log height로 자른다. 결과적으로 최소 지원 크기에서는 title border만 남고 실제 방향/item/`--More--` 안내가 보이지 않는다.
  - `apps/aihack-tui/src/tui/input.rs:281-297`의 command hit-test는 row를 검사하지 않고 16-column 고정 구간으로 Inventory/Wait/Open을 선택한다.
  - `render_panels.rs:103-115`의 실제 CTA 문자열 길이·border inset은 그 구간과 일치하지 않는다. panel title/border 클릭도 command가 되고 표시된 Wait 일부가 Inventory로 매핑될 수 있다.
  - 현재 ConPTY는 map click만 검증해 command/inspect CTA geometry를 잠그지 않는다.
- Expected: 지원 최소 크기에서 blocking instruction이 읽혀야 하고, click target은 실제 렌더된 label의 동일 row/column만 활성화해야 한다.
- Actual: 화면은 입력을 기다리지만 이유를 숨기고, 표시와 다른 core command를 제출할 수 있다.
- Impact: soft lock처럼 보이는 UX, 오입력에 의한 turn/state 변경, keyboard/mouse equivalence 위반.
- Suggested Fix:
  1. blocking overlay를 위한 최소 content 높이를 보장하거나 별도 modal 영역에 렌더한다.
  2. 렌더 label과 hit box를 같은 구조화된 CTA model에서 파생한다.
  3. border/title/blank/outside click은 focus 또는 no-op이어야 한다.
  4. 60x24, 80x24 buffer snapshot에서 실제 prompt text를 assert하고 CTA별 exact boundary negative를 추가한다.
- Re-audit: buffer와 ConPTY에서 MorePrompt/Awaiting 텍스트, command CTA와 inspect panel click을 직접 검증한다.

### [R25-IMP-F005] active 문서의 CLI·audit authority·gap lifecycle이 모순됨

- Related: FIN-F011, FIN-F012, FIN-F017 일부
- Pattern: `IMP-004`, `BUILD-001`, `DOC-BACKFILL-001`
- Severity: **Major**
- Status: **Needs Fix / Needs Documentation Recovery**
- Evidence:
  - `BUILD_GUIDE.md:15-24`의 active/current 표는 long run을 `wait-only, 조기 사망도 exit 0`으로 적는다.
  - 같은 문서 `:248-249`, `spec.md:683`, `apps/aihack-headless/src/main.rs:18-23`은 default `survival-v1`, target `1..=1,000,000`을 정의하고 실제 parser도 이를 따른다.
  - `GAP_CLOSURE_ROADMAP.md:16-21`은 `Open -> Implemented -> Verified -> Closed`를 정의한다.
  - G-BUILD-006, G-TEST-003, G-DOC-004, G-SEC-001은 `Implemented`인데 이들을 포함한 G-FINAL-001은 FIN-F001..F018 전체를 `Verified`라고 선언한다.
  - `tests/r8_documentation.rs:189-193`은 child gap의 `Implemented`를 유지하도록 assert하면서 aggregate mismatch를 검사하지 않는다.
  - `GAP_CLOSURE_ROADMAP.md:260`, `spec.md:784`, `DESIGN_DECISIONS.md:397,424`는 report 23 independent re-audit를 current pending처럼 유지하지만 `audit_roadmap.md:384,466`은 report 23/24를 historical closed로 정의한다.
  - `designs.md:9`는 전체 final remediation이 아니라 FIN-F008..F010만 진행 중이라고 표시한다.
  - 감사 보고서 이동 뒤 `README.md:63`의 `audit_report_21.md` 링크가 깨졌다. 실제 파일은 `docs/audit/audit_report_21.md`다.
- Expected: predecessor는 historical, 현재 authority는 report 25의 HOLD로 단일화되고 child와 aggregate 상태가 같은 lifecycle을 따라야 한다.
- Actual: 완료·Implemented·Verified·old pending이 동시에 current처럼 존재한다.
- Impact: 운영자와 다음 coder가 잘못된 gate를 반복하거나 미시정 항목을 release-ready로 오인할 수 있다.
- Suggested Fix:
  1. 문서 우선 원칙에 따라 active status table, BUILD current row, designs status, ADR verification update를 먼저 동기화한다.
  2. G-FINAL은 이 보고서의 Major가 닫힐 때까지 `Implemented` 또는 명시적 HOLD 상태로 두고 child 상태와 자동 대조한다.
  3. report 23/24는 historical closure, report 25는 current HOLD로 분리한다.
  4. README 링크를 `docs/audit/audit_report_21.md`로 수정하고 전체 상대 링크 checker를 gate에 넣는다.
- Re-audit: active section/row exact assertions와 전체 Markdown link scan을 다시 실행한다.

## 5. Pass 2 — Debug·Engineering Quality Findings

### [R25-DBG-F001] Save writer가 load budget을 적용하지 않아 self-unloadable artifact를 생성함

- Related: FIN-F002
- Pattern: `DBG-001`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Evidence:
  - `crates/aihack-runtime/src/save.rs:129-133`의 `save_session`은 session을 pretty JSON으로 직렬화한 뒤 크기와 semantic validation 없이 `write_atomic`한다.
  - 같은 파일 `:135-147`, `:181-195`의 load path는 16 MiB bounded read를 강제한다.
  - 512-byte 유효 Message 40,000개를 가진 session은 24,009,611-byte save를 성공 기록했고 즉시 같은 `ArtifactStore::load_session`에서 거부됐다.
- Expected: writer가 성공을 반환한 save는 같은 version의 loader budget 안에서 다시 읽혀야 하며 실패 시 기존 destination을 보존해야 한다.
- Actual: 정상 public API가 자신의 reader 계약을 위반하는 artifact를 원자적으로 게시한다.
- Impact: 사용자 데이터 복구 실패, 저장 성공 오표시, disk/memory 과소비.
- Suggested Fix:
  1. `to_save_data` 결과에 semantic validation을 먼저 적용한다.
  2. bounded serializer 또는 capped buffer로 `MAX_SAVE_BYTES`를 write 전에 강제한다.
  3. 초과 시 typed resource error를 반환하고 기존 save를 교체하지 않는다.
  4. long-run event/RNG retention과 max-turn save 가능 여부를 명세에서 결정한다.
- Re-audit: byte exact/+1, 기존 destination 불변, writer 성공 후 immediate reload round trip을 debug/release에서 검증한다.

### [R25-DBG-F002] replay 동일 파일 검사가 lexical PathBuf alias에 의존함

- Related: FIN-F004
- Pattern: `DBG-001`, `SEC-004`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Evidence:
  - `save.rs:637-649`의 `validate_relative_path`는 parent/absolute는 거부하지만 `.` component를 제거하지 않고 원 PathBuf를 반환한다.
  - `apps/aihack-headless/src/main.rs:56-63`은 replay input/output을 그 PathBuf의 lexical equality로만 비교한다.
  - input `replays/run.jsonl`, output `./replays/run.jsonl`은 같은 파일이지만 guard를 통과했고 exit 0으로 line 수가 1에서 2로 늘었다.
  - `save.rs:261-283`의 check-then-use ambient `resolve_path_in_root`도 production에서 사용하지 않지만 여전히 public API다.
- Expected: 동일 capability-relative object를 가리키는 alias는 입력/출력 동시 지정에서 모두 거부되고 production artifact I/O는 `ArtifactStore` 하나로 제한되어야 한다.
- Actual: 문자열이 다른 동일 파일을 허용해 입력 evidence를 mutation한다.
- Impact: replay 증폭·손상, 후속 mismatch, audit evidence 오염.
- Suggested Fix:
  1. relative component를 canonical lexical form으로 정규화하고 `CurDir`을 제거한다.
  2. Windows case alias와 기존 파일의 file identity까지 비교한다.
  3. public ambient helper는 private/test-only로 축소하거나 제거한다.
  4. `path`, `./path`, case variant와 file alias negative를 production binary test로 추가한다.
- Re-audit: 모든 alias가 exit 2이고 input bytes/line count가 불변인지 확인한다.

### [R25-DBG-F003] terminal cleanup 구현은 개선됐지만 claimed failure coverage가 완결되지 않음

- Related: FIN-F010
- Pattern: `DBG-001`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix / Verification Hold**
- Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:985-1031`은 best-effort restore와 Drop guard를 구현했고 각 restore step failure 뒤에도 나머지를 시도하는 unit test가 통과했다.
  - 실제 Windows ConPTY에서 one-event state, mouse input, normal exit와 alternate-screen leave는 통과했다.
  - 그러나 `apps/aihack-tui/tests/conpty_contract.rs:123-126`의 종료 assertion은 `?1049l` 하나뿐이다. 테스트 이름이 주장하는 mouse-disable, cursor-show와 raw-mode 복원은 assert하지 않는다.
  - failure injection은 restore ops에만 있으며 alternate/raw/mouse/cursor setup, `Terminal::new`, draw/read 실패 경로를 직접 통과하지 않는다.
- Expected: setup·draw·read·restore 어느 지점에서 실패해도 가능한 모든 terminal state를 복원하고 그 사실을 platform harness가 증명해야 한다.
- Actual: 코드 구조는 강해졌지만 원 finding의 failure matrix와 실제 Windows cleanup evidence가 일부 추론에 의존한다.
- Impact: Windows-only terminal 고착 회귀가 녹색 test를 통과할 수 있다.
- Suggested Fix:
  1. setup/draw/read 단계에 실패 주입 seam을 추가하고 guard state와 cleanup 호출을 확인한다.
  2. ConPTY transcript에서 mouse capture enable/disable, cursor hide/show와 alternate leave를 assert한다.
  3. 가능한 경우 child 종료 뒤 console mode/raw-mode 상태를 platform API로 확인한다.
- Re-audit: 각 단계 failure matrix와 actual ConPTY normal/error exit를 모두 통과해야 한다.

### [R25-DBG-F004] LLM timing test의 wall-clock flakiness가 일부 남음

- Related: FIN-F016
- Pattern: `TEST-001`
- Severity: **Minor**
- Status: **Needs Fix**
- Evidence:
  - UI cooldown의 260ms sleep은 injected `UiClock`으로 개선됐다.
  - `tests/llm_transport.rs:394-400`, `:662-690`에는 실제 1초 deadline polling, `yield_now`, elapsed 상한이 남아 있다.
  - 같은 파일의 250ms server delay와 `tests/llm_tui_integration.rs:255,286,326,350`의 실제 deadline polling도 남아 있다.
  - 이번 감사 실행에서는 실패하지 않았다.
- Expected: scheduler 부하가 assertion 의미를 바꾸지 않는 deterministic synchronization 또는 injected clock.
- Actual: 일부 integration timing은 wall clock과 host scheduling에 의존한다.
- Impact: 느린 CI에서 비결정적 실패, 동일 SHA 재현성 저하.
- Suggested Fix: readiness channel/barrier, deterministic fake transport clock과 bounded shutdown signal을 사용하고 실제 시간 smoke는 느슨한 별도 platform test로 격리한다.
- Re-audit: CPU 부하 또는 반복 실행에서도 결과가 같고 core hash가 불변인지 확인한다.

### [R25-DBG-F005] FIN-F017의 package-local semantic test와 maintenance 증거가 부분 시정에 그침

- Related: FIN-F017
- Pattern: `TEST-001`, `DEP-001`, `DOC-BACKFILL-001`
- Severity: **Minor**
- Status: **Needs Fix / Needs Documentation Recovery**
- Evidence:
  - `crates/aihack-runtime/tests/entity_store_contract.rs`는 spawn 후 `is_some()`만 확인한다.
  - `environment_systems_contract.rs`는 trap search 결과를 무시한다.
  - `projection_contract.rs`는 snapshot hash를 자기 clone과 비교한다.
  - `game_client_contract.rs`는 `unreachable!()` compile-only adapter다.
  - 이 package-local test들은 기준 HEAD 대비 변경되지 않았다. 새 TUI/ConPTY tests는 별도 영역이므로 원 국소 finding을 대체하지 않는다.
  - `tests/data_loading.rs`, `ui_screens.rs`, `ui_debug.rs`, `ui_labels.rs`에 `[v0.2.0] Phase ...` active 주석이 다수 남아 remediation 문서의 stale comment 제거 주장과 충돌한다.
  - Unix parent directory `sync_all`은 구현됐지만 Windows-only 감사와 과거 base-SHA CI로는 현재 branch를 compile/execute하지 못했다.
  - duplicate family/version budget은 exact하게 잠겼지만 owner/reason/shipped scope/review metadata는 없다.
- Expected: package 경계 자체의 semantic delta와 현재 working tree의 양 OS evidence, 현재 단계에 맞는 주석·triage 정보.
- Actual: broad workspace tests는 강하지만 원 maintenance 항목 일부는 그대로다.
- Impact: package 단위 회귀 신호 약화, 유지보수 오판, Unix-only compile/runtime 문제의 지연 발견.
- Suggested Fix: tautology/callability assertions를 state delta로 바꾸고 stale 주석을 current intent 또는 historical marker로 분류하며, duplicate budget에 소유·사유·review 조건을 추가하고 current SHA Linux CI를 확보한다.
- Re-audit: package tests를 단독 실행하고 mutation-negative가 실제 실패하는지 확인한다.

## 6. Pass 3 — Security·Supply Chain Findings

### [R25-SEC-F001] Windows release verifier가 unsigned extra output artifact를 허용함

- Related: FIN-F014
- Pattern: `SEC-006`, `BUILD-001`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Evidence:
  - `scripts/verify_release_bundle.ps1:67-78`은 필수 파일의 존재·크기만 확인한다.
  - `:123-150`은 `SHA256SUMS` record가 기대 checksum name 집합과 일치하는지 검사하지만 output directory의 실제 leaf file 집합은 열거하지 않는다.
  - `build.bat:55`는 기존 ignored `output/`을 정리하지 않고 그 위에 파일을 덮어쓴다. `build.sh`도 같은 방식으로 기존 output을 보존한다.
  - 정상 fixture와 checksum 생성 뒤 `UNTRACKED-UNSIGNED-PAYLOAD.exe`를 추가한 probe에서 PowerShell verifier가 exit 0과 `PASS Windows release bundle`을 반환했다.
  - Windows negative matrix의 6개 fault에는 extra output file이 없다.
- Expected: 배포 directory가 bundle이면 실제 파일 집합이 선언된 exact set과 같고 모든 배포 파일이 checksum inventory에 포함되어야 한다.
- Actual: stale·악성·실수로 남은 추가 실행 파일을 checksum 없이 함께 게시할 수 있다.
- Impact: release provenance와 checksum 완전성 우회, 잘못된 payload 배포.
- Suggested Fix:
  1. release staging을 새 empty temporary directory에서 수행한 뒤 원자적으로 publish한다.
  2. 또는 verifier가 실제 top-level leaf 집합과 expected set을 정확히 비교하고 extra directory/file을 모두 거부한다.
  3. Linux와 Windows에 extra file/directory/symlink·reparse negative를 같은 계약으로 추가한다.
- Re-audit: 추가 unsigned 파일 하나만 넣어도 양 OS verifier와 release build가 nonzero인지 확인한다.

### [R25-SEC-F002] Dependency exception checker가 구조적 TOML과 exact trigger 집합을 검증하지 않음

- Related: FIN-F013
- Pattern: `SEC-006`, `DEP-001`
- Severity: **Minor**
- Status: **Needs Fix**
- Evidence:
  - 현재 `dependency-exceptions.json`, `deny.toml`, resolved `winx 0.36.4` graph와 expiry는 정렬됐고 `cargo deny`도 PASS했다.
  - `tests/dependency_exception_gate.rs:79,104-112`는 deny table을 TOML로 파싱하지 않고 전체 문자열의 `contains`로 name/version/license를 찾는다. comment decoy가 구조 검증을 대신할 수 있다.
  - `:114-126`은 ledger에 존재하는 trigger key만 순회하므로 필수 key가 ledger에서 삭제된 경우 exact expected set을 확인하지 않는다.
  - unrelated negative는 ledger crate 이름만 바꾸며 실제 deny table 오배치와 decoy를 함께 검증하지 않는다.
- Expected: ledger, parsed deny exception table, exact resolved graph trigger set과 expiry가 구조적으로 결합된 fail-closed gate.
- Actual: 현재 값은 옳지만 regression checker가 미래 drift의 일부 형태를 놓칠 수 있다.
- Impact: exception provenance가 조용히 약화될 수 있다. 현재 cargo-deny가 추가 방어를 제공하므로 즉시 Major로 승격하지 않는다.
- Suggested Fix: `deny.toml`을 TOML AST로 파싱하고 exception table exact equality, trigger key exact set, valid calendar date와 decoy negative를 추가한다.
- Re-audit: comment decoy, deny table crate swap, trigger key 삭제, 만료/version/path drift가 각각 실패하는지 확인한다.

### [R25-SEC-F003] CI action pin은 immutable이지만 provenance 주석과 일반 gate가 부정확함

- Related: FIN-F015
- Pattern: `SEC-006`, `BUILD-001`
- Severity: **Minor**
- Status: **Needs Documentation Recovery / Remote Pending**
- Evidence:
  - `.github/workflows/ci.yml`의 두 `uses:`는 실제 40-hex SHA로 고정되어 immutable pin 핵심은 시정됐다.
  - checkout SHA `11d5960a326750d5838078e36cf38b85af677262`는 공식 tag 조회상 `v4.4.0`/`v4`인데 workflow comment는 `v4.2.2`라고 적는다. 실제 `v4.2.2`는 `11bd71901bbe5b1630ceea73d27597364c9af683`이다.
  - `tests/build_contract.rs:203-213`은 현재 두 literal SHA가 존재하는지만 검사한다. 앞으로 새 mutable `uses:`가 추가돼도 모든 action이 full SHA인지 일반적으로 검사하지 않는다.
  - 최신 원격 run `32110917881`은 base HEAD `80d959a`의 historical success이며 dirty remediation 변경을 포함하지 않는다.
- Expected: 주석/tag/SHA provenance가 일치하고 모든 `uses:`를 일반 full-SHA policy가 검사하며 현재 시정 commit의 양 OS run이 있어야 한다.
- Actual: 실행 action은 immutable이지만 설명과 future regression policy, current same-SHA evidence가 불완전하다.
- Impact: reviewer 오판과 향후 mutable action 재도입 가능성. 현재 program HOLD 문구는 이를 정확히 반영한다.
- Suggested Fix: 올바른 tag 주석 또는 verified source URL을 기록하고 workflow 모든 `uses:`의 40-hex ref를 구조적으로 검사한 뒤 clean commit의 Ubuntu/Windows CI를 확보한다.
- Re-audit: workflow parser와 official commit/tag 확인, current SHA 양 OS green을 대조한다.

## 7. Cross-Pass Conflicts

| Conflict | 결론 |
| --- | --- |
| 397개 named test와 전체 gate는 PASS지만 direct probes가 Major를 재현 | broad green은 구체적 누락 경계를 덮지 못하므로 finding을 유지한다. |
| remediation 문서는 FIN-F001..F018 Verified라지만 active gap child는 Implemented | aggregate Verified를 기각하고 report 25 HOLD를 current authority로 둔다. |
| CLI parser는 survival/range 계약을 구현했지만 active BUILD 표는 wait-only라고 선언 | 코드 동작은 유지하고 문서를 recovery해야 한다. |
| Windows verifier의 declared checksum set은 exact하지만 실제 output set은 exact하지 않음 | checksum record exactness만으로 bundle 완전성을 주장할 수 없다. |
| terminal RAII source는 개선됐지만 ConPTY assertion은 일부 cleanup만 증명 | 구현 개선을 인정하되 FIN-F010 closure evidence는 미완료로 둔다. |
| action은 full SHA지만 comment가 다른 tag를 주장 | 현재 immutability는 인정하고 provenance 문서/gate만 Minor로 유지한다. |

## 8. 반복 실패 집중진단

이번 재감사에서 이전과 같은 false-green 유형이 반복됐다.

| 반복 원인 | 관찰 | 결정 |
| --- | --- | --- |
| production entrypoint 대신 helper만 테스트 | TUI mapper는 green이나 event-loop guard 순서가 다름 | **Refactor**: dispatcher를 단일 함수로 통합 |
| 한 방향 positive/negative만 검사 | inventory entry→item은 검사하지만 item→owner/index는 누락 | **Refactor**: 양방향 invariant validator |
| read와 write에 서로 다른 budget 적용 | loader만 16 MiB 강제 | **Refactor**: ArtifactStore read/write 공통 contract |
| 검증 대상 집합과 실제 산출물 집합 혼동 | checksum names는 exact지만 output extras는 미검사 | **Continue with structural exact-set gate** |
| production 계산식을 test oracle에서 복제 | GoldScore가 실제 paired production path를 사용하지 않음 | **Refactor**: A/B production oracle |
| aggregate 상태를 child보다 먼저 승격 | G-FINAL Verified와 child Implemented 동시 존재 | **Continue with lifecycle invariant test** |

현재 구조 전체를 다시 쓰는 `Rewrite Module`까지는 필요하지 않다. 다만 save validation/ArtifactStore, TUI dispatch/hit geometry, causal GoldScore oracle은 국소 patch의 반복보다 책임 경계를 먼저 정리하는 `Refactor`가 적절하다.

## 9. PASS 전 필수 수정 순서

### P0 — correctness/security gate

1. R25-IMP-F001: save inverse relation, stat, derived arithmetic fail-closed.
2. R25-DBG-F001: write-side semantic/byte budget와 no-clobber failure.
3. R25-DBG-F002: replay path alias identity와 ambient helper 축소.
4. R25-IMP-F002: actual gold/no-gold production pair와 independent negatives.
5. R25-IMP-F003: TUI production dispatcher와 late response lifecycle 단일화.
6. R25-IMP-F004: minimum-size blocking prompt와 render-derived mouse hit boxes.
7. R25-SEC-F001: release output actual exact-set와 extra artifact negative.
8. R25-IMP-F005: active authority/lifecycle/BUILD/link 문서 동기화.
9. R25-DBG-F003: terminal setup/error/cleanup evidence 완결.

### P1 — 재발 방지와 유지보수

10. dependency exception gate의 구조적 TOML/exact trigger 검증.
11. CI action provenance 주석과 모든 `uses:` full-SHA 일반 gate.
12. LLM wall-clock tests의 deterministic seam.
13. package-local semantic tests, stale comment 분류, duplicate budget ownership.
14. clean commit 생성 후 동일 SHA Ubuntu/Windows CI와 platform release bundle 검증.

## 10. Accepted Risks와 명세 확인 항목

### 10.1 현재 동결된 Accepted/Excluded Risk

- FIN-F018의 same-account malicious concurrent directory-entry swap은 single-writer 사용자 전용 runtime root threat model 밖이다. bounded read, no-follow, single-link, atomic rewrite 경계는 유지한다.
- Windows parent directory power-loss durability는 OS/filesystem 정책에 따른 잔여 위험이다. file sync와 atomic replace는 계속 필수다.
- 실제 remote/model provider smoke는 v0.3.0 비차단 비목표다.
- Windows Terminal GUI rendering은 수동 범위지만 ConPTY state/cleanup contract를 대체하지 않는다.

### 10.2 Needs Spec Clarification

- headless target 1,000,000 turn과 save event 100,000/RNG 1,000,000/16 MiB 상한을 동시에 사용할 때 save 가능성을 보장할지 결정해야 한다. 보장한다면 event history compaction/retention이 필요하고, 보장하지 않는다면 `--save` 실패 조건과 사용자 메시지를 BUILD/spec에 명시해야 한다.
- release `output/` 전체를 게시 bundle로 정의하는 현재 문맥에서는 extra file을 거부해야 한다. 게시 대상이 checksum inventory 파일만이라는 다른 모델을 원한다면 build/publish 절차가 그 파일만 새 staging artifact로 묶도록 문서·코드에서 명시해야 한다.

## 11. 재감사 체크리스트

다음 조건을 모두 충족해야 report 25 HOLD를 해제할 수 있다.

1. 각 Major에 이름 붙은 failure-mode regression이 추가되고 수정 전 fixture가 실제 RED였다는 기록이 있다.
2. malformed save inverse matrix가 debug/release 모두 typed reject하며 validator panic이 없다.
3. save writer의 exact/+1 byte test와 writer-success→immediate-load round trip이 통과한다.
4. replay `path`/`./path`/Windows case alias가 nonzero이고 input hash/line count가 불변이다.
5. GoldScore가 같은 world/turn의 production gold/no-gold pair로 검증된다.
6. production TUI dispatcher 조합, 60x24/80x24 prompt buffer, CTA hit boundaries가 직접 테스트된다.
7. ConPTY normal/error exit에서 alternate, mouse, cursor와 가능한 raw-mode 상태를 확인한다.
8. 양 OS verifier가 extra file/directory를 nonzero로 거부한다.
9. active 문서의 current authority가 report 25와 일치하고 Markdown broken link가 0이다.
10. 아래 전체 gate가 다시 통과한다.

```text
git diff --check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --release --all-targets --locked
cargo audit
cargo deny check licenses bans sources
Git Bash scripts/r7_checkpoint.sh
Git Bash scripts/r8_checkpoint.sh
```

11. clean commit의 동일 SHA로 Ubuntu/Windows CI, 각 platform release build/verifier가 모두 green이다.
12. 새 독립 재감사가 FIN-F001~F018과 R25 finding을 연결해 `Verified`로 판정한다.

## 12. 최종 근거와 Coder Handoff

### 최종 근거

- 많은 시정은 실제로 유효하며 특히 FIN-F003, FIN-F005, FIN-F007, FIN-F018은 현재 근거로 Verified다.
- 그러나 save, path identity, causal oracle, TUI production path, release actual set, active authority에서 재현된 Major가 남았다.
- 따라서 현재 가능한 정확한 상태는 **LOCAL REMEDIATION PARTIAL / PROGRAM HOLD**다.
- clean same-SHA CI 부재만이 HOLD의 이유가 아니다. 로컬에서 먼저 수정해야 할 correctness/security finding이 존재한다.

### Coder Handoff

```text
`C:\LocalDev\rust\AIHack\docs\audit\audit_report_25.md`의 최신 재감사 결과를 확인하고,
각 finding을 관련 프로젝트 문서와 실제 코드에 대조하여 검토한 후 필요한 수정을 수행하세요.
계약 변경이 필요한 경우 `spec.md`, `designs.md`, BUILD/ADR/gap 문서를 먼저 갱신하고,
수정 전 실패 fixture를 보존한 회귀 테스트를 추가한 뒤 전체 로컬 gate와 clean same-SHA 양 OS CI를 실행하세요.
감사 보고서의 결론을 자동 권위로 복사하지 말고 현재 사용자 요구, 통제 문서, production 경로와 재현 증거를 함께 대조하세요.
```
