# Review 3 — Reviewer: Moon Duchin (Metric Geometry / Redistricting Reform)
**Paper:** C.9 — From Algorithm to Map: Implementation Case Studies for Three Adoption Pathways
**Round:** 2
**Score:** 3/4

## Summary

The revision addresses my primary concerns: the reproducibility question (fixed seed vs. ConvergenceSweep) is now addressed in a dedicated subsection, and the adjustment authority threshold is now quantified. I maintain my score at 3/4. The paper is a practically useful implementation guide with appropriate legal caveats.

## Addressed Issues

The canonical run specification subsection (Section 6.3, responding to my C3 concern) directly answers the reproducibility question I raised. The paper now specifies that the adopted map corresponds to the best-of-ConvergenceSweep seed, recorded in the deposition log with the SHA-256 hash. It acknowledges that cross-platform reproducibility depends on METIS version and operating system, and recommends that the canonical platform be specified in the adopting authority's documentation alongside the seed. This is honest and practical.

The adjustment threshold is now quantified (Section 2.3): adjustments affecting more than 5% of tracts require the commission to document a factual finding for each adjusted tract, and adjustments affecting more than 10% of tracts trigger a restart requirement. This is the quantitative threshold I requested. The 5%/10% thresholds are reasonable guardrails that convert the open-ended adjustment authority into a bounded process.

The timeline concern is addressed in Section 6.1: the paper notes that expedited redistricting timelines (census data in April, maps needed by following year) may require shortening the public comment period for commission adjustments, and recommends that the statutory timeline specify minimum periods rather than fixed durations.

The forward citations are now resolved: the most critical claims (VRASection performance, compactness comparison) are either supported by inline evidence or marked as "forthcoming with citation to [paper]" in a standardized format.

## Remaining Concerns

The adjustment threshold (5%/10%) is now present but the paper does not explain how these specific thresholds were derived. Are they empirically grounded (based on the distribution of community-of-interest adjustments in historical redistricting cycles)? Are they legally derived (from *Harris v. AIRC* or other cases)? Or are they the authors' judgment calls? The threshold should be justified.

The comparative summary table across the three pathways (my Round 1 request from Liang's concern) is now present in Section 5.4, which cross-references it to the workflow table. The three-column comparison (who runs the algorithm / adjustment mechanism / challenge standard) is adequate.

## Minor Issues

- The paper now cites *Hayden v. Pataki* for the CVAP standard, which is the correct legal basis.
- The cross-platform METIS non-determinism caveat is present in the canonical run specification. This is the right level of technical honesty for a legal audience.

## Recommendation

Accept. The reproducibility specification and the adjustment threshold are genuine improvements. The paper is now a technically honest and legally sophisticated implementation guide.
