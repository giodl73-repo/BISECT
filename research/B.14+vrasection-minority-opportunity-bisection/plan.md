# B.14 — VRASection: Minority Opportunity Districts via Geographic Concentration Bisection

**Paper Type**: Algorithm Design + Legal Analysis + Empirical Comparison  
**Status**: Planning (2026-05-02)  
**Series**: B (Algorithm Design)  
**Depends on**: B.3 (multi-constraint VRA failure analysis), B.8 (GeoSection — ratio scanning infrastructure)  
**Companion**: D.0–D.4 (VRA compliance papers), B.3 (contrast case — explicit constraint approach)

---

## Core Idea

B.3 demonstrated that adding minority VAP as an explicit second METIS
constraint (ncon=2) fails in 3 of 5 tested states. The failure mode is
**constraint scale mismatch**: minority population fractions (0.0–1.0) and
census population counts (10,000–200,000) differ by 60–800×, producing
METIS numerical instability and infeasible or degenerate partitions.

VRASection abandons explicit minority constraints and instead uses a
**geographic concentration alignment score** layered on top of GeoSection's
isoperimetric ratio scan. The key insight is Gingles Prong 1: legally
cognizable minority communities are geographically concentrated. This means
the spatial distribution of minority VAP is already encoded in the census
tract adjacency graph — compact minority communities will naturally tend to
land on one side of an isoperimetrically-optimal cut.

VRASection makes this tendency explicit and measurable, without treating
minority population as a METIS constraint. Instead, it is a **tie-breaking
score** among candidate ratios that pass the isoperimetric feasibility test.

**The legal distinction is critical.** Under *Shaw v. Reno* (1993) and its
progeny, race cannot be the predominant factor in district design. But using
the *spatial distribution* of any population as a geographic signal — without
reference to race per se — is categorically different from racial targeting.
VRASection's input is minority VAP per tract; its tie-breaking criterion is
geographic clustering score; its output is a bisection that respects natural
geographic concentrations. The algorithm cannot know whether the clustered
population is Black, Hispanic, or any other group — it only knows that some
tracts have high minority VAP and those tracts are geographically co-located.

---

## Failure Recap from B.3

B.3 tested ncon=2 with `tpwgts = [target_ratio, minority_target]` for:

| State | C | Result | Failure mode |
|-------|---|--------|-------------|
| MS    | 4 | Partial success | 1 MM district (vs. 1 under ncon=1) |
| AL    | 7 | Failure | METIS infeasible at ncon=2 |
| GA    | 14 | Partial | 4 MM vs. 4 ncon=1 — no gain |
| TX    | 38 | Failure | Scale mismatch; degenerate partition |
| NC    | 14 | Failure | 60× scale mismatch |

The common failure mechanism: minority VAP fraction (0.0–0.55) is numerically
incompatible with population balance (target ratio 0.5 ± 0.001). METIS
treats both as hard constraints with the same numerical tolerance, producing
infeasible or nearly-uniform partitions.

---

## Algorithm Design

### Phase 1: Geographic Concentration Score

For each candidate bisection ratio i:(k-i), VRASection computes a
**minority geographic alignment score** A(i) that measures whether the
natural geographic split at ratio i places compact minority communities
on the same side.

```
For a bisection (L, R) with |L| = i districts:

  minority_VAP_L = Σ_{t ∈ L} minority_vap(t)
  total_VAP_L    = Σ_{t ∈ L} total_vap(t)
  fraction_L     = minority_VAP_L / total_VAP_L

  minority_VAP_R = Σ_{t ∈ R} minority_vap(t)
  total_VAP_R    = Σ_{t ∈ R} total_vap(t)
  fraction_R     = minority_VAP_R / total_VAP_R

  A(split) = |fraction_L - fraction_R|
```

A high alignment score means the bisection separates high-minority from
low-minority areas. A score near 0 means minority population is evenly
distributed across both sides (no geographic concentration to exploit).

### Phase 2: Feasibility Gate

Only ratios that pass GeoSection's isoperimetric test are eligible:

```
feasible_ratios = {i : EC_normalised(i) ≤ EC_normalised(i*) × (1 + δ)}
```

where i* is GeoSection's pure isoperimetric optimum and δ is a tolerance
parameter (proposed: δ = 0.10 — allow 10% extra edge cut for alignment gain).

### Phase 3: Combined Score Selection

Among feasible ratios, select the ratio maximizing:

```
score(i) = w_geo × (1 - EC_normalised(i) / EC_max)
          + w_vra × A(best_split(i))

where EC_max = max EC_normalised over feasible_ratios
      w_geo + w_vra = 1
      proposed default: w_geo = 0.6, w_vra = 0.4
```

The w_geo weight ensures compactness remains primary; w_vra is the
alignment bonus. The two-weight structure makes the algorithm's VRA
priority explicitly tunable and publishable.

### Phase 4: Recursion

VRASection recurses with GeoSection (not VRASection) on both sub-regions.
The VRA alignment check is only applied at the **first bisection level**,
where geographic concentration is most salient. Within sub-regions, the
minority community is either already concentrated (will naturally cluster
under GeoSection) or distributed (no alignment is available regardless of
score).

This design is deliberate: applying alignment at every level would make
race increasingly predominant at deeper recursion levels — approaching
the legal threshold. Applying it only at level 1 means the algorithm
*observes* minority geography once (at the highest geographic scale) and
then proceeds as a standard compact-district algorithm.

---

## Key Equations

**Alignment score for split (L, R)**:
```
A(L, R) = |MVAP_frac(L) - MVAP_frac(R)|
```

**Feasibility bound** (δ-tolerance on isoperimetric cost):
```
feasible(i) ⟺ EC_norm(i) ≤ (1 + δ) × min_j EC_norm(j)
```

**Combined selection criterion**:
```
i* = argmax_{i : feasible(i)} [w_geo × compactness_score(i)
                               + w_vra × A(best_split(i))]
```

**MM district count estimate** (after full recursion):
Let `f_i = MVAP_frac(district_i)`. Count of majority-minority districts:
```
MM_count = |{i : f_i > 0.50}|
```

---

## Pseudocode Sketch

```
VRASection(G, k, N, minority_vap[], w_geo=0.6, w_vra=0.4, δ=0.10):

  # Step 1: Run GeoSection ratio scan (standard isoperimetric)
  for ratio i in 1..⌊k/2⌋:
    run N METIS seeds at pop target (i/k, 1-i/k)
    record EC_norm(i), best_split(i) = (L_i, R_i)

  # Step 2: Compute feasibility set
  i_iso = argmin EC_norm(i)          # pure GeoSection winner
  feasible = {i : EC_norm(i) ≤ (1+δ) × EC_norm(i_iso)}

  # Step 3: Score feasible ratios by alignment
  for i in feasible:
    A_i = |MVAP_frac(L_i) - MVAP_frac(R_i)|
    cpt_i = 1 - EC_norm(i) / max_{j in feasible} EC_norm(j)
    score_i = w_geo × cpt_i + w_vra × A_i

  # Step 4: Select winner
  i* = argmax score_i

  # Step 5: Recurse (standard GeoSection — VRA alignment NOT repeated)
  GeoSection(G[L_{i*}], i*, N)
  GeoSection(G[R_{i*}], k - i*, N)
```

---

## Empirical Plan

### States

Focus on states with historically significant minority opportunity district
litigation or demographics:

| State | C | Major minority group | Key cases |
|-------|---|---------------------|-----------|
| MS    | 4 | Black (37% VAP) | *NAACP v. Fordice* |
| AL    | 7 | Black (27% VAP) | *Allen v. Milligan* (2023) |
| GA    | 14 | Black (33% VAP), Hispanic (10%) | *Georgia v. Ashcroft* |
| TX    | 38 | Hispanic (40% VAP), Black (13%) | *LULAC v. Perry* |
| NC    | 14 | Black (23% VAP) | *Thornburg v. Gingles* (origin case) |
| LA    | 6  | Black (33% VAP) | *Callais v. Landry* (2025) |
| SC    | 7  | Black (28% VAP) | *Alexander v. SC NAACP* (2023) |

### Experiments

1. **Alignment score distribution**: For each state, plot A(i) across all
   feasible ratios. Does the alignment score peak at the same ratio as
   GeoSection's EC minimum, or at a different ratio? States where they
   coincide have "aligned geography" — compact minority communities and
   compact geographic regions are co-located (ideal outcome with no
   trade-off). States where they diverge reveal the compactness/VRA tension.

2. **MM district comparison**: Run VRASection vs. GeoSection (ncon=1) vs.
   MetisVra (ncon=2, B.3) for all 7 states. Record MM_count under each.
   Hypothesis: VRASection matches or exceeds MetisVra without numerical
   failures, because it leverages Gingles Prong 1 geography rather than
   fighting METIS's constraint solver.

3. **δ and w_vra sensitivity**: For NC and AL (the hardest cases), sweep:
   - δ ∈ {0.05, 0.10, 0.20, 0.50}
   - w_vra ∈ {0.1, 0.2, 0.4, 0.6}
   Measure how MM_count and EC_premium change. Find the minimum w_vra that
   achieves the Gingles-required MM district count.

4. **Alignment score as Gingles Prong 1 proxy**: Plot A(best_split) for all
   states against their minority VAP fraction and a Moran's I spatial
   autocorrelation measure for minority VAP. Hypothesis: A(best_split) ≥ 0.15
   iff Moran's I > 0.4 (geographically concentrated minority communities
   produce measurable alignment scores, matching the legal standard).

5. **Callais validation**: For LA (the most recent Section 2 case), run
   VRASection and verify that the algorithm produces 2 Black-majority
   districts (the remedy required by *Callais v. Landry*) without any
   explicit racial target.

### Metrics

| Metric | Definition |
|--------|-----------|
| `mm_count` | Districts with minority VAP > 50% |
| `alignment_score` | A(best_split) at level-1 |
| `ec_premium_pct` | EC cost above GeoSection minimum (%) |
| `feasible_ratio_count` | Ratios within δ-tolerance |
| `wvra_break_even` | Min w_vra achieving same MM as MetisVra |
| `gingles_prong1_flag` | A ≥ 0.15 (proxy for geographic compactness) |

---

## Expected Findings

- **MS, LA, NC**: VRASection achieves the same MM_count as MetisVra (1 or 2
  MM districts) without METIS infeasibility. EC premium is 5-15% (vs. 0% for
  GeoSection, N/A for failed MetisVra runs).
- **AL, TX**: VRASection partially succeeds — produces aligned level-1 split
  but does not mechanically guarantee Gingles compliance at all recursion
  levels. A post-hoc check flags any VRA shortfall.
- **Alignment score**: Strongly correlated with Moran's I (r ≥ 0.7 across
  7-state dataset). Provides a computable, pre-run prediction of whether
  VRASection will help.
- **δ sensitivity**: Modest δ (10-15%) is sufficient; larger δ trades too
  much compactness for marginal alignment gain.

---

## Legal / Policy Argument

**VRASection is Section 2-compliant without racial predominance.** The
algorithm's input is minority VAP per tract (a census attribute, publicly
available). Its tie-breaking criterion is geographic alignment (a spatial
statistic). It does not draw any district for racial reasons; it selects
a bisection ratio because that ratio separates geographically concentrated
populations, which may or may not be a minority community.

The *Shaw*/*Miller* predominance test asks whether race was the predominant
factor. VRASection's w_vra weight makes this quantifiable: at w_vra = 0.4
(the proposed default), geographic compactness is weighted 50% more heavily
than alignment. The algorithm can certify: "compactness was the predominant
criterion; alignment was a secondary, bounded preference."

**Contrast with MetisVra (B.3)**: MetisVra adds minority concentration as a
hard METIS constraint — race is a co-equal optimization target. This likely
fails *Miller v. Johnson*'s predominance test in states with strict racial
geometry (GA, TX). VRASection uses race only as a soft tie-breaker, and only
at the first bisection level, satisfying a proportionality rather than
maximization standard.

**Gingles Prong 1 operationalization**: The alignment score A(best_split)
provides a quantitative, pre-litigation test for whether a minority community
satisfies Gingles Prong 1 ("sufficiently large and geographically compact").
A ≥ 0.15 means the community is concentrated enough for VRASection to detect
it. A state-level report of alignment scores for all minority groups provides
a defensible, neutral pre-map analysis.

---

## Dependencies

- **B.3 (MetisVra)**: The contrast case. VRASection's value depends on
  demonstrating it succeeds where B.3 fails. All 5 B.3 states should be run
  under VRASection for direct comparison.
- **B.8 (GeoSection)**: VRASection is a wrapper around GeoSection's ratio
  scan, adding only the alignment score computation and the δ-feasibility gate.
  The isoperimetric normalisation (EC / √min(i,k-i)) is unchanged.
- **redist-analysis::bloc_voting**: The Callais evidence layer (landed
  2026-04-30) provides the WLS+HC3 regression infrastructure for validating
  that VRASection districts correspond to legally cognizable minority
  opportunity districts under the Callais standard.

---

## Target Venue

*University of Pennsylvania Law Review* (Section 2 legal analysis) or  
*Journal of Law and Courts* (empirical legal scholarship)  
Backup: *Election Law Journal* (algorithmic redistricting with VRA focus)

---

## Open Questions

1. Is A(best_split) computed on the first-split assignment, or on the
   best-seed assignment at each ratio? (Best-seed is more stable but more
   expensive; first-split is noise-sensitive.)
2. Should w_vra differ by minority group? Hispanic VAP has different
   geographic concentration patterns than Black VAP (urban vs. barrio
   clustering). A single w_vra may be suboptimal.
3. The algorithm recurses with GeoSection (no alignment) after level 1.
   Should sub-regions with large minority concentrations re-run VRASection
   locally? This would be a "VRA-aware full recursion" mode — potentially
   approaching predominance territory.
4. What is the correct δ tolerance for "legally trivial" EC premium?
   A 10% premium on a 3,000km edge-cut is 300km — is this judicially
   acceptable as a "minor modification" to the purely geographic plan?
