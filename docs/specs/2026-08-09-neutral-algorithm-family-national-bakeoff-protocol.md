# Neutral Algorithm-Family National Bakeoff Protocol

**Protocol ID:** `neutral-algorithm-family-national-bakeoff-v1`

**Frozen:** 2026-08-09, before any governed multi-State pilot run

**Prerequisite:** the Wisconsin proof slice passed after the separately retained
pre-remediation failure witness

## Question

Holding each State's 2020 tract universe, adjacency graph, congressional seat
count, population source, edge-weight signal, outer search mode, and requested
initial seed fixed, do four implemented BISECT structure families produce valid,
reproducible plans across the national multi-district cohort, and how different
are their graph-native assignments and boundary costs within each State?

This is a software-and-mechanics comparison. It is not a test of partisan
fairness, Voting Rights Act compliance, legal adequacy, or superiority for map
adoption.

## Frozen controls

| Field | Frozen value |
|---|---|
| Census year / chamber | 2020 / congressional |
| Atomic units | Census tracts in each repository 2020 adjacency package |
| Population source | total population |
| Balance tolerance | native configured value, recorded from every manifest |
| Partition preset | `edge-weighted` |
| Weight override | `geographic` |
| Outer search | `single` |
| Requested initial seed | `0` |
| Structures | `standard-bisect`, `ratio-optimal`, `ratio-optimal-area`, `prime-factor` |
| Runner | repository `target/release/bisect.exe`, SHA-256 bound in each package |

The structure names identify implemented families, not isolated one-factor
treatments. AreaSection activates its area constraint; ratio and prime-factor
families may perform documented internal seed search. Native automatic balance
retries are retained, with both requested and accepted seeds reported.

## National cohort

The target cohort is all 44 States apportioned at least two U.S. House seats in
the embedded 2020 manifest. Six one-seat States are design exclusions because
there is no nontrivial district partition to compare: AK, DE, ND, SD, VT, and WY.
Exclusions are reported in the aggregate package and are not counted as passes.

The complete cohort, grouped by frozen seat count, is:

| Seats | States |
|---:|---|
| 2 | HI, ID, ME, MT, NH, RI, WV |
| 3 | NE, NM |
| 4 | AR, IA, KS, MS, NV, UT |
| 5 | CT, OK |
| 6 | KY, LA, OR |
| 7 | AL, SC |
| 8 | CO, MD, MN, MO, WI |
| 9 | AZ, IN, MA, TN |
| 10 | WA |
| 11 | VA |
| 12 | NJ |
| 13 | MI |
| 14 | GA, NC |
| 15 | OH |
| 17 | IL, PA |
| 26 | NY |
| 28 | FL |
| 38 | TX |
| 52 | CA |

## Staged execution schedule

The governed pilot is executed in this order:

1. RI (2 seats)
2. NE (3)
3. CT (5)
4. KY (6)
5. SC (7)
6. WI (8; the already observed proof-slice anchor)
7. AZ (9)
8. CA (52; the maximum-scale case)

The pilot was selected to exercise increasing scale, prime and composite seat
counts, multiple regions, the known anchor, and the maximum-size congressional
problem. It is an engineering gate, not a probability sample.

If the pilot passes, full execution uses the pilot order above followed by every
remaining cohort State ordered by increasing seat count and then State code.
No failed State or structure may be dropped, replaced, or silently rerun under
changed controls. Code remediation after a failure requires retaining the failed
package and issuing a new protocol or expressly versioned remediation phase.

## Outcomes and aggregation

For every State-structure cell, retain the command, return code, requested and
accepted seed, audit result, population flag and its consistency with the audit,
tract and district counts, weighted edge cut, and canonical-assignment SHA-256.
Within each State, report all six maximum-overlap assignment agreements and each
alternative structure's edge-cut ratio to `standard-bisect`.

Raw edge cuts are not pooled across States. Pilot aggregation is limited to
cell and State validity counts, exact-regeneration status, and descriptive
within-State ranges. Runtime is diagnostic because machine load and cache state
are uncontrolled. A full-cohort package may report State-weighted distributions
of within-State ratios but may not convert them into a universal winner or legal
recommendation.

Maximum-overlap matching must use a polynomial-time exact assignment algorithm;
the subset dynamic program used in the Wisconsin proof slice is not admissible
for large delegations.

## Validation and decision rules

Every package binds this protocol, runner/analyzer/verifier source, exact
commands, binary hash, native artifacts, canonical assignments, and deterministic
aggregate outputs. Verification re-hashes the package, checks common-input
invariants within every State, reruns the scheduled matrix in a temporary
directory, and requires byte-identical deterministic outputs. Edge cuts are
normalized to six decimals to ignore sub-billionth parallel reduction noise.

The pilot passes only if all 32 cells execute, all native audits and population
checks pass, every State's four runs share the frozen common inputs, all eight
States pass, and exact normalized regeneration succeeds. Only a passing pilot
authorizes the already frozen 44-State full phase. The national gate passes only
if all 176 cells and all 44 States pass and regenerate exactly.

## Claim boundary

The pilot can establish bounded multi-State implementation validity and expose
scale- or factorization-related defects. It cannot estimate national prevalence,
rank algorithms, or establish geometric, electoral, demographic, VRA, causal,
or legal conclusions. Only the complete 44-State phase supports claims about
coverage of the frozen national multi-district cohort, and even that phase does
not establish a generally best algorithm.
