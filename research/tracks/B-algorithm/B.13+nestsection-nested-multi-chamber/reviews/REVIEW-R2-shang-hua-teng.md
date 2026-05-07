> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: NestSection — Shang-Hua Teng (Round 2)

**Reviewer**: Shang-Hua Teng (USC)
**Expertise**: Theoretical computer science, spectral graph theory, algorithmic game theory, smoothed analysis, combinatorial optimization
**Round**: 2
**Score**: 3.5/4 (accept with minor revisions)
**Recommendation**: Accept with Minor Revisions

---

## Response to Round 1 Concerns

The revision addresses my bimodality concern decisively (M2 analog / the theoretical formalization item) and makes honest progress on the scope clarifications. M1, M3, and M4 are partially or not addressed, but I am upgrading my score because the Bimodality Gap Theorem is the most theoretically significant change and it is well-executed.

### Bimodality as a formal theorem — Fully resolved

The Bimodality Gap Theorem (Theorem 3) is the change I was most hoping for, and it is correctly stated and proved. The proof is concise, the boundary case analysis is complete, and the Remark correctly flags that this is a universal number-theoretic fact rather than an empirical coincidence. The proof handles the $m=1$ boundary case and correctly computes the infimum of the non-zero range as exactly 50 (achieved when $g=1$, $m=2$). This is tight and correct.

The placement in §3.3 (after the score definition, before the nesting modes) is exactly right. The theorem is cited in §4.4 to explain the empirical gap in the distribution. This is how a good theorem should flow through a paper.

One note on proof structure: the proof says "if $g \mid m$ and $g > m/2$, then $m/g < 2$, so $m/g = 1$ (it must be a positive integer)." This is correct but the parenthetical should say "it must be a positive integer since $g \mid m$ by definition of GCD" — the GCD divisibility of $m$ is the key step that forces $m/g$ to be a positive integer rather than an arbitrary real. A one-word addition ("integer divisor") would make this self-contained without requiring the reader to recall that GCD implies divisibility.

### M1: Spine construction optimality scope — Not addressed

My Round 1 concern was that Proposition 1(3) claims the trunk is the "longest common prefix that can be guaranteed," which is true for the canonical non-decreasing ordering but does not account for the possibility that alternative orderings of the same prime factors could produce longer common subsequences (not prefixes) between pairs of spines.

The revision does not add the Remark I requested clarifying this scope. The paper still states "the trunk $\tau$ is the longest common prefix that can be guaranteed for any $(C,S,H)$ with the given GCD" without noting that "prefix" is relative to the canonical ordering convention and that different orderings may share longer structure in specific cases. For a theory venue this remains imprecise.

I am not escalating this to a required fix for the current round because: (a) the canonical ordering is explicitly stated, (b) the paper's Karypis-raised M1 remark about different orderings producing different geographic spines partially covers this ground, and (c) the distinction between common prefix and common subsequence is a nuance that may be more appropriate for an appendix or a companion theory paper. But I note it again for the record: the word "prefix" in Proposition 1(3) is doing more work than it appears, and a reader familiar with string matching will notice.

### M2: Complexity analysis — Not addressed

The complexity claim in §3.6 still reads "at most $\min(C,S,H)$ additional partitioning operations" without stating the complexity as a function of $n$ (census tract count). The dominant cost of trunk-level GeoSection on the full graph ($n$ tracts) is not compared to the three independent post-trunk calls on subgraphs of size $\approx n/g$.

For the two substantive cases (Oregon, Alabama), $g = 6$ and $g = 7$ respectively. The trunk cost is $g$ calls on the full graph; the post-trunk cost is $3g$ calls on subgraphs of size $n/g$. If GeoSection cost scales as $O(n \log n)$ (a reasonable approximation for multilevel METIS), the trunk cost is $O(g \cdot n \log n)$ and the post-trunk cost is $O(3g \cdot (n/g) \log(n/g)) = O(3n \log(n/g))$. The total is $O(g \cdot n \log n + n \log n) = O(g \cdot n \log n)$ for $g \geq 1$. For three independent runs: $3 \cdot O(n \log n)$. So NestSection is $O(g/3)$ times slower than independent runs for the trunk-level dominant cost. For $g = 6$ this is a factor of 2; for $g = 7$ it is roughly 2.3. This is not negligible and should be stated.

**Required (minor)**: Add one sentence in §3.6 giving the approximate cost ratio for the substantive cases: "For Oregon ($g=6$) and Alabama ($g=7$), the trunk-level GeoSection calls add approximately a factor of $g/3 \approx 2$ to the total computation relative to three independent GeoSection runs, since the $g$ trunk calls operate on the full graph."

### M3: Mode 3 hardness — Not addressed

The paper still does not characterize the computational difficulty of optimal Mode 3 alignment: given $(C, S, H)$ and a budget $\tau_\text{pop}$, find the trunk partition sizes maximizing cross-chamber alignment. For Texas ($g=1$), the GCD-based trunk is trivially size 1, but alternative non-divisor trunk sizes (e.g., 2 or 5 regions) might provide better alignment with moderate $\tau_\text{pop}$. The paper does not address whether this is tractable or hard.

I am not requiring this for the current round since: (a) the paper's primary claim is about the exact-GCD case; (b) the two substantive states (Oregon, Alabama) are Mode 1, so Mode 3 optimization is not relevant to the main results; and (c) characterizing hardness is a reasonable scope for a companion theory paper. But the Future Work paragraph should explicitly flag this as an open problem with a hardness question: "It is unknown whether the Mode 3 optimization problem — find the trunk partition sizes maximizing cross-chamber alignment subject to a population tolerance budget — is polynomial or NP-hard." This sentence costs nothing and would be useful to the community.

**Required (minor)**: Add the above sentence to the Mode 3 / Future Work discussion.

### M4: Sigma sensitivity analysis — Not addressed

The sensitivity analysis I requested (for each state, how does sigma change if $C$ is incremented or decremented by 1) is not in the revision. This was a "Required" in Round 1. I downgrade it to recommended for Round 2 since: (a) for the two substantive states, the analysis is trivial — Oregon with $C=7$ gives $\gcd(7,30,60)=1$, $\sigma \approx 85.7\%$ (catastrophic), and Oregon with $C=5$ gives $\gcd(5,30,60)=5$, $\sigma = 0\%$ (preserved) — which is an interesting story worth one sentence; (b) Alabama with $C=8$ gives $\gcd(8,35,105)=1$, $\sigma = 87.5\%$ (also catastrophic) — another sentence; (c) this illustrates the fragility of strict compatibility to reapportionment, which the paper discusses conceptually but not quantitatively.

**Recommended**: Add two sentences in the Reapportionment Stability paragraph (§5.3) noting what would happen to Oregon and Alabama if $C$ changed by $\pm 1$: "Oregon's compatibility is fragile: $C=5$ or $C=7$ would both break strict compatibility ($\gcd(5,30,60)=5 \to \sigma=0$ preserved for $C=5$; $\gcd(7,30,60)=1 \to \sigma \approx 85.7\%$ for $C=7$). Alabama faces similar fragility at $C=8$ ($\gcd(8,35,105)=1$) or $C=6$ ($\gcd(6,35,105)=1$)."

---

## New Issues in the Revision

### Proof of Lemma 1(3): min(C,S,H) <= C — Clarification

The Texas example in Lemma 1(3): "Texas: $g=1$, $\min=31$" — and the footnote notes $\min(C,S,H) \leq C \leq 52$. For Texas, $\min(C,S,H) = S = 31 < C = 38$. The bound "$\min \leq C$" holds because the minimum is at most any element, including $C$. The argument should be stated as: "$\sigma = 100(1 - 1/m)$ where $m = \min(C,S,H)$; since $m \geq 1$, $\sigma < 100$ always. For US states, $m \leq C \leq 52$, but the binding constraint is $m \geq 1$. The maximum $\sigma$ attained is $100(1 - 1/31) \approx 96.8\%$ for Texas where $m = S = 31$." This addresses the issue I raised in Round 1 minor notes.

### Non-decreasing ordering convention — Remark missing

Algorithm 1 states $\tau \leftarrow \textsc{PrimeFactors}(g)$ "sorted non-decreasing." The paper should add a remark after Algorithm 1 (or in the Definition) that this convention fixes a canonical representative but that other orderings are equally valid and produce different geographic spines. This connects to Karypis's M1 (trunk ordering) and Teng's M1 (spine optimality scope). A single remark can address both: "The non-decreasing ordering is a canonical convention. Different orderings of the same prime factors produce distinct geographic spines (different geographic cut sequences) with identical combinatorial structure. The choice of ordering is an optimization degree of freedom; we use non-decreasing order as a deterministic baseline."

---

## Assessment

The Bimodality Gap Theorem is the centerpiece of this revision and it is correct, well-placed, and properly developed. The compatible-state stratification resolves the empirical honesty concern. The remaining theoretical gaps — complexity as a function of $n$, Mode 3 hardness characterization, sigma sensitivity for the substantive cases — are minor enough that I am upgrading to accept-with-minor-revisions. The required changes in this round are: (a) one sentence on cost ratio in §3.6; (b) one sentence on Mode 3 hardness as an open problem; and (c) the ordering convention remark after Algorithm 1. All three are cosmetic additions that take fewer than 10 sentences total.

**Score: 3.5/4 — Accept with minor revisions.**
