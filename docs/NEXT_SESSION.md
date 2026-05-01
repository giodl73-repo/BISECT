# Next Session Handoff: Proportionality + E.4 Pilot

**Status:** active session pickup pointer
**Owner:** maintainer (you)
**Last update:** 2026-05-01 (commits `9084f4d5` + `9defb022`)

This file tells your next session — possibly on another machine — where to start. The repo is the durable artifact; auto-memory is local to a single machine and does not travel.

## What just landed

Two commits introduced six new features and a federal-statute strategy package:

- **`9084f4d5`** — Six features: proportionality analyzer, `--paper-mode`, `validate-ensemble`, HTML compare, civic URL-snapshot + candidate-race CLI, partisan-similarity primitive. 1278 workspace tests pass, 0 regressions.
- **`9defb022`** — Federal Districting Integrity Act drafts: `MODEL_FEDERAL_STATUTE.md` v0.1 + `STATUTE_RATIONALE.md` + `STATUTE_ONE_PAGER.md` + `STATUTE_REVIEW_NOTES.md` + `PARTISAN_OPTIONS.md`.

## Read these in order to pick up

1. **`CLAUDE.md`** — project instructions (always loaded).
2. **`docs/CHANGELOG.md`** — top entries describe the just-shipped features.
3. **`docs/legal/PARTISAN_OPTIONS.md`** — the doc that ties together the four partisan-input postures and explains how the new proportionality metric is the cross-cutting comparison lens. **§§ 6 + 7** are the open-work pointers.
4. **`docs/legal/STATUTE_REVIEW_NOTES.md`** — open tensions in the federal-statute work; Option B (conditional-spending architectural alternative) deferred.
5. **`redist/crates/redist-analysis/src/proportionality.rs`** — the new analyzer's source (well-commented; the unit tests at the bottom illustrate every signal).
6. **`research/E.4+partisan-similarity-districts/plan.md`** — E.4 paper plan; the math primitive shipped, the runner wiring + ablation are the next steps.

## What's NOT in the repo

- **Auto-memory entries** at `C:\Users\giodl\.claude\projects\C--src-apportionment\memory\` — local to the original machine. If you start on a different machine and want continuity, the relevant memories are summarized below in this file.
- **Election data + outputs** — per `.gitignore`. The proportionality analyzer needs `data/{year}/elections/presidential_by_tract.csv`. If you're on the machine with that data, the commands below work directly.

## Memory summary (for machines without auto-memory)

The auto-memory system on the original machine has these project entries:

- **Partitioning preference**: prefer recursive bisection; n-way disrupted fairness metrics.
- **Post-Callais design**: single constraint-driven bisection engine, three modes (state, challenger, commission).
- **Federal-statute strategy** (2026-05-01 pivot): advocate for a Huntington-Hill-style federal Districting Integrity Act; states execute, citizens verify byte-identically. State-court strategy (FAIRNESS_DOCTRINE) is for today; federal-statute strategy is for the 2030 census cycle.

## Concrete next steps (when you have the data)

```bash
# 0. Pull the latest two commits if you're on a fresh machine:
git pull

# 1. Build the binary (or just run via cargo):
cargo build --release --manifest-path redist/Cargo.toml

# 2. Confirm the data is in the expected place:
ls data/2020/elections/presidential_by_tract.csv
ls outputs/v1/2020/states/VT/data/final_assignments.json

# 3. Run proportionality on a single state to sanity-check:
redist analyze --state VT --year 2020 --version v1 --types proportionality
# Expected output:
# [OK] proportionality -> ... (gap = +XX.Xpp; D vote YY.Y% / D seats ZZ.Z%)

# 4. National sweep (loop over all 50 states):
for state in AL AK AZ AR CA CO CT DE FL GA HI ID IL IN IA KS KY LA ME MD \
             MA MI MN MS MO MT NE NV NH NJ NM NY NC ND OH OK OR PA RI SC \
             SD TN TX UT VT VA WA WV WI WY; do
    redist analyze --state $state --year 2020 --version v1 --types proportionality 2>&1 | grep '\[OK\] proportionality'
done

# 5. Compare to enacted maps — same loop with --label enacted_<state>_2020
#    once enacted plans are imported; or use redist compare --format json
#    to side-by-side numerical compare.
```

## Specific carry-over tasks (each well-scoped)

1. **National proportionality sweep + 50-state table.** Roll up per-state `proportionality.json` into a single CSV: `state, year, dem_vote_share, dem_seats, dem_seat_share, gap_pp` for both algorithmic and enacted plans. This answers the New England question (and the broader "what does default bisection score on proportionality") quantitatively. Estimated effort: 1–2 hours including a small `redist aggregate --types proportionality` wiring.

2. **E.4 partisan-similarity runner wiring.** Add `--partition-mode partisan-similarity` flag to `redist state` + integrate `build_partisan_similarity_weights` into `bisection_runner`. The math primitive is shipped (`redist-core::partisan_weights::build_partisan_similarity_weights`, 10 unit tests); this connects it. After wiring, run E.4's 3-state pilot (CA, PA, NC × α=10 / α=50 / α=100) and report proportionality deltas.

3. **`redist compare --format html` proportionality row.** The HTML side-by-side report (`redist-report::html_comparison`) currently shows Districts / Dem-leaning seats / MM count / Mean Polsby-Popper / Total population. Add a Proportionality row showing each plan's gap. Fed by the `redist-analysis::ProportionalityResult` already plumbed through `analysis/proportionality.json`.

4. **Federal Statute v0.2 (only if needed).** v0.1 incorporates the 5-role-review kill-shot fixes. Triggers to revisit:
   - Coalition outreach reveals categorical opposition to the mandate posture → invoke Option B (conditional spending under Dole) per `STATUTE_REVIEW_NOTES.md` § 2.
   - Tightening on the §106 race-predominance issue (the unresolved VRA-vs-EP tension) per `STATUTE_REVIEW_NOTES.md` § 1.

## What's complete and does not need re-doing

- Federal Statute drafts (v0.1) — committed.
- Proportionality analyzer (math + CLI + 10 tests) — committed.
- `--paper-mode` AEA replication renderer — committed.
- `redist research validate-ensemble` CLI — committed.
- `redist compare --format html` self-contained report — committed.
- `redist civic ingest --snapshot-urls` URL archival — committed.
- `redist civic add-candidate-race` end-to-end — committed.
- E.4 partisan-similarity edge-weighting math primitive — committed.
- `PARTISAN_OPTIONS.md` cross-cutting strategy doc — committed.

## See also

- `docs/legal/PARTISAN_OPTIONS.md` — partisan-input postures (1)–(4) explained
- `docs/legal/MODEL_FEDERAL_STATUTE.md` — bill text v0.1
- `docs/legal/STATUTE_REVIEW_NOTES.md` — open tensions
- `docs/legal/FAIRNESS_DOCTRINE.md` — state-court companion strategy
- `docs/CHANGELOG.md` — what shipped when
- `docs/REDIST_CLI.md` — operational CLI reference
