---
pulse: 22
title: Ensemble evidence ledger reconciliation
status: complete
wave: nrs-2010-national-baseline
validation_level: documentation and package replay
---

# Pulse 22 - Ensemble Evidence Ledger Reconciliation

## Purpose

Reconcile public G-track documentation with the already completed real
RI/IA/NC ensemble package before defining any separate block-level expansion.

## Findings

- The hash-bound real package and verifier were complete.
- `docs/PAPERS.md`, the paper scorecard, G.0, and the fairness doctrine still
  described the earlier synthetic or missing-evidence posture.
- G.1--G.3 compiled sources already used the bounded real evidence and retained
  the Wisconsin eligibility failure, incomplete 2016 coverage, low ESS, and
  cross-tool disagreement.

## Delivered

- [x] Route the public paper index to
  `docs/examples/g-ensemble-evidence-packages/G.1-G.3+real-2020/`.
- [x] Reconcile the scorecard and fairness doctrine with the real package.
- [x] Update G.0 to the three-state tract-level evidence boundary.
- [x] Remove two dangling internal bibliography keys and rebuild G.0.
- [x] Preserve the older synthetic and missing-evidence packages as history.

## Claim Boundary

The package supports RI/IA/NC tract cut-fraction and 2020 presidential-seat
diagnostics under its frozen samplers. It does not support national,
block-level, polygon-compactness, multi-election, neutrality, legal-validity,
or exact-optimality claims. Iowa and North Carolina retain the published ESS
limitations.

## Validation

```powershell
python scripts/research/verify_real_ensemble_package.py docs/examples/g-ensemble-evidence-packages/G.1-G.3+real-2020
latexmk -pdf -interaction=nonstopmode -halt-on-error main.tex
git diff --check
```
