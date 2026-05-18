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

- [x] `MonsterAiKind`
- [x] wander
- [x] chase on sight
- [x] stationary
- [x] hostile attitude

검증:

```bash
cargo test --test monster_ai
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

완료 기준:

- goblin은 LOS 내에서 거리가 줄어드는 방향으로 이동

Phase 6 완료 메모:

- `tests/monster_ai.rs` 추가, current-level filter / turn gate / chase / wander / stationary / death stop 검증 완료.
- `EntityMoved { entity, from, to }` event shape로 player/monster movement identity를 통일했다.
- `seed=42 turns=100 final_turn=20 final_hash=2fb549b5d2e1e67f`.

### Phase 7: NetHack Interaction Set 1

목표:

- NetHack 느낌을 만드는 핵심 상호작용 일부를 추가한다.

구현 항목:

- [x] simple trap
- [x] wand zap
- [x] throw item
- [x] read scroll
- [x] search hidden door

완료 기준:

- 각 기능은 하나 이상의 golden scenario를 가진다.

Phase 7 완료 메모:

- `tests/traps.rs`, `tests/projectiles.rs`, `tests/items.rs`, `tests/observation.rs`로 hidden tile / trap / throw / zap / read를 검증했다.
- `seed=42 turns=100 final_turn=20 final_hash=5aecd83cf284cb25`.

### Phase 8: Legacy Rule Absorption

목표:

- 레거시 코드에서 선별한 규칙 20개를 새 계약으로 재작성한다.

규칙 후보:

- [x] dice/RNG
- [x] monster difficulty
- [x] inventory letter policy
- [x] hunger tick
- [x] trap detection
- [x] door kicking
- [x] passive attack
- [x] wand beam
- [x] potion healing
- [x] scroll identify
- [x] level teleport
- [x] corpse drop
- [x] hallucination message
- [x] encumbrance
- [x] armor AC
- [x] weapon damage
- [x] luck adjustment
- [x] shop price base
- [x] prayer cooldown
- [x] death score

완료 기준:

- 20개 golden test 통과

Phase 8 완료 메모:

- `tests/golden_phase8_rules.rs`에 P8-G01~P8-G20 20개 golden scenario를 추가했다.
- `seed=42 turns=100 final_turn=20 final_hash=4c77dafb19dd2226`.

### Phase 9: Save, Load, Replay

목표:

- 세션을 저장하고 같은 hash path로 재개한다.

Phase 9 완료 메모:

- `tests/save_load.rs`, `tests/replay.rs`로 save/load hash equality와 replay JSONL schema를 검증했다.
- `seed=42 turns=100 final_turn=20 final_hash=4c77dafb19dd2226`.

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

Phase 10 완료 메모:

- `tests/ui_layout.rs`, `tests/ui_input_mapping.rs`, `tests/ui_effect_projection.rs`, `tests/ui_runtime_smoke.rs`를 추가했다.
- `cargo run --bin aihack -- --seed 42`가 small-terminal fallback과 clean exit를 보였다.

완료 기준:

- UI code가 `GameClient` trait만 사용
- 80x28 화면에서 겹침 없음

### Phase 11: AI Observation/ActionSpace

Phase 11 완료 메모:

- `tests/ai_api_schema.rs`, `tests/action_space.rs`를 추가해 Observation/ActionSpace schema freeze를 검증했다.
- save/load와 TUI consumer가 같은 AI-facing schema를 사용하도록 고정했다.


목표:

- AI가 매 턴 typed observation을 받고 legal action만 실행한다.

완료 기준:

- observation JSON schema snapshot 통과
- illegal action은 reject event

### Phase 12: LLM Narrative

Phase 12 완료 메모:

- `tests/llm_narrative.rs`를 추가해 timeout/fallback/non-hash narrative adapter를 검증했다.
- TUI consumer smoke가 same narrative response contract를 읽는 것을 확인했다.


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

- Phase 1-15 완료
- `cargo test` 통과
- seed 42/7/1234 headless 1000턴 통과
- save/load hash 일치
- replay 재생 hash 일치
- `Observation` schema snapshot 통과
- known debt triage 완료

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

- [x] 10A layout snapshot과 keyboard-only TUI 안정화
- [x] 10B hover/inspect, priority message, command hint
- [x] 10C mouse click/hover 좌표 매핑과 inventory click selection
- [x] 10D `GameEvent -> UiEffectEvent` projection
- [x] 10E reduced motion, color profile
- [ ] 10E 자동 라벨 우선순위

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


### Phase 13 완료 메모

- `tests/llm_decision_support.rs`를 추가해 legal/illegal/timeout/non-hash suggestion layer를 검증했다.
- suggestion execution은 `session.submit(...)` 경로를 통하는 경우에만 허용되도록 고정했다.

---

### Phase 16: RunState & CommandIntent 계약 정렬

목표:

- `spec.md` 8.2 `RunState` 계약과 실제 코드를 일치시킨다.
- `spec.md` 8.3 `CommandIntent` 계약과 실제 코드를 일치시킨다.

구현 항목:

- [x] `RunState` 확장: Title, CharacterCreation, AwaitingDirection, AwaitingInventorySelection, MorePrompt, GameOver with cause/final_score
- [x] `DirectionalAction`, `InventoryAction` 신규 정의
- [x] `AcknowledgeMore` CommandIntent 추가
- [x] `submit()` RunState 분기 처리
- [x] `RunStateSummary` 확장
- [x] `GameWorld.last_death_cause` 추가

Phase 16 완료 메모:

- `spec.md` 8.2/8.3 계약과 실제 코드를 일치시켰다.
- `GameSession::new()`는 Title 상태로 시작하고, `new_for_playing()`은 Playing 상태로 시작한다.
- snapshot hash가 변경되었고, 새 기준값 `seed=42 turns=1000 final_hash=569bc36895258349`으로 갱신했다.

검증:

```bash
cargo test --test observation --test action_space
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

완료 기준:

- `tests/observation.rs`, `tests/action_space.rs`가 새 RunState 종류별 legal_actions를 검증한다.
- headless baseline hash 동일 유지.

---

### Phase 17: Game Flow Screens

목표:

- `designs.md` 2의 Title, Character Creation, Game Over 화면을 구현한다.
- `designs.md` 11의 Game Over 필수 표시 항목을 구현한다.

구현 항목:

- [x] TUI 화면 분기: Title/CharacterCreation/Playing/GameOver
- [x] Title 화면: "AIHack", "Press Enter to Start", L/Q
- [x] Character Creation 화면: 기본 캐릭터 정보, Enter/Esc
- [x] Game Over 화면: 사망 원인, turn, depth, defeated, score, seed, N/Q/E
- [x] `AwaitingDirection` 상태: "Choose direction" 표시
- [x] `AwaitingInventorySelection` 상태: 인벤토리 오버레이
- [x] `MorePrompt` 상태: "--More--" 표시

Phase 17 완료 메모:

- `tests/ui_screens.rs` 8개 테스트 통과.
- headless baseline hash 동일 유지.

검증:

```bash
cargo test --test ui_runtime_smoke --test ui_layout --test ui_input_mapping
cargo run --bin aihack -- --seed 42
```

완료 기준:

- 80x28 터미널에서 Title -> Character Creation -> Playing -> Game Over 흐름 확인
- Game Over에서 N/Q/E 입력 동작 확인

---

### Phase 18: Debug Observation Panel (F9 Toggle)

목표:

- `designs.md` 10의 Debug Observation 패널을 F9 키로 토글 가능하게 구현한다.
- schema_version, seed, turn, snapshot_hash, legal_actions, recent events, visible tile/entity 수를 표시한다.

구현 항목:

- [x] `TuiApp.debug_observation_visible` 상태 추가
- [x] F9 키 입력 처리 (UI-only 토글)
- [x] Debug Observation lines 생성 함수
- [x] 80x28에서도 overlay로 표시

검증:

```bash
cargo test --test ui_debug --test ui_layout
```

완료 기준:

- F9 키 입력 시 debug 패널 토글 동작
- debug 패널이 기존 패널과 겹치지 않음

Phase 18 완료 메모:

- `tests/ui_debug.rs` 3개 테스트 통과.
- headless baseline hash 동일 유지 (UI-only 변경).

---

### Phase 19: Auto-Label Priority System

목표:

- `spec.md` 15.7의 Phase 10E 후속 "자동 라벨 우선순위"를 구현한다.
- 새로 보인 hostile/item/danger 라벨을 최대 3개, 1200ms 표시한다.

구현 항목:

- [x] `AutoLabel`, `LabelKind` 타입 정의
- [x] 라벨 우선순위: hostile adjacent > low HP > stairs > unidentified item > passive monster
- [x] `collect_auto_labels()` 함수
- [x] 맵 overlay 라벨 렌더링
- [x] `NewEntityLabel` effect 통합

검증:

```bash
cargo test --test ui_labels --test ui_effect_projection --test ui_layout
```

완료 기준:

- 최대 3개 라벨 표시
- 우선순위 정렬
- 1200ms 지속 시간
- headless hash 무영향

Phase 19 완료 메모:

- `tests/ui_labels.rs` 7개 테스트 통과.
- headless baseline hash 동일 유지 (UI-only 변경).

---

### Phase 20: 데이터 외부화 및 모듈 분리

목표:

- `spec.md` 7의 예정 구조와 실제 구조를 일치시킨다.
- `src/domain/status.rs`, `src/data/`, `src/ui/debug/`를 생성한다.

구현 항목:

- [x] `src/domain/status.rs`: Status, HungerState 분리
- [x] `src/data/items.toml`, `monsters.toml`, `levels/main_1.toml`
- [x] `src/data/mod.rs`: TOML 로더
- [x] `GameWorld` 필드 재배치: `status()`/`set_status()` getter/setter 추가

검증:

```bash
cargo test --test data_loading --test save_load
cargo run --bin aihack-headless -- --seed 42 --turns 1000
```

완료 기준:

- TOML 파일 로드 검증
- Status save/load roundtrip 검증
- headless baseline hash 동일 유지

Phase 20 완료 메모:

- `tests/data_loading.rs` 9개 테스트 통과.
- `GameWorld` 내부 필드 구조를 유지하여 hash 변경 없음.
- `src/ui/debug/`는 inline 구현으로 대체하여 선택적 분리는 미룸.
