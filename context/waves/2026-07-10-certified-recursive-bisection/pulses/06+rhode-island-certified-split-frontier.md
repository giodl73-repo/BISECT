---
pulse: 06
title: Rhode Island certified split frontier
status: done
depends_on: 03, 05
wave: certified-recursive-bisection
validation_level: L2 real data
---

# Pulse 06 - Rhode Island Certified Split Frontier

Build block adjacency/RCTX custody and attempt the first scalable certified
`1:1` State cut without substituting tract or heuristic evidence.

## Deliverables

- [x] Build canonical sorted 2020 block population custody.
- [x] Build positive shared-boundary land adjacency.
- [x] Emit a valid hash-bound local RCTX.
- [x] Verify Rust parsing and the explicit bounded-size rejection.
- [x] Measure edge, component, and proof-variable counts.
- [x] Identify the Block Island component.
- [x] Apply the established nearest same-county main-component bridge rule.
- [x] Assign median land-boundary weight to 64 deterministic bridges.
- [x] Record discovery and proof-tool availability.
- [x] Publish a source-backed and source-independent blocker package.
- [x] Preserve the no-tract/no-heuristic substitution boundary.

## Result

No exact Rhode Island certificate is issued. The connected bridged RCTX is
complete. The current bounded/static-no-good backend is exponentially inapplicable, and
no production discovery or external proof toolchain is installed.

## Validation

```powershell
python scripts/research/build_ri_block_rctx.py verify docs/experiments/certified-recursive/manifest.json --check-rctx
python -m pytest -q tests/unit/test_ri_block_rctx_frontier.py
cargo run -p bisect-cli -- exact --method certified-recursive --context data/2020/certified/ri_blocks_2020.rctx --out-dir <temp> --districts 2 --exact-fixture-limit 24
python scripts/research/verify_nrs_challenge_bundle.py
cargo fmt --all -- --check
git --no-pager diff --check
```
