# Legacy NetHack Port Reference Index

작성일: 2026-04-28

## 역할

이 폴더는 AIHack의 이전 Rust NetHack 포트 전체를 보존한 참조 베이스다. 새 런타임의 빌드 대상이 아니며, 새 엔진은 이 폴더 안의 코드를 직접 import하지 않는다.

이 폴더의 목적은 네 가지다.

- NetHack 3.6.7 동작을 Rust로 옮기며 축적된 도메인 지식 보존
- 기존 테스트와 순수 함수 구현을 새 엔진 설계의 참고 자료로 사용
- 과거 구조적 리스크와 실패 원인을 추적
- 새 AIHack 문서세트가 원본 포트의 진척과 예정 범위를 빠뜨리지 않도록 보조

## 보존된 주요 자산

| 경로 | 용도 | 새 엔진 사용 방식 |
| --- | --- | --- |
| `src/` | 이전 Rust 포트 소스 | 직접 의존 금지, 규칙/타입/테스트 참고 |
| `assets/data/` | 몬스터/아이템 TOML | 검토 후 새 데이터 스키마로 변환 |
| `assets/dat/` | 루머/오라클/비문 텍스트 | 라이선스 확인 후 narrative 데이터 후보 |
| `Cargo.toml`, `Cargo.lock` | 이전 의존성/빌드 상태 | 새 Cargo 설정의 참고 자료 |
| `README.md` | 이전 프로젝트 설명 | 새 README와 혼동 금지 |
| `IMPLEMENTATION_SUMMARY.md` | 이전 포트 진척 기록 | 기능 범위 누락 방지용 참고 |
| `STABILIZATION_ROADMAP.md` | 이전 런타임 안정화 기록 | 구조적 리스크 참고 |
| `DESIGN_DECISIONS.md` | 이전 의사결정 | 반복 논쟁 방지 |
| `ROGUELIKE_LOGIC_REFERENCE.md` | 로그라이크 규칙 참고 | 새 spec 작성 보조 |
| `AIHACK_REWRITE_VS_REPAIR_REPORT.md` | 재구현 판단 리포트 | 새 방향의 근거 |

## 재사용 원칙

새 엔진에서 재사용 가능한 것:

- 순수 함수 형태의 수식, 확률, 분류 로직
- `ItemKind`, `MonsterKind`, 역할/종족/상태 같은 닫힌 enum 후보
- deterministic RNG 유틸 아이디어
- 기존 단위 테스트의 입력/출력 사례
- NetHack 특수 규칙을 설명하는 문서 문장
- 아이템/몬스터/텍스트 자산의 데이터 항목

새 엔진에서 그대로 가져오면 안 되는 것:

- `game_loop.rs`의 직접 오케스트레이션 구조
- Legion `World`/`Resources`에 강하게 묶인 시스템
- `self.game.grid`와 `resources.Grid` 같은 이중 상태 구조
- UI가 게임 상태를 직접 수정하는 경로
- `ActionQueue`/`EventQueue`가 여러 리소스에 분산되는 기존 런타임 계약
- `phaseNN`, `_ext` 누적 방식의 파일 증식 패턴

## 라이선스 경계

이 폴더는 NetHack 3.6.7 파생 코드와 데이터를 포함한다. 기존 라이선스 파일은 다음 위치에 보존되어 있다.

- `LICENSE`
- `LICENSE.NGPL`

새 AIHack 런타임은 이 폴더의 코드를 복사하기 전에 라이선스 영향을 확인해야 한다. 새 엔진이 NetHack 파생물로 남을지, NetHack-inspired 독립 게임으로 갈지는 루트 `spec.md`와 `DESIGN_DECISIONS.md`의 정책을 따른다.

## 현재 판단

이전 포트는 폐기 대상이 아니라 참조 자산이다. 다만 새 실행 엔진의 기반으로 계속 밀고 가기에는 다음 구조적 리스크가 크다.

- 실제 플레이 루프가 `src/game_loop.rs`에 집중되어 있다.
- 순수 함수 테스트는 많지만 실행 경로와 미연결인 섬 코드가 많다.
- 문서상 100% 이식 주장과 미구현 핵심 시스템 목록이 공존한다.
- Legion 런타임 borrow conflict와 Grid 동기화 문제가 과거 안정화 문서에 반복 기록되어 있다.
- AI 관찰/행동 인터페이스가 엔진의 1급 계약으로 정의되어 있지 않다.

따라서 새 작업은 이 폴더를 레퍼런스로 읽되, 루트 문서세트에 정의된 Rust-native 엔진으로 진행한다.
