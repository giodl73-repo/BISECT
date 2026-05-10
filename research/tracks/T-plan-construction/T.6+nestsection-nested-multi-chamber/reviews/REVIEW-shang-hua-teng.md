> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: NestSection — Shang-Hua Teng

**Reviewer**: Shang-Hua Teng (USC)
**Expertise**: Theoretical computer science, spectral graph theory, algorithmic game theory, smoothed analysis, combinatorial optimization
**Round**: 1
**Score**: 3/4 (accept with revisions)
**Recommendation**: Accept with Revisions

---

## Summary

NestSection presents a polynomial-time algorithm (CompatibleSpines) for constructing a common bisection tree prefix for three redistricting chambers, grounded in the GCD of the three seat counts. The theoretical contribution is modest in scope — the GCD computation is textbook, and the factorization spine is a straightforward consequence — but the combinatorial structure of the problem has interesting depth that the paper does not fully exploit. The bimodal compatibility distribution is a clean structural result. My main theoretical concerns are: (1) the optimality claim for the spine construction is weaker than it appears; (2) the complexity analysis conflates operation count with algorithmic work; and (3) the paper misses an opportunity to characterize the computational hardness of optimal spine construction when the GCD constraint is relaxed.

---

## Strengths

1. **Proposition 1, part (3) is the right maximality claim.** The proof that gcd(C,S,H) is the longest trunk that can be "guaranteed for any (C,S,H) with the given GCD" is correct. The spine is optimal in the sense that no algorithm can do better given only the divisibility constraint.

2. **Bimodality as a structural theorem.** The gap between sigma=0 and sigma>=50 in US apportionments has a clean number-theoretic explanation (gcd > m/2 implies gcd = m for divisors of small integers), and the paper is on the right track in §4.2. This deserves formal treatment.

3. **The score metric is well-chosen.** sigma = (1 - g/min) * 100 is a natural normalized measure of divisibility distance. The [0,100] range and the exact-zero criterion are both appropriate.

4. **Integration with GeoSection.** Composing CompatibleSpines with GeoSection is the right modular design. The trunk is computed combinatorially; the geographic instantiation is delegated to the geometric algorithm. This separation of concerns is clean theoretical design.

---

## Major Concerns

### M1: The spine construction is not the unique optimal solution

Proposition 1, part (3) claims that the trunk tau is the "longest common prefix that can be guaranteed for any (C,S,H) with the given GCD." This is true for the canonical non-decreasing prime factorization ordering. But there are other common prefixes that share more structure in specific cases.

Consider C=12, S=18, H=8. Then gcd(12,18,8) = 2, so tau = [2]. The paper's spine gives:
- P_C = [2, 2, 3] (12 = 2*2*3)
- P_S = [2, 3, 3] (18 = 2*3*3)
- P_H = [2, 2, 2] (8 = 2*2*2)

All three share the prefix [2]. But now consider the alternative factorizations:
- P_C = [2, 6], P_S = [2, 9], P_H = [2, 4]

These also share [2] as the trunk, but the tails have a different structure. The claim that the non-decreasing factorization gives the "longest" common prefix is true in the literal sense that no common prefix longer than [2] exists. But the paper does not address whether a different ordering might produce a longer shared prefix through "accidental" alignment of tail factors.

More formally: the problem of finding the longest common prefix of two sequences tau1 + tail1 and tau2 + tail2 where tau1 and tail1 are permutations of the prime factors of C and S (respectively) is a combinatorial optimization problem. The paper solves this by fixing the canonical ordering, but does not prove that the canonical ordering maximizes the longest common prefix. It only proves that no common prefix longer than gcd(C,S,H) can exist regardless of ordering.

**Required**: Add a remark clarifying the scope of Proposition 1(3). The trunk is maximal in the sense that no common prefix can have a product exceeding gcd(C,S,H), but within the constraints of prime factorization ordering, different orderings may share longer common subsequences (not prefixes). The paper's construction is one canonical solution; it should not claim to maximize shared structure beyond the GCD-determined trunk length.

### M2: The complexity claim conflates operation count and algorithmic cost

Section 3.5 states that "Algorithm 1 runs in O(log(max(C,S,H))) time (GCD computation plus factorization)." This is correct for the arithmetic operations on the seat counts. But the paper's primary complexity claim is about NestSection (Algorithm 2), and the analysis given is:

"The dominant cost of Algorithm 2 is the g+3 calls to GeoSection, each of which runs in time proportional to graph-partitioning cost."

"Proportional to graph-partitioning cost" is not a complexity bound. What is the graph-partitioning cost? For a census graph with n tracts, METIS-based bisection runs in O(n * k) time for k-way partitioning in the multilevel setting (roughly — the exact complexity depends on the coarsening scheme). The paper needs to state this as a function of n.

More importantly, the claim that "the total additional cost over three independent GeoSection runs is the g trunk-level calls" is misleading. The trunk-level calls each operate on the full graph (n tracts), while the post-trunk calls operate on subgraphs of size n/g. The trunk-level calls are the most expensive per-call, not the cheapest. The "additional cost" is dominated by g calls on the full graph, which for large g could be comparable to the three independent runs.

**Required**: State the complexity of NestSection as a function of n (census tract count) and the seat counts, and compare it properly to three independent GeoSection runs. The current claim that NestSection adds "at most min(C,S,H) additional partitioning operations" is technically true but gives a false impression about the relative cost.

### M3: The hardness of optimal cross-chamber alignment is not addressed

The CompatibleSpines algorithm is optimal given the GCD divisibility constraint. But this assumes that the compatible spine must exactly follow prime factorization. If we relax this — allowing non-divisible spines with a bounded violation tau_pop — the problem of finding the maximum-alignment spine for a given population tolerance becomes a combinatorial optimization problem.

Specifically: given (C, S, H) and a budget tau_pop (maximum fraction of state population in boundary zones), find the spine partition sizes (trunk regions) that maximize cross-chamber alignment measured by some criterion (e.g., maximize the number of exact district nesting pairs). This is the Mode 3 optimization problem, and it is not addressed in the paper.

The paper's Mode 3 is described as "best-effort nesting" with a fixed trunk of size gcd(C,S,H). But for states like Texas (gcd=1), the "best-effort" trunk is trivially size 1. Could a different trunk size (e.g., 2 or 5 regions) provide better alignment with a moderate tau_pop? The paper does not say.

**Required**: Either (a) argue that the GCD-based trunk is also optimal for the approximate (tau_pop > 0) setting, or (b) acknowledge that Mode 3 optimization with a population tolerance budget is an open problem and characterize its difficulty (is it NP-hard? approximable?). For a theory venue, leaving this as a "future work" footnote without characterizing the computational difficulty is insufficient.

### M4: No analysis of the sensitivity of sigma to apportionment perturbations

The compatibility score sigma depends on gcd(C,S,H), which is an extremely sensitive function: gcd(38,31,150)=1, but gcd(38,32,150)=2. A single-seat change in the Texas Senate (31 to 32) would change sigma from 96.8% to 93.75%. This sensitivity is potentially important for the paper's proposals: a small reapportionment change could dramatically alter which nesting mode applies to a state.

The paper mentions in §5.3 ("Reapportionment stability") that Montana's 2020 reapportionment preserved sigma=0. This is the right observation, but it is one data point. A general analysis of sigma's sensitivity to +/-1 apportionment changes across all 50 states would substantially strengthen the evaluation. How many states are "one seat away" from strict compatibility? How many are "one seat away" from dropping to sigma >= 50?

**Required**: Add a sensitivity analysis showing, for each state, the sigma change if C is incremented or decremented by 1 (within the range of plausible reapportionments). This is a simple computation and would significantly enrich the evaluation.

---

## Minor Issues

- The proof of Lemma 1, part (3) states "the maximum attained score for any US state is approximately 96.8% (Texas: g=1, min=31)." The proof then notes "for any US state, min(C,S,H) <= C <= 52, so the score is at most 100(1 - 1/52) ≈ 98.1%." But min(C,S,H) is the congressional count C only when C < S and C < H. For Texas, min = S = 31 (31 < 38 and 31 < 150). The bound "min <= C" is correct since min <= any element, but the argument should be phrased more carefully. The actual bound is min >= 1 (achieved when a chamber has 1 seat), so sigma < 100 always.

- The NestRefine step (§3.4) invokes "boundary zones where a house district straddles a senate boundary." The sentence "they are assigned to the larger-overlap chamber and flagged" contains a grammatical/conceptual ambiguity: "assigned to the larger-overlap chamber" presumably means the house district is assigned to the senate district that contains a larger fraction of its population. This should be stated precisely.

- The paper does not address the uniqueness of the compatible spine. For a given (C, S, H), is the CompatibleSpines output unique? Yes, given the canonical non-decreasing ordering convention. But different orderings of the same prime factors yield different geographic spines with the same combinatorial structure. The paper should state whether uniqueness is desired and whether the canonical ordering achieves it.

- The paper's claim that the additional complexity is "at most min(C,S,H) additional partitioning operations" counts calls, not cost. For a more informative bound, count the total work: the g trunk calls on the full graph plus the per-region calls on subgraphs of size n/g, and compare to the three-independent baseline.

---

## Assessment

The theoretical content of NestSection is largely correct, but the paper underexploits the mathematical structure. The bimodality result deserves a formal theorem; the Mode 3 optimization problem deserves a hardness characterization; the complexity analysis should be stated in terms of n (graph size). The core CompatibleSpines algorithm is sound and the proof of correctness is complete. My concerns are primarily about the depth of the theoretical analysis relative to the stated contributions. For a theory venue, these gaps should be addressed; for a law review, the algorithmic claims should at least be stated carefully. I recommend acceptance with revisions.

**Score: 3/4 — Accept with revisions.**
