# audit report 28 시정 기록

문서 상태: 시정 진행 중
작성일: 2026-08-24
감사 기준: `docs/audit/audit_report_28.md`
현재 권한: 독립 재감사 HOLD

## 1. 문서 우선 계약

`spec.md`, `designs.md`와 `DESIGN_DECISIONS.md` ADR-0038을 코드보다 먼저 갱신했다. 다음 경계를 production source of truth로 고정한다.

1. exact-successor persisted allocator와 fallible spawn/no-partial-commit
2. live monster HP 및 item kind complete field shape, accepted registry bootstrap saveability
3. 공통 inventory removal/unequip lifecycle
4. text editing Repeat와 transition/control-key Press-only 분리, 실제 F9 candidate/handler
5. Windows-compatible archive component와 year `0001..9999` 양 OS calendar parity

## 2. 수정 전 RED fixture

| 경계 | adversarial fixture | 수정 전 결과 |
| --- | --- | --- |
| allocator | `next_id=u32::MAX-1`, 연속 corpse spawn 2회 | 첫 spawn commit 후 둘째 debug overflow panic |
| registry | Jackal `hp=0`/negative custom TOML | registry/session 수용, initial/Wait save round-trip 실패 |
| equipment | armor `damage=1d4`, Pickup→Wear→Throw | Throw accepted, equipped None/AC -1, save 실패 |
| control Repeat | Judge/Inventory/StorageError/MorePrompt/CharacterCreation Esc 및 Title Enter Press→Repeat→Release | Repeat가 새 state Quit/confirm 후보 |
| F9 | 기존 hash test | F9 candidate/handler를 실행하지 않음 |
| archive | uppercase/mixed-case, trailing dot/space, reserved device, case collision | Windows alias bundle verifier false-green |
| calendar | candidate/period year 0000 | Linux PASS, Windows reject |
| docs | implementation summary 10·11절 | 완료된 report 27 CI를 다음 단계로 기록 |

## 3. 구현·검증 증거

### 3.1 수정 전 RED와 표적 GREEN

| 경계 | RED 증거 | GREEN 증거 |
| --- | --- | --- |
| allocator save | `semantic_validator_requires_the_exact_allocator_successor_without_headroom_gaps` exit 1 | `next_id == max_id.checked_add(1)` typed reject PASS, headless 8-case exit 2 PASS |
| allocator core/transaction | `next_id=u32::MAX` direct spawn panic, corpse command panic | checked `EntityAllocationError::Exhausted`, no store mutation과 rejected transaction full save/RNG/hash 불변 PASS |
| custom registry | hp 0/negative 및 damage/hit armor registry 수용 | live HP와 complete item shape typed `ContentError`, accepted hp 5 registry initial/Wait/save/load PASS |
| equipment removal | forged equipped throwable armor의 AC가 Throw 뒤 -1 | Throw가 공통 `remove_inventory_item`/unequip 경계를 사용하여 equipped None/base AC/location 복원 PASS |
| control Repeat | Judge Esc Repeat가 `Quit`, F9 Repeat가 `ToggleDebug` | Esc/Enter/F9/Q Press-only sequence 및 undersized fallback PASS; soft-input 문자/Backspace Repeat 유지 |
| F9 실제 경로 | 기존 두 untouched session hash 비교 | F9 Press→candidate→handler false→flag toggle, revision/hash 불변, second Press 복원, Repeat/Release None PASS |
| archive | uppercase legacy alias production verifier exit 0 | case/trailing/reserved/collision negative와 normal similar positive Windows verifier PASS; Ubuntu actual matrix pending |
| calendar | Linux year 0000 PASS | explicit `0001..9999`; Git Bash helper 0000 reject/0001·9999 accept, actual Ubuntu bundle pending |
| docs | summary 10·11절 stale report 27 next-step | report 28 lifecycle로 갱신하고 후반 stale phrase negative regression PASS |

### 3.2 현재 검증 상태

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked -- --list` | PASS, Windows named test 445개 |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| allocator/content/equipment/TUI/F9/Windows archive 표적 tests | PASS |
| combat/death/golden/monster/projectile/compatibility/long-run/headless 영향 suite | PASS |
| dependency exception/duplicate gate | PASS |
| Git Bash `scripts/r7_checkpoint.sh`, `scripts/r8_checkpoint.sh` | 각각 PASS |
| `cargo audit` | PASS, 318 dependencies, vulnerabilities 0건 |
| cargo-deny 0.19.4 `licenses bans sources` | PASS |
| clean actual bundle과 새 same-SHA 양 OS CI | pending |

첫 전체 run은 allocator 오류만 no-commit하도록 의도했으나 모든 rejected outcome을 no-commit해 `AwaitingDirection`의 기존 Playing 복귀 계약을 깨뜨렸다. 이 실패를 보존하고 internal `transaction_aborted` 표식으로 allocation/projectile/monster-phase partial mutation 오류만 rollback하도록 범위를 좁힌 뒤 두 번째 전체 workspace run이 PASS했다.

## 4. 종료 경계

시정 구현과 CI success는 report 28을 수정하거나 독립 PASS를 대신하지 않는다. 후속 독립 재감사와 별도 게시 승인 전까지 PROGRAM/PUBLICATION HOLD를 유지한다.
