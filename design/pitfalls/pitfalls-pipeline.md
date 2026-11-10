# Pipeline Pitfalls

Structural vulnerabilities in multi-script pipeline chains. The root pattern: a flag or intent expressed at one level of the chain silently fails to reach the level where it matters.

---

## PP-01: Subprocess flag inheritance gap

**Pattern:** A flag registered and honored at level N of a pipeline chain is silently ignored at level N+1 because each subprocess starts with a fresh argument namespace. Child script defaults override parent intent. The system appears to accept the flag (no error) but produces output as if it weren't set.

**Domain:** Any system with a chain of subprocess calls where configuration flows from outer to inner scripts. The longer the chain, the more opportunities for a flag to be dropped. Flags registered in the outermost script must be explicitly passed at every subprocess boundary — they do not propagate automatically.

**Why it's hard to catch:** The bug is invisible at the call site. The parent passes the flag correctly; the child simply never receives it. Only end-to-end testing or chain-completeness auditing reveals the gap.

**Structural solution:** Chain completeness table — for every flag registered in the outermost script, trace its explicit pass-through at every subprocess boundary. Any gap in the table is a bug. `review-pipeline` skill automates this audit.

**Status:** SOLVED for `--version`, `--skip-political`, `--skip-demographic`, `--election-year`
**Proved by:** Flags explicitly traced through `run_complete → run_states_parallel → process_single_state → run_state_redistricting`
**Test:** `tests/unit/test_pipeline_flag_propagation.py` — all 15 tests

---

## PP-02: Force-rerun requires two independent mechanisms to both fire

**Pattern:** A "force re-run" operation requires clearing a completion marker AND passing a reset flag to the execution script. If either mechanism fires without the other, the system appears to re-run (clears the marker, says it's reprocessing) but actually uses stale outputs (execution script skips because output file exists).

**Domain:** Any pipeline with both (a) a completion tracking system (marker files) and (b) an execution-level skip check (output file existence). These are independent safeguards that must both be bypassed for a true force re-run. They were designed separately and their interaction was not specified.

**Why it's hard to catch:** The force-rerun appears to work — states cycle through the pipeline visibly. Only inspecting the output file timestamps reveals they weren't regenerated.

**Structural solution:** The force-rerun flag must propagate all the way to the execution-level skip check. `--reprocess` at the outer level becomes `--reset` at the inner level (deletes output files before checking). The two mechanisms are explicitly linked.

**Status:** SOLVED
**Proved by:** `--reprocess` now appends `--reset` to redistricting flags in process_single_state.py
**Test:** `tests/unit/test_pipeline_flag_propagation.py::TestReprocessPropagation`

---

## PP-03: Conditional step with inversion gap

**Pattern:** An optional analysis step is conditioned on a flag (`run_analysis=True`). The flag has an "enable" path (explicitly set True) and a "disable" path (explicitly set False). But only one path is propagated through the subprocess chain — the disable path is dropped. Result: the step always runs even when the flag says it shouldn't.

**Domain:** Any pipeline where an optional step can be skipped via a flag, and that flag must traverse subprocess boundaries. The inversion (False / skip) is harder to propagate than the assertion (True / run) because conditional logic at each boundary tends to only check for True.

**Structural solution:** Propagate the disable case explicitly: `if not args.run_analysis: cmd.append('--skip-analysis')`. Child scripts must also register the disable flag (`--skip-analysis` sets `run_analysis=False`). The default in child scripts must be False — explicit enable required, not implicit.

**Status:** SOLVED for `--skip-analysis`, `--skip-political`, `--skip-demographic`
**Proved by:** `run_states_parallel.py` defaults `run_analysis=False`; all three skip flags propagated
**Test:** `tests/unit/test_pipeline_flag_propagation.py::TestSkipFlags`

---

## PP-04: Cross-language subprocess environment locality

**Pattern:** A compiled binary spawns a subprocess using a short command name (e.g., `python`). The OS resolves that name against the spawning process's PATH, which differs from the environment the test runner or user expects. The subprocess finds a different interpreter — with different packages, different version, different behaviour — and fails with a package import error or silently produces wrong output.

**Domain:** Any system where a compiled binary (Rust, Go, C++) invokes a scripted runtime (Python, Node, Ruby) via subprocess. The binary's PATH is set at launch time, often from a system shell profile, not from the virtual environment the developer is working in.

**Why it's hard to catch:** The binary works when launched from the terminal (where the developer's shell profile sets PATH correctly). It fails in CI, tests, or when launched programmatically — where PATH comes from the test runner, not the developer's profile.

**Structural solution:** Never use a short command name for a language runtime in a compiled binary. Accept the executable path via an environment variable (`BISECT_PYTHON`, `BISECT_NODE`) that the caller must set explicitly. Document this requirement in the binary's `--help` output.

**Status:** SOLVED for Python subprocess calls in bisect-cli
**Proved by:** BISECT_PYTHON env var used in adjacency_loader.rs and runner.rs; acceptance tests set it to sys.executable
**Test:** `tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance` — fixtures pass BISECT_PYTHON=sys.executable

---

## PP-05: Dual-purpose path parameter creates read/write coupling

**Pattern:** A single path parameter controls both where outputs are written and where input data is read from. In production, these are the same directory tree, so no coupling is visible. In tests or non-standard deployments where inputs and outputs live in different locations, the system either fails to find inputs or writes outputs into the input directory.

**Domain:** Any pipeline with an `--output-dir` parameter that is also used to locate data (adjacency graphs, demographics, configurations) that predates the current run. The parameter name implies "where to write" but the implementation also uses it as "where to read".

**Why it's hard to catch:** All existing tests and documentation assume inputs and outputs share a root directory. The coupling is never visible until someone needs to read from a read-only data store or write to a separate location.

**Structural solution:** Separate read paths from write paths explicitly. Output data goes to `--output-dir`. Input data (adjacency graphs, TIGER files, demographics) is looked up from `outputs/{version}/` or a separate `--data-dir` parameter that defaults to the version root. The two paths are always independent.

**Status:** SOLVED in bisect-cli runner.rs
**Proved by:** Adjacency path uses `outputs/{version}/data/` independently of output_dir
**Test:** `tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance` — tmp dir as output-dir, V3 as adjacency source

---

## PP-06: Parallel task failure invisibility via silent filter

**Pattern:** When parallel tasks are collected with a filter-and-map pattern (`filter_map(.ok()?)` or equivalent), failures are silently discarded. The caller receives an empty or partial result with no indication of what failed. Downstream code that assumes complete results then produces wrong output (e.g., "0/1437 tracts assigned") with no connection to the actual failure.

**Domain:** Any parallel computation (Rayon, async, thread pool) that maps over items and collects results. The ergonomic choice `filter_map` makes partial failure the default path — a single task error quietly removes that item from the result set.

**Why it's hard to catch:** The system doesn't crash. It produces output (an empty or short result), and the error manifests far downstream: "0 tracts assigned" appears to be a bisection bug, not a METIS error. The actual failure message is lost.

**Structural solution:** Use `map(|item| ... .map_err(...)?).collect::<Result<Vec<_>, _>>()?` in parallel contexts. First failure aborts the collection and returns the error. If partial success is acceptable, collect errors explicitly into a separate channel that is always inspected.

**Status:** SOLVED in bisection_runner.rs
**Proved by:** run_all_splits uses .map().collect::<Result>()? — first METIS error propagates immediately
**Test:** `tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance::test_al_rust_mm_count` — would show "bisection failed: depth N node '...': <gpmetis error>" instead of silent empty result

---

## PP-07: External tool parameter unit mismatch at boundary

**Pattern:** A parameter stored internally in one unit or format is passed to an external tool that expects a different unit or format. The external tool silently accepts the wrong value and produces subtly wrong output. The symptom (e.g., population imbalance) looks like an algorithm failure, not a parameter error.

**Domain:** Any system that drives an external binary (METIS, GDAL, ffmpeg) whose parameter conventions differ from the calling system's conventions. The mismatch is invisible in the code (both look like "ufactor") and only manifests in output quality.

**Why it's hard to catch:** The external tool accepts the value without error. Its output is plausible — wrong values still partition the graph, just with looser or tighter balance than intended. The test that catches it (population balance check) is far from the conversion point.

**Structural solution:** Create a typed wrapper at every external tool call boundary that performs the conversion explicitly and documents it. The wrapper's function name encodes the output unit: `ufactor_for_metis(depth: usize) -> f64` makes the conversion and destination explicit. Unit tests verify the wrapper for known input/output pairs.

**Status:** SOLVED in bisection_runner.rs
**Proved by:** ufactor_for_depth() returns decimal (1.001..1.005); acceptance tests confirm pop balance <=0.5%
**Test:** `tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance::test_al_rust_population_balance`

---

## PP-08: URL Filename Extraction Without Query Parameter Stripping

**Pattern:** A local file path is derived from a URL by taking the last path segment (`url.split('/').last()`). If the URL contains query parameters (e.g., `?format=zip&version=2020`), the "filename" includes the query string, producing invalid filesystem paths on Windows (`?` is forbidden) and confusing path construction on all systems.

**Domain:** Any system that downloads files from URLs and stores them locally by inferring the filename from the URL. This appears whenever REST APIs use query parameters to specify format or version, which is common in government data portals.

**Why it's hard to catch:** During development, URLs are clean (census.gov TIGER URLs have no query params). The bug only surfaces when the URL format changes, when using mirrors, or when query-parameterised redirect URLs are followed.

**Structural solution:** Always strip query parameters before extracting filename:
```rust
let raw = url.split('/').last().unwrap_or("file.zip");
let filename = raw.split('?').next().unwrap_or(raw);
```

**Status:** SOLVED in fetch.rs
**Proved by:** URL stripping applied to all filename derivations in build_fetch_list
**Test:** Add test: URL `http://example.com/tl_2020_50_tract.zip?token=abc` → filename `tl_2020_50_tract.zip`

---

## PP-09: In-Memory Download Buffer OOM for Variable-Size Files

**Pattern:** A download function loads the entire response body into RAM before writing to disk. The comment documents the expected size ("typically 1-5MB") based on known examples. When a different data source (or a larger state) produces a 50-100MB file, the system OOMs with a generic error unrelated to the real cause (insufficient memory).

**Domain:** Any system that uses `response.bytes()` or equivalent to download files that vary significantly in size by state/entity. Government datasets are particularly prone: California's redistricting files are ~50-80× larger than Vermont's.

**Why it's hard to catch:** The system works correctly for small states (VT, RI, DE) during development and testing. The bug only manifests for large states (CA, TX, NY) or when disk quota is abundant but RAM is not.

**Structural solution:** Always stream downloads to a temp file regardless of expected size:
```rust
let mut out = std::fs::File::create(&tmp_zip)?;
std::io::copy(&mut response, &mut out)?;
```
Then extract from the temp file. The system's I/O subsystem handles large transfers without buffering.

**Status:** SOLVED in fetch.rs::download_and_extract_zip
**Proved by:** Streaming via std::io::copy to TempDir before zip extraction
**Test:** OPEN — no test with >50MB response; add test_download_large_zip

---

## PP-10: Silent State Omission on Invalid Year Parameter

**Pattern:** A lookup function filters a collection by a year key and returns only matching entries. If no entries match (e.g., the year "2030" or a typo "202a"), the function returns an empty collection with no error. The caller receives an empty list and may produce zero output with zero error messages, making the bug invisible.

**Domain:** Any system where a key parameter (year, version, region) is used to filter a collection, and the "no matches" result is indistinguishable from "valid but empty." Pipeline orchestrators are particularly vulnerable because they treat empty state lists as "nothing to do" rather than "invalid input."

**Structural solution:** Validate the key parameter against an allowlist before filtering. Return `Err` for unknown values:
```rust
if !["2020", "2010", "2000"].contains(&year) {
    return Err(format!("unsupported year '{year}' — valid: 2020, 2010, 2000"));
}
```

**Status:** SOLVED in runner.rs::load_all_states
**Proved by:** Explicit year allowlist check before manifest lookup
**Test:** Add test: `load_all_states("2030")` returns Err, not empty Vec

## PP-11: Compile-time asset path resolution mismatch in workspace builds

**Pattern:** A `include_bytes!("../assets/file")` path resolves relative to the source file, but when cargo builds from a workspace root, the CWD differs from the crate root. The macro resolves from the manifest directory, not the CWD — but the exact behavior depends on how cargo sets up the build environment. Files that appear to be sibling directories (`../assets/`) may resolve to nonexistent paths.

**Domain:** Any Rust workspace with embedded binary assets (fonts, icons, data files). Arises because `include_bytes!` resolves relative to the *source file*, not the CWD or crate manifest, but workspace vs. crate-local builds can expose this differently.

**Why it's hard to catch:** The path looks correct in isolation. It only fails when building from the workspace root vs. the crate root. No compile-time error until the macro can't find the file.

**Structural solution:** Place embedded assets in the same directory as the source file that includes them (`src/font_embed.ttf` next to `src/renderer.rs`), not in a sibling directory. Alternatively, use `concat!(env!("CARGO_MANIFEST_DIR"), "/assets/file")` inside a `build.rs` to generate an absolute path.

**Status:** SOLVED
**Proved by:** Font file moved to `BISECT/crates/bisect-map/src/font_embed.ttf` — `include_bytes!("font_embed.ttf")` works from any workspace build context
**Test:** `cargo build --release --manifest-path Cargo.toml` succeeds

## PP-12: WKB decode API surface mismatch across geospatial crate ecosystem

**Pattern:** Multiple Rust crates in the geo ecosystem provide WKB encoding/decoding with subtly different APIs and `geo-types` version pins. Calling the wrong API (`WkbReader` which doesn't exist, or `Wkb(bytes).to_geo()` through a private module path) produces compile errors. Adding a secondary WKB crate alongside `geo` risks `geo-types` version conflicts that cause type incompatibility at runtime.

**Domain:** Any Rust system using the GeoRust ecosystem (geo, geo-types, geozero). Arises because the ecosystem has multiple competing WKB crates with overlapping functionality.

**Why it's hard to catch:** The correct API (`geo_types::Geometry::from_wkb(&mut cursor, WkbDialect::Wkb)` via the `FromWkb` trait) is not obvious from crate documentation. Wrong calls compile only after adding the right imports, making the error appear to be an import issue rather than an API surface issue.

**Structural solution:** Use `geozero` as the single WKB bridge — it is already a transitive dependency of `geo` and guarantees type compatibility. Import via `use geozero::wkb::{FromWkb, WkbDialect}` and call `geo_types::Geometry::from_wkb(...)`. Never add a second WKB crate.

**Status:** SOLVED
**Proved by:** `dissolve.rs::wkb_to_geometry()` uses `geozero::wkb::FromWkb` — all WKB tests pass including MultiPolygon island tract case
**Test:** `bisect-map::dissolve::tests::test_wkb_decode_known_unit_square`, `test_wkb_multipolygon_does_not_panic`

## PP-13: Integer parameter truncation at subprocess boundary

**Pattern:** A system passes a numeric parameter to an external tool via a command-line string. The external tool uses integer parsing (`atoi()`) internally, which silently truncates any decimal part. The calling system computes the parameter as a float and formats it as a float string, assuming the tool receives the float value. Both sides behave correctly individually; the mismatch only appears at the boundary, and no error is raised — the tool accepts the (truncated) integer silently.

**Domain:** Any system that invokes external tools via command-line strings where the tool's parameter parsing is stricter than the calling language's default numeric formatting. Common in scientific computing where the external tool (METIS, BLAS, etc.) is written in C and uses `atoi`/`strtol` for performance. Arises because the calling code and the tool documentation may describe the parameter in compatible terms (e.g., both say "ufactor controls balance") without clarifying the type contract.

**Why it's hard to catch:** The external tool accepts the malformed value without error. Behavior changes only quantitatively, not qualitatively — partitions are still produced, just with different (usually tighter) balance tolerance than intended. At small district counts (10 congressional districts), the difference between ufactor=1 and ufactor=5 is invisible because METIS achieves balance easily either way. The error only manifests at large district counts (98 state house) where the tighter-than-intended tolerance becomes impossible to satisfy.

**Structural solution:** Explicitly convert parameters to the type expected by the external tool before formatting as a string. For integer tools, round to integer and document the conversion formula at the call site. Add a unit test that calls the conversion function and asserts the output type and expected value for known inputs. The test must use the same conversion path as the production code.

**Status:** SOLVED
**Proved by:** gpmetis receives `-ufactor=50` (integer) instead of `-ufactor=1.0500` (float that atoi truncates to 1). FL 120HD now passes balance check.
**Test:** `bisect-cli::bisection_runner::tests::test_ufactor_wasnt_silently_truncated_regression`, `test_ufactor_integer_conversion_5_pct`

---

## PP-14: Metadata Resolution Split — dual sources for plan parameters

**Pattern:** A command that operates on an existing plan (analyze, report, compare) resolves plan metadata (num_districts, chamber, balance_tolerance) from a global lookup table rather than the plan's own manifest. The global table only knows about the primary chamber (e.g., congressional: WA=10), so any plan run with a non-primary chamber (WA house=98, WA senate=49, IE parliamentary=43) silently receives wrong metadata. The plan was created correctly; the error is entirely in the post-creation command.

**Domain:** Any CLI tool that has both (a) a "create" phase that writes metadata to disk and (b) an "operate" phase that re-derives metadata from a global registry. Arises because the global registry is added for the create phase and then accidentally used in the operate phase. The two phases feel symmetric but have different correct information sources.

**Why it's hard to catch:** The bug produces plausible output — analysis files are written, balance checks run — but with wrong district counts. A WA house plan analyzed as 10 districts shows 10 districts with near-zero deviation (each district is very large) and reports "balance OK" trivially. Without knowing the plan should have 98 districts, the output looks valid. Only comparing district counts between manifest.json and analysis summary reveals the mismatch.

**Structural solution:** Introduce `PlanContext` — a typed wrapper that loads all plan metadata exclusively from `{plan_dir}/manifest.json`. All Class B commands (analyze, report, compare, verify, map) must construct a `PlanContext` at entry and use only its methods. Prohibit `load_all_states()` and `LocationRegistry` calls for plan-level metadata in Class B commands. Enforce via code review: any PR touching a Class B command that adds a `load_all_states()` call is a bug.

**Status:** SOLVED
**Proved by:** `bisect-cli::plan_context::PlanContext::from_label()` loads manifest directly. `analyze.rs` uses PlanContext for `num_districts`. WA house 98D: `analyze` now produces summary with 98 districts, balance_valid=true, max_deviation=0%.
**Test:** `bisect-cli::integration_pipeline_tests::test_plan_context_wa_house_98_not_congressional_10`

---

## PP-15: Entry-point switching without PATH pre-flight check

**Pattern:** A canonical command's underlying implementation is changed (e.g., script path → binary path) by editing a wrapper file. The user's invocation syntax stays the same, so the cutover appears transparent. But the new backing tool may not be installed or on PATH on every machine where the wrapper runs. Failure manifests as silent confusion ("command not found" yesterday-it-worked) without informing the user that the wiring changed.

**Domain:** Any system where doskey aliases, batch wrappers, shell aliases, or `npm run` scripts route a stable user-facing command name to a backing implementation, and the backing implementation can change between machines, environments, or release tags.

**Why it's hard to catch:** Pre-cutover the invocation works (old script). Post-cutover the invocation fails on any machine that doesn't have the new binary, with no hint that the wiring changed. Developers see "command not found" or worse, an unrelated tool with the same name resolved against PATH. Bisecting the failure points at the user's environment, not the cutover commit.

**Structural solution:** Pre-flight check at shell-init time. The setup file that creates the wrappers (`setup_env.bat` or equivalent) verifies the backing binary is on PATH and prints a clear, named warning if not — naming the binary, the cutover, and where to install it. The check runs before any wrapper is created, so users see the warning when they source the env, not when they first try to run.

**Status:** SOLVED
**Proved by:** `setup_env.bat` PATH preflight (Plan 01 entry-point cutover, Task 3.2)
**Test:** Sourcing `setup_env.bat` without `BISECT` on PATH prints a named warning; sourcing with `BISECT` on PATH prints no warning.

---

## PP-16: Incremental deletion commits create brittle rollback dependencies

**Pattern:** A multi-step deletion or refactor plan is structured as N separate commits, with the implicit promise of granular rollback ("if Task 4 breaks, just `git revert <hash>`"). But each commit may depend on earlier commits' state. Reverting commit M alone may produce a clean revert (no merge conflict) while leaving the codebase in an invalid intermediate state, because later commits already adapted to the deletion that's now being undone.

**Domain:** Any large refactor, migration, or deletion executed as multiple sequential commits where developers assume single-commit revert is safe. Most acute when deletion plans span weeks and intermediate work merges in between commits in the plan.

**Why it's hard to catch:** `git revert` succeeds without conflict — Git's view of "clean revert" is purely textual. The resulting tree may not compile or pass tests because intermediate work has accrued that depended on the deleted state. Tests fail with confusing errors that point at the intermediate work rather than the revert.

**Structural solution:** Every multi-commit plan includes a "Rollback Plan" section that specifies dependency-aware ordering. Single-commit revert is forbidden once any downstream work has merged; rollback then requires reverting in reverse order, in groups, with verification between each group. The plan document itself records which commits depend on which, making the dependency graph explicit and reviewable.

**Status:** SOLVED
**Proved by:** Plan 02 (Python archive + deletion) Rollback Plan section enumerating the dependency-aware procedure.
**Test:** N/A — procedural pitfall. The proof is the existence of the rollback procedure as a reviewable artifact in every multi-commit plan.

---

## PP-17: Sensitive-asset commit prevention via manual reminder, not structural enforcement

**Pattern:** A repository references sensitive or licensed assets (legal PDFs, certificates, large data files, third-party binaries) by hash or URL, but the assets themselves must not be committed. `.gitignore` prevents tracking under expected names, but a developer can bypass with `git add -f` or by misnaming the asset. Plan documents that say "do not commit X" rely on the developer reading them under deadline pressure.

**Domain:** Any project that deliberately references-without-committing — legal-citation projects, ML model registries with external storage, regulated data pipelines, projects that keep large binary fixtures out of git history.

**Why it's hard to catch:** A hurried `git add .` or `git add -f` commits the asset. .gitignore catches generic patterns but not renamed copies. Once committed and pushed, removal requires history rewriting, and any downstream consumers of the leaked asset have already pulled it.

**Structural solution:** Pre-commit hook that blocks staging of files matching the sensitive patterns (`*.pdf`, `*.pem`, etc.), requiring an explicit override flag (`--allow-pdf`) to bypass. The override creates an audit trail in the commit message. The hook is installed by the project's setup script so every developer's clone has it without manual configuration.

**Status:** SOLVED
**Proved by:** Pre-commit hook installed by setup_env (Plan 03 Task 1.4) blocks `*.pdf` staging unless `ALLOW_PDF=1` is set in the environment.
**Test:** `git add some.pdf && git commit` is rejected by the hook; `ALLOW_PDF=1 git commit` succeeds, producing an auditable commit.

---

## PP-18: Pulse completion without trace or risk closure

**Pattern:** A wave pulse marks deliverables complete and records validation commands, but does not name the accepted requirements, package boundaries, code-rigor constraints, risk disposition, pitfall/invariant effects, or public/custody impact. The work looks closed locally while VTRACE evidence remains partial.

**Domain:** Any wave/pulse-managed project where pulses predate a formal systems-engineering trace process. The failure is procedural rather than technical: implementation, tests, and docs may all exist, but downstream reviewers cannot tell which mission/spec IDs were satisfied or which risks were accepted.

**Structural solution:** VTRACE-governed pulses must follow the `context/waves/PHASES.md` closure rules: parent IDs, affected boundaries, validation level, role gates, verification state, risk/pitfall disposition, and public/custody effects are required before closure. Archived pulses are not retroactively rewritten, but future governed pulses cannot close on "implemented" alone.

**Status:** MITIGATED
**Proved by:** WP-007 updated `context/waves/PHASES.md` and VTRACE closure records.
**Test:** First future VTRACE-governed pulse should receive L1 checklist review before being treated as the process pattern.

---

## PP-19: Interrupted admission leaves a canonical-path partial file

**Pattern:** A verified source is copied directly to its canonical destination.
Interruption leaves incomplete bytes there, and a no-overwrite policy then
prevents an ordinary retry. Integrity detection exists; recovery does not.

**Domain:** Data download, archive extraction and derived artifact admission.

**Proposed structural solution:** Stage and verify a unique same-directory file,
then publish with cross-platform no-clobber semantics. Existing valid or unknown
user files must never be replaced. Retrying after interruption must be safe.

**Status:** SOLVED
**Structural remedy:** `copy_checked` now stages, verifies and publishes with
no-clobber semantics. Injected interruption leaves no canonical partial file;
racing publication cannot overwrite another writer's destination.
**Test:** `test_data_vault_failures.py::FailureTests::test_interrupted_admission_retries_without_partial_destination`
and `test_racing_publication_never_clobbers` pass.
**Original evidence:** Portable-vault review T1: fault injection left two of eight bytes
at the destination; the next copy was refused. No production data was changed.

## PP-20: Successful exit with an unsatisfied setup or selection contract

**Pattern:** A command prints missing/conflicting resources or silently ignores
requested selectors, but returns success. Callers interpret success as readiness
even though the intended dataset or storage location was not selected.

**Domain:** Portable setup and batch-selection CLI boundaries.

**Proposed structural solution:** Action-specific argument validation, complete
selector validation and a strict readiness result with nonzero exit on missing
or stale links. Any repair must distinguish owned links from real user folders.

**Status:** SOLVED
**Structural remedy:** Action-specific parsers reject unused options; full State
validation rejects unknown selections; incomplete links return nonzero and
explicit repair is restricted to tool-owned links.
**Tests:** `test_cli_rejects_inapplicable_options_before_io`,
`test_mixed_unknown_state_rejected`, `test_incomplete_links_exit_nonzero`,
`test_real_link_relocation_and_local_directory_preserved` in
`tests/unit/test_data_vault_failures.py` pass locally.
**Original evidence:** Portable-vault review T2/L1/L3: missing links return 0, a 2010 option
dispatches the 2020 builder, and RI plus an invalid State silently selects RI.

## PP-21: Output identity mistaken for complete execution custody

**Pattern:** A successful admission receipt binds an output and part of its
builder, but omits the wrapper, configuration, invocation or runtime that
actually selected and transformed the inputs. Matching bytes do not identify
all execution dependencies needed for independent reproduction and diagnosis.

**Domain:** Scientific build/admission receipts and portable recovery workflows.

**Proposed structural solution:** A versioned receipt binds the selected input
and catalog hashes, executed source identities, resolved command, environment
and subordinate manifest/log hashes. Validate receipt completeness in tests.

**Status:** SOLVED
**Structural remedy:** Versioned v2 admission receipt binds wrapper/helper,
catalog/input/profile identities, command/State, runtime/native dependencies,
thread controls and manifest/report/log hashes. Missing builder evidence prevents
admission; output identity remains checked independently.
**Test:** `test_builder_admission_failures_and_complete_receipt` passes; isolated
Windows venv RI build/replay also passed with the new receipt.
**Original evidence:** Portable-vault review C1/C2: the wrapper's admission receipt has
only status, context hash and historical profile hash; its current State helper
is outside that profile. Canonical output verification still protects identity.

Review: [portable-data-vault roles check](../../signals/roles/check/portable-data-vault-roles-check-2026-09-10.md).

---

## PP-22: Output-gated owner validation can pass without executing

**Pattern:** A CI owner-validation step is conditioned on a planner output. If
the output is missing or incorrectly false, the owner step is skipped and the
job can appear successful without validating the changed owner domain.

**Domain:** Any CI workflow where one planning step emits a boolean used to
gate an owner-native build or test.

**Structural solution:** Derive expected ownership independently from the
base/head changed-path set and fail the planning step unless the planner's
selection exactly matches that independent oracle. Emit the gate output only
after all positive, negative, and mutation controls pass.

**Status:** SOLVED for the Ferris web-docs shadow lane

**Proved by:** `scripts/ci/ferris-validation-domains.mjs` compares actual
`web/docs` path membership with the selected `web-docs-build` entrypoint before
writing `web_docs_selected`.

**Test:** `scenario-web-only`, `scenario-unknown-fallback`, and
`scenario-prefix-boundary` fail if the owner gate is omitted, inverted, or
matched across a path-segment boundary.

---

## PP-23: Repository-local change parser drifts from planner semantics

**Pattern:** A repository reconstructs merge bases, rename handling, deleted
paths, and revision identity before calling an external planner. The adapter can
silently disagree with the planner or bind a plan to a different tested
revision even when both components are individually deterministic.

**Domain:** CI adapters that translate Git history into path arguments for a
separate selection or validation tool.

**Structural solution:** Pass the exact base, head, and tested revisions to the
planner and retain its revision binding as the sole change-set evidence. Keep a
narrow repository-owned oracle only for the owner-domain decision that gates
execution, not as a second general-purpose change parser.

**Status:** SOLVED

**Proved by:** `scripts/ci/ferris-validation-domains.mjs` no longer parses
name-status output or constructs changed/deleted path arguments. The actual plan
uses Ferris revision options, while `git diff --quiet -- web/docs` remains an
independent owner-selection oracle.

**Test:** The Ferris web-docs shadow workflow must retain a native
`revision_binding`, agree with the independent `web/docs` oracle, and complete
the repository-owned owner build.

Hosted run `33349539121` exercised 2,143 derived paths and retained a 2.04 MB
plan, proving the adapter's explicit output bound. Its plan, change-set, and
dirty-state binding identities matched an exact Windows replay.


## PP-24: Engine execution completion mistaken for candidate acceptance

**Pattern:** A scheduler records successful execution even when the native ordinary run returns an out-of-tolerance or disconnected candidate. A UI labels the count Succeeded and omits the candidate constraints.

**Structural solution:** Preserve execution status and receipt counters, but label their meaning explicitly. Display native population tolerance, maximum deviation and graph contiguity alongside every state, including multi-year results. Missing checks are unknown; failed rows cannot inherit attached candidate checks. Multiscale connectivity requires a nonempty Boolean vector before aggregation.

**Status:** SOLVED for the State/States result tables.
**Proved by:** Retained native CVD output remains completed with No tolerance / No contiguity, while a configured balanced candidate displays Yes / Yes. Original receipts remain unchanged.
**Test:** `scripts/web/test_state_candidate_results.mjs` (portable schema regression); `--retained-native` (explicit local native/WASM receipt acceptance). Both modes passed 2026-10-18. This presentation fix does not establish whole-application parity or legal validity.

---

## PP-25: Declaration coverage mistaken for executable parity

**Pattern:** A port inventories commands and flags, then treats a parser reply, UI control, shared-adapter comparison or passing test invocation as proof of a complete desktop workflow. Unsupported paths, skipped external-data cases, stale binaries and substituted backends can disappear from the apparent completion count.

**Domain:** Cross-platform engine/workflow ports and release support claims.

**Proposed structural solution:** Retain separate implementation/evidence states and explicit native source/behavior contracts per capability. Bind evidence to actual source/build/fixture identities and the advertised browser action. Capture test-source identity at execution; a later test extension requires an explicit affected-scope rerun rather than silently updating the earlier invocation. A release closure gate must refuse execution claims supported only by inspection, stale artifacts, not-run integration cases or a different backend. Keep expected native failures and successful accepted outputs distinct. Preserve the original unsupported rows instead of removing them from a denominator.

**Status:** OPEN. The structural inventory/closure checker and portable mutation regressions are implemented (`scripts/web/wasm_parity_ledger.py`, `scripts/web/test_wasm_parity_ledger.py`), and both browser build scripts invoke them. Inventory reconciliation, real acceptance-bundle migration, predeclared capacity rows and release-package integration remain incomplete; this is not whole-workflow closure.

**Plan:** [Desktop-to-WASM parity plan, WP-01/WP-10](../../docs/specs/wasm-desktop-parity-plan.md).

**Required regression:** Portable synthetic closure tests reject inspector-only/shared-adapter references, mismatched artifact/fixture bytes, skipped required checks, substituted backends and failure mislabeled as successful support. A valid scoped contract can coexist with explicit unresolved rows, which still block full parity. Real workflow/capacity/release evidence must additionally be migrated and exercised before this pitfall is solved. See [current ledger status](../../docs/specs/wasm-parity-ledger.md).

---

## PP-26: Source and seed identity mistaken for runtime-equivalent execution

**Pattern:** A port compiles identical C source and preserves the seed, but libc
RNG behavior differs. Normalizing labels can hide a mismatch; changing a generator
only on the new target silently changes the declared backend's semantics.

**Structural approach:** Pin the native runtime and measure its sequence. Retain
the unmodified mismatch before evaluating a named compatibility profile. Bind
shim/build/runtime assets and compare exact assignments and errors. Preserve
the existing default and restore runtime selection between requests.

**Status:** MITIGATED. The explicit `msvcrt-v1` sidecar and scoped engine tests
cover the measured Windows reference. Full method/runtime/custody acceptance is
still incomplete; no universal native libc equivalence is asserted.

**Tests:** `scripts/web/test_metis_c_sidecar_browser.mjs` preserves unmodified
runtime differences and checks the named profile against 320 measured CRT
values/24 kernel cases. `scripts/web/test_cffi_engine.mjs` compares actual BISECT
dispatch and checks default/post-C core equality; the packaged browser test
checks explicit runtime loading and retained artifact identities. Broader
release evidence remains subject to PP-25.

---

## PP-27: Selected-root observations outlive their live authority

**Pattern:** A browser inspects folder paths, then treats that observation or a
saved selection record as continuing authority to read/write. Permission loss,
an input mutation during queued reads or a picker resolving after project Open
can otherwise finalize stale work under a newer project or different root.

**Structural approach:** Keep live selection authority private and session-local.
Open/source/root changes retire it and generation-bound previews. Check current
read permission and metadata before referenced reads, recheck each referenced
file, and observe the live root again before committing a staged queue. Retain
prior accepted/staged inputs on failure. Installation still requires its own
currently selected root, fresh review and verified recovery contract. Retained
ZIP bytes are data; explicit reload and inspection are separate actions.

**Status:** MITIGATED for the selected native input workspace. Tests reject
copied/retired authority, injected permission denial, source edits before/during
queueing and a late picker after Open. Browser APIs do not lock other programs
out of a directory, provide an atomic filesystem snapshot or imply that every
OS permission/revocation/storage path has been tested. Queued bytes are retained
snapshots, not automatic directory watchers.

**Tests:** `scripts/web/test_native_workspace_source.mjs` checks source authority,
exact native-compared outputs and reviewed interruption/recovery.
`scripts/web/test_native_workspace_browser.mjs` uses actual workers/OPFS handles,
an injected picker/read-permission denial, live mutations before and during
queueing, inert Open/late-picker completion and exact recovered installation
without another partition. See the
[workspace contract](../../docs/specs/wasm-selected-input-workspace.md).

The scoped [label repository application](../../docs/specs/wasm-label-repository-installation.md)
extends the same mitigation with private single-use review grants, exact
current/upstream/absence rechecks, index/registry-last readback and explicit
known-byte continuation. `test_label_repository_installation.mjs` checks stale
and copied authority, changed sources, corrupted/unknown partial bytes and
permission loss; its browser counterpart exercises actual worker/OPFS effects,
inert Open, edits retiring review and cancellation after close. OS picker and
external concurrent-edit atomicity remain outside that evidence; PP-27 stays
MITIGATED.

---

## PP-28: Flag names mistaken for filesystem disposition scope

**Pattern:** A port treats "reset", "force" and "replace" as equivalent. Native
State's warning names the whole target but its reset call clears only data/.
Force permits a labeled manifest collision and does not by itself remove files.
Mapping these flags to a browser whole-target installer can erase unrelated
plans, contexts, maps or notes and can bypass native preflight ordering.

**Structural approach:** Trace the actual filesystem consumer and separate the
actions. Native incomplete/labeled-collision guards run before a data-only reset;
whole-target installation overwrite retains its own explicit review. Bind each
reset review to the current selected root, output package, force value and target
digest; consume it once. Preserve and verify every non-data entry. If cancellation
arrives after removal begins, record an interrupted/unverified effect rather
than pretending no files changed or declaring success.

**Status:** MITIGATED for the single-target explicit browser reset action. The
actual CLI preflight/effect matrix and browser OPFS tests pass. A scoped
selected-State before-Run connection now has actual CLI and packaged browser
checks; engine-only flags remain refused. Bulk completion/reprocess,
receipt custody and broader target/platform cases remain open. The browser does
not lock other programs out or undo a filesystem call already in progress.

**Tests:** `scripts/web/test_native_state_data_reset.mjs` retains 32 actual CLI
filesystem comparisons (all deliberately failing before partition), original
arguments/stdout/stderr/file trees and 25 refusals. The browser test covers
review-only behavior, only data/ removal, native force/incomplete order, changed
targets, post-removal cancellation, retained downloads and inert Open. See
[the scoped contract](../../docs/specs/wasm-native-state-data-reset.md).
The [before-Run contract](../../docs/specs/wasm-native-state-target-run.md) adds
ten actual CLI comparisons for unused backend declarations, required-backend
refusal before cleanup, source failure after cleanup and grant replay refusal.
Its browser regression pauses after real removal, cancels Run, and checks a
downloadable interrupted receipt. Pending authority now retires in the controls
update path; unchanged-result autosave carries only passive records across its
bookkeeping revision. Neither action records nor autosave restore a live grant.
The [bulk workspace contract](../../docs/specs/wasm-native-bulk-workspace.md)
adds live canonical/historical completion filtering before scheduled Run reset,
separate reprocess, per-target continuation and three-year receipt scoping.
Six actual CLI cases and packaged worker/OPFS cancellation checks cover that
bounded journey. Saved-workspace replay/recovery remains open; it cannot restore
authority from action records or repeat reset as a verification step.

## PP-29: Cleanup assumes a first result exists

**Pattern:** A batch fails before its first checkpoint. UI finalization previews
the absent result, throws, and never reaches normal busy/cancel cleanup. An
intended preflight refusal then strands the operator in a running-looking UI.

**Structural approach:** Treat no-result failure as a first-class outcome. Do
not preview absent results, retain the original diagnostic and allow another
explicit review/Run. Cleanup should not depend on successful generation.

**Status:** MITIGATED for workbench state-batch pre-checkpoint failures. The
absent result is rendered as empty. The browser regression refuses reset without
live authority, then reviews and successfully runs the same queue. This does not
establish cleanup against arbitrary renderer exceptions or every other workflow.

**Tests:** `scripts/web/test_native_bulk_workspace_browser.mjs` checks the initial
refusal leaves Run usable, no page errors occur, and a subsequent reviewed Run
works. See [the contract](../../docs/specs/wasm-native-bulk-workspace.md).

## PP-30: Archive receipts mistaken for derived-source consistency

**Pattern:** A saved job carries a raw archive, receipt and self-consistent job
hash. Checking only the receipt/hash can overlook that its derived source recipe
was changed independently of the original archive, then rehashed. A serialized
"verified" marker can also be mistaken for current continuation authority.

**Structural approach:** Explicitly hash and decode retained archives, bind their
state/year/kind/URL to fresh native planning, rebuild through the same constructor
as acquisition, and compare the complete queued recipe. Mint private live
continuation proof only after that check. Open and imported markers stay inert.
This proves internal source consistency, not provider authenticity.

**Status:** MITIGATED for paired state-acquisition recovery. Native/configured
Node and packaged browser cases pass, including changed raw archives, rehashed
derived recipes, changed settings, forged markers and retained-job-only Resume.
Other archive/receipt consumers and full custody/replay remain open.

**Tests:** `scripts/web/test_acquisition_recovery.mjs`,
`scripts/web/test_acquisition_recovery_configured.mjs`, and both modes of
`scripts/web/test_acquisition_recovery_browser.mjs`. See
[the contract](../../docs/specs/wasm-state-acquisition-recovery.md).

## PP-31: Aggregate verification success conceals missing stages

**Pattern:** A composed verifier reports a matching assignment replay. Consumers
interpret that aggregate status as proving original-source alignment, historical
producer identity or installed bytes even when those stages were missing or not
run. Accurate individual checks can still produce a misleading overall claim.

**Structural approach:** Version the stage outcomes and preserve missing,
failed and not-run separately. Bind any complete inspection claim to all required
stages. Treat replay similarity as its own result, and keep passive verification
records separate from filesystem/continuation authority.

**Status:** MITIGATED for selected native-output inspection/replay. The final
packaged browser test retains a matching replay with missing original-source
evidence in both the visible summary and saved result. Imported/incomplete
records remain unverified and cannot authorize filesystem actions. Other
verification consumers and full custody/continuation acceptance remain open.

**Required regression:** A matching replay with missing raw-source or producer
evidence must still expose that incomplete evidence in the actual packaged UI
and exported record. Imported success markers must not authorize reset, writes
or Resume. See the [acceptance review](../../signals/roles/check/wasm-desktop-parity-plan-acceptance-roles-check-2026-10-23.md)
and `scripts/web/test_native_output_verification.mjs`, plus
`scripts/web/test_native_output_verification_browser.mjs`. The
[scoped delivery](../../docs/specs/wasm-native-output-verification.md) binds both
actual build variants and the final-package worker regression to an immutable
local snapshot. This is not a full-parity or authenticity claim.

## PP-32: Computational continuation repeats destructive preparation

**Pattern:** A saved batch contains accepted computations and interrupted target
actions. Replaying the original outer workflow repeats reset before it discovers
which results can be retained. A serialized action/verification marker can also
be mistaken for permission to delete newly present data.

**Structural approach:** Verify completed computations in a worker that receives
no folder authority. Mint continuation authority only after a new live-root
review. Retain verified rows exactly, dispatch unfinished states only, disable
reset when a previously touched target has no data, and block reappearing data
for separate review. Historical action records remain passive and unauthenticated.
Re-observe target bytes before effects and preserve interrupted receipts.

**Status:** MITIGATED in scoped packaged native Run/States continuation. Node and
actual worker/OPFS cases exercise cancellation after real removal and subsequent
continuation without another removal, altered result/input refusal, changed
targets, completed installed bytes, three-year ordering and two-district RI.
Both actual engine variants and the final package pass their registered checks; broader lifecycle/release acceptance remains required.

**Tests:** `scripts/web/test_native_workspace_continuation.mjs` and
`scripts/web/test_native_workspace_continuation_browser.mjs`. See the
[workflow contract](../../docs/specs/wasm-workspace-continuation.md). No global
SOLVED status or source-authenticity claim follows from this scoped mitigation.

## PP-33: Acquisition completion promoted to downstream data readiness

**Pattern:** A downloader/extractor writes a completion marker even though its
planned target is absent, or its downstream parser/spatial join is unimplemented.
An operator or composed UI interprets successful acquisition as usable algorithm
input. Inventing the absent target or silently substituting a different source
conceals the mismatch instead of completing the consumer contract.

**Structural approach:** Keep raw-byte identity, extraction completion, planned
target presence, data validity and actual consumer support as separate outcomes.
Preserve native extracted files and marker without fabricating missing artifacts.
Show target presence in the result; expose consumer limitations. Native incremental
skip still requires target plus marker. Open and imported receipts confer no
execution, installation or source-authenticity authority.

**Status:** MITIGATED for the browser Fetch ZIP package. Six controlled native
download/FLETCH/extraction cases over three vintages reproduce school targets
and EIA marker-without-target outcomes; both build variants and explicit local/
cache routes preserve those distinctions. The actual retained RI school archive
has a separate native-byte comparison. Native ZIP completion and zone spatial-join
limitations remain; this is not a global SOLVED or valid-data claim.

**Tests:** `scripts/web/test_fetch_archive_acquisition.mjs`,
`scripts/web/test_fetch_archive_acquisition_browser.mjs` and the explicit retained
tier `scripts/web/test_fetch_archive_real.mjs`. See the
[archive acquisition contract](../../docs/specs/wasm-fetch-archive-acquisition.md).


## PP-34: Duplicate evidence copies exhaust local storage before acceptance

**Pattern:** An acceptance snapshot copies the same engine/native binary under
many source paths without estimating unique storage. Local disk exhaustion
leaves a partial tree that may be mistaken for a complete evidence bundle, and
can consume the space needed to finish or diagnose the acceptance workflow.

**Structural approach:** Hash all intended inputs before creating output, count
unique bytes, reserve workspace headroom and refuse insufficient capacity before
writes. Store each content hash once in a snapshot-owned blob store; map every
original path/hash/size to its blob. Verify copies and publish the manifest only
after all required blobs/source identities check. Preserve failed-attempt records;
never mark a partial tree accepted or use hard links to mutable source files.

**Status:** MITIGATED for the label installation snapshot helper. A one-byte
budget test refuses before creating output. The deduplicated evidence snapshot
checks copied hashes and retains the original disk-full failure record. Concurrent
external disk usage can still fail a copy; failures retain a failure record and
publish no completed manifest. This is not global disk-quota or atomicity support.

**Tests/evidence:** `scripts/web/snapshot_label_repository_installation.py`,
`target/label-repository-delivery/capacity-refusal.log`, the retained
`target/label-repository-delivery/package-snapshots/wasm-label-repository-validated/failed-attempt.json`
and the [scoped contract](../../docs/specs/wasm-label-repository-installation.md).


## PP-35: Native summary text interpolated as executable HTML

**Pattern:** A report renderer concatenates untrusted state/status or metadata
strings into markup. A restricted browser preview can hide the defect while
an original downloaded HTML file still executes supplied tags outside that
preview. Native/WASM agreement alone does not make the interpolation safe.

**Structural approach:** Escape every dynamic HTML text interpolation in the
shared native renderer. Preserve original data in JSON and safe text semantics
in HTML. Keep browser preview CSP, tag/attribute allowlists, source hashes and
inert project Open as separate defenses; do not silently rewrite only the
browser download and call it native parity.

**Status:** SOLVED for the shared native label/year renderer and the tested
native/WASM/browser delivery. Seven shared-renderer/security tests pass; the
escaping regression fails on the original implementation. Fresh native/default/C
file checks and actual browser/download/preview checks pass in the final package.
This is not a blanket claim about unrelated HTML renderers.

**Tests/evidence:** `bisect-report::label_summary::tests::supplied_html_characters_remain_text_in_every_interpolation`,
`target/native-label-summary-delivery/html-escaping-before-fix.log`, and the
[native summary contract](../../docs/specs/wasm-native-label-summary.md).


## PP-36: Identity-bearing fixtures outlive their tested engine

**Pattern:** A browser regression compares a new measured engine/runtime to an
expected receipt generated for an older module. Assignments may still match,
but exact artifact identity correctly fails. Patching the expected hash or
ignoring the identity field would turn a fixture problem into false evidence.

**Structural approach:** Run the fixture producer against the exact staged
engine before its consumer browser test. Re-execute the actual native/C
comparison where the fixture includes backend identity. Retain old fixtures
and failure records; publish a separate verification record over unchanged
staged bytes rather than rewriting historical acceptance outcomes.

**Status:** MITIGATED. The native-summary package builder now invokes
`test_tui_verify_wasm.mjs` and `test_explicit_c_state_batch.mjs` against its
actual packaged WASM before the corresponding browser checks. Exact equality
assertions remain intact. Broader builders still need the same pairing when
adding identity-bearing fixtures; this is not global fixture-cache closure.

**Tests/evidence:** Retained stale TUI/C fixtures and failed package records in
`target/native-label-summary-delivery/`, fourteen fresh TUI constituent
receipts, actual C/native assignments and the
[native summary contract](../../docs/specs/wasm-native-label-summary.md).


## PP-37: A prior filesystem receipt appears current after project Open

**Pattern:** A tab keeps a truthful receipt for an earlier disk operation, then
opens or prepares another project while the status panel still says completed.
Even without restoring any write authority, that wording can make a historical
effect appear to verify or install the newly loaded project.

**Structural approach:** Bind the displayed receipt to its in-session
revision/result/operation context. When that context changes, label it historical
and state that it does not verify/install the current project. Keep passive
receipt download separate from fresh single-use selected-root review authority.
A new review must display its concrete effects rather than stale success text.

**Status:** MITIGATED. The rename controller now binds receipt presentation to
its current preparation context. An actual Chrome regression fails before the
fix and passes afterward, while Open leaves request, picker and write counts
unchanged. Current registered/final-package checks and broader writer-panel
coverage are separate closure obligations; no global UI or write-authority claim.

**Tests/evidence:** `scripts/web/test_label_rename_application_browser.mjs`,
`target/label-rename-application/receipt-history-before-fix.log`,
`target/label-rename-application/receipt-history-after-fix.log`, and the
[rename application contract](../../docs/specs/wasm-label-rename-application.md).


## PP-38: Different label strings alias the same filesystem directory

**Pattern:** An exact source-equals-destination check accepts labels differing
only in case. On a case-insensitive host, forced destination removal then removes
the source itself, and the subsequent rename fails after data loss.

**Structural approach:** Reject ASCII case-only labels in the shared validation
before touching the native registry or filesystem. Use the same portable policy
on every host; changing case requires a distinct temporary label. Browser root
observation still needs alias/type/collision refusal and cannot claim OS locking.

**Status:** SOLVED for the shared ASCII label guard and rebuilt native CLI.
An owned Windows destructive baseline and failing-before regression are retained.
Five shared tests, 31 serial native label tests and four actual CLI cases prove
source/registry preservation with both force settings and case directions.
WASM already refused these labels; new engine/package gates remain required.
This does not solve unrelated symlink, concurrent-writer or transaction behavior.

**Tests/evidence:** `scripts/web/test_native_case_alias_cli.py`,
`label_rename::tests::case_only_names_are_refused_before_filesystem_effects`,
`label_cmd::tests::test_mv_case_only_names_preserve_all_source_stages`, and the
[rename application contract](../../docs/specs/wasm-label-rename-application.md).


## PP-39: Clock normalization conceals downstream provenance changes

**Pattern:** A generated index is also a later input, through repeated years or
source/output overlap. Different timestamps change its bytes and a later source
SHA-256. Removing a top-level timestamp during comparison cannot justify removing
or replacing the downstream hash. A final flattened file map can conceal the
intermediate difference if a later year overwrites the evidence.

**Structural approach:** Preserve ordered transitions and original witnesses.
Give native orchestration an explicit recorded-clock entry point using the same
filesystem control flow; keep the ordinary CLI's separate live clock observations.
Compare recorded-clock outputs and upstream hashes exactly. Report live-clock
hash differences as differences; never normalize them into matches.

**Status:** MITIGATED. A self-overlap assertion failure is retained under
`target/native-summary-application/run-1792815136411`; source tests now retain
ordinary CLI and fixed-clock native outputs separately. Ordered browser writer
and all-prefix recovery checks pass on the default retained engine. All 42 native report tests and fresh default/C source/browser checks now pass.
Both registered profiles and all 27 final-package gates now pass with an
8,787-mapping verified inherited snapshot. The named recorded-clock comparison
scope is demonstrated; other multi-stage comparators and broader lifecycle
acceptance remain open, so the global status stays MITIGATED.

**Tests:** `scripts/web/test_native_summary_application.mjs`,
`test_report_clock_keeps_distinct_live_observations`, and
`test_report_clock_recorded_self_overlap_hashes_exact_previous_index`.


## PP-40: Frontend diagnostic drift changes persisted native result artifacts

**Pattern:** Two frontends use the same retry policy and reach the same failed
state, but persist different error text in a hashed year index. Treating all
errors as interchangeable, or normalizing the error field, hides a real artifact
contract difference even when success/failure counts match.

**Structural approach:** Preserve typed failure details across the portable
engine boundary. Share the native population/state error formatter with the CLI,
and use its ASCII-safe state-prefixed wording in label-build indices. Retain the
full retry diagnostic separately; other configured/multiscale interfaces keep
their existing detailed error contract. Compare index error bytes exactly.

**Status:** MITIGATED pending source/fresh-WASM/final-package acceptance. The
original actual CLI versus browser partial-index failure is retained in
`target/build-application/native-host-application-v2.log` and its owned run tree.
The native CLI pilot succeeds with tracked synthetic RADJ sources; live Build
preparation/output/recovery and broader failure classes remain under development.

**Tests:** `scripts/web/test_label_build_application.mjs`,
`balance_retry::tests::native_failure_wording_preserves_state_and_ascii_policy`,
`engine_build::failure_tests`, and `label_build::preflight_tests`.


## PP-41: Completed worker output is lost while waiting for the final reply

**Pattern:** A worker emits an accepted state and continues with downstream work,
but the caller retains only the final command reply. Cancellation in between can
lose a completed accepted result even though the worker delivered its bytes.
Native-host recovery tests do not exercise this transport boundary.

**Structural approach:** Bind live progress to the private loaded-worker request,
validate sequence, label, state/year, roster and hashes, and retain completed
state artifacts before awaiting the final reply. Drain already received events
on cancellation. Compare any final reply against retained state bytes. Stale
requests and saved projects cannot enter this live callback. Year completion is
a separate filesystem/index/registry transition. Open restores passive data only;
explicit continuation replays retained results and re-observes the selected root.

**Status:** MITIGATED pending registered regressions and final-package acceptance.
Both fresh default/C actual worker/OPFS browsers now cancel immediately after the
completed-state callback, save/open the interrupted build, and retain/replay its
state without repeating partition after confirmation. Broader host, permissions,
capacity and lifecycle acceptance remain separate.

**Tests:** `scripts/web/test_label_build_application.mjs` (four synthetic transport
outcomes), `scripts/web/test_label_build_application_browser.mjs` (actual worker
cancellation and inert archive/recovery), and
`scripts/web/test_label_build_application_ui.mjs` (controller/source freshness).


## PP-42: Evidence writes outgrow available validation storage

**Pattern:** Completed engine results fit their individual source/output limits,
but raw replies, reference copies, browser archives and temporary downloads together
exhaust the validation host's disk. A failed write may leave partial evidence;
completion of computation cannot turn those files into a passing acceptance record.

**Structural approach:** Check evidence-storage capacity before expensive validation,
record insufficient capacity as not-run, and run large evidence producers serially.
Keep original interrupted attempts and accepted source packages. Lossless NTFS
compression of verified owned evidence/inactive build-cache files can recover
space without deleting records; verify before/after hashes. Retry into a fresh
owned directory, never over an interrupted record.

**Status:** MITIGATED. A preflight cannot reserve OS-wide space against other
processes, so later I/O errors must remain interrupted/failed. The two observed
ENOSPC attempts (map/archive and national fixture preparation) remain retained.
The accepted Build snapshot was already verified before these attempts and is
unchanged. Capacity guards apply to validation evidence, not a browser product
minimum or a claim about an average user's machine.

**Tests:** `scripts/web/test_full_chamber_prerequisites.mjs` includes an actual
capacity-refusal path with no engine execution. `fixtures/matrix-prerequisites.mjs`
records typed not-run outcomes for missing prerequisites and insufficient capacity.

## PP-43: Aggregate result limits strand a valid complete workload

**Pattern:** Every individual State Run succeeds, but the combined queue exceeds
its retained-receipt limit. The remaining accepted work cannot be retained, and
an independent archive value limit prevents saving even the completed prefix.
Repeating full receipts in both the outer index and nested archives also makes
the result ZIP exceed its configured source budget.

**Structural approach:** Test the complete declared workload through Run, Save,
Open, recovery and export. Keep explicit aggregate limits consistent across live
checkpoints and portable archive readers. For large exports, store exact full
receipts once in state archives, bind them by digest in a versioned outer index,
and use standard lossless ZIP deflate. Bound expanded and packed output separately;
do not raise generic project/import limits or drop states to obtain a pass.

**Status:** MITIGATED. The actual 50-state staged browser witness is retained at
`target/national-matrix/browser-1792828755437/evidence.json`: four workers reached
the 96 MiB receipt guard; Save then refused excessive values. Source corrections
use a finite 128 MiB receipt cap, eight million workbench values, and a 256 MiB
expanded / 128 MiB packed large-export boundary. Ordinary project and generic
JSON bounds remain unchanged. Actual corrected browser/recovery and final scoped
package acceptance must still pass before broader closure.

**Tests:** `scripts/web/test_national_matrix_browser.mjs` (actual full roster and
saved recovery); `scripts/web/test_national_batch_zip.mjs` (all 50 exact receipts,
accepted plan/context bytes, independent Python ZIP/CRC checks and hostile paths).

## PP-44: An evidence producer captures its own changing output

**Pattern:** A snapshot collects a live producer log, hashes its initial bytes,
then appends progress to that same log while copying the inputs. The captured
identity no longer matches the bytes. Accepting the partial tree would falsely
describe an immutable evidence bundle.

**Structural approach:** Keep live output outside the capture tree, or declare
its explicit exclusion as an unbound informational log. A required pinned stage
log cannot be excluded. Copy/read-back digest checks must refuse any changed
input before emitting a snapshot manifest. Preserve failed copies and retry in
a fresh directory; never rewrite a completed or partial snapshot in place.

**Status:** MITIGATED. The first national freeze refused its own changing log,
and emitted no manifest. Its failed directory, original log and mismatch record
remain retained. The snapshot helper now records explicit live-log exclusions,
refuses exclusions of required evidence pins and permits a fresh owned output
directory. Other concurrent writers can still change a source; read-back guards
must continue to refuse rather than certify those bytes.

**Tests:** `scripts/web/snapshot_national_browser_delivery.py`; the actual negative
witness is `target/national-matrix/queue-source-before-1792828755437/first-freeze-failure.json`.
Corrected full snapshot verification remains required before scoped acceptance.


## PP-45: An error display discards the recoverable operation outcome

**Pattern:** A composed UI catches an error and displays only its message, losing
an attached receipt that records earlier completed effects. Users cannot easily
distinguish an interrupted multi-target operation from one that wrote nothing.

**Structural approach:** Retain a passive, versioned in-session outcome for each
attempt. Preserve the exact receipt when produced; explicitly encode absence and
unknown completed writes otherwise. Offer a local download that grants no folder
authority. Clear it on project/source replacement and disclose its limited lifetime.

**Status:** MITIGATED for reviewed native-output installation/recovery. Both staged
shells pass exact cancelled/completed batch-receipt downloads, single-target null
receipts, changed-target refusal, no-dispatch/no-write downloads and inert Open.
Crash-time automatic custody and broader composed workflows remain open.

**Tests:** `scripts/web/test_native_state_recovery_browser.mjs` and
`test_native_state_batch_recovery_browser.mjs`; eight staged outcomes at
`target/install-outcome/validation-1792857149850507900/evidence.json`.


## PP-46: Persisted intent is mistaken for completed filesystem effects

**Pattern:** A page closes after recording an installation attempt but before a
final receipt. On reopening, a UI may treat the record as success, absence of writes
or recovery authority. Another tab may also replace a newer attempt with old data.

**Structural approach:** Persist an explicitly incomplete observation before
attempting effects, separate from folder grants. A final record uses an atomic
attempt-ID comparison. Restored progress means unknown writes; inspect live files
before recovery. Forget uses the same ID guard. Bound and validate stored bytes.

**Status:** MITIGATED for passive native-output history. Both staged browsers
reload during a real write, observe only the prior progress record/incomplete
marker, dispatch nothing and require a fresh folder review. Stale-tab final save
and Forget refuse. Storage unavailable/quota/size cases remain explicit failures;
eviction or death before the initial commit can still prevent custody.

**Tests:** `test_installation_outcome_store.mjs`, both native-state recovery browser
gates; nine outcomes at
`target/install-outcome/validation-1792864144904949300/evidence.json`.


## PP-47: Display geometry silently replaces physical source measurements

**Pattern:** A grouped file importer treats a display/simplified polygon companion
as supplemental physical source geometry. It may change measurements embedded in
an accepted graph, or cause native preparation to reject an otherwise valid set.

**Structural approach:** Classify file roles explicitly. Preserve embedded physical
attributes, leave display geometry separate and require explicit native preparation.
Never weaken the native geometry override guard to make an import succeed.

**Status:** MITIGATED for grouped native release-file import. The first attempt
was refused by the native guard; the corrected importer never reads display bodies,
and both profiles pass exact native/WASM preparation and real browser input checks.
Other import/map/analysis paths retain their separate geometry contracts.

**Tests:** `test_native_adjacency_bundle.mjs` and
`test_native_adjacency_bundle_browser.mjs`; passing ten-stage record at
`target/native-adjacency-bundle/package-1792872648628282300/evidence.json`.
Original failure and source bytes are retained in the scoped source-before tree.


## PP-48: A private source credential outlives or follows its request authority

**Pattern:** A source token is serialized into a project/receipt or retained by
browser HTTP caching, or an asset redirect carries its Authorization header to a
separate download host. Imported discovery data may also be mistaken for a live
credential/request grant.

**Structural approach:** Bind memory-only credentials to private live selections;
retire on completion/cancel/Clear/project changes. Never serialize credentials or
signed URLs. Omit cookies/referrers and use no-store. Construct provider endpoints,
check bounded metadata/body identities and verify cross-origin header stripping.

**Status:** MITIGATED for the release client under controlled real-browser network
interception on both shells. Actual redirects strip the test credential; saved
projects/receipts contain neither credential nor signed URL. Real native gh access
keeps its credential in gh. Live browser-token/CORS acceptance is still unverified;
no protection from privileged same-origin scripts or secure heap erasure is claimed.

**Tests:** `test_adjacency_release.mjs`, `test_adjacency_release_browser.mjs` and
`test_adjacency_release_live.mjs`; final scoped package/live records are identified
in `docs/specs/wasm-adjacency-release-acquisition.md`.


## PP-49: A CORS workaround silently delegates a private credential or becomes a proxy

**Pattern:** A browser-only credential is forwarded to a hosting server without an
explicit decision, or a source bridge accepts arbitrary destinations and forwards
Authorization/cookies through redirects. A successful local check can also be
misrepresented as deployed-host acceptance.

**Structural approach:** Unchecked, clearly labeled site-bridge option; same-origin
POST and exact protocol/body; configured repository/tag and independently discovered
asset membership. Construct each upstream hop's headers, use recognized HTTPS
asset hosts only, omit auth after the API hop, and return only bounded bytes with
no-store. Client independently validates published length/digest before commit.

**Status:** MITIGATED for the local production-handler/browser scope. Both actual
private-provider browser profiles pass; invalid origin/scope/asset/redirect and
stream overflow/short/deadline/cancel/abort refuse. Hosted runtime, rate controls,
logging and intended-origin acceptance remain unverified. Host compromise and
privileged scripts remain outside the memory-only guarantee.

**Tests:** `test_adjacency_release_bridge.mjs`,
`test_adjacency_release_bridge_streams.mjs` and explicit opt-in
`test_adjacency_release_private_browser.mjs`; see
`docs/specs/wasm-adjacency-release-bridge.md` for exact retained records.


## PP-50: A source schema conversion silently changes vote meaning or source identity

**Pattern:** Raw provider data is relabeled as canonical election CSV without an
explicit party/year mapping, fractional values are rounded, leading-zero IDs are
lost, or multiple raw IDs collapse into an implicitly aggregated tract. Replacing
raw source bytes also destroys the ability to inspect the transformation later.

**Structural approach:** Explicit format record, exact declared column mapping,
preserved decimal lexemes, bounded strict CSV parsing, duplicate detection after
ID padding and independent original/canonical hashes. Retain raw project bytes;
convert only on explicit analysis or source-package export. Preserve native source
coverage and accepted-engine-provenance guards.

**Status:** MITIGATED for bounded synthetic Fekrazad-2020 mapping. Independent Python
CSV and native/WASM results match; both actual browser profiles retain raw source,
format and seed without execution on Open. Actual provider/vintage data and other
source schemas remain unverified; no election certification claim follows.

**Tests:** `test_election_source_format.mjs`,
`test_election_source_format_browser.mjs`, project security/handler regression;
see `docs/specs/wasm-election-source-format.md` for exact package and live403 records.


## PP-51: Opaque source artifacts or completion markers become unintended authority

**Pattern:** A full source download drops opaque files needed by a native consumer,
executes them during browser import, or treats a generated done marker as source
validation/installation evidence. Cancellation can also land after acquisition but
before package hashing ends, causing a supposedly cancelled ZIP to be offered.

**Structural approach:** Separate explicit full-bundle purpose from engine-input
import. Preserve exact bounded wildcard bytes without decoding; generate fixed
native layout/marker only after whole-source validation. Record primary presence,
installed=false and authenticity=false. Reject unsafe names, case aliases and
marker collisions. Check revision/signal during package hashes and before offering
any ZIP; portable data never grants a target handle or operation authority.

**Status:** MITIGATED for the bounded local release ZIP. Both actual private-provider
browser profiles match fresh native gh Fetch outputs. A targeted real-browser test
fails the initial prototype with one post-Cancel ZIP and passes final v2 with zero;
no engine runs. Actual target skip/force/application, hosted policy and wider source
coverage remain outside this slice.

**Tests:** `test_native_fetch_release_package.mjs`,
`test_native_fetch_release_names.mjs`, `test_native_fetch_package_cancel_browser.mjs`,
both-profile release browser journeys and explicit live private checks; exact
records are identified in `docs/specs/wasm-native-fetch-release-bundle.md`.


## PP-52: A resolved directory lookup outlives its write context

**Pattern:** Cancellation or a project change arrives during an awaited directory
lookup. Checking only before the await allows a subsequent createFile call, leaving
an empty file after cancellation even when no byte stream is ever committed.
A serialized review can also be mistaken for a current folder write capability.

**Structural approach:** Private single-use root-bound review, copied source bytes,
current scope/permission/fingerprint checks and post-await context checks before
further I/O. Mark possible effects before directory/file creation, invalidate an
old native marker before source replacement, verify readback and write marker last.
Open drops the root/review and resets Force; passive outcomes grant no authority.

**Status:** MITIGATED for bounded local Fetch application. A targeted final-lookup
fault creates one empty source file on the prototype and zero on final v2. Native
CLI supported final trees, both actual browser UI profiles and real Chrome OPFS
write/cancel/review recovery pass. Real external native-folder grants, cross-process
locking, close/reload custody and broader hosts remain unverified.

**Tests:** `test_native_fetch_install.mjs`,
`test_native_fetch_install_browser.mjs`, `test_native_fetch_install_opfs.mjs`,
`test_native_fetch_install_await_cancel.mjs`; exact records are in
`docs/specs/wasm-native-fetch-folder-install.md`.


## PP-53: Passive progress is mistaken for durable filesystem custody

**Pattern:** A page writes files but closes before its terminal outcome is saved.
A missing or stale local record can imply nothing happened, while another tab can
replace the latest record before the old tab saves or forgets it. Restoring a
record as execution authority compounds the error.

**Structural approach:** Attempt an in-progress checkpoint before effects; save
the terminal receipt using an atomic expected-attempt-ID transaction. Unknown
effects stay unknown after close. Separate Fetch/Run namespaces, guarded reads,
strict bounded passive schemas and explicit reselect/review prevent authority
restoration. Storage errors keep a warning and session download.

**Status:** MITIGATED. Real Chrome IndexedDB/OPFS close/reload and two-tab CAS
checks pass. Filesystem/storage atomicity, native permissions, eviction, process
crash and power-loss custody remain open; one latest record is not an archive.

**Tests:** `test_fetch_install_outcome_store.mjs`,
`test_fetch_install_outcome_browser.mjs`; contract
`docs/specs/wasm-fetch-outcome-history.md`.


## PP-54: Debugging transport disconnect is mistaken for process termination

**Pattern:** A test disconnects a remote debugging client and assumes the browser
process has exited. The browser can remain alive, hiding a crash-durability gap,
leaking its profile and hanging teardown while waiting for an exit never requested.

**Structural approach:** Launch an owned browser in a fresh profile, retain its
child-process handle and observe authoritative exit. Crash injection targets only
that live owned PID and its process tree. Cleanup explicitly terminates a remaining
owned process after debugging disconnect. Resolve/check the profile boundary before
recursive removal, and retain no raw profile in evidence.

**Status:** MITIGATED. The first harness exposed a teardown hang after passing its
checkpoint assertions; final gates explicitly terminate and observe the owned
process, pass restart checks and remove the checked profile. Other host/browser
process-lifecycle APIs require their own tests.

**Test:** `scripts/web/test_fetch_install_outcome_crash.mjs`.


## PP-55: Diagnostic declarations are mistaken for algorithm effects

**Pattern:** A flag is copied into native configuration, so a browser removes its
refusal and invents logging, timing or algorithm behavior. Alternatively, progress
metadata is added to strict round events and invalidates their existing schema.

**Structural approach:** Trace downstream consumers. Bind only implemented browser
orchestration choices to original flags; reconstruct the binding before source or
partition. Preserve native inactive debug, route STATUS position separately, keep
strict round events unchanged, and retain partition-timing refusal until the same
interval exists. Bound retained status events and disclose omissions.

**Status:** MITIGATED. Independent native assignments, eight real CLI status cases
per profile, stale-setting refusal, raw round passthrough and both actual packaged
worker/UI flows pass. Timing and broader methods/hosts remain open.

**Tests:** `test_state_diagnostic_flags.mjs`,
`test_state_diagnostic_flags_browser.mjs`.


## PP-56: Whole-job elapsed time is substituted for partition-attempt timing

**Pattern:** A native timing flag wraps each dispatcher return, but its browser
adapter measures an entire queued job, compares host numbers as reproducibility
evidence, or reports an unavailable clock as zero. Retry, preparation and export
work then receive a misleading partition-time label.

**Structural approach:** Opt-in capture at the dispatcher boundary, before final
population validation; exact attempt/seed/scope metadata; separate timing events;
owned dropped-capture cleanup and bounded reports with omission counts. Keep
normal/review measurement paths inactive. Treat invalid clocks as unavailable/null,
retain raw values outside canonical plan/replay identity, and disclose native BG
file-work differences rather than promising comparable timings.

**Status:** MITIGATED. Controlled boundary clocks, first/single/later/exhausted
attempts, wrapping seeds, custom/BG/adaptive paths, native non-time reply equality
and actual C worker/native checks pass. Wider method/source/host acceptance remains
open; this is operational instrumentation, not a benchmark.

**Tests:** `test_partition_timing.mjs`, `test_partition_timing_browser.mjs`,
`test_partition_timing_c_browser.mjs`, native `partition_timing::tests`.


## PP-57: A no-overflow assertion is mistaken for readable mobile tables

**Pattern:** A table wraps every character and stays within the viewport, so a
width assertion passes while headings and exact numeric identifiers become hard
to read. Functional DOM checks do not detect the visual failure.

**Structural approach:** Inspect actual narrow-view screenshots. Move shared
context into the note and reduce the timing table to attempt, seed and elapsed
time while retaining full data in the receipt. Keep wider pre-existing tables as
explicit UX debt rather than claiming the whole workbench is mobile-polished.

**Status:** MITIGATED for the new timing table. V1 functional pass/cramped screenshot
is retained; V2 three-column table and both browser profiles pass and were visually
checked. Broader table layouts remain open.

**Test:** `test_partition_timing_browser.mjs`; retained prototype and final images
are recorded in `docs/specs/wasm-partition-attempt-timing.md`.

## PP-58: Concurrent evidence jobs overcommit validation-host disk

**Pattern:** Each integration job checks free space independently, then concurrent
jobs and browser temporary downloads exhaust the same volume. Successful engine
executions are mistaken for successful export or archive validation.

**Structural approach:** Reserve the combined peak evidence/temporary-download
budget across an owned validation batch, or serialize large phases with a fresh
capacity check between them. Require terminal archive/map checks, preserve failed
attempts and never promote a completed engine job alone to passing evidence.

**Status:** OPEN for shared capacity reservation and an injected competing-writer
test. The national refresh preserves its ENOSPC/cancelled-download failures and
repeats required checks after removing only rebuildable incremental caches. This
local recovery does not establish a general capacity guarantee.

**Evidence:** `docs/specs/wasm-national-engine-refresh.md`; existing
`preflightMatrixCapacity` is a per-job guard, not a cross-process reservation.

## PP-59: Editing a native manifest through JavaScript rounds an exact seed

**Pattern:** Rust emits a correct JSON-text manifest containing an unsigned seed
above JavaScript's safe integer range. A browser adapter parses and reserializes
the object merely to add filename metadata. The resulting manifest records a
different seed even though the request and accepted plan retain the original.

**Structural approach:** Perform named-source manifest changes on the typed
native `PlanManifest` and preserve the resulting JSON text through browser
packaging. Keep selected-source identity separate from source authentication;
do not fabricate executable or Census provenance. Check the final file's exact
integer digits, not equality between two already-rounded JavaScript numbers.

**Status:** MITIGATED for named State output. The first staged adapter reproduced
the rounded maximum-u64 value and its failed check is retained. The replacement
Rust path preserves `u64::MAX`; raw-text seed assertions cover output preparation
and browser ZIP journeys. Other serialization boundaries remain separate work.

**Tests:** `plan_audit_text::named_manifest_tests`,
`test_state_name_workflow.mjs`, `test_state_name_workflow_browser.mjs`.

## PP-60: An unused selection result hides a separate output-source consumer

**Pattern:** A flag's calculated selection is unused by the partition engine, so
the port treats the whole flag/source workflow as inert. A later report or audit
writer independently discovers the same files, changing context or failing output.
An assignment-only comparison misses the difference.

**Structural approach:** Trace consumers through final artifacts, including
implicit source discovery. Bind partition populations, optional report counts,
requested manifest metadata and diagnostics separately. Test both complete output
and native failure stages with absent, valid and malformed named files.

**Status:** OPEN for browser workflow integration. Eleven actual native attempts
show identical successful population vectors but independent CVAP report loading;
empty-file and directory cases fail audit-sidecar construction. The initial
incorrect successful-output expectation is retained. No browser mapping is
accepted by this native investigation alone.

**Evidence:** `prepare_population_source_native_reference.py`;
`docs/specs/wasm-native-population-source.md`.

## PP-61: Concurrent tests share a timestamp-named fixture directory

**Pattern:** Two processes start in the same millisecond and recursively create
the same timestamp-based path. A deletion or mutation test in one process changes
the other's inputs, producing misleading failures or an accidental pass.

**Structural approach:** Allocate each mutable test root with atomic unique
directory creation. Keep profile evidence separate and retain failed concurrent
attempts. A timestamp remains descriptive metadata, never a uniqueness guarantee.

**Status:** MITIGATED for the report-source snapshot harness. The first parallel
default/C run collided in one directory. Both profiles pass after `fs.mkdtemp`
allocates separate owned roots. Other timestamp-based producers remain separate
maintenance work.

**Test:** `test_native_report_source_snapshot.mjs`; retained collision logs and
the pre-fix harness are under `target/state-report-sources`.

## PP-62: Follow-up regenerates a manifest that conflicts with final output

**Pattern:** A State job has accepted plan/context and selected adjacency identity.
Follow-up analysis generates a generic manifest, while final output generates a
named-source manifest. Both are locally valid, but file assembly rejects their
different bytes; patching the file afterward would leave report hashes stale.

**Structural approach:** Generate the containing workflow's typed native manifest
once before analysis. Supply that same text to analysis before package hashing and
report assembly. Standalone analysis keeps its generic manifest path. Never edit
exact-integer JSON through JavaScript merely to reconcile metadata.

**Status:** MITIGATED for the retained State workflow fixtures. The observed
conflict and failed logs remain under `target/state-report-workflow`; the default
WASM now passes original-flag execution with selected analysis/report stages and
final output. Actual packaged browser journeys remain separate acceptance work.

**Tests:** `test_native_report_state_workflow.mjs`, especially the valid-CVAP
follow-up/output case and exact maximum-u64 manifest assertion.

## PP-63: A polling test misses a transient successful UI status

**Pattern:** Open updates a status after import, then an autosave replaces it
300 ms later. A browser test that polls for the current text can time out even
when import completed, especially when rendering frames are delayed.

**Structural approach:** Install an observer for the exact expected import
transition before triggering the action. Retain the observed result for later
assertion, check saved input/result contents separately, and surface import errors
immediately. Do not broaden the success predicate to accept an arbitrary autosave.
When using MutationObserver, inspect exact added text nodes in its mutation
records: reading only current text in the callback can still miss coalesced
intermediate transitions. Retain structured observations on failure.

**Status:** MITIGATED for the folder and portable-source recovery harnesses: both
registered profiles pass with a shared preinstalled import observer and
independent project-data checks. The first registered portable run's timeout and
source preimages remain retained alongside its passing successor.
An integrated C run later exposed the final-text observer weakness. A controlled
DOM regression now deterministically reproduces the old oracle miss, verifies
coalesced success and rejects autosave-only/wrong-filename cases. Actual C and
core UI suites separately verify product behavior; the old specific timeouts are
not retroactively relabelled as passing.
The first retained timeout lacked enough diagnostics to prove its specific cause;
it is not retrospectively relabelled as successful. Other polling tests remain
separate maintenance work.

**Tests:** `test_native_report_folder_browser.mjs`,
`test_native_report_source_ui_browser.mjs`, `fixtures/observed-project-open.mjs`;
`test_observed_project_open.mjs`; retained sources/logs under
`target/state-report-folder`, `target/state-report-regression` and
`target/report-cffi`.

## PP-64: Publishing results before fallible cleanup completes

**Pattern:** An operation validates and publishes output, then frees foreign
allocations. If allocator cleanup traps, the caller receives failure but its
output memory already contains results. Error handling can then consume data
from a failed operation.

**Structural solution:** Stage validated results privately. Publish only after
cleanup completes successfully. A trapped foreign instance is retired without
additional allocator calls; retry requires a fresh instance. Ordinary allocation
refusal frees prior allocations and may retry the same instance.

**Status:** SOLVED for the synchronous C METIS bridge. Controlled real WASM traps
at each of five allocator/free positions and partition execution leave all Rust
caller memory unchanged. The old bridge fails this oracle on a cleanup trap.
Rust-engine backend restoration and fresh C recovery pass in Node and an actual
dedicated Chrome worker. Real exhaustion, process termination and other output
pipelines are separate open scopes.

**Tests:** `scripts/web/test_cffi_fault_recovery.mjs`,
`scripts/web/fixtures/cffi-fault-cases.mjs`; evidence
`target/cffi-faults/run-mjfLlb/evidence.json`, retained preimage
`target/cffi-faults/metis-c-sidecar.before.js` and `before-failure.log`.

## PP-65: A loaded-runtime cache retains a permanently retired instance

**Pattern:** A foreign runtime correctly retires after a fatal trap, but its
worker-level loaded flag remains true. Each explicit retry then reuses the
unusable instance indefinitely, without explaining how users can recover.

**Structural solution:** Read retirement from private loaded authority, invalidate
the worker's loaded cache, and report fixed recovery guidance. Keep the failed
outcome. Only a subsequent explicit Run may fetch and instantiate a fresh runtime.
Ordinary recoverable errors must not discard a usable instance.

**Status:** SOLVED for the current C METIS workbench/worker path. Genuine WASM
partition/free traps trigger a second verified asset load only on another Run;
controlled allocation refusal retries with the original instance. All three
actual UI cases retain failed requests, block completed export and save inertly.
Recovered assignments match the retained native fixture. Previous worker code
fails the retirement/recovery assertion. This is a source-overlay test; final
packaged release, real OOM/termination and other host recovery remain open.

**Tests:** `scripts/web/test_cffi_worker_recovery_browser.mjs`; evidence
`target/cffi-worker-recovery/run-aVT2cS/evidence.json`, previous sources under
`target/cffi-worker-recovery/before` and `before-failure.log`.

## PP-66: Shared adapter comparison misses CLI orchestration refusals

**Pattern:** Native-host and browser tests call the same adapter and agree, while
the actual CLI adds a guard around their shared kernel. The browser can then
accept an outcome the desktop command refuses. Source pinning the CLI alone does
not prove its orchestration was executed.

**Structural approach:** Compare the actual CLI on the same raw inputs and assert
success and failure boundaries before claiming workflow parity. Preserve useful
browser-only behavior under a distinct named operation and bounded contract.

**Status:** MITIGATED for ceremony verification. Actual CLI reproduction exposes
the completed-transcript requirement for a complete tree. Browser Verify now
matches its exact refusal; separate prefix inspection retains the earlier useful
feature. Actual CLI-generated Start/Announce/Halt transcript values and six real
UI actions are checked. The old browser fails the CLI contract oracle. Remaining
commands still need their own orchestration comparisons; this does not validate
all shared-adapter workflows or external witness/publication claims.

**Tests:** `scripts/web/test_ceremony_cli_contract.mjs`,
`scripts/web/test_ceremony_cli_browser.mjs`, `scripts/web/test_ceremony_wasm.mjs`,
`scripts/web/test_project.mjs`; before evidence under
`target/ceremony-native-parity-audit`, current evidence under
`target/ceremony-native-contract`.

## PP-67: Browser metadata guards narrow a native scalar domain

**Pattern:** A browser guard applies a signed range to an unsigned native metadata
field, even though the full value requires no additional memory or computation.
Valid native records then fail in the browser. Parsed JSON comparisons can also
hide rounding when both sides pass through JavaScript numbers.

**Structural approach:** Trace the native field type and actual command outcomes.
Keep exact canonical decimal text through the UI/project boundary and typed JSON
construction, rather than clamping or rounding. Distinguish necessary, declared
resource limits from an accidental scalar-domain restriction.

**Status:** MITIGATED for ceremony review duration. Native Start accepts u64 MAX;
the old browser configuration guard refuses it. The current guard and source form
preserve the full u64 value. Raw digit assertions, actual CLI Start/Verify/Halt
and unreachable-announce failure checks cover the observed boundary. Other scalar
and workflow contracts still need their own reconciliation.

**Tests:** `scripts/web/test_ceremony_source_start.mjs`,
`scripts/web/test_ceremony_source_start_browser.mjs`; before/after cases under
`target/ceremony-source-start`, including `old-core-review-failure.log`.

## PP-68: Text decoding changes an exact native artifact's bytes

**Pattern:** A shared UTF-8 reader silently consumes a BOM. The browser then
accepts JSON the native CLI rejects and can hash the normalized text while
describing it as the original source. Parser agreement after normalization does
not establish the file contract.

**Structural approach:** Preserve artifact bytes through decoding or refuse an
unsupported representation explicitly. Compare actual CLI behavior on the same
raw file, and separate byte-critical artifact readers from generic text readers.

**Status:** MITIGATED for ceremony sources. A dedicated UTF-8 reader preserves
the BOM; source-based and prepared-config ceremony requests refuse the same
native-invalid tree. Actual browser failure receipts retain the original BOM.
Original profile hashing already uses raw bytes. Other text consumers still need
their own byte/normalization contract review.

**Tests:** `test_ceremony_source_start.mjs`,
`test_ceremony_source_start_browser.mjs`; original reproduction at
`target/ceremony-source-start/bom-tree-probe/evidence-before.json`.

FLETCH follow-up: the cache-manifest reader now also preserves a BOM so the
actual native JSON parser refuses the same original file. Old browser acceptance
and native refusal are retained at `target/fletch-cli-contract/bom-before.json`;
`test_fletch_cli_contract.mjs` and `test_fletch_cli_browser.mjs` cover the scoped
reader and actual UI failure request. Other text consumers remain unreviewed.

## PP-69: Shared report values conceal native file-writer differences

**Pattern:** A browser download adds formatting bytes after a shared kernel has
produced the correct report. Value-level or shared-adapter tests pass while the
actual native CLI's artifact differs; a UI test may even enforce the wrong bytes.

**Structural approach:** Compare the actual CLI output file with the downloaded
artifact, including encoding, field order and trailing bytes. Make host-specific
representations explicit and narrow. Keep output production separate from a
later diagnostic gate's exit status.

**Status:** MITIGATED for FLETCH cache-index reports. The download preserves the
native writer's no-final-newline output and exact u64 digits. CLI gate failures
still produce the diagnostic report. Other report writers need their own review.

**Tests:** `test_fletch_cli_contract.mjs`, `test_fletch_cli_browser.mjs` and the
corrected cache-report assertion in `test_project.mjs`. The original byte mismatch
is retained at `target/fletch-cli-before-2.log`.

Native-format follow-up to PP-68: the selected RPLAN validator now also retains
BOM/CRLF bytes through `exactNativeJsonText`. Actual pre-fix UI acceptance versus
native refusal is retained at `target/native-format-cli/browser-hHCVts/progress.json`.
`test_native_format_cli.mjs` and `test_native_format_cli_browser.mjs` compare actual
CLI and final workbench input, strict outcomes and original-byte SHA. Generic
text consumers still require their own contract review.

## PP-70: Shared request assembly overwrites native-specific parsing

**Pattern:** An operation branch constructs the correct native artifact text,
but a later common request hook rebuilds that field with another decoder. A fix
at the branch alone does not reach the worker. Source hashes and parser outcomes
can silently describe normalized input instead of the selected file.

**Structural approach:** Assign one final owner to each byte-critical request
field. Compare the request actually posted to the real worker against the exact
native file; checking the early branch or shared kernel alone is insufficient.

**Status:** MITIGATED for native RPLAN format validation. The metadata branch
does not decode its text; final assembly uses the explicit native JSON reader
once. Other operation/shared-hook combinations still need their own review.

**Tests:** `test_native_format_cli_browser.mjs` asserts final requests from real
file inputs and actual native outcomes; `test_project.mjs` adds the final posted
BOM-byte regression. The earlier generic hook's normalization is retained in
`target/native-format-cli/before.log` and its original browser progress record.


Config follow-up to PP-68: native YAML now reaches inspection/validation and the
shared Run/TUI request builders without BOM consumption. The pinned native
parser refuses the tested named-prefix BOM but accepts a leading-comment BOM;
the accepted file's SHA must still include those original bytes. Actual CLI,
original-file browser requests and raw hashes cover both cases in
`test_config_cli_contract.mjs` and `test_config_cli_browser.mjs`. Original probes
are retained under `target/config-cli-probe`; other readers remain unreviewed.


## PP-71: Reentrant refusal executes another operation's cleanup

**Pattern:** A busy check inside an async try/finally returns or throws, but
the finally block clears the controller/busy state belonging to an already
running operation. A disabled button alone does not protect direct handler calls.

**Structural approach:** Reject reentrant calls before entering operation-owned
cleanup. Only the invocation which acquires busy state can release that state.

**Status:** MITIGATED for Config publication handlers. Early guards precede
try/finally. Broader handler ownership remains unreviewed.

**Tests:** `test_config_publication_browser.mjs` delays actual browser parent
creation, invokes both handlers again, and checks the original Cancel/busy state
remains active. Cancellation then reports the created parent without a file write.


Policy follow-up to PP-68: `policy-files.js` now selects the explicit native
UTF-8 reader for raw override bytes. The actual CLI rejects a BOM prefix before
its environment fallback; the old browser normalized it into a valid override
and changed its hash. `test_policy_cli_contract.mjs` and
`test_policy_cli_browser.mjs` check actual command/formatter bytes, original
worker inputs and inert source retention. Native fallback remains a required
missing workflow; this bounded byte fix does not establish fallback parity.


Policy fallback follow-up to PP-68/PP-71: native source choice operates on
original bytes, including zero-byte and invalid-UTF-8 sources, and never falls
through after valid JSON merely because lookup fails. Read callbacks carry
revision and ownership tokens; cancelled A cannot overwrite Open or release
active B. `test_policy_fallback_wasm.mjs` and
`test_policy_fallback_browser.mjs` retain actual CLI and controlled real-file UI
checks. The mitigation covers this explicit source-chain adapter, not ambient
host discovery or every file-input consumer.

Report PDF follow-up to PP-71: retiring a preview aborts its pending read, but
the old read's eventual `finally` previously cleared the shared file chooser
even while a newer preview was reading another PDF. Cleanup now checks that its
controller still owns the chooser. `test_report_pdf_browser.mjs` fails on the
prior handler with the newer pending selection missing, then passes both read
completion orders for all five genuine PDF cases after the fix. A separate
intermittent OS-backed file-read refusal remains unattributed; this deterministic
regression is not claimed to explain it. Broader file consumers remain unreviewed.

## PP-72: Host-sized metadata narrows a portable reader's accepted domain

**Pattern:** A native manifest uses `usize` for historical counts. Sharing that
typed reader with wasm32 rejects valid 64-bit metadata before certificate or
source checks, even when the reader never sizes an allocation from those counts.
A best-effort consumer can silently report the manifest as unavailable. This
occurs with original JSON bytes intact; it is separate from PP-59's JavaScript
numeric rounding.

**Structural approach:** Separate read-only count claims from execution sizes.
Use the shared generic schema with a `u64` metadata view, retaining the native
`usize` execution alias. Compare ILP report-count claims against the bounded
actual inventory without truncation. Review allocation/replay consumers
individually rather than widening every field globally.

**Status:** MITIGATED for Doctor, certificate verification, Verify prechecks,
assignment-context discovery and multi-label summaries. Other execution-coupled
readers and full workflow acceptance remain open.

**Tests:** `test_plan_manifest_width.mjs` and its workbench test cover Doctor.
`test_manifest_reader_width.mjs` exercises all 13 counts through `u64::MAX`,
valid certificate binding, nested manifest discovery, summary availability and
actual ILP count mismatches against native 64-bit tools and Verify dry runs.
`test_manifest_reader_width_browser.mjs` checks exact explicit worker requests,
downloads and inert source-preserving project roundtrips. The shared native
`readonly_wide_count_is_compared_without_truncation` test checks the count comparison.

---

## PP-73: Workflow instrumentation follows obsolete operation boundaries

**Pattern:** A workflow moves source preparation into a combined native
preparation/compiler entry point. A test still counts only the former standalone
operation, or compares the compiled command against a reviewed command that
intentionally omits filesystem intent. Correct execution then appears absent or
incorrect. Silently dropping these assertions would also hide duplicate execution
or source substitution.

**Structural approach:** Observe the current public operation boundary and retain
independent effects: native reply equality, partition count, exact selected
source bytes and command identities. When verified-source compilation restores
adjacency intent, assert the exact restored argument and `path_accessed: false`.
Keep original and reviewed command hashes separate. Preserve failing receipts
and prior test bytes rather than rewriting an old run as successful.

**Status:** MITIGATED for selected-adjacency and House/Senate source checks.
Other workflow instrumentation, the complete builder and final package still
require their own execution evidence.

**Tests:** `test_native_state_adjacency_workflow.mjs` observes one combined source
preparation, zero standalone preparation calls and one partition, with independent
native assignments and substitution refusals. `test_native_state_chamber_sources.mjs`
counts the combined operation. `test_native_state_chamber_sources_zip.py` verifies
the restored adjacency argument, both original/reviewed command identities,
no path access, exact source bytes and native plan/context exports.

### PP-31 follow-up: synthetic counters conceal real-state feasibility

A tuning report mixed synthetic parity counts with real-state project counts
while its balance-miss counter covered only the synthetic loop. It printed zero
although the retained CA VRA project missed its requested tolerance. Readback
of the 98 projects confirms one real balance miss. Five tuning reports now
record synthetic and real-state feasibility separately and sum their totals.
The original receipts stay unchanged; their zero is explicitly synthetic-only.
The retained readback, frozen project mapping and fresh R6 counters are separate
evidence. These changes mitigate this reporting scope and do not establish
all-state feasibility or close the general PP-31 class.


## PP-74: Copied worker modules outpace packaged runtime dependencies

**Pattern:** A packager copies a directory of browser modules while binary
runtime dependencies are installed through separate staging recipes. Newly
copied workers can reference absent solver/checker WASM, receipts or glue.
Successful compilation and source-server browser tests then fail to establish
that the distributable can execute those workflows. A literal-import asset scan
also misses some variable-driven URL paths and auxiliary HTML entry points.

**Structural approach:** Construct packages from one declared runtime dependency
inventory, with source/receipt correspondence checked before costly work and
final staged-byte identities checked before promotion. Exercise each declared
entry point and valid/invalid worker flows from the exact packaged routes. A
partially written directory must never substitute for a successful build receipt.

**Status:** MITIGATED for standard toolkit proof staging. The registered builder
now uses the existing identity-checked RoundingSat, VeriPB and native-proof
stagers; diagnostic 15 passes 1,255 literal module/worker/HTML references and
source stability. Dynamic dependency inventory, source-receipt enforcement and
final standard/C proof browser execution remain open. This is not full parity,
reproducible compilation, general proof capacity or a deployment claim.

**Tests/evidence:** `scripts/web/test_toolkit_assets.mjs` catches the observed
literal missing checker dependency. Existing three proof stagers check artifact
bytes and relevant provenance/license inputs. Focused package and stage logs are
preserved under `target/parity-full-build-20261030-r8/proof-package-correction`;
`toolkit-tail-15` preserves the independent final-section run. The review at
`signals/roles/check/wasm-proof-packaging-roles-check-2026-10-31.md` specifies the
remaining negative, dynamic dependency and packaged execution checks.


### PP-31 follow-up: character summary counters cover synthetic cases only

The fresh R8 character report counts 500 real prepared-state project/export
replays but its balance/disconnection counters are updated only in the synthetic
loop. Its 42 misses must therefore remain synthetic-only. A separately
predeclared, content-addressed 500-project/fifty-graph snapshot and independent
integer/rational population plus graph-component readback find zero real
10-percent-tolerance misses and zero disconnections in the selected prepared
graph, with all stored metrics matching. Synthetic character values are not
observed economic/housing data, and graph connectivity is not land-only or legal
contiguity. `target/parity-full-build-20261030-r8/character-readback` retains the
original summary and test alongside the audit. Counter-scope implementation
remains unchanged during captured validation; this evidence mitigates reporting
ambiguity without closing the general PP-31 class.


### PP-31 follow-up: proportional zero counter hides two real misses

The R8 proportional report's zero-miss counter covers only the synthetic loop;
its real-state diagnostics contain CA single proportional-section at 47.1194
percent maximum deviation and TX multi at 54.0460 percent, versus 10 percent
requested. Independent population/graph readback of all 120 frozen produced
projects confirms both failures, zero selected-graph disconnections and correct
stored feasibility flags. Fresh native/WASM replay accounts for all thirty
other sampled-search failures with exact error messages and same-instance
success recovery. The full fifty-state/three-method matrix is predeclared and
accounted exactly once under `target/parity-full-build-20261030-r8/proportional-readback`.
Historical zero counters remain unchanged. A reporting-only character/proportional
scope patch is prepared but unapplied. The shared algorithm outcomes and general
counter-scope failure class are not marked solved by this audit.

### PP-39 follow-up: platform metadata erasure conceals reply differences

Status: **OPEN** in maintained source; a validated isolated candidate provides
mitigation pending adoption. Recursive replacement of every
`target_pointer_width` value hides an invalid 17 and differences in unrelated
same-named data. Narrower overwrites can also conceal invalid native/WASM width
values. The captured source audit identifies 29 affected comparison callers.

The candidate shared comparator permits only exact operation/schema/path
metadata with numeric WASM 32 and measured native 64; typed composed results
also validate their envelopes. Every other value and both original replies
are preserved. One complete isolated run covers all 29 audited callers and
1,503 raw pairs; independent readback and 185 corruption controls pass. A stale
browser raw-host expectation is corrected by checking the entire identity
preview against native-derived children rather than deleting the preview.

Role review found that temporary corruption artifacts alone do not prevent
recurrence. Candidate v7 adds twelve retained reply fixtures, a portable
Node-only guard with 127 corruption checks, and early registration in all three
builders. Five deliberately broken comparators and an altered fixture fail;
the clean-CWD baseline passes. Paired invalid schemas test structural schema
checks independently of ordinary one-sided deep-equality rejection.

The patch also needs a separately pinned binary fixture. Disposable staging
refuses missing/altered payloads before writes and verifies all 35 proposed
files byte-for-byte. Its initial Git automatic line-ending conversion failure
is retained; `core.autocrlf=false` preserves the reviewed helper bytes. All real
source files remain unchanged. Source-pin reconciliation, maintained adoption,
amended builder/default/C execution and original comparison contracts remain
open. See `signals/roles/check/wasm-command-comparison-roles-check-2026-10-31.md`
and `target/parity-full-build-20261031-r9/comparison-integration-candidate-v7/`.
This class is not SOLVED by a candidate or by native/WASM equality alone.

### PP-25 follow-up: unfinished comparison definitions can close execution rows

Status: **OPEN** in maintained source; an isolated gate correction is validated.
The existing closure validator compares the bundle's comparison field to the
contract but does not establish that either is finalized. Four retained
synthetic fixtures pass its full gate with a matching pending definition,
missing definition or unversioned text. These are structural counterexamples,
not real engine/workflow acceptance, and no real parent is promoted.

The candidate requires an explicit positive integer comparison version, a
nonempty non-placeholder mode and an explicit list allowlist before execution closure.
Two new tests in the existing closure-test entry point reject 22 invalid forms
in both ordinary/full verified modes and preserve three explicitly unresolved
ordinary forms. All ten unit tests pass; all four original counterexamples now
refuse. The actual scanner/ledger CLI still validates 309 unresolved records
on the isolated source tree and correctly refuses full parity at unverified Run.

Hashes and matching metadata cannot establish comparison semantics or oracle
coverage. The twelve unfinished command comparison contracts still need their
native/browser requirements finalized; unversioned historical metadata must be
versioned before it can support a verified claim. Maintained adoption and the
broader PP-25 class remain open. Source and raw counterexamples are retained at
`target/parity-full-build-20261031-r9/unfinished-comparison-probe/` and
`target/parity-full-build-20261031-r9/closure-comparison-candidate/`.


## PP-75: Native execution logs substitute for accepted output artifacts

Status: **OPEN** in maintained source; isolated regression candidate v2 provides validated mitigation pending adoption.

A log event such as `loading adjacency` proves entry into a code path, not completion at the expected output destination. A test can observe the event and successful exit while accepting missing expected assignment/plan artifacts. A retained controlled test diverts only the fresh State destination into another owned directory: the first-generation gate accepts it, while strengthened v2 refuses it. The actual native CLI is unchanged; this is a test-sensitivity witness, not a production engine failure.

Structural candidate: require fresh assignment writes at declared paths, exact integer unit coverage, independent fixture population/connectivity checks, and State RPLAN/RCTX alignment. Preserve complete before/after snapshots and original commands. Do not replace artifact checks with log counts, equal receipts or a success status. Snapshot modification times are retained as exact decimal strings; raw native status bytes are copied rather than rounded through JavaScript numbers.

Three complete strengthened gate runs pass using freshly generated fixtures/baselines, including unrelated CWD and an isolated source composition. Three deliberately broken completion/force/observation controls refuse, and the wrong-destination witness now fails. Evidence: `target/parity-full-build-20261031-r9/native-cli-workspace-regression-candidate-v2/artifact-controls/evidence.json`, `independent-execution-audit.json`, and `adoption-package/manifest.json`. Maintained adoption, full amended builders and original profile/workflow acceptance remain open. This class is not SOLVED by isolated evidence or decision equivalence.


## PP-76: A counterfactual changes syntax without changing the parsed value

Status: **OPEN** in maintained source; an isolated unused-field regression candidate provides validated mitigation pending adoption.

An explicit `--dpi 150` versus native default 150 changes the argument vector but does not change the parsed value. Equal results then provide no informative intervention on DPI. Likewise, retaining print-only in metadata while dropping it from the executable handoff can perform a real partition under a claimed no-execution request.

Structural candidate: retain both exact original vectors and full parser reviews; require the changed field to have command-line source and a different parsed value, compare the effective native configuration, and separately verify browser-only intent before source preparation/partition. Preserve native print-only's actual generation as a separate native limitation. Keep original artifacts and independently read the exported assignments/projects rather than relying on inspection metadata alone.

The fresh-fixture gate covers all 22 source-declared unused fields and refuses two controls: unchanged parsed DPI, and a dropped print-only handoff that actually invokes the WASM partition. The accepted cases retain zero source/partition calls under print-only. Evidence: `target/parity-full-build-20261031-r9/native-unused-regression-candidate/fault-controls/evidence.json`, `independent-audit.json`, and `adoption-package/manifest.json`. The 44-file candidate has builder/source registration and byte-exact disposable staging, but maintained adoption and required full-profile/browser/contract acceptance remain open. This is not SOLVED by source review or a passing temporary campaign.


## PP-77: A transient project-status snapshot stands in for Open completion

Status: **MITIGATED** for the maintained JSON Open observation paths; complete registered standard/C/profile testing remains pending.

Workbench Open legitimately sets `Opened filename` and then autosave replaces that status. A later polling predicate can therefore time out after a successful inert import. The R9 timeout did not capture project-status text; a separate controlled reproduction demonstrates this race without retrospectively inventing the original text or changing the failed build's status.

Structural scoped fix: install a MutationObserver before selecting the file, ignore existing text, and latch only the exact new Open completion. Disconnect the watcher after success/finally. Keep zero worker-execution and all imported-result/source assertions. A real-browser regression waits for the subsequent autosave, proves the old current-text predicate fails, rejects a stale same-filename success followed by invalid JSON, and confirms source preservation and inert Open. The full label pipeline still passes seventeen native comparisons; independent ZIP inspection checks 71 files.

Maintained sources: `scripts/web/workbench_project_open.mjs`, its browser regression and 91 callers. The expanded AST audit classifies 163 workbench waits: 134 JSON Open observations migrated; 29 archive/source/transfer routes retain their distinct autosave-disabled contracts. This supersedes the preliminary 170-file/258-occurrence text candidate list. All 2,317 original assertion calls survive byte-identically. The helper retains original filename/readiness predicates, the actual friendly chooser route and a shared timeout budget. Eight representative full browser callers and the maintained observer/ledger/comparison checks pass after guarded 95-file adoption. Evidence is under `target/workbench-open-audit-20261101/`; earlier label-pipeline evidence remains preserved. The abandoned manual folder harness received source-only migration and was not executed. Complete registered standard/C/profile and full acceptance obligations remain open; this class is not marked SOLVED.

### Maintained adoption follow-up for PP-39, PP-25, PP-75 and PP-76

The preceding isolated-candidate descriptions record their historical stages. Their reviewed code is now maintained through the same guarded 47-file adoption. Ordinary ledger validation, twelve policy tests, the 127-case corruption guard, fresh CLI lifecycle/ignored-field gates and inert Open browser regression pass on actual worktree sources. Original source and raw evidence remain preserved. The known test paths are now mitigated; broad normalization/semantic-contract/accepted-artifact/backend/browser/profile and full acceptance obligations remain open. No whole pitfall class is marked SOLVED and no full parent is promoted by source adoption.

## PP-78: Artifact equality hides ordered completion-state divergence

Status: **MITIGATED** for the maintained selected aggregation paths; complete amended profile/workflow verification remains pending.

Native Aggregate checks for an existing national JSON at each requested analyzer. With a repeated analyzer and no Force, the first iteration writes it and the second is skipped. The WASM adapter initially checked only the imported output observation, so it returned the same correct files but reported both occurrences as generated. Comparing file sets/bytes alone missed the workflow divergence. Deduplicating types would also be wrong because native Force regenerates both occurrences.

Structural candidate: treat generated JSON files as completion state for later iterations when Force is false; keep explicit analyzer order and Force behavior. Extend the existing actual-CLI completion gate with paired duplicate/Force assertions. The extended gate fails on the original real WASM engine. Candidate full-engine/native compilation, differential checks and actual browser import/run/download/reopen cases are queued separately. Maintain the nine existing fixtures and preserve R10's captured sources until all queues are terminal.

Evidence: `target/aggregate-workflow-audit-20261101T070808562164Z/` retains sixteen actual CLI trees/transcripts, thirteen exact artifact matches, the summary mismatch and three explicit native/browser boundaries. Independent readback verifies source preservation, sorting and exact u64 values. The original per-case `whole_workflow_equal` field overstated its artifact/skip predicate; the original report/tool remain retained with a digest-bound scope correction, and future tooling uses a narrower field name. Neither this audit nor a candidate fix establishes full workspace installation, host/capacity/profile or acceptance-v1 closure.

Candidate executable follow-up: full WASM compilation and seventeen actual CLI artifact/ordered-event comparisons pass, including four fresh interleaved duplicate/Force cases with independent readback. The original-engine red control still fails as intended. Native-host/browser/durable regression checks and maintained adoption remain pending; keep this pitfall OPEN until its maintained structural solution and appropriate gates are verified.

Later candidate results: native-host compilation, thirteen whole host/WASM comparisons and the eleven-case CLI gate pass. The initial browser queue fails on the separate PP-79 UI issue. The joint UI/kernel candidate and a self-contained actual-CLI browser regression pass both Force modes, with independent eight-file ZIP/source/request/project readback. The ten-file integration is staged, byte-exact and source/policy validated; maintained adoption and complete amended profile acceptance remain pending.

## PP-79: Rebuilding a retained recipe from a set-valued UI silently changes execution

Status: **MITIGATED** for the maintained selected aggregation UI; full amended profile/workflow verification remains pending.

Aggregate permits ordered repeated analyzer types in retained projects. The workbench reconstructed those values from a multiple-select immediately before Run, losing repetition and imported order without a user edit. A kernel could be correct while executing a different recipe. The first actual browser test after the kernel fix refuses this: the posted request contains one summary type where the imported recipe contained two. Preserve that failed queue and served-source identities.

Structural scoped candidate: retain the original type sequence while the selected type set is unchanged; only an explicit type-selection edit replaces it. Other controls and Run keep the imported sequence. Show the retained order and edit behavior visibly. Real-browser tests cover an interleaved `summary, proportionality, summary` recipe, both Force modes, a format edit, an explicit type edit, source preservation, retired exports, exact native output bytes and inert reopen. The new self-contained test generates its own raw fixtures and native CLI baselines and is staged in all three builders.

Evidence: `target/native-aggregate-order-PUjSqI/`, its `independent-audit.json`, and the ten-file `integration-candidate-v2/` under the aggregation campaign. Current-only source reconciliation and policy checks do not promote parent evidence or waive workspace/profile/host/capacity contracts. This scoped fix makes no claim that every other form preserves arbitrary imported recipes; broader review remains required.

Maintained follow-up for PP-78/PP-79: the ten-file integration is adopted byte-exactly after controlled R10 termination and all dependent/candidate handles are terminal. Actual maintained source/policy and selected real-browser checks pass with the separately compiled matching candidate engine; original failures and before/after bytes remain retained. R11 performs the fresh complete standard/C/browser/package/proof run. The selected paths are mitigated; whole-class/profile/host/capacity/acceptance-v1 closure is not claimed.

## PP-80: Updating some source catalogs leaves another stale until a late gate

Status: **MITIGATED** for the four named canonical catalogs; complete amended execution and broader evidence acceptance remain pending.

The Aggregate UI change updated current capability/option/practitioner references but missed the Fetch source-authority catalog. The ordinary ledger passed, then R11's operation-specific Fetch check rejected the stale workbench hash after compilation. Its final source capture also passed: unchanged bytes do not prove that their internal references are reconciled. Preserve the failed build and dependent guards.

Structural fix: the ordinary ledger now also validates practitioner/Fetch catalog schemas, mandatory presence, nonempty distinct source rosters, safe paths, exact hashes and integer byte lengths, and declared mapping references/groups. It runs before any initializer can update inventory rows and never refreshes authority itself. Removing a stale pin while retaining its mapping is rejected. Four new policy methods cover both catalogs; a one-byte source makes the boolean-length control discriminatory. The original twelve closure tests remain, and full parity still refuses unverified Run.

Evidence: `target/parity-full-build-20261101-r11/source-catalog-fix/`. Retained old ordinary success demonstrates the gap. Final sixteen-test/source/practitioner checks pass; the unchanged actual Fetch gate separately passes 42 native planner and 42 handoff cases without acquisition. Only the current Fetch workbench hash/length is repinned; historical evidence, backend/scope states and acquisition claims are unchanged. These four registered catalogs do not stand for every legacy JSON file or runtime acceptance obligation.

## PP-81: Text decoding and empty-file filtering change native report inputs

Status: **OPEN** in maintained Aggregate sources; isolated correction and discriminatory execution pass.

Default UTF-8 decoding strips a BOM. Native Aggregate instead receives that BOM and skips the malformed report with its warning. The old browser turns it into a valid report and changes the state count. Rejecting a zero-byte source also prevents native's ordinary skip-and-continue behavior. `target/aggregate-application-planner-20261101/byte-edge-baseline-6tHM3R/` retains actual CLI and original adapter witnesses. A bounded aggregate-only staged correction preserves the BOM (`ignoreBOM:true`) and empty files, while fatal decoding remains. Generic package validation is unchanged.

`byte-edge-fixed-JqquSX/` verifies original raw bytes, warning payloads, native outputs and portable/project roundtrips. The actual workbench/worker regression passes at `target/native-aggregate-order-S9M2cN/`. Five live-reader/native cases and four actual OPFS reader cases also retain raw BOM/empty bytes. No broad claim about all text adapters, invalid UTF-8, native OS paths or failed-write behavior follows. The active R12 capture is unchanged; guarded integration and amended gates remain required.

## PP-82: Forwarding arbitrary options gives a read-only API write effects

Status: **OPEN** for adoption; corrected and tested in the unadopted selected-workspace prototype.

Spreading caller options after fixed lookup settings let `create:true` reach directory creation. The old reader created the version/year/states/national directories before refusing an empty state roster. Actual Node-backed and OPFS counterfactuals retain those effects at `read-option-baseline-H4TPsg/` and `workspace-browser-9Jc7AT/`. The prototype rejects unknown options, projects only cancellation/revision callbacks, fixes `create:false` last and fixes its own byte limit last. Shared lookup validates create/kind policy types. The corrected option controls leave the owned folders unchanged.

Keep read and write entry points separate; do not treat internal helper flexibility as public caller authority. Candidate source-read guards and positive metadata/zero-body size assertions pass. The reader is not yet exposed in the maintained UI. This correction is not a general guarantee for every option-spreading API or an acceptance-v1 closure.

## PP-83: Identical file bytes cannot establish whether a repeated write completed

Status: **OPEN** for maintained workflow adoption; the isolated recovery path refuses the ambiguity.

An interrupted Force write may have identical before/after bytes. Comparing the live file to a desired hash cannot prove that this occurrence completed. In particular, a saved intent followed by a process loss cannot be promoted to an acknowledged repeated operation merely because the file matches. The staged journal writes intent before the file and acknowledgement after close/readback. Fresh recovery distinguishes a persisted acknowledged prefix from an observed unacknowledged close; if both pre-effect and post-effect states match, it explicitly refuses ambiguity.

Actual Node-backed and real Chrome OPFS Force-interruption witnesses pass at `target/aggregate-application-planner-20261101/journal-5HQdvr/` and `journal-browser-vRdPur/`. Independent raw readback is `journal-reviewed-De0dF0/independent-audit.json`. This prevents cursor inference in the named candidate; it does not provide a UI/new-run workflow, exclusive filesystem lock, native temporary-file failure equivalence or full acceptance. Journals remain unauthenticated history and explicit recovery requires a newly selected root and private single-use review.

## PP-84: A failed journal write may leave an empty record without an acknowledgement

Status: **OPEN** for adoption; bounded empty-entry classification and explicit recovery pass in isolation.

Creating a journal file before a failed stream write can leave zero bytes. Treating its existence as an acknowledgement advances past unverified effects; discarding or overwriting it loses interruption evidence. The candidate retains the empty entry, hashes it, reports its physical slot separately from committed records, and does not advance semantic progress. A later confirmed recovery appends a new record with the last committed hash and a new physical sequence. Nonempty malformed JSON, missing records, invalid chains or unknown output bytes still refuse.

The initial implementation safely refused the empty acknowledgement but could not continue it. Its exact prior served modules and passing earlier-scope receipts are retained under `journal-before-empty-slot-source/` and `journal-browser-TVv6Gp/`. Final actual-file and real OPFS injected quota failures retain empty slot 4, recover the independently read-back closed file only after fresh review/confirmation, append without replacing that slot, and match native namespaces. Raw source/journal/native verification is `journal-reviewed-De0dF0/`. A failure creating the initial baseline causes no native output effect. None of this claims hardware durability, naturally exhausted disk coverage, authenticity, general damaged-journal repair or maintained closure.

## PP-85: Recovery can mutate its journal even when no native file remains to write

Status: **OPEN** for adoption; the isolated receipt correction has a discriminatory red/green test.

The initial durable writer set `journal_effects_may_have_started` only when creating a fresh journal. A recovery could append its final completion record while reporting this flag false, even if all native files were already acknowledged and there were zero new native writes. Native artifact effects and journal effects need separate truthful accounting.

`journal-receipt-klYAho/` retains the actual failing control against the exact earlier `journal-reviewed-Gz7r7G/` source: one journal write, zero native writes, false journal flag. The candidate now marks possible journal effects before recovery checkpoints. `journal-receipt-y8YdjO/` passes that same assertion, and amended actual-file/browser suites plus independent readback pass at `journal-reviewed-De0dF0/`. Original failures and source bytes remain retained. This scoped accounting correction does not establish correctness of every other receipt field or complete maintained integration.

## PP-86: Comparing transport records rejects equivalent native workspace selections

Status: **OPEN** for maintained adoption; the isolated semantic comparison and native-backed controls pass.

The reader and workbench represented the same input with different object-field order, so JSON.stringify equality refused a valid UI review before any write. A second counterfactual rejects portable inputs that omit the native default completion record. Advisory retained filenames, unused report bodies and order of an existing-output set likewise do not define native source effects. Conversely, analyzer order/repetition, effective source bytes/absence, state roster and observed output names do matter.

The staged gate now checks defaulted output observations as a set and compares hashes/lengths for actual active native source paths and their absence/roster across reads. It does not read reports skipped by native completion. Unused retained bytes remain in the project/journal and are disclosed; actual requests and ordered plans retain analyzer order. Seven fresh native-backed selections and four stale/deferred refusals pass at `target/aggregate-application-planner-20261101/review-selection-lwUkPM/`. Original failures are `workbench-ui-uTxYvq/` with exact served sources and `review-selection-dJCSM5/` with the preserved prior module. The full actual workbench case passes at `target/native-aggregate-workspace-browser-nnNg22/`, including 6-write no-Force and 9-write Force traces. This is semantic equivalence in the named gate, not general permission to normalize every input array or ignore request differences.

## PP-87: A small embedded asset registry no longer covers the eager import graph

Status: **OPEN**; source-derived discovery only, with structural packaging correction and executable verification pending.

The native web/catalog host embeds a fixed input-module list. Shared project validation now imports many additional modules, and staged workspace settings add more. Adding one new script does not repair an already incomplete transitive graph. The source audit at `target/aggregate-application-planner-20261101/native-asset-closure-baseline.json` follows eager imports from lab.js, static.js and the existing 18 registered inputs: 187 modules, 167 absent registry dependencies, about 1.49 MB of source. This includes inherited missing dependencies; it is not a claim that all 167 were introduced here or that a fresh native server execution has already failed.

The WASM assets builder separately copies the complete web/lab file set; that behavior does not prove the native embedded/catalog closure. Pending structural work is a checked deterministic registry/closure mechanism, query/fragment-aware dependency verification, correct minimum-shell budget tests and native executable/browser checks. Preserve the baseline, repair the whole closure and test it before maintaining or publishing the changed project module. No narrowed registry or broad native-host acceptance is inferred from the staged WASM UI tests.

PP-87 candidate update (2026-11-01): original native tests fail on native-state-vra.js. The staged complete-module registry, query-aware closure/exact byte-budget tests and seven-phase Cargo probe pass. Final native tests pass 11/11. Actual Chrome native/catalog hosts verify all 306 JS hashes and synthetic saved-map opening (`integration-replica/target/native-web-assets-browser-CTimeT/`). Independent readback: `integration-package-v2/independent-audit.json`. Status remains OPEN for maintained adoption/full builds.

## PP-88: Shared frontend accesses controls mounted by only one backend

Status: **OPEN** for maintained adoption; candidate native/catalog startup and saved-view tests pass.

After repairing assets, actual Chrome caught updateControls accessing COI/ILP controls mounted only by WASM. Trace `integration-replica/target/native-web-assets-browser-4FAKQC/` and prior `native-assets-original/web/lab/lab.js` remain. Candidate code gates updates and saved-history restoration on the browserWasm mount condition. Both native server and catalog then bootstrap and open synthetic saved maps, asserting absent WASM controls and no page errors. This does not establish engine-run or all-backend parity.

## PP-89: A native-valid version shares the journal metadata parent

Status: **OPEN** for maintained adoption; native-backed namespace and real UI history controls pass in v2.

Version `.bisect-aggregate-journals` is native-valid. V1 wrote correct outputs but treated year 2020 as a journal UUID; uppercase spelling also conflicts with a fixed lowercase lookup. Forbidding native-valid versions would narrow parity. The original native-backed counterexample is `integration-replica/target/native-aggregate-journal-version-YpUwX6/`; immutable v1 payload/audit remain.

V2 resolves only the fixed metadata parent's existing case variant; native paths retain exact-case checks. Bounded metadata listing filters unrelated names before the 512 recognized-ID limit and never reads their bodies. UUID files still refuse. Selected journals retain strict roster/size/hash/sequence checks; private observed parent spelling detects a subsequent case rename. Both actual applications match fresh native CLI outputs, while malformed selected records and UUID files refuse (`native-aggregate-selection-CBQIQq/`). Real UI history lists two journals beside a year folder/note with a body-read canary (`native-aggregate-workspace-browser-cmZL4B/`). This is coexistence, not a filesystem lock, authenticity or arbitrary path normalization.

## PP-90: Object-identity epochs do not coordinate aliased or cross-page writers

Status: **OPEN** for maintained adoption; same-origin participating Aggregate writer controls pass in candidate v3.

Two selected handles can identify the same directory without sharing JavaScript object identity. Per-handle WeakMap epochs alone cannot exclude an aliased or second-page application. The pre-fix writer admits another alias while an actual output stream is held; its bounded fail-fast test fails at `integration-replica/target/native-aggregate-workspace-browser-TmU2fW/`. Exact old modules, full source web tree, final test and failed source hashes are retained.

Candidate v3 wraps the entire confirmed application in a fixed same-origin exclusive Web Lock with ifAvailable, plus a same-context guard. It refuses rather than queues competing confirmations, releases on completion/error/cancel, and refuses browser effects without Web Locks. UI retains read/portable ZIP paths and explains the capability boundary. Real Chrome two-page/aliased OPFS tests hold an output stream, refuse both competitors with no journal/output effects, cancel the first, and succeed in the other page after a fresh review (`native-aggregate-workspace-browser-unjtRS/coordination.json`). Independent audit verifies final native/source bytes and three raw journal chains. The runtime gate asserts the held tree unchanged by competitors; that intermediate comparison is not separately persisted. This lock coordinates participating Aggregate writers only, within one origin. It does not lock out OS programs, different origins or other workspace operations; native failure-prefix, external changes and capacity/permission obligations remain open.

## PP-91: A collection limit checked after reading still requests an over-budget body

Status: **OPEN** for maintained adoption; exact/one-byte-over candidate controls pass.

The reader previously read each at-most-3-MiB File before checking the 8 MiB collection allowance. Application source checks counted only raw bytes rather than the validator's raw bytes plus state/analyzer labels. Existing-target review similarly checked its 50 MiB sum after each at-most-25-MiB read. These were bounded overshoots, not unlimited reading. Original actual-file counterexample: `target/aggregate-capacity-diagnostic-KbM8lA/`; exact pre-fix modules and the final discriminatory failing selection gate remain at `capacity-red-replica/` and `capacity-red-selection.log`.

V5 supplies min(per-file cap, remaining allowance) to workspaceReadFile, aligning source label accounting. Tests accept exact 8 MiB inputs and actual native-matching application, then refuse source total/file and stale-apply overflows before arrayBuffer or effects. Existing 25 MiB files totaling 50 MiB are accepted for read-only review; overflows refuse before the final body request. Final ten-case capacity evidence: `integration-replica/target/native-aggregate-selection-nfi1So/capacity.json`, with independent raw journal/native readback in integration-package-v5. The Node handle fixture reads files internally to emulate File; the canary proves no application arrayBuffer request, not no underlying OS read or natural quota pressure. Actual state/host capacity and maintained integration remain open.

## PP-92: Preparation can accept a file above the writer's read-back ceiling

Status: **OPEN** for maintained adoption; controlled pre-effect rejection has a red/green witness.

Preparation capped distinct output blobs at 50 MiB, but the writer's single-file read-back ceiling is 25 MiB. A reply between those limits could receive a live review, be written, and only then fail read-back. V5 adds the per-file bound during neutral preparation, before live authority or effects. `target/aggregate-output-capacity-glOzNR/` retains source hashes and old-accept/new-refuse discrimination for a controlled 25 MiB-plus-one reply; the final selection gate contains the guard. It uses a deliberately changed real-engine envelope, not a native large-state output. Actual native capacity and temporary/rename failure effects remain separate obligations.

## PP-93: Direct final-file writes ignore native scratch-file lifecycle effects

Status: **OPEN** for the practitioner installer; a private exact-effect prototype matches 23 native/OPFS trees.

Native Aggregate writes merged/record JSON to fixed *.tmp.json names and renames them. Existing scratch files are overwritten and disappear after success. A scratch directory causes failure before its associated final file; a final-directory rename collision leaves the newly written scratch bytes behind. The current staged direct-final-file writer ignores these paths, so it can report success where native fails or leave files native removes. `target/aggregate-native-effects-20261101/native-audit-6gA4zN/` preserves current-adapter counterexamples; integration-package-v5 carries a known-difference adoption marker without rewriting its payload/history.

The private native-aggregate-effects.js protocol uses actual write/rename operations and preserves exact failure prefixes. Node replay and actual Chrome OPFS move replay each match 23 native trees, including nine failures; independent byte/source/trace readback is `independent-readback-v1/`. No copy/delete rename approximation is used. This is not yet a reviewed/journaled practitioner workflow, selected OS-folder move support, natural quota/permission failure or power-loss atomicity. Preserve the original required native-effects scope while integrating it.

## PP-94: Native directory preparation precedes source discovery failure

Status: **OPEN** for reviewed workflow integration; actual native and private prototype failure-prefix trees agree.

run_aggregate creates outputs/version/year/national before checking whether states exists or is a directory. Preflight-only readers instead refuse without that directory effect. Empty states directories also complete successfully with a national directory and no reports. The actual native audit captures all three outcomes; the private effect prototype matches them in real Node and OPFS storage. The safe UI must explain and explicitly authorize the reviewed native effect prefix rather than silently create directories on Open or claim equivalence from an earlier no-effect refusal. Existing final-file-only recovery journals need an action-level contract before this prototype can replace the installer.

## PP-95: Final snapshot validation must not reread a grown file up to the general ceiling

Status: **OPEN** for maintained adoption; the private read-only planner has a discriminatory exact-size guard.

An initial snapshot may have captured a small source or scratch file, but final validation previously allowed a reread up to 3/25 MiB before discovering different bytes. That is still bounded, yet it unnecessarily materializes growth beyond the observed review snapshot. The private planner now passes the observed byte length as the final read allowance. Growth refuses from file metadata before body access; same-size changes still require a hash comparison.

`target/aggregate-native-effects-20261101/plan-exact-cap-red.log` fails the old version on its permitted reread size, with old source under red-plan/ and exact scripts retained. Final source/target drift guards pass and preserve only the deliberately injected external change (`effect-plan-0O133O/`); nine refusal cases and 23 positive/native-failure projections pass in Node, with 23 real OPFS read-only projections also passing (`browser-effects-UyxerC/`). Independent digests/action/raw-state verification is `independent-plan-readback-v2/`. These are File/adapter allowance guarantees, not atomic snapshot locking against external actors or durable write/recovery authority.

## PP-96: Window absence does not distinguish Node from a browser Worker

Status: **OPEN** for maintained adoption; private application has actual Worker red/green coverage.

The private action executor originally allowed its module-local Node lock fallback whenever window was absent. A real browser Worker also lacks window, so disabling its Web Locks let it create journals and native directories instead of refusing. The owned OPFS counterexample retains empty before/nonempty after trees at `target/aggregate-native-effects-20261101/browser-effect-coordination-wteJ4p/worker-result.json`; its exact pre-fix source, browser-served hash and failure log remain.

The executor now recognizes WorkerGlobalScope too and fails closed without browser coordination. Actual main/Worker tests preserve empty/unchanged roots and report no journal/native effects; two pages share the same real OPFS root, and a competing confirmed writer starts no effects while a temporary close is held (`browser-effect-coordination-38INrn/`). A fresh Force review succeeds after release. The fallback remains only for the tested non-browser execution context. These controls do not lock out other origins, OS programs or nonparticipating operations, and are not a recovery or public-UI acceptance claim.

PP-84/PP-90 native-action follow-up: final private action-journal faults retain empty acknowledgement slot 4 without advancing its committed intent cursor. Actual Chrome restart preserves completed/close-gap/empty-ack histories and restores no review authority (`browser-effect-persistence-faQ6Mg/`). Independent raw audit verifies 59 valid journal chains and the surrounding no-effect/fault/namespace boundaries. This is a new action-level persistence foundation, not implemented continuation/recovery or maintained closure of those pitfalls.

## PP-97: A hash-consistent journal can still describe non-native actions or blobs

Status: **OPEN** for maintained adoption; private recovery has real-engine recomputation and discriminatory self-rehashed forgery controls.

An attacker can change a saved plan and recalculate its plan hash, baseline hash and complete record chain. Integrity validation alone then succeeds; it cannot authenticate history or prove that the loaded Rust engine would generate those actions/bytes. A copied journal must never become a live grant merely because its checksums are consistent.

Private recovery recomputes native action semantics from retained original raw inputs using the original loaded WASM, regenerates native paths/actions/blobs and compares the full canonical plan, then checks a fresh live root against the recorded prefix. Historical overwritten-target identities remain unauthenticated before-images and do not claim original raw bytes. Self-rehashed unexpected action paths and altered blob bodies pass the syntax/integrity reader but fail recovery before effects (`effect-recovery-sqAze5/`, `browser-effect-recovery-P4IzFD/`). Ambiguous/unknown states still refuse; confirmation creates private single-use authority for the observed continuation only. Independent audit is `independent-application-readback-v4/`. Production UI/controller integration, original-runtime trust/distribution and full host/capacity acceptance remain separate requirements.

PP-83/PP-84/PP-85 action-recovery follow-up: identical before/after temporary-write or directory intents refuse rather than infer completion. New empty temp/CSV and partial planned directory states can continue only after fresh confirmation. Empty physical records remain untouched, and journal-only finalization truthfully marks possible journal effects while reporting no native file actions; it needs no move API. Actual Node and restarted Chrome OPFS tests cover those named scopes. They do not authenticate original syscall occurrences, repair arbitrary partial bytes or close maintained/full-workflow obligations.

## PP-98: A saved portable project is not a backup of browser-local action history

Status: **OPEN** for whole-workspace portability and maintained adoption; staged UI clearly separates portable projects and local workspace history.

Browser-managed native file moves can support the command lifecycle without relying on unsupported OS-folder move behavior. However, a local workspace includes scratch files and action journals beyond portable input/result fields. Saving the ordinary project does not preserve that complete tree. Clearing browser data can lose local history; importing a portable project must not recreate a selected handle, approval or recovery grant.

The private UI says what Save retains, keeps browser workspace IDs/handles/consent out of project files and clears selection/history/confirmed review on Open. Creation uses a fresh UUID and writes small metadata last; listing reads bounded metadata only, retains incomplete creation and does not read raw sources. Actual workbench browser-native-workbench-sFri7A proves inert Open, unchanged reviewed roots, no stored workspace IDs, metadata-only listing, genuine temporary-close interruption and newly confirmed recovery. Ten guards and no-storage/no-picker portable Run/ZIP also pass. Independent-workbench-readback-v1 verifies exact raw trees/chains/source hashes and unchanged prior journal bytes.

Remaining structural work is explicit bounded full-workspace export/import preserving original file/history bytes while treating imported data as untrusted, never overwriting existing workspaces and never granting write authority. Browser-local normal retention, portable project Save/Open and a complete action-history backup are separate capabilities. Current real-handle OPFS evidence does not establish selected OS-folder move support, natural quota/crash behavior or full multi-browser acceptance.

PP-98 private follow-up: full native workspace Backup/Import is now implemented in the staged UI. Strict stored ZIP validates all paths/blobs before fresh destination reservation, retains native raw/history/scratch/empty-directory bytes and regenerates only the browser selection marker. Actual workbench backup/import followed by separately confirmed recovery reaches the native CLI tree without touching the original interrupted root (browser-native-workbench-CcfrBh). Node format refusals, actual OPFS binary/BOM/empty round trip, unchanged-storage refusals and independent Python ZIP/chain/source readback support that scope. Maintained integration, broad host/capacity/natural-failure obligations and incomplete-copy inspection/export remain open; no imported provenance or permission is inferred.

## PP-99: Checking a browser workspace limit outside reservation can exceed the listing profile

Status: **OPEN** for maintained adoption; private registry has actual two-page final-slot coverage.

Listing refuses more than 200 known workspace directories. Without a matching creation bound, users can create a root that makes subsequent listing unusable. Checking count and then creating without shared coordination also lets two pages both observe slot 199 and create roots 200 and 201. Incomplete imports count too, because they consume preserved roots and must not be silently deleted.

Private creation/import now reserve under the same exclusive browser registry Web Lock, checking count and UUID collision inside the lock before the root is created. Missing/occupied coordination refuses. Actual Chrome browser-workspace-backup-cf0Zbn seeds 199 roots; two real pages race and exactly one succeeds. The final roster is 200, and both additional create/import attempts refuse with an unchanged roster. A real-close cancelled import remains visible as incomplete without a metadata marker, while valid roots still list normally before the ceiling test. Independent-workspace-backup-boundaries-v1 verifies raw evidence/source hashes. This protects participating same-origin reservations, not external/nonparticipating mutation or natural storage capacity.

## PP-100: A shared Cargo binary name can silently change a native comparison backend

Status: **OPEN** for complete amended build adoption; backend-matching actual CLI/WASM regressions pass and the maintained build recipe is corrected.

R12's C preview built bisect.exe with c-ffi-engine, replacing the earlier portable binary, then ran a declared default/core WASM regression against that shared path. Native legacy recursive bisection selects C at compile time, even when its parser retains metis-core. A four-unit State test produced the same grouping with opposite numeric district labels and correctly failed its exact assignment comparison. Parsed flags alone did not establish backend identity. The full attempt and both dependent queues remain failed.

The corrected recipe builds/copies/hashes a separately owned no-default-features core executable before building C, passes that path explicitly to both default/core regression suites and leaves the actual C reference for C comparisons. Retained actual core executable plus the same failing-attempt WASM passes all 22 exact unused-field cases and 18 workspace branch cases; no label normalization or assertion weakening was introduced. target/cffi-build-oracle-fix-20261101 retains old source, failure, artifact hashes and green receipts; r2 executes the corrected build/comparison fragment. Full amended profiles/catalogs are still required before this pitfall can be marked solved. Do not infer a native backend from filename, the last successful Cargo command elsewhere or a parsed-but-unused flag.

PP-100 focused follow-up: the exact corrected recipe now passes (target/cffi-build-oracle-fix-20261101-r2). Raw readback verifies separate owned core/C artifacts, builder identity, 52 native C assignment matches/four matching failures, 22 exact core/WASM assignment maps/three print-only adaptations and 18 workspace branches. The failed first scriptblock-path attempt remains retained. Complete amended builds and source/catalog reconciliation are still required for maintained closure.

## PP-101: Application metadata inside native data can consume a valid native version path

Status: **OPEN** for complete amended build/profile acceptance; maintained layout has structural separation and real CLI/browser coverage.

The draft browser workspace put .bisect-workspace.json inside the native root. That is also a valid native version component. Creating its version directory succeeded, then the app's label write failed because it collided with the directory. The actual browser failure and exact old sources remain at target/aggregate-native-integration-20261101-r2/red-selection-metadata-collision.log and before-external-selection-metadata. Native CLI acceptance of that same version is separately retained under native-selection-name-version.

Labels now live beside UUID roots in the app-owned parent container. Native operations and backup data therefore never reserve that filename within the native root. Completion labels are committed after file readback, and incomplete roots remain preserved without false completion. Backup v2 stores descriptive label metadata separately from native entries, retaining all native paths; draft v1 import migrates its explicitly defined old marker. Real Chrome passes the version-folder round trip, legacy migration, empty native-root backup, actual-close cancellation and two-page final slot. Current-source suite-nAwgUz and maintained suite-71W0gQ pass; independent-suite-audit-v2 verifies raw trees/ZIPs/chains/source identity. Complete amended profiles and broad failure/capacity evidence remain open; imported labels still provide neither authority nor provenance.

### PP-102: Salvaging an incomplete browser workspace must preserve uncertainty

**Status: OPEN** for maintained/full-profile acceptance; isolated structural candidate tested.

A missing/corrupt external completion label can leave useful native files. Hiding the root prevents inspection/export; inventing a normal label on export falsely suggests creation completed. A registry reservation lock alone also permits a snapshot to race the remaining copy writes.

Candidate structural solution: metadata-only inspection uses filename/kind/length capability without native body reads or mutation; explicit snapshot uses strict backup-v3/schema2 unconfirmed-creation descriptors, retained by fresh-root import/reexport with downgrade refusals. Source label/native bytes remain untouched. All participating copy writes hold the existing global Aggregate exclusive lock, while inspection/backup use shared coordination. Reports are observations, never completion, provenance or authority. Current version/year are supplied descriptive scope labels.

Evidence: target/aggregate-incomplete-workspaces-20261101/browser-hWDjuU (12 controls including close-held real copy exclusion), independent-readback.json, existing backup browser-workspace-backup-qkgQu7 and actual workbench browser-native-workbench-gdfTQg. Zero native body reads/writes during inspection; exact raw ZIP/import; explicit Open retirement. Same-size content drift is outside metadata inspection and no claim is made. Main sources remain frozen for live R13; adoption, maintained test hooks/catalog/docs and complete amended profiles remain outstanding. Do not mark SOLVED from private scoped evidence.

PP-102 follow-up: private maintained-path suite-z95JLC passes thirteen steps and independent-suite-audit verifies raw native trees/journals/ZIP/source closure. New metadata test catches backup path-depth eligibility and manifest-budget reporting; red failure and bounded fix are retained. A sixteen-file integration package applies byte-exactly in a disposable tree, with inverse catalog checks and passing source/policy/asset gates. It remains unapplied while R13 is live. PP-102 stays OPEN for maintained/full-profile acceptance.

### PP-103: Evidence readers must preserve producer schemas and final route identities

**Status: OPEN** for maintained/full-profile acceptance; current independent reader corrections tested.

A reader can falsely reject correct retained artifacts by treating a shared-string v2 archive as expanded v1 JSON, treating native binary-fixture index keys as browser original-ID keys, assuming only one native assignment layout, or listing copy actions as final served-source identities after a workbench index alias replaces the ordinary lab index. Conversely, silently dropping fields or erasing these distinctions can hide a real divergence.

Structural candidate: decode the exact declared archive schema and validate all shared references before comparing expanded data; compare all native assignment layouts and bind the complete ordered index-to-original-ID bijection without relabelling districts; keep raw byte hashes separately; model final source routes explicitly and verify all final bytes against their declared source. Preserve the failed readers/declarations and document any representation adaptation. Never call a format difference raw byte equality.

Evidence: target/parity-current-matrix-20261101 independent-partition-audit.json, independent-map-report-audit.json, independent-national-audit.json and independent-national-map-audit.json. Current native/core-WASM/actual Chrome all-state graph/queue/recovery/map/archive cases pass these independent checks. site-identity-v2 preserves the original shadowed-copy declaration and verifies the final workbench route; no runtime was altered to satisfy the audit. Maintained reader/test integration and complete required profile/source/host acceptance remain open.

### PP-104: A replay child and its result loader must agree on the complete version root

**Status: OPEN** for maintained/full-profile acceptance; isolated real-CLI fix and regression pass.

Native Verify passed an explicit base output-dir to State while later reading base/verify/year/plans/label. State interprets an explicit output-dir as the complete version root, so a completed replay was written to base/year/plans/label and Verify falsely reported a missing assignments file. Dry-run prechecks and a mocked child cannot detect this.

Structural candidate: pass Path(base)/verify as the real child output-dir, retaining original reference discovery and comparison. Regenerate a maintained synthetic binary graph, create the original with actual State and run actual Verify without --dry-run. Check successful exact replay, a mismatched original failing agreement after computation, and invalid certificate refusal before replay; verify actual versioned files, original source/plan file and directory rosters, and spaced Unicode roots. Preserve the raw failed baseline and new replay effects; no rollback is implied.

Evidence under target/native-verify-cli-replay-20261101 includes red observation/regression, separate pure Rust and C-enabled builds with three passing actual CLI controls each, independent raw readbacks, 58 passing Verify unit tests (one explicit real-L2 package ignore), three parsed builder hooks, source/policy gates and byte-exact ten-file disposable integration. Native parsed-unused seed/balance and the existing printed-command discrepancy are unchanged separate obligations. k=1 uses no C METIS call. Main adoption, complete amended builder/profile/package/proof and original broader workflow/host/parameter acceptance remain open.


### PP-105: Recorded desktop summary paths must not become file authority
Status: OPEN — scoped candidate passes; maintained/full integration pending.

Desktop ILP summaries may record absolute, relative or parent-relative output prefixes while browser selections use portable relative names. Rejecting every non-absolute prefix loses legitimate desktop packages; interpreting the recorded location as a path or URL risks granting access that the user never selected.

Structural candidate: keep the original raw summary bytes, try old portable comparison first, and allow only a full explicitly selected report roster with one shared opaque historical prefix. Only complete selected report/model names and independently audited bytes determine resolution. Require exact equality of every other typed summary field, preserve state on refusal, retain the existing summary byte budget and display recorded locations as escaped text without links. Active model references retain portable selected-map restrictions.

Evidence: target/ilp-native-summary-paths-20261101/final-run-v3.json identifies 22 whole native/WASM cases, ten altered-summary controls and two actual Chrome runs of seven audits each, with no historical URL requests. Nineteen shared unit tests, existing ILP regressions, independent source/metadata/hash readers, native exported-summary roundtrip and 14-file byte-exact disposable patch pass. Original real four-unit package L2 test explicitly passes; earlier ignored regular receipt remains unchanged. Packaging failures (incorrect assumed file count, trailing test whitespace) and predecessors remain preserved. Full amended builders/profiles/host/package/proof and original broader plan acceptance stay open.


### PP-106: Nonoverlapping patches can share source-pin dependencies
Status: OPEN — joint candidate passes; maintained/full integration pending.

Combining source patches by filename alone misses catalogs that pin a shared merged source. Fetch is supplied by only the ILP candidate, but its inventory-script pin changes when workspace and Verify test lists are combined. Treating that catalog as byte-unchanged incorrectly refuses a correct source union; retaining its older pin would create drift.

Structural candidate: preserve all original preimages, combine source changes against one captured baseline, merge exact builder insertions, union every explicitly declared source/test list, then recompute all affected catalog source pins from final bytes. Reject any other scalar/status/scope change. Reverse the recorded pin/append deltas and require full original JSON equality. Bind fresh joint source/staged checks and raw readers to a Git-native byte-exact disposable patch before adoption.

Evidence: target/wasm-parity-combined-20261101/combination.json, combined-evidence.json and integration-package/manifest.json. Twenty-nine changed files, seven raw overlaps plus the dependent Fetch catalog, 21 unchanged nonoverlapping payloads, four inverse-audited catalogs, three parsed builders, 309 records/69 practitioner rows/17 TUI groups/231 pins and sixteen policy tests pass. Source and staged suites each pass thirteen Aggregate steps; 22 ILP pairs, source/staged browser audits and actual Verify controls also pass. The initial nonoverlap assertion and missing declared Git dependency assumption are recorded separately. Maintained adoption, complete amended builds and original full host/profile/package obligations remain open.


### PP-107: Agreement with a generated endpoint list does not prove the source URL works
Status: OPEN — live defect reproduced; source correction prepared, engine/build integration pending.

Native source planning and browser endpoint generation shared incorrect built-in 2010 PL94 archive entries. Their agreement and offline fixture fetches passed while the real WA2010 URL returned 404. A generator reproducing a manifest is a consistency check, not independent provider validation.

Structural candidate: preserve the failed original source plan and HTTP body, verify official directory filenames and live bounded response prefixes independently, and change only the built-in 2010 URLs plus generated browser entries. Keep native custom-manifest URL behavior and other vintages untouched. Add durable offline expected URL evidence that fails the original data, then exercise actual corrected native/WASM planning, browser acquisition behavior and complete build/catalog integration. Provider probes must remain separate from full archive integrity and browser CORS acceptance.

Evidence: target/parity-vintage-resolution-20261101 retains the native/WASM WA2010 plan, original 404, explicit proposed source, original ZIP/CRC and fresh preparation. Exact native/WASM/filesystem graph preparation plus independent original block/tract source readback pass 1,458 tracts and 6,724,540 people. target/census-2010-source-correction-20261101 retains all 50 live official Range/ZIP-prefix receipts, the exact two-file correction and inverse all-other-byte equality. Full archive integrity is checked only for WA; corrected runtime/offline/browser/build/source/catalog and roles acceptance are not yet claimed. No maintained source adoption occurs while R13 remains live.


### PP-108: A current request allowlist must not erase retained historical source records
Status: OPEN — scoped compatibility passes; maintained/full integration pending.

Correcting built-in 2010 source URLs made strict project validation reject an otherwise valid original-format local-source project. Retained source/receipt labels are historical claims; treating them as current request authority either strands old bytes or grants obsolete requests.

Structural candidate: accept only fifty exact known former built-in labels in retained metadata, preserve the original strings/bytes and keep the active public/relay allowlists unchanged. Refuse unknown hosts/queries and wrong scalar types. Imported projects remain inert. Local input execution is explicit; fresh plan generation uses the current URI without silently rewriting the retained label. No record grants cache reuse, resume, installation or network authority.

Evidence: target/census-2010-source-correction-20261101/legacy-import-proof/evidence.json reproduces rejection on the untouched strict staged parser and exact import on the fixed parser. Node checks cover fifty historical records, active/relay refusal and unchanged acquisition code. Four source/staged actual Chrome runs each pass nine operations, including exact old-format input retention, native-equal local execution and explicit current-plan regeneration. Independent raw archive/CSV/request/source readers pass. An overwritten exploratory observation is disclosed separately; original fixture and strict parser bytes are retained, and no fabricated original record is used. The full 38-file source union has byte-exact disposable application proof. Full amended profiles and original broader lifecycle/host acceptance remain open.


### PP-109: A seed and tolerance are not a complete execution recipe

**Status:** OPEN

**Failure:** Native Verify printed recorded seed and tolerance while omitting them from its actual child arguments. Native State also cast a maximum unsigned seed into signed -1. A separate two-file experiment forwards the exact recorded values and preserves new unsigned values in engine_seed, but original multi-district replay still differs because a State run's explicit single search is absent from the general manifest and Verify falls back to multi-search. Passing one-district replay or parameter-field checks does not establish complete replay.

**Structural solution required:** Retain a versioned, source-bound original engine recipe and replay its actual algorithm/search/backend and relevant inputs, with explicit limits for historical incomplete evidence. Keep unsigned seeds exact, reject ambiguous legacy negatives, preserve precheck ordering and selected output authority, and never infer authenticity from local hashes.

**Evidence:** target/native-verify-parameters-20261102 preserves both red runs, a fresh pure-Rust candidate, three failed strict seed 0/17/max-u64 full-replay cases, direct single-search controls restoring exact original assignments, exact child parameter audit, the original three-case output-root regression, 58 unit tests plus a separately executed real ILP/RPLAN L2 test. New maintained-relative parameter probe is intentionally failing and unregistered. No whole-replay or source integration closure is claimed. Review: signals/roles/check/wasm-native-verify-parameters-roles-check-2026-11-02.md.

Current recipe-binding checkpoint — 2026-11-02 UTC: the isolated native recipe candidate now passes the original strict seed cases and hostile controls. The separate portable inspection extension passes 25 whole native-host/WASM recipe/context binding cases and four Chrome UI/Save/Open journeys, including altered search and legacy records. It visibly retains the maximum unsigned seed and recorded settings; Open remains inert. Selected original source/graph verification, exact-fraction recipe preparation/execution, readonly native64 metadata coverage, source integration and full amended release/host/matrix acceptance remain open. Evidence: `target/native-verify-parameters-20261102/portable-binding-phase/status.json`.

Selected source/graph checkpoint — 2026-11-02 UTC: the isolated `inspect-native-replay-sources` action now verifies all recorded existing source hashes against explicitly selected bytes and the shared native reader graph digest. All 28 whole native-host/WASM cases pass, including three synthetic self-consistent context claims that reach graph-specific refusal checks. Independent Python reads every original source and full RADJ/companion graph for twelve original native packages. Five actual Chrome journeys pass: four retained-source Save/Open cases and fresh local file selection checked against the current host. Existing adjacency inspection remains exact across previous WASM/current WASM/host in twelve cases. Exact seed bytes remain visible; no recorded path is opened or replay performed. Recorded absence, unrecorded implicit inputs, upstream authenticity, full readonly count-domain, exact-fraction preparation/execution and original complete release/host/capacity/matrix acceptance remain open. First project-envelope test failure and its object-wrapper fix are preserved. Evidence: `target/native-verify-parameters-20261102/portable-source-phase/status.json`.

Recorded replay checkpoint — 2026-11-02 UTC: the isolated candidate now has separate preparation and explicit Run operations through the existing State compositor, population retries and literal original-RPLAN comparison. An optional exact fractional tolerance preserves original f64 bits while percent remains display metadata; mismatched redundant values refuse. A causal two-unit population [2,4] boundary passes with exact 1/3 on attempt zero, while unchanged percent-only transport rounds one ULP lower and exhausts retries. Twenty-eight original source/graph replies remain exact across previous WASM/current WASM/current host. The original 41-case replay gate is NOT passing: all whole host/WASM replies agree except independently validated State width metadata, but three C-enabled/configured-core original runs reproduce different assignments. Inspection confirms native bisection_runner::use_c_metis always selects C when c-ffi-engine is compiled, irrespective of configured algo.metis.engine; all three original assignments equal explicit-C output and differ from pure Rust. Earlier C-enabled/core labels describe configured metadata, not proven Rust execution. Recipe metadata must record effective recursive dispatch; historical incomplete records must not be rewritten or treated as complete. Exact-fraction support does not close cross-backend replay, browser Run UI, auxiliary/multiscale/readonly/source integration or original release acceptance. Evidence: `target/native-verify-parameters-20261102/portable-replay-phase/status.json`.

Effective-backend replay checkpoint — 2026-11-02 UTC: current v2 metadata separately binds native recursive dispatch. The older manifest also contains an explicit native version label; R2 uses only its unambiguous native labels as visibly unbound legacy claims, rather than the narrower R1 blanket refusal. Missing/ambiguous claims and contradictions with bound v2 metadata refuse. Both current core/C profiles pass 58 cases, including original old/new records with unchanged settings/source bytes and literal assignments. Actual native controls pass 49 cases; twelve final Chrome journeys preserve raw inputs/seeds/settings through inert Open/Save/Open and show configured/effective backend, recorded search/split, exact tolerance and comparison. Independent Python reconstructs v1/v2/context digests, all original source/RADJ fields and 36 executed literal assignment/population/BFS results across 116 replies. Current eight codec tests, 58 Verify tests plus separately executed real L2, 28 old-source regressions and causal fraction boundary pass. Original failed 41-case receipts and narrower R1 artifacts remain unchanged. Full auxiliary/multiscale/readonly, maintained integration, handoff/lifecycle/capacity and original full release acceptance stay open. Evidence: `target/native-verify-parameters-20261102/effective-backend-phase/status.json`; review: `signals/roles/check/wasm-effective-native-backend-replay-roles-check-2026-11-02.md`.

### PP-110: Configured engine does not identify effective compiled dispatch

**Status:** OPEN — tested isolated correction; maintained integration and full acceptance pending.

**Failure:** A recorded Rust engine setting can accompany actual C execution because native recursive dispatch follows a compile-time feature. Treating the configured field as execution evidence produces different literal replay assignments. The original three C-enabled/configured-core failures and their exact explicit-C matches are preserved.

**Structural correction:** New v2 recipes bind effective recursive backend separately from configuration. Consumers resolve actual dispatch, refuse unavailable/incompatible kernels, preserve original seed/fraction/search and compare full literal assignments. Old records may use only an unambiguous explicit native version claim, labelled outside the v1 recipe hash; they are never silently rewritten into stronger evidence. Missing, ambiguous or contradictory evidence refuses. Stored backend text is not author authentication.

**Tests:** `target/native-verify-parameters-20261102/effective-backend-phase/` retains both 58-case core/C gates, 49 actual native controls, eight codec checks, twelve Chrome journeys, independent original-source/graph/assignment readback and the failed original baseline. Current role review has three full-delivery P1 items; all-method dispatch, auxiliary/multiscale/readonly/source integration and full current release acceptance remain open. These scoped passes do not mark the maintained project solved.

NWay extension — 2026-11-02: actual C-native NWay also uses the shared
compile-time dispatcher. An earlier exclusion and unit assertion incorrectly
inferred configured-engine behavior. The original declaration and failed literal
comparison remain retained. Corrected v2/v3 resolution uses the already recorded
effective backend; v1 still requires its visibly unbound native-version claim.
Two unchanged C-native maps differ under the old core reader, are refused by the
corrected core reader and match with explicitly loaded trusted C. Sixteen actual
native replay invocations match compatible binaries or refuse incompatible ones
before creating output. See `constraint-preflight-phase/status.json` and
`nway-original-assumption-correction.json`. This extends PP-110; it does not close
maintained integration, all-method dispatch or full acceptance.

### PP-111: Strict import policies can change native auxiliary replay

**Status:** OPEN - scoped private correction passes; maintained integration and full acceptance pending.

**Failure:** A stricter browser importer can reject native-valid sparse or duplicate TSV data, impute different shares, or turn the native COI warning/fallback into a hard failure. A supplied partisan flag may also be unused by the resolved weighting mode; a single-district route skips partisan loading but still attempts COI adjustment.

**Structural correction:** Bind original selected auxiliary hashes and configured source labels. Share the actual native TSV reader; retain raw bytes, padding, last-row wins, empty files and the native 0.5 default. Apply the shared COI adjustment after weighting and preserve the whole base map on native content/read failure. Reject malformed transport separately. Record defaults, warnings and absence claims in export provenance and visible UI. No project path grants filesystem authority.

**Tests:** `target/native-verify-parameters-20261102/auxiliary-replay-phase/` retains 54 pre-extraction native outcomes and 54 matching current outcomes; 96 cases per WASM profile; ten Chrome Save/Open journeys; thirteen hostile controls; Python source/assignment/population/BFS/alignment/provenance readback. Original inactive-mode controls and the failed harness assumption remain intact; separately declared active-mode controls prove the loading policy. Demographic/character, multiscale and full maintained release acceptance remain open.

### PP-112: Native replay source roots can drift across generations

**Status:** OPEN - scoped private correction and two-generation tests pass; maintained integration/full acceptance pending.

**Failure:** A replay can correctly load original implicit auxiliary data but publish its new invocation directory as the recipe's source root. The first plan matches, while its own replay then resolves another directory or refuses mismatched source identities. Eight original real root-drift failures are retained, separately from an earlier wrong-command harness failure.

**Structural correction:** Explicit recorded replay carries a dedicated auxiliary source root; normal State keeps cwd discovery. Capture, replay loading and required source-identity linkage use the original root. Portable consumers check selected raw bytes and discovery roles without opening project paths. Empty/missing/read-error character inputs retain native fallback, while absence and unknown history stay distinct.

**Tests:** `target/native-verify-parameters-20261102/character-replay-phase/` retains eight two-generation native controls across both profiles, exact original assignments/settings/bytes, 49 older native Verify controls, 88 character checks per WASM profile, fourteen Chrome Save/Open journeys, ten hostile controls and independent source/alignment/edge/population/BFS readback. The correction is not adopted, and demographic/composed/multiscale/readonly and full release obligations remain open.

### PP-113: Nested specifications can bypass outer configuration profiles

**Status:** OPEN - scoped private guard and native/host/WASM tests pass; maintained integration/full acceptance pending.

**Failure:** Checking only scalar weight mode or selected file presence misses an active partisan declaration carried by a nested WeightSpec. A single-district route may skip source loading, leaving no selected partisan source, while native Court profile still rejects the configured flag. The initial host accepted such a request although the actual native CLI rejected it; the red request and both outputs remain retained.

**Structural correction:** Validate the authoritative nested WeightSpec through the native configuration-profile guard before execution, independently of source presence or whether the weighting route consumes the file. Reject duplicate scalar controls and conflicting observation channels. Unknown history, recorded absence and an unused source remain distinct; do not manufacture data or bypass the profile to make replay succeed.

**Tests:** `target/native-verify-parameters-20261102/composed-replay-phase/` retains the actual native/profile red-green comparison, 23 core/C policy checks per profile, native required-file failures, four old unused-character native controls, 68 composition checks per profile, sixteen Chrome Save/Open journeys and independent original assignment/source/sparse-weight readback. The owned first WASM build was identity-verified and deliberately cancelled for the reproduced defect, not restarted after a timeout. The candidate is not adopted; the complete original plan remains open.


### PP-114: Fine output cannot inherit the seed graph or requested seed after retries

**Status:** OPEN (private native correction verified; WASM/integration/full acceptance remain open).

**Failure:** Multiscale BG assignments were validated against tract populations and
would inherit tract unit/manifest/context identity. Separately, a successful
balance retry published the accepted seed in the manifest but retained the
requested seed as the only recipe seed, causing the plan to reject its own
replay. Sampling two matching implementations can still return imbalanced or
disconnected results. Re-discovering a secondary graph during replay can select a
different cache despite retained source claims.

**Structural correction:** Keep computational and output graph channels separate.
Validate and export the actual fine universe; reject invalid completed results.
Bind original settings plus accepted retry outcome and exact sampler in a new
recipe version; preserve old recipe bytes. Bind the selected secondary source
path and binary/companion identities; unknown legacy sampling is not invented.
Carry these boundaries through the real worker/UI before adoption.

**Tests:** `target/native-verify-parameters-20261102/multiscale-replay-phase/`
retains original native failures, 24 corrected CLI outputs, 24 native replays with
discovery unavailable, independent source/population/BFS reads, eight source
mutation refusals, four retry cases, ten versioned recipe tests and 49 legacy
native controls. Sampler differentials and validity failures remain distinct.
The role review is `signals/roles/check/wasm-native-multiscale-handoff-roles-check-2026-11-02.md`.


### PP-115: Optional source receipts must preserve the absent-field contract

**Status:** OPEN (private correction and original reader comparison pass; maintained integration pending).

**Failure:** Adding `multiscale_fine_graph: null` to every source-inspection receipt
changed ordinary v1/v2 whole replies, even though their graph/math did not change.
Fresh host/WASM equality alone did not catch this shared regression. The original
previous/current/native comparison did: its unchanged first case exposed the
extra field.

**Structural correction:** Insert optional fine receipt data only for an actual
fine graph. Propagate digest failures instead of publishing a null digest. Keep
the old entire-reply comparison rather than stripping the new field or weakening
its assertions. New profile fields also preserve omission for old requests.

**Tests:** `multiscale-replay-phase/source-regression-before-omission-fix.log`
retains the real red. The corrected owned images pass all 28 original source
comparisons and prior composition/character/auxiliary/backend suites. New source-
bound multiscale passes 96 cases/profile, 27 policy controls and twelve browser
journeys. See `signals/roles/check/wasm-selected-native-multiscale-roles-check-2026-11-02.md`.


### PP-116: Readonly recipe metadata must not inherit executable pointer width

**Status:** OPEN (private u64/decimal correction and scoped proofs; maintained integration pending).

**Failure:** Native64 usize search/structure metadata can be valid while failing
WASM32 recipe decoding before any workload is requested. Reconstructing it through
JavaScript Number can also silently alter exact settings or typed recipe hashes.

**Structural correction:** Share a generic schema with native usize defaults and
u64 readonly aliases. Preserve original typed serialization/digests, native raw
search/split strings and decimal UI rows. Retain u64 district metadata through
preparation instead of an unchecked cast. Translate only the selected method's
consumed settings, leaving active work subject to checked resource validation;
never clamp, wrap or substitute a different declared search.

**Tests:** The owned pre-refactor native64 oracle supplies 435 metadata specimens;
336 reproduce previous WASM pointer-width refusals. Current core WASM preserves
all canonical strings/digests. Native/current readers refuse 116 malformed counts.
Sixteen actual native NWay references retain unused wide search choices and pass
32 preparation/replay checks. Four actual Chrome journeys preserve raw source
bytes and exact values through inert Open/Save/Open. Independent Python and the
original whole-reply source regression supplement the shared host/WASM checks.
Failures and harness preimages remain retained. Final profile evidence and scope:
`target/native-verify-parameters-20261102/readonly-recipe-phase/status.json`.


### PP-117: Raw source identity must survive incomplete geometry

**Status:** OPEN (private correction and native/WASM controls; maintained integration pending).

**Failure:** Native manifests hash an observed TIGER SHP even when malformed DBF
or sparse unit alignment prevents a GeometryContext. Context construction formerly
omitted that raw source hash, so Verify refused completed plans before replay.
Computational geometry also uses a single cwd-relative candidate while report
geometry uses three candidates; they must not be conflated or rediscovered from
a different replay cwd.

**Structural correction:** Bind raw report source identity independently of full
geometry validity, retain computational TIGER identities separately, and replay
from the recorded source root. Fine BG output continues to omit tract geometry.
Share original native ALAND/GEOID alignment and circular perimeter arithmetic;
keep missing/unreadable algorithm geometry and method-specific fallbacks visible.

**Tests:** Original 36 native outcomes plus six separately declared report-only
controls remain frozen. All 42 current outcomes match, including fourteen failures;
28 completed native plans replay from an empty cwd with exact assignments and
retained source identities. Source-bound WASM checks cover current/legacy records,
including old inconsistent/unknown evidence refusals. Retained pre-fix native
geometry-hash mismatch receipts and final profile/browser/independent evidence:
`target/native-verify-parameters-20261102/geometry-replay-phase/status.json`.

### PP-118: Aliased source roles must not contradict the same consumed path

**Status:** OPEN (private shared guard and native/WASM proofs; maintained integration pending).

**Failure:** A self-consistent synthetic recipe/context could record one consumed
algorithm TIGER SHP path as absent while a second geometry role recorded the same
path with selected existing bytes. The reader accepted the contradictory record
and chose the absent-source fallback. Hash consistency alone did not catch it.

**Structural correction:** For each consumed computational TIGER identity, reject
other records for the same normalized path when their presence/hash differs.
Check before TIGER decode or execution. This is consistency of supplied evidence,
not verification of historical absence, upstream authenticity or all source types.

**Tests:** `policy-host-xUPz5y` retains the real accepted contradiction. Corrected
host/core/C policy suites refuse it; four actual native policy invocations refuse
before creating output directories. Browser Save/Open retains the hostile bytes
and refuses explicit replay without executing the partitioner. Full phase receipts
and driver diagnostic-casing preimages remain preserved; see phase status.


### PP-119: Data availability does not replace native method input consumption

**Status:** OPEN (private execution correction and original native controls; maintained integration pending).

**Failure:** Native AreaSection loads computational TIGER areas only when Area
is present in its constraint list, and otherwise fails even if report/adjacency
areas exist. The old browser reader accepted a self-consistent synthetic recipe
with only Population and executed the partitioner with adjacency areas. The
native run failed and had no completed manifest. The executed-before-fix reply
is retained in `population-only-xrcol4ds`.

**Structural correction:** Check actual method preconditions before partitioning,
independently of available prepared arrays. Retain the exact recorded constraint
list. AreaSection uses Area presence and the original population vector; its
interleaved matrix is unused, so Area-only, reordered and duplicate-Area lists
preserve native behavior. Empty-list population fallback outside AreaSection is
retained; unsupported other non-population matrices remain explicit.

**Tests:** Twenty-eight actual native AreaSection outcomes (sixteen completions,
twelve failures), a separate eight-run empty-list extension, 72 worker checks per
profile, twelve browser Save/Open journeys and independent source/RADJ/literal
assignment/population/BFS/export readback. Synthetic negative metadata is never
represented as a completed native reference. All prior scoped suites remain green.
Evidence: `target/native-verify-parameters-20261102/constraint-preflight-phase/status.json`.


### PP-120: Inventory regeneration must preserve historical metadata

**Status:** SOLVED for command inventory history preservation (structural correction adopted; actual scanner regressions pass on main).

**Failure:** The command declaration scanner retained selected entry fields and
one root policy key, but dropped recorded_scope_notes and any other root history.
The ordinary inventory/ledger validators did not detect that loss. The independent
inverse audit did. The original catalog bytes and dropped-history candidate remain
retained; the final catalogs restore the exact old historical content.

**Structural correction:** Preserve the complete root metadata envelope and all
prior entry fields, overriding only current declaration identity and supplying
pending defaults for new entries. Reject unknown schema versions before writing.
Keep source reconciliation separate from historical acceptance. Current source
references are updated only in enumerated authority/consumer/action/test-source
slots; original evidence and all other mappings remain unchanged.

**Tests:** The maintained-source test_wasm_inventory.py runs the actual scanner
against owned declaration trees, checking root notes, future entry/root history,
current identity override and unknown-schema no-overwrite behavior. Two tests pass.
The independent four-catalog inverse audit passes and all 522 runtime compiler
inputs remain exact. Seventy command rows, 241 options, 152 direct-read traces,
seventeen TUI groups and 253 practitioner source pins validate without an execution
claim. Ledger/consumption regressions pass sixteen/four tests. See
`target/native-verify-parameters-20261102/catalog-reconciliation-phase/status.json`.

Maintained adoption readback — 2026-11-02 UTC: the corrected scanner and its two
actual-scanner tests are now on main; both tests pass there. The main ledger and
consumption regressions pass sixteen/four tests. Current source/test pin changes
are enumerated separately from historical evidence. Source adoption and raw
preimages: `target/native-verify-parameters-20261102/maintained-replay-gates-phase/main-adoption/`.
This solves the specific metadata-dropping generator defect, not the wider
WASM/desktop plan, full workflow acceptance or other open pipeline pitfalls.


### PP-121: Original source equality must not masquerade as reviewed evolution

**Status:** MITIGATED (explicit original/current transition guards and thirteen maintained mutation tests pass; amended full build remains required).

**Failure:** R14 stopped at the original State extraction fingerprint. Six type
bodies intentionally gained recipe serialization and/or a count-domain parameter,
but the old success claim still required all twenty-two bodies to be identical.
Blindly replacing original hashes would erase what the earlier evidence proved.

**Structural correction:** Preserve the original baseline bytes. Pin six current
bodies separately and require each to project to its original hash through one
named, narrow serde/count-domain rewrite. Keep all sixteen other bodies exact.
Reject unknown versions/projections, missing reviews and duplicate transitions.
Never treat source identity as full runtime parity or executable capacity.

**Tests:** Thirteen maintained mutation tests cover current bodies, original
baseline changes, unknown/missing/duplicate transitions, missing reviews, and
semantic changes with repinned current hashes that still fail original-body
projection. All pass; checks are registered in the three builders. Current
catalog slots are reconciled with an inverse preservation audit. R14 remains
terminal-failed; no package/proof queue from it ran. Evidence:
`target/replay-source-extraction-transition-20261102/`.

### PP-122: Final maps can consume the seed geography instead of fine units

**Status:** MITIGATED (bounded native/WASM/browser correction adopted; complete fine export/workflow acceptance remains open).

**Failure:** A completed native BG reference has thirty-two twelve-digit units.
The original legacy map CLI renders its sixteen parent tracts, reporting no
assignment match for every tract; browser tract mapping produces the identical
wrong-resolution PNG. The native geometry helper selects a BG filename but calls
the eleven-digit tract reader and refuses the real BG IDs. The browser raw TIGER
map path refuses BG sources entirely. Successful image creation is not a correct
fine-unit map.

**Required correction:** Resolve final geometry from explicit plan/unit metadata;
use the dedicated BG reader and accepted aliases. Keep tract seed-stage round
snapshots distinct from final fine assignments. Verify native, WASM, RPLAN/export
and browser handoffs against the same complete fine universe and original bytes.

**Tests:** Owned native CLI/worker red controls and original helper failure under
`target/fine-unit-map-workflow-20261102/`. Synthetic polygons are explicitly
labelled; they are not published Census geometry or algorithm provenance.

### PP-123: Label path composition can apply the version twice

**Status:** MITIGATED (corrected canonical CLI mapping covered by maintained actual-native tests; fresh full-build acceptance remains open).

**Failure:** Native Map calls PlanContext::from_label with an already versioned
output base, while PlanContext itself joins the version. A valid canonical
`output/v1/2020/plans/label` is looked up under `output/v1/v1/...` and fails before
geometry loading. Testing only legacy state folders misses the labeled workflow.

**Required correction:** Pass the unversioned output base exactly once and test
actual labeled native commands, accepted output locations and failure behavior.

**Tests:** The original canonical-label failure is retained separately from the
legacy folder map witness in `target/fine-unit-map-workflow-20261102/baseline.json`.


Current PP-122/123 verification — 2026-11-02: original failures remain retained.
Maintained native CLI gate covers 43 controls including 32 PNG references,
canonical labels, geometry aliases, missing/invalid metadata, legacy folders and
identical tract seed-round PNGs under tract/BG final manifests. WASM/browser
source preservation and explicit-run checks pass for owned candidate images.
Main source adoption and handler/security checks pass; fresh full-build and
complete fine export acceptance remain separate. Evidence:
`target/fine-unit-map-workflow-20261102/`.

### PP-124: Decimal-looking shared boundaries can overlap in exact binary coordinates

**Status:** MITIGATED (original refusal retained; independent exact audit and separate positive control; not general source repair).

**Failure:** Adjacent synthetic rectangles generated using decimal increments
look continuous but have eight positive overlaps under exact binary64 coordinates.
Native raw map rendering is intentionally topology-unchecked; a successful PNG
does not imply admission by the strict RPLAN geometry path.

**Structural boundary:** Preserve original coordinates and strict refusal. Audit
with exact fractions independently of the rendering kernel. Add a separately
declared exactly representable source for positive workflow testing; never move
original coordinates, reduce units, relax tolerance or replace the negative.

**Tests:** Original native and WASM strict overlap failures are mandatory.
Separate 32-unit geometry with a hole/island has no exact bbox-overlap pair and
matches corrected CLI, unchanged strict native and WASM PNGs. Maintained corpus
labels synthetic provenance and does not certify real source geometry. Evidence:
`target/fine-unit-map-workflow-20261102/source-topology-independent.json` and
`scripts/web/fixtures/fine-unit-map/`.


### PP-125: Replayed fine assignments can lose native context identity and edge classification

**Status:** MITIGATED (adopted-source native/runtime/browser gates pass; full builders and broader parity remain open).

**Failure:** Equal fine IDs, populations and weights did not imply native RCTX
parity: generic multiscale export changed source_id, unit_universe_hash and
Boundary edges to Custom. The original full equality failure remains retained.

**Structural correction:** For source-bound recorded fine replay, compare the
original selected context with the original plan and complete source fine graph
before execution. Preserve original context bytes/index/classification/hash;
use the accepted regenerated assignment and retain new replay metadata. Never
infer original context identity from a seed graph. Generic supplied exports keep
their separately disclosed conversion semantics. Retained auxiliary fields are
selected source claims, not regenerated or authenticated evidence.

**Tests:** Three native Rust tests include seven rehashed semantic mutations.
Both profiles pass 26 maintained export/alias cases and eleven browser checks;
independent Python checks every original context byte/field and original plan
body, all populations/connectivity and receipt/source retention. Evidence:
`target/fine-replay-export-workflow-20261102/`. No full-parity promotion.

### PP-126: An oracle adapter can create the malformed input it attributes to the product

**Status:** MITIGATED (direct native proof and corrected adapter adopted; fresh maintained gates pass, full acceptance remains open).

**Failure:** The old native map test adapter added GEOID to geometry that already
contained geoid, then a reader treating those as aliases refused duplicate fields.
The actual source contained only geoid. Attributing that refusal to the converter
would have prompted an unnecessary product change and weakened independent QA.

**Structural correction:** Preserve raw failing inputs and adapter binary. Verify
the unchanged input directly through the native reader. Normalize only matching
aliases, remove redundant ones, and refuse conflicting aliases before rendering.
Record adapter transformations and source hashes separately from kernel outcomes.

**Tests:** Direct native reader consumes the unchanged complete fine geometry and
matches WASM PNGs. Corrected adapter accepts lower-only/matching aliases and
refuses conflicts. Old failure, wrong-cwd build and corrected private build are
all retained under `target/fine-replay-export-workflow-20261102/`.


### PP-127: Counterfactual tests can confuse preserved provenance with behavioral changes

**Status:** MITIGATED (focused correction and negative controls pass; complete adopted-source builders remain required).

**Failure:** R15 stopped when an older test expected a maximum unsigned seed to
wrap to signed -1. The native manifest now preserves it in engine_seed with a
null legacy seed. Fixing that assertion exposed a second assumption: recording
each invocation's controlled working directory legitimately changes the recipe
digest and its bound context hash even for an unused CLI flag.

**Structural correction:** Require the exact unsigned seed and decimal recipe
seed; verify the actual typed recipe/context/certificate binding with native
verify --dry-run; independently recompute the fixture context hash. Compare
every remaining recipe, physical context, plan, provenance and effective option
after projecting only the already-verified declared cwd and its derived
comparison hashes. Original artifact bytes remain unchanged and retained.

**Tests:** The unchanged full 25-invocation / 22-variant counterfactual gate passes
with 22 exact WASM executions and three print-only adaptations. Ten controls
exercise accepted declared cwd variation and seed, cwd, stale/rehashed physical
context, literal label, option and native binding-chain failures. R15, seed-only,
helper-import and early certificate-refusal evidence is preserved under
target/max-u64-manifest-gate-20261102/. No full-build or parent parity promotion.


### PP-128: Ordinary recorded replay can lose the selected native context identity

**Status:** MITIGATED for selected original ordinary replay (main adoption and fresh host/core/C package/browser checks pass; complete amended acceptance and fresh-State conversion remain open).

**Failure:** Accepted native assignments, unit IDs, populations and physical weights
could match while ordinary replay exported a prepared-graph source_id/universe
and Custom edges instead of the original native identity and Boundary kinds.
Original direct-State and selected-replay gap witnesses remain retained; fresh
State conversion is still a separate open behavior, not silently claimed fixed.

**Structural correction:** Before replay, check the selected original RCTX against
the complete original plan index, selected native graph, ordered adjacency,
physical weights and native report counties. After execution, validate the
physical export, then retain exact original RCTX bytes/index/source hashes and
plan body, with new metadata, an original-context producer binding and explicit
auxiliary source-claim lineage. Native paths remain labels and handoffs remain
inert. No new geometry, source authenticity or execution authority is inferred.

**Tests:** Three new native Rust tests include ten independently rehashed valid
semantic mutations. Eleven maintained runtime cases/profile cover all ten actual
native State originals and a sixteen-unit actual C-FFI nway reference; core C
unavailability is refused. Eleven browser checks/profile cover exact downloads,
source/receipt retention, map/audit handoffs, stale hashing race and inert Open.
Full NC2,672/14 and CA9,129/52 native State/current-host/core/C-feature/browser
replay/archive/audit witnesses retain the original0.5%, requested42 and native
retry0/42 and4/46. Independent readback covers complete contexts/plan bodies,
all physical fields, labels/populations/BFS, archives/receipts and direct native
PNG equality. No original failure/workload, byte limit or computation budget was
changed. Evidence: target/ordinary-recorded-context-20261102/.


### PP-129: Standalone gates can accidentally require an earlier test's output parent

**Status:** MITIGATED (fresh-parent correction and all four original CLI cases pass; the complete amended full build still requires a new passing attempt).

**Failure:** R17 completed all500 character-project cases and later gates, then the standalone native case-only rename witness called root.mkdir() inside a missing target/label-rename-application directory. The same test worked in an older populated workspace. None of its four commands executed in that failed attempt; earlier passing gates did not make the full build pass.

**Structural correction:** Each standalone gate creates the parents of its own unique output root. The maintained test now uses mkdir(parents=True), retaining unique-root collision refusal and all original command/body assertions. No source label, force setting, stage byte, comparison or workload is changed.

**Tests:** The original script first reproduces the missing-parent failure in a fresh private tree. The corrected script then executes both case directions with and without force and checks exact registry and all stage bytes, absence of new files and native refusal text. The same four commands pass from main against the retained R17 native CLI. Original R17 logs and terminal packaging/proof refusals remain failures. Evidence: target/native-imported-replay-20261103/ and target/ordinary-recorded-context-adoption-20261103/.

### PP-130: Native Imported fallback must not acquire invented county identities

**Status:** OPEN (private host/core/C/browser correction passes; ABI/catalog/gate integration, broader original unit cases and main adoption remain pending).

**Failure:** Native State accepts a complete non-Census ID index and exports its Imported fallback. Recorded browser replay previously required the recipe's Census kind and refused all three actual accepted originals before execution. Reusing the separate custom-graph route would require complete explicit counties, although the native report may have complete, absent or mixed optional numeric county prefixes.

**Structural correction:** A separate explicit native Imported identity policy retains literal IDs and optional report prefixes while using the shared native weighting compositor. Generic custom graphs keep their existing complete explicit-county contract. Before replay, derive and compare the native unit index, validate every selected graph/context field and retain the original RCTX and plan body. Missing county identity stays absent. No county label, Census identity or source authority is invented.

**Tests:** Three unchanged-setting actual native four-unit originals cover complete, absent and mixed optional county prefixes. Three private Rust tests include five refusal mutations; both WASM profiles retain exact original context bytes and plan bodies, complete RADJ/companion fields, seed/search settings, district populations and BFS. Six full browser workflows each pass eleven checks, including source-preserving inert Open, explicit replay/audit/map, stale handoff refusal and exact native PNG readback. The existing eleven-case ordinary corpus still passes per profile, including its separate actual C-FFI reference. These fixtures execute metis-core; arbitrary native IDs, other resolutions, real-state Imported capacity and complete release acceptance remain separate. Evidence: target/native-imported-replay-20261103/.


Native unit identity integration checkpoint — 2026-11-03: the candidate now registers explicit native source identity and optional present-GEOID index fields, retaining complete/absent/sparse indices and native output-kind rules. Twelve actual accepted originals, five identity Rust tests, all24 browser workflows and independent RADJ/source/context/plan/COI/population/BFS/native PNG readback pass; two original CLI refusals remain retained. Both builders register the full corpus and portable failures. Generic Census validation remains strict unless the explicit native source policy is selected; no source or jurisdiction is authenticated. Main adoption and complete acceptance remain pending. Evidence: target/native-imported-integration-20261103/.

### PP-131: Regenerating declarations must not erase historical review metadata

**Status:** OPEN (candidate correction and two actual-generator regressions pass; main adoption and complete amended acceptance pending).

**Failure:** Regenerating engine declarations dropped three recorded_scope_notes root entries. Its entry whitelist also discarded unknown future historical fields, and an unknown schema could be silently replaced. A source update could therefore appear cleaner by erasing past evidence.

**Structural correction:** Update only declaration identity while preserving every non-declaration entry field and every uncomputed root field. Reject an unknown schema before writing. Current source hashes are reconciled separately and never rewrite original evidence or acceptance status. The exact three dropped notes are restored from their preimage.

**Tests:** The real generator first fails the retained history/schema controls. The corrected generator preserves root and entry extensions, original review/evidence/limitations/records through a changed declaration, and refuses schema999 without changing any original bytes. Both maintained builders execute these two tests before source-inventory validation. Canonical catalog reversal must reproduce every original object after undoing only current source pins, explicit candidate test references and the two new partial data-field rows.


Native identity main checkpoint — 2026-11-03 UTC: the reviewed implementation, typed fields, fixed corpus, history-preserving generator and maintained gates have been adopted with exact preimages. Fresh main host/core/C and both full twelve-case browser package gates pass. Earlier candidate/pending-adoption statements above describe the retained prior stage. PP-130/131 remain OPEN for their broader original acceptance conditions; neither the failed full build nor unverified parents is promoted. Evidence: target/native-unit-identity-adoption-20261103/.


### PP-132: Native build profile, configured backend and actual METIS use must remain distinct

**Status:** OPEN (corrected core/C WASM and maintained runtime/browser gates pass; reviewed local adoption is complete, while the full amended build and original acceptance remain pending).

**Failure:** Native recursive dispatch follows its compiled build, while prime-factor uses its configured backend. Treating the parsed flag as actual execution mislabels plans. A broad METIS classification also refuses BFS growth, centroidal Voronoi, capacity clustering and no-fallback ILP under a selected C profile even though those paths do not call METIS. Excluding all ILP would incorrectly hide its real METIS fallback requirement.

**Structural correction:** Bind an explicit native recursive profile separately from raw parser inspection, configured flags and actual effective/unused backend. Make fresh-profile usage method-aware, preserve legacy None-profile preparation/replay behavior, and require explicit C availability only when dispatch uses it. Profile edits retire current review and export grants. Save/Open stays inert.

**Tests:** `scripts/web/test_fresh_native_backend_profile.mjs` preserves all 48 original AL16 backend/fallback cases across two selected profiles per package, full native plans, source/export bindings, real native CLI controls and contrastive Core/C assignments. `test_fresh_native_backend_profile_browser.mjs` covers seven core and eleven C-package real Worker journeys, including pure-method no-sidecar success, natural ILP error, downloads, accepted seed, saved inputs and completed-profile edit invalidation. The original multiscale corpus remains unchanged apart from an explicit profile in its maintained loader adapter. Matching failures and unavailable-C outcomes are not valid plans; small synthetic cases do not close full-state or full parity acceptance. Original failed attempts remain retained under `target/backend-contract-review-20261104/`.
