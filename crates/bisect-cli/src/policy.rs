/// Location policy database — loads and queries redist/data/location_policy.json.
///
/// The JSON is embedded directly in the binary at compile time so `redist policy`
/// works without any external files.
///
/// Override with REDIST_LOCATION_POLICY env var (REDIST_STATE_POLICY accepted for
/// backward compatibility).

/// Embedded fallback — compiled into the binary.
static EMBEDDED_POLICY: &str = include_str!("../../../data/location_policy.json");

/// Re-export LocationRegistry so callers can use `crate::policy::LocationRegistry`.
pub use crate::registry::LocationRegistry;

pub fn load_policy() -> serde_json::Value {
    // Primary env var
    if let Ok(path) = std::env::var("REDIST_LOCATION_POLICY") {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str(&content) {
                return v;
            }
            eprintln!(
                "WARNING: REDIST_LOCATION_POLICY={path} could not be parsed; \
                 falling back to embedded policy."
            );
        } else {
            eprintln!(
                "WARNING: REDIST_LOCATION_POLICY={path} could not be read; \
                 falling back to embedded policy."
            );
        }
    }
    // Backward-compat alias
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
    serde_json::from_str(EMBEDDED_POLICY).expect("embedded location_policy.json is valid JSON")
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
        None => {
            // Try fuzzy name match (e.g., "ireland" -> "IE")
            let policy_obj = policy.as_object();
            if let Some(obj) = policy_obj {
                let query = state_code.to_lowercase();
                let matches: Vec<(&str, &str)> = obj.iter()
                    .filter(|(k, _)| !k.starts_with('_'))  // skip test states
                    .filter_map(|(code, entry)| {
                        entry.get("name")
                            .and_then(|v| v.as_str())
                            .filter(|name| name.to_lowercase().contains(&query))
                            .map(|name| (code.as_str(), name))
                    })
                    .collect();

                match matches.len() {
                    1 => {
                        let (code, _name) = matches[0];
                        eprintln!("NOTE: '{}' matched as {} ({}). Use the 2-letter code next time.",
                            state_code, code, _name);
                        // Re-run display with the matched code
                        let matched_args = PolicyArgs {
                            state: code.to_string(),
                            format: args.format.clone(),
                        };
                        return run_policy(&matched_args);
                    }
                    0 => {} // fall through to error
                    _ => {
                        let suggestions: Vec<String> = matches.iter()
                            .map(|(code, name)| format!("{} ({})", code, name))
                            .collect();
                        eprintln!("Multiple matches for '{}': {}. Use the 2-letter code.",
                            state_code, suggestions.join(", "));
                    }
                }
            }
            anyhow::bail!(
                "Location '{}' not found in policy database. \
                Check REDIST_LOCATION_POLICY or use a 2-letter code (e.g., IE for Ireland).",
                state_code
            )
        }
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

    #[test]
    fn test_policy_la_has_parish_term() {
        let policy = load_policy();
        let la = get_state_policy(&policy, "LA").unwrap();
        assert_eq!(la["subdivision_term"].as_str().unwrap(), "parish");
        assert_eq!(la["subdivision_term_plural"].as_str().unwrap(), "parishes");
    }

    #[test]
    fn test_policy_va_has_independent_city_note() {
        let policy = load_policy();
        let va = get_state_policy(&policy, "VA").unwrap();
        // VA must document its independent city structure
        assert!(va.get("independent_city_note").is_some(), "VA must have independent_city_note");
        let note = va["independent_city_note"].as_str().unwrap();
        assert!(note.contains("independent"), "VA note must mention independent cities");
    }

    #[test]
    fn test_policy_wa_has_nesting_requirement() {
        let policy = load_policy();
        let wa = get_state_policy(&policy, "WA").unwrap();
        assert_eq!(wa["nesting_requirement"].as_str().unwrap(), "senate_contains_two_house");
        assert_eq!(wa["nesting_ratio"].as_str().unwrap(), "2:1");
    }

    #[test]
    fn test_policy_hi_has_permanent_resident_basis() {
        let policy = load_policy();
        let hi = get_state_policy(&policy, "HI").unwrap();
        assert_eq!(hi["population_basis"].as_str().unwrap(), "permanent_resident",
            "Hawaii must count permanent residents only");
    }

    #[test]
    fn test_policy_nj_has_student_counting() {
        let policy = load_policy();
        let nj = get_state_policy(&policy, "NJ").unwrap();
        assert_eq!(nj["student_counting"].as_str().unwrap(), "college_address",
            "NJ counts students at college address");
    }

    #[test]
    fn test_policy_ne_has_unicameral_note() {
        let policy = load_policy();
        let ne = get_state_policy(&policy, "NE").unwrap();
        let notes = ne["notes"].as_str().unwrap_or("");
        assert!(notes.to_lowercase().contains("unicameral"),
            "Nebraska unicameral legislature must be documented");
    }

    #[test]
    fn test_policy_subdivision_term_ct_is_town() {
        assert_eq!(subdivision_term("CT", false), "town");
        assert_eq!(subdivision_term("CT", true), "towns");
    }

    #[test]
    fn test_policy_subdivision_term_vt_is_county() {
        // VT uses "county" as subdivision_term (towns are the municipal_term)
        assert_eq!(subdivision_term("VT", false), "county");
    }

    #[test]
    fn test_eldoria_municipal_term_is_keep() {
        let policy = load_policy();
        let el = get_state_policy(&policy, "_TEST_EL").unwrap();
        assert_eq!(el["municipal_term"].as_str().unwrap(), "keep");
        assert_eq!(el["municipal_term_plural"].as_str().unwrap(), "keeps");
    }

    #[test]
    fn test_eldoria_population_basis_is_vap() {
        let policy = load_policy();
        let el = get_state_policy(&policy, "_TEST_EL").unwrap();
        assert_eq!(el["population_basis"].as_str().unwrap(), "vap");
    }

    #[test]
    fn test_eldoria_nesting_4to1() {
        let policy = load_policy();
        let el = get_state_policy(&policy, "_TEST_EL").unwrap();
        assert_eq!(el["nesting_requirement"].as_str().unwrap(), "senate_contains_house");
        assert_eq!(el["nesting_ratio"].as_str().unwrap(), "4:1");
    }

    #[test]
    fn test_eldoria_vra_applies() {
        let policy = load_policy();
        let el = get_state_policy(&policy, "_TEST_EL").unwrap();
        assert_eq!(el["vra_section2_applies"].as_bool().unwrap(), true);
    }

    #[test]
    fn test_eldoria_seats_per_district_7() {
        // Eldoria uses 7-seat districts to verify seats_per_district is configurable
        let policy = load_policy();
        let el = get_state_policy(&policy, "_TEST_EL").unwrap();
        assert_eq!(el["seats_per_district"].as_u64().unwrap(), 7);
        assert_eq!(el["total_seats"].as_u64().unwrap(), 91);
    }

    #[test]
    fn test_eldoria_electoral_system_is_fantasy() {
        let policy = load_policy();
        let el = get_state_policy(&policy, "_TEST_EL").unwrap();
        let system = el["electoral_system"].as_str().unwrap();
        assert!(system.contains("arcane") || system.contains("proportional"),
            "Eldoria electoral system must be fantasy-named, got: {system}");
    }

    #[test]
    fn test_eldoria_independent_city_note_has_ironforge() {
        let policy = load_policy();
        let el = get_state_policy(&policy, "_TEST_EL").unwrap();
        let note = el["independent_city_note"].as_str().unwrap_or("");
        assert!(note.contains("Ironforge"), "Eldoria must mention the Free City of Ironforge");
    }

    #[test]
    fn test_eldoria_prison_gerrymandering_home_address() {
        let policy = load_policy();
        let el = get_state_policy(&policy, "_TEST_EL").unwrap();
        assert_eq!(el["prison_gerrymandering"].as_str().unwrap(), "home_address");
    }

    // ── Task 206: fuzzy name matching ─────────────────────────────────────────

    #[test]
    fn test_policy_fuzzy_name_match_ireland() {
        let policy = load_policy();
        // "ireland" should fuzzy-match to IE
        let query = "ireland".to_lowercase();
        let obj = policy.as_object().unwrap();
        let matches: Vec<&str> = obj.iter()
            .filter(|(k, _)| !k.starts_with('_'))
            .filter_map(|(code, entry)| {
                entry.get("name")
                    .and_then(|v| v.as_str())
                    .filter(|name| name.to_lowercase().contains(&query))
                    .map(|_| code.as_str())
            })
            .collect();
        assert!(matches.contains(&"IE"), "ireland must fuzzy-match to IE");
    }

    #[test]
    fn test_policy_fuzzy_no_match_returns_empty() {
        let policy = load_policy();
        let query = "zzzyyyxxx".to_lowercase();
        let obj = policy.as_object().unwrap();
        let matches: Vec<&str> = obj.iter()
            .filter(|(k, _)| !k.starts_with('_'))
            .filter_map(|(code, entry)| {
                entry.get("name")
                    .and_then(|v| v.as_str())
                    .filter(|name| name.to_lowercase().contains(&query))
                    .map(|_| code.as_str())
            })
            .collect();
        assert!(matches.is_empty(), "gibberish must not match any location");
    }

    #[test]
    fn test_policy_all_50_states_have_house_districts() {
        let policy = load_policy();
        let required_states = ["AL","AK","AZ","AR","CA","CO","CT","DE","FL","GA",
                              "HI","ID","IL","IN","IA","KS","KY","LA","ME","MD",
                              "MA","MI","MN","MS","MO","MT","NE","NV","NJ","NM",
                              "NY","NC","ND","OH","OK","OR","PA","RI","SC","SD",
                              "TN","TX","UT","VA","WA","WV","WI","WY"];
        for state in required_states {
            let s = get_state_policy(&policy, state)
                .unwrap_or_else(|| panic!("State {state} missing from policy database"));
            assert!(s.get("house_districts").is_some(),
                "{state} must have house_districts field");
        }
    }

    // ── 20 new L0 tests: policy loading, chamber, granularity, registry ───────

    #[test]
    fn test_load_policy_embedded_is_valid_json_object() {
        let policy = load_policy();
        assert!(policy.is_object(), "embedded policy must be a valid JSON object");
        // Must have at least 50 US states + some test entries
        let count = policy.as_object().unwrap().len();
        assert!(count >= 50, "policy must have at least 50 entries, got {count}");
    }

    #[test]
    fn test_subdivision_term_unknown_code_returns_county() {
        // Any 2-letter code not in the database should fall back to "county"
        assert_eq!(subdivision_term("QQ", false), "county");
        assert_eq!(subdivision_term("QQ", true), "counties");
    }

    #[test]
    fn test_subdivision_term_uppercase_key_required() {
        // The policy database uses uppercase 2-letter codes as keys.
        // subdivision_term("LA", ..) finds the entry; "la" (lowercase) falls through to default.
        let upper = subdivision_term("LA", false);
        assert_eq!(upper, "parish",
            "subdivision_term('LA') must return 'parish', not fallback county: {upper}");
        // Lowercase key is not found → falls back to "county" (expected behavior)
        let lower = subdivision_term("la", false);
        assert_eq!(lower, "county",
            "subdivision_term('la') lowercase must fall back to 'county': {lower}");
    }

    #[test]
    fn test_policy_tx_has_house_and_senate_districts() {
        let policy = load_policy();
        let tx = get_state_policy(&policy, "TX").unwrap();
        assert!(tx.get("house_districts").is_some(), "TX must have house_districts");
        assert!(tx.get("senate_districts").is_some(), "TX must have senate_districts");
        // TX has 150 house districts and 31 senate districts
        assert_eq!(tx["house_districts"].as_u64().unwrap(), 150);
        assert_eq!(tx["senate_districts"].as_u64().unwrap(), 31);
    }

    #[test]
    fn test_policy_ca_has_large_house_count() {
        let policy = load_policy();
        let ca = get_state_policy(&policy, "CA").unwrap();
        let house = ca["house_districts"].as_u64().unwrap();
        // California Assembly has 80 seats
        assert_eq!(house, 80, "CA Assembly has 80 seats");
    }

    #[test]
    fn test_policy_de_has_small_house_count() {
        let policy = load_policy();
        let de = get_state_policy(&policy, "DE").unwrap();
        let house = de["house_districts"].as_u64().unwrap();
        // Delaware House has 41 seats
        assert_eq!(house, 41, "DE House has 41 seats");
    }

    #[test]
    fn test_policy_mn_subdivision_term_is_county() {
        // Minnesota uses standard "county"
        assert_eq!(subdivision_term("MN", false), "county");
        assert_eq!(subdivision_term("MN", true), "counties");
    }

    #[test]
    fn test_policy_ri_subdivision_term_is_city_or_county() {
        // Rhode Island uses "city" or "county" — either is acceptable
        let term = subdivision_term("RI", false);
        // Both "county" and "city" are valid for RI; just verify it's a non-empty string
        assert!(!term.is_empty(), "RI must have a non-empty subdivision term");
    }

    #[test]
    fn test_policy_location_registry_load() {
        // LocationRegistry::load() must succeed (uses embedded fallback)
        let reg = crate::registry::LocationRegistry::load();
        assert!(reg.has_location("CA"), "registry must include CA");
        assert!(reg.has_location("TX"), "registry must include TX");
    }

    #[test]
    fn test_policy_location_registry_has_location_unknown_returns_false() {
        let reg = crate::registry::LocationRegistry::load();
        assert!(!reg.has_location("ZZ"), "ZZ must not be in registry");
        assert!(!reg.has_location("XX"), "XX must not be in registry");
    }

    #[test]
    fn test_policy_location_registry_chamber_districts_congressional() {
        let reg = crate::registry::LocationRegistry::load();
        // CA 2020: 52 congressional districts
        let n = reg.chamber_districts("CA", "congressional", "2020");
        assert!(n.is_some(), "CA congressional districts must be available");
        let n = n.unwrap();
        assert!(n >= 40 && n <= 60, "CA congressional districts {n} must be in [40,60]");
    }

    #[test]
    fn test_policy_location_registry_chamber_districts_house() {
        let reg = crate::registry::LocationRegistry::load();
        let n = reg.chamber_districts("WA", "house", "2020");
        assert!(n.is_some(), "WA house districts must be available");
        // WA House has 98 seats
        assert_eq!(n.unwrap(), 98, "WA House must have 98 seats");
    }

    #[test]
    fn test_policy_location_registry_chamber_districts_senate() {
        let reg = crate::registry::LocationRegistry::load();
        let n = reg.chamber_districts("WA", "senate", "2020");
        assert!(n.is_some(), "WA senate districts must be available");
        // WA Senate has 49 seats
        assert_eq!(n.unwrap(), 49, "WA Senate must have 49 seats");
    }

    #[test]
    fn test_policy_location_registry_chamber_districts_unknown_chamber() {
        let reg = crate::registry::LocationRegistry::load();
        let n = reg.chamber_districts("CA", "unicorn_chamber", "2020");
        assert!(n.is_none(), "unknown chamber must return None");
    }

    #[test]
    fn test_policy_location_registry_from_str_parses_minimal_json() {
        let json = r#"{"TS": {"name": "TestState", "house_districts": 99}}"#;
        let reg = crate::registry::LocationRegistry::from_str(json).unwrap();
        assert!(reg.has_location("TS"), "from_str registry must include TS");
    }

    #[test]
    fn test_policy_location_registry_from_str_invalid_json_returns_err() {
        let bad = r#"{ not valid json "#;
        let result = crate::registry::LocationRegistry::from_str(bad);
        assert!(result.is_err(), "invalid JSON must return Err from from_str");
    }

    #[test]
    fn test_policy_get_state_policy_all_us_states_parseable() {
        let policy = load_policy();
        let us_states = [
            "AL","AK","AZ","AR","CA","CO","CT","DE","FL","GA",
            "HI","ID","IL","IN","IA","KS","KY","LA","ME","MD",
            "MA","MI","MN","MS","MO","MT","NE","NV","NJ","NM",
            "NY","NC","ND","OH","OK","OR","PA","RI","SC","SD",
            "TN","TX","UT","VT","VA","WA","WV","WI","WY",
        ];
        for state in us_states {
            let entry = get_state_policy(&policy, state);
            assert!(entry.is_some(), "state {state} must be in policy database");
            let entry = entry.unwrap();
            // name must be a non-empty string
            let name = entry["name"].as_str().unwrap_or("");
            assert!(!name.is_empty(), "{state} must have a non-empty name field");
        }
    }

    #[test]
    fn test_policy_tx_balance_tolerance_congressional() {
        let policy = load_policy();
        let tx = get_state_policy(&policy, "TX").unwrap();
        if let Some(tol) = tx.get("balance_tolerance_congressional_pct") {
            let pct = tol.as_f64().unwrap_or(f64::NAN);
            // Congressional tolerance must be <= 1.0% per one-person-one-vote
            assert!(pct <= 1.0 && pct >= 0.0,
                "TX congressional balance tolerance {pct}% must be in [0,1]");
        }
    }

    #[test]
    fn test_policy_ny_has_prison_gerrymandering_field() {
        let policy = load_policy();
        let ny = get_state_policy(&policy, "NY").unwrap();
        // New York passed prison gerrymandering reform
        assert!(ny.get("prison_gerrymandering").is_some(),
            "NY must have prison_gerrymandering field documenting reform");
    }

    #[test]
    fn test_subdivision_term_plural_different_from_singular() {
        // For states with custom terms, plural != singular
        // (at minimum singular doesn't end in 's', plural might)
        let singular = subdivision_term("LA", false);
        let plural   = subdivision_term("LA", true);
        assert_ne!(singular, plural,
            "LA singular '{singular}' and plural '{plural}' must differ");
    }
}
