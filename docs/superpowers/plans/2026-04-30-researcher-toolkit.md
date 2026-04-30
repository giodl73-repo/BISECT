# Plan: Researcher Toolkit

> **For agentic workers:** Do not execute until the spec at `docs/superpowers/specs/2026-04-30-researcher-toolkit.md` v2 (with v2.1 patches) has been reviewed and approved. Steps use checkbox (`- [ ]`) syntax for tracking.

**Date:** 2026-04-30
**Spec:** `docs/superpowers/specs/2026-04-30-researcher-toolkit.md`
**v2.1 tracking ref:** `docs/superpowers/specs/2026-04-30-v21-tracking.md`
**Goal:** Package the existing redist engine for academic researchers: 5 Jupyter notebooks against the `redist_py` PyO3 wheel, an MCMC ensemble wrapper around GerryChain, an AEA-compliant `--paper-mode` replication-package emitter, and convergence diagnostics that meet field-standard rigor (R-hat across ≥4 chains, ESS on summary statistics, partition-distance autocorrelation).

**Depends on:** spec v2.1 approval; existing `redist_py` PyO3 wheel (built via `maturin` from `redist/python/redist_py/`); existing `redist export --format gerrychain` and `redist import --format gerrychain` (already shipped). No blocking code dependencies; the new CLI lives under a new `redist research` subcommand group and a new `--paper-mode` flag on `redist analyze`.
**Blocks:** none directly. The Researcher Toolkit is the last persona on the roadmap, so subsequent work is maintenance-mode.

**v2.1 items addressed by this plan:**
- M-01 (rename `--label` overload — apply to `validate-ensemble` as `--plan-label` / `--ensemble-label`)
- M-02 (rename `validate-against-ensemble` -> `redist research validate-ensemble`)
- D-05 (AEA REPRODUCE.sh: declare *target* platform Linux x86_64 glibc 2.35; pin Cargo.lock SHA + rust-toolchain.toml SHA + requirements.lock via uv/pip-tools)
- S-03 (ensemble diagnostics: ≥4 parallel chains for Gelman-Rubin R-hat <1.05; ESS on summary statistics not partitions; partition-distance autocorrelation Hamming)
- B-06 (kernel-state attestation: pin to compatible *range*, not single version)
- B-09 (synthetic ensemble diagnostics reframed as implementation-correctness; separate nightly N≥10k statistical-anchor test)

---

## Pre-Conditions

- `redist_py` builds cleanly via `maturin develop --release` from `redist/python/redist_py/` on Windows + Linux
- `redist export --format gerrychain` produces valid GerryChain v2.3 JSON (already in `export_cmd.rs`)
- `redist import --format gerrychain` round-trips assignments (already in `import_cmd.rs`)
- A working Python 3.11+ environment with `gerrychain>=0.3.2,<0.4` installable via `requirements.lock`
- `redist analyze` exists and accepts `--label` for plan-scoped analysis (confirmed in `args.rs`)

---

## Task 1: Notebook scaffolding + runtime-budget metadata convention

**Files:** `notebooks/{01_quickstart,02_parameter_sweep,03_callais_evidence,04_gerrychain_interop,05_mcmc_ensemble}.ipynb`, `notebooks/README.md`, `notebooks/_lib/__init__.py`, `notebooks/_lib/budget.py`, `.github/workflows/notebooks.yml`

The notebooks are the load-bearing artifact for this persona; everything else either supports them (the wrapper, the diagnostics) or is referenced from them (paper-mode).

- [ ] **1.1** Create `notebooks/` directory with the 5 ipynb files. Each notebook's cell-1 metadata carries `{"runtime_budget_secs": <N>}`: 60 for `01_quickstart`, 120 for `02_parameter_sweep`, 300 for `03_callais_evidence`, 120 for `04_gerrychain_interop`, 1800 for `05_mcmc_ensemble`.
- [ ] **1.2** Cell 2 of every notebook is a standardized header importing `redist_py`, `gerrychain`, asserting versions against compatible ranges (Task 2), and printing the runtime budget so it's visible on render.
- [ ] **1.3** Add `notebooks/_lib/budget.py` exposing `assert_versions(redist_py_range, gerrychain_range)` that raises `RuntimeError` with the spec's actionable message on mismatch. Notebooks call this from cell 2.
- [ ] **1.4** Write `notebooks/README.md` documenting: how to launch (`maturin develop --release && jupyter lab notebooks/`), the runtime-budget convention, the kernel-state attestation rule, the GerryChain version pinning policy, and the link to `redist research check-compat`.
- [ ] **1.5** Add `.github/workflows/notebooks.yml`: PR job runs notebooks 01-04 with `nbconvert --execute --ExecutePreprocessor.allow_errors=False`, enforcing budget * 1.5 timeout per the spec's CI YAML. Notebook 05 is gated behind a `nightly` label (not on PR) because of the 1800s budget.
- [ ] **1.6** Each notebook ends with the sanity output `notebook completed within budget`.

**Exit:** All 5 notebooks render in JupyterLab from a clean checkout; CI executes 01-04 inside their declared budgets; nightly executes 05.

---

## Task 2: Kernel-state attestation cell + compatible-range pinning (B-06)

**Files:** `notebooks/_lib/attestation.py`, final cell of every `notebooks/*.ipynb`, `tests/research/test_attestation.py`

The v2.1 patch reframes attestation from a single-version pin (which causes false-positive churn every wheel rev) to a *compatible range*, so the cell asserts the version satisfies the spec's pinned range AND emits the actual versions for the manifest.

- [ ] **2.1** Implement `notebooks/_lib/attestation.py::attest_kernel_state(redist_py_range='>=0.4,<0.5', gerrychain_range='>=0.3.2,<0.4')`. It uses `packaging.specifiers.SpecifierSet` to validate ranges, hashes the notebook's input cells (read from the running notebook via `IPython.get_ipython().history_manager`), and returns a dict suitable for JSON-emit at the end of every notebook.
- [ ] **2.2** Final cell of every notebook: `attestation = attest_kernel_state(); display(attestation); assert attestation['compatible']`. The cell saves the dict to `outputs/notebooks/{notebook_name}/attestation.json` for cross-reference from paper-mode.
- [ ] **2.3** Document the range-vs-pin policy in `notebooks/README.md`: ranges follow PEP 440, change requires a spec amendment, the attestation is *advisory* on mismatch within range and *fatal* on out-of-range.
- [ ] **2.4** L0 unit test in `tests/research/test_attestation.py`: monkeypatch `redist_py.__version__` to test in-range PASS, in-range different-patch PASS, out-of-range FAIL with the spec's actionable error string. Cover `gerrychain` not installed -> graceful degradation (PASS with `gerrychain_present=False`).
- [ ] **2.5** Drift test: a notebook fixture with mismatched recorded vs. live input-cell hash fails the attestation with `notebook_source_drift=True` set in the dict.

**Exit:** Out-of-range version raises with the spec's wording; in-range patch revs do NOT cause false-positive failures; B-06 closed.

---

## Task 3: GerryChain handshake — `redist research check-compat`

**Files:** `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/research.rs` (new), `redist/crates/redist-cli/src/main.rs`, `scripts/research/check_compat.py`

The handshake is the user-visible "is my environment OK?" command. It runs an actual round-trip against the user's installed GerryChain rather than just version-string matching.

- [ ] **3.1** Add a `Research` subcommand group to `args.rs` (`Subcommand::Research(ResearchArgs)`) with subcommands `check-compat` and `validate-ensemble` (Task 5). The dispatch lives in `redist-cli/src/research.rs`.
- [ ] **3.2** `research.rs::run_check_compat()` shells out to `python -m scripts.research.check_compat` (the Python script holds the GerryChain logic; the CLI is the user-facing wrapper). On `python` not found, exit with `[CONFIG]` category error per `error-conventions.md`.
- [ ] **3.3** `scripts/research/check_compat.py` performs: (a) version match against the pinned range; (b) round-trip on a synthetic 10-node graph (`gerrychain.Partition` -> `redist_py` -> `gerrychain.Partition`) asserting assignment equality; (c) prints PASS/FAIL with diff. Exit code 0 on PASS, 1 on FAIL.
- [ ] **3.4** L0 unit test in `tests/research/test_check_compat.py`: synthetic graph + monkeypatched gerrychain stub asserts the round-trip path; full pass + deliberately-tampered intermediate (assignment mutated) asserts FAIL.
- [ ] **3.5** Document `redist research check-compat` in `docs/REDIST_CLI.md`.

**Exit:** `redist research check-compat` runs end-to-end on a clean environment with `pip install gerrychain==0.3.5` and reports PASS.

---

## Task 4: GerryChain ↔ redist round-trip property test

**Files:** `tests/research/test_gerrychain_roundtrip.py`, `notebooks/04_gerrychain_interop.ipynb`

The round-trip is the technical guarantee that lets us claim interop. Property tests under `hypothesis` ensure we cover the message space, not just one example.

- [ ] **4.1** Property test using `hypothesis.strategies.graphs`: generate small connected graphs (5-30 nodes), partition into 2-4 districts, encode via `gerrychain.Partition.to_json()`, import via `redist import --format gerrychain`, re-export via `redist export --format gerrychain`, assert the assignments match modulo the GEOID encoding documented in the spec ("Map GerryChain's node IDs to our GEOIDs").
- [ ] **4.2** Mark property tests `@pytest.mark.slow` if hypothesis budget exceeds 30s; PR runs a deterministic-seeded subset, nightly runs the full property surface.
- [ ] **4.3** Notebook `04_gerrychain_interop.ipynb` walks a small VT subgraph through the round-trip with explanatory markdown; the notebook's L1 test (Task 1.5) asserts the round-trip cell output contains "round-trip OK".
- [ ] **4.4** Document the GEOID-mapping rule in `docs/file-formats/gerrychain-interop.md` (one-shot doc; reference from `04_gerrychain_interop.ipynb`).

**Exit:** Property tests pass on a 30-graph hypothesis sweep with no shrunken counterexamples; notebook 04 demonstrates a successful round-trip on a real VT subgraph.

---

## Task 5: `redist research validate-ensemble` (renamed M-02)

**Files:** `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/research.rs`, `scripts/research/validate_ensemble.py`, `tests/research/test_validate_ensemble.py`

The renamed command (formerly `validate-against-ensemble`) compares one plan against an ensemble of N random plans and reports per-metric percentile + plain-English flag.

- [ ] **5.1** Rename per M-02: `Subcommand::Research(ResearchArgs)` exposes `validate-ensemble` (NOT `validate-against-ensemble`). Apply M-01 simultaneously: flags are `--plan-label <LABEL>` and `--ensemble-label <LABEL>` (NOT bare `--label`); accept `--ensemble-dir <DIR>` as alternate path-based input.
- [ ] **5.2** Implementation in `scripts/research/validate_ensemble.py` (Python — uses `redist_py` for analysis, pandas for percentile math). Loads target plan + ensemble plan JSONs, computes efficiency gap, mean-median, MM count, and Polsby-Popper mean per plan, then computes the target's percentile per metric.
- [ ] **5.3** Output: `outputs/{version}/ensembles/{ensemble_label}/target_plan_percentiles.json` per the spec's directory layout. Plain-English flag string follows spec wording: `"Plan ranks at the 99th percentile for efficiency gap; visible-outlier candidate"`.
- [ ] **5.4** L0 unit test: 100 synthetic ensemble plans + 1 target with hand-computed percentiles; assert `redist research validate-ensemble` percentile output matches within 0.5 percentile points (per spec DoD).
- [ ] **5.5** Refuse-to-validate guard: if the ensemble directory lacks `diagnostics/{ess.json, burn_in.json, acceptance_rates.csv, rhat.json, hamming_autocorr.json}` (Task 7), exit non-zero with `[INPUT]` category error: "ensemble {label} missing convergence diagnostics; refusing to compute percentile against unfalsifiable ensemble". This wires the spec's risk-row "report tooling refuses to cite a percentile from an ensemble missing diagnostics".

**Exit:** `redist research validate-ensemble --plan-label vt_2020_target --ensemble-label vt_2020_recom_n1000` produces a percentile JSON within 0.5pp of hand-computed truth; missing-diagnostics guard fires with the spec wording.

---

## Task 6: MCMC wrapper around GerryChain in `scripts/research/`

**Files:** `scripts/research/__init__.py`, `scripts/research/mcmc_ensemble.py`, `notebooks/05_mcmc_ensemble.ipynb`, `tests/research/test_mcmc_ensemble.py`

The wrapper is a thin glue layer; GerryChain owns MCMC, redist owns analysis. Per the spec recommendation we ship (b) wrapper-first, (a) native-Rust deferred.

- [ ] **6.1** `mcmc_ensemble.py::generate_ensemble(state, year, n_steps, seed, n_chains=4, output_dir)` — runs n_chains parallel ReCom chains via `gerrychain.MarkovChain`, writes plan JSONs into `output_dir/ensemble_plans/chain_{i}/`, writes a `manifest.json` recording master seed + per-chain seeds + GerryChain version + Cargo.lock SHA + rust-toolchain.toml SHA + requirements.lock SHA (D-05 cross-reference).
- [ ] **6.2** `validate_against_ensemble(plan_label, ensemble_dir)` — thin shim around the Task 5 CLI for direct Python import from notebooks (no shell-out from inside Jupyter).
- [ ] **6.3** Per-step logging: every MCMC step records the proposal type (recom / flip), acceptance bit, current efficiency gap. Logged into `output_dir/chain_{i}/steps.parquet` for diagnostics consumption by Task 7.
- [ ] **6.4** Notebook `05_mcmc_ensemble.ipynb` walks generation + validation on a tiny ensemble (DE or VT, n_steps=200, n_chains=4) so it stays inside the 1800s nightly budget on CI hardware.
- [ ] **6.5** L1 test: `pytest -m slow` runs `generate_ensemble(state="VT", n_steps=100, n_chains=4)` end-to-end and asserts the directory layout matches the spec's `outputs/{version}/ensembles/{label}/` tree.

**Exit:** Generating a 4-chain N=100 ensemble for VT writes the canonical directory tree; notebook 05 executes within nightly budget.

---

## Task 7: Ensemble diagnostics — ≥4 chains + R-hat + Hamming + ESS-on-summary-statistics (S-03)

**Files:** `scripts/research/diagnostics.py`, `scripts/research/__init__.py`, `tests/research/test_diagnostics.py`

S-03 is the rigor-hardening for the "is this ensemble actually mixed?" question. v2 had ESS + Geweke + acceptance rates; v2.1 escalates to R-hat across ≥4 chains, clarifies ESS is on summary statistics (NOT partition labels — partition-space ESS is ill-defined), and adds Hamming-distance autocorrelation as a partition-space proxy.

- [ ] **7.1** `diagnostics.py::compute_diagnostics(ensemble_dir)` — reads `chain_{i}/steps.parquet` for i in 0..n_chains, computes:
  - **R-hat per summary statistic** (efficiency gap, MM count, mean Polsby-Popper) using the standard between-chain / within-chain variance ratio. Requires n_chains ≥ 4 (refuses with actionable error if fewer); flags any R-hat ≥ 1.05.
  - **ESS per summary statistic** (NOT per partition) using `arviz.ess` on the per-chain time series. Flag if any ESS < 100.
  - **Burn-in cut** via Geweke per chain; documented in the manifest. Auto-discard if Geweke z > 2.
  - **Acceptance rates** for population-balance proposal and compactness proposal, per chain, written to `acceptance_rates.csv`.
  - **Hamming-distance autocorrelation** of the partition trajectory: at lag k, mean over t of `hamming(partition_t, partition_{t+k}) / n_units`. Compute integrated autocorrelation time tau_int from this curve; report tau_int per chain. This is the partition-space mixing measure that ESS-on-statistics can't capture.
- [ ] **7.2** Output artifacts under `outputs/{version}/ensembles/{label}/diagnostics/`:
  - `rhat.json` (per metric)
  - `ess.json` (per metric, with explicit `{"computed_on": "summary_statistic", "metric": ...}` field per S-03)
  - `burn_in.json` (per chain)
  - `acceptance_rates.csv`
  - `hamming_autocorr.json` (per chain, with tau_int)
  - `trace.png` (multi-chain trace per metric)
- [ ] **7.3** Refuse-to-emit guard: if `n_chains < 4`, raise with `"S-03 requires >=4 parallel chains for Gelman-Rubin R-hat; got {n}"`. Document the override path (`--allow-fewer-chains` for exploratory work) and that override stamps the manifest with `diagnostics_compromised=true` so paper-mode refuses to cite it.
- [ ] **7.4** L0 implementation-correctness tests (B-09 per below): synthetic 200-step 4-chain trajectories with hand-computed R-hat, ESS, and Hamming tau_int; assert the diagnostics function reproduces the hand-computed values to numerical precision (rtol 1e-4 on R-hat, rtol 1e-2 on ESS).
- [ ] **7.5** Notebooks `03_callais_evidence.ipynb` and `05_mcmc_ensemble.ipynb` load the diagnostics JSONs and render trace plots; pass-through assertions in CI catch silent regression.

**Exit:** A 4-chain N=200 ensemble emits the full `diagnostics/` tree; R-hat, ESS, Hamming-autocorr values match hand-computed truth; n_chains<4 fails fast with the S-03 error message.

---

## Task 8: `redist analyze --paper-mode` AEA REPRODUCE.sh with target-platform pinning (D-05)

**Files:** `redist/crates/redist-cli/src/args.rs` (add `paper_mode: bool` to `AnalyzeArgs`), `redist/crates/redist-cli/src/analyze.rs`, `redist/crates/redist-cli/src/paper_mode.rs` (new), `scripts/research/paper_mode_template/{REPRODUCE.sh,README.md.tmpl,CITATION.bib.tmpl}`, `tests/research/test_paper_mode.py`

D-05 escalates the package from "soft pin" to AEA-compliant: declare *target* platform (Linux x86_64 glibc 2.35), pin Cargo.lock SHA + rust-toolchain.toml SHA, ship a `requirements.lock` produced by uv or pip-tools.

- [ ] **8.1** Add `--paper-mode` boolean to `AnalyzeArgs` in `args.rs`. When set, the analyzer's normal output is wrapped into an AEA replication package directory `outputs/{version}/{label}/paper_mode/`.
- [ ] **8.2** `paper_mode.rs::emit_replication_package()` writes:
  - `README.md` — populated from `paper_mode_template/README.md.tmpl` with: dataset list (each with citation + access date + DOI/URL), software requirements (`redist` version, `rustc` version, Python version, every dep with pinned version pulled from `requirements.lock`), step-by-step run instructions, expected output checksums.
  - `seeds.json` — master seed + per-step derivations (read from the analyze run's plan_context.rs provenance).
  - `inputs.sha256.json` — every input file's SHA-256.
  - `environment.json` — `pip freeze` output, `cargo metadata --format-version=1`, `uname -a`, glibc version (`ldd --version`), explicit field `target_platform: "linux-x86_64-glibc-2.35"` per D-05.
  - `Cargo.lock` (copy) + `cargo_lock.sha256` field in environment.json (D-05 explicit pin).
  - `rust-toolchain.toml` (copy) + `rust_toolchain_sha256` field in environment.json (D-05).
  - `requirements.lock` produced via `uv pip compile requirements.in > requirements.lock` OR `pip-compile`, whichever is on PATH; falls back to `pip freeze` if neither available, with a manifest annotation flagging the package as "lock file generated via pip freeze fallback — not guaranteed reproducible". (D-05 explicit pin.)
  - `CITATION.bib` / `CITATION.apa.txt` / `CITATION.chicago.txt` populated from `CITATION.bib.tmpl` with the user's plan label + commit SHA.
  - `REPRODUCE.sh` from `paper_mode_template/REPRODUCE.sh`. The script: (a) verifies platform == target_platform via `uname -m` + `ldd --version`, exits with actionable cross-platform note on mismatch; (b) `cargo build --release --locked` against the pinned `Cargo.lock`; (c) `pip install -r requirements.lock`; (d) re-runs the analyze invocation; (e) hashes outputs and diffs against `expected_outputs.sha256.json`; exits 0 only on byte-identical match.
- [ ] **8.3** `--paper-mode` for ensembles: cross-references Task 7's diagnostics directory; the README's "Convergence diagnostics" section is auto-populated from `diagnostics/rhat.json`, `ess.json`, `hamming_autocorr.json` (S-03 cross-link).
- [ ] **8.4** L0 paper-mode acceptance test: `redist analyze --label synthetic_vt --paper-mode` produces a `REPRODUCE.sh` whose execution from a clean Python venv reproduces the analyze headline numbers byte-identically (per spec DoD).
- [ ] **8.5** Conformance: lint the emitted README against `social-science-data-editors/template-readme` schema (where applicable; pin the linter version in `requirements.lock`).
- [ ] **8.6** Document the cross-platform contract in `docs/research/paper-mode.md`: REPRODUCE.sh is *target-pinned* to Linux x86_64 glibc 2.35, runs on macOS/Windows under WSL with a documented workaround. Reviewers on other platforms get an actionable note, not a cryptic checksum diff.

**Exit:** `redist analyze --paper-mode` produces an AEA-spec package; `bash REPRODUCE.sh` from a clean Ubuntu 22.04 container reproduces headline numbers byte-identically in ≤30 minutes on a 4-core laptop (per spec DoD).

---

## Task 9: Tests — implementation-correctness + nightly N≥10k statistical anchor (B-09)

**Files:** `tests/research/test_diagnostics_correctness.py`, `tests/research/test_diagnostics_anchor_nightly.py`, `.github/workflows/nightly.yml`

B-09 reframes the v2 spec's ensemble-diagnostic tests as *implementation-correctness* (synthetic, hand-computed, fast — every PR) and adds a *separate* nightly statistical-anchor test (N≥10k, real generation, asserts statistical properties hold).

- [ ] **9.1** PR-tier implementation-correctness suite (`tests/research/test_diagnostics_correctness.py`): synthetic time series with closed-form R-hat / ESS / Hamming-autocorr; assert the diagnostics module computes the closed-form value to numerical precision. Runs in <30s. NO real GerryChain calls.
- [ ] **9.2** Nightly statistical-anchor test (`tests/research/test_diagnostics_anchor_nightly.py`): generate a real DE ensemble at N=10000 (4 chains, 2500 steps each), assert R-hat <1.05 across all summary metrics, assert ESS ≥1000 per metric, assert Hamming tau_int falls within a documented range. Marked `@pytest.mark.nightly`; default-skipped on PR; runs on `ubuntu-latest-large` in nightly CI.
- [ ] **9.3** Document the two tiers in `tests/research/README.md`: PR test = "diagnostics function is correctly implemented"; nightly test = "diagnostics on a real ensemble of realistic size show convergence". Different failure modes, different remediations.
- [ ] **9.4** Wire nightly into `.github/workflows/nightly.yml` alongside the existing onboarding-acceptance nightly; runtime budget 90 minutes; alert-on-fail to the spec owner.
- [ ] **9.5** L1 runtime-budget enforcement test (cross-link to spec line "deliberately-slowed notebook with budget=60 fails CI"): a fixture notebook with `time.sleep(90)` + `runtime_budget_secs: 60` is asserted to fail the CI workflow with the budget-exceeded message.

**Exit:** PR pipeline runs the correctness suite in <30s; nightly runs the N=10000 anchor test and asserts statistical properties; B-09 closed.

---

## Task 10: Docs

**Files:** `docs/research/{toolkit-overview,paper-mode,ensemble-diagnostics,gerrychain-interop}.md`, `docs/REDIST_CLI.md`, `docs/CHANGELOG.md`, `CLAUDE.md`, `notebooks/README.md`

- [ ] **10.1** `docs/research/toolkit-overview.md` — entry point for the persona. Lists the 5 notebooks with their purpose + runtime budgets, the wrapper, the diagnostics, the paper-mode flag. Links to `quickstart-researcher.md` (already shipped via Onboarding plan).
- [ ] **10.2** `docs/research/paper-mode.md` — the AEA contract: target-platform pin, the three required SHA fields (Cargo.lock, rust-toolchain.toml, requirements.lock), the REPRODUCE.sh contract, the cross-platform note for non-Linux reviewers (D-05 cross-reference).
- [ ] **10.3** `docs/research/ensemble-diagnostics.md` — the convergence-diagnostic contract: ≥4 chains, R-hat <1.05 threshold, ESS-on-summary-statistics rationale, Hamming-autocorrelation tau_int interpretation. Documents the refuse-to-cite policy (S-03 cross-reference).
- [ ] **10.4** `docs/research/gerrychain-interop.md` — already cited from Task 4.4; documents the GEOID-mapping rule and the round-trip property.
- [ ] **10.5** `docs/REDIST_CLI.md` — add `redist research check-compat`, `redist research validate-ensemble`, `redist analyze --paper-mode` to the command reference.
- [ ] **10.6** `docs/CHANGELOG.md` entry.
- [ ] **10.7** `CLAUDE.md` "Recent Changes" section gets a researcher-toolkit entry pointing at `docs/research/toolkit-overview.md`.

**Exit:** Researcher-persona docs cross-wired; REDIST_CLI.md and CHANGELOG.md updated; CLAUDE.md surfaces the toolkit.

---

## Definition of Done

- All 5 notebooks under `notebooks/` execute end-to-end within their declared `runtime_budget_secs`; CI runs 01-04 on PR and 05 nightly
- Kernel-state attestation cell in every notebook validates `redist_py` and `gerrychain` versions against compatible *ranges*, not single versions (B-06)
- `redist research check-compat` runs the round-trip property test against the user's installed GerryChain and reports PASS/FAIL
- GerryChain ↔ redist round-trip property tests pass on a 30-graph hypothesis sweep
- `redist research validate-ensemble` (renamed from `validate-against-ensemble`, M-02) produces percentile output within 0.5 pp of hand-computed truth and refuses ensembles missing diagnostics
- `scripts/research/mcmc_ensemble.py` generates ≥4-chain GerryChain ensembles into the spec's directory layout
- Diagnostics emit `rhat.json`, `ess.json` (with `computed_on=summary_statistic`), `burn_in.json`, `acceptance_rates.csv`, `hamming_autocorr.json`, `trace.png` (S-03)
- `redist analyze --paper-mode` produces an AEA-compliant package with `target_platform=linux-x86_64-glibc-2.35`, `cargo_lock_sha256`, `rust_toolchain_sha256`, and `requirements.lock` (D-05)
- `bash REPRODUCE.sh` from clean Ubuntu 22.04 reproduces headline numbers byte-identically in ≤30 minutes on a 4-core laptop
- PR pipeline runs implementation-correctness diagnostic tests in <30s; nightly runs N≥10000 statistical-anchor test (B-09)
- `docs/research/{toolkit-overview,paper-mode,ensemble-diagnostics,gerrychain-interop}.md` exist and lint-clean

---

## Risks

| Risk | Mitigation |
|---|---|
| GerryChain API changes between pinning and shipping | Pin to compatible *range* (B-06); `redist research check-compat` runs round-trip on the user's installed version; doc-pin the change-policy in `notebooks/README.md` |
| MCMC ensemble generation is slow for large states (AL, TX) | Document realistic times; offer `--max-time-secs` cap on the wrapper; runtime-budget per notebook (Task 1); the nightly N=10000 test runs on DE not AL |
| Researchers want native Rust MCMC for speed | Ship Python wrapper first per spec recommendation (b); native-Rust deferred to a future spec |
| Notebook examples drift from CLI surface | Auto-execute notebooks in CI (Task 1.5); kernel-state attestation cell (Task 2) catches output-only-vs-source drift |
| Ensemble percentiles published without convergence diagnostics | Mandatory `diagnostics/` artifact; `validate-ensemble` refuses missing diagnostics with `[INPUT]` error (Task 5.5); `--paper-mode` README auto-cites diagnostics (Task 8.3); `--allow-fewer-chains` override stamps manifest with `diagnostics_compromised=true` |
| AEA replication package goes stale between paper acceptance and final publication | Nightly statistical-anchor test (Task 9.2) re-runs the canonical small-ensemble; `REPRODUCE.sh` is re-run nightly against pinned input set; staleness alerts before review |
| `requirements.lock` produced by pip-freeze fallback claims false reproducibility | Lock file generated via fallback is annotated in the manifest with `lock_method="pip_freeze_fallback"`; paper-mode README surfaces this annotation prominently |
| Cross-platform reviewers (macOS, Windows) can't run REPRODUCE.sh | `REPRODUCE.sh` exits early with an actionable WSL/Linux-VM note when `uname` doesn't match the target platform; `docs/research/paper-mode.md` documents the workaround (Task 8.6) |

---

## Out of Scope

- Native Rust MCMC implementation (deferred per spec; revisit if user demand surfaces)
- Custom MCMC variants beyond ReCom (balanced-tree, etc. — leave to academic literature)
- High-level statistical inference helpers (researchers do this in R/pandas)
- Survey-design tools for fairness measurement (separate research category)
- Notebooks on languages other than Python (no R notebooks; reach via `reticulate` is the user's responsibility)
- Cross-platform native REPRODUCE.bat for Windows (D-05 explicitly target-pins Linux; macOS/Windows reviewers run under WSL/VM per `paper-mode.md`)
