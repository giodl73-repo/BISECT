# Incumbency Effects in Algorithmic Redistricting: Overview

**Series**: I.0
**Status**: Accepted 3.5/4
**Target**: Election Law Journal

## Algorithm / Subject

Overview paper synthesising I.1–I.4 into a unified framework for understanding how congressional incumbency interacts with algorithmic redistricting. Covers the legal status of incumbency as a redistricting criterion (permissible but not required), the distinction between explicit incumbency protection (using incumbent address data or election results to draw favorable districts) and implicit protection (using partisan data that correlates with incumbency), and the empirical baseline for how \textsc{bisect} algorithmic maps — which neither receive nor use any incumbency data — compare to enacted maps on three incumbency metrics: incumbent pairing rate, safe-seat creation rate, and open-seat production rate. Introduces the concept of the "incumbency-neutral baseline" for each state as the expected value of each incumbency metric under a random redistricting process given the state's geographic constraints.

## Key Claims

1. \textsc{Bisect} algorithmic maps produce incumbency outcomes statistically indistinguishable from a random redistricting process: incumbent pairing rate, safe-seat rate, and open-seat rate all fall within the expected range of a geographic redistricting algorithm with no incumbency data inputs, establishing the incumbency-neutral baseline for NC, WI, TX, and FL.
2. Enacted 2022 congressional maps in all four states produce significantly lower incumbent pairing rates and higher safe-seat rates than the \textsc{bisect} baseline, consistent with deliberate incumbent protection using address-level incumbency data — protection that \textsc{bisect} cannot provide and does not attempt.
3. The legal framework for incumbency in redistricting is asymmetric: courts have held that incumbency protection is a permissible factor (legislators may consider it), but no court has required it, and algorithmic maps that ignore incumbency are constitutionally sound — the absence of incumbency input does not make \textsc{bisect} maps legally deficient.

## Layer

Legal

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), FL ($k=28$)
- Incumbency data: 2022 congressional incumbent addresses and prior district boundaries
- Metrics: (I.1) incumbent pairing rate; (I.2) safe-seat fraction ($|\text{partisan lean}| > 15\text{pp}$); (I.3) expected open-seat count; compare bisect vs. enacted 2022 maps
- Baseline: expected values under random geographic redistricting (no incumbency data)

## Test Invariants

- L0: Bisect incumbent pairing rate $\in [0, 1]$; safe-seat fraction $\in [0, 1]$
- L1: On a synthetic state with two equal incumbent-containing tracts, pairing rate = 0 under optimal redistricting and 1.0 under worst-case; bisect falls between
- L2: NC bisect pairing rate is within 2 standard deviations of the analytical expectation for random $k=14$ redistricting given NC tract geometry

## Legal / Practitioner Value

Incumbency protection is one of the most commonly cited legitimate redistricting criteria in legislative records, litigation affidavits, and commission proceedings. This paper establishes that \textsc{bisect} maps handle incumbency the same way a neutral random process would — not by protecting incumbents, but not by pairing them either. Courts evaluating whether an algorithmic redistricting plan is a viable remedy in a gerrymandering case need to know whether the algorithm's treatment of incumbency is constitutionally sound. The answer is yes: algorithmic indifference to incumbency is legally permissible, and the resulting maps are no more disruptive to incumbency than geographic chance would produce.

## Section Structure

§1 Introduction and Legal Framework, §2 Incumbency as a Redistricting Criterion, §3 Three Incumbency Metrics: Pairing, Safety, and Openness, §4 Bisect as Incumbency-Neutral Baseline, §5 Empirical Overview Across NC/WI/TX/FL, §6 Legal Landscape, §7 Conclusion
