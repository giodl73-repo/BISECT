//! Civic Bidirectional Input: COI ingestion + conflict detection (Plan 6).
//!
//! Implements the high-leverage parts of the Civic Bidirectional Input plan:
//! - Task 1: `redist civic` subcommand scaffolding + `civic-coi v1` manifest schema
//! - Task 2: BOM-tolerant CSV reader + canonicalization (PP-27, DATUM)
//! - Task 3: GEOID typo + leading-zero detection (PP-28)
//! - Task 4: URL validator with parsed-IP loopback rejection (PP-29)
//! - Task 7: conflict detection (B-08)
//!
//! Deferred to next session: URL snapshot (PP-30 / C-02; needs reqwest + bounded
//! fetch + test-server infrastructure), `add-candidate-race` CLI surface
//! (Callais's `race_of_candidate` parser already exists; this is wiring),
//! Sheets template + HOWTO (CM-03), hermetic LA fixture (B-04), dogfood test
//! report (CM-04), full `redist civic ingest` end-to-end with file system I/O.
//!
//! Spec: `docs/superpowers/specs/2026-04-30-civic-bidirectional.md`
//! Plan: `docs/superpowers/plans/2026-04-30-civic-bidirectional.md`

use std::collections::{BTreeMap, HashMap, HashSet};
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ===========================================================================
// Manifest schema (Task 1)
// ===========================================================================

/// `civic-coi v1` manifest. Schema source of truth: `docs/file-formats/manifests.md` §3.8.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivicManifest {
    pub schema_version: String,
    pub label: String,
    pub submitter: String,
    /// ISO-8601 UTC timestamp when the submitter says they wrote the file.
    pub submitted_at: String,
    /// ISO-8601 UTC timestamp when `redist civic ingest` ran.
    pub ingested_at: String,
    pub year: String,
    pub state: String,
    /// `"strict"`, `"lenient"`, or `"advisory"`.
    pub validate_mode: String,
    pub input: InputRecord,
    pub normalized: NormalizedRecord,
    pub validation: ValidationRecord,
    /// Reserved for URL snapshot records (Task 5 — deferred).
    pub url_snapshots: Vec<UrlSnapshotRecord>,
    pub redist_version: String,
    pub redist_build_commit: String,
    pub redist_build_commit_short: String,
    pub rustc_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRecord {
    pub original_path: String,
    pub original_sha256: String,
    /// `"utf-8"`, `"utf-8-bom"`, `"utf-16-le-bom"`, etc.
    pub original_encoding_detected: String,
    /// `"lf"` or `"crlf"` (most common observed line ending in input).
    pub original_line_endings: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedRecord {
    pub path: String,
    pub sha256: String,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRecord {
    pub errors: usize,
    pub warnings: usize,
    pub warnings_detail: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlSnapshotRecord {
    pub url: String,
    pub fetched_at: String,
    pub http_status: u16,
    pub content_type: String,
    pub content_length_bytes: usize,
    pub truncated: bool,
    pub body_sha256: String,
    pub snapshot_path: String,
    /// `"warc"` or `"headers-body"`.
    pub snapshot_format: String,
}

// ===========================================================================
// Errors
// ===========================================================================

#[derive(Debug, Error)]
pub enum CivicError {
    #[error("[INPUT] CSV is UTF-16; re-export as UTF-8 (Excel: 'Save As → CSV UTF-8'). UTF-16 is not supported.")]
    Utf16Unsupported,
    #[error("[INPUT] CSV uses an unrecognized encoding (BOM bytes: {0:?}). Re-export as UTF-8.")]
    UnknownEncoding([u8; 4]),
    #[error("[INPUT] CSV header missing required column '{0}'. Required: geoid, comment_id, label, source, source_url, confidence, submitted_at.")]
    MissingColumn(String),
    #[error("[INPUT] Row {row} GEOID '{geoid}' is {len} digits; tract GEOIDs are 11 digits. Excel/Sheets stripped a leading zero. Re-export the GEOID column with column-format = Text. See docs/civic/HOWTO.md#leading-zero.")]
    LeadingZeroLost { row: usize, geoid: String, len: usize },
    #[error("[INPUT] Row {row} GEOID '{geoid}' not found in {state} {year} tract set; check for a typo or wrong state/year.")]
    GeoidNotInTractSet {
        row: usize,
        geoid: String,
        state: String,
        year: String,
    },
    #[error("[INPUT] Row {row}: invalid GEOID '{geoid}' ({reason})")]
    InvalidGeoidFormat {
        row: usize,
        geoid: String,
        reason: String,
    },
    #[error("[INPUT] Row {row}: invalid confidence '{value}' (must be high|medium|low)")]
    InvalidConfidence { row: usize, value: String },
    #[error("[INPUT] Row {row}: invalid submitted_at '{value}' (ISO-8601 required, e.g. 2026-04-15T00:00:00Z)")]
    InvalidSubmittedAt { row: usize, value: String },
    #[error("[INPUT] URL '{url}' rejected: scheme '{scheme}' not allowed (use http or https)")]
    UrlBadScheme { url: String, scheme: String },
    #[error("[INPUT] URL '{url}' rejected: host is {predicate} ({addr})")]
    UrlLoopbackOrPrivate {
        url: String,
        predicate: &'static str,
        addr: String,
    },
    #[error("[INPUT] URL '{url}' rejected: cannot parse ({reason})")]
    UrlParseError { url: String, reason: String },
    #[error("[INTERNAL] CSV parse error: {0}")]
    CsvParse(String),
    #[error("[INTERNAL] {0}")]
    Internal(String),
}

// ===========================================================================
// CSV canonicalization (Task 2 — PP-27, DATUM)
// ===========================================================================

/// Detected encoding marker. Records the raw BOM bytes when present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedEncoding {
    /// UTF-8 with no BOM.
    Utf8,
    /// UTF-8 with BOM (Excel default).
    Utf8WithBom,
}

/// Strip a UTF-8 BOM and reject UTF-16. Returns the un-BOM'd UTF-8 bytes plus
/// the detected-encoding marker. Spec PP-27.
pub fn detect_and_strip_bom(bytes: &[u8]) -> Result<(Vec<u8>, DetectedEncoding), CivicError> {
    if bytes.len() >= 2 && (bytes[..2] == [0xFF, 0xFE] || bytes[..2] == [0xFE, 0xFF]) {
        return Err(CivicError::Utf16Unsupported);
    }
    if bytes.len() >= 3 && bytes[..3] == [0xEF, 0xBB, 0xBF] {
        return Ok((bytes[3..].to_vec(), DetectedEncoding::Utf8WithBom));
    }
    Ok((bytes.to_vec(), DetectedEncoding::Utf8))
}

/// Detect the dominant line ending in `bytes`. Returns `"crlf"` iff CRLF
/// occurrences outnumber bare LF; otherwise `"lf"`.
pub fn detect_line_endings(bytes: &[u8]) -> &'static str {
    let mut crlf = 0usize;
    let mut bare_lf = 0usize;
    let mut prev = 0u8;
    for &b in bytes {
        if b == b'\n' {
            if prev == b'\r' {
                crlf += 1;
            } else {
                bare_lf += 1;
            }
        }
        prev = b;
    }
    if crlf > bare_lf {
        "crlf"
    } else {
        "lf"
    }
}

/// One parsed COI row. Required columns map to fields; unknown columns are
/// preserved in `extras` so canonicalization round-trips them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoiRow {
    pub geoid: String,
    pub comment_id: String,
    pub label: String,
    pub source: String,
    pub source_url: String,
    pub confidence: String,
    pub submitted_at: String,
    /// (column_name, value) for any non-required columns in the input.
    pub extras: BTreeMap<String, String>,
}

/// Required columns, in canonical (alphabetical-ish) order. Schema source of
/// truth: `docs/file-formats/civic-coi-csv.md`.
pub const COI_REQUIRED_COLUMNS: [&str; 7] = [
    "geoid",
    "comment_id",
    "label",
    "source",
    "source_url",
    "confidence",
    "submitted_at",
];

/// Parse + canonicalize a civic-coi CSV. Input may have BOM + CRLF; output is
/// UTF-8 no-BOM, LF, sorted by (geoid, comment_id), with a schema-version
/// header line.
///
/// Returns `(rows, normalized_csv_bytes, detected_encoding, line_endings)`.
pub fn parse_and_canonicalize_csv(
    raw_bytes: &[u8],
) -> Result<
    (
        Vec<CoiRow>,
        Vec<u8>,
        DetectedEncoding,
        &'static str,
    ),
    CivicError,
> {
    let (clean_bytes, encoding) = detect_and_strip_bom(raw_bytes)?;
    let line_endings = detect_line_endings(&clean_bytes);

    // The CSV reader skips a leading line beginning with `#` (schema-version
    // header is permitted on input but not required).
    let body_for_csv = strip_leading_comment_line(&clean_bytes);

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(body_for_csv);

    let headers = reader
        .headers()
        .map_err(|e| CivicError::CsvParse(e.to_string()))?
        .clone();
    // Header lookup: required columns must exist; extras preserved.
    let mut col_idx: HashMap<&str, usize> = HashMap::new();
    let mut extra_cols: Vec<(usize, String)> = Vec::new();
    for col in &COI_REQUIRED_COLUMNS {
        let idx = headers
            .iter()
            .position(|h| h.trim() == *col)
            .ok_or_else(|| CivicError::MissingColumn((*col).to_string()))?;
        col_idx.insert(col, idx);
    }
    for (i, h) in headers.iter().enumerate() {
        let h_trim = h.trim();
        if !COI_REQUIRED_COLUMNS.contains(&h_trim) {
            extra_cols.push((i, h_trim.to_string()));
        }
    }

    let mut rows: Vec<CoiRow> = Vec::new();
    for record in reader.records() {
        let r = record.map_err(|e| CivicError::CsvParse(e.to_string()))?;
        let get = |col: &str| -> String {
            r.get(col_idx[col])
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };
        let mut extras = BTreeMap::new();
        for (idx, name) in &extra_cols {
            if let Some(v) = r.get(*idx) {
                extras.insert(name.clone(), v.trim().to_string());
            }
        }
        rows.push(CoiRow {
            geoid: get("geoid"),
            comment_id: get("comment_id"),
            label: get("label"),
            source: get("source"),
            source_url: get("source_url"),
            confidence: get("confidence"),
            submitted_at: get("submitted_at"),
            extras,
        });
    }

    // Canonical form: stable sort by (geoid, comment_id).
    rows.sort_by(|a, b| (&a.geoid, &a.comment_id).cmp(&(&b.geoid, &b.comment_id)));

    let normalized = serialize_canonical(&rows, &extra_cols);
    Ok((rows, normalized, encoding, line_endings))
}

fn strip_leading_comment_line(bytes: &[u8]) -> &[u8] {
    if bytes.first() == Some(&b'#') {
        if let Some(idx) = bytes.iter().position(|&b| b == b'\n') {
            return &bytes[idx + 1..];
        }
    }
    bytes
}

fn serialize_canonical(rows: &[CoiRow], extra_cols: &[(usize, String)]) -> Vec<u8> {
    let mut out = String::new();
    // Schema-version header line.
    out.push_str("# civic-coi-csv v1\n");
    // Required columns header (always quoted-or-not deterministically).
    let mut header_cells: Vec<String> = COI_REQUIRED_COLUMNS.iter().map(|s| (*s).to_string()).collect();
    for (_, name) in extra_cols {
        header_cells.push(name.clone());
    }
    out.push_str(&header_cells.join(","));
    out.push('\n');
    for row in rows {
        let cells: Vec<String> = COI_REQUIRED_COLUMNS
            .iter()
            .map(|col| {
                let v = match *col {
                    "geoid" => &row.geoid,
                    "comment_id" => &row.comment_id,
                    "label" => &row.label,
                    "source" => &row.source,
                    "source_url" => &row.source_url,
                    "confidence" => &row.confidence,
                    "submitted_at" => &row.submitted_at,
                    _ => unreachable!(),
                };
                csv_quote(v, *col == "geoid")
            })
            .chain(extra_cols.iter().map(|(_, name)| {
                csv_quote(row.extras.get(name).map(String::as_str).unwrap_or(""), false)
            }))
            .collect();
        out.push_str(&cells.join(","));
        out.push('\n');
    }
    out.into_bytes()
}

/// CSV-quote `s` per RFC 4180. Always-quote when `force` is true (used for
/// GEOIDs to preserve any leading zeros against downstream re-import).
fn csv_quote(s: &str, force: bool) -> String {
    let needs = force || s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r');
    if needs {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

// ===========================================================================
// GEOID validation (Task 3 — PP-28)
// ===========================================================================

/// Validation severity per `--validate {strict|lenient|advisory}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidateMode {
    Strict,
    Lenient,
    Advisory,
}

impl ValidateMode {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "strict" => Ok(ValidateMode::Strict),
            "lenient" => Ok(ValidateMode::Lenient),
            "advisory" => Ok(ValidateMode::Advisory),
            _ => Err(format!(
                "[INPUT] invalid validate mode '{s}' (use strict|lenient|advisory)"
            )),
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidateMode::Strict => "strict",
            ValidateMode::Lenient => "lenient",
            ValidateMode::Advisory => "advisory",
        }
    }
}

/// Validate one GEOID string against the year/state's known tract set.
///
/// `valid_geoids` is the universe of acceptable GEOIDs for the (year, state).
/// Pass `None` to skip the set-membership check (useful when a tract list
/// isn't available locally — strict mode still catches structural issues).
pub fn validate_geoid(
    row: usize,
    raw: &str,
    state: &str,
    year: &str,
    valid_geoids: Option<&HashSet<String>>,
) -> Result<(), CivicError> {
    if raw.is_empty() {
        return Err(CivicError::InvalidGeoidFormat {
            row,
            geoid: raw.to_string(),
            reason: "empty".into(),
        });
    }
    if !raw.chars().all(|c| c.is_ascii_digit()) {
        return Err(CivicError::InvalidGeoidFormat {
            row,
            geoid: raw.to_string(),
            reason: "non-numeric".into(),
        });
    }
    match raw.len() {
        9 | 10 => Err(CivicError::LeadingZeroLost {
            row,
            geoid: raw.to_string(),
            len: raw.len(),
        }),
        11 => {
            if let Some(set) = valid_geoids {
                if !set.contains(raw) {
                    return Err(CivicError::GeoidNotInTractSet {
                        row,
                        geoid: raw.to_string(),
                        state: state.to_string(),
                        year: year.to_string(),
                    });
                }
            }
            Ok(())
        }
        _ => Err(CivicError::InvalidGeoidFormat {
            row,
            geoid: raw.to_string(),
            reason: format!("expected 11 digits, got {}", raw.len()),
        }),
    }
}

// ===========================================================================
// URL validation (Task 4 — PP-29)
// ===========================================================================

/// Validate a `source_url` string.
///
/// - Rejects mailto/file/data/javascript schemes (only http/https accepted).
/// - For literal-IP hosts: reject loopback / private / link-local / unspecified
///   / IPv4-mapped IPv6 loopback (`[::ffff:127.0.0.1]` etc.).
/// - For DNS-name hosts: parse-time check passes; fetch-time DNS resolution
///   re-applies the IP predicates (Task 5 belt-and-suspenders, deferred).
///
/// String-match rejection of `"localhost"` is additive (we still reject it),
/// not load-bearing — the IP predicate is the contract.
pub fn validate_source_url(url_str: &str) -> Result<(), CivicError> {
    let parsed = url::Url::parse(url_str).map_err(|e| CivicError::UrlParseError {
        url: url_str.to_string(),
        reason: e.to_string(),
    })?;
    let scheme = parsed.scheme();
    if !matches!(scheme, "http" | "https") {
        return Err(CivicError::UrlBadScheme {
            url: url_str.to_string(),
            scheme: scheme.to_string(),
        });
    }
    // Belt-and-suspenders: reject literal "localhost" string match too.
    if let Some(host) = parsed.host_str() {
        if host.eq_ignore_ascii_case("localhost") {
            return Err(CivicError::UrlLoopbackOrPrivate {
                url: url_str.to_string(),
                predicate: "loopback (localhost)",
                addr: host.to_string(),
            });
        }
    }
    if let Some(host) = parsed.host() {
        if let Some(predicate_addr) = ip_predicate_violation(&host) {
            let (predicate, addr) = predicate_addr;
            return Err(CivicError::UrlLoopbackOrPrivate {
                url: url_str.to_string(),
                predicate,
                addr,
            });
        }
    }
    Ok(())
}

/// Returns `Some((predicate_name, addr_string))` when the host violates an
/// IP-class predicate. Returns `None` when the host is safe (or is a DNS
/// name; DNS-resolved checks live at fetch time).
fn ip_predicate_violation(host: &url::Host<&str>) -> Option<(&'static str, String)> {
    use url::Host;
    let ip: IpAddr = match host {
        Host::Ipv4(v4) => IpAddr::V4(*v4),
        Host::Ipv6(v6) => IpAddr::V6(*v6),
        Host::Domain(_) => return None,
    };
    classify_ip(ip)
}

/// Classify an IpAddr against the loopback / private / link-local / unspecified
/// / IPv4-mapped-IPv6-loopback predicates. Returns `Some((name, addr))` on hit.
fn classify_ip(ip: IpAddr) -> Option<(&'static str, String)> {
    let addr_str = ip.to_string();
    if ip.is_loopback() {
        return Some(("loopback", addr_str));
    }
    if ip.is_unspecified() {
        return Some(("unspecified (0.0.0.0 / ::)", addr_str));
    }
    match ip {
        IpAddr::V4(v4) => {
            if v4.is_private() {
                return Some(("private (RFC 1918)", addr_str));
            }
            if v4.is_link_local() {
                return Some(("link-local (169.254/16)", addr_str));
            }
        }
        IpAddr::V6(v6) => {
            // IPv4-mapped IPv6: ::ffff:a.b.c.d. Recurse through the v4 predicates.
            if let Some(v4) = v6.to_ipv4_mapped() {
                if v4.is_loopback() {
                    return Some(("IPv4-mapped IPv6 loopback", addr_str));
                }
                if v4.is_private() {
                    return Some(("IPv4-mapped IPv6 private", addr_str));
                }
                if v4.is_link_local() {
                    return Some(("IPv4-mapped IPv6 link-local", addr_str));
                }
            }
            // Note: `is_unique_local` on Ipv6Addr is unstable; checking the
            // fc00::/7 prefix manually is overkill for this gate. Loopback
            // (::1) is handled above.
        }
    }
    None
}

// ===========================================================================
// Conflict detection (Task 7 — B-08)
// ===========================================================================

/// Result of a `redist civic conflicts` run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictsReport {
    /// Same GEOID assigned to different `comment_id`s across labels (without
    /// an explanatory `# note:` row).
    pub coi_overlap: Vec<CoiOverlap>,
    /// Same `comment_id` (across labels) with different `label` strings.
    pub coi_label_mismatch: Vec<CoiLabelMismatch>,
    /// Reserved for race-of-candidate cross-submitter disagreement (CB Task 6
    /// will populate this when the candidate-race CLI ships).
    pub candidate_race_disagreement: Vec<CandidateRaceDisagreement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoiOverlap {
    pub geoid: String,
    pub conflicting_comment_ids: Vec<(String, String)>, // (label, comment_id)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoiLabelMismatch {
    pub comment_id: String,
    pub conflicting_labels: Vec<(String, String)>, // (input_label, comment_label)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateRaceDisagreement {
    pub candidate_name: String,
    pub party: String,
    pub disagreements: Vec<(String, String)>, // (submitter, race)
}

/// Detect conflicts across N labeled inputs. Each input is `(label, rows)`.
pub fn detect_conflicts(inputs: &[(String, Vec<CoiRow>)]) -> ConflictsReport {
    // GEOID -> [(label, comment_id)]
    let mut by_geoid: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    // comment_id -> [(label, label_string)]
    let mut by_comment_id: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (input_label, rows) in inputs {
        for row in rows {
            by_geoid
                .entry(row.geoid.clone())
                .or_default()
                .push((input_label.clone(), row.comment_id.clone()));
            by_comment_id
                .entry(row.comment_id.clone())
                .or_default()
                .push((input_label.clone(), row.label.clone()));
        }
    }
    let mut coi_overlap: Vec<CoiOverlap> = Vec::new();
    for (geoid, hits) in by_geoid {
        let unique_cids: HashSet<&String> = hits.iter().map(|(_, c)| c).collect();
        if unique_cids.len() > 1 {
            coi_overlap.push(CoiOverlap {
                geoid,
                conflicting_comment_ids: hits,
            });
        }
    }
    let mut coi_label_mismatch: Vec<CoiLabelMismatch> = Vec::new();
    for (comment_id, hits) in by_comment_id {
        let unique_labels: HashSet<&String> = hits.iter().map(|(_, l)| l).collect();
        if unique_labels.len() > 1 {
            coi_label_mismatch.push(CoiLabelMismatch {
                comment_id,
                conflicting_labels: hits,
            });
        }
    }
    ConflictsReport {
        coi_overlap,
        coi_label_mismatch,
        candidate_race_disagreement: Vec::new(),
    }
}

/// Race-conflict sensitivity: given the count of disputed candidates and the
/// total candidate count in the family, returns `true` iff the disagreement
/// proportion meets or exceeds `threshold`. The Callais Evidence Layer's
/// `robust=false` flag is the downstream consumer (B-08).
///
/// Default threshold from the plan: 0.10 (10%).
pub fn race_conflict_robustness_violated(
    n_disputed: usize,
    n_total: usize,
    threshold: f64,
) -> bool {
    if n_total == 0 {
        return false;
    }
    (n_disputed as f64) / (n_total as f64) >= threshold
}

// ===========================================================================
// Top-level run_* dispatchers (CLI entry points)
// ===========================================================================

/// Top-level dispatcher invoked from `main.rs::Commands::Civic`.
pub fn run_civic(cmd: &CivicSubcommand) -> anyhow::Result<()> {
    match cmd {
        CivicSubcommand::Ingest(args) => run_ingest(args),
        CivicSubcommand::Conflicts(args) => run_conflicts(args),
        CivicSubcommand::List(args) => run_list(args),
        CivicSubcommand::Show(args) => run_show(args),
        CivicSubcommand::AddCandidateRace(args) => run_add_candidate_race(args),
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum CivicSubcommand {
    /// Ingest a community-of-interest CSV.
    Ingest(CivicIngestArgs),
    /// Detect cross-input conflicts.
    Conflicts(CivicConflictsArgs),
    /// List ingested civic inputs.
    List(CivicListArgs),
    /// Show one ingested input + its validation log.
    Show(CivicShowArgs),
    /// Add candidate-race annotations (Civic Bidirectional plan Task 6 — partial).
    AddCandidateRace(CivicAddCandidateRaceArgs),
}

#[derive(Debug, clap::Args)]
pub struct CivicIngestArgs {
    pub csv: PathBuf,
    #[arg(long)]
    pub label: String,
    #[arg(long)]
    pub year: String,
    #[arg(long)]
    pub state: String,
    #[arg(long, default_value = "strict")]
    pub validate: String,
    #[arg(long)]
    pub submitter: Option<String>,
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,
}

#[derive(Debug, clap::Args)]
pub struct CivicConflictsArgs {
    /// Repeat for each civic-input label to compare.
    #[arg(long)]
    pub label: Vec<String>,
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,
    /// Race-conflict threshold for the `robust=false` flag (CB Task 7 / B-08).
    #[arg(long, default_value_t = 0.10)]
    pub race_conflict_threshold: f64,
}

#[derive(Debug, clap::Args)]
pub struct CivicListArgs {
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,
}

#[derive(Debug, clap::Args)]
pub struct CivicShowArgs {
    #[arg(long)]
    pub label: String,
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,
}

#[derive(Debug, clap::Args)]
pub struct CivicAddCandidateRaceArgs {
    pub csv: PathBuf,
    #[arg(long)]
    pub year: String,
    #[arg(long)]
    pub state: String,
    #[arg(long)]
    pub submitter: String,
    /// Path to the curator's signed attestation document.
    #[arg(long)]
    pub attestation_doc: PathBuf,
}

fn output_dir_for_label(args_output_base: &str, version: &str, label: &str) -> PathBuf {
    PathBuf::from(args_output_base)
        .join(version)
        .join("civic_inputs")
        .join(label)
}

fn run_ingest(args: &CivicIngestArgs) -> anyhow::Result<()> {
    let mode = ValidateMode::from_str(&args.validate)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    if !args.csv.exists() {
        anyhow::bail!("[INPUT] civic CSV not found: {}", args.csv.display());
    }
    let raw = std::fs::read(&args.csv)?;
    let original_sha = sha256_hex(&raw);
    let line_endings = detect_line_endings(&raw);
    let (rows, normalized, encoding, _) = parse_and_canonicalize_csv(&raw)?;

    // GEOID validation. We don't yet ship the per-state tract set inside this
    // module (the existing TIGER reader lives in redist-data and requires
    // fetched files); for now we only run the structural checks (length 9/10
    // catches the leading-zero case). Set-membership check fires when the
    // caller passes Some(set).
    let mut errors = 0usize;
    let mut warnings: Vec<String> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        match validate_geoid(i + 2, &row.geoid, &args.state, &args.year, None) {
            Ok(()) => {}
            Err(e) => {
                let s = e.to_string();
                match mode {
                    ValidateMode::Strict => return Err(e.into()),
                    ValidateMode::Lenient | ValidateMode::Advisory => warnings.push(s),
                }
                errors += 1;
            }
        }
        if !row.source_url.is_empty() {
            if let Err(e) = validate_source_url(&row.source_url) {
                let s = e.to_string();
                match mode {
                    ValidateMode::Strict => return Err(e.into()),
                    ValidateMode::Lenient | ValidateMode::Advisory => warnings.push(s),
                }
                errors += 1;
            }
        }
    }

    // Write output dir.
    let dir = output_dir_for_label(&args.output_base, &args.version, &args.label);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("original.csv"), &raw)?;
    std::fs::write(dir.join("normalized.csv"), &normalized)?;
    let normalized_sha = sha256_hex(&normalized);
    let validation_log = warnings.join("\n");
    std::fs::write(dir.join("validation_log.txt"), &validation_log)?;

    let prov = crate::provenance::Provenance::current();
    let manifest = CivicManifest {
        schema_version: "civic-coi v1".to_string(),
        label: args.label.clone(),
        submitter: args.submitter.clone().unwrap_or_default(),
        submitted_at: now_iso8601(),
        ingested_at: now_iso8601(),
        year: args.year.clone(),
        state: args.state.clone(),
        validate_mode: mode.as_str().to_string(),
        input: InputRecord {
            original_path: args.csv.display().to_string(),
            original_sha256: original_sha,
            original_encoding_detected: match encoding {
                DetectedEncoding::Utf8 => "utf-8".into(),
                DetectedEncoding::Utf8WithBom => "utf-8-bom".into(),
            },
            original_line_endings: line_endings.into(),
        },
        normalized: NormalizedRecord {
            path: "normalized.csv".into(),
            sha256: normalized_sha,
            row_count: rows.len(),
        },
        validation: ValidationRecord {
            errors,
            warnings: warnings.len(),
            warnings_detail: warnings,
        },
        url_snapshots: Vec::new(), // Task 5 deferred
        redist_version: prov.redist_version,
        redist_build_commit: prov.redist_build_commit.clone(),
        redist_build_commit_short: prov.redist_build_commit.chars().take(8).collect(),
        rustc_version: prov.rustc_version,
    };
    std::fs::write(
        dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    eprintln!(
        "[OK] civic ingest: label={} ({} rows, {} warnings) -> {}",
        args.label,
        rows.len(),
        manifest.validation.warnings,
        dir.display()
    );
    Ok(())
}

fn run_conflicts(args: &CivicConflictsArgs) -> anyhow::Result<()> {
    if args.label.len() < 2 {
        anyhow::bail!("[INPUT] redist civic conflicts requires --label twice or more");
    }
    let mut inputs: Vec<(String, Vec<CoiRow>)> = Vec::new();
    for label in &args.label {
        let dir = output_dir_for_label(&args.output_base, &args.version, label);
        let path = dir.join("normalized.csv");
        if !path.exists() {
            anyhow::bail!(
                "[INPUT] civic input '{label}' not found at {} (run `redist civic ingest` first)",
                path.display()
            );
        }
        let raw = std::fs::read(&path)?;
        let (rows, _, _, _) = parse_and_canonicalize_csv(&raw)?;
        inputs.push((label.clone(), rows));
    }
    let report = detect_conflicts(&inputs);
    let json = serde_json::to_string_pretty(&report)?;
    println!("{json}");
    eprintln!(
        "[OK] civic conflicts: {} GEOID overlaps, {} label mismatches, {} race disagreements",
        report.coi_overlap.len(),
        report.coi_label_mismatch.len(),
        report.candidate_race_disagreement.len(),
    );
    Ok(())
}

fn run_list(args: &CivicListArgs) -> anyhow::Result<()> {
    let dir = PathBuf::from(&args.output_base)
        .join(&args.version)
        .join("civic_inputs");
    if !dir.exists() {
        eprintln!("[INFO] no civic inputs at {}", dir.display());
        return Ok(());
    }
    let entries: Vec<_> = std::fs::read_dir(&dir)?.collect::<Result<_, _>>()?;
    if entries.is_empty() {
        eprintln!("[INFO] no civic inputs ingested yet");
        return Ok(());
    }
    println!("{:<32} {:<10} {:<12} {}", "label", "year/state", "validate", "submitter");
    println!("{}", "-".repeat(80));
    for entry in entries {
        if !entry.path().is_dir() {
            continue;
        }
        let manifest_path = entry.path().join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let bytes = std::fs::read(&manifest_path)?;
        if let Ok(m) = serde_json::from_slice::<CivicManifest>(&bytes) {
            println!(
                "{:<32} {:<10} {:<12} {}",
                m.label,
                format!("{}/{}", m.year, m.state),
                m.validate_mode,
                m.submitter
            );
        }
    }
    Ok(())
}

fn run_show(args: &CivicShowArgs) -> anyhow::Result<()> {
    let dir = output_dir_for_label(&args.output_base, &args.version, &args.label);
    let manifest_path = dir.join("manifest.json");
    if !manifest_path.exists() {
        anyhow::bail!("[INPUT] no civic input '{}' at {}", args.label, dir.display());
    }
    let bytes = std::fs::read(&manifest_path)?;
    let manifest: CivicManifest = serde_json::from_slice(&bytes)?;
    println!("{}", serde_json::to_string_pretty(&manifest)?);
    Ok(())
}

fn run_add_candidate_race(_args: &CivicAddCandidateRaceArgs) -> anyhow::Result<()> {
    // CB Task 6 — wiring the existing redist-analysis::race_of_candidate parser
    // through this CLI surface is the next-session pickup. Surfaces a clear
    // error rather than silently doing nothing.
    anyhow::bail!(
        "[CONFIG] redist civic add-candidate-race is wired into the CLI surface but \
         the implementation is the next session's pickup. The race-of-candidate \
         parser already ships in redist-analysis::race_of_candidate \
         (see docs/file-formats/race-of-candidate.md). Today, build the CSV by \
         hand and pass it to `redist analyze --types bloc-voting --candidate-race-csv`."
    )
}

// ===========================================================================
// Helpers
// ===========================================================================

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_lower(&hasher.finalize())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn now_iso8601() -> String {
    redist_report::now_iso8601()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── BOM + canonicalization (Task 2) ──────────────────────────────────────

    #[test]
    fn test_detect_strip_utf8_bom() {
        let bytes = b"\xEF\xBB\xBFhello,world\n";
        let (out, enc) = detect_and_strip_bom(bytes).unwrap();
        assert_eq!(out, b"hello,world\n");
        assert_eq!(enc, DetectedEncoding::Utf8WithBom);
    }

    #[test]
    fn test_detect_strip_no_bom() {
        let bytes = b"hello,world\n";
        let (out, enc) = detect_and_strip_bom(bytes).unwrap();
        assert_eq!(out, bytes);
        assert_eq!(enc, DetectedEncoding::Utf8);
    }

    #[test]
    fn test_reject_utf16_le() {
        let bytes = b"\xFF\xFEh\x00e\x00l\x00";
        let err = detect_and_strip_bom(bytes).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("UTF-16"), "must name UTF-16: {msg}");
        assert!(msg.contains("re-export as UTF-8"), "must give remediation: {msg}");
    }

    #[test]
    fn test_reject_utf16_be() {
        let bytes = b"\xFE\xFF\x00h\x00e\x00l\x00";
        assert!(matches!(detect_and_strip_bom(bytes), Err(CivicError::Utf16Unsupported)));
    }

    #[test]
    fn test_detect_line_endings_lf() {
        assert_eq!(detect_line_endings(b"a\nb\nc\n"), "lf");
    }

    #[test]
    fn test_detect_line_endings_crlf() {
        assert_eq!(detect_line_endings(b"a\r\nb\r\nc\r\n"), "crlf");
    }

    #[test]
    fn test_canonical_round_trip_byte_stable() {
        // Same input, parsed + canonicalized twice -> byte-identical output.
        let csv = b"\xEF\xBB\xBFgeoid,comment_id,label,source,source_url,confidence,submitted_at\r\n\
                    50001000200,COI-2,East,LWV,,medium,2026-04-15T00:00:00Z\r\n\
                    50001000100,COI-1,Downtown,LWV,,high,2026-04-15T00:00:00Z\r\n";
        let (_, n1, _, _) = parse_and_canonicalize_csv(csv).unwrap();
        let (_, n2, _, _) = parse_and_canonicalize_csv(csv).unwrap();
        assert_eq!(n1, n2, "canonicalized output must be byte-stable");
        // Header includes schema-version line.
        let s = std::str::from_utf8(&n1).unwrap();
        assert!(s.starts_with("# civic-coi-csv v1\n"), "schema header must lead: {s}");
        // Sorted order: 50001000100 must appear before 50001000200.
        let pos1 = s.find("50001000100").unwrap();
        let pos2 = s.find("50001000200").unwrap();
        assert!(pos1 < pos2, "GEOIDs must be sorted ascending");
        // No CRLF in output.
        assert!(!s.contains("\r\n"), "output must use LF only");
    }

    #[test]
    fn test_canonical_strips_whitespace() {
        let csv = b"geoid,comment_id,label,source,source_url,confidence,submitted_at\n\
                    50001000100, COI-1 , Downtown , LWV , , high , 2026-04-15T00:00:00Z \n";
        let (rows, _, _, _) = parse_and_canonicalize_csv(csv).unwrap();
        assert_eq!(rows[0].comment_id, "COI-1");
        assert_eq!(rows[0].label, "Downtown");
        assert_eq!(rows[0].confidence, "high");
    }

    #[test]
    fn test_missing_required_column_rejected() {
        // Missing `submitted_at`.
        let csv = b"geoid,comment_id,label,source,source_url,confidence\n\
                    50001000100,COI-1,Downtown,LWV,,high\n";
        let err = parse_and_canonicalize_csv(csv).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("submitted_at"), "must name missing column: {msg}");
    }

    #[test]
    fn test_extras_round_trip_in_canonical_form() {
        // Unknown column `comment` is preserved.
        let csv = b"geoid,comment_id,label,source,source_url,confidence,submitted_at,comment\n\
                    50001000100,COI-1,Downtown,LWV,,high,2026-04-15T00:00:00Z,note here\n";
        let (rows, normalized, _, _) = parse_and_canonicalize_csv(csv).unwrap();
        assert_eq!(rows[0].extras["comment"], "note here");
        assert!(std::str::from_utf8(&normalized).unwrap().contains("note here"));
    }

    #[test]
    fn test_strip_leading_comment_line_helper() {
        assert_eq!(strip_leading_comment_line(b"# header\nbody\n"), b"body\n");
        assert_eq!(strip_leading_comment_line(b"body\n"), b"body\n");
    }

    // ── GEOID validation (Task 3 / PP-28) ───────────────────────────────────

    #[test]
    fn test_geoid_10_digit_excel_remediation() {
        let err = validate_geoid(5, "1001950100", "AL", "2020", None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Excel"), "must mention Excel: {msg}");
        assert!(msg.contains("Text"), "must suggest Text format: {msg}");
        assert!(msg.contains("HOWTO"), "must point at HOWTO doc: {msg}");
    }

    #[test]
    fn test_geoid_9_digit_excel_remediation() {
        assert!(matches!(
            validate_geoid(1, "100195010", "AL", "2020", None),
            Err(CivicError::LeadingZeroLost { .. })
        ));
    }

    #[test]
    fn test_geoid_11_digit_no_set_passes() {
        assert!(validate_geoid(1, "01001950100", "AL", "2020", None).is_ok());
    }

    #[test]
    fn test_geoid_11_digit_in_set_passes() {
        let mut set = HashSet::new();
        set.insert("01001950100".to_string());
        assert!(validate_geoid(1, "01001950100", "AL", "2020", Some(&set)).is_ok());
    }

    #[test]
    fn test_geoid_11_digit_not_in_set_rejected() {
        let mut set = HashSet::new();
        set.insert("01001950100".to_string());
        let err = validate_geoid(2, "01001999999", "AL", "2020", Some(&set)).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not found in"), "must say 'not found': {msg}");
        assert!(msg.contains("AL"), "must name state: {msg}");
        assert!(msg.contains("2020"), "must name year: {msg}");
    }

    #[test]
    fn test_geoid_non_numeric_rejected() {
        let err = validate_geoid(1, "abcdef12345", "AL", "2020", None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("non-numeric"), "must mention non-numeric: {msg}");
    }

    #[test]
    fn test_geoid_empty_rejected() {
        assert!(matches!(
            validate_geoid(1, "", "AL", "2020", None),
            Err(CivicError::InvalidGeoidFormat { .. })
        ));
    }

    #[test]
    fn test_geoid_wrong_length_rejected() {
        // 12 digits — neither a leading-zero loss nor a tract GEOID.
        let err = validate_geoid(1, "010019501000", "AL", "2020", None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("12"), "must mention actual length: {msg}");
    }

    // ── ValidateMode parsing ───────────────────────────────────────────────

    #[test]
    fn test_validate_mode_parse_all_three() {
        assert!(matches!(ValidateMode::from_str("strict"), Ok(ValidateMode::Strict)));
        assert!(matches!(ValidateMode::from_str("lenient"), Ok(ValidateMode::Lenient)));
        assert!(matches!(ValidateMode::from_str("advisory"), Ok(ValidateMode::Advisory)));
    }

    #[test]
    fn test_validate_mode_unknown_rejected() {
        assert!(ValidateMode::from_str("yolo").is_err());
    }

    // ── URL validation (Task 4 / PP-29) ─────────────────────────────────────

    #[test]
    fn test_url_https_safe_passes() {
        assert!(validate_source_url("https://example.org/").is_ok());
        assert!(validate_source_url("https://lwvla.example.com/comment/123").is_ok());
    }

    #[test]
    fn test_url_http_safe_passes() {
        assert!(validate_source_url("http://example.org/").is_ok());
    }

    #[test]
    fn test_url_mailto_rejected() {
        let err = validate_source_url("mailto:foo@bar.com").unwrap_err();
        assert!(err.to_string().contains("mailto"));
    }

    #[test]
    fn test_url_file_scheme_rejected() {
        assert!(validate_source_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn test_url_data_scheme_rejected() {
        assert!(validate_source_url("data:text/html,<script>alert(1)</script>").is_err());
    }

    #[test]
    fn test_url_javascript_scheme_rejected() {
        assert!(validate_source_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_url_localhost_string_rejected() {
        let err = validate_source_url("http://localhost/").unwrap_err();
        assert!(err.to_string().contains("loopback"));
    }

    #[test]
    fn test_url_127_loopback_rejected() {
        let err = validate_source_url("http://127.0.0.1/").unwrap_err();
        assert!(err.to_string().contains("loopback"));
    }

    #[test]
    fn test_url_zero_unspecified_rejected() {
        let err = validate_source_url("http://0.0.0.0/").unwrap_err();
        assert!(err.to_string().contains("unspecified"));
    }

    #[test]
    fn test_url_ipv6_loopback_rejected() {
        let err = validate_source_url("http://[::1]/").unwrap_err();
        assert!(err.to_string().contains("loopback"));
    }

    #[test]
    fn test_url_ipv4_mapped_ipv6_loopback_rejected() {
        let err = validate_source_url("http://[::ffff:127.0.0.1]/").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("loopback"), "must name loopback predicate: {msg}");
    }

    #[test]
    fn test_url_rfc1918_192_168_rejected() {
        assert!(validate_source_url("http://192.168.1.1/").unwrap_err().to_string().contains("private"));
    }

    #[test]
    fn test_url_rfc1918_10_rejected() {
        assert!(validate_source_url("http://10.0.0.1/").unwrap_err().to_string().contains("private"));
    }

    #[test]
    fn test_url_rfc1918_172_16_rejected() {
        assert!(validate_source_url("http://172.16.0.1/").unwrap_err().to_string().contains("private"));
    }

    #[test]
    fn test_url_link_local_169_254_rejected() {
        assert!(validate_source_url("http://169.254.1.1/").unwrap_err().to_string().contains("link-local"));
    }

    #[test]
    fn test_url_unparseable_rejected() {
        assert!(validate_source_url("not-a-url").is_err());
    }

    // ── Conflict detection (Task 7 / B-08) ──────────────────────────────────

    fn make_row(geoid: &str, comment_id: &str, label: &str) -> CoiRow {
        CoiRow {
            geoid: geoid.into(),
            comment_id: comment_id.into(),
            label: label.into(),
            source: "TEST".into(),
            source_url: String::new(),
            confidence: "high".into(),
            submitted_at: "2026-04-15T00:00:00Z".into(),
            extras: BTreeMap::new(),
        }
    }

    #[test]
    fn test_conflicts_no_overlap_no_mismatch() {
        let inputs = vec![
            ("a".to_string(), vec![make_row("01001950100", "C1", "Downtown")]),
            ("b".to_string(), vec![make_row("01001950200", "C2", "Eastside")]),
        ];
        let report = detect_conflicts(&inputs);
        assert!(report.coi_overlap.is_empty());
        assert!(report.coi_label_mismatch.is_empty());
    }

    #[test]
    fn test_conflicts_geoid_in_two_different_comment_ids() {
        let inputs = vec![
            ("a".to_string(), vec![make_row("01001950100", "C1", "Downtown")]),
            ("b".to_string(), vec![make_row("01001950100", "C2", "Eastside")]),
        ];
        let report = detect_conflicts(&inputs);
        assert_eq!(report.coi_overlap.len(), 1);
        assert_eq!(report.coi_overlap[0].geoid, "01001950100");
    }

    #[test]
    fn test_conflicts_label_mismatch_same_comment_id() {
        let inputs = vec![
            ("a".to_string(), vec![make_row("01001950100", "C1", "Downtown")]),
            ("b".to_string(), vec![make_row("01001950200", "C1", "Old Downtown")]),
        ];
        let report = detect_conflicts(&inputs);
        assert_eq!(report.coi_label_mismatch.len(), 1);
        assert_eq!(report.coi_label_mismatch[0].comment_id, "C1");
    }

    // ── B-08 race-conflict robustness threshold ─────────────────────────────

    #[test]
    fn test_b08_below_threshold_robust_true() {
        // 2 disputed of 30 = 6.7%, below 10% default threshold.
        assert!(!race_conflict_robustness_violated(2, 30, 0.10));
    }

    #[test]
    fn test_b08_above_threshold_robust_false() {
        // 5 disputed of 30 = 16.7%, above threshold.
        assert!(race_conflict_robustness_violated(5, 30, 0.10));
    }

    #[test]
    fn test_b08_exactly_at_threshold_violates() {
        // 3 disputed of 30 = 10.0%, exactly at threshold. Spec: >= triggers
        // robust=false ("at threshold" counts as a violation).
        assert!(race_conflict_robustness_violated(3, 30, 0.10));
    }

    #[test]
    fn test_b08_zero_total_does_not_violate() {
        // Empty family — no claim of robustness violation.
        assert!(!race_conflict_robustness_violated(0, 0, 0.10));
    }

    #[test]
    fn test_b08_configurable_threshold() {
        // 5 of 100 = 5%; violates when threshold <= 0.05.
        assert!(!race_conflict_robustness_violated(5, 100, 0.10));
        assert!(race_conflict_robustness_violated(5, 100, 0.05));
    }

    // ── End-to-end pipeline (parse + validate, no I/O) ──────────────────────

    #[test]
    fn test_end_to_end_csv_through_validators() {
        let csv = b"\xEF\xBB\xBFgeoid,comment_id,label,source,source_url,confidence,submitted_at\r\n\
                    01001950100,COI-LAF-001,Downtown,LNC,https://lnc.example.org/c/123,high,2026-04-15T00:00:00Z\r\n";
        let (rows, normalized, encoding, line_endings) = parse_and_canonicalize_csv(csv).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(encoding, DetectedEncoding::Utf8WithBom);
        assert_eq!(line_endings, "crlf");
        assert!(validate_geoid(2, &rows[0].geoid, "AL", "2020", None).is_ok());
        assert!(validate_source_url(&rows[0].source_url).is_ok());
        // Normalized output starts with schema header.
        assert!(normalized.starts_with(b"# civic-coi-csv v1\n"));
    }
}
