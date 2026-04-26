/// State policy database — loads and queries redist/data/state_policy.json.
///
/// The JSON is embedded directly in the binary at compile time so `redist policy`
/// works without any external files. Override with REDIST_STATE_POLICY env var.

/// Embedded fallback — compiled into the binary.
static EMBEDDED_POLICY: &str = include_str!("../../../data/state_policy.json");

pub fn load_policy() -> serde_json::Value {
    if let Ok(path) = std::env::var("REDIST_STATE_POLICY") {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str(&content) {
                return v;
            }
            eprintln!(
                "WARNING: REDIST_STATE_POLICY={path} could not be parsed; \
                 falling back to embedded policy."
            );
        } else {
            eprintln!(
                "WARNING: REDIST_STATE_POLICY={path} could not be read; \
                 falling back to embedded policy."
            );
        }
    }
    serde_json::from_str(EMBEDDED_POLICY).expect("embedded state_policy.json is valid JSON")
}

pub fn get_state_policy(policy: &serde_json::Value, state_code: &str) -> Option<serde_json::Value> {
    policy.get(state_code).cloned()
}

/// Return the subdivision term for `state_code` (e.g. "parish" for LA, "borough" for AK).
/// Falls back to "county" / "counties" when the state is not in the database.
pub fn subdivision_term(state_code: &str, plural: bool) -> String {
    let policy = load_policy();
    if let Some(state) = get_state_policy(&policy, state_code) {
        let key = if plural { "subdivision_term_plural" } else { "subdivision_term" };
        if let Some(term) = state.get(key).and_then(|v| v.as_str()) {
            return term.to_string();
        }
    }
    if plural { "counties".to_string() } else { "county".to_string() }
}

use crate::args::PolicyArgs;

pub fn run_policy(args: &PolicyArgs) -> anyhow::Result<()> {
    let policy = load_policy();
    let state_code = args.state.to_uppercase();

    match get_state_policy(&policy, &state_code) {
        None => anyhow::bail!(
            "State '{}' not found in policy database. \
            Check REDIST_STATE_POLICY or add the state to redist/data/state_policy.json.",
            state_code
        ),
        Some(state) => {
            if args.format == "json" {
                println!("{}", serde_json::to_string_pretty(&state)?);
            } else {
                // Human-readable table format
                let name = state["name"].as_str().unwrap_or(&state_code);
                println!("=== {name} ({state_code}) Redistricting Policy ===\n");

                let fields = [
                    ("House districts",                    "house_districts"),
                    ("Senate districts",                   "senate_districts"),
                    ("Subdivision term",                   "subdivision_term_plural"),
                    ("Municipal term",                     "municipal_term_plural"),
                    ("House balance tolerance",            "balance_tolerance_house_pct"),
                    ("Senate balance tolerance",           "balance_tolerance_senate_pct"),
                    ("Congressional balance tolerance",    "balance_tolerance_congressional_pct"),
                    ("Population basis",                   "population_basis"),
                    ("Prison gerrymandering reform",       "prison_gerrymandering"),
                    ("Student counting",                   "student_counting"),
                    ("Nesting requirement",                "nesting_requirement"),
                    ("Nesting ratio",                      "nesting_ratio"),
                    ("Preservation standard",              "preservation_standard"),
                    ("Legal citation",                     "preservation_citation"),
                    ("Commission type",                    "commission_type"),
                    ("VRA Section 2 applies",              "vra_section2_applies"),
                ];

                for (label, key) in fields {
                    if let Some(val) = state.get(key) {
                        if !val.is_null() {
                            let display = match val {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => {
                                    if key.contains("pct") {
                                        format!("{}%", n)
                                    } else {
                                        n.to_string()
                                    }
                                }
                                serde_json::Value::Bool(b) => {
                                    if *b { "Yes".into() } else { "No".into() }
                                }
                                other => other.to_string(),
                            };
                            println!("  {label:<38} {display}");
                        }
                    }
                }

                if let Some(notes) = state.get("notes").and_then(|v| v.as_str()) {
                    println!("\nNotes: {notes}");
                }
                if let Some(note) = state.get("independent_city_note").and_then(|v| v.as_str()) {
                    println!("\nIndependent cities: {note}");
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eldoria_subdivision_term() {
        // Eldoria uses "realm" not "county" -- proves vocabulary is config-driven
        let policy = load_policy();
        let eldoria = get_state_policy(&policy, "_TEST_EL").unwrap();
        assert_eq!(eldoria["subdivision_term"].as_str().unwrap(), "realm");
        assert_eq!(eldoria["subdivision_term_plural"].as_str().unwrap(), "realms");
        assert_eq!(eldoria["balance_tolerance_house_pct"].as_f64().unwrap(), 7.5);
        assert_eq!(
            eldoria["commission_type"].as_str().unwrap(),
            "grand_council_of_wizards"
        );
    }

    #[test]
    fn test_la_subdivision_term_is_parish() {
        let term = subdivision_term("LA", false);
        assert_eq!(term, "parish");
        let term_plural = subdivision_term("LA", true);
        assert_eq!(term_plural, "parishes");
    }

    #[test]
    fn test_va_subdivision_term() {
        let term = subdivision_term("VA", false);
        assert!(
            term.contains("county") || term.contains("city"),
            "VA term={term}"
        );
    }

    #[test]
    fn test_unknown_state_defaults_to_county() {
        let term = subdivision_term("ZZ", false);
        assert_eq!(term, "county");
    }

    #[test]
    fn test_load_policy_returns_valid_json_object() {
        let policy = load_policy();
        assert!(policy.is_object(), "policy must be a JSON object");
    }

    #[test]
    fn test_get_state_policy_known_state() {
        let policy = load_policy();
        let la = get_state_policy(&policy, "LA");
        assert!(la.is_some(), "Louisiana must be in the policy database");
        let la = la.unwrap();
        assert_eq!(la["name"].as_str().unwrap(), "Louisiana");
    }

    #[test]
    fn test_get_state_policy_unknown_state_returns_none() {
        let policy = load_policy();
        assert!(get_state_policy(&policy, "ZZ").is_none());
    }

    #[test]
    fn test_ak_subdivision_term_is_borough() {
        assert_eq!(subdivision_term("AK", false), "borough");
        assert_eq!(subdivision_term("AK", true), "boroughs");
    }

    #[test]
    fn test_ct_subdivision_term_is_town() {
        assert_eq!(subdivision_term("CT", false), "town");
        assert_eq!(subdivision_term("CT", true), "towns");
    }
}
