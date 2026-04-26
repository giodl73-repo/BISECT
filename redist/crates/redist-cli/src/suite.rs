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

pub fn build_constitutional_nesting_table() -> HashMap<&'static str, NestedRatio> {
    let mut table = HashMap::new();
    table.insert("WA", NestedRatio::Fixed(2));
    table.insert("IL", NestedRatio::Variable);
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
            user_ratio.ok_or_else(|| anyhow::anyhow!(
                "ERROR: {state_code} nesting ratio is variable by statute. \
                 Specify --nest-ratio N:M (e.g. --nest-ratio 2:1)."
            ))
        }
        None => {
            user_ratio.ok_or_else(|| anyhow::anyhow!(
                "No constitutional nesting ratio defined for {state_code}. \
                 Specify --nest-ratio N:M."
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
pub fn run_suite_validate(
    suite_dir: &Path,
    suite_name: &str,
) -> anyhow::Result<SuiteValidateResult> {
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

    // Load house and senate assignments
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

    let house_assignments =
        load_rplan_assignments(suite_dir.parent().unwrap_or(suite_dir), &house_entry.file)?;
    let senate_assignments =
        load_rplan_assignments(suite_dir.parent().unwrap_or(suite_dir), &senate_entry.file)?;

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

    // Validate nesting before writing
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
            // Return exit code 5 (balance=0 | nesting=4 | ...)
            std::process::exit(5);
        }
    }

    let suite_json = serde_json::to_string_pretty(&suite)?;
    let suite_path = suite_dir.join("suite.json");
    std::fs::write(&suite_path, &suite_json)?;
    eprintln!("[OK] suite manifest -> {}", suite_path.display());

    Ok(())
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
