# 게임·입력 시정 및 P1–P3 캠페인 독립 재감사 34

- 감사일: 2026-09-05, Windows, Asia/Seoul
- 기준 HEAD: `899660167d59c4b06d27a59c0d75fcccda0cce33`
- 실제 감사 대상: 위 HEAD에 더해져 있는 **미커밋 P0 시정 및 P1–P3 캠페인 작업 트리**
- 기준 보고서: `docs/multi_audit/2/final_audit_report_2.md`
- 시정/확장 기준: `docs/multi_audit/2/remediation.md`, `docs/campaign_spec.md`, `docs/campaign_implementation.md`, `tasks/plan.md`
- 감사 기준: `AI_AUDIT_DOC_STANDARD.md`, 현재 `spec.md`/`designs.md`, `code-review-and-quality`
- 감사 방식: 문서·변경 소스·테스트 대조, 전체 workspace 검사, 실제 Windows PTY와 headless CLI 직접 실행
- 변경: 이 보고서만 추가. 시작 시 존재한 826개 Git 추적/미추적 관리 대상 파일의 SHA-256을 검사 종료 시 대조해 변경·삭제 0건 확인

## 1. 최종 판정과 범위

**PASS — 현재 명세에 정의된 P0 플레이 시정 및 첫 P1–P3 캠페인 범위. 비차단 Minor 1건은 Known Issue로 기록한다.**

역할 선택→생성 던전 탐험→전투 성장→아뮬렛 획득→지상 귀환→Victory까지 구현 경로가 연결됐으며 해당 명령 완주 테스트가 통과했다. 기존의 q 충돌, 동쪽 고정 행동, 적·물건 미표시, 인벤토리 4개 제한, 최소 화면 로그 소실과 TUI 저장 수명 문제도 해당 시정 범위에서 확인됐다.

전체 workspace **491개 테스트가 통과**했고 fmt, Clippy, release all-target build도 통과했다. 현재 범위에서 추가 Critical/Major를 확인하지 못했다. 잔여 `R34-DBG-F001`은 잘못된 낮은 target을 성공으로 처리하는 원래 결함이 아니라, 실패 시 CLI 종료 코드가 문서와 다른 국소 문제다.

이 판정은 **3직업, Main 6층+Mines 2층, XP 성장, 아뮬렛 귀환 승리**라는 현재 캠페인 계약에 유효하다. NetHack 3.6.7 전체의 역할/종족/성별/정렬/상점/주문/특수 레벨/실제 승천 절차 및 balance와 동등하다는 판정은 아니다. 이전 릴리스 문서 이관, 외부 CI와 게시 승인은 이 보고서로 종결하지 않는다.

## 2. 확인한 문서·파일·검증 대상

### 문서

`spec.md`의 P1–P3 확장 및 P0 시정 절, `designs.md`, `BUILD_GUIDE.md`, `CHANGELOG.md`, `DESIGN_DECISIONS.md`, `IMPLEMENTATION_SUMMARY.md`, `docs/campaign_spec.md`, `docs/campaign_implementation.md`, `docs/multi_audit/2/remediation.md`, `tasks/plan.md`, `tasks/todo.md`를 대조했다.

P0 remediation의 OPEN 표는 문서 상단에서 P0 당시 기록으로 한정했고, 현재 확장 계약을 campaign 문서로 연결했다. 이 역사 기록만을 이유로 완료된 P1–P3를 다시 미구현으로 판정하지 않았다.

### 구현

- core: `campaign.rs`, `action.rs`, `run_state.rs`, `save.rs`, `world.rs`, item/branch 타입
- runtime: `bootstrap.rs`, `campaign.rs`, `campaign_map.rs`, `session.rs`, `save.rs`, `observation.rs`, `snapshot.rs`, death/items/movement/stairs
- TUI: `main.rs`, `tui/play_menu.rs`, `input.rs`, `mod.rs`, `render_map.rs`, `render_panels.rs`
- headless: `src/lib.rs`, `src/main.rs`
- content: quest item, registry와 schema 및 registry hash 변경
- 연결 테스트: campaign, ma2_play, ma2_ui_selection, ma2_headless_regression, ConPTY와 기존 workspace 회귀

### 제외·미검증

- 이번 미커밋 변경의 Linux 실제 실행, 원격 same-SHA CI, actual release bundle과 게시 작업
- 모든 seed×모든 role 조합의 전수 완주 및 물리 입력 장치별 검증
- 실제 외부 LLM provider의 추론 품질
- NetHack 전체 규칙 동등성 및 장기 게임 balance
- 완전한 mouse-only 자유문 편집, legacy persisted Awaiting*/MorePrompt의 mouse 조작: 현 시정 문서가 명시적으로 제외한 범위

## 3. 실행 증거

| 검사 | 결과 |
| --- | --- |
| `cargo test --workspace --all-targets --locked --no-fail-fast` | PASS, exit 0 |
| `cargo test --workspace --all-targets --locked -- --list` | 491 named tests |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo build --workspace --all-targets --release --locked` | PASS |
| `git diff --check` | PASS; 기존 줄바꿈 정책 안내는 내용 수정 없이 보존 |
| multi-audit 2 원본 manifest verify | PASS, missing source reports 없음 |
| 감사 시작/종료 파일 SHA-256 대조 | 826개 동일, 변경/삭제 없음 |

### 캠페인과 회귀 검증

- `tests/campaign.rs` 12개: 세 역할 초기 상태와 V2 저장, XP/성장 이후 실제 attack roll 차이, 생성 지도 연결성, 목표 획득/Drop/Pickup, 승리 조건, 저장·RNG·마지막 Ascend replay, 위조 데이터 거부, 생성 지도 teleport landing.
- 실제 명령 완주는 `(42, Knight)`, `(7, Scout)`, `(1234, Mage)` **세 조합**이다. 3×3=9개 조합을 검사했다고 해석하지 않는다.
- 완주 함수는 core 상태를 임의 재배치하지 않고 이동·전투·줍기·장비·물약·버리기·계단 명령을 제출한다. 경로 계산은 테스트가 전체 월드 정보를 읽는 BFS이며, 실제 사용자의 시야 기반 탐험 전략이나 전체 TUI 자동 완주까지 증명하지는 않는다.
- `ma2_play` 9개, `ma2_ui_selection` 6개: 여러 방향의 concrete 명령, 물약 실제 소비/HP 회복, 선택 취소 revision 보존, 12개 목록의 다음 페이지 및 마지막 item mouse Drop, 실제 사망 후 mouse New Run, 최소 화면 rendering.
- `ma2_headless_regression` 9개: 마비·기도 cooldown·문 legality, stale candidate 뒤 Wait fallback, lower/equal/higher target과 replay 원본 보존.
- ConPTY 4개: 기존 기본 흐름/반복 Enter 및 legacy/V2 각각 **두 실제 TUI 프로세스**의 저장→종료→Title L→추가 Wait→저장 결과를 기대 hash와 비교.
- 기존 save validation, transaction, content/causal, public visibility, LLM stale/reset, terminal restoration, archive/provenance/documentation 검사도 이번 workspace 실행에서 통과했다.

### 메인 감사자의 실제 TUI

`target/debug/aihack.exe --seed 42 --save-dir runtime`을 Windows PTY에서 실행했다. 사용자 저장 파일에는 S/L을 실행하지 않았다.

1. Title의 시작 문구를 SGR mouse click → CharacterCreation.
2. `[1] Knight`를 mouse click → Playing, HP28/28, Knight L1 X0, food900, weight87/120 및 생성 지도 확인.
3. `q` → healing potion 선택 메뉴. 프로그램은 종료되지 않았고 turn은 0 유지.
4. 표시된 Cancel 행 click → 플레이 복귀.
5. `[?] All` click → 전체 명령 메뉴, 표시된 Next click → Page 2/3.
6. Quit 항목 click → 확인 메뉴. Quit without saving click → exit 0 및 terminal 복원.

마우스 시작/역할/메뉴/취소/페이지/종료 확인이 실제 프로세스에서 연결됐다. Victory까지의 전 여정을 실제 PTY로 수동 완주한 것은 아니며, 그 경로는 명령 완주 및 결과 화면 dispatcher 테스트로 검증했다.

## 4. MA2-F001~F011 재판정

| 이전 ID | 현재 판정 | 근거와 한계 |
| --- | --- | --- |
| MA2-F001 목표·성공 종료 없음 | **Verified — 첫 캠페인 범위** | Main 6 목표→귀환→Main 1 Ascend→Victory/score, 조기/중복 승리 거부. 원본 NetHack 전체 승천과는 다른 계약 |
| MA2-F002 선택·성장 없음 | **Verified — 3직업·XP 계약** | 선택별 HP/장비 차이, 처치 XP와 level/stat 성장, 저장/관찰/전투 소비. 종족·성별·정렬은 현재 계약에 포함되지 않음 |
| MA2-F003 q 즉시 종료 | **Verified** | q는 item menu, Q는 확인 후 종료. 실제 potion 소비 테스트와 메인 PTY 확인 |
| MA2-F004 방향·물건 prompt 단절 | **Verified — 변경된 계약** | TUI-only 선택→concrete submit. 방향·물건 선택/취소 검증. 자동 MorePrompt producer는 명시 제외 |
| MA2-F005 마우스 시작·modal·종료 부재 | **Verified — 새 메뉴/화면 범위** | 시작·역할·선택·종료확인·결과/new-run CTA. legacy 호환 modal과 자유문 입력까지 전부 mouse 지원이라는 주장은 하지 않음 |
| MA2-F006 적·바닥 item 미표시 | **Verified** | observation projection과 terrain→item→actor→player 렌더, jackal/potion buffer 및 hover 검사 |
| MA2-F007 inventory 4개 제한 | **Verified** | 전체 모델과 10개 단위 menu page, 12개 목록의 마지막 item 실제 선택·Drop |
| MA2-F008 최소 화면 log/hunger | **Verified** | 1행 LOG에 본문, food/HUD, 세 크기 buffer 회귀 및 실제 80×24 확인 |
| MA2-F009 TUI 저장 수명 | **Verified — 명시 저장 경로** | production 지속 store, legacy/V2 두 프로세스 ConPTY continuation hash |
| MA2-F010 상태별 illegal action/자동 진행 | **Verified — 지적한 상태 경계** | paralysis/cooldown/door guard 반영, Wait fallback, 실제 시도 수 보고 |
| MA2-F011 낮은 target 성공 | **핵심 결함 Verified / Minor 잔여** | runner typed error 및 원본 보존. 실제 CLI의 exit2 계약은 R34-DBG-F001로 남음 |

## 5. Pass 1 — 문서·구현 정합성

현재 first-campaign 계약에 반하는 차단급 신규 finding은 확인되지 않았다.

- `spec.md` 상단은 새 campaign 문서를 현재 확장 계약으로 연결한다. 기존 2층/SaveDataV1은 compatibility 경로로 보존한다.
- 캠페인은 role/xp/amulet를 저장하며 schema2, legacy None은 schema1이다. schema2를 받아 schema1로 조용히 낮추는 방식이 아니다.
- Victory는 살아 있는 player, 실제 고유 목표 소지, Main 1의 출구 및 score를 검증한다. victory 이후 게임 명령은 제한된다.
- Save의 생성 지도 일치 검증은 현재 campaign map에 변경 가능한 문·함정이 없다는 계약과 맞는다. 향후 그런 지형 변경을 추가하면 이 검증 계약도 같이 바뀌어야 한다.
- 메뉴 모델을 `play_menu.rs`로 분리해 렌더링과 클릭이 같은 entry/page를 사용한다. 이전 실제 Next span 오류를 잡는 테스트도 유지한다.

현재 코드의 기능 범위를 NetHack 3.6.7 전체와 동등하다고 표현해서는 안 된다. 이 한계는 campaign 문서도 명시하고 있으므로 새 회귀 결함으로 재개방하지 않는다.

## 6. Pass 2 — 비차단 잔여 Finding

### [R34-DBG-F001] 낮은 target 거부 시 CLI 종료 코드가 계약과 다름

- Pass: Debug / Engineering Quality
- Pattern: BUILD-001, TEST-001
- Area: headless runner 오류 → CLI exit mapping
- Severity: **Minor**
- Status: **Needs Fix — Known Issue, 이번 gameplay PASS 비차단**
- Related: MA2-F011
- Evidence:
  - `apps/aihack-headless/src/lib.rs:95`, `:217`: `TargetBeforeCurrent`를 명령 제출 전 반환한다.
  - `apps/aihack-headless/src/main.rs:157`, `:178`: replay/일반 runner 오류를 모두 exit 1로 처리한다.
  - `BUILD_GUIDE.md:267`: loaded turn이 target보다 크면 exit 2라고 명시한다.
  - 독립 CLI 실행: turn2 저장 생성 exit0 → 해당 저장을 target1/wait-v1로 재개하면 exit1. replay-file에서도 exit1.
  - 저장 파일의 실행 전후 SHA-256은 동일했다. 성공 stdout는 출력하지 않고 오류를 출력했다.
- Expected: 이 입력 오류는 문서대로 exit2이며 진행 실패(exit1)와 구분되어야 한다.
- Actual: 실패 자체는 올바르게 처리하지만 code1을 반환한다.
- Impact: exit code로 입력 오류와 진행 실패를 구분하는 automation에서 분류가 잘못된다. 게임 진행이나 저장 무결성을 손상시키지 않고, 이전의 잘못된 성공 보고도 재발하지 않는다.
- Suggested Fix: CLI에서 `TargetBeforeCurrent`를 exit2로 매핑하고 나머지 runner 실패의 exit1을 유지한다. 실제 binary lower/equal/higher target 검증을 연결한다.
- Re-audit Method: turn2 저장에 target1을 wait/survival/replay로 요청해 exit2, 오류 출력 및 저장 보존을 확인. target2는 정상 no-op, target3은 실제 1턴 진행인지 검사한다.
- Owner: headless CLI maintainer
- Gate: 후속 소규모 CLI 정리 항목. 이 하나 때문에 전체 gameplay 시정/캠페인 감사를 반복할 필요는 없다.

재현 명령은 감사자가 만든 고유 임시 runtime에서 수행했다. 기본 사용자 runtime 파일과 분리했으며 결과를 위에 기록한 후 해당 감사 임시 폴더만 정리했다.

## 7. Pass 3 — 저장·입력 신뢰 경계

이번 변경의 저장·입력 경계에서 차단급 신규 finding은 확인되지 않았다.

- production 지속 저장도 기존 ArtifactStore의 capability, no-follow, single-link, bounded serialization, atomic replace를 사용한다.
- V1/V2 조합, XP/stat 관계, level 집합/생성 map, 고유 목표의 ID/kind/location, Victory precondition을 검증한다. 위조 데이터와 정상 왕복 테스트가 함께 통과했다.
- 새 선택 메뉴는 core world를 직접 수정하지 않고 concrete command만 submit한다. 선택/페이지/취소의 revision 불변을 검사한다.
- 새 menu가 없는 기존 blocking 상태에서 underlying click 차단을 유지한다. 새 메뉴의 내부 row/CTA만 소비한다.
- 종료 확인과 지속 저장이 이전 q 즉시 종료/임시 저장 손실 문제를 분리해 해결했다.

이번에 dependency advisory DB를 새로 갱신·전수 감사한 것은 아니다. 위 판정은 변경 코드와 이번 전체 회귀에 포함된 보안 계약에 한정한다.

## 8. Cross-Pass Conflicts와 증거 해석

| 겉보기 충돌 | 판정 |
| --- | --- |
| 491 tests PASS vs CLI exit 차이 | runner unit 경계는 맞지만 CLI error mapping assertion이 부족하다. Minor 유지 |
| 이전 full NetHack 목표 gap vs 새 P3 완료 | 새 문서로 정의된 첫 campaign은 확인됐다. 전체 원본 동등성으로 승격하지 않는다 |
| P0 remediation의 OPEN 문구 vs P3 완료 | 문서 상단에 historical P0 기록임을 명시하고 campaign 문서로 연결하므로 새로운 blocker로 세지 않는다 |
| 상태 prompt를 core에서 만들지 않음 | 현재 계약은 TUI-only 선택으로 변경됐다. concrete command만 submit하는 구현과 일치한다 |
| 모든 mouse 기능이라는 이전 목표 vs legacy modal/free text 제외 | 새 화면·명령 메뉴 범위만 Verified. 예외를 포함한 무조건적인 전 기능 mouse-only PASS는 하지 않는다 |
| working tree tests PASS vs clean release | 미커밋 로컬 검증이다. 원격 CI/actual bundle/게시 PASS를 뜻하지 않는다 |

## 9. 남은 한계·재검토 조건

- Known Issue: R34-DBG-F001의 종료 코드 매핑.
- 정상 완주 검증은 3개 seed/role 조합과 Knight42 반복이다. 모든 seed에서 균형·생존 가능성을 보장하지 않는다.
- 실제 TUI에서는 mouse 시작·역할·메뉴·페이지·종료 확인을 확인했고, 장기 승리 경로는 core 명령 tour와 renderer/dispatcher 테스트로 확인했다.
- legacy persisted Awaiting*/MorePrompt는 키보드 호환 경로이며 자동 message overflow producer도 현재 제외된다. Judge 자유문은 키보드가 필요하다.
- campaign은 자체 balance/생성 방식/아뮬렛 귀환 승리를 쓰는 첫 구현이며 NetHack 원본의 전체 시스템 재현은 아니다.
- 이전 release-only 이관과 외부 게시 승인은 별도 상태다.

## 10. 인계와 종료 조건

**플레이 시정과 첫 캠페인의 차단급 재감사 항목은 이번 범위에서 해소됐다.** 일반 개발과 플레이 검증을 진행할 수 있다. 남은 Minor는 실제 headless CLI 계약 테스트와 함께 한 번 정리하면 된다. 같은 전체 감사·문서 번호 갱신 루프를 다시 시작할 필요는 없다.

```text
C:\LocalDev\rust\AIHack\docs\audit\audit_report_34.md를 확인하고,
R34-DBG-F001을 BUILD_GUIDE 및 실제 CLI 오류 처리에 대조해 검토하세요.
TargetBeforeCurrent의 exit2 매핑과 실제 binary 회귀만 보완하세요.
이번 PASS는 문서화된 P0/P1–P3 첫 캠페인 범위로 유지하고,
NetHack 전체 동등성이나 외부 게시 승인으로 확대하지 마세요.
```
