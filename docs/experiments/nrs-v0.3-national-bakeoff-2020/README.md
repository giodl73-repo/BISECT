# NRS v0.3 National 2020 Tier 1 Bakeoff

**Status:** pass

| Measure | NRS v0.3 | Official CD118 comparator |
|---|---:|---:|
| States passed | 50 | 50 |
| Districts | 435 | 435 |
| Source Census blocks | 8,126,956 | 8,126,956 |
| Excluded water-only blocks | 237,762 | 237,762 |
| Analyzed land-containing blocks | 7,889,194 | 7,889,194 |
| County split units | 1,808 | 404 |
| Tract split units | 19,789 | 4,720 |

After State-level maximum-overlap district-label matching,
4,194,107 blocks match and 3,695,087
differ (53.162680% block-weighted agreement).

State-weighted and district-weighted estimands are reported separately in
`analysis.json`; per-State results and every failure are in
`state-summary.csv`.

## Claim Boundary

Descriptive same-vintage atomic-block assignment overlap and county/tract split counts only; no compactness, population, partisan, demographic, VRA, legal-validity, optimality, superiority, or adoption claim.
