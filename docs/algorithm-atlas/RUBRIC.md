# BISECT Algorithm Atlas Rubric

This rubric scores an Algorithm Atlas page as a teaching artifact. It is adapted
from the ASPECT visual-scoring framework in `c:\src\aspect`, but tuned for
BISECT algorithm explainers rather than standalone visual works.

Each page is scored out of 60 points across six dimensions that spell BISECT.
A strong page should teach the algorithm, show the actual mechanism, connect it
to BISECT, and state the claim boundary without becoming a paper.

## Score Bands

| Score | Meaning |
|---|---|
| 54-60 | Excellent atlas page. Accurate, visual, memorable, and audit-aware. |
| 45-53 | Good page. Useful, but one dimension is underdeveloped. |
| 36-44 | Serviceable. Catalog-level or uneven; needs revision before relying on it for teaching. |
| 24-35 | Thin. Names the method but does not really explain it. |
| 0-23 | Failing. Misleading, visually absent, or disconnected from BISECT. |

## B - Behavioral Mechanics

**Question:** Does the page explain what the algorithm actually does, step by
step, including the choices it makes?

| Points | Anchor |
|---|---|
| 9-10 | The reader can follow the algorithm from input to output, including decision points, tie-breaking, iteration/recursion, and failure paths. |
| 6-8 | Main steps are correct, but one important mechanism is only named rather than explained. |
| 3-5 | The page gives a high-level summary but not enough mechanics to reproduce the idea. |
| 0-2 | The algorithm is described vaguely or incorrectly. |

Checklist:

- Names the input object: graph, plan, particles, columns, regions, seeds, etc.
- Shows the main transition: cut, merge, assign, improve, price, resample, verify.
- Explains what is optimized, sampled, repaired, or certified.
- Includes failure/status paths where they are part of the algorithm.

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
- Names the relevant crate/module and user-facing surface where known.
- States whether it emits final plans, summaries, witnesses, reports, or only
  exploratory output.
- Connects final-plan paths to RPLAN/RCTX/certificate/manifest.

## S - Spatial And Visual Specificity

**Question:** Do the pictures show the real structure of the method, rather than
decorative boxes?

| Points | Anchor |
|---|---|
| 9-10 | Diagrams make the core mechanism visible: area division, graph cuts, seed growth, capacity bars, merge lineage, tree cuts, particle genealogy, solver columns, or certificate binding. |
| 6-8 | Diagrams are relevant but too schematic; one needed concrete visual is missing. |
| 3-5 | Mostly flow boxes with little domain-specific visual encoding. |
| 0-2 | Visuals are absent, misleading, or decorative. |

Checklist:

- If the method divides area, the page shows area/region division.
- If the method cuts a graph, the page shows candidate cuts and selected cuts.
- If the method chooses seeds, the page shows seed placement and growth.
- If the method enforces capacity, the page shows population/capacity pressure.
- If the method is exact optimization, the page shows model/report/bound status
  or columns/master structure.
- If the method samples, the page shows chain/particle/frontier behavior.
- If the method audits, the page shows hashes, manifests, certificates, and
  failure reasons.

## E - Explanatory Story

**Question:** Does the page guide the reader through a memorable learning path?

| Points | Anchor |
|---|---|
| 9-10 | The page has a clear arc: mental model, BISECT role, visual walkthrough, mechanics, evidence, claim boundary, and example. |
| 6-8 | The sequence is readable but a section feels abrupt or under-motivated. |
| 3-5 | Sections exist but read like notes, not a guided explanation. |
| 0-2 | The page is a list of facts without a teaching path. |

Checklist:

- Opens with a plain mental model.
- Explains why the algorithm exists or what problem it solves.
- Uses captions/prose to narrate each diagram.
- Includes a tiny example when the algorithm is abstract.
- Ends by clarifying when to use it and what not to claim.

## C - Claim Boundary And Correctness

**Question:** Is the page accurate about what the method proves, samples,
optimizes, or merely demonstrates?

| Points | Anchor |
|---|---|
| 9-10 | Claims are tightly scoped, status words are precise, and limitations are explicit without burying the algorithm. |
| 6-8 | Mostly accurate, with one claim that needs sharper qualification. |
| 3-5 | Contains overbroad language such as optimal, representative, legal, or exact without enough conditions. |
| 0-2 | Misstates the algorithm, evidence level, or legal/statistical meaning. |

Checklist:

- Distinguishes valid, repaired, infeasible, invalid, solved, bounded,
  formulation-only, exploratory, and benchmark-tier outputs.
- Avoids legal-sufficiency claims unless the artifact actually supports them.
- Qualifies sampling and convergence claims.
- Separates "deterministic construction path" from "optimality proof."
- Separates "audit certificate verifies declared context/profile" from "the
  plan is legally sufficient in the world."

## T - Traceability And Evidence

**Question:** Can the reader find the implementation, paper, package, and
verification evidence from the page?

| Points | Anchor |
|---|---|
| 9-10 | References point to crates/modules, CLI/config, paper/spec, examples/packages, summaries/reports, and verifier path where applicable. |
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

## Required Page Skeleton

Every mature atlas page should contain:

1. `Mental Model`
2. `How BISECT Uses It`
3. At least two diagrams with explanatory prose
4. `Step-By-Step Mechanics`
5. `What The Certificate/Output Needs To Explain`
6. `Claim Boundary`
7. `References In This Repo`

For algorithms where two diagrams are not enough, add more. The visual standard
is concrete mechanism first: if the algorithm manipulates regions, seeds,
columns, particles, trees, capacities, or certificates, those objects should be
visible on the page.

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
```
