---
pulse: 05
title: Proof-producing backend contract
status: done
depends_on: 01, 02, 03, 04
wave: certified-recursive-bisection
validation_level: proof prototype
---

# Pulse 05 - Proof-Producing Backend Contract

Separate fast branch-and-cut or branch-and-price discovery from independently
checkable pseudo-Boolean/SAT optimality and infeasibility proofs.

## Deliverables

- [x] Define a non-proof discovery record.
- [x] Validate discovered assignment, connectivity, and exact objective.
- [x] Compile population lower-bound decision OPB.
- [x] Compile boundary lower-bound decision OPB.
- [x] Compile canonical predecessor decision OPB.
- [x] Encode seat counts, population, cut, orientation, and connectivity.
- [x] Classify bounded requests as SAT or UNSAT by counterexample search.
- [x] Commit optimal-UNSAT and suboptimal-SAT prototype artifacts.
- [x] Bind OPB and request identities into a package manifest.
- [x] Record unavailable RoundingSat/VeriPB toolchain explicitly.

## Result

The discovery/certification interface and deterministic OPB compiler work. No
VeriPB proof is claimed because neither RoundingSat nor VeriPB is installed.
Static connectivity no-goods are generated exhaustively and remain a bounded
prototype, not the scalable backend.

## Validation

```powershell
cargo test -p bisect-ilp proof_backend -- --test-threads=1
cargo run -p bisect-ilp --example certified_proof_path8 -- docs/examples/certified-recursive/path8-k4/output/certified-bisection-tree.json docs/examples/certified-proof-backend/path8-root
python scripts/research/verify_certified_proof_backend.py
cargo fmt --all -- --check
git --no-pager diff --check
```
