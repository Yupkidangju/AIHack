# AIHack v0.3.0 감사 보고서 28 시정 독립 재감사 보고서 29

- 감사 대상: `docs/audit/audit_report_28_remediation.md`
- 기준 감사: `docs/audit/audit_report_28.md`
- 프로젝트: `C:\LocalDev\rust\AIHack`
- 감사 일자: 2026-08-24
- 시정 구현 SHA: `9725c37896a8d149be5c500cdd26da154ab0a3fa`
- 현재 HEAD: `d9c0f8eb673e641f2a23b17f99a9327e90899628`
- 브랜치: `codex/audit-report-28-remediation`
- 작업 트리: 감사 시작 및 모든 임시 probe 정리 후 clean, 최종적으로 이 보고서만 추가
- 환경: Windows 11 Pro, `x86_64-pc-windows-msvc`, Asia/Seoul
- Rust/Cargo: 1.94.1
- 보안 도구: `cargo-audit 0.22.1`, `cargo-deny 0.19.4`
- 적용 기준: `AI_AUDIT_DOC_STANDARD.md`, `audit_roadmap.md`, `spec.md`, `designs.md`, `AGENTS.md`
- 추가 검토 기준: `code-review-and-quality`, `security-and-hardening`
- 감사 원칙: 구현·테스트·설정·기존 통제 문서는 수정하지 않고 이 보고서만 추가한다.

## 0. 최종 판정

**HOLD — REPORT 28 REMEDIATION PARTIAL / INDEPENDENT PASS 기각**

Report 28 시정은 핵심 production 결함을 실제로 닫았다. exact-successor allocator, fallible spawn과 `GameSession::submit` rollback, custom monster HP/bootstrap, armor forbidden shape와 공통 removal lifecycle, 명시 Esc/Enter/F9 및 soft-input 밖 Quit q/Q Repeat, 실제 F9 candidate/handler, year `0001..9999` calendar parity가 source·회귀·독립 probe에서 유효했다. 구현 SHA `9725c378`의 Actions `32694375654`와 current docs HEAD `d9c0f8eb`의 Actions `32695945790`도 Ubuntu/Windows 전체 gate 및 실제 플랫폼 bundle을 모두 통과했다.

그러나 인접 동등 경계와 문서 권위를 다시 확장한 결과 **Confirmed Major 5건**이 남았다.

1. TUI Repeat 차단이 Esc/Enter/F9 및 soft-input 밖 Quit q/Q 열거에 그쳐 Load, Inventory, MorePrompt, inventory selection 등 다른 상태 전이에서 같은 Repeat가 새 state의 core/UI 명령으로 재해석된다. 실제 terminal transport의 연속 Press도 `KeyEventKind::Repeat` 차단을 우회한다.
2. 양 release verifier가 Windows 금지문자, superscript COM/LPT, console device, entry type/link target, file-vs-directory prefix 및 extraction collision을 완전하게 검증하지 않는다.
3. source archive를 `ExpectedCommit`의 path/type/exported-content tree와 비교하지 않아 실제 Rust source가 전혀 없는 6-file archive도 checksum과 metadata만 맞추면 PASS한다.
4. remediation이 `spec.md`까지 동결했다고 주장한 TUI/archive/year 계약이 master spec에 없고, ADR/CHANGELOG의 rollback 문구도 최종 narrowing과 다르다.
5. README·ADR·audit roadmap의 stale current authority가 남아 있는데 document regression 10개가 모두 PASS하여 과거 Major `DBG-F008` false-green 패턴이 재발했다.

**Major 영향 후보 `Needs Spec Clarification` 1건**도 있다.

- item ID가 만드는 typed `ItemKind`와 별도 declared `kind`가 만드는 `ItemClass`의 canonical 관계가 없다. `item.weapon.dagger`를 armor로 선언한 registry가 `ItemKind::Dagger + ItemClass::Armor`로 bootstrap/save/Wear까지 수용되고 TUI kind-based command와 class-based ActionSpace가 다른 동작을 낸다.

추가로 **Minor 3건**이 있다.

- 여러 문자 glyph가 registry에서 수용된 뒤 runtime 첫 문자로 조용히 잘린다.
- production-valid allocator exhaustion과 Throw/Zap/RNG rollback 독립 probe는 PASS했지만 영구 회귀는 production-invalid fixture 하나에 집중되어 있다.
- 공개 low-level projectile/monster system은 transaction 밖 직접 호출 시 `Err`와 함께 world/RNG partial mutation을 남기므로 공개 API 계약을 닫아야 한다.

현재 실제 source ZIP은 `git archive HEAD`와 byte-identical이고 382/382 entry가 일치한다. 따라서 release finding은 현재 artifact 오염 주장이 아니라 verifier가 대체·누락 archive를 독립적으로 거부하지 못하는 hard-boundary 결함이다.

현재 정확한 상태는 **NORMAL GATES AND CURRENT ARTIFACT GREEN / EQUIVALENT PRODUCTION, RELEASE, AND AUTHORITY BOUNDARIES OPEN / PROGRAM AND PUBLICATION HOLD**다.

## 1. 감사 범위와 제한

### 1.1 확인한 구현·문서·증거

- `01b2bd3..9725c37`: allocator/content/equipment/TUI/release 시정 구현과 회귀
- `9725c37..d9c0f8e`: evidence lifecycle 문서와 `tests/r8_documentation.rs` 후속 갱신
- allocator/save: exact successor, checked allocation, corpse/Throw/Zap/monster-phase transaction rollback
- content/equipment: HP·item shape·bootstrap validation, Drop/Throw/Quaff/Eat/Read common removal
- TUI: dispatcher key kind, overlay/state transitions, actual F9 handler, ConPTY transport
- release: ZIP/TAR raw name·type·link·collision, calendar, checksums, source completeness와 commit binding
- active spec/design/ADR/README/BUILD/CHANGELOG/implementation summary/gap/audit roadmap와 semantic document regression
- dependency graph, action pin, cargo-audit/deny, current implementation/current HEAD 양 OS CI

### 1.2 제외 범위

- 실제 물리 keyboard key-hold 자체는 재현하지 않았다. constructed events, installed crossterm parser와 actual ConPTY repeated-byte transport를 구분해 기록한다.
- 외부 실제 LLM provider smoke는 v0.3.0 비목표다.
- Windows Terminal GUI pixel/font rendering은 제외하고 ConPTY·dispatcher·terminal restoration을 확인했다.
- `legacy_nethack_port_reference/` 본문은 제품 범위 밖이며 runtime import와 release archive 차단만 감사했다.
- 외부 tag/release/publish 및 Git commit/push는 수행하지 않았다.
- runtime same-account concurrent directory-entry swap은 기존 single-writer threat model 밖이다.
- artifact signing/attestation과 외부 업로드는 현재 release bundle 필수 계약 밖이다.

### 1.3 감사 도구 제한

적용한 skill이 참조하는 다음 세부 파일은 설치본에 없었다.

- `code-review-and-quality/references/security-checklist.md`
- `code-review-and-quality/references/performance-checklist.md`
- `security-and-hardening/references/security-checklist.md`

각 skill 본문과 `AI_AUDIT_DOC_STANDARD.md`로 대체했다. 이는 프로젝트 finding이 아니라 감사 환경 제한이다.

## 2. 실행·검증 증거

### 2.1 로컬 전체 gate

| 명령 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked -- --list` | named test **445개** |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| `cargo audit` | PASS, 318 dependencies, vulnerabilities 0 |
| `cargo deny check licenses bans sources` | PASS, bans/licenses/sources ok |
| dependency exception/duplicate gate | PASS |
| Git Bash `scripts/r7_checkpoint.sh` | PASS |
| Git Bash `scripts/r8_checkpoint.sh` | PASS |
| `cargo test --locked -p aihack --test r8_documentation` | 10 PASS, semantic stale authority는 검출하지 못함 |
| final clean `build.bat --release` | PASS, Windows 9-entry exact bundle, commit `d9c0f8e` |
| `cargo metadata --locked` | packages 318, registry 310, path 8, git 0 |
| active Markdown relative-link scan | PASS, broken 0; stale semantic target은 별도 finding |

첫 `build.bat --release` 감사 실행은 병렬 보안 probe의 untracked helper 두 개가 일시적으로 존재해 clean-worktree gate에서 실패했다. helper 정리와 절대 경로 확인 뒤 동일 current HEAD에서 재실행하여 PASS했으므로 프로젝트 build 결함으로 분류하지 않는다.

### 2.2 원격 evidence lineage

| SHA / Actions | Ubuntu | Windows | 판정 |
| --- | --- | --- | --- |
| `9725c378` / [`32694375654`](https://github.com/Yupkidangju/AIHack/actions/runs/32694375654) | PASS | PASS | report 28 production 시정 구현의 same-SHA 19-success-step gate와 actual bundle evidence |
| `d9c0f8eb` / [`32695945790`](https://github.com/Yupkidangju/AIHack/actions/runs/32695945790) | PASS | PASS | docs/test-only current HEAD의 전체 gate와 actual bundle evidence |

`9725c378..d9c0f8e`은 문서와 `tests/r8_documentation.rs`만 변경했다. 구현 authority를 `9725c378/32694375654`로 기록한 것은 정확하며 current HEAD run은 추가 evidence다.

### 2.3 독립 adversarial 결과

#### Allocator·transaction

```text
sequential_corpse:
  first_accepted=true
  next_after_first=4294967295
  second_no_panic=true
  second_accepted=false
  second_save_equal=true
  second_hash_equal=true

bump_exhausted:  accepted=false full_save_equal=true hash_equal=true
throw_exhausted: accepted=false full_save_equal=true hash_equal=true
zap_exhausted:   accepted=false full_save_equal=true hash_equal=true
```

`max_id=MAX-2,next_id=MAX-1` valid save의 첫 corpse spawn은 commit되고 두 번째 fallible allocator 오류는 `CommandRejected`로 투영됐다. `max_id=MAX-1,next_id=MAX` valid exhausted save도 실제 loader를 통과했고 반복 command가 world/RNG/hash를 보존했다.

#### Content identity·glyph

```text
ID_KIND_ACCEPTED kind=Dagger class=Armor
ASSERTIONS=bootstrap/save/Wear PASS

GLYPH_ACCEPTED registry=LONG runtime=L
```

Report 28의 hp 0/-1 및 armor damage/hit fixture는 모두 typed reject됐고 accepted hp 5 registry initial/Wait/save/load는 PASS했다.

#### TUI transition

```text
LOAD_PRESS_TITLE=Some(Load)
LOAD_STATE_AFTER_PRESS=Playing
LOAD_REPEAT_PLAYING=Some(Command(Move(East)))
LOAD_REPEAT_REVISION_CHANGED=true

INVENTORY_AFTER_PRESS=Inventory
INVENTORY_REPEAT_1=Some(CloseOverlay)
INVENTORY_AFTER_REPEAT_1=None
INVENTORY_REPEAT_2=Some(Command(ShowInventory))
INVENTORY_AFTER_REPEAT_2=Inventory

MORE_I_PRESS=Some(Command(AcknowledgeMore))
MORE_STATE_AFTER_PRESS=Playing
MORE_I_REPEAT_PLAYING=Some(Command(ShowInventory))

SELECTION_B_PRESS=Some(InventoryLetter('b'))
SELECTION_STATE_AFTER_PRESS=Playing
SELECTION_B_REPEAT_PLAYING=Some(Command(Move(SouthWest)))

CONPTY_DOUBLE_ENTER_SINGLE_WRITE_REACHED_PLAYING=true
```

crossterm 0.29 Windows parser는 key-down을 Press, key-up을 Release로 만들고 `repeat_count`를 사용하지 않는다. Unix enhanced-key mode 활성화도 프로젝트에 없다. parser 동작과 상태 전이 결과상 actual binary가 한 transport write의 `\r\r`를 두 Press event로 처리한 것으로 확인된다. 이는 physical key-hold 또는 native auto-repeat 자체의 재현은 아니다.

#### Archive component·type

Current Windows bundle 복사본에 entry를 삽입하고 source ZIP을 포함한 8개 checksum을 모두 다시 만든 결과다.

```text
normal                         EXIT=0
COM¹/probe.txt                 EXIT=0
LPT².log/probe.txt             EXIT=0
CONIN$/probe.txt               EXIT=0
CONOUT$/probe.txt              EXIT=0
bad?/probe.txt                 EXIT=0
bad|name/probe.txt             EXIT=0
bad"name/probe.txt             EXIT=0
bad<name>/probe.txt            EXIT=0
sanitize?.txt + sanitize*.txt  EXIT=0
file + file/child prefix       EXIT=0
ZIP symlink -> ../../outside   EXIT=0
```

Windows extraction은 sanitize pair를 한 `sanitize_.txt`로 축약하고 prefix fixture의 regular file을 directory 의미로 재해석했으며 symlink fixture는 `Invalid argument`로 실패했다. TAR에서도 금지문자/device/Unicode case/prefix/symlink/hardlink/character-device entry가 verifier를 통과하고 실제 GNU tar extraction은 일부 fixture에서 exit 2였다.

#### Corresponding source identity

```text
ARCHIVE_ENTRIES=6
RUST_SOURCE_ENTRIES=0
METADATA_COMMIT=<expected-current-commit>
CHECKSUMS_REGENERATED=true
BASH_VERIFIER_EXIT=0
```

반면 current actual artifact는 다음과 같이 정상이다.

```text
git archive HEAD SHA == output source ZIP SHA
BYTE_IDENTICAL=true
entry_count=382/382
regular=334 directory=48 symlink/hardlink/device=0
```

#### Documentation gate

```text
README current implementation order -> report 26
README Current re-audit/remediation links -> report 26
ADR-0037/report 27 current authority phrases remain
audit_roadmap gate/new CI pending phrase remains
cargo test -p aihack --test r8_documentation -> 10 PASS
```

### 2.4 Adversarial probe command·fixture 기록

#### Allocator·transaction

임시 Cargo probe와 target은 각각 아래 경로였고 종료 후 삭제했다.

```text
C:\Users\temp\AppData\Local\Temp\aihack-r29-transaction-probe
C:\Users\temp\AppData\Local\Temp\aihack-r29-transaction-target
```

```powershell
$env:CARGO_TARGET_DIR='C:\Users\temp\AppData\Local\Temp\aihack-r29-transaction-target'
cargo run --quiet --manifest-path 'C:\Users\temp\AppData\Local\Temp\aihack-r29-transaction-probe\Cargo.toml'
```

probe는 production validator를 통과하는 `max_id=MAX-2,next_id=MAX-1` 및 `max_id=MAX-1,next_id=MAX` save를 만들고 bump/Throw/Zap과 direct low-level system을 비교했다. 표적 영구 회귀 명령도 다음과 같이 실행했다.

```powershell
cargo test --locked -p aihack-core exhausted_allocator_returns_without_panicking_or_mutating_the_store
cargo test --locked -p aihack-runtime corpse_allocation_exhaustion_rejects_without_panicking_or_committing_partial_state
cargo test --locked -p aihack --test combat --test golden_phase8_rules --test monster_ai --test projectiles
cargo test --locked -p aihack --test transaction
```

#### Content identity·glyph

감사 중 생성한 `tests/__r29_content_probe.rs`는 최종 삭제했다.

```powershell
cargo test --locked -p aihack --test __r29_content_probe -- --nocapture
```

성공 재실행은 exit 0, 2 PASS였고 §2.3의 ID-kind/glyph 출력을 만들었다. 첫 작성본은 helper 이름 shadow로 Rust `E0618` compile exit 1이었으며 helper 이름 수정 후 같은 명령으로 성공했다. 이는 프로젝트 결함이 아닌 일회성 감사 harness 오류다.

fixture는 embedded dagger block의 `kind="weapon",slot="melee",hit_bonus=1,damage="1d4"`를 `kind="armor",slot="body",ac_bonus=1`로 바꾸고, 별도 glyph case는 armor `glyph="["`를 `glyph="LONG"`으로 바꿨다.

#### TUI production API·ConPTY

```powershell
cargo run --manifest-path 'target\r29-tui-probe\Cargo.toml'
cargo run --manifest-path 'target\r29-conpty-probe\Cargo.toml'
cargo test --locked -p aihack-tui --all-targets
```

첫 probe는 `aihack-runtime`, `aihack-tui`, crossterm 0.29를 path dependency로 사용하고 Load/Inventory/MorePrompt/selection/GameOver/soft-input의 `runtime_event_to_candidate`→handler sequence를 실행했다. 둘째는 repository ConPTY test와 같은 portable-pty 0.9.0 harness에서 실제 `target/debug/aihack.exe`에 한 번의 `write_all(b"\r\r")`을 보냈다. 두 임시 manifest/source/lock/build directory는 `cargo clean --target-dir` 후 삭제했고 `target/r29-*-probe`가 존재하지 않음을 확인했다.

#### Windows ZIP/TAR component와 complete source

아래 helper는 감사 중 생성한 뒤 모두 삭제했다.

```text
.audit-r29-security-probe.ps1
.audit-r29-zipmutate.py
.audit-r29-bashprobe.ps1
.audit-r29-tarbuild.py
.audit-r29-windowsextract.ps1
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\.audit-r29-security-probe.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\.audit-r29-bashprobe.ps1 -CasePattern '^normal$'
powershell -NoProfile -ExecutionPolicy Bypass -File .\.audit-r29-windowsextract.ps1 -CasePattern 'sanitization_|file_prefix_conflict|zip_symlink_escape'
```

ZIP helper는 current 9-entry output을 GUID temp에 복사하고 Python `zipfile`로 raw name/Unix-mode symlink를 추가한 뒤 archive를 포함한 8개 SHA-256 record를 재생성했다. TAR helper는 PAX archive에 release 문서/metadata 6개만 넣은 identity case와 name/type/link cases를 생성하고 Git Bash verifier를 호출했다. Windows/GNU extractor probe는 동일 archive를 실제로 풀어 결과 entry와 exit를 기록했다.

Current actual positive는 temp `git archive --format=zip HEAD`의 SHA-256·entry count를 `output/aihack-0.3.0-source.zip`과 비교했다. 양 SHA는 `42ad07b578de9470a07c2278eb239e1b511b97e877b225aef32c1518eb3e79ba`, entry는 382/382였다.

#### 최종 정리 확인

```powershell
Get-ChildItem $env:TEMP -Directory -Filter 'aihack-r29-*'
git status --short
git diff --check
```

최종 temp 검색과 Git status 출력은 없었고 `git diff --check`는 PASS했다.

모든 감사용 source/fixture/target은 정리했고 report 작성 전 tracked/untracked worktree는 clean이었다.

## 3. Report 28 finding 재감사 상태

| 원 finding | Report 29 상태 | 근거 |
| --- | --- | --- |
| R25-IMP-F001 Re-audit #3 | **Verified** | exact successor와 valid exhaustion의 fallible/no-commit probe PASS |
| R28-IMP-F001 | **Verified** | hp 0/-1 reject, bootstrap full saveability PASS |
| R27-IMP-F001 Re-audit #1 | **Verified** | armor damage/hit reject와 common removal/AC restoration PASS |
| R27-DOC-F002 Re-audit #1 | **Verified** | implementation summary 10·11절의 특정 stale next-step은 복구됨 |
| R28-DBG-F001 | **Needs Fix** | 명시 Esc/Enter fixture는 green이나 다른 transition key와 actual repeated Press transport가 열림 |
| R28-DBG-F002 | **Verified** | 실제 F9 Press→candidate→handler→flag/revision/hash와 second Press 복원 |
| R25-SEC-F001 Re-audit #3 | **Needs Fix** | classic alias는 수정됐으나 Windows component/type/extraction과 complete tree binding 미완료 |
| FIN-F015 Re-audit #4 | **Verified** | calendar implementation component: checksum-consistent year 0000 reject, 0001/9999 accept; aggregate 문서 상태는 §4 참조 |

## 4. FIN-F001~F018 재판정

| ID | Report 29 상태 |
| --- | --- |
| FIN-F001 | **Verified** — Report 28 범위의 exact allocator와 HP/bootstrap consumer safety; ID-kind는 별도 SC-DATA identity clarification |
| FIN-F002 | **Verified** — byte/cardinality/RNG/text와 write no-clobber |
| FIN-F003 | **Verified** — replay 및 `GameSession::submit` no-partial-commit |
| FIN-F004 | **Verified** — Windows runtime artifact path alias matrix |
| FIN-F005 | **Verified** — external mutable state 우회 제거 |
| FIN-F006 | **Verified** — 9종 field-only causal A/B와 structural equality |
| FIN-F007 | **Verified** — immutable registry/custom corpse continuation, armor shape와 Wear→Drop/Throw common lifecycle |
| FIN-F008 | **Needs Fix** — 동등 transition key와 repeated Press state crossing |
| FIN-F009 | **Needs Fix** — Inspect/debug mouse/F9는 Verified, broader transition semantics 잔여 |
| FIN-F010 | **Verified** — terminal RAII/ConPTY/evidence 정렬 |
| FIN-F011 | **Verified** — CLI default/range/docs |
| FIN-F012 | **Needs Documentation Recovery** — master/current authority와 document-wide false-green 재발 |
| FIN-F013 | **Verified** — parsed exception/current/future lifecycle |
| FIN-F014 | **Needs Fix** — archive component/type/extraction 및 ExpectedCommit tree binding 미완료 |
| FIN-F015 | **Needs Documentation Recovery** — calendar 구현은 Verified, calendar/year master contract가 spec에 없음 |
| FIN-F016 | **Needs Fix** — named Repeat는 fixed, equivalent transition/transport path 잔여 |
| FIN-F017 | **Verified** — dependency/package/current 양 OS evidence; corresponding source finding은 FIN-F014/SC-LICENSE에 분리 |
| FIN-F018 | **Verified** — 동결된 single-writer threat model 범위 |

## 5. Pass 1 — 구현·문서 정합성 Findings

### [R29-DOC-F001] Master spec과 ADR이 최종 시정 계약·rollback 범위를 반영하지 않음

- Pass: Implementation
- Pattern: IMP-004, DOC-BACKFILL-001, SPEC-GAP-001
- Area: master authority, implementation contract, transaction semantics
- Severity: **Major**
- Status: **Needs Documentation Recovery**
- Related: FIN-F008, FIN-F009, FIN-F012, FIN-F014, FIN-F015, FIN-F016 — FIN-F003의 GameSession 구현 closure와 별도 문서 범위
- Summary: remediation은 `spec.md`, `designs.md`, ADR-0038에 다섯 경계를 먼저 동결했다고 주장하지만 master spec에는 TUI control Repeat와 archive Windows component/year 계약이 없다. ADR/CHANGELOG의 모든 rejected command no-commit 표현도 최종 구현보다 넓다.
- Evidence:
  - `docs/audit/audit_report_28_remediation.md:10-16`: 세 문서에 다섯 경계를 동결했다고 주장한다.
  - `spec.md:642,649`: allocator/content만 갱신되고 Press-only, archive canonical component, year `0001..9999`는 없다.
  - `designs.md:308`, `DESIGN_DECISIONS.md:30-32`: 하위 문서에만 TUI/archive/year 계약이 존재한다.
  - remediation `:64`는 첫 구현이 모든 reject를 rollback하여 `AwaitingDirection→Playing` 계약을 깨뜨렸고 최종적으로 partial-mutation 오류만 `transaction_aborted`로 좁혔다고 기록한다.
  - `crates/aihack-runtime/src/session.rs:133-155`도 `was_aborted()`일 때만 원본을 보존한다.
  - `DESIGN_DECISIONS.md:27`과 `CHANGELOG.md:32`는 rejected command/transaction 전체가 commit되지 않는 것처럼 기록한다.
- Expected: `spec.md`가 구현된 destructive-input/release hard boundary와 최종 rollback 조건의 주된 기준이어야 하고 ADR/CHANGELOG가 그보다 넓거나 좁지 않아야 한다.
- Actual: master contract가 비어 있고 하위 문서는 첫 실패한 구현 범위를 일부 유지한다.
- Impact: 향후 구현자가 ADR 문구를 따를 경우 이번 작업에서 이미 재현된 AwaitingDirection 상태 회귀를 다시 만들 수 있다. TUI 및 source archive hard boundary도 spec authority 없이 drift한다.
- Suggested Fix:
  1. `spec.md`에 backend-aware transition Repeat와 Windows-compatible archive component/type/tree/year 계약을 추가한다.
  2. no-commit을 allocation/projectile/monster/death 등 `transaction_aborted` partial-mutation 오류에 한정하고 ordinary invariant-valid rejection의 state transition을 보존한다.
  3. ADR-0038, CHANGELOG, designs와 implementation summary를 같은 용어로 정렬한다.
- Re-audit Method: master spec의 새 계약을 source/tests와 양방향 대조하고 ordinary reject와 aborted reject의 state/save/hash matrix를 실행한다.
- Owner: Architect, Documentation, Coder

### [R29-DOC-F002] 문서 전체 current-authority false-green이 재발함

- Pass: Implementation
- Pattern: IMP-004, TEST-001, DOC-BACKFILL-001
- Area: active documentation lifecycle, regression quality
- Severity: **Major**
- Status: **Needs Fix**
- Related: R20-DBG-F008 recurrence, FIN-F012
- Summary: 여러 active 문서가 과거 report를 current/pending으로 유지하지만 문서 테스트는 report/SHA 문자열이 문서 어디에든 존재하는지만 확인하여 모두 green이다. Report 20의 Major `DBG-F008`과 같은 구조적 재발이다.
- Evidence:
  - `README.md:48`: 다음 단계가 report 26 독립 재감사다.
  - `README.md:65`: Current re-audit/remediation 링크가 report 26을 가리킨다.
  - `DESIGN_DECISIONS.md:51,67,533`: ADR-0037 independent pending 및 report 27 current authority/HOLD 표현이 남는다.
  - `audit_roadmap.md:388`: 전체 gate/새 CI도 pending이라고 쓰지만 `:465`는 `9725c378/32694375654` 완료를 기록한다.
  - `tests/r8_documentation.rs:117-194`: 대부분 `contains` positive 검사이며 README/ADR/roadmap의 current section과 supersede 관계를 parse하지 않는다.
  - 결과: stale 상태가 남아도 `r8_documentation` 10 PASS, physical links broken 0.
- Expected: 각 active current-status section은 단일 authority/lifecycle을 가리키고 알려진 predecessor를 current로 부르는 문장을 document-wide negative gate가 거부해야 한다.
- Actual: 특정 implementation summary만 section-scoped로 강화했고 다른 active 문서는 같은 과거 오류를 반복한다.
- Impact: green 문서 gate가 실제 release 상태를 잘못 안내하고 반복 remediation을 유발한다.
- Suggested Fix:
  1. README 구현 순서/current links, ADR supersede/status, audit roadmap pending gate를 report 29 lifecycle에 맞춘다.
  2. current-authority inventory를 구조화된 단일 데이터 또는 section parser로 검증한다.
  3. 모든 active 문서에서 `current`, `next`, `pending`, `superseded`를 구분하고 historical section은 명시적으로 격리한다.
- Re-audit Method: README/ADR/roadmap 각각에 stale predecessor phrase를 주입하는 mutation test가 RED인지 확인하고 current report/SHA/run exact-one을 검사한다.
- Owner: Documentation, Coder

### [R29-IMP-F001] Item ID와 declared kind의 canonical 관계가 없어 typed identity가 분리됨

- Pass: Implementation
- Pattern: IMP-003, SPEC-GAP-001
- Area: content identity, runtime consumer, TUI/ActionSpace parity
- Severity: **Major**
- Severity Basis: user-facing command consumers가 갈라질 수 있으나 ID-kind 변경을 허용할지 금지할지 제품 계약이 미정이다.
- Status: **Needs Spec Clarification**
- Related: SC-DATA-01 — FIN-F001/F007의 확정 closure와 별도 identity 계약
- Summary: ID는 고정 `ItemKind`, declared `kind`는 별도 `ItemClass`를 만들며 둘의 canonical pairing을 검사하지 않는다.
- Evidence:
  - `crates/aihack-runtime/src/domain/item.rs:26-37`: ID→`ItemKind` 고정 mapping.
  - `crates/aihack-content/src/lib.rs:172-186`: declared kind→`ItemClass` 별도 mapping.
  - 직접 probe: `item.weapon.dagger`를 armor shape로 바꾸자 `ItemKind::Dagger + ItemClass::Armor`가 registry/bootstrap/save/Wear까지 수용됐다.
  - `crates/aihack-runtime/src/observation.rs:248-254`는 class로 Wear action을 제공하지만 `apps/aihack-tui/src/tui/input.rs:407-412`는 kind로 Dagger에 Wield를 제공한다.
- Expected: canonical ID-kind pair를 강제하거나 class-changing mod를 허용한다면 모든 consumer가 같은 authoritative field를 사용해야 한다.
- Actual: 두 의미가 동시에 authoritative라 같은 item에 legal Wear와 UI Wield가 공존한다.
- Impact: accepted custom content가 adapter별로 다른 명령과 설명을 노출한다.
- Suggested Fix: ID-kind mapping을 spec에 동결한 뒤 registry reject 또는 consumer 통합 중 하나를 선택하고 모든 canonical pair 및 intentional mod pair를 표 기반 회귀로 고정한다.
- Re-audit Method: 각 known ID의 declared kind mutation이 명시 정책대로 reject/accept되고 TUI, ActionSpace, save와 item lifecycle이 동일 명령을 선택하는지 확인한다.
- Owner: Architect, Coder

### [R29-IMP-F002] 여러 문자 glyph가 첫 문자로 조용히 축약됨

- Pass: Implementation
- Pattern: IMP-003, TEST-001
- Area: content schema, typed conversion
- Severity: **Minor**
- Status: **Needs Fix**
- Summary: schema는 glyph를 `String`으로 수용하고 conversion은 첫 Unicode scalar만 취한다.
- Evidence: `crates/aihack-content/src/lib.rs:187-194`와 probe `registry=LONG runtime=L`.
- Expected: core glyph가 `char`이면 registry가 정확히 한 Unicode scalar만 수용해야 한다.
- Actual: 여러 문자를 silent truncate하고 content hash와 runtime 의미가 비직관적으로 분리된다.
- Impact: content typo가 시작 실패로 드러나지 않고 hash는 다른데 runtime 표현은 같은 비단사 변환을 만든다.
- Suggested Fix: item glyph를 exact-one scalar로 registry 단계에서 검사하고 empty/multi-scalar/valid Unicode fixture를 추가한다. monster glyph의 runtime 소비 여부는 별도 orphan 검토로 분리한다.
- Re-audit Method: `""`, `"AB"`, 결합문자 sequence와 한 scalar glyph를 검증한다.
- Owner: Coder

## 6. Pass 2 — Debug·Engineering Quality Findings

### [R29-DBG-F001] Repeat 방어가 key-code blocklist와 synthetic kind에 묶여 동등 transition을 놓침

- Pass: Debug
- Pattern: DBG-002, TEST-001
- Area: TUI dispatcher, state transition, terminal backend
- Severity: **Major**
- Status: **Needs Fix**
- Related: R28-DBG-F001, FIN-F008, FIN-F009, FIN-F016
- Summary: `runtime_event_to_candidate`는 Esc/Enter/F9 및 soft-input 밖 Quit q/Q의 `KeyEventKind::Repeat`만 차단한다. 다른 state-changing key와 backend가 Press로 전달하는 연속 입력은 같은 state-crossing 문제를 유지한다.
- Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:1589-1594`: 특정 key 계열만 열거한 narrow blocklist.
  - Title `l` Press→Load→Playing 뒤 `l` Repeat→Move(East), revision 변경.
  - Inventory `i` Repeat는 close/reopen, MorePrompt `i` Repeat는 Playing inventory, selection `b` Repeat는 Move(SouthWest).
  - installed `crossterm-0.29.0/src/event/sys/windows/parse.rs:204-292`: key-down/keyup을 Press/Release로만 만들고 repeat count를 사용하지 않는다.
  - actual ConPTY 한 write `\r\r`이 Title→CharacterCreation→Playing을 통과했다.
  - 기존 ConPTY test는 key를 한 번씩만 전송한다.
- Expected: ADR이 금지한 transition crossing은 특정 code/kind 열거가 아니라 실제 supported terminal transport에서 일관되게 강제되어야 한다.
- Actual: named synthetic Repeat는 차단하지만 equivalent transition과 repeated Press는 core/UI mutation을 만든다.
- Impact: Load 직후 이동·전투·trap, modal 자동 close/reopen, 선택 직후 이동과 두 화면 연속 confirm이 가능하다.
- Suggested Fix:
  1. state-changing candidate와 gesture lifecycle을 key code 열거와 분리해 설계한다.
  2. 실제 backend에서 구분 가능한 Press/Repeat/Release 범위를 spec에 기록하고 구분 불가능한 transport에는 안전한 transition suppression 또는 UX를 선택한다.
  3. Load, Inventory, MorePrompt, direction/inventory selection, GameOver, LLM transition 전체 matrix를 handler return/state/revision과 함께 검증한다.
  4. constructed Repeat뿐 아니라 ConPTY repeated-byte/native console 경계를 추가한다.
- Re-audit Method: 모든 state-changing key의 Press→Repeat, intervening Release 없는 Press→Press, Press→Release→Press를 분리해 각 pre/post-state에서 실행한다. 마지막 독립 입력은 정상 허용되는지 확인하고 실제 backend matrix 및 physical hold 주장 여부를 분리한다.
- Owner: Architect, Coder

### [R29-TEST-F001] Production-valid allocator exhaustion matrix가 영구 회귀로 보존되지 않음

- Pass: Debug
- Pattern: TEST-001
- Area: allocator regression evidence
- Severity: **Minor**
- Status: **Needs Fix**
- Summary: 구현은 독립 valid-save probe에서 안전하지만 permanent runtime test는 max ID가 약 12인데 `next_id`만 MAX로 바꾼 loader-invalid fixture와 bump 한 경로에 집중된다.
- Evidence: `crates/aihack-runtime/src/session.rs:642-663` 및 독립 MAX-2→MAX-1→MAX/bump/Throw/Zap PASS 결과.
- Expected: production validator를 통과하는 exact-successor fixture가 commit과 exhaustion rollback을 영구 고정해야 한다.
- Actual: valid load→last-ID commit→exhaustion 및 Throw/Zap rollback의 결합 경계가 drift해도 현재 단독 tests가 이를 모두 검출한다고 보장하지 못한다.
- Impact: 이번 독립 probe가 사라진 뒤 여러 계층을 함께 건드리는 allocator 회귀가 broad green 안에서 늦게 발견될 수 있다.
- Suggested Fix: valid load, 첫 corpse commit, 둘째 reject, Throw/Zap item·charge·RNG/full save/hash equality를 integration regression으로 승격한다.
- Re-audit Method: 새 회귀를 표적 실행하고 exact-successor 또는 rollback을 의도적으로 깨뜨렸을 때 RED인지 확인한다.
- Owner: Coder

### [R29-DBG-F002] 공개 low-level system API의 `Err` 원자성 계약이 불명확함

- Pass: Debug
- Pattern: DBG-002, SPEC-GAP-001
- Area: public module boundary, mutation ownership
- Severity: **Minor**
- Status: **Needs Spec Clarification**
- Summary: production `GameSession::submit`은 atomic하지만 public projectile/monster system을 직접 호출하면 allocation `Err` 전에 world/RNG가 바뀐다.
- Evidence:
  - `crates/aihack-runtime/src/lib.rs:11`은 systems를 공개하고 contract tests가 공개성을 고정한다.
  - `spec.md:233`은 production mutation entry를 `submit`으로 한정한다.
  - direct probe: death `world_equal=true`; Throw/Zap/monster plan은 `is_err=true`인데 `world_equal=false`, `rng_equal=false`.
  - `projectiles.rs:43-122`, `monster_ai.rs:64-99`는 mutation 뒤 corpse allocation 오류를 전파한다.
- Expected: 공개 system이 transaction-managed 호출 전용인지, `Err` atomic인지 명확해야 한다.
- Actual: public surface와 master mutation contract가 다른 기대를 만든다.
- Impact: workspace 또는 외부 consumer가 public API를 정상적인 fallible operation으로 사용하면 오류 뒤 partially mutated world/RNG를 계속 사용할 수 있다.
- Suggested Fix: `pub(crate)` 축소, non-atomic low-level 계약 명시, 또는 public atomic wrapper 중 하나를 선택한다.
- Re-audit Method: 외부 consumer compile contract와 direct `Err` world/RNG equality를 선택한 정책에 맞게 검사한다.
- Owner: Architect, Coder

## 7. Pass 3 — Security·Supply Chain Findings

### [R29-SEC-F001] Archive component/type 검증이 Windows-compatible extraction 의미를 닫지 못함

- Pass: Security
- Pattern: SEC-004, BUILD-001
- Area: ZIP/TAR raw entry, path/type/link/collision
- Severity: **Major**
- Status: **Needs Fix**
- Related: R25-SEC-F001, FIN-F014, SC-LICENSE-01
- Summary: 양 verifier가 공통으로 Windows 금지문자/device, raw entry type/link와 prefix·sanitizer collision을 놓치며 Bash는 비-ASCII case collision도 놓친다. Raw control은 명시 규칙이 아니라 `tar -t` escape에 의존해 현재 fixture가 간접 거부된다.
- Evidence:
  - `scripts/verify_release_bundle.ps1:141-167,198-212`.
  - `scripts/verify_release_bundle.sh:53-80,130-138`.
  - `crates/aihack-runtime/src/save.rs:850-873`은 같은 프로젝트 안에서 금지문자/control/`CONIN$`/`CONOUT$`/superscript device를 더 완전하게 거부한다.
  - 위 §2.3의 checksum-consistent ZIP/TAR false-green과 actual extraction failure/collision.
  - Microsoft는 Win32 금지문자·control과 `COM¹..³`/`LPT¹..³`을 공식적으로 reserved로 문서화한다: [Windows 파일명 규칙](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file). `CONIN$`/`CONOUT$`는 별도 [CreateFile console pseudofile](https://learn.microsoft.com/en-us/windows/win32/api/FileAPI/nf-fileapi-createfilea)이며 프로젝트 runtime 정책도 이를 금지한다.
- Expected: verifier가 raw archive name/type/link를 format-aware하게 읽고 target extraction에서 충돌·escape·실패할 entry를 fail-closed해야 한다.
- Actual: verifier PASS bundle이 Windows/GNU tar에서 이름을 합치거나 type을 재해석하거나 extraction에 실패한다.
- Impact: excluded source scope, corresponding-source 재현성과 archive 안전성 증거가 false-green이다.
- Suggested Fix:
  1. `tar -t` 표시 문자열 대신 format-aware ZIP/TAR parser로 raw name, type, link target을 검사한다.
  2. runtime path validator와 공통 Windows component 규칙을 공유한다.
  3. sanitizer/Unicode case/prefix collision과 entry type/link target을 expected manifest와 비교한다.
  4. 실제 extraction negative matrix를 양 OS gate에 추가한다.
- Re-audit Method: §2.3의 모든 fixture를 full checksum bundle로 양 verifier와 실제 extractor에 입력하고 전부 nonzero인지 확인한다.
- Owner: Coder, Security

### [R29-SEC-F002] Source archive가 `ExpectedCommit`의 complete tree와 결합되지 않음

- Pass: Security
- Pattern: SEC-006, BUILD-001
- Area: corresponding source, commit/tree identity, release verifier
- Severity: **Major**
- Status: **Needs Fix**
- Related: FIN-F014, SC-LICENSE-01
- Summary: verifier는 metadata의 commit 문자열과 필수 6개 파일만 확인하고 archive path/type/exported-content 집합을 expected Git tree와 비교하지 않는다.
- Evidence:
  - PowerShell `scripts/verify_release_bundle.ps1:198-227,259-286`.
  - Bash `scripts/verify_release_bundle.sh:126-160,187-190`.
  - `tests/release_bundle_windows.rs:140-155,318-333`의 positive fixture도 문서 6개 archive를 complete로 취급한다.
  - Rust source 0인 6-entry TAR가 checksum/metadata 재생성 후 exit 0.
  - current actual ZIP은 fresh `git archive HEAD`와 byte-identical 382/382로 정상.
- Expected: `ExpectedCommit`이 metadata text뿐 아니라 `export-ignore`와 `export-subst`가 적용된 archive path/type/exported-content hash exact set을 권위화해야 한다.
- Actual: 누락·대체·safe-name extra blob도 archive와 checksum을 함께 바꾸면 PASS한다.
- Impact: actual build가 현재 정상이어도 verifier/checksum만으로 complete corresponding source 또는 commit-bound source를 독립 증명할 수 없다.
- Suggested Fix:
  1. 검증 환경에서 `git archive ExpectedCommit`을 독립 재생성해 byte hash를 비교하거나,
  2. expected commit에서 생성한 path/type/exported-content hash manifest와 archive exact equality를 검증한다.
  3. 문서-only, crate 누락, Rust blob 변조, safe extra file와 mode/type/link mutation을 모두 negative로 고정한다.
- Re-audit Method: 현재 actual positive와 각 omission/substitution/type fixture를 양 verifier에 입력하고 expected tree가 아닌 모든 archive를 거부하는지 확인한다.
- Owner: Coder, Security, Release Manager

## 8. Cross-Pass Conflicts

| Conflict | 해소 판단 |
| --- | --- |
| 445 tests·양 OS CI green vs Major 5건 | 기존 fixture가 동등 transition/archive/authority 경계를 포함하지 않으므로 finding 유지 |
| named Esc/Enter Repeat tests green vs Load/Inventory/repeated Press mutation | 열거 기반 fix이므로 R28-DBG-F001 전체 closure 기각 |
| current source ZIP byte-identical vs 6-file substitute verifier PASS | current artifact positive와 verifier hard boundary를 분리하여 SEC-F002 유지 |
| classic archive alias tests green vs forbidden chars/type/prefix collisions | lexical matrix보다 extraction 의미가 넓으므로 SEC-F001 유지 |
| remediation이 spec/design/ADR 동결 주장 vs spec에 계약 부재 | master authority를 기준으로 DOC-F001 유지 |
| `r8_documentation` 10 PASS vs README/ADR/roadmap stale | document-wide semantic false-green이므로 R29-DOC-F002(R20-DBG-F008 recurrence) 유지 |
| item registry complete shape PASS vs Dagger/Armor split identity | field presence와 cross-field semantic identity가 다르므로 clarification 유지 |
| allocator permanent test가 좁음 vs independent valid probe PASS | 기능은 Verified, TEST-F001만 Minor로 유지 |

## 9. Verified로 유지하는 개선

- exact persisted allocator successor와 checked/fallible core allocation
- production-valid MAX-1/MAX exhaustion의 `GameSession::submit` no-panic/full rollback
- live monster HP 1..=10,000, accepted registry bootstrap initial/Wait/save/load
- 모든 known item kind의 source shape branch와 armor damage/hit negative regression; permanent full matrix는 armor 중심
- Drop/Throw/Quaff/Eat/Read common removal 및 equipped armor base AC 복원
- Report 28의 Judge/Inventory/StorageError/MorePrompt/CharacterCreation Esc와 Title Enter constructed Repeat cases
- 실제 F9 Press candidate/handler/flag/revision/hash와 Repeat/Release 차단
- year 0000 reject, 0001/9999 accept 양 OS calendar parity
- classic CON, colon/ADS, absolute, parent, trailing dot/space와 excluded root alias reject
- current actual source archive와 `git archive HEAD` byte equality
- causal 9종 field-only A/B와 나머지 8개 record equality
- terminal RAII/ConPTY, fresh staging, root/hardlink, action recursion과 dependency gates
- implementation `9725c378/32694375654` 및 current docs `d9c0f8e/32695945790` 양 OS success
- current 9-entry Windows bundle, cargo-audit/deny, R7/R8와 physical broken link 0

## 10. Rejected·Clarified·정보성 후보

- physical keyboard hold는 재현하지 않았다. TUI finding은 installed parser, constructed transition과 actual repeated-byte ConPTY에 한정한다.
- F9가 Title/CharacterCreation에서도 hidden flag를 바꾸는 동작은 낮은 영향이며 Playing 진입 후 표시되는 presentation state라 별도 blocker로 채택하지 않았다.
- Backspace/q Repeat의 source 동작은 독립 probe에서 정상이나 permanent event-level matrix가 좁다. R29-DBG-F001의 회귀 범위에 포함한다.
- unknown TOML field 무시와 materialize되지 않은 monster extreme stat은 현재 명시 계약·production spawn reachability가 부족하여 Info로 남긴다.
- actual source archive는 오염되지 않았다. SEC-F001/F002는 verifier가 future/substituted artifact를 fail-closed하지 못하는 finding이다.
- current docs HEAD run `32695945790` 미기록은 보고서 작성이 다시 HEAD를 바꾸므로 별도 finding으로 만들지 않는다.

## 11. PASS 전 필수 수정 순서

### P0 — Confirmed Major

1. `spec.md`를 먼저 갱신하고 ADR/CHANGELOG rollback 표현을 최종 `transaction_aborted` 범위와 맞춘다.
2. TUI transition gesture를 backend-aware하게 재설계하고 Load/Inventory/MorePrompt/selection/LLM/actual transport matrix를 추가한다.
3. archive raw component/type/link/prefix/extraction 규칙을 format-aware 공통 validator로 닫는다.
4. source archive를 `ExpectedCommit` tree/blob exact set과 결합한다.
5. README/ADR/audit roadmap current lifecycle을 복구하고 document-wide semantic negative gate를 만든다.

### P0-C — Needs Spec Clarification

6. item ID와 declared kind의 canonical 관계를 결정하고 모든 consumer의 authority를 하나로 정한다.

### P1 — Minor

7. item glyph exact-one-scalar validation을 추가하고 monster glyph orphan/consumer 범위를 별도 판정한다.
8. production-valid allocator exhaustion/Throw/Zap rollback probe를 영구 회귀로 승격한다.
9. public low-level system의 transaction/atomicity 계약을 닫는다.

## 12. Accepted Risks와 남은 제한

### 12.1 명시적 Accepted Risk

| Risk | Status | Owner | 수용 사유 | 영향 범위 | 만료·재검토 조건 |
| --- | --- | --- | --- | --- | --- |
| `hallucinating` SaveDataV1 compatibility orphan | **Accepted Risk** | Project owner / runtime maintainer | SaveDataV1 즉시 제거는 기존 wire/save 호환성을 불필요하게 깨뜨림 | R9 causal completeness에 한정, gameplay producer/consumer 없음 | SaveDataV2·v0.4.0 scope 승인 또는 2026-10-31 중 먼저 도래할 때 제거 migration과 실제 producer 중 하나 재결정 |

근거는 `spec.md:798`, `DESIGN_DECISIONS.md:274`이며 2026-08-24 현재 만료되지 않았다.

### 12.2 Excluded/known platform limits — Accepted Risk 아님

- runtime same-account concurrent directory-entry swap은 single-writer 제품 모델 밖이다.
- Windows parent-directory metadata power-loss durability는 `spec.md:731`의 OS/filesystem 제한이다.
- 실제 model provider, Windows Terminal GUI, signing/attestation/upload는 §1.2 제외 범위다.

위 accepted/excluded 범위는 transition crossing, archive false-green, incomplete source, stale authority 또는 item identity split을 허용하지 않는다.

## 13. Needs Spec Clarification

### 13.1 Item ID와 declared kind

- known ID가 canonical class를 강제하는가, custom registry가 기존 ID의 class를 바꿀 수 있는가?
- 허용한다면 `ItemKind` 기반 이름/CTA/causal behavior를 class 기반으로 바꿀 범위는 어디까지인가?
- 종료 조건: registry, runtime, TUI, ActionSpace, save가 같은 authoritative identity를 사용한다.

### 13.2 Public low-level systems

- public `systems::*`는 production API인가, transaction-managed 내부/테스트 primitive인가?
- `Err`가 world/RNG 원자성을 보장해야 하는가?
- 종료 조건: visibility, docs, contract tests와 direct error behavior가 같은 답을 제공한다.

## 14. 재감사 체크리스트

1. `spec.md`, ADR, CHANGELOG가 transition/archive/year와 aborted-vs-ordinary reject를 같은 범위로 정의한다.
2. Title Load, Inventory, MorePrompt, direction/inventory selection, GameOver와 LLM의 Press→Repeat, Release 없는 Press→Press, Press→Release→Press sequence를 분리해 검사한다.
3. 실제 Windows/Unix parser와 ConPTY repeated transport에서 결정한 gesture 정책이 유지된다.
4. F9 actual handler 및 기존 named Esc/Enter cases가 계속 green이다.
5. ZIP/TAR의 금지문자, 명시적 raw control, superscript device, console name, Unicode/sanitizer collision, prefix conflict를 거부한다.
6. expected manifest에 없는 symlink/hardlink/device type과 unsafe/escape link target을 raw entry 기준으로 거부한다.
7. current actual `git archive`는 accept하고 문서-only, crate omission, blob change, safe extra, mode/type mutation은 모두 reject한다.
8. README current steps/links, ADR supersede/current, audit roadmap pending gate가 report 29 lifecycle과 일치한다.
9. document test가 각 active section의 stale predecessor mutation을 거부한다.
10. ID-kind 정책에 따라 Dagger/Armor mutation과 모든 canonical pair가 adapter 전체에서 일치한다.
11. glyph empty/multi/one-scalar matrix가 typed reject/accept된다.
12. valid exact-successor allocator commit/exhaustion 및 Throw/Zap full rollback이 permanent integration test에 있다.
13. public low-level system visibility/atomicity가 결정된 계약과 일치한다.
14. Report 28에서 Verified된 HP/bootstrap, armor removal, calendar, causal 및 F9 경계를 재실행한다.
15. 아래 전체 gate를 단독 실행한다.

```text
git diff --check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --all-targets --locked -- --list
cargo build --workspace --release --all-targets --locked
cargo audit
cargo deny check licenses bans sources
Git Bash scripts/r7_checkpoint.sh
Git Bash scripts/r8_checkpoint.sh
build.bat --release
```

16. 새 clean implementation SHA의 Ubuntu/Windows same-SHA actual bundle을 확인한다.
17. 새 독립 감사가 report 29의 9개 finding과 FIN-F001~F018을 연결해 재판정한다.

## 15. 최종 근거와 Coder Handoff

### 최종 근거

- Report 28의 8개 finding을 production source, tests, local full gate, 두 세대 양 OS CI와 독립 adversarial fixture로 다시 대조했다.
- allocator, custom HP/bootstrap, armor removal, F9 실제 경로와 calendar는 Verified됐다.
- 그러나 TUI equivalent transition, archive extraction/commit tree hard boundary와 master/current documentation authority에서 Major 5건이 남았다.
- item ID-kind 계약도 adapter 불일치를 만든 상태로 미정이다.
- 따라서 `docs/audit/audit_report_28_remediation.md`의 same-SHA 기술 증거는 유효하지만 독립 PASS 조건은 충족하지 못했다. PROGRAM/PUBLICATION HOLD를 유지한다.

### Coder Handoff

```text
`C:\LocalDev\rust\AIHack\docs\audit\audit_report_29.md`의 독립 재감사 결과를 확인하고,
각 finding을 current spec/ADR, production entrypoint와 adversarial fixture에 대조하여 수정하세요.
문서를 먼저 갱신한 뒤 TUI transition gesture, archive raw component/type/extraction,
ExpectedCommit complete-source identity와 document-wide authority gate를 닫으세요.
item ID-kind 및 public system atomicity 계약을 결정하고 glyph/allocator regression도 보강하세요.
수정 후 전체 로컬 gate와 새 clean same-SHA Ubuntu/Windows actual bundle을 실행하여 결과를 기록하세요.
```
