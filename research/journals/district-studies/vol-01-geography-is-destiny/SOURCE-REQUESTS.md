---
journal: District Studies
volume: 1
title: "Geography Is Destiny"
status: active-requests
updated: 2026-05-09
---

# Source Requests

These are the concrete missing artifacts needed to promote the issue from
audition toward provisional.

## T.5 ProportionalSection

Owner lens: MERIDIAN/SCALE.

Needed:

1. Exact T.5 METIS run vector:
   - backend/engine;
   - METIS version or implementation revision;
   - `niter`;
   - `ncuts`;
   - `numbering`;
   - contiguity/minconn settings;
   - seed list or seed derivation rule.
2. Table 1 aggregation rule:
   - deterministic across all 30 seeds; or
   - average across seeds; or
   - maximum/worst seed; or
   - another rule.
3. Seed variation:
   - range, SD, CI, or explicit "all 30 seeds identical" statement.
4. C(G) estimator:
   - Lorenz-analytical; or
   - METIS-empirical; or
   - combined method with formula.

Current evidence:

- `reviews/b12-reproducibility-and-scope-audit.md`
- `reviews/b12-implementation-provenance-note.md`

## G.1 GerryChain Congressional Comparison

Owner lens: SCALE/DATUM.

Needed:

1. Confirm the draft note uses the current G.1 ESS caveats correctly.
2. Confirm whether the NC "middle of the distribution" phrasing should use:
   - exact percentile;
   - approximate percentile range; or
   - no number in public issue copy.
3. Identify whether T.4 support is needed in the article body or only in a
   source note.

Current evidence:

- `drafts/g1-ensemble-median-case.md`
- `reviews/source-chain-audit.md`

## L.1/C.5 Measurement Outputs

Owner lens: SCALE/BENCHMARK/DATUM.

Needed only if measurement is reopened:

1. Regenerate L.1/C.5 numeric outputs under current implementation; or
2. locate post-`81a57bbb` artifacts; or
3. rewrite the measurement piece as methods-only with no directional numbers.

Current decision:

- Result-bearing measurement is deferred for Vol. 1.
- See `decisions/measurement-slot.md`.

## Public Copy

Owner lens: BOUNDARY/COMMONS.

Needed:

1. Review all five draft notes for claim discipline.
2. Confirm title handling: "Geography Is Destiny" is a hook, not a theorem.
3. Ensure no adoption or legal conclusion appears before the terminal
   connection.
