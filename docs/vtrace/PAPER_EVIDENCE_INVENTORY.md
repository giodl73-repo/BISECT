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
| Indexed paper rows | 206 | Rows matching `\| CODE.NUM \|` in `docs/PAPERS.md`. |
| Rows with PDF links | 206 | Every current indexed row links a PDF. |
| Planned/source-only rows | 0 | Source-only work may exist outside the current paper index. |
| Committed PDFs under `docs/papers/` | 207 | File inventory; one PDF is not represented by a distinct indexed row. |
| Research `.tex` files | 1728 | Source inventory across `research/`. |

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
| M | 10 |
| N | 6 |
| O | 6 |
| P | 6 |
| Q | 5 |
| R | 5 |
| S | 5 |
| T | 17 |
| U | 21 |
| V | 22 |
| W | 6 |
| X | 6 |

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
| `planned` or `protocol` | `planned_gap_declared` | A PDF may document a proposed method without completed empirical evidence. |

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
| G.0 | active RI/IA/NC tract package; completed replayed RI block gate; v1/v2 failures and completed replayed v3 NH/NM/GA expansion retained | V1 stopped after five primaries on GA Kruskal disk exhaustion; v2 stopped before its first preflight on compiled-runner identity drift. V3 completed all six primaries and exact replays, but GA failed the registered convergence gate. Cite v3 only for bounded feasibility, deterministic replay, registered diagnostics, and State-specific kernel sensitivity—not mixing, sampler equivalence, neutrality, or national representativeness. |
| G.1 | active three-state real package plus retained synthetic/missing-evidence history | RI/IA/NC cut-fraction claims may cite the real package with ESS, metric-identity, and cross-tool caveats; former six-state percentiles remain withdrawn. |
| G.2 | active three-state real package | 2020 presidential seat percentiles may cite the package; 2016 and cross-election claims remain blocked by tract-universe mismatch. |
| G.3 | active three-state cut-fraction package | Tract-graph cut-fraction positions are supported; Polsby--Popper, Reock, and polygon compactness percentiles remain unsupported. |
| G.6 | endpoint/seed smoke package added; production CLI pending | Production CLI evidence before production short-burst claims. |
| G.12 | seed-stream and diagnostic smoke package added; production CLI pending | Production CLI evidence before calibrated-chain production claims. |
| A.0 | governed NRS v0.3 Tier 1--2 bakeoff integrated; clean-worktree vault-backed replay passed; ceremony implementation described | Current assignment/subdivision/common-block geometry claims may cite the published 2020 package and the 2026-09-09 replay record. The large source corpus remains outside Git. Ceremony claims are limited to the tested software contract until an externally witnessed run exists; elections, demographics, ensembles, alternative comparators, and inherited preliminary claims remain gated. |
| B.0 | exploratory four-State matrix; exactly regenerated Wisconsin, pilot, and 44-State packages; prospective ceremony extension added | The frozen full matrix passed 44/44 States and 176/176 cells and regenerates exactly. It is not retroactive ceremony evidence. Future procedural comparisons must use the separate ceremony scoreboard and may not support an overall ranking. |
| K.0 | three-State, four-algorithm, single-seed metric fixtures; separate national geometry contract | Do not pool K-series tract/algorithm values with the governed NRS/CD118 common-block package without a frozen transport study. |
| U.21 | governed operational baselines, clean-worktree three-cycle replay, published Tier 1--2 national comparison, and tested ceremony/prefix-verifier contract; exact proof frontier remains | The 2026-09-09 vault-backed replay matched all 150 governed assignments and the national Tier 1--2 checks passed; the large corpus remains outside Git. External witness validity, an official State ceremony, State-scale weighted-boundary/canonical optimality, and political comparisons remain open. |
| B.02 | candidate advocacy proposal with prospective ceremony administration layer | Canonical-spec alignment, external witness adapters, full seed/structure evidence, and external constitutional review are required before legislative reliance. |
| C.6 | preregistered protocol with synthetic illustrative outputs | Real registration, recruitment, de-identified data, analysis code, and ethics documentation before empirical public-opinion claims. |

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
