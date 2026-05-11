# U.19 Evolutionary Comparison

**Status:** Implementation slice active  
**Track:** U-series search and optimization  
**Crate:** `bisect-pareto`

## Purpose

Extend the existing Pareto/NSGA-II machinery into the algorithm-family roadmap
as the evolutionary comparison path. U.19 is about reproducible comparison and
audited selected outputs, not replacing construction families or exact solvers.

## Existing Foundation

`bisect-pareto` already provides:

- deterministic NSGA-II seeded from content/base seeds
- ReCom-style crossover with validity fallback
- boundary-flip mutation with validity fallback
- per-frontier-entry validity status
- NDJSON frontier output with reproducibility metadata

## U.19 Slice

The U.19 slice adds selected-frontier audit packaging:

- select a frontier entry by index
- convert its 1-based chromosome into RPLAN v0.2 assignments
- emit `selected-frontier.rplan`, `selected-frontier.rctx`,
  `audit-certificate.json`, and `manifest.json`
- include `bisect-pareto` algorithm lineage with selected index, config,
  validity status, generation, and objective values

This keeps the ordinary frontier file light while giving downstream workflows a
full audit package for any selected/exported plan.

## CLI Surface

`bisect pareto` writes the ordinary NDJSON frontier by default. To package a
chosen frontier entry for downstream audit, pass:

```text
bisect pareto --state NC --selected-frontier-index 0 \
  --selected-frontier-context nc_2020.rctx \
  --selected-frontier-out nc-pareto-selected \
  --selected-frontier-label nc-pareto-selected \
  --selected-frontier-tolerance 0.5
```

`--selected-frontier-index` is zero-based. The context flag is required for
packaging because audit certificates are verified against the RCTX unit graph
and population context. If `--selected-frontier-out` or
`--selected-frontier-label` is omitted, the CLI derives stable defaults from
the state, year, and selected index.

## Validity Guarantees

- Crossover returns either a valid child or the valid parent fallback.
- Mutation returns either a valid mutated plan or the unchanged valid input.
- Frontier entries carry plan-level validity status.
- Selected-frontier packages fail before writing a final package if assignment
  labels are outside `1..=k` or the audit fails.

## Tests

- L0/L1 crossover validity
- L0/L1 mutation validity
- deterministic frontier metadata for fixed seed
- selected-frontier RPLAN/RCTX/audit package verification
