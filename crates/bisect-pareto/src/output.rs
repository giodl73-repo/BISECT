//! ParetoResult and NDJSON serialisation.
//!
//! Output format: newline-delimited JSON (NDJSON) with:
//! - One line per Pareto-optimal plan: {"plan":[...],"ec":3124,"d_seats":0.2,"vra_deficit":0.0,...}
//! - Final metadata line with file_sha256 (SHA-256 of all preceding lines, LF-normalised)
//!
//! Per spec §8.2 and §9.

use std::io::{self, Write};
use std::fmt::Write as FmtWrite;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::algorithm::ParetoConfig;
use crate::objectives::Objectives;

/// A single Pareto-optimal plan entry.
#[derive(Debug, Clone)]
pub struct ParetoEntry {
    /// District assignment: plan[t] = district ID (1-based, 1..=k)
    pub plan: Vec<u32>,
    /// Objective values for this plan.
    pub objectives: Objectives,
    /// Always false in the returned frontier (no plan in the frontier is dominated).
    pub dominated: bool,
    /// Generation index in which this plan first appeared on the Pareto front.
    pub generation_found: usize,
}

/// Full result of an NSGA-II run.
#[derive(Debug, Clone)]
pub struct ParetoResult {
    /// Plans on the Pareto front (front rank 0 only).
    pub frontier: Vec<ParetoEntry>,
    /// Algorithm parameters (for audit / reproducibility).
    pub config: ParetoConfig,
    /// Wall-clock runtime in seconds.
    pub runtime_secs: f64,
    /// Version string for the pareto output format.
    pub pareto_version: String,
}

/// Per-plan NDJSON record (serialised).
#[derive(Debug, Serialize, Deserialize)]
struct PlanRecord {
    plan: Vec<u32>,
    ec: f64,
    d_seats: f64,
    vra_deficit: f64,
    dominated: bool,
    generation_found: usize,
}

/// Final metadata record (last line of NDJSON).
#[derive(Debug, Serialize, Deserialize)]
struct MetadataRecord {
    #[serde(rename = "type")]
    record_type: String,
    n_population: usize,
    n_generations: usize,
    pareto_front_size: usize,
    base_seed: u64,
    runtime_secs: f64,
    pareto_version: String,
    d_seats_discrete: bool,
    file_sha256: String,
}

impl ParetoResult {
    /// Write the Pareto result to NDJSON format.
    ///
    /// Line endings: `\n` (LF, 0x0A) — normalised regardless of host platform.
    /// `file_sha256` covers all plan lines (not the metadata line).
    ///
    /// Format:
    /// - One plan line per Pareto-optimal plan
    /// - Final metadata line with file_sha256
    pub fn write_ndjson<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut hasher = Sha256::new();

        // Write plan lines
        for entry in &self.frontier {
            let record = PlanRecord {
                plan: entry.plan.clone(),
                ec: entry.objectives.ec,
                d_seats: entry.objectives.d_seats,
                vra_deficit: entry.objectives.vra_deficit,
                dominated: entry.dominated,
                generation_found: entry.generation_found,
            };
            let json = serde_json::to_string(&record)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            // Always use LF regardless of platform
            let line = format!("{json}\n");
            hasher.update(line.as_bytes());
            writer.write_all(line.as_bytes())?;
        }

        // Compute file_sha256
        let digest = hasher.finalize();
        let mut file_hash = String::with_capacity(64);
        for byte in digest.iter() {
            write!(file_hash, "{byte:02x}").unwrap();
        }

        // Write metadata line
        let meta = MetadataRecord {
            record_type: "metadata".into(),
            n_population: self.config.n_population,
            n_generations: self.config.n_generations,
            pareto_front_size: self.frontier.len(),
            base_seed: self.config.base_seed,
            runtime_secs: self.runtime_secs,
            pareto_version: self.pareto_version.clone(),
            d_seats_discrete: true,
            file_sha256: file_hash,
        };
        let meta_json = serde_json::to_string(&meta)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let meta_line = format!("{meta_json}\n");
        writer.write_all(meta_line.as_bytes())?;

        Ok(())
    }

    /// Parse the metadata record from an NDJSON string (last metadata-type line).
    pub fn read_metadata_from_ndjson(ndjson: &str) -> Option<serde_json::Value> {
        ndjson.lines().rev().find_map(|line| {
            serde_json::from_str::<serde_json::Value>(line).ok().filter(|v| {
                v.get("type").and_then(|t| t.as_str()) == Some("metadata")
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::ParetoConfig;

    fn make_result(n: usize) -> ParetoResult {
        let entry = ParetoEntry {
            plan: vec![1u32, 1, 2, 2],
            objectives: Objectives { ec: 1.0, d_seats: 0.5, vra_deficit: 0.0 },
            dominated: false,
            generation_found: 0,
        };
        ParetoResult {
            frontier: vec![entry; n],
            config: ParetoConfig {
                n_population: 10,
                n_generations: 5,
                base_seed: 42,
                balance_tolerance: 0.005,
            },
            runtime_secs: 1.23,
            pareto_version: "1.0".into(),
        }
    }

    #[test]
    fn ndjson_line_count() {
        let result = make_result(3);
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s.lines().count(), 4, "3 plan lines + 1 metadata = 4");
    }

    #[test]
    fn ndjson_metadata_is_last() {
        let result = make_result(2);
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let last = s.lines().last().unwrap();
        let v: serde_json::Value = serde_json::from_str(last).unwrap();
        assert_eq!(v["type"], "metadata");
    }

    #[test]
    fn ndjson_sha256_is_64_hex() {
        let result = make_result(2);
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let meta = ParetoResult::read_metadata_from_ndjson(&s).unwrap();
        let sha = meta["file_sha256"].as_str().unwrap();
        assert_eq!(sha.len(), 64, "SHA-256 = 64 hex chars");
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()), "must be hex");
    }

    #[test]
    fn ndjson_lf_not_crlf() {
        let result = make_result(3);
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf).unwrap();
        assert!(!buf.windows(2).any(|w| w == b"\r\n"), "must not contain CRLF");
        assert!(buf.iter().any(|&b| b == b'\n'), "must contain LF");
    }

    #[test]
    fn ndjson_deterministic() {
        let result = make_result(4);
        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        result.write_ndjson(&mut buf1).unwrap();
        result.write_ndjson(&mut buf2).unwrap();
        assert_eq!(buf1, buf2, "identical result -> identical NDJSON");
    }

    #[test]
    fn read_metadata_roundtrip() {
        let result = make_result(2);
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let meta = ParetoResult::read_metadata_from_ndjson(&s).unwrap();
        assert_eq!(meta["base_seed"], 42);
        assert_eq!(meta["n_population"], 10);
        assert_eq!(meta["pareto_version"], "1.0");
        assert_eq!(meta["pareto_front_size"], 2);
    }

    #[test]
    fn ndjson_plan_lines_have_required_fields() {
        let result = make_result(2);
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = s.lines().collect();
        // All but last line are plan records
        for line in &lines[..lines.len() - 1] {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(v["plan"].is_array(), "plan must be array");
            assert!(v["ec"].is_number(), "ec must be number");
            assert!(v["d_seats"].is_number(), "d_seats must be number");
            assert!(v["vra_deficit"].is_number(), "vra_deficit must be number");
            assert!(v["dominated"].is_boolean(), "dominated must be boolean");
            assert!(v["generation_found"].is_number(), "generation_found must be number");
        }
    }
}
