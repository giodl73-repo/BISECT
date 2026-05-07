# R1 Review — Jonathan Rodden
**Paper**: H.3 Resolution-Aware Redistricting: Geographic Granularity as a First-Class Parameter
**Score**: 2/4
**Verdict**: Major Revision

## Summary
The paper presents a resolution system for redistricting pipelines, making geographic granularity (block group, tract, county) an explicit, auditable parameter. The technical contributions are solid: a GEOID prefix-matching function with correctness proof, county adjacency construction, and plan manifest extensions. However, the paper largely treats resolution as a computational parameter while underweighting its political implications. For a redistricting audience, the partisan sensitivity of resolution choice — and how resolution interacts with incumbent protection, community-of-interest claims, and court review — is at least as important as the algorithmic correctness results.

## Strengths
- The GEOID prefix hierarchy insight is well-presented: the connection between Census Bureau FIPS encoding and free fine-to-coarse derivation is elegant and non-obvious.
- The manifest audit trail (Section 4) is a genuine contribution: recording `fine_to_coarse_formula` enables courts and special masters to verify plan reproducibility independently.
- The coarse-tolerance rationale (3× fine tolerance for Option B) is carefully reasoned and grounded in tract-per-county counts.
- The VRA implications paragraph in Section 5b correctly notes that BG vs. tract resolution can shift minority VAP percentages in near-threshold districts.

## Concerns
- **Political implications of resolution choice**: The paper does not address whether resolution choice is itself a form of partisan manipulation. An actor choosing county-level coarsening for a large-$k$ state with politically polarized county structures (e.g., a state where counties are nearly all rural-Republican or urban-Democrat) is making a choice that may systematically affect partisan outcomes in ways not visible in compactness or population-balance metrics. The paper should acknowledge this and note that resolution choice should be disclosed and justified in litigation, not treated as a neutral computational decision.
- **Partisan sensitivity of granularity**: The empirical section (Section 5) measures autocorrelation reduction but not partisan-outcome sensitivity. A reviewer expecting redistricting research to connect to political outcomes will note the absence of any partisan metric comparison between single-scale and multi-scale plans. Even a brief comparison of mean Democratic seat share across the TX ensembles would strengthen the paper's claim that resolution does not introduce systematic partisan bias.
- **County-level coarsening in county-preservation states**: Section 5b notes this issue but frames it as a compliance check ("verify separately"). For a practitioner in a state like California or Florida, this is not a minor downstream check — the choice to use county-level coarsening in a county-preservation state is legally problematic from the outset. The paper should recommend that users of Option B verify their state's county-preservation requirements before selecting this option, not only after plan generation.
- **Ensemble comparability across resolutions**: The statement in Section 5b that "ensembles at different resolutions answer different questions" is correct but underdeveloped. An expert witness who uses Option B for one analysis and tract-level for another will face cross-examination on whether the resolution choice was pre-registered. The paper should explicitly recommend that resolution be fixed before analysis begins and disclosed in any testimony.

## Required Changes (P1/P2)
- **P1**: Add a paragraph in Section 5 (or 5b) addressing whether resolution choice can introduce partisan bias. At minimum, note that county structures in polarized states may make county-level coarsening politically non-neutral, and recommend disclosure of resolution choice in legal proceedings.
- **P1**: Add at least a brief partisan-outcome comparison for the TX Option B vs. single-scale experiment (mean D-seat share, or partisan efficiency gap comparison). Without this, the autocorrelation claim is decoupled from whether the two methods sample the same partisan distribution.
- **P2**: Strengthen the county-preservation warning in Section 5b: move the recommendation to verify state requirements to a position before Option B is invoked, not as a downstream check.
- **P2**: Add a pre-registration recommendation: resolution should be selected and documented before ensemble analysis begins, and any change in resolution between analyses should be disclosed.
