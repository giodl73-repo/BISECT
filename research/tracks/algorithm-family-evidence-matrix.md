# Algorithm-Family Paper Evidence Matrix

This matrix maps the new T/U paper-writing queue to implementation artifacts,
CLI surfaces, and current evidence. It is a writing scaffold, not a claim that
all empirical results are complete.

| Paper | Primary claim type | Code home | CLI/YAML surface | Current evidence | Next evidence needed |
|---|---|---|---|---|---|
| T.14 Spectral Partitioning | deterministic construction baseline | `bisect-apportion::spectral`, `bisect-cli` runner | `--structure spectral`, YAML `structure: spectral` | L0 path/two-clique/determinism fixtures; recursive split hardening | real-data smoke and comparison table vs METIS/GeoSection |
| T.15 Capacity-Constrained Clustering | capacity-aware construction | `bisect-clustering` | `--structure capacity-clustering`, YAML `structure: capacity-clustering` | L0 capacity/determinism fixtures; L1 RPLAN sidecar/audit path | real-data feasibility/quality sweep |
| T.16 Hierarchical Regionalization | agglomerative connected-region construction | `bisect-clustering::regionalization` | `--structure regionalization`, YAML `structure: regionalization` | L0 hierarchy/capacity/determinism fixtures; lineage sidecars | benchmark vs flat clustering and METIS |
| T.17 Flow-Based Construction | flow assignment as constructive baseline | `bisect-flow`, `bisect-cli` runner | `--structure flow-construction`, YAML `structure: flow-construction` | L0 capacity/infeasibility/determinism fixtures; L1 sidecar path | clarify scale limits and compare infeasibility witnesses |
| U.0 Search and Optimization Overview | taxonomy/synthesis | docs + all U crates | n/a | implemented U.1-U.20 surfaces | architecture figure and roadmap synthesis |
| U.12 Algorithm-Selection Matrix | practitioner method selection | docs + CLI surfaces | all relevant CLI surfaces | layer-cake concept doc; implemented surfaces | decision table with examples and limitations |
| U.13 Exact-vs-Heuristic Certification | certification semantics | `bisect-ilp`, `bisect-column`, `rplan-*` | `bisect exact`, `bisect verify`, `--structure ilp` | ILP/column solve reports; RPLAN audit verification | formal distinction between proof, bound, audit, and empirical quality |
| U.14 Multi-Objective Selection | trade-off selection methodology | `bisect-pareto`, `bisect-smc`, `bisect-analysis` | `bisect pareto`, ensemble/SMC outputs | Pareto frontier output; selected frontier audit package | selection examples and comparison to SMC/Pareto workflows |
| U.15 Legal Postures for Search | claim discipline | docs + reports | n/a | U.20 audit caveats; legal-use guidance in specs | legal/policy framing examples with limits |
| U.16 Branch-And-Cut Redistricting | exact optimization lifecycle | `bisect-ilp`, `bisect-cli` runner | `--structure ilp --ilp-method branch-and-cut` | solve reports, separation/cut metadata, audit lineage | real-data small-instance report and failure/fallback taxonomy |
| U.17 Branch-And-Price Redistricting | column-generation solver lifecycle | `bisect-column`, `bisect-cli::exact_cmd` | `bisect exact --method branch-and-price` | pricing/master L0; exact fixture package L1 | larger formulation-only examples and column-pool diagrams |
| U.18 Large-Neighborhood Search | validity-preserving improvement | `bisect-local-search`, `bisect-cli::improve_cmd` | `bisect improve`; staged `--search lns`/`tabu` | one-move L0; CLI/RPLAN sidecar L1 | empirical improvement distribution vs no-op cases |
| U.19 Evolutionary Search Comparison | validity-preserving evolutionary comparison | `bisect-pareto`, `bisect-cli::pareto_cmd` | `bisect pareto`, selected-frontier flags | crossover/mutation validity; selected package verification | compare frontier behavior across seeds/objectives |
| U.20 Plan Audit Certificates | fixed-point audit contract | `rplan-core`, `rplan-io`, `rplan-audit`, `bisect verify` | `rplan audit`, `bisect verify`, sidecar emission | schema specs, audit certificates, verifier tests | public-facing examples and failure-mode catalog |

## Shared Figures

- Algorithm family layer cake: derived from `docs/concepts/algorithm-family-layer-cake.md`.
- RPLAN fixed point: `algorithm output -> RPLAN -> RCTX -> audit certificate -> manifest -> verify/report`.
- CLI surface map: `bisect state`, `bisect improve`, `bisect exact`, `bisect pareto`, `bisect verify`.

## Shared Panel Expectations

Each paper should keep its claims falsifiable, distinguish fixture evidence
from real-data evidence, and avoid presenting audit certificates as proof of
political fairness or broad legal compliance.
