# NRS v0.3 National Bakeoff Paper Evidence Matrix

**Status:** Tier 1 and Tier 2 complete; later tiers remain separately gated

**Evidence vintage:** 2020 Census blocks and official CD118 comparator

**Claim posture:** governed descriptive comparison, not plan-family superiority

This matrix is the paper-facing routing record for the NRS v0.3 national
bakeoff. It prevents completed assignment and geometry evidence from being
conflated with unavailable political, demographic, ensemble, or alternative-
algorithm evidence.

| Surface | Universe / comparator | Status | Governed result | Permitted paper claim | Primary artifact |
|---|---|---|---|---|---|
| Tier 1 source and assignment checks | 50 States; 8,126,956 source blocks; NRS v0.3 versus official CD118 | complete | 7,889,194 land-containing blocks retained; 435 districts per family; zero State failures | Complete same-vintage atomic-unit comparison under the frozen projection and water rules | `docs/experiments/nrs-v0.3-national-bakeoff-2020/` |
| Tier 1 assignment overlap | Same retained blocks after State-level maximum-overlap label matching | complete | 4,194,107 matching and 3,695,087 differing assignments; 53.162680% block-weighted agreement | Descriptive assignment agreement only | `docs/experiments/nrs-v0.3-national-bakeoff-2020/analysis.json` |
| Tier 1 subdivisions | County and tract GEOID-prefix units in the common 2020 universe | complete | NRS splits 1,808 counties and 19,789 tracts; comparator splits 404 and 4,720 | Descriptive split counts; no legal-compliance or superiority interpretation | `docs/experiments/nrs-v0.3-national-bakeoff-2020/state-summary.csv` |
| Tier 2 geometry | Identical retained block polygons dissolved by plan assignment | complete | 50 States, 435 districts per family, zero failures, exact sequential regeneration | Descriptive block-projected geometry; not original enacted linework | `docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/` |
| Tier 2 compactness estimands | State- and district-weighted PP, exact Reock, convex-hull ratio, and Schwartzberg | complete | Both estimands place CD118 above NRS on PP, Reock, and convex-hull ratio and below NRS on Schwartzberg | Report metric values and direction together; no winner, composite, fairness, or legal claim | `docs/experiments/nrs-v0.3-national-bakeoff-geometry-2020/analysis.json` |
| Diagnostic seed sensitivity | Governed root and fallback diagnostic universes | complete within bounded scope | Root, complete-tree, and activated-fallback censuses found no competing physical cut in the governed candidate universes | Structural/candidate result only; not plan-label invariance or an ensemble distribution | `context/waves/2026-08-05-nrs-2010-national-baseline/WAVE.md` |
| Elections | Named elections plus published precinct-to-block crosswalks and frozen missing-data rules | unavailable | Required national inputs/protocol are not frozen | No current NRS v0.3 partisan comparison | `docs/legal/NRS_EVALUATION_SCHEDULE_V0.1.md` |
| Demographics and opportunity diagnostics | Frozen definitions and block aggregation contract | unavailable | Required national input/semantic gate is not closed | No current NRS v0.3 demographic, VRA, or opportunity result | `docs/legal/NRS_EVALUATION_SCHEDULE_V0.1.md` |
| Converged block-level ensembles | At least four chains with frozen proposals/stopping rules and convergence evidence | not run | No governed national ensemble distribution | No percentile, rarity, or tail claim | `docs/specs/2026-08-07-nrs-v0.3-national-bakeoff-protocol.md` |
| Alternative BISECT structures | Frozen 44-State multi-district 2020 congressional tract cohort; four structures under common neutral controls | complete: Wisconsin, pilot, and full 44-State/176-cell packages independently regenerated | Retained failure histories exposed hidden tolerance/reporting defects, fragmented equal recursive splits, and nondeterministic recursive/tie/summation paths. After remediation, the full matrix passed without seed retries and its normalized evidence regenerated exactly. Its 264 within-State overlap rates range from 47.6257% to 100%; 132 alternative edge-cut ratios to standard range from 0.623301 to 5.441826. | Frozen-cohort implementation validity and descriptive within-State mechanics only; not causal inference, legal conclusion, or ranking | `docs/experiments/neutral-algorithm-family-bakeoff-national-2020/`; pilot and pre-determinism witnesses retained separately |
| Non-enacted external plans | Source-bound plan families projected to the same atomic universe | not run nationally | No additional comparator family has passed the gate | No generalization beyond official CD118 | `docs/specs/2026-08-07-nrs-v0.3-national-bakeoff-protocol.md` |

## Paper Routing

- U.21 may use the bakeoff as national operational evidence while keeping exact
  boundary and canonical proof coverage at zero.
- A.0 may use Tier 1 and Tier 2 as the current governed NRS v0.3 comparison and
  must keep earlier tract-profile political and compactness results separate.
- A.5 may summarize coverage and the claim boundary, but should not reproduce a
  winner framing.
- B.0's older four-State, eight-configuration bakeoff is a different research
  artifact. The Wisconsin, pilot, and full national structure packages do not
  fill its differently defined estimated or pending cells. Their retained
  failures and exact post-remediation regenerations establish bounded
  implementation history and frozen-cohort coverage only.
- K-series papers may explain metric semantics and limitations. They should
  cite these values only as common-block, block-projected 2020 measurements.

## Frozen Claim Boundary

The completed package supports descriptive, source-bound comparison of NRS
v0.3 and official CD118 assignments on a common 2020 Census-block universe.
It does not establish original-linework compactness, plan-family superiority,
fairness, intent, VRA compliance, community preservation, robustness,
optimality, causation, legal validity, or adoption suitability.
