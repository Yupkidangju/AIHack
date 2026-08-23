# AIHack Final Multi-Audit Report 1

## 1. Audit Metadata

- Audit Turn: 1
- Audit Date: 2026-08-23 (Asia/Seoul)
- Project Root: `C:\LocalDev\rust\AIHack`
- Audited HEAD: `80d959af94cb08c5d9b2f2601f5e63f3827a1210`
- Branch: `codex/audit-report-24-remediation`
- Audit Mode: Standard-backed multi-agent audit
- Standard: `C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md`
- User Goal: 현재 프로젝트의 모든 문서와 구현을 파악하고 모순·문제점을 진단하여 해결 가능한 감사 결과를 만든다.
- Final Decision: **HOLD**

## 2. User Goal and Decision Basis

이번 감사는 문서만 검토하거나 테스트 성공 여부만 확인하지 않았다. 문서 → public/runtime API → production 호출 경로 → 테스트 → 실제 실행 결과를 양방향으로 추적했다. 최종 판정은 다음 원칙을 따른다.

- Critical 또는 미해결 Major finding이 있으면 PASS 금지
- 전체 테스트 green은 false-green 가능성을 별도로 제거한 뒤에만 완료 증거로 인정
- 파일·경로·데이터 무결성 같은 고위험 경계는 두 독립 보고서의 실제 증거가 있어야 Covered
- 직접 재검증하지 못한 Major는 `Unverified`로 유지하고 PASS 금지
- 문서에 없는 제품 요구는 창작하지 않고 `Needs Clarification`으로 분류

## 3. Scope and Exclusions

### 포함 범위

- 활성 제품·설계·구현·빌드·감사·변경 이력 문서 57개
- Rust source 208개, test/fixture 77개, Cargo manifest/lock 9개
- core/content/runtime/AI contract/LLM/TUI/headless workspace 전체
- save/load/replay/report 및 capability filesystem 경계
- transaction/invariant/RNG/entity lifecycle/content causality
- TUI 상태·입력·접근성·terminal lifecycle·LLM UI 통합
- dependency graph, RustSec, cargo-deny, CI, R7/R8, Linux/Windows bundle, provenance
- 현재 HEAD와 GitHub Actions same-SHA evidence

### 제외 범위

- `.git/`, `target/`, `runtime/` 생성물의 일반 내용
- `legacy_nethack_port_reference/` 본문 구현. 단 active runtime/release가 이를 참조·포함하는지는 검사
- 외부 유료/원격 LLM provider 호출: 현재 spec의 비필수 범위
- 외부 게시·release·tag·push 및 법률 자문
- Windows Terminal/ConPTY의 정확한 console mode bit와 initialization-failure 주입: 보완 감사 후에도 미검증

## 4. Work-Surface Inventory

| Surface | 주요 위치 | 핵심 경계 |
| --- | --- | --- |
| Product/document authority | `spec.md`, README, BUILD, summary, gap/audit roadmap, ADR, changelog | current/pending/Verified 계보 |
| Deterministic core | `crates/aihack-core` | state, RNG, invariant, save DTO |
| Content/runtime | `aihack-content`, `aihack-runtime` | registry source, transaction, entity lifecycle, causality |
| Headless | `apps/aihack-headless` | CLI, policy, save/replay/report |
| TUI | `apps/aihack-tui` | state/input/layout/accessibility/terminal lifecycle |
| Local LLM | `aihack-llm`, `aihack-ai-contract` | loopback, size/schema/revision/action gate |
| Test evidence | root/workspace tests, fixtures, scripts | false-green, deterministic hash, OS coverage |
| Supply chain/release | Cargo graph, deny/audit, CI/build scripts, provenance | license, bundle, checksum, same-SHA evidence |

## 5. Agent Allocation and Rationale

| Agent | Perspective | 배정 이유 |
| --- | --- | --- |
| A01 | contract_docs | 전체 문서 권한·CLI 계약·audit chain 정합성 |
| A02 | core_runtime | transaction/state/save/RNG/entity/content 인과 |
| A03 | file_security | 파일·경로·저장·replay·LLM 신뢰 경계 |
| A04 | tests_determinism | false-green, causal/replay/hash/OS test coverage |
| A05 | supply_chain_ci | dependency/license/CI/release/provenance |
| A06 | ui_llm_platform | TUI/입력/접근성/LLM/terminal/platform |

파일·경로·데이터 무결성은 A02와 A03이 독립 확인했고, A06도 TUI 호출 경로를 추가 교차 검증했다. 공급망·release integrity는 A05가 독립 검사했다. Windows interactive coverage gap에는 A06 supplement를 1회 수행했다.

## 6. Immutable Source Report Manifest

- Manifest: `C:\LocalDev\rust\AIHack\docs\multi_audit\1\source_report_manifest.json`
- Manifest SHA-256: `f50cbffb8d36b3dc80a59b20e917dd606381aee6b7edf40e82e78e3ebcb28cb9`
- Sidecar: `C:\LocalDev\rust\AIHack\docs\multi_audit\1\source_report_manifest.sha256.json`
- Missing Source Reports: `[]`

| Immutable Report | Perspective | SHA-256 |
| --- | --- | --- |
| `sub_audit_01_contract_docs.md` | 계약·문서 | `c2286046186c36c6dcd4e4e83760cb52cf5a5ebc9dc68de7c65d4ba8aaf48d37` |
| `sub_audit_02_core_runtime.md` | 코어·런타임 | `de465f962a2192dcb0e2ee8212859b944b411c0a9c78c35fc15e8baa9b6d344c` |
| `sub_audit_03_file_security.md` | 파일·보안 | `6c06411ca130986aaa3915ca4e14ba81c0e5693905cfcd56cc1ddfbac2191584` |
| `sub_audit_04_tests_determinism.md` | 테스트·결정론 | `fd4b1a376a05c5fbda62789a31db13108a18b9e1af9db1926cbcf652785ae75d` |
| `sub_audit_05_supply_chain_ci.md` | 공급망·CI | `f27e2b36af8f76b382a91ada83945595422974118ddcfe45c3294de3373f76e9` |
| `sub_audit_06_ui_llm_platform.md` | UI·LLM·플랫폼 | `8fb6b3bb6d5321ed776a6cabe041863c47e7b23ec2570f7677409e4f14af32cc` |
| `sub_audit_06_ui_llm_platform_supplement_1.md` | Windows interactive 보완 | `ae55e62a46fa24ff849308c8006ef09fe1427796a0530ff33138613c3622a9dd` |

## 7. Evidence and Commands

### 성공 증거

| Command / Evidence | Result |
| --- | --- |
| `cargo metadata --locked --no-deps --format-version 1` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 363 listed tests; Windows `release_bundle` target은 0 tests |
| `cargo build --workspace --release --locked` | PASS |
| `cargo audit` | PASS, 0 vulnerability/warning |
| Windows Git Bash R7/R8 | PASS / PASS |
| Current HEAD CI | Actions `32110917881`, Ubuntu/Windows 모든 단계 PASS |
| Current HEAD bundle | Linux/Windows positive build PASS |
| Windows native psmux | normal alternate restore, 59x23 안내, GameOver→New Run, pending worker 322ms exit 확인 |

### 메인 직접 실패 재현

| Probe | Direct Result |
| --- | --- |
| `runtime` root junction → outside | headless exit 0, outside `saves/escape.json` 생성 |
| SaveData player ID 변조 | exit 101, panic 재현 |
| SaveData duplicate entity ID | exit 0, 성공 report 수용 |
| SaveData control event | exit 0, persisted control text 수용 |
| RNG `draws=1,000,000,000` | 1초 내 load 미완료, 비제한 복원 재현 |
| Replay turn/outcome/hash 위조 | exit 0, accepted turn 1 성공 report 수용 |
| 외부 consumer의 `GameSession` mutation | `s.turn=999; s.world.nutrition=0` compile 성공 |
| Armor wear→drop | AC `0 -> -1 -> -1`, equipped는 `None` |
| Injected corpse nutrition 500 | eat delta 49, embedded default 50 경로 사용 확인 |

## 8. Coverage Gap Check

| Work Surface / Audit Question | Agents | Evidence | Coverage | Follow-up |
| --- | --- | --- | --- | --- |
| Active document authority | A01, A05, main | 문서/CI/git 대조 | Covered | FIN-F012 |
| Core transaction/public mutation | A02, main | source + external compile | Covered | FIN-F005 |
| Save/load semantic integrity | A02, A03, A04, main | source + malformed probes | Covered | FIN-F001/F002 |
| Replay integrity | A02, A03, A04, main | source + forged replay | Covered | FIN-F003 |
| File/path capability boundary | A02, A03, A06, main | source + junction probe | Covered | FIN-F004/F018 |
| Causal/content/entity lifecycle | A02, A04, main | source + registry/armor probes | Covered | FIN-F006/F007 |
| Test determinism/false-green | A02, A04, main | full/target tests + source | Covered | FIN-F001/F006/F014 |
| TUI shipped flow | A06 + supplement + main | source + Windows native capture | Covered with findings | FIN-F008/F009 |
| Windows Terminal/ConPTY failure path | A06 supplement | psmux normal path; no ConPTY | **Not Covered** | FIN-F010; dedicated harness |
| LLM transport/revision | A03, A06, main | 22+9+10 tests, source | Covered | FIN-F016 minor drift |
| Supply chain/CI/release | A05, A04, main | graph, CI logs, scripts | Covered | FIN-F013~F015 |
| External real LLM provider | None | spec non-goal | Excluded | Optional smoke only |
| Legacy reference body | None | runtime exclusion gate | Excluded | No product code audit |

사용자 핵심 범위 중 Windows ConPTY/error injection은 해소되지 않았다. 이 coverage gap만으로도 PASS 계열 판정은 금지된다.

## 9. Canonical Findings

### [FIN-F001] SaveDataV1이 semantic invariant를 검증하지 않아 panic과 모호한 entity state를 수용함

- Sources: A02 `DBG-CORE-F001`; A03 `A03-F002`; A04 `A04-F001`
- Areas: save/load, data integrity, test false-green
- Severity: **Major**
- Status: **Confirmed**
- Summary: schema version만 검사하고 world/entity 관계를 검증하지 않는다. schema test도 malformed JSON이라 실제 version branch를 잠그지 못한다.
- Verified Evidence: player ID 변조 save는 exit 101 panic, duplicate ID save는 exit 0 성공. `from_save_data`는 schema check 후 곧바로 world를 조합한다.
- Expected Basis: persisted input은 live session 생성 전 typed error로 fail-closed해야 한다.
- Actual: invalid state가 runtime `expect`/assert까지 도달하거나 모호한 entity store로 실행된다.
- Impact: crash, 잘못된 전투/inventory/entity lifecycle, 손상 save의 성공 수용.
- Required Action: full semantic save validator, typed `InvalidSave`, valid schema-mismatch test, production panic 전제 제거.
- Re-audit Method: missing/not-player/duplicate/dangling/map/stat/equipment fixture matrix를 debug/release에서 모두 typed reject.
- Synthesis Rationale: 두 독립 고위험 보고서와 메인 실행이 같은 원인을 확인했다.

### [FIN-F002] Save/replay/RNG/event 입력에 resource·control 경계가 없음

- Sources: A02 `DBG-CORE-F002/F003`; A03 `A03-F005/F006`
- Areas: resource exhaustion, persisted text, RNG restore
- Severity: **Major**
- Status: **Confirmed**
- Summary: save 전체 String, replay 전체 Vec, event log, RNG draws가 제한 없이 materialize/replay된다. persisted event control text도 decode 단계에서 통과한다.
- Verified Evidence: RNG 10억 draws save는 1초 내 load하지 못했고, ESC control event save는 exit 0으로 수용됐다. source는 bounded reader와 cardinality cap이 없다.
- Expected Basis: untrusted/s손상 artifact는 byte/record/range/control budget 안에서 fail-closed해야 한다.
- Actual: OOM/CPU hang, terminal/prompt text 오염 가능성이 있다.
- Required Action: 명세 수치 결정 후 byte/line/event/entity/draw cap, bounded streaming, event text validation/sanitization.
- Re-audit Method: boundary-1/boundary/boundary+1 및 control fixture matrix.
- Synthesis Rationale: static evidence와 메인 timeout/control probe가 일치했다.

### [FIN-F003] ReplayLineV1의 turn/outcome/hash가 검증되지 않아 위조 replay가 성공함

- Sources: A02 `SEC-CORE-F001`; A03 `A03-F003`; A04 `A04-F006`
- Areas: replay integrity, determinism, audit evidence
- Severity: **Major**
- Status: **Confirmed**
- Summary: replay runner는 `line.command`만 submit하고 나머지 metadata를 무시한다.
- Verified Evidence: turn 9999, accepted=false, inner/outer forged hash를 가진 replay가 exit 0, accepted turn 1, 실제 hash 성공 report를 만들었다.
- Expected Basis: replay가 self-verifying artifact인지 command-only log인지 spec에서 닫고 그 계약을 강제해야 한다.
- Actual: 현재 필드 정의는 integrity evidence처럼 보이지만 mismatch는 모두 허용된다.
- Impact: 손상·위조 trace를 결정론/감사 기준선으로 오인.
- Required Action: line별 turn/outcome/events/next-state/hash 검증과 typed `ReplayMismatch`, 또는 command-only wire로 명세·schema 축소.
- Re-audit Method: 각 필드를 하나씩 변조한 negative matrix와 no-partial-commit 검사.
- Synthesis Rationale: 세 독립 보고서와 메인 probe가 동일 결론을 냈다.

### [FIN-F004] Capability root bootstrap과 TUI path helper가 canonical root를 강제하지 않음

- Sources: A03 `A03-F001/F007`; A06 `A06-F013`
- Areas: path sandbox, TUI/headless storage API
- Severity: **Major**
- Status: **Confirmed**
- Summary: root 내부 escape는 cap-std가 막지만 `ArtifactStore::open`은 root 자체 symlink/junction을 따라간다. TUI public helper는 caller path의 parent를 새 root로 연다.
- Verified Evidence: Windows `runtime` junction이 외부를 가리킬 때 outside save가 exit 0으로 생성됐다. TUI production adapter는 unrestricted free helper를 호출한다.
- Expected Basis: production artifact I/O는 신뢰된 directory handle + relative path 하나의 경계를 사용해야 한다.
- Actual: root 선택 순간과 TUI helper에서 ambient authority가 다시 열린다.
- Impact: artifact redirect, 향후 file picker/plugin 연결 시 임의 parent read/write.
- Required Action: root 자체 no-follow/reparse 검증 또는 trusted handle injection, TUI `ArtifactStore` 소유, free helper test-only/private화.
- Re-audit Method: root junction/Unix symlink, root swap race, TUI absolute/parent path negative tests.
- Synthesis Rationale: 두 독립 보안 호출 경로와 메인 junction 실행으로 확인했다.

### [FIN-F005] Public DerefMut가 private transaction/state 계약을 무력화함

- Sources: A02 `IMP-CORE-F001`, `XPF-CORE-F001`
- Areas: API encapsulation, transaction/invariant
- Severity: **Major**
- Status: **Confirmed**
- Summary: GameSession/GameWorld/EntityStore가 public state에 `DerefMut`되어 external crate가 submit 없이 상태를 바꿀 수 있다.
- Verified Evidence: 외부 temp consumer에서 `s.turn=999; s.world.nutrition=0`이 compile 성공했다.
- Expected Basis: DEC-STATE-01/SC-CORE-01은 submit만 mutable entry로 선언한다.
- Actual: type system이 mutation, RNG/revision/event/invariant 우회를 허용한다.
- Impact: 새 adapter/LLM/UI consumer가 core truth를 조용히 손상할 수 있다.
- Required Action: public DerefMut 제거, field visibility 축소, typed internal accessors, external compile-fail/API surface test.
- Re-audit Method: 외부 consumer의 direct field/get_mut compile failure와 transaction regression.
- Synthesis Rationale: source surface와 main external compile이 직접 증명했다.

### [FIN-F006] R9 causal witness가 원인별 독립성을 증명하지 못함

- Sources: A02 `IMP-CORE-F002`; A04 `A04-F002/F003/F004/F007`; `XPF-CORE-F002`
- Areas: causal evidence, false-green tests
- Severity: **Major**
- Status: **Confirmed**
- Summary: 한 monster 이동이 Speed와 AI 두 witness를 동시에 채우고, GoldScore는 gold-only score delta를 비교하지 않으며 turn-only negative가 분리되지 않았다. raw TOML `replacen`은 record order에 의존한다.
- Verified Evidence: `CausalSummary`는 동일 `moved` predicate에서 두 witness를 기록하고 `final_score >= gold`만 검사한다.
- Expected Basis: 각 content field의 producer→consumer→semantic delta가 독립적으로 추적되어야 한다.
- Actual: 동일 사건 라벨 복제와 unrelated score 항으로 required set이 false-green 가능하다.
- Impact: 3-seed 반복 hash가 같은 잘못을 반복하는 증거가 될 수 있다.
- Required Action: witness에 entity/content field/value/scenario attribution 저장, field별 A/B와 removal negative, gold-only paired score, 독립 event/turn negative.
- Re-audit Method: speed/AI/gold 원인을 하나씩 제거했을 때 해당 witness만 사라지는지 검사.
- Synthesis Rationale: core와 test 관점이 독립적으로 동일 predicate 결함을 찾았다.

### [FIN-F007] Runtime-created content와 equipment lifecycle가 source of truth/state를 잃음

- Sources: A02 `IMP-CORE-F003`, `DBG-CORE-F004`
- Areas: content registry lifecycle, armor equipment
- Severity: **Major**
- Status: **Confirmed**
- Summary: injected registry는 bootstrap 뒤 보존되지 않아 corpse 생성이 embedded registry로 돌아가고, armor drop은 equipped pointer만 지우고 AC를 복원하지 않는다.
- Verified Evidence: injected corpse nutrition 500에서 실제 eat delta는 49였다. armor probe는 AC `0 -> -1 -> -1`, equipped `None`이었다.
- Expected Basis: session lifecycle 전체가 같은 registry를 사용하고 equipment pointer·location·derived stats가 원자적으로 일치해야 한다.
- Actual: 생성 시점에 따라 content source가 달라지고 armor bonus가 중복 누적 가능하다.
- Impact: combat/hash/save/replay와 content A/B 결과가 불일치.
- Required Action: immutable registry/factory context를 world에 보존; equip/unequip/drop 공통 helper와 AC/equipment invariant.
- Re-audit Method: custom corpse death→eat→save continuation, armor wear/drop/rewear/save/load matrix.
- Synthesis Rationale: source 분석과 두 메인 실행 probe가 확인했다.

### [FIN-F008] TUI state-aware 입력·NewRun/load lifecycle·오류 복구가 문서와 불일치

- Sources: A06 `A06-F001/F002/F003/F006/F012`; A02 `IMP-CORE-F004`; supplement `A06-S1-F005`
- Areas: Inventory/MorePrompt/Title/Creation/NewRun/load/error UX
- Severity: **Major**
- Status: **Confirmed**
- Summary: Inventory overlay/focus, Title L, Creation/Direction Esc, MorePrompt acknowledge가 실제 입력 경로와 연결되지 않는다. NewRun/load는 LLM/UI transient를 완전히 reset하지 않고 save/load error는 process 종료로 전파된다.
- Verified Evidence: Windows native capture에서 `i` 후 overlay 없이 normal screen이 유지됐다. source는 Esc를 global Quit로 처리하고 NewRun/load reset 범위가 좁다.
- Expected Basis: designs/spec의 state graph와 화면에 표시된 CTA.
- Actual: dead affordance, 잘못된 quit, stale presentation/request, 복구 불가능한 storage error가 있다.
- Impact: shipped TUI 핵심 흐름 실패와 old-session UI/LLM state leakage.
- Required Action: explicit UI state machine, item-letter/AcknowledgeMore/cancel mapping, complete transient reset, redacted recoverable storage error state.
- Re-audit Method: one-event/one-frame harness와 실제 PTY에서 각 상태 transition/hash 불변 검증.
- Synthesis Rationale: source, 직접 Windows capture, core dead-state 분석이 일치했다.

### [FIN-F009] Layout/accessibility/mouse 기능이 선언만 있고 실제 renderer/event source와 연결되지 않음

- Sources: A06 `A06-F004/F005/F009`; supplement `A06-S1-F003`
- Areas: responsive layout, high contrast, reduced motion, focus, mouse
- Severity: **Major**
- Status: **Confirmed**
- Summary: breakpoint/비율이 designs와 다르고 theme는 renderer가 소비하지 않으며 mouse capture를 활성화하지 않는다.
- Verified Evidence: `compute_layout`은 fixed 40/60 map이며 80..119 vertical branch가 없다. `theme()` consumer와 Enable/DisableMouseCapture가 없다. Windows startup에도 mouse tracking sequence가 없었다.
- Expected Basis: documented layout tiers, 7:1 high contrast, keyboard/mouse equivalence.
- Actual: synthetic mapper test만 green이고 실제 화면·event source는 미연결이다.
- Impact: minimum terminal clipping, accessibility 약속 불이행, mouse CTA 미동작.
- Required Action: authoritative layout 선택, renderer theme style, focus traversal, real reduced-motion rendering, RAII mouse capture.
- Re-audit Method: buffer snapshots + PTY/ConPTY click/resize/focus matrix.
- Synthesis Rationale: implementation과 Windows runtime evidence가 같은 결론을 냈다.

### [FIN-F010] Terminal cleanup error path와 Windows ConPTY coverage가 닫히지 않음

- Sources: A06 `A06-F010/F011`; supplement `A06-S1-F001/F002/F004`
- Areas: terminal restore, Windows platform evidence
- Severity: **Major**
- Status: **Confirmed implementation gap / Partially Unverified platform behavior**
- Summary: normal alternate restore와 pending worker 322ms exit는 확인됐지만 setup/restore는 RAII guard 밖이며 Windows Terminal/ConPTY/raw mode bit는 미검증이다.
- Verified Evidence: EnterAlternate→raw→Terminal→session 생성에 `?`가 있고 disable_raw 실패 시 LeaveAlternate가 건너뛰어진다. psmux normal path는 clean exit했다.
- Expected Basis: 모든 오류 경로에서 terminal restore 후 bounded worker shutdown.
- Actual: normal path만 Covered; initialization/restore failure와 ConPTY input semantics는 Not Covered.
- Impact: shell raw/alternate 상태 고착 가능성과 Windows-only 회귀 미탐지.
- Required Action: best-effort RAII restore guard, failure injection seam, Windows ConPTY one-event harness.
- Re-audit Method: raw/Terminal/draw/restore 실패 주입 및 Linux PTY/Windows ConPTY 비교.
- Synthesis Rationale: source 결함은 직접 확인됐고 platform gap은 supplement로도 해소되지 않았다.

### [FIN-F011] Headless CLI default와 `--turns` 범위가 문서 계약과 불일치

- Sources: A01 `A01-F003/F004`
- Areas: CLI contract, automation
- Severity: **Major**
- Status: **Confirmed / default choice Needs Clarification**
- Summary: BUILD는 default `survival-v1`, source/help는 `wait-v1`; 문서의 `1..=1,000,000` 범위도 parser가 강제하지 않는다.
- Verified Evidence: source default는 wait-v1이고 `--turns 0`이 exit 0/no-op success였다.
- Expected Basis: documented flag contract 또는 명시적으로 승인된 source behavior.
- Actual: 자동화가 다른 policy를 실행하고 invalid zero turn을 성공으로 오인한다.
- Impact: release/long-run command 의미가 호출자에 따라 달라진다.
- Required Action: canonical default 결정, Clap range parser, boundary/help/implicit-default tests.
- Re-audit Method: no-policy run과 turns 0/1/1,000,000/1,000,001 matrix.
- Synthesis Rationale: source·help·실행과 BUILD 문서가 명시적으로 충돌한다.

### [FIN-F012] Audit/release current-state 문서가 successor report와 gap lifecycle을 반영하지 않음

- Sources: A01 `A01-F001/F002/F006/F007`; A05 `SC-F006`
- Areas: document authority, gap lifecycle, CI evidence
- Severity: **Major**
- Status: **Confirmed**
- Summary: report 24가 report 23 finding을 검증했는데 active 문서/test는 report 23 pending을 현재 gate로 유지한다. `Closed / re-audit pending`도 lifecycle 정의와 모순된다.
- Verified Evidence: README/summary/build/gap/audit/design/compatibility와 r8 documentation test의 stale expectations를 직접 확인했다. 현재 HEAD CI `32110917881`은 success지만 active docs는 older implementation run 중심이다.
- Expected Basis: predecessor는 historical, current pending은 report24 후속 또는 본 multi-audit finding으로 단일화해야 한다.
- Actual: 완료·pending·Closed가 동시에 current처럼 존재한다.
- Impact: 운영자가 완료 작업을 반복하거나 미검증 시정을 release-ready로 오인.
- Required Action: active authority table과 gap enum 정규화, current HEAD/run 연결, ADR verification status와 CHANGELOG 동기화.
- Re-audit Method: section/row-aware stale predecessor negative tests와 current authority exact assertion.
- Synthesis Rationale: A01/A05와 main 문서·CI 대조가 일치했다. A01-F005의 “현재 HEAD CI 없음” 부분은 current run `32110917881`로 기각했다.

### [FIN-F013] Dependency license exception provenance와 expiry가 machine gate로 닫히지 않음

- Sources: A05 `SC-F001/F002`
- Areas: cargo-deny exception, provenance
- Severity: **Major**
- Status: **Confirmed**
- Summary: deny exception은 실제 PASS하지만 PROV-0005가 “no exception”이라 기록하고 expiry/owner/version trigger는 문자열 presence test뿐이다.
- Verified Evidence: deny에는 winx exception, PROVENANCE에는 no exception. CI cargo-deny는 current HEAD에서 PASS했다.
- Expected Basis: shipped exception ledger가 graph/owner/expiry와 fail-closed 연결되어야 한다.
- Actual: 2026-10-31 이후에도 checker 없이 cargo-deny만 계속 PASS 가능하다.
- Impact: stale exception과 provenance drift가 release gate를 우회.
- Required Action: machine-readable exception ledger와 expiry/version/graph checker, PROV-0005 동기화.
- Re-audit Method: expired/version-drift/unrelated-crate negative fixtures + cargo-deny.
- Synthesis Rationale: current build 실패가 아니라 lifecycle/provenance failure로 채택했다.

### [FIN-F014] Windows release bundle negative 검증이 Linux보다 약함

- Sources: A05 `SC-F003`, `XPF-SC-001`; A04 `A04-F005`
- Areas: Windows packaging, checksum/provenance
- Severity: **Major**
- Status: **Confirmed**
- Summary: Linux는 공통 verifier를 실행하지만 Windows build.bat은 일부 positive checks와 hash 생성만 하며 release_bundle test도 0개다.
- Verified Evidence: build.sh verifier 호출, build.bat의 exclusion/record equality/checksum reverify 부재, Windows `release_bundle` 0 tests를 직접 확인했다.
- Expected Basis: 양 OS bundle은 같은 fail-closed contract를 가져야 한다.
- Actual: 정상 CI positive path는 green이지만 legacy include/record mismatch/zero-size/hash tamper negative path가 Windows에서 닫히지 않았다.
- Impact: Windows artifact 무결성이 Linux보다 약하다.
- Required Action: cross-platform verifier 또는 Windows PowerShell parity, Windows negative fixture matrix.
- Re-audit Method: legacy/mismatch/duplicate/wrong hash/zero-size tamper cases가 양 OS 모두 nonzero.
- Synthesis Rationale: test와 supply-chain 보고서가 독립적으로 동일 gap을 확인했다.

### [FIN-F015] CI action pin과 v0.3.0 release modification boundary가 불명확함

- Sources: A05 `SC-F004/F005`, `XPF-SC-002`
- Areas: CI provenance, release scope
- Severity: **Major**
- Status: **Confirmed / release scope Needs Clarification**
- Summary: workflow는 mutable action tag를 사용하고 MODIFICATIONS cutoff는 2026-07-20인데 이후 R9/보안 변경이 Unreleased 상태로 v0.3.0 bundle에 들어간다.
- Verified Evidence: `actions/checkout@v4`, `dtolnay/rust-toolchain@1.94.1`; MODIFICATIONS 종료일과 post-July commits를 직접 확인했다.
- Expected Basis: release action code와 modification notice가 exact candidate commit에 묶여야 한다.
- Actual: preflight/bundle PASS와 final external release authority가 분리되어 있으나 machine/document boundary가 단일하지 않다.
- Impact: mutable CI orchestration, modification notice 누락, preflight를 final approval로 오인.
- Required Action: action full SHA pin, exact candidate/version/changelog/modification period 결정, pending이면 packaging HOLD gate.
- Re-audit Method: pinned workflow 양 OS run + candidate archive/metadata/changelog/modification exactness.
- Synthesis Rationale: 외부 게시 자체는 HOLD지만 future release gate가 fail-open인 점을 채택했다.

### [FIN-F016] LLM timeout/rationale/i18n 계약과 timing test가 일관되지 않음

- Sources: A06 `A06-F007/F008`; A04 `A04-F008`
- Areas: LLM contract, presentation locale, test stability
- Severity: **Minor / Major if 5-locale UI is required**
- Status: **Needs Clarification**
- Summary: decision timeout 1500/2000ms, rationale 0/1자, UI/LLM 언어 scope가 문서·API마다 다르고 일부 test가 wall-clock sleep에 의존한다.
- Verified Evidence: config/helper constants와 문서 값을 대조했고 locale source가 없으며 영어/한국어 fallback이 혼재한다.
- Required Action: canonical timeout/schema/locale authority와 test clock seam.
- Re-audit Method: 동일 provider fixture와 승인 locale matrix.
- Synthesis Rationale: core isolation 자체는 강하므로 별도 낮은 우선순위 finding으로 유지했다.

### [FIN-F017] 유지보수·내구성·국소 test 품질 위험

- Sources: A01 `A01-F008`; A03 `A03-F008`; A04 `A04-F009`; A05 `SC-F007`
- Areas: stale comments, parent fsync/root permission, package-local smoke, duplicate budget
- Severity: **Minor / Info**
- Status: **Confirmed as maintenance risk**
- Summary: stale phase 주석, crash durability 미정, compile/callability-only tests, duplicate dependency 전역 허용이 존재한다.
- Required Action: current/historical 주석 분류, durability threat model, semantic package tests, duplicate budget.
- Re-audit Method: maintenance checklist와 dependency/test diff review.
- Synthesis Rationale: 개별 항목은 즉시 blocker가 아니므로 하나의 maintenance finding으로 병합했다.

### [FIN-F018] Replay append hard-link race는 직접 재현되지 않음

- Sources: A03 `A03-F004`
- Areas: concurrent hard-link TOCTOU
- Severity: **Major candidate**
- Status: **Unverified / Needs Threat-Model Clarification**
- Summary: open 후 nlink 검사와 write 사이에 same-user attacker가 link를 추가할 수 있다는 정적 가설이다.
- Verified Evidence: preplaced hard-link 방어는 PASS. 실제 concurrent race는 source report와 main 모두 재현하지 못했다.
- Expected Basis: 문서가 same-user concurrent adversary까지 차단한다고 주장하는지 불명확하다.
- Actual: 단일 nlink check가 시간 전체의 불변을 보장하지는 않는다.
- Impact: threat model에 따라 외부 link inode 동시 변경 가능성.
- Required Action: threat model 결정; 필요하면 atomic rewrite/lock/디렉터리 권한 설계와 barrier test.
- Re-audit Method: deterministic open-before-write barrier를 둔 Unix/Windows concurrent fixture.
- Synthesis Rationale: 증거가 정적이고 공격자 가정이 미확정이므로 Confirmed로 승격하지 않았다. Unverified Major가 남아 PASS를 차단한다.

## 10. Critical/Major Direct Re-verification

| Canonical Finding | Main Checked | Evidence Re-opened / Command Re-run | Result | Gate Impact |
| --- | --- | --- | --- | --- |
| FIN-F001 | Yes | save loader/source + player/duplicate probes | Confirmed | HOLD |
| FIN-F002 | Yes | reader/RNG source + timeout/control probes | Confirmed | HOLD |
| FIN-F003 | Yes | replay runner + forged trace | Confirmed | HOLD |
| FIN-F004 | Yes | ArtifactStore/TUI call graph + root junction | Confirmed | HOLD |
| FIN-F005 | Yes | DerefMut source + external compile | Confirmed | HOLD |
| FIN-F006 | Yes | causal predicates/tests | Confirmed | HOLD |
| FIN-F007 | Yes | death/item/equipment source + two probes | Confirmed | HOLD |
| FIN-F008 | Yes | TUI mapping/reset/error source + native capture evidence | Confirmed | HOLD |
| FIN-F009 | Yes | layout/theme/mouse source + supplement runtime | Confirmed | HOLD |
| FIN-F010 | Yes/Partial | terminal source + psmux; no ConPTY failure injection | Partially Unverified | HOLD |
| FIN-F011 | Yes | source/help/turns=0 execution | Confirmed | HOLD |
| FIN-F012 | Yes | active docs/tests + current CI run | Confirmed | HOLD |
| FIN-F013 | Yes | deny/provenance/CI | Confirmed | HOLD |
| FIN-F014 | Yes | build.sh/build.bat/test listing | Confirmed | HOLD |
| FIN-F015 | Yes | workflow/history/modification docs | Confirmed | HOLD |
| FIN-F018 | No runtime reproduction | source-only race hypothesis | Unverified | HOLD |

## 11. Cross-Report Conflicts

1. **전체 363 tests/양 OS CI green vs runtime integrity failure**
   해소: test green은 valid fixture와 positive path 증거로 유지한다. malformed save, replay tamper, public mutation, lifecycle probes가 별도 Major를 확인하므로 전체 PASS로 승격하지 않는다.

2. **Capability path tests PASS vs root/TUI ambient authority escape**
   해소: root 내부 final/nested escape는 Verified control이다. root bootstrap과 TUI helper는 별도 실패 경계로 FIN-F004에 유지한다.

3. **R9 witness/hash repeatability PASS vs causal attribution false-green**
   해소: 반복성은 유지하되 원인별 독립성 부족으로 R9 causal completion은 HOLD한다.

4. **Current HEAD CI run 존재 vs A01-F005 “same-SHA 없음”**
   해소: `80d959a` run `32110917881`을 main이 확인해 A01-F005의 핵심 전제를 Rejected했다. 문서가 current run을 연결하지 않는 부분만 FIN-F012에 병합했다.

5. **R8 checkpoint/bundle PASS vs 외부 release HOLD**
   해소: checkpoint는 positive preflight로 인정한다. Windows negative parity와 candidate modification scope가 닫히기 전 final release PASS가 아니다.

## 12. Finding Adjudication Ledger

| Source Finding | Decision | Canonical | Rationale |
| --- | --- | --- | --- |
| A01-F001/F002/F006/F007 | Merged | FIN-F012 | audit authority/gap lifecycle 동일 원인 |
| A01-F003/F004 | Merged | FIN-F011 | headless CLI contract |
| A01-F005 | Rejected in core premise | FIN-F012 일부 | current HEAD CI `32110917881` 존재 |
| A01-F008 | Merged | FIN-F017 | maintenance 주석 |
| IMP-CORE-F001 | Accepted | FIN-F005 | external compile로 확인 |
| IMP-CORE-F002 | Merged | FIN-F006 | causal attribution |
| IMP-CORE-F003 | Accepted | FIN-F007 | registry probe 확인 |
| IMP-CORE-F004 | Merged | FIN-F008 | dead Awaiting/UI state |
| DBG-CORE-F001 | Merged | FIN-F001 | malformed save |
| DBG-CORE-F002/F003 | Merged | FIN-F002 | RNG/resource bounds |
| DBG-CORE-F004 | Merged | FIN-F007 | armor probe 확인 |
| SEC-CORE-F001 | Merged | FIN-F003 | replay integrity |
| XPF-CORE-F001/F002/F003 | Merged | FIN-F005/F006/F003 | cross-pass conflicts preserved |
| A03-F001/F007 | Merged | FIN-F004 | root/TUI path authority |
| A03-F002 | Merged | FIN-F001 | save validation |
| A03-F003 | Merged | FIN-F003 | forged replay |
| A03-F004 | Unresolved | FIN-F018 | direct race 미재현 |
| A03-F005/F006 | Merged | FIN-F002 | resource/control boundary |
| A03-F008 | Merged | FIN-F017 | durability/permission maintenance |
| A04-F001 | Merged | FIN-F001 | schema test false-green |
| A04-F002/F003/F004/F007 | Merged | FIN-F006 | causal test false-green |
| A04-F005 | Merged | FIN-F014 | Windows 0-test |
| A04-F006 | Merged | FIN-F003 | replay trust |
| A04-F008/F009 | Merged | FIN-F016/FIN-F017 | timing/package-local quality |
| SC-F001/F002 | Merged | FIN-F013 | exception provenance/lifecycle |
| SC-F003/XPF-SC-001 | Merged | FIN-F014 | Windows verifier parity |
| SC-F004/F005/XPF-SC-002 | Merged | FIN-F015 | CI/release authority |
| SC-F006 | Merged | FIN-F012 | current evidence documentation |
| SC-F007 | Merged | FIN-F017 | Info duplicate budget |
| A06-F001/F002/F003/F006/F012 | Merged | FIN-F008 | TUI state/lifecycle/error |
| A06-F004/F005/F009 | Merged | FIN-F009 | layout/accessibility/mouse |
| A06-F007/F008 | Merged | FIN-F016 | LLM contract/i18n |
| A06-F010/F011 | Merged | FIN-F010 | terminal/platform gap |
| A06-F013 | Merged | FIN-F004 | TUI path helper |
| A06-S1-F001/F002/F004 | Merged | FIN-F010 | Windows coverage/restore |
| A06-S1-F003 | Merged | FIN-F009 | mouse capture |
| A06-S1-F005 | Merged | FIN-F008 | Inventory direct evidence |

## 13. Required Actions Before Passing

### P0 — data/security correctness

1. Save semantic validator와 bounded RNG/artifact/event text 경계를 먼저 구현한다.
2. Replay trust model을 확정하고 line-by-line integrity/no-partial-commit을 강제한다.
3. GameSession/GameWorld/EntityStore public DerefMut를 제거한다.
4. ArtifactStore root bootstrap과 TUI storage API를 trusted handle + relative path로 단일화한다.
5. Registry lifetime, armor unequip/drop, causal witness attribution을 수정한다.

### P1 — shipped TUI/platform

6. Inventory/Title/Creation/Awaiting/MorePrompt/NewRun/load/error state machine을 문서와 일치시킨다.
7. layout/theme/high-contrast/reduced-motion/focus/mouse capture를 실제 renderer와 terminal lifecycle에 연결한다.
8. RAII terminal restore와 Windows ConPTY one-event/failure harness를 추가한다.
9. Headless default policy와 turns 범위를 제품 계약으로 확정하고 parser/test를 맞춘다.

### P1 — release/provenance

10. Active audit authority와 gap state를 report24/본 multi-audit 후속 상태로 정렬한다.
11. winx exception ledger/expiry checker와 PROV-0005를 동기화한다.
12. Windows bundle verifier/negative matrix를 Linux와 동등하게 만든다.
13. GitHub Actions를 immutable SHA로 pin하고 exact v0.3.0 candidate의 changelog/modification period를 확정한다.

### P2 — quality/maintenance

14. LLM timeout/rationale/locale contract, timing seam, package-local semantic tests, duplicate budget과 stale 주석을 정리한다.
15. FIN-F018 concurrent hard-link threat model을 결정하고 필요한 경우 재설계·재현한다.

## 14. Accepted and Remaining Risks

- `hallucinating` SaveDataV1 compatibility orphan: 기존 owner/2026-10-31 재검토 조건을 유지한다. 본 감사가 연장 승인하지 않는다.
- 실제 remote LLM provider smoke: spec상 비차단 optional evidence.
- duplicate dependency family: 현재 cargo-deny 정책상 허용된 Info 위험이며 신규 고위험 duplicate는 별도 review가 필요하다.
- 현재 PASS로 수용된 Major risk는 없다.

## 15. Clarifications and Inconclusive Areas

1. Headless default를 `wait-v1`과 `survival-v1` 중 무엇으로 고정할지 결정 필요.
2. Replay를 self-verifying artifact와 command-only log 중 무엇으로 볼지 결정 필요.
3. Save/replay byte/line/event/entity/RNG 상한 수치 결정 필요.
4. AwaitingDirection/Inventory/MorePrompt를 구현할지 deferred schema로 둘지 결정 필요.
5. TUI locale을 5개 언어로 지원할지 English-only로 좁힐지 결정 필요.
6. v0.3.0 candidate에 2026-07-20 이후 R9/보안 변경을 포함할지 결정 필요.
7. Windows Terminal/ConPTY raw-mode와 initialization-failure behavior는 미검증.
8. replay append concurrent hard-link race는 direct reproduction이 없어 `Unverified` 유지.

## 16. Re-audit Checklist

- [ ] malformed/duplicate/dangling SaveData fixture 모두 typed error, panic 0
- [ ] RNG/artifact/event budgets의 경계값 PASS/FAIL
- [ ] tampered replay field matrix nonzero + no partial commit
- [ ] external mutation compile-fail
- [ ] root junction/TUI arbitrary parent reject
- [ ] custom registry corpse lifecycle와 armor wear/drop/rewear 정합성
- [ ] independent causal field attribution/negative matrix
- [ ] TUI state/input/transient/error 실제 one-event flow
- [ ] buffer/PTY/ConPTY layout·contrast·mouse·restore matrix
- [ ] headless default/turn range help+runtime 일치
- [ ] active docs/gap/ADR/changelog current authority 일치
- [ ] exception expiry machine gate + PROVENANCE 일치
- [ ] Windows/Linux bundle negative parity
- [ ] action full-SHA pin + exact release candidate modification evidence
- [ ] fmt, Clippy, 전체 tests, release build, cargo audit/deny, R7/R8 PASS
- [ ] 최종 clean SHA의 Ubuntu/Windows CI와 bundle PASS
- [ ] 외부 게시 별도 사용자 승인

## 17. Final Decision

### **HOLD**

- Critical finding: 0
- Confirmed Major canonical findings: 15
- Minor/Info canonical findings: 2
- Unverified Major candidate: 1
- Unresolved core coverage gap: Windows Terminal/ConPTY failure path

현재 workspace는 valid fixture와 positive build/release path에서 매우 강한 자동화 증거를 가진다. 그러나 이 증거는 malformed persisted input, replay tampering, public mutation surface, entity/content lifecycle, TUI shipped state flow, Windows negative release verification을 닫지 못한다. 특히 메인 직접 실행에서 여러 Major가 실제로 재현되었으므로 전체 테스트 green을 프로젝트 완료나 외부 게시 적합성으로 해석할 수 없다.

## 18. Coder Handoff

```text
`C:\LocalDev\rust\AIHack\docs\multi_audit\1\final_audit_report_1.md`를 먼저 읽고, 각 finding을 프로젝트 문서와 실제 코드에 대조하여 검증한 뒤 우선순위대로 수정하세요. 계약 변경이 필요하면 관련 문서를 먼저 갱신하고, 수정 후 테스트·빌드·재감사 증거를 기록하세요.
```
