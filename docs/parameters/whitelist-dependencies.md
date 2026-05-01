# Whitelist Parameter Dependency DAG

**Status:** Source of truth (consumed by `redist depo recompute` and the future deposition daemon).
**Owner:** Deposition Prep plan Task 1 (`docs/superpowers/plans/2026-04-30-deposition-prep.md`)
**v2.1 tracking:** S-01

This document is the explicit dependency graph for the eight parameters exposed via `redist depo recompute --param KEY=VALUE`. The orchestrator consults this map to decide:

- **Which downstream computations a parameter change invalidates** (the analyzer must be re-run)
- **Which guardrails fire** (some parameter values block headline-significance language)
- **Which warnings surface** (some combinations are statistically anti-conservative)

The machine-readable counterpart is at `redist/crates/redist-cli/whitelist_dependencies.json` (compile-time embedded; the repo's top-level `/data/` is gitignored for raw Census downloads). The CI test asserts that the markdown table below and the JSON file declare the same parameter set with the same defaults; drift is impossible-to-merge.

If you're adding a new whitelist parameter: edit the table below AND the JSON in the same commit. CI will reject the PR if they disagree.

---

## Parameter table

| Parameter | Default | Type | Range | Invalidates downstream | Blocks narrative when | Warns when | Owner |
|---|---|---|---|---|---|---|---|
| `leaning_threshold` | 0.55 | float | [0.0, 1.0] | `close_call_band_flags`, `mm_count`, partisan-narrative classifications | — | — | Plan Comparison plan |
| `close_call_band` | 0.02 | float | [0.0, 0.10] | `close_call_band_flags` only (does NOT invalidate `mm_count`) | — | — | Plan Comparison plan |
| `vra_min_bvap` | 0.50 | float | [0.30, 0.70] | `mm_count`, VRA analyzer outputs | — | — | redist-analysis::vra_analysis |
| `bloc_p_value_method` | `holm` | enum | `holm`, `bonferroni`, `none` | Holm correction values | `none` blocks "statistically significant" wording in narrative | — | redist-analysis::bloc_voting |
| `bloc_robust_se_type` | `hc3` | enum | `hc3`, `hc1` | HC3/HC1 standard errors | — | `hc1` AND `n_clusters < 30`: anti-conservative; HC3 recommended (Long & Ervin 2000) | redist-analysis::bloc_voting |
| `bloc_cluster_unit` | `county` | enum | `county`, `tract` | Cluster bootstrap CI **AND** Holm correction (different cluster unit changes the test count, which redefines the family) | — | — | redist-analysis::bloc_voting |
| `compactness_metric` | `pp` | enum | `pp`, `reock`, `convex_hull` | Compactness analyzer outputs only; no cross-analyzer fan-out | — | — | redist-analysis::compactness |
| `partisan_efficiency_threshold` | 0.07 | float | [0.0, 0.20] | Partisan analyzer outputs only | — | — | redist-analysis::partisan |

---

## Why each edge exists

### `leaning_threshold` → `close_call_band_flags` + `mm_count`

The narrative classifies a district as "Democratic-leaning" iff its Dem-share ≥ `leaning_threshold`. Changing the threshold:

- Re-classifies every district whose Dem-share is between the old and new threshold (close-call set changes)
- Re-counts the seat totals ("Plan A elects N Democratic-leaning seats")
- Affects `mm_count` because some classifications cross the majority-minority bar

The orchestrator must re-run the partisan analyzer + per-district narrative classification when this changes. The plan-comparison narrative renderer auto-handles the re-classification once the underlying numbers are recomputed.

### `close_call_band` → `close_call_band_flags` only

The close-call band is purely a narrative-flagging concern; the `mm_count`, partisan composition, and analysis numbers don't change. Cheap recompute.

### `vra_min_bvap` → `mm_count` + VRA analyzer

The VRA analyzer's `analyze_mm_districts` filters by `bvap_share >= vra_min_bvap`. Changing this threshold re-classifies districts at the boundary; downstream MM-count narrative paragraphs change too.

### `bloc_p_value_method=none` → blocks narrative significance language

When the user disables multiple-testing correction, the bloc-voting result still produces raw p-values, but the narrative renderer MUST NOT emit "statistically significant" (or equivalent) wording. The Holm correction is what makes the significance defensible at the family-level; without it, every test is one-shot.

The orchestrator surfaces this as a hard guardrail (not a soft warning): if `bloc_p_value_method=none`, narrative output strips significance claims and the JSON output's `draft_interpretation` notes the disabled correction.

### `bloc_robust_se_type=hc1` + `n_clusters < 30` → warning

HC1 robust SEs are anti-conservative when `n_clusters < 30` (Long & Ervin 2000, "Using Heteroscedasticity Consistent Standard Errors in the Linear Regression Model"). HC3 is the safer default; HC1 is offered as a sensitivity probe but flagged when used in the small-cluster regime.

### `bloc_cluster_unit` → invalidates Holm correction

Changing the cluster unit (e.g., from county to tract) changes:

1. The cluster bootstrap CI (different resampling units, different effective sample size)
2. The number of distinct clusters (n_clusters), which feeds back into the HC3 sandwich denominators
3. **The Holm family** — when bootstrap-derived p-values change, the per-test family member changes, and the joint Holm correction must be recomputed across the entire family

The orchestrator must re-run BOTH `cluster_bootstrap` AND the joint Holm step; it cannot just patch one cluster's value.

### `compactness_metric` → compactness analyzer only

Polsby-Popper, Reock, and convex-hull-ratio are all isolated district-level metrics. No cross-analyzer fan-out.

### `partisan_efficiency_threshold` → partisan analyzer only

The "fair" / "unfair" partisan classification is a single-analyzer concern.

---

## Guardrail enforcement points

Per the table:

1. **`bloc_p_value_method=none`** triggers the narrative-significance block at:
   - `redist-analysis::bloc_voting_writer::render_summary_md` (when shipped)
   - `redist-report::narrative::render_narrative` (Plan Comparison narrative consumer)
   - The depo recompute output's `draft_interpretation` field

2. **`bloc_robust_se_type=hc1` AND `n_clusters < 30`** triggers a one-line stderr warning at:
   - `redist analyze --types bloc-voting` invocation
   - `redist depo eval` (when daemon ships)

These guardrails are not auto-enforced today (the bloc-voting analyzer doesn't yet read these whitelist parameters as overrides — that wiring is the depo daemon's Task 4/5). The doc + machine-readable map are the contract; the enforcement code lands when the daemon does.

---

## Cross-references

- Spec: `docs/superpowers/specs/2026-04-30-deposition-prep.md`
- Plan: `docs/superpowers/plans/2026-04-30-deposition-prep.md` Task 1
- Tracking: `docs/superpowers/specs/2026-04-30-v21-tracking.md` S-01
- Bloc voting parameter consumers: `redist-analysis::bloc_voting`
- Plan Comparison narrative consumer: `redist-report::narrative`
- Long & Ervin (2000): "Using Heteroscedasticity Consistent Standard Errors in the Linear Regression Model"
