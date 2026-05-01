//! Deposition Prep — Plan 7.
//!
//! Tasks shipped in this commit:
//! - Task 1 (consumer): reads `data/whitelist_dependencies.json` (the DAG)
//! - Task 2 (one-shot): `redist depo recompute --param KEY=VALUE` skeleton
//! - Task 5/6 (audit log): `DepoLogWriter` with canonical JSONL + hash chain;
//!   `redist depo verify-log` walks the chain and surfaces any tampering
//!
//! Deferred to next session:
//! - Task 3: IPC abstraction (Unix socket / Windows named pipe)
//! - Task 4: `redist deposition-server` daemon
//! - Task 7: `--enforce-build-commit` + `--case-mode` defaults
//! - Task 9: p99 benchmark methodology
//! - Task 10: `examples/deposition-checklist.ipynb`
//!
//! Spec: `docs/superpowers/specs/2026-04-30-deposition-prep.md`
//! Plan: `docs/superpowers/plans/2026-04-30-deposition-prep.md`

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

// ===========================================================================
// Whitelist DAG (Task 1 consumer)
// ===========================================================================

/// Parsed `data/whitelist_dependencies.json`. Schema: `whitelist-deps v1`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WhitelistDeps {
    pub schema_version: String,
    pub params: Vec<WhitelistParam>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WhitelistParam {
    pub name: String,
    /// One of "float" or "enum".
    #[serde(rename = "type")]
    pub ty: String,
    /// Default value (any JSON scalar — caller validates against `ty`).
    pub default: serde_json::Value,
    /// For float types: [min, max] inclusive.
    #[serde(default)]
    pub range: Option<[f64; 2]>,
    /// For enum types: allowed values.
    #[serde(default)]
    pub values: Option<Vec<serde_json::Value>>,
    /// Downstream computations this param invalidates (caller-named tags).
    pub invalidates: Vec<String>,
    /// Optional narrative-blocking rule. When present and the value matches,
    /// the named items in `blocks` MUST NOT appear in narrative output.
    #[serde(default)]
    pub blocks_narrative_when: Option<serde_json::Value>,
    /// Optional warning rule. When present and the condition fires, surface
    /// the message at runtime.
    #[serde(default)]
    pub warn_when: Option<serde_json::Value>,
    pub owner: String,
}

/// Compile-time-embedded whitelist deps JSON (canonical source for the
/// daemon's parameter validation).
///
/// File lives at `redist/crates/redist-cli/whitelist_dependencies.json`
/// (NOT under `/data/` because the repo's top-level `/data/` is gitignored
/// for raw Census downloads). The human-readable spec is at
/// `docs/parameters/whitelist-dependencies.md`; the CI test (when wired)
/// asserts the markdown table and this JSON declare the same parameter set.
const EMBEDDED_WHITELIST_JSON: &str =
    include_str!("../whitelist_dependencies.json");

/// Parse the embedded whitelist deps. Cached via `OnceLock`.
pub fn whitelist_deps() -> &'static WhitelistDeps {
    use std::sync::OnceLock;
    static CACHE: OnceLock<WhitelistDeps> = OnceLock::new();
    CACHE.get_or_init(|| {
        serde_json::from_str(EMBEDDED_WHITELIST_JSON)
            .expect("[INTERNAL] data/whitelist_dependencies.json must parse as WhitelistDeps")
    })
}

/// Look up a parameter by name. Returns `None` if the name isn't in the
/// whitelist (caller surfaces an `[INPUT]` error with the helpful message).
pub fn lookup_param(name: &str) -> Option<&'static WhitelistParam> {
    whitelist_deps().params.iter().find(|p| p.name == name)
}

/// SHA-256 of the embedded whitelist JSON. Recorded into `whatif-manifest v1`
/// outputs so a future reader knows which compat ranges were active at the
/// time of recompute.
pub fn whitelist_compat_sha256() -> String {
    let mut hasher = Sha256::new();
    hasher.update(EMBEDDED_WHITELIST_JSON.as_bytes());
    hex_lower(&hasher.finalize())
}

#[derive(Debug, Error)]
pub enum DepoError {
    #[error("[INPUT] --param argument '{0}' is not 'KEY=VALUE'")]
    BadParamKv(String),
    #[error("[INPUT] parameter '{name}' not in whitelist. Allowed: {allowed:?}. See docs/parameters/whitelist-dependencies.md.")]
    UnknownParam { name: String, allowed: Vec<String> },
    #[error("[INPUT] parameter '{name}' value '{value}' is not a valid {ty}")]
    BadParamValue { name: String, value: String, ty: String },
    #[error("[INPUT] parameter '{name}' value {value} out of range [{lo}, {hi}]")]
    OutOfRange { name: String, value: f64, lo: f64, hi: f64 },
    #[error("[INPUT] parameter '{name}' value '{value}' not in allowed enum {allowed:?}")]
    BadEnum { name: String, value: String, allowed: Vec<String> },
    #[error("[INTERNAL] {0}")]
    Internal(String),
}

/// Parse a `--param KEY=VALUE` string into a (name, value) pair, validated
/// against the whitelist. Returns the canonical `serde_json::Value` form.
pub fn parse_param_kv(s: &str) -> Result<(String, serde_json::Value), DepoError> {
    let (name, raw_value) = s
        .split_once('=')
        .ok_or_else(|| DepoError::BadParamKv(s.to_string()))?;
    let p = lookup_param(name).ok_or_else(|| DepoError::UnknownParam {
        name: name.to_string(),
        allowed: whitelist_deps().params.iter().map(|p| p.name.clone()).collect(),
    })?;
    match p.ty.as_str() {
        "float" => {
            let v: f64 = raw_value.parse().map_err(|_| DepoError::BadParamValue {
                name: name.into(),
                value: raw_value.into(),
                ty: "float".into(),
            })?;
            if let Some([lo, hi]) = p.range {
                if v < lo || v > hi {
                    return Err(DepoError::OutOfRange {
                        name: name.into(),
                        value: v,
                        lo,
                        hi,
                    });
                }
            }
            Ok((name.to_string(), serde_json::json!(v)))
        }
        "enum" => {
            let allowed: Vec<String> = p
                .values
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            if !allowed.iter().any(|a| a == raw_value) {
                return Err(DepoError::BadEnum {
                    name: name.into(),
                    value: raw_value.into(),
                    allowed,
                });
            }
            Ok((name.to_string(), serde_json::json!(raw_value)))
        }
        other => Err(DepoError::Internal(format!(
            "unknown param type '{other}' for '{name}'"
        ))),
    }
}

/// SHA-256 of the canonical-JSON serialization of a sorted-key map of
/// overrides. Used as the `param_hash` directory suffix in the what-if
/// output path.
pub fn overrides_hash(overrides: &BTreeMap<String, serde_json::Value>) -> String {
    let canonical = serde_json::to_string(overrides).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hex_lower(&hasher.finalize())
}

// ===========================================================================
// `whatif-manifest v1` output (Task 2 consumer)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatifManifest {
    pub schema_version: String,
    pub parent_plan_label: String,
    pub parent_plan_manifest_sha256: String,
    pub parent_report_pdf_sha256: Option<String>,
    pub overrides: BTreeMap<String, serde_json::Value>,
    pub overrides_hash: String,
    /// Path to this what-if directory, RELATIVE to repo root (M-04 / PP-31).
    pub override_path_relative: String,
    pub redist_version: String,
    pub redist_build_commit: String,
    pub redist_build_commit_short: String,
    pub rustc_version: String,
    /// SHA-256 of the embedded `data/whitelist_dependencies.json` at the time
    /// of the recompute (C-05 spirit applied to the depo whitelist).
    pub whitelist_compat_sha256: String,
    /// ISO-8601 UTC timestamp; pinned via SOURCE_DATE_EPOCH for reproducibility.
    pub generated_at: String,
    pub note: Option<String>,
}

// ===========================================================================
// Canonical JSONL log + hash chain (Task 5)
// ===========================================================================

/// One entry in `deposition_log_{date}.jsonl`. Serialized via
/// `to_canonical_json` (sorted keys, ryu floats, ISO-8601 Z timestamps).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub seq: u64,
    /// ISO-8601 UTC timestamp with explicit `Z` (e.g., `2026-04-30T14:22:08.123Z`).
    pub timestamp: String,
    /// Operation name (e.g., `"eval"`, `"sweep"`, `"shutdown"`).
    pub op: String,
    /// Parameter overrides for this op (sorted-key map).
    #[serde(default)]
    pub params: BTreeMap<String, serde_json::Value>,
    /// Analyzer types requested for this op.
    #[serde(default)]
    pub types: Vec<String>,
    /// Result summary (small JSON object — full results live in the
    /// what-if output dir referenced by `result_path`).
    #[serde(default)]
    pub result_summary: serde_json::Value,
    /// Optional pointer to the what-if output directory (relative to repo root).
    #[serde(default)]
    pub result_path: Option<String>,
    /// Wall-clock for this op, in milliseconds.
    pub elapsed_ms: u64,
    /// Build commit at the time of the op.
    pub build_commit: String,
    /// SHA-256 of the previous entry's complete line bytes (excluding the
    /// trailing newline). First entry uses `"GENESIS"`.
    pub prev_sha256: String,
}

pub const GENESIS: &str = "GENESIS";

/// Serialize a `LogEntry` to canonical JSON: sorted keys + tight separators
/// + (Rust-default) f64 formatting. The line is ASCII-only (PP-34) and ends
/// with a single trailing `\n` when written to the JSONL file.
pub fn to_canonical_json(entry: &LogEntry) -> Result<String, DepoError> {
    let value = serde_json::to_value(entry)
        .map_err(|e| DepoError::Internal(format!("serialize log entry: {e}")))?;
    canonicalize_json(&value)
}

/// Canonicalize a `serde_json::Value` to its sorted-key, no-whitespace JSON
/// representation. `BTreeMap` would give us this for free during serialization,
/// but we also accept `serde_json::Value` from external sources, so we walk
/// the tree.
pub fn canonicalize_json(value: &serde_json::Value) -> Result<String, DepoError> {
    let mut out = String::new();
    write_canonical(&mut out, value);
    Ok(out)
}

fn write_canonical(out: &mut String, v: &serde_json::Value) {
    use serde_json::Value;
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => {
            out.push_str(&serde_json::to_string(s).unwrap_or_else(|_| String::from("\"\"")));
        }
        Value::Array(arr) => {
            out.push('[');
            for (i, elem) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(out, elem);
            }
            out.push(']');
        }
        Value::Object(map) => {
            // Sort keys lexicographically.
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(k).unwrap_or_else(|_| String::from("\"\"")));
                out.push(':');
                write_canonical(out, &map[*k]);
            }
            out.push('}');
        }
    }
}

/// SHA-256 of a single line's bytes (no trailing newline).
pub fn sha256_of_line(line: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(line.as_bytes());
    hex_lower(&hasher.finalize())
}

/// Append-mode log writer. Single-threaded by design; the daemon (when
/// shipped) feeds it via mpsc.
pub struct DepoLogWriter {
    path: PathBuf,
    next_seq: u64,
    prev_sha: String,
    /// Buffer for total-entries / latest-entry-sha used by the sidecar manifest.
    latest_entry_sha256: String,
    total_entries: u64,
    /// Manifest sidecar path (deposition_log_{date}.manifest.json next to the JSONL).
    manifest_path: PathBuf,
    /// Build commit captured at writer construction; recorded in every entry.
    build_commit: String,
    /// Plan label + plan manifest SHA recorded in the sidecar.
    plan_label: String,
    plan_manifest_sha256: String,
    /// `started_at` ISO-8601 captured at writer construction.
    started_at: String,
}

impl DepoLogWriter {
    /// Open or create the log + sidecar manifest. Reads any existing JSONL to
    /// recover `next_seq` + `prev_sha` (so a daemon restart appends cleanly).
    ///
    /// `now_iso8601_fn` is a closure so tests can pin the timestamp.
    pub fn open(
        log_path: PathBuf,
        manifest_path: PathBuf,
        plan_label: String,
        plan_manifest_sha256: String,
        build_commit: String,
        now_iso8601_fn: impl Fn() -> String,
    ) -> Result<Self, DepoError> {
        let started_at = now_iso8601_fn();
        let (next_seq, prev_sha, total_entries, latest_sha) = if log_path.exists() {
            recover_log_state(&log_path)?
        } else {
            (0u64, GENESIS.to_string(), 0u64, GENESIS.to_string())
        };
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DepoError::Internal(format!("create log dir {}: {e}", parent.display()))
            })?;
        }
        Ok(DepoLogWriter {
            path: log_path,
            next_seq,
            prev_sha,
            latest_entry_sha256: latest_sha,
            total_entries,
            manifest_path,
            build_commit,
            plan_label,
            plan_manifest_sha256,
            started_at,
        })
    }

    /// Append one entry (caller supplies op/params/types/result_summary;
    /// writer fills seq, prev_sha, build_commit, timestamp via `now_fn`).
    pub fn append(
        &mut self,
        op: &str,
        params: BTreeMap<String, serde_json::Value>,
        types: Vec<String>,
        result_summary: serde_json::Value,
        result_path: Option<String>,
        elapsed_ms: u64,
        now_iso8601_fn: impl Fn() -> String,
    ) -> Result<u64, DepoError> {
        let entry = LogEntry {
            seq: self.next_seq,
            timestamp: now_iso8601_fn(),
            op: op.to_string(),
            params,
            types,
            result_summary,
            result_path,
            elapsed_ms,
            build_commit: self.build_commit.clone(),
            prev_sha256: self.prev_sha.clone(),
        };
        let line = to_canonical_json(&entry)?;
        let sha = sha256_of_line(&line);
        // Append line + LF. Open with append + create.
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| DepoError::Internal(format!("open log {}: {e}", self.path.display())))?;
        writeln!(&mut f, "{line}").map_err(|e| DepoError::Internal(format!("write log: {e}")))?;
        f.sync_all()
            .map_err(|e| DepoError::Internal(format!("fsync log: {e}")))?;
        let seq = self.next_seq;
        self.next_seq += 1;
        self.prev_sha = sha.clone();
        self.latest_entry_sha256 = sha;
        self.total_entries += 1;
        // Update sidecar manifest atomically.
        self.write_sidecar(None)?;
        Ok(seq)
    }

    /// Write the sidecar manifest. `closed_at` is `None` while the writer is
    /// still active; `Some(ts)` on graceful shutdown.
    fn write_sidecar(&self, closed_at: Option<String>) -> Result<(), DepoError> {
        let final_sha = if closed_at.is_some() {
            // Compute SHA-256 of the entire log file on shutdown.
            std::fs::read(&self.path)
                .ok()
                .map(|b| {
                    let mut h = Sha256::new();
                    h.update(&b);
                    hex_lower(&h.finalize())
                })
        } else {
            None
        };
        let sidecar = LogSidecarManifest {
            schema_version: "depo-log v1".to_string(),
            log_path: self
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            plan_label: self.plan_label.clone(),
            plan_manifest_sha256: self.plan_manifest_sha256.clone(),
            build_commit: self.build_commit.clone(),
            started_at: self.started_at.clone(),
            closed_at,
            total_entries: self.total_entries,
            latest_entry_sha256: self.latest_entry_sha256.clone(),
            final_sha256: final_sha,
        };
        let s = serde_json::to_string_pretty(&sidecar)
            .map_err(|e| DepoError::Internal(format!("serialize sidecar: {e}")))?;
        // Atomic write: tmp + rename.
        let tmp = self.manifest_path.with_extension("tmp");
        std::fs::write(&tmp, s.as_bytes())
            .map_err(|e| DepoError::Internal(format!("write sidecar tmp: {e}")))?;
        std::fs::rename(&tmp, &self.manifest_path)
            .map_err(|e| DepoError::Internal(format!("rename sidecar: {e}")))?;
        Ok(())
    }

    /// Graceful shutdown: write the final sidecar with `closed_at` + `final_sha256`.
    pub fn close(self, closed_at: String) -> Result<(), DepoError> {
        self.write_sidecar(Some(closed_at))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSidecarManifest {
    pub schema_version: String,
    pub log_path: String,
    pub plan_label: String,
    pub plan_manifest_sha256: String,
    pub build_commit: String,
    pub started_at: String,
    pub closed_at: Option<String>,
    pub total_entries: u64,
    pub latest_entry_sha256: String,
    pub final_sha256: Option<String>,
}

/// Walk the JSONL file and recover `(next_seq, prev_sha, total_entries, latest_sha)`.
fn recover_log_state(path: &Path) -> Result<(u64, String, u64, String), DepoError> {
    let bytes = std::fs::read(path)
        .map_err(|e| DepoError::Internal(format!("read log {}: {e}", path.display())))?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|e| DepoError::Internal(format!("log not UTF-8: {e}")))?;
    let mut next_seq = 0u64;
    let mut prev_sha = GENESIS.to_string();
    let mut total = 0u64;
    let mut latest_sha = GENESIS.to_string();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let entry: LogEntry = serde_json::from_str(line)
            .map_err(|e| DepoError::Internal(format!("parse log line: {e}")))?;
        next_seq = entry.seq + 1;
        prev_sha = sha256_of_line(line);
        latest_sha = prev_sha.clone();
        total += 1;
    }
    Ok((next_seq, prev_sha, total, latest_sha))
}

// ===========================================================================
// Verify-log (Task 6)
// ===========================================================================

/// Result of `redist depo verify-log <path>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyOutcome {
    pub entries_verified: u64,
    pub valid: bool,
    /// On the first divergence, the offending seq and a human-readable reason.
    pub first_failure: Option<VerifyFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyFailure {
    pub seq: u64,
    pub byte_offset: usize,
    pub reason: String,
}

/// Verify the hash chain of a `deposition_log_{date}.jsonl` file.
pub fn verify_log_file(path: &Path) -> Result<VerifyOutcome, DepoError> {
    let bytes = std::fs::read(path)
        .map_err(|e| DepoError::Internal(format!("read log {}: {e}", path.display())))?;
    verify_log_bytes(&bytes)
}

/// Verify a log given its raw bytes. Test entry point.
pub fn verify_log_bytes(bytes: &[u8]) -> Result<VerifyOutcome, DepoError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|e| DepoError::Internal(format!("log not UTF-8: {e}")))?;
    let mut entries = 0u64;
    let mut prev_sha = GENESIS.to_string();
    let mut byte_offset: usize = 0;
    let mut expected_seq: u64 = 0;
    for line in text.split_terminator('\n') {
        if line.is_empty() {
            byte_offset += 1; // for the \n
            continue;
        }
        let line_len_with_lf = line.len() + 1;
        let entry: LogEntry = serde_json::from_str(line).map_err(|e| {
            DepoError::Internal(format!("line at byte {byte_offset}: parse: {e}"))
        })?;
        // Check seq is strictly monotonic from 0.
        if entry.seq != expected_seq {
            return Ok(VerifyOutcome {
                entries_verified: entries,
                valid: false,
                first_failure: Some(VerifyFailure {
                    seq: entry.seq,
                    byte_offset,
                    reason: format!("seq mismatch: expected {expected_seq}, got {}", entry.seq),
                }),
            });
        }
        // Check prev_sha matches our running expectation.
        if entry.prev_sha256 != prev_sha {
            return Ok(VerifyOutcome {
                entries_verified: entries,
                valid: false,
                first_failure: Some(VerifyFailure {
                    seq: entry.seq,
                    byte_offset,
                    reason: format!(
                        "prev_sha256 mismatch: expected {prev_sha}, got {}",
                        entry.prev_sha256
                    ),
                }),
            });
        }
        // Advance: the recorded prev_sha for the NEXT entry is sha256(this line).
        prev_sha = sha256_of_line(line);
        entries += 1;
        expected_seq += 1;
        byte_offset += line_len_with_lf;
    }
    Ok(VerifyOutcome {
        entries_verified: entries,
        valid: true,
        first_failure: None,
    })
}

// ===========================================================================
// Helpers
// ===========================================================================

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── Whitelist DAG (Task 1 consumer) ──────────────────────────────────────

    #[test]
    fn test_whitelist_deps_loads_and_has_8_params() {
        let deps = whitelist_deps();
        assert_eq!(deps.schema_version, "whitelist-deps v1");
        assert_eq!(deps.params.len(), 8, "whitelist must have all 8 params per the spec");
    }

    #[test]
    fn test_whitelist_deps_includes_required_names() {
        let deps = whitelist_deps();
        let names: Vec<&str> = deps.params.iter().map(|p| p.name.as_str()).collect();
        for n in &[
            "leaning_threshold",
            "close_call_band",
            "vra_min_bvap",
            "bloc_p_value_method",
            "bloc_robust_se_type",
            "bloc_cluster_unit",
            "compactness_metric",
            "partisan_efficiency_threshold",
        ] {
            assert!(names.contains(n), "whitelist missing param {n}: {:?}", names);
        }
    }

    #[test]
    fn test_whitelist_compat_sha256_is_64_hex_chars() {
        let s = whitelist_compat_sha256();
        assert_eq!(s.len(), 64);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_lookup_param_known() {
        let p = lookup_param("leaning_threshold").unwrap();
        assert_eq!(p.ty, "float");
        assert_eq!(p.range, Some([0.0, 1.0]));
    }

    #[test]
    fn test_lookup_param_unknown() {
        assert!(lookup_param("not_a_real_param").is_none());
    }

    // ── parse_param_kv ───────────────────────────────────────────────────────

    #[test]
    fn test_parse_param_kv_float_in_range() {
        let (k, v) = parse_param_kv("leaning_threshold=0.53").unwrap();
        assert_eq!(k, "leaning_threshold");
        assert_eq!(v.as_f64(), Some(0.53));
    }

    #[test]
    fn test_parse_param_kv_enum_valid() {
        let (k, v) = parse_param_kv("bloc_p_value_method=holm").unwrap();
        assert_eq!(k, "bloc_p_value_method");
        assert_eq!(v.as_str(), Some("holm"));
    }

    #[test]
    fn test_parse_param_kv_no_equals_rejected() {
        assert!(matches!(
            parse_param_kv("leaning_threshold0.55"),
            Err(DepoError::BadParamKv(_))
        ));
    }

    #[test]
    fn test_parse_param_kv_unknown_param_lists_allowed() {
        let err = parse_param_kv("arbitrary_key=value").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not in whitelist"), "{msg}");
        assert!(msg.contains("leaning_threshold"), "must list allowed params: {msg}");
        assert!(
            msg.contains("docs/parameters/whitelist-dependencies.md"),
            "must point at the doc: {msg}"
        );
    }

    #[test]
    fn test_parse_param_kv_float_out_of_range() {
        let err = parse_param_kv("leaning_threshold=1.5").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("out of range"), "{msg}");
    }

    #[test]
    fn test_parse_param_kv_float_unparseable() {
        assert!(matches!(
            parse_param_kv("leaning_threshold=abc"),
            Err(DepoError::BadParamValue { .. })
        ));
    }

    #[test]
    fn test_parse_param_kv_enum_invalid_value() {
        let err = parse_param_kv("bloc_p_value_method=yolo").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not in allowed enum"), "{msg}");
        assert!(msg.contains("holm"), "must list allowed enums: {msg}");
    }

    #[test]
    fn test_parse_param_kv_bloc_cluster_unit_county_passes() {
        let (k, v) = parse_param_kv("bloc_cluster_unit=county").unwrap();
        assert_eq!(k, "bloc_cluster_unit");
        assert_eq!(v.as_str(), Some("county"));
    }

    // ── overrides_hash ───────────────────────────────────────────────────────

    #[test]
    fn test_overrides_hash_deterministic() {
        let mut a = BTreeMap::new();
        a.insert("leaning_threshold".to_string(), serde_json::json!(0.53));
        a.insert("vra_min_bvap".to_string(), serde_json::json!(0.50));
        let h1 = overrides_hash(&a);
        let h2 = overrides_hash(&a);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_overrides_hash_order_independent() {
        // BTreeMap means key insertion order doesn't matter; same content -> same hash.
        let mut a = BTreeMap::new();
        a.insert("a".to_string(), serde_json::json!(1.0));
        a.insert("b".to_string(), serde_json::json!(2.0));
        let mut b = BTreeMap::new();
        b.insert("b".to_string(), serde_json::json!(2.0));
        b.insert("a".to_string(), serde_json::json!(1.0));
        assert_eq!(overrides_hash(&a), overrides_hash(&b));
    }

    #[test]
    fn test_overrides_hash_changes_on_value_change() {
        let mut a = BTreeMap::new();
        a.insert("leaning_threshold".to_string(), serde_json::json!(0.55));
        let h1 = overrides_hash(&a);
        a.insert("leaning_threshold".to_string(), serde_json::json!(0.53));
        let h2 = overrides_hash(&a);
        assert_ne!(h1, h2);
    }

    // ── canonical JSON ───────────────────────────────────────────────────────

    #[test]
    fn test_canonicalize_json_sorted_keys() {
        let v = serde_json::json!({
            "z": 1,
            "a": 2,
            "m": 3,
        });
        let s = canonicalize_json(&v).unwrap();
        // Keys must appear in alphabetical order.
        let pos_a = s.find("\"a\"").unwrap();
        let pos_m = s.find("\"m\"").unwrap();
        let pos_z = s.find("\"z\"").unwrap();
        assert!(pos_a < pos_m && pos_m < pos_z, "keys must be sorted: {s}");
    }

    #[test]
    fn test_canonicalize_json_no_whitespace() {
        let v = serde_json::json!({"a": [1, 2, 3]});
        let s = canonicalize_json(&v).unwrap();
        assert!(!s.contains(' '), "canonical form must omit whitespace: {s}");
        assert!(!s.contains('\n'), "canonical form must omit newlines: {s}");
    }

    #[test]
    fn test_canonicalize_json_byte_stable() {
        let v = serde_json::json!({"b": [4, {"y": 1, "x": 2}], "a": "hi"});
        let s1 = canonicalize_json(&v).unwrap();
        let s2 = canonicalize_json(&v).unwrap();
        assert_eq!(s1, s2);
    }

    // ── Log writer + verifier (Tasks 5 + 6) ──────────────────────────────────

    fn pinned_clock() -> impl Fn() -> String {
        let mut counter = std::cell::Cell::new(0u64);
        move || {
            let n = counter.get();
            counter.set(n + 1);
            format!("2026-04-30T12:00:{:02}.000Z", n)
        }
    }

    fn make_writer(tmp: &TempDir) -> DepoLogWriter {
        let log = tmp.path().join("deposition_log_2026-04-30.jsonl");
        let mani = tmp.path().join("deposition_log_2026-04-30.manifest.json");
        DepoLogWriter::open(
            log,
            mani,
            "vt_2020_test".to_string(),
            "a".repeat(64),
            "deadbeef0123".to_string(),
            pinned_clock(),
        )
        .unwrap()
    }

    #[test]
    fn test_log_writer_first_entry_uses_genesis_prev_sha() {
        let tmp = TempDir::new().unwrap();
        let mut w = make_writer(&tmp);
        let seq = w
            .append(
                "eval",
                BTreeMap::new(),
                vec!["partisan".into()],
                serde_json::json!({"mm_count": 2}),
                None,
                10,
                pinned_clock(),
            )
            .unwrap();
        assert_eq!(seq, 0);
        let log_text = std::fs::read_to_string(&w.path).unwrap();
        assert!(log_text.contains(GENESIS), "first entry must reference GENESIS prev_sha");
    }

    #[test]
    fn test_log_writer_chains_consecutive_entries() {
        let tmp = TempDir::new().unwrap();
        let mut w = make_writer(&tmp);
        for _ in 0..5 {
            w.append(
                "eval",
                BTreeMap::new(),
                vec![],
                serde_json::json!({}),
                None,
                1,
                pinned_clock(),
            )
            .unwrap();
        }
        let outcome = verify_log_file(&w.path).unwrap();
        assert_eq!(outcome.entries_verified, 5);
        assert!(outcome.valid);
    }

    #[test]
    fn test_log_writer_recovers_state_on_reopen() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("deposition_log_2026-04-30.jsonl");
        let mani = tmp.path().join("deposition_log_2026-04-30.manifest.json");
        // First session: append 3 entries.
        {
            let mut w = DepoLogWriter::open(
                log.clone(),
                mani.clone(),
                "p".into(),
                "x".repeat(64),
                "abc".into(),
                pinned_clock(),
            )
            .unwrap();
            for _ in 0..3 {
                w.append("eval", BTreeMap::new(), vec![], serde_json::json!({}), None, 1, pinned_clock())
                    .unwrap();
            }
        }
        // Second session: reopen and append; new entry must have seq=3 and
        // prev_sha matching the chain.
        let mut w2 = DepoLogWriter::open(
            log.clone(),
            mani.clone(),
            "p".into(),
            "x".repeat(64),
            "abc".into(),
            pinned_clock(),
        )
        .unwrap();
        let seq = w2
            .append("eval", BTreeMap::new(), vec![], serde_json::json!({}), None, 1, pinned_clock())
            .unwrap();
        assert_eq!(seq, 3);
        let outcome = verify_log_file(&log).unwrap();
        assert_eq!(outcome.entries_verified, 4);
        assert!(outcome.valid);
    }

    #[test]
    fn test_verify_log_detects_single_byte_tamper() {
        let tmp = TempDir::new().unwrap();
        let mut w = make_writer(&tmp);
        for _ in 0..5 {
            w.append("eval", BTreeMap::new(), vec![], serde_json::json!({}), None, 1, pinned_clock())
                .unwrap();
        }
        let path = w.path.clone();
        // Tamper: flip a single character in entry 2's `params` field.
        let mut text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        // Modify line 2 (index 2, which is seq=2): replace "{}" with "{ }"
        // Actually, simpler: replace the FIRST occurrence of "elapsed_ms":1
        // with "elapsed_ms":2 in the second line.
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        let line2 = &new_lines[2];
        let mutated = line2.replacen("\"elapsed_ms\":1,", "\"elapsed_ms\":2,", 1);
        assert_ne!(mutated, *line2, "tamper must change the line");
        new_lines[2] = mutated;
        text = new_lines.join("\n") + "\n";
        std::fs::write(&path, text).unwrap();

        let outcome = verify_log_file(&path).unwrap();
        assert!(!outcome.valid);
        let f = outcome.first_failure.unwrap();
        // The mutation breaks line 2's bytes; the failure surfaces at the NEXT
        // entry whose recorded prev_sha doesn't match the recomputed sha of
        // the (mutated) line 2. So failure at seq=3.
        assert_eq!(f.seq, 3, "first divergence at the entry whose prev_sha no longer matches");
        assert!(f.reason.contains("prev_sha256 mismatch"), "{}", f.reason);
    }

    #[test]
    fn test_verify_log_detects_seq_skip() {
        let tmp = TempDir::new().unwrap();
        let mut w = make_writer(&tmp);
        for _ in 0..3 {
            w.append("eval", BTreeMap::new(), vec![], serde_json::json!({}), None, 1, pinned_clock())
                .unwrap();
        }
        let path = w.path.clone();
        // Delete the middle line (seq=1).
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        let kept = format!("{}\n{}\n", lines[0], lines[2]);
        std::fs::write(&path, kept).unwrap();
        let outcome = verify_log_file(&path).unwrap();
        assert!(!outcome.valid);
        let f = outcome.first_failure.unwrap();
        // seq mismatch (line 2 has seq=2 but expected_seq=1).
        assert_eq!(f.seq, 2);
        assert!(f.reason.contains("seq mismatch"), "{}", f.reason);
    }

    #[test]
    fn test_log_writer_close_writes_final_sha256() {
        let tmp = TempDir::new().unwrap();
        let mut w = make_writer(&tmp);
        for _ in 0..2 {
            w.append("eval", BTreeMap::new(), vec![], serde_json::json!({}), None, 1, pinned_clock())
                .unwrap();
        }
        let mani_path = w.manifest_path.clone();
        w.close("2026-04-30T13:00:00Z".into()).unwrap();
        let mani: LogSidecarManifest = serde_json::from_slice(&std::fs::read(&mani_path).unwrap()).unwrap();
        assert_eq!(mani.total_entries, 2);
        assert_eq!(mani.closed_at.as_deref(), Some("2026-04-30T13:00:00Z"));
        assert!(mani.final_sha256.is_some());
        assert_eq!(mani.final_sha256.unwrap().len(), 64);
    }

    #[test]
    fn test_log_writer_sidecar_updates_on_each_append() {
        let tmp = TempDir::new().unwrap();
        let mut w = make_writer(&tmp);
        let mani_path = w.manifest_path.clone();
        w.append("eval", BTreeMap::new(), vec![], serde_json::json!({}), None, 1, pinned_clock()).unwrap();
        let mani1: LogSidecarManifest =
            serde_json::from_slice(&std::fs::read(&mani_path).unwrap()).unwrap();
        assert_eq!(mani1.total_entries, 1);
        w.append("eval", BTreeMap::new(), vec![], serde_json::json!({}), None, 1, pinned_clock()).unwrap();
        let mani2: LogSidecarManifest =
            serde_json::from_slice(&std::fs::read(&mani_path).unwrap()).unwrap();
        assert_eq!(mani2.total_entries, 2);
        assert_ne!(mani1.latest_entry_sha256, mani2.latest_entry_sha256);
    }

    #[test]
    fn test_verify_log_empty_file_is_valid() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("empty.jsonl");
        std::fs::write(&path, "").unwrap();
        let outcome = verify_log_file(&path).unwrap();
        assert_eq!(outcome.entries_verified, 0);
        assert!(outcome.valid);
    }

    #[test]
    fn test_verify_log_bytes_directly() {
        // The bytes-based entry point lets tests construct hand-crafted logs
        // without hitting the filesystem.
        let tmp = TempDir::new().unwrap();
        let mut w = make_writer(&tmp);
        w.append("eval", BTreeMap::new(), vec![], serde_json::json!({}), None, 1, pinned_clock()).unwrap();
        let bytes = std::fs::read(&w.path).unwrap();
        let outcome = verify_log_bytes(&bytes).unwrap();
        assert!(outcome.valid);
        assert_eq!(outcome.entries_verified, 1);
    }
}
