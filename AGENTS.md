# AGENTS.md (GPT-Codex 5.3 Global Rules - D3D Protocol)

**[System Directive]**
이 문서는 현재 작업 공간에서 동작하는 **GPT-Codex 5.3 에이전트**가 최우선으로 참조해야 하는 전역 행동 지침(Master System Prompt)입니다. 에이전트는 사용자의 요청을 처리하거나 코드를 생성/수정할 때, 아래 15가지의 규칙을 단 하나도 누락 및 축약하지 말고 절대적으로 준수해야 합니다.

## 1. Template & Project Metadata
* **Template Name:** D3D Protocol
* **Template Version:** 1.0
* **Project Info:**
    * **Name:** ${PROJECT_NAME}
    * **Version:** ${CURRENT_VERSION}
    * **Environment:** ${ENV_TYPE}

## 2. Permissions
* **Filesystem Scope:** project_root
* **Allow Read:** ["all"]
* **Allow Write:** ["documentation", "config", "version_files"]
* **Allow Execute:** ["package_manager", "compiler", "test_runner"]

## 3. Initialization Protocol
* **Description:** 프로젝트 시작, 스캐폴딩(Scaffolding) 및 환경 설정 시 데이터 보존 규칙
* **CRITICAL SAFETY RULE:** * **NO_DESTRUCTIVE_INIT:** 프로젝트 루트에 `.md` 파일(특히 spec.md, designs.md 등 DNA 파일)이 하나라도 존재할 경우, **기존 파일을 삭제하거나 덮어쓰는 모든 종류의 초기화 명령(Scaffolding Tools)을 루트 경로에서 직접 실행하는 것을 절대 금지한다.**
    * **Forbidden Flags:** `--force`, `--overwrite`, `rm -rf` 등 파괴적 옵션 사용 금지.
    * **Applicable Tools:** `create-tauri-app`, `npm create`, `npx`, `cargo new`, `flutter create`, `django-admin`, `spring init` 등 모든 프레임워크 생성 도구 포함.
* **Safe Scaffolding Strategy (Merge Pattern):**
    * **Condition:** 이미 문서(DNA)가 존재하는 상태에서 프레임워크 초기화가 필요할 때
    * **Action:**
        1. **Temp Init:** 하위 임시 폴더(예: `./temp_init` 또는 `./_scaffold_temp`)에 프로젝트를 생성한다.
        2. **Selective Move:** 생성된 임시 폴더에서 소스 코드(`src`, `lib` 등)와 설정 파일(`package.json`, `Cargo.toml`, `pubspec.yaml` 등)만 루트로 이동(Move/Merge)시킨다.
		**[주의]** 이때 `.gitignore`, `.env` 등 **점(.)으로 시작하는 숨김 파일(Hidden Files)이 누락되지 않도록** 명시적으로 포함하여 이동시킬 것.
        3. **Conflict Check:** 이동 시 기존의 DNA 문서(README, spec.md, designs.md 등)를 절대 덮어쓰지 않도록 주의한다.
        4. **Cleanup:** 임시 폴더를 삭제한다.
* **Auto Create If Missing:** true
* **MASTER_PLAN_INIT (CRITICAL):** 프로젝트 초기화 단계이거나 루트에 `spec.md`가 없는 상태에서 사용자가 기능 구현이나 코드를 요청할 경우, **절대 즉시 코드를 작성하지 않는다.**
    * **Action:** 사용자의 초기 입력값(바이브)을 분석하여 프로젝트의 기술 스펙, 디자인 로직, 구현 목표, 방향성을 담은 마스터플랜 문서인 `spec.md`를 최우선으로 생성하여 제시한다.
    * 코딩은 사용자가 이 `spec.md`의 내용에 동의한 이후에만 시작할 수 있다.

## 4. Macro Commands
* **Description:** 사용자가 특정 키워드 입력 시 수행할 복합 작업 정의 (단축키)
* **Commands:**
    * **Trigger:** /audit
    * **Aliases:** ["감사 실행", "Audit Mode"]
    * **Action Source:** ./audit_roadmap.md
    * **Behavior:** 에이전트는 즉시 'audit_roadmap.md'를 로드하고, 해당 문서에 정의된 4단계 감사 프로세스(정합성, 위험요소, 아키텍처, 로드맵)를 수행하여 리포트를 출력한다.

## 5. Error Handling Rules
* **PROTOCOL:** 오류 수정 요청 시, 즉시 코드를 수정하지 말고 먼저 해당 오류와 관련된 기술, 라이브러리, 함수, 의존성, 버전 정보를 '그라운딩(Web Search)'을 통해 최신 상태로 파악해야 파악해야 한다.
* **MANDATORY GROUNDING:** 학습된 데이터(Training Data)는 최신 기술과 맞지 않을 가능성이 높으므로, 오류 해결에 실패할 경우 재시도 없이 '즉시' 그라운딩을 수행하여 최신 레퍼런스를 참조해야 한다.

## 6. Documentation Rules
* **SPEC_IS_LAW:** `spec.md`는 이 프로젝트의 절대적인 **'마스터플랜(Master Plan)'**이다.
    * `designs.md`, `README.md`, `DESIGN_DECISIONS.md` 등 모든 하위 문서는 오직 `spec.md`에 명시된 기술 스펙과 방향성을 근간으로 파생되어야 한다.
    * 하위 문서 생성 및 수정 시 `spec.md`의 내용과 충돌하는 설정이나 임의의 기능 추가는 엄격히 금지된다.
* **CRITICAL:** 모든 작업에서 문서 작성 및 갱신을 최우선 순위(Top Priority)로 두며, 개발 착수 전/후에 반드시 관련 문서를 먼저 점검한다.
* **STANDARD_ENFORCEMENT (CRITICAL):** 프로젝트 내 모든 기획/스펙/설계/요약 문서 작성 시 반드시 `AI_IMPLEMENTATION_DOC_STANDARD.md`를 우선 참조해야 한다. 해당 문서에 명시된 Typed Contracts(데이터 타입 명시), Concrete Numbers(구체적 수치), Real Data Samples(실데이터), Execution & Verification Path(구현/검증 순서), Scope Closure(목표/비목표 명확화) 기준을 충족하지 못하는 문서는 통과(Accepted)되지 않은 것으로 간주하며 재작성해야 한다.
* **UTF-8 ENFORCEMENT (CRITICAL):** 모든 파일의 읽기 및 쓰기(소스 코드, 마크다운 문서 등 포함) 작업 시 반드시 **UTF-8 인코딩**을 강제한다. 한국어 Windows 환경의 기본 인코딩(cp949 등)으로 인해 텍스트가 깨지거나 데이터가 손실되는 문제를 원천 차단하기 위해, 시스템 환경에 의존하지 말고 모든 파일 I/O 작업에 명시적으로 UTF-8을 지정해야 한다.
* **VERIFICATION:** 개발 시 소스 코드와 문서 간의 정합성을 검증하는 루틴을 상시 가동하며, 불일치 발견 시 즉시 코드 수정을 중단하고 문서를 동기화한다.
* **AUDIT_PROTOCOL (CRITICAL):** `implementation-auditor` 에이전트를 통한 감사는 반드시 **[1. 감사 -> 2. 리포트 및 승인(ask_user) -> 3. 문서 반영]**의 3단계 프로세스를 준수해야 한다. 승인 없이 문서를 임의로 수정하는 것은 금지되며, 반영 시 `AI_IMPLEMENTATION_DOC_STANDARD.md`의 'Reference Grade' 기준을 충족해야 한다.
* **README Language:** README.md는 반드시 다국어로 작성하며, 언어 순서는 [한 / 영 / 일 / 중(번체) / 중(간체)]를 엄수할 것
* **Standard Language:** README.md를 제외한 모든 문서(CHANGELOG, DESIGN_DECISIONS, IMPLEMENTATION_SUMMARY, designs.md 등)는 반드시 '한국어'로만 작성할 것
* **Sync Policy:** 코드 변경 시 연관된 모든 문서를 즉시 동기화할 것
* **Feature Description:** 새 기능은 README.md의 Features 섹션에 기술할 것
* **Changelog Policy:** 모든 변경사항은 CHANGELOG.md에 SemVer(Semantic Versioning) 기준으로 기록할 것
* **Architecture Change:** API 또는 아키텍처 변경 시 기술 명세(Spec)와 의사결정 문서(Design Decisions)를 반드시 최신화할 것
* **Dependency Change:** 의존성 패키지 변경 시 관련 설정 파일과 README를 동시에 업데이트할 것
* **LOCAL UPDATE ENFORCEMENT:** 프로젝트 진행 시 Git 공유 여부와 무관하게 모든 문서는 로컬 프로젝트 내에서 반드시 업데이트되어야 한다.
* **PRIORITY OVER CODE:** 문서 업데이트는 소스코드 작성보다 우선되는 절대적 중요사항이며, 개발 환경을 점검하여 문서가 누락 없이 갱신되도록 강제한다.
* **DESIGNS_REFERENCE:** 디자인이나 UI를 제작/수정할 때는 반드시 'designs.md'를 참조해야 하며, 디자인 또는 UI가 변경될 때마다 해당 문서를 반드시 최신화해야 한다.
* **DESIGNS_CONTENT_SPEC:** 'designs.md'에는 (1) ASCII 기반 프로젝트 디자인 구조도, (2) 각 부분별 기능 상세 설명, (3) 구현 시 주의사항 및 요청사항이 포함되어야 한다.
* **INITIAL_AI_INFERENCE:** 문서 생성 시 내용이 없는 초기 단계라면, AI는 spec.md의 내용을 바탕으로 디자인 구조와 기능을 판단하여 'designs.md'의 내용을 임시로 생성하고 채워넣는다.

## 7. Git Management Rules
* **Description:** Git 버전 관리 및 파일 업로드 정책 (소스 코드 포함)
* **Allowed Files:** ["All Source Codes (.*)", "All Markdown Documents (*.md)"]
* **Policy:**
    * **GIT INCLUSION STRATEGY:** 프로젝트의 모든 소스 코드와 개발 시 생성되는 모든 MD 문서는 Git에 업로드한다.

## 8. Source Code Annotation Rules
* **i18n Implementation:** 개발 시 모든 코드는 다국어(한 / 영 / 일 / 중(번체) / 중(간체))를 지원하도록 구현할 것
* **CRITICAL COMMENT:** 소스 코드 내의 모든 주석(Comment)은 반드시 '한국어'로만 작성할 것. (영문 등 타 언어 혼용 금지)
* **Comment Quality:** 주석은 코드의 의도와 맥락을 파악할 수 있도록 한국어로 풍부하게 작성할 것
* **Detail Spec:** 주석 작성 시 구현된 로직의 구체적인 동작 원리와 구현 사항을 명시적으로 기술할 것
* **Versioning in Code:** 코드 변경 시 '[vX.X.X]'와 같이 버전을 명시하고, 이전 버전 대비 무엇이 변경되었는지 한국어 주석으로 상세히 기술할 것
* **Feature Deletion:** 기능 삭제 시 소스 코드는 제거하되, 해당 위치에 '삭제된 기능의 내용', '삭제 사유', '삭제된 버전'을 한국어 주석으로 남겨 맥락을 보존할 것 (주석 처리된 코드는 남기지 않음)

## 9. Documentation Sync Checklist
* **on_feature_add:** ["Verify Code-Doc Consistency", "Update README Features (Multilingual)", "Add to CHANGELOG (Added - Korean)", "Update Docstrings (Korean)", "Update spec.md (새로운 기능 스펙 및 방향성 추가 - Korean)"]
* **on_bug_fix:** ["Grounding Check (Search Latest Info)", "Verify Code-Doc Consistency", "Add to CHANGELOG (Fixed - Korean)", "Update Troubleshooting in README (Multilingual)", "Add Root Cause Comment (Korean)"]
* **on_refactor:** ["Verify Code-Doc Consistency", "Add to CHANGELOG (Changed/Improved - Korean)", "Update Implementation Details"]
* **on_architecture_change:** ["Update spec.md (Master Plan 갱신 - Korean)", "Verify Structural Consistency", "Update DESIGN_DECISIONS.md (Why - Korean)", "Update IMPLEMENTATION_SUMMARY.md (Korean)", "Update designs.md (ASCII & Logic - Korean)"]
* **on_version_change:** ["Regenerate audit_roadmap.md (Analyze new risks - Korean)", "Update Version in Files"]
* **on_config_change:** ["Update README Configuration", "Update Environment Variables Example"]
* **on_ui_design_change:** ["Update designs.md (ASCII and Functional Specs - Korean)", "Ensure consistency with spec.md"]
* **SPEC_LIVING_DOCUMENT:** 코드가 대대적으로 리팩토링되거나 초기 방향성과 달라지는 요구사항이 발생할 경우, 다른 어떤 파일보다 먼저 `spec.md`의 관련 지침과 참고점을 최신화하여 마스터플랜을 갱신해야 한다.

## 10. Version Control
* **Format:** MAJOR.MINOR.PATCH
* **Increment Rules:**
    * **MAJOR:** Breaking changes or significant API shifts
    * **MINOR:** New features (backward compatible)
    * **PATCH:** Bug fixes and minor improvements

## 11. Documentation Standards
* **Primary Language:** Korean (Must be used for all docs except README)
* **Multilingual README:** ["Korean", "English", "Japanese", "Chinese (Traditional)", "Chinese (Simplified)"]
* **Comment Language:** Korean Only
* **Code i18n Support:** ["Korean", "English", "Japanese", "Chinese (Traditional)", "Chinese (Simplified)"]
* **Format:** Markdown
* **Changelog Style:** Keep a Changelog
* **Commit Message:** Conventional Commits

## 12. Required Files
* spec.md, README.md, CHANGELOG.md, BUILD_GUIDE.md, IMPLEMENTATION_SUMMARY.md, LESSONS_LEARNED.md, DESIGN_DECISIONS.md, audit_roadmap.md, designs.md

## 13. Automation Philosophy
* **Agent Mode:** Autonomous
* **Auto Approve:** true
* **Description:** 에이전트는 문서 동기화와 프로젝트 아카이빙을 별도의 스크립트 실행 없이 '내장 로직'으로 수행한다. 사용자의 명시적 중단이 없는 한 무한 루프 방지 하에 자동 완수를 지향한다.

## 14. AI Learning and Recovery DNA
* **Enabled:** true
* **Storage Strategy:**
    * **Local Archive Path:** ./.antigravity/archive
    * **Global Archive Path:** ~/antigravity/knowledge_base
    * **Method:** AI 에이전트는 프로젝트 완료 시점 또는 주요 마일스톤 도달 시, 스스로 주요 문서를 요약하여 위 경로에 기록한다.
* **Recovery Logic:**
    * **Principle:** 문서는 프로젝트의 유전 정보다. 소스 코드가 전실되어도 문서(Summary, Lessons Learned, Design Decisions, designs.md, spec.md)만으로 시스템 아키텍처를 95% 이상 복구할 수 있도록 상세히 기록한다.
    * **Targets:** ["Structure", "Core Logic", "Decision History", "Pitfalls", "UI/Design Layout", "Master Plan"]
* **Learning Loop:**
    * **Process:** 1. 과거 유사 프로젝트 문서 읽기 -> 2. 검증된 패턴 적용 -> 3. 동일 실수 방지 -> 4. 현재 프로젝트의 교훈 기록

## 15. Self Update Rules
* 프로젝트 완료 시 'LESSONS_LEARNED.md'를 자동 생성하고 AI 학습 데이터로 전환할 것
* 버전 번호가 올라갈 때(Version Bump), 변경된 기능에 맞춰 'audit_roadmap.md'를 재작성하여 최신 감사 기준을 수립할 것
