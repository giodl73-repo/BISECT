# BISECT Atlas Rubric R1 Role Review

**Artifact reviewed:** `docs/algorithm-atlas/RUBRIC.md`
**Review date:** 2026-05-11
**Roles consulted:** MERIDIAN, DATUM, SCALE, SURVEY, CONTOUR, BOUNDARY,
COVENANT, LEDGER, BENCHMARK, TRENCH

## Summary

The BISECT rubric is a strong foundation because it is memorable, project-native,
and directly addresses the original failure mode: atlas pages that summarize
complex algorithms without showing their actual mechanism. The six dimensions
cover the main needs for teaching pages.

The main gap is that the first draft scores pages as finished prose, but it does
not yet tell reviewers which role-lens should be applied for each kind of
algorithm. Exact solvers, sampling methods, legal/audit claims, and data-heavy
construction methods fail in different ways. The rubric should add role review
guidance so a page cannot score highly by being attractive while evading its
hardest claim boundary.

## Role Findings

### MERIDIAN

**Finding:** Behavioral Mechanics is necessary but should require algorithmic
invariants, not just steps.

Examples:

- A bisection page should identify the graph object, balance target, and cut
  selection rule.
- A solver page should identify formulation variables, constraints, objective,
  and status semantics.
- A sampling page should identify transition kernel, seed derivation, acceptance
  or resampling rule, and diagnostics.

**Rubric action:** Strengthen Behavioral Mechanics to name invariants and
decision rules.

### DATUM

**Finding:** Claim Boundary is good, but the rubric should ask "what would
falsify this page's claim?"

If a page says "deterministic," it should name the seed/tie rule or package
evidence. If it says "benchmark-tier," it should point to benchmark artifacts.

**Rubric action:** Add falsifiability/evidence-fit prompts to Claim Boundary and
Traceability.

### SCALE

**Finding:** Sampling, percentile, benchmark, and comparison pages need explicit
uncertainty language. The rubric currently says "qualifies sampling and
convergence claims," but that is too easy to wave through.

**Rubric action:** Add a quantitative-claims clause: score down pages that give
point estimates, percentiles, means, or ranks without scope and uncertainty.

### SURVEY

**Finding:** A mature atlas page should be usable by a practitioner, not only a
developer. The rubric should require audience/context awareness: what can a
judge, commission staffer, engineer, or researcher do after reading it?

**Rubric action:** Add audience and operational use prompts to Explanatory Story
and Integration.

### CONTOUR

**Finding:** The rubric mentions inputs generically but does not require data
provenance. For redistricting algorithms, graph/population/geography inputs are
not incidental.

**Rubric action:** Add data provenance and unit-order prompts to Behavioral
Mechanics and Traceability.

### BOUNDARY

**Finding:** Legal sufficiency disclaimers are present, but the rubric should
also catch pages that conflate algorithmic validity with legal validity.

**Rubric action:** Keep BOUNDARY as a gate in Claim Boundary: a page cannot
score high if it implies constitutional/VRA sufficiency from algorithmic output
alone.

### COVENANT

**Finding:** Traceability should cover chain-of-custody evidence, not only links.
For audit-facing pages, the reader needs to see what is hash-bound and what
external step could break reproducibility.

**Rubric action:** Add manifest/hash/binary/external-tool provenance checks.

### LEDGER

**Finding:** Traceability should name schemas and format versions when pages
discuss RPLAN, GeoJSON, NDJSON, or external compatibility.

**Rubric action:** Add format-version/schema prompts.

### BENCHMARK

**Finding:** The rubric should ask whether the page points to tests or fixtures
that would catch a wrong implementation.

**Rubric action:** Add test/fixture evidence under Traceability.

### TRENCH

**Finding:** The rubric should ask for failure modes and structural prevention.
Thin atlas pages often describe happy paths only.

**Rubric action:** Add failure-mode prompts to Behavioral Mechanics and Claim
Boundary, and require known pitfalls/status paths where relevant.

## Required Changes

1. Add a "Role Lenses" section so reviewers know which voices to invoke by page
   type.
2. Strengthen Behavioral Mechanics with invariants, data provenance, and failure
   paths.
3. Strengthen Explanatory Story with audience/context and operational takeaway.
4. Strengthen Claim Boundary with falsifiability, uncertainty, and legal gates.
5. Strengthen Traceability with tests, schemas, package tiers, and chain of
   custody.

## Acceptance

After the above changes, the rubric is suitable for grading atlas pages and for
planning upgrades to the thin B/T compositor pages.
