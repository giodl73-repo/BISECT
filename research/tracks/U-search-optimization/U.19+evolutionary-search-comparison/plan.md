# U.19 - Evolutionary Search Comparison

**Paper Type:** Evolutionary/multi-objective implementation  
**Status:** Round-1 review addressed
**Track:** U - Search and Optimization  
**Code Home:** `bisect-pareto`, `bisect-cli::pareto_cmd`  
**CLI Surface:** `bisect pareto`, selected-frontier audit flags

## Research Question

How can evolutionary/Pareto methods expose objective trade-offs while ensuring
selected frontier plans are audit-ready and validity-preserving?

## Hypotheses / Claims

- **H1:** Crossover and mutation can return valid children or explicit valid fallbacks.
- **H2:** Frontier entries can carry validity status and deterministic metadata for fixed seeds.
- **H3:** A selected frontier entry can be packaged as RPLAN/RCTX/audit certificate/manifest and verified through the standard path.

Falsification: invalid frontier output, nondeterministic fixed-seed metadata, or
selected packages that fail `bisect verify`-style certificate checks.

## Scope Boundary

- **In scope:** NSGA-II comparison path, validity-preserving operators, selected-frontier audit packaging.
- **Out of scope:** proof that the discovered frontier is globally complete.
- **Generalizability claim:** the paper establishes selected-output auditability for evolutionary comparison.

## Evaluation Plan

- Baselines: existing U.7 Pareto paper, SMC projection, single-objective construction.
- Evidence: L0/L1 crossover/mutation/determinism tests and selected package verification.
- Evidence needed: seed/objective sensitivity examples.

## Figures and Tables

- Pareto frontier and selected entry diagram.
- Crossover/mutation validity fallback flow.
- Selected package artifact table.

## Limitations

- Search budget affects frontier quality.
- VRA and partisan objectives require careful input disclosure.

## Panel Readiness Checklist

- [x] `main.tex` and sections exist.
- [x] Frontier quality and selected-plan validity are separated.
- [ ] Selected-frontier CLI example included.
- [x] P1 simulated feedback addressed.
