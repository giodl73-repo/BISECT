# Next Session Handoff: B.7 Paper — Round 2 Review Complete

**Status:** active session pickup pointer
**Owner:** maintainer (you)
**Last update:** 2026-05-01

## Paper Status

**Round 2 review: 3.4/4 — ACCEPT with minor revisions.**
Stage: `recheck` → advance to `ready` after P2 items.

### Convergence Data (all confirmed)

| State | Seeds | MEC (km) | Plan | Gap (pp) | Tail | Status |
|-------|-------|----------|------|---------|------|--------|
| PA | 1,100 | 2,441 | 8D/9R | −3.5 | 919 | CONVERGED |
| GA | 1,000 | 2,546 | 7D/7R | −0.1 | 511 | CONVERGED |
| MI | 500 | 2,098 | 7D/6R | +2.4 | 319 | CONVERGED |
| TN | 1,000 | 1,568 | 1D/8R | −27.1 | 391 | CONVERGED |
| MN | 1,000 | 1,357 | 3D/5R | −16.1 | 551 | CONVERGED |
| NC | 200 | 2,400 | 5D/9R | −13.6 | 158 | CONVERGED |
| TX | 400 | 8,176 | 15D/23R | −7.7 | 286 | CONVERGED |
| CT | 500 | 361 | 5D/0R | +39.8 | 316 | likely |
| WI | 1,000 | 1,615 | 3D/5R | −12.8 | 108 | partial — needs 300 |

**WI needs ~200 more seeds** to reach the 300-seed certification threshold.

### Key Empirical Findings (for the paper)

1. **False floors**: GA (overturned seed 489), TN (seed 609), WI (seed 318). 200 seeds is NOT enough.
2. **MEC and GMPP agree on outcome** when converged (WI/NC both 3D/5D). PA is the exception (MEC 8D vs GMPP 7D).
3. **MEC is the correct primary criterion**: no interpretive degrees of freedom, TIGER-only inputs.
4. **GMPP exact perimeters**: approximate (circular model) gave misleading convergence story for WI. Exact (TIGER pyproj EPSG:5070) shows GMPP last improved seed 930, not 109.
5. **National totals**: MEC gives 229D/206R algorithmically vs enacted 222D/213R (+7 Dem).
6. **Fiedler bound non-certifying**: λ₂ = 7-46m for all states, bound/achieved ≈ 2%. Cheeger is too loose for geographic graphs. Empirical tail criterion is the practical certification.

### Paper Sections Status

| Section | Status |
|---------|--------|
| §1 Introduction | Complete |
| §2 Background | Complete |
| §3 Methodology | Complete (Fiedler tightness note, ε statutory, content-derived seed) |
| §4 Evaluation | Complete (1100-seed PA table, WI GMPP caveat, national totals) |
| §5 Discussion | Complete (MEC primacy, geometric sorting theorem, WI caveat) |
| §6 Conclusion | Complete |

### Remaining P2 Items (before submission)

| Item | Description | Effort |
|------|-------------|--------|
| P2-I | Definition 3.3 practical annotation (non-certifying in practice) | 1 paragraph |
| P2-II | CompactBisect with release binary (currently debug binary results) | 1h compute |
| P2-III | ε sensitivity analysis (show outcome stable for ε ∈ {0.01,0.02,0.05,0.10}) | 2h compute |
| P2-IV | Explicit MEC primacy justification vs GMPP | 1 paragraph |
| P2-V | Fix O(n²) Lanczos → O(n×k) ARPACK complexity claim | 1 line |
| P2-VI | AK/HI election result citations (Don Young 54.1%, HI both D) | 1 footnote |

WI needs 200 more seeds for certified tail → run b7_WI_s1001..1200.

## Analysis Scripts (all working)

- `scripts/b7_sweep.ps1` — 50-state MEC sweep (resumable)
- `scripts/b7_fiedler_bounds.py` — λ₂ via scipy ARPACK (correct)
- `scripts/b7_audit_math.py` — level-1 EC, Cheeger h(G) for all states
- `scripts/b7_exact_perimeters.py` — exact TIGER perimeters via pyproj
- `scripts/b7_gmpp_exact.py` — GMPP convergence with exact perimeters
- `scripts/b7_pa_table.py` — PA 1100-seed distribution

## Rust Codebase Status

The other session added `AlgorithmParams` enum refactor (commit 3903c62):
- `partition_mode: String` replaced by `algo: AlgorithmParams` enum
- Each variant carries only its own params (CompactBisect has seeds_per_level, epsilon)
- `from_state_args()` / `defaults_for_mode()` constructors
- All 115 tests passing

CompactBisect, GeoSection (B.8), Proportional all in AlgorithmParams.
Check runner.rs for dispatch — no more string matching on partition_mode.

## Key Paper Conclusions

1. **Seed irrelevance** (for certified states): convergence tail ≥300 seeds certifies the MEC plan is stable regardless of which specific seed finds it.
2. **Geometric sorting** drives proportionality gaps — not algorithmic bias. Any compact criterion produces the same structure.
3. **MEC is the correct primary criterion**: TIGER boundary lengths only, no interpretive choices.
4. **GMPP and MEC agree on outcome** when both converge — GMPP doesn't make things "more proportional," just reaches the same place via different path.

## See Also

- `research/publications/solution-space-and-seed-sensitivity/reviews/SYNTHESIS-R2.md`
- `research/publications/solution-space-and-seed-sensitivity/_panel.yaml` (stage: recheck, avg: 3.4)
