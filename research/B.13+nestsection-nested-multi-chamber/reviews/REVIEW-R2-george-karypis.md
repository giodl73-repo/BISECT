> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: NestSection — George Karypis (Round 2)

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: Graph partitioning, multilevel algorithms, METIS, tree structures, sparse linear solvers
**Round**: 2
**Score**: 3.5/4 (accept with minor revisions)
**Recommendation**: Accept with Minor Revisions

---

## Response to Round 1 Concerns

The authors have addressed my four major concerns with varying completeness. I am upgrading my score.

### M1: Trunk ordering (canonical vs. geographic) — Addressed

The paper now contains a clear acknowledgment in §3.2 that the canonical non-decreasing ordering is a convention, and that different orderings of the same prime factors produce distinct geographic spines. The Remark after the Bimodality Gap theorem situates this correctly as a degree of freedom. The paper correctly identifies best-ordering as an open optimization problem. This resolves M1 as I stated it: the paper no longer conflates combinatorial and geometric canonicity.

What remains as a minor gap: the paper still does not suggest any criterion for choosing among orderings (e.g., minimize inter-trunk edge cut, maximize geographic compactness of trunk regions). This would be helpful for a practitioner implementing NestSection. A sentence in §3.5 or §5.1 pointing to this as an open problem with a specific candidate criterion would complete the treatment.

### M2: Seat allocation uniformity across trunk regions — Partially addressed

The paper now explicitly distinguishes trivially compatible ($C=1$), weakly compatible ($C=2$), and non-trivially compatible ($g \geq 3$) states in §4.2. For Oregon ($C=6$, $g=6$) and Alabama ($C=7$, $g=7$), the paper correctly states that $c_i = C/g = 1$ for every trunk region — so the uniformity concern is moot for the two substantive cases. Algorithm 2 line 6 sets $s_i = S/g$ uniformly, which is exact when $g \mid S$ and $g \mid H$, as holds in both substantive cases.

However, the bridge argument I requested — connecting population balance to uniform seat allocation — is still missing. The paper notes that for Mode 1 states $g = C$ which implies exact divisibility, but does not state the logical dependency chain: (a) $g = \gcd(C,S,H)$ implies $C/g$, $S/g$, $H/g$ are all integers; (b) Algorithm 2 assigns $s_i = S/g$ uniformly across trunk regions, which presupposes population balance has been achieved exactly; (c) ApportionRegions (line 5) may assign $c_i \in \{C/g, C/g+1\}$ for some regions depending on population. For Oregon with $C/g = 1$, this is trivially resolved. The paper should say so explicitly in a short remark attached to Algorithm 2.

### M3: Empirical timing data — Not addressed

The paper still contains no timing benchmarks comparing NestSection to three independent GeoSection runs. The complexity claim in §3.6 remains at the level of operation counting. The paper adds geographic description for Oregon and Alabama (§4.3) but no timing data. For the final publication, at least one sentence quantifying the trunk-level GeoSection cost relative to the per-trunk tail calls would be valuable. I accept that this may be deferred to future empirical work.

### M4: NestRefine specification — Partially addressed

The NestRefine paragraph in §3.5 now states that non-integer $h_i/s_i$ triggers floor-or-ceiling assignment by population weight. The boundary-zone assignment ("assigned to the larger-overlap chamber and flagged") is present. However, NestRefine is not formalized as Algorithm 3 as I requested. For the two substantive cases (Oregon and Alabama), $h_i/s_i = 10$ and $h_i/s_i = 3$ respectively — both integers — so NestRefine reduces to a clean recursive GeoSection call with no boundary zones. A parenthetical remark noting this simplification for Mode 1 states would suffice, avoiding the need for a full Algorithm 3 in the current paper.

### New contribution: Bimodality Gap Theorem (Theorem 3)

The addition of Theorem 3 is the most significant change in this revision. The proof is correct and complete: if $g \mid m$ and $g > m/2$, then $m/g < 2$, forcing $m/g = 1$ (since it is a positive integer), thus $g = m$ and $\sigma = 0$. This elevates what was an empirical observation to a universal number-theoretic invariant. The boundary cases discussion (m=1 and the infimum at $\sigma = 50$) is well handled. The Remark correctly emphasizes that this holds for all positive integers, not just current US apportionments. This is the right framing.

Minor note: Theorem 3 would benefit from a title in the theorem environment ("Bimodality Gap") rather than just "Theorem 3," to aid navigation. The authors may also want to note that the theorem implies no Mode 2 state has ever existed or could exist for any apportionment, not just current ones — which retroactively validates the Mode 2 classification as a purely definitional category.

---

## Remaining Concerns

### Remaining M2 fragment: Algorithm 2 uniformity remark

As noted above, a short remark in or after Algorithm 2 should state: "In Mode 1 states ($g = C$), $C/g = 1$, so every trunk region receives exactly one congressional seat. Population heterogeneity at the trunk level is handled by GeoSection's ratio-optimal bisection (B.8), which ensures trunk regions have approximately equal populations. ApportionRegions may assign varying seat counts when $C/g > 1$; the tail factorization within each region accommodates this." This takes three sentences and resolves M2 completely.

### Oregon case study: Geographic description but still no map

The new §4.3 provides a clear geographic spine description for Oregon (Cascade Range east-west bisection, latitudinal divisions, 6 trunk regions) and Alabama (Black Belt and north-south population corridors, 7-way direct partition). The figure environments (Fig. 1 and Fig. 2) are schematic descriptions rather than actual maps. I understand that the empirical pipeline run is deferred, and the schematic descriptions are informative. This is acceptable for a first paper on this topic, provided the figure captions make clear that these are schematic illustrations awaiting pipeline execution. The current captions do state this. I would prefer actual maps, but I no longer require them as a condition for acceptance.

### Minor: O(sqrt) vs. O(log) complexity for factorization

The complexity claim in §3.6 states Algorithm 1 runs in $O(\log(\max(C,S,H)))$. As I noted in Round 1, trial division for prime factorization runs in $O(\sqrt{\max(C,S,H)})$. For seat counts up to 52, this distinction is trivially irrelevant in practice. But the claim should read $O(\sqrt{\max(C,S,H)})$ for correctness, or the paper should specify that it uses a GCD-only computation without explicit factorization (which is possible since the trunk length equals the number of prime factors of $g$, computed via Euler's totient or log). Either fix is acceptable.

---

## Assessment

The revision substantially strengthens the paper. The Bimodality Gap Theorem converts the most interesting empirical observation into a universal mathematical fact. The trivial/weak/non-trivial stratification of the 11 compatible states is the correct framing, and the Oregon and Alabama geographic spine descriptions give the case studies the geographic grounding they lacked in Round 1. My remaining concerns are minor: a bridge argument remark for Algorithm 2, a title for Theorem 3, and a complexity claim correction. None of these require substantial revision.

**Score: 3.5/4 — Accept with minor revisions.**
