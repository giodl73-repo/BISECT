---
skill: validate-consistency
topic: U.21-certified-recursive-bisection
date: 2026-09-09
p1_count: 0
p2_count: 0
quantities_checked: 19
---

# U.21 consistency audit

## Result

PASS after one P1 correction. The draft had said ceremony-enabled bakeoffs
were wholly future evidence after adding the bounded path-8 bakeoff. The
conclusion now distinguishes that fixture from externally witnessed State-scale
evidence. A second overstatement about national bakeoff regeneration was also
corrected after the current verifiers exposed missing source-run assignments.
Those assignments were subsequently recovered from the data vault and all 150
governed hashes matched in a clean-worktree replay.

## Quantity and relation registry

| ID | Quantity or relation | Cross-check | Result |
|---|---|---|---|
| C-01 | California schedule | `52 -> 26/26 -> 13/13 -> 6/7` preserves seat counts | PASS |
| C-02 | Binary-tree cut count | Four districts require `k-1 = 3` internal cuts | PASS |
| C-03 | Ceremony release count | Path-8 transcript contains root plus depth-1, hence two rounds | PASS |
| C-04 | Comparison design | Eight graphs times five seeds equals 40 rows | PASS |
| C-05 | Primary-objective agreements | Published suite aggregate reports 30/40 | PASS |
| C-06 | Objective disagreements | 40 minus 30 equals the ten described disagreements | PASS |
| C-07 | Canonical agreements | Published suite aggregate reports 20/40 | PASS |
| C-08 | National node count | `435 districts - 50 State roots = 385` internal nodes per cycle | PASS |
| C-09 | Three-cycle node count | `3 * 385 = 1,155` | PASS |
| C-10 | RI edge accounting | 66,097 land edges plus 64 bridges equals 66,161 connected edges | PASS |
| C-11 | RI population accounting | 548,689 plus 548,690 equals 1,097,379 | PASS |
| C-12 | RI scaled deviation | `abs(2*548,689 - 1,097,379) = 1` | PASS |
| C-13 | Retained block universe | 8,126,956 minus 237,762 equals 7,889,194 | PASS |
| C-14 | Assignment accounting | 4,194,107 matches plus 3,695,087 differences equals 7,889,194 | PASS |
| C-15 | Agreement rate | 4,194,107 / 7,889,194 = 53.162680 percent after rounding | PASS |
| C-16 | National exact-proof coverage | Text consistently reports 0/1,155 boundary/canonical proofs | PASS |
| C-17 | Geometry means | Four district-weighted pairs match the published analysis JSON | PASS |
| C-18 | Replayed assignments | Three cycles times 50 States equals the 150/150 matched governed hashes in the replication record | PASS |
| C-19 | Geographic audit | The replication record and independent verifier agree on 231,765 checked State/level rows | PASS |

## Inconsistency register

| ID | Severity | Finding | Resolution |
|---|---|---|---|
| I-01 | P1, resolved | Conclusion called all ceremony bakeoffs future despite the new bounded fixture. | Limited the future-evidence statement to externally witnessed State-scale runs. |
| I-02 | P2, resolved | U.21 and A.0 initially lacked the source assignments needed to substantiate the national Tier 1--2 regeneration claim. | A clean-worktree replay matched all 150 governed assignment hashes, and the restored 2020 inputs passed the national Tier 1 and Tier 2 regeneration verifiers. The papers retain the external-data dependency. |

No contradictory numerical value remains in U.21.
