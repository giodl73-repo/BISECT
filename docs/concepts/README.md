# Concepts

Conceptual guides for understanding the REDIST project. Each guide has a "Short version" (TL;DR) followed by a deep dive.

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
| [ensemble-methods.md](ensemble-methods.md) | GerryChain ReCom evaluation, the Rust ensemble engine, and legal certificates |

## Where to go next

- **Running the pipeline**: see `CLAUDE.md` → Common Commands
- **Output files**: see `docs/PIPELINE_OUTPUTS.md`
- **Algorithm detail**: see `docs/RECURSIVE_BISECTION.md`
- **Research papers**: see `research/` and `artifacts/papers/`
