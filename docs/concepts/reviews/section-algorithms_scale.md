# Review: section-algorithms.md
**Reviewer**: SCALE (statistician)
**Date**: 2026-05-05

---

## What's accurate

The B.2 result "Mean Polsby-Popper improves by 56% over unweighted bisection across all 50 states" is referenced with the appropriate scope qualifier "across all 50 states." Similarly, County-Sticky's "34% fewer county splits...at a 3% mean Polsby-Popper cost" gives both the benefit and the cost, which is the correct way to report a trade-off. The AreaSection "76% seat stability versus standard bisection at the area_swing=1.10 regime boundary" is correctly qualified as a specific parameter regime. The statement that the Lorenz filter "identifies approximately 8 states per census year where geographic population concentration makes area balance infeasible at strict tolerances" is specific and appropriately hedged by "approximately." The NC GeoSection result "5D/9R outcome that is stable across seeds" is correctly described as an empirical stability result rather than a deterministic claim.

---

## P1 — Required fixes

**Section "B.2 (Edge-Weighted Bisection)"**: The 56% Polsby-Popper improvement is the most prominent quantitative claim in this guide. It appears without a confidence interval, standard deviation, or any indication of variance across states. For a result that spans 50 states and three census years, SCALE requires the distributional information: is 56% the mean? The median? The minimum? What is the state-level variance? Arizona may improve by 80% while Montana improves by 15%. The headline number without distributional context misleads readers into thinking the improvement is uniform. A minimum, mean, and maximum across states would make this result defensible.

**Section "GeoSection (B.8)", NC result**: "5D/9R outcome that is stable across seeds" requires quantification. What constitutes "stable across seeds"? Across how many seeds? Is it stable for all T=600 seeds in convergence mode, or across a smaller multi-seed sample? "Stable" as used here could mean "identical for all seeds tested" or "D/R split does not change for 95% of seeds." These are very different claims for a legal audience evaluating the partisan implications of seed choice.

**Section "ApportionRegions (B.11)", national result**: "223D/209R across all 50 states" is the national 2020 headline. SCALE notes that this is a point estimate for a single run (convergence mode), not the expected value of a distribution. The guide should clarify: Is this the result from a single seed sweep, the mean across multiple runs, or an analytically derived result? If partisan outcomes vary with seed, the national sum will also vary. Without this information, the 223D/209R figure cannot be correctly interpreted.

**Section "ApportionRegions (B.11)", compactness extremum claim**: The guide states "This is the compactness extremum: no redistricting algorithm can produce more compact districts while satisfying population balance and contiguity." This is a very strong claim. "No redistricting algorithm" is a claim about the universe of all algorithms, which requires proof or substantial empirical evidence. SCALE asks: has this been compared to GerryChain ensemble minima? Has it been compared to simulated annealing approaches? The ensemble-methods.md guide reports that the ApportionRegions plan sits at the 0.1-0.2nd percentile of the ensemble, meaning fewer than 0.2% of random plans are more compact — which is strong evidence but does not prove no algorithm can do better. The word "extremum" should be qualified as "compactness frontier" or "empirically near-optimal."

**Section "County-Sticky (B.10)"**: "34% fewer county splits compared to geographic-weight baseline, at a 3% mean Polsby-Popper cost." Neither the 34% nor the 3% figure is accompanied by a confidence interval or a statement of sample size (which states, which census years). Are these results robust across all three census years? Do states with many small counties behave differently from states with few large counties? Without variance information, these point estimates cannot be evaluated for statistical significance.

---

## P2 — Suggested improvements

The ProportionalSection description states "sigma approximately 0 — essentially a single deterministic outcome" for competitive states. SCALE would prefer the actual sigma values from empirical runs to support this claim. "Approximately 0" could mean many things; a concrete value (e.g., "sigma < 0.2 seats across 100 seeds") would make the claim falsifiable.

The section would benefit from a summary table of all quantitative results with uncertainty bounds. Legislative staff and court reporters will cite numbers, and they will cite the most accessible number — typically the one in the boldest sentence. A consolidated table with proper confidence intervals would reduce the risk of selective citation.

---

## Score: 2/4

The guide contains multiple important quantitative claims without uncertainty bounds. The 56% headline and the 223D/209R national result are the two most likely to be cited in legal proceedings, and neither carries the distributional information needed to defend them under cross-examination. The "compactness extremum" claim for ApportionRegions significantly overstates what has been demonstrated. Significant statistical qualification is needed before this guide should be shared with courts or legislative staff.
