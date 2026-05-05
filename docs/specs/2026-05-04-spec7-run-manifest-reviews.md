# Spec 7 Panel Reviews: Run Manifest

**Date**: 2026-05-04
**Spec**: `2026-05-04-spec7-run-manifest.md`
**Round**: 1

---

## Reviewer 1: Ce Zhang (ETH Zurich — Systems, ML Infrastructure)

**Score**: 8.5/10 — Strong. The core problem is real and the solution is well-scoped.

The path inconsistency diagnosis (§1.1) is precise and the fix is the right one: define the canonical output tree once, in the manifest, and derive all downstream paths from it. The template variable system is minimal and correct. Expanding `{base}` before `{state_name}` and `{year}` avoids the ordering ambiguity that plagued Makefiles for decades — good instinct.

My main concern is execution atomicity. §4.6 writes the manifest back to disk after each year completes. At six workers and 50 states per year, the per-year window is tens of minutes. If the process is killed mid-year, the operator knows only that the year was not completed — not which of the 50 states succeeded. The spec says "a subsequent `--reprocess` will re-run that year from scratch," but that wastes the work of the states that succeeded. State-level completion markers (not manifest writes) are the correct granularity. The existing `.states_complete` marker pattern in the codebase already does this for the `run` command. `run-manifest` should read and write the same per-state markers rather than inventing a new year-level resume protocol. This is a correctness concern, not just an efficiency concern: a race between the manifest write and a process kill can corrupt `meta.completed_years` if the write is not atomic AND the year did not actually complete.

The `resolve_path()` dotdot guard is correct but incomplete. Absolute paths beginning with `/` bypass the dotdot check. The spec says "paths escaping the repository root via `..` are rejected" but a user could write `base: /etc/` and the check would pass. Either restrict to relative paths entirely, or resolve to an absolute path and check that it is within `getcwd()` before writing anything.

Otherwise the implementation plan is clean. Reusing `run_states_parallel()` directly rather than spawning subprocesses is the right call — it avoids shell-quoting bugs and keeps error handling in Rust.

---

## Reviewer 2: Nadia Polikarpova (UC San Diego — Formal Methods, Program Synthesis)

**Score**: 7.5/10 — Good problem statement; schema spec needs tightening.

The spec correctly identifies that the reproducibility problem is a missing declarative artifact, not a missing process. The YAML schema is a reasonable first cut, but several fields need formal constraints that are currently only implicit.

First, the template variable grammar is defined informally in §2.2 but not as a formal production. What happens when a user writes `{base}/{base}/{year}`? The spec says "Resolution is eager and eager-only" but does not say what happens when `{base}` appears inside `outputs.base` itself (which would be circular). The implementation must either detect this at load time and exit with `[CONFIG]`, or document that `{base}` is only valid in `states`, `analysis`, and `intermediate` — not in `outputs.base` itself. The current spec is ambiguous.

Second, `schema_version` is a string field with exact-match semantics ("1"). But what is the forward-compatibility contract? The spec says "a v1 runner encountering an unrecognised top-level key emits a warning and continues." That is Postel's Law applied to the wrong side of the schema. Unknown keys in a run configuration are not benign — a user who writes `weight: county` (missing the `s`) will silently get the default (`geographic`) with no error. The schema should use `#[serde(deny_unknown_fields)]` and explicitly document which fields are optional vs. required, with defaults stated normatively.

Third, the `meta` block is both input and output — it is committed as `null` and written back by the runner. This dual-use of a single file as both specification and execution state is a design smell. Separate the committed input (`run.yml`) from the runtime state (`run.lock.json` or similar). The input is read-only to the user; the runtime state is written only by the runner. This avoids the awkward "null in committed file" pattern and makes `git diff run.yml` meaningful after a run.

The test table (§7) is thorough for unit tests. The L2 tests should include a test that verifies the manifest on disk after a completed year has `completed_years: [2020]` and `completed_at` non-null.

---

## Reviewer 3: Percy Liang (Stanford — Empirical ML, Reproducibility)

**Score**: 9/10 — This is exactly the right tool for empirical reproducibility.

The reproducibility problem in §1.4–1.5 is one I have seen destroy years of comparative work: two groups claim to run "the same" algorithm but used different flags, and neither has a checked-in artifact to settle the dispute. The manifest solves this by making the algorithm configuration a first-class committed artifact rather than a shell history entry.

The `meta.git_commit` field is correct but insufficient for full reproducibility. The git commit captures the source code version, but the adjacency binary files (`.adj.bin`) are not in version control (per the CLAUDE.md `.gitignore` rules). A manifest claiming to reproduce a run must also record the SHA-256 of every input adjacency file consumed — not just the git commit. The `PlanManifest` (Spec 1) already does this per-state; `RunManifest` should aggregate those hashes or include a pointer to the per-state manifests so an auditor can verify the full input provenance chain from a single file.

The dry-run output in §3.1 is excellent — showing the literal CLI commands that would be executed is the most useful debugging surface a manifest runner can provide. I would add one more thing: the dry-run should also print a SHA-256 of the resolved manifest (after variable substitution but before execution) so two operators can compare dry-run fingerprints and confirm they are running the same configuration before starting a multi-hour run.

The `artifacts.proportionality` field is listed in the official_proposal manifest but not defined in the `ArtifactSpec` struct in §4.1. The struct shows only `maps`, `dashboard`, `national_summary`. Either add `proportionality: bool` to the struct and document what it triggers, or remove it from the official manifest. This inconsistency will cause a YAML parse error if `deny_unknown_fields` is enabled (which it should be per Nadia's review).

---

## Reviewer 4: Jonathan Rodden (Stanford — Political Geography, Legal)

**Score**: 8/10 — The manifest addresses a real evidentiary gap. Two policy concerns.

From a court-submission perspective, the manifest solves a genuine problem: today there is no single document a special master can examine to understand what algorithm produced a set of outputs. The `meta.git_commit` plus the algorithm spec together provide the evidentiary chain. The requirement that the official manifest be committed to version control (not just written by the runner) is the correct design choice — it means the algorithm was specified before the run, not inferred from its outputs.

First concern: the `algorithm.structure: apportion-regions` field in the official manifest (§6) is a technical identifier whose meaning is not self-evident to a court. The manifest schema should support a normative `reference` field pointing to the legal or technical document that defines the algorithm. For the official proposal, this would point to B.11 and the proposed statute text. Without this, a court document citing "the official_proposal manifest" would need a separate exhibit explaining what `apportion-regions` means. The `description` field partially addresses this, but a structured `references` list would be stronger.

Second concern: the `artifacts.proportionality: true` field in the official manifest generates proportionality analysis. Proportionality analysis compares partisan outcomes to vote shares — exactly the kind of analysis that *Rucho v. Common Cause* (2019) said federal courts cannot adjudicate. Including proportionality in the official federal-statute manifest creates an appearance problem: if this is "the official federal proposal," why does it compute metrics that are not actionable in federal court? Consider either removing `proportionality` from the official manifest (it can live in a separate research manifest) or adding a `legal_note` field that records "proportionality output is generated for research purposes; it is not part of the statutory compliance claim."

The exit code table (§3.4) correctly distinguishes partial from total failure. For court use, I would recommend adding exit code 5 for "manifest modified after partial run" — detectable by comparing `meta.git_commit` against the current HEAD.

---

## Reviewer 5: Moon Duchin (Tufts — Mathematics, Redistricting Practice)

**Score**: 8.5/10 — Practitioner-facing design is sound. One substantive gap.

The primary virtue of this spec is that it treats a multi-year redistricting run as a single atomic experiment rather than three independent CLI invocations. In practice, every serious redistricting study runs multiple years for comparison, and the inconsistency between years is a major source of errors in practitioner submissions. The manifest enforces consistency by construction.

The state filter (`states: []` meaning all 50) is the correct default, and the `--states VT` override for smoke testing is exactly the workflow practitioners need. I use this pattern constantly and the current CLI requires remembering to pass `--states` to every subcommand separately. Having it in the manifest means I set it once and forget it.

The substantive gap is in the `artifacts` section. The spec lists `analysis_types: [demographic, political, compactness, summary]` but omits `contiguity` and `splits` — two of the most legally significant outputs. In every redistricting court case I am aware of, contiguity (all districts connected) and county/municipal splits are the first things an opposing expert checks. The official_proposal manifest should include `contiguity` and `splits` in `analysis_types` by default. Omitting them from the official proposal manifest creates a situation where the reference run does not generate the outputs most likely to be demanded in litigation. This is not a spec bug — the spec correctly lists all available analysis types in §4.2 — but the official manifest in §6 should be amended.

Additionally, the template variable `{state_name}` uses lowercase underscore convention (`north_carolina`) but the spec does not document what happens for territories or international locations that use non-standard names. The spec should state: "For US states, `{state_name}` follows the `lowercase_underscores` convention defined in CLAUDE.md §Conventions. For international locations loaded via `--adjacency`, `{state_name}` is the value of `--state-name`, lowercased with spaces replaced by underscores." Without this, a user running Ireland or Malta through a manifest will get an undocumented path.
