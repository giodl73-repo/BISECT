# Safe-Seat Creation and Competitive Districts

**Series**: I.2
**Status**: Accepted 3.5/4
**Target**: Journal of Politics

## Algorithm / Subject

Analysis of safe-seat creation — the production of congressional districts with partisan lean exceeding $\pm 15$ percentage points from parity (i.e., expected Democratic vote share outside the range $[0.35, 0.65]$) — as an outcome of redistricting. Safe seats protect incumbents from competitive general elections, reducing accountability and enabling long-serving legislators who are insulated from statewide vote swings. Enacted maps often deliberately create safe seats for incumbents of both parties (bipartisan incumbent protection); \textsc{bisect} maps are indifferent to partisan lean and produce safe seats only to the extent that the geographic distribution of voters makes them unavoidable. The paper documents the safe-seat fraction in \textsc{bisect} vs. enacted 2022 congressional maps across NC, WI, TX, and FL, and shows that enacted maps produce significantly more safe seats than compact algorithmic redistricting requires.

## Key Claims

1. \textsc{Bisect} algorithmic maps produce fewer safe seats (defined as $|\text{partisan lean}| > 15\text{pp}$) than enacted 2022 congressional maps in all four states: NC bisect safe-seat fraction $\approx 0.43$ vs. enacted $\approx 0.64$; WI bisect $\approx 0.38$ vs. enacted $\approx 0.63$; TX bisect $\approx 0.53$ vs. enacted $\approx 0.71$.
2. \textsc{Bisect} maps produce more competitive districts (partisan lean within $\pm 5\text{pp}$) than enacted maps in all four states, consistent with the theoretical prediction that compactness-driven redistricting without partisan data produces a roughly uniform distribution of district partisan lean, with fewer extreme-margin districts than deliberate packing or incumbent protection creates.
3. The safe-seat reduction from enacted to \textsc{bisect} maps is driven primarily by the elimination of the extreme-margin Republican safe seats created by cracking Democratic voters (see L.4), not by the elimination of geographic-sorting-driven safe seats in urban cores — the urban-core safe Democratic districts are largely preserved by compactness in both plans.

## Layer

Empirical

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), FL ($k=28$)
- Data: VEST 2020 presidential precinct returns mapped to 2020 census tracts
- Metric: Fraction of districts with $|v_D - 0.50| > 0.15$ (safe-seat fraction) and $|v_D - 0.50| < 0.05$ (competitive-seat fraction)
- Compare: bisect vs. enacted 2022 maps on both metrics
- Party breakdown: safe Democratic seats vs. safe Republican seats separately

## Test Invariants

- L0: Safe-seat fraction $\in [0, 1]$; safe + competitive + marginal = 1 (when marginal $= |v_D - 0.50| \in [0.05, 0.15)$)
- L1: On a synthetic perfectly sorted state (all tracts either 0\% or 100\% Democratic), safe-seat fraction = 1.0 for any map
- L2: NC bisect safe-seat fraction is lower than NC enacted safe-seat fraction at 99\% confidence

## Legal / Practitioner Value

Safe-seat creation is the mechanism by which partisan gerrymanders entrench themselves across election cycles: once safe districts are created, incumbents in those districts become effectively immune from statewide partisan shifts, producing a stable delegation that does not respond to voter preferences. State court claims under "free elections" clauses can draw on safe-seat evidence to show that the enacted map suppresses electoral accountability. The bisect comparison demonstrates that the safe-seat count is substantially above the geographic minimum — the excess is attributable to deliberate redistricting choices that insulate incumbents beyond what compact geography requires.

## Section Structure

§1 Introduction, §2 Safe-Seat Definition and Measurement, §3 Behavior Under Algorithmic Redistricting, §4 Empirical Results Across Four States, §5 Party-Specific Analysis: Which Incumbents Are Protected, §6 Legal Implications for Free Elections Doctrine, §7 Conclusion
