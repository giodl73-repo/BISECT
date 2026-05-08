# Panel Review — I-Incumbency Track (Batch 1)

**Date**: 2026-05-08
**Papers**: I.0–I.4 (five papers)
**Panel**: R1 Karypis (computational), R2 Rodden (political science),
R3 Duchin (math/redistricting), R4 Stephanopoulos (law), R5 Liang (ML/AI)

---

## Summary Table

| Paper | R1 | R2 | R3 | R4 | R5 | Avg | P1 count | Top P1 issue | Verdict |
|---|---|---|---|---|---|---|---|---|---|
| I.0 incumbency-overview | 3 | 2 | 2 | 3 | 2 | **2.4** | 4 | Single-seed results used to support significance claims that require ensemble | Major Revision |
| I.1 incumbent-pairing | 2 | 2 | 2 | 3 | 2 | **2.2** | 5 | Binomial test independence assumption violated; pairs are not i.i.d. | Major Revision |
| I.2 safe-seat-creation | 3 | 2 | 3 | 3 | 3 | **2.8** | 3 | Single-seed p-values treated as confirmatory without ensemble support | Minor Revision |
| I.3 open-seat-effects | 3 | 2 | 2 | 3 | 2 | **2.4** | 4 | Retirement incentive model parameters asserted without accessible calibration data | Major Revision |
| I.4 incumbency-legal-criterion | 3 | 3 | 3 | 3 | 3 | **3.0** | 1 | Burdick v. Takushi application to redistricting overstated as accepted doctrine | Accept |

**Scale**: 0 = Reject, 1 = Major problems, 2 = Significant revision needed, 3 = Minor revision, 4 = Accept as-is.
**Verdict thresholds**: Accept ≥3.0 | Minor Revision ≥2.5 | Major Revision ≥2.0 | Reject <2.0.

---

## Detailed Per-Paper Reviews

---

### I.0 — Incumbency Overview

**Avg: 2.4 | Verdict: Major Revision**

#### R1 — Karypis (computational): 3/4

The synthesis correctly frames the three metrics and the incumbency-neutral baseline.
The `E[pairs] = C(n,2)/k` formula is correct under uniform random assignment, and the
Poisson approximation for open seats (`k·(1-1/k)^n`) is appropriate. The geographic
adjustment factors (ρ_NC=0.85, ρ_WI=0.70, ρ_TX=0.78) are described but not derived in
this paper — they are deferred to I.1. That is acceptable for an overview paper.

**P1**: The table (Table 1) reports statistical significance for the safe-seat fraction
comparison ("statistically significant at conventional levels, p < 0.01 for NC and TX
under a two-sample test of proportions") but these p-values are computed against a
single-seed bisect result. A two-sample test of proportions treats each district as an
independent Bernoulli trial, but the districts are draws from a single constrained plan —
they are correlated by the partitioning structure. The p-values reported are the p-values
of the I.2 proportions test applied to a single seed; the overview paper presents them
as though they are established findings without noting that they depend on a specific
seed and could differ substantially with ensemble variance. This should be flagged with
an asterisk at minimum, and the overview should defer significance claims to I.2 or note
their seed-dependence explicitly.

**Minor**: The description of the random baseline range for open seats as "1.0–2.5" for
NC and "4.6–6.1" for TX (Table 1) does not match I.3's reported 95% CI of [0,3] for
NC and [1,8] for TX. These ranges should be consistent across the two papers.

#### R2 — Rodden (political science): 2/4

The core descriptive claim — that enacted maps achieve near-zero pairing while algorithmic
maps are closer to the random baseline — is plausible and interesting. The framing is
compelling. However:

**P1**: Single-seed dependence is endemic throughout. The abstract says bisect maps "fall
within this baseline range on all three metrics across all three states." But this is true
for one seed. With a different seed, bisect might produce a different pairing event or a
different competitive-seat count. The reader is not given the seed variance, so there is
no way to assess whether "within the baseline range" is a general property of the
algorithm or an artifact of seed 42. An overview paper synthesising I.1–I.4 should not
inherit single-seed results uncritically.

**P1**: The comparison to enacted maps is presented as if enacted maps have a single
fixed value, but there is no uncertainty interval for the enacted map either (correctly
so — it is a fixed plan). The framing should clarify that the bisect number is a
single-seed point estimate, while the baseline is a distributional expectation. Saying
"bisect outcomes fall within this baseline range" for a single point estimate vs. a
distributional range is statistically informal.

**Minor**: "These differences are statistically significant at conventional levels (I.2
reports p < 0.01 for NC and TX)" — the cross-referencing of significance from companion
papers is appropriate, but the reader should be told that these p-values come from
single-seed bisect results.

#### R3 — Duchin (math/redistricting): 2/4

**P1**: The safe-seat threshold of ±15pp (|v_D − 0.5| > 0.15) is used throughout the
overview without justification. It appears here in Section 3.2 and is called
"conventional in the political science literature on electoral competition." This is
partly true (Fiorina and others use similar thresholds), but 15pp is not universal —
some literature uses 10pp, others 20pp. The threshold choice materially affects the
safe-seat fraction (a district at 35.1% Democratic is safe under 15pp but competitive
under 10pp). The overview should either justify 15pp by reference to its calibration
properties or report sensitivity to alternative thresholds.

**P1**: The "incumbency-neutral baseline" for safe-seat fraction is described as "not
reported here because it depends on geographic partisan sorting, not on a simple
combinatorial formula." This is correct, but it means the overview table has an
asymmetric comparison: for pairing rate and open-seat count, a random baseline is
provided, but for safe-seat fraction, there is none. The comparison between bisect
(0.43) and enacted (0.64) is thus presented without a reference point. The reader does
not know whether 0.43 is itself anomalously high or low for compact geographic
redistricting in NC. This gap should be addressed by at least citing what I.2 reports
as the geographic-constraint-adjusted safe-seat baseline.

**Minor**: The pairing rate formula uses `|{(i,j): d(i)=d(j)}| / C(n,2)` — this
double-counts pairs because the set is over unordered pairs but the notation is
ambiguous. I.1 defines this more carefully. The overview should use consistent notation.

#### R4 — Stephanopoulos (law): 3/4

The legal framing is generally sound. The treatment of Karcher, Cox v. Larios, Burdick
v. Takushi, and Thornburg v. Gingles is accurate. The key holding of each case is
correctly characterised.

**P1**: The description of Burdick v. Takushi in Section 2 says its "articulation of
the state interest in 'election administration' stability has been invoked in redistricting
proceedings to justify incumbent protection." This is a characterisation of lower-court
use of Burdick, not a statement of the Supreme Court's own holding. Burdick addressed
ballot access, not redistricting. The paper should be more careful to say "lower courts
have invoked Burdick's framework" rather than implying the Court endorsed this extension.
The I.4 paper is more careful on this point; the overview should match I.4's precision.

**Minor**: Harper v. Hall citation as 868 S.E.2d 499 (N.C. 2022) is correct for the
2022 decision. Note that this decision was subsequently modified by the reinstated
conservative majority — the paper should note the procedural history briefly so readers
understand the legal status.

#### R5 — Liang (ML/AI): 2/4

**P1**: FEC address data availability. The paper states that "incumbent addresses" are
drawn from "Federal Election Commission (FEC) 2022 candidate filings, from which home
addresses for sitting incumbents were geocoded." FEC Form 2 disclosures do include a
home address field, but the FEC does not always make home addresses fully available in
bulk download format — some addresses are redacted or listed as the campaign office.
The paper does not address whether the addresses used are home addresses (from the
candidate's personal disclosure) or campaign/committee addresses. This is material: if
some incumbents' FEC filings list a campaign address rather than a home address, the
geocoding will produce an incorrect home census tract. The limitation is noted vaguely
("a small number of incumbents may have moved") but the more fundamental question of
address type is not addressed.

**P1**: No reference to the FEC data URL or specific form type. "FEC 2022 candidate
filings" is not specific enough for reproducibility. The reader needs: which FEC form
(Form 2 = Statement of Candidacy), which data file, what vintage (2022 cycle filing
cutoff date), and whether the addresses were the home addresses or the committee
addresses. Without this, the geocoding step is not reproducible.

**Minor**: "Census Bureau TIGER/Line geocoder" is referenced but the specific geocoder
endpoint (batch vs. interactive, single-address vs. batch) is not stated. The batch
geocoder URL is in the references (censusGeocoder), which partially addresses this.

---

### I.1 — Incumbent Pairing

**Avg: 2.2 | Verdict: Major Revision**

#### R1 — Karypis (computational): 2/4

The derivation of the analytical baseline is mathematically correct under the stated
assumptions. The formula `E[C] = C(n,2)/k` is proven correctly. The variance formula
`Var[C] = C(n,2)·(1/k)·(1-1/k) + 2·C(n,3)·(1/k²)` is correct for the i.i.d. uniform
assignment model (the second term captures pairwise covariance among overlapping pairs).

**P1 (critical)**: The binomial test in Section 3.2 is misapplied. The test treats each
pair of incumbents as an independent Bernoulli trial with success probability p_0 = ρ/k.
But the pairing events are not independent: if incumbents i_a and i_b are both assigned
to district d, and incumbents i_a and i_c are also both assigned to district d, then the
pair (i_a, i_b) and the pair (i_a, i_c) are correlated — knowing one pair is in d
increases the probability that the other pair is also in d. The variance formula in
Section 2.2 correctly accounts for this covariance with the `2·C(n,3)/k²` term, but
the binomial test ignores it entirely by treating pairing events as i.i.d. Bernoulli.
The p-values reported (p<0.001 for NC, p=0.014 for WI, p=0.033 for TX) are based on
an incorrect independence assumption and will be too small (anti-conservative) when
pairs are positively correlated. The test should use the actual variance of C (which the
paper derives) to construct a normal approximation or use a permutation test. As written,
the significance claims are overstated.

**P1**: The pairing count for NC under bisect is reported as "11" pairs from "two
geographic pairings" (two incumbents form one paired district). But C(2,2) = 1 pair,
not 11. If two incumbents are paired, they contribute exactly 1 paired pair, giving
pairing count = 1 and pairing rate = 1/C(13,2) = 1/78 ≈ 0.013, not 0.143. The rate
of 0.143 would require 11 paired pairs, which would need at least 4–5 incumbents all
in the same district. The text contradicts itself: it says "two geographic pairings
occur" (which would give count=1 or count=2 depending on interpretation) but the table
shows count=11 and rate=0.143. This is an internal inconsistency that must be resolved.

**Minor**: Table 2 reports TX pairing count as 67 with $\binom{36}{2}=630$ pairs,
giving rate ≈ 0.106, consistent with 0.105 rounded. But E[C]_geo is reported as 99.9,
which would be a geometric adjustment of 99.9/630 ≈ 0.159 — but the text says the
adjusted expected rate for TX is 0.207 (Section 2.3). 0.207 × 630 = 130.4 pairs
expected, not 99.9. The numbers in the table are internally inconsistent.

#### R2 — Rodden (political science): 2/4

**P1**: The paper claims the enacted NC map's zero pairing rate is "extraordinary
unlikely under random redistricting" with p < 0.001. But this p-value is computed under
the i.i.d. Bernoulli assumption, which the paper itself notes overstates expected pairing
rate relative to the geographic reality. More importantly, p < 0.001 for a one-tailed
test of "zero pairings" where the baseline is only E[C]=7.14 pairs is a very strong
claim. If the test is poorly calibrated (as R1 notes), this significance claim may be
unreliable. The paper should provide a permutation-based p-value where the null
distribution is the distribution of pairing counts over an ensemble of bisect runs, not
an analytical binomial.

**P1**: The geographic adjustment factors (ρ_NC=0.85, etc.) are stated without derivation.
The paper says these are computed by "constructing the set of all contiguous-district
plans consistent with the METIS adjacency graph structure" — but this set is astronomically
large and cannot be enumerated. The actual computation must be via Monte Carlo sampling
of the METIS adjacency graph. The methodology for computing ρ is not described in
sufficient detail to be reproducible, and there are no standard errors on ρ estimates.
If ρ is estimated from a small Monte Carlo sample, its uncertainty propagates into the
geographically adjusted baseline and the p-values.

**Minor**: "Members who announced retirement before redistricting was completed are
excluded" — this exclusion rule is sensible but the specific members excluded are not
identified. For TX, two members are excluded; for NC, one. Which members? If they are
excluded because their announced retirement influenced the redistricting (mapmakers
knew they were retiring and drew their districts differently), the exclusion could
bias the analysis.

#### R3 — Duchin (math/redistricting): 2/4

**P1**: The analysis is built on a single bisect run (seed=42). For a result to be
publishable as "bisect produces pairing rates within the random baseline," the paper
needs to show this is true across the full ensemble of bisect runs, not just one seed.
The paper acknowledges this in the conclusion ("Future work should extend this analysis
to multi-seed ensemble runs") but presents the single-seed result as though it supports
the main claim. It does not. A single run that happens to fall within the baseline range
is not evidence that the algorithm as a whole produces baseline-consistent results —
especially given that the bisect algorithm is deterministic for a fixed seed and uses
a METIS optimiser that produces structured (not random) partitions.

**P1**: The internal counting inconsistency (R1's P1 about NC pairing count=11 vs. "two
geographic pairings") represents a fundamental problem: the paper's empirical results
are unreliable if the basic counts do not add up. This must be corrected before any
significance testing can proceed.

**Minor**: The definition of pairing rate uses C(P)/C(n,2) where C(P) is the count of
unordered pairs. This is correct, but the "pairing rate" of a plan with all incumbents
in the same district would be 1.0 under this formula, not some maximum less than 1.
The interpretation is clean.

#### R4 — Stephanopoulos (law): 3/4

The legal section (Section 5) is accurate and appropriate. The three propositions
(pairing is not constitutionally prohibited; protection is permissible but not required;
an algorithm ignoring incumbency is sound) are correctly stated with proper citations.

**P1**: The paper says "no constitutional provision prevents a redistricting plan from
placing two incumbents in the same district; the First Amendment and the Equal Protection
Clause protect voters, not the electoral interests of sitting legislators." This is
correct, but the citation to Rucho v. Common Cause for this proposition is imprecise.
Rucho held that partisan gerrymandering claims are non-justiciable in federal court; it
did not directly address whether the Constitution protects the electoral interests of
incumbents per se. The correct citation for the voter-protection principle is the line
of cases from Reynolds v. Sims and its progeny. Rucho is relevant but should not be
the lead citation for this proposition.

**Minor**: The Karcher dicta passage is correctly quoted in Section 5.2, but the paper
could clarify that this language appeared in the context of explaining what justifications
would suffice for deviations — i.e., it is obiter dicta, not a holding. I.4 does this
more carefully.

#### R5 — Liang (ML/AI): 2/4

**P1**: The FEC geocoding methodology is described but not fully reproducible. The paper
states: "FEC Form 2 candidate filings for the 2022 election cycle report the candidate's
home address. We geocode these addresses to 2020 Census TIGER/Line census tracts using
the Census Bureau's online batch geocoder." But FEC Form 2 is the Statement of Candidacy,
which reports a mailing/committee address, not a home address. The home address is on
FEC Form 1 (Statement of Organization) for the campaign committee, or may be found in
state disclosure filings. The distinction matters: if the paper has used committee office
addresses rather than residential addresses, the geocoding is incorrect for incumbents
who do not live near their campaign office. The paper should state explicitly which FEC
form and field was used, and whether any manual verification of home vs. office addresses
was performed.

**P1**: The bisect command shown (`bisect build <state>_2022_test --year 2020 --structure
prime-factor --weights-override geographic --workers 8`) does not include a `--seed 42`
flag. If seed is a run-time argument that defaults to something else, or if the seed is
set by a config file, reproducibility requires knowing the exact invocation including
seed. Without the seed in the code block, a reader who runs this command will get a
different result.

**Minor**: The Census Bureau batch geocoder is cited (censusGeocoder), which is publicly
available and accessible — this is good for reproducibility. However, the vintage of the
TIGER/Line tract file used for geocoding should match the population data vintage (2020).
Is this guaranteed by the geocoder, or does the user select it?

---

### I.2 — Safe-Seat Creation

**Avg: 2.8 | Verdict: Minor Revision**

#### R1 — Karypis (computational): 3/4

The methodology is the cleanest of the five papers. The safe-seat fraction is computed
from a well-defined formula, and the area-weighted spatial join methodology is described.
The two-sample proportion test is the appropriate test for comparing fractions across
plans.

**P1**: The two-sample proportion test is applied with sample size k (the number of
districts) for each plan. But the districts in each plan are not independent Bernoulli
trials — they are outputs of a constrained partition of a contiguous graph. Two adjacent
districts tend to have more similar partisan lean than two non-adjacent districts, inducing
positive spatial autocorrelation in the safe-seat indicators. The reported z-statistics
and p-values do not account for this autocorrelation. For NC (k=14), the effective sample
size may be substantially less than 14, making the p=0.008 result less reliable than
stated.

**Minor**: The table has a notation error for WI enacted: "0.63 (rounded 0.50)" — the
table note says "WI enacted fraction corrected: 4/8 = 0.50" but the column entry says
0.63. This should be 0.50 in the column, not 0.63. This is a copy-paste error.

#### R2 — Rodden (political science): 2/4

**P1**: The paper claims that "The enacted NC map produces zero competitive districts"
while the bisect map produces 3. This is presented as evidence of deliberate packing,
which is a strong claim. However, the finding depends entirely on seed 42. If bisect with
a different seed produced 0–1 competitive districts (plausible given the variance in
compact geographic redistricting), the contrast would evaporate. The safe-seat comparison
needs ensemble support before it can bear the inferential weight the paper places on it.

**P1**: The discussion section says that "safe-Democratic seats are largely preserved by
geographic compactness... urban Democratic voter concentrations are sufficiently dense that
any compact district centred on them will produce a safe-D seat." This is the Rodden
(2019) argument about geographic sorting. But the claim that compact redistricting
necessarily preserves safe-D seats is not universally true — it depends on whether
the urban concentration is above or below the 65% threshold used here. The paper should
test this claim more carefully rather than asserting it as a mechanism.

**Minor**: The paper correctly cites Fiorina (1980) for the claim that safe-seat incumbents
are less responsive to constituent opinion. However, the Fiorina citation is
"fiorina1980incumbency" which the bib file for I.2 does not appear to contain (it is in
I.1's bib as well). Ensure the bib file for I.2 includes this reference.

#### R3 — Duchin (math/redistricting): 3/4

The safe-seat threshold definition (|v_D - 0.5| > 0.15) is clearly stated and applied
consistently. The geographic sources of safe seats (sorting vs. deliberate packing) are
correctly distinguished.

**P1**: The WI table entry notation error (0.63 vs. 0.50) is not merely cosmetic —
the z-statistic and p-value for WI (z=1.46, p=0.072) are based on WI's enacted fraction.
If the correct fraction is 0.50, the z-statistic should be recalculated. (0.38 vs 0.50,
k=8: z ≈ 0.80, p ≈ 0.21 — not significant even directionally.) The p-value reported
may be based on the wrong enacted fraction.

**Minor**: The "competitive" category is defined inconsistently: the definition in the
background section uses |v_D - 0.50| ≤ 0.15 for safety, implying competitive is
|v_D - 0.50| ≤ 0.15, but the methodology section redefines competitive as |v_D - 0.50|
≤ 0.05 (within 5pp). These two definitions are inconsistent. A district with v_D = 0.57
is safe-D under the strict 5pp competitive definition but lean-D under the 15pp safety
threshold. The paper uses both cutoffs (15pp for safe vs. not-safe, 5pp for competitive
within the not-safe range) but the interplay is not clearly explained in the background.

#### R4 — Stephanopoulos (law): 3/4

The legal section is sound. The citations to League of Women Voters v. Commonwealth and
Harper v. Hall for the free elections doctrine are accurate. The argument that bisect's
more competitive maps provide the counterfactual courts need is well-grounded.

**Minor**: The paper notes that Harper v. Hall "held that redistricting plans that
systematically reduce electoral competition violate state constitutional free-elections
guarantees." This is an accurate characterisation of the 2022 Harper decision. But as
noted above, that decision was subsequently revisited — practitioners using this case
need to know its current precedential status in NC courts.

#### R5 — Liang (ML/AI): 3/4

The data pipeline for this paper is cleaner than I.1. The area-weighted spatial join
methodology is described in sufficient detail (proportionate to precinct area within
tract), and the error bounds (0.5pp for 95% of NC/WI tracts, 1-2pp for some TX tracts)
give the reader a realistic sense of data quality.

**Minor**: The VEST 2020 data citation is present and links to Harvard Dataverse, which
is a stable, publicly accessible archive. However, it would strengthen reproducibility to
cite the specific DOI for the VEST 2020 release rather than the top-level dataverse page,
since the dataverse contains multiple versions of VEST data.

---

### I.3 — Open-Seat Effects

**Avg: 2.4 | Verdict: Major Revision**

#### R1 — Karypis (computational): 3/4

The analytical baseline for open seats (`E[open] = k·(1-1/k)^n`) is correct. The
Poisson approximation for large k is appropriate. The 95% CI derivation ("from the exact
distribution of the coupon collector analogue") is not shown explicitly but is standard.

**P1**: The retirement incentive (RI) model is:
`RI(i,P) = α·1[paired] + β·|Δv_D|`
with α=0.40 and β=0.50, "calibrated against historical post-redistricting retirement
rates." But the calibration dataset is not described. Ansolabehere and Snyder (2000) is
cited, but that paper studies the decline of competition, not individual retirement
probabilities as a function of pairing and lean-change. There is no identification of
which redistricting cycles were used for calibration, what the sample size was, whether
the coefficients are OLS estimates or something else, or what the model's in-sample fit
was. Without this, the RI scores in Table 3 are arbitrary. A mean RI of 0.38 (NC bisect)
vs. 0.12 (NC enacted) could mean anything depending on whether α=0.40 and β=0.50 are
appropriate for 2022-cycle NC incumbents specifically.

**Minor**: The geographic adjustment for open seats increases the expected count
(NC: 1.04 → 1.5; WI: 0.65 → 1.1; TX: 3.97 → 5.2) by a factor of ~1.3-1.4. This
direction is opposite to the geographic adjustment for pairing (which reduces expected
pairing from the uniform model). The explanation is that "peripheral incumbents are
more likely to be isolated in districts that capture them away from other incumbents."
This is plausible but the mechanism is not fully elaborated. The geographic adjustment
for open seats should be derived from the same Monte Carlo sampling used for the pairing
geographic adjustment, and the two should be consistent.

#### R2 — Rodden (political science): 2/4

**P1**: The comparison to Colorado's independent redistricting commission (3 open seats
out of 8 districts = 0.375 fraction) is presented as validation that "algorithmic
redistricting produces incumbency consequences similar to what a neutral human process
would produce." But this comparison is at best suggestive — one state with one commission
cycle is not sufficient to establish that claim. Colorado's geography, delegation size,
and incumbent distribution are very different from NC, WI, and TX. The paper should
acknowledge the comparison as illustrative rather than confirmatory.

**P1**: The NC court-drawn remedial map is cited as producing "approximately 3 open seats
and a safe-seat fraction of approximately 0.50." The hedge word "approximately" appears
twice in the same sentence, and no citation is given for these figures. The court-drawn
NC map is a specific, publicly available document — the paper should cite it precisely
and report the actual counts rather than approximations.

**Minor**: The broad open-seat definition (competitive district + one incumbent) is
interesting but produces a large count (11 broad open seats in TX under bisect) that
may overwhelm the strict definition in the summary. The reader is left unclear whether
the paper's main claim rests on strict or broad open seats. The abstract mentions only
"4 open seats in NC" (strict), while the discussion leans heavily on the broad count.

#### R3 — Duchin (math/redistricting): 2/4

**P1**: The open-seat baseline table reports a 95% CI of [0,3] for NC, but the bisect
result of 4 strict open seats is above this interval. The paper says "The bisect strict
count of 4 is above the 95% confidence interval of [0,3]" and attributes this to the
pairing effect (pairings create open districts). But if the bisect result is outside the
95% CI of the claimed baseline, the result is not "within or slightly above the random
baseline range" as the abstract claims — it is a statistically unusual outcome under
the baseline. The paper needs to reconcile this: either the CI is miscalculated, or the
bisect result is genuinely unusual and the framing should change.

**P1**: The probability formula for open seats is stated as:
`Pr[district d is open] = ((k-1)/k)^n = (1-1/k)^n`
but the notation `C(k-1,1)^n / C(k,1)^n` is incorrect combinatorially — C(k-1,1) = k-1
and C(k,1) = k, so the ratio is ((k-1)/k)^n, which equals (1-1/k)^n, but the expression
`C(k-1,1)^n / C(k,1)^n` is not a standard combinatorial expression and the notation is
misleading. The simpler `(1-1/k)^n` should be used directly.

**Minor**: The confidence intervals are described as "from the exact distribution of the
coupon collector analogue" but the open-seat problem is not exactly the coupon collector
problem (which asks how many draws are needed to collect all k coupons, not how many
coupons are uncollected after n draws). The correct analogue is the "occupancy problem"
or the distribution of the number of empty bins after n balls are thrown into k bins.
The 95% CIs should be derived from the binomial or Poisson-binomial distribution, not
the coupon collector.

#### R4 — Stephanopoulos (law): 3/4

The legal sections of I.3 are sound. The point that competitive elections are a
by-product of compactness (not a targeted goal of the algorithm) is legally important and
well-articulated.

**Minor**: The paper says "Proposals to mandate competitive districts... are
constitutionally contested: some state courts have accepted competitiveness as a criterion,
others have rejected it." This is accurate but a citation would strengthen it. The NC
Harper decision and the Pennsylvania LWV decision are both candidates.

#### R5 — Liang (ML/AI): 2/4

**P1**: The retirement incentive model is not reproducible in its current form. The
parameters α=0.40 and β=0.50 are described as calibrated against "historical
post-redistricting retirement rates in cycles where pairing and lean-change data are
available (Ansolabehere and Snyder 2000)." But Ansolabehere and Snyder (2000) is a
general incumbency/competition paper; it does not provide individual-level retirement
probabilities as functions of pairing and lean-change that could support this
calibration. A researcher who wanted to reproduce or update the calibration cannot do
so from the paper's description. The model should either: (a) derive α and β from a
documented regression on historical retirement data with a specific dataset and method,
or (b) present the RI scores as illustrative sensitivity analyses by reporting results
across a range of (α, β) values.

**P1**: The threshold "RI ≥ 0.30 based on calibration against the 2002 and 2012
post-redistricting cycles" introduces another undocumented calibration step. What was the
historical retirement rate in 2002 and 2012? What data source? The 0.30 threshold is
presented as if it is an empirically calibrated result, but it cannot be verified.

---

### I.4 — Incumbency Legal Criterion

**Avg: 3.0 | Verdict: Accept**

#### R1 — Karypis (computational): 3/4

The legal paper correctly situates the algorithmic indifference standard within the
existing doctrine. The three-prong definition of the algorithmic indifference standard
is precise and testable. The observation that Karcher/Cox apply to population deviations
that bisect does not produce (making the Karcher objection moot at the threshold) is
analytically clean.

**Minor**: The paper references the `--weights-override vra-aligned` flag for VRA-
constrained runs. This is described in the bisect CLI documentation, which is a software
document that could change. It would be preferable to describe the VRA-alignment
capability in general terms rather than by specific CLI flag, to avoid the paper becoming
outdated if the CLI changes.

#### R2 — Rodden (political science): 3/4

The empirical grounding of the legal analysis (cross-referencing I.1–I.3 for the
statistical evidence) is well-structured. The paper correctly avoids overclaiming:
it does not say bisect maps produce no incumbency effects — it says the effects are
indistinguishable from the geographic random baseline.

**P1**: The paper relies on I.1–I.3 for its empirical claims, but as the reviews of
those papers show, the empirical foundation is weaker than presented (single-seed,
binomial test independence assumption violated, internal counting inconsistency). I.4
should include a caveat that the empirical claims in I.1–I.3 are based on single-seed
results and pending ensemble validation. As it stands, I.4 presents the empirical
foundation as established when it is preliminary.

#### R3 — Duchin (math/redistricting): 3/4

The algorithmic indifference standard is well-defined. Prong 3 (incumbency outcomes
statistically consistent with the geographic random baseline) is the correct test, and
the paper correctly notes that I.1–I.3 provide evidence for this.

**Minor**: The prong 3 test depends on the quality of the statistical testing in I.1–I.3,
which the panel finds problematic. I.4 should acknowledge this dependency and note that
the standard is designed to be tested by ensemble evidence, with I.1–I.3 providing
preliminary single-seed support.

#### R4 — Stephanopoulos (law): 3/4

This is the strongest paper in the track. The legal analysis is accurate throughout.

- Karcher holding: correctly characterised (legitimate objective, not mandatory; dicta).
- Cox v. Larios holding: correctly characterised (incumbency cannot justify large
  deviations; summary affirmance of N.D. Ga.).
- Burdick v. Takushi: correctly identified as a write-in voting case and the extension
  to redistricting is correctly labelled as strained lower-court reasoning, not a Supreme
  Court endorsement.
- Thornburg v. Gingles: correctly characterised (three-part test; minority group
  opportunity, not specific incumbent protection).
- Rucho v. Common Cause: correctly characterised (federal non-justiciability).
- Harper v. Hall: correctly cited and characterised.

**P1**: Minor issue only. The decision table (Table 3) includes the row "Algorithm
produces unsafe seats | VRA requires minority opportunity, not safe seats." This is
correct as a legal matter, but the table entry says "VRA compliance is evaluated ex post;
majority-minority districts preserved where required." The VRA does not require
"majority-minority districts" in all cases — it requires minority voters to have an equal
opportunity to elect preferred candidates. A majority-minority district (BVAP > 50%) is
one sufficient condition but not the only one recognised under the Gingles framework.
The table simplifies this in a way that could mislead practitioners. Use "adequate
minority opportunity districts" rather than "majority-minority districts."

#### R5 — Liang (ML/AI): 3/4

The legal paper's reproducibility concerns are minimal since it is primarily doctrinal.
The software citation (bisect2024, URL: github.com/gdl/bisect) is present. The VEST and
TIGER/Line data citations are in the bib file.

**Minor**: The GitHub URL in bisect2024 (`https://github.com/gdl/bisect`) appears to be
a placeholder — if this is a private repository or a different URL, the citation will
fail. Verify the actual repository URL before submission.

---

## Cross-Paper Issues

### Issue 1: Internal quantitative inconsistency — I.1 NC pairing count

I.0 Table 1 reports NC bisect pairing rate = 0.143.
I.1 Table 2 reports NC bisect pairing count = 11, rate = 0.143.
I.1 Section 4.1 says "two geographic pairings occur."

If two pairings occur (2 incumbents paired together = 1 paired pair, or 2 separate
pairs = 2 paired pairs), the pairing count should be 1 or 2, not 11. Pairing rate 11/78
= 0.141 ≈ 0.143, which would require 11 paired pairs. This would require a much larger
set of incumbents to be paired. The text and the count are irreconcilable as written.
This must be resolved before any paper in the track can be accepted.

### Issue 2: Binomial test independence assumption (I.1, I.0)

The binomial test used to establish statistical significance of pairing-rate comparisons
assumes i.i.d. Bernoulli trials, but pairs of incumbents are not independent (shared
incumbents create correlated pairs). The variance formula in I.1 Section 2.2 correctly
accounts for this, but the test statistic does not. All p-values from this test are
suspect and should be replaced with permutation-based tests or normal approximation using
the correct variance.

### Issue 3: Single-seed results throughout

All five papers prominently feature single-seed (seed=42) bisect results as the primary
empirical evidence. Papers I.0, I.1, I.2, I.3 use single-seed results to support
statistical claims about the algorithm's typical behaviour. A single deterministic run
of METIS is not a random sample from any distribution; it is a specific, reproducible
outcome that may or may not be representative of the algorithm's typical outputs. Until
ensemble results are available, all significance claims about "bisect falls within the
baseline" are claims about one specific run, not about the algorithm in general.
Across-the-board revision is needed to clearly separate single-seed observations from
general algorithmic properties.

### Issue 4: FEC address data clarity (I.0, I.1, I.3)

All three empirical papers reference "FEC 2022 candidate filings" for incumbent home
addresses without specifying which FEC form, which field, or whether home addresses (as
distinct from campaign/committee addresses) were used. FEC Form 2 reports a committee
address; home addresses for individual candidates may not be systematically available
in bulk FEC data. This should be resolved across all papers with a unified data
appendix describing the exact source and any manual verification steps.

### Issue 5: WI safe-seat fraction notation error (I.2)

Table 1 of I.2 shows WI enacted safe-seat fraction as "0.63 (rounded 0.50)" in the
data column, with a table note correcting to 4/8 = 0.50. The column should show 0.50
(or 0.63 and note it as uncorrected). The z-statistic and p-value for WI may need
recalculation based on the correct fraction.

---

## Priority-1 Issues Requiring Fix Before Re-Review

| Issue | Papers | Fix required |
|---|---|---|
| I.1 NC pairing count=11 inconsistent with "two geographic pairings" | I.1, I.0 | Resolve counting; restate pairing rate consistently |
| Binomial test independence assumption violated | I.1, I.0 | Replace with permutation test or variance-corrected normal approximation |
| Single-seed results presented as confirmatory statistical claims | I.0, I.1, I.2, I.3 | Label as preliminary; defer confirmation to ensemble; revise abstract/conclusion claims |
| FEC address data — form and field unspecified | I.0, I.1, I.3 | Add unified data appendix specifying FEC Form 2 vs. other, field used, any manual verification |
| WI enacted safe-seat fraction notation error | I.2 | Correct table; recalculate WI z-statistic and p-value |
| RI model calibration undocumented | I.3 | Either document regression or present as sensitivity analysis |
| TX E[C]_geo inconsistency (99.9 in table vs. 207 implied by 0.207×630) | I.1 | Resolve; verify geographic adjustment calculation for TX |
| Burdick application to redistricting should be labelled as lower-court extension, not Supreme Court doctrine | I.0 | Add qualifier; align with I.4's more careful treatment |
| Bisect seed not shown in methodology code block | I.1, I.2, I.3 | Add `--seed 42` to all command examples |

---

## Recommendations by Paper

**I.0 (overview)**: Major Revision. Core synthesis is valuable but inherits the P1 issues
from I.1–I.3 without adequate caveats. Revised version should clearly distinguish
single-seed observations from algorithmic properties, and resolve the cross-paper
quantitative inconsistencies before claiming the overview findings are established.

**I.1 (pairing)**: Major Revision. The mathematical framework is largely correct and the
research question is important. Two critical issues must be fixed: (a) the NC pairing
count inconsistency (count=11 vs. "two geographic pairings"), and (b) the binomial test
independence assumption. Without these fixes, the p-values are unreliable and the
empirical results cannot be trusted.

**I.2 (safe seats)**: Minor Revision. The cleanest empirical paper. Fix the WI fraction
notation error, recalculate WI statistics, add the spatial autocorrelation caveat to the
proportion test, clarify the 15pp vs. 5pp threshold interplay, and add ensemble caveats.
This paper is close to submission-ready after these fixes.

**I.3 (open seats)**: Major Revision. The RI model is undocumented and the open-seat
count for NC (4) is outside the stated 95% CI ([0,3]) while being presented as "within
or slightly above" the baseline. The Colorado and NC court-map comparisons need
proper citation. The combinatorial notation error (coupon collector vs. occupancy) should
be corrected.

**I.4 (legal)**: Accept with minor revision. The best paper in the track. The legal
analysis is careful, the case holdings are correctly characterised, and the algorithmic
indifference standard is a genuine contribution. Minor fix: replace "majority-minority
districts" with "minority opportunity districts" in the decision table; add caveat that
empirical foundation from I.1–I.3 is preliminary pending ensemble validation.
