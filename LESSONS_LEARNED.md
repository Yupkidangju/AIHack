# AIHack Lessons Learned

## 2026-08-18: 감사 false-green과 파일 handle 경계

- 경로를 canonicalize한 뒤 bare path를 다시 여는 방식은 보안 경계가 아니다. root directory capability와 실제 open/rename을 결합하고, 열린 handle의 file type과 hard-link count를 확인해야 한다.
- 임시 파일 이름을 예측하기 어렵게 만드는 것만으로는 충분하지 않다. 원자적 신규 생성, 같은 directory의 sync·replace, 실패 시 기존 파일 보존을 하나의 API가 책임져야 한다.
- 장기 테스트의 “상태가 달라졌다” assertion은 여러 인과 계약을 한꺼번에 증명하지 못한다. 필수 witness를 닫힌 enum과 count map으로 정의하고 누락·event-only·turn-only negative를 acceptance validator에 넣어야 한다.
- 감사 보고서는 Initial Finding, coder remediation, 독립 재감사 판정을 같은 현재형으로 섞지 않는다. 후속 보고서의 종결 권한과 새 HOLD를 시간 순서대로 분리해야 자동 문서 회귀가 과거 pending을 현재 상태로 되살리지 않는다.
- line-ending fixture를 무조건 정규화하면 실제 Windows checkout 실패를 숨길 수 있다. canonical checkout 속성과 CRLF 입력 회귀를 모두 검사하고 실제 platform 명령을 CI에서 실행해야 한다.
- SPDX exception은 기본 license와 별개이며 일반 allowlist보다 crate/version 한정 exception이 권한 확대를 줄인다. owner, 만료일과 dependency 변경 trigger를 함께 기록하고 고정 cargo-deny binary로 실제 실행해야 한다.
- Unix mode 0600을 Windows owner-only ACL과 동일하게 표현하면 안 된다. Windows가 parent DACL을 상속한다면 문서·함수 이름·테스트가 그 실제 경계를 그대로 말해야 한다.
- Linux `O_TMPFILE`은 이름이 붙기 전 link count가 0이다. 기존 destination의 single-link 불변조건을 신규 anonymous temp에 그대로 재사용하면 보안 강화가 정상 저장을 차단하므로 lifecycle별 validator를 분리해야 한다.
- negative fixture는 실패 payload가 실제 산출물에 들어갔는지 먼저 증명해야 한다. Git ignore나 export attribute에 따라 주입이 사라지면 verifier가 아니라 fixture가 false-green/false-red를 만든다.

## 2026-08-17: 콘텐츠 인과 폐쇄

- 1000턴 도달과 반복 hash만으로는 콘텐츠가 simulation에 참여한다는 사실을 증명할 수 없다. turn/event metadata를 제외한 semantic snapshot delta와 콘텐츠별 producer-consumer witness가 함께 필요하다.
- schema에 필드가 있고 factory가 읽는 것만으로는 단일 진실원이 아니다. kind 기반 fallback이 실제 행동을 결정하면 content 값은 orphan이므로 동일 seed A/B registry로 행동 차이를 검증해야 한다.
- save 호환 필드를 억지로 기능화하면 잘못된 게임 규칙이 된다. production producer가 없는 `hallucinating`은 기능처럼 PASS시키지 않고 호환성 orphan과 후속 제거/별도 spec 대상으로 명시하는 편이 안전하다.
- 점수 helper가 존재해도 실제 GameOver 경로가 호출하지 않으면 시스템은 연결되지 않은 것이다. 종료·사망 양쪽에서 observable final score까지 추적해야 한다.

문서 상태: active
작성일: 2026-05-20
최근 갱신: 2026-08-18
버전: v0.3.0

---

## 1. 프로젝트 개요

AIHack은 NetHack 3.6.7의 결정론적 Rust 재구현을 목표로 한 프로젝트다.
기존 C-to-Rust 직접 포트(`legacy_nethack_port_reference/`)에서 새로운 `GameSession` 중심 아키텍처로 전환하면서
다음 교훈을 얻었다.

원본 source를 AI에 제공해 의도와 규칙을 추론한 semantic rewrite는 표현이 크게 달라도 생성 과정을 숨기지 않고 파생물로 분류하는 편이 provenance와 배포 의무를 가장 정직하게 보존한다. 이 프로젝트는 whole-work NGPL, 공식 LICENSE checksum, 파생·변경 NOTICE와 complete corresponding source gate로 그 결정을 기계 검증한다.

---

## 2. 아키텍처 결정 교훈

### 2.1 단일 상태 원천(Single Source of Truth)의 효과

**상황:**
기존 포트는 `self.game.grid`와 `resources.Grid`가 분리되어 있어 렌더링과 시뮬레이션 상태가 이중화되었다.

**결정:**
새 엔진은 `GameSession` 하나가 `GameWorld`(map + entity store + inventory)를 소유하도록 했다.

**결과:**
- AI observation, replay 검증, snapshot hash 모두 동일한 상태를 읽는다.
- `clone()` 기반 상태 복제 없이 deterministic headless 실행이 가능하다.

**회귀 방지:**
새로운 상태 저장소를 추가할 때 반드시 `GameSession` 내부로 통합할 것. 외부 resource/Grid 이중화는 금지한다.

---

### 2.2 ECS(Legion) 제외는 올바른 초기 결정이었다

**상황:**
Legion ECS를 사용한 이전 포트에서 런타임 borrow conflict가 반복됐다.

**결정:**
v0.1에서는 `EntityId`와 typed component-like struct를 사용하는 explicit entity store를 도입했다.

**결과:**
- 결정론적 디버깅이 단순해졌다.
- `GameSession`이 모든 상태를 소유하므로 snapshot/restore가 직관적이다.

**회귀 방지:**
성능 이슈가 실제로 측정되기 전에는 ECS를 재도입하지 않는다. 필요 시 v0.3 이후 performance audit에서만 검토한다.

---

## 3. 결정론적 실행 교훈

### 3.1 RNG 격리는 핵심 보안 게이트다

**상황:**
NetHack의 복잡한 무작위 요소(전투, 아이템 생성, 몬스터 AI)를 결정론적으로 재현해야 했다.

**결정:**
`thread_rng`/`random()` 호출을 전면 금지하고, `src/core/rng.rs`의 `GameRng`(StdRng 기반)만 사용한다.

**결과:**
- `cargo run --bin aihack-headless -- --seed 42 --turns 1000`이 반복 실행 시 동일한 `final_hash`를 출력한다.
- `tests/release_candidate.rs`가 multi-seed baseline을 자동 검증한다.

**회귀 방지:**
`rg "rand::|thread_rng|random" src`를 clippy/test 이후에도 항상 실행한다. `GameRng` 외의 RNG 소스 도입은 PR 금지다.

---

### 3.2 문서 baseline과 테스트 fixture의 동기화

**상황:**
Phase 14에서 `IMPLEMENTATION_SUMMARY.md`의 release baseline hash가 `tests/release_candidate.rs`의 fixture보다 늦게 갱신됐다.

**결과:**
감사자가 문서 기준으로 실행 결과를 regression으로 오판할 수 있었다.

**회귀 방지:**
- release candidate 테스트의 fixture를 변경하면 즉시 `IMPLEMENTATION_SUMMARY.md`의 Phase 14 검증 결과를 동기화한다.
- `BUILD_GUIDE.md`의 출력 예시도 동일한 기준으로 유지한다.
- baseline 변경 시 "historical baseline" 분리 구간을 명확히 표시한다.

---

## 4. 보안/의존성 교훈

### 4.1 supply chain scanner는 quality gate의 일부다

**상황:**
`ratatui 0.29.0`이 transitive dependency로 `paste`(unmaintained)와 `lru`(unsound)를 끌어들였다.

**처음 대응:**
`cargo audit` 결과를 무시하고 넘어갈 뻔했다.

**실제 대응:**
`ratatui 0.30.0`으로 업그레이드하여 두 advisory를 제거했다.

**결과:**
`cargo audit`이 clean 상태가 되어 보안 게이트를 통과했다.

**회귀 방지:**
- `cargo audit`을 CI/로컬 품질 게이트에 포함한다.
- advisory가 발생하면 즉시 upgrade 또는 accepted risk 문서화를 검토한다.
- `Cargo.toml`의 dependency 버전을 minor/patch 자동 업데이트보다는 명시적으로 관리한다.

---

## 5. 문서 운영 교훈

### 5.1 D3D Protocol의 문서-코드 동기화 요구는 실제로 유효하다

**상황:**
AGENTS.md의 Required Files(`IMPLEMENTATION_SUMMARY.md`, `LESSONS_LEARNED.md`)와 실제 루트 파일명(`implementation_summary.md`)이 충돌했다.

**결과:**
감사에서 Minor finding으로 지적됐고, 2차 재감사까지 남았다.

**회귀 방지:**
- 프로젝트 초기에 파일명 규칙(대문자 vs 소문자)을 명확히 결정하고 문서화한다.
- 파일 이동/재명명 시 전체 프로젝트의 참조 텍스트를 `rg`로 검색해 일괄 갱신한다.
- Markdown 문서 간 교차 참조는 파일명을 정확히 일치시킨다.

---

### 5.2 존재하지 않는 파일/함수를 문서 authority로 지목하면 혼란이 반복된다

**상황:**
`IMPLEMENTATION_SUMMARY.md`가 `systems/turn_pipeline.rs`와 `apply_turn()`을 실제로 존재하지 않는 authority로 언급했다.

**결과:**
구현자가 잘못된 파일을 기준으로 수정하거나 감사자가 정합성을 오판할 수 있었다.

**회귀 방지:**
- 문서에 파일/함수를 언급할 때 반드시 `rg -n "fn 이름" src/`로 존재를 확인한다.
- 파일 책임표는 실제 `git ls-files src/` 결과와 주기적으로 대조한다.

---

## 6. 테스트 전략 교훈

### 6.1 headless deterministic 실행은 regression 방지의 최선책이다

**상황:**
전투, 아이템, 몬스터 AI, stairs, vision 등 개별 단위 테스트는 풍부했지만,
전체 게임 흐름의 통합 검증 수단이 필요했다.

**결정:**
`aihack-headless` 바이너리와 `tests/release_candidate.rs`를 도입했다.

**결과:**
- seed 기반 전체 게임 1000턴 실행의 hash가 테스트와 문서에서 고정된다.
- TUI/UI 변경이 core 로직에 영향을 주는지 즉시 감지할 수 있다.

**회귀 방지:**
- core 로직 변경 시 반드시 `cargo test --test release_candidate`를 실행한다.
- TUI-only 변경이라도 headless smoke test를 함께 실행한다.

---

## 7. UI/TUI 교훈

### 7.1 TUI와 core의 경계를 명확히 분리해야 한다

**상황:**
`ratatui` 버전 업그레이드(0.29→0.30)가 UI 크레이트 전반에 영향을 미쳤다.

**결과:**
`GameSession`과 headless 실행은 TUI 의존성 없이 동작했으므로, core 로직 테스트가 영향을 받지 않았다.

**회귀 방지:**
- core(`src/core/`, `src/domain/`, `src/systems/`)는 TUI crate에 의존하지 않는다.
- UI-only 기능(reduced-motion, high-contrast, hover inspect 등)은 `src/ui/`에서만 구현한다.
- UI 변경이 snapshot hash에 영향을 주지 않음을 `tests/ui_effect_projection.rs`가 검증한다.

---

### 7.2 실제 PTY 검증은 후보 단위 테스트가 놓치는 event/render 경계를 찾는다

**상황:**
R6의 input candidate와 LLM state 테스트는 통과했지만 실제 PTY에서는 Enter가 상태 매퍼에 도달하지 않았고, footer의 `.` Wait 안내와 실제 키 매핑이 달랐다. failure fallback은 Retry를 가렸으며 modal 빈 행에는 기존 panel 내용이 남았다.

**결과:**
60x24/59x23 크기 계약, `KeyCode` 변환, footer 상태 우선순위와 overlay clear를 별도 회귀 테스트로 고정했다.

**회귀 방지:**
- TUI milestone은 순수 candidate 테스트뿐 아니라 실제 `KeyCode`→state mapping을 검증한다.
- 화면에 표시한 키와 CTA는 실제 mapping 목록과 같은 테스트에서 비교한다.
- overlay widget은 빈 문자열도 기존 buffer를 지우는지 검증한다.
- 최소 지원 크기와 바로 아래 크기를 모두 실제 PTY로 실행한다.

### 7.3 감사 증거는 결과표가 아니라 재실행 자산이어야 한다

**상황:**
일회성 loopback fixture의 PASS 기록만으로는 다음 감사자가 timing·wire·terminal 복원을 재현할 수 없었다.

**결과:**
deterministic fixture source, exact command와 semantic assertion을 저장소에 보존하고 success/timeout/stale/down 및 pending-exit를 자동 재실행하도록 전환했다.

**회귀 방지:**
- raw screen snapshot보다 `LLM: WAIT/TIMEOUT/STALE/DOWN`, CTA, restore order처럼 안정된 의미를 검사한다.
- fixture는 loopback과 표준 라이브러리에 한정하고 외부 provider나 secret을 요구하지 않는다.
- coder의 local 시정 PASS와 독립 감사의 Verified/PASS authority를 문서 상태에서 분리한다.

### 7.4 Provenance 구현 완료와 배포 승인은 다른 상태다

**상황:**
공식 archive checksum과 source locator를 검증해도 파생물 배포 권리가 자동 승인되지는 않는다.

**결과:**
내부 build/test inclusion은 engineering review로 fail-closed하게 관리하고, project license와 notice 의무는 소유자 또는 적격 검토자의 승인 항목으로 분리했다.

**회귀 방지:**
- R7 record/test GREEN과 `Reviewed -> Approved` 전환을 별도 gate로 유지한다.
- 자동화 결과를 법률 판단이나 외부 배포 허가로 표현하지 않는다.
- `UNLICENSED` 변경은 승인 근거와 R8 release 문서가 같은 작업 단위에서 동기화될 때만 수행한다.

### 7.5 승인 상태는 evidence 검증 없이 권한이 아니다

**상황:**
`Reviewed`를 `Approved`로 바꾼 개수만 세면 reviewer, 적용 license, scope, notice, checksum이 없는 상태도 통과할 수 있었다.

**결과:**
정상 승인 fixture와 status-only·필드 누락·checksum 변조·ID 중복·coverage 누락/중복·Blocked include negative fixture를 같은 checkpoint에 연결했다.

**회귀 방지:**
- 승인 gate는 상태와 evidence를 하나의 원자적 계약으로 검증한다.
- 문서 expected field는 해당 scenario test가 직접 assert하거나 명시된 보조 test에 연결한다.
- 인간 승인 부재와 자동화 결함을 별도 finding으로 유지한다.

### 7.6 단계 gate는 후속 단계의 mutation을 선행조건으로 요구하지 않는다

**상황:**
R7이 root license 변경을 요구하면서 그 변경을 R8에서만 허용해 승인 이후에도 순환 의존이 생겼다.

**결과:**
R7은 asset/scenario provenance, R8은 root distribution license/version/packaging을 소유하도록 분리했다. release script의 검사 root도 caller environment가 아닌 script 위치로 고정했다.

**회귀 방지:**
- 각 checkpoint는 자신이 소유한 변경만으로 PASS 가능해야 한다.
- 이전 단계 PASS와 외부 배포 허가는 별도 상태로 표현한다.
- release hard gate의 repository identity는 inherited 환경변수로 교체하지 않는다.

---

## 8. 마이그레이션 교훈

### 8.1 레거시는 삭제하지 말고 격리하라

**상황:**
기존 NetHack Rust 포트가 `game_loop.rs`, Legion ECS, Grid 이중화 등 구조적 부채를 안고 있었다.

**결정:**
기존 코드를 `legacy_nethack_port_reference/`로 이동하고 새 엔진을 루트에서 설계했다.

**결과:**
- 규칙 지식, 테스트 데이터, 몬스터/아이템 데이터가 보존됐다.
- `rg "legacy_nethack_port_reference" src Cargo.toml tests`가 clean하다.

**회귀 방지:**
- `src/`에 직접 `legacy_nethack_port_reference/` 파일을 `use`하지 않는다.
- 레거시 참조가 필요하면 명시적인 ADR을 작성하고 `DESIGN_DECISIONS.md`에 기록한다.

---

## 9. 감사 프로세스 자체에서 배운 점

### 9.1 감사-수정-재감사 루프는 효과적이다

**상황:**
`audit_report_1.md`는 R3 bootstrap 계약을 HOLD로 기록했고, 같은 파일의 Re-audit #2는 R3 LOCAL PASS로 갱신했다. `audit_report_2.md`는 R2 public-contract drift와 R3 secondary-document drift를 HOLD로 기록했다. 코더 remediation 설명은 `audit_report_2.md` 13절에 claim으로 보존하고, 독립 검증 판정은 순차 보고서인 `audit_report_3.md`가 담당한다.

**배운 점:**
- 문서와 코드의 양방향 정합성은 기능 구현만큼 중요하다.
- `cargo fmt`, `clippy`, `test`, `audit` 4개 게이트를 동시에 통과하는 것이 기준이다.
- Minor finding도 쌓이면 재감사를 요구한다.

**회귀 방지:**
- release 전에 `AI_AUDIT_DOC_STANDARD.md`의 Phase gate checklist를 자체 실행한다.
- 문서 변경은 코드 변경과 동일한 PR 리뷰 기준을 적용한다.

---

## 10. 향후 프로젝트에 적용할 체크리스트

새로운 Rust 게임/시뮬레이션 프로젝트를 시작할 때 다음을 반드시 먼저 결정한다:

1. [ ] 파일명 규칙(대문자 vs 소문자) 및 Required Files 목록을 `AGENTS.md`와 루트 파일명으로 동기화한다.
2. [ ] 결정론적 실행이 목표라면 `thread_rng`/`random()` 호출을 금지하고 단일 RNG 소스를 도입한다.
3. [ ] 단일 상태 원천을 설계하고 ECS 도입은 실제 성능 측정 이후로 미룬다.
4. [ ] 문서에 언급하는 파일/함수는 실제 존재 여부를 `rg`/`git ls-files`로 검증한다.
5. [ ] `cargo audit`을 품질 게이트에 포함하고 advisory 발생 시 즉시 대응 계획을 세운다.
6. [ ] headless deterministic runner와 multi-seed baseline 테스트를 초기에 도입한다.
7. [ ] 레거시 코드는 삭제하지 않고 reference tree로 격리한다.
8. [ ] release baseline hash가 변경되면 문서, 테스트, 빌드 가이드를 동시에 갱신한다.

---

(End of document)
