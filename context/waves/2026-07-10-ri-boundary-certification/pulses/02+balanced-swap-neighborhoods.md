---
pulse: 02
title: Balanced swap neighborhoods
status: done
depends_on: 01
wave: ri-boundary-certification
validation_level: L2 local search
---

# Pulse 02 - Balanced Swap Neighborhoods

Search same-population and bounded multi-unit exchanges while preserving
connectivity and the population floor.

## Deliverables

- [x] Group boundary units by exact Census population.
- [x] Exchange one unit from each child with equal population.
- [x] Reject articulation removals in both children.
- [x] Require destination adjacency after the counterpart leaves.
- [x] Compute exact two-unit weighted-cut delta.
- [x] Apply only strict deterministic improvements.
- [x] Regenerate discovery and proof models.
- [x] Search deterministic 1-to-2 equal-population exchanges.
- [x] Search a bounded deterministic 2-to-2 neighborhood.

## Result

- same-population pair swaps: 3;
- one-to-two exchanges: 25;
- bounded two-to-two exchanges: 1;
- expanded top-512 two-to-two candidate window: no additional move;
- prior cut: 102,622,860;
- improved cut: 97,994,953;
- pulse reduction: 4,627,907;
- cumulative reduction from population-floor incumbent: 4,664,403;
- population deviation: unchanged at 1; and
- population proof: unchanged and valid.

## Validation

```powershell
cargo test -p bisect-cli exact_cmd::tests::same_population_swap --lib -- --test-threads=1
cargo test -p bisect-cli exact_cmd::tests::one_to_two_swap --lib -- --test-threads=1
cargo test -p bisect-cli exact_cmd::tests::two_to_two_swap --lib -- --test-threads=1
python scripts/research/verify_ri_proof_frontier.py --check-local
python scripts/research/analyze_ri_model_package.py verify docs/experiments/scalable-certified/model-manifest.json --check-local
cargo fmt --all -- --check
git --no-pager diff --check
```
