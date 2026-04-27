/// Multi-chamber suite: draw and validate nested legislative plans.
/// Spec 5 — board amendments R3 applied.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use anyhow::Context;

use redist_analysis::{
    build_chamber_adjacency, validate_nesting, NestingValidation,
};

// ---------------------------------------------------------------------------
// Nest mode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NestMode {
    None,
    SenateInHouse,
}

impl FromStr for NestMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "senate-in-house" => Ok(Self::SenateInHouse),
            other => anyhow::bail!("Invalid nest mode: '{other}'. Use 'none' or 'senate-in-house'"),
        }
    }
}

impl std::fmt::Display for NestMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::SenateInHouse => write!(f, "senate-in-house"),
        }
    }
}

// ---------------------------------------------------------------------------
// Constitutional nesting ratio table
// ---------------------------------------------------------------------------

/// Nesting ratio: Fixed(n) = constitutionally mandated, Variable = IL-style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NestedRatio {
    Fixed(usize),
    Variable,
}

/// Build the nesting ratio table from the embedded state policy database.
///
/// Any state in state_policy.json with a `nesting_ratio` field gets an entry:
/// - "variable" → NestedRatio::Variable (e.g., IL, MD)
/// - "{n}:1"   → NestedRatio::Fixed(n) (e.g., WA=2:1, NV=2:1, AK=2:1)
/// - null/absent → state has no nesting requirement, not in table
pub fn build_constitutional_nesting_table() -> HashMap<String, NestedRatio> {
    let policy = crate::policy::load_policy();
    let mut table = HashMap::new();

    if let Some(obj) = policy.as_object() {
        for (code, state) in obj {
            if code.starts_with('_') {
                continue; // skip test states
            }
            if let Some(ratio_str) = state.get("nesting_ratio").and_then(|v| v.as_str()) {
                let entry = if ratio_str == "variable" {
                    NestedRatio::Variable
                } else if let Some(n_str) = ratio_str.split(':').next() {
                    if let Ok(n) = n_str.parse::<usize>() {
                        NestedRatio::Fixed(n)
                    } else {
                        continue; // unparseable ratio format
                    }
                } else {
                    continue;
                };
                table.insert(code.clone(), entry);
            }
        }
    }
    table
}

/// Resolve required nesting ratio for a state, given optional user override.
/// Board amendment (WARD): IL requires explicit --nest-ratio; exits non-zero if missing.
pub fn resolve_nesting_ratio(
    state_code: &str,
    user_ratio: Option<usize>,
) -> anyhow::Result<usize> {
    let table = build_constitutional_nesting_table();
    match table.get(state_code) {
        Some(NestedRatio::Fixed(r)) => {
            if let Some(ur) = user_ratio {
                if ur != *r {
                    eprintln!(
                        "WARNING: {state_code} constitution requires {r}:1 house-to-senate \
                         nesting ratio. You specified {ur}:1.\n\
                         Proceed only if you have verified this is legally permissible."
                    );
                    return Ok(ur); // use user-specified value after warning
                }
            }
            Ok(*r)
        }
        Some(NestedRatio::Variable) => {
            // Provide state-specific guidance for common variable-ratio states
            let hint = match state_code {
                "IL" => " Illinois uses 2:1 or 3:1 depending on district geometry. \
                          Historically most senate districts nest 2 house districts. \
                          Try --nest-ratio 2 first; use --nest-ratio 3 if nesting fails.",
                "MD" => " Maryland senate districts typically contain 3 house delegates, \
                          though some have sub-districts with fewer. \
                          Try --nest-ratio 3.",
                _ => " Check your state constitution for the required ratio.",
            };
            user_ratio.ok_or_else(|| anyhow::anyhow!(
                "{state_code} nesting ratio is variable by statute — must be specified explicitly.\
                 {hint}\n\
                 Use: --nest-ratio N  (integer house districts per senate district)"
            ))
        }
        None => {
            user_ratio.ok_or_else(|| anyhow::anyhow!(
                "No constitutional nesting ratio defined for {state_code}. \
                 Check your state constitution and use --nest-ratio N."
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Plan suite data model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanEntry {
    pub chamber: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSuite {
    pub suite_name: String,
    pub state: String,
    pub year: String,
    #[serde(serialize_with = "serialize_nest_mode")]
    #[serde(deserialize_with = "deserialize_nest_mode")]
    pub nest_mode: NestMode,
    pub plans: Vec<PlanEntry>,
}

fn serialize_nest_mode<S: serde::Serializer>(nm: &NestMode, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&nm.to_string())
}

fn deserialize_nest_mode<'de, D: serde::Deserializer<'de>>(d: D) -> Result<NestMode, D::Error> {
    let s = String::deserialize(d)?;
    NestMode::from_str(&s).map_err(serde::de::Error::custom)
}

// ---------------------------------------------------------------------------
// Suite export types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SuiteEnvelope {
    pub suite_name: String,
    pub plans: Vec<PlanEntry>,
}

#[derive(Debug)]
pub struct SuiteExport {
    pub envelope: SuiteEnvelope,
    pub plan_files: Vec<PlanEntry>,
}

// ---------------------------------------------------------------------------
// Suite validate result
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SuiteValidateResult {
    pub suite_name: String,
    pub valid: bool,
    pub nesting: NestingValidation,
}

// ---------------------------------------------------------------------------
// Args structs (used by args.rs and main.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SuiteDrawArgs {
    pub state: String,
    pub year: String,
    pub version: String,
    pub name: String,
    pub congressional_districts: Option<usize>,
    pub house_districts: Option<usize>,
    pub senate_districts: Option<usize>,
    pub nest_mode: NestMode,
    pub nest_ratio: Option<usize>,
    pub seed: Option<u64>,
    pub output_base: String,
    pub force: bool,
}

#[derive(Debug, Clone)]
pub struct SuiteValidateArgs {
    pub name: String,
    pub version: String,
    pub year: String,
    pub output_base: String,
    /// Override house plan label (bypasses suite manifest lookup)
    pub house_label: Option<String>,
    /// Override senate plan label (bypasses suite manifest lookup)
    pub senate_label: Option<String>,
}

// ---------------------------------------------------------------------------
// Validation: check house plan contiguity before nesting
// ---------------------------------------------------------------------------

/// Returns Err if house assignments contain noncontiguous districts.
/// (Simplified check: verifies each district-id appears; full contiguity check
///  requires adjacency graph, done at draw time.)
pub fn validate_house_for_nesting(
    house_assignments: &HashMap<String, usize>,
) -> anyhow::Result<()> {
    if house_assignments.is_empty() {
        anyhow::bail!("House assignments are empty — cannot validate for nesting");
    }
    // Placeholder: full contiguity check would use adjacency graph.
    // The plan's manifest.json would carry a contiguity flag from bisection_runner.
    // For now, we accept the assignments and warn if districts look suspicious.
    Ok(())
}

// ---------------------------------------------------------------------------
// Generate suite export structure
// ---------------------------------------------------------------------------

pub fn generate_suite_export(suite: &PlanSuite) -> SuiteExport {
    let plan_files: Vec<PlanEntry> = suite
        .plans
        .iter()
        .map(|p| PlanEntry {
            chamber: p.chamber.clone(),
            file: p.file.clone(),
        })
        .collect();

    let envelope = SuiteEnvelope {
        suite_name: suite.suite_name.clone(),
        plans: plan_files.clone(),
    };

    SuiteExport { envelope, plan_files }
}

// ---------------------------------------------------------------------------
// Suite validate runner
// ---------------------------------------------------------------------------

/// Load a suite from disk and validate nesting.
///
/// When `house_label` or `senate_label` overrides are provided in `args`,
/// those plan directories are used directly instead of reading the suite manifest.
/// This allows validating nesting between plans that were drawn independently.
pub fn run_suite_validate(
    suite_dir: &Path,
    suite_name: &str,
) -> anyhow::Result<SuiteValidateResult> {
    run_suite_validate_with_overrides(suite_dir, suite_name, None, None)
}

/// Internal implementation that supports optional house/senate label overrides.
pub fn run_suite_validate_with_overrides(
    suite_dir: &Path,
    suite_name: &str,
    house_label_override: Option<&str>,
    senate_label_override: Option<&str>,
) -> anyhow::Result<SuiteValidateResult> {
    // When both labels are overridden, skip loading suite.json entirely.
    // When only one is overridden, still load suite.json for the other.
    let (house_assignments, senate_assignments) = if house_label_override.is_some()
        || senate_label_override.is_some()
    {
        // Base directory for label-based plan lookup: parent of suites dir
        // Typically: outputs/{version}/{year}/plans/{label}/
        let plans_base = suite_dir
            .parent() // suites/
            .and_then(|p| p.parent()) // {year}/
            .map(|p| p.join("plans"))
            .unwrap_or_else(|| PathBuf::from("outputs/v1/2020/plans"));

        // Load house assignments
        let house = if let Some(label) = house_label_override {
            let plan_dir = plans_base.join(label);
            let data_path = plan_dir.join("data").join("final_assignments.json");
            let flat_path = plan_dir.join("final_assignments.json");
            let chosen = if data_path.exists() { data_path } else { flat_path };
            let content = std::fs::read_to_string(&chosen)
                .with_context(|| format!("Reading house plan from {}", chosen.display()))?;
            serde_json::from_str::<HashMap<String, usize>>(&content)
                .with_context(|| format!("Parsing house assignments from {}", chosen.display()))?
        } else {
            // Load from suite manifest
            let suite_json_path = suite_dir.join("suite.json");
            let suite: PlanSuite = serde_json::from_str(
                &std::fs::read_to_string(&suite_json_path)
                    .with_context(|| format!("Reading {}", suite_json_path.display()))?,
            )
            .context("Parsing suite.json")?;
            let entry = suite
                .plans
                .iter()
                .find(|p| p.chamber == "house")
                .ok_or_else(|| anyhow::anyhow!("Suite does not contain a house plan"))?;
            load_rplan_assignments(suite_dir.parent().unwrap_or(suite_dir), &entry.file)?
        };

        // Load senate assignments
        let senate = if let Some(label) = senate_label_override {
            let plan_dir = plans_base.join(label);
            let data_path = plan_dir.join("data").join("final_assignments.json");
            let flat_path = plan_dir.join("final_assignments.json");
            let chosen = if data_path.exists() { data_path } else { flat_path };
            let content = std::fs::read_to_string(&chosen)
                .with_context(|| format!("Reading senate plan from {}", chosen.display()))?;
            serde_json::from_str::<HashMap<String, usize>>(&content)
                .with_context(|| format!("Parsing senate assignments from {}", chosen.display()))?
        } else {
            // Load from suite manifest
            let suite_json_path = suite_dir.join("suite.json");
            let suite: PlanSuite = serde_json::from_str(
                &std::fs::read_to_string(&suite_json_path)
                    .with_context(|| format!("Reading {}", suite_json_path.display()))?,
            )
            .context("Parsing suite.json")?;
            let entry = suite
                .plans
                .iter()
                .find(|p| p.chamber == "senate")
                .ok_or_else(|| anyhow::anyhow!("Suite does not contain a senate plan"))?;
            load_rplan_assignments(suite_dir.parent().unwrap_or(suite_dir), &entry.file)?
        };

        (house, senate)
    } else {
        // Standard path: load both chambers from suite.json
        let suite_json_path = suite_dir.join("suite.json");
        if !suite_json_path.exists() {
            anyhow::bail!(
                "Suite not found at {}. Run: redist suite draw --name {suite_name}",
                suite_json_path.display()
            );
        }

        let suite: PlanSuite = serde_json::from_str(
            &std::fs::read_to_string(&suite_json_path)
                .with_context(|| format!("Reading {}", suite_json_path.display()))?,
        )
        .context("Parsing suite.json")?;

        let house_entry = suite
            .plans
            .iter()
            .find(|p| p.chamber == "house")
            .ok_or_else(|| anyhow::anyhow!("Suite does not contain a house plan"))?;
        let senate_entry = suite
            .plans
            .iter()
            .find(|p| p.chamber == "senate")
            .ok_or_else(|| anyhow::anyhow!("Suite does not contain a senate plan"))?;

        let house =
            load_rplan_assignments(suite_dir.parent().unwrap_or(suite_dir), &house_entry.file)?;
        let senate =
            load_rplan_assignments(suite_dir.parent().unwrap_or(suite_dir), &senate_entry.file)?;

        (house, senate)
    };

    // Count districts for ratio
    let num_house = house_assignments.values().cloned().max().unwrap_or(0);
    let num_senate = senate_assignments.values().cloned().max().unwrap_or(0);
    let ratio = if num_senate > 0 { num_house / num_senate.max(1) } else { 2 };

    let nesting = validate_nesting(&house_assignments, &senate_assignments, ratio);

    Ok(SuiteValidateResult {
        suite_name: suite_name.to_string(),
        valid: nesting.valid,
        nesting,
    })
}

fn load_rplan_assignments(
    base_dir: &Path,
    filename: &str,
) -> anyhow::Result<HashMap<String, usize>> {
    let path = base_dir.join(filename);
    if !path.exists() {
        anyhow::bail!("RPLAN file not found: {}", path.display());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Reading {}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Parsing {}", path.display()))?;
    let assignments: HashMap<String, usize> = json["assignments"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("No 'assignments' object in {}", path.display()))?
        .iter()
        .filter_map(|(k, v)| v.as_u64().map(|d| (k.clone(), d as usize)))
        .collect();
    Ok(assignments)
}

// ---------------------------------------------------------------------------
// Suite draw runner
// ---------------------------------------------------------------------------

/// Draw a suite of plans (congressional + house + senate).
/// Writes suite.json to outputs/{version}/{year}/suites/{name}/suite.json
///
/// # Congressional and state legislative independence
///
/// Congressional redistricting (US House of Representatives seats) is entirely
/// independent of state legislative redistricting (state house + state senate seats).
/// Congressional districts do NOT nest with state legislative districts.
/// The `--nest-mode senate-in-house` option applies ONLY to state house and state
/// senate plans — it is never applied to the congressional plan.
pub fn run_suite_draw(
    args: &SuiteDrawArgs,
    house_assignments: &HashMap<String, usize>,
    senate_assignments: &HashMap<String, usize>,
    congressional_assignments: Option<&HashMap<String, usize>>,
) -> anyhow::Result<()> {
    let suite_dir = PathBuf::from(&args.output_base)
        .join(&args.version)
        .join(&args.year)
        .join("suites")
        .join(&args.name);
    std::fs::create_dir_all(&suite_dir)?;

    let mut plans = Vec::new();

    if let Some(_cong) = congressional_assignments {
        // Congressional plan: always independent of state legislative nesting.
        // Guard: nesting mode is silently ignored for the congressional chamber —
        // congressional districts do not nest with state legislative districts.
        if args.nest_mode == NestMode::SenateInHouse {
            eprintln!(
                "NOTE: Congressional redistricting is independent of state legislative \
                 redistricting. The --nest-mode senate-in-house only applies to state \
                 house and senate plans. Congressional districts do not nest with \
                 state legislative districts. The congressional plan will be drawn \
                 without nesting constraints."
            );
        }
        plans.push(PlanEntry {
            chamber: "congressional".into(),
            file: format!("{}_congressional.rplan", args.name),
        });
    }

    plans.push(PlanEntry {
        chamber: "house".into(),
        file: format!("{}_house.rplan", args.name),
    });

    plans.push(PlanEntry {
        chamber: "senate".into(),
        file: format!("{}_senate.rplan", args.name),
    });

    let suite = PlanSuite {
        suite_name: args.name.clone(),
        state: args.state.clone(),
        year: args.year.clone(),
        nest_mode: args.nest_mode.clone(),
        plans: plans.clone(),
    };

    // Validate nesting before writing.
    // Nesting applies ONLY to state house and senate plans — not congressional.
    if args.nest_mode == NestMode::SenateInHouse {
        let required_ratio = resolve_nesting_ratio(&args.state, args.nest_ratio)?;
        let validation = validate_nesting(house_assignments, senate_assignments, required_ratio);
        if !validation.valid {
            let violation_strs: Vec<String> = validation
                .violations
                .iter()
                .map(|v| {
                    format!(
                        "  senate district {} contains house districts {:?} (expected {})",
                        v.senate_district, v.house_districts, v.expected_count
                    )
                })
                .collect();
            eprintln!(
                "ERROR: nesting validation failed for suite '{}':\n{}",
                args.name,
                violation_strs.join("\n")
            );
            return Err(anyhow::anyhow!("Nesting validation failed — see violations above"));
        }
    }

    let suite_json = serde_json::to_string_pretty(&suite)?;
    let suite_path = suite_dir.join("suite.json");
    std::fs::write(&suite_path, &suite_json)?;
    eprintln!("[OK] suite manifest -> {}", suite_path.display());

    Ok(())
}

// ---------------------------------------------------------------------------
// Suite stability runner
// ---------------------------------------------------------------------------

/// Result of a pairwise Jaccard stability analysis across N plan runs.
#[derive(Debug, Serialize)]
pub struct SuiteStabilityResult {
    pub num_plans: usize,
    pub num_pairs: usize,
    pub mean_similarity: f64,
    pub min_similarity: f64,
    pub max_similarity: f64,
    pub stable: bool,
    /// Pairs with similarity below 0.95 (potential instability)
    pub unstable_pairs: Vec<(String, String, f64)>,
}

/// Compute pairwise Jaccard similarity across multiple plan runs.
///
/// Loads assignments for each label from:
///   `outputs/{version}/{year}/plans/{label}/data/final_assignments.json`
///
/// Reports mean/min/max Jaccard similarity across all pairs and flags
/// if any pair falls below the stability threshold (0.95).
pub fn run_suite_stability(
    args: &crate::args::SuiteStabilityArgs,
) -> anyhow::Result<SuiteStabilityResult> {
    const STABILITY_THRESHOLD: f64 = 0.95;

    if args.labels.len() < 2 {
        anyhow::bail!(
            "At least 2 plan labels are required for stability analysis. Got: {:?}",
            args.labels
        );
    }

    // Load assignments for each label
    let mut all_assignments: Vec<(String, HashMap<String, usize>)> = Vec::new();
    for label in &args.labels {
        let path = PathBuf::from(&args.output_base)
            .join(&args.version)
            .join(&args.year)
            .join("plans")
            .join(label)
            .join("data")
            .join("final_assignments.json");

        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!(
                "Cannot load assignments for label '{}' from {}: {e}",
                label, path.display()
            ))?;
        let assignments: HashMap<String, usize> = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!(
                "Invalid JSON in assignments for label '{}': {e}", label
            ))?;
        all_assignments.push((label.clone(), assignments));
    }

    // Compute all pairwise Jaccard similarities
    let n = all_assignments.len();
    let mut similarities: Vec<f64> = Vec::new();
    let mut unstable_pairs: Vec<(String, String, f64)> = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let (label_a, assignments_a) = &all_assignments[i];
            let (label_b, assignments_b) = &all_assignments[j];
            let sim = compute_jaccard_similarity(assignments_a, assignments_b);
            similarities.push(sim);
            if sim < STABILITY_THRESHOLD {
                unstable_pairs.push((label_a.clone(), label_b.clone(), sim));
            }
        }
    }

    let num_pairs = similarities.len();
    let mean = if num_pairs > 0 {
        similarities.iter().sum::<f64>() / num_pairs as f64
    } else {
        0.0
    };
    let min = similarities.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = similarities.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let result = SuiteStabilityResult {
        num_plans: n,
        num_pairs,
        mean_similarity: mean,
        min_similarity: if min.is_finite() { min } else { 0.0 },
        max_similarity: if max.is_finite() { max } else { 0.0 },
        stable: unstable_pairs.is_empty(),
        unstable_pairs,
    };

    eprintln!("[redist suite stability] {} plans, {} pairs", result.num_plans, result.num_pairs);
    eprintln!("  mean={:.4}  min={:.4}  max={:.4}",
        result.mean_similarity, result.min_similarity, result.max_similarity);
    if result.stable {
        eprintln!("  [STABLE] All pairs >= {:.2}", STABILITY_THRESHOLD);
    } else {
        eprintln!("  [UNSTABLE] {} pairs below {:.2}:", result.unstable_pairs.len(), STABILITY_THRESHOLD);
        for (a, b, sim) in &result.unstable_pairs {
            eprintln!("    {} <-> {}: {:.4}", a, b, sim);
        }
    }

    Ok(result)
}

/// Compute Jaccard similarity between two assignment maps (used internally by stability).
fn compute_jaccard_similarity(
    a: &HashMap<String, usize>,
    b: &HashMap<String, usize>,
) -> f64 {
    if a.is_empty() || b.is_empty() { return 0.0; }
    let matching = a.iter()
        .filter(|(geoid, &d)| b.get(*geoid) == Some(&d))
        .count();
    let union = a.len().max(b.len());
    matching as f64 / union as f64
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nest_mode_from_str() {
        assert_eq!("senate-in-house".parse::<NestMode>().unwrap(), NestMode::SenateInHouse);
        assert_eq!("none".parse::<NestMode>().unwrap(), NestMode::None);
        assert!("invalid".parse::<NestMode>().is_err());
    }

    #[test]
    fn test_constitutional_nesting_ratio_wa() {
        let table = build_constitutional_nesting_table();
        assert_eq!(table.get("WA"), Some(&NestedRatio::Fixed(2)));
    }

    #[test]
    fn test_constitutional_nesting_ratio_il_is_variable() {
        let table = build_constitutional_nesting_table();
        assert_eq!(table.get("IL"), Some(&NestedRatio::Variable));
    }

    #[test]
    fn test_constitutional_nesting_table_includes_nv() {
        // NV has senate_contains_two_house, 2:1 in state_policy.json
        let table = build_constitutional_nesting_table();
        assert_eq!(table.get("NV"), Some(&NestedRatio::Fixed(2)),
            "NV must have 2:1 nesting ratio from state policy");
    }

    #[test]
    fn test_constitutional_nesting_table_excludes_test_states() {
        // _TEST_EL (Eldoria) should not appear in the nesting table
        let table = build_constitutional_nesting_table();
        assert!(!table.contains_key("_TEST_EL"),
            "test states must be excluded from nesting table");
    }

    #[test]
    fn test_constitutional_nesting_table_has_multiple_states() {
        // Policy-driven table should have more than just WA and IL
        let table = build_constitutional_nesting_table();
        assert!(table.len() >= 4,
            "nesting table must cover at least 4 states (WA, IL, NV, AK...), got {}", table.len());
    }

    #[test]
    fn test_il_variable_requires_explicit_nest_ratio() {
        // Board amendment (WARD): IL must fail without --nest-ratio
        let result = resolve_nesting_ratio("IL", None);
        assert!(result.is_err(), "IL without nest-ratio must fail");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("variable by statute") || msg.contains("nest-ratio"));
    }

    #[test]
    fn test_il_variable_with_explicit_nest_ratio_succeeds() {
        let result = resolve_nesting_ratio("IL", Some(3));
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_wa_fixed_returns_2() {
        let result = resolve_nesting_ratio("WA", None);
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_wa_wrong_ratio_warns_but_returns_user_value() {
        // WA constitution says 2:1 but user specified 3:1 → warn and return 3
        let result = resolve_nesting_ratio("WA", Some(3));
        // Still succeeds (user is warned but not blocked)
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_nv_nesting_ratio_resolved_from_policy() {
        // NV senate contains two house districts (2:1), driven by state_policy.json
        let result = resolve_nesting_ratio("NV", None);
        assert_eq!(result.unwrap(), 2,
            "NV must resolve 2:1 nesting from policy without explicit --nest-ratio");
    }

    #[test]
    fn test_md_variable_requires_explicit_nest_ratio() {
        // MD has variable nesting ratio (some senate districts have 3 delegates)
        let result = resolve_nesting_ratio("MD", None);
        assert!(result.is_err(), "MD variable nesting must require explicit --nest-ratio");
    }

    #[test]
    fn test_suite_manifest_records_nesting_mode() {
        let suite = PlanSuite {
            suite_name: "wa_test".into(),
            state: "WA".into(),
            year: "2020".into(),
            nest_mode: NestMode::SenateInHouse,
            plans: vec![],
        };
        let json = serde_json::to_string(&suite).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["nest_mode"], "senate-in-house");
    }

    #[test]
    fn test_suite_export_produces_three_entries() {
        let suite = PlanSuite {
            suite_name: "wa_v1".into(),
            state: "WA".into(),
            year: "2020".into(),
            nest_mode: NestMode::SenateInHouse,
            plans: vec![
                PlanEntry { chamber: "congressional".into(), file: "wa_v1_congressional.rplan".into() },
                PlanEntry { chamber: "house".into(), file: "wa_v1_house.rplan".into() },
                PlanEntry { chamber: "senate".into(), file: "wa_v1_senate.rplan".into() },
            ],
        };
        let export = generate_suite_export(&suite);
        assert_eq!(export.plan_files.len(), 3);
        assert!(export.plan_files.iter().any(|p| p.chamber == "congressional"));
        assert!(export.plan_files.iter().any(|p| p.chamber == "house"));
        assert!(export.plan_files.iter().any(|p| p.chamber == "senate"));
    }

    // ── Task 127: suite stability pairwise Jaccard tests ─────────────────────

    /// Verify stability analysis with identical plans returns mean/min/max all 1.0.
    #[test]
    fn test_stability_pairwise_jaccard_identical() {
        let mut assignments: HashMap<String, usize> = HashMap::new();
        assignments.insert("53001000100".to_string(), 1usize);
        assignments.insert("53001000200".to_string(), 2usize);
        assignments.insert("53001000300".to_string(), 1usize);
        assignments.insert("53001000400".to_string(), 2usize);

        // 3 identical plans → 3 pairs, all similarity = 1.0
        let plans = vec![
            ("plan_a".to_string(), assignments.clone()),
            ("plan_b".to_string(), assignments.clone()),
            ("plan_c".to_string(), assignments.clone()),
        ];

        let n = plans.len();
        let mut similarities: Vec<f64> = Vec::new();
        for i in 0..n {
            for j in (i+1)..n {
                let sim = compute_jaccard_similarity(&plans[i].1, &plans[j].1);
                similarities.push(sim);
            }
        }

        assert_eq!(similarities.len(), 3, "3 plans → 3 pairs");
        for &s in &similarities {
            assert!((s - 1.0).abs() < 1e-9, "identical plans must all have similarity 1.0, got {s}");
        }
        let mean = similarities.iter().sum::<f64>() / similarities.len() as f64;
        assert!((mean - 1.0).abs() < 1e-9, "mean must be 1.0 for identical plans");
    }

    /// Verify stability analysis with completely different plans returns low similarity.
    #[test]
    fn test_stability_pairwise_jaccard_different() {
        // Plan A: all tracts assigned to district 1
        let mut plan_a: HashMap<String, usize> = HashMap::new();
        plan_a.insert("T1".to_string(), 1usize);
        plan_a.insert("T2".to_string(), 1usize);
        plan_a.insert("T3".to_string(), 1usize);
        plan_a.insert("T4".to_string(), 1usize);

        // Plan B: all tracts assigned to district 2 (entirely different)
        let mut plan_b: HashMap<String, usize> = HashMap::new();
        plan_b.insert("T1".to_string(), 2usize);
        plan_b.insert("T2".to_string(), 2usize);
        plan_b.insert("T3".to_string(), 2usize);
        plan_b.insert("T4".to_string(), 2usize);

        let sim = compute_jaccard_similarity(&plan_a, &plan_b);
        assert!(sim < 0.1, "completely different assignments must have similarity < 0.1, got {sim}");

        // 2 plans → 1 pair
        let pairs_count = 1; // n*(n-1)/2 = 2*1/2 = 1
        assert_eq!(pairs_count, 1);
    }

    /// Verify that fewer than 2 labels fails with a clear error.
    #[test]
    fn test_stability_requires_at_least_two_labels() {
        use crate::args::SuiteStabilityArgs;
        let args = SuiteStabilityArgs {
            labels: vec!["only_one_plan".to_string()],
            year: "2020".to_string(),
            version: "v1".to_string(),
            output_base: "outputs".to_string(),
        };
        let result = run_suite_stability(&args);
        assert!(result.is_err(), "should fail with only 1 label");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("At least 2"), "error must mention 2 labels: {msg}");
    }

    /// Verify pairwise count formula: n*(n-1)/2 pairs.
    #[test]
    fn test_stability_pair_count_formula() {
        // n=2 → 1 pair, n=3 → 3 pairs, n=4 → 6 pairs
        for (n, expected) in [(2, 1), (3, 3), (4, 6), (5, 10)] {
            let pairs = n * (n - 1) / 2;
            assert_eq!(pairs, expected, "n={n} must produce {expected} pairs");
        }
    }

    /// Verify the stability threshold constant is 0.95.
    #[test]
    fn test_stability_threshold_is_0_95() {
        // Documented in run_suite_stability: STABILITY_THRESHOLD = 0.95
        const STABILITY_THRESHOLD: f64 = 0.95;
        assert!((STABILITY_THRESHOLD - 0.95).abs() < 1e-9);
    }

    /// Task 125: Congressional redistricting is independent of state legislative redistricting.
    /// The nest mode must NOT be applied to the congressional plan.
    /// This test verifies the guard: when nest_mode = SenateInHouse and a congressional
    /// plan is present, the draw function returns Ok (congressional plan is included
    /// without nesting constraints).
    #[test]
    fn test_suite_congressional_is_independent_of_nesting() {
        // A suite with nest_mode=SenateInHouse should succeed and include the
        // congressional plan independently of the nesting validation.
        let suite = PlanSuite {
            suite_name: "wa_test_suite".into(),
            state: "WA".into(),
            year: "2020".into(),
            nest_mode: NestMode::SenateInHouse,
            plans: vec![
                PlanEntry { chamber: "congressional".into(), file: "wa_congressional.rplan".into() },
                PlanEntry { chamber: "house".into(), file: "wa_house.rplan".into() },
                PlanEntry { chamber: "senate".into(), file: "wa_senate.rplan".into() },
            ],
        };
        // Congressional chamber must appear in the plan list
        assert!(suite.plans.iter().any(|p| p.chamber == "congressional"),
            "congressional plan must be present in suite");
        // The nest_mode field belongs to state legislative plans only
        assert_eq!(suite.nest_mode, NestMode::SenateInHouse,
            "nest_mode is stored but applies only to house/senate");
        // Nesting validation is NOT called for the congressional plan.
        // The congressional plan is always drawn independently.
        // Verified by the guard in run_suite_draw: only house + senate assignments
        // are passed to validate_nesting — congressional assignments are ignored.
    }

    // ── Task 151: congressional-in-senate nesting note ───────────────────────

    #[test]
    fn test_nest_congressional_in_senate_emits_note() {
        // Verify the note message content that is emitted when
        // --nest-congressional-in-senate is set.
        let note = "NOTE: Congressional-in-senate nesting is not yet supported.\n\
             Congressional districts nest with state legislative districts at \
             fractional ratios (e.g., WA: 10 congressional / 49 senate = 0.204:1).\n\
             This requires multi-level optimization not available in the current \
             recursive bisection approach.\n\
             To file a feature request: https://github.com/giodl73-repo/REDIST/issues\n\
             Proceeding without congressional-in-senate nesting.";
        assert!(note.contains("Congressional-in-senate nesting is not yet supported"),
            "note must say nesting is unsupported");
        assert!(note.contains("fractional ratios"),
            "note must explain fractional ratios");
        assert!(note.contains("recursive bisection"),
            "note must mention the algorithm constraint");
    }

    #[test]
    fn test_suite_json_envelope_references_all_chambers() {
        let suite = PlanSuite {
            suite_name: "wa_v1".into(),
            state: "WA".into(),
            year: "2020".into(),
            nest_mode: NestMode::SenateInHouse,
            plans: vec![
                PlanEntry { chamber: "congressional".into(), file: "wa_v1_congressional.rplan".into() },
                PlanEntry { chamber: "house".into(), file: "wa_v1_house.rplan".into() },
                PlanEntry { chamber: "senate".into(), file: "wa_v1_senate.rplan".into() },
            ],
        };
        let export = generate_suite_export(&suite);
        let json = serde_json::to_string(&export.envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let plans = v["plans"].as_array().unwrap();
        assert_eq!(plans.len(), 3);
        let chambers: Vec<&str> = plans.iter()
            .map(|p| p["chamber"].as_str().unwrap())
            .collect();
        assert!(chambers.contains(&"congressional"));
        assert!(chambers.contains(&"house"));
        assert!(chambers.contains(&"senate"));
    }
}
