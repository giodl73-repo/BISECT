# Review: Nadia Polikarpova
**Paper**: GeoSection: Isoperimetrically-Normalised Ratio-Optimal Bisection for Congressional Redistricting
**Reviewer**: Nadia Polikarpova (UCSD — program verification, algorithm correctness, formal methods)
**Date**: 2026-05-02
**Score**: 3.0 / 4

---

## Summary

GeoSection adds isoperimetric ratio normalisation to METIS-based recursive bisection. The algorithm scans all feasible split ratios 1:(k-1) through floor(k/2):ceil(k/2) and selects the ratio minimising EC/sqrt(min(i,k-i)). A directional penalty (Phase 2) is defined but not evaluated. Formal elements include a definition of "feasible bisection" and "natural ratio," a lemma (Lemma 3.1: Isoperimetric Scaling) with a proof sketch, and a remark clarifying the approximation's limitations.

The paper is well-structured and the formal elements are appropriate for an algorithms paper at this venue. The key formal gap is the status of the Remark: the paper is appropriately honest that the lemma is an approximation under uniform density, but it does not fully characterise the conditions under which the normalisation may select the wrong ratio. I recommend acceptance with minor revisions focused on the formal completeness of the correctness claims.

---

## Strengths

**S1. Appropriate formal precision on the normalisation's scope.**
Remark 3.1 explicitly states that the isoperimetric correction "does not guarantee that the winner is the globally optimal ratio in any formal sense." This is the correct caveat — the normalisation is motivated by an approximation theorem, not a tight characterisation — and stating it clearly is the right approach.

**S2. Definition of "natural ratio" is well-formed.**
The formal definition of natural ratio (Definition 3.2: argmin over i of EC_min(i:k-i)/sqrt(min(i,k-i))) is unambiguous and computable from public data. This is the key property for the paper's legal argument: auditable, reproducible, and independent of partisan data.

**S3. Per-node ufactor derivation is correct.**
The claim that ufactor = 1 + delta/k distributes cumulative balance error correctly across recursive levels is non-obvious. The paper derives this correctly and provides a numerical example (NC, k=14: ufactor = 1.000357) that makes the arithmetic concrete.

**S4. Recursive independence property stated explicitly.**
The paper correctly notes that "each recursive call finds its own natural ratio independently" and that the right half's natural ratio is not constrained to equal the left half's. This independence property is what makes the Wisconsin bisection tree's self-correcting behaviour (1:7, 1:6, 1:5, then 2:3) meaningful.

---

## Weaknesses

**W1. Lemma 3.1's proof sketch is not tight enough.**
Lemma 3.1 states that for a uniformly populated convex planar region of total boundary length L, EC_min(f) ≈ L * sqrt(min(f, 1-f)). The proof sketch invokes "the isoperimetric inequality for convex bodies" and notes that "for f near 1/2, the optimal cut approaches a straight line across the region's minor axis, with length proportional to sqrt(min(f, 1-f))."

This proof sketch has two gaps: (1) the claim about f near 1/2 assumes the minor axis cut length scales as sqrt(min(f,1-f)) — but for a rectangle, the minor-axis cut has constant length regardless of f. The correct scaling only holds for approximately circular regions. (2) The transition between the "f small" (disk approximation) and "f near 1/2" (linear cut) regimes is not characterised. The lemma's conclusion uses "≈" without bounding the approximation error.

These gaps do not invalidate the algorithm — as the remark correctly notes, the normalisation is working with actual METIS edge-cuts, not with the theoretical minimum. But the proof sketch as written would not survive scrutiny in a theory venue. The paper should either tighten the proof or explicitly label Lemma 3.1 as "Heuristic Lemma" or "Motivating Observation" to signal its informal status.

**W2. The boundary-swap rebalancer lacks formal properties.**
Section 3.5 describes a post-hoc boundary-swap loop (up to 200 iterations) that moves single tracts from the heavier to the lighter side until the target balance is reached. Two properties are uncharacterised: (a) termination — does the loop always terminate within 200 iterations? The paper says "for all but the largest metropolitan subgraphs," implying it does not always terminate; (b) correctness — after rebalancing, is the modified partition still a local minimum of the edge-cut, or could the swaps have increased the cut significantly? The paper should either prove termination within 200 iterations or report the empirical distribution of iterations needed.

**W3. Phase 2 correctness is unverifiable.**
The directional penalty re-weights edges as w'(u,v) = w(u,v) * (1 + lambda * |sin(theta(u,v))|), where theta is the angle between the edge direction and the minor-axis direction. The correctness of this formula depends on the interpretation of "angle between edge direction and cut direction." An edge (u,v) runs in the direction of centroid(v) - centroid(u). A cut that is perpendicular to the minor axis runs in the major-axis direction. The formula as written penalises edges that run perpendicular to the minor axis (high |sin(theta)| when theta measures angle from minor axis) — but the stated goal is to penalise cuts parallel to the minor axis, which would penalise edges perpendicular to the cut direction (i.e., parallel to the major axis). The formula may have the angle convention backwards. This needs to be clarified with a worked example.

**W4. Termination proof for the recursive algorithm is absent.**
The algorithm (Algorithm 1) terminates by the "k=1 base case" — if k=1, return {V}. The recursion is GeoSection(G_L, i*, ...) union GeoSection(G_R, k-i*, ...). Termination requires that i* >= 1 and k-i* >= 1, which follows from the range i in [1, floor(k/2)]. This is correct. However, it also requires that the induced subgraphs G_L and G_R are non-empty — which holds iff the METIS partition produces non-empty sides. For k=2 with extreme population imbalance, METIS could in principle produce a degenerate partition. The paper should add a one-line note confirming non-emptiness of the partition sides, or state that population balance at 0.5% tolerance guarantees this.

---

## P1 Items (Must Fix Before Acceptance)

**P1-I. Fix or relabel Lemma 3.1.**
Either (a) provide a tighter proof that clearly states the geometric conditions under which the approximation EC_min(f) ≈ L * sqrt(min(f,1-f)) holds (approximately circular or disk-like regions, not arbitrary convex regions), or (b) relabel Lemma 3.1 as "Motivating Heuristic" or "Informal Lemma" with a note that the formal status is an approximation rather than a theorem. The current "Proof sketch" is misleading in a paper that otherwise maintains formal precision.

---

## P2 Items (Should Fix)

**P2-I.** Verify the directional penalty angle convention in Section 3.4 with a worked example showing that the formula w'(u,v) = w(u,v) * (1 + lambda * |sin(theta(u,v))|) correctly penalises cuts parallel to the minor axis (not perpendicular to it).

**P2-II.** Add termination analysis for the boundary-swap rebalancer: report the empirical maximum number of iterations observed across all 45 states, and characterise whether the 200-iteration cap is ever reached.

**P2-III.** Add a one-sentence note confirming that the METIS partition always produces non-empty sides under the 0.5% balance constraint, to complete the algorithm's termination argument.

**P2-IV.** The population balance constraint in Definition 3.1 uses the formula [i/k/(1+delta), i/k*(1+delta)]. This is asymmetric around i/k (it allows slightly more undershoot than overshoot). Confirm that this matches the METIS ufactor convention or provide the correct formula.

---

## Verdict

Accept with Minor Revisions. GeoSection is a clean algorithmic contribution with sound implementation. The formal elements are mostly well-handled, and the paper's honesty about the approximation status of the isoperimetric lemma is commendable. The primary issue is that Lemma 3.1 is labelled as a lemma with a proof sketch, but the proof sketch has a gap (the scaling argument only holds for approximately circular regions, not arbitrary convex regions). This does not invalidate the algorithm, but it would not survive formal scrutiny in a theory venue.

The single P1 item — fix or relabel Lemma 3.1 — is straightforward. The P2 items (angle convention check, boundary-swap iteration count, termination note) are minor but worth addressing for a paper that claims algorithmic correctness as part of its legal argument.

**Score: 3.0 / 4**
