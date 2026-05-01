# Manifest Conventions for `redist`

**Status:** Living document — every plan that adds a new manifest type extends the inventory in §1.
**Owner:** Court Submission Reports plan Task 1 (docs/superpowers/plans/2026-04-30-court-submission-reports.md)
**v2.1 tracking:** M-03, D-01, 211-P1.3, 211-P1.4

This is the single source of truth for every manifest emitted by the `redist` toolchain. It exists so that a special master, a §2 expert, an academic replicator, or a future maintainer can read ONE document and know how every audit-trail JSON the project produces is shaped, hashed, dated, and chained.

If you're adding a new manifest type, land a one-task edit to §1 in the same commit. CI will not enforce this (yet); reviewer enforcement will.

---

## 1. Manifest inventory

| Manifest | Schema version | Owner crate / module | Plan reference |
|---|---|---|---|
| `PlanManifest` (`manifest.json` in every plan dir) | (legacy, no `schema_version` field) | `redist-report::manifest` | Pre-existing; documented here for reference |
| `narrative_manifest.json` | `narrative-manifest v1` | `redist-report::narrative_manifest` (when shipped) | Plan Comparison plan Task 9 |
| `what_if/.../manifest.json` | `whatif-manifest v1` | `redist-cli::depo::recompute` (when shipped) | Deposition Prep plan Task 2.6 |
| `deposition_log_{date}.manifest.json` | `depo-log v1` | `redist-cli::depo::log` (when shipped) | Deposition Prep plan Task 5.4 |
| `civic_inputs/.../manifest.json` | `civic-coi v1` | `redist-cli::civic::manifest` (when shipped) | Civic Bidirectional plan Task 1.3 |
| `examples/{tutorial}-walkthrough/checksums.json` | `tutorial-checksums v1` | `redist-cli::doctor` reads it | Onboarding plan Task 1.3 (shipped 2026-04-30) |
| `import_compat.json` (compile-time embedded) | `import-compat v1` | `redist-report::import_compat` (when shipped) | State Staff Interop plan Task 5.1 |
| `bloc_voting.json` `race_of_candidate_provenance` block | `race-of-candidate v1` | `redist-analysis::race_of_candidate` (shipped 2026-04-30) | Callais Evidence Layer plan Task 4 |
| `bloc_voting.json` (top-level) | `bloc-voting v1` | `redist-analysis::bloc_voting_writer` (shipped 2026-04-30) | Callais Evidence Layer plan Task 6 |
| `reproducibility_package_manifest.json` | `repro-package v1` | `redist-report::repro_zip` (when shipped) | Court Submission Reports plan Task 6 |

**Adding a new manifest type:** edit this table, add a `## §3.X — <kind> v<n>` subsection at the bottom enumerating fields beyond the canonical set, and reference both from the spec/plan that owns it.

---

## 2. Canonical required field set

Every manifest type MUST carry the following fields. Per-manifest extensions are additive; nothing in this list may be omitted without an explicit deprecation sub-spec.

| Field | Type | Required when | Definition |
|---|---|---|---|
| `schema_version` | string (`"<kind> v<n>"`) | always | Schema-version string; readers MUST refuse unknown schemas. |
| `redist_version` | string | always | The semver version of the `redist` binary that produced the manifest (e.g., `"0.1.0"`). Comes from `redist-cli::provenance::Provenance::current()`. |
| `redist_build_commit` | string (full 40-char SHA, may suffix `-dirty`) | always | The git SHA at build time (or `"unknown"` for builds outside a git checkout). |
| `redist_build_commit_short` | string (7-12 char prefix of `redist_build_commit`) | when space-constrained (e.g., narrative manifest header, summary card footer) | Convenience short SHA. |
| `rustc_version` | string | always | Output of `rustc --version` at build time. |
| `created_at` (or `ingested_at`, `generated_at`, `approved_at`) | string (ISO-8601 UTC, seconds precision, with explicit `Z`) | always | When the manifest was emitted. The exact field name varies per manifest type to reflect the action; semantics are identical. |

**Build-commit naming rule (v2.1.1 P1.4):** the canonical names are `redist_build_commit` (full) + `redist_build_commit_short` (short prefix). Plans that currently use `build_commit` (Civic, Callais bloc-voting summary header, Deposition log sidecar) MUST adopt `redist_build_commit` at implementation time. `binary_version` and `binary_sha256` in the legacy `PlanManifest` are pre-existing fields kept for backward compatibility; their semantics are documented in §3.1.

---

## 3. Per-manifest field detail

This section enumerates fields BEYOND the canonical set in §2. Read this when implementing a writer or a reader for the named manifest type.

### 3.1 `PlanManifest` (legacy, no `schema_version` field)

Source: `redist-report::manifest::PlanManifest`. Pre-existing; predates this convention document.

Required fields beyond §2:

| Field | Definition |
|---|---|
| `label` | Plan label (matches `plans/{label}/`). |
| `state_code` | Two-letter state code (uppercase). |
| `year` | Census year as a string. |
| `chamber` | `"congressional"`, `"house"`, `"senate"`. |
| `num_districts` | Plan's district count. Source of truth for downstream commands. |
| `population_source` | `"total"`, `"vap"`, `"cvap"`. Combined with `partition_mode` to detect Callais p.36 mutex violations (VRA-aware + partisan-weighted). |
| `partition_mode` | `"edge-weighted"`, `"metis-vra"`, `"partisan-weighted"`. |
| `seed` | Optional `i64`; populated for deterministic runs. |
| `binary_version` | The shipped `redist_version` value at the time the plan was written. (Legacy name; `redist_version` is the canonical field per §2 going forward.) |
| `binary_sha256` | SHA-256 of the running binary's executable bytes. Optional. |
| `binary_download_url` | GitHub release URL for the binary. Optional but recommended. |
| `adjacency_file` | Filename only (NOT a full path), for platform-independent verification. |
| `adjacency_sha256` | SHA-256 of the adjacency file bytes. |
| `adjacency_build_command` | The command that produced the adjacency file (for international/non-TIGER builds). |
| `adjacency_build_version` | `redist_version` of the binary that built the adjacency. |
| `tiger_source_url` | Census.gov TIGER URL for upstream provenance (NO local paths). |
| `tiger_sha256` | Optional SHA-256 of the upstream TIGER zip. |
| `balance_tolerance_pct` | Population balance tolerance applied during bisection, as a percentage (e.g., `0.5` for ±0.5%). |
| `population_balance_valid` | Boolean; `true` iff every district's population is within tolerance of ideal. |
| `seats_per_district` | Usually 1 for single-member; > 1 for multi-member chambers. |
| `total_seats` | `num_districts × seats_per_district`. |
| `electoral_system` | `"single_member"`, `"multi_member_party_list"`, etc. |
| `gpmetis_version` | The METIS version invoked, or `"unknown"`. |

**Migration to canonical naming:** at the next release, `binary_version` will be aliased as `redist_version` (additive); legacy field stays for compatibility.

### 3.2 `tutorial-checksums v1` (Onboarding plan; shipped 2026-04-30)

Source: `examples/{tutorial}-walkthrough/checksums.json`, consumed by `redist doctor --check-tutorial-data`.

Required fields beyond §2:

| Field | Definition |
|---|---|
| `tutorial` | Slug matching the directory name (`vermont-2020`, `louisiana-callais`). |
| `pinned_inputs` | Array of `{name, source_url, local_path, sha256}`. Each row's `local_path` is relative to repo root; `sha256` is the expected hash of the file at `local_path`. |
| `expected_outputs` | Array of `{path, sha256, tract_count?}`. `tract_count` is informational. |
| `pinned_at` | ISO-8601 UTC of the pin operation. |
| `_note` | Optional free-text explanation (e.g., "PIN_ON_FIRST_RUN placeholder until maintainer runs pin.sh"). |

### 3.3 `race-of-candidate v1` (Callais Evidence Layer; shipped 2026-04-30)

Source: `redist-analysis::race_of_candidate::RaceOfCandidateProvenance`, embedded as `race_of_candidate_provenance` in `bloc_voting.json`.

Required fields beyond §2 (note: this manifest is embedded, not standalone, so the §2 canonical set is provided by the parent `bloc_voting.json`):

| Field | Definition |
|---|---|
| `source_file` | Filename (NOT path) of the source CSV. |
| `source_sha256` | SHA-256 of the CSV bytes. |
| `annotations_independently_verified` | Boolean; `false` triggers the `[CAVEAT — annotations not independently verified]` injection in `draft_interpretation` (B-02 anchor 4). |
| `curators` | Array of `{curator, curator_credentials, curator_attestation_date, n_candidates}`. |
| `attestation_documents` | Array of `{path, format, sha256}`. `format` is one of `pdf|docx|md|txt|png|jpg|tiff` (BD-R2 reconciled union). `path` is relative to the source CSV's parent directory. |

See `docs/file-formats/race-of-candidate.md` for the full schema + curator-attestation protocol.

### 3.4 `bloc-voting v1` (Callais Evidence Layer; shipped 2026-04-30)

Source: `redist-analysis::bloc_voting_writer::BlocVotingJson`. JSON Schema at `redist/crates/redist-analysis/schemas/bloc_voting.schema.json`.

Required fields beyond §2:

| Field | Definition |
|---|---|
| `analyzer` | Constant `"bloc-voting"`. |
| `available` | Boolean; `false` only in degenerate empty-candidate sets. |
| `state` | State code. |
| `year` | Census year. |
| `election` | Election cycle name (e.g., `"presidential-primary"`). |
| `party` | `"DEM"`, `"REP"`. |
| `method` | `"wls"`, `"rxc"` (RxC is not yet implemented — returns the deferral error). |
| `minority_group` | `"black"`, `"hispanic"`, `"asian"`. |
| `alpha` | Significance threshold for Holm-corrected p-values (default `0.05`). |
| `ecology` | Object with `n_precincts`, `n_clusters`. |
| `candidates` | Array of per-candidate analyses. Each: `{candidate, regression, robustness_check, ecology_caveat, draft_interpretation}`. |
| `_family_detail` | Per-(candidate, variant) breakdown when robustness/LOO variants are present; empty for primary-only runs. |
| `race_of_candidate_provenance` | The `race-of-candidate v1` block (§3.3). |
| `provenance` | The §2 canonical fields plus `args` (the CLI invocation parameters). |

### 3.5–3.10 — to be filled by future plans

- §3.5 `narrative-manifest v1` (Plan Comparison plan)
- §3.6 `whatif-manifest v1` (Deposition Prep plan)
- §3.7 `depo-log v1` (Deposition Prep plan)
- §3.8 `civic-coi v1` (Civic Bidirectional plan)
- §3.9 `import-compat v1` (State Staff Interop plan)
- §3.10 `repro-package v1` (Court Submission Reports plan)

Each plan's Task that ships a new manifest type lands the corresponding §3.X subsection in the same commit.

---

## 4. Date semantics: `accessed_date` vs `fetched_at` (D-01)

Two date conventions exist in this codebase; they are NOT interchangeable.

- **`fetched_at`** — set by the fetcher when bytes were retrieved from a remote source. ISO-8601 UTC with seconds precision, explicit `Z` suffix. Lives inside per-input rows of any manifest that records bytes (e.g., `tiger_source_url` row in `PlanManifest`, attestation document rows in race-of-candidate provenance, civic-input source-URL snapshots).

- **`accessed_date`** — set by the citation generator at report-generation time. Recorded as a calendar date (no time-of-day) per court-citation custom. Appears ONLY in `citations[].accessed_date` (Court Submission Reports plan Task 5.11; see `docs/file-formats/citation-strings.md`).

**Rule when both apply to the same source:** `accessed_date` is derived from `fetched_at` by truncating to UTC date.

`created_at` / `ingested_at` / `generated_at` / `approved_at` are action-named variants of "when was this manifest emitted"; semantics are identical (ISO-8601 UTC seconds precision, `Z` suffix). Field name follows the action.

---

## 5. Path portability rule

Every path field in every manifest MUST be RELATIVE to the package root (the repository top-level directory or the reproducibility-zip root, whichever is the consumer's frame).

Concrete rules:
- Use forward-slash separators on all platforms (NEVER `C:\path\to\file` or `\\share\file`).
- UTF-8 encoding; no percent-encoding.
- Paths containing spaces or non-ASCII characters are stored verbatim in JSON (the JSON parser handles the quoting).
- Refuse paths that escape the package root (`..` traversal); abort with `[INTERNAL]` error.

Implementation helper (when implementing a writer): `path_relative_to_package_root(p: &Path) -> String` should be the single point of conversion.

The Court Reports reproducibility-zip pipeline runs a path-portability rewrite pass over every manifest entry before assembling the zip. Plans that emit manifests directly (e.g., the bloc-voting writer) should produce relative paths from the start; the rewrite pass is belt-and-suspenders.

---

## 6. Cross-manifest hash-link convention

Manifests reference upstream manifests by SHA-256 of the upstream's canonical-JSON representation.

- The `narrative_manifest.json` field `plan_a_manifest_sha256` is the SHA-256 of the canonicalized bytes of the upstream `PlanManifest.json`. The same applies to `plan_b_manifest_sha256` and `baseline_manifest_sha256`.
- The `whatif-manifest v1` field `parent_plan_manifest_sha256` is SHA-256 of the upstream `PlanManifest.json`. `parent_report_pdf_sha256` is SHA-256 of the rendered court PDF (when present).
- The `repro-package v1` manifest carries `pdf_sha256` (the embedded PDF's bytes) AND `included_files[].sha256` (every file in the zip).

**Canonicalization for SHA computation:**

JSON manifests are hashed in their pretty-printed form when the writer was set to "pretty" mode (default for human-readable artifacts). For byte-stable re-hashing, writers MUST:
- Use sorted-key serialization (e.g., serde's `BTreeMap` instead of `HashMap`).
- Serialize floats via `ryu` shortest-round-trip (matches our test fixtures).
- Write LF line endings (NOT CRLF) regardless of host OS.
- End with exactly one trailing newline.

The hash is computed over the file bytes as written. Re-reading the file and re-serializing through serde MAY produce different bytes if the canonicalization rules above are violated; this is a writer bug.

---

## 7. Schema versioning

Every manifest type carries `schema_version: "<kind> v<n>"`. Migration is **additive only without a major version bump**:

- Adding a new optional field → no version bump (readers must accept unknown fields, which serde does by default).
- Adding a new required field → v(n+1).
- Changing an existing field's type or semantics → v(n+1).
- Removing a field → v(n+1) AND a documented deprecation window.

Readers MUST refuse unknown major schema versions with an actionable error (per `docs/error-conventions.md` `[INPUT]` category).

The `tutorial-checksums v1` reader in `redist-cli::doctor` is the reference implementation.

---

## 8. See also

- `docs/error-conventions.md` — error categories used when manifest reads fail
- `docs/file-formats/race-of-candidate.md` — embedded `race-of-candidate v1` schema in detail
- `docs/file-formats/citation-strings.md` — `accessed_date` consumer
- `docs/REDIST_CLI.md` — `redist doctor --verify-manifest` and `--check-tutorial-data`
- `docs/superpowers/specs/2026-04-30-v21-tracking.md` — M-03, D-01, build-commit naming reconciliation
