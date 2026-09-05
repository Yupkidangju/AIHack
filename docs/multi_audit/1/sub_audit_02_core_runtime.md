# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: core_runtime (핵심 도메인·런타임·아키텍처·데이터 무결성)
- User Goal: 현재 프로젝트의 문서와 구현을 대조하고 모순·문제점을 증거 기반으로 진단한다.
- Audit Basis: Standard-backed
- Standard Path: C:\LocalDev\rust\AIHack\AI_AUDIT_DOC_STANDARD.md
- Multi-audit Contract: C:\Users\temp\.codex\skills\multi-audit\references\report-contract.md
- Report Path: C:\LocalDev\rust\AIHack\docs\multi_audit\1\sub_audit_02_core_runtime.md
- 실행 환경: Windows, PowerShell, 2026-08-23, workspace package version 0.3.0

## 2. Assigned Scope

다음 경계를 독립적으로 확인했다.

- crates/aihack-core: 상태·도메인 타입, invariant, RNG, save DTO, entity store primitive
- crates/aihack-content: embedded TOML registry, schema validation, core 변환
- crates/aihack-runtime: GameSession, GameWorld, transaction, systems, snapshot/observation, save/replay, causal projection
- root aihack compatibility facade와 integration-test fixture 경계
- apps/aihack-headless의 production GameClient·replay runner·artifact loading 경로
- headless/TUI가 fixture 또는 compatibility facade를 우회하는지 여부
- GameSession transaction, invariant, save/load/replay, entity lifecycle, content causality, 오류·경계값·명백한 비제한 동작
- 파일 저장 및 데이터 무결성 중 runtime loader/replay 의미 검증에 해당하는 부분

## 3. Excluded and Uninspected Scope

- 다른 multi-audit 원본 보고서는 읽지 않았다. 부모 결론이나 다른 agent의 finding도 가정하지 않았다.
- legacy_nethack_port_reference/의 reference body, target/, 생성 runtime 산출물은 제외했다.
- TUI 시각 품질·레이아웃, LLM transport/worker 세부 구현, remote CI의 실제 실행 상태는 제외했다.
- save/replay 경로의 symlink, hard-link, ACL 및 atomic replace의 전체 보안 판정은 별도 file_security 관점의 범위다. 이 보고서는 그 경계가 제공하는 파일을 runtime이 의미적으로 어떻게 신뢰하는지만 독립 확인했다.
- release build와 remote CI를 이 turn에서 재실행하지 않았다. 로컬 workspace compile/test/clippy/fmt와 production CLI probe를 사용했다.
- 소스, 테스트, 설정, 제품 문서는 수정하지 않았다. 지정된 본 보고서만 생성했다.

## 4. Evidence Examined

### 통제 문서

- spec.md: 상태·transaction·workspace·content·save/replay·R9 causal 계약(Sections 1, 3, 5, 6, 8, 9, 11.5, 12, 14, 19)
- designs.md: runtime 구조, content causality, 오류/저장 경계, 검증 명령
- DESIGN_DECISIONS.md: ADR-0023(private state/transaction), ADR-0024(ContentRegistry), ADR-0031(causal closure), ADR-0032(capability save/replay)
- IMPLEMENTATION_SUMMARY.md: R2~R5와 R9의 완료 주장·파일 책임·검증 기준
- audit_roadmap.md: R2 상태 무결성, R3 content, R4 long-run/replay, R5 workspace, R9 SC-CAUSE gate
- BUILD_GUIDE.md, README.md, CHANGELOG.md: production 명령·artifact·상태 주장과 compatibility facade 설명

### 구현·테스트

- crates/aihack-core/src/{session.rs,world.rs,invariant.rs,rng.rs,save.rs,domain/entity.rs,domain/inventory.rs}
- crates/aihack-content/src/{lib.rs,schema.rs} 및 embedded TOML
- crates/aihack-runtime/src/{session.rs,transaction.rs,world.rs,bootstrap.rs,save.rs,snapshot.rs,observation.rs,causal.rs}
- crates/aihack-runtime/src/systems/{items.rs,death.rs,monster_ai.rs,combat.rs,projectiles.rs}
- crates/aihack-runtime/src/domain/{entity.rs,item.rs,monster.rs}
- apps/aihack-headless/src/{lib.rs,main.rs}
- src/{lib.rs,core/mod.rs,core/session.rs,data/mod.rs,testing.rs}
- tests/{workspace_boundaries.rs,transaction.rs,save_load.rs,replay.rs,headless_policy.rs,causal_content.rs,long_run.rs,content_validation.rs,world_invariants.rs,items.rs,inventory.rs,ui_screens.rs}

### 실행 증거

| 명령/Probe | 결과 |
| --- | --- |
| cargo metadata --no-deps --format-version 1 | exit 0; runtime은 core/content/AI contract에 의존하고 TUI/headless manifest는 core를 직접 의존하지 않음 |
| cargo fmt --all -- --check | exit 0 |
| cargo clippy --workspace --all-targets --locked -- -D warnings | exit 0 |
| cargo test --workspace --all-targets --locked | exit 0; workspace 전체 test green |
| cargo test --locked -p aihack-runtime --all-targets | exit 0 |
| cargo test --locked -p aihack-content --all-targets | exit 0 |
| cargo test --locked -p aihack --test transaction --test save_load --test replay --test causal_content --test long_run --test workspace_boundaries | exit 0; transaction/save/replay/causal/long-run/boundary 표적 green |
| valid save 생성 후 world.player_id=999로 변조하고 aihack-headless --load ... --turns 1 | exit 101; crates/aihack-runtime/src/world.rs:136의 player 위치 expect panic |
| valid save의 두 번째 entity ID를 player ID 1과 중복시킨 뒤 동일 load | exit 0; 구조적으로 모호한 entity save를 accepted report로 수용 |
| valid replay line의 turn_before, outcome.accepted, inner/outer hash를 모두 변조하고 --policy replay-file | exit 0; 위조 line을 거부하지 않고 실제 실행 hash로 성공 report 생성 |

마지막 세 probe의 임시 artifact 디렉터리는 생성 경로를 확인한 뒤 삭제했다.

## 5. Pass 1: Implementation Compliance Findings

### [IMP-CORE-F001] DerefMut가 private session/world 계약을 외부에서 우회시킴

- Pass: Implementation Compliance
- Pattern: IMP-003, ARCH-001
- Area: GameSession/GameWorld/entity workspace boundary
- Severity: Major
- Status: Needs Fix / Confirmed
- Summary: 문서와 ADR은 GameSession·GameWorld의 mutable field를 private으로 만들고 submit만 mutation entry로 허용한다고 선언하지만, runtime wrapper가 public generic state로 Deref/DerefMut된다.
- Evidence:
  - spec.md:40,89,214-233은 private field와 submit mutation 경계를 명시한다.
  - crates/aihack-runtime/src/session.rs:31-46은 GameSession의 pub(crate) inner를 SessionState<GameWorld>로 DerefMut한다.
  - crates/aihack-core/src/session.rs:5-11의 SessionState field가 모두 pub (meta, rng, turn, state, world, event_log)다.
  - crates/aihack-runtime/src/world.rs:31-46은 GameWorld를 public field의 WorldState<EntityStore>로 DerefMut한다. crates/aihack-core/src/world.rs:10-25의 world field도 모두 pub다.
  - crates/aihack-runtime/src/domain/entity.rs:54-65도 EntityStore를 core store로 DerefMut하며, core Entity/EntityPayload는 public mutation surface를 가진다.
  - 현재 workspace_boundaries와 R2 grep은 직접 대입이 저장소에 존재하지 않음을 검사할 뿐, 외부 crate에서 autoderef field access가 가능한지 검사하지 않는다.
- Expected Basis: spec.md DEC-STATE-01/SC-CORE-01, ADR-0023, designs.md의 UI·LLM read-only 경계. runtime 소비자는 GameClient/getter/Observation만 사용해야 한다.
- Actual: runtime crate 소비자는 타입을 명시적으로 이름 붙이지 않고도 session.state, session.turn, session.rng, session.world.nutrition, session.world.entities.get_mut(...) 같은 경로로 transaction 밖의 mutation을 만들 수 있다. 이 경로는 event log, RNG/revision, invariant, save/replay truth를 함께 갱신하지 않는다.
- Impact: LLM/TUI가 실수로 또는 새 adapter 추가 시 accepted-turn 계약을 우회할 수 있고, snapshot hash와 실제 RNG/world가 불일치하거나 invariant가 검증되지 않은 상태가 commit·save된다. R2/R5 workspace PASS 주장이 type-level private 보장을 과대주장한다.
- Suggested Action: GameSession, GameWorld, EntityStore의 Deref/DerefMut를 제거하고 SessionState/WorldState field를 crate-private 또는 private으로 낮춘다. runtime 내부 system만 사용할 수 있는 제한된 accessor를 만들고 외부에는 immutable projection/typed command만 공개한다. external consumer compile-fail/API surface test로 직접 대입과 get_mut 접근 불가를 고정한다.
- Re-audit Method: 별도 임시 consumer crate에서 session.turn = ..., session.world.nutrition = ..., session.world.entities.get_mut(...)가 컴파일되지 않는지 확인하고, cargo test --workspace --all-targets --locked와 transaction/revision 회귀를 다시 실행한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: production app 현재 코드는 GameClient만 사용한다는 점은 확인했지만, public API 자체가 경계를 강제하지 못하므로 “현재 호출자가 우회하지 않는다”는 완화는 충분하지 않다.

### [IMP-CORE-F002] causal witness가 서로 다른 content 원인을 구분하지 않아 R9 false-green이 가능함

- Pass: Implementation Compliance
- Pattern: IMP-003, TEST-001
- Area: content causality / completion evidence
- Severity: Major
- Status: Needs Fix / Confirmed
- Summary: R9 계약은 producer·consumer와 실제 content 값의 semantic delta를 요구하지만, causal summary는 어떤 monster가 어떤 값으로 움직였는지 기록하지 않고 한 이동을 MonsterSpeed와 MonsterAi 양쪽 witness로 동시에 센다.
- Evidence:
  - spec.md:768-779은 content producer, 별도 consumer, semantic field delta, 후속 영향, 결정론을 모두 요구하며 event/turn-only를 FAIL로 정의한다.
  - crates/aihack-runtime/src/causal.rs:250-263은 살아 있고 speed>0이며 stationary가 아닌 monster 하나가 위치를 바꾸기만 하면 MonsterSpeed와 MonsterAi를 모두 기록한다. entity ID, content source, before/after speed·AI 값, 해당 값이 선택에 기여했다는 증거가 없다.
  - tests/long_run.rs:75-83은 각 witness의 count > 0만 확인한다. run_causal_fixture의 154-161도 임의의 이동이 발생하면 두 witness를 충족한 것으로 간주한다.
  - tests/causal_content.rs의 A/B 테스트는 일부 단일 monster fixture에서 speed/AI를 각각 바꾸지만, 장기 acceptance validator가 그 attribution을 보존하는지 negative test는 없다.
- Expected Basis: R9 SC-CAUSE-02, SC-CAUSE-05..07, DESIGN_DECISIONS.md ADR-0031. 각 required witness는 해당 content field의 실제 consumer와 직접 연결되어야 한다.
- Actual: speed=0 또는 AI가 제거된 monster가 있어도 다른 monster의 이동 한 번으로 해당 witness가 채워질 수 있다. unused/orphan speed·AI content가 남아도 long-run required-set이 PASS할 수 있다.
- Impact: R9 완료 주장과 causal hash가 content causality의 존재를 증명하지 못하며, 이후 hardcoded fallback을 다시 도입해도 false-green이 될 수 있다. 사용자 핵심 목표인 “content causality가 spec대로 닫힘”을 판정할 수 없다.
- Suggested Action: witness를 entity/kind/content field/value/consumer 결과와 함께 구조화하고, speed와 AI를 독립 A/B fixture로 검증한다. 한 field를 freeze/remove했을 때 해당 witness만 사라지는 negative gate를 추가하고, required validator가 witness count가 아닌 attribution과 semantic delta를 검사하게 한다.
- Re-audit Method: injected registry에서 speed, AI를 각각 한 값만 변경하고 동일 seed/command sequence를 실행한다. unrelated monster를 추가·제거한 경우 witness가 오염되지 않는지, producer/consumer 제거 시 required validation이 실패하는지 확인한다.
- Owner: Architect / Coder
- Confidence: High

### [IMP-CORE-F003] injected ContentRegistry가 미래 entity 생성 경로까지 전파되지 않음

- Pass: Implementation Compliance
- Pattern: IMP-001, IMP-002, DOC-BACKFILL-001
- Area: content source of truth / entity lifecycle
- Severity: Major
- Status: Needs Fix / Confirmed
- Summary: registry-injected session은 초기 entity만 주입 registry로 생성하고 registry를 session/world에 보존하지 않는다. 이후 jackal death가 생성하는 corpse는 embedded global registry의 기본값을 다시 읽는다.
- Evidence:
  - crates/aihack-runtime/src/session.rs:55-80의 try_new_with_registry/try_new_for_playing_with_registry는 world bootstrap 결과만 보존하고 registry handle을 보존하지 않는다.
  - crates/aihack-runtime/src/bootstrap.rs:41-59,71-98은 초기 monster/item을 전달받은 registry로 변환한다.
  - 반대로 crates/aihack-runtime/src/systems/death.rs:55-60은 corpse를 만들 때 item_data(ItemKind::CorpseJackal)를 호출한다.
  - crates/aihack-runtime/src/domain/item.rs:11-23의 item_data/try_item_data는 aihack_content::registry() global embedded registry를 사용하며, death system에는 try_item_data_from_registry 경로가 전달되지 않는다.
  - tests/causal_content.rs:328-386은 기본 registry corpse nutrition 49만 확인하고 injected registry에서 corpse definition을 바꾸는 회귀가 없다.
  - ADR-0024(결정/영향)와 IMPLEMENTATION_SUMMARY.md:378-406은 runtime factory와 initial placement를 registry source로 주장한다.
- Expected Basis: spec.md DEC-CONTENT-01/SC-DATA-01, ADR-0024, R9 content producer/consumer 계약. public *_with_registry가 지원 경계라면 session 전체 생애의 생성도 같은 registry를 사용해야 한다.
- Actual: injected registry로 initial item/monster를 바꿔도 jackal 사망 후 생성되는 corpse의 nutrition, price, class metadata는 embedded default가 된다. 기본 production TOML이 동일한 동안에는 보이지 않지만 public injection·import·A/B causal test의 중간 lifecycle은 결정론적으로 같은 content source를 사용하지 않는다.
- Impact: content 값 변경이 entity creation 시점에 따라 무시되고, corpse eat/score/replay continuation이 registry A/B와 일치하지 않는다. R9의 entity lifecycle causality가 초기 bootstrap에서만 닫힌다.
- Suggested Action: immutable Arc<ContentRegistry> 또는 registry-backed factory context를 GameWorld/session에 보존하고 death/spawn 모든 경로에 전달한다. injected registry가 test-only라면 public API를 제한하고 spec.md에 범위를 명시한다. custom corpse nutrition/price와 save/continuation 회귀를 추가한다.
- Re-audit Method: corpse item 정의 하나만 injected registry에서 변경한 뒤 death→pickup→eat 및 score를 비교하고, 저장 후 재개에서도 같은 data가 유지되는지 확인한다.
- Owner: Architect / Coder
- Confidence: High

### [IMP-CORE-F004] 명세의 Awaiting*/MorePrompt 상태가 production에서 생성되지 않고 inventory 상태는 재개 불능임

- Pass: Implementation Compliance
- Pattern: IMP-002, IMP-004
- Area: state machine / persisted run state
- Severity: Minor
- Status: Needs Fix / Confirmed
- Summary: active spec은 Playing에서 direction/item selection/more prompt 상태로 전이한다고 정의하지만, production submit 경로에는 해당 상태를 생성하는 command가 없다. 또한 AwaitingInventorySelection에 저장된 session은 item command를 모두 거부한다.
- Evidence:
  - spec.md:182-188은 Playing -> AwaitingDirection, Playing -> AwaitingInventorySelection, Playing -> MorePrompt 전이를 active state machine으로 선언한다.
  - crates/aihack-runtime/src/session.rs:190-220의 Playing dispatch는 Open/Close/Kick와 Wield/Wear/Quaff/Eat/Read를 직접 실행하고 Awaiting 상태를 만들지 않는다.
  - crates/aihack-runtime/src/session.rs:223-256에서 direction은 test/save로 주입된 경우에만 처리되고, inventory handler는 Quit 외 모든 intent를 “choose an item or Esc to cancel”로 거부하며 InventoryAction에 따른 submit_*를 호출하지 않는다.
  - 전체 runtime/app 소스에서 RunState::Awaiting*/MorePrompt의 대입은 constructor 또는 awaiting branch의 복귀뿐이며 실제 Playing producer는 없다. tests/ui_screens.rs:97-110은 fixture가 상태를 직접 구성한 경우만 검증한다.
- Expected Basis: spec.md state pipeline와 CHANGELOG.md Phase 16/17의 상태 계약. 현재 direct-command UI를 유지하려면 해당 상태를 deferred/non-goal로 문서화해야 한다.
- Actual: production에서는 상태가 dead branch이고, 상태가 save로 들어오면 inventory item을 선택·실행할 수 없어 Quit만 가능하다. MorePrompt도 실제 overflow/event producer가 없다.
- Impact: state machine 문서와 runtime behavior가 drift하고, 과거/외부에서 생성된 awaiting save를 정상 재개할 수 없다. 현재 direct-command path가 동작한다는 사실은 상태 계약의 폐쇄를 증명하지 않는다.
- Suggested Action: 상태 진입을 만드는 request command와 item/direction selection/cancel 실행을 구현하거나, 현재 phase에서 해당 상태를 명시적으로 defer하고 save migration/복구 규칙을 정의한다. 각 상태의 production entry와 save-resume 회귀를 추가한다.
- Re-audit Method: 실제 TUI/headless command sequence로 각 상태 entry→selection→cancel→resume을 실행하고, 각 상태 save/load 뒤 legal action과 submit 결과를 비교한다.
- Owner: Architect / Coder
- Confidence: High

## 6. Pass 2: Debug / Engineering Quality Findings

### [DBG-CORE-F001] schema-valid malformed save가 구조 검증 없이 수용되어 첫 runtime access에서 panic함

- Pass: Debug / Engineering Quality
- Pattern: DBG-001, TEST-001
- Area: save restore / startup chain / entity lifecycle integrity
- Severity: Major
- Status: Needs Fix / Confirmed
- Summary: from_save_data는 schema version만 검사하고 world/entity 관계를 검증하지 않는다. 그 결과 schema-valid save가 GameSession으로 반환된 뒤 production headless 첫 turn에서 expect panic을 일으키며, 일부 구조 오류(중복 entity ID)는 invariant도 통과한다.
- Evidence:
  - crates/aihack-runtime/src/save.rs:193-209은 schema_version 비교 후 즉시 GameRng::from_state, GameWorld::from_saved_world, event log를 조합한다. world invariant/entity validation 호출이 없다.
  - crates/aihack-core/src/invariant.rs:7-32,57-95의 여섯 검사는 current level/player identity·위치·inventory owner만 확인하며 entity ID 유일성, item↔inventory location, equipment 대상, map tile length, actor stat 범위 등을 확인하지 않는다.
  - crates/aihack-runtime/src/world.rs:121-145의 map/player lookup은 expect/assert를 사용한다. snapshot.rs:136-137와 observation.rs:95-100도 player 위치/stats를 panic 전제로 사용한다.
  - 실행 probe: 정상 CLI save를 만든 뒤 world.player_id=999로 바꾸고 target/debug/aihack-headless.exe --load audit-malformed/valid.json --turns 1 --policy wait-v1를 실행했다. exit 101, panic 위치 crates/aihack-runtime/src/world.rs:136:14, 메시지 Phase 5 world는 항상 player actor 위치를 가진다.
  - 별도 probe에서 두 번째 entity ID를 1로 바꾼 malformed save는 같은 load/turn 명령에서 exit 0으로 accepted report를 생성했다. EntityStore::get의 first-match semantics 때문에 duplicate ID는 actor/item lookup을 서로 가릴 수 있다.
  - 기존 tests/save_load.rs는 정상 roundtrip과 schema mismatch만, tests/world_invariants.rs는 여섯 persisted violation만 검사한다.
- Expected Basis: spec.md:203-208,607-623, designs.md:260-272, ADR-0023/0032. persisted input은 typed error로 fail-closed하며 invalid state가 production expect까지 도달하지 않아야 한다.
- Actual: semantic validation 없이 SaveDataV1을 live session으로 복원한다. malformed player는 load 자체가 아닌 첫 observation/turn에서 panic하고, duplicate ID·dangling item·stale equipment는 수용될 수 있다.
- Impact: 사용자가 지정한 save 하나로 TUI/headless가 비정상 종료할 수 있고, duplicate/dangling entity가 전투·inventory·death lifecycle을 잘못된 entity에 적용할 수 있다. save/load 데이터 무결성 gate가 schema-only gate로 축소되어 있다.
- Suggested Action: GameError::InvalidSave와 별도 validator를 추가해 schema 이후 construction 전에 map dimensions/tile count, level ID uniqueness, nonzero/unique entity ID와 next_id, player actor/location, item location/inventory/equipment 관계, actor stat/enum/range, event-log/revision 관계를 검사한다. production lookup의 expect/assert는 typed error 경계 안으로 옮긴다. 의도적으로 invalid fixture가 필요하면 명시적 test-only unchecked constructor를 분리한다.
- Re-audit Method: player missing/not-player, duplicate ID, missing map/tile, dangling inventory, wrong owner/equipment, out-of-bounds actor, malformed RNG를 각각 JSON fixture로 만들고 ArtifactStore::load_session 및 headless/TUI load가 typed error와 nonzero exit를 내는지 확인한다. valid save의 snapshot/hash/continuation은 유지한다.
- Owner: Architect / Coder
- Confidence: High

### [DBG-CORE-F002] persisted RNG 복원이 draws에 비례하는 비제한 재실행 루프임

- Pass: Debug / Engineering Quality
- Pattern: DBG-002
- Area: save boundary / deterministic RNG / resource bound
- Severity: Major
- Status: Needs Fix / Confirmed
- Summary: RngStateV1.draws가 임의의 u64인데 GameRng::from_state가 seed부터 그 횟수만큼 난수를 다시 뽑아 복원한다. save 입력에 상한·turn 관계 검증이 없어 큰 값은 사실상 무한 작업으로 보인다.
- Evidence:
  - crates/aihack-core/src/rng.rs:12-16은 draws: u64를 외부 직렬화한다.
  - crates/aihack-core/src/rng.rs:27-33은 for _ in 0..state.draws { rng.next_u64(); }를 사용하며 상한이나 checked bound가 없다.
  - crates/aihack-runtime/src/save.rs:200-206의 from_save_data가 모든 load에서 이 경로를 직접 호출한다.
  - crates/aihack-core/src/rng.rs:39-41의 draws += 1도 극단값에서 checked overflow 계약이 없다.
  - tests/save_load.rs:29-37은 draw 두 번인 정상 사례만 확인하며 extreme draw rejection/시간 상한 테스트가 없다.
- Expected Basis: spec.md:31,57,617, audit_roadmap.md R4 결정론/실행 gate. deterministic resume는 입력 크기에 따른 무제한 재계산 없이 bounded하게 실패하거나 복원되어야 한다.
- Actual: draws=u64::MAX인 schema-valid save는 load 과정에서 종료가 보장되지 않는다. 정상 장기 run의 낮은 draw 수만 검증되며 악성·손상 save와 매우 긴 실행의 비용 상한이 정의되어 있지 않다.
- Impact: headless/TUI load가 CPU를 독점해 hang/DoS가 되고 failure report나 typed error를 생성하지 못한다. draws가 turn보다 큰지, replay/command 수와 일관적인지도 확인되지 않는다.
- Suggested Action: RNG 내부 상태를 직접 serialize하거나 bounded jump/skip-ahead를 사용한다. 직접 replay를 유지한다면 draws의 명시적 최대값과 turn/accepted command 기반 상한을 정의하고 초과 시 InvalidSave를 반환한다. next_u64는 checked increment 또는 overflow 정책을 가져야 한다.
- Re-audit Method: 0, 정상 baseline, 허용 상한, 상한+1, u64::MAX fixture를 load해 성공/typed error/실행 시간 상한을 확인하고 save→load continuation hash를 비교한다.
- Owner: Architect / Coder
- Confidence: High

### [DBG-CORE-F003] save/replay reader와 event log에 크기·cardinality 상한이 없음

- Pass: Debug / Engineering Quality
- Pattern: DBG-001, DBG-002
- Area: artifact parsing / memory growth
- Severity: Minor
- Status: Needs Fix / Confirmed
- Summary: 파일 경계의 path safety와 별개로, runtime은 save 전체를 String으로 읽고 replay 전체를 Vec으로 수집하며 session event log를 매 accepted turn마다 계속 확장한다.
- Evidence:
  - crates/aihack-runtime/src/save.rs:64-71의 read_replay_lines는 모든 line을 collect()한다.
  - crates/aihack-runtime/src/save.rs:81-87의 load_session은 read_to_string으로 save 전체를 메모리에 올린다.
  - crates/aihack-runtime/src/session.rs:529,541의 accepted/UI event 경로는 event_log.extend(...)를 상한 없이 수행한다. save는 save.rs:180-190에서 전체 event log를 직렬화한다.
  - apps/aihack-headless/src/main.rs:102-110은 replay 전체를 읽은 뒤 runner에 전달한다.
- Expected Basis: 사용자 요청의 “명백한 비제한 동작” 질문과 spec.md:651,687-701의 artifact/replay 운영 계약. 파일 크기와 long-run 메모리 정책이 명시되어야 한다.
- Actual: 파일 크기·line 수·event count가 운영 상한과 연결되어 있지 않다. 현재 1000-turn fixture는 작아서 green이지만, 큰 local artifact나 장기 실행은 parse/serialize 비용과 memory가 입력 크기만큼 증가한다.
- Impact: malformed/대형 local artifact가 OOM 또는 긴 parse 시간을 유발하고, 오래 실행할수록 save/replay latency가 누적된다. file_security의 path 방어가 성공해도 resource exhaustion은 별도다.
- Suggested Action: ArtifactStore read에 byte limit/line limit를 두고 초과 시 typed error를 반환한다. replay는 target turn까지 stream하거나 bounded trace로 처리하고, event log는 최대 보존량·checkpoint·별도 history 정책을 정의한다.
- Re-audit Method: 경계-1/경계/경계+1 bytes와 lines/event count fixture를 실행해 bounded failure을 확인하고, 정상 long-run hash/last-event 계약을 유지한다.
- Owner: Architect / Coder
- Confidence: High

### [DBG-CORE-F004] armor drop/re-pickup lifecycle가 player AC를 복원하지 않음

- Pass: Debug / Engineering Quality
- Pattern: TEST-001
- Area: entity lifecycle / inventory-equipment state
- Severity: Major
- Status: Needs Fix / Confirmed
- Summary: armor 착용은 player AC를 감소시키지만, drop은 equipment pointer만 지우고 AC를 되돌리지 않는다. 같은 armor를 다시 착용하면 bonus가 중복 적용된다.
- Evidence:
  - crates/aihack-runtime/src/systems/items.rs:66-90의 wear는 stats.ac -= ac_bonus 후 equipped_body를 설정한다.
  - crates/aihack-runtime/src/systems/items.rs:93-107의 drop은 inventory.remove(item)와 item location 변경만 수행한다.
  - crates/aihack-core/src/domain/inventory.rs:39-47의 remove는 equipped_body=None만 처리하고 actor stat은 알지 못한다.
  - spec.md:775-777의 semantic state에는 AC와 entity lifecycle이 포함되고, tests/causal_content.rs:288-325는 wear 한 번만 검사한다. wear→drop→pickup→wear 회귀는 없다.
- Expected Basis: item/equipment lifecycle의 state consistency와 R9 ArmorDefense semantic delta. 장비 pointer와 실제 AC는 한 상태 전이로 함께 유지되어야 한다.
- Actual: Wear 후 Drop은 equipped_body=None/표 item인데 player AC는 이미 감소한 값으로 남긴다. 재Pickup 후 Wear는 같은 bonus를 다시 빼며 save에도 잘못된 AC가 저장된다.
- Impact: 전투 hit/defense 및 death score/hash가 item ownership과 불일치한다. invariant 6종은 이를 탐지하지 않아 장기 fixture가 green인 채 lifecycle drift를 보존한다.
- Suggested Action: equip/unequip를 단일 helper로 만들고 old body armor의 bonus를 복원한 뒤 pointer/location을 바꾼다. 장비 교체·drop·consume 공통 경로와 “AC = base AC + 현재 장비 효과” invariant를 추가한다.
- Re-audit Method: armor pickup→wear→drop→pickup→wear, save/load 각 단계의 equipped_body, location, AC, combat roll/hash를 검사한다.
- Owner: Coder
- Confidence: High

## 7. Pass 3: Security / Data-Integrity Findings

### [SEC-CORE-F001] replay의 recorded outcome/hash/turn을 검증하지 않아 위조 artifact가 성공으로 수용됨

- Pass: Security
- Pattern: SEC-004, SEC-005
- Area: replay integrity / untrusted persisted input
- Severity: Major
- Status: Needs Fix / Confirmed
- Summary: replay JSONL은 turn_before, outcome, snapshot_hash_after를 기록하지만 runner는 line.command만 submit하고 나머지 세 필드를 전혀 비교하지 않는다. 따라서 file path가 안전해도 artifact 의미 무결성은 보장되지 않는다.
- Evidence:
  - spec.md:687-701은 replay line의 네 필드와 save/replay truth를 명시하고 snapshot_hash_after를 wire에 포함한다.
  - apps/aihack-headless/src/lib.rs:58-95의 run_replay_to_turn은 line.command만 session.submit에 전달한다. turn_before, line.outcome, line.snapshot_hash_after를 검증하지 않고 실제 revision으로 성공 report를 만든다.
  - apps/aihack-headless/src/main.rs:102-110은 읽은 lines를 곧바로 replay runner에 전달한다.
  - 실행 probe: 정상 1-turn trace를 만든 뒤 turn_before=999, outer hash=0000000000000000, inner outcome hash=1111111111111111, outcome.accepted=false로 변조했다. target/debug/aihack-headless.exe --seed 42 --turns 1 --policy replay-file --replay-in audit-replay/trace.jsonl가 exit 0, accepted_turns=1, 실제 final hash 54e43384cefa2590인 성공 report를 출력했다.
  - tests/headless_policy.rs:31-55는 정상 trace의 command 재생만 확인하며 corrupted turn/hash/outcome negative gate가 없다.
- Expected Basis: spec.md save/replay contract, ReplayLineV1의 필드 의미, audit_roadmap.md R4의 save/load/replay continuation hash. 기록된 replay가 truth라면 line별 전후 검증과 mismatch fail-closed가 필요하다.
- Actual: wrong-session, truncated, tampered, out-of-order line도 command가 target turn을 만들면 성공할 수 있다. recorded evidence는 장식 필드가 되고, report의 final hash만 현재 실행 결과를 표시한다.
- Impact: 재현성·감사 artifact가 변조되어도 감지되지 않으며, 사용자/CI가 “기록된 outcome과 hash를 재현했다”고 오판할 수 있다. file_security의 no-follow/atomic boundary를 통과해도 data-integrity gate는 우회된다.
- Suggested Action: line별 current turn_before 일치, submit outcome의 accepted/turn/events/next_state/hash와 outer hash 일치, target/session seed 정책을 검증하고 mismatch를 typed ReplayMismatch로 실패시킨다. mismatch가 발생하면 working-copy/temporary session에서만 실행해 partial mutation을 남기지 않는다. corrupted turn/hash/outcome/sequence/target fixtures를 추가한다.
- Re-audit Method: 정상 trace는 계속 PASS하고 각 필드를 하나씩 변조한 trace는 nonzero exit와 failure report를 내며 session state가 commit되지 않는지 확인한다. load-resume replay와 direct run final hash도 다시 비교한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: 이 finding은 path traversal·symlink·ACL의 대체 판정이 아니라, 안전하게 열린 파일의 내용이 실제 runtime truth와 일치하는지에 대한 독립 판정이다.

## 8. Cross-Pass Conflicts

### [XPF-CORE-F001] 문서/grep 기반 private-state PASS와 실제 public API surface의 충돌

- Related Findings: IMP-CORE-F001, DBG-CORE-F001
- Conflict: R2/R5 문서와 workspace_boundaries/grep은 direct assignment 0건과 accessor 사용을 PASS로 기록하지만, DerefMut 때문에 외부 consumer의 assignment가 여전히 가능하고 malformed state도 loader로 유입된다.
- Resolution: 호출자 현재 사용이 안전하다는 사실만으로 API boundary PASS를 인정하지 않는다. type-level surface 제거와 malformed-state validation이 모두 필요하다.
- Gate Impact: Major; core/workspace state gate와 production data-integrity gate를 차단한다.
- Required Fix Before PASS: F001 수정 후 external compile-fail/API test와 malformed save test를 함께 통과시킨다.

### [XPF-CORE-F002] 장기 결정론 PASS와 causal attribution false-green의 충돌

- Related Findings: IMP-CORE-F002, DBG-CORE-F004
- Conflict: seed 42/7/1234의 1000-turn final hash와 witness multiset은 3회 반복 일치하지만, witness가 content field별 원인 증거가 아니라 임의 monster 이동과 armor wear/drop drift를 포함할 수 있다.
- Resolution: 반복성은 구현이 같은 잘못을 반복한다는 증거일 뿐 causal closure 증거가 아니다. field attribution·lifecycle invariant를 별도 gate로 둔다.
- Gate Impact: Major; R9 causal PASS를 배포 적합성으로 해석할 수 없다.
- Required Fix Before PASS: F002와 F004의 targeted negative/regression tests를 추가하고 required witness validator를 강화한다.

### [XPF-CORE-F003] 파일 경계 PASS와 replay 의미 무결성의 충돌

- Related Findings: SEC-CORE-F001, DBG-CORE-F003
- Conflict: ArtifactStore의 no-follow/hard-link/atomic write 표적 테스트는 green이지만, 안전하게 읽은 replay 내용의 turn/outcome/hash가 검증되지 않는다.
- Resolution: file_security는 경로·파일 객체의 안전성을, core runtime은 parsed artifact가 게임 결과와 일치하는지를 독립 gate로 유지한다.
- Gate Impact: Major; 두 증거가 모두 있어야 save/replay data-integrity 목표를 Covered로 볼 수 있다.
- Required Fix Before PASS: F001 replay semantic validation과 file_security의 path/permission 재감사를 모두 통과한다.

## 9. Required Fixes Before PASS

1. IMP-CORE-F001: GameSession/GameWorld/EntityStore의 public Deref mutation surface를 제거하고 external compile-fail/API boundary를 추가한다.
2. DBG-CORE-F001: schema-valid save의 semantic world/entity validator와 typed InvalidSave 경계를 추가해 malformed load panic과 duplicate/dangling state를 차단한다.
3. DBG-CORE-F002: RNG restore의 재실행 비용을 bounded하게 만들고 extreme draws/overflow를 typed error로 처리한다.
4. SEC-CORE-F001: replay line의 turn, outcome, inner/outer hash를 검증하고 mismatch 시 no-commit으로 실패시킨다.
5. IMP-CORE-F002: causal witness를 content field와 consumer에 귀속시키는 A/B·negative gate로 강화한다.
6. IMP-CORE-F003: injected registry가 corpse 등 runtime-created entity까지 유지되도록 하거나 API를 test-only로 제한하고 문서화한다.
7. DBG-CORE-F004: armor unequip/drop/re-pickup에서 AC와 equipment/location을 원자적으로 동기화한다.
8. DBG-CORE-F003 및 IMP-CORE-F004: artifact/event resource budget과 awaiting/more state의 구현 또는 명시적 defer/migration을 확정한다.

Major finding이 하나라도 남아 있는 동안 core/runtime 범위의 PASS 또는 PASS WITH KNOWN RISKS는 허용하지 않는다.

## 10. Accepted Risks

- cargo metadata와 source manifest를 기준으로 aihack-content -> aihack-core, aihack-runtime -> core/content/AI contract, aihack-tui/headless -> runtime/contract의 의존 방향은 확인되었다. aihack-core의 TUI/HTTP dependency도 없다.
- production binary(apps/aihack-tui, apps/aihack-headless)는 현재 GameSession::try_new* fallible bootstrap을 사용하며, infallible new/fixture_*는 root integration test와 compatibility fixture에서만 호출된다. 다만 public Deref와 registry lifecycle finding은 이 사실로 면제되지 않는다.
- valid fixture의 transaction no-commit, six invariant report, content parse, 1000-turn deterministic hash, normal save/load continuation, ArtifactStore path/link checks는 이번 local commands에서 green이었다. 이는 malformed input, replay tampering, entity lifecycle까지의 PASS를 의미하지 않는다.
- 별도 file-security 보고서가 path/ACL/atomic replace를 판정할 때 이 보고서의 replay semantic integrity와 loader validation finding을 함께 참조해야 한다. 현재 이 위험을 Accepted Risk로 수용할 owner/만료 조건은 없다.

## 11. Needs Spec Clarification

1. try_new_with_registry가 공개된 지원 runtime 경계인지, test/import 전용인지 결정해야 한다. 지원 경계라면 F003을 수정하고, test-only라면 API visibility와 문서 범위를 줄인다.
2. replay line의 outcome·inner/outer hash·turn_before가 필수 integrity evidence인지, 아니면 command-only compatibility trace인지 명시해야 한다. 현재 active spec의 필드 정의와 R4 hash wording은 검증을 기대하지만 mismatch error/rollback 계약은 부족하다.
3. AwaitingDirection, AwaitingInventorySelection, MorePrompt를 현재 phase에서 실제 producer까지 구현할지, direct-command UI와 함께 deferred/non-goal로 닫을지 결정해야 한다.
4. SaveDataV1의 semantic validation 범위(duplicate ID, inventory/entity relation, map shape, stat range)와 intentional test fixture용 unchecked constructor를 문서화해야 한다.
5. RngStateV1.draws, save/replay bytes/line count, event log 보존량의 최대값과 long-run resource budget을 정해야 한다.

## 12. Re-audit Checklist

- [ ] Deref/DerefMut 제거 또는 외부 mutation 불가를 별도 consumer compile-fail/API test로 확인
- [ ] malformed save: missing/not-player player, duplicate ID, dangling inventory, wrong equipment, malformed map, invalid stat, huge RNG를 typed error로 거부
- [ ] load/submit/observation/snapshot의 production expect/assert가 unvalidated persisted state에 도달하지 않음을 확인
- [ ] RNG restore의 bounded direct-state/jump 또는 explicit cap과 overflow 회귀 확인
- [ ] replay line의 turn, accepted/turn_advanced/events/next_state, inner hash, outer hash를 검증하고 mismatch 시 no-commit 확인
- [ ] custom registry에서 corpse 생성·pickup·eat·score와 save/load continuation이 registry source를 유지하는지 확인
- [ ] causal speed/AI witness를 field-specific A/B와 negative fixture로 검증하고 unrelated monster 이동이 witness를 오염시키지 않음을 확인
- [ ] armor wear→drop→pickup→wear 및 save/load 뒤 AC/equipment/location/combat hash를 확인
- [ ] awaiting/more state의 실제 production entry 또는 명시적 defer/migration을 검증
- [ ] save/replay byte/line/event limits를 경계값과 장기 실행으로 검증
- [ ] file_security 보고서의 path/ACL/atomic evidence와 이 report의 semantic integrity evidence를 통합 coverage 표에서 별도 행으로 유지

## 13. Perspective Decision

**HOLD — core/runtime 범위에서 PASS 불가.**

Critical finding은 확인하지 않았지만, private state 경계 우회, malformed save panic/부분 invariant, 비제한 RNG restore, replay tamper 수용, causal false-green, registry lifecycle drift, armor lifecycle corruption이 모두 Major 또는 그에 준하는 미해결 finding이다. 따라서 valid fixture의 workspace/test green을 근거로 GameSession transaction·save/replay·content causality가 production-ready라고 판정할 수 없다. 위 체크리스트를 수행한 뒤 관련 pass를 재감사해야 한다.

## 14. Coder Handoff

    C:\LocalDev\rust\AIHack\docs\multi_audit\1\sub_audit_02_core_runtime.md를 먼저 읽고, 각 finding을 현재 spec/ADR와 실제 코드에 대조하여 검증한 뒤 우선순위대로 수정하세요. 계약을 바꾸는 경우 관련 문서를 먼저 갱신하고, 수정 후 malformed save/replay, transaction/invariant, content causality, entity lifecycle 표적 테스트와 workspace quality gate를 실행해 재감사 증거를 기록하세요.
