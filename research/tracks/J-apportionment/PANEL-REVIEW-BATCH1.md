# J-Track Panel Review — Batch 1
**Papers**: J.0–J.6 (Apportionment Methods)
**Date**: 2026-05-08
**Panel**: R1 Karypis (algorithms/math), R2 Rodden (political science),
R3 Duchin (math), R4 Stephanopoulos (law), R5 Liang (systems)

---

## Summary Table

| Paper | Title | R1 | R2 | R3 | R4 | R5 | Avg | Verdict | P1 count | Top P1 issue |
|-------|-------|----|----|----|----|----|----|---------|----------|--------------|
| J.0 | Apportionment Overview | 3 | 3 | 3 | 3 | 2 | 2.8 | Minor Revision | 1 | R5: `apportion()` API described does not exist in actual code |
| J.1 | Huntington-Hill | 3 | 3 | 3 | 4 | 1 | 2.8 | Minor Revision | 2 | R5: HH crate uses f64 throughout; paper claims exact u128 arithmetic |
| J.2 | Webster Method | 3 | 3 | 3 | 3 | 2 | 2.8 | Minor Revision | 1 | R5: same fabricated API; `divisor_methods.rs` also uses f64 |
| J.3 | Adams Method | 2 | 3 | 2 | 3 | 2 | 2.4 | Major Revision | 2 | R1: "Adams maximises min representation" proof is incomplete/circular; R3: upper-quota corollary direction wrong |
| J.4 | Jefferson/D'Hondt | 3 | 3 | 3 | 3 | 2 | 2.8 | Minor Revision | 1 | R2: Jefferson/CA discrepancy between J.0 table and J.4 narrative |
| J.5 | Apportionment Paradoxes | 3 | 3 | 2 | 3 | 3 | 2.8 | Minor Revision | 2 | R3: Balinski-Young proof sketch is incomplete and does not demonstrate the impossible construction |
| J.6 | bisect-apportion Implementation | 2 | 3 | 3 | 3 | 1 | 2.4 | Major Revision | 4 | R5: SHA-256 `verify_sha256()` does not exist; API signatures fabricated; f64 vs u128 contradiction; test count inflated |

---

## Detailed Reviews

---

### J.0 — Apportionment Overview

**R1 — Karypis (algorithms/math): 3/4**

The taxonomy is correctly stated and the priority-queue formulation is
accurate. The $O(H \log n)$ complexity is cited for J.1's HH algorithm
in the J.1 paper and implicitly applies here; the overview section does
not state it but also does not misstate it. One technical confusion: the
taxonomy table (Section 2) lists the Adams and Jefferson priority values
as both $p_i/s$, with the comment "same as Adams; differs in base
allocation." This is correct for the priority formula at seat $s \geq 1$,
but the note that they "differ in base allocation" is insufficient
explanation — the key difference is that Adams initialises at 1 seat per
state and Jefferson at 0, producing systematically different outcomes for
the same priority formula. A reader could misread the table as claiming
the two methods are equivalent. Minor clarification required; no
mathematical error.

The "bias direction" ordering Adams ≻ HH ≻ Webster ≻ Jefferson is
correct and standard. The claim that HH's threshold lies strictly between
Webster and Adams is correctly derived from $s < \sqrt{s(s+1)} < s+0.5$
for all $s \geq 1$.

**R2 — Rodden (political science): 3/4**

The framing is appropriately careful. The introduction cites Wesberry v.
Sanders (1964) not as an apportionment ruling but as establishing that
"mathematical precision in apportionment and districting is a
constitutional value." This is a reasonable characterisation for overview
purposes, though R4 will have more to say about whether Wesberry is the
right citation here. The historical claim that "Congress has changed the
apportionment method eleven times since 1789" is consistent with
Balinski and Young (1982) and does not over-reach. The characterisation
of Congress's 1941 choice as reflecting "a subtle preference for
protecting small-state representation" is hedged with a citation and is
defensible.

One issue: the paper states that Hamilton was "used 1850–1900" but the
actual historical record (per Balinski and Young) is that Hamilton was
used from 1850 to 1900 (with Vinton method framing in some acts — these
are mathematically identical). The end-date of 1900 is correct; a
reviewer of the political history might flag that 1901–1910 used a fixed
apportionment from 1900, not Hamilton per se. This is a footnote-level
precision issue, not a substantive error.

**R3 — Duchin (math): 3/4**

The Balinski-Young theorem statement is correct: "no method can be both a
quota method (satisfying the quota rule) and a divisor method
(paradox-immune)." The paper does not attempt to prove it in J.0,
deferring to J.5. The quota rule definition is correctly stated as
$\lfloor q_i \rfloor \leq s_i \leq \lceil q_i \rceil$. The
characterisation of paradox immunity via monotonicity is correct.

One minor precision issue: the table on quota rule compliance states
"Adams: Always at upper quota ($s_i \geq \lceil q_i \rceil$)" and
"Jefferson: Always at lower quota ($s_i \leq \lceil q_i \rceil$); may
violate upper." The Jefferson entry should read $s_i \leq \lfloor q_i
\rfloor$ (lower quota), not $\lceil q_i \rceil$. Writing Jefferson as
"at lower quota ($s_i \leq \lceil q_i \rceil$)" is technically true
(since $\lfloor q \rfloor \leq \lceil q \rceil$ always) but the
relevant bound is $\lfloor q_i \rfloor$, and the notation is sloppy.
This should be corrected.

**R4 — Stephanopoulos (law): 3/4**

The Wesberry v. Sanders citation is used in a narrow and defensible way:
"congressional districts must be as equal in population as practicable,
establishing that mathematical precision in apportionment and districting
is a constitutional value." Wesberry held that the Georgia at-large
apportionment scheme violated Article I Section 2's requirement of
population equality in congressional districts — so the citation is
correct that mathematical precision is a constitutional value. However,
the paper's phrasing conflates apportionment (how many seats per state)
with districting (how district lines are drawn within a state). Wesberry
speaks to the latter. The paper's hedging ("apportionment and
districting") is technically correct, but a hostile reader could argue
that Wesberry does not speak to the choice of apportionment method at
all. The Montana case (cited in J.1) is the controlling precedent for
apportionment method choice.

Montana (1992) is correctly characterised in J.0's synthesis section
(Section 6) as the case that "confirmed Congress's broad discretion."
No legal error.

**R5 — Liang (systems): 2/4**

**P1 (major):** Section 5 (Implementation) describes a public API:
```
pub fn apportion(
    populations: &[(String, u64)],
    house_size: usize,
    method: ApportionMethod,
) -> Vec<(String, usize)>
```
and an `ApportionMethod` enum with `HuntingtonHill`, `Webster`, `Adams`,
`Jefferson`, `Hamilton` variants. This API does not exist in the actual
crate. The real public exports are `huntington_hill()` (takes a
`HashMap<String, u64>`), `apportionment_divisor()` (takes a
`&[(String, u64)]` with a `RoundingRule` enum — not `ApportionMethod`),
and `check_alabama_paradox()`. There is no unified `apportion()` function
and no `ApportionMethod` enum in the live code. The paper describes a
future desired API, not the current one. **Major revision required.**

The SHA-256 verification claim ("The two hashes match exactly, confirming
zero discrepancies across all 50 states") cannot be independently
verified from the paper; see J.6 for a full audit, but J.0 references
a SHA-256 protocol that does not exist in the crate code.

**Verdict: Minor Revision** (avg 2.8). Fix Jefferson quota table notation
(R3). Disambiguate Adams vs Jefferson priority initialisation (R1).
Clarify Wesberry citation scope (R4). Update or remove fabricated
`apportion()` API (R5) — this is the only P1.

---

### J.1 — Huntington-Hill

**R1 — Karypis (algorithms/math): 3/4**

The $O(H \log n)$ complexity claim is correct for the priority-queue
algorithm. The priority formula $P_i(s) = p_i / \sqrt{s(s+1)}$ is the
standard formulation. The worked example (5-state, 10 seats) is computed
correctly. The proof sketch of Theorem 1 (population pair test) correctly
identifies that HH priority inequality is equivalent to pairwise deviation
minimisation, and references Balinski and Young (1982) ch. 4 for the
full proof. This is legitimate for a proof sketch.

The Theorem 2 proof (paradox immunity) is mostly correct but the
proof of (c) (New-States paradox) is weak. The argument that "the divisor
$d = N/(H) \approx (N+p_k)/(H+m)$ changes by $O(p_k/H^2)$, which for
small $p_k/N$ does not perturb any other state's allocation" is informal
and relies on an unverified smallness assumption. The formal proof is
deferred to Balinski and Young ch. 5. This is acceptable for a proof
sketch but should explicitly acknowledge that the divisor perturbation
argument is the non-trivial part, not waved away as $O(p_k/H^2)$ without
justification. No computational error.

**R2 — Rodden (political science): 3/4**

The history section (Section 5) is well-documented. The claim that
"the apportionment ultimately used for the 1930s was the same as the 1910
apportionment — no reapportionment occurred" is historically accurate.
The characterisation of the National Academy of Sciences panel
recommending HH in 1929 is correct. The claim that "Congress's choice of
HH over Webster has been described as reflecting a subtle preference for
protecting small-state representation" is appropriately hedged with
citations to Balinski and Young and Schmeckebier.

One historical precision issue: the paper says Congress was using
Hamilton 1850–1900 and then froze the House at 435, but the Apportionment
Act of 1929 did not freeze the House immediately — the 1930 Census
reapportionment crisis is correctly described. No major error.

**R3 — Duchin (math): 3/4**

Theorem 1 (population pair test optimality) is stated correctly. The
proof sketch is an argument by contradiction: if a transfer reduces
max deviation, it contradicts the priority ordering. This is the right
approach. The reference to Balinski and Young ch. 4 for the full proof
is appropriate.

Theorem 2 proof part (b) (population paradox) contains a subtle logical
gap: "state $i$'s priority for its $(s_i+1)$-th seat may now exceed
some other state's priority, giving it $s_i+1$ or more seats. But it
cannot lose seats, because its priorities for seats $1, \ldots, s_i$
all increased." This argument is correct but needs to be more careful:
the priorities for state $i$'s seats 1 through $s_i$ increased, so those
steps remain allocated to state $i$ in the revised priority sequence.
The argument is sound but should be stated as: in the priority queue,
the rank of state $i$'s $k$-th seat (for $k = 1, \ldots, s_i$) can only
improve (move earlier), so state $i$ cannot lose any of its existing
seats. The current wording is slightly ambiguous.

**R4 — Stephanopoulos (law): 4/4**

The Montana section (Section 6) is the critical legal section and is
handled correctly. The paper identifies the four holdings:
(1) justiciable (not a political question), (2) Congress has broad
discretion, (3) HH is constitutional as a rational choice, (4) fractions
make perfection impossible. This accurately captures the 9-0 opinion
authored by Justice Stevens. The Court's rational basis framing (Congress
may choose any method that is a "reasonable interpretation" of
proportional representation) is correctly characterised as statutory
deference, not a constitutional imprimatur on HH specifically. The paper
correctly notes that the Dean method and maximum-deviation minimisation
were Montana's proposed alternatives, which the Court rejected.

No legal errors. The historical posture is correct: direct appeal under
28 U.S.C. §1253 is correctly cited. The district court finding for
Montana is correctly identified as the ruling the Supreme Court reversed.

**R5 — Liang (systems): 1/4**

**P1 (critical):** The implementation section (Section 7) states:
"The priority comparison uses *exact integer arithmetic* to avoid
floating-point rounding errors in tie-breaking:
$P_i(s_i) = p_i / \sqrt{s_i(s_i+1)}$
is represented as the rational $p_i^2 / (s_i(s_i+1))$, with numerator
and denominator stored as `u128` integers."

This is false. The actual `huntington_hill.rs` (the primary HH
implementation at `crates/bisect-apportion/src/huntington_hill.rs`)
uses `f64` throughout:
```rust
let priority_for = |pop: u64, n: u32| -> f64 {
    let n = n as f64;
    pop as f64 / (n * (n + 1.0)).sqrt()
};
```
The priority comparisons are floating-point, not exact integer
arithmetic. The `u128` exact arithmetic appears only in J.6's code
snippets (which describe aspirational code), not in the production
implementation. The discrepancy between what the paper claims and what
the code does is a material factual error that affects the paper's
correctness claims.

**P2 (major):** The test section claims L0/L1/L2 test levels, with L0
as "Priority formula produces exact values for small inputs;
$P(1) = p/\sqrt{2}$, $P(2) = p/\sqrt{6}$, etc." These tests are based
on the floating-point implementation, so the "exact values" are actually
floating-point approximations. This does not contradict correctness for
census-scale populations (where f64 has sufficient precision), but the
paper's description of the test as verifying "exact" values is misleading.

The SHA-256 verification claim ("the hashes match exactly") is not
independently verifiable from the code — `verify_sha256()` does not
exist as a public function in the crate.

**Verdict: Minor Revision** (avg 2.8). Critical fix required: the claim
of exact u128 integer arithmetic is wrong for `huntington_hill.rs`. The
paper must be corrected to accurately describe the f64 implementation,
or the implementation must be updated to match the paper's description.
This is the paper's most important correctness claim (SHA verification
depends on exact arithmetic) and it is wrong.

---

### J.2 — Webster Method

**R1 — Karypis (algorithms/math): 3/4**

The Webster priority formula $P_i^{Web}(s) = p_i/(s+0.5)$ is correct.
The Sainte-Laguë equivalence proof is correct: $p_i/(s+0.5) = 2p_i/(2s+1)$
is proportional to $V_j/(2s+1)$, producing identical priority orderings.
The least-squares optimality theorem and proof sketch are correct: the
proof correctly identifies that the transfer improvement condition
($q_i - s_i > q_j - s_j$) is equivalent to the rounding-at-0.5 criterion.

The rounding threshold comparison is correctly derived:
$(s+0.5) - \sqrt{s(s+1)} \approx 1/(8s)$ for large $s$. The derivation
shown is correct.

One issue: the paper states that the first-seat priority is
$P_i^{Web}(0) = 2p_i$ and notes "differs from Huntington-Hill's first
seat priority $P_i^{HH}(0) = p_i/\sqrt{1\cdot0}$ (which is technically
infinite)." The formula $p_i/\sqrt{1\cdot 0}$ has a zero denominator;
the intended formula is $p_i/\sqrt{s(s+1)}$ evaluated at $s=0$ giving
$p_i/0$, which is indeed infinite. But writing $P_i^{HH}(0) =
p_i/\sqrt{1\cdot0}$ is both ill-defined and misleading since
$\sqrt{1\cdot0} = 0$. The correct statement is that HH's first-seat
priority is undefined (infinite) because all states receive their first
seat via the constitutional floor before the queue begins. The formula
should not be written as $p_i/\sqrt{1\cdot0}$.

**R2 — Rodden (political science): 3/4**

Historical claims are accurate. Webster was used in 1842 and 1911–1940;
this matches the historical record. The paper correctly notes that the
difference between HH and Webster manifests in some states near the
rounding boundary and is otherwise politically invisible. No overstatement.

**R3 — Duchin (math): 3/4**

The least-squares optimality theorem is stated and proved correctly for
the integer apportionment case. The proof identifies the correct transfer
condition and explains why it is equivalent to rounding at 0.5. The
statement that "Webster is the unique divisor method with zero systematic
bias in the arithmetic-mean sense" is correctly hedged (arithmetic-mean
sense). No mathematical errors in the main theorems.

**R4 — Stephanopoulos (law): 3/4**

No legal claims specific to Webster; the paper correctly defers legal
framing to J.0 and J.1 (Montana). No errors.

**R5 — Liang (systems): 2/4**

**P1 (major):** Same fabricated API issue as J.0/J.1: the implementation
section describes `apportion()` with `ApportionMethod::Webster` which
does not exist. The actual function is `apportionment_divisor()` with
`RoundingRule::Webster`. The `divisor_methods.rs` implementation uses
`f64` (not the `u128` exact arithmetic the papers imply), which is not
mentioned.

SHA-256 verification is not implemented for Webster in the actual code.

**Verdict: Minor Revision** (avg 2.8). Fix HH first-seat priority
notation (R1). Update implementation API description (R5).

---

### J.3 — Adams Method

**R1 — Karypis (algorithms/math): 2/4**

**P1 (major):** Theorem 1 ("Adams Maximises Minimum Representation") has
a circular proof. The proof states: "Suppose some apportionment
$(s_1', \ldots, s_n')$ with the same total $H$ achieves higher minimum
representation. Let $j = \arg\min_i(p_i/s_i^{Ad})$. Since
$p_j/s_j' > p_j/s_j^{Ad}$, we have $s_j' < s_j^{Ad}$. But
$s_j^{Ad} = \lceil p_j/d \rceil$ is already the smallest possible
value for a ceiling-rounding method at divisor $d$. Any smaller $s_j'$
would violate ceiling rounding for some achievable divisor."

This argument is circular: it assumes that the competing method must also
be a ceiling-rounding method "at some achievable divisor." The theorem
claims Adams maximises minimum representation among *all* apportionments
(not just divisor methods). The proof needs to show that any
non-ceiling-rounding allocation cannot achieve higher minimum
representation, which requires a different argument (e.g., showing that
the Adams divisor choice is the minimax optimum over all integer
allocations summing to $H$, not just over divisor methods).

The worked example (finding the Adams divisor by binary search) correctly
identifies that Adams may not have a divisor giving exactly $H$ seats,
which is an important point raised by the example but not formalized in
the theorem statement. This is a real mathematical subtlety the paper
acknowledges in the example but not in the theorem.

**R2 — Rodden (political science): 3/4**

The characterisation of Adams as the "most small-state-favoring" method
is correct and politically well-framed. The paper correctly notes Adams
has never been adopted for federal apportionment, citing the imbalance
it would create for large states. No political science errors.

**R3 — Duchin (math): 2/4**

**P1 (major):** The Corollary (Upper Quota Compliance) states: "Under
Adams, every state satisfies $s_i \geq \lceil q_i \rceil$, i.e., no
state receives fewer seats than its exact proportional entitlement
rounded up." This corollary's direction is correct (Adams gives at least
the upper quota), but it creates a logical problem with the Balinski-Young
theorem (proved in J.5): if Adams always gives every state at least its
upper quota, and the upper quota satisfies $s_i \geq \lceil q_i \rceil
\geq q_i$, then Adams satisfies a strict version of the quota rule
(always at or above exact quota). But Balinski-Young says no paradox-free
method can satisfy the quota rule for all population vectors. The paper
needs to reconcile: does the Corollary apply only when the constitutional
floor is not binding? And does Adams ever violate the quota rule from
below? (It cannot by design, but it can violate from above: a state might
receive more than $\lceil q_i \rceil$ seats under Adams, violating the
upper bound of the quota rule.) The paper needs to clarify that Adams
satisfies a one-sided quota property (never below $\lceil q_i \rceil$),
not the full bilateral quota rule — and thus does not contradict
Balinski-Young since it can violate the upper bound ($s_i \leq \lceil
q_i \rceil$) which is the upper bound of the quota rule.

This is a precision issue that could confuse careful readers about the
relationship between Adams and the Balinski-Young result.

**R4 — Stephanopoulos (law): 3/4**

No substantive legal claims. Correct citation of Adams's historical
non-adoption. No errors.

**R5 — Liang (systems): 2/4**

Same fabricated API issue. The Adams priority `p_i/s` is correctly
described in `divisor_methods.rs` as `RoundingRule::Adams` with
`priority_for = p / n` (where `n` is the current seat count). The paper's
initialisation claim (Adams starts from 1 seat, Jefferson from 0) is
approximately correct for the actual code — `divisor_methods.rs` starts
all methods at 1 seat and uses the priority formula for subsequent seats,
with Jefferson using `p / (n+1)` from seat 1 onward. The description of
the initialisation difference is qualitatively right but the paper's
account of Jefferson starting from 0 is not what the code does.

**Verdict: Major Revision** (avg 2.4). The Theorem 1 proof (R1) has a
circular argument that must be corrected or replaced. The Corollary on
upper quota compliance (R3) needs clarification of the one-sided vs.
bilateral nature of Adams's quota property to avoid apparent contradiction
with Balinski-Young.

---

### J.4 — Jefferson/D'Hondt

**R1 — Karypis (algorithms/math): 3/4**

The Jefferson/D'Hondt equivalence theorem and proof are correct. The
proof identifies that $P_i^{Jeff}(s) = p_i/(s+1)$ and
$P_j^{D'H}(s) = V_j/(s+1)$ are identical formulas under the substitution
$p_i = V_j$. The conclusion that "the priority-queue algorithm depends
only on the priority function" and therefore produces identical sequences
is correct. This is a clean, complete proof of the equivalence.

The large-party bias analysis (Section 4) correctly traces the mechanism:
floor rounding discards fractional seats whose fractional value is a
larger share of small parties' total seats than large parties'. The
quantitative example (10% party, 10 seats) is correctly computed.

One concern: the Jefferson bias theorem (Theorem 1 in Section 4) is
stated as "Among all divisor methods, Jefferson is the most large-state-
favoring." The proof sketch is by a fractional-loss argument. This
ordering result (Adams ≻ HH ≻ Webster ≻ Jefferson) is well-established
in the literature, and the paper appropriately cites Balinski and Young
ch. 4. The proof sketch is qualitatively correct.

**P1 (notation):** The d'Hondt worked example has Party E receiving 0
seats, with the note "Party E (90 votes, 9% of total) receives 0 seats
despite crossing the 10% threshold with 90/1000 votes." The total votes
are $(340+280+160+130+90) = 1000$, so 90/1000 = 9%, not "crossing the
10% threshold." This is an error in the worked example: 90/1000 = 9% does
*not* cross any 10% threshold. The sentence is confused.

**R2 — Rodden (political science): 3/4**

**P1 (major):** The J.0 table (Section 3, `tab:2020seats`) shows
California receiving 53 seats under Jefferson, and Texas 39. But J.4's
Section 4 (Bias) states: "California and Texas lose 1 seat each under
Jefferson relative to HH (because HH's lower threshold for rounding up
awards their fractional excess seats)." If California gets 53 under
Jefferson and 52 under HH, California gains under Jefferson, not loses.
The J.4 narrative is inconsistent with the J.0 table. The J.0 table
(which shows CA: 53 Jefferson, 52 HH) is consistent with the large-state-
favoring bias claim (Jefferson favors large states), but J.4's text
says CA loses, which is the opposite direction. One of these is wrong.
Given that Jefferson = floor rounding = large-state-favoring, CA should
gain (or at least not lose) seats under Jefferson. The J.0 table appears
correct; the J.4 narrative is wrong.

This is a factual inconsistency between papers that must be corrected.

**R3 — Duchin (math): 3/4**

The equivalence proof is correct. The large-state bias ordering is
correctly stated and referenced. No mathematical errors beyond the minor
D'Hondt example issue (R1 note above).

**R4 — Stephanopoulos (law): 3/4**

Correct historical framing: Jefferson was used 1790–1840, was criticized
for large-state bias, and was replaced. No legal claims that require
checking.

**R5 — Liang (systems): 2/4**

Same fabricated API. Jefferson initialisation in actual code:
`divisor_methods.rs` initialises Jefferson at 1 seat per state and uses
`priority = p / (n+1)`. The paper claims Jefferson "initialises with 0
seats per state," which is what the algorithm theoretically does but not
what the code does (the code, like all methods in that file, starts at 1
seat). The description of "apply constitutional floor: if any $s_i = 0$,
redistribute from the state with most seats" as a post-processing step
is also not in the code; the code enforces the floor by starting at 1.

**Verdict: Minor Revision** (avg 2.8). The CA/Jefferson direction
discrepancy between J.0 and J.4 is a P1 that must be corrected (R2).
Fix the D'Hondt worked example text about the 10% threshold (R1). Update
Jefferson initialisation description to match code (R5).

---

### J.5 — Apportionment Paradoxes

**R1 — Karypis (algorithms/math): 3/4**

The Alabama paradox section is mathematically correct. The formal
definition, the 1880 historical example (Alabama: 8 seats at H=299, 7
seats at H=300), and the mechanism (remainder seats count decreasing when
lower quotas increase by 2) are all correct.

The divisor method immunity theorem (Theorem in Section 5) and proof are
correct. The three-part proof of Alabama paradox immunity, population
paradox immunity, and new-states paradox immunity is clearly structured.
The new-states immunity proof correctly acknowledges the divisor
perturbation subtlety and defers to Balinski and Young ch. 5.

One concern: the paper claims to prove "3 states with populations
$(3,2,1)$" suffice for the Balinski-Young proof sketch, but then the
proof sketch pivots to "The formal proof requires a more specific
construction where..." with $n=4$ states. The paper presents both a 3-state
example and a 4-state one, which is fine, but the 3-state example never
actually exhibits the impossibility — it shows no paradox. The reader
may be confused about why the 3-state example was introduced if it does
not produce the impossibility. The paper should clearly state that the
3-state example is pedagogical (showing quota compliance and house
monotonicity are compatible in that specific case) before moving to the
4-state impossibility construction.

**R2 — Rodden (political science): 3/4**

The political history is well-documented: Alabama 1880, Population
Paradox 1900, Oklahoma 1907 (new-states paradox). These historical cases
are accurately characterised. The conclusion that "Congress agreed [with
Balinski and Young that paradox immunity is more fundamental]" is
appropriate; the 1941 Act reflects this priority.

**R3 — Duchin (math): 2/4**

**P1 (major):** The Balinski-Young impossibility proof sketch (Section 6)
is incomplete and partially incorrect.

The paper presents three different example configurations but never
actually demonstrates the impossibility. The 3-state $(3,2,1)$ example
at H=6 and H=7 shows no paradox — the proof sketch acknowledges this.
The 4-state $(5k, 2k, k, k)$ example shows that Hamilton is house
monotone in that configuration. The paper then states "The formal proof
requires a more specific construction where a quota-compliant method is
forced to make a 'wrong' choice at some $H$ and then violate monotonicity
at $H+1$" and defers entirely to Balinski and Young (1982) Theorem 4.1.

A proof *sketch* for an impossibility theorem must at minimum exhibit
a specific configuration where any quota-compliant method is forced to
violate house monotonicity. The current proof sketch presents three
configurations that do *not* exhibit the impossibility and then defers.
This is not a proof sketch — it is a statement of the result with an
attempt at examples that fails to reach the punchline.

The theorem statement itself requires $n \geq 3$ states, but the informal
discussion of the 3-state case shows no impossibility, potentially
suggesting to a naive reader that 3 states might be safe. Balinski and
Young's actual result requires $n \geq 4$ in the strong form, and the
condition "$n \geq 3$" in the theorem as stated may be too weak (the
paper's own examples suggest the 3-state case does not exhibit the
paradox). The theorem condition should be verified.

**P2 (minor):** The claim "Balinski and Young's proof constructs a 4-state
example where: at $H = H_0$, any quota-compliant method must give state
$i$ exactly $\lceil q_i \rceil$ seats; at $H = H_0+1$, the quotas shift
such that state $i$'s upper quota decreases to $\lfloor q_i \rfloor <
\lceil q_i^{H_0} \rceil$, forcing a quota-compliant method to reduce
state $i$'s seat count." This correctly captures the structure of the
proof but is so vague as to be unverifiable without reading the original.
For a paper specifically about paradoxes, the proof of the central
impossibility result should be made more concrete.

**R4 — Stephanopoulos (law): 3/4**

No legal claims specific to J.5 beyond confirming the political context
of the 1941 Act. Correctly framed.

**R5 — Liang (systems): 3/4**

The paradox implementation (`paradoxes.rs`) accurately reflects what the
paper describes: `check_alabama_paradox()` iterates over house size ranges
and checks for seat decreases. The actual function signature matches
what the paper implies (though the paper does not explicitly describe it).
The claim that "The L0 test asserts this invariant" for Alabama paradox
immunity is consistent with the test code (`no_alabama_paradox_huntington_hill()`
in `paradoxes.rs`). Test coverage for Hamilton susceptibility is mentioned
in J.5 but the actual paradoxes.rs only tests divisor methods; no Hamilton
paradox test exists in that file. Minor discrepancy.

**Verdict: Minor Revision** (avg 2.8). The Balinski-Young proof sketch
is the P1: it must exhibit a concrete configuration showing the
impossibility, or acknowledge it cannot and simply state the result.
The current sketch misleads by presenting examples that do not demonstrate
the impossibility.

---

### J.6 — bisect-apportion Implementation

**R1 — Karypis (algorithms/math): 2/4**

The implementation description of the priority-queue algorithm is correct
in structure. The overflow analysis is correctly computed: maximum
cross-multiplication value $\approx 4.3 \times 10^{18}$ for HH is within
u128 range. The Webster overflow analysis is also correct.

**P1 (major):** The priority comparison code shown for Adams is:
```rust
fn adams_priority(pop: u128, seats: u32) -> Priority {
    Priority {
        numerator: pop * pop,
        denominator: (seats as u128) * (seats as u128),
    }
}
```
But Adams priority is $p_i/s$, and the exact comparison is
$p_i/s_i > p_j/s_j$ which cross-multiplies to $p_i \cdot s_j > p_j
\cdot s_i$, equivalently $p_i^2 \cdot s_j^2 > p_j^2 \cdot s_i^2$.
This is what the code computes ($p^2/s^2$). But the original priority
is $p/s$ not $p^2/s^2$ — the squaring preserves order only if both
are positive, which is correct. The code is correct but the paper's
description is mildly confusing: the text says "The comparison $p_i/s_i
> p_j/s_j$ becomes $p_i \times s_j > p_j \times s_i$ (or equivalently
$p_i^2 s_j^2 > p_j^2 s_i^2$)." These are equivalent only because both
are positive, but noting this would improve clarity.

**R2 — Rodden (political science): 3/4**

No political claims. Correctly contextualized.

**R3 — Duchin (math): 3/4**

The mathematical content of the implementation description is internally
consistent. The tie-breaking rule (alphabetical by state name) is
documented and correct — the paper correctly notes that true ties are
mathematically equivalent. No errors.

**R4 — Stephanopoulos (law): 3/4**

The legal significance of SHA verification is correctly framed: it
establishes an auditable chain of custody from Census Bureau publications
to the redistricting platform. The claim that Montana confirms
SHA-verified correctness is "legally as well as mathematically
significant" is defensible.

**R5 — Liang (systems): 1/4**

**P1 (critical):** The paper's entire premise is that `bisect-apportion`
implements exact u128 arithmetic and SHA-256 verification. Neither exists
in the actual code:

- `huntington_hill.rs` uses `f64` floating-point priority comparison,
  not the `u128 Priority` struct described throughout J.6.
- `divisor_methods.rs` uses `f64` for all priority values.
- There is no `verify_sha256()` function in the crate.
- There is no SHA-256 dependency in the crate.
- There is no committed reference hash constant in the code.

The `Priority` struct with u128 numerator/denominator shown in Section 2
("Architecture") does not exist as a type in the actual code. The code
snippets `hh_priority()`, `webster_priority()`, `adams_priority()`,
`jefferson_priority()` shown in Sections 3–4 are not real functions in
the crate.

**P2 (critical):** The test count claim of "61 tests organised into three
levels" is wrong. The actual test count in the apportionment-relevant
source files:
- `huntington_hill.rs`: 5 tests
- `divisor_methods.rs`: 12 tests  
- `paradoxes.rs`: 7 tests
Total in apportionment modules: 24 tests.

The total `#[test]` count across the entire crate is approximately 95,
but these include tests for `compositor.rs`, `prime.rs`, `split.rs`,
`nest.rs`, and `graph.rs` — none of which are apportionment tests. The
paper is either counting tests from the entire crate (misrepresenting
them as apportionment tests) or counting tests that do not exist.

**P3 (major):** The public API described in Section 2:
```rust
pub fn apportion(populations: &[(String, u64)], house_size: usize, method: ApportionMethod) -> Vec<(String, usize)>
pub fn verify_sha256(allocation: &[(String, usize)], reference_hash: &str) -> bool
```
does not match the actual exported API:
- `huntington_hill()` takes `&HashMap<String, u64>`, returns `HashMap<String, u32>`
- `apportionment_divisor()` takes `&[(String, u64)]`, returns `Vec<(String, u32)>`
- `check_alabama_paradox()` takes `&[(String, u64)]` and `&[u32]`
- No `apportion()`, no `verify_sha256()`, no `ApportionMethod` enum

**P4 (major):** The "last seat" section states "The 385th seat (the last
seat allocated above the constitutional floor) in the 2020 apportionment
was awarded to New York." This is arithmetically wrong: 435 total seats
minus 50 constitutional floor seats = 385 remaining seats, so the 385th
such seat is indeed the 435th overall. But the paper's New York priority
formula shows $P_{NY}(25) = 20{,}201{,}249^2 / (25 \times 26)$, implying
New York had 25 seats before receiving the last seat (i.e., it received
its 26th seat as the last). This is consistent with New York getting 26
seats total in 2020. However, the paper elsewhere (J.1 history section)
correctly identifies the last seat went to New York, while also saying
"Texas and Minnesota were within a small number of priority points of
gaining or losing a seat." This is internally consistent but the J.6
version is slightly garbled in presenting the "385th" seat calculation
without clarifying that this equals the 435th overall seat.

**Verdict: Major Revision** (avg 2.4). Four P1s. The paper fundamentally
misrepresents the state of the implementation. Either the code must be
updated to implement what the paper describes (exact u128 arithmetic,
SHA-256 verification, unified `apportion()` API), or the paper must be
rewritten to accurately describe what the code actually does. Given the
central role of the SHA-verified correctness claim in the paper's
contribution, this is not a minor correction.

---

## Cross-Paper Issues

### Issue 1: Jefferson seat allocation for California (J.0 vs J.4)

J.0 Table 1 shows California receiving **53 seats** under Jefferson and
**52 under HH**. J.4 Section 4 states "California and Texas **lose** 1
seat each under Jefferson relative to HH." These are directly
contradictory. Jefferson (floor rounding) is the large-state-favoring
method; California should gain seats under Jefferson relative to HH, as
the J.0 table shows. The J.4 narrative has the direction of bias backward
for California. **Must fix in J.4.**

### Issue 2: f64 vs u128 (J.1, J.6, and all papers claiming "exact arithmetic")

J.0, J.1, J.2, J.3, J.4 all reference SHA-256 verification or exact
integer arithmetic as part of the platform's correctness story. J.6
presents this as the core contribution. The actual implementation in
`huntington_hill.rs` and `divisor_methods.rs` uses `f64` floating-point.
For census-scale populations (up to ~40M) and seat counts up to ~52,
`f64` has 53-bit mantissa precision, which is sufficient to correctly
rank all priorities without ties. However:
(a) The papers' claims of "exact" comparison are factually incorrect.
(b) For extremely close priorities (near-tie cases), f64 could in principle
produce wrong results, though this has not occurred in practice.
(c) The SHA-256 verification story depends on exact computation, which
f64 undermines conceptually even if not in practice.

All papers should be corrected to accurately describe the implementation
as using floating-point arithmetic with sufficient precision for
census-scale inputs, or the implementation should be upgraded to use
exact arithmetic as described.

### Issue 3: Adams priority formula (J.3 says $p_i/s$; J.6 code shows $p_i^2/s^2$)

J.3 defines Adams priority as $P_i^{Ad}(s) = p_i/s$ and states "same
formula as Jefferson." J.6 shows the Adams implementation as
`numerator: pop * pop, denominator: seats * seats`, which represents
$p_i^2/s^2$. These are equivalent for comparison purposes (both are
positive), but the papers should consistently either use $p_i/s$ or
$p_i^2/s^2$, not both. The J.6 implementation is correct but the
notation is inconsistent with J.3.

### Issue 4: Test count

Multiple papers (J.6 introduction, J.0 implementation section) claim
"61 tests." The actual count in apportionment-specific modules is 24
tests. If "61 tests" refers to the entire crate (all modules), this is
misleading in the context of papers about apportionment. The claim should
be corrected to either (a) the correct count for apportionment modules,
or (b) the total crate count with clear disclosure that it includes
non-apportionment functionality.

---

## Recommended Priority of Fixes

**Critical (must fix before acceptance):**
1. J.6 P1: Implement actual SHA-256 verification and u128 exact arithmetic,
   OR rewrite J.6 to accurately describe the f64 implementation.
2. J.6 P2: Correct the test count (24, not 61, for apportionment modules).
3. J.6 P3: Correct the public API description to match actual exports.
4. J.4 P1: Fix the California/Jefferson direction error (J.0 table says CA
   gains under Jefferson; J.4 text says CA loses).

**Major (must fix, could be done in revision round):**
5. J.3 P1: Correct the circular proof of Adams Theorem 1.
6. J.3 P2 (R3): Clarify Adams upper-quota corollary vs. Balinski-Young.
7. J.5 P1 (R3): Provide a concrete Balinski-Young impossibility example.
8. J.1 P1 (R5): Correct the f64 vs u128 description for HH implementation.

**Minor (fix in copyedit):**
9. J.0: Jefferson quota table notation ($\lfloor q_i \rfloor$ not $\lceil q_i \rceil$).
10. J.2: HH first-seat priority formula $p_i/\sqrt{1\cdot0}$ is ill-defined.
11. J.4: D'Hondt worked example "crossing the 10% threshold" error (90/1000 = 9%).
12. J.1: New-states paradox proof sketch — divisor perturbation needs acknowledgment.

---

*Panel review completed 2026-05-08. All scores are 0–4 with 4 = Accept as-is.*
