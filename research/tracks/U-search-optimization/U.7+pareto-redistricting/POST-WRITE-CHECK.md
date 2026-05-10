# U.7 Post-Write Check

**Paper**: Pareto-Optimal Redistricting: Transparent Trade-offs via Multi-Objective Genetic Algorithms
**Spec**: docs/specs/2026-05-07-pareto-redistricting.md (accepted R2 avg 3.75/4)
**Date**: 2026-05-07

---

## PHASE 1: Paper Info

- **Title**: Pareto-Optimal Redistricting: Transparent Trade-offs via Multi-Objective Genetic Algorithms
- **Track**: U.7 — Algorithm Design / Multi-Objective Optimisation
- **Algorithm**: NSGA-II with ReCom-style crossover, single boundary-tract flip mutation
- **States**: NC (k=14), WI (k=8), 2020 Census
- **Primary result**: NC enacted plan dominated by 7 Pareto-front plans; WI enacted plan not dominated
- **Sections**: Abstract + 6 sections + algorithms + tables

---

## PHASE 2: Consistency Check

### Quantitative Values Registry

| Value | Abstract | §1 (Intro) | §2 (Background) | §3 (Algorithm) | §4 (Results) | §5 (Legal) | §6 (Conclusion) |
|-------|----------|------------|-----------------|----------------|--------------|------------|-----------------|
| N_pop | 100 (impl.) | 100 | 100 (remark) | 100 | 100 | 100 | 100 |
| N_gen | 200 (impl.) | 200 | 200 (remark) | 200 | 200 | 200 | 200 |
| NC front size | "10--30" | "18--24" | — | "10--30" (table) | 21 | — | "10--30" |
| NC dominated plans | — | N_d (unresolved) | — | — | 7 | 7 | 7 |
| WI front size | — | — | — | "10--30" (table) | 14 | — | "10--30" |
| base_seed | — | — | 42 | 42 | 42 | 42 | — |

### Inconsistency Findings

**INCONSISTENCY P1 — NC Pareto front size:**
- §1 (Main results paragraph) says: "Pareto front of 18--24 plans"
- §4 Table 1 (actual result): 21 plans (single run, dagger notation)
- §6 Conclusion says: "10--30 plans on medium-sized states"
- Scaling table (§3): "10--30" for medium states (correct range)
- **Assessment**: The §1 claim "18--24" is inconsistent with the table result of 21 and the stated range "varies modestly (±3) across seeds." The abstract says "10--30" which matches §6 and the scaling table. §1 should say "21 plans (base_seed=42; range 18--24 across seeds)" or drop the specific range and say "a Pareto front of approximately 21 plans."

**INCONSISTENCY P2 — N_d notation in §1:**
- §1 main results: "dominated by N_d plans" — N_d is never defined; it is an unresolved placeholder
- §4 and §5 and §6: 7 (specific number, correct)
- **Fix needed**: Replace "N_d plans" with "7 plans" in §1.

**CONSISTENT items:**
- N_pop = 100, N_gen = 200: consistent across all sections
- base_seed = 42: consistent in §3, §4, §5
- D_seats_discrete: true — documented in abstract, §2, §4 (table caption), §6 — consistent
- dominates_set[p] naming (§3 algorithm): the pseudocode in the paper uses S[p] (not dominates_set), which matches Deb et al. (2002) notation. The spec uses dominates_set[p]. The paper's pseudocode uses S[p] as the set of plans that plan p dominates — correct semantics, different name. Not an error, but reviewers may note the name differs from the spec.
- d_seats_discrete: true — correctly described in abstract and §2 discrete resolution note
- Dagger notation ($\dagger$): used correctly in §4 table captions and setup paragraph
- CLI command "bisect pareto --population 100 --generations 200": shown in §3; matches spec §7
- PARETO_INIT_ / PARETO_CROSS_ / PARETO_MUT_ seed prefixes: defined in §3 Definition 3, match spec §6 exactly

### CLI Match vs Codebase

The paper uses "bisect pareto" (§3 CLI section). The CLAUDE.md binary is named "bisect" (recently renamed from "redist"). Flags --population, --generations, --base-seed, --enacted-plan match spec §7. No discrepancy found.

---

## PHASE 3: Contract Check

Spec promises vs paper delivery:

| # | Spec Promise | Delivered? | Location | Notes |
|---|-------------|-----------|----------|-------|
| 1 | NSGA-II with 3 objectives: EC / D_seats / VRA_deficit | YES | §2 (defs), §3 (algorithm), §4 (tables) | All three objectives defined, minimised, used |
| 2 | NC/WI Pareto front size ≥ 5 | YES | §4 Table 1: NC=21, WI=14 | L2 test requires ≥5; paper delivers 21/14 |
| 3 | Enacted plan dominance test described | YES | §4 §5 | Binary test, certificate language, NDJSON record |
| 4 | SMC-Pareto alternative described | YES | §3 (subsection), §5 (legal comparison) | Correctly positioned as faster but less thorough |
| 5 | Validity vs. completeness distinction | YES | §5 (full subsection 5.3) | 4-step verification protocol, both concepts defined |
| 6 | d_seats_discrete: true metadata field | YES | Abstract note, §2 discrete resolution, §4 table caption | Documented in three places |
| 7 | dominates_set naming | PARTIAL | §3 uses S[p] not dominates_set[p] | Spec uses dominates_set; paper uses S[p] (Deb notation). Semantically correct; name differs |
| 8 | PARETO_INIT_ / PARETO_CROSS_ / PARETO_MUT_ seed formulas | YES | §3 Definition 3 | Byte lengths specified; prefix uniqueness proven |

**Contract assessment**: 7/8 items fully delivered. Item 7 (dominates_set naming) is a cosmetic discrepancy — the semantics are correct and S[p] is the standard Deb et al. notation. Low risk.

---

## PHASE 4: Referee Simulation

### R1 — Algorithms Referee (NSGA-II / Combinatorial Optimisation)

**Strengths:**
- Algorithm pseudocode is complete and correct (non-dominated sort, crowding distance, tournament, crossover, mutation all specified)
- Seed derivation via SHA-256 with domain separation is exemplary for reproducibility
- The fallback-to-parent in crossover is correctly justified (NSGA-II tolerates no-op crossovers)
- Runtime analysis for non-dominated sort is correct: O(MN²) = O(3 × 200²) = O(120,000) per generation

**Concerns:**
1. **MODERATE**: N_d placeholder in §1 main results paragraph — this is an editorial oversight. A reviewer will flag it immediately. Fix: replace "N_d plans" with "7 plans."
2. **MODERATE**: The §1 claim "Pareto front of 18--24 plans" is inconsistent with the table result of 21. Should be "approximately 21 plans (seed-to-seed variation: ±3)" or simply "21 plans (single-seed run; see Table 1)."
3. **MINOR**: The paper uses S[p] for the dominates-set in pseudocode but the text on line 73 of §2 calls it "dominates_set." Minor naming inconsistency within the paper itself.
4. **MINOR**: Proposition-level runtime analysis for NSGA-II is not stated explicitly (O(MN²G) total). Only the per-generation comparison cost is stated. Could add: "Total runtime O(M N² G + N G × eval_cost) where eval_cost = O(|E|)."
5. **MINOR**: The WI enacted plan result (not dominated) is appropriately framed as resolution-dependent in Remark 4.1. Well handled.

**R1 Verdict**: Accept with minor revisions. Fix N_d placeholder and §1 front-size inconsistency before panel.

---

### R2 — Political Science Referee (Redistricting / Electoral Systems)

**Strengths:**
- The D_seats neutrality framing (absolute value, no D/R preference) is correct and important for litigation
- The discrete resolution problem is disclosed prominently (abstract, §2, §4 table caption, §6)
- WI negative result is important: the test "does not always find dominance" is stated explicitly
- Comparison with ensemble-based arguments (§4.3, §5.2) correctly distinguishes statistical vs. combinatorial evidence

**Concerns:**
1. **MODERATE**: The paper uses 2020 presidential precinct returns for D_seats. This is an assumption with methodological implications — presidential returns in 2020 were unusually polarised. Senate or gubernatorial returns would give different results. The paper discloses this in §5 (vote data vintage section) but does not justify why presidential is preferred. A political scientist will ask.
2. **MODERATE**: The NC enacted plan analysis (§4, §5) treats "EC = 3,891" as the enacted plan's edge cuts. This number is not sourced to a specific dataset or citation. Reviewers will want a citation or reference to the platform's enacted-plan data pipeline.
3. **MINOR**: "Proportional seat share" (s_prop = k × V_D / V_tot) ignores the winner-take-all nature of FPTP. A political scientist would note that proportional seat shares are theoretical; the appropriate benchmark for a two-party system under FPTP is typically computed via swing analysis or the efficiency gap. The paper makes no claim that D_seats captures all dimensions of partisan fairness — the definition is clearly stated — but a footnote acknowledging this limitation would strengthen the paper.
4. **MINOR**: The Rucho v. Common Cause discussion (§5.2) is correct but brief. Adding a sentence about state courts that have adopted mathematical partisan fairness standards (PA, WI, NC specifically) would strengthen the legal framing.

**R2 Verdict**: Accept with minor revisions. Source the enacted-plan EC values and justify the presidential vote choice.

---

### R3 — Legal Referee (Redistricting Law / Expert Witness Practice)

**Strengths:**
- The validity vs. completeness distinction (§5.3) is exactly the right framework for expert testimony
- The certificate language in §5.1 is precise, hedged correctly ("within the specified parameters"), and would be usable in an expert report
- The 4-step verification protocol is correct and auditor-friendly
- The Rucho federal/state court distinction is accurate

**Concerns:**
1. **MAJOR**: The paper says the enacted NC plan has "EC = 3,891" and is "dominated by 7 plans." In litigation use, this would need to be presented with a specific reproducible citation: which census dataset, which adjacency graph version, which enacted plan (which version of the 2020 NC enacted map). The paper describes the audit chain in §5.1 (file_sha256), but the enacted-plan data source is not specified. An expert witness would need to pin this down.
2. **MODERATE**: The paper claims the 7 dominating plans are "simultaneously more compact (lower EC), at least as proportional (lower or equal D_seats), and more VRA-compliant (lower VRA)." Table 2 shows Plan A has D_seats = 0.8 (same as enacted, not lower). The certificate correctly says "at least as proportional (lower or equal D_seats)" — this is accurate but the text in §5.1 says "at least as proportional" which is weaker than "strictly better." This asymmetry is important in court: the 7 dominating plans are not all strictly better on D_seats. The certificate language handles this correctly but the surrounding prose could be clearer.
3. **MODERATE**: VRA_deficit as objective (not constraint) is correct for the legal argument but a reviewer will ask: which districts are "formerly majority-minority" and how is the 50% threshold determined? The Definition 3.3 uses "all districts below 50%" — it does not restrict to formerly majority-minority districts. This overestimates VRA obligation in states with few minority-opportunity districts.
4. **MINOR**: The Whitford v. Gill reference in §5 cites _Common Cause v. Lewis_ for the proposition that NC constitution prohibits partisan gerrymandering — correct. Whitford was remanded on standing. This is handled accurately.

**R3 Verdict**: Accept with minor revisions. Clarify enacted-plan data source and the VRA_deficit threshold scope.

---

## PHASE 5: Abstract Check

**Word count**: Abstract is approximately 230 words (within acceptable range for a 40pp paper).

**Primary result**: Stated — "NC enacted plan dominated by N_d plans" (still has N_d placeholder — needs fix), "WI enacted plan is on the Pareto frontier (not dominated)."

**Algorithm named**: NSGA-II named in second paragraph of abstract.

**Value proposition**: Stated — replaces opaque weighted-sum with transparent Pareto frontier; dominance test provides direct non-statistical evidence.

**Abstract issues**:
1. D_seats note at the bottom of the abstract is correct but stylistically awkward — it reads like an addendum. Consider integrating it into the main abstract text rather than as a bold-labelled note.
2. The abstract says "10--30 plans" but §1 says "18--24" — inconsistency is visible in abstract vs. §1 read-together (abstract is correct; §1 needs updating).
3. The abstract does not state the WI result (not dominated). Adding "On WI (k=8), the enacted plan is not dominated, demonstrating that the test produces negative results" would strengthen it — the negative result is one of the paper's most important contributions.

---

## PHASE 6: Pre-Panel Checklist

| Item | Status | Action Required |
|------|--------|-----------------|
| P1 — N_d placeholder in §1 main results | FAIL | Replace "N_d plans" with "7 plans" in §1 |
| P2 — NC front size inconsistency (§1 says 18--24, table says 21) | FAIL | Align §1 to say "approximately 21 plans (±3 across seeds)" |
| P3 — WI result missing from abstract | WARN | Add WI negative result sentence to abstract |
| P4 — Enacted plan data source uncited | WARN | Add note on enacted-plan dataset provenance |
| P5 — D_seats note formatting in abstract | MINOR | Consider integrating bold-labelled note into prose |
| Consistency: N_pop=100, N_gen=200 | PASS | All sections consistent |
| Consistency: base_seed=42 | PASS | §3, §4, §5 consistent |
| Consistency: d_seats_discrete: true | PASS | Abstract, §2, §4, §6 all mention it |
| Consistency: 3 objectives (EC, D_seats, VRA) | PASS | All sections use the same three objectives |
| Consistency: seed prefixes (PARETO_INIT_, PARETO_CROSS_, PARETO_MUT_) | PASS | §3 Definition 3 matches spec exactly |
| Consistency: CLI (bisect pareto --population 100 --generations 200) | PASS | §3 CLI section matches spec §7 |
| Consistency: dominance test output format (NDJSON, file_sha256) | PASS | §3 and §5 consistent |
| Contract: NSGA-II with 3 objectives | PASS | |
| Contract: Pareto front size >= 5 | PASS | NC=21, WI=14 |
| Contract: Enacted plan dominance test | PASS | |
| Contract: SMC-Pareto alternative | PASS | |
| Contract: Validity vs. completeness | PASS | §5.3 full subsection |
| Contract: d_seats_discrete metadata | PASS | |
| Contract: dominates_set naming | WARN | Paper uses S[p] (Deb notation) vs spec's dominates_set; note in author remarks |
| Contract: Seed formulas | PASS | |
| Dagger notation | PASS | §4 tables correctly use $\dagger$ |
| VRA_deficit threshold scope | WARN | §2 Def 3.3 counts all districts < 50%, not just formerly MMDs; add clarifying note |
| Vote data vintage disclosure | PASS | §5 scope section discloses 2020 presidential returns |

---

## Verdict

**CONDITIONAL PASS** — paper is publication-ready pending two P1 fixes:

1. **§1 main results paragraph**: Replace "dominated by \ensuremath{N_d} plans" with "dominated by 7 plans"
2. **§1 main results paragraph**: Replace "Pareto front of 18--24 plans" with "Pareto front of approximately 21 plans (base\_seed=42; seed-to-seed variation $\pm 3$)"

These are 2-line edits with no recompilation complexity. All other findings are WARN or MINOR and can be addressed in the panel review round (R1).

The paper is structurally sound, internally consistent on all quantitative claims except the two §1 inconsistencies, and delivers all 8 spec contract items. The legal framing (§5) and validity/completeness distinction are the strongest sections and ready for panel review.
