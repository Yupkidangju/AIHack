# AI Implementation Documentation Standard

## 0. Purpose

This standard governs AI-produced plans, specifications, designs, implementation guides, and roadmaps. The target is not merely readable documentation, but documentation from which an implementer can build immediately.

The central question is:

- Can the recipient start implementation without another planning meeting?

Unless the answer is `Yes`, the document is not yet sufficient.

## 1. What "Reference Grade" Means

Required qualities:

- Scope is settled.
- Non-goals are explicit.
- Settled decisions outnumber open decisions.
- States, types, data, formulas, events, and screen flows connect.
- Every named concept and ID resolves to an actual definition.
- Implementation order and verification methods are specified.
- Residual risks may remain, but must not themselves prevent implementation from starting.

Produce an implementation reference, not an idea document.

## 2. Minimum Document Set

For a production-grade document set, target at least:

| Document | Role | Consequence of Absence |
| --- | --- | --- |
| `spec.md` | Master specification, scope, contracts, formulas, real data | No overall baseline |
| `designs.md` | UI/UX, screen flow, layers, data connections | Implementers improvise screen behavior |
| `implementation_summary.md` | System decomposition, file responsibilities, implementation order | Unclear implementation starting point |
| `DESIGN_DECISIONS.md` | Reasons for decisions and rejected alternatives | The same debates recur |
| `BUILD_GUIDE.md` | Scaffolding, builds, runtime paths, execution procedures | First-run failures |
| `audit_roadmap.md` | Phase objectives, verification points, audit criteria | Unfocused implementation |
| `CHANGELOG.md` | Document change history | Changes and their timing cannot be traced |

Names may vary by domain, but preserve these roles.

## 3. Non-Negotiable Requirements

Documents must satisfy the following requirements.

### 3.1 Scope Closure

- State the objective.
- State success criteria.
- State non-goals.
- Eliminate "decide later" items as far as possible. Isolate any remaining items in a later Phase or as residual risks.

### 3.2 Frozen Decisions

- Settle major decisions that determine implementation.
- Examples:
  - Runtime structure
  - Persistence approach
  - Input scope
  - Rendering boundaries
  - Animation/resource pipelines
  - Core UI policy

"It could be A or B" is a failure in an implementation document.

### 3.3 Typed Contracts

Fix core documented contracts as types or schemas.

- State types
- Result types
- Persistence types
- Configuration types
- Event types
- Data-definition types
- Bridge/API method signatures

Explain the meaning of every documented field.

### 3.4 Concrete Numbers

Good documents contain more concrete values than adjectives.

- Use `moveSpeed: 260`, not "fast movement."
- Use `startup: 4f`, not "a short startup."
- Use `80ms ~ 120ms`, not "a brief shake."
- Use `invincible: 120ms`, not "short invincibility."

Without numbers, implementers fill the gaps with their own judgment.

### 3.5 Real Data Samples

Types alone are insufficient. Supply at least one set of real data.

- Player defaults
- Core action/attack data
- At least one enemy data set
- At least one stage/flow example
- Save defaults

Provide data ready to move into code, not merely illustrative placeholders.

### 3.6 Execution Path

Documents must specify implementation order.

- What to build first
- Which files are needed
- Which systems must exist first
- Where verification occurs

Without implementation order, the document remains an explanation rather than an execution guide.

### 3.7 Verification Path

Documents must define verification criteria.

- Commands to run
- Manual checks to perform
- What must pass at each stage
- Artifacts to produce

Define completion by verification results, not the implementer's intuition.

## 4. Master Spec Blueprint

The master specification includes at least:

1. Document operating rules
2. Project identity
3. Objectives and success criteria
4. Non-goals
5. Settled key decisions
6. Technology stack and architectural principles
7. Runtime/build pipeline
8. Directory structure
9. Core behavior definitions
10. System specifications
11. Boundary types and contracts
12. Persistence/configuration/progression policies
13. Settled formulas
14. Real-data reference tables
15. Phased roadmap
16. Commands and verification criteria
17. Security/implementation boundaries
18. Residual risks

In particular, do not omit:

- State transitions
- Data structures
- Result types
- Bridge/API contracts
- Save defaults
- Reward/cost/level-up formulas
- At least one actual content data set

## 5. Design Doc Blueprint

A design document is an implementation guide, not a decorative description.

Required contents:

- Core experience
- Complete screen flow
- Combat/core-screen layer structure
- Color/typography tokens
- Layout criteria
- HUD/screen-element inventory
- Data-connection criteria
- Per-screen button policies
- Hit-feedback/motion rules
- React/Canvas or UI/core responsibility split
- Settled design decisions

Do not merely say "make it look good." Specify what goes where, which data it reads, and how it reacts under each condition.

## 6. Implementation Summary Blueprint

The implementation summary provides a starting point.

Required contents:

- Overall runtime flow
- System decomposition table
- Boundary-contract summary
- File responsibilities
- Algorithm notes
- Summary of settled formulas
- Minimum scope of the first playable version
- Recommended implementation order
- Maintenance rules

Write for the implementer opening the code for the first time.

## 7. Decision Record Blueprint

Decision records must explain why.

Each decision includes:

- Context
- Decision
- Alternatives and reasons for rejection
- Consequences

Do not stop at "we did this." Explain why other options were rejected.

## 8. Build Guide Blueprint

The build guide must actually enable a successful first run.

Required contents:

- Prerequisites
- Safe scaffolding procedures
- Required installations
- Runtime output paths
- Entry-file wiring
- Required scripts in the root `package.json`
- Minimal `main/preload` examples
- Global type-declaration examples
- First-run commands
- Build artifacts to inspect
- Pre-deployment checklist

Prevent these failures in particular:

- A command exists, but the actual entrypoint is not connected.
- The renderer starts, but main/preload artifacts are missing.
- Security options are described, but code examples are absent.

## 9. Audit Roadmap Blueprint

A roadmap is an implementation-audit framework, not merely a schedule.

Required contents:

- Consistency audit
- Risk audit
- Architecture audit
- Roadmap audit
- Objectives per Phase
- Implementation items per Phase
- Actual algorithm notes per Phase
- Verification points per Phase
- Checkpoint policy
- Current key residual risks

A good roadmap answers "Can this Phase pass now?", not just "What comes next?"

## 10. Cross-Document Closure Rules

The document set must form a connected whole.

Required relationships:

- Events described in `designs.md` exist as types in `spec.md`.
- Types defined in `spec.md` appear in the file responsibilities in `implementation_summary.md`.
- `BUILD_GUIDE.md` scripts agree with the runtime pipeline in `spec.md`.
- `audit_roadmap.md` implementation items can be implemented directly from the data and types in `spec.md`.
- `CHANGELOG.md` records major changes to document structure.

A concept appearing in only one document is a warning sign.

## 11. Closure Rules for IDs, Flags, CTAs, and APIs

These are common AI omissions; resolve them explicitly.

### 11.1 IDs

Define every ID appearing in the documents.

- Stage IDs
- Enemy IDs
- Attack IDs
- Event IDs
- State names

A reference without a definition is a documentation failure.

### 11.2 Flags

Specify the meaning and creation conditions of Boolean or state flags.

- Name
- Meaning
- Calculation conditions
- Consumer location

Example:

- `flawlessBonus = damageTaken === 0`

Make definitions at least this concrete.

### 11.3 CTAs

Do not merely list screen buttons. Define their activation conditions and subsequent state.

For example:

- Button name
- When it becomes active
- Where it leads when pressed
- Which state it updates

### 11.4 Bridge / API

Describe bridges and APIs as method contracts, not merely as existing components.

Example:

```ts
type AppBridge = {
  loadSave(): Promise<SaveData | null>;
  writeSave(save: SaveData): Promise<void>;
};
```

## 12. Anti-Patterns

The following vague expressions are close to prohibited in implementation documents:

- "Appropriately"
- "As needed"
- "When desired"
- "Consider later"
- "Make it game-like"
- "Naturally"
- "Flexibly"
- "Roughly this much"

They may suggest a direction, but do not settle implementation.

These structures are also risky:

- Explanations without types
- Balance rules without numbers
- Buttons without state-update rules
- Event names without payloads
- Attack IDs without hitboxes/formulas
- Build commands without output-entrypoint connections
- Persistence structures without defaults/reset policies

## 13. Quality Checklist

Before submitting documents, check:

- Are both objectives and non-goals stated?
- Are key decisions settled?
- Are major typed contracts defined?
- Do all unique IDs resolve to real data?
- Are formulas defined with concrete values?
- Is there at least one real data set?
- Is implementation order specified?
- Are verification commands and checkpoints specified?
- Are screen-button policies defined?
- Are persistence/configuration/bridge policies defined?
- Are terms and types consistent across documents?
- Are residual risks below the level that prevents implementation?

Any `No` means the document is not yet sufficient as a reference blueprint.

## 14. Acceptance Standard

A document set is a production-grade reference when all three hold:

1. An implementer can build the first playable version from the documents alone.
2. A verifier can determine what is complete and incomplete from the documents alone.
3. A subsequent AI can read the documents without rewriting the same concepts at a lower quality.

## 15. Copy-Paste Instruction Block for GPTs, Gems, and Skills

Use this block directly as reference instructions for GPTs, Gems, or skills.

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

Good implementation documents contain more settled decisions than impressive prose.

AI documentation aims to make implementation possible, not merely impress the reader. At minimum, the result must answer:

- Can this be built immediately?
- Is every documented concept defined?
- Do the criteria prevent unsupported claims of completion?

Rewrite documents that cannot meet these three tests.
