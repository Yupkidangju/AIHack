# P1–P3 체크리스트

- [x] P1a 타입: Role/CampaignState + world/save/snapshot. 기존 V1/hash 및 성장 공식 검증.
- [x] P1b runtime: StartCampaign(role), 역할 장비/stat, kill XP. 역할 차이·중복 보상·저장 검증.
- [x] P1c UI: 생성 역할 선택·HUD. keyboard/mouse dispatcher 검증.
- [x] P2a 생성: seed별 connected 6+2층. BFS·동일/다른 seed 검사.
- [x] P2b 이동: 계단·분기·재진입 저장. 모든 왕복·replay 검증.
- [x] P3a 목표: 고유 AmuletAscension item. pickup/drop/recover와 forged save 거부.
- [x] P3b 승천: Ascend→Victory→결과→새 게임. 조기/중복 성공 거부와 실제 명령 완주.
- [x] 최종: 기존 478 회귀 포함 총 491 tests, fmt/clippy/release PASS. 증거: `docs/campaign_implementation.md`.

# P4 체크리스트 (2026-11-26)

- [x] P4-doc: `docs/campaign_spec.md`, `spec.md`, `DESIGN_DECISIONS.md` (`ADR-P4-01`), `tasks/plan.md`, `tasks/todo.md` 문서 선행 확정.
- [x] P4a: `TrapKind::Dart/Bear`, `campaign_map` 문/비밀문/함정 배치, `validate_persisted_map` 전이 검증.
- [x] P4b: `Role::Cleric`, 신규 5아이템(`RechargeWand` 포함), 신규 3몬스터, `PROVENANCE.md`·`r7-content.sha256`·`registry.rs` hash 동기화.
- [x] P4c: TUI(`render_map`, `render_panels`, `input`, `labels`, `mod`) 및 Headless(`--role cleric`, `PotionExtraHealing`) 연동.
- [x] P4d: `tests/campaign.rs` 4역할 완주 및 문/비밀문/함정/신규 아이템·몬스터/위조 저장 회귀 테스트.
- [x] P4-verify: `fmt`, `clippy`, `test -j 4`, `release`, `cargo-audit`, `cargo-deny`, `r7_checkpoint`, `r8_checkpoint`, headless smoke & 최종 문서 마감.

# P5 체크리스트 (2026-11-26)

- [x] P5-doc: `docs/campaign_spec.md`, `spec.md`, `DESIGN_DECISIONS.md` (`ADR-P5-01`), `tasks/plan.md`, `tasks/todo.md` 문서 선행 확정.
- [x] P5a: `BucStatus`/`enchantment`, `Entity` 별도 접근자, `DoorState::Locked` 및 `kick_door`/전이 검증, 인챈트·저주 해제 두루마리 3종.
- [x] P5b: `poison_turns`/`DeathCause::Poison`, `PotionCure`, `WandFire`, `GiantSpider`/`GnollArcher`/`Troll`, `PROVENANCE.md`·`r7-content.sha256`·`registry.rs` hash 동기화.
- [x] P5c: TUI(잠긴 문, `POIS:`, BUC/인챈트 라벨, 신규 몬스터/아이템, `DeathCause::Poison`) 및 Headless `survival-v1` 연동.
- [x] P5d: `tests/campaign.rs`, `tests/content_validation.rs`, `tests/campaign_p5.rs` 회귀·통합 테스트 및 전체 검증(`fmt`, `clippy`, `test -j 4`, `release`, `audit`, `deny`, `r7/r8`)·최종 문서 마감.
