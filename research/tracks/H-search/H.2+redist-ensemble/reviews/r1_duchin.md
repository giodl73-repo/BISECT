# Review — H.2: redist-ensemble
**Reviewer**: Moon Duchin (Geometric Group Theory, Mathematical Foundations of Redistricting)
**Round**: 1
**Score**: 3 / 5

---

## Summary

The paper describes a Rust port of the ReCom Markov chain and argues that Rust's compiled execution enables a 2,300× throughput improvement over GerryChain. The algorithmic description of Wilson's algorithm and the balance-cut enumeration logic is largely sound. However, the paper contains a significant mischaracterization of the Texas bipartition failure mode that overstates what the Rust implementation actually achieves, and the description of what pair reselection accomplishes requires substantial revision.

---

## The Texas/California Claim: A Combinatorial Misrepresentation

The most problematic passage in the paper is Section 4.5 (Texas and Bipartition Failures) and its companion in Section 5.3. The paper asserts:

> "redist-ensemble's pair-reselection mechanism enables it to traverse the low-feasibility region rapidly. [...] Rust's advantage is that the 10-resample budget is exhausted in microseconds rather than the seconds it would take in Python, so pair reselection adds negligible overhead."

And in Table 1, the TX and CA rows are marked "failure" for GerryChain (cold start) versus non-failure for redist-ensemble, with the footnote: "redist-ensemble handles via pair reselection."

This framing is misleading in a way that matters for the paper's credibility. The bipartition failures in Texas are not a Python problem — they are a combinatorial problem that exists independently of implementation language. GerryChain's documented difficulty with Texas is not that it cannot execute quickly enough to exhaust the resample budget; it is that the plan space for $k=38$ from an arbitrary cold-start configuration has very low balanced-bipartition feasibility across many district pairs, causing the chain to make no progress regardless of how fast each resample attempt runs. The chain stalls not because the resamples are slow, but because no spanning tree for the selected pair has a balanced cut, and the pair selection itself is not directed toward feasible regions.

Pair reselection — trying a different pair after 10 consecutive tree failures — does address a specific failure mode, but it is not a Rust-specific solution. GerryChain with pair reselection implemented in Python would behave identically in terms of which pairs it explores. The Rust implementation would exhaust the resample budget faster and therefore cycle through pairs faster, which is a real advantage. But the paper should state this accurately: the advantage is throughput within a combinatorially identical algorithm, not a structural fix to the TX bipartition problem.

Specifically, the claim in the conclusion (Section 7.1) that redist-ensemble enables "TX and CA runs that Python GerryChain cannot complete from cold start" should be revised. GerryChain with pair reselection and sufficient patience can run TX from cold start; GerryChain without pair reselection stalls. The relevant comparison is redist-ensemble (with pair reselection) versus GerryChain (with pair reselection). The paper does not make this comparison.

Section 4.5's statement that "this behavior is not a Rust-specific limitation; it reflects the intrinsic geometry of the Texas plan space" is correct and commendable — but then the performance table and abstract undo this clarity by framing redist-ensemble as "handling" what GerryChain "fails" to handle. The paper needs to be consistent: either the TX failure is combinatorial (in which case redist-ensemble handles it no differently than a fast Python implementation with pair reselection would) or it is a throughput problem (in which case the claim that 10-resample exhaustion at microsecond speed is the fix should be stated precisely as a throughput advantage, not a correctness one).

---

## Pair Reselection and Stationarity

This connects to a deeper issue. The ReCom stationary distribution proof (DeFord et al. 2021) assumes a specific Markov kernel, and pair reselection modifies that kernel. The paper correctly notes in Section 3.2 that full balanced-cut enumeration is necessary to match GerryChain's stationary distribution. But the same reasoning applies to pair reselection: pairs with low balanced-bipartition feasibility will be more frequently abandoned and re-selected, systematically altering the effective pair-selection distribution.

The paper gestures at this in Section 4.5 ("pair reselection triggers frequently during the first 50–100 steps...then subsides") but this is not an argument that stationarity is preserved. The stationarity claim requires either a proof that the modified kernel has the same stationary distribution as the unmodified ReCom chain, or an acknowledgment that the chain being implemented is a related-but-distinct variant. DeFord et al. (2021) do not analyze pair-reselection variants; the paper should not silently assume the stationarity result carries over.

This is not just a theoretical nicety. The paper's legal framing depends on the claim that the Rust implementation replicates GerryChain's stationary distribution. If pair reselection alters the stationary distribution, that claim is false.

---

## What the Paper Gets Right on Wilson's Algorithm

The formal description of Wilson's algorithm (Algorithm 1) is correct. The theorem statement in Section 3.2 correctly attributes cover-time complexity to Wilson's paper and the planar bound to Aldous (1991). The observation that the relevant subgraph has $m \approx 2n/k$ vertices, giving $O((n/k) \log(n/k))$ per-step cost, is the right analysis for ReCom's expected complexity. The balance-check enumeration (Section 3.3) is correctly described as a linear-time DFS pass.

The implementation detail in Section 4.4 — rooting the DFS at vertex 0 and accumulating subtree populations post-order — is the standard efficient approach. The use of SmallVec for balanced edges to avoid heap allocation in the common case (few balanced cuts) is a sensible micro-optimization.

---

## Minor Issues

- The paper never defines what "cold start" means for the TX/CA experiments. A cold start is presumably a random valid starting plan, but for $k=38$ (Texas), even generating a valid random starting plan is non-trivial. The paper should define what initial partition is used.
- The Hamming autocorrelation definition in Section 6.3 uses $d_H(\sigma_t, \sigma_0)$ (distance from the starting plan) as the argument for the lag-$k$ correlation, which is an unusual definition. More standard is the lag-$k$ autocorrelation of the summary statistic itself, or the lag-$k$ plan-to-plan Hamming distance $d_H(\sigma_t, \sigma_{t+k})$. The paper should clarify what is being computed.
- Section 5.3 states that "California ($k=52$) presents the highest per-step cost due to its large $n$ and high $k$." The cost is $O((n/k) \log(n/k))$, so high $k$ actually reduces per-step cost (smaller merged regions). For CA with $n=8057, k=52$, the merged region is $m \approx 310$, larger than NC's $m \approx 191$ but not as large as might be expected. The claim that CA has the "highest per-step cost" among the six states should be verified against the formula, not just assumed from large $n$.

---

## Recommendation

Major revision. The Texas/California claim must be revised to accurately characterize what pair reselection achieves: a throughput advantage within a combinatorially identical algorithm, not a structural fix unavailable to Python implementations. The stationarity argument for the pair-reselection variant of ReCom needs explicit treatment. These are correctness issues central to the paper's claims.
