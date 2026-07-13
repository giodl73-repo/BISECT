# Pulse 04 Real Ensemble Review

**Date:** 2026-07-10  
**Roles:** DATUM, SCALE, MERIDIAN, COVENANT  
**Posture:** Internal empirical evidence package; not external peer review

## Experiment

- States: Rhode Island, Iowa, North Carolina.
- Wisconsin retained as a failed eligibility case because district 1 was
  disconnected (177 tracts; largest component 119).
- Implementations: Rust ReCom and independent GerryChain 0.3.2.
- Four chains, 2,000 steps, 500-step burn-in, 0.5% population tolerance.
- Partition snapshots every ten steps.
- Primary partisan input: 2020 presidential tract estimates.
- 2016 excluded from primary inference because 2010/2020 tract universes did
  not align.

## Primary Findings

### Tract-graph cut fraction

| State | Rust percentile | GerryChain percentile | Evidence disposition |
|---|---:|---:|---|
| RI | 1.35% | 1.81% | Stable bounded low-tail result; ESS 4,290/6,000 |
| IA | 0.33% | 0.88% | No sub-1% headline; ESS 257/498 |
| NC | 0.00% | 0.20% | Non-converged tail; ESS 109/87 |

Cross-tool KS effect sizes were 0.038, 0.162, and 0.471 respectively. The
nominal p-values are not treated as independent-draw legal tests.

### 2020 presidential Democratic seats

| State | Benchmark | Rust percentile | GerryChain percentile |
|---|---:|---:|---:|
| RI | 2 | 50.0% | 50.0% |
| IA | 1 | 36.7% | 33.9% |
| NC | 5 | 17.1% | 21.2% |

Rhode Island is degenerate: every sampled plan has two Democratic seats.
North Carolina's seat ESS is below 100 in both tools.

## Negative And Heterogeneous Results

- Wisconsin failed the contiguity input gate.
- Iowa and North Carolina do not meet the preregistered ESS threshold for
  publishing extreme cut percentiles.
- Rust and GerryChain cut distributions differ materially in Iowa and North
  Carolina despite matched high-level settings.
- 2016 election estimates lack a complete 2020-tract crosswalk.
- The experiment measures unweighted tract cut fraction, not Polsby--Popper,
  Reock, perimeter, or the benchmark's shared-boundary-weight objective.

## Review Findings And Disposition

- DATUM found one copied Wisconsin hash pointing to NC. The hash was corrected,
  package hashes regenerated, and the verifier extended to check internal
  source-input hashes.
- SCALE confirmed numerical reproduction, percentile and CI calculations,
  R-hat/ESS gates, Hamming diagnostics, and bounded KS interpretation.
- Old G.1 six-state percentiles, G.2 median/corridor generalizations, and G.3
  Polsby--Popper percentile claims were removed from the compiled papers.
- A.0 now cites only the regenerated three-state, metric-bounded evidence.

## Evidence

- `docs/examples/g-ensemble-evidence-packages/G.1-G.3+real-2020/`
- `docs/experiments/2026-07-10-real-ensemble-preregistration.md`
- `docs/experiments/2026-07-10-real-ensemble-deviations.md`
- Revised G.1, G.2, G.3, and A.0 PDFs under `docs/papers/`

## Decision

Pulse 04 may close as a real, hash-bound, three-state empirical package. The
package supports only its stated tract cut-fraction and 2020 presidential
diagnostics. National, polygon-compactness, cross-election, legal, and
externally replicated claims remain blocked.

