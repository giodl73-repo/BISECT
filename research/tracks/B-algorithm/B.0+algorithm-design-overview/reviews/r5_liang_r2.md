---
reviewer: Percy Liang
round: 2
score: 3
date: 2026-05-05
---

## Summary

The bakeoff value provenance documentation is significantly improved. The tpwgts memory layout fix is a correctness improvement. The three core reproducibility P1 items — dataset release, estimation model specification, and software availability statement — remain unresolved. Score maintained at 3.

## P1 Resolution

**P1.1 — Public dataset release: NOT ADDRESSED.**
The paper still does not provide TIGER/Line adjacency graph SHA-256 hashes for the four bakeoff states, plan manifests from the confirmed bakeoff runs, or census_release_id strings. The reproducibility claim ("all results reproducible from stated parameters and SHA-256-verified adjacency graphs") remains aspirational. For the journal submission, this is a binary requirement: the data must be released or the reproducibility claim must be softened.

**P1.2 — Estimation model specification: PARTIALLY ADDRESSED.**
The new provenance paragraph states that estimated (†) values derive from "the theoretical relationship between edge-cut and proportionality gap established in B.8–B.9, interpolation from confirmed values in adjacent configurations, or model-derived predictions" with explicit ±1 seat / ±3pp uncertainty. This is better than R1 but still does not state which of the three sources applies to each estimated cell. The ambiguity in the estimation source is a reproducibility gap: an independent team cannot verify the estimated values without knowing whether they are theoretical, interpolated, or model-derived.

**P1.3 — Software availability statement: NOT ADDRESSED.**
The paper still states "available as open-source software" without a URL, repository name, or commit hash. The Cargo.lock is referenced but not released. For a paper using the hash chain as its reproducibility mechanism, the absence of a public repository URL is a structural gap in the reproducibility claim.

## Positive Assessment

The tpwgts memory layout correction is an important engineering fix. The explicit row-major description with the interleaved-form warning ("silently mis-weights both constraints") is the correct implementation-level specification. An independent team implementing AreaSection from the paper description will now produce correct METIS calls.

The LaTeX macro fix (`\pend` → `\pending`) is a compilation correctness fix that I note from the source. The `\pending` macro (`$^{\ddagger}$`) is now correctly invoked throughout the bakeoff tables.

The GerryChain revision correctly positions the comparison. The MCMC ensemble characterisation (thousands of plans, solution space geometry) vs. toolbox single-plan characterisation is now accurate and will not draw criticism from the computational redistricting community.

## Remaining Gap: EC_norm Interaction with AreaSection

Karypis P1.3 identified that the EC_norm normalisation interaction with the AreaSection dual-constraint tpwgts is unresolved. B.16 now contains the two-case EC_norm definition (recursive bisection vs. k-way partition), but B.0 does not reference B.16's definition for this purpose. B.0 should add a sentence in the AreaSection (C3) subsection: "The normalised edge-cut comparison across bisection ratios uses the recursive-bisection definition from B.16: at each level of the tree where a region of k_ℓ districts is split i:k_ℓ-i, EC_norm is normalised by sqrt(min(i, k_ℓ-i)). The area constraint tpwgts[0.5, 0.5] are fixed across all ratios; only the population tpwgts vary with p_L." This is a P2 item for the next revision.

## Score: 3 / 4 — Minor Revision

The paper is substantially improved in presentation and has one important correctness fix (tpwgts). The reproducibility gaps (dataset release, estimation model source, software URL) are structural issues that must be resolved for a journal submission. In the current B-series internal review context, the paper is useful as a synthesis reference.
