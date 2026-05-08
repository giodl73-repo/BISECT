# The Post-Shelby VRA Landscape and Algorithmic Redistricting

**Series**: D.8
**Status**: Accepted 3.5/4
**Target**: Columbia Law Review

## Algorithm / Subject

Legal synthesis paper analyzing the post-\textit{Shelby County v.\ Holder} VRA enforcement landscape and its implications for algorithmic redistricting. \textit{Shelby County v.\ Holder} (2013) struck down the coverage formula of Section 4(b) of the VRA, effectively invalidating the Section 5 preclearance requirement for covered jurisdictions without removing the requirement itself. This produced a paradox: Section 5 remains on the books but is unenforceable without a valid coverage formula. The 2021 decision in \textit{Brnovich v.\ Democratic National Committee} further weakened Section 2 vote-denial claims. This paper maps the enforcement landscape that remained after these decisions through 2026, analyzes which VRA protections remain effective for minority redistricting claims, and evaluates how algorithmic redistricting interacts with both the gutted preclearance regime and the surviving Section 2 framework.

## Key Claims

1. \textit{Shelby County v.\ Holder}\footnote{\textit{Shelby County v.\ Holder}, 570 U.S. 529 (2013).} struck down VRA Section 4(b)'s coverage formula as not rationally related to current conditions, effectively eliminating preclearance for states that had been covered (TX, GA, NC, AL, LA, SC, and others) — without overturning Section 5 itself. Congress has not enacted a new coverage formula as of 2026, meaning Section 5 preclearance is a dead letter: covered states have been free to enact redistricting plans without preclearance since 2013.
2. Section 2 vote dilution claims under the \textit{Thornburg v.\ Gingles} framework remain the primary VRA enforcement mechanism for redistricting after \textit{Shelby County}, but \textit{Brnovich v.\ Democratic National Committee}\footnote{\textit{Brnovich v.\ Democratic National Committee}, 141 S.Ct. 2321 (2021).} weakened Section 2 vote-denial claims (targeting election rules that disproportionately burden minority voters) while leaving Section 2 vote-dilution claims (targeting redistricting plans that dilute minority voting strength) largely intact under the \textit{Gingles} framework.
3. \textsc{Bisect}'s VRA mode, which implements the \textit{Gingles} three-prong analysis through a constraint that requires majority-minority districts where the geographic concentration, cohesion, and bloc-voting criteria are met, remains legally sufficient under the post-\textit{Shelby} Section 2 framework: the algorithm's VRA mode responds to Section 2's requirements without the preclearance submission process that Section 5 required, making it fully compatible with the post-\textit{Shelby} VRA landscape.

## Layer

Legal

## Empirical Targets

- Legal survey: all redistricting decisions citing Shelby County (2013–2026) in Section 2 context
- Section 5 coverage: former covered states (AL, AK, AZ, GA, LA, MS, NC, SC, TX, VA + counties) and their post-Shelby redistricting behavior
- Section 2 cases: post-Shelby Section 2 redistricting decisions, particularly those addressing the Gingles framework in algorithmic or commission-drawn maps
- Key cases: \textit{Shelby County v.\ Holder} (2013), \textit{Brnovich v.\ Democratic National Committee} (2021), \textit{Allen v.\ Milligan} (2023, upholding Section 2 vote dilution framework for redistricting)

## Test Invariants

- L0 (legal): Every case cited exists; citations are accurate; holdings are correctly stated
- L0 (coverage): Former Section 5 covered states are correctly identified
- L0 (Allen): \textit{Allen v.\ Milligan} (523 U.S. 1310, 2023) correctly stated as upholding Section 2 in redistricting context

## Legal / Practitioner Value

The post-\textit{Shelby} VRA landscape is the operational environment for all redistricting practitioners working in former preclearance states (TX, NC, GA, AL, etc.). Understanding what protections remain — Section 2 vote dilution, Section 203 language assistance, state VRA analogs — and what has been eliminated (Section 5 preclearance) is essential for expert witnesses, commission staff, and legislators. D.8 provides this map. For algorithmic redistricting practitioners, the key finding is that \textsc{bisect}'s VRA mode is designed to satisfy Section 2 (the surviving federal standard), not Section 5 (the defunct preclearance requirement), and is therefore fully adapted to the post-\textit{Shelby} environment. The paper also covers \textit{Allen v.\ Milligan} (2023), which reaffirmed the Gingles framework for redistricting challenges and is the most important post-\textit{Shelby} SCOTUS redistricting decision.

## Section Structure

§1 Introduction: The Post-Shelby VRA Enforcement Gap, §2 Section 5 Before and After Shelby County, §3 The Coverage Formula and Why It Was Struck Down, §4 Congressional Inaction: No New Coverage Formula as of 2026, §5 Section 2 as the Primary Remaining Tool: Vote Dilution Claims Under Gingles, §6 Brnovich v.\ DNC (2021): Section 2 Vote-Denial Claims Weakened, §7 Allen v.\ Milligan (2023): Gingles Reaffirmed for Redistricting, §8 State VRA Analogs: California, New York, and Others, §9 Bisect VRA Mode in the Post-Shelby Landscape, §10 Conclusion
