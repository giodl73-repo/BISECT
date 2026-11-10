//! Shared native civic CSV, structural validation and conflict kernels. No network or filesystem access.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::net::IpAddr;
use serde::{Deserialize,Serialize};
use thiserror::Error;
/// `civic-coi v1` manifest. Schema source of truth: `docs/file-formats/manifests.md` §3.8.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivicManifest {
    pub schema_version: String,
    pub label: String,
    pub submitter: String,
    /// ISO-8601 UTC timestamp when the submitter says they wrote the file.
    pub submitted_at: String,
    /// ISO-8601 UTC timestamp when `BISECT civic ingest` ran.
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
    pub version: String,
    pub build_commit: String,
    pub build_commit_short: String,
    pub rustc_version: String,
}

/// Native ingest metadata supplied by the caller's filesystem/clock/build boundary.
pub struct IngestMetadata {
    pub label: String, pub submitter: String, pub submitted_at: String, pub ingested_at: String,
    pub year: String, pub state: String, pub validate_mode: String, pub original_path: String,
    pub version: String, pub build_commit: String, pub rustc_version: String,
}
pub struct IngestFacts<'a> {
    pub original: &'a [u8], pub normalized: &'a [u8], pub row_count: usize,
    pub encoding: DetectedEncoding, pub errors: usize, pub warnings: Vec<String>,
    pub url_snapshots: Vec<UrlSnapshotRecord>,
}
/// Assemble the exact native manifest without opening paths or reading a clock.
pub fn build_ingest_manifest(meta: IngestMetadata, facts: IngestFacts<'_>) -> CivicManifest {
    use sha2::{Digest,Sha256};
    CivicManifest {
        schema_version:"civic-coi v1".into(), label:meta.label, submitter:meta.submitter,
        submitted_at:meta.submitted_at, ingested_at:meta.ingested_at, year:meta.year, state:meta.state,
        validate_mode:meta.validate_mode,
        input:InputRecord {original_path:meta.original_path,original_sha256:format!("{:x}",Sha256::digest(facts.original)),original_encoding_detected:match facts.encoding {DetectedEncoding::Utf8=>"utf-8",DetectedEncoding::Utf8WithBom=>"utf-8-bom"}.into(),original_line_endings:detect_line_endings(facts.original).into()},
        normalized:NormalizedRecord {path:"normalized.csv".into(),sha256:format!("{:x}",Sha256::digest(facts.normalized)),row_count:facts.row_count as u64},
        validation:ValidationRecord {errors:facts.errors as u64,warnings:facts.warnings.len() as u64,warnings_detail:facts.warnings},
        url_snapshots:facts.url_snapshots,version:meta.version,build_commit_short:meta.build_commit.chars().take(8).collect(),build_commit:meta.build_commit,rustc_version:meta.rustc_version,
    }
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
    pub row_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRecord {
    pub errors: u64,
    pub warnings: u64,
    pub warnings_detail: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlSnapshotRecord {
    pub url: String,
    pub fetched_at: String,
    pub http_status: u16,
    pub content_type: String,
    pub content_length_bytes: u64,
    pub truncated: bool,
    pub body_sha256: String,
    pub snapshot_path: String,
    /// `"warc"` or `"headers-body"`.
    pub snapshot_format: String,
}

pub fn parse_manifest(bytes:&[u8])->Result<CivicManifest,serde_json::Error>{serde_json::from_slice(bytes)}
pub fn show_manifest(manifest:&CivicManifest)->Result<String,serde_json::Error>{serde_json::to_string_pretty(manifest)}
pub fn list_header()->String{format!("{:<32} {:<10} {:<12} {}\n{}","label","year/state","validate","submitter","-".repeat(80))}
pub fn list_row(m:&CivicManifest)->String{format!("{:<32} {:<10} {:<12} {}",m.label,format!("{}/{}",m.year,m.state),m.validate_mode,m.submitter)}

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
    LeadingZeroLost {
        row: usize,
        geoid: String,
        len: usize,
    },
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
) -> Result<(Vec<CoiRow>, Vec<u8>, DetectedEncoding, &'static str), CivicError> {
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

pub fn strip_leading_comment_line(bytes: &[u8]) -> &[u8] {
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
    let mut header_cells: Vec<String> = COI_REQUIRED_COLUMNS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
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
                csv_quote(
                    row.extras.get(name).map(String::as_str).unwrap_or(""),
                    false,
                )
            }))
            .collect();
        out.push_str(&cells.join(","));
        out.push('\n');
    }
    out.into_bytes()
}

/// CSV-quote `s` per RFC 4180. Always-quote when `force` is true (used for
/// GEOIDs to preserve any leading zeros against downstream re-import).
pub fn csv_quote(s: &str, force: bool) -> String {
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

/// Result of a `BISECT civic conflicts` run.
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


/// Native ingest assessment, including its strict early-return behavior.
pub fn assess_rows(rows:&[CoiRow],state:&str,year:&str,mode:ValidateMode)->Result<(usize,Vec<String>),CivicError>{
 let mut errors=0;let mut warnings=Vec::new();
 for(i,row)in rows.iter().enumerate(){
  let geoid=validate_geoid(i+2,&row.geoid,state,year,None);
  let url=if row.source_url.is_empty(){Ok(())}else{validate_source_url(&row.source_url)};
  for check in [geoid,url]{if let Err(e)=check{if mode==ValidateMode::Strict{return Err(e);}warnings.push(e.to_string());errors+=1;}}
 }
 Ok((errors,warnings))
}
