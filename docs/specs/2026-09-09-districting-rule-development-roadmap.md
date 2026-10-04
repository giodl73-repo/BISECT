# Districting rule development roadmap

Status: active research program; implementation proceeds by evidence gates.
Date: 2026-09-09
Owner: project maintainer; implementation and local review: Codex.

## Ambition and principles

Develop a publicly chosen districting rule with predictable, explainable,
independently checkable execution. The Huntington–Hill analogy motivates an
explicit rule and its justification; no theorem currently selects BISECT as
the uniquely appropriate districting rule.

Proposed commitments: declared population requirements; connected districts;
respect for subdivisions; limited geographic boundary complexity; a fixed
recursive schedule; precommitted selection and failure rules; reproducible
decisions. Research may adjust these commitments. Official execution must use
a frozen, versioned profile with a declared authority.

## Evidence baseline

The 2020 NRS v0.3 geographic baseline has 1,808 split counties and 19,789 split
tracts on the retained land-containing block universe. Its national comparison
and three-cycle replay passed. Those results do not evaluate the separate
`configs/official_proposal.yml`, which uses apportion-regions, county weights
with additive alpha 2, and convergence search. T.3 manuscript results are
historical claims until their source artifacts are independently checked.

The certified national frontier remains zero weighted-boundary/canonical
proofs over 1,155 recursive nodes. Bounded certificates and a staged ceremony
exist. The ceremony currently requires a complete tree before genesis.

## Work packages and acceptance gates

| Package | Deliverable | Acceptance gate |
|---|---|---|
| W0: evidence and influence audit | Reproducible profile inventory and candidate-count diagnostic | Identify which settings reach candidate scoring; distinguish candidates before population filtering from those surviving boundary scoring |
| W1: county preservation | Controlled additive-alpha sweep; direct split and fragmentation objective designs | Paired results on common inputs, fixed population rule, schedule, seeds, and search budget; preserve all failures |
| W2: population objective | Compare strict lexicographic balance with a declared feasible population band | Validate final leaf deviations and child completion constraints; version changed objectives explicitly |
| W3: search and completion | Stronger candidates under the same objective; necessary feasibility checks and limited lookahead | Separate search gains from rule changes; report incomplete trees and timeout bounds |
| W4: robustness and principles | GEOID relabeling, population perturbation, geometry precision, bridge and resolution studies | Classify expected invariances; disclose violations and unresolved normative choices |
| W5: proof scale | Exact proofs or certified bounds for a preselected multi-district State | Checker acceptance, resource measurements, explicit feasible/optimal/incomplete status |
| W6: public commitment | Pre-run profile/input commitment, witness adapter, challenge and restart procedure | External signature/time checks; reject equivocation, early release, and invalid transitions |
| W7: demonstration and confirmation | Complete State demonstration plus reserved cohort | Reproduce with an independent operator; publish favorable and unfavorable outcomes |

W0 starts immediately. W1 follows its influence gate. W2 and W3 address any
candidate bottleneck before escalating a national sweep. W4–W7 remain planned;
their completion is not implied by this roadmap or the local role review.

## W0 implementation contract

Read existing governed profiles and the 385-node 2020 DFS census. Produce a
compact JSON report with source and analyzer hashes, profile differences,
pre-boundary minimum-deviation candidate counts, post-boundary physical-cut
counts, and fallback counts. Validate required columns, unique State/path
keys, expected State/node coverage, preservation flags, and positive counts.

A single post-boundary physical partition does not establish insensitivity to
new weights: other minimum-deviation candidates may have been eliminated by
the old boundary weights. The existing census cannot identify all distinct
physical partitions before boundary scoring. Record that missing measurement
as the next instrumentation task, without manufacturing a county-sweep result.

## W1 prospective experiment

Development cohort: IA, NC, TX, and RI (county-oriented geography, irregular
geography, large multi-district scale, and a two-district proof anchor).
Confirmation cohort: WI, PA, CA, and NH. This is a purposive cohort, not a
probability sample. Existing maps have already been inspected; no claim of
blinding to historical outcomes is made. Freeze new-run settings and hashes
before execution and do not tune to confirmation results.

Use additive alpha in {0, 0.5, 1, 2, 4, 8}. Internal county edge multiplier is
1 + alpha; cross-county edges keep geographic weight. Record both quantities
to avoid confusion with T.3's multiplicative alpha notation. Freeze integer
rounding, overflow checks, bridge handling, and base geographic weights.

First compare alpha values under one candidate engine and unchanged objective.
Any expanded engine is a separate paired arm with the same inputs and budget.
Population-band and direct-fragmentation objectives are separately versioned
arms, never silently mixed into the weight sweep. Use research copies, never
overwrite NRS v0.3 or the official proposal profile.

Before W1 execution, freeze an executable manifest with exact input hashes,
seed schedule, engine hashes, resource limits and output locations. Initial
resource proposal: one worker, 30 minutes and 8 GiB per State/alpha/engine cell;
preflight capacity, retain timeout status, and revise budgets only in a new
experiment version. W0 needs only repository artifacts and standard Python.

Metrics: count counties touching more than one district; extra fragments
sum(max(0, districts_touching_county - 1)); maximum fragments per county;
county population affected; analogous tract metrics; raw geographic cut;
weighted cut; final maximum and total population deviations; connectivity;
compactness; runtime; proof coverage and gaps. Define zero-population and
water-only inclusion explicitly and retain the existing common-block universe
for comparisons. County equivalents must retain Census identifiers.

Oversized counties may require division. Any proposed unavoidable-fragment
bound must state assumptions and be verified separately. County preservation
does not substitute for municipal, neighborhood, or community evaluation.

Publish paired per-State outcomes and distributions, not only pooled means.
Do not assume monotonic improvement from a heuristic. Select candidate rules
from the measured tradeoffs only after publishing all cells; justify selection
openly and confirm on the reserved cohort. Deterministic cohort differences
need no invented sampling significance; broader inference needs its own design.

## Governance and claim boundaries

Population tolerances in software are research parameters. No generic 0.5%
setting is treated as a universal constitutional safe harbor. A jurisdiction
and chamber-specific legal profile requires current primary-source review
before any compliance or adoption claim. Local role files are review lenses,
not legal authorities. This roadmap does not decide current law.

Political and demographic outcomes require separately governed evaluation
inputs. Prohibited optimization inputs remain prohibited; absence of such inputs
does not establish neutral effects. Human challenge authority and correction
criteria must be specified before an official ceremony. Hashes alone cannot
prove honest publication time or legal validity.

## Review and execution record

Review: `signals/roles/check/districting-rule-roadmap-roles-check-2026-09-09.md`.
First output: `docs/experiments/county-preservation-readiness-2020/analysis.json`.
Subsequent work is gated by observed evidence, not by a promise of improvement.

## Validation sequence added 2026-10-02

Preserve the immediate-cut rule before another national sweep. Bounded
regressions now demonstrate that the current locally optimal recursive rule
can fail to complete despite an alternative completable root, and that perfect
root balance does not establish final-leaf balance. The witnesses are in
`crates/bisect-ilp/src/certified_tree.rs`; run
`cargo test -p bisect-ilp --locked --offline certified_tree::tests`.
They describe synthetic instances and do not invalidate the separately
verified operational State assignments.

1. **Immediate-cut contract and failure semantics (W2/W3):** keep population,
   boundary and canonical ranking local to the current region. Descendant
   outcomes must not change cut eligibility or ranking under this rule.
   Specify how execution reports an infeasible descendant and separately checks
   final-leaf population requirements. Acceptance: the star witness returns
   the declared descendant failure without replacing its optimal root; the
   path witness is not presented as passing an undeclared final-leaf band.
   Consolidate existing cut experiments under identical instances and objective
   definitions, reporting incumbent and unrestricted lower bounds separately
   from restricted-neighborhood optima. A completion-aware oracle would be a
   distinct rule, not an improvement to execution of the current one.
2. **Rule/profile binding (W6):** define a machine-readable certified profile,
   validate its semantics against the instance, and include its version in
   proof/model bindings. Hashing an arbitrary profile file is insufficient.
   Acceptance: changed population field, weights, schedule or unsupported
   policy cannot produce a successful conformance claim merely by rebuilding
   envelope hashes. Keep external witness verification as a separate gate.
3. **Search versus policy (W1/W3):** freeze a nonadjacent exchange or connected
   group move experiment using existing RI starts and population bands. Test
   direct county fragmentation in a separate objective arm. Acceptance:
   independent final-label checks, exact score recomputation, retained failed
   cells, final-leaf checks for every full-tree experiment, and paired results.
4. **Proof milestone (W5):** preselect one nontrivial two-district State for
   full boundary and canonical proofs. Publish incumbent, lower bound, gap,
   runtime, memory, proof bytes and checking time, including timeouts. Restricted
   disagreement-band optima remain conditional; fixed cores must be justified
   before claiming unrestricted optimality.
5. **Confirmation and pitch (W7):** have a non-author reproduce the selected
   complete-plan demonstration. Publish one table distinguishing NRS v0.3,
   certified-standard-bisect and the ApportionRegions official proposal, with
   each profile's own evidence. Update adoption wording only after its gates
   pass; national operational evidence does not validate a different profile.

The first two items address correctness of the proposed execution claim;
the third tests map quality; the fourth establishes certification scale;
the fifth supplies external evidence. None is completed by adding this plan.
