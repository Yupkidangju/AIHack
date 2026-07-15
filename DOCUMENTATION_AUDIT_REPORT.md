# AIHack Documentation Audit Report

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
