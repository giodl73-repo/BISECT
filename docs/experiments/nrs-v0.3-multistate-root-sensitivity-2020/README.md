# NRS v0.3 Multi-State Root Sensitivity

**Status:** complete

| State | Root split | Units | Accepted | Unique assignments | Exact benchmark reproductions | Mean agreement | Minimum agreement |
|---|---:|---:|---:|---:|---:|---:|---:|
| GA | 7:7 | 232,717 | 100/100 | 1 | 100 | 100.000000% | 100.000000% |
| NH | 1:1 | 31,948 | 100/100 | 1 | 100 | 100.000000% | 100.000000% |
| NM | 1:2 | 107,215 | 100/100 | 1 | 100 | 100.000000% | 100.000000% |

State-weighted mean benchmark agreement:
100.000000%.

Block-weighted mean benchmark agreement:
100.000000%.

Objective values are not pooled across roots.

All three roots were invariant. The NRS v0.3 path consults the seeded METIS
assignment only after deterministic DFS candidates tie on population
deviation and cut. Further brute-force seed expansion is therefore gated on
instrumenting candidate-tie multiplicity rather than assuming more seeds or
States will exercise the parameter.

## Rebuild And Verify

```powershell
python scripts/research/run_nrs_multistate_root_sensitivity.py `
  --output-dir docs/experiments/nrs-v0.3-multistate-root-sensitivity-2020
python scripts/research/verify_nrs_multistate_root_sensitivity.py `
  docs/experiments/nrs-v0.3-multistate-root-sensitivity-2020
```

## Claim Boundary

Three selected 2020 State roots only; no national node census, ensemble, random-sampling, partisan, compactness, demographic, VRA, legal-validity, optimality, or benchmark-replacement claim.
