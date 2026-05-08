# Congressional Apportionment Methods: Overview and Framework

**Series**: J.0
**Status**: Accepted 3.5/4
**Target**: American Political Science Review

## Algorithm / Subject

Overview paper for the J-series, providing a unified framework for evaluating the five major congressional apportionment methods: Huntington-Hill (equal proportions, current U.S. method), Webster (major fractions), Adams (smallest divisors), Jefferson/d'Hondt (greatest divisors), and Hamilton (largest remainders / quota method). Covers the constitutional foundation (Art. I, §2: "representatives...shall be apportioned among the several states according to their respective numbers"), the mathematical distinction between divisor methods (Huntington-Hill, Webster, Adams, Jefferson) and quota methods (Hamilton), the Alabama paradox and related paradoxes that afflict quota methods, and the \textsc{bisect-apportion} crate's implementation which exactly reproduces the official 2020 Census Bureau apportionment. Introduces the fairness criterion taxonomy: population pair test (divisor methods), quota rule (quota methods), labor intensity (complexity of computation).

## Key Claims

1. All five major apportionment methods produce similar results for large states but diverge for small states (population close to the rounding threshold), where the choice of method can determine whether a state receives its "fair share" of seats — the paper documents which states would receive different seat counts under Webster vs. Huntington-Hill for 2020 Census data.
2. Divisor methods (Huntington-Hill, Webster, Adams, Jefferson) are immune to the Alabama paradox and related paradoxes (population paradox, new-states paradox) because their seat allocations are determined by a monotone divisor function; quota methods (Hamilton) are susceptible because the largest-remainder step can produce non-monotone results.
3. The \textsc{bisect-apportion} implementation exactly reproduces the official 2020 Census Bureau apportionment under Huntington-Hill, verified by SHA-256 comparison to the Census Bureau's published results, establishing that the crate is a correct implementation of U.S. law (2 U.S.C. §2a) with no rounding errors across all 50 states.

## Layer

Algorithm / Legal

## Empirical Targets

- 2020 Census population data, all 50 states
- Compare: seat allocations under Huntington-Hill, Webster, Adams, Jefferson, Hamilton for 2020 Census
- Identify: which states gain or lose seats under each method relative to Huntington-Hill
- Verify: \textsc{bisect-apportion} matches 2020 Census Bureau official apportionment exactly (SHA-256)

## Test Invariants

- L0: Total seats under any method = 435; all states receive $\geq 1$ seat; computation terminates
- L1: On a synthetic 5-state example with known correct answers, all five methods produce expected outputs
- L2: bisect-apportion output for 2020 Census matches official Census Bureau apportionment at 100\% match rate (all 50 states)

## Legal / Practitioner Value

The J-series establishes the mathematical foundation for \textsc{bisect-apportion} as a legally and algorithmically correct implementation of U.S. apportionment law. The Huntington-Hill method is mandated by 2 U.S.C. §2a, enacted in 1941. Its mathematical properties — including immunity to the Alabama paradox — have been litigated in the Supreme Court (\textit{United States Dept.\ of Commerce v.\ Montana}, 1992, upholding Congress's statutory choice). J.0 provides the framework that J.1–J.6 build on. State legislative apportionment (which may use different methods) and redistricting process design (which requires knowing how many districts each state receives before drawing boundaries) both depend on correct implementation of the apportionment step.

## Section Structure

§1 Introduction and Constitutional Foundation, §2 Taxonomy: Divisor Methods vs. Quota Methods, §3 Overview of Five Methods: Definitions and Properties, §4 Fairness Criteria: Population Pair Test, Quota Rule, Bias Direction, §5 Bisect-Apportion: Implementation and 2020 Verification, §6 Synthesis of J.1–J.6 Findings, §7 Conclusion
