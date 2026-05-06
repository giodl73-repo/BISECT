# Revision Plan — A.4 Replication Materials

**Status**: Round 2 complete

**Current round**: 2
**Round 1 avg score**: 3.10 / 4.0
**Round 2 avg score**: 3.46 / 4.0
**Stage**: R2 reviews complete; passes target ≥ 3.3/4.0

## Round 1 Scores

| Reviewer | R1 | Notes |
|---|---|---|
| Karypis | 3.2 | METIS build config gap; `--verify-sha256` CLI error; VT fixture spec incomplete |
| Rodden | 3.0 | Election data provenance missing; PL 94-171 processing unclear; output path unusual |
| Duchin | 3.3 | "Byte-identical" claim needs arch qualification; hash concatenation ambiguous |
| Stephanopoulos | 3.1 | Legal outputs (population deviation, MM counts, contiguity) not in SHA chain docs |
| Liang | 2.9 | Blocking CLI error; placeholder publication plan missing; Python 3.13 requirement |
| **Average** | **3.10** | Above 3.0 bar; P1 items required |

## Round 2 Scores

| Reviewer | R2 | Notes |
|---|---|---|
| Karypis | 3.5 | CLI error fixed; byte-identical qualified; VT fixture params still missing |
| Rodden | 3.4 | Election data provenance addressed; Harvard-Fekrazad DOI still missing |
| Duchin | 3.5 | Byte-identical fully resolved; build hash binary/ASCII ambiguity still open |
| Stephanopoulos | 3.4 | Legal outputs in SHA chain — fully addressed; AEA vs litigation gap P2 |
| Liang | 3.5 | Blocking CLI error fixed; reference run protocol P2; Python 3.13 note P2 |
| **Average** | **3.46** | Passes ≥ 3.3 target |

## Priority 1 (P1) — Required Before R2

### P1-A: Fix CLI flag `--verify-sha256` → `--verify-downloads`
**Location**: Section 3, "Fetching Data with `redist fetch`" subsection
**Issue**: The command `redist fetch --year 2020 --verify-sha256` uses a flag that does not exist in the CLI. The actual flag is `--verify-downloads` (confirmed in `FetchArgs` struct, `args.rs`). This is a blocking error: a researcher following the documented procedure will receive an "unrecognized argument" error.
**Fix**: Change `--verify-sha256` to `--verify-downloads` in all occurrences in Section 3.
**Flagged by**: Karypis, Liang (P1 critical)

### P1-B: Add election data provenance
**Location**: Section 3 (Data Provenance)
**Issue**: The package documents Census redistricting files (PL 94-171) and TIGER/Line shapefiles but omits the election data source used for all partisan analyses (B.11, C.5, and others). A researcher trying to independently reproduce partisan results cannot do so without knowing which election dataset was used, at what version, and how it was processed.
**Fix**: Add a subsection "Election Data" documenting the source (VEST election data or MIT Election Lab, at the specific version used), the download path, and whether `redist fetch --type elections` retrieves it automatically. If election data is not part of the reproducible pipeline, this must be explicitly stated.
**Flagged by**: Rodden

### P1-C: Specify "byte-identical" architecture constraint
**Location**: Section 1 (Introduction) and Section 5 (Verification)
**Issue**: Section 1 claims "byte-identical district assignments" from any two researchers. Section 5 discloses that METIS may differ across architectures (x86-64 vs arm64). The mismatch between the strong Section 1 claim and the Section 5 qualification creates a false impression. The Section 2 note that "all results were generated on Windows 11 with an AMD Ryzen 9 processor" implies x86-64 but does not state it explicitly.
**Fix**: Change Section 1's claim to "byte-identical on the same hardware architecture" and add a sentence in Section 5 specifying that the published reference hashes are for x86-64 (AMD/Intel). The published separate arm64 hashes should also be noted.
**Flagged by**: Duchin, Karypis

### P1-D: Placeholder publication workflow
**Location**: Sections 3, 4, 5, 6 (Tables 1, 2, 3, 4 all have placeholder hashes)
**Issue**: All SHA-256 values in the package are placeholders. The paper correctly notes they will be filled after a clean run, but does not document the process for doing so: who runs the reference run, on which hardware, when, and how the hashes are published. Without this, a researcher reading the pre-publication version does not know when or how to obtain the verified values.
**Fix**: Add a subsection in Section 5 ("Reference Run Protocol") describing: the reference hardware (AMD Ryzen 9, 32 GB RAM, Windows 11 x86-64), the `pin.sh` pattern (modeled on the Vermont fixture), the repository location for confirmed hashes (`examples/vermont-2020-walkthrough/pin.sh`), and a commitment to populate all tables before portfolio publication.
**Flagged by**: Liang

## Priority 2 (P2) — Recommended Before Journal Submission

### P2-A: Specify Vermont walkthrough synthetic configuration
**Location**: Section 6, Vermont walkthrough
**Issue**: The fixture uses "a synthetic two-seat configuration" on Vermont's tract data, but the specific parameters are not documented. What seat count, what METIS parameters, what is changed from the standard configuration?
**Fix**: Add: "The fixture sets `--seats 2` on Vermont's 184 census tracts, producing a single bisection. All other parameters match the standard configuration: population tolerance ±0.5%, METIS seed 42, edge-weight scaling factor α=5.0, T=600."
**Flagged by**: Karypis, Duchin

### P2-B: Clarify build hash construction (binary vs ASCII concatenation)
**Location**: Section 5, Build hash description
**Issue**: "SHA-256 of the concatenated SHA-256 values" is ambiguous: hex strings as ASCII, binary hash values, or a hash tree? These produce different results. An independent implementer will get the wrong answer without clarification.
**Fix**: Specify: "The build hash is the SHA-256 of the binary concatenation of the SHA-256 digest values (32 bytes each) of all input files in FIPS-sorted order."
**Flagged by**: Duchin

### P2-C: Confirm legally relevant outputs in SHA chain
**Location**: Section 5, Analysis hash description
**Issue**: For redistricting litigation purposes, the SHA chain's coverage of population deviation tables, majority-minority district counts, and contiguity certificates needs to be explicitly confirmed.
**Fix**: Add to the analysis hash description: "The analysis hash includes: district assignment CSV, population deviation by district, majority-minority population percentages by district, and a contiguity certificate (one flag per district)."
**Flagged by**: Stephanopoulos

### P2-D: Python version requirement
**Location**: Section 2, Software Stack table
**Issue**: Python 3.13+ is listed as a requirement. Python 3.13 is not the default on most Linux distributions or macOS. The package should clarify whether 3.13 is a hard requirement or a recommendation.
**Fix**: Add footnote: "Python 3.13 is required for the dashboard and research scripts. Python 3.11 or 3.12 may work for core redistricting functions but has not been tested."
**Flagged by**: Liang

### P2-E: Output path double-year notation
**Location**: Section 4, Step 4
**Issue**: `outputs/official_2020/2020/{state}/` shows a double-year path that may be a documentation error or an unusual path design. This should be confirmed.
**Fix**: Verify the actual output path structure and either confirm or correct the documentation.
**Flagged by**: Rodden

### P2-F: PL 94-171 processing pipeline description
**Location**: Section 3
**Issue**: PL 94-171 files come in a multi-segment format. Independent verification of the build hash requires knowing exactly which files are hashed (raw downloads? extracted aggregates?) and in what order.
**Fix**: Add a note: "The build hash covers the raw downloaded ZIP files for each state, in alphabetical FIPS order. The `redist fetch --verify-downloads` command checks individual file hashes; `redist label-verify` checks the combined build hash."
**Flagged by**: Rodden

## What Changed R1 → R2

- [x] Fix `--verify-sha256`: removed from Section 3; replaced with `redist fetch --year 2020` + note pointing to `redist label-verify` (P1-A)
- [x] Election data provenance: added "Election Data" subsection with Harvard-Fekrazad dataset, fetch command, SHA-256 placeholder, UTF-8/JSON encoding note (P1-B)
- [x] "Byte-identical" qualified: Section 1 now "on the same hardware architecture"; Section 5 analysis hash now specifies "same platform (x86-64 Linux/Windows/macOS with same METIS vendored build), cross-architecture ≤0.001 PP" (P1-C)
- [x] Legally relevant outputs in SHA chain: analysis hash paragraph now enumerates "district assignment CSV, population deviation by district, majority-minority population percentages by district, and a contiguity certificate (one flag per district)" (P2-C, addressed Stephanopoulos P2)
- [ ] Vermont fixture configuration parameters (P2-A) — deferred to journal submission
- [ ] Build hash binary concatenation specification (P2-B) — deferred
- [ ] Reference run protocol / placeholder publication workflow (P1-D) — partial (election SHA note added; full 50-state protocol deferred)
- [ ] Python 3.13 version clarification note (P2-D) — deferred
- [ ] Output path double-year structure (P2-E) — deferred
- [ ] PL 94-171 multi-segment file hashing (P2-F) — deferred

## Notes

The package passes the 3.0/4.0 bar at R1 (avg 3.10). The P1-A CLI error is the most critical: it must be fixed before this package is used for any actual replication attempt. Target R2 average: ≥ 3.4/4.0.

**R2 result**: 3.46/4.0 — passes ≥ 3.3 target. Key improvements: blocking CLI error fixed (P1-A), election data provenance documented with Harvard-Fekrazad dataset and fetch command (P1-B), byte-identical claim qualified in Sections 1 and 5 (P1-C), legally relevant outputs enumerated in analysis hash (P2-C). Remaining P2 items (VT fixture params, build hash binary spec, reference run protocol, Python version note) deferred to journal submission pass.
