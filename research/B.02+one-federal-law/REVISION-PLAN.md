# Revision Plan — B.02 One Federal Law (Districting Integrity Act)
Round 1 avg: 2.6/4
Round 2 avg: 3.0/4 (Karypis 3, Rodden 3, Duchin 3, Stephanopoulos 3, Liang 3)
Round 3 avg: 3.4/4 (Karypis 3, Rodden 4, Duchin 3, Stephanopoulos 4, Liang 3)
**STATUS: ACCEPTED (B-series internal track) — 3.4/4 ≥ 3.2 target with both P1s resolved**

## Round 3 Summary (2026-05-05)
- Karypis: 3/4 — isomorphism paragraph improves clarity; ubvec tolerance and k-way spec carry forward.
- Rodden: 4/4 — structural isomorphism substantially resolved; upgrades to Accept. P1.2/P1.3 remain journal conditions.
- Duchin: 3/4 — isomorphism conditional progress; "no political choices remain" overclaim and proof explicit statement carry forward.
- Stephanopoulos: 4/4 — VRA mutex scope precisely resolved; upgrades to Accept. Prime fallback enumeration carries forward.
- Liang: 3/4 — neither R3 item touches runtime/TIGER vintage P1s; unchanged.

## Key R3 Fixes Applied
- [x] Structural isomorphism limits paragraph added (Premise 4, Section 2): isomorphism in decision procedure not feasible set; HH for tree, METIS for contiguous realization
- [x] Proposition proof revised to remove circularity: adjacency constraint operates at realization level, not tree-selection level
- [x] VRA mutex scope paragraph added (Component 5, Section 5): in-run prohibition only; sequential comparison is the Callais strong-inference test itself

## Round 2 Summary (2026-05-05)
- Karypis: 3/4 — B.11 citation fixed, canonical ordering added; ubvec tolerance error and METIS k-way spec carry forward.
- Rodden: 3/4 — Elections Clause paragraph noted positively; structural isomorphism and Wisconsin circularity carry forward (require B.11).
- Duchin: 3/4 — Uniqueness for repeated primes resolved; Proposition proof circularity partially addressed; "no political choices" partially addressed.
- Stephanopoulos: 3/4 — Elections Clause paragraph resolves P1.1; VRA mutex scope and prime fallback list carry forward.
- Liang: 3/4 — B.11 citation fixed; runtime claim and TIGER/Line vintage carry forward.

## Key R2 Fixes Applied
- [x] B.11 "Table 3" citation replaced with companion-paper forward reference (NC+WI near-zero variance confirmed)
- [x] Canonical ordering for repeated prime factors added (k=12 → [3,2,2] tree specified)
- [x] Elections Clause paragraph added (Article I §4, Smiley v. Holm, algorithm-mandate as procedural regulation)

## P1 — Required (from reviewers)

- [ ] [Karypis R1.1] Resolve the B.11 zero-variance citation — either confirm B.11 is complete with Table 3 as an actual empirical result, or restate the zero seed variance claim as a theoretical prediction pending B.11 completion; update all downstream claims accordingly
- [ ] [Karypis R1.2] Specify METIS k-way vs. bisection for non-binary prime factors — for states where p_i > 2 (e.g., California k=52=4×13 requiring 4-way and 13-way partitions), specify whether `METIS_PartGraphKway` is used and provide exact ubvec and tpwgts settings for k-way runs
- [ ] [Karypis R1.3] Fix the ubvec tolerance inconsistency — ubvec[0]=1.001 means ±0.1% per-part tolerance, not ±0.05%; correct the "±0.05% deviation ceiling" claim and make consistent with B.0's "≤0.5% by triangle inequality" statement
- [x] [Rodden R2.1] Address the adjacency constraint gap in the structural isomorphism argument — RESOLVED R3: isomorphism in decision procedure (priority→tree), not feasible set; HH applies at bisection-ratio level, METIS handles contiguous realization; Rodden upgrades to 4
- [ ] [Rodden R2.2] Resolve the Wisconsin circularity — AR's 4D/4R outcome is used as both the problem (algorithm choice is political) and the solution (AR is constitutional); either show AR's proportional outcomes across all competitive states or explicitly disclaim proportionality as an AR property
- [ ] [Rodden R2.3] Develop a fuller answer to the Democratic geographic sorting objection — show AR outcomes in Massachusetts, Maryland, Connecticut, and other states where geographic sorting produces Democratic over-representation, or develop a principled argument for why AR is acceptable even in states where it produces Republican-leaning outcomes
- [ ] [Duchin R3.1] Fix the uniqueness claim for seat counts with repeated prime factors in different compositions — for k=12=2²×3, show that [3,2,2] and [2,2,3] split orderings produce the same geographic partition, or specify a uniqueness-preserving convention beyond "decreasing order" that handles this case
- [ ] [Duchin R3.2] Fix the circular Proposition proof — the proof concludes "AR is uniquely constitutionally derived" based on the undefined intrastate HH priority rule; either define the priority value of a census tract in the intrastate HH context, or reframe as a doctrinal rather than formal proof
- [ ] [Duchin R3.3] Replace "no political choices remain" with "no practitioner choices remain" — acknowledge that the statutory parameters (α=2.0, T=600, w_vra=0.40, census tract unit) are political choices encoded in the statute, not consequences of first principles
- [ ] [Stephanopoulos R4.1] Engage with the Elections Clause jurisdictional question — address whether Congress can mandate a specific computational algorithm (not just substantive standards) under Article I §4; identify this as a litigation risk and develop the best available response
- [x] [Stephanopoulos R4.2] Review VRA overlay mutex against actual Callais holding — RESOLVED R3: scope paragraph added; in-run prohibition only; sequential comparison is the Callais strong-inference test itself; Stephanopoulos upgrades to 4
- [ ] [Stephanopoulos R4.3] Complete the prime fallback state list — enumerate all six states where k is prime and k≥3 in the current apportionment explicitly in the statutory text
- [ ] [Liang R5.1] Qualify the 30-minute 50-state runtime claim — restrict to GeoSection-based benchmarked components or label as estimated pending B.11 implementation
- [ ] [Liang R5.2] Specify TIGER/Line vintage pinning — identify which TIGER/Line vintage year is canonical for each census decade and add to the statutory text or mandate EAC specification of the canonical vintage

## P2 — Suggested

- [ ] [Karypis R1.P2a] Add runtime complexity comparison for AR bisection tree vs. direct k-way for large states (TX, CA) — demonstrate 30-minute feasibility for the HH tree's k-way METIS calls
- [ ] [Karypis R1.P2b] Clarify factorisation-spine "exact nesting" claim — specify that spine score 0 measures factorisation compatibility, not actual geometric district boundary nesting
- [ ] [Rodden R2.P2a] Cite Rodden-Chen result with full precision — specify that concentrated urban Democratic populations drive the result and that it does not hold uniformly across all states
- [ ] [Rodden R2.P2b] Remove or substantially qualify "airtight" claim from introduction
- [ ] [Duchin R3.P2a] Engage with geometric mean vs. arithmetic mean divisor alternatives (HH vs. Webster/Sainte-Laguë) at the intrastate level
- [ ] [Duchin R3.P2b] Acknowledge formal limitations of isomorphism argument and lean more heavily on doctrinal extension framing
- [ ] [Stephanopoulos R4.P2a] Reframe "one sentence law" as "one operative principle" — acknowledge six implementing provisions
- [ ] [Stephanopoulos R4.P2b] Address mid-decade VRA obligation triggers — develop statutory language for court-ordered mid-decade redistricting when demographic shifts create new Gingles obligations
- [ ] [Liang R5.P2a] Add convergence threshold enforcement provision to statutory text — prevent research-mode runs (T<600) from generating legally effective maps
- [ ] [Liang R5.P2b] Specify manifest schema versioning — EAC delegation for schema evolution across census decades
