# Review 3: Moon Duchin (Metric Geometry, GerryChain)
**Paper**: H.1: BisectionEnsemble
**Round**: 1
**Recommendation**: Major Revision

---

## Summary

This paper proposes embedding local 2-way ReCom chains at bisection tree nodes as a workaround for bipartition failures in large-$k$ direct ensemble methods. I am the most appropriate reviewer to evaluate the paper's claims about GerryChain's limitations and the correctness of the "no bipartition failure" claim. I have substantial concerns about both.

## The "No Bipartition Failure" Claim: Not Quite Right

The paper's headline claim — "Since every local ensemble is always 2-way, bipartition failure is structurally impossible" — is too strong and requires careful qualification.

In standard ReCom (GerryChain), bipartition failure occurs when a merged district pair must be re-split into a balanced bipartition of the merged subgraph. For a $k=38$ state, the merged region holds 2 of 38 seats and the cut must achieve a $1:1$ ratio *within the merged region* — this is actually a 1:1 balance, not a 1:37 balance. The paper's description in the Introduction is incorrect on this point: "each district holds $1/38 \approx 2.6\%$ of the state population; after merging two adjacent districts the merged region holds $5.3\%$, and the cut must split this into two regions of $2.6\%$ each." This is correct: the merged region holds 2/38 of population, and the cut achieves a $1:1$ balance *within the merged region*. This is always achievable in principle.

The actual failure mode for large $k$ in GerryChain is different and more subtle: in a $k=38$ state, merging two adjacent districts produces a merged region of $\approx 2/38$ of state population, but the surrounding $k-2 = 36$ districts constrain the boundary. The spanning tree of the merged region must find a cut that not only achieves $1:1$ balance but also respects the irregular boundary imposed by the surrounding 36 districts. It is this geometric constraint — not an inherently extreme balance ratio — that causes low acceptance rates for large $k$. The paper's causal diagnosis is off.

For BisectionEnsemble, the claim is that every local ReCom step is 2-way, so the "bipartition failure problem" is avoided. But the problem is not merely the arity — it is the geometric structure of the subgraph. At each bisection node in BisectionEnsemble, the local subgraph $H$ is an induced subgraph of the state graph, and its shape is determined by the parent node's bisection. If the parent produces an elongated or geometrically awkward subregion (which METIS sometimes does), the child's local ReCom chain may have low acceptance rates regardless of the arity. This is not "bipartition failure" in the GerryChain sense, but it is an analogous failure mode.

The paper should be more precise: BisectionEnsemble avoids the specific failure mode where a $k$-way chain requires a $1:(k-1)$ imbalanced spanning tree cut at the full-state level. But it does not guarantee high acceptance rates at every node, and some nodes may produce nearly-degenerate subregions where the local chain has low acceptance.

## The ReCom Implementation at Each Node: An Important Subtlety

Definition 1 (Step 2) describes the local ReCom chain. Step 2(b) says: "Form the merged subgraph $M = H[A_{t-1} \cup B_{t-1}] = H$." This is correct — since the node manages exactly two parts $A$ and $B$ of $H$, the merge domain is all of $H$. But this means that at each step, the chain samples a *full spanning tree of $H$*, not a spanning tree of a small 2-district subregion. This is qualitatively different from standard ReCom, where the merge domain is typically a small fraction of the state.

In standard ReCom for a $k$-district plan, each step merges 2 of $k$ districts ($\approx 2/k$ of the full graph), samples a spanning tree of that small region, and makes a local move. The chain mixes efficiently because each step makes a local perturbation. In BisectionEnsemble, each step at the root node samples a spanning tree of the *entire state subregion*, which is $1/2$ of the state (for the root bisection). The acceptance probability involves a population balance condition on a tree spanning half the state. This is not "local" in any meaningful sense for the root node.

The paper claims "local feasibility sampling" as a key advantage, but the root-level ensemble is sampling over bisections of the entire root subregion — which, for the first application of BisectionEnsemble (the depth-0 root bisection), is the entire state. This is essentially full-state bisection ensemble, not local ensemble. The "locality" only applies at deeper tree levels where subregions are smaller. The paper should be transparent about this.

## Ergodicity: Claimed But Not Proved

Section 5.1 states: "By the ergodicity of the ReCom chain, the distribution of accepted plans converges to the stationary distribution over feasible bisections as $T \to \infty$." Ergodicity of the local chain is not established in this paper, and it is not obvious. The full Future Work section acknowledges "formal ergodicity proof for the local chain over the feasible bisection space of each node would strengthen the theoretical foundation." This acknowledgment in Future Work is appropriate — but Section 5.1 should not invoke ergodicity as established fact in the legal properties section. This is a legal claim based on an unproven theorem, which is problematic.

For the legal section, the claim should be weakened to: "To the extent the local chain mixes over the feasible bisection space, the median plan is representative of that space; mixing time is left to future work."

## GerryChain Acceptance Rates: Verifiability

Table 1 reports GerryChain acceptance rates for NC (61%), WI (84%), and TX (<1%). These are presented without a GerryChain version number, configuration, or hyperparameters. GerryChain's acceptance rate depends on the tolerance $\varepsilon$, the proposal distribution, and whether two-step or multi-step ReCom is used. These numbers are not reproducible as stated.

More importantly, the acceptance rate for TX is described as "<1% of ReCom steps are accepted at 100 steps, yielding 0--1 accepted plans." But GerryChain's actual failure mode on TX is typically that it does produce plans — they are just nearly identical to the initial plan (the chain is stuck). The characterization as "stalled" is accurate, but the mechanism (near-zero Metropolis acceptance) should be distinguished from true bipartition failure (where no balanced cut exists). For TX's actual factorization $38 = 2 \times 19$, GerryChain would be run differently (with custom handling), and the paper does not specify which GerryChain configuration it tested.

## Compactness Claims

The compactness improvements in Table 2 (5--12% at $p=0.5$, 15--25% at $p=0.0$) are plausible and internally consistent. I have no technical objection to these numbers, but note that they are relative improvements over a baseline (METIS bisection) that is not itself compactness-optimized. The absolute Polsby-Popper values (0.184 to 0.261) are unremarkable for census tract-level congressional districts; enacted congressional districts typically have lower PP scores (0.10--0.20 nationally). The paper should contextualize these values against actual enacted plans.

## Minor Points

- The GerryChain reference (Duchin and Walch 2019) is an early arXiv preprint. The primary citation for GerryChain should be the DeFord, Duchin, Solomon 2021 HDSR paper, which is already cited as `deford2021recombination`. The `duchin2019gerrychain` citation attributes GerryChain to a preprint co-authored by Duchin and Walch, which is not the standard citation for the method.
- Section 2: "The critical limitation identified in this paper — bipartition failure for large prime $k$ — is a known but underreported problem in the GerryChain community." This is partially true, but the community is well aware of the problem. "Underreported in the literature" would be more accurate.
- The paper does not define "bipartition failure" formally. A definition would help readers distinguish the GerryChain failure mode from BisectionEnsemble's potential low-acceptance-rate mode.

## Overall Assessment

The algorithmic contribution is genuinely useful, and the insight of embedding 2-way ensemble at each tree node is correct. The paper needs (1) a corrected causal account of GerryChain's TX failure, (2) honest acknowledgment that "locality" is relative to tree depth, (3) removal of the ergodicity claim from the legal section, and (4) reproducible GerryChain experimental parameters. These are substantial but addressable revisions.
