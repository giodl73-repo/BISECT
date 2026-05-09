# POST-WRITE CHECK — M.3 Housing Character Edge Weights via ACS

**Date**: 2026-05-08
**Status**: READY FOR PANEL

## Validation Summary

| Phase | Result |
|-------|--------|
| Consistency | PASS (4 P1 fixed) |
| Contract | PASS (7/7 promises) |
| Referee sim | Minor Revision |
| Abstract | ~185 words, daggered primary result |

## P1 Fixed

1. **VT formula diverges from spec**: Spec says `(2024 - year) / 100` (unbounded, unclamped). Paper formula `max(0, min(1, 1 - (year-1940)/80))` is correct and supersedes spec. Added implementation note in §3.2.

2. **VT calibration arithmetic error**: Year 2010 was stated as VT≈0.375; correct value is VT=0.125. Fixed in §3.2 calibration table.

3. **MF definition diverges from spec**: Spec informally says "5+ units"; paper uses 10+ units (B25024_007 + B25024_008), which is the correct ACS category boundary. Added clarifying note in §3.2.

4. **Test invariant year labels**: VT=0.10 labeled as ≈2014 (correct: ≈2012); VT=0.80 labeled as ≈1948 (correct: ≈1956). Fixed in §3.6.

## P2 Items

- `vintage_far_apart_low_similarity` L0 invariant needs concrete SF and OO values to be well-posed
- SF definition includes attached units (B25024_003); spec says detached only — add clarifying note

## Key Checks

- ACS tables B25024, B25003, B25035: consistently named throughout.
- VT formula: now consistent after fix.
- All empirical predictions daggered.
- CLI flag `--weights-override housing-character` stated in §3.5.
