---
journal: District Studies
volume: 1
title: "Geography Is Destiny"
status: audit-seed
updated: 2026-05-09
---

# Source-Chain Audit

## Current Result

Not locked. The source chain is sufficient to start audition, not sufficient to
publish the issue.

## Candidate Source Status

| Source | Use | Current status | Gate |
|--------|-----|----------------|------|
| B.12 ProportionalSection | Formal geography/proportionality argument | Strong lead candidate; separate audit note complete | HOLD numeric claims until METIS parameter vector, seed variance, and C(G) estimation are documented |
| G.1 GerryChain Congressional Comparison | Ensemble-position and NC median claim | Reviewed for audition; usable with caveats | Use `drafts/g1-ensemble-median-case.md` and `reviews/g1-scale-datum-review.md`; avoid legal-certificate wording and raw percentile precision |
| B.11 ApportionRegions | NC 7D/7R and prime-factor bisection context | Useful supporting source | Keep descriptive; avoid tool advocacy |
| C.5 Efficiency Gap Analysis | Measurement slot | Candidate but held | Numeric outputs must be traced to post-fix runs or regenerated |
| L.1 Efficiency Gap | Metric definition | Code-level sign blocker closed; publication provenance still gated | Use `reviews/l1-efficiency-gap-sign-audit.md`; do not lock result-bearing measurement copy until outputs are post-fix |
| L.6 Proportionality vs. Majoritarianism | Reader bridge | Candidate support | Confirm derivation and implementation location caveats |

## Known Risks

- **B.12 reproducibility gap**: Round 3 review flags missing METIS `niter`,
  `ncuts`, and `numbering` values. Treat B.12 as strong but not fully locked
  until the full parameter vector is present. Current implementation defaults
  are summarized in `reviews/b12-implementation-provenance-note.md`, but they
  do not by themselves prove the B.12 paper-run vector.
- **B.12 seed variance gap**: Round 3 reviews ask whether Table 1 values are
  deterministic across 30 seeds or averages/maxima. Locked copy needs range,
  standard deviation, or an explicit "all 30 seeds identical" statement.
- **B.12 C(G) provenance**: Round 3 review asks whether C(G) is
  Lorenz-analytical or METIS-empirical. Do not present C(G) constants as
  settled until the estimator is named.
- **B.12 legal-scope leakage**: the source's `Scope of Claims` section is
  useful, but later legal argument language is too strong for District Studies.
  Use `reviews/b12-reproducibility-and-scope-audit.md` as the issue gate.
- **B.12 scope strength**: Review notes praise the "Scope of Claims" section,
  especially the warning that proportional constraints do not guarantee
  proportional outcomes. Preserve that caveat.
- **B.12 cross-source grounding**: Review notes still ask for GA/WI
  reconciliation with B.9 data. Do not publish GA/WI numeric gaps until this is
  resolved or explicitly caveated.
- **G.1 ESS caveat**: source sections now include ESS notes, but the issue should
  use conservative phrasing: "in this ensemble run, with ESS caveats" rather
  than "fewer than X in 1,000 valid plans."
- **L.1 sign convention**: code-level blocker is closed by commit
  `81a57bbb4419cbccc6e23f534d039bb0a538554e`, which changes the implementation
  to `(Wasted_D - Wasted_R) / total`. The remaining publication blocker is
  output provenance: result-bearing C.5/L.1 copy needs a post-fix run or a
  traceable post-fix artifact.
- **Legal overread**: B.12-style geography findings do not prove partisan intent
  or constitutional invalidity.
- **Title overclaim**: "Geography Is Destiny" is a strong public title. Locked
  copy should say "strongly constrains" unless evidence supports stronger scope.

## Next Checks

1. Close the B.12 gates listed in `reviews/b12-reproducibility-and-scope-audit.md`
   and `reviews/b12-implementation-provenance-note.md`.
2. Add B.12 METIS `niter`, `ncuts`, and `numbering` values to the source chain.
3. Send `drafts/g1-ensemble-median-case.md` through SCALE/DATUM/BOUNDARY review.
4. Regenerate or locate post-`81a57bbb` L.1/C.5 outputs before locking the measurement slot.
5. Verify all candidate PDFs exist under `docs/papers/`.
6. Re-run `reviews/role-panel-review.md` after the first article drafts exist.
