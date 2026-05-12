# BISECT Algorithm Atlas Rubric

This rubric scores an Algorithm Atlas page as a teaching artifact. It is adapted
from the ASPECT visual-scoring framework in `c:\src\aspect`, but tuned for
BISECT algorithm explainers rather than standalone visual works.

Each page is scored out of 60 points across six dimensions that spell BISECT.
A strong page should teach the algorithm, show the actual mechanism, connect it
to BISECT, and state the claim boundary without becoming a paper.

The current gold-standard page is `docs/algorithm-atlas/geosection.md`. A page
does not need to copy GeoSection's content, but it should match its teaching
standard: show the candidate objects, explain why those candidates are being
tried, show the decision rule, and show the downstream workload or artifact
created by the selected candidate.

## Role Lenses

Atlas pages should be reviewed with the repo's `.roles` lenses when the page's
claim touches that role. A page does not need every role, but it should face the
roles that can falsify its claims.

| Role | Use When The Page Claims... | Rubric Pressure |
|---|---|---|
| MERIDIAN | graph partitioning, bisection, compactness, METIS, solver mechanics | Check algorithm invariants and decision rules |
| DATUM | methodological evidence, comparison, "baseline", "benchmark", "supports" | Ask what would falsify the claim |
| SCALE | percentiles, means, ranks, convergence, benchmark summaries | Require scope, uncertainty, and sample definition |
| SURVEY | court, commission, practitioner, CLI workflow, adoption | Require operational takeaway and audience fit |
| CONTOUR | census graph, populations, geography, unit order, demographics | Require input provenance and unit identity |
| BOUNDARY | validity, legal profile, VRA, equal population, contiguity | Prevent algorithmic validity from implying legal sufficiency |
| COVENANT | audit, certificate, manifest, reproduction, evidence | Require chain-of-custody and hash-bound artifacts |
| LEDGER | RPLAN, RCTX, GeoJSON, NDJSON, external tool compatibility | Require schema/version/interchange clarity |
| BENCHMARK | tests, fixtures, packages, verification | Require evidence that wrong behavior would be caught |
| TRENCH | status paths, failure modes, repair, infeasibility | Require named failure modes and structural prevention |

## Score Bands

| Score | Meaning |
|---|---|
| 54-60 | Excellent atlas page. Accurate, visual, memorable, audit-aware, and reconstructable from the figure plus prose. |
| 45-53 | Good page. Useful, but one dimension is underdeveloped or the main visual still needs stronger candidate/decision/consequence detail. |
| 36-44 | Serviceable. Catalog-level or uneven; needs revision before relying on it for teaching. |
| 24-35 | Thin. Names the method but does not really explain it. |
| 0-23 | Failing. Misleading, visually absent, or disconnected from BISECT. |

## B - Behavioral Mechanics

**Question:** Does the page explain what the algorithm actually does, step by
step, including the choices it makes?

| Points | Anchor |
|---|---|
| 9-10 | The reader can follow the algorithm from input to output, including why candidate choices are generated, how the decision rule selects among them, what workload/artifact follows, tie-breaking, iteration/recursion, and failure paths. |
| 6-8 | Main steps are correct, but one important mechanism is only named rather than explained. |
| 3-5 | The page gives a high-level summary but not enough mechanics to reproduce the idea. |
| 0-2 | The algorithm is described vaguely or incorrectly. |

Checklist:

- Names the input object: graph, plan, particles, columns, regions, seeds, etc.
- Names the required invariants: population tolerance, connectedness, coverage,
  capacity, objective, transition kernel, status taxonomy, or hash identity.
- States the data provenance or unit-order assumption when graph/population
  inputs matter.
- Shows the main transition: cut, merge, assign, improve, price, resample, verify.
- Explains why the candidate set exists: root allocations, seeds, columns,
  moves, particles, frontier points, or verifier checks.
- For recursive algorithms, shows the workload created by each candidate, not
  only the first local action.
- Explains what is optimized, sampled, repaired, or certified.
- Includes failure/status paths where they are part of the algorithm, including
  what structurally prevents invalid output from being treated as final output.

## I - Integration With BISECT

**Question:** Does the page explain how BISECT uses this method, not just what
the generic algorithm is?

| Points | Anchor |
|---|---|
| 9-10 | Clearly states the BISECT role, CLI/config surface, crate/module boundary, sidecars, and relationship to RPLAN/RCTX/audit flow. |
| 6-8 | BISECT role is present, but crate/CLI/artifact connection is incomplete. |
| 3-5 | Generic algorithm explanation with only a passing BISECT mention. |
| 0-2 | Could be copied into any project; BISECT-specific use is absent. |

Checklist:

- Says whether the method chooses a bisection cut, seeds, assignments, merges,
  samples, improves, optimizes, or verifies.
- Says what BISECT does with the selected candidate after the decision:
  recurse, package, audit, score, emit a report, resample, or export.
- Names the relevant crate/module and user-facing surface where known.
- States whether it emits final plans, summaries, witnesses, reports, or only
  exploratory output.
- Connects final-plan paths to RPLAN/RCTX/certificate/manifest.
- States what a practitioner can do after reading the page: run a CLI command,
  inspect a package, understand a report, or choose a safer method.

## S - Spatial And Visual Specificity

**Question:** Do the pictures show the real structure of the method, rather than
decorative boxes?

| Points | Anchor |
|---|---|
| 9-10 | Diagrams make the core mechanism visible with concrete domain objects, countable candidate examples, the decision rule, and the downstream consequence: area/block division, graph cuts, seed growth, capacity bars, merge lineage, recursive workload, particle genealogy, solver columns, or certificate binding. |
| 6-8 | Diagrams are relevant but too schematic; one needed concrete visual is missing, or candidates are shown without enough purpose/decision/consequence detail. |
| 3-5 | Mostly flow boxes with little domain-specific visual encoding. |
| 0-2 | Visuals are absent, misleading, or decorative. |

Checklist:

- The primary figure follows the teaching path: candidate set -> decision rule
  -> selected consequence.
- The visual says why candidates are being tried before asking the reader to
  compare scores.
- If candidates are spatial, they are shown with countable 2D block miniatures,
  maps, grids, regions, or graph structure instead of generic boxes.
- If the method divides something, the split is shown, not merely labeled.
- If a ratio is shown, the ratio is visually proportional or countable.
- If the method recurses, the figure shows the recursive workload or tree
  created by the first decision.
- If the method divides area, the page shows area/region division.
- If the method cuts a graph, the page shows candidate cuts and selected cuts.
- If the method chooses seeds, the page shows seed placement and growth.
- If the method enforces capacity, the page shows population/capacity pressure.
- If the method is exact optimization, the page shows model/report/bound status
  or columns/master structure.
- If the method samples, the page shows chain/particle/frontier behavior.
- If the method audits, the page shows hashes, manifests, certificates, and
  failure reasons.
- Labels may name objects, but geometry/structure must carry the meaning first.

## E - Explanatory Story

**Question:** Does the page guide the reader through a memorable learning path?

| Points | Anchor |
|---|---|
| 9-10 | The page has a clear arc: mental model, BISECT role, visual walkthrough, mechanics, evidence, claim boundary, and example; the reader can reconstruct the algorithm's purpose and decision path from the main figure. |
| 6-8 | The sequence is readable but a section feels abrupt or under-motivated. |
| 3-5 | Sections exist but read like notes, not a guided explanation. |
| 0-2 | The page is a list of facts without a teaching path. |

Checklist:

- Opens with a plain mental model.
- Explains why the algorithm exists or what problem it solves.
- Uses captions/prose to narrate each diagram.
- Each major diagram has a reading rule or caption that tells the reader what
  to notice.
- Includes a tiny example when the algorithm is abstract.
- Ends by clarifying when to use it and what not to claim.
- Names the intended reader context when it matters: developer, researcher,
  reviewer, court/commission practitioner, or general technical reader.

## C - Claim Boundary And Correctness

**Question:** Is the page accurate about what the method proves, samples,
optimizes, or merely demonstrates?

| Points | Anchor |
|---|---|
| 9-10 | Claims are tightly scoped, status words are precise, limitations are explicit, and visuals do not imply stronger guarantees than the algorithm/provenance supports. |
| 6-8 | Mostly accurate, with one claim that needs sharper qualification. |
| 3-5 | Contains overbroad language such as optimal, representative, legal, or exact without enough conditions. |
| 0-2 | Misstates the algorithm, evidence level, or legal/statistical meaning. |

Checklist:

- Distinguishes valid, repaired, infeasible, invalid, solved, bounded,
  formulation-only, exploratory, and benchmark-tier outputs.
- Avoids legal-sufficiency claims unless the artifact actually supports them.
- Qualifies sampling and convergence claims with sample size, scope, diagnostic,
  uncertainty, or "not yet established" language as appropriate.
- States what evidence would falsify or weaken the page's claim.
- Ensures the visual encoding cannot be mistaken for a proof of optimality,
  legal sufficiency, convergence, or representativeness unless that proof is
  actually present.
- Separates "deterministic construction path" from "optimality proof."
- Separates "audit certificate verifies declared context/profile" from "the
  plan is legally sufficient in the world."
- Does not imply constitutional/VRA sufficiency from algorithmic validity,
  compactness, or audit acceptance alone.

## T - Traceability And Evidence

**Question:** Can the reader find the implementation, paper, package, and
verification evidence from the page?

| Points | Anchor |
|---|---|
| 9-10 | References point to crates/modules, CLI/config, paper/spec, examples/packages, summaries/reports, verifier path where applicable, and the sidecar fields needed to reproduce the visual decision. |
| 6-8 | Most references exist, but one evidence tier is missing. |
| 3-5 | References are sparse or only point to a concept page. |
| 0-2 | No actionable repo references. |

Checklist:

- Links or names the relevant crate/module.
- Names the CLI/config surface where known.
- Points to the paper/spec or concept guide.
- Points to golden, method-produced, benchmark, or reference packages where
  available.
- Names verifier commands or audit reports for final-plan paths.
- Points to tests or fixtures when the page describes behavior that should be
  mechanically checked.
- Names schemas, format versions, or sidecar file types when discussing RPLAN,
  RCTX, GeoJSON, NDJSON, manifests, or external-tool compatibility.
- Names the output fields that correspond to the page's visual decisions:
  selected ratio, seed, column, move, particle ancestry, frontier index, status,
  hash, or certificate reason.
- For audit-facing pages, identifies the hash-bound artifacts and any external
  tool or binary provenance that could affect chain of custody.

## Required Page Skeleton

Every mature atlas page should contain:

1. `Mental Model`
2. `How BISECT Uses It`
3. At least one gold-standard teaching figure plus any supporting diagrams
4. `Step-By-Step Mechanics`
5. `What The Certificate/Output Needs To Explain`
6. `Claim Boundary`
7. `References In This Repo`

Where applicable, mature pages should also include `Failure Modes` or fold those
failure modes explicitly into `Claim Boundary`.

For algorithms where one teaching figure is not enough, add more. The visual
standard is concrete mechanism first: if the algorithm manipulates regions,
seeds, columns, particles, trees, capacities, or certificates, those objects
should be visible on the page.

## Gold-Standard Visual Requirements

An atlas page cannot score excellent if its main visual is only decorative or
catalog-like. The primary visual should satisfy these checks:

1. **Show the candidates.** Display the splits, seeds, columns, moves,
   particles, frontier points, verifier checks, or other objects the algorithm
   compares.
2. **Show why they exist.** State the downstream decision the candidates feed:
   root allocation, selected plan, exact cover, accepted move, resampling map,
   audit result, or package export.
3. **Show the decision rule.** Include the score, threshold, feasibility window,
   objective, status gate, or verifier predicate.
4. **Show the consequence.** Make visible what happens after selection:
   recursive workload, emitted report, descendant plan, selected package,
   rejected candidate, or audit failure reason.
5. **Use appropriate geometry.** For schematic spatial algorithms, prefer
   countable 2D block arrays, grids, maps, or graph structures over generic
   boxes or blobs.
6. **Make splits visible.** A ratio such as `1:13` must be drawn as one part
   against thirteen parts, and recursive splits must show the workload they
   create.
7. **Include a reading rule.** The figure or caption should say how to read the
   visual in one sentence.

## Atlas Review Form

```text
Page:
Reviewer:
Date:

B Behavioral Mechanics: __/10
I Integration With BISECT: __/10
S Spatial And Visual Specificity: __/10
E Explanatory Story: __/10
C Claim Boundary And Correctness: __/10
T Traceability And Evidence: __/10

Total: __/60
Band:

Strongest element:
Weakest element:
Required upgrades:
Optional upgrades:
Roles applied:
Falsifying evidence to check:
```
