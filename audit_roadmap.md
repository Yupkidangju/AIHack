# AIHack Audit Roadmap

문서 상태: active
작성일: 2026-04-28

## 1. 감사 원칙

로드맵의 목적은 일정 관리가 아니라 "다음 Phase로 넘어가도 되는가"를 판정하는 것이다.

각 Phase는 다음을 가져야 한다.

- 닫힌 목표
- 구현 항목
- 검증 명령
- 완료 판정
- 다음 Phase 진입 조건

## 2. 정합성 감사

항목:

- `spec.md` 타입 이름과 실제 Rust 타입 이름 일치
- `designs.md` 화면 이벤트와 `GameEvent` 일치
- `implementation_summary.md` 파일 책임과 실제 파일 구조 일치
- `BUILD_GUIDE.md` 명령이 실제 실행 가능
- `CHANGELOG.md`가 큰 구조 변경을 기록

검증:

```bash
rg "TODO|TBD|PLACEHOLDER" spec.md designs.md implementation_summary.md BUILD_GUIDE.md audit_roadmap.md
```

추가로 `AI_IMPLEMENTATION_DOC_STANDARD.md`의 금지 표현 목록을 기준으로 수동 확인한다. 허용 범위는 명시적으로 "잔여 리스크" 섹션에 격리된 표현뿐이다.

## 3. 위험요소 감사

| 위험 | 감사 방법 | 실패 기준 |
| --- | --- | --- |
| 레거시 직접 의존 | `rg "legacy_nethack_port_reference" src Cargo.toml` | src/Cargo에서 발견 |
| 상태 이중화 | `rg "clone\\(\\).*GameMap|resources|global" src` | 동기화용 복제 상태 발견 |
| non-determinism | seed별 replay hash 비교 | 같은 seed hash 불일치 |
| AI 우회 | `ActionIntent` 외 AI entry 검색 | AI가 session 직접 수정 |
| UI 직접 수정 | UI에서 domain mutable 접근 검색 | UI가 map/entity 수정 |

## 4. Phase별 로드맵

### Phase 0: 문서/레거시 경계

목표:

- 레거시 코드가 격리되어 있고 새 문서세트가 루트에 존재한다.

구현 항목:

- [x] `legacy_nethack_port_reference/` 생성
- [x] 기존 포트 파일 이동
- [x] `REFERENCE_INDEX.md` 작성
- [x] 새 문서세트 작성
- [x] `.gitignore`에서 주요 문서 ignore 제거

완료 기준:

- 루트에 `src/`와 `Cargo.toml`이 없음
- 루트에 `spec.md`, `designs.md`, `implementation_summary.md`, `DESIGN_DECISIONS.md`, `BUILD_GUIDE.md`, `audit_roadmap.md`, `CHANGELOG.md` 있음

### Phase 1: Headless Core

목표:

- 코드 없이 문서만 있던 상태에서 첫 deterministic runner를 만든다.

구현 항목:

- [x] Cargo 스캐폴딩
- [x] `GameRng`
- [x] `GameSession::new(seed)`
- [x] `CommandIntent::Wait`
- [x] `TurnOutcome`
- [x] replay hash

검증:

```bash
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

완료 기준:

- 같은 seed 42를 두 번 실행하면 final hash 동일

검증 증거(2026-04-28):

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: 7 passed
seed=42 turns=100 final_hash=f827bc2d4155ef66
seed=42 turns=100 repeated final_hash=f827bc2d4155ef66
seed=43 turns=100 final_hash=3ed5b4db4d5e7157
rg "legacy_nethack_port_reference" src Cargo.toml: no direct refs
```

### Phase 2: Map, Movement, Doors, Vision

목표:

- 40x20 fixture map에서 플레이어가 deterministic하게 움직인다.

구현 항목:

- [x] `GameMap`
- [x] `TileKind`
- [x] `DoorState`
- [x] movement blocker
- [x] open/close
- [x] LOS radius 8
- [x] `Observation.visible_tiles`

검증:

```bash
cargo test map movement doors vision
```

완료 기준:

- 닫힌 문은 이동/LOS를 막고 열린 문은 통과 가능

검증 증거(2026-04-28 Phase 2):

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test map: pass
cargo test movement: pass
cargo test doors: pass
cargo test vision: pass
cargo test observation: pass
seed=42 turns=100 final_hash=1aad6f4049778b0e
rg "legacy_nethack_port_reference" src Cargo.toml: no direct refs
```

### Phase 3: Combat and Death

목표:

- bump attack과 사망 이벤트를 구현한다.

구현 항목:

- [x] hit formula
- [x] damage formula
- [x] jackal/goblin combat
- [x] `EntityDied`
- [x] `RunState::GameOver`

검증:

```bash
cargo test --test combat
cargo test --test death
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

검증 증거(2026-04-28 Phase 3):

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test --test combat: pass
cargo test --test death: pass
seed=42 turns=100 final_hash=8b20a23301eea977
rg "legacy_nethack_port_reference" src Cargo.toml tests: no direct refs
```

완료 기준:

- seed 고정 전투 결과가 snapshot과 일치

### Phase 4: Items and Inventory

목표:

- 아이템 pickup, inventory letter, wield, quaff를 구현한다.

구현 항목:

- [x] item store
- [x] inventory letter map
- [x] pickup
- [x] wield dagger
- [x] quaff healing
- [x] item consumed event

검증:

```bash
cargo test --test inventory
cargo test --test items
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

검증 증거(2026-04-28 Phase 4):

```text
cargo fmt --check: pass
cargo clippy --all-targets -- -D warnings: pass
cargo test: pass
cargo test --test items: pass
cargo test --test inventory: pass
seed=42 turns=100 final_hash=00ba578d933177f2
rg "legacy_nethack_port_reference" src Cargo.toml tests: no direct refs
rg "Drop|Read|Zap|Throw|Descend|Ascend|save/load|ratatui|crossterm|monster AI|pathfind" src: no matches
rg "rand::|thread_rng|random" src: only src/core/rng.rs
```

완료 기준:

- serde/snapshot roundtrip 후 inventory letter 유지. 실제 file save/load는 Phase 9 범위

### Phase 5: Levels and Stairs

목표:

- 1층과 2층을 왕복하고 level state를 보존한다.

구현 항목:

- [x] `LevelId`
- [x] level registry
- [x] stairs up/down
- [x] current level snapshot

검증:

```bash
cargo test --test stairs --test levels
```

완료 기준:

- 1층 아이템 상태가 2층 왕복 후 유지

Phase 5 완료 메모:

- `tests/levels.rs`, `tests/stairs.rs`, `cargo test` 통과.
- `seed=42 turns=100 final_hash=88886c28698a1730`.

### Phase 6: Monster AI

목표:

- 몬스터 의도 수집과 적용을 player action과 분리한다.

구현 항목:

- [ ] `MonsterAiKind`
- [ ] wander
- [ ] chase on sight
- [ ] stationary
- [ ] hostile attitude

검증:

```bash
cargo test monster_ai
```

완료 기준:

- goblin은 LOS 내에서 거리가 줄어드는 방향으로 이동

### Phase 7: NetHack Interaction Set 1

목표:

- NetHack 느낌을 만드는 핵심 상호작용 일부를 추가한다.

구현 항목:

- [ ] simple trap
- [ ] wand zap
- [ ] throw item
- [ ] read scroll
- [ ] search hidden door

완료 기준:

- 각 기능은 하나 이상의 golden scenario를 가진다.

### Phase 8: Legacy Rule Absorption

목표:

- 레거시 코드에서 선별한 규칙 20개를 새 계약으로 재작성한다.

규칙 후보:

- dice/RNG
- monster difficulty
- inventory letter policy
- hunger tick
- trap detection
- door kicking
- passive attack
- wand beam
- potion healing
- scroll identify
- level teleport
- corpse drop
- hallucination message
- encumbrance
- armor AC
- weapon damage
- luck adjustment
- shop price base
- prayer cooldown
- death score

완료 기준:

- 20개 golden test 통과

### Phase 9: Save, Load, Replay

목표:

- 세션을 저장하고 같은 hash path로 재개한다.

검증:

```bash
cargo test save replay
cargo run --bin aihack-headless -- --seed 42 --turns 1000
```

완료 기준:

- save at turn 100, load, continue to 1000 hash equals uninterrupted run

### Phase 10: TUI Adapter

목표:

- core를 수정하지 않고 TUI로 플레이한다.

완료 기준:

- UI code가 `GameClient` trait만 사용
- 80x28 화면에서 겹침 없음

### Phase 11: AI Observation/ActionSpace

목표:

- AI가 매 턴 typed observation을 받고 legal action만 실행한다.

완료 기준:

- observation JSON schema snapshot 통과
- illegal action은 reject event

### Phase 12: LLM Narrative

목표:

- LLM은 메시지 꾸밈만 담당한다.

완료 기준:

- timeout 2초
- fallback text 존재
- LLM 실패 시 game hash 불변

### Phase 13: Limited LLM Decision Support

목표:

- LLM이 명령 후보를 제안하되 validator가 최종 결정한다.

완료 기준:

- LLM 출력이 invalid여도 게임 상태 불변
- legal action 중 하나만 실행 가능

## 5. 릴리즈 게이트

v0.1:

- Phase 1-6 완료
- `cargo test` 통과
- seed 42/7/1234 headless 1000턴 통과
- save/load hash 일치

v0.2:

- Phase 7-9 완료
- golden rules 20개 통과

v0.3:

- Phase 10-11 완료
- TUI와 AI observation 안정화

v0.4:

- Phase 12-13 완료
- LLM timeout/fallback 검증

### Phase 10A~10E: Modern TUI/UX

목표:

- headless core와 AI observation 경계를 유지하면서 ASCII TUI의 정보 가독성, 마우스 접근성, presentation-only 효과를 단계적으로 도입한다.

구현 항목:

- [ ] 10A layout snapshot과 keyboard-only TUI 안정화
- [ ] 10B hover/inspect, priority message, command hint
- [ ] 10C mouse click/hover 좌표 매핑과 inventory click selection
- [ ] 10D `GameEvent -> UiEffectEvent` projection
- [ ] 10E reduced motion, color profile, 자동 라벨 우선순위

검증:

```bash
cargo test ui_layout
cargo test ui_input_mapping
cargo test ui_effect_projection
cargo test replay_hash_ignores_ui_effects
cargo run --bin aihack-headless -- --seed 42 --turns 1000
```

완료 기준:

- 80x28에서 map/status/log/command bar가 겹치지 않는다.
- hover/inspect와 panel focus는 턴을 진행하지 않는다.
- mouse disabled 환경에서도 모든 v0.1 필수 명령이 keyboard-only로 가능하다.
- animation on/off 및 reduced motion 설정이 headless replay hash를 바꾸지 않는다.
- 그래픽 타일셋 관련 의존성 또는 렌더러가 추가되지 않는다.
