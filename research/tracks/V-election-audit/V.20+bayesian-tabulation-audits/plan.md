# V.20 Plan

## Thesis

Bayesian audit analytics can be useful for comparison and explanation, but
RCOUNT must distinguish posterior evidence from risk-limiting certification
unless a calibration rule is present.

## Atlas

- `docs/algorithm-atlas/v20-bayesian-tabulation-audits.md`

## Implementation Tasks

- [x] Add analytic transcript fields separate from verifier pass/fail.
- [x] Add synthetic posterior fixture with deterministic metadata.
- [x] Add negative fixture for impossible posterior risk.
- [x] Report Bayesian tabulation as a boundary in audit replay so posterior
  analytics are not confused with risk-limiting certification.
- [ ] Add tiny posterior fixture with hand-computable result.
- [ ] Add calibrated-risk boundary fixture.
- [x] Document UI/reporting language that prevents posterior probability from
  being confused with a frequentist risk limit.

## Claim Boundary

Bayesian outputs are analytic unless explicitly calibrated to a risk limit.
