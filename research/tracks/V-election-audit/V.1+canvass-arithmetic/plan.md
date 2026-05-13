# V.1 - Canvass Arithmetic

**Paper Type:** Focused technical report  
**Status:** Drafting  
**Track:** V - Vote Counting, Certification, And Public Verification  
**Depends On:** `V.0+rcount-overview`

## Research Question

How can a public verifier distinguish legitimate canvass evolution from
arithmetic errors, missing batch evidence, or unexplained changes in public
totals?

## Claims

- **H1:** Status-specific summaries let unofficial and canvassed snapshots both
  replay without erasing the fact that the public claim changed.
- **H2:** Correction and late-batch events are evidence objects, not proof that
  an election was lawful or unlawful.
- **H3:** Jurisdiction totals, contest sums, batch manifests, and source hashes
  give complementary checks that fail at different layers.

## Evidence

- `canvass-correction` fixture: unofficial and canvassed snapshots plus a
  public correction event.
- `mail-batch-added` fixture: accepted/counted/rejected batch accounting and
  batch summary totals.
- `missing-batch` fixture: negative batch-evidence failure.
- `bad-selection-sum`, `tampered-source`, and `missing-source-hash` as adjacent
  failure comparisons.

## Figures and Tables

- Status transition table.
- Canvass correction delta table.
- Batch accounting equations.
- Failure attribution table.

## Panel Readiness Checklist

- [x] Main LaTeX draft exists.
- [x] Uses V.0 terms and boundaries.
- [x] Explains correction without fraud implication.
- [x] Includes positive and negative fixture evidence.
- [ ] Simulated review round completed.

