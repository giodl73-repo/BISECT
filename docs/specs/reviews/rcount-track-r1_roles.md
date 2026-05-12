# RCOUNT Track R1 Role Review

reviewer: ROLE PANEL
date: 2026-05-12
scope: `research/tracks/V-election-audit/README.md`
roles: BOUNDARY, WARD, COVENANT, CANVASS, VAULT, CONTOUR, MERIDIAN, TALLY, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS, LEDGER, SURVEY, TRENCH

## Verdict

The V track is approved as a roadmap, not yet as an implementation spec.
The central framing is correct: RCOUNT should be to election-count packages
what RPLAN is to district plans. It should define neutral package formats,
canonical hashes, verifier paths, and evidence boundaries.

The current README is intentionally thin. Before crate work starts, V.0 needs a
real substrate spec that separates:

- canvass/legal status from machine arithmetic;
- ballot accounting from vote totals;
- public inclusion proofs from ballot-choice receipts;
- RCOUNT package verification from official certification;
- RPLAN district aggregation from the base count package.

## Role Scores

| Role | Score | Finding |
|---|---:|---|
| BOUNDARY | 3.0/4 | Correctly avoids legal overclaim, but election-law certification standards need jurisdiction hooks. |
| WARD | 2.8/4 | State/local certification variation is not yet modeled. |
| COVENANT | 3.3/4 | Strong audit instinct; canonical package hashes and source custody must be specified. |
| CANVASS | 2.5/4 | Roadmap names certification, but lacks unofficial/canvassed/recounted/amended/certified state model. |
| VAULT | 2.5/4 | The "prove inclusion, not candidate choices" rule is right; threat model and receipt-risk controls are pending. |
| CONTOUR | 2.8/4 | Needs source-data provenance for voter file, precinct, CVR, ballot manifest, and statement-of-vote inputs. |
| MERIDIAN | 3.0/4 | RPLAN dependency boundary for district vote aggregation is plausible. |
| TALLY | 2.5/4 | CVR, ballot, ballot-card, contest, selection, batch, and reporting-unit semantics are not yet defined. |
| BENCHMARK | 2.7/4 | Needs tiny executable fixtures for every reconciliation equation and privacy boundary. |
| SCALE | 2.8/4 | RLA and statistical audit claims need strict confidence/risk-limit language. |
| PRECINCT | 3.0/4 | Precinct lineage is named; political interpretation should remain outside count verification. |
| DATUM | 3.0/4 | Clear research track; needs falsifiable paper claims and evidence ladders. |
| COMMONS | 3.2/4 | Voter-facing rule is strong; inclusion proofs must be explainable without exposing choices. |
| LEDGER | 3.0/4 | Interop topic is named; NIST CDF/vendor/state export compatibility must be scoped. |
| SURVEY | 2.8/4 | Operational adoption depends on fitting election-office public-record workflows. |
| TRENCH | 2.6/4 | Failure modes are rich and need a first-class threat/pitfall ledger. |

Overall: 46.5 / 64. Roadmap accepted; V.0 substrate spec required before code.

## P0 Boundaries

### P0-A: RCOUNT must not certify elections

RCOUNT can verify package consistency, arithmetic, lineage, hashes, and replay
rules. It cannot itself certify an election. Certification is an official legal
act by the appropriate canvassing body or officer.

Required language for V.0:

```text
RCOUNT verifies declared evidence packages. It does not replace official
canvass, recount, judicial, or certification authority.
```

### P0-B: No vote-choice receipts

The voter-facing proof model must prove inclusion or non-exclusion of an
eligible counted record without letting a voter prove a candidate selection to
a coercer, buyer, employer, family member, campaign, or state actor.

Forbidden in baseline RCOUNT:

- public hash directly computed from voter identity plus candidate choice;
- voter-held artifact that proves a specific candidate selection;
- small-cell publication that lets rare ballot styles or write-ins identify a
  voter;
- "global hash includes my vote" claims unless the proof is receipt-safe.

### P0-C: Legal status changes are events, not overwrites

Election records legitimately change. Late mail ballots, provisional rulings,
cure periods, duplicated ballots, recounts, court orders, and corrected
canvasses must appear as typed events with source evidence. A changed total is
not automatically a tamper signal; an unexplained changed total is.

## P1 Spec Requirements

### P1-A: Count package layers

V.0 should define these package layers:

| Layer | Role | Examples |
|---|---|---|
| Source evidence | Raw public inputs | vendor export, statement of votes, ballot manifest, CVR, canvass report |
| Normalized evidence | Parsed canonical rows | contests, candidates, batches, reporting units, totals |
| Reconciliation ledger | Equations and deltas | ballots accepted = counted + rejected by reason; precinct sum = contest total |
| Status ledger | Human/legal events | unofficial, canvassed, recounted, amended, certified |
| Public verification | Stable hashes/proofs | package hash, batch hash, inclusion proof, replay transcript |

### P1-B: Canonical identity

RCOUNT needs a versioned canonical byte projection before hashes are meaningful.
At minimum:

- `rcount_version`;
- domain-separated hash prefixes;
- sorted object keys and stable numeric/string encodings;
- explicit excluded volatile metadata;
- source-file hash versus normalized-record hash;
- package hash versus content hash.

### P1-C: Reporting-unit model

The spec must define precincts without assuming every count is precinct-native.
RCOUNT should support:

- precinct;
- split precinct;
- vote center;
- central-count batch;
- absentee/mail batch;
- provisional batch;
- district aggregation using RPLAN;
- jurisdiction-wide contest totals.

### P1-D: Ballot and contest semantics

TALLY blocks implementation until the spec distinguishes:

- ballot versus ballot card;
- CVR row versus counted ballot;
- contest versus selection;
- candidate, write-in, undervote, overvote, blank;
- batch count versus vote total;
- accepted, rejected, cured, duplicated, adjudicated, recounted.

### P1-E: RPLAN dependency boundary

V.8 should depend on RPLAN/RCTX for district vote aggregation. The base RCOUNT
package should not require a district plan. This keeps countywide certification
and precinct totals usable without redistricting artifacts while allowing
district totals to be replayed against a specific plan package.

## P2 Paper Roadmap Adjustments

The V-paper list is good. Suggested staging:

1. `V.0`: substrate overview and package anatomy.
2. `V.1`: canvass arithmetic and legal-status ledger.
3. `V.3`: tamper-evident precinct/batch hashing, with VAULT threat model.
4. `V.5` and `V.6`: ballot manifests and CVR-to-summary reconciliation.
5. `V.4`: privacy-safe voter inclusion proofs only after VAULT signs off.
6. `V.8`: district vote aggregation with RPLAN.
7. `V.7`: RLA replay once the manifest and tally substrate is stable.
8. `V.9-V.11`: interop, certification matrix, and Rust performance.

## Acceptance Checklist For V.0

- Defines the package layers and file roles.
- Names non-goals: official certification, malware detection, ballot-marking
  correctness, and proof of individual vote choice.
- Defines the synthetic validation harness: state, precinct lineage, multiple
  elections, positive fixtures, negative fixtures, and test levels.
- Gives at least three tiny fixtures:
  - precinct summary arithmetic;
  - corrected canvass event;
  - receipt-safe inclusion proof sketch.
- States canonical hash rules.
- Separates raw source hashes from normalized hashes.
- Defines reporting units and ballot/contest terms.
- Shows how V.8 will attach RPLAN without making RPLAN mandatory.
- Includes a threat model reviewed by VAULT.

## Role Additions

This review added three RCOUNT-specific roles:

- `.roles/canvass.md`
- `.roles/tally.md`
- `.roles/vault.md`

Existing roles remain useful, especially COVENANT, LEDGER, BENCHMARK, SCALE,
DATUM, COMMONS, and TRENCH. The new roles are necessary because election-count
packages have different failure modes than redistricting plans: lawful totals
change during canvass, voting-system exports carry subtle semantics, and public
verification can accidentally create vote-buying or coercion receipts.
