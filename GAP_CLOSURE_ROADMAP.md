# AIHack 문서-구현 Gap Closure 계획

문서 상태: active
작성일: 2026-05-18
버전: v0.2.0-rc2 (Phase 15 완료 기준)

## 1. 문서 운영 규칙

이 문서는 2026-05-18 현재 AIHack 프로젝트에서 `spec.md`, `designs.md`에 명시된 항목과 실제 구현 코드 간의 gap을 닫는 구체적인 구현 계획이다.

- 이 문서의 모든 계약은 `AI_IMPLEMENTATION_DOC_STANDARD.md` 기준을 따른다.
- 이 문서를 받은 구현자는 추가 기획 회의 없이 바로 구현에 들어갈 수 있어야 한다.
- 금지 표현: "적당히", "필요시", "원하면", "추후 고려", "게임답게", "자연스럽게", "유연하게", "대충 이 정도"

## 2. 목표

### 2.1 단기 목표 (Phase 16~17): 문서-코드 불일치 해소

- `spec.md` 8.2 `RunState` 계약과 실제 코드를 일치시킨다.
- `spec.md` 8.3 `CommandIntent` 계약과 실제 코드를 일치시킨다.
- `designs.md` 2 화면 모드 중 미구현 3개 화면을 구현한다.
- `designs.md` 11 Game Over 화면을 구현한다.
- `designs.md` 10 Debug Observation 패널을 F9 토글 방식으로 구현한다.

### 2.2 중기 목표 (Phase 18~19): UX 미구현 완료

- Phase 10E 자동 라벨 우선순위를 구현한다.
- 데이터 외부화를 위한 `src/data/` 구조를 도입한다.
- `src/domain/status.rs`를 분리한다.

### 2.3 성공 기준

- `cargo test` 전체 통과
- `cargo run --bin aihack -- --seed 42`에서 Title -> Character Creation -> Playing -> Game Over 흐름 확인
- `cargo run --bin aihack-headless -- --seed 42 --turns 1000` 기준 hash 동일 유지
- F9 키로 debug observation 토글 동작
- Game Over 화면에서 사망 원인, turn, depth, score, seed 표시 확인
- spec.md, designs.md, audit_roadmap.md, CHANGELOG.md, DESIGN_DECISIONS.md 동시 갱신

## 3. 비목표

- 절차적 던전 생성 (v0.2+ 계획)
- 그래픽 타일셋
- 네트워크 멀티플레이
- LLM 자유 텍스트 명령 실행
- 몬스터 아이템 사용 (후속)
- 드래그앤드롭 인벤토리 (v0.3)
- 플레이어 중심 맵 스크롤 (40x20 고정 유지)
- Cogmind급 전체 효과 parity (v0.3)
- ECS/Legion 재도입
- NetHack 3.6.7 전체 규칙 1:1 복제

## 4. 동결된 핵심 결정

| 결정 | 값 | 이유 |
|---|---|---|
| 버전 | v0.2.0-rc3 | Phase 16~17 gap closure 완료 시 MINOR bump |
| 언어 | Rust 2021 stable | 기존 스택 유지 |
| 런타임 모델 | 단일 스레드 턴 트랜잭션 | 동결 |
| 상태 소유자 | `GameSession` 단일 원천 | 동결 |
| UI 경계 | UI는 `GameSnapshot` 읽기, `CommandIntent` 쓰기만 가능 | 동결 |
| AI 경계 | AI는 `Observation` 읽기, `ActionIntent` 쓰기만 가능 | 동결 |
| RNG | seed 기반 deterministic | 동결 |
| replay hash | FNV-1a 64-bit | 동결 |
| 레거시 코드 | 참조만, 직접 import 금지 | 동결 |
| 저장 | JSON save v1, replay JSONL | 동결 |

## 5. Gap 분류와 우선순위

### 5.1 A급: spec.md 명시 계약과 실제 코드 불일치 (즉시 구현)

| # | Gap | 위치 | 영향 |
|---|---|---|---|
| A1 | `RunState::Title` 누락 | spec 8.2 | 화면 흐름 시작점 부재 |
| A2 | `RunState::CharacterCreation` 누락 | spec 8.2 | 캐릭터 생성 화면 부재 |
| A3 | `RunState::AwaitingDirection` 누락 | spec 8.2 | 방향 대기 상태 부재 |
| A4 | `RunState::AwaitingInventorySelection` 누락 | spec 8.2 | 인벤토리 선택 대기 상태 부재 |
| A5 | `RunState::MorePrompt` 누락 | spec 8.2 | 메시지 더보기 프롬프트 부재 |
| A6 | `GameOver` 필드 부재 (cause, final_score) | spec 8.2 | 사망 정보 미전달 |
| A7 | `AcknowledgeMore` 명령 누락 | spec 8.3 | MorePrompt 상태 진출 불가 |
| A8 | `Pray` 명령 spec 미반영 | spec 8.3 | 문서-코드 불일치 |

### 5.2 B급: designs.md 명시 UI/화면 미구현 (즉시 구현)

| # | Gap | 위치 | 영향 |
|---|---|---|---|
| B1 | `screen.title` 미구현 | designs.md 2 | 게임 시작 흐름 부재 |
| B2 | `screen.character_creation` 미구현 | designs.md 2 | 캐릭터 확정 흐름 부재 |
| B3 | `screen.game_over` 미구현 | designs.md 2, 11 | 사망/점수/재시작 화면 부재 |
| B4 | `screen.debug_observation` F9 토글 미구현 | designs.md 10 | 디버그 패널 접근 불편 |
| B5 | Game Over 화면 필수 표시 8개 항목 미구현 | designs.md 11 | 사망 정보 미시각화 |

### 5.3 C급: spec.md 예정 구조 미구현 (선택적, 후속)

| # | Gap | 위치 | 영향 |
|---|---|---|---|
| C1 | `src/domain/status.rs` 미생성 | spec 7 | 상태 필드 산재 |
| C2 | `src/data/items.toml` 미생성 | spec 7 | 하드코딩 데이터 |
| C3 | `src/data/monsters.toml` 미생성 | spec 7 | 하드코딩 데이터 |
| C4 | `src/data/levels/` 미생성 | spec 7 | 하드코딩 데이터 |
| C5 | `src/ui/debug/` 미생성 | spec 7 | 디버그 기능 TUI에 인라인 |

### 5.4 D급: Phase 10E 후속 범위 (후속)

| # | Gap | 위치 | 영향 |
|---|---|---|---|
| D1 | 자동 라벨 우선순위 미구현 | spec 15.7 | 호스타일/아이템 자동 라벨 없음 |

## 6. Phase별 구현 계획

### Phase 16: RunState & CommandIntent 계약 정렬

#### 목표

`spec.md` 8.2, 8.3에 정의된 계약과 실제 코드를 일치시킨다. RunState 확장과 CommandIntent 추가/정렬을 한 번에 수행한다.

#### 구현 항목

**16.1 RunState 확장** (`src/core/session.rs`, `src/core/observation.rs`)

현재 `RunState` (lines 20-23):

```rust
pub enum RunState {
    Playing,
    GameOver,
}
```

변경 후:

```rust
pub enum RunState {
    Title,
    CharacterCreation,
    Playing,
    AwaitingDirection { action: DirectionalAction },
    AwaitingInventorySelection { action: InventoryAction },
    MorePrompt,
    GameOver { cause: DeathCause, final_score: i32 },
}
```

`DirectionalAction` 정의 (`src/core/action.rs`에 추가):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DirectionalAction {
    Open,
    Close,
    Kick,
}
```

`InventoryAction` 정의 (`src/core/action.rs`에 추가):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InventoryAction {
    Drop,
    Wield,
    Wear,
    Quaff,
    Read,
}
```

**구현 상세:**

- `AwaitingDirection`은 `Open`, `Close`, `Kick` 명령 수행 후 방향 입력을 기다리는 상태다.
- `AwaitingInventorySelection`은 `Drop`, `Wield`, `Wear`, `Quaff`, `Read` 명령 수행 후 인벤토리 항목 선택을 기다리는 상태다.
- `MorePrompt`는 한 턴에 5개 초과 메시지 발생 시 표시되는 상태다. (spec 8)
- `GameOver { cause, final_score }`는 기존 `GameOver` bare variant를 대체한다.
- `RunState::Title` 시작 시 `GameSession::new(seed)`는 `RunState::Title`로 시작하고, Enter 키 입력 시 `CharacterCreation`으로 전환한다.
- `RunState::CharacterCreation`에서 Enter 키 입력 시 기본 캐릭터를 확정하고 `Playing`으로 전환한다.

**16.2 CommandIntent 추가** (`src/core/action.rs`)

추가: `AcknowledgeMore`

```rust
pub enum CommandIntent {
    // ... existing variants ...
    AcknowledgeMore,
}
```

**spec.md 8.3에 `Pray` 반영:**

`spec.md` 8.3 CommandIntent 목록에 `Pray`를 추가한다.

**16.3 RunStateSummary 확장** (`src/core/observation.rs`)

```rust
pub enum RunStateSummary {
    Title,
    CharacterCreation,
    Playing,
    AwaitingDirection,
    AwaitingInventorySelection,
    MorePrompt,
    GameOver,
}
```

`GameOver`는 summary 레벨에서 cause/final_score를 생략하고 `GameOver`로만 표시한다.

**16.4 submit() 처리 변경** (`src/core/session.rs`)

- `RunState::Title` 상태에서는 `CommandIntent::Wait`, `Quit`만 accepted된다. `Wait`는 `CharacterCreation`으로 전환한다.
- `RunState::CharacterCreation` 상태에서는 `CommandIntent::Wait`, `Quit`만 accepted된다. `Wait`는 `Playing`으로 전환하고 fixture world를 초기화한다.
- `RunState::AwaitingDirection { action }` 상태에서는 `Move(dir)` 입력 시 해당 `DirectionalAction`을 수행한다.
- `RunState::AwaitingInventorySelection { action }` 상태에서는 인벤토리 letter 입력 시 해당 `InventoryAction`을 수행한다.
- `RunState::MorePrompt` 상태에서는 `AcknowledgeMore`만 accepted되며 `Playing`으로 복귀한다.
- `RunState::Playing` 상태에서는 기존 모든 명령이 가능하다.
- `RunState::GameOver { cause, final_score }` 상태에서는 `Quit`만 accepted된다. `New Run`은 `GameSession::new(seed)` 재생성으로 처리한다.

**16.5 GameOver 사망 정보 저장** (`src/core/session.rs`)

현재 `death::state_after_deaths()`가 `RunState::GameOver`를 반환한다. 변경 후 `RunState::GameOver { cause, final_score }`를 반환하도록 수정한다.

```rust
// death.rs 또는 session.rs 내부
pub fn state_after_deaths(world: &GameWorld) -> RunState {
    if let Some(player) = world.entities.get(world.player_id) {
        if !player.alive {
            let cause = world.last_death_cause.unwrap_or(DeathCause::Combat { attacker: EntityId(0) });
            let final_score = score::calculate_death_score(world);
            return RunState::GameOver { cause, final_score };
        }
    }
    RunState::Playing
}
```

`GameWorld`에 `last_death_cause: Option<DeathCause>` 필드를 추가한다.

**16.6 Observation legal_actions 갱신**

각 RunState에 맞는 legal_actions를 생성한다:

- `Title`: `[Wait, Quit]`
- `CharacterCreation`: `[Wait, Quit]`
- `Playing`: 기존과 동일
- `AwaitingDirection`: `[Move(N), Move(NE), ... Move(NW), Quit]`
- `AwaitingInventorySelection`: 인벤토리 항목 letter + `[Quit]`
- `MorePrompt`: `[AcknowledgeMore]`
- `GameOver`: `[Quit]`

#### 검증 명령

```bash
cargo test --test observation
cargo test --test action_space
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 100
```

#### 완료 기준

- `tests/observation.rs`: 새 RunState 종류별 legal_actions 검증 추가
- `tests/action_space.rs`: AwaitingDirection/MorePrompt action space 검증 추가
- headless baseline hash 동일 유지

---

### Phase 17: Game Flow Screens (Title, Character Creation, Game Over)

#### 목표

`designs.md` 2, 11에 명시된 화면 3개를 구현한다. TUI는 `RunState`에 따라 다른 화면을 렌더링한다.

#### 구현 항목

**17.1 TUI 스크린 모드 분기** (`src/ui/tui/mod.rs`)

`run_tui()` 내부에서 `app.session.state`를 읽고 화면을 분기한다:

```rust
match app.session.state {
    RunState::Title => render_title_screen(&mut app, frame, size),
    RunState::CharacterCreation => render_character_creation_screen(&mut app, frame, size),
    RunState::Playing | RunState::AwaitingDirection { .. } | RunState::AwaitingInventorySelection { .. } | RunState::MorePrompt => render_play_screen(&mut app, frame, size),
    RunState::GameOver { cause, final_score } => render_game_over_screen(&mut app, frame, size, cause, final_score),
}
```

**17.2 Title 화면** (`src/ui/tui/mod.rs` 또는 `src/ui/tui/screens.rs` 신규)

표시 항목:
- "AIHack" 타이틀 (ASCII art 또는 큰 글씨)
- "Press Enter to Start"
- "L - Load Game"
- "Q - Quit"

입력 처리:
- `Enter` -> `CommandIntent::Wait` 제출 (Title -> CharacterCreation 전환)
- `l` -> load dialog 또는 `CommandIntent::Wait` (load 처리)
- `q` -> `CommandIntent::Quit` (프로세스 종료)
- 기타 키 -> 무시 (rejected, no-turn)

레이아웃:
- 최소 터미널 80x28
- 타이틀 중앙 정렬
- 메뉴 항목 중앙 하단 배치

**17.3 Character Creation 화면**

표시 항목:
- "Character Creation"
- 기본 캐릭터 정보:
  - Class: Adventurer
  - HP: 16/16
  - Strength: 10
  - Dexterity: 10
  - AC: 0
- "Press Enter to confirm"
- "ESC - Back to Title"

입력 처리:
- `Enter` -> `CommandIntent::Wait` 제출 (CharacterCreation -> Playing 전환, world 초기화)
- `Esc` -> `CommandIntent::Wait` 제출 (CharacterCreation -> Title 전환)
- 기타 키 -> 무시

레이아웃:
- 최소 터미널 80x28
- 캐릭터 정보 중앙 정렬

**17.4 Game Over 화면** (`src/ui/tui/mod.rs` 또는 `src/ui/tui/screens.rs`)

필수 표시 (designs.md 11):
- 사망 원인 (DeathCause formatter)
- turn 번호
- depth (current_level.depth)
- defeated monsters count (`world.kill_count`)
- score (`final_score`)
- replay seed (`world.meta.seed`)
- "N - New Run"
- "Q - Quit"
- "E - Export Replay" (replay log 있을 경우)

입력 처리:
- `n` -> 새 `GameSession::new(랜덤 또는 이전 seed)` 생성, `RunState::Title` 또는 `RunState::CharacterCreation`로 전환
- `q` -> `CommandIntent::Quit` 제출, TUI 종료
- `e` -> replay 파일 경로를 message로 표시 (기존 `aihack-headless --replay-out` 사용)

레이아웃:
- 최소 터미널 80x28
- 상단: "GAME OVER" + 사망 원인
- 중앙: 통계 (turn, depth, defeated, score, seed) 2열 테이블 형태
- 하단: 버튼/메뉴

```text
+-----------------------------------------------------------+
|                      GAME OVER                            |
|                Killed by a goblin                         |
+-----------------------------------------------------------+
|  Turn:        42      Depth:        1                   |
|  Defeated:    3       Score:        150                 |
|  Seed:        42                                          |
+-----------------------------------------------------------+
|  [N] New Run    [Q] Quit    [E] Export Replay           |
+-----------------------------------------------------------+
```

**17.5 `AwaitingDirection` 상태 처리** (`src/ui/tui/mod.rs`)

`RunState::AwaitingDirection { action }` 상태에서는:
- 화면 하단에 "Choose direction: [hjklyubn]" 표시
- 방향키 입력 시 해당 `DirectionalAction` 수행
- `Esc` 입력 시 `Playing`으로 복귀 (취소)

**17.6 `AwaitingInventorySelection` 상태 처리**

`RunState::AwaitingInventorySelection { action }` 상태에서는:
- 화면 중앙에 인벤토리 목록 오버레이 표시
- 해당 action에 적합한 아이템만 하이라이트
- letter 입력 시 해당 아이템으로 action 수행
- `Esc` 입력 시 `Playing`으로 복귀

**17.7 `MorePrompt` 상태 처리**

`RunState::MorePrompt` 상태에서는:
- 화면 하단에 "--More--" 표시
- 아무 키 입력 시 `AcknowledgeMore` 제출

#### 검증 명령

```bash
cargo test --test ui_runtime_smoke
cargo test --test ui_layout
cargo run --bin aihack -- --seed 42
```

수동 검증:
- 80x28 터미널에서 Title/Character Creation/Playing/Game Over 화면 확인
- Game Over 화면에서 N/Q/E 입력 동작 확인
- Debug Observation F9 토글 동작 확인

#### 완료 기준

- `tests/ui_screens.rs` (신규): Title/Character Creation/Game Over 렌더링 검증
- `tests/ui_input_mapping.rs`: 화면 전환 키 입력 검증 추가
- headless baseline hash 동일 유지 (UI-only 변경이므로 hash에 영향 없음)

---

### Phase 18: Debug Observation Panel (F9 Toggle)

#### 목표

designs.md 10의 Debug Observation 패널을 F9 키로 토글 가능하게 구현하고, 전체 Observation 데이터를 표시한다.

#### 구현 항목

**18.1 Debug 패널 상태 추가** (`src/ui/tui/mod.rs`의 `TuiApp`)

```rust
pub struct TuiApp {
    // ... existing fields ...
    pub debug_observation_visible: bool,
}
```

**18.2 F9 키 매핑** (`src/ui/tui/input.rs`)

```rust
KeyCode::F(9) => Some(UiCommandCandidate::ToggleDebugObservation),
```

**18.3 `UiCommandCandidate::ToggleDebugObservation` 처리** (`src/ui/tui/mod.rs`)

```rust
UiCommandCandidate::ToggleDebugObservation => {
    self.debug_observation_visible = !self.debug_observation_visible;
    Ok(false) // no turn advance
}
```

**18.4 Debug Observation 패널 렌더링** (`src/ui/tui/render_panels.rs`)

표시 항목 (designs.md 10 기준):

```rust
pub fn debug_observation_lines(observation: &Observation) -> Vec<String> {
    vec![
        format!("schema_version: {}", observation.schema_version),
        format!("seed: {}", observation.seed),
        format!("turn: {}", observation.turn),
        format!("snapshot_hash: {:016x}", observation.snapshot_hash),
        format!("run_state: {:?}", observation.run_state),
        format!("player_pos: ({}, {})", observation.player_pos.x, observation.player_pos.y),
        format!("player_hp: {}/{}", observation.player.hp, observation.player.max_hp),
        format!("hunger: {}", observation.player.hunger),
        format!("luck: {}", observation.player.luck),
        format!("prayer_cooldown: {}", observation.player.prayer_cooldown),
        format!("paralysis_turns: {}", observation.player.paralysis_turns),
        format!("hallucinating: {}", observation.player.hallucinating),
        format!("visible_tiles: {}", observation.visible_tiles.len()),
        format!("visible_entities: {}", observation.visible_entities.len()),
        format!("inventory: {} items", observation.inventory.len()),
        format!("action_space: {} actions", observation.action_space.len()),
        format!("last_events: {} events", observation.last_events.len()),
        "legal_actions:".to_string(),
    ]
    .into_iter()
    .chain(
        observation.legal_actions
            .iter()
            .take(20)
            .map(|action| format!("  {:?}", action))
    )
    .chain(vec![
        "last 10 events:".to_string(),
    ])
    .chain(
        observation.last_events
            .iter()
            .rev()
            .take(10)
            .rev()
            .map(|event| format!("  {:?}", event))
    )
    .collect()
}
```

**18.5 Debug 패널 표시 조건**

- `debug_observation_visible == true`일 때 표시
- 기존 `layout.debug` (roomy layout, 120x36+)와 별개로 동작
- 80x28에서도 overlay 형태로 우측/하단에 표시 가능해야 함
- 다른 패널과 겹치지 않도록 작은 크기(최소 40x15)로 렌더링

**18.6 `src/ui/debug/` 모듈 분리 (선택적, C급)**

`src/ui/debug/mod.rs`를 생성하고 debug observation 관련 코드를 분리한다. TUI의 debug 기능을 별도 모듈로 격리한다.

```text
src/ui/debug/
├── mod.rs          # DebugPanel 상태와 토글
├── render.rs       # debug observation lines 생성
└── input.rs        # F9 키 매핑
```

#### 검증 명령

```bash
cargo test --test ui_runtime_smoke
cargo test --test ui_layout
```

#### 완료 기준

- `tests/ui_debug.rs` (신규): F9 토글, debug observation lines 생성 검증
- `tests/ui_layout.rs`: debug 패널 표시 시 layout 겹침 없음 검증

---

### Phase 19: Auto-Label Priority System

#### 목표

spec.md 15.7의 Phase 10E 후속 "자동 라벨 우선순위"를 구현한다.

#### 구현 항목

**19.1 라벨 타입 정의** (`src/ui/tui/mod.rs` 또는 `src/ui/tui/labels.rs` 신규)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelKind {
    HostileAdjacent,
    LowHpWarning,
    Stairs,
    UnidentifiedItem,
    PassiveMonster,
}

#[derive(Debug, Clone)]
pub struct AutoLabel {
    pub kind: LabelKind,
    pub pos: Pos,
    pub text: String,
    pub created_at_ms: u64,
    pub duration_ms: u16,
}
```

**19.2 라벨 우선순위** (spec 15.4)

우선순위 (높은 것부터):
1. `HostileAdjacent` (적대 몬스터 인접)
2. `LowHpWarning` (HP 30% 이하)
3. `Stairs` (계단)
4. `UnidentifiedItem` (미식별 아이템)
5. `PassiveMonster` (비적대 몬스터)

최대 동시 표시: 3개
지속 시간: 1200ms (spec 15.6 `new_entity_label_ms`)

**19.3 라벨 수집 로직** (`src/ui/tui/mod.rs`)

매 턴 `Observation` 기준으로 라벨을 수집한다:

```rust
fn collect_auto_labels(observation: &Observation, current_time_ms: u64) -> Vec<AutoLabel> {
    let mut candidates = Vec::new();
    
    for entity in &observation.visible_entities {
        let distance = manhattan_distance(observation.player_pos, entity.pos);
        match entity.kind {
            EntityKind::Monster(MonsterKind::Goblin) | EntityKind::Monster(MonsterKind::Jackal) 
                if distance == 1 => {
                candidates.push(AutoLabel {
                    kind: LabelKind::HostileAdjacent,
                    pos: entity.pos,
                    text: format!("{}", entity.kind.glyph()),
                    created_at_ms: current_time_ms,
                    duration_ms: 1200,
                });
            }
            EntityKind::Monster(MonsterKind::FloatingEye) if distance == 1 => {
                candidates.push(AutoLabel {
                    kind: LabelKind::PassiveMonster,
                    pos: entity.pos,
                    text: "floating eye".to_string(),
                    created_at_ms: current_time_ms,
                    duration_ms: 1200,
                });
            }
            _ => {}
        }
    }
    
    // HP 30% 이하 체크
    if observation.player.hp <= observation.player.max_hp * 3 / 10 {
        candidates.push(AutoLabel {
            kind: LabelKind::LowHpWarning,
            pos: observation.player_pos,
            text: "LOW HP!".to_string(),
            created_at_ms: current_time_ms,
            duration_ms: 1600, // spec 15.6 danger_label_ms
        });
    }
    
    // 계단 체크
    for tile in &observation.visible_tiles {
        if matches!(tile.kind, TileKind::StairsDown | TileKind::StairsUp) {
            candidates.push(AutoLabel {
                kind: LabelKind::Stairs,
                pos: tile.pos,
                text: ">".to_string(),
                created_at_ms: current_time_ms,
                duration_ms: 1200,
            });
        }
    }
    
    // 미식별 아이템 체크
    for item in &observation.inventory {
        if !item.identified {
            candidates.push(AutoLabel {
                kind: LabelKind::UnidentifiedItem,
                pos: observation.player_pos, // inventory item은 위치 없음, player 위치에 표시
                text: format!("{} unknown", item.letter),
                created_at_ms: current_time_ms,
                duration_ms: 1200,
            });
        }
    }
    
    // 우선순위 정렬 후 상위 3개 선택
    candidates.sort_by_key(|l| l.kind.priority());
    candidates.into_iter().take(3).collect()
}
```

**19.4 라벨 렌더링** (`src/ui/tui/render_map.rs`)

맵 위에 라벨 텍스트를 overlay로 렌더링한다. 라벨 텍스트는 맵 셀 우측/상단에 `[label text]` 형태로 표시한다.

**19.5 `UiEffectEvent::NewEntityLabel` 통합** (`src/ui/tui/effects.rs`)

`GameEvent::EntityMoved` 또는 `TurnStarted`에서 새로운 visible entity 발견 시 `NewEntityLabel` effect를 생성한다.

#### 검증 명령

```bash
cargo test --test ui_effect_projection
cargo test --test ui_layout
```

#### 완료 기준

- `tests/ui_labels.rs` (신규): 라벨 수집, 우선순위, 최대 3개 제한, 지속 시간 검증
- `tests/ui_effect_projection.rs`: NewEntityLabel effect 생성 검증
- headless baseline hash 동일 유지 (라벨은 UI-only)

---

### Phase 20: 데이터 외부화 및 모듈 분리

#### 목표

spec.md 7의 예정 구조와 실제 구조를 일치시킨다. `src/domain/status.rs`, `src/data/`, `src/ui/debug/`를 생성한다.

#### 구현 항목

**20.1 `src/domain/status.rs` 생성**

현재 `GameWorld`에 산재한 상태 필드를 `Status` struct로 분리한다.

```rust
// src/domain/status.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub nutrition: i32,
    pub luck: i16,
    pub prayer_cooldown: u16,
    pub paralysis_turns: u8,
    pub hallucinating: bool,
    pub identified_items: Vec<String>, // item id strings
    pub gold: i32,
    pub kill_count: u32,
}

impl Status {
    pub fn default_adventurer() -> Self {
        Self {
            nutrition: 900,      // spec 10.1 기준 ration 800 + 여유
            luck: 0,
            prayer_cooldown: 0,
            paralysis_turns: 0,
            hallucinating: false,
            identified_items: Vec::new(),
            gold: 0,
            kill_count: 0,
        }
    }
    
    pub fn hunger_state(&self) -> HungerState {
        match self.nutrition {
            0..=150 => HungerState::Fainting,
            151..=300 => HungerState::Weak,
            301..=500 => HungerState::Hungry,
            501..=2000 => HungerState::Satiated,
            _ => HungerState::Oversatiated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HungerState {
    Fainting,
    Weak,
    Hungry,
    Satiated,
    Oversatiated,
}
```

`GameWorld`에서 해당 필드를 제거하고 `status: Status` 필드로 대체한다.

**20.2 `src/data/items.toml` 생성**

```toml
[[item]]
id = "item.weapon.dagger"
kind = "weapon"
glyph = ")"
weight = 10
slot = "melee"
hit_bonus = 1
damage = "1d4"
base_price = 4

[[item]]
id = "item.food.ration"
kind = "food"
glyph = "%"
weight = 20
nutrition = 800
base_price = 45

[[item]]
id = "item.potion.healing"
kind = "potion"
glyph = "!"
weight = 20
effect = "heal_1d8_plus_4"
base_price = 100

[[item]]
id = "item.wand.magic_missile"
kind = "wand"
glyph = "/"
weight = 7
charges = 4
effect = "magic_missile"
base_price = 150

[[item]]
id = "item.scroll.identify"
kind = "scroll"
glyph = "?"
weight = 5
effect = "identify"
base_price = 20

[[item]]
id = "item.scroll.reveal"
kind = "scroll"
glyph = "?"
weight = 5
effect = "reveal"
base_price = 20

[[item]]
id = "item.scroll.teleport"
kind = "scroll"
glyph = "?"
weight = 5
effect = "teleport"
base_price = 20

[[item]]
id = "item.armor.leather"
kind = "armor"
glyph = "["
weight = 40
slot = "body"
ac_bonus = 1
base_price = 5

[[item]]
id = "item.weapon.rock"
kind = "weapon"
glyph = "*"
weight = 10
slot = "melee"
hit_bonus = 0
damage = "1d3"
base_price = 0
```

**20.3 `src/data/monsters.toml` 생성**

```toml
[[monster]]
id = "monster.jackal"
glyph = "d"
hp = 4
ac = 0
hit_bonus = 0
damage = "1d2"
ai = "wander"
speed = 12
difficulty = 1

[[monster]]
id = "monster.goblin"
glyph = "g"
hp = 6
ac = 1
hit_bonus = 1
damage = "1d4"
ai = "chase_on_sight"
speed = 12
difficulty = 2

[[monster]]
id = "monster.floating_eye"
glyph = "e"
hp = 8
ac = 2
hit_bonus = 0
damage = "0"
ai = "stationary"
passive = "paralyze_on_melee"
speed = 0
difficulty = 3
```

**20.4 `src/data/levels/` 생성**

```toml
# src/data/levels/main_1.toml
level_id = "main:1"
branch = "Main"
depth = 1
width = 40
height = 20
player_start = [5, 5]
stairs_down = [34, 15]

[[wall]]
x = 12
y_range = [4, 8]

[[door]]
pos = [10, 5]
state = "closed"

[[door]]
pos = [14, 5]
state = "closed"

[[hidden_door]]
pos = [20, 5]
tile = "wall"

[[hidden_trap]]
pos = [15, 10]
trap = "pit"
tile = "floor"

[[monster]]
id = "monster.jackal"
pos = [6, 5]

[[monster]]
id = "monster.goblin"
pos = [20, 12]

[[item]]
id = "item.potion.healing"
pos = [8, 5]
```

**20.5 데이터 로더 구현**

`src/data/mod.rs`에 TOML 파서를 구현한다. `toml` crate를 `Cargo.toml` dependencies에 추가한다.

```rust
// src/data/mod.rs
pub fn load_items() -> Vec<ItemData> { ... }
pub fn load_monsters() -> Vec<MonsterData> { ... }
pub fn load_level(level_id: &str) -> LevelData { ... }
```

**20.6 `src/ui/debug/` 모듈 생성**

```text
src/ui/debug/
├── mod.rs          # DebugPanel, ToggleDebugObservation candidate
├── render.rs       # debug_observation_lines
└── input.rs        # F9 key mapping
```

#### 검증 명령

```bash
cargo test --test data_loading
cargo test --test status
cargo test
```

#### 완료 기준

- `tests/data_loading.rs` (신규): TOML 파일 로드, 파싱, ItemData/MonsterData/LevelData 검증
- `tests/status.rs` (신규): Status struct, hunger state 계산 검증
- `tests/save_load.rs`: Status save/load roundtrip 검증
- headless baseline hash 동일 유지 (GameWorld 필드 재배치는 내부 구조 변경, hash 입력 동일)

## 7. 핵심 타입 계약 변경 요약

### 7.1 RunState (src/core/session.rs)

```rust
// BEFORE
pub enum RunState {
    Playing,
    GameOver,
}

// AFTER
pub enum RunState {
    Title,
    CharacterCreation,
    Playing,
    AwaitingDirection { action: DirectionalAction },
    AwaitingInventorySelection { action: InventoryAction },
    MorePrompt,
    GameOver { cause: DeathCause, final_score: i32 },
}
```

### 7.2 CommandIntent (src/core/action.rs)

```rust
// 추가
AcknowledgeMore,

// spec.md 8.3에 Pray 반영 (이미 구현됨, 문서만 갱신)
```

### 7.3 DirectionalAction (src/core/action.rs, 신규)

```rust
pub enum DirectionalAction {
    Open,
    Close,
    Kick,
}
```

### 7.4 InventoryAction (src/core/action.rs, 신규)

```rust
pub enum InventoryAction {
    Drop,
    Wield,
    Wear,
    Quaff,
    Read,
}
```

### 7.5 GameWorld 변경 (src/core/world.rs)

```rust
// BEFORE: 개별 필드
pub nutrition: i32,
pub luck: i16,
pub prayer_cooldown: u16,
pub paralysis_turns: u8,
pub hallucinating: bool,
pub kill_count: u32,
pub gold: i32,

// AFTER: Status struct로 통합
pub status: Status,
pub last_death_cause: Option<DeathCause>,
```

### 7.6 UiCommandCandidate (src/ui/tui/input.rs)

```rust
// 추가
ToggleDebugObservation,
```

### 7.7 GameMeta (src/core/session.rs)

```rust
// seed 필드 유지, replay 파일 경로 추가 (선택적)
pub replay_path: Option<String>,
```

## 8. 실데이터 기준

### 8.1 Title 화면 렌더링 데이터

```text
terminal: 80x28

+--------------------------------------------------------------------------+
|                                                                          |
|                               AIHack                                     |
|                                                                          |
|                        Press Enter to Start                              |
|                                                                          |
|                         L - Load Game                                     |
|                         Q - Quit                                        |
|                                                                          |
+--------------------------------------------------------------------------+
```

### 8.2 Game Over 화면 렌더링 데이터

```text
scenario: player died to goblin at turn 42, depth 1, kill_count 3, score 150, seed 42
terminal: 80x28

+--------------------------------------------------------------------------+
|                              GAME OVER                                   |
|                        Killed by a goblin                              |
|                                                                          |
|  Turn:              42              Depth:              1               |
|  Defeated:          3               Score:              150             |
|  Seed:              42                                                   |
|                                                                          |
|                    [N] New Run    [Q] Quit                               |
+--------------------------------------------------------------------------+
```

### 8.3 Debug Observation 패널 데이터

```text
schema_version: 1
seed: 42
turn: 42
snapshot_hash: 4c77dafb19dd2226
run_state: Playing
player_pos: (5, 5)
player_hp: 12/16
hunger: 850
luck: 0
prayer_cooldown: 0
paralysis_turns: 0
hallucinating: false
visible_tiles: 145
visible_entities: 2
inventory: 4 items
action_space: 27 actions
legal_actions:
  Command(Move(North))
  Command(Move(South))
  ... (최대 20개)
last 10 events:
  TurnStarted { turn: 42 }
  EntityMoved { entity: EntityId(1), from: Pos(5, 4), to: Pos(5, 5) }
  ...
```

### 8.4 Auto-Label 우선순위 예시

```text
상황: player (5,5), goblin (5,6), stairs_down (34,15), unidentified potion in inventory

수집된 라벨 (우선순위 순):
1. [HostileAdjacent] goblin @ (5,6)
2. [Stairs] > @ (34,15)
3. [UnidentifiedItem] c unknown @ player

표시: goblin 셀 옆에 "[g]", stairs 셀 옆에 "[>]"
```

## 9. 구현 순서와 의존성

```text
Phase 16 (RunState & CommandIntent)
  ├── src/core/action.rs: DirectionalAction, InventoryAction, AcknowledgeMore 추가
  ├── src/core/session.rs: RunState 확장, submit() 분기 수정
  ├── src/core/observation.rs: RunStateSummary 확장, legal_actions 수정
  ├── src/core/world.rs: last_death_cause 추가, status 필드 통합 (선택적)
  └── tests/observation.rs, tests/action_space.rs: 새 상태 검증 추가

Phase 17 (Game Flow Screens)
  ├── src/ui/tui/mod.rs: 화면 분기, Title/Character Creation/Game Over 렌더러
  ├── src/ui/tui/input.rs: 화면 전환 키 매핑
  ├── src/ui/tui/render_panels.rs: Game Over lines, character creation lines
  └── tests/ui_screens.rs (신규): 화면 렌더링 검증

Phase 18 (Debug Observation)
  ├── src/ui/tui/mod.rs: debug_observation_visible 상태, ToggleDebugObservation 처리
  ├── src/ui/tui/input.rs: F9 키 매핑
  ├── src/ui/tui/render_panels.rs: debug_observation_lines
  ├── src/ui/debug/ (선택적): 모듈 분리
  └── tests/ui_debug.rs (신규): F9 토글, debug lines 검증

Phase 19 (Auto-Label)
  ├── src/ui/tui/labels.rs (신규): AutoLabel, LabelKind, collect_auto_labels
  ├── src/ui/tui/render_map.rs: 라벨 overlay 렌더링
  ├── src/ui/tui/effects.rs: NewEntityLabel effect
  └── tests/ui_labels.rs (신규): 라벨 수집/우선순위/지속시간 검증

Phase 20 (데이터 외부화)
  ├── src/domain/status.rs (신규): Status, HungerState
  ├── src/core/world.rs: status 필드 통합
  ├── src/data/ (신규): items.toml, monsters.toml, levels/, mod.rs 로더
  ├── Cargo.toml: toml crate 추가
  ├── src/ui/debug/ (선택적): 모듈 분리
  └── tests/data_loading.rs, tests/status.rs (신규)
```

## 10. 검증 명령

### 10.1 Phase별 검증

```bash
# Phase 16
cargo test --test observation --test action_space
cargo run --bin aihack-headless -- --seed 42 --turns 100

# Phase 17
cargo test --test ui_runtime_smoke --test ui_layout --test ui_input_mapping
cargo run --bin aihack -- --seed 42

# Phase 18
cargo test --test ui_debug --test ui_layout

# Phase 19
cargo test --test ui_labels --test ui_effect_projection --test ui_layout

# Phase 20
cargo test --test data_loading --test status --test save_load
cargo run --bin aihack-headless -- --seed 42 --turns 1000
```

### 10.2 전체 회귀 검증

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin aihack-headless -- --seed 42 --turns 1000
cargo run --bin aihack-headless -- --seed 7 --turns 1000
cargo run --bin aihack-headless -- --seed 1234 --turns 1000
```

### 10.3 headless baseline 확인

Phase 20 완료 후 기준 hash (Phase 9와 동일해야 함):

```text
seed=42 turns=100 final_turn=20 final_hash=4c77dafb19dd2226
seed=43 turns=100 final_turn=21 final_hash=f8324eacbce50087
```

만약 GameWorld 필드 재배치(status 통합)로 hash가 변경되면, 이 문서를 갱신하고 `DESIGN_DECISIONS.md`에 ADR을 추가한다.

## 11. 문서 갱신 계획

### 11.1 spec.md 갱신 항목

- 8.2 RunState: Title, CharacterCreation, AwaitingDirection, AwaitingInventorySelection, MorePrompt 추가
- 8.2 GameOver: `cause: DeathCause, final_score: i32` 필드 추가
- 8.3 CommandIntent: `AcknowledgeMore` 추가, `Pray` 추가 (이미 구현됨)
- 8.3 DirectionalAction, InventoryAction 신규 추가
- 12.1~12.3 Phase 완료 메모: RunState/CommandIntent 변경 사실 기록
- 12.12 Phase 12, 12.13 Phase 13: 기존 완료 유지

### 11.2 designs.md 갱신 항목

- 2 ScreenId: Title, CharacterCreation 구현 완료로 표시
- 10 Debug Observation: F9 토글, 표시 항목 구현 완료로 표시
- 11 Game Over: 8개 필수 표시 + 3개 버튼 구현 완료로 표시
- 15.7 Phase 10E: auto-label priority 구현 완료로 표시 (Phase 19 완료 후)

### 11.3 audit_roadmap.md 갱신 항목

- Phase 16~20 추가
- 각 Phase별 목표, 구현 항목, 검증 명령, 완료 기준 추가
- v0.2 릴리즈 게이트에 Phase 16~17 포함

### 11.4 implementation_summary.md 갱신 항목

- 파일 책임표에 새 파일 추가
- 시스템 순서에 Title/CharacterCreation/MorePrompt 상태 전이 추가
- 구현 순서에 Phase 16~20 추가

### 11.5 CHANGELOG.md 갱신 항목

Phase 16~20 완료 시 각각 항목 추가:

```
## 2026-05-18 (또는 완료일)

### Added
- Phase 16 RunState & CommandIntent 계약 정렬: Title, CharacterCreation, AwaitingDirection, AwaitingInventorySelection, MorePrompt, GameOver with cause/final_score
- Phase 17 Game Flow Screens: Title, Character Creation, Game Over 화면 구현
- Phase 18 Debug Observation: F9 토글, 전체 Observation 데이터 표시
- Phase 19 Auto-Label Priority: hostile/item/stairs 자동 라벨, 우선순위, 최대 3개
- Phase 20 데이터 외부화: items.toml, monsters.toml, levels/, status.rs
```

### 11.6 DESIGN_DECISIONS.md 갱신 항목

ADR-0017: RunState 확장 결정 (왜 Title/CharacterCreation을 추가했는지)
ADR-0018: GameOver 필드 추가 결정 (cause, final_score)
ADR-0019: 데이터 외부화 결정 (TOML vs 하드코딩)
ADR-0020: Status 모듈 분리 결정 (GameWorld 필드 재배치)

## 12. 잔여 리스크

| 리스크 | 영향 | 완화 |
|---|---|---|
| RunState 확장으로 인한 snapshot hash 변경 | 높음 | GameWorld 필드 변경 시 hash 재계산, 기준값 갱신 |
| TUI 화면 분기로 인한 layout 복잡도 증가 | 중간 | 각 화면별 별도 layout 함수 사용, 기존 play layout 유지 |
| 데이터 외부화(toml)로 인한 빌드 의존성 증가 | 낮음 | `toml` crate는 경량, 하드코딩 fallback 유지 가능 |
| Auto-label이 80x28 화면에서 정보 과잉 | 중간 | 최대 3개 제한, degrade layout에서 라벨 생략 |
| Game Over 화면에서 New Run 시 RNG seed 처리 | 낮음 | 새 seed 생성 또는 이전 seed 재사용, 문서에 명시 |

## 13. 수동 검증 체크리스트

Phase 16~20 전체 완료 후 반드시 수동으로 확인할 항목:

- [ ] `cargo run --bin aihack -- --seed 42` 실행 후 Title 화면 확인
- [ ] Title에서 Enter 입력 -> Character Creation 화면 확인
- [ ] Character Creation에서 Enter 입력 -> Playing 화면 확인
- [ ] Playing에서 플레이어 사망 -> Game Over 화면 확인
- [ ] Game Over에서 사망 원인, turn, depth, defeated, score, seed 표시 확인
- [ ] Game Over에서 N 입력 -> Title 화면으로 복귀 확인
- [ ] Game Over에서 Q 입력 -> TUI 종료 확인
- [ ] Playing 상태에서 F9 입력 -> Debug Observation 패널 토글 확인
- [ ] Debug 패널에서 schema_version, seed, turn, snapshot_hash, legal_actions 확인
- [ ] 80x28, 100x32, 120x36 터미널에서 각 화면 텍스트 겹침 없음 확인
- [ ] mouse disabled 환경에서도 모든 화면 전환 키보드로 가능 확인
- [ ] headless `seed=42 turns=1000`이 Phase 9와 동일 hash 출력 확인
