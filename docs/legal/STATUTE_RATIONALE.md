# Statute Rationale: Why a Federal Reproducible Districting Act

**Status:** v0 draft, 2026-05-01
**Audience:** congressional staff, reform-organization legal teams, journalists, academic election-law specialists.
**Companion docs:** `MODEL_FEDERAL_STATUTE.md` (the bill text), `STATUTE_ONE_PAGER.md` (90-second version), `FAIRNESS_DOCTRINE.md` (state-court companion).

This memo answers four questions:

1. Why does Congress need to act? (Rucho left the gap.)
2. Why this approach? (Reproducibility-based mandate, not "fair maps" adjectives.)
3. Why does the algorithm work? (Recursive bisection is uniquely suited to be a statutory standard.)
4. Why now? (The implementation exists. Pre-Callais, it didn't.)

---

## 1. Why Congress: Rucho's invitation

In *Rucho v. Common Cause*, 588 U.S. 684 (2019), Chief Justice Roberts wrote for a 5–4 Court that partisan-gerrymandering claims present a "political question" that federal courts cannot adjudicate, because there is no "judicially manageable standard" for how much partisan advantage is too much.

The opinion was widely reported as the end of federal gerrymandering oversight. That reporting was incomplete. Roberts explicitly listed three paths that *Rucho* did not foreclose:

> "Our conclusion does not condone excessive partisan gerrymandering. Nor does our conclusion condemn complaints about districting to echo into a void. The States, for example, are actively addressing the issue on a number of fronts." — *Rucho*, 588 U.S. at 718.

He then enumerated those fronts: state constitutional litigation (post-*Rucho*: Pennsylvania *LWV*, North Carolina *Harper*), state independent commissions (California, Michigan, Arizona, Colorado), and **federal legislation**. He cited HR 1 (the For the People Act) by name as an example of "a host of recent state constitutional amendments and legislative changes" plus "various proposals for federal legislation."

So the Court did not say "gerrymandering is fine." It said "this is a political-branch problem; we don't have authority to fix it; the political branches do."

Two of the three fronts are working: state constitutional litigation has produced wins in PA, NC, NY, NM. State commissions have produced demonstrably less-biased maps in CA and MI. The third — federal legislation — has not advanced because the proposals to date (HR 1, the John R. Lewis Voting Rights Advancement Act, the Freedom to Vote Act) bundle districting reform with so many other voting-rights priorities that they cannot achieve 60 votes in the Senate.

A narrow, single-issue federal districting statute has not been seriously attempted. This memo proposes one.

## 2. Why reproducibility, not "fair maps"

Most "fair maps" proposals fail the legislative-drafting test because the standard cannot be written down with the precision of an enacted law. Here are the standards reform bills have proposed, and what is wrong with each from a drafting standpoint:

### "Compact" districts

There are at least six accepted mathematical definitions of compactness (Polsby-Popper, Reock, convex-hull ratio, Schwartzberg, length-width, X-symmetry). Each ranks district plans differently. A compactness statute either picks one definition (which is arbitrary, because the others are defensible) or leaves the choice to a court (which reintroduces the discretion *Rucho* killed). Neither is a manageable standard.

### "Respects communities of interest"

Communities of interest are intrinsically subjective. Two reasonable map-drawers will identify different communities. A statute that mandates respecting them imports the same discretion as no statute at all.

### "Partisan-fairness metrics" (efficiency gap, mean-median, partisan bias)

These metrics measure properties of a *completed* plan against a hypothetical voting outcome. They are useful as diagnostics. But specifying "the efficiency gap shall not exceed 7 percent" both (a) freezes a particular metric whose validity scholars dispute, and (b) leaves entirely open *how* the line-drawer should achieve that result. The *Rucho* objection — no judicially manageable *standard for line-drawing* — is not solved by a metric on the output.

### Independent commissions

Commissions move the discretion from the legislature to a different body. They are an improvement, but commissions can still be captured (see Arizona's IRC after the 2020 cycle), and inter-commission outputs vary widely. A commission-only mandate is a federalism delegation, not a national standard.

### Reproducibility (this proposal)

A reproducibility-based mandate works differently. It does not specify a *property* the map must have. It specifies a *procedure* the State must execute. The procedure has exactly one output. Any person with the published inputs can verify the output is correct.

This sidesteps the *Rucho* "no manageable standard" problem entirely. The standard is not "is this map fair?" The standard is "did you run the prescribed procedure on the prescribed inputs?" That question has a yes-or-no answer that any first-year computer-science student can verify in milliseconds.

Compare 2 U.S.C. § 2a (the Huntington-Hill apportionment statute). The standard there is not "are the seats fairly distributed?" — that is intrinsically contested. The standard is "did the Commerce Secretary apply the equal-proportions method to the census numbers?" *Department of Commerce v. Montana*, 503 U.S. 442 (1992) upheld this approach against an apportionment challenge precisely because the standard is mechanical: Montana's argument was about the *choice of method*, not about whether the method was correctly applied. The Court held the choice of method is a legislative judgment Congress is entitled to make.

This proposal applies the same logic to line-drawing. Congress picks the method (recursive bisection); the State applies it; any person can verify.

## 3. Why recursive bisection is the right algorithm

A statutory algorithm needs three properties: it must be writeable, executable, and constitutionally defensible. Recursive graph bisection has all three; most other candidates have at most two.

### It is writeable

The algorithm fits in one paragraph of statutory text. Inputs: tract adjacency graph and tract populations. Method: at each level, partition the current subgraph into two parts of approximately equal population, minimizing the number of cut edges (subject to a published tolerance schedule). Repeat until the partition count equals the apportioned district count. Output: a function from tracts to district numbers.

The full specification (with parameters) lives in the parameter table maintained by the Director of Census under § 107 of the model bill. The bill text constrains the *structure* of the parameter table: structure cannot vary, only specific numerical values can.

Other candidates that fail writeability:

- **Markov-chain Monte Carlo ensembles** (the GerryChain method): the output is a *distribution* over plans, not a single plan. A statute would need to additionally specify how to pick one plan from the distribution, which reintroduces discretion. Compare R-hat / ESS / Hamming autocorrelation thresholds in `redist-analysis::ensemble_diagnostics` — useful for *evaluating* whether a sample is converged, useless for picking a single answer.
- **Optimization with multi-objective weights**: any choice of weights between objectives (compactness vs. minority-opportunity vs. partisan-fairness) is a value judgment. Specifying the weights in statute means the statute is making the judgment.
- **Heuristics with stochastic steps**: any algorithm whose output depends on a random seed produces different maps with different seeds. A statute could fix the seed (this bill does), but only deterministic algorithms can be byte-verified afterward.

### It is executable

Recursive bisection on tract-adjacency graphs runs in seconds-to-minutes per state on a laptop. The METIS library has been available since 1997. The algorithm has been used in scientific computing for circuit layout, parallel-computing partitioning, and operations research for decades. There is no novel implementation risk.

The reference implementation (this project, `redist`) has been validated against:

- All 50 states' 2020 congressional districts (`redist states --year 2020`)
- Multi-chamber suites for 12 states (`redist suite`)
- Multiple census years (2000, 2010, 2020)
- Block-group resolution for high-district-count states (CA 52D, TX 38D)

Each run produces a `manifest.json` with input SHAs, parameters, and output SHA. Re-running with the same inputs produces byte-identical outputs.

### It is constitutionally defensible

The algorithm is partisan-blind by construction (§ 104(e) of the model bill prohibits partisan inputs from even reaching the algorithm). It is race-blind in its core operation (the algorithm sees tract populations, not racial composition). VRA § 2 deviations are an explicit, bounded exception under § 106.

This structure aligns with the Court's race-conscious-redistricting doctrine. *Allen v. Milligan*, 599 U.S. 1 (2023) requires race-conscious adjustments where the *Gingles* preconditions are met. *Louisiana v. Callais* (608 U.S. ___ 2026-04-29) p.36 requires that race and partisan considerations not be confounded. The deviation-from-baseline structure does both: the bisection map is the canonical baseline; § 106 deviations are race-conscious where § 2 demands; and partisan considerations have no entry point at all.

## 4. Why states execute, not Census

A natural question — one we asked ourselves when drafting this — is whether the federal government should just draw the maps directly via the Census Bureau, given that Census already produces the inputs (TIGER + decennial population) and is best positioned to run the reference implementation.

The answer is no, for three reasons:

### 4.1 Constitutional cleanliness

Article I § 4 (the Elections Clause) authorizes Congress to "make ... regulations" prescribing the "manner" of congressional elections, and to "make or alter" State regulations on the same. The Court has repeatedly read this as authority to *constrain* state line-drawing — see *Smiley v. Holm*, 285 U.S. 355 (1932) and *Arizona v. Inter Tribal Council*, 570 U.S. 1 (2013).

Congress has never used this authority to take the *act* of line-drawing entirely from a State. 2 U.S.C. § 2c (single-member districts) and the *Wesberry* one-person-one-vote rule are constraints on State line-drawing, not displacements of it. A federal statute that has Census Bureau geographers literally drawing district maps is constitutionally novel, and would invite an Elections Clause challenge that the Court has never had occasion to decide. Cautious drafting avoids that test.

### 4.2 Local knowledge

Line-drawing within a State requires local knowledge that Census Bureau geographers do not have:

- **VRA § 2 compliance**: the *Gingles* analysis is fact-intensive and state-specific. Has voting in the state been racially polarized? Are the *Gingles* preconditions met in this region? These are state-level expert judgments. The DOJ Civil Rights Division and state attorneys general have institutional capacity for this; the Census Bureau does not.
- **Communities of interest** (where statutes preserve them): which neighborhoods are split by a highway, which counties have integrated municipal services, which tribal reservations cross county lines. State map-drawing offices know these. Census Bureau headquarters in Suitland, Maryland does not.
- **State-constitutional constraints**: many State constitutions have their own districting requirements (county integrity, town integrity, "compact and contiguous"). These vary by state and are properly left to State authorities.

The deviation-from-baseline structure (§ 106 of the model bill) lets States apply this local knowledge as a justified deviation from the federal baseline. Federalizing the act erases the room for that local input.

### 4.3 Politics

"Washington draws your districts" is the worst possible framing for a reform bill. The path to 60 Senate votes runs through Senators from small states and politically heterogeneous states; both groups will reject any bill that takes the act of line-drawing away from State authorities. By contrast, a bill that says "States still draw the lines; the algorithm is just the rules" is structurally similar to existing federal mandates (single-member districts, equal population) that all 50 States already accept.

The HR 1 / For the People Act experience confirms this: that bill bundles many reforms, but its districting provisions were the most politically toxic precisely because critics framed them as federal control. A narrow bill that preserves State execution removes that attack surface.

### 4.4 What the federal contribution actually is

The federal contribution under the model bill is not the drawing — it is the **reproducibility regime**:

- The algorithm specification (§ 104) — federal.
- The parameter table (§ 107(c)) — federal, published 18 months before each cycle.
- The reference implementation designation (§ 107(a)) — federal.
- The execution (§ 103) — State.
- The verification right (§ 105(c)) — held by every citizen, not just by the federal government.
- The VRA § 2 deviation (§ 106) — State, judicially reviewable.

This is a different shape from existing federal districting law. Pre-this-bill, the federal contribution was a thin set of constraints (single-member, equal-population, VRA § 2). Post-this-bill, the federal contribution is a dense, deterministic algorithmic specification — but the *act* of compliance still happens at the State level. The federal government's enforcement role is mechanical: did the State publish the artifacts? Are they byte-identical to the algorithm's output (modulo § 106 deviations)? Can the State justify each deviation?

## 5. Why now

This proposal would not have been viable five years ago. Two things changed.

### 5.1 The implementation matured

In 2020, no public, validated, byte-reproducible implementation of recursive-bisection congressional districting existed. Academic implementations existed (DRA, Auto-Redistrict, GerryChain) but each had limitations: GerryChain produces ensembles not single maps; DRA is interactive and not byte-reproducible; Auto-Redistrict has not been validated at the state-by-state level for all 50 states.

By 2026, this project (`redist`) has:

- Validated 50-state output for 2000 / 2010 / 2020 census years.
- Per-plan `manifest.json` with input SHA-256s and parameter records.
- AEA-compliant replication packages (`redist analyze --paper-mode`).
- Civic-input pipeline with URL snapshotting (`redist civic ingest --snapshot-urls`).
- Court-submission report scaffolding (`redist report --format pdf`).
- Plan-comparison narrative generator with provenance manifest (`redist compare --format html`).
- Ensemble diagnostics for convergence claims (`redist-analysis::ensemble_diagnostics`).
- Bloc-voting analysis for VRA § 2 (`redist analyze --types bloc-voting`).
- Atomic plan import with Callais p.36 mutex (`PlanDirGuard` + `callais_preflight`).

The algorithmic and infrastructure work that the model bill assumes has already been done. § 107(a) of the bill can designate this implementation directly; § 107(c)'s parameter table is straightforward to populate from the existing parameter set; § 105(b)'s schema is the schema this project already publishes. There is no implementation risk that "Congress mandates X but X cannot be built." X is built.

### 5.2 The post-Callais clarification

*Louisiana v. Callais* (608 U.S. ___ 2026-04-29) p.36 established that race and partisan considerations must be addressed separately, not confounded — the "disentanglement" requirement. This clarifies the model bill's § 106 structure: the algorithm baseline is race-blind and partisan-blind; VRA § 2 deviations are explicitly race-conscious and bounded; partisan considerations never enter at all.

Pre-Callais, an algorithmic mandate that was both race-blind in its core and required race-conscious VRA adjustments was harder to defend (one might argue this counted as confounding). Post-Callais, the deviation-from-baseline structure is the *correct* structure under the disentanglement rule, not a workaround.

This project's `partisan-weighted` mode (Plan 03) is mutually exclusive with `metis-vra` (VRA-aware bisection) per Callais. The model bill applies the same disentanglement: the algorithm baseline is one thing; VRA § 2 deviations are another; partisan considerations are foreclosed entirely.

### 5.3 The political window

The 2030 census is four years away. A bill enacted in the next Congress (with effective dates targeting 2031–2032 for first compliance) could be in force for the 2030 cycle. This is the natural deadline.

An enacted bill in the 2027 or 2029 Congress is achievable if framed narrowly (single issue, no other voting-rights provisions), pitched as a Huntington-Hill–style reform (technical, deterministic, eliminates discretion), and supported by a coalition that includes both parties' good-government caucuses. The hard work of the next 24 months is coalition-building around exactly that framing.

## 6. What this memo does NOT claim

In the spirit of `FAIRNESS_DOCTRINE.md` § 6, here is what this proposal does *not* claim:

1. **It does not eliminate gerrymandering.** Nothing eliminates gerrymandering, because anything race-conscious enough to comply with VRA § 2 admits some discretion, and that discretion will be used. What this bill does is *radically narrow* the discretion: from "draw any map you can defend" to "execute this one algorithm and justify any deviation from it." That is a smaller surface for gamesmanship.

2. **It does not solve state-legislative gerrymandering.** Article I § 4 reaches only congressional districts. State-legislative reform requires separate State or 14th-Amendment-grounded action.

3. **It does not solve voter suppression.** Districting is one piece of the election-administration puzzle; ID requirements, voter-roll purges, polling-place closures, and ballot-access rules are independent issues.

4. **It does not pass on its own.** No bill passes on its merits alone. This proposal needs a coalition: good-government Republicans (small-state Senators concerned about partisan map-drawing in large states), reform Democrats, civil-rights organizations (with VRA § 2 confidence built in), and academic election-law scholars who can vouch for the algorithm.

5. **It is not the only path.** State-court litigation (`FAIRNESS_DOCTRINE.md`'s domain) and state-commission reform are independent paths and should continue. This bill is additive, not substitutive.

6. **It does not require this implementation.** § 107(a) authorizes a designation process. The Director of Census plus the expert panel could designate any implementation that meets the algorithmic specification. This project (`redist`) is positioned to be that implementation, but the model bill's machinery does not depend on it.

7. **The algorithm is not "fair" in some metaphysical sense.** It is partisan-blind and race-blind by construction, and it produces the same output for the same inputs. Whether the outputs are "fair" depends on what "fair" means; the model bill does not litigate that question. It substitutes a *procedural* fairness claim (reproducibility) for a *substantive* one (which the *Rucho* Court found unavailable to federal courts).

## 7. Implementation pathway

If the model bill is to advance:

1. **Year 1: Coalition assembly.** Reform organizations (Brennan Center, Common Cause, FairVote, RepresentUs), academic election-law scholars (Stanford Election Law Project, Election Law @ Ohio State), state-level reformers (League of Women Voters), and the bar (American Bar Association's Election Law Subcommittee) develop a unified position. Senate Judiciary and House Administration staff are briefed.

2. **Year 1–2: Bill introduction and mark-up.** The bill is introduced as a freestanding measure (not bundled). Hearings cover the algorithm (with academic-expert testimony), the constitutional architecture (with election-law scholars), and the reproducibility infrastructure (with the reference-implementation maintainers).

3. **Year 2–3: Floor consideration.** A successful path requires either reconciliation (unlikely — election rules don't fit reconciliation rules) or 60 Senate votes. The 60-vote path runs through small-state Senators from both parties who can be persuaded the bill is technical, not partisan.

4. **Year 3 onward: Implementation prep.** The Director of Census convenes the expert panel, publishes the parameter table, and certifies the reference implementation. States retool their map-drawing offices to consume the parameter table.

5. **Decennial year: First compliance.** States publish reproducibility artifacts. Citizens verify. The first cycle of litigation begins (most likely on § 106 VRA-deviation justifications, which is the right place for litigation to focus).

---

## See also

- `MODEL_FEDERAL_STATUTE.md` — the bill text
- `STATUTE_ONE_PAGER.md` — 90-second version
- `FAIRNESS_DOCTRINE.md` — state-court companion strategy
- `CALLAIS_REFERENCE.md` — race-conscious-redistricting grounding
