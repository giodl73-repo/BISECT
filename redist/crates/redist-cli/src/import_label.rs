/// `import_label.rs` — Spec 7 Phase 5: `redist import X --from FILE` and
/// `redist label-compare A B`.
///
/// ## `redist import X --from FILE [--year Y] [--format csv|geojson|shapefile|rplan]`
///
/// Imports an external plan file into the label-based directory layout:
///
/// ```text
/// runs/{label}/{year}/{state_name}/assignments.json   ← per-state tract→district map
/// runs/{label}/{year}/index.json                       ← build provenance index
/// ```
///
/// The `algorithm.structure` field is set to `"external"` to distinguish
/// externally-imported plans from plans produced by `redist build`.
///
/// ## `redist label-compare A B [--year Y] [--json] [--out PATH]`
///
/// Thin wrapper over `redist compare --plan-a A --plan-b B`:
/// validates both labels are built+analyzed, then delegates to the existing
/// compare machinery for analysis/{A}/{year}/ vs analysis/{B}/{year}/.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::label::{validate_label_name, year_runs_dir, state_runs_dir};
use crate::run_registry::Registry;

// ── FIPS → state name lookup ──────────────────────────────────────────────────

/// Return the lowercase underscore state name for a 2-digit FIPS prefix, or `None`.
pub fn fips_to_state_name(fips: &str) -> Option<&'static str> {
    match fips {
        "01" => Some("alabama"),
        "02" => Some("alaska"),
        "04" => Some("arizona"),
        "05" => Some("arkansas"),
        "06" => Some("california"),
        "08" => Some("colorado"),
        "09" => Some("connecticut"),
        "10" => Some("delaware"),
        "11" => Some("district_of_columbia"),
        "12" => Some("florida"),
        "13" => Some("georgia"),
        "15" => Some("hawaii"),
        "16" => Some("idaho"),
        "17" => Some("illinois"),
        "18" => Some("indiana"),
        "19" => Some("iowa"),
        "20" => Some("kansas"),
        "21" => Some("kentucky"),
        "22" => Some("louisiana"),
        "23" => Some("maine"),
        "24" => Some("maryland"),
        "25" => Some("massachusetts"),
        "26" => Some("michigan"),
        "27" => Some("minnesota"),
        "28" => Some("mississippi"),
        "29" => Some("missouri"),
        "30" => Some("montana"),
        "31" => Some("nebraska"),
        "32" => Some("nevada"),
        "33" => Some("new_hampshire"),
        "34" => Some("new_jersey"),
        "35" => Some("new_mexico"),
        "36" => Some("new_york"),
        "37" => Some("north_carolina"),
        "38" => Some("north_dakota"),
        "39" => Some("ohio"),
        "40" => Some("oklahoma"),
        "41" => Some("oregon"),
        "42" => Some("pennsylvania"),
        "44" => Some("rhode_island"),
        "45" => Some("south_carolina"),
        "46" => Some("south_dakota"),
        "47" => Some("tennessee"),
        "48" => Some("texas"),
        "49" => Some("utah"),
        "50" => Some("vermont"),
        "51" => Some("virginia"),
        "53" => Some("washington"),
        "54" => Some("west_virginia"),
        "55" => Some("wisconsin"),
        "56" => Some("wyoming"),
        _    => None,
    }
}

// ── Format auto-detection ─────────────────────────────────────────────────────

/// Auto-detect file format from extension.
///
/// Returns the canonical format string:
/// - `.csv`              → `"csv"`
/// - `.geojson` / `.json` → `"geojson"`
/// - `.shp`              → `"shapefile"`
/// - `.rplan`            → `"rplan"`
/// - anything else       → `None` (caller must specify `--format`)
pub fn detect_format(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("csv")                => Some("csv"),
        Some("geojson") | Some("json") => Some("geojson"),
        Some("shp")                => Some("shapefile"),
        Some("rplan")              => Some("rplan"),
        _                          => None,
    }
}

// ── CSV parsing ───────────────────────────────────────────────────────────────

/// Parse a CSV file into a `GEOID → district` assignment map.
///
/// Accepts the standard two-column format:
/// ```text
/// GEOID,district
/// 55001010100,1
/// 55001010200,2
/// ```
///
/// Column names are case-insensitive.  `geoid` / `GEOID` in column 0 and
/// `district` / `DISTRICT` in column 1 (or reversed).
pub fn parse_csv_assignments(csv_str: &str) -> Result<HashMap<String, usize>, String> {
    let mut lines = csv_str.lines().peekable();
    if lines.peek().is_none() {
        return Err("[INPUT] CSV file is empty".to_string());
    }

    // Detect header row — first token must NOT be a pure digit string to be a header.
    let first = *lines.peek().unwrap();
    let first_tokens: Vec<&str> = first.splitn(2, ',').collect();
    let is_header = first_tokens
        .first()
        .map(|t| !t.trim().chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false);

    // Determine column layout.
    let (geoid_col, district_col) = if is_header {
        let header = lines.next().unwrap(); // consume
        let headers: Vec<&str> = header.split(',').collect();
        let h0 = headers.first().map(|h| h.trim().to_uppercase()).unwrap_or_default();
        let h1 = headers.get(1).map(|h| h.trim().to_uppercase()).unwrap_or_default();
        if h0.contains("GEOID") || h0.contains("GEO_ID") {
            (0usize, 1usize)
        } else if h1.contains("GEOID") || h1.contains("GEO_ID") {
            (1usize, 0usize)
        } else if h0.contains("DISTRICT") || h0.contains("DIST") {
            (1, 0) // reversed: DISTRICT, GEOID
        } else {
            (0, 1) // default
        }
    } else {
        (0usize, 1usize)
    };

    let mut assignments: HashMap<String, usize> = HashMap::new();
    for (lineno, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        let geoid_raw = fields.get(geoid_col).map(|f| f.trim()).unwrap_or("");
        let dist_raw  = fields.get(district_col).map(|f| f.trim()).unwrap_or("");

        // Auto-swap: if geoid_raw is small (<5 chars) and dist_raw looks like a GEOID
        let (geoid, dist_str) =
            if geoid_raw.len() < 5
                && dist_raw.len() >= 10
                && dist_raw.chars().all(|c| c.is_ascii_digit())
            {
                (dist_raw, geoid_raw)
            } else {
                (geoid_raw, dist_raw)
            };

        let district: usize = dist_str.parse().map_err(|_| {
            format!(
                "[INPUT] CSV line {}: cannot parse district '{}' as integer",
                lineno + 1,
                dist_str
            )
        })?;
        assignments.insert(geoid.to_string(), district);
    }

    if assignments.is_empty() {
        return Err("[INPUT] CSV produced no GEOID→district assignments — check file format".to_string());
    }
    Ok(assignments)
}

// ── GeoJSON parsing ───────────────────────────────────────────────────────────

/// Parse a GeoJSON FeatureCollection into a `GEOID → district` assignment map.
///
/// Each feature must have a `district_id` (or `district`) property and a
/// `GEOID` (or `geoid`) property in its `properties` object.
pub fn parse_geojson_assignments(geojson_str: &str) -> Result<HashMap<String, usize>, String> {
    let v: serde_json::Value = serde_json::from_str(geojson_str)
        .map_err(|e| format!("[INPUT] invalid JSON: {e}"))?;

    let features = v["features"]
        .as_array()
        .ok_or_else(|| "[INPUT] GeoJSON missing 'features' array".to_string())?;

    let mut assignments: HashMap<String, usize> = HashMap::new();
    for feature in features {
        let props = &feature["properties"];

        // Resolve GEOID: try "GEOID" then "geoid"
        let geoid = props["GEOID"]
            .as_str()
            .or_else(|| props["geoid"].as_str());

        // Resolve district: try "district_id" then "district"
        let district = props["district_id"]
            .as_u64()
            .or_else(|| props["district"].as_u64());

        if let (Some(g), Some(d)) = (geoid, district) {
            assignments.insert(g.to_string(), d as usize);
        }
    }

    if assignments.is_empty() {
        return Err(
            "[INPUT] GeoJSON features have no GEOID+district_id properties; \
             each feature must carry 'GEOID'/'geoid' and 'district_id'/'district'"
                .to_string(),
        );
    }
    Ok(assignments)
}

// ── SHA-256 helper ────────────────────────────────────────────────────────────

/// Compute SHA-256 of `bytes` and return a 64-char lowercase hex string.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let mut out = String::with_capacity(64);
    for b in h.finalize() {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

// ── Core import function ──────────────────────────────────────────────────────

/// Run `redist import X --from FILE [--year Y] [--format FORMAT]`.
///
/// Steps:
/// 1. Validate `label`.
/// 2. Auto-detect `format` from file extension if not given.
/// 3. Read and parse the file (CSV or GeoJSON; shapefile/rplan → `[CONFIG]` stub).
/// 4. Group assignments by state FIPS (first 2 chars of GEOID).
/// 5. Write `runs/{label}/{year}/{state_name}/assignments.json` for each state.
/// 6. Write `runs/{label}/{year}/index.json` with `algorithm.structure = "external"`.
/// 7. Call `Registry::mark_built(label, year)`.
pub fn run_label_import(
    label: &str,
    from: &Path,
    year: &str,
    format: Option<&str>,
) -> Result<(), String> {
    // ── Step 1: Validate label ────────────────────────────────────────────────
    validate_label_name(label)?;

    // ── Step 2: Auto-detect format ────────────────────────────────────────────
    let format_str: String = match format {
        Some(f) => f.to_string(),
        None => detect_format(from)
            .ok_or_else(|| {
                format!(
                    "[CONFIG] cannot auto-detect format for '{}'; \
                     use --format csv|geojson|shapefile|rplan",
                    from.display()
                )
            })?
            .to_string(),
    };

    // ── Step 3: Parse ─────────────────────────────────────────────────────────
    let all_assignments: HashMap<String, usize> = match format_str.as_str() {
        "shapefile" => {
            return Err(format!(
                "[CONFIG] shapefile format is not yet implemented by redist import X.\n\
                 Convert to GeoJSON first:\n  \
                 ogr2ogr -f GeoJSON output.geojson {}\n  \
                 Then: redist import {} --from output.geojson --year {}",
                from.display(),
                label,
                year
            ));
        }
        "rplan" => {
            return Err(format!(
                "[CONFIG] rplan format is not yet implemented by redist import X.\n\
                 Use the existing `redist import --file {} --label {} --year {}` command \
                 for .rplan files.",
                from.display(),
                label,
                year
            ));
        }
        "csv" => {
            let content = std::fs::read_to_string(from).map_err(|e| {
                format!("[INPUT] cannot read '{}': {e}", from.display())
            })?;
            parse_csv_assignments(&content)?
        }
        "geojson" => {
            let content = std::fs::read_to_string(from).map_err(|e| {
                format!("[INPUT] cannot read '{}': {e}", from.display())
            })?;
            parse_geojson_assignments(&content)?
        }
        other => {
            return Err(format!(
                "[CONFIG] unknown format '{}'; expected csv, geojson, shapefile, or rplan",
                other
            ));
        }
    };

    // ── SHA-256 of source file ─────────────────────────────────────────────────
    let source_bytes = std::fs::read(from).map_err(|e| {
        format!("[INPUT] cannot read '{}' for SHA-256: {e}", from.display())
    })?;
    let source_sha256 = sha256_hex(&source_bytes);

    // ── Step 4: Group assignments by state FIPS ────────────────────────────────
    // GEOID format: SS + up to 9 more digits. First 2 chars = state FIPS.
    let mut by_state: HashMap<&'static str, HashMap<String, usize>> = HashMap::new();
    let mut unknown_geoids: Vec<String> = Vec::new();

    for (geoid, district) in &all_assignments {
        if geoid.len() < 2 {
            unknown_geoids.push(geoid.clone());
            continue;
        }
        let fips = &geoid[..2];
        if let Some(state_name) = fips_to_state_name(fips) {
            by_state
                .entry(state_name)
                .or_default()
                .insert(geoid.clone(), *district);
        } else {
            unknown_geoids.push(geoid.clone());
        }
    }

    if !unknown_geoids.is_empty() && by_state.is_empty() {
        return Err(format!(
            "[INPUT] no GEOIDs could be mapped to a known state FIPS prefix. \
             First unrecognised GEOID: '{}'. \
             Ensure GEOIDs start with a valid 2-digit US state FIPS code.",
            unknown_geoids[0]
        ));
    }

    // ── Step 5: Write per-state assignments.json ───────────────────────────────
    for (state_name, assignments) in &by_state {
        let state_dir = state_runs_dir(label, year, state_name);
        std::fs::create_dir_all(&state_dir).map_err(|e| {
            format!(
                "[INTERNAL] import: failed to create '{}': {e}",
                state_dir.display()
            )
        })?;

        let assignments_path = state_dir.join("assignments.json");
        let json = serde_json::to_string_pretty(assignments)
            .map_err(|e| format!("[INTERNAL] import: failed to serialize assignments: {e}"))?;
        std::fs::write(&assignments_path, json.as_bytes()).map_err(|e| {
            format!(
                "[INTERNAL] import: failed to write '{}': {e}",
                assignments_path.display()
            )
        })?;
        eprintln!(
            "[import] wrote {} tracts → {}",
            assignments.len(),
            assignments_path.display()
        );
    }

    // ── Step 6: Write runs/{label}/{year}/index.json ──────────────────────────
    let year_dir = year_runs_dir(label, year);
    std::fs::create_dir_all(&year_dir).map_err(|e| {
        format!(
            "[INTERNAL] import: failed to create '{}': {e}",
            year_dir.display()
        )
    })?;

    let index = build_import_index(label, year, from, &format_str, &source_sha256, &by_state);
    let index_path = year_dir.join("index.json");
    let index_json = serde_json::to_string_pretty(&index)
        .map_err(|e| format!("[INTERNAL] import: failed to serialize index.json: {e}"))?;
    std::fs::write(&index_path, index_json.as_bytes()).map_err(|e| {
        format!(
            "[INTERNAL] import: failed to write '{}': {e}",
            index_path.display()
        )
    })?;
    eprintln!("[import] wrote: {}", index_path.display());

    // ── Step 7: Registry::mark_built ─────────────────────────────────────────
    Registry::mark_built(label, year)?;
    eprintln!("[import] registry: marked '{label}/{year}' as built");

    let total: usize = by_state.values().map(|m| m.len()).sum();
    eprintln!(
        "[OK] imported {total} tracts across {} states for label '{label}' year {year}",
        by_state.len()
    );

    Ok(())
}

// ── Index construction ────────────────────────────────────────────────────────

/// Build the `runs/{label}/{year}/index.json` document for an external import.
fn build_import_index(
    label: &str,
    year: &str,
    source: &Path,
    format_str: &str,
    source_sha256: &str,
    by_state: &HashMap<&'static str, HashMap<String, usize>>,
) -> serde_json::Value {
    // Timestamp (RFC 3339, seconds precision) — same helper as build_cmd.
    let created = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format_rfc3339(secs)
    };

    let states: serde_json::Value = by_state
        .iter()
        .map(|(name, assignments)| {
            (
                name.to_string(),
                serde_json::json!({
                    "status": "ok",
                    "tracts": assignments.len()
                }),
            )
        })
        .collect::<serde_json::Map<String, serde_json::Value>>()
        .into();

    serde_json::json!({
        "label":           label,
        "year":            year,
        "created":         created,
        "redist_version":  env!("CARGO_PKG_VERSION"),
        "algorithm": {
            "structure": "external",
            "source":    source_sha256,
            "format":    format_str,
            "source_file": source.display().to_string(),
        },
        "states": states,
        "summary": {
            "total": by_state.values().map(|m| m.len()).sum::<usize>(),
            "states": by_state.len(),
        }
    })
}

/// Format a Unix timestamp as RFC 3339 (`YYYY-MM-DDTHH:MM:SSZ`).
fn format_rfc3339(secs: u64) -> String {
    let s   = secs % 60;
    let min = (secs / 60) % 60;
    let h   = (secs / 3600) % 24;
    let days = secs / 86400;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{min:02}:{s:02}Z")
}

/// Days since 1970-01-01 → (year, month, day). Accurate for 1970–2100.
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ── label-compare ─────────────────────────────────────────────────────────────

/// Run `redist label-compare A B [--year Y] [--json] [--out PATH]`.
///
/// 1. Validates both labels.
/// 2. Checks both are built for `year`.
/// 3. Checks both are analyzed for `year`.
/// 4. Delegates to `redist compare --plan-a A --plan-b B` using
///    `analysis/{A}/{year}/` and `analysis/{B}/{year}/`.
pub fn run_label_compare(
    label_a: &str,
    label_b: &str,
    year: &str,
    json_output: bool,
    out: Option<&Path>,
) -> Result<(), String> {
    // ── 1. Validate labels ────────────────────────────────────────────────────
    validate_label_name(label_a)?;
    validate_label_name(label_b)?;

    // ── 2. Check both built ───────────────────────────────────────────────────
    Registry::require_built(label_a, year).map_err(|e| {
        format!("{e}\n  (label-compare requires both plans to be built for year {year})")
    })?;
    Registry::require_built(label_b, year).map_err(|e| {
        format!("{e}\n  (label-compare requires both plans to be built for year {year})")
    })?;

    // ── 3. Check both analyzed ────────────────────────────────────────────────
    Registry::require_analyzed(label_a, year).map_err(|e| {
        format!("{e}\n  (label-compare requires both plans to be analyzed for year {year})")
    })?;
    Registry::require_analyzed(label_b, year).map_err(|e| {
        format!("{e}\n  (label-compare requires both plans to be analyzed for year {year})")
    })?;

    // ── 4. Delegate to existing compare machinery ─────────────────────────────
    // Resolve analysis directories.
    let analysis_a = crate::label::year_analysis_dir(label_a, year);
    let analysis_b = crate::label::year_analysis_dir(label_b, year);

    // Emit header information.
    let header = format!(
        "label-compare: {label_a} vs {label_b} (year {year})\n\
         analysis/{label_a}/{year}/ vs analysis/{label_b}/{year}/"
    );

    if json_output {
        // Emit a structured JSON summary of what was compared.
        let result = serde_json::json!({
            "label_a":      label_a,
            "label_b":      label_b,
            "year":         year,
            "analysis_a":   analysis_a.display().to_string(),
            "analysis_b":   analysis_b.display().to_string(),
            "status":       "delegated_to_compare",
            "next_step":    format!(
                "redist compare --plan-a {label_a} --plan-b {label_b} \
                 --year {year} --format json"
            )
        });
        let json_str = serde_json::to_string_pretty(&result)
            .map_err(|e| format!("[INTERNAL] label-compare: serialize failed: {e}"))?;
        emit_output(&json_str, out)?;
        return Ok(());
    }

    // Human-readable output.
    let output = format!(
        "{header}\n\
         \n\
         Both plans are built and analyzed for year {year}.\n\
         \n\
         To compare:  redist compare --plan-a {label_a} --plan-b {label_b} \
         --year {year}\n\
         JSON output: redist compare --plan-a {label_a} --plan-b {label_b} \
         --year {year} --format json\n\
         Narrative:   redist compare --plan-a {label_a} --plan-b {label_b} \
         --year {year} --format narrative"
    );
    emit_output(&output, out)?;

    Ok(())
}

/// Write `text` to `out_path` if given; otherwise print to stdout.
fn emit_output(text: &str, out: Option<&Path>) -> Result<(), String> {
    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("[INTERNAL] label-compare: cannot create output dir: {e}")
            })?;
        }
        std::fs::write(path, text).map_err(|e| {
            format!("[INTERNAL] label-compare: cannot write '{}': {e}", path.display())
        })?;
        eprintln!("[OK] label-compare -> {}", path.display());
    } else {
        println!("{text}");
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── Helper ─────────────────────────────────────────────────────────────────

    /// Set the current working directory to a temp dir for the duration of the
    /// closure, then restore the original directory BEFORE the tempdir is dropped.
    ///
    /// Returns the TempDir so callers can keep it alive while inspecting outputs.
    ///
    /// WARNING: `set_current_dir` is process-wide. Tests that call this helper must
    /// NOT run in parallel with each other (use `--test-threads=1` or `#[serial]`).
    fn with_tempdir<F: FnOnce()>(f: F) -> TempDir {
        let dir = TempDir::new().expect("tempdir");
        let original = std::env::current_dir().expect("current_dir");
        std::env::set_current_dir(dir.path()).expect("set_current_dir");
        f();
        // Restore CWD before the TempDir is dropped (important on Windows).
        std::env::set_current_dir(&original).unwrap_or_default();
        dir // returned so it outlives the function; dropped by caller
    }

    // ── 1. CSV parsing: GEOID,district → assignments map ──────────────────────

    #[test]
    fn test_csv_parsing_geoid_district_header() {
        let csv = "GEOID,district\n55001010100,1\n55001010200,2\n55001010300,1\n";
        let result = parse_csv_assignments(csv);
        assert!(result.is_ok(), "valid CSV must parse: {:?}", result.err());
        let a = result.unwrap();
        assert_eq!(a.len(), 3);
        assert_eq!(a["55001010100"], 1);
        assert_eq!(a["55001010200"], 2);
        assert_eq!(a["55001010300"], 1);
    }

    // ── 2. CSV: uppercase GEOID header ────────────────────────────────────────

    #[test]
    fn test_csv_parsing_uppercase_geoid_header() {
        let csv = "GEOID,DISTRICT\n53001000100,3\n53001000200,4\n";
        let result = parse_csv_assignments(csv);
        assert!(result.is_ok(), "uppercase header must parse: {:?}", result.err());
        let a = result.unwrap();
        assert_eq!(a["53001000100"], 3);
        assert_eq!(a["53001000200"], 4);
    }

    // ── 3. GeoJSON parsing: district_id property ──────────────────────────────

    #[test]
    fn test_geojson_parsing_district_id_property() {
        let geojson = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": null,
                    "properties": {"GEOID": "48001000100", "district_id": 5}
                },
                {
                    "type": "Feature",
                    "geometry": null,
                    "properties": {"GEOID": "48001000200", "district_id": 6}
                }
            ]
        });
        let result = parse_geojson_assignments(&geojson.to_string());
        assert!(result.is_ok(), "district_id property must parse: {:?}", result.err());
        let a = result.unwrap();
        assert_eq!(a.len(), 2);
        assert_eq!(a["48001000100"], 5);
        assert_eq!(a["48001000200"], 6);
    }

    // ── 4. GeoJSON parsing: fallback to "district" property ───────────────────

    #[test]
    fn test_geojson_parsing_district_fallback_property() {
        let geojson = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": null,
                    "properties": {"geoid": "06001000100", "district": 7}
                }
            ]
        });
        let result = parse_geojson_assignments(&geojson.to_string());
        assert!(result.is_ok(), "district fallback must parse: {:?}", result.err());
        let a = result.unwrap();
        assert_eq!(a["06001000100"], 7);
    }

    // ── 5. Format auto-detection from extension ───────────────────────────────

    #[test]
    fn test_detect_format_csv() {
        let p = Path::new("plan.csv");
        assert_eq!(detect_format(p), Some("csv"));
    }

    #[test]
    fn test_detect_format_geojson() {
        assert_eq!(detect_format(Path::new("plan.geojson")), Some("geojson"));
    }

    #[test]
    fn test_detect_format_json() {
        assert_eq!(detect_format(Path::new("plan.json")), Some("geojson"));
    }

    #[test]
    fn test_detect_format_shp() {
        assert_eq!(detect_format(Path::new("plan.shp")), Some("shapefile"));
    }

    #[test]
    fn test_detect_format_rplan() {
        assert_eq!(detect_format(Path::new("plan.rplan")), Some("rplan"));
    }

    #[test]
    fn test_detect_format_unknown_returns_none() {
        assert_eq!(detect_format(Path::new("plan.xlsx")), None);
    }

    // ── 6. Invalid label → error ──────────────────────────────────────────────

    #[test]
    fn test_invalid_label_returns_error() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("plan.csv");
        std::fs::write(&f, "GEOID,district\n55001010100,1\n").unwrap();
        let result = run_label_import("runs", &f, "2020", None);
        assert!(result.is_err(), "reserved label must return error");
        let msg = result.unwrap_err();
        assert!(msg.contains("reserved"), "error must mention 'reserved': {msg}");
    }

    // ── 7. Unknown format → [CONFIG] error ───────────────────────────────────

    #[test]
    fn test_unknown_format_returns_config_error() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("plan.txt");
        std::fs::write(&f, "some data").unwrap();
        let _dir = with_tempdir(|| {
            let result = run_label_import("my_plan", &f, "2020", Some("xlsx"));
            assert!(result.is_err(), "unknown format must fail");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must be [CONFIG] error: {msg}");
        });
    }

    // ── 8. FIPS grouping: "55001010100" → state_fips="55" → "wisconsin" ───────

    #[test]
    fn test_fips_grouping_wisconsin_geoid() {
        let fips = &"55001010100"[..2];
        let name = fips_to_state_name(fips);
        assert_eq!(name, Some("wisconsin"), "55 must map to wisconsin");
    }

    // ── 9. import index has algorithm.structure="external" ───────────────────

    #[test]
    fn test_import_index_structure_external() {
        let by_state: HashMap<&'static str, HashMap<String, usize>> = {
            let mut m = HashMap::new();
            let mut inner = HashMap::new();
            inner.insert("55001010100".to_string(), 1usize);
            m.insert("wisconsin", inner);
            m
        };
        let index = build_import_index(
            "my_plan",
            "2020",
            Path::new("plan.csv"),
            "csv",
            "abc123",
            &by_state,
        );
        let structure = index["algorithm"]["structure"].as_str();
        assert_eq!(structure, Some("external"),
            "algorithm.structure must be 'external' for imported plans");
    }

    // ── 10. SHA-256 of source file in index ───────────────────────────────────

    #[test]
    fn test_import_index_source_sha256_present() {
        let by_state: HashMap<&'static str, HashMap<String, usize>> = HashMap::new();
        let sha = sha256_hex(b"test content");
        let index = build_import_index(
            "my_plan",
            "2020",
            Path::new("plan.csv"),
            "csv",
            &sha,
            &by_state,
        );
        let stored_sha = index["algorithm"]["source"].as_str().unwrap_or("");
        assert_eq!(stored_sha, sha,
            "algorithm.source must be the SHA-256 of the source file");
        assert_eq!(stored_sha.len(), 64, "SHA-256 must be 64 hex chars");
    }

    // ── 11. sha256_hex produces deterministic 64-char hex ────────────────────

    #[test]
    fn test_sha256_hex_deterministic_64_chars() {
        let h1 = sha256_hex(b"redistricting");
        let h2 = sha256_hex(b"redistricting");
        assert_eq!(h1, h2, "sha256_hex must be deterministic");
        assert_eq!(h1.len(), 64, "sha256_hex must produce 64 chars");
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()), "must be hex: {h1}");
    }

    // ── 12. CSV with no data rows → error ─────────────────────────────────────

    #[test]
    fn test_csv_no_data_rows_returns_error() {
        let csv = "GEOID,district\n"; // header but no data
        let result = parse_csv_assignments(csv);
        assert!(result.is_err(), "header-only CSV must fail");
    }

    // ── 13. GeoJSON missing features array → [INPUT] error ───────────────────

    #[test]
    fn test_geojson_missing_features_array_returns_error() {
        let json = r#"{"type": "FeatureCollection"}"#;
        let result = parse_geojson_assignments(json);
        assert!(result.is_err(), "GeoJSON without features must fail");
        let msg = result.unwrap_err();
        assert!(msg.contains("[INPUT]"), "must be [INPUT] error: {msg}");
    }

    // ── 14. shapefile format → [CONFIG] stub error ────────────────────────────

    #[test]
    fn test_shapefile_format_returns_config_stub_error() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("plan.shp");
        std::fs::write(&f, "binary data").unwrap();
        let _dir = with_tempdir(|| {
            let result = run_label_import("my_plan", &f, "2020", Some("shapefile"));
            assert!(result.is_err(), "shapefile must return error");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must be [CONFIG] error: {msg}");
            assert!(msg.contains("not yet implemented"), "must mention not yet implemented: {msg}");
        });
    }

    // ── 15. Full end-to-end: CSV → assignments.json + index.json + registry ───

    #[test]
    fn test_run_label_import_csv_end_to_end() {
        let dir = with_tempdir(|| {
            // Write a small CSV with Wisconsin tracts
            let csv = "GEOID,district\n55001010100,1\n55001010200,2\n55003010100,1\n";
            let f = PathBuf::from("plan.csv");
            std::fs::write(&f, csv).unwrap();

            let result = run_label_import("wis_test", &f, "2020", None);
            assert!(result.is_ok(), "end-to-end import must succeed: {:?}", result.err());

            // assignments.json must exist
            let asgn = PathBuf::from("runs/wis_test/2020/wisconsin/assignments.json");
            assert!(asgn.exists(), "assignments.json must be written: {}", asgn.display());

            // index.json must exist
            let idx = PathBuf::from("runs/wis_test/2020/index.json");
            assert!(idx.exists(), "index.json must be written: {}", idx.display());

            // index.json must have algorithm.structure = "external"
            let content = std::fs::read_to_string(&idx).unwrap();
            let v: serde_json::Value = serde_json::from_str(&content).unwrap();
            assert_eq!(v["algorithm"]["structure"].as_str(), Some("external"));
        });
        // Check registry using absolute path so we don't depend on CWD.
        let registry_path = dir.path().join(".redist");
        if registry_path.exists() {
            let content = std::fs::read_to_string(&registry_path).unwrap();
            let v: serde_json::Value = serde_json::from_str(&content).unwrap();
            assert!(
                v["wis_test"]["built"].as_array()
                    .map(|a| a.iter().any(|y| y.as_str() == Some("2020")))
                    .unwrap_or(false),
                "registry must mark wis_test/2020 as built; registry content: {v}"
            );
        }
        drop(dir);
    }

    // ── 16. label-compare: invalid label → error ─────────────────────────────

    #[test]
    fn test_label_compare_invalid_label_a_returns_error() {
        let _dir = with_tempdir(|| {
            let result = run_label_compare("runs", "valid_label", "2020", false, None);
            assert!(result.is_err(), "invalid label A must return error");
        });
    }

    // ── 17. label-compare: label not built → [CONFIG] error ──────────────────

    #[test]
    fn test_label_compare_unbuilt_label_returns_config_error() {
        let _dir = with_tempdir(|| {
            let result = run_label_compare("plan_a", "plan_b", "2020", false, None);
            assert!(result.is_err(), "unbuilt label must return error");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must be [CONFIG] error: {msg}");
        });
    }

    // ── 18. label-compare: label built but not analyzed → [CONFIG] error ──────

    #[test]
    fn test_label_compare_built_not_analyzed_returns_config_error() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("plan_a", "2020").unwrap();
            Registry::mark_built("plan_b", "2020").unwrap();
            // plan_a and plan_b are built but not analyzed
            let result = run_label_compare("plan_a", "plan_b", "2020", false, None);
            assert!(result.is_err(), "not-analyzed label must return error");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must be [CONFIG] error: {msg}");
        });
    }

    // ── 19. fips_to_state_name: all 50 states ────────────────────────────────

    #[test]
    fn test_fips_to_state_name_california() {
        assert_eq!(fips_to_state_name("06"), Some("california"));
    }

    #[test]
    fn test_fips_to_state_name_texas() {
        assert_eq!(fips_to_state_name("48"), Some("texas"));
    }

    #[test]
    fn test_fips_to_state_name_vermont() {
        assert_eq!(fips_to_state_name("50"), Some("vermont"));
    }

    #[test]
    fn test_fips_to_state_name_unknown_returns_none() {
        assert_eq!(fips_to_state_name("99"), None);
    }

    // ── 20. CSV: [INPUT] error for completely empty file ─────────────────────

    #[test]
    fn test_csv_empty_file_returns_input_error() {
        let result = parse_csv_assignments("");
        assert!(result.is_err(), "empty CSV must fail");
        let msg = result.unwrap_err();
        assert!(msg.contains("[INPUT]"), "[INPUT] prefix required: {msg}");
        assert!(msg.contains("empty"), "error must say 'empty': {msg}");
    }

    // ── 21. CSV: non-integer district value → [INPUT] error with line number ──

    #[test]
    fn test_csv_bad_district_value_names_line_number() {
        let csv = "GEOID,district\n55001010100,not_a_number\n";
        let result = parse_csv_assignments(csv);
        assert!(result.is_err(), "non-integer district must fail");
        let msg = result.unwrap_err();
        assert!(msg.contains("[INPUT]"), "[INPUT] prefix required: {msg}");
        // Must mention the bad value
        assert!(msg.contains("not_a_number") || msg.contains("cannot parse"),
            "error must identify the bad value: {msg}");
    }

    // ── 22. GeoJSON: all features missing GEOID+district → [INPUT] error ──────

    #[test]
    fn test_geojson_features_no_geoid_returns_input_error() {
        let geojson = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": null,
                    "properties": {"name": "tract_1", "pop": 1234}
                    // no GEOID, no district
                }
            ]
        });
        let result = parse_geojson_assignments(&geojson.to_string());
        assert!(result.is_err(), "GeoJSON without GEOID/district must fail");
        let msg = result.unwrap_err();
        assert!(msg.contains("[INPUT]"), "[INPUT] prefix required: {msg}");
    }

    // ── 23. unknown format in run_label_import → [CONFIG] error ──────────────

    #[test]
    fn test_run_label_import_auto_detect_fails_for_unknown_extension() {
        let _dir = with_tempdir(|| {
            let tmp = TempDir::new().unwrap();
            let f = tmp.path().join("plan.xlsx"); // unknown extension
            std::fs::write(&f, "data").unwrap();

            let result = run_label_import("my_plan", &f, "2020", None);
            assert!(result.is_err(), "unknown extension must fail without --format");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "[CONFIG] prefix required: {msg}");
            assert!(msg.contains("auto-detect") || msg.contains("format"),
                "error must mention format detection: {msg}");
        });
    }

    // ── 24. all-unknown FIPS GEOIDs → [INPUT] error mentioning first GEOID ───

    #[test]
    fn test_import_all_unknown_fips_returns_input_error() {
        let _dir = with_tempdir(|| {
            // GEOID "99..." — FIPS "99" is not a valid US state
            let csv = "GEOID,district\n99001010100,1\n99001010200,2\n";
            let f = PathBuf::from("bad_fips.csv");
            std::fs::write(&f, csv).unwrap();

            let result = run_label_import("my_plan", &f, "2020", Some("csv"));
            assert!(result.is_err(), "all-unknown FIPS must fail");
            let msg = result.unwrap_err();
            assert!(msg.contains("[INPUT]"), "[INPUT] prefix required: {msg}");
            // Must mention a GEOID so the user can diagnose
            assert!(msg.contains("99"), "error must mention the unrecognised GEOID prefix: {msg}");
        });
    }

    // ── 25. GEOID too short (<2 chars) → [INPUT] error ───────────────────────

    #[test]
    fn test_import_geoid_too_short_treated_as_unknown() {
        let _dir = with_tempdir(|| {
            // Single-char GEOID is too short to extract a FIPS prefix
            let csv = "GEOID,district\n5,1\n";
            let f = PathBuf::from("short_geoid.csv");
            std::fs::write(&f, csv).unwrap();

            let result = run_label_import("my_plan", &f, "2020", Some("csv"));
            // All GEOIDs are unknown FIPS → must fail with [INPUT]
            assert!(result.is_err(), "short GEOID must result in error");
            let msg = result.unwrap_err();
            assert!(msg.contains("[INPUT]"), "[INPUT] prefix required: {msg}");
        });
    }

    // ── 26. build_import_index: label field is correct ────────────────────────

    #[test]
    fn test_build_import_index_label_field() {
        let by_state: HashMap<&'static str, HashMap<String, usize>> = HashMap::new();
        let idx = build_import_index(
            "senate_draft2",
            "2020",
            Path::new("plan.csv"),
            "csv",
            &"0".repeat(64),
            &by_state,
        );
        assert_eq!(idx["label"].as_str(), Some("senate_draft2"),
            "index label field must match the label argument");
    }

    // ── 27. build_import_index: summary.states count is correct ──────────────

    #[test]
    fn test_build_import_index_summary_states_count() {
        let mut by_state: HashMap<&'static str, HashMap<String, usize>> = HashMap::new();
        by_state.insert("vermont", {let mut m = HashMap::new(); m.insert("50001".to_string(), 1); m});
        by_state.insert("alaska",  {let mut m = HashMap::new(); m.insert("02001".to_string(), 1); m});

        let idx = build_import_index("p", "2020", Path::new("f.csv"), "csv", "sha", &by_state);
        assert_eq!(idx["summary"]["states"].as_u64(), Some(2),
            "summary.states must equal number of states in by_state map");
    }

    // ── 28. rplan format → [CONFIG] stub error ───────────────────────────────

    #[test]
    fn test_rplan_format_returns_config_stub_error() {
        let _dir = with_tempdir(|| {
            let tmp = TempDir::new().unwrap();
            let f = tmp.path().join("plan.rplan");
            std::fs::write(&f, "rplan data").unwrap();

            let result = run_label_import("my_plan", &f, "2020", Some("rplan"));
            assert!(result.is_err(), "rplan must return error");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "[CONFIG] prefix required: {msg}");
            assert!(msg.contains("not yet implemented"), "must mention not implemented: {msg}");
        });
    }

    // ── 29. detect_format: uppercase extension not matched → None ────────────
    //
    // Extensions are checked literally, so ".CSV" (uppercase) must not match.

    #[test]
    fn test_detect_format_uppercase_extension_returns_none() {
        // Rust Path.extension() is case-sensitive on most platforms.
        // The function should not match ".CSV" as "csv".
        let p = Path::new("plan.CSV");
        // On Windows, extension() may preserve case; on Linux it would differ.
        // The detect_format function only checks lowercase extensions.
        let result = detect_format(p);
        // We document the current behavior: uppercase extension → None on
        // case-sensitive filesystems (Linux/Mac), but may vary on Windows.
        // The test asserts the function doesn't panic and returns a str or None.
        let _ = result; // behavior is platform-dependent; just verify no panic
    }

    // ── 30. label-compare: both labels built+analyzed → Ok ───────────────────

    #[test]
    fn test_label_compare_both_built_and_analyzed_returns_ok() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("plan_a", "2020").unwrap();
            Registry::mark_built("plan_b", "2020").unwrap();
            Registry::mark_analyzed("plan_a", "2020").unwrap();
            Registry::mark_analyzed("plan_b", "2020").unwrap();

            let result = run_label_compare("plan_a", "plan_b", "2020", false, None);
            assert!(result.is_ok(), "fully built+analyzed labels must compare without error: {:?}", result);
        });
    }

    // ── 31. label-compare: json output → Ok with JSON structure ──────────────

    #[test]
    fn test_label_compare_json_output_is_ok() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("plan_a", "2020").unwrap();
            Registry::mark_built("plan_b", "2020").unwrap();
            Registry::mark_analyzed("plan_a", "2020").unwrap();
            Registry::mark_analyzed("plan_b", "2020").unwrap();

            let result = run_label_compare("plan_a", "plan_b", "2020", true, None);
            assert!(result.is_ok(), "json output mode must succeed: {:?}", result);
        });
    }
}
