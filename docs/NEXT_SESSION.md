# Next Session Handoff: B.7 Paper — Deep Convergence + Fiedler Bounds

**Status:** active session pickup pointer
**Owner:** maintainer (you)
**Last update:** 2026-05-01 (extended convergence sweeps + Fiedler approximation ratios)

## Convergence Status (all confirmed states as of session end)

| State | k | MEC (km) | D/R | Gap (pp) | Seeds | Tail | Status |
|-------|---|----------|-----|---------|-------|------|--------|
| VT | 1 | 0 | 1D | +31.7 | 200 | — | trivial |
| PA | 17 | 2,441 | 8D/9R | −3.5 | 1,100 | 819 | CONVERGED |
| GA | 14 | 2,546 | 7D/7R | −0.1 | 1,000 | 511 | CONVERGED |
| MI | 13 | 2,098 | 7D/6R | +2.4 | 500 | 319 | CONVERGED |
| TN | 9 | 1,568 | 1D/8R | −27.1 | 1,000 | 391 | CONVERGED |
| MN | 8 | 1,357 | 3D/5R | −16.1 | 1,000 | 551 | CONVERGED |
| NC | 14 | 2,400 | 5D/9R | −13.6 | 200 | 158 | CONVERGED |
| CT | 5 | 361 | 5D/0R | +39.8 | 500 | 316 | likely |
| AL | 7 | 1,716 | 0D/7R | −37.1 | 500 | 326 | likely |
| ID | 2 | 502 | 0D/2R | −34.1 | 500 | 309 | likely |
| WI | 8 | 1,689 | 2D/6R | −25.3 | 200 | 86 | marginal — needs more |
| TX | 38 | 8,176 | 15D/23R | −7.7 | ~350 | ~236 | running (b7tx_ prefix) |

## Fiedler Approximation Ratios

`scripts/b7_fiedler_bounds.py` computes λ₂ from adj.bin via deflated power iteration.
Ratio = MEC / (λ₂ × n/4). Near 1.0 = certifiable.

| State | Ratio | Interpretation |
|-------|-------|---------------|
| MI | **1.01×** | At the lower bound — fully certifiable |
| NC | 2.42× | Room to improve; needs more seeds |
| PA | 2.67× | Ditto |
| GA | 2.98× | Ditto |
| WI | 3.20× | High gap — needs many more seeds to certify |
| TN | 3.68× | Highest gap — sorted state with few districts |

**Key insight**: sorted states have higher ratios because their graphs have lower algebraic
connectivity (easier to cut). Mixed-geography states (MI) approach the bound.

## Key Empirical Findings (session)

1. **False floors are common**: GA (seed 190→288→489), TN (seed 182→334→609)
   both had incorrect interim results overturned by deeper sweeps.

2. **MI at 1.01×**: Michigan's MEC plan is provably within 1% of the theoretical
   geometric minimum. This is the paper's strongest Fiedler certificate example.

3. **50-state pattern**: near-proportional mixed states (GA −0.1, MI +2.4, PA −3.5),
   sorted R-favoring (TN −27, WI −25, MN −16), structural sweeps (CT +39.8).

4. **CA failure rate**: 64% of seeds fail for CA (52D) with flat 0.5% tolerance.
   TX: ~12% failure rate. Tiered tolerance schedule needed for large states.

5. **TN true MEC = 1D** (not 2D as in 200-seed sweep). 200 seeds was insufficient.

## Review Status

Round 1 complete (avg 2.8/4). P1 items A-E all addressed in this session.
Stage: synthesis → revision (P2/P3 items remain).

## Immediate Next Steps

1. **Run WI to 1,000 seeds** — only 86-seed tail; needs deep convergence for paper claims.
   `$ver = "b7_WI_s{seed}"; seeds 201..1000` using existing `b7_WI_s*` directories.

2. **Add TX to Fiedler bounds script** — once TX converges (seeds running in b7tx_)
   with results in `outputs/b7tx_s*/2020/states/texas/`.

3. **Wire Fiedler certificate into CompactBisect output** — add to manifest JSON.

4. **P2-A: Use exact TIGER perimeters** instead of 2√(πA) circular approximation.
   Reviewer Steinhardt flagged this as an attack surface; exact values are available.

5. **Run CompactBisect on confirmed focal states** with release binary.
   Currently only done with debug binary (Reviewer Liang P1.2).

6. **Write conclusion section** (§6 is still just a 4-line stub).

## Files of note

- `scripts/b7_fiedler_bounds.py` — Fiedler computation for all confirmed states
- `scripts/b7_sweep.ps1` — 50-state MEC sweep (resumable CSV)
- `research/publications/solution-space-and-seed-sensitivity/` — full paper
  - `reviews/SYNTHESIS.md` — P1/P2/P3 items from Round 1
  - `_panel.yaml` — stage=synthesis, avg=2.8/4
- `outputs/b7_sweep/convergence.csv` — 50-state 200-seed summary
- `outputs/b7_{STATE}_s{N}/` — per-state per-seed outputs (b7_PA_s1..1100, etc.)
- `outputs/b7tx_s{N}/` — TX fresh sweep (avoids stale manifests)
