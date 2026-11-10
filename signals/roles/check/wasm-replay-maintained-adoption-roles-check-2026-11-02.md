---
skill: roles-check
topic: wasm-replay-maintained-adoption
date: 2026-11-02
roles_used: [benchmark, covenant, ledger, survey, contour, trench]
p1_count: 3
verdict: NEEDS-WORK
---

# Replay integration adoption review

Code, regression fixtures, source inventories and the 81-file integration payload were reviewed using six repository role lenses by one agent. No independent reviewer or delegated review is claimed. Selection: benchmark for causal regression coverage; covenant for source/binary provenance; ledger for formats and compatibility; survey for practitioner workflow; contour for geographic identity; trench for structural failure controls. All installed role files were inventoried. The verdict applies to full release acceptance. Reversible source adoption may proceed with the explicit gaps below; it does not close WP-01–WP-10.

The concrete payload is `target/native-verify-parameters-20261102/maintained-replay-gates-phase/integration-source-package-7g5zpctg/manifest.json`. Both owned package checks passed 13 runtime and six actual Chrome journeys per profile. Package bytes stayed unchanged. The source ledger validates 311 partial records; all 522 compiler inputs match the tested private snapshot. Main before-images and a byte-exact disposable Git application are retained.

| Role | Finding | Severity | Source | Action |
|---|---|---|---|---|
| benchmark | Frozen vectors do not constitute fresh native filesystem replay or original real-state acceptance. | P1 | Runtime corpus scope | Retain original matrix and run amended release gates. |
| benchmark | Core whole replies allow only independently checked State pointer width; C maps compare literal original assignments. | P3 | test_native_recorded_replay.mjs | Preserve exact comparisons and backend counterexamples. |
| benchmark | Runtime and browser gates are now registered in both builders, but the complete builders have not run on the adopted source. | P2 | build_wasm.ps1; build_cffi_preview.ps1 | Run after adoption, retain terminal logs. |
| covenant | Selected bytes and historical producer hash claims do not authenticate upstream sources or a published binary. | P1 | Recipe/source scope and frozen references | Keep provenance limits visible and finish original authority requirements. |
| covenant | Legacy native version claims remain visibly outside the v1 recipe digest. | P2 | native_replay_recipe.rs | Preserve unbound evidence and contradictory-claim refusals. |
| covenant | The sampler's upstream MIT license was missing from the initial integration union. | P2 | native64_rng.rs | Corrected: include exact retained license; 81-file packet re-proved. |
| ledger | V1/V2 digests and typed serialization remain distinct from V3 accepted retry/sampler evidence. | P3 | wasm-native-state-replay-recipe.md | Retain old hash vectors. |
| ledger | A literal assignment export is not proof of fine polygon GeoJSON/map interoperability. | P1 | native_recorded_replay.rs | Complete original fine geometry/map/export workflows. |
| ledger | Historical inventory records were preserved while current identity/test source slots changed. | P2 | inventory preservation audits | Continue checking inverse preservation; do not promote acceptance records. |
| survey | Open and saved reopen are inert; C choice resets and needs explicit selection. | P3 | 12 Chrome journeys | Keep this behavior in maintained main tests. |
| survey | Native64 metadata is readable but does not imply browser executable capacity. | P2 | native64-metadata fixture | Keep execution limits distinct and test the fixed capacity matrix. |
| survey | Seed, constraints and exact recorded settings are visible without rebuilding unsafe integers. | P3 | count-results.js | Preserve source bytes through project roundtrip. |
| contour | Multiscale graph validation now uses the actual fine universe, while fine polygons remain unfinished. | P2 | native_recorded_replay.rs; multiscale evidence | Validate fine map/export separately before support claims. |
| contour | Original graph/source hashes and complete literal GEOID sets are checked before replay. | P3 | native_recipe_sources.rs | Preserve leading zeros and complete-set equality. |
| contour | Small synthetic-source vectors do not establish all vintage, geography or population-source correctness. | P2 | Corpus manifest | Retain all original raw-source expected successes and failures. |
| trench | Four changed/missing/unsafe input controls refuse before execution, followed by a successful run. | P3 | Runtime gate | Keep both refusal and recovery assertions. |
| trench | The corpus loader initially confused original producer and selected bound manifest identities. | P2 | Corrections note | Corrected: preserve both hashes and check the selected MF against its own pin. |
| trench | Six private reference examples and two old CLI scripts are outside the source-adoption union. | P2 | unpackaged-source-differences.json | Preserve as historical evidence, not maintained CI or acceptance claims; actual maintained gate inputs are self-contained. |

Roles reviewed: 6. P1 release blockers: 3; P2 issues: 9; P3 notes: 6. Verdict: NEEDS-WORK for full delivery. Top finding: the maintained selected-byte corpus is useful regression coverage but cannot replace original native/workflow/release acceptance. Benchmark, covenant, ledger and contour agree on that boundary.

Amendments: (1) include the omitted license and re-prove the integration payload (done); (2) adopt with exact main hash guards, retain preimages, build and run checks from main; (3) keep all original fine-map, real-source, lifecycle/capacity and full package/hosting obligations open. No commit, push, merge or deployment is authorized by this review.

Post-adoption amendments: main project security/handler test initially failed because its VM import scanner required whitespace after `import`; the new valid compact imports were left in script text. Both import extraction/removal now accept compact named imports and the whole handler suite passes. The binary-target Verify filter selected zero tests (retained, not counted); the actual library suite passed 58 tests with one environment-dependent ignored test. Review of the real copy_toolkit_assets.ps1 layout also found that the browser gate must select packaged index.html rather than source workbench.html. The gate now distinguishes explicit package roots from the default source tree; real-layout package evidence is required below. These changes affect test harnesses and current candidate pins, not historical acceptance records or compiled Rust inputs.

Terminal main integration results: the fresh main native host, core WASM and
C-capable WASM builds pass. Both directories were staged by the real toolkit
asset copier (index.html only); C assets passed the existing source-bound stager.
Each package passed 13 runtime and six actual Chrome journeys; package hashes
were unchanged. Main Verify library tests pass 58/one ignored, recipe tests ten,
shared edge weights 35, and actual native CLI Verify integration three cases.
Main project security/handler, strict command comparator and project limit
checks pass. The separate scanner metadata-loss defect PP-120 is closed by its
adopted structural fix and actual main scanner tests. None of the three P1 full
release blockers is closed by these integration results; full builders and
original workflow/real-state/capacity/hosting acceptance still must run.
