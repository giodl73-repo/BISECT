# Review — H.2: redist-ensemble
**Reviewer**: George Karypis (Graph Partitioning, High-Performance Computing)
**Round**: 2
**Score**: 3.5 / 5

---

## Response to Round 1 Concerns

### R03 (MAJOR): Wilson complexity attribution — Addressed

The abstract now reads: "Wilson's algorithm runs in expected time equal to the cover time of a random walk on the subgraph; for planar graphs this is $O(|V|\log|V|)$~\citep{aldous1991random}."
This is correct. The distinction between Wilson's cover-time result and Aldous's planar specialization is now explicit in both the abstract and Section 1.2.

Section 1.2 now reads: "Aldous~\citeyearpar{aldous1991random} established that $\tau_{\text{cover}} = O(|V|\log|V|)$; the planar bound is his result, not Wilson's." This phrasing is direct and accurate. The correction is well-executed.

### R02 (CRITICAL): Pair reselection and stationarity — Partially addressed

The new paragraph in Section 4.5 ("Stationarity under pair reselection") represents a genuine improvement. The claim that "pair reselection is equivalent to a proposal rejection that discards infeasible pair proposals uniformly, which preserves detailed balance if the pair-selection distribution is symmetric over adjacent district pairs" is the right argument structure. The condition stated — rejection only on balance failure, not cut quality — is correct and matches the implementation.

However, the argument has a gap I want to flag for round 3: the detailed-balance argument applies to standard Metropolis-Hastings rejection, where a single proposal is accepted or rejected. In this case, pair reselection is not a single-proposal rejection — it is a loop that tries up to 10 spanning trees per pair before abandoning the pair. The effective "proposal" seen by the chain is not a single (pair, tree) but a compound event: "after up to 10 tree failures, select a new pair." The compound event's proposal probability is not symmetric in the same way as a single-step proposal.

This does not make the argument wrong — it is possible that the compound protocol still preserves the stationary distribution — but the argument as stated is not tight. The paper correctly defers formal proof to future work and cites the planned G-track empirical comparison. For a Round 2 revision this is acceptable; the language should be slightly softened from "preserves detailed balance" to "is consistent with preservation of detailed balance under the following informal argument" to avoid overclaiming.

### R04 (MAJOR): Planarity of census-tract subgraphs — Not addressed

The revision plan identified R04 as requiring a sentence in Section 3.2 or 4.2 stating that census-tract adjacency subgraphs derived from TIGER shapefiles are planar (by construction from non-crossing polygons). This sentence does not appear in the revised paper. The abstract now cites Aldous for the planar bound, but the claim that the specific subgraphs used in practice are planar is still unverified. For a paper that applies the $O(m\log m)$ bound to actual runs, this is a required fix.

**Recommended addition** (Section 3.2, after the theorem): "In practice, the subgraph $H = G[V_i \cup V_j]$ is a planar graph: it is induced from the census-tract adjacency graph, which is derived from TIGER shapefiles defining non-crossing polygonal boundaries. Small non-planar artifacts (tri-point boundaries, state-line adjacencies) affect at most a small fraction of steps and do not alter the asymptotic cover-time bound."

---

## Remaining Concerns

### Throughput estimate sensitivity (R10 — not addressed in R1 P1 list, but still open)

Section 5.2's $13\times$ overhead factor remains stated without a sensitivity range. The revision plan called for a table showing the speedup under $6.5\times$ and $26\times$ overhead. This was not implemented. At minimum, the remark at the end of Section 5.2 should note the sensitivity bounds.

### BigCrush claim (R12 — not addressed)

Section 4.3 still states SmallRng "passes the BigCrush statistical test suite." The revision plan flagged this as incorrect (xoshiro128++ passes PractRand; its BigCrush status is uncertain). This should be corrected to "passes the PractRand statistical test suite."

---

## What Remains Strong

The core algorithmic correctness (balance-cut enumeration, tree-resample-on-failure) is sound. The performance table with its explicit dagger notation and Phase 2 disclaimer is appropriate. The stationarity paragraph, despite the gap noted above, represents a significant improvement over the Round 1 version.

---

## Recommendation

**Minor revision** (upgraded from Major Revision). The critical attribution issue is resolved. The stationarity argument is present and acceptably hedged. Two remaining items require attention before acceptance: the planarity claim for census-tract subgraphs (R04, was Major in R1), and the BigCrush correction (R12, was Moderate in R1). These are both one-sentence fixes.
