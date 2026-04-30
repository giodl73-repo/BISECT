# Plan: Civic Bidirectional Input

> **For agentic workers:** Do not execute until the spec at `docs/superpowers/specs/2026-04-30-civic-bidirectional.md` v2 (with v2.1 patches) has been reviewed and approved. Steps use checkbox (`- [ ]`) syntax for tracking.

**Date:** 2026-04-30
**Spec:** `docs/superpowers/specs/2026-04-30-civic-bidirectional.md`
**v2.1 tracking ref:** `docs/superpowers/specs/2026-04-30-v21-tracking.md`
**Goal:** Open the inbound pipe so civic groups can submit communities-of-interest, race-of-candidate annotations, and counter-proposals as first-class data — validated, canonicalized, URL-snapshotted, and conflict-aware — without requiring state-staff cooperation.

**Depends on:** Plan Comparison & Narrative (read-side `--comments` already shipped); State Staff Interop (provides the `--as-civic-counter-proposal` import flag); Callais Evidence Layer (consumes `race_of_candidate.csv` and the matching `race_of_candidate_provenance.json` schema this plan extends).
**Blocks:** Court Submission Reports (BD-R1 enforcement of `--allow-non-strict-civic` lives there but reads our manifest annotations).

**v2.1 items addressed by this plan:**
- B-04 (replace L2 "if available" with checked-in fixture `tests/fixtures/civic/la_2024_round/`, sanitized)
- B-08 (race-conflict sensitivity test asserts the *result* — disagreement above threshold X triggers `robust=false` flag)
- SUR-02 (add VT/New England COI example alongside the Lafayette one)
- PP-27 (UTF-8 BOM-tolerant reader, declared encoding policy)
- PP-28 (GEOID leading-zero loss detection with Excel-aware error message)
- PP-29 (parsed-IP loopback rejection, not string-match)
- PP-30 (URL snapshot bounded fetch / no-creds / dynamic-page handling)
- C-02 (URL snapshot protocol, WARC-when-available with `(headers.txt, body.bin)` fallback)
- CM-03 (Sheets template + plain-English how-to land *with* this plan, not after)
- CM-04 (dogfood test names a *small* civic group, not a state chapter)
- BD-R2 (race-of-candidate `attestation-doc` format / SHA / zip-inclusion — matched edit with the Callais plan)

---

## Pre-Conditions

- `redist` builds clean from `cargo build --release --locked`
- `redist import --as-civic-counter-proposal` is wired (State Staff Interop plan)
- `redist compare --comments-label <LABEL>` is wired (Plan Comparison plan)
- Callais Evidence Layer plan has landed `race_of_candidate_provenance.json` v1 schema and the parallel-curator-runs path (we extend it; we do not introduce it)
- Census tract-set lookup for `(year, state)` is reachable from `redist-data` (used by `redist fetch` today)

---

## Task 1: `redist civic` subcommand scaffolding + manifest schema

**Files:** `redist/crates/redist-cli/src/civic/{mod.rs, ingest.rs, list_show.rs, conflicts.rs, candidate_race.rs, manifest.rs}`, `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/main.rs`, `redist/crates/redist-cli/src/lib.rs`, `docs/file-formats/civic-coi-csv.md`

The scaffolding follows the existing subcommand pattern in `args.rs` (`SuiteCommands` enum + `SuiteArgs` wrapper).

- [ ] **1.1** Add `Civic(CivicArgs)` to `Commands` in `args.rs`. `CivicArgs` wraps `#[command(subcommand)] pub command: CivicCommands`. Variants: `Ingest`, `AddCandidateRace`, `List`, `Show`, `Conflicts`. Each variant is a `#[derive(Parser)]` struct mirroring spec §"CLI surface".
- [ ] **1.2** Wire dispatch in `main.rs` under `Commands::Civic(civic_args) => match civic_args.command { ... }`, calling into a new `redist_cli::civic` module. Errors flow through the same `eprintln!("ERROR: {e}"); std::process::exit(1);` envelope as siblings, with categorized `[INPUT]` / `[CONFIG]` / `[NETWORK]` prefixes per `docs/error-conventions.md` (see Onboarding plan Task 6).
- [ ] **1.3** Define `CivicManifest` in `civic/manifest.rs` with `serde::{Serialize, Deserialize}`. Required fields:
  ```jsonc
  {
    "schema_version": "civic-coi v1",
    "label": "lwv_comments",
    "submitter": "League of Women Voters Louisiana",
    "submitted_at": "2026-04-15T...Z",
    "ingested_at": "...",
    "year": "2020",
    "state": "LA",
    "validate_mode": "strict",
    "input": {
      "original_path": "original.csv",
      "original_sha256": "...",
      "original_encoding_detected": "utf-8-bom",
      "original_line_endings": "crlf"
    },
    "normalized": {
      "path": "normalized.csv",
      "sha256": "...",
      "row_count": 47
    },
    "validation": {
      "errors": 0, "warnings": 2,
      "warnings_detail": [...]
    },
    "url_snapshots": [ /* see Task 5 */ ],
    "build_commit": "<SHA>",
    "redist_version": "..."
  }
  ```
- [ ] **1.4** Output layout exactly per spec: `outputs/{version}/civic_inputs/{label}/{manifest.json, normalized.csv, original.csv, validation_log.txt, conflicts.json, snapshots/}`. Directory creation uses the same `io_utils.rs` helpers `redist export` uses; `--output-base` defaults to `outputs`.
- [ ] **1.5** `docs/file-formats/civic-coi-csv.md` — spec-mirror reference for the column schema, validation policy modes, and the canonical-CSV byte-stability rules. Linked from `docs/REDIST_CLI.md` and from the new `docs/civic/HOWTO.md` (Task 9).
- [ ] **1.6** L0 unit tests: `civic ingest --help`, `civic list --help`, etc. all parse without panicking; `CivicManifest` round-trips through serde with both required-only and full payloads.

**Exit:** `redist civic --help` lists the five subcommands. Manifest schema is documented + serde-tested.

---

## Task 2: CSV reader with BOM tolerance + canonicalization (PP-27)

**Files:** `redist/crates/redist-cli/src/civic/csv_io.rs`, `redist/crates/redist-cli/src/civic/canonical.rs`

The COI CSV reader is the single chokepoint for civic input. Every byte that flows downstream goes through it.

- [ ] **2.1** Encoding detection in `csv_io.rs`: read the first 4 bytes raw. UTF-8 BOM (`EF BB BF`) → strip and proceed. UTF-16 LE BOM (`FF FE`) or UTF-16 BE BOM (`FE FF`) → reject with the spec error: `[INPUT] CSV is UTF-16; re-export as UTF-8 (Excel: 'Save As → CSV UTF-8'). UTF-16 is not supported.` UTF-8 with no BOM → proceed. Record the detection result in `manifest.input.original_encoding_detected`.
- [ ] **2.2** Use the `csv` crate (already a workspace dep) with `ReaderBuilder::has_headers(true)`. Required columns: `geoid, comment_id, label, source, source_url, confidence, submitted_at`. Unknown columns are kept verbatim and round-tripped into `normalized.csv` with a `warning_unknown_column` validation entry (lenient/advisory) or a hard error (strict).
- [ ] **2.3** `confidence` parsing: case-insensitive match against `{high, medium, low}`. Anything else fails strict, warns in lenient.
- [ ] **2.4** `submitted_at` parsing: ISO-8601 only (`chrono::DateTime::parse_from_rfc3339`). Reject with explicit "ISO-8601 required (e.g. `2026-04-15T00:00:00Z`)" message.
- [ ] **2.5** Canonicalization in `canonical.rs` produces `normalized.csv` byte-stable across runs / platforms per spec §Canonicalization:
  - UTF-8, **no BOM**
  - LF line endings (`\n`); never `\r\n`
  - Exactly one trailing newline
  - Sort by `(geoid, comment_id)` lexicographically (stable on ties)
  - All cells with their trailing whitespace stripped
  - GEOIDs always quoted
  - First line: `# civic-coi-csv v1` schema-version header (a CSV comment line; we write it as a literal first line, then the column header). The reader skips a leading line beginning with `#` if present.
- [ ] **2.6** L0 tests: round-trip a UTF-8-with-BOM CRLF input through the reader → assert `normalized.csv` is UTF-8 no-BOM LF, sorted, byte-stable across two ingest runs (compare SHA-256). Round-trip a UTF-16 input → assert clean rejection with the documented error string. Round-trip a CSV with mixed leading/trailing whitespace → assert it is stripped.

**Exit:** Reader handles every encoding civic groups will throw at it (Excel default UTF-8 BOM, Sheets export UTF-8 no-BOM, Numbers export with CRLF). `normalized.csv` has byte-identical SHA across two consecutive runs.

---

## Task 3: GEOID typo + leading-zero detection (PP-28)

**Files:** `redist/crates/redist-cli/src/civic/geoid_validate.rs`

The spec calls leading-zero loss "the single most common Excel error." This task implements the detector that names it explicitly.

- [ ] **3.1** Build a `valid_geoids: HashSet<String>` for `(year, state)` by reusing the existing TIGER tract reader at `redist-data/src/tiger.rs` — the same source of record `redist fetch` uses today. Falls back to a checked-in tract list under `redist-data/data/tracts/{year}/{state}.txt` if TIGER files are unavailable in the current environment (deterministic CI path).
- [ ] **3.2** Per-row GEOID checks, in order:
  - **Length 11 + numeric + present in `valid_geoids`** → PASS
  - **Length 9 or 10, numeric**: emit the specific PP-28 message: `[INPUT] Row {N} GEOID '{geoid}' is {len} digits; tract GEOIDs are 11 digits. Excel/Sheets stripped a leading zero. Re-export the GEOID column with column-format = Text. See docs/civic/HOWTO.md#leading-zero.`
  - **Length 11 + numeric, but not in `valid_geoids`** → "GEOID not found in {year} {state} tract set; check for a typo or wrong state/year."
  - **Non-numeric or other lengths** → generic format error citing the row + column.
- [ ] **3.3** Strict mode: any GEOID error is fatal. Lenient: continues, records all errors as `validation.warnings_detail`. Advisory: only the structural format error is fatal; semantic checks (set-membership) become warnings.
- [ ] **3.4** Two-pass `comment_id` checks:
  - No GEOID appears in two different `comment_id`s without a `# note: <text>` row preceding the duplicate. Strict: fatal; lenient: warn.
  - Contiguity: warn-only — for each `comment_id`, load the adjacency graph (existing `adjacency_loader.rs`) and report whether the GEOIDs form a connected sub-graph. Non-contiguous COIs are legitimate (the spec says so); we surface, never block.
- [ ] **3.5** L0 tests: synthetic CSV with one 10-digit GEOID asserts the PP-28 error literal naming the leading-zero remediation; synthetic CSV with a typo'd 11-digit GEOID asserts the "not found in tract set" message; synthetic CSV where COI-A has tracts `{A,B}` and COI-B has tract `{B}` triggers the cross-COI duplicate error in strict and a warning in lenient.

**Exit:** Excel's leading-zero pitfall is detected and named in plain English; tract-set membership uses the same source `redist fetch` does.

---

## Task 4: URL parser with parsed-IP loopback rejection (PP-29)

**Files:** `redist/crates/redist-cli/src/civic/url_validate.rs`

String-match rejection of `"localhost"` is *insufficient*; the spec lists `127.x`, `0.0.0.0`, `::1`, `[::ffff:127.0.0.1]`, `192.168/16`, `10/8`, `172.16/12`, `169.254/16`. The fix is to parse the URL, resolve the host, and inspect the `IpAddr`.

- [ ] **4.1** Add `url = "2"` to `redist-cli/Cargo.toml`.
- [ ] **4.2** `validate_source_url(url: &str) -> Result<ParsedUrl, UrlError>`:
  - Parse with `url::Url::parse`. Reject `mailto:`, `file:`, `data:`, `javascript:` schemes.
  - Reject `WWW-Authenticate`-gated URLs at *fetch* time, not parse time (Task 5).
  - Extract host. If the host is a literal IP (`Url::host()` returns `Host::Ipv4 | Host::Ipv6`), use it directly. If the host is a name, do **not** resolve here — defer to fetch time. The parse-time check rejects only literal-IP loopbacks; fetch time rejects DNS-resolved loopbacks (Task 5.4).
  - For literal IPs, reject if **any** of: `is_loopback()`, `is_private()` (RFC 1918), `is_link_local()` (169.254/16), `is_unspecified()`, `IpAddr::Ipv6(v6).to_ipv4_mapped()` returns `Some(v4)` and that v4 hits any of the above.
  - Plain string match on `localhost` is *additive*, not load-bearing — we still reject `http://localhost/`.
- [ ] **4.3** Error messages cite the specific predicate hit (`"loopback"`, `"private (RFC 1918)"`, `"link-local"`, `"IPv4-mapped IPv6 loopback"`) so a debugging civic group knows what to fix.
- [ ] **4.4** L0 tests covering each predicate: `http://127.0.0.1/`, `http://0.0.0.0/`, `http://[::1]/`, `http://[::ffff:127.0.0.1]/`, `http://192.168.1.1/`, `http://10.0.0.1/`, `http://172.16.0.1/`, `http://169.254.1.1/`, `http://localhost/`. Each asserts a non-zero exit. Plus positive cases: `https://example.org/`, `https://lwvla.example.com/comment/123`.

**Exit:** Every loopback / private / link-local / IPv4-mapped form the spec lists is rejected with a predicate-named error; a string-match-on-`localhost` regression cannot pass these tests.

---

## Task 5: URL snapshot protocol (PP-30 / C-02)

**Files:** `redist/crates/redist-cli/src/civic/snapshot.rs`, `redist/crates/redist-cli/Cargo.toml`

The snapshot is the trial-time record. A 404 between submission and trial is the failure mode this task closes.

- [ ] **5.1** HTTP client: `reqwest = "0.12"` is already in `redist-cli/Cargo.toml` (blocking). Construct a dedicated `reqwest::blocking::Client` per ingest with: `connect_timeout(10s)`, `timeout(30s)` (total deadline), `redirect::Policy::limited(5)`, `cookie_store(false)`, custom `User-Agent: redist-civic/<version>`.
- [ ] **5.2** Bounded fetch: stream the response body. Read `Body` chunks accumulated into a `Vec<u8>` capped at **1 MB**. On cap hit, drop the rest, set `truncated=true`. Larger HTTP `content-length` headers are logged but not used to short-circuit; we always cap on bytes read so a server lying about length cannot bypass the cap.
- [ ] **5.3** No-creds policy:
  - Strip `Cookie` and `Authorization` headers on every request.
  - On any response with `WWW-Authenticate` header, abort the snapshot, mark the URL `requires_auth=true`, refuse in `strict`, accept-with-warning in `lenient`/`advisory`.
- [ ] **5.4** Fetch-time DNS-resolved loopback re-check (PP-29 belt-and-suspenders): use `reqwest::dns::Resolve` (or the `hickory-resolver` crate gated by an existing workspace policy) to resolve the host once before the request and reject if any resolved address hits the IP predicates from Task 4. Without this, a malicious DNS entry like `evil.example.com → 127.0.0.1` bypasses the literal-IP check.
- [ ] **5.5** Storage layout under `outputs/{version}/civic_inputs/{label}/snapshots/{idx}/`:
  - **WARC path (preferred)**: optional dependency `warc = "0.3"` (or equivalent crate) gated behind a `warc` Cargo feature. When the feature is enabled, write `snapshot.warc` containing one `WARC-Type: response` record with the request, response headers, and (possibly truncated) body.
  - **Fallback path (default, no extra deps)**: write `headers.txt` (one HTTP header per line, status line first) + `body.bin` (raw bytes, possibly truncated). The fallback is the spec's contract; WARC is the upgrade.
  - Both paths produce the same manifest fields: `url, fetched_at (ISO-8601 UTC), http_status, content_type, content_length_bytes, truncated, body_sha256, snapshot_path, snapshot_format ("warc" | "headers-body")`.
- [ ] **5.6** Dynamic-page heuristic per spec line 123: if `body_size < 4096 || content_type not in {text/html, application/pdf, text/plain}`, set `manifest.url_snapshots[i].suspicious = true` and append validation note `"URL snapshot may be incomplete; submitter should attach a PDF via --source-pdf <PATH>"`. Strict refuses; lenient/advisory accepts. The `--source-pdf` argument on `redist civic ingest` accepts a path that is hashed and stored alongside the snapshot.
- [ ] **5.7** Reproducibility-package inclusion: snapshot files are referenced by relative path from `manifest.json` (per M-04 / PP-31 path-portability rule); the Court Reports zipper picks them up automatically when the civic input is referenced.
- [ ] **5.8** L0 tests using a localhost test server (`hyper::server` in a thread bound to `127.0.0.1:0`): assert the truncation cap, the timeout behavior (a server that hangs > 30s), the `WWW-Authenticate` rejection, the dynamic-page heuristic firing on a 200-byte HTML response. **Note**: localhost test servers are an explicit allowed-exception to PP-29 — the test bypasses the URL validator and calls `snapshot::fetch_and_store_internal` directly, which is `#[cfg(test)] pub(crate)`-visible only.

**Exit:** A submitted URL is captured as a self-contained record by trial time. Truncation, timeouts, auth, and dynamic-page edge cases all have manifest fields and tests.

---

## Task 6: `redist civic add-candidate-race` with attestation-doc handling (BD-R2)

**Files:** `redist/crates/redist-cli/src/civic/candidate_race.rs`, the matching edit in the Callais Evidence Layer plan to land `attestation_doc_sha256` + `attestation_doc_format` in `race_of_candidate_provenance.json`

This task ships the BD-R2 matched edit. Both this plan and the Callais plan must agree on the schema fields, accepted formats, and zip-inclusion semantics.

- [ ] **6.1** CLI surface per spec:
  ```
  redist civic add-candidate-race candidates.csv \
    --year 2020 --state LA \
    --submitter "NAACP Louisiana State Conference" \
    --attestation-doc path/to/letter.pdf
  ```
  `candidates.csv` schema matches the existing Callais `race_of_candidate.csv` (curator-attested) plus a `submitter` column populated from the flag.
- [ ] **6.2** **Accepted attestation-doc formats** (BD-R2): `.pdf`, `.png`, `.jpg`/`.jpeg`, `.tif`/`.tiff` (a scanned letter on letterhead). Reject `.docx`, `.html`, `.rtf` — those are mutable. Format detection is by magic bytes (the `infer` crate, or add `infer = "0.16"`), not extension; the manifest records both the detected format and the file extension for cross-check.
- [ ] **6.3** Hash: `attestation_doc_sha256` is computed from the file bytes; `attestation_doc_format` is the magic-byte detection result; both land in `race_of_candidate_provenance.json` (Callais plan extends the schema to include them — matched edit).
- [ ] **6.4** Storage: copy the attestation doc into `outputs/{version}/civic_inputs/{label}/attestation/<sha256>.<ext>` (content-addressed). The reproducibility-package zipper (Court Reports plan) picks it up via the manifest reference.
- [ ] **6.5** Conflict surface: when two civic submitters disagree on a candidate's race, this command does **not** silently pick one. It writes both into the merged `race_of_candidate.csv` consumed by Callais analysis with distinct `submitter` rows; the Callais analysis halts the lookup and runs the with/without sensitivity check (Task 7 below verifies this end-to-end).
- [ ] **6.6** L0 tests: a `.docx` attestation is rejected with format error; a corrupt-PDF (magic bytes `%PDF` but truncated) is accepted but flagged in `validation.warnings_detail` (Daubert-style "we record, we don't adjudicate"); two submissions disagreeing on a candidate's race produce two rows with distinct `submitter` values + distinct attestation hashes.

**Exit:** Attestation docs land with format whitelist + SHA + zip-inclusion semantics matched to the Callais plan. The `race_of_candidate_provenance.json` schema includes `attestation_doc_sha256` and `attestation_doc_format` in *both* plans (single source of truth, two implementation sites).

---

## Task 7: Conflict detection + sensitivity-result asserting test (B-08)

**Files:** `redist/crates/redist-cli/src/civic/conflicts.rs`, `redist/crates/redist-cli/tests/civic_conflict_sensitivity.rs`

This is the B-08 fix: the test asserts the *result*, not just that the conflict-detection code path was invoked.

- [ ] **7.1** `redist civic conflicts --label LABEL_A --label LABEL_B [--label ...]` reads each label's `normalized.csv`, joins on GEOID, and emits `conflicts.json` with three categories per spec line 191:
  - `coi_overlap`: same GEOID in different `comment_id`s across labels. Surface; do not adjudicate.
  - `coi_label_mismatch`: same `comment_id` (across labels) with different `label` strings. Surface.
  - `candidate_race_disagreement`: same candidate, different race, different submitter. Surface.
- [ ] **7.2** Threshold for `robust=false` propagation (the B-08 specific): define a numeric threshold X = **proportion of candidates in the family with disagreement**, default 0.10 (10%), configurable via `--race-conflict-threshold`. When ≥ X of the candidate-race annotations in a Callais regression family disagree across submitters, the Callais provenance JSON is annotated with `robust=false` and the report's headline number gets the documented caveat. Below X, `robust=true` and the report omits the caveat. The threshold + the actual disagreement proportion are recorded in `race_of_candidate_provenance.json`.
- [ ] **7.3** L0 unit tests for `conflicts.rs`: synthetic two-label inputs covering each conflict category; assert `conflicts.json` shape + counts.
- [ ] **7.4** **B-08 asserting test** at `tests/civic_conflict_sensitivity.rs` (L1):
  - Build a synthetic 30-candidate Callais family
  - Inject submitter disagreement on **2** candidates (6.7%, below threshold) → run end-to-end → assert `race_of_candidate_provenance.json.robust == true` AND no caveat in the rendered report
  - Inject submitter disagreement on **5** candidates (16.7%, above threshold) → run end-to-end → assert `race_of_candidate_provenance.json.robust == false` AND the caveat string appears in the report's text
  - Inject disagreement on **3** candidates (10.0%, exactly at threshold) → assert the boundary policy (default: `>= threshold` triggers `robust=false`)
  - The test reaches into the rendered text/HTML, not just the invocation log — that is the difference B-08 calls out
- [ ] **7.5** Document the threshold in `docs/file-formats/civic-coi-csv.md` and link from the Callais analysis page.

**Exit:** Race-conflict sensitivity has a numeric threshold + a test that asserts the downstream `robust=false` flag and the caveat in the rendered report.

---

## Task 8: Civic-counter-proposal flow integration

**Files:** `examples/civic-counter-proposal/{README.md, run.sh, run.bat, plan.json, comments.csv, expected_outputs/}`, `redist/crates/redist-cli/tests/civic_counter_proposal_e2e.rs`

The flow chains three already-shipped commands; this task ships a worked example + an L1 end-to-end smoke.

- [ ] **8.1** `examples/civic-counter-proposal/` is the flow from spec lines 80-91 on synthetic Vermont data (small, deterministic, fast):
  ```
  redist import --format districtr theirplan.json \
    --as-civic-counter-proposal \
    --submitted-by "League of Women Voters Vermont" \
    --label lwvvt_alternative_2026
  redist civic ingest community_comments.csv --label lwvvt_comments \
    --submitter "Lake Champlain Neighborhood Council" \
    --year 2020 --state VT
  redist compare --plan-a vt_state_proposal --plan-b lwvvt_alternative_2026 \
    --comments-label lwvvt_comments \
    --format html,narrative,card
  ```
- [ ] **8.2** SUR-02: include both the Vermont/Lake Champlain example here AND keep the Lafayette neighborhood example documented in the spec / `docs/civic/HOWTO.md`. Two regional flavors so a civic group on either coast sees themselves.
- [ ] **8.3** L1 acceptance test runs the three commands and asserts:
  - `lwvvt_comments/manifest.json` exists with the canonical SHA stable across runs
  - The comparison HTML contains the "comments received and addressed" appendix (verified by grep, not just file existence)
  - The summary card PNG carries the civic-counter-proposal watermark from BD-N3 (already in Plan Comparison plan)
- [ ] **8.4** `expected_outputs/` carries the SHAs of the deterministic outputs; mismatch on CI surfaces a diff (same pattern as the Vermont walkthrough's `checksums.json`).

**Exit:** A civic group can run a worked, deterministic flow on synthetic VT data and produce a publishable comparison without state-staff cooperation. CI re-runs it on every PR touching the civic module.

---

## Task 9: Sheets template + HOWTO docs (CM-03)

**Files:** `docs/civic/HOWTO.md`, `docs/civic/templates/community-of-interest.xlsx`, `docs/civic/templates/community-of-interest.csv`, `docs/civic/templates/race-of-candidate.csv`, `docs/civic/templates/README.md`

The CM-03 demand: the templates ship *with* this plan, not after. Civic groups without an engineer on staff need a starting line.

- [ ] **9.1** `community-of-interest.xlsx` is an Excel/Sheets-friendly template:
  - Column 1 (`geoid`) is **Text-formatted** at the column level (this is the PP-28 mitigation at the source). A leading note row above the header instructs the user not to "Convert to Number." The XLSX is constructed via the `rust_xlsxwriter` crate or equivalent committed-once with the `cellFormat="@"` attribute — once committed, no rebuild dependency.
  - Sample rows for both Lafayette and Lake Champlain examples (SUR-02), with conspicuous "DELETE THESE EXAMPLES" comments.
  - Hidden validation sheet listing `confidence ∈ {high, medium, low}`.
- [ ] **9.2** `community-of-interest.csv` is the same template in plain CSV (UTF-8 no-BOM, LF) for civic groups using LibreOffice / Numbers / direct text editors.
- [ ] **9.3** `race-of-candidate.csv` template ships alongside, documenting the attestation-doc requirement (BD-R2) and accepted formats inline.
- [ ] **9.4** `docs/civic/HOWTO.md` is **plain English**. Sections:
  - "What this is for" (one paragraph, no jargon)
  - "What you need" (just a spreadsheet program; we explicitly do not require git/Rust/Python)
  - "Step 1: Fill in the template" — screenshot of the XLSX with arrows
  - "Step 2: Save as UTF-8 CSV" — explicit Excel / Sheets / Numbers instructions per app
  - "Step 3: Send it to your contact" — alternative: "if you have technical staff, here's the `redist civic ingest` command"
  - "Common mistakes" — leading zeros (link to the PP-28 rationale), wrong year, mixing two communities into one row
  - "Where to get help" — issue tracker URL; not "open a PR"
- [ ] **9.5** `docs/civic/templates/README.md` records the template version + the SHA of the canonical template content, so a downstream consumer can detect template drift.

**Exit:** A civic group with Sheets and no engineer can produce a valid CSV by following one page. The `geoid` column is Text-formatted at the source (PP-28 prevented up-front, not just detected at ingest).

---

## Task 10: Tests including hermetic LA fixture (B-04)

**Files:** `tests/fixtures/civic/la_2024_round/{README.md, comments.csv, expected_normalized.csv, expected_manifest_keys.json, SANITIZATION.md}`, `redist/crates/redist-cli/tests/civic_la_fixture.rs`

B-04 replaces the spec's L2 "if available" with a checked-in, sanitized fixture. CI runs it on every PR.

- [ ] **10.1** Sanitize a real Louisiana 2024 public-comment-round dataset: redact names of natural persons, replace organization names with anonymized but plausible substitutes ("Lafayette Neighborhood Coalition" stays; specific named pastors and individual contact info are removed). The full sanitization script + diff is committed at `tests/fixtures/civic/la_2024_round/SANITIZATION.md` so reviewers can audit what changed.
- [ ] **10.2** `tests/fixtures/civic/la_2024_round/comments.csv` is the sanitized input. ~100 rows touching real Louisiana 2020 GEOIDs that exist in our tract-set lookup.
- [ ] **10.3** `expected_normalized.csv` is the byte-stable canonical form. Test asserts SHA-256 equality, not just structural equality — that is the full PP-27 contract.
- [ ] **10.4** `expected_manifest_keys.json` lists the manifest keys + types (not values; ingestion timestamps differ). The test asserts presence + types, not concrete values.
- [ ] **10.5** L1 hermetic test at `tests/civic_la_fixture.rs`:
  - Runs `redist civic ingest tests/fixtures/civic/la_2024_round/comments.csv --year 2020 --state LA --label la_2024_test --validate strict`
  - Asserts exit 0
  - Asserts `outputs/{version}/civic_inputs/la_2024_test/normalized.csv` SHA matches `expected_normalized.csv` SHA
  - Asserts manifest key inventory matches `expected_manifest_keys.json`
  - Default test runs in CI on every PR.
- [ ] **10.6** L1 lenient/advisory mode tests: copy the fixture, inject a deliberate 10-digit GEOID, re-run with `--validate lenient` → assert exit 0 + warning surface; re-run with `--validate strict` → assert non-zero exit + the PP-28 error literal.
- [ ] **10.7** L0 supplements (already partially in Tasks 2-7): localhost URL rejection (PP-29), each loopback predicate, BOM-tolerance, leading-zero detector, conflict detection, race-conflict sensitivity threshold (B-08).

**Exit:** A real Louisiana 2024 round fixture (sanitized) lives in the repo and runs on every PR; the spec's L2 "if available" footnote is gone.

---

## Task 11: Dogfood test scoped to a small group (CM-04)

**Files:** `docs/civic/dogfood-test-plan.md`, `docs/civic/dogfood-test-report.md` (filled in post-test)

CM-04: dogfood with a *neighborhood association*, not a state chapter. State chapters have engineers; neighborhood associations are the actual users we are designing for.

- [ ] **11.1** Identify a small-scope test partner:
  - Target: a neighborhood association of 1-3 volunteers, no on-staff technologist
  - Geographic scope: 5-20 census tracts (i.e., a single neighborhood, not a city / not a state)
  - Examples scoped per the spec's existing examples: a Lafayette LA neighborhood association, a Burlington VT neighborhood council, a Birmingham AL precinct-level civic group
  - **Anti-target**: a state ACLU chapter, a state NAACP-LDF, a state League of Women Voters — those have litigation-grade infrastructure. Test there *after* this dogfood, not as it.
- [ ] **11.2** Test protocol in `docs/civic/dogfood-test-plan.md`:
  - Hand the volunteer the Sheets template + HOWTO from Task 9. No verbal walkthrough.
  - Ask them to fill in 5-10 rows for a real neighborhood they know
  - Time them; do not help unless they ask
  - When they hit `redist civic ingest`, observe failure modes — every failure is a docs/error-message bug filed against this plan's exit
- [ ] **11.3** Capture friction in `docs/civic/dogfood-test-report.md`: per-step time, per-step "did they need help?", per-error "did the error message tell them what to do?" The DoD edit (CM-04) is that this report exists and links the exact issue numbers we filed.
- [ ] **11.4** Re-runnable: at least one of the project's quarterly dogfood loops uses this protocol again with a different neighborhood association.

**Exit:** A real neighborhood association (small, volunteer-only) ingested a real CSV without our help. The friction report drives the next round of error-message edits.

---

## Definition of Done

- `redist civic ingest`, `add-candidate-race`, `list`, `show`, `conflicts` all parse + dispatch + produce the documented output layout
- BOM-tolerant UTF-8 reader; UTF-16 rejected with a documented remediation message (PP-27)
- GEOID leading-zero detection names the Excel cause and the column-format-as-Text fix (PP-28)
- URL parser rejects every literal-IP loopback / private / link-local form by parsed-IP, never by string-match (PP-29)
- URL snapshot fetches with bounded body / timeout / no-creds; WARC when feature-enabled, `(headers.txt, body.bin)` fallback always (PP-30, C-02)
- `attestation-doc` accepts `.pdf/.png/.jpg/.tiff` only; format + SHA recorded in `race_of_candidate_provenance.json`; matched edit landed in the Callais plan (BD-R2)
- Race-conflict sensitivity test asserts `robust=false` flag in rendered output above threshold X, `robust=true` below — the *result*, not just invocation (B-08)
- `examples/civic-counter-proposal/` runs deterministically end-to-end on synthetic VT data with stable SHAs
- `docs/civic/HOWTO.md` + Sheets/CSV templates ship with this plan (CM-03)
- Sanitized Louisiana fixture at `tests/fixtures/civic/la_2024_round/` runs on every PR (B-04)
- Dogfood test report cites a small civic group, not a state chapter (CM-04)
- VT/New England COI example lives alongside the Lafayette one (SUR-02)
- `docs/CHANGELOG.md` + `CLAUDE.md` "Recent Changes" updated

---

## Risks

| Risk | Mitigation |
|---|---|
| WARC crate selection churns or is abandoned | WARC is feature-gated; `(headers.txt, body.bin)` is the default and the spec contract; we can swap WARC crates without touching the manifest schema |
| Civic group's CSV is in CP1252 / Latin-1 (older Windows Excel default) | Detected by the BOM/encoding sniff in Task 2.1; rejected with the same "re-export as UTF-8 CSV" remediation message as UTF-16; documented in HOWTO |
| Local DNS poisoning bypasses the Task 4 literal-IP check | Task 5.4 re-checks resolved IPs at fetch time; the IP predicate is the load-bearing contract, not the URL string |
| LA 2024 fixture sanitization misses an identifier | `SANITIZATION.md` documents the diff; second reviewer sign-off required before merge; redaction is a checklist, not a function |
| Sensitivity threshold X (default 0.10) is wrong for some real case | `--race-conflict-threshold` flag exposes it; the manifest records the value used; downstream readers see the bar |
| Civic group's `attestation-doc` is a screenshot of an email, not letterhead | Magic-byte detection accepts the format; manifest records what was provided; expert witness can disqualify it; we do not adjudicate evidentiary weight |
| Dogfood partner cancels | Plan B partner is named in `dogfood-test-plan.md`; we explicitly avoid testing only with our personal networks |

---

## Out of Scope

- A web form for civic groups to submit comments (UI; civic group's own infra fills the CSV)
- NLP translation of free-text public comments into structured COI rows (civic group does the translation)
- Submitter authentication (we record `submitted_by` as free text; chain of custody is the submitter's responsibility, per spec)
- Litigation-grade evidentiary chain assertion (we record provenance, we do not certify it; the expert witness signs off, not the tool)
- Headless-browser snapshots for JS-rendered pages (spec line 123 explicitly defers to a `--source-pdf` attachment from the submitter)
- Per-state civic-counter-proposal walkthroughs beyond VT (LA fixture serves the validation path; VT serves the worked example)
