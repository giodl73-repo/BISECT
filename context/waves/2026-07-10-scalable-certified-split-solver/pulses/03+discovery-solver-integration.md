---
pulse: 03
title: Discovery solver integration
status: done
depends_on: 02
wave: scalable-certified-split-solver
validation_level: solver integration
---

# Pulse 03 - Discovery Solver Integration

Produce deterministic connected split incumbents and exact objective records
without treating solver metadata as proof.

## Deliverables

- [x] Add `bisect exact --method certified-discovery`.
- [x] Use fixed seed, iteration count, and target ratios.
- [x] Enable METIS connectivity for equal-seat discovery.
- [x] Reject disconnected candidates before emission.
- [x] Recompute the exact integer objective from the certified instance.
- [x] Emit instance, discovery, and hash manifest.
- [x] Mark every discovery package `unproved-incumbent`.
- [x] Demonstrate deterministic replay.
- [x] Produce and analyze the Rhode Island root incumbent.

## Rhode Island Result

- populations: 548,689 / 548,690;
- connected children: yes / yes;
- scaled deviation: 1;
- weighted boundary cut: 102,659,356;
- arithmetic population floor: 1; and
- population proof status: verified after Pulse 05.

## Validation

```powershell
cargo test -p bisect-cli exact_cmd --lib -- --test-threads=1
python scripts/research/analyze_ri_certified_discovery.py verify docs/experiments/scalable-certified/manifest.json --check-local
python -m pytest -q tests/unit/test_ri_certified_discovery.py
cargo fmt --all -- --check
git --no-pager diff --check
```
