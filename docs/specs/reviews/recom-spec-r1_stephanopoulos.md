---
reviewer: Nicholas Stephanopoulos
spec: redist-ensemble ReCom crate
round: 1
score: 3
date: 2026-05-06
---

## Summary
The spec proposes a capable technical tool but does not adequately address the legal argument implications of its two headline results: the "compactness extremum" (0.1th percentile cut-fraction) and the "50th-percentile NC" result. These findings point in opposite legal directions, and the spec must resolve the tension before the ensemble is used as a litigation artifact.

## Strengths
- The framing of ensemble analysis as an auditing tool ("does the enacted plan fall in the middle of the distribution?") is legally sound and consistent with how expert witnesses have deployed GerryChain in *League of Women Voters v. PA* and subsequent cases. Presenting METIS-generated plans against a random baseline is a well-accepted methodology.
- The JSON output includes `plan_percentile` alongside the pooled distribution statistics. This is exactly the format that translates to a litigation exhibit: "the enacted plan falls at the Xth percentile of compact, population-balanced maps." Expert reports built on this output will have a clear, auditable data trail.
- The SHA-chain integration (via `redist label-verify`) creates a tamper-evident record of the plan's provenance that can survive discovery. Courts in *Rucho* and its progeny have not required this level of cryptographic integrity, but having it is a significant defensive asset.

## P1 — Required changes

- **The "compactness extremum" finding (0.1th percentile) creates a new legal vulnerability that the spec does not address.** If the METIS plans are more compact than 99.9% of random plans, opposing counsel will argue that the algorithm over-optimises compactness in a way that suppresses minority communities or creates unnatural concentrations of like-minded voters. The Supreme Court in *Shaw v. Reno* identified "bizarre" shapes as a constitutional problem; "hyper-regular" shapes from over-optimisation could be a mirror-image problem. The spec must add a section addressing: (a) whether the compactness extremum is a bug (METIS is minimising edge cuts, which is a proxy for compactness but not identical to it) or a feature; (b) how to present this finding in an expert report without inviting the over-optimisation challenge; and (c) whether the algorithm should be modified to target the 40th–60th percentile rather than the minimum.

- **The spec conflates two legally distinct uses of the ensemble that courts have treated differently.** The first use (auditing the *enacted* plan) has the strongest legal support: the question "is the legislature's map an outlier among compact balanced plans?" was central to the *LWV v. PA* and *Harper v. Hall* expert analyses. The second use (characterising the position of the METIS-generated plan in the feasible space) is a newer and less-tested argument. Presenting these two uses without distinction in the same spec risks courts treating the ensemble methodology as tainted if the METIS plan audit fails (e.g., if the METIS plan is itself an outlier on partisan statistics). The spec must clearly separate these use cases and specify which result types are appropriate for which legal context.

- **The 50th-percentile NC result should not be presented as evidence of partisan neutrality without a separate partisan-outcome ensemble analysis.** Courts in *Rucho* and *Gill v. Whitford* explicitly rejected compactness-based arguments as proxies for partisan fairness. A 50th-percentile cut-fraction result says nothing about whether the plan is partisan — it says only that the plan is "typically compact." The spec must include a warning that the cut-fraction percentile is not a partisan-neutrality result, and should propose a timeline for adding partisan-outcome statistics (Democratic seat share percentile, efficiency gap percentile) to the ensemble output before the tool is used in litigation support.

## P2 — Suggested improvements

- Add a one-paragraph "legal use guidance" section that specifies what claims the ensemble output does and does not support. The current spec presents the tool as research infrastructure, but the connection to the "DIA statutory argument" in the research significance section makes clear that litigation use is contemplated. Courts will scrutinize the expert's understanding of the tool's limitations.

- Consider whether the 50th-percentile NC result, if reproduced with the Rust implementation, is worth presenting as a standalone research finding in a technical report before the G-series papers are complete. A pre-publication technical report would establish the methodology in the public record and provide a dated baseline for any subsequent legal challenge to the ensemble approach.

## Score: 3/4
The tool is legally usable but the spec does not resolve the tension between the compactness extremum and the 50th-percentile results, and does not adequately separate the audit use case from the methodology-validation use case. One focused revision can close these gaps.
