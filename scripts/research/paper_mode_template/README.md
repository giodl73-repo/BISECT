# `redist analyze --paper-mode` Template Pack

**Status:** scaffolding shipped 2026-04-30; the `redist analyze --paper-mode` flag wiring is the next-session pickup.
**Spec:** Researcher Toolkit plan Task 8 (D-05).

This directory holds the template files that `redist analyze --paper-mode` (when shipped) renders into each AEA-compliant replication package.

## What ships in this directory

| File | Purpose | Status |
|---|---|---|
| `REPRODUCE.sh` | Bash script that re-runs the analyze pipeline + verifies output checksums against the package's recorded values | Template shipped |
| `README.md` | This file (template documentation) | Shipped |
| `README.md.tmpl` | Per-package README template (rendered into each replication package) | TODO |
| `CITATION.bib.tmpl` | BibTeX citation template per `docs/file-formats/citation-strings.md` | TODO |

## What `redist analyze --paper-mode` (when shipped) will produce

Per Researcher Toolkit plan Task 8.2:

```
outputs/{version}/{label}/paper_mode/
├── README.md                      # AEA-compliant replication README (citations, prereqs, runtime budget)
├── REPRODUCE.sh                   # Rendered from this template; executable
├── seeds.json                     # Master seed + per-step seed derivations
├── inputs.sha256.json             # SHA-256 of every input file
├── expected_outputs.sha256.json   # SHA-256 of every output the analyze pipeline produced
├── environment.json               # pip freeze + cargo metadata + uname + ldd --version + target_platform
├── Cargo.lock                     # copy of the project's Cargo.lock at paper-mode time
├── rust-toolchain.toml            # copy of rust-toolchain.toml
├── requirements.lock              # uv pip compile / pip-compile output, with fallback annotation if absent
├── CITATION.bib                   # BibTeX rendering of citation-strings.md templates
├── CITATION.apa.txt               # APA rendering
└── CITATION.chicago.txt           # Chicago author-date rendering
```

The renderer substitutes the following placeholders in `REPRODUCE.sh`:

| Placeholder | Source |
|---|---|
| `{{REDIST_VERSION}}` | Runtime `redist_version` |
| `{{REDIST_BUILD_COMMIT}}` | `provenance::Provenance::current().redist_build_commit` |
| `{{CARGO_LOCK_SHA256}}` | `sha256sum redist/Cargo.lock` at paper-mode time |
| `{{RUST_TOOLCHAIN_SHA256}}` | `sha256sum redist/rust-toolchain.toml` if present |
| `{{REQUIREMENTS_LOCK_SHA256}}` | `sha256sum requirements.lock` if present |
| `{{TARGET_PLATFORM}}` | `linux-x86_64-glibc-2.35` (D-05 explicit pin) |
| `{{ANALYZE_INVOCATION}}` | The exact `redist analyze ...` command line that produced this run |
| `{{EXPECTED_OUTPUTS_JSON}}` | Path to `expected_outputs.sha256.json` (relative to package root) |

The renderer is not yet implemented; its module shape lives in `redist-cli::paper_mode` (when added).

## Why D-05 pins Linux x86_64 glibc 2.35

The AEA Data and Code Availability Policy requires byte-identical reproducibility OR a documented divergence. Floating-point math, glibc string functions, and Cargo's transitive-dependency build hashes can all differ across platforms in ways that break checksum equality. The D-05 fix: declare a SINGLE target platform, pin everything against it, and document the cross-platform workaround (WSL / Linux VM / Docker).

For reviewers on macOS or Windows: the recommended workflow is `docker run --rm -it -v $(pwd):/work ubuntu:22.04 bash`, then run `REPRODUCE.sh` from inside the container. Ubuntu 22.04 ships glibc 2.35 by default.

## What's deferred

- The renderer itself (`redist-cli::paper_mode::emit_replication_package`).
- The `--paper-mode` flag wiring on `AnalyzeArgs` (Task 8.1).
- L0 acceptance test that runs `bash REPRODUCE.sh` from a clean Ubuntu 22.04 container and verifies headline numbers byte-identically (Task 8.4).
- Conformance-lint against `social-science-data-editors/template-readme` (Task 8.5).
- Cross-platform note in `docs/research/paper-mode.md` (Task 8.6).

## See also

- `docs/superpowers/plans/2026-04-30-researcher-toolkit.md` Task 8
- `docs/superpowers/specs/2026-04-30-v21-tracking.md` D-05
- `docs/file-formats/citation-strings.md` — the citation templates
- `docs/file-formats/manifests.md` — provenance fields the package consumes
