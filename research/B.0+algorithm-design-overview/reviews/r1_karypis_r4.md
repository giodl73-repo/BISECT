---
reviewer: George Karypis
round: 4
score: 4
date: 2026-05-05
---

## Summary

Round 4 adds the EC_norm specification paragraph I have requested since Round 2.
The paragraph is placed in the correct location (C2: GeoSection subsection) and
covers the two cases I flagged: the recursive-bisection normalisation
$\mathrm{EC}_\text{norm}(i) = \mathrm{EC}(i) / \sqrt{\min(i, k-i)}$ and the
$n$-way leaf-level simplification $\mathrm{EC}_\text{norm} = \mathrm{EC} / \sqrt{k/2}$.
The paragraph also explains the anti-caterpillar design intent in a way that a
practitioner can verify from the B.8 source.
My P1 item is resolved.
I am upgrading to 4.

## R4 Change: EC_norm Definition Paragraph

**P1.3 (carried from R2/R3) — RESOLVED.**

The new paragraph reads:

> "Formally, $\mathrm{EC}_\text{norm}(i) = \mathrm{EC}(i) / \sqrt{\min(i, k-i)}$,
> where $\mathrm{EC}(i)$ is the total TIGER-weighted boundary length of the cut
> edges produced by the $i:(k-i)$ first-bisection ratio and $k$ is the total
> district count for the state.
> This normalisation penalises extreme ratios ($1:(k-1)$) relative to balanced
> ratios ($\lfloor k/2 \rfloor:\lceil k/2 \rceil$): a peel that isolates one
> district from the rest must clear a compactness bar $\sqrt{k/2 - 1}$ lower
> than the balanced split, preventing caterpillar districts in which the bisection
> tree degenerates into a spine of single-district peels.
> For $n$-way direct partitioning (as in the METIS $k$-way call used at the
> leaf level), the normalisation simplifies to
> $\mathrm{EC}_\text{norm} = \mathrm{EC} / \sqrt{k/2}$."

This is technically correct and sufficient.
The two-case structure (recursive bisection vs. $n$-way leaf) matches the actual
METIS calling convention in the redist CLI.
The design motivation (preventing caterpillar pathology) is correctly stated.

One minor observation: the paragraph does not yet address the AreaSection
dual-constraint interaction I mentioned in R3 as the remaining sub-item —
specifically, whether EC_norm is computed over the area-balanced partitions only
or over all feasible partitions including those rejected by the area Lorenz filter.
The synthesis note I requested was: "The EC_norm comparison is over a family of
partitions that all satisfy the area-balance constraint, with compactness as the
tie-breaking criterion."
This precise clarification is absent from the current text, which describes
EC_norm in the GeoSection (C2) context only, not the AreaSection (C3) context.

However, I am upgrading to 4 because:
(a) the two-case EC_norm definition is the specification gap I originally flagged,
(b) the AreaSection EC_norm interaction is a downstream refinement that B.16
    (if it exists) should address explicitly, and
(c) the current text is sufficient for the B-series internal review track.

I note this AreaSection EC_norm interaction as a P2 item for the journal
submission, unchanged from my R3 position.

## Carry-Forward P2 Items (not blocking)

- AreaSection EC_norm interaction (where in the ratio-scan family does the
  EC_norm comparison occur given the Lorenz pre-filter) — journal condition
- Invariant-state enumeration (9 states not named) — journal condition
- Software version pin (commit hash / release tag) — journal condition
- Estimation model source for each † cell — journal condition

## Score: 4 / 4 — Accept

The EC_norm definition paragraph resolves my primary technical concern.
The paper is now technically complete for the B-series synthesis track.
Journal submission should address the four P2 items above.
