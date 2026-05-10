# Panel Review — Track A-synthesis (Batch 1)

**Date**: 2026-05-07
**Reviewer panel**: R1 Karypis (graph algorithms), R2 Rodden (political science), R3 Duchin (math/redistricting), R4 Stephanopoulos (law), R5 Liang (ML/AI)
**Score scale**: 0–4. Verdict: Accept ≥3.0 | Minor Revision ≥2.5 | Major Revision ≥2.0 | Reject <2.0

---

## Summary Table

| Paper | Avg Score | Verdict | P1 Count | Top P1 Issue |
|-------|-----------|---------|----------|--------------|
| A.0 Synthesis Metapaper | 2.8 | Minor Revision | 2 | Compactness figure inconsistency (+56% in findings vs +22% in abstract/A.1/A.3); *Louisiana v. Callais* citation needs verification |
| A.1 Portfolio Guide | 2.9 | Minor Revision | 1 | `redist/` workspace reference is stale (crates are named `bisect-*`) |
| A.2 Portfolio Summary | 2.4 | Major Revision | 3 | 11-paper vs 75+-paper count contradiction; stale GitHub URL; +56% vs +22% compactness inconsistency |
| A.3 Portfolio Visualization | 2.7 | Minor Revision | 1 | Track count mismatch (abstract says 7, table lists 7 but text says "five research tracks"); `redist map` command uses old binary name |
| A.4 Replication Materials | 2.5 | Minor Revision | 2 | `redist run`, `redist doctor`, `redist map` commands throughout; placeholder SHA-256 hashes not yet confirmed |
| A.5 Policy Brief | 3.1 | Accept | 0 | — |

---

## A.0 — Synthesis Metapaper

### Reviewer scores

**R1 Karypis — 3.0**
The algorithm description in §3 is technically accurate: METIS recursive bisection (METIS_PTYPE_RB), 10-cut attempts, 10 KL refinement iterations, ufactor=1.005. The O(n^1.12) empirical vs O(n log k) theoretical complexity is correctly distinguished. The edge-weight formula w(i,j) = α·s_demo(i,j)/(d(i,j)+ε) is well-specified. One concern: the abstract claims "+22% compactness" and Finding 1 (§4.1) reports "+56% over unweighted baseline." These measure different baselines (enacted maps vs unweighted partitions) but both numbers appear in the same document without clarifying this distinction. A reader treating them as interchangeable will be confused, and a referee will demand one consistent comparison. The METIS-5.1.0 specification in §3 also conflicts with A.4's claim of METIS-5.2-vendored; the version should be unified.

**R2 Rodden — 2.5**
The partisan claims are carefully hedged in the metapaper, which I appreciate: the −3.2% Democratic EG bias is correctly attributed to geographic sorting (Rodden 2019), the NC 7D/7R result is correctly scoped to the ApportionRegions algorithm only (not GeoSection). However, §4.4 ("Partisan Patterns Reflect Geography") claims 56.5% Democratic vote share algorithmically—this figure appears without citation or methodology note. Post-hoc vote imputation using precinct-level data applied to algorithmically-drawn tracts is methodologically non-trivial and the paper needs to cite the exact election year(s) and dataset. The conclusion that "algorithms cannot amplify geographic patterns" is strong; ensemble analysis (A.0 §4.1 mentions GerryChain) provides support but should be foregrounded more.

**R3 Duchin — 2.5**
The 42% threshold is correctly presented as empirical regularity rather than legal bright line—good. The Gingles prong-1 limitation (opportunity vs performing districts) is properly flagged in §4.2. The variance decomposition "3.2× geographic vs temporal" is cited to deluca2026temporal but the methodology is not explained: what decomposition method, what state subset, what years? For a Science-level paper this needs one sentence in the main text. The ensemble percentile positioning (0.1–0.7th compactness percentile for WI/GA/PA/CA; 50th for NC) is an important finding correctly stated. Cross-paper number consistency is the main flaw: the compactness improvement headline varies across the portfolio (22%, 44%, 56%) without the document explaining which comparisons each measures.

**R4 Stephanopoulos — 2.5**
*Rucho v. Common Cause* (2019) is correctly cited and the ruling is accurately characterized. *Karcher v. Daggett* (1983) is correctly cited for the 0.5% population equality standard. *Thornburg v. Gingles* (1986) citation and the three-prong test are accurate. The document appropriately distinguishes Gingles prong-1 opportunity from full Section 2 liability. **P1 flag**: The paper cites "*Louisiana v. Callais* (608 U.S. ___ (2026))" for a specific proposition about VRA evidentiary standards in §5.1 (`sections/05-implications.tex` line 11). *Callais v. Landry* was argued before the Supreme Court in 2024–2025; the case is real but its final resolution and slip-opinion citation are uncertain as of the paper's May 2026 date. Using a specific reporter citation (608 U.S. ___) for a case whose final disposition or reporter page is unconfirmed risks citing a fictitious or incorrect citation. Either (a) confirm the exact citation if decided, (b) cite it as "slip op." with a footnote, or (c) remove the parenthetical and describe only the evidentiary implication. The *Arlington Heights* (1977) effects-based discussion is accurate.

**R5 Liang — 3.0**
The supplementary parameter specification is thorough (seed=42, ncuts=10, niter=10, ufactor=1.005). The SHA-256 audit chain concept is present. The self-citation "deluca2026recursive" for the Rust implementation is appropriate. The compactness metric (Polsby-Popper) is consistently used. The figure placeholders (PNG files referenced but not checked-in to repo) are acceptable for a working paper but should be noted. Determinism across architectures (x86-64 vs arm64) is correctly flagged as a known limitation. No major reproducibility blockers.

### Panel average: 2.7 → Minor Revision

### Issues

**P1 — Compactness figure inconsistency (all reviewers)**
The abstract and conclusion say "+22% compactness." Finding 1 (§4.1) says "+56% over unweighted baseline." A.1 says "+22%–44%." These measure different baselines:
- +22% = edge-weighted algorithmic vs enacted maps (apples-to-apples)
- +56% = edge-weighted vs *unweighted* bisection (algorithm-internal gain)

Both are valid but the document must name each comparison explicitly at first use. The abstract should use only the enacted-map comparison (+22%) for the headline; the +56% can appear in the technical section with baseline labelled.

**P1 — *Louisiana v. Callais* citation verification required**
`sections/05-implications.tex`: The paper cites "608 U.S. ___ (2026)" as settled law clarifying VRA §2 evidentiary standards. This case is in litigation; the reporter citation is speculative. Replace with "slip op." + date if decided, or "cert. granted, argued [date], decision pending" if not yet decided, or remove the specific parenthetical.

**P2 — METIS version conflict**
§3 specifies METIS 5.1.0; A.4 (replication package) specifies METIS 5.2 vendored. One document will be wrong. Unify to whichever version is actually compiled in the binary.

**P2 — 56.5% Democratic vote share (§4.4) lacks citation and methodology note**
Add: which election years, which dataset (e.g., VEST precinct-level returns), how tract-to-precinct imputation was performed.

**P3 — Figure PNG files are placeholders**
`figure2_national_map_placeholder.png` name reveals the file is not a real figure. Rename before journal submission.

---

## A.1 — Portfolio Guide

### Reviewer scores

**R1 Karypis — 3.0**
The three-layer compositor table is accurate and consistent with the CLAUDE.md specification. The METIS attribution (Karypis & Kumar, 1998) is correct. The O(n log k) complexity claim is accurate. The guide correctly lists `--search` variants including `forest-recom` and `vra-recom`. One issue: "Code and Data" section references `the \texttt{bisect} binary, built from the \texttt{redist/} workspace in this repository." The crates directory contains `bisect-*` named crates (bisect-cli, bisect-core, etc.); there is no `redist/` workspace. This is a stale reference from before the rename.

**R2 Rodden — 3.0**
The reading paths are appropriately differentiated by audience. The NC 7D/7R result is correctly scoped to ApportionRegions with the note that GeoSection gives 5D/9R—this is an important disclosure that prevents misrepresentation. The 223D/209R national result via ApportionRegions is consistent with T.4's reported finding. The efficiency gap and majority-minority district counts are consistently stated throughout (−3.2%, +69 surplus, 137 vs 68). No partisan neutrality concerns; the guide is appropriately balanced.

**R3 Duchin — 3.0**
The Gingles test is correctly stated in the glossary (three prongs, correct citations). The Polsby-Popper formula is correctly given (4π·Area/Perimeter²). The glossary correctly distinguishes majority-minority districts as VAP-based, not CVAP-based. The VRASection glossary entry cites *Allen v. Milligan* (2023) and *Callais* (2025); the Callais year here is listed as 2025 (consistent with when it was argued, pre-decision), differing from A.0's "2026" citation. This inconsistency should be resolved across all papers.

**R4 Stephanopoulos — 2.5**
The legal citations are handled lightly but accurately. The *Rucho* description is correct. The *Gingles* preconditions are accurately stated. One concern: the guide references "D.5 (Gingles Bloc-Voting): Expert witness methodology for the Gingles bloc-voting analysis required when algorithmic plans face Section 2 challenge"—this implies the portfolio includes expert witness materials ready for court use. Courts require such materials to meet Daubert/Kumho standards; the guide should add a caveat that D.5 is a methodological paper, not pre-certified expert testimony. Absent this caveat, a practitioner might over-rely on D.5 in litigation preparation.

**R5 Liang — 2.5**
The guide references `redist-metis` crate (glossary, METIS entry) and `redist/` workspace. The crate is correctly named `bisect-metis` in the actual codebase. These stale names would break replication for any researcher following the guide. The `bisect fetch` command in "Code and Data" is correct. The guide otherwise does not contain replication steps, so the damage is limited to terminology confusion.

### Panel average: 2.8 → Minor Revision

### Issues

**P1 — Stale `redist/` workspace reference**
`guide.tex` line 427: "built from the `\texttt{redist/}` workspace in this repository." The workspace is `crates/` and all crate names begin with `bisect-`. Update to: "built from the `crates/` workspace; the CLI crate is `bisect-cli`."

**P1 — `redist-metis` vs `bisect-metis`**
`guide.tex` line 442: "(\\texttt{redist-metis} crate)". Correct crate name is `bisect-metis`. Fix the reference.

**P2 — *Callais* year inconsistency (2025 in A.1, 2026 in A.0)**
Establish canonical treatment across all A-track papers. If decided, use the correct year; if pending, use consistent phrasing.

**P2 — D.5 Daubert caveat**
Add a sentence noting that D.5 presents a methodological framework, not pre-certified court testimony, and that practitioners should engage qualified experts for actual litigation.

**P3 — Track H omitted from reading paths**
The reading paths for Computer Scientists cite U.10 (Rust ReCom) but the guide does not include a dedicated H-track reading path. For completeness, add or note that Track H is covered under ensemble reading paths.

---

## A.2 — Portfolio Summary

### Reviewer scores

**R1 Karypis — 2.0**
Technical description of Papers 02 and 08 conflict. Paper 02 summary says edge-weight parameter α=5 achieves 56% compactness improvement (0.39 vs 0.25 mean PP)—this is the vs-unweighted comparison. Paper 08 says single-objective edge-weighting produces 18% better compactness than multi-constraint methods. These are three different comparisons (vs enacted, vs unweighted, vs multi-constraint) that are not distinguished anywhere in the summary. The runtime complexity O(n log k) in Paper 02 summary is described correctly. The "constraint non-commutativity" claim for Paper 08 is a strong theoretical statement that should reference whether it is proved or only empirically demonstrated. Paper 06 summary claims "non-MM districts gain +7.5% compactness when VRA-weighting is enabled"—this is a counterintuitive result that needs verification against Paper 06's actual finding.

**R2 Rodden — 2.0**
Critical problem: the summary's framing (title, overview, paper count) describes "an 11-paper research program" with Papers 01–11. However, the Track G and H summaries then describe a 75+-paper portfolio with 8 tracks. These cannot both be true. The document appears to be a stale early-stage summary (the 11-paper version) that was partially updated with the larger portfolio's G/H descriptions but not globally revised. The efficiency gap result (−3.2% algorithmic, +5.1% enacted, 62% reduction) is consistent. But saying "All 11 papers are complete and ready for submission" directly contradicts the rest of the portfolio's status (many papers in draft). This is a material misrepresentation if submitted to journals.

**R3 Duchin — 2.5**
The 42% threshold paper summary correctly limits the claim to geometric feasibility, not legal obligation—good. The cross-census validation methodology summary (slice-based, variance decomposition 3.2×) is consistent with A.0. The statistical equivalence claim in Paper 10 (recursive vs n-way: p=0.23 for compactness, p=0.18 for MM districts) is presented without specifying test type (t-test? Mann-Whitney?). These p-values are high enough to be plausible but need a note on sample size (50 states is not a large n for frequentist inference). The temporal stability numbers (80% vs 66% retention, +14pp advantage) are consistent across A-track papers.

**R4 Stephanopoulos — 2.5**
The legal framing is generally accurate. The Rucho discussion (post-Rucho courts lack judicially manageable standards) is correct. The efficiency gap as "standard legal metric" claim is overstated—it is an academic metric used in expert reports; courts have not uniformly adopted it as a legal standard. The 7-percentage-point threshold claim (enacted plans deviating >7pp from algorithmic baselines "suggest manipulation") is the summary's most legally significant assertion; it is not supported by a court citation because no court has adopted this threshold. The summary should describe this as a proposed empirical benchmark, not a legal standard. The Gingles analysis (Paper 04/D.1) is correctly hedged.

**R5 Liang — 2.5**
The replication section contains two P1 issues. First, the GitHub URL is `https://github.com/giodl73-repo/REDIST`—this is the old repository name (pre-rename); it should be verified or updated. Second, `bisect states --workers 8` is the correct current command syntax (consistent with CLAUDE.md), but the section also says "Binary available at [old URL]" which suggests the published binary is named `redist`, not `bisect`. Third, the summary says "redist-metis crate" where the actual crate is `bisect-metis`. The description of A.4 as "AEA-compliant replication materials" is consistent with A.4's own framing.

### Panel average: 2.3 → Major Revision

### Issues

**P1 — 11-paper vs 75+-paper portfolio contradiction**
The title says "An 11-Paper Research Program." The G/H summaries describe a 75+-paper portfolio. The document must choose: either rewrite as a concise 11-paper early overview (clearly date-stamped as a prior version), or globally update to reflect the current 75+-paper portfolio and retire the 11-paper framing. As published, this is misleading.

**P1 — "All 11 papers complete and ready for submission" is false**
The current portfolio has many papers in draft. This sentence must be removed or corrected. If this is an early-version document, it should be clearly marked as superseded by A.0.

**P1 — Stale GitHub URL and binary name**
`\url{https://github.com/giodl73-repo/REDIST}` and `redist-metis` crate reference. Update to reflect current repository and crate name (`bisect-metis`), or confirm if the REDIST URL remains valid.

**P2 — Three compactness comparisons not distinguished**
+56% (vs unweighted), +22–44% (vs enacted), +18% (vs multi-constraint). Each measures a different comparison. All appear in the same document without labels. Add a note at first use distinguishing the baselines.

**P2 — Efficiency gap framing overstated**
Rephrase "standard legal metric" to "academic metric proposed as a legal standard" or similar. Rephrase the ">7 percentage points" threshold as a proposed empirical benchmark, not an adopted legal standard.

**P3 — Paper 10 statistical test not specified**
Name the test type and note n=50 states for the compactness and MM-district comparisons.

---

## A.3 — Portfolio Visualization

### Reviewer scores

**R1 Karypis — 3.0**
The algorithm description in section 02 is accurate for a lay audience. The track-by-track summaries correctly characterize B.6 as O(n^1.07) complexity, B.2 as +22% compactness, and U.1 as T=600 convergence. The five headline numbers are internally consistent and correctly sourced to specific papers. The note distinguishing ApportionRegions (7D/7R NC) from GeoSection (5D/9R) is present and correctly placed—critical for technical accuracy.

**R2 Rodden — 2.5**
Track G summary states algorithmic plans are "statistically indistinguishable from random draws on compactness and partisan metrics." This is too strong. A.0 reports bisection plans at the 0.1–0.7th compactness percentile in WI/GA/PA/CA—these are *extreme* compactness outliers in the ensemble, not random draws. Saying the plans are indistinguishable from random draws on compactness directly contradicts the core finding. What is presumably meant is that they are not partisan outliers in the ensemble; this should be stated precisely. This is a material misstatement of Track G's findings.

**R3 Duchin — 3.0**
The 42% threshold note correctly adds "not a legal bright line" and "five covered states" context. The five headline numbers are internally consistent with A.0 and A.1. The track summaries correctly cite paper codes (B.2, B.6, U.1, D.1). The visualization doc is appropriately targeted at lay audiences.

**R4 Stephanopoulos — 2.5**
The audience guide for judges directs them to `docs/quickstart/quickstart-callais-expert.md`. This is an internal repository file, not a public document. Judicial audiences cannot be expected to clone the repository; either provide the content inline or reference A.5 (Policy Brief) as the judicial entry point. The sentence about *Rucho* is accurate. No false legal citations found in this document.

**R5 Liang — 2.5**
The "Reproducing Any Figure" code block in dashboards section (06) mixes `bisect` and `redist` commands in the same three-line block: `bisect fetch`, `bisect state`, then `redist map`. The `redist map` command is the old binary name; the current command is `bisect map` (or equivalent). A researcher following this code block will fail on the third command. This is a P1 replication failure point.

### Panel average: 2.7 → Minor Revision

### Issues

**P1 — Mixed `bisect`/`redist` commands in figure reproduction block**
`sections/06-dashboards.tex` line 56: `redist map --state NC --year 2020 --version v1`. The first two commands use `bisect`; this third uses `redist`. Update to `bisect map` (or the correct current subcommand).

**P2 — Track G "statistically indistinguishable from random" contradicts ensemble percentile finding**
`sections/04-track-summaries.tex`: "produces plans that are statistically indistinguishable from random draws on compactness and partisan metrics." Replace with: "produces plans that are compactness-extremal (0.1–0.7th percentile for most states) but partisan-central within the ensemble, demonstrating that compactness optimization and partisan neutrality are decoupled."

**P2 — Internal file referenced for judicial audiences**
`sections/05-how-to-use.tex`: replace `docs/quickstart/quickstart-callais-expert.md` reference with either the content itself or a citation to A.5 as the judicial entry point.

**P3 — Abstract/intro says "more than forty papers organized into seven tracks (A–G)"**
The track table in `01-overview.tex` lists seven tracks (A–G) with paper counts summing to 56, while the guide (A.1) reports 75+ papers across eight tracks (A–H). Reconcile with current portfolio count or note this is an earlier draft.

---

## A.4 — Replication Materials

### Reviewer scores

**R1 Karypis — 2.5**
The SHA-256 audit chain design is sound: four-step chain (config → build → analysis → report) is a recognized pattern for computational reproducibility. METIS determinism caveat (x86-64 vs arm64 may differ) is correctly flagged. The METIS 5.2 vendored specification is appropriate for reproducibility. The 0.001 pp Polsby-Popper tolerance for cross-architecture differences is plausible. The `cargo build --release` + PATH setup is standard. Main concern: A.0 specifies METIS 5.1.0; A.4 specifies METIS 5.2 vendored. These cannot both be correct for the same binary. One document has the wrong version.

**R2 Rodden — 3.0**
The replication package correctly focuses on computational reproducibility rather than claiming partisan neutrality. It does not overclaim. The Vermont walkthrough as a canonical test case is practical. The SHA chain protects against post-hoc modification of results. No partisan bias concerns in this document.

**R3 Duchin — 2.5**
The placeholder SHA-256 hashes (0000...0001 through 0000...0004) are labeled as "to be confirmed." This is honest but incomplete: a replication package that does not contain the actual expected hashes is not yet functional as a replication package. The abstract claims "Any researcher with an internet connection and a modern laptop can reproduce the full 50-state results"—this is the goal, but the package is not complete until real hashes are published. Similarly, the Vermont walkthrough hash table contains placeholder values (`abcdef01...`, `23456789...`). The document should either (a) be marked explicitly as draft/pre-release, or (b) defer publication until real hashes are available.

**R4 Stephanopoulos — 3.0**
The AEA Data and Code Availability Policy compliance framing is appropriate and the four requirements (public data, posted code, README, runtimes) are addressed. The legal claim about "tamper-evident" SHA chain is accurate in the computational sense; courts will need independent expert testimony to interpret the chain, but the mechanism is sound. No legal accuracy concerns.

**R5 Liang — 2.0**
Multiple P1 replication failures from stale binary names:
1. `sections/02-software.tex` line 60: `# Binary appears at: target/release/redist` — the binary is named `bisect` (per CLAUDE.md and crate names), not `redist`.
2. Line 74: `redist --version   # Should print: redist 0.2.0` — command and expected output both wrong. Should be `bisect --version` printing `bisect 0.2.0` (or whatever the current version is).
3. Line 75: `redist doctor` — should be `bisect doctor` if that command exists, or this command does not exist.
4. Line 107: `redist doctor` again.
5. `sections/04-reproduction.tex` line 44: `redist run --year 2020 --version official_2020` — `redist run` is the old command; the current command per CLAUDE.md is `bisect build <label> --year 2020`. No `redist run` subcommand exists in the current codebase.
6. `sections/06-test-suite.tex` line 12–13: `redist doctor` commands.
7. `sections/05-how-to-use.tex` (A.3) line 39: `redist analyze` command — current command is `bisect label-analyze`.

Any researcher following the install instructions in §2 will produce a binary named `bisect` (from `bisect-cli` crate), then be told to run `redist --version` which will fail immediately. The entire replication procedure is broken at step 1 of verification.

Additionally, the GitHub URL `https://github.com/giodl73-repo/REDIST` should be verified; if the repository was renamed, the URL is stale.

### Panel average: 2.6 → Minor Revision (borderline, driven by Liang's P1 failures)

### Issues

**P1 — Binary name is `bisect`, not `redist`, throughout**
`sections/02-software.tex`, `sections/04-reproduction.tex`, `sections/06-test-suite.tex`: Every occurrence of `redist` as a command name must be changed to `bisect`. Specific instances:
- `target/release/redist` → `target/release/bisect`
- `redist --version` → `bisect --version`
- `redist doctor` → `bisect doctor` (verify this subcommand exists)
- `redist run` → `bisect build` (per CLAUDE.md syntax)

This is the single most important fix: a researcher cannot reproduce anything if they cannot run the binary.

**P1 — Placeholder SHA-256 hashes must be replaced before publication**
`sections/04-reproduction.tex` table and `sections/06-test-suite.tex` table: Replace all `0000...` and `abcdef01...` placeholders with confirmed hash values from a clean first run. Until this is done, mark the document as "DRAFT — hashes pending."

**P2 — METIS version conflict with A.0**
A.0 says METIS 5.1.0; A.4 says METIS 5.2. Pick one (the one actually vendored in the binary). Add a note explaining the version choice.

**P2 — `redist run` command does not match CLAUDE.md**
The correct command for building a plan is `bisect build <label> --year 2020 --workers 8`. The A.4 replication procedure uses `redist run --year 2020 --version official_2020` which matches neither the binary name nor the subcommand structure. The reproduction steps will fail on Step 4.

**P3 — Bootstrap scripts not verified to exist**
`sections/04-reproduction.tex` references `./bootstrap.sh` and `bootstrap.bat`. Confirm these files exist in the repository or remove the reference.

---

## A.5 — Policy Brief

### Reviewer scores

**R1 Karypis — 3.0**
The algorithm description is appropriately simplified for a lay audience. The structural guarantee box ("cannot see partisan data") is technically accurate. The model statutory language (Appendix A) correctly specifies algorithm parameters (0.5% population tolerance, no partisan data). No technical inaccuracies found. The compactness figure "20% more compact" is lower than both A.0 (+22%) and A.2 (+56%); this is the most conservative figure and therefore the most defensible for a policy brief where overclaiming carries political risk.

**R2 Rodden — 3.5**
This is the strongest paper in the A-track from a political science perspective. The findings are appropriately hedged: "69 more majority-minority districts" is presented descriptively, not prescriptively. "62% less partisan bias" is attributed to the efficiency gap and correctly described as a reduction, not elimination. The "30% more competitive districts" claim is sourced to C.8. The policy brief appropriately avoids the impossibility defense overclaim ("cannot gerrymander at all") while correctly saying the algorithm "cannot be instructed to favor any party." The three adoption models (legislative mandate, commission guidelines, court remedy) are practically framed. The cost comparison (Appendix B) based on publicly reported cases (Pennsylvania, North Carolina, Maryland, Ohio) is specific and verifiable.

**R3 Duchin — 3.0**
The 42% threshold is correctly described as applying to the geographic feasibility of proportional representation, not as a VRA legal requirement. The limitation section (implied by "What stays out") correctly excludes race from the algorithm inputs. The "Finding 2" box states "The algorithm uses no racial data"—this is accurate for the default mode but needs a clarifying note that the VRA-weighted mode (vra-aligned) uses demographic counts; the impossibility defense for the VRA-weighted mode is nuanced and the brief should acknowledge it exists. This is a P2, not P1, since the brief targets commissions that will use the standard (non-VRA) mode.

**R4 Stephanopoulos — 3.5**
The legal content is strong. The model statute (Appendix A) is well-drafted: it specifies inputs, outputs, the audit requirement, permitted adjustments, and the no-partisan-manipulation clause. The written Gingles justification requirement for VRA deviations (Section 3(b) of model statute—adoption section) is legally appropriate. The cost comparison is based on identified public cases, not speculation. The court-ordered remedy section correctly names Pennsylvania and North Carolina as precedents for court-drawn maps (though the algorithmic-map remedy is not precedent; this is accurately described as an extension of existing practice). One note: the brief does not cite *Rucho* in the main body (only in A.5 sections/01-problem.tex via \textit{Rucho v. Common Cause}) but the main.tex wrapper does not have a bibliography that would resolve this. Confirm the bibliography compiles.

**R5 Liang — 2.5**
No broken commands in the main body. The brief correctly references `bisect` tool (adoption section) and `bisect label-verify` (implied by "SHA-256 audit chain makes the result tamper-evident"). The open-source claim is accurate. The "Anyone can verify this in minutes using open-source code" claim requires that the repository URL be functional and the binary name be correct—both issues found in A.2 and A.4 carry over here if readers follow the chain. The brief itself is clean; the replication risk is inherited from upstream issues in A.4.

### Panel average: 3.1 → Accept

### Issues

No P1 items.

**P2 — VRA-weighted mode acknowledgment missing from Finding 2 box**
Add a footnote or parenthetical: "For VRA-compliance runs, the algorithm may receive demographic *counts* (minority VAP) as edge weights; it still receives no partisan affiliation or election data."

**P2 — Bibliography compilation**
Confirm the `.bib` file is present and the `\cite{rucho2019}` in `sections/01-problem.tex` resolves. (The main.tex file does not include a bibliography section in the main column layout; Appendix A has no citations; the .bib may not be referenced correctly in the two-column layout.)

**P3 — "20% more compact" vs "+22%" in other A-track papers**
The brief uses 20%, A.0/A.1/A.3 use 22%. If both are correct for different comparison sets, note this; if 20% is a rounding or a different measure, align. For internal consistency, 22% is preferable as it matches the peer-reviewed papers.

---

## Cross-Paper Issues (Affecting Multiple Papers)

### CX1 — Binary rename: `redist` → `bisect` (P3 in A.1/A.3, P1 in A.4)

The project renamed its CLI binary from `redist` to `bisect` (per recent commit history). The A-synthesis track has partially absorbed this rename:
- **Clean**: A.1 (uses `bisect` throughout except two residual references), A.5 (uses `bisect` throughout)
- **Mixed**: A.3 (`bisect` + `redist map` mixed in same code block)
- **Mostly stale**: A.4 (`redist` appears in every code block and binary verification step)
- **One residual in A.1**: `redist/` workspace, `redist-metis` crate

A find-and-replace pass over A.4 `sections/02-software.tex`, `sections/04-reproduction.tex`, and `sections/06-test-suite.tex` will fix most instances. Also fix `sections/05-how-to-use.tex` (A.3) line 39 (`redist analyze` → `bisect label-analyze`).

### CX2 — Compactness headline inconsistency (P1 in A.0, P1 in A.2, P3 in A.5)

Three different compactness improvement figures appear across the A-track:
- **+22%**: vs enacted maps (A.1, A.3 headline number; the enacted-map comparison)
- **+22%–44%**: range across algorithm variants vs enacted maps (A.1 technical description)
- **+56%**: vs unweighted bisection (A.0 Finding 1, A.2 Paper 02 summary, A.2 core contributions)
- **+20%**: A.5 policy brief (conservative rounding?)

Recommended resolution: adopt "+22% vs enacted maps" as the universal portfolio headline; report "+56% vs unweighted baseline" as a secondary technical finding with explicit baseline label. Retire "+20%" in favor of the documented +22%.

### CX3 — *Callais* date inconsistency (P2 in A.0 and A.1)

A.1 glossary: "*Callais* (2025)." A.0 §5.1: "*Louisiana v. Callais* (608 U.S. ___ (2026))." These conflict. Establish a single canonical treatment for all A-track papers. As of May 2026, confirm whether the Supreme Court has decided this case and what the correct citation is.

### CX4 — Portfolio paper count inconsistency (P1 in A.2, P3 in A.3)

A.0: "ten papers" (figure 1 caption). A.1: "75+ papers across eight tracks." A.2: "11-paper research program" (title). A.3: "more than forty papers organized into seven tracks." The actual portfolio has evolved and the documents have not been globally updated. Recommended: update all A-track documents to use the current count (75+) with 8 tracks (A–H), or explicitly date-stamp early drafts as superseded.

---

*Panel review conducted 2026-05-07. Reviewers: R1 Karypis, R2 Rodden, R3 Duchin, R4 Stephanopoulos, R5 Liang.*
