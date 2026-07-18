# AIHack Design Decisions v2

> Archive chain
> - Latest: `.archive/DESIGN_DECISIONS_archive_260715.md`
> - Previous: first archive
>
> ADR-0001..ADR-0020과 과거 중복 번호는 아카이브에 보존한다. 이 파일은 v0.3.0 활성 결정만 포함한다.

문서 상태: active decisions
작성일: 2026-07-15
기준: `spec.md`

Accepted는 계획 승인을 뜻하며 구현 완료를 뜻하지 않는다. 아카이브의 과거 결정과 충돌하면 이 파일과 `spec.md`를 적용한다.

## ADR-0021: NetHack 3.6.7 행동 호환 clean reimplementation

Status: Implemented (2026-07-15)
Date: 2026-07-15
Decision ID: DEC-PRODUCT-01

Context:

사용자의 제품 목표는 NetHack 3.6.7을 Rust로 재구성하고 local LLM을 메시지와 판정에 사용하는 것이다. ADR-0005의 “NetHack-inspired 독립 게임”은 이 목표를 정확히 표현하지 않으며, 줄 단위 포트는 기존 legacy 구조 문제와 출처 위험을 되살린다.

Decision:

AIHack은 NetHack 3.6.7의 관찰 가능한 행동을 compatibility scenario로 재구현한다. 구현 단위는 source locator, precondition, typed command, expected event/state, Rust test를 갖는다. 원본 C control flow, symbol layout, 문자열, 데이터 테이블을 그대로 번역하거나 복사하지 않는다. v0.3.0은 NH367-C001..C010만 release scope로 고정한다.

Alternatives Considered:

- NetHack-inspired 독립 게임 유지: 사용자의 3.6.7 변환 목표와 어긋나므로 기각
- C 소스 자동 변환: Rust 경계·안전성·유지보수성과 provenance가 악화되어 기각
- full 3.6.7 parity를 v0.3.0에 포함: 검증 가능한 종료 조건이 없어 기각

Consequences:

- ADR-0005의 제품 범위는 이 ADR로 대체된다.
- compatibility ID 없는 NetHack 규칙 변경은 release에 포함하지 않는다.
- full parity는 이후 milestone로 남으며 v0.3.0 완료 조건이 아니다.
- R7 provenance와 compatibility gate가 필수가 된다.

## ADR-0022: Rust 1.94.1과 단일 UI dependency 계열 고정

Status: Accepted; UI dependency 선택은 ADR-0028로 대체됨
Date: 2026-07-15
Decision IDs: DEC-RUST-01, DEC-UI-DEP-01

Context:

R1 시작 전에는 빌드가 통과하지만 repository toolchain과 MSRV가 없고, ratatui 0.30이 crossterm 0.29를 추가하여 direct crossterm 0.28과 공존했다. 두 binary에 default-run도 없었다.

Decision:

`rust-toolchain.toml` channel을 1.94.1로, Cargo rust-version을 1.94로 고정한다. Cargo 자동화에는 `--locked`를 사용하고 default-run은 `aihack`이다. UI dependency 선택은 ADR-0028을 따른다.

Alternatives Considered:

- ratatui 0.30과 crossterm 0.29로 동시 상향: 당시에는 회귀 범위가 불명확하여 보류했으나 RustSec advisory 확인 후 ADR-0028에서 채택
- dependency duplicate 허용: event/key type 혼선과 build drift가 남아 기각
- stable 최신을 매 실행 사용: 재현 불가능하여 기각

Consequences:

- R1에서 lockfile 변경과 UI compile regression을 한 번 검증한다.
- toolchain upgrade는 별도 ADR과 CI matrix 변경이 필요하다.
- R1 이후 quick start는 `default-run = "aihack"`을 사용한다.

## ADR-0028: RustSec 경고 없는 ratatui 0.30/crossterm 0.29 계열

Status: Accepted; R1 local verification complete, remote CI pending
Date: 2026-07-15
Decision ID: DEC-UI-DEP-01

Context:

ADR-0022의 ratatui 0.29 계열은 crossterm 중복을 피했지만, 현재 RustSec advisory에서 필수 dependency `lru 0.12.5`의 memory-corruption unsound 문제와 `paste 1.0.15`의 unmaintained 상태가 확인됐다. `lru`는 ratatui 0.29의 비선택 dependency이며 수정된 버전은 0.16.3 이상이다.

Decision:

UI dependency를 `ratatui = "0.30"`과 `crossterm = "0.29"`로 함께 올린다. lockfile은 ratatui 0.30.2, crossterm 0.29.0, lru 0.18.1을 고정하며 `cargo audit`, `cargo deny check licenses bans sources`, crossterm 단일 버전 검증을 R1 gate에 포함한다.

Alternatives Considered:

- ratatui 0.29 유지와 RustSec 예외: memory-corruption advisory와 유지보수 중단 dependency를 release baseline에 남기므로 기각
- lru만 단독 상향 또는 feature 제거: ratatui 0.29가 요구하는 API/의존성 제약과 맞지 않아 기각
- ratatui 0.29 fork/patch: 보안 수정의 장기 유지 책임을 프로젝트가 떠안으므로 기각

Consequences:

- R1 lockfile diff와 full test/UI compile regression을 검증한다.
- 기존 UI API가 바뀔 경우 최소 호환 수정과 회귀 테스트를 같은 Task에 포함한다.
- R1의 SC-BUILD-02는 Linux/Windows 원격 CI가 green일 때만 PASS다.

## ADR-0023: private state와 transaction/invariant commit

Status: Implemented (2026-07-15)
Date: 2026-07-15
Decision IDs: DEC-STATE-01, DEC-RUNTIME-01, DEC-RNG-01

Context:

`GameSession`과 `GameWorld`의 mutable field가 공개되어 UI, LLM, test가 command validation과 turn semantics를 우회할 수 있다. submit 내부 mutation 중 오류가 나면 부분 state와 RNG draw rollback 계약도 없다.

Decision:

`GameSession`을 유일한 mutable owner로 유지하되 모든 field를 private으로 만든다. 외부 read는 session getter, `Observation`, snapshot query로 제한한다. accepted command는 `TurnTransaction`의 prepare → apply → 6 invariant validate → atomic commit 순서를 따른다. 거절과 invariant failure는 `accepted=false` outcome으로 표현하며 world, event log, turn, RNG draw를 모두 보존한다.

Alternatives Considered:

- public field + coding convention: compiler가 우회 경로를 차단하지 못해 기각
- interior mutability를 광범위하게 사용: borrow 오류를 runtime으로 이동시켜 기각
- ECS로 전환: 상태 캡슐화와 behavior preservation보다 범위가 커 기각

Consequences:

- integration test는 `tests/support/session_builder.rs`를 사용한다.
- R2에서는 hash field order와 게임 공식 변경을 허용하지 않는다.
- invariant 오류는 no-commit `accepted=false` result가 된다.
- `GameClient`, revision, typed submit error는 R5 workspace boundary와 R6 stale-response gate에서 함께 도입한다.

## ADR-0024: embedded TOML ContentRegistry가 runtime 데이터 원천

Status: Implemented (Re-audit #2, 2026-07-16)
Date: 2026-07-15
Decision ID: DEC-CONTENT-01

Context:

기존 TOML loader는 data test에서만 사용되고 runtime factory와 level 생성은 hardcoded 값에 의존했으며, invalid data의 일부는 `expect` 또는 panic으로 끝났다.

Decision:

items, monsters, levels TOML을 build에 embed하고 process 시작 시 한 번 parse/validate하여 immutable `ContentRegistry`를 만든다. runtime factory는 ID로 registry를 조회한다. duplicate ID, unknown reference, invalid dice/coordinate, unpaired stairs, unsupported schema는 `ContentError`로 반환한다. canonical content hash는 정렬된 schema v1 데이터의 FNV-1a 64-bit다.

Alternatives Considered:

- hardcoded Rust 상수 유지: 데이터와 test의 이중 원천이 남아 기각
- 매 access마다 TOML parse: 비용과 오류 시점이 불안정해 기각
- 외부 mutable data directory: v0.3.0 배포·replay 재현성을 낮춰 기각

Consequences:

- content schema/version과 hash가 save/replay compatibility metadata가 된다.
- invalid embedded data는 게임 시작 실패이며 fallback hardcoded data를 쓰지 않는다. TUI/headless production bootstrap은 R3-4에서 fallible `ContentError` 경계로 전환됐고, injected missing level/item regression test가 이를 고정한다. legacy infallible fixture adapter는 production startup 경계가 아니다.
- R3에서 현재 TOML 값의 provenance도 함께 조사한다.

## ADR-0025: core/content/AI/adapter workspace 경계

Status: Implemented and verified by `audit_report_6.md`
Date: 2026-07-15
Decision ID: DEC-WORKSPACE-01

Context:

단일 package에 core, TOML, TUI, LLM scaffold, 두 binary가 있어 dependency upgrade와 compile failure의 영향 범위가 넓다. mutable core type의 노출도 adapter 경계를 약화한다.

Decision:

R1~R4 behavior gate가 통과한 뒤 `aihack-core`, `aihack-content`, `aihack-ai-contract`, `aihack-llm`, `aihack-runtime`, `aihack-tui`, `aihack-headless` workspace로 분리한다. runtime은 core와 content의 조합, content bootstrap, command 실행 및 저장 경계를 소유하고 `GameClient`만 adapter에 노출한다. core는 serde, thiserror, rand만 허용하며 UI와 HTTP dependency를 갖지 않는다. binary 이름과 CLI는 유지한다.

Alternatives Considered:

- 단일 package 유지: dependency와 public API 경계가 compile-time에 보장되지 않아 기각
- workspace를 먼저 수행: behavior bug와 file move regression을 구분하기 어려워 기각
- plugin/dynamic library: 배포 복잡도와 ABI 위험이 커 기각

Consequences:

- R5는 mechanical move만 수행하며 hash 변경을 허용하지 않는다.
- crate public API는 `GameClient`, DTO, registry constructor 중심으로 최소화한다. runtime을 두어 adapter가 core/session 구현에 직접 의존하지 않게 한다.
- file move는 Task당 5개 이하로 나눈다.

## ADR-0026: local LLM은 loopback presentation adapter

Status: Accepted; audit report 11 independent R6 PASS
Date: 2026-07-15
Decision IDs: DEC-AI-01, DEC-LLM-01, DEC-LLM-02

Context:

R6 시작 시 narrative/decision module은 provider trait와 mock만 있었고 실제 transport, 강제 timeout, request와 current session의 correlation이 없었다. 사용자는 local LLM을 메시지 생성과 판정에 쓰려 하지만 core 결정론을 잃으면 빌드·재현 문제를 더 악화시킨다.

Decision:

기본 endpoint는 loopback OpenAI-compatible HTTP이며 기본 enabled는 false다. `reqwest 0.13.4` blocking/json client를 전용 worker thread 1개와 capacity 16 bounded channel 안에 격리한다. connect 500ms, narrative 2000ms, decision 1500ms를 transport가 강제한다. redirect와 system proxy는 끈다. request는 request ID와 `SessionRevision { turn, snapshot_hash }`를 포함한다. narrative와 soft verdict는 presentation-only다. suggestion은 current `ActionSpace`와 revision을 재검증하고 사용자의 `Y` 승인 뒤 normal submit path를 사용한다.

Alternatives Considered:

- LLM이 자유 텍스트로 state patch 반환: 무결성·보안·replay 문제로 기각
- LLM 응답을 core turn에서 동기 대기: provider 장애가 gameplay를 중단시켜 기각
- remote endpoint 기본 허용: privacy와 운영 의존성이 늘어 기각
- soft verdict가 능력치 modifier를 반환: 사실상 core 판정권이 되어 기각

Consequences:

- provider가 없어도 전체 core 게임이 동작한다.
- timeout/invalid/stale response는 hash를 바꾸지 않는다.
- prompt와 response body를 save/replay에 기록하지 않는다.
- remote provider는 v0.3.0 비목표다.
- R6-1은 연결 직전 resolve 결과를 재검사하고 검증된 loopback 주소를 client에 고정한다. R6-2는 opaque request ID, current revision/ActionSpace, submit 직전 revision을 연속 검증해 response-validation 사이의 stale gap도 막는다.
- R6-3은 strict soft payload와 `Neutral / LLM_UNAVAILABLE` fallback을 UI-only state로 보관하고, terminal 복원 뒤 worker를 최대 250ms만 정리한다.
- R6 통합은 G/A/J 요청과 Y/N/R 안전 경로, 상태·modal, 동일 종류 outstanding·250ms cooldown, capacity 16 oldest-drop 표시 큐를 실제 TUI loop에 연결한다. 자동 failure matrix와 live PTY/loopback fixture matrix를 통과했고 `audit_report_11.md`가 checkpoint를 PASS로 종결했다.
- public request는 `schema_version = 1`, `SessionRevision`, `LlmObservationView`, 독립 `ActionSpace`, `LlmRequestKind`를 소유하고 enqueue 전에 version·bounds·canonical size를 검증한다. response envelope도 TUI payload 수용 전에 version을 거부한다.
- deterministic loopback fixture와 PTY script는 success/timeout/stale/down 및 pending-exit 복원 순서를 저장소에서 재현한다. `audit_report_11.md` 독립 재감사에서도 같은 evidence가 통과했다.
- 실제 모델 smoke는 R6 필수 gate가 아니다. 최종 통합에서 별도 호환성 증거가 필요할 때만 localhost OpenAI-compatible 임시 adapter가 Google AI Studio Gemini 같은 원격 API를 대리 호출한다. AIHack은 계속 loopback만 호출하며 API key는 adapter 환경변수에만 주입하고 model ID는 실행 시점에 확인한다.

## ADR-0027: provenance approval이 runtime 포함의 선행 조건

Status: Accepted for v0.3.0 plan; implementation pending
Date: 2026-07-15
Decision ID: DEC-LICENSE-01

Context:

legacy tree에는 Apache-2.0 text와 NGPL text가 함께 있고 적용 범위가 명시되지 않았다. `LICENSE.NGPL` 33..35행은 반복 단어로 손상되어 있다. root package는 UNLICENSED이며 현재 TOML 값의 출처도 완전히 기록되지 않았다.

Decision:

`PROVENANCE.md`의 상태를 Unknown, Reviewed, Approved, Blocked로 고정한다. runtime에는 Approved 자산만 포함한다. legacy code/data/string은 기본 Blocked이며 path import와 복사를 금지한다. 공식 NetHack 3.6.7 archive는 공식 SHA-256을 확인한 뒤 source locator로만 사용한다. 배포 라이선스 결정은 프로젝트 소유자 또는 적격 검토자의 승인을 요구한다.

Alternatives Considered:

- legacy의 Apache file을 전체 tree license로 간주: 적용 notice 증거가 없어 기각
- 손상된 NGPL을 그대로 배포 notice로 사용: 원문 신뢰성이 없어 기각
- 코드만 새로 쓰면 provenance 생략: 데이터·문구·규칙 표현의 출처 위험이 남아 기각

Consequences:

- R7 전에는 release artifact 외부 게시를 중단한다.
- 공식 source metadata 확인은 재사용 승인과 동일하지 않다.
- compatibility record마다 source locator와 provenance status가 필요하다.
- 이 ADR과 inventory는 법률 자문을 대체하지 않는다.
