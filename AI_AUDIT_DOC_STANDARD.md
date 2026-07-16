# AI Audit Documentation Standard
# AI 감사 문서 표준

## Fast Use Guide

- 새 기능 감사: Sections 4, 6-10, 14-16, 20-21
- 버그 수정 재감사: Sections 6-9, 11, 16-17, 관련 Pattern
- 보안 변경 감사: Sections 8-9, 12-13, 16, 18, SEC Patterns
- 배포 전 최종 감사: Sections 3-9, 13, 16-18, 전체 Pattern Index

## 0. Purpose

이 문서는 AI가 생성하거나 수정한 프로젝트 산출물을 감사하기 위한 단일 표준 참고 문서다. 코드 리뷰 조언 모음이 아니라, 문서, 소스, 테스트, 설정, 빌드 산출물, 보안 경계, 감사 기록을 증거 기반으로 정렬하기 위한 기준이다.

목표는 AI 생성 코드가 완벽하다고 주장하는 것이 아니다. 목표는 구현이 문서화되고 반복 가능하며 증거 기반인 검증 절차를 통과했는지 판정하는 것이다.

감사자는 이 문서를 기준으로 다음을 수행한다.

- 문서와 구현의 정합성 판정
- 디버그 가능성, 빌드 재현성, 테스트 품질 판정
- 보안 경계와 공격 표면 판정
- 문서-소스 양방향 동기화 판정
- 역방향 문서 복구 가능성 판정
- 재감사와 Phase gate 판정
- 최종 수동 전체감사 지원

## 1. Cooperative Audit Philosophy

감사는 적대적 검열이 아니다. 감사는 구현 AI, 설계 AI, 감사 AI, 인간 운영자가 같은 프로젝트 완성을 위해 정렬되는 협력형 품질 게이트다.

- 감사자는 비난하지 않고 수정 가능한 finding을 작성한다.
- 구현자는 감사 결과를 방어하지 않고 문서, 코드, 테스트, 실행 결과에 근거해 정렬한다.
- 불일치가 발견되면 주장 경쟁을 하지 않고 증거 기반 분류를 수행한다.
- 의도가 불명확하면 추측하지 않고 `Needs Spec Clarification`으로 돌린다.
- 감사 루프의 목적은 책임 추궁이 아니라 불확실성 감소다.
- finding은 “틀렸다”가 아니라 “무엇을 어떤 증거로 다시 맞춰야 하는가”를 설명해야 한다.

## 2. Required Inputs

감사 AI는 가능한 범위에서 다음 입력물을 확인한다. 없는 문서는 `Excluded Scope` 또는 `Missing Inputs`에 기록한다.

| Input | Audit Use |
| --- | --- |
| `spec.md` | 요구사항, 범위, 보안 요구, acceptance 기준 |
| `designs.md` | UX, 아키텍처, deferred behavior, 설계 의도 |
| `implementation_summary.md` | 구현 주장, 파일 책임표, 완료 범위 |
| `DESIGN_DECISIONS.md` | 결정 이력, trade-off, authority |
| ADR 문서 | 장기 설계 결정과 supersede 관계 |
| `BUILD_GUIDE.md` | 빌드, 패키징, 산출물, 실행 경로 |
| `audit_roadmap.md` | 감사 계획, Phase gate, known risks |
| `CHANGELOG.md` | 변경 주장과 릴리즈 범위 |
| `README.md` | 사용자-facing 기능, 명령, 운영 가이드 |
| `LESSONS_LEARNED.md` | 과거 실패와 회귀 위험 |
| `kanban_board.md` | 작업 상태, 미완료 항목 |
| Source code | 실제 구현, 호출 경로, runtime defaults |
| Tests | 결정적 검증, 회귀 테스트, 보안 테스트 |
| Config files | env, runtime config, feature flag, launch mode |
| Dependency manifests | package version, optional/lazy/override drift |
| CI/CD files | 품질 게이트, scanner, release path |
| Security documents | hard boundary, heuristic, accepted risk |
| Environment examples | secret defaults, bind address, local/remote mode |

## 3. Required Outputs

감사 결과물은 하나의 통합 리포트로 작성하되, 내부 finding은 3-pass 결과로 분리한다.

필수 포함 항목:

- 감사 요약
- 감사 범위
- 제외 범위
- 확인한 문서
- 확인한 파일
- 검사한 케이스
- 제외한 케이스
- pass별 finding 목록
- cross-pass conflict 목록
- `Status`
- `Severity`
- evidence
- expected / actual
- suggested fix
- re-audit method
- accepted risks
- `Needs Spec Clarification` 항목
- 판단 근거
- 남은 리스크
- 재감사 조건
- final decision

권장 최종 리포트 구조:

```md
# D3D Audit Report

## 1. Audit Scope
## 2. Excluded Scope
## 3. Pass 1: Implementation Compliance Findings
## 4. Pass 2: Debug / Engineering Quality Findings
## 5. Pass 3: Security Findings
## 6. Cross-Pass Conflicts
## 7. Required Fixes Before PASS
## 8. Accepted Risks
## 9. Needs Spec Clarification
## 10. Re-audit Checklist
## 11. Final Decision
```

## 4. Audit Execution Model: 3-Pass Audit

감사 AI는 `AI_AUDIT_DOC_STANDARD.md` 하나를 참조한다. 그러나 실제 감사는 3개의 독립 pass로 수행한다.

```md
Reference Document:
- AI_AUDIT_DOC_STANDARD.md

Audit Execution:
- Pass 1: Implementation Compliance Audit
- Pass 2: Debug / Engineering Quality Audit
- Pass 3: Security Audit

Final Output:
- One integrated audit report
```

각 pass는 독립적인 목적함수를 가진다. 최종 리포트는 하나로 통합하되 pass별 결과를 분리해 기록한다.

| Pass | Primary Question | Primary Inputs | Primary Output |
| --- | --- | --- | --- |
| Pass 1: Implementation Compliance | 문서대로 구현됐는가? | spec, design, implementation summary, source, tests | 문서-구현 drift, Phase gate, reverse documentation 판단 |
| Pass 2: Debug / Engineering Quality | 코드 자체가 안정적이고 재현 가능한가? | source, tests, build docs, configs, dependencies, CI | 빌드/실행/테스트/아키텍처 finding |
| Pass 3: Security | 공격 표면과 보안 경계가 안전한가? | security docs, auth code, config, network, filesystem, shell, scanner | 보안 finding과 gate 조건 |

Pass별 우선 질문:

- Pass 1: 문서의 기능, 타입, API, 상태, UI, 보안 요구사항이 실제 코드와 테스트에 구현됐는가?
- Pass 2: 부팅, 빌드, dependency, runtime config, 테스트, 복잡 상태 로직이 결정적으로 검증 가능한가?
- Pass 3: secret, 인증/권한, 입력, 파일/경로/shell, 네트워크 노출, CORS/WebView/Electron/Tauri/Agent 경계가 안전한가?

## 5. Audit Scope Inventory Rules

감사 시작 시 범위 인벤토리를 먼저 작성한다. 범위가 불명확하면 finding보다 `Needs Spec Clarification`이 먼저다.

필수 기록:

- 프로젝트 경로와 프로젝트 유형
- 언어, 프레임워크, 런타임
- 주요 문서 목록과 누락 문서
- source directories
- test directories
- config files
- dependency manifests
- CI/CD files
- security documents
- build/run documents
- excluded scope
- generated/vendor/reference-only tree
- 감사에 사용한 실행 명령과 실패 명령

제외 범위는 조용히 생략하지 않는다. `node_modules`, `dist`, `target`, `.git`, reference corpus, generated output, permission-denied path는 제외 이유를 기록한다.

## 6. Finding Format

모든 finding은 수정 가능한 단위로 작성한다. 한 finding 안에 서로 다른 pass의 문제를 섞지 않는다. 단, cross-pass conflict는 별도 finding으로 연결한다.

```md
## [Finding ID] 제목

- Pass:
  - Implementation / Debug / Security / Cross-Pass
- Pattern:
- Area:
- Severity:
- Status:
- Summary:
- Evidence:
- Expected:
- Actual:
- Impact:
- Suggested Fix:
- Re-audit Method:
- Owner:
  - Architect / Coder / Auditor / Human
- Notes:
```

Finding ID 권장 형식:

- `IMP-F001`
- `DBG-F001`
- `SEC-F001`
- `XPF-F001`
- 재감사는 원 finding ID를 유지하고 `Re-audit #1`, `Re-audit #2`를 붙인다.

## 7. Status Rules

| Status | Meaning | Use Condition |
| --- | --- | --- |
| `Verified` | 감사 기준을 통과했다 | 문서, 코드, 테스트, 실행 증거가 같은 결론을 지지한다 |
| `Needs Fix` | 코드, 테스트, 설정, 빌드, 보안 경계 수정이 필요하다 | 요구사항은 명확하지만 실제 산출물이 불일치한다 |
| `Needs Documentation Recovery` | 코드에 존재하는 의도를 문서에 복구해야 한다 | 두 개 이상의 독립 근거가 기존 의도를 지지한다 |
| `Needs Spec Clarification` | 명세가 부족해 판정할 수 없다 | 요구사항, Phase, authority, security boundary가 불명확하다 |
| `Accepted Risk` | 위험을 인지하고 명시적으로 수용한다 | owner, 사유, 만료 조건, 재검토 조건이 기록된다 |
| `Deferred to Later Phase` | 현재 Phase 밖으로 이관한다 | 후속 Phase 문서와 추적 ID가 있다 |
| `Rejected as False Positive` | finding이 증거와 맞지 않는다 | 반증 evidence와 기각 근거가 기록된다 |
| `Hold` | 통과/배포/다음 Phase 진행을 멈춘다 | Critical, 불명확한 hard boundary, scope 충돌, 검증 불능 상태 |

`Accepted Risk`는 침묵의 면제가 아니다. 위험 설명, 영향 범위, 책임자, 재검토 조건이 없으면 `Accepted Risk`가 아니라 `Hold`다.

## 8. Severity Rules

| Severity | Meaning | Default Gate Impact |
| --- | --- | --- |
| `Critical` | 데이터 손상, 원격 제어, 인증 우회, secret 노출, 배포 차단급 실패 | PASS 불가, 즉시 수정 또는 Hold |
| `Major` | 기능/보안/빌드/문서 authority가 크게 어긋남 | 수정 또는 명시적 Accepted Risk 필요 |
| `Minor` | 국소 drift, 낮은 위험의 문서/테스트 누락 | Known Issue 또는 후속 Phase 이관 가능 |
| `Info` | 감사 메모, 개선 여지, 추적 정보 | gate 차단 없음, 후속 작업 참고 |

보안 finding의 Critical 또는 Major 후보:

- 인증/권한 우회
- secret, token, password 평문 노출
- non-loopback 무인증 노출
- command injection
- path traversal
- 임의 파일 접근
- 배포 빌드에 남은 개발용 우회
- permissive CORS와 네트워크 bind 결합
- WebView/Electron/Tauri에서 renderer trust boundary 위반
- 보안 문서의 hard boundary 과대주장

## 9. Phase Gate Rules

Phase 통과 기준:

- `Critical` finding이 남아 있으면 통과 불가다.
- `Major` finding은 수정되거나 명시적 `Accepted Risk`로 처리되어야 한다.
- 보안 hard boundary가 불명확하면 `Hold`다.
- 문서 기준이 불명확하여 구현/감사 판정이 불가능하면 `Needs Spec Clarification`이다.
- 남은 `Minor` finding은 `Known Issues` 또는 후속 Phase로 명시적으로 이관되어야 한다.
- 재감사에서 기존 finding 해소 여부가 원 finding ID와 연결되어야 한다.
- Pass 1이 통과되어도 Pass 3 보안 finding이 남아 있으면 전체 PASS가 아니다.
- Pass 2에서 빌드/의존성 문제가 남아 있으면 Pass 1 구현 정합성 PASS를 배포 가능 상태로 해석하지 않는다.
- Phase 밖 구현은 호출 가능 여부, 문서 승인 여부, 테스트 포함 여부로 분류해야 한다.

최종 판정:

- `PASS`: Critical/Major 없음, 모든 pass의 gate 조건 충족
- `PASS WITH KNOWN RISKS`: 남은 위험이 Accepted Risk로 명시됨
- `HOLD`: 보안/명세/빌드/Phase gate가 불명확하거나 차단급 finding 존재
- `REWORK REQUIRED`: 구조적 재작업 없이는 반복 실패가 예상됨

## 10. Audit Stage 1: Implementation Compliance

Pass 1의 목적은 문서와 구현의 양방향 정합성을 판정하는 것이다.

주요 질문:

- 문서에 정의된 기능, 타입, API, 상태, UI, 보안 요구사항이 실제 코드에 있는가?
- 코드에 존재하는 기능, 타입, API, 상태, UI, 보안 결정이 문서에 기록되어 있는가?
- 현재 Phase 범위와 호출 가능한 구현이 일치하는가?
- 완료 주장에 파일 책임표와 결정적 검증 기준이 있는가?
- 2차 요약 문서가 현재 소스와 충돌하지 않는가?

우선 적용 패턴:

- `IMP-001`
- `IMP-002`
- `IMP-003`
- `IMP-004`
- `DOC-BACKFILL-001`
- `SPEC-GAP-001`

산출물:

- implementation drift finding
- missing documentation finding
- orphan/unauthorized implementation finding
- reverse documentation recovery 판단
- Phase gate 판단

## 11. Audit Stage 2: Debug / Engineering Quality

Pass 2의 목적은 코드가 안정적이고 재현 가능하며 유지보수 가능한지 판정하는 것이다.

주요 질문:

- 부팅 체인이 경로, migration, config, locale, service wiring까지 검증되는가?
- 빌드 가이드와 실제 산출물, runtime path assumption이 일치하는가?
- dependency 선언이 eager/lazy/optional/override 경로에서 일치하는가?
- 테스트는 broad smoke를 넘어 구체적 실패 모드 회귀를 잠그는가?
- 복잡한 상태/AI 로직은 headless, fixture, schema, hash, seed로 재현 가능한가?
- env/runtime config는 단일 choke point를 가지는가?
- 로컬 API 포트와 경로는 문서, backend, frontend가 같은 source of truth를 공유하는가?

우선 적용 패턴:

- `DBG-001`
- `BUILD-001`
- `DEP-001`
- `TEST-001`
- `DBG-002`
- `ARCH-001`
- `ARCH-002`

산출물:

- build/run drift finding
- dependency parity finding
- missing regression finding
- deterministic debug surface finding
- architecture choke point finding

## 12. Audit Stage 3: Security

Pass 3의 목적은 공격 표면과 보안 경계가 문서, 코드, 테스트에서 같은 강도로 강제되는지 판정하는 것이다.

주요 질문:

- secret은 참조형 저장 또는 암호화 저장 경계로 고정되어 있는가?
- 개발용 우회 플래그가 배포 경로에 남지 않도록 코드에서 차단되는가?
- non-loopback bind, browser control, remote mutation 표면은 인증과 allowlist를 강제하는가?
- path, workspace, shell, file mode 경계가 독립 제어군으로 검증되는가?
- 보안 문서가 heuristic과 hard boundary를 구분하는가?
- scanner 결과는 shipped scope와 rule provenance를 함께 기록하는가?
- 로컬 데스크톱 API도 네트워크 서비스로 감사했는가?
- 문서가 `innerHTML` 금지를 선언한다면 renderer도 동일 제약을 강제하는가?

우선 적용 패턴:

- `SEC-001`
- `SEC-002`
- `SEC-003`
- `SEC-004`
- `SEC-005`
- `SEC-006`
- `SEC-007`
- `SEC-008`

산출물:

- security boundary finding
- network exposure finding
- secret storage finding
- dev bypass finding
- scanner/supply-chain finding
- security-document overclaim finding

## 13. Cross-Pass Conflict Resolution

서로 다른 pass에서 상충하는 판단이 나올 수 있다. 상충 판단은 하나를 폐기하지 않고 최종 리포트의 `Cross-Pass Conflicts`에 기록한다.

예시:

```md
Implementation Pass:
로컬 API는 문서상 의도된 기능이다.

Security Pass:
그러나 0.0.0.0 bind + permissive CORS + 무인증이면 위험하다.
```

해결 규칙:

- 문서상 의도된 구현이라도 보안 경계가 약하면 `Needs Fix` 또는 `Hold`로 판정한다.
- 보안상 위험한 구현이 문서에 명시되어 있으면 문서 자체도 수정 대상이다.
- Implementation pass의 PASS가 Security pass의 PASS를 보장하지 않는다.
- Security pass의 finding은 구현 정합성과 별개로 독립적인 gate 조건이다.
- Debug pass의 빌드 실패는 Implementation pass의 “기능 있음” 판단을 배포 가능 판단으로 바꾸지 못하게 한다.
- Cross-pass conflict는 영향을 받은 finding ID와 함께 기록한다.

Cross-pass finding 최소 형식:

```md
## [XPF-F001] 제목

- Related Findings: IMP-F001, SEC-F002
- Conflict:
- Resolution:
- Gate Impact:
- Required Fix Before PASS:
```

## 14. Bidirectional Document-Code Sync

문서-소스 정합성은 두 방향으로 감사한다.

### Forward Sync

문서에 정의된 기능, 타입, API, 상태, UI, 보안 요구사항이 실제 코드에 구현되었는지 확인한다.

Forward Sync finding 후보:

- 문서에는 기능 완료로 되어 있으나 코드가 없다.
- 문서에는 UI 완료로 되어 있으나 도메인 테스트만 있다.
- 문서에는 배포 산출물이 포함된다고 하나 package artifact에 없다.
- 문서에는 보안 hard boundary가 있다고 하나 코드에서는 warning 또는 heuristic뿐이다.

### Backward Sync

코드에 존재하는 기능, 타입, API, 상태, UI, 보안 결정이 문서에 기록되어 있는지 확인한다.

코드에만 존재하는 구현은 즉시 오류로 단정하지 않고 다음 중 하나로 분류한다.

| Classification | Meaning | Typical Status |
| --- | --- | --- |
| `Intentional but Undocumented` | 기존 결정의 하위 구현으로 보이나 문서 누락 | `Needs Documentation Recovery` |
| `Accidental / Orphan Code` | 호출되지 않거나 현 범위와 무관한 잔재 | `Needs Fix` |
| `Design Drift` | 코드가 기존 설계와 다른 방향으로 이동 | `Needs Fix` 또는 `Needs Spec Clarification` |
| `Spec Gap` | 기능 밀도는 높지만 통제 문서가 부족 | `Needs Spec Clarification` |
| `Unauthorized Scope Expansion` | 현재 Phase 또는 승인 범위를 넘어 호출 가능 | `Hold` 또는 `Needs Fix` |

## 15. Reverse Documentation Sync

역방향 문서 복구는 코드에 존재하는 사실을 문서로 되돌리는 절차다. 모든 코드 전용 사실을 문서화하지 않는다.

허용 조건:

- 소스, 주석, 테스트, 기존 ADR, `spec.md`, `designs.md` 중 둘 이상의 독립 근거가 같은 의도를 지지한다.
- 새 요구사항을 창작하지 않는다.
- 기존 결정의 하위 구현으로 추적 가능하다.
- 용어, 범위, Phase, 테스트가 기존 문서 체계와 충돌하지 않는다.
- 복구 문서는 “이미 구현된 사실”과 “새로 결정해야 하는 요구사항”을 분리한다.

금지 조건:

- 단일 코드 조각만으로 의도를 확정한다.
- 테스트 fixture만 보고 제품 요구사항을 창작한다.
- 오래된 summary와 충돌하는 현재 코드를 근거 없이 최신 truth로 승격한다.
- 보안상 위험한 구현을 문서화했다는 이유만으로 정당화한다.

의도가 불명확하면 `Needs Spec Clarification`으로 돌린다.

## 16. Re-audit Rules

재감사는 수정된 파일만 보는 절차가 아니다. 관련 문서, 테스트, 설정, 빌드 경로, 보안 경계까지 다시 확인한다.

공통 규칙:

- 기존 finding이 실제로 해소되었는지 확인한다.
- 수정이 새 drift를 만들지 않았는지 확인한다.
- 보안 finding은 동일 취약 패턴이 다른 경로에 남아 있는지 확인한다.
- 문서 복구 finding은 문서가 소스와 기존 결정 체계에 맞게 추가되었는지 확인한다.
- 재감사 결과는 이전 finding ID와 연결한다.
- 3-pass 전체를 매번 반복할 필요는 없지만, 수정된 영역과 연결된 pass는 반드시 다시 수행한다.

재감사 매핑:

| Fixed Finding Type | Required Re-audit |
| --- | --- |
| 문서-구현 finding | Pass 1 필수, 빌드/테스트 영향 시 Pass 2 부분, 보안 경계 영향 시 Pass 3 부분 |
| Debug/build finding | Pass 2 필수, 문서 명령 변경 시 Pass 1 부분, 배포/보안 설정 변경 시 Pass 3 부분 |
| Security finding | Pass 3 필수, 설정/빌드/배포 명령 변경 시 Pass 2 부분, 보안 문서 변경 시 Pass 1 부분 |
| Documentation recovery finding | Pass 1 필수, 복구된 문서가 테스트/보안 경계를 바꾸면 연결 pass 부분 |

재감사 finding에는 다음을 남긴다.

- 원 finding ID
- 수정된 문서/파일
- 재실행한 테스트/명령
- 통과 증거
- 남은 리스크
- 새로 생긴 finding

## 17. Repeated Failure Diagnosis

같은 finding 또는 같은 유형의 finding이 반복되면 단순 재작업을 중단하고 집중진단으로 전환한다.

반복 실패 분류:

- 문서가 모호함
- 구현 구조가 잘못됨
- 테스트가 부족함
- 감사 기준이 충돌함
- 보안 경계가 설계되지 않음
- AI 수정 지시가 너무 넓음
- Phase 범위가 과도함

집중진단 결과:

| Decision | Use Condition |
| --- | --- |
| `Continue` | finding 원인이 국소적이고 수정 방향이 명확하다 |
| `Refactor` | 구조는 유지하되 책임 분리가 필요하다 |
| `Rewrite Module` | 현재 구조가 요구사항을 반복적으로 위반한다 |
| `Revise Spec` | 구현보다 명세가 먼저 바뀌어야 한다 |
| `Split Phase` | Phase 범위가 너무 커서 검증 단위가 무너진다 |
| `Human Review Required` | 보안, 제품 방향, 법적/운영 위험 판단이 필요하다 |

## 18. Final Whole-Project Audit

중요 프로젝트는 사람이 GPT, Gemini, Claude, Codex 등 복수 모델 또는 복수 CLI를 사용해 최종 교차감사를 수행할 수 있다.

수행 시점:

- 배포 전
- major version 전
- 보안/권한/저장/네트워크 관련 변경 후
- 사용자가 중요 프로젝트로 지정한 경우
- 반복 실패 진단에서 `Human Review Required`가 나온 경우

교차감사 결과 분류:

- `Common Finding`: 둘 이상의 감사자가 같은 문제를 지적
- `Single-Model Finding`: 한 감사자만 지적했으나 증거가 있는 문제
- `Rejected Finding`: 증거로 기각한 문제
- `Accepted Risk`: 위험을 명시적으로 수용한 문제

최종 판정:

- `PASS`
- `PASS WITH KNOWN RISKS`
- `HOLD`
- `REWORK REQUIRED`

최종 수동 전체감사에서는 한 모델의 PASS를 전체 PASS로 간주하지 않는다. 공통 finding과 보안 finding을 우선 gate로 처리한다.

## 19. Project-Type Applicability Matrix

| Project Type | Must-Check Patterns |
| --- | --- |
| Web | `IMP-001`, `BUILD-001`, `ARCH-001`, `SEC-001`, `SEC-008` |
| PWA | `IMP-001`, `BUILD-001`, `TEST-001`, `SEC-001`, `SEC-008` |
| Desktop | `IMP-001`, `BUILD-001`, `ARCH-002`, `SEC-002`, `SEC-007` |
| Electron | `BUILD-001`, `SEC-002`, `SEC-008`, `ARCH-001`, `TEST-001` |
| Tauri | `BUILD-001`, `ARCH-001`, `ARCH-002`, `SEC-001`, `SEC-007` |
| CLI | `BUILD-001`, `DEP-001`, `TEST-001`, `SEC-004`, `DBG-001` |
| Agent | `IMP-003`, `DEP-001`, `SEC-003`, `SEC-004`, `SEC-005`, `SEC-006` |
| Backend | `DBG-001`, `ARCH-001`, `ARCH-002`, `SEC-001`, `SEC-003` |
| Game | `IMP-002`, `IMP-003`, `DBG-002`, `TEST-001`, `BUILD-001` |
| Library | `IMP-003`, `DEP-001`, `TEST-001`, `SPEC-GAP-001` |
| Monorepo | `DEP-001`, `SEC-006`, `BUILD-001`, `IMP-004`, `ARCH-001` |

## 20. Quick Pattern Index

### Implementation Compliance

- `IMP-001`: 계약은 도메인, UI, 배포 산출물로 분리해 정합성 검사
- `IMP-002`: 현재 Phase 범위 밖 구현은 의도 문서와 호출 경로로 분류
- `IMP-003`: 완료 주장에는 파일 책임표와 결정적 검증 기준이 동반되어야 함
- `IMP-004`: 2차 요약 문서는 소스보다 빨리 부패하므로 독립 감사 대상

### Debug / Engineering Quality

- `DBG-001`: 부팅은 기능별이 아니라 체인 전체로 감사
- `DBG-002`: 복잡한 상태/AI 로직은 결정적 디버그 표면으로 잠가야 함
- `ARCH-001`: 환경변수와 런타임 설정은 단일 choke point를 가져야 함
- `ARCH-002`: 로컬 API의 포트와 경로는 문서, 백엔드 기본값, 프론트 호출자가 하나의 진실원을 공유해야 함

### Build / Dependency / Test

- `BUILD-001`: 빌드 가이드와 실제 실행 경로는 산출물 기준으로 대조
- `DEP-001`: 중복 의존성 선언은 eager/lazy/override 경로까지 일치 검증
- `TEST-001`: 광범위 스모크보다 구체적 실패모드 회귀 테스트를 우선 추출

### Security

- `SEC-001`: 비밀정보는 참조형 저장 또는 암호화 저장 경계로 고정
- `SEC-002`: 개발용 우회 플래그와 로컬 기본값은 배포 전 별도 재감사
- `SEC-003`: 비루프백 노출은 인증과 허용목록 없이는 통과 불가
- `SEC-004`: 경로, 워크스페이스, 셸 실행 경계는 별도 제어군으로 감사
- `SEC-005`: 보안 문서는 실제 보호 경계를 과대주장하지 않아야 함
- `SEC-006`: 스캐너 결과는 shipped scope와 규칙 provenance를 함께 검증
- `SEC-007`: 로컬 데스크톱 API도 네트워크 서비스로 취급해 개방 설정을 감사
- `SEC-008`: 문서가 `innerHTML` 금지를 선언하면 렌더러도 동일 제약을 강제해야 함

### Document Recovery / Spec Gap

- `DOC-BACKFILL-001`: 코드 전용 사실은 기존 결정과 테스트가 함께 있을 때만 역문서화
- `SPEC-GAP-001`: 기능 밀도가 높은데 통제 문서가 부족한 프로젝트는 명세 보완 대상으로 격리

## 21. Pattern Catalog

### Implementation Compliance Patterns

#### [IMP-001] 계약은 도메인, UI, 배포 산출물로 분리해 정합성 검사

- Pass:
  - Implementation
- Area: 문서-구현 레이어 정합성
- Applies To: Web, Desktop, Agent, Backend
- Audit Question: 이 요구사항은 도메인 계약, 사용자 노출 UI, 배포 산출물 중 어디까지 닫혀야 하는가?
- Inspection Targets: `spec.md`, `designs.md`, `implementation_summary.md`, runtime entrypoint, package/build scripts, acceptance tests
- PASS Condition: 문서가 닫힌 레이어를 명시하고, 코드와 테스트가 해당 레이어 범위에 맞게 정렬되어 있다.
- Needs Fix Condition: 문서는 shipped behavior처럼 쓰였는데 실제로는 도메인 또는 테스트 레벨까지만 구현되어 있다.
- Needs Spec Clarification Condition: 문서가 도메인 완료와 UI/배포 완료를 구분하지 않아 shipped 여부를 판정할 수 없다.
- Suggested Fix Instruction: 명세를 도메인 완료, UI 완료, 배포 완료로 분리하고 각 레이어의 검증 파일과 산출물 경로를 연결한다.
- Re-audit Method: 문서 레이어 선언, 관련 테스트, 실제 패키지 또는 런타임 경로를 다시 대조한다.
- False Positive Notes: 단계 배포는 허용 가능하지만, defer와 shipped 범위가 문서에 분리되어 있어야 한다.
- Source Evidence Summary: `chitchat`에서 provider capability 계약은 domain/test로 닫혔지만 SPA hiding과 packaged frontend/migration asset은 별도 레이어로 남았다.

#### [IMP-002] 현재 Phase 범위 밖 구현은 의도 문서와 호출 경로로 분류

- Pass:
  - Implementation
- Area: Phase 범위, 호출 가능 구현, scope control
- Applies To: Game, Web, Desktop, CLI
- Audit Question: 이 코드가 현재 Phase에 속하는가, 다음 Phase 선행인가, 문서 누락인가, 잔재 코드인가?
- Inspection Targets: `spec.md`, `audit_roadmap.md`, `implementation_summary.md`, source entrypoints, call graph, tests
- PASS Condition: 코드가 현재 Phase 범위와 일치하거나, 문서상 선반영/후속 Phase 예고가 명시되어 있다.
- Needs Fix Condition: 현재 Phase 비목표나 다음 Phase 기능이 호출 가능한 상태로 섞여 있고 문서 승인 흔적이 없다.
- Needs Spec Clarification Condition: 문서가 현 단계 범위와 선반영 허용 범위를 판정할 기준을 주지 못한다.
- Suggested Fix Instruction: 구현을 `Intentional but Undocumented`, `Accidental / Orphan Code`, `Design Drift`, `Spec Gap`, `Unauthorized Scope Expansion` 중 하나로 분류하고 문서 또는 코드를 맞춘다.
- Re-audit Method: Phase 문서, 호출 경로, 테스트 범위를 다시 대조해 분류가 바뀌었는지 확인한다.
- False Positive Notes: 데이터 스키마 또는 UI 골격 선반영은 호출 불가이고 문서화되어 있으면 허용될 수 있다.
- Source Evidence Summary: `LazyRoomLife`는 early-phase 문서보다 runtime engine/storage/renderer가 앞섰고, `AIHack`은 phase claims와 deterministic tests가 촘촘히 연결되어 대조 사례가 되었다.

#### [IMP-003] 완료 주장에는 파일 책임표와 결정적 검증 기준이 동반되어야 함

- Pass:
  - Implementation
- Area: 완료 주장, verification authority
- Applies To: Game, Agent, Backend, Desktop
- Audit Question: 이 완료 선언을 코드 기준으로 재현할 수 있는 책임표, 테스트, 해시, 스키마, 진단기가 있는가?
- Inspection Targets: `implementation_summary.md`, release gates, deterministic tests, doctor/health checks, changelog
- PASS Condition: 완료 주장마다 대응 파일 책임, 검증 명령, 결정적 기준이 연결되어 있다.
- Needs Fix Condition: 완료 선언은 있으나 파일 책임표 또는 재현 가능한 검증 기준이 없다.
- Needs Spec Clarification Condition: 완료 기준이 정성 문장만 있고 측정 가능한 종료 조건이 없다.
- Suggested Fix Instruction: 완료 주장 옆에 책임 파일, 검증 명령, 기준 fixture 또는 baseline을 명시한다.
- Re-audit Method: 완료 문서와 실제 baseline 테스트 또는 doctor 결과를 재실행해 일치 여부를 본다.
- False Positive Notes: 초기 스캐폴딩도 적어도 종료 조건과 소유 파일은 남겨야 한다.
- Source Evidence Summary: `AIHack`은 phase contracts, release candidate tests, save/load/schema/UI/monster AI tests로 완료 주장을 잠갔고, `openclaw`는 doctor checks로 운영 주장을 뒷받침했다.

#### [IMP-004] 2차 요약 문서는 소스보다 빨리 부패하므로 독립 감사 대상

- Pass:
  - Implementation
- Area: secondary docs, generated docs, stale summary
- Applies To: Web, Desktop, Game, Agent
- Audit Question: 이 보조 문서는 현재 소스와 실행 경로를 반영하는가, 아니면 과거 단계의 상태를 보존한 것인가?
- Inspection Targets: `implementation_summary.md`, generated docs data, source modules, runtime defaults, changelog
- PASS Condition: 2차 요약 문서가 현재 소스와 일치하거나, snapshot 시점과 한계를 명시한다.
- Needs Fix Condition: 2차 요약 문서가 현재 소스와 충돌하면서도 최신 상태처럼 읽힌다.
- Needs Spec Clarification Condition: 어떤 문서가 primary authority이고 어떤 문서가 참고 요약인지 구분이 없다.
- Suggested Fix Instruction: 보조 문서에 기준 시점과 authority를 명시하고, 소스와 충돌하는 항목을 동기화한다.
- Re-audit Method: summary/doc data와 source entrypoints, defaults, feature flags를 다시 대조한다.
- False Positive Notes: 과거 마일스톤 기록 문서는 남을 수 있지만 현재 상태 문서처럼 링크되면 finding 대상이다.
- Source Evidence Summary: `TypeTris` summary는 일부 기능을 미완료로 남겼지만 source는 구현을 포함했고, `smaLLMRust` docs data는 API 포트 drift를 보였다.

### Debug / Engineering Quality Patterns

#### [DBG-001] 부팅은 기능별이 아니라 체인 전체로 감사

- Pass:
  - Debug
- Area: startup chain, initialization order
- Applies To: Web, Desktop, Agent, Backend, CLI
- Audit Question: 앱이 시작될 때 경로, migration, 설정, 번역, service wiring이 어떤 순서로 연결되는가?
- Inspection Targets: runtime entrypoint, config loader, migration code, i18n bootstrap, startup smoke tests
- PASS Condition: 부팅 체인이 문서 또는 코드에서 명확하고, 실패 지점별 테스트 또는 smoke가 존재한다.
- Needs Fix Condition: 초기화 순서가 암묵적이거나 한 단계 실패가 다음 단계에서 늦게 폭발한다.
- Needs Spec Clarification Condition: 문서가 부팅 책임과 순서를 설명하지 않아 어떤 초기화가 필수인지 판정할 수 없다.
- Suggested Fix Instruction: 부팅 단계를 명시적으로 나누고 각 단계의 실패 조건과 smoke 검증을 추가한다.
- Re-audit Method: 빈 데이터 디렉토리, 손상 설정, locale 부재, migration 필요 상태에서 부팅 경로를 다시 확인한다.
- False Positive Notes: 단일 바이너리 CLI도 config/path 초기화 체인이 있으면 적용한다.
- Source Evidence Summary: `chitchat` startup은 app dir, legacy migration, DB migration, preference, translator, service wiring이 연결된 체인으로 관찰되었다.

#### [DBG-002] 복잡한 상태/AI 로직은 결정적 디버그 표면으로 잠가야 함

- Pass:
  - Debug
- Area: deterministic debugging, state machine, AI policy
- Applies To: Game, Agent, Backend, Desktop
- Audit Question: 이 복잡한 로직을 UI 없이 재현할 수 있는 headless, schema, hash, fixture, seed 검증이 있는가?
- Inspection Targets: headless binaries, schema tests, snapshot hashes, fixture loaders, deterministic seeds
- PASS Condition: 핵심 로직을 UI와 분리한 결정적 검증 경로가 있다.
- Needs Fix Condition: 동일 입력에서 결과를 재현할 표면이 없거나 UI 수동 테스트에만 의존한다.
- Needs Spec Clarification Condition: 어떤 상태가 결정적으로 유지되어야 하는지 문서가 말해주지 않는다.
- Suggested Fix Instruction: headless entrypoint, schema fixture, hash baseline, deterministic seed 검증 중 하나 이상을 추가한다.
- Re-audit Method: 동일 fixture와 seed로 결과가 동일한지, 저장/복원 후 invariant가 유지되는지 다시 확인한다.
- False Positive Notes: 외부 호출은 mock 또는 recorded fixture로 치환한 뒤 평가한다.
- Source Evidence Summary: `AIHack`은 headless binary, save/load hash, schema tests, monster AI seed/policy tests로 복잡 로직을 고정했다.

#### [ARCH-001] 환경변수와 런타임 설정은 단일 choke point를 가져야 함

- Pass:
  - Debug
- Area: config architecture, env handling
- Applies To: Web, Desktop, Backend, Agent
- Audit Question: 환경변수와 런타임 설정은 한 모듈에서 해석되는가, 아니면 여러 화면/서비스가 직접 읽는가?
- Inspection Targets: config modules, direct `process.env` 또는 동등 read, test bootstrap, config docs
- PASS Condition: 설정 해석이 집중되어 있고 테스트에서 seam 또는 mock으로 제어 가능하다.
- Needs Fix Condition: 여러 모듈이 직접 env를 읽어 서로 다른 fallback을 사용한다.
- Needs Spec Clarification Condition: 어떤 설정이 build-time이고 runtime인지 문서가 구분하지 않는다.
- Suggested Fix Instruction: 직접 env read를 공용 config 모듈 뒤로 숨기고 테스트 가능한 lookup seam을 제공한다.
- Re-audit Method: direct env read 검색과 config module 호출 경로를 다시 대조한다.
- False Positive Notes: 아주 작은 스크립트는 예외가 가능하지만 다중 런타임 표면에서는 drift가 빠르게 누적된다.
- Source Evidence Summary: `openhuman`은 frontend config와 Rust config loader에서 runtime-authoritative config와 test seam을 분리했다.

#### [ARCH-002] 로컬 API의 포트와 경로는 문서, 백엔드 기본값, 프론트 호출자가 하나의 진실원을 공유해야 함

- Pass:
  - Debug
- Area: local API route/source of truth
- Applies To: Desktop, Agent, Backend, Web
- Audit Question: API 주소, 포트, WebSocket 경로의 canonical source는 어디이며, 모든 호출자가 그것을 참조하는가?
- Inspection Targets: API docs, docs data generators, backend router registration, frontend URL helpers, config defaults
- PASS Condition: 포트와 경로가 단일 설정 또는 generator에서 파생되고, 문서와 호출자가 동일 값을 사용한다.
- Needs Fix Condition: 문서와 호출자 또는 handler와 router registration이 서로 다른 주소를 사용한다.
- Needs Spec Clarification Condition: 어떤 실행 모드가 canonical API surface인지 문서가 정하지 않는다.
- Suggested Fix Instruction: API base URL과 route map을 단일 config/source 모듈로 모으고 문서도 그 값에서 생성되게 한다.
- Re-audit Method: API docs, backend bind/router, frontend helpers, smoke request를 다시 대조한다.
- False Positive Notes: 개발/배포 포트 분리는 가능하지만 모드별 source of truth와 전환 규칙이 필요하다.
- Source Evidence Summary: `smaLLMRust`에서 docs는 `3001`, backend/frontend는 `8080` 중심으로 갈라졌고 WebSocket registration drift 위험도 관찰되었다.

### Security Patterns

#### [SEC-001] 비밀정보는 참조형 저장 또는 암호화 저장 경계로 고정

- Pass:
  - Security
- Area: secret storage, config persistence
- Applies To: Web, Desktop, Agent, Backend
- Audit Question: 실제 비밀값은 어디에 저장되고, 애플리케이션 데이터에는 어떤 참조나 암호문만 남는가?
- Inspection Targets: spec/security docs, secret-storage module, config persistence code, DB models, tests
- PASS Condition: 문서, 코드, 테스트가 동일한 secret boundary를 설명하고 평문 비밀값이 모델/DB에 남지 않는다.
- Needs Fix Condition: 비밀값이 평문으로 저장되거나 문서와 코드가 다른 저장 경계를 가정한다.
- Needs Spec Clarification Condition: 어떤 저장소가 secret source of truth인지 문서가 불명확하다.
- Suggested Fix Instruction: 비밀값 저장 경계를 단일 정책으로 고정하고 참조형 저장 또는 암호화 저장 테스트를 추가한다.
- Re-audit Method: DB/config 파일, secret module, 관련 테스트 assertions를 다시 확인한다.
- False Positive Notes: 로컬 단일 사용자 앱도 secret boundary 검증이 필요하며 “로컬이라 안전”은 통과 근거가 아니다.
- Source Evidence Summary: `chitchat`은 `secret_ref`와 OS keyring 테스트를 사용했고, `openhuman`은 암호화 저장, backup, atomic rename, rollback을 구현했다.

#### [SEC-002] 개발용 우회 플래그와 로컬 기본값은 배포 전 별도 재감사

- Pass:
  - Security
- Area: dev bypass, release mode fence
- Applies To: Desktop, Web, Agent
- Audit Question: 이 우회 플래그나 로컬 기본값은 개발 전용으로 fenced 되었는가, 배포 빌드에서 제거 또는 차단되는가?
- Inspection Targets: manifests, README/build guide caveats, feature flags, CORS config, Electron/Tauri launch scripts
- PASS Condition: 우회 설정이 개발 전용임이 코드와 문서에서 명확하고 배포 표면에서 자동 차단된다.
- Needs Fix Condition: 개발 우회가 배포 경로에도 남아 있거나 경계가 문서에만 있고 코드에서 강제되지 않는다.
- Needs Spec Clarification Condition: 어떤 실행 모드가 개발용이고 어떤 모드가 배포용인지 문서가 구분하지 않는다.
- Suggested Fix Instruction: 개발 전용 플래그를 명시적 feature/build mode 뒤로 숨기고 배포 경로에서는 실패하도록 만든다.
- Re-audit Method: 개발/배포 명령, feature gates, 런타임 경고 또는 차단 로직을 다시 확인한다.
- False Positive Notes: 개발 환경 우회는 존재할 수 있지만 fence와 재감사 절차가 없으면 통과로 보지 않는다.
- Source Evidence Summary: `chitchat` local CORS, `TypeTris` Electron `--no-sandbox`, `openhuman` disabled-by-default destructive E2E feature가 대표 사례다.

#### [SEC-003] 비루프백 노출은 인증과 허용목록 없이는 통과 불가

- Pass:
  - Security
- Area: network bind, remote control, allowlist
- Applies To: Agent, Backend, Desktop with local server
- Audit Question: 루프백 밖으로 bind 하거나 원격 제어 경로를 열 때 인증, allowlist, mutation guard가 강제되는가?
- Inspection Targets: bind logic, server startup guards, auth middleware, allowlist config, tests
- PASS Condition: 비루프백 노출 시 인증과 허용목록이 코드에서 강제되고 테스트로 잠겨 있다.
- Needs Fix Condition: 네트워크 노출이 가능하지만 인증 또는 allowlist가 경고 수준에 그치거나 없다.
- Needs Spec Clarification Condition: 프로젝트가 로컬 전용인지 원격 노출 허용인지 문서가 명확히 말하지 않는다.
- Suggested Fix Instruction: non-loopback bind 시 필수 인증과 allowlist 검사를 추가하고 미설정이면 시작 실패로 전환한다.
- Re-audit Method: bind guard tests, startup path, configuration defaults를 다시 확인한다.
- False Positive Notes: air-gapped local-only 도구도 wildcard bind 또는 browser control 표면이 보이면 우선 적용한다.
- Source Evidence Summary: `hermes-agent`는 non-loopback/API key/allowlist guard를 테스트했고, `openclaw`는 browser control auth와 CSRF-style mutation protection을 구현했다.

#### [SEC-004] 경로, 워크스페이스, 셸 실행 경계는 별도 제어군으로 감사

- Pass:
  - Security
- Area: path traversal, workspace trust, shell execution
- Applies To: Agent, CLI, Desktop, Backend
- Audit Question: 경로 검증, 작업공간 경계, 명령 승인, 파일 권한이 각각 독립적으로 통제되는가?
- Inspection Targets: path validation helpers, command approval modules, workspace traversal tests, file permission tests, security docs
- PASS Condition: 각 제어군이 분리된 코드와 테스트를 가지고 있고, 하나의 실패가 다른 제어군 성공으로 덮이지 않는다.
- Needs Fix Condition: 경로나 셸 실행이 ad hoc string check에만 의존하거나 테스트가 특정 제어군을 직접 잠그지 않는다.
- Needs Spec Clarification Condition: 어떤 파일/명령 경계가 신뢰 영역 밖인지 문서가 정의하지 않는다.
- Suggested Fix Instruction: path, workspace, shell, file-mode 검사를 분리된 helper와 회귀 테스트로 분해한다.
- Re-audit Method: traversal, dangerous command, permission regression 테스트를 다시 확인한다.
- False Positive Notes: read-only 도구도 future tool execution 계획이 있으면 선제 적용한다.
- Source Evidence Summary: `hermes-agent`는 path security, approval, worktree traversal rejection, secure file mode tests를 분리했고 `openclaw`는 operator trust와 code guard를 나눴다.

#### [SEC-005] 보안 문서는 실제 보호 경계를 과대주장하지 않아야 함

- Pass:
  - Security
- Area: security documentation, hard boundary, heuristic
- Applies To: Agent, Backend, Desktop
- Audit Question: 문서가 선언한 보안 경계가 실제 코드와 같은 수준의 강제력을 가지는가?
- Inspection Targets: `SECURITY.md`, auth/boundary code, redaction modules, tests, startup warnings
- PASS Condition: 문서가 heuristic, policy, hard boundary를 구분하고 hard boundary는 코드와 테스트가 뒷받침한다.
- Needs Fix Condition: 문서가 in-process heuristic을 sandbox나 isolation처럼 과대표현한다.
- Needs Spec Clarification Condition: 문서가 어떤 보호가 advisory인지 mandatory인지 구분하지 않는다.
- Suggested Fix Instruction: 보안 문서에서 hard boundary와 heuristic을 분리 표기하고 hard boundary마다 대응 코드 위치를 연결한다.
- Re-audit Method: SECURITY 문구와 실제 enforcement code/test를 나란히 다시 확인한다.
- False Positive Notes: README의 마케팅 문구가 운영 보안 가이드처럼 읽히면 감사 대상이다.
- Source Evidence Summary: `hermes-agent`는 OS isolation과 heuristic guard를 분리했고, `openclaw`는 operator-trust policy를 concrete auth/CSRF/redaction code와 연결했다.

#### [SEC-006] 스캐너 결과는 shipped scope와 규칙 provenance를 함께 검증

- Pass:
  - Security
- Area: scanner scope, supply-chain, rule provenance
- Applies To: Monorepo, Backend, Agent, Web
- Audit Question: 이 스캐너 finding은 shipped product scope에 속하는가, 그리고 규칙 출처와 소유자가 추적 가능한가?
- Inspection Targets: scanner ignore files, rule metadata checks, pre-commit hooks, CI workflows, finding path
- PASS Condition: shipped scope와 non-shipped scope가 구분되고 규칙 provenance가 검증된다.
- Needs Fix Condition: 제품 외 경로가 동일 위험도로 집계되거나 규칙 메타데이터가 없어 triage 근거를 잃는다.
- Needs Spec Clarification Condition: 어떤 경로가 배포 범위인지 문서와 스캐너 설정이 말해주지 않는다.
- Suggested Fix Instruction: scanner scope 파일과 rule metadata 검증을 추가하고 finding 리포트에 shipped 여부를 포함한다.
- Re-audit Method: ignore scope, metadata checks, sample finding triage 흐름을 다시 확인한다.
- False Positive Notes: QA-only 코드도 냄새를 가질 수 있지만 제품 취약점과 같은 심각도로 집계하지 않는다.
- Source Evidence Summary: `openclaw`는 `.semgrepignore`, OpenGrep rule metadata checks, pre-commit, GitHub workflows로 scanner hygiene를 구성했다.

#### [SEC-007] 로컬 데스크톱 API도 네트워크 서비스로 취급해 개방 설정을 감사

- Pass:
  - Security
- Area: desktop local API, CORS, private network access
- Applies To: Desktop, Agent, Backend
- Audit Question: 이 로컬 API는 실제로 어떤 인터페이스에 bind 하며, CORS와 인증 기본값이 원격 접근을 얼마나 허용하는가?
- Inspection Targets: backend bind address, CORS middleware, auth bootstrap, API key defaults, local API docs
- PASS Condition: 로컬 API가 loopback 중심으로 제한되거나, 비루프백/무인증/광범위 CORS가 명시적으로 차단된다.
- Needs Fix Condition: 기본 bind가 외부 인터페이스이고 permissive CORS 또는 무인증 접근이 함께 열린다.
- Needs Spec Clarification Condition: 로컬 전용인지 LAN 공개 허용인지 제품 문서가 말해주지 않는다.
- Suggested Fix Instruction: loopback을 기본값으로 바꾸고 원격 공개는 명시적 opt-in과 인증 설정이 없으면 실패하게 한다.
- Re-audit Method: bind address, CORS headers, no-key startup behavior, remote reachability smoke를 다시 확인한다.
- False Positive Notes: 연구용 도구라도 `0.0.0.0`와 permissive CORS가 함께 보이면 위험으로 본다.
- Source Evidence Summary: `smaLLMRust`는 `0.0.0.0`, permissive CORS, private-network header, no-key open access 조합이 관찰되었다.

#### [SEC-008] 문서가 `innerHTML` 금지를 선언하면 렌더러도 동일 제약을 강제해야 함

- Pass:
  - Security
- Area: DOM injection, renderer policy
- Applies To: Web, Desktop WebView, Agent UI
- Audit Question: 문서에서 금지한 DOM 주입 패턴이 실제 렌더러 코드에서 여전히 사용되는가?
- Inspection Targets: spec security section, audit roadmap, renderer/view code, templating helpers
- PASS Condition: 금지된 DOM 삽입 패턴이 제거되었거나 엄격한 sanitizer와 허용 정책으로 감싸져 있다.
- Needs Fix Condition: 문서상 금지인데 코드가 `innerHTML` 또는 동등한 unsanitized DOM 삽입을 계속 사용한다.
- Needs Spec Clarification Condition: 문서는 XSS 경계를 말하지만 허용 가능한 HTML source와 sanitization 정책을 정하지 않는다.
- Suggested Fix Instruction: `textContent`, DOM node construction, 또는 명시적 sanitizer 기반 렌더링으로 교체하고 문서와 맞춘다.
- Re-audit Method: renderer 코드에서 `innerHTML` 사용처를 다시 검색하고 user-controlled data source로 확장되는 경로가 없는지 확인한다.
- False Positive Notes: 현재 내부 생성값이라도 향후 외부 데이터 유입 가능성이 있으면 금지 규칙 위반을 남겨두지 않는다.
- Source Evidence Summary: `LazyRoomLife`는 spec/audit roadmap에서 XSS와 `innerHTML` 제한을 선언했지만 renderer가 `innerHTML`을 사용했다.

### Build / Dependency / Test Patterns

#### [BUILD-001] 빌드 가이드와 실제 실행 경로는 산출물 기준으로 대조

- Pass:
  - Debug
- Area: build guide, artifact path, runtime packaging
- Applies To: Web, Desktop, CLI, Agent
- Audit Question: 문서에 적힌 빌드/실행 명령이 실제 스크립트와 산출물 경로를 정확히 가리키는가?
- Inspection Targets: `README.md`, `BUILD_GUIDE.md`, package manifests, build scripts, artifact path assumptions in runtime code
- PASS Condition: 문서된 명령이 현재 스크립트와 일치하고 산출물 구조가 런타임 경로 가정과 맞는다.
- Needs Fix Condition: README와 BUILD_GUIDE가 서로 다른 절차를 요구하거나 런타임이 산출물에 없는 경로를 기대한다.
- Needs Spec Clarification Condition: 배포 대상이 무엇인지 문서가 불명확해 어떤 빌드 경로를 검증해야 하는지 알 수 없다.
- Suggested Fix Instruction: 빌드 문서를 단일 진실원으로 정리하고 산출물 트리와 런타임 경로 의존성을 함께 명시한다.
- Re-audit Method: 문서된 명령만 사용해 fresh build 후 실제 실행 경로와 자산 포함 여부를 다시 확인한다.
- False Positive Notes: dev/release 명령은 달라도 되지만 차이가 문서에서 분명해야 한다.
- Source Evidence Summary: `chitchat`, `TypeTris`, `smaLLMRust`에서 build docs, scripts, packaged asset/runtime path drift가 반복 관찰되었다.

#### [DEP-001] 중복 의존성 선언은 eager/lazy/override 경로까지 일치 검증

- Pass:
  - Debug
- Area: dependency parity, lazy install, workspace policy
- Applies To: Agent, Backend, Monorepo, Desktop
- Audit Question: 직접 설치, optional extra, lazy installer, workspace override, patch 경로가 같은 버전 정책을 공유하는가?
- Inspection Targets: dependency manifests, lazy-install maps, workspace policy files, packaging tests
- PASS Condition: 중복 선언 간 버전과 정책 의도가 일치하거나 차이가 문서로 정당화되어 있다.
- Needs Fix Condition: 설치 경로별 버전이 드리프트했고 이를 감지하는 테스트나 문서가 없다.
- Needs Spec Clarification Condition: 어떤 설치 경로가 canonical인지 문서가 정하지 않아 parity 기준을 세울 수 없다.
- Suggested Fix Instruction: canonical dependency source를 정하고 나머지 선언을 자동 검증하거나 생성하도록 바꾼다.
- Re-audit Method: manifest diff, lazy-install map, packaging tests를 함께 다시 확인한다.
- False Positive Notes: canary pin 차이는 가능하지만 이유와 적용 범위가 기록되어야 한다.
- Source Evidence Summary: `hermes-agent`는 eager extra와 lazy dependency pin drift가 있었고, `openclaw`는 workspace overrides와 release-age policy를 dependency control로 사용했다.

#### [TEST-001] 광범위 스모크보다 구체적 실패모드 회귀 테스트를 우선 추출

- Pass:
  - Debug
- Area: regression tests, failure-mode evidence
- Applies To: All project types
- Audit Question: 이 저장소는 실제 과거 실패를 이름 붙인 회귀 테스트로 고정하고 있는가?
- Inspection Targets: test file names, failure-specific assertions, changelog, inline rationale comments
- PASS Condition: 테스트가 구체적 실패 모드와 경계를 직접 이름 붙여 보존한다.
- Needs Fix Condition: 중요한 회귀가 broad smoke에만 숨겨져 있고 어떤 위험을 막는지 테스트 이름이나 assertion으로 드러나지 않는다.
- Needs Spec Clarification Condition: 프로젝트가 어떤 실패를 중점 관리하는지 문서나 테스트 이름에서 추적할 수 없다.
- Suggested Fix Instruction: broad smoke를 유지하되 과거 버그나 경계별로 명시적인 회귀 테스트를 추가한다.
- Re-audit Method: 새 회귀 테스트가 실패 모드를 직접 설명하는지와 broad smoke에 흡수되지 않았는지 확인한다.
- False Positive Notes: 초기 프로젝트도 사고 이력이 changelog나 주석에 남아 있으면 회귀 테스트 후보가 된다.
- Source Evidence Summary: `hermes-agent`는 worktree security, SQL injection, file permission, timezone 회귀를 별도 테스트로 보존했고 `AIHack`은 monster AI/save/load/UI geometry를 결정적으로 테스트했다.

### Document Backfill / Spec Gap Patterns

#### [DOC-BACKFILL-001] 코드 전용 사실은 기존 결정과 테스트가 함께 있을 때만 역문서화

- Pass:
  - Implementation
- Area: reverse documentation, code-only fact classification
- Applies To: Web, Agent, Backend, Build Tooling
- Audit Question: 이 구현 사실은 소스, 테스트, 기존 문서 중 둘 이상에서 같은 의도로 교차 확인되는가?
- Inspection Targets: source comments, tests, existing spec/design/README, build config, call sites
- PASS Condition: 두 개 이상의 독립 근거가 같은 의도를 지지하고 역문서화가 기존 방향을 바꾸지 않는다.
- Needs Fix Condition: 구현은 반복 사용되지만 문서 체계 어디에도 기록되지 않아 다음 작업자가 추적하기 어렵다.
- Needs Spec Clarification Condition: 코드 흔적은 있으나 의도와 범위를 두 개 이상 근거로 묶을 수 없다.
- Suggested Fix Instruction: 새 요구사항을 만들지 말고 기존 섹션 아래에 구현 사실과 근거 파일을 역추적해서 보강한다.
- Re-audit Method: 보강된 문서가 기존 용어 체계, 호출 흐름, 테스트 assertions와 일치하는지 다시 본다.
- False Positive Notes: 단일 주석이나 단일 테스트만으로는 의도 확정 근거가 약하다.
- Source Evidence Summary: `chitchat` prompt snapshot fields는 source와 tests와 spec section이 연결되었고, `openclaw` build boundaries는 comments, harness, README guidance가 같은 의도를 지지했다.

#### [SPEC-GAP-001] 기능 밀도가 높은데 통제 문서가 부족한 프로젝트는 명세 보완 대상으로 격리

- Pass:
  - Implementation
- Area: spec gap, control document absence
- Applies To: Desktop, Agent, Backend, Library
- Audit Question: 이 프로젝트에 현재 구현을 묶어주는 동결 명세, 단계 정의, 파일 책임표가 존재하는가?
- Inspection Targets: root docs, source tree breadth, build guide, README feature lists, config files
- PASS Condition: README 외에도 spec/design/implementation control 문서가 존재하거나 README가 그 역할을 구조적으로 대체한다.
- Needs Fix Condition: 기능 목록과 소스 표면은 큰데 감사 기준 문서가 없어 구현의 의미 좌표를 잃는다.
- Needs Spec Clarification Condition: 문서가 일부 있으나 성공 기준, 범위, 책임 경계를 판정할 구조가 없다.
- Suggested Fix Instruction: 범위, 핵심 계약, 파일 책임, 검증 명령을 담은 `spec.md` 또는 동등 문서를 추가한다.
- Re-audit Method: 새 통제 문서가 실제 소스 모듈과 빌드 절차를 포괄하는지 다시 대조한다.
- False Positive Notes: 실험용 프로토타입은 문서가 적을 수 있지만 외부 사용자에게 공개되는 순간 이 패턴이 중요해진다.
- Source Evidence Summary: `smaLLMRust`는 README/BUILD_GUIDE는 있으나 agent, MCP, workflow, plugin, RAG 표면을 묶는 spec/design/implementation control 문서가 부족했다.

## 22. Copy-Paste Instruction Block

### English

```md
You are performing a cooperative D3D audit.

Use `AI_AUDIT_DOC_STANDARD.md` as the single audit reference.

Run the audit in three independent passes:

1. Implementation Compliance
2. Debug / Engineering Quality
3. Security

Do not blame the implementer.
Do not invent requirements.
Use project documents, source code, tests, configs, build artifacts, and runtime evidence.

For every finding, include:
- pass
- pattern
- area
- severity
- status
- evidence
- expected behavior
- actual behavior
- impact
- suggested fix
- re-audit method

If a requirement is unclear, mark it as `Needs Spec Clarification`.
If code exists without documentation, classify it before judging it:
- Intentional but Undocumented
- Accidental / Orphan Code
- Design Drift
- Spec Gap
- Unauthorized Scope Expansion

Produce one integrated audit report with pass-separated findings and cross-pass conflicts.
```

### 한국어

```md
당신은 협력형 D3D 감사를 수행한다.

단일 감사 기준 문서로 `AI_AUDIT_DOC_STANDARD.md`를 사용한다.

감사는 3개의 독립 pass로 수행한다.

1. Implementation Compliance
2. Debug / Engineering Quality
3. Security

구현자를 비난하지 않는다.
요구사항을 창작하지 않는다.
프로젝트 문서, 소스 코드, 테스트, 설정, 빌드 산출물, 런타임 증거를 사용한다.

모든 finding에는 다음을 포함한다.
- pass
- pattern
- area
- severity
- status
- evidence
- expected behavior
- actual behavior
- impact
- suggested fix
- re-audit method

요구사항이 불명확하면 `Needs Spec Clarification`으로 표시한다.
문서에 없는 코드가 있으면 먼저 다음 중 하나로 분류한다.
- Intentional but Undocumented
- Accidental / Orphan Code
- Design Drift
- Spec Gap
- Unauthorized Scope Expansion

최종 결과는 하나의 통합 감사 리포트로 작성하되, finding은 pass별로 분리하고 cross-pass conflict를 별도 기록한다.
```

## 23. Final Principle

감사의 최종 원칙은 “누가 맞는가”가 아니라 “무엇이 어떤 증거로 검증되었고, 무엇이 아직 불확실한가”를 명확히 하는 것이다.

문서가 코드보다 강하면 코드를 맞춘다. 코드가 문서보다 앞서 있으면 근거를 분류한다. 보안 경계가 불명확하면 통과시키지 않는다. 반복 실패가 보이면 더 세게 밀어붙이지 말고 기준, 구조, 테스트, Phase를 재설계한다.

협력형 감사는 처벌 루프가 아니라 불확실성 감소 루프다.
