---
title: "Bayesian Redistricting: Prior Specification and Posterior Interpretation"
series: S.2
status: Planned
date: 2026-05-09
track: S-statistical-inference
---

## Claims
1. The GerryChain ensemble can be reinterpreted as a prior distribution over redistricting plans. Given the observed partisan outcome of an enacted plan, the posterior probability that the outcome was intentional (vs. geographic) can be computed via Bayes' theorem.
2. For NC 2022: prior probability of enacted Republican seat share (11/14) under the ensemble = 0.003 (G.1). Likelihood ratio of intentional gerrymander vs. geographic accident = 280. This is strong Bayesian evidence of intentional manipulation.
3. The Bayesian framework handles the "geography defense" naturally: the prior absorbs all geographic effects; the posterior measures only the residual that geography cannot explain.
4. A Bayesian gerrymandering detection score (BDS) is defined as the posterior probability that the enacted plan is not a random draw from the compactness-constrained ensemble. BDS > 0.95 is proposed as the evidentiary threshold for strong gerrymander evidence.
