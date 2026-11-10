// Generated from crates/bisect-tui/src/widgets/glossary.rs by scripts/web/native_metric_glossary.py.
export const NATIVE_METRIC_GLOSSARY=Object.freeze([
  [
    "Jaccard",
    "Fraction of census units shared between two plans. Range 0-1; >0.95 = near-identical."
  ],
  [
    "PP / Polsby-Popper",
    "Compactness metric: 4pi*Area/Perimeter^2. Range 0-1; 1 = perfect circle, higher is more compact."
  ],
  [
    "Max dev%",
    "Maximum population deviation from ideal district size, as a percent. Congressional standard: ±0.5%."
  ],
  [
    "County splits",
    "Number of counties divided between two or more districts. Lower is better for community cohesion."
  ],
  [
    "Contiguous",
    "All tracts in each district are geographically connected with no isolated islands."
  ],
  [
    "VRA districts",
    "Majority-minority districts drawn to comply with Voting Rights Act Section 2 (minority opportunity districts)."
  ],
  [
    "Bisection depth",
    "Level of recursive binary splitting (depth 1 = 2 halves, depth N = 2^N parts). Determines granularity of METIS partitioning."
  ]
].map(entry=>Object.freeze(entry)));
