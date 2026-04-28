# Test Spec: AIHack Phase 1 Headless Core

문서 상태: approved-test-spec
작성일: 2026-04-28
대상 PRD: `.omx/plans/prd-phase1-headless-core.md`
범위: Phase 1 Headless Core 검증만 해당

## 1. 테스트 목표

Phase 1 구현이 다음 성질을 만족하는지 검증한다.

1. 루트 Rust 패키지가 빌드된다.
2. `GameRng`는 seed 기반으로 deterministic하다.
3. `GameSession::new(seed)`는 안정적인 초기 상태를 만든다.
4. `CommandIntent::Wait`은 턴을 정확히 1 증가시키고 accepted outcome을 반환한다.
5. snapshot hash는 같은 seed/turn/input에서 동일하다.
6. headless runner는 같은 명령을 두 번 실행했을 때 같은 final hash를 출력한다.
7. 레거시 직접 의존과 Phase 2 scope creep이 없다.

## 2. 테스트 매트릭스

| ID | 종류 | 대상 | 검증 내용 |
| --- | --- | --- | --- |
| T1 | unit | `GameRng` | 같은 seed의 첫 N개 값 동일 |
| T2 | unit | `GameRng` | 다른 seed의 값 sequence가 달라짐 |
| T3 | unit | `GameSession::new` | seed, turn=0, Playing 상태 확인 |
| T4 | unit | `submit(Wait)` | accepted=true, turn_advanced=true, turn +1 |
| T5 | unit | `TurnOutcome` | events와 next_state가 현재 상태와 일치 |
| T6 | unit | snapshot hash | 같은 snapshot 입력은 같은 hash |
| T7 | integration | headless runner | `--turns 0` 실행 성공 |
| T8 | integration | headless runner | seed 42 turns 100 두 번 실행 final_hash 동일 |
| T9 | integration | headless runner | seed 42와 43 turns 100 final_hash 다름 |
| T10 | audit | dependency boundary | `legacy_nethack_port_reference` direct reference 없음 |
| T11 | audit | scope boundary | `src/ui`, map/movement/combat/item 구현 없음 |
| T12 | quality | cargo | fmt/clippy/test 통과 |

## 3. Unit Test 상세

### T1: 같은 seed RNG sequence 동일

Given:

```rust
let mut a = GameRng::new(42);
let mut b = GameRng::new(42);
```

When:

- 두 RNG에서 `next_u64()` 또는 동등 API를 10회 호출한다.

Then:

- 모든 index의 값이 동일하다.

### T2: 다른 seed RNG sequence 차이

Given:

```rust
let mut a = GameRng::new(42);
let mut b = GameRng::new(43);
```

Then:

- 첫 10개 값 중 최소 1개 이상이 다르다.

### T3: `GameSession::new(seed)` 초기 상태

Expected:

```text
seed = 42
turn = 0
state = Playing
event_log length = 0 또는 문서화된 초기 이벤트 수
```

초기 이벤트를 생성할 경우 PRD와 코드 주석에 명시해야 한다.

### T4: `submit(Wait)` 결과

Given:

```rust
let mut session = GameSession::new(42);
let outcome = session.submit(CommandIntent::Wait);
```

Expected:

```text
outcome.accepted = true
outcome.turn_advanced = true
session.turn = 1
outcome.next_state = Playing
outcome.snapshot_hash != empty
```

### T5: 반복 Wait

Given:

- seed 42 session.

When:

- `Wait` 100회 제출.

Then:

```text
session.turn = 100
모든 accepted = true
모든 turn_advanced = true
final hash는 같은 실행에서 재계산해도 동일
```

### T6: stable snapshot hash

Given:

- 같은 seed, turn, state, event summary를 가진 snapshot 2개.

Then:

- `snapshot_hash(a) == snapshot_hash(b)`.

주의:

- `std::collections::hash_map::DefaultHasher`처럼 구현/버전 안정성을 보장하기 어려운 해시는 사용하지 않는다.
- 권장 알고리즘은 자체 FNV-1a 64-bit다.

## 4. Integration Test / Manual Command

### T7: turns 0

```bash
cargo run --bin aihack-headless -- --seed 42 --turns 0
```

Expected stdout contains:

```text
seed=42
turns=0
final_turn=0
final_hash=
```

Exit code: 0.

### T8: seed 42 deterministic

```bash
cargo run --bin aihack-headless -- --seed 42 --turns 100
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

Expected:

- 두 실행의 `final_hash`가 동일하다.

### T9: seed 차이 반영

```bash
cargo run --bin aihack-headless -- --seed 42 --turns 100
cargo run --bin aihack-headless -- --seed 43 --turns 100
```

Expected:

- 두 실행의 `final_hash`가 다르다.

## 5. Audit Commands

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 0
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

레거시 직접 의존 검사:

```bash
rg "legacy_nethack_port_reference" src Cargo.toml
```

Expected:

- 결과 없음.

Scope creep 검사:

```bash
find src -maxdepth 3 -type f | sort
```

Expected:

- Phase 1 PRD의 파일 목록을 벗어난 map/movement/combat/item/TUI 구현 파일 없음.

## 6. 실패 시 수정 루프

1. 실패한 테스트 이름과 명령을 기록한다.
2. 원인을 Phase 1 범위 안에서만 수정한다.
3. 수정이 Phase 2 이상의 타입/기능을 요구하면 중단하고 PRD 갱신 필요 여부를 보고한다.
4. `cargo fmt --check`, `cargo clippy`, `cargo test`, headless runner를 다시 실행한다.

## 7. 완료 증거 형식

Ralph 완료 보고는 다음 형식을 포함해야 한다.

```text
변경 파일:
- ...

검증:
- cargo fmt --check: pass
- cargo clippy --all-targets -- -D warnings: pass
- cargo test: pass
- cargo run --bin aihack-headless -- --seed 42 --turns 0: pass
- cargo run --bin aihack-headless -- --seed 42 --turns 100: pass, final_hash=<값>
- repeated seed 42 hash compare: pass
- seed 43 differs: pass

남은 리스크:
- ...
```
