# Five-Star Roadmap: From Engineer-Built to Practitioner-Ready
**Date:** 2026-04-30
**Status:** Proposed; pending review
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
| [Callais Evidence Layer](2026-04-30-callais-evidence-layer.md) | §2 expert (★★★→★★★★★), researcher | ~5-7 days |
| [Court Submission Reports](2026-04-30-court-submission-reports.md) | special master, §2 expert, civic | ~3-4 days |
| [Onboarding & Tutorial](2026-04-30-onboarding-and-tutorials.md) | all personas | ~2-3 days |
| [Plan Comparison & Narrative](2026-04-30-plan-comparison-and-narrative.md) | civic, state staff, special master | ~3-4 days |
| [Researcher Toolkit](2026-04-30-researcher-toolkit.md) | researcher (★★★★→★★★★★) | ~3-5 days |
| [State Staff Interop](2026-04-30-state-staff-interop.md) | state staff (★★→★★★★) | ~2-3 days |

Total estimated: 18-26 person-days. Realistic 4-6 weeks calendar.

## Sequencing Decision

The dependencies and value-density argue for this order:

1. **Onboarding & Tutorial** first. Without it, every other capability lands on users who can't pick the tool up. Cheapest spec, highest leverage.
2. **Callais Evidence Layer** second. The user identified this as the gap that turns the project from infrastructure into usable §2 tool. Highest single-capability impact.
3. **Court Submission Reports** third. Builds on Callais Evidence Layer (the report generator consumes the evidence outputs).
4. **State Staff Interop** fourth. Standalone; can ship in parallel with the above. Unblocks state-staff persona.
5. **Plan Comparison & Narrative** fifth. Builds on report infrastructure; easier after #3 lands.
6. **Researcher Toolkit** sixth. Standalone but lowest urgency; researchers can already use the project today.

## Out of Scope

- **Native GUI / web app for map drawing.** Districtr exists. Build interop instead.
- **Real-time multiplayer collaboration.** Not a tool category we're targeting.
- **Embedded election-night reporting** (Clarity-style). Not a redistricting workflow.
- **Automated litigation prediction / outcome modeling.** Out of scope; courts decide.

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
