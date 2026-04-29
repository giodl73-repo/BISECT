# Louisiana v. Callais — Reference

| Field | Value |
|---|---|
| Citation | 608 U.S. ___ (2026) |
| Argued | 2025-10-15 |
| Decided | 2026-04-29 |
| Vote | 6-3 |
| Majority | Alito (Roberts, Thomas, Gorsuch, Kavanaugh, Barrett) |
| Concurring | Thomas (joined by Gorsuch) |
| Dissenting | Kagan (joined by Sotomayor, Jackson) |
| Disposition | 732 F. Supp. 3d 574 affirmed and remanded |
| Source URL | https://www.supremecourt.gov/opinions/25pdf/24-109_21o3.pdf |
| PDF SHA-256 | `2243c2444d6f56503fac1bd39870aa90cc140007a8a3c7960921aff50f8369b0` |
| Date accessed | 2026-04-29 |

## Holding

> Because the Voting Rights Act did not require Louisiana to create an additional majority-minority district, no compelling interest justified the State's use of race in creating SB8, and that map is an unconstitutional racial gerrymander.

## Why this case is in the project

This case is the legal grounding for the partisan-edge-weighting feature that lives in `redist/crates/redist-core/src/partisan_weights.rs` (introduced via Plan 03 at `docs/superpowers/plans/2026-04-29-partisan-bisection-weighting.md`).

Callais (p. 23 majority) describes the strong-inference framework that a §2 challenger uses to demonstrate intentional discrimination: showing that the State's stated political goals could have been achieved with better minority outcomes. The partisan-weighting feature lets the same algorithm be invoked by both sides — a state hits its partisan target with maximally compact districts; a challenger uses the same engine, with the same partisan target as a constraint, to produce an alternative map that improves minority outcomes.

Callais (p. 36 majority) requires that race-conscious and partisan signals not be mixed in a production map run. The partisan-weighting code in `redist-core` and the existing VRA code in `redist-core/src/vra.rs` are mutually exclusive at the CLI level — a single `redist state` invocation can use one or the other, never both. See Plan 03 Task 4.

## PDF storage

The PDF itself is not committed to the repository (per `.gitignore` rule on `*.pdf`). To verify the SHA-256, download the source PDF and run:

```
sha256sum 24-109_21o3.pdf
# or on Windows:
certutil -hashfile 24-109_21o3.pdf SHA256
```

The hash above is the audit trail.
