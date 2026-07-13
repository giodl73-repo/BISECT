# Rationale For A Federal Benchmark And Disclosure Standard

**Status:** v0.2 candidate, 2026-07-10
**Companion:** `MODEL_FEDERAL_STATUTE.md`

## 1. Why Congress

*Rucho v. Common Cause* rejected an open-ended federal constitutional test for
partisan fairness while leaving Congress free to enact positive law governing
congressional elections. Congress already regulates district count,
single-member districts, and population equality.

The proposal supplies a national evidence and process floor rather than a
constitutional definition of fair outcomes.

## 2. Why A Benchmark

Compactness, community preservation, partisan symmetry, minority opportunity,
and subdivision integrity can conflict. Hiding those choices inside a human
map, an algorithm, or a commission does not make them disappear.

A published benchmark does three useful things:

1. fixes one comparison procedure before outcomes are observed;
2. separates reproducible execution from legal judgment; and
3. makes every baseline-to-final change visible.

The benchmark is not self-executing law. A final legal authority remains
responsible for VRA, constitutional, State-law, and public-process decisions.

## 3. Why The Candidate Profile

### Standard bisection

Standard floor/ceiling recursive bisection is simpler to specify than
prime-factor or ratio-search structures. It minimizes structural choices
without claiming that binary recursion is value-free.

### Geographic weights

Shared-boundary length is public, geometric, and already broadly validated.
County weights encode an additional policy preference and therefore belong in
State law or the modification layer, not the national benchmark.

### One manifest-derived seed

A complete input-manifest hash prevents post hoc seed shopping if all
seed-affecting rules are fixed before the census release. Seed sensitivity is
reported as a diagnostic; operators may not replace the benchmark with a
preferred seed.

### Blocks

Congressional equality requires the finest practical population units. Tracts
are useful for research and smoke tests but are too coarse to serve as the
universal final legal unit.

## 4. Why Not Mandate The Benchmark Map

A final-map mandate would:

- treat legal and community judgment as defects rather than obligations;
- increase Elections Clause and anti-commandeering risk;
- convert a geometric profile into a hidden substantive fairness rule;
- invite VRA safe-harbor arguments; and
- reduce legitimacy when communities can identify harms but cannot obtain a
  reasoned modification.

Mandatory benchmark publication plus mandatory reasons captures most of the
anti-manipulation value while preserving lawful final-plan responsibility.

## 5. VRA And Community Layer

VRA-required changes are mandatory. The benchmark is never evidence of
compliance and never a defense.

Public VRA conclusions may rely on protected or in-camera expert appendices.
Required disclosure is not itself an admission of racial predominance. Where
race-conscious and State-authorized partisan considerations overlap, the record
must disentangle each authority, input, stage, decision maker, and effect;
separate software runs are not enough.

Community criteria are permitted only when defined and locked before the
benchmark or candidate maps are generated. They require a public evidentiary
record and disparate-impact review. Good-faith racial or language-minority
communities may qualify through shared interests; a computed socioeconomic
proxy is not automatically a community.

Authorized State-law, COI, and correction changes require a reproducible
lower-departure alternative, affected units and population, metric effects,
public comments, and reasons. Federally required remedies are not disfavored
for moving farther from the race-excluded benchmark.

## 6. Governance

- Congress owns assignment-affecting statutory rules.
- Census owns census/geographic releases and corrections.
- NIST owns technical conformance and canonicalization.
- EAC is the funded lead agency and owns the reference service, publication,
  independent review, State and community assistance, grants, comments, and
  challenge intake.
- A balanced advisory board supplies technical, State, civil-rights, community,
  civic, and cross-partisan review.

Technical custodians cannot change assignment-affecting rules.

## 7. Constitutional Postures

### Primary

An Elections Clause disclosure and process floor. The State retains final line
drawing.

### Federal backstop

A federal service may publish the benchmark when a State does not, without
adopting the final map.

### Preemption

A congressional plan lacking the required public record is ineffective after a
cure period.

### Commission support

Grants, fee waivers, and technical assistance help States, commissions,
courts, community organizations, and qualified challengers meet and review the
standard.

### Conditional funding

If direct State duties are invalidated, future election-administration funds
may carry voluntary, noncoercive benchmark and disclosure conditions.

## 8. Technical Readiness

Implemented:

- Rust partitioning and analysis;
- fixed-seed config and provenance;
- canonical assignment output;
- label SHA chain;
- RPLAN/RCTX and audit certificates;
- a two-run Rhode Island reference replay; and
- a real RI/IA/NC Rust/GerryChain ensemble package.

Incomplete:

- block-level national execution;
- manifest-derived benchmark seed;
- full package conformance CLI;
- independent non-author replay;
- release-grade data custody; and
- legislative or judicial acceptance.

## 9. What This Rationale Does Not Claim

- The benchmark is fair.
- One algorithm is constitutionally compelled.
- The benchmark eliminates manipulation.
- Reproducibility proves discriminatory intent or effect.
- Ensemble percentiles determine legality.
- Current BISECT output is ready for statutory designation.
