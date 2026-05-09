# Review — C.1: Spatial Resolution and Algorithmic Redistricting (MAUP)
**Reviewer**: Albert Liang (Reproducibility, computational methods, statistical computing)
**Round**: 1
**Score**: 3/4
**Verdict**: Minor Revision

## Summary

C.1 addresses the MAUP for algorithmic redistricting and finds that PP and seat counts are stable across resolutions. The methodology is clear and reproducible — the three-resolution experiment is well-specified. My main concern is the single-seed limitation, which Karypis also flags. The reproducibility contribution would be stronger with multi-seed validation.

## Strengths
- The methodology is transparent and reproducible: same bisect parameters, three resolutions, five states.
- The RSI metric is simple and could be included in pipeline output for routine MAUP checking.
- The 50-state PP summary provides a broad view.

## Concerns
- **Single seed**: All results from seed 42. The stability claims need multi-seed support.
- **No confidence intervals**: Table 1 reports point estimates without uncertainty. Even within a single seed, the bisect algorithm may produce different partitions across resolutions — reporting only the RSI mean without uncertainty makes it impossible to know if the stability is consistently achieved or only for these specific runs.
- **Data availability**: The paper relies on county-level, tract-level, and block-group adjacency files. These are not standard outputs of the bisect pipeline for all states. The paper should confirm that these files are available for all 50 states and document how to generate them, so the MAUP check is genuinely replicable.

## Required Changes (P1/P2)
- **P1**: Report multi-seed (≥5) RSI estimates for the five focus states to validate stability claim is not seed-dependent.
- **P2**: Add a note on adjacency file availability and generation procedure for all three resolutions.
- **P2**: Recommend including RSI computation as a standard pipeline output flag (e.g., `bisect label-analyze --types maup`).
