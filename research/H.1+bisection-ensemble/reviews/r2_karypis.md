# Review 1 (Round 2): George Karypis (Graph Partitioning, METIS)
**Paper**: H.1: BisectionEnsemble
**Round**: 2
**Recommendation**: Minor Revision

---

## Response to Round 1 Concerns

### C-1: Theorem 1 Proof — Addressed (Satisfactory)

The revised proof is substantially improved. The three-part structure I requested is now present: (1) the discrete sweep over all $|V_H|-1$ edges of the spanning tree is made explicit, with the adjacent-edge population difference bounded by $\max_v \text{pop}(v)$; (2) the tolerance condition is tied to the empirical observation that tracts represent $\ll 0.5\%$ of node population; (3) the $O(\log(1/\delta))$ resample bound is stated correctly.

One remaining issue: the proof opens with "The merged region has $m = n/k$ tracts on average," but $m$ is then used as a free variable for node size throughout the proof, while earlier in the paper $n/k$ refers to the average leaf-node size (not the average node size across all depths). At depth $\ell$, the node size is $n/2^\ell$, not $n/k$. This conflation is minor but should be corrected for precision: state "at a given node managing $m$ tracts" rather than equating $m = n/k$.

### C-2: Wilson's Algorithm Complexity Caveat — Addressed (Satisfactory)

The revised Proposition 2 proof correctly adds: "For near-path-like subgraphs (rural regions), worst-case expected time is $O(|V_H|^2)$; the bound $O(|V_H|)$ should be interpreted as an average-case statement over typical redistricting subgraphs." This is exactly the qualification I requested.

### C-3: "No Bipartition Failure" Scoping — Addressed (Satisfactory)

The Introduction, Abstract, and Conclusion now consistently use "at the local ReCom level" and "at bisection nodes" to scope the guarantee. The Conclusion explicitly states "The guarantee applies to 2-way bisection nodes; $p'>2$ ApportionRegions nodes continue to use METIS directly." This is correct and defensible.

### S-6: $m=0$ Edge Case — Addressed (Satisfactory)

The fallback to $\pi_0$ with the note that $m=0$ has probability zero in the limit is correctly placed in Definition 1 Step 4. This is the right treatment.

## Remaining Concerns

### Minor Issue: Theorem Statement vs. Proof Scope

The Theorem 1 statement says "will produce at least one accepted plan within $O(|V_H|)$ steps in expectation," but the proof now establishes this only probabilistically: "with probability $\geq 1-\delta$ after $O(\log(1/\delta))$ resamples." These are not the same statement. The theorem statement should either say "in $O(\log(1/\delta))$ steps with probability $\geq 1-\delta$" or the proof needs to convert the probabilistic bound to an expected-value bound (via $E[T_{\text{accept}}] = \sum_\delta \Pr[T > t] dt$). This is a minor precision issue; the result is correct but the statement and proof need to match.

### Minor Issue: "bipartition failure" Still Not Formally Defined

The paper introduced the term "bipartition failure" in Round 1 without a formal definition. This was flagged as a minor point and remains unaddressed. A one-sentence definition in Section 1 or Section 2 would prevent confusion: "We say a ReCom step *fails* if no balanced cut edge exists in the sampled spanning tree; a chain *stalls* if the acceptance rate falls below a threshold $\alpha$."

## Overall Assessment

The theoretical section has improved substantially. The proof of Theorem 1 is now defensible, the Wilson's caveat is in place, and the scoping of the "no failure" guarantee is correct. The remaining issues are minor precision matters that can be addressed in a final revision pass. I am prepared to recommend acceptance after these minor corrections.

**Score**: 3.0/4
