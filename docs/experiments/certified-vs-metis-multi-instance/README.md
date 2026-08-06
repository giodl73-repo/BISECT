# Certified Versus METIS Multi-Instance Result

**Status:** independently verified committed package

**Protocol:** `certified-vs-metis-multi-instance-v1`

Across all 40 precommitted rows, METIS matched the exact canonical assignment in 20, the complete exact primary objective in 30, and the exact population objective in 30; 0 rows errored, 0 were disconnected, and 0 had insufficient child units.

| Fixture | Units | Exact candidates | Feasible | Exact ties | Exact ms | Assignment matches | Objective matches |
|---|---:|---:|---:|---:|---:|---:|---:|
| `path8-equal` | 8 | 127 | 5 | 1 | 0.175 | 5/5 | 5/5 |
| `cycle10-varied` | 10 | 511 | 35 | 1 | 0.287 | 5/5 | 5/5 |
| `ladder2x6-varied` | 12 | 2047 | 54 | 2 | 1.615 | 0/5 | 0/5 |
| `grid3x4-weighted` | 12 | 2047 | 134 | 4 | 1.391 | 0/5 | 5/5 |
| `barbell12-bridge` | 12 | 2047 | 53 | 1 | 1.437 | 5/5 | 5/5 |
| `tree13-unequal` | 13 | 8190 | 10 | 1 | 4.123 | 5/5 | 5/5 |
| `grid4x4-unequal` | 16 | 65534 | 1198 | 1 | 50.698 | 0/5 | 0/5 |
| `cycle20-equal` | 20 | 524287 | 150 | 6 | 290.608 | 0/5 | 5/5 |


All 40 METIS rows were retained. All were feasible and connected. METIS matched the complete exact primary objective in 30/40 rows and the exact canonical assignment in 20/40 rows. Seed-invariant outcome within each fixture on this backend: `true`.

The two complete-objective disagreements were fixture-wide: on `ladder2x6-varied`, METIS chose a boundary-2 split with scaled population deviations 20/40 while the certified optimum chose exact population balance with boundary 4; on `grid4x4-unequal`, METIS chose deviations 95/190 with boundary 5 while the certified optimum chose 40/80 with boundary 8. Under the precommitted lexicographic objective, population deviation is minimized before boundary, so the certified choices are better under the certified model even though they cut more edge weight.

The weighted grid and 20-cycle matched the exact objective but not the canonical assignment, demonstrating the distinction between objective optimality and deterministic tie selection.

## Rebuild and verify

```powershell
cargo run --release --locked -p bisect-cli --example certified_vs_metis_suite -- <new-output-directory>
python scripts/research/verify_certified_vs_metis_suite.py <new-output-directory>
```

Timings are descriptive observations from `LAPTOP-2IACVKSU` / `Intel64 Family 6 Model 151 Stepping 2, GenuineIntel` using `rustc 1.95.0 (59807616e 2026-04-14)` and `release`; they are not reproducibility requirements or speed claims.

## Claim boundary

Eight precommitted bounded synthetic instances and five fixed seeds only; no State-scale, national, map-quality, fairness, VRA, legal-validity, or adoption claim.
