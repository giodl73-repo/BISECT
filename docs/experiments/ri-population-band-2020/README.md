# Rhode Island population-band diagnostic

Date: 2026-09-10. Result: the pre-run weight-response hypothesis is falsified
within this candidate family and fixed experiment schedule. No production
rule or previous experiment was changed.

| Maximum relative child deviation | Eligible physical cuts | County-weight winner changed | Split counties among winners |
|---|---:|---|---:|
| 10 ppm (0.001%) | 0 | No feasible candidate | — |
| 100 ppm (0.01%) | 1 | No | 4 |
| 1000 ppm (0.1%) | 4 | No | 4 |

Five predetermined roots, the same source graph, and additive alpha values
0, 0.5, 1, 2, 4, 8 were fixed before execution. Every proper tree subtree is
tested for eligibility, not only each tree's population-minimum cut. Complements
and repeated physical partitions are deduplicated. The input retains 25,649
blocks and population 1,097,379. Bridge weights and zero-population units are
unchanged.

With two seats and total population P, a cut with population p satisfies a
band b exactly when `abs(2*p-P)*1000000 <= b*P`. Both children's relative
deviation from their ideal P/2 is `abs(2*p-P)/P`. These bands are experimental
parameters, not legal thresholds. An empty band means no eligible cut in
these five trees, not global infeasibility.

## What the result means

The 1000-ppm band admits three additional physical cuts beyond the 100-ppm
band. Its winner differs from the tighter band's winner even at alpha zero:
that is a population-objective effect, not a county-weight response. Within
each band the winner stays unchanged across all tested county weights.

All four eligible cuts split four counties. Their cross-county cut cost is
identical (5,359,049), so ordering by raw cut also orders by within-county cut.
Consequently any nonnegative additive alpha preserves that ordering within
this particular candidate set. A direct split-county count would tie all four;
it cannot distinguish these candidates either.

This is evidence to test a materially different candidate generator—for
example county-aware candidate trees or region-first construction—with these
arms preserved as controls. It is not evidence against county preservation
generally, nor permission to keep widening population bands until a desired
answer appears. No confirmation States were used. There is no new complete-plan
package, certificate, compactness analysis, or legal/neutrality claim here.

## Hypothesis and reproducibility

The [frozen hypothesis](../../../signals/discover/hypothesis/ri-population-band-hypothesis-2026-09-10.md)
required both physical choice and a weight-induced winner-set change in at
least one fixed band. The first component passed; the second failed. Per the
discover-hypothesis stopping rule, this investigation stops without changing
the bands. The hypothesis artifact remains unchanged to preserve its pre-run
hash; this README records its outcome.

- [Manifest](manifest.json): input, code and hypothesis hashes; fixed settings.
- [Analysis](analysis.json): all four physical candidates, every band and alpha.

```powershell
python -m unittest discover -s scripts/research -p test_ri_population_band.py
python scripts/research/run_ri_population_band.py run --manifest docs/experiments/ri-population-band-2020/manifest.json --out target/ri-population-band-new.json
```

Five tests cover exact threshold inclusion, enumeration beyond population
minima, empty bands, a synthetic county-weight response, and subtree population
accounting with a nonzero root. The actual run reproduces byte-for-byte; the
100-ppm candidate also matches the previous strict five-root result by physical
hash, deviation and costs. The runner refuses changed hashes and existing
outputs. Serial execution has no enforced memory or time cap; this remains a
small diagnostic, not the full-plan resource-governed bakeoff. Empty-band scores
are serialized as the string `"None"`; use status and empty winner lists to
identify infeasibility. Local pre-run hashes are not external timestamp proofs.
