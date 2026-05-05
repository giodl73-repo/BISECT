# Panel Reviews — Spec 7: Label-Based Run Management
**Spec**: `2026-05-04-spec7-run-manifest.md`
**Round**: 1
**Date**: 2026-05-04

---

## Reviewer 1: Mitchell Hashimoto (HashiCorp — Vagrant, Terraform, CLI Tooling Design)

**Score**: 3 / 4
**Verdict**: Accept with required additions. The label-as-API inversion is correct and the verb surface is clean. Three CI/CD gaps must be closed before this ships.

The core design is sound. Naming things once and deriving all paths through convention is exactly how Vagrant solved the "Vagrantfile as the source of truth" problem. The separation of config (committed), registry (local state), and index (execution record) maps cleanly onto Terraform's `terraform.tf` / `.terraform.lock.hcl` / `terraform.tfstate` trio. The lesson from that ecosystem: the local state file (`.redist`) will cause pain the moment two operators share an output tree. Plan for it now, even if §12 defers it.

**P1 — No machine-readable output mode.** Every CI pipeline that calls `redist ls` or `redist show` will screen-scrape the ASCII table. `--json` must be specified for `ls`, `show`, and any command that emits structured data. Without `--json`, CI scripts are brittle against table formatting changes. This is not optional — it is the difference between a tool that CI can consume and one that it cannot.

**P1 — No `--no-interactive` flag.** `redist rm` prompts for confirmation. In CI, stdin is not a terminal. The spec says "prompts unless `--force`," but `--force` is too blunt — it skips confirmation entirely. Separate `--no-interactive` (machine-safe, no prompts, fails on ambiguity) from `--force` (skip all guards). Terraform's `--auto-approve` is the precedent.

**P2 — No `redist mv X Y` verb.** `redist label X Y` copies the registry entry but does not rename `runs/X/` to `runs/Y/` or update index files that contain `"label": "X"`. This is not a rename — it is an alias. If an operator wants to rename `senate_draft2` to `senate_final`, there is no path. Add `redist mv X Y` that renames directories, updates index `"label"` fields, and updates the registry atomically. Without it, operators will do this by hand and corrupt the audit chain.

**P2 — Config search path is deferred but should have a note.** §12 defers auto-discovery of `configs/X.yml` when label is `X`. In Terraform, the convention is `terraform.tfvars` in the working directory. Deferring is fine, but the spec should note that `--config configs/X.yml` is verbose for the common case and name the auto-discovery rule explicitly so implementors do not invent inconsistent conventions.

---

## Reviewer 2: Nadia Polikarpova (UC San Diego — Formal Methods, Program Synthesis)

**Score**: 3 / 4
**Verdict**: Accept with required clarifications. Registry invariants are well-stated but two concurrent-write hazards remain unaddressed.

The decomposition into three artifact types — committed config, auto-managed registry, machine-written index — is formally clean. Each type has a clear write authority: the user writes configs, the system writes the registry and indexes. The invariants in §6.2 are stated normatively ("A year appears in `analyzed` only if it also appears in `built`") and each has a corresponding L0 test. This is the right level of rigor for a spec that will be cited in litigation.

**P1 — Registry write safety under concurrent builds.** The spec runs three years in parallel (`years: [2020, 2010, 2000]`). Each year's build calls `registry::mark_built(label, year)` when complete. `registry.rs` uses write-to-.tmp-then-rename, which is atomic for a single writer. With three concurrent writers, two writers can each load the registry, each add their year, and then each rename — the second rename wins and the first year's mark is lost. The fix is a file lock (advisory `flock`/`LockFile`) around load-modify-save. §8.5 shows the function signatures but does not specify lock protocol. This must be stated explicitly.

**P1 — Escape-hatch breaks index invariant.** §3.10 allows `redist build official_proposal --out /mnt/results/`. The build index at `runs/official_proposal/index.json` will still record `"label": "official_proposal"` and the algorithm config. But the actual output files are at `/mnt/results/`, not at `runs/official_proposal/`. When `analyze` runs later, it reads `runs/official_proposal/` (the convention) and finds nothing — or finds stale outputs from a previous build. The escape hatch and the convention are in direct conflict. Either: (a) write a stub `runs/official_proposal/index.json` with a `"redirected_to"` field pointing to `/mnt/results/`, and have all path resolution check for redirects; or (b) prohibit `--out` on `build` and restrict overrides to `analyze` and `report` only. The current spec leaves this inconsistency unresolved.

**P2 — `label` field collision: reserved names.** A user could create a label named `runs`, `analysis`, or `reports`. `redist build runs --config ...` would attempt to write to `runs/runs/`, which is legal on the filesystem but semantically confusing. Add a validation rule in §7.2: label names must not equal any top-level directory name used by the convention (`runs`, `analysis`, `reports`, `configs`). State this as an invariant enforced at registry write time, with a corresponding L0 test.

**P2 — `test_registry_write_is_atomic` is untestable as stated.** "No partial write" is not a falsifiable predicate without injecting a failure between the `.tmp` write and the rename. The test must simulate a crash at that boundary — write the `.tmp`, then panic before rename — and assert the pre-existing `.redist` is intact. Without this, the test name overpromises its coverage.

---

## Reviewer 3: Ce Zhang (ETH Zurich — ML Systems, Data Provenance, Distributed Infrastructure)

**Score**: 3 / 4
**Verdict**: Accept with required fixes. Cache staleness model is absent; concurrent write protocol is underspecified.

Label-as-API is the correct abstraction for a pipeline with heterogeneous compute stages and audit requirements. The three-index chain (build → analysis → report, each referencing the prior by SHA) is a sound provenance model: any artifact is traceable to the config that produced it. This is meaningfully stronger than the previous path-threading approach.

**P1 — No staleness model.** The registry marks years as `built`, `analyzed`, `reported`, but never invalidates those marks. If an operator runs `redist build official_proposal --year 2020 --force` (overwriting existing build outputs), the registry will mark 2020 as built again — but the existing analysis outputs at `analysis/official_proposal/2020/` were produced from the previous build. They are now stale: the `run_index_sha256` in `analysis/official_proposal/index.json` will no longer match the new `runs/official_proposal/index.json`. The spec has no mechanism to detect or communicate this. At minimum: when `--force` rebuilds a year, mark that year as `analyzed: []` and `reported: []` in the registry (cascade invalidation), and emit a warning if analysis outputs exist. Without this, operators will compare reports from different builds without knowing it.

**P1 — Concurrent registry writes lose data.** Three parallel year builds each call `mark_built`. The load-modify-save pattern with atomic rename is safe for a single writer but not for N concurrent writers — the last rename wins and prior marks are dropped. Require advisory file locking around all registry mutations. This applies to `mark_built`, `mark_analyzed`, and `mark_reported`. The `registry.rs` function signatures in §8.5 show no lock parameter or lock acquisition pattern. Fix the spec before implementation.

**P2 — Analysis index SHA chain is one-directional.** The analysis index records `run_index_sha256` (pointing back to the build). The build index has no forward pointer to the analysis. An auditor starting from the build index cannot discover which analysis was derived from it. For full bidirectional traceability, the build index should record `"derived_analyses": ["official_proposal"]` or equivalent. This is not strictly required for correctness, but for a litigation-facing tool, auditors will want forward discovery.

**P2 — `config_sha256` verification is not specified.** §5.1 records `config_sha256` in the build index. §9.1 says "any run claiming to reproduce these results must use this config unmodified." But there is no `redist verify official_proposal` command that checks the SHA of `configs/official_proposal.yml` against the build index. Add a `redist verify X` verb, or add SHA verification as a pre-flight step in `redist analyze X` that warns if the config has changed since the build.

---

## Reviewer 4: Moon Duchin (Tufts University — Mathematics, Redistricting Practice)

**Score**: 3 / 4
**Verdict**: Accept with important UX additions. The label model matches practitioner mental models. Two workflow gaps prevent real-world use.

Plans have names. Every redistricting engagement I work on uses names — `commission_v2`, `minority_proposal`, `court_submission_final`. The previous path-threading model forced practitioners to translate plan names into directory paths. This spec eliminates that translation. The cognitive relief is significant: a GIS director can say "show me the analysis for senate_draft2" and the system knows what they mean without further navigation.

The `redist plan` TUI (§3.9) is particularly well designed. The configure screen surfaces the three-layer compositor as a menu rather than a flag list. Practitioners can see and reason about algorithm choices — `● convergence T=600` and `○ multi N=50` — without memorising flag names. The in-TUI research references (`HH prime-factor tree (B.11)`, `Certified optimal (B.16)`) link design choices to their mathematical justification at the point of decision. This is the right approach for a tool used in court-facing work.

**P1 — No import path for externally-produced plans.** `redist build` assumes the plan was produced by the `redist` algorithm. In practice, the most common redistricting scenario is adversarial review: the commission produces a plan, the minority caucus produces an alternative, and staff must analyze both. The minority plan arrives as a shapefile or CSV of precinct-to-district assignments — it was not produced by `redist build`. Without `redist import X --from assignments.csv --year 2020 --format csv`, the label system is unusable for the primary practitioner workflow. This is not optional for a tool aimed at state legislatures.

**P1 — `redist plan` TUI does not show partial build state.** The label screen shows `✓ 50/50` and `✓ 49/50` but the key variant is the mid-build state: `▶ 23/50 running`. When an operator presses `[b]` to build 2010, the TUI must either stream build progress or poll and update the counts. The spec says `plan` dispatches to `redist build` in the background but does not specify how progress is surfaced. A TUI that shows `—` for a year that is currently building is actively harmful — the operator may assume nothing is running and press `[b]` again, spawning a second build.

**P2 — Cascade direction in `redist rm` is implicit.** §3.7 shows `redist rm official_proposal --stage analyze` and notes it "also deletes report if it exists." The cascade rule should be stated as a principle: removing a stage removes all stages that depend on it. `build` deletion removes analysis and report; `analyze` deletion removes report; `report` deletion removes only the report stage. This principle should appear as an explicit rule, not as a note in an example.

**P2 — `redist ls` partial-build indicator is absent.** If 47 of 50 states succeeded, `redist ls` shows `2020` in the Built column with no indication of the shortfall. A commissioner reviewing status must run `redist show` for each label to find failures. Add a partial indicator: `2020(47/50)` or `2020*` with a legend.

---

## Reviewer 5: Dana Hendricks (Wisconsin Legislative Technology Office — State GIS Director)

**Score**: 2 / 4
**Verdict**: Revise and resubmit. The label model matches legislative workflows. Two blocking gaps make the tool unusable for the actual legislative redistricting process.

I manage redistricting technology for the Wisconsin Legislature. My team receives proposed plans from the commission, minority caucus, advocacy groups, and occasionally the courts. We must analyze every submitted plan against the same metrics and produce side-by-side comparisons for legislators. The label system is exactly how we think about plans: each plan has a name, and we need to run the same analysis pipeline on each name.

**P1 — No import verb for externally-submitted plans.** Legislative staff receive plans as shapefiles and CSVs. The commission uses GIS software, not `redist`. The minority caucus submits a CSV of ward-to-district assignments. Advocacy groups submit shapefiles. None of these plans were produced by `redist build`. The current spec has no way to ingest them into the label system. Without `redist import X --from file --year YEAR --format {csv|shapefile}`, the spec covers only in-house algorithmic plans and is therefore unusable for the legislative review workflow. This is the most important gap in the entire spec.

**P1 — No enacted map comparison path.** §3.4 mentions `redist compare official_proposal --enacted --year 2020` but does not specify how enacted maps are stored, retrieved, or integrated into the label system. Do enacted maps get a label? Are they stored in `runs/enacted_2020/`? The `--enacted` flag is mentioned in one example and never explained. Wisconsin staff must compare every proposal against the current enacted map before presenting to the legislature. The spec should define an `enacted` label or a reserved label namespace (`enacted/2020`, `enacted/2010`) with a corresponding `redist fetch-enacted --year 2020` command.

**P2 — No top-level report index page.** `redist report X` produces `reports/X/dashboard_2020.html`. Legal staff and commissioners who do not run the pipeline need to browse these files. There is no `reports/X/index.html` linking to all years. Add this as a required output of `redist report X` when multiple years exist.

**P2 — `redist rm --force` is too easy to misfire.** Legislative staff work under time pressure. Deleting a complete three-year build by typing `redist rm official_proposal --force` is a single command that cannot be undone (outputs are in git-ignored directories). Require the label name as an argument to `--force`: `redist rm official_proposal --force official_proposal`. This pattern (confirming by retyping) is standard in destructive CLI operations (Heroku, AWS CLI for certain resource deletions) and prevents the most common accidental-deletion scenario.

---

## Reviewer 6: Jonathan Rodden (Stanford — Political Geography, Expert Witness)

**Score**: 3 / 4
**Verdict**: Accept with required documentation. The audit chain is technically sound but not yet navigable by a court-appointed special master. Close the chain documentation gap and the config-SHA verification gap before court submission.

I have served as an expert witness in redistricting cases and reviewed expert reports in several others. The central challenge in litigation is reproducibility: can opposing counsel, a special master, or a court-appointed statistician independently verify that the outputs were produced by the stated algorithm with the stated parameters? This spec makes a serious attempt at that guarantee. The three-index SHA chain — report references analysis SHA, analysis references build SHA, build records config SHA — is the right structure for a forensic audit trail.

The `git_commit` field in every index (§5.1, §5.2, §5.3) is the critical provision. It means an auditor can check out that commit, run the build with the committed config, and verify that the outputs reproduce. The `config_sha256` field closes the "was the config file modified between commit and run?" question. These are meaningful improvements over any redistricting system I have reviewed in litigation.

**P1 — Audit chain is not documented as a traversal procedure.** §5 describes three index schemas but never states the audit chain as a sequential procedure a special master can follow. A court-appointed reviewer needs: Step 1: locate `reports/X/index.json`, Step 2: extract `analysis_index_sha256`, verify it matches `analysis/X/index.json`, Step 3: extract `run_index_sha256`, verify it matches `runs/X/index.json`, Step 4: extract `config_sha256`, verify it matches the committed `configs/X.yml`. Without this procedure documented in §5 or §9, auditors will not know how to traverse the chain. Add a "Verification Procedure" subsection to §5 or §9 with this exact sequence.

**P1 — No `redist verify X` command.** The SHA chain is only useful if it can be verified mechanically. A special master will not manually hash JSON files and compare them. `redist verify X --year 2020` should traverse the chain and print a table showing each link's status (MATCH / MISMATCH / MISSING). Without this command, the audit chain is a forensic artifact but not a forensic tool.

**P2 — Config file is the citable artifact but has no canonical URI.** §9.1 says "this config is the citable artifact defining the official proposal parameters." But a citation needs a stable reference: a git commit SHA and a path. Add a `citation` field to the YAML schema: `citation: "git:f263823a/configs/official_proposal.yml"`. This gives opposing counsel a precise reference to challenge or verify. The `git_commit` in the build index captures this implicitly, but the config file itself should carry it explicitly for standalone reference.

**P2 — Registry is git-ignored; litigation requires it to be discoverable.** `.redist` records what was built and when. In litigation, this is evidence: it establishes when a run was executed relative to when certain data became available or when a relevant event occurred. If `.redist` is git-ignored and lives only on a local machine, it may be lost before discovery. Consider a `redist export-registry X` command that emits a signed, timestamped snapshot of the registry entry for a label, suitable for attachment to an expert report.
