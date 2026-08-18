# 감사 리포트 23 코더 시정 기록

- 작성일: 2026-08-18 (Asia/Seoul)
- 기준 finding: `docs/audit/audit_report_23.md`
- 기준 HEAD: `41a1b63f11a57a671b0f705883431dab24298b5a`
- 문서 성격: 코더 구현·검증 기록. 독립 재감사 또는 release 승인 아님

## 1. 범위와 권한

보고서 23의 모든 finding을 문서·코드·테스트에 다시 대조했다. 우선순위는 SEC-F001, TEST-F001, DBG-F009, IMP-F016/017 순서로 적용했고, 이후 SEC-F002, IMP-F018, DBG-F010을 처리했다. 기존 report 19~21의 R8 기술·문서 종결 evidence와 report 23의 새 HOLD를 구분했으며, 외부 게시·Git commit·push는 수행하지 않았다.

## 2. Finding별 시정 결과

| Finding | 코더 상태 | 핵심 시정 | 재현·검증 근거 |
| --- | --- | --- | --- |
| SEC-F001 | Remediated / re-audit pending | `ArtifactStore` capability root, no-follow open, regular-file/single-hard-link handle 검증, `cap-tempfile` atomic replace, 실행별 TUI quick-save directory | `tests/headless_paths.rs`의 preplaced temp/destination/replay hard-link victim 불변과 기존 save 교체 |
| TEST-F001 | Remediated / re-audit pending | `CausalProjection`, 9종 typed witness, 필수 집합 validator와 deterministic `causal-v1` fixture | seed 42/7/1234 각 witness count 1, 3회 multiset/hash 일치, event-only·turn-only·누락 negative PASS |
| DBG-F009 | Remediated / re-audit pending | manifest/content LF checkout, script CR 정규화, Windows CI canonical R7/R8 실행 | 실제 Windows Git Bash R7/R8 exit 0, CRLF positive와 checksum drift negative PASS |
| IMP-F016 | Remediated / re-audit pending | report 21의 report 20 종결, R9 SHA/run, report 23 새 HOLD를 활성 문서에 분리 | `tests/r8_documentation.rs` current authority와 stale predecessor negative PASS |
| IMP-F017 | Remediated / re-audit pending | report 22 finding을 Initial/Current로 구분하고 status·검증 파일·handoff를 정렬 | report 22 current status, risk owner/trigger와 문서 회귀 PASS |
| SEC-F002 | Remediated / re-audit pending | `lru` 0.18.1을 0.18.2로 갱신 | `cargo tree -i lru`, `cargo audit` warning/vulnerability 0 |
| IMP-F018 | Remediated / re-audit pending | 두 CLI help의 과거 v0.1.0/Phase 설명 제거 | headless/TUI `--help` contract test PASS |
| DBG-F010 | Remediated / re-audit pending | trailing whitespace 제거, 테스트 주석 한국어화, line-ending-only 상태 정리 | `git diff --check` PASS, 불필요한 content/manifest diff 0 |

## 3. R9 witness baseline

필수 집합은 `FoodNutrition`, `CorpseNutrition`, `ArmorDefense`, `MonsterSpeed`, `MonsterAi`, `MonsterPassive`, `MonsterDifficultyEconomy`, `PrayerLuckCombat`, `GoldScore`다. 각 seed에서 모든 count는 1이며 세 번 반복해 같은 multiset과 hash를 얻었다.

| Seed | accepted turn 기준 | Causal final hash |
| --- | --- | --- |
| 42 | absolute turn 1000 이상 | `5cde4a5f145ff3af` |
| 7 | absolute turn 1000 이상 | `942403c665e19ad9` |
| 1234 | absolute turn 1000 이상 | `01a8631d0ad95d96` |

기존 `survival-v1`은 사용자 정책으로 유지하고 테스트 전용 fixture만 causal action sequence를 사용한다. `hallucinating` compatibility risk owner는 Project owner/runtime maintainer이며 SaveDataV2·v0.4.0 범위 승인 또는 2026-10-31 중 먼저 도래하는 시점에 재검토한다.

## 4. 전체 검증 결과

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --locked` | PASS |
| `cargo metadata --locked --no-deps --format-version 1` | PASS |
| `cargo audit` | PASS, vulnerability/warning 0 |
| 프로젝트 로컬 cargo-deny 0.19.4 `check licenses bans sources` | PASS — `bans ok, licenses ok, sources ok` |
| Windows Git Bash `scripts/r7_checkpoint.sh` | PASS, exit 0 |
| Windows Git Bash `scripts/r8_checkpoint.sh` | PASS, exit 0 |
| `git diff --check` | PASS |

## 5. 남은 gate와 위험

- 현재 변경은 기준 HEAD 뒤의 working-tree diff이므로 same-SHA Linux/Windows CI evidence가 아직 없다.
- report 24의 DBG-F011 시정에서 `winx 0.36.4` 전용 SPDX exception을 추가하고 프로젝트 로컬 cargo-deny 0.19.4 실제 PASS를 확인했다. clean same-SHA CI의 고정 cargo-deny 단계는 commit/push 뒤 재검증한다.
- Windows의 Unix 전용 `release_bundle` integration test 0건 문제는 CI의 `build.bat --release`와 새 canonical R7/R8 실행으로 범위를 보강했지만, 이 코더 기록이 독립 재감사를 대신하지 않는다.
- 외부 게시는 독립 재감사 PASS와 별도 사용자 승인 전까지 HOLD다.
