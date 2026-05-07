# Review 1: George Karypis (Graph Partitioning, METIS)
**Paper**: H.1: BisectionEnsemble
**Round**: 1
**Recommendation**: Major Revision

---

## Summary

The paper proposes embedding a local 2-way ReCom Markov chain at each bisection node of a recursive partition tree, claiming this resolves bipartition failures that plague direct GerryChain application to large-$k$ states. The algorithmic idea is clean and the implementation note (Rayon work-stealing) is plausible. However, the theoretical claims around Theorem 1 and Proposition 2 contain gaps that require attention before publication.

## Technical Concerns

### Theorem 1 (Tractability): The proof is incomplete as stated.

The proof invokes an "intermediate-value property of the discrete population function" to guarantee the existence of a balanced cut edge. The discrete IVP argument is correct in its usual form: if we order spanning tree edges by the population of the resulting left component, adjacent edges in spanning tree order differ by at most one tract. Hence, for any target ratio $r$ with tolerance $\varepsilon \geq \max_v \text{pop}(v)/\text{pop}(H)$, a balanced cut always exists.

However, the proof silently assumes that the spanning tree $\mathcal{T}$ is a path-connected structure in which edges can be ordered to produce a monotone sequence of left-component populations. This holds for any spanning tree — removing any edge of a tree produces exactly two components, and as you sweep over all $|V_H|-1$ edges of the spanning tree, the left-component populations do form a sequence that traverses from near-zero to near-total-population. But the paper does not make this enumeration argument explicit. The casual appeal to "intermediate-value property" will confuse readers unfamiliar with the discrete IVT for spanning tree cuts, and referees from pure CS theory will demand the argument spelled out.

More critically: the proof conflates "at least one accepted plan will exist within $O(|V_H|)$ steps" with "acceptance is guaranteed at each individual step." These are different statements. The correct claim is that the chain will eventually accept, not that each step accepts. The proof should be restructured to show: (a) there exists at least one spanning tree of $H$ that has a balanced cut edge, and (b) this spanning tree is sampled with positive probability under the UST distribution, so the chain accepts in finite expected time. Neither (a) nor (b) is explicitly argued.

### Proposition 2 ($O(Tn\log n)$ Complexity): Sound but overstates sharpness.

The summation argument is correct:
$$\sum_{\ell=0}^{d-1} 2^\ell \cdot O(Tn/2^\ell) = O(Tn \cdot d) = O(Tn\log k).$$

The bound $O(Tn\log n)$ follows from $\log k \leq \log n$. This is fine. However, the claim that Wilson's algorithm runs in $O(|V_H|)$ *expected* time per step is accurate only if the random walk cover time of the local subgraph $H$ is $O(|V_H|)$, which holds for expander-like graphs but not in general. Census tract adjacency graphs in rural regions can be nearly path-like (e.g., linear counties), where cover time can be $O(|V_H|^2)$. The paper should note this caveat and clarify that $O(|V_H|)$ is for "well-connected" subgraphs or state it as an expectation bound under the UST distribution. The worst-case per-step cost for Wilson's is $O(|V_H|^2)$.

### k=2 Per-Node Claim: Accurate for pure binary trees; partially wrong for ApportionRegions nodes.

Section 3.2 (Integration) correctly states that BisectionEnsemble applies "only at bisection nodes where $p'=2$" and that nodes with $p'>2$ use standard METIS. However, the Introduction and Abstract claim "bipartition failure is structurally impossible" for BisectionEnsemble on any $k$. This is technically true for the local ReCom steps, but misleading in the following sense: for TX ($k=38 = 2 \times 19$), the depth-1 split is a 19-way METIS call on the full state. This 19-way METIS call is not handled by BisectionEnsemble and could in principle produce an imbalanced or poor partition that propagates through the tree. The paper should be explicit that the "no bipartition failure" guarantee applies only to the local ReCom steps, not to the METIS calls at $p'>2$ nodes.

### Interaction between METIS Initialization and ReCom Mixing

The paper initializes the ReCom chain with the METIS bisection (Definition 1, Step 1). This is a reasonable warm start, but it introduces a potential bias: if the METIS solution is in a mode of the feasible bisection space that is hard to escape in $T=100$ steps, the ensemble may undersample other modes. The paper does not discuss the mixing time of the local chain starting from the METIS seed, and the Future Work section acknowledges this gap. However, the empirical claims about the ensemble "providing better local coverage of the feasible bisection space" (Section 2) are not substantiated without mixing time analysis. I would require at least an empirical mixing time diagnostic (trace plot of edge-cut over steps) for one state.

## Minor Points

- Algorithm 1 line 83: `$(p', k_1, \ldots, k_{p'}) \leftarrow \text{split}(k)$` — this references the ApportionRegions split prescription but does not define it in this paper. A brief footnote or forward reference to the AR citation would help.
- The paper uses "bipartition failure" and "bipartition imbalance" interchangeably. Standardize on one term.
- The $\lfloor p \times m \rfloor$ indexing in Definition 1 Step 4 should address the edge case $m=0$ (zero accepted plans). What does the algorithm return? The tractability theorem guarantees $m \geq 1$, but this should be stated explicitly as a precondition on the Select step.

## Overall Assessment

The algorithm is sound and the core insight (embed 2-way ensemble at each node) is correct. The tractability theorem needs a cleaner proof, the complexity bound needs the Wilson's algorithm caveat, and the "always 2-way" claim needs qualification for $p'>2$ nodes. With these revisions, the paper can be recommended for acceptance.
