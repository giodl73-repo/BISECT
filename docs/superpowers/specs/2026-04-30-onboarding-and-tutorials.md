# Onboarding & Tutorials: 5-Minute Quickstart
**Date:** 2026-04-30
**Updated:** 2026-04-30 (v2 — incorporates SURVEY/DATUM/COVENANT/TRENCH/BENCHMARK revisions)
**Status:** Revised; pending re-review
**Closes gap for:** all personas
**Depends on:** existing CLI surface
**Estimated effort:** 3-4 days (v2: pin tutorial data, real smoke test)

## Why this exists

A user today goes from `git clone` to first useful run in ~30 minutes (install Rust, METIS, build with cargo, possibly maturin, possibly Dataverse API key). That's too long for a practitioner deciding whether to adopt the tool. Every other capability spec lands on users who can't get past the first step.

This spec gets onboarding to ≤ 5 minutes for the happy path and provides one worked example per persona category.

## Scope

### In scope

1. **Bootstrap script** (`bootstrap.sh` + `bootstrap.bat`)
   - Detects the platform
   - Installs `rustup` if missing
   - Installs the pinned rustc via `rust-toolchain.toml`
   - Builds `redist` in release mode
   - **Verifies the build produced the binary at the expected path** before mutating PATH (TRENCH PP-18 prevention)
   - Adds `redist/target/release` to PATH; verifies the next shell can find the binary via `which redist` / `where redist`
   - Optionally installs maturin + builds the `redist_py` wheel; verifies via `python -c "import redist_py"` before claiming success
   - Optionally prompts for `DATAVERSE_API_KEY`. **Validates the key with one Dataverse API round-trip** before writing it to local config (TRENCH PP-19 prevention). Writes to `~/.config/redist/credentials.toml` (Linux/macOS) or `%APPDATA%\redist\credentials.toml` (Windows) — explicit path, not hidden in a project dir.
   - **Runs a real smoke test, not `--print-only`** (TRENCH PP-20): `redist state --state VT --year 2020 --label bootstrap_test --output-dir /tmp/bootstrap_smoke`. Verifies the bisection actually completed and final_assignments.json has 193 tract entries.
   - Reports success with next-step pointers

2. **Persona-specific quickstart docs** under `docs/quickstart/`:
   - `quickstart-special-master.md` — verify a submitted plan against its manifest
   - `quickstart-researcher.md` — run a parameter sweep + analyze
   - `quickstart-callais-expert.md` — full §2 evidence kit for one state
   - `quickstart-state-staff.md` — import from Districtr, validate, export back
   - `quickstart-civic-advocate.md` — produce a comparison narrative for the public

3. **End-to-end Vermont walkthrough** (the canonical "complete example", **changed from Louisiana per SURVEY**)
   - Vermont is the canonical because: 1 district = trivial population balance, fully-public Fekrazad data, no live §2 litigation that might color the project as advocacy-aligned to a special master. Louisiana remains as a separately-documented "advanced post-Callais walkthrough".
   - Fetch Census + adjacency + election data
   - Run `redist state --state VT --year 2020`
   - Run `redist analyze --types all`
   - Generate the PDF expert report
   - Verify with `redist doctor --verify-manifest`
   - Each step shows expected output + how long it should take

3a. **Tutorial data versioning (DATUM/COVENANT/TRENCH consensus)**
    - Walkthrough pins:
      - Fekrazad DOI version
      - Census TIGER year + URL
      - Expected output checksums for each step (`final_assignments.json`, `district_summary.csv`)
    - New CLI: `redist doctor --check-tutorial-data` validates that the user's local data matches the pinned versions (by SHA-256). If the upstream changed, the user gets an actionable warning, not a silent drift.
    - Tutorial output blocks show:
      ```
      Expected: final_assignments.json sha256 = abc123...
      Got:      def456...
      → Data has drifted from tutorial baseline. Either:
         (a) re-fetch with the pinned commands above
         (b) Run `redist doctor --check-tutorial-data` for diagnosis
      ```

3b. **Advanced Louisiana / Callais walkthrough** (separate doc; not the canonical example)
    - Fetch primary-election data via the registry
    - Run `redist analyze --types bloc-voting` with the Callais Evidence Layer
    - Generate court-formatted PDF
    - Explicitly framed as "the post-Callais §2 evidence kit" not as the project's default workflow.

4. **Error-message audit**
   - Walk every `unwrap()` and `?` in the CLI binary's user-facing surface
   - Replace stack traces with actionable hints
   - Categorize errors: `[INPUT]`, `[CONFIG]`, `[INTERNAL]`, `[NETWORK]`
   - Document the categorization in `docs/error-conventions.md`

5. **README.md rewrite**
   - First 30 seconds: what is this and who's it for (persona table)
   - Next 5 minutes: quickstart link tree
   - Architecture diagram (one image)
   - Link to deeper docs (REDIST_CLI.md, architecture spec, etc.)

### Out of scope

- Video walkthroughs (separate work, requires recording infrastructure)
- Localized docs (English-only is fine for the redistricting space)
- Interactive web tutorial (Districtr-style; that's the State Staff Interop spec)
- Per-state walkthroughs beyond Louisiana (one canonical example is enough)

## Implementation notes

### Bootstrap script structure

```bash
# bootstrap.sh (Linux/macOS)
#!/usr/bin/env bash
set -euo pipefail

step() { echo; echo "[step $1] $2"; }

step 1 "Checking rustup..."
if ! command -v rustup >/dev/null 2>&1; then
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain none
    source "$HOME/.cargo/env"
fi

step 2 "Installing pinned toolchain..."
cd redist && rustup show

step 3 "Building redist (release)..."
cargo build --release --locked

step 4 "Adding to PATH..."
# ... (platform-specific shell config)

step 5 "Smoke test..."
./target/release/redist state --state VT --year 2020 --label bootstrap_test --print-only

step 6 "Done!"
echo "Try: redist state --state VT --year 2020"
```

Plus `bootstrap.bat` mirroring for Windows.

### Quickstart doc structure (template)

Each `quickstart-PERSONA.md` follows:
- **Who you are** (1 sentence)
- **What you'll have at the end** (1 sentence)
- **Time** (target: 5-15 minutes)
- **Steps** (numbered, 3-7 steps max)
- **Expected output at each step** (so users know if something's wrong)
- **Where to go next** (deeper docs)

### Louisiana walkthrough is the primary tutorial

It's the full Callais story: fetch data, run constraint-aware bisection, run §2 analysis, generate court-ready PDF, verify provenance. Every other quickstart can reference sections of this walkthrough.

## Tests (v2 per BENCHMARK)

- L0: bootstrap script syntax check (bash + cmd parser)
- L1: docs/quickstart/* renders cleanly (markdown lint)
- L2: separately-runnable acceptance test (NOT doctest — long commands, network, etc.) at `tests/acceptance/test_walkthrough_vermont.py` that runs the canonical walkthrough end-to-end and asserts all expected output checksums. Marked `@pytest.mark.network` and `@pytest.mark.slow`; default-skipped.
- Manual: full bootstrap from a clean Windows VM + clean Linux container; record actual time-to-first-run

Doctest of walkthrough markdown was considered (BENCHMARK noted) but rejected: long commands cause CI hangs, network dependency, no reasonable mock surface. The L2 acceptance test is the right level.

## Risks

| Risk | Mitigation |
|---|---|
| Bootstrap-script maintenance burden across platforms | Keep it minimal; rely on `cargo` and `rustup` which are themselves cross-platform |
| Walkthroughs go stale as the CLI surface evolves | Auto-test the walkthrough commands (`pytest --doctest-glob='quickstart-*.md'`) |
| "5 minutes" is aspirational for slow connections / first-time Rust users | Document realistic timings; show progress bars for build steps |
| Error-message audit is open-ended (could go forever) | Bound to a list — every CLI subcommand's main entry path |

## Definition of done

- `bootstrap.sh` + `bootstrap.bat` succeed on a clean VM in ≤ 10 minutes wall-clock
- `quickstart-*.md` exists for all 5 personas; each tested end-to-end manually
- Louisiana walkthrough committed under `examples/louisiana-callais-walkthrough/` with input data references
- README.md leads with the persona table; link tree has no broken links
- Error-message audit covers every CLI subcommand main path; documented in error-conventions.md
