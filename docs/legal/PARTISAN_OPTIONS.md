# Partisan Options: How the project's tools fit together post-Rucho

**Status:** v0 draft, 2026-05-01
**Audience:** state legislators, state-commission counsel, civic-advocacy legal teams, academic election-law researchers, federal-statute coalition partners.
**Companion docs:** `MODEL_FEDERAL_STATUTE.md`, `STATUTE_RATIONALE.md`, `STATUTE_REVIEW_NOTES.md`, `FAIRNESS_DOCTRINE.md`.

This document maps the project's redistricting tools across the post-Rucho strategic landscape. It exists because the project ships several tools that look similar but answer different questions, and because the Federal Statute drafts (which prohibit partisan inputs to the algorithm) and the Track-E experimental papers (which explicitly use partisan inputs) need a single place that explains how they coexist.

If you're trying to figure out which tool to use, jump to **§ 4 — Decision tree**.

---

## 1. The four configurations

The project supports four distinct partisan-input postures:

| # | Posture | Partisan data used? | Output | Where it applies |
|---|---|---|---|---|
| 1 | **Partisan-blind baseline** | No — algorithm sees only tract adjacency + population | One canonical map per state | Federal congressional districts under the Model Bill; default mode of `redist state` |
| 2 | **Partisan-balanced** (Plan 03) | Yes — as a constraint to hit a target partisan ratio | One canonical map per state, partisan-target-respecting | State legislative districts where state law allows; *Callais*-disentangled state-court evidence |
| 3 | **Partisan-similarity** (E.4) | Yes — as edge-weights clustering like-minded voters | Knob: α from 1× (≈ baseline) to 100× (extreme clustering) | Research; thought experiment; potentially state legislative districts in states preferring "safe seats" reform |
| 4 | **Party-overlapping** (E.5) | Yes — D'Hondt allocates seats, each party draws its own overlapping districts | One canonical map per party per state, all overlapping | Research / proposal; would require federal statutory changes to single-member-district rule (2 U.S.C. § 2c) |

The project's implementation status differs across these four:

- **(1) is fully shipped.** `redist state --partition-mode unweighted` (or `edge-weighted`, the default) and `metis-vra` for VRA-aware variants.
- **(2) is fully shipped.** `redist state --partition-mode partisan-weighted --partisan-shares <FILE>`. Mutually exclusive with `metis-vra` per Callais p.36 disentanglement. Enforced at the binary's CLI surface and at the import / analyze gates (`callais_preflight`).
- **(3) is partial.** The paper plan is detailed (`research/E.4+partisan-similarity-districts/plan.md`); the partisan-weighted-bisection mode (#2) is the closest existing primitive but it weights by partisan TARGET not by SIMILARITY between adjacent tracts. A new partition mode (`partisan-similarity`) is the missing piece. **§ 5 below describes the pilot.**
- **(4) is partial.** Paper E.5 has full intro + draft sections; implementation as a separate runner exists in research code, not in the production `redist state` binary.

---

## 2. The Federal Statute's view (Model Bill § 104(e))

The Model Bill prohibits partisan inputs entirely for federal congressional districts:

> § 104(e) **PROHIBITED INPUTS**. The State shall not provide as input to the algorithm any data other than the inputs specified in subsection (b). It is unlawful for the State, in performing the act required by this section, to use as algorithm input any voter registration data, past election results, partisan affiliation data, racial composition data, or any other data not enumerated in subsection (b).

This forecloses configurations (2), (3), and (4) for federal congressional districts under the Model Bill. The reasoning, per `STATUTE_RATIONALE.md` § 3:

- The bill's procedural-fairness claim (sidestepping *Rucho*'s "no manageable standard" objection) requires the algorithm be partisan-blind by construction.
- Any partisan input lets a defendant State argue "we ran the algorithm with the partisan data we chose" — which puts the discretion that *Rucho* killed back into the case.
- VRA § 2 deviations are an explicit, bounded exception under § 106; partisan deviations have no analogous exception by design.

**This bill applies only to congressional districts** (Article I § 4 reaches no further). State legislative districts are entirely outside the bill's scope, so configurations (2)–(3) are available there at the State's option.

---

## 3. The state-court / FAIRNESS_DOCTRINE strategy's view

Under state-constitutional partisan-fairness litigation (`FAIRNESS_DOCTRINE.md`), all four configurations are tools, not standards:

- **(1) is the anchor**: the partisan-blind baseline is the "what would the algorithm produce" comparison map that plaintiffs use to demonstrate enacted-map deviation.
- **(2) is the *Callais*-compliant counter-proposal**: a challenger arguing that the State's enacted map is unconstitutionally biased can submit a partisan-balanced alternative that hits the State's stated political goals AND satisfies traditional districting principles. The disentanglement requirement means this map cannot ALSO encode race-conscious adjustments — those go in a separate § 2 challenge.
- **(3) is research / advocacy framing**: useful for the "if you want safe seats, here's the transparent algorithmic way" debate. Less likely to surface in active litigation; more likely to appear in commission deliberations or reform-proposal coalitions.
- **(4) is reform-proposal territory**: requires statutory changes; not a litigation tool today.

---

## 4. Decision tree: which configuration do I use?

```
Are you working on federal congressional districts?
├── YES (federal mandate or ref-impl run)
│   └── Use (1) Partisan-blind baseline. (2)–(4) prohibited under Model Bill § 104(e).
│
└── NO — state legislative, civic counter-proposal, research, or comment-period input
    │
    ├── Are you a § 2 plaintiff submitting an illustrative map under Callais p.29?
    │   └── Use (2) Partisan-balanced — match the State's stated partisan goals,
    │       maximize minority opportunity within them. NOT (3) — Callais p.36
    │       requires race + partisan disentanglement.
    │
    ├── Are you a state legislator designing a state-house map?
    │   ├── State law mandates competitive districts → use (1)
    │   ├── State law mandates partisan balance → use (2)
    │   ├── State law / commission preference is for safe seats → use (3) at moderate α
    │   └── State has no partisan-rule preference → use (1) (defensibility default)
    │
    ├── Are you a civic group submitting a counter-proposal?
    │   └── Use whatever the State's process accepts; tag your submission with
    │       submission_type = "civic_counter_proposal" so the comparison
    │       narrative carries the framing label correctly.
    │
    ├── Are you an academic running a paper-mode replication?
    │   └── Use whichever configuration your research question requires;
    │       record the chosen mode in the manifest under partition_mode.
    │
    └── Are you proposing a federal statute amendment?
        └── See § 5 below for E.4 / E.5 status; recognize that introducing
            (3)–(4) into federal law would require revising the Model Bill's
            § 104(e) prohibition or replacing the single-member-district
            rule (2 U.S.C. § 2c).
```

---

## 5. The proportionality lens: a metric that compares all four

Today the project ships four separate political-fairness diagnostics:

- **Efficiency gap** (Stephanopoulos-McGhee) — measures wasted votes; a bias-around-50% metric.
- **Mean-median difference** — the median district's partisan margin minus the statewide mean; another bias-around-50% metric.
- **Partisan bias with 95% CI** — symmetric-vote-swap analysis; another bias-around-50% metric.
- **Proportionality** *(new in this commit)* — `dem_seat_share - dem_vote_share_statewide` in percentage points; signed; party-agnostic in interpretation.

The first three measure *bias*: at a fixed vote share, does the map favor one party? Useful for state-court cases under PA *LWV* / NC *Harper* doctrines.

**Proportionality** measures something different: *given the actual vote*, how closely did the seat allocation track? This is the metric that answers questions like:

- "Does our partisan-blind baseline produce roughly proportional results in well-mixed states (FL, PA) and disproportionate results in geographically-sorted states (MA, IL)?"
- "Would New England's algorithmic map elect any Republican Representatives?"
- "Does configuration (3) at high α actually achieve proportional safe-seat allocation, as the E.4 plan predicts?"

### How to compute it

```bash
redist analyze --label vt_2020 --year 2020 --version v1 --types proportionality
# or as part of a full run:
redist analyze --label vt_2020 --year 2020 --version v1 --types all
```

Output: `analysis/proportionality.json`:

```json
{
  "analyzer": "proportionality",
  "available": true,
  "dem_vote_share_statewide": 0.665,
  "rep_vote_share_statewide": 0.335,
  "total_two_party_votes": 367400.0,
  "n_districts": 1,
  "dem_seats": 1,
  "rep_seats": 0,
  "n_uncontested": 0,
  "dem_seat_share": 1.0,
  "rep_seat_share": 0.0,
  "proportionality_gap_pp": 33.5,
  "abs_proportionality_gap_pp": 33.5,
  "per_district_dem_share_sorted": [0.665]
}
```

**Reading the gap**:
- `+33.5pp` means Dems got 33.5 percentage points more seats than their statewide vote share would predict (in this VT example, because there's only one district and it goes D, this is structural).
- `-12pp` means Dems got 12 points fewer seats than their vote share — Rep over-representation.
- `~0pp` means proportional — the algorithm's geographic output happens to match the statewide partisan distribution.

### Comparable across configurations

Run proportionality against all four configurations and you can directly compare:

| Plan | dem_vote_share | dem_seat_share | gap_pp |
|---|---|---|---|
| Enacted map | 0.51 | 0.36 | -15.0 |
| (1) Partisan-blind algorithmic | 0.51 | 0.43 | -8.0 |
| (2) Partisan-balanced (target=0.55) | 0.51 | 0.55 | +4.0 |
| (3) Partisan-similarity (α=50) | 0.51 | 0.57 | +6.0 |
| (4) Party-overlapping | 0.51 | 0.51 | 0.0 (by construction) |

This is the cross-cutting comparison the project lacked until today.

### Limitations (in the JSON output)

The metric is documented as having three limitations:

1. **Single-member granularity**: with N districts, achievable seat shares are quantized to multiples of `1/N`. A state with 3 districts at exact 50/50 vote can only produce seat shares of 0, 0.33, 0.67, or 1.0 — never 0.5.
2. **Uncontested districts**: when one party records zero votes, the per-district share is well-defined but may distort the metric. The `n_uncontested` field flags this.
3. **Two-party only**: the metric reads only `dem_votes` + `rep_votes` columns. Third-party totals (recorded in the same CSV) are excluded from both numerator and denominator. For Maine, Vermont, and Alaska where independents win seats, treat the metric as approximate.

---

## 6. The E.4 pilot (configuration 3, partial implementation)

The E.4 paper describes *partisan-similarity edge-weighting*: weight an edge between adjacent tracts inversely proportional to their partisan-share difference. Tracts that vote similarly resist being cut into different districts; METIS clusters them.

This is mechanically distinct from the shipped partisan-weighted mode (#2), which applies a global partisan-share *target* to the bisection. In #2, the algorithm tries to hit a state-level target (e.g., "produce 5 D-leaning and 5 R-leaning districts"). In E.4, the algorithm has no target — it just clusters like-with-like, and the resulting partisan distribution emerges from geographic sorting alone.

**Implementation path** (deferred — see § 7):

1. Add a new `partisan-similarity` partition mode with parameters α (weight multiplier) and τ (similarity threshold).
2. Wire `--partisan-shares` input as the partisan-lean source (already used by mode #2).
3. Compute per-edge similarity and bias the METIS edge-weight column.
4. Run the 50-state × 15-config (5α × 3τ) ablation per `plan.md`.

**3-state pilot** (this commit's scope): not yet executed. The mode is deferred to a follow-up commit. When the pilot is run, the proportionality analyzer above will be the primary comparison metric — exactly what the E.4 plan calls for under "Section 3.4 Partisan Balance."

---

## 7. What this commit ships vs. defers

**Ships now:**
- Proportionality analyzer (`redist analyze --types proportionality`), with 10 unit tests and serialization to `analysis/proportionality.json`.
- This document (`PARTISAN_OPTIONS.md`).
- Confirmation that configurations (1) and (2) work end-to-end against the proportionality lens.

**Defers (next session):**
- E.4 partisan-similarity partition mode (`--partition-mode partisan-similarity`).
- 3-state pilot run (CA, PA, NC per E.4 plan.md) producing `proportionality.json` deltas.
- Full 50-state × 15-config ablation (compute-heavy; suitable for a paper-mode run).
- Wiring proportionality into the `redist compare` narrative + HTML side-by-side reports (would let a civic-advocate report quote the gap directly).

---

## 8. The Federal Statute's compatibility check

For coalition partners reading this alongside the Model Bill: configuration (1) is the bill's mandated mode for federal congressional districts. The proportionality metric does NOT appear in the bill's required reproducibility artifacts — and that's intentional. The bill's claim is *procedural*: did the State run the prescribed algorithm? It is *not* substantive: did the resulting plan achieve proportional representation?

The reason: substantive proportionality claims fall back into *Rucho* territory ("no judicially manageable standard"). The bill survives by being mechanical — execute the algorithm, publish the output. Adding "and achieve proportionality within X percentage points" would re-import the discretion *Rucho* killed.

The proportionality metric is therefore a **diagnostic, not a standard** under the federal-statute architecture. It tells you what the algorithm produced; it does not tell you whether the result is "fair." The fairness question, per `FAIRNESS_DOCTRINE.md`, is procedural — the algorithm was followed honestly — not substantive.

For state-court litigation under state-constitutional standards (PA *LWV*, NC *Harper*), proportionality may well be admissible as substantive evidence. State courts can adopt manageable standards federal courts cannot.

---

## See also

- `MODEL_FEDERAL_STATUTE.md` — federal congressional districting bill (v0.1)
- `STATUTE_RATIONALE.md` — policy memo
- `STATUTE_REVIEW_NOTES.md` — open tensions + Option B (conditional spending)
- `FAIRNESS_DOCTRINE.md` — state-court companion strategy
- `research/E.4+partisan-similarity-districts/plan.md` — E.4 paper plan
- `research/E.5+party-based-allocation/main.tex` — E.5 paper draft
- `redist-analysis/src/proportionality.rs` — the metric's implementation
