# Deposition Prep: Fast Iterative Re-Analysis for Trial
**Date:** 2026-04-30
**Status:** Proposed; pending review (NEW spec from SURVEY 9-role review)
**Closes gap for:** §2 expert at deposition / trial, special master under cross-examination
**Depends on:** Callais Evidence Layer, Court Submission Reports, existing redist-cli analyze surface
**Estimated effort:** 3-4 days

## Why this exists

The most stressful moments in a redistricting case are not the report-writing months — they are the deposition hours. Opposing counsel asks: *"Dr. Smith, if you set the leaning threshold to 0.53 instead of 0.55, do you still find a Section 2 violation?"* The expert needs the answer in seconds, not the next morning, and it needs to be derivable from the same record their report was built on.

Today, re-running with a parameter tweak means re-running `redist state` (~2 min for VT, much longer for AL/LA), re-running `redist analyze`, and re-running `redist report`. That is too slow for a deposition rhythm and too disconnected from the original record (a fresh build commit might disagree with the published report). Experts work around this by avoiding sensitivity questions or hand-waving — both bad for credibility.

This spec adds a "what-if" mode where the expert (or special master) keeps the bisection result fixed, varies a single analysis parameter, and gets sub-second turnaround with a manifest tying the answer back to the original record.

## Scope

### In scope

1. **`redist recompute-if` subcommand** — re-runs *only* the analysis layer against a frozen plan with one or more parameter overrides:
   ```
   redist recompute-if --plan vt_2020_v1 \
     --param leaning_threshold=0.53 \
     --param vra_min_bvap=0.50 \
     --param bloc_p_value_method=holm \
     --types partisan,vra,bloc-voting \
     --format json,narrative
   ```
   Output goes to `outputs/{version}/states/{state}/what_if/{timestamp}_{param_hash}/` so each experiment is a separate, manifested artifact.

2. **In-memory plan representation** — the analyze layer loads the plan + tract attributes once and keeps them resident. Subsequent `recompute-if` invocations reuse the loaded state via:
   - `redist deposition-server --plan vt_2020_v1` — long-running daemon process listening on a Unix socket / named pipe
   - Client commands: `redist depo eval --param leaning_threshold=0.53` — sub-second turnaround for parametric tweaks
   - Server prints `READY` when warm; deposition rhythm is "ask, type, answer in 1-2 seconds"

3. **Sensitivity-sweep one-liner** — `redist depo sweep --param leaning_threshold=0.50:0.60:0.01 --metric mm_count` walks the parameter range and prints a small table the expert can read aloud:
   ```
   leaning_threshold | mm_count | dem_seats | gap_to_significance
   0.50              | 2        | 6         | 0.012
   0.51              | 2        | 6         | 0.018
   ...
   0.55              | 1        | 5         | -0.004 (not significant)
   ```
   This is the answer to "is your finding robust to threshold choice" — pre-computable so the expert is never surprised.

4. **Provenance binding** — every `recompute-if` / `depo eval` output records:
   - The original plan label + its `manifest.json` SHA-256
   - The base report's PDF SHA-256 (if generated)
   - The set of parameter overrides + their hash
   - `redist` build commit at the time of the depo (must match the report's build commit; mismatch raises a warning the expert can address: "this build differs from my report; let me re-load the report's build")
   - This is what lets the expert testify "the underlying plan and methods are identical to my report; only this one threshold changed."

5. **Cross-examination notebook template** — a `examples/deposition-checklist.ipynb` the expert pre-runs the night before:
   - Loads the plan
   - Sweeps the 5-10 most-likely-asked parameters
   - Generates a sensitivity table per parameter
   - Bookmarks the answers (PDF or printed) so the expert isn't typing during the depo
   - Each entry has a one-sentence summary the expert can read aloud verbatim

6. **Special-master workflow** — same daemon, different framing: the special master loads a submitted plan and answers questions from the bench in real time without rebuilding the whole analysis pipeline.

### Out of scope

- Re-running the *bisection* itself (different plan, not deposition territory — that's a fresh `redist state` run)
- Live courtroom display ("Mr. Chairman, here is the new map") — that's the State Staff Interop / Districtr loop
- Audio transcription of the deposition into parameter changes (out of scope; Otter.ai, etc.)
- Multi-user shared depo server (single expert per session is the realistic mode)

## Implementation notes

### Why a daemon rather than just a faster CLI

Cold-start cost of `redist analyze` is dominated by:
- Loading the adjacency graph (1-30s depending on state resolution)
- Loading per-tract attribute tables (Census + election data, 1-5s)
- JIT warmup for analysis routines

These are amortized to zero in a daemon. A pure CLI would still pay them per command; a deposition rhythm cannot tolerate 5-30s pauses.

### Parameter override surface

The set of parameters exposed to `recompute-if` is a curated whitelist, not arbitrary code injection:
- `leaning_threshold` (default 0.55)
- `close_call_band` (default 0.02)
- `vra_min_bvap` (default 0.50)
- `bloc_p_value_method` (default `holm`; alternatives `bonferroni`, `none`)
- `bloc_robust_se_type` (default `hc3`; alternative `hc1`)
- `bloc_cluster_unit` (default `county`; alternative `tract`)
- `compactness_metric` (default `pp`; alternative `reock`, `convex_hull`)
- `partisan_efficiency_threshold` (default 0.07)

Anything outside the whitelist requires a fresh `redist analyze` run with explicit CLI flags — and a corresponding new manifest. The whitelist is precisely the set of parameters most likely to be probed in cross-examination.

### Daemon protocol

Length-prefixed JSON messages over Unix socket / Windows named pipe:
```
{"op": "eval", "params": {"leaning_threshold": 0.53}, "types": ["partisan"]}
→ {"status": "ok", "results": {...}, "elapsed_ms": 340}
```

No network exposure (loopback / pipe only). The daemon shuts down on a `SIGTERM` and writes a "deposition log" of every eval it served — useful for reconstructing the depo afterwards.

## Outputs

```
outputs/{version}/states/{state}/what_if/
├── {ISO8601_timestamp}_{param_hash}/
│   ├── analysis.json              # The recomputed analysis under the override
│   ├── narrative.md               # [DRAFT] narrative under the override
│   ├── manifest.json              # Override params, parent plan SHA, parent report SHA
│   └── deposition_context.txt     # Optional human note ("asked by opposing counsel re: VRA threshold")
└── deposition_log_{date}.jsonl    # All daemon evals from one session
```

## CLI surface

```
redist recompute-if --plan LABEL [--year YYYY] \
  --param KEY=VALUE [--param KEY=VALUE ...] \
  --types {all|partisan|vra|bloc-voting|compactness} \
  [--format {json|narrative|both}] \
  [--note "free-text context"]

redist deposition-server --plan LABEL [--year YYYY] \
  [--socket /tmp/redist.sock] \
  [--whitelist-config path/to/extra_params.json]

redist depo eval --param KEY=VALUE [--param KEY=VALUE ...] \
  [--types ...] [--note ...]

redist depo sweep --param KEY=START:STOP:STEP --metric METRIC \
  [--types ...] [--out depo_sweep.csv]

redist depo log [--since "1 hour ago"]   # show recent evals from the daemon
redist depo stop
```

## Tests

- L0 **whitelist enforcement**: pass `--param arbitrary_key=value`; assert non-zero exit + "parameter not in whitelist" error
- L0 **sweep correctness**: hand-compute mm_count for thresholds 0.50, 0.55, 0.60 on a synthetic 10-tract plan; assert the sweep output matches
- L0 **provenance binding**: run `recompute-if` against a plan whose manifest is mutated mid-run; assert the warning fires
- L0 **build-commit mismatch warning**: bind the daemon to a plan whose report was built on commit X; spoof the running binary as commit Y; assert the warning text
- L1 **daemon rhythm**: start a daemon on a small synthetic plan; submit 50 evals back-to-back; assert p99 < 1.5s
- L1 **deposition log replay**: run a daemon session of 10 evals; assert the log file contains all 10 and `redist depo log` reads them back
- L2 skipped-by-default — full Vermont depo simulation: pre-built sweep + 20 ad-hoc evals; assert the report's headline numbers are reproducible from the daemon

## Risks

| Risk | Mitigation |
|---|---|
| Daemon protocol becomes a security risk if exposed | Loopback / named-pipe only; no TCP option; documented |
| Whitelist too narrow — expert needs a parameter not in it | The list is one config file; can be extended pre-depo with `--whitelist-config` so changes are explicit + versioned |
| Build-commit drift between report and depo (expert upgrades during prep) | Loud warning at daemon startup; `redist deposition-server --enforce-build-commit` errors out |
| Expert mis-reads sweep output mid-depo | One-line auto-summary per row ("at threshold=0.53 the bloc-voting result remains significant at p<0.05 after Holm correction") |
| Parametric tweak silently changes underlying assumptions (e.g., changing `bloc_cluster_unit` invalidates the Holm correction) | Each whitelist entry has dependencies; changing it triggers a re-run of dependent steps automatically; documented per parameter |

## Definition of done

- `redist deposition-server` warm-starts in ≤ 5 seconds for VT, ≤ 30 seconds for LA
- `redist depo eval` p99 ≤ 1.5 seconds across the whitelist parameter set
- `redist depo sweep` produces a readable table for the 8 default whitelist parameters
- Build-commit mismatch warns; `--enforce-build-commit` blocks
- Deposition log records every eval; `redist depo log` reads it back
- One worked example (`examples/deposition-checklist.ipynb`) demonstrates a full pre-depo sweep set
