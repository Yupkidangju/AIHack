# A04 보완 증거 — 메인 감사자의 실제 Windows TUI 실행

## 1. Audit Metadata

- Audit Turn: 2
- Perspective: 메인 모델의 실제 런타임 직접 재검증
- User Goal: NetHack 3.6.7에 준하는 시작·진행·끝과 키보드·마우스 전 기능 플레이 여부
- Audit Basis: Standard-backed
- Standard Path: `C:/LocalDev/rust/AIHack/AI_AUDIT_DOC_STANDARD.md`
- HEAD: `899660167d59c4b06d27a59c0d75fcccda0cce33`
- 일자: 2026-09-05
- Supplement Of: `sub_audit_04_input_ui.md` (독립 원본과 별도의 메인 직접 증거)

## 2. Assigned Scope

실제 debug TUI 바이너리를 Windows PTY에서 실행하고 사용자 입력에 따른 출력과 관련 코드의 소비 경로를 대조했다. 테스트 또는 소스 파일을 만들거나 변경하지 않았다.

## 3. Excluded and Uninspected Scope

전체 게임 기능의 수동 완주, 물리 마우스 장치와 키보드 hold, 외부 LLM provider, Linux 런타임은 이번 직접 실행으로 확인하지 않았다. 실행한 SGR 마우스 입력은 pseudoconsole 경로이며 물리 장치 검증이라고 주장하지 않는다. 기능 도달 불가능은 정적 dispatcher 검토와 함께 판정한다.

## 4. Evidence Examined

1. `cargo test -p aihack-tui --all-targets --locked`: library 4, main 1, ConPTY 2, contract 20 = 총 27 PASS. ConPTY가 실제 바이너리 Enter→Creation→Playing, 마우스 이동/공격 명령, Inventory/Esc, quit와 ANSI 복원을 확인한다.
2. `.\\target\\debug\\aihack.exe --seed 42`를 PTY에서 실행했다. 출력 좌표 기준 80×24 화면이었다. 프로세스 session 72625.
3. Title에 `Press Enter to Start`, `L - Load Game`, `Q - Quit`가 출력됐다.
4. Enter 1회 → `Character Creation`; Class Adventurer, HP 16/16, Strength 10, Dexterity 10, AC 0 고정 표시.
5. Enter 1회 → `COMMANDS`, turn 0, hp 16/16, Main:1, pos 5,5. 맵은 타일과 @를 표시했다. 기본 콘텐츠에서 (6,5)의 jackal, (8,5)의 potion이 있어도 해당 glyph가 보이지 않았다.
6. SGR `ESC[<0;28;11M` 및 release를 전달했다. turn 1, hp 14/16으로 바뀌고 일시적인 `[[` 라벨만 나타났다. 맵 renderer는 `visible_entities`를 순회하지 않는다.
7. `i` → INVENTORY. a dagger, b food ration, c wand, d reveal scroll 4개만 표시됐다. 실제 초기 inventory는 bootstrap상 Rock을 포함해 5개다.
8. inventory item 위치로 SGR 클릭했으나 상태/출력 변화가 없었다. `runtime_event_to_candidate`는 inventory overlay에서 모든 mouse event를 None으로 반환한다.
9. Esc → Playing 복귀. `.` 20개를 전달하자 turn이 진행되어 19턴에 GAME OVER가 표시됐다. cause `Killed by entity 2`, Depth 1, Defeated 0, Score 384, Seed 42.
10. GameOver의 New Run 문구 위치로 SGR 클릭했으나 변화가 없었다. `N`은 Title로 이동했다.
11. Title의 시작 문구 위치로 SGR 클릭했으나 변화가 없었다. `q`는 exit code 0으로 종료했다. alternate-screen leave 및 cursor-show 출력이 관찰됐다.
12. 게임 진행 내내 80×24의 LOG는 제목만 표시했다. `compute_layout(80,24)`의 log.height는 1이며 `render_text_panel`은 그 한 줄을 title로 소비한다.

관련 코드: `apps/aihack-tui/src/tui/mod.rs`의 dispatcher, state render, handler; `input.rs`의 key/mouse/CTA; `render_map.rs`; `render_panels.rs`; `layout.rs`; `crates/aihack-runtime/src/observation.rs`, `bootstrap.rs`.

## 5. Findings

### [MAIN-UI-F001] 적·바닥 아이템 표시 경로가 비어 있음

- Pass: Implementation
- Area: 맵 정보와 실제 전투
- Severity: Major
- Status: Needs Fix
- Evidence: `MapWidget::render`는 tiles, player, 임시 label만 렌더한다. observation의 visible entity producer도 actor만 반환한다. 위 실행에서 HP 감소와 공격 대상 glyph 부재를 확인했다.
- Expected Basis: 사용자 요청의 정상 플레이, designs의 map/entities observation 연결.
- Actual: 인접 적의 위치를 기본 glyph로 확인할 수 없고 바닥 아이템은 observation/hover에도 포함되지 않는다. 임시 label은 첫 글자 `[`를 다른 셀에 그린다.
- Impact: 전투 대상과 수집물을 눈으로 파악할 수 없다.
- Suggested Action: actor/item projection 및 renderer를 연결하고 기본 fixture의 적·아이템을 실제 buffer에서 검증한다.
- Re-audit Method: 시작 위치의 jackal, potion과 획득·사망 이후 glyph 변화를 검사한다.
- Confidence: High

### [MAIN-UI-F002] 지원 최소 높이에서 LOG 본문을 볼 수 없음

- Pass: Implementation
- Area: 결과/위험 피드백
- Severity: Major
- Status: Needs Fix
- Evidence: `layout.rs::standard_layout`/`degraded_layout`는 높이 24에서 log.height=1. `render_panels.rs::render_text_panel`은 제목 다음 행부터 본문을 그린다. 직접 실행에서도 공격 중 LOG 제목만 보였다.
- Expected Basis: designs의 core message 우선순위와 60×24 지원.
- Actual: 영역의 존재와 겹침 여부 검사는 통과하지만 내용 표시 공간이 0행이다.
- Impact: 적 공격·명령 결과를 읽기 어렵다.
- Suggested Action: 최소 1행 이상의 log 본문을 확보하거나 별도의 메시지 줄을 배치한다.
- Re-audit Method: 60×24, 80×24, 120×36 실제 render buffer에 공격/거부 텍스트가 남는지 확인한다.
- Confidence: High

### [MAIN-UI-F003] 마우스로 시작·모달 조작·재시작 불가

- Pass: Implementation
- Area: 입력 도달성
- Severity: Major (사용자 목표 대비)
- Status: Needs Spec Clarification
- Evidence: Title/GameOver/Inventory 직접 클릭 무반응 및 `mod.rs:1667-1680` 모든 관련 mouse event 조기 반환.
- Expected Basis: 사용자의 모든 기능 키보드·마우스 플레이 목표. 기존 designs는 modal click 무시를 의도적으로 명시한다.
- Actual: 기존 안전 정책에는 일치하지만 사용자 목표의 마우스 완주에는 미달한다.
- Impact: 마우스만으로 시작 또는 종료 후 재시작할 수 없다.
- Suggested Action: 기존 정책을 modal 내부 CTA 처리와 underlying 차단으로 확장하도록 명세부터 정한다.
- Re-audit Method: 시작→플레이→인벤토리→종료→재시작을 mouse-only event로 실행한다.
- Confidence: High

## 6. Uncertainties and Clarifications Needed

위 순서는 사망 엔딩의 화면 전환 증거이며 승리·승천·탈출을 증명하지 않는다. 전체 feature mouse parity와 key/item/direction 선택은 A04 원본 및 통합의 코드 도달성 대조를 사용한다.

## 7. Perspective Decision

HOLD. 기초 키보드 루프와 process 복원은 작동하지만 정상 전투 정보와 mouse-only 완주가 충족되지 않는다.
