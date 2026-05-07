//! Race-of-candidate annotation parser + provenance struct.
//!
//! This is the chain-of-custody artifact for *Louisiana v. Callais* §2 expert
//! testimony. Each row attests that a curator (with stated credentials, dated
//! attestation, and a supporting document at a hashed path) classified a named
//! candidate's race for the bloc-voting analysis.
//!
//! Schema source of truth: `docs/file-formats/race-of-candidate.md` (also
//! produced by Onboarding plan Task 1.6's BD-R2 reconciliation).
//!
//! Spec: `docs/superpowers/plans/2026-04-30-callais-evidence-layer.md` Task 4.
//!
//! BD-R2 reconciliation: `attestation_doc_format` accepts the union of curator
//! narrative formats (pdf|docx|md|txt) and civic submitter scanned-letterhead
//! formats (png|jpg|jpeg|tif|tiff). Court-mode rejection of non-PDF lives in
//! Court Submission Reports plan (BD-R1 `--allow-non-strict-civic`), not here.
//!
//! Anchor 4: when ANY ingested annotation has `independently_verified=false`,
//! the orchestrator (Task 3) prepends `[CAVEAT — annotations not independently
//! verified]` to the `draft_interpretation` field and sets
//! `race_of_candidate_provenance.annotations_independently_verified = false`.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use thiserror::Error;

/// Closed enumeration of accepted race classifications. Any value outside this
/// set is a hard `[INPUT]` error per `docs/error-conventions.md`.
///
/// Case-sensitive. Matches the spec's race vocabulary; do not silently widen.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CandidateRace {
    Black,
    #[serde(rename = "white")]
    White,
    Hispanic,
    Asian,
    Native,
    #[serde(rename = "other")]
    Other,
}

impl CandidateRace {
    pub fn from_str(s: &str) -> Result<Self, RaceParseError> {
        match s {
            "Black" => Ok(CandidateRace::Black),
            "white" => Ok(CandidateRace::White),
            "Hispanic" => Ok(CandidateRace::Hispanic),
            "Asian" => Ok(CandidateRace::Asian),
            "Native" => Ok(CandidateRace::Native),
            "other" => Ok(CandidateRace::Other),
            _ => Err(RaceParseError::InvalidRace {
                value: s.to_string(),
            }),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CandidateRace::Black => "Black",
            CandidateRace::White => "white",
            CandidateRace::Hispanic => "Hispanic",
            CandidateRace::Asian => "Asian",
            CandidateRace::Native => "Native",
            CandidateRace::Other => "other",
        }
    }
}

/// Accepted attestation-doc formats, BD-R2 reconciled union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttestationDocFormat {
    Pdf,
    Docx,
    Md,
    Txt,
    Png,
    Jpg,
    Tiff,
}

impl AttestationDocFormat {
    /// Detect from filename extension. Matches the BD-R2 union. `.jpeg` and
    /// `.tif` are aliases for `.jpg` and `.tiff` respectively.
    ///
    /// Note: production code SHOULD use magic-byte detection (Civic
    /// Bidirectional plan Task 6.2 calls for `infer = "0.16"`); this
    /// extension-based fallback is the read-side stub. The Civic plan's
    /// add-candidate-race command does the magic-byte detection at write time.
    pub fn from_extension(ext: &str) -> Result<Self, RaceParseError> {
        match ext.to_ascii_lowercase().as_str() {
            "pdf" => Ok(AttestationDocFormat::Pdf),
            "docx" => Ok(AttestationDocFormat::Docx),
            "md" => Ok(AttestationDocFormat::Md),
            "txt" => Ok(AttestationDocFormat::Txt),
            "png" => Ok(AttestationDocFormat::Png),
            "jpg" | "jpeg" => Ok(AttestationDocFormat::Jpg),
            "tif" | "tiff" => Ok(AttestationDocFormat::Tiff),
            "html" | "htm" | "rtf" => Err(RaceParseError::InvalidAttestationFormat {
                value: ext.to_string(),
            }),
            _ => Err(RaceParseError::InvalidAttestationFormat {
                value: ext.to_string(),
            }),
        }
    }

    /// True if this format is acceptable for court-mode submission without the
    /// `--allow-non-strict-civic` override (BD-R1). Currently only PDF.
    pub fn is_court_strict(&self) -> bool {
        matches!(self, AttestationDocFormat::Pdf)
    }
}

/// One parsed annotation row.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CandidateAnnotation {
    pub candidate_name: String,
    pub party: String,
    pub race: CandidateRace,
    pub curator: String,
    pub curator_credentials: String,
    /// ISO-8601 date string per spec (we parse format-only; no chrono dep).
    pub curator_attestation_date: String,
    pub source: String,
    pub independently_verified: bool,
    /// Path to the attestation document, RELATIVE to the CSV's parent
    /// directory (kept relative for reproducibility-zip portability per M-04).
    pub attestation_doc_path: PathBuf,
    pub attestation_doc_format: AttestationDocFormat,
    /// SHA-256 of the attestation doc bytes, computed at parse time (BD-R2).
    pub attestation_doc_sha256: String,
}

/// Provenance block emitted into `bloc_voting.json` per spec §JSON schema.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RaceOfCandidateProvenance {
    pub schema_version: String,
    /// Path to the source CSV, relative to the analysis output directory.
    pub source_file: String,
    /// SHA-256 of the source CSV bytes.
    pub source_sha256: String,
    /// True iff every annotation row had `independently_verified=true`. False
    /// triggers the caveat injection in the orchestrator (anchor 4).
    pub annotations_independently_verified: bool,
    /// Per-curator attestation summary (one row per distinct curator).
    pub curators: Vec<CuratorRecord>,
    /// Per-attestation-doc record (one row per distinct attestation file).
    pub attestation_documents: Vec<AttestationDocRecord>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CuratorRecord {
    pub curator: String,
    pub curator_credentials: String,
    pub curator_attestation_date: String,
    pub n_candidates: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttestationDocRecord {
    pub path: String,
    pub format: AttestationDocFormat,
    pub sha256: String,
}

/// All annotations parsed from one CSV file, plus their provenance block.
#[derive(Debug, Clone)]
pub struct AnnotationSet {
    /// Every row read, in CSV order.
    pub annotations: Vec<CandidateAnnotation>,
    /// Multi-curator dispute support (BD-R2 + spec Task 4.5): rows with the
    /// same `(candidate_name, party)` from different curators land in this
    /// vector (NOT collapsed). The orchestrator iterates and runs one analysis
    /// per curator, surfacing both as `parallel_curator_runs`.
    ///
    /// Keyed by (candidate_name, party); value is the list of curator names
    /// who annotated this candidate.
    pub disputes: HashMap<(String, String), Vec<String>>,
    /// Flat provenance block ready for serialization into bloc_voting.json.
    pub provenance: RaceOfCandidateProvenance,
}

#[derive(Debug, Error)]
pub enum RaceParseError {
    #[error("[INPUT] race-of-candidate CSV not found: {0}")]
    FileNotFound(PathBuf),
    #[error("[INPUT] cannot read race-of-candidate CSV: {0}")]
    Io(#[from] std::io::Error),
    #[error("[INPUT] race-of-candidate CSV header missing required column: {0}")]
    MissingColumn(String),
    #[error("[INPUT] invalid race value '{value}' (must be one of: Black, white, Hispanic, Asian, Native, other)")]
    InvalidRace { value: String },
    #[error("[INPUT] invalid attestation_doc_format '{value}' (BD-R2 union: pdf|docx|md|txt|png|jpg|jpeg|tif|tiff; html/htm/rtf rejected)")]
    InvalidAttestationFormat { value: String },
    #[error("[INPUT] row {row}: empty {field}")]
    EmptyRequired { row: usize, field: String },
    #[error("[INPUT] row {row}: invalid bool '{value}' for independently_verified (use true|false)")]
    InvalidBool { row: usize, value: String },
    #[error("[INPUT] row {row}: attestation document not found at '{path}'")]
    AttestationDocMissing { row: usize, path: String },
    #[error("[INPUT] row {row}: attestation_doc_format '{declared}' does not match file extension '{actual}'")]
    AttestationFormatMismatch {
        row: usize,
        declared: String,
        actual: String,
    },
    #[error("[INPUT] csv parse error at line {line}: {message}")]
    Csv { line: usize, message: String },
}

/// Parse a race-of-candidate CSV at `csv_path`, computing SHA-256 of every
/// attestation document referenced (per Task 4.4).
///
/// Required columns: `candidate_name`, `party`, `race`, `curator`,
/// `curator_credentials`, `curator_attestation_date`, `source`,
/// `independently_verified`, `attestation_doc_path`, `attestation_doc_format`.
///
/// `attestation_doc_path` is interpreted relative to the parent directory of
/// `csv_path`. The parser computes `attestation_doc_sha256` by reading the
/// referenced file; this is the BD-R2 chain-of-custody anchor.
pub fn parse_race_of_candidate_csv<P: AsRef<Path>>(
    csv_path: P,
) -> Result<AnnotationSet, RaceParseError> {
    let csv_path = csv_path.as_ref();
    if !csv_path.exists() {
        return Err(RaceParseError::FileNotFound(csv_path.to_path_buf()));
    }
    let csv_bytes = std::fs::read(csv_path)?;
    let source_sha256 = sha256_hex(&csv_bytes);
    let csv_dir = csv_path.parent().unwrap_or(Path::new("."));

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_bytes.as_slice());

    let headers = reader.headers().map_err(|e| RaceParseError::Csv {
        line: 0,
        message: e.to_string(),
    })?;
    let required = [
        "candidate_name",
        "party",
        "race",
        "curator",
        "curator_credentials",
        "curator_attestation_date",
        "source",
        "independently_verified",
        "attestation_doc_path",
        "attestation_doc_format",
    ];
    let mut col_idx: HashMap<&str, usize> = HashMap::new();
    for col in &required {
        let idx = headers
            .iter()
            .position(|h| h == *col)
            .ok_or_else(|| RaceParseError::MissingColumn((*col).to_string()))?;
        col_idx.insert(*col, idx);
    }

    let mut annotations: Vec<CandidateAnnotation> = Vec::new();
    let mut disputes: HashMap<(String, String), Vec<String>> = HashMap::new();
    let mut all_verified = true;

    for (i, record) in reader.records().enumerate() {
        let row_num = i + 2; // 1-based + header row
        let record = record.map_err(|e| RaceParseError::Csv {
            line: row_num,
            message: e.to_string(),
        })?;
        let get = |col: &str| -> &str {
            record
                .get(col_idx[col])
                .map(|s| s.trim())
                .unwrap_or("")
        };
        let need_nonempty = |col: &str| -> Result<String, RaceParseError> {
            let v = get(col);
            if v.is_empty() {
                Err(RaceParseError::EmptyRequired {
                    row: row_num,
                    field: col.to_string(),
                })
            } else {
                Ok(v.to_string())
            }
        };

        let candidate_name = need_nonempty("candidate_name")?;
        let party = need_nonempty("party")?;
        let race = CandidateRace::from_str(get("race"))?;
        let curator = need_nonempty("curator")?;
        let curator_credentials = need_nonempty("curator_credentials")?;
        let curator_attestation_date = need_nonempty("curator_attestation_date")?;
        let source = need_nonempty("source")?;
        let iv_str = get("independently_verified");
        let independently_verified = match iv_str {
            "true" | "True" | "TRUE" | "1" | "yes" => true,
            "false" | "False" | "FALSE" | "0" | "no" => false,
            other => {
                return Err(RaceParseError::InvalidBool {
                    row: row_num,
                    value: other.to_string(),
                })
            }
        };
        if !independently_verified {
            all_verified = false;
        }

        let doc_path_rel_str = need_nonempty("attestation_doc_path")?;
        let doc_path_rel = PathBuf::from(&doc_path_rel_str);
        let doc_path_abs = csv_dir.join(&doc_path_rel);

        let declared_fmt = need_nonempty("attestation_doc_format")?;
        let format = AttestationDocFormat::from_extension(&declared_fmt)?;

        // Cross-check declared format against extension.
        let actual_ext = doc_path_rel
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let actual_fmt = AttestationDocFormat::from_extension(&actual_ext);
        match actual_fmt {
            Ok(actual) if actual != format => {
                return Err(RaceParseError::AttestationFormatMismatch {
                    row: row_num,
                    declared: declared_fmt.clone(),
                    actual: actual_ext,
                });
            }
            Err(_) => {
                return Err(RaceParseError::AttestationFormatMismatch {
                    row: row_num,
                    declared: declared_fmt.clone(),
                    actual: actual_ext,
                });
            }
            _ => {}
        }

        if !doc_path_abs.exists() {
            return Err(RaceParseError::AttestationDocMissing {
                row: row_num,
                path: doc_path_abs.display().to_string(),
            });
        }
        let mut f = File::open(&doc_path_abs)?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 64 * 1024];
        loop {
            let n = f.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let doc_sha = hex_lower(&hasher.finalize());

        // Multi-curator dispute tracking.
        disputes
            .entry((candidate_name.clone(), party.clone()))
            .or_default()
            .push(curator.clone());

        annotations.push(CandidateAnnotation {
            candidate_name,
            party,
            race,
            curator,
            curator_credentials,
            curator_attestation_date,
            source,
            independently_verified,
            attestation_doc_path: doc_path_rel,
            attestation_doc_format: format,
            attestation_doc_sha256: doc_sha,
        });
    }

    // Build provenance summaries.
    let mut curators: HashMap<String, CuratorRecord> = HashMap::new();
    let mut docs: HashMap<String, AttestationDocRecord> = HashMap::new();
    for a in &annotations {
        let entry = curators.entry(a.curator.clone()).or_insert_with(|| CuratorRecord {
            curator: a.curator.clone(),
            curator_credentials: a.curator_credentials.clone(),
            curator_attestation_date: a.curator_attestation_date.clone(),
            n_candidates: 0,
        });
        entry.n_candidates += 1;
        let path_str = a.attestation_doc_path.display().to_string();
        docs.entry(a.attestation_doc_sha256.clone()).or_insert(AttestationDocRecord {
            path: path_str,
            format: a.attestation_doc_format,
            sha256: a.attestation_doc_sha256.clone(),
        });
    }
    let mut curator_list: Vec<CuratorRecord> = curators.into_values().collect();
    curator_list.sort_by(|a, b| a.curator.cmp(&b.curator));
    let mut doc_list: Vec<AttestationDocRecord> = docs.into_values().collect();
    doc_list.sort_by(|a, b| a.sha256.cmp(&b.sha256));

    // Filter disputes: only keep candidates with > 1 distinct curator.
    let dispute_filtered: HashMap<(String, String), Vec<String>> = disputes
        .into_iter()
        .filter_map(|((cn, pty), curators)| {
            let mut uniq = curators;
            uniq.sort();
            uniq.dedup();
            if uniq.len() > 1 {
                Some(((cn, pty), uniq))
            } else {
                None
            }
        })
        .collect();

    let provenance = RaceOfCandidateProvenance {
        schema_version: "race-of-candidate v1".to_string(),
        source_file: csv_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string(),
        source_sha256,
        annotations_independently_verified: all_verified,
        curators: curator_list,
        attestation_documents: doc_list,
    };

    Ok(AnnotationSet {
        annotations,
        disputes: dispute_filtered,
        provenance,
    })
}

// ---------------------------------------------------------------------------
// SHA-256 helpers (duplicated from doctor.rs to avoid a cross-crate utility
// dependency; both are tiny)
// ---------------------------------------------------------------------------

fn sha256_hex(bytes: &[u8]) -> String {
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Build a minimal CSV + matching attestation files in a temp dir.
    fn build_fixture(
        dir: &Path,
        rows: &[(
            &str, &str, &str, &str, &str, &str, &str, bool, &str, &str, // doc body
        )],
    ) -> PathBuf {
        let mut csv = String::from(
            "candidate_name,party,race,curator,curator_credentials,curator_attestation_date,source,independently_verified,attestation_doc_path,attestation_doc_format\n",
        );
        for (name, party, race, curator, creds, date, source, iv, doc_path, doc_fmt) in rows {
            // Write the doc file (use the format string as the extension)
            let body = format!("attestation by {curator} for {name}");
            let abs = dir.join(doc_path);
            if let Some(parent) = abs.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&abs, body).unwrap();
            // Quote any field that might contain a comma so the CSV writer
            // doesn't split on it. Cheap, sufficient for the fixture.
            let q = |s: &str| -> String {
                if s.contains(',') {
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    s.to_string()
                }
            };
            csv.push_str(&format!(
                "{name},{party},{race},{curator},{creds},{date},{source},{iv},{doc_path},{doc_fmt}\n",
                name = q(name),
                party = q(party),
                race = q(race),
                curator = q(curator),
                creds = q(creds),
                date = q(date),
                source = q(source),
                doc_path = q(doc_path),
                doc_fmt = q(doc_fmt),
            ));
        }
        let csv_path = dir.join("race.csv");
        fs::write(&csv_path, csv).unwrap();
        csv_path
    }

    #[test]
    fn test_parse_minimal_csv_succeeds() {
        let tmp = tempfile::TempDir::new().unwrap();
        let csv = build_fixture(
            tmp.path(),
            &[(
                "Adams, J.",
                "DEM",
                "Black",
                "Smith",
                "Ph.D., Demography",
                "2026-04-15",
                "campaign website",
                true,
                "attestation/Adams_Smith.pdf",
                "pdf",
            )],
        );
        let set = parse_race_of_candidate_csv(&csv).expect("parse");
        assert_eq!(set.annotations.len(), 1);
        let a = &set.annotations[0];
        assert_eq!(a.candidate_name, "Adams, J.");
        assert_eq!(a.race, CandidateRace::Black);
        assert!(a.independently_verified);
        assert_eq!(a.attestation_doc_format, AttestationDocFormat::Pdf);
        assert_eq!(a.attestation_doc_sha256.len(), 64);
        assert!(set.provenance.annotations_independently_verified);
        assert_eq!(set.provenance.curators.len(), 1);
        assert_eq!(set.provenance.attestation_documents.len(), 1);
        assert!(set.disputes.is_empty(), "no multi-curator rows -> no disputes");
    }

    #[test]
    fn test_parse_invalid_race_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        let csv = build_fixture(
            tmp.path(),
            &[(
                "X", "DEM", "Greenish", "C", "creds", "2026-01-01", "src", true,
                "doc.pdf", "pdf",
            )],
        );
        match parse_race_of_candidate_csv(&csv) {
            Err(RaceParseError::InvalidRace { value }) => {
                assert_eq!(value, "Greenish");
            }
            other => panic!("expected InvalidRace, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_html_attestation_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        let csv = build_fixture(
            tmp.path(),
            &[(
                "X", "DEM", "Black", "C", "creds", "2026-01-01", "src", true,
                "doc.html", "html",
            )],
        );
        match parse_race_of_candidate_csv(&csv) {
            Err(RaceParseError::InvalidAttestationFormat { .. }) => {}
            other => panic!("expected InvalidAttestationFormat, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_accepts_image_attestation_per_bd_r2() {
        // BD-R2 reconciled union accepts scanned letterhead images.
        let tmp = tempfile::TempDir::new().unwrap();
        let csv = build_fixture(
            tmp.path(),
            &[(
                "Y", "DEM", "Black", "Curator", "creds", "2026-01-01", "src", true,
                "scan.png", "png",
            )],
        );
        let set = parse_race_of_candidate_csv(&csv).expect("png attestation accepted under BD-R2");
        assert_eq!(set.annotations[0].attestation_doc_format, AttestationDocFormat::Png);
        assert!(!set.annotations[0].attestation_doc_format.is_court_strict());
    }

    #[test]
    fn test_parse_format_mismatch_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        // declared pdf but file has .docx extension.
        let csv = build_fixture(
            tmp.path(),
            &[(
                "Z", "DEM", "Black", "C", "creds", "2026-01-01", "src", true,
                "doc.docx", "pdf",
            )],
        );
        match parse_race_of_candidate_csv(&csv) {
            Err(RaceParseError::AttestationFormatMismatch { .. }) => {}
            other => panic!("expected AttestationFormatMismatch, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_missing_attestation_doc_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        // CSV references a path we never write.
        let csv_str = "candidate_name,party,race,curator,curator_credentials,curator_attestation_date,source,independently_verified,attestation_doc_path,attestation_doc_format\n\
            X,DEM,Black,C,creds,2026-01-01,src,true,does_not_exist.pdf,pdf\n";
        let csv_path = tmp.path().join("race.csv");
        fs::write(&csv_path, csv_str).unwrap();
        match parse_race_of_candidate_csv(&csv_path) {
            Err(RaceParseError::AttestationDocMissing { row, .. }) => {
                assert_eq!(row, 2);
            }
            other => panic!("expected AttestationDocMissing, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_independently_verified_false_records_flag() {
        let tmp = tempfile::TempDir::new().unwrap();
        let csv = build_fixture(
            tmp.path(),
            &[(
                "X", "DEM", "Black", "C", "creds", "2026-01-01", "src", false,
                "doc.pdf", "pdf",
            )],
        );
        let set = parse_race_of_candidate_csv(&csv).expect("parse");
        assert!(!set.annotations[0].independently_verified);
        assert!(!set.provenance.annotations_independently_verified);
    }

    #[test]
    fn test_parse_multi_curator_dispute_recorded() {
        let tmp = tempfile::TempDir::new().unwrap();
        let csv = build_fixture(
            tmp.path(),
            &[
                (
                    "Adams", "DEM", "Black", "Smith", "Ph.D.", "2026-01-01",
                    "src1", true, "smith_adams.pdf", "pdf",
                ),
                (
                    "Adams", "DEM", "Hispanic", "Jones", "M.A.", "2026-01-15",
                    "src2", true, "jones_adams.pdf", "pdf",
                ),
            ],
        );
        let set = parse_race_of_candidate_csv(&csv).expect("parse");
        assert_eq!(set.annotations.len(), 2);
        assert_eq!(set.disputes.len(), 1, "Adams should be disputed");
        let key = ("Adams".to_string(), "DEM".to_string());
        let curators = &set.disputes[&key];
        assert!(curators.contains(&"Smith".to_string()));
        assert!(curators.contains(&"Jones".to_string()));
    }

    #[test]
    fn test_parse_missing_required_column_rejected() {
        let tmp = tempfile::TempDir::new().unwrap();
        let csv_path = tmp.path().join("race.csv");
        // missing attestation_doc_format column
        fs::write(
            &csv_path,
            "candidate_name,party,race,curator,curator_credentials,curator_attestation_date,source,independently_verified,attestation_doc_path\n\
             X,DEM,Black,C,creds,2026-01-01,src,true,doc.pdf\n",
        )
        .unwrap();
        match parse_race_of_candidate_csv(&csv_path) {
            Err(RaceParseError::MissingColumn(c)) => {
                assert_eq!(c, "attestation_doc_format");
            }
            other => panic!("expected MissingColumn, got {:?}", other),
        }
    }

    #[test]
    fn test_attestation_doc_sha256_matches_known_content() {
        let tmp = tempfile::TempDir::new().unwrap();
        let csv = build_fixture(
            tmp.path(),
            &[(
                "X", "DEM", "Black", "Curator", "creds", "2026-01-01", "src", true,
                "doc.pdf", "pdf",
            )],
        );
        let set = parse_race_of_candidate_csv(&csv).expect("parse");
        // The fixture writes "attestation by Curator for X" to doc.pdf.
        let expected = sha256_hex(b"attestation by Curator for X");
        assert_eq!(set.annotations[0].attestation_doc_sha256, expected);
    }

    #[test]
    fn test_format_court_strict_only_pdf() {
        assert!(AttestationDocFormat::Pdf.is_court_strict());
        for fmt in [
            AttestationDocFormat::Docx,
            AttestationDocFormat::Md,
            AttestationDocFormat::Txt,
            AttestationDocFormat::Png,
            AttestationDocFormat::Jpg,
            AttestationDocFormat::Tiff,
        ] {
            assert!(
                !fmt.is_court_strict(),
                "{:?} must not be court-strict (BD-R1 lives in Court Reports plan)",
                fmt
            );
        }
    }
}
