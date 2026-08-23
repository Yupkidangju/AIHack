# Final Multi-Audit Report 1 1차 시정 기록(Report 25에서 부분 기각)

> 현재 권위: `docs/audit/audit_report_25.md`의 HOLD. 이 문서의 최초 `로컬 Verified`/`LOCAL REMEDIATION PASS` 표시는 broad gate와 당시 fixture 범위의 역사적 결과이며, report 25가 production probe로 재현한 9 Major/4 Minor를 닫지 못했다. 현재 시정·RED/GREEN·same-SHA 증거는 `docs/audit/audit_report_25_remediation.md`에 기록한다.

- 기준 감사: `docs/multi_audit/1/final_audit_report_1.md`
- 감사 기준 HEAD: `80d959af94cb08c5d9b2f2601f5e63f3827a1210`
- 시정 일자: 2026-08-23
- 실행 환경: Windows 11 Pro `10.0.26200.0`
- Rust: `rustc 1.94.1`, `cargo 1.94.1`
- 보안 도구: `cargo-deny 0.19.4`, `cargo-audit 0.22.1`
- 상태 의미: 아래 `로컬 Verified`는 1차 시정 당시 fixture 범위의 역사적 주장으로만 보존하며 현재 상태가 아니다.

## 1. 동결한 계약

1. ReplayLineV1은 command-only log가 아니라 consumed prefix 전체를 검증하는 self-verifying artifact다.
2. SaveDataV1 wire/schema version 1은 유지하되 16 MiB, event/entity 각 100,000개, RNG 1,000,000 draw, persisted text 512 UTF-8 byte를 상한으로 둔다.
3. replay는 64 MiB, 100,000 lines, line당 65,536 byte를 상한으로 둔다.
4. headless 기본 policy는 `survival-v1`, target turn은 `1..=1,000,000`이다.
5. TUI built-in runtime locale은 English다. README 다국어 문서는 runtime 5-locale catalog 완료 주장이 아니다.
6. runtime root는 single-writer 사용자 전용 directory다. 사전 배치 link/reparse와 외부 inode write는 차단하되 같은 계정의 악성 동시 directory-entry 교체를 OS sandbox처럼 보장하지 않는다.
7. v0.3.0 candidate는 2026-08-23까지의 R9, 보안, final audit 시정을 포함한다.

## 2. Finding별 대조 및 시정 결과

| Finding | 로컬 상태 | 문서·코드·테스트 증거 |
| --- | --- | --- |
| FIN-F001 | 로컬 Verified | `GameError::InvalidSave`, full saved-world validator, `tests/save_validation.rs` malformed relation matrix와 valid schema mismatch |
| FIN-F002 | 로컬 Verified | save/replay/RNG/event/entity/text 상한, bounded reader, exact/+1 회귀와 control 거부 |
| FIN-F003 | 로컬 Verified | `ReplayMismatchField` 7종, cloned working session, field별 tamper와 exhaustion no-partial-commit |
| FIN-F004 | 로컬 Verified | root final component no-follow, Windows junction/Unix symlink·root swap 회귀, TUI app-owned `ArtifactStore`, ambient save helper 제거 |
| FIN-F005 | 로컬 Verified | GameSession/GameWorld/runtime EntityStore `DerefMut` 제거, 실제 외부 Cargo consumer compile-fail |
| FIN-F006 | 로컬 Verified | `CausalWitnessRecord`의 scenario/producer/field/source/consumer attribution, speed/AI 독립 pair, exact gold-only score |
| FIN-F007 | 로컬 Verified | immutable registry context와 injected restore, custom corpse nutrition 500, armor wear/drop/rewear/save/load 원자성 |
| FIN-F008 | 로컬 Verified | Inventory/error overlay, Title load, Creation/Awaiting cancel, item letter, MorePrompt, load/new-run transient reset |
| FIN-F009 | 로컬 Verified | 60/65/70% layout tier, 실제 theme style, Tab focus, reduced-motion effect, mouse capture와 ConPTY mouse click |
| FIN-F010 | 로컬 Verified | best-effort terminal RAII와 restore 단계별 failure injection, actual Windows ConPTY one-event/state/mouse/restore test |
| FIN-F011 | 로컬 Verified | Clap implicit default와 `0/1/1,000,000/1,000,001` parser matrix |
| FIN-F012 | 로컬 Verified | report 23/24 historical closure와 final multi-audit HOLD current authority 분리, compound gap status 제거, section-aware 문서 회귀 |
| FIN-F013 | 로컬 Verified | `DEP-EXC-0001`, 오늘 날짜/90일/deny exact version/resolved graph checker와 expired/version/unrelated negative fixture, PROV-0005 동기화 |
| FIN-F014 | 로컬 Verified | `verify_release_bundle.ps1`, Windows legacy/metadata/record/hash/zero-size/duplicate checksum negative matrix |
| FIN-F015 | 로컬 Verified / 원격 대기 | GitHub Actions full SHA pin, v0.3.0 2026-08-23 changelog·modification ID/기간·metadata 일치. 새 clean same-SHA CI는 commit 부재로 미실행 |
| FIN-F016 | 로컬 Verified | timeout 500/2000/1500 단일 상수, rationale 1..=160, English fallback, injected UI clock cooldown test |
| FIN-F017 | 로컬 Verified | stale phase/version 주석 제거, Unix parent fsync/Windows durability 경계, package TUI/ConPTY semantic smoke, exact 23-family duplicate budget |
| FIN-F018 | 로컬 Verified within threat model | replay batch bounded read + single atomic rewrite, initial creation 후 추가 hard-link 불변 회귀, concurrent external inode append 경로 제거 |

## 3. 로컬 실행 증거

| 명령 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, Windows ConPTY와 platform bundle negative matrix 포함 |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| Git for Windows Bash `scripts/r7_checkpoint.sh` | PASS |
| Git for Windows Bash `scripts/r8_checkpoint.sh` | PASS |
| `cargo audit` (`0.22.1`) | PASS, vulnerability 0 |
| `cargo deny check licenses bans sources` (`0.19.4`) | PASS: bans/licenses/sources ok |
| `cargo test --locked -p aihack-tui --test conpty_contract` | PASS, actual Windows ConPTY |
| `cargo test --locked -p aihack --test release_bundle_windows` | PASS, positive 1 + negative matrix 1 |
| `cargo test --locked -p aihack --test provenance_manifest` | PASS, 14 tests |

## 4. 잔여 위험과 권한 경계

- 새 working tree는 commit이 아니므로 clean same-SHA Ubuntu/Windows CI와 platform bundle 결과가 없다. 기존 run `32110917881`은 audited base HEAD의 historical evidence일 뿐 현재 시정을 증명하지 않는다.
- Git commit/push와 외부 게시 권한은 이번 사용자 요청에 포함되지 않았으므로 수행하지 않는다.
- Windows는 file sync + atomic replace를 보장 범위로 두며 parent-directory metadata의 전원 손실 durability는 OS/filesystem 정책에 따른 잔여 위험이다.
- actual ConPTY contract는 자동화했지만 Windows Terminal GUI application 자체의 rendering은 별도 수동 범위다.
- `hallucinating` SaveDataV1 compatibility risk와 remote real-provider smoke의 기존 owner/비차단 조건은 연장하거나 승격하지 않는다.

## 5. 로컬 판정

최종 correctness/readability/architecture/security/performance review에서 replay line별 atomic rewrite가 production trace 출력에서 O(n²)이 되는 문제를 발견해 `append_replay_lines` batch 경로로 수정했다. 그 밖의 Critical/Important review finding은 남지 않았고 batch 회귀, Clippy와 전체 test를 다시 통과했다.

**HISTORICAL PARTIAL EVIDENCE / PROGRAM HOLD**

FIN-F001..F018의 1차 문서·코드·표적 회귀와 전체 workspace/all-target test는 당시 녹색이었으나 report 25의 inverse relation, writer budget, path alias, paired score, production TUI/terminal, release actual-set 및 문서 lifecycle probe에서 충분하지 않은 것으로 확인됐다. 전체 program 또는 외부 release PASS에는 report 25 시정, 새 commit의 clean same-SHA 양 OS CI, 독립 재감사와 별도 게시 승인이 모두 필요하다.
