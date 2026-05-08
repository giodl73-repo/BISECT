# POST-WRITE CHECK — B.25 Moving-Knife Algorithm
**Date**: 2026-05-07  
**Paper**: "Moving-Knife Redistricting: Fair-Division Theory for Maximum Reock Compactness"  
**Validator**: research-post-write pipeline (6 phases)

---

## PHASE 1 — PAPER PROFILE

| Field | Value |
|---|---|
| Series code | B.25 — Algorithm Design / Geometric Structure |
| Algorithm | Moving-Knife Algorithm (MKA), `SplitStrategy::MovingKnife` |
| CLI flag | `--structure moving-knife` |
| Primary claim | MKA is the first redistricting algorithm that explicitly maximises Reock; achieves highest min-Reock on NC, FL, WA |
| Secondary claim | Moving-knife provides a fair-division certificate more accessible to non-technical judges than METIS/CVD arguments |
| Target audience | Algorithms researchers + redistricting practitioners + litigation attorneys |
| Structure | 6 sections: introduction (legal landscape + gap + contributions), background (theory + Reock + Welzl + related), algorithm (pseudocode + complexity + CLI + audit), comparison (3-state empirical table), fair-division legal argument, conclusion |

---

## PHASE 2 — CONSISTENCY CHECK

### Quantitative Value Registry

| Value | Abstract | §1 Intro | §3 Algorithm | §4 Comparison | §6 Conclusion | VERDICT |
|---|---|---|---|---|---|---|
| `n_orientations` default | 180 ("every 1°") | 180 | 180 | 180 | 180 | PASS |
| NC improvement (min-Reock) | "approximately 8%" (no exact) | "approximately 8%" | — | +8.5% (+0.019 over BFS) | — | PASS — abstract rounds, §4 specifies |
| FL improvement (min-Reock) | — (not stated) | "approximately 14%" | — | +13.8% (+0.025 over BFS) | — | PASS — §1 rounds, §4 specifies |
| WA improvement (min-Reock) | — (not stated) | — | — | +11.8% (+0.024 over BFS) | — | PASS — not in abstract or §1, §4 only |
| States tested | NC (k=14), FL (k=28), WA (k=10) | NC (k=14), FL (k=28), WA (k=10) | — | NC (k=14), FL (k=28), WA (k=10) | NC, FL, WA | PASS |
| 200-iteration rebalance | — | mentioned | §3.1 (Choice 3) and pseudocode | — | mentioned | PASS |
| Boundary-swap rebalance | — | yes | yes | — | yes | PASS |
| Reock range clamped to [0,1] | — | — | §3.2 Remark and formula | §2 Remark 2.1 | — | PASS |
| Dagger notation (single-run) | — | — | — | Table 4.1 (§4.1), Table 4.2 | — | PASS — consistently marked |
| Welzl MEC expected O(m) | abstract | §1.1 | §3.4 (Prop) | — | §6 | PASS |
| MKA runtime complexity | O(n_orient × m) | O(n_orient × m) | O(n_orient · m log m) [with sort] | — | O(n_orient × m) | **WARNING — see below** |

### P1: Runtime Complexity Inconsistency

**Finding**: There is a minor terminology inconsistency between sections.

- **Abstract** states: "Runtime is $O(n_{\mathrm{orient}} \times m)$ per bisection node"
- **§1 Introduction** (main result paragraph): "$O(n_{\mathrm{orient}} \times m)$ runtime"
- **§3 Algorithm Table 2.4** (algorithm comparison table, §2): "$O(n_{\mathrm{orient}} \times m)$"
- **§3 Proposition 3.1** (formal): "$O(n_{\mathrm{orient}} \cdot m \log m)$" — the sort step is $O(m \log m)$ per orientation, making total $O(n_{\mathrm{orient}} \cdot m \log m)$

The proposition is technically correct: the sort step per orientation is $O(m \log m)$, and Welzl is $O(m)$, so the per-orientation cost is $O(m \log m)$ dominated by sort. The abstract/introduction simplify to $O(n_{\mathrm{orient}} \times m)$, which is only exact if using radix sort.

**Verdict**: The paper notes the radix-sort path gives $O(180m)$ strictly, and the proposition explicitly states $O(n_{\mathrm{orient}} \cdot m \log m)$ as the tight bound with comparison sort. The abstract's "$O(n_{\mathrm{orient}} \times m)$" is the radix-sort case, not the default. This is **not wrong but is potentially misleading** — a referee may flag it. The proposition is the correct formal statement; the abstract simplification should be marked as the radix-sort case or qualified with "for fixed $n_{\mathrm{orient}}$."

**Action needed**: Qualify the abstract runtime as "$O(n_{\mathrm{orient}} \times m \log m)$, reducible to $O(n_{\mathrm{orient}} \times m)$ with radix sort" or keep consistent with the formal proposition.

### CLI Flag Verification (against codebase)

| Flag in paper | Flag in codebase | Match? |
|---|---|---|
| `--structure moving-knife` | `--structure moving-knife` (StructureMode::MovingKnife) | PASS |
| `--mka-orientations 180` | `--mka-orientations` default 180 | PASS |
| `--mka-metric reock` | `--mka-metric` default "reock" | PASS |
| `--mka-metric polsby` | `--mka-metric` accepts "polsby" | PASS |
| `--structure area-section --area-section-init moving-knife` | AreaSectionInit::MovingKnife present in spec; not verified in code | UNVERIFIED (see below) |

**P2 (minor)**: The `--area-section-init moving-knife` flag for the hybrid mode is described in §3.5 and the spec. The grep of the codebase found `SplitStrategy::MovingKnife` and `AreaSectionInit` mentioned in the spec but not confirmed implemented in the current codebase. Task #162 ("MKA-AreaSection hybrid") is still **pending**. The paper presents this as a current feature ("The `split_subgraph_mka_direction` function returns the optimal angle..."). This could be flagged as overclaiming if the hybrid is not yet implemented.

**Action needed**: Verify whether `--area-section-init moving-knife` is currently implemented. If not yet done (Task #162 pending), add a Phase 2 caveat to §3.5, or add a footnote: "The hybrid mode is available from the platform; implementation in crate `bisect-cli` pending."

### SHA-256 Seed Formula Consistency

| Location | Formula |
|---|---|
| Spec §4 | `SHA-256("MKA_INIT_" \|\| node_path_len:u32le \|\| node_path.as_bytes() \|\| base_seed:u64le)` |
| §3 Definition 3.1 (paper) | `SHA-256("MKA_INIT_" \|\| le4(\|node_path\|) \|\| node_path \|\| le8(base_seed))` |
| §3 Audit chain (paper) | `"SHA-256('MKA_INIT_' \|\| node_path_len:u32le \|\| node_path \|\| base_seed:u64le)"` |
| Codebase `MKA_INIT_` prefix | Confirmed present in `bisection_runner.rs:3408` | 

**Verdict**: PASS — all three representations are consistent; the math notation and string notation match.

---

## PHASE 3 — CONTRACT CHECK (Spec vs. Paper)

The spec (`docs/specs/2026-05-07-moving-knife.md`) was accepted at R2 avg 3.75/4. Each spec promise is checked against the paper.

| # | Spec Promise | Found in Paper? | Location | Verdict |
|---|---|---|---|---|
| 1 | NC/FL/WA Reock comparison table showing MKA beats METIS/CVD/BFS | Yes | Table 4.1 (§4.2) — full 4-algorithm × 3-state table | PASS |
| 2 | Runtime O(n × n_orientations) per bisection node | Yes — with important qualification | §3 Prop 3.1 gives O(n_orient · m log m); abstract/intro say O(n_orient × m) | PASS with WARNING (see P1) |
| 3 | Fair-division legal argument (§5) | Yes | §5 (§sec:fair-division) — Rawlsian, METIS comparison, limitations | PASS |
| 4 | MEC via Welzl's algorithm | Yes | §2.3 Prop 2.1, §3.4, §3 pseudocode line 17 | PASS |
| 5 | Seeding: SHA-256("MKA_INIT_" \|\| ...) formula documented | Yes | §3.3 Definition 3.1 | PASS |
| 6 | AreaSection direction integration described | Yes | §3.5 CLI section — `split_subgraph_mka_direction` described with hybrid CLI example | PASS (implementation status caveat — P2 above) |
| 7 | Reock clamping to [0,1] with centroid approximation caveat | Yes | §2 Remark 2.1 (full caveat) + §3 pseudocode comment + Reock subroutine formula | PASS |
| 8 | Audit chain fields documented | Yes | §3.6 — all fields present: structure, mka_orientations, mka_metric, mka_seed_formula, node_path, base_seed, mka_seed, optimal_angle_deg, reock_left, reock_right, min_reock_score, convergence_warning | PASS |

**Contract verdict**: 8/8 spec promises delivered. One warning (P1 runtime), one pending-implementation note (P2 hybrid mode).

---

## PHASE 4 — REFEREE SIMULATION

### R1 — Algorithms Reviewer (complexity, proofs, baselines)

**Profile**: Expects tight complexity statements, clear pseudocode, well-defined baselines, comparison with known algorithms.

**Findings**:

**A (Positive)**:
- Proposition 3.1 is careful and explicit about the sort step dominating: $O(n_{\mathrm{orient}} \cdot m \log m)$. The radix-sort optimisation path is noted.
- Pseudocode (Algorithm 3.1) is complete and matches the spec pseudocode exactly.
- Welzl's MEC algorithm is cited correctly (Proposition 2.1), with the expected-$O(m)$ bound clearly stated.
- The note that Welzl is called $2 \times n_{\mathrm{orient}}$ times per node (once per half, per orientation) is correct.
- Four-algorithm baseline comparison is methodologically sound: METIS, CVD-Geo, BFS-Growth, MKA.

**Concern 1 (Major — will raise)**: The abstract states runtime $O(n_{\mathrm{orient}} \times m)$, but Proposition 3.1 correctly states $O(n_{\mathrm{orient}} \cdot m \log m)$ due to the sort step. The paper resolves this by noting radix sort reduces it to $O(n_{\mathrm{orient}} \times m)$, but the abstract does not flag this as the radix-sort case. A referee reading the abstract in isolation will see $O(n_{\mathrm{orient}} \times m)$ and then read $O(n_{\mathrm{orient}} \cdot m \log m)$ in the proposition — this is technically inconsistent in the paper as printed and will generate a reviewer comment.

**Concern 2 (Minor)**: The comparison table (Table 2.4) lists CVD runtime as $O(n_{\mathrm{iter}} \times m)$ with a "BFS hop-count" distance. This is the CVD (graph-distance) variant. CVD-Geo is listed separately as $O(n_{\mathrm{iter}} \times m)$ with Euclidean distance. The §4 comparison uses "CVD-Geo" but the table lists both CVD and CVD-Geo. A referee may ask: which CVD is in the empirical comparison? (Answer: CVD-Geo, clearly stated in §4.1.) Low concern, but a clarifying sentence would help.

**Concern 3 (Minor)**: No proof that the sweep finds a global Reock maximum even for convex subgraphs. The paper correctly acknowledges this as a heuristic for non-convex regions (§5.4 limitation), but for convex regions, is the sweep provably optimal? The paper does not claim it is; but it also does not prove it. For an algorithms venue, a sentence noting "for convex subgraphs, the orientation sweep over $[0°, 180°)$ is exhaustive over all bisecting hyperplane directions, so the result is globally optimal within the centroid approximation" would strengthen the claim.

**R1 verdict**: Accept with minor revisions. P1 (runtime consistency) is a mandatory fix. Concerns 2 and 3 are optional but recommended.

---

### R2 — Political Science Reviewer (partisan neutrality, single-run validity, court claims)

**Profile**: Expects ensemble-averaging for empirical claims, partisan analysis, caution about court claims.

**Findings**:

**A (Positive)**:
- The paper is appropriately cautious: Table 4.1 is marked $\dagger$ (single run) throughout, with explicit text: "the purpose of this table is to illustrate *relative ordering* of Reock performance, not to report production-quality compactness figures."
- The partisan neutrality check (Table 4.3) is present and well-framed: D-seats are identical across all three algorithms on all three states, and the paper correctly notes this is seed-42 only.
- VRA limitations are explicitly noted (§5.4): the algorithm does not account for racial composition.
- The multi-criterion limitation (§5.4) acknowledges that Florida requires both Reock and Polsby-Popper.
- Bisection-tree structure limitation is noted (§5.4): local Reock maximisation does not guarantee global optimum.

**Concern 1 (Moderate)**: The partisan neutrality table uses only seed $s=42$ and shows identical D-seat outcomes across algorithms. A political science reviewer will correctly note that this is a single-seed result, not ensemble evidence. The paper should add a one-sentence qualifier: "Partisan neutrality is shown here for a single seed; ensemble verification across seeds is deferred to the G.15 comprehensive comparison."

**Concern 2 (Minor)**: The fair-division certificate argument in §5.2 quotes Puppe and Tasnadi (2026) as proving "uniqueness" of the MKA procedure among procedures satisfying anonymity, population balance, and compactness maximisation. The paper attributes an axiomatic uniqueness result to Puppe-Tasnadi (2026). If this citation is to a preprint, the strength of the uniqueness claim should be noted as "forthcoming" or "unpublished." As a spec decision, the paper is permitted to rely on it, but a reviewer may push back on a uniqueness result from a preprint.

**Concern 3 (Minor)**: The three court cases cited (Common Cause v. Lewis NC 2019, Whitford v. Gill WI 2016, League of Women Voters v. Lee FL 2022) are all real. The paper correctly notes the SC remanded Whitford on standing (§1.1). The FL case name ("League of Women Voters v. Lee, 2022") uses "Lee" — this should be verified against the actual case name (typically the Florida SoS at the time). Low concern for an algorithms paper but a legal reviewer will check it.

**R2 verdict**: Accept with minor revisions. Add ensemble caveat to partisan neutrality table. Note Puppe-Tasnadi citation status.

---

### R3 — Legal/Practitioner Reviewer (case citation accuracy, operationalizability)

**Profile**: Redistricting practitioner or voting rights attorney. Expects accurate case names, correct legal holdings, and operationalizable recommendations.

**Findings**:

**A (Positive)**:
- §5 is well-structured for legal audiences: the plain-language certificate (§5.2 quote block) is excellent and court-ready.
- Rawlsian framing (§5.3) is appropriate and correctly cited (Rawls 1971).
- The three contexts where MKA is preferable (§5.4) are practical and well-reasoned.
- Limitations are appropriately caveated: VRA, multi-criterion, non-convexity, bisection tree.
- The recommendation to use `bisect analyze --types vra` for VRA compliance is concrete and actionable.

**Concern 1 (Moderate)**: **Florida case name accuracy**. The paper cites "*League of Women Voters v. Lee* (Florida, 2022)." The Florida League of Women Voters cases against SoS Lee involve voter registration and election administration, not redistricting compactness. The primary Florida redistricting compactness litigation is *League of Women Voters of Florida v. Florida House of Representatives* (under the Fair Districts Amendment, Article III §20) from the 2010–2014 period, and the 2022 congressional redistricting was challenged in *Florida State Conference of NAACP v. Lee* and *Soto v. DeSantis*, not a League of Women Voters case specifically named v. Lee. The paper should verify this citation or replace it with a confirmed Florida redistricting-Reock case.

**Concern 2 (Minor)**: The paper cites Florida's Article III, §20 as generating "sustained litigation" with Reock and Polsby-Popper (§5.4). This is accurate. However, it also states "Florida courts have applied Reock alongside Polsby-Popper" — this is a strong claim. Florida's Fair Districts litigation primarily involved visual compactness analysis rather than formal Reock scoring in a quantitative sense; if Reock was used explicitly in expert reports, a citation to the specific expert report or case document would strengthen this.

**Concern 3 (Minor)**: *Common Cause v. Lewis* (NC, 2019) is correctly cited as striking down the 2017 maps. The paper states the remedial order required that the remedial plan "not be drawn with the intent or effect" of producing Reock scores below the enacted-plan baseline. This is a specific factual claim about the order's language. It should be verified against the actual remedial order text, and a citation to the remedial order (not just the case) would be appropriate.

**R3 verdict**: Accept with revision. The Florida case citation (Concern 1) is a **mandatory fix** — the case name appears to be misidentified. The other concerns are recommended improvements.

---

## PHASE 5 — ABSTRACT CHECK

**Word count** (estimated): ~210 words. Appropriate for a conference/journal abstract (150–250 word target range). PASS.

**Algorithm named**: Yes — "Moving-Knife Algorithm (MKA)" in sentence 1. PASS.

**Primary result stated**: Yes — "MKA achieves the highest minimum Reock score among four algorithms on North Carolina (k=14), Florida (k=28), and Washington (k=10)." PASS.

**Value proposition**: Yes — "fair-division certificate: a procedurally neutral, legally articulable justification... complements — and in court contexts may be more persuasive than — the purely mathematical arguments available for other algorithms." PASS.

**Theoretical basis**: Yes — Dubins-Spanier 1961, Puppe-Tasnadi 2026, Welzl 1991, Albers EPSG:5070. PASS.

**CLI recommendation**: Yes — `--structure moving-knife`. PASS.

**Missing from abstract**: The abstract does not state the improvement magnitude (e.g., "8–14% over BFS-Growth"). This is acceptable (§1 gives the range), but a single headline number would strengthen the abstract for practitioners scanning titles.

**Abstract verdict**: PASS with optional enhancement (add improvement range).

---

## PHASE 6 — PRE-PANEL CHECKLIST

### P1 Items (Mandatory — must fix before submission)

| ID | Issue | Location | Fix |
|---|---|---|---|
| **P1-A** | Runtime inconsistency: abstract/§1/Table 2.4 say $O(n_{\mathrm{orient}} \times m)$; Proposition 3.1 correctly says $O(n_{\mathrm{orient}} \cdot m \log m)$ | Abstract, §1 main result, §2 Table caption | Qualify abstract as "$O(n_{\mathrm{orient}} \times m \log m)$, or $O(n_{\mathrm{orient}} \times m)$ with radix sort" to match the formal proposition |
| **P1-B** | Florida case name likely misidentified: "*League of Women Voters v. Lee* (FL, 2022)" does not appear to be a redistricting-Reock case under that name | §1.1 and abstract | Verify the actual Florida redistricting compactness case name and replace; candidates include *Soto v. DeSantis* or a Fair Districts Amendment case |

### P2 Items (Recommended — should fix before submission)

| ID | Issue | Location | Fix |
|---|---|---|---|
| **P2-A** | Hybrid `--area-section-init moving-knife` described as current feature but Task #162 is pending | §3.5 CLI section | Add footnote: "AreaSection hybrid mode implementation is in progress; pure `--structure moving-knife` is fully implemented." |
| **P2-B** | Partisan neutrality table: single-seed result (s=42) only | §4.4 / Table 4.3 | Add one sentence: "Ensemble verification across seeds is deferred to G.15 comprehensive comparison." |
| **P2-C** | Puppe-Tasnadi (2026) uniqueness result attributed from preprint | §5.3 | Note citation status ("forthcoming, *Public Choice*") |
| **P2-D** | Common Cause v. Lewis remedial order language cited without sub-citation | §1.1 | Add citation to the specific remedial order document |
| **P2-E** | For convex subgraphs: paper does not state whether sweep is provably optimal | §3, Prop 3.1 | Optional: add note "for convex subgraphs, the orientation sweep over $[0°, 180°)$ is exhaustive over all bisecting hyperplane directions, so MKA is optimal within the centroid approximation" |

### P3 Items (Optional polish)

| ID | Issue | Location | Fix |
|---|---|---|---|
| **P3-A** | Abstract does not include improvement magnitude (8–14%) | Abstract | Add: "with minimum-Reock improvements of 8–14% over the next-best algorithm" |
| **P3-B** | WA improvement (11.8%) not mentioned in §1 introduction or abstract | §1.1 and abstract | Add WA to the introduction range: "ranges from approximately 8% (NC) to 14% (FL), with WA at 12%" |
| **P3-C** | Florida Polsby-Popper multi-criterion claim ("Florida courts have applied Reock alongside Polsby-Popper") | §5.4 | Strengthen with a citation to the expert report or judicial opinion |

---

## OVERALL VERDICT

**Paper quality**: Strong. The paper is well-structured, technically correct, and delivers all 8 spec contract items. The pseudocode is precise, the audit chain is complete, and the fair-division legal argument is genuinely novel and accessible.

**Blocking issues**: 2 (P1-A runtime consistency; P1-B Florida case name)

**Panel readiness**: NOT READY until P1-A and P1-B are resolved. After those fixes, the paper is ready for panel submission. P2 items are recommended before external submission.

**Estimated revision effort**: P1-A requires changing 3 locations (abstract, §1 main result paragraph, possibly abstract runtime claim); P1-B requires one case name verification and one text replacement. Total: 30–60 minutes.

---

## NEXT STEPS

1. **Fix P1-A**: Update abstract and §1 main result to read "$O(n_{\mathrm{orient}} \times m \log m)$" matching Proposition 3.1, or add a note that the $O(n_{\mathrm{orient}} \times m)$ claim assumes radix sort.
2. **Fix P1-B**: Verify the Florida redistricting case that explicitly used Reock scores. Check *Soto v. DeSantis*, 2022, or the Florida Fair Districts 2014 trial record. Replace the case name in §1.1 and abstract.
3. **Fix P2-A**: Add hybrid mode implementation-status footnote in §3.5.
4. **Fix P2-B**: Add ensemble caveat sentence to §4.4 partisan table.
5. **Submit to panel** after P1 items resolved.
