# Build the browser practitioner preview



The complete WASM practitioner-suite port is active work. Its scope includes

engine options, certification, election tools, analysis/reporting, data/plan

management and browser replacements for TUI workflows. The

[specification](../specs/2026-10-03-wasm-laboratory.md) and

[command coverage inventory](../specs/wasm-capability-inventory.json) track the

completion gates. Pending entries are not available simply because Rust builds.



## Reproduce the current preview



The Rust `wasm32-unknown-unknown` target must be installed. From the repository

root, run:



```powershell

scripts/web/build_wasm.ps1 -Output dist/wasm-preview

python -m http.server 4319 --directory dist

```



Open `http://127.0.0.1:4319/wasm-preview/`. Output must be new or empty. The

builder compiles the release module and runs actual WASM execution tests before

copying assets. It uses no C METIS or generated wasm-bindgen glue. A raw buffer

ABI invokes the shared Rust implementations, and the browser provides Web Crypto

entropy. The module runs in a Web Worker with serial iterator paths; cancellation

terminates the Worker. No file contents are submitted to a backend.



The preview includes RPLAN validation/conversion, profile-driven audit

certificate generation, verification against a plan/context, and an initial

RCOUNT package-object verification operation, complete package-folder verification

with source hashes and election audit replay. Aggregation and external-source

CSV/NIST import workflows remain pending.

The example button loads the repository's public grid fixture. Generate its

certificate, inspect the checks, then use Verify this certificate. Download

result JSON preserves the certificate for use with other RPLAN tools.



## Current verification and limits



The optimized module was 2,839,014 bytes on 2026-10-03. Actual WASM tests executed

seven structures and nine additional search cases on a grid, checked assignment

coverage and independently recomputed connectivity. The engine boundary reports

actual balance and connectivity, including a BFS fixture that did not meet the

requested tolerance. These smoke cases do not prove full state/option coverage.



RPLAN tests imported an existing native certificate, verified its hashes,

generated a WASM certificate with explicit browser provenance, verified that

certificate and rejected a modified certificate. A browser Worker check

generated and verified the public fixture certificate under a nested static URL.



The test suite exposed and fixed Forest ReCom's acceptance of a replacement

pair when only one district passed balance. Both forward and reverse cut counts

now require both components to pass; a three-vertex regression checks the

asymmetric-total case. Keep this shared fix in native and browser builds.



Engine-wide native/WASM seed equivalence, AreaSection dual constraints, remaining

structures/advanced options, real-state engine UI integration, all-state input

packages, ceremony/exact workflows and the remaining practitioner screens are

still required. In particular Rand's SmallRng varies by architecture, and seed

derivations must use fixed-width integers; both need auditing before claiming

native/WASM reproducibility. Browser certificate checks are not independent

external attestation or a proof of source custody.



Check inventory drift with:



```powershell

python scripts/web/wasm_inventory.py --check

```



Regeneration preserves implementation/evidence fields, but new CLI commands

start pending. Publication is separate from this local preview.





The practitioner preview includes Save project, Open project and Restore autosave.

A versioned `.bisect` JSON file contains parsed JSON inputs and optional exact RCOUNT package files,

selected operation and checks, project name and latest completed result. It can

be opened without access to the original input paths. Downloads are portable;

IndexedDB autosave is scoped to the current browser and origin, and may be cleared

by the browser. Storage failures display a request to save a project file.

This format bundles raw RCOUNT package directories, but engine state geography

and a full history of runs remain part of the suite port.



`node scripts/web/test_project.mjs` checks snapshot roundtrips with the real audit

fixtures, snapshot independence and invalid format rejection. The browser smoke

check generated a certificate, requested a project download, reloaded the page

and restored its inputs, settings and certificate from IndexedDB.





Project import security: version 1 accepts only its declared top-level fields,

listed input kinds and operations/checks. Imported content is data, rendered

with textContent; it cannot supply scripts, worker URLs or paths to read. Opening

never runs the engine or verifies a saved result automatically. Saved results are

explicitly labeled unverified. The workbench uses a same-origin content security

policy with object embedding, base changes and form submission disabled.



Imports are limited to 25 MiB of UTF-8 JSON, 64 nesting levels and 500,000 values.

A dedicated worker reads and parses the selected file and is terminated after

success, failure, cancellation or a 10-second timeout. Starting another import

or restoring an autosave cancels the previous import worker. Prototype-sensitive keys, accessors,

cycles, nonfinite numbers, duplicate checks and unknown fields are rejected.

No current state is replaced until validation succeeds, and an edit or run during

import prevents replacement. Autosave restores and input/example reads also

check for intervening changes before committing their results; a failed input

read retains the prior input. UI-handler tests exercise delayed reads and

superseded imports. The local JSON format is not encrypted or signed;

a saved pass result is not proof of authenticity. Browser autosave is local to

the origin and not an encrypted vault. Large future engine projects will need

a bounded streaming/binary format rather than increasing these limits blindly.



Security tests exercise these limits and worker cleanup. Browser tests opened an

HTML-bearing project, showed the markup as plain JSON and labeled the result

unverified; a subsequent prototype-key import was rejected with the loaded inputs

and result still present. These tests do not substitute for an independent

security review of the full suite before publication.





Complete election packages are now selectable as a local folder or restorable

from a `.bisect` project. Exact file bytes are kept as base64 records under

`files.packageFiles`; decoding occurs only for an explicitly requested WASM run.

The package importer rejects traversal, absolute/URL paths, duplicate entries,

invalid encodings, over 2,000 files and over 8 MiB total decoded bytes. No files

are extracted to disk. The same JSON project security limits still apply.

The current directory-picker workflow was tested in the in-app browser; project

import remains the portable way to transfer a bundled package between browsers.



`verify-count-files` runs the shared native RCOUNT audit checks, including

manifest/package hash equality and hashes of exact source bytes. Source path

separators are normalized before path validation for native Windows package

compatibility. `replay-count-audits` runs the native statistical replay and returns

both verification and replay evidence. A source/consistency failure keeps the

overall result failed while allowing the same diagnostic calculations as the

native replay command. Missing methods retain native boundary results. Empty

run lists are marked boundary rather than a successful replay.



Native reference generation and the WASM test cover comparison, Kaplan–Markov,

Minerva multi-round, Athena boundary and invalid comparison packages. Additional

source-byte modification and traversal tests fail correctly, and the summary

package matches its committed native golden transcript exactly. Browser tests

selected a real local package directory, verified it and replayed Minerva rounds.

The UI shows bounded check/round tables plus the complete JSON result; displayed

p-values use the exact native parts-per-million values. Saved/reloaded package

projects retain source bytes and can be rerun. Verification establishes internal

consistency against supplied hashes, not external authentication of those hashes.

The full engine/practitioner goal remains incomplete; consult the inventory.





## Live browser laboratory



`scripts/web/build_wasm_lab.ps1 -Output dist/wasm-laboratory` builds a static

laboratory with prepared inputs for RI, IA and NC (2020) and preserves the

published saved-run catalog. The builder prepares versioned graph assets from

the same native binary adjacency, ordered GEOIDs, geographic centroids, physical

edge lengths, areas and exterior perimeters. Every graph and display asset is

hashed. Display geometry is reused from saved runs when available, checked for a

complete GEOID join, and never used to derive engine boundaries.



The UI calls a WASM adapter rather than `/api` network endpoints. State and

national jobs run in a dedicated Worker, with states processed sequentially.

Working graph caches are released after each state; initialized module memory

is reused. Cancellation terminates the Worker and retains completed results.

Saved catalog maps still open, and comparisons, district metrics, run-record and

GeoJSON downloads use the existing laboratory UI. Save project now downloads a

portable browser experiment; Open project and Restore autosave recover its

settings and completed assignments. Other runs in tab history are not currently

autosaved together as a library.



`node scripts/web/test_wasm_lab.mjs dist/wasm-laboratory` runs real WASM graph

execution behind a Worker test double, verifies a three-state batch's connectivity,

5% population tolerance, map joins and a 128 MiB pilot module-memory bound. It

also checks saved maps, failed asset integrity and cancellation before startup,

during module initialization and while a Worker is running. Browser smoke tests

ran Rhode Island and the same three-state national cohort through actual Workers.



The Rust n-way adapter now invokes direct k-way partitioning rather than recursive

bisection, enables contiguity/minimum connectivity and scales its coarsening target

with the requested district count. The grid test and native backend tests pass,

but the NC fourteen-district case still misses a 5% balance target substantially.

An independent population recomputation validates the displayed failed flag;

this remains an unresolved engine quality issue, not a successful constraint gate.

Native/WASM exact seeded assignment equivalence also remains pending. This

preview exposes seven structures; AreaSection and advanced/missing CLI options

remain required by the full specification.



All 50 states' 2020 prepared inputs and shared maps are exported to

`dist/wasm-national` (approximately 172 MB). Input validation checked 84,208

tracts, asset/native graph hashes, connectivity, populations and complete map

joins. Allocated-count execution is a separate gate:

`node scripts/web/test_wasm_national.mjs dist/wasm-national` records all states

and fails unless all 50 plans meet both connectivity and the requested 5%

population tolerance. The current release completes all 50 with contiguous

plans, but CA and TX fail the population criterion (48 of 50 pass). Full national

browser verification and engine parity remain pending.



The Git `metis-core` dependency is patched to `vendor/metis-core`, preserving its

MIT license and pinned revision. See `vendor/metis-core/BISECT-PATCHES.md` for the

slow-coarsening, contiguous pre-balance, aggregate-weight limits and targeted

population-growth fixes. Verification: 184 partitioner

unit tests, 264 pure-Rust runner tests (14 ignored), and 16 release WASM smoke

cases. Smoke diagnostics retain the known BFS balance failure; these tests do

not prove all-state balance or the full options gate.



To inspect individual split failures and test runtime parity, build

`cargo build -p bisect-wasm --example trace_splits --release --offline` and run

`node scripts/web/test_native_wasm_splits.mjs dist/wasm-national`. This compares

all CA/TX tract assignments and exact population metrics from native Rust and

WASM for standard geographic bisection, seed 42, 10 iterations and 5% tolerance.

The report `target/native-wasm-split-parity.json` records both executable/module

hashes and the failing split paths. Current assignments match across runtimes;

the balance misses occur in three-district nodes CA `1100` and TX `1111`.

Fixed-width random vertex sampling removes the discovered `usize` RNG mismatch.

Other engine structures/searches and native C METIS parity remain unverified.



Standard bisection now supports `multi` in the shared runner, native CLI and

browser laboratory. At each node it tries exactly the requested consecutive

seeds starting at the configured seed, rejects cuts that miss either child

population target or contiguity, and chooses the least weighted boundary cost

(seed index breaks ties). Unweighted graphs use cut-edge count. This evaluates

the sampled cuts at that node without exploring child plans. A node with no

feasible candidate reports an error; this is not a proof of the global best cut.

The native CLI previously fell through to single-seed execution for this mode.



The percentile seed walk now hashes a fixed-width `u64` index so native64 and

WASM32 agree. Failed seed executions propagate an error instead of becoming

invented one-district, zero-cut candidates. Its ranking remains the existing

CLI's unweighted cut-edge-count ranking, unlike Multi's weighted cost ranking.



Build `cargo build -p bisect-wasm --example execute_request --release --offline`

and run `node scripts/web/test_search_parity.mjs dist/wasm-national` for exact

RI/IA native/WASM assignment comparisons for both searches with eight seeds.

All four comparisons pass and meet 5% population tolerance. The laboratory

adapter test also checks Multi's recorded search and seed budget. An actual

browser Worker run produced a connected RI two-district map at 2.300% maximum

deviation; screenshot: `target/wasm-multi-browser.png`. Current validation has

267 pure-Rust runner tests passing (14 ignored), 13 relevant C METIS backend

tests passing (one ignored), 17 release WASM smoke cases, and a successful CLI

compile check. The full national Multi matrix and remaining options remain open.





The laboratory project format `bisect-laboratory-project-v1` stores one state or

national experiment, exact active settings, completed assignments, metrics and

prepared-graph/display-geometry hashes. Shared published inputs and geometry are

referenced rather than bundled. Opening requires the matching input catalog;

this format is not yet a self-contained offline data archive. A snapshot of an

unfinished batch keeps completed states and labels remaining states interrupted.

Opening or restoring never resumes a run automatically.



Project parsing uses a cancellable Worker with the same 25 MiB, depth-64 and

500,000-value limits as practitioner projects. Unknown top-level fields, unsafe

properties, malformed identities and configuration/assignment inconsistencies

are rejected. Before committing, the adapter independently recomputes population,

connectivity, boundary costs and county splits against hash-checked prepared

graphs. Changed inputs, metrics, options or assignments reject the entire import

and preserve existing history. Recorded engine provenance remains an unverified

imported claim; a consistent plan is not proof of which algorithm generated it.

Assignment and algorithm-history replay also runs in a separate, cancellable

Worker, with a 30-second limit per state. A timeout, cancellation or Worker

failure rejects the import before any result is committed. This keeps expensive

imported histories from blocking the browser UI. This verification Worker checks

saved evidence; it does not run the partition engine.

`node scripts/web/test_assignment_verification.mjs` exercises the real Worker

handler on valid and tampered assignments, plus timeout, cancellation, crashes,

malformed replies and termination.

Browser verification also opened a valid RI/IA project through this Worker, then

rejected changed boundary metrics and duplicate JSON fields while preserving its

loaded settings and both completed state results.

The WASM laboratory exporter now includes a same-origin script/Worker/connect

CSP; inline styles remain permitted for existing map styling.



`node scripts/web/test_lab_project.mjs dist/wasm-national` verifies actual WASM

county-weighted eight-seed Multi experiments for RI/IA, portable roundtrips,

complete map joins, independent measurements, tamper rejection, cancellation and

edit races, historical provenance and unfinished snapshots. Both laboratory test

scripts accept state-list and year arguments, and the builder passes its selected

inputs rather than assuming the three-state 2020 pilot. Direct n-way imbalance

is still independently reported; the test no longer requires NC to keep failing

after future improvements. Actual browser evidence reopened a two-state project,

restored it after reload and downloaded a 36,758-byte `.bisect` file. The downloaded

file was parsed and independently checked again without executing an engine:

1,146 assignments, Multi, eight seeds. Screenshot:

`target/wasm-laboratory-project-browser.png`. Autosave stores the latest browser

experiment in origin-scoped IndexedDB; Save project remains the portable backup.

Full history/session libraries and offline prepared-package import remain open.



The national Multi matrix has also passed all 50 states for 2020 congressional

allocations, geographic weights, seed 42, eight seeds per node, 10 iterations and

5% tolerance. `target/wasm-national-multi-validation.json` records 50 completed

and 50 valid states, with independent assignment coverage, connectivity and

population checks. Maximum retained module memory was 24,379,392 bytes. This

does not measure total browser/JS/map memory, validate every engine option or

prove an optimal cut; a full 50-state browser Worker/map run remains pending.





AreaSection (`ratio-optimal-area`) is now available in the WASM laboratory with

single or Multi search. The Rust adapter retains both interleaved population

and hectare weights and their independent tolerances. Joint initialization and

multiconstraint pre-balance/refinement replace population-only processing for

these calls. The two-district shortcut no longer drops the area constraint, and

scalar boundary rebalancing cannot overwrite dual-constraint results.



Root candidates must be connected and meet both physical population upper

bounds (each target multiplied by 1.001) and physical land-area upper bounds

(each half-area target multiplied by `area_swing`). Thus swing 1.1 permits

45–55% of land on either side. The Lorenz pre-filter uses those same bounds.

No feasible sampled candidate produces an explicit error. The area constraint

applies only at the root, matching the existing AreaSection design; descendants

use population-only ratio search. This does not impose equal area on every final

district. Root ratios, area/population totals and targets, achieved area fraction,

active multipliers and seed budget are recorded and displayed. Saved projects

independently recheck that evidence before importing it.



GeoSection/AreaSection now use the configured consecutive seed walk from

`base_seed`, including descendants. Native CLI dispatch passes its configured

seed and retry seed into the same shared implementation. The legacy

`run_geosection` API remains a wrapper starting at seed one. Previously, the

WASM/native configuration's seed was ignored by this ratio-search path, and the

two-district shortcut also ignored additional seeds.



`node scripts/web/test_area_wasm.mjs` tests tight versus loose area swing,

infeasible indivisible area, zero-area input and numeric aggregate overflow.

Build `cargo build -p bisect-wasm --example execute_request --release --locked --offline`

and run `node scripts/web/test_area_parity.mjs` to compare native Rust with actual

WASM: two controlled fixtures plus RI/IA 2020 congressional plans. Complete

assignments and district metrics match; physical root constraints are separately

checked. RI uses 46.284% of land on the left at 0.031621% maximum population

deviation; IA uses 53.004% at 2.400819%. Both are connected and within the chosen

5% final population tolerance, using seed42,32seeds,20iterations,swing1.1.

Report: `target/native-wasm-area-parity.json`. The release module is 2,901,557

bytes, SHA-256 `147bb23cb75b2a2211649a8f3f90e76e796bb83a718a6e5603d46206cbefd72c`.



An actual browser Worker completed the RI configuration and saved a portable

project whose root measurements were independently checked on disk. Evidence:

`target/wasm-area-browser-project.bisect` and

`target/wasm-area-browser.png`. Current verification includes 187 partitioner

unit tests, 270 pure-Rust runner tests (13 ignored), two C METIS area regressions,

18 release WASM smoke cases, project/root-tamper tests and CLI/exporter compile

checks. These specific cases do not prove AreaSection across all states, years,

chambers or searches. Remaining engine/practitioner inventory gates are active.





Open Project security review (2026-10-03): local project and individual JSON

inputs are treated as untrusted data. Imports enforce a 25 MiB byte limit,

strict UTF-8, rejection of duplicate JSON fields (including escaped aliases),

64 nesting levels, 500,000 values, finite numbers and safe property names.

Parsing uses a disposable worker with a 10-second deadline. Individual

practitioner plan/context/profile/certificate/package JSON selections now use

these same checks. Invalid or stale imports retain current work. Opening a

laboratory project independently checks saved assignments and measurements

against the catalog's hashed inputs; it does not execute the engine or certify

imported provenance. Embedded package paths are validated and never extracted

to disk. Imported text is rendered as text, with bounded result previews.



Validation: `node scripts/web/test_project.mjs`,

`node scripts/web/test_package_files.mjs`, and

`node scripts/web/test_lab_project.mjs` passed. These cover hostile properties,

markup, oversized/deep input, duplicate fields, invalid bytes, worker cleanup,

stale imports, package traversal, result tampering, and preservation of current

work. This is a targeted import review, not a full application security audit.



Follow-up: parsed JSON imports also reject integer values outside JavaScript's

exact range (±9,007,199,254,740,991), before they can become practitioner inputs

or saved results. This prevents accepting silently rounded large counts or

identifiers. Exact binary package contents remain byte-preserving. A lossless

numeric JSON interface is still needed for the full native i64/u64 range;

ordinary fractional JSON numbers retain JavaScript floating-point semantics.





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



### National browser execution and portable results



The current module was executed in the actual browser Worker for all 50 states,

2020 congressional allocations, standard bisection, geographic weights, Multi

search with eight seeds, seed 42, 10 refinement iterations and 5% requested

population tolerance. All 50 completed. The downloaded project contains 435

districts and 84,208 tracts. Independent checks against hash-verified prepared

graphs found every district connected and every state within tolerance; the

largest maximum deviation was Maryland's 4.217946443256704%.



The combined map rendered all selected states in Albers projection with Alaska

and Hawaii insets. Opening the downloaded 2,373,466-byte project in a fresh

browser tab recovered all 50 results through the cancellable evidence workers,

without executing the partition engine. Complete assignment-to-geometry joins

were also checked independently for every state.



Run `node scripts/web/verify_national_project.mjs` against the saved browser

download to repeat the graph hashes, district allocations, assignment metrics,

connectivity, balance and map coverage checks. Evidence is in

`target/wasm-national-browser-project.bisect` and

`target/wasm-national-browser-verification.json`. The recorded module hash is

`d2b121eff1a0cadd85152dcb2c9a12b6f8c842de1c8c628a93ee93fb9871e61c`.

This gate proves this configuration's results and persistence; it does not prove

optimality, native parity for every state, all engine options or the full

practitioner capability inventory. The full WASM capability goal remains active.



### Flow repair option



Flow construction now exposes both native repair modes: `bfs` and `none`.

The browser's Flow repair select records the choice as `flow_repair` in the

request, effective configuration and portable project. Projects from before

this control remain compatible and use the existing BFS default. Other

structures reject flow repair parameters. Opening checks the recorded native

summary against the selected mode, as well as assignment metrics and inputs.

An unrepaired invalid construction is reported as a failure.



The 42-case native/WASM gate passed: 27 exact successes and 15 matching failures,

covering both repair modes, three boundary modes, RI/IA and feasible,

zero-population and infeasible path fixtures. The six-district path needs BFS

repair; selecting none fails in both runtimes. Both explicit modes and the

legacy default passed project/map roundtrips and evidence-tampering checks.

Existing project security, bounded evidence Worker, laboratory roundtrip,

21-case engine smoke and 65-case annealing parity gates passed.



Actual browser RI with no repair completed with 250 tracts, two connected

districts and 2.030383304218508% maximum deviation at requested 5%. Save and

Restore recovered the no-repair setting, and the downloaded file independently

reopened without engine execution. Repeat with

`node scripts/web/verify_browser_project.mjs dist/wasm-flow-options target/wasm-flow-options-browser-project.bisect`.

The isolated preview is `/wasm-flow-options/`; the previous national gate's

module/catalog remains at `/wasm-national/`.



Module: 3,277,096 bytes, SHA-256

`b12ea340d5a8f37815ff3a507b42259e503e26a345e2a5dc1f7b81d47025a16f`.

Evidence: `target/native-wasm-flow-parity.json`,

`target/wasm-flow-options-browser-project.bisect`, and

`target/wasm-flow-options-browser-verification.json`.

The full WASM capability inventory remains incomplete and active.



### SMC weighted percentile search



Standard bisection's browser search choices now include the native

`smc-percentile` compositor. Controls expose particle count (1–10,000), weighted

selection percentile, ESS resampling threshold and base seed. General seed/step

budgets and METIS refinement do not affect this search and are hidden/disabled.

Boundary weights affect reported costs; selection ranks unweighted edge cuts.

Requests, effective settings, saved projects and restored controls retain the

active SMC parameters.



The native compositor fixes sampler tolerance at 0.005, applied to remaining

component population in each proposal. It does not enforce 0.5% final district

deviation. The UI preserves this native rule, disables the tolerance control at

0.5%, explains the distinction and displays independently measured final balance

and connectivity. Completed samples that miss these checks remain visible as

misses. Neither a finite weighted sample nor its diagnostics certifies uniform

distribution calibration or optimality.



SMC now uses explicit ChaCha12 and u64 index sampling in proposals, Wilson

trees and resampling (`chacha12-u64-v1`). The legacy spanning-tree entry point

used by other searches retains its existing index draws. Native SMC seed

sequences change; old results are not reproduction references for this protocol.

Native NDJSON metadata records `smc_version: 2.0-chacha12-u64-v1`. Invalid graph,

population, tolerance and resampling inputs are rejected before proposals.

Percentile selection rejects missing positive weight and never selects a

zero-weight particle, including the percentile-1 rounding fallback.



Returned evidence includes a sorted particle weight/cut ledger, selected

particle, decimal base/derived seeds, resampling rounds and ESS trace. Open

independently verifies ledger shape, normalization, rank order, percentile

selection, selected plan edge cut, resampling/ESS consistency and final plan

metrics against hashed inputs. It does not attest every particle's generation.

The browser diagnostics panel previews 50 particles/stages and also shows the

selected particle if outside that range. Full evidence remains in project/run

exports. Rendering uses text nodes and remains bounded.



Final-module verification: 58 native/WASM cases, 49 exact assignment successes

with numeric metric/weight comparisons and nine matching rejections. Coverage

includes RI/IA, three boundary modes, percentiles 0/0.5/1, particle/resampling

controls, zero-population/feasible/infeasible paths, ignored budgets, project/map

roundtrips and tampered-evidence rejection. Sixteen successful samples missed

final balance; these are recorded, not counted as valid plans. The full runner

suite passed (278 tests, 13 ignored), plus ensemble/SMC suites, 40 SMC unit tests,

CLI/web/WASM example compilation, 21-case engine smoke, laboratory project and

import security/worker checks. Bounded diagnostics rendering passed its test.



Actual final-module browser RI, 16 particles, threshold 0.5, percentile zero,

seed 42: 250 tracts, two connected districts, 0.49144370358827905% maximum

deviation. Save/Restore recovered controls; a page reload reopened the full

project and diagnostics without engine execution. Independent downloaded-file

verification passed with

`node scripts/web/verify_browser_project.mjs dist/wasm-smc target/wasm-smc-browser-project.bisect`.

Preview: `/wasm-smc/`. Module: 3,350,671 bytes, SHA-256

`edca212714748260f2389838da3c6b91498b860025e76a4fa18c761f89c786d5`.

Evidence: `target/native-wasm-smc-parity.json`,

`target/wasm-smc-browser-project.bisect`,

`target/wasm-smc-browser-verification.json`, and `target/wasm-smc-browser.png`.

The full engine and practitioner capability goal remains active.



Open Project follow-up review (2026-10-03): selected package paths and

duplicates are now checked before reading bytes. File sizes must be nonnegative

safe integers, and actual bytes are capped at 8 MiB before base64 encoding.

Embedded election import results receive the same package path, encoding and

size checks as package inputs. Malformed parser-worker replies fail cleanly and

terminate the worker. Laboratory state codes must be strings rather than values

that merely coerce to two-letter strings.



The project, package, assignment-worker and actual WASM laboratory roundtrip

suites passed. In the isolated `/wasm-project-security/` browser preview, the

real saved Rhode Island SMC project reopened with its map and two district

results. Opening a duplicate-key file then displayed `Duplicate JSON property.`

and retained that project. Evidence: `target/open-project-security-browser.png`.

Opening imports data; it does not run the engine, extract package files to disk,

or establish the authenticity of an imported certificate or provenance claim.

This review covers the local project import path, not the entire application.



Practitioner election imports (2026-10-03): the workbench now exposes eight

operations, including statement-of-votes CSV and the native NIST CDF JSON

adapter. Select a source, choose its import operation, and supply country,

state, jurisdiction, election date/type, scope and status. Metadata drafts are

saved; all seven fields must be filled before execution. The supplied metadata

is recorded explicitly rather than inheriting the native CLI's synthetic

fixture manifest defaults. The adapters retain their native supported subsets.



The WASM operations use the shared native byte adapters and in-memory RCOUNT

writer. Original source bytes are retained in the package with source hashes.

Count-bearing JSON/NDJSON is carried as exact base64 bytes rather than parsed

as JavaScript numbers. Each source and generated package is limited to 8 MiB;

project files retain their existing 25 MiB import limit. The result preview

shows verification checks and manifest metadata; complete package bytes are

available through Download RCOUNT package archive and Download result JSON.

Use imported package for verification prepares the package inputs; Run then

performs the verification. Open RCOUNT package archive accepts the downloaded

`bisect-rcount-files-v1` envelope without extracting files to disk. Project

Save/Open retains source, metadata, results and package inputs. Imported

results remain explicitly unverified until the user runs verification again.



Validation: all 51 `rcount-io` tests passed, including disk/in-memory writer

byte comparisons for both importers. `test_election_imports.mjs` passed four

native/WASM exact-result comparisons (both adapters, 80 and

9,007,199,254,740,993 counts), five matching invalid-request rejections,

source-byte/hash preservation, archive/project roundtrips and modified-source

rejection. The build also passed engine smoke, project/security/workbench UI,

native election fixture, package verification and package roundtrip gates.

Actual browser CSV and NIST imports, downloaded projects/archives, reload/Open

and explicit package verification passed. Downloaded artifacts matched a fresh

WASM execution via `verify_election_project.mjs`. These checks concern source

hashes and recorded count equations; they do not certify an election outcome.



Preview: `/wasm-election-imports/`. Module: 3,611,545 bytes, SHA-256

`64562919c1fe880991599179e5e69589999c31b16d39bea69a3a4123539fadde`.

Evidence: `target/election-import-native.log`,

`target/election-import-preview-build.log`,

`target/wasm-election-import-browser-verification.json`,

`target/wasm-nist-import-browser-verification.json`, and

`target/wasm-election-import-browser.png`. The full engine/practitioner goal

remains active; the RI RLA importer and other inventory entries still need ports.



RI RLA import (2026-10-03): the ninth practitioner operation ports the native

RI 2024 Representative 28 ballot-polling adapter. Choose that operation and

select its audit-report, ballot-manifest and ballot-retrieval CSV files. Partial

selections persist in projects; reopening restores all selected filenames.

Three nonempty sources totaling at most 8 MiB are required. The native RI

manifest identifies the election; browser-created provenance identifies the

WASM importer. This is the RI-specific native adapter, not a general RLA format.



The shared byte adapter preserves all three original sources and their hashes,

cross-checks report/retrieval sampled-ballot keys, checks manifest totals, and

records the native source summary and public seed. Count-bearing files stay as

encoded bytes. The native importer now rejects negative and overflowing vote

or manifest totals rather than allowing unchecked arithmetic. The native

14-batch fixture has exact disk/in-memory package-byte parity. Both native and

WASM ordinary/large-count fixtures match exactly, including counts above 2^53

and public seed `34053800000000000000`. Six invalid-source cases reject on

both runtimes, and altering any of the three sources fails verification.



The adapter preserves the native claim boundary: risk calculations are

recorded without reconstructed sample-step replay, and human observations are

not independently verified. Replay therefore reports `boundary`, with the

native explanation that Minerva round-one replay requires a sample step. A

passing package/source check does not establish statistical audit success.



All 51 native I/O tests, engine smoke, existing election verification/replay,

CSV/NIST parity, package/project security and workbench UI gates passed. The

actual browser imported the large-count fixture, downloaded its project and

archive, restored all three filenames after reload/Open, then replayed both

the reopened project package and downloaded archive with the expected boundary.

`verify_election_project.mjs` confirmed the downloaded files match a fresh

WASM execution exactly. The UI shows adapter inputs only for their operation.



Preview: `/wasm-ri-rla/`. Module: 3,738,291 bytes, SHA-256

`0ac5a299a97d5dfe0be5764afe38ef63a3255390f5585cfd45f9fb97462a8bf7`.

Evidence: `target/ri-rla-native-tests.log`, `target/ri-rla-preview-build.log`,

`scripts/web/test_ri_rla_import.mjs`,

`target/wasm-ri-rla-browser-project.bisect`,

`target/wasm-ri-rla-browser-archive.json`,

`target/wasm-ri-rla-browser-verification.json`, and

`target/wasm-ri-rla-browser.png`. Other engine and practitioner inventory

entries remain unfinished; the full goal remains active.



### District election aggregation in WASM



The practitioner workbench now offers **Aggregate election counts by district**.

Select a complete RCOUNT folder/archive and RPLAN, supply a contest ID and count

status, and optionally provide RCTX plus exact crosswalk NDJSON bytes. Shared

native code checks the plan, context/hash binding and integral allocations.

Results show district and selection counts as decimal strings. Download native

aggregation transcript preserves exact integer JSON in compact or pretty form,

including counts above JavaScript's exact integer range. Opening a saved project

does not run the engine and labels its result unverified; Run recalculates it.



Resource limits remain 25 MiB per project, 2,000 files/8 MiB per package,

8 MiB per crosswalk/transcript and 500 districts in the browser. The native

crosswalk verifier currently receives no source index from this adapter, so

crosswalks containing source references are rejected exactly as in the native

command. Nonintegral allocations are also rejected. Modified original source

bytes produce failed package verification even when normalized totals can still

be aggregated. These checks establish recorded consistency, not election

certification or imported provenance.



Verification covers ten native district tests, eight exact native/WASM results

(direct, context and explicit crosswalk; both output formats; large counts),

nine matching rejections, overflow rejection and source tampering. Actual

browser Run, download, reload/Open and explicit rerun preserved the synthetic

total `9007199254740995`. `verify_aggregation_project.mjs` checks the downloaded

project against fresh execution and compares transcript bytes without parsing

large native integers into JavaScript numbers.



Preview: `/wasm-district-aggregation/`. Module: 4,050,090 bytes, SHA-256

`1621257cdf3203f03acf70f64c6948e40789c8be06651d9490485f057af50beb`.

Evidence: `scripts/web/test_district_aggregation.mjs`,

`scripts/web/verify_aggregation_project.mjs`,

`target/district-aggregation-preview-build.log`,

`target/wasm-district-aggregation-browser-project.bisect`,

`target/wasm-district-aggregation-browser-transcript.json`,

`target/wasm-district-aggregation-browser-verification.json`, and

`target/wasm-district-aggregation-browser.png`. Full engine and practitioner

coverage remains incomplete.



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


### Open Project security review (2026-10-04)

Follow-up hardening rejects `__proto__`, `constructor`, and `prototype` keys, including escaped spellings, during the JSON preflight before allocating the full tree. The post-parse defense remains. Regression tests instrument `JSON.parse` to prove early rejection for root and nested hostile keys. All four project, package, assignment-verification, and laboratory-project suites passed after this change.

Both the practitioner workbench and laboratory treat local projects as untrusted data. Opening a project does not execute its recorded operation or engine options. Imported markup remains text; package paths are validated and package bytes are not extracted to disk. Imported engine provenance and practitioner results remain unverified claims. Laboratory assignments and measurements are independently checked against the application's hashed prepared inputs before committing the imported experiment.

The shared JSON reader now checks nesting depth (64), total values (500,000), and duplicate keys, including escaped spellings, before `JSON.parse` allocates the complete object tree. Strict UTF-8 decoding and the 25 MiB limit apply before parsing. Post-parse schema and tree checks remain in place. Worker deadlines, cancellation, stale-import guards and atomic restoration preserve current work on rejection. Project files are unsigned and unencrypted; these controls validate and safely handle data, rather than authenticate the file's author or provide confidentiality.

Validation: `node scripts/web/test_project.mjs`, `node scripts/web/test_package_files.mjs`, `node scripts/web/test_assignment_verification.mjs`, and `node scripts/web/test_lab_project.mjs dist/wasm-standalone-chains RI,IA` passed. New tests prove that a 20,000-level tree, more than 500,000 values, and escaped duplicate keys fail before full-tree parsing; depth-boundary and ordinary JSON examples remain accepted. Integration tests cover malformed imports, prototype keys, invalid UTF-8, resource limits, inert markup, worker failure/deadline/cancellation, stale edits, independent metric verification and tamper rejection. Actual WASM laboratory Save/Open/map roundtrips continue to pass with no engine execution on Open. This review covers the local project import path, not an exhaustive security audit of the entire application.


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

### Demographic CSV and count-preserving projects

The laboratory now loads local CSV files through the shared native `bisect-data::demographics::import_demographic_csv` adapter in a dedicated WASM Worker. Select the CSV's state and population basis; the selected Census year applies and the filename becomes its user-supplied source label. For national batches, load each state's CSV separately. Existing fraction JSON files and projects remain supported.

Supported columns: `GEOID`/`geoid`; total population uses `total_pop` and `white_non_hispanic`; VAP uses `total_vap`/`vap` and `minority_vap`; explicit CVAP basis uses `cvap`/`total_vap`/`vap` and `minority_vap`. Supply exactly one matching total column. CVAP cannot be established from the column name alone when `total_vap`/`vap` is used; the basis is an explicit source claim. Short numeric GEOIDs are zero padded to eleven digits. Quoted fields, multiline extra fields, CRLF and UTF-8 BOM work. This strict adapter rejects duplicate headers/tracts, wrong-state GEOIDs, invalid UTF-8, missing/nonfinite/negative counts, negative zero and minority/white counts exceeding totals. It preserves the native formulas on valid rows; the historical CLI loaders still have their lenient error policies.

Each CSV is limited to 8 MiB and 100,000 records, with the existing 25 MiB combined input/project and JSON complexity limits still enforced. The Worker verifies the module SHA-256 before instantiation and is terminated on completion, rejection, cancellation or its ten-second deadline. Failed and stale reads cannot partially replace demographic inputs. Parsing an import does not submit an experiment.

Imported data uses `bisect-demographic-counts-v2`, with the existing metadata and fraction map plus a GEOID-keyed `counts` map containing `{total, minority}` in native `f64` semantics, bounded to the browser's safe magnitude. Total-population minority counts are derived as total minus non-Hispanic white. Zero total gives zero fraction; its descriptive district share is undefined when the district total is zero. Native and JS validation require complete count/fraction agreement. Save/Open retains these totals and minority counts; the original CSV byte stream is not embedded.

The results panel displays descriptive minority-count/total-count shares separately from unweighted tract means, with independent scrolling for long tables. This does not change the native fraction-based VRA ReCom search or VRASection's total-population proxy. VRASection still rejects VAP/CVAP basis. The existing engine demographic identity hashes operational fractions and scope, not count magnitudes or the source label. Unsigned imports remain source claims; coordinated changes can preserve internal consistency without authenticating their origin. True count-based VRA auditing and its workbench input flow remain separate unfinished work.

Verification: two native parser tests, 17 exact native/WASM comparisons, four real-RI engine/Save/Open/map cases with explicitly synthetic total-population/VAP/CVAP counts, and 33 malformed/count-tamper rejections passed. Actual Worker tests cover Rust conversion, count retention, module integrity failure, UTF-8/scope/duplicate rejection, pre-read limits/cancellation and cleanup. The previous 432-case ReCom and 576-case VRASection matrices still pass. Open Project, package-file and assignment-verification security suites passed again. Actual browser VAP CSV import/run, local Save, reload/Open and duplicate-CSV rejection were checked; fresh native/WASM replay matches the downloaded assignments and metrics. Browser RI had 250 tracts, two connected districts and 0.002460407935633846% population deviation using synthetic counts.

Evidence: `target/demographic-csv-verification.json`, `target/demographic-csv-parity.log`, `target/demographic-csv-browser-replay.log`, `target/wasm-demographic-csv-browser-project.bisect`, `target/wasm-demographic-csv-browser-verification.json`, `target/wasm-demographic-csv-browser.png`. Preview: `dist/wasm-demographic-laboratory/`; module 4,587,815 bytes, SHA-256 `bf64002ae72fd705bf26c86f5af900718be6819476571eba47e6c0248a60fcbc`. Full-toolkit completion is still unproved; this phase covers CSV import, preserved count inputs and descriptive reporting.

Full export completed successfully (`target/demographic-csv-full-build.log`): configured engine option matrices, laboratory integration, 387 one-district project/map cases with 1,161 tamper rejections, generic restoration, the CSV gate, twelve real-input chain cases and 24 VRA project cases with 48 tamper rejections passed. The dedicated CSV Worker gate was added during that run and passed separately on the final export (`target/demographic-csv-worker.log`); future builds include it directly. The final table-scroll layout was added after export, copied into both previews, syntax checked and visually verified. Exported JS/CSS match current source. The downloaded browser project also passed fresh replay against the final export. These gates cover their named cases, not the entire native capability inventory.


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