# P1–P3 구현 계획

2026-09-05. 사용자 요청: P3까지 구현. 기존 미커밋 P0 및 봉인 원본을 보존한다. 기준: `spec.md`, `docs/campaign_spec.md`.

| 기능 ID | 책임 | 선행 |
| --- | --- | --- |
| roles-growth | 역할·XP·저장·관찰 | 기존 submit |
| exploration | 6층+광산 2층 생성·왕복 | roles-growth |
| ascension | 목표 물건·귀환·성공 결과 | exploration |

타입→runtime→UI의 작은 단위마다 표적 테스트, 마지막에 전체 테스트/fmt/clippy/release를 실행한다. 구형 save/hash는 campaign=None 생략으로 보존하고 새로운 저장은 wire schema 2로 구분한다. 위조 성장/목표와 단절된 dungeon은 validator/BFS/실제 명령 완주 테스트로 방어한다. 커밋·푸시·배포는 이번 범위가 아니다. 체크리스트는 `tasks/todo.md`, 결과는 `docs/campaign_implementation.md`.

# P4 구현 계획 (2026-11-26)

기준: `spec.md`, `docs/campaign_spec.md` (P4), `DESIGN_DECISIONS.md` (`ADR-P4-01`). 커밋·푸시는 수행하지 않는다.

| 단계 | 책임 | 핵심 대상 |
| --- | --- | --- |
| P4a | `TrapKind::Dart/Bear`, 캠페인 문/비밀문/함정 생성, 전이-인식 V2 맵 검증 | `core/tile.rs`, `runtime/campaign_map.rs`, `runtime/save.rs`, `runtime/systems/traps.rs`, `runtime/systems/movement.rs`, `runtime/session.rs` |
| P4b | `Role::Cleric`, 신규 아이템 5종(`RechargeWand` 포함), 몬스터 3종, 캠페인 배치 | `core/campaign.rs`, `core/domain/*`, `aihack-content/*`, `runtime/bootstrap.rs`, `runtime/systems/items.rs`, `runtime/observation.rs`, `PROVENANCE.md`, `r7-content.sha256` |
| P4c | TUI 문/함정/아이템/몬스터/Cleric 렌더·입력 및 Headless `--role cleric` | `apps/aihack-tui/*`, `apps/aihack-headless/*` |
| P4d | 통합/부정 테스트, 전체 gate(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`) 및 문서 동기화 | `tests/campaign.rs`, `docs/campaign_implementation.md`, `IMPLEMENTATION_SUMMARY.md`, `README.md`, `CHANGELOG.md`, `tasks/todo.md` |

# P5 구현 계획 (2026-11-26)

기준: `spec.md`, `docs/campaign_spec.md` (P5), `DESIGN_DECISIONS.md` (`ADR-P5-01`). 커밋·푸시는 수행하지 않는다.

| 단계 | 책임 | 핵심 대상 |
| --- | --- | --- |
| P5a | `BucStatus`/`enchantment`, 별도 `Entity` 접근자, `DoorState::Locked` 및 `kick_door`·전투/AC/저장 검증 | `core/domain/*`, `core/doors.rs`, `runtime/systems/{combat,items,projectiles}.rs`, `runtime/campaign_map.rs`, `runtime/save.rs` |
| P5b | `poison_turns`/`DeathCause::Poison`, 신규 5아이템(인챈트/해저주 두루마리 3종·`PotionCure`·`WandFire`), 신규 3몬스터(`GiantSpider`·`GnollArcher`·`Troll`), content/provenance 동기화 | `core/world.rs`, `core/save.rs`, `aihack-content/*`, `runtime/systems/monster_ai.rs`, `runtime/session.rs`, `runtime/bootstrap.rs`, `PROVENANCE.md`, `r7-content.sha256` |
| P5c | TUI(`locked door`, `POIS:`, BUC/인챈트 라벨, 몬스터/아이템, `DeathCause::Poison`) 및 Headless `survival-v1` 연동 | `aihack-ai-contract/*`, `apps/aihack-tui/*`, `apps/aihack-headless/*` |
| P5d | 회귀/통합 테스트, 전체 gate(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`) 및 문서 동기화 | `tests/campaign.rs`, `tests/content_validation.rs`, `tests/campaign_p5.rs`, `docs/campaign_implementation.md`, `IMPLEMENTATION_SUMMARY.md`, `README.md`, `CHANGELOG.md`, `tasks/todo.md` |
