# Paper Evidence Inventory

## Scope

This inventory resolves DREQ-002 for the internal VTRACE baseline by classifying
the paper rows indexed in `docs/PAPERS.md`.

This is an L1 control inventory. It does not recompute paper tables, validate
every quantitative claim, assert external peer review, publish a release bundle,
or upgrade the current `internal_engineering_baseline_only` posture.

## Inventory source and counts

Source index: `docs/PAPERS.md`.

| Measure | Count | Evidence |
|---|---:|---|
| Indexed paper rows | 146 | Rows matching `\| CODE.NUM \|` in `docs/PAPERS.md`. |
| Rows with PDF links | 134 | Indexed rows whose artifact column links a PDF. |
| Planned/source-only rows | 12 | Indexed rows whose artifact column has no PDF link; currently V.0 through V.11. |
| Committed PDFs under `docs/papers/` | 135 | File inventory, not all necessarily linked from the paper index. |
| Research `.tex` files | 1569 | Source inventory across `research/`. |

## Track coverage

| Track | Indexed rows |
|---|---:|
| A | 6 |
| B | 9 |
| C | 10 |
| D | 9 |
| E | 8 |
| F | 7 |
| G | 15 |
| I | 5 |
| J | 7 |
| K | 8 |
| L | 7 |
| M | 5 |
| T | 17 |
| U | 21 |
| V | 12 |

## Evidence posture mapping

Each indexed paper row inherits one or more evidence postures from its artifact
and note fields:

| Signal in `docs/PAPERS.md` | Inventory posture | Claim boundary |
|---|---|---|
| PDF link present | `indexed_pdf` | A committed PDF is linked; this is not table/figure recomputation evidence. |
| Artifact column has no PDF link | `planned_source_only` | No PDF evidence is claimed from the index row. |
| `Accepted` or `reviewed draft` | `internal_review_marker` | Internal project review only; not external peer review or venue acceptance. |
| `package`, `fixture`, or `verified` | `package_or_fixture_evidence_claimed` | Evidence exists only within the named package/fixture scope. |
| `pending`, `required`, or `missing real-evidence` | `evidence_gap_declared` | The row itself blocks final or release-grade reliance on the affected claim. |
| `planned` | `planned_gap_declared` | The paper is not yet PDF-backed in the index. |

## Declared gap rows

The following indexed rows currently carry explicit planned, pending, required,
or missing-real-evidence language:

| Code | Current index posture | Required before stronger claim |
|---|---|---|
| U.2 | synthetic sweep package added; real 50-state sweep pending | Real 50-state sweep package before final tuning-robustness claims. |
| U.3 | synthetic SA seed/grid smoke package added; state empirical run package pending | State empirical package before final SA performance claims. |
| U.4 | synthetic PT package added; production CLI pending | Production CLI/evidence package before production PT claims. |
| U.5 | adaptive alpha-trace smoke package added; state convergence archive pending | State convergence archive before final adaptive-MCMC claims. |
| U.6 | solver package required | Solver package before exact-redistricting certificate claims. |
| U.11 | resolution mapping smoke package added; autocorrelation archive pending | Autocorrelation archive before final resolution-sensitivity claims. |
| V.0 | planned | PDF/source evidence required before paper-backed claims. |
| V.1 | planned | PDF/source evidence required before paper-backed claims. |
| V.2 | planned | PDF/source evidence required before paper-backed claims. |
| V.3 | planned | PDF/source evidence required before paper-backed claims. |
| V.4 | planned | PDF/source evidence required before paper-backed claims. |
| V.5 | planned | PDF/source evidence required before paper-backed claims. |
| V.6 | planned | PDF/source evidence required before paper-backed claims. |
| V.7 | planned | PDF/source evidence required before paper-backed claims. |
| V.8 | planned | PDF/source evidence required before paper-backed claims. |
| V.9 | planned | PDF/source evidence required before paper-backed claims. |
| V.10 | planned | PDF/source evidence required before paper-backed claims. |
| V.11 | planned | PDF/source evidence required before paper-backed claims. |
| G.1 | active synthetic plus missing real-evidence packages | Archived real trace, election/metric inputs, diagnostics, and RPLAN/RCTX packages before final real-ensemble claims. |
| G.2 | active synthetic plus missing real-evidence packages | Archived real trace, election/metric inputs, diagnostics, and RPLAN/RCTX packages before final real-ensemble claims. |
| G.3 | active synthetic plus missing real-evidence packages | Archived real trace, election/metric inputs, diagnostics, and RPLAN/RCTX packages before final real-ensemble claims. |
| G.6 | endpoint/seed smoke package added; production CLI pending | Production CLI evidence before production short-burst claims. |
| G.12 | seed-stream and diagnostic smoke package added; production CLI pending | Production CLI evidence before calibrated-chain production claims. |

## Use rules

1. Treat `docs/PAPERS.md` as the paper-by-paper source index.
2. Treat this file as the evidence-posture classifier for that index.
3. Do not convert internal labels such as `Accepted`, `reviewed`, `golden`, or
   score values into external peer-review, legal, certification, or release
   claims.
4. Do not cite rows with `pending`, `required`, `missing real-evidence`, or
   `planned` language as final evidence for the missing scope.
5. Before a paper claim is used in a release, legal/court package, public
   dashboard, or evidence bundle, re-run the relevant claim review and update
   DCR, trace, custody, and release-gate records.

## Validation commands

```powershell
$rows = Select-String -Path docs\PAPERS.md -Pattern '^\| [A-Z][A-Z]?\.[0-9]+[^|]* \|' | ForEach-Object { $_.Line }
$rows.Count
$rows | Where-Object { $_ -match '(?i)planned|pending|required|missing real-evidence' }
Get-ChildItem -Path docs\papers -Filter *.pdf -File -Recurse | Measure-Object
Get-ChildItem -Path research -Filter *.tex -File -Recurse | Measure-Object
```
