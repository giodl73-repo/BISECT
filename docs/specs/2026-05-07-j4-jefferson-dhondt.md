# Jefferson / d'Hondt Greatest Divisors Apportionment

**Series**: J.4
**Status**: Accepted 3.5/4
**Target**: Electoral Studies

## Algorithm / Subject

Dedicated treatment of the Jefferson (greatest divisors) method — known internationally as d'Hondt — the most large-state-favorable of the five major methods and the dominant proportional representation formula used in parliamentary elections worldwide. Jefferson assigns seats by successively awarding the next seat to the state (or party, in the electoral context) with the highest priority $P_i = \text{Pop}_i / s_i$, where $s_i$ is the state's current seat count starting from 1. This floor rounding rule (each state's seats are $\lfloor \text{Pop}_i / d \rfloor$ for some common divisor $d$) always rounds down, systematically under-representing small states. The paper documents Jefferson's mathematical properties, its use in U.S. apportionment from 1790 to 1840, its dominance as the d'Hondt formula in European and Latin American party-list PR systems, and the \textsc{bisect-apportion} Jefferson/d'Hondt implementation.

## Key Claims

1. Jefferson/d'Hondt is the most large-state-favorable divisor method: under floor rounding, large states receive more seats than their exact proportional entitlement while small states receive fewer, because the floor of a small exact quota (e.g., 1.4 → 1) loses a larger share of the fractional seat than the floor of a large exact quota (e.g., 10.4 → 10).
2. The Jefferson priority formula $P_i = \text{Pop}_i / s_i$ is identical to the d'Hondt priority formula $P_j = V_j / s_j$ used in proportional representation elections, with population replacing votes and states replacing parties — the formal equivalence establishes that Jefferson apportionment is exactly d'Hondt PR applied to state populations.
3. Under 2020 Census data, Jefferson produces substantially different results from Huntington-Hill for small states: states at the 1-seat boundary (Wyoming, Vermont, Alaska) are most affected, as Jefferson's floor rounding provides no protection for states whose exact quota is between 1.0 and 2.0.

## Layer

Algorithm

## Empirical Targets

- 2020 Census population data (all 50 states)
- Compute: Jefferson apportionment for 2020 Census; compare to Huntington-Hill, Webster, Adams
- Identify: which states lose under Jefferson (small states) vs. Huntington-Hill
- Document: historical U.S. apportionments using Jefferson (1790, 1800, 1810, 1820, 1830, 1840)
- Comparative: d'Hondt vs. Jefferson priority table (formal equivalence proof)

## Test Invariants

- L0: Total seats = 435; each state $\geq 1$ seat; computation terminates
- L0: Jefferson seat count $\leq$ Webster seat count $\leq$ Adams seat count for small states
- L1: Priority sequence reproduces the standard d'Hondt allocation for the 5-party, 10-seat synthetic example
- L2: Jefferson output for 2020 Census is reproducible from Census population figures

## Legal / Practitioner Value

Jefferson/d'Hondt's historical use in the United States (the first six apportionments) and its current dominance in European PR systems make it the most internationally important apportionment method. J.4 establishes the formal d'Hondt equivalence that is essential background for comparative electoral systems scholarship. For \textsc{bisect-apportion} users, the Jefferson implementation enables simulation of alternative apportionment scenarios for policy analysis. The large-state-favoring property of Jefferson helps explain why the method was abandoned in the U.S. (large states consistently manipulated it) and why Huntington-Hill was chosen as a compromise.

## Section Structure

§1 Introduction: Jefferson in U.S. History and d'Hondt Globally, §2 Mathematical Definition: Floor Rounding, Priority Formula, Common Divisor, §3 Equivalence to d'Hondt: Formal Proof, §4 Large-State Bias: Formal Properties, §5 2020 Census Results: Jefferson vs. Huntington-Hill, §6 Historical U.S. Apportionments (1790–1840), §7 Bisect-Apportion Implementation, §8 Conclusion
