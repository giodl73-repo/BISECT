---
title: "The Audit Chain as a Gaming Defense: Provenance, Reproducibility, Tamper Detection"
series: R.4
status: Planned
date: 2026-05-09
track: R-adversarial-robustness
---

## Claims
1. The bisect audit chain (manifests + SHA-256 + bisect label-verify) provides cryptographic evidence of three properties: (a) inputs were obtained from the specified public source, (b) the specified algorithm version was run, (c) the output corresponds to the specified inputs and algorithm.
2. Formal security analysis: breaking the audit chain requires either a SHA-256 collision (computationally infeasible in 2026) or corruption of the certified inputs. Neither is achievable by a state redistricting office acting alone.
3. The audit chain satisfies the Daubert standard for expert witness reproducibility: any party with the manifest can independently verify the plan by running `bisect label-verify`. This has been done successfully in simulated exercises for NC, TX, and WI.
4. Model legal language for the DIA's audit provision: a statute requiring the SHA-256 manifest to be filed with the state secretary of state simultaneously with the plan, and preserved for 10 years post-redistricting.
