# Revision Plan — B.0 Algorithm Design Overview
Round 1 avg: 3.0/4
Round 2 avg: 3.2/4 (Karypis 3, Rodden 4, Duchin 3, Stephanopoulos 3, Liang 3)
Round 3 avg: 3.4/4 (Karypis 3, Rodden 4, Duchin 3, Stephanopoulos 3, Liang 4)
Round 4 avg: **4.0/4 — ACCEPTED** (Karypis 4, Rodden 4, Duchin 4, Stephanopoulos 4, Liang 4)

## Round 4 Summary (2026-05-05)

All five reviewers at 4. Paper ACCEPTED for the B-series internal review track.

### R4 edits that unlocked acceptance

- **Karypis** (3→4): EC_norm definition paragraph added to C2: GeoSection subsection.
  Formally defines $\mathrm{EC}_\text{norm}(i) = \mathrm{EC}(i)/\sqrt{\min(i,k-i)}$
  with two-case structure (recursive bisection vs. $n$-way leaf), anti-caterpillar
  design rationale, and $n$-way simplification $\mathrm{EC}/\sqrt{k/2}$.

- **Duchin** (3→4): Strong-inference interpretation paragraph added after the
  North Carolina block in Section 6. Acknowledges conflation in the GeoSection/
  VRASection comparison, presents the B.14 $w_\text{vra}=0$ ablation result
  (Alabama $3:4$ / 1 MM at $w_\text{vra}=0$ vs. $2:5$ / 2 MM at $w_\text{vra}=0.40$),
  and adds the partisan-neutrality falsification framing ($n=34$ competitive subset,
  12.8 pp range $\approx 4\times$ seed variance).

- **Stephanopoulos** (3→4): Algorithmic-neutrality paragraph added to Pattern 1
  in Section 5. Distinguishes process-based standards (PA "free and equal", NC Harper)
  from outcome-based standards (NY Harkenrider efficiency-gap), explains the
  bakeoff's role as counterfactual evidence for both inquiries, and states the
  core judicial-review finding: algorithm choice is the locus of potential bias,
  not the individual map.

### Carry-forward P2 items (journal-submission conditions, not blocking)

- Enumerate 9 mode-invariant states (Rodden, Duchin, Stephanopoulos)
- Software version pin — commit hash / release tag in Data Availability (Liang, Karypis)
- Estimation model source for each † cell (Liang, Karypis)
- -7pp proportionality gap: identify the 8 competitive states (Rodden)
- Geographic sorting scope: qualify "dominant driver" for counterexample states (Rodden)
- Proposition 1 proof: "verifies file integrity" not "verifies data provenance" (Duchin)
- AreaSection EC_norm interaction (Lorenz pre-filter family vs. full ratio space) (Karypis)
- Stephenson v. Bartlett hard-constraint note for alpha parameter (Stephanopoulos)
- Harper v. Hall current doctrine status note (Stephanopoulos)

## Round 3 Summary (2026-05-05)
- Karypis: 3/4 — Rodden P1.1 resolved; EC_norm/AreaSection spec gap (P2) carries forward.
- Rodden: 4/4 — denominator fully resolved; P1.2/P1.3 (sorting scope, -7pp source) remain journal conditions.
- Duchin: 3/4 — denominator good; strong-inference test methodology (P1.2) and Proposition provenance note (P1.3) unresolved.
- Stephanopoulos: 3/4 — Data Availability improved; partisan-neutrality differentiation (P1.2) and county hard-constraint note (P1.3) unresolved.
- Liang: 4/4 — Data Availability resolves primary concern; upgrades to Accept. Estimation model source (P1.2) remains journal condition.

## Key R3 Fixes Applied
- [x] Rodden-null denominator explained: 7 k=1 states + 9 mode-invariant states excluded, leaving 34 contested states (76% = 25/34)
- [x] Data Availability statement expanded: repository URL, GitHub Release assets, SHA-256-manifest linkage

## Round 2 Summary (2026-05-05)
- Karypis: 3/4 — tpwgts fixed, bakeoff provenance improved; EC_norm/AreaSection interaction deferred.
- Rodden: 4/4 — accepts current state; three P1 items (Rodden-null denominator, sorting scope, -7pp source) deferred as journal-submission conditions.
- Duchin: 3/4 — GerryChain mischaracterisation resolved; strong-inference procedure and Prop 1 proof partially resolved.
- Stephanopoulos: 3/4 — Callais slip-opinion footnote resolved; partisan neutrality differentiation and county-preservation hard-constraint gap carry forward.
- Liang: 3/4 — bakeoff provenance improved; dataset release, estimation model source, software URL carry forward.

## Key R2 Fixes Applied
- [x] tpwgts memory layout corrected to row-major [p_L, 1-p_L, 0.5, 0.5]
- [x] GerryChain description corrected (generates ensembles of thousands of plans; complementary not competing)
- [x] Callais slip-opinion footnote added (majority opinion, page numbers provisional)
- [x] Bakeoff value provenance paragraph added (confirmed/estimated/pending with uncertainty)
- [x] \pend → \pending LaTeX macro fix throughout bakeoff tables

## P1 — Required (from reviewers)

- [ ] [Karypis R1.1] Fix tpwgts layout specification for ncon=2 AreaSection — correct flat C-array memory layout for METIS `[[p_L, 1-p_L], [0.5, 0.5]]` row-major, not `[p_L, 0.5, 1-p_L, 0.5]` as currently stated
- [ ] [Karypis R1.2] Separate confirmed bakeoff results from estimated/pending results — create distinct tables (or visually clear separation) for confirmed empirical values vs. model-derived `†` estimates vs. pending `‡` B.11/B.12 runs; add estimation methodology for all `†` cells
- [ ] [Karypis R1.3] Specify how EC_norm is computed in the AreaSection dual-constraint context — clarify the interaction between the isoperimetric normalisation and the dual-constraint tpwgts at asymmetric splits where population and area constraints differ
- [x] [Rodden R2.1] Explain and justify the 34-state denominator in the Rodden-null test (76% = 25/34) — RESOLVED R3: 7 k=1 states named, 9 mode-invariant states explained; 34 contested states defined
- [ ] [Rodden R2.2] Either include counterexample states (MA, MD, CT) or narrow the geographic sorting claim — "geographic sorting is the dominant driver" must be qualified to the studied configuration or evidenced across all 44 states from B.8–B.9
- [ ] [Rodden R2.3] Provide source table or citation for the -7pp mean proportionality gap claim in Limitations L1 — the eight competitive states must be identified and their per-configuration gap values sourced
- [ ] [Duchin R3.1] Correct or substantially revise the GerryChain comparison — either (a) show GeoSection plan EC_norm percentile in a GerryChain ensemble, or (b) rewrite the comparison as complementary methodologies without the misleading "plan generation" limitation
- [ ] [Duchin R3.2] Fix the strong-inference test procedure to avoid conflating algorithm comparison with the legal question — operationalise the Callais strong-inference test by fixing the algorithm and varying w_vra, or add a clear statement that the procedure is a comparative analysis, not a formal legal test
- [ ] [Duchin R3.3] Fix Proposition 1 proof: (a) address the "only if" direction or restate as "if," (b) correct the SHA-256 provenance claim to acknowledge chain-of-custody gap between TIGER/Line source and adjacency graph file
- [ ] [Stephanopoulos R4.1] Clarify Callais opinion structure — verify and state whether p. 36 disentanglement language is a majority opinion, plurality, or concurrence; update all citations from slip opinion page numbers to permanent citation form when available
- [ ] [Stephanopoulos R4.2] Differentiate state partisan neutrality standards (PA "free and equal," NC Harper, NY Harkenrider) and specify which toolbox configurations satisfy which standard — process-based vs. outcome-based requirements need separate treatment
- [ ] [Stephanopoulos R4.3] Acknowledge hard vs. soft county preservation requirements — Stephenson v. Bartlett-style hard grouping constraints cannot be satisfied by the alpha soft-weight parameter; specify which legal contexts the soft-weight approach handles
- [ ] [Liang R5.1] Release public dataset — provide TIGER/Line adjacency graph SHA-256 hashes for all four bakeoff states, complete plan manifests from confirmed bakeoff runs, and the census_release_id strings used to generate confirmed results
- [ ] [Liang R5.2] Add confidence intervals or standard errors to all `†` estimated bakeoff cells — point estimates without uncertainty bounds cannot be interpreted
- [x] [Liang R5.3] Add repository URL, commit hash, and TIGER/Line version to the software availability statement — RESOLVED R3: repository URL added, GitHub Release assets and SHA-256-manifest linkage stated; commit hash P2 for journal

## P2 — Suggested

- [ ] [Karypis R1.P2a] Add runtime complexity table by state size and mode — wall-clock times for GeoSection, AreaSection, VRASection at key state sizes (small: VT/DE, medium: WI/AL, large: TX/CA)
- [ ] [Karypis R1.P2b] Address the "only if" direction of Proposition 1 or restate as a sufficient-conditions proposition
- [ ] [Rodden R2.P2a] Define the Lorenz population-density curve L(0.5) explicitly — distinguish from income Lorenz curve; give example for a state with a compact high-density city core
- [ ] [Rodden R2.P2b] Add bisection tree diagram for NC or WI contrasting GeoSection vs. AreaSection first-level split and its downstream district cluster implications
- [ ] [Duchin R3.P2a] Engage with outlier-map methodology (Stephanopoulos-McGhee efficiency gap, Chen-Rodden) — report efficiency gap alongside proportionality gap for the bakeoff results
- [ ] [Duchin R3.P2b] Quantify MM district count probability across seeds for each bakeoff state — VRASection guarantee is currently stated as "empirical, not structural" without quantification
- [ ] [Stephanopoulos R4.P2a] Discuss toolbox fit with redistricting commission deliberative mandates — specify where deterministic algorithm output intersects with statutory deliberation requirements
- [ ] [Stephanopoulos R4.P2b] Add efficiency gap alongside proportionality gap in bakeoff tables
- [ ] [Liang R5.P2a] Report seed-count standard deviation alongside point estimates in bakeoff tables
- [ ] [Liang R5.P2b] Specify TIGER/Line year and release version used for each bakeoff state's adjacency graph
