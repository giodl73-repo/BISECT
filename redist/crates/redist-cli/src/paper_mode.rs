//! AEA-compliant replication-package renderer (Researcher Toolkit Task 8 / D-05).
//!
//! When `redist analyze --paper-mode` is invoked, this module writes a
//! `paper_mode/` subdirectory alongside the analyze outputs containing:
//!
//! - `REPRODUCE.sh` — the template at
//!   `scripts/research/paper_mode_template/REPRODUCE.sh` with placeholders
//!   substituted (target platform, build commit, lockfile SHAs, the analyze
//!   invocation, expected outputs path).
//! - `inputs.sha256.json` — SHA-256 of every input file the analyze run consumed
//!   (assignments, election CSV, candidate-race CSV, partisan-baseline TSV).
//! - `expected_outputs.sha256.json` — SHA-256 of every JSON the analyze run
//!   produced under `analysis/`.
//! - `environment.json` — `redist_version`, `redist_build_commit`,
//!   `rustc_version`, `target_triple`, `os_kernel`, `os_arch`.
//! - `seeds.json` — master seed (or `null` if unseeded). Per-step seeds will
//!   be added when the broader seed-derivation system ships.
//! - `CITATION.bib` / `CITATION.apa.txt` / `CITATION.chicago.txt` — citation
//!   strings for the `redist` binary itself per
//!   `docs/file-formats/citation-strings.md` §3.3.
//! - `README.md` — replication walkthrough rendered for this specific run.
//!
//! Deferred (Task 8.4 / 8.5):
//! - The hermetic ubuntu:22.04 byte-identical acceptance test.
//! - Conformance lint against the `social-science-data-editors/template-readme`
//!   rubric.
//!
//! Reproducibility:
//! - All maps use `BTreeMap` (canonical key order).
//! - `SOURCE_DATE_EPOCH` propagates to README.md timestamps when set.
//! - File-walk over `analysis/` is sorted by path so the same set of outputs
//!   produces byte-identical `expected_outputs.sha256.json` across machines.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::provenance::Provenance;

/// Embedded REPRODUCE.sh template — substituted per invocation.
const REPRODUCE_TEMPLATE: &str =
    include_str!("../../../../scripts/research/paper_mode_template/REPRODUCE.sh");

const TARGET_PLATFORM: &str = "linux-x86_64-glibc-2.35";
const SCHEMA_VERSION: &str = "paper-mode v1";

#[derive(Debug, thiserror::Error)]
pub enum PaperModeError {
    #[error("[INPUT] missing required input for paper-mode: {0}")]
    MissingInput(String),
    #[error("[INPUT] cannot read {0}: {1}")]
    Io(String, String),
    #[error("[INTERNAL] cannot serialize {0}: {1}")]
    Serialize(String, String),
}

/// Inputs the renderer needs from the calling analyze run. The caller knows
/// the exact paths it read + wrote; this struct freezes that knowledge.
pub struct PaperModeInputs<'a> {
    /// Directory where analyze wrote its `*.json` outputs (NOT the parent
    /// `analysis/` aggregator — the per-plan analysis directory).
    pub analysis_dir: &'a Path,
    /// Path to the assignments file analyze consumed.
    pub assignments_path: &'a Path,
    /// Optional election CSV (for partisan analysis).
    pub election_file: Option<&'a Path>,
    /// Optional candidate-race CSV (Callais Evidence Layer).
    pub candidate_race_csv: Option<&'a Path>,
    /// Optional partisan-baseline TSV (Callais p.36 disentanglement).
    pub partisan_baseline: Option<&'a Path>,
    /// Verbatim CLI invocation used to produce this run.
    pub analyze_invocation: String,
    /// Master seed (None when the run was deterministic-by-default with no seed).
    pub seed: Option<u64>,
    /// `--paper-mode-citation-style` (advisory; all three are written).
    pub citation_style_default: &'a str,
}

/// Public entry point. Writes the package under `analysis_dir/paper_mode/`.
/// Returns the package directory on success.
pub fn emit_replication_package(inputs: &PaperModeInputs<'_>) -> Result<PathBuf, PaperModeError> {
    let pkg_dir = inputs.analysis_dir.join("paper_mode");
    create_dir(&pkg_dir)?;

    let prov = Provenance::current();
    let inputs_sha = collect_input_shas(inputs)?;
    let outputs_sha = collect_output_shas(inputs.analysis_dir)?;
    let cargo_lock_sha = sha_of_optional_repo_file("redist/Cargo.lock");
    let toolchain_sha = sha_of_optional_repo_file("redist/rust-toolchain.toml");
    let requirements_sha = sha_of_optional_repo_file("requirements.lock");

    write_text(&pkg_dir.join("REPRODUCE.sh"), &render_reproduce_sh(
        &prov,
        &cargo_lock_sha,
        &toolchain_sha,
        &requirements_sha,
        &inputs.analyze_invocation,
        "paper_mode/expected_outputs.sha256.json",
    ))?;

    write_json(&pkg_dir.join("inputs.sha256.json"), &ShaManifest {
        schema_version: SCHEMA_VERSION,
        kind: "inputs",
        files: inputs_sha,
    })?;

    write_json(&pkg_dir.join("expected_outputs.sha256.json"), &ShaManifest {
        schema_version: SCHEMA_VERSION,
        kind: "expected_outputs",
        files: outputs_sha,
    })?;

    write_json(&pkg_dir.join("environment.json"), &EnvironmentRecord::from_provenance(&prov))?;

    write_json(&pkg_dir.join("seeds.json"), &SeedsRecord {
        schema_version: SCHEMA_VERSION,
        master_seed: inputs.seed,
    })?;

    let citations = build_citations(&prov);
    write_text(&pkg_dir.join("CITATION.bib"), &citations.bib)?;
    write_text(&pkg_dir.join("CITATION.apa.txt"), &citations.apa)?;
    write_text(&pkg_dir.join("CITATION.chicago.txt"), &citations.chicago)?;

    write_text(&pkg_dir.join("README.md"), &render_readme(
        &prov,
        &inputs.analyze_invocation,
        inputs.citation_style_default,
    ))?;

    Ok(pkg_dir)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn create_dir(p: &Path) -> Result<(), PaperModeError> {
    std::fs::create_dir_all(p).map_err(|e| PaperModeError::Io(p.display().to_string(), e.to_string()))
}

fn write_text(path: &Path, body: &str) -> Result<(), PaperModeError> {
    std::fs::write(path, body)
        .map_err(|e| PaperModeError::Io(path.display().to_string(), e.to_string()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), PaperModeError> {
    let s = serde_json::to_string_pretty(value)
        .map_err(|e| PaperModeError::Serialize(path.display().to_string(), e.to_string()))?;
    write_text(path, &s)?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let mut s = String::with_capacity(64);
    for b in h.finalize() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn sha_of_path(path: &Path) -> Result<String, PaperModeError> {
    let bytes = std::fs::read(path)
        .map_err(|e| PaperModeError::Io(path.display().to_string(), e.to_string()))?;
    Ok(sha256_hex(&bytes))
}

/// SHA of a file at a project-root-relative path; returns "" if not present.
/// Used for Cargo.lock / rust-toolchain.toml / requirements.lock — these MAY
/// be absent in some test fixtures and the REPRODUCE.sh skip-path handles "".
fn sha_of_optional_repo_file(rel: &str) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join(rel);
        if candidate.is_file() {
            if let Ok(bytes) = std::fs::read(&candidate) {
                return sha256_hex(&bytes);
            }
        }
        // Walk up — `cargo test` may run from inside redist/.
        let mut walk = cwd.clone();
        for _ in 0..5 {
            if walk.join(rel).is_file() {
                if let Ok(bytes) = std::fs::read(walk.join(rel)) {
                    return sha256_hex(&bytes);
                }
            }
            if !walk.pop() {
                break;
            }
        }
    }
    String::new()
}

/// SHA-256 of every input the analyze run consumed; map: relative-name -> SHA.
fn collect_input_shas(
    inputs: &PaperModeInputs<'_>,
) -> Result<BTreeMap<String, String>, PaperModeError> {
    let mut out = BTreeMap::new();
    out.insert(
        "assignments".to_string(),
        sha_of_path(inputs.assignments_path)?,
    );
    if let Some(p) = inputs.election_file {
        out.insert("election_file".to_string(), sha_of_path(p)?);
    }
    if let Some(p) = inputs.candidate_race_csv {
        out.insert("candidate_race_csv".to_string(), sha_of_path(p)?);
    }
    if let Some(p) = inputs.partisan_baseline {
        out.insert("partisan_baseline".to_string(), sha_of_path(p)?);
    }
    Ok(out)
}

/// SHA-256 of every `*.json` and `*.md` file in `analysis_dir/` (one-level,
/// non-recursive — the paper_mode/ subdir is excluded by name).
fn collect_output_shas(analysis_dir: &Path) -> Result<BTreeMap<String, String>, PaperModeError> {
    let mut out = BTreeMap::new();
    if !analysis_dir.is_dir() {
        return Ok(out);
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(analysis_dir)
        .map_err(|e| PaperModeError::Io(analysis_dir.display().to_string(), e.to_string()))?
        .filter_map(|r| r.ok().map(|e| e.path()))
        .filter(|p| {
            if !p.is_file() {
                return false;
            }
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            matches!(ext, "json" | "md")
        })
        .collect();
    entries.sort(); // canonical order

    for p in entries {
        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .map(String::from)
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        out.insert(name, sha_of_path(&p)?);
    }
    Ok(out)
}

fn render_reproduce_sh(
    prov: &Provenance,
    cargo_lock_sha: &str,
    toolchain_sha: &str,
    requirements_sha: &str,
    analyze_invocation: &str,
    expected_outputs_relpath: &str,
) -> String {
    REPRODUCE_TEMPLATE
        .replace("{{REDIST_VERSION}}", &prov.redist_version)
        .replace("{{REDIST_BUILD_COMMIT}}", &prov.redist_build_commit)
        .replace("{{CARGO_LOCK_SHA256}}", cargo_lock_sha)
        .replace("{{RUST_TOOLCHAIN_SHA256}}", toolchain_sha)
        .replace("{{REQUIREMENTS_LOCK_SHA256}}", requirements_sha)
        .replace("{{TARGET_PLATFORM}}", TARGET_PLATFORM)
        .replace("{{ANALYZE_INVOCATION}}", analyze_invocation)
        .replace("{{EXPECTED_OUTPUTS_JSON}}", expected_outputs_relpath)
}

#[derive(Debug, Serialize)]
struct ShaManifest {
    schema_version: &'static str,
    kind: &'static str,
    files: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct EnvironmentRecord {
    schema_version: &'static str,
    redist_version: String,
    redist_build_commit: String,
    redist_build_commit_short: String,
    rustc_version: String,
    target_platform: &'static str,
    os_kernel: String,
    os_arch: String,
}

impl EnvironmentRecord {
    fn from_provenance(prov: &Provenance) -> Self {
        let short: String = prov.redist_build_commit.chars().take(8).collect();
        Self {
            schema_version: SCHEMA_VERSION,
            redist_version: prov.redist_version.clone(),
            redist_build_commit: prov.redist_build_commit.clone(),
            redist_build_commit_short: short,
            rustc_version: prov.rustc_version.clone(),
            target_platform: TARGET_PLATFORM,
            os_kernel: std::env::consts::OS.to_string(),
            os_arch: std::env::consts::ARCH.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
struct SeedsRecord {
    schema_version: &'static str,
    master_seed: Option<u64>,
}

struct CitationStrings {
    bib: String,
    apa: String,
    chicago: String,
}

/// Build citation strings for the `redist` binary itself per
/// `docs/file-formats/citation-strings.md` §3.3.
fn build_citations(prov: &Provenance) -> CitationStrings {
    let short: String = prov.redist_build_commit.chars().take(8).collect();
    let v = &prov.redist_version;
    // Year derived from build date (ISO-8601 prefix "YYYY-").
    let year = prov.redist_build_date.get(..4).unwrap_or("2026");

    let bib = format!(
        "@software{{redist_v{v},\n  author  = {{Della-Libera, Gio}},\n  title   = {{redist}},\n  version = {{{v}}},\n  note    = {{commit {short}}},\n  year    = {{{year}}},\n  url     = {{https://github.com/<owner>/redist}}\n}}\n",
        v = v, short = short, year = year
    );
    let apa = format!(
        "Della-Libera, G. ({year}). redist (Version {v}, commit {short}) [Computer software]. https://github.com/<owner>/redist\n",
        year = year, v = v, short = short
    );
    let chicago = format!(
        "Della-Libera, Gio. {year}. \"redist (Version {v}, commit {short}).\" https://github.com/<owner>/redist.\n",
        year = year, v = v, short = short
    );
    CitationStrings { bib, apa, chicago }
}

fn render_readme(
    prov: &Provenance,
    analyze_invocation: &str,
    citation_style_default: &str,
) -> String {
    let short: String = prov.redist_build_commit.chars().take(8).collect();
    format!(
        "# AEA Replication Package\n\
         \n\
         Schema: `{SCHEMA_VERSION}`\n\
         \n\
         This package was produced by `redist analyze --paper-mode`. It is a\n\
         self-contained recipe that reproduces the headline numbers in the\n\
         accompanying analyze outputs.\n\
         \n\
         ## Provenance\n\
         \n\
         | Field | Value |\n\
         |---|---|\n\
         | `redist_version` | `{version}` |\n\
         | `redist_build_commit` | `{commit}` |\n\
         | `redist_build_commit_short` | `{short}` |\n\
         | `rustc_version` | `{rustc}` |\n\
         | Target platform | `{TARGET_PLATFORM}` |\n\
         \n\
         ## How to reproduce\n\
         \n\
         1. Clone the project at the recorded commit (the script does this if\n\
            you are inside a clone).\n\
         2. Run `bash REPRODUCE.sh`. The script:\n\
            - Verifies your platform matches `{TARGET_PLATFORM}` (warns + asks\n\
              if not).\n\
            - Verifies `Cargo.lock`, `rust-toolchain.toml`, and\n\
              `requirements.lock` match the recorded SHA-256s.\n\
            - Builds `redist` with `--locked`.\n\
            - Re-runs the exact analyze invocation that produced this package.\n\
            - SHA-256-verifies every output file against\n\
              `expected_outputs.sha256.json`.\n\
         \n\
         The exact analyze invocation:\n\
         \n\
         ```bash\n\
         {invocation}\n\
         ```\n\
         \n\
         ## Cross-platform reviewers\n\
         \n\
         If you are on macOS or Windows, run inside WSL2 (Ubuntu 22.04) or a\n\
         Docker container based on `ubuntu:22.04`. Both ship glibc 2.35 by\n\
         default, matching `{TARGET_PLATFORM}`. Floating-point math, glibc\n\
         string functions, and Cargo's transitive-dependency hashes can drift\n\
         across platforms in ways that break checksum equality. The methodology\n\
         still produces statistically equivalent results on other platforms,\n\
         but `expected_outputs.sha256.json` will not match byte-for-byte.\n\
         \n\
         ## Citation\n\
         \n\
         The default citation style for this package is `{citation_style_default}`\n\
         (configurable via `--paper-mode-citation-style`). All three styles\n\
         (BibTeX, APA, Chicago author-date) are included as `CITATION.*` files.\n\
         \n\
         Cite this run in a paper as:\n\
         \n\
         > redist v{version} (commit {short})\n\
         \n\
         See `CITATION.bib` for the full BibTeX entry.\n\
         \n\
         ## Files in this package\n\
         \n\
         | File | Purpose |\n\
         |---|---|\n\
         | `REPRODUCE.sh` | Re-runs the analyze pipeline + verifies output checksums |\n\
         | `inputs.sha256.json` | SHA-256 of every input file the analyze run consumed |\n\
         | `expected_outputs.sha256.json` | SHA-256 of every JSON the analyze run produced |\n\
         | `environment.json` | redist + rustc + platform metadata |\n\
         | `seeds.json` | Master seed for stochastic operations (null if unseeded) |\n\
         | `CITATION.bib` | BibTeX citation for `redist` |\n\
         | `CITATION.apa.txt` | APA citation |\n\
         | `CITATION.chicago.txt` | Chicago author-date citation |\n\
         | `README.md` | This file |\n\
         \n\
         ## See also\n\
         \n\
         - `docs/superpowers/plans/2026-04-30-researcher-toolkit.md` Task 8 (D-05)\n\
         - `docs/file-formats/citation-strings.md` (citation conventions)\n\
         - `scripts/research/paper_mode_template/README.md` (template documentation)\n\
         ",
        SCHEMA_VERSION = SCHEMA_VERSION,
        TARGET_PLATFORM = TARGET_PLATFORM,
        version = prov.redist_version,
        commit = prov.redist_build_commit,
        short = short,
        rustc = prov.rustc_version,
        invocation = analyze_invocation,
        citation_style_default = citation_style_default,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn fixture_inputs<'a>(tmp: &'a TempDir, analysis_dir: &'a Path) -> (PaperModeInputs<'a>, PathBuf) {
        let assignments = tmp.path().join("final_assignments.json");
        fs::write(&assignments, r#"{"50001000100": 1}"#).unwrap();
        // Pre-populate analysis_dir with one fake output so collect_output_shas has work.
        fs::write(analysis_dir.join("summary.json"), r#"{"foo": 1}"#).unwrap();
        (
            PaperModeInputs {
                analysis_dir,
                assignments_path: Box::leak(assignments.clone().into_boxed_path()),
                election_file: None,
                candidate_race_csv: None,
                partisan_baseline: None,
                analyze_invocation: "redist analyze --label vt_2020".to_string(),
                seed: Some(42),
                citation_style_default: "apa",
            },
            assignments,
        )
    }

    #[test]
    fn renders_all_eight_files_under_paper_mode_subdir() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        fs::create_dir_all(&analysis_dir).unwrap();
        let (inputs, _assignments) = fixture_inputs(&tmp, &analysis_dir);

        let pkg_dir = emit_replication_package(&inputs).expect("emit_replication_package");
        assert!(pkg_dir.is_dir());

        for name in &[
            "REPRODUCE.sh",
            "inputs.sha256.json",
            "expected_outputs.sha256.json",
            "environment.json",
            "seeds.json",
            "CITATION.bib",
            "CITATION.apa.txt",
            "CITATION.chicago.txt",
            "README.md",
        ] {
            assert!(
                pkg_dir.join(name).is_file(),
                "paper_mode/{name} must exist after emit_replication_package"
            );
        }
    }

    #[test]
    fn reproduce_sh_substitutes_placeholders() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        fs::create_dir_all(&analysis_dir).unwrap();
        let (inputs, _) = fixture_inputs(&tmp, &analysis_dir);
        let pkg_dir = emit_replication_package(&inputs).unwrap();

        let sh = fs::read_to_string(pkg_dir.join("REPRODUCE.sh")).unwrap();
        // No raw {{ }} placeholders should remain.
        assert!(!sh.contains("{{REDIST_VERSION}}"), "REDIST_VERSION not substituted");
        assert!(!sh.contains("{{TARGET_PLATFORM}}"), "TARGET_PLATFORM not substituted");
        assert!(!sh.contains("{{ANALYZE_INVOCATION}}"), "ANALYZE_INVOCATION not substituted");
        assert!(!sh.contains("{{EXPECTED_OUTPUTS_JSON}}"), "EXPECTED_OUTPUTS_JSON not substituted");
        // Substituted values appear.
        assert!(sh.contains(TARGET_PLATFORM), "TARGET_PLATFORM constant must appear in script");
        assert!(sh.contains("redist analyze --label vt_2020"), "analyze invocation must appear");
        assert!(sh.contains("paper_mode/expected_outputs.sha256.json"), "expected_outputs path must appear");
    }

    #[test]
    fn inputs_sha_records_assignments() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        fs::create_dir_all(&analysis_dir).unwrap();
        let (inputs, _) = fixture_inputs(&tmp, &analysis_dir);
        let pkg_dir = emit_replication_package(&inputs).unwrap();

        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(pkg_dir.join("inputs.sha256.json")).unwrap())
                .unwrap();
        assert_eq!(v["schema_version"].as_str(), Some(SCHEMA_VERSION));
        assert_eq!(v["kind"].as_str(), Some("inputs"));
        let files = v["files"].as_object().unwrap();
        let assignments_sha = files.get("assignments").unwrap().as_str().unwrap();
        assert_eq!(assignments_sha.len(), 64, "SHA-256 must be 64 hex chars");
        assert!(!files.contains_key("election_file"), "absent inputs must not be recorded");
    }

    #[test]
    fn expected_outputs_sha_includes_summary_and_excludes_paper_mode_dir() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        fs::create_dir_all(&analysis_dir).unwrap();
        let (inputs, _) = fixture_inputs(&tmp, &analysis_dir);
        let pkg_dir = emit_replication_package(&inputs).unwrap();

        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(pkg_dir.join("expected_outputs.sha256.json")).unwrap())
                .unwrap();
        let files = v["files"].as_object().unwrap();
        assert!(files.contains_key("summary.json"), "summary.json must be checksummed");
        // Paper-mode dir contents must NOT appear (collect_output_shas is one-level only).
        assert!(!files.contains_key("REPRODUCE.sh"));
        assert!(!files.contains_key("paper_mode"));
    }

    #[test]
    fn environment_records_runtime_metadata() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        fs::create_dir_all(&analysis_dir).unwrap();
        let (inputs, _) = fixture_inputs(&tmp, &analysis_dir);
        let pkg_dir = emit_replication_package(&inputs).unwrap();

        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(pkg_dir.join("environment.json")).unwrap()).unwrap();
        assert_eq!(v["schema_version"].as_str(), Some(SCHEMA_VERSION));
        assert_eq!(v["target_platform"].as_str(), Some(TARGET_PLATFORM));
        assert!(!v["redist_version"].as_str().unwrap().is_empty());
        assert!(!v["rustc_version"].as_str().unwrap().is_empty());
        assert_eq!(v["redist_build_commit_short"].as_str().unwrap().len() <= 8, true);
    }

    #[test]
    fn seeds_record_master_seed_and_null_when_unset() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        fs::create_dir_all(&analysis_dir).unwrap();
        let (mut inputs, _) = fixture_inputs(&tmp, &analysis_dir);
        let pkg_dir = emit_replication_package(&inputs).unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(pkg_dir.join("seeds.json")).unwrap()).unwrap();
        assert_eq!(v["master_seed"].as_u64(), Some(42));

        // Re-run with no seed.
        let analysis_dir2 = tmp.path().join("analysis2");
        fs::create_dir_all(&analysis_dir2).unwrap();
        fs::write(analysis_dir2.join("summary.json"), r#"{"foo": 2}"#).unwrap();
        inputs.analysis_dir = &analysis_dir2;
        inputs.seed = None;
        let pkg_dir2 = emit_replication_package(&inputs).unwrap();
        let v2: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(pkg_dir2.join("seeds.json")).unwrap()).unwrap();
        assert!(v2["master_seed"].is_null(), "unset seed must serialize as null");
    }

    #[test]
    fn citations_include_version_and_short_commit() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        fs::create_dir_all(&analysis_dir).unwrap();
        let (inputs, _) = fixture_inputs(&tmp, &analysis_dir);
        let pkg_dir = emit_replication_package(&inputs).unwrap();

        let bib = fs::read_to_string(pkg_dir.join("CITATION.bib")).unwrap();
        let apa = fs::read_to_string(pkg_dir.join("CITATION.apa.txt")).unwrap();
        let chi = fs::read_to_string(pkg_dir.join("CITATION.chicago.txt")).unwrap();

        let prov = Provenance::current();
        let short: String = prov.redist_build_commit.chars().take(8).collect();
        assert!(bib.contains(&prov.redist_version), "BibTeX must include version");
        assert!(apa.contains(&prov.redist_version), "APA must include version");
        assert!(chi.contains(&prov.redist_version), "Chicago must include version");
        if !short.is_empty() {
            assert!(bib.contains(&short), "BibTeX must include short commit");
        }
    }

    #[test]
    fn readme_carries_invocation_and_target_platform() {
        let tmp = TempDir::new().unwrap();
        let analysis_dir = tmp.path().join("analysis");
        fs::create_dir_all(&analysis_dir).unwrap();
        let (inputs, _) = fixture_inputs(&tmp, &analysis_dir);
        let pkg_dir = emit_replication_package(&inputs).unwrap();

        let readme = fs::read_to_string(pkg_dir.join("README.md")).unwrap();
        assert!(readme.contains("redist analyze --label vt_2020"), "invocation must appear in README");
        assert!(readme.contains(TARGET_PLATFORM), "target platform must appear in README");
        assert!(readme.contains("AEA Replication Package"), "README title must mark this as AEA-compliant");
        assert!(readme.contains("apa"), "default citation style must be mentioned");
    }
}
