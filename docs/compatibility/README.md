# NetHack 3.6.7 Compatibility Records

문서 상태: active template, scenarios planned
작성일: 2026-07-15
관련 Task: R7-2
기준: `../../spec.md` 13절, `../../PROVENANCE.md`

## 1. 목적

각 record는 NetHack 3.6.7의 관찰 가능한 규칙 하나를 source, 입력, 기대 결과, Rust test에 연결한다. record는 원본 코드나 문장을 복사하는 장소가 아니다.

## 2. 예정 scenario

| ID | 범위 | 최소 기대 |
| --- | --- | --- |
| NH367-C001 | 벽 이동 | 위치와 turn 정책 |
| NH367-C002 | 닫힌 문 | open/blocked 상태 전이 |
| NH367-C003 | bump attack | hit/damage/death event |
| NH367-C004 | pickup/wield/quaff | inventory와 item state |
| NH367-C005 | stairs 왕복 | level state 보존 |
| NH367-C006 | search hidden door/trap | reveal 조건 |
| NH367-C007 | throw/zap/read | charge와 projectile stop |
| NH367-C008 | hunger/status | threshold 전이 |
| NH367-C009 | save/load continuation | command 결과와 hash 동일 |
| NH367-C010 | death/game over | cause와 final state |

현재 상태는 모두 Planned다. 해당 test와 승인된 source record가 생기기 전 PASS로 표시하지 않는다.

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
  reference_seen: false
provenance_status: Unknown | Reviewed | Approved | Blocked
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

## 5. 완료 게이트

```bash
test "$(find docs/compatibility -maxdepth 1 -name 'NH367-C*.md' | wc -l)" -eq 10
cargo test --workspace --locked --test nethack_367_compat
cargo test --workspace --locked --test golden_phase8_rules
```

PASS에는 NH367-C001..C010 record 10개, test 10개, Approved provenance 10개가 모두 필요하다.

