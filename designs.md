# AIHack Design Specification

문서 상태: active
작성일: 2026-04-28

## 1. 핵심 경험

AIHack의 화면 경험은 "고전 ASCII 로그라이크를 현대적 디버깅/AI 관찰 도구와 함께 플레이한다"이다.

기본 플레이어는 키보드 중심으로 게임을 한다. 개발자와 AI 연구자는 같은 화면에서 현재 관찰 상태, 이벤트 로그, legal action을 확인할 수 있어야 한다.

## 2. 화면 모드

| ScreenId | 목적 | 입력 | 출력 |
| --- | --- | --- | --- |
| `screen.title` | 시작/로드/설정 | Enter, L, Q | `RunState::Title` |
| `screen.character_creation` | v0.1 기본 캐릭터 확정 | Enter, Esc | `RunState::CharacterCreation` |
| `screen.play` | 핵심 게임 화면 | command key | `CommandIntent` |
| `screen.inventory` | 아이템 선택 | letter, Esc | Phase 4 현재 `Wield`/`Quaff`; `Wear`/`Read`/`Drop`은 후속 범위 |
| `screen.game_over` | 사망/점수/재시작 | N, Q | 새 세션 또는 종료 |
| `screen.debug_observation` | AI 관찰 디버그 | F9 toggle | read-only |

## 3. Play 화면 레이아웃

초기 TUI 기준:

```text
+------------------------------------------------------------+
| Dungeon Map 40x20                       | Player/Status    |
|                                         | HP 16/16          |
| @....                                   | AC 0              |
|                                         | Turn 42           |
|                                         | Depth 1           |
|                                         +------------------+
|                                         | Visible Entities  |
|                                         | g goblin 12,5     |
+-----------------------------------------+------------------+
| Message Log                                                |
| You hit the goblin.                                        |
+------------------------------------------------------------+
| Command Hint / More Prompt                                 |
+------------------------------------------------------------+
```

최소 터미널 크기:

- width: 80
- height: 28

지도 viewport:

- v0.1 고정 40x20
- 플레이어 중심 scrolling은 v0.2
- 지도 glyph는 `TileObservation`과 `EntityObservation`에서만 읽는다.

## 4. 컬러 토큰

| Token | RGB | 용도 |
| --- | --- | --- |
| `color.bg` | `#0b0d0e` | 전체 배경 |
| `color.panel` | `#15191c` | 상태/로그 패널 |
| `color.text` | `#d8d5c8` | 기본 텍스트 |
| `color.dim` | `#7d8178` | 비활성/기억 타일 |
| `color.player` | `#f5f1d0` | 플레이어 |
| `color.monster.hostile` | `#d75f5f` | 적대 몬스터 |
| `color.item` | `#d7b95f` | 아이템 |
| `color.wall` | `#8a8f98` | 벽 |
| `color.floor` | `#4f5458` | 바닥 |
| `color.warning` | `#e0a84f` | 경고 |
| `color.danger` | `#ff5f5f` | 치명 상태 |

## 5. 타이포그래피

TUI:

- monospace only
- negative letter spacing 금지
- map glyph cell은 동일 폭/높이 유지

GUI adapter가 생길 경우:

- map은 monospace grid
- panel text는 13-15px
- 버튼은 기능 텍스트 + 단축키 표시

## 6. 입력 정책

기본 키:

| Key | CommandIntent |
| --- | --- |
| `h`/Left | `Move(West)` |
| `j`/Down | `Move(South)` |
| `k`/Up | `Move(North)` |
| `l`/Right | `Move(East)` |
| `y` | `Move(NorthWest)` |
| `u` | `Move(NorthEast)` |
| `b` | `Move(SouthWest)` |
| `n` | `Move(SouthEast)` |
| `.` | `Wait` |
| `o` then dir | `Open(dir)` |
| `c` then dir | `Close(dir)` |
| `,` | `Pickup` |
| `i` | `ShowInventory` |
| `>` | `Descend` |
| `<` | `Ascend` |
| `s` | 후속 Phase `Search` |
| `q` in inventory | `Quaff(item)` if potion |
| `w` in inventory | `Wield(item)` if weapon |
| `W` in inventory | 후속 Phase `Wear(item)` if armor |
| `Esc` | cancel current selection |

입력은 UI 내부 상태를 직접 바꾸지 않고 `CommandIntent`를 생성한다. 방향 대기 상태는 후속 UI Phase에서 core 상태와 동기화한다.

## 7. HUD 데이터 연결

| UI 요소 | 데이터 소스 | 갱신 시점 |
| --- | --- | --- |
| HP | `GameSnapshot.player.hp` | 매 `TurnOutcome` |
| AC | `GameSnapshot.player.ac` | 장비/상태 변경 후 |
| Turn | `GameSnapshot.turn` | 턴 진행 후 |
| Depth | `GameSnapshot.current_level.depth` | 레벨 변경 후 |
| Visible Entities | `Observation.visible_entities` | 매 렌더 |
| Inventory | `Observation.inventory` | 인벤토리 열 때 |
| Legal Actions | `Observation.legal_actions` | debug panel |

## 7.1 Phase 5 계단/레벨 상태 구조

```text
GameWorld
├── levels: LevelRegistry
│   ├── main:1 GameMap (stairs down 34,15)
│   └── main:2 GameMap (stairs up 5,5)
├── current_level: LevelId
├── entities: EntityStore
│   ├── Actor location: OnMap { level, pos }
│   └── Item location: OnMap { level, pos } | Inventory | Consumed
└── inventory: Inventory
```

UI는 stairs tile에서 `Observation.legal_actions`에 포함된 `Descend`/`Ascend`만 표시한다. `Move` 입력은 계단 타일 진입까지만 처리하고 층 이동은 하지 않는다.

## 8. Message Log 정책

로그는 `GameEvent::Message`와 event formatter에서만 생성한다.

표시 규칙:

- 최근 5줄 표시
- priority `High` 이상은 `color.warning`
- `Death` 관련 메시지는 `color.danger`
- 같은 턴 동일 텍스트는 1회로 압축하고 `(xN)` 표기
- More prompt 기준: 한 턴에 5개 초과 메시지가 발생하면 `RunState::MorePrompt`

## 9. Inventory 화면

표시 열:

```text
a - dagger (wielded)
b - food ration
c - potion of healing
```

아이템 상태 suffix:

- `(wielded)`
- `(worn)`
- `(quivered)`
- `(cursed)`는 식별된 경우만 표시
- `(unknown)`은 미식별 potion/scroll/wand에 표시

선택 정책:

- 선택 불가능한 아이템은 dim 처리
- 잘못된 선택은 `CommandRejected`를 만들고 턴을 진행하지 않음
- inventory letter는 save/load 후 유지

## 10. Debug Observation 패널

F9로 토글한다. 플레이에는 영향을 주지 않는다.

표시 항목:

- `schema_version`
- `seed`
- `turn`
- `snapshot_hash`
- `legal_actions` 최대 20개
- 최근 event 10개
- AI가 보는 visible tile/entity 수

이 패널은 AI 통합 전에도 구현한다. Observation이 UI와 테스트 양쪽에서 같은 값을 보여야 하기 때문이다.

## 11. Game Over 화면

필수 표시:

- 사망 원인
- turn
- depth
- defeated monsters count
- score
- replay seed
- New Run
- Quit

버튼 정책:

| CTA | 활성 조건 | 결과 |
| --- | --- | --- |
| `New Run` | 항상 | 새 `GameSession` 생성 |
| `Quit` | 항상 | 프로세스 종료 또는 title |
| `Export Replay` | replay log 있음 | `runtime/replays` 경로 출력 |

## 12. UI/Core 경계

UI가 호출 가능한 core API:

```rust
pub trait GameClient {
    fn snapshot(&self) -> GameSnapshot;
    fn observation(&self) -> Observation;
    fn submit(&mut self, intent: CommandIntent) -> TurnOutcome;
}
```

UI 금지 사항:

- entity vector 직접 수정
- map tile 직접 수정
- RNG 직접 호출
- event log 직접 push
- save 파일 직접 serialize

## 13. AI 디자인 경계

AI용 표현은 사람 UI와 다르다. AI는 glyph가 아니라 typed observation을 우선 사용한다.

AI observation에서 위치는 절대 좌표와 플레이어 상대 좌표를 모두 제공한다.

```rust
pub struct EntityObservation {
    pub id: EntityId,
    pub kind: EntityKind,
    pub name: String,
    pub pos: Pos,
    pub rel: Delta,
    pub hp_band: HpBand,
    pub attitude: Attitude,
}
```

`hp_band`는 정확한 HP 은닉을 위해 `Healthy`, `Wounded`, `Critical`, `Unknown` 중 하나다. 플레이어 본인 HP는 정확히 제공한다.

## 14. 완료 기준

v0.1 UI 완료:

- 80x28 터미널에서 텍스트 겹침 없음
- Play 화면에서 map/status/log가 동시에 표시
- inventory letter 선택이 core command로 연결
- F9 debug observation이 core observation과 동일
- Game Over 화면에서 seed와 turn 표시
- UI 없이도 headless 테스트 통과

## 15. 현대 TUI/UX 리팩토링 선반영

2026-04-28 deep-interview 결정에 따라 디자인 목표는 `단계형 혼합 로드맵`으로 고정한다. 첫 체감 개선 우선순위는 마우스 자체가 아니라 **정보 가독성**이다.

### 15.1 현대화 핵심 경험

플레이어는 키보드만으로 빠르게 플레이할 수 있고, 마우스로는 같은 정보를 더 쉽게 발견하고 선택할 수 있어야 한다. 화면 효과는 게임 상태를 바꾸지 않는 presentation layer이며, 전투/위험/획득/문 상태 변화의 인지를 돕는 보조 채널이다.

### 15.2 현대화 후 화면 구조

```text
+--------------------------------------------------------------------------------+
| Dungeon Map / Labels / Hover Inspect                         | Status + Alerts  |
| @..g.        [g goblin]                                      | HP 12/16 ALERT   |
| ....!        [! potion]                                      | AC 0  Turn 42    |
| ....#                                                         | Depth 1 Seed 42  |
|                                                                +----------------+
|                                                                | Inspect / Hints |
| Hover: goblin, hostile, wounded                               | l-click inspect |
| Path: 5 steps, safe known tiles                               | r-click command |
+---------------------------------------------------------------+----------------+
| Message Log: priority color, duplicate compression, category markers            |
| ! The goblin hits you.  -4 HP                                                 |
+--------------------------------------------------------------------------------+
| Command Bar: [i] Inventory [.] Wait [o] Open [F9] Observation [?/hover] Help   |
+--------------------------------------------------------------------------------+
```

레이어 순서:

1. `MapGlyphLayer`: tile/entity glyph.
2. `MemoryLayer`: visible이 아닌 기억 타일 dim 처리.
3. `LabelLayer`: 새 hostile/item/danger 라벨 최대 3개.
4. `EffectLayer`: damage/heal/miss/door/item flash.
5. `CursorHoverLayer`: hover 대상 outline 또는 inverse style.
6. `PanelLayer`: status, inspect, log, command bar.

### 15.3 마우스 정책

| 입력 | v0.1 | v0.2 | v0.3 후보 |
| --- | --- | --- | --- |
| left click map | inspect 또는 adjacent move 후보 표시 | reachable tile 이동 intent 후보 | path preview + confirm 정책 |
| right click map | command menu 후보 표시 | open/close/search/context action | 상황별 radial/text menu |
| hover map | 없음 | tile/entity read-only inspect | threat/path/item 비교 tooltip |
| click inventory row | 선택 | use/wield/wear/drop action 후보 | drag/drop reorder/drop |
| click panel tab | F9/debug/status/log focus | panel pin/filter | 사용자 레이아웃 저장 |

마우스 입력은 항상 `UiInputEvent -> UiCommandCandidate -> CommandIntent` 변환을 거친다. 지도 좌표 변환은 `Viewport { origin: Pos, width, height, cell_width: 1, cell_height: 1 }` 계약을 사용한다.

### 15.4 정보 가독성 규칙

- hostile, low HP, unknown item, stairs, closed door는 색상만으로 구분하지 않고 glyph/style/text 중 2개 이상 채널을 사용한다.
- `MessagePriority::High` 이상은 로그와 status alert 양쪽에 표시한다.
- 새로 시야에 들어온 hostile/item 라벨은 최대 3개만 표시한다.
- 라벨 우선순위: `hostile adjacent > low HP warning > stairs > unidentified item > passive monster`.
- hover inspect는 항상 턴 비진행이며 `Observation` 또는 `GameSnapshot`의 read-only 데이터만 사용한다.

### 15.5 ASCII 애니메이션/효과 규칙

| 효과 | 트리거 `GameEvent` | duration | 표시 | 비고 |
| --- | --- | --- | --- | --- |
| DamageFlash | `AttackResolved { hit: true }` | 120ms | defender cell red/inverse flash + `-N` | replay hash 제외 |
| MissWisp | `AttackResolved { hit: false }` | 80ms | defender 주변 dim `*` 1 frame | reduced motion에서 생략 |
| HealPulse | healing event/message | 160ms | player cell green pulse + `+N` | 색맹 모드에서는 `+` 텍스트 유지 |
| DoorToggle | `DoorChanged` | 100ms | 문 glyph highlight | open/close 구분 텍스트 |
| ItemPickup | `ItemPickedUp` | 120ms | item cell fade 또는 status blink | inventory letter 표시 |
| DangerAlert | HP 30% 이하 또는 death-adjacent event | 400ms | status border warning flash | reduced motion에서 고정 warning |

프레임 정책:

- 기본 `frame_rate = 30.0`.
- 기본 `tick_rate = 8.0`.
- 애니메이션은 최대 400ms 이내로 끝난다.
- UI frame 누락은 gameplay turn 누락으로 취급하지 않는다.

### 15.6 색상/접근성 토큰 추가

| Token | RGB | 용도 |
| --- | --- | --- |
| `color.inspect` | `#7aa2f7` | hover/inspect outline |
| `color.path.safe` | `#87af87` | 안전 경로 preview |
| `color.path.risky` | `#d7af5f` | 위험 경로 preview |
| `color.effect.damage` | `#ff5f5f` | damage flash |
| `color.effect.heal` | `#87d75f` | heal pulse |
| `color.label.bg` | `#20262b` | 자동 라벨 배경 |
| `color.focus` | `#5fd7ff` | focused panel border |

색맹/저모션 정책:

- `UiColorProfile::ColorBlindSafe`에서는 red/green 단독 대비 금지.
- `UiColorProfile::Monochrome`에서는 bold, underline, inverse, glyph prefix를 사용한다.
- `reduced_motion=true`에서는 반복 점멸 금지.

### 15.7 완료 기준 확장

v0.1 UI 완료 기준에 다음을 추가한다.

- 정보 가독성 개선 계획의 타입/수치/비목표가 `spec.md`와 일치한다.
- TUI adapter 구현 시 `ui_layout`, `ui_input_mapping`, `ui_effect_projection` 테스트를 추가한다.
- 80x28 degraded layout에서도 command bar와 message log가 겹치지 않는다.
- mouse 미지원 터미널에서도 keyboard-only parity가 유지된다.

## Phase 3 전투/사망 Headless 구조 반영

```text
GameSession
├── GameWorld
│   ├── GameMap 40x20
│   ├── player_id: EntityId(1)
│   └── EntityStore
│       ├── Player @ (5,5), hp 16, dagger 1d4
│       ├── Jackal @ (6,5), hp 4, 1d2
│       └── Goblin @ (20,12), hp 6, 1d4
├── systems::movement
│   └── 빈 passable tile 이동
├── systems::combat
│   └── hostile occupied tile bump attack
└── systems::death
    ├── monster: alive=false tombstone
    └── player: RunState::GameOver
```

구현 시 주의사항:

- UI/TUI는 Phase 3 combat state를 직접 수정하지 않고 `GameSession::submit(CommandIntent::Move(direction))` 결과 event만 표현한다.
- 공격 animation/effect는 후속 UI Phase에서 `AttackResolved`/`EntityDied` event를 projection하는 presentation layer로만 구현한다.
- 사망 monster는 즉시 Vec에서 제거하지 않는다. `EntityId` 안정성이 replay/hash의 일부이므로 tombstone 정책을 유지한다.
- Phase 4 이후 player 공격은 `Inventory.equipped_melee`의 item profile 또는 unarmed fallback으로 해석한다.


## Phase 4 아이템/인벤토리 Headless 구조 반영

```text
GameWorld
├── EntityStore
│   ├── Actor: Player / Monster
│   └── Item
│       ├── PotionHealing @ OnMap(8,5), letter=None
│       ├── Dagger @ Inventory(player), letter=a
│       └── FoodRation @ Inventory(player), letter=b
└── Inventory(player)
    ├── entries: [dagger=a, food=b, picked potion=c]
    └── equipped_melee: Option<EntityId>
```

구현 시 주의사항:

- Inventory UI는 core state를 직접 바꾸지 않고 `CommandIntent::Pickup`, `ShowInventory`, `Wield`, `Quaff`만 제출한다.
- `ShowInventory`는 eventless/no-turn이며, 화면은 `Observation.inventory`를 표시한다.
- 소비된 potion은 inventory에서 사라지지만 item entity는 `Consumed` tombstone과 `assigned_letter`를 유지한다.
- Phase 4에는 drag/drop, drop, read, zap, throw, file save/load를 구현하지 않는다.
