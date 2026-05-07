# Review 5 — Christina Liang
**Paper**: F.4: Satisfying 50 Different Rule Sets — State Constitutional Redistricting Criteria and Algorithmic Adaptation
**Round**: R1
**Score**: 3/4

## Summary

F.4 is primarily a legal/policy paper with computational content in Section 3. From a reproducibility perspective, the key question is whether the YAML configuration parameters described actually exist in the redist CLI and whether the 50-state classification in Table 1 is verifiable.

## Strengths

The parameter descriptions in Section 3 are specific enough to be implemented: compactness (0.0 to 1.0 scale, edge weights by boundary length), county_weight (1.0 to 3.0 range, multiplicative on cross-county edges), coi=true/false, partisan_neutral=true/false, vra=true/false. These map to identifiable CLI flags or YAML keys. A reviewer can check whether these parameters exist in the redist binary documentation.

The classification table (Table 1), while long, is structured to be independently verifiable from primary sources: state constitutions, statutes, and court decisions are all publicly accessible. Each state entry lists the key criteria and the primary legal authority (implicit in the Type classification).

## Concerns

**C1 — YAML parameter existence not verified.** Section 3 describes parameters including compactness, county_weight, coi, partisan_neutral, and vra_mode. The CLAUDE.md for this project describes the redist CLI's flags as: --chamber, --resolution, --year, --version, --states, --workers, --dpi, --run-type, --partition-mode, --election-year. The YAML configuration parameters described in F.4 (compactness, county_weight, coi, etc.) do not appear in the CLI documentation. If these are YAML-level configuration parameters rather than CLI flags, the paper should provide the YAML configuration file format and show a complete example for at least one state.

**C2 — Arizona competitive margins not sourced.** Section 5 (Key State Examples) states that METIS maps produce "a mean margin of 7.2 percentage points in competitive congressional districts, compared to a mean margin of 9.4 in the 2022 enacted Arizona map." These figures require: (1) Arizona algorithmic maps to have been generated at the congressional level using this project's pipeline; (2) precinct-level 2022 election data to have been interpolated to the relevant geographic units; (3) competitive districts to have been defined and classified. None of these steps are documented in this paper or confirmed to have been performed. This claim appears to be speculative or forward-looking.

**C3 — YAML model configurations not provided.** The abstract promises "one model configuration per state type." Section 4 references model configurations for each state type but no YAML is shown in the paper. This is the paper's most practically useful contribution — a practitioner implementing algorithmic redistricting for Iowa needs the actual YAML configuration — and its absence is a significant gap.

**C4 — 50-state classification update requirements.** The 50-state classification in Table 1 reflects the redistricting landscape at the time of writing. Redistricting law changes rapidly: North Carolina's Harper II (2023) reversed the state supreme court's partisan gerrymandering holding; Ohio's commission maps were struck down multiple times; Michigan's MICRC has ongoing litigation. A paper making specific legal characterisations for all 50 states has a high maintenance burden and should specify the date as of which the classifications are accurate, with a note that subsequent legal developments may require updates.

## Recommendation

Accept with minor revisions. C1 (YAML parameter existence) and C3 (missing YAML configurations) are the most important reproducibility concerns. The paper should demonstrate that at least one state's YAML configuration actually runs in the redist binary.
