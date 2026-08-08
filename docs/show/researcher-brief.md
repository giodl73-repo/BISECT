# Researcher brief (election analysts / journalists)

**Audience:** people who think in seats, maps, baselines, and uncertainty —
e.g. a FiveThirtyEight-style election analyst — not maintainers.

**Time:** 15–25 minutes cold; longer if you run code.

**Posture:** empirical research claims with disclosed methods. Not official
results, not court findings. Wording rules:
[`../vtrace/COMMUNICATIONS_STRATEGY.md`](../vtrace/COMMUNICATIONS_STRATEGY.md).

## The story in four beats

1. **Procedure before politics.** Default BISECT builds a plan from Census
   geography and population only. Partisan and racial data are for *analysis*
   (and optional modes), not for the default cut objective.
2. **Recursive bisection is the model.** Regions with `k` seats split
   `⌊k/2⌋ / ⌈k/2⌉` with proportional population targets until one seat remains.
   You can watch rounds (see Minnesota/Alabama figures in the root README).
3. **Design choices still matter.** Especially the **tree structure** (how `k`
   factors). That is the interesting causal story for election researchers —
   not a claim of neutrality-as-virtue.
4. **Operational verification and exact proof are different.** NRS v0.3 has
   complete, independently verified block assignments for all three Census
   cycles. A fully certified BISECT would additionally prove each cut unique
   under fixed objectives; a full State boundary/canonical proof is still the
   frontier.

## What to open first (no install)

1. **NRS v0.3 national verification bundle:**
   [`../external/nrs-v0.3-national-verification/README.md`](../external/nrs-v0.3-national-verification/README.md)
   — three-cycle counts, replay levels, and claim boundaries.
2. **Root README → Results at a glance**  
   Governed block-level evidence first; legacy tract/dashboard claims are
   separately labeled.
3. **This brief’s “talk track”** below.

## Talk track (for a 10-minute conversation)

| Minute | Say | Show |
|---|---|---|
| 0–2 | “Geographic recursive bisection: freeze the rule, turn politics off in the objective.” | README “How it works” |
| 2–5 | “You still choose structure, weights, search — structure moves seats more than people expect.” | NC 7–7 vs 5–9 story (below) |
| 5–7 | “Three governed Census-cycle assignments pass operational checks; structure and subdivision audits are published.” | NRS v0.3 results table / verifier bundle |
| 7–10 | “Operational conformance is verified; exact boundary and canonical optimality are still unproved.” | Certified concept doc |

## Structure dominates (the NC story)

North Carolina has `k = 14 = 7 × 2`. Under **ApportionRegions** (prime-aware
tree), documented bakeoffs often land near **7D / 7R**. Under plain binary
bisection, documented runs land closer to **5D / 9R**.

**Interpretation for researchers:**

- Seat totals can move a lot when the **tree** changes even if the compactness
  objective stays geographic.
- “Nonpartisan inputs” ≠ “proportional seats.” Geography sorts voters; the
  recursion shape interacts with that sorting.
- This is why BISECT is useful as a **baseline family**, not a single sacred map.

Reproduce the controlled bakeoff:
[`../quickstart/quickstart-algorithm-explorer.md`](../quickstart/quickstart-algorithm-explorer.md).

## Headline metrics (how to cite)

Governed NRS v0.3 evidence:

| Claim | How to phrase it | How not to phrase it |
|---|---|---|
| 27,398,654 blocks across three cycles | “The governed assignments cover 50 States and 435 districts in each cycle.” | “One persistent map was reused across decades.” |
| 1,155/1,155 operational nodes pass | “All nodes pass the frozen population-tolerance and connectivity checks.” | “All nodes are mathematically optimal.” |
| 120 common node signatures; 18 exact-topology States | “The recursive structures have these measured similarities.” | “District assignments overlap by this amount.” |
| County/tract split counts | “Descriptive within each Census geography vintage.” | “Splits improved or worsened across vintages.” |

Separate legacy 2020 tract-level research snapshot:

| Claim | How to phrase it | How not to phrase it |
|---|---|---|
| Mean Polsby–Popper ~0.361 vs enacted ~0.296 (~+22%) | “In the project’s 2020 tract pipeline, mean PP was higher for algorithmic maps than for the enacted comparison set.” | “Proves maps are fair / non-gerrymandered.” |
| 37/44 states beat enacted on mean PP | “On this metric and comparison set…” | “Official ranking of states.” |
| 2010 PP ~0.320 with same algorithm | “Geographic procedure is relatively stable across decades of politics.” | “Politics don’t matter.” |
| ApportionRegions seat tallies | “Under this structure and election overlay…” | “The true fair seat total.” |

Before any external article or briefing deck:

1. Re-run or cite the specific label/manifest hashes you used.
2. Note tract vs block resolution if relevant.
3. Link the exact package, paper, or dashboard source; do not merge legacy
   tract metrics into the NRS v0.3 block-level claim.
4. Keep VTRACE posture: not release-final unless a cited gate says so.

## Hands-on (optional)

**Smallest pipeline smoke (1 district):**

```bash
bash examples/vermont-2020-walkthrough/run.sh
```

**Analyst workflow (after data present):**

```text
bisect state --state VT --year 2020 --label vt_demo
bisect label-analyze vt_demo --year 2020 --types all
bisect label-report vt_demo --year 2020 --format html json
```

**Comparative research:**
[`../quickstart/quickstart-researcher.md`](../quickstart/quickstart-researcher.md)
(parameter sweeps, ensembles, diagnostics requirements).

## What BISECT is bad at (say this out loud)

- Replacing a commission’s full multi-criteria deliberation.
- VRA §2 proof packages without expert design (there is a separate Callais
  quickstart; it is advanced and mode-explicit).
- Declaring a map “fair” because it is compact.
- Exact nationwide boundary/canonical uniqueness (not achieved).

## Package context (one slide)

```text
RLINE (kernels) → RPLAN (plan packages) → RCOUNT (count audit)
                         ↑
                      BISECT (draw, map, report, research)
```

Analysts who only need **plan IO / hashing** may care about RPLAN more than the
CLI. Analysts who need **count reconciliation** may care about RCOUNT.

## Next reading

| Depth | Doc |
|---|---|
| Concept | [`../concepts/certified-recursive-bisection.md`](../concepts/certified-recursive-bisection.md) |
| Journals | [`../../research/journals/README.md`](../../research/journals/README.md) |
| Papers | [`../PAPERS.md`](../PAPERS.md) |
| Legal framing (not first meeting) | [`../legal/`](../legal/) |
