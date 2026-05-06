# Revision Plan — G.2: Partisan Outcome Distributions
**Round 1 → Round 2**

## Scores

| Reviewer | Score | Recommendation |
|---|---|---|
| Karypis | 3/4 | Moderate revision |
| Rodden | 3/4 | Moderate revision |
| Duchin | 2/4 | Major revision (pending G.1 corrections) |
| Stephanopoulos | 3/4 | Moderate revision |
| Liang | 3/4 | Minor-to-moderate revision |
| **Mean** | **2.8/4** | |

## Dependency: G.2 must wait for G.1 corrections

G.2's empirical results table (Section 3) is drawn directly from G.1. The NC 54th percentile and GA 38th percentile cannot be confirmed until G.1's NC PP inconsistency and DeFord 2021 attribution are resolved. The following revisions can be made independently of G.1; the results table revision awaits G.1.

## Blocking Issues

### B1. Overdispersion direction is wrong
**Issue**: Paper models $S_D \sim \text{Binomial}(k, p_{\rm geo})$ "with overdispersion." Using $p_{\rm geo} \approx 0.36$ for GA gives binomial $\sigma = 1.80$, but the paper uses $\sigma_{S_D} \approx 1.2$ (from G.1) — the actual SD is LESS than the binomial prediction.
**Fix**: Change to "approximately Binomial($k$, $p_{\rm geo}$) with underdispersion in sorted states due to contiguity and population balance constraints." Provide a brief explanation: contiguous districts in a sorted state cannot be arbitrarily rearranged, so the effective plan space is narrower than what an unconstrained binomial would predict.

### B2. "Five of six states" framing conflates geographic-median with partisan-neutral
**Issue**: Being at the ensemble median in sorted states means replicating geographic Republican over-representation, not being partisan-neutral.
**Fix**: Add a paragraph after the results table:

> "A plan at the ensemble median is not neutral by any proportionality standard. In geographically sorted states, the ensemble median itself reflects the Rodden effect: Democratic voters concentrated in cities cannot win as many compact districts as their statewide vote share would imply. The AR plan's near-median position establishes geographic consistency — the plan is not an outlier relative to what geography permits — but this is distinct from proportional fairness."

This is a Rodden-flagged issue and is important enough to address in the abstract.

## High-Priority Revisions

### H1. Minnesota ensemble source
**Issue**: $\phi = 59\%$ corridor fraction and 44th Democratic seat percentile for MN reported without any ensemble source.
**Fix**: Either (a) cite the specific ReCom ensemble used for MN and confirm it meets G.4 convergence standards, (b) label the result with \est{} and describe the estimation method ("estimated from the binomial model using $p_{\rm geo} \approx 0.5$ for MN"), or (c) remove MN from the paper.

### H2. Corridor fraction uncertainty intervals
**Issue**: $\phi = 42\%, 61\%, 28\%, 55\%$ reported without uncertainty.
**Fix**: Add standard errors using ESS: for $\ess \approx 1{,}000$ (from G.4), the 90% CI for $\hat{\phi} = 0.42$ is $\pm 0.026$. Add uncertainty to all corridor fractions in the table.

### H3. Standard deviation table
**Issue**: The range $\sigma_{S_D} \approx 1.0$–$1.5$ for $k = 8$–$17$ is stated without source.
**Fix**: Add a table of state-specific $\sigma_{S_D}$ values with sources (either from G.1 ensemble data or from the literature). Cross-reference G.1's state summaries.

### H4. p_geo derivation
**Issue**: $p_{\rm geo} \approx 0.36$ for Georgia is stated without derivation.
**Fix**: Add: "$p_{\rm geo}$ is estimated as the fraction of census tracts with Democratic plurality, weighted by population and adjusted for compactness constraints. For Georgia, approximately 36\% of census tracts fall in areas where a randomly drawn compact district would lean Democratic." Or alternatively: "estimated as $1 - q$ where $q$ is the fraction of valid ensemble plans with fewer than $\lfloor k/2 \rceil$ Democratic districts."

## Moderate-Priority Revisions

### M1. Algorithm choice section (Section 5)
**Issue**: "Algorithm choice determines which tail you land in" is asserted but not empirically supported. B.0 is referenced but not available.
**Fix**: Either provide a figure comparing edge-cut vs. compactness-maximizing algorithms on partisan distributions from B.0, or label the claim as: "Hypothesis (to be verified in B.0): minimum-edge-cut objectives tend to produce partisan outcomes closer to the geographic median than objectives that maximise compactness directly."

### M2. Efficiency gap analysis
**Issue**: Stephanopoulos requests engagement with standard partisan fairness metrics.
**Fix**: Add a subsection computing the efficiency gap for the AR plan in each state. Compare with the corridor result. Note where the two metrics agree and disagree.

### M3. Georgia corridor analysis
**Issue**: 28% of ensemble plans achieve the proportionality corridor (6D–7D) for GA, but AR produces 5D (outside corridor). Stephanopoulos asks: are compact plans with 6D achievable?
**Fix**: Add: "Within the DeFord ensemble, approximately 28\% of valid plans achieve 6D or more Democratic seats for Georgia. These plans typically have a less compact boundary that captures more Atlanta suburbs in Democratic-leaning districts — at the cost of a higher edge cut. The AR algorithm trades one Democratic seat for minimum edge cut. This trade-off is a geometric property of the minimum-edge-cut objective, not a partisan design choice."

## Low-Priority Revisions

### L1. B.0 availability note
**Issue**: B.0 is referenced but not published.
**Fix**: Add: "B.0 is an unpublished companion paper in preparation."

### L2. Abstract framing
**Issue**: Abstract leads with "not systematically biased" — which requires the nuance from B1/B2 fixes.
**Fix**: After B1 and B2 are incorporated, revise the abstract to add: "...while acknowledging that near-median outcomes in sorted states reflect geographic Republican over-representation rather than proportional fairness."
