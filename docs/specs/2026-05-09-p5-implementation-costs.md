---
title: "Implementation Costs, Administrative Law, and Audit Requirements"
series: P.5
status: Planned
date: 2026-05-09
track: P-reform-pathways
---

## Claims
1. The bisect pipeline requires: (a) one-time data fetch (~40 GB, $0 marginal cost from Census Bureau), (b) computation (~18 minutes on commodity hardware), (c) audit chain storage (~200 MB). Total implementation cost: under $50,000 for a state redistricting office.
2. The Iowa model (current gold standard for nonpartisan redistricting) costs approximately $3M per cycle in legislative staff time; bisect reduces this to <$200K in staff time for plan review and VRA verification.
3. Administrative law compliance: the bisect pipeline's run manifests and SHA-256 audit chain satisfy the APA's record-keeping requirements for agency rulemaking. The audit chain demonstrates that the algorithm was run as specified.
4. A model procurement specification is developed for state redistricting offices to contract for bisect implementation, covering hardware requirements, data governance, audit retention, and public disclosure.
