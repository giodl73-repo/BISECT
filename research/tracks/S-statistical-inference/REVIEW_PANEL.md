# Track S — Statistical Inference Panel Review
**Date**: 2026-05-09 | **Round**: 1
**Papers**: S.0–S.4 (5 papers)
**Reviewers**: Duchin, Karypis, statistician (Efron), Liang, Stephanopoulos

## Paper Scores (Round 1)

| Paper | Title | Score | Verdict |
|-------|-------|-------|---------|
| S.0 | Statistical Inference Overview | 3.2/4 | Conditional Accept |
| S.1 | Hypothesis Testing / Permutation | 3.4/4 | Conditional Accept |
| S.2 | Bayesian Redistricting | 3.6/4 | Conditional Accept (strong) |
| S.3 | Power Analysis | 3.2/4 | Conditional Accept |
| S.4 | Multiple Testing (FDR) | 3.8/4 | Conditional Accept (strong) |
| **Track Mean** | | **3.4/4** | |

## Module Score: 8.6/10

## Track-Level Strengths
- S.4 is the highest-scored paper in the Track S series: the connection to the existing bloc_voting.rs Holm-Bonferroni implementation is a genuine contribution — it makes FDR-corrected redistricting analysis immediately deployable without new code.
- S.2's Bayesian Detection Score is the most mathematically elegant contribution in the track. The Beta posterior closed-form expression and the BDS = 0.97 for NC 2022 are compelling.
- S.1's ESS correction changes the NC p-value from 0.003 to 0.041 at 1,000 steps — this is the most practically consequential result in the track, as courts are currently using 1,000-step ensembles without ESS correction.

## P1 Items by Paper
- **S.1**: The paper claims "all five enacted plans achieve p < 0.05 at 1,000 steps" but Table 3 shows TX achieves p = 0.11 at 1,000 steps — only p = 0.03 at 10,000 steps. Fix the abstract/conclusion to be consistent with the table.
- **S.3**: The power function derivation assumes normality of the test statistic distribution — this may not hold for very small ensembles (ESS < 30). Add a note on minimum ESS for the power formula to be reliable.
- **S.2**: The uniform prior is a strong assumption — in adversarial redistricting contexts, a prior that places more mass on extreme plans (gerrymandering-prone states) would be more informative. Acknowledge prior sensitivity.

## P2 Items
- S.1: Provide Python/scipy code for the ESS-corrected p-value computation alongside the Rust implementation — practitioners are more likely to use Python. (Liang)
- S.4: Apply the FDR correction retrospectively to the published redistricting literature — which published gerrymandering findings survive FDR correction? This would be a significant contribution. (Efron)
- S.3: Add the power analysis as a `bisect label-analyze --types power` subcommand recommendation — this would make power analysis integrated into the standard workflow. (Karypis)

## Next Action
S.1 abstract/table consistency (P1) is a two-sentence fix. S.3 normality assumption note (P1) is a one-sentence footnote. S.2 prior sensitivity (P1) is a paragraph. All are short. The track is methodologically the strongest new addition to the program.
