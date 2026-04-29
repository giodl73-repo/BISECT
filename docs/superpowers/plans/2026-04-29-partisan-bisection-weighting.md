# Plan 03: Partisan Edge-Weighting at the Bisection Layer

> **For agentic workers:** Do not start until Plan 01 (cutover) and Plan 02 (archive + deletion) are complete. The point of the deletion is to avoid mid-port new-feature drift; this plan is the resumption of paused feature work.

**Date:** 2026-04-29
**Updated:** 2026-04-29 (v2 — incorporates MERIDIAN, DATUM, TRENCH, COVENANT review findings)
**Goal:** Add partisan-aware edge-weighting to `redist-core` so the bisection algorithm can preserve partisan clusters when a state's stated political goals require a particular delegation outcome. Distinct from the existing `redist-analysis/src/partisan.rs` (post-hoc metrics) and `redist-cli/src/partisan.rs` (CLI subcommand).

**Depends on:** Plans 01 + 02 complete.
**Status:** WIP code stashed in `stash@{0}` (created 2026-04-29). Not restored until this plan is approved.

---

## Background

After *Louisiana v. Callais* (2026-04-29), states may use partisan motive in redistricting (Rucho-protected) but may not openly mix race-conscious and partisan signals (Callais p. 36 disentanglement requirement). A constraint-aware bisection that preserves partisan clusters serves three legitimate post-Callais use cases:

1. **State map-draw mode:** state hits a partisan target with maximally compact districts.
2. **Challenger evidence mode:** show that the state's stated partisan target could have been achieved with better minority outcomes (Callais p. 23 strong-inference test).
3. **Commission/academic mode:** explore the trade-off between partisan goals and compactness.

The algorithm is the same in all three modes; the constraints encoded are what differ. See spec at `docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md` for context.

---

## Naming Conflict

Two files named `partisan.rs` already exist in the workspace:
- `redist/crates/redist-analysis/src/partisan.rs` — post-hoc metrics on completed partitions
- `redist/crates/redist-cli/src/partisan.rs` — CLI subcommand wiring

The stashed work introduces a third partisan module in `redist-core` that does **constraint encoding for the bisection** (a different layer entirely). To avoid confusion, the stashed file is renamed:

| File | New name | Purpose |
|---|---|---|
| `redist-core/src/partisan.rs` (stashed) | `redist-core/src/partisan_weights.rs` | Edge-weight construction for bisection |
| `redist-analysis/src/partisan.rs` (existing) | unchanged | Post-hoc metrics |
| `redist-cli/src/partisan.rs` (existing) | unchanged | CLI subcommand |

---

## Task 1: Restore stash with chain-of-custody for Callais reference (REVISED in v2)

The Callais PDF is being deliberately not committed (`.gitignore` excludes `*.pdf`), but the legal grounding for this code change must be auditable. **Per COVENANT, TRENCH review:** replace the manual "don't commit" reminder with a structural artifact + a structural prevention.

- [ ] **1.1** Create a feature branch: `git checkout -b feature/partisan-bisection-weighting`.
- [ ] **1.2** Compute SHA-256 of the local `.callais_ruling.pdf`: `certutil -hashfile .callais_ruling.pdf SHA256` (Windows) or `sha256sum .callais_ruling.pdf`.
- [ ] **1.3** Create `docs/legal/CALLAIS_REFERENCE.md` containing:
   ```
   # Louisiana v. Callais — Reference

   Citation:        608 U.S. ___ (2026)
   Decided:         2026-04-29
   Vote:            6-3 (Alito; Thomas concurring with Gorsuch; Kagan dissenting with Sotomayor, Jackson)
   Disposition:     732 F. Supp. 3d 574 affirmed and remanded
   Source URL:      https://www.supremecourt.gov/opinions/25pdf/24-109_21o3.pdf
   PDF SHA-256:     <hash from step 1.2>
   Date accessed:   2026-04-29

   This case is the legal grounding for the partisan-weighting feature
   in redist-core/src/partisan_weights.rs. The feature implements the
   Callais p. 23 strong-inference framework: a §2 challenger demonstrates
   intentional racial discrimination by showing that the State's stated
   partisan goals could have been achieved with better minority outcomes.
   ```
- [ ] **1.4** Add a pre-commit hook at `.git/hooks/pre-commit` (and document in `docs/CONTRIBUTING.md`) that blocks any `*.pdf` file from being staged unless `ALLOW_PDF=1` env var is set. This is structural prevention of PP-17.
- [ ] **1.5** Pop the stash: `git stash pop`. Expect conflicts on `redist-core/src/lib.rs` and `redist_py/src/lib.rs` because both have evolved on the merged main.
- [ ] **1.6** Resolve `redist-core/src/lib.rs`: the merged file has `population.rs`, `fips.rs`, etc. Re-apply the partisan module declaration and `pub use` against the current structure.
- [ ] **1.7** Resolve `redist_py/src/lib.rs`: re-apply the PyO3 wrapper for `build_partisan_edge_weights` against the current import block.
- [ ] **1.8** Move the `.callais_ruling.pdf` to a personal location outside the repo. The reference doc + SHA stays.

**Exit:** Stash popped, conflicts resolved, working tree compiles. Callais reference doc committed. Pre-commit hook installed.

---

## Task 2: Rename the module

**File:** `redist/crates/redist-core/src/partisan.rs` → `redist/crates/redist-core/src/partisan_weights.rs`

- [ ] **2.1** Rename the file with `git mv`.
- [ ] **2.2** Update `redist-core/src/lib.rs`: `pub mod partisan_weights;` and `pub use partisan_weights::build_partisan_weights;`.
- [ ] **2.3** Rename the function: `build_partisan_edge_weights` → `build_partisan_weights` (shorter; crate name disambiguates from analysis-layer partisan).
- [ ] **2.4** Update the PyO3 wrapper to use the new path/name.
- [ ] **2.5** Run `cargo test -p redist-core --lib partisan_weights::` — all 13 tests should pass against the renamed module.

**Exit:** Module renamed, tests pass.

---

## Task 3: Verify the module fits the merged architecture

The merged main has `redist-core/src/population.rs` and `fips.rs` — modules I didn't have when writing the stashed code. Check whether they offer abstractions the partisan module should be using.

- [ ] **3.1** Read `redist-core/src/population.rs` and `fips.rs`. Look for: per-unit weight handling, balance-checking patterns, FIPS-keyed lookups.
- [ ] **3.2** Look at how `vra.rs` interacts with population/balance — does the merged `vra.rs` differ from the version the stashed code was built against?
- [ ] **3.3** If patterns have evolved, refactor `partisan_weights.rs` to match. If not, leave it.

**Exit:** Module fits the current architectural style.

---

## Task 4: Wire into the CLI with structural disentanglement guard (REVISED in v2)

**Per MERIDIAN review:** Callais p. 36 requires that race-conscious and partisan signals not be mixed in production map runs. Make this structurally impossible, not merely documented.

The existing `redist/crates/redist-cli/src/partisan.rs` handles partisan analysis (post-hoc). It needs to gain a new path: when invoked with constraint inputs, it produces a partition that respects them.

- [ ] **4.1** Add a CLI flag: `redist state --partition-mode partisan-weighted` activates the new path. Mode enum entries are mutually exclusive in the type system — `partisan-weighted` and `metis-vra` (existing VRA mode) cannot both be active. The compiler enforces this; no runtime check needed.
- [ ] **4.2** Define the input format: per-tract D-share TSV per the spec's File Formats section. Header: `geoid<tab>dem_share`. GEOIDs are 11-character TIGER strings with leading zeros. dem_share is float in [0.0, 1.0].
- [ ] **4.3** The partisan-share producer lives in Rust (`redist-data/src/partisan_shares.rs`) — see Task 5.
- [ ] **4.4** Wire `build_partisan_weights` call into the runner: load shares, call the function, pass resulting edge weights into the existing METIS-edge-weighting code path that VRA mode uses.
- [ ] **4.5** Add a hard error if both `--partisan-shares` and a VRA-mode flag are passed: `Error: --partisan-shares is mutually exclusive with --partition-mode metis-vra (Callais disentanglement)`.

---

## Task 5: Producer for per-tract D-share

**Files (new):**
```
redist/crates/redist-data/src/partisan_shares.rs
docs/file-formats/partisan-shares.md  (format spec)
```

Produces per-tract Democratic vote share by joining precinct-level election results to tracts.

- [ ] **5.1** Decide input format. Options:
   - MGGG OpenPrecincts (has shapefiles + results, but format varies by state)
   - State SOS sources (varies by state)
   - User-provided CSV with `precinct_id, dem_votes, rep_votes`
- [ ] **5.2** Decide aggregation method. Tract-to-precinct overlap, area-weighted vote allocation. Existing geometry tools in `redist-data` should suffice.
- [ ] **5.3** Output a normalized per-tract D-share file in `outputs/data/{year}/partisan/{state}/dem_shares.tsv` per the spec's format.
- [ ] **5.4** Document the format in `docs/file-formats/partisan-shares.md`: byte layout (UTF-8 TSV), GEOID format (11-char TIGER with leading zeros), share range [0.0, 1.0], header row required, validation rules.
- [ ] **5.5** Add unit tests for the aggregation against a known-state fixture.

**Exit:** `redist analyze partisan-shares --state VT --year 2020` produces a per-tract D-share file.

---

## Task 6: End-to-end test with quantitative success criteria (REVISED in v2)

**Per DATUM review:** "more preserved" is a vague qualifier. Replace with measurable thresholds.

- [ ] **6.1** Run `redist state --state VT --year 2020 --partition-mode partisan-weighted --partisan-shares outputs/data/2020/partisan/vt/dem_shares.tsv`.
- [ ] **6.2** Verify the output is a valid partition: population balance ≤ 0.5%, contiguity, correct district count.
- [ ] **6.3** **Per MERIDIAN review:** Verify population balance is enforced at every bisection-tree level, not just final districts. Add a test that walks the BisectionTree and asserts each intermediate node respects its local ufactor tolerance.
- [ ] **6.4** Compute partisan-cluster preservation metric: define as the sum, over all districts, of `|dem_share_district - dem_share_state| × population_district`. Higher = more concentrated partisan distribution = clusters preserved.
- [ ] **6.5** Compare partisan-weighted vs unweighted runs for the same state:
   - **Required:** partisan-cluster preservation metric ≥ 5% higher in weighted run
   - **Required:** mean Polsby-Popper degradation ≤ 2% (compactness preserved)
   - **Required:** all hard gates from 6.2 still pass
- [ ] **6.6** Repeat for AL or LA where the effect is more pronounced.
- [ ] **6.7** Document the run inputs, outputs, and metric values in `docs/superpowers/plans/2026-04-29-partisan-bisection-weighting-results.md` (or similar) for posterity.

**Exit:** Quantitative success criteria met. Comparison documented.

---

## Task 7: Documentation

- [ ] **7.1** Update `docs/REDIST_CLI.md` with the new flag(s) and example invocation.
- [ ] **7.2** Add a short section to the architecture spec at `docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md` noting partisan edge-weighting is now in the production crate.
- [ ] **7.3** Cite *Louisiana v. Callais* via the reference doc (`docs/legal/CALLAIS_REFERENCE.md`) as the legal grounding.
- [ ] **7.4** Add a section to `docs/REDIST_CLI.md` clearly stating that `--partisan-shares` and `--partition-mode metis-vra` cannot be used in the same run, and why (Callais p. 36 disentanglement).

---

## Out of Scope

- Delegation-ratio targeting ("force 5R-1D"): different problem at the bisection-tree level, not edge-weighting. Tracked as a future enhancement.
- Mixing partisan and VRA edge-weighting in one map run: deliberately not supported (Callais SB8 fact pattern). Code structure makes it impossible to specify both at once (Task 4.1, 4.5).

## Status

Not started. WIP code in `stash@{0}` from 2026-04-29. Resume after Plans 01-02.

## v2 Changelog

- **TRENCH/COVENANT:** Callais reference doc + SHA-256 + pre-commit PDF block (replaces manual reminder)
- **MERIDIAN:** structural mutual exclusion of partisan and VRA modes (compiler-enforced)
- **MERIDIAN:** tree-level invariant check (every bisection node respects local ufactor)
- **DATUM:** quantitative success criteria for Task 6 (≥5% cluster preservation gain, ≤2% PP degradation)
