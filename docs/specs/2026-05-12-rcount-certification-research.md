# Research Roadmap: RCOUNT Election Certification And Audit Algorithms

**Status:** Draft research and implementation roadmap  
**Date:** 2026-05-12  
**Depends on:** [`2026-05-12-rcount-incubation.md`](2026-05-12-rcount-incubation.md)  
**Purpose:** Map current election certification and audit practice into RCOUNT
algorithms that can be implemented in Rust.

## Scope Note

Election certification is state-specific. This document records cross-state
patterns from official and institutional sources, then translates those patterns
into algorithmic checks. It is not a substitute for a jurisdiction-by-
jurisdiction legal survey.

## What Certifiers Must Establish

Across jurisdictions, certification is not usually a single mathematical proof.
It is a legal canvass: officials compile returns, validate the outcome, resolve
known categories of ballot status, complete required audits or recounts, and
attest that the official results are ready to certify.

The U.S. Election Assistance Commission describes the process as the path from
unofficial to official results and points election officials to canvass
checklists and best practices for certification communication and post-election
work: <https://www.eac.gov/election-officials/election-results-canvass-and-certification>.

The concrete proof obligations vary by state, but the recurring evidence
families are:

| Evidence family | What the certifier is trying to establish |
|-----------------|-------------------------------------------|
| Eligibility/accounting | Eligible voter universe, voter check-ins, ballots issued, ballots returned, accepted ballots, rejected/provisional/spoiled categories. |
| Tabulation | Vote totals by contest, choice, precinct/reporting unit, batch, and jurisdiction. |
| Canvass corrections | Late ballots, adjudications, cured ballots, recount deltas, duplicate resolution, and official adjustments. |
| Audit/recount completion | Required post-election audits, hand checks, risk-limiting audits, or recounts are complete before certification where state law requires it. |
| Chain of custody | Ballots, batches, scanners, containers, manifests, and audit samples can be traced. |
| Certification attestation | Officials sign or transmit the official results under the relevant state procedure and deadline. |

## What Systems Are Used Now

Current election administration uses a mixture of:

- voting systems certified at the federal or state level;
- election management systems for ballot definition, tabulation, reporting, and
  result exports;
- pollbooks and voter registration systems for check-in and voter-credit data;
- ballot manifests that describe where ballots are stored;
- cast vote record exports where voting systems support them;
- canvass systems, spreadsheets, PDFs, state reporting systems, and public
  results portals;
- RLA tools such as Arlo or state-provided audit software.

The EAC operates a voting system testing and certification program that tests
hardware and software, accredits labs, and may decertify systems. State
participation is voluntary at the federal level, though some states require
some level of participation by law or regulation:
<https://www.eac.gov/voting-equipment/system-certification-process>.

The EAC also publishes certified voting systems and explains that HAVA requires
the EAC to accredit voting system test laboratories and certify voting
equipment; systems are tested against VVSG requirements:
<https://www.eac.gov/voting-equipment/certified-voting-systems>.

NIST develops conformance test suites for use in the EAC testing program:
<https://www.nist.gov/itl/voting/test-development-voting-systems/testing-faqs-hava-certification-and-testing>.

## Current Audit Patterns To Encode

### Ballot Manifests

Colorado's RLA FAQ describes a ballot manifest as a county-created description
of how many ballot cards were counted, organized by batches and ballot counts:
<https://www.coloradosos.gov/pubs/elections/RLA/faqs.html>.

California's RLA regulations require an accurate ballot manifest created without
reliance on the voting system, and require it to identify the storage container
for each validly cast ballot card after tabulation:
<https://www.sos.ca.gov/administration/regulations/current-regulations/elections/risk-limiting-audits>.

RCOUNT implication: ballot manifests are first-class inputs, not comments.
They need canonical row hashing, batch counts, container ids, and independent
source provenance.

### CVR And Summary Reconciliation

Colorado Rule 25 requires counties conducting comparison audits to verify that
the number of individual CVRs equals the aggregate ballot-card count in the
manifest and that CVR vote totals equal the summary results report. The rule
also requires hashing the CVR export and uploading verified hashes:
<https://www.law.cornell.edu/regulations/colorado/8-CCR-1505-1-25>.

RCOUNT implication: a core verifier should compare:

```text
manifest ballot-card count
  == CVR row/card count
  == summary tabulation ballot-card count
```

and:

```text
sum(CVR choices by contest)
  == summary results by contest
  == certified totals before/after canvass deltas
```

### Public Random Seeds

Colorado's RLA process uses a public seed generated by rolling 20 ten-sided
dice, then uses audit software to sample ballots from the manifests:
<https://www.coloradosos.gov/pubs/elections/RLA/faqs.html>.

Colorado Rule 25 references a SHA-256 pseudo-random generator and requires the
seed to be published immediately after it is established:
<https://www.law.cornell.edu/regulations/colorado/8-CCR-1505-1-25>.

RCOUNT implication: sample selection should be independently replayable from:

```text
ballot_manifest_hash
target_contests
risk_limit
public_seed
sampling_algorithm_id
```

### Public Algorithms And Source

California's RLA regulations require RLA software algorithms and source code to
be disclosed publicly:
<https://www.sos.ca.gov/administration/regulations/current-regulations/elections/risk-limiting-audits>.

RCOUNT implication: audit algorithms must be named, versioned, deterministic,
and open enough that third parties can reproduce certificates without trusting a
vendor UI.

### Risk-Limiting Audit Semantics

NCSL summarizes RLAs as statistical audits designed to limit the risk of
certifying an incorrect outcome; tighter races require more ballots to be
audited. It also notes that some states require audits to be completed before
canvass or certification steps:
<https://www.ncsl.org/elections-and-campaigns/post-election-audits>.

RCOUNT implication: RCOUNT should not only say "sample checked." It should
store the target contest, risk limit, margin, sample size, stopping rule, and
whether the audit met the risk limit or escalated.

## Algorithm Suite

The first serious RCOUNT implementation should expose these algorithms as
library functions with CLI wrappers.

| Algorithm | Crate | Inputs | Output |
|-----------|-------|--------|--------|
| Canonical count hash | `rcount-core` / `rcount-io` | `.rcount` document | global root plus typed child roots |
| Manifest verifier | `rcount-audit` | ballot manifest, container/batch rows | count totals, duplicate/missing-row failures |
| CVR-summary reconciler | `rcount-audit` | CVR export, summary report, manifest | row-count and vote-total reconciliation certificate |
| Canvass delta verifier | `rcount-audit` | preliminary totals, canvass changes, certified totals | arithmetic proof of official totals |
| Unit-lineage verifier | `rcount-audit` | prior/current precinct universes and lineage rows | split/merge/rename/boundary-change report |
| Receipt inclusion verifier | `rcount-audit` | privacy-safe receipt token, Merkle proof, public root | inclusion result without vote-choice proof |
| RLA sampler replay | `rcount-audit` | manifest hash, seed, contest, risk limit, algorithm id | selected ballot/batch list |
| RLA stopping verifier | `rcount-audit` | sample observations, margin, risk limit, method | pass/escalate/full-hand-count status |
| Plan-linked aggregator | `rcount-rplan` | RPLAN/RCTX, count package, crosswalk | district vote totals bound to plan/context hashes |

## Rust Advantage

Rust is a good fit because these checks are:

- deterministic;
- hash-heavy;
- schema-heavy;
- parallelizable by precinct, batch, contest, and county;
- sensitive to integer overflow and type confusion;
- well suited to CLI tools that emit stable JSON certificates.

The claim should not be "Rust replaces certification." The useful claim is:
Rust can make the arithmetic, hashing, replay, and reconciliation layers fast,
public, deterministic, and independently checkable.

## Candidate CLI Surface

```text
rcount hash --count election.rcount
rcount verify --count election.rcount --certificate count-certificate.json
rcount reconcile-cvr --manifest manifest.csv --cvr cvr.csv --summary summary.csv
rcount verify-canvass --prelim prelim.json --deltas canvass.json --certified certified.json
rcount replay-rla --manifest manifest.csv --seed 123... --contest CONTEST --risk-limit 0.05
rcount verify-receipt --count election.rcount --receipt TOKEN --proof proof.json
rcount aggregate-plan --plan plan.rplan --context context.rctx --count election.rcount --crosswalk crosswalk.json
```

## Research Checklist

- [ ] Build a state-by-state certification matrix: local canvass deadline,
      state certification deadline, required audit type, recount triggers,
      public result format, and public data availability.
- [ ] Inventory voting-system export formats from EAC-certified systems where
      public docs exist: CVR exports, summary reports, ballot manifests, audit
      logs, and results XML/CSV.
- [x] Select three model jurisdictions for fixtures: one Colorado-style RLA,
      one California-style RLA, and one ordinary canvass/manual-audit state.
      Initial Colorado-style synthetic adapter fixture landed as
      `docs/examples/rcount-golden-packages/colorado-rla`; initial
      California-style public audit software fixture landed as
      `docs/examples/rcount-golden-packages/california-rla`; ordinary
      canvass/manual-audit fixture landed as
      `docs/examples/rcount-golden-packages/manual-audit`.
- [ ] Define RCOUNT v0.1 canonical JSON and CSV ingest rules.
- [ ] Implement aggregate arithmetic before ballot-level support.
- [ ] Add privacy review before any voter-facing receipt proof ships.
- [ ] Keep all legal claims jurisdiction-qualified.

## Non-Goals

RCOUNT should not:

- store personally identifiable voter records by default;
- let a voter prove plaintext candidate selections to a third party;
- claim to detect every form of fraud;
- certify voting equipment;
- replace human ballot inspection, canvass boards, courts, or statutory
  certification officers.

RCOUNT should:

- make public totals reproducible;
- make tampering with published ledgers detectable;
- make precinct and batch arithmetic machine-checkable;
- make RLA sampling replayable;
- make plan-linked district totals traceable to plan and count hashes.
