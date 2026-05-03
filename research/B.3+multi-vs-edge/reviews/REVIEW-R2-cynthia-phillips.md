# Review Round 2: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Reviewer**: Cynthia A. Phillips (Sandia National Labs)
**Expertise**: Combinatorial optimization, approximation algorithms, experimental algorithm design
**Round**: 2 (Revision Review)
**Date**: 2026-05-02

## Summary of Revisions Received

The authors revised the paper addressing four P1 items I flagged: the multi-constraint implementation, the theoretical section errors, the asymmetric configuration counts, and the absence of statistical rigor. I assess each item and describe what still requires attention.

## P1 Resolution Assessment

### P1-1: Multi-Constraint Implementation — RESOLVED

The implementation bug is confirmed and corrected. Corrected results (30.0% success vs the buggy 35.0%) strengthen the conclusion. I accept this resolution. The authors have done the right thing: confirmed the bug, corrected it, and re-ran. That the result moves in a direction that *supports* the paper's claim, rather than undermining it, is scientifically satisfying.

### P1-2: Theoretical Section — RESOLVED

Section 3.1.2 is rewritten with the formal tightness definition (τ_c = ε) and the 60–800× ratio quantification. The confusing percentage-vs-ratio calculation is gone. I asked either for real theorems or an honest reframing as "informal analysis," and the authors have done the latter coherently. This is an acceptable choice.

### P1-3: Asymmetric Configuration Counts — RESOLVED

The 140-vs-140 balanced comparison is in place. Fair comparison: MC 35.7% vs EW 47.9%. I accept this. I specifically note the pattern of *complete failure* across all 28 MC configurations in Alabama and Louisiana — this is the most interesting result in the paper and validates the claim more convincingly than the aggregate percentage.

### P1-4: Statistical Rigor — RESOLVED

Section 5.6 is a genuine contribution. Wilson CIs (MC: [28.3%, 43.9%], EW: [39.8%, 56.1%]) do not overlap at the 90% level; χ²(1)=4.243 p=0.039 is significant at α=0.05. The 30-seed per-state variance table (SD=0 across all states) answers my core question: variance across seeds is zero, so single-run results are exact. The Phase 2 multi-seed population estimates bound the zero-success state probability at 2.7% upper end. This is thorough.

## Remaining Concerns

### P2-1: Reproducibility Still Incomplete

I remain unsatisfied with implementation details. The revised paper improves the description, but I still cannot reproduce the exact experiment from the paper alone. Required for final acceptance:
- Exact METIS version (5.1.0? 5.2.1?)
- Complete command-line templates for both methods
- Hardware and OS specification
- Explicit description of how adjacency graphs are constructed (rook? queen? distance?)

This is a minimum reproducibility bar for SIAM or INFORMS venues.

### P2-4: No Maps, No Polsby-Popper

I flagged this as P2 (important, not blocking), and it remains unaddressed. The paper reports a 10% average edge-cut penalty for edge-weighted and a 48% penalty for Louisiana. Without geographic visualizations and Polsby-Popper scores, I cannot assess whether these penalties are legally or practically meaningful. At minimum, add one Alabama map and compute Polsby-Popper for the best configurations. This is "should address" level but its continued absence is conspicuous for a redistricting paper.

### Notation Inconsistency (P3-7)

The paper still mixes "12.1 pp," "+12.1 pp," and "12.1 percentage point" in different locations. This is a copyedit-level fix but should be done before submission.

### p=0.039 Framing

The abstract currently emphasizes the config-level significance test. I would prefer the abstract lead with the state-level result (80% vs 40%, zero MC success in AL/LA across 28 parameter values), which is the stronger evidence. The χ² test is meaningful context, not the headline. I suggest restructuring the abstract accordingly.

## Score and Recommendation

**Score: 3/4 — Accept with Minor Revisions**

Round 1 concerns have been addressed in substance. The statistical section (5.6) is well-executed and directly answers my methodological objections. The implementation correction and balanced experimental design are credible improvements. Remaining issues are all minor-revision territory: reproducibility details, maps/compactness scores, notation cleanup, and abstract restructuring.

I do not require further re-review if the authors add a complete reproducibility appendix, add Polsby-Popper for best-config districts, and restructure the abstract to lead with state-level evidence. These are achievable in a final revision pass.

**Verdict**: Accept with minor revisions.
