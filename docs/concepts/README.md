# Concepts

Conceptual guides for understanding the BISECT project. Each guide has a "Short version" (TL;DR) followed by a deep dive.

Some older concept guides are workbench material and may contain strong legal,
statistical, or implementation claims. For public-facing claim discipline, use
[research-press.md](research-press.md) and [algorithmic-baseline.md](algorithmic-baseline.md).

## Guides

| Guide | What it explains |
|-------|----------------|
| [recursive-bisection.md](recursive-bisection.md) | The core algorithm: why bisection, how METIS splits the graph, the binary tree structure |
| [edge-weighted-bisection.md](edge-weighted-bisection.md) | How weighting edges by boundary length improves compactness automatically |
| [polsby-popper.md](polsby-popper.md) | The compactness metric: formula, benchmarks, intuition, limitations |
| [vra-compliance.md](vra-compliance.md) | Voting Rights Act districts, the 42% threshold, the metis-vra algorithm |
| [census-data.md](census-data.md) | TIGER shapefiles, PL 94-171 files, GEOID format, downloading |
| [pipeline-stages.md](pipeline-stages.md) | The five pipeline stages, how they connect, how to run individual stages |
| [three-layer-compositor.md](three-layer-compositor.md) | The three orthogonal choices that define every run: structure, weights, search |
| [section-algorithms.md](section-algorithms.md) | The B-series algorithm family: GeoSection, AreaSection, ApportionRegions, and more |
| [label-pipeline.md](label-pipeline.md) | Label-based run management, directory layout, and the SHA-256 audit chain |
| [ensemble-methods.md](ensemble-methods.md) | GerryChain ReCom evaluation, the Rust ensemble engine, and review-stage evidence packets |
| [research-press.md](research-press.md) | Public journal model, issue states, claim classes, and review gates |
| [algorithmic-baseline.md](algorithmic-baseline.md) | Conservative public language for Bisect as a reproducible redistricting baseline |

## Where to go next

- **Running the pipeline**: see `CLAUDE.md` → Common Commands
- **Output files**: see `docs/PIPELINE_OUTPUTS.md`
- **Algorithm detail**: see `docs/RECURSIVE_BISECTION.md`
- **Research papers**: see `research/` and `artifacts/papers/`
- **Public journal curation**: see `research/journals/`
