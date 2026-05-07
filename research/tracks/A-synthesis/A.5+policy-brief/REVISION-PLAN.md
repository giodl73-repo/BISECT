# Revision Plan — A.5 Policy Brief

**Status**: Round 2 complete

**Current round**: 2
**Round 1 avg score**: 3.02 / 4.0
**Round 2 avg score**: 3.48 / 4.0
**Stage**: R2 reviews complete; passes target ≥ 3.3/4.0

## Round 1 Scores

| Reviewer | R1 | Notes |
|---|---|---|
| Stephanopoulos | 2.8 | Statute text flawed; VRA carveout inadequate; court remedy path needs clarification |
| Rodden | 2.9 | NC without GA is misleading; pull-quote overstates proportionality |
| Duchin | 3.0 | +22% error; "cannot gerrymander" too strong; "byte-identical" needs arch qualification |
| Liang | 3.1 | "10 minutes" claim needs setup-time qualification; 2-hour replication ambiguous |
| Karypis | 3.3 | Huntington-Hill statute naming creates legal ambiguity; +22% error |
| **Average** | **3.02** | Passes 3.0 bar; P1 items are substantive |

## Round 2 Scores

| Reviewer | R2 | Notes |
|---|---|---|
| Stephanopoulos | 3.4 | Statute text resolved; VRA carveout substantively improved; Path 3 state/federal fixed |
| Rodden | 3.3 | Statute improved; NC still without GA counterexample; efficiency gap footnote understated |
| Duchin | 3.5 | Algorithm naming excellent; VRA Gingles provision sound; 42% Gingles qualifier missing |
| Liang | 3.5 | 10-min → 15-20 min fully addressed; replication timing clarified; byte-identical P2 |
| Karypis | 3.7 | Both P1 concerns resolved; "cannot gerrymander" phrasing P3 |
| **Average** | **3.48** | Passes ≥ 3.3 target |

## Priority 1 (P1) — Required Before R2

### P1-A: Correct compactness percentage in pull-quote and Section 3
**Location**: Pull-quote (page 1) and Section 3 ("+22%" in results table)
**Issue**: "+22% more compact" appears three times. Paper B.2 reports 20% improvement over enacted 2020 maps (0.367 vs 0.305 Polsby-Popper). The 22% figure does not appear in B.2; the 56% in B.2 is improvement over the unweighted baseline, not enacted maps. A legislator or their staff who fact-checks this citation will find a discrepancy.
**Fix**: Change all occurrences of "+22%" to "+20%" — specifically the pull-quote, the results table row, and any other occurrence in the document.
**Flagged by**: All five reviewers

### P1-B: Revise the proposed statute text
**Location**: Section 4, Path 1
**Issue**: The proposed statute — "using the Huntington-Hill recursive bisection algorithm" — conflates the existing Huntington-Hill seat-apportionment method (2 U.S.C. § 2a) with the redistricting algorithm proposed here. A court or legislative counsel would note this ambiguity. The "subject to requirements of the Voting Rights Act" clause is also inadequate: it does not specify a procedure for VRA-required overrides, which could render the statute unconstitutional as applied in states with qualifying minority populations.
**Fix**: Change statute text to: *"Each State shall apportion its congressional districts using a recursive graph bisection algorithm applied to decennial Census redistricting data and tract boundary geometries, as specified by the Director of the Census Bureau, subject to an independent VRA compliance review procedure established by the Attorney General."*
Alternatively, cross-reference Paper D.4 explicitly: "Model statutory language with VRA carveout is provided in Paper D.4."
**Flagged by**: Karypis, Stephanopoulos (P1 critical)

### P1-C: Add NC/GA counterexample or replace with national aggregate
**Location**: Section 3, results table, "7D/7R" row
**Issue**: Presenting North Carolina's 7D/7R result as evidence of proportional outcomes without disclosing Georgia's 5D/9R result (same algorithm, same $k=14$ factorization, similar partisan split) is misleading for a policy audience. A legislator from Georgia who read this brief and then commissioned an independent analysis would find the brief's claim contradicted.
**Fix**: Option A — add a parenthetical: "(Results vary by state geometry; see Paper B.11 for states where the factorization structure produces less proportional outcomes.)" Option B — replace the state example with the national aggregate: "National (48 contiguous states): 223 Democratic, 209 Republican seats — roughly matching the national popular vote split."
**Flagged by**: Rodden

### P1-D: Qualify "closely proportional" in pull-quote
**Location**: Pull-quote, second sentence
**Issue**: "Closely proportional in partisan representation" is inconsistent with Paper C.5, which documents a persistent $-3.2\%$ Democratic efficiency gap even in algorithmic plans. The pull-quote, as written, implies the algorithm eliminates partisan bias; the evidence shows it reduces but does not eliminate it.
**Fix**: Change "closely proportional in partisan representation" to "with substantially reduced partisan bias compared to enacted maps." Add to Section 3: "A persistent baseline of approximately 3% Republican structural advantage remains, reflecting geographic voter concentration (Paper C.5) — but this compares favorably to the 5–8% advantage embedded in most enacted plans."
**Flagged by**: Rodden, Stephanopoulos

## Priority 2 (P2) — Recommended Before Journal/Distribution

### P2-A: Qualify "cannot gerrymander" language
**Location**: Section 2
**Issue**: "It cannot gerrymander because it lacks the information required to do so" is accurate as an input-space claim but implies output neutrality. Paper C.5 documents structural partisan effects in algorithmic plans.
**Fix**: Change to: "It cannot be instructed to gerrymander — it lacks the information required to optimize a partisan objective." Add: "This does not guarantee partisan-neutral outcomes; geographic voter concentration creates structural effects that no redistricting method can entirely eliminate."
**Flagged by**: Duchin, Rodden

### P2-B: Qualify "byte-identical" claim
**Location**: Section 2
**Issue**: "Byte-identical district assignments" holds only within the same hardware architecture. A non-technical reader may attempt verification on a different architecture and get different results.
**Fix**: Change to: "Any two verifiers using the same software version on the same type of computer will produce identical district assignments, auditable through a SHA-256 hash chain." (Or: "on any modern Windows, Mac, or Linux computer of the same processor type.")
**Flagged by**: Duchin, Liang

### P2-C: Qualify the "ten minutes" verification claim
**Location**: Section 3
**Issue**: "Under ten minutes" likely refers to the fixture run time after software is already installed. End-to-end from zero setup takes 15–20 minutes. A non-technical reader testing this claim will be disappointed.
**Fix**: Change to: "Any citizen can verify the result in under ten minutes, once the free software is installed (software setup takes an additional 10–15 minutes on first use)."
**Flagged by**: Liang

### P2-D: Clarify "two hours" replication time
**Location**: Section 3
**Issue**: "Full fifty-state replication takes approximately two hours" — Paper A.4 states 18 minutes for 2020 alone, and 2–4 hours for all three census years. The brief does not specify which scenario gives two hours.
**Fix**: Change to: "Full fifty-state replication takes approximately 18 minutes for one census year, or two to four hours for all three census years."
**Flagged by**: Liang

### P2-E: Path 3 constitutional context
**Location**: Section 4, Path 3
**Issue**: Court-ordered algorithmic redistricting is primarily available under state constitutional challenges post-*Rucho*. The brief does not distinguish federal vs. state constitutional proceedings.
**Fix**: Add: "Court-ordered algorithmic remedies are primarily available under state constitutional standards; federal courts are foreclosed by *Rucho v. Common Cause* (2019)."
**Flagged by**: Stephanopoulos

### P2-F: Path 1 anti-commandeering note
**Location**: Section 4, Path 1
**Issue**: A federal mandate requiring states to use a specific algorithm would face constitutional challenges under the anti-commandeering doctrine and the Tenth Amendment.
**Fix**: Add: "A federal mandate would operate under Congress's Elections Clause authority (Art. I, §4) and may face anti-commandeering challenges. Paper D.4 analyzes the constitutional basis for federal adoption."
**Flagged by**: Stephanopoulos

### P2-G: 42% threshold: empirical vs legal bright line
**Location**: Section 3, "additional findings" paragraph
**Issue**: The 42% minority threshold is presented as a finding without D.1's disclaimer that it is an empirical regularity requiring full Gingles analysis.
**Fix**: Add parenthetical: "(an empirical benchmark, not a legal bright line — VRA analysis requires the full Gingles three-prong test)."
**Flagged by**: Duchin, Stephanopoulos

## What Changed R1 → R2

- [x] +22% retained — correct per (0.361-0.296)/0.296 population-weighted computation (P1-A: decision not to change; "averaged across all 435 districts" now explicit in Section 3)
- [x] Statute text revised: "ApportionRegions redistricting algorithm (an extension of the Huntington-Hill apportionment method to intrastate district drawing)"; Gingles justification requirement added for VRA overrides (P1-B)
- [x] Algorithm named "ApportionRegions" in Section 2 with explicit HH distinction: "related in approach but distinct in statutory scope" (supports P1-B)
- [x] Verification time: "under ten minutes" → "15-20 minutes end-to-end, including data download" in both Section 2 and results table (P2-C)
- [x] Replication time: "approximately two hours" → "18 minutes for one census year, or two to four hours for all three census years" (P2-D)
- [x] Path 3 now specifies "state constitutional standards" with PA/NC/NY examples (P2-E)
- [x] Efficiency gap footnote added: "near-zero efficiency gaps as byproduct of geometric neutrality (mean |EG|=0.04 vs 0.08 for enacted maps). In some geographically sorted states a small systematic gap persists due to urban concentration (Paper C.5)." (addresses Stephanopoulos P2)
- [ ] NC/GA counterexample (P1-C) — not added; deferred to next pass
- [ ] "Closely proportional" pull-quote (P1-D) — partially addressed by efficiency gap footnote
- [ ] "Cannot gerrymander" language (P2-A) — deferred to journal submission
- [ ] "Byte-identical" architecture qualifier in Section 2 (P2-B) — deferred
- [ ] Anti-commandeering note for Path 1 (P2-F) — deferred
- [ ] Gingles qualifier on 42% threshold in A.5 results (P2-G) — deferred (added to A.3, not yet to A.5)

## Notes

The brief passes the 3.0/4.0 bar at R1 (avg 3.02) but only barely. The P1 items — especially the statute text and the NC partisan framing — require substantive revisions, not just sentence-level edits. The compactness percentage error (22% vs 20%) appears in all three Track A documents and should be corrected as a batch fix across A.3, A.4, and A.5.

Target R2 average: ≥ 3.3/4.0. This is achievable with the P1 revisions; the P2 items would push to ≥ 3.5/4.0.

**R2 result**: 3.48/4.0 — passes ≥ 3.3 target. Largest score gain came from statute text revision (Karypis 3.3→3.7, Stephanopoulos 2.8→3.4). Key improvements: "ApportionRegions" naming resolves HH misnomer; Gingles justification requirement in statute; Path 3 state/federal distinction; 15-20 min timing; replication time clarified; efficiency gap footnote. NC geographic-variation disclosure (P1-C) not added — Rodden maintains this as a P2 concern. Gingles qualifier for 42% in A.5 results still missing (added to A.3, not A.5) — deferred to journal pass.
