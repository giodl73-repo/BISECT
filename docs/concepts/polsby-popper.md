# Polsby-Popper Compactness

## Short version

Polsby-Popper = 4π × (Area / Perimeter²). A perfect circle scores 1.0. A long thin district scores near 0. Higher is more compact. It is the standard legal and academic metric for redistricting compactness. Our algorithm achieves a national mean of 0.361 (95% CI: 0.351–0.370), compared to 0.296 for enacted 2020 congressional districts.

---

## The formula

```
PP = 4π × A / P²
```

Where A is the district area and P is the district perimeter.

This is the isoperimetric quotient — the ratio of a shape's area to the area of a circle with the same perimeter. A circle maximizes area for a given perimeter, so a circle scores exactly 1.0. Any other shape scores lower.

## Intuition

| Shape | Score |
|-------|-------|
| Circle | 1.00 |
| Square | 0.785 |
| Rectangle (2:1) | 0.698 |
| Rectangle (5:1) | 0.403 |
| Typical compact district | 0.30–0.50 |
| Gerrymandered district | 0.05–0.15 |

A district shaped like the letter "C" or "L" has a long perimeter relative to its area, producing a low score. A roughly circular district has a short perimeter relative to its area.

## Why perimeter matters

A long perimeter means the district snakes around to include or exclude specific communities. Short perimeter at fixed area means the district is a cohesive geographic unit — people inside it share a common geography.

Minimizing perimeter is also mathematically equivalent to what our edge-weighted graph partitioner does: minimize total weight of cut edges, where weight = shared boundary length. The algorithm improves Polsby-Popper without ever computing it. See [edge-weighted-bisection.md](edge-weighted-bisection.md).

## Reock score (alternative metric)

Reock = Area / Area of minimum bounding circle.

It measures how much of a district's enclosing circle the district fills. A compact district fills most of its bounding circle; a stretched district does not. We compute both Polsby-Popper and Reock in `compactness/district_compactness.csv`.

Polsby-Popper is more sensitive to perimeter jaggedness. Reock is more sensitive to elongation. Together they provide a fuller picture.

## Benchmarks

| Dataset | Mean PP |
|---------|---------|
| Algorithmic (2020) | 0.361 (95% CI: 0.351–0.370) |
| Enacted 2020 maps | 0.296 |
| Algorithmic (2010) | 0.320 |
| Enacted 2010 maps | ~0.26 |
| Algorithmic (2000) | ~0.31 |

The algorithm beats enacted maps in **37 of 44** multi-district states (2020) — 6 single-district states are excluded as they have no meaningful enacted comparison. The gap has been narrowing as redistricting reform spread: 2010 enacted maps were worse than 2020 enacted maps.

## Where to find compactness scores in outputs

- `states/{state}/data/district_summary.csv` — Polsby-Popper per district (in the `polsby_popper` column)
- `states/{state}/compactness/district_compactness.csv` — Both PP and Reock per district
- `states/{state}/compactness/maps/polsby_popper.png` — Visual map
- `outputs/data/{year}/partitioning/partitioning_statistics_{year}.csv` — National summary

## Limitations

Polsby-Popper penalizes districts along coastlines and rivers, which naturally have long perimeters through no fault of the mapmaker. A coastal district will always score lower than an inland district of equivalent political reasonableness.

We do not adjust for this. The enacted maps face the same coastline penalty, so the comparison is fair. But it is worth keeping in mind when interpreting individual state scores.
