# AIHack Modification Manifest

Notice ID: `AIHACK-MODIFICATIONS-2026-07-20-01`  
Covered change period: `2025-05-20..2026-07-20`  
Release version: `0.3.0`

AIHack is an AI-assisted semantic rewrite derivative of NetHack 3.6.7. The following path scopes were newly created or changed for the AIHack Rust rewrite during the covered change period.

| Distributed path scope | Change date | Modification notice |
| --- | --- | --- |
| `src/**` | 2025-05-20..2026-07-20 | Rust compatibility facade and project entry library |
| `crates/**` | 2025-05-20..2026-07-20 | Deterministic core, content, AI contracts, LLM adapter and runtime rewrite |
| `apps/**` | 2025-05-20..2026-07-20 | TUI and headless application adapters |
| `tests/**` | 2025-05-20..2026-07-20 | AIHack behavior, compatibility, security and release verification |
| `scripts/**` | 2025-05-20..2026-07-20 | Build, checkpoint, deterministic fixture and PTY verification tooling |
| `docs/**` | 2025-05-20..2026-07-20 | Compatibility, provenance and implementation documentation |
| `.archive/**` | 2025-05-20..2026-07-20 | Immutable AIHack planning and design snapshots |
| root Cargo/build/config files | 2025-05-20..2026-07-20 | Rust workspace, dependency, CI and release configuration |
| root project documents | 2025-05-20..2026-07-20 | Specification, decisions, provenance, license and release guidance |

The exact file set covered by each scope is the tree identified by the expanded commit in `RELEASE-METADATA`. This manifest is carried inside the source archive and alongside released binaries, so modification evidence does not depend on distributed Git history or a `.git/` directory.

The historical `legacy_nethack_port_reference/**` tree is excluded from the AIHack release archive and is not covered as approved AIHack runtime source. The operative license is the root `LICENSE`; the damaged legacy license copy is not used.

This manifest records the project's modification scope and dates. It is not a legal opinion that substitutes for qualified license review.
