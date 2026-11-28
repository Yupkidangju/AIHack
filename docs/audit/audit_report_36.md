# AIHack 3-Pass Cooperative Audit Report (Audit 36)

- **Audit Date:** 2026-11-28
- **Target Scope:** `8766ca3..67a1897` (`10c6c02`, `b20a8f2`, `67a1897`) + Uncommitted Stage 7 (`P13`) working tree on `HEAD` (`67a18973ac0883d0fc1a68fed36a5e29ed826e87`, 48 modified tracked files + 1 untracked [tests/stage7_p13.rs](../../tests/stage7_p13.rs))
- **Prior Baseline:** [audit_report_35.md](audit_report_35.md) (`8766ca3`)
- **Governing Standards:** [AGENTS.md](../../AGENTS.md) (v1.4), [AI_AUDIT_DOC_STANDARD.md](../../AI_AUDIT_DOC_STANDARD.md), [campaign_spec.md](../campaign_spec.md), [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md)
- **Toolchain Verified:** `rustc 1.94.1 (e408947bf 2026-03-25)`, `cargo 1.94.1 (29ea6fb6a 2026-03-24)` (Windows host)
- **Final Decision:** **`HOLD`** (2 Critical, 8 Major, 3 Minor `Needs Fix`, 1 Minor `Open`, 1 Minor `Needs Clarification`, 1 Minor `Accepted Risk`, 2 Cross-Pass Conflicts)

---

## 1. Audit Scope

### 1.1 Audited Commits and Dirty Working Tree

1. **Committed Range (`8766ca3..67a1897`):**
   - `10c6c02`: `feat: implement P10, Audit 35 remediation, and P11 campaign expansion` (`79 files, +12119/-701`)
   - `b20a8f2`: `chore: align modification notice period to 2026-11-28`
   - `67a1897`: `feat: implement Stage 6 (P12) spell system and large content catalog`
2. **Uncommitted Stage 7 (`P13`) Working Tree on `67a18973ac0883d0fc1a68fed36a5e29ed826e87`:**
   - 48 modified tracked files spanning [CHANGELOG.md](../../CHANGELOG.md), [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md), [PROVENANCE.md](../../PROVENANCE.md), [README.md](../../README.md), `apps/*`, `crates/*`, [campaign_implementation.md](../campaign_implementation.md), [campaign_spec.md](../campaign_spec.md), [r7-content.sha256](../provenance/r7-content.sha256), `spec.md`, [plan.md](../../tasks/plan.md), [todo.md](../../tasks/todo.md), [campaign.rs](../../tests/campaign.rs), [stage6_p12.rs](../../tests/stage6_p12.rs), and `tests/{campaign_p7,stage2_p8}.rs`, plus 1 untracked [stage7_p13.rs](../../tests/stage7_p13.rs) (49 files total; [IMPLEMENTATION_SUMMARY.md](../../IMPLEMENTATION_SUMMARY.md) unmodified).

### 1.2 Observed Verification Gate Results

All 8 local workspace gates executed cleanly (exit `0`) against the current dirty Stage 7 tree prior to writing this report:

| Gate | Command | Result | Observed Evidence |
| :--- | :--- | :--- | :--- |
| Format | `cargo fmt --all -- --check` | `PASS` (`0`) | Zero formatting diffs |
| Lint | `cargo clippy -j 2 --workspace --all-targets --locked -- -D warnings` | `PASS` (`0`) | 0 warnings across workspace targets |
| Security Audit | `cargo audit` | `PASS` (`0`) | 318 crate dependencies scanned; 2 allowed warnings (`rustix 1.1.4` `RUSTSEC-2026-0486`, `tokio 1.52.4` `RUSTSEC-2026-0386`); 0 hard vulnerabilities |
| Supply Chain | `cargo deny check licenses bans sources` | `PASS` (`0`) | `bans ok, licenses ok, sources ok` |
| R7 Checkpoint | `bash scripts/r7_checkpoint.sh` | `PASS` (`0`) | `R7 CHECKPOINT: PASS`; `items.toml` (130), `monsters.toml` (95), LF SHA-256 manifest match |
| R8 Checkpoint | `bash scripts/r8_checkpoint.sh` | `PASS` (`0`) | `R8 CHECKPOINT: PASS` |
| Full Test Suite | `cargo test -j 2 --workspace --all-targets --locked --no-fail-fast` | `PASS` (`0`) | 103 suites, 571 passed; 0 failed; 0 ignored (incl. `audit35_remediation` 9, `build_contract` 12, `provenance_manifest` 14, `public_mutation_boundary` 2, `r8_documentation` 11, `release_archive_security` 2, `conpty_contract` 4, `ma2_play` 9, `tui_contract` 20) |
| Release Build | `cargo build -j 2 --workspace --all-targets --release --locked` | `PASS` (`0`) | Clean workspace release build |

Additional direct static inspections:
- `legacy_nethack_port_reference` search across `Cargo.toml`, `src`, `crates`, `apps`: 0 matches.
- `unsafe` keyword search across `crates/`, `apps/`, `src/`: 0 matches.
- `.attacks` schema access across `crates/`, `apps/`: 1 match ([schema.rs](../../crates/aihack-content/src/schema.rs) line 526 only).
- `cargo tree -p aihack-core --locked`: only `rand 0.8.6`, `serde 1.0.228`, `thiserror 1.0.69`.
- `cargo tree -d --locked`: duplicate versions limited to `bitflags`, `getrandom`, `hashbrown`, `io-lifetimes`, `syn`, `thiserror`, `windows-sys`/`windows-targets`/`windows_x86_64_msvc` (`crossterm` duplicate count = 0).
- Nondeterministic RNG / clock leak scan (`thread_rng|rand::random|SystemTime::now`): 0 RNG leaks; `SystemTime::now` limited to TUI auto-label expiry in [mod.rs](../../apps/aihack-tui/src/tui/mod.rs) lines 920 and 1581, and test helper [p10_cli_build.rs](../../apps/aihack-headless/tests/p10_cli_build.rs) line 10.

### 1.3 Prior Audit (`audit_report_35.md`) Re-Audit Status

| Prior IDs | Disposition Summary | Current Status | Notes |
| :--- | :--- | :--- | :--- |
| `CRIT-01..03`, `MAJ-01..02`, `MAJ-04..06`, `MIN-02..06`, `GAP-01` | Remediated in `10c6c02`; covered by `audit35_remediation` (9/9 PASS) | `Closed` | Core domain/runtime fixes verified in current suite |
| `MAJ-03` (unidentified item masking) | Core masking preserved via `audit35_remediation` and `aihack-llm` wrappers; new Stage 6/7 observation, UI, and log paths leak unmasked item identity/BUC | `Reopened (Partial)` | Tracked under `SEC-F002` |
| `MIN-01` (observation parity) | Core parity verified; new P12/P13 observation/dig/wish leaks found | `Reopened (Partial)` | Tracked under `DBG-F004` & `SEC-F002` |
| `MIN-07` (`campaign_implementation.md`) | Superseded by Stage 7 doc-sync drift | `Open -> Merged` | Tracked under `IMP-F005` |

---

## 2. Excluded Scope

- **Cross-OS External Release Verification:** Release binary packaging and dual-OS archive verification remain governed by historical Report 32 authority; only Windows host `rustc 1.94.1` build/test/checkpoint commands were executed in this audit, and no remote Git push or release publication was performed.
- **Live External LLM Daemon Integration:** Evaluated [service.rs](../../crates/aihack-llm/src/service.rs), [transport.rs](../../crates/aihack-llm/src/transport.rs), and [decision.rs](../../crates/aihack-llm/src/decision.rs) statically and via contract tests without connecting to an external running inference server.
- **Historical Archive Files:** Pre-Audit-35 snapshots outside active Stage 5–7 commits and documents listed in Section 1 were not re-audited.

---

## 3. Pass 1: Implementation Compliance Findings

### `IMP-F001` — Astral Plane High Altars Can Be Converted Across Alignments (`P10` x `P13`)

- **ID:** `IMP-F001`
- **Pass:** Pass 1 (Implementation Compliance)
- **Pattern:** Spec-to-Code Behavioral Conflict (Unscoped Altar Conversion)
- **Area:** [altar.rs](../../crates/aihack-runtime/src/systems/altar.rs) (lines 104–141), [save.rs](../../crates/aihack-runtime/src/save.rs) (lines 435–459)
- **Severity:** `Major`
- **Status:** `Needs Fix`
- **Summary:** Offering sacrifices on a cross-aligned altar unconditionally converts that altar's alignment, including high altars on `Planes 5` (`Astral Plane`).
- **Evidence:** In [altar.rs](../../crates/aihack-runtime/src/systems/altar.rs) lines 104–141, `offer` computes `let cross_aligned = altar_alignment(world).is_some_and(|align| align != campaign.alignment);` and inside `if cross_aligned` calls `level.set_altar_alignment(pos, campaign.alignment);` without checking `world.ascension_altars.contains(&(level_id, pos))` or `Planes 5`.
- **Expected:** Astral high altars on `Planes 5` should remain fixed (`Lawful`, `Neutral`, `Chaotic`) per P13 ascension rules so victory requires reaching the player's matching co-aligned high altar rather than converting an enemy high altar.
- **Actual:** Any sacrifice on a cross-aligned `Planes 5` high altar rewrites its alignment to `campaign.alignment`, bypassing the wrong-altar/Astral trial constraint.
- **Impact:** Breaks Stage 7 endgame difficulty and undermines `Victory` alignment checks in [save.rs](../../crates/aihack-runtime/src/save.rs).
- **Suggested Fix:** Guard `set_altar_alignment` in [altar.rs](../../crates/aihack-runtime/src/systems/altar.rs) to skip/reject conversion when `level_id == LevelId::new(BranchId::Planes, 5)` or `world.ascension_altars.contains(&(level_id, pos))` (see `XPF-F001` for joint handling with `SEC-F003`).
- **Re-audit Method:** Unit/integration test attempting sacrifice on a cross-aligned `Planes 5` altar in [stage7_p13.rs](../../tests/stage7_p13.rs) confirming alignment remains unchanged and anger/consequence behavior triggers as designed.
- **Owner:** Runtime / Systems
- **Notes:** Overlaps with Security finding `SEC-F003` (`XPF-F001`).

### `IMP-F002` — Spell Casting Failure Penalty, MP Deduction Order, and Cursed Study Drift (`P12`)

- **ID:** `IMP-F002`
- **Pass:** Pass 1 (Implementation Compliance)
- **Pattern:** Incomplete Spec Contract Implementation
- **Area:** [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs) (lines 50–77, 79–161, 190–210), [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md) (`ADR-P12-01`), [campaign_spec.md](../campaign_spec.md)
- **Severity:** `Major`
- **Status:** `Needs Fix`
- **Summary:** Spell casting deducts full MP before failure checks with zero skill practice on fail, omits `campaign.level()` in `spell_failure_percent`, clamps failure at `95%` instead of `100%`, and `study_spellbook` leaves `_rng` unused while applying only partial cursed penalties.
- **Evidence:** In [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs): (1) lines 50–77 omit `campaign.level()` in the reduction term and end with `(base - reduction).clamp(0, 95)` rather than `0..=100%`; (2) `study_spellbook` lines 79–161 accept `_rng` unused (`Uncursed` always succeeds 100%) and lines 121–127 only apply `confused = 4` on `Cursed` spellbooks (omitting blind / `1d6` damage); (3) `cast_at` deducts full `cost` at line 190 before evaluating `fail_pct` at line 201 and returning `Ok(events)` with 0 skill practice on failure.
- **Expected:** Failure chance, MP loss/refund policy, skill practice on cast attempt, and blessed/uncursed/cursed spellbook study outcomes should match `ADR-P12-01` and [campaign_spec.md](../campaign_spec.md).
- **Actual:** Failed casts still consume full MP with zero skill XP gain, `spell_failure_percent` ignores `campaign.level()` and caps failure at `95%`, and `_rng` in `study_spellbook` is unused with partial cursed consequences.
- **Impact:** Skews caster balance, penalizes early casting disproportionately, and leaves blessed/uncursed difficulty tuning and cursed study hazards incomplete.
- **Suggested Fix:** Align `spell_failure_percent`, `study_spellbook`, and `cast_at` in [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs) with `ADR-P12-01` / [campaign_spec.md](../campaign_spec.md) (including clarifying or testing MP spend on fail, level bonus, `0..=100` clamp, and cursed blind/`1d6` penalty).
- **Re-audit Method:** Verify updated failure formula boundary cases (`0%` and `100%`), MP handling, and cursed spellbook penalty assertions in [stage6_p12.rs](../../tests/stage6_p12.rs).
- **Owner:** Runtime / Magic Systems
- **Notes:** See `IMP-F006` for the separate mithril armor ambiguity in `ADR-P12-01`.

### `IMP-F003` — Missing `"npc"` Flags in `monsters.toml` and Missing `pet_hunger` Initialization (`P12`)

- **ID:** `IMP-F003`
- **Pass:** Pass 1 (Implementation Compliance)
- **Pattern:** Dead Data Guard / Uninitialized Companion State
- **Area:** [monsters.toml](../../crates/aihack-content/src/data/monsters.toml), [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs) (lines 547–558, 673–676), [projectiles.rs](../../crates/aihack-runtime/src/systems/projectiles.rs) (lines 754–756), [pets.rs](../../crates/aihack-runtime/src/systems/pets.rs) (line 173)
- **Severity:** `Major`
- **Status:** `Needs Fix`
- **Summary:** Both `CharmMonster` and `CreateFamiliar` guard against `"npc"` flags that do not exist anywhere in `monsters.toml`, and both leave `pet_hunger` at `0`.
- **Evidence:** `rg -n '"npc"' crates/aihack-content/src/data/monsters.toml` returned 0 matches. Yet [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs) line 551 and [projectiles.rs](../../crates/aihack-runtime/src/systems/projectiles.rs) line 754 rely on `aihack_content::monster_has_ai_flag(mk, "npc", reg)` and `mk != MonsterKind::Minotaur` to protect unique NPCs and bosses (`Shopkeeper`, `TemplePriest`, `Oracle`, `Watchman`, `WizardOfYendor`, `HighPriestMoloch`, `Orcus`, `QuestNemesis`, `RiderDeath`, `RiderPestilence`, `RiderFamine`). Furthermore, lines 555–557 (`CharmMonster`) and lines 673–676 (`CreateFamiliar`) set `Faction::Pet` and `tameness = 5` without setting `pet_hunger`, triggering hunger decay/hostile reversion at [pets.rs](../../crates/aihack-runtime/src/systems/pets.rs) line 173 (`turn.is_multiple_of(20)`).
- **Expected:** Unique shopkeepers, priests, quest/endgame bosses, and riders cannot be charmed into pets, and legitimately charmed/summoned pets start with non-zero hunger (e.g., initial fed buffer matching normal pets).
- **Actual:** Every NPC and endgame boss lacking the `"npc"` flag in [monsters.toml](../../crates/aihack-content/src/data/monsters.toml) can be charmed into a `Faction::Pet`, while charmed/summoned pets immediately enter starving decay (`pet_hunger = 0`).
- **Impact:** Allows trivializing `WizardOfYendor`, `Orcus`, `HighPriestMoloch`, Riders, and `Shopkeeper` via charm effects, while ordinary charmed pets/familiars starve abnormally fast.
- **Suggested Fix:** Tag shopkeepers, priests, oracle, watchmen, quest nemesis, and unique endgame bosses/riders with `"npc"` in [monsters.toml](../../crates/aihack-content/src/data/monsters.toml) (plus explicit unique/boss immunity test coverage) and initialize `pet_hunger` in `CharmMonster` and `CreateFamiliar`.
- **Re-audit Method:** Verify `monsters.toml` contains `"npc"` flags for protected NPCs/bosses, confirm charm rejection tests, and verify newly created/charmed pets initialize positive `pet_hunger`.
- **Owner:** Content / Runtime Systems
- **Notes:** None.

### `IMP-F004` — `MakeInvisible` Wand Is a No-Op and `wish()` Silently Clamps + Skips Artifact Alignment Check (`P12`)

- **ID:** `IMP-F004`
- **Pass:** Pass 1 (Implementation Compliance)
- **Pattern:** Stubbed Effect / Contract Constraint Violation
- **Area:** [projectiles.rs](../../crates/aihack-runtime/src/systems/projectiles.rs) (lines 117–125, 164–201, 590–602)
- **Severity:** `Major`
- **Status:** `Needs Fix`
- **Summary:** `WandEffect::MakeInvisible` prints flavor text without modifying entity state, and `wish` silently clamps out-of-range enchantments and ignores artifact alignment blast rules.
- **Evidence:** In [projectiles.rs](../../crates/aihack-runtime/src/systems/projectiles.rs): (1) lines 590–602 (`WandEffect::MakeInvisible`) only push `format!("The shimmer briefly conceals creature #{}.", target.0)` with zero entity/state mutation; (2) line 124 uses `enchant.clamp(0, 3)` instead of rejecting `|enchant| > 3` per `ADR-P12-01`; (3) lines 165–190 place wished artifacts directly into inventory without hostile-alignment blast / `ArtifactBlast` verification.
- **Expected:** `MakeInvisible` mutates invisibility state/timer on the target entity, `wish` enforces enchantment bounds per spec without silent `+0..3` coercion on invalid negative/excessive requests, and cross-aligned wished artifacts obey artifact blast rules.
- **Actual:** `MakeInvisible` does nothing beyond logging, excessive/negative wish enchant requests coerce silently, and cross-aligned artifact wishes skip alignment blast consequences.
- **Impact:** Leaves a P12 wand effect non-functional and enables free over-enchant / alignment-bypass artifact wishing.
- **Suggested Fix:** Implement actual invisibility status/flag mutation for `WandEffect::MakeInvisible`, enforce spec bounds in `wish` cleanly (rejecting or clamping strictly as documented in `ADR-P12-01`), and invoke artifact alignment/blast checks when gifting a wished artifact.
- **Re-audit Method:** Test `WandEffect::MakeInvisible` state mutation, invalid wish enchant handling, and hostile-alignment artifact wish damage in [stage6_p12.rs](../../tests/stage6_p12.rs).
- **Owner:** Runtime / Projectiles & Wishes
- **Notes:** None.

### `IMP-F005` — Stage 7 Specification, UI Copy, Expiry Date, and Summary Documentation Drift

- **ID:** `IMP-F005`
- **Pass:** Pass 1 (Implementation Compliance)
- **Pattern:** Documentation / UI / Provenance Drift
- **Area:** [campaign_spec.md](../campaign_spec.md) (lines 349, 353, 356–357), [session.rs](../../crates/aihack-runtime/src/session.rs) (lines 231, 271), [render_panels.rs](../../apps/aihack-tui/src/tui/render_panels.rs) (lines 612, 717), [PROVENANCE.md](../../PROVENANCE.md) (line 102), [dependency-exceptions.json](../../dependency-exceptions.json) (line 12), [IMPLEMENTATION_SUMMARY.md](../../IMPLEMENTATION_SUMMARY.md)
- **Severity:** `Minor`
- **Status:** `Needs Fix`
- **Summary:** Multiple Stage 7 docs, TUI strings, and provenance dates drift from the implemented branch/level IDs, current campaign depth, and exception expiry.
- **Evidence:**
  1. [campaign_spec.md](../campaign_spec.md) lines 349, 353, 356–357 reference `"quest_home"`, `"quest_locate"`, `"quest_sanctum"`, `"gehennom_sanctum"`, `"plane_earth"`, `"plane_astral"`, and pursuit `"Gehennom 2 -> Main 4 -> Main 2 -> Main 1"`, whereas implemented code and [stage7_p13.rs](../../tests/stage7_p13.rs) (lines 310, 406–409, 467–503, 569, 611–613, 670) use `"quest_1"`, `"valley_of_the_dead"`, `"orcus_town"`, `"wizards_tower"`, `"molochs_sanctum"`, `"plane_of_earth"`..`"plane_of_water"`, `"astral_plane"`, and pursuit `"Main 4/2/1 -> Planes 2/4"`.
  2. [session.rs](../../crates/aihack-runtime/src/session.rs) lines 231 and 271 still display `"recover the Amulet on Main 6 and return to the surface."`.
  3. [render_panels.rs](../../apps/aihack-tui/src/tui/render_panels.rs) lines 612 and 717 still show `"Campaign: Main 6 + Mines 2; Main 3 up: B->Mines"` and `"The Amulet has reached the surface."`.
  4. [PROVENANCE.md](../../PROVENANCE.md) line 102 states `"2026-10-31 expiry"` for `DEP-EXC-0001` (`winx 0.36.4`), conflicting with [dependency-exceptions.json](../../dependency-exceptions.json) line 12 (`"expires_on": "2027-02-20"`).
  5. [IMPLEMENTATION_SUMMARY.md](../../IMPLEMENTATION_SUMMARY.md) lacks a Stage 7 (`P13`) summary entry despite Stage 7 status markers in [plan.md](../../tasks/plan.md) line 156 and [todo.md](../../tasks/todo.md) line 121.
- **Expected:** Spec level identifiers, TUI/session strings, provenance expiry dates, and `IMPLEMENTATION_SUMMARY.md` accurately reflect the Stage 7 implementation.
- **Actual:** Players see outdated `Main 6` surface copy in TUI/session prompts, and spec/provenance readers encounter mismatched IDs and expiry dates.
- **Impact:** Misleads players on victory conditions/depth and confuses future contributors/auditors reading [campaign_spec.md](../campaign_spec.md) and [PROVENANCE.md](../../PROVENANCE.md).
- **Suggested Fix:** Synchronize [campaign_spec.md](../campaign_spec.md), [session.rs](../../crates/aihack-runtime/src/session.rs) startup copy, [render_panels.rs](../../apps/aihack-tui/src/tui/render_panels.rs) status/victory text, [PROVENANCE.md](../../PROVENANCE.md) `DEP-EXC-0001` expiry (`2027-02-20`), and add the Stage 7 entry to [IMPLEMENTATION_SUMMARY.md](../../IMPLEMENTATION_SUMMARY.md).
- **Re-audit Method:** Verify `r8_documentation`, inspect updated strings across [session.rs](../../crates/aihack-runtime/src/session.rs) and [render_panels.rs](../../apps/aihack-tui/src/tui/render_panels.rs), and check matching level IDs between [campaign_spec.md](../campaign_spec.md) and [stage7_p13.rs](../../tests/stage7_p13.rs).
- **Owner:** Docs / TUI / Runtime
- **Notes:** Incorporates historical doc-sync remainder of Audit 35 `MIN-07`.

### `IMP-F006` — Mithril Armor Spell-Failure Penalty Contradiction (`ADR-P12-01` vs Code & Spec)

- **ID:** `IMP-F006`
- **Pass:** Pass 1 (Implementation Compliance)
- **Pattern:** Governing Document Contradiction
- **Area:** [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md) (line 52, `ADR-P12-01`), [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs) (line 59), [stage6_p12.rs](../../tests/stage6_p12.rs) (lines 258–271)
- **Severity:** `Minor`
- **Status:** `Needs Clarification`
- **Summary:** `ADR-P12-01` lists `(iron/steel/mithril)` as heavy armor incurring spell-failure penalties, whereas code and `tests/stage6_p12.rs` explicitly treat `mithril` as exempt.
- **Evidence:** [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md) line 52 records `iron`/`steel`/`mithril` armor spell-failure penalty, while [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs) checks `matches!(mat, "iron" | "steel")` and [stage6_p12.rs](../../tests/stage6_p12.rs) line 270 explicitly asserts `"mithril incurs no metal penalty"`.
- **Expected:** A single unambiguous policy in `DESIGN_DECISIONS.md` and `campaign_spec.md` defining whether mithril incurs a metal spell-failure penalty or is exempt.
- **Actual:** `ADR-P12-01` disagrees with the runtime and test suite.
- **Impact:** Minor ambiguity for contributors maintaining spell-failure balance.
- **Suggested Fix:** Confirm design intent (typically exempting mithril like NetHack) and align `ADR-P12-01` in [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md) or update [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs) and [stage6_p12.rs](../../tests/stage6_p12.rs) accordingly.
- **Re-audit Method:** Verify `ADR-P12-01`, [campaign_spec.md](../campaign_spec.md), [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs), and [stage6_p12.rs](../../tests/stage6_p12.rs) agree verbatim.
- **Owner:** Design / Spec Maintainer
- **Notes:** Listed in Section 9 (`Needs Spec Clarification`).

---

## 4. Pass 2: Debug / Engineering Quality Findings

### `DBG-F001` — `SaveV3` Validation Rejects Legitimate Locked Doors Created During Play

- **ID:** `DBG-F001`
- **Pass:** Pass 2 (Debug / Engineering Quality)
- **Pattern:** Save/Load State-Transition Matrix Mismatch
- **Area:** [campaign_map.rs](../../crates/aihack-runtime/src/campaign_map.rs) (lines 1164–1179), [tools.rs](../../crates/aihack-runtime/src/systems/tools.rs) (lines 178–194), [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs) (lines 780–801), [projectiles.rs](../../crates/aihack-runtime/src/systems/projectiles.rs) (lines 640–667)
- **Severity:** `Critical`
- **Status:** `Needs Fix`
- **Summary:** Players can legitimately lock a `Closed` or `Open` door via Skeleton Key, Wizard Lock, or Wand of Locking, producing a `SaveV3` file that fails `validate_persisted_map` on reload.
- **Evidence:** In [campaign_map.rs](../../crates/aihack-runtime/src/campaign_map.rs) lines 1164–1179, `validate_persisted_map` permits:
  - `(Closed, Closed | Open)`
  - `(Open, Open | Closed)`
  - `(Locked, Locked | Open | Closed)`
  - `(HiddenDoor, HiddenDoor | Closed | Open)`
  Transitions `(Closed | Open | HiddenDoor, Locked)` fall through to `_ => false`. Yet [tools.rs](../../crates/aihack-runtime/src/systems/tools.rs) lines 178–194 (`SkeletonKey` on `Closed`), [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs) lines 780–801 (`WizardLock` on `Open | Closed`), and [projectiles.rs](../../crates/aihack-runtime/src/systems/projectiles.rs) lines 640–667 (`WandEffect::Locking` on `Open | Closed`) all transition non-locked doors to `DoorState::Locked`. Additionally, [stage6_p12.rs](../../tests/stage6_p12.rs) lines 519–535 tested locking on `door_sess` but serialized `sess` instead of `door_sess`, masking this bug.
- **Expected:** Any reachable in-game tile transition from legitimate player or monster actions must round-trip cleanly through `SaveV3` serialization and `validate_persisted_map`.
- **Actual:** Locking any unlocked door (`Closed`, `Open`, or revealed `HiddenDoor`) yields a save state that fails reload validation.
- **Impact:** **Save corruption / un-loadable valid player saves** across `SkeletonKey`, `WizardLock`, and `WandEffect::Locking`.
- **Suggested Fix:** Allow `DoorState::Locked` as a valid persisted target from `DoorState::Closed`, `DoorState::Open`, and `TileKind::HiddenDoor` in [campaign_map.rs](../../crates/aihack-runtime/src/campaign_map.rs) lines 1164–1179.
- **Re-audit Method:** Fix [stage6_p12.rs](../../tests/stage6_p12.rs) lines 519–535 to serialize and reload `door_sess`, plus add round-trip save/load regression tests for `SkeletonKey`, `WizardLock`, and `WandEffect::Locking`.
- **Owner:** Runtime / Persistence
- **Notes:** Direct `HOLD` blocker.

### `DBG-F002` — `SaveV3` Validation Rejects Sokoban Boulder Over a Previously Plugged Pit

- **ID:** `DBG-F002`
- **Pass:** Pass 2 (Debug / Engineering Quality)
- **Pattern:** Multi-Step Terrain Mutation Mismatch
- **Area:** [campaign_map.rs](../../crates/aihack-runtime/src/campaign_map.rs) (lines 1148–1163), [movement.rs](../../crates/aihack-runtime/src/systems/movement.rs) (lines 213–236)
- **Severity:** `Critical`
- **Status:** `Needs Fix`
- **Summary:** Plugging a Sokoban pit turns `Trap(Pit)` / `HiddenTrap(Pit)` into `Floor`, allowing a second boulder to be pushed onto that square; `validate_persisted_map` rejects the resulting `(Pit, Boulder)` delta on reload.
- **Evidence:** [movement.rs](../../crates/aihack-runtime/src/systems/movement.rs) lines 213–236 convert `TileKind::Trap(TrapKind::Pit) | TileKind::HiddenTrap(TrapKind::Pit)` into `TileKind::Floor` when a boulder plugs it, making that cell passable to another boulder push onto `Floor` -> `TileKind::Boulder`. However, [campaign_map.rs](../../crates/aihack-runtime/src/campaign_map.rs) lines 1148–1163 only accept:
  - `(Floor, Boulder)`
  - `(Boulder, Boulder | Floor)`
  - `(Trap(TrapKind::Pit), Floor)`
  - `(HiddenTrap(TrapKind::Pit), Floor)`
  falling through to `_ => false` for `(Trap(Pit) | HiddenTrap(Pit), Boulder)`.
- **Expected:** Valid Sokoban puzzle progressions—including pushing a second boulder over a plugged pit—must pass `validate_persisted_map` on `SaveV3` reload.
- **Actual:** Saving in Sokoban with a boulder resting over a formerly pitted square produces an un-loadable `SaveV3` file.
- **Impact:** **Save corruption / un-loadable valid Sokoban saves** during normal puzzle play.
- **Suggested Fix:** Update [campaign_map.rs](../../crates/aihack-runtime/src/campaign_map.rs) lines 1154–1163 to accept `TileKind::Floor | TileKind::Boulder` for `(Trap(TrapKind::Pit) | HiddenTrap(TrapKind::Pit), ...)` when `id.branch == BranchId::Sokoban`.
- **Re-audit Method:** Add a Sokoban save/load regression test pushing two boulders sequentially into/onto a pit tile (`Pit -> Floor -> Boulder`), persisting `SaveV3`, and confirming clean reload.
- **Owner:** Runtime / Persistence
- **Notes:** Direct `HOLD` blocker.

### `DBG-F003` — Global Shopkeeper Debt Wipe and Oracle Enchant / Reveal-Event Gaps

- **ID:** `DBG-F003`
- **Pass:** Pass 2 (Debug / Engineering Quality)
- **Pattern:** Unscoped Global Side Effect / Incomplete Event Emission
- **Area:** [npc.rs](../../crates/aihack-runtime/src/systems/npc.rs) (lines 230–260, 387–437)
- **Severity:** `Major`
- **Status:** `Needs Fix`
- **Summary:** Killing one shopkeeper clears all player shop debt across every branch/level, and Oracle consultation matches unidentified enchantment without setting `enchant_known` or emitting `GameEvent::TileRevealed`.
- **Evidence:** In [npc.rs](../../crates/aihack-runtime/src/systems/npc.rs):
  1. `on_shopkeeper_died` (lines 230–260) zeroes `world.shop_debt = 0` and clears `unpaid` on all player inventory items globally without scoping to the killed shopkeeper's floor/shop (`Main 2` vs `Mines 1`).
  2. `find_first_unidentified_inventory_item` (lines 426–437) matches `!entity.item_enchant_known().unwrap_or(false)` at line 434, yet Oracle payment resolution (lines 411–413) never calls `set_item_enchant_known(item_id, true)`—leaving `enchant_known` permanently `false`.
  3. Oracle floor revelation (lines 388–404) reveals `HiddenDoor` / `HiddenTrap` directly without emitting `GameEvent::TileRevealed`.
- **Expected:** Shopkeeper death only clears debt belonging to that shopkeeper's shop/floor; Oracle full item consultation marks `enchant_known = true` when matching on it and emits `GameEvent::TileRevealed` for revealed hidden terrain.
- **Actual:** Cross-floor shoplifting debts disappear when any single shopkeeper dies elsewhere, and Oracle consultations leave enchantment unknown and omit tile-reveal events.
- **Impact:** Economic exploit across multi-shop branches (`Main 2`, `Mines 1`), incomplete Oracle identification rewards, and missed UI/event log updates.
- **Suggested Fix:** Scope `on_shopkeeper_died` debt clearing to items/debt originating on the shopkeeper's level (`level_id` / shop ID), mark `set_item_enchant_known(item_id, true)` in Oracle item-knowledge handling, and emit `GameEvent::TileRevealed` in lines 388–404.
- **Re-audit Method:** Unit/integration tests with unpaid items from `Main 2` and `Mines 1` verifying selective debt clearance, plus Oracle consultation tests checking `item_enchant_known` and `GameEvent::TileRevealed`.
- **Owner:** Runtime / NPC Systems
- **Notes:** None.

### `DBG-F004` — Observation Emits Illegal `Dig` Actions on Non-Diggable Sokoban and Planes Walls

- **ID:** `DBG-F004`
- **Pass:** Pass 2 (Debug / Engineering Quality)
- **Pattern:** Action Space / Executor Desynchronization
- **Area:** [observation.rs](../../crates/aihack-runtime/src/observation.rs) (lines 941–957), [tools.rs](../../crates/aihack-runtime/src/systems/tools.rs) (lines 269–273)
- **Severity:** `Minor`
- **Status:** `Needs Fix`
- **Summary:** `observation.rs` advertises `Apply { direction: Some(...) }` for `ToolAction::Dig` toward walls in `Sokoban` and `Planes 2..5`, even though `tools::apply_dig` rejects digging there.
- **Evidence:** [observation.rs](../../crates/aihack-runtime/src/observation.rs) lines 941–957 emit directional `Dig` intents without checking branch immunity guards enforced in [tools.rs](../../crates/aihack-runtime/src/systems/tools.rs) lines 269–273.
- **Expected:** Legal-action observation filters match executor preconditions so AI/LLM agents and headless contracts do not receive guaranteed-invalid dig moves.
- **Actual:** Observation legal actions include disallowed directional digs in Sokoban and Planes 2–5.
- **Impact:** Wastes LLM turns / triggers avoidable invalid-action penalties in Sokoban and Endgame Planes.
- **Suggested Fix:** Mirror the non-diggable branch/level check from [tools.rs](../../crates/aihack-runtime/src/systems/tools.rs) inside the `ToolAction::Dig` legal-action emitter in [observation.rs](../../crates/aihack-runtime/src/observation.rs).
- **Re-audit Method:** Verify `legal_actions` in `Sokoban` and `Planes 2..5` omit directional `Dig` commands when holding a pick-axe or wand of digging.
- **Owner:** Runtime / Observation
- **Notes:** Partial reopen of Audit 35 `MIN-01`.

### `DBG-F005` — Secondary Runtime Edge Cases and `build.rs` Line-Parser Fragility

- **ID:** `DBG-F005`
- **Pass:** Pass 2 (Debug / Engineering Quality)
- **Pattern:** Incomplete State Reset / Attacker Attribution / Parser Edge Cases
- **Area:** [death.rs](../../crates/aihack-runtime/src/systems/death.rs) (lines 40–57, 128–137), [monster_ai.rs](../../crates/aihack-runtime/src/systems/monster_ai.rs) (lines 335–339, 558–563), [artifacts.rs](../../crates/aihack-runtime/src/systems/artifacts.rs) (lines 384–394), [build.rs](../../crates/aihack-content/build.rs) (lines 64–158)
- **Severity:** `Minor`
- **Status:** `Needs Fix`
- **Summary:** Six localized runtime/build edge cases across lifesaving, kill attribution, monster armor stacking, reflected death rays, artifact door events, and `build.rs` TOML parsing.
- **Evidence:**
  1. [death.rs](../../crates/aihack-runtime/src/systems/death.rs) lines 40–57 (`AmuletLifeSaving`): resets `poison_turns`, `paralysis_turns`, `nutrition`, and `hp`, but omits resetting `status_timers` (`blind`/`confused`/`stunned`/`hallucinating`) and omits AC recomputation after amulet consumption.
  2. [death.rs](../../crates/aihack-runtime/src/systems/death.rs) lines 128–137: increments `kill_count` and awards gold (`difficulty * 25`) for non-Pet monster deaths regardless of `cause`, crediting the player for monster-vs-monster / environmental kills.
  3. [monster_ai.rs](../../crates/aihack-runtime/src/systems/monster_ai.rs) lines 335–339: applies `s.ac = s.ac.saturating_sub(ac_bonus.min(4)).max(-10)` on every armor pickup without slot uniqueness, allowing cumulative AC stacking from multiple suits/helms.
  4. [monster_ai.rs](../../crates/aihack-runtime/src/systems/monster_ai.rs) lines 558–563: reflected `WandEffect::Death` checks `Resistances::MAGIC_RESISTANCE` only, omitting `"undead"` immunity enforced at [projectiles.rs](../../crates/aihack-runtime/src/systems/projectiles.rs) lines 720–724.
  5. [artifacts.rs](../../crates/aihack-runtime/src/systems/artifacts.rs) lines 384–394: `master_key_of_thievery` transitions `DoorState::Locked` -> `Closed` without emitting `GameEvent::DoorChanged`.
  6. [build.rs](../../crates/aihack-content/build.rs) lines 64–158: custom TOML line parser flushes entries on any line beginning with `[` and slices strings at the first `"`, without handling escape sequences.
- **Expected:** Lifesaving clears active disabling `status_timers` and recomputes AC; non-player kills do not award player kill credit/gold; monster armor equips replace rather than infinitely stack AC; reflected death rays share undead immunity; `master_key_of_thievery` emits `DoorChanged`; `build.rs` parses TOML strings/headers safely (or fails cleanly on malformed quotes/brackets).
- **Actual:** These 6 localized flaws cause subtle gameplay/UI/content-build anomalies.
- **Impact:** Gameplay balance drift (lifesaving loops, monster AC `-10` stacking, undeserved score/gold) and fragile TOML content build parsing if descriptions ever contain escaped quotes or `[` prefixes.
- **Suggested Fix:** Fix items 1–5 in their respective runtime handlers with targeted unit tests, and harden string/header parsing (e.g., proper quote unescaping and array-of-tables `[[items]]`/`[[monsters]]` header matching) in [build.rs](../../crates/aihack-content/build.rs).
- **Re-audit Method:** Add targeted unit assertions for each of the 6 behaviors (note: if [items.toml](../../crates/aihack-content/src/data/items.toml) or [monsters.toml](../../crates/aihack-content/src/data/monsters.toml) change, keep [r7-content.sha256](../provenance/r7-content.sha256) and [PROVENANCE.md](../../PROVENANCE.md) synchronized).
- **Owner:** Runtime / Content Build
- **Notes:** None.

### `DBG-F006` — Test Fixture Shortcuts & Wrong-Session Serialization in Integration Tests

- **ID:** `DBG-F006`
- **Pass:** Pass 2 (Debug / Engineering Quality)
- **Pattern:** Masked Regression in Integration Suite
- **Area:** [stage6_p12.rs](../../tests/stage6_p12.rs) (lines 519–535), [stage7_p13.rs](../../tests/stage7_p13.rs)
- **Severity:** `Minor`
- **Status:** `Open`
- **Summary:** `tests/stage6_p12.rs` serializes an unmodified session instead of the locked-door session `door_sess`, and P12/P13 tests bypass gameplay progression by directly mutating Turn-0 `xp = 180`, `+30 max_hp`, and `+40 max_mp` in saves.
- **Evidence:** In [stage6_p12.rs](../../tests/stage6_p12.rs) lines 519–535, `door_sess` performs door locking while `sess` (without the locked door) is saved/reloaded—hiding `DBG-F001`. Additionally, [stage6_p12.rs](../../tests/stage6_p12.rs) and [stage7_p13.rs](../../tests/stage7_p13.rs) construct valid-looking `SaveV3` test snapshots by directly boosting `xp = 180`, `max_hp` (`+30`), and `max_mp` (`+40`) on Turn 0 without corresponding monster kills.
- **Expected:** Persistence round-trip tests serialize the actually mutated session, and test fixtures should align with stricter `SaveV3` progression validation or dedicated in-crate test helpers (`XPF-F002`).
- **Actual:** Wrong-session serialization left `DBG-F001` undetected, and direct Turn-0 stat inflation in public tests relies on weak anti-tamper tolerances (`SEC-F001`).
- **Impact:** Lower regression sensitivity on map/progression serialization invariants.
- **Suggested Fix:** Fix `door_sess` serialization in [stage6_p12.rs](../../tests/stage6_p12.rs) alongside `DBG-F001`, and coordinate fixture generation with `SEC-F001` (`XPF-F002`).
- **Re-audit Method:** Verify `door_sess` save/load passes only after `DBG-F001` is fixed and P12/P13 tests pass under tightened `campaign::validate` bounds (`SEC-F001`).
- **Owner:** QA / Runtime Tests
- **Notes:** Coupled with `SEC-F001` via Cross-Pass Conflict `XPF-F002`.

---

## 5. Pass 3: Security Findings

*(Positive Controls Verified: [save.rs](../../crates/aihack-runtime/src/save.rs) `ArtifactStore` enforces `cap-std` directory handles, no-follow symlink guards, single hard-link checks, atomic temp writes, and Unix `0600` file permissions; [service.rs](../../crates/aihack-llm/src/service.rs) and [transport.rs](../../crates/aihack-llm/src/transport.rs) enforce loopback-only HTTP, pinned DNS `resolve_to_addrs`, no redirects/proxies/credentials, `32 KiB`/`64 KiB` body caps, `deny_unknown_fields`, and control-character sanitization.)*

### `SEC-F001` — Loose `SaveV3` Progression Validation Permits Turn-0 XP / HP / MP Tampering

- **ID:** `SEC-F001`
- **Pass:** Pass 3 (Security)
- **Pattern:** Missing Save Integrity Upper-Bound / Cross-Field Invariant
- **Area:** [campaign.rs](../../crates/aihack-runtime/src/campaign.rs) (lines 166–188), [campaign.rs](../../tests/campaign.rs) (lines 162–170)
- **Severity:** `Major`
- **Status:** `Needs Fix`
- **Summary:** `campaign::validate` accepts edited `SaveV3` files with arbitrary XP up to `MAX_XP` (without kill/action budget) plus inflated `max_hp` (`+60`) and `max_mp` (`+40`) on Turn 0.
- **Evidence:** In [campaign.rs](../../crates/aihack-runtime/src/campaign.rs) lines 166–188: `campaign.xp <= MAX_XP && campaign.xp % 10 == 0` enforces no kill-count or elapsed-turn consistency bound; `stats.max_hp <= campaign.max_hp().saturating_add(60)` and `world.max_mp <= campaign.max_mp().saturating_add(40)` permit arbitrary upfront stat buffers regardless of potions/turn/kill state.
- **Expected:** `SaveV3` integrity validation enforces plausible upper bounds derived from turn count, kills/quaffs/progression, and class/level formulas—rejecting Turn-0 forged XP/HP/MP saves as checked by [campaign.rs](../../tests/campaign.rs) lines 162–170.
- **Actual:** A crafted `SaveV3` JSON can grant high experience level, `+60 max_hp`, and `+40 max_mp` immediately at Turn 0 with 0 kills.
- **Impact:** Weakens `SaveV3` anti-tamper trust boundary (`R6` / leaderboard / replay integrity).
- **Suggested Fix:** Bind `campaign.xp` in `campaign::validate` to observed turn/kill progression (or recorded progression counters) and restrict `max_hp`/`max_mp` drift above `campaign.max_hp()`/`campaign.max_mp()` to bounded alchemy/intrinsic counters, updating P12/P13 test fixtures per `XPF-F002`.
- **Re-audit Method:** Verify forged Turn-0 saves with `xp = 180` or un-earned `+60 HP` / `+40 MP` are rejected by `SaveV3` load validation while legitimate leveled/potted saves load cleanly.
- **Owner:** Security / Runtime Persistence
- **Notes:** Conflicts with current P12/P13 test fixtures (`DBG-F006` / `XPF-F002`).

### `SEC-F002` — Unidentified Item Identity and BUC Leakage in P12/P13 Observation, UI, and Logs (`MAJ-03` Partial Regression)

- **ID:** `SEC-F002`
- **Pass:** Pass 3 (Security)
- **Pattern:** Information Disclosure / Oracle Leak of Hidden Item Identity
- **Area:** [observation.rs](../../crates/aihack-runtime/src/observation.rs) (lines 974–981, 1017, 1110–1122, 1144–1149), [play_menu.rs](../../apps/aihack-tui/src/tui/play_menu.rs) (lines 161, 520, 780), [pets.rs](../../crates/aihack-runtime/src/systems/pets.rs) (lines 367, 403), [monster_ai.rs](../../crates/aihack-runtime/src/systems/monster_ai.rs) (lines 331, 980–983, 1035–1038)
- **Severity:** `Major`
- **Status:** `Needs Fix`
- **Summary:** New P12/P13 observation legal-action builders, TUI play menus, and pet/monster logs leak unidentified `WandWishing`, `WandFire`, `PotionWater`, hidden BUC status, and unmasked `item_display_name` values.
- **Evidence:**
  1. [observation.rs](../../crates/aihack-runtime/src/observation.rs) lines 974–981 emit `CommandIntent::Wish` whenever the player carries `ItemKind::WandWishing`, without checking `world.is_item_identified(ItemKind::WandWishing)`.
  2. [play_menu.rs](../../apps/aihack-tui/src/tui/play_menu.rs) lines 161, 520, and 780 expose the `Make a Wish` menu whenever `i.kind == ItemKind::WandWishing`, without checking `i.identified`.
  3. [observation.rs](../../crates/aihack-runtime/src/observation.rs) line 1017 includes `ItemKind::WandFire` in `Engrave { method: Item(..), .. }` legal actions even when unidentified.
  4. [observation.rs](../../crates/aihack-runtime/src/observation.rs) lines 1110–1122 and 1144–1149 inspect `ItemKind::PotionWater` and `item_buc() == Some(BucStatus::Uncursed)` directly to filter `Dip` legal actions without checking `is_item_identified` / `item_buc_known`.
  5. [pets.rs](../../crates/aihack-runtime/src/systems/pets.rs) (lines 367, 403) and [monster_ai.rs](../../crates/aihack-runtime/src/systems/monster_ai.rs) (lines 331, 980–983, 1035–1038) log unmasked `aihack_content::item_display_name` directly.
- **Expected:** Players and LLM agents never learn the true `ItemKind` or uninspected `BucStatus` of unidentified items via `legal_actions`, TUI submenus, or pet/monster event messages (preserving Audit 35 `MAJ-03`).
- **Actual:** Carrying an unidentified `WandWishing` immediately unlocks `Make a Wish` in TUI/legal actions; unidentified `WandFire`, `PotionWater`, hidden BUC, and monster/pet item interactions reveal true item identities/BUC status.
- **Impact:** Breaks hidden-information fairness for both human TUI players and autonomous LLM agents.
- **Suggested Fix:** Gate `Wish` action/menu visibility or allow zapping unidentified wands neutrally (`Zap` triggers wish prompt only on use, or require `identified` for preemptive Wish menu); gate `Engrave`/`Dip` filters on visible/known item facts (or allow attempting `Dip`/`Engrave` on candidate wands/potions without leaking hidden `PotionWater`/BUC truth); route pet/monster item messages through masked appearance labels (`world.masked_item_label`).
- **Re-audit Method:** Add `audit35_remediation` / P12-P13 tests verifying unidentified `WandWishing`, `WandFire`, `PotionWater`, and unidentified-BUC items do not change `legal_actions`, `play_menu`, or `pets`/`monster_ai` log text before identification.
- **Owner:** Runtime / TUI / Security
- **Notes:** Reopens Audit 35 `MAJ-03`.

### `SEC-F003` — Insufficient `SaveV3` Integrity Validation on `ascension_altars` and `Planes 5` Altars (`P13`)

- **ID:** `SEC-F003`
- **Pass:** Pass 3 (Security)
- **Pattern:** Forged Victory Precondition in Save Deserialization
- **Area:** [campaign.rs](../../crates/aihack-runtime/src/campaign.rs) (lines 108–123), [save.rs](../../crates/aihack-runtime/src/save.rs) (lines 435–459, 583–597)
- **Severity:** `Major`
- **Status:** `Needs Fix`
- **Summary:** `campaign::validate` only checks length (`<= 8`), coordinate bounds, and uniqueness on `ascension_altars`, and `save.rs` only verifies `TileKind::Altar` for `altar_alignments`—allowing forged `Planes 5` altar coordinates/alignments to satisfy `RunState::Victory`.
- **Evidence:** [campaign.rs](../../crates/aihack-runtime/src/campaign.rs) lines 108–123 validate `ascension_altars.len() <= 8`, map bounds, and deduplication, without pinning coordinates to `Planes 5`'s 3 generated `TileKind::Altar` coordinates. In [save.rs](../../crates/aihack-runtime/src/save.rs), lines 435–459 check `Victory` against `ascension_altars` and altar alignment match on `Planes 5`, while lines 583–597 only verify that `altar_alignments` entries fall on `TileKind::Altar`.
- **Expected:** `ascension_altars` in `SaveV3` must match the generated `Planes 5` high altar coordinates (`3` altars), and `Planes 5` `altar_alignments` must enforce `{Lawful, Neutral, Chaotic}` invariants consistent with deterministic map generation.
- **Actual:** A tampered `SaveV3` can spoof `ascension_altars` or mutate `Planes 5` altar alignment to achieve invalid `Victory` validation.
- **Impact:** Undermines `SaveV3` Endgame / `Victory` integrity (`R6`).
- **Suggested Fix:** Cross-validate `world.ascension_altars` against the generated `Planes 5` `TileKind::Altar` layout (exact 3 high altar positions) and enforce immutable `{Lawful, Neutral, Chaotic}` alignment assignment on `Planes 5` inside [campaign.rs](../../crates/aihack-runtime/src/campaign.rs) and [save.rs](../../crates/aihack-runtime/src/save.rs).
- **Re-audit Method:** Verify forged `SaveV3` files with altered `ascension_altars` coordinates or duplicated/wrong `Planes 5` altar alignments fail load validation, while genuine Astral runs load and win cleanly.
- **Owner:** Security / Runtime Persistence
- **Notes:** Interacts with `IMP-F001` via Cross-Pass Conflict `XPF-F001`.

### `SEC-F004` — Transitive `cargo audit` Unsound Warnings (`rustix 1.1.4`, `tokio 1.52.4`)

- **ID:** `SEC-F004`
- **Pass:** Pass 3 (Security)
- **Pattern:** Allowed Transitive Dependency Advisory
- **Area:** Workspace dependency graph (`Cargo.lock`, [deny.toml](../../deny.toml))
- **Severity:** `Minor`
- **Status:** `Accepted Risk`
- **Summary:** `cargo audit` exited `0` across 318 scanned crates with 2 allowed `unsound` warnings: `rustix 1.1.4` (`RUSTSEC-2026-0486`) and `tokio 1.52.4` (`RUSTSEC-2026-0386`).
- **Evidence:** `task-493` `cargo audit` output recorded `GATE_AUDIT=0` (`warning: 2 allowed warnings found`: `rustix 1.1.4` `RUSTSEC-2026-0486` `unsound`, `tokio 1.52.4` `RUSTSEC-2026-0386` `unsound`), and `cargo deny check licenses bans sources` returned `GATE_DENY=0`. Workspace crates contain 0 `unsafe` blocks.
- **Expected:** Zero unaddressed high/critical CVEs and tracked visibility on allowed transitive unsound advisories.
- **Actual:** Two `unsound` advisories remain allowed warnings in transitive dependencies; 0 vulnerabilities fail `cargo audit` or `cargo deny`.
- **Impact:** Minimal immediate exposure on Windows / loopback HTTP client usage; monitor upstream crate patch releases.
- **Suggested Fix:** Upgrade `rustix` and `tokio` in `Cargo.lock` when compatible patched releases drop during routine dependency maintenance.
- **Re-audit Method:** Re-run `cargo audit` and `cargo deny check licenses bans sources`.
- **Owner:** Maintainer / Supply Chain
- **Notes:** Recorded in Section 8 (`Accepted Risks`).

---

## 6. Cross-Pass Conflicts

### `XPF-F001` — P10 Altar Conversion (`IMP-F001`) vs P13 Astral High Altar Immutability (`SEC-F003`)

- **Linked Findings:** `IMP-F001` (Pass 1) ↔ `SEC-F003` (Pass 3)
- **Conflict Summary:** P10 deliberately allows converting cross-aligned dungeon altars via `offer` ([altar.rs](../../crates/aihack-runtime/src/systems/altar.rs)), which requires `SaveV3` to trust mutable `altar_alignments` ([save.rs](../../crates/aihack-runtime/src/save.rs)). However, P13 requires the 3 `Planes 5` Astral high altars to remain fixed `{Lawful, Neutral, Chaotic}` endpoints for `Victory`.
- **Resolution Rule:** Preserve mutable `altar_alignments` ONLY for non-Astral altars (`level_id != (Planes, 5)`), forbid cross-aligned conversion on `Planes 5` / `ascension_altars` in `altar::offer`, and enforce fixed high-altar coordinates + alignments on `Planes 5` in `SaveV3` validation.

### `XPF-F002` — Integration Test Fixture Shortcuts (`DBG-F006`) vs Strict SaveV3 Progression Caps (`SEC-F001`)

- **Linked Findings:** `DBG-F006` (Pass 2) ↔ `SEC-F001` (Pass 3)
- **Conflict Summary:** External integration tests (`tests/stage6_p12.rs`, `tests/stage7_p13.rs`) mutate Turn-0 `xp = 180` and `+30 max_hp` / `+40 max_mp` directly on saved state to test high-level spells/endgame flows quickly. Tightening `campaign::validate` (`SEC-F001`) to reject un-earned Turn-0 stat jumps will break tests that edit raw fields without recording legitimate progression.
- **Resolution Rule:** Tighten `campaign::validate` against un-earned Turn-0 XP/HP/MP spoofing without introducing unsafe `#[cfg(test)]` bypasses in production binaries; update P12/P13 integration tests to earn XP/HP/MP via valid gameplay actions or seeded combat steps (plus fix `door_sess` serialization in [stage6_p12.rs](../../tests/stage6_p12.rs)).

---

## 7. Required Fixes Before PASS

All **10 blocking `Needs Fix` items (2 Critical + 8 Major)** plus the **3 Minor `Needs Fix` items** and **`DBG-F006` (`Open` serialization defect)** should be resolved before Audit 37 re-audit:

1. **Critical Gate Blockers (2):**
   - `DBG-F001`: Allow `Closed | Open | HiddenDoor -> Locked` door transitions in [campaign_map.rs](../../crates/aihack-runtime/src/campaign_map.rs) lines 1164–1179; fix `door_sess` serialization in [stage6_p12.rs](../../tests/stage6_p12.rs) lines 519–535.
   - `DBG-F002`: Allow `Trap(Pit) | HiddenTrap(Pit) -> Floor | Boulder` in Sokoban inside [campaign_map.rs](../../crates/aihack-runtime/src/campaign_map.rs) lines 1148–1163; add 2-boulder plugged-pit round-trip save test.
2. **Major Gate Blockers (8):**
   - `IMP-F001` & `SEC-F003` (`XPF-F001`): Protect `Planes 5` high altars from conversion in [altar.rs](../../crates/aihack-runtime/src/systems/altar.rs) and enforce `Planes 5` `ascension_altars` + alignment immutability in [campaign.rs](../../crates/aihack-runtime/src/campaign.rs) and [save.rs](../../crates/aihack-runtime/src/save.rs).
   - `IMP-F002`: Fix `spell_failure_percent` (`campaign.level()`, `0..=100` policy), `study_spellbook` (`_rng` / cursed blind + `1d6`), and `cast_at` MP/skill-practice order in [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs).
   - `IMP-F003`: Add `"npc"` AI flags to shopkeepers, priests, oracle, watchmen, quest nemesis, and unique endgame bosses/riders in [monsters.toml](../../crates/aihack-content/src/data/monsters.toml) (updating [r7-content.sha256](../provenance/r7-content.sha256) + [PROVENANCE.md](../../PROVENANCE.md)); initialize `pet_hunger` in `CharmMonster` / `CreateFamiliar`.
   - `IMP-F004`: Implement actual invisibility mutation for `WandEffect::MakeInvisible`, enforce wish enchantment policy cleanly, and add artifact alignment blast check on wished artifacts in [projectiles.rs](../../crates/aihack-runtime/src/systems/projectiles.rs).
   - `DBG-F003`: Scope `on_shopkeeper_died` debt/unpaid clearance per shopkeeper/floor and fix Oracle `set_item_enchant_known` + `GameEvent::TileRevealed` in [npc.rs](../../crates/aihack-runtime/src/systems/npc.rs).
   - `SEC-F001` (`XPF-F002`): Enforce kill/turn/progression consistency bounds for `xp`, `max_hp`, and `max_mp` in [campaign.rs](../../crates/aihack-runtime/src/campaign.rs) and update Turn-0 shortcut test fixtures (`DBG-F006`).
   - `SEC-F002`: Eliminate unidentified item/BUC leakage across `observation.rs` (`WandWishing`, `WandFire`, `PotionWater`, BUC dip filter), `play_menu.rs` (`Make a Wish`), and `pets.rs` / `monster_ai.rs` logs.
3. **Minor Fixes & Open Cleanup (4):**
   - `IMP-F005`: Sync Stage 7 `campaign_spec.md` level IDs & pursuit path, `session.rs` & `render_panels.rs` startup/victory copy, `PROVENANCE.md` `DEP-EXC-0001` expiry (`2027-02-20`), and add Stage 7 to `IMPLEMENTATION_SUMMARY.md`.
   - `DBG-F004`: Exclude non-diggable `Sokoban` and `Planes 2..5` walls from `observation.rs` directional `Dig` actions.
   - `DBG-F005`: Fix the 5 localized gameplay edge cases (`death.rs` lifesaving & kill/gold attribution, `monster_ai.rs` AC slot/stacking & reflected death undead immunity, `artifacts.rs` `DoorChanged` event) and harden [build.rs](../../crates/aihack-content/build.rs) TOML line parser.
   - `DBG-F006`: Fix `door_sess` serialization in [stage6_p12.rs](../../tests/stage6_p12.rs) and align P12/P13 fixture progression with `SEC-F001`.

---

## 8. Accepted Risks

- **`SEC-F004` (`Accepted Risk`):** Transitive `cargo audit` unsound warnings on `rustix 1.1.4` (`RUSTSEC-2026-0486`) and `tokio 1.52.4` (`RUSTSEC-2026-0386`). Both are currently allowed (`cargo audit` exits `0`), `cargo deny` passes (`GATE_DENY=0`), and workspace crates contain zero `unsafe` blocks. Track for upstream `Cargo.lock` patch updates when compatible versions release.

---

## 9. Needs Spec Clarification

- **`IMP-F006` (`Needs Clarification`):** Clarify whether mithril armor incurs a metal spell-failure penalty or is exempt, reconciling `ADR-P12-01` in [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md) (line 52 listing `iron`/`steel`/`mithril`) with [spells.rs](../../crates/aihack-runtime/src/systems/spells.rs) (checking `iron | steel`) and [stage6_p12.rs](../../tests/stage6_p12.rs) (line 270 asserting `"mithril incurs no metal penalty"`).
- Also confirm exact intended MP spend/refund and `0..=100%` clamp details in `ADR-P12-01` / [campaign_spec.md](../campaign_spec.md) when resolving `IMP-F002`.

---

## 10. Re-audit Checklist

When verifying remediation in Audit 37, confirm all of the following:
- [ ] `DBG-F001`: `SkeletonKey`, `WizardLock`, and `WandEffect::Locking` saves pass `validate_persisted_map` and reload (`door_sess` fixed in `tests/stage6_p12.rs`).
- [ ] `DBG-F002`: Sokoban save with a 2nd boulder pushed over a previously plugged pit passes `validate_persisted_map` and reloads.
- [ ] `IMP-F001` + `SEC-F003`: `Planes 5` altars cannot be converted via `offer`, and forged `ascension_altars` / `Planes 5` alignments fail `SaveV3` load validation.
- [ ] `IMP-F002` + `IMP-F006`: `spell_failure_percent`, `study_spellbook`, `cast_at`, and mithril policy in `DESIGN_DECISIONS.md` / `campaign_spec.md` / `spells.rs` match 100%.
- [ ] `IMP-F003`: `monsters.toml` tags unique NPCs/bosses/riders with `"npc"`, `r7-content.sha256` & `PROVENANCE.md` hashes match, and charmed/created pets start with positive `pet_hunger`.
- [ ] `IMP-F004`: `WandEffect::MakeInvisible` mutates target invisibility state, `wish` rejects/handles invalid enchant bounds per spec, and wished artifacts check alignment blast.
- [ ] `DBG-F003`: Killing a `Main 2` shopkeeper preserves `Mines 1` unpaid debt; Oracle sets `enchant_known` and emits `GameEvent::TileRevealed`.
- [ ] `SEC-F001` + `DBG-F006`: Forged Turn-0 XP/HP/MP saves are rejected by `campaign::validate`, and all P12/P13 tests pass without raw Turn-0 tampering.
- [ ] `SEC-F002`: Unidentified `WandWishing`, `WandFire`, `PotionWater`, BUC status, and `pet`/`monster_ai` item names never leak in `legal_actions`, `play_menu`, or logs.
- [ ] `IMP-F005`, `DBG-F004`, `DBG-F005`: Docs/UI strings/expiry/`IMPLEMENTATION_SUMMARY.md` synced; non-diggable `Dig` legal actions suppressed; 6 `DBG-F005` edge cases resolved.
- [ ] All 8 local workspace gates (`fmt`, `clippy`, `audit`, `deny`, `r7_checkpoint`, `r8_checkpoint`, `cargo test --workspace --all-targets --locked`, `cargo build --release --locked`) pass after all fixes.

---

## 11. Final Decision

- **Verdict:** **`HOLD`**
- **Rationale:** Although all 8 build, lint, checkpoint, and test gates passed on the current Stage 7 working tree (571 tests passing across 103 suites), independent 3-pass inspection uncovered **2 Critical save-corruption defects** (`DBG-F001`, `DBG-F002`), **8 Major compliance/gameplay/security defects** (`IMP-F001`..`IMP-F004`, `DBG-F003`, `SEC-F001`..`SEC-F003`), **3 Minor fixes** (`IMP-F005`, `DBG-F004`, `DBG-F005`), **1 Open test defect** (`DBG-F006`), **1 Spec clarification** (`IMP-F006`), and **2 Cross-pass conflicts** (`XPF-F001`, `XPF-F002`). Under [AI_AUDIT_DOC_STANDARD.md](../../AI_AUDIT_DOC_STANDARD.md) Section 9, Critical and Major `Needs Fix` findings strictly block `PASS` until resolved and re-audited.

---

## 12. Coder Handoff

Copy and pass the following handoff prompt directly to the implementation session to remediate Audit 36 findings cleanly:

```text
AIHack Audit 36 결과(판정: HOLD)를 반영하여 아래 파일의 모든 차단 결함과 연계 문서를 수정해 주세요.

참조 감사 보고서: docs/audit/audit_report_36.md

[필수 차단 수정 (Critical 2 + Major 8 + Open Test 1)]
1. DBG-F001 (Critical): crates/aihack-runtime/src/campaign_map.rs(L1164-1179)에서 Closed/Open/HiddenDoor -> Locked 문 상태 전이를 허용하고, tests/stage6_p12.rs(L519-535)에서 sess 대신 door_sess를 직렬화/재로드하도록 수정.
2. DBG-F002 (Critical): crates/aihack-runtime/src/campaign_map.rs(L1148-1163)에서 Sokoban 분기의 (Trap(Pit)|HiddenTrap(Pit), Floor | Boulder) 전이를 허용하고 2번째 바위가 메워진 구덩이 위로 이동한 SaveV3 왕복 테스트 추가.
3. IMP-F001 + SEC-F003 (XPF-F001, Major): crates/aihack-runtime/src/systems/altar.rs(L104-141)에서 Planes 5 / ascension_altars 개종을 차단하고, crates/aihack-runtime/src/campaign.rs(L108-123) 및 save.rs(L435-459, L583-597)에서 Planes 5 제단 3개 좌표 및 {Lawful, Neutral, Chaotic} 정렬 불변성을 SaveV3 로드 시 검증.
4. IMP-F002 + IMP-F006 (Major + Clarification): ADR-P12-01(DESIGN_DECISIONS.md L52)의 mithril 갑옷 패널티 문구와 spells.rs/campaign_spec.md 기준을 하나로 확정/정합화하고, crates/aihack-runtime/src/systems/spells.rs(L50-210)의 실패율 공식(campaign.level(), 0..=100%), study_spellbook(_rng 및 저주 패널티), cast_at MP 선차감/스킬 숙련 처리 수정.
5. IMP-F003 (Major): crates/aihack-content/src/data/monsters.toml에 상점주인/사제/신탁/경비병/퀘스트 네메시스/최종 보스 및 기사들에 "npc" AI 플래그 추가(docs/provenance/r7-content.sha256 및 PROVENANCE.md 동시 갱신), CharmMonster/CreateFamiliar 생성 시 pet_hunger 초기화.
6. IMP-F004 (Major): crates/aihack-runtime/src/systems/projectiles.rs(L117-201, L590-602)에서 WandEffect::MakeInvisible의 실제 투명 상태 부여, wish 인챈트 범위 처리, 소원 아티팩트 반대 정렬 폭발 체크 구현.
7. DBG-F003 (Major): crates/aihack-runtime/src/systems/npc.rs(L230-260, L387-437)에서 상점주인 사망 시 해당 상점/층의 채무·미납품만 정리하도록 범위 제한, 신탁 감정 시 set_item_enchant_known 호출 및 GameEvent::TileRevealed 이벤트 발행.
8. SEC-F001 + DBG-F006 (XPF-F002, Major + Open): crates/aihack-runtime/src/campaign.rs(L166-188)에서 Turn-0 xp/max_hp/max_mp 임의 변조 차단을 강화하고, tests/stage6_p12.rs 및 tests/stage7_p13.rs 테스트 픽스처도 정당한 진행 경로에 맞게 정비.
9. SEC-F002 (Major, MAJ-03 재발): observation.rs(L974-981 WandWishing, L1017 WandFire 각인, L1110-1149 PotionWater/BUC 담그기 필터), apps/aihack-tui/src/tui/play_menu.rs(L161, L520, L780 Make a Wish 메뉴), pets.rs / monster_ai.rs 직접 item_display_name 로그 호출에서 미감정 아이템/BUC 정보 유출 제거.

[추가 정리 (Minor 3)]
10. IMP-F005: docs/campaign_spec.md(L349-357) 분기 ID·추적 경로, session.rs(L231,271) 및 render_panels.rs(L612,717) 안내 문구, PROVENANCE.md(L102 DEP-EXC-0001 만료일 2027-02-20), IMPLEMENTATION_SUMMARY.md Stage 7 항목 동기화.
11. DBG-F004: observation.rs(L941-957)에서 Sokoban 및 Planes 2..5 비굴착 벽 방향 Dig 액션 필터 제거.
12. DBG-F005: death.rs(AmuletLifeSaving status_timers/AC 초기화, 비플레이어 킬/골드 미지급), monster_ai.rs(갑옷 AC 중첩 방지, 반사 죽음 광선 undead 면역), artifacts.rs(DoorChanged 이벤트 발행), crates/aihack-content/build.rs 파서 보강.

[검증 게이트]
수정 후 아래 8개 게이트 모두 EXIT 0 확인 후 Audit 37 재감사를 요청하세요:
- cargo fmt --all -- --check
- cargo clippy -j 2 --workspace --all-targets --locked -- -D warnings
- cargo audit && cargo deny check licenses bans sources
- bash scripts/r7_checkpoint.sh && bash scripts/r8_checkpoint.sh
- cargo test -j 2 --workspace --all-targets --locked --no-fail-fast
- cargo build -j 2 --workspace --all-targets --release --locked
```
