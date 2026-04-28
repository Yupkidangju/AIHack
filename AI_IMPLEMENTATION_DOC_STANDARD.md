# AI Implementation Documentation Standard
## AI 구현 문서 표준

## 0. Purpose

이 문서는 AI가 기획, 스펙, 설계, 구현 가이드, 로드맵 문서를 만들 때 따라야 하는 기준 문서다.
목표는 "읽기 좋은 문서"가 아니라 "이 문서만 보고 구현자가 바로 만들 수 있는 문서"를 강제하는 것이다.

이 표준의 핵심 질문은 하나다.

- 이 문서를 받은 구현자가 추가 기획 회의 없이 실제 구현에 들어갈 수 있는가

그 답이 `예`가 아니면 문서는 아직 부족하다.

## 1. What "Reference Grade" Means

이 문서가 요구하는 품질은 아래와 같다.

- 범위가 닫혀 있다.
- 비목표가 명확하다.
- 열린 결정보다 동결된 결정이 많다.
- 상태, 타입, 데이터, 공식, 이벤트, 화면 흐름이 서로 연결된다.
- 문서에 등장하는 고유명사와 ID가 실제 정의로 닫힌다.
- 구현 순서와 검증 방법이 있다.
- 리스크는 남길 수 있지만, 리스크가 구현 착수 자체를 막아서는 안 된다.

즉, `아이디어 문서`가 아니라 `구현 참조 문서`여야 한다.

## 2. Minimum Document Set

AI가 실전급 문서 세트를 만든다면 최소 아래 묶음을 목표로 한다.

| 문서 | 역할 | 없으면 생기는 문제 |
| --- | --- | --- |
| `spec.md` | 마스터 스펙, 범위, 계약, 공식, 실데이터 | 전체 기준점이 없음 |
| `designs.md` | UI/UX, 화면 흐름, 레이어, 데이터 연결 | 구현자가 화면을 임의 해석함 |
| `implementation_summary.md` | 시스템 분해, 파일 책임, 구현 순서 | 구현 시작점이 흐려짐 |
| `DESIGN_DECISIONS.md` | 왜 이렇게 정했는지, 무엇을 기각했는지 | 나중에 같은 논쟁 반복 |
| `BUILD_GUIDE.md` | 스캐폴딩, 빌드, 런타임 경로, 실행 절차 | 첫 실행에서 막힘 |
| `audit_roadmap.md` | 단계별 목표, 검증 포인트, 감사 기준 | 구현이 산만해짐 |
| `CHANGELOG.md` | 문서 변경 이력 | 무엇이 언제 바뀌었는지 추적 불가 |

도메인에 따라 이름은 바꿔도 되지만 역할은 유지한다.

## 3. Non-Negotiable Requirements

어떤 문서든 아래 항목은 반드시 만족해야 한다.

### 3.1 Scope Closure

- 목표를 적는다.
- 성공 기준을 적는다.
- 비목표를 적는다.
- "나중에 생각" 항목은 최대한 제거하고, 남긴다면 후속 Phase나 잔여 리스크로 격리한다.

### 3.2 Frozen Decisions

- 구현을 좌우하는 큰 결정은 동결한다.
- 예시
  - 런타임 구조
  - 저장 방식
  - 입력 범위
  - 렌더링 경계
  - 애니메이션/리소스 파이프라인
  - 핵심 UI 정책

"A일 수도 있고 B일 수도 있다"는 문장은 구현 문서에서 실패다.

### 3.3 Typed Contracts

문서에 등장하는 핵심 계약은 타입이나 스키마로 고정한다.

- 상태 타입
- 결과 타입
- 저장 타입
- 설정 타입
- 이벤트 타입
- 데이터 정의 타입
- 브리지/API 메서드 시그니처

문서에 나온 필드는 모두 의미가 설명돼야 한다.

### 3.4 Concrete Numbers

좋은 문서는 형용사보다 수치가 많다.

- "빠른 이동"이 아니라 `moveSpeed: 260`
- "짧은 선딜"이 아니라 `startup: 4f`
- "잠깐 흔들림"이 아니라 `80ms ~ 120ms`
- "짧은 무적"이 아니라 `invincible: 120ms`

수치가 없으면 구현자는 자기 판단으로 메운다.

### 3.5 Real Data Samples

타입만 있어서는 부족하다. 최소 1세트 이상의 실데이터가 있어야 한다.

- 플레이어 기본값
- 핵심 액션/공격 데이터
- 적 1세트 이상
- 스테이지/플로우 예시 1세트 이상
- 저장 기본값

"예시 데이터"가 아니라 "바로 코드로 옮길 수 있는 데이터"여야 한다.

### 3.6 Execution Path

문서는 반드시 구현 순서를 제시해야 한다.

- 무엇부터 만들지
- 어떤 파일이 필요한지
- 어떤 시스템이 먼저 서야 하는지
- 어디서 검증할지

구현 순서가 없으면 문서는 설명서에 머문다.

### 3.7 Verification Path

문서는 반드시 검증 기준을 가져야 한다.

- 어떤 명령을 실행할지
- 어떤 수동 검증을 할지
- 어느 단계에서 무엇이 통과되어야 하는지
- 어떤 산출물이 생성되어야 하는지

"완료"는 구현자의 감각이 아니라 검증 결과로 정의해야 한다.

## 4. Master Spec Blueprint

마스터 스펙은 최소 아래 구조를 가져야 한다.

1. 문서 운영 규칙
2. 프로젝트 정체성
3. 목표와 성공 기준
4. 비목표
5. 동결된 핵심 결정
6. 기술 스택과 아키텍처 원칙
7. 런타임/빌드 파이프라인
8. 디렉터리 구조
9. 핵심 동작 정의
10. 시스템 명세
11. 경계 타입과 계약
12. 저장/설정/진행 정책
13. 동결된 공식
14. 실데이터 기준표
15. 단계별 로드맵
16. 명령어와 검증 기준
17. 보안/구현 경계
18. 잔여 리스크

특히 아래는 빠지면 안 된다.

- 상태 전이
- 데이터 구조
- 결과 타입
- 브리지/API 계약
- 기본 저장값
- 보상/비용/레벨업 공식
- 최소 1개 이상의 실제 콘텐츠 데이터

## 5. Design Doc Blueprint

디자인 문서는 보기 좋은 설명이 아니라 구현 지시서여야 한다.

반드시 포함할 것:

- 핵심 경험
- 전체 화면 흐름
- 전투/핵심 화면 레이어 구조
- 컬러/타이포 토큰
- 레이아웃 기준
- HUD/화면 요소 목록
- 데이터 연결 기준
- 화면별 버튼 정책
- 타격감/모션 규칙
- React/Canvas 또는 UI/코어 역할 분리
- 동결된 디자인 결정

좋은 디자인 문서는 "예쁘게 만들어라"라고 쓰지 않는다.
대신 "어디에 무엇이 있고, 어떤 데이터만 읽고, 어떤 상황에서 어떻게 반응하는지"를 적는다.

## 6. Implementation Summary Blueprint

구현 요약 문서는 시작점 역할을 해야 한다.

반드시 포함할 것:

- 전체 런타임 흐름
- 시스템 분해표
- 경계 계약 요약
- 파일 책임
- 알고리즘 메모
- 동결된 공식 요약
- 첫 플레이어블의 최소 범위
- 구현 순서 권장
- 유지보수 규칙

이 문서는 "처음 코드를 여는 구현자"를 위한 문서다.

## 7. Decision Record Blueprint

결정 문서는 반드시 `왜`를 남겨야 한다.

각 결정은 아래를 포함한다.

- 배경
- 결정
- 대안과 기각 사유
- 결과

좋은 결정 문서는 "이렇게 했다"로 끝나지 않는다.
"왜 다른 길을 버렸는지"를 남긴다.

## 8. Build Guide Blueprint

빌드 가이드는 실제로 첫 실행을 성공시켜야 한다.

반드시 포함할 것:

- 사전 준비
- 안전한 스캐폴딩 절차
- 필수 설치 항목
- 런타임 출력 경로
- 엔트리 파일 연결 방식
- 루트 `package.json` 필수 스크립트
- 최소 `main/preload` 예시
- 전역 타입 선언 예시
- 첫 실행 명령
- 빌드 후 확인할 산출물
- 배포 전 체크리스트

특히 아래 실패를 막아야 한다.

- "명령은 있는데 실제 엔트리가 연결되지 않음"
- "renderer는 뜨는데 main/preload 산출물이 없음"
- "보안 옵션은 말했지만 코드 예시는 없음"

## 9. Audit Roadmap Blueprint

로드맵 문서는 일정표가 아니라 구현 감사 프레임이어야 한다.

반드시 포함할 것:

- 정합성 감사
- 위험요소 감사
- 아키텍처 감사
- 로드맵 감사
- Phase별 목표
- Phase별 구현 항목
- Phase별 실제 알고리즘 메모
- Phase별 검증 포인트
- 체크포인트 정책
- 현재 남은 핵심 리스크

좋은 로드맵은 "다음에 뭐 하지?"가 아니라 "지금 이 페이즈를 넘겨도 되는가?"를 판단하게 해준다.

## 10. Cross-Document Closure Rules

문서 세트는 서로 물려 있어야 한다.

아래 규칙을 만족해야 한다.

- `designs.md`에서 말한 이벤트는 `spec.md` 타입에 존재해야 한다.
- `spec.md`에서 정의한 타입은 `implementation_summary.md` 파일 책임에 반영돼야 한다.
- `BUILD_GUIDE.md` 스크립트는 `spec.md` 런타임 파이프라인과 일치해야 한다.
- `audit_roadmap.md`의 구현 항목은 `spec.md`의 데이터와 타입으로 바로 구현 가능해야 한다.
- `CHANGELOG.md`는 큰 문서 구조 변경을 기록해야 한다.

한 문서에서만 존재하는 개념은 위험 신호다.

## 11. Closure Rules for IDs, Flags, CTAs, and APIs

AI가 가장 자주 놓치는 구간이다. 반드시 닫아야 한다.

### 11.1 IDs

문서에 등장한 ID는 모두 정의돼야 한다.

- 스테이지 ID
- 적 ID
- 공격 ID
- 이벤트 ID
- 상태 이름

참조만 있고 정의가 없으면 문서 실패다.

### 11.2 Flags

불리언이나 상태 플래그는 의미와 생성 조건이 있어야 한다.

- 이름
- 의미
- 계산 조건
- 소비 위치

예시:

- `flawlessBonus = damageTaken === 0`

이 정도로 닫혀 있어야 한다.

### 11.3 CTAs

화면 버튼은 존재만 적지 말고 활성화 조건과 후속 상태를 적어야 한다.

예시:

- 버튼 이름
- 언제 활성화되는지
- 누르면 어디로 가는지
- 어떤 상태를 갱신하는지

### 11.4 Bridge / API

브리지나 API는 "존재한다"가 아니라 메서드 계약으로 써야 한다.

예시:

```ts
type AppBridge = {
  loadSave(): Promise<SaveData | null>;
  writeSave(save: SaveData): Promise<void>;
};
```

## 12. Anti-Patterns

아래 문장은 구현 문서에서 금지에 가깝다.

- "적당히"
- "필요시"
- "원하면"
- "추후 고려"
- "게임답게"
- "자연스럽게"
- "유연하게"
- "대충 이 정도"

이 표현들은 방향은 있어 보여도 구현을 닫지 못한다.

아래 구조도 위험하다.

- 타입 없는 설명
- 수치 없는 밸런스
- 버튼은 있는데 상태 갱신 규칙이 없음
- 이벤트 이름은 있는데 payload가 없음
- 공격 ID는 있는데 히트박스/공식이 없음
- 빌드 명령은 있는데 출력 엔트리 연결이 없음
- 저장 구조는 있는데 초기값/리셋 정책이 없음

## 13. Quality Checklist

AI가 문서를 제출하기 전에 아래를 체크해야 한다.

- 목표와 비목표가 모두 적혀 있는가
- 핵심 결정이 동결됐는가
- 큰 타입 계약이 정의돼 있는가
- 고유 ID가 모두 실데이터로 닫혔는가
- 공식이 수치와 함께 정의돼 있는가
- 최소 1세트 이상의 실제 데이터가 있는가
- 구현 순서가 있는가
- 검증 명령과 체크포인트가 있는가
- 화면 버튼의 정책이 있는가
- 저장/설정/브리지 정책이 있는가
- 문서 간 용어와 타입이 일치하는가
- 남은 리스크가 구현 불가 수준이 아닌가

하나라도 `아니오`면 아직 참조용 설계도로 부족하다.

## 14. Acceptance Standard

아래 세 가지를 모두 만족하면 실전급 참조 문서로 본다.

1. 구현자가 문서만 보고 첫 플레이어블 버전을 만들 수 있다.
2. 검증자가 문서만 보고 무엇이 완료/미완료인지 판단할 수 있다.
3. 후속 AI가 문서를 읽고 같은 개념을 더 낮은 품질로 다시 쓰지 않는다.

## 15. Copy-Paste Instruction Block for GPTs, Gems, and Skills

아래 블록은 GPTs/Gems/skills의 참조 지침으로 바로 붙여 넣을 수 있다.

```md
You are generating implementation-grade documentation, not brainstorming notes.

Your output must be closed enough that an engineer can start building without a follow-up planning meeting.

Required qualities:
- explicit goals, success criteria, and non-goals
- frozen key decisions instead of open options
- typed contracts for major data/state/event/API boundaries
- concrete formulas, thresholds, timings, and numeric defaults
- real sample data, not placeholder examples
- implementation order and verification steps
- CTA/button policies, save/reset policies, and bridge/API signatures
- cross-document consistency if multiple docs are produced

Do not leave vague placeholders such as "later", "if needed", "optimize", "make it intuitive", or "adjust values during implementation".
If a field, ID, event, flag, or screen is mentioned, define it.
If a formula or bonus exists, specify exactly how it is calculated.
If a build or runtime command exists, specify the actual entry/output path it depends on.

Target output quality: reference-grade implementation docs.
```

## 16. Final Principle

좋은 구현 문서는 "멋진 문장"보다 "닫힌 결정"이 많다.

AI가 문서를 만들 때 목표는 감탄이 아니라 착수 가능성이다.
이 문서를 기준으로 삼는다면, 결과물은 최소한 아래 질문에 버티어야 한다.

- "이걸 지금 바로 만들 수 있는가"
- "문서에 나온 모든 개념이 정의돼 있는가"
- "검증 기준 없이 완료를 주장할 수 없게 되어 있는가"

이 세 질문에 버티지 못하면 다시 써야 한다.
