# National Redistricting Standard v0.1

**Status:** Candidate specification
**Date:** 2026-07-09
**Scope:** Congressional redistricting baseline and review procedure
**Wave:** `context/waves/2026-07-09-national-standard-evidence-and-specification/`
**VTRACE posture:** `internal_engineering_baseline_only`
**Claim posture:** Internal proposal; not enacted law, legal certification,
external peer review, or a finding that any resulting plan is substantively
fair

## 1. Purpose

This specification defines a reproducible national redistricting procedure.
Its central object is not one legally privileged map. It is a public evidence
record containing:

1. a mandatory constitutional and data floor;
2. a partisan-input-excluded geographic benchmark;
3. any legally or publicly justified modifications to that baseline; and
4. evaluation of both the baseline and final plan.

The standard reduces hidden discretion by forcing important choices to be
versioned, executed reproducibly, and explained in public. It does not remove
legal judgment, guarantee politically symmetric outcomes, decide Voting Rights
Act liability, or identify a uniquely fair plan.

## 2. Normative Architecture

### Layer A - Mandatory floor

Every conforming record must establish:

- the legally authorized district count;
- complete assignment of the population-bearing geographic units;
- contiguity under the published adjacency definition;
- population equality as nearly exact as practicable;
- one published census release and one published geographic vintage;
- canonical unit identifiers and ordering;
- no partisan, racial, incumbency, candidate, or community signal in the
  baseline execution inputs; and
- a machine-verifiable manifest covering source, binary, inputs, parameters,
  assignments, analysis, and reports.

Layer A is a conformance floor. Failure is not cured by favorable political,
compactness, community, or demographic outcomes.

### Layer B - Geographic benchmark

The baseline is a mandatory evidence artifact and presumptive comparison
plan. It is not automatically the legally controlling final map.

The v0.1 baseline profile is:

| Component | Canonical choice |
|---|---|
| Structure | `standard-bisect` |
| Edge weights | `geographic` shared-boundary length |
| Search | `single` |
| Initial seed | Domain-separated SHA-256 of the canonical input manifest |
| Candidate selection | The single reference-engine result for the canonical seed |
| Population objective | Equal population as nearly exact as practicable |
| Political/racial/COI inputs | Prohibited from baseline generation |
| Engine | Versioned reference engine named by the implementation profile |

The benchmark openly encodes a geographic-compactness preference through
recursive splitting and shared-boundary weighting. Excluding partisan inputs
does not make these choices value-free. The benchmark exists to answer a
reproducible question:

> What plan results when the published reference procedure uses only the
> published population and geographic inputs?

It does not answer:

> What plan best satisfies every legal, representational, political, or
> community interest?

### Layer C - Governed modifications

A final plan must differ from the baseline when federal constitutional law,
the Voting Rights Act, or a court order requires a change. A final plan may
differ when a competent decision maker determines that a modification is
authorized by:

- controlling state constitutional or statutory criteria;
- correction of census, TIGER, topology, adjacency, or unit-assignment error;
- a publicly adopted community-of-interest criterion; or
- another authority expressly identified in the controlling legal profile.

The baseline is not the appropriate vote-dilution benchmark under
*Allen v. Milligan*. It is never evidence that Voting Rights Act obligations
have been satisfied, never evidence negating discriminatory purpose or effect,
and never a defense to a substantive claim.

Community-of-interest criteria must be published and locked before the
benchmark or candidate plans are generated, use a public evidentiary definition
rather than an algorithmic proxy, and receive disparate-impact review.
Good-faith recognition of a racial or language-minority community with shared
interests is permitted; the anti-proxy rule addresses pretextual partisan,
dilutive, candidate, or incumbency selection.

Every modification must produce a machine-readable baseline-to-final diff and
a public decision record. The record must identify:

- authority for the modification;
- evidence considered;
- units and population moved;
- districts affected;
- alternatives considered;
- for an authorized State-law, COI, or correction change, the precommitted
  departure measures and a reproducible lower-departure lawful alternative;
- why the selected authorized change was proportionate to its stated purpose;
- applicable effects on population, contiguity, compactness, subdivisions,
  communities, racial opportunity, and partisan metrics; and
- decision maker, date, legal profile version, and artifact hashes.

For a VRA-required modification, necessity and scope must be documented under
the controlling statutory and constitutional framework. The benchmark does not
create a proximity presumption. The comparator is a fully lawful alternative,
and additional lawful minority opportunity is not disfavored because it moves
more units. Public legal conclusions may reference protected expert appendices
available under court or reviewer protective order. The standard does not
convert a racial opportunity diagnostic into a legal conclusion.

Before adoption, the benchmark, proposed final plan, community submissions,
modifications, and alternatives must receive notice, public comment, and
independent adversarial review. Where race-conscious and State-authorized
partisan considerations affect overlapping geography, the record must
disentangle each authority, input, stage, decision maker, and effect. Separate
software runs alone do not satisfy that requirement; partisan evidence used for
racially polarized voting analysis is not prohibited optimization input.

The standard does not encode a fixed percentage cap on modified units. A
universal cap could prevent legally required changes in one jurisdiction while
allowing outcome-changing manipulation in another. Scope is instead reviewed
against the stated authority, alternatives, and effects.

### Layer D - Evaluation

Evaluation is diagnostic and adversarial. It does not select the final plan
automatically.

A conforming evaluation reports, where data are available:

- population deviation and contiguity;
- multiple geometric compactness measures;
- county, municipal, and other subdivision splits;
- submitter-defined community splits;
- minority population and opportunity-district diagnostics;
- disparate-impact evaluation of the benchmark and final plan;
- multiple partisan metrics across multiple elections;
- baseline-to-final differences;
- sensitivity to seed, geographic resolution, and parameter profile; and
- position within a converged ensemble of legally valid alternatives.

No single compactness, partisan, community, or demographic metric is designated
as the definition of fairness. Ensemble percentiles require archived traces,
convergence diagnostics, immutable election and metric inputs, and uncertainty
reporting.

Evaluation datasets, election selection rules, metric definitions, and
stopping rules must be precommitted before candidate plans are evaluated. A
jurisdiction may not select among evaluation profiles after observing which
profile makes its preferred plan appear least unusual.

## 3. Canonical Baseline Decisions

### 3.1 Structure: standard recursive bisection

`standard-bisect` is selected because it is the simplest structure already
used by the statute draft, is broadly implemented, and minimizes the number of
additional structural choices in the baseline.

`prime-factor` ApportionRegions remains an important research comparator. It is
not selected for v0.1 because the claimed Huntington-Hill derivation, repeated
prime ordering, prime-seat fallback, and full 50-state seed-stability evidence
remain contested or incomplete.

### 3.2 Weights: geographic only

Shared-boundary length is selected because it is public, geometric, and the
only weight profile documented as validated across all 50 states and three
census years.

County-sticky weighting is rejected as the national baseline because selecting
a county multiplier is a substantive policy choice. County preservation may
enter Layer C under controlling law or be reported in Layer D.

VRA-aligned and proportional weights are not baseline inputs. They remain
research or case-specific tools and must not be described as the v0.1
partisan-input-excluded geographic benchmark.

### 3.3 Search: one precommitted seed

A single content-derived seed is selected because the current standard-bisect
implementation executes one seed and does not implement convergence search for
that structure. The choice makes the benchmark reproducible but does not make
it seed-robust.

Layer D must therefore report seed sensitivity using a separately versioned
evaluation profile. Seed-sensitivity runs are diagnostics; they do not permit
the operator to replace the canonical result with a politically preferred
seed.

### 3.4 Seed: content-derived from the complete input manifest

The initial seed must be derived from:

```text
SHA-256(
  "NRS_BASELINE_V0_1" ||
  canonical_input_manifest_bytes
)
```

The canonicalization rule is `canonical-json-v1` as defined by
`docs/specs/2026-05-10-plan-audit-certificates.md`: UTF-8, lexicographically
sorted object keys, preserved array order, no insignificant whitespace, base-10
integers without leading zeros, finite shortest-roundtrip floats, and a
distinction between absent and explicit null fields.

The seed is the unsigned 64-bit little-endian integer represented by bytes 0
through 7 of the SHA-256 digest. Engines that require a narrower signed seed
must use a profile-defined, fixture-tested conversion and may not silently
truncate it.

The manifest includes only assignment-affecting fields: census release,
geographic vintage, unit index hash, population hash, adjacency hash, district
count, legal profile hash, algorithm profile hash, reference-engine hash, and
canonicalization version. Timestamps, paths, comments, report settings, and
other presentation metadata are excluded.

This is stronger than deriving the seed from a census release identifier alone:
any change to a map-producing input produces a different seed and a visible
manifest hash. All seed-affecting schemas, profiles, releases, and correction
rules must be fixed before redistricting census data are released. An errata
change requires a superseding profile record and preserves the original run.

This seed path is a target requirement and is not yet wired into the current
standard-bisect runner.

### 3.5 Resolution: blocks are normative; coarser units are computational

The final legal assignment, population accounting, and baseline computation
unit is the census block or another legally controlling unit of equivalent or
finer resolution.

Tracts or block groups may be used for exploratory or comparison runs only
when:

- the result is labeled nonconforming with the national benchmark profile; and
- the aggregation mapping and limitations are published and hashed.

The national block adjacency profile must specify water, island, enclave,
point-contact, sliver, zero-population-block, and documented bridge treatment.
The current tract-based BISECT workflow is therefore a research and comparison
implementation, not a full v0.1 conformance implementation.

### 3.6 Population equality

The controlling rule is equality as nearly exact as practicable, not a claim
that a fixed 0.5 percent deviation is always constitutionally sufficient.

The implementation profile may use an internal search tolerance, but the
published record must report:

- ideal district population;
- maximum and total deviation;
- whether a lower-deviation plan was found under the same legal criteria; and
- justification for any avoidable deviation.

### 3.7 Baseline posture

The baseline is:

- mandatory for the jurisdiction to generate using the published reference
  implementation;
- mandatory to publish;
- presumptive as a comparison and disclosure artifact; and
- rebuttable as the final map through Layer C.

It is not:

- automatically binding as the final map;
- a safe harbor from the Voting Rights Act;
- proof of nondiscriminatory effect;
- proof of partisan fairness;
- a substitute for public process; or
- a substitute for a court, commission, legislature, or special master.

## 4. Decision Matrix

| Layer | Question | Decision owner | Version owner | Challenge path |
|---|---|---|---|---|
| A | What constitutional and data rules must every plan satisfy? | Congress and controlling courts | Congress; courts through controlling precedent | Judicial review and conformance verifier |
| A | Which census/geographic release is authoritative? | Census Bureau under published rule | Census Bureau | Public errata petition and judicial review |
| B | What canonical benchmark profile is run? | Congress or other enacting authority | Normative profile text; assignment-changing revisions require the same authority | Public comment, legal challenge, independent rerun |
| B | Which implementation is authoritative? | Jurisdiction executes the published reference implementation | NIST/EAC technical custodian for ministerial patches only | Reproducibility challenge and defect process |
| C | Is a modification legally required or authorized? | State authority, commission, or court | Controlling legal profile | Administrative record and judicial review |
| C | Does a COI criterion apply? | Jurisdiction under published law/rule | Jurisdiction | Public hearing, evidence challenge, judicial review |
| D | Which diagnostics are reported? | Normative national reporting profile plus jurisdiction additions | NIST/EAC technical custodian for non-substantive corrections | Method challenge and corrected report |
| D | What legal significance does a metric have? | Court or other competent legal decision maker | Controlling law | Ordinary appellate review |

## 5. Versioning And Change Control

Every benchmark profile has:

- semantic version;
- effective census cycles;
- normative source commit;
- canonical schema versions;
- reference-engine source identity and hash;
- implementation-independent test vectors;
- change rationale;
- public-comment record; and
- migration and compatibility statement.

Patch versions may correct documentation or an implementation defect without
changing valid output. Minor or major versions that can change assignments require approval by the
profile's enacting authority and must be published before the redistricting
census release. Technical custodians may not alter structure, weights, search,
seed derivation, population rules, or other assignment-affecting parameters.

Until a public authority adopts the standard, this repository is the interim
custodian. Repository governance may advance candidate versions but may not
describe them as federally authoritative.

No profile may be changed retroactively for a completed census cycle except
through a published defect or errata process that preserves the original
artifacts.

## 6. Challenge And Errata Process

Any person may submit a challenge alleging:

- source or binary mismatch;
- incorrect input hash;
- census or TIGER vintage error;
- adjacency or topology defect;
- noncanonical parameter or seed use;
- nondeterministic output;
- incomplete baseline-to-final diff;
- unsupported evidence claim; or
- failure to disclose a required decision.

The responsible authority must publish:

- the challenge;
- machine-readable affected artifact identifiers;
- disposition and reasons;
- corrected artifacts when applicable; and
- whether prior outputs remain valid, are superseded, or require rerun.

Technical conformance findings do not decide VRA liability, constitutional
validity, discriminatory effect, or substantive fairness.

## 7. Conformance Artifacts

A conforming package contains the following logical records:

1. `standard_profile.json`
2. `legal_profile.json`
3. `input_manifest.json`
4. `unit_index.json`
5. `baseline_assignments.json`
6. `baseline_manifest.json`
7. `final_assignments.json`
8. `modification_record.json`
9. `evaluation_manifest.json`
10. `report_manifest.json`
11. `challenge_log.json`

Every JSON artifact must have a schema version, canonical content hash, creation
authority, source references, and verifier path.

### 7.1 Existing and planned physical artifacts

The logical records must reuse existing schemas and chains rather than create
parallel identities:

| Logical record | Existing or planned physical artifact |
|---|---|
| Standard profile | Planned `standard_profile.json` |
| Legal profile | Existing `legal-profile-v1`, extended only through its versioning rules |
| Input manifest | Planned assignment-affecting subset of the existing plan/provenance manifests |
| Unit index | Existing RPLAN unit index extended for block-level conformance |
| Baseline assignments | Existing canonical RPLAN assignment representation |
| Baseline manifest | Existing plan manifest plus provenance and reference-engine fields |
| Final assignments | Existing canonical RPLAN assignment representation |
| Modification record | Planned schema and verifier |
| Evaluation record | Existing `audit-certificate-v1` plus analysis evidence manifests |
| Report record | Existing report manifest |
| Challenge log | Planned append-only challenge schema |

`docs/file-formats/manifests.md` remains the manifest registry. New physical
artifacts are not active until that registry names their schema, canonical
fields, verifier path, and unknown-version rejection behavior.

Every manifest inherits the canonical field rules in
`docs/file-formats/manifests.md`, including build commit, tool version, compiler
version, creation time, and refusal of unknown schemas.

The conformance verifier must also check benchmark input purity: the profile
must bind geographic weights and must not bind partisan, racial, candidate,
incumbency, or COI columns to benchmark generation. Until this verifier exists,
input purity is declared and inspectable but not machine-certified.

## 8. Required Conformance Test Vectors

The implementation-independent fixture suite must include:

| Fixture | Required behavior |
|---|---|
| Single-district jurisdiction | All units assigned to one district with no partition call |
| Even district count | Deterministic equal-seat recursive structure |
| Odd district count | Deterministic floor/ceiling seat allocation at each node |
| Disconnected input | Explicit failure or versioned bridge correction; never silent success |
| Island/water geography | Published adjacency treatment and stable expected hash |
| Population boundary case | Exact acceptance/rejection behavior around configured tolerance |
| Seed derivation | Known manifest bytes produce a known seed |
| Search mode | Canonical profile executes exactly one seed |
| Canonicalization | Key/order/whitespace variants produce the same canonical hash |
| Tampered input | Verifier rejects mismatched hash |
| Baseline modification | Diff identifies every moved unit and affected metric |
| Legal-profile mismatch | Verifier refuses to certify conformance |
| Engine comparison | Different engine is labeled comparative, not reference-engine-equivalent output |

Positive and negative fixtures are required. Fixtures must be small enough for
an outside verifier to inspect manually. Derivation, canonicalization, and
verifier fixtures are implementation-independent; full assignment fixtures are
pinned to the named reference-engine source hash.

Reference conformance requires functional equivalence: canonical assignment and
record hashes must match. It does not require independently built executable
files to be byte-identical.

### 8.1 Seed derivation vector NRS-SEED-001

Canonical input bytes:

```json
{"adjacency_sha256":"00","algorithm_profile_sha256":"11","canonicalization_version":"canonical-json-v1","census_release":"test","district_count":2,"geographic_vintage":"test","legal_profile_sha256":"22","population_sha256":"33","reference_engine_sha256":"44","unit_index_sha256":"55"}
```

After prefixing the ASCII bytes `NRS_BASELINE_V0_1`, the expected values are:

```text
sha256 = e50326ede53a03cd59ffe98bb95ff04e784ad607bdd242ebda4f927b0decf690
seed_u64_little_endian = 14772715961905972197
```

Changing object-key order, adding whitespace, or changing presentation-only
metadata before canonicalization must not change the expected seed. Changing an
assignment-affecting value must change it.

## 9. Claim Boundary

Conformance supports only the following claims:

- the named procedure was executed;
- the named public inputs and parameters were used;
- the artifacts have the recorded identities and relationships;
- the reported mathematical checks were performed; and
- the baseline-to-final changes were disclosed.

Conformance does not establish:

- that the final plan is fair;
- that partisan advantage was absent;
- that racial vote dilution was absent;
- that a VRA modification was legally required or sufficient;
- that communities were adequately represented;
- that the algorithm found a global optimum;
- that the profile is constitutionally compelled; or
- that a court must adopt the plan.

## 10. Rejected v0.1 Alternatives

| Alternative | Decision | Reason |
|---|---|---|
| Binding algorithmic final map | Rejected | Suppresses legal/community judgment and increases constitutional and legitimacy risk |
| `prime-factor` national baseline | Deferred comparator | Distinctive theory but incomplete validation and fallback questions |
| County-sticky baseline weights | Rejected | Encodes a contestable policy preference and multiplier |
| Convergence search | Deferred from canonical profile | Current standard-bisect path does not implement it; threshold evidence remains incomplete |
| Partisan metric threshold as fairness rule | Rejected | No accepted single metric and high Goodhart risk |
| Fixed cap on baseline modifications | Rejected | Can obstruct legal compliance or permit strategically concentrated changes |
| Tracts as universally final legal units | Rejected | May prevent equality as nearly exact as practicable |
| Algorithm-only community proxy | Rejected | A proxy is not a community and may carry demographic effects |

## 11. Implementation Status

BISECT already implements substantial pieces of Layers A, B, and D, including
graph partitioning, deterministic seed infrastructure, manifests, provenance,
analysis, reports, and verification.

This specification intentionally identifies gaps rather than treating existing
behavior as automatically conforming. The governing wave must still:

- implement the manifest-derived seed and block-level benchmark path;
- register planned schemas and build a package conformance verifier;
- reconcile flagship claims;
- create a clean reference replay;
- archive real ensemble evidence;
- align the model statute; and
- obtain external replication.
