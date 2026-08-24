# AIHack 감사 보고서 26 시정 기록

- 기준 보고서: `docs/audit/audit_report_26.md`
- 시정 시작일: 2026-08-24
- 기준 HEAD: `65d8fe5a7fa418794d050e318931b12133bbf616`
- 현재 상태: **IMPLEMENTATION SAME-SHA VERIFIED / INDEPENDENT RE-AUDIT PENDING / PROGRAM HOLD**

## 1. 문서 우선 계약

1. SaveDataV1은 unequipped base AC, consumer-safe scalar/aggregate와 복원 registry의 exact `ItemData`를 fail-closed로 검사한다.
2. Windows artifact component는 trailing dot/space, ADS·금지 문자와 reserved device name을 거부한다.
3. causal 누락 검증은 완성 summary를 편집하지 않고 actual producer/content/pair를 실행 전에 제거한다.
4. modal·overlay·soft input은 event 종류보다 먼저 mouse authority를 차단하고, Inspect renderer/hit-test는 단일 presentation을 공유한다.
5. release는 workspace 내부 fresh staging root에서 생성·검증 후 directory 단위로 승격하며 root reparse/symlink와 expected hard link를 거부한다.
6. exact candidate commit date를 metadata에 넣고 modification period와 자동 교차 검증한다.
7. LLM request key는 Press만 허용하고 dependency approval은 미래일 수 없으며 action pin은 전체 GitHub YAML node를 구조적으로 검사한다.

상세 결정과 기각 대안은 `DESIGN_DECISIONS.md`의 ADR-0036을 따른다. report 25의 구현·CI는 부분 positive evidence로 유지한다. report 26 시정은 RED→GREEN, 전체 로컬 gate와 새 clean same-SHA Ubuntu/Windows actual bundle까지 Verified됐지만 새 독립 감사와 별도 게시 승인 전에는 program 또는 외부 게시를 PASS로 올리지 않는다.

## 2. 수정 전 RED fixture

아래 fixture는 production 수정 전에 실행 결과를 기록하고 이후 같은 이름의 회귀 테스트로 보존한다.

| 경계 | fixture | 수정 전 결과 |
| --- | --- | --- |
| malformed save | unequipped AC, max turn, max kill count, forged ItemData의 load→command/quit | RED: direct validator가 4종 모두 수용, production headless가 unequipped AC에서 exit 0/turn 1 |
| Windows alias | trailing dot/space/repeated dot/reserved replay output의 headless binary 실행 | RED: `run.jsonl.`이 exit 0으로 원 replay를 교체 |
| causal producer | 9종 producer/content/pair actual removal full run | RED: 기존 test는 summary label 삭제였고 새 full run은 difficulty economy의 `before.gold + difficulty` debug overflow까지 재현 |
| modal mouse | Inventory/StorageError/soft input 및 Inspect hover/decision click | RED: Inventory command-panel click이 `Wait`를 제출해 turn 0→1, hover 첫 행은 숨은 item command 반환 |
| release authority | output root symlink/junction, expected-name hard link와 outside victim | RED: Windows verifier가 root junction과 expected-name hard link를 모두 exit 0으로 승인 |
| candidate date | candidate date가 modification end date보다 뒤인 actual archive | RED: metadata에 candidate date가 없고 2026-08-24 commit을 2026-08-23 종료 manifest가 승인 |
| P1 | LLM Repeat, future dependency approval, YAML action variant | RED: `G` Repeat가 `LlmNarrative`, 2026-09-01 approval이 2026-08-24 gate를 통과; line-prefix action scan은 valid YAML 변형을 누락 |

## 3. 구현·검증 증거

### 3.1 finding별 local 판정

| Finding | local 판정 | production·회귀 근거 |
| --- | --- | --- |
| R25-IMP-F001 re-audit | Verified | unequipped base AC, turn/score 조합과 registry `ItemData` equality를 load 전에 검사하고 좁은 산술을 widening/saturating 처리. direct 및 headless malformed 4종 exit 2 |
| R25-DBG-F002 re-audit | Verified | Windows component trailing dot/space, ADS·금지 문자, reserved device와 superscript COM/LPT를 preflight 거부. production replay alias matrix에서 input 불변 |
| R25-IMP-F002 re-audit | Verified | `CausalSummary::without` 제거, speed/AI/difficulty production pair와 9종 producer/content/pair 실행 전 누락 matrix. 각 run은 정확히 8 record/한 witness 누락이며 반복 summary/hash 일치 |
| R25-IMP-F003 re-audit | Verified | overlay/modal/soft input mouse authority를 event-kind 분기 전에 차단. 실제 candidate 처리 뒤에도 revision 불변 |
| R25-IMP-F004 re-audit | Verified | renderer와 hit-test가 `InspectPresentation`을 필수 인자로 공유. hover/decision에서 hidden item command 0건 |
| R25-IMP-F005 re-audit | Verified | spec의 GoldScore pair 완료와 actual producer-removal 시정을 분리하고 report 26 authority 회귀 추가 |
| R25-DBG-F003 re-audit | Verified | designs를 Windows Console API mouse/raw call matrix와 ANSI alternate/cursor transcript 범위로 정정 |
| R26-IMP-F001 | Verified | `G/A/J/R` Repeat/Release는 request candidate를 만들지 않고 Press만 허용 |
| R25-SEC-F001 re-audit | Verified | fresh random staging→verifier→directory promotion, root/nested reparse와 expected hard-link 거부. outside victim 불변 |
| FIN-F015 re-audit #2 | Verified | notice `AIHACK-MODIFICATIONS-2026-08-24-01`, `candidate_date=$Format:%cs$`, period 자동 교차 검증 |
| R25-SEC-F002 re-audit | Verified | `approved_on <= today`와 future approval negative |
| R25-SEC-F003 re-audit | Verified | dev-only `saphyr 0.0.12`로 `.github/**/*.yml|yaml` 전체 node 순회. local action 외 원격 action은 40-hex, Docker는 full SHA-256 digest만 허용 |

### 3.2 수정 후 표적 GREEN

- `save_validation::semantic_validator_rejects_consumer_unsafe_scalars_unequipped_ac_and_forged_item_data`: PASS
- `headless_contract::production_headless_rejects_consumer_unsafe_malformed_saves_before_running`: PASS
- `headless_contract::replay_windows_name_aliases_are_rejected_without_mutating_input`: PASS
- `long_run::causal_actual_producer_removal_loses_exactly_one_required_witness`: PASS
- `tui_contract::modal_and_overlay_mouse_clicks_never_submit_underlying_core_commands`: PASS
- `tui_contract::inspect_hover_and_decision_presentations_do_not_expose_hidden_inventory_commands`: PASS
- `release_bundle_windows` root/nested junction, expected hard link, fresh promotion, stale candidate date matrix: PASS
- dependency future approval 및 YAML inline/spaced/nested/composite/Docker mutable ref: PASS

정합 validator 도입 뒤 기존 death/monster AI fixture가 player AC를 임의 변경해 실패한 것은 제품 회귀가 아니라 새 계약과 충돌한 테스트 전제였다. 두 fixture는 player AC를 보존하고 attacker hit bonus로 deterministic 사망을 만들도록 교정했다. persisted attack profile의 `name`은 wire에서 `serde(skip)`이므로 registry equality는 저장되는 hit bonus/damage를 비교하고 비저장 label은 비교하지 않는다.

### 3.3 전체 로컬 gate

| 명령 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked -- --list` | PASS, 당시 named test 432개 |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| `cargo audit` | PASS, 318 dependencies, vulnerability 0 |
| `cargo deny check licenses bans sources` | PASS (`bans ok, licenses ok, sources ok`) |
| Git Bash `scripts/r7_checkpoint.sh` | PASS |
| Git Bash `scripts/r8_checkpoint.sh` | PASS |

### 3.4 5축 재검토

- correctness: 정상 save/replay hash, 3-seed causal hash와 v1 wire는 유지됐고 malformed 입력만 typed reject한다.
- readability: presentation enum과 release staging helper가 renderer/input 및 batch lifecycle 책임을 명시한다.
- architecture: causal difficulty는 사후 event label이 아니라 active/control production pair로 분리했고 public mouse mapper가 presentation 인자를 필수로 받는다.
- security: review 중 기존 output의 nested junction cleanup과 mutable Docker tag라는 추가 false-green을 발견해 각각 fail-closed fixture로 보강했다.
- performance: registry equality는 기존 100,000 entity budget 안의 bounded lookup이고 production binary에는 YAML parser가 포함되지 않는다. score/combat widening은 고정 크기 산술이다.

### 3.5 clean implementation SHA와 양 OS CI

- 구현 SHA: `fc01ec12bac522e601bc56bced06b0908f5873b0`
- clean local Windows production entrypoint: `cmd /c build.bat --release` PASS
- local bundle: 9-entry exact set, metadata commit `fc01ec12bac522e601bc56bced06b0908f5873b0`, `candidate_date=2026-08-24`, notice `AIHACK-MODIFICATIONS-2026-08-24-01`
- Actions: [run `32658658526`](https://github.com/Yupkidangju/AIHack/actions/runs/32658658526)
- Ubuntu: 19개 step success — tests, dependency gates, R7/R8, actual Linux bundle, cargo-audit, cargo-deny 0.19.4, lockfile 불변
- Windows: 19개 step success — tests/ConPTY, dependency gates, R7/R8, actual Windows bundle, cargo-audit, cargo-deny 0.19.4, lockfile 불변

로컬 셸에서 별도 outside-victim hard-link를 production output에 직접 사전 배치하는 명령은 실행 정책이 파일 삭제가 포함된 계산 경로 작업을 실행 전에 거부했다. 같은 경계는 `windows_release_staging_promotes_a_fresh_directory_without_writing_a_preplaced_hard_link`와 양 OS verifier hard-link negative로 자동 보존했고 모두 PASS했다. 이 구현 evidence는 독립 감사 PASS나 외부 게시 승인이 아니며 program/publication HOLD를 유지한다.

### 3.6 최종 evidence SHA CI에서 발견한 Linux verifier false-green

evidence commit `a9a39d87235109c0fb1d1ea7a31ea3751fd37a30`의 Actions `32660221745`에서 Ubuntu `Run tests`가 `verifier_rejects_a_source_archive_containing_the_blocked_legacy_tree` 실패를 재현했다. source archive에 legacy path가 실제 포함됐지만 `set -o pipefail` 아래의 `tar -tzf | grep -q`에서 grep 조기 종료가 tar의 SIGPIPE를 만들었고, pipeline 전체 nonzero가 차단 경로 없음처럼 해석됐다. 구현 run `32658658526`의 양 OS PASS는 유효한 부분 증거지만 이 재현 뒤의 최종 closure로 사용하지 않는다. archive listing을 먼저 완전히 캡처해 tar 성공과 blocked-path predicate를 분리하고 같은 negative fixture를 GREEN으로 만든 뒤 새 clean same-SHA 양 OS run을 요구한다.

### 3.7 최종 verifier successor와 clean same-SHA closure

`1e84a94aa0623b5cee5349b5832992a4682e93a8`은 archive listing을 먼저 완전히 캡처한 뒤 tar 성공과 차단 경로 predicate를 분리했다. 수정 전 legacy fixture와 `linux_release_verifier_does_not_hide_a_blocked_path_behind_pipefail_sigpipe`가 GREEN이고, [Actions `32660514315`](https://github.com/Yupkidangju/AIHack/actions/runs/32660514315)에서 Ubuntu/Windows 각 19개 step, actual platform bundle, cargo-audit, cargo-deny 0.19.4와 lockfile 불변이 모두 success다. 따라서 `fc01ec12/32658658526`은 부분 선행 evidence, `a9a39d8/32660221745`는 Linux failure evidence, `1e84a94/32660514315`만 report 26 최종 same-SHA closure로 분류한다. 이후 `docs/audit/audit_report_27.md`가 canonical archive alias 등 새 범위를 재개방했으므로 report 26은 역사적 predecessor다.
