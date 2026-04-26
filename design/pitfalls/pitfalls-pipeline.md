# Pipeline Pitfalls (PP-01..PP-07)

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

**Structural solution:** Never use a short command name for a language runtime in a compiled binary. Accept the executable path via an environment variable (`REDIST_PYTHON`, `REDIST_NODE`) that the caller must set explicitly. Document this requirement in the binary's `--help` output.

**Status:** SOLVED for Python subprocess calls in redist-cli
**Proved by:** REDIST_PYTHON env var used in adjacency_loader.rs and runner.rs; acceptance tests set it to sys.executable
**Test:** `tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance` — fixtures pass REDIST_PYTHON=sys.executable

---

## PP-05: Dual-purpose path parameter creates read/write coupling

**Pattern:** A single path parameter controls both where outputs are written and where input data is read from. In production, these are the same directory tree, so no coupling is visible. In tests or non-standard deployments where inputs and outputs live in different locations, the system either fails to find inputs or writes outputs into the input directory.

**Domain:** Any pipeline with an `--output-dir` parameter that is also used to locate data (adjacency graphs, demographics, configurations) that predates the current run. The parameter name implies "where to write" but the implementation also uses it as "where to read".

**Why it's hard to catch:** All existing tests and documentation assume inputs and outputs share a root directory. The coupling is never visible until someone needs to read from a read-only data store or write to a separate location.

**Structural solution:** Separate read paths from write paths explicitly. Output data goes to `--output-dir`. Input data (adjacency graphs, TIGER files, demographics) is looked up from `outputs/{version}/` or a separate `--data-dir` parameter that defaults to the version root. The two paths are always independent.

**Status:** SOLVED in redist-cli runner.rs
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
**Proved by:** Font file moved to `redist/crates/redist-map/src/font_embed.ttf` — `include_bytes!("font_embed.ttf")` works from any workspace build context
**Test:** `cargo build --release --manifest-path redist/Cargo.toml` succeeds

## PP-12: WKB decode API surface mismatch across geospatial crate ecosystem

**Pattern:** Multiple Rust crates in the geo ecosystem provide WKB encoding/decoding with subtly different APIs and `geo-types` version pins. Calling the wrong API (`WkbReader` which doesn't exist, or `Wkb(bytes).to_geo()` through a private module path) produces compile errors. Adding a secondary WKB crate alongside `geo` risks `geo-types` version conflicts that cause type incompatibility at runtime.

**Domain:** Any Rust system using the GeoRust ecosystem (geo, geo-types, geozero). Arises because the ecosystem has multiple competing WKB crates with overlapping functionality.

**Why it's hard to catch:** The correct API (`geo_types::Geometry::from_wkb(&mut cursor, WkbDialect::Wkb)` via the `FromWkb` trait) is not obvious from crate documentation. Wrong calls compile only after adding the right imports, making the error appear to be an import issue rather than an API surface issue.

**Structural solution:** Use `geozero` as the single WKB bridge — it is already a transitive dependency of `geo` and guarantees type compatibility. Import via `use geozero::wkb::{FromWkb, WkbDialect}` and call `geo_types::Geometry::from_wkb(...)`. Never add a second WKB crate.

**Status:** SOLVED
**Proved by:** `dissolve.rs::wkb_to_geometry()` uses `geozero::wkb::FromWkb` — all WKB tests pass including MultiPolygon island tract case
**Test:** `redist-map::dissolve::tests::test_wkb_decode_known_unit_square`, `test_wkb_multipolygon_does_not_panic`

## PP-13: Integer parameter truncation at subprocess boundary

**Pattern:** A system passes a numeric parameter to an external tool via a command-line string. The external tool uses integer parsing (`atoi()`) internally, which silently truncates any decimal part. The calling system computes the parameter as a float and formats it as a float string, assuming the tool receives the float value. Both sides behave correctly individually; the mismatch only appears at the boundary, and no error is raised — the tool accepts the (truncated) integer silently.

**Domain:** Any system that invokes external tools via command-line strings where the tool's parameter parsing is stricter than the calling language's default numeric formatting. Common in scientific computing where the external tool (METIS, BLAS, etc.) is written in C and uses `atoi`/`strtol` for performance. Arises because the calling code and the tool documentation may describe the parameter in compatible terms (e.g., both say "ufactor controls balance") without clarifying the type contract.

**Why it's hard to catch:** The external tool accepts the malformed value without error. Behavior changes only quantitatively, not qualitatively — partitions are still produced, just with different (usually tighter) balance tolerance than intended. At small district counts (10 congressional districts), the difference between ufactor=1 and ufactor=5 is invisible because METIS achieves balance easily either way. The error only manifests at large district counts (98 state house) where the tighter-than-intended tolerance becomes impossible to satisfy.

**Structural solution:** Explicitly convert parameters to the type expected by the external tool before formatting as a string. For integer tools, round to integer and document the conversion formula at the call site. Add a unit test that calls the conversion function and asserts the output type and expected value for known inputs. The test must use the same conversion path as the production code.

**Status:** SOLVED
**Proved by:** gpmetis receives `-ufactor=50` (integer) instead of `-ufactor=1.0500` (float that atoi truncates to 1). FL 120HD now passes balance check.
**Test:** `redist-cli::bisection_runner::tests::test_ufactor_wasnt_silently_truncated_regression`, `test_ufactor_integer_conversion_5_pct`
