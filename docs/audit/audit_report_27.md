# AIHack v0.3.0 감사 보고서 26 시정 독립 재감사 보고서 27

- 감사 대상: `docs/audit/audit_report_26_remediation.md`
- 기준 감사: `docs/audit/audit_report_26.md`
- 프로젝트: `C:\LocalDev\rust\AIHack`
- 감사 일자: 2026-08-24
- 현재 HEAD: `1e84a94aa0623b5cee5349b5832992a4682e93a8`
- 최초 report 26 시정 SHA: `fc01ec12bac522e601bc56bced06b0908f5873b0`
- 중간 evidence SHA: `a9a39d87235109c0fb1d1ea7a31ea3751fd37a30`
- 브랜치: `codex/audit-report-24-remediation`
- 작업 트리: 감사 시작과 probe 정리 후 clean, 최종적으로 이 보고서만 untracked
- 환경: Windows 11 Pro, `x86_64-pc-windows-msvc`, Asia/Seoul
- Rust/Cargo: 1.94.1
- 보안 도구: `cargo-audit 0.22.1`, `cargo-deny 0.19.4`
- 적용 기준: `AI_AUDIT_DOC_STANDARD.md`, `audit_roadmap.md`, `spec.md`, `designs.md`, `AGENTS.md`
- 추가 검토 기준: `code-review-and-quality`, `security-and-hardening`
- 감사 원칙: 구현·테스트·설정·기존 통제 문서는 수정하지 않고 이 보고서만 추가한다.

## 0. 최종 판정

**HOLD — REPORT 26 REMEDIATION PARTIAL / INDEPENDENT PASS 기각**

Report 26 시정은 report 25보다 다시 진전했다. 명시된 malformed score/ItemData/AC 4종, Win32 trailing-name alias, modal mouse, Inspect presentation, key-repeat request, fresh release staging, candidate date, future dependency approval와 YAML AST action scan은 실제 production source와 회귀에 연결됐다. 최종 HEAD `1e84a94`의 Actions `32660514315`도 Ubuntu/Windows 전체 gate와 actual platform bundle을 모두 통과했다.

그러나 열거되지 않은 동등 경계를 독립 검증한 결과 다음 **Confirmed Major 6건**이 남았다.

1. save가 `next_id=u32::MAX`, level depth `i16::MAX`, wand `charges=None`을 수용한다. allocator와 stairs command는 debug panic을 만든다.
2. injected ContentRegistry가 `ac_bonus=i16::MIN` armor를 수용하고 정상 Wear→Drop 뒤 unequipped AC `-1`과 save 불가능 상태를 만든다.
3. “actual producer removal” test가 여러 witness에서 producer/content를 바꾸지 않고 consumer command 또는 observer 호출 자체를 생략한다.
4. 양 release verifier가 `./legacy_nethack_port_reference/` archive alias를 blocked path로 인식하지 않는다.
5. Linux verifier가 `2026-00-00..2026-99-99` 같은 불가능한 modification period를 승인한다.
6. remediation과 active 문서가 최종 fix/CI `1e84a94`/`32660514315`를 기록하지 않고, 스스로 최종 closure에서 제외한 `fc01ec12`/`32658658526`을 Verified evidence로 고정한다.

추가로 **Minor 3건**과 **Major-impact Needs Spec Clarification 1건**이 있다.

- Judge text input에서 uppercase G/A/J/R key repeat가 문자 입력까지 차단된다.
- repository root의 local composite action은 전체 action pin scan 범위 밖이다.
- ADR current authority, changelog와 named test count가 최종 verifier 수정 뒤 갱신되지 않았다.
- F9 debug overlay가 가린 map 영역의 mouse click을 underlying Move/Inspect로 전달하는 동작의 의도가 문서에 없다.

따라서 현재 정확한 상태는 **LOCAL/CI GREEN WITH UNTESTED EQUIVALENT BOUNDARIES / PROGRAM HOLD**다. Report 26 시정의 `Verified` 표는 부분 범위에서만 유효하며 `Closed`, 전체 program PASS 또는 외부 게시 승인으로 승격할 수 없다.

## 1. 감사 범위와 제한

### 1.1 확인한 구현·증거

- `65d8fe5..fc01ec1`: report 26의 12개 boundary 시정
- `fc01ec1..a9a39d8`: implementation evidence 문서와 lifecycle 갱신
- `a9a39d8..1e84a94`: Linux archive scan pipefail/SIGPIPE 후속 수정
- save/content/runtime: registry-aware save validator, arithmetic consumers, entity allocator, stairs와 item lifecycle
- causal: 9-case omission harness, speed/AI/difficulty pair와 witness validator
- TUI: modal mouse, `InspectPresentation`, key event kind, debug overlay와 terminal lifecycle
- release/supply chain: staging/promotion, 양 verifier, source archive, candidate date, dependency/YAML gates
- active 문서와 current CI/evidence lineage

### 1.2 제외 범위

- 외부 실제 LLM provider smoke는 v0.3.0 비목표다.
- Windows Terminal GUI pixel/font rendering은 제외하고 ConPTY·call matrix를 확인했다.
- `legacy_nethack_port_reference/` 본문은 제품 범위 밖이며 runtime import와 release archive 차단만 감사했다.
- 외부 tag/release/publish는 수행하지 않았다.
- runtime same-account concurrent directory-entry swap은 기존 single-writer threat model 밖이다. 이 제외는 save allocator, release archive canonical path 또는 build staging 검증을 면제하지 않는다.

### 1.3 감사 도구 제한

적용한 skill이 참조하는 다음 파일은 설치본에 없었다.

- `code-review-and-quality/references/security-checklist.md`
- `code-review-and-quality/references/performance-checklist.md`
- `security-and-hardening/references/security-checklist.md`

skill 본문 checklist와 프로젝트 감사 표준으로 대체했다. 이는 project finding이 아니라 감사 환경 제한이다.

## 2. 실행·검증 증거

### 2.1 로컬 전체 gate

| 명령 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked -- --list` | named test **432개** |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| `cargo audit` | PASS, 318 dependencies, vulnerability 0 |
| `cargo deny check licenses bans sources` | PASS, bans/licenses/sources ok |
| Git Bash `scripts/r7_checkpoint.sh` | PASS |
| Git Bash `scripts/r8_checkpoint.sh` | PASS |
| clean `build.bat --release` | PASS, commit `1e84a94`, candidate date `2026-08-24` |
| active Markdown relative link scan | PASS, broken 0 |

### 2.2 원격 evidence lineage

| SHA / Actions | Ubuntu | Windows | 판정 |
| --- | --- | --- | --- |
| `fc01ec12` / `32658658526` | PASS | PASS | 구현 positive evidence, 이후 verifier false-green 발견으로 최종 closure 아님 |
| `a9a39d8` / `32660221745` | FAIL | PASS | exact legacy fixture가 Linux pipefail/SIGPIPE 문제 재현 |
| `1e84a94` / `32660514315` | PASS | PASS | current source의 전체 gate와 actual platform bundle success |

현재 HEAD/run은 실제로 존재하고 green이다. 문서 finding은 CI 부재가 아니라 그 최종 evidence를 active authority가 기록하지 않은 문제다.

### 2.3 독립 adversarial probe

```text
NEXT_ID=u32::MAX
HEADLESS_SURVIVAL_EXIT=101
entity.rs:617: attempt to add with overflow

LEVEL_DEPTH=i16::MAX
PLAYER_TILE=StairsDown
REPLAY_COMMAND=Descend
HEADLESS_EXIT=101
stairs.rs:17: attempt to add with overflow

WAND_MAX_CHARGES=Some(3)
WAND_CHARGES=None
LOAD_EXIT=0
FINAL_STATE=Playing
```

```text
CUSTOM_REGISTRY_ARMOR_AC_BONUS=i16::MIN
REGISTRY_ACCEPTED=true
WEAR_AC=32767
DROP_AC=-1
EQUIPPED_BODY=None
NEXT_SAVE_REJECTED=true
```

```text
LINUX_DOT_LEGACY_EXIT=0
WINDOWS_DOT_LEGACY_EXIT=0
ARCHIVE_ENTRY=./legacy_nethack_port_reference/probe

LINUX_INVALID_PERIOD_EXIT=0
PERIOD=2026-00-00..2026-99-99
CANDIDATE_DATE=2026-08-24
```

모든 감사용 fixture와 ignored probe source는 종료 후 정확히 정리했다.

## 3. Report 26 finding 재감사 상태

| Report 26 대상 | 재감사 상태 | 핵심 판정 |
| --- | --- | --- |
| R25-IMP-F001 Re-audit #1 | **Needs Fix** | 명시 4종은 수정, allocator/level/charges 및 custom registry safety 누락 |
| R25-IMP-F002 Re-audit #1 | **Needs Fix** | 사후 label 삭제는 제거했지만 field-specific producer A/B가 아님 |
| R25-IMP-F003 Re-audit #1 | **Verified for enumerated states** | Inventory/StorageError/soft-input/blocking mouse 차단 확인; debug overlay 별도 clarification |
| R25-IMP-F004 Re-audit #1 | **Verified** | renderer/hit-test의 `InspectPresentation` 공유 확인 |
| R25-IMP-F005 Re-audit #1 | **Needs Documentation Recovery** | report 26 status는 정렬됐으나 final SHA/run evidence가 stale |
| R25-DBG-F002 Re-audit #1 | **Verified** | trailing dot/space/reserved/ADS/8.3/hard-link/junction matrix 확인 |
| R25-DBG-F003 Re-audit #1 | **Verified** | terminal source/call matrix/designs boundary 정렬 |
| R26-IMP-F001 | **Partial** | request Repeat 차단은 유효, soft text Repeat 과차단 |
| R25-SEC-F001 Re-audit #1 | **Needs Fix** | staging/hard-link는 수정, archive canonical path 누락 |
| FIN-F015 Re-audit #2 | **Needs Fix** | actual date/notice는 정렬, Linux invalid calendar 승인 |
| R25-SEC-F002 Re-audit #1 | **Verified** | future approval negative 포함 |
| R25-SEC-F003 Re-audit #1 | **Partial** | GitHub YAML AST/Docker pin 개선, external local-action recursion 누락 |

## 4. FIN-F001~F018 재판정

| FIN | 재감사 상태 |
| --- | --- |
| FIN-F001 | **Needs Fix** — allocator/level/charge/custom registry consumer safety |
| FIN-F002 | **Verified** — byte/cardinality/RNG/text와 write no-clobber |
| FIN-F003 | **Verified** — replay self-verification/no-partial-commit |
| FIN-F004 | **Verified** — Windows artifact path alias matrix |
| FIN-F005 | **Verified** — external mutable state 우회 제거 |
| FIN-F006 | **Needs Fix** — producer/content-specific removal evidence 미완료 |
| FIN-F007 | **Needs Fix** — custom registry armor lifecycle가 unsaveable state 생성 |
| FIN-F008 | **Verified for documented modal states / Debug overlay clarification** |
| FIN-F009 | **Partial** — Inspect는 Verified, debug overlay와 soft-input repeat 잔여 |
| FIN-F010 | **Verified** — terminal RAII/ConPTY/evidence 문구 정렬 |
| FIN-F011 | **Verified** — CLI default/range/docs |
| FIN-F012 | **Needs Documentation Recovery** — final evidence SHA/run와 ADR authority drift |
| FIN-F013 | **Verified** — parsed exception/current/future lifecycle |
| FIN-F014 | **Needs Fix** — canonical archive blocked path |
| FIN-F015 | **Needs Fix** — Linux calendar validation, local-action scope와 evidence docs |
| FIN-F016 | **Partial** — signal timing/normal request repeat Verified, Judge text repeat 잔여 |
| FIN-F017 | **Verified** — package tests/duplicate metadata/current 양 OS evidence |
| FIN-F018 | **Verified within frozen threat model** |

## 5. Pass 1 — 구현·문서 정합성 Findings

### [R25-IMP-F001 — Re-audit #2] Save consumer-safe 검증이 allocator·level·dynamic charge를 포함하지 않음

- Related: FIN-F001, FIN-F007
- Pattern: `IMP-001`, `DBG-002`, `TEST-001`, `SEC-004`
- Severity: **Major**
- Status: **Needs Fix**
- Verified Remediation:
  - registry-aware serialized ItemData equality
  - unequipped/equipped AC
  - score range와 widened/saturating score/combat/weight arithmetic
  - 명시된 malformed 4종의 headless exit 2
- Remaining Evidence:
  - `save.rs:564-566`은 allocator `next_id`가 nonzero이고 기존 max ID보다 큰지만 검사한다.
  - `next_id=u32::MAX` save는 load되고 production survival run에서 corpse/item spawn이 `entity.rs:617`의 `next_id += 1` debug panic을 일으켰다.
  - persisted level ID/depth를 active registry 또는 stairs arithmetic headroom과 대조하지 않는다.
  - `depth=i16::MAX`, player on StairsDown save는 load되고 Descend에서 `stairs.rs:17`의 `depth + 1` panic을 일으켰다.
  - `save.rs:555-556`은 `charges=Some`인 경우만 max와 비교한다. registry `max_charges=Some(3)` wand의 `charges=None`은 load되고 영구 `no charge state`가 된다.
- Expected: 성공 load된 persisted state는 모든 정상 allocator/level/item consumer에서 panic·wrap·불가능 optional state를 만들지 않아야 한다.
- Actual: report 26의 consumer-safe 범위가 score/ItemData fixed fields에 한정되어 다음 consumer가 열려 있다.
- Impact: process crash, allocator wrap, 불가능한 level transition, 정상 wand 기능 상실.
- Suggested Fix:
  1. `next_id < u32::MAX` 또는 checked allocator exhaustion error를 강제한다.
  2. LevelId 집합/depth를 active registry와 대조하고 stairs는 checked depth arithmetic을 사용한다.
  3. `max_charges.is_some() == charges.is_some()`과 `charges <= max`를 양방향 검증한다.
  4. load→spawn/Descend/Zap을 debug/release production entrypoint matrix로 추가한다.
- Re-audit: 세 save가 load 단계 typed reject 또는 정상 typed consumer error이고 panic/wrap이 없는지 확인한다.

### [R27-IMP-F001] Injected ContentRegistry가 비가역 armor arithmetic을 허용함

- Related: FIN-F001, FIN-F007
- Pattern: `IMP-001`, `DBG-002`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Evidence:
  - `aihack-content/src/schema.rs:209-224`는 item price와 dice는 검사하지만 armor `ac_bonus` consumer range를 검사하지 않는다.
  - `ac_bonus=i16::MIN` custom registry가 정상 생성됐다.
  - Wear는 base AC 0에서 saturating subtraction으로 32767, Drop은 saturating addition으로 -1을 만들었다.
  - equipment pointer는 None이지만 AC는 base와 다르며 command transaction은 기존 6 invariant를 통과했다. 다음 save round-trip만 validator에서 실패했다.
- Expected: runtime이 신뢰하는 registry 자체가 save validator와 같은 consumer-safe 계약을 만족하고 Wear/Drop이 reversible해야 한다.
- Actual: persisted ItemData가 registry와 정확히 같다는 검사가 오히려 unsafe registry 값을 신뢰한다.
- Impact: 정상 injected-content command가 live session을 unsaveable/inconsistent state로 만든다.
- Suggested Fix:
  1. content schema에서 armor AC bonus, weight, price, charges/effects와 모든 numeric consumer bound를 검증한다.
  2. equip/unequip은 이전 결과의 saturating inverse가 아니라 base/현재 equipment에서 derived AC를 재계산한다.
  3. custom registry extreme matrix에 registry reject와 Wear→Drop→save round-trip을 추가한다.
- Re-audit: 모든 accepted registry에서 equipment lifecycle 후 base AC와 saveability가 복원되는지 확인한다.

### [R25-IMP-F002 — Re-audit #2] 9-case omission이 field-specific producer A/B를 증명하지 않음

- Related: FIN-F006
- Pattern: `IMP-003`, `DBG-002`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Verified Remediation:
  - `CausalSummary::without` 사후 record 삭제 제거
  - 9회 full fixture 실행과 exactly-one missing label/count
  - 반복 summary/hash 확인
- Remaining Evidence:
  - Food/Corpse/Armor/Prayer omission은 content/state producer 값을 바꾸지 않고 consumer command를 생략한다.
  - Speed/AI/Difficulty omission은 neutralized pair를 실행하지 않고 `observe_*_pair` 호출 전체를 생략한다.
  - Difficulty positive pair는 active가 Attack, control이 Wait이고 두 monster difficulty 값은 동일하다. reward가 difficulty와 우연히 같은 hardcoded 값이어도 통과할 수 있다.
  - 나머지 8개 record의 producer/field/source/consumer structural value가 complete run과 같은지도 비교하지 않는다.
- Expected: 같은 command/consumer를 유지한 채 해당 content field 또는 producer state만 A/B로 제거·변경해야 한다.
- Actual: “그 witness를 생성할 코드 경로를 호출하지 않음”이 producer isolation을 대신한다.
- Impact: field가 실제 consumer에 영향을 주지 않아도 test harness 분기로 exactly-one label 결과를 만들 수 있다.
- Suggested Fix:
  1. Food/Corpse/Armor는 같은 Eat/Wear command에서 nutrition/AC field만 neutralize한다.
  2. Prayer는 같은 Pray+attack flow에서 luck producer만 neutralize한다.
  3. Speed/AI/Difficulty는 observer까지 동일 실행하고 content field만 active/control에서 바꾼다.
  4. Difficulty는 양쪽 모두 동일 kill을 수행하고 difficulty 차이와 gold delta 차이를 대조한다.
  5. 비누락 8개 record의 전체 attribution value가 complete와 같은지 검사한다.
- Re-audit: 각 field-only A/B full run이 exactly-one witness loss와 나머지 record equality를 보장하는지 확인한다.

### [R26-DOC-F001] 최종 verifier fix SHA/run이 current authority에 기록되지 않음

- Related: R25-IMP-F005, FIN-F012, FIN-F015
- Pattern: `IMP-004`, `BUILD-001`, `DOC-BACKFILL-001`
- Severity: **Major**
- Status: **Needs Documentation Recovery**
- Evidence:
  - remediation `:100-102`는 `fc01ec12/32658658526`을 최종 closure로 사용하지 않고 새 clean same-SHA run을 요구하며 끝난다.
  - 실제 후속 fix는 `1e84a94`, Actions `32660514315`에서 양 OS success다.
  - spec, designs, README, BUILD, ADR, GAP, summary, audit roadmap, documentation audit와 `tests/r8_documentation.rs`는 모두 invalidated 이전 `fc01ec12/32658658526`을 Verified evidence로 고정하고 final SHA/run을 전혀 기록하지 않는다.
  - remediation에 3.7 resolution이 없고 target 문서의 named test 431개도 current 432개보다 하나 적다.
- Expected: 실패 evidence, fix SHA와 최종 same-SHA CI가 하나의 successor chain으로 active 문서와 tests에 반영되어야 한다.
- Actual: 실제 CI는 green이지만 문서상 요구된 final evidence는 계속 pending인 채 이전 run을 current로 표시한다.
- Impact: release/audit authority의 self-contradiction과 false-green 문서 gate.
- Suggested Fix:
  1. remediation 3.7에 `1e84a94/32660514315`과 exact blocked-path regression 결과를 기록한다.
  2. `fc01ec12`는 partial, `a9a39d8`은 failed historical evidence로 분류한다.
  3. 모든 active 문서, child/aggregate gap과 document tests를 final SHA/run으로 갱신한다.
  4. report 27의 새 HOLD와 432 test count를 반영한다.
- Re-audit: current SHA/run exact assertion과 stale predecessor absence를 section-aware로 확인한다.

### [R27-IMP-F002] F9 debug overlay의 mouse authority가 불명확함

- Related: FIN-F008, FIN-F009
- Pattern: `IMP-001`, `SPEC-GAP-001`, `TEST-001`
- Severity: **Major impact if click-through is unintended**
- Status: **Needs Spec Clarification**
- Evidence:
  - designs와 source는 F9 view를 `Debug overlay`라고 부르고 실제 map 위 `x=12..51, y=0..19` 영역을 덮는다.
  - mouse guard는 UiOverlay, soft input과 core blocking state만 검사하며 `debug_observation_visible`은 포함하지 않는다.
  - debug text 위 Down/Move event는 underlying hidden map의 Move/Inspect/hover candidate로 전달될 수 있다.
  - active mouse authority 목록은 debug overlay를 열거하지 않는다.
- Expected: debug panel이 modal인지, panel rect만 mouse를 차단하는지, transparent click-through인지 문서가 정해야 한다.
- Impact: unintended라면 보이지 않는 map tile click으로 player turn/position이 변경될 수 있다.
- Suggested Fix: authority를 문서에 동결하고, overlay라면 최소한 visible debug rect의 mouse를 consume하며 buffer/hit regression을 추가한다.
- Re-audit: F9 visible/hidden 각각 동일 coordinate candidate와 core revision을 검증한다.

### [R27-DOC-F001] active ADR와 changelog에 final successor 상태가 남지 않음

- Related: FIN-F012, FIN-F015
- Pattern: `IMP-004`, `DOC-BACKFILL-001`
- Severity: **Minor**
- Status: **Needs Documentation Recovery**
- Evidence:
  - `DESIGN_DECISIONS.md:466`은 여전히 `report 25 current HOLD`라고 적지만 최신 ADR과 나머지 문서는 report 26을 current로 둔다.
  - CHANGELOG에는 `1e84a94`의 pipefail/SIGPIPE false-green 수정이 없다.
  - remediation full gate count는 431이지만 current list는 432다.
- Suggested Fix: 동적 audit status를 최신 successor로 갱신하거나 ADR decision 본문과 분리하고 changelog/test count를 동기화한다.
- Re-audit: current-authority negative와 release verifier fix changelog assertion을 추가한다.

## 6. Pass 2 — Debug·Engineering Quality Findings

### [R26-IMP-F001 — Re-audit #1] Request Repeat guard가 Judge text Repeat까지 차단함

- Related: FIN-F009, FIN-F016
- Pattern: `IMP-001`, `TEST-001`
- Severity: **Minor**
- Status: **Needs Fix**
- Verified Remediation: 일반 Playing 상태에서 G/A/J/R Press만 request candidate를 만들고 Repeat/Release는 None이다.
- Remaining Evidence:
  - G/A/J/R Repeat guard가 soft-input branch보다 먼저 실행된다.
  - Judge text input에서 uppercase G/A/J/R을 누르고 있으면 첫 Press만 입력되고 Repeat 문자는 삭제된다.
  - 현재 repeat test는 soft input이 없는 app만 사용한다.
- Expected: request candidate의 Repeat만 거부하고 text editor의 정상 character repeat는 허용해야 한다.
- Suggested Fix: soft-input branch 뒤에서 request key kind를 필터하거나 생성된 candidate가 request일 때만 Repeat를 거부한다.
- Re-audit: Judge input의 Press/Repeat/Release 문자열과 일반 Playing request ID count를 함께 검증한다.

## 7. Pass 3 — Security·Supply Chain Findings

### [R25-SEC-F001 — Re-audit #2] Blocked archive path가 canonical form으로 검사되지 않음

- Related: FIN-F014
- Pattern: `SEC-004`, `SEC-006`, `BUILD-001`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Verified Remediation:
  - fresh random staging과 directory promotion
  - output root/nested reparse와 expected hardlink 거부
  - 정확한 prefix의 legacy path 및 pipefail/SIGPIPE 수정
- Remaining Evidence:
  - Linux `verify_release_bundle.sh:90`과 Windows verifier `:168`은 raw archive entry에 `^(legacy_nethack_port_reference|target|output)/`만 적용한다.
  - required entries는 정상이고 blocked tree만 `./legacy_nethack_port_reference/`인 완전 bundle을 양 verifier가 exit 0으로 승인했다.
  - 반복 `./`, parent/absolute와 separator canonicalization negative가 없다.
- Expected: archive entry를 lexical canonical form으로 바꾸고 absolute, parent traversal, platform separator와 blocked first segment를 fail-closed로 검사해야 한다.
- Actual: 같은 blocked path의 표기 alias가 exact-prefix policy를 우회한다.
- Impact: 격리된 legacy/target/output tree가 source archive에 포함돼도 verifier PASS 가능.
- Suggested Fix: archive list를 완전히 읽은 뒤 각 entry를 component 단위 normalize하고 unsafe/canonical blocked path를 양 OS 공통 fixture로 거부한다.
- Re-audit: `./`, repeated `./`, `a/../`, absolute, backslash와 exact prefix matrix를 양 verifier에서 실행한다.

### [FIN-F015 — Re-audit #3] Linux modification period가 실제 calendar로 검증되지 않음

- Related: FIN-F015
- Pattern: `SEC-006`, `BUILD-001`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Verified Remediation:
  - current notice ID/date와 candidate commit은 2026-08-24로 정렬
  - Windows DateTime parser와 stale candidate negative
- Remaining Evidence:
  - Linux verifier는 date-looking regex와 lexical comparison만 사용한다.
  - `Covered change period: 2026-00-00..2026-99-99`와 valid candidate `2026-08-24`를 가진 동기화 bundle을 exit 0으로 승인했다.
- Expected: 시작/종료/candidate가 실제 Gregorian calendar date이고 start <= candidate <= end여야 한다.
- Actual: 불가능한 calendar value가 broad lexical range로 정책을 우회한다.
- Impact: Linux release modification provenance false-green과 Windows parity 위반.
- Suggested Fix: Linux에서 strict parse+round-trip을 수행하고 invalid month/day/leap/start-after-end matrix를 추가한다.
- Re-audit: 양 OS가 같은 calendar fixture를 동일하게 거부/허용하는지 확인한다.

### [R25-SEC-F003 — Re-audit #2] Repository-root local action의 transitive `uses`가 scan되지 않음

- Related: FIN-F015
- Pattern: `SEC-006`, `BUILD-001`, `TEST-001`
- Severity: **Minor**
- Status: **Partial Verified / Needs Fix**
- Verified Remediation:
  - `.github/**/*.yml|yaml` AST traversal
  - inline/spaced/nested refs, remote SHA와 Docker digest
  - YAML anchor/alias는 parser가 expanded mapping으로 수집함
- Remaining Evidence:
  - scanner root는 `.github` 하나다.
  - `./actions/local`은 즉시 허용되지만 repository root `actions/local/action.yml` 내부 mutable `uses`는 scan되지 않는다.
  - 현재 repository에는 local action reference가 없어 current pins 자체는 안전하다.
- Expected: local action path를 repository 내부로 resolve하고 action metadata의 transitive uses를 cycle-safe 재귀 검사해야 한다.
- Suggested Fix: missing/escape/local action metadata를 거부하고 `.github` 밖 local composite fixture를 추가한다.
- Re-audit: local action→local action→mutable remote chain과 cycle/escape를 검사한다.

## 8. Cross-Pass Conflicts

| Conflict | 해소 판단 |
| --- | --- |
| 432개 named test와 양 OS CI green vs allocator/stairs debug panic | consumer 열거 누락이므로 save finding 유지 |
| ItemData가 registry와 exact match하지만 registry 자체가 unsafe | registry validation이 먼저 consumer-safe해야 함 |
| 9-case full run이 exactly-one label을 잃지만 harness가 code path를 생략 | field A/B가 아니므로 causal closure 기각 |
| fresh staging/hardlink boundary는 green이나 archive `./` alias 허용 | staging Verified, archive canonicalization Needs Fix |
| candidate date/notice 값은 current와 일치하지만 Linux invalid dates 허용 | current positive Verified, policy validator Needs Fix |
| current HEAD CI는 green이나 active docs는 이전 invalidated run을 current로 고정 | 운영 evidence 존재, document authority Needs Recovery |
| TUI 명시 modal은 fixed지만 debug overlay authority 미정 | enumerated finding Verified, debug는 spec clarification 분리 |

## 9. Verified로 유지하는 개선

- Report 26이 열거한 score/ItemData mutation, unequipped/equipped AC와 widened arithmetic
- Windows trailing dot/space/reserved/ADS/8.3/hard-link/junction artifact path matrix
- `InspectPresentation` renderer/hit-test 공유
- Inventory/StorageError/soft-input/core blocking state의 mouse 차단
- terminal RAII, Windows Console API/ANSI evidence 경계와 ConPTY
- future dependency approval 거부
- YAML AST parsing, current remote action SHA와 Docker digest
- fresh release staging, output root/nested reparse와 expected hardlink 거부
- current candidate date/notice and Windows verifier
- replay self-verification, mutable state encapsulation과 FIN-F018 atomic rewrite
- current HEAD `1e84a94` Actions `32660514315` 양 OS success
- active Markdown relative link 0 broken

## 10. Rejected/Clarified 후보

- GitHub Actions는 YAML anchor/alias를 공식 지원한다. 감사 probe에서 saphyr는 alias mapping을 확장해 동일 mutable `uses`를 두 번 수집했으므로 anchor 우회 후보는 기각한다. 근거: [GitHub 공식 YAML anchors 문서](https://docs.github.com/en/actions/reference/workflows-and-actions/reusing-workflow-configurations#yaml-anchors-and-aliases).
- `CLOCK$`, `COM0`, `LPT0`, `CON ` 등 추가 Windows 이름은 현재 host에서 distinct regular file이거나 기존 trailing-space guard로 안전하게 거부되어 same-file alias finding으로 채택하지 않았다.
- actor immutable registry equality는 현재 spec가 ItemData에만 exact equality를 요구하고 widened consumers가 immediate overflow를 막는다. Save를 anti-tamper artifact로 확장할지 별도 제품 계약이 필요하다.

## 11. PASS 전 필수 수정 순서

### P0 — Confirmed Major

1. save allocator headroom, level identity/depth와 charge presence를 fail-closed로 닫는다.
2. injected content registry의 armor/numeric consumer safety와 equipment reversibility를 보장한다.
3. causal 9종을 동일 consumer command의 field-only A/B와 structural record equality로 검증한다.
4. archive entry를 canonical component로 검사해 blocked path aliases를 양 OS에서 거부한다.
5. Linux modification dates에 strict calendar parsing/parity를 적용한다.
6. 최종 `1e84a94/32660514315`과 report 27 HOLD를 active authority에 동기화한다.

### P1 — Minor/Clarification

7. F9 debug overlay mouse authority를 결정하고 visible rect hit behavior를 고정한다.
8. Judge text character repeat와 LLM request repeat를 분리한다.
9. repository root local composite action을 재귀 scan한다.
10. ADR-0030 status, changelog와 432 test count를 최종 successor에 맞춘다.

## 12. Accepted Risks와 남은 제한

- runtime same-account concurrent directory-entry swap은 기존 single-writer threat model 밖이다.
- Windows parent-directory power-loss durability는 OS/filesystem 잔여 위험이다.
- 실제 provider smoke와 Windows Terminal GUI rendering은 비차단 제외 범위다.
- artifact signing/attestation/upload는 현재 v0.3.0 필수 범위가 아니다.
- 위 accepted risk는 save allocator/level, unsafe registry, archive alias, invalid calendar나 stale audit evidence를 허용하지 않는다.

## 13. 재감사 체크리스트

1. `next_id=max`, level depth max, wand charge-none save가 load 전에 typed reject되거나 safe typed consumer error가 된다.
2. extreme custom registry가 bootstrap에서 거부되고 모든 accepted armor의 Wear→Drop→save가 reversible하다.
3. 9개 causal field-only A/B가 exactly-one missing witness와 나머지 record equality를 만든다.
4. archive dot/parent/absolute/backslash alias matrix가 양 verifier에서 nonzero다.
5. Linux/Windows calendar invalid/leap/range matrix가 동일하다.
6. F9/debug와 Judge repeat의 결정된 UI contract가 buffer/candidate/revision test에 있다.
7. local action transitive mutable ref가 실패한다.
8. remediation 3.7과 모든 active docs가 final SHA/run, report 27 HOLD와 432 tests를 기록한다.
9. 아래 전체 gate를 단독 실행한다.

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

10. 새 clean commit의 Ubuntu/Windows same-SHA actual bundle을 확인한다.
11. 새 독립 감사가 report 27 finding과 FIN-F001~F018을 연결해 판정한다.

## 14. 최종 근거와 Coder Handoff

### 최종 근거

- Report 26의 12개 항목을 모두 재대조했고 여러 하위 경계는 Verified됐다.
- 그러나 save/content, causal attribution, release canonicalization/calendar와 current authority에서 Confirmed Major 6건이 남았다.
- F9 debug overlay는 의도에 따라 Major가 될 수 있어 명세를 먼저 닫아야 한다.
- 따라서 `docs/audit/audit_report_26_remediation.md`의 전체 local finding Verified 주장은 부분적으로만 유효하며 program/publication HOLD를 유지한다.

### Coder Handoff

```text
`C:\LocalDev\rust\AIHack\docs\audit\audit_report_27.md`의 독립 재감사 결과를 확인하고,
각 finding을 current spec/ADR, 실제 production entrypoint와 adversarial fixture에 대조하여 수정하세요.
문서를 먼저 갱신한 뒤 save allocator/level/charge와 custom registry, field-only causal A/B,
archive canonical path, strict calendar, debug mouse, Judge repeat와 local action recursion을 회귀로 보존하세요.
수정 후 전체 로컬 gate와 새 clean same-SHA Ubuntu/Windows actual bundle을 실행하고 결과를 기록하세요.
```
