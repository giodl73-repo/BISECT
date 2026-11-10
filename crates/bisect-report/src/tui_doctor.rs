//! Shared native TUI Doctor messages; hosts supply policy, count and tolerance observations.
use serde::{Deserialize, Serialize};
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_location_stops_checks() {
        let checks = checks(&serde_json::json!({}), "ZZ", "house", "2020", Some(1), 0.05);
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Fail);
        assert!(checks[0]
            .hint
            .as_ref()
            .unwrap()
            .contains("BISECT_LOCATION_POLICY"));
    }
    #[test]
    fn native_hints_messages_and_order() {
        let raw = serde_json::json!({"XX":{"name":"Example","house_districts":75,"census_years":["2020"],"compactness_standard":"specified","nesting_requirement":"required","nesting_ratio":"3:1"}});
        let checks = checks(&raw, "XX", "house", "2020", Some(75), 0.025);
        assert_eq!(checks.len(), 7);
        assert_eq!(checks[3].message, "Balance tolerance for 'house': 2.5%");
        assert_eq!(checks[4].status, CheckStatus::Warn);
        assert_eq!(checks[6].message, "Nesting requirement: required (3:1)");
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub status: CheckStatus,
    pub message: String,
    pub hint: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Info,
}
pub fn checks(
    raw: &serde_json::Value,
    location: &str,
    chamber: &str,
    year: &str,
    district_count: Option<u64>,
    tolerance: f64,
) -> Vec<DoctorCheck> {
    let registry = bisect_analysis::location_registry::PolicyRegistry::new(raw);
    let mut checks = Vec::new();

    // 1. has_location
    if registry.has_location(location) {
        let name = registry
            .state_name(location)
            .unwrap_or_else(|| location.to_string());
        checks.push(DoctorCheck {
            status: CheckStatus::Pass,
            message: format!("Location '{location}' found: {name}"),
            hint: None,
        });
    } else {
        checks.push(DoctorCheck {
            status: CheckStatus::Fail,
            message: format!("Location '{location}' not found in policy database"),
            hint: Some(
                "Check BISECT_LOCATION_POLICY or add the location to location_policy.json"
                    .to_string(),
            ),
        });
        // Early return — remaining checks are meaningless without a valid location
        return checks;
    }

    // 2. validate_year
    match registry.validate_year(location, year) {
        Ok(_) => {
            checks.push(DoctorCheck {
                status: CheckStatus::Pass,
                message: format!("Year '{year}' is available for '{location}'"),
                hint: None,
            });
        }
        Err(msg) => {
            checks.push(DoctorCheck {
                status: CheckStatus::Fail,
                message: msg.clone(),
                hint: Some(format!(
                    "Available years: {}",
                    registry.available_years(location).join(", ")
                )),
            });
        }
    }

    // 3. chamber_districts
    match district_count {
        Some(0) => {
            checks.push(DoctorCheck {
                status: CheckStatus::Warn,
                message: format!("Chamber '{chamber}' has 0 districts (unicameral or no chamber?)"),
                hint: Some("Use --chamber house for unicameral legislatures (e.g. NE)".to_string()),
            });
        }
        Some(n) => {
            checks.push(DoctorCheck {
                status: CheckStatus::Pass,
                message: format!("Chamber '{chamber}' has {n} districts"),
                hint: None,
            });
        }
        None => {
            checks.push(DoctorCheck {
                status: CheckStatus::Warn,
                message: format!("Chamber '{chamber}' district count not found for '{location}'"),
                hint: Some("Try 'congressional', 'house', or 'senate'".to_string()),
            });
        }
    }

    // 4. chamber_balance_tolerance
    let tol = tolerance;
    checks.push(DoctorCheck {
        status: CheckStatus::Info,
        message: format!("Balance tolerance for '{chamber}': {:.1}%", tol * 100.0),
        hint: None,
    });

    // 5. granularity_warning
    match registry.granularity_warning(location, year, chamber, "tract") {
        Some(warn) => {
            checks.push(DoctorCheck {
                status: CheckStatus::Warn,
                message: warn,
                hint: Some("Use --resolution block_group for better balance".to_string()),
            });
        }
        None => {
            checks.push(DoctorCheck {
                status: CheckStatus::Pass,
                message: "Tract resolution is sufficient for this chamber".to_string(),
                hint: None,
            });
        }
    }

    // 6. compactness_standard from raw()
    let location_upper = location.to_uppercase();
    if let Some(entry) = raw.get(&location_upper).or_else(|| raw.get(location)) {
        if let Some(cs) = entry.get("compactness_standard").and_then(|v| v.as_str()) {
            checks.push(DoctorCheck {
                status: CheckStatus::Info,
                message: format!("Compactness standard: {cs}"),
                hint: None,
            });
        }

        // 7. nesting from raw()
        if let Some(nr) = entry.get("nesting_requirement").and_then(|v| v.as_str()) {
            let ratio = entry
                .get("nesting_ratio")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            checks.push(DoctorCheck {
                status: CheckStatus::Info,
                message: format!("Nesting requirement: {nr} ({ratio})"),
                hint: None,
            });
        }
    }

    checks
}
