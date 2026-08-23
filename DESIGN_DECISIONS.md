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

## ADR-0035: report 25 production 경계와 exact-set 재시정

Status: Accepted; implementation and same-SHA verification in progress (2026-08-23)
Date: 2026-08-23
Decision ID: DEC-AUDIT-R25-01

Context:

`docs/audit/audit_report_25.md`는 final multi-audit report 1의 첫 시정본을 실제 production entrypoint와 변조 fixture로 재검증했다. broad workspace green과 helper-level test는 inverse save relation, write-side budget, replay alias identity, production GoldScore pair, TUI event-loop 우선순위/geometry, terminal failure lifecycle, actual release output 집합을 닫지 못했다. 문서도 report 23/24, final report 1과 report 25의 authority를 동시에 current처럼 표현했다.

Decision:

- save read/write는 동일 semantic/byte 예산을 사용한다. actor `alive/hp/max_hp`, inventory 양방향 relation과 armor checked arithmetic을 복원 전에 검증하고, writer도 capped serialization 성공 전에 destination을 열거나 교체하지 않는다.
- headless target 1,000,000은 실행 범위일 뿐 save 성공 보장이 아니다. budget 초과 save는 typed 실패와 no-clobber로 닫으며 v0.3.0에서 history를 자동 폐기하지 않는다.
- artifact relative path는 `.`을 제거한 canonical lexical form을 사용한다. replay input/output은 lexical form, Windows case와 열린 file identity를 함께 비교하고 ambient path helper를 public API에서 제거한다.
- GoldScore는 동일 world/turn의 active/control clone에서 gold만 0으로 바꾸고 양쪽 모두 production `death_score`를 호출한 결과로만 witness를 기록한다.
- TUI는 raw event에서 candidate를 만드는 state-aware dispatcher 하나를 production loop와 tests가 공유한다. blocking state는 LLM dismiss/focus보다 우선하고, reset 이전 response ID는 새 outstanding 존재 여부와 무관하게 먼저 폐기한다.
- blocking prompt는 최소 60x24/80x24에서 별도 modal 높이를 보장한다. command/inspect mouse hit box는 renderer와 같은 CTA label model에서 파생한다.
- release `output/` 전체를 게시 bundle로 정의하고 platform별 actual top-level entry exact set을 검사한다. extra file/directory/link/reparse는 checksum 미포함 여부와 관계없이 실패다.
- dependency exception은 parsed TOML AST, exact trigger set과 valid calendar date를 검사한다. 모든 CI `uses:`는 40-hex ref여야 하며 provenance comment는 실제 tag/SHA와 일치해야 한다.
- 현재 authority는 report 25 HOLD다. report 23/24는 historical closed, final report 1과 첫 remediation은 partial historical evidence이며 gap child/aggregate는 동일 lifecycle을 따른다.

Alternatives:

- loader, checksum inventory, mapper helper만 보강: production writer/output/event loop가 계속 다른 계약을 실행하므로 기각한다.
- 1,000,000턴 save를 항상 보장하도록 즉시 event compaction 도입: wire/evidence retention 정책을 새로 만들고 범위를 확대하므로 v0.3.0에서는 기각한다.
- release publish 대상을 checksum에 적힌 파일만으로 간주: 현재 build와 문서가 `output/` directory를 bundle로 전달하므로 기각한다.

Consequences:

기존 malformed save와 stale release directory 일부는 새 gate에서 거부된다. public ambient path helper 제거는 pre-release internal API 정리이며 production caller는 이미 `ArtifactStore`를 사용한다. 각 수정은 report 25의 수정 전 fixture를 이름 붙인 RED 기록으로 보존하고, 로컬 전체 gate 뒤 clean 동일 SHA의 Ubuntu/Windows release verifier까지 통과해야 `Verified`로 승격한다.

2026-08-24 same-SHA verification update: 첫 clean run `32648979651`의 Ubuntu가 O_PATH `Dir` clone에 대한 parent `sync_all`을 EBADF로 거부했다. capability root를 ambient path로 되돌리지 않고 parent 아래 `.`을 read-only directory `File`로 다시 열어 sync 가능한 descriptor를 얻는 방식으로 시정한다. 첫 run은 실패 증거로 보존하며 후속 same-SHA run만 closure evidence로 사용한다.

## ADR-0034: SaveDataV1 fail-closed 복원과 self-verifying replay

Status: Superseded in part by ADR-0035; report 25 remediation in progress
Date: 2026-08-23
Decision ID: DEC-SAVE-03

Context:

최종 다중 감사 `FIN-F001..F007`은 v1 save가 schema 외 semantic 관계와 자원 상한을 검사하지 않고, replay metadata를 검증하지 않으며, public `DerefMut`와 registry/equipment lifecycle가 transaction truth를 우회한다고 확인했다. 기존 valid-fixture test와 deterministic hash는 손상 artifact와 외부 mutation을 증명하지 못한다.

Decision:

- SaveDataV1 wire shape와 schema version 1은 유지하되 16 MiB, event/entity 각 100,000개, RNG 1,000,000 draw, persisted text 512 UTF-8 byte의 fail-closed 경계를 둔다.
- 복원 전에 entity, player, map, inventory, equipment, actor stat과 text 관계를 typed validator로 검사한다. invalid artifact는 `GameError::InvalidSave`이며 live session이나 RNG를 만들지 않는다.
- ReplayLineV1은 self-verifying artifact다. 소비한 line의 turn, full outcome, outer hash를 working clone에서 비교하고 전체 성공 후 한 번만 commit한다.
- GameSession/GameWorld/runtime EntityStore의 외부 `DerefMut`를 제거하고 crate-private typed mutation surface만 사용한다.
- GameWorld는 immutable registry context를 runtime-only로 보존한다. injected save 복원은 동일 registry를 인자로 받으며, runtime-created corpse/equipment도 그 context와 공통 lifecycle helper를 사용한다.
- replay append는 bounded read 후 atomic rewrite로 바꿔 외부 hard-link inode에 열린 append handle로 쓰지 않는다.
- atomic replace 뒤 Unix parent directory를 fsync한다. Windows directory handle flush의 portable 보장은 현재 dependency에서 제공하지 않으므로 file sync + atomic replace 이후 전원 손실 metadata durability는 명시적 platform 잔여 위험이다.

Alternatives:

- SaveDataV2로 즉시 변경: v0.3.0 wire 호환을 불필요하게 깨므로 기각한다.
- replay를 command-only로 축소: 기존 integrity 필드와 감사/재현 용도를 무의미하게 만들어 기각한다.
- invalid state를 deserialize 뒤 runtime `expect`에 맡김: panic과 부분 상태를 허용하므로 기각한다.
- 같은 계정의 악성 concurrent writer까지 애플리케이션 lock으로 격리: OS sandbox 없이 완전한 경계를 보장할 수 없고 현재 single-writer 제품 모델을 넘으므로 비목표로 둔다. 사전 배치 link와 외부 inode write는 계속 차단한다.

Consequences:

valid v1 save/replay wire는 유지되지만 과거에 우연히 수용된 malformed artifact는 typed error가 된다. 테스트는 boundary-1/boundary/boundary+1, malformed 관계 matrix, replay field별 tamper/no-partial-commit, 외부 compile-fail, custom registry continuation, armor wear/drop/rewear를 직접 검증한다.

## ADR-0033: winx 0.36.4 LLVM exception 한정 허용

Status: Implemented with machine expiry/graph gate (2026-08-23)
Date: 2026-08-18
Decision ID: DEC-DEP-02

Context:

ADR-0032의 capability filesystem은 Windows에서 `cap-primitives -> winx 0.36.4`를 shipped dependency로 추가한다. `winx`의 유일한 SPDX 식은 `Apache-2.0 WITH LLVM-exception`이며 기존 `deny.toml`의 일반 `Apache-2.0` 허용만으로는 cargo-deny license gate를 통과하지 못한다.

Decision:

- 일반 allowlist를 넓히지 않고 `winx 0.36.4`에만 `Apache-2.0 WITH LLVM-exception`을 허용한다.
- exception owner는 Dependency owner / Release manager다.
- exception은 2026-10-31에 만료하며, 그 전에 `winx`, `cap-primitives`, `cap-std`, `cap-fs-ext`, `cap-tempfile` 중 하나의 version이 바뀌면 즉시 재검토한다.
- `dependency-exceptions.json`의 `DEP-EXC-0001`을 예외 owner, 사유, 승인일, 만료일과 trigger version의 단일 ledger로 사용한다. CI test는 오늘 날짜가 만료일을 지났거나 deny 설정·Cargo graph가 ledger와 다르면 실패한다.
- cargo-deny 0.19.4의 `licenses`, `bans`, `sources` 실제 PASS를 R1/R8 필수 증거로 유지한다.

Alternatives:

- `Apache-2.0 WITH LLVM-exception`을 일반 allowlist에 추가: 향후 무관한 crate까지 자동 허용하므로 기각한다.
- capability dependency 제거: SEC-F001의 portable path sandbox와 atomic replace 경계를 후퇴시키므로 기각한다.
- cargo-deny gate 생략: shipped dependency policy를 검증하지 못하므로 기각한다.

Consequences:

`deny.toml` exception은 crate와 version을 함께 고정한다. `dependency-exceptions.json` checker가 만료, unrelated crate 확장, dependency version 변경을 fail-closed한다. 실패 시 exception 유지, dependency 교체, 일반 정책 변경 중 하나를 다시 결정하고 `BUILD_GUIDE.md`와 감사 기록을 갱신한다.

전역 `multiple-versions = "allow"`는 무제한 허용이 아니다. `dependency-duplicate-budget.json`의 정확한 all-target family/version 집합과 최대 23개 family를 별도 machine gate로 고정하며 drift는 review 전까지 실패한다.

Verification update: implementation SHA `2519bc8e0ede81c39f46b5778e62a41d4ca66901`의 Actions run `32107862171`에서 Ubuntu/Windows cargo-deny 0.19.4와 전체 quality gate가 success다. report 23/24 시정은 후속 독립 재감사와 외부 게시 승인을 대체하지 않는다.

## ADR-0032: capability 기반 save/replay 파일 경계

Status: Superseded in part by ADR-0034 (2026-08-23)
Date: 2026-08-18
Decision ID: DEC-SAVE-02

Context:

기존 headless 경로는 문자열 경로를 canonicalize한 뒤 실제 파일을 다시 열었고, save는 예측 가능한 `.tmp`를 truncate mode로 생성했다. 이 구조는 경로 검사와 open 사이의 교체 경쟁을 남기며, 미리 배치한 symbolic link 또는 hard link가 root 밖 파일을 열거나 truncate하게 할 수 있다. TUI quick-save도 모든 프로세스가 같은 OS temp 경로를 공유했다.

Decision:

- headless의 save/load/replay/report I/O는 열린 runtime root directory capability와 검증된 상대 경로를 함께 보유하는 `ArtifactStore`를 통한다. absolute path와 root 탈출은 구조적으로 거부하고, 각 open/rename은 root handle 기준으로 수행한다.
- 쓰기 대상의 마지막 경로 요소는 symbolic link를 따라가지 않는다. 열린 handle이 일반 파일이며 hard-link count가 1인지 쓰기 전에 검증한다.
- save는 대상과 같은 directory에 충돌하지 않는 임시 파일을 `create_new`로 생성하고 payload와 file metadata를 동기화한 뒤 capability-relative rename으로 교체한다. Unix는 mode `0600`을 강제하고 Windows는 parent directory DACL을 상속하므로, Windows에서 기밀성이 필요한 runtime root는 사용자 전용 directory여야 한다. 실패 시 새 임시 파일만 정리하고 기존 save는 보존한다.
- replay 기록은 기존 payload를 bounded read한 뒤 같은 directory의 temporary file로 atomic rewrite한다. 열린 외부 inode에 append하지 않는다.
- TUI quick-save는 프로세스별 임시 directory를 사용해 다른 실행과 경로를 공유하지 않는다.
- `ArtifactStore::open`은 root의 마지막 component를 no-follow로 열고 symbolic link와 Windows junction/reparse root를 거절한다. TUI는 store와 relative quick-save path를 소유한다.

Alternatives:

- canonicalize 후 일반 `File::open`/`File::create` 유지: 검사와 사용 사이 경쟁을 닫지 못해 기각한다.
- 고정 `.tmp` 이름에 `create_new`만 추가: 사전 배치 공격의 truncate는 막지만 동시 실행 충돌과 parent 교체 경계를 해결하지 못해 기각한다.
- 플랫폼별 raw descriptor API를 프로젝트에서 직접 구현: `unsafe`와 OS별 유지보수 표면을 늘리므로 기각하고, 검증된 capability filesystem dependency를 사용한다.
- Windows DACL을 runtime에서 직접 재작성: `unsafe` Windows ACL API와 권한 상속 파괴 위험이 현재 save 데이터 민감도보다 크므로 기각한다. owner-only가 필수인 배포는 사용자 전용 application directory를 root로 제공한다.

Consequences:

`aihack-runtime`은 capability filesystem dependency를 추가한다. path 기반 compatibility helper는 신뢰된 테스트 경로에서만 유지하고 production CLI는 `ArtifactStore`를 사용한다. `tests/headless_paths.rs`는 Windows와 Unix에서 temp hard-link/symlink, replay link, parent escape, 기존 save 보존과 platform permission contract를 직접 회귀한다. Windows inherited DACL은 owner-only hard boundary로 표현하지 않는다.

## ADR-0031: 콘텐츠 인과 폐쇄와 상태-delta 검증

Status: Accepted (2026-08-17)
Date: 2026-08-17
Decision ID: DEC-CAUSE-01

Context:

기존 content registry와 snapshot은 많은 값을 저장하지만 일부 값은 kind 기반 상수에 가려지거나 production producer/consumer가 없다. 기존 1000턴 테스트는 생존과 최종 hash 반복성만 확인하므로 orphan content가 남아 있어도 통과한다.

Decision:

- 주요 콘텐츠는 `producer -> semantic state delta -> consumer -> downstream delta` 경로를 가져야 한다.
- PASS 판정에서 turn, event count, last event만의 변화는 제외한다.
- item nutrition은 Eat 행동과 hunger 전이에 연결한다.
- armor, monster behavior처럼 schema가 지원한다고 선언한 값은 typed runtime data에 투영한다. 현재 milestone에서 의미 있게 지원하지 않을 값은 거짓 계약으로 남기지 않고 명시적으로 제거 또는 후속 비목표로 분류한다.
- 경제 값은 가격이 후속 score 또는 경제 상태를 바꾸는 최소 루프로 연결한다.
- luck과 hallucination은 production producer와 downstream consumer를 함께 제공할 때만 active state로 인정한다.
- seed 42, 7, 1234의 장기 regression은 필수 causal witness와 semantic delta를 집계하고 반복 hash와 함께 검증한다.

Alternatives:

- 이벤트 존재만 검증: 구현 호출은 증명하지만 실제 세계 변화는 증명하지 못해 기각한다.
- 모든 orphan 필드 삭제: save v1 호환성과 이미 노출된 관찰 계약을 불필요하게 파괴하므로 기각한다.
- 한 번에 범용 event bus 도입: 현재 규모보다 복잡하고 인과 증거를 자동으로 보장하지 않아 기각한다.

Consequences:

R9는 테스트 기반을 먼저 만들고 음식/영양, content behavior, 경제/점수, 상태 orphan 순서로 작은 수직 슬라이스를 적용한다. 의도된 snapshot hash 변화는 witness와 ADR 근거 없이 baseline만 갱신할 수 없다.

2026-08-18 verification update: `CausalProjection`과 9종 필수 `CausalWitness`가 seed 42/7/1234의 1000턴 fixture에서 각 1회 이상 발생하고 witness multiset/final hash가 3회 반복 일치한다. `hallucinating` SaveDataV1 compatibility risk owner는 Project owner/runtime maintainer이며 SaveDataV2·v0.4.0 scope 승인 또는 2026-10-31 중 먼저 도래하는 시점에 제거 migration과 실제 producer feature 중 하나를 재결정한다.

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
- R7 source 대조에서 기존 hunger 상태 경계가 3.6.7과 다름을 발견해 C008 RED test 후 `newuhs`의 다섯 상태 경계로 수정했다. 기존 `Oversatiated` enum variant는 호환 목적으로 남지만 새 projection에서는 생성하지 않는다.

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

Status: Accepted; R1/report 21 verified, RUSTSEC-2026-0253 remediation updated 2026-08-18
Date: 2026-07-15
Decision ID: DEC-UI-DEP-01

Context:

ADR-0022의 ratatui 0.29 계열은 crossterm 중복을 피했지만, 현재 RustSec advisory에서 필수 dependency `lru 0.12.5`의 memory-corruption unsound 문제와 `paste 1.0.15`의 unmaintained 상태가 확인됐다. `lru`는 ratatui 0.29의 비선택 dependency이며 수정된 버전은 0.16.3 이상이다.

Decision:

UI dependency를 `ratatui = "0.30"`과 `crossterm = "0.29"`로 함께 유지한다. lockfile은 ratatui 0.30.2, crossterm 0.29.0을 고정하고 전이 의존성 `lru`는 `RUSTSEC-2026-0253`이 수정된 0.18.2 이상을 요구한다. `cargo audit`, `cargo deny check licenses bans sources`, crossterm 단일 버전 검증을 R1 gate에 포함한다.

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
- timeout의 단일 기본값은 connect 500ms, narrative 2000ms, decision/soft-adjudication 1500ms이며 helper와 env config가 같은 상수를 사용한다. decision rationale은 trim 후 1..=160자다.
- v0.3.0 built-in runtime locale은 English로 한정한다. 다국어 README와 provider Unicode pass-through는 runtime 5-locale catalog 완료 증거가 아니다.
- deterministic loopback fixture와 PTY script는 success/timeout/stale/down 및 pending-exit 복원 순서를 저장소에서 재현한다. `audit_report_11.md` 독립 재감사에서도 같은 evidence가 통과했다.
- 실제 모델 smoke는 R6 필수 gate가 아니다. 최종 통합에서 별도 호환성 증거가 필요할 때만 localhost OpenAI-compatible 임시 adapter가 Google AI Studio Gemini 같은 원격 API를 대리 호출한다. AIHack은 계속 loopback만 호출하며 API key는 adapter 환경변수에만 주입하고 model ID는 실행 시점에 확인한다.

## ADR-0027: provenance approval이 runtime 포함의 선행 조건

Status: Implemented; owner license decision superseded by ADR-0030
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
- R7 engineering review는 공식 archive와 `dat/license` checksum을 검증하고 legacy direct import 0건을 자동화한다. 내부 build/test inclusion approval은 외부 배포 승인과 분리하며, content data와 scenario는 project owner 또는 적격 검토자가 범위를 승인할 때까지 `Reviewed`다.
- `audit_report_12.md` 이후 approval gate는 상태 문자열을 신뢰하지 않는다. runtime coverage, reviewer/date/license/scope/notice/evidence, content checksum, scenario schema/function과 Blocked reference를 모두 machine validation한 경우에만 PASS한다.
- `audit_report_13.md`의 phase-cycle 시정에 따라 R7은 asset/scenario provenance를, R8은 root distribution license/version/packaging을 소유한다. R7 PASS만으로 외부 배포를 허용하지 않는다.
- release checkpoint는 caller environment가 지정한 root를 신뢰하지 않고 script-relative canonical repository만 검사한다.
- release bundle 무결성은 Linux Bash와 Windows PowerShell verifier가 동일한 required file, excluded path, metadata/record equality, checksum exact-set와 tamper negative matrix를 갖는다. 한 OS의 positive build만으로 다른 OS의 fail-closed parity를 대신하지 않는다.

## ADR-0029: 미승인 provenance를 R8 실제 런칭 게이트로 이관

Status: Superseded for licensing by ADR-0030; R7 pass with known risks remains historical
Date: 2026-07-18
Decision ID: DEC-LICENSE-02

Context:

`audit_report_14.md`까지 R7의 checksum inventory, legacy 격리, compatibility trace, fail-closed validator와 전체 회귀는 검증됐지만 PROV-0004와 NH367 scenario 10개의 actual license approval는 남아 있다. 사용자는 실제 런칭이 개시될 때 이 범위를 명확히 검토하고, 현재 개발 단계에서는 최종 검토사항으로만 유지하기로 결정했다.

Decision:

R7 engineering 단계는 `PASS WITH KNOWN RISKS`로 종결한다. SC-LICENSE-01, content/scenario actual approval, root distribution license와 notice는 R8 실제 런칭 전 필수 게이트로 이관한다. 이 결정 당시에는 `Reviewed`, `UNLICENSED`, `publish = false`와 외부 배포 차단을 유지했다. 이후 상태는 ADR-0030이 대체한다.

Alternatives Considered:

- actual approval 없이 `Approved`로 변경: 근거를 조작하므로 기각
- R7 전체를 계속 HOLD: 구현·테스트 진행과 외부 배포 위험을 불필요하게 같은 gate로 묶어 기각
- 라이선스 검토를 완전히 면제: 런칭 시 법적·배포 위험이 남아 기각

Consequences:

- R8 비배포 준비와 후속 개발은 진행할 수 있다.
- R7 PASS는 라이선스 적합성 보장이나 외부 배포 허가가 아니다.
- 당시 `scripts/r7_checkpoint.sh`의 HOLD는 실패가 아니라 R8에 남은 승인 evidence를 표시했다. ADR-0030 반영 후에는 PASS한다.
- 외부 배포 전 project owner 또는 적격 검토자가 approval authority/evidence를 기록해야 한다.
- 승인 불가 자산은 Blocked 처리하고 독립 작성 자산으로 교체한다.

## ADR-0030: NetHack 3.6.7 파생물 분류와 whole-work NGPL 배포

Status: Implemented; report 21 and report 24 historical closure retained, report 25 current HOLD
Date: 2026-07-20
Decision ID: DEC-LICENSE-03

Context:

AIHack은 NetHack 3.6.7 원본 소스를 AI 추론에 제공해 의도와 관찰 가능한 규칙을 추출한 뒤 Rust 구조로 재작성했다. 결과 소스의 표현과 아키텍처가 크게 다르더라도 프로젝트 소유자는 원 저작권을 존중해야 하는 파생물로 취급하기로 명시했다. 따라서 clean-room 독립 저작물 전제를 유지하거나 permissive license를 선택하는 것은 실제 생성 과정과 소유자의 위험 판단에 맞지 않는다.

Decision:

AIHack 전체를 NetHack 3.6.7의 AI-assisted semantic rewrite 파생물로 분류하고 SPDX `NGPL`을 workspace 전체에 적용한다. 이 project-owner 결정의 범위와 직접 지시는 `AIHACK-OWNER-2026-07-20-NGPL-01`로 기록한다. root `LICENSE`는 공식 3.6.7 archive의 `dat/license`와 byte-for-byte 동일한 원문을 사용하고 SHA-256 `93a3ae2cb8dee482daddfaebe53bcffe5b114b603def19b4dca21621cbc5a747`로 고정한다. `NOTICE`에는 원 저작권, 파생·수정 사실과 AIHack 기여를 명시한다. 바이너리 배포에는 `LICENSE`, `NOTICE`, `MODIFICATIONS.md`, `PROJECT_OWNER_LICENSE_APPROVAL.md`, commit이 확장된 `RELEASE-METADATA`, `SHA256SUMS`와 해당 바이너리를 만든 커밋의 complete corresponding source archive를 동반한다. metadata의 필수 key는 각각 정확히 한 번 존재하고 owner/modification ID 전체 값은 함께 배포되는 record ID와 일치해야 한다. 검증되지 않은 `legacy_nethack_port_reference/`는 release archive에서 제외한다.

Alternatives Considered:

- MIT/Apache-2.0 등 permissive license 적용: 소유자의 파생물 분류와 NGPL whole-work 조건에 맞지 않아 기각
- 코드 표현이 다르다는 이유로 독립 저작물로 선언: 원본 소스 기반 의도 추론이라는 실제 생성 과정을 축소하므로 기각
- 손상된 legacy `LICENSE.NGPL` 복구본 사용: 공식 원문이 아니며 보존 증거를 훼손하므로 기각
- 라이선스 결정을 외부 게시 직전까지 다시 연기: 이미 소유자 판단이 확정되어 문서·manifest 불일치를 지속할 이유가 없어 기각

Consequences:

- 모든 workspace package는 version 0.3.0, `license = "NGPL"`, `publish = false`를 유지한다.
- PROV-0001..0012와 NH367-C001..C010은 소유자 승인 authority, 날짜, scope, notice와 `AIHACK-OWNER-2026-07-20-NGPL-01` evidence를 기록한다.
- R8 checkpoint는 NGPL 정확성, 공식 LICENSE checksum, NOTICE, modification manifest, release commit metadata와 source archive 계약을 fail-closed로 검사한다.
- source archive는 `.git/` history에 의존하지 않고 `MODIFICATIONS.md`의 scope/date와 `RELEASE-METADATA`의 commit을 수신자에게 전달한다.
- 라이선스 정비 완료는 독립 R8 기술 감사 `PASS`나 외부 게시 실행을 자동 승인하지 않는다.
- 이 결정과 프로젝트 기록은 qualified legal opinion이나 법률 자문을 대체하지 않는다.
- Release verification update (2026-08-23): `audit_report_21.md`가 report 20을, `audit_report_24.md`와 implementation SHA `2519bc8e0ede81c39f46b5778e62a41d4ca66901`의 Actions run `32107862171`이 report 23/24를 historical closed했다. final multi-audit report 1의 첫 시정은 `docs/audit/audit_report_25.md`가 production 결함을 재현해 partial evidence로 강등했으며, report 25 시정·same-SHA CI·독립 재감사 전까지 전체 PASS나 외부 게시를 승인하지 않는다.
