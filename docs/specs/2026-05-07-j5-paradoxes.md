# Apportionment Paradoxes: Alabama, Population, and New-States

**Series**: J.5
**Status**: Accepted 3.5/4
**Target**: American Mathematical Monthly

## Algorithm / Subject

Mathematical paper proving which apportionment methods are susceptible to the three classical apportionment paradoxes and which are immune, with formal proofs and numerical examples from U.S. apportionment history. The three paradoxes are: (1) the Alabama Paradox (increasing House size causes a state to lose a seat); (2) the Population Paradox (one state's population grows faster than another's, but the faster-growing state loses a seat to the slower-growing state when seats are reapportioned); (3) the New-States Paradox (adding a new state with a proportional number of new seats causes existing states to lose seats to each other). The central theorem: divisor methods (Huntington-Hill, Webster, Adams, Jefferson) are immune to all three paradoxes; quota methods (Hamilton) are susceptible to all three.

## Key Claims

1. \textbf{Hamilton susceptibility}: Hamilton (largest remainders) is susceptible to all three classical paradoxes, with documented historical instances of the Alabama Paradox (1880 Census: adding a seat reduced Alabama from 8 to 7), the Population Paradox (1900 Census), and the New-States Paradox (Oklahoma statehood, 1907) — formal proofs that these paradoxes are not eliminable from any quota method that satisfies the quota rule.
2. \textbf{Divisor method immunity}: All four divisor methods (Huntington-Hill, Webster, Adams, Jefferson) are provably immune to all three paradoxes — the proof proceeds by showing that divisor method seat allocations are monotone in population and House size under a fixed common divisor, and that the common divisor can always be adjusted to maintain monotonicity when House size or populations change.
3. \textbf{Impossibility theorem}: Balinski and Young's impossibility theorem (1982) proves that no apportionment method can simultaneously satisfy the quota rule and be free from the Alabama paradox — establishing that the tradeoff between quota compliance (Hamilton) and paradox immunity (divisor methods) is fundamental, not a design deficiency of any particular method.

## Layer

Algorithm / Mathematical

## Empirical Targets

- Historical documentation: Alabama Paradox (1880 U.S. Census), Population Paradox (1900 Census), New-States Paradox (Oklahoma 1907)
- Formal proofs: Alabama paradox susceptibility for Hamilton; immunity for all divisor methods
- Balinski-Young theorem: statement, intuition, implications
- Numerical examples: 3-state synthetic examples demonstrating each paradox under Hamilton
- Verify: bisect-apportion's Huntington-Hill implementation produces no paradox instances across 2000, 2010, 2020 Census data

## Test Invariants

- L0 (monotonicity): For any two Census data sets where $\text{Pop}_i^{(2)} > \text{Pop}_i^{(1)}$ for all $i$, Huntington-Hill seat counts are monotone non-decreasing
- L0 (House size): For Huntington-Hill, increasing $H$ from 435 to 436 causes no state to lose a seat
- L1: Hamilton on synthetic 3-state example with $H \in \{5, 6\}$ demonstrates Alabama Paradox
- L2: Huntington-Hill on 2020 Census with $H \in \{434, 435, 436\}$ shows no paradox instances

## Legal / Practitioner Value

J.5 provides the mathematical foundation for Congress's 1941 statutory choice of Huntington-Hill. The Alabama Paradox was politically damaging: it meant that a state could lobby Congress to increase the House size, only to end up with fewer seats than before — a result that was politically embarrassing and constitutionally questionable. The immunity of divisor methods to paradoxes was the primary technical reason for abandoning Hamilton in 1941. For \textsc{bisect-apportion} users and for apportionment reform advocates, J.5 establishes that the paradox-immunity of Huntington-Hill is not accidental but provable — and that the Balinski-Young impossibility theorem means no quota method can achieve the same property. The theorem is also central to the academic literature on electoral system design and provides the theoretical foundation for why proportional representation systems universally use divisor methods (d'Hondt, Webster) rather than quota methods (Hamilton).

## Section Structure

§1 Introduction: Three Paradoxes in U.S. Apportionment History, §2 Hamilton (Largest Remainders) and the Alabama Paradox: Definition and Historical Example, §3 Population Paradox: Definition, Hamilton Susceptibility, Historical Example, §4 New-States Paradox: Definition, Hamilton Susceptibility, Oklahoma Example, §5 Divisor Method Immunity: Formal Proof for All Four Divisor Methods, §6 The Balinski-Young Impossibility Theorem: Statement and Implications, §7 Consequences for Apportionment Method Choice, §8 Conclusion
