//! Native suite manifest assembly and policy ratio resolution. No filesystem access.
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use std::str::FromStr;
use crate::try_validate_nesting;
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
pub fn build_constitutional_nesting_table(policy: &serde_json::Value) -> HashMap<String, NestedRatio> {

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
pub fn resolve_nesting_ratio(policy:&serde_json::Value, state_code: &str, user_ratio: Option<usize>, mut emit: impl FnMut(String)) -> anyhow::Result<usize> {
    let table = build_constitutional_nesting_table(policy);
    match table.get(state_code) {
        Some(NestedRatio::Fixed(r)) => {
            if let Some(ur) = user_ratio {
                if ur != *r {
                    emit(format!(
                        "WARNING: {state_code} constitution requires {r}:1 house-to-senate \
                         nesting ratio. You specified {ur}:1.\n\
                         Proceed only if you have verified this is legally permissible."
                    ));
                    return Ok(ur); // use user-specified value after warning
                }
            }
            Ok(*r)
        }
        Some(NestedRatio::Variable) => {
            // Provide state-specific guidance for common variable-ratio states
            let hint = match state_code {
                "IL" => {
                    " Illinois uses 2:1 or 3:1 depending on district geometry. \
                          Historically most senate districts nest 2 house districts. \
                          Try --nest-ratio 2 first; use --nest-ratio 3 if nesting fails."
                }
                "MD" => {
                    " Maryland senate districts typically contain 3 house delegates, \
                          though some have sub-districts with fewer. \
                          Try --nest-ratio 3."
                }
                _ => " Check your state constitution for the required ratio.",
            };
            user_ratio.ok_or_else(|| {
                anyhow::anyhow!(
                "{state_code} nesting ratio is variable by statute — must be specified explicitly.\
                 {hint}\n\
                 Use: --nest-ratio N  (integer house districts per senate district)"
            )
            })
        }
        None => user_ratio.ok_or_else(|| {
            anyhow::anyhow!(
                "No constitutional nesting ratio defined for {state_code}. \
                 Check your state constitution and use --nest-ratio N."
            )
        }),
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


pub fn prepare(policy:&serde_json::Value,name:&str,state:&str,year:&str,nest_mode:NestMode,nest_ratio:Option<usize>,house_assignments:&HashMap<String,usize>,senate_assignments:&HashMap<String,usize>,congressional_present:bool,mut emit:impl FnMut(String))->anyhow::Result<PlanSuite>{
    let mut plans = Vec::new();

    if congressional_present {
        // Congressional plan: always independent of state legislative nesting.
        // Guard: nesting mode is silently ignored for the congressional chamber —
        // congressional districts do not nest with state legislative districts.
        if nest_mode == NestMode::SenateInHouse {
            emit(format!(
                "NOTE: Congressional redistricting is independent of state legislative \
                 redistricting. The --nest-mode senate-in-house only applies to state \
                 house and senate plans. Congressional districts do not nest with \
                 state legislative districts. The congressional plan will be drawn \
                 without nesting constraints."
            ));
        }
        plans.push(PlanEntry {
            chamber: "congressional".into(),
            file: format!("{}_congressional.rplan", name),
        });
    }

    plans.push(PlanEntry {
        chamber: "house".into(),
        file: format!("{}_house.rplan", name),
    });

    plans.push(PlanEntry {
        chamber: "senate".into(),
        file: format!("{}_senate.rplan", name),
    });

    let suite = PlanSuite {
        suite_name: name.to_owned(),
        state: state.to_owned(),
        year: year.to_owned(),
        nest_mode: nest_mode.clone(),
        plans: plans.clone(),
    };

    // Validate nesting before writing.
    // Nesting applies ONLY to state house and senate plans — not congressional.
    if nest_mode == NestMode::SenateInHouse {
        let required_ratio = resolve_nesting_ratio(policy,state,nest_ratio,|message|emit(message))?;
        let validation =
            try_validate_nesting(house_assignments, senate_assignments, required_ratio)?;
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
            emit(format!(
                "ERROR: nesting validation failed for suite '{}':\n{}",
                name,
                violation_strs.join("\n")
            ));
            return Err(anyhow::anyhow!(
                "Nesting validation failed — see violations above"
            ));
        }
    }

    Ok(suite)
}
