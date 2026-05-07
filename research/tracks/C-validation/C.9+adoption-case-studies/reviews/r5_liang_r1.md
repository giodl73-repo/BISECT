# Review 5 — Reviewer: Christina Liang (Computational Social Science / Implementation Research)
**Paper:** C.9 — From Algorithm to Map: Implementation Case Studies for Three Adoption Pathways
**Round:** 1
**Score:** 3/4

## Summary

A practically oriented paper that addresses implementation rather than algorithm design. The case study framework is well-structured and the workflow table is the paper's most concrete contribution. From a social science perspective, the paper would benefit from more systematic comparative analysis across the three pathways and clearer specification of the success criteria for adoption.

## Strengths

The three-pathway framework cleanly structures an otherwise complex institutional landscape. The identification of three dimensions of variation (who runs the algorithm, how adjustments are made, what counts as a successful challenge) provides an analytical framework that could be applied to other states beyond the three case studies.

The workflow table (Table 1) is a genuine practical contribution. The one-to-one mapping from redistricting stage to responsible actor to redist command provides a template that any adopting jurisdiction could adapt. The differentiation between the public comment period durations (60-day for commissions, 14-day for court-ordered) reflects real institutional practice.

The challenges section (Section 6) is honest and well-balanced. The identification of data preparation, public transparency, and the challenge process as the three main obstacles is accurate and pragmatically useful.

## Weaknesses and Concerns

The paper lacks a systematic comparison of the three pathways along the three identified dimensions. Each case study is developed independently, but the reader must extract the comparisons across pathways themselves. The paper should add a summary table comparing the three pathways along the three dimensions: (a) who runs the algorithm (commission staff, state agency, special master), (b) adjustment mechanism (community-of-interest testimony, legislative review, court approval), and (c) challenge standard (criteria violation, legislative authority, constitutional defect). This would make the comparative argument explicit rather than leaving it implicit in the workflow table.

The selection of the three case studies (Arizona, California, North Carolina) is reasonable but the generalizability claim is overstated. These three states are all large, politically sophisticated, and have established redistricting frameworks that make them more amenable to algorithmic adoption than most states. The paper's claim that "the three pathways analyzed here provide the institutional templates for that process" assumes that smaller states with less redistricting infrastructure can follow the same templates. A state like Wyoming or Montana, with small delegations and limited GIS capacity, would face different adoption challenges that the templates do not address.

The paper assumes throughout that the algorithm is run by technically competent staff (commission staff, state GIS office, or special master's technical support team). But in practice, many state redistricting bodies do not have in-house technical capacity for running computational algorithms. The paper should address the contracting and external expertise question: is it appropriate for a redistricting commission to contract with the algorithm's developer (the redist system's authors) to run the algorithm? This creates a potential conflict of interest — the algorithm's developer has an interest in the algorithm performing well, which might not be the same as the neutral interest in the correct redistricting outcome.

## Minor Issues

- The paper's data preparation time estimates ("30 minutes to 4 hours for a single state") should be benchmarked against specific hardware. What is the computational environment assumed? A state GIS office might have very different hardware than a research institution.
- The conclusion mentions that "adoption at scale would likely follow the pattern of early adopters in states with independent commissions." The paper should name specific states that are plausible early adopters (Michigan, Ohio, New Mexico all currently have commission-adjacent processes) and identify the institutional barriers each would need to overcome.
- Forward citations (dellaLibera2026vra, dellaLibera2026legal, dellaLibera2026threshold, redistCliQuickstart) are numerous and represent a significant portion of the technical and legal claims in the paper. The reader cannot evaluate these claims without access to the cited documents. The most critical claims (VRASection produces more majority-minority districts; algorithmic maps' compactness exceeds CRC map) should be either included in this paper or marked as "forthcoming."

## Recommendation

Accept with minor revisions. Add a comparative summary table across the three pathways, address the contracting/conflict-of-interest question, and mark unresolvable forward citations explicitly.
