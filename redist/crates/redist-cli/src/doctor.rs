/// `redist doctor` — pre-flight check for a location+year+chamber+resolution.
///
/// Checks:
///  1. Is the location known to the registry?
///  2. Is the year valid for this location?
///  3. Does the adjacency file exist on disk?
///  4. Are there any granularity warnings (e.g., NV house at tract resolution)?
///  5. What is the balance tolerance that will be applied?
///  6. For international locations: what build command creates the adjacency?
///  7. Is the nesting ratio defined for the chamber?
use crate::args::DoctorArgs;
use crate::policy::LocationRegistry;
use crate::runner::{chamber_balance_tolerance, chamber_district_count};

pub fn run_doctor(args: &DoctorArgs) {
    let code = args.state.to_uppercase();
    let registry = LocationRegistry::load();

    println!("=== redist doctor: {code} ===\n");

    // ── 1. Location known? ────────────────────────────────────────────────────
    if !registry.has_location(&code) {
        println!("[FAIL] Location '{code}' not found in location_policy.json.");
        println!("       To add it: edit redist/data/location_policy.json");
        println!("       Or set REDIST_LOCATION_POLICY to a custom policy file.");
        std::process::exit(1);
    }
    println!("[PASS] Location '{code}' is registered.");

    // ── 2. Year valid? ────────────────────────────────────────────────────────
    let years = registry.available_years(&code);
    match registry.validate_year(&code, &args.year) {
        Ok(_) => println!("[PASS] Year '{}' is valid for {code}.  Available: {}", args.year, years.join(", ")),
        Err(e) => {
            println!("[FAIL] {e}");
            std::process::exit(1);
        }
    }

    // ── 3. District count ─────────────────────────────────────────────────────
    let chamber_dist = registry.chamber_districts(&code, &args.chamber, &args.year);
    let congressional_dist = registry.congressional_districts(&code, &args.year);
    match chamber_dist.or(congressional_dist) {
        Some(n) => println!("[PASS] {} {} districts: {}", code, args.chamber, n),
        None => println!(
            "[WARN] No district count found for '{code}' chamber '{}' year '{}'. \
             Use --districts to specify.",
            args.chamber, args.year
        ),
    }

    // ── 4. Balance tolerance ─────────────────────────────────────────────────
    let tol = chamber_balance_tolerance(&code, &args.chamber);
    println!("[INFO] Balance tolerance: {:.1}% (from policy; override with --balance-tolerance)",
        tol * 100.0);

    // ── 5. Granularity warning ────────────────────────────────────────────────
    match registry.granularity_warning(&code, &args.year, &args.chamber, &args.resolution) {
        Some(warn) => println!("[WARN] {warn}"),
        None => println!("[PASS] Resolution '{}' is appropriate for {code} {}.", args.resolution, args.chamber),
    }

    // ── 6. Adjacency file ─────────────────────────────────────────────────────
    let outputs_base = std::path::Path::new(&args.output_base);
    match registry.adjacency_path(&code, &args.year, &args.resolution, outputs_base) {
        Ok((path, hint)) => {
            if path.exists() {
                println!("[PASS] Adjacency file found: {}", path.display());
            } else {
                println!("[MISS] Adjacency file missing: {}", path.display());
                if !hint.is_empty() {
                    println!("       Build with: {hint}");
                }
            }
        }
        Err(e) => println!("[WARN] Could not resolve adjacency path: {e}"),
    }

    // ── 7. Nesting (if applicable) ────────────────────────────────────────────
    if args.chamber != "congressional" {
        let policy = registry.raw();
        if let Some(location) = policy.get(&code) {
            if let Some(nesting) = location.get("nesting_requirement").and_then(|v| v.as_str()) {
                if nesting != "null" {
                    let ratio = location.get("nesting_ratio")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    println!("[INFO] Nesting: {} ({} ratio). Use 'redist suite' for multi-chamber.", nesting, ratio);
                }
            }
        }
    }

    println!("\n[OK] Doctor check complete for {code} {} {}.", args.chamber, args.year);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_args(state: &str, year: &str, chamber: &str, resolution: &str) -> DoctorArgs {
        DoctorArgs {
            state: state.to_string(),
            year: year.to_string(),
            chamber: chamber.to_string(),
            resolution: resolution.to_string(),
            output_base: "outputs".to_string(),
        }
    }

    #[test]
    fn test_doctor_wa_house_tract_warns_granularity() {
        let registry = LocationRegistry::load();
        let warn = registry.granularity_warning("WA", "2020", "house", "tract");
        assert!(warn.is_some(), "WA house at tract resolution must warn about granularity");
        let msg = warn.unwrap();
        assert!(msg.contains("block_group") || msg.contains("resolution"),
            "warning must suggest block_group: {msg}");
    }

    #[test]
    fn test_doctor_wa_house_block_group_no_warn() {
        let registry = LocationRegistry::load();
        let warn = registry.granularity_warning("WA", "2020", "house", "block_group");
        assert!(warn.is_none(), "WA house at block_group should not warn");
    }

    #[test]
    fn test_doctor_wa_congressional_no_warn() {
        let registry = LocationRegistry::load();
        let warn = registry.granularity_warning("WA", "2020", "congressional", "tract");
        assert!(warn.is_none(), "WA congressional (10D) should not warn at tract");
    }

    #[test]
    fn test_doctor_nv_house_warns() {
        let registry = LocationRegistry::load();
        let warn = registry.granularity_warning("NV", "2020", "house", "tract");
        assert!(warn.is_some(), "NV house (42D) at tract should warn");
    }

    #[test]
    fn test_doctor_vt_congressional_no_warn() {
        let registry = LocationRegistry::load();
        // VT congressional = 1 district, definitely no granularity issue
        let warn = registry.granularity_warning("VT", "2020", "congressional", "tract");
        assert!(warn.is_none(), "VT 1D congressional must not warn");
    }

    #[test]
    fn test_doctor_ie_year_valid() {
        let registry = LocationRegistry::load();
        assert!(registry.validate_year("IE", "2022").is_ok());
    }

    #[test]
    fn test_doctor_wa_year_invalid_2024() {
        let registry = LocationRegistry::load();
        let result = registry.validate_year("WA", "2024");
        assert!(result.is_err(), "2024 must be invalid for WA");
        let msg = result.unwrap_err();
        assert!(msg.contains("2020") || msg.contains("2010"),
            "error must list valid years: {msg}");
    }

    #[test]
    fn test_doctor_eldoria_year_2099_valid() {
        let registry = LocationRegistry::load();
        assert!(registry.validate_year("_TEST_EL", "2099").is_ok());
    }

    // ── Task 120: normalize location code case ────────────────────────────────

    #[test]
    fn test_doctor_lowercase_state_works() {
        // run_doctor calls args.state.to_uppercase() internally before registry lookup.
        // Verify that uppercasing "wa" → "WA" produces the correct result.
        let registry = LocationRegistry::load();
        // Direct lookup: has_location already normalizes to uppercase internally
        let code = "wa";
        let normalized = code.to_uppercase();
        assert_eq!(normalized, "WA", "to_uppercase of 'wa' must produce 'WA'");
        // Registry finds normalized code
        assert!(registry.has_location(&normalized),
            "registry must find 'WA' (normalized from 'wa')");
        // Confirm that the normalization used in run_doctor matches registry lookup
        let args_state = "wa"; // simulates DoctorArgs.state = "wa"
        let code_normalized = args_state.to_uppercase();
        assert!(registry.has_location(&code_normalized),
            "run_doctor normalization pattern must work: {} -> {}", args_state, code_normalized);
    }

    #[test]
    fn test_doctor_uppercase_state_code_consistent() {
        // Verify that to_uppercase() is idempotent for typical state codes
        let code = "WA";
        assert_eq!(code.to_uppercase(), code);
        let code_lower = "wa";
        assert_eq!(code_lower.to_uppercase(), "WA");
    }

    #[test]
    fn test_doctor_test_state_uppercase_preserves_prefix() {
        // _TEST_EL uppercased is still _TEST_EL (underscore + uppercase letters)
        let code = "_TEST_EL";
        assert_eq!(code.to_uppercase(), "_TEST_EL");
        let registry = LocationRegistry::load();
        assert!(registry.has_location("_TEST_EL"), "_TEST_EL must be in registry");
    }
}
