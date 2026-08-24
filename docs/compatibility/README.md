# NetHack 3.6.7 Compatibility Records

문서 상태: NH367-C001..C010 engineering/provenance closed; report 27 remediation `ea7822a5/32683076204` same-SHA verified; independent re-audit pending
작성일: 2026-07-15 (2026-07-18 R7 구현 갱신)
관련 Task: R7-2
기준: `../../spec.md` 13절, `../../PROVENANCE.md`

## 1. 목적

각 record는 NetHack 3.6.7의 관찰 가능한 규칙 하나를 source, 입력, 기대 결과, Rust test에 연결한다. record는 원본 코드나 문장을 복사하는 장소가 아니다.

## 2. 구현 scenario

| ID | 범위 | record | engineering test | provenance |
| --- | --- | --- | --- | --- |
| NH367-C001 | 벽 이동 | `NH367-C001-wall-movement.md` | PASS | Approved |
| NH367-C002 | 닫힌 문 | `NH367-C002-closed-door.md` | PASS | Approved |
| NH367-C003 | bump attack | `NH367-C003-bump-attack.md` | PASS | Approved |
| NH367-C004 | pickup/wield/quaff | `NH367-C004-item-actions.md` | PASS | Approved |
| NH367-C005 | stairs 왕복 | `NH367-C005-stairs.md` | PASS | Approved |
| NH367-C006 | search hidden door/trap | `NH367-C006-search.md` | PASS | Approved |
| NH367-C007 | throw/zap/read | `NH367-C007-projectiles.md` | PASS | Approved |
| NH367-C008 | hunger/status | `NH367-C008-hunger-status.md` | PASS | Approved |
| NH367-C009 | save/load continuation | `NH367-C009-save-continuation.md` | PASS | Approved |
| NH367-C010 | death/game over | `NH367-C010-game-over.md` | PASS | Approved |

10개 record와 integration test는 구현돼 R7 engineering 범위에서 통과했다. 2026-07-20 프로젝트 소유자는 원본 NetHack 3.6.7 source를 사용한 AI-assisted semantic rewrite라는 실제 생성 과정을 근거로 10개 record와 AIHack 전체의 NGPL 파생물 배포를 승인했다. 각 record의 `Approved` authority/evidence는 `../../PROVENANCE.md`와 함께 SC-LICENSE-01에서 machine validation한다. 독립 R8 기술 감사 전에는 외부 게시하지 않는다.

## 3. record schema

각 파일 이름은 `NH367-CNNN-short-name.md`다.

```yaml
id: NH367-C001
title: ""
status: Planned | Implemented | Verified | Blocked
source:
  release: NetHack 3.6.7
  archive_sha256: 98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2
  url: https://www.nethack.org/v367/download-src.html
  locator: "archive/path.c:symbol or Guidebook section"
  reference_seen: true
provenance_status: Unknown | Reviewed | Approved | Blocked
approval:
  approval_reviewer: Project owner
  approval_reviewed_at: 2026-07-20
  license_id: NGPL
  license_scope: whole AIHack derivative distribution
  notice_required: true
  modification_notice_required: true
  evidence: Project owner derivative classification; AI-assisted semantic rewrite from NetHack 3.6.7 source
preconditions:
  seed: 42
  level: main:1
  player: {}
  world: {}
commands: []
expected:
  accepted_turns: 0
  events: []
  state: {}
  hash_fields: []
implementation:
  task: R7-2
  module: ""
test:
  file: tests/nethack_367_compat.rs
  function: ""
  command: cargo test --workspace --locked --test nethack_367_compat
review:
  reviewer: ""
  reviewed_at: ""
```

## 4. 작성 규칙

- 한 record는 한 관찰 가능한 rule만 검증한다.
- precondition은 fixture builder로 재현 가능해야 한다.
- command는 typed `CommandIntent` 이름과 payload를 쓴다.
- expected event는 variant와 핵심 field를 쓴다.
- hash 전체 값을 source truth로 삼지 않고 hash에 포함될 field를 나열한다.
- 공식 source 문장, 게임 메시지, 데이터 테이블을 장문 복사하지 않는다.
- `reference_seen: true`이면 독립 작성 review를 별도 reviewer가 수행한다.
- test function이 없으면 Verified가 될 수 없다.
- provenance_status가 Approved가 아니면 release compatibility count에 포함하지 않는다.
- Approved이면 approval reviewer/date/license/scope/notice/modification-notice/evidence가 모두 유효해야 한다. 상태 문자열만 바꾸면 checkpoint는 FAIL한다.

## 5. 완료 게이트

```bash
test "$(find docs/compatibility -maxdepth 1 -name 'NH367-C*.md' | wc -l)" -eq 10
cargo test --workspace --locked --test nethack_367_compat
cargo test --workspace --locked --test golden_phase8_rules
```

R7 engineering 완료에는 NH367-C001..C010 record 10개와 test 10개가 필요하다. Approved provenance 10개와 runtime content approval는 완료됐으며, R8은 workspace NGPL·공식 LICENSE·NOTICE·source archive 계약과 전체 기술 gate를 추가로 검증한다.
