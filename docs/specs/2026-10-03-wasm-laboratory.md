# BISECT browser engine



Objective: deliver the complete BISECT engine option capability through WASM,

with the state and national laboratory running on visitors' devices. A narrow

partition demo or a catalog-only viewer does not satisfy this objective.



The user's clarified scope also includes the complete practitioner suite:

certification/audit tools, election workflows, and TUI functionality with

appropriate browser UI. CLI/TUI commands are the inventory baseline. Keep a

command-by-command coverage register; each entry needs an implemented in-memory

operation, a browser workflow and behavior/provenance tests before completion.

Do not declare a command covered just because its crate compiles.



## Practitioner workspaces



- Plan library: labels, configuration wizard, import, migration, rename, sessions,

  doctor checks, lineage, replay verification and cross-year builds. Adapt the

  TUI home/run/compare/verify/doctor screens into browser panels with keyboard

  navigation, accessible forms, errors, progress and persisted local sessions.

- Analysis: demographics, elections, compactness, connectivity, splits, nested

  multi-chamber suites, comparisons, national aggregation and commission reports.

- Certification: RPLAN validation, profile-driven audits, certificate generation

  and independent certificate verification, ILP model/solve evidence, exact proof

  artifacts, and certified-bisection ceremony start/announce/halt/verify flows.

  Preserve complete/partial/not-evaluated statuses, source hashes, constraints,

  solver bounds, transcript custody and algorithm lineage. A browser audit must

  not invent a native executable identity or claim an external authority signed it.

- Election practitioners: RCOUNT import, package verification, audit-algorithm

  replay, district aggregation/crosswalks and RHIST evidence/projections; retain

  the existing statistical and provenance contracts.

- Research/community: sweeps, ensembles, Pareto/exact/improve workflows,

  convergence diagnostics, percentile comparisons, civic-input ingestion,

  conflicts and annotations.

- Data delivery: adapt fetch/cache-index/source handoff and registry operations

  to downloadable prepared packages, local file selection, IndexedDB/OPFS and

  exportable archives. Preserve source validation and integrity checks.



The terminal renderer and operating-system process launcher are replaced by

browser UI and Workers. Their functionality remains in scope. Operations that

depend on external solvers or unavailable source packages require a real browser

replacement; a disabled button or server-only fallback is not completion.



## Architecture



Use the pinned Rust `metis-core` backend through `bisect-runner`. Expose an

in-memory graph/configuration/result boundary. Prepare Census graphs outside

the browser and publish versioned, hashed state packages alongside shared display

geometry. Preserve GEOID order, populations, physical boundaries, counties,

areas, perimeters, centroids and optional demographic/election input signals.

Do not derive engine boundaries from simplified display geometry.



Execute the WASM module in a dedicated Worker. Start with serial iteration

inside that Worker so hosting does not depend on shared-memory headers.

National mode queues states, releases each working graph after use and exposes

progress, cancellation, failures, comparisons and downloadable records/maps.

Cancellation terminates the Worker rather than pretending a synchronous engine

can receive messages while it is blocked. Seeded native Rust and WASM runs

must be compared; exact C METIS output equivalence is not assumed.



## Scope and gates



1. Compile and link the Rust runner and browser boundary to a real WASM module.

   Resolve entropy, parallel iteration, timing and host I/O explicitly.

2. Exercise all current laboratory structures, weights and supported searches

   (66 combinations), including AreaSection's dual constraints. Preserve active

   budgets and distinguish requested, effective and achieved settings.

3. Inventory and deliver the remaining CLI engine structures/searches/weights

   and advanced options, including demographic and election signals, multiscale

   execution and exact-solver modes. Each option needs a browser implementation

   and validation; rejecting it does not count as completing its port.

4. Publish compact prepared graph packages for all 50 states and the supported

   Census years/chambers; include explicit availability and district allocations.

5. Integrate Worker execution into the laboratory without losing saved-result

   browsing, state/national maps, metrics, comparison or exports.

6. Verify real browser runs across small, medium and large states, population,

   connectivity, configuration fidelity, cancellation, failed-state coverage,

   bounded memory and reload recovery. Compare against native Rust execution.

7. Produce a static publication artifact and documented reproducible build;

   preserve the existing Pages site and check the combined hosting size.



## Current evidence



The isolated `metis-core` compile probe passed on `wasm32-unknown-unknown`.

The complete runner compile initially failed because workspace Rand enables

OS entropy. Its Rust bisection adapter also disables contiguity by default and

rejects ncon=2, despite the pinned partitioner's contiguity and multi-constraint

APIs. These are integration issues to resolve and test, not proof of parity.

No full engine or browser-run completion is established by the compile probe.



## Progress evidence — 2026-10-03



The new `bisect-wasm` crate builds a linked module and provides graph execution

and practitioner operation dispatch. The release module is about 2.4 MB. Actual

WASM tests exercised 16 engine cases and RPLAN validation, native certificate

verification, browser-provenance certificate generation and tamper rejection.

The public fixture certificate was generated and verified in a browser Worker.

The initial workbench has four operation interfaces; full-suite UI remains open.



The baseline inventory contains 69 command/screen entries, with implementation,

UI and verification tracked separately. The engine-wide options gate and all

remaining suite gates above remain active. Known open issues include the Rust

AreaSection adapter's ncon=2 rejection, remaining structure dispatch, BFS balance

failure on the smoke fixture, fixed-width seed derivations, architecture-specific

SmallRng behavior, and actual-state parity/performance testing. No scope reduction

or complete-suite claim follows from this first workbench.





Live laboratory integration now has a static prepared-graph exporter and a Worker

adapter behind the existing state/national map, metrics, comparisons and exports.

RI/IA/NC 2020 runs are verified in WASM and actual browser Workers. Input assets

and the module are hash-checked, state graph caches are released, and cancellation

is tested before startup, during initialization and during execution. Shared

saved-result geometry is preserved and reused. The all-state 2020 export is

complete: 84,208 tracts, 50 graph packages, approximately 172 MB with shared maps.

Input validation verifies hashes, connectivity, population totals and map joins.

Full national browser evidence is not yet established. Engine project

persistence remains pending. Direct Rust k-way n-way is now wired accurately;

NC exposes a substantial population imbalance and is recorded as a quality gap.

Do not count that case as a valid plan or close the engine options/parity gate.



The first allocated national standard-bisection test completed 39 states with

valid plans and failed in 11 states. A repository-local patch of pinned

`metis-core` now stops slow coarsening and preserves source contiguity during

label-propagation pre-balance. All 182 partitioner unit tests pass, including

weighted-star coarsening and articulation-move regressions. Actual release WASM

now completes all 50 runs with contiguous plans, but only 44 meet the requested

5% population tolerance. CA, FL, NY, OH, TX and WA remain balance failures.

The national test writes diagnostics and exits unsuccessfully unless all 50

meet both constraints. This gate remains open; completed runs do not establish

valid plans or best-cut guarantees. Baseline and current evidence are retained

under `target/wasm-national-allocated-before-partitioner-fix.json` and

`target/wasm-national-allocated-validation.json`.



Subsequent fixes bound parameterized SHEM/HEM aggregate weights and make initial

growth population-aware, including asymmetric targets before refinement. Current

release evidence improves the national result to 48 valid states out of 50; all

50 remain contiguous, while CA (73.46% maximum deviation) and TX (33.10%) fail

the 5% population criterion. The national command therefore exits with status 1.

All 184 partitioner tests and 264 pure-Rust runner tests pass (14 runner tests

ignored). Release WASM smoke and RI/IA/NC adapter integration tests also pass;

NC direct n-way still misses balance at 25.64%. These corrections change seeded

assignments and do not establish native/WASM equivalence. Current diagnostics

replace neither the parity gate nor the remaining engine and practitioner scope.



Fixed-width `u32` sampling in the partitioner's HEM shuffle and initial seed

selection resolves a native64/WASM32 RNG stream mismatch. A release native

split-trace example and runtime parity test now compare every CA/TX assignment

and population/contiguity metric against actual WASM. They match exactly for

standard geographic bisection, seed 42, 10 iterations and 5% tolerance. Hashes

and failing split paths are recorded in `target/native-wasm-split-parity.json`.

Both balance misses occur in late three-district nodes, CA `1100` and TX `1111`.

This is evidence for the specified Rust configuration; the full structures,

searches, signal inputs, Census years and practitioner inventory remain required.



Standard bisection Multi is implemented in the shared runner, CLI and WASM

boundary and advertised by the static exporter. It samples the requested

consecutive seeds at each cut, selects minimum weighted cost among cuts meeting

both child balance targets and connectivity, and errors if none qualify. It

does not evaluate child plans to choose a parent. Native CLI dispatch previously

ignored Multi for standard bisection. Percentile search now uses fixed-width

seed-index hashing and propagates failed seed executions rather than inventing

a one-district candidate. RI/IA native/WASM parity passes for both searches,

including the complete assignments and population metrics. Actual browser Multi

execution for RI renders a connected map at 2.300% maximum deviation. Evidence

includes `target/native-wasm-search-parity.json` and `target/wasm-multi-browser.png`.

Current tests: 267 pure-Rust runner tests (14 ignored), 13 relevant C METIS tests

(one ignored), 17 release WASM smoke cases, laboratory integration and CLI check.

All-state Multi validation and full option/practitioner inventories remain open.





Portable laboratory projects now store one browser experiment's configuration,

completed tract assignments, metrics and hashed input identities. The UI supports

Save project, Open project and latest-experiment IndexedDB autosave/recovery.

Imports are bounded and parsed in a cancellable Worker, and commit only after

independent population, connectivity, boundary-cost and county-split checks

against hash-verified prepared graphs. Imported engine identities remain

unverified claims. Interrupted snapshots retain completed states and do not

automatically resume execution. Actual browser evidence reopened RI/IA county

Multi results, recovered them after reload and saved a file whose 1,146

assignments/settings were independently rechecked. Tests also exercise tampered

evidence, input drift, cancellation/edit races and unfinished national batches.

The exporter includes all project modules and a same-origin CSP. Shared input

packages must still be available; offline archives, full history/session libraries

and the remaining practitioner/engine options are not complete.



The 2020 congressional geographic Multi matrix now passes 50/50 states at

seed 42, eight seeds per node, 10 iterations and 5% tolerance. Evidence is

`target/wasm-national-multi-validation.json`, hash

`15e673157b26bbe14c33dc7e42f5ccd7a0c1e119681f846cfafa90c8b263d446`.

Single-seed CA/TX balance failures remain separate evidence. Full national

browser Workers/maps, other years/chambers and the full option inventory remain

required; no best-cut guarantee follows from this passing sampled configuration.





AreaSection is now dispatched through Rust/WASM with two constraints and

separate tolerances. Joint population/area initialization and multiconstraint

LP/FM replace the former rejection/scalar-only path. Root candidates are

independently filtered for physical population, physical area and connectivity;

no feasible sample errors. The two-district shortcut no longer omits area,

Lorenz bounds match both half-area upper limits, and configured seed walks now

reach GeoSection/AreaSection roots and descendants in CLI and WASM. The UI and

portable projects expose/check root-only constraint evidence. The existing design

still uses population-only descendants, which is made explicit in the UI.



`target/native-wasm-area-parity.json` proves exact assignments/district metrics

for controlled tight/loose-area fixtures and RI/IA 2020 congressional Multi

(seed42,32seeds,20iterations,5%final tolerance,swing1.1). Both state plans meet

connectivity and population checks, with root area fractions 46.284% and 53.004%.

An actual browser RI run and independently checked saved project also pass.

Tests: 187 partitioner, 270 pure-Rust runner (13 ignored), two C METIS area

regressions, 18 actual WASM smoke cases, project/root-tamper tests and CLI/exporter

compile checks. Release module SHA-256:

`147bb23cb75b2a2211649a8f3f90e76e796bb83a718a6e5603d46206cbefd72c`.

The all-state/year/chamber/option matrix and practitioner inventory remain open.

Non-equal secondary-target Rust constraints for other algorithms are still

explicitly unsupported. No global best-cut or full-suite completion claim follows.





ApportionRegions WASM port (2026-10-03): the browser now calls the existing

`bisect-apportion::PfrCompositor` with the pure Rust METIS backend. It preserves

largest-prime-first composition, floor/ceil fallback for prime counts above 3,

per-level tolerance and the native 3% research rejection limit. The UI enables

Single search for this structure and displays factors, depth, tolerance, seed

and the research limit. Other search compositors are not silently substituted.

The requested global tolerance and connectivity are independently reported;

a completed run can still contain disconnected districts.



The module is 2,939,494 bytes, SHA-256

`60094e9304da29353b885a638843875457e4c6e68080eb8afa781f688d8076c0`.

`target/native-wasm-prime-parity.json` records 21 native/WASM comparisons:

12 completed cases with identical assignments, district measurements and

structure evidence, plus nine identical research-limit rejections. Coverage

includes prime/composite and large-prime fallback fixtures and RI/IA 2020

congressional districts with geographic, uniform and county weights. Iowa's

geographic/county results have disconnected districts in both runtimes;

those outcomes remain visible, not treated as valid certification.



The real Rhode Island browser run completed with 250 assignments, two connected

districts, 2.361353734671434% maximum deviation, 50.5 km boundary and one split

county. Its downloaded `target/wasm-prime-browser-project.bisect` was reopened

without an engine worker and independently verified. Saved prime-factor

evidence is checked against options and the integer boundary cut; tampering

is rejected. The screenshot is `target/wasm-prime-browser.png`.



Validation: 105 pure Rust apportion tests, 19 WASM smoke cases, AreaSection

regressions, native/WASM prime parity, laboratory project tests, practitioner

import security and package tests passed. The laboratory exporter compiled.

This addition does not complete the full engine-option and practitioner matrix.





Spectral WASM port (2026-10-03): the native recursive spectral helper now lives

in `bisect-apportion::spectral::run_spectral_recursive`. Both CLI and WASM call

that implementation. The browser exposes spectral smoothing iterations through

the step budget (labelled explicitly), preserves all three boundary-cost modes,

and disables seed and METIS refinement controls because they do not affect the

spectral partition. Boundary weights affect reported costs; the spectral sweep

itself uses unweighted adjacency. Recorded effective settings identify the

actual smoothing budget and omit seed/refinement effects.



The current module is 2,997,587 bytes, SHA-256

`fd835ba9680045c0d55bb6bddb359300c73bb538268074e5bf8e51638553e991`.

`target/native-wasm-spectral-parity.json` records 18 native/WASM comparisons:

RI/IA 2020 congressional districts, iteration limits 1/20/200 and all three

boundary-cost modes. Assignments, district measurements and full per-node

summaries match. Changing seed or METIS refinement iterations leaves spectral

results unchanged. Infeasible population-balanced sweeps return explicit errors.

These RI/IA cases have disconnected districts in both runtimes; smoothing

convergence is not an optimality or contiguity certificate.



Portable projects independently recompute each recursive node's cut and

population deviation, verify its target fraction and configuration, and reject

missing or altered evidence. Reopening does not run the engine. A real Rhode

Island browser run with 20 smoothing iterations produced 250 assignments,

0.6957486884658914% maximum deviation and 61 unweighted cut edges. One district

has two components. Autosave recovery preserved that warning after reload;

recorded engine provenance remains an unverified imported claim.



Validation: 107 pure Rust apportion tests, 20 WASM smoke cases, 18 spectral

native/WASM cases, secure Save/Open in every boundary-cost mode, existing

laboratory/practitioner import tests and native CLI/exporter compilation passed.

`target/wasm-spectral-browser.png` shows the browser result. The complete WASM

engine-option and practitioner capability matrix is still unfinished.





Flow construction WASM progress (2026-10-03): the browser now calls the same

`bisect-flow::construct_flow` function used by the native CLI, with farthest

graph seeds, edge-cut cost and BFS repair defaults. Single execution is exposed;

random seed, seed budget and METIS iterations do not alter construction.

Physical/county weights change reported costs, not the unweighted construction.

Successful results include the native flow summary. Open Project independently

checks its measured deviation, edge cut, valid status, seed dimensions and

connectivity; imported execution provenance remains unverified.



Verification: four shared flow unit tests passed. The flow WASM integration test

matched nine feasible path-fixture results and nine native failures across all

three boundary modes. It also checked parameter independence, summary tamper

rejection, hashed-input project roundtrip and map joins, and confirmed that Open

starts no engine. The existing 21-case WASM smoke and laboratory project suites

passed; the exporter compiled. The actual browser RI 2020 run reported the same

native failure: `weights must be positive`. RI/IA prepared graphs contain

zero-population tracts; enabling those inputs in the shared algorithm remains

unfinished. No population values were altered to force success.



Current module: 3,090,442 bytes, SHA-256

`0f2247c227ddf04cfb6f4cf777ff80a429137b49be857976f3567d379140f5a1`.

Evidence: `target/native-wasm-flow-parity.json` and

`target/wasm-flow-browser.png`. This advances one structure; full engine-option,

practitioner, state/year/chamber and national-browser coverage remain open.





Flow zero-population follow-up (2026-10-03): the shared native/WASM algorithm

now accepts nonnegative tract populations with positive, checked i64 totals.

Negative/all-zero/overflowing input and nonfinite/out-of-range tolerances are

rejected. Small connected repair now distinguishes an unused district label

from a populated-by-tracts district whose population is zero. Tests compare

repair feasibility against every contiguous split for 728 six-vertex path

population patterns including zero weights.



Seven shared flow tests passed. Current native/WASM parity covers 15 successful

cases (including unmodified RI 2020 at 5% tolerance and a zero-population path)

and six matching failures across the three boundary modes. Iowa returns

NeedsRepair; it is not claimed as a successful plan. RI secure Save/Open and

map joins passed in all three modes, without starting an engine on Open.

The 21-case WASM smoke suite passed. A real browser RI run completed with

250 assigned tracts, two connected districts, 2.030% maximum deviation,

94.8 km physical boundary and one split county. The default 0.5% tolerance

returned NeedsRepair; the successful run explicitly used 5%.

The downloaded browser project was independently reopened and its current

module identity checked without engine execution.



Current module: 3,091,216 bytes, SHA-256

`5a8d54a484c4278b2c4d5fe3de62e8b1f22b4e7c051dc8358177a1282382cb13`.

Evidence: `target/native-wasm-flow-parity.json`,

`target/wasm-flow-zero-browser-project.bisect`, and

`target/wasm-flow-zero-browser.png`. Full capability coverage remains active.





Capacity clustering / regionalization WASM progress (2026-10-03): both browser

structures call the existing native `bisect-clustering` constructors with

native defaults. They are deterministic single executions; random seed and

METIS iterations are disabled, and physical/county boundary modes affect

reported costs rather than construction. Shared input validation now permits

zero-population tracts, with checked positive total population, finite tolerance

in [0,1), and bounded adjacency endpoints. No tract populations are replaced.



Capacity summaries are retained. Regionalization retains its native summary

and complete merge log in projects and run records; the UI shows a bounded

50-merge preview. Open independently checks population and edge-cut metrics,

every merge's active region IDs, population sum and adjacency boundary count,

hierarchy depth, and unrepaired final assignment labels. Small exhaustive repair

can change the final assignments; imported provenance remains unverified.



Verification: 15 shared clustering integration tests passed, including zero

population and invalid input checks. Native/WASM comparisons covered 36 cases:

each structure had nine feasible fixture successes and nine matching failures

across all boundary modes, including unmodified RI/IA state graphs at 5%

tolerance. Portable fixture projects and map joins passed without engine

execution on Open; modified metrics, merges and hierarchy depth were rejected.

The actual UI renderer passed display-budget and text-only cell checks. The

21-case WASM smoke and existing laboratory project suites passed; native CLI

and web exporter compilation passed. Actual browser RI regionalization reported

NeedsRepair, matching native behavior. These methods remain unable to produce

valid plans for the tested RI/IA configurations; no success is claimed there.



Current module: 3,226,335 bytes, SHA-256

`1b3542945c0a7603b569f680776db678fb8d1483015c0bbfd68f18aa627d2194`.

Evidence: `target/native-wasm-clustering-parity.json` and

`target/wasm-regionalization-browser.png`. The complete WASM capability goal

remains active; other structures/search options and practitioner workflows,

full state/year/chamber coverage, and national browser verification are open.





Simulated annealing WASM progress (2026-10-03): the browser exposes the native

steps-per-tract, initial-temperature factor and final-temperature parameters.

They are recorded in requests/projects, validated, and restored with saved

results. Only the executed single-run search is enabled. Native initialization

uses 100 refinement iterations; the general refinement field is disabled.

Boundary weights affect initialization; annealing's objective is unweighted

edge cut. Cooling uses T0=max(1,factor*initial-cut), with final temperature

clamped per split to [1e-12,T0]. Zero steps is a valid baseline.



Shared correctness fixes: recursive annealing now receives a relative balance

tolerance rather than a multiplier and uses floor/ceil seat-ratio population

targets for odd district counts. Per-level tolerances bound accumulated error

on the deepest path. Balance checks use actual populations (including zero),

check both targets, and require a valid connected initial partition. Flips

cannot empty a district. Step multiplication and initial-temperature overflow

are rejected. Platform-dependent SmallRng was replaced for this method by

explicit ChaCha12 with u64 range sampling (`chacha12-u64-v1`). This changes the

native annealing seed sequence; prior results from the old implementation are

not reproduction references for this implementation. Other random search

methods still require a broader platform-RNG audit.



Verification: full pure-Rust runner suite passed (276 tests, 13 ignored).

65 native/WASM annealing cases passed: 56 exact successful results and nine

matching rejections. Coverage includes feasible/zero-population paths, odd

and composite district counts, RI/IA 2020 at 5%, steps-per-tract 0/2/10,

all three boundary modes, changed cooling parameters and seeds, and ignored

non-annealing budgets/refinement. RI portable projects and map joins passed

in all boundary modes; tampered parameters/feasibility were rejected and Open

started no engine. Existing 21-case WASM smoke, laboratory project and project

security suites passed. CLI/web/WASM examples compiled.



Actual browser RI geographic annealing completed with 250 tracts, two connected

districts, 4.935% maximum deviation at requested 5%, 69.7 km boundary and two

split counties. Save and Restore recovered all annealing controls; the actual

downloaded project was independently reopened without engine execution and its

module identity verified. No optimality or improvement over other methods is

claimed by this successful run.



Current module: 3,276,872 bytes, SHA-256

`d2b121eff1a0cadd85152dcb2c9a12b6f8c842de1c8c628a93ee93fb9871e61c`.

Evidence: `target/native-wasm-annealing-parity.json`,

`target/wasm-annealing-browser-project.bisect`, and

`target/wasm-annealing-browser.png`. The full WASM capability goal remains active.



### Verified national browser gate (2026-10-03)



The current d2b121ef module completed an actual 50-state browser Worker run

using 2020 congressional allocations, standard bisection, geographic weights,

Multi eight-seed search, seed 42, 10 refinement iterations and 5% tolerance.

The browser download contains 435 districts and 84,208 tracts. Independent

hash-checked assignment and geometry verification passed for all 50 states;

every district is connected and maximum state deviation is 4.217946443256704%.

The combined map rendered, and a fresh browser tab opened the complete project

through bounded evidence workers without running the partition engine.



Reproduce saved-evidence verification with

`node scripts/web/verify_national_project.mjs`.

Artifacts: `target/wasm-national-browser-project.bisect`,

`target/wasm-national-browser-verification.json`, and

`target/wasm-national-reopened-browser.png`.

This completes this national execution/persistence gate. Full engine options,

practitioner coverage, portable input archives and broader parity gates remain

required by the active goal; this result does not establish optimality.

### Flow repair options (2026-10-03)



The browser now exposes native flow repair choices bfs/none. Typed requests,

effective settings, project persistence and imported-summary checks retain the

selection; legacy projects preserve the BFS default. Both modes passed 42

native/WASM cases (27 exact successes, 15 matching failures), including a

six-district fixture where BFS repair succeeds and no repair fails. Both modes

passed fixture project/map roundtrips; actual browser RI no-repair Save/Restore

and independent downloaded-project verification passed. RI: 250 tracts, two

connected districts, maximum deviation 2.030383304218508% at requested 5%.



Module b12ea340d5a8f37815ff3a507b42259e503e26a345e2a5dc1f7b81d47025a16f,

3,277,096 bytes. Isolated preview: /wasm-flow-options/. Evidence:

`target/native-wasm-flow-parity.json`,

`target/wasm-flow-options-browser-project.bisect`,

`target/wasm-flow-options-browser-verification.json`,

`target/wasm-flow-options-browser.png`.

The complete engine/practitioner option goal remains active.

### SMC percentile browser port (2026-10-03)



Standard-bisect now exposes the native SMC percentile compositor with particle,

weighted percentile, ESS threshold and seed controls. Requests/projects retain

active parameters; diagnostics include ranked weights, selected particle, ESS

and resampling rounds. Bounded UI tables display diagnostics. Open verifies

ledger consistency and independently checks assignments/maps against hashes.



SMC uses ChaCha12/u64 sampling on both platforms. Native seed sequences change,

and NDJSON metadata identifies protocol 2.0-chacha12-u64-v1. Invalid input bounds

are rejected; percentile-1 fallback cannot select a zero-weight particle. The

native fixed 0.005 proposal tolerance uses remaining-component population;

final balance checks remain independent and can fail. No calibration or global

optimality claim follows from this finite sample or its diagnostics.



Final-module gate: 58 cases, 49 exact assignment successes with numerical

metric/weight comparisons, nine matching rejections; 16 successful samples

miss final balance and are recorded as misses. Native runner: 278 passed,

13 ignored. Ensemble/SMC suites, 40 SMC unit tests, compilation, smoke,

project/import-security and bounded diagnostics tests passed. Actual browser RI

16-particle percentile-zero run has two connected districts, 250 tracts and

0.49144370358827905% deviation. Save/Restore, reload and independent downloaded

project verification passed without engine execution on Open.



Module edca212714748260f2389838da3c6b91498b860025e76a4fa18c761f89c786d5,

3,350,671 bytes. Preview /wasm-smc/. Evidence:

`target/native-wasm-smc-parity.json`, `target/wasm-smc-browser-project.bisect`,

`target/wasm-smc-browser-verification.json`, `target/wasm-smc-browser.png`.

Full engine options, ensemble/practitioner commands and inventory completion

remain required by the active goal.



Election import gate (2026-10-03): statement CSV and the existing native NIST

CDF subset now run in WASM through shared byte adapters and package writers.

The workbench includes source selection, explicit metadata with persisted

drafts, readable verification checks, package archive download/reopen and

project Save/Open. Exact source bytes and count-bearing normalized files stay

encoded; counts above 2^53 are never parsed as JavaScript numeric inputs.

The native CLI's synthetic fixture manifest defaults are replaced by explicit

user metadata. Source/package bounds remain 8 MiB; project bounds remain 25 MiB.



The 51 native I/O tests passed, including disk/memory exact package bytes.

Four native/WASM exact-result cases cover both adapters with ordinary and

9,007,199,254,740,993 counts; five malformed requests reject on both runtimes.

Source tampering fails verification, and source/project/archive roundtrips

preserve counts. Actual browser imports, downloaded files, reload/Open and

explicit verification passed for CSV and NIST. Downloaded artifacts match

fresh WASM execution. Package checks do not certify election outcomes or an

external authority. The two corresponding inventory entries now record these

bounded native-adapter workflows; the full goal remains incomplete.



Preview `/wasm-election-imports/`, module 3,611,545 bytes,

SHA-256 `64562919c1fe880991599179e5e69589999c31b16d39bea69a3a4123539fadde`.

Evidence: `scripts/web/test_election_imports.mjs`,

`scripts/web/verify_election_project.mjs`,

`target/wasm-election-import-browser-verification.json`,

`target/wasm-nist-import-browser-verification.json`, and

`target/wasm-election-import-browser.png`.



RI RLA gate (2026-10-03): the RI 2024 Rep 28 three-source adapter now accepts

bytes in memory, preserves original report/manifest/retrieval sources, and

uses a shared package writer. Native vote and manifest arithmetic now reject

negative counts and overflow. The workbench adds its ninth operation, with

three file selections, persisted partial inputs, restored filenames, package

archive export/reopen and explicit native replay boundaries. Input panels

follow the selected import operation.



All 51 native I/O tests passed; the native 14-batch fixture has exact

disk/in-memory writer parity. Two native/WASM fixtures match exactly, including

counts above 2^53 and a public seed above u64. Six malformed-source cases

reject on both runtimes; tampering each original source fails verification.

Existing engine, election, project/package security and workbench UI gates

passed. Actual browser import, downloads, reload/Open and archive replay

passed. Downloaded artifacts match fresh WASM execution exactly.



This ports the native RI-specific adapter's capability. Risk calculations

remain recorded without reconstructed sample steps, and human observations

are not independently verified. Replay returns the native `boundary` status;

passing package equations/source hashes do not certify an RLA outcome. Full

practitioner/engine coverage remains incomplete.



Preview `/wasm-ri-rla/`, module 3,738,291 bytes, SHA-256

`0ac5a299a97d5dfe0be5764afe38ef63a3255390f5585cfd45f9fb97462a8bf7`.

Evidence: `scripts/web/test_ri_rla_import.mjs`,

`target/wasm-ri-rla-browser-verification.json`,

`target/wasm-ri-rla-browser-project.bisect`,

`target/wasm-ri-rla-browser-archive.json`, and

`target/wasm-ri-rla-browser.png`.



### District aggregation port evidence



`aggregate-districts` uses the shared native in-memory adapter with direct and

explicit integral crosswalk paths. Its UI supplies package, plan, optional

context/crosswalk, contest/status and compact/pretty output. Display counts are

decimal strings; native transcript exports retain exact integer JSON. Projects

preserve exact inputs/settings and result bytes; Open leaves results unverified.



Ten native tests, eight exact native/WASM results and nine matching rejected

requests passed. Native overflow checks now cover direct aggregation as well

as weighted allocation. Actual browser download/reload/Open and explicit rerun

retained `9007199254740995`; fresh execution matches the saved result and

downloaded transcript bytes. Native crosswalk source references remain rejected

when this adapter supplies no source index. Failed original-source hashes remain

failed; aggregation does not authenticate provenance or certify elections.



Preview `/wasm-district-aggregation/`, module 4,050,090 bytes, SHA-256

`1621257cdf3203f03acf70f64c6948e40789c8be06651d9490485f057af50beb`.

Evidence: `scripts/web/test_district_aggregation.mjs`,

`scripts/web/verify_aggregation_project.mjs`, native/build logs under

`target/district-aggregation-*.log`, and the browser project, transcript,

verification report and screenshot under `target/wasm-district-aggregation-*`.

The full WASM capability objective remains incomplete.



### RHIST history verification in WASM



The practitioner workbench now offers **Verify RHIST unit history package**.

Select a complete RHIST folder and Run. The in-memory reader preserves original

source bytes and shares source hash/path checks, package hash verification and

core cycle/context, lineage cardinality, crosswalk rational weight and claim

boundary checks with the native disk reader. The CLI and WASM now share the

verification transcript implementation. Source paths reject traversal, Windows

separators, drive/stream syntax and empty/dot components. No package is extracted

to disk by the browser. Native zero-hash sentinel behavior is preserved; neither

hash matching nor the package's own claim boundary authenticates provenance.



The UI renders up to 500 history check rows as inert text. Folder inputs persist

as exact bytes in portable projects, and the native transcript can be downloaded

as result JSON. Open restores the package with an unverified result and executes

no engine; Run performs fresh verification. Existing portable limits apply:

2,000 package files, 8 MiB package bytes and 25 MiB project JSON.



All 24 native history tests passed. Six fixtures match disk CLI, native memory

and WASM transcripts exactly, covering rename, split/merge, three cycles, the

RI tract history fixture, missing units and invalid crosswalk weights. Seven

additional failed memory packages match native/WASM output: changed/missing

sources, hash drift, malformed JSON, invalid UTF-8, missing manifest and an

unsafe source path. Existing engine smoke, count/import/aggregation parity,

package/project security and operation-panel gates passed.



Actual browser verification selected the 14-file three-cycle fixture, displayed

ten passing checks, downloaded project/transcript, reloaded and reopened the

saved project, then explicitly reran. Downloaded files match fresh WASM output.

The preview packaging step encountered Windows PowerShell 5.1's missing

Path.GetRelativePath method after all gates passed; the builder now validates

and strips the resolved fixture-root prefix, and preview assembly succeeded

with that fix. This did not require repeating completed compilation/parity gates.



Preview: `/wasm-history/`. Module: 4,272,513 bytes, SHA-256

`f1458fe27308fa20c89aaac7f75fafdcb1a433bd055b27a2dfcb223c58959b9a`.

Evidence: `scripts/web/test_history_wasm.mjs`,

`scripts/web/verify_history_project.mjs`, `target/history-preview-build.log`,

`target/wasm-history-browser-project.bisect`,

`target/wasm-history-browser-transcript.json`,

`target/wasm-history-browser-verification.json`, and

`target/wasm-history-browser.png`. Full engine/practitioner coverage remains

incomplete; this operation verifies consistency, not historical truth.



### Centroidal Voronoi controls and native/WASM portability



CVD now exposes Voronoi iterations and graph-distance/geographic distance in

both the engine request and laboratory form. New form configurations use native

CLI defaults (20 iterations, graph distance). Legacy requests without these

fields preserve the prior browser selection (50 iterations, geographic).

Zero iterations retains native semantics of at least one iteration. Explicit

budgets are bounded to 0–10,000; parameters on other structures are rejected.

Requested settings, effective budget and actual distance/RNG evidence persist

through Save/Open; generic METIS refinement iterations are disabled for CVD.



The shared native graph-distance medoid sampler now uses ChaCha12 with fixed

u64 Fisher–Yates draws. Both metrics reduce the full u64 derived seed modulo

vertex count before converting to a platform index. This removes 32/64-bit

seed truncation and sampling differences. Graph-distance seeded outputs can

change from the previous SmallRng implementation; old saved assignments are

still checked against their recorded inputs rather than silently regenerated.

New results identify `chacha12-u64-v1` and `full-u64-modulo-v1`.



Fourteen native CVD-related tests passed (two data-dependent tests ignored).

Eighteen synthetic requests match native/WASM assignments and all metrics

exactly across both metrics, iteration budgets 0/1/20, high seeds and medoid

samples beyond 50 units. Five invalid parameter requests are rejected by both.

Evidence field tampering is rejected; old parameter-free configurations remain

readable. Parity does not establish quality: this matrix contains 15 balance

misses and nine disconnected plans, retained honestly in the results.



The actual RI browser run used graph distance, 20 iterations and seed 42. Its

250 tracts form two connected districts with 0.643077733399311% deviation.

The requested tolerance was 0.5%, so the saved result correctly fails balance.

Browser download/reload/Open restored both controls and the map without running

the engine; independent project verification agreed. Native CVD balance and

connectivity weaknesses remain visible, not disguised as successful plans.



Preview `/wasm-cvd-options/`, module 4,276,965 bytes, SHA-256

`d32a6ca193cb36c4596f3634474f88d8ef74e27d28190e512554580d98f56f44`.

Evidence: `scripts/web/test_cvd_wasm.mjs`, `target/cvd-native-wasm-parity.json`,

`target/cvd-preview-build.log`, `target/wasm-cvd-browser-project.bisect`,

`target/wasm-cvd-browser-verification.json`, and `target/wasm-cvd-browser.png`.

The full engine/practitioner port remains incomplete.


### Moving-knife orientation and scoring controls

Moving-knife engine requests and laboratory forms now expose orientation count
and requested scoring metric. New forms use the native CLI default of 180
orientations and Reock; old requests/projects without these fields retain the
browser's prior 36-orientation Reock configuration. Explicit counts are bounded
to 0–10,000, with native zero-to-one orientation semantics preserved. Parameters
on unrelated structures are rejected. Generic METIS refinement controls are
disabled for this structure; explicit effective budgets and requested options
persist through Save/Open.

The native Polsby option currently falls back to Reock. The option label states
that it currently scores Reock, and structure evidence separately records the
requested metric, effective Reock metric and fallback boundary. This port does
not claim to implement Polsby perimeter scoring. Tampered requested/effective
metric, orientation budget or boundary evidence is rejected on project checks.

Fifteen native moving-knife-related tests passed (one ignored). Twenty-four
requests match native/WASM assignments and metrics exactly across Reock/Polsby,
0/1/36/180 orientations and high seeds; five invalid settings are rejected by
both. Requested Polsby assignments match Reock's native fallback. Legacy defaults
are preserved. Synthetic outcomes include zero balance misses and 24 disconnected
plans; parity does not establish feasibility or optimality. Native connectivity
limitations remain visible rather than being repaired by an unrelated algorithm.

Actual browser RI verification used 180 orientations and requested Polsby.
The 250 tracts produced two connected districts at 0.5976968759198065% deviation,
which misses the requested 0.5% tolerance. The project correctly records failed
balance, requested scoring and effective Reock evidence. Browser Save/download,
reload/Open restored both controls and map. Independent project verification
checks metrics against hashed inputs and executes no engine on Open.

Preview `/wasm-mka-options/`, module 4,280,331 bytes, SHA-256
`995662e89b753c8a36a47399159448631e0df7ca39c21e8209b1740652914131`.
Evidence: `scripts/web/test_mka_wasm.mjs`, `target/mka-native-tests.log`,
`target/mka-native-wasm-parity.json`, `target/mka-preview-build.log`,
`target/wasm-mka-browser-project.bisect`,
`target/wasm-mka-browser-verification.json`, and `target/wasm-mka-browser.png`.
Engine/practitioner coverage and true Polsby scoring remain incomplete.

### CompactBisect cut slack and seed policy

The WASM engine and laboratory now expose CompactBisect's near-minimum edge-cut
slack as `compact_epsilon` (fraction 0–1), displayed as 0–100%. Legacy requests
retain 5% slack. Single search uses one candidate at each recursive split; multi
uses the seed budget. Native candidates always use seeds 1 through that budget;
changing the starting seed does not alter execution. The starting-seed control
is disabled for CompactBisect, and its fixed seed policy is explained in the
form and recorded in structure evidence. Explicit configurations report the
actual one-or-many candidate budget and an unused starting-seed marker.

Selection follows shared native code: among candidates whose weighted cut is
within `(1+epsilon)` of the minimum candidate cut, maximize geometric-mean
Polsby-Popper using supplied area/perimeter data and boundary weights. This is
best among those sampled candidates, not a global optimum. Uniform or county
weights retain native scoring behavior; no equivalence to a physical perimeter
score under transformed weights is claimed. Invalid slack or parameters on other
structures are rejected. Projects preserve requested slack, candidate count and
fixed seed policy; evidence tampering is rejected.

A native geometric fixture verifies that zero slack excludes the more compact
candidate and 50% slack admits it. Twenty-four exact native/WASM comparisons
cover single/multi searches, two/four districts, uniform/geographic weights and
0/5/100% slack. All are connected and balanced; three invalid requests are
rejected by both. Changing the unused starting seed preserves assignments.
Default behavior remains identical to explicit 5% slack.

The actual RI browser run used eight candidates per split and 20% slack. Its
250 tracts formed two connected districts at 0.2291824428934719% deviation,
passing the requested 0.5% tolerance. Download/reload/Open restored slack,
search, seed budget and map. Independent verification matched assignments and
metrics against hashed input, without executing an engine on Open.

Preview `/wasm-compact-options/`, module 4,282,680 bytes, SHA-256
`d20eaf8216927d7d3a4c3161ad78b498e18101fdfd2f4afe666dd4a0227bdc06`.
Evidence: `scripts/web/test_compact_wasm.mjs`, `target/compact-native-tests.log`,
`target/compact-native-wasm-parity.json`, `target/compact-preview-build.log`,
`target/wasm-compact-browser-project.bisect`,
`target/wasm-compact-browser-verification.json`, and
`target/wasm-compact-browser.png`. Full engine/practitioner coverage remains
incomplete; no global best-cut or certification claim follows from this port.

### AreaSection initialization in WASM

The laboratory exposes `area_init` for `ratio-optimal-area`: `ratio-optimal` (the existing default) or `moving-knife`. The moving-knife initializer uses the native direction-only scan of 180 orientations and applies directional lambda 1 at the root; recursive splits retain the original weights. The scan balances tract count when finding its direction, not population. Population and area feasibility still come from the dual-constraint root partition. This is a heuristic warm start, with no best-cut certificate.

The shared native/WASM direction scan keeps the earliest orientation when Reock scores differ by at most 1e-10. This fixes a platform-dependent near-tie detected on a symmetric grid and can change the selected direction from earlier native builds. Saved assignments remain verifiable without rerunning the engine.

Explicit initialization settings and root direction evidence persist in projects. Legacy projects without the optional setting retain their existing behavior. Displayed root boundary cost uses the original selected boundary weights rather than directional search penalties. Open checks evidence shape and requested policy; it does not attest execution or recompute the direction scan.

Validation: `scripts/web/test_area_init_wasm.mjs` compares 16 compiled native/WASM cases (two initializers, single/multi, 2/4 districts, uniform/geographic weights), checks matching invalid requests and tampered initialization evidence, and independently checks assignments and root metrics. These cases had zero balance misses and zero disconnected plans. The native moving-knife test filter passed 15 tests with one ignored. Full engine and practitioner coverage remains pending.

RI 2020 browser proof: `dist/wasm-area-init/` uses a 4,291,883-byte module, SHA-256 `ebcbf0e7b4a15e15233a9a9ba43035cc577efa218aa53ba2f1cf1317309fe0d2`. Moving-knife initialization, geographic weights, multi search with 8 seeds starting at 42, 100 refinement iterations and area swing 1.1 produced two connected districts across 250 tracts. Maximum population deviation was 0.08064670455695255% (requested 0.5%); root left land share 48.320% passed the area constraint. Recorded direction was 105 degrees. Actual Save/download/reload/Open restored the initializer, settings, results and map. Independent project verification checked hashed inputs and assignment metrics without invoking an engine. Evidence: `target/wasm-area-init-browser-project.bisect`, `target/wasm-area-init-browser-verification.json`, `target/wasm-area-init-browser.png`, `target/area-init-preview-build.log`, and `target/area-init-native-wasm-parity.json`.

### Short-burst search controls and portable execution

The WASM laboratory exposes `burst_length` and `n_bursts` for standard bisection with short-burst ReCom, short-burst Forest ReCom, and short-burst Merge-Split. New UI defaults match native CLI defaults: 20 steps and 50 bursts. Old requests without these fields retain the mapping of `steps` to `ceil(steps/20)` bursts of 20 steps. Both fields must be present together and be nonnegative integers; burst length is at most 100,000, burst count at most 10,000, and their product at most 100,000. Zero bursts or zero steps return the initial assignment. Worker cancellation and per-state time limits remain available.

Native short-burst routines now use ChaCha12 with explicit u64 draws and canonical district-pair ordering through new portable chain step entry points. Existing generic chain `step` entry points keep their legacy draw behavior. The portable ReCom entry uses the Wilson tree sampler (the sampler used by these short-burst routines). This changes seeded short-burst sequences from previous builds, including legacy requests; existing saved assignments remain readable and independently checkable without regeneration. Endpoints are ranked by unweighted cut-edge count and percentile, with burst index as the tie break. User boundary weights affect initialization, not the endpoint ranking.

Explicit options persist in Save/Open. Saved evidence records the chain, burst length, burst count, total configured proposals, endpoint selection policy and portable RNG version. Zero-proposal runs can still record endpoints; `total_proposals` is configured chain steps, not accepted moves. Project Open verifies requested policy and independently checks assignments, population and connectivity, without executing an engine or attesting recorded provenance.

`test_burst_wasm.mjs` passed 144 exact compiled native/WASM cases across three methods, zero/nonzero work, 2/4 districts, seeds 42 and 4294967297, and minimum/median/maximum endpoint percentiles. Six malformed configurations failed in both engines; project evidence tampering failed verification. There were zero balance misses or disconnected plans in this matrix. Native short-burst tests passed 17 with two ignored; the ensemble library passed all 120 tests. The build gate includes this parity matrix and the existing laboratory map, hash, cancellation and project verification gates. Full capability coverage remains pending.

Browser preview: `dist/wasm-burst-options/`, 4,323,288-byte module, SHA-256 `996a77ce82274be8e4d0605fd48f52e2f445380056510668628b56195554f6e6`. Actual RI 2020 browser execution used geographic weights, seed 42, 100 refinement iterations, six Forest ReCom bursts of seven steps, minimum endpoint percentile, and 0.5% population tolerance. Two connected districts covered all 250 tracts; maximum population deviation was 0.002460407935633846%. Save/download/reload/Open restored both burst controls, assignments, evidence and map, without executing an engine on Open. Fresh compiled native/WASM runs matched exact assignments for all three methods on those real inputs; population deviations were 0.4883454121137776% (ReCom), 0.002460407935633846% (Forest ReCom) and 0.32960353715535406% (Merge-Split), all connected and within the requested tolerance. The Forest ReCom assignments also matched the downloaded browser project exactly.

Evidence: `target/burst-preview-build.log`, `target/burst-native-wasm-parity.json`, `target/burst-native-tests.log`, `target/burst-ensemble-tests.log`, `target/wasm-burst-browser-project.bisect`, `target/wasm-burst-browser-verification.json`, `target/wasm-burst-real-parity.json` and `target/wasm-burst-browser.png`. `verify_burst_project.mjs` verifies fresh native/WASM assignments on the saved project's real inputs; `verify_browser_project.mjs` separately checks Open and maps without executing an engine.

### Parallel-tempering controls and support checks

The WASM laboratory exposes the native replica count, swap interval, cold population tolerance and hot population tolerance. The cold tolerance comes from the existing balance-tolerance run setting; the hot tolerance is a separate percentage control. Explicit engine fields are `pt_replicas`, `pt_swap_interval`, `pt_cold_tol` and `pt_hot_tol`, with tolerances encoded as fractions. New UI defaults are four replicas, swap interval ten, and 5% hot tolerance. Tempering steps use the existing `seeds` field, now permitting zero when explicit tempering options are present. A zero-step run returns the initial partition.

Browser bounds: 1–32 replicas, swap interval 1–100,000, cold fraction 0.0001–0.25 matching the balance tolerance, hot fraction between cold and 1, and at most 100,000 replica steps. All four optional fields must be supplied together and apply only to standard bisection with parallel-tempering search. Legacy requests without these fields retain four replicas, interval ten and hot tolerance four times the requested cold tolerance; zero steps remain unavailable for that legacy configuration shape.

The shared exchange implementation now rejects a swap unless each assignment meets the receiving replica's population tolerance. A regression test isolates exchange and verifies that a hot 3:1 partition cannot enter a cold 2:2 population chain, while a valid exchange succeeds. This is a correctness change for native and WASM execution. The native runner also uses ChaCha12 and the portable Forest ReCom step entry point; previous seeded sequences can change. Existing saved assignments remain independently verifiable without regeneration. Generic legacy generator helpers remain available to callers. Cold-chain records are selected by unweighted cut percentile with step-index tie breaking; boundary weights affect initialization. This is a heuristic research workflow, without a global optimum or distribution certificate.

Explicit options and policy evidence persist in projects. Open checks evidence consistency and independently verifies assignments, metrics and hashed inputs without executing an engine or attesting its recorded provenance. The evidence reports configured steps and tolerances, not swap acceptance statistics.

Validation: `test_pt_wasm.mjs` passed 288 exact compiled native/WASM cases across zero/nonzero steps, 1/2/4 replicas, intervals 1/7, equal and increasing ladders, 2/4 districts, two seeds including 4294967297, and minimum/maximum record percentiles. Nine invalid configurations failed in both engines; tampered evidence was rejected. This matrix had zero balance misses or disconnected plans. Native parallel-tempering library tests passed 15; native runner tests passed six with one ignored. Full capability coverage remains pending.

The final RI preview is `dist/wasm-tempering-controls/`, module size 4,319,329 bytes, SHA-256 `996d65f1aba6e2340a618660762141c7a25ad59f687adf9a8cdde3761be7774c`. Actual browser execution used three replicas, 24 steps per replica, swap interval four, cold tolerance 0.5%, hot tolerance 5%, geographic weights, seed 42, 100 refinement iterations and minimum record percentile. It produced two connected districts covering all 250 tracts with maximum population deviation 0.002460407935633846%. Save/download/reload/Open restored the replica count, step count, swap interval, hot tolerance, assignments, policy evidence and map; Open performed no engine execution. A fresh compiled native/WASM replay matched the recorded browser assignments exactly.

Evidence: `target/pt-controls-preview-build.log`, `target/pt-native-wasm-parity.json`, `target/pt-native-tests.log`, `target/pt-runner-tests.log`, `target/wasm-tempering-browser-project.bisect`, `target/wasm-tempering-browser-verification.json`, `target/wasm-tempering-real-parity.json`, and `target/wasm-tempering-browser.png`. `verify_engine_project.mjs` checks a fresh engine replay separately from the no-engine project Open verifier. The earlier `dist/wasm-tempering-options/` pilot preceded zero-step support; the final preview above includes it and passed the 288-case gate.

### One-district project verification (2026-10-04)

The engine returns the all-tracts-in-district-1 assignment when the requested district count is one; it creates no split or search evidence. Project verification previously demanded evidence for explicit CVD, moving-knife and CompactBisect options even in this case, which blocked Save/Open for one-seat allocations and one-district overrides. Those checks now apply only when a split is needed. For one-district results, any non-null root-split or structure/search evidence is rejected as a fabricated claim. Population, connectivity, boundary totals and assignments remain independently verified against hashed graph inputs. The UI explains that no split or search was needed.

The new build gate `test_one_district_wasm.mjs` covers every currently published structure/search combination, three weight modes, and legacy plus explicit option profiles where they differ. On RI (one-district override) and AK (natural one-seat congressional allocation), all 240 cases matched compiled native assignments and completed Save/Open/map roundtrips without invoking an engine on Open. It rejected 720 tampered projects, including fabricated split/search evidence and population mismatches, with atomic rejection. The 14 currently published structures and their supported searches are the test scope; this does not establish the remaining native functionality as ported. Mixed RI/AK laboratory and project integration gates also passed. The existing project roundtrip test compares serialized metrics because JSON normalizes negative zero to zero for an empty boundary sum.

Preview `dist/wasm-one-district/` contains RI and AK 2020 inputs and reuses the unchanged 4,319,329-byte module SHA-256 `996d65f1aba6e2340a618660762141c7a25ad59f687adf9a8cdde3761be7774c`. The fix is in project verification and result presentation; engine execution is unchanged. Evidence: `target/one-district-project-matrix.json`, `target/one-district-project-matrix.log`, `target/one-district-existing-project-tests.log`, `target/one-district-existing-lab-tests.log`, and `target/one-district-preview-build.log`. Full WASM capability coverage remains pending.

Actual browser proof: Alaska 2020 congressional allocation, explicit CVD settings (20 iterations, graph distance), geographic weights, seed 42 and 0.5% balance tolerance produced the trivial one-district assignment covering 177 tracts and population 733,391. It had zero deviation and zero cut boundary, with graph connectivity checked. Actual Save/download/reload/Open restored the CVD configuration and complete Alaska map without starting an engine on Open. Fresh compiled native/WASM assignments matched the downloaded project. UI boundary formatting normalizes signed zero to display `0 km`. Evidence: `target/wasm-one-district-browser-project.bisect`, `target/wasm-one-district-browser-verification.json`, `target/wasm-one-district-real-parity.json` and `target/wasm-one-district-browser.png`. These results establish one-district behavior, not execution of a CVD split or completion of all engine and practitioner requirements.


### Standalone Forest ReCom and Merge-Split portability (2026-10-04)

Both standalone chain runners now use ChaCha12 and the shared portable chain steps: explicit u64 bounded draws, canonical district-pair order, and portable Wilson tree generation. The forward/reverse seed domains remain `FR_FORWARD_`/`FR_REVERSE_` and `MS_STEP_`/`MS_REVERSE_`. This changes seeded sequences from the earlier SmallRng implementation, including requests without new option fields; historical projects retain their recorded assignments rather than being silently rerun. The generic legacy chain-step APIs remain available.

The laboratory accepts zero through 100,000 proposals for these two searches; zero proposals return the initial METIS plan. The initial plan and accepted states are ranked by unweighted cut count, with step order breaking ties. Selection uses floor(percentile × record count), clamped at the final record. Boundary weights affect initialization. This is the native accepted-record selection policy, not selection over every visited state or a proof of a stationary sampling distribution. For more than one district, the recorded policy identifies method, RNG, proposal budget, percentile, ranking rule, selection policy and seed domains. Open checks this ledger when present and independently checks assignments and metrics; legacy projects without the ledger remain readable with provenance unverified.

Weighted boundary reporting now sums in prepared graph edge order rather than HashMap order. The real-input regression found last-decimal differences between targets despite identical assignments; deterministic summation removes that difference without changing the partition algorithm.

Validation: `test_chains_wasm.mjs` passed 192 exact native/WASM cases across both methods, zero/1/20/60 proposals, 2/4 districts, seeds 42/4294967297, minimum/median/maximum percentiles and geographic/uniform weights. Four invalid requests failed in both engines; ledger tampering was rejected and zero-step assignments matched initialization. This matrix had zero balance misses and zero disconnected plans. Native runner filters passed 21 tests with four ignored in total. The final build passed the existing engine/option/laboratory gates, 360 one-district roundtrips and 1,080 tampered-project rejections.

New build gate `test_chains_real_wasm.mjs` covers both methods on up to three available states, preferring RI, IA and NC. It compares complete native/WASM metrics and assignments, independently restores the serialized project with engine creation forbidden, and checks map assignment coverage. In the 2020 RI/IA/NC build, all six cases passed with connected plans and no balance misses. Settings: congressional allocation, 20 proposals, minimum percentile, geographic weights, seed 42, 100 refinement iterations and 0.5% tolerance. Forest ReCom maximum deviations: RI 0.002460407935633846%, IA 0.010249598087241907%, NC 0.1861986545571348%. Merge-Split: RI 0.002460407935633846%, IA 0.042032755458698556%, NC 0.1861986545571348%. These cases cover 2/4/14 districts and 250/896/2,672 tracts; they do not prove the full state/year/chamber matrix.

Final preview: `dist/wasm-chain-replay/`; module 4,313,351 bytes, SHA-256 `334bab43f8f86d410828764df06e8b759ed2be4a30a580bdf5c508a3f92cb934`. Both methods also ran in the actual browser and were saved/downloaded. The final downloaded projects passed independent no-engine Open verification and fresh native/WASM replay against their recorded browser assignments. Evidence: `target/chains-replay-preview-build.log`, `target/chains-native-wasm-parity.json`, `target/chains-real-input-parity.json`, `target/wasm-forest-chain-final-project.bisect`, `target/wasm-merge-chain-final-project.bisect`, their `final-verification.json` and `final-parity.json` reports. The earlier `dist/wasm-standalone-chains/` pilot predates deterministic weighted-metric summation. Full engine, practitioner, project/library and TUI capability coverage remains pending; standalone boundary-flip and local-bisection-ensemble portability are upcoming engine work.

Actual final browser reload/Open restored the Merge-Split controls, three-state results and North Carolina map. The imported-project banner confirms that Open verified assignment evidence without engine execution. Screenshot: `target/wasm-chain-final-browser.png`.


### Boundary-flip portability and population checks (2026-10-04)

The standalone boundary-flip runner now uses ChaCha12 with explicit u64 bounded random draws and a sorted adjacent-district list. The seed derivation remains `FLIP_CHAIN_` plus the original encoded seed components. Seeded assignments change from the earlier SmallRng implementation, including legacy requests. Before the change, the four-district parity probe selected different native/WASM assignments and population deviations of 100%/75% despite a requested 25% tolerance (`target/flip-before-parity.json`).

The shared native runner also had a population-check defect: it allowed deviation based on total state population and rounded deviations down to integer counts with added slack. Accepted moves now compare relative deviation against ideal district population directly, without integer truncation or added population slack. Source districts cannot become empty. Both native Rust and WASM use this corrected implementation. The initial METIS plan is still retained, so this change is not a universal guarantee that every input produces a feasible final plan; actual population and connectivity checks remain authoritative.

The browser supports zero through 100,000 flip proposals. Zero returns initialization. Initialization uses the native fixed 100 refinement iterations, now shown as a disabled control and in effective settings. The initial plan and accepted moves are ranked by unweighted cut, with insertion order breaking ties and floor(percentile × record count), clamped, selecting the record. Boundary weights affect initialization. The recorded ledger contains method, RNG, proposals, percentile, selection/rank rule, seed domain, initial refinement count, population rule, nonempty-district rule, retained count and selected rank. Open validates ledger consistency when present, without claiming attested execution or a certified sampling distribution.

Historical ledger-free projects remain readable whether their old effective settings recorded the unused requested refinement count or the fixed 100. A one-district integration test exposed the need to accept both representations when there is no ledger; the verifier now does so. Other recorded options, assignment metrics and hashed input identities remain checked. Newly recorded flip ledgers require the fixed count and corrected population rule.

Validation: `test_flip_wasm.mjs` passed 384 exact native/WASM assignment, metric and ledger cases: zero/1/20/200 proposals, 2/4 districts, seeds 42/4294967297, minimum/median/maximum percentiles, geographic/uniform weights, 5%/25% tolerances, and requested refinement counts 1/100. All twelve ledger fields were tamper-tested. Four invalid configurations failed in both engines. The matrix had zero balance misses and disconnected plans. Native flip tests passed nine with one ignored; regressions exercise ideal-district tolerance, low-count populations without integer slack, and selected accepted-record bounds. The project, package and evidence-worker security suites passed.

The initial preview build passed the compiled engine/option matrices and laboratory integration test, then stopped at the one-district compatibility issue described above. After the verifier fix, the exact failed gate was rerun: 360 one-district native/WASM Save/Open/map roundtrips passed with 1,080 tampered projects rejected. Laboratory project integration passed, and the expanded real-input build gate passed nine cases across Forest ReCom, Merge-Split and boundary flip on RI, IA and NC, with matching complete native/WASM assignments/metrics, no engine execution on Open and matching restored maps. Its flip legacy-project compatibility checks also passed. Evidence: `target/flip-preview-build.log`, `target/flip-native-tests.log`, `target/flip-native-wasm-parity.json`, `target/flip-one-district-recovery.log`, `target/flip-project-integration.log`, `target/flip-real-input-tests.log`, and `target/chains-real-input-parity.json`. These gates are included in subsequent builds; no failed full-build command is represented as a successful exit.

Preview `dist/wasm-flip-controls/`: 4,315,608-byte module, SHA-256 `df47572e7dc8e66290acff73d37348f555283b4c7107b473eb2787cd0bb8d61a`. Actual browser execution used 200 proposals, minimum percentile, geographic weights, seed 42, 0.5% tolerance and congressional allocations. It produced connected plans in RI/IA/NC with maximum deviations 0.002460407935633846% / 0.010249598087241907% / 0.1861986545571348%. NC retained 62 records and selected rank zero. Save/download/reload/Open restored controls, all three state results and the NC map without executing the engine. Independent Open verification and fresh compiled native/WASM replay matched the downloaded assignments. Evidence: `target/wasm-flip-browser-project.bisect`, `target/wasm-flip-browser-verification.json`, `target/wasm-flip-real-parity.json`, `target/wasm-flip-browser.png`.

Full engine, practitioner, project/library and TUI coverage remains incomplete. Local-bisection-ensemble portability is the next search gap; the full state/year/chamber matrix and other documented native commands still require implementation and verification.


### Local-bisection ensemble portability (2026-10-04)

The native local-bisection ensemble now uses ChaCha12 and the shared portable ReCom step with fixed-width bounded draws and Wilson trees. The local graph is still indexed by sorted global tract IDs, the incoming node seed remains unchanged, and the existing unequal target-population constructor is retained. This changes seeded sequences from the earlier SmallRng implementation for nonzero proposals. The pre-change compiled native/WASM probe selected different assignments (`target/local-ensemble-before-parity.json`). No stationary distribution or optimality claim is inferred from the port.

At each prescribed recursive split, retain the initial METIS partition and accepted local cuts, rank by unweighted edge cut using stable insertion-order ties, and select floor(percentile × retained count), clamped. Boundary weights affect initialization; local ranking uses unweighted cut count. The browser now labels the budget as proposals per split and accepts zero through 100,000. Zero proposals match the standard initial partition. Regions of at most four tracts retain the native single-METIS shortcut regardless of budget or percentile. The ledger records method, RNG, per-split budget, percentile, selection/rank rules, floor/ceil seat target policy, small-region shortcut, scope and Wilson sampler. Project Open checks the ledger when present plus independent assignments/metrics against hashed inputs; historical ledger-free results remain readable with provenance unverified.

The native unit filter passed six tests. New regressions verify 1:2, 2:3 and reversed 2:1 target-population orientation, including both sides' connectivity; zero proposals and the small-region shortcut match initialization; high seeds reproduce across chain instances. The final targeted `test_local_ensemble_wasm.mjs` passed 304 exact compiled native/WASM assignment and metric cases. Its main 288 cases cover zero/1/20/60 proposals, 2/3/5 districts, seeds 42/4294967297, minimum/median/maximum percentiles, geographic/uniform weights and refinement counts 10/100. Sixteen additional 3/4-tract cases verify the shortcut. Ten ledger fields were tamper-tested; four invalid configurations failed in both engines. There were zero balance misses or disconnected plans in this matrix. Evidence: `target/local-ensemble-native-tests.log`, `target/local-ensemble-final-parity.log`, `target/local-ensemble-native-wasm-parity.json`.

The complete preview build exited successfully with the compiled option matrices, laboratory integration, 360 one-district roundtrips and 1,080 tampered-project rejections, and project integration. Its real-input gate now covers four searches (Forest ReCom, Merge-Split, boundary flip, local bisection ensemble) on RI/IA/NC: twelve cases with exact native/WASM assignments and complete metrics, independent no-engine Save/Open validation and restored map assignment coverage; all were connected and within 0.5%. The build ran the main 288-case local matrix before the additional shortcut tests were added; the final 304-case matrix was then run separately and is used by subsequent builds. Evidence: `target/local-ensemble-preview-build.log`, `target/chains-real-input-parity.json`. The project and evidence-worker security tests also passed, and the 69-entry native command/screen inventory still matches current sources.

Preview `dist/wasm-local-ensemble/`: module 4,303,751 bytes, SHA-256 `5ab109d7cf654318f51255c0d5de499e05b27d0eb95c6d2db2ec058ee9b2e360`. Actual browser run: 20 proposals per recursive split, minimum percentile, geographic weights, seed 42, 100 refinement iterations, 0.5% tolerance and 2020 congressional allocations. RI (2 districts/250 tracts), IA (4/896), NC (14/2,672) all completed with connected plans; maximum deviations were 0.019592137265256326%, 0.24583363241055256%, and 0.31858189388114244%. Downloaded assignments matched fresh native/WASM replay. Actual reload/Open restored controls, all three results and the North Carolina map without engine execution. Evidence: `target/wasm-local-ensemble-browser-project.bisect`, `target/wasm-local-ensemble-browser-verification.json`, `target/wasm-local-ensemble-real-parity.json`, `target/wasm-local-ensemble-browser.png`.

Full WASM coverage is not complete. Native `ratio-optimal-vra` and `vra-recom` remain absent from the browser engine; demographic/election-aware inputs and engine paths are upcoming dimensional work. Other native command, practitioner, TUI, library/archive and full state/year/chamber requirements remain pending as documented in the capability inventory and specification.

### Demographic boundary and VRA ReCom core (2026-10-04)

The engine API now accepts `vra-recom` with a required threshold and complete `bisect-demographic-fractions-v1` data matching graph state, year, and every tract GEOID. Input fractions must be finite and between zero and one; negative zero, sparse coverage, extra tracts, unknown fields, and incompatible options are rejected. Declared bases are total population, voting-age population, and citizen voting-age population. Source labels are user supplied, not authenticated. The existing native sampler uses an unweighted mean of tract fractions; it does not aggregate raw population or VAP counts, and this rule alone does not establish legal VRA compliance.

Native and WASM execution share ChaCha12 and portable Forest ReCom proposals. Evidence records the initial assignment, protected district IDs, proposal/acceptance/rejection counters, retained-record count, selected rank, and a canonical SHA-256 demographic identity. JSON decimal parsing uses `float_roundtrip`: a compiled test exposed differing binary fractions and hashes between JavaScript and the default Rust parser. The browser helper and Rust boundary now hash sorted GEOIDs and little-endian binary fractions identically, independently of source label or JSON field order.

Validation: the self-contained `test_vra_recom_wasm.mjs` matrix passed 432 exact native/WASM assignment, metric, and option cases, plus twelve matching input/option rejections. Sixty cases exercised minority rollback; all final protected districts remained at or above threshold, with no balance misses or disconnected plans. Native runner tests passed six with one data-dependent test ignored; twelve ensemble tests and the WASM examples check passed. The demographic helper tests also passed. Evidence: `target/vra-native-wasm-parity.json`, `target/vra-runner-tests.log`, `target/vra-ensemble-tests.log`, `target/vra-examples-check.log`.

This is the core/input phase. Browser controls, demographic file selection, project persistence and independent verification of the new preservation ledger are still pending. The sampler is deliberately not listed as a browser-selectable search until those paths are implemented. `ratio-optimal-vra`, raw-count demographic/election adapters, and the remaining capability inventory are also pending. Subsequent laboratory builds run the new demographic and VRA gates and export their browser helper and strict JSON worker.

The complete fresh RI/IA/NC preview build passed all configured parameter matrices and integration gates. This includes 360 one-district native/WASM and Save/Open/map cases, 1,080 tampered-project rejections, laboratory project restoration with no engine execution on Open, and twelve real-input chain replay/Save/Open/map cases. All three declared demographic bases, reordered input maps, user-label independence, and the VRA one-district bypass also passed separate native/WASM probes. Evidence: `target/vra-preview-build.log`, `target/vra-final-parity.log`, `target/vra-project-security.log`. Preview: `dist/wasm-vra-boundary/`; module 4,514,715 bytes, SHA-256 `2128e0c2954dcd0ce0723586e7ec76db34808249a594699794e93db03e3b9857`. The new VRA core has not yet been exercised through browser controls or saved projects; those remain the next implementation phase.

### VRA ReCom laboratory and project workflow (2026-10-04)

`vra-recom` is now selectable with standard bisection in the WASM laboratory. Users choose a preservation threshold, load one complete demographic JSON input per selected state, and run state or national experiments. Zero proposals return initialization. Population tolerance remains the native fixed 0.5%; boundary weights and requested METIS refinement iterations apply to initialization. The UI explains that preservation uses an unweighted tract mean. A minority-fraction map view colors individual tracts, and a district summary displays rounded tract means and initially protected districts. Source labels are rendered as text and remain user-provided claims.

The experiment configuration stores `vra_threshold` and a state-keyed `demographics` map; only the threshold goes into engine options, and each engine request receives its state's input separately. Save/Open and autosave retain the full inputs. Existing non-VRA project formats remain readable. Opening independently checks complete graph coverage, demographic scope/basis, canonical SHA-256 identity, initial protected IDs, final preservation, zero-step assignments, and proposal/retained-record/rank accounting in the verification Worker. It never runs the engine or authenticates execution history. A user can alter all unsigned claims together; hash consistency does not authenticate demographic sources or establish legal VRA compliance.

Demographic files use these fields: `schema_version: "bisect-demographic-fractions-v1"`, a two-letter `state`, four-digit `year`, `basis` (`total-population`, `voting-age-population`, or `citizen-voting-age-population`), a nonempty `source_label`, and `minority_fractions` mapping every eleven-digit tract GEOID to a finite fraction in [0,1]. Select at most fifty files totaling 25 MiB. Files are parsed in bounded strict JSON workers; duplicate states within a selection, prototype keys, invalid coverage/scope, and stale file reads cannot partially replace inputs. Changing the selected year requires data matching that year. Raw count CSV, population-weighted VAP/CVAP aggregation, and `ratio-optimal-vra` remain pending.

Verification: 432 synthetic-grid native/WASM cases also pass independent preservation-ledger and hash validation, with individual ledger-field tamper rejection. The real-input project gate covers RI/IA/NC graphs with explicitly synthetic demographic fractions at zero and twenty proposals: six exact native/WASM and Save/Open/map cases, twelve atomic tamper rejections, and actual verification Workers. Browser execution used the same labeled synthetic inputs, twenty proposals, minimum cut percentile, geographic weights, seed 42, 100 refinement iterations and 2020 congressional allocations. RI, IA and NC completed with connected plans and deviations 0.002460407935633846%, 0.010249598087241907%, and 0.24924832758396187%; Iowa had three and North Carolina six minority-rule rejections. The downloaded project matched fresh native/WASM replay and reopened after reload with all inputs, settings, results and the minority map restored. A hostile prototype-key demographic file was rejected without replacing completed results.

Evidence: `target/vra-ui-parity.log`, `target/vra-ui-project-verification.json`, `target/vra-ui-one-district.log`, `target/vra-ui-lab.log`, `target/vra-ui-security.log`, `target/wasm-vra-browser-project.bisect`, `target/wasm-vra-browser-verification.json`, `target/wasm-vra-browser.png`. This verifies the sampler workflow on the named inputs; it does not complete all native engine, practitioner, TUI, data-preparation, archive/library, or state/year/chamber requirements.

The full `build_wasm_lab.ps1` export to `dist/wasm-vra-laboratory/` completed successfully. Every configured native/WASM matrix passed, followed by laboratory integration, 369 one-district assignment/Save/Open/map cases, 1,107 tampered-project rejections, generic project restoration, twelve real-input chain cases, and the six-case VRA project gate. Module: 4,514,715 bytes, SHA-256 `2128e0c2954dcd0ce0723586e7ec76db34808249a594699794e93db03e3b9857`. Evidence: `target/vra-ui-build.log`. Actual browser Open and the minority map/table were also checked in this final export. The 69-entry command/screen inventory remains broader than implemented coverage; full goal completion is still unproven.

### VRASection root scoring and browser workflow (2026-10-04)

`ratio-optimal-vra` now runs in the engine API and WASM laboratory with single or multi seed search and explicit `w_vra` in [0,1] (UI default 40%). It requires complete total-population demographic fractions. As in the native CLI, each tract contributes minority mass proxy `fraction * graph population`; true VAP/CVAP counts are not inferred. At each root ratio the native engine first retains the seed with minimum weighted cut. Across ratios it minimizes `normalised_cut - w_vra * alignment * max(normalised_cut,1)`, where `normalised_cut = weighted_cut / sqrt(min(left seats,right seats))` and `alignment = 2 * abs(left minority-mass share - 0.5)`. Descendants run ordinary ratio scans without the demographic bonus. Two districts with single search retain the native direct-split shortcut; one district bypasses splitting entirely.

Unequal population halves can receive a nonzero alignment bonus with uniform minority fractions. The formula does not normalize against the root population share or use district minority percentages. The UI makes this behavior explicit and keeps it separate from ReCom's protected-district rule. Zero minority mass or zero alignment weight reproduces ordinary GeoSection assignments. Weight values cannot be interpreted as literal fixed percentage contributions to an aggregate objective. No legal VRA compliance or globally optimal split is claimed.

Native mass accumulation now follows graph-index order; root boundary reporting follows canonical edge order, so selected-score calculations are portable. The shared native function rejects malformed/nonfinite/negative minority mass and out-of-range weights. The engine records sixteen selected-root evidence fields, including basis, aggregation, mass/share, alignment, normalized cut, adjusted score, weight, selection and tie policies, nominal ratio count, shortcut flag, and demographic SHA-256. Save/Open preserves inputs and the weight; its verification Worker independently recomputes the selected-root quantities and data identity without executing the engine. This checks consistency of the selected witness, not optimality over unrecorded candidates or author authenticity.

The UI has a Root split side map view for inspecting the first cut independently of descendant districts. VRASection can also display the input minority fractions on the map. The root summary only mentions area constraints when the selected structure actually activates them.

Validation: `test_vra_section_wasm.mjs` passed 576 exact native/WASM cases across zero/uniform/concentrated/decimal fractions, 2/3/5 districts, single/multi search, zero/40%/100% weight, high seeds, geographic/uniform/county weights and tiny physical boundaries. Nine invalid configurations failed in both engines and all sixteen evidence fields were tamper tested. Sixty-four cases exercised negative adjusted scores below the normalization floor; ninety-six preserved the native uniform-fraction alignment behavior. Nine sampled plans missed balance tolerance and none were disconnected; parity does not turn those misses into successful balance checks. Native VRA tests passed fifteen with one data-dependent test ignored, and the WASM examples check passed. The 432-case ReCom regression still passed.

The expanded real-input project gate passed 24 exact native/WASM and Save/Open/map cases on RI/IA/NC graphs with explicitly synthetic fractions: six ReCom cases and eighteen VRASection cases spanning single/multi and all three alignment weights, with 48 atomic tamper rejections and real verification Workers. The complete preview build also passed its configured option matrices, laboratory integration, 387 one-district native/WASM/Save/Open/map cases, 1,161 tampered-project rejections, generic project restoration, and twelve real-input chain replay cases. The root-color view and corrected area wording were added after export, copied into both preview directories, syntax checked, and verified in the browser.

Actual browser run: 2020 congressional allocations for RI/IA/NC, geographic boundaries, seed 42, four seeds per ratio, 100 refinement iterations, 0.5% tolerance and 40% alignment weight, with explicitly labeled synthetic fractions. All three results were connected and within tolerance: RI 0.2291824428934719%, IA 0.24539481169733035%, NC 0.262659075417071%. North Carolina selected root ratio 6:8 and took about 20.5 seconds in this run. The downloaded project matched fresh native/WASM replay and reopened after navigation/reload in the final export with settings, inputs, results and the root-side map restored, without engine execution on Open.

Evidence: `target/vra-section-native-wasm-parity.json`, `target/vra-section-native-tests.log`, `target/vra-section-check.log`, `target/vra-section-recom-regression.log`, `target/vra-section-project.log`, `target/vra-ui-project-verification.json`, `target/vra-section-one-district.log`, `target/vra-section-build.log`, `target/wasm-vra-section-browser-project.bisect`, `target/wasm-vra-section-browser-verification.json`, `target/vra-section-browser-replay.log`, `target/wasm-vra-section-browser.png`. Preview `dist/wasm-vra-section-laboratory/`: module 4,518,216 bytes, SHA-256 `40bbb55b54303a48461041c6dec5090c4e12aacb0bd49a6a88cbf895e0fb89df`.

At this checkpoint, demographic CSV/count adapters were pending. The next phase implements import, count retention and descriptive district aggregation. Count-based VRA engine/audit paths, other engine tuning/preparation options, practitioner/TUI/library/archive workflows and full state/year/chamber coverage remain pending. The 69-entry inventory does not imply those entries are all implemented.

### Demographic CSV/count phase

`bisect-data::demographics::import_demographic_csv` is now an in-memory native adapter used by the WASM toolkit's `import-demographic-csv` operation. It supports the existing GEOID/total-population/non-Hispanic-white and GEOID/VAP-or-CVAP/minority-VAP formats with explicit state/year/basis/source metadata, proper CSV quoting, zero padding and strict invalid/duplicate/count-bound rejection. The historical CLI's lenient readers remain separate. CSV import has an 8 MiB/100,000-record bound; the dedicated browser Worker has a ten-second deadline, module SHA verification and cleanup on all outcomes.

New `bisect-demographic-counts-v2` retains `{total,minority}` by GEOID along with fractions. Both native requests and browser projects validate complete coverage and exact fraction/count arithmetic in native f64 semantics. Existing v1 fraction inputs remain readable. The engine's identity remains a hash of operational fractions/scope; retained counts and source labels are unsigned source claims. UI count-share reporting is descriptive and does not alter the native VRA ReCom tract-mean rule or VRASection population proxy; VRASection still rejects VAP/CVAP. Count-based VRA audit/workbench integration remains unfinished.

Two native parser tests, 17 exact native/WASM comparisons, four real-RI engine/project/map cases with synthetic counts and 33 malformed/tampered-count rejections passed. Real Worker tests verify conversion, count retention, module integrity failure, UTF-8/scope/duplicate rejection, pre-read size/cancellation and termination. Existing ReCom 432 and VRASection 576 parity matrices and project/package/assignment security suites passed. Browser VAP CSV import, run, Save, reload/Open, count tables and duplicate rejection preserved the completed plan. Fresh native/WASM replay agrees with the downloaded project.

Evidence: `target/demographic-csv-verification.json`, `target/demographic-csv-parity.log`, `target/demographic-csv-browser-replay.log`, `target/wasm-demographic-csv-browser-project.bisect`, `target/wasm-demographic-csv-browser-verification.json`, `target/wasm-demographic-csv-browser.png`. Preview `dist/wasm-demographic-laboratory/` has a 4,587,815-byte module, SHA-256 `bf64002ae72fd705bf26c86f5af900718be6819476571eba47e6c0248a60fcbc`. This phase does not complete the original all-engine/all-toolkit objective.

Full export passed all configured matrices and project gates (`target/demographic-csv-full-build.log`), including 387 one-district cases/1,161 tamper rejections, twelve real-input chains and 24 VRA project cases/48 tamper rejections. The CSV Worker gate was added during the run and passed separately against the final export (`target/demographic-csv-worker.log`); future builds run it directly. Late table-scroll styling was copied to both previews and verified with syntax checks and browser screenshots. Final exported JS/CSS match source. Remaining inventory entries retain their original scope and pending status.


### Native plan export bridge (2026-10-04, implementation in progress)

The practitioner workbench now supports attaching complete tract VAP/CVAP CSV counts to a matching native RCTX, explicitly applying a reporting policy, generating a count-based audit certificate and verifying it. The native audit uses district sums and strict share > threshold, distinct from the sampler's tract mean. Original CSV bytes are retained in practitioner projects. The latest toolkit build passed 98 exact native/WASM cases and 51 malformed, stale or tampered-input rejections (`target/vra-audit-verification.json`). This supersedes the earlier statement that the count-audit adapter and controls were unfinished. Full manual browser Save/Open verification of that workflow remains pending.

A new Rust toolkit operation, `export-engine-plan`, accepts an engine request, exact GEOID-keyed one-based assignments, label, chamber and creation date. It validates the prepared graph and assignment coverage, preserves explicit graph unit order, converts district IDs to native zero-based IDs, writes native RPLAN 0.2 and RCTX 0.1, and checks both through their native file readers. Populations, county IDs, adjacency, physical edge weights, graph hash and engine options are retained. Edges are explicitly custom: the prepared format lacks original bridge/land-edge classification. Geometry is absent rather than inferred from centroids. Supplied VAP/CVAP counts are retained with their explicit basis; total-population proxies and fraction-only inputs do not become VAP counts. Assignment export is unsigned and does not prove generation, balance, connectivity or optimality; native audits remain separate explicit operations.

Verification: `scripts/web/test_plan_export.mjs` passes seven exact native/WASM comparisons and ten matching rejections. It runs the WASM engine, exports the actual generated assignments, checks non-sorted unit order and each physical edge weight, validates the RPLAN, audits population/contiguity and verifies the resulting certificate. It also covers VAP/CVAP count retention, exclusion of total-population proxy counts, missing/extra/out-of-range labels, malformed graphs and altered contexts. Evidence: `target/plan-export-verification.json`. The gate is included in future practitioner builds. Rust native and WASM release builds pass. Existing generic WASM and count-audit regressions passed during this phase.

The export operation is not yet connected to laboratory UI controls or portable project handoff. That integration, real-state browser Save/Open/audit verification, remaining engine options and the full practitioner/TUI inventory still require work. The original whole-toolkit WASM objective remains active and incomplete.


### Laboratory to practitioner project handoff (2026-10-04)

Completed state results now have a Prepare selected state for auditing control and a separate direct-click Download audit project control. National runs use the currently selected state. Preparation rechecks assignments and recorded metrics against the hashed prepared graph, invokes the native export adapter in WASM, releases graph memory and produces an ordinary `bisect-project-v1` with native plan/context inputs. Opening that file in the practitioner workbench does not run generation or auditing. No legal profile is invented or embedded; the user loads it explicitly. Concurrent exports/runs/restores are rejected, changed selections cannot download the previously prepared state, and failed validation does not emit a project.

`build_wasm_lab.ps1` now builds the practitioner workbench under `toolkit/`, with its existing native examples and security/parity gates. The lab links to that bundled workbench. `test_audit_handoff.mjs` checks generated RI/IA/NC plans, native/WASM certificate equality, project Open and certificate roundtrip, exports from restored lab projects, graph-cache release, concurrency rejection and assignment tamper rejection. Three real states passed; laboratory integration and existing lab project tests passed. The complete nested practitioner build also passed all its configured gates. Preview: `dist/wasm-audit-handoff/`; evidence: `target/audit-handoff-verification.json`, `target/audit-handoff-*-prepared.bisect`, `target/audit-handoff-*.bisect`.

Browser verification covered RI generation, reload/autosave restoration without rerunning, preparation of the restored state, navigation to the bundled workbench, Open Project using an integration-exported RI project with no automatic audit, explicit profile selection, passing shape/population/contiguity checks and certificate verification. Screenshots: `target/wasm-audit-handoff-browser.png` and `target/wasm-audit-handoff-checks.png`. The in-app browser reported a download request but neither a download event nor a file in Downloads was observed; actual UI disk-save completion remains unverified. The separate direct-click download flow is implemented, but requires a browser with observable downloads for conclusive end-to-end file-save proof. The file opened in the browser was produced by the integration export, not claimed to be that unconfirmed download.

This establishes one-state audit-project handoff within national or state experiments. It does not yet preserve a whole national lab plus all practitioner projects/certificates in one unified archive, nor complete the remaining native engine and TUI/library capability inventory. The original entire-capability WASM objective remains active.


### N-way METIS refinement controls (2026-10-04)

Direct n-way partitioning now exposes the pure Rust METIS refinement objective (`cut` or `volume`) and internal trial count (1..100). Existing refinement iterations apply to each trial. Options are accepted only for n-way; legacy inputs retain cut/one-trial behavior. Native `run_nway_partition_tuned` and the WASM adapter share the implementation. The C FFI wrapper preserves legacy defaults and explicitly rejects advanced values; native CLI objective flags are not claimed to be wired by this change. Other recursive methods still need objective/control coverage.

Volume controls FM refinement, but metis-core selects among internal trials by population excess then edge cut for either objective. The UI describes this boundary. Runtime evidence records the objective, trial count, iterations, selection order and edge-weight scaling. Projects retain and check this evidence, restore controls and reject altered evidence. One-district results remain trivial with no split/refinement evidence. No global optimality or guaranteed balance is claimed.

Verification: `test_nway_tuning.mjs` passed 24 exact native/WASM cases spanning two objectives, two trial counts, two iteration budgets and three seed magnitudes. Five invalid requests were rejected by both engines and evidence tampering was rejected. There were 19 distinct assignments, eight population-tolerance misses and zero disconnected results on the synthetic fixture; misses remain visible, not repaired or hidden. RI/IA/NC real-state projects retain the controls, reopen without generation and reject altered trial evidence. Generic engine, lab/project security and audit-handoff regressions passed; the nested practitioner build passed its configured gates. Evidence: `target/nway-tuning-verification.json`, `target/nway-volume-*.bisect`.

Browser RI run with volume/three trials and ten refinement iterations completed on 250 tracts: two connected districts, 4.785% population deviation under a requested 5% bound, approximately 52 km graph boundary and one split county. Reload/autosave restoration retained the controls and result without rerunning. Screenshot: `target/wasm-nway-controls-browser.png`. Preview: `dist/wasm-nway-controls/`, including `toolkit/`. A text-encoding error introduced during editing was corrected and the exported source refreshed before final browser restoration. The existing unconfirmed in-app-browser disk-download boundary is unchanged. The full native-capability WASM objective remains incomplete.


### Recursive METIS controls for standard Single/Multi (2026-10-04)

The shared native split path now accepts an explicit cut/volume refinement objective and 1..100 internal trials. Legacy entry points pass cut/one-trial defaults, including the existing NRS profile. Standard recursive Single/Multi in WASM passes the controls to every prescribed floor/ceil node. Multi keeps its separate outer seed budget and balanced/contiguous candidate filter, selecting minimum weighted boundary cost without completing child plans to rank a parent cut. Native population rebalance still runs after METIS. Internal trials keep metis-core's population-excess then edge-cut ranking even for volume refinement. The UI and evidence state these different selection rules.

Result evidence records per-candidate trials, refinement iterations, tree scope, outer candidate count/selection, wrapping seed schedule and native post-refinement. Projects retain settings and reject altered evidence. One-district execution remains trivial with no refinement evidence. Unsupported searches reject these fields rather than ignore them. The C FFI path preserves default behavior and rejects advanced tuning; native CLI flags are not newly claimed to be wired. Controls for other methods/searches remain pending.

Verification: 96 exact native/WASM cases, five matching invalid-request rejections and 864 evidence tamper rejections passed. The synthetic fixture covers odd/even district counts, Single/Multi, cut/volume, one/three internal trials, one/ten iterations and three seed magnitudes; it had no generation failures, balance misses or disconnected results. Six RI/IA/NC Single/Multi project roundtrips retain controls, reopen without generation and reject altered evidence. Six existing native pure-Rust split tests passed. Generic WASM, n-way tuning, laboratory/project, audit-handoff and the complete nested practitioner build gates passed. Evidence: `target/recursive-tuning-verification.json`, `target/recursive-volume-*-*.bisect`. These named cases do not establish universal feasibility or optimality.

Actual browser RI standard Multi with volume, three internal trials, four outer seeds and ten refinement iterations produced two connected districts on 250 tracts, 0.132% population deviation, about 50.3 km graph boundary and one split county. Reload/autosave restoration retained settings/result without rerunning. Screenshot: `target/wasm-recursive-controls-browser.png`. Preview: `dist/wasm-recursive-controls/`, including the practitioner workbench under `toolkit/`. The previously unconfirmed in-app-browser disk-download boundary remains. The entire native-engine/practitioner/TUI WASM goal remains active and incomplete.


### Percentile METIS controls (2026-10-04)

Standard recursive percentile search now accepts cut/volume refinement and 1..100 internal trials per split. The prescribed SHA-256 seed walk and full-plan ranking remain unchanged: candidates rank by unweighted edge cut, with seed index breaking ties; floor(percentile * candidate count), clamped to the last rank, selects the plan. This is a sampled family, not a claim of global optimality. Legacy calls retain cut/one-trial behavior.

Results record every candidate cut, selected rank/index/cut, seed-walk identity and refinement settings. Project Open checks consistency, selected assignment edge cut and ranking without executing generation. Other candidate cut claims are not replayed on Open; unsigned, coordinated alterations can remain internally consistent. The laboratory restores settings and displays the selection rule and result evidence.

Verification: 48 exact native/WASM comparisons, 32 independent native candidate replays, 624 evidence tamper rejections and three RI/IA/NC project roundtrips passed, with no balance misses or disconnected plans on these fixtures. Seven native percentile module tests passed. Generic engine, recursive and n-way tuning regressions, laboratory/project and audit-handoff gates, and the nested practitioner build passed. Evidence: target/percentile-tuning-verification.json and target/percentile-volume-*.bisect.

Actual browser RI with four candidates, 100th percentile, volume refinement, three internal trials and ten iterations produced two connected districts on 250 tracts: 0.132% deviation, about 50.3 km graph boundary and one split county. Selected rank was 4/4, seed index 3 and 23 cut edges. Reload and Restore autosave retained controls and evidence without generation. Screenshot: target/wasm-percentile-controls-browser.png. Preview: dist/wasm-percentile-controls/, including toolkit/. Actual disk-download completion and the remaining engine/practitioner/TUI capability inventory are still incomplete; the full WASM objective remains active.


### Convergence METIS controls (2026-10-04)

The no-improvement seed sweep now accepts cut/volume refinement and 1..100 internal trials at each prescribed recursive split. A shared native tuned entry point feeds the WASM engine. Feasible full plans still rank by recursive normalized weighted cut, strict improvement keeps earliest ties, consecutive seeds use wrapping u64 arithmetic, rejected candidates count toward the stale tail, and stopping distinguishes the no-improvement threshold from the maximum-seed cap. Legacy requests omit new refinement evidence and retain default assignments and stopping summaries.

Tuned summaries include exact refinement metadata (objective, trials, iterations, tree scope, internal population-excess/edge-cut selection and native population rebalance). The UI displays these settings and stopping evidence; Save/Open checks metadata and the selected assignment's score, feasibility and stopping consistency. Opening never replays generation and does not authenticate unsigned execution claims or prove global optimality.

Verification: six native convergence tests passed. The expanded real RI/IA 2020 matrix passed 60 native/WASM assignment, district-metric and summary comparisons across three boundary weights, two caps, legacy settings and cut/volume with one/three internal trials. All 205 attempted candidates were replayed independently of the sweep to check ranking and stopping. Explicit cut/one-trial defaults match legacy assignments and summaries. Ten project roundtrips and 98 tamper rejections passed, including every refinement field; a no-feasible-plan request rejects. Evidence: target/native-wasm-convergence-parity.json. Recursive (96), n-way (24) and percentile (48 plus 32 native candidate replays) regressions passed, as did laboratory and project integration. The convergence gate is included in build_wasm_lab.ps1 and selects available multi-district congressional graphs for the requested year; exports with none explicitly skip this real-state gate.

Actual browser RI volume/three-trial run tried four seeds, rejected none, selected seed 42 and reached a three-seed stale threshold. It produced two connected districts on 250 tracts, 0.132% deviation, approximately 50.3 km boundary and one split county. Reload/Restore autosave preserved controls, stopping evidence and results without generation. Screenshot: target/wasm-convergence-controls-browser.png. Preview: dist/wasm-convergence-controls/. The nested practitioner build passed all configured gates and is bundled under toolkit/. Whole native/practitioner/TUI coverage, unified national archives and actual browser disk-save proof remain unfinished; the full WASM objective remains active.


### Local ensemble METIS initialization controls (2026-10-04)

Local bisection ensembles now accept cut/volume refinement and 1..100 internal METIS trials for initialization at each recursive node and for the at-most-four-tract shortcut. Native legacy entry points retain cut/one-trial defaults. The shared tuned native split and tree entry points are used by WASM. Wilson tree proposals, prescribed floor/ceil targets, portable ChaCha12 draws and initial/accepted unweighted-cut percentile ranking remain unchanged. Zero proposals return tuned initialization. Internal trials still rank population excess then edge cut; native population rebalance follows refinement.

Tuned result evidence includes initialization objective, trials, iterations, scope and ranking/post-refinement rules. Save/Open requires this evidence when tuning is recorded, rejects absent or altered metadata, restores the controls and independently checks assignments/metrics without generation. Historical untuned projects can still lack their old optional policy ledger. Evidence remains unsigned execution provenance, not proof of optimality or sampling distribution.

Verification: 256 exact native/WASM cases cover cut/volume, one/three trials, zero/twenty proposals, odd/even district counts, two seed magnitudes, three percentiles, two edge-weight modes and the small-region fallback. Six invalid requests reject in both engines. Every policy and initialization field is tamper-tested; removing tuned evidence rejects. Zero proposals and small-region results match tuned Single initialization; explicit cut/one-trial assignments match legacy behavior. Three RI/IA/NC tuned projects reopen without generation. These fixtures have no balance misses or disconnected results. Fourteen native ensemble tests, the 304-case legacy ensemble matrix, recursive/n-way tuning regressions, laboratory/project integration, audit handoff and all configured nested practitioner build gates passed. Evidence: target/ensemble-tuning-native-wasm-parity.json and target/ensemble-volume-*.bisect. The tuned ensemble test is now a laboratory build gate.

Actual browser RI volume/three-trial initialization with twenty proposals per split and ten refinement iterations produced two connected districts on 250 tracts: 0.132% deviation, about 50.3 km graph boundary and one split county. Reload/Restore autosave retained controls, evidence and results without generation. Screenshot: target/wasm-ensemble-controls-browser.png. Preview: dist/wasm-ensemble-controls/, including toolkit/. Remaining native engine options, practitioner/TUI workflows, unified national archives and conclusive browser disk-save proof remain unfinished. The entire-capability WASM objective remains active.


### Simple exposure and exact seed boundary (2026-10-04)

User steering: expose a simple starting workflow and build up, while establishing the technical boundaries and ability to load data/projects. This changes product exposure, not the scope of the underlying entire-capability WASM goal. The WASM laboratory now defaults to standard recursive bisection, geographic weights and Single search. Scope, state/year/chamber, district override, Run, map metrics and Save/Open remain directly accessible. Algorithm dimensions and tuning sit in a collapsed Experiment options disclosure; practitioner handoff/workbench links sit under Practitioner tools. Technical result evidence is under Run details. Opening a project/history expands its experiment settings so recorded choices are visible; closing a disclosure preserves them. Native/precomputed pages keep their existing presentation.

Browser proof: the simple RI baseline runs with options closed and produces two connected districts on 250 tracts, approximately 0.002% population deviation, 152 km graph boundary and three split counties. Reload/autosave restoration works without generation. A local target/u64-RI-single.bisect was selected through the actual browser file picker, passed graph/assignment validation and loaded its map/results without generation. Its exact seed 18446744073709551615 was verified in the visible input; hiding options retained settings. Screenshots: target/wasm-simple-browser.png and target/wasm-simple-open-project-browser.png. Preview: dist/wasm-simple/, including toolkit/. Disk-save completion remains separately unconfirmed in the in-app browser; file-format roundtrips and inbound Open are verified.

The engine boundary now accepts canonical decimal u64 strings alongside native integer JSON seeds. Output/options/root evidence preserve existing safe numeric seeds and serialize larger values as decimal strings. Browser config/input normalizes safe seeds to numbers and preserves larger seeds exactly as strings. It rejects unsafe JS numbers, negative zero, malformed decimal strings and overflow before serialization; native integer requests still support the exact u64 range. Practitioner plan provenance retains the exact seed too. Generic engine (21), lab/project integration and all configured practitioner build gates passed.

Seed verification: target/u64-seed-verification.json records 51 exact native/WASM results against raw native integer JSON, four matching algorithm failures, 19 invalid browser values, five existing BFS parity discrepancies, and three max-u64 project/practitioner-export roundtrips (Single, percentile and convergence). Native numeric/string BFS transport agrees, but BFS uses architecture-dependent SmallRng and yields different native/WASM assignments. This is an explicit remaining algorithm portability defect, not a seed rounding claim or a passed parity test. The transport gate records these gaps rather than claiming all methods match. No change to BFS randomness is made in this phase. The full engine/toolkit/TUI objective remains active and incomplete; future work should keep new exposure incremental.


### Visible run seeds and portable BFS (2026-10-04)

Run seed is visible above the collapsed Experiment options panel. Algorithms that do not use the supplied seed hide this field. The results heading uses the recorded job configuration, so editing the next-run seed does not rewrite the displayed result's provenance. Convergence results additionally show the selected seed when stopping evidence supplies one. Save/Open retains the existing exact-u64 representation. Browser verification restored a max-u64 project, showed its full seed in both input and result heading, hid the input for spectral smoothing, restored it for standard bisection, and verified that changing the input to 42 left the displayed recorded seed unchanged. Screenshot: target/wasm-visible-run-seed.png. The simple preview remains dist/wasm-simple/.

BFS initialization now uses an explicitly fixed xoshiro256++ stream with rand 0.8's default u64 seed expansion, preserving the existing 64-bit native SmallRng stream rather than choosing the architecture-dependent SmallRng implementation on wasm32. A native compatibility test compares u64/u32 draws, byte fills and the zero-state fallback. Twelve BFS-related tests passed, with the existing NC benchmark ignored. The exact-seed gate now requires whole-result native/WASM equality for BFS rather than recording allowed discrepancies: 56 matches, zero BFS gaps, four matching algorithm failures, nineteen invalid browser seed values rejected and three project/practitioner-export roundtrips.

A new real-state BFS build gate checks six seed/weight combinations on each available graph. RI/IA/NC yielded eighteen exact native/WASM results and three max-u64 project roundtrips; opening executes no engine. All eighteen plans miss the requested 0.5% population tolerance, while all are connected. Reproducibility does not repair this native BFS balance limitation. Evidence: target/bfs-native-wasm-parity.json. Existing saved BFS assignments remain loadable without regeneration; newly generated wasm32 BFS plans intentionally follow the preserved native64 stream. Generic WASM engine (21 cases) and laboratory project validation regressions passed. Remaining engine options and practitioner/TUI coverage keep the full WASM goal incomplete.


### GeoSection METIS refinement controls (2026-10-04)

GeoSection Single/Multi now accepts cut/volume refinement and one through one hundred internal METIS trials at every recursive ratio search and the two-seat shortcut. The new native tuned entry point shares the implementation with WASM; legacy entry points retain cut/one-trial defaults. The C FFI entry point explicitly rejects advanced values. Original-weighted-cut selection within each ratio and cut/sqrt(min-child-seats) ratio ranking remain unchanged, including strict first-minimum ties and wrapping consecutive u64 seed schedules. This is local cut selection, without child-plan lookahead. The native partial-assignment fallback remains legacy standard cut/one-trial, and the policy ledger identifies that exception. Intermediate output for tuned two-seat calls is retained.

The controls remain inside collapsed Experiment options. Recorded settings, scope and selection policy are checked on Open and included in practitioner plan engine provenance. Missing or altered tuning evidence rejects; legacy untuned projects retain their old format. AreaSection and VRASection retain their browser defaults and do not yet expose these tuning controls, although their shared recursive native routine has been refactored through the tuned implementation.

Verification: seventeen pure-Rust n-way/ratio tests passed, including an independent root-candidate replay across two/three/four/six seats, cut/volume and one/three trials. The GeoSection matrix passed 192 whole-result native/WASM comparisons, five matching invalid-request rejections, 2,496 evidence tamper/missing-ledger rejections, six RI/IA/NC project roundtrips and six practitioner exports retaining tuning. No balance misses or disconnected plans occurred in that matrix. Evidence: target/geosection-tuning-verification.json. The gate is included in build_wasm_lab.ps1. Shared-path regressions passed: AreaSection initialization sixteen cases, VRASection 576 cases (nine existing balance misses), standard recursive tuning ninety-six cases, generic engine twenty-one cases, laboratory project security and laboratory integration. The existing NC n-way 25.638% deviation remains reported.

Actual browser RI Single with volume/three trials and seed 42 produced two connected districts on 250 tracts, 0.229% population deviation, approximately 67.7 km boundary and one split county. Reload/autosave restoration preserved tuning and results without generation; the options panel was collapsed again. Screenshot: target/wasm-geosection-controls-browser.png. Source JS, module and integrity hash are updated in dist/wasm-simple/; its toolkit module is updated too. The full engine/practitioner/TUI goal remains incomplete, with remaining controls, input dimensions, exact-solver paths, analysis/library/archive/offline equivalents and broad data coverage still outstanding.


### AreaSection and VRASection refinement controls (2026-10-04)

AreaSection and VRASection now accept cut/volume refinement and one through one hundred internal trials together with their existing area initializer or demographic settings. Browser controls remain in collapsed Experiment options. Configuration validation now composes the permitted tuning fields with method-specific fields instead of selecting only one group. Engine requests retain both groups, and practitioner engine provenance carries the controls.

Tuned AreaSection initialization and VRASection scoring evidence include a metis_refinement ledger. It records objective, trials, iterations, recursive scope, internal/candidate selection, seed schedule, seeds per ratio, population rebalance and the existing legacy partial-assignment fallback. Open requires and checks this ledger when tuning is requested. Area initialization without an explicit initializer records the existing ratio-optimal default; old untuned evidence formats remain readable. The area root constraints and Lorenz feasibility mask, VRA root mass bonus, original-weighted-cut-per-ratio selection and descendant population-only ratio scans are unchanged. Tuning does not imply global optimality, legal certification or minority preservation.

Verification: target/ratio-constraint-tuning-verification.json records 288 exact whole-result native/WASM comparisons, zero matching generation failures, 3,168 altered/missing refinement-ledger rejections, six RI/IA/NC project roundtrips and six practitioner exports, with zero balance misses or disconnected plans in this matrix. Cases cover two/four/six seats, Single/Multi, cut/volume, one/three trials, seed 42/max-u64, ratio/moving-knife/default area initialization and VRA weights zero/0.4/one. Six invalid objective/trial requests reject in both engines; explicit cut/one-trial assignments match legacy inputs and one-district outputs claim no execution evidence. The new gate is included in build_wasm_lab.ps1. Legacy AreaSection initialization (sixteen) and VRASection (576) matrices passed; the latter retains nine native balance misses. Generic engine (twenty-one), laboratory project-security and integration checks passed, retaining the known NC n-way imbalance report.

Actual browser RI AreaSection Multi with volume/three trials, eight seeds and seed 42 produced two connected districts with 0.079% deviation; the root's left half has 53.220% of land, satisfying the 1.1 area swing. Reload/autosave restoration retained initializer and tuning. Browser VRASection loaded a local synthetic demographic JSON through the file picker, ran the combined configuration and produced two connected districts with 0.030% deviation and about 64.4 km boundary. Restoration retained demographics, alignment weight forty percent and volume/three-trial controls. Source labels explicitly identify the input as synthetic. Screenshots: target/wasm-area-refinement-browser.png and target/wasm-vra-refinement-browser.png. The simple preview's JS/module/hash and toolkit module are updated. The full engine, practitioner/TUI, input-dimension, archive/offline and data-coverage goal remains active and incomplete.


### Partisan edge weighting and local shares (2026-10-04)

WASM now calls the shared native bisect-core partisan weight builder for standard bisection with Single/Multi search. The optional controls remain inside Experiment options: strong Democratic/Republican thresholds and local complete-share JSON files. Each file uses schema_version bisect-partisan-shares-v1, state, Census year, source_label and dem_shares keyed by eleven-digit tract GEOIDs. Inputs must cover every graph tract; wrong scope, missing/extra tracts, invalid shares and negative zero reject before serialization. Imports use the existing bounded JSON reader and commit atomically after validation.

The native policy starts with unit edge weights. An edge between two strong Democratic or two strong Republican tracts receives alpha=max(3,10*(1-0.7*f)), where f is the fraction of strong tracts. Physical graph lengths remain available separately. Results record thresholds, strong/total tract counts, alpha, policy and a SHA-256 identity over scope plus sorted GEOIDs and little-endian f64 shares. Open checks both the weighting evidence and independently recomputed weighted metrics. Projects retain the complete input; practitioner exports include its identity in the hashed context and the input in producer provenance. Generation provenance remains unsigned. This edge weighting does not establish proportional seats or certify election outcomes.

Verification: target/partisan-wasm-verification.json records 360 whole-result native/WASM comparisons and independent manually weighted graph replays, 3,960 altered/missing evidence rejections, fifteen invalid input/option rejections and six RI/IA/NC project/export roundtrips. The matrix covers uniform, stripe, boundary, zero and mixed strong/swing shares; two/three/four seats; Single/Multi; cut/volume; one/three internal trials; seed 42/max-u64. No generation failures, balance misses or disconnected plans occurred. The gate is wired into build_wasm_lab.ps1. Existing release engine twenty-one cases, native practitioner export seven parity cases/ten rejections, laboratory project security/integration and Area/VRA tuning 288 cases all passed. The generic smoke test defaults to an older debug module; its initial failure disappeared when run against the rebuilt release module. The known NC n-way imbalance remains reported by integration.

Actual browser verification loaded explicitly synthetic shares for all 250 RI tracts through the local picker, ran Single/cut/one trial with seed 42, and produced two connected districts with 0.066% deviation, approximately 78.9 km physical boundary and one split county. Reload/autosave restoration preserved shares, thresholds, seed and results without generation; the threshold display is 55 rather than a floating-point artifact. Experiment options are collapsed again, and Run details displays the policy/source. Screenshot: target/wasm-partisan-browser.png. Save triggered the application's download-requested status, but browser automation returned no download file path; arrival on local disk remains unverified. The simple preview and toolkit module are updated.

Remaining scope includes partisan weighting in other native methods/searches, strict vote/count and TSV/CSV inputs, proportional recursive/ProportionalSection algorithms, other input dimensions, multiscale preparation, solver alternatives, complete practitioner/TUI/archive/offline equivalents and all-state data coverage. The full goal remains active and incomplete.


### Proportional native correctness prerequisites (2026-10-04)

Before exposing proportional algorithms in WASM, native inspection found a mismatch in run_all_splits_proportional: it chose population target ratios from local Democratic votes/census population, but followed a fixed floor/ceil BisectionTree afterward. A 3:1 root population split could therefore be subdivided as two seats on each side. The function now carries each chosen child seat count through a deterministic frontier until exactly k one-seat leaves remain. Every intermediate snapshot includes completed leaves and covers the entire graph. Sorted global vertex indices define floating-point vote accumulation order. Input dimensions, finite nonnegative votes bounded by census population, positive population totals without i64 overflow, symmetric valid adjacency and option bounds are validated before partitioning. A child without enough tracts for its seats returns an error. This deliberately corrects native behavior; asymmetric results are not expected to match older erroneous runs.

ProportionalSection now validates population/election dimensions, finite nonnegative counts with Democratic votes no greater than two-party counts, positive finite totals, adjacency, budgets and eta. Its geometric-mean quota rounding is factored into proportional_section_seat_counts. Both children receive at least one seat, preventing an extreme share from creating k:0 recursion. The rule rounds the Democratic quota and assigns the complement; it is described precisely as HH-style quota rounding rather than claimed to certify partisan outcomes. Statewide share remains clamped to 0.01..0.99; one-seat output records that share instead of a constant 0.5. Selected-root vote accumulation is sorted. Fixed seeds 1..budget, root population/vote constraints and population-only GeoSection descendant recursion remain the existing method.

Six pure-Rust tests pass: balanced connected four-district generation after a 3:1 root, repeated seeded assignments, complete depth-zero through depth-three snapshots, malformed input rejection, geometric-mean boundary/extreme-share rounding, and repeatable actual ProportionalSection root/descendant generation. Seventeen n-way/Geo/Area/VRA tests also pass. Command: cargo test -p bisect-runner --lib --no-default-features proportional --locked --offline. The proportional native test gate is added to build_wasm_lab.ps1. cargo check -p bisect-wasm --target wasm32-unknown-unknown --locked --offline passed. This compilation check does not establish native/WASM runtime parity for proportional paths: typed election-count imports, ABI routing, browser controls, project/evidence validation and practitioner export are still pending. The current preview module/UI are unchanged in this prerequisite phase. The full goal remains active and incomplete.


### Proportional methods, election input and browser projects (2026-10-04)

Both native proportional methods are now wired into the WASM ABI and the laboratory under Experiment options. Proportional bisection uses Democratic votes divided by census population at every node, rounds and clamps the child seat counts, and follows the chosen counts recursively. Its requested exact-u64 seed applies at every node. ProportionalSection uses statewide Democratic/two-party share clamped to 0.01..0.99, geometric-mean Democratic quota rounding with positive child counts, a population/vote root constraint, and population-only GeoSection descendants. Single/Multi searches use fixed seeds 1 through the budget, with up to fifty seeds per descendant ratio; the editable seed is hidden for that method and the recorded fixed schedule is shown in the map heading. Its root vote multiplier is configurable from one through two. Current paths use the native default cut refinement; advanced METIS objective/trials and partisan edge weights are not yet enabled for these methods.

Election files use bisect-election-counts-v1 with state, Census year, election_year, source_label and counts. Every eleven-digit graph GEOID requires democratic and two_party finite nonnegative counts, Democratic counts no greater than two-party counts, and complete exact graph coverage. Fractional counts are supported as explicit f64 values. Counts and total two-party mass are bounded by the browser's safe numeric range; negative zero, invalid shapes, unused inputs and wrong scopes reject. Recursive Democratic votes cannot exceed census population. Section requires positive Democratic mass; both paths require positive two-party mass. Imports use the bounded JSON Worker reader and atomic revision/sequence guards. Counts are stored separately from partisan shares and demographic fractions. CSV/TSV election input and conversion remain pending.

Operational SHA-256 identities include state, Census/election years, sorted GEOIDs and little-endian Democratic/two-party f64 values. Results retain input totals/identity even when one district requires no split. Recursive evidence includes every chosen split, its vote/population ratio, child counts and final district groups. Project opening independently checks complete tree coverage, seat prescriptions, canonical sums, the input identity and district metrics without running the engine. Section evidence records quota, targets, seed policy, root cut, supplied vote concentration and separate root population/vote constraint checks. These checks use actual census population and supplied counts, distinct from rounded/clamped internal METIS vertex weights. A completed plan can pass the user's overall tolerance while missing the tighter requested root multiplier. The UI explicitly reports these root misses in Run details. Practitioner exports preserve full election inputs in producer provenance and their identity in the hashed native context. Provenance remains unsigned and does not certify generation or partisan election outcomes.

Verification: target/proportional-wasm-verification.json records 270 exact whole-result native/WASM comparisons, 180 matching invalid/infeasible generation failures, 4,590 altered evidence rejections, fifteen additional invalid input/option rejections, three exact native/WASM practitioner exports and nine RI/IA/NC native comparisons plus project/export roundtrips. The matrix covers uniform/asymmetric/stripe/fractional/zero Democratic counts, one/two/three/four/six seats, both methods, Single/Multi where supported, geographic/unweighted/county weights, seed 42/max-u64 and vote multipliers 1.1/1.5. Four generated synthetic cases miss the requested final balance tolerance; no generated case is disconnected. Real-state comparisons use explicitly synthetic census-scaled election counts and ten percent tolerance; deviations and root checks are retained in the report rather than claimed as election results. All nine state cases generated successfully and their assignments/metrics matched native execution.

The proportional gate runs with project fixtures after static export in build_wasm_lab.ps1, alongside the six native proportional prerequisite tests. Regression checks passed: six native proportional tests; generic release engine twenty-one cases; exact-u64 seeds fifty-six matches/four matching failures/nineteen rejected inputs/three project exports; existing practitioner export seven matches/ten rejections; laboratory project security and integration; partisan weighting 360 matches/3,960 evidence rejections/six project exports. Static exporter cargo check passed. The known NC n-way imbalance remains reported. Source/export copy lists include the new election module, and the simple preview's module/hash/JS plus toolkit module are updated.

Actual browser RI Section Single imported local explicitly synthetic 2016 counts aligned with 2020 Census tracts. It produced two connected districts, 0.152% overall deviation, about 127.2 km boundary and three split counties. The requested root population multiplier is missed; the supplied-vote multiplier is met, and both checks are visible. Reload/autosave restoration retained counts, vote multiplier 1.1, fixed Seed 1 and results without generation. Recursive proportional bisection with max-u64 seed produced two connected districts, 0.049% deviation, about 150.9 km boundary and two split counties. Restoration retained the exact seed, source counts and results. Experiment options are collapsed at the end. Screenshots: target/wasm-proportional-section-browser.png and target/wasm-proportional-recursive-browser.png.

The full engine/practitioner/TUI goal remains active and incomplete. Outstanding work includes additional weighting/control combinations, strict election CSV/TSV and other input dimensions, multiscale preparation, exact-solver alternatives, analysis/library/archive/offline equivalents, broad state/year/chamber data coverage and conclusive browser download-to-disk verification.

### Election count CSV import (2026-10-04)

The proportional experiment controls now accept the native tract CSV columns `geoid,dem_votes,rep_votes` (also `GEOID`). Select the CSV state and election year explicitly; the selected Census year supplies the tract vintage. JSON retains its embedded scope. Import one state at a time for national experiments. Short numeric GEOIDs are padded to eleven digits, while duplicate aliases, duplicate tracts, foreign-state rows, missing/invalid counts, negative zero, nonfinite values, unsafe totals and invalid UTF-8 are rejected. No rows are silently filtered and no counts are filled. Imports are capped at 8 MiB and 100,000 tracts; graph coverage is checked before execution.

`import-election-counts-csv` uses the same Rust parser in native and WASM. Browser parsing runs in a dedicated integrity-checked worker with cancellation/timeout and atomic input replacement. Imported Democratic and computed two-party counts, election year and filename source label are retained in projects; the existing operational count digest verifies subsequent results. This is a derived count identity, not a raw CSV-byte digest or certification of the supplied election data.

Verification: release native and WASM builds passed; `test_election_csv.mjs` passed 13 exact comparisons, 31 malformed/tamper rejections (including the byte and row limits) and six RI/IA/NC generation/Save/Open/map cases. `test_election_csv_worker.mjs` exercised actual WASM in a worker, integrity mismatch, invalid UTF-8/state/duplicates, pre-read size rejection, cancellation and termination. Project regression checks passed; Open performs no generation. Both gates are in `build_wasm_lab.ps1`. Actual browser file-picker import of the explicitly synthetic RI CSV produced two connected districts with seed 42, 0.002% population deviation, about 152 km boundary and three county splits. Reload/autosave restoration retained counts, source label, election year, assignments and the visible seed without rerunning the engine. CSV state/year controls are restored from saved inputs.

Remaining input work includes partisan share TSV, other input dimensions and raw-source provenance where required. The broader full WASM capability goal remains incomplete.
The updated release module also passed the proportional regression: 270 exact native/WASM results, 180 matching failures, 4,590 tamper rejections, 15 invalid requests, three matching practitioner exports and nine real-state project cases. Four synthetic cases still miss balance and are reported as such; none were disconnected. Browser verification screenshot: `target/wasm-election-csv-browser.png`. New phase changes remain local after the preceding WASM snapshot was merged as PR #53.
### Partisan-share TSV import (2026-10-04)

The partisan boundary-weighting controls now accept native `geoid<TAB>dem_share` files, with optional recognized headers, blank lines, # comment lines, UTF-8 BOM and short numeric GEOID padding. TSV state is explicit and the selected Census year supplies scope. Imports reject duplicate tracts, foreign-state rows, unknown headers, missing/extra columns, nonfinite/out-of-range values and negative zero. They are capped at 8 MiB and 100,000 tracts. Unlike the historical lenient CLI loader, this boundary never fills missing tracts with swing shares; complete graph coverage is required before generation. JSON remains supported.

The new `import-partisan-shares-tsv` operation uses a shared Rust parser in native and WASM; CSV and TSV share state/year/label/GEOID validation. A dedicated worker verifies the module digest before parsing, supports cancellation/timeout, and the host retains atomic replacement guards. Projects retain operational shares, source filename label, state/year, thresholds and run provenance. The operational share hash does not certify supplied election data or identify the original raw TSV bytes.

Verification: native/WASM release builds and static export passed; the TSV gate passed 13 exact comparisons, 27 malformed/tamper rejections and six RI/IA/NC Single/Multi run/Save/Open/map cases. Actual worker tests covered integrity/UTF-8/scope/duplicate rejection, pre-read size/cancellation and termination. CSV regression passed 13 exact comparisons, 31 rejections, six project cases and worker checks after sharing input-scope code. The existing partisan gate passed 360 exact comparisons and weighted replays, 3,960 evidence rejections, 15 invalid requests and six project/export cases; no balance misses or disconnected plans in that test matrix. New gates are included in `build_wasm_lab.ps1`.

Actual browser RI TSV file-picker import used explicitly synthetic shares on 250 Census tracts. The seed-42 plan had two connected districts, 0.036% deviation, approximately 125 km physical boundary and two county splits. Reload/autosave restoration retained the shares, filename, 55%/45% thresholds, selected TSV state, seed and map without generation. Screenshot: `target/wasm-partisan-tsv-browser.png`; preview: `dist/wasm-partisan-tsv`. The simple view is left with Experiment options collapsed.

The full WASM capability goal remains active and incomplete. Outstanding work includes further engine weighting/search/control combinations, multiscale input preparation, exact solvers, additional analysis/library/archive/offline capabilities, broader data coverage, and conclusive browser download arrival on disk. CSV/TSV phase changes remain local after PR #53.
### Partisan weighting with percentile, convergence and local ensemble (2026-10-04)

Standard bisection now permits partisan adaptive unit weights with Single, Multi, percentile, convergence and local bisection ensemble in the WASM request boundary, project validator and UI. Explicit complete shares and ordered thresholds remain required. Other structures/searches still reject unsupported combinations. METIS cut/volume and internal-trial controls remain available on the new paths. The existing native runner algorithms are used unchanged.

Scoring scope is preserved: percentile uses partisan weights in seeded construction but ranks full plans by unweighted edge cuts; local ensemble uses partisan weights in METIS initialization, then unweighted Wilson-tree sampling and percentile ranking. Convergence ranks the native recursively normalized weighted cut. The UI explains these distinctions; weighting does not imply certified partisan outcomes.

Verification: release native/WASM builds and static export passed. Expanded `test_partisan_wasm.mjs` passed 900 exact native/WASM cases and independent weighted-graph replays, 9,900 weighting-evidence rejections, 15 invalid requests and 15 RI/IA/NC project/export roundtrips, with no matching generation failures, balance misses or disconnected plans in its synthetic matrix. All 15 saved real-state assignments were independently replayed in native and WASM, with identical full results; this comparison is now part of the permanent gate. `test_partisan_search_edges.mjs` passed 12 additional native/WASM and weighted-replay cases with default METIS options, one-district inputs, percentile endpoints and zero local proposals. The new edge gate is wired into `build_wasm_lab.ps1`. Project and TSV-worker regressions passed.

Actual browser RI used explicitly synthetic TSV shares on 250 Census tracts, max-u64 seed, volume refinement and three internal trials. Percentile with four seeds and 100th percentile produced two connected districts, 0.220% deviation, 104.2 km physical boundary and two county splits. Convergence with a two-seed non-improvement tail and five-seed cap stopped after three seeds at the threshold, with 0.190% deviation, 100.3 km boundary and two county splits. Local ensemble with five proposals and 100th percentile produced two connected districts, 0.150% deviation, 115.4 km boundary and two county splits. Reload/Restore autosave preserved shares, thresholds, max-u64 seed, local-ensemble controls, refinement options and assignments without generation. Screenshots: `target/wasm-partisan-convergence-browser.png` and `target/wasm-partisan-search-browser.png`. Preview: `dist/wasm-partisan-search`. Experiment options are left collapsed.

The full WASM goal remains active and incomplete. Remaining work includes further structures/weighting/search combinations, multiscale preparation, exact solvers, analysis/library/archive/offline equivalents, broader state/year/chamber data coverage, and verified download arrival on disk. This phase and the CSV/TSV phases were subsequently merged in PR #54.

### Economic and housing character kernels and import boundary (2026-10-04)

The native economic and housing derivation, cosine-similarity and geographic-blend kernels now live in bisect-core. Native CLI loaders re-export the same types/functions and both native edge weighters call the shared blend. Economic job intensity preserves the existing C000 / 10,000 cap at one; it is not a resident-population ratio. Housing vintage retains the native 1940–2020 reference range. Economic cosine uses both-zero=1 and one-zero=0.5; housing uses any-zero=1.

The WASM toolkit ABI adds import-character-csv and build-character-weights. Imports support native LODES WAC count headers or native ACS housing derived-value headers (including pct_sf/pct_mf aliases), with case-insensitive unique headers, UTF-8/BOM, reordered/extra columns, quoted fields, strict nonnegative count/fraction ranges, numeric GEOID padding and state scope, duplicate rejection, an 8 MiB limit and 100,000-tract limit. Census geometry year and observation data_year are separate required metadata. Economic inputs retain raw counts and verify their derived vectors; housing imports explicitly contain derived fractions/proxies and do not claim raw ACS count provenance.

Weight evaluation requires complete exact canonical tract coverage, unique canonical positive boundary edges, finite alpha in [0,1], and valid retained inputs. It returns native blended edge weights (possibly zero at alpha=0) and an operational SHA-256 over kind, scope, observation year and ordered derived values. Source descriptions do not change operational identity; economic raw observations remain retained and must reproduce their vectors. This hash describes operational character values rather than raw source bytes.

Verification: four shared-core formula/boundary tests, 45 existing native CLI LODES/housing/edge-weight regressions, and 26 exact compiled native/WASM operations passed. The operation gate includes ten analytical/non-axis blend cases, an independent little-endian digest oracle, alpha endpoints and the distinct zero-vector policies. Eighty-eight matching malformed-input, limit, coverage, edge, unknown-field and altered-vector rejections passed. Evidence: target/character-csv-verification.json; script: scripts/web/test_character_csv.mjs. The laboratory build includes the new gates.

This is a shared-kernel/input prerequisite. Economic/housing weight modes are not yet routed through laboratory generation, controls, Save/Open, evidence verification or practitioner export; no new browser-selectable option is claimed. Character input file workers and full UI/project/engine integration are the next phase. Zone spatial joins, the remaining engine/practitioner/TUI inventory and broad state/year/chamber coverage also remain unfinished. The full WASM objective remains active.

### Economic and housing laboratory integration (2026-10-04)

The laboratory now exposes economic and housing boundary weights with explicit character CSV/JSON inputs and geographic blend alpha. Inputs require complete selected-state tract coverage and retain Census scope, observation year and source labels. Integrity-checked workers parse CSV with cancellation and atomic replacement. State and national experiments retain character inputs, weighting evidence and exact run seeds in portable projects; Open verifies identities and metrics without generating a new plan. Practitioner export retains the character input and operational digest. Search descriptions explain when native algorithms use weighted construction, unweighted sampling or unweighted ranking.

Verification passed 564 exact native/WASM cases and independent weighted-graph replays across the 17 supported structures, 6,204 evidence tamper rejections, 24 invalid requests, 66 matching generation failures, and 30 RI/IA/NC project/map/native replay/practitioner export cases. Forty-two successful synthetic cases missed population tolerance; none were disconnected. Two three-state national project roundtrips restored six maps without generation and rejected missing, foreign-scope and altered-year inputs. CSV operation and real-worker gates passed. Economic browser import/run on explicitly synthetic RI inputs produced two connected districts with 0.039% population deviation and retained the visible max-u64 seed. Housing browser import and disk download arrival are not claimed by these checks.

The matrix exposed platform differences in SMC exponential/logarithmic calculations. Both runtimes now use libm exp/log and record math identity in SMC evidence; the SMC regression passed 49 successes and nine matching failures. VRASection passed 576 exact comparisons; proportional passed 270 exact results and 180 matching failures. Native web tests passed all 11 checks, including the eager JavaScript module dependency closure for static export. Native/precomputed hosting now includes the input-validation modules required by the laboratory.

The full WASM capability objective remains incomplete: exact solvers, multiscale preparation, further practitioner/TUI/archive/offline capabilities, broad state/year/chamber data coverage and verified browser disk downloads remain outstanding. Native algorithm balance failures are reported rather than represented as certified plans.
### Portable multiscale engine boundary (2026-10-04)

The WASM toolkit now accepts run-multiscale with explicit initialization request, fine/coarse resolutions, steps, alpha, percentile and optional adaptation parameters. All three native pairings are represented: tract to county, block group to tract, and block group to county. Block-group input requires complete numeric unique GEOIDs, matching state/Census year, symmetric connected adjacency, connected units within each parent tract, exact tract population sums and an adjacency quotient matching the tract graph. Missing fine graphs are rejected rather than inferred. Retained candidate assignments are capped at ten million unit-plan cells because the native percentile algorithm stores every candidate.

Shared runner implementations now have separate portable entry points using ChaCha12 and fixed-width portable Wilson/ReCom draws. Historical native entry points retain SmallRng and their existing draw policy. Native/WASM parity below compares the versioned portable path in both runtimes, not historical native SmallRng assignments. Native initialization remains geographic weighted standard METIS; this boundary rejects unused character, election, demographic and partisan inputs and unsupported refinement overrides. Sampling and percentile ranking remain unweighted. Evidence records the seed, graph identity, resolution, native last-fine-index coarse assignment policy and adaptation counters; native fine acceptance counters count every fine attempt and do not prove proposal acceptance.

Verification: all 15 existing native multiscale tests passed. The compiled release native and WASM operation gate passed 78 identical deterministic results covering all three pairings, fixed/adaptive search, alpha and percentile endpoints, zero steps, and max-u64 seed. Independent checks recomputed populations, connectivity, deviation and zero-step seed projection. Thirty-two malformed/scope/coverage/adjacency/adaptation/work-limit requests were rejected in both runtimes; JavaScript also rejected unsafe numeric seeds and negative zero before JSON conversion. Six returned synthetic plans missed tolerance; none were disconnected. Character CSV regression remained green at 26 exact operations and 88 rejections. Evidence: target/multiscale-verification.json; gate: scripts/web/test_multiscale_wasm.mjs, included in both WASM build scripts.

This engine boundary remains local and does not yet expose multiscale laboratory controls, block-group geometry preparation/import UI, portable project Save/Open or practitioner export. Those integrations and real-state verification are next; the full WASM goal remains incomplete.
### Multiscale laboratory integration evidence (2026-10-04)

Fixed and adaptive multiscale search are exposed in state/national laboratory controls for tract-to-county, block-group-to-tract and block-group-to-county. Geographic standard bisection initializes these paths; unsupported weighting and METIS overrides are rejected. Exact seeds, resolution and adaptation parameters are retained. Strict worker imports bind block-group graphs, physical boundaries and geometry to complete tract parent scope. Canonical fine-bundle hashes and a little-endian operational graph codec support independent input verification.

The permanent gates passed 78 exact native/WASM cases, 32 matching invalid requests, 25 project roundtrips, 37 maps, 25 native replays, 177 tamper rejections and 11 terminated input workers. Six real RI/IA/NC tract cases supplement synthetic state/national resolution fixtures. Nine project-matrix balance misses are reported; none were disconnected. Open verifies retained assignments and metrics without generation and does not certify recorded execution provenance. Actual RI browser runs and local-file Open passed; block-group browser data was explicitly synthetic. Disk arrival of browser downloads is unverified. Multiscale practitioner export, authentic fine-input preparation, broader weighting and the remaining capability inventory are still outstanding.
### Multiscale practitioner export evidence (2026-10-04)

`export-multiscale-plan` validates the same initialization/resolution boundary as generation without executing the engine. It exports explicit tract/block-group units, supplied assignments converted to native zero-based IDs, fine population and adjacency, complete canonical physical boundary weights and county IDs. Native context hashes bind unit order and graph inputs; source hashes bind multiscale inputs and physical boundaries. Original edge classification and map geometry are not supplied by this bridge. Laboratory projects remain the full map/input backup. Exported plans still require an explicit audit profile.

All 37 plans from the 25 state/national project fixtures passed exact export/reader/audit/certificate comparison in native and WASM: 148 operations and 327 matching invalid requests. Nine balance misses correctly failed native population checks. Actual browser restored fine-project preparation and workbench file Open/audit/certificate verification passed on the explicitly synthetic RI 500-unit fixture. The 0.5% profile reports population failure with shape/contiguity success. Existing plan export and multiscale generation/project/worker regressions passed. Authentic block-group data preparation, broader weights, remaining capability inventory and verified browser download arrival are incomplete; this follow-up remains local after PR #56.

### Authentic fine-input preparation evidence (2026-10-04)

The native `wasm-fine-input` boundary now converts explicit local TIGER block-group geometry and PL block counts into the existing browser bundle. It validates numeric unique unit identity, geographic NAD83, full population coverage, connected parent regions, exact tract population sums and quotient adjacency, geographic rings and browser resource limits. It never infers missing counts or adjusts cross-tract edges to fit. Native island bridges require an explicit option, use median edge costs, and are listed separately from measured boundaries in a source report. The shared bridge helper now orders equal-size components deterministically. Native historical prepared files are unchanged; regeneration of formerly nondeterministic ties may select a different stable edge set.

Evidence: local RI 2020 Census inputs yielded 792 units and six declared bridge costs. Two independent preparations were byte-identical; seven source hashes were independently checked and seven malformed/overwrite/policy requests rejected. Fourteen TIGER and nineteen bridge tests passed. The authentic matrix passed 64 exact native/WASM operations, sixteen project/map roundtrips and sixteen population/contiguity audits, with no balance misses. Browser worker import and an adaptive ten-step block-group-to-tract run with max-u64 seed completed at 0.206% deviation, two graph-connected districts and one county split. The stabilized bundle matches the browser-tested bytes. Graph connectivity includes declared bridges; geographic connectivity and authenticated Census provenance are not asserted. Source reports remain separate backups rather than embedded project evidence. Broader authentic coverage and the remaining capability inventory remain incomplete.

### Two-plan WASM comparison (2026-10-04)

The practitioner workbench exposes **Compare two plans** with primary RPLAN, second RPLAN and optional matching RCTX. The shared native readers require identical ordered units, unit kind, state and Census year; reordered or cross-cycle plans need an explicit crosswalk and are rejected by this operation. The adapter reuses native directional mean-best district Jaccard and audit measurement kernels for actual population, graph components and county/municipal split witnesses. Jaccard is unweighted, directional and permits reuse of a matching district; allowed empty districts are omitted from similarity and unit-count proxies but remain visible in actual district measurements. Graph connectivity includes declared bridges. Partial subdivision coverage reports known and missing units.

Exact population counts are decimal strings, including values beyond JavaScript safe integers. Unit-count deviation is labelled separately from population deviation. Missing populations and zero total population leave relative deviation unavailable. Compactness and partisan measurements are explicitly not evaluated; projected district geometry and election observations are not supplied by this operation. Comparison is descriptive and does not establish a legal profile, audit certificate, provenance, replay or optimality. Work limits bound units, districts, graph edges and unit/district products.

Verification: 127 exact native/WASM comparisons, 126 portable project roundtrips and 14 matching invalid requests cover missing/partial/zero/large populations, empty/disconnected districts, identities and context hashes. Twelve further exact native/WASM export/comparison operations cover prepared RI/IA/NC 2020 plans with seed 42 and max-u64; population sums were independently checked with BigInt. Native audit tests (29) and comparison tests (13) passed, as did the existing 98-operation VRA audit and 148-operation multiscale export gates. The workbench security/UI gate checks busy input guards, operation requests, second-plan retention and unverified imported results. Gates: scripts/web/test_plan_comparison.mjs and scripts/web/test_plan_comparison_real.mjs.

Actual browser project Open, Restore autosave and worker recomputation passed for the explicitly synthetic nine-unit fixture. Screenshot: target/wasm-plan-comparison-browser.png. Comparison remains partial: native multi-plan reports/formats, cross-cycle comparisons, compactness, partisan measurements and remaining research modes are not ported. The full engine/practitioner/TUI/archive/offline and all-state/year/chamber objective remains active. These changes remain local after PR #57.

The comparison also reports label-sensitive changed-unit counts, exact population in changed units when populations exist, and sorted touched district IDs. The native report disk adapter and WASM operation now share the assignment-change kernel. Pure district renumbering counts as change; this is not a claim of population movement or a metric after matching districts. The native disk reader regression verifies common-unit intersection and touched-label behavior. Fourteen native comparison tests and 56 existing report comparison tests passed, plus the new disk-adapter regression. The 127-case WASM matrix independently checks changed counts, touched IDs and BigInt population sums in both directions. Browser verification above predates these added rows; the updated rows are covered by the permanent table/project gates.

### Portable native report reader (2026-10-04)

The shared report library now exposes `load_plan_side_from_files` over explicit relative names and original bytes. Native disk reads delegate to the same parser, preserving supported partisan/compactness JSON shapes, civic metadata, missing-analysis behavior and SHA-256 hashes of original files. The default `filesystem` feature retains the existing native report API; disabling it exposes the comparison data reader, narrative/HTML renderers and margin-of-error helpers without filesystem-only dependencies. The WASM crate uses this portable configuration.

`inspect-report-side` accepts manifest.json and the three named native analysis files with a supplied leaning threshold. File names, total bytes, fallback label length, district count and share-array size are bounded. Results include raw native report data, explicit supplied-analysis flags and an integrity-only boundary. Imported analysis is not recomputed or authenticated; zero defaults for absent counts and the native total-population placeholder are not measured values. Compactness missing-value NaN serializes as null. This is an inspection boundary, not a new visible workbench operation or a published report.

Evidence: all 362 native report tests and 76 tests with filesystem features disabled passed. Native disk/in-memory fixtures matched all three partisan shapes and original-byte hashes. Compiled native/WASM inspection matched thirteen cases and sixteen input rejections, including unknown/traversal names, malformed/invalid UTF-8 inputs, absent manifest, thresholds, district limits and invalid analysis fields. Gate: scripts/web/test_report_side_wasm.mjs. The existing comparison matrix remains a regression gate. Report import UI, strict completeness for rendered claims, manifests with reproducible receipts, multi-plan summaries and the remaining full WASM inventory are still unfinished. Changes remain local after PR #57.

### Report inspection workbench (2026-10-04)

The workbench exposes imported report inspection with a dedicated folder selection, literal fallback label and percentage threshold. Only the manifest and supported analysis files are read; unrelated folder entries are skipped without reading their bytes. Paths, duplicates, byte limits and settings are validated before adoption. Original bytes and settings survive portable Save/Open; Open does not run the engine and marks retained results unverified. The displayed table uses explicit usable-analysis flags, so missing or incomplete native defaults do not appear as measured counts. Total population remains explicitly unavailable in this reader. Inspection does not recompute analyses or publish narratives.

The project/UI gate passed exact requests, byte/settings roundtrips, ignored-file non-reading, traversal/root/duplicate/dishonest-size rejection, busy and stale imports, restored percentage fidelity and unverified Open. The native/WASM gate passed nineteen exact inspection cases, sixteen matching rejections and fourteen project/table roundtrips, including present-but-empty analysis and a partial district-share series. Actual browser Open and WASM-worker execution passed for complete and incomplete report projects: the percentage restored as 55, hostile HTML labels remained literal text, original-byte hashes were visible, and incomplete measurements remained unavailable. Save created `Incomplete-report-inspection.bisect` in Downloads; its report bytes, settings, operation and result matched the fixture exactly. Reopening that downloaded file retained the result as unverified without executing it. Browser evidence: `target/wasm-report-inspection-browser.png`. Actual browser folder-picker import then loaded four supported files from a native report folder containing an unrelated HTML file; WASM execution reproduced the complete measurements and original-byte hashes with no browser warnings/errors. Evidence: `target/wasm-report-folder-browser.png`. Ignored-file non-reading is independently covered by the project/UI gate. The broader report-rendering, receipts, native practitioner/research/TUI and full state/year/chamber capabilities remain active work.


### Imported comparison report renderer (2026-10-04)

The portable practitioner API adds `render-report-comparison`: two bounded native report-file maps, fallback labels, a leaning threshold, close-call band and an explicit supplied generation timestamp. The shared Rust narrative and HTML comparison renderers generate draft Markdown, self-contained HTML and a SHA-256 byte receipt. Each side uses the same native report reader as inspection. Original manifest and analysis hashes are retained. Available metrics use the native threshold/close-call and comparison logic; unavailable sides suppress directional claims. Population totals and assignment differences stay unavailable because report files do not supply these measurements. No approval, authenticated provenance, certification or legal compliance is inferred.

The report reader now treats impossible majority-minority counts, incomplete district compactness series and out-of-range Polsby-Popper as unavailable for display and narrative claims. Native raw values remain intact for inspection. Imported report metadata is capped at 200 UTF-8 bytes per field. HTML escapes labels/submitters; Markdown escapes imported metadata separately. Both-civic comparisons now select the same side for the native banner and narrative.

Verification: `test_report_comparison_wasm.mjs` passed 68 exact native/WASM cases and 16 matching rejections, all 64 analysis-availability combinations, partial/impossible inputs, escaped markup, output SHA-256 checks and rejection of attempted approval fields. The existing inspection gate passed 19 exact cases and 16 matching rejections. The new rendering gate is included in `build_wasm.ps1`, which the laboratory build invokes. The native report suite passed 363 tests, including the both-civic framing regression. Actual browser inspection of the generated HTML showed the draft badge, native numerical table, unavailable assignment/population inputs, consistent civic framing and literal hostile labels/submitters; no script, image or iframe elements were created. Screenshot: `target/wasm-imported-comparison-report-browser.png`. The workbench now exposes this operation, saved projects and verified report downloads; see the browser workflow below. Multi-plan/cross-cycle reports, assignment/population integration, native receipt/approval workflows and the broader WASM capability inventory remain active work.


### Comparison report workbench and local downloads (2026-10-04)

The fifteenth visible workbench operation, **Generate imported comparison report**, accepts native report folders A and B, fallback labels, the leaning threshold and close-call band. An optional supplied generation timestamp defaults to the current UTC time at Run and is retained for reproducible reruns. Only the four supported report files are read from each folder. Report-only controls hide unrelated practitioner inputs. Result tables retain explicit missing-measurement boundaries and inert metadata; HTML is never inserted into the workbench.

Projects retain both folders' original bytes, settings and generated result. Open executes nothing and treats the result as unverified. HTML, Markdown and receipt export controls require a fresh WASM run after Open; generic JSON remains an inert inspection export. Download checks both generated UTF-8 payloads against their SHA-256 receipt before creating a file. The new output helper is included in the toolkit build, native web input assets and static laboratory asset export because `project.js` imports it.

Verification: the project/UI gate passed exact requests, two-folder/settings retention, busy and stale reads, default timestamps, operation-specific panels, unverified Open, computed-only exports, byte limits, hash-tamper rejection and exact Blob payloads. The native/WASM rendering gate still passed 68 exact comparisons and 16 rejections, and now checks 64 project/table roundtrips plus three exact download payloads. Actual browser file-picker imports loaded both native report folders containing unrelated HTML, then generated a report in the WASM worker. HTML (6,618 bytes), Markdown (1,523 bytes), the byte receipt and a `.bisect` project arrived in Downloads. Their output bytes/hashes and saved source/settings/result matched the native/WASM fixture exactly. Reload/Open of that downloaded project retained its data without execution and disabled report exports until an explicit rerun reproduced the original hashes. Browser evidence: `target/wasm-report-generation-workbench-browser.png`; byte verification: `target/report-comparison-fixtures/browser-download-verification.json`.

The native static exporter also passed with an RI-only prepared catalog. Exported `project.js` and `report-output.js` matched source bytes and the catalog WASM hash matched the engine bytes. The static laboratory gate passed generation, saved maps, integrity-failure rejection and cancellation for that RI-only validation scope.

This completes the imported two-side report workflow, not the full native practitioner/report suite. Analysis recomputation, assignment/population integration, cross-cycle/multi-plan reports, native approval/provenance receipts and the wider capability inventory remain active work. These changes are local and have not been deployed to Cloudflare.


### Supplied-plan measurements in comparison reports (2026-10-04)

The imported two-report workflow now has an explicit **Include measurements from supplied plans** option. It adds plans A and B and an optional matching context to the report request. Both plans are required together; ordered units, unit kind, state/year, district counts and context identity are validated through the shared comparison adapter. Declared report state/year and district counts must agree with the supplied plans. Matching scope does not authenticate imported partisan, demographic or compactness analysis to those plans; that boundary remains visible in the UI, narrative and receipt.

The shared Rust renderers show label-sensitive changed-unit counts, sorted zero-based touched district IDs, exact population totals and population in changed units. Population values are decimal strings in portable JSON and stay exact above JavaScript's safe integer range. Missing population remains null/unavailable and measured zero remains zero. Pure district renumbering counts as change and does not imply population movement. The portable report's unused native diff placeholder is null, rather than a misleading zero-change result. Receipts retain canonical plan/context hashes and the separately computed measurements.

Verification: `test_report_plan_evidence_wasm.mjs` passed 15 exact native/WASM cases and project/table roundtrips, plus nine matching rejections. Independent oracles assert zero, one and nine changed units across identical, single-unit and renumbered assignments, with absent, graph-only, zero, ordinary and large population contexts. The fixture keeps its two plan objects independent. The native report suite passed 363 tests. Actual browser execution showed nine changed labels and the exact population 40532396646334464; report HTML/Markdown/receipt and the saved project matched the fixture (`target/report-plan-evidence-fixtures/browser-download-verification.json`).

Report exports now become unavailable after input edits, including invalid or empty settings. Renaming the project preserves export availability. The project/UI gate covers valid changes, empty thresholds, out-of-range bands and oversized timestamps. Actual browser restore retained an unverified result with exports disabled; rerunning enabled them, changing the threshold disabled them, and clearing it also disabled them despite validation failure. Restoring the valid setting and rerunning reproduced the original output hashes. Screenshot: `target/wasm-report-freshness-browser.png`.

This covers assignment/population integration for the supplied-plan imported two-report workflow. Native analysis recomputation, authenticated report-to-plan association, approval workflows, multi-plan/cross-cycle reports and the remaining engine/practitioner/TUI/offline/all-state capabilities remain incomplete. These changes remain local and are not yet deployed to Cloudflare.


### Bounded certified exact split/tree WASM workflow (2026-10-04)

The shared native `bisect-ilp` certified split and recursive tree algorithms are now exposed through four practitioner operations: `solve-certified-split`, `verify-certified-split`, `solve-certified-tree` and `verify-certified-tree`. The workbench offers file selection, bounded result tables, artifact download, explicit adoption for verification and project Save/Open. Inputs and full artifacts travel as JSON text, so individual i64 populations and u64 boundary/objective/proof counts are not rounded by JavaScript. Summary objectives use decimal strings. Open retains results as unverified, performs no solve/verification and disables artifact export until an explicit new run.

The browser boundary accepts native certified split instances with 2–24 units, 2–unit-count parent seats, canonical bounded edges, IDs and node paths; JSON text is capped at 3 MiB. Tree verification caps seats, nodes and leaves before invoking the native schedule. Solve/verification execute in the existing cancellable WASM worker. These are exhaustive algorithms and may be expensive near their bound. The optimal objective is lexicographic: maximum scaled population deviation, total scaled deviation, weighted boundary cut, then canonical assignment. Each split requires both children connected with enough units for their seats. Local optimality does not imply a globally optimal completed plan; the native tree can reject an infeasible descendant. Large-instance proof backends, canonical benchmark/ILP dispatch, certified single-district and ceremony controls are still incomplete.

`bisect-ilp` keeps filesystem adapters in its default native feature. The WASM dependency disables that feature, excluding disk model-artifact verification; the split/tree calls themselves use no filesystem or clock. Native default-feature tests passed 106 cases, and the no-default-features library passed 101. Compiled native/WASM parity passed 52 solve/verify operations, 19 independent exhaustive/BigInt optimal-cut oracles and 31 matching invalid-input, proof-tamper and tree-tamper rejections (`scripts/web/test_certified_exact_wasm.mjs`). The build script runs this gate after compiling the native reference. The project/UI gate passed original byte retention, exact requests/download bytes, busy and pre-read size guards, explicit artifact adoption and inert/unverified restore.

Actual browser file selection and WASM solve preserved the individual population 9007199254740993, selected assignment [0,1,0,1] with zero population deviation and boundary objective 7, and downloaded `certified-split-artifacts.json` with bytes identical to the compiled native/WASM fixture. Explicit adoption and a second Run reenumerated and verified the split proof with no browser warnings/errors. Evidence: `target/certified-exact-wasm/browser-download-verification.json` and `target/wasm-certified-exact-browser.png`. Tree runtime is covered by native/WASM gates; actual browser tree Save/Open and download arrival are not yet claimed. These local changes are not deployed. The full WASM goal remains active.


### Certified single-district WASM workflow (2026-10-04)

The workbench adds **Certify a single district** and **Verify single-district certificate**, using the native `build_certified_single_district` and `verify_certified_single_district` implementations. The instance uses ordered unit IDs, exact i64 populations, canonical positive-weight edges and a unit-universe hash. Its certificate contains every unit assigned to district zero, a checked population sum, zero boundary cut, connectivity, a native certificate identity and the trivial one-seat proof kind. No bisection or search is performed. This workflow accepts up to 100,000 units and 200,000 edges within the existing 3 MiB JSON-text boundary; split/tree enumeration remains capped at 24 units.

Original instance/artifact JSON text stays lossless through the WASM ABI, source-file base64 storage, project Save/Open and artifact download. The result table displays exact population strings and identifies verification as a trivial-certificate check rather than enumeration. Generated artifact adoption clears the old file-picker label, selects the corresponding single-district verifier and requires an explicit Run. Restored project results stay unverified and cannot be exported as freshly computed certificates.

Verification: the extended exact gate passed 24 single-district native/WASM operations, independent complete-assignment and BigInt population checks, project/table roundtrips and 17 matching rejections. Cases cover one, two, four, 250 and 1,000 units, zero counts, an individual population above JavaScript's exact range and the maximum i64 population. Disconnected graphs, malformed scope/units/edges, oversized unit sets, sum overflow and altered assignments reject. The prior 52 split/tree operations, 19 independent exhaustive cut oracles and 31 rejection cases still pass. The project/UI gate checks lossless single build/verification requests, correctly named downloaded artifact bytes, explicit adoption and the cleared stale picker label.

Actual browser Open of the synthetic 250-unit project retained an unverified result without running. A Run produced a single-district certificate with population 95875. Its downloaded artifact bytes and `.bisect` project inputs/settings/result matched native/WASM fixtures exactly. Reload/Open of that downloaded project again disabled artifact export until a new Run; explicit artifact adoption and verification checked the same native certificate identity, with no browser warnings/errors. Evidence: `target/certified-exact-wasm/single-browser-download-verification.json` and `target/wasm-certified-single-browser.png`. The workbench now has 21 operations. Larger exact/proof backends, canonical benchmark/ILP dispatch, ceremonies and the wider engine/practitioner/TUI/offline/all-state goal remain incomplete. Changes are local and not deployed.


### Canonical two-district exact benchmark in WASM (2026-10-04)

The workbench adds **Solve exact two-district benchmark** and **Verify exact two-district benchmark**, calling the shared native canonical solver and artifact verifier. Unlike a local recursive split certificate, this benchmark certifies the complete two-district assignment for its supplied exact model. The model requires connected nonempty districts and orders objectives by maximum scaled population deviation, total scaled deviation, weighted boundary cut and canonical assignment. The first unit fixes the district-label symmetry. The native bounded exhaustive limit is 24 units and the browser file limit remains 3 MiB. This does not replace the larger-instance ILP/proof backends or establish source truth/legal compliance.

Native instances, certificates and proof transcripts are retained in lossless JSON-text bundles. Large population and edge integers bypass JavaScript numeric parsing; displayed objectives use decimal strings. The original source, result and operation survive Save/Open. Import is inert and labels retained results unverified; artifact export requires an explicit fresh Run. Generated canonical artifact adoption selects its own verifier and clears the old picker label.

The extended exact gate passed 42 canonical native/WASM solve/verify operations, 20 independent exhaustive/BigInt optimal-assignment checks and 23 matching input/proof rejections. Cases include path, ring and complete graphs, zero population, infeasibility, individual population/edge weights above JavaScript's safe range and altered proof commitments with recomputed transcript/certificate hashes. The prior 52 split/tree and 24 single-district operations still pass. Canonical project/table roundtrips and actual UI request/download/adoption handlers are included in the permanent gates.

Actual browser Open and WASM execution of the synthetic six-unit ring produced two districts, scaled population deviations 9/18 and weighted boundary objective 2. Exported artifact bytes and the downloaded project inputs/settings/result matched native/WASM fixtures. Reload/Open of that downloaded project preserved an unverified result without execution and kept artifact exports disabled until a new Run. Explicit adoption and verification reenumerated the complete canonical model and confirmed the same certificate identity, with no browser warnings/errors. Evidence: `target/certified-exact-wasm/canonical-browser-download-verification.json` and `target/wasm-certified-canonical-browser.png`. The workbench now offers 23 operations. Larger native ILP/proof backends, ceremonies, map/plan export and the wider engine/practitioner/TUI/offline/all-state requirements remain incomplete. These changes are local and not deployed.


### Portable staged ceremony boundary (2026-10-04)

The practitioner WASM boundary now exposes `start-ceremony`, `announce-ceremony`, `halt-ceremony` and `verify-ceremony`, reusing native `bisect-ilp` ceremony functions. A supplied complete certified tree and native configuration start an open transcript. Announce accepts an exact decimal i64 recorded timestamp and supplied witness receipt strings, then releases only the next complete depth. Halt preserves an accepted prefix and prohibits further advancement. Verify accepts the released transcript alone for prefix verification, or an explicitly supplied full tree for tree-binding verification. The result identifies which verification scope ran.

Trees, configurations and transcripts cross the ABI as bounded JSON text, preserving individual i64/u64 values. Summary timestamps are decimal strings. The browser adapter limits tree seats/instances to the existing 24-unit exhaustive scope; transcript cut/round counts, configuration identities, witness counts/receipt lengths and halt reasons are checked before native verification. Recorded timestamps must be canonical decimal i64 strings. The native functions retain transactional update behavior. Exported transcripts contain accepted records, not future cuts. No external publication, timestamp authentication or witness receipt validation occurs; the boundary says so explicitly.

The permanent `scripts/web/test_ceremony_wasm.mjs` gate passed 29 exact native/WASM operations and 32 matching rejections for four- and five-seat trees. Independent checks cover canonical depth prefixes, hash-chain identities, open/completed/halted status transitions, review times and lossless timestamps above JavaScript's exact integer range. Tests reject premature rounds, duplicate/insufficient witnesses, advancing halted/completed records, blank halt reasons, wrong root binding, oversized scope, noncanonical/overflow timestamps, altered local proofs and rehashed review-time tampering. Prefix verification works without the future tree. The build script includes this gate after compiling the native reference. Existing certified split/tree/single/canonical gates still pass.

This is the shared API phase. Dedicated ceremony controls, project persistence, transcript download/adoption and actual browser workflow verification remain unfinished. The existing workbench still exposes 23 operations; ceremony is not yet a visible operation. The full WASM objective remains active, including larger exact/proof backends and wider engine/practitioner/TUI/offline/all-state coverage. These changes are local and not deployed.


### Ceremony workbench, saved projects and transcript exports (2026-10-04)

The workbench adds four visible ceremony operations: start a transcript, record the next round, halt and verify. Dedicated file selectors accept the native complete tree, configuration and released transcript as lossless original bytes. Announce has explicit exact-decimal timestamp and one-receipt-per-line controls; halt has a reason. Verification defaults to the released prefix, and an explicit checkbox includes the supplied full tree. Unrelated practitioner inputs are hidden. Generated certified trees can be adopted for ceremony start; generated transcripts can be downloaded or explicitly adopted for the next step. No action is executed merely by adopting an artifact.

Projects retain the three original file inputs, current ceremony settings and results. Open performs no computation and labels retained results unverified. Transcript exports/adoption require a fresh WASM result; valid or invalid settings changes disable exports. Receipt lines are trimmed and empty lines omitted; their strings remain opaque and are not treated as authenticated publication evidence. File reads enforce pre-read and actual byte bounds, strict UTF-8, busy guards and stale-read rejection. Native operations still enforce record identities, proofs, review intervals and witness requirements. The workbench now exposes 27 operations.

Verification: the project/UI gate passed all four exact requests, source/settings persistence, timestamps above JavaScript's safe range, explicit prefix/full-tree scope, busy/size/stale guards, transcript download payloads and adoption, invalid-edit export invalidation, unverified Open without execution and generated-tree adoption. The native/WASM ceremony gate still passed 29 operations and 32 rejections and now includes a real ceremony request builder, project roundtrip and result table fixture. Package/source and inert table regression gates passed.

Actual browser Open of the synthetic four-seat ceremony project retained an unverified result and disabled exports. A Run recorded the first accepted round. The downloaded transcript bytes and project inputs/settings/result matched native/WASM fixtures exactly. Reload/Open of that downloaded project again executed nothing. An explicit rerun and transcript adoption advanced the second depth, yielding two accepted rounds and three released cuts. Both prefix-only and explicitly selected complete-tree verification succeeded with the same transcript identity. A fresh browser start, halt with a supplied reason and verification also succeeded. No browser warnings/errors occurred. Evidence: `target/ceremony-wasm/browser-download-verification.json`, `target/wasm-ceremony-browser.png` and `target/wasm-ceremony-halt-browser.png`.

This covers the bounded local ceremony record workflow. Configuration editing/preparation assistance, external publication/timestamp/witness authentication, larger-instance proof backends and the broader engine/practitioner/TUI/offline/all-state port remain unfinished. Changes are local and are not deployed to Cloudflare.


### Native column benchmark in WASM (2026-10-04)

The practitioner workbench now exposes `solve-branch-price`, calling the existing `bisect-column` solver directly. Import an instance JSON containing `adjacency`, `populations`, `k`, `tolerance`, `formulation_only`, and `exact_fixture_limit`. Original JSON text preserves i64 populations across JavaScript; total population is exported as a decimal string. Save/Open retains input bytes and labels restored results unverified, without execution. The result JSON includes the native report and an original-input byte hash.

The existing pricing implementation completely enumerates connected subsets only up to 20 units. Both public native entry points now reject unsupported size, malformed undirected adjacency, invalid k/tolerance, nonpositive population and checked-total overflow. Previously, inputs above 20 could produce zero columns and be mislabeled infeasible. `price_columns` now returns `Result`; callers must handle validation errors. Browser exact master search is bounded to 12 units; larger supported inputs return formulation-only with no solution or bounds. These limits describe the existing benchmark, not a state-scale branch-and-price implementation. Native floating-point population bounds and unweighted boundary objective are preserved; this path does not emit an external proof certificate.

Verification: seven native tests; 30 exact native/WASM results; 26 independent exhaustive k-way partition objective checks; 15 matching rejections. The real browser imported a synthetic eight-unit/four-district project without execution, then explicitly ran it and displayed the native optimum of three cut edges. Larger solver backends, RCTX/package integration and scalable pricing remain unfinished.


### RCTX column solver workflow (2026-10-04)

`solve-branch-price-context` reads original RCTX JSON text, checks its native context identity, and sends its ordered graph/populations to the same column solver. The workbench exposes district count, population tolerance percentage, formulation-only mode, exact master unit limit, plan label, chamber and recorded creation timestamp. Inputs remain bounded to 3 MiB and the existing 20-unit pricing/12-unit browser exact-master limits. Individual i64 counts and original input bytes are retained without JavaScript numeric conversion.

Solved assignments export as native RPLAN v0.2 with ordered unit identity, zero-based districts, input context hash, solver parameters and supplied source hashes. Native serialization/readback validates the document. The output also retains native algorithm lineage and the original RCTX text. Formulation-only and infeasible results have no RPLAN. Exported context retains optional fields and weighted/bridge edges, but the solver objective counts every unweighted graph edge; it does not use edge weights or classifications. No external proof, source authentication, automatic legal audit, complete native output package/manifest or embedded plan map geometry is produced in this step.

Save/Open preserves settings and source bytes without execution. Restored results are visibly unverified and RPLAN/RCTX export remains disabled until an explicit fresh run. Input edits, including invalid settings, invalidate generated-file export; project naming preserves it. Verification covers 15 exact native/WASM outputs, 20 matching rejections, large i64 count preservation, native plan validation, assignment/identity binding, weighted bridge retention, and no-plan formulation behavior. UI checks cover exact requests/downloads, panels, busy guards, edit freshness and inert reopen. Actual browser RPLAN/RCTX exports and saved project exactly matched native fixtures for the synthetic nine-unit/three-district grid.


### Exact-text practitioner audit workflow (2026-10-04)

The workbench adds `audit-rplan-text` and `verify-audit-text`. Original RPLAN, RCTX, legal profile, certificate and optional algorithm-lineage JSON bytes are retained in local projects and passed to Rust as text. Native RPLAN v0.1/v0.2 reading and RCTX identity checks precede the shared native audit kernel. Portable limits are 3 MiB per file, 100,000 plan units, 500 districts, two million graph arcs and six selected checks. Checked population measurement validates total/district sums and matching ordered plan/context identity before generation. Certificate JSON is returned and downloaded as exact text, preserving i64 population witnesses above JavaScript's integer range.

A computed RCTX column result has an explicit 'Use solved plan for audit' action. It transfers the native RPLAN, original context and exact native lineage text, then asks for a supplied legal profile and selected checks. Native lineage uses numeric formatting in its canonical hash (for example `6.0`); passing it through a JavaScript object could change that formatting. The solver now also emits `algorithm_lineage_json`, and the text audit/verification paths check the lineage parameters hash. This establishes lineage integrity, not execution authentication or an external optimality proof.

Audit generation recomputes selected checks. Verification checks native certificate content/hash, lineage parameters and supplied plan/context binding without rerunning check outcomes. A certificate with result `fail` can correctly pass integrity verification while its result remains `fail`. Context-referencing certificates require that context. Pass refers only to the supplied profile. Source truth and recorded runtime/solver provenance are not authenticated. Full native package/manifest export, authenticated approval, preparation UI for legal profiles and broader project migration remain unfinished.

Save/Open retains exact source bytes, settings and selected checks without execution; imported results are unverified. Certificate export/adoption requires a fresh run and is invalidated by source/check/timestamp edits, including invalid edits. Plan/context manual replacement clears the inherited solver lineage. Verification: 31 native/WASM matches, 18 rejections, shared typed-audit parity across all six checks, independent BigInt totals and exact failing population witnesses, native lineage handoff and rejection of JS-normalized lineage, and UI request/export/freshness/security checks. Actual browser solver → profile audit → certificate download → integrity verification passed; downloaded certificate and project exactly matched native output. Reload/Open of that downloaded project executed nothing and disabled generated-file export until rerun.


### Native TUI configured Run

**Native TUI configured Run** exposes the native form's location, chamber, year, resolution, optional seed, plan label, version and optional balance tolerance. Select native YAML, its profile, a prepared graph, district count and any required demographics. Optional seed/tolerance form values override only those YAML fields; blank values retain the YAML settings. All remaining compositor settings pass through the shared native schema. Select an explicit portable engine such as `metis-core`; unavailable native backends are not substituted.

This operation runs the existing native population retry builder and exports accepted RPLAN, matching RCTX and report manifest. The result distinguishes requested and accepted execution seed and accepted attempt, retains original/effective YAML identities separately and records the native output namespace. The namespace is metadata; the browser does not install a native output directory. Download the native files, effective YAML, final assignments and run receipt in a ZIP. Separate buttons prepare exact accepted plan/context inputs for combined analysis or the accepted plan for native mapping; neither button runs another tool. Matching geometry and other required inputs must be supplied there.

The prepared graph/export ABI currently describes tracts. Other resolution choices are rejected explicitly. Fetching, non-tract adapters, coordinated automatic analyses/maps, external scripts and complete native State/States orchestration remain unfinished. Progress reports actual wrapper stages, not invented engine depth. Published input loading uses the form's chamber and updates location/year to match the selected graph. Edits retire downloads; saved projects preserve source bytes and form values, and Open never executes the engine.

Validation: 96 native TUI tests, a shared exact argument test, a native full-field YAML override preservation test and native binary check pass. `test_tui_run_wasm.mjs` compares native YAML/form output and native accepted-attempt assignments for four successful structure/seed cases including maximum-u64, and requires unsupported fixture cases to also fail the constituent builder. It checks exact retry/export/report-manifest composition, original/effective hashes, real stage events, ZIP/project source retention and request guards. `test_tui_run_browser.mjs` checks the actual worker, exact result/settings/ZIP, visible maximum-u64 seed, saved Open without execution, freshness, exact nonexecuting analysis/map handoffs, rejected unsupported resolution/unsafe namespace and published state/year/chamber loading. These gates run after configured-build fixtures in standalone builds. Evidence: `target/tui-run-*-validation.log`, `target/tui-run-browser/completed-run-preview.png`. Local/unpublished; this is a partial Run workflow, not full native State parity.


### Configured Run map and analysis continuity

A fresh native configured TUI Run now displays its accepted assignments on retained matching unit geometry. The interactive preview parses the exact exported RPLAN, including a maximum-u64 engine seed, matches every unique unit ID, checks recorded state/year when present and accepts only bounded WGS84 polygon positions. Clicking a unit displays its accepted district. Imported results do not display a computed map until an explicit fresh Run.

The map and combined-analysis handoffs preserve the original matching geometry bytes. Analysis receives the run's actual execution tolerance and fresh WGS84 compactness settings, clearing incompatible prior projected/population options. Both tools still require a separate explicit Run. Missing geometry leaves an explicit selection step. Mismatched state/year, missing/duplicate/unrelated units or invalid positions stop the handoff before any source replacement and preserve the accepted run. The preview can show a map error without discarding an otherwise successful engine result.

`test_tui_run_wasm.mjs` checks retained/absent geometry and state/year/count/duplicate guards. `test_tui_run_browser.mjs` checks a visible interactive accepted map with max-u64 seed and matching path count, exact source retention, native combined contiguity/compactness computation, native district SVG rendering and exact download, atomic mismatched-source rejection, absent-geometry handoff, freshness and inert Open. Combined analysis correctly remains partial if required summary/data inputs are absent; this integration does not manufacture them or establish full State orchestration. Evidence: `target/tui-run-map-*-validation.log`, `target/tui-run-browser/completed-run-preview.png`, `target/tui-run-browser/districts.svg`. Local/unpublished.


### Measured population summary and report continuity

Combined analysis now uses matching RCTX population counts when no demographic CSV is supplied. It validates the original context hash and ordered plan/context identity, computes checked district totals with the existing native measurement kernel, then fills population fields through `bisect_analysis::summary::with_population_totals`. The shared integer ideal population and absolute deviation calculations are the same as the demographic summary. Minority shares and majority flags remain unknown without demographic data; optional election and urban values are retained. A supplied demographic CSV keeps population precedence.

A matching population context plus compactness geometry can now produce the required summary/contiguity/compactness/manifest files and enable the existing explicit commission-report handoff. The package can still be partial because optional election/urban/demographic inputs are unavailable. Report-ready denotes the presence of required files, not population balance passing, source authentication or certification. Missing context populations keep summary unavailable. Bad hashes, identity/coverage mismatch, negative counts and i64 overflow fail the step without discarding other computed analyses. Source hashes and the summary population source distinguish `matching-rctx` from `demographic-csv`; ordinary non-context statistics preserve their original unknown-population behavior.

Validation: 10 native summary tests compare context-derived and demographic population metrics including zero/large totals, unknown shares and size/negative/overflow/NaN errors. `test_population_summary_wasm.mjs` checks ten complete native-host/WASM receipts, genuine native report assembly, native integer ideal population, retained election/urban data, demographic CSV precedence, absent/zero/negative/above-JavaScript-safe-i64/overflow counts and independently rehashed typed contexts. Existing statistics checks pass with 10 native/WASM receipts, 31 actual native filesystem report-file comparisons and 15 rejections; existing combined-package native receipts, missing/failed-source guards and commission assembly also pass. `test_tui_run_browser.mjs` now completes actual worker Run -> measured analysis -> native commission report with maximum-u64 seed, exact sources and separate explicit runs. Evidence: `target/population-summary-*-validation.log`, `target/population-summary-wasm/report.json`, `target/tui-run-browser/commission-rendered.bisect`, `target/tui-run-browser/population-report-preview.png`. Local/unpublished; complete State orchestration and the remaining inventory gaps are still unfinished.


### Native session defaults for Run

The existing explicit session Apply action now prepares the native Run defaults as well as Home sorting and Doctor location defaults. It copies the same five fields used by the native TUI startup form: location, chamber, year, resolution and version. Existing optional seed, label, tolerance and recorded export timestamp remain unchanged. Native output paths, adjacency override, seat and glossary preferences remain in TOML; this action does not open paths, load datasets or execute an engine.

All target preferences and the merged Run form validate before any source/control replacement. Run defaults exceeding its 200-byte field bounds or containing control characters reject the complete application and retain the prior Home/Doctor/Run state. Valid non-tract preferences can be saved in the form but the prepared-tract Run adapter still rejects execution for unsupported resolutions explicitly. Applying defaults invalidates generated downloads, and Save/Open retains the original session and form without execution.

Validation: the 23 existing exact native session parser/serializer comparisons pass. Additional Node checks verify the exact five-field patch, immutable input, retained maximum-u64 seed/label/tolerance/timestamp and closed field/bounds guards. `test_tui_session_browser.mjs` verifies actual worker session inspection/application, exact saved Run defaults with retained options, original TOML retention, invalid preference atomicity across all four panels and inert Open/freshness. Existing project and configured Run Node/browser regression gates pass. Evidence: `target/session-run-*-validation.log`, `target/session-run-*-regression.log`, `target/tui-session-browser/applied.bisect`. Local/unpublished; the full capability goal and remaining orchestration gaps remain active.


### Explicit prepared Census unit kinds

Prepared graph JSON may now declare `unit_kind` as `county`, `tract`, `block-group` or `block`. Omission preserves the original tract ABI, serialization, graph identities and export bytes. The shared engine requires unique numeric IDs of 5, 11, 12 or 15 digits respectively. Explicit declarations also bind IDs to the selected state and supported Census vintage. Existing population, dimensions, physical boundaries and geometry checks still apply. Precinct/imported units require their own county identity adapters and are rejected here.

Configured Run and native plan/context export retain the declared kind, ordered unit identities, population vector and first-five-digit county membership. The Run form must match the graph kind: `county`, `tract`, `block_group` (or native `bg`) and `block`. A kind/ID-width mismatch or form-resolution mismatch fails before execution; existing tract graphs are not relabeled into another resolution. The tract-parent multiscale adapter explicitly rejects other parent graph kinds. Map display and separate native rendering use matching geometry with the original IDs.

This enables runs on explicitly supplied prepared graphs at those resolutions. It does not prepare block, block-group or county sources from Census archives, bypass existing source/graph/practitioner/map capacity limits, establish Census source authenticity, or finish complete native State/States orchestration. Large block-state source and output capacities still require validation and further work.

Validation: `test_unit_kind_wasm.mjs` checks eight actual native assignment comparisons across four declared kinds and seed42/maximum-u64, then feeds each exported RPLAN/RCTX into the independent native reader and BFS contiguity kernel. It verifies exact kind/order/population/county-prefix retention, legacy graph and export identities, block-group alias, safe source/project/geometry retention, width/state/form/unsupported-kind guards and rejection by the tract-parent multiscale adapter. `test_unit_kind_browser.mjs` checks actual worker full-result equality, inert Open, original graph/geometry retention, visible accepted maps and separate native district map rendering for all four kinds. Existing 78 native/WASM multiscale cases and 32 invalid requests, native binary input parity, legacy Run and project regressions pass. Evidence: `target/unit-kind-*-validation.log`, `target/unit-kind-*-regression.log`, `target/unit-kind-browser/block-map-preview.png`. Local/unpublished; remaining capability and orchestration gaps stay active.

### Independent N-way METIS imbalance setting

Direct N-way runs expose an optional **METIS imbalance factor** in both state and national mode. This forwards native State's `--ufactor` convention: 5 means 0.5%, and the shared native kernel clamps the supplied u32 factor to 1–1000. Accepted-plan population tolerance remains a separate audit setting. Leave the field blank to preserve the laboratory's existing tolerance-derived factor. The override applies only to direct N-way partitioning; recursive cuts and native YAML retain their existing semantics.

Saved projects retain the requested factor, including zero; opening restores it without running. Results record requested and effective factors alongside population tolerance. Invalid values and altered factor evidence are rejected. Verification includes 20 complete native/WASM result comparisons and 20 independent calls to native State's N-way dispatch entrypoint, with seven distinct assignments, u32 boundary values and the maximum u64 seed. Two real-state national requests also match native results. Actual Rhode Island browser checks cover forwarding, blank/zero handling, Save/Open, scope visibility and invalid-input refusal. Evidence: `target/nway-ufactor/evidence.json` and `target/nway-ufactor-browser/evidence.json`. Full capability parity remains incomplete; these changes are local and unpublished.

### Home selection and native metric help

On Home, Up/Down outside editable controls selects the previous/next inspected plan and stops at the first/last row, matching native TUI bounds. It updates detail and retained selection through the existing selection handler, without a worker request. Save/Open retains the selection and exact source files; reopening retires exports until a fresh inspection. Compare uses the selected row's original analysis sources.

The Metric glossary button (or `?` on Home) shows all seven native TUI definitions, labeled as desktop wording. Escape, `?` and Close dismiss the modal and restore focus. Modal focus prevents screen shortcuts from changing the operation behind it. Busy operations disable help and selection; field editing keeps normal keyboard behavior. The glossary uses text nodes from a generated source module. `python scripts/web/native_metric_glossary.py --check` detects any difference from the native definitions. Actual Chrome tests check literal content, modal focus, mobile bounds, row movement and both bounds, exact source/selection Save/Open, no worker execution during selection/help and a fresh selected-plan Compare handoff. Evidence: `target/workbench-navigation-browser/evidence.json`. Native sort-cycle and remaining screen actions still require review.

### Native Home sort cycle

On Home, **Cycle native sort and inspect** (or `s` outside editable controls) explicitly invokes Rust library inspection on the retained source files. The native cycle is label ascending, label descending, county splits ascending, county splits descending, Polsby-Popper ascending, Polsby-Popper descending, population deviation ascending, then label ascending. Manually selected deviation descending also cycles to label ascending. Shared Rust supplies ordering, stable ties and unknown metric handling; there is no browser comparator. As on desktop, the selected row position stays fixed, so the selected plan identity may change after sorting. Ordinary sort dropdown edits retain the existing explicit Run behavior.

The action inspects sources and updates result/export freshness; it never draws or certifies a plan. Busy operations block it, Cancel retires exports and clears pending positional selection, and a fresh inspection recovers the rows. Save/Open retains exact sources, sort and selected plan with no execution on Open. `?` now opens native metric help on Home, Run, Compare, Verify and Doctor outside editable fields. Actual Chrome checks all seven Rust-sorted row lists against the independent native TUI discovery executable, selected position, stable ties/unknown metrics, source bytes, save/reopen, editing, busy and cancellation. Existing 40 native library comparisons and actual folder browser regressions pass. Evidence: `target/tui-sort-browser/evidence.json`. Remaining screen-specific actions and complete orchestration remain open.

### Native uncapped convergence in the laboratory

Standard-bisection convergence runs have an explicit **Native convergence: no maximum seed count** checkbox. Native CLI/YAML convergence already dispatches without a seed cap; this option now reaches the same existing WASM engine behavior from the state/national laboratory. Checked runs stop when the requested no-improvement tail is reached. The browser's wall-clock timeout and Cancel still terminate work. The maximum-seeds field is hidden and disabled while inactive. Unchecked/omitted (and explicitly false saved settings) preserve the existing capped behavior. Results display the recorded seed limit or its absence separately from the actual halt reason. Neither stopping rule proves global optimality.

Save/Open retains the choice and checks assignments, score, seed schedule, limit, threshold and halt consistency. Uncapped completed results require a null native seed limit and threshold stop; a cap-stop claim is rejected. Opening remains inert. Verification includes eight complete native/WASM real RI/IA comparisons across geographic/unweighted boundaries and seed42/maximum-u64, independent replay of every attempted candidate including seed wrap, capped/false compatibility, state/national roundtrips and forged evidence rejection. Actual Chrome checks Rhode Island native assignments and stopping evidence, checked/unchecked forwarding, inactive cap controls, search scope and inert restoration. Evidence: `target/convergence-unlimited/evidence.json` and `target/convergence-unlimited-browser/evidence.json`. Full parity is still incomplete; this is a local unpublished change.

### Selected native maps in the accepted-plan workflow

Native TUI Run's optional follow-up workflow now has independent Districts, Political, Demographic and Compactness map choices. These reach the existing shared Rust `map_types` option. Omitted settings retain the original district-only default; maps enabled with no selected type fail preflight. Selected non-district maps use computed workflow analysis. If that analysis is absent, Rust records the map as unavailable while retaining the accepted Run and earlier files. Map SVG/PNG files are included in the existing hash-checked workflow ZIP.

After a fresh run, **Completed workflow map preview** displays the selected native PNG directly from the generated file bytes. Changing map settings retires previews/exports. Imported results do not display a fresh preview until rerun. Object URLs are revoked when outputs change or become stale. Save/Open retains exact source files and map choices without execution.

Verification: six complete native-host/WASM workflow receipts, exact constituent analysis/four-map/report bytes, invalid/duplicate/empty map preflight and district-only compatibility. Actual Chrome checks all four outputs and rendered PNG source bytes, selected subsets, unavailable analysis, accepted-plan retention, export freshness, exact source retention and inert restoration. Evidence: `target/state-workflow-browser/map-types-evidence.json`, `target/workflow-map-types-native.log` and `target/workflow-map-types-browser.log`. Four-map fixtures use synthetic geometry and demographic/election data. This does not establish all-state four-map capacity or full workflow parity; changes remain local and unpublished.

### Native report delivery within the accepted-plan workflow

The optional Run follow-up report stage now exposes **Choose report filenames and formats**. Custom delivery accepts a 1–100 character label containing ASCII letters, digits, underscores or hyphens, HTML/JSON choices (at least one), and an optional companion `audit.json`. Shared Rust writes `report/<label>_report.html` and/or `report/<label>_report.json` plus the selected companion audit. Omission retains the original `report/report.html`, `report/report.json` and `report/audit.json` package. The companion audit records report inputs/checks and is not a plan certification. Analysis remains required before reporting.

Changing settings retires computed outputs; invalid labels or formats fail before worker dispatch. Busy operations disable delivery controls. Save/Open retains exact sources and delivery choices without executing, and imported outputs require rerun before export. The existing hash-checked workflow ZIP contains the selected native report bytes and filenames. Turning custom delivery off returns to legacy behavior.

Verification: eight complete native-host/WASM workflow receipts, JSON-only/no-audit, HTML with audit and both formats/no-audit, unchanged constituent report bytes for matching inputs and preflight refusals. Actual Chrome checks whole native result equality, exact ZIP bytes, source retention, inert reopening, busy controls, label freshness, invalid-format no-dispatch and restored legacy delivery. Standalone commission-delivery regression also passes. Evidence: `target/state-workflow-browser/delivery-evidence.json`, `target/workflow-delivery-native.log` and `target/workflow-delivery-browser.log`. Fixtures use synthetic geometry/demographic/election data; all-state report capacity and full parity remain unproven. Local and unpublished.

### Optional compactness measurements in the accepted-plan workflow

Native TUI Run's optional analysis stage now exposes **Include native exact Reock** and an optional local population centroid CSV. Columns are `GEOID,x,y,population`, bounded to 3 MiB; coordinates must already use the retained geometry's coordinate basis. The native analyzer checks coverage, duplicates, finite coordinates and exact u64 counts, and records the source hash and measurement units. Exact Reock uses the existing native numerical routine and 50-million work bound. The workflow does not generate or authenticate centroid observations. Omitted options retain legacy measurement output; absent geometry leaves compactness unavailable.

Save/Open retains exact CSV bytes and the Reock choice without execution. Oversized selections are refused before source reading/worker dispatch; clear and settings edits retire outputs. Busy operations disable controls. **Prepare combined plan analysis** preserves both options and the exact accepted plan/context for a separate explicit Run. A CSV missing a plan unit produces a partial analysis with no compactness file while preserving the accepted plan and earlier files.

Verification: ten complete native-host/WASM workflow receipts, actual optional measurement/source-hash output, independent standalone native-kernel report equality and missing-unit failure retention. Standalone compactness regression passes 38 native/WASM comparisons, 37 report-byte comparisons and 44 refusals. Actual Chrome checks whole native workflow equality, exact CSV Save/Open, nonexecuting option/source-preserving handoff, clear/legacy behavior, oversized-source refusal and busy guards. Evidence: `target/state-workflow-browser/measurements-evidence.json`, `target/workflow-compactness-native.log`, `target/workflow-compactness-browser.log` and `target/workflow-compactness-standalone-regression.log`. Workflow fixtures are synthetic; full-state exact-Reock capacity and full parity remain unproven. Local and unpublished.

### Accepted Run to native comparison

After a fresh completed native TUI Run, **Use accepted Run as Compare Plan A/B** prepares that comparison side without running the comparator. Keep Plan A, return to Run, change seed/options and run again, then retain the second Run as Plan B. The other side's exact sources stay retained. Run Compare explicitly to inspect both. Opening a project or changing Run inputs retires handoff eligibility.

The handoff checks engine assignments against native RPLAN's zero-based district labels, then retains native one-based assignment JSON and any available exact compactness/summary/splits/contiguity workflow files. Available analysis hashes and accepted-plan/context source bindings are verified. Missing analysis is not fabricated; native comparison records its existing defaults. Async hashing captures project revision/result identity; an edit aborts the handoff before mutation. Existing native-plan-reader 3 MiB and comparison 8 MiB per-source/16 MiB total bounds apply. Larger handoffs need separate capacity work.

Run and workflow ZIPs now contain `data/final_assignments.json`, matching the native comparison folder path. Run ZIPs also retain the original root `final_assignments.json` alias with identical bytes. Workflow ZIPs retain generated native analysis paths. Existing export capacity rules remain in force; this change does not install files into a native output tree.

Verification: a second actual native/WASM Run, native directory comparison equality, exact analysis/assignment sources, explicit zero-to-one-based alignment, exported workflow ZIP extraction and native comparison equality, absent analysis/default behavior and binding/hash tamper refusal. Actual Chrome generates two Runs, retains A/B without executing, checks whole native comparison output and exact download, inert reopening, other-side/source retention, busy refusal and an edit during real hash work. Evidence: `target/tui-run-compare/evidence.json`, `target/tui-run-compare-browser/evidence.json`. Existing native Run/Compare and browser ZIP/workflow regressions pass. Fixtures use synthetic graph/geometry/data; full-state capacity and full parity remain unproven. Local and unpublished.

### Exact RPLAN/context comparison and Run-pair handoff

**Compare exact RPLAN files with context** accepts two original RPLAN files (3 MiB each) and an optional matching RCTX (8 MiB). It calls the same native plan-measurement and comparison kernels as the existing structured operation, preserving raw text and recording SHA-256 for both plans and context. This supports exact large integer tokens without browser numeric conversion. Measured population, connectivity and subdivision results remain distinct from unit-count proxies and label-sensitive assignment differences. Missing context leaves measurements unevaluated. Identical ordered units/kind/state/year/universe are required; cross-cycle comparison needs a crosswalk.

Accepted Run A/B handoffs now retain exact RPLAN and RCTX sources alongside the five TUI comparison sources. **Prepare full plan/context comparison** opens the exact-text form without execution. Run explicitly to compute actual measurements. The native TUI summary remains available with its historical field defaults. If an exact source exceeds the full-comparison boundary, that side's exact handoff is retired while the TUI handoff remains usable. Source edits retire exports; Clear and missing-plan preflight do not execute. Save/Open retains exact source bytes and stays inert.

Verification covers whole native/WASM text results across the existing context fixtures, equality of existing structured measurements, original-byte hashes, a raw maximum-u64 producer field, input limits and project roundtrip. Actual Chrome verifies whole native results, exact source retention, actual versus missing-context measurements, inert reopening, invalid-source no-dispatch and two actual Run handoffs into measured population/connectivity comparison. Existing structured comparison and proof regressions pass after rebuilding the WASM/native host. Evidence: `target/text-comparison/evidence.json`, `target/text-comparison-browser/evidence.json`, `target/tui-run-compare-browser/evidence.json`. Fixture-based verification does not prove all-state capacity or full goal completion. Local and unpublished.

### Accepted Run to audit and verification

**Prepare accepted plan audit** appears after a fresh completed native TUI Run. It selects the exact accepted RPLAN and RCTX, retains any existing supplied legal profile, check choices and recorded audit timestamp, and removes an unrelated retained audit certificate/algorithm lineage. The handoff does not execute an audit. Select any missing profile and Run explicitly. Missing profiles are refused before worker dispatch; no default legal policy or certificate is fabricated.

After audit, the existing **Use certificate for verification** handoff selects the original certificate bytes without execution. Run again to verify certificate integrity and supplied plan/context bindings. Save/Open retains all exact sources and leaves imported results unverified until rerun. Busy, imported and stale Runs cannot prepare the handoff. Existing plan/context limits remain 3/8 MiB.

Verification: complete native/WASM audit and verification equality using accepted Run outputs, exact source preservation, supplied profile/time/check retention, stale certificate/lineage retirement and missing-profile preflight. Actual Chrome runs the full Run → Audit → Verify sequence, checks whole native outputs, inert reopen, busy/imported guards and no dispatch for a missing profile. Evidence: `target/tui-run-audit/evidence.json`, `target/tui-run-audit-browser/evidence.json`. Fixtures use synthetic graph/geometry and an explicitly synthetic test profile; this does not establish legal compliance, authenticated data or full goal completion. Local and unpublished.

### Named and custom chambers on prepared graphs

Configured Run now accepts a chamber name of 1–200 UTF-8 bytes without controls. The chamber input suggests congressional, house, senate, lower, assembly, upper and custom, while retaining any bounded supplied name. Native aliases use their existing chamber policy lookup. Explicit YAML tolerance still takes precedence; an absent policy entry uses the actual native fallback (0.5% for congressional, 5% otherwise), which is an engine default rather than a supplied legal policy. Existing known-chamber behavior is preserved. District count remains explicitly chosen within the current 1–500 UI range.

The existing TUI form also preserves named chambers through accepted native plan/context and manifest export. An explicitly supplied imported graph can retain its own four-digit year, arbitrary bounded unit IDs and aligned county identities. This does not fetch country data, invent allocation counts, infer a chamber's legal policy or validate source authenticity. US-only source batches retain their native congressional allocation workflow.

Verification: 33 whole native/WASM configured Run, export and accepted TUI Run comparisons, default/explicit tolerance across eight chamber names/aliases and an independent native tolerance-function oracle. A synthetic imported MT/2021 graph with Unicode name `Dáil Éireann` retains exact units, year and chamber metadata. Actual Chrome checks whole native output, Unicode/project retention, configured custom-chamber fallback and explicit count, inert reopening and empty-name no-dispatch. Existing configured-run and proof regressions pass after rebuilding the engine and native host. Evidence: `target/custom-chamber/evidence.json`, `target/custom-chamber-browser/evidence.json`. Synthetic MT/Irish-name fixture labels test metadata portability and do not represent an authentic country's map or electoral system. Full capability parity remains unfinished; local and unpublished.

### Accepted Run to one-move improvement and comparison

**Prepare accepted plan improvement** selects exact accepted RPLAN/RCTX sources without executing. Existing improvement tolerance, label and timestamp remain selected. If no improvement settings exist, the starting tolerance is the accepted Run's requested tolerance; label and timestamp stay unset for explicit choice/Run defaults. Prior certificate/lineage inputs retire. Native improvement chooses the best feasible single neighboring-unit move minimizing unweighted graph edge cut, including bridges. It does not prove a globally optimal cut or optimize geometric boundary length.

After a fresh improvement, **Compare original and improved plans** retains original and child RPLAN text plus child context, verifies output file hashes and opens exact-text comparison without execution. Revision/result/source guards abort an asynchronous handoff if the project changes. Run comparison explicitly to inspect changed units, actual population and graph connectivity. The existing verification handoff separately selects the exact child certificate and sources; verification applies to that certificate's declared improvement profile. Open stays inert and imported/stale/busy outputs cannot hand off.

Verification: four complete native/WASM Run/improve/compare/verify outputs on a synthetic six-unit imported graph. The accepted Run uses 5% tolerance; an explicitly selected 34% improvement tolerance permits one move reducing unweighted cut from 3 to 1. Both the population change and one changed unit are measured in comparison, and certificate bindings verify. This is not a same-tolerance improvement or global-optimality proof. Exact sources, retained settings, new-setting defaults, retired prior claims, output-hash tampering and project roundtrips are checked. Actual Chrome runs the full sequence and verifies native outputs, nonexecuting handoffs, inert reopening and busy/imported guards. Evidence: `target/run-improvement/evidence.json`, `target/run-improvement-browser/evidence.json`. Full-state capacity and full parity remain incomplete. Local and unpublished.

### Accepted Run to native reproducibility package

**Prepare native reproducibility package** selects the exact accepted RPLAN, original native report manifest and every available `analysis/*.json` workflow source. Workflow plan/context bindings and analysis hashes are checked before mutation, under the existing 3 MiB combined package-input bound. Existing package labels are retained; otherwise a safe Run label is suggested, with `plan` as fallback. Missing analysis is left absent. Run export explicitly after reviewing the inputs.

The original RPLAN, RCTX and configured source inputs remain in the project. Prior active certificate/lineage inputs retire when its plan/context selections change. Native package output converts modern plans to native legacy RPLAN 0.1 and includes its generated audit/instruction files; it does not include the original RCTX or all configured source recipes. Use the accepted-run/workflow ZIP or project archive for those original sources. Verification scripts are generated instructions and are never executed by the browser; this handoff does not establish that a script can reproduce a configured/prepared-graph run.

Save/Open stays inert. Imported/stale/busy Runs cannot hand off; edits during hashing abort before changing package sources. Verification checks complete native/WASM package equality, exact manifest/analysis bytes, native zero-to-one-based legacy assignments, context retention, changed source/hash/label refusal and plain-run absence behavior. Actual Chrome runs the export workflow, compares native result and ZIP bytes, retains sources across inert Open, blocks busy/imported handoffs and rejects a revision change during actual hash work. Evidence: `target/run-reproducibility/evidence.json`, `target/run-reproducibility-browser/evidence.json`. Native exporter regression also passes. Fixtures are synthetic; full reproduction and full capability parity remain incomplete. Local and unpublished.

### Raw native state-analysis aggregation

**Aggregate raw native state analysis folders** accepts a selected collection containing `state/analysis/type.json` files, an explicit native Census year and multiple analyzer types. Raw reports need no inserted state/year/analyzer fields. Shared native merge functions inject the supplied folder-state scope into district rows; proportionality keeps native state-level records and year injection. Source declarations and first-record column behavior remain unchanged, including native `unknown` analyzer labels where absent. This is distinct from the existing strict report aggregator that checks matching declared scope/methods.

Native `all` uses `AnalyzerType::all_concrete()` and excludes opt-in compactness, partisan and bloc-voting. Explicit type selection supports those analyzers. Outputs match native `us_type.json`, CSV and district/state JSON-array filenames according to the selected format/CSV option. Invalid JSON is recorded/skipped; native no-data selections create no output for that type. Duplicate/unsafe fields, deep JSON, nonobject district rows and resource bounds are guarded before merge. Limits: 60 state labels, 3 MiB per file, 8 MiB combined, 500 rows per district report. State/year scope is supplied and not authenticated. No analyzer or referenced path is executed.

Save/Open retains exact raw source bytes and settings without aggregation. Settings/source edits retire ZIP exports; Run explicitly to recompute. The output ZIP checks native file hashes and excludes ungenerated files. Native filesystem discovery, existing-output skip/force installation and parquet remain separate host/stub boundaries.

Verification: eight full native-host/WASM matches and 30 exact files from actual native CLI aggregation across district summary, state proportionality, native all/default exclusions, explicit compactness and CSV/JSON-array modes. Fixtures deliberately omit scope fields and include malformed JSON and a raw u64 maximum. Source/folder limits, output-hash tampering and project roundtrip are checked. Actual Chrome verifies selected folders, whole native output and ZIP bytes, default/opt-in selections, visible skipped sources, export freshness and inert reopening. Existing strict aggregation and proof regressions pass after the rebuilt engine. Evidence: `target/native-aggregate/evidence.json`, `target/native-aggregate-browser/evidence.json`. Fixtures are synthetic; all-state capacity and full capability parity remain unproven. Local and unpublished.

### Native aggregate selection controls actual reads

Raw aggregation now resolves requested types before browser file preflight/read and before native parsing of inactive reports. The browser catalog is generated from native `AnalyzerType.name()` and `all_concrete()`; `python scripts/web/native_aggregate_catalog.py --check` detects drift. Native `all` continues to take precedence over extra opt-in selections. An excluded large or malformed file does not block selected aggregation or appear in selected-source skip diagnostics.

After changing types, **Read selected folder for current types** reloads only the selected analyzer files from an existing local FileList. Imported projects restore retained bytes and do not regain disk access. Selected files retain existing bounds; empty selected source maps produce explicit no-data outcomes. Run and navigation are blocked during reads, and source/settings changes retire outputs. Retained inactive source bytes remain in a project but are excluded from the worker request.

Verification includes an excluded 100 MiB file that must never be read, explicit selection refusal before read, native all precedence, inactive request filtering, direct-WASM unsafe-row exclusion and no-selected-data handling. Actual browser loading is held at a real File.arrayBuffer boundary to verify Run/navigation guards; rereading selected opt-in types is checked. The rebuilt engine passes eight native/WASM aggregation cases, 30 actual CLI output-file byte comparisons, and the native proof-model and proof-pipeline gates. Workbench navigation and the staged package dependency checks also pass. Evidence: `target/aggregate-selection/evidence.json`, `target/native-aggregate-browser/evidence.json`. Capacity and full parity remain incomplete; local and unpublished.

Raw aggregate discovery now matches the native immediate state-folder scope: only `<collection>/<state>/analysis/<type>.json` is loaded. Other files reveal state directories without being read; nested backup analyses are ignored. States with no selected reports are visible and survive project Save/Open. Case-fold state collisions fail before reads. Selection and real-browser gates verify this behavior; the eight-case/30-file native CLI regression also passes. Completely empty directories remain invisible to a FileList.

### Native research strict outcomes

Research validation now exposes the native strict option. The shared native strict_failure kernel supplies the exact failed-metric message to CLI and WASM. Browser Run retains report/report_json on strict failure, with status strict-failed and native exit_code 1; ordinary threshold failure with strict false remains exit_code 0. Legacy request/project omission means false. Strict settings survive inert Save/Open, edits retire export authority, and threshold edits advance source revisions.

The strict gate passes three actual CLI exit/report comparisons and whole host/WASM matches. Browser gates verify the failed report remains downloadable, changing strict/threshold settings retires outputs, strict passing runs and inert Save/Open. This validates convergence-report orchestration, not sampler correctness or exploration.

The shared CLI strict path passes `cargo check --offline --locked -j1 -p bisect-cli`. Existing research gates pass six native/WASM matches, three CLI report-byte comparisons and 13 refusals. Project and laboratory security/roundtrip gates pass. A held real browser source read is rejected after a threshold edit, preserving previous chains and the newer setting. Rebuilt artifact-bound proof-model and proof-pipeline gates pass; the local package resolves 171 files and 578 dependencies. Full parity remains unfinished and these changes are unpublished.

### Native multi-label comparison summary

Native compare --labels now has a separate summarize-plan-labels WASM/UI workflow. CLI row projection and exact native table/CSV writers delegate to shared bisect-analysis::multi_plan_summary. Selected per-label manifests and four analysis JSON files are retained exactly; native missing/invalid placeholders, empty compactness NaN, fallback deviation fields, raw u64 split counts, tolerance, requested order and duplicate labels are preserved. Actual folder reads block Run/navigation; edits retire exports; Save/Open is inert.

Select the plans collection itself: `<collection>/<label>/manifest.json` and `<collection>/<label>/analysis/{summary,compactness,splits,contiguity}.json`. Requested label order is editable and unavailable labels retain native placeholders. Other files are not read; nested backup analyses are ignored. Native CSV format is preserved, including its existing values and rounding. Native `--format json` in this multi-label branch still produces a table. Bounds are 60 requested labels/plan folders, 3 MiB per source and 8 MiB total. Analyses are supplied claims, not recomputed or authenticated; completely empty folders cannot be inferred. Local and unpublished; full parity remains unfinished.

Multi-label validation passes three whole host/WASM matches and nine actual CLI output-byte comparisons, plus real browser export/order/folder-read/inert-project gates. Worker requests omit retained unrequested-label sources. The shared CLI path passes its compile check; project regressions, proof-model/pipeline regressions and the staged package checks pass (172 files, 581 dependencies). Evidence: `target/multi-summary/evidence.json`, `target/multi-summary-browser/evidence.json`.

### Multi-label selection controls local reads

Multi-label summary folder reading now honors requested labels before file size checks or reads, matching native compare label lookup. Empty labels discover all plans; explicit labels preserve order, duplicates and unavailable rows. An explicit reread button loads only the current label selection from the retained local FileList. Imported projects retain bytes but do not restore local-folder access. Unselected folders do not consume the selected-plan count; universal folder entry/path and project/request bounds remain.

The selection gate verifies excluded 100 MiB sources, 70 unused plan folders, explicit selected-file refusal before reads and duplicate/missing-label placeholders. The browser gate selects a real folder containing a 100 MiB excluded file and asserts its File.arrayBuffer is never called; selected-label rereads produce the same native output and survive inert Save/Open. The three host/WASM and nine actual CLI byte comparisons still pass. These are local, unpublished changes; full parity remains unfinished.

### Engine option completion checklist

`docs/specs/wasm-engine-option-inventory.json` complements the command inventory with 227 exact source declarations: 20 Run fields, 79 State fields, 53 States fields, 22 shared YAML fields, 12 prepared-graph fields and 41 direct engine options. Run `python scripts/web/wasm_engine_option_inventory.py --check` to detect field/type/comment/default-attribute drift. Regeneration retains reviews only when the exact declaration matches; changed fields revert to unreviewed. This checks declaration coverage, not runtime parity. Four focused source reviews are recorded; the remaining 223 are unreviewed in this checklist.

Native State currently loads adjacency populations, checks CVAP availability but discards the returned effective source, and writes the requested population source into its manifest. Its `--population-source` declaration is not proof that VAP, CVAP or prison-adjusted populations were selected. Browser execution uses the supplied prepared-graph population vector, and RCTX export preserves it. Neither field authenticates the upstream population basis. Explicit metis-core execution is available; incompatible native backend declarations are refused, with no silent substitution. Source review must distinguish declarations, execution and metadata before further parity claims.

### YAML resolution and dispatched options

Configured run results now show **Options sent to the engine**, retaining every present option separately from the resolved YAML compositor. Native dispatch can interpret declarations differently: ratio search convergence becomes a per-split multi-seed budget, and the native moving-knife `polsby-popper` YAML token currently dispatches the Reock branch. Two actual browser runs verify these displays, original YAML/result Save/Open retention and inert opening. The display reports dispatched parameters, not a claim that each parameter influences each structure.

All 22 shared YAML fields now have scoped reviews in the field inventory. Refreshed gates pass 61 native conversion/validation comparisons, 80 configured native/WASM results including 70 actual CLI assignment oracles, ten native AreaSection comparisons and five native inspection comparisons. Data-dependent weighted execution, independent moving-knife CLI assignment coverage and complete workers/years/analysis-type orchestration remain incomplete. There are now 25 reviewed fields in the new checklist, with 202 still unreviewed; this does not replace or invalidate older capability evidence. Full parity remains unfinished. Changes are local and unpublished.

### Configured moving-knife native dispatch oracle

The native CLI YAML reference now supports the exact moving-knife dispatch branch. Its 80 assignment/balance comparisons match configured WASM for default and 0/1/36/180/360 orientation counts, omitted/42/max-u64 seeds, Reock/Polsby/Polsby-Popper and native unknown-token interpretation, recursive district counts and uneven population/centroid fixtures. Exported RI (250 units) and IA (896 units) graphs also match. Independent BFS verifies reported connectivity.

These first-attempt fixtures use an explicit 25% tolerance to compare native behavior, not a recommended districting tolerance. There are 4 balance misses and 1 disconnected outcome, including IA's disconnected plan. Successful native/WASM matching is not plan acceptance, legal certification or optimality. Original filesystem source acquisition, arbitrary capacities and full capability parity remain unproven. Evidence: `target/config-mka-oracle/evidence.json`. Changes remain local and unpublished.

### Native inactive county coefficient dispatch

Configured YAML county coefficients at or below the native 1e-10 activation threshold now dispatch geographic weights with effective alpha zero. This fixes the previous direct-engine rejection of tiny/negative inactive declarations, while preserving original YAML, declared coefficient and resolved native compositor in the project. Active coefficients keep native county weighting. Native CLI reference now composes the actual SubdivisionWeighter and refuses a C-FFI feature build or non-metis-core declaration.

Seventy actual native CLI assignment and whole host/WASM comparisons pass across inactive/active coefficients, selected split/search modes, exact seeds and exported RI/IA graphs. Native and independent weighted/physical cut sums match. Browser runs show declared versus effective coefficients while preserving original YAML and inert Save/Open. Native comparison must use `cargo build --offline --locked --no-default-features -j1 -p bisect-cli --example config_seed_reference`; C FFI is a compile-time bisection choice and the oracle explicitly rejects incompatible builds or declarations. The 80 configured-run, 80 moving-knife and 16 direct-county regressions and rebuilt proof gates pass. Full capacities and complete parity remain unfinished; local and unpublished.

### Native sparse minority dispatch and root scoring

Native VRA configuration parity is now checked against actual portable CLI dispatch and its MinorityOverrideWeighter. Mixed-profile multi-seed comparisons exposed WASM dense expansion of the native sparse boost map; dispatch now preserves the sparse map. Unweighted dispatch now preserves the native empty map. All-cut reporting still uses implicit unit defaults, while VRASection root scoring retains the native candidate cost. New weighting evidence explicitly identifies the representation. The native two-district GeoSection shortcut now uses canonical edge-order accumulation, fixing one-ULP report drift from randomized HashMap order.

The configured oracle covers 49 native assignment/cut and whole host/WASM cases. Four exported RI/IA graph cases use explicitly synthetic demographics; no Census demographic authenticity is claimed. Direct minority regression now uses actual CLI dispatch instead of a dense geographic control, which could conceal candidate-ranking differences. Current sparse and historical dense ledgers retain arithmetic checks; importing a record never authenticates algorithm execution. The staged preview refreshes 17 structures, 16 searches and 7 weights from the native WASM exporter, checked with `python scripts/web/wasm_catalog_capabilities.py dist/wasm-lab-build/catalog.json --check`. Full parity remains incomplete; local and unpublished.

Final canonical-build verification passes 49 configured native CLI/full host/WASM comparisons, 576 direct VRASection cases, 16 strengthened minority cases and 24 real-graph ReCom/VRASection project/worker/map roundtrips with 48 atomic tamper refusals. Synthetic demographic provenance remains explicit. Current sparse and legacy-format ledger arithmetic checks pass. Existing 80 configured-run/70 county comparisons, proof-model/pipeline checks and the staged catalog/package checks also pass. None of these outcomes establish global optimality, authentic demographic observations or full capability parity.

### Native sparse partisan override

Direct partisan boundary weighting now preserves the native PartisanOverrideWeighter sparse boost map instead of expanding implicit unit edges before candidate search. A direct native CLI comparison exposed mixed-profile multi-seed assignment differences. Full reported boundary arithmetic retains implicit unit defaults; new evidence identifies native dispatch representation. Current and legacy weighting ledgers remain readable, and the broad regression no longer treats a dense geographic replay as an independent sparse native oracle.

Partisan shares in these fixtures are synthetic and remain explicitly labelled. Native `--partisan-shares` override semantics are compared via the actual native weighter on complete supplied share vectors. This does not port the shared YAML schema to accept a partisan weights token; native YAML itself rejects that token. Direct browser engine settings expose partisan weighting separately. Full source acquisition, structural/search combinations and capacity remain unfinished. Local and unpublished.

Direct n-way field reviews were refreshed against the current engine: 20 whole native/WASM and actual native State-kernel comparisons, 24 refinement cases and real RI browser worker/native equality, zero/blank overrides, clamping and inert Save/Open pass. Fields reviewed are METIS imbalance factor, objective, trials and iterations. Imbalance factor and final population tolerance are distinct. This evidence remains scoped to tested direct n-way cases and does not prove every recursive method, refinement or capacity.

Seven geometric/annealing option reviews were refreshed on the current artifact. The gates pass 56 annealing successes with nine matching failures, 18 Voronoi cases and 24 moving-knife cases. Voronoi fixtures retain 15 balance misses/nine disconnected plans; all 24 moving-knife folded-line fixtures are disconnected. These are parity observations, not plan acceptance. Direct ABI and YAML moving-knife defaults remain distinct. Full orchestration, capacities and every parameter combination remain unproven.


The corrected sparse-partisan build passes 250 exact whole-native/WASM comparisons across all 50 prepared 2020 tract graphs: five standard-bisection searches, congressional district counts, Volume objective, three METIS trials, seed 42 and 10% population tolerance. All 250 project reopen/export roundtrips retain the synthetic stripe shares and exact raw RCTX; large contexts keep raw bytes without a duplicate structured copy. Fifteen invalid inputs/options reject. Independent population totals, district sizes and graph-component checks match all saved reports. Florida single-seed has 44.03780596760534% maximum population deviation and misses the 10% request; the other 249 pass balance, and all 250 are connected. Native and WASM agree on the Florida outcome. Gate bookkeeping now counts real-state constraint misses, with the completed evidence independently corrected by `scripts/web/summarize_partisan_real_states.py`. Evidence: `target/partisan-real-state-verification.json` and `target/partisan-real-state-outcomes.json`. These checks use synthetic partisan shares, not election observations, and do not establish global optimality or full capability parity.


### Canonical CompactBisect geometry scoring

CompactBisect's shared Polsby-Popper geometry sums now visit vertices in index order. Previously, random HashSet iteration could change rounded area/perimeter totals and break candidate ties differently per process. The pre-fix native reference produced two assignment variants in 32 identical invocations. The targeted sum-order regression passes; the corrected configured oracle matches all 31 native CLI assignment/balance/cut results and twelve repeated native processes. Four cases use prepared RI/IA graphs; remaining inputs are synthetic. Matching rejection of unsupported YAML epsilon declarations passes 96 cases. The current direct API regression passes 24 cases with three option refusals.

Native YAML's CompactBisect slack is fixed at 1%, while the direct laboratory API exposes slack with a 5% default. The configured-result UI now explains that distinction and native seeds 1 through the per-split budget, regardless of the declared seed. Its actual browser Run and inert Save/Open retain the exact YAML, graph and results, including the maximum u64 declaration and short-burst-to-two-candidate budget interpretation. Configured fixtures retain 0 balance misses and 0 disconnected plans; those remain outcomes, not certification. The direct 24-case fixture has no balance/connectivity misses.

Proof-model and native proof-pipeline gates and the lab-to-workbench browser transfer pass on the rebuilt artifact `d0e896657537912d88e16fa4a25277517c24af44b0140e5be2ae0c5795bb3b65`. The earlier nationwide partisan gate remains evidence for its recorded pre-compact-fix artifact, not a new run on this artifact. This shared change is confined to CompactBisect geometry ranking; full option combinations, source acquisition and capacity parity remain unfinished. Local and unpublished.


### Flow repair and unused backend declarations

Flow construction now has an independent actual CLI reference for its native FlowConfig::new dispatch. The current gate passes 78 assignment/balance/cut comparisons, 39 matching generation failures and two unsupported YAML repair-key refusals. The direct API passes 27 exact successes and 15 matching failures across BFS/no-repair choices, weights, selected district counts and prepared RI/IA graphs. Zero cut values compare numerically with signed zero normalized; nonzero costs retain exact comparisons. Both repair modes run through actual RI browser workers with exact native assignments/summaries, inert Save/Open and scoped controls. Native YAML uses BFS repair; the laboratory can choose BFS or none. Construction uses adjacency/population; weight settings affect reported costs, and declared seed/search settings do not change flow assignments.

Configured execution no longer rejects a backend declaration that the native method does not use. Moving-knife, spectral, regionalization and flow construction retain omitted/default C-FFI or explicit C-FFI/gpmetis/metis-core declarations and invoke their shared native Rust kernels without METIS. No C-FFI/gpmetis backend runs and no backend is substituted. Metadata retains the original declaration and resolved compositor while reporting method-level METIS use and the browser execution backend. The UI labels this classification **Method requires METIS**; it is not a runtime call trace. Methods depending on METIS still require explicit metis-core, including their currently guarded one-district shortcuts.

The independent gate passes 68 native CLI/WASM results and twelve matching failures across 80 requests (48 synthetic and 32 prepared RI/IA), with 22 backend guard refusals and exact inert projects. Regionalization and flow failures remain Native outcomes; agreement does not establish acceptable plans. Actual browser Run retains default C-FFI YAML, displays the Rust/no-METIS backend and preserves source/results on Save/Open. Updated regression gates pass 61 native/CLI validation comparisons, 80 configured executions with 70 actual CLI assignment oracles, 60 template/YAML-byte matches, 31 CompactBisect CLI comparisons, proof-model/pipeline checks and both repair-mode browser runs. Current WASM SHA256: `469461d282b2330df6c6d602776141b70734a60a5b9afa0fe0cea30f6a5ffb40`. Original acquisition, alternate population bases, arbitrary capacities, all engine combinations and full desktop parity remain unfinished. Local and unpublished.


### One-district configured backend shortcut

Configured requests for one district now retain native omitted/default C-FFI or explicit C-FFI/gpmetis/metis-core declarations and use the existing all-units assignment without invoking a partitioning backend. Backend availability previously rejected these declarations before reaching that shortcut. Configuration inspection still describes the method-level requirement; a run also records the selected shortcut and its actual backend requirement. The UI shows **Method requires METIS** separately from **METIS required for this run**, **One-district shortcut** and **single-district-no-partitioning**. No backend is substituted or declared executable merely because it is unused.

The current gate passes 297 exact native CLI assignment/inert project roundtrips: eleven shared YAML structures, four backend declarations, omitted/42/max-u64 seeds, geographic/unweighted modes and prepared VT/WY graphs. Ratio-optimal-VRA fixtures include explicit synthetic fractions; graph/options/data validation still applies. A disconnected graph assigns all units together but reports failed connectivity. Nine malformed-input or multi-district backend refusal cases match the native host's errors. Actual browser Run, original YAML/declaration retention, metadata display and inert Save/Open pass. Existing 61 validation, 80 configured-execution/70 CLI oracle, 80 unused-backend and proof-model/pipeline regressions also pass. Current WASM SHA256: `5dfc88b5896b2cc28dcf423e3b94f60de679e99b8091eb703281af37ba6c62c4`. Other METIS-dependent runs still require explicit metis-core. This is backend-shortcut parity on supplied graphs, not full filesystem orchestration, complete parameter/capacity coverage or legal certification. Local and unpublished.


### Independent State AreaSection initializer controls

AreaSection warm-start and swing controls are now checked through actual State clap parsing and AlgorithmConfig::from_state_args, followed by the native AreaSection dispatcher on identical supplied adjacency, populations, areas and centroids. The independent oracle passes 90 successful assignment/balance/root-direction/swing comparisons and two matching generation failures across 92 requests. It covers ratio-optimal/moving-knife initialization, single/multi, omitted/42/max-u64 seeds, geographic/unweighted/county modes, swing 1.1/1.5, recursive counts 2/3/4 and eight prepared RI/IA graph requests. Both failures are RI single-seed sampled-root feasibility failures; no feasible plan is fabricated.

The legacy --partition-mode areasection preset resets the compositor seed to None, so native dispatch starts at 1 despite a --seed declaration. The --structure ratio-optimal-area override on the edge-weighted preset retains the declared seed. The browser laboratory uses its visible starting seed; the initializer control now explains using seed 1 to reproduce the legacy preset. Actual RI browser workers match native State assignments/directions and whole-native summaries for both initializer choices, retain controls and assignments through inert Save/Open, and hide/disable the control outside AreaSection. The existing direct gate passes sixteen comparisons, two matching invalid-option refusals and evidence tamper checks. Runtime WASM remains `5dfc88b5896b2cc28dcf423e3b94f60de679e99b8091eb703281af37ba6c62c4`; this pass changes the independent native oracle, UI help and scoped reviews, not engine dispatch.

Prepared geometry is identical between implementations. Native absent-centroid fallback, original TIGER/source acquisition, States/Run argument forwarding, arbitrary capacities and every control combination remain unproven. Root initialization does not establish a globally optimal split or legal compliance. Full desktop parity remains unfinished; local and unpublished.


### Short-burst budget and percentile controls

Three short-burst variants now have an independent native CLI YAML-dispatch gate: 228 exact assignment/balance/cut comparisons cover lengths 0/2/5, burst counts 0/3, three percentile declarations including native clamping below zero/above one, omitted/42/max-u64 seeds, geographic/unweighted modes and twelve prepared RI/IA requests. Zero work returns the initial plan; an omitted native burst seed initializes with seed 0, which is distinct from an omitted single-run seed. The direct whole-native/WASM gate passes 144 comparisons, six invalid-option refusals, evidence tamper checks and legacy step-budget fallback. Tested outputs have no balance/connectivity misses; this is not a general acceptance guarantee.

Actual RI browser workers match native CLI assignments for all three variants with lengths zero and two. Exact proposal controls and maximum u64 seeds survive Save/Open without execution; burst controls hide/disable outside their search scope and an oversized proposal product never executes. The configured-results note now explains proposal budgets, initialization, weighted initialization versus unweighted endpoint selection, and the absence of a calibrated-ensemble claim. Its browser Run and inert Save/Open retain exact source/results. WASM remains `5dfc88b5896b2cc28dcf423e3b94f60de679e99b8091eb703281af37ba6c62c4`; engine dispatch is unchanged by this pass. Full source acquisition, capacities, other percentile/refinement modes and complete desktop parity remain unfinished. Local and unpublished.


### Independent tempering ladder and final balance

The browser previously forced parallel tempering's cold-chain tolerance to equal the final requested population tolerance, losing an independent native State control. The engine/project validators now permit an independent finite cold tolerance from 0.0001 through 1, with hot at least cold and at most 1. The laboratory exposes cold tolerance separately from run balance tolerance, retains replicas/swap controls and validates the ladder/work budget. Existing equal-tolerance projects remain valid; old omitted ladder settings preserve legacy defaults when reopened.

Actual State clap parsing/AlgorithmConfig::from_state_args and native tempering dispatch match 196 WASM assignments/final-balance flags across selected zero/ten-step, one/three-replica, swap-one/seven, cold-2/10/50%, seed-42/max-u64, geographic/unweighted, percentile-zero/one and prepared RI/IA requests. Forty-two selected plans miss the independent final 5% criterion; none is disconnected. The existing 288 whole-native/WASM comparisons and nine malformed-option refusals pass. Three actual RI browser runs keep final 5% while cold changes to 2/10/50%; native assignments and final flags match, exact controls survive inert Save/Open, scoped controls disable outside tempering and an inverted ladder never executes. Cold 10/50% RI fixtures miss final balance, and the UI keeps those outcomes explicit rather than claiming acceptance.

Proof-model/pipeline and project regressions pass on current WASM `0fd56b1fd1433afd0052372ecd6341e186f7151f112a205690f063f1317712c1`. Native source acquisition, States/Run forwarding, full retry orchestration, every numeric combination, arbitrary capacities and calibrated sampling remain unproven. Matching first-attempt native dispatch does not establish complete desktop parity or legal compliance. Local and unpublished.


### SMC sampler versus final balance tolerance

SMC's native State dispatcher samples using its fixed 0.5% remaining-component proposal tolerance, then checks the selected assignment against an independently requested final population tolerance. The browser previously locked that final check to 0.5%. Engine/project validators now retain the normal final-tolerance bounds while allowing a separate SMC final request, and the laboratory leaves that control editable. Sampler settings, evidence and native selection are unchanged. Existing 0.5% projects remain valid.

Actual State clap parsing/AlgorithmConfig conversion and native SMC selection pass 114 exact assignment/final-balance comparisons across particles 8/24, ESS thresholds 0/.5/1, seed 42/max-u64, percentile 0/.5/1, final .5/5/25% and six prepared RI/IA requests. Assignments remain invariant under changes only to final tolerance; two strict-final misses remain reported, and no selected plan is disconnected. The direct 58-case gate records 49 successes/nine matching errors and sixteen constraint misses across its broader fixtures, with ledger tamper and inert project/map checks. Native State uses --particles, not a --smc-particles alias.

Actual RI browser runs retain the editable final tolerance and match native State assignments/final flags at all three final values; fixed sampler .005 evidence and assignments remain unchanged, and Save/Open executes no engine. The tempering 196-case regression and fresh proof-model/pipeline gates pass. Current WASM: `54c655a62c2192ed5fa9d5abaea40644c697d5365cc11aa9eed78c26a5723496`. Original acquisition, States/Run forwarding, full retries, every parameter combination, arbitrary capacities and distribution certification remain unproven. Full desktop parity remains active; local and unpublished.


### VRA-ReCom proposal versus final balance tolerance

Native State VRA-ReCom uses a fixed 0.5% proposal tolerance and separately checks the selected plan against the requested final balance tolerance. Browser engine/project validation previously forced the latter to 0.5%. The final control is now editable, retains normal bounds, and leaves the native proposal policy unchanged. Existing projects remain valid; complete explicit demographic inputs and preservation evidence remain required.

Actual State clap parsing and AlgorithmConfig conversion pass 70 exact native/WASM assignment and detailed preservation-report comparisons, including thresholds .5/.8, geographic/unweighted objectives, steps 0/8, seed 42/max-u64 and prepared RI/IA graphs. Changing only final tolerance preserves assignments. All selected cases satisfy their final checks and are connected. Actual RI browser runs at .5/5/25% final tolerance match native State assignments/reports, expose the separate fixed proposal tolerance and retain inert Save/Open.

The direct gate passes 432 exact comparisons, 12 matching refusals and 60 minority rollback cases. The separate 24 real-graph project gate covers six ReCom and eighteen VRASection cases, with 48 atomic tamper rejections and real verification Workers. These demographic fixtures are explicitly synthetic total-population fractions, not Census observations. The SMC 114-case regression and fresh proof-model/pipeline gates pass. Current WASM: `a1431b30d1be3c860dad3b2c42f71c00de90d18949565c003779f37b6b3d6729`. Native CSV acquisition, authentic demographic datasets, States orchestration, full retries, arbitrary capacities and legal certification remain unproven. Full desktop parity remains active; changes are local and unpublished.


### ProportionalSection State controls and root-score verification

An independent native oracle now parses actual State arguments and uses AlgorithmConfig conversion before calling the native ProportionalSection kernel. Across 108 requests, 102 assignments and root outputs match WASM exactly and six failures match native behavior. Selected eta values are 1/1.1/1.5/2, budgets 1/3, geographic/unweighted/county weighting, two exact declared seeds and prepared RI/IA graphs. Twenty-four final population tolerance misses remain reported; no successful selected plan is disconnected. Explicit supplied election counts are synthetic, not election observations.

This exposed two browser evidence-verification errors. Native ProportionalSection ranks the supplied weight map: unweighted execution has an empty map and a zero root score, independently of its actual crossing-edge count. County weights use boundary length times (1+alpha) on same-county edges when alpha exceeds 1e-10. The verifier now follows both native calculations and honors explicit graph county IDs. Engine selection and native outputs are unchanged. The 450-request direct regression passes 270 exact results/180 matching failures, 4590 tamper rejections, 15 invalid-input refusals and three exact practitioner exports.

Actual RI browser runs in all three weighting modes retain eta, synthetic election counts and the full-u64 declared seed; assignments and root scores match native State, and Save/Open remains inert. Native ProportionalSection uses the fixed one-through-budget seed schedule; a declared seed is retained for documentation but does not alter selection. Current WASM remains `a1431b30d1be3c860dad3b2c42f71c00de90d18949565c003779f37b6b3d6729`. Authentic inputs, native CSV acquisition, complete orchestration, all parameter combinations, retries, capacities and partisan outcome guarantees remain unproven. Full desktop parity remains active; changes are local and unpublished.


### Character blend State default and native weighter API

The native State CLI fixes economic/housing geographic blend alpha at 0.5. The laboratory exposes the native weighter API's full [0,1] range as additional settings. The visible input guidance now distinguishes these scopes; reproducing a State CLI run uses alpha 0.5. This does not change any engine selection or existing saved settings.

An independent oracle uses actual State parsing/AlgorithmConfig conversion, actual CLI EconomicCharacterWeighter/HousingCharacterWeighter, and native Single/Multi bisection dispatch. Sixty exact assignments match WASM across both character types, alpha 0/.5/1, k2/4, seed 42/max-u64 and prepared RI/IA graphs. Alpha .5 uses the actual State default; 0/1 explicitly override only the native weighter API. Independent browser edge formulas and weighted metrics match the native weights, including zero-vector cases. Alpha 1 also reproduces geographic assignments. No selected case misses final balance or connectivity.

Six actual RI browser runs retain the full input, blend and exact seed; assignments match the independent native oracle and Save/Open executes no engine. Observations are explicitly synthetic, not authentic LODES/ACS records. Native acquisition and missing-data fallback, other structures/searches, every blend value, full retry orchestration and arbitrary capacities remain unproven. Current WASM: `a1431b30d1be3c860dad3b2c42f71c00de90d18949565c003779f37b6b3d6729`. Full desktop parity remains active; local and unpublished.


The current broad direct character regression additionally passes 564 exact native/WASM results and weighted replays, 66 matching generation failures, 6204 evidence tamper rejections and 24 invalid-input refusals across selected structures/searches. It reports 42 final balance misses and no disconnected outputs; this gate does not include real-state project/export replays. Evidence: `target/current-character-direct.log` and `target/character-wasm-verification.json`. This wrapper regression supplements the independent State/weighter oracle without expanding its CLI acquisition or orchestration scope.


### Convergence State threshold and browser cap

Actual State conversion uses --convergence-threshold independently of --seeds, and native State invokes convergence with no seed cap. The CLI's generic seed-budget help previously implied that --seeds controlled the convergence threshold; it now accurately directs users to --convergence-threshold. Browser guidance identifies this mapping and distinguishes its optional maximum-seed setting as a native API laboratory extension. Existing settings and execution are unchanged.

An independent State parser/converter/native dispatcher oracle passes 80 exact assignments and complete stopping summaries across thresholds 1/3, no cap/1/8 attempts, geographic/unweighted/county weighting, k2/4, seed 42/max-u64 and prepared RI/IA graphs. Each CLI request deliberately declares --seeds 7; selection and stopping still follow the named threshold. Uncapped comparisons use actual State dispatch. Capped comparisons explicitly inject the native API cap. Every successful plan meets population/connectivity checks in these fixtures.

Eight real-graph direct comparisons additionally pass independent replay of every attempted candidate, unsigned-seed wrap, legacy omitted/false cap compatibility, state/national inert Save/Open and stopping-evidence tamper refusals. Actual RI browser capped/uncapped runs match independent State assignments and stopping fields. Browser default cut/one-trial refinement annotations are checked separately from the native State summary, which omits that annotation. Controls hide and disable inactive limits, and saved runs retain the choice. Current WASM: `a1431b30d1be3c860dad3b2c42f71c00de90d18949565c003779f37b6b3d6729`. Full sources, all thresholds/refinement values, retries/orchestration, arbitrary capacities and global optimality remain unproven. Full desktop parity remains active; local and unpublished.


### Shared historical demographic source reader

The native CLI's historical total-population and VAP/CVAP parsers now live in bisect-data::native_demographics as filesystem-free functions. Native file-reading adapters call those same functions. Existing native parsing remains unchanged: simple comma splitting, original header/alias precedence, short-GEOID padding, duplicate replacement, malformed-number zero defaults, total-population fraction clamping and VAP invalid-count refusal. Native alignment and missing-tract zero defaults remain in the CLI adapter. The browser's existing strict import API remains separate.

All twenty existing CLI demographic tests and four shared compatibility tests pass. The shared crate compiles for wasm32-unknown-unknown. Evidence: target/shared-native-demographics-cli-tests.log, target/shared-native-demographics-data-tests.log and target/shared-native-demographics-wasm-check.log. This establishes an in-memory source-reading boundary; it does not prove a running browser import, authentic demographic acquisition, native missing-data fallback, or complete source parity. The current packaged browser engine has not been rebuilt for this source change. Next work is explicit compatibility import with visible default/replacement diagnostics and retained original bytes, followed by native/browser source-to-run verification. Full desktop parity remains active; local and unpublished.


### Native demographic compatibility inspection in the workbench

The workbench now exposes Inspect native demographic CSV compatibility. It calls the shared historical native total-population or VAP/CVAP reader in WASM and displays its parsed records and compatibility diagnostics: numeric zero defaults, duplicate replacements, nonpositive totals, clamped fractions and non-finite native fractions. Non-finite fractions are explicitly marked rather than silently represented as usable numbers. Inspection is limited to nonempty UTF-8 sources of at most 8 MiB and 100,000 source rows. Selected original CSV bytes, requested basis, source label and inspection output are retained in saved projects; output carries the SHA-256 of exact source bytes.

Eight synthetic original-format fixtures match the actual native CLI file adapters exactly; three invalid inputs fail in both and five additional encoding/basis/label/row-budget requests are refused. Five shared Rust tests pass. An actual browser worker reproduces the native records and shows defaults/replacements/non-finite values, with exact CSV retention and inert Save/Open. Total-population inspection is separate from VAP/CVAP context attachment; the attachment flow disables that basis and still requires a plan and matching context. Existing strict import passes seventeen exact native/WASM comparisons, four prepared-RI engine/Save/Open/map cases and thirty-three malformed/count-tamper refusals. Full project security/interaction regressions and fresh proof-model/pipeline checks pass. Current WASM: `0c4ed06e0238020c136ed052594d0369e0d48927b8d3c7b64fcd3febe2326ef9`.

This is a native source inspection tool, not yet a compatibility source-to-run handoff. Graph alignment, missing-tract zero-fill policy, explicit user review before adopting defaulted observations, authentic Census acquisition and complete demographic source parity remain incomplete. Original strict import remains available. Full desktop parity remains active; local and unpublished.


### Reviewed native demographic source-to-run handoff

After running native demographic inspection in the workbench, select the intended prepared graph under configured Run. Return to inspection and choose Preview alignment to selected run graph. The preview verifies the original CSV digest, aligns fractions to the graph's exact unit order, and shows matched units, missing units that will receive the historical native zero default, ignored source records and native CSV compatibility diagnostics. A graph-matched non-finite, negative-zero or out-of-range fraction is refused. VAP/CVAP adoption retains original parsed counts, including explicit zero counts for missing graph units. Total-population adoption retains the native fraction result and original CSV rather than fabricating raw counts.

Adopt displayed fractions and zero defaults for configured Run is a separate explicit action. Only fresh locally computed inspection/preview can be adopted; imported results, changed source bytes, edited settings, changed graph, busy execution and stale preview cannot authorize adoption. Adoption selects a complete demographic input and preserves the original CSV plus a source/graph SHA-256 alignment receipt in the workbench project. It does not execute the engine. Review YAML and district count, then Run explicitly. Existing native engine restrictions still apply: VRASection and native VRA boundary weights require total-population fractions. Receipt/source integrity does not authenticate observations.

Six original-format synthetic fixture comparisons match actual native CLI file reading and native graph alignment for all three bases, with complete and sparse inputs. Two total-population configured VRASection executions match actual native YAML dispatch assignments. The actual RI browser checks fresh preview, visible missing-unit defaults, stale-label refusal, adoption without execution, subsequent native YAML assignment equality, original CSV/receipt preservation and inert Save/Open. Project security/interaction regressions pass; the staged package resolves 175 files and 589 local dependencies. The engine artifact is unchanged from native compatibility inspection. Evidence: target/native-demographic-adoption/evidence.json, target/native-demographic-adoption-browser/evidence.json, target/native-demographic-adoption-project-tests.log. Native source bytes and the alignment receipt stay in workbench projects; standalone engine-plan export currently retains the adopted operational demographic input/digest rather than this workbench receipt.

Authentic Census observations, every source selector, all state/resolution capacities, complete native fallback/retry orchestration and full desktop parity remain unproven. Native missing-unit fraction alignment is now available through this explicit handoff; it does not establish the independent native absent-file fallback. Full desktop parity remains active; local and unpublished.


### Configured plan and original-source ZIP

After a fresh configured Run and Prepare native RPLAN/RCTX, Download plan and original sources ZIP exports the exact native plan/context files alongside selected original YAML, graph JSON and operational demographic JSON. A reviewed native CSV adoption also contributes the exact original CSV and alignment receipt. bundle.json records each artifact's byte length and SHA-256. Fixed archive paths avoid user-supplied path traversal; the existing ZIP writer bounds total source bytes to 25 MiB. Native RPLAN/RCTX bytes and their hashes are unchanged. The manifest supplies package integrity and retained provenance, not source authentication or legal certification.

Export checks selected YAML/graph/demographics against the completed run and compares native exported assignments and demographic source-hash consistency. Native source adoption receipts now bind the exact derived demographic JSON bytes as well as the original CSV and graph source. Changed sources or missing/mismatched receipt binding are refused. Older receipts lacking the derived-input digest require a fresh inspection preview/adoption; they remain importable as recorded evidence. Async export snapshots sources and refuses a download if revision, busy state, freshness or selected result changes before completion.

Two native export fixture packages pass fourteen source/receipt/plan binding refusals; ordinary receipt-free configured exports remain available and omit unrelated native CSV sources. Actual RI browser download preserves original CSV/receipt through inspection, explicit adoption, configured Run and native export. Python ZIP verification independently checks CRCs, paths, byte lengths, every manifest digest and exact native/source bytes in both fixture archives and the actual browser download. Workbench project/security regressions pass. Staged toolkit: 176 files, 593 resolved dependencies. Evidence: target/configured-source-bundle/evidence.json, target/configured-source-bundle-browser.log, target/configured-source-bundle-zip-verification.log, target/configured-source-bundle-project-tests.log. Engine artifact unchanged.

The native demographic export currently stores an operational-input digest in RPLAN/RCTX source hashes, and VAP/CVAP counts in context when supplied; it does not put the complete demographic input into native producer metadata. The source ZIP supplies that full operational JSON and original-source sidecars. Standalone RPLAN/RCTX alone do not include those sidecars. Synthetic fixtures do not prove authentic Census acquisition, all source selectors/capacities, full orchestration or complete desktop parity. Full desktop parity remains active; local and unpublished.


### Open a configured plan-and-sources ZIP

The workbench's Open plan-and-sources ZIP chooser reads the stored ZIP32 format emitted by the source exporter in an isolated worker. It checks exact known member paths, matching local/central headers, CRCs, nonoverlapping extents, complete directory coverage and manifest byte hashes/lengths. Compressed, encrypted, linked, duplicate, unknown-path and hidden-byte archives are refused. File size is checked before reading: at most 32 MiB ZIP and 25 MiB member contents. JSON uses the existing prototype/duplicate-key/depth/value-count guards. Graph and native plan/context unit order, state and vintage must agree; native CSV/adoption sidecars must be paired and their raw-source/graph/derived-input hashes must agree.

Successful import selects exact RPLAN/RCTX text for audit and retains original YAML/graph/demographic inputs plus CSV/receipt where supplied. Imported results and last-operation claims are empty. YAML source name/profile come from retained native configuration metadata; original COI source name comes from native producer metadata. COI JSON is now included in configured export whenever the run uses it, is checked against the fresh request, and survives source ZIP import exactly. Source paths are fixed archive members and never become filesystem destinations. Import does not run an engine or certify native plan hashes/constraints; select an explicit audit profile and Run to perform those checks. Hashes and receipt metadata do not authenticate observations.

Three source-preserving imports, including a fresh native configured COI export, pass fourteen malformed/tampered/hostile refusals and oversized pre-read rejection. An actual worker/browser import retains exact CSV, receipt and graph, selects audit, makes zero engine requests, and preserves user edits on a stale import or corrupt ZIP rejection. Import uses the workbench archive envelope for larger reconstructed source collections and leaves prior autosave unchanged. The cache backup parser retains its existing hardcoded path policy and security regression after its stored-ZIP reader was shared. Workbench project/security tests pass. Staged toolkit: 178 files and 599 local dependencies. Engine artifact unchanged. Evidence: target/configured-source-import/evidence.json, target/configured-source-import-browser/evidence.json, target/configured-source-import-cache-regression.log, target/configured-source-import-project-tests.log.

This closes local export/reopen for the tested configured native source bundle, while authenticity, authentic Census inputs, all source/capacity combinations and full desktop parity remain unproven. Full desktop parity remains active; local and unpublished.


### VRASection State alignment weight and original CSV

The independent State VRASection oracle now parses actual State arguments, reads the original native total-population CSV through the CLI file adapter, applies native missing-tract/duplicate/numeric-default alignment, converts fractions times graph population to minority mass and executes native root-ratio selection. Across 228 requests, WASM exactly matches assignments, root ratios and native cuts; independent demographic score/evidence checks also pass. Cases include weight 0/.4/1, budgets 1/3, geographic/unweighted/county weighting, zero/stripe/sparse profiles, declared seed42/max-u64 and prepared RI/IA graphs. No selected case misses final balance/connectivity in this matrix.

Actual State conversion distinguishes the legacy --partition-mode vra-section preset, which discards the declared seed and starts at base1, from explicit --structure ratio-optimal-vra, which retains it. The laboratory now explains this distinction and the percent/fraction conversion (40% maps to --w-vra 0.4). Six actual RI browser runs match native State assignments/root ratios for explicit max-u64 and legacy base1 at all three weights, retain full demographics and settings, and keep Save/Open inert. Browser settings record the effective requested starting seed; this does not pretend the legacy preset retains its discarded declaration.

The broader current direct gate passes 576 exact native/WASM results, nine matching refusals and sixteen ledger tamper checks. It exercises 128 negative adjusted-score cases and 96 uniform-fraction alignment cases; 33 final population misses remain explicit, with no disconnected outputs. Current WASM: `0c4ed06e0238020c136ed052594d0369e0d48927b8d3c7b64fcd3febe2326ef9`. Demographic sources are explicitly synthetic, not Census observations. Absent-file fallback, authentic acquisition, every value/structure/search, full retry/orchestration and arbitrary capacities remain unproven. Full desktop parity remains active; local and unpublished.


### Explicit native VRASection absent-file fallback

The native State VRASection dispatcher uses plain GeoSection when its demographic source file is absent. The laboratory now exposes an explicit No demographic file: use native plain GeoSection fallback choice. This does not infer absence from an unreadable or incomplete source. With the choice enabled, demographic input is omitted, the alignment weight control is disabled while its requested value remains retained, and the engine executes the same population-only GeoSection branch. Evidence records the source-policy declaration, resolved structure and alignment_bonus_used=false. It does not fabricate zero observations or claim a demographic bonus. Normal VRASection still requires complete explicit inputs when the choice is cleared. Minority boundary weighting disables and clears this incompatible fallback choice.

The optional vra_absent_file_fallback=true engine field applies only to VRASection with a non-demographic objective and no supplied demographic input. Wrong method/objective, explicit false and contradictory demographic data are refused. Project verification requires matching fallback evidence and refuses an altered bonus claim. Save/Open retains the explicit declaration and requested weight without running generation. Native plan export retains the engine option as recorded provenance. A declared absence is not proof of filesystem state on another device.

Thirty-six independent actual State-conversion/native absent-mass dispatch comparisons exactly match WASM and plain GeoSection across RI/IA, geographic/unweighted/county objectives, Single/Multi and weight0/.4/1. Actual RI browser controls, native assignment equality, inert Save/Open and the restored data requirement on uncheck pass. The supplied-data 576-case VRASection regression remains green; its 33 final misses remain native outcomes. Project security/interaction regressions and fresh proof-model/pipeline gates pass. Current WASM: `f9d0cce0554d1e8be4f859a371c15927b6488b7a41f3d850cea11204d42c670a`. Engine declaration inventory now tracks 228 fields (42 WASM options). Browser native-file discovery, actual native path lookup, authentic observations, every tuning/weighting combination, full retry/orchestration and arbitrary capacities remain unproven. Full desktop parity remains active; local and unpublished.


### Configured YAML VRASection absent-file workflow

Configured YAML runs now accept the explicit VRASection absent-file choice independently of the laboratory. Configured run/build preparation forwards the choice to engine options while preserving the original YAML and requested alignment weight. It requires VRASection with no demographic input; contradictory selected demographics or the wrong method are refused. The workbench offers an explicit Clear selected run demographics action and records the fallback choice in run settings. Changing the choice invalidates prior native exports. It does not modify the YAML to pretend that it requested another structure.

The selected-graph native population retry workflow retains the choice and exports the accepted attempt with the same declared absence. Native RPLAN producer options retain it. Source ZIP import restores it from native metadata. An inactive old demographic CSV/receipt is omitted from an absent-source run bundle rather than represented as an active input; original workbench source evidence remains retained. Engine-plan export also refuses a contradictory fallback-plus-demographic input.

Six independent native State absent-source comparisons pass for configured Run/build across weights0/.4/1. The tested build cases accept attempt0; this verifies choice propagation through the build boundary, not every retry outcome. Accepted-attempt native export and ZIP import retain the choice. Actual browser Run/build, native export, inert source ZIP reopening and edit invalidation pass. Forty-nine supplied-data native CLI comparisons, workbench project/security tests and fresh proof-model/pipeline gates pass. Current WASM: `eabe485865eab58500d5d650f6120b7d5dd00844ef88219932a142e76e144086`. Batch label-build defaults remain unchanged; this flag is not yet exposed as a per-job national/label policy. Native path discovery, authentic sources, all retries, full orchestration/capacities and complete desktop parity remain unproven. Full desktop parity remains active; local and unpublished.


### Per-job VRASection absence policy in label/state builds

Prepared label-build jobs can now carry vra_absent_file_fallback=true individually. A job with supplied demographics cannot declare absence. Each job forwards its policy through configured native build and accepted-attempt export; native plan producer options and fallback execution evidence retain it. The label checkpoint request identity includes true choices, so changing a job's policy changes the identity. Omitted/false native API choices keep the previous normalized identity behavior; the browser job editor emits true or removes the field.

The workbench lists retained state/year jobs and their source policy. Select one or more, choose Require demographic input or Explicit absent-file GeoSection fallback, then Apply to selected jobs. This edits retained prepared-jobs JSON without running the engine. An edit that would replace supplied demographics with absence fails atomically for the whole selection. Policy is per selected job, allowing supplied and absent sources in the same multi-state/year build. It is only executable for VRASection; other YAML methods refuse the incompatible source policy through existing configured validation.

A mixed real prepared RI/IA build passes exact native-host/WASM output comparison: RI uses explicit absence and IA supplied synthetic fractions. Per-state native plan provenance, changed checkpoint identity, atomic conflicting-edit refusal and saved jobs preservation pass. An actual browser worker repeats the mixed build, retains choices on inert Save/Open and makes no engine request during editing. The legacy two-year capture regression passes against the current release native host, including capture-on/off assignments, no-event single-district metadata, accepted-attempt binding and archives. Its previous debug-host path pointed to an outdated native artifact; the gate now defaults to the current release host, with BISECT_NATIVE_HOST override. Workbench project/security and fresh proof-model/pipeline gates pass. Current WASM: `946ed68b2b6a65bdde83f2101998dba5516309dfa4c92ae03d59e7d78b8efe03`.

Tests establish a mixed selected-state/year policy boundary, not all fifty states or every year/capacity/control combination. Supplied demographic observations are synthetic. Native filesystem discovery, authentic acquisition, all retries and complete native orchestration remain unproven. Full desktop parity remains active; local and unpublished.


### Laboratory balance-tolerance bounds match native execution

Native State bounds and native YAML/WASM execution accept final balance tolerance from 0.01 through 50 percent. Laboratory HTML controls and saved-project validation still capped the value at 25 percent. Both now accept the same native range, retaining actual final-balance checks and refusing out-of-range values. No engine implementation or artifact was changed for this correction.

Twelve actual native YAML dispatch assignment/final-check comparisons pass for standard Single/Multi on prepared RI/IA graphs at25/35/50 percent. State and two-state national projects retain 50 percent on inert Save/Open. Actual RI browser runs at35/50 percent match native assignments and retain the requested tolerance; 50.01 is refused without submitting an engine run. Evidence: target/laboratory-tolerance-range/evidence.json and target/laboratory-tolerance-browser/evidence.json. This does not establish all algorithm/search combinations or full capacities.

The source investigation also reconfirmed the separate native population-source gap: native State check_cvap_availability resolves a requested source, but its result is assigned to _effective_population_source and does not replace loaded graph vertex weights. The requested string is recorded in metadata. The browser therefore does not expose a basis selector that would misleadingly imply native vector switching. Explicit supplied graph populations remain the operational input. A functioning native/WASM alternate-population selector would require an explicit shared vector-selection contract and source alignment, beyond reproducing the current metadata-only flag. Full desktop parity remains active; local and unpublished.


### Source ZIP execution-settings retention

Configured source ZIPs now include sources/run-settings.json with a typed bisect-configured-run-recipe-v1 record. It retains the requested chamber/district count, native-build choice, round-capture choice, explicit absent-file policy and seats-per-district/total-seats inputs, plus the recorded accepted build attempt. Export and import compare those settings with native RPLAN producer metadata. This fixes source ZIP imports previously dropping capture/build/electoral choices while retaining graph and YAML bytes. Native RPLAN/RCTX remain unchanged. The settings member is hashed like every other ZIP member.

Reopening remains inert and selects audit; users explicitly choose Run to regenerate. New packages restore exact requested settings. Older packages without the settings member remain supported and recover observable build/capture/electoral choices from native metadata; original omitted-versus-explicit-default spelling cannot be recovered. The stored ZIP path policy now admits this fixed settings path and at most ten known members. It does not accept arbitrary archive paths or scripts.

Twelve combinations pass exact settings restoration and explicit replay comparisons of assignments, engine options and electoral inputs. Cases cover build/capture on/off, explicit absent-file fallback and default/3-seats-total7/zero electoral inputs. Forty-eight changed-setting exports and twelve rehashed inconsistent-recipe imports are refused. Actual browser ZIP import restores controls without an engine request, then explicit rerun uses build/capture/absence and requested seats3/total7 with capture/build outputs present. Existing source-import, cache ZIP and independent ZIP byte/hash/CRC gates pass. Staged toolkit: 179 files and 602 local dependencies. Engine artifact unchanged. Evidence: target/configured-run-recipe/evidence.json, target/configured-run-recipe-browser/evidence.json, target/configured-run-recipe-import-regression.log, target/configured-run-recipe-cache-regression.log, target/configured-run-recipe-zip-verification.log.

Recipe/native metadata consistency and byte hashes do not authenticate generation or observations. Tested builds accept attempt0; later retry outcomes, every algorithm/control/capacity, identical export timestamps, cross-version replay and full desktop parity remain unproven. Native electoral inputs remain recorded metadata, not variable-magnitude district generation. Full desktop parity remains active; local and unpublished.


### Measured configured-run and export engine identities

Every engine worker result now carries the artifactIdentity measured by instantiateEngine over the same private WASM byte snapshot that was compiled. The workbench records a configured run's measured engine descriptor independently of the engine descriptor used for its subsequent native export. Native TUI Run performs build/export in one worker operation and records the same measured descriptor for both. A fresh configured result without a measured descriptor clears any old imported identity rather than inheriting it. No native Rust result or RPLAN/RCTX bytes are modified.

Projects retain typed configuredRuntimeArtifacts records, and source ZIPs include an optional fixed runtime/engine-artifacts.json member with the usual manifest hash. ZIP readers allow at most eleven known members. Import remains inert and preserves descriptors as recorded claims; it never fetches or executes a recorded engine. Visible guidance compares the recorded run hash/byte length with the currently loaded engine and distinguishes missing identity, matching bytes and differing bytes. These fields do not grant freshness or permission to export imported results. Run/export identities may differ if the engine changed between those operations; the run descriptor is preserved rather than silently relabeled.

Node tests match the descriptor to independently hashed compiled WASM bytes, retain it through source ZIP import, preserve exact native output bytes and refuse four malformed/incomplete identity cases. Actual browser worker tests verify separate run/export identities, source ZIP/project retention, zero execution during import, visible mismatch without export authority and native TUI combined run/export identity. Project/security, source ZIP and cache ZIP regressions pass. Measured artifact: `946ed68b2b6a65bdde83f2101998dba5516309dfa4c92ae03d59e7d78b8efe03`. Staged toolkit: 180 files and 606 local dependencies. Engine binary unchanged.

WASM byte identity does not authenticate generation or observations, pin JavaScript/browser/import behavior, or guarantee cross-version deterministic replay. Older exports without a descriptor remain supported as unrecorded identity. All algorithms/capacities/source workflows and complete desktop parity remain unproven. Full desktop parity remains active; local and unpublished.


### County strength controls and native dispatch parity

The laboratory county-strength input accepts arbitrary fractions in its existing [0,100] bounds. The former HTML step of0.25 prevented valid values such as0.125 or1e-11. Native county weighting leaves physical boundary weights unchanged at alpha <=1e-10; otherwise same-county edges use boundary length times(1+alpha), with county identity derived from the first five GEOID digits.

Native State has two distinct dispatch paths: --alpha-county on the geographic baseline preserves sub-one values, while --weights-override county clamps them to at least1. The laboratory uses the raw native weighter/YAML semantics and explains the CLI distinction in its control help. The independent actual State oracle checks both paths rather than silently changing the requested browser strength.

Saved WASM results now record the actual county alpha in effective_config. Previously the static native-catalog normalizer incorrectly clamped that summary to1 even though requested_options, recorded_options and weighting_evidence correctly retained the fractional strength. Native static catalog matching keeps its clamp. Older projects with that historical summary remain importable; requested/recorded values and graph-derived weighting evidence continue to be checked. Saved project contents remain recorded evidence, not execution authentication.

Evidence:62 exact native State assignment/edge-formula comparisons, including48 two-county grid cases, six explicit county-override probes and eight prepared RI/IA cases. All62 pass final tolerance and contiguity in this selected corpus. Four actual RI browser runs at0.125,1e-11,1.1e-10 and100 match native State baseline assignments, preserve exact effective strength and pass inert Save/Open. Sixteen direct native/WASM and actual native-weighter comparisons also pass, including overflow rejection/recovery. Metadata checks cover exact new summaries, historical imports and inconsistent summary/weighting evidence refusal. The project regression suite and toolkit package dependency check pass. Sources: scripts/web/test_county_state_oracle.mjs, scripts/web/test_county_strength_browser.mjs, scripts/web/test_county_effective_config.mjs and scripts/web/test_county_weights_wasm.mjs.

This closes the scoped county-strength control and metadata gap. It does not establish every structure/search, non-tract identity, all-state capacity or complete desktop orchestration. Engine WASM bytes are unchanged; full parity remains unfinished. Changes are local and unpublished.


### Laboratory native intermediate-round capture

The laboratory has an opt-in Capture native intermediate rounds setting. It passes the existing engine option directly, preserves the returned native round_capture in each completed state's metrics, and saves it with the project. The Native rounds download exports that state's exact capture JSON. The results panel states how many snapshots were emitted and whether the last snapshot exactly matches returned labels. Methods that emit no events display that explicitly; an empty capture does not establish a complete history. Events can be candidates or partial regions, and a matching terminal event does not certify lineage.

Project opening verifies retained trace source hashes, the prepared graph's exact unit order, sequential events, coverage/region counts, declared final-match flags, and the trace's terminal comparison. It retains existing worker verification, cancellation/staleness checks and atomic import. It runs no engine on Open. Capture uses the engine's existing128-event/two-million-entry bounds; ordinary project files retain their25 MiB bound. A trace that exceeds project limits must not be described as successfully saved.

Current evidence: six exact native-host/WASM results for RI/IA, covering recursive Multi, no-events one-district runs and accepted native balance-build attempts. Capture on/off preserves actual assignments and metrics. Six inert project roundtrips and36 hostile trace refusals pass. The actual RI browser checks forwarding, exact download, retained Save/Open, hostile import preserving current trace, explicit no-events display and disabled capture producing no retained trace. Sources: scripts/web/test_laboratory_round_capture.mjs and scripts/web/test_laboratory_round_capture_browser.mjs, with evidence under target/laboratory-round-capture and target/laboratory-round-capture-browser. The project regression suite passes; staged toolkit resolves181 files and610 dependencies. No engine WASM bytes changed.

Remaining: laboratory map exploration of individual snapshots and multiscale capture are not yet exposed. Multiscale capture requests are explicitly refused instead of silently dropping the option. Configured/batch toolkit capture and round-map paths remain available. These checks do not prove every structure/search callsite, full lineage or execution authenticity. Full desktop parity remains active; local and unpublished.


### Captured snapshots on the laboratory map

The Map assignment picker now switches between returned final assignments and individual captured native snapshots. It preserves native event order/names, identifies complete versus partial coverage and colors snapshot region IDs through the exact capture index-to-GEOID companion. Uncovered units remain neutral. Snapshot hover labels and region selection use snapshot IDs; final-plan population/deviation metrics are not attributed to snapshots. A note above the metric strip states that those metrics describe the returned final plan.

Snapshot GeoJSON exports the currently selected snapshot with event metadata and neutral null labels for uncovered units. Final GeoJSON continues to export the returned plan. Choosing snapshots never changes the saved assignments, metrics, trace or prepared geometry and never runs an engine. Returning to final assignments restores ordinary color controls. New runs and state/history changes clear the prior selection; national overview continues to show final results, with individual snapshots explored in each state's district view. Region legend buttons show at most50 labels; the map and exported snapshot preserve every captured region.

Tests verify exact native captured index-to-GEOID mapping, unchanged source project/geometry, invalid selection refusal and neutral uncovered units. The actual RI browser verifies snapshot/final exports independently, unchanged saved experiment/assignments, hover labels without final district population, fresh-run picker population and state reset. Partial coverage uses an explicitly labeled synthetic imported fixture on real RI geometry, with125 units neutral; this is not evidence that the native RI run emitted a partial event. Native full-event mapping is verified separately against the actual capture. The partial view was rendered and visually inspected (target/laboratory-snapshot-maps-browser/partial-snapshot.png). Sources: scripts/web/test_laboratory_snapshot_maps.mjs and scripts/web/test_laboratory_snapshot_maps_browser.mjs. The prior capture browser gate and project regression suite pass.

This exposes laboratory round-map exploration previously listed as unfinished. Multiscale capture and complete all-method trace lineage remain unproven/unavailable as described above. No WASM engine artifact changed. Full desktop parity remains active; these changes remain local and unpublished.


### Native omitted seed in laboratory Single/Multi runs

The laboratory now accepts a blank Run seed for standard bisection Single/Multi. It sends and saves null, preserving the native omitted-seed behavior instead of converting it to zero. Single uses backend-default seeds at each recursive node; Multi uses the native default base. The header documents Run seed native default. Explicit0 remains distinct in the request and header. Exact u64 integer/string transport is unchanged.

Native population-retry validation now handles an omitted first seed and subsequent Some(1), Some(2), etc. An accepted initial omitted seed remains null in recorded options; a later accepted attempt records its actual numeric seed. A report that claims initial0 for an originally omitted seed is refused. Native default is not an execution-authentication guarantee; engine/browser version and input scope still matter.

Eight current-artifact native-host/WASM results and independently parsed native YAML assignment comparisons pass across RI/IA Single/Multi with and without native balance retries. Eight inert project roundtrips preserve the requested null and accepted options. Actual RI browser tests verify blank/null forwarding, Single's backend-default evidence, visible default, Single/Multi inert Save/Open and explicit-zero distinction. Current exact-u64 regression and three real-state project/native-export roundtrips also pass; its old assertion classifying null as an invalid integer was corrected to check valid native optional behavior separately. Sources/evidence: scripts/web/test_optional_laboratory_seed.mjs, scripts/web/test_optional_laboratory_seed_browser.mjs, scripts/web/test_u64_seed.mjs and target/optional-laboratory-seed, target/optional-laboratory-seed-browser, target/u64-seed-verification.json. Project regressions and package dependency checks pass. No WASM engine artifact changed.

Remaining: optional-seed exposure for other laboratory structures/searches needs their specific native-default evidence checks; blank values for those paths are explicitly refused for now. Configured/native API optional-seed support remains available with prior scoped checks. The multiscale capture investigation confirmed a separate issue: shared native initialization runs on tracts even when output is block groups. Captures must identify that universe rather than bind initialization labels to the fine output. Laboratory multiscale capture remains explicitly unavailable pending that implementation. Full desktop parity remains active; local and unpublished.


### Omitted seeds across laboratory methods

Blank Run seed now works across all37 currently declared laboratory structure/search combinations. It retains the null requested seed through projects, requests and imports. Validators compare sampler/ratio evidence with the engine-resolved default (0 for ordinary sampler paths,1 for ratio methods), while preserving genuinely optional native backend/compositor declarations. Election evidence preserves the native default0 ledger. Multiscale adaptation retains an accepted omitted retry seed as null instead of sending it through integer parsing; later attempts keep their actual numeric seeds. The UI labels the choice native default and distinguishes explicit0.

The current-artifact gate covers74 native-host/WASM pairs on prepared RI/IA inputs, one for every declared structure/search combination in each state. It finds67 exact full results and inert project roundtrips, with altered requested/recorded seed refusals. Seven pairs fail identically in native and WASM: regionalization on both states, capacity clustering/flow on IA, prime-factor research-balance refusal on both, and sampled AreaSection root failure on RI. Three successful results miss requested final tolerance. No fallback or automatic method substitution was added. Demographic/election-dependent paths use explicitly synthetic uniform inputs, so this does not establish real-data VRA/election suitability. Multiscale fixed/adaptive checks use tract-to-county resolution and native balance build.

Actual RI browser tests check ratio default1, convergence default0 and multiscale balance-build's omitted first seed. Worker results match direct WASM assignments and inert Save/Open retains the blank request; native generation history remains a recorded claim. The independent native YAML seed oracle was refreshed to the current portable release reference:56 native dispatcher/WASM comparisons pass, including11 distinctions between omitted and explicit0, exact u64/export checks and10 boundary rejections. Standard Single/Multi optional-seed and project regressions also pass. Staged toolkit resolves181 files/611 dependencies. Evidence: target/optional-seed-methods/evidence.json, target/optional-seed-methods-browser/evidence.json and target/optional-seed-methods-native-yaml-regression.log; sources scripts/web/test_optional_seed_methods.mjs, scripts/web/test_optional_seed_methods_browser.mjs and scripts/web/test_optional_seed_wasm.mjs.

This supersedes the earlier laboratory restriction to Standard Single/Multi. It does not establish every weight/resolution/seed/default outcome, complete lineage, authentic execution or all-state capacities. Multiscale round capture remains a separate unresolved universe boundary. Engine WASM bytes are unchanged; full desktop parity remains active. Local and unpublished.


### Zero-work budgets and laboratory search transitions

Native zero-work settings already execute through the WASM boundary: standalone chains/local ensemble/VRA can request zero proposals, explicit parallel tempering can request zero tempering steps, burst controls can request zero bursts/length, and multiscale can request zero steps. Their native initialization/selection behavior remains intact; zero work does not mean no initialization or universally identical initial plans.

The laboratory had a form transition bug: after a valid zero-work run, switching methods could leave a hidden seed/step budget at0 even though the next request's native bound was positive. This blocked submission through an invisible invalid control. updateControls now normalizes only hidden zero fields to their new positive minimum. Visible zero-work settings remain0 where supported; visible invalid inputs still require correction. Existing saved runs are immutable and retain their original0 budgets.

Sixteen exact native-host/WASM results and inert project roundtrips pass across prepared RI/IA: forest ReCom, merge-split, flip, local bisection ensemble, VRA ReCom (explicit synthetic uniform fractions), parallel tempering, short burst and tract-to-county multiscale. Actual RI browser checks cover chain-to-Multi hidden steps, tempering-to-chain hidden seeds and multiscale-to-Single hidden steps. Each retains the original zero-work result on Save/Open, normalizes the inactive field to1 and submits a real worker request with assignments matching direct WASM. Source/evidence: scripts/web/test_zero_work_laboratory.mjs, scripts/web/test_zero_work_laboratory_browser.mjs, target/zero-work-laboratory/evidence.json and target/zero-work-laboratory-browser/evidence.json. Project regression suite and181-file/611-dependency toolkit package checks pass. WASM engine bytes did not change.

This is a scoped budget/UI correction, not proof that every seed/step minimum, maximum, default and native CLI/YAML dispatch branch is covered. Native search budgets retain distinct meanings; every algorithm outcome, all-state capacity, complete orchestration and multiscale capture remain unfinished. Local and unpublished; full parity remains active.


### All-state county-weighting native oracle

All50 prepared2020 state graphs:200 independent native State comparisons, 193 exact assignment matches, 7 matching native generation failures, 4 successful final-tolerance misses, 0 disconnected results. The corpus covers84208 prepared tracts, 330759736 supplied population and435 congressional districts, with24 one-district comparisons. Each published graph asset is SHA-256 checked before use.

The oracle parses actual native State arguments, builds native county weights and invokes standard/GeoSection native Single/Multi kernels independently of the WASM wrapper. Single requests omit the seed and use alpha0.125 on the geographic baseline; Multi requests use max-u64 seed, alpha100 and three candidates. Both use10 refinement iterations and5 percent final tolerance. Exact assignments, native weighted edge formulas/cut, final population checks and independent project assignment checks are compared. Matching generation failures remain failures; successful final-tolerance misses are recorded instead of relabeled as valid plans. One-district shortcuts remain included.

Evidence: scripts/web/test_county_all_states.mjs and target/county-all-states/evidence.json, artifact SHA 946ed68b2b6a65bdde83f2101998dba5516309dfa4c92ae03d59e7d78b8efe03. This expands two-state county evidence to the current50-state prepared tract corpus. It does not establish all options/weights/searches/resolutions/years/chambers, full capacities, all50 browser project roundtrips, legal certification or globally optimal cuts. No engine artifact or deployment changed. Full desktop parity remains active; local and unpublished.


### Multiscale tract initialization capture and block-group projection

Multiscale execution now honors capture_rounds instead of dropping it. It observes the existing shared native intermediate calls inside tract-level METIS initialization. Every multiscale capture includes unit_universe: tract / multiscale-initialization, the returned tract or BG resolution, and an explicit final-comparison policy. Native kernels do not currently emit snapshots for subsequent fine/coarse proposals. These snapshots therefore do not imply a complete sampling history or certified lineage.

For BG output, the native index companion remains the exact tract order. Final matching compares the initialization tract label against every returned BG of that parent. A split tract cannot be collapsed to the last child to claim equality. Laboratory import independently verifies universe/stage, source hashes, coverage and this comparison. Snapshot maps project parent labels onto supplied BG geometry and label that projection explicitly. Snapshot GeoJSON carries the universe metadata. Native balance builds retain only the accepted attempt's trace, with the existing report. Capture failures still reject execution, with scoped cleanup and no truncated trace.

The block-group map adapter also now derives state/year and per-feature state/county display metadata from validated run identity/GEOIDs on its cloned output. Minimal valid supplied geometry may omit those display properties; the source geometry and its identity remain unchanged. Accepted omitted seeds display native default instead of the implementation value null.

Evidence:72 exact native-host/WASM results across tract-to-county, BG-to-tract and BG-to-county, fixed/adaptive sampling, run/build, omitted/42/max-u64 seeds, and0/5 sampling steps. Each trace exactly matches the ordinary native tract initializer, with only the explicitly defined final-label comparison differing. Forty-eight cases project to BG output. Capture on/off leaves returned assignments, populations, sampling evidence and retry reports unchanged. One-district multiscale initialization emits the native depth snapshot; ordinary engine shortcuts can instead emit none. Failure cleanup is checked.

Sixteen laboratory native/project roundtrips cover tract/BG fixed/adaptive run/build and0/5 steps. Forty-eight universe/match mutations are refused; an additional split-parent fixture proves that matching only the last BG cannot claim equality. Actual RI browser checks exact projected maps/export, visible universe, derived display metadata, fresh worker request/result, altered-universe refusal preserving current map, and inert Save/Open. Fine graphs/duplicated geometry in these RI browser cases are explicitly synthetic; this does not establish real BG-boundary provenance. The projected view was rendered and inspected. Sources: scripts/web/test_multiscale_round_capture.mjs, scripts/web/test_multiscale_capture_projects.mjs and scripts/web/test_multiscale_capture_browser.mjs. Evidence resides in matching target directories.

Fresh proof-model/native proof-pipeline checks match the new engine artifact, along with ordinary round-capture/project regressions,46 multiscale seed/replay cases, and the181-file/611-dependency toolkit package. The older200-case all-state county oracle remains scoped to its recorded older artifact; it is not silently relabeled as a new-artifact run. This supersedes the earlier multiscale capture refusal, while all-search trace coverage, later-proposal lineage, full capacities and complete desktop parity remain unfinished. Local and unpublished.

Engine artifact SHA-256: 31b0d47631e07c26157b97d23d06f82ea5e2e6a150dedb90eca034cb5c72e7ce.


### Research chain names and fractional diagnostic thresholds

Research diagnostics now accept plain UTF-8 JSONL basenames with spaces, Unicode and leading dots, matching native directory loading for the checked corpus. Earlier browser/Rust adapters restricted names to ASCII. Exact selected names and bytes remain retained in projects and source hashes. JavaScript and Rust reject slash/backslash/drive separators, control characters, invalid Unicode, bare .jsonl and non-lowercase extensions. The existing120-byte filename bound remains; both adapters now count UTF-8 bytes in names against the3 MiB aggregate source bound. Names remain data, not executable paths, and no uploaded chain is extracted to disk by the browser.

The R-hat and minimum ESS controls use step=any, matching the native finite f64 thresholds rather than falsely flagging arbitrary fractional values as invalid. Positive R-hat and nonnegative ESS request validation remain in force. Strict failure still retains the native report for inspection/download, and imported projects require a fresh explicit Run before exporting.

Four exact native CLI report/exit comparisons pass with spaced, accented, Japanese and hidden filenames, fractional R-hat/ESS thresholds and strict/non-strict outcomes. Native-host/WASM complete results, byte-exact report text and project settings/source/result roundtrips pass. Ten malformed path/control/Unicode/extension/UTF-8-length cases are refused before upload reads and by the runtime boundary. Actual browser checks cover real worker execution, retained names/bytes, arbitrary fractional controls, exact native report download, stale export invalidation, inert Save/Open and a spaced Japanese file-picker name. Sources/evidence: scripts/web/test_research_names_thresholds.mjs, scripts/web/test_research_names_browser.mjs, target/research-names-thresholds/evidence.json and target/research-names-browser/evidence.json. Legacy strict/project regressions, staged181-file/611-dependency package checks and fresh proof-model/native proof-pipeline gates pass.

The32-chain,5,000-step-per-chain,3 MiB aggregate and200-million autocorrelation-work bounds remain. This is a selected filename/threshold corpus, not every filesystem name/order/platform or arbitrary diagnostic capacity. Native sampling provenance, MCMC generation and partition-space Hamming diagnostics remain outside this operation's proved scope. Prior engine/county/capture evidence keeps its recorded artifact identities; it is not relabeled as a run of the new artifact. Full desktop parity remains active. Local and unpublished.

Engine artifact SHA-256: 70847b9146125d0c0058dbd8c61dbb982bd6b68a073571c59797430ade8a75a6.


### Native partition Hamming diagnostics in the toolkit

The toolkit exposes diagnose-partition-traces using the existing bisect-analysis::ensemble_diagnostics::hamming_records kernel. This is a native library capability; the current research CLI validate-ensemble command does not itself accept partition trajectories. The new browser operation supplies a typed in-memory adapter and UI rather than claiming a new native CLI subcommand already existed.

Input JSON has schema_version bisect-partition-traces-v1, a unique ordered unit_ids roster, and chains containing arrays of partition label arrays. Each partition covers that exact roster in the same order. Labels accept unsigned integers and canonical decimal u64 strings. Global compact indexing preserves every label equality across all chains/steps, enabling full-u64 values on the32-bit WASM target without truncation. Unit metadata is an explicit alignment claim; this operation does not check valid district plans. The result records source-byte SHA-256, chain lengths, requested lag, native records and byte-exact native report JSON.

The UI loads local trace JSON, sets maximum lag, runs explicitly and displays per-chain native integrated time plus mean fraction of changed labels by lag. The table displays the first500 lag rows; the native JSON download retains the full report. District renumbering changes these labeled distances. Native tau_int uses the library's convention based on1 minus distance; this is not an independent certification of mixing, sampling authenticity, optimality or legal correctness. Saved projects retain source/settings/results. Open does not execute; lag/source changes and Clear invalidate exports. Clear retires pending reads, and source loads use revision/busy guards.

Sixteen direct64-bit native-kernel / native-host / WASM comparisons pass for constant, fully renumbered, partial-change and full-u64 traces at0/1/2/10 lags. They verify native records, exact report text, source hashes and project roundtrips. Nine malformed roster/dimension/label/source/lag cases are refused. Actual browser checks verify explicit worker execution, rendered tables, exact report download, lag edit freshness, original local source bytes, inert Save/Open and Clear without execution. Sources: scripts/web/test_partition_diagnostics.mjs and scripts/web/test_partition_diagnostics_browser.mjs; the independent probe is crates/bisect-analysis/examples/hamming_reference.rs. Fresh proof-model/native proof-pipeline checks match the artifact, and project/package checks pass (182 files/614 dependencies).

Bounds remain explicit:3 MiB source,1–32 chains,1–5,000 partitions per chain,1–100,000 ordered units, up to100,000 requested lags and200 million estimated label comparisons. Native returned lag range is retained unchanged. This is selected library-kernel evidence, not full arbitrary trajectory capacity or MCMC generation. Prior workflow/artifact evidence retains its own identities. Full desktop parity remains active. Local and unpublished.

Engine artifact SHA-256: ba9e1b456b4829261cddf7b8f784ac4c213296d67812231bc4f58e8c84753e41.


### Native ILP dispatch in WASM and the laboratory

The engine/laboratory expose structure ilp with formulation-only, branch-and-cut and iterative-separation methods, declared metis/error fallback, native time-limit/gap and maximum-node-unit settings. Execution calls the existing shared run_all_splits_ilp function. The native State oracle uses --partition-mode ilp; this is not presented as proof that native YAML structure parsing supports the same spelling.

The solver implementation is ahead of the former runner comments: bounded branch-and-cut can return local two-district assignments for nodes up to24 units. Formulation-only/unsupported nodes can return no plan; the declared fallback either invokes native unweighted METIS with10 refinement iterations/backend-default seeds or returns the native error. Requested boundary weights still affect reported cost but do not govern this dispatch's construction. Requested seed/iteration controls are disabled in the laboratory and retained as inactive declarations. Effective summary records backend default/zero sampling budget/fallback10 iterations; structure evidence discloses per-node report delivery is not captured and proof status is not-certified. Whole-plan execution/optimality remain heuristic/unproved, even if some local nodes were solved exactly.

Native branch-and-cut timing used std::time::Instant, which traps on wasm32-unknown-unknown. The shared solver now uses an explicit browser monotonic-millisecond import on that target, retaining native Instant on other targets. The loader supplies performance.now and only admits the named import in addition to existing entropy/progress callbacks. This records real elapsed time; it does not implement a new solver cutoff. Native ILP declared time-limit remains recorded but is not used as a wall-clock cutoff by the current dispatch. Existing browser run timeout/Cancel still applies. Full-u64 declared time values use exact numeric/string transport.

Evidence:45 exact native State assignments and27 matching native failures for three methods, metis/error fallbacks, node guards0/18/100 and1/2/3/4 districts on a synthetic36-unit graph. Eight invalid/foreign settings are refused. Alternate requested weights/seed/iterations leave assignments unchanged as in native dispatch. One-district shortcuts have no solver evidence. Six RI/IA runs and inert project roundtrips pass with altered-evidence refusal. Actual RI browser checks all three methods, exact requests/assignments, inert Save/Open, no exactness claim and visible fallback error. Sources: scripts/web/test_native_ilp_dispatch.mjs, crates/bisect-cli/examples/ilp_state_reference.rs, scripts/web/test_ilp_laboratory.mjs and scripts/web/test_ilp_laboratory_browser.mjs; evidence resides in matching target directories. Fresh proof-model/native proof-pipeline checks match the artifact. Project and183-file/617-dependency package checks pass. The declaration inventory now tracks233 fields (47 WASM options).

Remaining: per-node native LP/solve/certificate artifacts are not delivered by this engine path, and global optimality is not certified. Native YAML/State filesystem report workflows, other backend choices, complete capacities and orchestration remain unproven. The zone-membership and target-MM declarations inspected during this audit are not grounds for inventing operational browser controls: current State conversion declares zone flags, but runtime use was not established; target-MM runtime use likewise was not found in inspected State dispatch. Full desktop parity remains active. Local and unpublished; prior artifact evidence keeps its original hashes.

Engine artifact SHA-256: a56e468f54a3e2e4587422d2be49c0f7e555bbb9cc1061fa2f7d905d46727427.


### Correct ILP node population-tolerance contract

Per-node report review exposed a shared native dispatcher bug: run_all_splits_ilp computes the METIS multiplier1+t/node.k and passed that same number into ILP formulation/solver/LP APIs, whose argument is a relative tolerance fraction. For a10% requested final tolerance and a two-district root, the intended ILP fraction is0.05; passing1.05 effectively removes useful population balance constraints.

The observed counterexample is a4x4 grid of100-population units. Both native State dispatch and WASM branch-and-cut/iterative-separation returned1,500/100 population splits and a two-edge cut, failing the requested tolerance. After correction, both return800/800 and the balanced minimum four-edge cut. The shared tree dispatcher now supplies the relative fraction t/node.k to ILP model/solver/export and keeps1+t/node.k for the native METIS fallback. It passes those two numerical roles explicitly to an internal helper, avoiding subtraction/cancellation and preserving the existing standalone split_subgraph_ilp helper's legacy parameter interpretation.

Nineteen native ILP/ILP-audit tests pass, including the two exact methods with fallback=error, balanced8/8 partitions, cut4, and actual filesystem LP lower/upper population bounds760/840. The portable native State reference was rebuilt. Native/WASM before/after records are retained separately. The72-pair native dispatch regression still finds45 exact assignment matches/27 matching failures and refuses8 invalid settings; six RI/IA projects, actual browser three-method/fallback-error checks, general project suite, package dependency guard and fresh proof-model/native proof-pipeline checks pass. Sources: crates/bisect-runner/src/bisection_runner/ilp.rs, crates/bisect-runner/src/bisection_runner/tests/ilp.rs and scripts/web/test_ilp_balance_contract.mjs. Evidence: target/ilp-balance-contract/before.json and evidence.json, and target/ilp-population-contract-* logs.

This corrects the operational State/WASM dispatcher, not every legacy direct-helper call. The odd-district equal-half node schedule remains a separate unresolved limitation, and local solver results do not certify the whole plan. Per-node browser LP/solve report delivery remains pending; report review led to this correctness fix first. Native field declarations/transport are unchanged. Prior artifact evidence remains historical and keeps its original identities. Full desktop parity remains active. Local and unpublished.

Engine artifact SHA-256: 42e6c29eff4afc5c3c0a718e51577a652a7a103e4e3856a2b6245b075d662b3c.


### Native ILP node inspection in the toolkit

The toolkit now exposes Inspect native ILP run. Select a JSON engine request containing graph and options with structure="ilp", then press Run. It executes the shared native dispatcher and displays each node's outcome. Download inspection packet retains the original unit roster, local-to-original unit indices, exact native LP and solve-report strings, source hash, and hashes for each exported file. Save/Open retains the source and results; opening a project is inert and requires a fresh run to enable the packet download.

The shared dispatcher accepts an optional per-invocation Sync observer. Normal callers retain their existing behavior. Actual solver attempts produce native reports and LP text before fallback or failure, so a failed inspection can still deliver the reports produced before it stopped. Size-guard skips retain their unit mapping and settings but do not invent a solve report. Successful inspection results identify report delivery as captured-in-inspection-packet; ordinary laboratory runs still report not-captured.

Validation covers 36 host/WASM/native State cases across all three methods, both fallback policies, size-guard settings, and one/two/four districts. Eight dispatch failures match native outcomes. Forty-four native filesystem artifacts match: LP bytes are exact, and report comparisons exclude only genuinely measured solve_time_secs. Delivered report bytes and hashes retain those timings. Six invalid input cases are refused. Nineteen native ILP/audit tests pass. The actual worker browser checks failed and solved runs, native claim tables, exact packet download, inert Open, and Clear. Native dispatch and population-balance regressions, general project checks, package dependencies, and fresh proof gates also pass.

Inspection limits are 8 MiB source text, 512 nodes, two million retained unit references, 16 MiB report/model text, and 200,000 variables per exported model. Bounds reject the inspection rather than exporting a truncated packet. The input refusals exercise malformed JSON, oversized source, wrong structure, missing source, invalid UTF-8, and extra project-input fields; the aggregate recorder ceilings are enforced in code but are not separately stress-tested by this gate.

Scope remains explicit: native local solver/proof statuses are report claims, not whole-plan optimization, authentic-execution, or legal certification. A failed native run can leave only the reports already produced. This operation inspects one engine dispatch; integration into normal laboratory runs and accepted population-retry runs remains pending. Native equal-half scheduling for odd district counts also remains unresolved. Native filesystem reporting has a separate empty-directory issue when every node is size-skipped: the algorithm without report_dir succeeds, while report-directory audit can fail. The comparison gate distinguishes algorithm outcomes from artifact delivery rather than claiming that filesystem behavior is equivalent.

Full desktop parity remains active. This work is local and unpublished.

Engine artifact SHA-256: bce91d0b6a6c9dc553505a7e013df38030ba74a1b3737adb3b2a677eaa6ab0ca.


### Laboratory ILP reports from the returned dispatch

Native ILP dispatch now offers Retain native node LP models and solve reports in the laboratory's engine settings. Run produces the map and retains an ILP reports packet from that same dispatch. ILP reports downloads the packet for the selected completed state. Size-guard records explicitly contain no fabricated LP or solve report. Capture is optional and does not execute the algorithm a second time.

The optional capture_ilp_reports engine field wraps the existing invocation-local observer. The packet uses schema bisect-native-ilp-run-reports-v1 and retains exact serialized request bytes, their hash, returned assignment bytes and their hash, the original unit roster, native node reports/models, local-to-global indices, and file hashes. The packet has no nested duplicate engine result. Population retries retain only the packet belonging to the returned accepted attempt and attach its accepted_attempt number. Current native ILP dispatch ignores requested seeds; these tests exercise accepted attempt zero and complete retry exhaustion, not a successful later attempt. A failed ordinary laboratory run has no accepted packet; use the explicit inspection tool to retain reports produced before a failed dispatch.

Saved projects retain the packet. Opening remains inert and verifies evidence in the existing assignment-verification worker. Checks bind request graph/options to the saved run, returned assignments to the saved plan, node paths/unit mappings to the native floor/ceil tree and final leaf labels, source/model/report bytes to hashes, report models to their native artifact references, population settings to each node's seat count, and the packet to its accepted attempt. File paths are generated from bounded binary node paths and must match the exact report roster. No file is evaluated or extracted. Imported report claims remain unverified execution evidence; run again to enable a fresh packet download. Existing project retention still allows Save without executing anything.

Forty-eight portable native-host/WASM comparisons cover three methods, two size guards, one/two/three/four districts, and ordinary versus population-build execution. Capture preserves assignments and retry decisions. Six actual RI/IA runs round-trip through inert projects. Twenty-two refusals cover malformed capture settings, altered source/assignment bytes, unit mappings, node roster, model text, hashes, extra file paths, population fractions and attempt labels, wrong structure, retry exhaustion, and model capacity. A dense synthetic 300-unit graph exceeds 200,000 exported model variables: native and WASM refuse capture explicitly, while the uncaptured run still returns a fallback plan. The real RI browser checks the worker request, accepted-attempt label, exact download, inert Open, hostile import refusal, and capture-off/failure export retirement. Existing native dispatch, population-balance, explicit inspection, project, dependency and fresh proof gates pass.

Request/model/recorder limits remain 8 MiB request text, 200,000 variables per model, 512 nodes, two million retained unit references, and 16 MiB report/model text. A saved laboratory project remains bounded at 25 MiB. Recorder ceilings reject capture rather than delivering partial packets; aggregate node/reference/text ceilings are enforced but have not been separately stress-tested. Hashes establish consistency, not execution authenticity. Local exact solver claims do not prove whole-plan optimality or legal certification.

Ordinary laboratory and accepted-attempt ILP report delivery are now implemented. Native odd-district equal-half solving, the all-size-skipped native report-directory audit issue, other orchestration/capacity gaps, and full desktop parity remain unfinished. These updates are local and unpublished.

Engine artifact SHA-256: 8943be596923c304e87a47235aa44913a8ff0278c491f4ea602796160f2a9011.


### Native ILP report packets open in the artifact auditor

Captured laboratory ILP runs now offer Audit ILP reports for the selected completed state. Native ILP inspection also offers Open artifacts in native auditor, including a failed dispatch that retained solver artifacts. Both prepare a source-preserving workbench project for the existing audit-ilp-artifacts operation and transfer it through the bounded, expiring, one-use project handoff. Preparation does not execute an audit. Press Run in the auditor to check the selected artifact bytes and download its native summary.

The adapter checks node paths, duplicates, exact report/model file roster, original node/file byte agreement, file hashes, strict report JSON and existing audit byte limits before constructing source files. It retains exact report and LP bytes as Base64. It carries no generated result, last operation or certificate authority into the new project. Size-skipped nodes are counted in the project title and have no fabricated artifacts; a run with no solve artifacts cannot open an artifact audit. Imported inspections and laboratory reports require a fresh run for the handoff controls. Preparation blocks duplicate clicks and refuses a changed run/selection or a cleared/edited inspection before navigation.

Artifact count limits are now 512 reports and 512 models in Rust, chooser helpers, saved projects and folder selection. This covers the portable engine's maximum 499 internal split nodes for 500 districts. Existing limits remain 8 MiB total reports, 8 MiB per model, 16 MiB combined report/model bytes and 1 MiB optional saved summary. A packet exceeding those auditor limits is refused explicitly; the handoff does not select a smaller subset or discard artifacts. Laboratory/inspection capture limits and 25 MiB project/transfer limits are unchanged.

Two actual node packets, one solved and one failed, compare against both portable native-host execution and the actual native ilp-audit --dir command over the same retained files. Native CLI and WASM artifact identities are recorded. A separate capacity test uses 512 explicitly replicated native fixture reports/models and confirms native-host/WASM summaries and project retention; this is not a 512-node engine-generation claim. Seventeen malformed/changed/path/count/empty-packet cases are refused. The real browser runs RI with actual LP/report capture, transfers exact sources without an audit run, then performs the native audit explicitly. It repeats the workflow from a failed inspection and confirms that a clear during held hash preparation retires the handoff. Existing artifact audit, general project, laboratory report, explicit inspection, dependency, proof-model and native proof-pipeline gates pass.

Artifact verification establishes report/model byte consistency. Native outcome, proof-status and fallback fields remain reported claims. A verified artifact audit for a failed dispatch does not validate a plan, authenticate solver execution, or prove global optimality. Whole-plan certification and full desktop parity remain unfinished. Native odd-district half-split scheduling and all-size-skipped filesystem report-directory auditing also remain unresolved. Changes are local and unpublished.

Engine artifact SHA-256: ad0d981616619fadf61d77b0156fa8f155b8d4e3879a1c92642be330ce31c34c.


### Spectral State iteration controls and single-district summaries

Independent State argument conversion exposed a single-district observability mismatch. Native State's Spectral arm always calls run_spectral_recursive and writes spectral_summary.json, including when k=1: the native summary retains method, declared maximum iterations, tolerance, one district, zero cut edges and an empty node list. The browser's generic one-district shortcut returned identical assignments but omitted that summary. The before artifact records this mismatch separately.

Portable engine execution now calls the shared spectral kernel before the generic single-district shortcut. It retains the actual native summary for every spectral district count and leaves other methods' shortcut behavior unchanged. This also supplies the summary in configured YAML and population-build results. The kernel performs no split for k=1; an empty node list does not claim smoothing iterations were executed. Zero iteration budgets remain rejected by both native spectral validation and the portable request boundary.

The new independent CLI example parses actual State arguments and applies AlgorithmConfig::from_state_args before invoking the same Spectral arm used by the native runner. Seventy-four comparisons cover native --partition-mode spectral and --structure spectral, synthetic 4x4/RI/IA graphs, one through four districts, requested budgets1/20/200 (including omitted native default200), maximum-u64 requested seed and declared refinement123. Seventy-two cases match native assignments and full summaries; two deliberately infeasible positive-population graphs fail with matching native generation errors. Two zero-iteration requests are refused. The native smoothing algorithm ignores random seed and METIS refinement iterations; alternate declarations retain identical assignments/summaries. This oracle verifies State conversion and first-attempt spectral dispatch, not the complete native State acquisition, filesystem, analysis or population-retry workflow. Normal laboratory results still report final balance and connectivity separately, and native population-build execution is separately available.

Laboratory project verification now checks supplied one-district spectral summaries against graph/options and requires zero split nodes and zero cut edges. Four actual RI/IA ordinary/population-build runs save and reopen inertly; twenty changed iteration/district/cut/node/tolerance claims are refused. Older one-district projects that lack a summary remain loadable as unverified history, with an explicit missing-summary notice. A current summary omitted from a saved file cannot be distinguished from that older representation; loading does not authenticate history. The actual RI browser checks that notice, a fresh native empty-node summary, original iteration budget, inert Open, changed-summary refusal and preservation of current work.

The297-case configured one-district gate now expects the native summary for Spectral while preserving other shortcuts and declarations. Existing RI/IA spectral weight/budget checks, ILP report delivery, general project and dependency checks, and fresh proof-model/native proof-pipeline gates pass. The portable smoothing budget remains1–100,000; larger native usize values and full filesystem/orchestration capacities are not established by this gate. A generated spectral plan can still miss final population tolerance or contiguity, and smoothing convergence does not prove an optimal or legally certified cut. Full desktop parity remains active. Changes are local and unpublished.

Engine artifact SHA-256: 3356ea06e30d50a19264df16eb45d91b617d763895fe396a463c9bf7633ceed5.


### Native district-seat metadata in the laboratory

The laboratory's engine settings now expose optional Record electoral settings, requested seats per district, and total seats per selected state. A blank total retains the native default: requested seats per district multiplied by that state's district count. National mode resolves this separately for each selected state. The result displays requested and effective counts, total seats, and the native ideal population per seat. An inconsistent total or zero-total denominator is disclosed explicitly. Fractional per-seat populations use significant digits rather than being rounded to a misleading integer zero.

These controls expose existing native State manifest/settings semantics, not a new seat-weighted algorithm. Requested seats per district are clamped to at least one for the effective count; the requested zero still produces a default total of zero, and the per-seat denominator is clamped to one. Inconsistent explicit totals remain recorded metadata. Generation continues to target equal district populations. Variable district magnitudes, proportional electoral outcomes and legal eligibility are not implemented by this native metadata path.

The optional config.electoral object travels through ordinary, population-build, ILP capture, and multiscale requests. Native evidence is retained in state metrics and the effective configuration. Inert project verification checks requested/effective counts, totals, system token, district count, consistency flag, native population basis and the canonical settings hash. ILP packet source requests are also bound to the recorded electoral settings. Multiscale keeps the existing native tract-input population basis; laboratory fine bundles already enforce complete connected parent-tract coverage and exact population sums. Original settings and their native source hash are retained when preparing a practitioner RPLAN/RCTX project, including multiscale plan exports. Turning the option off omits the input and evidence instead of inventing a default settings source.

Twenty-four run/project cases cover RI/IA ordinary and population builds, national mode with different district counts, captured ILP, and explicit synthetic tract/block-group multiscale inputs. Twenty independent native StateConfig oracles check effective settings/per-seat arithmetic. Seven plan exports retain original settings, context hashes and manifest values. Ninety refusals cover changed/missing evidence, source settings changed inside an ILP packet even after rehashing, malformed counts/fields and derived total overflow. Metadata-on/off comparisons retain identical generated assignments. The actual RI browser checks inert Open, explicit/automatic/zero totals, native audit-plan export/hash binding, altered-evidence refusal with current work preserved, and optional controls. Existing25-case multiscale state/national project/map gate,48-case ILP report gate, and113 native-host electoral comparisons/69 replay bindings pass. General project/dependency/source drift checks pass. Proof evidence remains bound to the unchanged engine artifact.

Portable metadata is bounded to unsigned32-bit seat values and totals, with derived totals checked for each resolved state before submission. Larger native usize values are not covered. Metadata hashes establish consistency, not authentic generation or certification. Full desktop parity, other orchestration/capacity gaps and native algorithm issues remain unfinished. Changes are local and unpublished.

Engine artifact SHA-256 (unchanged): 3356ea06e30d50a19264df16eb45d91b617d763895fe396a463c9bf7633ceed5.


### Ordinary laboratory community-of-interest weights

Ordinary laboratory runs now accept optional named COI weights JSON using the existing bounded native input. The source maps unit IDs to finite nonnegative factors; missing units default to one and unknown IDs are counted and ignored. A national run supplies the same file to each state's prepared graph. The result displays matched/defaulted/ignored units, computed edge counts, changed weights, native status and exact source SHA-256. Original UTF-8 JSON text is retained rather than parsing and reserializing numeric declarations for Rust. Multiscale retains its separate existing tract-initialization COI source; the ordinary panel is hidden for that search.

Native post-weighting multiplies each explicitly computed algorithm edge weight by sqrt(factor_u*factor_v). Sparse partisan and minority maps contain only boosted edges: the implicit unit reporting cost on a missing map edge must remain unchanged. Unweighted mode has an empty map and the COI source has no effect. Constructors using unweighted objectives retain their existing behavior even when weighted costs are reported. Physical graph boundary lengths and census populations are retained.

Ordinary, population-build and ILP requests now carry the selected COI input. Saved projects preserve source text and validate computed-map counts, adjusted weighted costs and exact source hashes in the existing verification worker. Captured ILP request bytes are bound to that same source. Practitioner RPLAN/RCTX export retains the original COI input in producer provenance and its native hash in the context/source bindings. The effective configuration records the source name without duplicating the complete file once per state.

Twenty-three run/project cases include nineteen explicitly synthetic combinations across all seven weighting modes and the laboratory's supported ordinary/ratio/spectral compositions, actual RI inputs, a native population build, national mode and captured ILP. Nineteen independent native selected-file post-weighter oracles confirm adjusted sparse/full maps and reported cut costs; portable native-host receipts also match. One synthetic practitioner export and the actual RI browser export retain source/provenance/hash bindings. One hundred three refusals cover changed raw bytes, changed map counts, forged hashes/costs, rehashed ILP source substitution and invalid sources; two of these confirm the existing partisan composition restriction, not new native capability. The actual RI browser checks source-preserving inert Open, explicit Run, export, changed-source refusal, invalid selection blocking until Clear, clearing input/evidence, and a delayed read unable to replace an opened project. Existing toolkit COI,24-case electoral,25-case multiscale and general project/dependency checks pass. Proof evidence remains bound to the unchanged engine artifact.

Source limits remain3 MiB,100,000 safe unit IDs and200 UTF-8 bytes for the source name. Invalid or overflowing selected sources reject the browser run explicitly. The native filesystem wrapper's warning/fallback policy is a different existing path and is not reproduced here. The laboratory still restricts partisan weighting to its existing Standard-compatible searches; broader structure composition remains a separate parity gap. Exact bytes/hashes prove consistency, not community provenance, legal status or authentic generation. Full desktop parity, other orchestration/capacity gaps and native algorithm issues remain unfinished. Changes are local and unpublished.

Engine artifact SHA-256 (unchanged): 3356ea06e30d50a19264df16eb45d91b617d763895fe396a463c9bf7633ceed5.


### Native partisan GeoSection and Spectral composition

Native State permits --partition-mode partisan-weighted followed by --structure ratio-optimal or spectral. The browser previously restricted partisan inputs to Standard-compatible searches. Portable request validation, laboratory configuration and UI selection now additionally allow GeoSection Single/Multi and Spectral Single. Switching between these structures retains the selected partisan source and ordered thresholds. Other structures/searches remain outside the verified partisan compatibility set; this update does not claim complete composition parity.

The independent State oracle parses actual arguments, uses AlgorithmConfig::from_state_args, loads the native TSV source, applies PartisanOverrideWeighter to the explicit sparse map, and runs the corresponding native GeoSection or Spectral kernel. One hundred eight comparisons cover synthetic4x4/actual RI/IA prepared graphs, two seed declarations (None and maximumu64), one/three districts, uniform/stripe/exact-threshold synthetic shares, and all three newly exposed method/search pairs. All assignments and reported weighted boundaries match. Thirty-six cases have an empty boost map. Native GeoSection ratios and candidate cuts and complete spectral summaries, including k=1, match. GeoSection uses explicit metis-core in this portable oracle; the test is State conversion/source-loading/first-attempt dispatch evidence, not complete native acquisition/filesystem/analysis/population-retry orchestration.

Comparison exposed a necessary distinction between costs. The existing root weighted_boundary reports the cut with implicit unit weights included. Native GeoSection's returned candidate-selection cost counts explicit map weights; an empty map can return zero even when the reported boundary cost is positive. Partisan GeoSection now retains that returned scalar separately as root_split.native_candidate_cut. The UI displays both values and their meanings. Project verification recomputes the candidate cut from sparse membership and any applied COI factors; it does not substitute the reporting cost for the native selection objective. This observation does not establish a globally best cut or optimization certificate.

Six actual RI/IA projects save/open inertly, refuse twelve changed source/summary/candidate-cut claims and export original partisan sources. The real RI browser runs GeoSection Single/Multi and Spectral Single, retains source data across structure changes, distinguishes candidate/reporting costs, and reopens without executing. The prior900-case partisan native-host/weighted-cut gate and9,900 changed-evidence refusals, twelve search-boundary cases, and the expanded COI gate pass. COI now covers25 run/projects and21 native file post-weighter oracles, including the two previously refused partisan compositions;111 malformed/changed cases are refused. General project/dependency checks and fresh proof-model/native proof-pipeline gates pass.

Portable thresholds remain finite, ordered and within[0,1], with complete bounded supplied share inputs. Native TSV missing-unit fallback, additional partisan structure/search pairs, nonportable METIS backends and full capacities/orchestration remain separate gaps. Synthetic share profiles on real geographic graphs are explicitly test data, not actual elections. Spectral constructs from its native unweighted objective while partisan weights affect reported costs; convergence does not prove global optimality or legal compliance. Full desktop parity remains active. Changes are local and unpublished.

Engine artifact SHA-256: ee3206cb08f5c1e12cf84999eeafc6753b22b282d57ff49f68285dbcb0037501.


### Native direct n-way partisan weighting

Portable requests and the laboratory now additionally accept partisan weights with Direct n-way/Single. Native State supports partisan-weighted mode plus a nway structure override. The existing complete share-input requirement and ordered thresholds remain; the UI keeps those sources while exposing n-way refinement and imbalance settings.

The independent native State oracle now applies the actual TSV loader and sparse PartisanOverrideWeighter before the native direct n-way dispatcher. Seventy-two cases compare synthetic4x4/actual RI/IA graphs, one/three districts, None/maximum-u64 seeds, uniform/stripe synthetic shares, and requested imbalance factors0/5/maximum-u32. All assignments and reported weighted boundaries match;36 cases have an empty computed map. The native clamp1–1000 is retained independently from final population tolerance. Both these tests and the prior108-case GeoSection/Spectral gate now retain a separate TSV file for each oracle request instead of overwriting a shared file.

Four RI/IA projects cover cut/volume refinement and three internal trials through the portable native-host API, inert Open, altered imbalance evidence refusal and source-preserving plan exports. The actual browser runs both refinement objectives, keeps the original share input, confirms requested0/effective1 imbalance and reopens without generation. Native CLI direct n-way uses the default cut/one-trial wrapper; volume/multiple-trial parity here is a comparison against the shared native tuned kernel API, not a claim that the CLI State wrapper dispatches those declarations. First-attempt State tests also do not establish complete native acquisition/filesystem/analysis/retry orchestration.

The existing20-case separate-imbalance native/kernel gate,108-case partisan GeoSection/Spectral gate, general project checks, partisan regression and fresh proof-model/native proof-pipeline checks pass. Other partisan structure/search pairs, missing-share defaults, nonportable backends and full capacities/orchestration remain unfinished. These heuristics can miss final population balance; generated results retain independent checks and unproved optimality. Full desktop parity remains active. Changes are local and unpublished.

Engine artifact SHA-256: 66d4ae87333875dfcdc10d5b54e4c93c4a0218faf07f379978090551e14f1bfa.


### Native report manifests in label-build packages

Label builds now offer Include native report manifests for completed states. The optional write_manifest input defaults false so legacy assignment/plan/context packages retain their output shape. When selected, each successful state calls the existing shared native report-manifest builder on its accepted exported RPLAN/RCTX and stores manifest.json beside those exact files. A state receipt records manifest path/hash, plan/context source hashes, population-balance evaluation and the builder's scope. The option participates in checkpoint request identity.

The native manifest retains state/year/chamber, district/unit counts, exact unsigned seed representation, electoral metadata, relative plan/context paths and context identity. Its classification remains research. No native Census origin, executable digest, authentic runtime provenance or solver certificate is invented. Full native manifest/provenance promotion remains a separate unfinished workflow.

Manifest generation is a requested follow-up stage. If it fails after generation—for example, a valid COI source makes the exported RPLAN exceed the manifest builder's3 MiB text limit—the state reports failed with generation_status=completed and an explicit report-manifest error. Completed assignment/plan/context files and capture, when requested, remain in progress checkpoints and the final package. The build index records the failed requested workflow, and the results table distinguishes generation status from manifest availability. The wrapper's native-compatible year completion policy still records the completed year as built even when a state failed. It does not label a failed manifest as successful or truncate it to fit.

Six manifests across two synthetic three-state/year batches match full native-host results and the shared builder's exact manifest bytes. Source hashes and relative references match the generated files, population validity is recomputed, classification remains research and progress deltas retain the files. Native/WASM checks verify a manifest-size failure with completed generation retained. Project settings/results round-trip without execution. The real worker browser checks inert Open, enabled manifests, setting edits retiring downloads and option-off legacy outputs. An independent Python ZIP reader checks all output bytes/CRCs and all six manifest entries. Existing five-batch native assignment/seed and ordered progress regressions, general project/dependency checks and fresh proof-model/native proof-pipeline gates pass.

Optional manifests inherit the builder's text/context/unit capacities, and complete batch output remains bounded at2,000 files/64 MiB. The file has not been added to the static source-based engine option inventory because write_manifest is a label-build workflow input, not a new engine Options declaration. Native source acquisition, parallel workers, full manifest provenance, automatic analysis/report stages, existing filesystem merge and full desktop parity remain unfinished. Changes are local and unpublished.

Engine artifact SHA-256: df081d6f90d338e1b34c1bfab988d93a5d8976ce554d4e819016f752a441a6d0.


### Built-state manifest handoff to native TUI prechecks

Selecting a retained label-build plan now carries any generated native report manifest alongside its exact RPLAN/RCTX. The practitioner selector adds Verify native manifest prechecks while preserving Validate plan/context as its default. Verification requires a retained manifest; builds without one keep the existing plan/context handoff and do not acquire a fabricated manifest.

The handoff retains original Base64 source bytes, supplies plan.rplan/context.rctx under the manifest's relative names, validates bounded manifest/package sources, and selects native TUI Verify prechecks mode. It executes nothing. Previous manifest/package, replay, certificate and lineage inputs are retired before applying the new selected state's sources; a stale replay mode is reset explicitly to prechecks. Users must press Run to obtain a fresh typed precheck receipt. Other practitioner operations also retain the manifest for later use.

Three generated state packages produce matching native-host/WASM precheck receipts and source-preserving inert projects. Cross-runtime comparison excludes only current_build.build_unix because the host and WASM compilations occurred at different real times; the retained receipts keep their actual build metadata. Plan/manifest bytes remain exact. The browser starts with prior replay/package/certificate/lineage sources, completes a fresh batch, selects the generated state, and confirms those prior authorities are retired without a run. Explicit prechecks retain execution_performed=false and replay_matched=false. Legacy packages without manifests remain supported for plan/context selection. Existing fourteen native TUI verification constituent cases and general project/dependency checks pass. Proof evidence remains bound to the unchanged engine artifact.

Prechecks verify the native manifest's ILP/certificate branches when declared. They do not constitute plan/context validation, an engine replay, independent Census/source rehashing, generation authentication, whole-plan optimality or legal certification. Those remain separate explicit tools and scopes. Native source acquisition, full manifest provenance, other orchestration/capacity gaps and full desktop parity remain unfinished. Changes are local and unpublished.

Engine artifact SHA-256 (unchanged): df081d6f90d338e1b34c1bfab988d93a5d8976ce554d4e819016f752a441a6d0.


### Accepted batch engine requests retained for explicit replay

Label builds now offer Retain accepted-run artifacts for explicit replay. The optional retain_replay flag defaults false, preserving existing package shape. When selected, each completed generation retains accepted_engine_run.json containing the actual accepted engine_request returned by configured population-build execution. It records the accepted seed and all supplied graph/options/auxiliary data without reconstructing a default request. State receipts bind its exact text hash and accepted attempt, and completed-state checkpoints retain the file. The option participates in checkpoint request identity and the file/byte output budgets.

The generated-state practitioner selector adds Verify manifest and replay accepted run. It requires both the native report manifest and retained run artifact. Original assignment sources are derived from the exported plan's explicit unit order and zero-based district labels, with index/label validation and bounded replay-source validation. The raw retained request text is Base64 encoded without parsing/reserializing its seed/numeric declarations. Tool selection sets native TUI replay mode and runs nothing; users explicitly press Run for manifest-to-run binding, engine execution and exact assignment comparison. The existing precheck/default plan handoffs remain available, and old replay/certificate authorities are still retired before selection.

Three state artifacts match full native-host batch output, exact source hashes and progress deltas, and produce three matching native manifest-bound replays. Three changed-seed requests fail binding before execution. A separate configured retry fixture accepts attempt4 with seed46; retained request, native-host package and manifest-bound replay all match that later accepted plan. This confirms accepted-seed retention rather than reuse of the original request seed. The real browser checks inert Open/handoff, original run-source retention, explicit replay and exact similarity1. Independent Python ZIP validation confirms all bytes/CRCs and all three run artifacts. Existing batch/index/progress and general project/dependency checks, plus fresh proof-model/native proof-pipeline gates, pass.

This workflow replays the accepted engine request, not the earlier rejected population trials. It retains raw configured options/data in that request, but does not independently revalidate the original YAML configuration through a configured replay kind. Matching results establish reproducibility under the loaded portable implementation and selected inputs, not authentic original execution, Census source truth, optimality or legal certification. Source and output capacities remain the existing bounded engine/replay/batch capacities. Native acquisition/workers, full manifest provenance, additional orchestration/capacity gaps and full desktop parity remain unfinished. Changes are local and unpublished.

Engine artifact SHA-256: fabe725b5089d485bb3010a5ede5973f8eeb0ab497b5613e845b7183c98c9f44.


### Original configured inputs and accepted retry attempt replay

Retain accepted-run artifacts now keeps accepted_configured_run.json alongside the existing accepted_engine_run.json. The configured recipe contains the original YAML, source name, profile, chamber, prepared graph and supplied auxiliary inputs, plus the accepted attempt. It uses the shared native seed policy for that explicit attempt. Earlier rejected attempts are not rerun. Both artifacts remain opt-in, participate in existing output budgets, and retain exact raw JSON text through the practitioner handoff. The handoff prefers the configured recipe; older packages with only engine artifacts remain supported. Opening or selecting a project remains inert.

Configured manifest replay resolves the original native YAML compositor, binds recorded configuration and explicit execution attempt/seed, and reconstructs the native context before generation. Four matching replays cover three batch states and the later accepted attempt4/seed46 fixture. Native Rust and WASM binding receipts and assignment similarity match. Twenty-four changes to YAML, source name, profile, chamber, valid attempt or out-of-policy attempt block execution. Four invalid JavaScript attempt declarations are refused. The actual browser retains the configured recipe and performs only explicit replay. Legacy engine-only replay, exact package ZIP contents, general project/module checks and fresh proof gates pass.

This closes the original-YAML resolution gap described in the earlier accepted-engine artifact entry. It establishes reproducibility for the selected accepted attempt under the loaded portable implementation. It does not authenticate original execution or source truth, replay rejected history, certify legal compliance or establish optimality. Full native acquisition, orchestration, provenance and capacity parity remain unfinished. Changes are local and unpublished.

Engine artifact SHA-256: cc36fbe43990a5c0d7660b2cbbbcfba39f766a328e6049e1ac752421bf38760f.


### Native partisan simulated annealing composition

The portable engine and laboratory now accept partisan weights with simulated-annealing/Single. This follows native State's simulated-annealing preset with the proportional weight override and a selected partisan TSV. The existing full aligned share input and finite ordered thresholds remain required. The laboratory preserves shares and exact optional seeds while exposing the existing annealing settings. Native initialization consumes the sparse partisan boost map, with fixed 100 refinement iterations. Subsequent annealing minimizes unweighted edge cuts, as before; weighted boundary cost remains a separate reported measurement. No weighted annealing objective has been invented.

An independent native State parser/converter and TSV-loader reference verifies 108 cases on a synthetic grid and actual RI/IA prepared Census graphs: neutral/striped/exact-threshold shares, absent/maximum-u64 seeds, one/three districts and zero/two/ten annealing steps. All assignments and weighted boundaries match; 36 cases exercise empty sparse maps. Six further cases apply the native filesystem COI post-weighter, including zero/missing/unknown unit factors, and match assignments, costs and selected-source hashes. Four RI/IA projects preserve exact source data and practitioner exports, restore without execution and reject twelve altered-share/objective/step claims. The actual RI browser runs zero/ten steps through the worker, displays the exact maximum-u64 seed, retains source data and reopens inertly.

Existing native/WASM annealing regression has 56 successes and nine matching failures; native structure/search regressions, original-configured accepted-attempt replay, general project/package checks and fresh proof gates pass. Native State weight-override comparisons use its effective default thresholds, 0.55/0.45; these tests do not claim that this State override applies arbitrary threshold flags. The portable boost-map kernel continues to use explicitly supplied validated thresholds. Additional partisan structure/search combinations and full native command/acquisition/orchestration/capacity parity remain unfinished. This heuristic does not establish global optimality or legal certification. Changes are local and unpublished.

Engine artifact SHA-256: 92612a59ba2fa2accedeb41fbf059877ff155a72ced74bb61eb052f1223676d2.


### Native partisan Compact Polsby composition

The portable engine and laboratory now accept partisan weights with Compact Polsby Single/Multi. Native State supports partisan-weighted mode with a compact-polsby structure override. The sparse boost map, required aligned share input and ordered thresholds retain native semantics. CompactBisect chooses geometric-mean Polsby candidates among cuts within the allowed weighted-cut slack. Single uses one candidate seed per split; Multi uses the selected budget. Native candidate seeds remain fixed one through budget; the user seed is retained as a declaration and does not change this search. The browser keeps the share sources while exposing the existing slack and candidate-budget controls.

An independent native State parser/converter, TSV loader and dispatch reference verifies 72 cases on a synthetic grid and actual RI/IA prepared Census graphs, using identical supplied areas/exterior perimeters: neutral/striped/exact-threshold shares, absent/maximum-u64 declared seeds, one/three districts and Single/Multi. Assignments and weighted boundaries all match; 24 cases exercise empty sparse boost maps. Twelve further cases apply native filesystem COI weights and match assignments, costs and selected-source hashes. Native State's override fixes epsilon at .05. Twelve separate shared-kernel native-host/WASM comparisons verify API slack values zero/.2/one, without claiming those values are native State flags. Four RI/IA projects and practitioner exports retain exact shares, restore without execution and reject twelve changed-source/selection/slack claims. Actual RI browser Single/Multi runs use zero/20% slack, preserve shares, display exact maximum-u64 seed and reopen inertly.

Generic CompactBisect, partisan annealing/search, configured accepted-attempt replay, project/package and fresh proof gates pass. The independent reference supplies identical prepared geometry directly; native filesystem TIGER geometry discovery and its geometry preparation are not reproduced by this test. Other partisan structure/search combinations and full native command/acquisition/orchestration/provenance/capacity parity remain unfinished. CompactBisect remains a heuristic, not a global optimality or legal certificate. Changes are local and unpublished.

Engine artifact SHA-256: 25e1ee0ee0d787b453ff117646c0ab8b1b9db6cef47195b20415fbd490c8b847.


### Partisan sources with native BFS, CVD and moving-knife construction

The portable engine and laboratory now accept partisan inputs with BFS growth, Centroidal Voronoi and moving knife, each with Single search. Native State accepts partisan-weighted presets with these structure overrides. Their constructors consume adjacency/population and, where required, centroids; boundary weights are used only for reported costs. The UI explicitly discloses this, keeps exact shares/thresholds and exposes the existing native method controls. No influence on construction has been invented. Native full-u64 seeded BFS/CVD/moving-knife behavior remains unchanged.

An independent native State parser/converter, TSV loader and shared dispatch verifies 180 cases on a synthetic grid and actual RI/IA prepared Census graphs. Coverage includes neutral/striped/exact-threshold shares, absent/maximum-u64 seeds, one/three districts, CVD geographic zero-iteration and graph-distance three-iteration modes, and moving-knife zero-Reock/two-Polsby orientation declarations. All assignments and weighted costs match. Every case also matches an unweighted constructor's assignments and structure evidence, directly checking the reported-cost-only boundary. Thirty native filesystem COI post-weighter compositions match and preserve assignments while retaining exact source hashes. Ten RI/IA projects preserve source inputs and practitioner exports, open without execution and reject twenty altered-share/reported-cost claims. Actual RI browser runs all three methods, exposes their cost-only descriptions, keeps shares and visible maximum-u64 seed, and reopens inertly. Native moving-knife Polsby declarations retain their existing Reock fallback.

Existing native/WASM BFS, CVD, moving-knife and partisan CompactBisect regressions, configured accepted-attempt replay, general project/package checks and fresh proof gates pass. These are native constructor comparisons on identical supplied prepared graphs/centroids, not filesystem centroid acquisition or full State orchestration. Matching outputs do not prove balance/contiguity acceptance for every parameter/state; measured outcomes remain explicit in the case evidence. Other compositions and full native command/acquisition/orchestration/provenance/capacity parity remain unfinished. No optimality/legal certification is inferred. Changes are local and unpublished.

Engine artifact SHA-256: 2afe26e725e70cc95a5bad9c275f0d98fec6971da70b528447e14eb86f6826eb.


### Partisan capacity/region/flow composition and native one-district summaries

The portable engine and laboratory now accept partisan inputs with capacity clustering, regionalization and flow construction, each with Single search. These native constructors use adjacency/populations, with weights affecting reported costs only. Existing UI descriptions disclose that boundary. Native seed/refinement declarations remain retained without inventing influence on deterministic construction. Capacity clustering remains a State/direct-engine choice; native YAML does not define it and no YAML alias has been invented.

The generic one-district shortcut previously omitted these three methods' native summaries. They now execute the native constructor at one district, retain exact summaries and regional merge logs, and keep native validity failures. Project import strictly checks present one-district clustering/flow summaries against assignments and requested settings. Historical null-summary projects still open as imported claims. Removing a summary cannot be distinguished from historical format, so this remains an unauthenticated claim boundary. Configured regionalization/flow record native Rust execution rather than a partitioning shortcut at one district.

Small-graph repair now prunes a prefix only when its changed-unit count already exceeds the incumbent's primary objective. Equal counts still compete on edge cut and lexicographic assignment; original labels are visited first to find an incumbent early. The returned global objective tuple remains unchanged. An independent exhaustive enumerator checks 816 small line/ring/zero-population/original-label/tolerance combinations. The formerly slow 16-unit/three-district case also exactly matches assignments, native summary and cost from an unchanged native executable retained before the optimization. The complete clustering test suite passes. This is exact search pruning, not a new repair policy, solver certificate or machine-wide performance guarantee.

Native State/TSV tests cover 108 synthetic/RI/IA tract cases: 72 matching outputs and 36 matching native failures. Successful cases include 72 unweighted assignment/evidence invariance checks and six native filesystem COI compositions. Ten plans save, reopen inertly and export with exact partisan sources; two additional regionalization runs preserve matching native failure. Thirty changed-source/cost/summary claims are refused, and six historical one-district null-summary imports remain supported. Actual RI browser tests cover one/two districts for all three methods, native summaries, explicit failures, source retention and inert reopen. Configured one-district tests have 297 native assignment/project comparisons; generic clustering/flow, configured accepted-attempt replay, project/package and fresh proof gates pass.

Tests use explicitly supplied prepared tract graphs, not full native source acquisition or filesystem orchestration. Heuristic native failures are visible; matching execution does not imply all configurations are feasible, globally optimal or legally certified. A focused RI/IA one-district matrix passes 286 native assignment/Save/Open/map cases across supported methods/searches, legacy/explicit options and three weights, with 858 tamper refusals and no execution on Open. It is bound to this artifact and explicitly records its two states. The attempted full 50-state one-district matrix failed during a Windows native-executable rebuild lock and does not provide a passing gate; the focused regression cannot establish all-state coverage. Other compositions and full command/acquisition/orchestration/provenance/capacity parity remain unfinished. Changes are local and unpublished.

Engine artifact SHA-256: f464b054abff741911c1885ef53d3243b9a0c0df8523375d92dc2fa7345efd55.


### Native partisan prime-factor partitioning

The portable engine and laboratory now accept partisan weights with prime-factor/Single. Native State supports partisan-weighted mode plus the prime-factor structure override. The existing shared PFR compositor receives the native sparse boost map, per-level tolerance derived from factor-tree depth, explicit metis-core backend, refinement count and exact optional seed. Largest-prime-first scheduling, floor/ceil handling of primes above three, native integer-cut/cache evidence and the independent final 3% research balance limit remain unchanged. Other search modes are still refused. No native YAML partisan-source support has been invented.

An independent actual native State parser/converter and TSV/post-weighter reference prepares 72 cases on a synthetic 120-unit line and actual RI/IA tract inputs, across neutral/stripe/exact-threshold shares, absent/maximum-u64 seeds and one/two/three/five districts. Fifty-six native runs complete and sixteen retain native rejection. Independent unit coverage, sparse weighted cut and final 3% balance checks accompany completed reference plans. WASM matches all assignments/costs, nontrivial factor/cache summaries and all sixteen rejection messages. Native one-district output retains the existing no-split summary boundary in the portable API. Two RI/IA projects retain exact source shares, open inertly and export matching native contexts/plans; six changed source/schedule/limit claims are refused. Actual RI browser runs retain visible maximum-u64 seed, original sources and the displayed native 3% limit, with inert reopening.

Existing prime-factor, standard partisan, ratio/spectral, nway, annealing, compactness, geometric-construction and capacity/flow regressions are refreshed. General project/module checks and fresh proof gates pass. The composition table now records nineteen scoped partisan combinations on this artifact. A separately running all-state one-district matrix uses its earlier frozen loaded WASM and unchanged native executable; no passing or current-artifact all-state claim is inferred while it runs. Native helpers serving that run have not been rebuilt. Other combinations, native acquisition/orchestration/provenance/capacity and full parity remain unfinished. Prime-factor is a heuristic research method, not a global optimality or legal certificate. Changes are local and unpublished.

Engine artifact SHA-256: 1c0e90685b9711c07391a35ef6d9661b95685d4381ae8e3e308e4177c1d08abf.


### Completed frozen all-state one-district matrix

The previously pending matrix has completed successfully in the Node/WASM harness: 7,150 native/WASM assignment cases and Save/Open/map roundtrips across all 50 prepared 2020 state inputs, all catalog-exposed structures/searches, legacy/explicit supported settings and geographic/unweighted/county weights. The 21,450 altered-project checks refuse fabricated evidence or modified metrics; opening projects executes no engine. This is shared browser code exercised in Node, not 7,150 actual Chrome runs. It verifies one district only, not multi-district feasibility or every parameter/weighting combination.

The test instantiated its WASM before the partisan prime-factor guard was added and kept the same native executable throughout. Its receipt and original WASM bytes are retained separately; the later artifact is not substituted into this record. Earlier notes reporting the matrix as running or its failed rebuild-lock attempt remain historical. Current prime-factor/partisan and other scoped composition gates have separate later-artifact evidence. Full native command/data acquisition/orchestration, other combinations, capacity/provenance and publication remain incomplete.

Verified frozen engine artifact SHA-256: f464b054abff741911c1885ef53d3243b9a0c0df8523375d92dc2fa7345efd55.


### Prime-factor native helper verification after the frozen matrix

After the frozen fifty-state matrix terminated successfully, the current native direct-engine/toolkit helper executables were rebuilt without an in-use file lock. All seventy-two partisan prime-factor cases now also match complete native-host engine output or the same error, in addition to the independent native State/TSV reference. Configured original-source/accepted-attempt replay and fresh proof-model/native proof-pipeline gates pass against those rebuilt helpers. The separate frozen fifty-state result remains bound to its original artifact; it is not relabeled as a current-artifact run. Local preview and evidence are updated, while publication and broader full-parity work remain unfinished.


### Native ILP all-size-skipped report path corrected

The previously documented all-size-skipped report-directory gap is resolved. The shared dispatcher now counts solve reports actually written by the current invocation. If every node exceeds max_tracts, successful native METIS fallback returns its plan without trying to scan/audit a missing or old report directory. No reports or empty audit certificate are fabricated, and prior files are preserved. A mixed run still writes/verifies its actual solve reports and native audit summary. A new native report-count return interface preserves the existing assignment-map/observer APIs. Native State uses that invocation count when deciding report and summary provenance fields, so an all-skipped invocation cannot attach prior directory artifacts to its newly generated manifest. This source boundary does not authenticate old files or independently establish every full CLI filesystem workflow.

Twenty-one targeted native ILP tests pass, including new missing/stale-directory and mixed-skip cases. A direct native State reference with an explicit report directory matches WASM's skipped-node fallback plan; old report/summary bytes remain untouched. The empty WASM packet cannot be handed to the artifact auditor. A four-district mixed case retains one skipped node and two attempted-node report/model pairs that match actual native files, excluding only measured solve_time_secs. Its audit handoff remains inert.

The browser explicitly states that all nodes skipped solving, no solver artifacts were produced and artifact auditing is unavailable. One-district zero-node packets are distinguished as needing no ILP splits. The actual RI browser verifies the empty packet/download, disabled auditor, inert reopening, changed-source refusal and capture-off/error retirement. Existing inspection has 36 host/WASM/native State cases; ordinary capture has 48 cases and six RI/IA projects; report handoff/native directory audits and fresh proof/replay/project/module gates pass.

No solver behavior, fallback seed/weight policy, solve tolerance, declared time-limit semantics or global proof policy is changed. Mixed/nonempty directories still use the existing native directory audit policy; this fix does not certify arbitrary stale files or redefine a summary as current execution authenticity. Full native orchestration, other engine compositions, provenance/capacity/publication and complete parity remain unfinished. The earlier frozen fifty-state matrix remains a separate artifact-specific record. Changes are local and unpublished.

Engine artifact SHA-256: e7c36f24383f8ac48454eada1b310f648bb7869038ae766a3b7bb74355d7e5ec.


### Partisan ILP reporting and complete capture auxiliary bindings

The portable engine and laboratory now accept partisan weights with ILP/Single. This follows native State's ILP preset with its proportional weight override and selected partisan TSV. Native ILP construction and its METIS fallback remain unweighted, with backend-default fallback seeds and fixed refinement settings; partisan weights affect reported costs only. Existing UI descriptions and native report metadata disclose this boundary. Native solve/fallback methods, declared time-limit semantics, solver/capture bounds and proof status remain unchanged. State weight-override comparisons use its effective default thresholds, .55/.45; this is not proof that State applies arbitrary override threshold flags.

There are 216 native State/TSV comparisons on synthetic/actual RI/IA tract inputs, across neutral/striped shares, three methods, solver-size versus all-skipped conditions, fallback versus error, absent/maximum-u64 seed and one/two districts. All 180 completed cases also match unweighted assignments and structure evidence; thirty-six native failures match exactly. Two real-state captured projects retain shares in the actual captured request, restore inertly and export source-preserving contexts/plans. Six altered configuration/capture-source cases are refused, including changed partisan data with its source hash recomputed. Actual RI browser execution retains the visible exact seed and captured shares, displays native unweighted construction and keeps auditing disabled for empty artifacts.

Capture import previously compared graph/options/COI/electoral data but did not bind demographics, character, partisan or election auxiliary fields to the selected configuration. Those fields are now compared as exact per-state inputs before trusting a captured packet. Three legitimate minority/economic/housing captures still validate, while twelve recomputed-hash changes or injected auxiliary sources are refused. This strengthens consistency of imported claims; it does not authenticate original execution. Existing 48 native/WASM capture cases and six real-state projects, all-size-skipped native directory behavior, general project/module checks and fresh proof gates pass.

Remaining partisan Area/VRA/proportional and additional search combinations, full native acquisition/orchestration, capacities, provenance and publication remain incomplete. Native ILP local solutions, fallback plans, report hashes and import consistency do not provide a global optimality or legal certificate. The frozen all-state matrix remains on its retained earlier artifact. Changes are local and unpublished.

Engine artifact SHA-256: 36dd85e5a0dea54e636217d7579403f0e0b50e0c9e17a219ab6f8fb920b10c72.


### Partisan AreaSection with separately bound native candidate cost

The portable engine and laboratory now accept partisan weights with AreaSection Single/Multi. This follows native State's partisan-weighted preset with the ratio-optimal-area structure override. Native ratio/moving-knife initialization, seed schedule, root 1.001 population multiplier and half-area swing constraints remain unchanged. The reference uses identical supplied prepared areas/centroids; it does not reproduce native filesystem TIGER ALAND discovery/preparation.

AreaSection now retains native_candidate_cut alongside the existing reported root weighted_boundary for partisan inputs. The former is the native sparse-boost-only selection cost, measured on original unbiased weights even when moving-knife directional initialization is selected. The latter still includes the reporting convention's implicit unit defaults. This preserves both meanings rather than replacing one with the other. Import independently recomputes the sparse native cost, including selected COI adjustments through the existing edge-weight adapter, and rejects changed candidate cost or constraint claims. Native negative-zero candidate values are preserved; comparisons treat numeric zero consistently.

There are 216 native State/TSV/kernel comparisons on synthetic/actual RI/IA graphs, across neutral/striped/exact-threshold shares, absent/maximum-u64 seeds, one/two/three districts, Single/Multi and both area initializations. Seventy native failures match; all completed assignments, full-plan costs and nontrivial root ratios/native costs match. Twenty-two completed empty-boost-map roots retain native cost zero while reporting a positive boundary. Four real-state configurations save/reopen/export with exact shares; twelve changed source/native-cut/population-multiplier cases are refused. The four unsuccessful project configurations retain native rejection instead of a fabricated success. The actual successful-state browser run retains exact seed and shares, native area constraints and distinct costs, with inert reopen.

Existing State area oracle has ninety assignment/balance/direction matches and two matching failures; initialization, ratio, partisan ILP, configured accepted-attempt replay, general project/module and fresh proof gates pass. Partisan VRA/proportional methods and additional search combinations remain unfinished, along with complete acquisition/orchestration/provenance/capacity/publication and full parity. This is native heuristic behavior on selected prepared inputs, not all-state/multi-district feasibility, global optimization or legal certification. The frozen fifty-state one-district matrix remains on its earlier artifact. Changes are local and unpublished.

Engine artifact SHA-256: 955eed97eaebdff6c21fc49f43e038419aa897ebcc60331b0841f03f6a6bacb9.


### Partisan VRASection source, score and cost boundaries

The portable engine and laboratory now accept partisan weights with VRASection Single/Multi. Native State permits a partisan-weighted preset with a ratio-optimal-vra override. It loads partisan shares independently from the demographic fractions used for the root minority-mass proxy. The original fraction-times-graph-population proxy, alignment formula/weight, normalized sparse-cut score, root-only bonus, strict-first-minimum ratio selection and population-only descendants remain unchanged. This does not add a minority-preservation threshold or reinterpret total-population fractions as exact VAP/CVAP counts.

For partisan VRASection, root weighted_boundary retains its existing native selection-cost meaning. Additional native_candidate_cut and reported_weighted_boundary make both costs explicit: the native sparse-map cut used in the score, and the reporting cost including implicit unit defaults. The browser uses the additional reporting field when explaining that distinction. Import separately checks both cut values, input hashes, mass, alignment and score. Other VRA weighting modes keep their established fields/behavior. The partisan validator is prepared once per VRA verification rather than rebuilt for every cut edge.

There are 288 native State/TSV/demographic-CSV comparisons on synthetic/actual RI/IA tract graphs, across neutral/striped shares, uniform/striped minority fractions, absent/maximum-u64 seed, one/two/three districts, Single/Multi and zero/.4 alignment weights. Actual native CSV loading/alignment supplies the same minority fractions and mass; all assignments, costs, ratios and proxy scores match. Ninety-six empty-boost-map roots retain native cost zero with positive reporting cost. Eight further native cases cover explicit absent-demographics fallback and sparse COI composition, including zero/missing/unknown factors. No automatic source acquisition or native data-directory discovery is inferred from these selected-file tests.

Four real-state projects retain both selected inputs, open inertly and reject twenty changed-source/cut/score claims. Exports retain raw partisan producer data and the canonical demographic input hash; original proxy fractions remain in the laboratory project, while no exact VAP counts or missing demographic producer field are fabricated. Actual state browser execution preserves both inputs, exact visible seed and the two costs. Existing native VRA State oracle has 228 exact assignment/ratio/score cases; area, partisan ILP, configured replay, project/module and fresh proof gates pass.

Remaining partisan proportional-method and additional search compositions, complete native acquisition/orchestration/provenance/capacity/publication and full parity remain unfinished. Hash/score consistency is not authenticated execution, global optimality or legal certification. The frozen fifty-state one-district matrix retains its earlier artifact scope. Changes are local and unpublished.

Engine artifact SHA-256: 143faa5307672e374f519728a8bcf71997d3c94043bc4140508646124158531c.


### Partisan ProportionalSection with independent vote and boundary inputs

The portable engine and laboratory now accept partisan weights with ProportionalSection Single/Multi. Native State supports the proportional-section preset plus proportional weight override and selected partisan TSV. Election counts determine the statewide two-party share and geometric-mean seat quota; the independent partisan input supplies the native sparse boundary map. Native root population/vote constraints, eta, fixed candidate seeds one through budget and population-only GeoSection descendants remain unchanged. The visible optional user seed is retained as a declaration, while the UI keeps its existing fixed-seed policy disclosure. This State evidence does not invent a Proportional Bisect CLI preset; that separate native-kernel API composition remains unfinished.

Native comparison includes 288 actual State parser/TSV/vote-CSV/kernel cases on synthetic/RI/IA graphs, across neutral/striped boundary shares, uniform/striped vote counts, absent/maximum-u64 declared seed, one/two/three districts, Single/Multi and eta1.1/1.5. The 160 native failures are preserved; all completed assignments, root ratios, shares and native sparse cuts match. Sixteen empty-map roots have native cut zero while the separately reported complete-plan cut remains positive. Successful project fixtures are selected from the tested native-completed cases rather than asserting every requested heuristic configuration succeeds. Two plans retain both original inputs in the project and native producer export, open inertly and refuse eight changed-source/cut/seat claims. Two requested state/search project configurations had no successful tested multi-district fixture and are recorded as unavailable, not promoted to success. Actual-state browser execution preserves both inputs, exact visible seed and native fixed-seed/seat-ratio evidence.

Election-result import now computes ProportionalSection's sparse partisan/minority root map and selected COI adjustments, rather than assuming a dense geographic root cost. It prepares the source weight adapters once. Seven geographic/unweighted/county/partisan/minority/economic/housing weight families with COI validate, and seven altered root cuts are refused. Existing native Proportional State oracle has 102 assignment/root matches and six matching failures, including explicit measured final tolerance misses. Shared-kernel API regressions, replay, project/module and fresh proof gates pass. Matching heuristic outputs or a quota do not guarantee partisan outcomes, population acceptance for all settings, global optimization or legal certification.

Source tests explicitly select CSVs/prepared graphs; native filesystem discovery and complete acquisition/orchestration are not inferred. Partisan Proportional Bisect and additional standard search compositions, capacity/provenance/publication and full parity remain incomplete. The frozen fifty-state one-district receipt retains its prior artifact scope. Changes are local and unpublished.

Engine artifact SHA-256: a9c77a644546a736b007dccf6c434ca57ae2f704ac3afbf0e1878cd9ed177ae3.


### Partisan Proportional Bisect kernel API and explicit nullable seed

The portable engine and laboratory now accept partisan weights with Proportional Bisect/Single. This calls the shared native run_all_splits_proportional_recorded API. Native State has no corresponding Proportional Bisect preset; the independent reference explicitly calls the kernel after native TSV/vote-CSV loading rather than manufacturing a State route. Native recursive seat counts still use Democratic votes divided by census population, with nearest rounding/clamped positive children. Two-party counts remain retained election input/evidence; this fix does not substitute them as the recursive denominator. Partisan sparse weights and optional COI adjustment feed native cut construction. Other searches remain refused.

Native comparison covers 144 synthetic/actual RI/IA graph cases across neutral/striped boundary shares, uniform/striped population-scaled vote counts, absent/maximum-u64 seed, one/two/three districts and selected/no COI. All assignments, weighted cuts and recursive split records match the independent native API; seventy-two completed cases include COI. Two real-state projects retain both input sources, reopen inertly, export matching partisan/election producer data and reject six changed-source/schedule claims. Actual-state browser execution retains both inputs and displays the native recursive schedule and requested-seed policy.

The existing proportional split summary stored seed0 for an omitted seed even though the kernel received None. A new requested_seed field retains the actual nullable seed while preserving the legacy seed field for compatibility. Import checks new fields when present and accepts historical omission as an imported claim. None, explicit0 and maximum-u64 records are distinct; three altered optional-seed declarations are refused. Browser text now shows backend default for None, rather than presenting zero as a supplied seed. The kernel's seed argument and behavior are unchanged.

The broader proportional API regression has 270 exact outputs, 180 matching failures and 4,710 tamper refusals. ProportionalSection, configured original-source/accepted-attempt replay, project/module and fresh proof gates pass. All-state/parameter feasibility, additional standard partisan search combinations, full native acquisition/orchestration/provenance/capacity/publication and complete parity remain unfinished. Historical missing optional fields do not authenticate execution. This heuristic schedule, native cutoff and hash consistency do not establish global optimum, partisan outcomes or legal certification. The frozen fifty-state one-district matrix remains on its retained earlier artifact. Changes are local and unpublished.

Engine artifact SHA-256: 977a1ed64f304a219d50e9413b02710738c8b066fb3dc4da85c7330faa34cd85.


### Partisan full-plan Flip, forest ReCom and merge-split chains

Partisan standard bisection now supports Flip, forest ReCom and merge-split. Each calls the existing shared native chain with the native sparse partisan weights, including optional COI adjustment. Weights affect METIS initialization; chain proposals and percentile ranking retain unweighted adjacency/cut semantics. Flip initialization uses native fixed 100 refinement iterations; forest ReCom and merge-split use requested iterations. Missing seeds retain the native zero default; explicit maximum-u64 seeds remain exact. Separate METIS objective/trial controls remain refused for these native chain APIs. This is full-plan sampling after initialization, not a new local best-cut criterion.

Independent native State conversion and TSV loading match 288 synthetic/actual RI/IA cases, including 0 matching native failures and 144 completed COI cases. Cases include neutral/striped shares, absent/maximum seed, one/three districts, zero/four proposals and both percentile endpoints. Six actual-state laboratory projects retain share sources and practitioner exports, reopen inertly, and reject twelve source/chain-evidence changes. Actual Chrome Worker execution covers all three chains with visible exact seeds and retained shares. Fresh proof gates, project/module/package integrity and option inventory checks pass.

This is selected composition evidence; remaining short-burst/tempering/multiscale/other search compositions and wider native acquisition/orchestration/capacity/provenance/publication parity remain unfinished. Matching heuristic outputs and hashes do not authenticate original execution, establish a global optimum or certify legal compliance. The frozen fifty-state one-district matrix retains its earlier artifact identity. Changes are local and unpublished.

Engine artifact SHA-256: 68069348da90fb5a3af48a3208d0fce87c153851c3ea82bd26d77554f442c6ab.


### Partisan short-burst ReCom, forest and merge-split searches

Partisan standard bisection now supports short-burst ReCom, forest and merge-split. Each calls the existing shared native chain with the native sparse partisan weights, including optional COI adjustment. Weights affect METIS initialization; burst proposals retain native unweighted adjacency and selection ranks burst endpoints by unweighted cut, with insertion-index ties. All three methods use requested initialization iterations. Bursts restart from the preceding endpoint, rather than choosing the minimum visited inside each burst. Missing seeds retain the native zero default; explicit maximum-u64 seeds remain exact. Separate METIS objective/trial controls remain refused for these native chain APIs. This is full-plan sampling after initialization, not a new local best-cut criterion.

Independent native State conversion and TSV loading match 576 synthetic/actual RI/IA cases, including 0 matching native failures and 288 completed COI cases. Cases include neutral/striped shares, absent/maximum seed, one/three districts, zero/nonzero burst counts and lengths and both percentile endpoints. Six actual-state laboratory projects retain share sources and practitioner exports, reopen inertly, and reject twelve source/burst-evidence changes. Actual Chrome Worker execution covers all three chains with visible exact seeds and retained shares. Fresh proof gates, project/module/package integrity and option inventory checks pass.

This is selected composition evidence; remaining tempering/multiscale/other search compositions and wider native acquisition/orchestration/capacity/provenance/publication parity remain unfinished. Matching heuristic outputs and hashes do not authenticate original execution, establish a global optimum or certify legal compliance. The frozen fifty-state one-district matrix retains its earlier artifact identity. The ReCom burst seed vector/selected burst index from the native tuple is not yet retained in the WASM result or project. Matching assignments do not establish that ledger parity. Changes are local and unpublished.

Engine artifact SHA-256: b88e4484b5d4698408c6a81455c7ccaf656237dc12b39e9947c546c9f736892a.


### Native short-burst ReCom seed ledger retained

WASM now retains the native short-burst ReCom tuple’s burst seed vector and selected burst index. Seeds are exact decimal strings, including values above the JavaScript safe-integer range. This metadata is present for both explicit burst controls and the legacy step-budget route (length20, ceiling(steps/20) bursts). One-district plans still have no split/chain evidence. Zero bursts retain the native empty ledger and index0; zero-length bursts preserve native cut ties and insertion-index percentile selection. Forest and merge-split APIs return assignments without this tuple; no ledger is fabricated for those methods.

Project Open checks exact ledger shape, count, u64 representation and index bounds; it recomputes each native SHORT_BURST_CHAIN_ SHA-256 seed from the requested/default0 run seed and zero-based burst index. Zero-work selections are checked against the known native tie behavior. Positive-work selected endpoint indices remain native execution claims; the ledger does not independently establish the chain trajectory or cut ordering. Historical missing ledgers remain accepted as imported claims. The UI lists exact seeds and the selected index in an expandable panel and clears it on other runs.

The updated independent State/TSV matrix has 576 exact comparisons, including 96 nontrivial ReCom ledger comparisons against native string-preserved seeds and indices; other methods and one-district cases retain their original scopes. Twelve independent Node crypto cases reject 54 invalid/altered ledgers. Two actual-state legacy/zero-length saved projects reopen inertly and reject seven changed seed/ledger/index claims. Three actual Chrome Worker runs confirm ledger display and clearing. The 144-case generic burst regression, fresh proof gates and project/package checks pass. Artifact SHA-256: 10e66cc795753802097793e99092c00b1bac0f2d0eeffa3f1c01ef128bd84f38.

Remaining search compositions and wider native acquisition/orchestration/capacity/provenance/publication parity are still unfinished. Matching hashes and recorded indices do not authenticate execution or establish global/legal certification. Changes remain local and unpublished.


### Partisan parallel tempering

Partisan standard bisection now supports parallel tempering. Each calls the existing shared native chain with the native sparse partisan weights, including optional COI adjustment. Weights affect METIS initialization; replica moves, exchanges and cold-record percentile ranking retain native unweighted adjacency/cut semantics. Initialization uses the requested cold tolerance and refinement iterations. Swap support requires both receiving tolerances. Run balance tolerance independently checks the resulting plan; it is not substituted for the cold tolerance. Missing seeds retain the native zero default; explicit maximum-u64 seeds remain exact. Separate METIS objective/trial controls remain refused for these native chain APIs. This is full-plan sampling after initialization, not a new local best-cut criterion.

Independent native State conversion and TSV loading match 192 synthetic/actual RI/IA cases, including 0 matching native failures and 96 completed COI cases. Cases include neutral/striped shares, absent/maximum seed, one/three districts, zero/four tempering steps, one/three replicas, swap intervals1/3, equal and distinct cold/hot tolerances, and both percentile endpoints. Two actual-state laboratory projects retain share sources and practitioner exports, reopen inertly, and reject four source/tempering-evidence changes. Actual Chrome Worker execution covers this compositor with visible exact seeds and retained shares. Fresh proof gates, project/module/package integrity and option inventory checks pass. The broader PT regression has 288 exact native/WASM cases and nine matching invalid-input rejections. The independent State tolerance regression has 196 exact native matches, including 42 final-balance misses; those misses remain reported and do not become feasible because cold/hot sampling tolerances were looser.

This is selected composition evidence; remaining multiscale/other search compositions and wider native acquisition/orchestration/capacity/provenance/publication parity remain unfinished. Matching heuristic outputs and hashes do not authenticate original execution, establish a global optimum or certify legal compliance. The frozen fifty-state one-district matrix retains its earlier artifact identity. Changes are local and unpublished.

Engine artifact SHA-256: 22a77b8100abeb51ed8ac5939f53440e9cd33c2ff787d159ba9365bd942c13cd.


### Partisan SMC reporting with native weight-independent sampling

The laboratory and portable engine now accept partisan weights with standard SMC percentile. Native SMC does not consume boundary weights: proposal generation, importance weighting, resampling and selected unweighted-cut percentile remain unchanged. Partisan/optional COI weights affect reported boundary costs. The fixed sampler tolerance remains 0.5% of remaining component population; requested final balance is an independent check. None keeps native seed0, while maximum-u64 seeds stay exact. The UI explains the weighting scope; no weighted-proposal algorithm or distribution guarantee is invented.

Independent native State conversion, TSV loading and dispatch match 96 synthetic/actual RI/IA cases, including 4 matching failures and 46 completed COI cases. Tests cover neutral/striped shares, absent/maximum seed, one/two districts, four/eight particles, resampling thresholds0/1 and percentile endpoints. Successful assignments also match unweighted SMC replays. Two actual-state projects retain share inputs and practitioner exports, reopen inertly and reject four altered source/sampler claims. One actual Chrome Worker run retains shares and the exact seed. Fresh proof gates and project/package/inventory checks pass. Artifact SHA-256: 98cac5775e085ad0822a764f99654721681f46d364f63f6361c183b85623af7c.

Additional multiscale/VRA search compositions and native acquisition/orchestration/capacity/provenance/publication parity remain unfinished. Matching sampled plans and hashes do not authenticate execution, calibrate a sampling distribution, prove global optimum or certify legal compliance. Changes are local and unpublished.


### Partisan VRA ReCom with independent demographic preservation

Standard VRA ReCom now accepts partisan boundary weights together with a separate, complete demographic fraction input. The native sparse partisan/optional COI map affects initialization and reported costs. Proposal moves, MH decisions and cut-percentile ranking keep native unweighted adjacency; demographic preservation uses the unweighted tract mean in each district, with protection determined from initialization. Fractions retain the total-population basis rather than pretending to be observed VAP. The fixed proposal tolerance is 0.5%, with independent final balance checks.

The independent reference uses actual native State conversion and native TSV/demographic CSV loaders followed by shared detailed dispatch on identical supplied graphs. It explicitly receives the demographic CSV path; this is not native default-path discovery or missing-file no-op parity. 192 synthetic/actual RI/IA cases match, including 0 native failures and 96 completed COI cases. Checks cover neutral/striped shares and corresponding distinct demographic profiles, missing/maximum-u64 seed, one/two districts, zero/four proposals, thresholds0.4/1 and percentile endpoints. Complete native initial assignments, protection lists, proposal/rejection accounting and selected rank match. Two actual-state projects preserve both inputs, reopen inertly, export partisan source data and reject six source/preservation changes. One actual Chrome Worker run retains both sources and the exact seed. Fresh proof and project/package/inventory gates pass. Artifact SHA-256: 41131423b0905350198fd1ab0f4645d3728a4de3a6572d2b271324a43d061190.

The preserved mean-fraction rule is a geographic heuristic, not a legal VRA certificate. Matching evidence and source hashes do not authenticate the original chain trajectory or establish global optimum. Remaining multiscale compositions and full acquisition/orchestration/provenance/capacity/publication, including native demographic discovery/absence semantics, remain unfinished. Changes are local and unpublished.


### Multiscale partisan initialization API; laboratory exposure pending

The in-memory multiscale API now accepts complete partisan shares for the parent tract graph, using the shared native sparse partisan builder before optional native COI post-weighting. Both fixed and adaptive multiscale kernels accept the same weights for tract initialization. Tract/county, block-group/tract and block-group/county pairings are supported at the API boundary. Fine/coarse Wilson sampling and percentile ranking remain unweighted. Evidence names this partisan-tract-metis initialization and retains source hash, thresholds, strong-tract count, alpha and the parent-only scope. Complete source validation also applies to export; unused shares on a geographic request are refused. Exports retain the raw shares inside the hashed multiscale producer input and preserve physical fine boundary weights.

An independent native kernel oracle uses the native TSV and COI loaders/post-weighter and shared portable multiscale kernels. All288 synthetic/actual RI/IA parent-graph cases match assignments and adaptive diagnostics, covering all pairings, fixed/adaptive kernels, neutral/striped shares, None/maximum-u64 seeds, zero/two proposals and absent/present COI. The block-group graphs are explicitly synthetic two-child projections that preserve parent population, not original Census block-group data. Eighteen exports preserve source inputs; three missing/incomplete/unused source requests are refused. Existing geographic laboratory coverage passes with24 state/national projects,96 native comparisons and120 tamper refusals. Fresh proof gates and package checks pass. Artifact SHA-256: d0b580624e45622b7a8719a0ba95020bd7885b426499056c81e6bccf6621d0df.

This is an API addition. Laboratory weight controls, partisan multiscale project adaptation/import checks and appropriate cost labels are still pending; the existing UI continues to refuse this combination. Parent tract shares must not become invented block-group observations or a claim that fine physical costs are partisan costs. Native State data acquisition and manifest reporting are not verified by the kernel oracle; in particular its final edge-cut reporting uses tract edge indices against the returned assignment map, so block-group reporting needs a separate explicit treatment. Real Census block-group coverage, native acquisition/orchestration/provenance/capacity/publication and complete parity remain unfinished. Matching kernels and hashes do not authenticate execution or prove optimality/legal certification. Changes are local and unpublished.


### Partisan multiscale laboratory controls and safe project import

The laboratory now exposes partisan initialization for fixed and adaptive multiscale runs across tract/county, block-group/tract and block-group/county. Controls retain complete parent-tract share sources, thresholds and exact seeds. The run adapter forwards parent shares to the existing native multiscale API; exports retain them in the hashed multiscale producer input. Saved-project validation checks parent source identity, counts, thresholds, adaptive alpha and initialization scope against the tract graph before verifying the fine assignments. It does not invent shares for block-group children. Fine-map costs and exported context weights remain physical boundaries and are explicitly described as such.

COI evidence now checks the actual computed seed-edge map. For partisan initialization that map is sparse: only boosted parent edges exist, so neutral shares have zero applied edges. Geographic initialization still uses every physical parent edge. Both use native COI multiplication, with unknown and block-group-only IDs counted but not mapped to parents. Import rejects altered sources, initialization scope, count/cost claims and nonnumeric alpha; opening a project invokes no engine.

Synthetic three-state tests cover48 state/national projects across all pairings, fixed/adaptive methods, Native Build selected/unselected and neutral/striped shares. They have192 exact Native host comparisons,432 changed-project refusals and source-preserving exports. Four additional actual RI/IA tract projects cover both methods, export and inert Open using synthetic shares on the actual prepared graphs. Actual Chrome Worker execution on the explicitly synthetic block-group fixture verifies source retention, visible maximum-u64 seed, physical cost text, practitioner export, COI clear/replacement and delayed-read guards. Existing geographic multiscale coverage remains green, and fresh proof/package/inventory gates pass. The independent API oracle from the preceding gate retains288 native kernel/TSV/COI comparisons across all pairings. Artifact SHA-256: d0b580624e45622b7a8719a0ba95020bd7885b426499056c81e6bccf6621d0df.

The former API-only compositions are now exposed, but this does not establish all-state/parameter/capacity feasibility, authentic Census block-group coverage or Native State acquisition and manifest cost reporting. The fine physical cost view does not reproduce Native State’s tract-index lookup against block-group assignment indices. Complete native acquisition/orchestration/provenance/capacity/publication parity remains unfinished, and imported records/hash matches do not authenticate execution or prove global/legal certification. Changes remain local and unpublished.


### Explicit native VRA ReCom absent-file zero fractions

The existing explicit vra_absent_file_fallback option now also covers standard VRA ReCom, matching the native missing-file branch that supplies zero fractions. No selected demographic input is permitted with the flag, and ordinary missing/incomplete input still rejects when it is omitted. Minority boundary weights remain incompatible with absence. The laboratory checkbox describes zero fractions for ReCom and plain GeoSection for VRASection. Projects retain explicit-native-absent-file-zero-fractions evidence and a null demographic identity rather than a fabricated observed source. Practitioner exports carry the option without demographic observations.

Native State conversion/partisan TSV loading plus the explicit zero-vector branch match288 synthetic/actual RI/IA graph cases, including144 COI cases. Both native assignments and full preservation records match across thresholds0/0.5/1, None/maximum-u64 seeds, one/two districts, zero/four proposals and percentile endpoints. Threshold0 protects every initial district even with zero fractions; positive thresholds protect none in this branch. Two projects retain the declaration, reopen inertly and reject six altered source/preservation claims; ordinary omission, supplied data and false flags are also refused. Initial assignments remain strictly checked for valid labels and nonempty districts. Actual Chrome Worker execution confirms explicit selection, no demographic input and inert Open. The existing432-case supplied-input ReCom regression and36-case VRASection absence regression pass; a preexisting extra parenthesis in the latter harness was corrected so it could run. Fresh proof gates and package/project/inventory checks pass. Artifact SHA-256: e57602b29d9714376cafb809ffa6dc71658a4189fd140afc31b914cb58137860.

The zero-vector comparison verifies the selected native branch; it does not establish that a file is absent on another filesystem, reproduce native data discovery, authenticate execution or certify legal compliance. Full acquisition/orchestration/provenance/capacity/publication parity remains incomplete. Changes remain local and unpublished.


### Manifest ILP report-count consistency before certificate checks

The shared native manifest ILP verifier now checks an explicitly declared ilp_solve_report_count against the number of independently audited reports. The check occurs after report/model/summary verification and before manifest certificate checks. A two-report directory with two individually valid reports and a correctly hashed/matching two-report summary cannot pass a manifest that claims one report. Zero, undercounts and overcounts are refused. Missing historical count fields retain existing behavior; absent summary references still return not-recorded rather than authenticating report counts. No report file is removed or rewritten by verification.

The shared verifier has11 native tests, including matching/mismatching counts and typed count rejection. The updated gate passes22 Native host/WASM cases and16 actual filesystem-backed CLI Verify cases, with five existing input-boundary refusals. New cases cover matching count, zero/overcount and a mixed-directory mismatch whose reports and summary otherwise pass. On mismatch, computed valid-report diagnostics remain available and certificate checks stay not-run. Actual Chrome Worker execution confirms the mismatch, failed receipt preservation and inert Save/Open. Fresh proof, project/package and option-inventory checks pass. Artifact SHA-256: ae1dc8af0536675e0ff38f498752b39c8cfac927f2451ac8d66eb1f5f313bb4e.

This checks consistency of the selected/audited inventory, not whether a manifest authentically describes a historical execution or whether selected files are the complete external directory. The earlier native generation gap remains: mixed existing report directories are still scanned when writing an audit summary. The verifier now refuses an inconsistent generated manifest, but generation has not yet been isolated to current-invocation reports. Broader native acquisition/orchestration/provenance/capacity/publication and full parity remain unfinished. Changes are local and unpublished.


### Current-invocation ILP report selection in generated summaries

ILP generation now tracks the exact reports successfully written by the current invocation and builds its audit summary from those paths, rather than rescanning old report files. A generated summary retains sorted, unique, portable report_selection paths relative to its report directory. Old skipped-root/unrelated report files are preserved, and no zero-report summary is fabricated. Native manifest verification and browser selected-inventory prechecks honor the explicit scope, independently audit those reports/models, compare typed summary records and then enforce the declared report count. Verified receipts retain the selection.

Historical summaries without a selection still use native whole-directory/report-name discovery. Explicit whole-selection artifact audits continue to audit every report selected by the operator; a scoped summary cannot hide extra explicitly audited reports. Scope validation rejects empty/duplicate/unsorted lists, unsafe paths and declared-count mismatches. Missing selected reports fail verification. A selection identifies what is being checked; it is not proof of current execution or external inventory completeness.

Twenty-three native ILP tests pass, including mixed size-skip generation with malformed old root/unrelated reports: only the two newly written child reports are summarized and verified, old bytes remain untouched, and removing a selected child causes verification failure. Fresh Native State reference/WASM capture comparison retains exact report/model pairs with only measured solve time normalized. Twenty-seven Native host/WASM precheck cases and21 actual CLI Verify cases pass, including scoped verification beside stale files, unsafe/duplicate/empty/missing selection and legacy whole-directory count mismatch. Two additional Native host/WASM artifact-audit cases show that extra explicit inputs remain audited. Actual Chrome Worker execution verifies the scoped report, preserves stale source bytes and the scoped receipt, and reopens inertly. Fresh proof and package/project/inventory checks pass. Artifact SHA-256: d35c0e1064e29eb713a87859a0408dc5707f77127824bb33eddd991125c72380.

The previously recorded mixed-directory generation issue is resolved for successful invocations with reports. Native same-path report/model writes can replace earlier versions at those paths; this change does not retain a full execution archive, clean old files or prove provenance. Historical unscoped summaries, source acquisition/orchestration, arbitrary capacities, publication and complete parity still require further work. Changes remain local and unpublished.


### Verified ILP artifact-summary scope survives output and download

The shared audit helper now retains an explicit report_selection after the supplied summary matches all independently audited records. Native ilp-audit output and WASM artifact-audit summary/summary_json carry the same validated scope, so the practitioner download no longer silently turns a scoped summary into an unscoped directory summary. Comparison failure leaves computed records unchanged and does not copy a mismatched or unsafe supplied selection. Historical summaries without a selection still produce unscoped output. Scope consistency is distinct from per-report success; overall failed report/model status is still reported independently.

Eleven native ILP-audit tests and a fixture-backed shared integration test pass. The integration test retains a matching scope, refuses unsafe/incorrect/empty scope without mutating computed records, and removes scope when a matching historical summary is supplied. Four Native host/WASM and four actual filesystem CLI cases cover scoped/legacy match and extra-report/unsafe mismatch. Summary download bytes match displayed records. Actual Chrome Worker execution retains the scope in its downloaded summary and saved receipt and reopens inertly. Existing27 Native host/WASM and21 CLI manifest prechecks, the explicit full-selection audit gate and fresh proof/package/inventory gates pass. Artifact SHA-256: 1124c02f19b137d357445743c73ff8d6ce392b20b6e4305bcc8c01ec0d2091ee.

This preserves a validated declaration about audited records, not authenticated historical execution or external directory completeness. Explicit extra reports remain audited; a scoped supplied summary cannot conceal them. Whole native workflow/acquisition/orchestration/provenance/capacity/publication and full parity remain unfinished. Changes are local and unpublished.


### Explicit report-scope choice preserves the full practitioner inventory

The ILP artifact auditor now offers All selected reports (the unchanged default) or Reports named by saved summary. The latter is an explicit input-selection choice, not a verification result. Preparation requires a sorted unique safe list, matching declared count and every named report present in the loaded inventory. The interface lists retained reports outside the selected Run and states that the list is unverified until Run. Native auditing still verifies report/model/summary records; a declared list alone grants no authority.

The request builder selects named reports while preserving all report/model/summary source bytes in the saved project. New bounded scope settings roundtrip safely; changing scope retires the prior result and runs nothing. Replacing sources, removing the summary or clearing sources resets to All selected reports. Legacy projects omit settings and retain the old default. Invalid scope settings or unsafe/incomplete lists reject before execution.

Two scope requests match the Native host exactly: the default with two reports and a one-report summary fails, while an explicit one-report request verifies and retains its scope. Seven invalid list/settings cases are refused and both modes preserve the original inventory in project roundtrips. Actual Chrome Worker tests cover default failure, preparation without execution, verified subset, both reports retained through Save/Open, restored mode without execution and reset on summary removal. Existing default-mode browser behavior, project/module suite and package/inventory checks pass. The Rust/WASM artifact is unchanged from the preceding verified gate: 1124c02f19b137d357445743c73ff8d6ce392b20b6e4305bcc8c01ec0d2091ee.

This supports using a current-run summary beside stale files while preserving the operator’s source inventory. It does not authenticate execution, establish external directory completeness or silently remove sources from an all-selected audit. Native full workflow/acquisition/orchestration/provenance/capacity/publication parity remains unfinished. Changes remain local and unpublished.


### ILP report-byte preservation and native malformed-report diagnostics

ILP artifact folder/project import now retains bounded report bytes without assuming that every selected report is valid UTF-8. The new optional reports_base64 API representation passes raw selected reports to the shared native byte auditor, preserving failed-record diagnostics for invalid encodings and empty/malformed reports. Text reports remain supported; simultaneous text and byte representations are refused. The existing file/path/case-collision, 512-file and decoded byte limits remain enforced. Models retain their existing byte representation.

The browser request helper uses text for valid UTF-8 and byte reports when selected bytes cannot decode. UTF-8 decoding now preserves a leading BOM rather than silently removing bytes before hashing or audit. Summaries still require valid bounded JSON; report bytes remain inert data until explicit Run. A malformed stale report can stay in the saved inventory and be explicitly excluded by a safe complete summary list; All selected reports still audits it and shows failure.

Five Native host/WASM comparisons cover raw invalid UTF-8, BOM and empty reports plus both full/declared-list requests. Three actual filesystem CLI verdicts match checked/failed counts. Report SHA-256 values match the exact retained bytes. Two ambiguous/invalid encoding requests are refused. Project roundtrips retain the invalid stale bytes and both modes preserve the full inventory. Actual Chrome Worker tests load/save/open those bytes, show a failed stale report in the default audit, and verify the explicitly named valid report without executing on preparation/Open. The existing15 Native host/WASM/four CLI ILP audit cases and helper bounds/binary-model tests pass. Existing27 Native host/WASM/21 CLI manifest prechecks, fresh proof and project/package/inventory gates pass. Artifact SHA-256: 90a613381c65fc2957fbdd76e6db523d3533b292a7421e72f862b9096d9c58fa.

The actual CLI comparison checks verdict/count parity for malformed files, not identical filesystem error-path text. Matching report/model hashes and a scoped result do not authenticate execution or establish external completeness/global/legal certification. Broader Native workflow/acquisition/orchestration/provenance/capacity/publication and full parity remain unfinished. Changes are local and unpublished.


### Unweighted and county multiscale initialization in the laboratory

Fixed/adaptive multiscale API and laboratory controls now expose unweighted and county initialization alongside geographic and partisan modes. Unweighted initialization passes the native empty computed-weight map and retains native no-effect COI status. County initialization uses physical parent-tract boundaries with the native additive same-county multiplier (1+alpha), disabled at alpha<=1e-10, using the first five GEOID digits. County alpha remains a separate control from coarse-move probability. Seed evidence binds these policies and parent-only scope. Fine/coarse sampling and percentile ranking remain unweighted; fine-map costs and exported contexts remain physical boundaries for all four modes.

The independent native kernel/GeographicWeighter/SubdivisionWeighter reference matches720 synthetic/actual RI/IA parent-graph cases over all three pairings, fixed/adaptive kernels, None/maximum-u64 seed, zero/two proposals and absent/present COI. County alpha0/1e-10/0.5/2 and unweighted are covered; ninety exports retain the chosen weights and alpha. Block-group inputs in this gate are synthetic population-preserving children, not authentic Census BG observations. Forty-eight synthetic state/national laboratory projects have192 Native host comparisons and288 source/scope tamper refusals, with Native Build selected/unselected. Eight actual prepared RI/IA tract projects cover both modes/methods, export and inert Open. Browser Worker execution covers county and unweighted controls and no-effect COI. Geographic and partisan laboratory regressions, fresh proof, project/package and declaration-inventory gates pass. Artifact SHA-256: 7ff7f294263f3868639137144af0fd07942ddbf0fafacfcdf28f7dc95979dee6.

This adds native-supported initialization choices rather than weighting the samplers or inventing fine-unit preferences. Additional demographic/character initialization families, authentic Census BG acquisition/capacity, complete Native State orchestration/manifest reporting, provenance and publication/full parity remain unfinished. Matching kernels and hashes do not authenticate execution or prove optimum/legal certification. Changes are local and unpublished.


### Minority-weighted multiscale tract initialization without preservation claims

The multiscale API and laboratory now expose native minority-aligned initialization. Complete parent-tract total-population fractions feed the shared native sparse override builder with fixed40% cutoff and adaptive alpha. Evidence retains operational fraction identity, high-tract count, alpha, basis and parent-only scope. Optional COI applies only to computed boosted tract edges, including an empty map for neutral fractions. Fine/coarse sampling and percentile ranking stay unweighted; this mode does not add VRA ReCom preservation or fabricate fine-unit observations. Fine-map costs remain physical boundaries.

Native CSV loading/alignment and MinorityOverrideWeighter plus shared kernels match288 synthetic/actual RI/IA parent-graph cases across all three pairings, fixed/adaptive, neutral/boundary-threshold profiles, None/maximum-u64 seed, zero/two proposals and absent/present COI. All assignments and adaptive diagnostics match. Eighteen exports retain sources; three missing/incomplete/unused input cases are refused. Block-group children in this gate are synthetic population-preserving projections. Forty-eight synthetic state/national projects with Native Build selected/unselected have192 Native host comparisons and432 source/scope tamper refusals. The laboratory fixtures include synthetic parent counts under the native counts-v2 schema; source/count bytes survive export but the fine demographic table stays hidden. Four actual RI/IA tract projects cover both methods, export and inert Open. Actual Chrome Worker checks source retention, exact seeds, physical cost wording, COI changes and parent-count display boundary. Basic/partisan regressions, fresh proof and project/package/inventory checks pass. Artifact SHA-256: 9ac01334ccbe126d11e1906c9492fc5d70bce91cfdf97c0ae2a980f33066205c.

Operational fraction identity deliberately does not authenticate optional count observations or their provenance. Parent minority fractions and synthetic counts are not VAP observations or legal certification. Character initialization families, authentic Census BG acquisition/capacity, Native State discovery/absence/orchestration/manifest reporting, provenance/publication and full parity remain unfinished. Changes are local and unpublished.


### Economic and housing character multiscale parent initialization

Economic and housing character weights are now available for fixed and adaptive multiscale in the WASM API and state/national laboratory. Complete parent-tract character inputs feed the shared character builder and native economic/housing weighters. The geographic blend alpha, observation year, operational character identity, formula and distinct economic/housing zero-vector policies are retained in initialization evidence. COI post-weighting remains on parent seed edges. Fine/coarse sampling and percentile ranking stay unweighted; fine map costs and practitioner export contexts use physical boundaries. No fine-unit observations are inferred from the parent data. All seven current weighting families are exposed for these multiscale methods.

The native aligned-observation weighter and portable-kernel reference matches864 cases across synthetic and actual RI/IA parent tract graphs, three resolution pairings, fixed/adaptive, both kinds, alpha0/0.5/1, zero vectors, None/maximum-u64 seeds, zero/two proposals and absent/present COI. All assignments and adaptive diagnostics match;108 exports retain the request. Fifteen malformed/missing/unused/scope/blend/raw-count cases are refused by both the Native host and WASM. This proves the aligned-observation calculation, not Native State filesystem discovery or the Native CSV acquisition pipeline. Imported character CSV fixtures use synthetic observations and explicit observation-year metadata. Block-group children in this matrix are synthetic population-preserving projections, not authentic Census BG inputs.

Forty-eight synthetic state/national projects with Native Build selected/unselected have192 full Native host comparisons and384 import tamper refusals. Eight actual RI/IA tract projects cover both character kinds and both methods, source-preserving export and inert Open. Actual Chrome checks economic Worker/native-fixture equality, visible blend/source controls, exact seed, physical-cost wording, native practitioner export, COI changes and housing selection/run/Save/inert Open. Basic, minority and partisan multiscale regressions pass. Fresh proof-model and bounded proof-pipeline checks, generic project checks, package/inventory/composition checks pass. Artifact SHA-256: 8eee3561652b1d2f0db1616d3a9dffc141d94ed0162c2ba366424c5896594313.

Full parity remains unfinished: Native source discovery/acquisition and orchestration, authentic fine-data capacity, provenance/publication and other workflow gaps remain separately tracked. Character source hashes identify operational inputs; they do not authenticate observations or establish legal certification. This is heuristic search with unproved optimality. Changes are local and unpublished.


### Paired local Census ZIP import at every polygon resolution

The paired local TIGER/PL94 importer now exposes counties, tracts, block groups and blocks. Resolution is validated before archive reads and retained in the TIGER source selection. Tracts retain the existing prepare-tract-engine-input path; other polygons use the existing native Census preparation adapter. The panel remains available after switching to Census preparation. Open restores saved source resolution and PL94 vintage. Both archives must pass extraction before replacing inputs; state/resolution/population joins remain checked by explicit Run. Save retains selected parts, not the original ZIP containers.

Twelve synthetic ZIP cases cover all four resolutions and Census vintages2000/2010/2020, exact retained WASM fixture graph/map values, Native host discrete preparation values, source-preserving project/archive roundtrips and four invalid-resolution pre-read refusals. Native projected floats differ by at most 9.022187441587448e-10 under the existing1e-5 bound; byte-identical graphs across targets are not claimed. The tract adapter retains its legacy implicit tract graph kind. An authentic local RI2020 block-group ZIP plus original PL94 ZIP produces792 units and1,097,379 population. Original TIGER source hashes were rechecked and current Native filesystem preparation matches discrete graph/population/topology/display values, with maximum projected float delta 8.446079391433159e-09. Source archive and exact prepared graph/map handoff pass. This actual-data gate prepares inputs; it does not run a plan.

Actual Chrome covers county/BG/block ZIP workers, graph download, exact retained source parts, corrupt companions, cancellation/stale-edit atomicity, busy controls, inert Open, resolution-specific native TUI handoff and completed synthetic assignments matching the Native host. The existing tract browser and paired-file tests pass. Existing paired provider acquisition, project security, staged package and option-inventory checks also pass. No Rust engine changes were needed; the verified WASM artifact remains 8eee3561652b1d2f0db1616d3a9dffc141d94ed0162c2ba366424c5896594313.

This closes a local source-exposure gap, not Native provider acquisition or all-state orchestration/capacity. The provider acquisition path still queues its supported tract source pairs. Original large block-state capacity, full Native State/States discovery/reporting/provenance/publication and the complete WASM objective remain unfinished. Changes are local and unpublished.


### Explicit native character alignment with retained original observations

The character input adapter now accepts an optional alignment receipt containing the original nonempty character source data, matched count, sorted missing/ignored tract IDs and explicit-native-nonempty-source-alignment-v1 policy. A new align-character-input operation produces complete target-tract input using the historical Native alignment rules: missing economic vectors/raw counts become zero; missing housing vectors become neutral0.5 values in all four components. Original observations remain distinct from defaults. Unmatched same-state source tracts are retained in source_data and listed as ignored. Empty/missing-file geographic fallback remains a separate unfinished policy; no empty character source is accepted as alignment. Existing complete inputs remain unchanged.

Rust build_weights recomputes alignment from retained source_data and target GEOIDs before using aligned observations. JavaScript input/project validation independently enforces the same receipt and vectors. All callers sharing this builder, including ordinary and multiscale runs and practitioner export, receive the check. The laboratory exposes an explicit alignment button and shows observed/defaulted/ignored counts. Alignment fetches the hashed selected parent graph and stages all selected states before changing inputs. It executes no plan. Source edits, Clear or scope changes invalidate in-flight adoption. Save/Open and exported character inputs retain original sparse observations and receipt.

Eighteen alignments agree across JavaScript, WASM and the Native host over synthetic and actual RI/IA parent tract graphs with partial, fully unmatched and full-plus-extra synthetic sources. Fifty-four independent Native align_lodes_to_adjacency/align_housing_to_adjacency plus weighter cases match both kinds and alpha0/0.5/1, including actual housing neutral defaults. One hundred eight altered output/source/receipt cases are refused. Forty-eight multiscale state/national projects have192 full Native host matches and480 import tamper refusals. Sixteen ordinary single/multi state/national projects with Native Build on/off have64 full Native host matches, source-preserving export and inert Open. Actual Chrome exercises both explicit alignment actions without run requests, later Worker execution, exact seed, original sparse-source Save/Open/export, native fixture equality, housing selection and COI changes. Complete-input multiscale, CSV and refusal regressions pass. Fresh proof-model/bounded proof-pipeline and project/package/inventory checks pass. Artifact SHA-256: d6db157e8243eb91cd3c5f4049967785807f8d259c2453fc4baeabb2c0cc427e.

These comparisons exercise supplied observations and actual Native alignment/weighter functions. They do not prove Native CSV filesystem discovery, provider acquisition, missing-file/error geographic fallback, authentic observation provenance or all-state fine capacity. Character identity hashes the operational aligned vectors; the retained original source is not authenticated by that identity, particularly ignored observations. No legal certification or optimality is established. Full WASM parity remains unfinished. Changes are local and unpublished.


### Prepared character source handoff from practitioner tools to laboratory

Fresh LODES workplace and ACS housing preparation results now expose Use character input in laboratory. The sender builds a bounded preparation project containing the original selected source/settings and result, validates the character kind and scope, and queues it using the existing same-origin15-minute/25MiB one-use project transfer. The link opens a separate tab with noopener; a visible fallback link remains if opening is blocked. Imported saved results require a fresh preparation Run before this action is available. Source edits, result changes and busy state prevent stale transfers. Commuting-only or unavailable character outputs are not eligible. The original practitioner tab and its source selections remain intact.

The laboratory consumes and parses the packet under existing project bounds, validates preparation operation/result/type/source scope and available tract-graph scope, then selects state, Census year, character kind and observation year. Experiment options open for review. It performs no alignment or plan execution. Alignment stays an explicit action, followed by explicit Run. The consumed token is removed from the URL and replay is rejected. Preparation sources stay in the practitioner project; the laboratory experiment retains the selected character input and, after alignment, the original sparse observations/alignment receipt. The ephemeral transfer does not make the full original ACS/LODES source part of a laboratory experiment or authenticate observations.

Two fresh preparation projects match the Native host exactly; original source projects and detached input copies pass13 unavailable/type/scope refusals and three deployment-prefix URL checks. Actual Chrome exercises both real preparation Workers and transfers, separate tabs without opener access, zero run/alignment requests on arrival, consumed-token refusal, intact practitioner source Save, explicit alignment/Run, two Native host assignment matches, maximum-u64 seed retention and inert laboratory Open. Forty-eight aligned multiscale project regressions retain192 Native host matches and480 import refusals. Project security, current package190 files/657 dependencies and option-inventory checks pass. No Rust changes were needed; artifact identity remains d6db157e8243eb91cd3c5f4049967785807f8d259c2453fc4baeabb2c0cc427e.

This is a single prepared-source UI handoff. Native acquisition/discovery/absence fallbacks, multi-source orchestration, authenticated provenance, national fine capacity/publication and the full WASM objective remain unfinished. Changes are local and unpublished.


### Original character preparation sources in saved laboratory runs and exports

Laboratory character configurations may now retain a character_preparations map for selected states. Each entry is the original bounded preparation project, including raw source bytes, settings, prepared observations and result receipts. Direct practitioner handoff retains this entry. Explicit alignment preserves it; selecting a replacement character file removes that state's old preparation entry, Clear removes all entries, and Open restores the retained map. Older/manual character experiments without preparation entries remain valid. The map may cover a subset of states; it is not silently synthesized for missing sources.

Validation binds preparation metadata and original prepared observations to the active character input or its retained alignment source_data. Before engine execution, on graph-checked laboratory restore, and before practitioner export, asynchronous checks compare native request source bytes and CSV output bytes with their recorded hashes and source name. Plain text follows the existing preparer's decoding policy; gzip requests hash original encoded source bytes. The laboratory also checks these bindings before adopting a handoff. No data preparation or engine execution occurs during Open. Hash checks do not recompute the ACS/LODES transformation or authenticate a publisher.

Practitioner exports attach the retained preparation project as files.characterPreparation, alongside the original native plan/context; the native document bytes and engine request remain unchanged. Project validation checks the companion's kind/scope/observations against available plan producer input. The practitioner UI displays the retained source and allows a preparation-project download after explicit hash checks, without engine execution. General practitioner Open structurally validates this companion; download checks its hashes. Existing project byte/depth/value limits apply to the full retained source package, and oversized sources are not silently omitted.

Sixteen single/multi state/national experiments with Native Build on/off pass64 full Native host comparisons, inert restore and companion-source export. National fixtures retain a preparation for RI while other selected states use explicit character observations, exercising subset metadata. Four original/aligned binding checks and14 raw-source/CSV/hash/scope/nesting refusals pass; accessor input is rejected without invoking it. Actual Chrome covers both preparation Workers and direct handoffs, source preservation in laboratory Save and practitioner export, changed raw-source laboratory Open refusal, source-project download without execution, Native assignment equality, exact maximum-u64 seed and inert project Open. Forty-eight multiscale alignment regressions retain192 Native host matches and480 tamper refusals. Project security, package190 files/660 dependencies and option-inventory checks pass. No Rust changes were needed; the artifact remains d6db157e8243eb91cd3c5f4049967785807f8d259c2453fc4baeabb2c0cc427e.

Authenticated provenance, source discovery/acquisition/absence fallback, multi-source orchestration, national fine capacity/publication and full WASM parity remain unfinished. Changes are local and unpublished.


### Character preparation receipt scope and original encoding boundaries

Retained character preparation projects now require both usable character and native CSV outputs. Receipt state/Census year/observation year, written row count, source name and hash formats must agree with selected observations/source settings. Economic preparation must record workplace WAC; an economic character result cannot be relabeled as commuting OD. LODES encoding and source-byte length must match the request reconstructed from retained sources. These checks supplement the existing source/CSV hashes and alignment-source binding, without running preparation on Open.

Four synthetic format cases pass exact Native-host/WASM preparation and original-byte archive roundtrips: LODES and ACS JSON with UTF-8 BOM, gzip WAC and a gzip WAC source expanding beyond20MiB. Plain source decoding removes a BOM before supplying text to Native preparation; retained file bytes still include it, so their raw hash differs from the recorded Native request hash. Gzip uses the original compressed bytes and records a matching compressed-source hash. Twenty-nine top-level scope/row/CSV-header/type/encoding/length mutations are refused. Existing14 source/hash/scope/nesting refusals and the detached handoff checks pass.

Actual Chrome exercises the20MiB-expanded gzip workplace source and the housing BOM source through fresh preparation Workers, separate-tab handoff without automatic alignment/Run, one-use token refusal, explicit alignment and native-matching Run, exact maximum-u64 seed, laboratory Save/Open, practitioner companion export, original preparation-project download without execution and changed raw-source Open refusal. Ordinary retained-source16-project/64-host-comparison, original small-source browser, generic project security, package190 files/660 dependencies and option-inventory regressions pass. The Rust artifact is unchanged: d6db157e8243eb91cd3c5f4049967785807f8d259c2453fc4baeabb2c0cc427e.

The expanded source uses synthetic rows/padding; this verifies a format and size path, not authentic observations, Native publisher provenance, all-state acquisition or national block capacity. Transformation recomputation/authenticated provenance, discovery/acquisition/absence fallback, multi-source orchestration/publication and full WASM parity remain unfinished. Changes are local and unpublished.


### Explicit retained-source preparation replay in a verified WASM worker

The practitioner source companion panel now offers Recompute and compare preparation in WASM. Only this explicit action starts a dedicated preparation worker. It validates the bounded original project and source/CSV byte bindings, verifies the loaded WASM module against the main engine fingerprint, reconstructs the original Native preparation request, executes preparation and compares every returned output field with the saved result. No plan request is submitted. Sources and saved preparation remain unchanged. The new optional files.characterPreparationRecheck stores a compact matched/mismatch receipt with original project/source/expected CSV/recomputed CSV hashes, current WASM identity, sorted differing fields and a scope excluding publisher authentication and plan certification.

Open performs structural receipt validation and labels a saved receipt unverified on this open; it never starts replay. Fresh receipt download requires recomputation on this device and a hash binding to the current retained preparation. Cancellation, timeout, changed project revision and failed engine integrity preserve previous sources and receipt without marking them freshly verified. Clear removes both source companion and receipt. A dedicated cancellation button and the main Cancel action terminate the separate worker. Terminal cancellation/error messages survive autosave instead of being replaced by the old receipt status. The worker uses the existing25MiB project and60-second explicit-action bounds; Native source preparation retains its own source/expansion limits.

Four unmodified synthetic BOM/gzip/20MiB-expanded preparations match current WASM replay. Four internally consistent but incorrect saved character or CSV transforms pass the earlier byte-only checks and are now detected as mismatches by recomputation. All eight output comparisons agree with independent Native-host preparation;48 malformed/binding/scope receipt cases are refused and source/receipt project roundtrips pass. Actual Chrome confirms explicit large compressed-source match and incorrect-transform mismatch, exact API receipts, fresh receipt downloads, inert Open with saved receipts unverified, unchanged sources, busy controls, cancellation, stale-result discard, wrong-module rejection, Clear and zero plan requests. Generic project security,16 retained-source projects/64 full Native host comparisons, handoff, package192 files/671 dependencies and option-inventory regressions pass. The Rust artifact remains d6db157e8243eb91cd3c5f4049967785807f8d259c2453fc4baeabb2c0cc427e.

This provides explicit transformation recomputation for retained LODES workplace and ACS housing projects with current preparation code. It does not authenticate a data publisher or historical execution, verify a plan, prove optimality or complete Native acquisition/discovery/absence fallback, national fine capacity/orchestration/publication. Full WASM parity remains unfinished. Changes are local and unpublished.


### Explicit character-source absence policy with geographic fallback

Economic and housing weights now accept an optional character_absent_file_fallback:true request/configuration policy. Without a selected character input, the explicitly enabled policy uses the native missing-file geographic weighting path. The default still requires complete explicit observations. Supplied observations always take precedence, including in mixed national experiments; malformed supplied data is not silently treated as absent. The policy applies only to character modes and false/non-character uses are refused. Character blend settings stay recorded for source-present states but have no weighting effect in absent-source states. This is browser authorization for absent selected input, not proof that a file was absent on a historical filesystem.

Ordinary fallback evidence records native-character-absent-geographic-v1, kind, geographic baseline and source policy with null character identity and observation year. Multiscale uses absent-character-geographic-metis initialization and parent-only absence evidence; fine/coarse sampling and ranking stay unweighted. COI post-weighting still applies to the computed geographic seed graph. Fine costs and audit contexts remain physical. The laboratory exposes an explicit checkbox, restores it on Open and labels fallback results without implying character observations. Exports retain the option and omit character source hashes/observations; the Rust multiscale producer serializes absent optional character input as null.

Six cases exercise actual Native State argument conversion, native absent-file LODES/ACS loaders in an isolated working directory, native geographic weighter and Single kernel on synthetic AL and actual RI/IA parent graphs. One hundred sixty-eight ordinary and144 multiscale differential cases agree with geographic execution;28 algorithm feasibility failures occur identically in both paths and are recorded individually rather than counted as successful plans. These include existing prime-factor research balance limits, area-section infeasibility and capacity/regionalization NeedsRepair outcomes. Eighteen policy refusals and six source-honest ordinary exports pass. Multiscale BG children are synthetic population-preserving projections.

Forty-eight multiscale state/national projects have192 full Native host comparisons and384 import tamper refusals. Sixteen ordinary Single/Multi state/national projects have64 full Native host comparisons, export and inert Open. Native Build is tested on/off; national cases mix supplied AL observations with absent AK/RI sources. Actual Chrome verifies the visible/restored absence permission and absence wording, no fabricated source identity, physical fine costs, exact seed, source-honest export, source replacement and supplied housing data taking precedence while permission stays enabled. Existing complete-source and missing/invalid-source refusal regressions pass. Fresh proof-model/bounded proof-pipeline, project security, package and option-inventory checks pass. Artifact SHA-256: 3163800ebbcdcaef2f61eb39b6cf717a7e00cc4a6091060e977ecfe82d81e20b. The declaration inventory now includes235 fields and49 WASM options.

This does not implement automatic Native source discovery, error-file fallback or filesystem orchestration, authenticate observations, establish national fine capacity, publish the app or complete full WASM parity. Changes are local and unpublished.


### Shared native tract character CSV loading and browser inspection

Native LODES tract/housing CSV parsing is now shared from bisect-core::native_character_csv. Native CLI filesystem loaders retain their missing-file/read behavior and call the shared pure parsers after reading text. The extracted code retains first matching headers, last duplicate row, whitespace handling, simple comma splitting, housing clamps and long-name-before-alias precedence, native float parsing and error messages. No general quoted-CSV parser or BOM removal replaces these historical native semantics. Twenty-seven original filesystem-loader outputs were captured before extraction and retained as synthetic golden fixtures.

A new WASM prepare-native-character-source tool and practitioner operation inspect these exact readers against selected raw bytes, state/Census/observation metadata and character kind. The result reports native loaded/empty/error status, sorted observed vectors, finite markers, source hash/length and strict engine-input compatibility. Nonfinite numeric results are encoded as JSON null with finite=false. Native-loaded foreign/noncanonical IDs or unusable raw counts remain visible in the report instead of being relabeled as native parse failures. Compatible selected-state, bounded finite/raw-count data can be downloaded as the existing character input for explicit laboratory selection/alignment. Original raw CSV bytes and metadata survive project Save/Open; source parsing never runs on Open. Empty files may be inspected. Browser guards remain8MiB and100,000 unique records, with only the first200 rows rendered and the full report downloadable. This performs source inspection, not alignment, plan generation, file installation or certification.

All27 golden cases match current shared-parser WASM, Native-host tool output and the rebuilt native filesystem loaders, including native error strings with only path-label normalization. Five golden cases produce compatible strict character downloads; the rest retain their native outcome and disclose compatibility failures. An additional canonical housing fixture independently checks [-2,2,-1,2] clamps to [0,1,0,1] and remains downloadable. Source size/read-change and invalid UTF-8 cases are guarded. Actual Chrome exercises housing aliases, economic duplicate rows, native parse errors and empty files, exact reports/downloads, original-byte Save, inert Open, fresh-download gates and markup-looking IDs rendered only as text. The mock-import UI harness was extended with the new module. Project security,16 retained-source projects/64 full Native host comparisons, fresh proof-model/bounded proof-pipeline, package193 files/675 dependencies and option-inventory checks pass. Final Cargo build check found the current WASM up to date. Artifact SHA-256: fa23dac4272a0686caf7c4d19dd1805b1e7fe3e592b2f85d7e416c3545aa08bf.

Strict character download availability is an additional browser engine boundary, not a change to native file loading. This does not automatically discover native disk files, wire every legacy-loaded value into engine requests, perform State error-file fallback or authenticate publisher provenance. Native acquisition/orchestration, national fine capacity/publication and full WASM parity remain unfinished. Changes are local and unpublished.


### Native CSV observations through laboratory, original-source export and replay

Fresh compatible native tract CSV results now offer the same separate-tab laboratory handoff as LODES/ACS preparation. The preparation-project contract accepts prepare-native-character-source with exactly its retained nativeCharacterInputs. It checks loaded/ready status, kind/state/Census/observation scope, unique record coverage, finite record vectors agreeing with compatible character observations, selected source name, raw source hash and byte length. Incompatible native-loaded records, parse errors and empty-source results remain inspection/report outputs and cannot be transferred as engine observations. Original source bytes and native report remain in the preparation project.

The laboratory verifies and retains this project through explicit alignment, Run, Save/Open and practitioner companion export. Source replacement/Clear preserve the established invalidation behavior. The practitioner source companion can download the original native preparation project and explicitly recompute it with the existing verified WASM worker. For native CSV replay, the receipt's expected_csv_sha256/recomputed_csv_sha256 identify the original input CSV bytes, also recorded as source_sha256; LODES/ACS retain their derived CSV hash meaning. Full native report/character output comparison still detects incorrect saved transforms. No source replay, alignment or plan run occurs on Open or handoff. This contract extension does not add publisher authentication.

All five compatible cases in the original-loader fixture corpus pass retained-source project/replay/Native-host comparisons. Twenty-two unavailable handoffs and30 record/scope/byte mutations are refused. Two internally consistent native character/report vector alterations pass byte-only checks but are detected by explicit recomputation. Sixteen Single/Multi state/national projects with Native Build on/off pass64 full Native host comparisons, inert restore and original-source companion export. Matched synthetic native CSV fixtures include one observed tract matching the target graph, with explicit native defaults for the rest; housing alias/clamp and economic duplicate-row semantics survive the full path. Additional Native-host preparation checks verify these matched fixtures.

Actual Chrome verifies fresh native source Workers, separate tabs without opener, current-module integrity, source selection without alignment/Run, consumed-token refusal, explicit alignment and two Native-matching plan results, exact maximum-u64 seed, original-source laboratory Save/Open/export, changed raw-source Open refusal, source download without execution and matching explicit Native CSV rechecks. The test catalogue now serves the current WASM fingerprint; the application correctly rejected the earlier stale fixture fingerprint. Existing LODES/ACS handoff/replay/source-retention and project-security regressions pass. Package193 files/677 dependencies and option-inventory checks pass. No Rust changes were needed; the artifact remains fa23dac4272a0686caf7c4d19dd1805b1e7fe3e592b2f85d7e416c3545aa08bf.

This enables the strict-compatible subset of native-loaded observations. Other legacy-loaded values remain visible in inspection but are not silently coerced into this engine boundary. Automatic native discovery, complete legacy-value dispatch, State error-file fallback, authenticated provenance, national fine capacity/orchestration/publication and full WASM parity remain unfinished. All examples here use synthetic observations; no authentic data provenance or legal certification is claimed. Changes are local and unpublished.


### Retained native empty/error sources with explicit geographic fallback

Fresh native character CSV inspections with native_status empty or error now offer a separate laboratory transfer that explicitly selects geographic fallback. The failed or empty file is retained as character_fallback_sources, separate from observed character input and observed preparation projects. Loaded sources, including incompatible legacy-loaded values, cannot use this route. The retained preparation project records original bytes, source name/hash/length, scope, native status and diagnostic. Byte hashes bind the selected file; they do not authenticate the diagnostic or publisher. Open and transfer never rerun the parser or generate a plan. An explicit source recheck compares every recorded output with the current verified WASM loader, and can detect an altered diagnostic even when retained byte hashes agree.

The fallback contract requires explicit character_absent_file_fallback permission, matching state/Census year/kind and no overlapping observations or observed preparation for that state. Run and graph-checked Open verify retained raw bytes. Save/Open keeps the original empty/error source; practitioner export attaches characterFallbackSource while Native plan/context output continues to record geographic absence policy without fabricated character observations. The companion is bound to the plan jurisdiction and selected producer fallback. Source download and explicit recheck support both observation preparations and empty/error companions. A saved recheck remains unverified on a new Open.

The original native-loader corpus supplies11 eligible empty/error files;16 loaded-source cases are refused by this fallback route. Source/policy gates reject121 mutations and explicit replay detects11 altered saved diagnostics. Sixteen ordinary Single/Multi state/national projects with Native Build on/off pass64 full Native-host comparisons, inert restore, source companions and80 refusal cases. Forty-eight fixed/adaptive multiscale state/national projects across tract/county and synthetic block-group parent combinations pass192 full Native-host comparisons and432 refusal cases. National cases combine supplied observations in one state with retained failed-source fallback in another. Synthetic fine inputs remain capacity fixtures, not authentic Census block-group data.

Actual Chrome verifies both an economic CSV parse error and a housing empty file: fresh inspection Workers, separate laboratory tabs without opener, consumed-token refusal, no implicit alignment/Run, explicit checked fallback and Run, two Native-matching assignments, maximum-u64 seed, inert Save/Open, altered-source rejection, original companion export/download and matching explicit source rechecks. Existing ready-observation browser workflow and preparation-recheck browser safeguards pass, together with project security, original preparation retention,194-file/686-dependency package and235-field/49-option inventory checks. No Rust engine change or new WASM artifact was needed; artifact fa23dac4272a0686caf7c4d19dd1805b1e7fe3e592b2f85d7e416c3545aa08bf.

This delivers an explicit, file-backed browser counterpart to the native State loader's empty/error geographic behavior. Native automatic file discovery, complete legacy-loaded-value dispatch, whole State/States orchestration, authentic national fine capacity, authenticated provenance, publication and full WASM parity remain unfinished. Changes are local and unpublished.


### Batch preparation-project selection and mixed national alignment

The laboratory character picker accepts up to50 CSV/JSON inputs or saved character preparation projects together, with one selected file per state and a combined25MiB limit checked before reading. JSON continues through the existing bounded safe JSON worker. Saved compatible native CSV, LODES WAC and ACS housing preparation projects retain their complete original source companions. Recorded native empty/error projects require the geographic fallback checkbox to be selected before import. Kind and Census year must match the laboratory; each source retains its own observation year. Saved preparation selection verifies original byte hashes and report/input consistency but does not replay the saved transformation. Explicit practitioner source recheck remains available after export. No saved project can trigger preparation replay, alignment or plan generation merely by selection. Raw CSV import continues to use the explicit existing CSV conversion worker.

All selected files are read and checked before applying any state changes. A duplicate state, invalid project, scope conflict or altered source rejects the whole batch and keeps existing observations/preparation/fallback companions. Valid replacement retires both prior companion types for the affected state; plain JSON replacement retains no stale source companion. Clear or another selection invalidates an in-flight batch; changes to selected kind/Census year, CSV state, observation year or fallback permission also reject a stale batch. File size, source bytes, complex JSON and dangerous property protections continue to use the existing import boundaries.

Explicit alignment now aligns only observed states when geographic absence fallback is enabled, preserving source observations and native missing-tract defaults. States with no observations remain on geographic fallback; no alignment graph is fetched for them. Without fallback permission a missing observed state is still an error. A fallback-only selection reports that no observations need alignment. The resulting observed/fallback maps continue through existing state/national Run, Save/Open and source companion export contracts. Fallback observation-year display is restored from its retained report.

Synthetic tests cover100 original preparation projects, both character kinds and all50 states, with66 explicit observed-state alignments alongside34 fallback states and20 refusal cases. These exercise the50-file selection boundary, not authentic50-state data or national engine capacity. Actual Chrome verifies both kinds in three-state national batches, original preparation and fallback retention, explicit mixed alignment, four national runs with12 Native-host assignment matches, plus three direct JSON/CSV/stale-scope state runs with three more Native matches. Seven browser refusal cases cover fallback without permission, duplicate states, changed raw hashes and settings changed during a delayed read; failed batches preserve current sources. Delayed-read Clear, inert selection/alignment/Open, exact maximum-u64 seed and per-source observation years pass. The existing ready-source handoff browser and project-security regression pass. The staged package resolves195 files/691 dependencies; the inventory remains235 fields/49 WASM options. Current artifact fa23dac4272a0686caf7c4d19dd1805b1e7fe3e592b2f85d7e416c3545aa08bf; no Rust engine change.

This improves national source selection and observed/fallback orchestration. Native automatic disk/provider discovery, incompatible legacy-loaded-value engine dispatch, complete State/States workflow, authentic national fine capacity, authenticated provenance, publication and full parity remain unfinished. Changes are local and unpublished.


### Native source alignment and legacy-value edge-weight evaluation

Practitioner tools now expose Evaluate native character alignment and edge weights. Select an original native tract CSV and a target JSON containing state, year, canonical target geoids and positive canonical physical boundary edges; a prepared graph is projected to those fields. Choose geographic blend alpha and explicitly permit empty/error geographic fallback if desired. The target is a weighting input, not a validation of population, adjacency, geometry or graph connectivity. Source/target inputs and report survive inert Save/Open. Fresh evaluation is required for report downloads; edits/Clear invalidate freshness. Source and target Census scope must match. Limits retain8MiB original CSV,100,000 source/target records,one million target edges and the existing project/JSON limits.

The source inspection operation and weighting operation now share one scope/encoding/CSV loading function. Weighting uses native exact-ID alignment directly: legacy noncanonical/foreign/reserved source IDs may be ignored just as native State ignores unmatched source IDs; a nonempty loaded source still applies native zero economic or neutral0.5 housing defaults to missing targets. It does not turn into absent-file fallback merely because no source IDs match. Empty/error sources require explicit permission to retain geographic edges. The evaluator calls shared native character derivation, cosine and blend kernels without applying the stricter portable observation/raw-count restrictions. Every aligned vector and edge reports a numeric class: finite, negative-zero, nan, positive-infinity or negative-infinity. Nonfinite JSON values are null with explicit classes, not replacement observations. Ordered target and original source hashes identify evaluation inputs, not authentic observations or execution history.

The27 pre-extraction source cases plus eight additional matching-ID numeric sources pass175 native filesystem-loader/alignment/CLI-weighter comparisons and175 full Native-host/WASM report comparisons across alpha0,.25,.5,.75,1. Additional sources exercise negative and NaN economic counts, infinity-derived vectors, raw counts above the strict exact-integer range, matched housing NaN, infinities, signed zero and zero vectors.73 permission/scope/target/encoding/numeric refusal cases pass. Original source/target/report project roundtrips retain classifications. The original corpus labels alone were insufficient to prove matched numeric behavior: several malformed numeric fixtures also had noncanonical IDs. The additional fixtures ensure those values actually affect target edges. A matched housing NaN propagates to edge weights even at alpha1 under native floating-point arithmetic; no silent geometric replacement is applied.

Actual Chrome exercises six legacy/empty/error evaluations with the real Worker, exact matching reports, visible NaN, ignored-source behavior, raw source/target Save and inert Open, explicit fallback denial, alpha freshness and blank-alpha refusal, target projection/validation, pending-read Clear and zero plan requests. Source inspection now also retires fresh downloads and both observation/fallback handoffs on edits. Source Run and weighting Run validate current visible source settings, so an invalid edit cannot execute the last valid scope. Both source and target loading block evaluation while reads are pending; Clear and Open retire in-flight reads. Actual browser checks cover these source-edit/loading guards, including no old-source request after an invalid state edit. Original source-loader and strict ready-observation regressions,16 ordinary state/national projects with64 Native-host comparisons, project security,196-file/697-dependency package closure and unchanged235-field/49-option inventory checks pass. Scoped proof-model/pipeline regressions also pass on current artifact af60e97423ee6ae3eea70050b446398cc0d48d245b338f9da693ccc8c7784a06; these remain bounded model/pipeline checks, not global or legal certification.

This establishes native legacy-value alignment/weighting as an explicit browser boundary and a reference for subsequent engine integration. Partition dispatch, Save/Open metric validation and practitioner plan export for incompatible legacy-loaded values are not yet connected to this evaluator. Full State/States acquisition/discovery/orchestration, authentic national fine capacity, authenticated provenance, publication and full WASM parity remain incomplete. Changes are local and unpublished.


### Original native character CSVs in engine execution and physical plan export

The portable engine request now accepts optional native_character_source with the original selected CSV encoding, source name, scope and observation year. This field is mutually exclusive with explicit character observations and must match economic/housing character weights and the prepared graph scope. Ordinary execution, native population-build retries, multiscale initialization and both plan exporters use the same native file-reader/alignment/weighter evaluator. Loaded legacy values are not coerced into the stricter derived-observation input. Nonempty sources align exact IDs with native missing-vector defaults; loaded files with no matching IDs remain loaded/defaulted rather than absent-file fallback. Empty/error files require explicit character_absent_file_fallback permission.

Ordinary native-source runs add weighted_boundary_class to distinguish finite, signed-zero and nonfinite weighted measurements. A native NaN remains null with class nan; it is not replaced by geographic cost or zero. Ratio root weighted costs also carry numeric class. The underlying engine/search/structure behavior and applicable option validation remain native. Multiscale uses these source weights only for parent-tract initialization; fine/coarse proposals and ranking remain unweighted and no fine-unit character observations are inferred. Export contexts retain original physical graph boundaries. Export source hashes record original CSV and ordered weighting-target identities; the producer retains the selected original source input. These are unsigned byte/input identities, not source or execution authentication.

Practitioner tools expose Review / run engine with original native character CSV. Select the original CSV and exact UTF-8 request JSON: an ordinary graph/options request, run-engine-build, run-multiscale or run-multiscale-build. Block-group wrappers can additionally carry top-level fine_edges for physical export. All existing applicable options/composed inputs are preserved rather than reconstructed from a reduced form. The default review-only checkbox parses the typed request, checks prepared graph/base options and hashes selected CSV bytes without loading observations or partitioning. The full option table displays exact u64 seeds. Uncheck review-only to run; the packet retains engine output, the executed request and native RPLAN/RCTX bytes where export is possible. Native retries record the accepted seed in the executed request and export. Export errors preserve a generated result with export_error; for example, missing fine physical edges prevent RCTX export without inventing boundaries or discarding the multiscale assignments.

Request JSON remains opaque text in the host until explicit typed Rust review/run. Numeric u64 JSON values are not rounded by JavaScript; BOM/CRLF text is retained exactly, with ordinary native JSON validity checked on explicit review. Save/Open retains CSV bytes, original JSON and mode/metadata without execution, loading observations or enabling fresh artifacts. File edits, mode/metadata changes and Clear retire fresh reports/plan downloads. Pending source/request reads block execution of the previous selection; Clear and Open retire stale completions. Unsupported operations in request text cannot dispatch arbitrary tools, and JavaScript-looking text is never evaluated. Original request and accepted execution remain distinguishable.

Current evidence:108 selected native filesystem-loader/State Single-Multi kernel comparisons across both kinds, nine legacy/empty/error profiles, two searches and three blend values all match. Ordinary API/review/export gives144 full Native-host/WASM comparisons, including12 NaN weighted metrics,18 nonexecuting reviews and18 physical context exports. The120 synthetic fixed/adaptive/build multiscale cases across tract/county and block-group parent combinations give242 Native-host comparisons and120 physical exports; two missing-fine-edge cases retain identical generated results with explicit export errors.38 selected option/search/structure/composed-input choices across both kinds give76 Native-host comparisons:75 successes also match compatible strict-observation dispatch, while one economic PrimeFactor run preserves the native3% research-balance rejection. These are selected option cases, not every possible combination. Both source kinds also match a native geographic retry fixture accepted on attempt4; three Native-host comparisons prove accepted executed/export seeds and exact accepted-request replay. Combined comparison count is465, with27 source/request/encoding/export guard refusals recorded separately.

Actual Chrome verifies three real review/run workflows, exact matching Native fixtures, physical RPLAN/RCTX downloads, raw source/request Save and inert Open, visible maximum-u64 seed, numeric-u64 JSON preservation, nonexecuting opaque text and pending-read/freshness guards. Original27-case loader,175-case weighting, strict observation flow,16 ordinary state/national projects/64 Native-host matches, project security and scoped proof-model/pipeline regressions pass on artifact f993eafbcc5fc7d0ad3155ed0ba672b8e9d3860bfc8e479c8eec74ef58b6eb65. Package197 files/701 dependencies and235-field/49-option inventory checks pass. Proof-model checks remain bounded pipeline evidence, not global or legal certification.

This connects canonical tract/parent-tract native-source weighting to the portable engine and practitioner export. First-class laboratory/map adoption of the raw-source path, non-tract identifier/vintage coverage, large authentic fine/national capacity, complete native State/States acquisition/discovery/orchestration, source/execution authentication and publication remain unfinished. The advanced request UI preserves option access but does not prove all settings/capacities or complete CLI parity. Changes are local and unpublished; the full WASM objective remains active.


### Original native CSVs in state and national laboratory runs

The laboratory now supports retained original native economic/housing CSV sources. In the character-input controls, select Retain original native CSVs, choose the CSV state and observation year, then select the local CSV. This is a loading choice: existing strict-observation selections keep their type. Saved native-source projects and native source input JSON can also supply original bytes. Each state has one exclusive source path; replacing it retires strict observations, preparation or fallback companions for that state. National runs can mix strict observations, original native files and explicitly permitted absent-source states.

Selecting original CSVs, aligning strict observations, or opening a saved project does not parse the original CSV or run the engine. Original CSV parsing and exact native alignment happen on explicit Run. Empty/error sources require the existing explicit geographic-fallback permission. Native numerical behavior is preserved: NaN weighted measurements appear as null plus class nan and are visibly identified, rather than changed to zero or geographic cost. Multiscale source weights affect parent-tract initialization only; fine-map costs remain physical boundaries. Scope/source descriptions in the result panel distinguish this path from strict observations, absent-source fallback and partisan weighting.

Save/Open retains original source bytes, selected state/Census/observation year, options and exact seeds. Open checks source and ordered-target byte hashes, target/graph correspondence, recorded vector defaults, edge arithmetic and assignment metrics without replaying source transformation. The report includes exact Native target_json so its Native byte identity can be checked without reproducing Rust float serialization in JavaScript. Recorded transformation/output claims are not thereby authenticated. Practitioner exports retain the original source in their native producer. The toolkit offers hash-checked Download original CSV and Select original CSV for inspection; selecting for inspection does not execute it.

Evidence for this increment:16 ordinary state/national Single/Multi projects with native build on/off yield64 complete Native-host comparisons and104 mutation refusals, including8 NaN measurements.72 synthetic fixed/adaptive multiscale state/national/build projects across tract/county and block-group parent combinations yield288 complete Native-host comparisons and456 mutation refusals. Three more comparisons preserve Native failures for nonfinite COI adjustment and empty/error CSVs without fallback; failed projects retain sources and reopen inertly without fabricated assignments. Source selection adds16 refusals and producer download/inspection adds12 source/scope/hash refusals. Actual Chrome checks state maps, mixed-input national runs, a synthetic block-group map, five additional Native assignment comparisons, source/year/seed retention, changed-source rejection, inert Open, physical export and exact original CSV download/inspection selection. Synthetic fine fixtures are not Census capacity evidence.

Original175-case native source weighting comparisons, project security, proof-model/pipeline regressions and the235-field/49-option inventory pass on artifact 38515856a023082a0748aa4680fb28572afe1a01936e379a503f0e249a86714a. The staged local toolkit resolves199 files and712 local dependencies. Proof checks remain bounded pipeline evidence. These local changes have not been published.

The laboratory/map raw-source adoption listed as unfinished in the preceding historical entry is now implemented for canonical tract/parent-tract sources. Full native State/States acquisition/discovery/orchestration, other identifier/vintage coverage, authentic large fine/national capacity, authenticated provenance, publication and the full WASM parity audit remain unfinished.


### Original native CSV weighting for declared Census and custom unit namespaces

Native CSV weighting now accepts the unit namespaces already supported by prepared engine graphs: county, tract, block-group, block, precinct and imported. Target inputs carry optional unit_kind and county_ids; engine targets retain the graph declarations. The engine and native-source evaluator share the existing graph identifier validator. Census IDs retain their declared widths/state prefixes and supported Census years. Custom IDs retain bounded UTF-8 identity and require explicit county IDs; their unit year can be any explicit four-digit value. Source inspection accepts that four-digit unit year, while offering strict derived tract observations only when their existing contract permits it.

Alignment remains exact native string matching. No parent-prefix matching, tract-to-fine expansion, geographic imputation or aggregation is introduced. A loaded tract-only file against different fine/custom IDs remains loaded with native missing-vector defaults and ignored source IDs; it is not claimed as matched fine-unit observations or absent-file fallback. Empty/error fallback still requires explicit permission. Target hashes include declared unit kind and county identity. Original undeclared canonical tract target bytes remain valid, including previously saved declared-tract projects whose old reports omitted the type; an omitted type cannot authorize another namespace. Historical matched_tracts/missing_tract_ids JSON field names remain for compatibility and count selected target units. UI labels now say units.

Practitioner tools retain the selected graph's unit declarations when loading a weighting target. The source year field supports custom unit years, and target status shows the declared kind. Review, Run, native population-build retries and physical plan/context export preserve the declared IDs, county identity, original CSV bytes and seed. Save/Open remains inert; custom non-Census empty sources are not offered as Census-laboratory fallback handoffs. This does not add non-tract sources to the published state catalog or authenticate their claimed resolution.

The synthetic matrix covers six unit types, Census2000/2010/2020 and custom2018/2024, two character kinds, partial/no-match/NaN sources and alpha0/0.5/1. It produces288 weight evaluations,556 complete Native-host/WASM matches,288 independent native filesystem-loader/alignment/CLI-weighter comparisons,192 selected native State Single/Multi kernel comparisons,32 nonexecuting reviews,32 physical exports,12 native-build exports,32 source project roundtrips and13 matching refusals. The filesystem reference deliberately reads selected fixture files under its isolated2020 fixture directory; these comparisons establish selected-file reader/alignment/kernel behavior, not native vintage-based discovery or acquisition. Actual Chrome verifies block-group and two custom-unit/year workflows, exact Native output/RPLAN/RCTX, inert target selection and Open, namespace/county retention, original bytes, visible custom years and exact maximum-u64 seeds.

Regression evidence on artifact a0ff90721be25f366812a57e0dee25c2f1cbbc590639b1dd69ecd63f4b07b328 includes the27 original parser goldens,175 weighting cases, eight Census-unit and32 whole precinct/imported workflows,144 original-source engine comparisons,76 selected option/search/structure comparisons, native later-attempt retry/export checks,27 source/request guards,16 ordinary and72 synthetic multiscale laboratory projects, their state/national/fine-map browser workflows, project security and scoped proof-model/pipeline checks. Toolkit packaging resolves199 files and712 dependencies;235 engine-facing fields including49 WASM options match source. These are scoped checks, not all possible combinations or full parity certification.

Full native State/States acquisition/discovery/orchestration, authentic large fine/national capacity, other unsupported vintage/provider paths, first-class non-tract catalog exposure, authenticated provenance and publication remain unfinished. Changes are local and unpublished; the full WASM objective remains active.


### General prepared engine request review and execution

Practitioner tools now offer Review / run a prepared engine request. This general route accepts exact UTF-8 JSON for an ordinary graph/options request, run-engine-build, run-multiscale or run-multiscale-build. It preserves all fields supported by the existing typed engine request, including graph unit/county identity, applicable options, demographic/election/partisan/strict-character/native-CSV inputs, COI and electoral settings. It does not require a separately selected character CSV or reconstruct a reduced request from YAML. The existing CSV-specific route delegates to the same implementation and keeps its original source-attachment restrictions and result protocol.

The default review-only mode checks typed JSON and prepared graph/base options without partitioning or loading embedded CSV observations. It does not claim to validate every auxiliary or multiscale constraint. The report displays scope, declared unit kind, target count, exact seed/options and original request SHA-256. Uncheck review-only for explicit execution. Applicable dispatch and native population-build retry behavior are unchanged; the returned executed request records the accepted seed. Physical RPLAN/RCTX exports are available where supported. Export errors retain generated results; a missing block-group fine boundary list prevents physical export without inventing edges or discarding assignments.

The selected request remains opaque host text until typed Rust review/run. Numeric u64 literals are not parsed or rounded by JavaScript. Save/Open retains original JSON and mode/metadata without executing it or enabling fresh artifacts. Request/metadata edits invalidate reports and downloads. Pending reads block execution of the prior source, and Clear/Open retire stale reads. Unsupported operations and JavaScript-looking text cannot dispatch arbitrary tools. Imported result shapes remain recorded claims and do not authorize fresh downloads. Optional review defaults to true at the general Rust boundary; the legacy CSV operation retains its existing API default.

Evidence on artifact 79ed88af80d867b7eb08935ab8ed4a5a1fa19ef856fba11fc96b2d402a8daaa5: 287 ordinary combinations across seven weighting modes produce 297 full Native-host/WASM comparisons, including four matching native validation/research-balance failures, 283 physical exports and eight nonexecuting reviews. All 49 current engine option fields occur in successful carried requests; this tests selected applicable values, not every combination. Embedded original CSV/NaN results match the CSV-specific route. A native population-build fixture accepted on attempt 4 preserves accepted seed and exact accepted-request replay. A further 84 synthetic fixed/adaptive/build multiscale cases across seven weights and three fine/coarse pairings give 85 full Native matches and 84 exports, including one result-preserving missing-fine-edge case. Guard checks add seven successful Native comparisons for numeric-u64 execution and six declared unit namespaces, plus 12 matching refusals. Combined scoped comparison count is 389, with those 12 refusals recorded separately.

Actual Chrome verifies four ordinary/multiscale/build workflows, exact matching Native fixtures and physical file downloads, visible requested/accepted options, inert Open, source-only save/restore, lossless numeric maximum-u64 text, nonexecuting opaque text, incomplete imported result shapes and pending-read/Clear freshness. The legacy CSV runner passes 144 ordinary and 242 multiscale Native-host comparisons, its independent filesystem/State cases and three browser workflows after delegation. Project security and scoped proof-model/pipeline regressions pass. The local staged toolkit resolves 200 files and 715 dependencies; 235 engine-facing fields including 49 WASM options match source.

This supplies general full-request access in the browser, alongside the simpler laboratory and YAML workflows. It does not establish full CLI parity, authentic large fine/national capacity, native State/States acquisition/discovery/orchestration, unsupported external-tool behavior, authenticated source/execution provenance or publication. These local changes are unpublished and the full WASM objective remains active.


### Prepared engine results in maps, audit, improvement, analysis and accepted replay

Fresh general prepared engine runs now display their accepted assignments in the existing Plan map when matching WGS84 unit GeoJSON is selected. Exact unit IDs and state/year scope are checked; engine one-based assignments become native zero-based display labels. Ordinary, block-group and custom imported/precinct identities are retained. Unit polygons can be clicked or keyboard-selected to inspect assignments. Selecting display geometry does not rerun the engine. Opening a saved project retains geometry and results without drawing an imported result as freshly computed or enabling downstream handoffs.

The prepared request panel adds Audit this plan, Improve this plan, Analyze this plan and Prepare accepted replay. These select inputs for the existing tools and never run a downstream stage automatically. Physical-export handoffs preserve exact native RPLAN/RCTX text after checking retained text/object consistency, scope and accepted assignments. Existing profile, recorded audit time, check choices and additional analysis inputs remain selected. A changed plan/context retires prior certificate and lineage selections. Improvement settings are retained, or begin with the accepted engine tolerance; the form is synchronized with those settings. No legal profile, demographic observations or certificate result is invented.

The general Rust result now records replay_artifact_json and its SHA-256. This is native typed serialization of the accepted ordinary request or accepted multiscale input with its original fine boundary list. Replay uses the accepted seed as a plain engine/sampler replay, rather than rerunning population retries. The host verifies artifact byte hash and correspondence with the accepted request, then retains those exact replay bytes and original final assignments. An edit/Open during asynchronous hash checking retires the pending handoff. These are input/byte consistency checks, not source or historical execution authentication. The legacy CSV result protocol stays unchanged and does not allocate a discarded replay artifact.

A generated block-group result without supplied fine physical edges can still display on matching fine geometry and replay its accepted sampler inputs. Its physical export remains unavailable, so audit/improvement/analysis handoffs remain disabled and no context is fabricated. Older general result packets without a replay artifact require explicit execution with the current engine before preparing replay.

Evidence on artifact 264db3e984f866368f91b38e8522e43228f116bb95bd3f1409d1e14c1aefef83: 18 source/map projects cover seven weight modes, selected composed inputs, captured ILP results, later accepted population retries, all six declared unit kinds, custom years/Unicode IDs and an embedded native CSV NaN measurement. They give 99 full Native-host/WASM workflow comparisons: 19 accepted replays, 18 audits, 18 certificate verifications, 18 improvements and 18 analysis packages, plus eight prepared-run setup comparisons. There are 180 source/scope/artifact/geometry refusals. Source-only and result projects roundtrip inertly; exact export text, user profile/time/check choices and retirement of stale certificates are checked. Synthetic geometry is not authentic geographic capacity evidence, and verification of these supplied synthetic profiles does not establish legal compliance.

Actual Chrome contributes 13 complete Native-host result comparisons and six handoffs without execution, covering ordinary, block-group and imported 2018/Unicode maps, clickable assignments, explicit audit/improvement/analysis/replay, exact source Save/Open, missing-export map/replay, wrong-year geometry rejection and an edit during replay hashing. Measured worker identity matches the current artifact. Map previews are retained under target/prepared-engine-flow-browser. Existing 297 general ordinary, 85 multiscale, source/request guard, 144 legacy CSV ordinary and 242 legacy multiscale comparisons pass, with general/legacy browser, project security and scoped proof-model/pipeline regressions. Local packaging resolves 201 files and 719 dependencies; 235 engine-facing fields including 49 WASM options match source.

This connects general prepared engine execution to existing practitioner workflows and maps. Full native acquisition/discovery/State/States orchestration, authentic large fine/national capacity, unsupported platform tools, authenticated provenance, publication and requirement-by-requirement full parity remain unfinished. Changes are local and unpublished; the full WASM objective remains active.


### Exact prepared engine requests in state batches and supplied-state national maps

The existing browser State batch queue now accepts general prepared engine requests alongside configured TUI runs and explicit Census-source jobs. After fresh typed Review or Run, Add reviewed request to state batch retains the exact original JSON, export metadata and optional display geometry. It selects matching queued states for that Census year; Execute remains a separate action. A blank export timestamp is recorded on the explicit queue action. Each prepared job keeps its declared ordinary/build/multiscale mode and all applicable request fields; queuing does not silently introduce population retries or configured follow-up stages. Native congressional allocation and existing completion filtering still select the batch.

Prepared jobs carry explicit state/year/district/chamber scope and the reviewed request byte identity. Their original JSON stays opaque in host selection and project Open. On explicit execution, the owning worker first performs native typed review and checks actual state, year, district count and request hash before partitioning. The subsequent Run uses the original request text. Scope/hash/malformed-request failures do not execute the partition request or stop other states. Queue construction snapshots its inputs before asynchronous hashing; edits/Open retire a pending UI queue handoff. State batches retain the existing congressional/Census 2000/2010/2020 allocation scope; custom years and other chambers remain available through the individual prepared runner.

Completed prepared states retain source review, original request, generated output, native physical exports and accepted replay artifact. Matching geometry produces an interactive per-state map. A display geometry error is recorded without discarding generation. Missing fine physical edges can leave a multiscale result/map available while physical export remains unavailable. Worker slots, input snapshotting, ordered result collection, independent per-state seeds, failure continuation, received-output retention on cancellation and same-session Resume reuse the existing coordinator. Imported checkpoints cannot activate map/download/resume authority. Verify saved completed states explicitly reexecutes and compares full outputs before reuse.

Batch ZIP downloads include a prepared-run.zip for each prepared state, with exact request.json, accepted-replay.json, final assignments, native export text where available, display geometry and a run receipt. Configured jobs keep their native-run.zip. Missing physical export does not create plan/context files. Use completed prepared maps nationally selects completed states with both physical export and matching geometry for the existing supplied-state national renderer. Selection itself does not render; unavailable/failed/nonprepared states are reported rather than treated as mapped states. The national view is of supplied states and does not establish complete nationwide input coverage.

Evidence uses three synthetic RI/IA/VT prepared jobs with numeric maximum-u64 JSON, captured rounds, N-way volume/trial/imbalance options and ordinary/native-build modes. Sixteen full Native tool comparisons include initial/per-state typed review and execution, multiscale parent data/fine boundaries, a mixed configured job and a stateful native national-map reference sequence. Serial/parallel state outputs agree; worker caps and cleanup, partial failures, bad geometry retention, absent physical exports, input snapshots, cancel/resume and explicit saved verification pass. Fifteen refusals cover allocation, scope, hash, malformed/opaque requests and altered saved outputs. Independent Python validates nine nested prepared archives for CRC/path safety and exact original/native/replay/geometry bytes. This is selected synthetic orchestration evidence, not authentic state capacity or every option combination.

Actual Chrome verifies review/queue without partitioning, edit-during-hash retirement, three independent prepared workers, exact request text/seeds, interactive state map, ZIP download, national-source handoff without rendering, explicit national rendering matching the native sequence, inert saved Open, explicit full saved verification and real-worker cancellation/Resume without rerunning completed states. Measured worker identity matches artifact 264db3e984f866368f91b38e8522e43228f116bb95bd3f1409d1e14c1aefef83. Existing 54 complete configured/native batch comparisons including 50 synthetic states, Resume/recovery/source-batch tests and the original batch browser pass. Project security, 203-file/735-dependency package and 235-field/49-option inventory checks pass. No Rust/WASM engine behavior changed in this increment.

This adds full prepared-request access to the existing queue and map/download lifecycle. Complete native acquisition/discovery/State/States orchestration, authentic large fine/national capacity, other unsupported workflows/platform tools, authenticated provenance, publication and requirement-by-requirement full parity remain unfinished. Changes are local and unpublished; the full WASM objective remains active.


### Full engine request templates through Census acquisition and source jobs

An explicit advanced source mode now accepts a full engine request template in the browser. Supplied Census sources, paired acquisition, batch acquisition and disk streaming can carry that template through native graph preparation and into the existing prepared runner without a YAML dependency. Legacy configured source workflows retain their existing controls. Selection, queueing and project Open preserve opaque request text and do not bind, load engine CSVs or partition. Explicit execution performs preparation, binding, typed scope review and then the declared ordinary/build/multiscale run.

Templates must declare graph:null and options.districts:null or a count matching the selected native allocation. Rust binds only those slots, rejects existing graphs and conflicting counts, and retains other raw JSON value tokens, including maximum-u64 numeric seeds. Native typed review rejects unknown request fields; duplicate processed template fields reject. Original template, its hash, bound request, binding receipt, accepted replay and native exports remain available in saved outputs. Invalid binding retains completed source preparation. Auxiliary inputs and multiscale fine data stay explicit; parent Census display geometry is never presented as fine-unit geometry when they differ.

Acquisition retains exact original ZIPs and the template recipe. Streamed runs use a fresh owned directory and verify file close/readback/hash before writing a completion marker. Reopenable source workbenches retain the template mode without executing it. The advanced UI exposes template selection, export metadata and explicit display-geometry selection; pending reads and Clear/Open retire stale selections. Congressional allocation and Census 2000/2010/2020 batch scope remain the existing limits.

Evidence: 289 native template bindings and 592 full Native-host comparisons cover all 49 current engine option fields in successful bound executions, exact u64 tokens, ordinary/build/multiscale modes and 12 binder refusals. Twenty-four source cases cover four Census resolutions, three vintages and ordinary/build modes; two additional explicit multiscale cases retain generation when parent display geometry cannot render fine units. These provide 91 full Native binding/review/run comparisons and six source guard refusals. Controlled ZIP-response acquisition/streaming adds six comparisons, original-source retention, worker cleanup, verified disk output and reopenable recipes. Independent Python validates three nested output archives and disk completion hashes. Actual Chrome verifies inert selection/queue/Open, source preparation through Run matching the fixture, visible exact seed, state map, ZIP download and stale-read controls.

Legacy source acquisition, 50-state synthetic streaming, source batches, prepared batches, project security and actual acquisition/batch browser regressions pass. Current proof gates include 32 native OPB/request byte comparisons and the native proof pipeline. The staged package resolves 204 files and 748 local dependencies; the source inventory still covers 235 engine-facing fields and 49 WASM option fields. Measured WASM SHA-256: e9f9950c34fc677d75bdceb54dd80abc1de1118f6e8d18721516ef6a8a702fa2.

These are selected fixtures and controlled provider responses, not evidence of live provider availability or authentic large-state/national capacity. Complete native source discovery/orchestration, unsupported platform workflows, authenticated provenance, publication and the full capability-by-capability parity audit remain unfinished. This increment is local and unpublished; the overall WASM objective remains active.


### Original-state prepared runs, replay and practitioner boundaries

Original retained Rhode Island Census 2020 inputs now exercise the full request-template source path at county (5 units), block-group (792 units) and block (25,649 units) resolution. Each preparation retains population 1,097,379, exact source identities, the original template and maximum-u64 numeric seed token. Explicit two-district runs use native build semantics. County tests explicitly allow 50 percent deviation for indivisible coarse units; block-group and block tests use 0.5 percent. These are selected configurations: standard bisection/geographic weights for county/block and direct N-way volume/trial/imbalance settings for block groups. The block-group build wraps the requested maximum-u64 seed and accepts seed 4.

Real graph size exposed three host/format mismatches. Accepted replay and prepared context inspection used a 500,000-value JSON budget, below the existing four-million-value workbench boundary. Those paths now use the workbench budget while retaining byte caps, depth, duplicate-key and unsafe-key rejection. Repeated-group base64 regular expressions could exhaust the JavaScript regexp stack on valid multi-megabyte inputs. Replay, assignment verification, package verification and ILP audit readers now share the existing linear-character/padding validation approach, retaining canonical decode/reencode checks where required.

Prepared ordinary and multiscale exports now use the shared native context writer that preserves pretty bytes through 8 MiB and compacts larger contexts. Export itself does not discard a larger compact context; individual consumers still enforce their own limits. The original block context occupies 16,002,276 bytes when pretty-printed and 8,098,582 bytes compactly, allowing the existing 8 MiB practitioner boundary. Native and WASM exports match exactly. Context contents/hash and plan bytes are unchanged by formatting. No partition/search behavior changed.

The fresh source-to-run test makes 12 full Native binding/review/run/replay comparisons on the actual WASM-prepared graphs. Independent checks recompute population balance and graph contiguity from final assignments, check every map assignment, and verify accepted replay and source-preserving inert Save/Open. Native comparison uses those actual prepared graphs; this does not assert bit-identical cross-platform Census projection or equivalence to every native CLI acquisition/adjacency profile. Independent Python validates three real prepared ZIPs for exact template/u64/replay/native export bytes, geometry hashes, CRCs and paths.

Nine additional Native comparisons re-export captured original plans and audit/verify them under an explicitly test-only population/contiguity profile. These do not certify jurisdiction rules, historical execution, legal compliance or publisher authenticity. Actual Chrome runs original block-group and block inputs through source-only queueing, real worker preparation/binding/build, exact Native-compared output, all-unit interactive maps and source-preserving archives. Imported completed archives remain inert, with map/download authority disabled until fresh execution. No provider network calls are used by these retained-source tests.

Current-artifact regression evidence includes 289 bindings/592 Native comparisons across all 49 successfully exercised option fields, 24 Census source cases plus two explicit multiscale cases/91 Native comparisons, 84 multiscale cases/85 Native matches and the proof model/pipeline gates. JSON/base64 safety, project lifecycle, assignment/package/ILP reader regressions, 204-file/748-dependency staging and the 235-field/49-option inventory pass. Measured WASM SHA-256: 8c77223abb6aaa8e76db4261e29291062acd1c3daa272006e154a930f3cc7e66.

This establishes selected original-state capacity on the tested runtime, not an average-browser performance profile or all-50-state/fine/national capacity. Full native discovery/State/States orchestration, unsupported platform workflows, authenticated provenance, publication and requirement-by-requirement complete parity remain unfinished. Changes remain local and unpublished; the overall WASM goal remains active.


### Native adjacency graphs to full prepared engine requests

The native adjacency preparation screen now offers Use graph with full engine request alongside the existing configured-YAML handoff. The new action retains the freshly validated graph and selected template/export metadata, clears previous display geometry and opens a separate graph-binding operation. It does not execute binding or partitioning. Original adjacency, GEOID, centroid and physical-geometry companions remain in the project. Matching display polygons remain a separate explicit input; adjacency does not manufacture a map.

The standalone Bind graph to full engine request screen accepts opaque UTF-8 graph and template JSON, explicit district count and export metadata. Run calls the existing Rust binder, requiring graph:null and options.districts:null or an explicit matching count. Other request values and tokens remain unchanged. Typed native review follows binding without partitioning or CSV observation loading. The result exposes selected scope, graph/template/bound hashes, run mode and original option values including the exact seed. Use bound request for engine review checks those source hashes and native review scope, selects the exact returned request and forces review mode. Choosing execution and Run remains an explicit later action. Ordinary/build/multiscale modes and auxiliary inputs stay in the original template.

Project Open retains graph/template text without interpreting it as code or preparing/running it. Imported native preparation or binding results cannot enable their handoff buttons. Pending reads block binding execution; Clear/Open retire stale reads. Source changes retire fresh bindings; revision/result checks reject edits during asynchronous source hashing before any handoff commit. Binding checks hashes and consistency, not source publication or historical execution authenticity. Manual graphs can retain the engine's explicit precinct/imported unit kinds and four-digit custom years; native binary preparation retains its existing Census unit/vintage scope. District count is a selected laboratory parameter; this screen does not implement automatic legislative allocation or full native State orchestration.

Evidence includes 18 cases and 60 full Native preparation/binding/review/execution comparisons: binary versions 2/3, four Census resolutions, ordinary/build modes, two explicit multiscale modes, manual precinct/imported graphs and the retained original 250-tract Rhode Island graph. Maximum-u64 numeric template seeds, direct N-way cut/volume/trial/imbalance controls, export chamber metadata, source-only handoffs, original native bytes and inert ordinary/workbench Save/Open pass. Seventeen refusals cover input bounds, changed graph/template/bound hashes, mismatched scope/review metadata and opaque program text. This is selected integration coverage; the existing generic binder evidence covers all 49 native option fields.

Actual Chrome covers both four-unit fixture and original 250-tract flows: inert imported preparation, explicit native preparation, handoff without binding, explicit binding without partitioning, exact seed display, handoff into review mode, explicit review and subsequent execution matching the Native-compared full output, exact native RPLAN download, source preservation, stale geometry clearing, inert Open and hash/read/Clear guards. The existing native-to-configured-YAML browser and prepared Census browser regressions pass. Project security, the 205-file/752-dependency staged package and 235-field/49-option inventory pass. This increment changes host UI, project state and handoffs; measured WASM SHA-256 remains 8c77223abb6aaa8e76db4261e29291062acd1c3daa272006e154a930f3cc7e66.

The new screen completes this source-preserving route to the general prepared runner. Automatic raw-native source jobs in state batches, broader native discovery/State/States orchestration, all-state fine/national capacity, other platform workflows, authenticated provenance, publication and full capability-by-capability parity remain unfinished. Changes remain local and unpublished; the overall WASM objective stays active.


### Original native adjacency sources in state batches

State batches now accept original native adjacency sources plus a full engine request template. Queue original native inputs for state batch snapshots the selected binary, GEOID/centroid/physical-geometry companions, state/year/unit-kind settings, exact template and recorded congressional export metadata. It uses the original selected native inputs rather than the editable graph JSON in the binding screen. Queueing does not prepare a graph, bind a request or partition. The queue selects matching-year queued states; native congressional selection supplies district counts on Run. Explicit template counts must agree; no declared retry mode is silently upgraded. Optional selected display geometry stays a separate source.

Each owning worker prepares the original native bytes, checks actual source scope, binds graph:null and the selected count inside Rust, checks graph identity/units/kind, performs native request scope/hash review and then executes the declared ordinary/build/multiscale mode. The existing general prepared output retains exact original template, binding receipt, accepted request/replay, plan/context exports and original native recipe. Preparation survives later binding/run refusal. Adjacency supplies no polygon geometry: absent display input stays absent, while bad supplied geometry can leave generation available with an explicit map error.

Native-source jobs reuse ordered state selection, worker caps/cleanup, snapshot isolation, independent failures, mixed prepared/configured queues and the existing cancellation/Resume protocol. Imported completed states require explicit full reexecution before reuse. Their output ZIPs include exact original adjacency/companions under fixed safe paths, a recipe preserving original names/settings, the template/bound request/replay/native exports and preparation graph/receipt. Missing source polygon geometry does not create a prepared-geometry.json containing undefined or a manufactured map. Completed eligible maps can be selected for the existing supplied-state national renderer; handoff does not render automatically.

Evidence covers 12 ordinary/build source cases across native binary versions 2/3, four Census resolutions and retained original Rhode Island 250-tract inputs. Three synthetic RI/IA/VT native-source states exercise serial/parallel execution, saved verification, cancel/Resume without redoing completed states, input snapshots, altered saved source refusal, per-state failure continuation, optional matching maps and national-source handoff. A mixed native-source/prepared batch passes. There are 137 full Native tool comparisons and ten refusal checks. Independent Python validates six nested archives from API/browser outputs: exact original bytes match native preparation hashes, template/u64 and native export/replay bytes are unchanged, CRC/path checks pass and missing polygon sources remain absent.

Actual Chrome verifies source-only queueing without YAML, ignored editable graph text never executing, actual original-binary preparation/full-template execution matching the Native-compared fixture, native two-seat allocation independent of the binding screen's manual count, exact visible seed, state map/ZIP, national handoff without rendering, inert imported Open and explicit saved reexecution. Existing prepared/Census/legacy source batches, project security and graph/Census browser regressions pass. The staged package resolves 206 files and 760 local dependencies; the inventory remains 235 engine-facing fields and 49 options. No Rust/WASM engine behavior changed in this increment. Measured WASM SHA-256: 8c77223abb6aaa8e76db4261e29291062acd1c3daa272006e154a930f3cc7e66.

This is selected original-input and synthetic orchestration evidence, not authentic all-state fine/national capacity or full native filesystem/cache discovery. Legislative batch allocation, broader native discovery/State/States orchestration, other unsupported platform workflows, authenticated provenance, publication and complete capability-by-capability parity remain unfinished. Changes remain local and unpublished; the full WASM objective stays active.


### Native single-state chamber counts and graph-binding handoff

Native State House/Senate count resolution is now a shared pure kernel in bisect-analysis::location_policy::state_chamber_count. The CLI still preserves its congressional early return, policy loader/environment behavior, exact case-sensitive state-key lookup, chamber aliases, missing/non-u64 count fallback, zero-as-absent semantics, diagnostic text and process exit. The extracted kernel returns a count/source or error; CLI presentation remains in its adapter. The existing native bulk States command remains congressional-only. This increment exposes the existing single-state rule rather than inventing broader native bulk semantics.

The browser Resolve native chamber count operation explicitly reads the embedded or selected policy and congressional fallback manifest, with bounded sources and 1–500 resolved districts. It reports allocation source and source identities without partitioning or changing a graph. Selected malformed overrides fail at the explicit portable source boundary; filesystem/environment discovery is not performed. Reference counts retain the native static-policy behavior across selected Census vintages and are not verified current law.

Use count and chamber for matching graph binding requires a fresh result and checks the selected graph's state/year using bounded read-only JSON inspection. It updates only the binding count and export chamber, preserving original graph/template bytes, tolerance and other engine settings. The original raw template is still passed to Rust on a separate binding Run, which rejects conflicting explicit counts. The resulting engine handoff remains review-only until the user explicitly chooses execution. Project Open does not resolve counts or bind/partition. Pending source reads, edits and Clear retire fresh handoff authority; source BOM bytes are preserved so malformed JSON is reported rather than silently rewritten.

Evidence includes all 50 eligible native states across Census 2000/2010/2020 and congressional/House/Senate. There are 458 actual CLI reference cases, 457 full Native/WASM result comparisons, three expected absent-chamber errors, eight handoff/source refusals, selected-policy overrides and missing/negative/fractional/string count fallback cases. The retained original Rhode Island 250-tract graph binds to the native 75-seat House count without partitioning; source graph/template/tolerance remain unchanged. This verifies count lookup and binding, not a generated or legally valid 75-district plan. Eight preexisting native CLI chamber-count unit tests also pass.

Actual Chrome verifies inert imported counts, explicit RI House75 lookup, nonexecuting count/chamber handoff, exact Native-compared binding result, review-only engine selection, unchanged template, inert reopen, Nebraska's absent Senate error and pending-read/Clear controls. Current-artifact regressions include all 49 successful option fields/592 Native binder comparisons,137 Native-source batch comparisons, actual native-source browser,32 proof-model byte comparisons and the proof pipeline. Project security, 207-file/766-dependency staging and the 235-field/49-option inventory pass. Measured WASM SHA-256: 638b8807d877564a69ce56d00cc5a736fa25cd7b34198f805997e0e41deccb48.

Counts are bounded source claims, not policy authenticity, historical/current legal verification or proof of plan feasibility. Complete native discovery/orchestration, authentic all-state fine/national capacity, remaining unsupported platform workflows, authenticated provenance, publication and capability-by-capability full parity remain unfinished. Changes remain local and unpublished; the overall WASM goal stays active.


### Native canonical input-folder discovery and source staging

The native State resolver and WASM now share bisect-core::native_input_paths::select_adjacency_path. The kernel preserves requested-store ordering (V3 then V4), block_group/block-group alias behavior and tract fallback only after both requested stores are absent. CLI filesystem/manifest loading, warnings and missing-file diagnostics remain in its adapter. The browser supplies bounded portable relative-path existence observations; this does not claim full filesystem, case-insensitive lookup, empty-directory, permission or symlink equivalence.

Inspect native input folder accepts an explicitly selected output root containing V3/V4, Census year, state filters, resolution and optional manifest. File selection retains only observations and File references in the active page. Run performs native path inspection without reading graph/source/pickle contents. The selected .pkl path remains authoritative: its .adj.bin sibling is browser-loadable, while a selected pickle-only store is reported as requiring the external native shim. The adapter never switches to a later store merely because it has a binary. Native pickle code is not executed in the browser.

Queue inspected original binary sources verifies fresh path/filter/manifest identity and selected file references, preflights bounds, then reads only referenced binary/GEOID/centroid inputs. Optional browser sibling artifacts are explicit conventions: stem.request.json overrides the shared full template, stem_geometry.json supplies physical geometry and stem_display.geojson supplies display polygons. These optional conventions are host additions, not native canonical resolver rules. Original relative source names and bytes remain in the existing native-source jobs. The selected file list is not restored as file-access authority on Open; folder reselection and fresh inspection are required.

Source reads stage privately. Failure, stale observations, changed filters, cancellation or an oversized batch leave the existing queue unchanged. The final job-source validation applies the existing 64 MiB bound; no arbitrary empty or partial source package is committed. Existing completion-selection settings remain available when the staged queue is adopted. Graph preparation, template binding and partitioning occur only on a later explicit state-batch Run. Imported queued jobs retain their original embedded bytes for that explicit execution.

Evidence covers all sixteen presence combinations of requested/fallback V3/V4 paths, across Census 2000/2010/2020 and tract/block_group/block-group selection: 144 actual CLI filesystem-reference cases and 171 full Native/WASM comparisons, including the three-state source executions. A selected V3 pickle without its binary stays pickle-only even when V4 has a binary. Seven refusal checks cover stale observations/filters, unsafe paths, cancellation, partial read failure and pre-read size rejection. Metadata selection/inspection reads no source contents; pickle markers and unrelated files are never read. Three synthetic native state sources retain original bytes, optional display geometry and a per-state build override.

Actual Chrome verifies folder selection and inspection without source reads, referenced-byte queueing without graph preparation/partitioning, three independent worker outputs matching the Native-compared batch, retained per-state build mode and inert Open requiring folder reselection/reinspection. Regressions cover all 49 successful option fields/592 Native binder matches, 137 Native-source matches, actual native-source/chamber browsers, the 32 proof-model byte comparisons and the proof pipeline. Project security, 208-file/776-dependency staging and the 235-field/49-option inventory pass. Measured WASM SHA-256: 360862e4a87742ecf79ee6cfc0c14a94235fa21fae121740572f1b0428d84822.

This adds native path selection and binary-source staging from explicit local folder observations. Legacy pickle shim execution, unrestricted filesystem/cache discovery, source acquisition, all-state original fine/national capacity, remaining platform workflows, authenticated provenance, publication and capability-by-capability complete parity remain unfinished. Changes remain local and unpublished; the full WASM objective stays active.


### Commission report preview and browser printing

After an explicit commission report Run with HTML selected, choose **Preview report**, then **Print / Save as PDF** in the results pane. Printing opens the browser print dialog for the report frame. The original native HTML remains available as an unchanged download. JSON-only delivery does not enable preview or printing. Open never renders retained HTML or grants print authority; rerun the report first. Edits, source clearing, Close, a new run and project reopening retire the preview, including a pending frame load.

The preview is a separate restricted copy of native report markup. It retains text, native CSS, tables and embedded PNG images, while omitting navigation and active elements/attributes. A detached template is traversed into new allowlisted elements; original nodes are never inserted. The separate static preview page must supply the expected restrictive CSP before content is inserted. The sandbox permits parent-driven printing and document access but does not permit scripts. Network resources, forms, frames and objects are disabled. Preview limits are 25 MiB, 250,000 traversed nodes and 64 nesting levels. Invalid embedded PNGs are reported visibly. This protects preview rendering; downloaded native HTML remains the original artifact.

`test_commission_preview_browser.mjs` runs an actual Chrome worker report and compares its HTML download with the native host result byte for byte. Its synthetic report fixture has all ten sections and a valid one-pixel embedded PNG; this is not evidence of full nationwide report capacity. It checks inert Open, explicit preview/print, edit/Close/Clear (including a pending load), JSON-only output, absent restricted preview page refusal, size/depth bounds, and malicious markup attempting script execution, navigation and network access. The test instruments the frame's `print()` entry point to avoid opening an OS dialog; it does not compare PDF bytes or validate a desktop external renderer. Browser pagination and PDF settings remain user controlled.

Evidence: `target/commission-preview-browser/evidence.json`, `target/commission-preview-browser/preview.png`, `target/commission-preview-browser.log`, `target/commission-preview-project.log`, and `target/commission-preview-assets.log`. The unchanged WASM SHA-256 is `360862e4a87742ecf79ee6cfc0c14a94235fa21fae121740572f1b0428d84822`. The staged toolkit contains 210 files with 777 resolved local dependencies. Local and unpublished; broad platform parity remains unfinished.

Browser reference: [MDN template content](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/template) and [MDN iframe sandbox](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/iframe). A separately served preview document avoids inheriting the workbench policy that excludes data images and inline report CSS.


### Preview generated reports across practitioner workflows

Fresh HTML reports are selectable under **Generated HTML report** in the results pane for standalone commission delivery, accepted State workflows, state Run batches, single-year label reports, multi-year label reports, and draft comparisons. Select a file, choose **Preview report**, then **Print / Save as PDF**. Only successful available report files are listed; partial workflows retain successful HTML, while JSON-only output and failed report stages do not create previews. State batch choices include the state code; label report paths retain the label/year/state. Selecting a report does not execute the engine.

Packaged report text or decoded Base64 bytes must match the recorded native file SHA-256 before a frame is created. UTF-8 decoding is strict and Base64 must be canonical. Legacy standalone native commission results without a delivery hash remain supported only with live Run authority. Discovery is restricted to known native result schemas, with portable file paths, at most 512 HTML reports and 25 MiB per selected report. Original report files/downloads are retained exactly. A digest is an integrity check, not source authentication or certification.

The isolated preview also retains native comparison `main` and definition-list structure, table accessibility attributes, draft badges and civic-source notices. Opening saved results remains inert. Busy state, a new result, selection changes, edits, clearing and Close retire preview authority; delayed hash/image/frame completion cannot revive a retired preview. The selected report is a transient view and is not trusted from imported project state.

Evidence is synthetic and scoped: `test_generated_report_files.mjs` compares ten complete native-host/WASM results, checks eight exact HTML sources/hashes and eleven path/hash/encoding/size refusals. `test_generated_report_preview_browser.mjs` executes five actual Chrome workflows, checks six native results and six report previews/instrumented print calls, compares all native report text after removing style/title nodes and normalizing whitespace, and checks draft tables/notices, inert Open/Save/reopen, no extra engine execution, no remote requests and rejection of altered HTML before frame creation. The project UI test additionally delays hashing and verifies selection/Close retirement before frame creation. Standalone preview security tests and ten full native State workflow cases plus the existing browser workflow regression pass. PDF bytes remain untested; no desktop external-renderer equality or original nationwide report capacity is claimed.

Receipts: `target/generated-report-files/evidence.json`, `target/generated-report-preview-browser/evidence.json`, `target/generated-report-preview-browser/comparison.png`, `target/generated-report-preview-project.log`, `target/generated-report-workflow-native-regression.log`, `target/generated-report-workflow-regression.log`, `target/generated-report-preview-commission.log`, and `target/generated-report-preview-assets.log`. The toolkit stages 211 files and resolves 780 local dependencies. The WASM SHA-256 remains `360862e4a87742ecf79ee6cfc0c14a94235fa21fae121740572f1b0428d84822`. Local and unpublished; full capability parity remains in progress.


### Native run-folder export and assignment fallback

Choose **Export existing native run folder**, enter the native filename label, select the plan folder and run explicitly. The browser now shares the CLI export loader: `{label}.rplan` takes priority, then root `final_assignments.json`, then `data/final_assignments.json`. A malformed preferred source fails instead of falling through. The labeled RPLAN uses the native legacy schema; assignment fallback uses `manifest.json` metadata when present and otherwise preserves native empty defaults. It does not infer district count, state, year, seed, geometry or audit authority. The existing modern RPLAN interchange tool remains available separately.

CSV, GerryChain, GeoJSON and reproducibility-package files use the same native builders and filenames as the CLI. Output format order and duplicates are retained, including across Save/Open; the checkbox controls and explicit format-order field edit that order. The first failed format stops the sequence and retains earlier completed files. GeoJSON may have null geometry. Reproducibility files retain source manifest/analysis bytes; `verify.sh` is downloaded as inert text and is never executed. Export does not validate assignment labels or GEOID structure beyond what the native exporter itself does, authenticate a source, reproduce an engine run or certify a plan.

Only supported selected folder sources are read, with exact bytes retained in the project and SHA-256 source/output receipts. Ignored files are not read. Preferred-source precedence and parsing run in Rust, not a JavaScript approximation. No filesystem path is opened by WASM. Folder reading is atomic; Clear, relabeling, changing settings and reopening retire pending reads/results, and an in-progress read checks cancellation before reading another file. Open remains inert and generated ZIP access requires a fresh result. ZIP construction checks each output hash and rejects edited results.

Bounds: a 1–100 ASCII letter/digit/underscore/hyphen label, up to 504 selected sources, 3 MiB per source, 25 MiB combined source bytes, up to 100,000 assignments, 1–8 ordered formats and 64 MiB combined export output. Native `usize` metadata/assignment values must fit wasm32. Browser folder observations require bounded portable paths; native unrestricted discovery, large original all-state fine capacity and platform installation remain unfinished.

`test_export_folder.mjs --original-ri-block` verifies twelve successful/partial actual CLI file sets and exact file bytes, twelve full native-host/WASM results and five matching preferred-source refusals. It compares the shared CLI filesystem loader document, source hashes, exact project roundtrip, malformed/invalid-UTF-8 ignored sources, missing metadata defaults, raw analysis retention, geometry, duplicate/reordered formats, and native unvalidated assignment semantics. One case exports the original 25,649 Rhode Island block assignments retained from the previously verified prepared Run; it does not execute that engine run again or supply new geometry. The rebuilt CLI's 22 existing export tests pass.

`test_export_folder_browser.mjs` runs four actual Chrome worker cases, including those original block assignments, partial output and reordered/duplicated formats, and checks exact retained sources/results, inert Open/Save/reopen, selective reads, pending-read cancellation and relabeling. `test_export_folder_zip.py` independently checks 16 ZIPs/85 exact files, CRCs and safe paths. Existing full-template binding (289 templates/592 native comparisons/all 49 option fields), original-native source jobs (137 native tool matches), generated reports and project UI gates pass on the new WASM artifact.

Evidence: `target/export-folder/evidence.json`, `target/export-folder/zip-evidence.json`, `target/export-folder-browser/evidence.json`, `target/export-folder-browser/export-controls.png`, `target/export-folder-cli-tests.log`, `target/export-folder-binding-regression.log`, `target/export-folder-source-regression.log`, `target/export-folder-report-regression.log`, `target/export-folder-report-browser-regression.log`, `target/export-folder-project.log`, and `target/export-folder-assets.log`. WASM SHA-256: `2676bc714d13b8bb0a04f94e1e6e6b66b3c6f3234b3542560fee5062d632a058`. Staged toolkit: 212 files/785 resolved local dependencies. Local and unpublished; full parity remains in progress.


### Native CLI import delivery and filename dispatch

The workbench now offers **Import plan with native CLI delivery**, alongside the existing context-aligned modern interchange import. It calls shared native `import_source` parsing and legacy file construction. Explicit `gerrychain` or `dra-csv` selects that parser; other native format strings select DRA CSV for a lowercase `.csv` source basename and GeoJSON assignment properties otherwise. The original format flag remains the manifest's `source_tool`, while `parser_used` records the actual parser. Shapefile-family/ZIP sources are refused, as in the CLI. No source is parsed or imported automatically when a project opens.

This mode emits `{label}.rplan`, `final_assignments.json` and `manifest.json`, with native state-code normalization and FIPS lookup (including native `00` for an unknown state), legacy metadata defaults and source notes. The shared CLI builder now writes compatibility assignments in deterministic GEOID order. Native disk order remains labeled RPLAN, assignments, then manifest. Browser metadata records the selected basename instead of a local absolute path, and explicit host time replaces the native clock. Civic imports default an omitted submission timestamp to that recorded import time. A missing required civic submitter retains the first two native files with a manifest-stage error, matching the native partial-output boundary. ZIP delivery verifies every output hash; source bytes and metadata survive inert Save/Open.

Legacy import follows native behavior for sparse district labels and empty GerryChain assignments. It does not perform the modern context-alignment or contiguous-label checks, infer geometry/populations, execute an engine, run a population/contiguity audit or establish legal compliance. Native manifest submission tags, including `authoritative`, are recorded source declarations, not authenticated official status. The existing modern importer remains the separate choice for explicit RCTX alignment and modern RPLAN output.

Bounds: a portable 1–100 character filename label, a two-letter ASCII state, four-digit recorded year, 50-byte format flag, 200-byte basename/recorded metadata, 3 MiB source bytes, 100,000 tract assignments and unsigned wasm32 district IDs. `4294967295` is retained exactly; `4294967296` is rejected rather than truncated. The native 64-bit CLI can accept the latter, so that case is explicitly a browser platform restriction, not a matching-refusal claim. Scripts/paths/URLs from sources are not executed or opened. Folder installation, unbounded native sizes and full platform parity remain unfinished.

`test_native_plan_import.mjs` checks 17 actual CLI cases against full Native-host/WASM results and per-file text, normalizing only native clock values and the native absolute source path to the supplied host timestamp/basename. Six parser/guard refusals agree; a separate above-wasm32 case is accepted by the 64-bit CLI and rejected by the bounded adapters. Cases cover native filename/explicit format precedence, FIPS and unknown-state defaults, civic default/explicit timestamps, missing-submitter partial files, BOM, sparse district labels, empty GerryChain, exact source hashes, projects, source-read bounds and output hash refusal. The 27 existing rebuilt CLI import tests pass.

`test_native_plan_import_browser.mjs` executes five actual worker cases and checks exact retained sources/results, CSV filename dispatch, civic defaults/partial delivery, empty/unknown defaults, inert Open/Save/reopen, selection without execution, late-read Clear and edited-source download retirement. Independent `test_native_plan_import_zip.py` checks 22 archives/64 exact files, CRCs, expected two/three-file sets and safe paths. Existing modern import passes 14 Native/WASM matches and 15 refusals; full-template binding passes 289 cases/592 comparisons/all 49 option fields; native source jobs pass 137 tool matches; existing native folder export passes 12 CLI/native cases including the previously verified original RI 25,649-block assignments.

Evidence: `target/native-plan-import/evidence.json`, `target/native-plan-import/zip-evidence.json`, `target/native-plan-import-browser/evidence.json`, `target/native-plan-import-browser/controls.png`, `target/native-import-cli-tests.log`, `target/native-import-modern-regression.log`, `target/native-import-binding-regression.log`, `target/native-import-source-regression.log`, `target/native-import-export-regression.log`, `target/native-import-project.log`, and `target/native-import-assets.log`. WASM SHA-256: `9c84ef3c403ff2d6b2cacc226aafda920dc83e44301f8c33f91e8eea5d22ef92`. Staged toolkit: 213 files/789 resolved local dependencies. Local and unpublished; full parity remains in progress.


### Native migration from a selected base directory

**Inspect native migration base** now resolves a legacy source under the selected native base (`states/` and `plans/`). Choose a directory in read-only mode, Run to inspect source selection, choose **Retain selected source and target files**, then Run **Stage native base migration** to create the ZIP. The shared CLI selector first tries the lowercased identifier directly and only then resolves a state code through the native manifest. An optional explicit resolution manifest replaces the built-in source; malformed resolution text is ignored when the direct directory already exists, matching native lazy lookup.

The directory picker observes directories, including empty ones, and file metadata without calling file-body readers. Only referenced source/target file bodies are retained after a fresh native selection. Without Force, an observed target manifest invokes the shared native collision rule; the host reads only the collision manifest before that refusal. With Force, source bytes replace corresponding target files while unrelated target files and empty directories remain. Existing targets without a completed manifest may merge without Force, as in the CLI. Native migration replaces the root manifest, removes the temporary manifest, counts distinct unsigned 64-bit assignment IDs and retains its one-district fallback for missing/empty/invalid assignments. It does not infer year, authentic provenance or plan correctness.

The output is an immutable, hash-checked snapshot ZIP. The app never writes to the chosen directory. Save/Open retains data and observations, not filesystem handles or permission authority; Open runs nothing. A retained complete snapshot can be explicitly restaged without reopening local files. Editing its recorded timestamp preserves those retained bytes while retiring prior output authority. Live capture authority is held privately and cannot be recreated from serialized observations. Scope changes, Clear, cancellation and metadata/size drift retire reads before another file body is requested; failed reads do not adopt a partial snapshot.

The fallback folder input can observe only directories implied by files and is explicitly labeled **empty folders not observed**. Use the directory picker when empty-directory precedence matters. The browser picker is feature gated. API references: [directory entries](https://developer.mozilla.org/en-US/docs/Web/API/FileSystemDirectoryHandle/entries) and [read-mode directory picker](https://developer.mozilla.org/en-US/docs/Web/API/Window/showDirectoryPicker). A picker grant is not a certificate of source authenticity.

Bounds: 10,000 portable observed paths, 32 directory levels in the host walk, 2 MiB observation/resolution text, 512 selected source/target files totaling 32 MiB, 2,000 output ZIP entries and 240-byte output paths. Case collisions, file/directory conflicts, reserved manifest directories and noncanonical reserved-path case are refused. Directory evidence is matched with ASCII case folding; selected bytes remain exact. Native unrestricted Unicode/path/size behavior, symlinks, permissions, partial filesystem-write failures and coordinated installation are not emulated by immutable staging. Full platform parity remains unfinished.

`test_migration_base.mjs` compares thirteen actual CLI file and directory outcomes (clock normalized only), fifteen complete Native-host/WASM selections and five agreeing native refusals. Cases cover direct precedence over manifest resolution, empty direct/empty source folders, nested empties, full identifiers, malformed assignments and resolution text, full-u64 resolution counts, source manifest replacement, existing targets with/without manifests, Force preservation/overwrite, temporary markers, missing sources and file/directory failures. Host tests cover metadata-only capture, referenced body reads, cancelled/changed selections, metadata drift, unavailable serialized capture authority, unsafe observations, unexpected byte paths and altered output hashes. The twenty existing rebuilt CLI migration tests pass.

`test_migration_base_browser.mjs` uses real directory handles in a private origin filesystem with an injected OS chooser; it does not drive a real OS permission dialog or user directories. Six successful commands and a collision failure have native-host counterparts. It checks read-mode options, no bodies during observation, collision-only reads, Force merge with binary/retained target bytes and empty folders, empty-direct precedence with no source reads, inert Open, loss of live directory authority, exact restaging from retained bytes, timestamp edits, Clear and operation-change cancellation. Production write calls and remote requests are refused. Independent ZIP checks cover fifteen archives, thirty exact files and eighty-two explicit directories, including CRCs and empty-directory attributes.

Receipts: `target/migration-base/evidence.json`, `target/migration-base/zip-evidence.json`, `target/migration-base-browser/evidence.json`, `target/migration-base-browser/controls.png`, `target/migration-base-cli-tests.log`, `target/migration-base-project.log`, `target/migration-base-binding-regression.log`, `target/migration-base-source-regression.log`, `target/migration-base-import-regression.log`, `target/migration-base-export-regression.log`, and `target/migration-base-assets.log`. Regressions retain all 49 option fields (289 binding cases/592 Native comparisons), 137 native-source tool matches, 17 native import comparisons and 12 native export cases. WASM SHA-256: `1bf029c582d8726505ce5b8631885d8fa7ed9eeb439b0b9100f4160f1876217e`. Toolkit staging resolves 214 files/794 local dependencies. Local and unpublished; the full WASM goal remains active.

### Read-only Doctor plans collection

Choose **Plan artifact inventory**, then **Choose plans collection (read only)** and select the native `plans` directory. Every immediate plan directory is included, even an empty folder or a folder containing only unrelated files. Click **Run** explicitly to inspect the retained inventory. **Download native table** uses the same table formatter as `bisect doctor --all`; the collection path header is omitted because the browser does not obtain an absolute disk path. Long Unicode labels truncate safely in both implementations.

Only supported manifest and analysis JSON bodies are read. Maps, audit files and named RPLAN packages are presence-only. Empty or malformed JSON remains present; invalid UTF-8 or failed reads remain unreadable rather than disappearing from the inventory. Native PASS may mean presence alone. A directory at a supported JSON filename is present but unreadable. Readable raw bytes, including invalid UTF-8 and empty files, retain their source hashes. None of this recomputes analysis, authenticates provenance or establishes legal readiness.

The picker is feature-gated and read-only. The file-input fallback discovers plans from supported artifacts and cannot observe empty directories. Limits are 100 plans, 1 MiB per JSON, 16 MiB total, 10,000 observed directory entries and native manifest counts fitting wasm32. No unrelated file bodies are read. Cancel, Clear, route changes and project changes stop adoption and subsequent body reads; an already-issued browser read cannot be revoked. Changing source metadata rejects the selection. Read failures are recorded as unreadable inventory observations.

Directory handles are never serialized. Saved projects retain selected bytes and presence claims; reopening runs nothing and grants no directory access. Explicit reruns use retained inputs. Test evidence covers nine actual filesystem directories with a byte-exact native CLI table, native-host/WASM comparisons, and real private-origin browser directory handles with an injected chooser. OS dialogs, symlinks, unrestricted filesystem sizes, authentication and full parity remain outside this evidence. This increment is local and unpublished.

### Native State configuration filter in the laboratory

The WASM laboratory exposes **Configuration filter** under **Population and run settings**. **Open** retains the default behavior. **Court filter** passes the native State/States filter to the engine, including ordinary runs, population retry runs and selected states in national mode. Settings, requested/recorded engine options and practitioner exports retain the selection. Reopening a project executes nothing; imported provenance remains unverified. The engine also accepts native alias `bulletproof` in exact prepared requests.

This preserves actual native behavior: partisan-share presence is rejected first, then partisan/proportional partition modes, proportional weight overrides and searches other than Single/Convergence. Native State filtering intentionally ignores structure names and seed; unseeded Single and Spectral can therefore pass this filter. The native rejection wording does not establish that accepted configurations are deterministic or legally defensible. Multiscale checks its outer search rather than passing through a Single initializer. The existing YAML court validator remains separate. No policy is silently tightened or substituted.

Evidence includes 17 original CLI preflight observations (continuation or refusal, not completed generation), 11 full native-host/WASM executions across ordinary/retry/two-state national/Spectral cases, exact refusal and option-tamper guards, and real browser controls with inert Save/Open. The broader prepared-binding gate covers 302 templates, 620 full native comparisons and all 50 exposed option fields on the compiled artifact. Engine-facing declaration coverage is 236 fields and remains distinct from runtime parity. This increment is local and unpublished; full parity, unrestricted capacity and authentication remain unfinished.

### Civic URL snapshot archives

For **Civic ingestion**, Run once to inspect the selected CSV and list its source URLs. Click **Fetch listed URL snapshots** explicitly, review the retained observations, then Run again to package them. **Import HTTP observations JSON** also accepts retained native HTTP observations. Successes, HTTP error responses and failure attempts are recorded. Clear and Cancel do not discard the previous retained observations during an incomplete acquisition. Opening projects or archives never fetches URLs; rerunning an ingestion packages the retained observations without reacquisition.

The shared native snapshot kernel retains the first 5 MiB of each body, records truncation using one additional observed byte, hashes the retained body, and writes native `snapshots/<sha8>.body` and `<sha8>.headers.txt` sidecars. Native sorted URL order and shared-prefix overwrite behavior are preserved, including identical bodies with different response headers. Every selected CSV URL needs exactly one success or failure observation when snapshots are enabled. Limits are 20 URLs, 64 KiB headers per response and 16 MiB total response observations. Use **Save workbench archive** for retained data exceeding the ordinary 25 MiB project limit; the existing 128 MiB disk/256 MiB expanded archive boundaries remain.

Browser acquisition requires a source that passes native structural URL checks, public HTTPS without embedded credentials, CORS access and no redirects. It omits cookies and referrer and uses a 10-second timeout per URL. The page CSP permits HTTPS connections for this explicit action; Census acquisition continues to use its pinned endpoint validation. CORS, redirect, transport and timeout failures produce status-zero records. Body bytes, including HTML, are never rendered or executed. Cancellation or project changes stop adoption and subsequent URL requests; an already-issued request may have reached its server.

Browser captures use `browser-fetch-observed-headers-body` and a `BROWSER-FETCH` header marker because browser APIs do not reveal HTTP protocol versions or all response headers. They do not fabricate the native HTTP header stream. Native transport allows HTTP and up to three redirects; those differences remain explicit. Imported `headers-body` observations retain declared native metadata and bytes as unverified source claims. Timestamp labels, HTTP observations and body digests do not authenticate a remote publisher or establish legal readiness.

Observation JSON is an array of objects with `url`, `fetched_at`, `http_status`, `content_type`, `body_base64`, `headers_text`, `snapshot_format`, and `error`. Successful observations have an HTTP status and exact canonical Base64 body, including an empty encoding; failed observations use status zero, empty content type, null body/headers and an error string. For a truncated body, retain the first 5 MiB plus one observed witness byte in the input; only the first 5 MiB enters the archive.

Evidence covers five original native HTTP snapshots (binary/empty/404/redirect/oversized), full native snapshot ingestion, browser streaming and real controlled HTTPS/CORS responses, a three-body archive exceeding the ordinary project limit, independent ZIP CRC/path/body-hash checks, existing civic regressions and all 50 engine-option binding fields. Browser evidence uses an isolated local TLS server and test DNS mapping, not a live external provider. This increment is local and unpublished; full parity remains unproven.

### Portable failed worker requests

The practitioner workbench now retains a **Last failed worker request** record when a worker returns an error. It includes the error, recorded time, submitted operation, loaded worker-measured WASM SHA-256/length, serialized request SHA-256/length and retained request JSON when available. Seed declaration text is shown from known request fields and YAML text without resolving defaults or claiming an accepted seed. Opaque request JSON keeps exact unsigned 64-bit seed tokens; displays do not substitute rounded unsafe integers.

Use **Download failure receipt** to preserve the diagnostic record, **Download retained request** for the original serialized JSON, and **Check retained request hash** to check byte integrity. Downloads include request data and any embedded source bytes. Request export requires a matching digest. Metadata and digests do not authenticate execution or publishers. Worker-owned runtime bindings may override submitted fields, so the receipt is not a resolved configuration or a trace of every inner attempt. It does not claim that no partial work occurred or replace retained checkpoints and actual plan results.

Receipts travel with ordinary projects or workbench archives. Opening them runs and fetches nothing; imported receipts are marked unverified. The last receipt remains separate from later input edits/results until **Clear failure record**. Pending hash/download work is retired when its receipt is cleared or replaced. JSON request retention is limited to 25 MiB; larger JSON requests retain their full digest/byte count and require the original source project. Host File/Blob objects, cycles and other non-JSON/uncapturable requests require their original local sources. Fatal worker termination may prevent a receipt. Large combined sources and receipts use the existing workbench archive boundary.

Evidence uses the complete original 2020 Alabama tract graph (1,437 units) and the existing ApportionRegions failure. WASM and the native API return the same error; independent native YAML dispatch rejects the same 3% research limit with different diagnostic wording. Real-browser checks verify exact request/receipt downloads and native API replay, inactive project Open, visible seed, altered-request detection, inert HTML error text, cancellation of a pending download after Clear, and worker-measured identity despite a forged caller identity. No alternative algorithm is selected and no plan is fabricated. This increment is local and unpublished; full parity remains in progress.

### Laboratory state and national failure records

The laboratory retains returned worker failure receipts on the individual failed state. Select that state in the national results table to see its error, recorded WASM identity, request digest and seed declaration text. **Failure receipt** downloads the diagnostic record; **Retained request** checks its digest before downloading the exact serialized request. Exports can contain embedded source data. No accepted plan or metrics are fabricated for a failed state, and other accepted states remain available in a partial national run.

Saved laboratory projects retain these optional per-state records. On Open, request bytes/digests are checked against a reconstruction from the saved configuration and pinned, hashed prepared graph before committing the imported project. Only prepared catalog assets are read; requests, embedded URLs and engine operations are not executed. A changed digest, request/configuration mismatch or inconsistent failed-state record rejects the import and keeps the current project. Imported execution and runtime identity claims remain unverified. Older failed projects without receipts remain readable. Omitted request bytes still require original local sources; existing project and retention limits remain.

The catalog ignores stale error IDs and events from retired worker instances. Execution and import use the same request-construction path for ordinary, retry and multiscale runs, with existing accepted-result checks preserved.

Evidence includes real native filter refusals for two states, native request replay, real Alabama failure alongside an accepted Vermont result, a controlled partial-transport fixture, stale-worker guards and actual browser Save/Open/download/tamper checks. A project containing failure records for all 50 prepared catalog states reopens without workers and fits within 25 MiB; those are filter refusals, not 50 successful generations. Electoral-setting, multiscale and legacy laboratory project checks pass. This increment is local and unpublished; the complete WASM parity goal remains unproven.
