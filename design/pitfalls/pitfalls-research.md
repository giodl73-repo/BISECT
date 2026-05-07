# Research Pitfalls (RP-01..RP-02)

Structural vulnerabilities in how findings are generated, stated, and maintained. Research pitfalls produce claims that look rigorous but aren't — or claims that were rigorous when written and have since drifted.

---

## RP-01: Threshold sensitivity presented as a point result

**Pattern:** A system operating near a threshold (where small parameter changes flip the outcome) reports its output as a finding without quantifying the sensitivity. Readers interpret the result as stable when it is actually fragile. A slightly different parameter, seed, or data vintage produces a different finding — and neither finding is clearly more correct.

**Domain:** Any analysis near a natural threshold — VRA majority-minority (50%), the 42% minority population threshold, borderline state results. States near the threshold are exquisitely sensitive to edge weight parameters, boost factors, and METIS seed values.

**Why it's harmful:** A point result ("Mississippi achieves 2 MM districts") stated without sensitivity analysis becomes a commitment the system may not keep under future conditions. When the pipeline changes and the result shifts, the paper looks wrong rather than incomplete.

**Structural solution:** For any result near a threshold, report the sensitivity range alongside the point result. "Mississippi achieves 2 MM districts with parameters X; with parameters Y, 1 MM district." The paper's claim is bounded, not a single fragile point.

**Status:** OPEN — Mississippi result (1 MM achieved vs. 2 predicted) reveals this pitfall
**Test:** None yet — needs a parameter sensitivity test for borderline states

---

## RP-02: Claim-to-data drift across pipeline evolution

**Pattern:** A claim is verified against the pipeline at time T. The pipeline evolves (algorithm fix, new version, parameter change). The claim is not re-verified. At time T+N, the claim may no longer match the pipeline's actual output — but it appears in a paper as if it still does.

**Domain:** Any research project with an evolving computational pipeline. Claims made early in development may be invalidated by improvements meant to fix other problems. The same pipeline change that fixes a constitutional violation (e.g., multi-constraint → edge-weighting) may shift minority district counts in borderline states.

**Why it persists:** Re-verifying every claim against the current pipeline requires systematic tooling. Without that tooling, claims accumulate and drift silently. The drift only becomes visible at submission time.

**Structural solution:** `/review-papers` skill cross-references every quantitative paper claim against current pipeline outputs before any submission. Claims that drift receive UPDATE NEEDED status. The coverage table in `design/pitfalls/README.md` tracks which claims have active tests.

**Status:** MITIGATED — `/review-papers` skill exists; not all claims have automated tests yet
**Test:** Partial — VRA state counts checked in `test_vra_compliance.py`
