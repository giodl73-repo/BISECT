---
reviewer: Jonathan Rodden
round: 1
score: 3
date: 2026-05-06
---

## Review: PercentileSweep — Statutory Choice of Legal Posture in Algorithmic Redistricting

### Strengths

The paper's central empirical claim — that projected Democratic seat counts are identical across all five percentile values for all six tested states — is stated with appropriate precision. The use of $T = 101$ plans with exact quantile ranks at $\{0, 25, 50, 75, 100\}$ is a sensible design that avoids interpolation artifacts. Reporting both the partisan outcome and the $\ECnorm$ spread (Table 2) allows readers to verify that compactness is in fact varying while partisanship is not. The invocation of B.7's CV-below-2% result as the mechanistic explanation is the right framing; if plans in the bisection family cluster tightly in compactness, they necessarily cluster tightly in the geographic-partisan correlation that compactness encodes.

The acknowledgment of the geographic-sorting mechanism is present and correct in its broad strokes. The paper correctly attributes partisan insensitivity to "geographic structure, not search strategy" (§6.2) and cross-references B.0 and B.7, which establishes the causal chain. This is the appropriate framing for a political science audience familiar with the geographic determinism literature.

The six-state sample is defensible given the G.1 ensemble baseline requirement. Restricting to G.1 states allows direct positioning of each percentile within the broader ReCom ensemble, which is the paper's methodological innovation. This is a reasonable scope decision.

### P1 Items (must resolve before acceptance)

**P1-A: The geographic-sorting concern is acknowledged but not engaged.**
The paper cites B.7's finding that "geographic structure determines partisan outcomes" as an explanation for insensitivity, but this framing elides the Rodden geographic-sorting problem. The concern is not merely that geography correlates with partisanship, but that urban clustering of Democratic voters produces a structural bias: any compact districting scheme — regardless of percentile — will systematically pack Democratic voters into urban districts and crack suburban ones. The insensitivity result then says: across percentiles $p \in [0,1]$, you get the same seat count, but that seat count reflects the compact-algorithm family's structural bias, not a neutral outcome. The paper does not acknowledge that the "223D / 209R" national headline may itself reflect this structural crack. A minimum engagement is required: either (i) cite the urban clustering literature (Rodden 2019 or equivalent) and note that insensitivity across percentiles does not imply neutrality across algorithm families, or (ii) show that 223D/209R is consistent with a proportionality benchmark. The current text reads as if insensitivity implies neutrality, which is a non-sequitur to a political science reader.

**P1-B: TX and CA results are marked "interpolated from B.11" — this weakens the headline claim.**
Table 1 footnotes that Texas and California results are interpolated from B.11's 50-state run rather than from actual PS sweeps. These are the two largest delegations in the sample (38 and 52 seats respectively). If even one seat shifts for TX or CA across percentiles when the actual sweep is run, the "0 seats" maximum variation finding collapses to "at most 1 seat." The paper should either run the actual TX and CA PS sweeps (a computation that takes approximately 3.4 minutes per state by the paper's own runtime estimate) or remove TX and CA from the main table and treat them as a pending replication. Presenting interpolated values as confirmation of the insensitivity finding is premature.

**P1-C: No confidence interval or bootstrap bound is provided.**
The "0.5-seat bound" claim requires error characterisation. With $T = 101$ draws, each quantile estimate has sampling variance. A bootstrap confidence interval over the $p = 1.0$ plan's partisan outcome (the most variable draw) would strengthen the claim. The current presentation reports point estimates only, which is insufficient for a quantitative claim being submitted to a legal/legislative record.

### P2 Items (recommended improvements)

**P2-A:** The paper should cite Rodden (2019, *Why Cities Lose*) or equivalent geographic-sorting literature. The geographic determinism framing in B.7 is downstream of this body of work; claiming "geographic structure determines partisan outcomes" without engaging the political economy of how that structure is formed reads as incomplete to specialists.

**P2-B:** The six states in the sample are not a representative cross-section of partisan geography types. Four of the six (NC, GA, PA, WI) are classic swing-state battlegrounds. Adding a reliable blue state (Massachusetts, 9 seats) and a reliable red state (Oklahoma, 5 seats) would test whether the insensitivity result holds when geographic-partisan concentration is extreme in one direction.

**P2-C:** The abstract states the bound is "at most 0.5 seats" but §4.4 clarifies the actual observed bound is 0 seats. This discrepancy should be resolved by either updating the abstract to state 0 seats (pending TX/CA actual sweeps) or explaining why 0.5 is the appropriate bound to state despite the 0-seat observation.

### Score

**3 / 4** — Accept with revisions. The geographic-sorting concern (P1-A) is a substantive gap that a political science reviewer will flag; the interpolated TX/CA data (P1-B) undercuts a key finding. Both are addressable with modest additional work.
