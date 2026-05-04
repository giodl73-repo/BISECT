> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review R-3: George Karypis
**Paper**: The Proportionality Compromise
**Date**: 2026-05-03
**Score**: 3.0 / 4

The ncon=2 formulation is technically sound. The closed-form tpwgts derivation correctly encodes proportionality in the balance constraint space. I have not seen this derivation written down before — genuine novelty for the algorithms community.

**Concerns:** Empirical validation is the weakest section. The tradeoff table is analytical, not from actual METIS runs. METIS's multilevel coarsening introduces approximations the formula cannot predict; FM refinement can violate balance constraints for highly irregular graphs. The C×σ bound needs empirical verification.

**P1:**
1. Run actual METIS partitions for 8 competitive states at η∈{0.01, 0.05, 0.10} and report measured vs. formula-predicted gap.
2. Report achieved vs. target balance (ubvec) for each run.
3. Specify METIS version, seed, ncuts, niter parameters used.

**Score: 3.0 / 4** — Accept with minor revisions.
