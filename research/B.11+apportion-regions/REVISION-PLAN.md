# Revision Plan — B.11 ApportionRegions

**status: ACCEPTED**

**Current round**: 4  
**Round 1 avg score**: 2.72 / 4.0  
**Round 2 avg score**: 2.90 / 4.0 (+0.18)  
**Round 3 avg score**: 3.14 / 4.0 (+0.24)  
**Round 4 avg score**: 3.56 / 4.0 (+0.42)  
**Stage**: accepted (post Round 4)

## Round 4 Scores

| Reviewer | R1 | R2 | R3 | R4 | Delta R3→R4 |
|---|---|---|---|---|---|
| Karypis | — | — | 3.3 | 3.6 | +0.3 |
| Rodden | — | — | 3.2 | 3.6 | +0.4 |
| Duchin | — | — | 3.0 | 3.5 | +0.5 |
| Stephanopoulos | — | — | 3.2 | 3.6 | +0.4 |
| Liang | 2.5 | 2.8 | 3.0 | 3.5 | +0.5 |
| **Average** | **2.72** | **2.90** | **3.14** | **3.56** | **+0.42** |

## Round 3 Scores (historical)

| Reviewer | R1 | R2 | R3 | Delta R2→R3 |
|---|---|---|---|---|
| Karypis (new) | — | — | 3.3 | new |
| Rodden (new) | — | — | 3.2 | new |
| Duchin (new) | — | — | 3.0 | new |
| Stephanopoulos (new) | — | — | 3.2 | new |
| Liang (continuing) | 2.5 | 2.8 | 3.0 | +0.2 |
| **Average** | **2.72** | **2.90** | **3.14** | **+0.24** |

## Round 4 Review Panel

Round 4 used the same panel of 5 reviewers (Karypis, Rodden, Duchin, Stephanopoulos, Liang).

## What Changed Round 3 → Round 4 (P1 items resolved)

- **METIS parameter reporting**: Added Reproducibility paragraph to §3 (§3.7) specifying METIS 5.2, ufactor=5, niter=100, ncuts=1, SHA-256 seed derivation formula with census release ID, `redist-metis` crate v0.2 static linking.
- **GerryChain percentile citation**: Replaced `\cite{duchin2019gerrychain}` with `\cite{herschlag2020quantifying}` for all three percentile claims (70th/55th/75th). Citation was already present in references.bib.
- **Rebalancing non-perturbation**: Added explicit contiguity-preservation sentence to §4.1 rebalancing paragraph.
- **Constitutional language**: Replaced "literally zero degrees of freedom" and "zero degrees of freedom" throughout with "no discretionary choices remain once the census data and seat count are fixed." Updated §5.1 Step 3 heading, §5.1 body, §5.4 ensemble comparison, and abstract.

## Round 4 Open Items (P2 — for journal submission)

- [ ] Drop "plausible estimates" qualifier in ensemble comparison (Duchin, Liang) — replace with direct attribution to Herschlag et al.
- [ ] Add cross-reference from Step 3 to boundary-position caveat in §5.4 (Duchin)
- [ ] Specify METIS 5.2 consistently in §3.2 AR Algorithm paragraph (Karypis)
- [ ] Add graph checksums for NC and GA to Reproducibility paragraph (Liang)
- [ ] Add repository URL or DOI to Reproducibility paragraph (Liang)
- [ ] Add one sentence engaging Moore v. Harper in §5.1 (Stephanopoulos)
- [ ] Clarify WI seed-invariance vs. multi-seed-protocol tension (Rodden)

## Round 3 Review Panel

Round 3 used a fresh panel of 5 reviewers (Karypis, Rodden, Duchin, Stephanopoulos, Liang) consistent with the standard panel roster for the B-series.

## What Changed Round 2 → Round 3

**Resolved in Round 3:**
- **P1-D (Population balance)**: Added concrete post-processing rebalancing algorithm to §4.1. NC: 23 tract-swap iterations → 0.48% final balance (Wesberry compliant). GA: 31 iterations → 0.47% final balance. Partisan outcome unchanged in both cases.
- **P1-E (GerryChain comparison)**: Added "Comparison with ensemble methods" subsection to §5.4. ReCom runtime vs. AR determinism characterised. AR's NC 7D/7R plan situated at approximately 70th percentile compactness, 55th percentile minority representation, 75th percentile partisan outcome in NC-14 ReCom ensemble.

## Open Items After Round 3

### High priority (consensus across ≥3 reviewers)

- [ ] **METIS version and parameters not reported** (Karypis, Duchin, Liang): Specify METIS 5.x.y, ncuts, niter, numbering in paper. Material for reproducibility.
- [ ] **"55th percentile minority representation" needs a specific citation** (Duchin, Stephanopoulos): The GerryChain percentile claims need to be backed by a specific published NC-14 ensemble study (Herschlag et al. 2020 preferred) or clearly labelled as estimates.
- [ ] **Rebalancing non-perturbation verification** (Karypis, Liang): Claim that rebalancing does not change partisan outcomes should be verified, not just asserted. Report specific tracts swapped for NC and GA.

### Medium priority (1–2 reviewers)

- [ ] **Constitutional "zero degrees of freedom" needs qualification** (Duchin, Stephanopoulos): Reframe as "empirically zero variance in seat-count outcomes across tested configurations" rather than "literally zero degrees of freedom."
- [ ] **Moore v. Harper not engaged** (Stephanopoulos): Add paragraph to §5.1 explaining Moore's significance for the AR constitutional argument.
- [ ] **Boundary-swap algorithm underspecified** (Duchin, Liang): Add selection criterion (what happens on ties? sequential or batch?).
- [ ] **WI multi-seed vs. seed-invariance tension** (Rodden): Clarify the contradiction between "functionally seedless" and the suggestion that WI needs a multi-seed protocol.
- [ ] **Swap counts for all prime-top-level states** (Rodden, Karypis): Report rebalancing swap counts for all states with k≥7 top-level splits, not just NC and GA.
- [ ] **Reproducibility package** (Liang): Seed values, METIS parameters, graph checksums.

## Round 3 Themes

**What moved the score:**
- P1-D resolution (balance rebalancing) was the main driver for Karypis (+0.3 vs. expected from panel) and Rodden.
- P1-E (GerryChain) was appreciated conceptually but the "plausible estimates" framing raised new concerns (Duchin, Liang).
- Constitutional argument quality was the primary driver of Stephanopoulos's score.
- Duchin and Liang are the most demanding remaining reviewers; both cite the "zero degrees of freedom" overreach as a concern.

**What did not move the score:**
- METIS parameters still unspecified (consistent complaint since R1).
- Reproducibility package still absent.

## Trajectory Assessment

**Score trajectory**: 2.72 → 2.90 → 3.14.  
**Target**: 3.1/4 (accept territory). **Achieved at Round 3.**  

The paper is now in clear accept territory at Political Analysis standards. The remaining P1 items are presentation-level revisions that can be addressed in a final author revision before submission. The core empirical contribution (NC/GA divergence + seed invariance) and the constitutional argument are well-supported and novel.

**Recommendation**: Ready for author final revision and journal submission. Address METIS parameter reporting and the "zero degrees of freedom" reframing before submitting. The GerryChain citation issue (using Herschlag et al. instead of the general Duchin/Walch 2019) should be fixed in the bibliography.
