# R7 Provenance and Compatibility Implementation Report

작성일: 2026-07-18
범위: R7-1, R7-2 engineering implementation
판정: **PASS WITH KNOWN RISKS — LICENSE REVIEW DEFERRED TO R8**

> 후속 상태 (2026-07-20): 프로젝트 소유자가 AIHack을 NetHack 3.6.7 source 기반 AI-assisted semantic rewrite 파생물로 분류하고 whole-work NGPL을 승인했다. 이 보고서의 당시 HOLD 증거는 보존하며, 현재 승인 상태는 `PROVENANCE.md`, ADR-0030과 R8 checkpoint 결과를 따른다.

## 결과

- 공식 NetHack 3.6.7 source archive SHA-256을 공식 배포 페이지와 실제 임시 다운로드에서 대조했다.
- 공식 archive의 `dat/license` checksum과 손상된 legacy `LICENSE.NGPL` checksum을 별도 보존했다.
- runtime source에서 `legacy_nethack_port_reference` 직접 import/path dependency가 없음을 자동 검증한다.
- NH367-C001..C010 record 10개가 source file/symbol locator, precondition, command, expected event/state/hash field, Rust test를 연결한다.
- `tests/nethack_367_compat.rs`의 10개 scenario와 기존 `golden_phase8_rules` 20개를 R7 engineering gate로 사용한다.
- C008 source 대조에서 기존 hunger threshold가 3.6.7과 다름을 발견했다. 공식 `newuhs` 경계를 RED test로 고정한 뒤 core projection과 기존 회귀 테스트를 수정했다.
- `audit_report_12.md`의 SEC-F002를 따라 status-only approval 우회를 RED로 재현하고, runtime coverage·승인 필드·content checksum·scenario schema/function을 검증하는 checkpoint로 강화했다.
- `audit_report_13.md`의 IMP-F013/SEC-F003에 따라 R7 provenance와 R8 distribution 책임을 분리하고 checkpoint root를 script-relative repository로 고정했다.
- DBG-F005를 따라 C003 hit/damage/HP/death/RNG와 C007 turn/item/charge/map/RNG assertion을 연결 test에 직접 추가했다.

## 검증 결과

| gate | 결과 |
| --- | --- |
| R7 provenance + NH367 + P8 표적 | PASS, 42 tests |
| full workspace | PASS, 322 tests |
| fmt/check/clippy `-D warnings` | PASS |
| release build | PASS |
| `cargo audit --no-fetch` | PASS, 1160 advisories / 267 dependencies |
| cargo-deny licenses/bans/sources | PASS |
| blocked legacy runtime reference | 0건 |
| `scripts/r7_checkpoint.sh` | 예상대로 HOLD, runtime Approved 1건·scenario Approved 10건 대기 |
| approval negative fixture | PASS, status-only/7개 필드/checksum/ID/schema/function/coverage/Blocked reference 우회 차단 |
| phase/root negative fixture | PASS, `UNLICENSED` root에서 완전 승인 R7 fixture 통과; inherited root override 무효 |

## 라이선스 경계

프로젝트 구현은 `LicenseRef-AIHack-UNLICENSED` 아래 내부 build/test만 승인한다. `crates/aihack-content/src/data/**`는 observable NetHack behavior와의 관계 및 배포 라이선스 범위가 소유자 또는 적격 검토자에게 승인되지 않아 `Reviewed`다. scenario도 동일하게 `Reviewed`이며 release compatibility count에는 아직 포함하지 않는다.

R7 engineering 구현과 SC-COMPAT-01 evidence는 통과했다. 2026-07-18 사용자 결정에 따라 다음 항목은 R8 실제 런칭 전 최종 검토로 이관하며 아직 선언하지 않는다.

- SC-LICENSE-01 PASS
- 외부 배포 가능
- 법률 적합성 보장

## 승인 후 전환 절차

1. 프로젝트 소유자 또는 적격 검토자가 content data와 scenario의 저작권/파생물 범위를 검토한다.
2. 배포 라이선스, notice, source 제공 및 modification notice 의무를 기록한다.
3. `PROVENANCE.md`의 PROV-0004와 NH367 record 10개의 승인 필드를 근거와 함께 채운 뒤 `Approved`로 전환한다. 상태 문자열만 바꾸면 checkpoint가 FAIL한다.
4. `Cargo.toml`의 `UNLICENSED` 변경은 승인된 배포 라이선스와 R8 release 작업에서만 수행한다.
5. R8 최종 런칭 gate와 독립 감사를 다시 실행한다.

R7 `PASS WITH KNOWN RISKS`는 compatibility/provenance engineering evidence와 개발 진행 허용만 뜻한다. asset/scenario actual approval와 외부 배포는 R8에서 root license, version, packaging과 최종 release audit가 모두 통과할 때까지 계속 차단한다. 현재 `scripts/r7_checkpoint.sh`의 HOLD는 R8에 남은 license evidence를 가시화하는 기대 결과다.

## 재현 명령

```bash
cargo test -p aihack --locked --test provenance_manifest
cargo test -p aihack --locked --test nethack_367_compat
cargo test -p aihack --locked --test golden_phase8_rules
scripts/r7_checkpoint.sh # ADR-0030 승인 반영 후 예상 결과: PASS, exit 0
! rg -n "legacy_nethack_port_reference" Cargo.toml crates apps src \
  --glob '*.toml' --glob '*.rs'
cargo deny check licenses bans sources
```
