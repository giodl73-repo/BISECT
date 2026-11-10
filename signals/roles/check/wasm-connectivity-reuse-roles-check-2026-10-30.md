---
skill: roles-check
topic: wasm-connectivity-reuse
date: 2026-10-30
roles_used: [meridian, benchmark, covenant, scale, trench, datum, survey]
p1_count: 1
p2_count: 10
p3_count: 10
verdict: NEEDS-WORK
---

# Connectivity reuse adoption review

This is a single-agent review through seven installed role lenses, not seven
independent signoffs. The artifact is the isolated Rust caller optimization and
its comparison harness in `target/connectivity-caller-experiment-20261030`.
Product Rust source and normal native/WASM artifacts remain unchanged. The full
WP-01–WP-10 objective is unchanged.

## Artifact and evidence

The new prepared checker owns a validated, weak-undirected topology snapshot and
query scratch. Only the scalar population boundary-repair loop uses it, lazily,
within one call. Singleton bypasses, the multiconstraint early return, candidate
order, score, global-ID tie break, seed and 200-iteration limit are unchanged.
The original one-shot public function remains unchanged.

The predecessor isolated kernel experiment passed 593,110 independent-reference
and old-function checks in each of native and WASM. These include directed and
undirected graphs, duplicate/reversed subsets and invalid/empty query histories.
The original native CA VRA diagnostic completed with a balance miss; its full
reply and request are retained. Its process was sampled, so timing is diagnostic.

The first caller build failed because the copied tree omitted the shared
`include!` build-provenance source. `attempt-1` preserves that failure. The source
was copied unchanged. The retry's native build passed; WASM and the queued full
comparison are still pending at review time. The copy now captures 891 baseline
files and a separate scan finds 121 literal Rust include references, none missing.
Computed include paths are outside that scan. No whole-builder or C-profile
acceptance follows from these facts.

## Role selection

MERIDIAN checks graph and decision equivalence. BENCHMARK checks falsifiable
regressions and target coverage. COVENANT checks retained source/binary/reply
identities. SCALE checks performance and population-balance claims. TRENCH checks
new cache/lifetime and copy failure modes. DATUM checks baseline construction and
reproducibility. SURVEY checks practical browser/package adoption and recovery.
The change introduces no legal policy, demographic source, file format, ballot
record or certification-status behavior requiring the other specialized lenses.

## MERIDIAN

| # | Finding | Severity | Section | Recommendation |
|---|---------|----------|---------|----------------|
| M1 | Kernel equality does not prove unchanged selected cuts after caller integration. | P2 | repair loop / compare.mjs | Require exact whole replies for the synthetic matrix and retained CA case before adopting the two Rust edits. |
| M2 | Preparation preserves the existing weak-undirected normalizer and is local to immutable adjacency. | P3 | PreparedSubsetConnectivity | Keep ownership local; do not reuse across changed graphs or replace the normalizer with directed traversal. |
| M3 | CA native completion reports a balance miss and unproved optimality. | P3 | retained CA reply | Preserve these fields exactly; connectivity parity does not prove balance or best-cut optimality. |

## BENCHMARK

| # | Finding | Severity | Section | Recommendation |
|---|---------|----------|---------|----------------|
| B1 | The candidate has no completed full-engine native/WASM comparison yet. | P1 | comparison-status.json | Block adoption on a terminal passing receipt, all 96 cases, successful Area and VRA examples, and exact CA native/WASM replies. |
| B2 | The exhaustive prepared-checker test currently lives only in a target experiment. | P2 | kernel harness | Add a maintained independent-reference regression when adopting the API and register the affected full-engine checks. |
| B3 | The candidate has no fresh C-enabled module or final-package browser acceptance. | P2 | build/package matrix | Run the applicable registered standard/C builders and actual packaged browser checks; Node WASM is insufficient for package acceptance. |

## COVENANT

| # | Finding | Severity | Section | Recommendation |
|---|---------|----------|---------|----------------|
| C1 | Working evidence paths remain mutable and are not an immutable acceptance snapshot. | P2 | experiment evidence | Freeze inputs, outputs, source, modules and command identities and verify readback before ledger acceptance. |
| C2 | Original reference executables/modules and the CA request/reply were copied and hash-pinned. | P3 | reference-inputs.json | Preserve those exact copies and compare full replies; do not substitute a freshly rebuilt reference for the original. |
| C3 | The initial build failure is preserved rather than rewritten as a successful retry. | P3 | attempt-1 | Retain separate attempt and terminal identities in later closure. |

## SCALE

| # | Finding | Severity | Section | Recommendation |
|---|---------|----------|---------|----------------|
| S1 | Kernel timings do not establish whole-engine or typical-browser speedup. | P2 | performance claims | Make no end-to-end speedup claim from the small grid or concurrent sampled runs. |
| S2 | Original CA diagnostic timing includes process sampling and concurrent work. | P3 | native diagnostic | Describe elapsed values as local observations, not clean paired benchmark estimates. |
| S3 | The proposed 96-case matrix and CA replay cover selected structures and one census vintage. | P3 | comparison scope | Do not infer the full all-state/method/vintage/host matrix or WP-08 closure. |

## TRENCH

| # | Finding | Severity | Section | Recommendation |
|---|---------|----------|---------|----------------|
| T1 | Rebuilding immutable topology for every candidate amplifies graph-wide work inside repair/search loops. | P2 | original connectivity caller | Record AP-12 as open until product integration and maintained acceptance exist; do not mark solved from a prototype. |
| T2 | Query state can leak after an early invalid or empty query unless reset before reuse. | P3 | scratch lifecycle | Preserve reset-before-nonempty-query behavior and independent mixed-history regressions; empty queries return true without consuming scratch. |
| T3 | Copying only include_str!/include_bytes! omitted an include! dependency and wasted a build. | P2 | isolated copy bootstrap | Check all literal include kinds before future copied-workspace builds; retain the 121-reference scan and its computed-path limitation. |

## DATUM

| # | Finding | Severity | Section | Recommendation |
|---|---------|----------|---------|----------------|
| D1 | A commit label alone cannot identify this dirty copied experiment. | P2 | build provenance | Retain the source byte roster, lockfile, vendored inputs, actual compiler identity and invocation with the artifact hashes. |
| D2 | The proposed mechanism removes topology preparation, not candidates, scoring or search effort. | P3 | isolated diff | Keep the adoption diff restricted to topology/scratch reuse; any cycle skipping or budget change needs separate proof. |
| D3 | A completed execution and an exact reply can still contain an infeasible result. | P3 | evidence narrative | Retain failure and feasibility fields and their separate interpretation in documentation and ledger evidence. |

## SURVEY

| # | Finding | Severity | Section | Recommendation |
|---|---------|----------|---------|----------------|
| U1 | Replacing a binary or source captured by live tests would invalidate their evidence. | P2 | running original suite | Preserve live artifacts. If superseded after successful comparisons, explicitly stop only owned jobs, retain an interrupted receipt, then adopt source and run fresh gates. |
| U2 | Source adoption alone would not update the deployed site or demonstrate local project/worker parity. | P2 | release scope | Complete fresh package/browser/recovery checks before publication; keep the completed Windows folder case closed. |
| U3 | Separate compiler caches consume the headroom needed by the queued package preflight. | P3 | disk capacity | After all experiment processes terminate, preserve evidence and runtime artifacts and reclaim only owned rebuildable cache if needed. |

## Synthesis and amendments

Roles reviewed: 7. Findings: 1 P1, 10 P2, 10 P3. Verdict: **NEEDS-WORK for
adoption**, with the isolated experiment appropriate to continue. These are
findings and acceptance conditions, not newly established product failures.
The most important condition is complete caller-level output equivalence.
MERIDIAN and BENCHMARK agree that kernel evidence alone cannot supply it;
COVENANT and DATUM agree that a dirty commit label cannot identify the experiment.

1. Finish the queued 96-case/CA whole-reply comparison and retain terminal/raw
   evidence. Preserve the passing native build and failed initial attempt.
2. Before adoption, add maintained independent connectivity tests, AP-12
   traceability and the include-dependency preflight. Preserve original suite
   evidence if the slow diagnostic is explicitly superseded; never call it passed.
3. Run fresh registered standard/C and final-package browser checks, freeze a
   readback-verifiable evidence snapshot, then update only scoped current ledger
   records. No full parent, publication readiness or typical-host speed claim is
   accepted by this review.

## Cache-condition follow-up

U3's conservative wait for every experiment process was refined to the actual
compiler/runtime boundary. Both isolated builds had terminal passing receipts
and no cargo/rustc process was live. Only owned, single-link `.rlib`/`.rmeta`
compiler files were removed; all runtime executable/module/PDB/DLL and evidence
files remain. The active comparison's recorded native/WASM hashes matched both
before and after. The eviction receipt retains each removed cache hash and actual
free-space change. Reclamation restores the package minimum without stopping or
changing the runtime test. This addresses current headroom, not future capacity
or the still-pending comparison/package acceptance conditions.

## Source-adoption follow-up

The completed caller experiment now supplies all 104 small cases and exact CA
native/WASM replies against the original retained native reference. CA's balance
miss and unproved optimality are unchanged. All original engine jobs terminated
before the two tested Rust files were adopted. Three maintained independent
connectivity tests pass in the repository and are registered in standard, lab
and C legacy paths. This addresses M1/B1 for the stated comparison scope and B2
for maintained test integration. It does not close C1's immutable release
evidence or B3/U2's fresh C/final-package requirements. The R6 whole-build
pipeline now freezes 3,232 inputs and is running after focused test corrections;
its first premature preflight is separately retained. Source adoption may
proceed with those release conditions; the original review was not a full-parity
or publication approval.

The new C legacy Cargo regression was moved below the existing capacity and
supplied-record preflight. R6 was deliberately interrupted before editing its
frozen source; its source-stability receipt passed. R7 captures and verifies
inputs before launch and now runs the unchanged full 50-state standard/C
commands. The 592-case/18-project corrected-test diagnostics are preserved and
reused because their bytes are unchanged. This fixes registration order and
retains the review's pending fresh-package/closure conditions.
