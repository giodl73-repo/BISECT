# Track W -- Election Forensics And Anomaly Analytics

Track W is adjacent to RCOUNT but has a different claim boundary.

V-series papers and algorithms replay audit evidence that can support
certification-style claims when assumptions are met. W-series work produces
investigative analytics: outlier scores, residuals, digit tests, scanner/batch
effects, spatial anomalies, and discrepancy clustering.

The rule is strict: W-series outputs can prioritize review, but they do not
certify election outcomes and do not prove fraud.

## Papers

| Code | Working title | Status |
|------|---------------|--------|
| W.01 | Election Forensic Analytics For RCOUNT Packages | atlas scaffold landed |

## Implementation Contract

Every W-series method must report:

- source package hashes;
- feature set and baseline population;
- model/test id;
- unit-level scores;
- caveats and false-positive boundary;
- a plain statement that the output is investigative, not certifying.

Atlas page: `docs/algorithm-atlas/w01-election-forensic-analytics.md`.
