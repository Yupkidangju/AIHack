# AIHack v0.3.0 감사 보고서 27 시정 독립 재감사 보고서 28

- 감사 대상: `docs/audit/audit_report_27_remediation.md`
- 기준 감사: `docs/audit/audit_report_27.md`
- 프로젝트: `C:\LocalDev\rust\AIHack`
- 감사 일자: 2026-08-24
- 시정 구현 SHA: `ea7822a5b32b3bb9ee8224176381c44871037bc4`
- 현재 HEAD: `01b2bd324f691d70600661fbb979885cbc6dc482`
- 브랜치: `codex/audit-report-27-remediation`
- 작업 트리: 감사 시작과 adversarial probe 정리 후 clean, 최종적으로 이 보고서만 추가
- 환경: Windows 11 Pro, `x86_64-pc-windows-msvc`, Asia/Seoul
- Rust/Cargo: 1.94.1
- 보안 도구: `cargo-audit 0.22.1`, `cargo-deny 0.19.4`
- 적용 기준: `AI_AUDIT_DOC_STANDARD.md`, `audit_roadmap.md`, `spec.md`, `designs.md`, `AGENTS.md`
- 추가 검토 기준: `code-review-and-quality`, `security-and-hardening`
- 감사 원칙: 구현·테스트·설정·기존 통제 문서는 수정하지 않고 이 보고서만 추가한다.

## 0. 최종 판정

**HOLD — REPORT 27 REMEDIATION PARTIAL / INDEPENDENT PASS 기각**

Report 27 시정은 실제로 큰 진전을 만들었다. active level·stairs·charge save 검증, armor AC 범위와 일반 Wear→Drop 복원, 9종 field-only causal A/B, F9 visible rect mouse consume, Judge 문자 반복, archive의 명시 dot/parent/absolute/backslash matrix, 일반 calendar 오류, repository-root local action 재귀 검증은 production 경로와 회귀 테스트에 연결됐다. 구현 SHA `ea7822a5`의 Actions `32683076204`와 현재 문서 SHA `01b2bd3`의 Actions `32684490662`도 Ubuntu/Windows 전체 gate 및 실제 플랫폼 bundle을 통과했다.

그러나 열거된 fixture의 인접 경계를 독립적으로 확장한 결과 **Confirmed Major 4건**이 남았다.

1. `next_id=u32::MAX-1` save는 수용되고, 첫 spawn이 `next_id=u32::MAX`인 저장 불가능 상태를 commit한 뒤 두 번째 spawn에서 debug overflow panic을 낸다.
2. custom monster `hp=0` registry/session이 수용되어 시작 직후부터 save round-trip 불가능한 live actor를 만든다.
3. custom armor에 `damage`를 함께 넣을 수 있어 장착 armor가 Throw 가능해지고, Throw가 공통 unequip 경계를 우회하여 AC와 save 불변식을 깨뜨린다.
4. release archive verifier가 차단 root의 대소문자 및 trailing dot/space Windows 별칭 아래 entry를 포함한 ZIP을 승인한다.

**Major 영향 후보 `Needs Spec Clarification` 1건**도 있다.

- dispatcher는 Esc/Enter `Repeat`를 state 전환 뒤 새 state의 Quit 또는 추가 전환으로 재해석한다. 다만 현재 통제 문서는 이 control-key gesture 정책을 정의하지 않았고 실제 키 홀드/ConPTY 도달성도 확인하지 않았으므로 confirmed implementation defect로 단정하지 않는다.

추가로 **Minor 3건**이 있다.

- Linux verifier는 Gregorian/.NET 공통 범위 밖인 연도 `0000`을 승인하여 양 OS accept/reject 판정이 다르다. 현재 production build가 year-0000 candidate를 생성하는 경로는 확인되지 않았다.
- F9 core-hash 회귀가 실제 F9→candidate→handler 경로를 전혀 실행하지 않는 동어반복 테스트다.
- `IMPLEMENTATION_SUMMARY.md` 후반부가 이미 완료된 report 27 전체 gate와 새 CI를 여전히 다음 단계로 기록한다.

따라서 현재 정확한 상태는 **NORMAL GATES GREEN / ADVERSARIAL EQUIVALENT BOUNDARIES AND ONE CONTROL-KEY CONTRACT OPEN / PROGRAM AND PUBLICATION HOLD**다. Report 27 remediation의 same-SHA evidence는 정확하지만, 열거된 positive fixture의 성공을 전체 계약 closure로 확장할 수 없다.

## 1. 감사 범위와 제한

### 1.1 확인한 구현·문서·증거

- `1e84a94..ea7822a`: report 27 save/content/causal/TUI/release/action 시정 구현
- `ea7822a..01b2bd3`: 시정 증거 문서와 active documentation regression 갱신
- save/runtime: semantic validator, allocator, transaction commit, headless production entrypoint
- content/runtime: registry schema, monster/item 변환, bootstrap, equipment/drop/throw lifecycle
- causal: 동일 command/observer 9종 field-only A/B와 full record equality
- TUI: 단일 dispatcher, key kind, overlay/soft-input/state transition, F9 debug surface
- release/supply chain: 양 OS verifier, source archive entry, calendar, fresh staging, path alias, local action graph, dependency gates
- active spec/ADR/README/BUILD/CHANGELOG/implementation summary와 report 27 remediation evidence lineage

### 1.2 제외 범위

- 외부 실제 LLM provider smoke는 v0.3.0 비목표다.
- Windows Terminal GUI의 pixel/font rendering은 제외하고 ConPTY·dispatcher·terminal restoration을 확인했다.
- `legacy_nethack_port_reference/` 본문은 제품 범위 밖이며 runtime import와 release archive 차단만 감사했다.
- 외부 tag/release/publish 및 Git commit/push는 수행하지 않았다.
- runtime same-account concurrent directory-entry swap은 기존 single-writer threat model 밖이다.
- artifact signing/attestation과 외부 업로드는 현재 release bundle 필수 계약 밖이다.

### 1.3 감사 도구 제한

적용한 skill이 참조하는 다음 세부 파일은 설치본에 없었다.

- `code-review-and-quality/references/security-checklist.md`
- `code-review-and-quality/references/performance-checklist.md`
- `security-and-hardening/references/security-checklist.md`

각 skill의 본문 체크리스트와 프로젝트의 `AI_AUDIT_DOC_STANDARD.md`로 대체했다. 이는 프로젝트 finding이 아니라 감사 환경 제한이다.

## 2. 실행·검증 증거

### 2.1 로컬 전체 gate

| 명령 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked -- --list` | named test **437개** |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| `cargo audit` | PASS, 318 dependencies, vulnerabilities 0 |
| `cargo deny check licenses bans sources` | PASS, bans/licenses/sources ok |
| dependency exception/duplicate gate | PASS, duplicate family 24/24 |
| Git Bash `scripts/r7_checkpoint.sh` | PASS |
| Git Bash `scripts/r8_checkpoint.sh` | PASS |
| `build.bat --release` | PASS, Windows 9-entry exact bundle, commit `01b2bd3` |
| active Markdown relative-link scan | PASS, broken 0 |
| `cargo test --locked -p aihack --test r8_documentation` after report 28 authoring | 초안 code block의 PowerShell type syntax를 reference link로 오인해 1회 FAIL; 표현 수정 후 **9 PASS** |

전체 gate의 성공은 정상 fixture와 이미 이름 붙은 회귀가 green임을 증명한다. 아래 적대적 fixture는 기존 437개 테스트가 다루지 않는 동등 경계를 검사하므로 이 결과와 모순되지 않는다.

### 2.2 원격 evidence lineage

| SHA / Actions | Ubuntu | Windows | 판정 |
| --- | --- | --- | --- |
| `ea7822a5` / [`32683076204`](https://github.com/Yupkidangju/AIHack/actions/runs/32683076204) | PASS | PASS | report 27 production 시정 구현의 same-SHA 전체 gate와 actual bundle evidence |
| `01b2bd32` / [`32684490662`](https://github.com/Yupkidangju/AIHack/actions/runs/32684490662) | PASS | PASS | docs/test-only 후속 HEAD의 전체 gate와 actual bundle evidence |

`ea7822a..01b2bd3`은 문서와 `tests/r8_documentation.rs`만 변경했다. 따라서 remediation이 구현 authority를 `ea7822a/32683076204`로 기록한 것은 정확하다. 현재 HEAD run은 추가 current-tree evidence이며, 이번 감사 보고서 추가로 다시 HEAD가 바뀐다는 이유만으로 구현 증거를 무효화하지 않는다.

### 2.3 독립 adversarial probe와 source derivation

감사 전용 source와 randomized fixture directory는 실행 후 삭제했고 프로젝트 구현 파일은 변경하지 않았다. 보안 probe의 ignored compiled helper/PDB만 `target/`에 남아 있다.

```text
SAVE_NEXT_ID=4294967294
FIRST_SPAWN_ACCEPTED=true
NEXT_ID_AFTER_FIRST=4294967295
SECOND_SPAWN_PANIC=true
PANIC=entity.rs:617: attempt to add with overflow
```

```text
CUSTOM_MONSTER_HP_ZERO_REGISTRY_OK=true
CUSTOM_MONSTER_HP_ZERO_SESSION_OK=true
WAIT_ACCEPTED=true
INITIAL_ROUNDTRIP_OK=false
AFTER_WAIT_ROUNDTRIP_OK=false
```

```text
CUSTOM_ARMOR_DAMAGE=1d4
REGISTRY_OK=true
WEAR_ACCEPTED=true
AC_AFTER_WEAR=-1
THROW_ACCEPTED=true
EQUIPPED_BODY=None
AC_AFTER_THROW=-1
ROUNDTRIP_OK=false
```

아래 TUI 결과는 별도 ConPTY key-hold 실행값이 아니라 production dispatcher와 handler의 source-level deterministic derivation이다.

```text
CONSTRUCTED_SEQUENCE=Judge Esc Press -> LlmCancelInput
NEXT_DISPATCH=Judge Esc Repeat -> Playing Quit
QUIT_HANDLER_RETURN=true

CONSTRUCTED_SEQUENCE=Inventory Esc Press -> CloseOverlay
NEXT_DISPATCH=Inventory Esc Repeat -> Playing Quit
QUIT_HANDLER_RETURN=true
```

동일 source path가 `StorageError`, `MorePrompt`, `CharacterCreation`에도 이어진다. Title의 constructed Enter Press→Repeat는 CharacterCreation을 지나 Playing까지 전환한다. 실제 terminal key hold가 Esc/Enter Repeat를 내는지는 제외했으며 정책도 현재 문서에 없다.

```text
WINDOWS_ARCHIVE_ALIAS=LEGACY_NETHACK_PORT_REFERENCE/probe.txt
VERIFIER_EXIT=0

WINDOWS_ARCHIVE_ALIAS=legacy_nethack_port_reference./probe.txt
VERIFIER_EXIT=0

WINDOWS_ARCHIVE_ALIAS=legacy_nethack_port_reference /probe.txt
VERIFIER_EXIT=0
```

각 ZIP fixture는 current Windows bundle 복사본에 entry를 추가하고 checksum을 다시 만든 완전한 bundle이며 verifier가 PASS했다.

```text
CANDIDATE_DATE=0000-06-15
COVERED_PERIOD=0000-01-01..0000-12-31
LINUX_VERIFIER_EXIT=0
```

### 2.4 Adversarial probe 명령과 fixture 기록

#### Allocator

실행 당시 임시 manifest/source는 `target/r27-allocator-probe/`, 별도 build는 `target/r27-allocator-build/`에 두었고 종료 후 삭제했다.

```powershell
$env:CARGO_TARGET_DIR='C:\LocalDev\rust\AIHack\target\r27-allocator-build'
cargo run --quiet --manifest-path 'C:\LocalDev\rust\AIHack\target\r27-allocator-probe\Cargo.toml'
```

probe는 정상 `SaveDataV1` JSON의 `next_id`를 `u32::MAX-1`로 바꾸고 player 양옆에 1 HP stationary Jackal 두 마리를 둔 뒤 hit bonus 100으로 `Move(East)`, `Move(West)`를 제출했다. 두 번째 submit은 `catch_unwind`로 관찰했기 때문에 probe process exit는 0이고 내부 panic은 `SECOND_SPAWN_PANIC=true`로 기록됐다.

#### Custom monster와 armor Throw

실행 당시 임시 manifest/source와 target은 아래 경로였고 절대 경로를 확인한 뒤 삭제했다.

```text
C:\LocalDev\rust\AIHack-r26-save-probe\Cargo.toml
C:\LocalDev\rust\AIHack-r26-save-probe\src\main.rs
C:\LocalDev\rust\AIHack-r26-save-target\
```

```powershell
$env:CARGO_TARGET_DIR='C:\LocalDev\rust\AIHack-r26-save-target'
cargo run --quiet --manifest-path 'C:\LocalDev\rust\AIHack-r26-save-probe\Cargo.toml'
```

HP probe는 embedded monster TOML의 Jackal hp만 0으로 바꾸고 `ContentRegistry::from_toml_sources`, `GameSession::try_new_for_playing_with_registry`, initial round-trip, Wait, after-Wait round-trip 순서로 실행했다. Armor probe는 armor TOML에 `damage="1d4"`를 추가하고 Pickup→Wear→Throw 뒤 equipped pointer, player AC와 round-trip을 검사했다. 두 probe process exit는 0이며 false 값 자체가 기대한 negative failure evidence다.

#### TUI control-key Repeat

별도 임시 test나 실제 key-hold probe는 실행하지 않았다. 아래 existing suite만 실행했고 transition 결과는 `runtime_event_to_candidate`와 `handle_candidate_owned` source의 결정적 합성 결과다.

```powershell
cargo test --locked -p aihack-tui --all-targets
cargo test --quiet --locked -p aihack-tui --test tui_contract --test conpty_contract
```

재감사에서는 다음과 같은 production API sequence를 영구 회귀로 추가해야 한다.

```text
TuiApp::new_with_llm_enabled(...)
handle_candidate_owned(LlmJudge)
runtime_event_to_candidate(Esc Press) -> LlmCancelInput
handle_candidate_owned(...) -> false
runtime_event_to_candidate(Esc Repeat) -> 현재 source에서는 Quit
handle_candidate_owned(...) -> 현재 source에서는 true
runtime_event_to_candidate(Esc Release) -> None
```

같은 방식으로 Inventory, StorageError, MorePrompt, CharacterCreation의 Esc와 Title Enter를 검사한다. 정책 동결 뒤 권장 표적 명령은 다음과 같다.

```powershell
cargo test --locked -p aihack-tui --test tui_contract control_key_repeat_does_not_cross_state_boundaries -- --exact --nocapture
```

#### Windows archive alias

current `output/`을 GUID 임시 디렉터리에 복사하고 alias entry를 ZIP에 추가한 뒤 8개 payload checksum을 다시 생성하여 production PowerShell verifier를 호출했다. 임시 디렉터리는 각 iteration 종료 후 삭제했다.

```powershell
$auditRepo = 'C:\LocalDev\rust\AIHack'
$auditHead = (git -C $auditRepo rev-parse HEAD).Trim()
$auditDate = (git -C $auditRepo show -s --format=%cs HEAD).Trim()
$auditAliases = @(
  'LEGACY_NETHACK_PORT_REFERENCE/probe.txt',
  'legacy_nethack_port_reference./probe.txt',
  'legacy_nethack_port_reference /probe.txt'
)

foreach ($auditAlias in $auditAliases) {
  $auditTemp = Join-Path $env:TEMP ('aihack-r27-alias-' + [guid]::NewGuid().ToString('N'))
  $auditOutput = Join-Path $auditTemp 'output'
  New-Item -ItemType Directory -Path $auditOutput | Out-Null
  Copy-Item "$auditRepo\output\*" $auditOutput -Force

  Add-Type -AssemblyName System.IO.Compression
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  $auditArchive = Join-Path $auditOutput 'aihack-0.3.0-source.zip'
  $auditZip = [IO.Compression.ZipFile]::Open($auditArchive, [IO.Compression.ZipArchiveMode]::Update)
  try {
    $auditEntry = $auditZip.CreateEntry($auditAlias)
    $auditWriter = [IO.StreamWriter]::new($auditEntry.Open())
    try { $auditWriter.Write('blocked') } finally { $auditWriter.Dispose() }
  } finally { $auditZip.Dispose() }

  $auditNames = @(
    'aihack.exe', 'aihack-headless.exe', 'LICENSE', 'NOTICE',
    'MODIFICATIONS.md', 'PROJECT_OWNER_LICENSE_APPROVAL.md',
    'RELEASE-METADATA', 'aihack-0.3.0-source.zip'
  )
  $auditSums = foreach ($auditName in $auditNames) {
    $auditHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $auditOutput $auditName)).Hash.ToLowerInvariant()
    "$auditHash  $auditName"
  }
  $null = [IO.File]::WriteAllLines((Join-Path $auditOutput 'SHA256SUMS'), $auditSums, [Text.Encoding]::ASCII)

  & powershell -NoProfile -ExecutionPolicy Bypass `
    -File "$auditRepo\scripts\verify_release_bundle.ps1" `
    -OutputDir $auditOutput `
    -ExpectedCommit $auditHead `
    -ExpectedCandidateDate $auditDate
  "alias=$auditAlias exit=$LASTEXITCODE"
  Remove-Item -LiteralPath $auditTemp -Recurse -Force
}
```

세 alias 모두 exit 0이었다. verifier가 reject해야 하는 negative fixture였으므로 이 exit 0이 실패 증거다.

#### Linux year 0000

감사 helper는 GUID temp 아래 complete Linux fixture를 생성하고 source/output의 `MODIFICATIONS.md`와 `RELEASE-METADATA`를 year-0000 값으로 일치시킨 뒤 tar와 checksum을 만들었다. 감사 당시 binary invocation과 결과는 다음과 같다.

```powershell
.\target\security-r27-probe.exe
```

```text
YEAR_ZERO_EXIT=Some(0)
STDOUT=PASS release bundle: version=0.3.0 commit=01b2bd324f691d70600661fbb979885cbc6dc482
```

helper source와 randomized fixture directory는 보존하지 않았고 ignored binary/PDB만 `target/security-r27-probe.*`에 남아 있다. production verifier의 핵심 호출은 Git Bash에서 다음과 동등했다.

```text
scripts/verify_release_bundle.sh output <current-head> 0000-06-15
```

binary에 의존하지 않는 최소 complete fixture 재현 순서는 다음과 같다.

```powershell
$auditRepo = 'C:\LocalDev\rust\AIHack'
$auditTemp = Join-Path $env:TEMP ('aihack-r27-year0-' + [guid]::NewGuid().ToString('N'))
$auditSource = Join-Path $auditTemp 'source'
$auditOutput = Join-Path $auditTemp 'output'
$null = New-Item -ItemType Directory -Path $auditSource, $auditOutput
$auditHead = (git -C $auditRepo rev-parse HEAD).Trim()
$auditUtf8 = [Text.UTF8Encoding]::new($false)
$auditAscii = [Text.Encoding]::ASCII

$auditMetadata = @"
product=AIHack
version=0.3.0
commit=$auditHead
candidate_date=0000-06-15
source_license=NGPL
modification_notice=AIHACK-MODIFICATIONS-2026-08-24-01
owner_approval=AIHACK-OWNER-2026-07-20-NGPL-01
"@
$auditApproval = 'Approval ID: `AIHACK-OWNER-2026-07-20-NGPL-01`'
$auditModifications = @'
Notice ID: `AIHACK-MODIFICATIONS-2026-08-24-01`
Covered change period: `0000-01-01..0000-12-31`
'@

$null = [IO.File]::WriteAllText((Join-Path $auditSource 'LICENSE'), "fixture license`n", $auditUtf8)
$null = [IO.File]::WriteAllText((Join-Path $auditSource 'NOTICE'), "fixture notice`n", $auditUtf8)
$null = [IO.File]::WriteAllText((Join-Path $auditSource 'Cargo.toml'), "[package]`nname='fixture'`nversion='0.0.0'`n", $auditUtf8)
$null = [IO.File]::WriteAllText((Join-Path $auditSource 'RELEASE-METADATA'), $auditMetadata, $auditUtf8)
$null = [IO.File]::WriteAllText((Join-Path $auditSource 'PROJECT_OWNER_LICENSE_APPROVAL.md'), $auditApproval, $auditUtf8)
$null = [IO.File]::WriteAllText((Join-Path $auditSource 'MODIFICATIONS.md'), $auditModifications, $auditUtf8)

Copy-Item (Join-Path $auditSource 'LICENSE') $auditOutput
Copy-Item (Join-Path $auditSource 'NOTICE') $auditOutput
Copy-Item (Join-Path $auditSource 'RELEASE-METADATA') $auditOutput
Copy-Item (Join-Path $auditSource 'PROJECT_OWNER_LICENSE_APPROVAL.md') $auditOutput
Copy-Item (Join-Path $auditSource 'MODIFICATIONS.md') $auditOutput
$null = [IO.File]::WriteAllText((Join-Path $auditOutput 'aihack'), 'fixture binary', $auditAscii)
$null = [IO.File]::WriteAllText((Join-Path $auditOutput 'aihack-headless'), 'fixture binary', $auditAscii)

Push-Location $auditSource
tar.exe -czf (Join-Path $auditOutput 'aihack-0.3.0-source.tar.gz') `
  LICENSE NOTICE MODIFICATIONS.md PROJECT_OWNER_LICENSE_APPROVAL.md `
  RELEASE-METADATA Cargo.toml
Pop-Location

Push-Location $auditOutput
& 'C:\Program Files\Git\usr\bin\sha256sum.exe' `
  aihack aihack-headless LICENSE NOTICE MODIFICATIONS.md `
  PROJECT_OWNER_LICENSE_APPROVAL.md RELEASE-METADATA `
  aihack-0.3.0-source.tar.gz |
  Set-Content -Encoding Ascii SHA256SUMS
Pop-Location

Push-Location $auditTemp
& 'C:\Program Files\Git\bin\bash.exe' `
  "$auditRepo\scripts\verify_release_bundle.sh" `
  output $auditHead 0000-06-15
$auditExit = $LASTEXITCODE
Pop-Location
"YEAR_ZERO_EXIT=$auditExit"
Remove-Item -LiteralPath $auditTemp -Recurse -Force
```

별도 `date -u -d '0000-01-01' '+%F'`도 `0000-01-01`, exit 0을 반환했다. production build의 year-0000 candidate 도달성을 검증하지 않은 점은 이 Minor finding의 증거 제한이다.

## 3. Report 27 finding 재감사 상태

| 원 finding | 재감사 상태 | 근거 |
| --- | --- | --- |
| R25-IMP-F001 Re-audit #2 | **Needs Fix** | max sentinel은 막았지만 `MAX-1` 두 spawn과 custom monster invariant가 열림 |
| R27-IMP-F001 | **Needs Fix** | AC extreme과 일반 Drop은 수정됐지만 armor `damage`→Wear→Throw가 derived AC를 깨뜨림 |
| R25-IMP-F002 Re-audit #2 | **Verified** | 9종 동일 command/observer, field-only neutralization, exactly-one loss, 나머지 8개 full record equality |
| R26-DOC-F001 | **Verified** | 구현 SHA/run과 predecessor lineage가 active authority에 정렬됨 |
| R27-IMP-F002 | **Verified** | renderer/dispatcher가 F9 visible rect를 공유하고 mouse를 consume함 |
| R27-DOC-F001 | **Verified** | active ADR/changelog가 report 27 HOLD와 final successor를 기록함 |
| R26-IMP-F001 Re-audit #1 | **Verified** | 보고된 Judge G/A/J/R Press·Repeat 입력과 Release 차단은 정상; 제어키 transition repeat는 새 clarification으로 분리 |
| R25-SEC-F001 Re-audit #2 | **Needs Fix** | 명시 dot/parent는 수정됐으나 Windows case/trailing component alias가 열림 |
| FIN-F015 Re-audit #3 | **Needs Fix** | 일반 invalid calendar는 수정됐으나 Linux year 0000 승인; local action scope는 Verified |
| R25-SEC-F003 Re-audit #2 | **Verified** | root local→local 재귀, cycle/missing/escape reject, pinned chain accept |

## 4. FIN-F001~F018 재판정

| ID | Report 28 상태 |
| --- | --- |
| FIN-F001 | **Needs Fix** — allocator exhaustion과 accepted custom monster invalid world |
| FIN-F002 | **Verified** — byte/cardinality/RNG/text와 write no-clobber |
| FIN-F003 | **Verified** — replay self-verification/no-partial-commit |
| FIN-F004 | **Verified** — Windows artifact path alias matrix |
| FIN-F005 | **Verified** — external mutable state 우회 제거 |
| FIN-F006 | **Verified** — 9종 field-only causal A/B와 structural equality |
| FIN-F007 | **Needs Fix** — accepted armor의 Wear→Throw가 equipment/derived AC/save 불변식 파손 |
| FIN-F008 | **Needs Spec Clarification** — 개별 modal isolation은 동작하지만 control-key Repeat의 state-crossing 정책이 문서에 없음 |
| FIN-F009 | **Needs Spec Clarification** — Inspect/debug mouse는 Verified; control-key policy 미정이며 F9 실제 경로 증거는 Minor Needs Fix |
| FIN-F010 | **Verified** — terminal RAII/ConPTY/evidence 정렬 |
| FIN-F011 | **Verified** — CLI default/range/docs |
| FIN-F012 | **Needs Documentation Recovery** — final evidence는 정렬됐으나 implementation summary의 다음 단계가 stale |
| FIN-F013 | **Verified** — parsed exception/current/future lifecycle |
| FIN-F014 | **Needs Fix** — Windows-compatible archive component canonicalization 미완료 |
| FIN-F015 | **Needs Fix** — Minor year 0000 양 OS calendar parity 불일치; action recursion은 Verified |
| FIN-F016 | **Needs Spec Clarification** — Judge 문자 repeat는 Verified, control-key transition repeat 정책 미정 |
| FIN-F017 | **Verified** — package tests/duplicate metadata/current 양 OS evidence |
| FIN-F018 | **Verified** — 동결된 single-writer threat model 범위 |

## 5. Pass 1 — 구현·문서 정합성 Findings

### [R25-IMP-F001 — Re-audit #3] allocator max sentinel 거부가 usable headroom을 보장하지 않음

- Pass: Implementation
- Pattern: IMP-003, TEST-001
- Area: save semantic validation, entity allocation, transaction safety
- Severity: **Major**
- Status: **Needs Fix**
- Related: FIN-F001
- Summary: validator는 `next_id == u32::MAX`만 추가 거부하고 `u32::MAX-1`을 수용한다. allocator의 unchecked increment 때문에 첫 spawn이 저장 불가능 상태를 commit하고 다음 spawn이 panic한다.
- Evidence:
  - `crates/aihack-runtime/src/save.rs:576-580`: `next_id == 0`, `<= max_id`, `== u32::MAX`만 거부한다.
  - `crates/aihack-core/src/domain/entity.rs:615-618`: `self.next_id += 1`이 fallible하지 않다.
  - 직접 probe: 첫 spawn 후 `next_id=4294967295`, 두 번째 spawn에서 `attempt to add with overflow`.
- Expected: loader가 수용한 state와 이후 accepted command는 panic이나 invalid committed state를 만들지 않아야 한다. ID 고갈은 load 전에 거부하거나 spawn transaction이 typed/fallible error로 원자적으로 거부해야 한다.
- Actual: `MAX-1` save가 load되고 첫 spawn이 commit되며, 두 번째 spawn이 debug panic한다.
- Impact: 조작되거나 오래 진행된 save가 production command로 프로세스를 중단시키고, 그 직전 state도 다음 save가 불가능하다.
- Suggested Fix:
  1. entity allocation을 `checked_add` 기반 fallible API로 바꾸고 모든 spawn caller가 typed error를 transaction 밖으로 전파하게 한다.
  2. entity tombstone을 제거하지 않는 현재 모델을 유지한다면 persisted `next_id == max_id.checked_add(1)`을 검증하여 임의 headroom gap을 차단한다.
  3. `MAX`, `MAX-1`, 첫 spawn, 두 번째 spawn 및 rejected transaction의 RNG/hash/state 불변을 회귀로 고정한다.
- Re-audit Method: production load/headless/session entrypoint에서 두 spawn fixture를 실행하고 panic 없음, typed rejection, no-partial-commit, 다음 save 가능 여부를 함께 검사한다.
- Owner: Coder

### [R28-IMP-F001] accepted custom monster가 시작부터 invalid live world를 생성함

- Pass: Implementation
- Pattern: IMP-003, TEST-001
- Area: content registry, monster conversion, world bootstrap
- Severity: **Major**
- Status: **Needs Fix**
- Related: R25-IMP-F001, FIN-F001
- Summary: content registry가 monster `hp=0`을 수용하고 bootstrap은 actor를 `alive=true`로 생성한다. 만들어진 session은 command를 받을 수 있지만 초기 상태와 Wait 후 상태 모두 save validator를 통과하지 못한다.
- Evidence:
  - `crates/aihack-content/src/schema.rs:247-260`: monster는 speed, positive difficulty와 dice 문법만 검사하고 live actor HP를 검사하지 않는다.
  - `crates/aihack-content/src/lib.rs:118-129`: registry hp를 template hp로 그대로 복사한다.
  - `crates/aihack-core/src/domain/entity.rs:334-376`: template hp를 actor에 넣고 `alive=true`로 생성한다.
  - `crates/aihack-runtime/src/session.rs:48-74`: custom registry bootstrap 뒤 world invariant 검증 없이 session을 반환한다.
  - 직접 probe: registry/session/Wait는 성공, initial 및 after-Wait round-trip은 모두 실패했다.
- Expected: accepted registry는 즉시 consumer-safe하고 save 가능한 world만 생성해야 한다. `alive=true` monster의 hp/max_hp는 world invariant를 만족해야 한다.
- Actual: hp 0인 invalid live actor가 정상 session으로 materialize된다.
- Impact: mod/custom content가 시작 즉시 저장 불가능한 run을 만들 수 있다. 오류가 registry parsing 단계가 아니라 늦은 save 단계에서 나타난다.
- Suggested Fix:
  1. live monster hp/max_hp의 최소 유효 범위를 registry validation에 둔다.
  2. registry validation과 world invariant가 같은 HP 계약을 공유하게 한다.
  3. custom-registry bootstrap 반환 전에 `GameWorld`의 전체 persisted invariant를 검사한다.
  4. hp 0/negative reject fixture와 accepted custom registry save round-trip을 추가한다.
- Re-audit Method: malformed registry가 session 생성 전에 typed `ContentError`로 실패하고, 모든 accepted boundary fixture가 initial/Wait/save/load를 통과하는지 검사한다.
- Owner: Coder

### [R27-IMP-F001 — Re-audit #1] armor field shape와 Throw가 공통 unequip lifecycle을 우회함

- Pass: Implementation
- Pattern: IMP-003, TEST-001
- Area: item schema, equipment, projectile, derived state
- Severity: **Major**
- Status: **Needs Fix**
- Related: FIN-F001, FIN-F007
- Summary: armor kind validation이 `damage`/`hit_bonus` 부재를 요구하지 않는다. damage가 있는 custom armor는 attack profile을 받아 Throw 가능하며, projectile path의 직접 inventory removal이 AC 복원 helper를 건너뛴다.
- Evidence:
  - `crates/aihack-content/src/schema.rs:348-354`: armor는 slot/ac/effect/charges/nutrition만 확인하고 damage와 hit_bonus를 금지하지 않는다.
  - `crates/aihack-content/src/lib.rs:195-210`: kind와 무관하게 damage가 있으면 attack profile을 만든다.
  - `crates/aihack-runtime/src/systems/projectiles.rs:25-53`: Throw는 attack profile만 확인한 뒤 `inventory.remove(item)`을 직접 호출한다.
  - `crates/aihack-core/src/domain/inventory.rs:39-47`: remove는 equipped pointer만 지우고 derived AC를 복원하지 않는다.
  - `crates/aihack-runtime/src/systems/items.rs:278-303`: 정상 remove 경로의 `unequip_body`만 adventurer base AC를 복원한다.
  - 직접 probe: Wear 후 AC `-1`, Throw accepted, `equipped_body=None`인데 AC `-1`, 다음 round-trip 실패.
- Expected: accepted kind shape는 의도된 consumer 집합만 활성화해야 하며, inventory에서 장착 item을 제거하는 모든 경로가 단일 unequip/derived-state lifecycle을 사용해야 한다.
- Actual: schema가 비의도 projectile capability를 열고 Throw가 lifecycle choke point를 우회한다.
- Impact: accepted custom content와 정상 command만으로 내부 상태가 모순되고 이후 save가 실패한다.
- Suggested Fix:
  1. armor에는 `damage`와 `hit_bonus`가 없어야 함을 kind-specific shape에 추가한다. 다른 kind도 required/forbidden field를 완전한 표로 검증한다.
  2. Throw/drop/consume/destroy 등 inventory removal을 하나의 공통 fallible helper로 통합한다.
  3. 다중 역할 armor를 의도한다면 금지 대신 Wear→Throw의 unequip·AC restoration을 원자적으로 구현하고 명세를 먼저 갱신한다.
  4. accepted item kind별 활성 consumer 목록과 Wear→Drop/Throw/save round-trip matrix를 회귀로 만든다.
- Re-audit Method: damage/hit bonus armor가 registry에서 거부되는지, 또는 명시적으로 허용한 경우 Wear→Throw 후 base AC/equipment/location/save가 모두 일치하는지 확인한다.
- Owner: Architect, Coder

### [R27-DOC-F002 — Re-audit #1] implementation summary가 완료된 gate를 다음 단계로 유지함

- Pass: Implementation
- Pattern: DOC-BACKFILL-001
- Area: active implementation authority, evidence lifecycle
- Severity: **Minor**
- Status: **Needs Documentation Recovery**
- Related: FIN-F012
- Summary: 문서 앞부분은 `ea7822a/32683076204` 완료를 정확히 기록하지만 후반 active plan은 같은 gate와 새 CI를 여전히 pending으로 둔다.
- Evidence:
  - `IMPLEMENTATION_SUMMARY.md:18-20`: report 27 구현과 Actions success를 기록한다.
  - `IMPLEMENTATION_SUMMARY.md:926`: 다음 단계를 report 27 전체 local gate와 새 clean same-SHA CI라고 기록한다.
  - `IMPLEMENTATION_SUMMARY.md:934`: “새 CI와 독립 재감사 전”이라고 기록하지만 새 CI는 이미 완료됐다.
  - `tests/r8_documentation.rs:195-214`: summary 1절 중심 검사여서 10·11절의 stale 상태를 놓친다.
- Expected: active 문서의 모든 현재 상태 문장이 same evidence lifecycle을 가리키고, 남은 gate는 독립 재감사와 게시 승인으로 한정되어야 한다.
- Actual: 같은 문서 안에서 완료/미완료가 동시에 존재한다.
- Impact: 후속 작업자가 이미 완료된 CI를 다시 요구하거나 현재 HOLD의 실제 원인을 잘못 이해할 수 있다.
- Suggested Fix: `:926`, `:934`를 report 28 시정과 독립 재감사 기준으로 갱신하고 active-state regression이 후반 stale “새 CI” 문구도 검출하게 한다.
- Re-audit Method: active status phrase scan, section 1/10/11 상호 비교와 documentation regression을 실행한다.
- Owner: Coder, Documentation

## 6. Pass 2 — Debug·Engineering Quality Findings

### [R28-DBG-F001] 제어키 Repeat의 state-crossing 정책이 정의되지 않음

- Pass: Debug
- Pattern: DBG-002, TEST-001
- Area: TUI input dispatcher, modal/state transition, state-crossing Repeat policy
- Severity: **Major**
- Severity Basis: 실제 terminal 도달성과 비의도 여부가 미확정이므로 confirmed Major 집계가 아니라 조건부 영향 등급이다.
- Status: **Needs Spec Clarification**
- Related: FIN-F008, FIN-F009, FIN-F016
- Summary: dispatcher는 Release만 전역 차단하고 Repeat는 G/A/J/R request key에만 별도 처리한다. constructed Press→Repeat sequence에서 Press가 modal/state를 닫은 뒤 Repeat가 새 state 규칙으로 다시 해석된다. 그러나 active spec/design은 Esc/Enter control-key Repeat를 하나의 gesture로 취급할지 반복 명령으로 취급할지 정의하지 않는다.
- Evidence:
  - `apps/aihack-tui/src/tui/mod.rs:1586-1588`: Release만 전역 `None`이다.
  - `apps/aihack-tui/src/tui/mod.rs:1589-1608`: overlay와 soft-input의 Esc가 close/cancel을 만든다.
  - `apps/aihack-tui/src/tui/mod.rs:1610-1613`: Repeat 예외는 G/A/J/R에만 한정된다.
  - `apps/aihack-tui/src/tui/mod.rs:1501-1513`: CharacterCreation Esc는 BackToTitle, Title/Playing Esc는 Quit이다.
  - `apps/aihack-tui/src/tui/mod.rs:781`: Quit handler는 `Ok(true)`로 event loop 종료를 요청한다.
  - `apps/aihack-tui/tests/tui_contract.rs:294-320`: Judge 문자 반복만 검사하며 Esc/Enter/Backspace 및 transition을 검사하지 않는다.
- Actual sequences:
  - Judge Esc Press→cancel, Esc Repeat→Playing Quit.
  - Inventory/StorageError Esc Press→close, Esc Repeat→Playing Quit.
  - MorePrompt Esc Press→acknowledge/Playing, Esc Repeat→Quit.
  - CharacterCreation Esc Press→Title, Esc Repeat→Quit.
  - Title Enter Press→CharacterCreation, Enter Repeat→Playing.
- Expected: 통제 문서가 Esc/Enter Repeat의 state-crossing 의미, destructive transition 허용 여부와 실제 OS 도달성 검증 수준을 먼저 정의해야 한다. 안전한 기본 후보는 첫 state-changing Press 뒤 같은 control-key Repeat를 새 state 명령으로 승격하지 않는 것이다.
- Actual: production dispatcher에 constructed `KeyEventKind::Repeat`를 주면 unsaved run 종료 또는 확인 화면 건너뛰기 후보가 결정적으로 생성된다. 실제 키 홀드/ConPTY가 Esc/Enter Repeat를 발생시키는지는 이번 감사에서 검증하지 않았다.
- Impact: 해당 Repeat가 실제 terminal에서 도달하고 현재 동작이 비의도라면 앱 종료와 미저장 진행 손실이 가능하다. core hash/revision만 검사하는 테스트는 handler의 `true` 반환을 놓친다.
- Suggested Fix:
  1. 먼저 `spec.md`/`designs.md`에 Esc/Enter 및 실제 state-changing candidate의 Repeat 정책을 동결한다.
  2. 비의도 동작으로 결정하면 해당 control-key Repeat를 전역 또는 state별 명시 allowlist로 차단한다. 의도 동작으로 결정하면 미저장 종료 위험과 사용자 피드백을 명시한다.
  3. 문자 편집 repeat와 game/request shortcut repeat를 의미 기반으로 분리한다.
  4. Press→Repeat 연속 event를 실제 `runtime_event_to_candidate`와 `handle_candidate_owned`에 통과시키고 candidate, state, overlay, handler return을 모두 assert한다.
  5. 최소 지원 크기 fallback의 Esc/Q Repeat 정책도 결정된 계약으로 정렬한다.
- Re-audit Method: 문서로 동결한 정책에 따라 위 다섯 transition sequence를 Press/Repeat/Release 조합으로 실행한다. 실제 키 홀드 도달성을 주장하려면 별도 PTY/ConPTY 증거도 확인한다.
- Owner: Coder

### [R28-DBG-F002] F9 hash 회귀가 실제 toggle 경로를 실행하지 않음

- Pass: Debug
- Pattern: TEST-001
- Area: UI regression evidence
- Severity: **Minor**
- Status: **Needs Fix**
- Related: FIN-F009
- Summary: F9가 presentation-only라는 테스트 이름과 달리 두 untouched session의 hash만 비교한다.
- Evidence:
  - `tests/ui_debug.rs:40-47`: 동일 seed의 새 session 두 개를 만들고 hash를 비교할 뿐 F9/candidate/handler가 없다.
  - `apps/aihack-tui/tests/tui_contract.rs:324-345`: debug mouse test도 `debug_observation_visible = true`를 직접 대입한다.
- Expected: 실제 F9 Press가 `ToggleDebug` candidate를 만들고 handler 뒤 UI flag만 바꾸며 core revision/hash는 유지됨을 검증해야 한다.
- Actual: toggle handler가 core를 변경하는 회귀가 생겨도 테스트가 계속 green일 수 있다.
- Impact: 현재 source 결함은 확인되지 않았지만 회귀 증거가 주장한 경계를 잠그지 못한다.
- Suggested Fix: 실제 F9 Press→candidate→handler를 실행하고 handler return false, flag 전환, 동일 revision/hash, 두 번째 toggle 복원을 assert한다. Repeat/Release 정책도 함께 고정한다.
- Re-audit Method: 기존 테스트의 handler 호출 여부를 source로 확인하고 의도적 core mutation을 넣었을 때 테스트가 RED가 되는 mutation sanity를 수행한다.
- Owner: Coder

## 7. Pass 3 — Security·Supply Chain Findings

### [R25-SEC-F001 — Re-audit #3] archive component가 Windows 이름 별칭을 canonicalize하지 않음

- Pass: Security
- Pattern: SEC-004, BUILD-001
- Area: release source archive, path canonicalization, cross-platform extraction
- Severity: **Major**
- Status: **Needs Fix**
- Related: FIN-F014
- Summary: slash/dot/parent 검사는 추가됐지만 first component 비교가 case-sensitive이고 각 component의 Windows trailing dot/space/reserved-name 규칙을 적용하지 않는다.
- Evidence:
  - `scripts/verify_release_bundle.ps1:141-155`: component가 `.`/`..`인지 확인한 뒤 blocked root를 `-ccontains`로 비교한다.
  - `scripts/verify_release_bundle.sh:50-66`: exact case pattern만 차단하며 Windows-compatible component normalization은 없다.
  - 완전한 Windows bundle에서 uppercase, trailing dot, trailing space의 세 alias 모두 checksum 재생성 후 verifier exit 0/PASS.
- Expected: Windows 대상 archive와 cross-platform source archive는 추출 후 같은 파일 시스템 객체가 될 이름을 verifier 단계에서 동일 canonical component로 취급해야 한다.
- Actual: default case-insensitive Windows filesystem에서 blocked root와 충돌하는 alias 아래 `probe.txt` entry가 release source archive에 포함되고 verifier가 PASS했다.
- Impact: 검증된 한 entry와 같은 방식으로 제외한 legacy tree 전체도 alias root 아래 들어갈 수 있다고 추론된다. 따라서 provenance/source-scope hard boundary가 이름 canonicalization으로 우회될 수 있다.
- Suggested Fix:
  1. 양 verifier가 모든 component에 Windows-safe canonical rule을 공유하도록 한다.
  2. blocked first component는 최소 ASCII case-insensitive 비교를 사용하고 trailing dot/space를 거부한다.
  3. empty/dot/parent/colon/backslash뿐 아니라 Windows reserved device basename과 extraction collision을 거부한다.
  4. 두 verifier의 동일 fixture matrix에 uppercase/mixed-case, trailing dot/space, reserved device와 정상 유사 이름을 추가한다.
- Re-audit Method: 실제 source archive에 각 alias를 하나씩 삽입하고 checksum을 재생성한 완전한 bundle을 양 verifier에 입력하여 모두 nonzero인지 확인한다.
- Owner: Coder, Security

### [FIN-F015 — Re-audit #4] Linux strict calendar가 year 0000을 승인함

- Pass: Security
- Pattern: BUILD-001, TEST-001
- Area: release provenance date, cross-platform verifier parity
- Severity: **Minor**
- Status: **Needs Fix**
- Related: FIN-F015
- Summary: Linux verifier는 GNU `date -d` round-trip을 strict Gregorian 판정으로 사용하지만 GNU date가 `0000-MM-DD`를 canonical로 받아들인다. Windows `.NET DateTime`과 허용 도메인이 다르다. 현재 production build는 candidate date를 Git commit에서 생성하므로 이번 fixture가 실제 build에서 year-0000 candidate로 도달한다는 증거는 없다.
- Evidence:
  - `scripts/verify_release_bundle.sh:40-48`: 네 자리 형식과 GNU date round-trip만 확인한다.
  - `scripts/verify_release_bundle.ps1:118-136`: `.NET DateTime.TryParseExact`를 사용하여 year 1..9999 도메인을 따른다.
  - 완전한 Linux bundle에서 candidate `0000-06-15`, period `0000-01-01..0000-12-31`이 verifier exit 0/PASS.
- Expected: 양 OS verifier가 동일한 명시 범위와 leap/calendar 규칙을 사용해야 한다. 현 Windows 및 project strict-calendar 의미에 맞는 공통 범위는 `0001..9999`다.
- Actual: 같은 metadata가 Linux에서는 승인되고 Windows에서는 거부된다.
- Impact: 직접 verifier의 양 OS accept/reject parity가 불완전하다. 현재 실제 candidate date와 production entrypoint는 정상이라 즉시 release bypass로 확대하지 않고 Minor로 판정한다.
- Suggested Fix:
  1. parser 호출 전 또는 후 year를 숫자로 분해하여 `0001..9999`를 명시 검증한다.
  2. candidate/start/end에 같은 helper를 사용한다.
  3. year 0000, 0001, 9999, invalid leap/day와 start-after-end를 양 OS 대칭 fixture로 고정한다.
- Re-audit Method: 같은 완전한 bundle metadata matrix를 Linux/Windows verifier에 입력하고 동일 fixture에 대한 accept/reject 판정이 같은지 확인한다.
- Owner: Coder, Security

## 8. Cross-Pass Conflicts

| Conflict | 해소 판단 |
| --- | --- |
| 437개 tests와 양 OS CI green vs 4개 confirmed Major·1개 Major-impact clarification·3개 Minor | 열거된 regression의 성공과 인접 동등 경계의 누락이므로 finding 유지 |
| `next_id=MAX` reject vs `MAX-1` 첫 commit/둘째 panic | sentinel 차단은 usable allocator headroom을 증명하지 않으므로 save closure 기각 |
| registry 자체는 valid vs bootstrap/save world가 invalid | producer가 consumer 불변식을 보장해야 하므로 content finding 유지 |
| armor Wear→Drop는 reversible vs Wear→Throw는 irreversible | removal lifecycle이 분산되어 있어 FIN-F007 전체 closure 기각 |
| dispatcher가 event마다 state-aware vs constructed Repeat가 새 state에서 다시 해석 | 단일 event 정합성만으로 gesture 정책을 결정할 수 없어 Needs Spec Clarification 유지 |
| archive dot/parent matrix green vs Windows case/trailing alias PASS | lexical canonicalization과 target-platform canonicalization이 다르므로 FIN-F014 유지 |
| invalid month/day/leap reject vs year 0000 Linux PASS | strict-calendar 양 OS parity가 불완전하므로 FIN-F015 유지 |
| active summary 앞부분은 완료 vs 후반은 새 CI pending | 현재 authority 내부 모순이므로 Minor documentation recovery 유지 |

## 9. Verified로 유지하는 개선

- save level registry identity/depth, checked stairs pairing과 wand charge optional shape
- `next_id=u32::MAX` 직접 fixture reject 자체
- custom armor `ac_bonus` consumer-safe range와 일반 Wear→Drop→save 복원
- 9종 causal의 동일 command/observer field-only A/B, exactly-one witness loss, 나머지 8개 record equality 및 반복 결정성
- F9 visible debug rect의 shared layout와 mouse consume
- Judge G/A/J/R Press·Repeat 문자 입력 `GGAAJJRR`, Release 무시, Playing request Repeat 차단
- Inventory/StorageError/soft-input/blocking state의 개별 mouse isolation과 shared `InspectPresentation`
- terminal RAII, Windows Console API/ANSI evidence 경계와 ConPTY
- source archive의 slash/dot/repeated-dot/parent/absolute/backslash 명시 matrix
- invalid month/day/non-leap/start-after-end calendar matrix
- fresh staging/promotion, output root/nested reparse, expected-name hardlink 거부
- repository-root local→local action recursion, cycle/missing/escape reject와 pinned chain accept
- future dependency approval 거부, current action SHA/Docker digest, dependency exception/duplicate gate
- replay self-verification, mutable state encapsulation과 FIN-F018 atomic rewrite
- implementation `ea7822a/32683076204` 및 current docs `01b2bd3/32684490662` 양 OS success
- current Windows 9-entry exact bundle과 active Markdown broken link 0

## 10. Rejected/Clarified 후보

- 9종 causal trace의 label 문자열만으로는 outcome 증거가 약하지만, 동일 command/observer source path와 나머지 full `CausalWitnessRecord` equality를 함께 확인했으므로 R25-IMP-F002/FIN-F006을 Verified로 판정한다.
- current docs run `32684490662`을 remediation 본문에 넣지 않은 사실은 별도 Major가 아니다. production implementation authority는 정확하고, docs-only 후속 commit에도 자체 green evidence가 있다.
- F9 toggle source는 현재 UI-only로 보이며 직접 runtime defect는 확인되지 않았다. 따라서 실제 경로를 실행하지 않는 테스트 문제만 Minor로 분리한다.
- 일반 calendar의 invalid month/day/leap 수정은 유효하다. finding은 year 0000과 양 OS domain parity에 한정한다.
- archive finding은 임의 Unicode 정규화 전체를 새 요구로 만들지 않는다. 현재 Windows 배포와 명시 excluded root를 우회하는 case/trailing/reserved-name equivalence에 한정한다.

## 11. PASS 전 필수 수정 순서

### P0 — Confirmed Major

1. 관련 `spec.md`/ADR/implementation plan에 accepted save·registry·release canonical domain을 먼저 명시한다.
2. allocator를 fallible하게 만들고 `MAX-1` 연속 spawn에서 no-panic/no-partial-commit을 보장한다.
3. live monster HP 범위와 bootstrap invariant validation을 완성한다.
4. item kind의 forbidden field shape를 완성하고 모든 inventory removal을 공통 unequip lifecycle로 통합한다.
5. 양 archive verifier에 Windows-compatible component canonicalization을 적용한다.

### P0-C — Gate clarification

6. Esc/Enter control-key Repeat의 state-crossing 의미와 실제 terminal reachability 요구를 `spec.md`/`designs.md`에 동결한다.
7. 비의도 동작이면 dispatcher와 sequence regression으로 차단하고, 의도 동작이면 미저장 종료 영향과 사용자 피드백을 명시한다.

### P1 — Minor

8. 양 calendar verifier에 명시적인 year `0001..9999` 범위를 적용한다.
9. F9 regression을 실제 candidate/handler 경로로 교체한다.
10. `IMPLEMENTATION_SUMMARY.md`의 stale next-step 두 곳과 documentation regression을 갱신한다.

## 12. Accepted Risks와 남은 제한

### 12.1 명시적 Accepted Risk

| Risk | Status | Owner | 수용 사유 | 영향 범위 | 만료·재검토 조건 |
| --- | --- | --- | --- | --- | --- |
| `hallucinating` SaveDataV1 compatibility orphan | **Accepted Risk** | Project owner / runtime maintainer | SaveDataV1 필드를 즉시 제거하면 기존 wire/save 호환성을 불필요하게 깨뜨림 | R9 causal completeness에 한정; 현재 gameplay producer/consumer가 없는 필드를 보존하며 save integrity나 security 예외로 확장하지 않음 | SaveDataV2·v0.4.0 scope 승인 또는 2026-10-31 중 먼저 도래할 때 제거 migration과 실제 producer feature 중 하나를 재결정 |

근거는 `spec.md:798`과 `DESIGN_DECISIONS.md:240`이다.

### 12.2 Excluded/known platform limits — Accepted Risk 아님

- runtime same-account concurrent directory-entry swap은 기존 single-writer 제품 모델 밖이며 §1.2 제외 범위다.
- Windows parent-directory metadata의 power-loss durability는 `spec.md:731`과 `DESIGN_DECISIONS.md:135`에 기록된 OS/filesystem 제한이다.
- 실제 model provider smoke, Windows Terminal GUI rendering, artifact signing/attestation/upload는 §1.2 제외 범위다.

### 12.3 정보성 evidence

- 현재 dependency graph는 registry 310, path 8, git 0이며 `saphyr 0.0.12`는 dev-only다.
- 위 accepted/excluded 범위는 allocator panic, invalid custom world, equipment lifecycle, archive alias, year 0000, stale active status 또는 control-key 정책 미정을 허용하지 않는다.

## 13. Needs Spec Clarification

### R28-DBG-F001 control-key Repeat

- 미정 질문: Esc/Enter `Repeat`를 첫 Press와 같은 물리 gesture의 연장으로 보는가, 새 state에서 반복 실행 가능한 독립 명령으로 보는가?
- 미정 질문: 정책 검증은 constructed crossterm event까지만 요구하는가, 실제 PTY/ConPTY key hold까지 요구하는가?
- Major 영향 후보: 실제 terminal에서 도달하고 비의도라면 modal close 뒤 Quit과 확인 화면 건너뛰기가 가능하다.
- 권장 기본안: 텍스트 편집 문자 Repeat는 허용하되 첫 state-changing Press 뒤 같은 control-key Repeat는 새 state의 destructive command로 승격하지 않는다.
- 종료 조건: `spec.md`와 `designs.md`에 정책을 동결하고 production dispatcher, handler return과 필요한 terminal-level fixture가 같은 결론을 증명한다.

다중 역할 armor를 새로 허용하려면 R27-IMP-F001의 forbidden-shape 수정 대신 별도 계약 변경과 완전한 removal lifecycle을 먼저 정의한다.

## 14. 재감사 체크리스트

1. `next_id=MAX`와 `MAX-1` fixture가 production loader 또는 fallible allocator에서 typed reject된다.
2. `MAX-1` fixture에 두 번의 spawn-producing command를 실행해 panic 없음, no-partial-commit, RNG/hash 보존과 다음 save 가능 여부를 검사한다.
3. custom monster hp 0/negative가 registry 또는 bootstrap 전에 거부된다.
4. accepted custom registry의 initial/Wait/save/load가 전체 world invariant를 유지한다.
5. armor damage/hit_bonus forbidden shape가 거부되거나, 명시 허용 시 Wear→Throw가 base AC와 equipment/location을 원자적으로 복원한다.
6. 모든 inventory removal consumer(drop/throw/consume/destroy)가 같은 unequip lifecycle과 save round-trip matrix를 통과한다.
7. 결정된 control-key 계약에 따라 Judge/Inventory/StorageError/MorePrompt/CharacterCreation의 Esc Press→Repeat→Release candidate, handler return과 state를 직접 assert한다.
8. 결정된 계약에 따라 Title Enter Press→Repeat의 CharacterCreation/Playing 전환을 assert하고, 실제 terminal 도달성을 주장하면 PTY/ConPTY key-hold evidence를 추가한다.
9. F9 Press→candidate→handler가 flag만 바꾸고 revision/hash를 보존하며 Release/Repeat 정책이 명시된다.
10. uppercase/mixed-case, trailing dot/space, reserved device archive component가 양 verifier에서 nonzero이고 정상 유사 이름은 통과한다.
11. year 0000은 양 OS에서 reject, 0001/9999 valid fixture는 accept하며 invalid leap/day와 기간 순서 결과가 동일하다.
12. causal 9종 full record equality와 기존 save/content/TUI/release/action regressions가 계속 green이다.
13. implementation summary의 완료/남은 gate 문구가 report 28 remediation lifecycle과 일치한다.
14. 아래 전체 gate를 단독 실행한다.

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

15. 새 clean implementation SHA의 Ubuntu/Windows same-SHA actual bundle을 확인한다.
16. 새 독립 감사가 report 28의 8개 finding과 FIN-F001~F018을 연결해 재판정한다.

## 15. 최종 근거와 Coder Handoff

### 최종 근거

- Report 27의 시정 항목을 production source, tests, local full gate, 양 OS CI와 독립 adversarial fixture로 다시 대조했다.
- causal, F9 mouse, Judge 문자 repeat, local action recursion 등 핵심 수정은 실제로 Verified됐다.
- 그러나 accepted input만으로 panic/invalid world/derived-state 파손이 가능하고 archive hard boundary가 platform alias를 허용한다. control-key Repeat는 Major 영향 후보인데 제품 정책과 terminal 도달성 기준이 미정이다.
- 따라서 `docs/audit/audit_report_27_remediation.md`의 same-SHA 검증 기록은 유효하되 독립 PASS 조건은 충족하지 못했다. PROGRAM/PUBLICATION HOLD를 유지한다.

### Coder Handoff

```text
`C:\LocalDev\rust\AIHack\docs\audit\audit_report_28.md`의 독립 재감사 결과를 확인하고,
각 finding을 current spec/ADR, 실제 production entrypoint와 adversarial fixture에 대조하여 수정하세요.
계약 변경은 관련 문서를 먼저 갱신한 뒤 allocator/custom registry/equipment removal,
TUI control-key Repeat 계약을 먼저 동결하고, archive Windows component alias와 year 0000 calendar parity를 닫으세요.
F9 실제 경로 회귀와 stale implementation summary도 함께 복구하고,
수정 후 전체 로컬 gate와 새 clean same-SHA Ubuntu/Windows actual bundle을 실행하여 결과를 기록하세요.
```
