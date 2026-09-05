# P1–P3 구현·검증 기록

2026-09-05. 기준 작업은 미커밋 P0 위의 사용자 P3까지 구현 요청이다. 계약은 `docs/campaign_spec.md`. 봉인 감사 원문은 변경하지 않았다.

## 구현

- P1: core `campaign.rs` Role/CampaignState, runtime bootstrap/XP 소비와 save validator, TUI 역할 keyboard/mouse 및 HUD.
- P2: runtime `campaign_map.rs`의 combat RNG와 독립인 40×20 연결 방/통로 생성, Main 1..6/Mines 1..2, Main 3 branch 입출구와 실제 생성 좌표 teleport.
- P3: content `item.quest.ascension` → typed Quest item → Pickup/Drop/Inventory → Main 1 Ascend → Victory/score → 결과 화면/새 게임. Headless는 Victory와 사망을 구분한다.
- Save: legacy None은 schema 1/생략 field로 보존, campaign은 schema 2. 구형 reader는 새 저장을 거부한다. 기존 runtime capability/atomic 파일 쓰기 및 mutable API 경계는 유지한다.

## 재현 증거

- 신규 타입/StartCampaign/Amulet/Victory 부재의 compile RED, 역할 성장 및 위조 XP 승인 behavioral RED를 확인하고 수정했다.
- 첫 정상 완주가 legacy 무게 80에서 막혔다. campaign 120 계약을 문서화하고 actual movement/observation/HUD에 연결했다. 과다 loot는 테스트도 실제 Quaff/Drop으로 관리하며 guard를 우회하지 않는다.
- `tests/campaign.rs`: 3 seed/3 role 실제 명령 완주, 광산 왕복, 목표 drop/pickup, 저장·RNG 연속성, 마지막 Ascend replay, 반복 최종 hash, UI 최소 크기 60×24 role/새 게임, 성장 후 실제 attack roll +1, 위조 schema/XP/stat/amulet/topology/Victory, teleport landing 검증.
- Windows ConPTY: legacy와 campaign 각각 서로 다른 두 실제 CLI 프로세스 저장→종료→재시작→추가 Wait의 최종 hash 일치.
- content 체크섬 gate가 신규 quest 데이터의 미반영 manifest를 검출했다. 원래 10개 item 정의는 보존하고 신규 데이터 출처·이전/현재 SHA를 PROVENANCE에 기록한 뒤 manifest를 갱신했다. checksum negative tests는 유지한다.
- content crate의 registry golden hash도 신규 item 포함에 따라 `c491b83c6f499a62` → `f106d044fee3e340`으로 변경된다. 기존 item/monster/level 조회와 정확한 hash assertion을 유지하고 기대값만 새 콘텐츠 기준으로 갱신한다. legacy world snapshot hash와 별개의 콘텐츠 registry hash다.

## Gate 결과

- `cargo test --workspace --all-targets --locked --no-fail-fast`: **491 PASS / 0 FAIL / 0 ignored**, 89개 test binary, exit 0. 기존 P0 478개와 캠페인 12개·V2 ConPTY 1개를 포함한다.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS.
- `cargo build --workspace --all-targets --release --locked`: PASS.
- 표적 campaign 12 tests 및 legacy/V2 ConPTY 4 tests: PASS.
- 전체 로그: `runtime/campaign-evidence/workspace-tests.log`, SHA256 `464a753f8561c0d5ba3564d325c8434b96f41a304d3620cf6e9594b15fa35317`. 최종 검증 OS는 Windows다.

P1–P3의 명시된 첫 캠페인 범위는 local verified로 완료했다. 독립 감사 PASS나 NetHack 전체 구현 완료 선언과는 구분한다.

실제 headless CLI: seed 42/Knight/wait-v1 10턴 저장 후 별도 실행에서 12턴까지 복원 진행(2 accepted/2 submitted)과 새 게임에서 직접 12턴 진행(12 accepted/12 submitted)의 최종 hash는 모두 `8236e9774f4b6d40`이다. report는 `runtime/campaign-evidence/headless-start.json`, `headless-resume.json`, `headless-direct.json`에 생성했다. runtime 증거는 Git 추적 대상이 아니다.

리뷰: generator는 새 모듈, 성장·semantic 검증은 `campaign.rs`로 분리했다. 기존 10개 item 값, 기존 2층 생성, V1 field/hash, legacy 운반 80은 유지한다. 신규 XP 예산 누적은 saturating arithmetic, 승리 score 검사는 기존 범위 검증 이후 수행하여 malformed save가 overflow/panic을 만들지 않도록 했다. 운영 계정·네트워크·인증 설정은 변경하지 않았다.

## 제한과 인수인계

첫 bounded campaign이며 NetHack 전체 직업/마법/종족/분기/신앙/원소계 구현을 뜻하지 않는다. 생성 map에는 현재 문·함정이 없고 V2는 generator v1 map을 검증한다. save의 semantic validator는 로컬 사용자의 완전한 파일 위조를 막는 암호학적 인증 수단이 아니다. XP cap은 180이며 실제 8층 콘텐츠의 보상 총량에 따라 도달 레벨은 더 낮을 수 있다.

release/게시 승인, modification notice와 final-SHA bundle 갱신, 새 양 OS CI는 별도 작업이며 이번 기능 검증으로 승인하지 않는다. 커밋·푸시는 실행하지 않았다.
