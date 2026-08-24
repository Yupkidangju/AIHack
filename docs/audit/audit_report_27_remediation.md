# audit report 27 시정 기록

문서 상태: 시정 진행 중
작성일: 2026-08-24
감사 기준: `docs/audit/audit_report_27.md`
현재 권한: 독립 재감사 HOLD

## 1. 시정 계약

`spec.md`와 `DESIGN_DECISIONS.md` ADR-0037을 먼저 갱신했다. wire schema v1과 사용자 동작은 유지하되 다음 production 경계를 fail-closed로 강화한다.

1. save allocator headroom, active registry level 집합, checked stairs depth, charge optional shape
2. custom registry item consumer 범위와 base-derived reversible armor lifecycle
3. 동일 command/observer를 유지한 9종 field-only causal A/B 및 나머지 8개 record equality
4. archive component canonical path와 양 OS strict Gregorian calendar
5. F9 visible rect mouse consume, Judge text Repeat, local composite action recursion

## 2. 수정 전 RED fixture

아래 fixture는 report 27의 독립 production probe를 회귀 이름으로 보존한다. 구현 전 개별 실행 결과는 이 문서의 검증 표에 기록한다.

| 경계 | RED fixture | 수정 전 기대 실패 |
| --- | --- | --- |
| save | allocator `u32::MAX`, level `i16::MAX`+StairsDown, wand `charges=None` | load가 성공하거나 다음 consumer가 panic |
| registry | armor `ac_bonus=i16::MIN` 및 Wear→Drop→save | registry 수용, AC 비가역 |
| causal | 9종 field-only neutralization matrix | omission branch가 command/observer를 생략 |
| release path | `./`, repeated dot, parent, absolute, backslash, excluded canonical first component | verifier false-green |
| calendar | invalid month/day/leap, start-after-end | Linux verifier false-green |
| TUI | debug visible/hidden same coordinate, Judge Press/Repeat/Release | click-through와 문자 repeat 손실 |
| CI action | root local→local→mutable remote, cycle, missing, escape | transitive mutable ref 미검출 |

## 3. 구현 및 검증 증거

### 3.1 수정 전 RED와 표적 GREEN

| 경계 | RED 증거 | GREEN 증거 |
| --- | --- | --- |
| save allocator/level/charge | `semantic_validator_rejects_allocator_level_and_charge_consumer_traps` exit 1 | typed `InvalidSave`, test PASS; headless 7-case exit 2 PASS |
| custom registry | `causal_numeric_content_rejects_invalid_ranges`가 `ac_bonus=i16::MIN`을 수용해 exit 1 | unsafe registry reject와 `accepted_custom_armor_registry_keeps_wear_drop_and_save_round_trip_reversible` PASS |
| field-only causal | report 27에서 command/observer omission 재현 | `causal_field_only_ab_loses_exactly_one_witness_and_preserves_other_records` PASS; 9개 동일 trace, exactly-one loss, 나머지 record equality |
| archive/calendar | Windows dot alias verifier false-green; Linux invalid period은 독립 감사에서 false-green | 양 OS alias/calendar matrix와 verifier 구현 GREEN, Linux matrix는 새 Ubuntu CI에서 최종 확인 예정 |
| debug/Judge | visible debug click이 `Inspect` candidate, Judge 결과 `GAJR`로 exit 1 | visible rect candidate None/revision 불변, Judge `GGAAJJRR`, 일반 request Repeat None PASS |
| local action | local ref terminal trust assertion exit 1 | root local→local→mutable, cycle, missing, escape reject와 pinned chain accept PASS |

### 3.2 현재 로컬 검증

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked -- --list` | PASS, named test 437개 |
| 표적 save/content/causal/headless/TUI/Windows release/document tests | PASS |
| `cargo test --workspace --all-targets --locked` | PASS, 437 named tests |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| dependency exception/duplicate gate | PASS |
| `scripts/r7_checkpoint.sh && scripts/r8_checkpoint.sh` | R7 PASS / R8 PASS |
| `cargo audit` | PASS, 318 dependencies, vulnerabilities 0건 |
| cargo-deny 0.19.4 `licenses bans sources` | PASS |
| `git diff --exit-code -- Cargo.lock` | PASS |
| clean Windows release bundle과 same-SHA 양 OS CI | pending |

구현 SHA와 clean same-SHA Ubuntu/Windows Actions run은 전체 GREEN 후 successor evidence로 갱신한다. report 26 계보는 `fc01ec12/32658658526` partial, `a9a39d8/32660221745` Linux failure, `1e84a94/32660514315` final predecessor PASS로 보존한다.

## 4. 종료 조건

- 모든 RED fixture가 production entrypoint에서 GREEN
- `cargo fmt`, clippy, workspace all-target tests/build, cargo-audit, cargo-deny 0.19.4, R7/R8와 release verifier PASS
- clean 동일 SHA의 Ubuntu/Windows actual bundle PASS
- 결과와 남은 HOLD를 active 문서에 동기화

시정 성공은 report 27 자체를 수정하거나 독립 재감사를 대체하지 않으며 외부 게시를 승인하지 않는다.
