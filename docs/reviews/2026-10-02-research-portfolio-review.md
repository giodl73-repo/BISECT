# BISECT research portfolio review

The portfolio has already explored most of the construction and search ideas
that a new reviewer would suggest. Its strongest recent contribution is the
separation of a publicly chosen immediate-cut rule from independently checked
execution. The main remaining problems are establishing the unrestricted
winner efficiently and reconciling older claims with later governed evidence.
Lookahead or final-plan optimization would change the user's intended rule;
neither is recommended here.

Review date: 2026-10-02. Source checkout: `2b90f926`, with the earlier local
counterexample tests and documentation additions present. This is a portfolio
review, not an assertion that every derivation, dataset, or PDF has been audited.

## Review coverage

The current public index contains 207 distinct paper codes across 23 tracks.
The source inventory found 206 track `main.tex` files, four publication
manuscripts, and three additional research documents: 213 document entry points
in total. Expanding their local TeX inputs found 212 abstract blocks and about
6.76 million source characters. Extraction does not mean every character was
read. The review screened the available abstracts across every track, read
selected conclusions and evidence sections, examined the quality/evidence
ledgers, and investigated the construction, search, certification and recent
experiment records in greater depth. A.5 lacks an abstract; A.1/A.2 index
entries were covered through the portfolio/index materials rather than a
matching track `main.tex`. G/U source inventories include additional entries
outside the indexed counts.

Forty-six top-level experiment READMEs exist. Their status/results were
screened, with deeper reads of national comparisons, DFS/tie/candidate
diagnostics, RI search, ensemble gates and proof frontiers. Large external
vault inputs and complete national runs were not regenerated for this review.
The prior turn's Level 1 artifact verification and focused ILP tests passed;
they do not validate all older manuscript tables.

| Tracks | Research already represented | How it informs the current rule |
|---|---|---|
| A | Synthesis, policy brief, replication and portfolio navigation | Newer A.0 distinguishes operational evidence from exact proofs; older summaries need alignment. |
| B | Recursive/n-way comparison, edge weighting, constraint conflict, complexity and seed sensitivity | Establishes the architecture's history; some broad theoretical and empirical claims require repair. |
| T | Ratio normalization, area balance, county weights, prime-factor trees, proportionality, nesting, stability, Voronoi, BFS, moving-knife, spectral, clustering, regionalization and flow | These alternatives have already been considered; many change the rule rather than merely find its winner. |
| U | Seed convergence, parameter sweeps, annealing, tempering, multiscale adaptation, ILP, Pareto, percentile choice, node-local ensembles, branch-and-cut/price, LNS, evolution, audits and certified cuts | The search menu already exists. Evidence maturity ranges from synthetic mechanics to real-state experiments. |
| G | ReCom/cross-tool comparison, diagnostics, short bursts, SMC, flips, forest/merge-split and multiscale chains | Sampling distributions and optimization are separate claims; completed replay does not establish mixing. |
| C | Resolution, cross-census validation, stability, partisan analysis, uncertainty, competition and public perceptions | Supports a robustness program, with experiment-specific coverage. C.6 explicitly remains an unfielded proposal. |
| D | Minority opportunity, threshold analysis, method comparisons, bloc voting, population adjustment and legal pathways | Geographic/demographic outputs require their own interpretation; this review did not independently revalidate current legal doctrine. |
| E | Multi-member, county-based, national, partisan-similarity, party-allocation and international alternatives | Tests representation design beyond the current single-member immediate-cut rule. |
| F | State chambers, high-seat-count resolution, nesting and jurisdiction criteria | Identifies scale and indivisible-unit limitations; congressional and legislative evidence must remain distinct. |
| I | Incumbent pairing, safe/open seats and incumbency criteria | Outcome analyses do not define the geographic selection objective. |
| J | Huntington–Hill and other divisor methods, paradoxes and implementation | Provides the policy/execution analogy and a separate arithmetic verification surface. |
| K | Perimeter, enclosing-circle, hull, elongation and population-weighted measures | Distinguishes metrics and proxies; exact Reock work corrects earlier approximations. |
| L | Efficiency gap, mean/median, bias, declination, seats/votes and representation standards | Supplies separate outcome evaluation; a geographic cut is not automatically optimal on these measures. |
| M | Economic, land-use, housing, commuting, terrain, administrative and transit signals | Community-weight alternatives already exist, but adding them changes the objective/input policy. |
| N | Population definitions, prison/student adjustment, CVAP and military treatment | The meaning of population is part of the rule and needs its own provenance. |
| O | Competition, turnout, polarization, travel distance and composite representation quality | Downstream associations require dataset and causal-evidence checks before promotion into the pitch. |
| P | Legislative, initiative, commission, court and cost pathways | Several adoption postures coexist; they should name the algorithm/profile they concern. |
| Q | Projections, reapportionment and 2030 infrastructure | Scenario inputs and seat counts need reconciliation; forecasts are not observed outcomes. |
| R | Parameter/data/geography gaming and cryptographic audit | Some claims depend on sweeps still marked pending elsewhere; hashing does not justify an input choice. |
| S | Inference, Bayesian treatment, power and multiple testing | Statistical conclusions inherit G-track sampling limitations; some formulas and abstracts disagree. |
| V | Count reconciliation, provenance and audit replay | An extensive adjacent implementation family with carefully scoped contracts; it does not prove district-cut optimality. |
| W | Forensic analytics and anomaly screens | Method frameworks for investigation, with explicit boundaries. |
| X | History, context, custody, case composition and future evidence layers | Several layers are deliberately reserved rather than implemented. |

The authoritative starting materials are the [public index](../PAPERS.md),
[evidence inventory](../vtrace/PAPER_EVIDENCE_INVENTORY.md), and
[quality ledger](../papers/PAPER-QUALITY-REVIEW.md). The evidence inventory
still reports 206 indexed rows; the current index has 207, including 22 U rows.
Internal review scores are not external acceptance or validation of datasets.

## Findings that change the next recommendation

### The national operational baseline has unfavorable geometry results

The governed 2020 [Tier 1 comparison](../experiments/nrs-v0.3-national-bakeoff-2020/README.md)
reports 1,808 split county units for NRS versus 404 for the official CD118
comparator, and 19,789 versus 4,720 split tract units. The
[Tier 2 comparison](../experiments/nrs-v0.3-national-bakeoff-geometry-2020/README.md)
reports district-weighted mean Polsby–Popper 0.025380339 versus 0.061087819,
exact Reock 0.245552262 versus 0.377238283, and convex-hull ratio 0.557042985
versus 0.702220801. Both families use the same retained block geometry.

These are descriptive block-projected measurements, not original enacted
polygon comparisons or judgments of fairness. Nevertheless, they prevent
transporting the older tract pipeline's compactness gains into an NRS v0.3
superiority claim. They do not refute exact certified bisection: NRS is an
operational candidate engine and has no unrestricted boundary proofs.

### More seeds often cannot reach the decision that matters

The [complete-tree DFS census](../experiments/nrs-v0.3-complete-tree-dfs-census-2020/README.md)
records one winning physical initial cut at every one of 385 governed 2020
nodes and no fallback activation. The [root sensitivity study](../experiments/nrs-v0.3-multistate-root-sensitivity-2020/README.md)
explains that seeded METIS affects selection only after deterministic DFS
candidates tie on population and boundary. Thus invariant results are partly
a consequence of the candidate mechanism, not proof that the entire connected
cut space has one answer. This interpretation is supported by both the
diagnostic records and `bisect-cli/src/exact_cmd.rs`.

The [five-root experiment](../experiments/multiroot-county-response-2020/README.md)
already improves population deviation in RI, IA and NC, but leaves one pooled
population-minimum physical cut per State. RI and IA boundary costs increase.
The [band experiment](../experiments/ri-population-band-2020/README.md) admits
up to four cuts without a county-weight response. The [county-aware trees](../experiments/ri-county-tree-2020/README.md)
provide no eligible cuts under the frozen bands. Recommending these experiments
as new work would repeat completed investigations.

### Substantial immediate-cut improvements already exist

The [RI proof frontier](../experiments/scalable-certified/FRONTIER-REVIEW.md)
records zero-population cleanup, equal-population swaps, one-to-two and
two-to-two exchanges, a 32-seed METIS screen, consensus cores, fixed-variable
MILP and expanded disagreement shells. The recorded connected population-floor
incumbent reaches weighted cut 43,047,238, down from 102,659,356 in the earlier
proof attempt. These are improvements under the immediate-cut model without
lookahead. Restricted-region optima do not prove unrestricted optimality.

Regional county and tract branching also exists in
`scalable-certified/regional-decomposition-frontier.json`; central branches
timed out despite substantial model reduction. Population optimality was
checked with RoundingSat/VeriPB; boundary and canonical stages remain open.
The later population-band county diagnostics are a separate experiment family:
single-block descent reaches three split counties, and adjacent swaps make
no further progress. Those outcomes cannot be pooled as if all used the same
population objective or starting assignment.

### Ensemble evidence already limits claims about typicality

The [RI block gate](../experiments/nrs-v0.3-block-ensemble-gate/README.md)
passes registered scalar diagnostics but finds materially different Wilson
and Kruskal distributions. The [NH/NM/GA expansion](../experiments/nrs-v0.3-block-ensemble-expansion-v3/README.md)
completes its primary runs and exact replays but closes non-converged because
GA fails the registered gate. These are useful negative results. They block
claims of sampler equivalence, mixing, national typicality or statistical
optimality based merely on successful execution.

## Claims requiring repair or evidence recovery

1. **B.4 empirical and theoretical claims.** Its
   [alpha generator](../../research/tracks/B-foundations/B.4+adaptive-bisection/run_alpha_ablation_simple.py)
   explicitly simulates alpha responses from assumed theory. Such a sweep
   illustrates assumptions; it cannot independently validate a phase
   transition. The active `04_theory_REVISED.tex` claims the ordered Laplacian
   eigenvalue ratio `lambda_2/lambda_3` tends to infinity, although its own
   ordering gives `lambda_2 <= lambda_3`. That is a direct mathematical
   contradiction. Its assertion that every balance-feasible algorithm becomes
   near-optimal also lacks the algorithmic premises needed for that guarantee.
   Equal district demographic summaries do not prove identical assignments.
2. **County monotonicity.** The publication manuscript's
   [proposition](../../research/publications/subdivision-respecting-redistricting/sections/03-methodology.tex)
   claims the number of split counties decreases with alpha. For a fixed
   feasible set and exact objective `A + alpha B`, the standard comparison
   establishes monotonicity of `B`, the weighted within-county boundary cost.
   It does not establish monotonicity of the number of counties split. The
   proof also reverses direction in its prose. A lower weighted length can
   occur across more counties. This needs a corrected theorem or counterexample
   study before use as a selection principle.
3. **Claims inherited from pending work.** U.2 describes a pending real
   national sweep, whereas R.1 treats its numerical national conclusions as
   established. G.1–G.3 withdrew earlier broader percentile/metric claims,
   while U.8 and some S-track narratives still cite stronger older results.
   Audit the dependencies before treating these as independent confirmations.
4. **S.2 abstract/framework mismatch.** The abstract omits the `+1` terms
   present in its uniform-prior Beta posterior. Its framework defines a
   probability of percentile extremeness; that is not a probability of
   intentional manipulation. ESS substitution is a modeling approximation,
   not automatically a calibrated likelihood for dependent samples.
5. **R.4 cryptographic arithmetic.** The abstract assigns SHA-256 a birthday
   collision work factor of `2^80`; the generic strength is 128 bits, as
   summarized by [NIST](https://csrc.nist.gov/Projects/hash-functions).
   A hash's security parameter does not directly supply a false-nondetection
   rate for a whole operational audit.
6. **Version and proposal drift.** B.02 now favors a benchmark and disclosed
   departures; the older official proposal uses ApportionRegions, county
   weights and convergence search; U.21 defines certified floor/ceiling cuts.
   Older publication counterparts retain stronger claims than updated track
   sources. The project needs a clear mapping from each claim to its exact
   rule, data vintage, evidence package and supersession status.

These findings identify repair targets; this review does not label all
unexamined empirical results synthetic or invalid. Legal claims and statistical
derivations outside the examples above require their own full audits.

## Recommendation consistent with immediate bisection

The current intended object is the single best connected split of a region,
with population deviation, boundary weight and canonical ordering fixed in
advance. Descendant outcomes must not alter that ranking. The synthetic
completion counterexamples from the earlier turn should remain tests of
failure semantics and claim limits, not a reason to replace this rule.

The next useful work is to consolidate the existing same-objective cut
experiments into one comparison: identical region/graph/population data,
fixed objective and orientation, incumbent objective, unrestricted lower
bound, search time, memory, proof size and verifier time. Distinguish
restricted-neighborhood lower bounds from unrestricted bounds. Existing
results should be recovered and compared before adding another heuristic or
rerunning a national sweep. Strengthen the most promising exact proof path
using this ledger, with timeouts retained.

Repair the B.4 and county-monotonicity claims in parallel with that evidence
consolidation. Then align the pitch around what the corpus supports:
substantial exploration of geographic rules, complete operational baselines,
measured limitations, and a proof-carrying immediate-cut architecture whose
unrestricted State-scale winner remains unproved.
