# P1–P3 체크리스트

- [x] P1a 타입: Role/CampaignState + world/save/snapshot. 기존 V1/hash 및 성장 공식 검증.
- [x] P1b runtime: StartCampaign(role), 역할 장비/stat, kill XP. 역할 차이·중복 보상·저장 검증.
- [x] P1c UI: 생성 역할 선택·HUD. keyboard/mouse dispatcher 검증.
- [x] P2a 생성: seed별 connected 6+2층. BFS·동일/다른 seed 검사.
- [x] P2b 이동: 계단·분기·재진입 저장. 모든 왕복·replay 검증.
- [x] P3a 목표: 고유 AmuletAscension item. pickup/drop/recover와 forged save 거부.
- [x] P3b 승천: Ascend→Victory→결과→새 게임. 조기/중복 성공 거부와 실제 명령 완주.
- [x] 최종: 기존 478 회귀 포함 총 491 tests, fmt/clippy/release PASS. 증거: `docs/campaign_implementation.md`.
