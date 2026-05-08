---
name: research-pre-write
description: "Pre-writing pipeline for bisect research papers. Reads the accepted spec from docs/specs/, checks algorithm correctness, identifies empirical gaps, verifies claims are testable, then produces a ready-to-write outline. Adapted from RMM research-pre-write for the bisect redistricting research structure."
allowed-tools: [Read, Write, Glob, Grep, WebSearch]
param_set: lean
---

You are running /research-pre-write for: {{topic}}

Run the full pre-writing signal pipeline for a bisect research paper. This reads the accepted
spec, checks algorithm correctness and claim testability, identifies empirical gaps, and
produces a ready spec/outline before writing begins.

---

## PHASE 1 -- FIND THE PAPER AND SPEC

Search for the paper in `research/tracks/`:
```
Glob: research/tracks/**/{{topic}}*
```

Also search for the accepted spec:
```
Glob: docs/specs/*{{topic}}*
```

Extract from the spec:
- **Algorithm**: what does it compute?
- **Claims**: what does the paper claim to prove or show?
- **Layer**: Structure / Search / Standalone
- **Test invariants**: what L0/L1/L2 tests does the spec require?
- **Empirical targets**: NC/WI/TX comparisons, specific metrics

Print:
```
Paper: {{topic}}
Spec: docs/specs/[spec-file] (Status: [Accepted X.X/4 / Not found])
Algorithm layer: [Structure / Search / Standalone]
Key claims: [list]
Empirical targets: [states/metrics]
```

---

## PHASE 2 -- ALGORITHM CORRECTNESS CHECK

Read the implementation in `crates/bisect-cli/src/bisection_runner.rs` (or relevant crate).
Compare the spec pseudocode against the actual Rust implementation.

Check:
1. Does the implementation match the spec algorithm step-by-step?
2. Are there edge cases in the spec not handled in the implementation?
3. Does the seeding formula in the spec match the code?
4. Are there any mathematical claims in the spec that need verification?

Print:
```
ALGORITHM CHECK:
  Spec vs implementation: [MATCH / MISMATCH]
  Edge cases handled: [N/M]
  Seeding: [CORRECT / INCORRECT]
  Math claims: [VERIFIED / UNVERIFIED items]
```

---

## PHASE 3 -- CLAIM TESTABILITY CHECK

For each major claim the paper will make, assess testability:

| Claim | Evidence type | Available? | Gap |
|-------|---------------|-----------|-----|
| MKA achieves highest Reock | Single-run NC/FL/WA table | Yes (single seed) | Multi-seed needed |
| Runtime O(n×orientations) | Timing measurement | Estimated | Needs benchmark |

Flag claims that:
- Require data not yet generated (L2 tests marked #[ignore])
- Need multi-seed variance not available from single runs
- Make theoretical statements without proofs

Print:
```
CLAIM TESTABILITY:
  Fully supported claims: [N]
  Single-run only (needs hedge): [N]
  Unverifiable without Phase 2: [N]
  Blocking gaps: [list]
```

---

## PHASE 4 -- LITERATURE GAP CHECK

For each algorithm/paper, check:
1. Is the primary citation correct and findable?
2. Are there competing approaches we should cite?
3. Are legal/court citations current and accurate?

```
LITERATURE:
  Primary citations: [verified / needs check]
  Missing competitors: [list]
  Legal citations: [verified / needs check]
```

---

## PHASE 5 -- COHERENCE VERDICT

Based on Phases 2-4, produce a PROCEED / PAUSE / PIVOT verdict:

**PROCEED**: All major claims testable, algorithm correct, no blocking gaps.
→ Paper can be written now.

**PAUSE**: Some claims need hedging (single-run dagger notation), or minor gaps.
→ List specific items to address in the paper with dagger notation and caveats.

**PIVOT**: Major algorithm mismatch, or core claims not testable at all.
→ Paper design needs rethinking.

---

## PHASE 6 -- READINESS REPORT

```
═══════════════════════════════════════════════════════
PRE-WRITE COMPLETE: {{topic}}
═══════════════════════════════════════════════════════

Spec status: [Accepted X.X/4 / Draft]
Algorithm: [CORRECT / ISSUES]
Claims testable: [N of M] ([M-N] need dagger notation)
Literature: [READY / N items to check]

VERDICT: [READY TO WRITE / WRITE WITH CAVEATS / PAUSE]

Required caveats for paper:
1. [single-run claim: use dagger notation]
2. [missing comparison: note as future work]
3. [theoretical claim: add proof sketch or cite]

Recommended section structure:
§1 Introduction — [key framing]
§2 Background — [what to cover]
§3 Algorithm — [what to prove/describe]
§4 Comparison — [what states/metrics]
§5 [Legal/Applications] — [if applicable]
§6 Conclusion
═══════════════════════════════════════════════════════
```
