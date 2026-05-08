# Open-Seat Effects Under Algorithmic Redistricting

**Series**: I.3
**Status**: Accepted 3.5/4
**Target**: Legislative Studies Quarterly

## Algorithm / Subject

Analysis of open-seat creation — the production of congressional districts that contain no sitting incumbent from either party — as an outcome of redistricting. Open seats are more competitive, have higher campaign expenditure, and produce lower incumbency advantage than seats with a sitting member. Enacted maps typically minimize open-seat creation, assigning each incumbent a "home" district that maximizes their reelection prospects. \textsc{Bisect} maps are indifferent to incumbent locations and produce open seats at the rate expected by geographic chance: when redistricting randomly (without incumbency data) draws boundaries that place no incumbent's residence in a given district, that district is effectively open regardless of historical partisan lean.

## Key Claims

1. \textsc{Bisect} algorithmic maps produce open-seat counts that are statistically indistinguishable from the analytical expectation under random geographic redistricting: for NC ($k=14$) with 13 incumbents seeking reelection, the expected number of open \textsc{bisect} districts is approximately 1.0–2.5 depending on how "open" is defined (no incumbent in district vs. incumbent in district but with competitive lean), and actual \textsc{bisect} open-seat counts fall within this range.
2. Enacted 2022 congressional maps produce significantly fewer open seats than the \textsc{bisect} baseline in states where the majority party controls redistricting (NC, TX, FL), consistent with explicit use of incumbent address data to guarantee each incumbent a home district — a practice requiring information that \textsc{bisect} does not access.
3. Open-seat creation rates under \textsc{bisect} are comparable to rates in neutral redistricting contexts (competitive commissions, court-drawn remedial maps) but substantially higher than in partisan maps, establishing that algorithmic indifference to incumbency produces the same open-seat dynamics as neutral human redistricting — not more disruptive, but more accountable.

## Layer

Empirical

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), FL ($k=28$)
- Incumbency data: 2022 congressional incumbents by census tract of home address; incumbents seeking reelection
- Metric: Number of districts with no incumbent's home census tract assigned to them (strict open seat) and number with incumbent but competitive lean ($|v_D - 0.50| < 0.10$)
- Baseline: Analytical expectation $E[\text{open seats}]$ under random tract-to-district assignment
- Compare: bisect vs. enacted vs. random baseline vs. commission-drawn maps (where available)

## Test Invariants

- L0: Open-seat count $\in [0, k]$; open-seat count + incumbent-seat count = $k$
- L1: On a synthetic state with one incumbent per district (perfectly distributed), open-seat count = 0 under any redistricting that preserves district count; bisect inherits 0 open seats
- L2: NC bisect open-seat count is within 95\% CI of analytical random expectation given NC geographic constraints

## Legal / Practitioner Value

Open-seat creation is politically significant: open seats are more likely to change party, receive more campaign investment, and produce legislators without pre-existing donor networks from the previous district. State legislators resist creating open seats because it increases electoral uncertainty for their caucus. This paper establishes that \textsc{bisect}'s indifference to incumbency produces a constitutionally sound number of open seats — comparable to what neutral processes produce — while enacted maps deliberately minimize open seats through incumbency data use. For courts evaluating whether algorithmic redistricting is a viable remedy, this finding establishes that algorithmic maps are not more disruptive to incumbency than neutral human redistricting; the disruption is comparable to what a fair process would produce.

## Section Structure

§1 Introduction, §2 Open-Seat Definition and Measurement, §3 Analytical Baseline Under Random Redistricting, §4 Behavior Under Algorithmic Redistricting, §5 Empirical Results, §6 Comparison to Commission-Drawn and Court-Drawn Maps, §7 Conclusion
