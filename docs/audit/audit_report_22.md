# D3D 감사 리포트 22: 콘텐츠 인과 폐쇄

작성일: 2026-08-17
상태: 2026-08-17 post-fix 판정 기록; `audit_report_23.md`가 장기 witness 증거를 HOLD했고 2026-08-18 coder remediation 후 독립 재감사 대기
근거: `spec.md`, `AI_AUDIT_DOC_STANDARD.md`, active workspace source, root integration tests

## 1. 감사 범위

- `crates/aihack-content`의 item, monster, level 정의
- `crates/aihack-core`의 world, entity, status, score, save 계약
- `crates/aihack-runtime`의 bootstrap, session, system, snapshot, observation 경로
- root `tests/`의 콘텐츠·상태 전이·장기 결정론 증거
- 생성 원인, 소비 주체, 직접 상태 변화, 후속 영향이 이어지는지 여부

## 2. 제외 범위

- `legacy_nethack_port_reference/`의 미배포 참고 구현
- presentation-only LLM narrative와 TUI 시각 효과
- NetHack 전체 콘텐츠 확장 및 save schema v2

## 3. 판정 규칙

PASS는 함수 호출이나 이벤트 발생만으로 인정하지 않는다. 명령 전후의 semantic world projection에서 turn, event count, last event처럼 자동으로 변하는 메타 필드를 제외하고도 하나 이상의 관찰 가능한 상태 delta가 있어야 한다. 후속 소비 루프는 그 delta가 뒤의 규칙 판정 또는 추가 상태 전이에 입력되는 것으로 증명한다.

### 3.1 상태 계보

- 5절의 `CAUSE-F001..004`, `TEST-F001`은 2026-08-17 구현 전 상태를 기록한 **Initial Finding**이다.
- 같은 날 구현 뒤 4절 인과 인벤토리와 7절은 개별 A/B 인과 테스트를 근거로 post-fix 상태를 기록했지만, `TEST-F001`의 3-seed 필수 witness 집계는 실제로 구현되지 않았다.
- `docs/audit/audit_report_23.md`가 이 false-green을 `TEST-F001` Re-audit #1로 HOLD했다.
- 2026-08-18 coder remediation은 `CausalProjection`, 9종 `CausalWitness`, seed별 witness multiset/final hash 3회 반복, event-only·turn-only·witness 누락 negative gate를 추가했다. 이는 독립 재감사 판정이 아니라 시정 기록이다.

## 4. 인과 인벤토리

| 콘텐츠/시스템 | 생성 원인 | 소비 주체 | 직접 상태 변화 | 후속 영향 | 현재 판정 |
| --- | --- | --- | --- | --- | --- |
| level/map/wall | embedded registry bootstrap | movement, vision, projectile | 위치·LOS·투사체 경로 제한 | 전투·탐색 가능성 변화 | 연결됨 |
| door/hidden door | level content | open/close/kick/search | tile state | 이동·LOS 변화 | 연결됨 |
| hidden pit | level content | search/movement | reveal, HP 감소 | 사망 가능 | 연결됨 |
| stairs | paired level content | ascend/descend | current level, player location | depth·행동 공간 변화 | 연결됨 |
| weapon/rock | bootstrap/level | wield/throw/combat | 장비·위치·HP | 사망·시체·처치 수 | 연결됨 |
| healing potion | level content | quaff | 소비, HP 회복 | 생존 가능성 변화 | 연결됨 |
| wand | bootstrap | zap/projectile | charge, target HP | 사망·시체·처치 수 | 연결됨 |
| scrolls | bootstrap/level | read | 소비, reveal/identify/level | 행동 공간·관찰 변화 | 연결됨 |
| armor kind/`ac_bonus` | level content | pickup/wear | 장비, content 기반 AC | 피격 확률 변화 | 연결됨 (R9) |
| food ration | bootstrap | Eat | nutrition 증가, consumed | hunger state 변화 | 연결됨 (R9) |
| jackal corpse | jackal death | pickup/Eat | nutrition 증가, consumed | hunger state 변화 | 연결됨 (R9) |
| item `base_price` | content factory | death/quit score | final score | run result 변화 | 연결됨 (R9) |
| gold | monster difficulty/kill | death score | 처치 시 증가 | final score 변화 | 연결됨 (R9) |
| monster HP/AC/hit/damage | content factory | combat | HP·명중·피해 | 사망·시체·처치 수 | 연결됨 |
| monster `ai` | content factory | intent selector | 이동/대기/추적 차이 | 전투·위치 변화 | 연결됨 (R9) |
| monster `passive` | content factory | melee response | paralysis status | 후속 command legality 변화 | 연결됨 (R9) |
| monster `speed` | content factory | turn cadence | 행동/대기 차이 | 위치·전투 빈도 변화 | 연결됨 (R9) |
| monster `difficulty` | content factory | death reward | gold 증가량 | final score 변화 | 연결됨 (R9) |
| nutrition 감소/회복 | accepted turn/Eat | hunger projection | 매 턴 -1, 섭취 시 content nutrition 증가 | hunger state 변화 | 연결됨 (R9) |
| prayer cooldown | Pray/turn tick | legality check | 20 설정 후 감소 | 재기도 차단/허용 | 연결됨 |
| paralysis | FloatingEye melee | command legality/tick | 2 설정 후 감소 | 이동·행동 차단 | 연결됨 |
| luck | Pray | player combat | +1, 최대 3 | attack roll 변화 | 연결됨 (R9) |
| hallucinating | save v1/test builder | presentation helper | production producer 없음 | core 영향 없음 | 호환성 orphan, R9 비목표 |
| kill count | monster death | death score | +1 | final score | 연결됨 |
| save/replay/hash | session state | load/replay/regression | 상태 보존 | 결정론 증거 | 연결됨 |

## 5. Initial Finding과 현재 시정 상태

### [CAUSE-F001] 콘텐츠 schema 값이 runtime behavior에 도달하지 않는다

- Initial Status: **Needs Fix**
- Current Status: **Verified by targeted tests** — `tests/causal_content.rs`의 monster speed/AI/passive, difficulty/economy, armor A/B·상태 delta 검증
- 심각도: 높음
- 증거: `speed`, `difficulty`, `passive`, `ac_bonus`가 parse되지만 actor/item 행동 데이터에 보존되지 않거나 kind 기반 상수로 대체된다.
- 영향: TOML을 바꿔도 simulation 결과가 달라지지 않아 ContentRegistry가 실질적 진실원이 아니다.
- 수정 방향: 필요한 behavior 필드를 typed runtime data에 투영하고 규칙 소비자가 그 값을 사용하게 한다. 지원하지 않을 필드는 schema에서 제거해 거짓 계약을 없앤다.
- 재감사: 동일 seed에서 한 콘텐츠 값만 바꾼 registry A/B가 예상한 semantic state delta 차이를 만들어야 한다.

### [CAUSE-F002] 음식·시체 생성이 영양 상태와 닫힌 루프를 만들지 않는다

- Initial Status: **Needs Fix**
- Current Status: **Verified by targeted tests** — `tests/causal_content.rs`의 food/corpse producer-consumer 및 hunger/lifecycle delta 검증
- 심각도: 높음
- 증거: `nutrition` item field와 `ItemClass::Food/Corpse`는 존재하지만 `CommandIntent`에 섭취 행동이 없다.
- 영향: 시체 생성과 식량 콘텐츠가 무게·인벤토리 외 다른 시스템에 영향을 주지 않는다.
- 수정 방향: Eat command를 추가하고 소비 tombstone, nutrition 증가, hunger 전이를 하나의 transaction으로 묶는다.
- 재감사: 장기 seeded run에서 영양 감소 → 음식 획득/섭취 → 영양 증가 → hunger state 변화가 모두 상태 delta로 관찰되어야 한다.

### [CAUSE-F003] 경제 상태는 production producer가 없다

- Initial Status: **Needs Fix**
- Current Status: **Verified by targeted tests** — monster difficulty kill reward와 `base_price` 기반 final score 차이를 검증
- 심각도: 중간
- 증거: `gold`와 `base_price`는 저장/조회되지만 실제 플레이에서 gold를 생성하거나 가격을 소비하는 경로가 없다.
- 영향: 가격 데이터와 score 일부가 사실상 fixture 전용이다.
- 수정 방향: inventory 가치가 score에 반영되게 하거나, 범위가 작은 loot-to-gold 경로를 정의한다. 단순 getter 호출은 증거가 아니다.
- 재감사: 서로 다른 가격 콘텐츠를 획득한 동일 seed run의 후속 score 또는 경제 상태가 달라야 한다.

### [CAUSE-F004] luck·hallucination은 저장 가능한 orphan state다

- Initial Status: **Needs Fix**
- Current Status: **Partially Verified / Time-bounded Accepted Compatibility Risk** — luck은 Pray producer와 combat consumer로 닫혔고 `hallucinating`만 SaveDataV1 호환 필드로 남음
- 심각도: 중간
- 증거: production producer가 없고 standalone pure helper만 값을 소비한다.
- 영향: save/hash surface는 커지지만 플레이 인과관계는 없다.
- 수정 방향: 명확한 producer와 downstream rule을 추가하거나 다음 schema에서 제거 대상으로 분류한다.
- 재감사: 명령/콘텐츠로 상태가 생성되고 이후 명중·표현 또는 해제 전이에 실제로 사용되어야 한다.

`hallucinating` accepted risk owner는 **Project owner / runtime maintainer**다. 재검토 시점은 **SaveDataV2 또는 v0.4.0 기능 범위 승인 시점과 2026-10-31 중 먼저 도래하는 때**이며, 그때 필드 제거 migration 또는 production producer/downstream consumer feature spec 중 하나로 닫는다. 그 전까지 새 producer가 없는 `hallucinating`을 active causal coverage나 완료 기능으로 주장하지 않는다.

### [TEST-F001] 기존 장기 테스트는 인과 커버리지를 증명하지 않는다

- Initial Status: **Needs Fix**
- Post-report-22 Re-audit: `audit_report_23.md`에서 **Needs Fix / Hold**
- Current Coder Status (2026-08-18): **Remediated, independent re-audit pending** — `crates/aihack-runtime/src/causal.rs`, `tests/long_run.rs`
- 심각도: 높음
- 증거: `tests/long_run.rs`는 accepted turn 수, Playing 상태, 제출 수 범위, 최종 hash 반복성만 검사한다.
- 영향: 모든 orphan이 남아 있어도 현재 테스트는 PASS한다.
- 수정 방향: semantic projection, causal witness count, required loop coverage를 갖는 seed 기반 장기 regression을 추가한다.
- 재감사: 이벤트만 발생하거나 turn만 증가한 가짜 구현에서는 테스트가 실패해야 한다.

## 6. 수정 순서

1. semantic state delta와 causal witness 테스트 기반
2. 음식/영양/시체 수직 슬라이스
3. armor `ac_bonus`와 monster behavior content projection
4. 가격/gold/score 연결
5. luck/hallucination의 producer-consumer 폐쇄 또는 명시적 범위 제외
6. 3 seed 장기 인과 커버리지와 반복 hash

## 7. 최종 판정

2026-08-17의 post-fix 판정은 **PASS WITH ACCEPTED COMPATIBILITY RISK**였으나, `audit_report_23.md`가 3-seed 필수 witness 집계 부재를 확인해 R9 전체를 HOLD로 대체했다. 2026-08-18 coder remediation은 각 seed에서 9종 witness를 모두 1회 집계하고 final hash `5cde4a5f145ff3af`(42), `942403c665e19ad9`(7), `01a8631d0ad95d96`(1234)를 3회 반복 검증하며 negative gate를 추가했다. 현재 권한 상태는 **코더 시정 완료 / 독립 재감사 대기**이고, `hallucinating`은 위 owner·재검토 조건을 가진 time-bounded compatibility risk다.

## 8. Coder Handoff

2026-08-17의 최초 coder handoff는 위 Initial Finding 시정으로 종결됐다. 현재 잔여 handoff는 `audit_report_23.md` 시정의 독립 재감사와 `hallucinating` risk의 지정 trigger 재검토뿐이며, 이미 Verified된 CAUSE finding을 다시 미구현 상태로 취급하지 않는다.
