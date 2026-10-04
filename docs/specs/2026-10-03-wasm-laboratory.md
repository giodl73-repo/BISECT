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

The full WASM goal remains active and incomplete. Remaining work includes further structures/weighting/search combinations, multiscale preparation, exact solvers, analysis/library/archive/offline equivalents, broader state/year/chamber data coverage, and verified download arrival on disk. This phase and the CSV/TSV phases remain local after PR #53.