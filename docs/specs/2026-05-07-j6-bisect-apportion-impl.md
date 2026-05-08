# Bisect-Apportion: Implementation and 2020 Census Verification

**Series**: J.6
**Status**: Accepted 3.5/4
**Target**: Journal of Statistical Software

## Algorithm / Subject

Software engineering and verification paper documenting the \textsc{bisect-apportion} Rust crate, which implements all five major congressional apportionment methods (Huntington-Hill, Webster, Adams, Jefferson/d'Hondt, Hamilton) with a unified interface. The central claim is that the \textsc{bisect-apportion} Huntington-Hill implementation exactly reproduces the official 2020 Census Bureau congressional apportionment, verified by SHA-256 comparison to the Census Bureau's published results. The paper documents the implementation architecture, the rounding logic for each method, the edge case handling (tie-breaking, minimum 1 seat, integer arithmetic to avoid floating-point rounding errors), and the test suite that achieves 100\% line coverage across all five methods.

## Key Claims

1. The \textsc{bisect-apportion} crate exactly reproduces the official 2020 Census Bureau congressional apportionment under Huntington-Hill, verified by computing the SHA-256 hash of the bisect-apportion output (state name + seat count, sorted alphabetically) and comparing to the SHA-256 of the Census Bureau's published apportionment results — zero discrepancies across all 50 states.
2. The implementation uses exact integer arithmetic throughout (avoiding floating-point rounding errors) by representing all priority values as rational fractions with numerators and denominators that fit in u128 integers, enabling exact comparisons between priority values for tie-breaking without numerical precision loss.
3. The crate implements all five methods with a unified trait interface that enables comparative analysis (see J.0–J.5), and the test suite includes 61 tests across three levels: L0 (unit tests on priority formulas), L1 (integration tests on synthetic examples with known correct answers), and L2 (real-data regression tests on 2000, 2010, and 2020 Census apportionments for all five methods).

## Layer

Algorithm / Software

## Empirical Targets

- 2020 Census official apportionment: 435 seats, 50 states (apportionment table published by Census Bureau)
- SHA-256 verification: hash of bisect-apportion Huntington-Hill output matches hash of Census Bureau table
- 2010 Census: reproduce 2010 official apportionment (Huntington-Hill) — 0 discrepancies
- 2000 Census: reproduce 2000 official apportionment (Huntington-Hill) — 0 discrepancies
- Comparative: compute all five methods for 2020 Census; document which states differ across methods

## Test Invariants

- L0 (61 tests): All priority formulas produce expected values on unit inputs; edge cases (1 state, 1 seat; tie at boundary) handled correctly
- L1 (synthetic): 3-state examples with known answers for all five methods produce expected outputs
- L2 (regression): Huntington-Hill matches 2020, 2010, 2000 Census Bureau results at 100\% match rate; Webster, Adams, Jefferson, Hamilton reproduce academic results from Balinski-Young (1982) for 1980 Census

## Legal / Practitioner Value

The SHA verification is the key claim: it proves that \textsc{bisect-apportion} is not an approximation of U.S. apportionment law but an exact implementation. For a redistricting pipeline that begins with apportionment, this exactness matters: any error in the apportionment step propagates to every downstream computation (number of districts per state, district population targets). The 61-test suite at L0/L1/L2 provides the test coverage expected by professional software standards for a crate implementing federal law. The unified five-method interface enables academic and policy analysis of alternative apportionment scenarios without requiring separate implementations. For practitioners using \textsc{bisect} in litigation or commission contexts, the SHA verification provides auditable proof that the apportionment step is correct — a level of transparency that custom legislative software rarely achieves.

## Section Structure

§1 Introduction: Why Exact Implementation Matters, §2 Implementation Architecture: Priority Queue, Integer Arithmetic, Trait Interface, §3 Huntington-Hill Implementation: Priority Formula and Tie-Breaking, §4 Webster, Adams, Jefferson, Hamilton Implementations, §5 SHA-256 Verification Against 2020 Census Bureau Results, §6 Test Suite: L0/L1/L2 Coverage, §7 Comparative Output: All Five Methods for 2020 Census, §8 Conclusion
