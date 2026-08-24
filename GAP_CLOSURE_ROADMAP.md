# AIHack Refactoring Gap Closure Roadmap v2

> Archive chain
> - Latest: `.archive/GAP_CLOSURE_ROADMAP_archive_260715.md`
> - Previous: first archive
>
> Phase 16~20의 완료 gap은 아카이브에 있다. 이 문서는 v0.3.0에서 닫아야 할 active gap만 관리한다.

문서 상태: active gap register
작성일: 2026-07-15
목표 버전: 0.3.0
기준: `spec.md`, `IMPLEMENTATION_SUMMARY.md`

## 1. 운영 규칙

- gap 상태는 `Open -> Implemented -> Verified -> Closed` 순서만 허용한다.
- 코드가 있어도 검증 명령이 실패하면 `Implemented` 이상으로 올리지 않는다.
- 한 gap은 최소 하나의 Task와 하나의 성공 기준 ID에 연결한다.
- baseline hash는 의도와 ADR 없이 갱신하지 않는다.
- 이 문서의 계획 작성은 코드 구현 완료를 의미하지 않는다.
- `Closed`는 해당 gap의 독립 audit 또는 동등한 re-audit evidence가 source·test·문서 정합성을 확인했음을 뜻한다. `Verified`는 구현 검증이 끝났지만 독립 closure evidence가 아직 없는 중간 상태다.
- aggregate gap은 연결된 active child 중 가장 낮은 lifecycle보다 앞설 수 없다. child가 `Implemented`이면 aggregate도 최대 `Implemented`이고, 모두 로컬 gate를 통과한 뒤에만 함께 `Verified`로 올린다.

## 2. 우선순위

| 등급 | 의미 | 다음 단계 진입 |
| --- | --- | --- |
| P0 | 제품·법적·상태 무결성 위험 | 해당 gap이 소속된 phase checkpoint를 PASS로 선언하기 전 반드시 Closed |
| P1 | 빌드·테스트·경계 신뢰성 위험 | 해당 gap이 소속된 phase checkpoint를 PASS로 선언하기 전 반드시 Closed. 후속 phase 착수는 명시된 dependency와 local evidence를 따르며, 외부 CI 같은 별도 evidence pending은 해당 phase의 final PASS만 막는다. |
| P2 | 구조 확장성·문서 드리프트 | R8 전 반드시 Closed |

## 3. Active gap 목록

| Gap ID | 등급 | 문제 | 현재 증거 | Task | 성공 기준 | 상태 |
| --- | --- | --- | --- | --- | --- | --- |
| G-PRODUCT-001 | P0 | inspired game와 3.6.7 변환 목표 충돌 | 과거 spec은 1:1 포트 비목표 | R0-1, R0-2 | SC-DOC-01 | Closed |
| G-LICENSE-001 | P0 | Apache/NGPL 범위와 손상된 NGPL 사본 | owner approval ID, whole-work NGPL, 공식 LICENSE/NOTICE와 modification manifest; `audit_report_21.md` 종결 및 `2519bc8e0ede81c39f46b5778e62a41d4ca66901` Actions `32107862171` 양 OS success | R8-1 | SC-LICENSE-01 | Closed |
| G-BUILD-001 | P1 | toolchain/MSRV 미고정 | rust-toolchain과 locked local build/audit evidence; `audit_report_3.md` independent verification | R1-1 | SC-BUILD-01 | Closed |
| G-BUILD-002 | P1 | UI dependency의 RustSec advisory와 crossterm 중복 | locked dependency/security policy와 `audit_report_3.md` independent verification | R1-1 | SC-BUILD-01 | Closed |
| G-BUILD-003 | P1 | build script가 copy 실패를 무시 | fail-fast script와 `audit_report_3.md` independent verification | R1-2 | SC-BUILD-01 | Closed |
| G-BUILD-004 | P1 | CI 부재 | report 24 implementation `2519bc8e0ede81c39f46b5778e62a41d4ca66901`, [Actions `32107862171`](https://github.com/Yupkidangju/AIHack/actions/runs/32107862171) Ubuntu/Windows success | R1-3 | SC-BUILD-02 | Closed |
| G-BUILD-005 | P0 | Windows Git Bash R7/R8 checkpoint가 CRLF manifest에서 실패 | report 24 재감사와 Actions `32107862171` 양 OS 검증 | R8/R9 시정 | SC-BUILD-01, SC-COMPAT-01 | Closed |
| G-BUILD-006 | P0 | dependency exception lifecycle/graph drift | future approval, 전체 YAML pin과 repository-root local action recursion; SHA `ea7822a5` Actions `32683076204` cargo-deny 0.19.4 양 OS success | report 27 시정 | SC-BUILD-01, SC-LICENSE-01 | Verified |
| G-RUN-001 | P1 | README 기본 실행 명령 실패 | default binary/run command와 `audit_report_3.md` independent verification | R1-1 | SC-BUILD-01 | Closed |
| G-CORE-001 | P0 | session/world mutable field 공개 | private state, read accessor, fixture boundary와 `audit_report_3.md` independent verification | R2-1, R2-2 | SC-CORE-01 | Closed |
| G-CORE-002 | P1 | submit/accept mutation과 commit 결합 | cloned working-copy transaction과 no-commit regression, `audit_report_3.md` independent verification | R2-3 | SC-CORE-02 | Closed |
| G-CORE-003 | P1 | invariant가 타입으로 검증되지 않음 | 6종 invariant/no-commit regression과 `audit_report_3.md` independent verification | R2-2 | SC-CORE-02 | Closed |
| G-CORE-004 | P0 | allocator/custom registry/equipment removal consumer 불변식 | exact successor/fallible spawn, bootstrap saveability와 common removal; SHA `9725c378` Actions `32694375654` 양 OS success | report 28 시정 | FIN-F001, FIN-F007 | Verified |
| G-CORE-005 | P1 | public projectile/monster primitive의 직접 `Err` partial mutation | 외부 mutation을 atomic `GameSession::submit`으로 한정하고 low-level system을 crate 내부로 축소, `1fa6d90` local gate GREEN | report 29 시정 | R29-DBG-F002 | Verified |
| G-CORE-006 | P0 | submit-only master와 broader public World/system mutation 불일치 | default visibility/testing 격리와 `ed02dbf/32733235414` external compile·양 OS GREEN | report 30 시정 | R30-IMP-F001, FIN-F005 | Verified |
| G-DATA-001 | P1 | TOML loader가 runtime과 분리 | runtime ContentRegistry factory/level construction과 `audit_report_3.md` independent verification | R3-1..R3-3 | SC-DATA-01 | Closed |
| G-DATA-002 | P1 | invalid embedded content가 session bootstrap에서 panic 가능 | fallible TUI/headless bootstrap, injected missing level/item regression, `audit_report_3.md` independent verification | R3-1, R3-4 | SC-DATA-01 | Closed |
| G-DATA-003 | P0 | item ID-kind identity 분리와 multi-scalar glyph 축약 | canonical ID-kind table, shape-valid mismatch reject와 exact-one-scalar Unicode matrix, `1fa6d90` local gate GREEN | report 29 시정 | R29-IMP-F001, R29-IMP-F002 | Verified |
| G-TEST-001 | P0 | 1000턴 명령이 18~28턴 사망을 성공 처리 | policy runner 조기 실패 처리와 3 seed accepted turn 1000, `audit_report_6.md` 재감사 | R4-1, R4-2 | SC-TEST-01 | Closed |
| G-TEST-002 | P1 | long-run 반복 hash가 실제 1000 accepted turn을 증명하지 않음 | 3 seed x 1000 accepted turn x 3회 hash, `audit_report_6.md` 재감사 | R4-2 | SC-TEST-02 | Closed |
| G-TEST-003 | P0 | R9 causal witness 원인별 독립성 | 동일 command/observer를 유지한 9종 field-only A/B, exactly-one loss와 나머지 8개 record equality; SHA `ea7822a5` Actions `32683076204` 양 OS success | report 27 시정 | SC-CAUSE-05, SC-CAUSE-06, SC-CAUSE-07 | Verified |
| G-ARCH-001 | P2 | core/UI/LLM이 한 package dependency tree 공유 | 7개 crate/app workspace, app core 직접 의존 0건, R4 hash 유지, `audit_report_6.md` 재감사 | R5-1, R5-2 | SC-ARCH-01 | Closed |
| G-LLM-001 | P0 | 실제 local LLM provider 없음 | loopback transport·strict response validation·bounded worker 구현, `audit_report_11.md` 독립 재감사 | R6-1, R6-6 | SC-LLM-01 | Closed |
| G-LLM-002 | P0 | timeout이 provider 인자일 뿐 강제되지 않음 | transport deadline·deterministic fallback·재현 fixture 구현, `audit_report_11.md` 독립 재감사 | R6-1, R6-6 | SC-LLM-01 | Closed |
| G-LLM-003 | P0 | stale request와 현재 session correlation 없음 | versioned request/response, opaque request ID, current revision/ActionSpace 이중 gate와 submit 직전 재검증; `audit_report_11.md` 독립 재감사 | R6-2, R6-6 | SC-LLM-02 | Closed |
| G-LLM-004 | P1 | LLM 판정의 권한 범위 미정 | strict soft verdict와 Neutral fallback을 presentation-only TUI state로 구현, core/save/replay effect 0; `audit_report_11.md` 독립 재감사 | R6-3, R6-6 | SC-LLM-03 | Closed |
| G-COMPAT-001 | P1 | NetHack 규칙 출처와 테스트 trace 없음 | NH367-C001..C010 record/test와 독립 재감사 42 tests; license risk는 별도 ledger | R7-2 | SC-COMPAT-01 | Closed |
| G-DOC-001 | P2 | Cargo 0.1.0과 문서 v0.2.0 불일치 | 0.3.0 동기화와 report 20 active-status/false-green 시정을 `audit_report_21.md`가 Verified | R8-1 | SC-DOC-01 | Closed |
| G-DOC-004 | P0 | 과거 finding과 당시 권한 상태 혼재 | report 26 최종 `1e84a94/32660514315` 계보 복구와 당시 report 27 `ea7822a5/32683076204` authority 동기화 | report 27 시정 | SC-DOC-01 | Verified |
| G-DOC-005 | P2 | SC-CAUSE-01..07 개별 ID mapping | report 24와 same-SHA Actions `32107862171`이 종결 | report 24 시정 | SC-CAUSE-01..07 | Closed |
| G-DOC-006 | P2 | implementation summary 후반 stale next-step | report 28 current lifecycle과 section 1/10/11 negative regression, Actions `32694375654` | report 28 시정 | FIN-F012 | Verified |
| G-DOC-007 | P0 | active 문서 current-authority false-green 재발 | report 29 단일 authority와 README/ADR/roadmap/summary/build/gap section-scoped predecessor mutation, `1fa6d90` local gate GREEN | report 29 시정 | R29-DOC-F001, R29-DOC-F002 | Verified |
| G-DOC-008 | P0 | designs/compatibility/remediation/roadmap active lifecycle 누락 | report 30 단일 authority/common negative gate와 `ed02dbf/32733235414` 양 OS GREEN | report 30 시정 | R29-DOC-F002 Re-audit #1, FIN-F012 | Verified |
| G-DOC-009 | P0 | implementation summary 10·11절 predecessor current/completed-work false-green | Report 32 independent closure, ADR-0041과 `8c042d48/32741917348` 양 OS GREEN | report 31 시정 | R29-DOC-F002 Re-audit #3, FIN-F012 | Closed |
| G-BUILD-007 | P0 | current HEAD candidate date가 bundled modification period 밖임 | 2026-08-25 Notice ID/period와 actual HEAD-date early regression 표적 GREEN | report 32 시정 | R32-DBG-F001, FIN-F015 | Implemented |
| G-SEC-001 | P0 | artifact link/root/ambient/archive 경계 | Windows alias/staging/link와 archive canonical alias·strict calendar 및 actual 양 OS bundle `32683076204` | report 27 시정 | SEC-F001, FIN-F004, FIN-F014 | Verified |
| G-SEC-003 | P0 | archive Windows component alias와 year 0000 parity | case/trailing/reserved/collision 및 year 0000/0001/9999와 actual 양 OS bundle `32694375654` | report 28 시정 | FIN-F014, FIN-F015 | Verified |
| G-UI-002 | P0 | control-key Repeat state crossing과 F9 실제 경로 evidence | Esc/Enter/F9/Q Press-only sequence와 actual F9 handler regression, Actions `32694375654` | report 28 시정 | FIN-F008, FIN-F009, FIN-F016 | Verified |
| G-SEC-004 | P0 | archive raw type/extraction 및 `ExpectedCommit` complete tree 미결합 | format-aware common validator, safe extraction, independent `git archive` identity와 `1fa6d90` clean Windows actual bundle GREEN | report 29 시정 | R29-SEC-F001, R29-SEC-F002 | Verified |
| G-UI-003 | P0 | 동등 transition과 Release 없는 연속 Press state crossing | 합성 Release 비신뢰·500ms quiet+2 idle gate, constructed/actual ConPTY와 `1fa6d90` local gate GREEN | report 29 시정 | R29-DBG-F001 | Verified |
| G-SEC-002 | P2 | Windows save owner-only 권한 과대주장 | report 24와 Actions `32107862171`이 Unix 0600/Windows parent DACL 계약을 종결 | report 24 시정 | SEC-F003 | Closed |
| G-FINAL-001 | P0 | final multi-audit FIN-F001..F018 및 report 32 release-date HOLD | report 31 lifecycle/FIN-F012 Closed; R32 date contract Implemented, exact-final-headSha Actions가 external verification authority | report 32 remediation | R32-DBG-F001, FIN-F001..F018 | Implemented |
| G-DOC-002 | P2 | 완료 이력과 active 계약 혼재 | spec/summary/audit 600~1250 lines | R0-1, R0-2, R0-3 | SC-DOC-01 | Closed |
| G-DOC-003 | P2 | LLM interface scaffold가 live integration 완료로 표현 | 과거 Phase 12/13 문서 | R0-1, R0-2, R0-3 | SC-DOC-01 | Closed |

## 4. Gap별 수정 계약

### 4.1 G-PRODUCT-001

**결정:** NetHack 3.6.7 행동 호환 재구현으로 동결한다.

**금지:**

- 줄 단위 C-to-Rust 변환
- 출처 없는 데이터/문자열 복사
- full parity를 v0.3.0 완료 조건으로 사용

**완료 증거:**

- `spec.md` DEC-PRODUCT-01
- ADR-0021
- README의 현재/목표 범위
- NH367-C001..C010 trace

### 4.2 G-BUILD-001..004와 G-RUN-001

**수정:**

- Rust 1.94.1과 rust-version 1.94
- ratatui 0.30/crossterm 0.29
- default-run aihack
- 모든 자동 명령에 `--locked`
- build script artifact fail-fast
- Linux/Windows CI

**완료 증거:**

```bash
cargo tree -d
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo run --locked -- --seed 42
```

crossterm 중복 0건, 마지막 명령이 TUI binary를 선택해야 한다.

### 4.3 G-CORE-001..003

**수정:**

- session/world field private
- query API와 fixture builder
- prepare/validate/commit transaction
- accepted turn마다 invariant 6종

**완료 증거:**

- `tests/world_invariants.rs`
- `tests/transaction.rs`
- mutable field assignment search 0건
- existing golden hash 유지

### 4.4 G-DATA-001..002

**수정:**

- embedded TOML parse 1회
- immutable `ContentRegistry`
- ContentError 6종
- runtime factory가 registry ID로 생성

**완료 증거:**

- dagger와 jackal 값을 TOML에서 바꾸면 runtime fixture test 기대값도 바뀜
- duplicate/unknown/invalid coordinate가 panic이 아닌 typed error
- registry content hash 반복 3회 동일

### 4.5 G-TEST-001..002

**수정:**

- `HeadlessPolicyId`와 `HeadlessRunReport`
- survival-v1 policy
- accepted_turns와 requested_turns 분리
- GameOver early break는 non-zero exit

**완료 증거:**

| seed | requested | required accepted | repeats |
| --- | ---: | ---: | ---: |
| 42 | 1000 | 1000 | 3 |
| 7 | 1000 | 1000 | 3 |
| 1234 | 1000 | 1000 | 3 |

각 seed의 3개 final hash는 동일해야 한다.

### 4.6 G-ARCH-001

**수정:** R4 behavior gate 뒤 workspace로 순차 추출한다.

**완료 증거:**

- `cargo tree -p aihack-core`에 ratatui, crossterm, HTTP client 없음
- TUI/headless binary 이름과 인자 유지
- workspace all-target test 통과

**2026-07-17 closure:** `audit_report_6.md`가 path dependency version, 실행 가능한 audit 명령, 활성 문서와 source/test 정합성을 재검증해 PASS했으므로 `Closed`다.

**후속 감사 상태:** `audit_report_9.md`가 IMP-F008 시정과 R1~R5 전체 회귀를 PASS로 종결했다. R5 closure는 유지되며 다음 구현 단계는 R6다.

### 4.7 G-LLM-001..004

**수정:**

- loopback OpenAI-compatible transport
- 500ms connect, 2000ms narrative, 1500ms decision timeout
- request_id + turn + snapshot_hash correlation
- current action space 재검증
- request 32,768 bytes, response 65,536 bytes, queue capacity 16
- redirect/proxy/remote IP와 control/ANSI response 거부
- SoftVerdict는 presentation-only

**완료 증거:**

| Case | status | submit 호출 | hash 영향 |
| --- | --- | ---: | ---: |
| disabled | `LlmEnqueueError::Disabled` | 0 | 없음 |
| connect failure | `LlmResponseError::Unavailable` | 0 | 없음 |
| timeout | `LlmResponseError::Timeout` | 0 | 없음 |
| invalid JSON | `InvalidSchema` | 0 | 없음 |
| stale revision | `LlmResponseError::Stale` | 0 | 없음 |
| valid legal action | `LlmPayload::Decision` | 사용자 승인 시 1 | 정상 command 영향만 |
| soft adjudication | `LlmPayload::SoftAdjudication` 또는 Neutral UI fallback | 0 | 없음 |

**2026-07-18 audit closure:** `audit_report_10.md`의 R6 HOLD 이후 public versioned projection·독립 ActionSpace·request/response schema 0/2 rejection·synchronous payload bound·public enum stability를 구현했고, 저장소 fixture로 success/timeout/stale/down 및 pending-exit matrix를 재현했다. `audit_report_11.md`가 IMP-F009/010/011, DBG-F004와 XPF-F007을 Verified하고 R6 checkpoint를 PASS로 종결했으므로 G-LLM-001..004는 `Closed`다. 실제 model provider smoke는 비차단 고려 대상이다.

### 4.8 G-LICENSE-001과 G-COMPAT-001

**수정:**

- `PROVENANCE.md` 상태: Unknown, Reviewed, Approved, Blocked
- runtime inclusion은 Approved만 허용
- NH367 scenario 문서에 source path/function과 관찰 결과 기록
- 공식 source archive checksum 별도 기록

**완료 증거:**

- source 직접 import search 0건
- Unknown/Blocked runtime asset 0건
- NH367-C001..C010 test 통과

**2026-07-18 implementation status:** 공식 archive/license checksum, runtime inventory, legacy 격리 자동 검증과 NH367-C001..C010 record/test를 구현했다. 보고서 13의 단계 순환과 audit-root 지적은 시정됐고 보고서 14에서 Verified됐다. 사용자 결정에 따라 PROV-0004와 scenario의 actual approval는 R8 런칭 전 최종 검토로 이관했다. G-COMPAT-001은 engineering 범위에서 Closed이며 G-LICENSE-001은 외부 배포를 차단하는 final launch gate다.

**2026-07-20 licensing closure:** 프로젝트 소유자가 AIHack 전체를 NetHack 3.6.7 원본 source 기반 AI-assisted semantic rewrite 파생물로 분류하고 NGPL 배포를 승인했다. PROV-0004와 scenario 10개를 근거 포함 `Approved`로 전환했고, workspace 0.3.0/NGPL, 공식 `LICENSE`, `NOTICE`, release source archive와 R8 fail-closed 검증을 구현했다. 로컬 SC-LICENSE-01은 충족했으며 `Closed` 전환은 독립 R8 감사 evidence를 기다린다.

**2026-07-20 report 16 remediation:** 직접 project-owner 지시와 범위를 `AIHACK-OWNER-2026-07-20-NGPL-01`로 기록하고 PROV/scenario에 연결했다. root NOTICE의 배포되지 않는 Git history 주장은 bundle-carried `MODIFICATIONS.md`와 commit-expanded `RELEASE-METADATA`로 교체했다. 임시 clean Git commit으로 실제 tar archive의 required files, commit identity, checksum과 legacy exclusion을 검증하는 회귀 테스트를 추가했다. qualified legal opinion은 주장하지 않으며, 실제 R8 clean commit과 same-commit CI는 아직 final gate다.

### 4.9 G-DOC-001..003

**수정:**

- current와 target을 모든 문서에서 분리
- 과거 이력은 archive chain
- LLM scaffold는 현재 상태, live provider는 R6 target
- v0.3.0 release 시 manifest/doc version 동시 변경

**완료 증거:**

- AI 구현 문서 표준 12항목 PASS
- README, spec, summary, build, audit의 명령과 Phase ID 일치
- 한 문서에만 정의된 R/SC/Gap ID 0건

## 5. 의존 그래프

```text
R0
 ├─ R1 ─ R2 ─ R3 ─ R4 ─ R5 ─ R6 ─┐
 └─ R7-1 ─────── R7-2(after R3) ───┤
                                    v
                                   R8
```

## 6. Checkpoint 정책

- R1: 빌드 재현성
- R2: 상태 무결성
- R3: 데이터 진실 원천
- R4: 실제 장기 결정론
- R5: dependency boundary
- R6: LLM degraded/stale safety
- R7: compatibility/provenance
- R8: release/document closure

checkpoint에서 하나라도 실패하면 후속 Phase 구현을 중단하고 같은 Phase에서 원인을 수정한다.

## 7. 현재 완료 범위

R0~R8 기존 remediation과 report 23/24는 역사적으로 종결됐다. report 25는 partial, report 26~29는 historical/technical evidence로 보존한다. report 30의 G-CORE-006/G-DOC-008은 `ed02dbf/32733235414` clean same-SHA 양 OS actual bundle과 Report 31 독립 검증으로 Verified다.

Report 31 G-DOC-009/FIN-F012는 successor `8c042d48/32741917348`와 Report 32 independent closure로 Closed다.

현재 authority는 `docs/audit/audit_report_32.md`다. G-BUILD-007/G-FINAL-001은 R32-DBG-F001/FIN-F015의 Notice ID/period, actual HEAD-date gate와 final same-SHA 양 OS evidence가 끝날 때까지 Open이며, 후속 독립 감사와 별도 게시 승인 전까지 Closed 및 program/publication PASS로 올리지 않는다.
