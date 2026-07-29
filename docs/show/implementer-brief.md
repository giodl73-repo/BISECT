# Implementer brief (algorithm / systems engineers)

**Audience:** people who will ask “what is the objective, where is the
nondeterminism, what is proved, and which crate owns the type?”

**Time:** 20–40 minutes cold; longer to run VT + one bakeoff state.

**Posture:** engineering demonstration. Not a claim that the certified
nationwide pipeline is finished.

## Problem statement BISECT optimizes

**Input (default geographic mode):**

- Census units (tract or block, depending on pipeline) with population
- Adjacency with shared-boundary (or configured) edge weights
- District count `k` and chamber/year policy
- Recursive split rule and search/weight modes

**Output:**

- Assignment of units → districts
- Manifests, analysis, maps, reports
- Optional RPLAN packages and audit sidecars
- Optional exact/certificate artifacts on bounded instances

**Default objective stack (heuristic path):**

1. Connected cut approximating the seat-proportional population split
2. Prefer low weighted boundary cut (METIS edge-cut on the weighted graph)
3. Search mode selects among METIS candidates (convergence / percentile / …)

**Certified objective stack (north star):**

1. Best population balance among permitted connected cuts
2. Among those, minimum weighted boundary
3. Among those, lexicographically canonical assignment  
   …with independently checkable proofs. See
   [`../concepts/certified-recursive-bisection.md`](../concepts/certified-recursive-bisection.md).

## The three-layer compositor

Every plan is structure × weights × search:

| Layer | CLI | Implementer meaning |
|---|---|---|
| Structure | `--partition-mode` | Shape of the recursion tree (e.g. ApportionRegions / prime-factor vs plain bisect vs ratio-optimal) |
| Weights | `--weights` | Edge metric (geographic boundary length vs county-sticky, etc.) |
| Search | `--search` | How a METIS multi-start cloud becomes one cut |

**Experimental discipline:** change **one** layer per label. The controlled
bakeoff is documented in
[`../quickstart/quickstart-algorithm-explorer.md`](../quickstart/quickstart-algorithm-explorer.md).

**Empirically important result for implementers:** on states whose `k` factors
symmetrically (NC `14=7×2`, MN `8=2³`), **structure** often moves partisan
overlays more than weight/search. That is a sensitivity result about the
algorithm family, not a fairness theorem.

## System map

```text
bisect-cli          argv, doctor, fetch, labels, reports
bisect-runner       multi-state orchestration (heavy)
bisect-core         partitions, shared domain types
bisect-data         adjacency / serialization
bisect-analysis     metrics, VRA-oriented analytics hooks
bisect-map/report   cartography and reporting
bisect-ilp / exact  bounded exact / proof-oriented paths
rgraph/rstat/ropt…  kernel math (RLINE-shaped, in-tree today)
rplan-*             plan package IO / audit
rcount-*            count package verify / replay
```

Workspace members are listed in the root `Cargo.toml`. Prefer extending a named
crate over growing `bisect-cli/src/runner/ (`mod.rs` + `support.rs` + `tests.rs`)` further.

## Determinism and nondeterminism

| Source | Behavior |
|---|---|
| Graph + weights + tree | Deterministic given pinned inputs |
| METIS multi-start / seeds | Search-dependent; pin `--seed` and search mode for replay |
| Data vintage | TIGER/adjacency hashes must match manifests |
| Certified path | Aims for unique cut under fixed rules + proofs |

Replay posture today is **candidate / local smoke** unless a cited VTRACE
reproducibility gate says otherwise. Do not advertise “clean full-scale
reproducible release” without that gate.

## Proof / exact frontier (read before overclaiming)

Implemented pieces include bounded certificates, recursive certificate trees,
RPLAN binding, hostile corpora, RI connected block RCTX, small-state operational
coverage, and population-optimality stages on named instances.

**Not done:** first full large-State wall-to-wall certificate with boundary +
canonical stages proved end-to-end for production citation.

Track:
[`../../context/waves/CERTIFIED_NATIONAL_ROADMAP.md`](../../context/waves/CERTIFIED_NATIONAL_ROADMAP.md).

```bash
bisect exact --help
```

## Minimal reproduce loop

```bash
# build
cargo build --release -p bisect-cli

# canonical tutorial (1-CD Vermont)
bash examples/vermont-2020-walkthrough/run.sh    # run.bat on Windows
bisect doctor --check-tutorial-data --tutorial vermont-2020

# one-factor structure probe (needs NC 2020 data)
bisect state --state NC --year 2020 --partition-mode apportion-regions \
  --weights geographic --search convergence --label nc_ar
bisect state --state NC --year 2020 --partition-mode standard-bisect \
  --weights geographic --search convergence --label nc_std
bisect label-compare nc_ar nc_std --year 2020
```

## Interfaces worth reading next

| Concern | Start |
|---|---|
| CLI surface | `docs/BISECT_CLI.md`, `bisect --help` |
| Plan packages | `crates/rplan-core`, RPLAN upstream docs if split |
| Count audit | `crates/rcount-*` |
| Edge/vertex weights | `bisect-cli` `edge_weights` / `vertex_weights` modules |
| Ensemble diagnostics | `bisect ensemble`, researcher quickstart diagnostics rules |
| Legal non-goals | `docs/legal/`, VTRACE DCR-006 — do not skip |

## Contribution taste (for PR authors)

1. Prefer one-factor experiments and named labels over silent default changes.
2. Any new public metric language needs a claim packet (evidence + non-claims).
3. Do not expand METIS heuristic results into “certified” wording.
4. Keep Python as auxiliary; Rust CLI is the supported spine.
5. Large orchestration belongs in `bisect-runner` / focused modules, not new
   godfiles.

## Related

- Showcase hub: [`../../SHOWCASE.md`](../../SHOWCASE.md)
- Researcher brief: [`researcher-brief.md`](researcher-brief.md)
- Certified concept: [`../concepts/certified-recursive-bisection.md`](../concepts/certified-recursive-bisection.md)
