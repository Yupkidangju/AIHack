# 감사 보고서 25 시정 및 재검증 기록

- 기준 감사: `docs/audit/audit_report_25.md`
- 연결 감사: `docs/multi_audit/1/final_audit_report_1.md`
- 기준 HEAD: `80d959af94cb08c5d9b2f2601f5e63f3827a1210`
- 작업 브랜치: `codex/audit-report-24-remediation`
- 시정 시작일: 2026-08-23
- 현재 판정: **LOCAL QUALITY GATES GREEN / CLEAN BUNDLE·SAME-SHA CI PENDING / PROGRAM HOLD**

이 문서는 report 25 finding의 현재 시정 권위다. final multi-audit report 1의 첫 시정 기록은 당시 broad gate의 역사적 증거로 보존하지만, report 25가 production probe로 재현한 경계를 닫았다는 근거로 사용하지 않는다.

## 1. 문서 우선 동결 계약

1. SaveDataV1 wire/schema version 1과 16 MiB 상한은 유지한다. reader와 writer는 같은 semantic/byte budget을 사용하며 writer 성공은 immediate reload 가능성을 보장한다.
2. actor는 `max_hp > 0`, `hp <= max_hp`, `alive == (hp > 0)`을 만족한다. non-GameOver player는 alive, dead player는 GameOver다. 살아 있는 GameOver는 기존 quit sentinel(`Combat/EntityId(0)`)만 허용한다.
3. v1 inventory owner는 player 하나다. item location과 inventory index/letter를 양방향으로 검사하고 armor derived AC는 넓은 정수형 checked arithmetic 뒤 i16으로 변환한다.
4. headless `--turns 1..=1,000,000`은 실행 범위이며 save 성공 보장이 아니다. save budget 초과는 exit 2, typed resource error, 기존 destination 불변으로 닫고 history를 자동 폐기하지 않는다.
5. artifact path는 `.`을 제거한 normalized relative form을 사용한다. replay input/output은 lexical form, Windows case와 기존 열린 file identity 중 하나라도 같으면 실행 전에 거부한다. public ambient path helper는 제거한다.
6. GoldScore witness는 동일 world/turn clone에서 gold만 0으로 바꾼 control과 active 양쪽 모두 production `death_score`를 실행한 pair로만 기록한다.
7. TUI raw event는 production과 test가 공유하는 단일 state-aware dispatcher를 통한다. blocking state는 LLM dismiss/F9/focus보다 우선하며 ignored old response ID는 current outstanding 여부와 무관하게 먼저 폐기한다.
8. 60x24/80x24의 blocking prompt는 별도 modal에 실제 안내 3행을 모두 표시한다. command/inspect click rectangle은 renderer와 동일 CTA label model에서 파생한다.
9. terminal setup 단계별 상태 기록과 loop/terminal-new/draw/read 오류 뒤 best-effort cleanup을 seam으로 검증한다. 실제 Windows ConPTY는 mouse input 수용과 ANSI alternate/cursor enter/leave pair를 확인한다. Windows crossterm의 mouse/raw 전환은 ANSI가 아니라 Console API이므로 transcript로 과대주장하지 않고 lifecycle state/call matrix로 분리 검증한다.
10. release `output/` 전체가 게시 bundle이다. actual top-level entry는 선언된 exact set이어야 하며 extra file/directory/link/reparse를 거부한다.
11. dependency exception gate는 parsed TOML AST, exact trigger key 집합, valid calendar date와 dependency path를 검사한다. duplicate budget은 owner/reason/shipped scope/review metadata를 필수로 둔다.
12. 모든 workflow `uses:`는 40-hex ref여야 하며 action 주석/tag/SHA provenance가 일치해야 한다.
13. report 23/24는 historical closed, final report 1과 첫 remediation은 partial historical evidence, report 25는 current HOLD다. child/aggregate gap은 같은 lifecycle을 사용한다.

## 2. Finding 추적표

| 우선 | Finding | Production 책임 | 수정 전 RED fixture | 현재 상태 |
| ---: | --- | --- | --- | --- |
| 1 | R25-IMP-F001 | `crates/aihack-runtime/src/save.rs` semantic validator | inverse owner/index, HP/alive/max, armor min/max typed reject | LOCAL GREEN(debug/release) |
| 2 | R25-DBG-F001 | `ArtifactStore::save_session` | writer +1/self-unloadable/no-clobber | LOCAL GREEN(debug/release) |
| 3 | R25-DBG-F002 | `ArtifactStore` + headless entrypoint | `path`/`./path`, Windows case/file identity input hash 불변 | LOCAL GREEN |
| 4 | R25-IMP-F002 | runtime score + causal projection | paired production gold oracle, event-only/turn-only/all-witness removal | LOCAL GREEN |
| 5 | R25-IMP-F003 | TUI production dispatcher/response lifecycle | GameOver+N+LLM, MorePrompt Tab/BackTab, old/new response 교차 | LOCAL GREEN |
| 6 | R25-IMP-F004 | TUI render/input geometry | 60x24/80x24 prompt buffer, CTA exact boundary negatives | LOCAL GREEN |
| 7 | R25-SEC-F001 | 양 OS release verifier | extra file/directory/link/reparse | WINDOWS LOCAL GREEN / LINUX CI PENDING |
| 8 | R25-IMP-F005 | active 문서와 lifecycle gate | current authority, child/aggregate, Markdown link scan | LOCAL GREEN |
| 9 | R25-DBG-F003 | terminal lifecycle + ConPTY | setup/new/draw/read failure cleanup와 escape pair | LOCAL GREEN |
| 10 | R25-DBG-F004 | LLM worker/service synchronization | busy polling/elapsed assertion 제거와 bounded signal | LOCAL GREEN |
| 11 | R25-DBG-F005 | runtime package tests/maintenance ledger | semantic state delta와 metadata negative | LOCAL GREEN / LINUX CI PENDING |
| 12 | R25-SEC-F002 | dependency exception checker | TOML decoy/swap/trigger/date/path drift | LOCAL GREEN |
| 13 | R25-SEC-F003 | workflow provenance gate | 모든 `uses:` full SHA 일반 검사 | LOCAL GREEN / REMOTE CI PENDING |

## 3. 검증 순서

각 finding은 다음 순서를 지킨다.

1. 현재 production 경로에서 최소 실패 fixture를 추가한다.
2. 수정 전 표적 명령이 기대 이유로 실패하는지 기록한다.
3. 최소 production 수정 후 같은 명령이 통과하는지 확인한다.
4. 인접 package test/build를 통과한 뒤 다음 finding으로 이동한다.
5. 전체 fmt, Clippy, workspace all-target test, release build, R7/R8, cargo-audit 0.22.1, cargo-deny 0.19.4를 실행한다.
6. 변경 diff를 correctness/readability/architecture/security/performance 축으로 재감사한다.
7. clean commit을 push하고 동일 SHA의 Ubuntu/Windows quality/release bundle 결과를 기록한다.

## 4. 수정 전 RED 증거

2026-08-23에 아래 fixture를 production 수정 전에 실행했다. 최초 save test의 잘못된 import와 prompt test의 잘못된 test filter는 fixture 오류로 분류해 RED 증거에서 제외하고 바로잡은 뒤 다시 실행했다.

| Finding | RED 명령/fixture | 관찰 |
| --- | --- | --- |
| R25-IMP-F001 | `cargo test --locked -p aihack --test save_validation semantic_validator_rejects_inverse_inventory_actor_and_armor_arithmetic_boundaries -- --exact --nocapture` | exit 1, orphan `Inventory { owner: 999 }`가 `InvalidSave`가 아니어서 assertion 실패 |
| R25-DBG-F001 | `cargo test --locked -p aihack --test save_validation save_writer_rejects_a_self_unloadable_payload_without_clobbering_destination -- --exact --nocapture` | exit 1, 40,000 x 512-byte message save가 writer error가 아니었음 |
| R25-DBG-F002 | headless package의 `replay_input_and_curdir_output_alias...`, Windows `...case_variant...` | 둘 다 expected exit 2 대신 exit 0; 동일 input을 output으로 append |
| R25-IMP-F002 | `cargo test --locked -p aihack --test long_run gold_score_witness_uses_a_paired_production_score -- --exact --nocapture` | exit 1, causal source가 production paired score를 호출하지 않음 |
| R25-IMP-F003 | TUI `runtime_keys_are_state_aware...` | exit 1, `MorePrompt + Tab`이 `AcknowledgeMore`가 아니라 `FocusNext` |
| R25-IMP-F003 | `reset_ignored_response_is_discarded_before_matching_a_new_outstanding_request` | exit 1, old ignored response가 새 Decision pending을 `Invalid`로 변경 |
| R25-IMP-F004 | `command_and_inspect_clicks_follow_the_rendered_label_boundaries` | exit 1, 표시된 Wait 시작 열이 Inventory command로 매핑 |
| R25-IMP-F004 | TUI lib `minimum_supported_sizes_render_complete_blocking_prompt_content` | exit 1, 60x24 buffer에 `--More--`가 없음 |
| R25-SEC-F001 | Windows bundle negative matrix에 `ExtraFile`, `ExtraDirectory` 추가 | exit 1, `UNTRACKED-UNSIGNED-PAYLOAD.exe`를 verifier가 PASS |
| R25-IMP-F005 | `active_r8_status_docs_share_the_same_audited_ci_and_hold_boundary` | exit 1, `DOCUMENTATION_AUDIT_REPORT.md`에 report 25 current authority 누락 |
| R25-DBG-F003 | `terminal_lifecycle_routes_setup_and_loop_failures_through_one_restore_boundary` | exit 1, 단일 setup/loop restore seam 부재 |
| R25-SEC-F002 | `comment_decoy_missing_trigger_and_invalid_calendar_date_fail_closed` | exit 1, comment decoy가 deny exception 구조 검사를 통과 |
| R25-SEC-F003 | `ci_and_dependency_policy_run_the_same_locked_gates` | exit 1, checkout SHA 주석이 실제 `v4.4.0`이 아니라 `v4.2.2` |
| R25-DBG-F005 | duplicate budget live exact test를 schema 2 metadata 계약으로 실행 | exit 1, owner/scope/review metadata와 schema 2 부재 |

README 이동 링크는 계약 문서 갱신 단계에서 먼저 고쳤기 때문에 새 scanner는 첫 실행부터 PASS했다. 수정 전 broken link 증거는 report 25의 독립 probe(`README.md:63 -> audit_report_21.md`)에 보존한다.

## 5. GREEN 및 전체 gate 증거

| 명령 | 결과 |
| --- | --- |
| `git diff --cached --check` | PASS, whitespace error 0; 봉인된 sub-audit 원문은 SHA 보존용 path-scoped whitespace attribute 적용 |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS; report 25 회귀, package semantic test, Windows ConPTY/bundle matrix 포함 |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| release mode malformed save/writer 두 표적 | PASS; typed reject, panic 0, destination 불변 |
| Git for Windows Bash `scripts/r7_checkpoint.sh` | PASS |
| Git for Windows Bash `scripts/r8_checkpoint.sh` | PASS |
| `cargo audit` 0.22.1 | PASS, exit 0, 312 dependency scan |
| `cargo deny check licenses bans sources` 0.19.4 | PASS: bans/licenses/sources ok |
| `cargo test --locked -p aihack-tui --all-targets` | PASS; setup/loop failure matrix와 actual ConPTY 포함 |
| `cargo test --locked -p aihack --test release_bundle_windows` | PASS; extra file/directory/junction 포함 negative matrix |
| 전체 Markdown inline/reference relative link scan | PASS, broken 0 |

clean worktree가 필요한 actual `build.bat --release`와 Linux `build.sh --release`는 시정 commit 생성 뒤 같은 SHA에서 실행한다. 현재 dirty tree에서의 의도된 preflight failure를 release PASS로 세지 않는다.

## 6. Same-SHA CI 증거

아직 clean 시정 commit이 없으므로 미실행이다.

## 7. 3-pass 재감사와 5축 코드 리뷰

- Pass 1 구현·문서: report 23/24 historical closure, final report 1 partial evidence, report 25 current HOLD가 active 문서와 lifecycle test에서 일치한다. save/headless/TUI/release 계약은 production entrypoint와 동일 심볼을 가리킨다.
- Pass 2 debug·engineering: 수정 전 RED가 모두 GREEN으로 전환됐고 전체 workspace/all-target 및 release build가 통과했다. GoldScore control world clone이 모든 projection에서 실행되는 성능 문제를 리뷰 중 발견해 GameOver projection 한 번으로 축소했다.
- Pass 3 security·supply chain: capability-relative path identity, save capped writer, 양 verifier actual exact set, parsed TOML exception과 general action pin gate를 대조했다. staged 137개 파일에서 secret-like filename/pattern 0건이며 cargo-audit/deny가 PASS했다.
- correctness: inverse relation, state priority, no-clobber, late response와 exact-set error path를 named regression이 직접 검증한다.
- readability/architecture: `ArtifactStore`, production score pair, state-aware dispatcher, shared CTA model, terminal lifecycle, response signal queue로 책임을 모았고 public ambient resolver와 복제 score 식을 제거했다.
- performance: save serialization은 16 MiB capped buffer, replay는 batch atomic rewrite를 유지하며 GoldScore pair는 Quit/GameOver에서만 world clone 1회를 수행한다.
- 공식 provenance: checkout SHA `11d5960a326750d5838078e36cf38b85af677262`는 공식 `actions/checkout` v4.4.0 release commit이며 workflow 주석과 일반 full-SHA gate를 동기화했다.

로컬 리뷰에서 남은 Critical/Major는 없다. clean platform bundle과 same-SHA remote evidence가 없으므로 program HOLD는 유지한다.

## 8. 잔여 위험

- 같은 계정의 악성 concurrent directory-entry swap은 기존 single-writer threat model 밖이며 report 25가 유지한 accepted risk를 확장하지 않는다.
- Windows parent-directory power-loss durability와 실제 Windows Terminal GUI pixel/font rendering은 기존 platform 잔여 위험이다.
- 외부 게시·tag·release는 이 시정과 CI 요청 범위에 포함되지 않는다.
