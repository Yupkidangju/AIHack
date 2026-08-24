# AIHack Master Spec v2

> Archive chain
> - Latest: `.archive/spec_archive_260715.md`
> - Previous: first archive
>
> 이 문서는 active 계약만 포함한다. Phase 1~20의 완료 이력과 과거 hash는 위 아카이브에 있다.

문서 상태: active implementation target
작성일: 2026-07-15
목표 버전: 0.3.0
현재 코드 기준: Cargo package 0.3.0, report 30 successor SHA `ed02dbff3911194e1c4aaaf9b989e5bd41c1b80a`의 [Actions `32733235414`](https://github.com/Yupkidangju/AIHack/actions/runs/32733235414) clean same-SHA Ubuntu/Windows actual bundle Verified. 후속 독립 재감사와 별도 게시 승인 전까지 PROGRAM/PUBLICATION HOLD를 유지한다.
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
| SC-BUILD-01 | `cargo build --workspace --all-targets --locked`가 새 target 디렉터리에서 통과 |
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
| SC-LICENSE-01 | runtime 포함 자산 provenance가 모두 machine-validated `Approved`이고, reviewer/date/license/scope/notice/evidence와 checksum이 유효하며 project-owner approval ID가 추적되고 legacy direct import 0건 |
| SC-DOC-01 | AI 문서 표준 체크리스트 12개 항목 전부 PASS |

SC-LICENSE-01은 외부 배포를 시작하기 전 R8 최종 런칭 게이트다. 2026-07-20 프로젝트 소유자는 AIHack을 NetHack 3.6.7 원본 소스로 의도를 추론한 AI-assisted semantic rewrite 파생물로 분류하고 whole-work NGPL 적용을 승인했다. 직접 지시, 범위와 경계는 `PROJECT_OWNER_LICENSE_APPROVAL.md`의 `AIHACK-OWNER-2026-07-20-NGPL-01`로 추적한다. runtime/scenario provenance, 공식 라이선스 원문, `NOTICE`, `MODIFICATIONS.md`, `RELEASE-METADATA`, `SHA256SUMS`와 complete corresponding source 배포 계약을 함께 검증하며 metadata의 필수 key는 archive와 output에 정확히 한 번 존재하고 owner/modification 값 전체가 실제 record ID와 일치해야 한다. 이 project-owner 결정은 qualified legal opinion, R8 기술 감사 `PASS`나 외부 게시 자체를 뜻하지 않는다.

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
| DEC-UI-DEP-01 | v0.3.0은 RustSec 경고가 없는 `ratatui 0.30`와 `crossterm 0.29` 한 계열 사용 |
| DEC-RUNTIME-01 | 단일 스레드 deterministic turn transaction |
| DEC-STATE-01 | `GameSession`이 유일한 mutable session owner이며 필드는 private |
| DEC-AI-01 | AI read는 `Observation`, write 제안은 `ActionIntent` |
| DEC-LLM-01 | loopback OpenAI-compatible HTTP endpoint만 기본 허용 |
| DEC-LLM-02 | LLM 소프트 판정은 presentation-only verdict이며 core effect 없음 |
| DEC-RNG-01 | 모든 난수는 `GameRng`을 통과 |
| DEC-SAVE-01 | v0.3.0은 JSON `SaveDataV1`과 replay JSONL v1 유지 |
| DEC-CONTENT-01 | embedded TOML을 시작 시 한 번 파싱해 consumer-safe immutable registry 생성 |
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
  aihack-runtime/
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
aihack-runtime ------> core + content + AI contract
aihack-tui ----------> runtime + AI contract + LLM
aihack-headless -----> runtime + AI contract
root aihack facade --> all public workspace libraries, tests only
```

금지 방향:

- `aihack-core -> aihack-llm`
- `aihack-core -> ratatui/crossterm/http client`
- `aihack-content -> UI`
- `aihack-tui/headless -> aihack-core` 직접 접근
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
cargo run --locked -p aihack-headless --bin aihack-headless -- --seed 42 --turns 1000 --policy survival-v1
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

검증 실패 시 transaction은 commit하지 않고 `TurnOutcome { accepted: false, turn_advanced: false, .. }`를 반환한다. UI-only command의 `accepted=true, turn_advanced=false`와 거절을 같은 상태로 취급하지 않는다. `Result<..., SubmitError>`와 revision 기반 public boundary는 R5/R6에서 도입할 후속 계약이다.

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

pub struct TurnOutcome {
    pub accepted: bool,
    pub turn_advanced: bool,
    pub events: Vec<GameEvent>,
    pub snapshot_hash: SnapshotHash,
    pub next_state: RunState,
}
```

현재 공개 getter는 `seed()`, `turn()`, `run_state()`, `event_log()`, `world()`, `snapshot()`, `observation()`이며 `action_space`는 observation의 read-only field다. `submit(&mut self, CommandIntent) -> TurnOutcome`의 `accepted`가 command acceptance를 나타내고, `turn_advanced=true`인 결과만 headless accepted turn에 합산한다. invariant failure와 allocation/projectile/monster/death 처리처럼 mutation 뒤 실패할 수 있어 `transaction_aborted`로 표시한 오류는 working world/RNG/turn/event log/state를 전부 폐기한다. 반면 action-space 내 다단계 입력의 ordinary rejection은 invariant-valid한 `AwaitingDirection`/`AwaitingInventorySelection` 복귀 같은 기존 state transition을 commit할 수 있으며 `accepted=false`만으로 전체 rollback을 뜻하지 않는다. `GameSession`, `GameWorld`, runtime `EntityStore`는 외부 crate에 `DerefMut` 또는 동등한 범용 mutable reference를 제공하지 않으며, production mutation은 `submit`과 저장 복원 validator만 통과한다. default `aihack-runtime` public surface는 read-only `GameWorld` query, validated save/bootstrap, `GameSession::submit`, 순수 score/vision projection만 제공한다. mutating `GameWorld` method와 combat/death/doors/items/movement/monster/projectile/stairs/traps system은 transaction-managed crate 내부 primitive다. 외부 consumer의 fallible·atomic mutation 경계는 `GameSession::submit` 하나다. 호환성용 depleted-death 등 직접 fixture는 opt-in `testing` feature 아래에서만 제공하고 비원자 test primitive임을 명시하며, shipped TUI/headless와 default external consumer는 이 feature를 활성화하거나 import할 수 없다. 일반 integration test 상태 설정은 `aihack::testing::SessionBuilder`의 persisted validator 경로만 사용한다.

`GameClient`, `SessionRevision`, typed `SubmitError`, `ReplayTurnOutcomeV1` projection은 현재 R2 완료 범위가 아니다. R5-2에서 `aihack-runtime`은 content bootstrap·명령 실행·저장 경계를 조합하고, TUI/headless에는 `GameClient` trait만 노출한다. 이 trait의 최소 읽기 계약은 `observation()`, `revision()`, `run_state()`이며 mutation entry는 `submit(CommandIntent) -> TurnOutcome`이다. 저장 I/O와 실제 session 구현은 runtime 내부에 남긴다. R6-2에서 stale-response 판정을 이 revision 계약에 연결한다.

### 9.2 Turn transaction과 invariant

```rust
pub enum WorldInvariantError {
    CurrentLevelMissing { level: LevelId },
    PlayerMissing { player: EntityId },
    PlayerIsNotPlayer { player: EntityId },
    PlayerLevelMismatch { current_level: LevelId, player_level: LevelId },
    PlayerOutOfBounds { level: LevelId, pos: Pos },
    InventoryOwnerMismatch { player: EntityId, owner: EntityId },
}

pub struct InvariantReport {
    pub checked: u8,
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

`schema_version = 1`. 동일 ID 중복, 존재하지 않는 참조, 맵 밖 좌표는 시작 실패다. production bootstrap에는 panic fallback을 허용하지 않는다. v0.3.0의 알려진 item ID는 wire·TUI·ActionSpace가 공유하는 canonical `ItemKind`를 정하며 각 ID는 고정 declared kind/class와 정확히 일치해야 한다. class-changing custom override는 지원하지 않고 registry 단계에서 typed `ContentError`로 거부한다. item glyph는 core `char`와 같은 정확히 한 Unicode scalar여야 하며 empty, 여러 scalar, 결합문자 sequence를 조용히 축약하지 않는다. monster glyph는 현재 typed runtime consumer에 연결되지 않은 schema 값이므로 이번 item 계약의 완료 근거로 계산하지 않고 후속 orphan 판정으로 분리한다.
현재 ID 저장·조회는 콘텐츠 파일과의 호환성을 위해 `String`/`&str`를 사용한다. read-only query는 `schema_version()`, `content_hash()`, `item(id)`, `monster(id)`, `level(id)`이며, 검증·diagnostic/import 지원을 위해 `items()`, `monsters()`, `levels()` iterator와 `from_toml_sources(...)` source constructor도 공개한다. 후자는 runtime mutation API가 아니며 동일 validation path를 테스트와 content import에 제공한다.

### 9.3.1 현재 구현 상태와 R3-4 정렬 조건

R3-4에서 `GameSession::try_new`, `try_new_for_playing` 및 registry-injected variants가 `Result<_, ContentError>`로 bootstrap 오류를 반환하도록 전환했다. TUI와 headless binary는 이 fallible production 경로를 사용한다. 기존 `new`/`new_for_playing` 및 phase fixture는 기존 test fixture 호환을 위해 남아 있는 infallible adapter이며, production startup contract의 근거가 아니다. `from_toml_sources(...)`는 injected-source validation과 import 지원의 명시적 public 경계다. typed ID newtype은 콘텐츠 형식/API를 넓게 바꾸는 후속 호환성 작업으로 분리하며, 현재 SC-DATA-01의 범위는 String ID 계약과 fallible bootstrap을 문서·구현 모두에서 일치시키는 것이다.

### 9.4 LLM 설정과 요청

R6-2에서 stale response를 판정하기 위해 아래 revision type을 도입한다. 이는 현재 R2 `GameSession` public API가 아니라 LLM contract target이다.

```rust
pub struct SessionRevision {
    pub turn: u64,
    pub snapshot_hash: SnapshotHash,
}
```

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
    UnsupportedSchema { expected: u16, actual: u16 },
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

실제 모델 추론 smoke는 SC-LLM-01..03 또는 R6 완료의 필수 조건이 아니다. 자동 failure matrix와 실제 PTY/loopback fixture가 wire, timeout, stale, 승인 및 core effect 경계를 충족하면 R6 구현 증거로 인정한다. 최종 통합 단계에서 추가 호환성 증거가 반드시 필요하다고 판단될 때만, AIHack의 loopback 제한을 유지한 채 localhost OpenAI-compatible 임시 adapter가 Google AI Studio Gemini 같은 원격 API를 대리 호출할 수 있다. 이 adapter는 재사용 가능하게 분리하고 API key는 adapter process의 환경변수로만 주입하며, 실제 model ID는 실행 시점의 제공 목록을 확인해 선택한다. 이 선택 검증은 기본 release gate를 차단하지 않는다.

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
- rationale은 trim 후 Unicode scalar 1..=160자
- confidence는 finite `0.0..=1.0`
- reason code는 ASCII `[A-Z0-9_]{1,32}`
- C0/C1 control과 ANSI escape를 포함하면 `ControlCharacter`
- unknown JSON field는 거부하고 누락 field에 default를 넣지 않음
- soft verdict는 message 표시 외 상태 변경 권한이 없음
- provider 실패의 UI fallback은 narrative `Local narrator unavailable.`, decision 없음, soft verdict `Neutral/LLM_UNAVAILABLE`이며 snapshot hash를 바꾸지 않음
- v0.3.0 runtime TUI와 built-in fallback의 canonical locale은 English다. 다국어 README는 runtime locale 지원을 뜻하지 않으며, provider가 반환한 유효 Unicode text는 번역 없이 표시한다. runtime 5-locale 지원은 versioned message catalog가 승인되는 후속 범위다.

### 9.6 Public contract 안정성

- public struct field는 DTO 외 private이며 constructor가 validation을 수행한다.
- public error와 command enum은 `#[non_exhaustive]`로 선언하고 consumer는 wildcard branch를 가진다.
- LLM wire, save, replay, content schema는 각각 명시적 `schema_version = 1`을 가진다.
- version 불일치는 default 보정 없이 `UnsupportedSchema { expected, actual }`로 실패한다.
- 기존 field 제거, type 변경, enum 의미 변경은 v0.3.x에서 금지한다.
- 새 optional wire field는 default가 기존 동작과 같고 golden fixture를 추가한 경우에만 허용한다.
- dependency와 public DTO는 한 version만 사용하며 병렬 v1/v2 type을 같은 release에 노출하지 않는다.
- v2가 필요한 변경은 새 ADR, migration fixture, 한 minor release의 deprecation notice를 먼저 추가한다.

### 9.7 TUI transition gesture

TUI 입력은 key code 열거가 아니라 candidate의 repeat 안전성과 한 gesture의 lifecycle로 판정한다. soft-input의 문자·Backspace와 안정된 `Playing` 상태의 이동/대기처럼 명시된 연속 동작만 `Repeat`를 허용한다. Title/CharacterCreation/GameOver 전환, Load, Inventory open/close/selection, MorePrompt acknowledge, direction selection, LLM request/result/apply, Esc/Enter/F9/Quit 등 state·overlay·soft-input·debug presentation을 바꾸는 candidate는 `Press` 한 번만 소비한다.

`Release` event 자체는 후보를 만들지 않지만 ConPTY가 한 byte마다 합성 `Press/Release`를 만들 수 있으므로 transition quarantine을 즉시 해제하는 authority로 쓰지 않는다. 논리 identity는 modifier 차이를 제외하고 Enter/CR/LF, Esc/ESC, Backspace/DEL alias를 각각 하나로 정규화한다. transition candidate 뒤 같은 논리 key 또는 새 state의 다른 transition/control candidate를 최소 500ms quiet window와 production loop의 50ms poll 두 번 연속 empty가 모두 충족될 때까지 억제한다. 중간에 억제 대상이 도착하면 quiet window와 idle count를 다시 시작한다. 다른 논리 key가 repeat-safe movement/text candidate이면 즉시 새 gesture로 허용한다. 따라서 `Press→Repeat`, Release 없는 즉시 `Press→Press`, 한 transport write가 합성한 빠른 `Press→Release→Press`는 state 경계를 넘지 않으며, Release 뒤에도 quiet/drain을 확인한 독립 Press는 정상 허용된다. physical key-hold를 직접 재현했다는 주장은 backend parser·ConPTY 증거와 분리한다.

## 10. 동결된 게임 공식

v0.3.0 리팩터링은 아래 공식을 변경하지 않는다.

```text
attack_roll = d20 + attacker.hit_bonus + weapon.hit_bonus
defense = 10 + defender.ac
hit = attack_roll >= defense

damage = max(1, damage_roll + attacker.damage_bonus - defender.damage_reduction)

death_score = gold + kill_count * 10 + depth * 100 + carried_item_base_price - turn / 10

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

#### Save/replay v1 입력 예산과 semantic validation

v0.3.0의 artifact 입력은 신뢰하지 않는다. decoder와 `GameSession::from_save_data*`는 live session을 만들기 전에 아래 예산과 관계 invariant를 fail-closed로 검사한다.

| 항목 | 상한 | 경계 동작 |
| --- | ---: | --- |
| save JSON | 16 MiB (`16,777,216` bytes) | `+1` byte부터 `InvalidSave(ResourceLimit)` |
| replay JSONL 전체 | 64 MiB (`67,108,864` bytes) | `+1` byte부터 artifact read 실패 |
| replay record | 100,000 lines | 100,001번째 record 전에 실패 |
| replay 한 line | 65,536 UTF-8 bytes | `+1` byte부터 실패 |
| save event log | 100,000 events | `+1` event부터 `InvalidSave(ResourceLimit)` |
| save entity store | 100,000 entities | `+1` entity부터 `InvalidSave(ResourceLimit)` |
| RNG restore | 1,000,000 draws | `+1` draw부터 `InvalidSave(ResourceLimit)` |
| persisted message/rejection text | 512 UTF-8 bytes | C0/C1/DEL control 문자 또는 `+1` byte부터 `InvalidSave(InvalidText)` |
| headless target | `1..=1,000,000` accepted turns | Clap parse 단계에서 범위 밖 값을 거절 |

semantic validator는 최소한 schema, save seed와 RNG seed 일치, consumer-safe turn·score 산술, current level 존재, unique entity ID와 allocator 진행값, player 존재·종류·위치·map bounds, actor stat 범위, item 위치, inventory owner/중복/letter/location, equipped item의 class/location/derived AC, event text를 검사한다. persisted `ItemData`는 복원에 전달된 immutable `ContentRegistry`가 같은 `ItemKind`에 제공하는 typed 값과 정확히 일치해야 한다. allocator `next_id`는 persisted maximum entity ID의 `checked_add(1)`과 정확히 같아야 하며 임의 gap을 허용하지 않는다. allocation은 checked/fallible API이고 ID 고갈은 command transaction을 typed rejection으로 끝내며 원본 world/RNG/hash를 보존한다. persisted level ID 집합은 active registry의 `Main` depth 집합과 같아야 하며 registry depth는 `1..=i16::MAX-1`이다. stairs target depth는 checked arithmetic으로 계산한다. item의 dynamic `charges`와 registry `max_charges`는 optional shape가 같고, 값이 있으면 `charges <= max_charges`여야 한다. 실패는 panic이나 임의 문자열 성공이 아니라 typed `GameError::InvalidSave`다. injected content를 사용하는 복원은 같은 registry를 명시적으로 전달하며, v1 wire에는 registry 자체를 새 필드로 추가하지 않는다.

actor와 inventory의 저장 불변조건은 다음처럼 닫는다.

- 모든 actor는 `max_hp > 0`, `hp <= max_hp`, `alive == (hp > 0)`을 만족한다. `GameOver`가 아닌 save의 player는 살아 있어야 하며, 죽은 player는 `GameOver`여야 한다. 현재 `Quit`은 `DeathCause::Combat { attacker: EntityId(0) }` sentinel을 사용하는 호환 경로이므로 이 경우에만 살아 있는 player의 `GameOver`를 허용한다.
- 모든 `EntityLocation::Inventory { owner }` item은 `owner == player_id`이고 inventory index에 정확히 한 번 존재해야 한다. 반대로 모든 inventory entry는 동일 item location과 letter를 가리켜야 한다. v1은 monster/다른 actor inventory를 지원하지 않는다.
- persisted item의 class, glyph, weight, base price, AC/attack/effect/charge/nutrition 데이터는 복원 registry와 일치해야 한다. armor 적용 후 AC는 넓은 정수형의 checked arithmetic으로 계산하고 `i16` 범위와 persisted player AC가 모두 일치해야 한다. body armor를 착용하지 않은 player AC도 adventurer base AC와 정확히 일치해야 한다.
- `ContentRegistry::from_toml_sources`는 runtime consumer보다 먼저 item kind별 required/forbidden field shape와 numeric 범위를 검증한다. weight와 price는 음수가 아니어야 하고, food/corpse nutrition은 `1..=10,000`, armor AC bonus는 `0..=10,000`, live monster HP는 `1..=10,000`, level depth는 `1..=i16::MAX-1`이다. weapon만 melee slot·hit bonus·damage를, wand만 positive `charges`와 wand effect를, armor만 body slot·AC bonus를 가질 수 있다. armor의 `damage`/`hit_bonus`를 포함해 kind 전용이 아닌 필드는 거부한다. accepted registry bootstrap은 반환 전에 동일 persisted/save invariant를 통과해야 한다. accepted armor는 adventurer base AC에서 직접 derive하고 inventory에서 item을 제거하는 Drop/Throw/consume/read 경로는 공통 fallible unequip lifecycle을 사용하여 Wear→removal→save가 가역적이어야 한다.
- `turn`, gold, kill count, inventory value와 score 조합은 다음 정상 command·observation·Quit에서 좁은 정수 overflow나 wraparound를 만들지 않는 범위여야 한다. runtime의 turn 증가, kill count 증가, 전투·회복·무게·점수 산술도 malformed state가 내부 경계를 통과하더라도 wrapping하지 않는 widening 또는 saturating 정책을 사용한다.
- writer는 `to_save_data` 결과를 같은 semantic validator로 검사하고 pretty JSON을 16 MiB capped buffer에 직렬화한 뒤에만 destination을 원자 교체한다. 성공한 write는 같은 version loader로 즉시 읽을 수 있어야 하며, validation/byte 초과 실패는 기존 destination을 바꾸지 않는다.

## 12. Headless policy

```rust
pub enum HeadlessPolicy {
    WaitV1,
    SurvivalV1,
    ReplayFile,
}

pub struct HeadlessRunReport {
    pub seed: u64,
    pub policy: HeadlessPolicy,
    pub requested_turns: u64,
    pub accepted_turns: u64,
    pub submitted_commands: u64,
    pub final_state: RunState,
    pub final_hash: SnapshotHash,
}

pub enum HeadlessRunError {
    NoAcceptedAction { turn: u64, attempts: u8, submitted_commands: u64 },
    GameOver { turn: u64, submitted_commands: u64 },
    ReplayExhausted { turn: u64, submitted_commands: u64 },
}
```

성공 report는 `HeadlessRunReport`만 serialize한다. runner 실패 시 CLI는 같은 경로에 현재 session의 seed, accepted/submitted command 수, final state/hash와 `HeadlessRunError`를 포함한 failure report를 기록한 뒤 성공 exit로 처리하지 않는다.

`survival-v1` 규칙:

1. HP가 max HP의 50% 이하이고 healing potion legal이면 quaff
2. adjacent hostile에 대한 bump attack legal이면 공격
3. 현재 tile에서 pickup legal이면 pickup
4. 통과 가능한 방향을 North, East, South, West, NorthEast, SouthEast, SouthWest, NorthWest 순으로 선택
5. 이동이 모두 불법이면 Wait
6. GameOver가 되면 같은 seed의 새 session을 만들지 않고 report를 실패 처리

`--turns 1000` 성공은 `accepted_turns == 1000`일 때만 인정한다.

CLI의 canonical 기본 policy는 `survival-v1`이며 `--turns`는 `1..=1,000,000`만 허용한다. help, implicit-default 실행, BUILD 예시는 이 값을 공유한다.

`--turns` 상한은 실행 target 계약이며 모든 target에서 `--save` 성공을 보장하지 않는다. event history 또는 serialized payload가 위 save 예산을 넘으면 headless는 save 단계에서 typed resource error를 출력하고 exit code 2로 실패하며 기존 save를 보존한다. v0.3.0은 자동 event compaction으로 기록을 조용히 버리지 않는다.

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

NH367-C008 hunger projection은 3.6.7 `newuhs` 경계를 따른다: nutrition `<= 0`은 Fainting, `1..=50`은 Weak, `51..=150`은 Hungry, `151..=1000`은 NotHungry, `> 1000`은 Satiated다. FAINTED/STARVED와 메시지 전이는 v0.3.0 범위 밖이다.

## 14. 저장·replay 정책

- `SaveDataV1` schema_version은 1 유지
- 저장 직후와 load 직후 snapshot hash가 같아야 함
- replay line은 turn_before, command, outcome, snapshot_hash_after를 포함
- replay v1은 command-only log가 아니라 self-verifying artifact다. 소비자는 각 line의 `turn_before`, submit 결과 전체(`accepted`, `turn_advanced`, `events`, `snapshot_hash`, `next_state`), `snapshot_hash_after`를 실제 결과와 비교한다.
- replay 적용은 cloned working session에서 수행하고 소비한 prefix 전체가 일치할 때만 원 session을 교체한다. mismatch, exhaustion, GameOver를 포함한 실패는 원 session을 부분 변경하지 않으며 typed `ReplayMismatch`가 field와 line index를 제공한다.
- LLM narrative, rationale, soft verdict는 save와 replay truth에 포함하지 않음
- content_hash는 save metadata에 추가하지 않고 v0.3.0 load 시 현재 registry와 fixture test로 검증
- schema 필드 추가가 필요하면 v0.4.0 spec에서 SaveDataV2를 정의
- production artifact I/O는 마지막 root component를 no-follow로 연 directory capability와 정규화된 상대 경로를 결합한 단일 경계를 사용한다. `.` component를 제거하고 absolute/parent/root/prefix component를 거부한다. 기존 symbolic link, Windows junction/reparse root는 거절한다.
- save 임시 파일은 대상과 같은 directory에서 원자적 `create_new`로 생성하고, 일반 파일·single hard link를 handle 기준으로 확인한 뒤 sync와 atomic replace 수행
- Unix는 replace 후 parent directory handle도 `sync_all`하여 directory entry crash durability를 요청한다. Windows는 payload file sync와 atomic replace를 보장 범위로 두며 parent-directory metadata의 전원 손실 내구성은 filesystem/OS flush 정책에 따른 잔여 위험이다.
- Unix save/temp는 mode `0600`을 강제한다. Windows save/temp는 parent directory DACL을 상속하며 runtime이 owner-only DACL을 재작성하지 않으므로, Windows 기밀성 경계는 사용자가 선택한 runtime root의 ACL이다.
- replay 기록은 기존 payload를 bounded read·검증한 뒤 새 임시 파일로 atomic rewrite한다. 따라서 외부 inode를 직접 append하지 않으며 destination link 검증과 file sync를 save와 공유한다.
- `--replay-in`과 `--replay-out`은 정규화 상대 경로, Windows case-insensitive path, 열린 파일의 device/file identity 중 하나라도 같으면 동일 artifact로 판정해 실행 전에 거부한다. Windows 상대 경로 component는 trailing dot/space, ADS 구분자, 제어·금지 문자와 reserved device name을 받지 않아 compare/open/replace가 같은 이름 의미를 사용하게 한다.
- TUI quick-save는 실행별 `ArtifactStore`와 relative `quick-save.json`을 소유하며 caller가 전달한 absolute/parent path를 production 저장 경계로 사용하지 않는다. ambient `resolve_path_in_root` compatibility helper는 제거하고 production과 test 모두 `ArtifactStore` 경계를 사용한다.
- runtime root는 한 프로세스가 쓰는 사용자 전용 directory다. 같은 계정의 악성 프로세스가 root directory entry를 동시에 교체하거나 여러 writer가 같은 replay를 갱신하는 상황은 OS sandbox가 없는 v0.3.0의 비목표다. 다만 사전 배치 link/reparse와 외부 inode write는 fail-closed하며 atomic rewrite가 open-after-link hard-link write race를 제거한다.

현재 replay wire는 `TurnOutcome`을 직접 직렬화한다. `ReplayLineV1`은 turn_before, command, outcome, snapshot_hash_after를 보유하며, `outcome.accepted=false`는 거절/no-commit 결과를 나타낸다. v1 소비자는 이 모든 필드를 검증하되 wire field를 추가하거나 제거하지 않는다. canonical JSON fixture의 추가와 `Result<TurnOutcome, SubmitError>` projection은 후속 versioned schema에서만 정의한다.

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
| R7 | NetHack 호환성·출처 추적 | R3 | SC-COMPAT-01, provenance inventory/checksum/legacy 격리 evidence; SC-LICENSE-01은 R8로 defer 가능 |
| R8 | v0.3.0 통합 감사·런칭 승인 | R6, R7 | SC-LICENSE-01을 포함한 전체 SC와 문서 gate PASS |

세부 Task와 파일 단위는 `IMPLEMENTATION_SUMMARY.md`가 정의한다.

R7 provenance validator는 runtime asset과 NH367 scenario의 승인 근거를 판정하며 root `Cargo.toml`의 배포 라이선스는 검사하지 않는다. R8 release gate는 workspace 전체 `NGPL`, version 0.3.0, 공식 `LICENSE` checksum, owner approval ID, `NOTICE`, modification manifest, expanded commit metadata, checksums와 source archive 계약을 최종 검증한다. 라이선스 승인이 완료돼도 R8 기술 감사 PASS 전에는 release artifact를 외부 게시하지 않는다.

release `output/` directory 전체가 게시 bundle이다. build는 workspace 내부의 예측 불가능한 새 staging directory에 create-new 방식으로 bundle을 완성하고 검증한 뒤 directory 단위로 `output/`에 승격한다. 기존 output root가 symbolic link, junction 또는 다른 reparse 경계면 쓰기 전에 실패하며, 기존 expected-name hard link inode를 직접 덮어쓰지 않는다. platform verifier는 no-follow root와 각 expected file의 single-link 상태를 확인하고 top-level actual entry 집합을 선언된 binary, 문서, metadata, source archive, `SHA256SUMS`의 exact set과 비교한다. extra file, directory, symbolic link, hard link 또는 Windows reparse point는 bundle 실패다. source archive는 ZIP/TAR를 공통 format-aware parser로 읽어 raw entry name, regular-file/directory type, link target 부재와 extraction prefix를 검사한다. 각 component는 C0/C1/DEL과 Windows 금지문자, trailing dot/space, `CONIN$`/`CONOUT$`, classic 및 superscript COM/LPT device, Unicode normalization·casefold collision을 거부하고 file-vs-directory prefix 충돌과 excluded canonical root를 fail-closed한다. symlink, hardlink, device, FIFO와 알 수 없는 type은 허용하지 않는다. 검증된 entry만 임시 root에 추출해 exact path/content manifest를 대조한다. 최종 archive byte는 `ExpectedCommit`에서 같은 format으로 독립 재생성한 `git archive`와 같아야 하므로 path, mode/type, `export-ignore`, `export-subst`, blob content의 complete set 중 omission·substitution·safe extra도 거부한다. `RELEASE-METADATA`의 `candidate_date`는 exact commit에서 생성되고 bundled modification period 안에 포함되어야 하며 양 OS 모두 year `0001..9999`의 canonical Gregorian date만 수용한다.

## 16. 보안 및 구현 경계

- LLM endpoint는 loopback 기본이며 remote host는 v0.3.0에서 거부
- prompt에 save 경로, 환경변수, 파일 내용, 비밀정보를 넣지 않음
- LLM response를 로그에 남길 때 최대 240 chars로 제한
- content 파일은 embedded read-only이며 실행 중 임의 경로를 읽지 않음
- save/replay/report path는 열린 runtime root capability 아래의 상대 경로로만 처리하고 absolute path, 상위 디렉터리 탈출, root 밖 symbolic link를 거부
- save/replay 쓰기는 no-follow open, 일반 파일·single hard-link handle 검증, 충돌 없는 `create_new` 임시 파일을 강제하며 검사 후 bare path를 다시 열지 않음
- Windows에서 다른 principal의 읽기를 차단해야 하는 배포는 사용자 전용 application directory를 runtime root로 사용해야 하며, 일반 workspace root를 owner-only 저장소로 간주하지 않음
- `unsafe` 도입은 별도 ADR과 Miri 검증 없이는 금지
- 레거시 라이선스 파일은 수정하지 않고 공식 원문과 checksum으로 교체 계획을 별도 기록

## 17. 잔여 리스크

| 리스크 | 영향 | 구현을 막지 않는 완화 |
| --- | --- | --- |
| NetHack 파생물 배포 의무 누락 | 높음 | whole-work NGPL, 공식 LICENSE, NOTICE, complete corresponding source와 R8 fail-closed gate 적용 |
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

## 19. R9 후속 목표: 콘텐츠 인과 폐쇄

작성일 2026-08-17 기준 최신 사용자 요구에 따라, v0.3.0 릴리스 기준을 훼손하지 않는 후속 구현 단계 R9를 추가한다. 현재 Cargo version은 R9 완료와 릴리스 결정 전까지 0.3.0을 유지한다.

진행 상태: 2026-08-17 R9-1..R9-5 표적 인과 루프를 구현했고 report 25 시정에서 동일 world/turn의 gold/no-gold production score pair까지 완료했다. report 26의 최종 predecessor는 SHA `1e84a94`/Actions `32660514315`다. report 27이 omission branch가 command/observer를 생략한 점을 재개방하여, 9종 모두 동일 flow에서 대상 field/state만 neutralize하고 나머지 8개 full record equality를 검사하는 matrix로 교체했다. SHA `ea7822a5`/Actions `32683076204`에서 전체 gate와 clean same-SHA 양 OS actual bundle까지 Verified됐으며, 후속 독립 재감사 전까지 전체 R9/program은 HOLD다. `hallucinating`은 SaveDataV1 호환성 orphan으로 명시적 제외하며 owner는 Project owner/runtime maintainer, 재검토 시점은 SaveDataV2·v0.4.0 범위 승인 또는 2026-10-31 중 먼저 도래하는 때다.

### 19.1 목표

- 주요 콘텐츠마다 생성 원인, 소비 주체, 직접 상태 변화, 후속 영향을 추적한다.
- 다른 시스템의 원인도 결과도 되지 않는 orphan content를 제거하거나 실질적 simulation 경로에 연결한다.
- embedded content 값이 kind 기반 하드코딩에 가려지지 않고 runtime behavior의 진실원이 되게 한다.
- 장기 테스트는 함수 호출이나 이벤트 존재가 아니라 관찰 가능한 semantic world state delta를 증명한다.

### 19.2 인과 PASS 계약

인과 루프 하나는 다음 조건을 모두 만족해야 PASS다.

1. 동일 seed와 초기 상태에서 원인 입력을 재현할 수 있다.
2. producer가 콘텐츠 또는 세계 상태를 생성·변경한다.
3. 별도 consumer가 그 값을 규칙 판정의 입력으로 사용한다.
4. 명령 전후 snapshot에서 turn, event count, last event를 제외한 semantic field가 변한다.
5. 그 변화가 후속 행동의 legality, 위치, HP, AC, nutrition, gold, score, run state, entity lifecycle 중 하나 이상에 다시 영향을 준다.
6. 같은 seed와 command sequence의 반복 실행은 같은 witness와 final hash를 만든다.

이벤트만 추가되거나 코드가 호출되기만 한 경우, 또는 turn 증가만 있는 경우는 FAIL이다.

### 19.3 초기 orphan register

상세 근거와 함수 경로는 `docs/audit/audit_report_22.md`를 따른다. 초기 수정 대상은 item `nutrition`/food/corpse, `ac_bonus`, `base_price`/gold, monster `ai`/`passive`/`speed`/`difficulty`, production producer가 없는 luck/hallucination이다.

### 19.4 검증 기준

| ID | 기준 |
| --- | --- |
| SC-CAUSE-01 | active content와 world state의 인과 매트릭스에 생성 원인·소비 주체·직접 delta·후속 영향이 기록됨 |
| SC-CAUSE-02 | 지원한다고 선언한 content behavior field가 runtime typed data에 투영되고 A/B registry test에서 다른 semantic delta를 생성 |
| SC-CAUSE-03 | 음식/시체가 nutrition과 hunger 후속 전이를 실제 명령 경로에서 변경 |
| SC-CAUSE-04 | 가격/경제, luck, hallucination은 producer-consumer 루프를 갖거나 명시적으로 비목표·제거 대상으로 닫힘 |
| SC-CAUSE-05 | seed 42, 7, 1234 각각 1000 accepted turn 이상의 장기 simulation에서 필수 causal witness가 모두 1회 이상 발생 |
| SC-CAUSE-06 | 각 seed를 3회 실행한 witness summary와 final hash가 모두 동일 |
| SC-CAUSE-07 | causal regression은 event-only 또는 turn-only 변화에서 실패 |

`SC-CAUSE-05..07`의 필수 typed witness 집합은 `FoodNutrition`, `CorpseNutrition`, `ArmorDefense`, `MonsterSpeed`, `MonsterAi`, `MonsterPassive`, `MonsterDifficultyEconomy`, `PrayerLuckCombat`, `GoldScore` 9개다. 일반 사용자 정책 `survival-v1`은 변경하지 않고, 테스트 전용 deterministic `causal-v1` fixture가 같은 `GameSession::submit` 경로에서 각 원인 명령과 downstream delta를 만든 뒤 absolute turn 1000까지 진행하고 마지막 score projection을 닫는다. 각 witness record는 scenario ID, producer entity/item, 원인 content field와 before/after value, consumer delta를 보유한다. 9개 isolation 회귀는 active/control 양쪽에서 동일 producer command, consumer command와 observer를 실행하고 대상 content field 또는 producer state 하나만 변경한다. omission은 observer 호출이나 Eat/Wear/Pray/attack/kill/Quit 명령을 생략해서 만들지 않는다. difficulty pair도 양쪽에서 같은 kill을 수행하고 difficulty 차이와 gold delta 차이를 대조한다. 각 omission run은 정확히 대상 witness 하나만 잃고 나머지 8개 record의 전체 attribution 값이 complete run과 같아야 한다. `MonsterSpeed`는 speed budget에 의해 실행 기회가 달라진 경우에만, `MonsterAi`는 동일 speed에서 AI 선택이 달라진 경우에만 기록한다. `GoldScore`는 동일 world/turn에서 gold만 제거한 paired score와 실제 final score의 차이가 정확히 gold와 같을 때만 기록한다. witness는 명령·event 이름만으로 기록하지 않으며 command 전후 `CausalProjection`의 실제 semantic field가 함께 변한 경우에만 집계한다.

각 seed의 완료 증거는 9개 witness count map과 final hash다. seed 42, 7, 1234를 각각 3회 반복했을 때 두 값이 모두 같아야 한다. 동일 projection을 전후 값으로 넣은 event-only fixture와 turn만 증가한 projection은 실패해야 한다. 누락 negative는 완성 summary의 record/count를 사후 삭제하지 않고 각 producer command, content field 또는 paired production scenario를 실행 전에 하나씩 제거한 9-case full run이어야 하며, 해당 witness만 빠지고 나머지 required witness가 유지되는지를 검증한다.

### 19.5 구현 순서

R9-1은 semantic delta/witness 테스트 기반, R9-2는 음식·영양·시체, R9-3은 content behavior projection, R9-4는 경제·점수, R9-5는 상태 orphan 폐쇄, R9-6은 3 seed 장기 회귀와 전체 감사다. 각 단계는 RED 테스트를 먼저 만들고 해당 단계의 workspace build/test가 통과한 뒤 다음 단계로 진행한다.
