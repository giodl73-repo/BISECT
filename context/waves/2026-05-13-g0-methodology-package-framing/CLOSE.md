# G.0 Methodology Package Framing Closeout

**Status:** complete

## Outcome

G.0 now reflects the current evidence-package architecture for ensemble
percentile claims. The paper distinguishes executable synthetic package-contract
evidence from real external-ensemble empirical evidence.

## Delivered

- Updated the G.0 percentile framework to require hash-bound trace, metric,
  election, RPLAN, RCTX, and diagnostic artifacts for empirical claims.
- Updated convergence diagnostics to treat diagnostic artifacts as package
  evidence, not self-authenticating table values.
- Updated the conclusion with the active synthetic and missing-evidence package
  boundary.
- Rebuilt `docs/papers/G.0+ensemble-methodology.pdf`.
- Updated `docs/PAPERS.md` and `docs/papers/ALGORITHM-PAPER-SCORECARD.md`.

## Validation

```powershell
Test-Path docs/papers/G.0+ensemble-methodology.pdf
git diff --check
```

## Carry-Forwards

- Add real external trace, election, metric, diagnostic, RPLAN, and RCTX packages
  before citing G.1--G.3 percentiles as empirical findings.
