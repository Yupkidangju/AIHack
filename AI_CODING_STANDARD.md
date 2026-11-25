# AI Coding Standard

**Draft Version:** 0.1

## 0. Purpose

This is a concise implementation reference for AI-generated or AI-modified code in D3D projects.

The goal is not to standardize implementation methods. It is to allow free implementation within documented contracts while enabling subsequent workers and auditors to trace intent and verification status through code, comments, tests, and execution evidence.

## 1. Document Contract

- Before changing code, review the current user request, relevant project documents (`spec.md`, `designs.md`, `DESIGN_DECISIONS.md`, `IMPLEMENTATION_SUMMARY.md`, etc.), current Phase, and roadmap.
- Implement after the current objective, scope, key decisions, implementation order, and verification criteria are documented at a depth proportional to the change under `AI_IMPLEMENTATION_DOC_STANDARD.md`.
- A plan existing only in conversation, `plan`, `grill-me`, or `goal` is not an implementation baseline.
- If the contract or roadmap changes during implementation, update the documents before continuing code changes.
- Do not silently mix in work that conflicts with the documents or exceeds current scope. Separate it into a document revision, an unconfirmed matter, or a follow-up proposal.

## 2. Implementation Autonomy

- Within the established contract, the coder chooses architectural details, algorithms, work order, tools, and decomposition not fixed by the documents.
- This standard does not impose a uniform pattern, framework, amount of commentary, or test format.
- Respect existing conventions, but change them with a recorded rationale when quality, safety, or contract consistency calls for a better approach. Update the documents first when the contract is affected.
- Separate unrequested large refactoring, optimization, and feature expansion from the current change.
- Prefer coherent changes with clear responsibilities and boundaries over a small diff for its own sake, without unnecessarily widening scope.

## 3. Traceable Code

- Make names, types, modules, and interfaces expose the documented domain vocabulary and responsibility boundaries.
- Keep state changes, external I/O, input validation, error conversion, configuration interpretation, and security boundaries traceable through the call flow.
- Distinguish enforced security boundaries from warnings, heuristics, and fallbacks in code.
- Do not create duplicate sources of truth for the same rule, configuration, or state.
- Minimize hidden side effects, implicit fallbacks, and unreachable out-of-scope implementation.
- For complex state, AI, or asynchronous logic, provide reproducible interfaces as needed: pure functions, fixtures, seeds, schemas, hashes, headless paths, or diagnostic tools.

## 4. Comments as Audit Anchors

- Explain why, invariants, risks, security/compatibility constraints, and non-obvious choices, rather than what the code already says.
- Leave enough traceability at important boundaries for an auditor to connect documents, code, and tests.
- When the project uses trace IDs, optional short labels such as `Spec:`, `Decision:`, `Invariant:`, `Risk:`, and `Verified by:` may be used; no format is mandatory.
- Give temporary workarounds a removal condition or traceable task ID.
- Comments do not replace documents or tests. Immediately update or remove comments that contradict the code.
- Do not fill self-explanatory code with ceremonial comments.

## 5. Verification

- Verify normal, failure, and boundary paths relevant to the change.
- When feasible, add a regression test that captures the specific failure mode of a bug fix.
- Make test names and assertions reveal the contract or failure mode they protect.
- Use reproducible commands and conditions for builds, tests, linting, and manual checks.
- Do not claim unexecuted verification passed. Record checks that could not be run and their risks.

## 6. Completion Evidence

- At completion, synchronize relevant documents, code, tests, configuration, and build paths.
- Record changed files, behavior changes, verification commands and results, unexecuted checks, and remaining risks at the necessary depth in `IMPLEMENTATION_SUMMARY.md` or a relevant project document.
- Completion claims must be reproducible by subsequent coders and auditors from project artifacts without the session conversation.
- A coder's completion report does not replace an audit PASS.

## 7. Final Principle

Implement freely within the contract, without hiding intent, boundaries, side effects, or verification evidence.

Documents authorize implementation; code is execution evidence; tests are reproduction evidence; comments are navigation markers for audits.
