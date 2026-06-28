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
| W.01 | Election Forensic Analytics For RCOUNT Packages | framework + report contract |
| W.02 | Turnout And Vote-Share Residuals | method (framework) |
| W.03 | Batch And Scanner Effects | method (framework) |
| W.04 | Digit And Benford-Style Tests | method (framework) |
| W.05 | Spatial Outliers And Neighbor Anomalies | method (framework) |
| W.06 | Change-Point Tests And Audit-Discrepancy Clustering | method (framework) |

The W.01 overview fixes the shared report contract and claim boundary; W.02-W.06
develop one analytic family each. These are method/framework papers: the report
contract is specified and the methods are established, but no forensics crate is
implemented yet (placement in `rcount-audit` reports vs a later
`rcount-forensics` crate remains an open decision).

## Implementation Contract

Every W-series method must report:

- source package hashes;
- feature set and baseline population;
- model/test id;
- unit-level scores;
- caveats and false-positive boundary;
- a plain statement that the output is investigative, not certifying.

Atlas page: `docs/algorithm-atlas/w01-election-forensic-analytics.md`.
