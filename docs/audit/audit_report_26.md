# AIHack v0.3.0 감사 보고서 25 시정 독립 재감사 보고서 26

- 감사 대상: `docs/audit/audit_report_25_remediation.md`
- 기준 감사: `docs/audit/audit_report_25.md`
- 프로젝트: `C:\LocalDev\rust\AIHack`
- 감사 일자: 2026-08-24
- 현재 HEAD: `65d8fe5a7fa418794d050e318931b12133bbf616`
- 구현 SHA: `b732c42d62f295f4d8be64480c1d0a5a440fe738`
- 브랜치: `codex/audit-report-24-remediation`
- 작업 트리: 감사 시작 및 probe 정리 후 clean, 최종적으로 이 보고서만 untracked
- 환경: Windows 11 Pro, `x86_64-pc-windows-msvc`, Asia/Seoul
- Rust/Cargo: 1.94.1
- 보안 도구: `cargo-audit 0.22.1`, `cargo-deny 0.19.4`
- 적용 기준: `AI_AUDIT_DOC_STANDARD.md`, `audit_roadmap.md`, `spec.md`, `designs.md`, `AGENTS.md`
- 추가 검토 기준: `code-review-and-quality`, `security-and-hardening`
- 감사 원칙: 구현·테스트·설정·기존 통제 문서는 수정하지 않고 이 보고서만 추가한다.

## 0. 최종 판정

**HOLD — REPORT 25 REMEDIATION PARTIAL / INDEPENDENT PASS 기각**

Report 25 시정은 이전보다 크게 개선됐다. reader/writer 공통 save budget, inverse inventory와 HP/alive 검증, production GoldScore pair, keyboard dispatcher, 최소 화면 prompt, command CTA geometry, terminal RAII, actual release entry exact-set, parsed dependency ledger, semantic package tests, immutable action pin과 양 OS CI는 실제 source·tests·실행 증거로 확인됐다.

그러나 독립 재감사는 기존 회귀가 열거하지 않은 동등 경계를 공격했고 다음 Major를 재현했다.

1. unequipped player AC, persisted `turn`/`kill_count`/`ItemData`가 validator를 통과해 debug panic 또는 release wraparound를 만든다.
2. Windows trailing dot/space path alias가 replay input/output 동일 파일 guard를 우회해 원 trace를 교체한다.
3. causal all-witness removal이 actual producer를 제거하지 않고 완성된 summary의 label을 삭제하는 tautology다.
4. Inventory, StorageError, soft judgment modal 위 mouse click이 underlying `Wait`를 제출하고 turn을 진행한다.
5. Inspect panel이 hover/LLM 내용을 표시할 때도 숨은 inventory CTA가 활성화되어 `Wield` 같은 보이지 않는 command를 반환한다.
6. release verifier와 build script가 output root junction 및 expected-name hardlink를 신뢰하며 Linux `cp` 경로에서 외부 inode를 변경한다.
7. active master spec가 완료된 GoldScore 시정을 여전히 “시정 중”으로 기록한다.
8. 2026-08-24 commit을 포함하는 release archive가 2026-08-23 종료 modification notice로 PASS한다.

Finding 집계는 **Critical 0, Major 8, Minor 4**다. Major가 남아 있으므로 clean same-SHA 양 OS CI가 성공했어도 전체 program, R8 release 또는 외부 게시를 PASS로 올릴 수 없다. 현재 정확한 상태는 **LOCAL/CI GREEN WITH UNTESTED PRODUCTION GAPS / PROGRAM HOLD**다.

## 1. 감사 범위와 제한

### 1.1 확인한 변경·증거 범위

- `80d959a..b732c42`: report 25 production 시정과 Linux parent-fsync 후속 수정
- `b732c42..65d8fe5`: active 문서와 `tests/r8_documentation.rs` evidence 갱신
- 저장·경로: `crates/aihack-runtime/src/save.rs`, `session.rs`, `world.rs`, `apps/aihack-headless/*`
- 인과·점수: `causal.rs`, runtime/core `score.rs`, `tests/causal_content.rs`, `tests/long_run.rs`
- TUI·LLM·terminal: `apps/aihack-tui/src/tui/*`, ConPTY/package/root UI·LLM tests
- release·공급망: 양 verifier, `build.sh`, `build.bat`, CI workflow, dependency ledgers/gates, license/provenance tests
- active 문서: `spec.md`, `designs.md`, README, BUILD, ADR, GAP, implementation summary, audit roadmap, changelog, modification/metadata records
- 원격 evidence: 구현 SHA Actions `32650404618`, 현재 HEAD Actions `32651576393`

### 1.2 제외 범위

- 외부 실제 LLM provider smoke는 v0.3.0 비목표라 제외했다.
- Windows Terminal GUI pixel/font rendering은 제외하고 ConPTY와 lifecycle call matrix를 확인했다.
- `legacy_nethack_port_reference/` 본문은 제품 범위 밖이며 direct import/path dependency gate만 확인했다.
- 외부 tag/release/publish는 수행하지 않았다.
- runtime의 같은 계정 악성 concurrent directory-entry swap은 문서화된 single-writer threat model 밖으로 유지했다. 이 제외는 build/release staging root와 destination inode를 신뢰해도 된다는 의미가 아니다.

### 1.3 감사 도구 제한

적용한 두 skill이 참조한 다음 세부 checklist 파일은 설치본에 존재하지 않았다.

- `code-review-and-quality/references/security-checklist.md`
- `code-review-and-quality/references/performance-checklist.md`
- `security-and-hardening/references/security-checklist.md`

따라서 각 skill 본문에 포함된 checklist와 프로젝트 `AI_AUDIT_DOC_STANDARD.md`를 사용했다. 이 누락은 project finding이 아니라 감사 환경 제한이다.

## 2. 실행·검증 증거

### 2.1 로컬 전체 gate

| 명령 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | 단독 재실행 PASS |
| `cargo test --workspace --all-targets --locked -- --list` | named test 416개 |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| `cargo audit` | PASS, 312 dependencies, vulnerability 0 |
| `cargo deny check licenses bans sources` | PASS, bans/licenses/sources ok |
| Git Bash `scripts/r7_checkpoint.sh` | PASS |
| Git Bash `scripts/r8_checkpoint.sh` | PASS |
| clean `build.bat --release` | PASS, commit `65d8fe5`, 9-entry Windows output |
| 전체 active Markdown 상대 링크 scan | PASS, broken 0 |

첫 전체 test는 여러 sub-audit cargo/process가 동시에 실행되는 동안 `provenance_manifest`의 diagnostic 문자열 assertion 1개가 실패했다. 동일 exact test, 전체 `provenance_manifest` 14개와 두 번째 단독 workspace/all-target 실행은 모두 PASS했다. 해당 최초 결과는 감사 실행 간섭으로 분류하되 실패 명령 이력에서 숨기지 않는다.

### 2.2 원격 same-SHA evidence

| SHA / Run | Ubuntu | Windows | 범위 |
| --- | --- | --- | --- |
| `02050fe` / `32648979651` | FAIL | PASS | 최초 시정, Linux parent fsync EBADF 재현 |
| `b732c42` / `32650404618` | PASS | PASS | 최종 구현, actual platform bundle 포함 |
| `65d8fe5` / `32651576393` | PASS | PASS | 현재 문서·회귀 HEAD, actual platform bundle 포함 |

양 OS CI 성공은 유효한 positive evidence다. 다만 아래 finding은 CI fixture가 생성하지 않은 malformed scalar, Windows name alias, modal mouse 조합, preplaced staging link와 calendar boundary다.

### 2.3 독립 production probe 관찰값

감사용 probe는 ignored `target/` 또는 임시 fixture에서 실행했고 종료 후 생성물을 정확히 정리했다.

```text
UNEQUIPPED_BODY=true
MUTATED_PLAYER_AC=-1
HEADLESS_LOAD_EXIT=0
FINAL_STATE=Playing

EXTREME_KILL_COUNT_ACCEPTED=true
DEBUG_QUIT_SCORE_PANIC=true
RELEASE_QUIT_SCORE_PANIC=false
RELEASE_FINAL_SCORE=375

MAX_TURN_ACCEPTED=true
DEBUG_NEXT_TURN_PANIC=true
RELEASE_NEXT_TURN_PANIC=false
RELEASE_TURN_AFTER_WAIT=0

EXTREME_ITEM_DATA_ACCEPTED=true
DEBUG_ITEM_PRICE_SCORE_PANIC=true
RELEASE_ITEM_PRICE_SCORE_PANIC=false
RELEASE_ITEM_FINAL_SCORE=380
```

```text
WINDOWS_REPLAY_DOT_ALIAS_EXIT=0
WINDOWS_REPLAY_SPACE_ALIAS_EXIT=0
ONLY_ENTRY_AFTER=run.jsonl
INPUT_SHA_OR_CREATION_ID_CHANGED=true
```

```text
INVENTORY_OVERLAY_MOUSE_CANDIDATE=Some(Command(Wait))
INVENTORY_OVERLAY_MOUSE_ADVANCED=true
STORAGE_OVERLAY_MOUSE_CANDIDATE=Some(Command(Wait))
STORAGE_OVERLAY_MOUSE_ADVANCED=true
SOFT_MODAL_MOUSE_CANDIDATE=Some(Command(Wait))
SOFT_MODAL_MOUSE_ADVANCED=true
HOVER_INSPECT_HIDDEN_CANDIDATE=Some(Command(Wield { item: EntityId(5) }))
```

```text
ROOT_JUNCTION_VERIFIER_EXIT=0
EXPECTED_HARDLINK_VERIFIER_EXIT=0
BASH_CP_EXIT=0
OUTSIDE_VICTIM_CHANGED=true
LINUX_VERIFIER_EXPECTED_HARDLINK_EXIT=0
```

## 3. Report 25 finding 재감사 상태

| Report 25 Finding | 재감사 상태 | 핵심 판정 |
| --- | --- | --- |
| R25-IMP-F001 | **Needs Fix** | 기존 inverse/HP/ac-bonus는 개선됐지만 unequipped AC와 scalar/ItemData 산술 경계 잔존 |
| R25-DBG-F001 | **Verified** | writer semantic 검사, 16 MiB capped serialization, no-clobber 확인 |
| R25-DBG-F002 | **Needs Fix** | `./`/case는 수정됐지만 Windows trailing dot/space alias 우회 |
| R25-IMP-F002 | **Needs Fix** | production paired score는 수정됐지만 actual producer-removal evidence가 아님 |
| R25-IMP-F003 | **Needs Fix** | keyboard/late response는 수정됐지만 mouse가 modal guard 우회 |
| R25-IMP-F004 | **Needs Fix** | prompt/command CTA는 수정됐지만 Inspect presentation과 hit model 분리 |
| R25-SEC-F001 | **Needs Fix** | extra entry는 거부하지만 staging root와 expected hardlink 미검증 |
| R25-IMP-F005 | **Needs Documentation Recovery** | lifecycle 대부분 정렬됐지만 active master spec에 stale “시정 중” 남음 |
| R25-DBG-F003 | **Partially Verified / Needs Documentation Recovery** | production RAII 구조는 유효하나 evidence 표현과 designs가 불일치 |
| R25-DBG-F004 | **Verified** | response signal/Condvar 기반으로 semantic busy polling 제거 |
| R25-DBG-F005 | **Verified** | runtime package semantic tests, stale 주석, duplicate metadata, 양 OS evidence 확인 |
| R25-SEC-F002 | **Needs Fix (Minor)** | parsed TOML/exact trigger/date는 개선됐지만 future approval 수용 |
| R25-SEC-F003 | **Partial** | 현재 pins/CI는 Verified, general all-workflow gate는 우회 가능 |

## 4. FIN-F001~F018 재판정

| FIN | 재감사 상태 |
| --- | --- |
| FIN-F001 | **Needs Fix** — persisted scalar/ItemData와 unequipped AC |
| FIN-F002 | **Verified** — read/write/RNG/event/entity/text budgets와 no-clobber |
| FIN-F003 | **Verified** — replay 7-field self-verification/no-partial-commit |
| FIN-F004 | **Needs Fix** — Windows trailing-name alias |
| FIN-F005 | **Verified** — mutable public state 우회 제거 |
| FIN-F006 | **Needs Fix** — actual producer-removal matrix 미완료 |
| FIN-F007 | **Needs Fix** — 정상 registry/equipment lifecycle는 통과, unequipped derived AC는 열림 |
| FIN-F008 | **Needs Fix** — modal/overlay mouse가 underlying core command 제출 |
| FIN-F009 | **Needs Fix** — Inspect hidden CTA와 key-repeat 계약 |
| FIN-F010 | **Partially Verified / Needs Documentation Recovery** |
| FIN-F011 | **Verified** — default/range/help/BUILD 일치 |
| FIN-F012 | **Needs Documentation Recovery** — active spec status drift |
| FIN-F013 | **Needs Fix (Minor)** — future approval lifecycle |
| FIN-F014 | **Needs Fix** — release staging root/hardlink |
| FIN-F015 | **Needs Fix** — modification period와 commit date, general action gate |
| FIN-F016 | **Partial** — timing synchronization은 Verified, LLM key repeat는 미강제 |
| FIN-F017 | **Verified** — package semantics/maintenance metadata/Unix CI |
| FIN-F018 | **Verified within frozen threat model** |

## 5. Pass 1 — 구현·문서 정합성 Findings

### [R25-IMP-F001 — Re-audit #1] Save가 consumer-safe scalar와 unequipped derived state를 검증하지 않음

- Related: FIN-F001, FIN-F007
- Pattern: `IMP-001`, `DBG-002`, `TEST-001`, `SEC-004`
- Severity: **Major**
- Status: **Needs Fix**
- Verified Remediation:
  - `save.rs:568-603`의 HP/alive/max 관계
  - `save.rs:622-666`의 inventory 양방향 관계
  - `save.rs:500-515,669-686`의 armor bonus range와 checked arithmetic
- Remaining Evidence:
  - `save.rs:669-687`은 `equipped_body=Some`일 때만 player AC를 검증한다. body가 `None`인 save의 AC를 `0 -> -1`로 변조한 production headless load가 exit 0/Playing이었다.
  - `save.rs:384-411`은 `turn`, `kill_count`, `gold`와 consumer arithmetic bound를 검사하지 않는다.
  - `turn=u64::MAX`는 load 후 debug `session.rs:306`에서 `turn + 1` panic, release에서는 turn 0으로 wrap했다.
  - `kill_count=i32::MAX`는 load 후 Quit에서 debug `aihack-core/src/score.rs:14` multiply overflow, release에서는 정상 범위처럼 보이는 score 375를 만들었다.
  - persisted ItemData는 kind/ac_bonus/charges만 검증한다. 두 item의 `base_price=u32::MAX`를 수용했고 debug inventory score sum이 panic, release는 score 380으로 wrap했다.
  - 같은 표면에는 item weight, nutrition, dice, damage/hit과 `kill_count += 1` 등 좁은 정수 consumer가 더 존재한다.
- Expected: untrusted SaveDataV1을 성공 load한 session은 다음 정상 command/observation에서 panic 또는 wraparound를 만들지 않아야 한다.
- Actual: loader validity와 runtime arithmetic validity가 분리되어 있다.
- Impact: crash, turn rollback to 0, forged score/AC, save/replay hash와 게임 규칙 손상.
- Suggested Fix:
  1. unequipped body에서는 player AC가 current base AC와 일치하도록 검사한다.
  2. `turn`, economy와 ItemData를 registry-aware equality 또는 명시적 consumer-safe range로 검증한다.
  3. score, turn increment, weight/value sum과 death counter도 넓은 정수형 checked/saturating policy로 방어한다.
  4. 모든 malformed scalar를 debug/release에서 typed reject하고 load-success→observation/next command/quit matrix를 추가한다.
- Re-audit: 위 probe가 모두 `InvalidSave`, panic 0, release wrap 0인지 확인한다.

### [R25-IMP-F002 — Re-audit #1] causal removal test가 actual producer를 제거하지 않음

- Related: FIN-F006
- Pattern: `IMP-003`, `DBG-002`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Verified Remediation:
  - `systems/score.rs:9-15`의 실제 gold/no-gold world clone production pair
  - `causal.rs:196-199,270-290`의 final score와 gold delta 대조
  - `tests/long_run.rs:117-140`의 독립 event-only/turn-only negative
- Remaining Evidence:
  - `tests/long_run.rs:142-145`는 실제 content/producer를 제거하지 않고 완성된 summary에 `without(witness)`를 호출한다.
  - `causal.rs:421-424`의 `without`은 count와 record label을 직접 삭제한다.
  - 같은 summary의 count를 `validate_required`로 다시 검사하므로 required-set validator의 tautology이지 causal producer-removal test가 아니다.
  - paired score 전용 test `long_run.rs:150-159`도 source 문자열 존재/복제식 부재를 검사한다.
  - `tests/causal_content.rs`의 content mutation은 여전히 ID가 아닌 raw TOML `replacen` 순서에 의존한다.
- Expected: 각 producer/content field를 fixture에서 하나씩 제거했을 때 해당 witness만 사라지고 나머지 8개 record/hash가 유지되어야 한다.
- Actual: 생성 원인이 아니라 이미 생성된 evidence label을 삭제한다.
- Impact: generator attribution 회귀가 있어도 validator test는 계속 green일 수 있다.
- Suggested Fix: typed registry/session fixture에서 각 producer를 실제 제거·변경하고 full causal run을 다시 수행한 9-case removal matrix를 만든다.
- Re-audit: GoldScore를 포함한 각 producer removal이 exactly-one missing witness를 만드는지 확인한다.

### [R25-IMP-F003 — Re-audit #1] Mouse event가 modal·overlay state guard를 우회함

- Related: FIN-F008
- Pattern: `IMP-001`, `DBG-001`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Verified Remediation:
  - GameOver N과 LLM dismiss 우선순위
  - MorePrompt Tab/BackTab/N acknowledge
  - old ignored response의 new outstanding 선처리
- Remaining Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:1519-1541`은 non-key event를 먼저 분기하여 mouse를 즉시 underlying state mapper로 보낸다.
  - Inventory/StorageError/soft-input guards는 이후 `:1550-1569`에 있어 mouse에는 적용되지 않는다.
  - direct probe에서 세 modal/overlay 모두 command panel click을 `Some(Command(Wait))`로 반환했고 실제 turn이 진행됐다.
  - accepted command는 `mod.rs:729-736`에서 overlay까지 닫는다.
- Expected: modal/overlay가 active이면 mouse도 그 layer의 허용 동작만 수행하거나 명시적으로 무시해야 한다.
- Actual: keyboard는 modal-safe지만 mouse가 underlying core command를 제출한다.
- Impact: 보이지 않는 turn 진행, storage error/Inventory/Judge 입력 손실, keyboard/mouse semantics 분리.
- Suggested Fix: terminal-size와 modal/overlay/soft-input guard를 event kind 분기보다 먼저 적용하고 layer별 mouse dispatcher를 둔다.
- Re-audit: 각 modal 위 map/status/inspect/command/footer click이 core revision/hash를 바꾸지 않는 matrix를 추가한다.

### [R25-IMP-F004 — Re-audit #1] Inspect renderer와 mouse hit model이 presentation state를 공유하지 않음

- Related: FIN-F009
- Pattern: `IMP-001`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Verified Remediation:
  - 60x24/80x24 blocking prompt content
  - command panel label-derived CTA geometry
- Remaining Evidence:
  - `render_panels.rs:200-218`은 hover 또는 decision lines가 있으면 inventory 대신 그 내용을 표시한다.
  - `input.rs:291-293,341-353`은 presentation state 없이 항상 `inventory_panel_ctas`를 hit-test한다.
  - hover text가 보이는 Inspect 첫 행 click의 direct 결과는 `Some(Command(Wield { item: EntityId(5) }))`였다.
  - 현재 UI test는 기본 inventory presentation과 label 외부 column만 검사해 이 조합을 놓친다.
- Expected: renderer가 표시한 CTA와 같은 state/model만 click target을 제공해야 한다.
- Actual: 표시되지 않은 inventory command가 활성화된다.
- Impact: 사용자 의도와 다른 장비·소비 command 제출, turn/state 변조.
- Suggested Fix: `InspectPresentation`과 CTA list를 한 번 계산해 renderer와 mouse input이 공유하고 hover/decision/modal view에서는 inventory candidates를 제거한다.
- Re-audit: inventory/hover/decision/soft-input별 rendered buffer와 click candidate를 같은 fixture로 대조한다.

### [R25-IMP-F005 — Re-audit #1] active master spec가 완료 상태와 충돌함

- Related: FIN-F012
- Pattern: `IMP-004`, `DOC-BACKFILL-001`
- Severity: **Major**
- Status: **Needs Documentation Recovery**
- Evidence:
  - `spec.md:7`은 active contract만 포함한다고 선언한다.
  - `spec.md:12`는 report 25 시정을 same-SHA 양 OS Verified라고 기록한다.
  - 그러나 `spec.md:796`은 production GoldScore pair와 negative matrix가 “현재 ... 시정 중”이라고 적는다.
  - code, `GAP_CLOSURE_ROADMAP.md:52,64`와 implementation summary는 Verified다.
  - `tests/r8_documentation.rs`의 active tests는 master spec의 이 status 문장을 검사하지 않아 8개가 green이다.
- Expected: `spec.md`의 current status, gap child/aggregate와 implementation evidence가 같은 lifecycle을 사용해야 한다.
- Actual: master plan 안에서 Verified와 in-progress가 동시에 current다.
- Impact: 다음 coder/auditor가 완료 범위와 현재 gate를 다르게 해석한다.
- Suggested Fix: report 26 HOLD를 current authority로 기록하면서 GoldScore 구현 완료와 producer-removal 미완료를 분리하고 section-aware regression을 추가한다.
- Re-audit: active spec/GAP/summary/audit authority exact status를 다시 대조한다.

## 6. Pass 2 — Debug·Engineering Quality Findings

### [R25-DBG-F002 — Re-audit #1] Windows trailing-name alias가 replay same-file guard를 우회함

- Related: FIN-F004
- Pattern: `DBG-001`, `SEC-004`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Verified Remediation:
  - `.` component 제거
  - ASCII case 비교
  - 기존 file identity, hard-link와 parent junction 거부
- Remaining Evidence:
  - `save.rs:737-755`는 Windows component 끝의 dot/space를 허용한다.
  - preflight `symlink_metadata/open`은 `run.jsonl.`을 NotFound로 보지만 Windows replace는 이를 기존 `run.jsonl`과 같은 이름으로 정규화한다.
  - `run.jsonl.`, `run.jsonl..`, `run.jsonl ` output이 모두 exit 0으로 input trace를 atomic replace했다.
  - 8.3 alias, ADS, hard-link, parent junction control은 exit 2로 통과했다.
- Expected: compare와 write 단계가 같은 path resolution 의미를 사용하며 모든 same-file alias를 실행 전에 거부해야 한다.
- Actual: 서로 다른 API의 Windows name normalization 차이가 guard를 우회한다.
- Impact: replay evidence 변조·축약, 후속 determinism mismatch, input/output 분리 계약 위반.
- Suggested Fix: Windows relative component의 trailing dot/space와 reserved device name을 거부하고 compare/open/replace를 동일 handle-resolution 정책에 묶는다.
- Re-audit: dot/space/repeated-dot/reserved/8.3/ADS/case/hard-link matrix에서 exit 2와 input bytes/hash/file identity 불변을 확인한다.

### [R25-DBG-F003 — Re-audit #1] Terminal implementation은 개선됐지만 evidence 설명이 정확히 닫히지 않음

- Related: FIN-F010
- Pattern: `DBG-001`, `TEST-001`, `IMP-004`
- Severity: **Minor**
- Status: **Partially Verified / Needs Documentation Recovery**
- Verified Evidence:
  - setup completed-state tracking과 best-effort restore
  - `Terminal::new`, draw, read가 restore wrapper closure 내부에 존재
  - Windows ConPTY alternate/cursor enter/leave pair와 actual mouse input 5회 PASS
- Remaining Evidence:
  - failure test는 `terminal_new/draw/read` 이름만 바꾼 동일 즉시-Err closure를 사용하며 실제 operation별 seam은 아니다.
  - setup test는 Drop 자체가 아니라 setup helper 뒤 restore helper를 직접 호출한다.
  - `designs.md:327`은 Windows mouse enable/disable을 byte stream에서 검증한다고 하지만 remediation과 ConPTY comment는 Windows Console API call matrix로 분리한다고 설명한다.
- Expected: code/test가 실제 보장 범위를 지지하고 active design이 같은 evidence boundary를 설명해야 한다.
- Suggested Fix: designs를 Console API/lifecycle matrix에 맞추고, 필요하면 setup guard Drop 및 operation seam test를 강화한다.
- Re-audit: source call graph, injected lifecycle와 ConPTY claim을 동일 문구로 대조한다.

### [R26-IMP-F001] Key repeat가 LLM request 중복 금지 계약을 우회할 수 있음

- Related: FIN-F009, FIN-F016
- Pattern: `IMP-001`, `TEST-001`
- Severity: **Minor**
- Status: **Needs Fix**
- Evidence:
  - `designs.md:306,320`은 key repeat가 새 LLM request를 만들지 않는다고 정의한다.
  - `runtime_event_to_candidate`는 `KeyEventKind::Release`만 거부하고 `Repeat`은 통과시킨다.
  - 빠른 response로 outstanding이 해제되고 250ms cooldown 뒤 held G/A/J repeat가 도착하면 새 request candidate를 다시 만들 수 있다.
- Expected: LLM CTA는 `Press`만 enqueue하고 `Repeat`은 presentation/core state에 영향을 주지 않아야 한다.
- Suggested Fix: LLM request keys에서 Repeat를 거부하고 injected clock/fast-response repeat regression을 추가한다.
- Re-audit: held G/A/J가 request ID를 한 번만 발급하는지 확인한다.

## 7. Pass 3 — Security·Supply Chain Findings

### [R25-SEC-F001 — Re-audit #1] Release exact-set이 staging root와 expected inode를 검증하지 않음

- Related: FIN-F014
- Pattern: `SEC-004`, `SEC-006`, `BUILD-001`, `TEST-001`
- Severity: **Major**
- Status: **Needs Fix**
- Verified Remediation:
  - output의 extra file/directory/symlink/reparse entry 거부
  - expected name/checksum/metadata exact set
  - 양 OS actual bundle positive와 extra-entry negatives
- Remaining Evidence:
  - PowerShell verifier `:67`은 `Resolve-Path`로 OutputDir junction을 따라가며 exact external root를 PASS했다.
  - 양 verifier는 expected-name hardlink를 regular file로 수용한다. link-count/file identity 검사가 없다.
  - `build.sh:58-69`는 기존 gitignored `output/`과 destination inode를 신뢰하고 일반 GNU `cp`로 덮어쓴다.
  - preplaced output binary hardlink probe에서 outside victim content가 `new-release-binary`로 변경됐다.
  - clean-worktree preflight는 ignored output root/link를 보지 않는다.
- Expected: release staging은 verified fresh root와 single-link destinations 안에서만 생성되고 verifier는 같은 boundary를 검사해야 한다.
- Actual: entry 이름 집합은 exact하지만 root/inode authority는 외부로 redirect될 수 있다.
- Impact: build 실행 시 외부 파일 overwrite, 검증 후 외부 inode mutation, 잘못된 release provenance.
- Suggested Fix:
  1. build가 예측 불가능한 fresh staging directory를 생성하고 완료 뒤 검증된 destination으로 promote한다.
  2. output root를 no-follow로 열고 workspace 내부 real directory인지 확인한다.
  3. expected destination은 copy 전에 link/reparse/nlink를 거부하거나 safe new-file + atomic replace를 사용한다.
  4. 양 OS에 root junction/symlink와 expected hardlink negative를 추가한다.
- Re-audit: outside victim 불변, verifier nonzero와 actual clean bundle positive를 양 OS에서 확인한다.

### [FIN-F015 — Re-audit #2] Modification notice 기간이 실제 release commit을 포함하지 않음

- Related: R25-SEC-F003, FIN-F015
- Pattern: `IMP-004`, `BUILD-001`, `SEC-006`
- Severity: **Major**
- Status: **Needs Fix**
- Evidence:
  - 프로젝트 기준 시간대는 Asia/Seoul이며 commit offset도 `+09:00`이다.
  - `02050fe`, `b732c42`, `65d8fe5`의 author/commit date는 모두 2026-08-24다.
  - `MODIFICATIONS.md:3-19`의 notice ID와 모든 path scope는 2026-08-23 종료다.
  - `CHANGELOG.md:3,26`, `README.md:3`, `RELEASE-METADATA:5`, verifier와 R8/license tests도 2026-08-23을 정답으로 고정한다.
  - current source archive는 commit `65d8fe5`를 기록하면서 그 commit이 수정한 파일을 하루 전 종료 notice로 함께 운반한다.
- Expected: modification manifest period와 notice ID가 실제 candidate commit의 변경 일자를 포함해야 한다.
- Actual: release gate가 self-consistent한 stale 날짜를 검사하여 false-green이다.
- Impact: 배포 modification provenance 부정확, FIN-F015 release boundary 재개방.
- Suggested Fix: 2026-08-24까지 새 notice ID/period, changelog, README, metadata, scripts와 fixtures를 일괄 갱신하거나 UTC를 사용한다면 timezone과 date derivation을 단일 계약으로 명시한다.
- Re-audit: archive commit timestamps와 bundled manifest 범위가 자동으로 교차 검증되는지 확인한다.

### [R25-SEC-F002 — Re-audit #1] Dependency exception gate가 미래 승인일을 허용함

- Related: FIN-F013
- Pattern: `SEC-006`, `DEP-001`
- Severity: **Minor**
- Status: **Needs Fix**
- Verified Remediation: parsed TOML exact table, trigger key/version/path, calendar validity, expiry/90-day budget와 current graph는 통과한다.
- Remaining Evidence:
  - date predicate는 `expires < today`, `expires <= approved`, 90일만 검사하고 `approved > today`를 거부하지 않는다.
  - `today=2026-08-24`, `approved=2026-09-01`, `expires=2026-10-31`은 현재 predicate를 통과한다.
- Expected: approval은 감사 실행일 이후일 수 없다.
- Suggested Fix: `approved <= today`를 강제하고 future-approval negative를 추가한다.
- Re-audit: current, future, expired, invalid calendar와 90-day boundary를 함께 실행한다.

### [R25-SEC-F003 — Re-audit #1] General full-SHA gate가 유효 YAML 변형과 다른 workflow를 놓침

- Related: FIN-F015
- Pattern: `SEC-006`, `BUILD-001`, `TEST-001`
- Severity: **Minor**
- Status: **Partial Verified / Needs Fix**
- Verified Remediation:
  - 현재 checkout/rust-toolchain action은 공식 full SHA
  - checkout comment `v4.4.0` 일치
  - current HEAD Actions `32651576393` 양 OS green
- Remaining Evidence:
  - `tests/build_contract.rs:203-231`은 `.github/workflows/ci.yml`의 `- uses:`/`uses:` line prefix만 추출한다.
  - 유효 YAML inline map `- { uses: actions/setup-node@v4 }`와 spaced key `uses : actions/cache@v4`를 놓쳤다.
  - 새 workflow와 composite action YAML도 scan하지 않는다.
- Expected: “모든 workflow uses” 주장은 전체 `.github/**/*.yml|yaml`의 YAML node를 구조적으로 순회해야 한다.
- Suggested Fix: YAML parser로 모든 `uses` value를 수집해 local action 예외와 40-hex remote ref를 검증한다.
- Re-audit: inline/spaced/nested/composite/multiple workflow mutable refs가 모두 실패하는지 확인한다.

## 8. Cross-Pass Conflicts

| Conflict | 해소 판단 |
| --- | --- |
| 416개 named test와 양 OS CI는 green이나 direct malformed save가 panic/wrap | 열거된 회귀가 consumer-safe scalar를 덮지 못하므로 save finding 유지 |
| Path guard는 lexical/case/file identity를 검사하지만 Windows replace 의미가 다름 | compare/write API semantic을 통일할 때까지 finding 유지 |
| GoldScore production pair는 진짜지만 removal test는 record 삭제 | positive pair Verified, producer-isolation closure는 Needs Fix |
| CTA model을 공유했지만 Inspect presentation state는 공유하지 않음 | command CTA sub-scope만 Verified, 전체 mouse geometry는 Needs Fix |
| Release actual entries는 exact하지만 root와 inode는 외부 가능 | name exact-set은 security authority exact-set을 대체하지 않음 |
| Release metadata 값은 서로 일치하지만 commit date를 포함하지 않음 | self-consistency test는 historical accuracy를 증명하지 않음 |
| Current HEAD CI도 green이지만 active docs는 구현 SHA run만 current로 기록 | 운영 CI 부재는 아니며 traceability Minor; Major production findings와 별개 |

## 9. Verified로 유지하는 개선

- Save writer 16 MiB capped serialization과 failure no-clobber
- Replay 7-field self-verification와 working-copy no-partial-commit
- public `DerefMut`와 ambient resolver 제거
- immutable registry와 정상 corpse/equipment wear/drop/rewear lifecycle
- TUI keyboard state priority, ignored response lifecycle, blocking prompt, command CTA
- signal/Condvar 기반 LLM response waiting
- runtime package semantic tests와 duplicate dependency metadata
- parsed dependency exception TOML과 current graph/expiry
- current workflow action pins와 양 OS CI
- extra release entry exact-set rejection
- FIN-F018 atomic batch rewrite within documented threat model
- active Markdown relative link 0 broken

## 10. PASS 전 필수 수정 순서

### P0 — Major

1. save unequipped AC, turn/economy/ItemData와 모든 narrow arithmetic consumer를 fail-closed로 닫는다.
2. Windows trailing dot/space/reserved-name replay alias를 거부하고 compare/write semantics를 통일한다.
3. 9개 causal witness를 actual producer removal로 검증한다.
4. modal/overlay mouse를 layer-aware dispatcher로 제한한다.
5. Inspect renderer와 hit model이 같은 presentation/CTA 구조를 사용하게 한다.
6. release를 fresh no-follow staging root에서 생성하고 expected hardlink/reparse를 거부한다.
7. active spec status를 actual partial closure와 report 26 HOLD에 맞춘다.
8. modification notice를 2026-08-24 candidate commit 범위와 동기화한다.

### P1 — Minor

9. terminal designs/evidence boundary를 실제 Windows Console API 검증 범위와 맞춘다.
10. LLM CTA의 key-repeat 중복 생성을 금지한다.
11. dependency exception future approval을 거부한다.
12. 모든 workflow/composite YAML의 action ref를 구조적으로 검사한다.
13. current HEAD `65d8fe5` Actions `32651576393`을 최종 문서-evidence SHA로 기록한다.

## 11. Accepted Risks와 남은 제한

- runtime same-account malicious concurrent directory-entry swap은 기존 single-writer threat model 밖이다.
- Windows parent-directory power-loss durability는 OS/filesystem 잔여 위험이다.
- 외부 real-provider smoke와 Windows Terminal GUI rendering은 비차단 제외 범위다.
- CI artifact 서명·attestation·upload는 현재 spec 필수 범위가 아니므로 새 blocker로 승격하지 않는다.
- 위 accepted/excluded risk는 malformed save, release staging root, expected hardlink, stale modification period를 허용하지 않는다.

## 12. 재감사 체크리스트

1. 각 Major의 direct probe를 named regression으로 보존하고 수정 전 RED/수정 후 GREEN을 기록한다.
2. malformed save는 load 단계에서 typed reject하고 debug/release 모두 panic·wrap이 없다.
3. trailing dot/space/reserved alias는 exit 2이고 input bytes/hash/file ID가 불변이다.
4. actual producer removal 9-case가 exactly-one witness loss를 만든다.
5. modal/overlay 및 Inspect presentation mouse matrix가 core revision/hash 불변을 보장한다.
6. output root junction/symlink와 expected hardlink가 양 OS build/verifier에서 nonzero이며 outside victim이 불변이다.
7. bundled modification period가 candidate commit date를 포함한다.
8. future exception approval과 YAML action variants가 실패한다.
9. active docs가 report 26의 current HOLD와 partial/Verified sub-scope를 일치시킨다.
10. 아래 전체 gate를 단독 환경에서 재실행한다.

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

11. 새 clean commit의 Ubuntu/Windows same-SHA actual bundle을 확인한다.
12. 새 독립 감사가 report 26 finding과 FIN-F001~F018을 연결해 판정한다.

## 13. 최종 근거와 Coder Handoff

### 최종 근거

- Report 25의 13개 finding을 모두 재대조했으며 세 finding은 완전 Verified, 여러 finding은 유효한 부분 시정을 갖는다.
- 하지만 save, Windows path, causal negative, TUI mouse, release staging/provenance와 master spec에서 Major 8건이 남았다.
- 따라서 `docs/audit/audit_report_25_remediation.md:8,120,133`의 로컬/remote 구현 closure 주장은 부분적으로만 유효하다.
- program/publication HOLD는 유지되어야 하며 report 26 finding 시정 전에는 `Closed`로 승격할 수 없다.

### Coder Handoff

```text
`C:\LocalDev\rust\AIHack\docs\audit\audit_report_26.md`의 독립 재감사 결과를 확인하고,
각 finding을 현재 사용자 요구, spec/design/ADR, 실제 production entrypoint와 재현 probe에 대조하여 수정하세요.
계약 변경은 관련 문서를 먼저 갱신하고, helper-level green이 아니라 실제 malformed save, Windows alias,
modal mouse, producer removal, release staging root/inode와 candidate-date fixture를 회귀 테스트로 보존하세요.
수정 후 전체 로컬 gate와 새 clean same-SHA Ubuntu/Windows actual bundle을 실행하고 결과를 기록하세요.
```
