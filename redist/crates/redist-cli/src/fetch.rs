/// Data fetch command: download census data needed to run redistricting.
///
/// Three data sources in priority order:
///   1. Local manifest override (~/.config/redist/manifest.json or REDIST_MANIFEST)
///      Points to already-present local files — no network needed.
///   2. GitHub Releases (--release flag) — pulls adjacency data from project releases.
///      Requires `gh auth login`.
///   3. Public Census Bureau URLs (default) — TIGER shapefiles and PL 94-171.
///
/// Incremental by default: checks for existing files before downloading.
/// Use --force to re-download even if present.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
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
    pub kind: String,         // "tiger", "pl94171", "adjacency"
    pub url: Option<String>,  // None = use github release or local only
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
    // Priority: REDIST_MANIFEST env > ~/.config/redist/manifest.json > builtin
    if let Ok(path) = std::env::var("REDIST_MANIFEST") {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("REDIST_MANIFEST={path}: {e}"))?;
        return serde_json::from_str(&content)
            .map_err(|e| format!("manifest parse error: {e}"));
    }

    // Check ~/.config/redist/manifest.json
    if let Some(home) = dirs_next_home() {
        let local = home.join(".config").join("redist").join("manifest.json");
        if local.exists() {
            let content = std::fs::read_to_string(&local)
                .map_err(|e| format!("local manifest read error: {e}"))?;
            return serde_json::from_str(&content)
                .map_err(|e| format!("local manifest parse error: {e}"));
        }
    }

    serde_json::from_str(BUILTIN_MANIFEST)
        .map_err(|e| format!("builtin manifest parse error: {e}"))
}

fn dirs_next_home() -> Option<PathBuf> {
    std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .ok().map(PathBuf::from)
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
    let want_pl = all_types || data_types.iter().any(|t| matches!(t, DataType::Redistricting));
    let want_adj = all_types || data_types.iter().any(|t| matches!(t, DataType::Adjacency));

    // Data types that exist as CLI flags but are not yet downloaded by `redist fetch`:
    // Elections, Enacted, Geography. Warn explicitly so users don't think the request
    // succeeded silently. The Python downloaders below remain the canonical source.
    let want_elections = !all_types && data_types.iter().any(|t| matches!(t, DataType::Elections));
    if want_elections {
        eprintln!(
            "WARNING: --type elections is not implemented in `redist fetch` yet. \
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
        states.iter()
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
                let local_path = data_dir.join(year).join("tiger").join("tracts")
                    .join(filename.replace(".zip", "")).join(filename.replace(".zip", ".shp"));
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
                let local_path = data_dir.join(year).join("redistricting")
                    .join(&state_lower).join(filename);
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
            let local_path = outputs_dir.join("V3").join("data").join(year)
                .join("adjacency").join(&adj_filename);
            let done_marker = local_path.with_extension("done");
            items.push(FetchItem {
                state_code: code.clone(),
                year: year.to_string(),
                kind: "adjacency".to_string(),
                url: None, // adjacency comes from GitHub Releases
                available_locally: local_path.exists(),
                local_path,
                done_marker,
            });
        }
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
        println!("{status} {} {} {} -> {}",
            item.state_code, item.year, item.kind,
            item.local_path.display());
        if status == "[NEED]" {
            println!("       src: {src}");
        }
    }
    println!();
    println!("Summary: {have} available, {need} need download");
    if need == 0 {
        println!("[OK] All data present. Ready to run: redist states --year 2020 --version V3");
    }
}

// ---------------------------------------------------------------------------
// Download
// ---------------------------------------------------------------------------

/// Download all items that aren't already present.
/// Uses native Rust (reqwest) for HTTP downloads. No Python subprocess.
pub fn download_items(
    items: &[FetchItem],
    force: bool,
    use_release: bool,
    manifest: &Manifest,
) -> Result<(), String> {
    for item in items {
        if !force && item.is_done() {
            println!("[SKIP] {} {} {} (already present)", item.state_code, item.year, item.kind);
            continue;
        }

        if let Some(parent) = item.local_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }

        match item.kind.as_str() {
            "adjacency" if use_release => {
                // GitHub Releases: use gh CLI (or REDIST_GH override for testing).
                // REDIST_GH may be "python /path/to/fake_gh.py" — split on first space.
                let gh_raw = std::env::var("REDIST_GH").unwrap_or_else(|_| "gh".to_string());
                let mut gh_parts = gh_raw.splitn(2, ' ');
                let gh_cmd = gh_parts.next().unwrap_or("gh");
                let gh_extra_args: Vec<&str> = gh_parts.next()
                    .map(|s| s.split_whitespace().collect())
                    .unwrap_or_default();

                let release_tag = &manifest.releases.data_inputs;
                let adj_dir = item.local_path.parent().unwrap();
                std::fs::create_dir_all(adj_dir).map_err(|e| e.to_string())?;
                println!("[DOWN] {} {} adjacency from release {release_tag}", item.state_code, item.year);
                let mut cmd = std::process::Command::new(gh_cmd);
                cmd.args(&gh_extra_args);
                let out = cmd
                    .args(["release", "download", release_tag,
                           "--pattern", &format!("{}_adjacency_{}.pkl", item.state_code.to_lowercase(), item.year),
                           "--dir", adj_dir.to_str().unwrap(),
                           "--repo", &manifest.github_repo,
                           "--clobber"])
                    .output()
                    .map_err(|e| format!("gh not found: {e}. Install: https://cli.github.com/"))?;
                if !out.status.success() {
                    return Err(format!(
                        "gh release download failed:\n{}",
                        String::from_utf8_lossy(&out.stderr)
                    ));
                }
            }

            "tiger" | "pl94171" => {
                // Pure Rust HTTP download + zip extraction (no Python)
                let url = item.url.as_deref().unwrap();
                let dest = item.local_path.parent().unwrap();
                println!("[DOWN] {} {} {} <- {}", item.state_code, item.year, item.kind,
                    url.split('/').last().unwrap_or(url));

                download_and_extract_zip(url, dest)?;
            }

            _ => {
                println!("[SKIP] {} {} {} (use --release to download from GitHub)",
                    item.state_code, item.year, item.kind);
                continue;
            }
        }

        std::fs::write(&item.done_marker, b"done")
            .map_err(|e| format!("done marker write failed: {e}"))?;
    }
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

    let actual = redist_report::sha256_file(path)
        .map_err(|e| format!("SHA-256 computation failed for {}: {e}", path.display()))?;

    if actual != expected.to_lowercase() {
        // Delete the corrupt file so it won't be reused
        let _ = std::fs::remove_file(path);
        return Err(format!(
            "SHA-256 mismatch for {}:\n  expected: {}\n  actual:   {}\n\
             Corrupt file deleted. Re-run to re-download.",
            path.display(), expected, actual
        ));
    }
    Ok(())
}

/// Download a ZIP from url and extract it to dest_dir.
/// Streams response to a temp file to avoid OOM for large ZIPs (Critical 3).
/// California PL 94-171 can be 80MB+ — in-memory loading would OOM on constrained systems.
pub fn download_and_extract_zip(url: &str, dest_dir: &Path) -> Result<(), String> {
    let mut response = reqwest::blocking::get(url)
        .map_err(|e| format!("HTTP GET {url}: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}: {url}", response.status()));
    }

    // Stream to temp file — avoids loading large ZIPs (80MB+ for CA PL 94-171) into RAM
    let tmp_dir = tempfile::TempDir::new().map_err(|e| e.to_string())?;
    let tmp_zip = tmp_dir.path().join("download.zip");
    {
        let mut out = std::fs::File::create(&tmp_zip)
            .map_err(|e| format!("tmp file create: {e}"))?;
        std::io::copy(&mut response, &mut out)
            .map_err(|e| format!("streaming download: {e}"))?;
    }

    // Extract from temp file
    let zip_file = std::fs::File::open(&tmp_zip).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("invalid ZIP from {url}: {e}"))?;

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
        let vt = manifest.states.get("VT").expect("Vermont must be in manifest");
        assert_eq!(vt.fips, "50", "Vermont FIPS is 50");
        assert!(vt.tiger["2020"].contains("tl_2020_50_tract"), "VT TIGER URL");
        assert!(vt.pl94171["2020"].contains("vermont"), "VT PL URL");
    }

    #[test]
    fn test_manifest_has_alabama() {
        let manifest = load_manifest().unwrap();
        let al = manifest.states.get("AL").expect("Alabama must be in manifest");
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
        let items = build_fetch_list(
            &manifest, &["VT".to_string()], "2020", &[DataType::Tiger]
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, "tiger");
        assert!(items[0].url.as_deref().unwrap().contains("tl_2020_50_tract"));
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
            let url = state.tiger.get("2020")
                .unwrap_or_else(|| panic!("{code} missing tiger 2020 URL"));
            assert!(url.starts_with("https://"), "{code} tiger URL must be https");
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
        assert!(result.is_ok(), "correct hash should pass: {:?}", result.err());
        // File must still exist after a passing check
        assert!(path.exists(), "file must still exist after passing verification");
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
        assert!(msg.contains("mismatch") || msg.contains("SHA-256"),
            "error must mention mismatch: {msg}");
        // File must be deleted after mismatch
        assert!(!path.exists(), "corrupt file must be deleted after mismatch");
    }

    #[test]
    fn test_verify_file_sha256_empty_expected_skips() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("data.bin");
        std::fs::write(&path, b"any content").unwrap();
        // Empty expected = no check performed
        let result = verify_file_sha256(&path, "");
        assert!(result.is_ok(), "empty expected hash must skip check and return Ok");
        assert!(path.exists(), "file must not be deleted when check is skipped");
    }

    #[test]
    fn test_verify_downloads_flag_parses() {
        use crate::args::FetchArgs;
        use clap::Parser;
        let args = FetchArgs::parse_from([
            "fetch",
            "--verify-downloads",
        ]);
        assert!(args.verify_downloads, "--verify-downloads flag must parse to true");
    }

    #[test]
    fn test_verify_downloads_default_false() {
        use crate::args::FetchArgs;
        use clap::Parser;
        let args = FetchArgs::parse_from(["fetch"]);
        assert!(!args.verify_downloads, "--verify-downloads must default to false");
    }

    #[test]
    fn test_all_50_states_have_pl94171_2020_url() {
        let manifest = load_manifest().unwrap();
        for (code, state) in &manifest.states {
            let url = state.pl94171.get("2020")
                .unwrap_or_else(|| panic!("{code} missing pl94171 2020 URL"));
            assert!(url.contains("census.gov"), "{code} PL URL must be census.gov");
        }
    }
}
