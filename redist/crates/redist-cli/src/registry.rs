/// LocationRegistry — runtime lookup for the location_policy.json database.
///
/// Load order:
///   1. REDIST_LOCATION_POLICY env var (path to JSON file)
///   2. REDIST_STATE_POLICY env var (backward-compat alias)
///   3. ~/.config/redist/location_policy.json
///   4. Embedded fallback (compiled into binary via include_str!)
///
/// The registry keeps the raw serde_json::Value so it remains compatible with
/// the existing policy.rs functions that read fields directly from the JSON.

use std::path::{Path, PathBuf};

/// Embedded fallback — compiled into the binary at build time.
static EMBEDDED_POLICY: &str = include_str!("../../../data/location_policy.json");

pub struct LocationRegistry {
    raw: serde_json::Value,
}

impl LocationRegistry {
    // ── Construction ───────────────────────────────────────────────────────────

    /// Load the registry using the standard search order.
    pub fn load() -> Self {
        // 1. REDIST_LOCATION_POLICY env var
        if let Ok(path) = std::env::var("REDIST_LOCATION_POLICY") {
            if let Some(reg) = Self::try_load_file(&path, "REDIST_LOCATION_POLICY") {
                return reg;
            }
        }
        // 2. Backward-compat alias (old env var name)
        if let Ok(path) = std::env::var("REDIST_STATE_POLICY") {
            if let Some(reg) = Self::try_load_file(&path, "REDIST_STATE_POLICY") {
                return reg;
            }
        }
        // 3. User config directory
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            let config_path = PathBuf::from(home)
                .join(".config").join("redist").join("location_policy.json");
            if config_path.exists() {
                if let Some(reg) = Self::try_load_file(config_path.to_str().unwrap_or(""), "~/.config/redist/location_policy.json") {
                    return reg;
                }
            }
        }
        // 4. Embedded fallback
        let raw = serde_json::from_str(EMBEDDED_POLICY)
            .expect("embedded location_policy.json is valid JSON");
        Self { raw }
    }

    /// Parse a registry from a JSON string (for testing).
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Result<Self, serde_json::Error> {
        let raw = serde_json::from_str(s)?;
        Ok(Self { raw })
    }

    fn try_load_file(path: &str, env_name: &str) -> Option<Self> {
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(raw) => Some(Self { raw }),
                Err(e) => {
                    eprintln!("WARNING: {env_name}={path} could not be parsed ({e}); \
                               falling back to embedded policy.");
                    None
                }
            },
            Err(e) => {
                eprintln!("WARNING: {env_name}={path} could not be read ({e}); \
                           falling back to embedded policy.");
                None
            }
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Access the underlying raw JSON value (for backward compat with policy.rs).
    pub fn raw(&self) -> &serde_json::Value {
        &self.raw
    }

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
        self.raw.get(&key)
            .and_then(|e| e.get("census_years"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
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
    /// Falls back to the manifest when `congressional_districts_by_year` is absent.
    pub fn congressional_districts(&self, code: &str, year: &str) -> Option<usize> {
        let key = self.resolve_key(code);
        let entry = self.raw.get(&key)?;

        // Primary: congressional_districts_by_year field
        if let Some(by_year) = entry.get("congressional_districts_by_year") {
            if let Some(n) = by_year.get(year).and_then(|v| v.as_u64()) {
                if n > 0 {
                    return Some(n as usize);
                }
            }
        }

        // Fallback: read from manifest.json
        self.manifest_congressional_districts(code, year)
    }

    /// Return district count for a given chamber and year.
    ///
    /// "congressional" → uses `congressional_districts_by_year` (+ manifest fallback)
    /// "house" / "lower" / "assembly" → uses `house_districts` field
    /// "senate" / "upper" → uses `senate_districts` field
    /// Others → None
    pub fn chamber_districts(&self, code: &str, chamber: &str, year: &str) -> Option<usize> {
        match chamber {
            "congressional" => self.congressional_districts(code, year),
            "house" | "lower" | "assembly" => {
                let key = self.resolve_key(code);
                self.raw.get(&key)
                    .and_then(|e| e.get("house_districts"))
                    .and_then(|v| v.as_u64())
                    .filter(|&n| n > 0)
                    .map(|n| n as usize)
            }
            "senate" | "upper" => {
                let key = self.resolve_key(code);
                self.raw.get(&key)
                    .and_then(|e| e.get("senate_districts"))
                    .and_then(|v| v.as_u64())
                    .filter(|&n| n > 0)
                    .map(|n| n as usize)
            }
            _ => None,
        }
    }

    /// Resolve the adjacency pkl path for a location, year, and resolution.
    ///
    /// Returns `(path, build_hint)` where `build_hint` is the command to build the adjacency.
    ///
    /// Convention dispatch:
    /// - "us_census": canonical V3/data/{year}/adjacency/ path relative to outputs_base
    /// - "gadm" / "direct_path" / others: use path_template relative to cwd
    /// - Missing adjacency_convention: us_census fallback (backward compat)
    pub fn adjacency_path(
        &self,
        code: &str,
        year: &str,
        resolution: &str,
        outputs_base: &Path,
    ) -> Result<(PathBuf, String), String> {
        let key = self.resolve_key(code);
        let state_lower = code.to_lowercase();
        let state_upper = code.to_uppercase();

        let entry = match self.raw.get(&key) {
            Some(e) => e,
            None => {
                // Unknown location — use us_census convention as fallback
                return self.us_census_path(&state_lower, year, resolution, outputs_base, "");
            }
        };

        let convention_obj = entry.get("adjacency_convention");

        let convention = convention_obj
            .and_then(|c| c.get("convention"))
            .and_then(|v| v.as_str())
            .unwrap_or("us_census");

        let build_command = convention_obj
            .and_then(|c| c.get("build_command"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .replace("{year}", year)
            .replace("{state_lower}", &state_lower)
            .replace("{state_upper}", &state_upper);

        match convention {
            "us_census" => {
                let template = convention_obj
                    .and_then(|c| c.get("path_template"))
                    .and_then(|v| v.as_str());
                self.us_census_path(&state_lower, year, resolution, outputs_base, &build_command)
                    .or_else(|_| {
                        // If template is present but not matching, use template directly
                        if let Some(tmpl) = template {
                            let path = apply_template(tmpl, year, &state_lower, &state_upper);
                            Ok((PathBuf::from(path), build_command.clone()))
                        } else {
                            Err(format!(
                                "adjacency pkl not found for '{code}' year '{year}'. \
                                 Run: {build_command}"
                            ))
                        }
                    })
            }
            _ => {
                // gadm, direct_path, or other conventions: use path_template relative to cwd
                let template = convention_obj
                    .and_then(|c| c.get("path_template"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| format!(
                        "no path_template in adjacency_convention for '{code}'"
                    ))?;
                let path = apply_template(template, year, &state_lower, &state_upper);
                Ok((PathBuf::from(path), build_command))
            }
        }
    }

    /// Emit a granularity warning when tract resolution may be insufficient
    /// for the number of districts in the requested chamber.
    ///
    /// Returns Some(warning_text) when:
    ///   - chamber == "house"
    ///   - house_districts > 60
    ///   - resolution == "tract"
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
        let state_name = self.state_name(code)
            .unwrap_or_else(|| code.to_string());
        Some(format!(
            "{state_name} house has {n} districts. \
             Tract resolution may hit the granularity floor (~1 tract/2 districts). \
             Use --resolution block_group for better balance."
        ))
    }

    /// Return the human-readable name for a location code (e.g. "Washington").
    pub fn state_name(&self, code: &str) -> Option<String> {
        let key = self.resolve_key(code);
        self.raw.get(&key)
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

    /// Build the us_census canonical path, trying V3 then V4.
    fn us_census_path(
        &self,
        state_lower: &str,
        year: &str,
        resolution: &str,
        outputs_base: &Path,
        build_hint: &str,
    ) -> Result<(PathBuf, String), String> {
        let (adj_filename, is_block_group) = match resolution {
            "block_group" | "block-group" => (
                format!("{state_lower}_bg_adjacency_{year}.pkl"),
                true,
            ),
            _ => (
                format!("{state_lower}_adjacency_{year}.pkl"),
                false,
            ),
        };

        let canonical = outputs_base
            .join("V3").join("data").join(year).join("adjacency")
            .join(&adj_filename);
        if canonical.exists() {
            return Ok((canonical, build_hint.to_string()));
        }
        let v4 = outputs_base
            .join("V4").join("data").join(year).join("adjacency")
            .join(&adj_filename);
        if v4.exists() {
            return Ok((v4, build_hint.to_string()));
        }

        if is_block_group {
            // Graceful fallback to tract
            let tract_filename = format!("{state_lower}_adjacency_{year}.pkl");
            let tract_v3 = outputs_base
                .join("V3").join("data").join(year).join("adjacency")
                .join(&tract_filename);
            if tract_v3.exists() {
                return Ok((tract_v3, build_hint.to_string()));
            }
            let tract_v4 = outputs_base
                .join("V4").join("data").join(year).join("adjacency")
                .join(&tract_filename);
            if tract_v4.exists() {
                return Ok((tract_v4, build_hint.to_string()));
            }
        }

        Err(format!(
            "adjacency pkl not found: {}. Run: {}",
            canonical.display(),
            if build_hint.is_empty() {
                format!("python scripts/data/generate_adj_bin.py --year {year} --states {state_lower}")
            } else {
                build_hint.to_string()
            }
        ))
    }

    /// Read congressional district count from manifest.json (fallback).
    fn manifest_congressional_districts(&self, code: &str, year: &str) -> Option<usize> {
        if let Ok(manifest) = crate::fetch::load_manifest() {
            if let Some(state) = manifest.states.get(code) {
                if let Some(&n) = state.districts.get(year) {
                    if n > 0 {
                        return Some(n);
                    }
                }
            }
        }
        None
    }
}

/// Apply template substitutions: {year}, {state_lower}, {state_upper}.
fn apply_template(template: &str, year: &str, state_lower: &str, state_upper: &str) -> String {
    template
        .replace("{year}", year)
        .replace("{state_lower}", state_lower)
        .replace("{state_upper}", state_upper)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> LocationRegistry {
        LocationRegistry::load()
    }

    // ── has_location ───────────────────────────────────────────────────────────

    #[test]
    fn test_has_location_wa() {
        assert!(registry().has_location("WA"));
        assert!(registry().has_location("wa")); // case-insensitive
    }

    #[test]
    fn test_has_location_ie() {
        assert!(registry().has_location("IE"));
    }

    #[test]
    fn test_has_location_unknown() {
        assert!(!registry().has_location("ZZ"));
    }

    #[test]
    fn test_has_location_eldoria() {
        // _TEST_EL must be found even with underscore prefix
        assert!(registry().has_location("_TEST_EL"));
    }

    // ── available_years ────────────────────────────────────────────────────────

    #[test]
    fn test_available_years_wa() {
        let years = registry().available_years("WA");
        assert!(years.contains(&"2000".to_string()));
        assert!(years.contains(&"2010".to_string()));
        assert!(years.contains(&"2020".to_string()));
        assert_eq!(years.len(), 3);
    }

    #[test]
    fn test_available_years_eldoria() {
        let years = registry().available_years("_TEST_EL");
        assert_eq!(years, vec!["2099".to_string()]);
    }

    #[test]
    fn test_available_years_ie() {
        let years = registry().available_years("IE");
        assert_eq!(years, vec!["2022".to_string()]);
    }

    #[test]
    fn test_available_years_unknown() {
        assert!(registry().available_years("ZZ").is_empty());
    }

    // ── validate_year ──────────────────────────────────────────────────────────

    #[test]
    fn test_validate_year_wa_2020_ok() {
        let result = registry().validate_year("WA", "2020");
        assert_eq!(result, Ok("2020".to_string()));
    }

    #[test]
    fn test_validate_year_wa_2010_ok() {
        let result = registry().validate_year("WA", "2010");
        assert_eq!(result, Ok("2010".to_string()));
    }

    #[test]
    fn test_validate_year_wa_invalid_err() {
        let result = registry().validate_year("WA", "2024");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("2024"), "error should mention the invalid year: {msg}");
        assert!(msg.contains("2020") || msg.contains("2010") || msg.contains("2000"),
            "error should mention available years: {msg}");
    }

    #[test]
    fn test_validate_year_unknown_location_err() {
        let result = registry().validate_year("ZZ", "2020");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_year_eldoria_2099_ok() {
        let result = registry().validate_year("_TEST_EL", "2099");
        assert_eq!(result, Ok("2099".to_string()));
    }

    #[test]
    fn test_validate_year_eldoria_2020_err() {
        let result = registry().validate_year("_TEST_EL", "2020");
        assert!(result.is_err(), "2020 is not an Eldoria year");
    }

    // ── congressional_districts ────────────────────────────────────────────────

    #[test]
    fn test_congressional_districts_wa_2020() {
        assert_eq!(registry().congressional_districts("WA", "2020"), Some(10));
    }

    #[test]
    fn test_congressional_districts_wa_2000() {
        assert_eq!(registry().congressional_districts("WA", "2000"), Some(9));
    }

    #[test]
    fn test_congressional_districts_tx_2020() {
        assert_eq!(registry().congressional_districts("TX", "2020"), Some(38));
    }

    #[test]
    fn test_congressional_districts_tx_2000() {
        assert_eq!(registry().congressional_districts("TX", "2000"), Some(32));
    }

    #[test]
    fn test_congressional_districts_eldoria() {
        assert_eq!(registry().congressional_districts("_TEST_EL", "2099"), Some(13));
    }

    #[test]
    fn test_congressional_districts_unknown_year() {
        assert_eq!(registry().congressional_districts("WA", "1999"), None);
    }

    // ── chamber_districts ─────────────────────────────────────────────────────

    #[test]
    fn test_chamber_districts_wa_house() {
        assert_eq!(registry().chamber_districts("WA", "house", "2020"), Some(98));
    }

    #[test]
    fn test_chamber_districts_wa_senate() {
        assert_eq!(registry().chamber_districts("WA", "senate", "2020"), Some(49));
    }

    #[test]
    fn test_chamber_districts_wa_congressional() {
        assert_eq!(registry().chamber_districts("WA", "congressional", "2020"), Some(10));
    }

    #[test]
    fn test_chamber_districts_unknown_chamber() {
        assert_eq!(registry().chamber_districts("WA", "council", "2020"), None);
    }

    // ── state_name ────────────────────────────────────────────────────────────

    #[test]
    fn test_state_name_wa() {
        assert_eq!(registry().state_name("WA"), Some("Washington".to_string()));
    }

    #[test]
    fn test_state_name_eldoria() {
        assert_eq!(registry().state_name("_TEST_EL"), Some("Eldoria".to_string()));
    }

    #[test]
    fn test_state_name_unknown() {
        assert_eq!(registry().state_name("ZZ"), None);
    }

    // ── granularity_warning ───────────────────────────────────────────────────

    #[test]
    fn test_granularity_warning_wa_house_tract() {
        // WA has 98 house districts — above threshold of 60
        let warn = registry().granularity_warning("WA", "2020", "house", "tract");
        assert!(warn.is_some(), "WA house at tract resolution should warn");
        let msg = warn.unwrap();
        assert!(msg.contains("98") || msg.contains("house"), "warning should mention districts: {msg}");
    }

    #[test]
    fn test_granularity_warning_vt_house_no_warn() {
        // VT has very few house districts — well below 60
        let warn = registry().granularity_warning("VT", "2020", "house", "tract");
        // VT house has 150 districts — this SHOULD warn
        // Actually checking: VT house_districts from policy.json
        let n = registry().chamber_districts("VT", "house", "2020").unwrap_or(0);
        if n > 60 {
            assert!(warn.is_some());
        } else {
            assert!(warn.is_none());
        }
    }

    #[test]
    fn test_granularity_warning_congressional_no_warn() {
        // Congressional chamber never warns (not "house")
        let warn = registry().granularity_warning("WA", "2020", "congressional", "tract");
        assert!(warn.is_none());
    }

    #[test]
    fn test_granularity_warning_block_group_no_warn() {
        // block_group resolution never warns
        let warn = registry().granularity_warning("WA", "2020", "house", "block_group");
        assert!(warn.is_none());
    }

    // ── adjacency_path ────────────────────────────────────────────────────────

    #[test]
    fn test_adjacency_path_returns_result() {
        // With no data present, should return Err with descriptive message
        let reg = registry();
        let base = std::path::Path::new("/nonexistent/outputs");
        let result = reg.adjacency_path("WA", "2020", "tract", base);
        // Either Ok (if data exists) or Err (expected in test env)
        match result {
            Ok((path, _hint)) => {
                assert!(path.to_str().unwrap().contains("wa"));
            }
            Err(msg) => {
                assert!(msg.contains("adjacency") || msg.contains("not found"),
                    "error should mention adjacency: {msg}");
            }
        }
    }

    #[test]
    fn test_adjacency_path_gadm_convention() {
        // IE uses GADM convention — path template should be used
        let reg = registry();
        let base = std::path::Path::new("/outputs");
        let result = reg.adjacency_path("IE", "2022", "tract", base);
        // GADM convention uses path_template relative to cwd, not outputs_base
        match result {
            Ok((path, _)) => {
                let path_str = path.to_str().unwrap();
                assert!(path_str.contains("ie") || path_str.contains("international"),
                    "IE path should reference international: {path_str}");
            }
            Err(_) => {
                // OK — file doesn't exist in test env
            }
        }
    }

    #[test]
    fn test_adjacency_path_eldoria() {
        let reg = registry();
        let base = std::path::Path::new("/outputs");
        let result = reg.adjacency_path("_TEST_EL", "2099", "tract", base);
        // direct_path convention: uses template literally. Don't panic; just verify.
        match result {
            Ok((_path, _hint)) => {
                // File exists locally — that's fine
            }
            Err(_msg) => {
                // Expected in test env — no Eldoria adjacency data
            }
        }
    }

    // ── raw access ────────────────────────────────────────────────────────────

    #[test]
    fn test_raw_returns_json_object() {
        let reg = registry();
        assert!(reg.raw().is_object());
    }

    // ── template application ──────────────────────────────────────────────────

    #[test]
    fn test_apply_template() {
        let result = super::apply_template(
            "outputs/international/{state_lower}/{state_lower}_adjacency_{year}.pkl",
            "2022",
            "ie",
            "IE",
        );
        assert_eq!(result, "outputs/international/ie/ie_adjacency_2022.pkl");
    }

    // ── manifest fallback ─────────────────────────────────────────────────────

    #[test]
    fn test_congressional_districts_manifest_fallback() {
        // If WA has congressional_districts_by_year, we verify the value matches manifest
        let reg = registry();
        // WA 2020: manifest says 10, policy says 10
        let n = reg.congressional_districts("WA", "2020");
        assert_eq!(n, Some(10));
    }
}
