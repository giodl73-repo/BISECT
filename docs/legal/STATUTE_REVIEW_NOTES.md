# Statute Review Notes: Open Tensions and Architectural Alternatives

**Status:** v0, 2026-05-01 (companion to MODEL_FEDERAL_STATUTE.md v0.1)
**Audience:** coalition partners, drafting-team principals, future maintainers of the bill text.

This document captures the issues that the v0 → v0.1 redraft did NOT fully resolve, and the architectural alternative (Option B — conditional-spending restructure) that we deliberately deferred. It is the place to record the things a serious negotiation will need to address.

---

## 1. The VRA-versus-Equal-Protection tension (unresolved)

The 5-role review surfaced a deep doctrinal tension that no single drafting choice can resolve. v0.1 mitigates it; it does not fully solve it.

**The two pulls:**

- **VRA § 2 plaintiffs' counsel** wants § 106 *strengthened* — "shall modify" not "may," compelled § 2 analysis even when no modification is made, mandatory pre-bisection § 2 set-asides. Their concern: the algorithm's race-blind output becomes a defendant's brief — "we ran the algorithm, the algorithm has no intent and no disparate effect, § 2 doesn't reach us." Without strong § 106, the bill *manufactures* a defense that current law doesn't give defendants.

- **Hostile constitutional challenger** wants § 106(c) compelled written justification *removed* — exactly because requiring the State to publish a *Gingles* analysis for every modification makes race the explicit, predominant factor for those districts under *Miller v. Johnson* (1995) and *Cooper v. Harris* (2017). Strict scrutiny attaches; the State has manufactured the *Shaw* claim against itself.

- **Constitutional law professor** identifies the same tension: the bill's transparency theory (publish *Gingles* analysis → auditable § 2 compliance) is in tension with strict-scrutiny doctrine (compelled race-conscious justification → race predominated → strict scrutiny → narrow tailoring required).

**v0.1's mitigations** (partial):

- § 106(b) "may" → "shall" (strengthens VRA per civil-rights counsel's #1 ask).
- New § 106(b)(2): "Failure to modify creates no presumption" (closes the "algorithm-did-it" defense).
- § 106(c) restructured around *Allen v. Milligan*'s **narrow-tailoring framework**, not a bare *Gingles* recital. The State must show the modification is "no broader than necessary" and "does not subordinate traditional districting principles further than § 2 compels." This pre-emptively answers strict-scrutiny narrow-tailoring on the State's own record.
- § 106(d) presumption limited to *Shaw*-type challenges; expressly excludes § 2 effect claims. So the asymmetric-burden problem (presumption only when State modifies, never when State declines) is replaced with: presumption applies only against *Shaw*-trolls, never against § 2 plaintiffs.
- § 107(g) asymmetric fee-shifting: § 2-protective plaintiffs recover; *Shaw*-troll plaintiffs bear their own costs.

**What remains unresolved:**

The compelled written justification under § 106(c) still creates an evidentiary record that supports strict-scrutiny analysis. v0.1 frames the justification around narrow-tailoring (which is the showing the State must make under strict scrutiny anyway), but a hostile court could still hold that the federal mandate to publish *Gingles* analysis itself triggers strict scrutiny — that the federal compulsion supplies the "predominant factor" element, regardless of the State's own intent.

**Three further-mitigation options** (none adopted in v0.1; each has costs):

1. **In-camera filing**: have § 106(c) require an unsealed *Gingles* analysis filed under judicial seal, accessible only on litigation. Defuses *NIFLA*-style compelled-speech objection but breaks the bill's transparency theory.
2. **State-as-licensee framing**: rewrite § 106 so the State opts INTO the deviation (rather than the federal government compelling it), with § 2 compliance as the underlying obligation. Doctrinally cleaner under *Murphy*/*Printz* but harder to administer.
3. **Pre-bisection § 2 set-asides**: per civil-rights counsel's preferred reading of *Callais* p.36, the State could draw § 2-required districts FIRST and then run the algorithm on the remaining geography. Keeps race-conscious districts from being "deviations" framed as exceptional. Trade-off: makes the algorithm's output not strictly deterministic from public inputs — the § 2 pre-set is a State judgment that the verifier cannot reproduce.

The drafting team should expect this tension to be the single most contested issue in committee mark-up, likely from BOTH directions simultaneously. Hold position; do not concede on § 106(b) ("shall") or § 106(b)(2) (no-presumption-from-failure-to-modify).

## 2. Option B: Conditional-spending architectural alternative (deferred)

The hostile-challenger review predicted **60–70% chance of striking the v0 bill on Tenth Amendment / *Murphy v. NCAA* grounds** at the Supreme Court. v0.1 mitigates with explicit § 101(g) findings citing *Foster v. Love* + *Cook v. Gralike* + *Smiley v. Holm* and adds § 109(d) dual-authority (§ 5 of the 14th Amendment as fallback). But the hostile reviewer's deepest critique stands: the bill commands State officials to perform specified acts, and that's the *Printz* paradigm regardless of the Elections Clause defense.

**Option B is the architectural restructure that closes this attack vector entirely.** It restructures § 103's mandate as a condition on federal funding rather than a direct command:

> *"§ 103. Conditional spending.* No State shall receive Help America Vote Act § 257 disbursements, or such other federal election-administration funding as Congress may designate, for any election cycle in which the State has not (a) executed the algorithm under § 104, (b) published the reproducibility artifacts under § 105, and (c) complied with § 106 VRA modifications and § 110 state-constitutional integrity rules."

This is structurally identical to *South Dakota v. Dole* (1987): Congress doesn't command; it conditions. *NFIB v. Sebelius* (2012) applies a "coercion" limit (Medicaid expansion's all-or-nothing structure was unconstitutional), but HAVA funding is a much smaller fiscal lever and cannot plausibly be called coercive.

**Why we DID NOT adopt Option B in v0.1:**

1. **Politics.** The bill's goal is national mandatory compliance. A conditional-spending architecture means a wealthy, politically homogeneous State (consider AL, MS, or CA) can simply forgo the federal funding and continue current practice. The bill's reform value drops significantly if even 5–10 States opt out.

2. **Optics.** "If you want federal money, you have to follow this rule" reads as soft compared to "every State shall." The political case for the bill rests on universal applicability.

3. **Existing authority is robust.** *Foster v. Love* (1997) and *Cook v. Gralike* (2001) are firmly on the books. The Court has never narrowed Article I § 4 the way it narrowed the Tenth-Amendment carve-out for general legislative powers. v0.1's § 101(g) findings + § 109(d) dual-authority approach is defensible.

**When to revisit Option B:**

1. **If 60-vote outreach reveals categorical opposition to the mandate posture** — particularly from small-state Senators on Tenth-Amendment grounds — a conditional-spending pivot may be the only path to passage.
2. **If a circuit splits adversely** during pre-effective-date challenges (assuming the bill ships and is challenged before its first compliance cycle), Option B becomes a quick legislative response.
3. **If *Moore v. Harper*'s narrower-Elections-Clause dissent picks up a fifth vote** in some future composition. The hostile reviewer's litigation roadmap projects this is unlikely in the current 6–3 alignment but worth tracking.

A reserved Option B redraft of § 103 is preserved in our private working files; if any of the above triggers, the redraft is ~3 hours of work to produce.

## 3. Other open issues v0.1 mitigates but does not fully solve

### 3.1 § 110 state-constitutional integrity cap

The 1.5% tract-modification cap in § 110(c) is calibrated to permit small-county-line preservation without enabling swing-district reconstruction. The state-director review noted that some state-constitutional integrity rules (notably Iowa's "do not split counties unless absolutely necessary") are stricter than 1.5% can accommodate.

**Trade-off**: raising the cap weakens the bill's protection against integrity-pretext gerrymandering; lowering it forces state-constitutional preemption for more states. v0.1 chose 1.5% as the median between staff-drafter analysis (5% is gerrymandering-feasible) and state-director feedback (small-county preservation rarely exceeds 0.5%).

**Future fix**: tier the cap by state size — e.g., 1.0% for states with > 25 districts, 2.5% for states with ≤ 8 districts. Would require an additional finding in § 101 explaining the differentiation.

### 3.2 § 107(c)(7) TIGER errata mechanism scope

The errata mechanism authorizes the Director of Census to publish corrections to the tract adjacency graph for documented TIGER errors. The state-director review accepted this in principle but flagged that 60-day publication is too long for some errors discovered post-§-105(a)-deadline.

**v0.1 mitigation**: § 108(e) tolls the State publication clock during periods when the parameter table (which references the tract adjacency graph) is not in force. So an errata-pending State is not technically out of compliance.

**What remains unresolved**: a State that needs an errata correction but doesn't get one could be in compliance procedurally while producing a map the State knows is geographically wrong. The errata mechanism is currently optional from the State's perspective; making it mandatory (Director MUST publish or explain refusal in writing) would address this.

### 3.3 § 107(b) expert panel composition

Three members (graph-partitioning expert, election geographer, software-reproducibility expert), selected by NIST from National Academies nominees. Civil-rights counsel flagged that this composition has no slot for a VRA-expertise role and warned that a hostile administration could populate the panel with race-blind-formalism scholars to narrow the parameter table in ways that disfavor minority-friendly outcomes.

**v0.1 mitigation**: the parameter table cannot modify the algorithm structure (statutory) or the tolerance schedule (statutory). The Director sets only the seed and the binary SHA-256. Range of administrative manipulation is therefore narrow.

**What remains unresolved**: the seed CAN affect outcomes in close cases — particularly where partisan or racial sorting is borderline. A future amendment adding a fourth panel member with VRA / civil-rights expertise would be a defensible improvement; v0.1 omits it because the panel is otherwise narrow and tied to algorithmic technicalities.

### 3.4 § 105(e) data preservation period

12 years was chosen as 2 decennial cycles plus a buffer. The state-director review pointed out that storage costs at scale (TIGER files alone are 5–10 GB per cycle per state) are a real budget item. A shorter retention period (e.g., 8 years = current cycle + next cycle's drawing window) would reduce the burden but limit historical-record litigation.

v0.1 chose 12 years as the conservative default. This is a candidate for negotiation in committee mark-up.

### 3.5 § 107(e)(1)(B) citizen-standing language

v0.1 restricts citizen standing to "concrete injury beyond bare statutory violation, consistent with *TransUnion LLC v. Ramirez* (2021)." This is the right doctrinal framing but pragmatically narrows the bill's enforcement reach. A reform organization that wants broad enforcement should consider whether organizational standing under § 107(e)(1)(C) (revised to track *Hunt v. Washington Apple*) is sufficient compensation. Our drafting view: yes, but with the caveat that organizations have to run their own enforcement campaigns rather than relying on individual-citizen volunteers.

## 4. Issues v0.1 does NOT mitigate (acknowledged for completeness)

### 4.1 Article I § 4 limited to congressional districts

The bill addresses only U.S. House congressional districts. State legislative districts, where partisan gerrymandering is at least as severe, are entirely outside its reach. *Moore v. Harper* and *Wesberry* don't extend Article I § 4 to state-legislative line-drawing.

This is a **fundamental scope limitation**, not a drafting gap. A separate bill grounded in Section 5 of the Fourteenth Amendment plus *Reynolds v. Sims* one-person-one-vote would be needed. Out of scope here; flagged for future plan.

### 4.2 Communities of interest

Civic groups currently submit COI inputs at state public hearings. Under § 104(e), COI data cannot enter the algorithm. § 106(g) repurposes public hearings toward § 2 evidence-gathering, which is a real but narrower use case. Civic-engagement groups will see this as a loss, even though the underlying tradeoff (mechanical determinism vs. discretionary local input) is what makes the bill survive *Rucho*.

**Mitigation outside the bill**: `redist civic ingest` lets civic groups archive their COI submissions independently. The submissions become public-record artifacts but do not modify the federally-mandated map. Reform-organization messaging should emphasize this complementarity rather than pretending COI input affects the federal districts.

### 4.3 Computational-accessibility concerns

A small State's election office may lack the staff to run the algorithm + perform the § 106 review + defend it in litigation. § 107 designates a federally-maintained reference implementation, which lowers the technical bar substantially, but the legal-defense capacity remains a State responsibility. Expect a "small-state amendment" in mark-up adding federal technical-assistance funding for States below a population threshold.

## 5. The drafting team's residual judgment calls

For the v0.1 drafters: the following are choices that reasonable lawyers could make differently. They are documented here so a successor maintainer knows what was *chosen* (vs. what was *missed*).

1. **§ 106(b) "shall"**: chosen over "may" per civil-rights counsel non-negotiable. Trade-off: stronger VRA protection vs. more compelled-race-consciousness exposure under *Miller*. Hold the line at "shall."

2. **§ 106(d) Shaw-only presumption**: chosen to avoid imposing the rebuttable presumption against § 2 effect claims, where it would have been strongly plaintiff-disfavoring. Trade-off: a State drawing a § 2 modification still must defend it on the merits in § 2 cases (no presumptive validity), only against *Shaw*-trolls (which is the right asymmetry).

3. **§ 107(c) parameter table moved into statute**: chosen over Director-discretion to defuse non-delegation. Trade-off: harder to update parameters across cycles; locks the algorithm parameters until Congress amends. Acceptable cost for constitutional durability.

4. **§ 110 1.5% cap**: chosen as median between 0.5% (state-director min) and 5% (staff-drafter swing-district feasibility threshold). Open to negotiation in mark-up.

5. **Title rename to "Districting Integrity Act"**: chosen for political register. Long title preserves "Reproducible Congressional Districting Act." Trade-off: "Reproducibility" is the bill's load-bearing technical claim; some technical-audience readers may prefer the long title as the primary name.

6. **§ 109(d) dual-authority enactment**: chosen as belt-and-suspenders against a future Court narrowing the Elections Clause. Trade-off: invites § 5 Fourteenth Amendment challenges to provisions the Elections Clause comfortably supports. Net positive.

7. **No conditional-spending restructure (Option B deferred)**: chosen on political grounds (Option B weakens reform value if States opt out). Trade-off: 60–70% commandeering-strike risk per hostile-challenger projection. Mitigated by § 101(g) findings + § 109(d) dual-authority. Revisit if pre-mark-up outreach shows mandate posture is unworkable.

---

## See also

- `MODEL_FEDERAL_STATUTE.md` v0.1 — the bill text these notes annotate
- `STATUTE_RATIONALE.md` — policy memo
- `STATUTE_ONE_PAGER.md` — staff briefing version
- `PARTISAN_OPTIONS.md` — how the four partisan-input postures coexist
- `FAIRNESS_DOCTRINE.md` — state-court companion strategy
