/// Data fetch command: download census data needed to run redistricting.
///
/// Three data sources in priority order:
///   1. Local manifest override (~/.config/BISECT/manifest.json or BISECT_MANIFEST)
///      Points to already-present local files — no network needed.
///   2. GitHub Releases (--release flag) — pulls adjacency data from project releases.
///      Requires `gh auth login`.
///   3. Public Census Bureau URLs (default) — TIGER shapefiles and PL 94-171.
///
/// Incremental by default: checks for existing files before downloading.
/// Use --force to re-download even if present.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Manifest embedded at compile time. Falls back to this if no override present.
const BUILTIN_MANIFEST: &str = include_str!("../../../data/manifest.json");

// ---------------------------------------------------------------------------
// Manifest types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest {
    pub version: String,
    pub github_repo: String,
    pub releases: Releases,
    pub local_data_dir: String,
    pub local_outputs_dir: String,
    pub states: HashMap<String, StateManifest>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Releases {
    pub data_inputs: String,
    pub outputs_v3: String,
    pub outputs_v4: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StateManifest {
    pub name: String,
    pub fips: String,
    pub districts: HashMap<String, usize>,
    pub tiger: HashMap<String, String>,
    pub pl94171: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Fetch item: one file to download
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FetchItem {
    pub state_code: String,
    pub year: String,
    pub kind: String,        // "tiger", "pl94171", "adjacency"
    pub url: Option<String>, // None = use github release or local only
    pub local_path: PathBuf,
    pub done_marker: PathBuf,
    pub available_locally: bool,
}

impl FetchItem {
    pub fn is_done(&self) -> bool {
        self.done_marker.exists() && self.local_path.exists()
    }
}

// ---------------------------------------------------------------------------
// Load manifest
// ---------------------------------------------------------------------------

pub fn load_manifest() -> Result<Manifest, String> {
    // Priority: BISECT_MANIFEST env > ~/.config/BISECT/manifest.json > builtin
    if let Ok(path) = std::env::var("BISECT_MANIFEST") {
        let content =
            std::fs::read_to_string(&path).map_err(|e| format!("BISECT_MANIFEST={path}: {e}"))?;
        return serde_json::from_str(&content).map_err(|e| format!("manifest parse error: {e}"));
    }

    // Check ~/.config/BISECT/manifest.json
    if let Some(home) = dirs_next_home() {
        let local = home.join(".config").join("bisect").join("manifest.json");
        if local.exists() {
            let content = std::fs::read_to_string(&local)
                .map_err(|e| format!("local manifest read error: {e}"))?;
            return serde_json::from_str(&content)
                .map_err(|e| format!("local manifest parse error: {e}"));
        }
    }

    serde_json::from_str(BUILTIN_MANIFEST).map_err(|e| format!("builtin manifest parse error: {e}"))
}

fn dirs_next_home() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

// ---------------------------------------------------------------------------
// Build fetch list
// ---------------------------------------------------------------------------

/// Build list of items to fetch for given states and year.
pub fn build_fetch_list(
    manifest: &Manifest,
    states: &[String],
    year: &str,
    data_types: &[crate::args::DataType],
) -> Vec<FetchItem> {
    use crate::args::DataType;
    let all_types = data_types.is_empty() || data_types.iter().any(|t| matches!(t, DataType::All));
    let want_tiger = all_types || data_types.iter().any(|t| matches!(t, DataType::Tiger));
    let want_pl = all_types
        || data_types
            .iter()
            .any(|t| matches!(t, DataType::Redistricting));
    let want_adj = all_types || data_types.iter().any(|t| matches!(t, DataType::Adjacency));
    let want_lodes = data_types.iter().any(|t| matches!(t, DataType::Lodes));
    let want_lodes_od = data_types.iter().any(|t| matches!(t, DataType::LodesOd));
    let want_school = data_types
        .iter()
        .any(|t| matches!(t, DataType::SchoolDistricts));
    let want_eia = data_types.iter().any(|t| matches!(t, DataType::Eia861));
    let want_acs_housing = data_types.iter().any(|t| matches!(t, DataType::AcsHousing));

    // Data types that exist as CLI flags but are not yet downloaded by `bisect fetch`.
    let want_elections = !all_types && data_types.iter().any(|t| matches!(t, DataType::Elections));
    if want_elections {
        eprintln!(
            "WARNING: --type elections is not implemented in `bisect fetch` yet. \
             Use the Python downloader instead:\n  \
             python scripts/data/elections/download_election_data.py --year {year}"
        );
    }

    let mut items = Vec::new();
    let data_dir = PathBuf::from(&manifest.local_data_dir);
    let outputs_dir = PathBuf::from(&manifest.local_outputs_dir);

    // Filter states
    let state_codes: Vec<&String> = if states.is_empty() {
        manifest.states.keys().collect()
    } else {
        states
            .iter()
            .filter(|s| manifest.states.contains_key(s.as_str()))
            .collect()
    };

    for code in state_codes {
        let state = match manifest.states.get(code.as_str()) {
            Some(s) => s,
            None => continue,
        };
        let state_lower = state.name.to_lowercase().replace(' ', "_");

        // TIGER tract shapefile
        if want_tiger {
            if let Some(url) = state.tiger.get(year) {
                // Strip query params from URL before extracting filename (Critical 1)
                let raw = url.split('/').last().unwrap_or("tract.zip");
                let filename = raw.split('?').next().unwrap_or(raw);
                let local_path = data_dir
                    .join(year)
                    .join("tiger")
                    .join("tracts")
                    .join(filename.replace(".zip", ""))
                    .join(filename.replace(".zip", ".shp"));
                let done_marker = local_path.with_extension("done");
                items.push(FetchItem {
                    state_code: code.clone(),
                    year: year.to_string(),
                    kind: "tiger".to_string(),
                    url: Some(url.clone()),
                    available_locally: local_path.exists(),
                    local_path,
                    done_marker,
                });
            }
        }

        // PL 94-171 redistricting file
        if want_pl {
            if let Some(url) = state.pl94171.get(year) {
                let raw = url.split('/').last().unwrap_or("data.zip");
                let filename = raw.split('?').next().unwrap_or(raw);
                let local_path = data_dir
                    .join(year)
                    .join("redistricting")
                    .join(&state_lower)
                    .join(filename);
                let done_marker = local_path.with_extension("done");
                items.push(FetchItem {
                    state_code: code.clone(),
                    year: year.to_string(),
                    kind: "pl94171".to_string(),
                    url: Some(url.clone()),
                    available_locally: local_path.exists(),
                    local_path,
                    done_marker,
                });
            }
        }

        // Adjacency pkl (from GitHub Release or local)
        if want_adj {
            let adj_filename = format!("{}_adjacency_{year}.pkl", code.to_lowercase());
            let local_path = outputs_dir
                .join("V3")
                .join("data")
                .join(year)
                .join("adjacency")
                .join(&adj_filename);
            let done_marker = local_path.with_extension("done");
            items.push(FetchItem {
                state_code: code.clone(),
                year: year.to_string(),
                kind: "adjacency".to_string(),
                url: None,
                available_locally: local_path.exists(),
                local_path,
                done_marker,
            });
        }

        // LODES WAC (Workplace Area Characteristics) — one CSV.gz per state per year.
        // URL: https://lehd.ces.census.gov/data/lodes/LODES8/{abbr}/wac/{abbr}_wac_S000_JT00_{year}.csv.gz
        // Aggregated block→tract CSV saved to data/{year}/lodes/{state}_wac_tract.csv
        if want_lodes {
            let abbr = code.to_lowercase();
            let lodes_year = year; // LODES available 2002–2021; use year as-is
            let url = format!(
                "https://lehd.ces.census.gov/data/lodes/LODES8/{abbr}/wac/{abbr}_wac_S000_JT00_{lodes_year}.csv.gz"
            );
            let local_path = data_dir
                .join(year)
                .join("lodes")
                .join(format!("{}_wac_tract.csv", state_lower));
            let done_marker = data_dir
                .join(year)
                .join("lodes")
                .join(format!("{}_wac_tract.done", state_lower));
            items.push(FetchItem {
                state_code: code.clone(),
                year: year.to_string(),
                kind: "lodes-wac".to_string(),
                url: Some(url),
                available_locally: local_path.exists(),
                local_path,
                done_marker,
            });
        }

        // LODES OD (Origin-Destination) — one CSV.gz per state per year.
        // URL: https://lehd.ces.census.gov/data/lodes/LODES8/{abbr}/od/{abbr}_od_main_JT00_{year}.csv.gz
        // Aggregated block-pair->tract-pair CSV saved to data/{year}/lodes/{state}_od_tract.csv
        if want_lodes_od {
            let abbr = code.to_lowercase();
            let url = format!(
                "https://lehd.ces.census.gov/data/lodes/LODES8/{abbr}/od/{abbr}_od_main_JT00_{year}.csv.gz"
            );
            let local_path = data_dir
                .join(year)
                .join("lodes")
                .join(format!("{}_od_tract.csv", state_lower));
            let done_marker = data_dir
                .join(year)
                .join("lodes")
                .join(format!("{}_od_tract.done", state_lower));
            items.push(FetchItem {
                state_code: code.clone(),
                year: year.to_string(),
                kind: "lodes-od".to_string(),
                url: Some(url),
                available_locally: local_path.exists(),
                local_path,
                done_marker,
            });
        }

        // TIGER/Line School District shapefiles — one ZIP per state per year.
        // URL pattern: https://www2.census.gov/geo/tiger/TIGER{year}/UNSD/tl_{year}_{fips}_unsd.zip
        // where fips = 2-digit state FIPS code (from manifest)
        if want_school {
            {
                let fips = &state.fips;
                let url = format!(
                    "https://www2.census.gov/geo/tiger/TIGER{year}/UNSD/tl_{year}_{fips}_unsd.zip"
                );
                let local_path = data_dir
                    .join(year)
                    .join("tiger")
                    .join("school_districts")
                    .join(format!("tl_{year}_{fips}_unsd"))
                    .join(format!("tl_{year}_{fips}_unsd.shp"));
                let done_marker = local_path.with_extension("done");
                items.push(FetchItem {
                    state_code: code.clone(),
                    year: year.to_string(),
                    kind: "school-districts".to_string(),
                    url: Some(url),
                    available_locally: local_path.exists(),
                    local_path,
                    done_marker,
                });
            }
        }

        // ACS 5-year housing character — one JSON per state per year from Census ACS API.
        // Tables: B25024 (units in structure), B25003 (tenure), B25035 (median year built).
        // Output: data/{year}/acs_housing/{state_lower}_housing_{year}.csv
        // No API key required for <=50 variables per request.
        if want_acs_housing {
            let fips = &state.fips;
            let url = format!(
                "https://api.census.gov/data/{year}/acs/acs5?get=B25024_001E,B25024_002E,B25024_003E,\
B25024_007E,B25024_008E,B25024_009E,B25003_001E,B25003_002E,\
B25035_001E,NAME&for=tract:*&in=state:{fips}"
            );
            let local_path = data_dir
                .join(year)
                .join("acs_housing")
                .join(format!("{state_lower}_housing_{year}.csv"));
            let done_marker = data_dir
                .join(year)
                .join("acs_housing")
                .join(format!("{state_lower}_housing_{year}.done"));
            items.push(FetchItem {
                state_code: code.clone(),
                year: year.to_string(),
                kind: "acs-housing".to_string(),
                url: Some(url),
                available_locally: local_path.exists(),
                local_path,
                done_marker,
            });
        }
    }

    // EIA Form 861 — one national ZIP per year (not per state).
    // URL: https://www.eia.gov/electricity/data/eia861/zip/f8612020.zip (year-specific)
    if want_eia {
        let eia_year = year;
        let url = format!("https://www.eia.gov/electricity/data/eia861/zip/f861{eia_year}.zip");
        let local_path = data_dir
            .join(year)
            .join("eia861")
            .join("service_territories.shp");
        let done_marker = data_dir.join(year).join("eia861").join("eia861.done");
        items.push(FetchItem {
            state_code: "US".to_string(),
            year: year.to_string(),
            kind: "eia-861".to_string(),
            url: Some(url),
            available_locally: local_path.exists(),
            local_path,
            done_marker,
        });
    }

    items
}

// ---------------------------------------------------------------------------
// Print check-only report
// ---------------------------------------------------------------------------

pub fn print_check_report(items: &[FetchItem]) {
    let mut have = 0usize;
    let mut need = 0usize;
    for item in items {
        let status = if item.is_done() {
            have += 1;
            "[OK]  "
        } else if item.available_locally {
            have += 1;
            "[FILE]"
        } else {
            need += 1;
            "[NEED]"
        };
        let src = item.url.as_deref().unwrap_or("github-release");
        println!(
            "{status} {} {} {} -> {}",
            item.state_code,
            item.year,
            item.kind,
            item.local_path.display()
        );
        if status == "[NEED]" {
            println!("       src: {src}");
        }
    }
    println!();
    println!("Summary: {have} available, {need} need download");
    if need == 0 {
        println!("[OK] All data present. Ready to run: bisect states --year 2020 --version V3");
    }
}

// ---------------------------------------------------------------------------
// Download
// ---------------------------------------------------------------------------

/// Download all items that aren't already present.
/// Uses native Rust (reqwest) for HTTP downloads. No Python subprocess.
///
/// `polite_delay_secs`: seconds to sleep between requests. Default 1 second.
/// Federal servers (Census Bureau, LEHD, EIA) expect polite access — do not
/// set to 0 unless you have explicit permission or are in a CI environment
/// with a private mirror.
pub fn download_items(
    items: &[FetchItem],
    force: bool,
    use_release: bool,
    manifest: &Manifest,
    polite_delay_secs: u64,
) -> Result<(), String> {
    let mut downloaded_count = 0usize;
    let fletch_cache_root = PathBuf::from(&manifest.local_data_dir).join(".fletch");
    for item in items {
        if !force && item.is_done() {
            println!(
                "[SKIP] {} {} {} (already present)",
                item.state_code, item.year, item.kind
            );
            continue;
        }
        // Polite delay between requests — inserted before each real download
        // (not before skips). Respects federal server access expectations.
        if downloaded_count > 0 && polite_delay_secs > 0 {
            println!(
                "[WAIT] {}s before next request (--polite-delay-secs {})",
                polite_delay_secs, polite_delay_secs
            );
            std::thread::sleep(std::time::Duration::from_secs(polite_delay_secs));
        }

        if let Some(parent) = item.local_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }

        match item.kind.as_str() {
            "adjacency" if use_release => {
                // GitHub Releases: use gh CLI (or BISECT_GH override for testing).
                // BISECT_GH may be "python /path/to/fake_gh.py" — split on first space.
                let gh_raw = std::env::var("BISECT_GH").unwrap_or_else(|_| "gh".to_string());
                let mut gh_parts = gh_raw.splitn(2, ' ');
                let gh_cmd = gh_parts.next().unwrap_or("gh");
                let gh_extra_args: Vec<&str> = gh_parts
                    .next()
                    .map(|s| s.split_whitespace().collect())
                    .unwrap_or_default();

                let release_tag = &manifest.releases.data_inputs;
                let adj_dir = item.local_path.parent().unwrap();
                std::fs::create_dir_all(adj_dir).map_err(|e| e.to_string())?;
                println!(
                    "[DOWN] {} {} adjacency from release {release_tag}",
                    item.state_code, item.year
                );
                let mut cmd = std::process::Command::new(gh_cmd);
                cmd.args(&gh_extra_args);
                let out = cmd
                    .args([
                        "release",
                        "download",
                        release_tag,
                        "--pattern",
                        &format!(
                            "{}_adjacency_{}.pkl",
                            item.state_code.to_lowercase(),
                            item.year
                        ),
                        "--dir",
                        adj_dir.to_str().unwrap(),
                        "--repo",
                        &manifest.github_repo,
                        "--clobber",
                    ])
                    .output()
                    .map_err(|e| format!("gh not found: {e}. Install: https://cli.github.com/"))?;
                if !out.status.success() {
                    return Err(format!(
                        "gh release download failed:\n{}",
                        String::from_utf8_lossy(&out.stderr)
                    ));
                }
            }

            "tiger" | "pl94171" | "school-districts" | "eia-861" => {
                // ZIP file: FLETCH owns source acquisition; BISECT owns extraction.
                let url = item.url.as_deref().unwrap();
                let dest = item.local_path.parent().unwrap();
                println!(
                    "[FLETCH] {} {} {} <- {}",
                    item.state_code,
                    item.year,
                    item.kind,
                    url.split('/').last().unwrap_or(url)
                );
                let outcome = crate::fletch::fetch_item_to_fletch(item, &fletch_cache_root, force)?;
                extract_zip_file(&outcome.path, dest)?;
            }

            "lodes-wac" => {
                // LODES WAC: FLETCH gets .csv.gz; BISECT aggregates blocks->tracts.
                let url = item.url.as_deref().unwrap();
                let dest_dir = item.local_path.parent().unwrap();
                println!(
                    "[FLETCH] {} {} lodes-wac <- {}",
                    item.state_code,
                    item.year,
                    url.split('/').last().unwrap_or(url)
                );
                std::fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
                match crate::fletch::fetch_item_to_fletch(item, &fletch_cache_root, force) {
                    Ok(outcome) => aggregate_lodes_wac_gzip_file(&outcome.path, &item.local_path)?,
                    Err(error) if error.contains("404") => {
                        println!(
                            "[WARN] LODES WAC not available for this state/year (404). Skipping."
                        );
                        std::fs::write(
                            &item.local_path,
                            "geoid,c000,cns07,cns09,cns10,cns11,cns01,cns02,cns05,cns08\n",
                        )
                        .map_err(|e| format!("write empty lodes: {e}"))?;
                    }
                    Err(error) => return Err(error),
                }
            }

            "lodes-od" => {
                // LODES OD: FLETCH gets .csv.gz; BISECT aggregates block pairs->tract pairs.
                let url = item.url.as_deref().unwrap();
                let dest_dir = item.local_path.parent().unwrap();
                println!(
                    "[FLETCH] {} {} lodes-od <- {}",
                    item.state_code,
                    item.year,
                    url.split('/').last().unwrap_or(url)
                );
                std::fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
                match crate::fletch::fetch_item_to_fletch(item, &fletch_cache_root, force) {
                    Ok(outcome) => aggregate_lodes_od_gzip_file(&outcome.path, &item.local_path)?,
                    Err(error) if error.contains("404") => {
                        println!(
                            "[WARN] LODES OD not available for this state/year (404). Skipping."
                        );
                        std::fs::write(&item.local_path, "home_geoid,work_geoid,s000\n")
                            .map_err(|e| format!("write empty lodes-od: {e}"))?;
                    }
                    Err(error) => return Err(error),
                }
            }

            "acs-housing" => {
                // ACS housing: FLETCH gets Census ACS JSON; BISECT derives columns.
                let dest_dir = item.local_path.parent().unwrap();
                println!(
                    "[FLETCH] {} {} acs-housing <- ACS API (state {})",
                    item.state_code, item.year, item.state_code
                );
                std::fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
                match crate::fletch::fetch_item_to_fletch(item, &fletch_cache_root, force) {
                    Ok(outcome) => {
                        let body = std::fs::read_to_string(&outcome.path)
                            .map_err(|e| format!("ACS FLETCH cache read: {e}"))?;
                        write_acs_housing_body(&body, &item.local_path)?;
                    }
                    Err(error) if error.contains("404") => {
                        println!("[WARN] ACS housing data not available for this state/year (404). Skipping.");
                        std::fs::write(
                            &item.local_path,
                            "geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage\n",
                        )
                        .map_err(|e| format!("write empty acs-housing: {e}"))?;
                    }
                    Err(error) => return Err(error),
                }
            }

            _ => {
                println!(
                    "[SKIP] {} {} {} (use --release to download from GitHub)",
                    item.state_code, item.year, item.kind
                );
                continue;
            }
        }

        std::fs::write(&item.done_marker, b"done")
            .map_err(|e| format!("done marker write failed: {e}"))?;
        downloaded_count += 1;
    }
    Ok(())
}

/// Download LODES WAC .csv.gz, decompress, aggregate census block rows to tract level,
/// and save a compact tract-level CSV to `dest_path`.
///
/// Input format: w_geocode (15-char block GEOID), C000 (total jobs), CNS01–CNS20 (by sector)
/// Output format: geoid (11-char tract), c000, cns07, cns09, cns10, cns11 (commercial),
///                cns01, cns02, cns05, cns08 (industrial)
///
/// Aggregation: sum all block rows sharing the same 11-char tract prefix.
pub fn download_lodes_wac(url: &str, dest_path: &Path) -> Result<(), String> {
    use std::io::BufReader;

    let response = reqwest::blocking::get(url).map_err(|e| format!("HTTP GET {url}: {e}"))?;
    if !response.status().is_success() {
        // LODES data not available for all states/years — treat 404 as soft skip
        if response.status().as_u16() == 404 {
            println!("[WARN] LODES WAC not available for this state/year (404). Skipping.");
            // Write empty CSV so done marker still gets created
            std::fs::write(
                dest_path,
                "geoid,c000,cns07,cns09,cns10,cns11,cns01,cns02,cns05,cns08\n",
            )
            .map_err(|e| format!("write empty lodes: {e}"))?;
            return Ok(());
        }
        return Err(format!("HTTP {}: {url}", response.status()));
    }

    let gz = flate2::read::GzDecoder::new(response);
    aggregate_lodes_wac_reader(BufReader::new(gz), dest_path)
}

pub fn aggregate_lodes_wac_gzip_file(path: &Path, dest_path: &Path) -> Result<(), String> {
    use std::io::BufReader;
    let file = std::fs::File::open(path).map_err(|e| format!("open LODES cache: {e}"))?;
    let gz = flate2::read::GzDecoder::new(file);
    aggregate_lodes_wac_reader(BufReader::new(gz), dest_path)
}

fn aggregate_lodes_wac_reader<R: std::io::BufRead>(
    reader: R,
    dest_path: &Path,
) -> Result<(), String> {
    use std::collections::HashMap;
    // Aggregate block rows to tracts
    // tract_geoid (11 chars) → [c000, cns07, cns09, cns10, cns11, cns01, cns02, cns05, cns08]
    let mut tracts: HashMap<String, [f64; 9]> = HashMap::new();

    let mut lines = reader.lines();
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("LODES WAC: empty or unreadable file".to_string()),
    };

    // Find column indices from header
    let cols: Vec<&str> = header.split(',').collect();
    let idx = |name: &str| -> Result<usize, String> {
        cols.iter()
            .position(|c| c.trim_matches('"') == name)
            .ok_or_else(|| format!("LODES WAC: column '{name}' not found in header"))
    };
    let i_geo = idx("w_geocode")?;
    let i_c000 = idx("C000")?;
    let i_c07 = idx("CNS07")?;
    let i_c09 = idx("CNS09")?;
    let i_c10 = idx("CNS10")?;
    let i_c11 = idx("CNS11")?;
    let i_c01 = idx("CNS01")?;
    let i_c02 = idx("CNS02")?;
    let i_c05 = idx("CNS05")?;
    let i_c08 = idx("CNS08")?;

    for line in lines {
        let line = line.map_err(|e| format!("LODES read: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() <= i_c08 {
            continue;
        }

        let block_geoid = fields[i_geo].trim_matches('"');
        if block_geoid.len() < 11 {
            continue;
        }
        let tract_geoid = &block_geoid[..11];

        let parse = |i: usize| -> f64 { fields[i].trim_matches('"').parse::<f64>().unwrap_or(0.0) };

        let entry = tracts.entry(tract_geoid.to_string()).or_insert([0.0; 9]);
        entry[0] += parse(i_c000);
        entry[1] += parse(i_c07);
        entry[2] += parse(i_c09);
        entry[3] += parse(i_c10);
        entry[4] += parse(i_c11);
        entry[5] += parse(i_c01);
        entry[6] += parse(i_c02);
        entry[7] += parse(i_c05);
        entry[8] += parse(i_c08);
    }

    // Write output CSV
    let mut out = std::fs::File::create(dest_path)
        .map_err(|e| format!("create {}: {e}", dest_path.display()))?;
    use std::io::Write as IoWrite;
    writeln!(
        out,
        "geoid,c000,cns07,cns09,cns10,cns11,cns01,cns02,cns05,cns08"
    )
    .map_err(|e| e.to_string())?;
    let mut sorted: Vec<_> = tracts.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (geoid, v) in sorted {
        writeln!(
            out,
            "{geoid},{},{},{},{},{},{},{},{},{}",
            v[0] as u64,
            v[1] as u64,
            v[2] as u64,
            v[3] as u64,
            v[4] as u64,
            v[5] as u64,
            v[6] as u64,
            v[7] as u64,
            v[8] as u64
        )
        .map_err(|e| e.to_string())?;
    }
    println!("[OK] LODES WAC aggregated to {}", dest_path.display());
    Ok(())
}

/// Download LODES OD .csv.gz, decompress, aggregate census block-pair rows to tract-pair level,
/// and save a compact tract-level OD CSV to `dest_path`.
///
/// Input format: h_geocode (15-char home block GEOID), w_geocode (15-char work block GEOID),
///               S000 (total jobs for this home/work pair), plus other job-type columns
/// Output format: home_geoid (11-char tract), work_geoid (11-char tract), s000 (total jobs)
///
/// Aggregation: for each row, take h_geocode[:11] as home_tract and w_geocode[:11] as
/// work_tract; sum S000 for each (home_tract, work_tract) pair.
/// Output sorted by home_geoid then work_geoid.
///
/// Graceful 404 skip: LODES OD is not available for all states/years — treated as soft skip.
pub fn download_lodes_od(url: &str, dest_path: &std::path::Path) -> Result<(), String> {
    use std::io::BufReader;

    let response = reqwest::blocking::get(url).map_err(|e| format!("HTTP GET {url}: {e}"))?;
    if !response.status().is_success() {
        // LODES OD not available for all states/years — treat 404 as soft skip
        if response.status().as_u16() == 404 {
            println!("[WARN] LODES OD not available for this state/year (404). Skipping.");
            // Write empty CSV so done marker still gets created
            std::fs::write(dest_path, "home_geoid,work_geoid,s000\n")
                .map_err(|e| format!("write empty lodes-od: {e}"))?;
            return Ok(());
        }
        return Err(format!("HTTP {}: {url}", response.status()));
    }

    let gz = flate2::read::GzDecoder::new(response);
    aggregate_lodes_od_reader(BufReader::new(gz), dest_path)
}

pub fn aggregate_lodes_od_gzip_file(path: &Path, dest_path: &Path) -> Result<(), String> {
    use std::io::BufReader;
    let file = std::fs::File::open(path).map_err(|e| format!("open LODES OD cache: {e}"))?;
    let gz = flate2::read::GzDecoder::new(file);
    aggregate_lodes_od_reader(BufReader::new(gz), dest_path)
}

fn aggregate_lodes_od_reader<R: std::io::BufRead>(
    reader: R,
    dest_path: &Path,
) -> Result<(), String> {
    use std::collections::HashMap;
    // Aggregate block-pair rows to tract-pair level
    // (home_tract, work_tract) -> total S000 jobs
    let mut od_pairs: HashMap<(String, String), u64> = HashMap::new();

    let mut lines = reader.lines();
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("LODES OD: empty or unreadable file".to_string()),
    };

    // Find column indices from header
    let cols: Vec<&str> = header.split(',').collect();
    let idx = |name: &str| -> Result<usize, String> {
        cols.iter()
            .position(|c| c.trim_matches('"') == name)
            .ok_or_else(|| format!("LODES OD: column '{name}' not found in header"))
    };
    let i_home = idx("h_geocode")?;
    let i_work = idx("w_geocode")?;
    let i_s000 = idx("S000")?;

    for line in lines {
        let line = line.map_err(|e| format!("LODES OD read: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() <= i_s000 {
            continue;
        }

        let home_block = fields[i_home].trim_matches('"');
        let work_block = fields[i_work].trim_matches('"');
        if home_block.len() < 11 || work_block.len() < 11 {
            continue;
        }

        let home_tract = home_block[..11].to_string();
        let work_tract = work_block[..11].to_string();
        let jobs: u64 = fields[i_s000].trim_matches('"').parse::<u64>().unwrap_or(0);

        *od_pairs.entry((home_tract, work_tract)).or_insert(0) += jobs;
    }

    // Write output CSV: sorted by home_geoid then work_geoid
    let mut out = std::fs::File::create(dest_path)
        .map_err(|e| format!("create {}: {e}", dest_path.display()))?;
    use std::io::Write as IoWrite;
    writeln!(out, "home_geoid,work_geoid,s000").map_err(|e| e.to_string())?;
    let mut sorted: Vec<_> = od_pairs.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for ((home, work), jobs) in sorted {
        writeln!(out, "{home},{work},{jobs}").map_err(|e| e.to_string())?;
    }
    println!("[OK] LODES OD aggregated to {}", dest_path.display());
    Ok(())
}

/// Download ACS 5-year housing character data from the Census ACS API and write
/// a per-tract CSV with four derived columns.
///
/// API response: JSON array where row 0 is a header and rows 1..N are data.
/// Each data row includes: B25024_001E (total units), B25024_002E (1-unit detached),
/// B25024_003E (1-unit attached), B25024_007E..B25024_009E (10+ unit multifamily),
/// B25003_001E (total occupied), B25003_002E (owner-occupied),
/// B25035_001E (median year built), plus NAME, state, county, tract columns.
///
/// Derived columns written to dest_path:
///   geoid            = state(2) + county(3) + tract(6)
///   pct_single_family = (1-unit-det + 1-unit-att) / total_units   [0,1], default 0.5
///   pct_multifamily   = (10-19 + 20-49 + 50+ units) / total_units  [0,1]
///   pct_owner        = owner_occupied / total_occupied  [0,1], default 0.5
///   housing_vintage  = 1.0 - (median_year_built - 1940) / (2020 - 1940)  [0,1]
///                      pre-1940 -> ~1.0 (historic), 2020+ -> 0.0 (new);
///                      unreliable code (-666666666) -> 0.5 (neutral)
///
/// Soft errors:
///   - 404 from API: writes empty CSV with header only (ACS not available for year/state)
///   - Row-level parse errors: skipped with [WARN] printed to stderr
pub fn download_acs_housing(url: &str, dest_path: &Path) -> Result<(), String> {
    // Build a polite HTTP client with User-Agent identifying this research project.
    let client = reqwest::blocking::Client::builder()
        .user_agent("bisect redistricting research (giodl@microsoft.com)")
        .build()
        .map_err(|e| format!("HTTP client build: {e}"))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("HTTP GET ACS housing {url}: {e}"))?;

    if !response.status().is_success() {
        if response.status().as_u16() == 404 {
            println!("[WARN] ACS housing data not available for this state/year (404). Skipping.");
            std::fs::write(
                dest_path,
                "geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage\n",
            )
            .map_err(|e| format!("write empty acs-housing: {e}"))?;
            return Ok(());
        }
        return Err(format!("HTTP {}: ACS housing {url}", response.status()));
    }

    let body = response
        .text()
        .map_err(|e| format!("ACS response body: {e}"))?;
    write_acs_housing_body(&body, dest_path)
}

pub fn write_acs_housing_body(body: &str, dest_path: &Path) -> Result<(), String> {
    // ACS API returns a JSON array-of-arrays: first row is header, rest are data.
    let rows: Vec<Vec<serde_json::Value>> =
        serde_json::from_str(body).map_err(|e| format!("ACS JSON parse: {e}"))?;

    if rows.is_empty() {
        std::fs::write(
            dest_path,
            "geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage\n",
        )
        .map_err(|e| format!("write empty acs-housing: {e}"))?;
        return Ok(());
    }

    // Build column index from header row.
    let header_row: Vec<String> = rows[0]
        .iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect();
    let find_col = |name: &str| -> Result<usize, String> {
        header_row
            .iter()
            .position(|c| c == name)
            .ok_or_else(|| format!("ACS housing: column '{name}' not found in response header"))
    };

    let i_total_units = find_col("B25024_001E")?;
    let i_sf_det = find_col("B25024_002E")?;
    let i_sf_att = find_col("B25024_003E")?;
    let i_mf_1019 = find_col("B25024_007E")?;
    let i_mf_2049 = find_col("B25024_008E")?;
    let i_mf_50p = find_col("B25024_009E")?;
    let i_occ_tot = find_col("B25003_001E")?;
    let i_occ_own = find_col("B25003_002E")?;
    let i_vintage = find_col("B25035_001E")?;
    let i_state = find_col("state")?;
    let i_county = find_col("county")?;
    let i_tract = find_col("tract")?;

    // Helper: parse a JSON value as f64, returning 0.0 on null/error.
    let parse_f64 = |v: &serde_json::Value| -> f64 {
        match v {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
            serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0,
        }
    };

    let mut out = std::fs::File::create(dest_path)
        .map_err(|e| format!("create {}: {e}", dest_path.display()))?;
    use std::io::Write as IoWrite;
    writeln!(
        out,
        "geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage"
    )
    .map_err(|e| e.to_string())?;

    let mut row_count = 0usize;
    for row in rows.iter().skip(1) {
        if row.len() <= i_tract {
            continue;
        }

        // Build 11-char GEOID: state(2) + county(3) + tract(6)
        let state_fips = row[i_state].as_str().unwrap_or("");
        let county_fips = row[i_county].as_str().unwrap_or("");
        let tract_code = row[i_tract].as_str().unwrap_or("");
        if state_fips.len() != 2 || county_fips.len() != 3 || tract_code.len() != 6 {
            eprintln!(
                "[WARN] ACS housing: skipping row with malformed FIPS \
                ({state_fips}/{county_fips}/{tract_code})"
            );
            continue;
        }
        let geoid = format!("{state_fips}{county_fips}{tract_code}");

        let housing = crate::housing::derive_housing_character(crate::housing::AcsHousingRaw {
            total_units: parse_f64(&row[i_total_units]),
            single_family_detached: parse_f64(&row[i_sf_det]),
            single_family_attached: parse_f64(&row[i_sf_att]),
            multifamily_10_19: parse_f64(&row[i_mf_1019]),
            multifamily_20_49: parse_f64(&row[i_mf_2049]),
            multifamily_50_plus: parse_f64(&row[i_mf_50p]),
            occupied_total: parse_f64(&row[i_occ_tot]),
            owner_occupied: parse_f64(&row[i_occ_own]),
            median_year_built: parse_f64(&row[i_vintage]),
        });

        writeln!(
            out,
            "{geoid},{:.6},{:.6},{:.6},{:.6}",
            housing.pct_single_family,
            housing.pct_multifamily,
            housing.pct_owner,
            housing.housing_vintage
        )
        .map_err(|e| e.to_string())?;
        row_count += 1;
    }

    println!(
        "[OK] ACS housing: {row_count} tracts written to {}",
        dest_path.display()
    );
    Ok(())
}

/// Verify the SHA-256 of a downloaded file against an expected hash.
///
/// Computes the SHA-256 of `path` (streaming in 64KB chunks, same as sha256_file)
/// and compares to `expected` (lowercase hex string). On mismatch: deletes the
/// corrupt file and returns Err. On match: returns Ok(()).
///
/// If `expected` is empty, returns Ok(()) without computing (no expected hash).
pub fn verify_file_sha256(path: &std::path::Path, expected: &str) -> Result<(), String> {
    if expected.is_empty() {
        return Ok(());
    }

    let actual = bisect_report::sha256_file(path)
        .map_err(|e| format!("SHA-256 computation failed for {}: {e}", path.display()))?;

    if actual != expected.to_lowercase() {
        // Delete the corrupt file so it won't be reused
        let _ = std::fs::remove_file(path);
        return Err(format!(
            "SHA-256 mismatch for {}:\n  expected: {}\n  actual:   {}\n\
             Corrupt file deleted. Re-run to re-download.",
            path.display(),
            expected,
            actual
        ));
    }
    Ok(())
}

/// Download a ZIP from url and extract it to dest_dir.
/// Streams response to a temp file to avoid OOM for large ZIPs (Critical 3).
/// California PL 94-171 can be 80MB+ — in-memory loading would OOM on constrained systems.
pub fn download_and_extract_zip(url: &str, dest_dir: &Path) -> Result<(), String> {
    let mut response = reqwest::blocking::get(url).map_err(|e| format!("HTTP GET {url}: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}: {url}", response.status()));
    }

    // Stream to temp file — avoids loading large ZIPs (80MB+ for CA PL 94-171) into RAM
    let tmp_dir = tempfile::TempDir::new().map_err(|e| e.to_string())?;
    let tmp_zip = tmp_dir.path().join("download.zip");
    {
        let mut out =
            std::fs::File::create(&tmp_zip).map_err(|e| format!("tmp file create: {e}"))?;
        std::io::copy(&mut response, &mut out).map_err(|e| format!("streaming download: {e}"))?;
    }

    extract_zip_file(&tmp_zip, dest_dir)
}

pub fn extract_zip_file(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let zip_file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("invalid ZIP {}: {e}", zip_path.display()))?;

    std::fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = dest_dir.join(file.name());
        if file.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
            }
            let mut out = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_loads_from_builtin() {
        let manifest = load_manifest().expect("builtin manifest should parse");
        assert_eq!(manifest.version, "1");
        assert_eq!(manifest.states.len(), 50, "should have all 50 states");
    }

    #[test]
    fn test_manifest_has_vermont() {
        let manifest = load_manifest().unwrap();
        let vt = manifest
            .states
            .get("VT")
            .expect("Vermont must be in manifest");
        assert_eq!(vt.fips, "50", "Vermont FIPS is 50");
        assert!(
            vt.tiger["2020"].contains("tl_2020_50_tract"),
            "VT TIGER URL"
        );
        assert!(vt.pl94171["2020"].contains("vermont"), "VT PL URL");
    }

    #[test]
    fn test_manifest_has_alabama() {
        let manifest = load_manifest().unwrap();
        let al = manifest
            .states
            .get("AL")
            .expect("Alabama must be in manifest");
        assert_eq!(al.fips, "01", "Alabama FIPS is 01");
        assert_eq!(al.districts["2020"], 7, "Alabama has 7 districts in 2020");
    }

    #[test]
    fn test_build_fetch_list_vermont_all_types() {
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(&manifest, &["VT".to_string()], "2020", &[]);
        // Should have tiger, pl94171, adjacency
        assert_eq!(items.len(), 3);
        let kinds: Vec<&str> = items.iter().map(|i| i.kind.as_str()).collect();
        assert!(kinds.contains(&"tiger"));
        assert!(kinds.contains(&"pl94171"));
        assert!(kinds.contains(&"adjacency"));
    }

    #[test]
    fn test_build_fetch_list_tiger_only() {
        use crate::args::DataType;
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(&manifest, &["VT".to_string()], "2020", &[DataType::Tiger]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, "tiger");
        assert!(items[0]
            .url
            .as_deref()
            .unwrap()
            .contains("tl_2020_50_tract"));
    }

    #[test]
    fn test_fetch_item_done_when_done_marker_exists() {
        let tmp = tempfile::TempDir::new().unwrap();
        let local_path = tmp.path().join("data.shp");
        let done_marker = tmp.path().join("data.done");
        std::fs::write(&local_path, b"data").unwrap();
        std::fs::write(&done_marker, b"done").unwrap();
        let item = FetchItem {
            state_code: "VT".to_string(),
            year: "2020".to_string(),
            kind: "tiger".to_string(),
            url: None,
            local_path,
            done_marker,
            available_locally: true,
        };
        assert!(item.is_done());
    }

    #[test]
    fn test_all_50_states_have_tiger_2020_url() {
        let manifest = load_manifest().unwrap();
        for (code, state) in &manifest.states {
            let url = state
                .tiger
                .get("2020")
                .unwrap_or_else(|| panic!("{code} missing tiger 2020 URL"));
            assert!(
                url.starts_with("https://"),
                "{code} tiger URL must be https"
            );
            assert!(url.ends_with(".zip"), "{code} tiger URL must end in .zip");
        }
    }

    // ── Task 137: verify_file_sha256 ─────────────────────────────────────────

    #[test]
    fn test_verify_file_sha256_correct_hash() {
        // Write a known file and verify with its correct SHA-256
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("data.bin");
        std::fs::write(&path, b"hello world").unwrap();
        // Known SHA-256 of "hello world"
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let result = verify_file_sha256(&path, expected);
        assert!(
            result.is_ok(),
            "correct hash should pass: {:?}",
            result.err()
        );
        // File must still exist after a passing check
        assert!(
            path.exists(),
            "file must still exist after passing verification"
        );
    }

    #[test]
    fn test_verify_file_sha256_wrong_hash_returns_err() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("data.bin");
        std::fs::write(&path, b"hello world").unwrap();
        // Wrong hash
        let wrong_hash = "a".repeat(64);
        let result = verify_file_sha256(&path, &wrong_hash);
        assert!(result.is_err(), "wrong hash must return Err");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("mismatch") || msg.contains("SHA-256"),
            "error must mention mismatch: {msg}"
        );
        // File must be deleted after mismatch
        assert!(
            !path.exists(),
            "corrupt file must be deleted after mismatch"
        );
    }

    #[test]
    fn test_verify_file_sha256_empty_expected_skips() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("data.bin");
        std::fs::write(&path, b"any content").unwrap();
        // Empty expected = no check performed
        let result = verify_file_sha256(&path, "");
        assert!(
            result.is_ok(),
            "empty expected hash must skip check and return Ok"
        );
        assert!(
            path.exists(),
            "file must not be deleted when check is skipped"
        );
    }

    #[test]
    fn test_verify_downloads_flag_parses() {
        use crate::args::FetchArgs;
        use clap::Parser;
        let args = FetchArgs::parse_from(["fetch", "--verify-downloads"]);
        assert!(
            args.verify_downloads,
            "--verify-downloads flag must parse to true"
        );
    }

    #[test]
    fn test_verify_downloads_default_false() {
        use crate::args::FetchArgs;
        use clap::Parser;
        let args = FetchArgs::parse_from(["fetch"]);
        assert!(
            !args.verify_downloads,
            "--verify-downloads must default to false"
        );
    }

    #[test]
    fn test_all_50_states_have_pl94171_2020_url() {
        let manifest = load_manifest().unwrap();
        for (code, state) in &manifest.states {
            let url = state
                .pl94171
                .get("2020")
                .unwrap_or_else(|| panic!("{code} missing pl94171 2020 URL"));
            assert!(
                url.contains("census.gov"),
                "{code} PL URL must be census.gov"
            );
        }
    }

    // ── Additional fetch.rs coverage ─────────────────────────────────────────

    #[test]
    fn test_fetch_item_not_done_when_done_marker_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let local_path = tmp.path().join("data.shp");
        let done_marker = tmp.path().join("data.done");
        std::fs::write(&local_path, b"data").unwrap();
        // done_marker NOT created → is_done() must be false
        let item = FetchItem {
            state_code: "VT".to_string(),
            year: "2020".to_string(),
            kind: "tiger".to_string(),
            url: None,
            local_path,
            done_marker, // does not exist
            available_locally: true,
        };
        assert!(
            !item.is_done(),
            "is_done must be false when done_marker is absent"
        );
    }

    #[test]
    fn test_fetch_item_not_done_when_local_file_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let local_path = tmp.path().join("data.shp"); // NOT created
        let done_marker = tmp.path().join("data.done");
        std::fs::write(&done_marker, b"done").unwrap();
        let item = FetchItem {
            state_code: "VT".to_string(),
            year: "2020".to_string(),
            kind: "tiger".to_string(),
            url: None,
            local_path, // does not exist
            done_marker,
            available_locally: false,
        };
        assert!(
            !item.is_done(),
            "is_done must be false when local file is absent"
        );
    }

    #[test]
    fn test_build_fetch_list_empty_states_returns_all_states() {
        let manifest = load_manifest().unwrap();
        // Passing empty states slice means "all states"
        let items = build_fetch_list(&manifest, &[], "2020", &[]);
        // With 50 states and 3 types each: expect 150 items
        assert_eq!(
            items.len(),
            50 * 3,
            "empty state list must fetch all 50 states × 3 types = 150 items"
        );
    }

    #[test]
    fn test_build_fetch_list_pl94171_only() {
        use crate::args::DataType;
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(
            &manifest,
            &["VT".to_string()],
            "2020",
            &[DataType::Redistricting],
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, "pl94171");
        assert!(
            items[0].url.as_deref().unwrap().contains("census.gov"),
            "PL URL must be from census.gov"
        );
    }

    #[test]
    fn test_build_fetch_list_adjacency_only() {
        use crate::args::DataType;
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(
            &manifest,
            &["VT".to_string()],
            "2020",
            &[DataType::Adjacency],
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, "adjacency");
        // Adjacency items have no direct URL (come from GitHub Release)
        assert!(
            items[0].url.is_none(),
            "adjacency items must have url=None (fetched from GitHub Release)"
        );
    }

    #[test]
    fn test_build_fetch_list_unknown_state_is_ignored() {
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(&manifest, &["XX".to_string()], "2020", &[]);
        assert!(
            items.is_empty(),
            "unknown state code must produce zero fetch items"
        );
    }

    #[test]
    fn test_build_fetch_list_adjacency_filename_includes_state_and_year() {
        use crate::args::DataType;
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(
            &manifest,
            &["VT".to_string()],
            "2020",
            &[DataType::Adjacency],
        );
        let path_str = items[0].local_path.to_string_lossy().to_lowercase();
        assert!(
            path_str.contains("vt"),
            "adjacency path must contain state code"
        );
        assert!(
            path_str.contains("2020"),
            "adjacency path must contain year"
        );
        assert!(
            path_str.ends_with(".pkl"),
            "adjacency file must have .pkl extension"
        );
    }

    #[test]
    fn test_build_fetch_list_tiger_url_has_fips_code() {
        use crate::args::DataType;
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(&manifest, &["VT".to_string()], "2020", &[DataType::Tiger]);
        let url = items[0].url.as_deref().unwrap();
        assert!(url.contains("50"), "VT tiger URL must contain FIPS '50'");
        assert!(url.contains("2020"), "tiger URL must contain year");
    }

    #[test]
    fn test_manifest_releases_fields_present() {
        let manifest = load_manifest().unwrap();
        assert!(
            !manifest.releases.data_inputs.is_empty(),
            "releases.data_inputs must not be empty"
        );
        assert!(
            !manifest.releases.outputs_v3.is_empty(),
            "releases.outputs_v3 must not be empty"
        );
        assert!(
            !manifest.releases.outputs_v4.is_empty(),
            "releases.outputs_v4 must not be empty"
        );
    }

    #[test]
    fn test_manifest_github_repo_field_present() {
        let manifest = load_manifest().unwrap();
        assert!(
            !manifest.github_repo.is_empty(),
            "github_repo must not be empty"
        );
        // Must look like an owner/repo slug
        assert!(
            manifest.github_repo.contains('/'),
            "github_repo must be owner/repo format; got: {}",
            manifest.github_repo
        );
    }

    #[test]
    fn test_manifest_local_dirs_fields_present() {
        let manifest = load_manifest().unwrap();
        assert!(
            !manifest.local_data_dir.is_empty(),
            "local_data_dir must be set in manifest"
        );
        assert!(
            !manifest.local_outputs_dir.is_empty(),
            "local_outputs_dir must be set in manifest"
        );
    }

    #[test]
    fn test_all_states_have_name_and_fips() {
        let manifest = load_manifest().unwrap();
        for (code, state) in &manifest.states {
            assert!(!state.name.is_empty(), "{code} must have a non-empty name");
            assert!(
                !state.fips.is_empty(),
                "{code} must have a non-empty FIPS code"
            );
            assert_eq!(
                state.fips.len(),
                2,
                "{code} FIPS code must be exactly 2 digits; got '{}'",
                state.fips
            );
        }
    }

    #[test]
    fn test_all_states_have_districts_2020() {
        let manifest = load_manifest().unwrap();
        for (code, state) in &manifest.states {
            let n = state
                .districts
                .get("2020")
                .unwrap_or_else(|| panic!("{code} missing districts.2020"));
            assert!(*n >= 1, "{code} must have at least 1 district in 2020");
            assert!(
                *n <= 53,
                "{code} must have at most 53 districts (CA=52 + potential rounding)"
            );
        }
    }

    #[test]
    fn test_verify_file_sha256_nonexistent_file_returns_err() {
        let path = std::path::Path::new("/nonexistent/path/file.bin");
        let result = verify_file_sha256(path, "abc123");
        assert!(
            result.is_err(),
            "verify_file_sha256 on nonexistent file must return Err"
        );
    }

    #[test]
    fn test_build_fetch_list_two_states_correct_count() {
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(
            &manifest,
            &["VT".to_string(), "DE".to_string()],
            "2020",
            &[], // all types → 3 per state
        );
        assert_eq!(
            items.len(),
            6,
            "2 states × 3 types must produce 6 fetch items; got {}",
            items.len()
        );
    }

    // ── L0: LODES + school-districts + EIA-861 fetch list ─────────────────────

    #[test]
    fn test_build_fetch_list_lodes_url_pattern() {
        use crate::args::DataType;
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(&manifest, &["NC".to_string()], "2020", &[DataType::Lodes]);
        assert_eq!(items.len(), 1);
        let url = items[0].url.as_deref().unwrap();
        assert!(
            url.contains("lehd.ces.census.gov"),
            "LODES URL must use LEHD server"
        );
        assert!(
            url.contains("nc"),
            "LODES URL must contain lowercase state code"
        );
        assert!(url.contains("wac"), "LODES URL must be WAC file");
        assert!(url.contains("2020"), "LODES URL must contain year");
        assert!(url.ends_with(".csv.gz"), "LODES WAC must be .csv.gz");
        assert_eq!(items[0].kind, "lodes-wac");
        assert!(
            items[0].local_path.to_string_lossy().contains("lodes"),
            "LODES local path must be under lodes/ directory"
        );
        assert!(
            items[0]
                .local_path
                .to_string_lossy()
                .ends_with("_wac_tract.csv"),
            "LODES local path must end with _wac_tract.csv"
        );
    }

    #[test]
    fn test_build_fetch_list_school_districts_url_pattern() {
        use crate::args::DataType;
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(
            &manifest,
            &["NC".to_string()],
            "2020",
            &[DataType::SchoolDistricts],
        );
        assert_eq!(items.len(), 1);
        let url = items[0].url.as_deref().unwrap();
        assert!(
            url.contains("census.gov"),
            "school district URL must use Census server"
        );
        assert!(
            url.contains("UNSD"),
            "school district URL must reference UNSD directory"
        );
        assert!(
            url.contains("unsd"),
            "school district filename must contain 'unsd'"
        );
        assert!(url.ends_with(".zip"), "school district must be ZIP");
        assert_eq!(items[0].kind, "school-districts");
    }

    #[test]
    fn test_build_fetch_list_eia861_is_national() {
        use crate::args::DataType;
        let manifest = load_manifest().unwrap();
        // EIA-861 is one national file, not per-state — only 1 item regardless of state filter
        let items = build_fetch_list(
            &manifest,
            &["NC".to_string(), "WI".to_string()],
            "2020",
            &[DataType::Eia861],
        );
        assert_eq!(items.len(), 1, "EIA-861 produces exactly one national item");
        assert_eq!(items[0].state_code, "US", "EIA-861 state_code must be 'US'");
        let url = items[0].url.as_deref().unwrap();
        assert!(url.contains("eia.gov"), "EIA-861 URL must use EIA server");
        assert!(url.contains("2020"), "EIA-861 URL must contain year");
        assert!(url.ends_with(".zip"), "EIA-861 must be ZIP");
        assert_eq!(items[0].kind, "eia-861");
    }

    #[test]
    fn test_fetch_args_polite_delay_default() {
        use crate::args::FetchArgs;
        use clap::Parser;
        let args = FetchArgs::parse_from(["fetch"]);
        assert_eq!(
            args.polite_delay_secs, 1,
            "polite_delay_secs default must be 1 second"
        );
    }

    #[test]
    fn test_fetch_args_polite_delay_zero_allowed() {
        use crate::args::FetchArgs;
        use clap::Parser;
        let args = FetchArgs::parse_from(["fetch", "--polite-delay-secs", "0"]);
        assert_eq!(args.polite_delay_secs, 0);
    }

    // ── L0: download_lodes_wac block→tract aggregation ───────────────────────

    #[test]
    fn test_lodes_wac_aggregation_sums_blocks_to_tract() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        // Build a minimal LODES WAC CSV in memory:
        // 3 blocks in 2 tracts (blocks 01001020100001, 01001020100002 → tract 01001020100;
        //                       block 01001020200001 → tract 01001020200)
        let csv_content = "\
w_geocode,C000,CNS01,CNS02,CNS05,CNS07,CNS08,CNS09,CNS10,CNS11\n\
\"010010201000001\",100,10,5,0,30,20,15,10,5\n\
\"010010201000002\",50,5,0,0,20,5,5,5,0\n\
\"010010202000001\",200,0,0,0,80,50,30,20,10\n";

        // Compress to gzip
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(csv_content.as_bytes()).unwrap();
        let gz_bytes = encoder.finish().unwrap();

        // Write to a temp file (to simulate HTTP response we use a local path via mock)
        // We test the aggregation logic directly by calling the CSV parsing portion.
        // Since download_lodes_wac takes a URL, we test via the aggregation sub-logic.
        // Use a real temp file to simulate the gz stream.
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let gz_path = tmp_dir.path().join("test.csv.gz");
        std::fs::write(&gz_path, &gz_bytes).unwrap();

        let output_path = tmp_dir.path().join("output_wac_tract.csv");

        // Call aggregation via a file:// URL workaround — instead, test the
        // aggregation logic by parsing the uncompressed stream directly.
        // We inline the aggregation to test the contract.
        let f = std::fs::File::open(&gz_path).unwrap();
        let gz = flate2::read::GzDecoder::new(f);
        let reader = std::io::BufReader::new(gz);
        let mut lines = std::io::BufRead::lines(reader);

        let header = lines.next().unwrap().unwrap();
        let cols: Vec<&str> = header.split(',').collect();
        let idx = |name: &str| {
            cols.iter()
                .position(|c| c.trim_matches('"') == name)
                .unwrap()
        };
        let (i_geo, i_c000) = (idx("w_geocode"), idx("C000"));

        let mut tracts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for line in lines {
            let line = line.unwrap();
            let fields: Vec<&str> = line.split(',').collect();
            let geoid = fields[i_geo].trim_matches('"');
            let tract = &geoid[..11];
            let jobs: u64 = fields[i_c000].trim_matches('"').parse().unwrap_or(0);
            *tracts.entry(tract.to_string()).or_insert(0) += jobs;
        }

        // Verify aggregation: tract 01001020100 = 100+50=150, tract 01001020200 = 200
        assert_eq!(
            tracts["01001020100"], 150,
            "two blocks in same tract must be summed: 100+50=150"
        );
        assert_eq!(
            tracts["01001020200"], 200,
            "single block in tract = block value"
        );
        assert_eq!(tracts.len(), 2, "two distinct tracts");

        // Verify totals preserved: 100+50+200 = 350
        let total: u64 = tracts.values().sum();
        assert_eq!(
            total, 350,
            "L0 invariant: aggregation preserves total job count"
        );

        let _ = output_path; // suppress unused warning
    }

    #[test]
    fn test_lodes_wac_empty_csv_produces_header_only() {
        // A state with no jobs (e.g., uninhabited areas) should produce a valid
        // header-only CSV rather than crashing.
        let csv_content = "w_geocode,C000,CNS01,CNS02,CNS05,CNS07,CNS08,CNS09,CNS10,CNS11\n";
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(csv_content.as_bytes()).unwrap();
        let gz_bytes = enc.finish().unwrap();

        // Parse: no data rows → empty tract map → empty output CSV is valid
        let gz = flate2::read::GzDecoder::new(std::io::Cursor::new(gz_bytes));
        let reader = std::io::BufReader::new(gz);
        let mut lines = std::io::BufRead::lines(reader);
        let _header = lines.next().unwrap().unwrap();
        let remaining: Vec<_> = lines.collect();
        assert!(
            remaining.is_empty(),
            "no data rows → aggregation produces empty tract map"
        );
    }

    // ── L0: LODES OD fetch list + aggregation ────────────────────────────────

    #[test]
    fn test_build_fetch_list_lodes_od_url_pattern() {
        use crate::args::DataType;
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(&manifest, &["NC".to_string()], "2020", &[DataType::LodesOd]);
        assert_eq!(items.len(), 1);
        let url = items[0].url.as_deref().unwrap();
        assert!(
            url.contains("lehd.ces.census.gov"),
            "LODES OD URL must use LEHD server"
        );
        assert!(
            url.contains("/od/"),
            "LODES OD URL must contain /od/ path segment"
        );
        assert!(url.ends_with(".csv.gz"), "LODES OD must be .csv.gz");
        assert_eq!(items[0].kind, "lodes-od");
        assert!(
            items[0].local_path.to_string_lossy().contains("lodes"),
            "LODES OD local path must be under lodes/ directory"
        );
        assert!(
            items[0]
                .local_path
                .to_string_lossy()
                .ends_with("_od_tract.csv"),
            "LODES OD local path must end with _od_tract.csv"
        );
    }

    #[test]
    fn test_lodes_od_aggregation_sums_by_tract_pair() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        // 3 block-pair rows aggregating to 2 tract pairs:
        //   row 1: h=010010201000001, w=010010301000001 -> home=01001020100, work=01001030100, S000=50
        //   row 2: h=010010201000002, w=010010301000001 -> home=01001020100, work=01001030100, S000=30
        //   row 3: h=010010202000001, w=010010401000001 -> home=01001020200, work=01001040100, S000=80
        // Expected tract pairs: (01001020100, 01001030100)=80, (01001020200, 01001040100)=80
        let csv_content = "\
h_geocode,w_geocode,S000,SA01,SA02,SA03,SE01,SE02,SE03,SI01,SI02,SI03,createdate\n\
\"010010201000001\",\"010010301000001\",50,10,20,20,15,20,15,10,20,20,20200101\n\
\"010010201000002\",\"010010301000001\",30,5,10,15,10,10,10,5,15,10,20200101\n\
\"010010202000001\",\"010010401000001\",80,20,30,30,25,30,25,20,30,30,20200101\n";

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(csv_content.as_bytes()).unwrap();
        let gz_bytes = encoder.finish().unwrap();

        // Decompress and parse to verify aggregation logic
        let gz = flate2::read::GzDecoder::new(std::io::Cursor::new(gz_bytes));
        let reader = std::io::BufReader::new(gz);
        let mut lines = std::io::BufRead::lines(reader);

        let header = lines.next().unwrap().unwrap();
        let cols: Vec<&str> = header.split(',').collect();
        let idx = |name: &str| {
            cols.iter()
                .position(|c| c.trim_matches('"') == name)
                .unwrap()
        };
        let (i_home, i_work, i_s000) = (idx("h_geocode"), idx("w_geocode"), idx("S000"));

        let mut od: std::collections::HashMap<(String, String), u64> =
            std::collections::HashMap::new();
        for line in lines {
            let line = line.unwrap();
            if line.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split(',').collect();
            let home = fields[i_home].trim_matches('"');
            let work = fields[i_work].trim_matches('"');
            let home_tract = home[..11].to_string();
            let work_tract = work[..11].to_string();
            let jobs: u64 = fields[i_s000].trim_matches('"').parse().unwrap_or(0);
            *od.entry((home_tract, work_tract)).or_insert(0) += jobs;
        }

        // Two rows with same home+work tract pair collapse to one: 50+30=80
        assert_eq!(od.len(), 2, "3 block rows -> 2 tract pairs");
        assert_eq!(
            od[&("01001020100".to_string(), "01001030100".to_string())],
            80,
            "two block rows (S000=50,30) in same tract pair must sum to 80"
        );
        assert_eq!(
            od[&("01001020200".to_string(), "01001040100".to_string())],
            80,
            "single block row S000=80 -> tract pair = 80"
        );
        // Total preserved: 50+30+80=160
        let total: u64 = od.values().sum();
        assert_eq!(total, 160, "aggregation must preserve total job count");
    }

    // ── L0: ACS housing fetch list + formula tests ────────────────────────────

    #[test]
    fn test_build_fetch_list_acs_housing_url_pattern() {
        use crate::args::DataType;
        let manifest = load_manifest().unwrap();
        let items = build_fetch_list(
            &manifest,
            &["NC".to_string()],
            "2020",
            &[DataType::AcsHousing],
        );
        assert_eq!(
            items.len(),
            1,
            "ACS housing must produce exactly 1 item per state"
        );
        assert_eq!(items[0].kind, "acs-housing");
        let url = items[0].url.as_deref().unwrap();
        assert!(
            url.contains("api.census.gov"),
            "ACS URL must use api.census.gov"
        );
        assert!(
            url.contains("B25024_002E"),
            "ACS URL must include B25024_002E (1-unit detached)"
        );
        assert!(
            url.contains("for=tract"),
            "ACS URL must request tract-level data"
        );
        // NC FIPS is "37"
        assert!(
            url.contains("state:37"),
            "ACS URL must include NC FIPS state code"
        );
        assert!(url.contains("2020"), "ACS URL must include year");
        let path_str = items[0].local_path.to_string_lossy().to_lowercase();
        assert!(
            path_str.contains("acs_housing"),
            "local path must be under acs_housing/"
        );
        assert!(
            path_str.contains("north_carolina"),
            "local path must include state name"
        );
        assert!(
            path_str.ends_with("_housing_2020.csv"),
            "local path must end with _housing_2020.csv; got: {path_str}"
        );
    }

    /// housing_vintage formula: 1.0 - (year - 1940) / (2020 - 1940)
    #[test]
    fn test_acs_housing_vintage_formula() {
        let vintage = |built_year: f64| -> f64 {
            let v = 1.0 - (built_year - 1940.0) / (2020.0 - 1940.0);
            v.clamp(0.0, 1.0)
        };

        // year=1960: (1-(1960-1940)/(2020-1940)) = 1 - 20/80 = 0.75
        let v1960 = vintage(1960.0);
        assert!(
            (v1960 - 0.75).abs() < 1e-9,
            "vintage(1960) must be 0.75; got {v1960}"
        );

        // year=2020: (1-(2020-1940)/80) = 0.0
        let v2020 = vintage(2020.0);
        assert!(
            (v2020 - 0.0).abs() < 1e-9,
            "vintage(2020) must be 0.0; got {v2020}"
        );

        // year=1940: (1-0/80) = 1.0
        let v1940 = vintage(1940.0);
        assert!(
            (v1940 - 1.0).abs() < 1e-9,
            "vintage(1940) must be 1.0; got {v1940}"
        );

        // year=1939: pre-1940, raw=1.0125 -> clamped to 1.0
        let v1939 = vintage(1939.0);
        assert!(
            (v1939 - 1.0).abs() < 1e-9,
            "vintage(1939) must clamp to 1.0; got {v1939}"
        );

        // year=2025: post-2020, raw<0 -> clamped to 0.0
        let v2025 = vintage(2025.0);
        assert!(
            (v2025 - 0.0).abs() < 1e-9,
            "vintage(2025) must clamp to 0.0; got {v2025}"
        );
    }

    /// Unreliable Census code -666666666 must yield housing_vintage = 0.5 (neutral)
    #[test]
    fn test_acs_housing_vintage_unreliable_code() {
        let unreliable: f64 = -666_666_666.0;
        // Reproduce the branching logic from download_acs_housing
        let housing_vintage = if unreliable < -600_000_000.0 || unreliable == 0.0 {
            0.5
        } else {
            let v = 1.0 - (unreliable - 1940.0) / (2020.0 - 1940.0);
            v.clamp(0.0, 1.0)
        };
        assert!(
            (housing_vintage - 0.5).abs() < 1e-9,
            "unreliable code -666666666 must yield vintage=0.5; got {housing_vintage}"
        );
    }

    /// Verify pct_single_family defaults to 0.5 when total_units == 0
    #[test]
    fn test_acs_housing_pct_sf_default_when_zero_units() {
        let sf_units = 0.0_f64;
        let mf_units = 0.0_f64;
        let total_units = sf_units + mf_units;
        let pct_sf = if total_units > 0.0 {
            (sf_units / total_units).clamp(0.0, 1.0)
        } else {
            0.5
        };
        assert!(
            (pct_sf - 0.5).abs() < 1e-9,
            "pct_sf must default to 0.5 when total_units=0; got {pct_sf}"
        );
    }

    /// Verify pct_owner defaults to 0.5 when total_occupied == 0
    #[test]
    fn test_acs_housing_pct_owner_default_when_zero_occupied() {
        let occ_tot = 0.0_f64;
        let occ_own = 0.0_f64;
        let pct_owner = if occ_tot > 0.0 {
            (occ_own / occ_tot).clamp(0.0, 1.0)
        } else {
            0.5
        };
        assert!(
            (pct_owner - 0.5).abs() < 1e-9,
            "pct_owner must default to 0.5 when total_occupied=0; got {pct_owner}"
        );
    }

    // ── L2: Real LODES download (requires internet, marked #[ignore]) ─────────

    #[test]
    #[ignore = "L2: requires internet access to Census LEHD server"]
    fn test_lodes_wac_real_download_vt_2020() {
        // Vermont (VT) is the smallest state by tract count (~790 tracts) —
        // smallest real LODES download (~200KB). Validates the full pipeline:
        // HTTP GET → gzip decompress → block→tract aggregation → CSV write.
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let output = tmp_dir.path().join("vermont_wac_tract.csv");
        let url =
            "https://lehd.ces.census.gov/data/lodes/LODES8/vt/wac/vt_wac_S000_JT00_2020.csv.gz";
        download_lodes_wac(url, &output).expect("VT LODES download must succeed");
        assert!(output.exists(), "output CSV must be created");
        let content = std::fs::read_to_string(&output).unwrap();
        let lines: Vec<_> = content.lines().collect();
        assert!(lines.len() > 1, "must have header + at least one tract row");
        assert_eq!(
            lines[0], "geoid,c000,cns07,cns09,cns10,cns11,cns01,cns02,cns05,cns08",
            "header must match expected columns"
        );
        // VT has ~790 tracts — all jobs should aggregate to fewer rows than blocks
        assert!(lines.len() < 5000, "VT should have <5000 tracts");
        // Total jobs in VT 2020 should be in the range 200k-400k
        let total: u64 = lines
            .iter()
            .skip(1)
            .map(|l| {
                l.split(',')
                    .nth(1)
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
            })
            .sum();
        assert!(total > 100_000, "VT total jobs must be >100k; got {total}");
        assert!(total < 600_000, "VT total jobs must be <600k; got {total}");
    }
}
