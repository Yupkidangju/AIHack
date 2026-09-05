# P1–P3 구현 계획

2026-09-05. 사용자 요청: P3까지 구현. 기존 미커밋 P0 및 봉인 원본을 보존한다. 기준: `spec.md`, `docs/campaign_spec.md`.

| 기능 ID | 책임 | 선행 |
| --- | --- | --- |
| roles-growth | 역할·XP·저장·관찰 | 기존 submit |
| exploration | 6층+광산 2층 생성·왕복 | roles-growth |
| ascension | 목표 물건·귀환·성공 결과 | exploration |

타입→runtime→UI의 작은 단위마다 표적 테스트, 마지막에 전체 테스트/fmt/clippy/release를 실행한다. 구형 save/hash는 campaign=None 생략으로 보존하고 새로운 저장은 wire schema 2로 구분한다. 위조 성장/목표와 단절된 dungeon은 validator/BFS/실제 명령 완주 테스트로 방어한다. 커밋·푸시·배포는 이번 범위가 아니다. 체크리스트는 `tasks/todo.md`, 결과는 `docs/campaign_implementation.md`.
