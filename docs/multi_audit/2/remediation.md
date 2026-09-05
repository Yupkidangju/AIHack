# 멀티 감사 2 시정 기록

이 문서는 P0 완료 시점의 기록이다. 이후 사용자가 P3까지 구현을 요청하여 MA2-F001/002의 후속 구현을 진행했다. 현재 캠페인 기능 상태·검증은 `docs/campaign_spec.md`, `docs/campaign_implementation.md`(저장소 루트 기준)에 기록하며 아래 OPEN/계획 표시는 P0 당시 판정이다. 봉인 감사 원문 판정 자체를 수정하지 않는다.

작성일: 2026-09-05. 기준 HEAD: 8996601. 감사 원문과 봉인 원본은 보존한다.

## 범위와 순서

1. MA2-F006: visible actor와 floor item을 같은 observation으로 제공하고 지도/hover에 표시한다. 타일→아이템→살아 있는 actor→player 순서이며 라벨은 entity를 덮지 않는다.
2. MA2-F003/004/007/008: Playing q는 Quaff 선택, Q는 종료 확인이다. 방향/아이템 선택은 TUI 소유 임시 상태로 처리하여 선택·취소는 turn/RNG를 바꾸지 않고 concrete command만 submit한다. 전체 inventory는 페이지를 통해 모든 letter에 도달한다. 최소 화면에서도 메시지 본문과 hunger를 표시한다.
3. MA2-F005/009: 모달 내부 클릭만 수용하는 공유 geometry, 지속 저장 opt-in 경로와 production 기본 저장 수명을 문서화한다.
4. MA2-F010/011: 마비/cooldown/문 상태와 legal action을 정렬한다. headless는 legal Wait fallback과 실제 시도 수를 보고하고 loaded turn보다 낮은 target을 mutation 없이 거부한다.
5. MA2-F001/002: 사용자는 2026-09-05에 **더 넓은 NetHack식 탐험·역할·성장·승천을 단계별 구현**으로 확정했다. 기존 두 층/고정 캐릭터를 완결 게임으로 과대 주장하지 않으며 아래 후속 단계로 분리한다.

MorePrompt는 이번 TUI 메시지 탐색과 분리된 SaveDataV1 호환 상태로 유지하며 자동 overflow producer는 현재 범위에서 제외한다. 실제 로그는 최소 한 본문 행과 메시지 탐색으로 제공한다.

## 검증

전체 gate에서 봉인 원본 `sub_audit_01_game_contract.md`의 `../designs.md` 링크가 발견됐다. 원본 hash를 유지하기 위해 대상 `docs/multi_audit/designs.md`에 canonical `designs.md` 안내 문서를 추가한다. 링크 검사를 제외하거나 원본을 다시 봉인하지 않는다.

실제 observation/ratatui buffer, production dispatcher→handler, headless runner 실패 fixture로 RED→GREEN을 기록한다. 관련 workspace tests, fmt/clippy, debug/release build와 가능한 ConPTY 실제 실행을 검증한다. release 승인 문서 이관 및 날짜/번들/CI 전수 감사는 이번 플레이 시정 범위가 아니다.

## Finding 상태

| Finding | 문서·실행 경로 대조 및 처리 | 증거 / 현재 상태 |
| --- | --- | --- |
| MA2-F001 | content의 두 층, stairs의 최상층 경계, RunState에 성공 outcome 없음 확인 | 사용자 선택 확정; P2/P3 후속 구현, OPEN goal gap |
| MA2-F002 | 고정 Adventurer/bootstrap/Creation, death의 gold·kill_count 보상에 XP 없음 확인 | P1 후속 구현, OPEN goal gap |
| MA2-F003 | spec 입력 계약 → input baseline/dispatcher → handler 종료 경로 대조; q는 물약 선택, Q는 확인 | `playing_q_then_potion_letter_quaffs_without_exiting_and_heals`: 실제 potion 소모·HP 회복·turn 증가 PASS |
| MA2-F004 | 문서의 기존 persisted prompt 진입 주장 정정; `play_menu` → concrete submit 연결 | Open/Close/Kick 각 8방향 24-case, Throw/Zap 16-case, 복수 item·취소 revision 보존 PASS; 자동 MorePrompt는 명시 제외 |
| MA2-F005 | designs의 underlying click 차단 유지, 내부 CTA 및 전체 명령 메뉴 추가 | 실제 renderer 좌표 → dispatcher → handler: 시작/생성/뒤로/전투 사망 후 새 게임, inventory 12개 마지막 item Drop PASS; legacy persisted prompt와 자유문 편집은 키보드 범위 유지 |
| MA2-F006 | runtime visible_entities에서 floor item 누락 확인; MapWidget에 item→actor→player 표시와 label 보호 | 시작 jackal `d`/potion `!` 실제 buffer와 potion hover PASS; 기존 off-level/death/item lifecycle core 회귀 유지 |
| MA2-F007 | shared CTA 모델의 take(4) 제거, 전체 메뉴 10개씩 pagination | 초기 5개 표시, 12개 목록의 PgDown 및 표시된 Next 클릭, 마지막 item 선택·Drop PASS |
| MA2-F008 | 최소 layout.log.height=1과 title-only renderer 원인 확인; 1행은 본문, HUD food 추가 | 60×24/80×24/120×36 메시지 buffer와 food 검사 PASS; 최근 event 메뉴 M 제공 |
| MA2-F009 | production TempDir 수명 → 기존 capability store의 지속 경로로 전환, S/L/종료 정책 문서화 | 실제 Windows ConPTY 두 CLI 프로세스 저장→종료→Title L→추가 Wait→저장 해시 일치 PASS |
| MA2-F010 | observation과 paralysis/prayer/door submit guard 대조; policy fallback·attempts 실제값 수정 | SaveDataV1 복원 fixture, legal/action_space 각 command submit, stale candidate 뒤 Wait fallback 4종 PASS |
| MA2-F011 | 두 public runner의 start_turn/target 검사 추가 | loaded lower/equal/higher, replay 무변경 및 같은 최종 hash PASS |

수정된 기존 기능은 local verified 상태다. 위 표는 외부 독립 재감사의 대체물이 아니며, 넓은 게임 목표 격차가 남아 있으므로 멀티 감사 전체 PASS/게임 완성/외부 게시 승인을 선언하지 않는다.

## 재현·재검증 이력

- 최초 표시 회귀에서 지도 `.`/기대 `d`, 인벤토리 e 누락, 최소 LOG 본문 누락을 RED로 보존하고 수정 뒤 GREEN을 확인했다.
- headless 9개 회귀의 최초 결과는 7 FAIL/2 PASS였다. 마비 legality·fallback·lower target 경계를 수정한 뒤 9 PASS다.
- 별도 작업자의 actual rendered Next 클릭 회귀가 `MenuPage(false)`를 검출했다(2 PASS/1 FAIL). 행의 절반 좌표 추정을 제거하고 표시된 `[<] Prev`/`[>] Next` span으로 판정한 뒤 PASS다.
- ConPTY 비교 fixture는 `new_for_playing`이 실제 Title/Creation 이력을 생략하여 같은 turn의 hash가 달랐다. 기대 세션도 실제 Title→Creation→Playing을 submit하도록 정정한 후 두 프로세스 continuation hash가 일치했다. core hash 검증은 제거하지 않았다.
- GameOver fixture는 살아 있는 player에 non-quit GameOver를 주입해 기존 save validator가 거부했다. validator를 완화하지 않고 실제 전투 사망을 producer로 사용하는 fixture로 교체했다.
- 전체 문서 링크 gate는 감사 원본의 잘못된 상대 링크를 검출했다. 연결 문서 추가 후 11개 documentation 회귀 PASS이며 봉인된 5개 원본과 manifest SHA256은 모두 그대로다.

## 로컬 gate 기록 (2026-09-05)

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS.
- `cargo build --workspace --all-targets --release --locked`: PASS.
- `cargo audit`: PASS, RustSec 1,239 advisory / locked dependency 318개 검사.
- `cargo deny check`: cargo-deny **0.19.4**, advisories/bans/licenses/sources 모두 PASS.
- `cargo test -p aihack-tui --all-targets --locked`: 37 PASS (library 4, CLI 1, ConPTY 3, MA2 play 9, TUI contract 20).
- `cargo test -p aihack --test ma2_ui_selection --locked`: 6 PASS (24 방향 case 및 실제 물약·전투 사망·마우스 재시작 포함).
- `cargo test --workspace --all-targets --locked`: **478 PASS, 0 FAIL, 0 ignored**, 88개 test binary 결과, exit 0. Report 29 기술·Report 30 public visibility·문서 negative gate도 포함한다. 생성 로그는 `runtime/ma2-evidence/workspace-tests.log`이며 Git 추적 대상이 아니다. SHA256: `7010d92da197fe4d0ce6cbca163a035f8aeacecc18a64ff766e29114f18e4ce6`.

코드 리뷰 결과: 선택 모델은 `play_menu.rs`에 분리해 rendering/click/key가 같은 entry를 공유하고, 새 core mutable API나 dependency version 변경은 없다. 회귀 테스트의 상태 주입은 기존 validator를 통과하며 invalid fixture 때문에 validator를 약화하지 않았다. 검증 후 남은 범위는 P1~P3와 명시된 legacy modal/영구 메시지 이력 제한이다. 구현자 검증 및 별도 회귀 작성자의 재현 결과이며 외부 독립 감사 승인과 구분한다.

이번 요청의 검증은 로컬 Windows다. 새 Linux 실행, same-SHA CI, release actual bundle, 커밋·푸시는 실행하지 않았다. 이전 release-only Report 33 이관과 외부 게시 HOLD는 이 플레이 시정으로 자동 종결되지 않는다.

## 사용자 확정 확장 로드맵

현재 시정 단계 P0는 기존 플레이의 표시·입력·저장·자동진행 오류를 다룬다. 아래 P1~P3는 순서가 승인된 후속 구현 범위이며 이 변경의 완료 항목이 아니다. NGPL 출처/호환 시나리오와 결정론적 submit 경계를 유지한다.

| 단계 | 먼저 확정할 계약 | 구현 범위 | 완료 증거 |
| --- | --- | --- | --- |
| P1 역할·성장 | 역할 목록/초기 장비·능력치, XP 획득·레벨 상승 규칙, save migration과 observation schema | 실제 생성 화면 역할 선택, 전투 XP와 능력 성장, UI 표시 | 역할별 시작 차이, 같은 seed 재현, 성장 전후 전투 결과, 저장 왕복 |
| P2 탐험 | 층/분기 연결, seed 기반 생성과 도달성, 층 간 entity/RNG 수명 | 고정 두 층을 넘는 생성 던전, 왕복 계단·분기, 지속 탐험 | seed별 시작/목표 도달성, 층 재진입 상태 보존, 장기 replay 동일 hash |
| P3 목표·승천 | 목표 물건 획득·운반·최종 승천 조건, 사망/자진 종료/성공 구분, 점수 | 목표 진행 상태, 최종 성공 종료와 결과 화면 | 정상 승천 end-to-end, 목표 누락/위조/조기 승천 거부, 저장·재생 및 양 UI 결과 일치 |

역할 개수·생성 규모·승천 조건 세부 값은 각 단계 착수 시 spec/ADR과 공식 호환 record에서 닫는다. 지금 임의의 2층 미니 승리를 추가하지 않는다. MA2-F001/002는 `Accepted goal gap / planned`이며 완성 게임 또는 감사 전체 PASS로 표시하지 않는다.

## 계약 선택

- core의 persisted Awaiting*/MorePrompt는 이전 SaveDataV1 호환 상태로 유지한다. 새 TUI 선택은 임시 메뉴이며 concrete command만 core로 전달하므로 저장 schema를 바꾸지 않는다. 기존 호환 모달은 키보드 경로를 유지한다.
- mouse는 시작/생성/게임 종료, 새로운 선택/명령/종료확인 메뉴와 Judge 제출/취소 CTA를 지원한다. Judge 자유문 입력은 키보드가 필요하다. 숨겨진 지도·명령으로 modal click을 전달하지 않는다.
- visible_entities는 같은 층의 보이는 살아 있는 actor 및 바닥 item만 포함한다. renderer의 현재 kind glyph는 기존 콘텐츠 종류를 표시하며 custom glyph 편집 기능을 추가한 것으로 주장하지 않는다.
- 지원 크기보다 작은 터미널은 화면을 확장하거나 q/Q/Esc로 즉시 빠져나온다. 이 경로에서는 표시 불가능한 확인 메뉴를 열지 않는다.
- `M`은 event log의 최근 최대 8개 event를 보여주는 탐색 메뉴이며 전체 세션의 영구 메시지 이력은 아니다.
