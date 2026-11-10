//! In-memory native registry lookups; hosts own file and manifest discovery.
pub struct PolicyRegistry<'a>{raw:&'a serde_json::Value}
impl<'a> PolicyRegistry<'a>{
    pub fn new(raw:&'a serde_json::Value)->Self{Self{raw}}
    /// Return true if `code` (case-insensitive lookup via uppercase) exists in the registry.
    pub fn has_location(&self, code: &str) -> bool {
        let key = code.to_uppercase();
        // _TEST_EL is stored as-is (leading underscore, already uppercased)
        // Handle case where code already has underscore prefix
        if self.raw.get(&key).is_some() {
            return true;
        }
        false
    }

    /// Return the years available for a location, as Strings (e.g. ["2000","2010","2020"]).
    /// Returns empty vec if location not found or `census_years` is absent.
    pub fn available_years(&self, code: &str) -> Vec<String> {
        let key = self.resolve_key(code);
        self.raw
            .get(&key)
            .and_then(|e| e.get("census_years"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Validate that `year` is in the location's available years.
    /// Returns Ok(year.to_string()) on success, Err with message on failure.
    pub fn validate_year(&self, code: &str, year: &str) -> Result<String, String> {
        if !self.has_location(code) {
            return Err(format!("unknown location '{code}'"));
        }
        let years = self.available_years(code);
        if years.is_empty() || years.contains(&year.to_string()) {
            // Empty years = no restriction (backward compat); or year found
            Ok(year.to_string())
        } else {
            Err(format!(
                "year '{year}' not available for '{code}'. Available years: {}",
                years.join(", ")
            ))
        }
    }

    /// Return the congressional district count for a location and year.
    /// Hosts may add manifest fallback when the policy count is absent.
    pub fn congressional_districts(&self, code: &str, year: &str) -> Option<u64> {
        let key = self.resolve_key(code);
        let entry = self.raw.get(&key)?;

        // Primary: congressional_districts_by_year field
        if let Some(by_year) = entry.get("congressional_districts_by_year") {
            if let Some(n) = by_year.get(year).and_then(|v| v.as_u64()) {
                if n > 0 {
                    return Some(n);
                }
            }
        }

        // Manifest discovery belongs to the host.
        None
    }

    /// Return district count for a given chamber and year.
    ///
    /// "congressional" → uses `congressional_districts_by_year` (hosts may add manifest fallback)
    /// "house" / "lower" / "assembly" → uses `house_districts` field
    /// "senate" / "upper" → uses `senate_districts` field
    /// Others → None
    pub fn chamber_districts(&self, code: &str, chamber: &str, year: &str) -> Option<u64> {
        match chamber {
            "congressional" => self.congressional_districts(code, year),
            "house" | "lower" | "assembly" => {
                let key = self.resolve_key(code);
                self.raw
                    .get(&key)
                    .and_then(|e| e.get("house_districts"))
                    .and_then(|v| v.as_u64())
                    .filter(|&n| n > 0)
                    .map(|n| n)
            }
            "senate" | "upper" => {
                let key = self.resolve_key(code);
                self.raw
                    .get(&key)
                    .and_then(|e| e.get("senate_districts"))
                    .and_then(|v| v.as_u64())
                    .filter(|&n| n > 0)
                    .map(|n| n)
            }
            _ => None,
        }
    }

    pub fn granularity_warning(
        &self,
        code: &str,
        year: &str,
        chamber: &str,
        resolution: &str,
    ) -> Option<String> {
        if resolution != "tract" || chamber != "house" {
            return None;
        }
        let n = self.chamber_districts(code, chamber, year)?;
        if n <= 25 {
            return None;
        }
        let state_name = self.state_name(code).unwrap_or_else(|| code.to_string());
        Some(format!(
            "{state_name} house has {n} districts. \
             Tract resolution may hit the granularity floor (~1 tract/2 districts). \
             Use --resolution block_group for better balance."
        ))
    }

    /// Return the human-readable name for a location code (e.g. "Washington").
    pub fn state_name(&self, code: &str) -> Option<String> {
        let key = self.resolve_key(code);
        self.raw
            .get(&key)
            .and_then(|e| e.get("name"))
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    // ── Private helpers ────────────────────────────────────────────────────────

    /// Resolve the canonical JSON key for a location code.
    ///
    /// Most codes are uppercased. _TEST_EL uses underscore prefix but is already
    /// correctly cased when uppercased.
    fn resolve_key(&self, code: &str) -> String {
        // Try uppercase first (covers all US states + international codes)
        let upper = code.to_uppercase();
        if self.raw.get(&upper).is_some() {
            return upper;
        }
        // Try as-is (covers _TEST_EL which starts with underscore and is stored as-is)
        if self.raw.get(code).is_some() {
            return code.to_string();
        }
        // Default: uppercase
        upper
    }

}

pub fn chamber_balance_tolerance(policy:&serde_json::Value,state_code:&str,chamber:&str)->f64{
    if let Some(state) = policy.get(state_code) {
        let key = match chamber {
            "congressional" => "balance_tolerance_congressional_pct",
            "house" | "lower" | "assembly" => "balance_tolerance_house_pct",
            "senate" | "upper" => "balance_tolerance_senate_pct",
            _ => "",
        };
        if !key.is_empty() {
            if let Some(pct) = state.get(key).and_then(|v| v.as_f64()) {
                if pct > 0.0 {
                    return pct / 100.0; // policy stores in %, we use fraction
                }
            }
        }
    }
    // Fallback: constitutional standard for one-person-one-vote
    match chamber {
        "congressional" => 0.005,
        _ => 0.05,
    }
}

#[cfg(test)]mod tests {
 use super::*;use serde_json::json;
 #[test]fn native_year_alias_and_granularity_rules(){let data=json!({"XX":{"name":"Example","house_districts":26,"senate_districts":13,"census_years":["2020"],"congressional_districts_by_year":{"2020":2}}});let r=PolicyRegistry::new(&data);assert!(r.has_location("xx"));assert!(r.validate_year("XX","2010").is_err());assert_eq!(r.chamber_districts("xx","assembly","2020"),Some(26));assert!(r.granularity_warning("XX","2020","house","tract").is_some());assert!(r.granularity_warning("XX","2020","lower","tract").is_none());assert!(r.granularity_warning("XX","2020","house","block_group").is_none());}
 #[test]fn absent_years_and_manifest_counts_remain_explicit(){let data=json!({"XX":{"house_districts":0}});let r=PolicyRegistry::new(&data);assert!(r.validate_year("XX","arbitrary").is_ok());assert!(r.congressional_districts("XX","2020").is_none());assert!(r.chamber_districts("XX","house","2020").is_none());assert!(r.validate_year("ZZ","2020").is_err());}
 #[test]fn native_tolerance_case_and_fallback_rules(){let data=json!({"XX":{"balance_tolerance_house_pct":2.5}});assert_eq!(chamber_balance_tolerance(&data,"XX","assembly"),0.025);assert_eq!(chamber_balance_tolerance(&data,"xx","house"),0.05);assert_eq!(chamber_balance_tolerance(&data,"ZZ","congressional"),0.005);}
}
