# Interface and Protocol Invariants (IP-01..IP-03)

Properties of the CLI interface and inter-process communication protocol.

---

## IP-01: STATUS output is ASCII-only (no Unicode)

**Invariant:** All output through the STATUS:{pos}:{msg} protocol must contain only ASCII characters (bytes 0-127). No Unicode, no emoji, no em-dashes, no checkmarks.

**Why it must hold:** The Python progress coordinator reads STATUS lines as raw bytes. Windows CP1252 encoding cannot represent Unicode characters and will crash with a UnicodeEncodeError or produce corrupted output if a non-ASCII byte appears. See CLAUDE.md: "NEVER in console (Windows CP1252)".

**When it can be violated:** Adding state names with accented characters; copying Unicode symbols (✓, →, •) into STATUS messages; changing status() to not sanitize input in release mode.

**Enforcement:** status() in status.rs auto-sanitizes non-ASCII via ascii_safe() before printing. In debug builds, additional check was added.

**Test:** `tests/unit/test_rust_cli.py::TestStatusProtocol::test_no_unicode_in_help_output`
`redist/crates/redist-cli/src/status.rs::test_status_sanitizes_non_ascii_not_panic`

**Status:** ENFORCED

---

## IP-02: Error propagation from parallel tasks — no silent filter

**Invariant:** When parallel tasks are collected with Rayon or any parallel iterator, errors must propagate to the caller. Silent error filtering (`filter_map(.ok()?)`) is prohibited for tasks where failure means partial or incorrect output.

**Why it must hold:** Silent filtering produces "0/N items processed" symptoms that are extremely hard to diagnose — the actual error (METIS failure, file not found) is lost. The correct Rayon pattern: `.map(|item| ...).collect::<Result<Vec<_>, _>>()?` — first error propagates.

**When it can be violated:** Adding a new par_iter() that uses filter_map or .ok() to ignore errors; refactoring run_all_splits to use filter_map for "robustness".

**Enforcement:** run_all_splits uses `.map(...).collect::<Result<Vec<_>, _>>()?` — any split failure immediately returns with the actual error message including node path and depth.

**Test:** Tested implicitly: if Alabama's first split failed, the acceptance test would see "bisection failed: depth 0 node '': <actual error>" not silent "0/1437 tracts assigned".

**Status:** ENFORCED

---

## IP-03: REDIST_PYTHON used for all Python subprocess calls

**Invariant:** Every subprocess call to a Python interpreter in redist-cli must use the REDIST_PYTHON environment variable as the command, falling back to 'py' (Windows) or 'python3' (Unix). The bare 'python' command is prohibited.

**Why it must hold:** Compiled binaries resolve command names against their own PATH, which differs from the user's shell environment. `python` may resolve to a system Python without the required packages (numpy, pickle). REDIST_PYTHON lets the caller specify the exact interpreter.

**When it can be violated:** Adding a new subprocess call that uses `Command::new("python")` directly; copying adjacency_loader.py's python_cmd logic incorrectly.

**Enforcement:** Both adjacency_loader.rs and runner.rs read `REDIST_PYTHON` env var. Acceptance test fixtures set `env['REDIST_PYTHON'] = sys.executable`.

**Test:** `tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance` — all fixtures pass REDIST_PYTHON=sys.executable; tests would fail with ImportError if bare 'python' were used.

**Status:** ENFORCED
