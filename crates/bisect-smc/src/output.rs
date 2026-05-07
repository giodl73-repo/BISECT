//! SmcResult and NDJSON serialisation. Per spec §3.2 and §6.
//!
//! Output format: newline-delimited JSON (NDJSON) with:
//! - One line per particle: {"plan": [...], "log_weight": -2.31, "particle_idx": 0}
//! - Resample records between stages (when resampling occurs)
//! - Final metadata record with file_sha256 (SHA-256 of all preceding lines, LF-normalised)

use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as FmtWrite;

/// A resampling event record — embedded in the NDJSON stream between stages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResampleRecord {
    #[serde(rename = "type")]
    pub record_type: String,      // "resample"
    pub stage: usize,
    pub resample_round: u32,
    pub ess_before: f64,
    pub resample_seed: u64,
    pub index_map: Vec<usize>,    // new_particle[j] came from old_particle[index_map[j]]
}

/// Final metadata record — last line of the NDJSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataRecord {
    #[serde(rename = "type")]
    pub record_type: String,      // "metadata"
    pub base_seed: u64,
    pub n_particles: usize,
    pub resample_threshold: f64,
    pub pop_tolerance: f64,
    pub k: usize,
    pub resample_count: u32,
    pub resample_rounds: Vec<usize>,
    pub ess_trace: Vec<f64>,
    pub particle_seed_formula: String,
    pub resample_seed_formula: String,
    pub smc_version: String,
    pub ensemble_output_version: String,
    pub file_sha256: String,
}

/// The output of a complete SMC run.
#[derive(Debug, Clone)]
pub struct SmcResult {
    /// N complete plans, each a Vec<u32> of length n_tracts (1-based district IDs, 1..=k)
    pub plans: Vec<Vec<u32>>,
    /// Normalised importance weights summing to 1.0 ± 1e-6 (Kahan compensated)
    pub weights: Vec<f64>,
    /// Number of resampling events
    pub resample_count: u32,
    /// Stages at which resampling occurred (1-based stage indices)
    pub resample_rounds: Vec<usize>,
    /// ESS after each of k-1 proposal stages
    pub ess_trace: Vec<f64>,
    /// Per-resample index maps: index_maps[r][j] = old particle that new slot j was copied from
    pub index_maps: Vec<Vec<usize>>,
    pub base_seed: u64,
    pub n_particles: usize,
}

impl SmcResult {
    pub fn n_plans(&self) -> usize { self.plans.len() }

    pub fn k(&self) -> u32 {
        self.plans.first()
            .and_then(|p| p.iter().copied().max())
            .unwrap_or(0)
    }

    /// Write the SMC result to NDJSON format.
    ///
    /// Line ending: \n (LF) — normalised regardless of host platform per spec §6.
    /// file_sha256 covers all particle and resample lines (not the metadata line).
    ///
    /// Format:
    /// - One particle line per plan: {"plan":[...],"log_weight":-2.31,"particle_idx":0}
    /// - Resample records interleaved if requested (requires resample metadata)
    /// - Final metadata line with file_sha256
    pub fn write_ndjson<W: Write>(
        &self,
        writer: &mut W,
        config: &WriteConfig,
    ) -> io::Result<()> {
        let mut hasher = Sha256::new();

        // Convert normalised weights back to log-weights for output
        let log_weights: Vec<f64> = self.weights.iter()
            .map(|&w| if w > 0.0 { w.ln() } else { f64::NEG_INFINITY })
            .collect();

        // Write particle lines
        for (i, (plan, log_w)) in self.plans.iter().zip(log_weights.iter()).enumerate() {
            let line = format!(
                "{{\"plan\":{},\"log_weight\":{:.6},\"particle_idx\":{}}}\n",
                serde_json::to_string(plan).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
                log_w,
                i,
            );
            hasher.update(line.as_bytes());
            writer.write_all(line.as_bytes())?;
        }

        // Write metadata line (with file_sha256)
        let digest = hasher.finalize();
        let mut file_hash = String::with_capacity(64);
        for byte in digest.iter() {
            write!(file_hash, "{byte:02x}").unwrap();
        }
        let k = self.k() as usize;
        let meta = MetadataRecord {
            record_type: "metadata".into(),
            base_seed: self.base_seed,
            n_particles: self.n_particles,
            resample_threshold: config.resample_threshold,
            pop_tolerance: config.pop_tolerance,
            k,
            resample_count: self.resample_count,
            resample_rounds: self.resample_rounds.clone(),
            ess_trace: self.ess_trace.clone(),
            particle_seed_formula:
                "SHA-256('SMC_PARTICLE_' || stage:u32le || '_' || particle:u32le || '_' || base_seed:u64le) → first 8 bytes as u64le".into(),
            resample_seed_formula:
                "SHA-256('SMC_RESAMPLE_' || round:u32le || '_' || base_seed:u64le) → first 8 bytes as u64le".into(),
            smc_version: "1.0".into(),
            ensemble_output_version: "1.0".into(),
            file_sha256: file_hash,
        };
        let meta_line = format!(
            "{}\n",
            serde_json::to_string(&meta).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        );
        writer.write_all(meta_line.as_bytes())?;
        Ok(())
    }

    /// Parse an NDJSON file into particle lines and the metadata record.
    pub fn read_metadata_from_ndjson(ndjson: &str) -> Option<MetadataRecord> {
        ndjson.lines().rev()
            .find_map(|line| serde_json::from_str::<MetadataRecord>(line).ok()
                .filter(|m| m.record_type == "metadata"))
    }
}

/// Configuration for NDJSON output (parameters not stored in SmcResult).
#[derive(Debug, Clone)]
pub struct WriteConfig {
    pub resample_threshold: f64,
    pub pop_tolerance: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(n_plans: usize, k: u32) -> SmcResult {
        let plan: Vec<u32> = (0..4).map(|i| if i < 2 { 1 } else { k }).collect();
        SmcResult {
            plans: vec![plan; n_plans],
            weights: vec![1.0 / n_plans as f64; n_plans],
            resample_count: 0,
            resample_rounds: vec![],
            ess_trace: vec![10.0],
            index_maps: vec![],
            base_seed: 42,
            n_particles: n_plans,
        }
    }

    #[test]
    fn ndjson_output_has_correct_line_count() {
        let result = make_result(5, 2);
        let config = WriteConfig { resample_threshold: 0.5, pop_tolerance: 0.005 };
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf, &config).unwrap();
        let s = String::from_utf8(buf).unwrap();
        // 5 particle lines + 1 metadata line = 6 non-empty lines
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 6, "5 particles + 1 metadata = 6 lines");
    }

    #[test]
    fn ndjson_metadata_line_is_last() {
        let result = make_result(3, 2);
        let config = WriteConfig { resample_threshold: 0.5, pop_tolerance: 0.005 };
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf, &config).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let last_line = s.lines().last().unwrap();
        let meta: serde_json::Value = serde_json::from_str(last_line).unwrap();
        assert_eq!(meta["type"], "metadata");
        assert!(meta["file_sha256"].as_str().unwrap().len() == 64, "SHA-256 = 64 hex chars");
    }

    #[test]
    fn ndjson_file_sha256_is_deterministic() {
        let result = make_result(4, 2);
        let config = WriteConfig { resample_threshold: 0.5, pop_tolerance: 0.005 };
        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        result.write_ndjson(&mut buf1, &config).unwrap();
        result.write_ndjson(&mut buf2, &config).unwrap();
        assert_eq!(buf1, buf2, "identical result → identical NDJSON output");
    }

    #[test]
    fn ndjson_lines_use_lf_not_crlf() {
        let result = make_result(2, 2);
        let config = WriteConfig { resample_threshold: 0.5, pop_tolerance: 0.005 };
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf, &config).unwrap();
        assert!(!buf.windows(2).any(|w| w == b"\r\n"), "must not contain CRLF");
        assert!(buf.iter().any(|&b| b == b'\n'), "must contain LF");
    }

    #[test]
    fn ndjson_particle_lines_parse_as_json() {
        let result = make_result(3, 2);
        let config = WriteConfig { resample_threshold: 0.5, pop_tolerance: 0.005 };
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf, &config).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = s.lines().collect();
        // All lines except the last should be particle records
        for line in &lines[..lines.len() - 1] {
            let v: serde_json::Value = serde_json::from_str(line)
                .unwrap_or_else(|_| panic!("particle line is not valid JSON: {line}"));
            assert!(v["plan"].is_array(), "particle line must have 'plan' array");
            assert!(v["log_weight"].is_number(), "particle line must have 'log_weight'");
            assert!(v["particle_idx"].is_number(), "particle line must have 'particle_idx'");
        }
    }

    #[test]
    fn read_metadata_roundtrip() {
        let result = make_result(3, 2);
        let config = WriteConfig { resample_threshold: 0.5, pop_tolerance: 0.005 };
        let mut buf = Vec::new();
        result.write_ndjson(&mut buf, &config).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let meta = SmcResult::read_metadata_from_ndjson(&s)
            .expect("metadata must be parseable");
        assert_eq!(meta.base_seed, 42);
        assert_eq!(meta.n_particles, 3);
        assert_eq!(meta.smc_version, "1.0");
    }
}
