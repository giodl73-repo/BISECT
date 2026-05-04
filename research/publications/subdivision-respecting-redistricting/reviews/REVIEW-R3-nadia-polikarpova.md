> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review — Nadia Polikarpova (R-37)
**Score: 2.0 / 4**

## Summary
The paper proposes encoding governmental hierarchy as a multiplicative edge-weight signal for METIS-based redistricting, with a compositional framework that stacks subdivision levels via function composition. The central monotonicity claim is the load-bearing theoretical result. The hierarchical extension to multi-level subdivision is a natural generalization.

## Strengths
- **Clean compositional design.** The framework w* = f_k ∘ ⋯ ∘ f_1(w_geo) is elegant and separates concerns correctly. Each layer is independently parameterized, and composition preserves non-negativity of edge weights.
- **Operationally motivated formulation.** The multiplicative form f_α(u,v) = w(u,v)·(1+α·1[…]) scales relative to existing edge weight rather than imposing an absolute floor, which matters when base weights span orders of magnitude.
- **Hierarchical extension is coherent.** The multi-level formula Σ_ℓ α_ℓ·1[level_ℓ(u)=level_ℓ(v)] correctly treats levels as independent signals. The additive structure inside the multiplicative envelope is dimensionally consistent.

## Weaknesses
1. **The Monotonicity Proposition is wrong as stated.** The proof argument "any partition feasible under f_α is feasible under f_α' for α'<α" is a statement about the feasible set, not about METIS outputs. METIS is a multilevel k-way heuristic — it does not return the globally optimal min-cut partition. Increasing α can cause METIS to navigate a different neighborhood and land on a partition with *more* county splits. The proposition establishes a claim about exact min-cut; it says nothing about METIS trajectories. Either restrict the claim to exact min-cut or replace with empirical evidence.
2. **"Not vice versa" is unearned.** The proof's asymmetry argument tries to show strict feasibility containment but provides no such argument even for exact min-cut.
3. **No stability analysis for the compositional stack.** When f_k ∘ ⋯ ∘ f_1 is applied, the effective weight on an intra-county, intra-municipality edge is w_geo·∏(1+α_ℓ). For k=3 levels and α_ℓ=0.5, this is 3.375× amplification. The paper does not characterize when this overwhelms population-balance penalties.
4. **county(·) function domain underspecified.** The paper assumes an unambiguous map from census units to counties. Some tract geometries straddle county boundaries; this precondition should be stated.

## Detailed Comments
Proposition label says "weakly decreasing" but the proof implicitly assumes strict containment in the "not vice versa" clause. The composition operator ∘ needs a type signature. Empirical validation (NE, NC, etc.) is noted in context but not integrated into the paper's formal claims; if monotonicity is ultimately empirical, say so with standard errors across seeds.

**Score: 2/4** — The compositional framework is a genuine contribution. However, the Monotonicity Proposition — the paper's only formal theorem — does not survive contact with the fact that METIS is a heuristic. Acceptance requires a corrected proof scoped to exact min-cut, or replacement with rigorous empirical monotonicity results.
