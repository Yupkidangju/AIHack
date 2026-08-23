# AIHack R7 License Deferral Decision Addendum 15

감사 기준: `AI_AUDIT_DOC_STANDARD.md`

문서 유형: `audit_report_14.md` 이후 사용자 위험 수용 및 gate 이관 결정 보충 보고서

결정 일자: 2026-07-18 (Asia/Seoul)

기준 commit: `eb62984` (`main`, `origin/main`) + 현재 R7 working tree

코드·설정 수정: 없음

문서 수정: `spec.md`, `IMPLEMENTATION_SUMMARY.md`, `GAP_CLOSURE_ROADMAP.md`, `audit_roadmap.md`, `PROVENANCE.md`, `DESIGN_DECISIONS.md`, `BUILD_GUIDE.md`, `README.md`, `CHANGELOG.md`, `docs/R7_COMPATIBILITY_REPORT.md`, `docs/compatibility/README.md`

## 1. Decision Summary

최종 판정: **PASS WITH KNOWN RISKS — R7 engineering complete; license review deferred to R8 pre-launch gate**

사용자는 NGPL/Apache 적용 범위, 손상된 legacy NGPL 사본, PROV-0004와 NH367 scenario의 actual approval 미완료를 현재 개발 단계의 blocker로 유지하지 않고, 실제 런칭 개시 전 최종 검토사항으로 이관하기로 결정했다.

이 결정은 다음을 의미한다.

- R7 engineering 단계와 후속 비배포 개발은 진행할 수 있다.
- `audit_report_14.md`까지 Verified된 코드·테스트·보안 결과는 그대로 유지한다.
- SC-LICENSE-01, content/scenario actual approval, root distribution license와 notice는 R8 실제 런칭 전 필수 gate다.
- 현재 `Reviewed`, root `UNLICENSED`, `publish = false`와 외부 배포 차단은 유지한다.
- 라이선스 위반 없음, 법률 적합성 또는 외부 배포 가능성을 선언하지 않는다.

따라서 보고서 14의 기술 사실과 finding closure는 변경하지 않고, `IMP-F012`의 gate impact만 R7 HOLD에서 R8 launch blocker로 이관한다.

## 2. Authority and Scope

- 결정 주체: 사용자 / project owner direction
- 수용 범위: 로컬 개발, 테스트, R8 비배포 준비 진행
- 수용하지 않은 범위: 외부 artifact 생성·게시, 배포, release, 라이선스 적합성 선언
- 기술 감사 권한: checksum, 포함 경로, gate 동작과 문서 정합성 검증
- 별도 필요 권한: 실제 저작권·파생물·배포 라이선스 판단을 수행할 project owner 또는 적격 검토자

## 3. Supersede Relationship

이 보고서는 `audit_report_14.md`의 다음 항목만 supersede한다.

| 보고서 14 항목 | 보고서 15 결정 |
| --- | --- |
| R7 최종 판정 `HOLD` | **PASS WITH KNOWN RISKS** |
| IMP-F012의 R7 gate impact | R8 실제 런칭 gate로 defer |
| R7 다음 단계 | R8 비배포 준비 진행 가능 |

다음 항목은 supersede하지 않는다.

- PROV-0004와 NH367 10개의 actual approval가 없다는 사실
- SC-LICENSE-01 미충족
- `scripts/r7_checkpoint.sh` HOLD/exit 1
- 외부 배포 BLOCKED
- R8, SC-BUILD-02와 전체 release audit 미완료
- 보고서 12~14의 기술 evidence와 Verified finding 계보

## 4. Accepted Risk

### [AR-F001] content/scenario license approval를 R8 실제 런칭 전까지 defer

- Category: Accepted Risk / Deferred Release Gate
- Related Finding: IMP-F012, XPF-F008
- Inherent Severity: **Major**
- Current Development Impact: **Accepted for non-distribution work**
- Release Impact: **Blocking**
- Status: **Accepted until R8 pre-launch review**
- Evidence:
  - legacy tree에 Apache/NGPL 문서가 혼재하고 적용 scope가 명확하지 않다.
  - local `LICENSE.NGPL` 33..35행은 손상되어 정식 notice 증거로 사용할 수 없다.
  - PROV-0004 runtime content와 NH367-C001..C010은 `Reviewed`이며 actual approval field가 비어 있다.
  - runtime의 Unknown/Blocked direct include는 0건이고 fail-closed validator와 checksum evidence는 구현됐다.
- Rationale: 기술 구현·격리·검증은 완료됐고 위험은 외부 배포 시점에 집중된다. 개발 진행과 release authority를 분리한다.
- Guardrails:
  - `UNLICENSED`, `publish = false`, 외부 배포 BLOCKED 유지
  - 미승인 항목을 `Approved`로 변경하지 않음
  - legacy code/data/string direct import 금지
  - validator의 HOLD를 제거하거나 우회하지 않음
- Expiration/Review Trigger:
  - R8 final release audit 시작
  - 실제 런칭 일정 확정
  - 외부 artifact 생성·게시 요청
  - `Cargo.toml` license 또는 `publish` 변경
  - PROV-0004/NH367 자산 교체·승인 상태 변경
- Required Closure: 적격 검토자의 license/scope/notice/evidence 결정과 SC-LICENSE-01 PASS. 승인 불가 항목은 Blocked/교체한다.
- Owner: Project owner / qualified license reviewer

## 5. Documentation Alignment

| 문서 | 반영 내용 |
| --- | --- |
| `spec.md` | SC-LICENSE-01을 R8 pre-launch gate로 명시하고 R7 known-risk 이관 허용 |
| `IMPLEMENTATION_SUMMARY.md` | R7 `PASS WITH KNOWN RISKS`, R8 next step |
| `GAP_CLOSURE_ROADMAP.md` | G-COMPAT Closed, G-LICENSE R8 final gate |
| `audit_roadmap.md` | R7 engineering pass와 validator HOLD를 분리, R8 closure 조건 명시 |
| `PROVENANCE.md` | Reviewed/UNLICENSED/BLOCKED 유지와 R8 검토 트리거 명시 |
| `DESIGN_DECISIONS.md` | ADR-0029로 사용자 결정, 대안, consequence 기록 |
| `BUILD_GUIDE.md` | validator HOLD는 R8 pending evidence이며 외부 배포 차단임을 명시 |
| R7/compatibility docs | engineering PASS와 release count/approval 미완료 분리 |
| README/CHANGELOG | 현재 상태와 결정 변경 공개 |

## 6. Verification

문서 변경 후 실행:

| 명령/검사 | 결과 |
| --- | --- |
| `cargo test -p aihack --locked --test build_contract --test provenance_manifest` | PASS, 20 tests |
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |
| active docs의 `PASS WITH KNOWN RISKS` 정합성 검색 | PASS |
| `PROVENANCE.md` 외부 배포 BLOCKED 유지 | PASS |
| `scripts/r7_checkpoint.sh` | 예상된 HOLD, exit 1; PROV-0004 + scenario 10개 pending |

코드 테스트·빌드의 전체 증거는 같은 working tree에서 직전에 수행한 `audit_report_14.md`의 표적 42개, 전체 322개, clippy/build/RustSec/cargo-deny PASS를 계승한다. 이번 결정 반영은 문서만 변경했다.

## 7. Required Final Review Before Launch

다음 항목은 실제 런칭 전에 모두 완료해야 한다.

1. PROV-0004 content data의 저작권·파생물·배포 범위 결정
2. NH367-C001..C010의 license/scope/notice/evidence 승인
3. 손상된 legacy NGPL 사본을 정식 notice로 사용하지 않음 확인
4. 승인 불가 자산의 Blocked 처리 또는 독립 자산 교체
5. 승인된 root distribution license와 notice를 `Cargo.toml` 및 release 문서에 반영
6. `scripts/r7_checkpoint.sh`와 SC-LICENSE-01 PASS
7. R8 전체 build/test/security/document/release audit PASS
8. 외부 배포 전 인간 또는 독립 교차감사

## 8. Final Decision

**PASS WITH KNOWN RISKS — R7 engineering complete; license review deferred to R8 pre-launch gate**

| Gate | 상태 |
| --- | --- |
| R7 engineering implementation | PASS |
| R7 technical findings through report 14 | Verified |
| R7 stage disposition | **PASS WITH KNOWN RISKS** |
| SC-COMPAT-01 engineering evidence | PASS |
| IMP-F012 actual approval | Deferred to R8 |
| SC-LICENSE-01 | Pending / launch blocker |
| Root distribution license | `UNLICENSED`, launch blocker |
| External distribution | **BLOCKED** |
| R8/final release | NOT RUN |

이 보고서는 개발 진행을 허용하지만 외부 배포를 허가하지 않는다. 라이선스 관련 미비점은 삭제하거나 승인된 것으로 간주하지 않고 R8 실제 런칭 전 최종 검토사항으로 보존한다.
