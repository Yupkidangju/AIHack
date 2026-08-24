# AIHack 감사 보고서 30 시정 기록

- 기준 독립 감사: `docs/audit/audit_report_30.md`
- 기준 HEAD: `1d6e6669c74ba04263e450ac821b443e4bdc053c`
- 작업 브랜치: `codex/audit-report-30-remediation`
- 작업 일자: 2026-08-24
- 현재 판정: **ADR-0040 구현·표적 GREEN 완료 / 전체 local gate·새 CI pending / PROGRAM·PUBLICATION HOLD**

독립 감사 원문은 수정하지 않는다. report 29 기술 successor `a91a9c7/32706869079`의 evidence는 보존하되, report 30의 active authority와 public Rust visibility를 별도 RED/GREEN 및 새 same-SHA 양 OS bundle로 검증한다.

## 1. Finding 대조와 결정

| Finding | current spec/ADR 대조 | public/active entrypoint | 보존할 adversarial fixture | 결정 |
| --- | --- | --- | --- | --- |
| R29-DOC-F002 Re-audit #1 | `spec.md`, ADR-0040 | designs/compatibility/remediation/roadmap 및 `r8_documentation` | 네 stale header와 predecessor current phrase를 각각 복원 | report 30 marker active section exact-one, predecessor는 historical section에서만 허용 |
| R30-IMP-F001 | `spec.md` 9.1, ADR-0040 | default `aihack-runtime`, root facade, TUI/headless dependency | external read compile-pass, World mutator/system compile-fail, shipped feature/import scan | submit-only를 literal visibility로 강제하고 test-only direct fixture만 opt-in `testing` feature로 격리 |

## 2. 문서 선행 변경

production code와 test를 수정하기 전에 다음을 갱신했다.

- `spec.md`: default public read/query·validated restore·submit과 crate-private mutation, feature-gated compatibility fixture 계약
- `DESIGN_DECISIONS.md`: ADR-0040 추가, ADR-0039를 historical technical decision으로 supersede
- `designs.md`, compatibility index, report 29 remediation top status와 audit roadmap: report 30 단일 current lifecycle
- README 5개 locale, implementation summary, gap, build guide, documentation audit, changelog: report 30 current/pending 상태

## 3. 수정 전 RED

독립 감사가 보존한 document RED는 다음 네 active 위치였다.

```text
designs.md -> report 28 remediation / independent pending
docs/compatibility/README.md -> report 28 remediation / independent pending
audit_report_29_remediation.md -> 구현·검증 진행 중
audit_roadmap.md -> local gate·새 CI pending
r8_documentation -> 10 PASS
```

문서와 ADR을 먼저 복구하고 `validate_current_authority` 대상을 designs/compatibility/report29-remediation/report30-remediation header까지 확장했다. 각 header에 report 29/28 stale current phrase를 주입하는 in-memory mutation은 `Err`다.

public compile fixture는 current implementation에서 다음처럼 RED였다.

```text
default_runtime_read_queries_compile_for_an_external_consumer ... ok
default_runtime_rejects_external_world_and_system_mutation ... FAILED
  forbidden default public mutation compiled: world-mutation
```

즉 read query는 정상 public이지만 external crate의 `GameWorld::set_player_pos`가 실제 compiler에서 허용됨을 재현했다. 다음 system-mutation case는 첫 failure 뒤 실행이 중단됐으므로 구현 후 두 case를 모두 GREEN으로 확인한다.

## 4. 구현과 표적 GREEN

### 4.1 Document-wide authority

- `validate_current_authority`의 current marker를 report 30으로 올리고 report 29/28 current phrase를 stale set에 포함했다.
- README 5 locale, summary, ADR-0040, documentation audit 10.17, roadmap 두 current block, build/gap final block에 designs header, compatibility header, report 29/30 remediation header를 추가했다.
- compatibility test가 report 28 phrase를 positive로 요구하던 assertion을 report 30 current + report 29 technical evidence로 교체했다.

결과: `r8_documentation` 10 PASS이며 각 신규 active header의 stale mutation이 `Err`다.

### 4.2 Submit-only public Rust surface

- runtime mutating system 9개(combat/death/doors/items/movement/monster_ai/projectiles/stairs/traps)를 `pub(crate)`로 통일하고 root facade에는 public score/vision만 남겼다.
- `GameWorld`의 map/location/identified state mutator를 crate-private로 축소하고 외부 test helper에서만 쓰던 gold/kill/status/player-pos mutator와 미사용 system wrapper를 제거했다.
- compatibility C010 `ResolveDeath`는 `aihack-runtime/testing` opt-in feature helper로 격리했다. root compatibility host만 feature를 활성화하고 TUI/headless manifest/source는 feature/helper/low-level import를 금지한다.
- runtime integration movement/vision/world test는 public `GameSession`·read-only query로 전환하고 direct low-level contract test 4개는 이미 존재하는 production/unit 회귀로 대체했다.

```text
default_runtime_read_queries_compile_for_an_external_consumer ... ok
default_runtime_rejects_external_world_and_system_mutation ... ok
combat 11 / NH367 10 / runtime all-target / TUI 27 ... PASS
```

external default crate는 `GameWorld::set_player_pos`에서 `no method named`, `systems::movement`에서 private module compile error를 반환한다. read-only world/entity query, score와 vision은 compile-pass다.

`cargo tree --locked -p aihack-tui -e features`와 headless 결과에서 `aihack-runtime feature "testing"` match는 각각 0개였다.

## 5. Report 29 기술 회귀와 전체 로컬 gate

clean-worktree actual bundle을 제외한 전체 gate를 실행했다.

| 명령 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --locked` | PASS |
| `cargo test --workspace --all-targets --locked -- --list` | Windows named test **452개** |
| `cargo build --workspace --release --all-targets --locked` | PASS |
| `cargo metadata --locked --format-version 1` | packages/nodes **318/318** |
| `cargo audit` | PASS, 318 dependencies, vulnerabilities 0 |
| `cargo deny --version` | `cargo-deny 0.19.4` |
| `cargo deny check licenses bans sources` | PASS |
| Git Bash `scripts/r7_checkpoint.sh` | PASS |
| Git Bash `scripts/r8_checkpoint.sh` | PASS |

Report 29 기술 회귀는 content 13, allocator/transaction 6, document 10, archive 2, Windows bundle 7, NH367 10, runtime all-target와 TUI lib/main/ConPTY/contract 27을 별도 표적 실행해 모두 PASS했다. test 수 감소는 external direct low-level contract 4개를 production `GameSession`/unit/feature-gated compatibility 경계로 교체한 의도된 결과이며 coverage 삭제가 아니다.

`build.bat --release`는 dirty worktree를 의도적으로 거부하므로 자체 review와 implementation commit 뒤 clean 동일 SHA에서 실행한다.

## 6. clean same-SHA Ubuntu/Windows actual bundle

아직 실행하지 않았다. implementation/local evidence를 clean commit으로 고정하고 같은 SHA의 Ubuntu/Windows actual platform bundle job이 모두 success해야 한다.

## 7. 잔여 경계

- report 30 후속 독립 재감사 전에는 finding을 `Closed`로 올리지 않는다.
- report 29 technical evidence는 유효하지만 report 30 authority/API closure를 대신하지 않는다.
- actual provider, Windows Terminal GUI, physical key-hold, signing/attestation과 외부 게시 승인은 이번 기술 시정 범위 밖이다.
