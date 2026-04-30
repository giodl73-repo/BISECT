# Plan: Deposition Prep — Fast Iterative Re-Analysis Daemon for Trial

> **For agentic workers:** Do not execute until the spec at `docs/superpowers/specs/2026-04-30-deposition-prep.md` v2 (with v2.1 patches) has been reviewed and approved. Steps use checkbox (`- [ ]`) syntax for tracking.

**Date:** 2026-04-30
**Spec:** `docs/superpowers/specs/2026-04-30-deposition-prep.md`
**v2.1 tracking ref:** `docs/superpowers/specs/2026-04-30-v21-tracking.md`
**Goal:** Ship a `redist deposition-server` long-running daemon plus a `redist depo` family of subcommands that lets a §2 expert or special master answer parametric "what-if" questions in 1-2 seconds during a deposition, with a tamper-evident audit trail and provenance binding back to the original report.

**Depends on:** spec v2.1 approval. `redist-analysis` crate (existing). Plan Comparison plan must define `narrative_manifest.json` shape first; the deposition `manifest.json` matches its provenance fields. No hard code dependency on the Callais Evidence Layer plan, but several whitelist parameters (`bloc_p_value_method`, `bloc_robust_se_type`, `bloc_cluster_unit`) only become semantically meaningful once Callais lands.
**Blocks:** none. Researcher Toolkit is the only later spec.

**v2.1 items addressed by this plan:**
- M-05 (roadmap persona-table cell update for §2 expert)
- B-03 (daemon p99 benchmark — warm-up of N=5 discarded evals; separate VT PR-budget vs LA nightly-budget; runner class pinned per roadmap CI strategy)
- B-07 (`REDIST_BUILD_COMMIT` env override for build-commit spoof)
- S-01 (followup-doc `docs/parameters/whitelist-dependencies.md` — explicit DAG)
- BD-N2 (`--enforce-build-commit` defaults on under `--case-mode`)
- PP-23 (stale socket / orphaned pipe detection)
- PP-24 (two-phase shutdown drain)
- PP-26 (Windows named-pipe vs Unix socket — split flags)
- C-01 (deposition log canonical JSONL + hash chain + sidecar manifest + discoverability note)

---

## Pre-Conditions

- `redist analyze --label LABEL` runs end-to-end on the Vermont walkthrough fixture (delivered by Onboarding plan).
- `redist-analysis` exposes the seven analyzer surfaces (`PartisanAnalyzer`, `PoliticalAnalyzer`, `DemographicAnalyzer`, `UrbanAnalyzer`, `SummaryAnalyzer`, `analyze_mm_districts`, `compactness::all_metrics`) with stable signatures (already true at HEAD).
- `redist/crates/redist-cli/src/provenance.rs` exposes `BUILD_COMMIT` and `Provenance::current()` (already true).
- `redist/crates/redist-cli/build.rs` populates `REDIST_BUILD_COMMIT` from `git rev-parse HEAD` (already true). This plan extends the build script to honor an env-var override.
- `sha2`, `hex`, `chrono`, `tokio`, `mio` are all transitively available in `Cargo.lock`; no new top-level dependencies expected except `interprocess` (or feature-gated `tokio::net::UnixListener` + `tokio::net::windows::named_pipe`).

---

## Task 1: Whitelist dependencies follow-up doc (S-01) — write FIRST

**Files:** `docs/parameters/whitelist-dependencies.md` (new), `data/whitelist_dependencies.json` (new)

This doc is the source-of-truth DAG for which whitelist tweaks invalidate which downstream computations. It is referenced by both the daemon (which reads the dependency edges to decide what to recompute / which guardrails to fire) and the spec's risk table. It MUST land before Task 4 because the daemon parses it.

- [ ] **1.1** Create `docs/parameters/whitelist-dependencies.md` with the schema:
  ```markdown
  # Whitelist Parameter Dependency DAG

  Each row: parameter -> [downstream effects | invalidation rules | warning text].
  Read-side: `redist depo eval` consults this map to decide which analyzers re-run
  and which warnings to emit. Write-side: any new whitelist entry MUST be added here
  in the same commit that extends `WHITELIST_PARAMS` in code.
  ```
- [ ] **1.2** Document each of the eight default whitelist parameters with explicit edges:
  - `leaning_threshold` -> invalidates `close_call_band` flag set; invalidates per-district MoE flag (a district moves in/out of "competitive"); affects `mm_count`. Re-run: `partisan` analyzer + per-district narrative classification.
  - `close_call_band` -> invalidates close-call flag set only; does NOT invalidate `mm_count`.
  - `vra_min_bvap` -> invalidates `analyze_mm_districts` output. Re-run: `vra` analyzer.
  - `bloc_p_value_method` (`holm`|`bonferroni`|`none`) -> when set to `none`, BLOCKS headline-significance language in narrative output (the daemon refuses to emit "statistically significant" wording without a multiple-testing correction). Documented as a hard guardrail, not a soft warning.
  - `bloc_robust_se_type` (`hc3`|`hc1`) -> when `hc1` is selected AND `n_clusters < 30`, daemon emits a WARNING ("HC1 with n_clusters<30 is anti-conservative; HC3 recommended"). Threshold = 30 from Long & Ervin (2000).
  - `bloc_cluster_unit` (`county`|`tract`) -> changing this INVALIDATES the Holm correction family because the test count changes (different cluster grouping = different family of hypotheses). Daemon must auto-rerun the Holm step; warning surfaced in the eval response.
  - `compactness_metric` (`pp`|`reock`|`convex_hull`) -> only the compactness analyzer changes; no cross-analyzer fan-out.
  - `partisan_efficiency_threshold` -> partisan analyzer only.
- [ ] **1.3** Document the **machine-readable counterpart** at `data/whitelist_dependencies.json`. Schema: `{"schema_version": "whitelist-deps v1", "params": [{"name": "leaning_threshold", "default": 0.55, "type": "float", "range": [0.0, 1.0], "invalidates": ["close_call_band_flags", "mm_count"], "blocks_narrative_when": null, "warn_when": null}, ...]}`. The markdown doc and the JSON file must agree; an L0 test asserts they do.
- [ ] **1.4** Cross-link from the spec (`docs/superpowers/specs/2026-04-30-deposition-prep.md` §"Implementation notes") and from `docs/REDIST_CLI.md` `redist depo` section.

**Exit:** `docs/parameters/whitelist-dependencies.md` exists, `data/whitelist_dependencies.json` exists, the L0 in Task 2 round-trips both.

---

## Task 2: One-shot `redist depo recompute` subcommand

**Files:** `redist/crates/redist-cli/src/depo/mod.rs` (new module), `redist/crates/redist-cli/src/depo/recompute.rs` (new), `redist/crates/redist-cli/src/depo/whitelist.rs` (new), `redist/crates/redist-cli/src/args.rs` (extend), `redist/crates/redist-cli/src/main.rs` (wire), `redist/crates/redist-cli/src/lib.rs` (re-export)

The one-shot subcommand is the simplest implementation surface. It re-runs the analyze layer once, with overrides, against an existing plan — no daemon, no IPC. This is also the implementation that the daemon thread pool will call internally for each `eval` request.

- [ ] **2.1** Create the `depo` module skeleton: `mod.rs` exports `run_recompute`, `run_eval` (placeholder), `run_sweep`, `run_log`, `run_verify_log`, `run_stop`. Re-exported via `redist-cli/src/lib.rs`.
- [ ] **2.2** `whitelist.rs` parses `data/whitelist_dependencies.json` at startup (lazy, `OnceLock`). Expose: `parse_param_kv(s: &str) -> Result<(String, ParamValue), WhitelistError>`, `validate(name: &str, val: &ParamValue) -> Result<(), WhitelistError>`, `dependencies_of(name: &str) -> &[Dependency]`. The error type's Display string is the exact "parameter not in whitelist" message asserted by the L0 test in spec §Tests.
- [ ] **2.3** Implement `DepoRecomputeArgs` in `args.rs` with: `--plan-label LABEL`, `--year YYYY` (default 2020), `--version` (default v1), `--output-base` (default outputs), `--param KEY=VALUE` (repeatable, `Vec<String>`), `--types {all,partisan,vra,bloc-voting,compactness}` (repeatable), `--format {json,narrative,both}`, `--note STRING`, `--enforce-build-commit` (bool flag), `--whitelist-config PATH` (optional override JSON merged on top of `data/whitelist_dependencies.json`).
- [ ] **2.4** Wire the subcommand into `Commands` enum: `Depo(DepoArgs)` where `DepoArgs` has its own `subcommand` (`recompute|eval|sweep|log|verify-log|stop`). The variant for the one-shot is `DepoSubcommand::Recompute(DepoRecomputeArgs)`.
- [ ] **2.5** `recompute.rs::run_recompute()`:
  1. Load plan via `PlanContext::from_label(...)` (existing).
  2. Compute `param_hash` = SHA-256 over canonical-JSON of overrides (sorted keys, ryu-formatted floats — same canonicalization used by the log writer in Task 5).
  3. Compute output dir: `{output_base}/{version}/states/{state_lower}/what_if/{ISO8601_timestamp}_{param_hash[..12]}/`. Path is recorded in the manifest **relative to package root** per Court-Reports §3 portability rule (M-04).
  4. Build an `AnalyzerContext` injecting overrides where the analyzer accepts a config struct; for analyzers that take parameters via function args (e.g., `vra_min_bvap`), pass directly.
  5. Run only the requested analyzer types.
  6. Write `analysis.json`, `narrative.md` (when `--format` includes narrative — the [DRAFT] gate from Plan Comparison applies), `manifest.json` (parent SHAs + override hash + `Provenance::current()`), `deposition_context.txt` if `--note` was supplied.
- [ ] **2.6** `manifest.json` shape mirrors `narrative_manifest.json` from Plan Comparison so consumers can read both with one parser. Required fields:
  ```json
  {
    "schema_version": "whatif-manifest v1",
    "parent_plan_label": "...",
    "parent_plan_manifest_sha256": "...",
    "parent_report_pdf_sha256": "..." | null,
    "overrides": {"leaning_threshold": 0.53},
    "overrides_hash": "<sha256 of canonical-JSON of overrides>",
    "override_path_relative": "outputs/v1/states/vermont/what_if/2026-...",
    "redist_version": "...", "redist_build_commit": "...", "rustc_version": "...",
    "generated_at": "2026-04-30T14:22:08Z",
    "note": "..." | null
  }
  ```
- [ ] **2.7** L0 tests in `redist/crates/redist-cli/tests/depo_recompute.rs`:
  - `whitelist_rejects_unknown_param`: invoke with `--param arbitrary_key=value`; assert exit non-zero + the exact error message from spec.
  - `whitelist_accepts_known_param`: `--param leaning_threshold=0.53` succeeds against a synthetic 10-tract plan fixture.
  - `output_dir_path_is_deterministic`: same overrides + same timestamp -> same param_hash -> same dir.
  - `manifest_records_parent_sha`: assert `parent_plan_manifest_sha256` matches `sha256(plan_dir/manifest.json)`.
  - `path_is_relative_to_package_root`: assert `override_path_relative` does not start with `C:\` or `/home/`.

**Exit:** `redist depo recompute --plan-label vt_2020 --param leaning_threshold=0.53` produces an output dir whose manifest binds back to the parent plan SHA, and the whitelist gate works.

---

## Task 3: IPC abstraction layer (PP-26)

**Files:** `redist/crates/redist-cli/src/depo/ipc.rs` (new), `redist/crates/redist-cli/Cargo.toml` (add dependency)

Pick one of two implementation paths. Recommended path: **add `interprocess = "2"`** (single crate, exposes both Unix domain sockets and Windows named pipes behind a unified `LocalSocketStream` API and works without async). Fallback: feature-gated `cfg(unix)` / `cfg(windows)` impls using `std::os::unix::net::UnixListener` and `windows-sys` named-pipe bindings. Decision criterion: pick `interprocess` unless its blocking-only API forces awkward shutdown plumbing.

- [ ] **3.1** Define `trait IpcListener: Send` with `accept(&self) -> io::Result<Box<dyn IpcStream>>`, `local_addr_str(&self) -> String`, and `close(self)`. Define `trait IpcStream: Read + Write + Send`.
- [ ] **3.2** Define `enum IpcMode { UnixSocket(PathBuf), WindowsPipe(String) }` with platform-appropriate `Default::default()` resolution per spec:
  - Unix: `${XDG_RUNTIME_DIR:-/tmp}/redist-depo-${USER}.sock`. Permissions on creation: `0600` (only the user).
  - Windows: `\\.\pipe\redist-depo-${USERNAME}` (use `GetUserNameW`; fall back to `redist-depo-default`).
- [ ] **3.3** Implement `bind(mode: &IpcMode) -> io::Result<Box<dyn IpcListener>>` that chooses the right backend.
- [ ] **3.4** Stale-detection helper `detect_stale(mode: &IpcMode) -> StaleDetection` (PP-23): attempt a connect with 200ms timeout; if connection succeeds AND server responds to a `{"op":"ping"}` probe, return `Live`; if connect fails AND a stale path exists (Unix: file at the socket path; Windows: pipe doesn't exist after `WaitNamedPipe` short timeout), return `StaleSocket(path)`. Caller (the daemon `start` path) unlinks the stale socket / does nothing for the named pipe (Windows pipes don't persist past their owning process; the only "stale" case there is a PID-file mismatch).
- [ ] **3.5** PID file at `${ipc_path}.pid` (Unix) or `%LOCALAPPDATA%\redist\depo-${USERNAME}.pid` (Windows) — written atomically (write-then-rename), removed on graceful shutdown. Liveness check: `is_pid_alive(pid: u32) -> bool` using `kill(pid, 0)` on Unix and `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, pid)` on Windows.
- [ ] **3.6** L0: `ipc_round_trip` — bind, spawn a client thread, send one length-prefixed JSON message, receive an echo. Run on Linux + Windows CI; not flaky.
- [ ] **3.7** L0: `stale_socket_unlinked_on_start` — pre-create a dangling socket file, call `start`; assert the daemon binds successfully and the new socket replaces it.

**Exit:** A single `bind(IpcMode)` call returns a working listener on Linux, macOS, and Windows; stale-socket and PID-file paths are tested.

---

## Task 4: Daemon scaffolding — `redist deposition-server`

**Files:** `redist/crates/redist-cli/src/depo/server.rs` (new), `redist/crates/redist-cli/src/depo/protocol.rs` (new), `redist/crates/redist-cli/src/depo/state.rs` (new — in-memory plan state)

- [ ] **4.1** `protocol.rs` defines the wire protocol: 4-byte big-endian length prefix + UTF-8 JSON body. Request enum tags: `ping`, `eval`, `stop`, `status`. Response: `{status: "ok"|"error", ...}`. Strict size cap on incoming messages (1 MiB) to defend against accidental flooding from a runaway client.
- [ ] **4.2** `state.rs` owns the warm in-memory plan: tract assignments, adjacency graph, attribute tables, JIT-warmed analyzer caches. Loaded once at daemon start via a `PlanContext` + `Arc<LoadedPlan>`. Subsequent `eval` requests clone the `Arc`, apply overrides, and run analyzers without re-reading disk.
- [ ] **4.3** `server.rs::run_server(args: DepoServerArgs) -> anyhow::Result<()>`:
  1. Resolve `IpcMode` (CLI flag overrides default).
  2. `detect_stale` -> unlink if stale; refuse to start if `Live` (clear error: "another daemon is already serving on `<path>`; use `redist depo stop` first").
  3. Eagerly load the plan; warmup pass: run each whitelisted analyzer once with default params and discard (ensures the JIT/cache state is hot).
  4. Set up signal handler:
     - Unix: `signal_hook::flag::register(SIGTERM, ...)`, also `SIGINT` for Ctrl-C interactive testing. Hook also catches `SIGHUP` (treat as graceful shutdown).
     - Windows: `windows-sys::Win32::System::Console::SetConsoleCtrlHandler` for `CTRL_CLOSE_EVENT`, `CTRL_C_EVENT`, `CTRL_BREAK_EVENT`.
  5. Print `READY` to stdout (line-flushed) so a parent process / notebook can synchronize.
  6. Accept loop: per connection, spawn a worker on a small fixed thread pool (rayon's existing pool or a new `std::thread`-backed pool of size = `max(2, num_cpus / 2)`).
  7. Per request: validate against whitelist, dispatch to analyzer, write JSONL log entry (Task 5), respond.
- [ ] **4.4** `DepoServerArgs` (`args.rs`): `--plan-label`, `--year`, `--version`, `--output-base`, `--socket PATH` (Unix-only), `--pipe-name NAME` (Windows-only), optional cross-platform alias `--ipc <PATH-or-NAME>` resolved per platform, `--whitelist-config PATH`, `--enforce-build-commit` (bool, default determined by `--case-mode`), `--case-mode` (bool, default false), `--no-log` (bool), `--log-dir PATH` (default `outputs/{version}/states/{state}/depo_logs/`).
- [ ] **4.5** Discoverability of mode-flag mismatch: if user passes `--socket` on Windows or `--pipe-name` on Linux, refuse with the exact message from spec — "On this platform, use `--<correct-flag>` instead." (PP-26 patched in spec; this implements it.)

**Exit:** `redist deposition-server --plan-label vt_2020 --year 2020` warm-starts in ≤5s on VT, prints `READY`, and accepts an `eval` request from a separately-launched `redist depo eval` client.

---

## Task 5: Two-phase shutdown + canonical JSONL log + hash chain (PP-24, C-01)

**Files:** `redist/crates/redist-cli/src/depo/log.rs` (new), `redist/crates/redist-cli/src/depo/canonical.rs` (new — canonical JSON helpers)

- [ ] **5.1** `canonical.rs::to_canonical_json(value: &serde_json::Value) -> String`: keys sorted lexicographically; floats via `ryu`; integers as `i64`; ISO-8601 timestamps with `Z`; no trailing whitespace; one JSON value, no newline. Asserted by an L0 round-trip test against a hand-crafted golden string.
- [ ] **5.2** `log.rs::DepositionLogWriter`: opens (creates if missing) `deposition_log_{date}.jsonl` using O_APPEND on Unix and `FILE_APPEND_DATA` on Windows. Single writer thread fed by an `mpsc::Sender<LogEntry>` so concurrent eval workers don't interleave; the writer thread fsyncs after every entry (durability for evidentiary integrity outweighs throughput; we are at single-digit-Hz anyway).
- [ ] **5.3** Hash chain: each entry has a `prev_sha256` = SHA-256 of the previous entry's complete line bytes (excluding trailing newline). First entry uses `"GENESIS"`. Per-entry shape:
  ```json
  {"seq": 0, "timestamp": "2026-04-30T14:22:08.123Z", "op": "eval",
   "params": {"leaning_threshold": 0.53}, "types": ["partisan"],
   "result_summary": {"mm_count": 2, "elapsed_ms": 340},
   "build_commit": "abc...", "prev_sha256": "GENESIS"}
  ```
  Each line is the canonical-JSON serialization (Task 5.1) of the entry struct.
- [ ] **5.4** Sidecar manifest at `deposition_log_{date}.manifest.json` written:
  - On daemon start (initial fields: build_commit, plan_label, plan_manifest_sha256, ipc_mode, started_at).
  - On every entry write (atomically updated `total_entries`, `latest_entry_sha256`).
  - On graceful shutdown (final `closed_at`, `final_sha256` = SHA-256 of the entire log file).
- [ ] **5.5** Two-phase shutdown (PP-24):
  1. **Phase 1 — drain.** Set `accepting = false` (atomic bool); `accept` loop returns `EBUSY`/`{"status":"draining"}` to any new connection. Wait up to 30s for in-flight evals to complete (configurable via `--shutdown-grace-secs`, default 30).
  2. **Phase 2 — fsync + exit.** Drain the log channel, fsync the JSONL, write the final `manifest.json`, remove PID file, unlink Unix socket (Windows pipes auto-close), exit 0.
  - If a SIGKILL / unrecoverable error skips phase 2, the next daemon startup detects an "open" sidecar manifest (no `closed_at`) and prints a recovery message; the log file's hash chain is still valid because each entry was fsynced.
- [ ] **5.6** `--no-log` (BD-N1, already in spec): the daemon constructs a no-op `LogWriter` and prints a one-line stderr warning at startup ("WARNING: deposition log disabled per --no-log; you have lost the audit trail"). Discoverability note also surfaced once at startup.
- [ ] **5.7** L1: `daemon_log_replay` (10 evals; `redist depo log` reads them all back; SHA chain verifies). L0: `canonical_json_golden` (round-trip a known struct against a hex-pinned canonical string). L0: `log_chain_tamper_detected` (mutate one byte in entry #5; `verify-log` returns non-zero with the offending seq).

**Exit:** Daemon survives SIGTERM mid-eval without losing the in-flight entry; the log file passes `verify-log`; the sidecar manifest is updated atomically.

---

## Task 6: Client subcommands — `eval`, `sweep`, `log`, `verify-log`, `stop`

**Files:** `redist/crates/redist-cli/src/depo/client.rs` (new), `redist/crates/redist-cli/src/depo/{eval,sweep,log,verify_log,stop}.rs` (new)

- [ ] **6.1** `client.rs::DepoClient::connect(mode: &IpcMode) -> io::Result<DepoClient>` and `send(&mut self, req: Request) -> io::Result<Response>`. Connection is short-lived (open-send-receive-close) so the daemon never accumulates idle clients; sub-second overhead is negligible compared to analyzer cost.
- [ ] **6.2** `eval.rs::run_eval(args: DepoEvalArgs)`: parse `--param`, `--types`, `--note`; resolve `IpcMode` the same way the daemon did (use the `--socket`/`--pipe-name` discovery rules); send `{"op":"eval", ...}`; print response. If no daemon answers, the error is "no daemon found at `<path>`; start one with `redist deposition-server --plan-label LABEL`".
- [ ] **6.3** `sweep.rs::run_sweep(args: DepoSweepArgs)`: parse `--param KEY=START:STOP:STEP`, `--metric`. Walk the range, send one `eval` per step, print a small ASCII table with one auto-summary sentence per row. Also writes `--out depo_sweep.csv` if requested. Hand-checks each row's params against the whitelist locally before sending (early failure).
- [ ] **6.4** `log.rs::run_log(args: DepoLogArgs)`: read the JSONL log file (no IPC needed — it's just a file). Filter by `--since "1 hour ago"` (parse with `chrono`'s `Duration::from_human_readable`-equivalent, or accept ISO-8601 absolute timestamps). Pretty-print last N entries.
- [ ] **6.5** `verify_log.rs::run_verify_log(args: DepoVerifyLogArgs)`: walk the JSONL line by line, recompute each `prev_sha256`, assert it matches. On mismatch, print the first divergent seq number + the byte offset. Cross-check the sidecar manifest's `final_sha256` against the recomputed full-file hash (when present).
- [ ] **6.6** `stop.rs::run_stop(args: DepoStopArgs)`: send `{"op":"stop"}` to the daemon. With `--force`, also remove the socket file / PID file regardless of liveness. Print the daemon's drain time on success.
- [ ] **6.7** L1: `daemon_rhythm_50_evals` (start daemon, send 50 evals back-to-back, assert p99 < 1.5s on the runner specified in Task 9).

**Exit:** A user can run `redist deposition-server --plan-label vt_2020` in one terminal and `redist depo eval --param leaning_threshold=0.53` in another, getting sub-second responses.

---

## Task 7: Provenance binding + `--enforce-build-commit` default-on under `--case-mode` (BD-N2)

**Files:** `redist/crates/redist-cli/src/depo/provenance.rs` (new — depo-specific), `redist/crates/redist-cli/src/depo/server.rs` (extend)

- [ ] **7.1** At daemon start, read the parent plan's `manifest.json` -> capture `binary_build_commit_at_plan_time`. Compare with `provenance::BUILD_COMMIT`. If different, log a WARNING line with both values + the actionable hint ("the report you generated was on commit X; you are running on commit Y. Rebuild from commit X with `git checkout X && cargo build --release` to match.").
- [ ] **7.2** `--enforce-build-commit` flag: when the warning above would fire, error out instead. Daemon refuses to start; exit code 2.
- [ ] **7.3** `--case-mode` flag: a bundle that defaults `--enforce-build-commit=true` AND `--no-log=false` AND surfaces a one-line "case mode active" stderr banner. The user must opt out explicitly with `--enforce-build-commit=false` to weaken it. (This is the precise BD-N2 disposition.) Default for the standalone `--enforce-build-commit` flag without `--case-mode` is OFF (current spec).
- [ ] **7.4** Also bind the parent **report PDF SHA-256** when present: read `outputs/{version}/{year}/plans/{label}/reports/court_submission.pdf` (path convention from Court Reports plan; if the file doesn't exist, the field is `null` — not an error). Recorded into every `eval` response and every `manifest.json` written by `recompute`.
- [ ] **7.5** L0: `enforce_build_commit_blocks_mismatch` — daemon started with `--enforce-build-commit` against a manifest whose `binary_build_commit` differs from `provenance::BUILD_COMMIT`; assert exit code 2 + actionable error. Uses `REDIST_BUILD_COMMIT` env override (Task 8) to spoof the running binary's commit.
- [ ] **7.6** L0: `case_mode_default_on` — invoke `redist deposition-server --case-mode --plan-label X` against a mismatched manifest WITHOUT explicitly passing `--enforce-build-commit`; assert it still blocks. Inverse: with `--case-mode --no-enforce-build-commit`, it warns but starts.

**Exit:** A daemon refuses to serve a 2026-05-15 deposition against a report built on 2026-04-15 unless the operator explicitly weakens the gate.

---

## Task 8: `REDIST_BUILD_COMMIT` env override for tests (B-07)

**Files:** `redist/crates/redist-cli/build.rs` (extend), `docs/error-conventions.md` (one-line addition)

- [ ] **8.1** Extend `build.rs` to honor an optional env var: if `REDIST_BUILD_COMMIT_OVERRIDE` is set at build time, use its value instead of `git rev-parse HEAD`. This is the **build-time override** for reproducible-build pinning.
- [ ] **8.2** Separately, expose a **runtime override** for tests: `provenance::current_for_test()` (cfg-gated to `#[cfg(test)]` + a public escape hatch behind `cfg(feature = "test-provenance-override")`) reads `REDIST_BUILD_COMMIT` from the env at runtime and substitutes for `BUILD_COMMIT`. The L0 tests in Task 7 use this. **Production binaries do not consult the runtime override** — the feature is off in release builds.
- [ ] **8.3** Document both overrides in `docs/error-conventions.md` (under a new "Build provenance overrides" subsection): when each is appropriate (build-time = reproducible packaging; runtime = automated tests), and the explicit warning that the runtime override is feature-gated for a reason.
- [ ] **8.4** L0 unit test in `redist/crates/redist-cli/tests/build_commit_override.rs`: build with `REDIST_BUILD_COMMIT_OVERRIDE=deadbeef0000`, assert `provenance::BUILD_COMMIT == "deadbeef0000"`. Runtime override test sets `REDIST_BUILD_COMMIT=cafef00d0000` before invoking the daemon and asserts the daemon's logged commit matches.

**Exit:** Tests can spoof either build-time or runtime build commits without polluting release binaries.

---

## Task 9: p99 benchmark methodology (B-03)

**Files:** `redist/crates/redist-cli/benches/depo_p99.rs` (new — Criterion benchmark), `.github/workflows/pr.yml` (extend), `.github/workflows/nightly.yml` (extend), `docs/superpowers/specs/2026-04-30-roadmap-five-star.md` (already documents the runner class — confirm it; do NOT modify)

- [ ] **9.1** Criterion benchmark with **N=5 discarded warm-up evals** (explicit per B-03) followed by N=50 measured evals. The benchmark prints p50/p95/p99 plus the full histogram so a regression on the tail is visible, not just the mean.
- [ ] **9.2** Two named runs:
  - **VT (PR-budget)**: synthetic VT-sized plan (~193 tracts), all 8 whitelist parameters cycled. Budget assertion: p99 ≤ 1.5s on `ubuntu-latest-large`. Runs in PR CI.
  - **LA (nightly-budget)**: real LA tract-level plan (~1100 tracts). Budget assertion: p99 ≤ 5s on `ubuntu-latest-large`. Runs in nightly CI only (per roadmap CI strategy v2.1 patch — already pinned).
- [ ] **9.3** PR workflow gate: the benchmark exits non-zero if the VT budget is exceeded. Nightly workflow gate: the benchmark exits non-zero if the LA budget is exceeded. Failure surfaces as a CI annotation pointing at the slowest analyzer in the budget.
- [ ] **9.4** Document the methodology in this plan and cross-link from the spec's §"Definition of done".

**Exit:** PR runs assert p99 ≤ 1.5s for VT; nightly asserts p99 ≤ 5s for LA; both use 5 discarded warm-ups.

---

## Task 10: `examples/deposition-checklist.ipynb`

**Files:** `examples/deposition-checklist.ipynb` (new), `examples/README.md` (extend)

- [ ] **10.1** Notebook with cells:
  1. Markdown intro: who this is for, when to run it (the night before a deposition), what to print and bring.
  2. Start daemon (`redist deposition-server --plan-label vt_2020 --case-mode &` via `subprocess.Popen`; wait for `READY`).
  3. Pre-sweep the 8 default whitelist parameters around their defaults (`leaning_threshold` 0.50:0.60:0.01, `vra_min_bvap` 0.45:0.55:0.01, etc.). Each sweep cell renders a markdown table.
  4. One bookmarked cell per likely-asked question (the 5-10 from spec §5): "If opposing counsel asks about leaning_threshold robustness, the answer is at row N in §3.1 — read aloud verbatim: `<one-sentence summary>`".
  5. Generate a printable PDF of the notebook via `nbconvert --to pdf`; the PDF + the live daemon are the two artifacts the expert brings.
  6. Final cell: graceful `redist depo stop`.
- [ ] **10.2** Notebook declares `runtime_budget_secs: 120` in cell metadata (per roadmap CI strategy nightly notebook execution).
- [ ] **10.3** Cross-link from `docs/quickstart/quickstart-callais-expert.md` and `docs/REDIST_CLI.md`.

**Exit:** The expert can open the notebook, run all cells, and walk into a deposition with a printed sweep table + a live daemon ready to answer follow-ups.

---

## Task 11: Documentation cross-wiring + roadmap persona table update (M-05, M-04)

**Files:** `docs/REDIST_CLI.md` (extend), `docs/superpowers/specs/2026-04-30-roadmap-five-star.md` (one cell edit), `CLAUDE.md` (entry), `docs/CHANGELOG.md` (entry), `docs/file-formats/manifests.md` (extend — written by Court Reports plan as a followup-doc; this plan adds the `whatif-manifest v1` row)

- [ ] **11.1** **M-05**: edit the persona table in `docs/superpowers/specs/2026-04-30-roadmap-five-star.md`. The "§2 plaintiff's expert post-Callais" row's "What's missing" cell currently lists evidence layer items only. Append: "fast iterative re-analysis at trial (`redist depo`)". This is a pure doc edit; no other roadmap text changes.
- [ ] **11.2** Add a `redist depo` section to `docs/REDIST_CLI.md` covering all six subcommands with example invocations and expected output snippets. Include the platform-flag note (Unix `--socket`, Windows `--pipe-name`). Document `--case-mode` and the build-commit gate.
- [ ] **11.3** Add to `docs/file-formats/manifests.md` (written first by the Court Reports plan): a `whatif-manifest v1` row listing every field from Task 2.6.
- [ ] **11.4** **M-04**: confirm `override_path_relative` in the depo manifest is relative to repo root (not absolute); add an explicit assertion test under `redist/crates/redist-cli/tests/depo_path_portable.rs`.
- [ ] **11.5** `CLAUDE.md` "Recent Changes" entry; `docs/CHANGELOG.md` entry. CLAUDE.md "Common Commands" gets a "Deposition prep" subsection.

**Exit:** A first-time visitor can find the daemon docs in 60 seconds; the roadmap persona table reflects the new capability.

---

## Definition of Done

- `redist depo recompute` runs one-shot with overrides and writes a `what_if/.../manifest.json` whose `parent_plan_manifest_sha256` matches the parent plan.
- `redist deposition-server --plan-label vt_2020` warm-starts in ≤5s for VT (≤30s for LA — nightly only).
- `redist depo eval` p99 ≤ 1.5s across the eight default whitelist parameters on the VT fixture (PR-gated, runner pinned per CI strategy).
- LA p99 ≤ 5s on the LA fixture (nightly-gated). Both benchmarks use 5 discarded warm-up evals.
- Whitelist enforcement: arbitrary keys rejected with the spec's exact error string.
- Two-phase shutdown verified: SIGTERM mid-eval drains and the in-flight entry appears in the log.
- `redist depo verify-log` detects single-byte tampering at any seq position.
- Sidecar `deposition_log_{date}.manifest.json` is written atomically on every entry and on shutdown.
- `--enforce-build-commit` blocks a mismatched binary; defaults on under `--case-mode`.
- `REDIST_BUILD_COMMIT_OVERRIDE` (build-time) and `REDIST_BUILD_COMMIT` (runtime, feature-gated) both work; documented in `docs/error-conventions.md`.
- `docs/parameters/whitelist-dependencies.md` (markdown) and `data/whitelist_dependencies.json` (machine-readable) agree (L0 round-trip).
- `examples/deposition-checklist.ipynb` runs nightly with `runtime_budget_secs: 120` enforced.
- Roadmap persona table cell updated (M-05).
- All deposition manifest paths are relative to package root (M-04).

---

## Risks

| Risk | Mitigation |
|---|---|
| `interprocess` crate has a regression on Windows named-pipe permissions | Pin to a known-good version (currently 2.x); fallback path is feature-gated `cfg(unix)`/`cfg(windows)` impls already designed in Task 3. Decision is reversible without re-architecting clients. |
| Two-phase shutdown deadlocks waiting on a stuck eval | `--shutdown-grace-secs` (default 30) bounds phase-1; after timeout, force-cancel the worker and write a `{"op":"shutdown_force","stranded_eval_seq":N}` log entry so the audit trail records the truncation. |
| Hash-chain canonicalization disagrees across platforms (CRLF leak on Windows) | The log writer opens files in binary mode and writes `\n` only; no `text` mode. L0 golden test asserts byte-for-byte equality of the log file produced on Linux vs Windows for the same eval sequence. |
| p99 budget flakes on shared CI | Budget pinned to `ubuntu-latest-large` per roadmap CI strategy; benchmark exits with an explicit annotation on regression instead of just failing red so on-call can see the histogram. |
| Whitelist DAG drift between markdown doc and JSON | One CI test parses both and asserts they declare the same parameter set with the same defaults. Drift is impossible-to-merge, not a runtime warning. |
| FRCP 26 discoverability of the log file surprises an expert mid-deposition | Discoverability note printed at daemon start (every start, not gated on first-use). `--no-log` documented in the same paragraph. Notebook (Task 10) cell-1 markdown re-states it. |
| Build-commit override accidentally enabled in a release binary | Runtime override is gated behind `cfg(feature = "test-provenance-override")` AND `#[cfg(test)]`. Release build's `Cargo.toml` does NOT enable the feature. CI release-gate workflow asserts the feature is off. |

---

## Out of Scope

- Re-running the bisection itself (different plan = `redist state`, not deposition).
- Live courtroom display (handled by State Staff Interop / Districtr loop).
- Multi-user shared depo server (single-expert model is the realistic mode).
- Audio transcription of depositions into parameter changes.
