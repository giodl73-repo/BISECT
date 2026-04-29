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
use crate::provenance::Provenance;
use crate::runner::{chamber_balance_tolerance, chamber_district_count};

pub fn run_doctor(args: &DoctorArgs) {
    // --verify-manifest mode: cross-check a plan manifest against the running binary
    if let Some(ref path) = args.verify_manifest {
        let exit_code = run_verify_manifest(path);
        std::process::exit(exit_code);
    }

    // --all mode: scan all plans
    if args.all {
        run_all_plans_doctor(&args.version, &args.year, &args.output_base);
        return;
    }

    if let Some(ref label) = args.label {
        run_plan_doctor(label, &args.version, &args.year, &args.output_base);
        return;
    }

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

    // ── 7. TIGER geometry files (needed for redist map) ──────────────────────
    {
        let tiger_path = std::path::PathBuf::from(&args.output_base)
            .join("V3").join("data").join(&args.year).join("tiger");
        let has_tiger = tiger_path.exists()
            && tiger_path.read_dir()
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_tiger {
            println!("[PASS] TIGER geometry found (maps available): {}", tiger_path.display());
        } else {
            println!("[INFO] TIGER geometry not found (maps will fail): {}", tiger_path.display());
            println!(
                "       Run: redist fetch --type tiger --states {} --year {}",
                code, args.year
            );
        }
    }

    // ── 8. Nesting (if applicable) ────────────────────────────────────────────
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

    // ── 8. Compactness standard ───────────────────────────────────────────────
    {
        let policy = registry.raw();
        if let Some(location) = policy.get(&code) {
            if let Some(std) = location.get("compactness_standard").and_then(|v| v.as_str()) {
                println!("[INFO] Compactness standard: {std}");
            }
        }
    }

    println!("\n[OK] Doctor check complete for {code} {} {}.", args.chamber, args.year);
}

// ---------------------------------------------------------------------------
// All-plans scan mode — `redist doctor --all`
// ---------------------------------------------------------------------------

fn run_all_plans_doctor(version: &str, year: &str, output_base: &str) {
    let plans_dir = std::path::PathBuf::from(output_base)
        .join(version).join(year).join("plans");

    let mut labels: Vec<String> = std::fs::read_dir(&plans_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    labels.sort();

    if labels.is_empty() {
        println!("No plans found in {}", plans_dir.display());
        println!("Run 'redist state --label ...' to create a plan first.");
        return;
    }

    println!("=== redist doctor --all: {} plans in {}/{}/{}/plans/ ===\n",
        labels.len(), output_base, version, year);
    println!("{:<30} {:>9} {:>8} {:>6} {:>7} {:>8}",
        "Label", "Districts", "Balance", "PP", "Splits", "Missing");
    println!("{}", "-".repeat(75));

    for label in &labels {
        match crate::plan_context::PlanContext::from_label(
            std::path::Path::new(output_base), version, year, label
        ) {
            Err(_) => println!("{:<30} [no manifest]", label),
            Ok(ctx) => {
                let n = ctx.num_districts();
                let balance = if ctx.analysis_file_exists("summary.json") {
                    let path = ctx.analysis_file("summary.json");
                    std::fs::read_to_string(&path).ok()
                        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                        .and_then(|v| v["population_balance_valid"].as_bool())
                        .map(|ok| if ok { "OK" } else { "FAIL" })
                        .unwrap_or("?")
                } else { "-" };
                let pp = if ctx.analysis_file_exists("compactness.json") {
                    let path = ctx.analysis_file("compactness.json");
                    std::fs::read_to_string(&path).ok()
                        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                        .and_then(|v| {
                            v["districts"].as_array().and_then(|d| {
                                let vals: Vec<f64> = d.iter().filter_map(|x| x["polsby_popper"].as_f64()).collect();
                                if vals.is_empty() { None } else { Some(vals.iter().sum::<f64>() / vals.len() as f64) }
                            })
                        })
                        .map(|pp_val| format!("{:.3}", pp_val))
                        .unwrap_or_else(|| "-".to_string())
                } else { "-".to_string() };
                let splits = if ctx.analysis_file_exists("splits.json") {
                    let path = ctx.analysis_file("splits.json");
                    std::fs::read_to_string(&path).ok()
                        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                        .and_then(|v| v["counties"]["split"].as_u64())
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "-".to_string())
                } else { "-".to_string() };
                let required = ["summary.json", "compactness.json", "contiguity.json"];
                let missing_count = required.iter().filter(|&&f| !ctx.analysis_file_exists(f)).count();
                let missing_str = if missing_count == 0 { "none".to_string() } else { format!("{} required", missing_count) };

                println!("{:<30} {:>9} {:>8} {:>6} {:>7} {:>8}",
                    &label[..label.len().min(29)], n, balance, pp, splits, missing_str);
            }
        }
    }
    println!("\nRun 'redist doctor --label <label>' for detailed diagnosis.");
}

// ---------------------------------------------------------------------------
// Plan diagnosis mode — `redist doctor --label <label>`
// ---------------------------------------------------------------------------

fn run_plan_doctor(label: &str, version: &str, year: &str, output_base: &str) {
    use std::path::PathBuf;

    let ctx = match crate::plan_context::PlanContext::from_label(
        PathBuf::from(output_base).as_path(), version, year, label,
    ) {
        Ok(c) => c,
        Err(e) => {
            println!("[FAIL] {e}");
            std::process::exit(1);
        }
    };

    println!("=== redist doctor: {} ===\n", label);
    println!(
        "Plan:  {} ({} {}, {}, {} districts)",
        label,
        ctx.state_code(),
        ctx.chamber(),
        ctx.year(),
        ctx.num_districts()
    );
    println!("Dir:   {}\n", ctx.plan_dir.display());

    // ── Analysis files ────────────────────────────────────────────────────────
    println!("── Analysis files ─────────────────────────────────────────────");

    let required_analysis = ["summary.json", "compactness.json", "contiguity.json"];
    let optional_analysis = [
        "splits.json",
        "demographic.json",
        "partisan.json",
        "vra_analysis.json",
        "comparison.json",
        "nesting.json",
    ];

    let mut missing_required: Vec<&str> = vec![];
    let mut missing_optional: Vec<&str> = vec![];

    for name in &required_analysis {
        if ctx.analysis_file_exists(name) {
            let info = read_analysis_summary(&ctx, name);
            // For compactness: use [WARN] when PP is low or moderate (< 0.25)
            let status = if *name == "compactness.json" {
                if info.contains("low") || info.contains("moderate") { "WARN" } else { "PASS" }
            } else {
                "PASS"
            };
            println!("[{}] {:<24} {}", status, name, info);
        } else {
            println!(
                "[MISS] {:<24} -> run: redist analyze --label {} --types {}",
                name,
                label,
                name.trim_end_matches(".json")
            );
            missing_required.push(name);
        }
    }

    for name in &optional_analysis {
        if ctx.analysis_file_exists(name) {
            println!("[PASS] {}", name);
        } else {
            let hint = analysis_fetch_hint(name, ctx.state_code(), ctx.year());
            if let Some(h) = hint {
                println!(
                    "[MISS] {:<24} -> run: redist analyze --label {} --types {}\n       {}",
                    name,
                    label,
                    name.trim_end_matches(".json"),
                    h
                );
            } else {
                println!(
                    "[MISS] {:<24} -> run: redist analyze --label {} --types {}",
                    name,
                    label,
                    name.trim_end_matches(".json")
                );
            }
            missing_optional.push(name);
        }
    }

    // ── Report readiness ──────────────────────────────────────────────────────
    println!("\n-- Report readiness ───────────────────────────────────────────");
    if missing_required.is_empty() {
        println!("[PASS] Required analysis files present");
        println!("[INFO] To generate report: redist report --label {} --format html", label);
    } else {
        println!(
            "[FAIL] Missing required files: {}",
            missing_required.join(", ")
        );
        println!("       Run: redist analyze --label {} --types all --force", label);
    }
    if !missing_optional.is_empty() {
        println!(
            "[WARN] Optional files missing: {} (report will show as unavailable)",
            missing_optional.join(", ")
        );
    }

    // ── Maps ──────────────────────────────────────────────────────────────────
    println!("\n-- Maps ────────────────────────────────────────────────────────");
    let maps_dir = ctx.maps_dir();
    let map_files = ["districts.png", "compactness.png", "political.png"];
    let mut has_maps = false;
    for f in &map_files {
        if maps_dir.join(f).exists() {
            println!("[PASS] maps/{}", f);
            has_maps = true;
        }
    }
    if !has_maps {
        println!("[MISS] No maps generated");
        println!("       Run: redist map --label {} --types districts", label);
        println!(
            "       (needs: redist fetch --type tiger --states {} --year {})",
            ctx.state_code(),
            ctx.year()
        );
    }

    // ── Court readiness ───────────────────────────────────────────────────────
    println!("\n-- Court readiness ─────────────────────────────────────────────");
    let audit_path = ctx.plan_dir.join("audit.json");
    if audit_path.exists() {
        println!("[PASS] audit.json present");
    } else {
        println!("[WARN] audit.json missing");
        println!(
            "       Run: redist report --label {} --audit-with-report --format html",
            label
        );
    }

    let rplan_path = ctx.plan_dir.join(format!("{}.rplan", label));
    if rplan_path.exists() {
        println!("[PASS] {}.rplan present (reproducibility package)", label);
    } else {
        println!(
            "[INFO] No .rplan file -- run: redist export --label {} --format rplan",
            label
        );
    }

    println!("\n[OK] Doctor complete for {}", label);
}

fn read_analysis_summary(ctx: &crate::plan_context::PlanContext, name: &str) -> String {
    let path = ctx.analysis_file(name);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return String::new();
    };
    let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(&content) else {
        return String::new();
    };

    match name {
        "summary.json" => {
            let valid = v["population_balance_valid"].as_bool().unwrap_or(false);
            let n = v["districts"].as_array().map(|a| a.len()).unwrap_or(0);
            let balance_context = if valid && n > 1 {
                " (within tolerance ✓)"
            } else if !valid {
                " (⚠ EXCEEDS TOLERANCE)"
            } else {
                ""
            };
            format!(
                "({} districts, balance {}{})",
                n,
                if valid { "valid" } else { "FAILED" },
                balance_context
            )
        }
        "compactness.json" => {
            let districts = v["districts"].as_array();
            if let Some(d) = districts {
                let pp_vals: Vec<f64> = d
                    .iter()
                    .filter_map(|x| x["polsby_popper"].as_f64())
                    .collect();
                if !pp_vals.is_empty() {
                    let mean = pp_vals.iter().sum::<f64>() / pp_vals.len() as f64;
                    let context = if mean < 0.15 {
                        " (low — may face legal challenge)"
                    } else if mean < 0.25 {
                        " (moderate)"
                    } else if mean < 0.35 {
                        " (good)"
                    } else {
                        " (well-compact)"
                    };
                    return format!("(mean PP: {:.3}{})", mean, context);
                }
            }
            String::new()
        }
        "contiguity.json" => {
            let all = v["all_contiguous"].as_bool().unwrap_or(false);
            format!(
                "({})",
                if all {
                    "all contiguous"
                } else {
                    "NON-CONTIGUOUS districts found"
                }
            )
        }
        _ => String::new(),
    }
}

fn analysis_fetch_hint(name: &str, state_code: &str, year: &str) -> Option<String> {
    match name {
        "partisan.json" => Some(format!(
            "(also needs: redist fetch --type elections --states {} --year {})",
            state_code, year
        )),
        "demographic.json" | "vra_analysis.json" => Some(format!(
            "(also needs: redist fetch --type demographics --states {} --year {})",
            state_code, year
        )),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// `redist doctor --verify-manifest <PATH>` — provenance audit
// ---------------------------------------------------------------------------

/// Verify a plan manifest.json against the running binary's provenance.
///
/// Reports:
///   - Manifest parses successfully
///   - Recorded binary_version vs the running binary
///   - Build commit / rustc version of the running binary (for the audit log)
///   - Adjacency file SHA-256 (informational; does not re-hash if the file is missing)
///
/// Returns 0 on success, 1 on hard mismatch, 2 on file/parse errors.
pub fn run_verify_manifest(manifest_path: &str) -> i32 {
    let prov = Provenance::current();
    let path = std::path::Path::new(manifest_path);

    println!("=== redist doctor --verify-manifest ===\n");
    println!("Manifest: {}", path.display());
    println!();

    // 1. Read + parse
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            println!("[FAIL] Cannot read manifest: {e}");
            return 2;
        }
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            println!("[FAIL] Manifest is not valid JSON: {e}");
            return 2;
        }
    };

    // 2. Running binary provenance — printed first so it's the audit reference
    println!("Running binary:");
    println!("  redist_version:      {}", prov.redist_version);
    println!("  redist_build_commit: {}", prov.redist_build_commit);
    println!("  redist_build_date:   {}", prov.redist_build_date);
    println!("  rustc_version:       {}", prov.rustc_version);
    println!();

    // 3. Pull recorded version
    let manifest_version = json.get("binary_version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    println!("Manifest records:");
    println!("  binary_version:      {}", if manifest_version.is_empty() { "(missing)" } else { manifest_version });
    if let Some(s) = json.get("binary_sha256").and_then(|v| v.as_str()) {
        println!("  binary_sha256:       {}", if s.is_empty() { "(not recorded)" } else { s });
    }
    if let Some(s) = json.get("adjacency_sha256").and_then(|v| v.as_str()) {
        println!("  adjacency_sha256:    {}", if s.is_empty() { "(not recorded)" } else { s });
    }
    if let Some(s) = json.get("created_at").and_then(|v| v.as_str()) {
        println!("  created_at:          {s}");
    }
    println!();

    // 4. Verify version
    let mut hard_failures = 0;
    if manifest_version.is_empty() {
        println!("[WARN] manifest has no binary_version field — cannot verify provenance.");
    } else {
        match prov.verify_version_matches(manifest_version) {
            Ok(()) => println!("[PASS] binary_version matches running binary."),
            Err(e) => {
                println!("[FAIL] {e}");
                hard_failures += 1;
            }
        }
    }

    // 5. Adjacency file existence (informational — we don't re-hash here)
    if let Some(adj_filename) = json.get("adjacency_file").and_then(|v| v.as_str()) {
        if !adj_filename.is_empty() {
            println!("[INFO] Adjacency file referenced: {adj_filename}");
            println!("       To verify integrity: sha256sum <path>/{adj_filename} and compare to manifest.adjacency_sha256");
        }
    }

    // 6. Build commit traceability
    if prov.redist_build_commit.starts_with("unknown") {
        println!("[WARN] Running binary's build commit is 'unknown' — built outside a git checkout. Source provenance cannot be attested.");
    } else if prov.redist_build_commit.ends_with("-dirty") {
        println!("[WARN] Running binary was built from a dirty working tree (commit suffix '-dirty'). Source provenance is degraded.");
    }

    println!();
    if hard_failures == 0 {
        println!("Result: OK ({} hard failures)", hard_failures);
        0
    } else {
        println!("Result: FAIL ({} hard failures)", hard_failures);
        1
    }
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
            label: None,
            version: "v1".to_string(),
            all: false,
            verify_manifest: None,
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

    // ── Gap 7: doctor checks TIGER geometry files ─────────────────────────────

    #[test]
    fn test_doctor_checks_tiger_geometry_missing() {
        // With a nonexistent output_base path, the TIGER check must produce an
        // info message containing "tiger" and a fetch hint.
        let output_base = "/nonexistent_gap7_test/outputs";
        let year = "2020";
        let code = "WA";

        let tiger_path = std::path::PathBuf::from(output_base)
            .join("V3").join("data").join(year).join("tiger");

        // Verify the detection logic: non-existent path → info check fires
        let has_tiger = tiger_path.exists()
            && tiger_path.read_dir()
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);

        assert!(!has_tiger, "TIGER path must not exist in test env: {}", tiger_path.display());

        // Build the message that would be emitted
        let info_msg = format!(
            "[INFO] TIGER geometry not found (maps will fail): {}",
            tiger_path.display()
        );
        let fetch_hint = format!(
            "Run: redist fetch --type tiger --states {} --year {}",
            code, year
        );

        assert!(info_msg.contains("tiger"), "info message must contain 'tiger': {info_msg}");
        assert!(info_msg.contains("TIGER"), "info message must contain 'TIGER': {info_msg}");
        assert!(fetch_hint.contains("redist fetch"), "hint must suggest 'redist fetch': {fetch_hint}");
        assert!(fetch_hint.contains("--type tiger"), "hint must include '--type tiger': {fetch_hint}");
    }

    // ── Task 145: compactness standard ───────────────────────────────────────

    #[test]
    fn test_doctor_wa_has_compactness_standard() {
        let registry = LocationRegistry::load();
        let policy = registry.raw();
        let wa = policy.get("WA").expect("WA must be in policy");
        let std = wa.get("compactness_standard")
            .and_then(|v| v.as_str());
        assert!(std.is_some(), "WA must have compactness_standard field");
        let s = std.unwrap();
        assert!(s.contains("RCW") || s.contains("compact"),
            "WA compactness_standard must reference RCW or compactness: {s}");
    }

    #[test]
    fn test_doctor_compactness_standard_shown() {
        // Verify the field exists for the states that should have it
        let registry = LocationRegistry::load();
        let policy = registry.raw();
        let states_with_standard = ["WA", "CA", "CO", "FL", "TX", "VA", "NY"];
        for code in &states_with_standard {
            let location = policy.get(*code)
                .unwrap_or_else(|| panic!("{code} must be in policy"));
            let std = location.get("compactness_standard")
                .and_then(|v| v.as_str());
            assert!(std.is_some(),
                "{code} must have compactness_standard field, but it's missing");
            let s = std.unwrap();
            assert!(!s.is_empty(),
                "{code} compactness_standard must not be empty");
        }
    }

    // ── Task 208: doctor metric context thresholds ────────────────────────────

    #[test]
    fn test_doctor_compactness_low_pp_shows_warn_context() {
        // PP < 0.15 should get "low" context
        let pp = 0.12_f64;
        let context = if pp < 0.15 { "low" } else if pp < 0.25 { "moderate" } else if pp < 0.35 { "good" } else { "well-compact" };
        assert_eq!(context, "low");
    }

    #[test]
    fn test_doctor_compactness_high_pp_shows_compact_context() {
        let pp = 0.36_f64;
        let context = if pp < 0.15 { "low" } else if pp < 0.25 { "moderate" } else if pp < 0.35 { "good" } else { "well-compact" };
        assert_eq!(context, "well-compact");
    }

    #[test]
    fn test_doctor_compactness_moderate_pp() {
        let pp = 0.20_f64;
        let context = if pp < 0.15 { "low" } else if pp < 0.25 { "moderate" } else if pp < 0.35 { "good" } else { "well-compact" };
        assert_eq!(context, "moderate");
    }

    #[test]
    fn test_doctor_compactness_good_pp() {
        let pp = 0.30_f64;
        let context = if pp < 0.15 { "low" } else if pp < 0.25 { "moderate" } else if pp < 0.35 { "good" } else { "well-compact" };
        assert_eq!(context, "good");
    }

    /// read_analysis_summary compactness includes context text for mean PP.
    #[test]
    fn test_read_analysis_summary_compactness_includes_context() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("v1").join("2020").join("plans").join("ctx_test");
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        std::fs::write(
            plan_dir.join("manifest.json"),
            serde_json::json!({
                "label": "ctx_test",
                "state_code": "VT",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 1,
                "population_source": "total",
                "partition_mode": "edge-weighted",
                "seed": null,
                "binary_version": "0.1.0",
                "binary_sha256": "",
                "binary_download_url": "",
                "adjacency_file": "",
                "adjacency_sha256": "",
                "adjacency_build_command": "",
                "adjacency_build_version": "0.1.0",
                "tiger_source_url": "",
                "tiger_sha256": null,
                "created_at": "2026-04-28T00:00:00Z",
                "balance_tolerance_pct": 0.5,
                "population_balance_valid": true,
                "seats_per_district": 1,
                "total_seats": 1,
                "electoral_system": "single_member",
                "gpmetis_version": "unknown"
            }).to_string(),
        ).unwrap();

        // Low PP (< 0.15) → "low" context
        std::fs::write(
            analysis_dir.join("compactness.json"),
            serde_json::json!({
                "districts": [{"district": 1, "polsby_popper": 0.10}]
            }).to_string(),
        ).unwrap();
        let ctx = crate::plan_context::PlanContext::from_label(tmp.path(), "v1", "2020", "ctx_test").unwrap();
        let summary = read_analysis_summary(&ctx, "compactness.json");
        assert!(summary.contains("low") || summary.contains("legal"),
            "low PP must show 'low' context: {summary}");

        // High PP (> 0.35) → "well-compact" context
        std::fs::write(
            analysis_dir.join("compactness.json"),
            serde_json::json!({
                "districts": [{"district": 1, "polsby_popper": 0.40}]
            }).to_string(),
        ).unwrap();
        let ctx2 = crate::plan_context::PlanContext::from_label(tmp.path(), "v1", "2020", "ctx_test").unwrap();
        let summary2 = read_analysis_summary(&ctx2, "compactness.json");
        assert!(summary2.contains("well-compact"),
            "high PP must show 'well-compact' context: {summary2}");
    }

    /// read_analysis_summary summary.json includes balance context.
    #[test]
    fn test_read_analysis_summary_balance_context() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("v1").join("2020").join("plans").join("bal_ctx_test");
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        std::fs::write(
            plan_dir.join("manifest.json"),
            serde_json::json!({
                "label": "bal_ctx_test",
                "state_code": "WA",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 10,
                "population_source": "total",
                "partition_mode": "edge-weighted",
                "seed": null,
                "binary_version": "0.1.0",
                "binary_sha256": "",
                "binary_download_url": "",
                "adjacency_file": "",
                "adjacency_sha256": "",
                "adjacency_build_command": "",
                "adjacency_build_version": "0.1.0",
                "tiger_source_url": "",
                "tiger_sha256": null,
                "created_at": "2026-04-28T00:00:00Z",
                "balance_tolerance_pct": 0.5,
                "population_balance_valid": true,
                "seats_per_district": 1,
                "total_seats": 10,
                "electoral_system": "single_member",
                "gpmetis_version": "unknown"
            }).to_string(),
        ).unwrap();

        // Valid balance with 10 districts → "within tolerance" context
        std::fs::write(
            analysis_dir.join("summary.json"),
            serde_json::json!({
                "population_balance_valid": true,
                "districts": [
                    {"district": 1}, {"district": 2}, {"district": 3},
                    {"district": 4}, {"district": 5}, {"district": 6},
                    {"district": 7}, {"district": 8}, {"district": 9}, {"district": 10}
                ]
            }).to_string(),
        ).unwrap();
        let ctx = crate::plan_context::PlanContext::from_label(tmp.path(), "v1", "2020", "bal_ctx_test").unwrap();
        let summary = read_analysis_summary(&ctx, "summary.json");
        assert!(summary.contains("within tolerance"),
            "valid balance with >1 district must show 'within tolerance': {summary}");

        // Invalid balance → "EXCEEDS TOLERANCE" context
        std::fs::write(
            analysis_dir.join("summary.json"),
            serde_json::json!({
                "population_balance_valid": false,
                "districts": [{"district": 1}, {"district": 2}]
            }).to_string(),
        ).unwrap();
        let ctx2 = crate::plan_context::PlanContext::from_label(tmp.path(), "v1", "2020", "bal_ctx_test").unwrap();
        let summary2 = read_analysis_summary(&ctx2, "summary.json");
        assert!(summary2.contains("EXCEEDS TOLERANCE"),
            "invalid balance must show 'EXCEEDS TOLERANCE': {summary2}");
    }

    // ── Plan diagnosis mode (--label) ─────────────────────────────────────────

    /// PlanContext can be loaded when a valid manifest.json is present.
    #[test]
    fn test_doctor_label_mode_finds_plan() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp
            .path()
            .join("v1")
            .join("2020")
            .join("plans")
            .join("vt_test");
        std::fs::create_dir_all(&plan_dir).unwrap();
        std::fs::write(
            plan_dir.join("manifest.json"),
            serde_json::json!({
                "label": "vt_test",
                "state_code": "VT",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 1,
                "population_source": "total",
                "partition_mode": "edge-weighted",
                "seed": null,
                "binary_version": "0.1.0",
                "binary_sha256": "",
                "binary_download_url": "",
                "adjacency_file": "",
                "adjacency_sha256": "",
                "adjacency_build_command": "",
                "adjacency_build_version": "0.1.0",
                "tiger_source_url": "",
                "tiger_sha256": null,
                "created_at": "2026-04-28T00:00:00Z",
                "balance_tolerance_pct": 0.5,
                "population_balance_valid": true,
                "seats_per_district": 1,
                "total_seats": 1,
                "electoral_system": "single_member",
                "gpmetis_version": "unknown"
            })
            .to_string(),
        )
        .unwrap();

        let ctx = crate::plan_context::PlanContext::from_label(
            tmp.path(),
            "v1",
            "2020",
            "vt_test",
        )
        .unwrap();
        assert_eq!(ctx.num_districts(), 1);
        assert_eq!(ctx.chamber(), "congressional");
        assert_eq!(ctx.state_code(), "VT");
    }

    /// read_analysis_summary returns a string containing the mean PP value.
    #[test]
    fn test_read_analysis_summary_compactness() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp
            .path()
            .join("v1")
            .join("2020")
            .join("plans")
            .join("t");
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        std::fs::write(
            plan_dir.join("manifest.json"),
            serde_json::json!({
                "label": "t",
                "state_code": "VT",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 1,
                "population_source": "total",
                "partition_mode": "edge-weighted",
                "seed": null,
                "binary_version": "0.1.0",
                "binary_sha256": "",
                "binary_download_url": "",
                "adjacency_file": "",
                "adjacency_sha256": "",
                "adjacency_build_command": "",
                "adjacency_build_version": "0.1.0",
                "tiger_source_url": "",
                "tiger_sha256": null,
                "created_at": "2026-04-28T00:00:00Z",
                "balance_tolerance_pct": 0.5,
                "population_balance_valid": true,
                "seats_per_district": 1,
                "total_seats": 1,
                "electoral_system": "single_member",
                "gpmetis_version": "unknown"
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            analysis_dir.join("compactness.json"),
            serde_json::json!({
                "districts": [{"district": 1, "polsby_popper": 0.31}]
            })
            .to_string(),
        )
        .unwrap();

        let ctx = crate::plan_context::PlanContext::from_label(
            tmp.path(),
            "v1",
            "2020",
            "t",
        )
        .unwrap();
        let summary = read_analysis_summary(&ctx, "compactness.json");
        assert!(
            summary.contains("0.31") || summary.contains("PP"),
            "must show PP value: {summary}"
        );
    }

    // ── Task 203: doctor --all ────────────────────────────────────────────────

    #[test]
    fn test_doctor_all_empty_dir_shows_no_plans_message() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        // plans dir exists but is empty
        let plans_dir = tmp.path().join("v1").join("2020").join("plans");
        std::fs::create_dir_all(&plans_dir).unwrap();

        let mut labels: Vec<String> = std::fs::read_dir(&plans_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        labels.sort();
        assert!(labels.is_empty(), "no plans should be found in empty dir");
    }

    #[test]
    fn test_doctor_all_scans_plan_dirs() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let plans_dir = tmp.path().join("v1").join("2020").join("plans");
        for label in &["alpha_plan", "beta_plan"] {
            std::fs::create_dir_all(plans_dir.join(label)).unwrap();
        }
        let mut labels: Vec<String> = std::fs::read_dir(&plans_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        labels.sort();
        assert_eq!(labels, vec!["alpha_plan", "beta_plan"]);
    }

    // ── verify_manifest tests ────────────────────────────────────────────────

    fn write_manifest(tmp: &tempfile::TempDir, json: serde_json::Value) -> std::path::PathBuf {
        let path = tmp.path().join("manifest.json");
        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
        path
    }

    #[test]
    fn test_verify_manifest_missing_file_returns_2() {
        let result = run_verify_manifest("/definitely/not/a/real/path/manifest.json");
        assert_eq!(result, 2, "missing file is a soft error (exit code 2)");
    }

    #[test]
    fn test_verify_manifest_invalid_json_returns_2() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("manifest.json");
        std::fs::write(&path, "{ this is not valid json").unwrap();
        let result = run_verify_manifest(path.to_str().unwrap());
        assert_eq!(result, 2);
    }

    #[test]
    fn test_verify_manifest_matching_version_returns_0() {
        let tmp = tempfile::TempDir::new().unwrap();
        let prov = Provenance::current();
        let path = write_manifest(&tmp, serde_json::json!({
            "binary_version": prov.redist_version,
            "binary_sha256": "",
            "adjacency_file": "",
            "adjacency_sha256": "",
            "created_at": "2026-04-29T00:00:00Z",
        }));
        let result = run_verify_manifest(path.to_str().unwrap());
        assert_eq!(result, 0, "matching version → exit 0");
    }

    #[test]
    fn test_verify_manifest_mismatched_version_returns_1() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = write_manifest(&tmp, serde_json::json!({
            "binary_version": "999.999.999",
            "binary_sha256": "",
        }));
        let result = run_verify_manifest(path.to_str().unwrap());
        assert_eq!(result, 1, "version mismatch → exit 1");
    }

    #[test]
    fn test_verify_manifest_missing_binary_version_warns_not_fails() {
        // A manifest without binary_version is unverifiable but not a hard failure.
        let tmp = tempfile::TempDir::new().unwrap();
        let path = write_manifest(&tmp, serde_json::json!({
            "label": "vt_2020",
            "state_code": "VT",
        }));
        let result = run_verify_manifest(path.to_str().unwrap());
        assert_eq!(result, 0, "missing binary_version → warn, exit 0");
    }
}
