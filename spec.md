# AIHack Master Spec v2

> Archive chain
> - Latest: `.archive/spec_archive_260715.md`
> - Previous: first archive
>
> 이 문서는 active 계약만 포함한다. Phase 1~20의 완료 이력과 과거 hash는 위 아카이브에 있다.

문서 상태: active implementation target
작성일: 2026-07-15
목표 버전: 0.3.0
현재 코드 기준: Cargo package 0.1.0, 문서상 Phase 20 완료 상태
기준 문서: `AI_IMPLEMENTATION_DOC_STANDARD.md`

## 1. 문서 운영 규칙

이 문서는 AIHack v0.3.0 리팩터링의 최상위 계약이다.

- 구현 전 이 문서와 `IMPLEMENTATION_SUMMARY.md`의 해당 Task를 읽는다.
- 계약 변경은 이 문서를 먼저 수정하고 `DESIGN_DECISIONS.md`에 새 ADR을 추가한다.
- 완료 표시는 `audit_roadmap.md`의 명령과 산출물 증거가 모두 통과한 뒤에만 변경한다.
- 현재 구현과 목표 구현을 같은 문장에서 완료로 표현하지 않는다.
- `legacy_nethack_port_reference/`의 코드, 데이터, 문자열을 출처 기록 없이 복사하지 않는다.
- UI와 LLM은 `GameSession` 또는 `GameWorld`의 내부 상태를 직접 변경하지 않는다.
- LLM 출력은 신뢰하지 않으며, 현재 turn과 snapshot hash를 재검증한다.

## 2. 프로젝트 정체성

AIHack은 NetHack 3.6.7의 관찰 가능한 규칙을 시나리오별로 재구현하는 Rust 턴제 로그라이크다. 목표는 C 소스의 줄 단위 번역이 아니라 다음 세 가지를 동시에 만족하는 호환 엔진이다.

1. 같은 seed와 command sequence에서 동일한 결과를 내는 결정론적 코어
2. NetHack 3.6.7 규칙을 출처와 golden scenario로 추적할 수 있는 호환성 계층
3. 로컬 LLM을 메시지, 상황 요약, legal-action 추천, 비상태성 소프트 판정에 사용하는 격리된 adapter

## 3. v0.3.0 목표와 성공 기준

### 3.1 목표

- 빌드 도구·의존성·명령을 고정하여 깨끗한 환경에서 같은 결과를 재현한다.
- `GameSession`과 `GameWorld`의 mutable 상태를 private으로 만들고 명령 트랜잭션만 변경 경로로 허용한다.
- TOML 데이터를 실제 런타임 `ContentRegistry`가 소비하게 한다.
- 요청 turn과 snapshot hash가 일치하는 LLM 응답만 표시 또는 추천에 사용한다.
- `--turns 1000` 검증이 실제로 accepted turn 1000개를 수행하게 한다.
- NetHack 참조 항목마다 출처, 변환 방식, 테스트 ID를 추적한다.
- 기존 Phase 1~20의 사용자 동작과 save/replay schema v1을 회귀 없이 유지한다.

### 3.2 정량 성공 기준

| ID | 기준 |
| --- | --- |
| SC-BUILD-01 | `cargo build --locked --all-targets`가 새 target 디렉터리에서 통과 |
| SC-BUILD-02 | Linux와 Windows CI가 fmt, clippy, test, release build를 통과 |
| SC-CORE-01 | `src/ui`, `src/llm`, integration test에서 session/world 필드 직접 대입 0건 |
| SC-CORE-02 | 모든 accepted turn 뒤 invariant 검사 오류 0건 |
| SC-DATA-01 | 시작 월드의 item, monster, level 데이터가 `ContentRegistry`에서 생성 |
| SC-TEST-01 | seed 42, 7, 1234 각각 `final_turn = 1000` |
| SC-TEST-02 | 같은 seed와 같은 1000-command sequence를 3회 실행한 hash가 모두 동일 |
| SC-ARCH-01 | `aihack-core` dependency tree에 TUI/HTTP 0건이며 기존 binary CLI 유지 |
| SC-LLM-01 | provider 미설정, 연결 실패, 2초 초과, 빈 응답에서 코어 hash 불변 |
| SC-LLM-02 | stale turn 또는 stale snapshot hash 응답 실행 0건 |
| SC-LLM-03 | narrative/soft verdict core effect 0건, suggestion은 명시 승인 전 submit 0건 |
| SC-COMPAT-01 | 기존 P8-G01..P8-G20과 신규 NH367-C001..C010 모두 통과 |
| SC-LICENSE-01 | runtime 포함 자산 provenance가 모두 `Approved`이고 legacy direct import 0건 |
| SC-DOC-01 | AI 문서 표준 체크리스트 12개 항목 전부 PASS |

## 4. v0.3.0 비목표

- NetHack 3.6.7의 모든 몬스터, 아이템, 특수 레벨을 한 번에 구현
- NetHack C 소스를 자동 번역해 Rust 모듈로 포함
- LLM이 HP, 위치, 인벤토리, RNG, score 또는 save 데이터를 직접 변경
- LLM 자유 텍스트를 명령으로 실행
- 네트워크 멀티플레이와 원격 계정 시스템
- 그래픽 타일 렌더러
- save schema v2 배포
- ECS 도입
- 비동기 또는 다중 스레드 게임 규칙 실행

## 5. 동결된 핵심 결정

| 결정 ID | 값 |
| --- | --- |
| DEC-PRODUCT-01 | NetHack 3.6.7 행동 호환 재구현, 줄 단위 포트 금지 |
| DEC-VERSION-01 | v0.3.0 구현 종료 시 Cargo와 문서 버전을 0.3.0으로 동기화 |
| DEC-RUST-01 | `rust-toolchain.toml` channel 1.94.1, `rust-version = "1.94"` |
| DEC-UI-DEP-01 | v0.3.0은 `ratatui 0.29`와 `crossterm 0.28.1` 한 계열 사용 |
| DEC-RUNTIME-01 | 단일 스레드 deterministic turn transaction |
| DEC-STATE-01 | `GameSession`이 유일한 mutable session owner이며 필드는 private |
| DEC-AI-01 | AI read는 `Observation`, write 제안은 `ActionIntent` |
| DEC-LLM-01 | loopback OpenAI-compatible HTTP endpoint만 기본 허용 |
| DEC-LLM-02 | LLM 소프트 판정은 presentation-only verdict이며 core effect 없음 |
| DEC-RNG-01 | 모든 난수는 `GameRng`을 통과 |
| DEC-SAVE-01 | v0.3.0은 JSON `SaveDataV1`과 replay JSONL v1 유지 |
| DEC-CONTENT-01 | embedded TOML을 시작 시 한 번 파싱해 immutable registry 생성 |
| DEC-WORKSPACE-01 | core, content, AI contract, LLM adapter, TUI/headless app 경계로 분리 |
| DEC-LICENSE-01 | 출처 상태가 `Approved`가 아닌 NetHack 자산은 런타임에 포함 금지 |

## 6. 기술 스택과 목표 워크스페이스

```text
Cargo.toml
rust-toolchain.toml
src/
  lib.rs                  # aihack compatibility facade와 root integration-test host
crates/
  aihack-core/
  aihack-content/
  aihack-ai-contract/
  aihack-llm/
apps/
  aihack-tui/
  aihack-headless/
tests/
  compatibility/
  fixtures/
  long_run/
docs/
  compatibility/
runtime/
  saves/
  replays/
```

root package `aihack`은 `publish = false`인 compatibility facade와 기존 `tests/**`의 host로 유지한다. production binary는 `apps/aihack-tui`와 `apps/aihack-headless`가 각각 `aihack`, `aihack-headless` 이름으로 생성한다. facade는 core, content, AI contract, LLM, TUI/headless library의 승인된 public API를 기존 module path로 re-export하되 private mutable field는 노출할 수 없다. facade는 배포 binary dependency가 아니다.

의존 방향:

```text
aihack-content ------> aihack-core
aihack-ai-contract --> aihack-core read-only DTO
aihack-llm ----------> aihack-ai-contract
aihack-tui ----------> core + content + AI contract + LLM
aihack-headless -----> core + content + AI contract
root aihack facade --> all public workspace libraries, tests only
```

금지 방향:

- `aihack-core -> aihack-llm`
- `aihack-core -> ratatui/crossterm/http client`
- `aihack-content -> UI`
- adapter에서 core 내부 필드 접근

워크스페이스 이동은 R5에서 수행한다. R1~R4 동안 현재 `src/` 구조를 유지하여 기능 변경과 파일 이동을 분리한다.

## 7. 빌드 및 런타임 파이프라인

### 7.1 고정 명령

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo run --locked --bin aihack -- --seed 42
cargo run --locked --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
```

R5 이전에는 `--workspace`를 제거하고 같은 명령을 사용한다.

### 7.2 산출물

| 산출물 | 경로 |
| --- | --- |
| TUI debug binary | `target/debug/aihack` |
| headless debug binary | `target/debug/aihack-headless` |
| release binaries | `target/release/aihack*` |
| save | `runtime/saves/dev_save_v1.json` |
| replay | `runtime/replays/<seed>-<policy>-<turns>.jsonl` |
| long-run report | `runtime/reports/long-run-<seed>.json` |
| compatibility report | `runtime/reports/compatibility-v0.3.0.json` |

빌드 스크립트는 복사할 두 binary가 없으면 exit code 1을 반환한다. 성공 메시지는 두 파일 존재와 실행 가능 권한 확인 뒤에만 출력한다.

## 8. 상태 전이와 턴 파이프라인

```text
Title --Start--> CharacterCreation --Confirm--> Playing
Playing --needs direction--> AwaitingDirection --direction/cancel--> Playing
Playing --needs item--> AwaitingInventorySelection --item/cancel--> Playing
Playing --message overflow--> MorePrompt --ack--> Playing
Playing --death/quit--> GameOver
GameOver --new run--> Title
```

new run은 이전 seed에 `wrapping_add(1)`을 적용한다. session/world/RNG/turn/event와 LLM/UI transient state는 초기화하고 접근성 theme 설정은 유지한다. 기존 save/replay artifact는 자동 삭제하지 않는다.

accepted turn 순서:

1. 요청의 run state와 command legality 검증
2. `TurnTransaction` 생성
3. player action 적용
4. tile/item/level 상호작용 적용
5. monster intent 수집
6. monster intent 적용
7. status tick
8. death/score 계산
9. invariant 검사
10. event log commit
11. snapshot hash 생성
12. observation 생성

검증 실패 시 transaction은 commit하지 않고 `Err(SubmitError)`를 반환한다. UI-only command의 `Ok(TurnOutcome { turn_advanced: false, .. })`와 실패를 같은 상태로 취급하지 않는다.

## 9. 경계 타입 계약

### 9.1 Session API

```rust
pub struct GameSession {
    meta: GameMeta,
    rng: GameRng,
    turn: u64,
    state: RunState,
    world: GameWorld,
    event_log: Vec<GameEvent>,
}

pub trait GameClient {
    fn snapshot(&self) -> GameSnapshot;
    fn observation(&self) -> Observation;
    fn action_space(&self) -> ActionSpace;
    fn submit(&mut self, intent: CommandIntent) -> Result<TurnOutcome, SubmitError>;
}

pub struct SessionRevision {
    pub turn: u64,
    pub snapshot_hash: SnapshotHash,
}

pub struct TurnOutcome {
    pub revision_before: SessionRevision,
    pub revision_after: SessionRevision,
    pub turn_advanced: bool,
    pub events: Vec<GameEvent>,
}

pub enum SubmitError {
    InvalidRunState { state: RunStateSummary },
    IllegalAction { action: ActionIntent },
    MissingDirection,
    MissingInventorySelection,
    InvariantViolation { errors: Vec<WorldInvariantError> },
}
```

공개 getter는 `seed()`, `turn()`, `run_state()`, `snapshot()`, `observation()`, `action_space()`로 제한한다. 테스트 상태 설정은 `tests/support/SessionFixtureBuilder`만 사용한다. `Ok(TurnOutcome)`은 command accepted를 뜻하며, `turn_advanced=true`인 결과만 headless accepted turn에 합산한다. `Err(SubmitError)`는 world, RNG, turn, event log를 바꾸지 않는다.

### 9.2 Turn transaction과 invariant

```rust
pub struct TurnTransaction {
    pub revision_before: SessionRevision,
    pub command: CommandIntent,
    pub events: Vec<GameEvent>,
    pub rng_draws_before: u64,
}

pub enum WorldInvariantError {
    MissingPlayer,
    PlayerLevelMismatch,
    DuplicateInventoryLetter { letter: char },
    InvalidEntityId { id: EntityId },
    MissingLevel { level: LevelId },
    ItemLocationMismatch { item: EntityId },
}

pub struct InvariantReport {
    pub checked: u16,
    pub errors: Vec<WorldInvariantError>,
}
```

v0.3.0 invariant 수는 6종이며 accepted turn마다 모두 검사한다.

### 9.3 Content registry

```rust
pub struct ContentRegistry {
    schema_version: u16,
    content_hash: String,
    items: BTreeMap<ItemId, ItemDefinition>,
    monsters: BTreeMap<MonsterId, MonsterDefinition>,
    levels: BTreeMap<LevelId, LevelDefinition>,
}

pub enum ContentError {
    Parse { file: String, message: String },
    DuplicateId { id: String },
    UnknownReference { owner: String, target: String },
    InvalidDice { value: String },
    InvalidCoordinate { level: LevelId, x: i16, y: i16 },
    MissingStairsPair { level: LevelId },
}
```

`schema_version = 1`. 동일 ID 중복, 존재하지 않는 참조, 맵 밖 좌표는 시작 실패다. panic fallback은 허용하지 않는다.
`schema_version()`, `content_hash()`, `item(id)`, `monster(id)`, `level(id)` read-only query만 공개한다.

### 9.4 LLM 설정과 요청

```rust
pub struct LocalLlmConfig {
    enabled: bool,
    base_url: String,
    model: String,
    connect_timeout_ms: u64,
    narrative_timeout_ms: u64,
    decision_timeout_ms: u64,
    max_output_chars: usize,
}

impl LocalLlmConfig {
    pub fn disabled() -> Self;
    pub fn from_env() -> Result<Self, LlmConfigError>;
    pub fn enabled(&self) -> bool;
    pub fn endpoint(&self) -> &str;
    pub fn model(&self) -> &str;
    pub fn connect_timeout_ms(&self) -> u64;
    pub fn request_timeout_ms(&self, kind: &LlmRequestKind) -> u64;
    pub fn max_output_chars(&self) -> usize;
}

pub enum LlmConfigError {
    InvalidBoolean { name: String },
    InvalidRange { name: String, min: u64, max: u64 },
    MissingModel,
    InvalidEndpoint,
}

pub struct RequestId(String);

pub struct LlmRequestInput {
    pub schema_version: u16,
    pub revision: SessionRevision,
    pub observation: LlmObservationView,
    pub action_space: ActionSpace,
    pub kind: LlmRequestKind,
}

pub struct LlmObservationView {
    pub turn: u64,
    pub run_state: RunStateSummary,
    pub player: PlayerObservation,
    pub visible_tiles: Vec<VisibleTile>,
    pub visible_entities: Vec<VisibleEntity>,
    pub inventory: Vec<InventoryObservation>,
    pub last_events: Vec<GameEventSummary>,
}

pub enum LlmRequestKind {
    Narrative,
    Decision,
    SoftAdjudication { user_text: String },
}

pub trait LocalLlmPort {
    fn enqueue(&self, input: LlmRequestInput) -> Result<RequestId, LlmEnqueueError>;
    fn try_recv(&self) -> Option<LlmResponseEnvelope>;
}

pub enum LlmEnqueueError {
    Disabled,
    Busy { capacity: u16 },
    InvalidEndpoint,
    InvalidModel,
    InvalidInput { code: LlmInputCode },
    WorkerStopped,
}

pub enum LlmInputCode {
    EmptyUserText,
    TextTooLong,
    ControlCharacter,
    PayloadTooLarge,
}
```

`RequestId`는 worker가 생성하는 opaque UUID 문자열이며 caller가 임의 문자열을 만들 수 없다. input과 output type을 분리하고 모든 external response는 worker boundary에서 validate한다.

`LlmObservationView`는 `Observation`에서 만든 read-only projection이다. visible tiles 최대 800개, visible entities 128개, inventory 52개, last events 20개, action space 64개로 제한한다. canonical request JSON이 32,768 bytes를 넘으면 `PayloadTooLarge`다. soft-adjudication user text는 trim 후 1..=240자이며 control/ANSI escape를 거부한다.

기본값:

```toml
[llm]
enabled = false
base_url = "http://127.0.0.1:11434/v1"
model = ""
connect_timeout_ms = 500
narrative_timeout_ms = 2000
decision_timeout_ms = 1500
max_output_chars = 240
```

`enabled=true`이면 model은 1..=128자다. Narrative는 2000ms, Decision과 SoftAdjudication은 1500ms deadline을 사용한다. URL은 `http` scheme, userinfo 없음, query/fragment 없음, host가 `127.0.0.1`, `localhost`, `[::1]` 중 하나, port 1..=65535여야 한다. 연결 직전에 resolve한 모든 IP가 loopback인지 다시 검사한다.

transport는 `reqwest 0.13.4`의 `blocking`, `json` feature만 사용하고 default feature, redirect, system proxy를 끈다. TUI thread에서 직접 호출하지 않고 capacity 16의 bounded request/response channel과 LLM worker thread 1개를 사용한다. queue가 가득 차면 `Busy { capacity: 16 }`을 즉시 반환한다. 같은 request kind의 outstanding 요청은 1개이며 CTA cooldown은 250ms다.

HTTP contract:

```text
POST {base_url}/chat/completions
Content-Type: application/json
```

```json
{
  "model": "local-model",
  "messages": [
    {"role": "system", "content": "Return one JSON object matching the requested AIHack schema. Never emit a state patch."},
    {"role": "user", "content": "<canonical LlmRequestInput JSON>"}
  ],
  "temperature": 0.0,
  "max_tokens": 128,
  "stream": false
}
```

성공 HTTP body는 최대 65,536 bytes까지만 읽고 `choices[0].message.content`를 JSON object로 다시 parse한다. 분류는 다음과 같이 단일화한다.

| 경계 결과 | typed result |
| --- | --- |
| DNS/connect/read/write 오류 | `Unavailable` |
| request deadline 초과 | `Timeout` |
| HTTP 2xx 외 | `HttpStatus { code }` |
| body 65,536 bytes 초과 | `BodyTooLarge { limit_bytes: 65536 }` |
| outer JSON/content JSON 불일치 | `InvalidSchema { code }` |
| current revision 불일치 | `Stale` |

### 9.5 LLM 출력

```rust
pub struct LlmResponseEnvelope {
    pub schema_version: u16,
    pub request_id: RequestId,
    pub revision: SessionRevision,
    pub result: Result<LlmPayload, LlmResponseError>,
}

pub enum LlmPayload {
    Narrative(NarrativePayload),
    Decision(DecisionPayload),
    SoftAdjudication(SoftAdjudicationPayload),
}

pub struct NarrativePayload {
    pub text: String,
}

pub struct DecisionPayload {
    pub action: ActionIntent,
    pub rationale: String,
    pub confidence: f32,
}

pub enum SoftVerdict {
    Favorable,
    Neutral,
    Unfavorable,
}

pub struct SoftAdjudicationPayload {
    pub verdict: SoftVerdict,
    pub reason_code: String,
    pub message: String,
}

pub enum LlmResponseError {
    Unavailable,
    Timeout,
    HttpStatus { code: u16 },
    BodyTooLarge { limit_bytes: usize },
    InvalidSchema { code: LlmValidationCode },
    Stale,
}

pub enum LlmValidationCode {
    UnknownRequestId,
    MissingChoice,
    NonTextContent,
    WrongKind,
    EmptyText,
    TextTooLong,
    InvalidAction,
    InvalidConfidence,
    InvalidReasonCode,
    ControlCharacter,
}
```

wire JSON은 camelCase field와 UPPER_SNAKE enum을 사용한다.

`request_id`와 `revision`은 provider content에서 읽지 않는다. worker가 enqueue 시 보관한 값을 response envelope에 복사하며, provider는 `LlmPayload` JSON만 반환한다.

```json
{"kind":"DECISION","action":{"type":"MOVE","direction":"EAST"},"rationale":"Visible floor is clear.","confidence":0.72}
```

검증 규칙:

- response request ID가 outstanding request와 다르면 `InvalidSchema { code: UnknownRequestId }`
- payload kind가 요청 kind와 다르면 `InvalidSchema { code: WrongKind }`
- response turn 또는 snapshot hash가 현재 session과 다르면 `Stale`
- action이 현재 action space에 없으면 `InvalidAction`
- narrative/message는 trim 후 Unicode scalar 1..=240자
- rationale은 Unicode scalar 0..=160자
- confidence는 finite `0.0..=1.0`
- reason code는 ASCII `[A-Z0-9_]{1,32}`
- C0/C1 control과 ANSI escape를 포함하면 `ControlCharacter`
- unknown JSON field는 거부하고 누락 field에 default를 넣지 않음
- soft verdict는 message 표시 외 상태 변경 권한이 없음
- provider 실패의 UI fallback은 narrative `Local narrator unavailable.`, decision 없음, soft verdict `Neutral/LLM_UNAVAILABLE`이며 snapshot hash를 바꾸지 않음

### 9.6 Public contract 안정성

- public struct field는 DTO 외 private이며 constructor가 validation을 수행한다.
- public error와 command enum은 `#[non_exhaustive]`로 선언하고 consumer는 wildcard branch를 가진다.
- LLM wire, save, replay, content schema는 각각 명시적 `schema_version = 1`을 가진다.
- version 불일치는 default 보정 없이 `UnsupportedSchema { expected, actual }`로 실패한다.
- 기존 field 제거, type 변경, enum 의미 변경은 v0.3.x에서 금지한다.
- 새 optional wire field는 default가 기존 동작과 같고 golden fixture를 추가한 경우에만 허용한다.
- dependency와 public DTO는 한 version만 사용하며 병렬 v1/v2 type을 같은 release에 노출하지 않는다.
- v2가 필요한 변경은 새 ADR, migration fixture, 한 minor release의 deprecation notice를 먼저 추가한다.

## 10. 동결된 게임 공식

v0.3.0 리팩터링은 아래 공식을 변경하지 않는다.

```text
attack_roll = d20 + attacker.hit_bonus + weapon.hit_bonus
defense = 10 + defender.ac
hit = attack_roll >= defense

damage = max(1, damage_roll + attacker.damage_bonus - defender.damage_reduction)

vision_radius = 8
low_hp = hp * 100 <= max_hp * 30
trap_pit_damage = 3
```

공식 변경은 별도 feature spec과 golden scenario ID가 필요하다.

## 11. 실데이터 기준

### 11.1 플레이어

```toml
[player]
id = "player.adventurer"
hp = 16
max_hp = 16
energy = 6
strength = 10
dexterity = 10
ac = 0
hit_bonus = 2
damage_bonus = 0
vision_radius = 8
start_items = ["item.weapon.dagger", "item.food.ration"]
```

### 11.2 몬스터

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
```

### 11.3 아이템

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
```

### 11.4 레벨

```toml
level_id = "main:1"
branch = "Main"
depth = 1
width = 40
height = 20
player_start = [5, 5]
stairs_down = [34, 15]

[[monster]]
id = "monster.jackal"
pos = [6, 5]
```

### 11.5 SaveDataV1 기본값

기존 `SaveDataV1`의 top-level 필드를 생략하지 않는다.

| field | type | 새 게임 기본값 |
| --- | --- | --- |
| schema_version | u16 | 1 |
| seed | u64 | CLI seed, 예: 42 |
| turn | u64 | 0 |
| run_state | RunState | `GameSession::new`은 Title, headless `new_for_playing`은 Playing |
| rng_state | RngStateV1 | `{ seed: 42, draws: 0 }` |
| world | SavedWorldV1 | `ContentRegistry`의 main:1과 player fixture |
| event_log | Vec<GameEvent> | 빈 배열 |

`SavedWorldV1` 필드는 `levels`, `current_level`, `entities`, `player_id`, `inventory`, `nutrition`, `luck`, `prayer_cooldown`, `paralysis_turns`, `hallucinating`, `kill_count`, `gold`, `identified_items` 13개다.

`world` JSON shape는 현재 save/load v1 fixture를 source of truth로 유지한다. R2/R3는 serialization field 이름과 enum tag를 바꾸지 않는다. R8 전에 `tests/fixtures/save_v1.json`을 현재 serializer로 생성하고 load→save canonical-JSON stable test를 추가한다.

## 12. Headless policy

```rust
pub enum HeadlessPolicyId {
    WaitV1,
    SurvivalV1,
    ReplayFile,
}

pub struct HeadlessRunReport {
    pub seed: u64,
    pub policy: HeadlessPolicyId,
    pub requested_turns: u64,
    pub accepted_turns: u64,
    pub submitted_commands: u64,
    pub final_state: RunStateSummary,
    pub final_hash: SnapshotHash,
    pub error: Option<HeadlessRunError>,
}

pub enum HeadlessRunError {
    InvalidCli,
    NoAcceptedAction { turn: u64, attempts: u8 },
    GameOver { turn: u64, cause: DeathCauseSummary },
    Submit { turn: u64, error: SubmitError },
    ReportWrite { path: String },
}
```

`survival-v1` 규칙:

1. HP가 max HP의 50% 이하이고 healing potion legal이면 quaff
2. adjacent hostile에 대한 bump attack legal이면 공격
3. 현재 tile에서 pickup legal이면 pickup
4. 통과 가능한 방향을 North, East, South, West, NorthEast, SouthEast, SouthWest, NorthWest 순으로 선택
5. 이동이 모두 불법이면 Wait
6. GameOver가 되면 같은 seed의 새 session을 만들지 않고 report를 실패 처리

`--turns 1000` 성공은 `accepted_turns == 1000`일 때만 인정한다.

## 13. NetHack 3.6.7 호환성 계약

호환 시나리오 ID 형식은 `NH367-Cnnn`이다.

v0.3.0 신규 10개:

| ID | 규칙 |
| --- | --- |
| NH367-C001 | 벽 방향 이동의 위치와 turn 정책 |
| NH367-C002 | 닫힌 문의 이동·LOS·open 전이 |
| NH367-C003 | bump attack의 hit/damage/death event |
| NH367-C004 | pickup/wield/quaff의 inventory와 item state |
| NH367-C005 | stairs 왕복의 level state 보존 |
| NH367-C006 | hidden door/trap search의 reveal 조건 |
| NH367-C007 | throw/zap/read의 charge와 projectile stop |
| NH367-C008 | hunger/status threshold 전이 |
| NH367-C009 | save/load 후 command와 RNG continuation 일치 |
| NH367-C010 | death cause와 GameOver final state |

각 시나리오는 `docs/compatibility/NH367-Cnnn.md`에 출처 파일/함수, 관찰 규칙, AIHack 입력, 기대 이벤트, 저작권 상태를 기록한다.

## 14. 저장·replay 정책

- `SaveDataV1` schema_version은 1 유지
- 저장 직후와 load 직후 snapshot hash가 같아야 함
- replay line은 turn_before, command, outcome, snapshot_hash_after를 포함
- LLM narrative, rationale, soft verdict는 save와 replay truth에 포함하지 않음
- content_hash는 save metadata에 추가하지 않고 v0.3.0 load 시 현재 registry와 fixture test로 검증
- schema 필드 추가가 필요하면 v0.4.0 spec에서 SaveDataV2를 정의

R2의 internal `Result<TurnOutcome, SubmitError>`는 replay wire shape를 바꾸지 않는다.

```rust
pub struct ReplayTurnOutcomeV1 {
    pub accepted: bool,
    pub turn_advanced: bool,
    pub events: Vec<GameEvent>,
    pub snapshot_hash: SnapshotHash,
    pub next_state: RunState,
}

pub struct ReplayLineV1 {
    pub turn_before: u64,
    pub command: CommandIntent,
    pub outcome: ReplayTurnOutcomeV1,
    pub snapshot_hash_after: SnapshotHash,
}
```

`Ok(TurnOutcome)`은 `accepted=true`로, `Err(SubmitError)`는 현재 revision의 hash/state와 빈 event를 가진 `accepted=false`로 projection한다. 이 projection test가 기존 replay fixture와 field/tag 단위로 같아야 한다.

## 15. 구현 단계

| Phase | 이름 | 선행 | 완료 게이트 |
| --- | --- | --- | --- |
| R0 | 문서·출처 기준 확정 | 없음 | 문서 세트와 gap register PASS |
| R1 | 빌드 재현성 | R0 | SC-BUILD-01..02 |
| R2 | 코어 캡슐화·transaction | R1 | SC-CORE-01..02 |
| R3 | ContentRegistry 실연결 | R2 | SC-DATA-01 |
| R4 | 장기 실행·테스트 정당성 | R3 | SC-TEST-01..02 |
| R5 | 워크스페이스 경계 분리 | R4 | SC-ARCH-01 |
| R6 | 실제 로컬 LLM adapter | R5 | SC-LLM-01..03 |
| R7 | NetHack 호환성·출처 추적 | R3 | SC-COMPAT-01, SC-LICENSE-01 |
| R8 | v0.3.0 통합 감사 | R6, R7 | 전체 SC와 문서 gate PASS |

세부 Task와 파일 단위는 `IMPLEMENTATION_SUMMARY.md`가 정의한다.

## 16. 보안 및 구현 경계

- LLM endpoint는 loopback 기본이며 remote host는 v0.3.0에서 거부
- prompt에 save 경로, 환경변수, 파일 내용, 비밀정보를 넣지 않음
- LLM response를 로그에 남길 때 최대 240 chars로 제한
- content 파일은 embedded read-only이며 실행 중 임의 경로를 읽지 않음
- save/replay path는 CLI가 받은 root 안에서 정규화하고 상위 디렉터리 탈출을 거부
- `unsafe` 도입은 별도 ADR과 Miri 검증 없이는 금지
- 레거시 라이선스 파일은 수정하지 않고 공식 원문과 checksum으로 교체 계획을 별도 기록

## 17. 잔여 리스크

| 리스크 | 영향 | 구현을 막지 않는 완화 |
| --- | --- | --- |
| NetHack 라이선스 범위 미확정 | 높음 | R1~R6은 독립 코드만 수정, R7 자산 반입은 Approved provenance만 허용 |
| workspace 이동 중 대형 diff | 중간 | R5 이전 behavior gate 고정, crate별 순차 이동 |
| HTTP dependency 증가 | 중간 | LLM crate에만 격리, core dependency tree 비교 |
| hash 변경 가능성 | 높음 | R2는 behavior-preserving, 의도된 hash 변경은 ADR과 새 baseline 필요 |
| 1000-turn survival policy 실패 | 중간 | 실패 seed와 command index를 report에 기록하고 원인 수정 후 재실행 |
| 문서와 구현 재드리프트 | 중간 | 각 Phase 종료 시 문서 표준 checker와 audit roadmap 실행 |

## 18. v0.3.0 최종 완료 조건

아래가 모두 충족되어야 완료다.

- SC-BUILD-01..02, SC-CORE-01..02, SC-DATA-01, SC-TEST-01..02, SC-ARCH-01, SC-LLM-01..03, SC-COMPAT-01, SC-LICENSE-01, SC-DOC-01 PASS
- `Cargo.toml`, README, CHANGELOG 버전 0.3.0 일치
- `cargo tree -d`에 crossterm 중복 0건
- provider 없는 환경에서도 TUI와 headless가 정상 동작
- `PROVENANCE.md`에 Unknown 상태로 런타임에 포함된 자산 0건
- `audit_roadmap.md`의 R8 판정이 PASS
