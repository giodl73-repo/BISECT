//! Native Fetch manifest and source planning with caller-supplied storage observations.
use serde::{Deserialize,Serialize};
use std::collections::HashMap;
use std::path::{Path,PathBuf};
pub const DONE_MARKER_BYTES:&[u8]=b"done";
pub type Manifest = ManifestWithDistricts<usize>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ManifestWithDistricts<D> {
    pub version: String,
    pub github_repo: String,
    pub releases: Releases,
    pub local_data_dir: String,
    pub local_outputs_dir: String,
    pub states: HashMap<String, StateManifestWithDistricts<D>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Releases {
    pub data_inputs: String,
    pub outputs_v3: String,
    pub outputs_v4: String,
}

pub type StateManifest = StateManifestWithDistricts<usize>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StateManifestWithDistricts<D> {
    pub name: String,
    pub fips: String,
    pub districts: HashMap<String, D>,
    pub tiger: HashMap<String, String>,
    pub pl94171: HashMap<String, String>,
}

#[derive(Debug,Clone,Serialize)]
pub struct FetchItem {pub state_code:String,pub year:String,pub kind:String,pub url:Option<String>,pub local_path:PathBuf,pub done_marker:PathBuf,pub available_locally:bool}
#[derive(Debug,Clone,Deserialize,Serialize)]
#[serde(rename_all="kebab-case")]
pub enum DataType {Tiger,Redistricting,Adjacency,Enacted,Geography,Elections,Lodes,SchoolDistricts,#[serde(rename="eia-861")]Eia861,LodesOd,AcsHousing,All}
pub fn build_fetch_list(
    manifest: &Manifest,
    states: &[String],
    year: &str,
    data_types: &[DataType],
    mut exists:impl FnMut(&Path)->bool,
    mut warn:impl FnMut(&str),
) -> Vec<FetchItem> {
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
        warn(&format!(
            "WARNING: --type elections is not implemented in `bisect fetch` yet. \
             Use the Python downloader instead:\n  \
             python scripts/data/elections/download_election_data.py --year {year}"
        ));
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
                    available_locally: exists(&local_path),
                    local_path,
                    done_marker,
                });
            }
        }

        // PL 94-171 redistricting file
        if want_pl {
            if let Some(manifest_url) = state.pl94171.get(year) {
                let url = pl94171_url(code, &state.name, year, manifest_url);
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
                    url: Some(url),
                    available_locally: exists(&local_path),
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
                available_locally: exists(&local_path),
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
                available_locally: exists(&local_path),
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
                available_locally: exists(&local_path),
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
                    available_locally: exists(&local_path),
                    local_path,
                    done_marker,
                });
            }
        }

        // ACS 5-year housing character — one JSON per state per year from Census ACS API.
        // Tables: B25024 (units in structure), B25003 (tenure), B25035 (median year built).
        // Output: data/{year}/acs_housing/{state_lower}_housing_{year}.csv
        // Census data queries require a key as of May 2026; acquisition hosts
        // supply it separately. Source plans do not retain credentials.
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
                available_locally: exists(&local_path),
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
            available_locally: exists(&local_path),
            local_path,
            done_marker,
        });
    }

    items
}

pub fn pl94171_url(code: &str, state_name: &str, year: &str, manifest_url: &str) -> String {
    if year == "2020" {
        let census_dir = state_name.replace(' ', "_");
        return format!(
            "https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/{census_dir}/{}2020.pl.zip",
            code.to_lowercase()
        );
    }
    manifest_url.to_string()
}

