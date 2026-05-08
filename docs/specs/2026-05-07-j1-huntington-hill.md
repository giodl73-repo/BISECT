# Huntington-Hill Equal Proportions: U.S. Apportionment Law

**Series**: J.1
**Status**: Accepted 3.5/4
**Target**: Harvard Law Review

## Algorithm / Subject

Dedicated treatment of the Huntington-Hill (equal proportions) method, the apportionment formula mandated by 2 U.S.C. §2a for congressional seat allocation among the 50 states. The method assigns seats by successively awarding the next seat to the state with the highest priority value $P_i = \text{Pop}_i / \sqrt{s_i(s_i+1)}$, where $s_i$ is the state's current seat count. This priority formula implements the geometric mean rounding rule: a state receives an additional seat if its current per-capita representation (using its existing seats) is below the geometric mean of the representations at $s_i$ and $s_i+1$ seats. The paper develops the method's mathematical properties, its 1941 statutory adoption, its Supreme Court validation, and the \textsc{bisect-apportion} implementation's exact reproduction of the 2020 Census result.

## Key Claims

1. The Huntington-Hill priority formula implements the geometric mean rounding rule, which minimizes the relative difference in per-capita representation between any two states receiving $s$ and $s+1$ seats respectively — a property that distinguishes it from Webster (arithmetic mean, minimizes absolute difference) and makes it slightly favorable to small states relative to Webster.
2. The Huntington-Hill method is immune to the Alabama paradox: adding population to any state never decreases its seat count, and increasing the House size never causes any state to lose a seat — formal proofs of these properties establish why Congress chose this method in 1941 over Hamilton, which was susceptible to both paradoxes.
3. The \textsc{bisect-apportion} crate implementation of Huntington-Hill produces an output that matches the 2020 Census Bureau official apportionment exactly across all 50 states (SHA-verified), confirming that the implementation correctly handles the "each state gets at least 1 seat" constitutional floor, the priority queue termination condition at 435 total seats, and all tie-breaking edge cases.

## Layer

Algorithm / Legal

## Empirical Targets

- 2020 Census population data (all 50 states, DC excluded from apportionment)
- Reproduce: official 2020 apportionment (Alabama 7, Alaska 1, ..., Wyoming 1)
- Verify: SHA-256 hash of bisect-apportion output matches Census Bureau published results
- Compute: priority sequence for 2020 data (seats 51 through 435)
- Identify: the "last seat" recipient and the "just missed" state for 2020

## Test Invariants

- L0: Total seats = 435; each state $\geq 1$ seat; priority queue terminates
- L0: All priority values $P_i > 0$ for all states with population $> 0$
- L1: On the 3-state synthetic example (pop 6, 6, 1; 3 seats), produces [2, 2, 1] — the same result as Jefferson and Webster
- L2: 2020 Census output matches official apportionment (SHA-256 verification)
- L2: NC receives 14 seats (correct); TX receives 38 seats (correct)

## Legal / Practitioner Value

Huntington-Hill is U.S. law. Any redistricting pipeline that produces congressional district maps must begin with a correct apportionment step to know how many districts each state requires. \textsc{Bisect-apportion} provides that step. J.1 establishes the method's mathematical foundations (for expert witness testimony in apportionment litigation), its legal history (the 1941 statute and \textit{United States Dept. of Commerce v. Montana}, 503 U.S. 442 (1992)), and its implementation correctness (SHA verification). The Montana case is the definitive SCOTUS ruling on the constitutionality of Huntington-Hill; the paper documents the Court's holding that Congress has broad discretion in choosing among apportionment methods and the chosen method need not be "optimal" under any particular fairness criterion.

## Section Structure

§1 Introduction: Huntington-Hill in U.S. Law, §2 Mathematical Definition: Priority Formula and Geometric Mean Rounding, §3 Algebraic Properties: Immunity to Paradoxes, §4 Comparison to Neighboring Methods (Webster and Adams), §5 1941 Statutory Adoption and Congressional Intent, §6 \textit{United States Dept.\ of Commerce v.\ Montana} (1992), §7 Bisect-Apportion Implementation and 2020 Verification, §8 Conclusion
