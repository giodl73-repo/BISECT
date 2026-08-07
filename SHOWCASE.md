# BISECT Showcase

**Who this is for:** someone you would hand the repo to for 20–40 minutes —
a data journalist / election researcher (the “Nate Silver path”), or an
algorithm implementer who wants to know what is actually novel and how to
reproduce it.

**Posture:** research and engineering demonstration. Not a public release
package, not court-ready, not an official score, and not a claim that maps are
“fair.” See [`docs/vtrace/COMMUNICATIONS_STRATEGY.md`](docs/vtrace/COMMUNICATIONS_STRATEGY.md).

| Audience | Open this first | Time |
|---|---|---|
| Election researcher / journalist | [Researcher brief](docs/show/researcher-brief.md) | 15–25 min read + optional dashboards |
| Algorithm implementer | [Implementer brief](docs/show/implementer-brief.md) | 20–40 min read + optional VT smoke |
| Either, hands-on | [Vermont 2020 walkthrough](examples/vermont-2020-walkthrough/README.md) | 2–5 min once data is cached |

## One-minute pitch

BISECT draws districts by **recursive geographic bisection** on a Census unit
graph. Shared boundary length weights edges. At each step a region with `k`
seats splits into `floor(k/2)` and `ceil(k/2)` children with matching population
targets. Default mode uses **no partisan or racial inputs**.

That procedure is a **published algorithmic baseline**, not a black-box optimizer
score and not a legal conclusion. A stronger **certified** path aims to prove
each cut is unique under enacted rules (population, then boundary, then
canonical tie-break)—implemented in pieces, not yet a full nationwide proof.

## Two doors

### A. Election researcher path (Silver-style)

**Question BISECT answers well:** *If you freeze a geographic procedure and
turn off political inputs, what maps and seat tallies do you get—and which
design choices move the outcome?*

| Step | What to look at | Why |
|---|---|---|
| 1 | [Public dashboards](https://giodl73-repo.github.io/BISECT/) | Round-by-round maps and 2020/2010 views without installing anything |
| 2 | [Researcher brief](docs/show/researcher-brief.md) | Headline metrics, structure-vs-outcome story, citation rules |
| 3 | README “Results at a glance” | Current paper/dashboard claims with evidence caveats |
| 4 | Optional: VT or NC bakeoff | Hands-on plan labels and CSV metrics |

**Headline research claims (cite with caveats):**

- 2020 tract pipeline: mean Polsby–Popper **~0.361** algorithmic vs **~0.296**
  enacted (**~+22%**), 37/44 states beating enacted on that metric — *empirical
  research claim, not a legal finding; re-verify before treating as final.*
- **Structure dominates** partisan composition in bakeoffs: NC `k=14=7×2` under
  ApportionRegions often near **7D/7R** vs standard binary bisection closer to
  **5D/9R** in documented explorer runs — *structure choice, not “fairness.”*
- Same algorithm on 2010: PP **~0.320** — geographic structure is relatively
  stable across a decade of politics.

**Do not say:** official result, court-ready, certified nationwide map, or that
compactness proves non-partisanship.

### B. Algorithm implementer path

**Question BISECT answers well:** *What is the exact recursive rule, where does
METIS sit, what is proved today, and which packages own graph/plan/count?*

| Step | What to look at | Why |
|---|---|---|
| 1 | [Implementer brief](docs/show/implementer-brief.md) | Three-layer compositor, crate map, proof frontier |
| 2 | [Certified recursive bisection](docs/concepts/certified-recursive-bisection.md) | North-star claim posture |
| 3 | [Algorithm explorer quickstart](docs/quickstart/quickstart-algorithm-explorer.md) | Controlled one-factor experiments |
| 4 | `bisect exact --help` / ILP/proof crates | Bounded exact and certificate machinery |

**Implementer takeaways:**

- **Compositor:** structure (`--partition-mode`) × weights × search — change one
  layer at a time.
- **Heuristic core:** METIS cut at each node; fast nationwide baselines.
- **Certified core:** lexicographic uniqueness + independent proof checking;
  RI and small-state operational coverage exist; full-state boundary/canonical
  proof still the frontier ([roadmap](context/waves/CERTIFIED_NATIONAL_ROADMAP.md)).
- **Packages:** RLINE kernels → RPLAN plans → RCOUNT counts; BISECT is the
  application that generates, maps, reports, and researches.

## Fastest hands-on (both audiences)

```bash
# from repo root after bootstrap / cargo build --release -p bisect-cli
bash examples/vermont-2020-walkthrough/run.sh   # or run.bat on Windows
bisect doctor --check-tutorial-data --tutorial vermont-2020
```

Vermont is **one district** — a pipeline and provenance smoke, not a partisan
story. For a structure story, use North Carolina in the algorithm explorer
quickstart (needs 2020 adjacency/data).

## Claim packet (this showcase)

| Field | Value |
|---|---|
| Claim text | BISECT can be shown as a research/engineering redistricting baseline with separate researcher and implementer entry paths. |
| Audience | Election researchers/journalists; algorithm implementers. |
| Evidence | README results table; public dashboards; certified concept doc; VT walkthrough; algorithm explorer quickstart; VTRACE communications strategy. |
| Validation | L0/L1 documentation and existing tutorial path; not L2 external-user validation. |
| Limitations | Not public-release ready; not legal/court certified; dashboards/papers need re-verify for release-final citation; certified nationwide incomplete. |
| Non-claims | Fairness, VRA compliance, election certification, official scores. |
| Review lane | COMMONS / DATUM / SCALE; BOUNDARY if legal wording is extended. |

## Where not to start

| Avoid leading with… | Why |
|---|---|
| Full 50-state rebuild | Slow; hides the idea under logistics |
| Legal statute drafts alone | Wrong first meeting for researchers/implementers |
| VTRACE control corpus | Maintainer machinery; use only if they ask about release gates |
| Un-caveated seat projections | Easy to overclaim |

## Related

- Persona quickstarts: [`docs/quickstart/`](docs/quickstart/)
- Paper index: [`docs/PAPERS.md`](docs/PAPERS.md)
- VTRACE posture: [`docs/vtrace/INDEX.md`](docs/vtrace/INDEX.md)
