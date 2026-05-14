---
wave: rstat-core-kernel
pulse: 02
status: done
depends_on: [01]
governing_roles:
  - SCALE
  - BENCHMARK
---

# Pulse 02 - Probability Kernel

## Completion Notes

- Added `rstat-core::probability::regularized_incomplete_beta`.
- Moved the incomplete-beta/Lanczos-gamma kernel out of
  `bisect-analysis::permutation`.
- Kept the permutation-test report and Bayesian Detection Score interpretation
  in `bisect-analysis`.

