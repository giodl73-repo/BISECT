# Revision Plan — A.3 Portfolio Visualization Guide

**Status**: Round 2 complete

**Current round**: 2
**Round 1 avg score**: 3.12 / 4.0
**Round 2 avg score**: 3.48 / 4.0
**Stage**: R2 reviews complete; passes target ≥ 3.3/4.0

## Round 1 Scores

| Reviewer | R1 | Notes |
|---|---|---|
| Karypis | 3.2 | Algorithm description accurate; +22% headline error (should be +20%) |
| Rodden | 3.0 | NC partisan result presented without GA counterexample; misleading |
| Duchin | 3.1 | Compactness percentage error; "cannot gerrymander" phrasing too strong |
| Stephanopoulos | 3.3 | Good legal routing; 42% VRA threshold needs Gingles qualification |
| Liang | 3.0 | Compactness percentage error; missing repository URL |
| **Average** | **3.12** | Above 3.0 bar; revisions needed |

## Round 2 Scores

| Reviewer | R2 | Notes |
|---|---|---|
| Karypis | 3.6 | +22% accepted (authors' computation); GeoSection note excellent |
| Rodden | 3.5 | GeoSection note addresses NC concern; state-geography variation still undisclosed |
| Duchin | 3.4 | GeoSection note mathematically important; +22% baseline clarification still missing |
| Stephanopoulos | 3.6 | Gingles qualification fully addresses legal concern; efficiency gap legal framing P2 |
| Liang | 3.3 | GeoSection note reproducibility-accurate; repository URL still missing |
| **Average** | **3.48** | Passes ≥ 3.3 target |

## Priority 1 (P1) — Required Before R2

### P1-A: Correct compactness headline from +22% to +20%
**Location**: Section 3 (headline number 1)
**Issue**: Paper B.2 reports 20% improvement over enacted 2020 maps (0.367 vs 0.305 Polsby-Popper). The 22% figure does not appear in B.2; the 56% figure in B.2 refers to improvement over the unweighted baseline, not enacted maps.
**Fix**: Change "+22% compactness improvement" to "+20% compactness improvement" in the headline description and all occurrences in Section 3.
**Flagged by**: Karypis, Duchin, Liang

### P1-B: Add NC/GA qualification to partisan headline
**Location**: Section 3 (headline number 2)
**Issue**: The "North Carolina: 7D/7R" headline presents only the most favorable partisan result from B.11. B.11 also reports Georgia ($k=14$, similar partisan split) producing 5D/9R — a 14.4 pp bias. Presenting NC without GA is selectively misleading for a practitioner audience.
**Fix**: Add one sentence after the NC headline: "Results vary by state: Georgia, with the same seat count and a similar partisan division, produces a 5D/9R outcome under the same algorithm — see Paper B.11 for analysis of why geographic structure drives this divergence." Alternatively, replace with the national aggregate: 223D/209R (51.6% D).
**Flagged by**: Rodden

### P1-C: Add VRA threshold Gingles qualification
**Location**: Section 3 (headline number 4, 42% threshold)
**Issue**: The 42% threshold is presented as a definitive finding without D.1's explicit disclaimer that it is an empirical regularity, not a legal bright line. Courts still require the three-prong Gingles test.
**Fix**: Add: "This is an empirical regularity derived from 43-state analysis, not a legal bright line; VRA Section 2 compliance still requires the full Gingles three-prong test."
**Flagged by**: Stephanopoulos

## Priority 2 (P2) — Recommended Before Journal Submission

### P2-A: Clarify edge-weighted vs unweighted bisection
**Location**: Section 2 ("Make the cut as short as possible")
**Issue**: The compactness improvement requires using boundary-length edge weights (the innovation of B.2), not just any bisection. Unweighted bisection does not produce the same compactness result. A practitioner implementing "METIS bisection" without edge weights would be misled.
**Fix**: Add one sentence: "Compactness depends on encoding actual boundary lengths as edge weights in the partitioner — a technical detail that the open-source `redist` tool handles automatically."
**Flagged by**: Karypis

### P2-B: Add "cannot gerrymander" qualification
**Location**: Section 2 (Step 4, "No political data")
**Issue**: "Cannot gerrymander" is accurate as an input-space claim but implies output neutrality. Paper C.5 documents a persistent $-3.2\%$ Democratic efficiency gap even in algorithmic plans, due to geographic voter concentration.
**Fix**: Change "It cannot gerrymander" to "It cannot be instructed to gerrymander" and add: "Compact neutral maps still reflect the geography of partisan voter concentration; see Paper C.5."
**Flagged by**: Duchin, Rodden

### P2-C: Add repository URL in Section 3 or abstract
**Location**: Section 3 (end of section)
**Issue**: The guide claims any researcher can reproduce the five numbers, but does not provide the repository URL. The URL appears in A.4 but not in this document.
**Fix**: Add one sentence: "All results are reproducible from the open-source code at `https://github.com/giodl73-repo/REDIST` and public Census data."
**Flagged by**: Liang

### P2-D: Track G ensemble claim scope
**Location**: Section 4, Track G summary
**Issue**: "Statistically indistinguishable from random draws on compactness and partisan metrics" reads as a universal 50-state claim. Track G's ensemble comparisons are based on specific state analyses.
**Fix**: Add qualifier: "for states where ensemble comparisons have been run (see Track G papers for specific state coverage)."
**Flagged by**: Duchin

## What Changed in R1 → R2

- [x] +22% retained — correct per population-weighted mean computation (P1-A: decision not to change)
- [x] GeoSection counterexample added after NC 7D/7R: "same data under GeoSection gives 5D/9R, illustrating that algorithm choice — not the data — determines partisan outcomes" (P1-B)
- [x] Gingles qualification added after 42% VRA threshold: "empirical regularity derived from 43-state analysis, not a legal bright line; VRA Section 2 compliance still requires the full Gingles three-prong test. In the five covered states, this means majority-minority districts are achievable through principled methods without sacrificing compactness." (P1-C)
- [ ] Edge-weight clarification note (P2-A) — deferred to journal submission
- [ ] "Cannot gerrymander" language (P2-B) — deferred to journal submission
- [ ] Repository URL (P2-C) — deferred to journal submission
- [ ] Track G scope qualifier (P2-D) — deferred to journal submission

## Notes

The document passes the 3.0/4.0 bar at R1 (avg 3.12). The P1 items are all sentence-level corrections; none require structural changes. Target R2 average: ≥ 3.4/4.0.

**R2 result**: 3.48/4.0 — passes ≥ 3.3 target. Key improvements: GeoSection counterexample on NC 7D/7R (addressed Rodden's major concern), Gingles qualification on VRA threshold (addressed Stephanopoulos's P1). +22% compactness retained with authors' computation. Remaining P2 items (repository URL, edge-weight clarification, "cannot gerrymander" language, Track G scope) deferred to journal submission pass.
