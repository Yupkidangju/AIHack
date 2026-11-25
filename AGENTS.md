# AGENTS.md (Global Rules - D3D Protocol)

**Template Name:** D3D Protocol  
**Template Version:** 1.4  
**Project Name:** ${PROJECT_NAME}  
**Project Version:** ${CURRENT_VERSION}  
**Environment:** ${ENV_TYPE}

## 0. Rule Priority and Interpretation

This file defines the working policies for the current workspace. Apply this order of precedence:

1. Platform, system, security, and legal constraints
2. The user's explicit request for the current task
3. The project's `spec.md` and approved design documents
4. This `AGENTS.md`
5. Consistent codebase conventions and tool constraints

Resolve conflicts by priority and specificity. When the latest user request changes `spec.md`, do not reject the request: update `spec.md` first or within the same work unit, then synchronize code and documentation.

Reserve emphatic terms such as `MUST`, `PROHIBITED`, and `NEVER` for safety or data-preservation needs. If rules cannot be reconciled, do not reject the entire task. Continue what can safely be done, then report the conflict and its resolution.

## 1. Project Metadata

- **Name:** ${PROJECT_NAME}
- **Version:** ${CURRENT_VERSION}
- **Environment:** ${ENV_TYPE}
- **Repository Type:** ${REPOSITORY_TYPE}
- **Documentation Profile:** ${DOC_PROFILE}
- **User-facing Languages:** ${SUPPORTED_LANGUAGES}

Infer missing values from existing project structure and files, but label inferred values as `Assumption` or `Unconfirmed` in the documentation.

## 2. Permissions

### 2.1 Filesystem Scope

- The default read/write scope is `project_root`.
- The agent may use temporary directories it creates for the task.
- Write outside the project only with the user's explicit permission.
- Do not write or delete through a symbolic link or normalized path that resolves outside the allowed scope.

### 2.2 Read Permission

The agent may read project files, including:

- Source, tests, documentation, configuration, scripts, assets, migrations, lock files, and build metadata
- Hidden files and tool configuration
- Error logs and generated output within the project

Read secrets such as `.env` files, certificates, tokens, and private keys only as needed. Do not expose them in responses, logs, documentation, commits, or external searches.

### 2.3 Write Permission

To fulfill the user's request, the agent may create, modify, move, rename, or safely delete the following within `project_root`:

- Documentation and version files
- Source and generated code
- Tests, fixtures, and snapshots
- Project assets such as images, icons, font references, audio, and static files
- Configuration, manifests, lock files, and build files
- Scripts, tool files, and migrations
- Project-local archives and temporary work

Routine source and asset writes do not require separate approval. Bulk deletion, hard-to-reverse data conversion, secret changes, and production-data operations follow Section 13's approval boundaries.

### 2.4 Execute Permission

As needed for the task, the agent may run:

- Shells and file utilities
- Package managers and dependency inspection tools
- Compilers, build tools, and task runners
- Test runners, linters, formatters, and type checkers
- Code generators, migration tools, and development servers
- Read-only Git operations such as status, diff, and log

Commit, push, tag, release, forced reset, and history rewriting require an explicit user request or a pre-approval policy recorded in the project.

### 2.5 Text and Binary Files

- Use UTF-8 for text files by default.
- Do not impose text encoding on binary assets.
- Preserve existing BOMs, line endings, and generator requirements without corrupting data.

## 3. Initialization and Scaffolding Protocol

### 3.1 Project State Detection

Before initialization, distinguish:

- **Empty Greenfield:** A new directory with no meaningful files.
- **Document-first Greenfield:** A new project containing only documents such as `spec.md` and `designs.md`.
- **Brownfield:** A project with source, configuration, Git history, or an existing application.

The presence of a single `.md` file does not prohibit all scaffolding.

### 3.2 Protected Files

Protect these by default:

- `spec.md`, `designs.md`, `DESIGN_DECISIONS.md`
- Existing `README.md` and `CHANGELOG.md`
- `.env`, certificates, keys, and user data
- Existing source, configuration, tests, assets, and user-created files

Do not overwrite protected files without an explicit merge or user request.

### 3.3 Safe Scaffolding

- Root initialization is allowed when the directory is empty and the tool can operate safely.
- If files exist or overwrite behavior is unclear, scaffold in a temporary directory and merge.
- Check generated output for conflicts with all existing files, not just DNA documents.
- Merge in no-clobber mode by default.
- Include required hidden files such as `.gitignore` and `.editorconfig`, merging rather than overwriting existing files.
- Do not overwrite an existing `.env` with a generated one. Handle required keys through `.env.example` or safe merge instructions.
- Delete a temporary directory only after verifying its exact path and that the agent created it for this task.

### 3.4 Destructive Command Policy

Prohibited:

- Recursive deletion from the project root or an ancestor without path verification
- Indiscriminate `--force`, `--overwrite`, `reset --hard`, or `clean -fdx` against existing files
- Bulk overwrites without a backup, diff, or target verification

Allowed:

- Verified cleanup of temporary directories created for this task
- Clearly scoped cleanup of generated output, caches, and build directories
- Replacements specifically requested by the user with a verified impact scope

### 3.5 Master Plan Initialization

- If a new project lacks `spec.md`, create a minimum actionable `spec.md` before implementation.
- When the user explicitly requests implementation or completion and no material ambiguity remains, implementation may continue after creating `spec.md` without waiting for separate approval.
- When the user requests only planning or review, stop at documentation.
- If an existing project lacks `spec.md`, reconstruct it from current code, configuration, and tests. For a simple bug fix or small maintenance task, first record a minimal change plan and verification criteria in the relevant governing document, then proceed.
- Require explicit approval for hard-to-reverse decisions such as destructive data-model changes, public API changes, or production migrations, rather than for routine planning decisions.

## 4. Document-First Workflow and Reference Standards

Before changing code for planning, design, implementation, fixes, refactoring, or automated development, record the objective, scope, key decisions, implementation order, and verification criteria in the relevant project documents and establish them as the working baseline.

Whatever planning or automation method is used, including `plan`, `grill-me`, or `goal`, document its outcome at the appropriate depth under `AI_IMPLEMENTATION_DOC_STANDARD.md`, then implement against those documents.

Documentation depth is proportional to the change. A small fix may need only a brief update, but do not change code before documenting its plan. If the objective, contract, or roadmap changes during implementation, update the documents before continuing the code changes.

Establishing this baseline means settling the current implementation criteria in writing. It does not mean waiting for user approval unless the decision separately requires it.

All workers use this `AGENTS.md` and current project documents as their shared baseline. Select relevant portions of these references according to the task, project state, and required verification depth:

- Planning, specifications, design, implementation guides, and roadmaps: `AI_IMPLEMENTATION_DOC_STANDARD.md`
- Code, fixes, refactoring, tests, and implementation evidence: `AI_CODING_STANDARD.md`
- Audits, re-audits, consistency, engineering quality, and security reviews: `AI_AUDIT_DOC_STANDARD.md`

A mixed task may use multiple references. These links do not fix worker roles or require wholesale application of every document. Determine scope and order from relevance, project size, current Phase, and available evidence.

These standards do not change Section 0's precedence. They are evolving references refined by analyzing multiple projects and audit results, not rules for mechanically imposing past patterns on the current project. Record discrepancies between a standard, project documents, and actual evidence, and decide under Section 0.

Do not stop solely because a reference is absent or only partly applicable. Proceed safely with available evidence and report unused references and judgment limits.

## 5. Error Handling and Grounding

### 5.1 Local-first Diagnosis

Before fixing an error, inspect:

1. Whether the error is reproducible
2. Logs, stack traces, and failing tests
3. Actual installed versions, lock files, and configuration
4. Relevant code and recent changes

Simple syntax errors and internal logic errors may be fixed without unnecessary web searches.

### 5.2 Grounding Conditions

Consult current official sources when any of these apply:

- Behavior may depend on a library, framework, API, or platform version.
- The error or technology is unfamiliar or uncertain.
- Local documents and code are insufficient to isolate the cause.
- The first targeted fix and verification fail.
- Security, compatibility, or migration accuracy matters.

Prefer official documentation, release notes, standards, and official repositories, in that order.

### 5.3 Offline and Privacy Fallback

If web access is unavailable, use installed documentation, package metadata, help, lock files, and test results; report the limitation. Do not send secrets, proprietary code, customer data, or identifiable paths verbatim to external searches.

### 5.4 Bounded Recovery Loop

Bound diagnosis/fix/verification iterations. Do not repeat the same failure without new grounds. If unresolved, record the reproduction, attempted changes, remaining hypotheses, and next safe action.

## 6. Documentation Rules

### 6.1 Source of Truth

`spec.md` is the primary baseline for approved product goals and scope. The latest user request may change it; update related documents first or within the same work unit.

### 6.2 Proportional Documentation

Keep document updates proportional to change impact.

- Update relevant documents when user-facing features, public APIs, architecture, installation, configuration, or operating procedures change.
- For typos, internal implementation details, or behavior-preserving refactoring, update documents only when affected.
- Lengthy documentation work must not indefinitely block an obvious code fix. Keep the required update brief and proportional, but do not omit it.

### 6.3 Encoding

- Use UTF-8 by default for project-owned text documents.
- Do not force UTF-8 onto binary files or special tool-managed formats.
- Verify content integrity when changing an existing encoding.

### 6.4 Language Policy

- The language of these instruction/reference files does not determine the language of project deliverables. Select the language for new project-owned narrative documents in this order: the user's explicit language instruction for the task or deliverable; the existing project's documented language policy or established document-language convention; the verified display language of the user's operating system; otherwise, the user's current conversation language.
- Use the user's environment, not the agent VM, container, build server, or shell locale, to determine the OS display language. If it cannot be verified with available information, use the conversation-language fallback without blocking work; do not claim to have detected the OS language.
- Preserve the language of existing documents unless the user requests a translation or a project language-policy change. Continue to follow existing project conventions when no override is requested.
- Public API documentation, generated code, schemas, translation resources, tool conventions, and external contribution documents follow their ecosystem's requirements. Keep identifiers, paths, machine-readable formats, and contract-defined literal values unchanged unless their contracts are deliberately revised.
- README language coverage follows `${DOC_PROFILE}` and `${SUPPORTED_LANGUAGES}`. When multilingual READMEs are required, retain this default order unless the project specifies another: Korean, English, Japanese, Traditional Chinese, Simplified Chinese.

### 6.5 Document Synchronization

- Reflect new user-facing features in the README's feature description.
- Record release-worthy changes in CHANGELOG.
- For API or architecture changes, update `spec.md`, `DESIGN_DECISIONS.md`, and related design documents.
- Reflect dependency changes affecting installation or configuration in README or BUILD_GUIDE.
- Update `designs.md` for UI or design changes.
- Label inferred designs as `Assumption`, `Draft`, or `Unconfirmed`.

### 6.6 Design Document Content

For a project with UI or structural design, `designs.md` includes:

1. An ASCII or Mermaid structural diagram
2. Roles of major areas and components
3. Implementation constraints and cautions
4. Accessibility, responsive behavior, state, and error-handling criteria

Do not impose unnecessary screen designs on a UI-less library or single script.

## 7. Git Management Rules

- Version project-managed source, tests, documentation, configuration, scripts, migrations, required assets, and lock files.
- Respect `.gitignore` and existing repository policy.
- Do not commit `.env`, tokens, private keys, certificates, customer data, or secret configuration.
- Include dependency directories, caches, build output, and large generated files only when the project intentionally tracks them.
- Commit, push, tag, and release only on an explicit user request or under a pre-approval policy.
- Forced pushes, history rewriting, and branch deletion require separate explicit approval.

## 8. Source Code Annotation and i18n Rules

### 8.1 Internationalization

- Externalize user-facing strings to support required languages and the project's locale structure.
- Do not impose unnecessary five-language support on backend, library, CLI, test, or other code without a user UI.
- Supported languages follow `${SUPPORTED_LANGUAGES}` or existing project settings.

### 8.2 Comments and Docstrings

- For new project-owned explanatory comments with no existing project convention, use the narrative-document language selected in Section 6.4.
- Prefer existing code language style, public API documentation conventions, and linter or generator requirements where present.
- Do not impose comment-language rules on generated or external code.
- Explain intent, invariants, risks, and reasons for choices rather than repeating what the code does.
- Do not impair readability with excessive comments.

### 8.3 Change History

- Do not insert `[vX.X.X]` into code comments on every change.
- Keep change history in Git, CHANGELOG, and DESIGN_DECISIONS.
- Do not leave placeholder comments marking removed features.
- Keep comments for compatibility-related deprecations, migration warnings, and non-obvious constraints.

## 9. Documentation Sync Checklist

Apply only checklist items actually relevant to the change.

### Feature Add

- Check code/document consistency.
- Update README for user-facing features.
- Record release-worthy changes under CHANGELOG `Added`.
- Update `spec.md` if scope or contracts change.
- Update necessary tests and user-facing translations.

### Bug Fix

- Reproduce the issue and establish its cause.
- Consult current official sources for version-dependent behavior.
- Add or update regression tests.
- Update CHANGELOG when the fix should be communicated to users.
- Record only non-obvious recurrence-prevention conditions in code or related documents.

### Refactor

- Verify behavior is preserved.
- Avoid unnecessary document or version changes when public contracts and architecture are unchanged.
- Update implementation or design documents if structure changes.

### Architecture Change

- Update `spec.md`.
- Record the rationale and alternatives in `DESIGN_DECISIONS.md`.
- Update related `designs.md` and `IMPLEMENTATION_SUMMARY.md`.
- Verify migration and compatibility.

### Config or Dependency Change

- Update README, BUILD_GUIDE, and `.env.example` when installation, execution, or environment variables are affected.
- Synchronize lock files and configuration.
- Check that no secrets entered documentation or Git.

### UI Design Change

- Check consistency between `designs.md` and implementation.
- Verify accessibility, responsive behavior, state, and error presentation.
- Update `spec.md` if product scope changes.

## 10. Semantic Versioning

- Default to `MAJOR.MINOR.PATCH` for versioned projects.
- **MAJOR:** Incompatible public contract changes.
- **MINOR:** Backward-compatible new features.
- **PATCH:** Backward-compatible bug fixes and small improvements.
- Do not automatically increment the version for every code change.
- Change versions according to release policy, user requests, or project automation.
- Use prerelease and build metadata according to project conventions.

## 11. Documentation Standards

- Markdown is the default format.
- Prefer Keep a Changelog when CHANGELOG exists.
- Follow Conventional Commits if used by the repository.
- Existing, proven project document and commit conventions take precedence over these defaults.
- Edit a generated document's source, not the generated document by hand.

## 12. Required Files and Profiles

### 12.1 Core Files

The basic D3D project documents are:

- `spec.md`
- `README.md`

### 12.2 Extended Files

Create or maintain these according to project size and task needs:

- `CHANGELOG.md`: Release or user-facing change history.
- `BUILD_GUIDE.md`: Nontrivial build or deployment procedures.
- `IMPLEMENTATION_SUMMARY.md`: Implementation-structure handoff.
- `LESSONS_LEARNED.md`: Milestone or project closeout.
- `DESIGN_DECISIONS.md`: Important technical decisions.
- `audit_roadmap.md`: An ongoing audit process.
- `designs.md`: UI or structural design.

Do not create empty shells merely to satisfy a list. Preserve different existing filenames and document their correspondence.

## 13. Automation and Approval Boundaries

### 13.1 Autonomous Routine Work

No separate approval is required for:

- Routine creation or modification of source, tests, documentation, configuration, and assets
- Formatting, linting, building, and testing
- Cleanup of task-created temporary files and safe generated output
- Small, clearly scoped refactoring and bug fixes

### 13.2 Explicit Approval Required

A user request or pre-approval policy is required for:

- Writing outside the project
- Deleting production data or performing irreversible migrations
- Changing secrets, authentication, or authorization systems
- Paid external service calls or expensive operations
- External publication, deployment, release, email, or message sending
- Git commit, push, tag, forced reset, or history rewriting
- Bulk deletion or complete project replacement

### 13.3 Completion and Loop Control

- Aim to complete the requested scope unless the user stops the task.
- Do not repeat the same failure indefinitely.
- Give repeated work a termination condition, attempt limit, or completion gate.
- Continue independently completable work even when another step is blocked.
- Conclude with changed files, verification results, remaining risks, and unfinished items.

## 14. Learning and Recovery DNA

### 14.1 Local Archive

The default archive is `./.antigravity/archive` inside the project. Use an external global knowledge store only when the user explicitly permits it and grants write scope.

### 14.2 Privacy and Security

- Do not archive secrets, personal data, customer data, or credentials.
- Do not automatically upload archives to external services or model-training systems.
- `AI training data` means reusable project-local knowledge documents by default.
- Refer to another project's material only with access rights and user permission.

### 14.3 Recovery Content

Write recovery documentation specifically enough to reconstruct:

- Directory and component structure
- Core logic and data flow
- Key decisions and rejected alternatives
- Known pitfalls and recovery procedures
- UI or design structure
- Build, test, and deployment procedures

Treat `95% recovery` as a quality target, not an unverified guarantee.

## 15. Self-update Rules

- Create or update `LESSONS_LEARNED.md` when a major milestone or project closeout yields meaningful lessons.
- Do not rewrite `audit_roadmap.md` on every version change. Review only what new risks and change scope require.
- Change `AGENTS.md` policy itself only on the user's explicit request.
- Record repeatable success/failure patterns as candidates for improving related standards; do not immediately promote one case into a universal mandatory rule.
- State generation time, source files, assumptions, and unconfirmed matters in automatically generated documents.

## 16. Work-Unit Closure and Proportional Verification

This section follows Section 0's precedence and Section 13's approval boundaries. It does not lower quality, safety, or required verification criteria, or permit unverified work to be reported as complete.

- Use the existing work records under Section 4 to define each work unit's objective, direct and indirect impact, completion criteria, and required verification. Do not create separate forms or approval steps just for this section.
- Match verification to change impact, specific risks, and the project's verification plan. Reuse evidence and artifacts whose applicability remains valid; briefly record the reason for expanding verification scope, repetitions, or artifacts in the existing work record.
- Close a work unit once its completion criteria are met, then proceed to the next task within the requested scope. Mark unresolved blockers as blocked or on hold and separate out-of-scope improvements into follow-up work. Do not add completion criteria or reopen closed work without relevant grounds.
- At handoff, distinguish completed and incomplete work, actually verified scope and unverified scope, and key materials that are currently referenced, historical records, or unreferenced, as needed. Mark unknowns separately; classification alone does not justify routine exhaustive inventories.
- Distinguish verification needed for the current product, compatibility, or audit from preserving or reproducing development history. A reference in a historical record alone does not require re-verifying or re-copying the entire history for routine work.
- Before cleaning up intermediate artifacts, check actual use, references, and reproducibility; retain necessary conclusions and reproduction conditions. Lack of references alone does not authorize deletion. Preserve current inputs, evidence of unresolved failures, and irreplaceable originals.
