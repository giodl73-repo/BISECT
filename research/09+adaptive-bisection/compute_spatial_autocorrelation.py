"""
P2.1: Spatial Structure Analysis - Compute Moran's I

Calculates spatial autocorrelation (Moran's I) for minority distribution
in the 5 test states to validate that high clustering enables method equivalence.

Moran's I ∈ [-1, 1]:
- I > 0: Positive spatial autocorrelation (clustering)
- I ≈ 0: Random spatial pattern
- I < 0: Negative spatial autocorrelation (dispersion)

Expected result: High Moran's I (~0.6-0.8) for test states, supporting
claim that spatial clustering enables method equivalence.
