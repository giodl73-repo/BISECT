//! Output writers for the bloc-voting analyzer.
//!
//! Produces:
//! - `bloc_voting.json` — schema `bloc-voting v1`, validates against
//!   `redist-analysis/schemas/bloc_voting.schema.json`.
//! - `bloc_voting_summary.md` — plain-English [DRAFT] summary with
//!   ecology caveat, robustness table, and the anchor-4 caveat injection.
//!
//! Spec: `docs/superpowers/plans/2026-04-30-callais-evidence-layer.md` Task 6.
//!
//! The schema includes verbatim string fields (`specification`, `ecology_caveat`)
//! that downstream consumers (court report Typst templates, Plan Comparison
//! narratives) match by exact substring. Do not paraphrase.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::bloc_voting::{
    BlocVotingFamilyResult, BlocVotingTestResult, ClusterCi, Coef, RegressionFit, RobustnessCheck,
};
use crate::race_of_candidate::RaceOfCandidateProvenance;

/// Top-level JSON wrapper. One object per analyze run; covers every analyzed
/// candidate.
#[derive(Debug, Clone, Serialize)]
pub struct BlocVotingJson {
    pub schema_version: String,
    pub analyzer: String,
    pub available: bool,
    pub state: String,
    pub year: String,
    pub election: String,
    pub party: String,
    pub method: String,
    pub minority_group: String,
    pub alpha: f64,
    pub ecology: EcologyBlock,
    pub candidates: Vec<CandidateBlock>,
    pub race_of_candidate_provenance: RaceOfCandidateProvenance,
    /// Optional per-(candidate, variant) detail. Always present when the family
    /// included alternative baselines or LOO variants; empty otherwise.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub _family_detail: Vec<FamilyDetail>,
    pub provenance: ProvenanceBlock,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcologyBlock {
    pub n_precincts: usize,
    pub n_clusters: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateBlock {
    pub candidate: String,
    pub regression: RegressionBlock,
    pub robustness_check: RobustnessCheck,
    pub ecology_caveat: String,
    pub draft_interpretation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegressionBlock {
    pub model: String,
    pub specification: String,
    pub diagnostics: DiagnosticsBlock,
    /// BTreeMap so JSON key order is canonical (sorted) for byte-stable output.
    pub coefficients: BTreeMap<String, Coef>,
    pub n_precincts: usize,
    pub n_clusters: usize,
    pub r_squared: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticsBlock {
    pub vif_pct_minority_vs_baseline: f64,
    pub vif_underpowered_flag: bool,
    pub ci_naive_vs_cluster_diverged: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FamilyDetail {
    pub candidate: String,
    pub variant: String,
    pub regression: RegressionBlock,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvenanceBlock {
    pub redist_version: String,
    pub redist_build_commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redist_build_commit_short: Option<String>,
    pub rustc_version: String,
    /// Free-form args that were used to invoke the analyzer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
}

/// Verbatim ecology caveat per spec line 110. Distinguishes per-precinct
/// association from individual-voter behavior; required in every output by the
/// SCALE-block-lifting bargain.
pub const ECOLOGY_CAVEAT: &str =
    "Estimates above describe per-precinct association between racial composition \
     and candidate share. They do NOT imply individual-voter behavior; that is \
     the ecological inference fallacy. The numbers should be interpreted \
     alongside the cluster-bootstrap CI, the VIF diagnostic, and the robustness \
     check across the three alternative baselines.";

/// Build the verbatim regression specification string (Task 6.1 / spec).
pub fn regression_specification_string(bootstrap_samples: usize) -> String {
    format!(
        "WLS weighted by precinct vote count; HC3 robust SE; cluster-bootstrap by county B={bootstrap_samples}"
    )
}

/// Inputs that are common across every analyze invocation but not visible to
/// the regression engine itself (state code, election name, etc.). The caller
/// fills these from the CLI args + `redist-cli/src/provenance.rs`.
#[derive(Debug, Clone)]
pub struct WriteContext<'a> {
    pub state: &'a str,
    pub year: &'a str,
    pub election: &'a str,
    pub party: &'a str,
    pub method: &'a str,
    pub minority_group: &'a str,
    pub alpha: f64,
    pub bootstrap_samples: usize,
    pub provenance: &'a ProvenanceBlock,
    pub race_provenance: &'a RaceOfCandidateProvenance,
}

/// Build the `BlocVotingJson` wrapper from the orchestrator's family result
/// plus the run context. Pure function — no I/O.
pub fn build_bloc_voting_json(
    family: &BlocVotingFamilyResult,
    ctx: &WriteContext<'_>,
) -> BlocVotingJson {
    let n_precincts = family
        .results
        .first()
        .map(|r| r.fit.n)
        .unwrap_or(0);
    let n_clusters = family
        .results
        .first()
        .map(|r| r.fit.n_clusters)
        .unwrap_or(0);

    // Group results by candidate; "primary" variant is the headline regression.
    let mut by_candidate: BTreeMap<String, Vec<&BlocVotingTestResult>> = BTreeMap::new();
    for r in &family.results {
        by_candidate.entry(r.candidate.clone()).or_default().push(r);
    }

    let mut candidates_block: Vec<CandidateBlock> = Vec::new();
    let mut family_detail: Vec<FamilyDetail> = Vec::new();
    for (cand, variants) in by_candidate {
        // Pick the primary variant for the headline regression, else the first.
        let primary = variants
            .iter()
            .find(|r| r.variant == "primary")
            .copied()
            .unwrap_or(variants[0]);

        let robustness = family
            .robustness
            .get(&cand)
            .cloned()
            .unwrap_or_else(|| RobustnessCheck {
                candidate: cand.clone(),
                variants_tested: variants.iter().map(|r| r.variant.clone()).collect(),
                race_coefficient_min: f64::NAN,
                race_coefficient_max: f64::NAN,
                race_coefficient_significant_under_all: false,
            });

        candidates_block.push(CandidateBlock {
            candidate: cand.clone(),
            regression: regression_block(primary, ctx.bootstrap_samples),
            robustness_check: robustness,
            ecology_caveat: ECOLOGY_CAVEAT.to_string(),
            draft_interpretation: family.draft_interpretation.clone(),
        });

        // Family detail: only include if there are non-primary variants.
        if variants.iter().any(|r| r.variant != "primary") {
            for r in &variants {
                family_detail.push(FamilyDetail {
                    candidate: r.candidate.clone(),
                    variant: r.variant.clone(),
                    regression: regression_block(r, ctx.bootstrap_samples),
                });
            }
        }
    }

    BlocVotingJson {
        schema_version: "bloc-voting v1".to_string(),
        analyzer: "bloc-voting".to_string(),
        available: !candidates_block.is_empty(),
        state: ctx.state.to_string(),
        year: ctx.year.to_string(),
        election: ctx.election.to_string(),
        party: ctx.party.to_string(),
        method: ctx.method.to_string(),
        minority_group: ctx.minority_group.to_string(),
        alpha: ctx.alpha,
        ecology: EcologyBlock {
            n_precincts,
            n_clusters,
        },
        candidates: candidates_block,
        race_of_candidate_provenance: ctx.race_provenance.clone(),
        _family_detail: family_detail,
        provenance: ctx.provenance.clone(),
    }
}

fn regression_block(r: &BlocVotingTestResult, bootstrap_samples: usize) -> RegressionBlock {
    let mut coefficients: BTreeMap<String, Coef> = BTreeMap::new();
    for (name, coef) in &r.fit.coefficients {
        coefficients.insert(name.clone(), coef.clone());
    }
    RegressionBlock {
        model: "WLS".to_string(),
        specification: regression_specification_string(bootstrap_samples),
        diagnostics: DiagnosticsBlock {
            vif_pct_minority_vs_baseline: r.fit.vif_pct_minority_vs_baseline,
            vif_underpowered_flag: r.fit.vif_underpowered_flag,
            ci_naive_vs_cluster_diverged: r.cluster.naive_vs_cluster_diverged,
        },
        coefficients,
        n_precincts: r.fit.n,
        n_clusters: r.fit.n_clusters,
        r_squared: r.fit.r_squared,
    }
}

/// Write `bloc_voting.json` and `bloc_voting_summary.md` into `analysis_dir`.
/// Atomic via tmp-then-rename per the rest of the pipeline.
pub fn write_bloc_voting_outputs(
    json: &BlocVotingJson,
    analysis_dir: &Path,
) -> std::io::Result<()> {
    std::fs::create_dir_all(analysis_dir)?;
    write_atomic_json(analysis_dir.join("bloc_voting.json").as_path(), json)?;
    let md = render_summary_md(json);
    write_atomic_str(
        analysis_dir.join("bloc_voting_summary.md").as_path(),
        &md,
    )?;
    Ok(())
}

fn write_atomic_json<T: Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    let serialized = serde_json::to_string_pretty(value)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    write_atomic_str(path, &serialized)
}

fn write_atomic_str(path: &Path, contents: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Plain-English [DRAFT] summary. ASCII-only per PP-34 — file content can be
/// Unicode but this writer stays ASCII so the same string can be echoed to the
/// console safely on Windows.
pub fn render_summary_md(json: &BlocVotingJson) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "# Bloc Voting Analysis (Callais Evidence) — {} {} {}\n\n",
        json.state, json.year, json.election
    ));
    s.push_str(&format!(
        "Party: {} | Method: {} | Minority group: {} | alpha: {}\n\n",
        json.party, json.method, json.minority_group, json.alpha
    ));
    s.push_str(&format!(
        "Schema: {}\nbinary: {} (commit {})\n\n",
        json.schema_version, json.provenance.redist_version, json.provenance.redist_build_commit
    ));

    s.push_str("## Ecology\n\n");
    s.push_str(&format!(
        "- precincts analyzed: {}\n- distinct clusters (counties): {}\n\n",
        json.ecology.n_precincts, json.ecology.n_clusters
    ));

    s.push_str("## Race-of-candidate provenance\n\n");
    s.push_str(&format!(
        "- source file: `{}` (sha256 `{}`)\n- annotations independently verified: {}\n- curators ({}):\n",
        json.race_of_candidate_provenance.source_file,
        json.race_of_candidate_provenance.source_sha256,
        json.race_of_candidate_provenance.annotations_independently_verified,
        json.race_of_candidate_provenance.curators.len()
    ));
    for c in &json.race_of_candidate_provenance.curators {
        s.push_str(&format!(
            "  - {} ({}), attested {}, {} candidates\n",
            c.curator, c.curator_credentials, c.curator_attestation_date, c.n_candidates
        ));
    }
    s.push('\n');
    if !json.race_of_candidate_provenance.annotations_independently_verified {
        s.push_str(
            "**[CAVEAT - annotations not independently verified]** at least one row \
             carries `independently_verified=false`. Headline numbers must be \
             rebutted on the stand if challenged.\n\n",
        );
    }

    s.push_str("## Candidates\n\n");
    for c in &json.candidates {
        s.push_str(&format!("### {}\n\n", c.candidate));
        let race = c
            .regression
            .coefficients
            .get("pct_minority")
            .cloned()
            .unwrap_or_else(Coef::new_unfit_public);
        let baseline = c
            .regression
            .coefficients
            .get("pct_dem_baseline")
            .cloned()
            .unwrap_or_else(Coef::new_unfit_public);
        s.push_str(&format!("- **specification**: {}\n", c.regression.specification));
        s.push_str(&format!("- **n_precincts**: {}, n_clusters: {}, R^2: {:.4}\n",
            c.regression.n_precincts, c.regression.n_clusters, c.regression.r_squared));
        s.push_str(&format!(
            "- **diagnostics**: VIF(pct_minority|pct_dem_baseline) = {:.3} (underpowered={}); naive-vs-cluster CI diverged: {}\n",
            c.regression.diagnostics.vif_pct_minority_vs_baseline,
            c.regression.diagnostics.vif_underpowered_flag,
            c.regression.diagnostics.ci_naive_vs_cluster_diverged
        ));
        s.push_str(&format!(
            "- **race coefficient**: estimate {:.4}, SE_HC3 {:.4}, 95% cluster CI [{:.4}, {:.4}], standardized beta {:.4}, p_raw {:.4}, p_holm {:.4}\n",
            race.estimate, race.stderr_hc3, race.ci_95_cluster.0, race.ci_95_cluster.1,
            race.standardized_beta, race.p_value_raw, race.p_value_holm
        ));
        s.push_str(&format!(
            "- **partisan baseline coefficient**: estimate {:.4}, SE_HC3 {:.4}, p_raw {:.4}\n",
            baseline.estimate, baseline.stderr_hc3, baseline.p_value_raw
        ));
        s.push_str("\n");
        s.push_str("#### Robustness check\n\n");
        s.push_str(&format!(
            "Variants tested: {}\n",
            c.robustness_check.variants_tested.join(", ")
        ));
        s.push_str(&format!(
            "Race coefficient range across variants: [{:.4}, {:.4}]\n",
            c.robustness_check.race_coefficient_min,
            c.robustness_check.race_coefficient_max
        ));
        s.push_str(&format!(
            "**Significant under ALL variants (Holm-corrected p < alpha)**: {}\n\n",
            c.robustness_check.race_coefficient_significant_under_all
        ));
        s.push_str("#### Ecology caveat\n\n");
        s.push_str(&c.ecology_caveat);
        s.push_str("\n\n");
        s.push_str("#### Draft interpretation\n\n");
        s.push_str(&c.draft_interpretation);
        s.push_str("\n\n---\n\n");
    }

    if !json._family_detail.is_empty() {
        s.push_str("## Full family detail (auditor view)\n\n");
        s.push_str(&format!(
            "{} (candidate, variant) regressions in the joint Holm family. See `bloc_voting.json::_family_detail` for per-variant coefficients.\n",
            json._family_detail.len()
        ));
    }

    s
}

// Visibility shim: Coef::new_unfit is pub(super) inside bloc_voting.rs; mirror it here.
impl Coef {
    fn new_unfit_public() -> Self {
        Coef {
            estimate: f64::NAN,
            stderr_hc3: f64::NAN,
            ci_95_cluster: (f64::NAN, f64::NAN),
            standardized_beta: f64::NAN,
            p_value_raw: f64::NAN,
            p_value_holm: f64::NAN,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bloc_voting::{run_bloc_voting_family, variants, BlocVotingConfig, BlocVotingTest, Precinct};

    fn synthetic_precincts(n: usize, beta_min: f64, seed: u64) -> Vec<Precinct> {
        use rand::rngs::SmallRng;
        use rand::{Rng, SeedableRng};
        let mut rng = SmallRng::seed_from_u64(seed);
        (0..n)
            .map(|i| {
                let pm: f64 = rng.gen_range(0.0..1.0);
                let pd: f64 = rng.gen_range(0.0..1.0);
                let noise: f64 = rng.gen_range(-1.0..1.0) * 0.05;
                let y = (0.05 + beta_min * pm + 0.20 * pd + noise).clamp(0.0, 1.0);
                Precinct {
                    id: format!("P{i}"),
                    county_fips: format!("01{:03}", i % 5),
                    total_votes: 100.0,
                    candidate_share: y,
                    pct_minority: pm,
                    pct_dem_baseline: pd,
                }
            })
            .collect()
    }

    fn fixture_provenance() -> RaceOfCandidateProvenance {
        RaceOfCandidateProvenance {
            schema_version: "race-of-candidate v1".into(),
            source_file: "test.csv".into(),
            source_sha256: "0000000000000000000000000000000000000000000000000000000000000000".into(),
            annotations_independently_verified: true,
            curators: vec![],
            attestation_documents: vec![],
        }
    }

    fn fixture_provenance_block() -> ProvenanceBlock {
        ProvenanceBlock {
            redist_version: "0.1.0-test".into(),
            redist_build_commit: "deadbeef0000000000000000000000000000beef".into(),
            redist_build_commit_short: Some("deadbeef".into()),
            rustc_version: "rustc 1.80.0 (test)".into(),
            args: None,
        }
    }

    #[test]
    fn test_build_bloc_voting_json_single_candidate() {
        let pre = synthetic_precincts(150, 0.45, 7);
        let cfg = BlocVotingConfig {
            bootstrap_samples: 30,
            ..Default::default()
        };
        let tests = vec![BlocVotingTest {
            candidate: "Adams".into(),
            variant: variants::PRIMARY.into(),
            precincts: pre,
        }];
        let family = run_bloc_voting_family(&tests, &cfg).unwrap();

        let prov = fixture_provenance();
        let pblock = fixture_provenance_block();
        let ctx = WriteContext {
            state: "VT",
            year: "2020",
            election: "test-primary",
            party: "DEM",
            method: "wls",
            minority_group: "black",
            alpha: 0.05,
            bootstrap_samples: 30,
            provenance: &pblock,
            race_provenance: &prov,
        };
        let json = build_bloc_voting_json(&family, &ctx);
        assert_eq!(json.schema_version, "bloc-voting v1");
        assert_eq!(json.analyzer, "bloc-voting");
        assert!(json.available);
        assert_eq!(json.candidates.len(), 1);
        assert_eq!(json.candidates[0].candidate, "Adams");
        assert!(json.candidates[0]
            .regression
            .specification
            .contains("WLS weighted by precinct vote count"));
        assert!(json.candidates[0]
            .regression
            .specification
            .contains("HC3 robust SE"));
        assert!(json.candidates[0]
            .regression
            .specification
            .contains("cluster-bootstrap by county B=30"));
    }

    #[test]
    fn test_build_bloc_voting_json_round_trips_through_serde() {
        let pre = synthetic_precincts(150, 0.45, 8);
        let cfg = BlocVotingConfig {
            bootstrap_samples: 30,
            ..Default::default()
        };
        let tests = vec![BlocVotingTest {
            candidate: "X".into(),
            variant: variants::PRIMARY.into(),
            precincts: pre,
        }];
        let family = run_bloc_voting_family(&tests, &cfg).unwrap();
        let prov = fixture_provenance();
        let pblock = fixture_provenance_block();
        let ctx = WriteContext {
            state: "VT",
            year: "2020",
            election: "x",
            party: "DEM",
            method: "wls",
            minority_group: "black",
            alpha: 0.05,
            bootstrap_samples: 30,
            provenance: &pblock,
            race_provenance: &prov,
        };
        let json = build_bloc_voting_json(&family, &ctx);
        let s = serde_json::to_string_pretty(&json).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&s).expect("parse");
        assert_eq!(parsed["schema_version"], "bloc-voting v1");
        assert_eq!(parsed["analyzer"], "bloc-voting");
        assert_eq!(parsed["candidates"][0]["candidate"], "X");
        assert_eq!(parsed["candidates"][0]["regression"]["model"], "WLS");
    }

    #[test]
    fn test_render_summary_md_contains_ecology_caveat_verbatim() {
        let pre = synthetic_precincts(150, 0.45, 9);
        let cfg = BlocVotingConfig {
            bootstrap_samples: 30,
            ..Default::default()
        };
        let tests = vec![BlocVotingTest {
            candidate: "X".into(),
            variant: variants::PRIMARY.into(),
            precincts: pre,
        }];
        let family = run_bloc_voting_family(&tests, &cfg).unwrap();
        let prov = fixture_provenance();
        let pblock = fixture_provenance_block();
        let ctx = WriteContext {
            state: "VT", year: "2020", election: "x", party: "DEM", method: "wls",
            minority_group: "black", alpha: 0.05, bootstrap_samples: 30,
            provenance: &pblock, race_provenance: &prov,
        };
        let json = build_bloc_voting_json(&family, &ctx);
        let md = render_summary_md(&json);
        // The verbatim ecology_caveat must appear in the markdown summary.
        assert!(
            md.contains("ecological inference fallacy"),
            "ecology caveat must be present verbatim: {md}"
        );
        assert!(md.contains("WLS weighted by precinct vote count"));
        assert!(md.contains("[DRAFT"));
    }

    #[test]
    fn test_render_summary_md_injects_unverified_caveat() {
        let pre = synthetic_precincts(150, 0.45, 10);
        let cfg = BlocVotingConfig {
            bootstrap_samples: 30,
            provenance: Some(RaceOfCandidateProvenance {
                schema_version: "race-of-candidate v1".into(),
                source_file: "test.csv".into(),
                source_sha256: "0".repeat(64),
                annotations_independently_verified: false,
                curators: vec![],
                attestation_documents: vec![],
            }),
            ..Default::default()
        };
        let tests = vec![BlocVotingTest {
            candidate: "X".into(),
            variant: variants::PRIMARY.into(),
            precincts: pre,
        }];
        let family = run_bloc_voting_family(&tests, &cfg).unwrap();
        let pblock = fixture_provenance_block();
        let ctx = WriteContext {
            state: "VT", year: "2020", election: "x", party: "DEM", method: "wls",
            minority_group: "black", alpha: 0.05, bootstrap_samples: 30,
            provenance: &pblock,
            race_provenance: cfg.provenance.as_ref().unwrap(),
        };
        let json = build_bloc_voting_json(&family, &ctx);
        let md = render_summary_md(&json);
        assert!(
            md.contains("annotations not independently verified"),
            "unverified caveat must appear in summary md: {md}"
        );
    }

    #[test]
    fn test_write_bloc_voting_outputs_atomic() {
        let tmp = tempfile::TempDir::new().unwrap();
        let pre = synthetic_precincts(100, 0.4, 11);
        let cfg = BlocVotingConfig {
            bootstrap_samples: 20,
            ..Default::default()
        };
        let tests = vec![BlocVotingTest {
            candidate: "X".into(),
            variant: variants::PRIMARY.into(),
            precincts: pre,
        }];
        let family = run_bloc_voting_family(&tests, &cfg).unwrap();
        let prov = fixture_provenance();
        let pblock = fixture_provenance_block();
        let ctx = WriteContext {
            state: "VT", year: "2020", election: "x", party: "DEM", method: "wls",
            minority_group: "black", alpha: 0.05, bootstrap_samples: 20,
            provenance: &pblock, race_provenance: &prov,
        };
        let json = build_bloc_voting_json(&family, &ctx);
        write_bloc_voting_outputs(&json, tmp.path()).expect("write");
        assert!(tmp.path().join("bloc_voting.json").exists());
        assert!(tmp.path().join("bloc_voting_summary.md").exists());
        // No tmp file lingering.
        assert!(!tmp.path().join("bloc_voting.tmp").exists());
        assert!(!tmp.path().join("bloc_voting.json.tmp").exists());
    }

    #[test]
    fn test_family_detail_present_when_loo_variants() {
        let pre = synthetic_precincts(150, 0.45, 12);
        let cfg = BlocVotingConfig {
            bootstrap_samples: 20,
            ..Default::default()
        };
        let tests = vec![
            BlocVotingTest {
                candidate: "X".into(),
                variant: variants::PRIMARY.into(),
                precincts: pre.clone(),
            },
            BlocVotingTest {
                candidate: "X".into(),
                variant: variants::loo_excluded("CuratorA"),
                precincts: pre.clone(),
            },
        ];
        let family = run_bloc_voting_family(&tests, &cfg).unwrap();
        let prov = fixture_provenance();
        let pblock = fixture_provenance_block();
        let ctx = WriteContext {
            state: "VT", year: "2020", election: "x", party: "DEM", method: "wls",
            minority_group: "black", alpha: 0.05, bootstrap_samples: 20,
            provenance: &pblock, race_provenance: &prov,
        };
        let json = build_bloc_voting_json(&family, &ctx);
        assert_eq!(json._family_detail.len(), 2,
            "family with non-primary variants must populate _family_detail");
    }

    #[test]
    fn test_family_detail_empty_for_primary_only() {
        let pre = synthetic_precincts(100, 0.45, 13);
        let cfg = BlocVotingConfig {
            bootstrap_samples: 20,
            ..Default::default()
        };
        let tests = vec![BlocVotingTest {
            candidate: "X".into(),
            variant: variants::PRIMARY.into(),
            precincts: pre,
        }];
        let family = run_bloc_voting_family(&tests, &cfg).unwrap();
        let prov = fixture_provenance();
        let pblock = fixture_provenance_block();
        let ctx = WriteContext {
            state: "VT", year: "2020", election: "x", party: "DEM", method: "wls",
            minority_group: "black", alpha: 0.05, bootstrap_samples: 20,
            provenance: &pblock, race_provenance: &prov,
        };
        let json = build_bloc_voting_json(&family, &ctx);
        assert!(json._family_detail.is_empty(),
            "primary-only family must have empty _family_detail");
    }
}
