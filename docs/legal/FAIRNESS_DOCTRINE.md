# The `BISECT` Fairness Doctrine

**Status:** v1, 2026-04-30
**Audience:** litigators preparing redistricting cases, expert witnesses, special masters, journalists evaluating claims about this tool, civic-advocacy groups, opposing counsel.
**Companion docs:** `docs/legal/CALLAIS_REFERENCE.md` (race-related grounding), `docs/file-formats/manifests.md` (chain of custody), `docs/BISECT_CLI.md` (the user-facing surface).

> **Candidate-statute boundary:** The current CLI and many historical examples
> use census tracts and a configured 0.5% tolerance. The candidate federal
> benchmark in NRS v0.1 is block-level, uses equality as nearly exact as
> practicable, and requires a manifest-derived seed that is not yet wired.
> Current tract runs are research and evidence fixtures, not statutory
> conformance.

This document states what the `BISECT` toolchain claims about fairness, what it
does not claim, and what a court can verify. It is the citable artifact for the
project's input-exclusion and reproducibility posture.

If you are arguing that this tool is biased, **read §6 first** — it lists the things this project explicitly does not claim. Many criticisms aimed at "computational redistricting" generally do not apply to this project specifically because we do not assert them.

---

## 1. The bounded procedural claim

The default `BISECT` benchmark excludes partisan, racial, candidate,
incumbency, and community signals from execution inputs. This exclusion is
verifiable, but it does not make the benchmark value-free or substantively
neutral. Structure, resolution, adjacency, weights, seed, engine, and legally
authorized modifications can all affect outcomes.

### 1.1 Input minimality

The core bisection (`BISECT/crates/bisect-core/src/bisection.rs`) sees ONLY:

- The tract adjacency graph (pairs of tracts that share a non-trivial border)
- Per-tract total population (or VAP/CVAP when the operator opts into a population-source change)
- A target district count
- A balance tolerance (default ±0.5%)
- An optional random seed

It does NOT see:
- Party registration, vote share, candidate identity, or any partisan signal
- Race, ethnicity, language preference, or any demographic signal beyond population
- Incumbency, candidate residence, or political-actor preferences
- Community-of-interest (COI) annotations (those flow through the Plan Comparison overlay, not the bisection)

A reader can confirm which signals are absent by inspecting the function
signature and manifest. Absence of those signals prevents direct targeting with
them during execution; it does not establish absence of bias, discriminatory
effect, or strategic upstream profile selection.

**Caveat:** when an operator runs `bisect state --partition-mode partisan-weighted` (Plan 03, Callais 2026), the algorithm DOES consume per-tract Democratic vote shares. This is opt-in only, NOT the default, and is documented in `docs/superpowers/plans/2026-04-29-partisan-bisection-weighting.md`. The Callais p.36 mutex (enforced at three CLI gates per `bisect-report::manifest::callais_preflight`) prevents combining partisan-weighted mode with VRA-aware mode in the same run.

### 1.2 Geometric objective, not political

The algorithm's optimization target is **edge-cut minimization on a graph**: cut as few tract-adjacency edges as possible while keeping per-district populations within the configured balance tolerance. This is a graph-theoretic quantity. It does not reference parties, races, or candidates.

The geometric output happens to correlate with traditional compactness metrics (Polsby-Popper, Reock, convex-hull) which courts have historically accepted as neutral criteria (see e.g., *Karcher v. Daggett*, 462 U.S. 725 (1983); *Larios v. Cox*, 300 F. Supp. 2d 1320 (N.D. Ga. 2004)).

### 1.3 Reproducibility, bit-for-bit

Every plan written by `bisect state` ships with a `manifest.json` (schema documented at `docs/file-formats/manifests.md` §3.1). The manifest records:

- Binary version (`BISECT_version`)
- Build commit (`BISECT_build_commit`, with `-dirty` suffix when built from uncommitted changes)
- Rust compiler version (`rustc_version`)
- SHA-256 of the input adjacency file (`adjacency_sha256`)
- Census TIGER source URL (`tiger_source_url`)
- Population source (`total`, `vap`, or `cvap`)
- Partition mode (`edge-weighted`, `metis-vra`, or `partisan-weighted`)
- Balance tolerance, seed, district count
- ISO-8601 UTC timestamp of plan creation

Verification command: `BISECT doctor --verify-manifest path/to/manifest.json` cross-checks every recorded field against the running binary and surfaces any mismatch.

This record can support chain-of-custody and reproducibility testimony. Whether
it satisfies a specific court's evidentiary requirements remains a
case-specific legal determination.

### 1.4 Constraint-extensible with explicit claim changes

VRA-aligned and partisan-weighted modes are research, litigation, or governed
Layer-C tools. They are not features of the candidate federal geographic
benchmark, which excludes both signals during generation.

The bisection accepts additional constraints — VRA-awareness (`partition-mode metis-vra`), county integrity, contiguity, chamber nesting — that tighten the feasible region but do NOT introduce partisan signals. The constraint code is in:

- `BISECT/crates/bisect-core/src/vra.rs` — VRA-aware edge weighting (race-conscious, but partisan-blind)
- `BISECT/crates/bisect-analysis/src/contiguity.rs` — contiguity check
- `BISECT/crates/bisect-analysis/src/splits.rs` — county/municipal split analysis
- `BISECT/crates/bisect-analysis/src/nesting.rs` — chamber nesting validation

**The Callais p.36 mutex matters here.** A single run of `bisect state` cannot enable BOTH race-conscious mode AND partisan-weighted mode. The mutex is enforced at three places (`bisect state` runtime per `runner::validate_partisan_config`; `BISECT import` and `bisect analyze` via `bisect-report::manifest::callais_preflight`). The downstream consequence is narrower: the manifest can establish that the named benchmark execution excluded race-conscious signals.

### 1.5 Falsifiability via ensemble + convergence diagnostics

A single bisection produces one plan, and many plans satisfy the constraints.
An adequately sampled ensemble can estimate a distribution under one
precommitted legal and algorithmic profile. It cannot bound every neutral
output or define neutrality. The active G.1--G.3 package archives real RI/IA/NC
Rust and GerryChain traces, but supports only the reported tract cut-fraction
and 2020 presidential-seat diagnostics. Iowa and North Carolina retain ESS
limitations, and the package is not national or block-level evidence.
A separate governed Rhode Island block package completed four chains per
kernel, passed its registered scalar rules, and regenerated exactly. Its
material Wilson-versus-Kruskal differences support kernel sensitivity, not
sampler equivalence, mixing proof, or national inference.

The just-shipped `bisect-analysis::ensemble_diagnostics` module provides the convergence diagnostics a court can use to verify the ensemble was sufficient:

- **Gelman-Rubin R-hat** across ≥4 parallel chains (must be < 1.05 for the ensemble to be considered converged; threshold from standard Bayesian practice)
- **Effective Sample Size (ESS)** on summary statistics (efficiency gap, mean-median, MM count) — flag if ESS < 100 (indicates excessive autocorrelation)
- **Hamming-distance autocorrelation** on the partition trajectory itself, with integrated autocorrelation time `tau_int` (partition-space mixing measure that ESS-on-statistics can't capture)

Real ensemble generation and diagnostic replay exist today for the bounded
G.1--G.3 tract package and the separate Rhode Island block slice. A national
block-level expansion remains deferred until its own protocol freezes the
graph, starting plans, samplers, stopping rules, instrumented resource budget,
and claim boundary.

---

## 2. What courts can verify today

| Question a court might ask | What this tool produces today |
|---|---|
| "Was this plan generated by the algorithm its manifest claims?" | `BISECT doctor --verify-manifest <PATH>` (binary version, build commit, input SHA cross-check) |
| "Is this plan internally consistent (population balance, contiguity)?" | `bisect analyze --types summary,contiguity` produces validation booleans |
| "What are the partisan metrics on this plan?" | `bisect analyze --types partisan` (efficiency gap, mean-median, partisan bias, with bootstrap CIs) |
| "Are minority districts of opportunity present?" | `bisect analyze --types vra` (MM-district count by configurable BVAP threshold) |
| "How does this plan compare to that plan?" | `BISECT compare --plan-a A --plan-b B` (Jaccard similarity, population diff, compactness diff, splits diff) |
| "Does this state's racial bloc voting survive partisan controls (Callais p.36)?" | `bisect analyze --types bloc-voting --candidate-race-csv ...` (WLS+HC3, cluster bootstrap by county, Holm-Bonferroni, robustness across 3 baselines, race-of-candidate provenance chain) |
| "Are this plan's race-of-candidate annotations defensibly attested?" | `bloc_voting.json::race_of_candidate_provenance` (curator name + credentials + signed attestation document SHA-256, BD-R2 reconciled format whitelist) |
| "Is the analysis chain free of post-Callais inadmissible mixing?" | `callais_preflight` runs at `bisect state`, `BISECT import`, `bisect analyze` — refuses any plan whose manifest carries both VRA and partisan markers |

---

## 3. What remains beyond the bounded ensemble package

The full Rucho-overcoming methodology requires a small number of additional pieces. Each is documented in a per-capability plan with explicit deferral notes.

| Question | What's needed | Deferred by |
|---|---|---|
| "Where does this plan fall in a national block-level distribution?" | A separately frozen and executed national block-level ensemble protocol; the completed Rhode Island kernel-sensitivity slice is not national evidence | Certified National Roadmap Stage 6 ensemble expansion gate |
| "Did the ensemble explore enough partition space to support a tail claim?" | R-hat / ESS / Hamming tau_int on each generated ensemble; the RI/IA/NC package demonstrates these gates and retains failures | Extend only under the frozen protocol; do not reuse tract-level diagnostics as national evidence |
| "Court-formatted PDF/A-2b expert report" | Typst rendering + verapdf gate | Court Submission Reports Tasks 4-7 |
| "Civic-friendly comparison narrative for press release / public-comment record" | Narrative renderer + `[DRAFT]` gate (shipped as library); CLI dispatch | Plan Comparison Task 11 |

The narrative-renderer, ensemble generation, diagnostics, and provenance-chain
pieces are exercised by the bounded real package. The missing connector is a
separately governed national block-level execution, not basic MCMC capability.

---

## 4. The case-law landscape this serves

### 4.1 Federal partisan claims after Rucho v. Common Cause (2019)

In *Rucho v. Common Cause*, 588 U.S. ___ (2019), the Supreme Court held 5-4 that **partisan gerrymandering claims are nonjusticiable political questions in federal court** under Article III. The decisive ground in Roberts' majority was that there is no "judicially manageable standard" for partisan fairness.

Rucho explicitly **left intact**:

1. **State-court partisan gerrymandering claims under state constitutions.** Roberts' opinion named *League of Women Voters v. Commonwealth*, 178 A.3d 737 (Pa. 2018) approvingly. State courts have continued to adjudicate partisan claims under state Free and Equal Elections clauses, state Equal Protection clauses, and state-specific anti-gerrymandering provisions:
   - Pennsylvania: *LWV* (above), 2018
   - North Carolina: *Harper v. Hall*, 380 N.C. 317 (2022)
   - New York: *Matter of Harkenrider v. Hochul*, 2022 N.Y. Slip Op. 02478
   - Maryland: *In re 2022 Legislative Districting*, 475 Md. 1 (2021)
   - New Mexico: *Grisham v. Reeb*, ___ N.M. ___ (2024)

2. **Federal racial gerrymandering claims under the Voting Rights Act §2 and the 14th/15th Amendments.** Constrained by *Allen v. Milligan*, 599 U.S. 1 (2023) and *Louisiana v. Callais*, 608 U.S. ___ (2026) — see `docs/legal/CALLAIS_REFERENCE.md` for the post-Callais §2 framework.

3. **Independent commission-drawn maps** challenged on procedural or constitutional grounds at the state level.

### 4.2 What state courts have accepted as evidence

State courts adjudicating post-Rucho partisan claims have accepted **ensemble-outlier methodology** as a judicially manageable standard:

- *Harper v. Hall* (NC 2022): expert testimony on ensemble comparison was the central evidence; court ruled the enacted plan was a statistical outlier inconsistent with the state constitution
- *LWV v. Commonwealth* (PA 2018): same; ensemble of computer-drawn neutral plans showed enacted plan as extreme outlier
- *Harkenrider v. Hochul* (NY 2022): expert testimony on partisan bias quantification

This project's stack is engineered to produce exactly the evidence these courts accepted. The legitimate state-court litigator's workflow:

1. Generate an ensemble of partisan-blind plans for the state in question (deferred — Researcher Toolkit Task 6)
2. Validate the ensemble's convergence (`ensemble_diagnostics` — shipped)
3. Compute the enacted plan's percentile rank for partisan metrics (`bisect analyze --types partisan` shipped; percentile reporting deferred)
4. Generate a comparison report (`BISECT compare`, narrative renderer shipped as library; CLI dispatch deferred)
5. Produce a court-formatted PDF (Court Submission Reports — Typst path deferred)
6. Verify chain of custody at trial (`BISECT doctor --verify-manifest` — shipped)

### 4.3 Federal racial claims after Callais

The Callais Evidence Layer (`bisect analyze --types bloc-voting`, shipped in this project) implements the WLS+HC3+Holm+cluster-bootstrap methodology that survives the Callais p.36 disentanglement requirement. See `docs/superpowers/plans/2026-04-30-callais-evidence-layer.md` for the implementation receipts and `docs/legal/CALLAIS_REFERENCE.md` for the legal grounding.

---

## 5. The Rucho "judicially manageable standard" — addressed

Rucho's central rhetorical move was: "We could not even pretend to apply [partisan fairness standards] without venturing far from the well-trodden paths of judicial administration." The Court was unwilling to pick a single partisan-fairness metric (efficiency gap? mean-median? Gallagher? proportional representation?) as constitutionally privileged.

The ensemble-outlier methodology this project supports does not require the court to pick one metric. It says:

> Run one precommitted legal and algorithmic profile many times, validate the
> resulting sample, and report the enacted plan's percentile under that
> profile. The percentile is conditional evidence, not a universal neutral
> distribution or a legal conclusion.

This is judicially manageable because:

- **The constraint set is the court's choice** — the court adopts whatever constraints state law specifies; the algorithm respects them.
- **The metric is the court's choice** — the algorithm reports many metrics; the court picks the one that maps onto the controlling state-constitutional provision.
- **The threshold is the court's choice** — "is the 99th percentile far enough out to constitute intentional gerrymandering?" is a legal question, not a statistical one. The tool reports the percentile; the court draws the line.
- **The methodology is reviewable** — open source, peer-reviewed methods (R-hat, ESS, cluster bootstrap, Holm-Bonferroni), reproducible by anyone with the binary + commit hash + inputs.

The court is not being asked to invent a standard. The court is being asked to apply state law to falsifiable statistical evidence. Rucho's denial of a federal forum does not preclude that workflow at the state level.

---

## 6. What this project explicitly does NOT claim

This section is the load-bearing antiparty-overclaim guardrail. If this list ever becomes incomplete, the doctrine loses its credibility.

### 6.1 We do NOT claim our output is "the most fair" plan

`bisect state` produces ONE plan that satisfies the configured constraints. Many plans satisfy them. We claim:

- THIS PLAN was produced by THIS algorithm under THESE constraints
- THIS PLAN's measured properties are within THESE bounds
- AN ENSEMBLE OF N plans bounds the distribution of neutral output

We do NOT claim:

- This plan is the unique "fair" plan
- This plan is more fair than any other plan satisfying the constraints
- A court should adopt this plan as remedial

### 6.2 We do NOT call geographic packing "gerrymandering"

When voters of one party cluster geographically, a geographic benchmark may
produce seat asymmetry. Geography is one plausible mechanism, but the project
does not infer causation from a single benchmark comparison.

The ensemble-outlier methodology distinguishes natural packing from intentional packing: if the enacted map is at the median of the neutral ensemble's partisan-bias distribution, the bias is geometry. If the enacted map is at the 99th percentile, the bias is intent.

We DO NOT call any plan "gerrymandered" without that ensemble-comparison context.

### 6.3 We do NOT have a position on COIs the algorithm doesn't see

The bisection sees the adjacency graph + populations. It does NOT see communities of interest, neighborhood boundaries, school catchment areas, or any other socially defined community.

The Plan Comparison `--comments-label` overlay (Civic Bidirectional plan, partial) is the consumer of community-of-interest CSVs that civic groups submit. The algorithm doesn't know what a "community" is; the overlay reports whether a plan kept submitter-defined communities whole.

We do NOT claim the algorithm respects communities it cannot see. We DO claim that the overlay surfaces the comparison defensibly.

### 6.4 We do NOT pick the partisan-fairness metric for the court

`bisect analyze --types partisan` reports efficiency gap, mean-median, partisan bias, Gallagher index, and seat-vote curve. The literature has not converged on a single "best" metric; different states' constitutions implicate different metrics. The tool reports all of them; the court selects.

### 6.5 We do NOT certify expert testimony

The race-of-candidate provenance protocol (BD-R2) requires curator attestation documents. We compute SHA-256 of every attestation and embed both the CSV and the documents in the reproducibility zip. We do NOT certify that a curator's classification is correct — we record their name, credential, and signed assertion. The expert witness signs the headline number; we don't.

### 6.6 We do NOT replace the special master

A court-appointed special master evaluates submitted plans against case-specific criteria. This tool produces inputs to that evaluation (verifiable plans, comparison metrics, ensemble percentiles). The special master applies legal judgment. The two roles are not interchangeable.

### 6.7 We do NOT take a position on partisan winners

The algorithm is partisan-blind by default. The Plan 03 partisan-weighted mode is opt-in and explicitly Callais-mutex-gated against race-conscious mode. We ship the partisan-weighted mode primarily so that §2 challengers can demonstrate "the state's stated political goals could have been achieved with better minority outcomes" (Callais p.23) — NOT so that anyone can draw a partisan map and call it neutral.

### 6.8 We do NOT promise that bisection always converges to the global optimum

METIS recursive bisection is a heuristic. It produces good cuts, not provably optimal cuts. The seed parameter affects which local optimum is found. This is why ensemble methodology matters — single-seed claims about "optimality" are unsound.

### 6.9 We do NOT claim Daubert admissibility on behalf of a specific expert

The methodology meets Daubert's testable / peer-reviewed / known-error-rate / general-acceptance criteria (see §1.3 + §1.5). Whether a specific expert witness, in a specific case, can establish admissibility under a specific judge's Daubert ruling is a case-by-case determination. This doctrine document is one input to that determination, not a substitute for it.

---

## 7. The reproducibility recipe

A court-appointed special master or opposing expert verifying a plan produced by this tool should be able to reproduce it from scratch:

```bash
# 1. Clone the repository at the build commit recorded in the manifest.
git clone https://github.com/<owner>/BISECT.git
cd <repo-root>
git checkout <BISECT_build_commit from manifest.json>

# 2. Build the binary with the locked dependency tree.
cd <repo-root>
cargo build --release --locked

# 3. Verify the manifest matches the rebuilt binary.
./target/release/BISECT doctor --verify-manifest path/to/manifest.json

# Expected: [PASS] binary_version matches running binary.
# If [FAIL]: the published plan was NOT produced by this commit.

# 4. Re-run the bisection with the manifest's parameters.
./target/release/bisect state \
    --state <manifest.state_code> \
    --year <manifest.year> \
    --seed <manifest.seed> \
    --balance-tolerance <derived from manifest.balance_tolerance_pct> \
    --output-base /tmp/repro

# 5. Compare the rebuilt plan against the published one.
./target/release/BISECT compare \
    --plan-a <published plan dir> \
    --plan-b <rebuilt plan dir> \
    --format json
# Expected: Jaccard similarity = 1.0; population diff = 0; compactness diff = 0
```

If steps 3 + 5 both pass, the published plan is byte-for-byte verifiable. If either fails, the published plan is unreproducible — which is itself information a court can act on.

For ensemble-based outlier claims (when MCMC wrapper ships), the same recipe extends:

```bash
# 6. Generate the ensemble (when shipped).
python scripts/research/mcmc_ensemble.py \
    --state <code> --year <year> --n-steps 10000 --n-chains 4 \
    --seed <recorded master seed> \
    --output-dir /tmp/repro/ensemble

# 7. Validate convergence.
./target/release/BISECT research validate-ensemble \
    --plan-label <enacted plan> --ensemble-label /tmp/repro/ensemble

# Expected: rhat.json shows R-hat < 1.05 across all summary metrics;
#           ess.json shows ESS > 100 per metric;
#           hamming_autocorr.json shows tau_int within documented range;
#           target_plan_percentiles.json reports the enacted plan's
#           percentile rank for each partisan metric.
```

The output `target_plan_percentiles.json` is the document an expert witness cites when testifying that the enacted plan is or isn't a statistical outlier.

---

## 8. The legitimate concrete claim

> **`BISECT` provides a partisan-input-excluded geographic benchmark with
> published inputs, parameters, manifests, and diagnostics. It does not certify
> fairness, VRA compliance, community preservation, partisan neutrality,
> admissibility, or legal validity. Ensemble percentiles are conditional on one
> precommitted profile and require real archived traces and convergence
> evidence.**

That is the claim this project will defend. Anything stronger is overreach; anything weaker undersells the work.

---

## 9. What this document is NOT

- **Not legal advice.** This is engineering documentation. A litigator using these tools should retain their own counsel.
- **Not a substitute for an expert witness.** The tool produces inputs; an expert testifies. A court does not accept a Markdown file as testimony.
- **Not a partisan position paper.** The project has no preferred party. The methodology is symmetric: it identifies extreme partisan outliers regardless of which party benefits.
- **Not a guarantee of court acceptance.** Each court rules on each motion. This document records what the project provides; whether a specific court depends on it in a specific case is a determination only that court can make.

---

## 10. Maintenance + amendment

This document is versioned. Changes are tracked in git. The current version is **v1, 2026-04-30**.

Substantive amendments — additions to the §6 "what we do not claim" list especially — should be reviewed against the project's current capabilities. If we ever ship a feature that violates §6, either the feature is wrong or §6 needs updating to reflect the new capability honestly. There is no third option.

The doctrine's credibility rests on §6 being complete. If you are reading this and you can think of a claim a critic could legitimately make against the project that §6 does not preempt, please file an issue.

---

## 11. Related documents

- `docs/legal/CALLAIS_REFERENCE.md` — the Callais case, with the PDF SHA-256 + access date, and the post-Callais §2 framework
- `docs/file-formats/manifests.md` — the chain-of-custody manifest schemas
- `docs/file-formats/race-of-candidate.md` — the curator-attested annotation protocol
- `docs/file-formats/citation-strings.md` — Bluebook / APA / Chicago templates per source class
- `docs/error-conventions.md` — the categorized error model used in CLI output
- `docs/BISECT_CLI.md` — the user-facing CLI reference with the §"What courts can verify today" surface
- `docs/superpowers/specs/2026-04-30-roadmap-five-star.md` — the persona-driven capability roadmap
- `BISECT/crates/bisect-analysis/src/ensemble_diagnostics.rs` — R-hat, ESS, Hamming autocorrelation implementations
- `BISECT/crates/bisect-analysis/src/bloc_voting.rs` — WLS+HC3+Holm+cluster-bootstrap (Callais Evidence)
- `BISECT/crates/bisect-report/src/manifest.rs::callais_preflight` — the post-Callais p.36 mutex enforcement
