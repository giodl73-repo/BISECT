# Schwartzberg Score in Algorithmic Redistricting

**Series**: K.4
**Status**: Accepted 3.5/4
**Target**: Political Analysis

## Algorithm / Subject

Mathematical and empirical analysis of the Schwartzberg compactness score (S = Perimeter / (2$\sqrt{\pi \cdot \text{Area}}$)), which measures deviation from a circle on a $[1, \infty)$ scale where 1 represents a perfect circle. Covers the formula derivation, the mathematical equivalence with Polsby-Popper ($S = 1/\sqrt{\text{PP}}$ up to a constant), state statutes that mandate or reference the Schwartzberg score, and an empirical comparison of S values across bisect structure algorithms. Provides the exact conversion formula between PP and S and documents which legislative contexts prefer each representation.

## Key Claims

1. The Schwartzberg score is mathematically equivalent to $1/\sqrt{\text{PP}}$ (within a scaling constant of $1/\sqrt{4\pi}$): S = $\text{Perimeter} / (2\sqrt{\pi \cdot \text{Area}}) = 1/\sqrt{\text{PP}}$ exactly, meaning S and PP convey identical geometric information with inverted scales and no information loss.
2. Colorado's redistricting statute (Colo. Const. Art. V, §44) references a Schwartzberg-equivalent score as one of the compactness criteria for the independent redistricting commission, making S the legally operative metric in at least one state where bisect results may be submitted.
3. The conversion formula S = $1/\sqrt{\text{PP}}$ allows direct translation between the PP values reported in bisect label-analyze output and the S values required by state statutes: a district with PP = $0.22$ has S = $1/\sqrt{0.22} \approx 2.13$, and a district with PP = $0.40$ has S $\approx 1.58$.

## Layer

Standalone

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), 2020 census; Colorado ($k=8$) as the statute-mandated case
- Compare across: standard-bisect, prime-factor, ratio-optimal, moving-knife structure algorithms
- Metrics: district-level S distribution (min, median, mean, max), S–PP equivalence verification (confirm $|S - 1/\sqrt{\text{PP}}| < 10^{-6}$ for all computed districts), state-level mean S

## Test Invariants

- L0: S of a circle is exactly 1.0; S $\geq$ 1.0 for any polygon (equality only for a circle); $|S - 1/\sqrt{\text{PP}}| < 10^{-9}$ for all test polygons (equivalence holds to floating-point precision); S is monotone with perimeter for fixed area
- L1: on a $4 \times 4$ grid district, S matches the analytical formula for a square ($S_\text{square} = \sqrt{\pi}/2 \approx 0.886$... corrected: $S_\text{square} = 4/\sqrt{4\pi} = 2/\sqrt{\pi} \approx 1.128$) within 0.001; PP and S computed on the same geometry satisfy $|S \cdot \sqrt{\text{PP}} - 1| < 10^{-6}$
- L2: Colorado 2020 bisect districts have mean S $\leq 1.8$ (corresponding to PP $\geq 0.31$); S values for all bisect structure algorithms satisfy $|S - 1/\sqrt{\text{PP}}| < 10^{-4}$ across all NC/WI/TX districts

## Legal / Practitioner Value

The Schwartzberg score is preferred over PP in legislative contexts because its $[1, \infty)$ scale is easier to communicate as a "distance from circular": a district with S = 2 is twice as non-circular as a circle, whereas PP = 0.25 requires practitioners to remember that the scale is inverted. Colorado's constitution mandates a compactness criterion that has been operationalised as Schwartzberg-equivalent by the state redistricting commission. Iowa, Oregon, and Utah redistricting guidelines reference perimeter-based compactness measures that can be mapped to S. This paper provides the conversion formula practitioners need when court filings or state statutes specify Schwartzberg but bisect reports PP.

## Section Structure

§1 Introduction, §2 Mathematical Definition (Schwartzberg Formula and Range), §3 Equivalence Theorem (S = $1/\sqrt{\text{PP}}$) with Proof, §4 Conversion Table and Calculator, §5 Statutory Survey (States That Mandate or Reference S), §6 Empirical Comparison Across Structure Algorithms, §7 Practitioner Usage Guide, §8 Conclusion
