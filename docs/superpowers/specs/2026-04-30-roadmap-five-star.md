# Five-Star Roadmap: From Engineer-Built to Practitioner-Ready
**Date:** 2026-04-30
**Updated:** 2026-04-30 (v2 — incorporates 9-role review findings; 2 new capability specs added)
**Status:** Revised; pending re-review
**Author:** Gio Della-Libera

This is the umbrella spec for closing the gap between "well-engineered redistricting toolkit" (current state) and "the tool every redistricting practitioner picks up." It defines the personas, scores them today, scores the target, and points at the per-capability specs.

## Personas and Today's Score

| Persona | Today | Target | What's missing |
|---|---|---|---|
| **Court-appointed special master / expert** | ★★★★★ | ★★★★★ | nothing structural — polish only |
| **Researcher running parameter sweeps** | ★★★★☆ | ★★★★★ | Jupyter examples, MCMC ensemble, external-tool interop |
| **§2 plaintiff's expert post-Callais** | ★★★☆☆ | ★★★★★ | within-party racial bloc voting, race-of-candidate annotation, disentanglement regression, PDF report |
| **State legislative staff drawing maps** | ★★☆☆☆ | ★★★★☆ | not a build target — partner with Districtr/DRA via import/export |
| **Civic advocacy group** | ★★☆☆☆ | ★★★★★ | plain-English narrative output, side-by-side comparisons, partisan-fairness storytelling |
| **(All categories)** | — | — | bootstrap script, end-to-end tutorial, polished errors |

**Note on state legislative staff:** building a Districtr-quality GUI is a multi-year project. MGGG already does it. The honest move is to ship clean import/export so staff can use Districtr as the front-end and our CLI as the analytical backend.

## Capability Specs

| Spec | Closes gap for | Effort |
|---|---|---|
| [Callais Evidence Layer](2026-04-30-callais-evidence-layer.md) | §2 expert (★★★→★★★★★), researcher | ~7-9 days (v2: more rigor) |
| [Court Submission Reports](2026-04-30-court-submission-reports.md) | special master, §2 expert, civic | ~4-5 days (v2: PDF/A + expert conventions) |
| [Onboarding & Tutorial](2026-04-30-onboarding-and-tutorials.md) | all personas | ~3-4 days (v2: pin tutorial data) |
| [Plan Comparison & Narrative](2026-04-30-plan-comparison-and-narrative.md) | civic, state staff, special master | ~3-4 days |
| [Researcher Toolkit](2026-04-30-researcher-toolkit.md) | researcher (★★★★→★★★★★) | ~3-5 days |
| [State Staff Interop](2026-04-30-state-staff-interop.md) | state staff (★★→★★★★) | ~3-4 days (v2: atomic import + version handshake) |
| [Deposition Prep](2026-04-30-deposition-prep.md) **NEW** | §2 expert at trial, special master | ~3-4 days |
| [Civic Bidirectional Input](2026-04-30-civic-bidirectional.md) **NEW** | civic advocate (★★→★★★★★) | ~3-4 days |

Total estimated: **29-39 person-days**. Realistic 6-8 weeks calendar.

## Sequencing Decision

The dependencies and value-density argue for this order:

1. **Onboarding & Tutorial** first. Without it, every other capability lands on users who can't pick the tool up. Cheapest spec, highest leverage.
2. **Callais Evidence Layer** second. The user identified this as the gap that turns the project from infrastructure into usable §2 tool. Highest single-capability impact.
3. **Court Submission Reports** third. Builds on Callais Evidence Layer (the report generator consumes the evidence outputs).
4. **State Staff Interop** fourth. Standalone; can ship in parallel with the above. Unblocks state-staff persona.
5. **Plan Comparison & Narrative** fifth. Builds on report infrastructure; easier after #3 lands.
6. **Civic Bidirectional Input** sixth (NEW). Builds on Plan Comparison; closes COMMONS gap.
7. **Deposition Prep** seventh (NEW). Independent capability; can ship anytime after #2 + #3.
8. **Researcher Toolkit** eighth. Standalone but lowest urgency; researchers can already use the project today.

## CI strategy (BENCHMARK P0 — added v2.1)

Every spec in this set declares L0/L1/L2 tests. Without a named CI surface, the "skipped-by-default" L2s are decorative. The roadmap commits the following CI surface across all specs:

- **PR gate (`.github/workflows/pr.yml`)** — runs L0 + L1 on `ubuntu-latest` and `windows-latest`. Wall-clock budget: 15 min. Markers excluded: `network`, `slow`, `acceptance`.
- **Nightly (`.github/workflows/nightly.yml`)** — cron `0 6 * * *` UTC; runs L2 acceptance suite including the Vermont walkthrough, AEA REPRODUCE.sh, and any spec marked `slow` or `acceptance`. Runs on `ubuntu-latest-large` (8-core). Wall-clock budget: 90 min. Notebooks under `notebooks/` execute here with their declared `runtime_budget_secs` enforced at 1.5×.
- **Release gate (`.github/workflows/release.yml`)** — runs the full nightly suite plus `verapdf` validation on every generated court PDF, byte-identical re-render verification on every narrative, and PyPI/wheel-build smoke. Required green before tagging.
- **Daemon p99 benchmark hardware (Deposition Prep)** — pinned to `ubuntu-latest-large` (8-core, 32 GB) for budget assertions; VT budget p99 ≤ 1.5s in PR; LA budget p99 ≤ 5s in nightly only.
- **Acceptance fixture policy (BENCHMARK P1/P2)** — every L2 test must reference either a checked-in fixture under `tests/fixtures/` or a network-fetched artifact with a pinned SHA-256. "If available" or "from public examples" without a pinned reference is rejected at PR review.

Per-spec CI labels live in pytest markers (`unit`, `integration`, `network`, `slow`, `acceptance`) and Cargo features. The CI YAML is canonical; each spec's Tests section may reference the marker but does not redefine the workflow.

## Out of Scope

- **Native GUI / web app for map drawing.** Districtr exists. Build interop instead.
- **Real-time multiplayer collaboration.** Not a tool category we're targeting.
- **Embedded election-night reporting** (Clarity-style). Not a redistricting workflow.
- **Automated litigation prediction / outcome modeling.** Out of scope; courts decide.

## v2 Changelog

Incorporates findings from the 9-role review (2026-04-30):
- **SCALE BLOCK** unblocked by hardening the Callais Evidence Layer statistical methodology (heteroskedasticity, multiple testing, cluster-bootstrap, replace binary significance flag).
- **2 new specs** added per SURVEY (deposition prep) and COMMONS (civic bidirectional input) findings.
- Tutorial canonical example switched **Louisiana → Vermont** per SURVEY (Louisiana looks advocacy-aligned to a special master).
- Race-of-candidate provenance hardening across Callais Evidence + Court Reports per BOUNDARY/COVENANT/DATUM/TRENCH consensus.
- Tutorial data versioning per DATUM/COVENANT/TRENCH consensus.
- Auto-generated narrative guardrails (human sign-off, threshold parameterization, value-correctness tests) per 7-role consensus.
- Reproducibility appendix completeness (PDF/A, expert conventions, citation format) per 4-role consensus.

## Definition of Done

A persona is "5★" when:
- They can complete their full workflow with our tool alone (or our tool + one named partner like Districtr).
- The output is admissible / publishable / actionable in their domain.
- Onboarding is ≤ 5 minutes from `git clone` to first useful run.
- Errors are actionable, not stack traces.
- Documentation has a worked example for their persona.

## What This Doc is NOT

- An implementation plan. See per-capability `plans/` (drafted after spec review).
- A funding ask. The estimates are calendar-day rough sketches.
- A commitment to ship every capability. The user picks priority.

## Related

- `docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md` — current architecture (what we have today)
- `docs/legal/CALLAIS_REFERENCE.md` — legal grounding for the §2 evidence layer
- `docs/REDIST_CLI.md` — current CLI surface
