# AIHack Documentation Audit Report

> Historical scope notice (2026-07-20): 1~9절은 2026-07-15의 **R0 계획 문서 전용** 감사 기록으로 보존한다. 당시 판정과 수치는 수정하지 않는다. 현재 R8 문서 구현 상태는 10절의 self-check와 후속 독립 `audit_report_[turn].md`를 authority로 사용한다.

감사일: 2026-07-15
감사 범위: v0.3.0 리팩터링 구현 계획 문서
판정 기준: `AI_IMPLEMENTATION_DOC_STANDARD.md`
코드 구현 범위: 제외

## 1. 최종 판정

**R0 Documentation: PASS**

이 판정은 후속 구현자가 R1-1부터 작업을 시작할 수 있을 정도로 계획 계약이 닫혔다는 뜻이다. R1~R8의 코드, 설정, CI, local LLM, provenance 승인, compatibility test는 구현 전이므로 NOT RUN이다. 전체 제품 release PASS를 뜻하지 않는다.

## 2. 감사 대상

- `spec.md`
- `IMPLEMENTATION_SUMMARY.md`
- `GAP_CLOSURE_ROADMAP.md`
- `designs.md`
- `DESIGN_DECISIONS.md`
- `BUILD_GUIDE.md`
- `audit_roadmap.md`
- `PROVENANCE.md`
- `docs/compatibility/README.md`
- `README.md`
- `CHANGELOG.md`

보존 snapshot:

- `.archive/spec_archive_260715.md`
- `.archive/IMPLEMENTATION_SUMMARY_archive_260715.md`
- `.archive/GAP_CLOSURE_ROADMAP_archive_260715.md`
- `.archive/audit_roadmap_archive_260715.md`
- `.archive/designs_archive_260715.md`
- `.archive/BUILD_GUIDE_archive_260715.md`
- `.archive/DESIGN_DECISIONS_archive_260715.md`

## 3. AI 구현 문서 표준 체크리스트

| # | 기준 | 판정 | 증거 |
| --- | --- | --- | --- |
| 1 | 목표와 비목표 | PASS | `spec.md` 2~4절 |
| 2 | 핵심 결정 동결 | PASS | DEC table, ADR-0021..ADR-0027 |
| 3 | 큰 타입 계약 | PASS | Session, transaction, content, headless, LLM typed contract |
| 4 | 고유 ID 폐쇄 | PASS | SC, DEC, Gap, Task, CTA, NH367 ID 정의와 중복 0건 |
| 5 | 공식과 수치 | PASS | combat/vision 공식, timeout, size, queue, turn 기준 |
| 6 | 실제 데이터 | PASS | player/monster/item/level, LLM request/response, report schema |
| 7 | 구현 순서 | PASS | R0~R8 dependency와 Task별 선행 |
| 8 | 검증 명령과 checkpoint | PASS | `audit_roadmap.md` R0~R8 |
| 9 | 화면 버튼 정책 | PASS | CTA ID, input, 활성 조건, 후속 상태 |
| 10 | 저장·설정·bridge 정책 | PASS | SaveDataV1, ReplayLineV1, env/CLI, GameClient/LocalLlmPort |
| 11 | 문서 간 용어와 타입 | PASS | ID closure script, headless/LLM/compatibility 필드 통일 |
| 12 | 잔여 리스크의 착수 가능성 | PASS | gap owner Task와 release blocking boundary 명시 |

## 4. 자동 검증 결과

| 검사 | 결과 |
| --- | --- |
| 필수 문서 10개 non-empty | PASS |
| archive 7개 non-empty | PASS |
| SC 14개 정의와 cross-reference | PASS |
| Task 22개 heading과 cross-reference | PASS |
| Gap 23개 unique definition | PASS |
| active ADR 7개 unique definition | PASS |
| 표준 금지·모호 표현 | PASS, 0건 |
| Markdown fence parity | PASS |
| relative link existence | PASS |
| active 문서 `git diff --check` | PASS |
| current `cargo fmt --all -- --check` | PASS |
| current `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| current `cargo test --all-targets --locked` | PASS, 186 tests |
| source/test/build script working-tree 변경 | PASS, 0건 |

현재 build 검증은 코드 기준선이 문서의 “현재 상태”와 일치하는지만 확인했다. SC-BUILD-01/02는 toolchain file, dependency downgrade, CI가 구현되지 않아 아직 PASS가 아니다.

## 5. 교차 문서 폐쇄 결과

- 제품 정체성은 “NetHack 3.6.7 행동 호환 clean reimplementation”으로 통일했다.
- 현재 Cargo 0.1.0과 target 0.3.0을 분리했다.
- 현재 ratatui 0.30/crossterm 0.28 혼합과 target ratatui 0.29/crossterm 0.28.1을 분리했다.
- current headless의 조기 사망과 target accepted-turn 1000 성공을 분리했다.
- LLM scaffold와 live provider target을 분리했다.
- LLM error는 enqueue error, response error, UI status로 역할을 분리했다.
- headless report field는 `requested_turns`, `accepted_turns`, `submitted_commands`, `final_state`, `final_hash`, `error`로 통일했다.
- NH367-C001..C010 의미를 spec과 compatibility template에서 통일했다.
- internal Result API 변경과 ReplayLineV1 wire 호환을 `ReplayTurnOutcomeV1` projection으로 닫았다.
- GameOver new run은 seed wrapping-add, Title 복귀, transient reset으로 통일했다.

## 6. 보안·API 검토

- external LLM response는 untrusted input으로 취급한다.
- endpoint는 HTTP loopback host와 resolved loopback IP만 허용한다.
- redirect, proxy, remote host, state patch를 금지한다.
- request 32,768 bytes, response 65,536 bytes, queue 16, CTA cooldown 250ms를 고정했다.
- typed input/output, opaque request ID, revision gate, consistent error enum을 정의했다.
- user text와 response의 control/ANSI escape, unknown field, invalid action을 거부한다.
- cargo-audit 0.22.1과 cargo-deny 0.19.4 gate를 R1/R8에 추가했다.
- save/replay/report path traversal과 symlink escape를 거부하고 save atomic replace를 정의했다.

## 7. 남은 차단 조건

| 범위 | 상태 | 해제 조건 |
| --- | --- | --- |
| R1 build reproducibility | Open | Rust/dependency/script/CI 구현과 SC-BUILD PASS |
| R2 state integrity | Open | private state, transaction, 6 invariant |
| R3 content truth | Open | runtime ContentRegistry와 typed error |
| R4 long run | Open | three seeds, accepted turn 1000, 3회 hash |
| R5 workspace | Open | crate extraction과 SC-ARCH-01 |
| R6 local LLM | Open | live loopback transport와 SC-LLM-01..03 |
| R7 provenance | Open | reviewer 승인과 runtime Approved 자산만 포함 |
| R7 compatibility | Open | NH367 record/test 10개 |
| R8 release | Open | 모든 checkpoint와 version sync |

license scope와 root 배포 license는 프로젝트 소유자 또는 적격 검토자의 승인이 필요하다. 이 승인은 문서 작성자가 대신하지 않았으며, 승인 전 외부 release를 차단하는 조건으로 남겼다.

## 8. 구현 착수점

다음 세션의 첫 작업은 `IMPLEMENTATION_SUMMARY.md`의 **Task R1-1**이다. R1-1은 `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` 세 파일만 수정하고 해당 Task의 세 검증 명령이 통과한 뒤 종료한다.

## 9. 외부 근거

- [NetHack 3.6.7 official source download and SHA-256](https://www.nethack.org/v367/download-src.html)
- [reqwest 0.13.4 blocking ClientBuilder timeout contract](https://docs.rs/reqwest/latest/reqwest/blocking/struct.ClientBuilder.html)
- [RustSec cargo-audit usage](https://github.com/rustsec/rustsec/blob/main/cargo-audit/README.md)
- [cargo-deny license, bans, advisories, sources checks](https://github.com/EmbarkStudios/cargo-deny)

## 10. R8 문서 동기화 self-check (2026-07-20)

**R8 Documentation Self-check: PASS**

이 판정은 2026-07-20 구현 세션의 로컬 정합성 검사다. 1절의 역사적 R0 판정을 바꾸지 않는다. 당시 독립 R8 감사와 SC-BUILD-02는 pending이었으며, 2026-07-22 후속 상태는 10.7절이 대체한다.

### 10.1 변경 기준

- Cargo/README/CHANGELOG 0.3.0 동기화
- ADR-0030의 project-owner 파생물 분류와 whole-work NGPL 결정
- official LICENSE checksum과 파생·변경 `NOTICE`
- clean release commit의 complete corresponding source packaging
- `scripts/r8_checkpoint.sh`의 canonical-root fail-closed 검증

### 10.2 AI 구현 문서 표준 12개 항목

| # | 기준 | R8 self-check | 현재 근거 |
| --- | --- | --- | --- |
| 1 | 목표와 비목표 | PASS | `spec.md`의 v0.3.0 범위와 R8 외부 게시 경계 |
| 2 | 핵심 결정 동결 | PASS | ADR-0021..ADR-0030; license 결정은 ADR-0030 |
| 3 | 큰 타입 계약 | PASS | session/content/LLM/save/replay 계약과 실제 workspace API |
| 4 | 고유 ID 폐쇄 | PASS | SC/Gap/Task/CTA/NH367/PROV ID와 checkpoint |
| 5 | 공식과 수치 | PASS | turn/hash, timeout/size/queue, 공식 LICENSE SHA-256 |
| 6 | 실제 데이터 | PASS | embedded player/monster/item/level과 NH367 scenario 10개 |
| 7 | 구현 순서 | PASS | R1~R8 dependency와 R8-0/R8-1A..C slice |
| 8 | 검증 명령과 checkpoint | PASS | `audit_roadmap.md` R8 명령과 `scripts/r8_checkpoint.sh` |
| 9 | 화면 버튼 정책 | PASS | `designs.md` CTA ID·활성 조건·후속 상태 |
| 10 | 저장·설정·bridge 정책 | PASS | SaveDataV1/ReplayLineV1, GameClient, loopback LLM 설정 |
| 11 | 문서 간 용어와 타입 | PASS | 0.3.0, NGPL, provenance, release packaging 용어 동기화 |
| 12 | 잔여 리스크의 착수 가능성 | PASS | 독립 R8 감사와 SC-BUILD-02 원격 CI를 별도 gate로 격리 |

### 10.3 로컬 검증 상태

| 검사 | 결과 |
| --- | --- |
| R7 checkpoint | PASS |
| R8 checkpoint | PASS |
| documentation contract regression | PASS |
| `scripts/r8_tui_core_flow.sh` | PASS; high contrast/reduced motion, core flow, Game Over/New Run, 59x23 clean exit |
| R6 deterministic PTY degraded matrix | PASS; success/timeout/stale/down, pending-exit restore |
| fmt / clippy / full workspace test / release build | PASS |
| cargo-audit / cargo-deny | PASS; cargo-audit는 local advisory DB 사용 |
| 독립 R8 문서 시정 감사 | PASS — report 21이 report 20의 `IMP-F016`/`DBG-F008`/`XPF-F011` 종결 |
| Linux/Windows remote CI evidence | PASS — R9 commit `41a1b63f11a57a671b0f705883431dab24298b5a`, Actions `32034295607` |
| report 23/24 통합 시정 | report 24와 same-SHA Actions `32107862171`로 historical closed |
| final multi-audit report 1 | HOLD / 2026-08-23 coder remediation과 전체 재검증 진행 |

따라서 R8 문서 구현은 독립 감사에 전달 가능한 상태지만, 전체 프로그램 또는 외부 배포 `PASS`는 아직 선언하지 않는다.

### 10.4 `audit_report_16.md` HOLD 후속 상태

- `IMP-F015`: compatibility index 10행을 개별 `Approved` record와 동기화하고 파싱 회귀 테스트 추가
- `IMP-F012`: direct user instruction과 승인 범위를 `AIHACK-OWNER-2026-07-20-NGPL-01`로 기록해 PROV/scenario에 연결; qualified legal opinion은 주장하지 않음
- `IMP-F014`: 배포되지 않는 Git history 주장을 제거하고 `MODIFICATIONS.md`, commit-expanded `RELEASE-METADATA`, `SHA256SUMS`와 실제 archive verifier로 대체
- `DBG-F006`: clean R8 commit, 실제 release package와 same-commit Ubuntu/Windows CI는 미완료이므로 HOLD 유지

이 항목은 coder remediation 상태이며 독립 재감사 판정을 대신하지 않는다.

### 10.5 `audit_report_17.md` HOLD 후속 상태

- `IMP-F014`, `IMP-F015`: 독립 재감사에서 Verified
- `DBG-F007`: approval record를 output/source archive 필수 항목으로 추가하고 metadata owner/modification ID와 실제 bundled record ID를 대조하도록 시정
- 누락·불일치 actual archive fixture와 양 플랫폼 packaging 계약을 회귀 테스트로 고정
- `IMP-F012`, `DBG-F006`: 이 tree를 포함하는 release commit과 same-commit Ubuntu/Windows CI가 확인될 때까지 HOLD 유지

이 항목은 coder remediation 기록이며 report 17 또는 후속 독립 감사 판정을 덮어쓰지 않는다.

### 10.6 `audit_report_18.md` HOLD 후속 상태

- `DBG-F007`: Linux 부분 문자열 검증을 단일 key·완전 값 parser로 교체
- archive/output 각각의 wrong, suffix, duplicate owner/modification metadata를 actual archive fixture로 거부
- Windows release gate도 같은 key 단일성·완전 값 계약으로 동기화
- 이 coder 시정은 same-commit 양 OS CI와 후속 독립 감사 판정을 대신하지 않음

### 10.7 `audit_report_19.md` HOLD 후속 상태 (2026-07-22)

- `IMP-F012`, `IMP-F014`, `IMP-F015`, `DBG-F006`, `DBG-F007`: 기술·release evidence Verified
- SC-BUILD-02: 2026-07-22 기준 commit `b9bd680200d82b20d7c9ba961a2758caa3d49e16`, [Actions run `29886410221`](https://github.com/Yupkidangju/AIHack/actions/runs/29886410221)의 `ubuntu-latest quality gate`와 `windows-latest quality gate` 및 release bundle PASS
- `IMP-F016`/`XPF-F011`: 활성 문서의 pending/NOT RUN 상태를 위 evidence와 report 19 documentation-sync HOLD로 동기화
- 당시 권한 상태: 문서 시정 완료 후 독립 재감사 대기. 이 self-check는 최종 R8 PASS를 대신하지 않으며 외부 게시를 승인하지 않는다.

### 10.8 `audit_report_20.md` HOLD 후속 상태 (2026-07-22)

- `IMP-F016`: 최상단 current baseline, `G-LICENSE-001`과 BUILD_GUIDE의 고정 테스트 수를 현재 evidence와 정렬
- `DBG-F008`: document-wide token 검사에 section/row별 positive·negative assertion을 추가
- 현재 권한 상태: report 20 시정 완료 후 독립 재감사 대기. 기술 evidence는 다시 pending으로 되돌리지 않으며 최종 R8 PASS나 외부 게시를 선언하지 않는다.

### 10.9 `audit_report_21.md` 종결 상태 (2026-07-22)

- `IMP-F016`, `DBG-F008`: Verified
- `XPF-F011`: Resolved
- report 20의 active-state/false-green 시정은 PASS로 종결됐으며 해당 재감사 대기는 더 이상 current state가 아님
- 외부 게시 승인은 별도 운영 gate로 유지

### 10.10 `docs/audit/audit_report_22.md` / `docs/audit/audit_report_23.md` 역사적 상태 (2026-08-18)

- report 22의 5절은 Initial Finding, 4·7절은 2026-08-17 post-fix 판정으로 분리했으며 report 23이 장기 witness false-green을 후속 HOLD함
- report 23의 SEC-F001, TEST-F001, DBG-F009, IMP-F016/017은 report 24 재감사와 후속 same-SHA CI로 종결됐으며 현재 pending gate로 사용하지 않음
- `hallucinating` compatibility risk는 Project owner/runtime maintainer가 소유하고 SaveDataV2·v0.4.0 범위 승인 또는 2026-10-31 중 먼저 도래하는 시점에 재검토

### 10.11 `docs/audit/audit_report_24.md` 시정 상태 (2026-08-18)

- `DBG-F011`: winx 0.36.4 전용 SPDX exception과 cargo-deny 0.19.4 실제 PASS로 시정
- `SEC-F003`: Unix mode 0600과 Windows parent DACL 상속을 분리해 문서·코드·platform test를 정렬
- `IMP-F019`: SC-CAUSE-01..07을 audit roadmap/implementation summary의 production 심볼·테스트 함수에 개별 연결
- 현재 권한 상태: implementation SHA `2519bc8e0ede81c39f46b5778e62a41d4ca66901`, Actions `32107862171` 양 OS PASS. 후속 독립 재감사와 외부 게시 승인은 별도 gate

### 10.12 `docs/audit/audit_report_25.md` 역사적 부분 상태 (2026-08-23~24)

- final multi-audit report 1은 audited HEAD `80d959af94cb08c5d9b2f2601f5e63f3827a1210`의 역사적 FIN-F001..F018 입력이며 첫 coder remediation은 partial evidence로 보존
- `docs/audit/audit_report_25.md`는 inverse save relation, writer budget, replay alias, paired score, production TUI/terminal, release actual-set과 active lifecycle을 HOLD했고 해당 시정의 RED/GREEN 및 SHA `b732c42d` Actions `32650404618` clean same-SHA Ubuntu/Windows CI는 유효한 부분 evidence다.
- report 26 독립 재감사가 열거되지 않은 production gap을 재개방했으므로 report 25를 현재 최상위 권위나 전체 closure로 사용하지 않는다.

### 10.13 `docs/audit/audit_report_26.md` 역사적 최종 상태 (2026-08-24)

- report 26은 malformed scalar/ItemData, Win32 trailing-name alias, actual causal producer removal, modal/Inspect mouse, release staging root/hardlink/candidate date와 네 P1 범위를 재현한 역사적 predecessor다.
- ADR-0036, active spec/design/gap과 `docs/audit/audit_report_26_remediation.md`가 문서 우선 계약과 RED/GREEN을 추적한다.
- `fc01ec12/32658658526`은 부분 evidence, `a9a39d8/32660221745`는 Linux pipefail failure이며 최종 verifier fix SHA `1e84a94aa0623b5cee5349b5832992a4682e93a8`의 Actions `32660514315`에서 clean same-SHA Ubuntu/Windows actual bundle까지 Verified됐다.

### 10.14 `docs/audit/audit_report_27.md` 역사적 predecessor 상태 (2026-08-24)

- report 27은 save allocator/level/charge, unsafe custom registry, field-only causal A/B, archive canonical path, strict calendar, debug mouse, Judge repeat와 local action recursion을 재현한 predecessor다.
- ADR-0037, active spec/design/gap과 `docs/audit/audit_report_27_remediation.md`가 문서 우선 계약, 수정 전 RED와 표적 GREEN을 추적한다.
- report 27 시정은 전체 local gate와 implementation SHA `ea7822a5b32b3bb9ee8224176381c44871037bc4`의 Actions `32683076204` clean same-SHA Ubuntu/Windows actual bundle까지 Verified됐다. 완료 후에도 새 독립 감사와 별도 게시 승인 전까지 program/publication HOLD다.

### 10.15 `docs/audit/audit_report_28.md` 역사적 predecessor 상태 (2026-08-24)

- report 28은 allocator MAX-1, invalid custom monster/equipment removal, control-key Repeat/F9 실제 경로, Windows archive component alias, year 0000 parity와 stale summary를 재현한 역사적 predecessor다.
- ADR-0038과 `docs/audit/audit_report_28_remediation.md`가 문서 우선 계약, 수정 전 RED와 표적 GREEN을 추적한다.
- report 28 시정은 전체 local gate와 implementation SHA `9725c37896a8d149be5c500cdd26da154ab0a3fa`의 Actions `32694375654` clean same-SHA Ubuntu/Windows actual bundle까지 Verified됐지만 report 29가 인접 경계를 재개방했다.

### 10.16 `docs/audit/audit_report_29.md` 역사적 technical predecessor 상태 (2026-08-24)

- report 29는 TUI 동등 transition, archive raw type/extraction, `ExpectedCommit` complete tree, document-wide authority, item ID-kind/glyph, allocator fixture와 public atomicity를 재현한 technical predecessor다.
- ADR-0039와 `docs/audit/audit_report_29_remediation.md`가 문서 우선 계약과 RED/GREEN을 추적한다.
- ADR-0039 successor `a91a9c7/32706869079`의 전체 gate와 clean same-SHA Ubuntu/Windows actual bundle은 Verified됐지만 report 30이 authority/public surface를 재개방했다.

### 10.17 `docs/audit/audit_report_30.md` 역사적 technical predecessor 상태 (2026-08-24)

- report 30은 designs/compatibility/remediation/roadmap lifecycle과 broader public World/system mutation을 재개방한 predecessor다.
- ADR-0040과 `docs/audit/audit_report_30_remediation.md`가 문서 우선 계약, external compile RED/GREEN과 새 gate를 추적한다.
- ADR-0040 successor `ed02dbf/32733235414`의 전체 gate와 clean same-SHA Ubuntu/Windows actual bundle 및 Report 31 독립 API 검증은 Verified됐다.

### 10.18 `docs/audit/audit_report_31.md` 역사적 independent closure (2026-08-25)

- report 31은 implementation summary 1·10·11절 lifecycle과 document regression source를 시정한 predecessor다.
- ADR-0041과 `docs/audit/audit_report_31_remediation.md`가 report 번호 기반 predecessor current/completed-work generic negative gate의 RED/GREEN을 추적한다.
- Report 30 public visibility와 Report 29 기술 회귀는 독립 Verified다.
- Report 31 시정의 전체 local quality gate 453 tests와 successor `8c042d48/32741917348` clean same-SHA Ubuntu/Windows actual bundle은 Verified됐고 Report 32가 FIN-F012와 함께 independent Closed로 종결했다.

### 10.19 `docs/audit/audit_report_32.md` 현재 상태 (2026-08-25)

- report 32가 current HEAD release candidate date와 modification evidence lifecycle의 단일 현재 권위다.
- ADR-0042와 `docs/audit/audit_report_32_remediation.md`가 R32-DBG-F001/FIN-F015의 Notice ID/period, actual HEAD-date 조기 gate와 final-SHA bundle을 추적한다.
- current HEAD `8045249` date `2026-08-25`가 bundled period end `2026-08-24` 밖이어서 actual Windows bundle은 fail-closed했다. ADR-0042의 2026-08-25 Notice ID/period, actual HEAD-date 조기 regression, 전체 local quality gate 453 tests와 candidate `57d8108a` clean Windows actual bundle은 PASS했다. final same-SHA 양 OS evidence와 후속 독립 PASS 및 별도 게시 승인 전까지 program/publication HOLD다.
