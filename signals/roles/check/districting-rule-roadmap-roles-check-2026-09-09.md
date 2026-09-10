---
skill: roles-check
topic: districting-rule-roadmap
date: 2026-09-09
roles_used: [meridian, datum, benchmark, scale, contour, boundary, ward, covenant, commons, survey]
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# Districting rule roadmap: local roles review

Artifact: `docs/specs/2026-09-09-districting-rule-development-roadmap.md`.
Type: research and implementation roadmap. Review is a single-agent application
of installed role lenses, not independent expert testimony or external approval.
All installed role descriptions were inspected. Selection below covers the
current algorithm, comparison, geographic, governance, and operational scope.
CANVASS/TALLY are outside this non-election-count task. VAULT, LEDGER, TRENCH,
and PRECINCT require focused reviews when witness implementation, interchange,
production hardening, or political evaluation becomes executable work.

Findings refer to the proposed program and its phase-entry requirements.
Resolved means addressed in the written plan, not that a future experiment ran.

| Role and reason | ID | Severity | Section / finding | Disposition |
|---|---|---|---|---|
| MERIDIAN: candidate mechanics | M1 | P2 | W0: one cut after scoring does not mean weights cannot matter | Resolved: require pre-boundary counts and mark physical diversity unknown |
| MERIDIAN | M2 | P2 | W2: per-node tolerance need not ensure final leaf tolerance | Resolved: final-leaf gate and descendant completion requirements |
| MERIDIAN | M3 | P2 | W1: changing search and weights together confounds mechanism | Resolved: separate paired engine arms |
| DATUM: evidence scope | D1 | P2 | Baseline: NRS is not the official county-aware configuration | Resolved: named profiles and explicit distinction |
| DATUM | D2 | P2 | W1: T.3 manuscript numbers are not a verified current baseline | Resolved: source-artifact gate |
| DATUM | D3 | P2 | W1: cherry-picking a successful demonstration | Resolved: declared development and confirmation cohorts; all failures retained |
| BENCHMARK: falsifiable checks | B1 | P2 | W0: malformed or incomplete census could look successful | Implementation condition: reject missing, duplicate, invalid and non-preserving rows |
| BENCHMARK | B2 | P2 | W0: derived report must regenerate | Implementation condition: deterministic JSON, input/code hashes, independent repeat |
| BENCHMARK | B3 | P2 | W1: tests of config plumbing do not prove map influence | W1 entry condition: actual candidate/assignment measurements |
| SCALE: aggregation and inference | S1 | P2 | W1: purposive cohort cannot estimate national prevalence | Resolved: descriptive scope and reserved confirmation |
| SCALE | S2 | P2 | W1: pooled means obscure geographic variation | Resolved: paired State rows, distributions, explicit aggregation |
| SCALE | S3 | P3 | W1: exact descriptive differences do not need invented p-values | Resolved: distinguish deterministic measurement from inference |
| CONTOUR: input identity | C1 | P2 | W1: tract and block experiments are not interchangeable | Resolved: common input/hash manifest gate |
| CONTOUR | C2 | P2 | W1: water and zero-population handling changes split counts | W1 entry condition: freeze inclusion and report both policies |
| CONTOUR | C3 | P2 | W0: preserve GEOID identifiers and State coverage | Implementation condition: keep strings and validate coverage |
| BOUNDARY: legal scope | L1 | P2 | Governance: local lens calls 0.5% universal constitutional threshold | Resolved: do not adopt role text as legal authority; no legal threshold asserted |
| BOUNDARY | L2 | P2 | W2: experimental feasibility must not become legal validity | Resolved: research status and jurisdiction-specific legal gate |
| BOUNDARY | L3 | P2 | W7: exact proof does not settle adoption | Resolved: independent legal review before adoption claims |
| WARD: subdivision semantics | W1 | P2 | W1: additive alpha differs from multiplicative notation | Resolved: report alpha and 1+alpha |
| WARD | W2 | P2 | W1: split count ignores repeated fragmentation | Resolved: extra and maximum fragment metrics |
| WARD | W3 | P2 | W1: county preservation is not a universal legal rule | Resolved: descriptive metrics and separate jurisdiction/chamber profile |
| COVENANT: custody and commitment | E1 | P2 | W0: derived artifact lacks executable provenance | Implementation condition: bind inputs and analyzer SHA-256 |
| COVENANT | E2 | P2 | W6: existing genesis occurs after complete-tree construction | Resolved in plan: pre-run commitment is an explicit future deliverable |
| COVENANT | E3 | P2 | W7: same-laptop replay is not independent replication | Resolved: independent operator gate |
| COMMONS: public meaning | O1 | P2 | W1: county intactness does not measure neighborhoods | Resolved: separate community evaluation requirement |
| COMMONS | O2 | P2 | W6: review intervals without adjudication are incomplete | W6 condition: named authority, grounds, restart and correction rules |
| COMMONS | O3 | P3 | W7: proof artifacts alone are inaccessible | W7 condition: plain-language cut explanations and review materials |
| SURVEY: execution feasibility | P1 | P2 | W1: unbounded national sweeps delay actionable evidence | Resolved: phased cohort and proposed resource limits |
| SURVEY | P2 | P2 | W0: first task should run without the large vault | Resolved: repository-only diagnostic |
| SURVEY | P3 | P3 | W7: demonstration needs an operator beyond the author | Resolved: independent operator and replay instructions |

## Synthesis

Roles reviewed: 10. Findings: 0 P1, 27 P2, 3 P3.
Verdict: APPROVED-WITH-CONDITIONS for phased research; W0 may proceed now.
W1–W7 require their written entry and acceptance gates. No official rule,
legal profile, winning alpha, or empirical outcome is approved by this review.

Top finding: county weights cannot be evaluated honestly without measuring the
candidate set on which they act. MERIDIAN, DATUM and BENCHMARK converge on this.
COVENANT and COMMONS converge on pre-run commitment and real challenge handling.

Three highest-priority amendments incorporated in the roadmap:
1. Add W0 before the sweep; distinguish pre- and post-boundary candidate counts.
2. Separate population policy, search engine, and county penalty interventions.
3. Specify input custody, confirmation cohort, final-leaf checks, and legal and
   public-process gates without claiming those later stages are completed.

## W0 execution follow-up

Implemented `scripts/research/analyze_county_preservation_readiness.py` and six
failure-path/measurement tests. All six pass. Existing census coverage is 50
States, 44 multi-district States and 385 nodes. Multiple pre-boundary candidates
occur at 287 nodes; boundary scoring reduces candidate count at 71. Physical
partition diversity before scoring is explicitly unknown. Output is at
`docs/experiments/county-preservation-readiness-2020/analysis.json`.

B1/B2/C3/E1 are satisfied for this diagnostic by validation, deterministic
regeneration, string identifiers, and source/analyzer hashes. The W1 influence
gate remains open pending physical-candidate instrumentation and actual weight
response. Later-stage conditions remain open as listed above.
