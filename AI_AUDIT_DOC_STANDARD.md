# AI Audit Documentation Standard

## Fast Use Guide

- New-feature audit: Sections 4, 6-10, 14-16, 20-21
- Bug-fix re-audit: Sections 6-9, 11, 16-17, and relevant patterns
- Security-change audit: Sections 8-9, 12-13, 16, 18, and SEC patterns
- Final pre-deployment audit: Sections 3-9, 13, 16-18, and the full Pattern Index

## 0. Purpose

This is the single standard reference for auditing AI-created or AI-modified project artifacts. It is not a collection of code-review tips. It aligns documentation, source, tests, configuration, build artifacts, security boundaries, and audit records through evidence.

The goal is not to claim that AI-generated code is perfect. It is to determine whether the implementation has passed documented, repeatable, evidence-based verification.

Use this standard to assess:

- Document/implementation consistency
- Debuggability, build reproducibility, and test quality
- Security boundaries and attack surfaces
- Bidirectional document/source synchronization
- Whether undocumented implementation intent can be recovered into documentation
- Re-audits and Phase gates
- Final human-operated whole-project audits

## 1. Cooperative Audit Philosophy

An audit is not adversarial policing. It is a cooperative quality gate aligning implementation AI, design AI, audit AI, and the human operator toward completing the same project.

- Write actionable findings without blaming the implementer.
- Align the implementation with documents, code, tests, and execution results rather than defending it against findings.
- Classify discrepancies by evidence rather than competing claims.
- When intent is unclear, use `Needs Spec Clarification`; do not guess.
- Reduce uncertainty rather than assign blame.
- Explain what needs to be aligned and which evidence is required, not merely that something is wrong.

## 2. Required Inputs

Inspect these inputs where available. Record absent documents under `Excluded Scope` or `Missing Inputs`.

| Input | Audit Use |
| --- | --- |
| `spec.md` | Requirements, scope, security requirements, acceptance criteria |
| `designs.md` | UX, architecture, deferred behavior, design intent |
| `implementation_summary.md` | Implementation claims, file responsibilities, completion scope |
| `DESIGN_DECISIONS.md` | Decision history, trade-offs, authority |
| ADR documents | Long-term design decisions and supersession relationships |
| `BUILD_GUIDE.md` | Builds, packaging, artifacts, execution paths |
| `audit_roadmap.md` | Audit plan, Phase gates, known risks |
| `CHANGELOG.md` | Change claims and release scope |
| `README.md` | User-facing features, commands, operating guidance |
| `LESSONS_LEARNED.md` | Past failures and regression risks |
| `kanban_board.md` | Work status and unfinished items |
| Source code | Actual implementation, call paths, runtime defaults |
| Tests | Deterministic verification, regression tests, security tests |
| Config files | Environment, runtime configuration, feature flags, launch modes |
| Dependency manifests | Package versions and optional/lazy/override drift |
| CI/CD files | Quality gates, scanners, release paths |
| Security documents | Hard boundaries, heuristics, accepted risks |
| Environment examples | Secret defaults, bind addresses, local/remote modes |

## 3. Required Outputs

Produce one integrated report, separating findings by the three audit passes.

Required contents:

- Audit summary
- Audit scope
- Excluded scope
- Documents inspected
- Files inspected
- Cases checked
- Cases excluded
- Findings by pass
- Cross-pass conflicts
- `Status`
- `Severity`
- Evidence
- Expected / actual behavior
- Suggested fix
- Re-audit method
- Accepted risks
- `Needs Spec Clarification` items
- Decision rationale
- Remaining risks
- Re-audit conditions
- Final decision

Recommended report structure:

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
## 12. Coder Handoff
```

### 3.1 Audit Report Storage and Coder Handoff

- Save the report to `{project_root}/docs/audit/audit_report_[turn].md`.
- `[turn]` is the next audit number that does not conflict with an existing report in that project.
- During the audit, do not directly modify source code, tests, configuration, or implementation documents. Record required changes in findings and the audit report.
- Where available, give each finding evidence, expected and actual states, a correction direction, and a re-audit method.
- After the audit, include a short correction prompt in a code block in the final response, with the latest report's path, ready for the user to paste into a separate coder agent. Putting it only inside the report is insufficient.
- The handoff prompt directly references the report just produced and asks the coder to compare findings with project documents and actual code before making necessary changes.
- In the report's `Coder Handoff`, use the actual report path instead of `[turn]` wherever possible.

Example Coder Handoff Prompt:

```text
Read the latest audit findings in `{project_root}/docs/audit/audit_report_[turn].md`.
Compare each finding with the relevant project documents and actual code, then make the necessary corrections.
If a contract must change, update its documents first. After correcting the implementation, run relevant tests and verification and record the results.
```

The report is evidence for verification and handoff, not an unconditional change order. The coder compares findings with the current user request, governing project documents, actual code, and evidence rather than treating findings as a new absolute authority.

## 4. Audit Execution Model: 3-Pass Audit

Use `AI_AUDIT_DOC_STANDARD.md` as the single audit-standard reference, but conduct the audit in three independent passes.

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

Each pass has a separate objective. Integrate the final report while keeping pass results distinct.

| Pass | Primary Question | Primary Inputs | Primary Output |
| --- | --- | --- | --- |
| Pass 1: Implementation Compliance | Does the implementation match the documents? | Spec, design, implementation summary, source, tests | Document/implementation drift, Phase-gate and reverse-documentation decisions |
| Pass 2: Debug / Engineering Quality | Is the code stable and reproducible? | Source, tests, build documents, configuration, dependencies, CI | Build/execution/test/architecture findings |
| Pass 3: Security | Are attack surfaces and security boundaries safe? | Security documents, authentication code, configuration, network, filesystem, shell, scanner | Security findings and gate conditions |

Priority questions:

- Pass 1: Are documented features, types, APIs, states, UI, and security requirements implemented in code and tests?
- Pass 2: Can startup, builds, dependencies, runtime configuration, tests, and complex state logic be verified deterministically?
- Pass 3: Are secrets, authentication/authorization, inputs, file/path/shell operations, network exposure, and CORS/WebView/Electron/Tauri/Agent boundaries safe?

## 5. Audit Scope Inventory Rules

Start with a scope inventory. When scope is unclear, address it as `Needs Spec Clarification` before making findings about that scope.

Required records:

- Project path and project type
- Languages, frameworks, and runtimes
- Key documents and missing documents
- Source directories
- Test directories
- Config files
- Dependency manifests
- CI/CD files
- Security documents
- Build/run documents
- Excluded scope
- Generated/vendor/reference-only trees
- Commands run for the audit, including failed commands

Do not silently omit excluded scope. Record why `node_modules`, `dist`, `target`, `.git`, reference corpora, generated output, or permission-denied paths are excluded.

## 6. Finding Format

Make each finding an actionable unit. Do not mix issues from different passes into one finding; link cross-pass conflicts through a separate finding.

```md
## [Finding ID] Title

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

Recommended finding IDs:

- `IMP-F001`
- `DBG-F001`
- `SEC-F001`
- `XPF-F001`
- Preserve the original finding ID in re-audits and append `Re-audit #1`, `Re-audit #2`, etc.

## 7. Status Rules

| Status | Meaning | Use Condition |
| --- | --- | --- |
| `Verified` | Passed audit criteria | Documents, code, tests, and execution evidence support the same conclusion |
| `Needs Fix` | Code, tests, configuration, builds, or security boundaries need correction | Requirements are clear, but actual artifacts disagree |
| `Needs Documentation Recovery` | Existing implementation intent needs to be recovered into documents | At least two independent sources support the existing intent |
| `Needs Spec Clarification` | The specification is insufficient for a decision | Requirements, Phase, authority, or security boundaries are unclear |
| `Accepted Risk` | Risk is understood and explicitly accepted | Owner, rationale, expiry conditions, and review conditions are recorded |
| `Deferred to Later Phase` | Moved outside the current Phase | A later-Phase document and tracking ID exist |
| `Rejected as False Positive` | The finding does not match the evidence | Counterevidence and rejection rationale are recorded |
| `Hold` | Stops passage, deployment, or the next Phase | Critical issues, unclear hard boundaries, scope conflicts, or unverifiable conditions exist |

`Accepted Risk` is not a silent waiver. Without a risk description, impact scope, owner, and review conditions, the status is `Hold`, not `Accepted Risk`.

## 8. Severity Rules

| Severity | Meaning | Default Gate Impact |
| --- | --- | --- |
| `Critical` | Data corruption, remote control, authentication bypass, secret exposure, or deployment-blocking failure | No PASS; immediate correction or Hold |
| `Major` | Significant feature/security/build/document-authority discrepancy | Correction or explicit Accepted Risk required |
| `Minor` | Local drift or low-risk document/test omission | May become a Known Issue or move to a later Phase |
| `Info` | Audit notes, improvement opportunities, traceability information | Does not block a gate; reference for follow-up work |

Potential Critical or Major security findings:

- Authentication/authorization bypass
- Plaintext exposure of secrets, tokens, or passwords
- Unauthenticated non-loopback exposure
- Command injection
- Path traversal
- Arbitrary file access
- Development bypasses remaining in deployment builds
- Permissive CORS combined with network binding
- Renderer trust-boundary violations in WebView/Electron/Tauri
- Overstated hard boundaries in security documents

## 9. Phase Gate Rules

Phase-passage criteria:

- No passage while a `Critical` finding remains.
- A `Major` finding requires correction or explicit `Accepted Risk` treatment.
- An unclear security hard boundary means `Hold`.
- If unclear documentation prevents implementation/audit judgment, use `Needs Spec Clarification`.
- Explicitly move remaining `Minor` findings to `Known Issues` or a later Phase.
- Link re-audit resolution of existing findings to their original IDs.
- Passing Pass 1 is not an overall PASS while Pass 3 security findings remain.
- If Pass 2 build/dependency problems remain, do not interpret a Pass 1 implementation-compliance PASS as deployability.
- Classify out-of-Phase implementation by reachability, documented approval, and test coverage.

Final decisions:

- `PASS`: No Critical/Major findings; all pass gate conditions met.
- `PASS WITH KNOWN RISKS`: Remaining risks are explicitly documented as Accepted Risk.
- `HOLD`: Security/specification/build/Phase gates are unclear or blocking findings exist.
- `REWORK REQUIRED`: Repeated failure is expected without structural rework.

## 10. Audit Stage 1: Implementation Compliance

Pass 1 assesses bidirectional consistency between documents and implementation.

Key questions:

- Do documented features, types, APIs, states, UI, and security requirements exist in code?
- Are features, types, APIs, states, UI, and security decisions present in code recorded in the documents?
- Does reachable implementation match the current Phase scope?
- Do completion claims include file responsibilities and deterministic verification criteria?
- Do secondary summaries contradict the current source?
- Do not pass a feature, state, datum, or event merely because it is created or called. Trace its actual consumption and subsequent changes in state, output, or behavior to verify effective system integration.
- Identify orphan elements that are created but not consumed, references with no substantive effect, and broken causal paths. Record findings unless those elements are intentionally independent.

Priority patterns:

- `IMP-001`
- `IMP-002`
- `IMP-003`
- `IMP-004`
- `DOC-BACKFILL-001`
- `SPEC-GAP-001`

Outputs:

- Implementation-drift findings
- Missing-documentation findings
- Orphan/unauthorized-implementation findings
- Reverse-documentation recovery decisions
- Phase-gate decisions

## 11. Audit Stage 2: Debug / Engineering Quality

Pass 2 assesses stability, reproducibility, and maintainability.

Key questions:

- Is the startup chain verified through paths, migrations, configuration, locale, and service wiring?
- Do build guidance, actual artifacts, and runtime path assumptions agree?
- Are dependency declarations consistent across eager/lazy/optional/override paths?
- Do tests lock down specific failure-mode regressions beyond broad smoke coverage?
- Can complex state/AI logic be reproduced through headless paths, fixtures, schemas, hashes, and seeds?
- Does environment/runtime configuration have a single control point?
- Do documentation, backend, and frontend share one source of truth for local API ports and paths?

Priority patterns:

- `DBG-001`
- `BUILD-001`
- `DEP-001`
- `TEST-001`
- `DBG-002`
- `ARCH-001`
- `ARCH-002`

Outputs:

- Build/run-drift findings
- Dependency-parity findings
- Missing-regression findings
- Deterministic-debugging-interface findings
- Architecture control-point findings

## 12. Audit Stage 3: Security

Pass 3 assesses whether attack surfaces and security boundaries are enforced to the same strength in documents, code, and tests.

Key questions:

- Are secrets constrained to reference-based or encrypted storage?
- Does code prevent development bypass flags from remaining in deployment paths?
- Do non-loopback binding, browser control, and remote mutation enforce authentication and allowlists?
- Are path, workspace, shell, and file-mode boundaries verified as independent controls?
- Do security documents distinguish heuristics from hard boundaries?
- Do scanner results record both shipped scope and rule provenance?
- Are local desktop APIs audited as network services?
- If documents prohibit `innerHTML`, does the renderer enforce the same restriction?

Priority patterns:

- `SEC-001`
- `SEC-002`
- `SEC-003`
- `SEC-004`
- `SEC-005`
- `SEC-006`
- `SEC-007`
- `SEC-008`

Outputs:

- Security-boundary findings
- Network-exposure findings
- Secret-storage findings
- Development-bypass findings
- Scanner/supply-chain findings
- Overclaim findings in security documents

## 13. Cross-Pass Conflict Resolution

Passes may reach conflicting judgments. Record the conflict under `Cross-Pass Conflicts` in the final report instead of discarding either judgment.

Example:

```md
Implementation Pass:
The local API is an intended, documented feature.

Security Pass:
However, 0.0.0.0 binding + permissive CORS + no authentication is unsafe.
```

Resolution rules:

- Even intended, documented behavior is `Needs Fix` or `Hold` when its security boundary is weak.
- If unsafe implementation is specified in the documents, correct the documents too.
- An Implementation-pass PASS does not imply a Security-pass PASS.
- Security findings are independent gate conditions, separate from implementation consistency.
- A Debug-pass build failure prevents an Implementation-pass finding that a feature exists from being interpreted as deployability.
- Record affected finding IDs with each cross-pass conflict.

Minimum cross-pass finding format:

```md
## [XPF-F001] Title

- Related Findings: IMP-F001, SEC-F002
- Conflict:
- Resolution:
- Gate Impact:
- Required Fix Before PASS:
```

## 14. Bidirectional Document-Code Sync

Audit document/source consistency in both directions.

### Forward Sync

Check that documented features, types, APIs, states, UI, and security requirements are implemented in code.

Potential findings:

- A feature is documented as complete but has no code.
- UI is documented as complete, but only domain tests exist.
- A documented deployment artifact is absent from the package.
- A claimed security hard boundary is only a warning or heuristic in code.

### Backward Sync

Check that features, types, APIs, states, UI, and security decisions in code are recorded in the documents.

Do not immediately label undocumented implementation as erroneous. Classify it first:

| Classification | Meaning | Typical Status |
| --- | --- | --- |
| `Intentional but Undocumented` | Appears to implement an existing decision, but documentation is missing | `Needs Documentation Recovery` |
| `Accidental / Orphan Code` | Unreachable code or remnants unrelated to current scope | `Needs Fix` |
| `Design Drift` | Code has moved away from the established design | `Needs Fix` or `Needs Spec Clarification` |
| `Spec Gap` | Substantial functionality lacks governing documentation | `Needs Spec Clarification` |
| `Unauthorized Scope Expansion` | Reachable behavior exceeds the current Phase or approved scope | `Hold` or `Needs Fix` |

## 15. Reverse Documentation Sync

Recover existing implementation facts into documentation. Do not document every code-only fact indiscriminately.

Allowed when:

- At least two independent sources among source code, comments, tests, existing ADRs, `spec.md`, and `designs.md` support the same intent.
- No new requirement is invented.
- The implementation is traceable to an existing decision.
- Terms, scope, Phase, and tests agree with the existing document system.
- Recovered documents distinguish already implemented facts from requirements still needing a decision.

Prohibited:

- Establishing intent from one code fragment alone
- Inventing product requirements from test fixtures alone
- Promoting current code over a conflicting old summary without supporting evidence
- Justifying unsafe behavior merely because it has been documented

If intent is unclear, use `Needs Spec Clarification`.

## 16. Re-audit Rules

Re-audits cover more than changed files: revisit related documents, tests, configuration, build paths, and security boundaries.

Common rules:

- Verify that original findings are actually resolved.
- Check that corrections introduced no new drift.
- For security findings, check whether the same vulnerability pattern remains elsewhere.
- For documentation-recovery findings, check consistency with source and existing decisions.
- Link re-audit results to the original finding IDs.
- Repeating all three passes every time is not required, but the passes connected to changed areas must be repeated.

Re-audit mapping:

| Fixed Finding Type | Required Re-audit |
| --- | --- |
| Document/implementation finding | Pass 1; relevant parts of Pass 2 if builds/tests are affected, and Pass 3 if security boundaries are affected |
| Debug/build finding | Pass 2; relevant parts of Pass 1 if documented commands change, and Pass 3 if deployment/security settings change |
| Security finding | Pass 3; relevant parts of Pass 2 if configuration/build/deployment commands change, and Pass 1 if security documents change |
| Documentation recovery finding | Pass 1; relevant parts of connected passes if recovered documents change test/security boundaries |

Record in each re-audit finding:

- Original finding ID
- Changed documents/files
- Tests/commands re-run
- Passing evidence
- Remaining risks
- New findings

## 17. Repeated Failure Diagnosis

When the same finding or finding type recurs, stop routine rework and switch to focused diagnosis.

Classify repeated failure:

- Ambiguous documents
- Incorrect implementation structure
- Inadequate tests
- Conflicting audit criteria
- Undesigned security boundaries
- Overly broad AI correction instructions
- Excessive Phase scope

Diagnosis outcomes:

| Decision | Use Condition |
| --- | --- |
| `Continue` | The cause is local and the correction direction is clear |
| `Refactor` | The structure can remain, but responsibilities need separation |
| `Rewrite Module` | The current structure repeatedly violates requirements |
| `Revise Spec` | The specification must change before implementation |
| `Split Phase` | Phase scope is too large for coherent verification units |
| `Human Review Required` | Security, product direction, or legal/operational risk needs human judgment |

## 18. Final Whole-Project Audit

For important projects, a human may run final cross-audits using multiple models or CLIs, such as GPT, Gemini, Claude, or Codex.

When to perform this audit:

- Before deployment
- Before a major version
- After security/authorization/persistence/network changes
- When the user designates the project as important
- When repeated-failure diagnosis yields `Human Review Required`

Classify cross-audit results:

- `Common Finding`: Two or more auditors identify the same issue.
- `Single-Model Finding`: Only one auditor identifies the issue, but evidence supports it.
- `Rejected Finding`: Evidence refutes the issue.
- `Accepted Risk`: Risk is explicitly accepted.

Final decisions:

- `PASS`
- `PASS WITH KNOWN RISKS`
- `HOLD`
- `REWORK REQUIRED`

In a final human-operated whole-project audit, one model's PASS is not an overall PASS. Prioritize common findings and security findings as gates.

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

- `IMP-001`: Check domain, UI, and deployment-artifact contracts separately.
- `IMP-002`: Classify out-of-Phase implementation using documented intent and call paths.
- `IMP-003`: Support completion claims with file responsibilities and deterministic verification criteria.
- `IMP-004`: Audit secondary summaries independently; they become stale faster than source.

### Debug / Engineering Quality

- `DBG-001`: Audit startup as a complete chain, not isolated features.
- `DBG-002`: Make complex state/AI logic deterministically reproducible through debugging interfaces.
- `ARCH-001`: Centralize environment and runtime configuration in one control point.
- `ARCH-002`: Give documents, backend defaults, and frontend callers one source of truth for local API ports and paths.

### Build / Dependency / Test

- `BUILD-001`: Compare build guidance with actual execution paths and artifacts.
- `DEP-001`: Check dependency-declaration parity across eager/lazy/override paths.
- `TEST-001`: Prioritize explicit failure-mode regression tests over broad smoke coverage alone.

### Security

- `SEC-001`: Constrain secrets to reference-based or encrypted storage.
- `SEC-002`: Separately re-audit development bypasses and local defaults before deployment.
- `SEC-003`: No PASS for non-loopback exposure without authentication and allowlists.
- `SEC-004`: Audit path, workspace, and shell-execution boundaries as separate controls.
- `SEC-005`: Do not overstate implemented security boundaries.
- `SEC-006`: Verify scanner shipped scope and rule provenance together.
- `SEC-007`: Audit local desktop API exposure as a network service.
- `SEC-008`: Enforce documented `innerHTML` restrictions in the renderer.

### Document Recovery / Spec Gap

- `DOC-BACKFILL-001`: Recover code-only facts only with support from existing decisions and tests.
- `SPEC-GAP-001`: Isolate substantial undocumented functionality for specification work.

## 21. Pattern Catalog

### Implementation Compliance Patterns

#### [IMP-001] Check Domain, UI, and Deployment Contracts Separately

- Pass:
  - Implementation
- Area: Document/implementation layer consistency
- Applies To: Web, Desktop, Agent, Backend
- Audit Question: Must this requirement be complete at the domain-contract, user-facing UI, or deployment-artifact layer?
- Inspection Targets: `spec.md`, `designs.md`, `implementation_summary.md`, runtime entrypoint, package/build scripts, acceptance tests
- PASS Condition: Documents identify completed layers, and code and tests match that layer scope.
- Needs Fix Condition: Documents describe shipped behavior, but only domain or test-level implementation exists.
- Needs Spec Clarification Condition: Documents do not distinguish domain completion from UI/deployment completion, so shipped status cannot be determined.
- Suggested Fix Instruction: Separate domain, UI, and deployment completion in the specification; link each layer to verification files and artifact paths.
- Re-audit Method: Compare layer declarations, relevant tests, and actual packages or runtime paths again.
- False Positive Notes: Staged delivery is acceptable when deferred and shipped scope are distinguished in the documents.
- Source Evidence Summary: In `chitchat`, provider-capability contracts were complete at domain/test level, while SPA hiding and packaged frontend/migration assets remained separate layers.

#### [IMP-002] Classify Out-of-Phase Implementation by Intent and Call Paths

- Pass:
  - Implementation
- Area: Phase scope, reachable implementation, scope control
- Applies To: Game, Web, Desktop, CLI
- Audit Question: Does this code belong to the current Phase, anticipate the next Phase, lack documentation, or remain as obsolete code?
- Inspection Targets: `spec.md`, `audit_roadmap.md`, `implementation_summary.md`, source entrypoints, call graph, tests
- PASS Condition: Code matches current Phase scope, or early implementation/later-Phase plans are documented.
- Needs Fix Condition: Current non-goals or later-Phase features are reachable without evidence of documented approval.
- Needs Spec Clarification Condition: Documents provide no basis for judging current Phase scope or permitted early implementation.
- Suggested Fix Instruction: Classify as `Intentional but Undocumented`, `Accidental / Orphan Code`, `Design Drift`, `Spec Gap`, or `Unauthorized Scope Expansion`, then align documents or code.
- Re-audit Method: Recompare Phase documents, call paths, and test scope to check the classification.
- False Positive Notes: Early data schemas or UI scaffolding may be acceptable when unreachable and documented.
- Source Evidence Summary: `LazyRoomLife` runtime engine/storage/renderer implementation exceeded early-Phase documents; `AIHack` provided a contrasting example with closely linked Phase claims and deterministic tests.

#### [IMP-003] Support Completion Claims with File Responsibilities and Deterministic Criteria

- Pass:
  - Implementation
- Area: Completion claims, verification authority
- Applies To: Game, Agent, Backend, Desktop
- Audit Question: Are responsibility mappings, tests, hashes, schemas, or diagnostics available to reproduce this completion claim against the code?
- Inspection Targets: `implementation_summary.md`, release gates, deterministic tests, doctor/health checks, changelog
- PASS Condition: Every completion claim links to responsible files, verification commands, and deterministic criteria.
- Needs Fix Condition: Completion is claimed without file responsibilities or reproducible verification criteria.
- Needs Spec Clarification Condition: Completion criteria are qualitative only, with no measurable termination conditions.
- Suggested Fix Instruction: Put responsible files, verification commands, and reference fixtures or baselines next to completion claims.
- Re-audit Method: Re-run baseline tests or doctor checks and compare results with completion documents.
- False Positive Notes: Even initial scaffolding needs termination criteria and owned-file records.
- Source Evidence Summary: `AIHack` anchored completion through Phase contracts, release-candidate tests, and save/load/schema/UI/monster-AI tests; `openclaw` supported operational claims with doctor checks.

#### [IMP-004] Audit Secondary Summaries Independently for Staleness

- Pass:
  - Implementation
- Area: Secondary documents, generated documents, stale summaries
- Applies To: Web, Desktop, Game, Agent
- Audit Question: Does this supporting document reflect current source and execution paths, or preserve a past-stage snapshot?
- Inspection Targets: `implementation_summary.md`, generated document data, source modules, runtime defaults, changelog
- PASS Condition: The summary matches current source or identifies its snapshot time and limitations.
- Needs Fix Condition: The summary contradicts current source while appearing current.
- Needs Spec Clarification Condition: Primary authority and supporting summaries are not distinguished.
- Suggested Fix Instruction: State the summary's reference time and authority; synchronize contradictions with source.
- Re-audit Method: Recompare summary/document data with source entrypoints, defaults, and feature flags.
- False Positive Notes: Historical milestone records may remain, but warrant a finding if linked as current-state documents.
- Source Evidence Summary: The `TypeTris` summary marked implemented features as incomplete; `smaLLMRust` document data showed API-port drift.

### Debug / Engineering Quality Patterns

#### [DBG-001] Audit Startup as a Complete Chain

- Pass:
  - Debug
- Area: Startup chain, initialization order
- Applies To: Web, Desktop, Agent, Backend, CLI
- Audit Question: In what order are paths, migrations, configuration, localization, and services connected at startup?
- Inspection Targets: Runtime entrypoint, config loader, migration code, i18n bootstrap, startup smoke tests
- PASS Condition: The startup chain is explicit in documents or code, with tests or smoke checks for failure points.
- Needs Fix Condition: Initialization order is implicit, or one stage's failure only surfaces much later.
- Needs Spec Clarification Condition: Startup responsibilities and order are undocumented, so required initialization cannot be determined.
- Suggested Fix Instruction: Make startup stages explicit and add their failure conditions and smoke checks.
- Re-audit Method: Recheck startup with empty data directories, corrupt configuration, missing locales, and required migrations.
- False Positive Notes: Applies to a single-binary CLI if it has a configuration/path initialization chain.
- Source Evidence Summary: `chitchat` startup connected app directory, legacy migration, DB migration, preferences, translator, and service wiring.

#### [DBG-002] Provide Deterministic Debugging for Complex State and AI

- Pass:
  - Debug
- Area: Deterministic debugging, state machines, AI policy
- Applies To: Game, Agent, Backend, Desktop
- Audit Question: Can headless paths, schemas, hashes, fixtures, or seeds reproduce this complex logic without UI?
- Inspection Targets: Headless binaries, schema tests, snapshot hashes, fixture loaders, deterministic seeds
- PASS Condition: Core logic has a deterministic verification path independent of UI.
- Needs Fix Condition: No interface reproduces results from the same input, or verification relies only on manual UI testing.
- Needs Spec Clarification Condition: Documents do not identify which state must remain deterministic.
- Suggested Fix Instruction: Add one or more headless entrypoints, schema fixtures, hash baselines, or deterministic-seed checks.
- Re-audit Method: Verify identical results for the same fixture/seed and preserved invariants after save/restore.
- False Positive Notes: Evaluate external calls using mocks or recorded fixtures.
- Source Evidence Summary: `AIHack` made complex logic reproducible through a headless binary, save/load hashes, schema tests, and monster-AI seed/policy tests.

#### [ARCH-001] Centralize Environment and Runtime Configuration

- Pass:
  - Debug
- Area: Configuration architecture, environment handling
- Applies To: Web, Desktop, Backend, Agent
- Audit Question: Does one module interpret environment/runtime configuration, or do multiple screens/services read it directly?
- Inspection Targets: Configuration modules, direct `process.env` or equivalent reads, test bootstrap, configuration documents
- PASS Condition: Configuration interpretation is centralized and controllable through a test seam or mock.
- Needs Fix Condition: Multiple modules directly read the environment with inconsistent fallbacks.
- Needs Spec Clarification Condition: Documents do not distinguish build-time from runtime settings.
- Suggested Fix Instruction: Put direct environment reads behind a shared configuration module with a testable lookup seam.
- Re-audit Method: Search for direct environment reads and compare configuration-module call paths.
- False Positive Notes: Tiny scripts may be exceptions, but drift accumulates quickly across multiple runtime interfaces.
- Source Evidence Summary: `openhuman` separated runtime-authoritative configuration and test seams in frontend configuration and the Rust configuration loader.

#### [ARCH-002] Share One Source of Truth for Local API Ports and Paths

- Pass:
  - Debug
- Area: Local API routes and source of truth
- Applies To: Desktop, Agent, Backend, Web
- Audit Question: What is the canonical source for API addresses, ports, and WebSocket paths, and do all callers reference it?
- Inspection Targets: API documents, document-data generators, backend router registration, frontend URL helpers, configuration defaults
- PASS Condition: Ports and paths derive from one configuration or generator; documents and callers use the same values.
- Needs Fix Condition: Documents and callers, or handlers and router registration, use different addresses.
- Needs Spec Clarification Condition: Documents do not establish the canonical API execution mode.
- Suggested Fix Instruction: Consolidate the API base URL and route map in one configuration/source module and generate documentation from the same values.
- Re-audit Method: Recompare API documents, backend binding/router, frontend helpers, and smoke requests.
- False Positive Notes: Development/deployment ports may differ with a documented source of truth and switching rules per mode.
- Source Evidence Summary: `smaLLMRust` documents used `3001`, while backend/frontend centered on `8080`; WebSocket registration also showed drift risk.

### Security Patterns

#### [SEC-001] Constrain Secrets to Reference-Based or Encrypted Storage

- Pass:
  - Security
- Area: Secret storage, configuration persistence
- Applies To: Web, Desktop, Agent, Backend
- Audit Question: Where are actual secret values stored, and which references or ciphertext remain in application data?
- Inspection Targets: Specification/security documents, secret-storage module, configuration persistence code, DB models, tests
- PASS Condition: Documents, code, and tests describe the same secret boundary, with no plaintext secrets left in models or the DB.
- Needs Fix Condition: Secrets are stored in plaintext, or documents and code assume different storage boundaries.
- Needs Spec Clarification Condition: Documents do not identify the secret storage source of truth.
- Suggested Fix Instruction: Establish one secret-storage policy and add tests for reference-based or encrypted storage.
- Re-audit Method: Recheck DB/configuration files, the secret module, and related test assertions.
- False Positive Notes: A local single-user application still needs secret-boundary verification; being local is not grounds for PASS.
- Source Evidence Summary: `chitchat` used `secret_ref` and OS-keyring tests; `openhuman` implemented encrypted storage, backup, atomic rename, and rollback.

#### [SEC-002] Re-audit Development Bypasses and Local Defaults Before Deployment

- Pass:
  - Security
- Area: Development bypasses, release-mode isolation
- Applies To: Desktop, Web, Agent
- Audit Question: Are these bypass flags or local defaults confined to development and removed or blocked in deployment builds?
- Inspection Targets: Manifests, README/build-guide caveats, feature flags, CORS configuration, Electron/Tauri launch scripts
- PASS Condition: Code and documents identify development-only settings, and deployment paths automatically block them.
- Needs Fix Condition: Development bypasses reach deployment paths, or the boundary exists only in documentation.
- Needs Spec Clarification Condition: Documents do not distinguish development from deployment modes.
- Suggested Fix Instruction: Put development-only flags behind an explicit feature/build mode and fail when used in deployment paths.
- Re-audit Method: Recheck development/deployment commands, feature gates, and runtime warnings or blocking behavior.
- False Positive Notes: Development bypasses may exist, but cannot pass without isolation and re-audit procedures.
- Source Evidence Summary: Examples include `chitchat` local CORS, `TypeTris` Electron `--no-sandbox`, and `openhuman` destructive E2E features disabled by default.

#### [SEC-003] Require Authentication and Allowlists for Non-Loopback Exposure

- Pass:
  - Security
- Area: Network binding, remote control, allowlists
- Applies To: Agent, Backend, Desktop with local server
- Audit Question: Are authentication, allowlists, and mutation guards enforced when binding beyond loopback or exposing remote control?
- Inspection Targets: Bind logic, server startup guards, authentication middleware, allowlist configuration, tests
- PASS Condition: Non-loopback authentication and allowlists are enforced in code and covered by tests.
- Needs Fix Condition: Network exposure is possible, but authentication or allowlists are missing or warning-only.
- Needs Spec Clarification Condition: Documents do not establish whether the project is local-only or permits remote exposure.
- Suggested Fix Instruction: Require authentication and allowlist checks for non-loopback binding; fail startup when unconfigured.
- Re-audit Method: Recheck bind-guard tests, startup paths, and configuration defaults.
- False Positive Notes: Prioritize this pattern even for air-gapped local-only tools when wildcard binding or browser-control interfaces exist.
- Source Evidence Summary: `hermes-agent` tested non-loopback/API-key/allowlist guards; `openclaw` implemented browser-control authentication and CSRF-style mutation protection.

#### [SEC-004] Audit Path, Workspace, and Shell Boundaries Independently

- Pass:
  - Security
- Area: Path traversal, workspace trust, shell execution
- Applies To: Agent, CLI, Desktop, Backend
- Audit Question: Are path validation, workspace boundaries, command approval, and file permissions independently controlled?
- Inspection Targets: Path-validation helpers, command-approval modules, workspace-traversal tests, permission tests, security documents
- PASS Condition: Each control has separate code and tests; success in one does not conceal failure in another.
- Needs Fix Condition: Path/shell execution relies only on ad hoc string checks, or tests do not directly cover individual controls.
- Needs Spec Clarification Condition: Documents do not define which file/command boundaries lie outside the trusted area.
- Suggested Fix Instruction: Separate path, workspace, shell, and file-mode checks into helpers and regression tests.
- Re-audit Method: Recheck traversal, dangerous-command, and permission regression tests.
- False Positive Notes: Apply proactively to read-only tools when future tool execution is planned.
- Source Evidence Summary: `hermes-agent` separated path security, approval, worktree-traversal rejection, and secure-file-mode tests; `openclaw` separated operator trust from code guards.

#### [SEC-005] Do Not Overstate Implemented Security Boundaries

- Pass:
  - Security
- Area: Security documentation, hard boundaries, heuristics
- Applies To: Agent, Backend, Desktop
- Audit Question: Does the documented security boundary have the same enforcement strength in code?
- Inspection Targets: `SECURITY.md`, authentication/boundary code, redaction modules, tests, startup warnings
- PASS Condition: Documents distinguish heuristics, policy, and hard boundaries; code and tests support hard-boundary claims.
- Needs Fix Condition: Documents exaggerate an in-process heuristic as a sandbox or isolation boundary.
- Needs Spec Clarification Condition: Documents do not distinguish advisory from mandatory protection.
- Suggested Fix Instruction: Distinguish hard boundaries from heuristics in security documents and link each hard boundary to its implementing code.
- Re-audit Method: Compare SECURITY wording with actual enforcement code and tests side by side.
- False Positive Notes: Marketing language in a README is auditable when it reads as operational security guidance.
- Source Evidence Summary: `hermes-agent` distinguished OS isolation from heuristic guards; `openclaw` connected operator-trust policy to concrete authentication/CSRF/redaction code.

#### [SEC-006] Verify Scanner Shipped Scope and Rule Provenance Together

- Pass:
  - Security
- Area: Scanner scope, supply chain, rule provenance
- Applies To: Monorepo, Backend, Agent, Web
- Audit Question: Does this scanner finding affect shipped product scope, and are the rule's origin and owner traceable?
- Inspection Targets: Scanner ignore files, rule-metadata checks, pre-commit hooks, CI workflows, finding paths
- PASS Condition: Shipped and non-shipped scope are separated, and rule provenance is verified.
- Needs Fix Condition: Non-product paths receive the same risk classification, or absent rule metadata prevents evidence-based triage.
- Needs Spec Clarification Condition: Documents and scanner settings do not define deployment scope.
- Suggested Fix Instruction: Add scanner-scope files and rule-metadata verification; record whether each finding affects shipped scope.
- Re-audit Method: Recheck ignore scope, metadata checks, and sample-finding triage.
- False Positive Notes: QA-only code may have problems without the same severity as a product vulnerability.
- Source Evidence Summary: `openclaw` used `.semgrepignore`, OpenGrep rule-metadata checks, pre-commit, and GitHub workflows for scanner hygiene.

#### [SEC-007] Audit Local Desktop API Exposure as a Network Service

- Pass:
  - Security
- Area: Desktop local API, CORS, private-network access
- Applies To: Desktop, Agent, Backend
- Audit Question: Which interfaces does the API actually bind to, and how much remote access do CORS and authentication defaults allow?
- Inspection Targets: Backend bind addresses, CORS middleware, authentication bootstrap, API-key defaults, local API documents
- PASS Condition: The API is restricted to loopback-oriented access, or non-loopback/unauthenticated/broad-CORS exposure is explicitly blocked.
- Needs Fix Condition: The default bind is external and permits permissive CORS or unauthenticated access.
- Needs Spec Clarification Condition: Product documents do not distinguish local-only operation from permitted LAN exposure.
- Suggested Fix Instruction: Default to loopback. Require explicit opt-in and authentication configuration for remote exposure; fail otherwise.
- Re-audit Method: Recheck bind addresses, CORS headers, no-key startup behavior, and remote-reachability smoke tests.
- False Positive Notes: Treat `0.0.0.0` with permissive CORS as risky even in research tools.
- Source Evidence Summary: `smaLLMRust` combined `0.0.0.0`, permissive CORS, a private-network header, and no-key open access.

#### [SEC-008] Enforce Documented `innerHTML` Restrictions in the Renderer

- Pass:
  - Security
- Area: DOM injection, renderer policy
- Applies To: Web, Desktop WebView, Agent UI
- Audit Question: Does the renderer still use DOM-injection patterns prohibited in the documents?
- Inspection Targets: Specification security section, audit roadmap, renderer/view code, templating helpers
- PASS Condition: Prohibited DOM insertion is removed or protected by strict sanitization and an explicit allowed-content policy.
- Needs Fix Condition: Despite the documented prohibition, code still uses `innerHTML` or equivalent unsanitized DOM insertion.
- Needs Spec Clarification Condition: Documents mention an XSS boundary without defining permitted HTML sources or sanitization policy.
- Suggested Fix Instruction: Use `textContent`, DOM-node construction, or explicit sanitizer-based rendering; align code and documents.
- Re-audit Method: Search renderer code for `innerHTML` and check whether paths can extend to user-controlled data sources.
- False Positive Notes: Do not leave prohibited patterns merely because data is currently internal if external input may arrive later.
- Source Evidence Summary: `LazyRoomLife` specification/audit-roadmap documents restricted XSS and `innerHTML`, but the renderer still used `innerHTML`.

### Build / Dependency / Test Patterns

#### [BUILD-001] Compare Build Guidance with Actual Paths and Artifacts

- Pass:
  - Debug
- Area: Build guidance, artifact paths, runtime packaging
- Applies To: Web, Desktop, CLI, Agent
- Audit Question: Do documented build/run commands point to the actual scripts and artifact paths?
- Inspection Targets: `README.md`, `BUILD_GUIDE.md`, package manifests, build scripts, runtime artifact-path assumptions
- PASS Condition: Documented commands match current scripts, and artifact structure meets runtime path assumptions.
- Needs Fix Condition: README and BUILD_GUIDE require different procedures, or runtime expects paths absent from the artifacts.
- Needs Spec Clarification Condition: Deployment targets are unclear, so required build paths cannot be determined.
- Suggested Fix Instruction: Establish one source of truth for build guidance and document the artifact tree with runtime path dependencies.
- Re-audit Method: Perform a fresh build using only documented commands, then check actual execution paths and included assets.
- False Positive Notes: Development/release commands may differ if documents clearly explain the distinction.
- Source Evidence Summary: `chitchat`, `TypeTris`, and `smaLLMRust` repeatedly showed drift among build documents, scripts, packaged assets, and runtime paths.

#### [DEP-001] Check Dependency Parity Across Eager, Lazy, and Override Paths

- Pass:
  - Debug
- Area: Dependency parity, lazy installation, workspace policy
- Applies To: Agent, Backend, Monorepo, Desktop
- Audit Question: Do direct installation, optional extras, lazy installers, workspace overrides, and patch paths follow the same version policy?
- Inspection Targets: Dependency manifests, lazy-install maps, workspace policy files, packaging tests
- PASS Condition: Duplicate declarations agree on version and policy intent, or differences are justified in documents.
- Needs Fix Condition: Versions drift across installation paths without tests or documents to detect it.
- Needs Spec Clarification Condition: Documents do not identify the canonical installation path, so parity cannot be judged.
- Suggested Fix Instruction: Establish a canonical dependency source and automatically verify or generate other declarations from it.
- Re-audit Method: Recheck manifest diffs, lazy-install maps, and packaging tests together.
- False Positive Notes: Canary-pin differences are acceptable with documented reasons and scope.
- Source Evidence Summary: `hermes-agent` showed drift between eager extras and lazy-dependency pins; `openclaw` controlled dependencies through workspace overrides and release-age policy.

#### [TEST-001] Prioritize Explicit Failure-Mode Regression Tests

- Pass:
  - Debug
- Area: Regression tests, failure-mode evidence
- Applies To: All project types
- Audit Question: Does the repository preserve actual past failures as explicitly named regression tests?
- Inspection Targets: Test filenames, failure-specific assertions, changelog, inline rationale comments
- PASS Condition: Tests explicitly name and preserve specific failure modes and boundaries.
- Needs Fix Condition: Important regressions are hidden in broad smoke coverage, without names or assertions identifying the protected risk.
- Needs Spec Clarification Condition: Documents and test names do not reveal which failures the project prioritizes.
- Suggested Fix Instruction: Keep broad smoke coverage and add explicit regression tests for past bugs or boundaries.
- Re-audit Method: Check that new tests directly describe the failure mode rather than disappearing into broad smoke coverage.
- False Positive Notes: Even an early project can derive regression cases from incident history in changelogs or comments.
- Source Evidence Summary: `hermes-agent` kept separate worktree-security, SQL-injection, permission, and timezone regressions; `AIHack` deterministically tested monster AI, save/load, and UI geometry.

### Document Backfill / Spec Gap Patterns

#### [DOC-BACKFILL-001] Recover Code-Only Facts with Corroborating Decisions and Tests

- Pass:
  - Implementation
- Area: Reverse documentation, classification of code-only facts
- Applies To: Web, Agent, Backend, Build Tooling
- Audit Question: Do at least two sources among code, tests, and existing documents corroborate the same intent for this implementation fact?
- Inspection Targets: Source comments, tests, existing specification/design/README, build configuration, call sites
- PASS Condition: At least two independent sources support the same intent, and recovery does not change the established direction.
- Needs Fix Condition: The implementation is used repeatedly but undocumented, making it difficult for the next worker to trace.
- Needs Spec Clarification Condition: Code exists, but two or more sources cannot establish intent and scope.
- Suggested Fix Instruction: Trace implementation facts and supporting files into an existing document section without inventing requirements.
- Re-audit Method: Recheck consistency with existing terminology, call flow, and test assertions.
- False Positive Notes: A single comment or test is weak evidence for establishing intent.
- Source Evidence Summary: `chitchat` prompt-snapshot fields linked source, tests, and specification sections; `openclaw` build-boundary comments, harness, and README guidance supported the same intent.

#### [SPEC-GAP-001] Isolate Substantial Undocumented Functionality for Specification Work

- Pass:
  - Implementation
- Area: Specification gaps, missing governing documents
- Applies To: Desktop, Agent, Backend, Library
- Audit Question: Do a settled specification, Phase definitions, and file responsibilities govern the current implementation?
- Inspection Targets: Root documents, source-tree breadth, build guidance, README feature lists, configuration files
- PASS Condition: Specification/design/implementation-governance documents exist beyond README, or README structurally fulfills their roles.
- Needs Fix Condition: Features and source are extensive but lack an audit baseline, leaving their intended meaning unclear.
- Needs Spec Clarification Condition: Some documents exist, but they do not establish success criteria, scope, or responsibility boundaries.
- Suggested Fix Instruction: Add `spec.md` or an equivalent covering scope, core contracts, file responsibilities, and verification commands.
- Re-audit Method: Recompare the governing documents with actual source modules and build procedures.
- False Positive Notes: Experimental prototypes may have little documentation, but this pattern matters when they are exposed to external users.
- Source Evidence Summary: `smaLLMRust` had README/BUILD_GUIDE but lacked governing documents for its agent, MCP, workflow, plugin, and RAG interfaces.

## 22. Copy-Paste Instruction Block

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

## 23. Final Principle

The final question is not who is right, but what has been verified, by which evidence, and what remains uncertain.

When documents establish stronger requirements than the code fulfills, align the code. When code goes beyond the documents, classify the evidence. Do not pass unclear security boundaries. For repeated failures, redesign the criteria, structure, tests, or Phase instead of pushing harder.

A cooperative audit reduces uncertainty; it is not a punishment loop.
