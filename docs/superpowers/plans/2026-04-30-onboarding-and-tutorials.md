# Plan: Onboarding & Tutorials (≤ 5-Minute Quickstart)

> **For agentic workers:** Do not execute until the spec at `docs/superpowers/specs/2026-04-30-onboarding-and-tutorials.md` v2 (with v2.1 patches) has been reviewed and approved. Steps use checkbox (`- [ ]`) syntax for tracking.

**Date:** 2026-04-30
**Spec:** `docs/superpowers/specs/2026-04-30-onboarding-and-tutorials.md`
**v2.1 tracking ref:** `docs/superpowers/specs/2026-04-30-v21-tracking.md`
**Goal:** Get bootstrap-to-first-useful-run to ≤ 5 minutes for the happy path on a clean Windows VM and a clean Linux container, with one worked quickstart per persona and an L2 acceptance test that runs the canonical Vermont walkthrough end-to-end with pinned data checksums.

**Depends on:** spec v2.1 approval. No code dependencies (existing `redist` CLI surface is sufficient).
**Blocks:** none directly. Other plans cite quickstart paths but do not require them to ship first.

**v2.1 items addressed by this plan:**
- D-02 (commit `examples/vermont-walkthrough/checksums.json`)
- B-01 fixture path / SUR-01 (already patched in spec; this plan delivers the artifacts)
- CM-01 (`quickstart-civic-advocate.md` documents obtaining the state's plan when not Districtr-published)
- PP-34 (Unicode/CP1252 console policy line in `docs/error-conventions.md`)

---

## Pre-Conditions

- `redist` binary builds clean on Windows + Linux from `cargo build --release --locked`
- Census 2020 TIGER data for VT is fetchable via existing `redist fetch` infrastructure
- Fekrazad DOI 10.7910/DVN/Z8TSH3 is reachable (verified by network-marked integration tests)
- `redist doctor` exists and supports `--verify-manifest`; this plan extends it with `--check-tutorial-data`

---

## Task 1: Author the Vermont canonical walkthrough fixture

**Files:** `examples/vermont-walkthrough/{README.md, run.sh, checksums.json, expected_outputs/}`

The walkthrough is the load-bearing artifact: every quickstart links to it, the L2 acceptance test runs it, and `redist doctor --check-tutorial-data` verifies it.

- [ ] **1.1** Run the full canonical pipeline once locally on VT 2020 with the pinned Census TIGER URL + Fekrazad DOI; capture every command, every output checksum, every wall-clock time.
- [ ] **1.2** Write `examples/vermont-walkthrough/run.sh` (and a `run.bat` equivalent) executing the pinned commands in order. No interactive prompts; everything declarative.
- [ ] **1.3** Generate `examples/vermont-walkthrough/checksums.json` with the schema:
  ```json
  {
    "schema_version": "tutorial-checksums v1",
    "tutorial": "vermont-2020",
    "pinned_inputs": {
      "census_tiger_url": "...", "census_tiger_sha256": "...",
      "fekrazad_doi": "10.7910/DVN/Z8TSH3", "fekrazad_file_sha256": "..."
    },
    "expected_outputs": {
      "final_assignments.json": {"sha256": "...", "tract_count": 193},
      "district_summary.csv": {"sha256": "..."},
      "analysis/partisan.json": {"sha256": "..."}
    },
    "build_commit": "<commit-sha-at-pinning>"
  }
  ```
- [ ] **1.4** `examples/vermont-walkthrough/README.md` explains: who this is for, how to run it, what the expected timings are (per step, per total), how to interpret a checksum mismatch, how to re-pin if the upstream changes.

**Exit:** A second machine running `bash examples/vermont-walkthrough/run.sh` produces output whose checksums match `checksums.json` byte-for-byte.

---

## Task 2: Implement `redist doctor --check-tutorial-data`

**Files:** `redist/crates/redist-cli/src/doctor.rs`, `redist/crates/redist-cli/src/main.rs`

- [ ] **2.1** Extend `redist doctor` with the `--check-tutorial-data` flag accepting an optional `--tutorial <NAME>` (default: `vermont-2020`).
- [ ] **2.2** The command reads `examples/{tutorial}-walkthrough/checksums.json`, hashes each pinned input + expected output that exists locally, reports per-row PASS/FAIL/MISSING, exits 0 only if all PASS.
- [ ] **2.3** On any FAIL, print the actionable message from the spec ("Data has drifted from tutorial baseline. Either: (a) re-fetch with the pinned commands above; (b) Run `redist doctor --check-tutorial-data` for diagnosis").
- [ ] **2.4** L0 unit tests: synthetic 3-row checksums file with one PASS / one FAIL / one MISSING; assert exit code + per-row output.

**Exit:** `redist doctor --check-tutorial-data` correctly identifies drift; tests cover all three row outcomes.

---

## Task 3: Bootstrap script (Linux/macOS)

**Files:** `bootstrap.sh`

The script implements the spec's §"Bootstrap script structure" with the v2.1 PP-20 fix (real smoke test, not `--print-only`).

- [ ] **3.1** Write `bootstrap.sh` per the v2.1 spec. The smoke test asserts `final_assignments.json` exists AND `jq 'length' ... -eq 193`. ASCII-only output (`[OK]`, `[FAIL]`, `->`).
- [ ] **3.2** PATH preflight (PP-18): after `cargo build`, verify `target/release/redist` exists at the expected path BEFORE adding to PATH. After PATH update, verify `which redist` finds it. If not, exit non-zero with a clear "build succeeded but binary not at expected path" message.
- [ ] **3.3** Optional Python wheel build via maturin: gated by `--with-python` flag. After build, verify `python -c "import redist_py"` succeeds before claiming success.
- [ ] **3.4** Optional Dataverse API key prompt: gated by `--with-api-key` flag. PP-19 requires validating the key with one Dataverse round-trip BEFORE writing to `~/.config/redist/credentials.toml`. On validation failure, do NOT write the key; print the actionable error from Dataverse.
- [ ] **3.5** Run smoke test (Task 1's `run.sh` is the canonical smoke; bootstrap.sh runs an abbreviated version: `redist state --state VT --year 2020 --label bootstrap_test --output-dir /tmp/bootstrap_smoke` and asserts the tract count).
- [ ] **3.6** Print final summary: time elapsed, what was installed, what to try next.

**Exit:** On a clean Ubuntu 22.04 container, `bash bootstrap.sh` completes in ≤ 10 minutes wall-clock and the smoke test passes.

---

## Task 4: Bootstrap script (Windows)

**Files:** `bootstrap.bat`

- [ ] **4.1** Mirror Task 3 in cmd.exe / PowerShell-bridge syntax. Use `where redist` instead of `which redist`. Use `%TEMP%\bootstrap_smoke` instead of `/tmp/bootstrap_smoke`. ASCII-only output (CP1252 console).
- [ ] **4.2** Credentials path: `%APPDATA%\redist\credentials.toml`. Same PP-19 validation before write.
- [ ] **4.3** Test on a clean Windows 11 VM (no Rust, no Python, no Visual Studio Build Tools — bootstrap should detect missing build tools and surface the install URL, not fail with a cryptic linker error).

**Exit:** On a clean Windows 11 VM, `bootstrap.bat` completes in ≤ 10 minutes wall-clock and the smoke test passes.

---

## Task 5: Per-persona quickstart docs

**Files:** `docs/quickstart/{quickstart-special-master, quickstart-researcher, quickstart-callais-expert, quickstart-state-staff, quickstart-civic-advocate}.md`

Each follows the spec's template (Who you are / What you'll have / Time / Steps / Expected output / Where to go next).

- [ ] **5.1** `quickstart-special-master.md` — verify a submitted plan against its manifest using `redist doctor --verify-manifest`. Use the Vermont walkthrough's output as the example plan.
- [ ] **5.2** `quickstart-researcher.md` — run a parameter sweep + analyze. Cross-references the (future) Researcher Toolkit notebooks but ships a CLI-only path.
- [ ] **5.3** `quickstart-callais-expert.md` — full §2 evidence kit; explicitly framed as advanced; references the Louisiana walkthrough fixture.
- [ ] **5.4** `quickstart-state-staff.md` — Districtr → `redist import` → `redist analyze` → `redist report` round trip.
- [ ] **5.5** `quickstart-civic-advocate.md` — produce a comparison narrative for the public.
  - **CM-01**: include explicit guidance on obtaining the state's official plan when not Districtr-published — shapefile path (`redist import --format shapefile`), GeoJSON path, and "if all you have is a PDF, here's how to ask the state for a machine-readable export" template language.
- [ ] **5.6** Each quickstart manually walked end-to-end on a clean machine; record actual wall-clock time at the top.

**Exit:** All 5 quickstarts exist, render cleanly via markdown lint, walk end-to-end manually.

---

## Task 6: Error-message audit + `docs/error-conventions.md`

**Files:** `docs/error-conventions.md`, `redist/crates/redist-cli/src/**/*.rs`

- [ ] **6.1** Walk every `unwrap()`, `expect()`, and bare `?` in `redist-cli/src/`. For each user-facing path (top-of-main return paths, CLI argument parsing, file I/O on user-supplied paths), replace stack-trace exits with `[INPUT]` / `[CONFIG]` / `[INTERNAL]` / `[NETWORK]` categorized errors that include an actionable hint.
- [ ] **6.2** Document the four categories in `docs/error-conventions.md`: when each is appropriate, what an actionable hint looks like, examples per category.
- [ ] **6.3** **PP-34**: add the line "All CLI stdout/stderr is ASCII-only on Windows; Unicode is permitted in file outputs only" to `error-conventions.md` with a one-paragraph rationale (CP1252 console crash; real Unicode `→` bug history).
- [ ] **6.4** L0 unit tests for at least 5 representative error paths: assert the category prefix + actionable hint string.

**Exit:** Every CLI subcommand's main-entry path uses categorized errors; ASCII-only console policy is documented.

---

## Task 7: README.md rewrite

**Files:** `README.md`

- [ ] **7.1** First 30 seconds: persona table mapping `What you want to do` → `Where to start` → `Time`. Five rows, one per persona.
- [ ] **7.2** Next 5 minutes: link tree to the bootstrap script + the 5 quickstarts.
- [ ] **7.3** Architecture diagram (one image — reuse the existing one if it's accurate; otherwise a 5-box block diagram is fine).
- [ ] **7.4** Link to deeper docs (REDIST_CLI.md, architecture spec, CLAUDE.md for contributors).
- [ ] **7.5** "What this is NOT" section: not a GUI; not a real-time multiplayer drawer; not a litigation predictor. Manage expectations.

**Exit:** A first-time visitor reaches the bootstrap script + their persona's quickstart in ≤ 60 seconds of reading.

---

## Task 8: L2 acceptance test

**Files:** `tests/acceptance/test_walkthrough_vermont.py`

- [ ] **8.1** Pytest test marked `@pytest.mark.network` and `@pytest.mark.slow` (default-skipped per spec).
- [ ] **8.2** The test invokes `bash examples/vermont-walkthrough/run.sh` (or shell-equivalent on Windows runners), then runs `redist doctor --check-tutorial-data`. Assert exit code 0.
- [ ] **8.3** On failure, the test prints the diff between expected and got checksums (don't just assert — surface the drift for debugging).
- [ ] **8.4** Wire into the nightly CI workflow (per roadmap §CI strategy): runs on `ubuntu-latest-large` with the `network` and `slow` markers selected.

**Exit:** Nightly CI run executes the Vermont walkthrough end-to-end and passes.

---

## Task 9: Documentation cross-wiring

**Files:** `CLAUDE.md`, `docs/REDIST_CLI.md`, `docs/CHANGELOG.md`

- [ ] **9.1** CLAUDE.md "Common Commands" section gets a "First time here?" subsection at the top pointing at `bootstrap.sh` / `bootstrap.bat`.
- [ ] **9.2** CLAUDE.md "Recent Changes" entry for this plan's completion date.
- [ ] **9.3** `docs/REDIST_CLI.md` documents `--check-tutorial-data` and the `bootstrap` flow.
- [ ] **9.4** `docs/CHANGELOG.md` entry.

**Exit:** A contributor reading CLAUDE.md cold lands on the bootstrap path.

---

## Definition of Done

- `bootstrap.sh` + `bootstrap.bat` succeed on a clean VM in ≤ 10 minutes wall-clock (verified, not aspirational)
- All 5 `quickstart-*.md` exist, lint-clean, and have been manually walked end-to-end
- Vermont walkthrough committed under `examples/vermont-walkthrough/` with `checksums.json` consumed by `redist doctor --check-tutorial-data`
- Louisiana walkthrough exists separately under `examples/louisiana-callais-walkthrough/` and is framed as "advanced post-Callais §2 evidence kit"
- `tests/acceptance/test_walkthrough_vermont.py` passes nightly
- README.md leads with the persona table; no broken links
- `docs/error-conventions.md` documents the four error categories and the ASCII-only console rule
- Every CLI subcommand main-entry path uses categorized errors

---

## Risks

| Risk | Mitigation |
|---|---|
| Census TIGER URL changes between pinning and shipping | Pin to a Wayback URL in addition to the live one; `--check-tutorial-data` surfaces the drift |
| Fekrazad DOI gets a new file revision | Pin to the exact filename SHA, not the DOI alone; document the re-pin process |
| Bootstrap on Windows 11 VM hits a missing-VS-Build-Tools failure mode we didn't anticipate | Task 4.3 explicitly tests this; escalation surface is the install URL, not a cryptic linker error |
| Quickstarts drift as the CLI surface evolves | Vermont walkthrough is the L2 acceptance test; quickstarts that share commands inherit drift detection. Quickstarts with novel commands need their own L1 smoke. |
| Time budget "5 minutes" misses on slow connections | Spec is "happy path"; document realistic timings ("5 min on broadband; 15-20 min on slow link") |

---

## Out of Scope

- Video walkthroughs (separate work)
- Localized docs (English-only)
- Interactive web tutorial (Districtr-style — that's the State Staff Interop spec)
- Per-state walkthroughs beyond VT and LA (not needed for ★★★★★)
