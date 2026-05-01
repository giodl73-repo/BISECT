# Error Conventions for `redist` CLI

**Date:** 2026-04-30
**Owner:** Onboarding plan Task 6 (`docs/superpowers/plans/2026-04-30-onboarding-and-tutorials.md`)

This document codifies the categorized error model the `redist` CLI presents to end users. It is not an internal Rust style guide — it is a contract with the people running the binary.

## Categories

Every user-facing error message starts with one of four categorized prefixes. The prefix tells the user *what kind of fix* is appropriate, before any details.

| Prefix | Meaning | Who fixes it | Example |
|---|---|---|---|
| `[INPUT]` | The user supplied bad data, a bad path, or a bad CLI flag. Fix the inputs. | The user | `[INPUT] Row 47 GEOID '1001950100' is 10 digits; tract GEOIDs are 11 digits. Excel/Sheets stripped a leading zero. Re-export the GEOID column with column-format = Text.` |
| `[CONFIG]` | A required external dependency, environment variable, file, or pinned version is missing or wrong. Fix the environment. | The user (or DevOps) | `[CONFIG] verapdf 1.x not found on PATH. Install: https://verapdf.org/software/. Required for --format pdf court-mode validation.` |
| `[NETWORK]` | A remote fetch failed, timed out, or returned an unexpected HTTP status. Probably retry; possibly fall back. | The user (or the network) | `[NETWORK] Census TIGER fetch failed (HTTP 503): https://www2.census.gov/geo/tiger/.... Retry in 30s, or fetch manually and place at outputs/v1/2020/data/tiger/.` |
| `[INTERNAL]` | A bug or unexpected state in `redist` itself. The user cannot fix it; the developer must. | The developer | `[INTERNAL] Unreachable code in bisection runner — please file an issue with the manifest at outputs/v1/2020/plans/.../manifest.json.` |

## Anatomy of a good error

Every error message MUST contain:

1. **The category prefix** in square brackets at the start of the line.
2. **A one-sentence description** of what went wrong, without jargon.
3. **An actionable hint** — the user knows what to do next without reading the source.

Examples:

```
[INPUT] CSV is UTF-16; re-export as UTF-8 (Excel: 'Save As → CSV UTF-8'). UTF-16 is not supported.
```

```
[CONFIG] redist not on PATH. Run: source ~/.cargo/env  (or re-run bootstrap.sh).
```

```
[NETWORK] Dataverse API key validation failed (HTTP 401). The key in ~/.config/redist/credentials.toml is invalid or expired. Generate a new one at https://dataverse.harvard.edu/account.
```

```
[INTERNAL] PlanManifest deserialization succeeded but population_balance_valid was unset.
   This is a bug. File at https://github.com/.../issues with: redist --version, the manifest path,
   and the cargo build commit (run: redist doctor --verify-manifest <manifest>).
```

## Anti-patterns

- **Bare stack traces** as user output. If a stack trace would be useful (it usually isn't), gate it behind `--debug` or `RUST_LOG=debug`.
- **`unwrap()` in user-facing paths** that expects success. Use `?` with a typed error, or `.with_context()` from `anyhow`.
- **Errors that name internal code paths** ("PlanContext::from_label() failed"). The user does not know what `PlanContext` is. Translate.
- **Errors that omit the path** when a file operation failed. Always include the path.

## Windows console policy (TRENCH PP-34)

**All CLI stdout/stderr is ASCII-only on Windows; Unicode is permitted in file outputs only.**

Rationale: the Windows console runs CP1252 by default. Emoji or non-ASCII characters (`✓`, `✗`, `→`, `•`, em-dashes) raise `UnicodeEncodeError` mid-run, crashing the binary with a confusing trace. We have a real bug history on this — the OpenElections fetcher hit it with `→`.

Concrete rules:

- Use `[OK]`, `[FAIL]`, `[WARN]`, `[INFO]`, `[INPUT]`, `[CONFIG]`, `[NETWORK]`, `[INTERNAL]` — not `✓`/`✗`/colored bullets.
- Use `->` for arrows, not `→`.
- Use `+/-` for plus/minus, not `±`.
- Use `--` for em-dashes, not `—`.
- Use ASCII quotes `"` `'`, not curly quotes `"` `"` `'` `'`.
- File outputs (Markdown, JSON, HTML) MAY use Unicode; they are read via UTF-8 readers, not the console.
- Narrative templates that go to file MAY use Unicode (em-dashes, etc.); tests assert that the narrative writer never funnels through `println!` on Windows.

This rule is enforced by:

1. CI grep over `redist-cli/src/**/*.rs` for non-ASCII bytes in `println!`/`eprintln!` literals (not yet automated; see Onboarding plan Task 6.4).
2. The Onboarding L2 acceptance test asserts `all(c in string.printable for c in stdout)` on the Vermont walkthrough output (v2.1.1 tracking item 211-P3.1).

## Migration plan

The full error audit (every `unwrap()`, `expect()`, and bare `?` in `redist-cli/src/`) is incremental work. The current state:

- New code (post-2026-04-30) uses categorized prefixes by default
- Existing user-facing error paths are migrated as they're touched in unrelated work
- `redist doctor`'s `--check-tutorial-data` (the first new entry point) demonstrates the conventions
- High-priority paths to migrate next (in order): `redist state` failure modes, `redist fetch` HTTP errors, `redist analyze` missing-input errors

Track migration coverage in `docs/superpowers/plans/2026-04-30-onboarding-and-tutorials.md` Task 6.

## Build provenance overrides (B-07)

Two related env-var overrides for the recorded `redist_build_commit`:

- **`REDIST_BUILD_COMMIT_OVERRIDE`** (build-time, supported in production):
  When set during `cargo build`, replaces `git rev-parse HEAD` as the recorded
  commit. Used by reproducible-build packagers pinning to a release tag.
  Verbatim — does NOT append the `-dirty` suffix (the override is the
  authoritative attestation). The build script logs a `cargo:warning=` so the
  override is visible in build logs.

- **`REDIST_BUILD_COMMIT`** (runtime, test-only): the test harness can spoof
  the running binary's commit string to exercise build-commit-mismatch
  warnings + `--enforce-build-commit` gates without rebuilding. This path is
  available only when the binary is compiled with `cfg(test)` OR a future
  `cfg(feature = "test-provenance-override")` (not currently set in release
  builds). Production binaries do NOT consult the runtime override.

When both apply: the build-time override wins (the recorded value at compile
time is locked into the binary).

## See also

- TRENCH pitfall PP-34 (`docs/superpowers/specs/2026-04-30-v21-tracking.md`)
- `docs/superpowers/plans/2026-04-30-onboarding-and-tutorials.md` Task 6
- CLAUDE.md "Common Pitfalls" #5 (Unicode rule history)
