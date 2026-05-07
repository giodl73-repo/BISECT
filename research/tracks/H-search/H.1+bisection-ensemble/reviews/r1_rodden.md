# Review 2: Jonathan Rodden (Political Geography, Partisan Bias)
**Paper**: H.1: BisectionEnsemble
**Round**: 1
**Recommendation**: Minor Revision

---

## Summary

BisectionEnsemble is an interesting technical contribution to the ensemble redistricting literature. My concerns are principally about the paper's partisan inference claims: whether the method produces meaningfully different partisan outcomes than standard bisection, and whether the percentile parameter $p$ constitutes a latent partisan lever that the paper has not adequately analyzed.

## Partisan Outcomes: What Do Three States Tell Us?

Table 3 reports partisan outcomes for three states. The headline finding is that BisectionEnsemble "does not systematically shift partisan outcomes in any direction." But three states is a thin basis for this claim. The observed variation — NC stable, WI one-seat shift at $p=0.5$, TX stable at $p=0.5$ and one-seat shift at $p=0.0$ — is consistent with both "the method is partisan-neutral" and "the method interacts unpredictably with local geography in ways we cannot yet characterize."

The paper needs at least a broader state sample, or a more careful analysis of *why* WI shifts at $p=0.5$ but not $p=0.0$, and *why* TX shifts at $p=0.0$ but not $p=0.5$. The current text attributes the WI shift to "the median-cut bisection at the three root-level nodes produces a slightly different spatial arrangement of competitive tracts, pushing one district from safe-Republican to lean-Democratic," but this is descriptive, not analytical. What geographic or demographic feature of WI makes it susceptible to median-cut perturbation? Without this, the claim of partisan neutrality is unsubstantiated.

## The Percentile Parameter as a Latent Partisan Lever

The paper's legal section argues that the choice of $p$ is a "transparent, pre-committed procedure that does not depend on partisan outcomes." This is procedurally true but substantively incomplete. The question is whether different values of $p$ systematically favor one party given the geographic structure of partisan sorting in the United States.

This question has a well-known answer in the redistricting literature: compactness-maximizing procedures (which $p=0.0$ approximates locally) tend to disadvantage Democrats in geographically concentrated urban areas, because the most compact districts around cities tend to pack Democratic votes. The paper's empirical results actually hint at this: in WI at $p=0.5$, Democrats gain a seat; in TX at $p=0.0$, Democrats lose a seat. These are small-sample results but they align with the theoretical prediction.

The paper does not engage with this literature (Rodden 2019, Chen and Rodden 2013) at all. This is a significant omission. The claim that percentile selection is "politically neutral" requires either (a) a theoretical argument that the local edge-cut metric is uncorrelated with partisan geography, or (b) an empirical analysis across more states showing no systematic partisan direction as $p$ varies. Neither is provided.

## Does BisectionEnsemble Sample a Different Space Than Standard Bisection?

A key implicit claim in the paper is that the local ReCom ensemble explores bisections that METIS, with its multilevel coarsening heuristic, cannot reach. This is plausible — METIS with a fixed seed finds a single local minimum, while ReCom explores a neighborhood around that minimum. But the paper provides no direct evidence that the ensemble distribution is meaningfully broader than what multiple independent METIS runs with different seeds would produce.

The natural comparison is BisectionEnsemble($p=0.5$, $T=100$) against MultiSeedMETIS($N=100$) — run METIS 100 times with different random seeds at each node and select the median-edge-cut result. If these produce similar outcomes (compactness and partisan), then the ReCom component of BisectionEnsemble is adding little beyond what seed randomization achieves. This comparison is absent from the paper and would substantially clarify the marginal contribution of the ReCom chain.

## Does Choosing $p$ at Each Node Compound to Produce Systematic Partisan Bias?

The paper treats the percentile $p$ as a single scalar applied uniformly across all nodes. But the partisan implications of a bisection cut depend on local geography, which varies across nodes (e.g., the subregion containing Charlotte behaves differently than the subregion containing rural western NC). A fixed $p$ applied to all nodes will have different partisan implications at each node, and these effects may not cancel.

In principle, a partisan actor who knew the geographic distribution of Democratic and Republican voters could choose a $p$ for each node that systematically favors one party, while claiming to use a neutral criterion (e.g., $p=0.5$ everywhere). The paper should address whether this attack is practical and what defenses against it exist.

## Minor Points

- Section 4.4 (Partisan Outcomes): "The key finding is that BisectionEnsemble does not systematically shift partisan outcomes in any direction" — this needs hedging. "On the three states tested" at minimum.
- Table 3 reports 2020 presidential returns. For WI and TX, which are competitive states, the one-seat shifts may be within the margin of precinct-to-tract interpolation error. The paper should report confidence intervals or sensitivity of the seat count to interpolation method.
- The Kuriwaki (2023) citation is used for partisan data, but CCES is a survey-based measure, not precinct returns. If the partisan data are precinct returns interpolated to tracts, the citation should be to the precinct-level election results dataset, not CCES.

## Overall Assessment

The core algorithmic contribution is sound. The partisan neutrality claim requires broader empirical support and engagement with the geographic sorting literature. The absence of a multi-seed METIS comparison leaves the marginal contribution of ReCom unclear. These are revisions that strengthen the paper's claims without altering the algorithm.
