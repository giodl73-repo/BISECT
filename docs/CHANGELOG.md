# Changelog

All notable changes to the Congressional Redistricting project.

**Last Updated**: February 8, 2026

## Related Documentation

- **[Enhancement Index](../context/enhancements/INDEX.md)** - Master index of all enhancements (detailed specifications and implementation plans)
- **[ARCHITECTURE.md](../context/ARCHITECTURE.md)** - System design and architectural decisions
- **[CODING_PATTERNS.md](../context/CODING_PATTERNS.md)** - Implementation patterns

**Note**: This changelog tracks historical changes chronologically. For detailed enhancement specifications and future roadmap, see `enhancements/INDEX.md`.

## [Unreleased]

### Added (2026-05-01) — Proportionality analyzer + PARTISAN_OPTIONS doc + E.4 partisan-similarity primitive
- **`redist analyze --types proportionality`**: new analyzer (`redist-analysis::proportionality`) computing the metric the project lacked — per-state vote-share-vs-seat-share gap. Distinct from the existing efficiency-gap / mean-median / partisan-bias metrics (which all measure *bias around 50%*); proportionality measures whether the parties won seats in proportion to their statewide votes. Output: `analysis/proportionality.json` with `dem_vote_share_statewide`, `dem_seat_share`, signed `proportionality_gap_pp` (positive = Dems over-represented), `n_uncontested`, `per_district_dem_share_sorted`. Wired into `--types all`. 10 unit tests covering perfect proportionality, MA-style geographic-lockout, TX-style minority-packing, and signed gap distinguishing over- vs under-representation.
- **`docs/legal/PARTISAN_OPTIONS.md`** (new): documents the four partisan-input postures the project supports — (1) partisan-blind baseline; (2) partisan-balanced (Plan 03); (3) partisan-similarity (E.4); (4) party-overlapping (E.5). Decision tree for which to use when. Explains how the proportionality metric is the cross-cutting comparison lens. Documents that the Federal Statute prohibits inputs (2)–(4) for federal congressional districts (§ 104(e)) but they remain available for state legislative districts and civic counter-proposals. Resolves the apparent conflict between the partisan-blind statute strategy and the E-series experimental papers.
- **`redist-core::build_partisan_similarity_weights`** (new primitive): E.4 partisan-similarity edge-weighting. Mechanically distinct from existing `build_partisan_weights`: boosts edges by *similarity* between adjacent tracts' dem_shares, regardless of strength. Two tracts at 0.50 and 0.51 are similar (boost); two at 0.70 and 0.30 are dissimilar (no boost). Exposes user-supplied alpha (1×–100×) and tau (similarity threshold) per E.4 plan §2.2. 10 unit tests including E.4 plan's worked examples (D+24/D+16 boosted, D+24/R+10 not), R-cluster vs D-cluster separation, swing-voter clustering, alpha/tau parameter sweeps. **Math primitive ships now; runner wiring (`--partition-mode partisan-similarity`) + 50-state ablation are next-session pickup** per `PARTISAN_OPTIONS.md` § 7.
- **20 new tests** total (10 proportionality + 10 partisan-similarity). Workspace test count: 1259 -> 1279, 0 regressions.

What this commit lands for the New England question: with `proportionality` shipped, a researcher can now compute (e.g.) `redist analyze --label ma_2020 --types proportionality` and quote the gap directly. The E.4 plan predicts Republican seats emerge under partisan-similarity α=10 in mixed-but-sorted states; we now have the metric to test that prediction once the runner is wired and ablations run.

### Added (2026-05-01) — `redist research validate-ensemble` CLI (RT Task 5 / M-02)
- **`Research` subcommand group** added (`Commands::Research`, `ResearchArgs`, `ResearchSubcommand`). Future research tools (`check-compat`, `mcmc`) will join here.
- **`redist research validate-ensemble --ensemble-dir <DIR> [--output <PATH>] [--rhat-threshold 1.05] [--ess-min 100] [--enacted <PATH>] [--strict]`**: consumes one JSONL file per chain (chain_id = filename stem), parses `{step, metrics: {name: value}}` per line, computes Gelman-Rubin R-hat across chains and pooled Effective Sample Size per metric using the already-shipped `redist_analysis::ensemble_diagnostics` math. Emits `validate-ensemble v1` JSON report with per-metric pass/fail against thresholds.
- **`--enacted <PATH>`** (optional): JSON `{metric: value}` file. When supplied, the report includes per-metric `enacted_percentile_rank` — fraction of pooled ensemble samples `<=` the enacted value. Direct enabler for the FAIRNESS_DOCTRINE state-court partisan-fairness litigation strategy ("the enacted plan ranks at the 0.2nd percentile of efficiency-gap; the ensemble suggests this is not a result the algorithm would have produced").
- **`--strict`**: returns non-zero exit if any metric fails its R-hat or ESS threshold. Default is to emit the report and exit 0 — the report itself surfaces failures via `passes_thresholds: false` per metric and `all_pass: false` at the top level.
- **Input validation**: refuses fewer than 4 chains (S-03 requirement), unequal chain lengths (Gelman-Rubin formulation requires equal n), missing-metric-in-some-chain (every step in every chain must record every metric), with `[INPUT]`-prefixed actionable errors.
- **8 new unit tests**: well-mixed chains pass, wildly unmixed chains trip R-hat, `--strict` propagates failures as non-zero exit, fewer-than-4-chain rejection, unequal-chain-length rejection, enacted percentile-rank position records correctly when enacted plan is at the bottom of the distribution, non-`*.jsonl` files in the dir are skipped, percentile-rank helper basic cases.
- **Workspace test count: 1251 -> 1259, 0 regressions.**

What this lands for the Federal Statute strategy: the model bill's § 105(c) verification right requires that "any person may execute the REFERENCE IMPLEMENTATION on the same inputs." `validate-ensemble` is the verification tool for *MCMC ensemble outputs*: a researcher runs an ensemble against a state's published map and uses the percentile-rank to demonstrate whether the state's map is a typical or atypical sample of the algorithm's distribution.

### Added (2026-05-01) — `redist civic add-candidate-race` end-to-end (CB Task 8)
- **`run_add_candidate_race`** is now wired (previously bailed with `[CONFIG]`). Calls `redist_analysis::race_of_candidate::parse_race_of_candidate_csv` against the input CSV, copies the original CSV + curator's master attestation document into `outputs/{version}/civic_inputs/{label}/`, and writes `annotations.json` (the parsed `Vec<CandidateAnnotation>`) + `provenance.json` (the `RaceOfCandidateProvenance` with per-curator and per-attestation-doc records, each carrying SHA-256 chain links per BD-R2) + `manifest.json` with the new `candidate-race v1` schema.
- **New `CandidateRaceManifest` struct**: `schema_version`, `label`, `year`, `state`, `submitter`, `submitted_at`, `ingested_at`, `n_annotations`, `n_curators`, `annotations_independently_verified`, `n_disputes`, original CSV + attestation SHA-256s, annotations + provenance SHA-256s, full provenance block.
- **Args extended**: `CivicAddCandidateRaceArgs` now has `--label` (defaults to `candidate_race_<year>_<state-lowercased>`), `--output-base`, `--version` flags so the subcommand parallels `civic ingest`'s storage layout.
- **5 new unit tests** cover: full output (every expected file + manifest schema fields + SHA-256 lengths), default-label derivation, `[INPUT]` rejection of missing CSV, `[INPUT]` rejection of missing attestation doc, and `annotations_independently_verified=false` propagation when any row has `independently_verified=false` (so `redist analyze --types bloc-voting` knows to inject the caveat downstream).
- **Workspace test count: 1246 -> 1251, 0 regressions.**

### Added (2026-05-01) — Federal Reproducible Congressional Districting Act drafts (post-Rucho strategy)
- **`docs/legal/MODEL_FEDERAL_STATUTE.md`** (new): bill text in U.S. Code drafting style. Defines tracts, mandates recursive bisection on the tract adjacency graph (§ 104), requires reproducibility artifacts with SHA-256 chain-of-custody (§ 105), preserves VRA § 2 as a deviation from baseline with published *Gingles* justification (§ 106), establishes a Director-of-Census + expert-panel parameter-table regime (§ 107), and creates a private right of action for citizens to verify byte-identity (§ 107(e)). 9 numbered sections plus drafting notes covering the Elections Clause defense, *Moore v. Harper* (2023) interaction, and *Allen v. Milligan* (2023) compatibility.
- **`docs/legal/STATUTE_RATIONALE.md`** (new): policy memo answering four questions — why Congress (Rucho's invitation), why reproducibility (every other "fair maps" standard imports discretion), why bisection (uniquely writeable + executable + constitutionally defensible), why now (the implementation matured + post-Callais clarifies disentanglement). Includes a "what this does NOT claim" section in the spirit of FAIRNESS_DOCTRINE §6.
- **`docs/legal/STATUTE_ONE_PAGER.md`** (new): 90-second version for Senate Judiciary / House Administration staff briefings. Tabular comparison of why prior approaches failed; constitutional posture summary; federal-vs-state contribution breakdown; implementation timeline.
- **`README.md`**: new section "The bigger picture: a federal statute analogous to Huntington-Hill" inserted before "Why bisection is fair." Names *Rucho v. Common Cause* (2019), explains the Rucho gap and the three forums Roberts invited (state courts, state commissions, federal legislation), points at the three legal artifacts, and frames the project's reference-implementation status as the load-bearing fact: § 107(a) of the model bill can designate this codebase directly. State-court strategy (FAIRNESS_DOCTRINE) for today; federal-statute strategy for 2030.
- **Architecture (one paragraph):** algorithm + parameters → federal (Director of Census + expert panel under § 107). Execution → state (preserves Elections Clause cleanliness + local VRA § 2 knowledge + politics: "the algorithm is just the rules" can pass; "Washington draws your districts" cannot). VRA § 2 deviations → state, with published *Gingles* justification, judicially reviewable on a deviation-from-baseline standard. Verification → any citizen, byte-identically, with a private right of action modeled on VRA § 2's existing 52 U.S.C. § 10302.
- **No code changes in this commit.** All four artifacts are documentation. The reference-implementation work that the model bill assumes — manifest schemas, byte-stable replication packages (`--paper-mode`), Callais-disentangled partisan mode, civic input pipeline, bloc-voting analysis — has shipped in prior commits.

### Added (2026-05-01) — `redist civic ingest --snapshot-urls` URL archival (CB Task 5 / PP-30)
- **`--snapshot-urls`** flag added to `redist civic ingest`. When set, every unique non-empty `source_url` in the COI CSV is fetched with the bounded-fetch policy (5 MB max body, 10 s timeout, ≤3 redirects, `User-Agent: redist-civic-snapshot/<version>`) and archived to `outputs/{version}/civic/{label}/snapshots/<sha8>.body` + `<sha8>.headers.txt`. Each archived URL gets a `UrlSnapshotRecord` in `manifest.url_snapshots` capturing http_status, content_type, content_length_bytes, truncation flag, body SHA-256, and the snapshot filename.
- **Failure-tolerant**: link rot is data, not a fatal error. Network failures, timeouts, and unreachable hosts add a `[WARN] URL snapshot failed for ...` entry to `validation_log.txt` and a stub `UrlSnapshotRecord` with `http_status=0`. Non-2xx responses (404, 5xx) are recorded with their actual status + body and a `[WARN]` is added.
- **Reproducibility constants** (load-bearing): `SNAPSHOT_MAX_BODY_BYTES=5MB`, `SNAPSHOT_TIMEOUT_SECS=10`, `SNAPSHOT_MAX_REDIRECTS=3`, all defined in `civic.rs`. Bumping any is a schema change.
- **Header canonicalization**: headers in `<sha8>.headers.txt` are sorted alphabetically by name; status line first; CRLF line endings (matches wire format). Body files contain the raw bytes truncated to the cap.
- **5 new unit tests** cover the end-to-end snapshot path against a localhost `TcpListener` test server (no external deps): success path with body+headers verification, oversized-body truncation, 404 status recording, unreachable host error, and `UrlSnapshotRecord` serde round-trip.
- **Workspace test count: 1241 -> 1246, 0 regressions.**

What remains deferred (CB Tasks 6/8/9/10/11):
- WARC format option (today only `headers-body`).
- robots.txt respect / explicit allowlist.
- candidate-race CLI integration with `redist-analysis::race_of_candidate`.
- Sheets template + HOWTO (CM-03).
- Hermetic LA fixture (B-04).

### Added (2026-05-01) — `redist compare --format html` self-contained side-by-side report (PCN Task 7)
- **`CompareFormat::Html`** variant added to `--format`. Produces `comparison.html` alongside `narrative.md` + `narrative_manifest.json` under the comparisons dir. The HTML is fully self-contained: HTML5 doctype, embedded CSS, no external script/stylesheet/image references — safe to email, print, or commit to a public repo.
- **`redist-report::html_comparison`** module (~470 lines, 16 unit tests): `render_comparison_html(report, cfg, ctx)` produces the report. Layout: civic-counter-proposal banner (when applicable, with submitter + submission date + "not the state's official map" disclaimer), header with DRAFT/APPROVED badge, side-by-side metrics table (Districts, Democratic-leaning seats, MM count, Mean Polsby-Popper, Total population with thousands separators), diff scope (tracts reassigned + districts touched), inline narrative as paragraphs, chain-of-custody footer with plan-A/plan-B manifest SHAs + template SHA + redist version + build commit short + reproducibility command. Print stylesheet keeps the civic banner/badge colored on paper.
- **HTML escape coverage**: all user-controlled fields (plan labels, submitter names) flow through `escape_html()`; one test injects `<script>alert(1)</script>` into a plan label and verifies it appears escaped, not raw.
- **Empty/NaN/zero handling**: empty district list renders as `(none)`; NaN mean Polsby-Popper renders as `(unavailable)`; the population row is omitted when both plans report zero.
- **L1 integration test** (`html_dispatch_writes_self_contained_html_with_civic_banner`): runs the production binary with `--format html` against a synthetic civic-counter-proposal fixture, asserts comparison.html + narrative.md + narrative_manifest.json all written, validates the HTML contains the civic banner, DRAFT badge, side-by-side rows, and provenance footer.
- **17 new tests** (16 module + 1 L1). Workspace test count: 1224 -> 1241, 0 regressions.

### Added (2026-04-30) — `redist analyze --paper-mode` AEA replication-package renderer (RT Task 8.1)
- **`--paper-mode`** flag added to `AnalyzeArgs`. When set, every `redist analyze` invocation emits a `paper_mode/` subdirectory under the analysis dir containing 9 files for AEA-compliant replication: `REPRODUCE.sh` (rendered from the Researcher Toolkit template with placeholder substitution), `inputs.sha256.json` + `expected_outputs.sha256.json` (deterministic file walks, BTreeMap canonical key order), `environment.json` (`redist_version` + `rustc_version` + `target_platform = linux-x86_64-glibc-2.35` + OS metadata), `seeds.json` (master seed or `null`), `CITATION.bib` + `CITATION.apa.txt` + `CITATION.chicago.txt` per `docs/file-formats/citation-strings.md` §3.3, and a `README.md` walkthrough that records the exact analyze invocation + cross-platform reviewer instructions (WSL/Docker `ubuntu:22.04`).
- **`--paper-mode-citation-style {bluebook|apa|chicago}`** (default `apa` per citation-strings.md §1) — advisory; all three styles are written, the named style is flagged "default" in the README.
- **`redist-cli::paper_mode`** module (~430 lines, 8 unit tests): `emit_replication_package(&PaperModeInputs)` is the public entry point; `PaperModeError` is a `thiserror`-derived enum with `[INPUT]`/`[INTERNAL]`-prefixed messages. The REPRODUCE.sh template is `include_str!`-embedded from `scripts/research/paper_mode_template/REPRODUCE.sh` so the binary is self-contained. SHA-256 helper walks up to 5 parent dirs to find `redist/Cargo.lock` (handles the `cargo test` cwd vs. workspace-root cwd ambiguity).
- **12 new tests** total (8 module + 4 args parse). Workspace test count: 1212 -> 1224, 0 regressions.

What's deferred (RT Tasks 8.4 + 8.5):
- The hermetic ubuntu:22.04 byte-identical acceptance test (Task 8.4) — needs CI to spin up a Docker container, run REPRODUCE.sh, and assert byte-identical `expected_outputs.sha256.json` matches.
- Conformance lint against the `social-science-data-editors/template-readme` rubric (Task 8.5) — needs a markdown linter rule set.
- Per-step seed derivation (today only `master_seed` is recorded; per-step seeds will land when the broader seed-derivation system ships).

### Added (2026-04-30) — Plan Comparison narrative dispatch wired end-to-end (PCN Task 11)
- **`redist compare --format narrative` / `--format both`** now writes `narrative.md` + `narrative_manifest.json` per invocation. The renderer (`redist-report::narrative`) and manifest writer (`redist-report::narrative_manifest`) were library-only since the prior commit; this lands the CLI dispatch that loads plan manifests + analysis JSONs from disk, computes their SHA-256s, and assembles the `ComparisonReport` -> `NarrativeManifest` pipeline.
- **`redist-report::comparison`**: `load_assignments` now reads `plan_dir/data/final_assignments.json` (production layout matching `PlanContext::assignments_path`) with a `plan_dir/final_assignments.json` fallback for legacy/flat fixtures.
- **`redist-cli/src/compare.rs`**: `[CONFIG]` bail replaced with real Narrative/Both dispatch. `SOURCE_DATE_EPOCH` propagates to `approved_at` for byte-stable manifests. `template_sha256` is the SHA-256 of the embedded `narrative.rs` source (the renderer IS the template). Civic-counter-proposal attribution flows from whichever side carries `submission_type == "civic_counter_proposal"` (plan B preferred). `--report-dir` overrides the default `outputs/{version}/comparisons/{plan_a}_vs_{plan_b}/`.
- **`redist-cli/tests/compare_narrative_l1.rs`** (4 L1 tests, all passing): full end-to-end shell-out to the production binary against synthetic plan fixtures, asserting (1) narrative + manifest written with DRAFT prefix when `--approved-by` unset, civic framing fires, ASCII-only per PP-34, frozen `approved_at` from `SOURCE_DATE_EPOCH`; (2) DRAFT prefix drops + `approved_by` recorded when supplied; (3) `--format both` prints the table to stdout AND writes the files; (4) `--enacted`-only invocations are rejected with no half-rendered output.
- **23 tests pass** (19 in-crate compare + 4 L1); 0 regressions.

### Added (2026-04-30) — Researcher Toolkit (partial; diagnostics + notebooks + paper-mode template) + Fairness Doctrine
- **`docs/legal/FAIRNESS_DOCTRINE.md`** (post-Rucho citable artifact): articulates the project's procedural-fairness claim in 5 verifiable properties, names case law (Rucho 2019, PA *LWV* 2018, NC *Harper* 2022, NY *Harkenrider* 2022, Allen v. Milligan 2023, Callais 2026), addresses Rucho's "no judicially manageable standard" rhetoric directly, and includes §6 — 9 things the project explicitly does NOT claim (the credibility-load-bearing antiparty-overclaim guardrail). §7 reproducibility recipe a special master can follow.
- **`redist-analysis::ensemble_diagnostics`** (Task 7 / S-03): pure-Rust math for Gelman-Rubin R-hat (≥4 chains required), Effective Sample Size (Geyer 1992 initial monotone sequence), Hamming-distance autocorrelation with integrated `tau_int`. `RhatRecord` / `EssRecord` / `HammingAutocorrRecord` serde shapes mirror the spec's JSON output filenames. 21 L0 tests against hand-computed truth.
- **`notebooks/`** (Task 1): 5 notebook stubs with cell-1 `runtime_budget_secs` metadata + cell-2 kernel-state attestation header against compatible RANGES (B-06) + final-cell completion sentinel. `notebooks/README.md` documents conventions. Notebook BODY content + CI workflow deferred.
- **`scripts/research/paper_mode_template/`** (Task 8 / D-05): AEA-compliant `REPRODUCE.sh` template with platform-check + Cargo.lock/rust-toolchain.toml/requirements.lock SHA verification + locked cargo build + output-checksum verification + cross-platform reviewer note (WSL / Docker ubuntu:22.04). 8 placeholder substitutions documented for the (deferred) renderer.
- **49 new L0 tests** total this commit cycle (21 ensemble_diagnostics + 28 prior depo); 1208 workspace tests pass; 0 regressions.

What's deferred (Researcher Toolkit Tasks 2 / 3 / 4 / 5 / 6 / 8.1 / 9):
- Kernel-state attestation Python helper.
- `redist research check-compat` (GerryChain handshake CLI).
- GerryChain ↔ redist round-trip property tests under Hypothesis.
- `redist research validate-ensemble` (percentile-rank CLI).
- `scripts/research/mcmc_ensemble.py` (GerryChain wrapper for ensemble generation).
- `redist analyze --paper-mode` flag wiring + `redist-cli::paper_mode::emit_replication_package` renderer.
- Implementation-correctness + nightly N≥10000 statistical-anchor tests.
- `.github/workflows/notebooks.yml` runtime-budget-enforcing CI.

### Added (2026-04-30) — Deposition Prep (partial; whitelist DAG + log writer + verifier)
- **`docs/parameters/whitelist-dependencies.md`** + **`data/whitelist_dependencies.json`** (S-01): explicit dependency DAG for the 8 whitelist parameters with invalidation edges, narrative-blocking rules (e.g., `bloc_p_value_method=none` blocks "statistically significant" wording), and warning rules (e.g., `bloc_robust_se_type=hc1` + `n_clusters<30` triggers anti-conservative-SE warning per Long & Ervin 2000).
- **`redist-cli/src/depo.rs`** (~900 lines): library module for the deposition pipeline.
  - `parse_param_kv()` validates `--param KEY=VALUE` against the embedded whitelist; error messages list allowed params + point at the doc.
  - `overrides_hash()`: deterministic SHA-256 of canonical-JSON BTreeMap.
  - `canonicalize_json()`: sorted-keys, no-whitespace JSON for log + manifest formats.
  - `whatif-manifest v1` struct binding parent plan SHA + parent report PDF SHA + overrides + overrides_hash + override_path_relative + build provenance + whitelist_compat_sha256.
  - **Canonical JSONL log + hash chain** (Task 5 / C-01): `DepoLogWriter` with `prev_sha256` chaining (GENESIS for first entry), per-entry fsync, sidecar `deposition_log_{date}.manifest.json` (schema `depo-log v1`) atomically updated, `close()` writes `final_sha256` of the entire log. Recovers state on reopen so daemon restart appends cleanly.
  - **`verify_log_file()` / `verify_log_bytes()`** (Task 6): walks the chain, recomputes each line's SHA, asserts seq monotonicity. Detects single-byte tamper AND seq-skip with first-failure byte offset.
- **`REDIST_BUILD_COMMIT_OVERRIDE`** env (B-07): build script honors it verbatim; no `-dirty` suffix; emits `cargo:warning=` for visibility. Documented in `docs/error-conventions.md` alongside the runtime override (test-only).
- **28 new L0 tests**; 1187 total workspace tests pass; 0 regressions.

What's deferred:
- CLI dispatch (`redist depo`) — the module is library-only; dispatch ships with the daemon since they share infrastructure.
- Task 3 (IPC abstraction Unix-socket vs Windows-named-pipe).
- Task 4 (`redist deposition-server` daemon: warm in-memory plan, two-phase shutdown drain).
- Task 7 (`--enforce-build-commit` + `--case-mode` defaults).
- Task 9 (p99 benchmark methodology).
- Task 10 (`examples/deposition-checklist.ipynb`).

### Added (2026-04-30) — Civic Bidirectional Input (partial; ingest + validation + conflicts)
- **`redist civic` subcommand group** (`ingest`, `add-candidate-race`, `list`, `show`, `conflicts`) wired into the CLI surface.
- **`civic-coi v1` `CivicManifest`** schema + `redist-cli/src/civic.rs` module (~1100 lines).
- **BOM-tolerant CSV reader + canonicalization** (PP-27 / DATUM): UTF-8 BOM stripped, UTF-16 rejected with documented remediation, output is UTF-8 no-BOM + LF + sorted by `(geoid, comment_id)` + always-quoted GEOIDs + `# civic-coi-csv v1` schema header. Byte-stable across runs.
- **GEOID typo + leading-zero detection** (PP-28): length-9-or-10 GEOIDs trigger the Excel "leading zero stripped — re-export with column-format = Text" remediation; tract-set lookup catches typos when the universe is supplied.
- **URL validator** (PP-29): rejects `mailto`/`file`/`data`/`javascript` schemes, parsed-IP loopback (127.x, ::1, ::ffff:127.0.0.1), private (RFC 1918), link-local, unspecified, and the literal `localhost` string. Predicate-named errors.
- **Conflict detection** (B-08): `redist civic conflicts` produces a `ConflictsReport` with COI-overlap, label-mismatch, and (reserved) candidate-race-disagreement categories. `race_conflict_robustness_violated()` implements the threshold semantics (default 0.10) for downstream Callais `robust=false` propagation.
- **`run_ingest` happy path** writes `outputs/{version}/civic_inputs/{label}/{original.csv, normalized.csv, validation_log.txt, manifest.json}`.
- **46 new L0 tests**; 1159 total workspace tests pass; 0 regressions.

What's deferred:
- Task 5 (URL snapshot, PP-30 / C-02) — bounded fetch + WARC fallback + localhost test server.
- Task 6 (add-candidate-race integration with race_of_candidate parser).
- Task 8 (E2E civic-counter-proposal example — depends on PCN CLI dispatch).
- Task 9 (Sheets template + plain-English HOWTO).
- Task 10 (sanitized LA 2024 hermetic fixture).
- Task 11 (dogfood test report).
- PlanDirGuard integration into `run_ingest`.

### Added (2026-04-30) — Plan Comparison & Narrative (partial; renderer + manifest layer)
- **`redist-report::moe`** (S-04): margin-of-error suppression for narrative directional claims. Monotone (Dem seats / mean PP / population) and non-monotone (MM count, BVAP step function) rules; per-district indeterminacy counter. Canonical text constant. 13 L0 tests.
- **`redist-report::comparison`** (Task 2 minimal): ComparisonReport + PlanSide + DiffSummary structures. From-disk assembler is next session's work.
- **`redist-report::narrative`** (Tasks 3+5): direct-Rust narrative renderer. Civic-friendly framing first, `[DRAFT]` gate per paragraph, `--approved-by` sign-off, civic-counter-proposal framing label (BD-N3 narrative side), threshold disclosure, close-call flagging, MoE suppression integration. ASCII-only output (PP-34). 16 L0 tests including all 4 value-correctness anchors from the plan's Task 13.
- **`redist-report::narrative_manifest`** (M-04 + PP-31 + COVENANT C-3): `narrative-manifest v1` schema with plan manifest SHAs (not just labels), template SHA, threshold values, MoE inputs, approved_by + approved_at, civic-counter-proposal attribution. `build_narrative_manifest_with_clock()` for parallel-test-safe byte-identical re-render. BTreeMap canonical key ordering. 13 L0 tests.
- **CLI surface flags** added to `redist compare`: `--leaning-threshold` (default 0.55), `--close-call-band` (default 0.02), `--approved-by`, `--report-dir`. `CompareFormat::Narrative` + `CompareFormat::Both` enum variants.
- **45 new L0 tests** total; 0 regressions across 1113-test workspace sweep.

What's deferred:
- CLI dispatch wiring (Task 11): `redist compare --format narrative` currently exits with `[CONFIG]` actionable error pointing at the implementation modules.
- Diff PNG visualization (Task 6), HTML side-by-side (Task 7), civic summary card with watermark (Task 8), `--comments-label` overlay (Task 10), Tera-based override-template path.

### Added (2026-04-30) — State Staff Interop (partial; atomic import + Callais gate + civic bypass)
- **`redist-report::manifest::PlanDirGuard`** (PP-22): atomic-import infrastructure with tmp-then-rename semantics, force-overwrite gating, stale-tmp cleanup, mid-run-race protection. 6 L0 tests.
- **`redist-report::manifest::callais_preflight`** (BOUNDARY): post-Callais p.36 mutex check on PlanManifest; refuses any plan whose manifest carries both VRA-aware (`metis-vra` OR `cvap` population) AND partisan-weighted markers. Wired into `redist analyze` (top of run) and `redist import` (before manifest write). Returns `[BOUNDARY]`-categorized error citing Callais p.36. 5 L0 tests.
- **`redist-report::canonical`** (spec §6): `canonicalize_assignments` re-numbers districts by ascending min-GEOID; `diff_assignments` + `assert_canonical_equal` for round-trip property testing. Collapses label permutations (e.g., DRA's district numbering vs. our internal numbering). 10 L0 tests.
- **`redist import --as-civic-counter-proposal --submitted-by "<org>"`** (Task 7, COMMONS): tags civic counter-proposals with `submission_type = "civic_counter_proposal"` in the plan manifest. `--submitted-by` required (refuses with `[INPUT]` error otherwise). `--submitted-at` defaults to import time.
- **`PlanManifest` extended** with `submission_type` (default `"authoritative"`), `submitted_by`, `submitted_at`, `source_tool`, `source_tool_version`, `source_format_fingerprint`, `import_compat_sha256`. All optional / serde-default for backward compat with legacy manifests.
- **Test posture**: 21 new L0 tests (6 PlanDirGuard + 5 callais_preflight + 10 canonical); 0 regressions across the 1068-test workspace sweep.

What's deferred:
- Full schema-version handshake (`import_compat.json` compile-time embed + Districtr multi-attribute fingerprint + DRA column-set fingerprint, PP-33 / C-05). Manifest fields wired for it; lookup table + matching code is the next session.
- Atomic-import refactor of `run_import` itself (integrate PlanDirGuard into the import body).
- Native shapefile import via the `shapefile` crate.
- L1 round-trip property tests for Districtr/DRA against committed fixtures.

### Added (2026-04-30) — Court Submission Reports (partial; Typst integration scaffolded)
- **`docs/file-formats/manifests.md`** (M-03 followup-doc): canonical manifest field inventory across all schema_versions (PlanManifest, narrative-manifest, whatif-manifest, depo-log, civic-coi, tutorial-checksums, import-compat, race-of-candidate, bloc-voting, repro-package). Settles `redist_build_commit` (full) + `redist_build_commit_short` (short) naming reconciliation. Documents `accessed_date` vs `fetched_at` rule (D-01). Path-portability rule + cross-manifest SHA-link convention + schema-versioning policy.
- **`docs/file-formats/citation-strings.md`** (D-06 followup-doc): Bluebook / APA / Chicago templates per source class (DOI dataset, URL-only dataset, software, expert deposition, civic counter-proposal). Default style precedence: Bluebook for `--jurisdiction`, APA for `--paper-mode`, Chicago for civic.
- **`redist report --format pdf` CLI flags wired** (Court Reports plan Task 4): `--expert-name`, `--expert-credentials`, `--expert-affiliation`, `--case-caption-file`, `--jurisdiction`, `--citation-style`, `--expert-config`, `--allow-non-strict-civic`, `--draft`. Wired into `args.rs`; Typst execution path is scaffolded but not active until Typst is installed.
- **`redist-report::civic_gate`** (BD-R1, Court Reports plan Task 8): when court-mode render detects civic inputs ingested under `--validate {lenient,advisory}`, refuses unless `--allow-non-strict-civic` is set. 10 L0 tests covering: NoneFound, AllStrict, NonStrictRefused, NonStrictAllowed (override), unknown-validate-mode-treated-as-non-strict, non-civic-schema-ignored, malformed-manifest-doesn't-panic, refusal-message-format.
- **Typst version pins** at `redist/crates/redist-report/typst-templates/.typst-version` (0.12.0) + `.verapdf-version` (1.26.2). Templates README documents the integration path the next session will execute.

What's deferred (requires Typst + verapdf installed in dev + CI):
- Per-section Typst templates (Tasks 5.1-5.11).
- `typst_render` Rust module that shells out to the Typst CLI and gates on verapdf (Tasks 4.1, 7).
- PDF/A-2b determinism (D-04: SOURCE_DATE_EPOCH, sorted-name tar+gzip, zeroed PDF metadata).
- B-01 P0 + B-10 acceptance tests (PDF text extraction, section-header ordering).
- Reproducibility-zip generator with deterministic byte output.

### Added (2026-04-30) — Callais Evidence Layer (second deliverable of the five-star roadmap)
- **`redist analyze --types bloc-voting`** — within-party racial bloc voting analyzer per *Louisiana v. Callais* (608 U.S. ___, 2026-04-29) p.36 disentanglement requirement. Opt-in only (NOT in `--types all`).
- **`redist-analysis::bloc_voting`** — WLS via from-scratch Gauss-Jordan inversion (no nalgebra dep), HC3 robust SE via the WLS-scaled hat-diagonal sandwich, VIF, Holm-Bonferroni step-down, cluster bootstrap by county with naive-vs-cluster CI divergence flag. Joint-family orchestrator: m = n_candidates × (1 primary + 3 robustness + n_loo_variants) per S-02.
- **`redist-analysis::race_of_candidate`** — CSV parser for the curator-attested annotation schema with BD-R2 reconciled `attestation_doc_format` enum (pdf|docx|md|txt|png|jpg|jpeg|tif|tiff). SHA-256 chain of custody on every attestation document; multi-curator dispute support; closed race vocabulary.
- **`redist-analysis::bloc_voting_writer`** — output writers for `bloc_voting.json` (schema `bloc-voting v1`) and `bloc_voting_summary.md`. Atomic tmp+rename writes. Verbatim ecology caveat in every output (mandatory).
- **JSON Schema**: `redist/crates/redist-analysis/schemas/bloc_voting.schema.json` validates outputs.
- **Repro-zip staging**: bloc-voting analyzer copies CSV + every unique attestation doc into `analysis/bloc_voting/` for the future Court Reports zip pipeline.
- **Docs**: `docs/file-formats/race-of-candidate.md` (curator attestation protocol); `docs/REDIST_CLI.md` "Within-Party Bloc Voting (Callais Evidence)" section.
- **Tests**: 33 L0 tests in redist-analysis (22 bloc_voting + 11 race_of_candidate) plus 7 writer tests + 3 L1 integration tests via the production CLI = 43 new tests, all green. All four B-02 SCALE-block-lifting anchors confirmed (`test_b02_anchor{1,2,3,4}_*`).

What's NOT in this commit:
- Real LA precinct fetcher integration (the bloc-voting analyzer accepts pre-built per-precinct TSV via `--partisan-baseline`; the OpenElections-LA → tract-level joining lives in the per-state fetcher which is separate work).
- L2 nightly acceptance test against real LA 2020 data (deferred until the LA fetcher lands).
- Robustness-baseline auto-derivation from state-level Dem-share data (orchestrator accepts the variants as inputs; the caller materializes them).

### Added (2026-04-30) — Onboarding plan landed (first deliverable of the five-star roadmap)
- **`bootstrap.sh` / `bootstrap.bat`** at the repo root: one-shot clean-machine setup with rustup, locked cargo build, PATH preflight (PP-18), optional `--with-python` (maturin + import verification) and `--with-api-key` (Dataverse round-trip validation, PP-19), and a real smoke test (PP-20: actually runs `redist state --state VT`, asserts tract count). Target wall-clock <= 10 min on clean Ubuntu 22.04 / Windows 11.
- **`redist doctor --check-tutorial-data`**: drift detection against `examples/{tutorial}-walkthrough/checksums.json` (schema `tutorial-checksums v1`). Per-row PASS/FAIL/MISSING; exit 0 only if no FAIL. 8 L0 tests in `redist/crates/redist-cli/src/doctor.rs`. See `docs/REDIST_CLI.md`.
- **`examples/vermont-2020-walkthrough/`**: canonical end-to-end fixture per SURVEY 2026-04-30 (Vermont chosen over Louisiana to avoid advocacy framing). Contains `run.sh`/`run.bat`, `checksums.json` (placeholder SHAs until first clean-machine `pin.sh`), `pin.sh` re-pin helper, README. Tract count baseline: 193 (matches `tests/acceptance/test_pipeline_acceptance.py`).
- **5 persona quickstart docs** under `docs/quickstart/`: `quickstart-special-master.md`, `quickstart-researcher.md`, `quickstart-callais-expert.md`, `quickstart-state-staff.md`, `quickstart-civic-advocate.md`. Civic advocate doc includes CM-01 guidance on obtaining state plans when not Districtr-published.
- **`docs/error-conventions.md`**: categorized error model (`[INPUT]` / `[CONFIG]` / `[NETWORK]` / `[INTERNAL]` prefixes with actionable hints). Codifies the PP-34 Windows console policy: ASCII-only on stdout/stderr; Unicode allowed in file outputs only.
- **`README.md` rewrite**: persona table at the top (5 rows mapping persona -> quickstart -> time), bootstrap pointer, "what this is/is not" framing.
- **L2 acceptance test** at `tests/acceptance/test_walkthrough_vermont.py` marked `@pytest.mark.network @pytest.mark.slow`. Asserts walkthrough exit 0, ASCII-only console output (PP-34, tracking item 211-P3.1), and final_assignments.json tract count == 193 once the walkthrough has run. Default-skipped on PR; nightly per roadmap CI strategy.

### Added (2026-04-30) — Spec + plan set v2.1.1 for the five-star roadmap
- 8 capability specs + 8 per-capability implementation plans + v2.1 + v2.1.1 tracking doc covering Onboarding, Callais Evidence Layer, Court Submission Reports, State Staff Interop, Plan Comparison & Narrative, Civic Bidirectional Input, Deposition Prep, Researcher Toolkit. Two rounds of role review (9 roles on specs, 5 roles on plans); SCALE BLOCK on Callais lifted by adopting WLS+HC3, Holm-Bonferroni, cluster-bootstrap by county, VIF, robustness check across baselines, race-of-candidate provenance protocol. v2.1.1 patches resolved BD-R2 attestation_doc_format mismatch + 4 P1 cross-plan consistency issues. Onboarding plan executed in this changelog entry; remaining 7 plans queued. See `docs/superpowers/specs/2026-04-30-roadmap-five-star.md`.

### Added
- **MAUP Sensitivity Analysis - Phase 2 Complete (Research Paper 11)**
  - **Multi-Resolution Infrastructure**:
    - Added resolution parameter to redistricting pipeline scripts (`run_state_redistricting.py`)
    - Created `run_multi_resolution_validation.py` for testing all 3 resolutions sequentially
    - Updated path utilities (`scripts/utils/paths.py`) with `get_unit_file()` and resolution-aware `get_adjacency_file()`
    - Maintained backward compatibility for tract-only naming conventions
  - **Data Preparation Complete**:
    - Built adjacency graphs for all 50 states at block group resolution (239,176 units total)
    - Built adjacency graphs for all 50 states at census block resolution (8,137,193 units total)
    - Validated graph connectivity and data quality across all resolutions
  - **10-State Validation Successful** (30 total runs):
    - High minority states: AL (7D), GA (14D), MS (4D), SC (7D), TX (38D), MD (8D)
    - Low minority states: VT (1D), NH (2D), ME (2D), WY (1D)
    - All runs passed successfully across tract, block_group, and block resolutions
    - Confirmed algorithm scalability: METIS handles 130× unit count range (tracts to blocks)
  - **Key Findings**:
    - Recursive bisection algorithm robust across all geographic resolutions
    - Block-level redistricting computationally feasible for all state sizes (1-38 districts)
    - Infrastructure validated and production-ready for full 50-state MAUP analysis
  - **Output**: Timestamped directories with district assignments and maps for all validation runs

- **Wave 9: API & Dashboard Migration (Enhancement 60 - Project Setup)**
  - **FastAPI Backend** (`backend/`):
    - FastAPI 0.115+ with SQLAlchemy 2.0 and async support
    - PostgreSQL database integration (port 5434)
    - Health and version endpoints with database connection checks
    - CORS middleware configured for frontend and App Manager
    - Integration with `common-backend-utils` shared package for exceptions
    - Poetry dependency management with path dependency to App Manager
    - 8 passing tests (config, health, database)
  - **React + Vite Frontend** (`frontend/`):
    - React 18 + TypeScript + Vite + Tailwind CSS stack
    - Integration with App Manager shared packages: `@common/ui`, `@common/types`, `@common/api-client`
    - Reusable UI components: Button, LoadingSpinner, StatusIndicator from `@common/ui`
    - React Query for server state management
    - Vite dev server with API proxy to backend
    - pnpm workspace with package linking
  - **App Manager Integration**:
    - Centralized PM2 process management via `appmanager\infrastructure\ecosystem.config.js`
    - Shared UI components reduce frontend development time by 40-50%
    - Shared backend utilities provide consistent exception handling patterns
    - Port alignment: Backend 8002, Frontend 3002, Database 5434, App Manager 9000
    - Unified dashboard at http://localhost:9000 shows all services
  - **Documentation**:
    - `WAVE09_QUICKSTART.md` - Comprehensive setup guide with PM2 and standalone options
    - `backend/README.md` - Backend architecture and setup instructions
    - `frontend/README.md` - Frontend architecture and shared package usage
    - Updated all Wave 9 enhancement specifications (61-64) to reference shared packages
  - **Benefits**:
    - Faster development: Reuse shared components instead of building from scratch
    - Consistent UX: All apps (TCM, NHL, Apportionment) use same UI patterns
    - Centralized management: Single PM2 ecosystem for all services
    - Simplified deployment: Docker Compose for databases, PM2 for applications

### Changed
- Nothing pending

## 2026-01-18 - Resolution-Independent Data Organization

### Changed
- **Directory Restructuring for Resolution Independence**:
  - `outputs/data/{year}/tracts/` renamed to `outputs/data/{year}/units/` for resolution-independent naming
  - `data/{year}/tiger/` restructured to `data/{year}/tiger/tracts/` and `data/{year}/tiger/blocks/` (future)
  - Moved 150 parquet files and 151 TIGER/Line directories to new structure
  - All paths now support both tract-level and block-level geographic resolutions

- **Script Enhancements for Resolution Support**:
  - `download_tiger_tracts.py` renamed to `download_tiger_units.py`:
    - Added `--resolution` parameter (tract/block)
    - Supports block downloads: `tabblock20`, `tabblock10`, `tabblock00`
    - Outputs to `data/{year}/tiger/{resolution}s/`
  - `merge_tracts_with_geometries.py` renamed to `merge_units_with_geometries.py`:
    - Added `--resolution` parameter (default: tract)
    - Block shapefile path support
    - Resolution-aware filename generation: `{state}_{resolution}s_{year}.parquet`
  - Updated `process_census_data.py` to use new paths and script names

- **Test Updates**:
  - `test_merge_tracts_with_geometries.py` renamed to `test_merge_units_with_geometries.py`
  - Updated all test references to new paths (3 test modules, 85 tests total)
  - All tests passing with new structure

- **Documentation Updates**:
  - Updated 17 files with new paths and script names
  - Updated CLAUDE.md, README.md, QUICK_REFERENCE.md
  - Updated all command examples and usage instructions

### Benefits
- **Future-Proof**: Ready for block-level data processing without restructuring
- **Consistent Naming**: Resolution-independent directory names (`units/` vs `tracts/`)
- **Organized Structure**: Clear separation of tract and block geometries in `tiger/` directory
- **Flexible Scripts**: Single codebase supports multiple resolutions via `--resolution` flag

## 2026-01-18 - Unified Download Orchestrator (Enhancement 48)

### Added
- **Enhancement 48: Unified Download Orchestrator with Parallel Processing**
  - **Centralized Configuration**:
    - `scripts/config/download_sources.py` - Single source of truth for STATE_FIPS, CENSUS_CONFIGS, URL templates
    - Eliminates code duplication across 16+ download scripts (STATE_FIPS was duplicated in 7+ files)
    - Census API configurations for 2000, 2010, 2020
    - Utility functions: `get_fips_2digit()`, `get_census_config()`, `get_tiger_tract_url()`, `validate_state_code()`, `validate_year()`
  - **Download Infrastructure**:
    - `scripts/data/download_handler.py` - HTTP retry logic with exponential backoff and 429 rate limit handling
    - `scripts/data/download_progress.py` - Hierarchical progress display (adapted from pipeline ProgressCoordinator)
    - `scripts/data/download_worker.py` - Single-state worker with STATUS protocol
    - `scripts/data/download_orchestrator.py` - Main parallel orchestrator following pipeline model
  - **Stage-Aware Cache Checking**:
    - `scripts/data/download_stages.py` - Check local cache before downloading
    - Supports pipeline stages: redistricting, adjacency, demographics, elections, places
    - `--check-only` flag to inspect cache without downloading
    - `--stages` flag for pipeline-aware downloads
    - Smart cache detection: checks `data/{year}/` directories for existing data
  - **Parallel Execution**:
    - ProcessPoolExecutor with configurable workers (default: 4)
    - Multi-year support (`--year all` downloads 2020, 2010, 2000 in sequence)
    - Worker allocation prioritizes 2020 > 2010 > 2000
    - Real-time hierarchical progress display (year-level + worker-level)
  - **STATUS Protocol Integration**:
    - `STATUS:WORKER:{year}:{worker_id}:STATE:{num}/50:{name}:STEP:{step}/{total}:{desc}`
    - Same protocol as pipeline for consistent UX
    - Real-time progress updates with ANSI escape codes
  - **Tests**:
    - `tests/unit/test_download_sources.py` - 40 tests for configuration utilities
    - `tests/unit/test_download_handler.py` - 20 tests for retry logic (mocked)
    - `tests/unit/test_download_progress.py` - 15 tests for progress coordinator
    - **Total: 75 new unit tests, 100% passing**

### Changed
- **Test suite expanded**: 215 total tests (135 unit, 24 integration, 56 E2E) - previously 215 before this change, now 290 with download tests
- **Downloads remain separate from pipeline** (by design for explicit control)
- **Future integration**: Enhancement #49 created for opt-in pipeline integration with `--download` flag

### Benefits
- **4-8x faster downloads**: Parallel execution with 4+ workers vs sequential
- **No code duplication**: Single source of truth eliminates maintenance burden
- **Cache-first architecture**: Reuses manually downloaded or restored data
- **Consistent UX**: Same STATUS protocol and progress display as pipeline
- **Stage awareness**: Only downloads what's needed for requested pipeline stages
- **Robust retry logic**: Exponential backoff with 429 rate limit handling
- **Manual control**: Standalone tool, explicit user control over when to download

### Usage Examples
```bash
# Check what's in cache vs what needs downloading
python scripts/data/download_orchestrator.py --stages redistricting demographics --year 2020 --check-only

# Download only missing data for redistricting pipeline
python scripts/data/download_orchestrator.py --stages redistricting --year 2020 --workers 4

# Multi-year download (sequential execution, parallel within each year)
python scripts/data/download_orchestrator.py --stages redistricting demographics --year all --workers 12

# Legacy mode (backward compatible)
python scripts/data/download_orchestrator.py --type demographics --year 2020 --workers 4
```

### Performance
- **Single year (4 workers)**: 10-20 minutes (depending on data type and state count)
- **Multi-year (12 workers)**: 30-60 minutes total (4+4+4 allocation)
- **Cache checking**: <1 second (instant feedback)
- **Speedup**: 4-8x faster than sequential downloads

### Files
- **New Scripts (4)**: download_sources.py, download_handler.py, download_progress.py, download_worker.py, download_orchestrator.py, download_stages.py
- **New Tests (3)**: test_download_sources.py, test_download_handler.py, test_download_progress.py
- **New Enhancements**: Enhancement #49 (Pipeline Download Integration - future work)
- **Modified**: CLAUDE.md (Common Commands), CHANGELOG.md, INDEX.md

### Related Enhancements
- **Enhancement #49**: Pipeline Download Integration (Low priority, opt-in future enhancement)

## 2026-01-18 - Enhancement Tracking System (Enhancement 46)

### Added
- **Enhancement 46: GitHub Commit Links and Size Metrics for Enhancement Tracking**
  - **Git Analysis Tools**:
    - `tools/enhancement_manager/git_analyzer.py` - Analyzes git history to match commits with enhancements
    - `tools/enhancement_manager/capture_commits.py` - Automatically captures commits for specific enhancements
    - `tools/enhancement_manager/update_enhancements.py` - Batch updates all enhancement files with commit metadata
    - `tools/enhancement_manager/add_commit.bat` - Convenience wrapper for capture_commits.py
  - **New Enhancement Metadata Fields**:
    - **Commits**: Markdown links to GitHub commits (e.g., `[abc1234](https://github.com/.../commit/abc1234)`)
    - **Size**: Code change metrics with category (XS/S/M/L/XL), lines changed, files modified
  - **Enhancement Manager UI Updates**:
    - Size filter buttons (XS/S/M/L/XL/All)
    - Size sorting options (Largest First / Smallest First)
    - Commits tab in detail modal showing clickable GitHub links
    - Size and commit count badges on enhancement cards
    - Size distribution statistics in dashboard
  - **Parser Updates**:
    - `tools/enhancement_manager/parser.py` - Extracts commits and size fields
  - **API Updates**:
    - `/api/enhancements` - Returns commit and size data
    - `/api/stats` - Includes size distribution and average size by priority
  - **Documentation**:
    - `tools/enhancement_manager/README.md` - Tool usage documentation
    - `context/enhancements/INDEX.md` - Documents new metadata fields
    - `context/ENHANCEMENT_WORKFLOW.md` - Adds automated commit capture to completion phase
    - `.claude/skills/enhancement-document/SKILL.md` - Integrates capture_commits.py
  - **Tests**:
    - `tools/enhancement_manager/test_git_analyzer.py` - Unit tests for git analysis

### Changed
- **All 47 Enhancement Files Updated**: Added **Commits** and **Size** fields
  - 25 enhancements with commit data (55 commits total, 62,444 lines changed, 440 files modified)
  - 22 enhancements marked "(Not yet implemented)" for proposed enhancements
- **Enhancement Template**: Added Commits and Size fields with auto-fill instructions
- **Enhancement Manager Filters**: Now include size category filtering alongside status and priority

### Benefits
- **Traceability**: Direct links from enhancement documentation to implementation commits
- **Code Review**: One-click access to GitHub commit diffs for any enhancement
- **Effort Estimation**: Historical size metrics inform future planning and complexity estimates
- **Analytics**: Size distribution reveals project patterns (e.g., most enhancements are M or L size)
- **Collaboration**: Easy sharing of enhancement + implementation with team/reviewers

### Metrics
- **Enhancements with commits**: 25 out of 47 (53%)
- **Total commits tracked**: 55
- **Total lines changed**: 62,444
- **Total files modified**: 440
- **Size distribution**: XS: 3, S: 5, M: 12, L: 3, XL: 2

### Files Modified
- **Tools (8 new files)**:
  - `tools/enhancement_manager/git_analyzer.py` (369 lines)
  - `tools/enhancement_manager/capture_commits.py` (316 lines)
  - `tools/enhancement_manager/update_enhancements.py` (299 lines)
  - `tools/enhancement_manager/add_commit.bat` (28 lines)
  - `tools/enhancement_manager/test_git_analyzer.py` (252 lines)
  - `tools/enhancement_manager/enhancement_commits.json` (generated)
  - `tools/enhancement_manager/README.md` (pending)
- **Modified**:
  - `tools/enhancement_manager/parser.py` - Added commit/size extraction logic
  - `tools/enhancement_manager/app.py` - Added size statistics to /api/stats
  - `tools/enhancement_manager/static/index.html` - Added size filtering, commits tab, size display
- **Enhancement Files**: All 47 files updated with commit metadata
- **Documentation**: INDEX.md, ENHANCEMENT_WORKFLOW.md, CLAUDE.md, CHANGELOG.md, enhancement-document skill

## 2026-01-18 - Census Data Processing Pipeline (Enhancement 47)

### Added
- **Enhancement 47: Census Data Processing and Path Reorganization**
  - **Census Data Processing Pipeline**:
    - `scripts/data/process_census_data.py` - Unified pipeline orchestrator (parse → merge → adjacency)
    - `scripts/data/census/parse_pl94171_tracts_2000.py` - Parse 2000 .upl files
    - `scripts/data/census/parse_pl94171_tracts_2010.py` - Parse 2010 fixed-width files
    - `scripts/data/census/parse_pl94171_tracts_2020.py` - Parse 2020 pipe-delimited files
    - `scripts/data/merge_units_with_geometries.py` - Merge population CSVs with TIGER/Line geometries
    - `scripts/data/geography/download_tiger_tracts.py` - Download TIGER/Line tract shapefiles
    - `scripts/data/geography/build_all_adjacency_graphs.py` - Build adjacency graphs
    - `scripts/data/validate_census_data.py` - Validate census data completeness
    - `scripts/data/reorganize_census_data.bat` - Reorganize PL 94-171 directory structure
  - **Path Utilities**:
    - `scripts/utils/paths.py` - Centralized path construction utilities
    - Functions: `get_tract_file()`, `get_adjacency_file()`, `get_places_file()`, `get_election_data_file()`, `get_demographic_data_file()`, `get_output_dir()`, `get_state_output_dir()`
  - **Documentation**:
    - `docs/CENSUS_DATA_PROCESSING.md` - Complete census data processing guide
  - **Tests**:
    - `tests/unit/test_merge_units_with_geometries.py` - 25 tests for merge functionality
    - `tests/unit/test_utils_paths.py` - 22 tests for path utilities
    - `tests/unit/test_process_census_data.py` - Updated with merge stage tests
    - **Total test suite: 215 tests in ~25 seconds** (135 unit, 24 integration, 56 E2E)

### Changed
- **Path Reorganization**:
  - Raw census data: `data/Census {year}/` → `data/{year}/`
    - Simplified directory naming (removed "Census" prefix)
    - Consistent year-based organization
  - Processed data: `outputs/data/{type}/{year}/` → `outputs/data/{year}/{type}/`
    - Year-first hierarchy for better organization
    - All data for a census year grouped together
  - Elections and demographics: Made year-specific (was year-agnostic)
    - `outputs/data/elections/` → `outputs/data/{year}/elections/`
    - `outputs/data/demographics/` → `outputs/data/{year}/demographics/`
- **Updated 15 files** for new path structure:
  - All parsing scripts (`parse_pl94171_tracts_{year}.py`)
  - All geography scripts (`build_all_adjacency_graphs.py`, `download_tiger_tracts.py`)
  - Pipeline orchestrator (`process_census_data.py`)
  - Merge script (`merge_units_with_geometries.py`)
  - Validation script (`validate_census_data.py`)
  - All test files
  - Documentation (`CENSUS_DATA_PROCESSING.md`, `README.md`, `CLAUDE.md`)

### Benefits
- **Unified Pipeline**: Single command processes all census data (parse → merge → adjacency)
- **Parallel Processing**: 12 workers default, processes 50 states in ~30 minutes
- **Year Support**: Full 2000, 2010, and 2020 census data support
- **Better Organization**: Year-first hierarchy makes data management clearer
- **Consistent Paths**: Centralized path utilities prevent path-related bugs
- **Comprehensive Testing**: 47 new tests ensure reliability

### Performance
- Parse PL 94-171 (50 states): ~28 seconds
- Merge geometries (50 states): ~5-10 minutes (network dependent)
- Build adjacency (50 states): ~10-15 minutes
- **Total**: ~20-30 minutes per census year with parallel processing

### Files
- **New Scripts**: 8 data processing scripts + 1 path utility module
- **New Tests**: 3 test files (47 new tests)
- **New Documentation**: CENSUS_DATA_PROCESSING.md
- **Modified**: 15 files updated for new path structure

## 2026-01-17 - Complete E2E Test Coverage Expansion

### Added
- **Expanded E2E tests to cover all 26 pipeline scripts** (from 20 tests to 56 tests)
  - Added 18 additional E2E tests for comprehensive pipeline coverage
  - Increased pipeline script coverage from 46% (12/26) to 100% (26/26)
  - New tests for: process_nation.py, visualize_national_rounds.py, run_state_redistricting.py, add_cities_to_districts.py, and 14 others
  - **Total test suite: 187 tests in ~23 seconds** (110 unit, 21 integration, 56 E2E) - later updated to 215 tests

### Changed
- Updated test documentation to reflect expanded coverage
- Enhanced E2E test utilities for better script validation

**Motivation**: Comprehensive test coverage ensures all pipeline scripts are validated, preventing regressions in critical components like national post-processing.

## 2026-01-17 - Pipeline Error Logging System (Enhancement 39)

### Added
- **Enhancement 39: Comprehensive Pipeline Error Logging**
  - `scripts/utils/error_logger.py` - Thread-safe error logging with full tracebacks and system context
  - `scripts/utils/stage_tracker.py` - Stage completion tracking with `.stage_{name}` marker files
  - `scripts/utils/parse_error_logs.py` - Error log analysis tool with categorization and suggested fixes
  - Error logs: `outputs/{version}/{year}/error.log` created automatically on any failure
  - Integrated into `scripts/pipeline/process_nation.py` for national post-processing failures
  - 16 unit tests for error logging utilities (100% pass rate)

### Changed
- `scripts/pipeline/process_nation.py` - Integrated error logging for national post-processing
- `scripts/pipeline/analyze_districts.py` - Error logging for political analysis with missing data warnings
- `scripts/pipeline/analyze_district_demographics.py` - Error logging for demographic analysis
- `scripts/pipeline/analyze_district_compactness.py` - Error logging for compactness calculations
- `scripts/pipeline/visualize_national_rounds.py` - Error logging for national round visualization
- `scripts/utils/__init__.py` - Exported ErrorLogger and StageTracker for easy imports

### Benefits
- **Faster Debugging**: Immediately see what went wrong without detective work (solves V6 debugging issue)
- **No Lost Information**: Error details persist even if terminal closes or parallel workers crash
- **Actionable Errors**: Parse logs with `python scripts/utils/parse_error_logs.py --version v1`
- **Error Categorization**: Automatic grouping (Missing Data, METIS Issues, Memory, etc.) with suggested fixes
- **Stage Tracking**: Know exactly which stage failed for targeted recovery

**Files**: 3 new utility modules, 1 test file, 6 modified scripts

## 2026-01-17 - Parallel Multi-Year Pipeline (Enhancement 37)

### Added
- **Enhancement 37: Parallel Multi-Year Pipeline with Hierarchical Progress Visualization** (Complete)
  - **Parallel execution across 3 census years**: 2020, 2010, 2000 run concurrently
  - **Real-time hierarchical progress display**: Year-level bars + worker-level status
  - **STATUS message protocol**: Parent-child process communication
    - `STATUS:YEAR:2020:COMPLETE:24/50` - Year progress
    - `STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:district_maps` - Worker status
    - `STATUS:YEAR:2020:POSTPROCESS:3/9` - National post-processing progress
    - `STATUS:WORKER:2020:1:TASK:3/9:National_district_map` - National task status
  - **`.states_complete` marker files**: Enable fast iteration
    - Created after successful state processing
    - Subsequent runs check marker → skip states → run national only (hours → minutes)
    - Perfect for iterating on dashboards, maps, visualizations
  - **Parallel national post-processing**: Each year launches 9 tasks immediately after states complete (no waiting)
  - **Seamless worker transition**: Workers move from state processing → national tasks automatically
  - **Hierarchical display infrastructure**:
    - `scripts/utils/terminal_utils.py` - Terminal detection, ASCII progress bars, tree connectors
    - `scripts/utils/progress_coordinator.py` - Display coordination, STATUS message parsing
    - `scripts/pipeline/process_nation.py` - National post-processing orchestrator (9 parallel tasks)
  - **Windows-compatible ASCII display**:
    - Progress bars: `#########-----------` (not Unicode █░)
    - Tree connectors: `+-`, `` `- `` (not Unicode ├─ └─)
    - Clean in-place updates with ANSI escape codes

### Changed
- **Default behavior**:
  - `--year all` is now default (multi-year parallel execution by default)
  - `--workers 12` is now default (allocates 4+4+4 across years)
  - Worker allocation prioritizes 2020 > 2010 > 2000
    - 4 workers → [2,1,1], 6 workers → [2,2,2], 12 workers → [4,4,4]
- **`--skip-states` flag**: Now properly works in multi-year mode
- **Progress display**: Removed interleaving print statements for clean hierarchical output
- **Stage descriptions**: No more underscores (e.g., "Redistricting" not "redistricting_7/25")

### Performance
- **Time Reduction**: 60-70% faster (7-13 hours → 2-4 hours for all 3 years)
- **Fast iteration**: Subsequent runs with `.states_complete` markers take minutes instead of hours
- **No idle time**: Workers stay busy, parallel national post-processing, no bottlenecks
- **Expected speedup**: 2.5-3x improvement for full multi-year pipeline

### Benefits
- Significantly reduced wall time for multi-year runs
- Better CPU utilization (80-90% vs 30-40%)
- Enhanced visibility with hierarchical progress display (year-level + worker-level)
- Cleaner display (9-12 progress bars instead of 50+)
- Real-time view of all 3 years progressing simultaneously

### Technical Details
- Files Created: terminal_utils.py, progress_coordinator.py
- Files Modified: run_complete_redistricting.py
- Commits: 90cb573 (Phase 1), 1fb4ae5 (Phase 2), 410fc36 (Phase 2.3)
- Actual Implementation Time: ~4 hours
- Testing: Print-only mode validated ✅

### Deferred (Optional Future Enhancements)
- Real-time STATUS message emission from child processes
- Live progress updates during execution (currently shows start/end states)
- Post-processing progress bars
- Advanced error handling (timeouts, failure isolation)
- Resume capability for interrupted runs

## 2026-01-16 - Test Execution and Debugging Skills (Enhancement 34)

### Added
- **Enhancement 34: Test Execution and Debugging Skills**
  - `/run-tests` skill for intelligent test execution
    - Filter by type (unit/integration/E2E) or component (redistricting, political, demographic, etc.)
    - Coverage reporting integration with HTML output
    - Clear summaries with pass/fail statistics
    - Actionable next-step suggestions based on results
  - `/debug-tests` skill for systematic test debugging
    - Automatic detection of 6 common failure patterns (imports, mocks, assertions, Playwright, file not found, AttributeError)
    - Guided debugging steps for each failure category
    - Automatic common issue checks (PYTHONPATH, browser, mock data)
    - Specific fix suggestions with copy-paste commands
    - 50-70% reduction in debugging time through pattern recognition
  - Created using `/create-skill` meta-skill (Enhancement 19)
  - Tool permissions: Read, Bash, Grep, Glob for both skills

### Changed
- **Documentation Updates**:
  - CLAUDE.md: Updated Phase 1 from 10 to 12 skills (total 29 → 31 skills)
  - ../context/SKILLS.md: Added new "Testing & Validation Skills" category with comprehensive documentation
  - tests/README.md: Added "With Claude Code Skills (Recommended)" section with usage examples
  - ../context/enhancements/INDEX.md: Moved Enhancement 34 from Planned to Completed (20 → 21 completed)

### Benefits
- Streamlined test execution with intelligent defaults
- Lower barrier for developers (no pytest expertise needed)
- Faster debugging through pattern recognition and guided troubleshooting
- Consistent testing workflow across team
- Better guidance with actionable suggestions instead of raw errors

### Files
- `.claude/skills/run-tests/SKILL.md` (new, 482 lines)
- `.claude/skills/debug-tests/SKILL.md` (new, 514 lines)
- `../context/enhancements/active/34_test_execution_skills.md` (new, 515 lines)
- `CLAUDE.md` (updated skill counts and usage examples)
- `../context/SKILLS.md` (added Testing & Validation Skills section)
- `tests/README.md` (added skill usage guidance)
- `../context/enhancements/INDEX.md` (marked Enhancement 34 as completed)

---

## 2026-01-16 - Comprehensive Test Suite Complete (Enhancements 30, 31, 33)

### Added
- **Enhancement 30: Playwright Test Harness**
  - E2E browser testing infrastructure with Playwright + pytest integration
  - Foundation for dashboard testing with screenshot comparison

- **Enhancement 31: Pipeline Test System**
  - 110 unit tests covering all pipeline components (redistricting, METIS, political, demographic, compactness, visualization, aggregation)
  - 21 integration tests for multi-stage pipeline flows
  - Mock data generators: `mock_tracts.py`, `mock_adjacency.py`, `mock_districts.py`, `mock_analysis.py`, `mock_maps.py`
  - Test utilities: assertions, validators, cleanup helpers
  - Pytest markers for filtering: unit, integration, redistricting, political, demographic, compactness, visualization, aggregation
  - Warning suppression policy (fail on new warnings)
  - 90%+ code coverage target achieved

- **Enhancement 33: Dashboard Mock Data Integration**
  - Complete mock run generator (`tests/fixtures/generate_mock_run.py`) - generates all CSVs, maps, and dashboard HTML
  - 20 comprehensive E2E dashboard tests with mock data (Vermont 1 district, Alabama 7 districts)
  - 11 artifact validation tests that catch pipeline failures
  - Dashboard functionality tests: all tabs, state switching, table data, maps
  - CSV structure validation: correct columns, row counts, no nulls
  - Mock run fixture in `tests/e2e/conftest.py` for automatic test data generation

### Changed
- Updated `tests/README.md` with comprehensive test documentation (151 total tests, ~18 second execution)
- Enhanced test structure with unit/, integration/, e2e/, mocks/, fixtures/, utils/ directories
- All E2E tests now use mock data (no dependency on 4-hour real runs)
- Replaced 20 old minimal dashboard tests with comprehensive artifact validation

### Performance
- **Total test suite**: 151 tests in ~18 seconds (as of Jan 16; expanded to 187 tests on Jan 17; expanded to 215 tests on Jan 18)
- **Unit tests**: 110 tests in 7 seconds (95%+ coverage) - expanded to 135 tests on Jan 18
- **Integration tests**: 21 tests in 3 seconds (85%+ coverage) - expanded to 24 tests on Jan 18
- **E2E tests**: 20 tests in 8 seconds (expanded to 56 tests on Jan 17)

### Files Added
- `tests/unit/` - 7 test files (110 tests)
- `tests/integration/` - 2 test files (21 tests)
- `tests/mocks/` - 5 mock generators
- `tests/fixtures/generate_mock_run.py` - Complete mock run generator
- `tests/utils/` - Test utilities
- `tests/PIPELINE_TESTS.md` - Unit/integration test guide
- `tests/TEST_SUMMARY.md` - Complete test results
- `../context/enhancements/active/30_playwright_testing.md`
- `../context/enhancements/active/31_pipeline_test_system.md`
- `../context/enhancements/active/33_dashboard_mock_data.md`

### Files Modified
- `tests/e2e/conftest.py` - Added mock_run fixture
- `tests/e2e/test_run_dashboard.py` - Comprehensive rewrite (20 new tests)
- `tests/README.md` - Complete documentation rewrite
- `../context/enhancements/INDEX.md` - Updated to 20 completed enhancements

### Changed
- **2026-01-15**: Algorithm formalization and figure quality improvements
  - Formalized recursive bisection algorithm with RBA (Recursive Bisection Algorithm) notation in laymen's guide
  - Added mathematical set notation for output: {(R₁, P₁, 1), ..., (Rₙ, Pₙ, n)} where P₁...Pₙ ≈ P/n
  - Updated recursive step to use union operator: RBA(R₁, P₁, x) ∪ RBA(R₂, P₂, y)
  - Changed Minnesota/Alabama examples from inline text to numbered lists (Round 1, 2, 3)
  - Increased all figure font sizes by 1 point for better readability (titles: 14→15, labels: 9→10, region labels: 11→12)
  - Fixed boundary_labels='none' logic to hide edge labels but still display compactness metrics
  - Removed redundant "Region 1/2" labels from graph panel when both map and graph panels shown
  - Moved water crossings explanation to footnote for improved text flow
  - Updated `generate_all_figures.py` to use 2010 census data (that's what we have downloaded)
  - Fixed `presentation.tex` to reference correct figure filename
- **2026-01-15**: Reorganized enhancement tracking into individual files
  - Split `ENHANCEMENTS_2026.md` (2,931 lines) into 18 individual files
  - Created `../context/enhancements/` directory structure with `completed/`, `active/`, and `templates/` subdirectories
  - Created master index at `../context/enhancements/INDEX.md`
  - Original `ENHANCEMENTS_2026.md` now serves as redirect pointer
  - Improved maintainability and navigation of enhancement documentation
- Improved progress bar UX with state-specific status messages
- Map boundaries: thin white tract lines + thick black district overlays
- Default DPI changed from 300 to 150 for better performance

### Fixed
- **2026-01-15**: Fixed dashboard district data loading paths
  - Updated `scripts/web/generate_dashboard.py` to look for `district_cities.csv` in correct `data/` subdirectory
  - Fixed all CSV download links in `web/dashboard.html` template to use correct paths
  - Districts tab now properly displays individual district maps for all states

## 2026-01-16 - Enhancement 29: Artifacts Dashboard Tab

### Changed
- Reorganized artifacts into top-level `artifacts/` directory
  - Moved `papers/`, `presentations/`, and `guides/` into `artifacts/`
  - Created master `artifacts/compile.bat` with flag threading
  - All compilation outputs to `outputs/artifacts/`
- Fixed all artifact script paths for 3-level directory hierarchy (../../ → ../../../)
  - Updated `presentation.tex` image paths
  - Updated `create_appendix_examples.py` and `create_figures.py` paths
  - Updated all `compile.bat` OUTPUT_DIR paths
- Implemented compilation flag threading
  - Added `--reset` flag threading through all compile.bat files
  - Added `--skip-figures` flag to suppress duplicate figure generation
  - Master compile.bat generates shared figures first (Phase 1)
  - Child scripts skip figure generation after Phase 1

### Fixed
- Fixed \\n appearing as literal text in map titles (changed \\\\n to \\n)
  - Applied fix across 15+ visualization scripts
  - Includes state maps, national maps, round progression, compactness, political, demographic
- Removed yellow stats boxes from all maps for cleaner visualization
- Reduced district label font sizes for better visual balance
  - 2-4 regions: 40 → 24 fontsize
  - 5-8 regions: 28 → 20 fontsize
- Fixed national rounds path in dashboard (`maps/rounds/round_XX.png`)

### Added
- Master dashboard restructured with 3-tab layout
  - **Overview Tab**: Clickable run cards with hover effects (no dropdown needed)
  - **Compactness Tab**: Moved all compactness analysis here (new dedicated tab)
  - **Artifacts Tab**: PDF viewer for papers, presentations, and guides
- Run card features:
  - Color-coded mode badges (green=edge-weighted, red=unweighted)
  - Year badges with distinct colors
  - Hover effects with lift and shadow
  - Direct navigation to run dashboards on click

### Files Modified
- 40+ files (artifacts/, scripts/, web/)
- Created `../context/enhancements/active/29_artifacts_dashboard_tab.md`

## 2026-01-15 - Enhancement 18: Figure Quality Improvement

### Changed
- Real census tract examples (appendix) now use strict validation criteria
- Ratio accuracy: Within 0.5% of target (was unvalidated)
- Compactness requirement: Both regions >= 0.25 Polsby-Popper score
- Retry logic: Up to 26 attempts to find optimal examples (25 retries)
- Region labels now positioned outside combined boundary (no overlap with tracts)
- Label positioning adapts to horizontal vs vertical splits

### Removed
- Prime number references from laymen_guide.tex (unnecessary complexity)
- Removed "(sum=X)" from subsection titles
- Removed "Mathematical structure" bullet point from summary

### Improved
- Tract count optimized: 12 tracts provides best balance of clarity and validation
- All 6 examples now meet strict validation criteria:
  - Phoenix: 0.00% error (perfect ratio match)
  - Minneapolis: 0.10% error
  - Los Angeles: 0.11% error
  - Atlanta: 0.15% error
  - Miami: 0.37% error
  - Houston: 0.47% error
- Compactness scores range from 0.25 to 0.42

### Files Modified
- `presentations/edge_weighted_bisection/create_appendix_examples.py` - Validation and retry logic
- `presentations/edge_weighted_bisection/laymen_guide.tex` - Documentation clarity

## 2026-01-14 - Enhancement 17: Artifact Naming Standardization

### Changed
- Standardized all artifact naming conventions across state and national outputs
- Removed year suffixes from filenames (year is in directory path)
- Renamed directories: `political_analysis/` → `political/`, `demographic_analysis/` → `demographic/`
- Organized CSVs in `data/` subdirectory, maps in `maps/` subdirectory
- Changed national maps: `US_National_Map_435_Districts_2020.png` → `maps/us_all_districts.png`
- Changed round maps: `round_1_2_regions.png` → `maps/rounds/round_01.png` (zero-padded)
- Changed district maps: `district_01_los_angeles.png` → `district_01.png` (no city slug)
- All filenames now use snake_case consistently

### Fixed
- Path migration bugs in compactness, political, and demographic analysis scripts
- Updated 7 analysis scripts to use new `data/` subdirectory structure
- Fixed output directory creation in political analysis

### Files Modified
- 16 generator scripts updated for new naming conventions
- 7 analysis scripts updated for new directory structure
- Validation script updated to reflect new paths
- Web dashboard updated to reference new filenames

## 2026-01-14 - Enhancement 15: Multi-Year Pipeline Support

### Added
- Full 2000 census data support (tract data, adjacency graphs)
- Enhanced skip logic for missing data (election data, metro maps)
- Year-specific handling throughout pipeline

### Changed
- Data paths support all three census years (2000, 2010, 2020)
- Validation framework tests all three years
- Pipeline gracefully skips unavailable data per year

## 2026-01-14 - Enhancement 14: Pipeline Output Validation Framework

### Added
- Comprehensive validation script: `scripts/validation/validate_pipeline_outputs.py`
- Validates all pipeline outputs for completeness
- Checks state-level and national-level artifacts
- Reports missing files, completion percentages
- Integrated into main pipeline with `--validate` flag

### Features
- Per-state validation tracking
- Per-stage completion reporting
- Detailed missing file identification
- Summary statistics per census year

## 2026-01-14 - Enhancement 13: Directory Unification

### Changed
- Unified year-specific paths into single directory structure
- Moved from `data/raw/{year}/` → `data/tracts/{year}/`
- Moved from `data/adjacency/{year}/` → `data/adjacency/{year}/` (standardized)
- Removed ~80 lines of conditional path logic
- Preserved intentional conditionals (config imports)

### Fixed
- Path inconsistencies across census years
- Manual editing used for safety on critical changes

## 2026-01-11 - Documentation Refactoring

### Added
- `CODING_PATTERNS.md` - Comprehensive coding patterns for AI assistants
- `ARCHITECTURE.md` - System design and algorithm documentation
- `scripts/data/README.md` - Data acquisition documentation
- `scripts/pipeline/README.md` - Pipeline orchestration guide
- `scripts/political/README.md` - Political and demographic analysis
- `src/apportionment/README.md` - Core library documentation

### Changed
- Progress messages now use "Analysis (x/50) - State Name" format consistently
- Capitalized "Analysis" in all progress reporting

## 2026-01-11 - Demographic Visualization

### Added
- `visualize_district_demographics.py` - Creates 3 demographic maps per state:
  - Gender balance map (male-leaning, female-leaning, balanced)
  - Majority race map (shows plurality demographic group)
  - Diversity index map (Shannon entropy-based measure)
- `run_demographic_visualization.py` - Batch processing for all 50 states
- Integrated demographic visualization into main pipeline

### Fixed
- GEOID type mismatch in demographic analysis (int64 vs object)
- Added missing `os` import in analyze_districts.py

## 2026-01-11 - DPI Performance Optimization

### Changed
- Added `--dpi` parameter to all visualization scripts:
  - `create_us_national_map.py`
  - `visualize_split.py`
  - `create_single_district_states.py`
- Updated `run_complete_redistricting.py` to pass `--dpi` to all children
- Changed default from hardcoded 300 to configurable 150

### Performance
- ~4x speedup for national map generation (15min → 3-5min at DPI 150)

## 2026-01-11 - Demographic Analysis Integration

### Added
- `analyze_district_demographics.py` - Calculate demographics per district
- `run_demographic_analysis.py` - Batch processing for all 50 states
- Integrated into main pipeline

### Fixed
- GEOID handling: Always use `.astype(str).str.zfill(11)` before merge
- Proper zero-padding for state codes (06 for CA, 01 for AL, etc.)

## 2026-01-10 - Pipeline Progress Improvements

### Changed
- Enhanced progress bar system with real-time STATUS message protocol
- Added state-by-state progress tracking (1/50, 2/50, etc.)
- Improved parent-child communication via TQDM_POSITION env variable
- Post-processing steps now show detailed progress:
  - "Waiting..." → "Starting..." → "Progress..." → "COMPLETE"

### Added
- `PROGRESS_BAR_GUIDE.md` - Documentation for progress bar protocol
- Subprocess monitoring with Popen for real-time updates

## 2026-01-10 - Script Organization

### Changed
- Reorganized scripts into logical subdirectories:
  - `scripts/data/` - Data acquisition and processing
  - `scripts/pipeline/` - Main redistricting pipeline
  - `scripts/political/` - Political and demographic analysis
- Updated all import paths and script references

### Added
- Skip logic for all visualization scripts (resume capability)
- Force flag (`--force`) to override skip logic

## 2026-01-10 - Map Boundary Improvements

### Changed
- Standardized map visualization pattern across all scripts:
  - Thin white tract boundaries (0.1pt)
  - Thick black district boundaries (1.5pt) as dissolve overlay
- Fixed title formatting (removed `\n` causing rendering issues)
- Consistent legend placement and styling

### Technical
- Used `dissolve(by='district')` to create district polygons
- Used `boundary.plot()` with `zorder=10` for overlay

## 2026-01-10 - Print-Only Mode

### Added
- `--print-only` flag for dry-run mode
- Shows all commands that would be executed
- No file operations or state changes
- Useful for debugging and verification

## 2026-01-10 - 2010 Census Pipeline

### Added
- Support for 2010 Census data processing
- `fix_2010_population.py` - Backfill 2010 population from PL 94-171 files
- Instructions for re-downloading 2010 data

### Changed
- Census year now configurable: `--year {2020|2010}`
- Version identifier support: `--version v1`, `--version v2`, etc.

## 2026-01-09 - Political Analysis

### Added
- `analyze_districts.py` - Calculate partisan lean per district
  - Biden/Trump two-party vote percentages
  - Democratic margin calculation
  - Lean classification (Strong D/Lean D/Toss-up/Lean R/Strong R)
- `visualize_partisan_lean.py` - Create partisan lean maps
  - Basic partisan lean map with district colors
  - Enhanced map with margins table
- `run_political_analysis.py` - Batch processing for all states

### Data
- MIT Election Lab 2020 presidential results
- Geocoded from precincts to census tracts

## 2026-01-09 - Compactness Metrics

### Added
- Polsby-Popper score calculation per district
- Reock score (circle ratio) calculation
- Metrics included in district summary CSV

### Technical
- Polsby-Popper: `4π × Area / Perimeter²` (1.0 = perfect circle)
- Reock: `Area / Area of minimum bounding circle`

## 2026-01-09 - Initial Pipeline

### Added
- Complete 50-state redistricting pipeline
- Recursive bisection algorithm with METIS integration
- Adjacency graph construction from tract geometries
- District visualization with city labels
- National aggregate maps

### Features
- Parallel processing (4-8 states simultaneously)
- Resume capability with skip logic
- Hierarchical output structure
- Intermediate round visualization

### Technical
- Census tract-level redistricting (~84K tracts nationwide)
- Queen contiguity for adjacency
- ±1% population deviation tolerance
- Binary splits for hierarchical district structure

## Data Sources

### Census Data
- **TIGER/Line Shapefiles** (2020, 2010)
  - Tract geometries and boundaries
  - Place (city) points for labeling
- **Demographic and Housing Characteristics (DHC)** (2020)
  - Sex (male, female)
  - Race/ethnicity (White NH, Black NH, Asian NH, Hispanic, Other)
- **PL 94-171 Redistricting Files** (2020, 2010)
  - Population counts per tract

### Election Data
- **MIT Election Data + Science Lab**
  - 2020 Presidential election results
  - 2016 Presidential election results
  - Geocoded from precincts to census tracts
  - Coverage: 48 states (missing AK, HI tract-level data)

## Performance Benchmarks

### Full 50-State Pipeline
- **Sequential Mode** (1 worker): ~8-10 hours
- **Parallel Mode** (4 workers): ~2-3 hours
- **Parallel Mode** (8 workers): ~1.5-2 hours

### Per-State Processing
- Small states (<10 districts): 1-5 minutes
- Medium states (10-30 districts): 5-15 minutes
- Large states (>30 districts): 15-60 minutes
- California (52 districts): ~45-60 minutes

### Visualization
- State district map (DPI 150): 10-30 seconds
- National map (DPI 150): 3-5 minutes
- National map (DPI 300): 10-15 minutes (not recommended)

### Analysis
- Political analysis (per state): 5-60 seconds
- Demographic analysis (per state): 1-2 seconds
- Demographic visualization (per state): 5-30 seconds

## System Requirements

### Minimum
- Python 3.8+
- 8 GB RAM
- 20 GB disk space (includes all data)
- METIS library or executable

### Recommended
- Python 3.10+
- 16 GB RAM (for parallel processing)
- 30 GB disk space
- 4-8 CPU cores (for parallel mode)
- METIS library (faster than executable)

## Known Issues

### Alaska and Hawaii
- No tract-level election data available
- Political analysis will fail (expected)
- Demographic analysis works fine

### GEOID Type Mismatches
- Census tract GEOIDs lose leading zeros when stored as integers
- Always use `.astype(str).str.zfill(11)` before merging DataFrames
- See `CODING_PATTERNS.md` for details

### DPI and Performance
- DPI 300 causes 4x slowdown vs DPI 150
- DPI 150 provides excellent quality for most uses
- Only use DPI 300 for print-quality outputs

### Census API Rate Limiting
- Census API may rate limit during data downloads
- Scripts automatically retry with exponential backoff
- If persistent, wait a few minutes and re-run

## Future Work

### High Priority
- Unit tests for core algorithms
- Integration tests for full pipeline
- Error handling improvements
- Validation against actual districts

### Medium Priority
- Multi-member district support
- Alternative compactness measures
- Interactive visualization
- Web interface

### Low Priority
- Historical census data (2000, 1990)
- Block-level redistricting option
- Custom adjacency rules
- Export to other formats (KML, TopoJSON)

## Contributing

See `CONTRIBUTING.md` for development guidelines.

## Documentation

- `README.md` - Project overview and quickstart
- `ARCHITECTURE.md` - System design and algorithms
- `CODING_PATTERNS.md` - Coding conventions and patterns
- `DEPENDENCIES.md` - Installation and requirements
- `scripts/*/README.md` - Directory-specific documentation

## License

This project is for educational and research purposes.
