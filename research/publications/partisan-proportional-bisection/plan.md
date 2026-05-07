# B.12 — The Proportionality Compromise: Dual-Constrained Bisection for Partisan-Fair Redistricting

**Paper Type**: Algorithm Theory + Empirical Analysis  
**Status**: Draft (2026-05-03)  
**Series**: B (Algorithm Design)  
**Depends on**: B.9 (AreaSection — ncon=2 infrastructure), B.11 (PrimeFactor — HH seat allocation)  
**Companion**: B.9 (geographic balance), B.8 (compactness)

---

## Core Idea

Standard minimum-edge-cut redistricting uses ncon=1 (population balance only).
AreaSection (B.9) added ncon=2 with land area to prevent urban peeling.
B.12 adds ncon=2 with **Democratic vote share** to investigate what partisan
proportionality would require geometrically — and how far current algorithms
fall short.

**The formula.** For a state with D fraction `d`, R fraction `r = 1-d`,
`k` districts, and proportional allocation `k_D` D-seats, `k_R` R-seats:

The first bisection uses METIS ncon=2 with vertex weights [population, D_votes]
and tpwgts:

```
pop_left  = k_D / k            D_left  = 1 - k_R / (2 × d × k)
pop_right = k_R / k            D_right = k_R / (2 × d × k)
```

This gives the R-bloc exactly 50% D concentration (minimum for R to win each
district) while the D-bloc absorbs all surplus Democratic votes.

**The compromise.** Federal §104(e) prohibits partisan inputs for congressional
redistricting. B.12 investigates this as a theoretical benchmark:
- How much partisan input (measured in D_votes vertex weight constraint strength)
  is needed to achieve proportionality?
- How close do purely geometric algorithms (B.8/B.9) get to proportional outcomes?
- What geographic structures allow proportionality to emerge without explicit
  partisan constraints?

This is NOT advocacy for partisan redistricting. It is measurement of the
*proportionality gap* induced by geography-only algorithms.

---

## Key Contributions

1. **The proportionality formula**: closed-form tpwgts for any (d, r, k_D, k_R)
   that guarantee partisan-proportional bisection when geography allows it.

2. **The partisan Lorenz curve**: analogous to B.9's area Lorenz curve —
   sorts tracts by D fraction (most D first), maps cumulative population to
   cumulative D-voter fraction. Determines when proportional packing is
   geographically feasible.

3. **The proportionality gap**: for all 50 states, compare:
   - B.9 (geographic-only) outcome
   - B.12 (partisan-constrained) outcome
   - The theoretical proportional outcome
   
4. **The minimum partisan input**: for each state, how much does the D_votes
   constraint need to "pull" from the pure geographic optimum to achieve
   proportionality? States with uniform partisan geography need no pull;
   states with concentrated D voters need maximum pull.

---

## Mathematical Framework

### The Proportionality Condition

A redistricting plan with k districts is **proportional** if the number of
D-winning districts equals `k_D = round(d × k)` (Huntington-Hill rounding).

For a bisection into k_D:k_R to be proportional, each D-bloc district must
have D > 50% and each R-bloc district must have R > 50%.

**Minimum packing lemma**: The minimum D_votes fraction for the R-bloc
(to achieve exactly 50% D in the R-bloc) is:

```
D_right_min = k_R / (2 × d × k)
```

Any less: R-bloc is D-majority, R cannot win.
Any more: D votes are wasted in the D-bloc (D concentration exceeds what's needed).

### The Partisan Lorenz Curve

Sort tracts by D fraction (densest D first). Define:
- `L_D(x)` = cumulative D-voter fraction at cumulative population fraction x
- `L_R(x)` = cumulative R-voter fraction at x (in sparse-D-first order)

**Feasibility condition**: The proportional tpwgts is achievable if and only if:

```
L_D(k_R/k) ≤ D_right_min = k_R/(2dk)
```

i.e., the least-D portion of the state (the R-bloc's geography) contains
at most the minimum D fraction needed.

For states where D voters are perfectly uniform (L_D = diagonal): all ratios
are feasible. For states with extreme D concentration: only k_D=1 ratios
are feasible (can't create competitive R districts without contiguity violation).

### ubvec Design

ncon=2 with:
- ubvec[0] = 1.001 (tight population balance, same as B.9)
- ubvec[1] = configurable partisan tolerance (D_votes constraint softness)

The partisan tolerance is the "compromise" parameter: at ubvec[1]=1.0 (hard),
strict proportionality is enforced. At ubvec[1]=∞, it reduces to pure
population balance (B.1/B.8). The paper characterizes the tradeoff curve.

---

## Empirical Plan

### 50-State Sweep

For each state:
1. Run B.9 (geographic-only) → record proportionality gap
2. Run B.12 (partisan-constrained) at ubvec[1] = 1.05, 1.10, 1.20, ∞
3. Compute proportionality gap at each setting
4. Plot gap vs partisan constraint strength (the tradeoff curve)

### Key States (case studies)

- **WI** (50.3% D, 3D/5R under B.9): how much partisan constraint reduces the gap?
- **NC** (49.3% D, 5D/9R under B.9): starts R-leaning, how does B.12 adjust?
- **GA** (50.1% D, 3:11 geographic split under B.9): extreme geographic concentration

### The Partisan Lorenz Figure

Plot partisan Lorenz curves for:
- Iowa (uniform density → curve near diagonal → proportionality easy)
- Wisconsin (Milwaukee concentration → curve above diagonal → needs constraint)
- Georgia (Atlanta concentration → curve far from diagonal → hardest case)

---

## Legal / Normative Framing

B.12 does NOT advocate for partisan redistricting. The framing is:

**"This paper answers a mathematical question: what is the minimum partisan
information required to guarantee proportional outcomes, and how far do
geography-only algorithms fall short? The answer has legal implications —
states with extreme geographic partisan sorting cannot achieve proportionality
through neutral algorithms alone, which is a fact about geography, not a
flaw in the algorithm."**

This connects to:
- *Rucho v. Common Cause* (2019): federal courts cannot remedy partisan gerrymandering
- B.9's Rodden effect: geographic sorting, not algorithm choice, drives the gap
- The normative question: is geographic proportionality achievable without
  explicit partisan optimization?

The paper measures the *distance* between geographic neutrality and partisan
proportionality, characterizing when that distance is zero (easy states) vs
large (concentrated-partisan states).

---

## Paper Outline

1. **Introduction**: the proportionality gap, why geographic algorithms can't close it
2. **Related Work**: efficiency gap, B.9, B.11, partisan Lorenz curves
3. **Framework**: ncon=2 [pop, D_votes], tpwgts formula, partisan Lorenz
4. **Theory**: proportionality lemma, feasibility condition, ubvec tradeoff
5. **Evaluation**: 50-state sweep, tradeoff curves, Lorenz feasibility
6. **The Compromise**: states where proportionality is cheaply achievable vs expensive
7. **Legal Discussion**: what B.12 means for redistricting reform proposals
8. **Conclusion**

---

## Connection to B.9 and B.11

**From B.9 (AreaSection)**:
- Same ncon=2 METIS infrastructure
- Same Lorenz curve feasibility analysis (partisan Lorenz = area Lorenz analog)
- Same isoperimetric normalisation for ratio selection
- Difference: vertex weight[1] = D_votes instead of land area

**From B.11 (PrimeFactor)**:
- HH provides k_D and k_R (only relevant for determining the bisection ratio)
- PrimeFactor tree structure applies recursively: D-bloc uses D-heavy bisection,
  R-bloc uses R-heavy bisection, each recursively achieving proportionality
- B.12 provides the "proportional leaf guarantee" that makes PrimeFactor
  politically meaningful
