# AIHack 게임 완결성·키보드·마우스 멀티 감사 2

## 1. Audit Metadata

- 일자: 2026-09-05, Asia/Seoul
- 프로젝트: `C:\LocalDev\rust\AIHack`, Rust 0.3.0, TUI/headless workspace
- 감사 HEAD: `899660167d59c4b06d27a59c0d75fcccda0cce33`
- 기준: `AI_AUDIT_DOC_STANDARD.md`, `spec.md`, `designs.md`, 명시 호출된 `multi-audit` 스킬과 보고서 계약
- 방식: 독립 에이전트 4명(`gpt-5.6-luna`, reasoning `max`) + 메인 직접 코드·Windows PTY 검증
- 시작 상태: clean. 종료 상태: 추적 파일 변경 없음, 이번 `docs/multi_audit/2/`만 추가
- 판정 범위: 게임의 시작·진행·종료, 실제 플레이 기능의 입력·표시·복원 경로
- 최종 판정: **HOLD — NetHack 3.6.7에 준하는 완결 게임과 전 기능 입력 지원 목표 미충족**

## 2. User Goal and Decision Basis

사용자 목표는 “게임 루프 및 시작→진행→끝이 NetHack 3.6.7에 준하는지”와 “모든 기능을 키보드와 마우스로 정상 플레이할 수 있는지”다.

**현재 상태는 일부 NetHack 규칙을 구현한 고정 두 층의 게임이다. 사망·재시작 흐름은 동작하지만 목표 달성으로 끝나는 완주 경로가 없다. 키보드 조작도 일부 기능의 방향·아이템을 지정할 수 없고, 마우스는 시작과 주요 창 조작에서 중단된다.** 혼합 입력으로 해석해도 맵 정보 누락과 명령 도달성 문제는 남는다.

이번 감사는 두 기준을 구분한다.

1. 기존 v0.3.0 계약 준수: 제한된 NH367-C001..C010, deterministic turn, save/RNG, terminal/LLM 경계.
2. 사용자 비교 목표 충족: 선택·탐험·성장·목표·성공/실패 종료와 실제 입력 도달성.

기존 명세가 의도한 축소를 회귀 버그로 단정하지 않는다. 비교상 없는 기능은 `Needs Spec Clarification`과 **목표 격차**로 표시하며, 이미 구현된 기능의 입력·표시 단절은 `Needs Fix`로 분리한다. 정확히 모든 NetHack 콘텐츠를 복제해야 한다거나 모든 역할 수를 맞춰야 한다는 새 요구를 만들지 않는다.

공식 비교 근거는 [NetHack 3.6.7 Guidebook](https://www.nethack.org/v367/Guidebook.html)과 [3.6.7 태그의 end.c](https://raw.githubusercontent.com/NetHack/NetHack/NetHack-3.6.7_Released/src/end.c), [pray.c](https://raw.githubusercontent.com/NetHack/NetHack/NetHack-3.6.7_Released/src/pray.c)다. Guidebook은 캐릭터 선택, 생성되는 던전, 경험 성장, 저장 후 재개를 설명한다. end.c의 종료 목록과 pray.c의 `done(ASCENDED)`는 사망·포기·탈출·승천을 구분한다. 3.7 자료를 비교 기준으로 사용하지 않았다.

| 비교 축 | AIHack 확인 결과 | 사용자 목표 판정 |
| --- | --- | --- |
| 시작 | Title→Enter→고정 Adventurer 확인→Enter→Playing | 기본 진입은 가능, 캐릭터 선택 없음 |
| 탐험 | `main:1`, `main:2` 고정 40×20 지도와 왕복 계단 | 일부 탐험만 가능 |
| 성장 | 음식·장비·물약·시체·기도·점수 효과 존재, XP/경험 레벨/숙련 없음 | 자원 변화는 있으나 장기 성장 미구현 |
| 목표와 성공 종료 | Amulet, 목표 획득·제공, 탈출/승천 결과 없음 | 완주 불가 |
| 실패 종료 | 전투/함정 사망→GameOver→N→Title | 직접 실행으로 확인 |
| 저장 후 재개 | runtime 저장은 검증됨. TUI는 실행별 TempDir의 quick-save | 프로세스 재시작 후 TUI 이어하기 불가 |
| 키보드 | 이동·기본 명령 일부 가능. 방향·물건 선택 제한, q 충돌 | 전 기능 지원 아님 |
| 마우스 | 인접 이동·일부 CTA 가능. Title/Creation/Inventory/GameOver 등 차단 | 전 과정 플레이 불가 |

## 3. Scope and Exclusions

포함: 기본 콘텐츠, runtime 상태 전이·명령·사망·계단·저장, headless 진행 목표, observation/action-space, TUI event→candidate→handler→render, 관련 기존 테스트와 Windows 실제 pseudoconsole 실행.

제외: 이전 Report 33의 릴리스 승인 문서 이관 항목, 새 배포/CI 감사, 의존성 전수 재감사, 유료 provider, legacy reference 전체, 새 기능 구현. 이전 릴리스 이관을 이번 플레이 감사의 원인으로 사용하지 않는다.

물리 장치별 클릭/hold, Linux 실제 화면, 모든 터미널 조합은 미검증이다. 이는 하드웨어 전체 지원을 보장하지 않는 한계이며, 코드상 도달할 수 없는 기능에 대한 부정 판정을 뒤집지는 않는다.

## 4. Work-Surface Inventory

| 작업면 | 주요 확인 파일 |
| --- | --- |
| 목표/설계 | `spec.md`, `designs.md`, `README.md`, `IMPLEMENTATION_SUMMARY.md`, `BUILD_GUIDE.md`, compatibility 10 records |
| 시작/월드 | `crates/aihack-runtime/src/bootstrap.rs`, `world.rs`, `session.rs`; `crates/aihack-content/src/lib.rs`, `data/levels/main_1.toml`, `main_2.toml` |
| 명령/상태/종료 | `crates/aihack-core/src/action.rs`, `run_state.rs`, `domain/player.rs`, `domain/combat.rs`, `world.rs`; runtime systems/stairs/death/items |
| 입력/표시 | `apps/aihack-tui/src/tui/{mod,input,render_map,render_panels,layout,labels,viewport}.rs` |
| 저장/자동 진행 | runtime `save.rs`, `observation.rs`; headless `src/lib.rs`, `main.rs` |
| 검증 | ui_screens, ui_input_mapping, ui_layout, ui_runtime_smoke, stairs, long_run, release_candidate, nethack_367_compat, save_load, replay, transaction, golden_phase8_rules, package TUI/ConPTY tests |

`target/`는 기존 검사와 실행이 생성한 캐시·바이너리로만 사용했다. Git 이력은 기준 SHA 및 추적 파일 변경 확인에 사용했다.

## 5. Agent Allocation and Rationale

| 감사자 | 독립 질문 | 결과 |
| --- | --- | --- |
| A01 game_contract | 실제 시작·성장·목표·끝이 3.6.7의 게임 구조에 준하는가 | 완료, 공식 자료·코드·표적 테스트 |
| A02 runtime_loop | 상태 전이와 자동 진행/저장 검증이 실제 진행을 증명하는가 | 완료, 실행·코드 경로 |
| A03 recovery | 종료·취소·저장·LLM·터미널 복원이 진행을 보호하는가 | 완료, 코드·표적 테스트 |
| A04 input_ui | 모든 명령이 키보드·마우스·화면에서 도달 가능한가 | 완료, 전체 명령표·코드·표적 테스트 |

기본 세 관점에 전수 입력·렌더 연결이라는 독립 질문을 더해 4명을 사용했다. 저장/데이터 무결성은 A02와 A03이 독립적으로 확인했다. 메인은 공식 자료, 모든 채택 Major의 파일/호출 경로와 실제 TUI를 재확인했다.

## 6. Immutable Source Report Manifest

원본과 보완 보고서는 읽기 전용으로 봉인했다. 원본의 부정확한 세부 표현은 고치지 않고 §11~12에서 판정했다.

| 파일 | SHA-256 |
| --- | --- |
| `sub_audit_01_game_contract.md` | `405cf5c1f46e159e1a587ce4504d1703924bd86a2f90dc1765a84274500fbad3` |
| `sub_audit_02_runtime_loop.md` | `a562c8bbdfe4d0911db2d9bf81d69fcf490b548758e813da65df576d097dd6d9` |
| `sub_audit_03_recovery.md` | `b729e18e0d95886d91de33a89f1f392f97e8d3438fed9cc34c9b8173d370daa3` |
| `sub_audit_04_input_ui.md` | `be40bfa84c34a03683d085d37cbebb8c50b364be28cdcd16434bd8746168d7e5` |
| `sub_audit_04_input_ui_supplement_1.md` — 메인 실제 PTY 증거 | `2584fd31dfa02317c27d1a0797c56c12425c862cbb8e9b8b5b8a3b71be138a2d` |

- Manifest: `C:\LocalDev\rust\AIHack\docs\multi_audit\2\source_report_manifest.json`
- Manifest SHA-256: `aa771174e6d6e9f8c0221491e37d04da08bb9859e7ea7c168840a461d249cbf2`
- Sidecar: `C:\LocalDev\rust\AIHack\docs\multi_audit\2\source_report_manifest.sha256.json`
- `missing_source_reports`: `[]`
- 통합 직전 verify: PASS. 통합 저장 후 동일 명령으로 재검증한다.

## 7. Evidence and Commands

메인이 실행한 `cargo test -p aihack-tui --all-targets --locked`는 library 4 + main 1 + ConPTY 2 + contract 20 = **27 PASS**였다. 독립 감사자들이 실행한 root UI, compatibility, stairs, long-run, save/replay, transaction, headless 표적 검사도 원본 보고서에 모두 PASS로 기록됐다. 일부 명령은 감사자 사이에 중복되므로 합계를 부풀리지 않는다. 이번에 full workspace 453개를 다시 실행했다고 주장하지 않는다.

메인 실제 실행: `.\target\debug\aihack.exe --seed 42`, Windows PTY, 출력 기준 80×24.

| 순서 | 입력/관찰 | 결과 |
| --- | --- | --- |
| 시작 | Enter, Enter | Title→Creation→Playing, HP16/16, turn0 |
| 플레이 | SGR 인접 cell click | turn1, HP14/16. 적 glyph는 없고 임시 `[` 표시만 발생 |
| inventory | i, 내부 item cell 클릭, Esc | 4개 항목 표시, 클릭 무반응, Esc 복귀 |
| 사망 | `.` 연속 입력 | turn19 GameOver, attacker2, score384 |
| 재시작 | New Run 텍스트 클릭, N | 클릭 무반응, N으로 Title 복귀 |
| 마우스 시작 | Title 시작 텍스트 클릭 | 무반응 |
| 종료 | q | exit0, cursor/alternate-screen 복원 출력 |

이 실행은 **사망 경로**의 시작→진행→끝을 확인한다. 승리 경로와 전체 입력 지원의 증거는 아니다. 80×24 LOG는 제목만 표시했고 초기 적·바닥 아이템은 코드에 존재하지만 지도에서 식별되지 않았다.

절차상 제한: A02는 사전 지시보다 범위를 넓혀 프로젝트 밖 일회성 runtime에 headless CLI 결과를 생성·정리했다고 보고했다. 임시 원본 로그가 봉인되지 않았으므로 해당 동적 주장을 메인의 독립 실행으로 계산하지 않는다. 낮은 target finding은 메인이 runner·CLI·BUILD_GUIDE를 다시 읽어 확인했다. 이 절차 일탈은 제품 결함과 분리한다. 메인이 확인한 저장소 추적 파일 변경과 원본 해시 불일치는 없다.

## 8. Coverage Gap Check

| 질문 | 독립 근거와 메인 확인 | Coverage | 남은 한계 |
| --- | --- | --- | --- |
| 시작 선택/고정 캐릭터 | A01/A02, player/bootstrap/session, 실제 Creation | Covered | 전체 역할 구현은 존재하지 않음 |
| 탐험·성장·성공 종료 도달성 | A01/A02, registry/stairs/state/death, 공식 자료 | Covered | 승리 실행 불가는 구현 부재로 확인 |
| 사망·재시작·종료 | A02/A03/A04, 메인 actual PTY | Covered | 물리 GUI 전체 조합 미검증 |
| 각 명령의 키보드·마우스 경로 | A01/A04 전수표, 메인 dispatcher/input/handler | Covered | 미지원 조합을 성공 실행한 것으로 계산하지 않음 |
| 적·아이템·로그 표시 | A04, 메인 renderer/projection/actual PTY | Covered | 큰 화면의 전체 장기 시각 QA 미실행 |
| 저장/복원 무결성 | A02/A03 독립 tests·save trace | Covered | 두 프로세스 TUI 복원 수동 실험 없음; 임시 저장 lifetime은 코드로 확인 |
| 상태별 자동 진행 | A02, 메인 legality/runner direct trace | Covered | 마비 조합 새 E2E probe는 미실행 |
| 물리 hold·플랫폼별 모든 터미널 | 부분 ConPTY만 | Partially Covered | 보편적 장치 지원 PASS를 주장하지 않음 |
| 이전 release 문서/배포 | 이번 사용자 목표와 무관 | Excluded | 이전 이관 유지 |

핵심 부정 판정은 실제 증거가 있는 도달성·표시 결함에 근거한다. 플랫폼 전체에 대한 미검증을 기능 부재로 바꾸지 않는다. 새로운 독립 보고서가 필요한 핵심 미확정 고위험 후보는 남기지 않았고, mouse hold 추정은 비차단 정보로 제한했다.

## 9. Canonical Findings

다음 ID는 이번 멀티 감사 2에만 해당한다. `Major / 목표 격차`는 현재 코드가 비교 목표를 충족하지 않는다는 의미이며 기존 v0.3.0 계약의 회귀 결함과 다르다.

### [MA2-F001] 두 고정 층 이후의 목표·성공 종료가 없음

- Sources: A01-F002, A02의 완결성 분석
- Pass / Pattern: Implementation / IMP-002, SPEC-GAP-001
- Severity / Status: **Major / Needs Spec Clarification — 목표 격차 확인**
- Verified Evidence: content `src/lib.rs:39-48`의 두 level include, `data/levels/main_2.toml`의 up stair만 존재, runtime `systems/stairs.rs:39-50`의 main:1 위 이동 거부, core `run_state.rs`/`action.rs`/`DeathCause`에 성공 outcome 없음.
- Expected Basis / Actual: 비교 목표는 살아서 달성할 목적과 끝을 필요로 하나, 현재는 두 층 왕복·사망·quit만 있다. 계단 `Ascend`는 승천 구현이 아니다.
- Impact: 시작→탐험→목표 달성→성공 종료의 완주가 불가능하다.
- Action / Owner: 제품·설계 담당자가 완결된 축소 게임의 목표/끝 또는 더 넓은 NetHack 목표를 명세화한 뒤 구현. 모든 원본 콘텐츠 복제를 자동 요구하지 않는다.
- Re-audit: 정상 새 게임에서 목표 획득→성공 조건→결과 화면→새 게임을 fixture 주입 없이 도달 가능한 명령열로 확인.
- Synthesis: 의도적 content slice는 인정하되 전체 게임 동등성은 기각한다.

### [MA2-F002] 캐릭터 선택과 경험 성장이 없음

- Sources: A01-F001, A01-F003; A02 시작 분석
- Pass / Pattern: Implementation / SPEC-GAP-001
- Severity / Status: **Major / Needs Spec Clarification — 목표 격차 확인**
- Verified Evidence: `domain/player.rs:3-20` 고정 Adventurer, `bootstrap.rs:38-96` 고정 시작, `session.rs:189-200` 확인만 하는 Creation, `render_panels.rs:345-359` 상수 화면; `systems/death.rs:59-65` 처치 보상은 kill_count/gold이고 XP/level 없음.
- Expected Basis / Actual: 3.6.7 비교의 선택·성장 축에 비해 캐릭터 선택과 XP/숙련 모델이 없다. 장비·음식·물약으로 AC/HP/자원이 변하는 기능은 존재한다.
- Impact: 역할 선택과 장기 성장에 따른 전략 차이를 제공하지 못한다.
- Action / Owner: 제품 담당자가 필요한 선택·성장 최소 범위를 정하고 typed state, save, 화면에 연결. 단일 캐릭터를 유지하면 이를 제품 범위로 명확히 표현.
- Re-audit: 선택이 시작 장비/능력에 영향을 주고 경험 변화가 후속 전투·상태에 소비되는지 확인.
- Synthesis: “모든 HP/AC가 고정”이라는 원본의 넓은 표현은 채택하지 않고 **XP/레벨 성장 부재**만 채택한다.

### [MA2-F003] q 물약 단축키가 즉시 종료에 가려짐

- Sources: A01-F005, A02-F006/F004, A03-F001/F002, A04-F002
- Pass / Pattern: Implementation / IMP-001, TEST-001
- Severity / Status: **Major / Needs Fix**
- Verified Evidence: `input.rs:194` q→Quit, `:220-232` 조기 반환, `:247` 이후 Quaff branch 도달 불가. `mod.rs:826`은 UI Quit에서 `Ok(true)`, `:1291-1294` loop 종료. core `submit_quit` 호출이 아니다.
- Expected / Actual: potion 사용 코드와 실제 단축키가 일치해야 하나 lowercase q는 프로그램을 닫는다. Playing uppercase Q는 현재 무매핑이며 Title/GameOver와 다르다. Inventory를 열고 해당 letter를 누르는 Quaff 우회 경로는 존재한다.
- Impact: 물약을 마시려는 키 입력이 확인 없이 run을 종료할 수 있다. 임시 quick-save와 결합하면 재개도 불가능하다.
- Action / Owner: 입력 담당자가 Quit/Quaff 키 소유권을 분리하고 Playing Esc/q, 종료 확인·결과 화면 정책을 일치시킬 것.
- Re-audit: potion 소지 상태의 key→candidate→handler가 potion 효과·turn을 만들고 exit하지 않는지, 종료키는 별도 의도한 결과를 만드는지 검사.
- Synthesis: 대체 letter 경로가 있다는 이유로 우발적 종료 위험을 Minor로 낮추지 않는다. 기존 Quit sentinel 자체는 명세상 호환 정책이므로 별도 회귀 버그로 세지 않는다.

### [MA2-F004] 방향·물건 선택과 중간 prompt가 실제 플레이에 연결되지 않음

- Sources: A01-F005, A02-F002, A04-F001/F006
- Pass / Pattern: Implementation / IMP-001, IMP-002
- Severity / Status: **Major / Needs Fix**
- Verified Evidence: `input.rs:181-186,261-278` Open/Close/Kick/Throw/Zap East 고정, Drop 첫 item; `session.rs:203-233`은 concrete command만 소비. Awaiting assignments는 이미 그 상태일 때의 재설정이며 MorePrompt producer 없음. `tests/ui_screens.rs:87-135`는 상태를 주입한다.
- Expected / Actual: core는 여러 방향·아이템 명령을 지원하고 spec §8은 선택 전이를 그리지만 일반 TUI는 방향/행동 대상 전체를 선택할 수 없다. 저장된 prompt 상태 처리만 있다고 진입까지 구현된 것은 아니다.
- Impact: 서·북·남쪽 문 조작, 다른 방향 투사체, 원하는 물건 Drop 등을 직접 지정할 수 없다.
- Action / Owner: UI/runtime 담당자가 행동 시작→방향/물건 선택→검증→실행→취소 경로를 연결. MorePrompt는 실제 overflow 기준과 producer를 정하거나 현재 범위에서 명시 분리.
- Re-audit: 새 session에서 8방향×방향 행동, 복수 물건 선택, 취소, 메시지 overflow를 실제 입력 경로로 검사.
- Synthesis: 같은 진입 단절에서 파생되는 East/first-item과 unused prompt findings를 병합했다.

### [MA2-F005] 마우스로 시작·주요 modal·종료를 조작할 수 없음

- Sources: A01-F005, A04-F001/F003, MAIN-UI-F003
- Pass / Pattern: Implementation / IMP-001
- Severity / Status: **Major / Needs Spec Clarification — 목표 격차 확인**
- Verified Evidence: `mod.rs:1667-1680` Title/Creation/GameOver와 overlay/soft-input/blocking 상태의 mouse를 모두 None으로 처리. `input.rs:86-117`의 제한된 command CTA와 mapper. 메인 Title/Inventory/GameOver SGR click 무반응.
- Expected / Actual: 각 기능의 양 입력 지원 목표에 비해 mouse 내부 button/row geometry가 없다. 기존 designs §10의 underlying input 차단에는 부합한다.
- Impact: 마우스로 run 시작, 열린 인벤토리 닫기/선택, Judge 제출, 새 게임을 완결할 수 없다. 검색·줍기·기도·계단·저장에도 직접 mouse CTA가 없다.
- Action / Owner: 설계/UI 담당자가 **창 뒤 입력 차단을 유지하면서 창 내부 명령**을 제공하도록 계약 확장.
- Re-audit: mouse-only 시작→플레이→물건→층 이동→끝→재시작; 키보드와 동일 candidate/turn 결과 비교.
- Synthesis: NetHack 모든 windowport의 mouse parity를 가정하지 않는다. 전 기능 mouse 평가는 사용자 목표에 근거한다.

### [MA2-F006] 적과 바닥 아이템이 지도에 표시되지 않음

- Sources: A04-F004, MAIN-UI-F001
- Pass / Pattern: Implementation / IMP-001
- Severity / Status: **Major / Needs Fix**
- Verified Evidence: `render_map.rs:16-58` tiles/@/labels만 렌더. runtime `observation.rs:114-140` visible_entities는 actor만 포함해 floor item을 누락. 초기 jackal(6,5)·potion(8,5)이 있는 actual PTY에서 glyph 부재와 HP 감소 확인.
- Expected / Actual: 플레이 판단에 필요한 관찰 대상이 화면에 나타나야 하나 적·아이템 glyph가 없다. 임시 label은 첫 문자 `[`를 오른쪽 셀에 표시하므로 대체 표시가 아니다.
- Impact: 공격 대상·위험·획득할 물건을 정상적으로 식별하기 어렵다.
- Action / Owner: observation→renderer 담당자가 floor item과 actor projection 및 그리기 순서를 연결하고 hover도 공유.
- Re-audit: 시작 적/아이템, 처치·획득·이동 뒤 각 world cell의 실제 buffer glyph와 hover 정보를 검증.
- Synthesis: core 데이터 존재와 화면 소비를 구분했고 실제 실행으로 재확인했다.

### [MA2-F007] 인벤토리 창까지 첫 네 물건만 표시함

- Sources: A04-F002, A01-F005, 메인 actual Inventory
- Pass / Pattern: Implementation / IMP-001
- Severity / Status: **Major / Needs Fix**
- Verified Evidence: `input.rs:120-148`의 `.take(4)` 모델을 `render_panels.rs:241-254`가 sidebar와 전체 inventory overlay에 함께 사용. `bootstrap.rs:76-96` 시작 물건은 5개.
- Expected / Actual: 전체 소지품을 찾아 선택할 수 있어야 하나 초기 Rock부터 목록에서 빠지고 스크롤/페이지 UI가 없다. 보이지 않는 letter를 알고 누를 가능성과 실제 발견 가능성은 다르다.
- Impact: 획득한 물건의 위치·letter를 화면에서 확인하지 못한다.
- Action / Owner: UI 담당자가 요약 sidebar와 전체 목록을 분리하고 scrolling/pagination 및 item/action 선택 제공.
- Re-audit: 초기 5개와 추가 획득 후 전체 item 접근성, 마지막 item keyboard/mouse 선택 검증.
- Synthesis: q 충돌 및 방향 선택과 독립된 표시 원인이므로 별도 finding으로 분리했다.

### [MA2-F008] 지원 최소 화면의 로그 본문과 일반 HUD 정보 부족

- Sources: A04-F005, MAIN-UI-F002
- Pass / Pattern: Implementation / IMP-001, TEST-001
- Severity / Status: **Major / Needs Fix**
- Verified Evidence: `layout.rs:58-106`의 60×24/80×24 log.height=1; `render_panels.rs:42-72` 제목이 첫 행을 차지해 본문 0행. `status_lines:75-103`은 turn/HP/level/pos만 출력. 메인 전투 중 LOG 제목만 관찰.
- Expected / Actual: 지원 화면에서 core message를 읽을 수 있어야 하나 전투·획득 본문이 잘린다. hunger도 일반 HUD에서 보이지 않는다. 저HP 텍스트는 STATUS에 있으므로 모든 위험 정보가 사라진다고 주장하지 않는다.
- Impact: 행동 결과와 자원 상태에 기반한 결정을 방해한다.
- Action / Owner: UI 담당자가 실제 메시지 본문 공간과 compact 상태 표시 확보. rectangle 검사를 실제 text/buffer 검사로 보완.
- Re-audit: 60×24, 80×24, 120×36에서 공격·획득·허기·오류 메시지의 실제 출력 확인.
- Synthesis: panel 존재와 내용 표시 가능성을 구분한다.

### [MA2-F009] TUI 저장은 프로그램 재시작을 넘지 못함

- Sources: A03-F004; A02 저장/복원 범위 분석
- Pass / Pattern: Implementation / SPEC-GAP-001
- Severity / Status: **Major / Needs Spec Clarification — 목표 격차 확인**
- Verified Evidence: `mod.rs:205-220` TempDir + quick-save.json, `:525-543` 해당 store만 사용, 새 TuiApp마다 새 저장소. runtime ArtifactStore의 durable save API 자체는 존재하며 별도 tests PASS.
- Expected / Actual: NetHack식 저장 후 재개 비교에 비해 TUI는 같은 실행 내 복원만 가능. 기존 실행별 quick-save 명세에는 부합한다.
- Impact: 프로그램을 닫고 Title의 L로 이전 플레이를 이어갈 수 없다.
- Action / Owner: 제품/runtime 담당자가 사용자별 지속 저장 위치와 저장·종료·load 정책을 정하고 기존 atomic/capability 경계 재사용.
- Re-audit: 서로 다른 두 실제 프로세스에서 저장→종료→재시작→복원 후 hash/RNG/입력 연속성 확인.
- Synthesis: 저장 직렬화나 corruption 문제로 확대하지 않고 저장 수명·사용 경로 격차로 제한한다.

### [MA2-F010] 상태에 맞지 않는 legal action이 자동 진행을 막을 수 있음

- Sources: A02-F003
- Pass / Pattern: Debug / IMP-001, DBG-002
- Severity / Status: **Major / Needs Fix**
- Verified Evidence: runtime `observation.rs:229-303`은 paralysis를 제외하지 않고 Pray/Kick도 무조건 추가. `session.rs:204-207` 마비 시 Wait/Quit 외 거부, `:495-498` cooldown Pray 거부. headless `lib.rs:267-308`은 대개 한 후보만 반환하고 `:196-223`은 실패 후 Wait fallback 없이 종료한다.
- Expected / Actual: 현재 실행 가능한 action과 submit guard가 맞아야 하나, 마비 상태에서도 Move가 legal로 보인다. 그 상태에서 이동 후보가 있으면 자동 정책은 거부된 후보 뒤 NoAcceptedAction으로 끝난다. 사람이 `.`를 누르는 경로는 가능하다.
- Impact: 지원 상태·복원 데이터·FloatingEye passive에서 진행 실패, LLM/입력 consumer의 legal-action 신뢰 저하.
- Action / Owner: runtime 담당자가 legality와 submit guard를 정렬하고 자동 정책의 실행 가능한 Wait fallback을 검증.
- Re-audit: paralysis=1, cooldown, 문 없는 위치에서 action-space와 submit 결과 및 target 1 진행을 비교.
- Synthesis: 메인 소스 추적으로 확인했다. 원본의 “같은 명령을 16회 반복”은 부정확하다. 실제 벡터가 1개면 1회 거부 후 실패하면서 attempts 필드만 16으로 보고한다. 기본 두 층에 FloatingEye가 배치됐다고 주장하지 않는다.

### [MA2-F011] 저장 턴보다 낮은 목표를 성공으로 보고함

- Sources: A02-F001
- Pass / Pattern: Debug / TEST-001, BUILD-001
- Severity / Status: **Major / Needs Fix — headless 계약**
- Verified Evidence: `BUILD_GUIDE.md:263`은 loaded turn>target에서 exit2 요구. headless `lib.rs:182-238`은 이미 target을 넘으면 loop를 생략하고 Ok; replay `:88-91,139-157`도 같은 누락. main.rs는 그 결과를 성공 출력.
- Expected / Actual: turn2 저장을 target1로 요청하면 거부해야 하나 0개 명령의 성공 결과를 만들 수 있다.
- Impact: 진행 검증·자동 실행의 완료 판정이 잘못된다. TUI의 직접 마우스 문제와는 별도다.
- Action / Owner: headless 담당자가 lower/equal/higher target 계약을 구현·CLI에서 통일.
- Re-audit: 저장/Replay의 현재 turn과 target 관계별 exit/report/원본 보존 검사.
- Synthesis: A02의 임시 CLI 결과는 보조로만 사용하고 메인이 조건문과 성공 반환 경로를 직접 재확인했다.

## 10. Critical/Major Direct Re-verification

| Finding | Main Checked | 직접 증거 | 결과 |
| --- | --- | --- | --- |
| MA2-F001 | Yes | state/action/level/stairs + official end/pray | 목표 종료 부재 |
| MA2-F002 | Yes | template/bootstrap/death + actual Creation | 선택·경험 성장 부재, 장비/HP 효과는 유지 |
| MA2-F003 | Yes | baseline→조기 return→UI Quit→loop break | q shadow 및 종료 경로 확인 |
| MA2-F004 | Yes | input East/first + session assignments + fixture tests | 정상 prompt 진입 부재 |
| MA2-F005 | Yes | dispatcher early return + 실제 세 화면 클릭 | mouse 도달 불가 |
| MA2-F006 | Yes | observation/MapWidget + actual HP 감소/glyph 부재 | 확인 |
| MA2-F007 | Yes | take(4), bootstrap5, actual Inventory | 확인 |
| MA2-F008 | Yes | layout/renderer + actual LOG | 확인 |
| MA2-F009 | Yes | TempDir lifetime·app 구성·store 경로 | 수명 제한 확인, 두 프로세스 실험 없음 |
| MA2-F010 | Yes | legality→policy→submit→NoAcceptedAction | 코드 경로 확인, 새 status E2E 미실행 |
| MA2-F011 | Yes | documented target→양 runner→CLI success | 코드 경로 확인 |

모든 채택 Major는 파일 또는 실제 실행으로 메인이 재확인했다. 코드 검토 증거와 동적 재현을 서로 바꿔 표현하지 않는다.

## 11. Cross-Report Conflicts

- **q의 실제 결과:** A04 일부 문장은 `Command(Quit)`→GameOver라 했지만 실제는 `UiCommandCandidate::Quit`→즉시 exit다. A02/A03 및 메인 소스로 바로잡았다. Playing uppercase Q는 무매핑이다.
- **Quaff 전체 불가:** q 직접 shortcut은 불가하지만 Inventory letter로 potion을 사용할 수 있다. 기능 전체 삭제가 아닌 충돌·발견 가능성 문제로 판정했다.
- **캐릭터 stats 고정:** HP는 피해/회복으로, AC는 장비로 변한다. 성장 부족은 XP/experience/skill 부재로 한정했다.
- **Quit의 Combat(0):** `spec.md`는 이를 저장 호환 sentinel로 명시한다. 새로운 회귀 결함으로 세지 않고 전체 종료 설계 비교의 기술 부채로 기록한다.
- **1000턴 테스트:** fixture와 production 기본 content가 서로 다르다고 단정하지 않는다. `world.rs`는 같은 content bootstrap을 사용한다. 다만 생존 1000턴 성공은 게임 승리나 UI 완주 증거가 아니다.
- **실제 사망→재시작 증거 부족:** A02 원본 작성 시 공백은 메인 PTY 보완으로 해당 한 시나리오에 한해 해소했다. 모든 기능의 완주는 여전히 아님.
- **마우스 hold 다중 턴:** A03-F003의 실제 backend 중복 Down 생성은 미재현이다. 여러 의도적 클릭이 여러 턴을 만드는 것만으로 버그로 확정하지 않으며 비차단 정보로 제한한다.
- **공식 source 줄:** A01의 GitHub end.c 일부 fragment 줄 번호는 메인이 확인한 raw 파일과 맞지 않는다. 그 anchor를 근거로 사용하지 않고 raw `end.c:273-288`, `pray.c:1483-1494`와 공식 Guidebook을 사용한다.
- **potion 위치:** A03의 bootstrap 관련 한 Notes 문장은 본문과 모순된다. 시작 inventory에 없을 뿐 기본 map (8,5)에 potion은 존재한다.

## 12. Finding Adjudication Ledger

| Source Finding | Decision | Canonical / 이유 |
| --- | --- | --- |
| A01-F001, A01-F003 | Merged, narrowed | MA2-F002; 선택·XP 격차, HP/AC 불변 주장 기각 |
| A01-F002 | Accepted | MA2-F001; 성공 종료 없음 |
| A01-F004 | Clarified | Quit sentinel은 기존 계약; MA2-F003 종료 정책 검토와 MA2-F001 목표 범위에 연결 |
| A01-F005 | Split/Merged | MA2-F003/004/005/007 |
| A01-F006 | Accepted as evidence limit | 새 제품 버그를 중복 산정하지 않고 §7/16의 acceptance 개선으로 연결 |
| A02-F001 | Accepted | MA2-F011; main code verification |
| A02-F002 | Merged | MA2-F004 |
| A02-F003 | Accepted with correction | MA2-F010; 16회 반복 문구 수정 |
| A02-F004, A02-F006 | Merged | MA2-F003; uppercase/lowercase와 direct exit 구분 |
| A02-F005 | Partly resolved / narrowed | 메인 사망→재시작 실행으로 해당 공백 해소, 승리·전 기능 증거는 여전히 없음 |
| A03-F001, A03-F002 | Merged | MA2-F003 |
| A03-F003 | Unresolved / Info | 물리 반복 발생 미재현, 차단 finding으로 채택하지 않음 |
| A03-F004 | Accepted as scope gap | MA2-F009 |
| A04-F001, A04-F006 | Merged | MA2-F004; mouse 부분은 MA2-F005 |
| A04-F002 | Split/Merged | MA2-F003, MA2-F007; 종료 경로 정정 |
| A04-F003 | Merged | MA2-F005 |
| A04-F004 | Accepted | MA2-F006 |
| A04-F005 | Accepted | MA2-F008 |
| A04-F007 | Accepted / Minor | Title N alias는 design에 있으나 actual Enter만 시작. 별도 대규모 시정 대상 아님 |
| A04-F008 | Accepted / Info | Up/Drag/Scroll이 Map focus로 매핑됨. 명시 정책과 대조할 후속 UX 항목 |
| MAIN-UI-F001/002/003 | Merged | MA2-F006/008/005, 실제 실행 보완 |

## 13. Required Actions Before Passing

수정 범위는 사용자의 후속 구현 요청에서 정한다. 이번 감사는 아래 순서를 권고한다.

1. **기존 기능을 정상적으로 플레이할 수 있게 복구:** 적·아이템 표시, q 종료 충돌, 방향/물건 선택, 전체 inventory, 최소 화면 메시지부터 수정한다. 기본 게임에서 즉시 체감되는 문제다.
2. **입력과 세션 흐름 완성:** 시작/모달/끝의 mouse CTA, persistent save 정책, action-space와 자동 진행 target을 정렬한다.
3. **완결된 게임 목표 결정:** 두 층 데모의 제한된 승리 조건인지 더 넓은 NetHack식 탐험·성장·목표·엔딩인지 명세화한다. 구현 없는 목표를 문서상 완료로 바꾸지 않는다.

이전 릴리스 Notice 문서 작업을 이 순서의 선행조건으로 다시 끌어오지 않는다. 이번 발견은 단순 감사 보고서 번호·날짜 동기화 문제가 아니다.

## 14. Accepted and Remaining Risks

이번 감사에서 새로운 제품 위험 수용은 부여하지 않는다. 기존 v0.3.0의 제한된 콘텐츠/허기 projection/영문 canonical UI 등은 현재 범위로 존중하되 사용자 비교 목표를 충족한 것으로 계산하지 않는다. 공급망·라이선스·배포 승인 판정은 이번 보고서 범위 밖이다.

절차상 A02의 임시 CLI 출력 생성·정리와 중복 테스트 실행이 있었다. 원본 보고서의 기록과 메인 검증을 구분했고, 미보존 로그나 불확실한 플랫폼 동작에만 의존한 Major를 채택하지 않았다.

## 15. Clarifications and Inconclusive Areas

- “준하는”의 최종 제품 규모는 후속 설계 결정 사항이다. 현재 성공 종료 부재와 조작·표시 결함만으로도 **현재 충족 여부는 아니오**라고 판단할 수 있다.
- 마우스를 각 기능의 독립 입력으로 볼지 혼합 보조 입력으로 볼지에 따라 개선 범위는 달라진다. 혼합 입력 기준에서도 MA2-F003/004/006/007/008은 해결되지 않는다.
- 물리 mouse hold 문제는 확정하지 않는다. Windows PTY/ConPTY 증거를 모든 OS·터미널·입력 장치 보장으로 확대하지 않는다.

## 16. Re-audit Checklist

차기 플레이 감사는 모든 기존 release gate를 반복하는 대신 변경된 사용자 경로를 먼저 검증한다.

- 기본 새 게임에서 표시되는 적/아이템, 첫 전투·줍기·물약을 실제 TUI로 확인.
- 8방향 행동과 복수 item에 대해 key/mouse→candidate→core outcome을 비교.
- Inventory 5개 이상 및 마지막 item을 화면에서 찾아 조작.
- 각 화면의 start/confirm/select/cancel/save/load/end/new-run을 지원 입력으로 실행.
- 60×24/80×24/120×36의 실제 text buffer에 메시지와 핵심 상태가 나타나는지 확인.
- 두 프로세스 간 저장·복원과 마비 중 진행, loaded turn보다 작은 target 거부를 검증.
- 결정된 제품 목표에 따른 성공 run과 사망 run을 별도로 끝까지 실행.

## 17. Final Decision

**NetHack 3.6.7에 준하는 시작·진행·완결: 미충족. 모든 기능의 키보드·마우스 정상 플레이: 미충족. 최종 HOLD.**

동작하는 부분은 deterministic turn, 일부 전투·아이템·계단·저장 계약, 키보드 기본 시작, 사망 화면·새 게임, terminal 복원이다. 이를 전체 게임 완성도와 혼동하지 않는다. 독립 원본 4개와 메인 보완 1개가 존재하고 봉인됐으며, 누락 원본은 없다. 구현 변경이나 외부 게시·commit은 수행하지 않았다.

## 18. Coder Handoff

```text
`C:\LocalDev\rust\AIHack\docs\multi_audit\2\final_audit_report_2.md`를 먼저 읽고, 각 finding을 프로젝트 문서와 실제 코드에 대조하여 검증한 뒤 우선순위대로 수정하세요. 계약 변경이 필요하면 관련 문서를 먼저 갱신하고, 수정 후 테스트·빌드·재감사 증거를 기록하세요.
우선 기존 플레이의 표시·입력 결함(MA2-F003/004/006/007/008)을 복구하고, 전체 NetHack 규모 확장은 명시된 제품 목표에 따라 별도 계획하세요. 기존 release 문서 이관 항목을 이번 플레이 수정의 선행 작업으로 재개하지 마세요.
```
