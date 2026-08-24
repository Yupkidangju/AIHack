# AIHack 감사 보고서 29 시정 기록

- 기준 독립 감사: `docs/audit/audit_report_29.md`
- 현재 독립 권위: `docs/audit/audit_report_31.md`
- 기준 HEAD: `d9c0f8eb673e641f2a23b17f99a9327e90899628`
- 작업 브랜치: `codex/audit-report-29-remediation`
- 작업 일자: 2026-08-24
- 기술 판정: **REPORT 29 TECHNICAL VERIFIED (`a91a9c7/32706869079`) / REPORT 30 `ed02dbf/32733235414` clean same-SHA 양 OS actual·independent Verified**
- 현재 판정: **REPORT 31 implementation-summary lifecycle remediation pending / PROGRAM·PUBLICATION HOLD**

독립 감사 원문은 수정하지 않는다. 이 문서는 계약 선행 변경, 수정 전 adversarial RED, production 구현, 표적 GREEN, 전체 local gate와 clean same-SHA Ubuntu/Windows actual bundle을 시간순으로 기록한다. 구현이나 CI가 끝나기 전에는 해당 단계를 PASS로 표시하지 않는다.

## 1. Finding 대조와 결정

| Finding | current spec/ADR 대조 | production entrypoint | 보존할 adversarial fixture | 결정 |
| --- | --- | --- | --- | --- |
| R29-DOC-F001 | `spec.md` 9.1/9.7/15절, ADR-0039 | `GameSession::submit`, TUI loop, release verifier | ordinary reject 대 aborted reject의 state/save/hash | `transaction_aborted`/invariant failure만 전체 rollback하고 ordinary invariant-valid reject 전이는 보존 |
| R29-DOC-F002 | README·ADR·roadmap·summary·build/gap active section | `tests/r8_documentation.rs` | 각 문서 active section에 predecessor current phrase 삽입 | report 29 current exact-one, predecessor는 historical/superseded section에만 허용 |
| R29-IMP-F001 | `spec.md` 9.3, ADR-0039 | `ContentRegistry::from_toml_sources`→runtime/TUI/ActionSpace | `item.weapon.dagger`를 armor로 선언 | 알려진 ID의 canonical kind/class pair 강제, class-changing override 거부 |
| R29-IMP-F002 | `spec.md` 9.3 | registry typed conversion | empty, `AB`, 결합문자 sequence, 단일 Unicode scalar | item glyph 정확히 한 Unicode scalar |
| R29-DBG-F001 | `spec.md` 9.7, ADR-0039 | actual event loop→stateful dispatcher→handler | Load/Inventory/MorePrompt/direction·inventory selection/GameOver/LLM의 Repeat·Press/Press·Release sequence와 ConPTY repeated bytes | transition candidate 뒤 same-key 입력을 matching Release 또는 500ms quiet+50ms poll 2회 연속 idle까지 억제; repeat-safe allowlist만 허용 |
| R29-TEST-F001 | `spec.md` 11.5 | save loader→`GameSession::submit` | MAX-2→MAX-1 commit→MAX exhaustion, Throw/Zap item·charge·RNG·save/hash | production-valid exact-successor integration matrix로 승격 |
| R29-DBG-F002 | `spec.md` 9.1, ADR-0039 | external `GameSession::submit`; internal projectile/monster systems | external consumer compile boundary와 session error equality | mutating projectile/monster system은 crate 내부 transaction primitive, public fallible mutation은 atomic submit만 제공 |
| R29-SEC-F001 | `spec.md` 15절, ADR-0039 | PowerShell/Bash verifier→공통 archive validator | 금지문자/control, superscript device, console name, Unicode/sanitizer collision, prefix, symlink/hardlink/device | format-aware raw name/type/link/prefix 검증 후 safe temp extraction exact manifest |
| R29-SEC-F002 | `spec.md` 15절, ADR-0039 | verifier의 `ExpectedCommit` 경계 | docs-only, crate omission, Rust blob substitution, safe extra, mode/type mutation | 동일 commit·format의 독립 `git archive`와 byte-identical complete identity |

## 2. 문서 선행 변경

2026-08-24에 production code와 test를 수정하기 전에 다음 계약을 갱신했다.

- `spec.md`: ordinary reject/aborted rollback 범위, public mutation ownership, canonical item ID-kind/glyph, Release/queue-idle transition gesture, raw archive/extraction와 complete tree identity, year `0001..9999`
- `DESIGN_DECISIONS.md`: ADR-0039 추가, ADR-0037/0038 active authority supersede와 historical evidence 분리
- `designs.md`: 실제 TUI gesture 흐름과 양 OS archive 검증 구조
- README·implementation summary·gap·audit roadmap·build guide·documentation audit: report 29 단일 current lifecycle과 pending 범위
- `CHANGELOG.md`: 제품 계약 결정과 시정 범위

## 3. 수정 전 RED

기준 SHA `d9c0f8e` 구현에 문서·fixture만 추가한 상태에서 다음을 확인했다.

| fixture/명령 | 수정 전 결과 | 고정한 실패 경계 |
| --- | --- | --- |
| `content_validation::known_item_id_rejects_a_shape_valid_declared_class_override` | **RED**, dagger armor registry가 `Ok` | ID-kind canonical pair 부재 |
| `content_validation::item_glyph_requires_exactly_one_unicode_scalar` | **RED**, empty glyph가 registry에서 `Ok` | runtime conversion 전 exact scalar 검증 부재 |
| `tui_contract::equivalent_transition_keys_do_not_cross_state_on_repeat_or_adjacent_press` | **RED**, Load 뒤 `l Repeat`가 `Move(East)` | key blocklist가 transition gesture를 보존하지 않음 |
| `public_mutation_boundary` | **RED**, projectile/monster module이 `pub mod` | public submit와 low-level mutation ownership 불일치 |
| `release_archive_security::format_aware_validator...` | **RED**, normal positive control에서 공통 validator 파일 부재 | negative-only harness 자체의 missing-verifier false-green도 positive control로 차단 |
| `release_archive_security::expected_commit_requires...` | **RED**, actual `git archive` positive control에서 verifier 부재 | complete-source verifier 미구현 |
| 기존 `r8_documentation::active_r8_status...` | **RED**, README에서 과거 report 28을 요구 | current-authority test 자체가 stale 문자열에 결합 |

allocator finding은 구현 결함이 아니라 영구 fixture 누락이므로 수정 전에도 다음 새 production-valid test가 **GREEN**이었다.

```text
transaction::production_valid_allocator_last_commit_and_exhaustion_are_atomic ... ok
transaction::production_valid_throw_and_zap_exhaustion_restore_item_charge_rng_and_hash ... ok
```

fixture는 `next_id=MAX-1` exact-successor load→마지막 jackal corpse commit→`next_id=MAX`→다음 corpse reject와 full save/hash equality를 검사한다. 별도 exhausted load는 Throw/Zap에서 item location 또는 charge, RNG, world, save와 outcome hash가 모두 원본과 같은지 검사한다.

## 4. 구현과 표적 GREEN

### 4.1 Content·public mutation

- `ContentRegistry`가 known ID 10개의 canonical declared kind table을 검사한다. shape-valid dagger→armor mutation은 typed `ContentError::Parse`다.
- item glyph는 empty, `AB`, `e + combining acute`를 거부하고 단일 scalar `🗡`을 수용한다. runtime conversion도 두 번째 scalar를 방어적으로 다시 거부한다.
- `systems::projectiles`와 `systems::monster_ai`를 `pub(crate)`로 축소하고 root facade re-export를 제거했다. 기존 deterministic monster plan test는 crate unit boundary로, public item contract는 read-only query로 이동했다.

### 4.2 TUI transition gesture

stateful gate는 soft-input 문자/Backspace, 안정된 Playing Move/Wait와 focus만 repeat-safe로 허용한다. transition candidate 뒤에는 논리 key alias를 정규화하고 새 state의 다른 transition candidate도 quarantine한다. actual ConPTY probe에서 `\r\r`가 `Press→Release→Press→Release`로 투영되는 사실 때문에 Release만으로는 해제하지 않고 500ms quiet와 50ms poll 2회 연속 empty를 함께 요구한다.

```text
tui_contract: 20 PASS
conpty_contract: 2 PASS
windows_conpty_repeated_enter_bytes_do_not_cross_two_state_transitions ... ok
```

double-byte write는 CharacterCreation에서 멈추고 drain 뒤 별도 Enter는 Playing으로 진행했다. 기존 one-key Title→Creation→Playing, mouse, Inventory/Esc, Q와 terminal restore도 PASS했다.

### 4.3 Archive·ExpectedCommit

`scripts/verify_source_archive.py`를 양 verifier가 공유한다. ZIP/TAR를 format-aware하게 읽어 Windows 금지문자, C0/C1/DEL, superscript COM/LPT, `CONIN$`/`CONOUT$`, NFKC+casefold collision, file prefix와 symlink/hardlink/device를 거부한다. 검증한 regular file만 임시 root에 create-new 추출해 path/content hash manifest를 대조한다. full mode는 40-hex `ExpectedCommit`을 resolve하고 같은 format의 `git archive`를 독립 재생성해 byte hash가 같아야 한다.

```text
release_archive_security: 2 PASS
release_bundle_windows: 7 PASS
build_contract: 12 PASS
license_compliance: 6 PASS
release_gate: 7 PASS
```

docs-only, Rust omission/blob mutation, safe extra와 모든 raw/type fixture는 nonzero였고 current HEAD의 fresh ZIP/TAR는 positive control로 통과했다.

### 4.4 Authority·allocator

- `r8_documentation` 10 PASS. README 5개 locale current section, ADR-0039, summary, roadmap 두 active block, build/gap final block과 documentation audit 10.16은 report 29 marker exact-one 및 predecessor current phrase mutation을 검사한다.
- production-valid allocator integration 2 PASS. 마지막 corpse commit 후 exhaustion과 Throw/Zap rollback이 full save/RNG/item/charge/hash equality를 유지한다.
- `aihack-runtime --all-targets`와 `aihack-tui --all-targets` 표적 package suite가 PASS했다.

## 5. 전체 로컬 quality gate

clean-worktree actual bundle을 제외한 전체 gate를 단독 실행했다.

| 명령 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked -- --list` | Windows named test **455개** |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| `cargo metadata --locked --format-version 1` | packages/resolved nodes **318/318** |
| `cargo audit` | PASS, 318 dependencies scan, vulnerabilities 0 |
| `cargo deny --version` | `cargo-deny 0.19.4` |
| `cargo deny check licenses bans sources` | PASS, bans/licenses/sources ok |
| Git Bash `scripts/r7_checkpoint.sh` | PASS |
| Git Bash `scripts/r8_checkpoint.sh` | PASS |

자체 5축 review와 staged secret/path scan 뒤 implementation commit `1fa6d903ea09170014154c0c64e0fdaf673fcb6c`를 만들고 clean worktree에서 `cmd /d /c build.bat --release`를 실행했다.

```text
PASS source archive: format=zip entries=385
PASS Windows release bundle: version=0.3.0 commit=1fa6d903ea09170014154c0c64e0fdaf673fcb6c
빌드 완료: output\aihack.exe, output\aihack-headless.exe
```

output은 binary 2, `LICENSE`, `NOTICE`, `MODIFICATIONS.md`, approval record, metadata, source ZIP, `SHA256SUMS`의 9개 exact entry다. checksum record는 8개이며 metadata commit/candidate date는 `1fa6d90...`/`2026-08-24`다. 공통 validator가 source ZIP 385개 raw entry와 safe extraction을 검사하고 fresh `git archive 1fa6d90...`와 byte equality를 확인했다. 이 시점 local lifecycle은 `Verified`, 양 OS CI는 pending이다.

## 6. clean same-SHA Ubuntu/Windows actual bundle

첫 docs successor SHA `1a68f76a30ec62204895168b68f2b122860f0f52`의 Actions `32706287953`에서 Ubuntu job `97367988636`은 test step에서 FAIL 101이었다. production verifier 결함은 아니며 Unix-only `tests/release_bundle.rs`의 세 fixture가 complete identity 도입 뒤에도 commit 후 archive를 재압축한 원인이었다.

```text
verifier_accepts_a_normal_similar_archive_name ... FAILED
verifier_accepts_minimum_and_maximum_supported_calendar_years ... FAILED
  source archive is not byte-identical to git archive ExpectedCommit
verifier_rejects_a_source_archive_containing_the_blocked_legacy_tree ... assertion text mismatch
```

유사 이름과 year 0001/9999 positive는 temp Git commit 전에 source/metadata/period를 구성하고 실제 `git archive`를 사용하도록 고쳤다. blocked legacy negative는 공통 validator의 더 구체적인 `excluded or absolute path` 문구와 맞췄다. 첫 run의 Windows job은 Ubuntu 확정 실패 뒤 successor queue를 열기 위해 취소했다.

최종 기술 evidence successor는 `a91a9c70523288bf2d5289bb35c9d1f1e5565a33`이며 [Actions `32706869079`](https://github.com/Yupkidangju/AIHack/actions/runs/32706869079)가 completed/success다.

| Job | ID | 결과 | 시간 | step |
| --- | ---: | --- | ---: | --- |
| `ubuntu-latest quality gate` | `97369721441` | PASS | 9m17s | 19 success, Windows bundle 1 skip |
| `windows-latest quality gate` | `97369721295` | PASS | 24m29s | 19 success, Linux bundle 1 skip |

양 job은 checkout SHA `a91a9c7...`에서 metadata/fmt/clippy/all-target tests, dependency exception/duplicate, R7/R8, release all-target build, actual platform bundle, cargo-audit, cargo-deny 0.19.4와 lockfile 불변을 모두 통과했다. 실제 Linux TAR와 Windows ZIP은 각 verifier의 raw/type/extraction 및 `ExpectedCommit` byte identity를 통과했다.

이 결과로 report 29 시정 lifecycle은 `Verified`다. 다만 `Closed`, 전체 PROGRAM PASS와 외부 게시 허가는 새 독립 재감사 및 별도 사용자 승인 전까지 HOLD다.

## 7. 잔여 경계

- report 29 후속 독립 재감사 전에는 finding을 `Closed`로 올리지 않는다.
- actual physical key-hold, 실제 model provider, signing/attestation과 외부 게시 승인은 이번 시정의 기술 증거에 포함하지 않는다.
- `hallucinating` SaveDataV1 compatibility orphan accepted risk는 기존 owner와 2026-10-31 재검토 조건을 유지한다.
