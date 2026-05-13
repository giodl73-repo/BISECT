# RCOUNT Validation Data Landscape

**Status:** Draft validation roadmap
**Date:** 2026-05-13
**Track:** `research/tracks/V-election-audit`
**Related specs:** [`2026-05-12-rcount-incubation.md`](2026-05-12-rcount-incubation.md),
[`2026-05-12-rcount-substrate.md`](2026-05-12-rcount-substrate.md),
[`2026-05-12-rcount-certification-research.md`](2026-05-12-rcount-certification-research.md),
[`2026-05-12-v-election-audit-paper-track.md`](2026-05-12-v-election-audit-paper-track.md),
[`2026-05-13-rcount-audit-algorithm-roadmap.md`](2026-05-13-rcount-audit-algorithm-roadmap.md)

## Purpose

This document identifies external standards, tools, algorithms, and public data
sources that can validate RCOUNT beyond synthetic fixtures.

RCOUNT should not only verify hand-written examples. It should prove that its
record model can ingest real election artifacts, preserve their source bytes,
normalize them into typed count records, and reproduce the mechanical claims
those artifacts support.

This roadmap is about validation of the RCOUNT substrate. It is not a claim that
any dataset proves election legality, voter eligibility, ballot-chain custody,
or machine security by itself.

## Validation Question

For each external source, ask four questions:

1. What public artifact is available?
2. Which RCOUNT records can it populate?
3. Which RCOUNT equations can it verify?
4. Which claims remain outside the source's evidence boundary?

The useful target is a package whose limitations are explicit:

```text
raw source artifact
  -> source hash
  -> adapter transcript
  -> normalized RCOUNT records
  -> verifier transcript
  -> claim boundary
```

## Standards And Schemas

| Source | Fit | RCOUNT use | Boundary |
|--------|-----|------------|----------|
| NIST Election Results Reporting CDF | High | V.9 adapters for CDF-shaped election, contest, reporting-unit, selection, and count records. | CDF conformance does not prove source completeness or legal certification. |
| NIST CDF implementation guidance | High | Field-coverage matrix for adapters and negative fixtures for unsupported fields. | Guidance is not a state-specific source of legal obligations. |
| NIST Election Event Logging CDF | Medium | Future canvass/status/event provenance and election-management event records. | Event logs may be incomplete, proprietary, or unavailable in public releases. |
| State RLA rules and protocols | High | V.5, V.7, and V.10 checks for ballot manifests, seeds, audit reports, and stopping evidence. | RCOUNT can replay published artifacts, not replace audit boards or state law. |

Primary references:

- NIST Election Results Reporting CDF:
  <https://www.nist.gov/itl/voting/interoperability/election-results-reporting-cdf>
- NIST Election Results CDF revision 2.0:
  <https://www.nist.gov/publications/election-results-common-data-format-specificationrevision-20>
- NIST CDF implementation guidance:
  <https://nvlpubs.nist.gov/nistpubs/gcr/2024/24-058/NIST.GCR.24-058.html>
- NIST Election Event Logging CDF:
  <https://pages.nist.gov/ElectionEventLogging/>

## Tools And Algorithms

| Tool or algorithm | Fit | RCOUNT comparison point | First validation use |
|-------------------|-----|-------------------------|----------------------|
| VotingWorks Arlo | Very high | RLA workflow, audit reports, ballot/batch retrieval, discrepancy recording. | Import Arlo-style audit reports and compare RCOUNT RLA transcripts. |
| Colorado RLA / CORLA-style process | Very high | Public seed, ballot manifest, CVR hash, sample replay, jurisdiction method metadata. | Extend existing Colorado-style fixture toward real public audit artifacts. |
| Rhode Island RLA materials | Very high | Ballot manifests, CVR files, audit reports, retrieval files, tally sheets, protocols. | First real-world V.7/V.10 validation package. |
| SHANGRLA | High | General RLA assertion framework. | Compare RCOUNT RLA stopping evidence and risk calculations to known method families. |
| R2B2 / Minerva / PROVIDENCE | Medium-high | Ballot-polling and round-by-round RLA methods. | Add alternate RLA method fixtures after Arlo/Colorado/Rhode Island are stable. |
| RAIRE | Medium | Risk-limiting audits for IRV elections. | Future ranked-choice extension; not needed for the first RCOUNT core. |
| ElectionGuard | Medium | End-to-end verifiable election artifacts and voter-facing verification boundaries. | Long-term V.4/V.10 comparison for inclusion proofs and cryptographic claim boundaries. |

The algorithm implementation roadmap now lives in the Algorithm Atlas:

- [V.12 BRAVO ballot-polling](../algorithm-atlas/v12-bravo-ballot-polling.md);
- [V.13 Minerva/Athena ballot-polling](../algorithm-atlas/v13-minerva-athena-ballot-polling.md);
- [V.14 Kaplan-Markov/MACRO comparison](../algorithm-atlas/v14-kaplan-markov-macro-comparison.md);
- [V.15 ALPHA betting martingales](../algorithm-atlas/v15-alpha-betting-martingale.md);
- [V.16 SHANGRLA assorters](../algorithm-atlas/v16-shangrla-assorters.md);
- [V.17 stratified/hybrid RLAs](../algorithm-atlas/v17-stratified-suite-hybrid.md);
- [V.18 batch comparison](../algorithm-atlas/v18-batch-comparison.md);
- [V.19 RAIRE/AWAIRE for RCV/IRV](../algorithm-atlas/v19-raire-awaire-rcv.md);
- [V.20 Bayesian tabulation audits](../algorithm-atlas/v20-bayesian-tabulation-audits.md);
- [V.21 SOBA observable ballot-level audits](../algorithm-atlas/v21-soba-observable-ballot-audits.md);
- [W.01 forensic anomaly analytics](../algorithm-atlas/w01-election-forensic-analytics.md).

### V.14 MACRO Validation Boundary

The published MACRO formula source is sufficient to validate the exact rational
product primitive in `rcount-stats`, and the synthetic
`synthetic_kaplan_markov_macro_package` fixture validates RCOUNT's package
schema, IO, and replay path. A public end-to-end V.14 validation package still
requires an artifact set that exposes all of:

- ballot count `N`;
- reported margin `V`;
- gamma;
- ordered sampled ballot or batch overstatement categories.

The currently indexed public audit sources are excellent for audit workflow,
source hashing, manifests, CVRs, and ballot-polling reports, but they do not yet
provide a ready MACRO comparison transcript with that full tuple. Until such a
source is added, V.14 external validation remains a documented boundary rather
than a claimed public replay.

Primary references:

- Arlo documentation: <https://docs.voting.works/arlo>
- Arlo source: <https://github.com/votingworks/arlo>
- Arlo audit report guide:
  <https://docs.voting.works/arlo/resources/audit-report-guide>
- Colorado RLA FAQ:
  <https://www.coloradosos.gov/pubs/elections/RLA/faqs.html>
- Rhode Island RLA Center:
  <https://elections.ri.gov/elections/risk-limiting-audit-center>
- ElectionGuard source:
  <https://github.com/Election-Tech-Initiative/electionguard>
- ElectionGuard technical paper:
  <https://www.usenix.org/conference/usenixsecurity24/presentation/benaloh>
- PROVIDENCE / R2B2 paper:
  <https://arxiv.org/abs/2210.08717>
- RAIRE paper:
  <https://arxiv.org/abs/1903.08804>

## Dataset Targets

| Priority | Dataset family | Available artifacts | Validates | Does not validate |
|----------|----------------|---------------------|-----------|-------------------|
| P0 | Rhode Island RLA public data | Ballot manifests, CVR files, audit reports, retrieval files, tally sheets, protocols. | Ballot-manifest accounting, CVR-summary reconciliation, audit transcript capture, source hashing, evidence matrix. | Full legal certification or machine security. |
| P0 | Colorado RLA public data and rules | Public seed, ballot manifest concepts, CVR/hash requirements, audit workflow metadata. | Seed handling, sample replay boundary, jurisdiction adapter metadata, CVR/hash checks where source files are available. | Claims not present in public audit releases. |
| P1 | MIT Election Data and Science Lab precinct returns | Official precinct-level returns by state/cycle, including current-cycle datasets. | Statement-result adapters, precinct/reporting-unit normalization, large summary-only packages. | CVR, ballot manifests, audit samples, chain of custody. |
| P1 | OpenElections | State/county official results in many formats. | Messy real-world statement-of-vote adapters and source-format diversity. | Consistent CVR/audit evidence; many records are compiled from official sources. |
| P1 | 2020 public CVR database | Large cast vote record corpus across multiple jurisdictions. | V.6 CVR-to-summary reconciliation and V.11 performance/parallel verification. | Ballot custody or state-specific certification unless paired with official artifacts. |
| P2 | State SOS canvass exports | Certified results, statements of vote, canvass PDFs/CSVs/XLSX. | V.1 canvass arithmetic and V.9 adapter breadth. | Often lacks raw CVRs, manifests, and audit sample data. |
| P2 | Precinct shapefile/result products | Precinct geometry and election returns. | RCOUNT + RPLAN district aggregation and crosswalk stress tests. | Official count evidence unless traceable to original official sources. |

Primary references:

- MIT Election Data and Science Lab data:
  <https://electionlab.mit.edu/data>
- MEDSL GitHub:
  <https://github.com/MEDSL>
- OpenElections project overview:
  <https://docs.openelections.net/getting-involved/>
- 2020 CVR database article:
  <https://pmc.ncbi.nlm.nih.gov/articles/PMC11604945/>
- Rhode Island RLA Center:
  <https://elections.ri.gov/elections/risk-limiting-audit-center>
- Colorado RLA FAQ:
  <https://www.coloradosos.gov/pubs/elections/RLA/faqs.html>

## Adapter Backlog

### P0: Rhode Island RLA Package Adapter

Goal: build the first real-world RCOUNT audit package from public RLA artifacts.

Expected inputs:

- ballot manifest files;
- CVR files where available;
- audit report files;
- ballot or batch retrieval files;
- tally sheets or aggregate tally files;
- protocol PDF or metadata notes as source evidence.

RCOUNT outputs:

- `sources/source-index.json` with source hashes;
- normalized ballot-manifest records;
- normalized CVR records;
- contest and summary records;
- RLA audit metadata records;
- verification transcript;
- claim-boundary note.

Checks:

- source bytes are preserved and hashed;
- manifest counts reconcile to declared counts where fields exist;
- CVR rows reconcile to summaries where contest fields are interpretable;
- audit sample/retrieval records are captured and replayed where enough seed and
  selection metadata exists;
- unsupported claims are listed rather than inferred.

### P0: Colorado RLA Adapter Expansion

Goal: connect the existing Colorado-style RLA fixture to public process and
available public artifacts.

Checks:

- 20-digit public seed syntax and provenance;
- ballot manifest shape;
- CVR/hash field handling;
- jurisdiction method id and public software-source metadata;
- replayable sampler transcript when the artifact set supports it.

### P1: MEDSL / OpenElections Statement Adapter

Goal: stress V.9 source normalization with broad, messy public result formats.

Checks:

- precinct/reporting-unit id handling;
- contest and candidate label normalization;
- county/state/jurisdiction rollups;
- summary-only package verification;
- format-specific adapter transcripts.

### P1: Large CVR Benchmark Adapter

Goal: validate V.6 correctness and V.11 performance on a corpus larger than
hand-written fixtures.

Checks:

- CVR row canonicalization;
- contest-selection aggregation;
- CVR-to-summary reconciliation where summaries exist;
- serial versus parallel verification timing;
- memory and package-size notes.

### P2: State Statement-Of-Vote Adapter Family

Goal: build jurisdiction-specific adapters only after the generic statement and
CDF paths are stable.

Candidate sequence:

1. one clean CSV or XLSX state;
2. one PDF-heavy state where tables require extraction;
3. one state with precinct splits or reporting-unit naming drift;
4. one state with separate canvass delta files.

## Validation Matrix By V Paper

| Paper | External validation target |
|-------|----------------------------|
| V.0 | All external packages should confirm the package anatomy and source-hash model. |
| V.1 | State canvass exports and statement-of-vote files. |
| V.2 | Multi-cycle precinct datasets and state/local reporting-unit change files. |
| V.3 | Public source files plus package/file/content hash recomputation. |
| V.4 | ElectionGuard comparison and privacy-boundary negative fixtures. |
| V.5 | Rhode Island and Colorado ballot manifest artifacts. |
| V.6 | Public CVR database, Rhode Island CVRs, Colorado-style CVR exports. |
| V.7 | Arlo reports, Rhode Island RLA artifacts, Colorado RLA process. |
| V.8 | MEDSL/OpenElections returns joined to RPLAN district assignments. |
| V.9 | NIST ERR/CDF, MEDSL, OpenElections, state statement exports. |
| V.10 | Evidence-family matrix built from what each real package can and cannot prove. |
| V.11 | Large CVR and large summary-only packages for serial/parallel verification timing. |

## Claim Boundaries

RCOUNT can validate:

- source-file preservation and hashing;
- normalized record consistency;
- contest arithmetic;
- jurisdiction rollups;
- CVR-to-summary reconciliation;
- ballot-manifest count reconciliation where fields exist;
- public seed syntax and sampler replay where enough metadata exists;
- audit report field preservation;
- package-level reproducibility.

RCOUNT cannot validate by data ingestion alone:

- voter eligibility;
- whether every accepted ballot was lawfully accepted;
- whether every rejected ballot was lawfully rejected;
- whether a voting machine correctly captured voter intent;
- whether public source files are complete;
- whether chain of custody was physically maintained;
- whether an election is legally certified.

Those boundaries should appear in every real-world validation package and in
the V.10 evidence matrix.

## First Implementation Slice

Status: first narrow slice landed.

The first external-data adapter is:

```text
rcount import-ri2024-rep28-rla \
  <audit-report.csv> \
  <ballot-manifest.csv> \
  <ballot-retrieval.csv> \
  <output-dir>
```

It targets the Rhode Island 2024 General Election State Representative District
28 ballot-polling audit artifacts published by the Rhode Island Board of
Elections. It imports:

- the Arlo-style audit report CSV;
- the Rep. 28 ballot manifest CSV;
- the ballot retrieval CSV;
- an adapter source-summary transcript that records the RI RLA evidence boundary.

The landed adapter verifies:

- source preservation and source hashes for all three files;
- contest selection arithmetic from the audit report totals;
- manifest batch accounting for all declared manifest rows;
- sampled-ballot rows from the audit report match ballot-retrieval rows by
  normalized ballot key;
- retrieval/sample row count does not exceed the declared round sample size;
- package hash and read/write reproducibility.

It intentionally does not yet claim:

- Minerva risk calculation replay;
- public-seed sample replay;
- ballot-level observation correctness;
- legal certification;
- full Rhode Island RLA format coverage beyond the named 2024 Rep. 28 slice.

The recommended next implementation slice is:

1. Promote the RI Rep. 28 adapter from source-preservation plus batch accounting
   to an explicit ballot-polling RLA evidence model.
2. Promote the sampled-ballot/retrieval key checks into typed sample records.
3. Add a Minerva/Arlo method id that records, but does not falsely replay,
   external audit math until the method is implemented.
4. Add one negative fixture for manifest total drift and one for tampered source
   bytes.
5. Update V.7 and V.10 with the exact evidence boundary.

If Rhode Island artifacts prove too format-heavy for the first pass, use MEDSL
or OpenElections as a smaller P1 summary-only package while keeping Rhode Island
as the first full audit target.
