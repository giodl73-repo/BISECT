---
skill: validate-dimensional
topic: U.21-certified-recursive-bisection
date: 2026-09-09
equations_checked: 7
p1_errors: 0
dimensionless_paper: false
---

# U.21 dimensional audit

## Result

PASS. The paper is mathematical but not a physical-units model. Counts are
dimensionless; population quantities carry persons; boundary weights retain
the unit declared by the input profile. Lexicographic tiers are ordered rather
than added, so population and boundary quantities are never combined into an
invalid scalar.

| ID | Expression | Dimensions | Result |
|---|---|---|---|
| D-01 | `k_L = floor(k/2)` | district count on both sides | PASS |
| D-02 | `k_R = k-k_L` | district count on every term | PASS |
| D-03 | `Delta_L = abs(k P_L-k_L P)` | district counts are dimensionless; both terms are persons | PASS |
| D-04 | right-child deviation equals left | complements preserve the same persons dimension | PASS |
| D-05 | total deviation is twice maximum | dimensionless factor two times persons | PASS |
| D-06 | weighted boundary cut | sum of homogeneous declared edge weights | PASS, conditional on a single weight unit per profile |
| D-07 | canonical binary assignment | dimensionless bit-vector ordering | PASS |

No exponentials, logarithms, trigonometric arguments, or differential
equations occur. The only hidden-unit obligation is already handled by freezing
the edge-weight construction in the rule profile.
