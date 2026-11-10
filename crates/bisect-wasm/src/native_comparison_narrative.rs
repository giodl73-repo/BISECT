//! Portable delivery of the native Compare narrative artifact set.
use std::collections::{BTreeMap, HashMap};

use bisect_analysis::comparison::{format_comparison_table, parse_direct_assignment_object, try_compare_plans};
use bisect_report::comparison::{diff_from_assignment_files, load_plan_side_from_files, ComparisonReport};
use bisect_report::comparison_delivery::{render_native_comparison, sha256_hex, ComparisonProducer};
use bisect_report::narrative::NarrativeConfig;
use serde::Deserialize;
use serde_json::{json, Value};

const MAX_SIDE_BYTES: usize = 4 * 1024 * 1024;
const MAX_ASSIGNMENTS: usize = 100_000;
const MANIFEST: &str = "manifest.json";
const PRIMARY: &str = "data/final_assignments.json";
const FALLBACK: &str = "final_assignments.json";
const ANALYSES: [&str; 3] = ["analysis/partisan.json", "analysis/vra_analysis.json", "analysis/compactness.json"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    plan_a: Folder,
    plan_b: Folder,
    format: Format,
    leaning_threshold: f64,
    close_call_band: f64,
    source_date_epoch: Option<String>,
    allow_cross_year: bool,
    signoff: Option<Signoff>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Folder {
    name: String,
    files: BTreeMap<String, String>,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Format { Narrative, Both, Html }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Signoff { name: String, confirmed_for_this_run: bool }

struct Parsed {
    side: bisect_report::comparison::PlanSide,
    files: BTreeMap<String, Vec<u8>>,
    assignments: HashMap<String, u64>,
    assignment_path: &'static str,
    year: Option<String>,
}

fn plain(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 200
        && !value.chars().any(|c| c.is_control() || matches!(c, '<' | '>' | '[' | ']' | '`' | '\\'))
}

fn read(folder: &Folder, threshold: f64) -> Result<Parsed, String> {
    if !plain(&folder.name) || folder.name.contains('/') || matches!(folder.name.as_str(), "." | "..") {
        return Err("Use a plain native plan folder name.".into());
    }
    if folder.files.len() > 6 || folder.files.values().map(String::len).sum::<usize>() > MAX_SIDE_BYTES
        || folder.files.keys().any(|p| ![MANIFEST, PRIMARY, FALLBACK].contains(&p.as_str()) && !ANALYSES.contains(&p.as_str())) {
        return Err("Select only bounded native comparison files from each folder.".into());
    }
    let manifest = folder.files.get(MANIFEST).ok_or("Select a plan folder containing manifest.json.")?;
    let meta: Value = serde_json::from_str(manifest).map_err(|e| format!("Invalid comparison manifest: {e}"))?;
    let year = match meta.get("year") {
        None | Some(Value::Null) => None,
        Some(Value::String(y)) if y.len() <= 20 && y.chars().all(|c| c.is_ascii_digit()) && !y.is_empty() => Some(y.clone()),
        _ => return Err("The supplied manifest Census year must be a short decimal string.".into()),
    };
    let assignment_path = if folder.files.contains_key(PRIMARY) { PRIMARY } else { FALLBACK };
    let text = folder.files.get(assignment_path).ok_or("Select a native plan folder containing final_assignments.json.")?;
    let value: Value = serde_json::from_str(text).map_err(|e| format!("Invalid comparison assignments: {e}"))?;
    let object = value.as_object().ok_or("Native comparison assignments must be a JSON object.")?;
    if object.len() > MAX_ASSIGNMENTS {
        return Err("Native comparison assignments exceed the portable unit limit.".into());
    }
    let assignments = parse_direct_assignment_object::<u64>(&folder.name, object)?;
    let files = folder.files.iter().map(|(p, s)| (p.clone(), s.as_bytes().to_vec())).collect();
    let side = load_plan_side_from_files(&files, &folder.name, threshold).map_err(|e| e.to_string())?;
    if side.n_districts > 500 || side.per_district_dem_share.len() > 500 {
        return Err("Native comparison report exceeds the portable district limit.".into());
    }
    if !plain(&side.label) || side.submitted_by.as_deref().is_some_and(|x| !plain(x))
        || side.submitted_at.as_deref().is_some_and(|x| !plain(x)) {
        return Err("Use plain report labels and attribution text without Markdown or control characters.".into());
    }
    Ok(Parsed { side, files, assignments, assignment_path, year })
}

pub(crate) fn run(input: Input) -> Result<Value, String> {
    if !(0.0..=1.0).contains(&input.leaning_threshold) || !(0.0..=1.0).contains(&input.close_call_band) {
        return Err("Report threshold and close-call band must be fractions between zero and one.".into());
    }
    let approved_by = match input.signoff {
        None => None,
        Some(Signoff { name, confirmed_for_this_run: true }) if plain(&name) => Some(name),
        _ => return Err("Enter a plain sign-off name and confirm it for this run.".into()),
    };
    let epoch = input.source_date_epoch.as_deref().map(|text| {
        text.parse::<i64>().ok().filter(|value| value.to_string() == text)
            .ok_or("Use a canonical signed integer for the optional source date epoch.")
    }).transpose()?;
    let a = read(&input.plan_a, input.leaning_threshold)?;
    let b = read(&input.plan_b, input.leaning_threshold)?;
    let cross_year = matches!((&a.year, &b.year), (Some(a), Some(b)) if a != b);
    if cross_year && !input.allow_cross_year {
        return Err("These manifests have different Census years. Confirm a literal comparison to proceed; it does not align geography.".into());
    }
    let diff = diff_from_assignment_files(&a.files, &input.plan_a.name, &b.files, &input.plan_b.name)
        .map_err(|e| e.to_string())?;
    let source_hashes = |files: &BTreeMap<String, Vec<u8>>, assignment| {
        files.iter().filter(|(name, _)| ![PRIMARY, FALLBACK].contains(&name.as_str()) || name.as_str() == assignment)
            .map(|(name, bytes)| (name.clone(), sha256_hex(bytes))).collect::<BTreeMap<_,_>>()
    };
    let a_hashes = source_hashes(&a.files, a.assignment_path);
    let b_hashes = source_hashes(&b.files, b.assignment_path);
    let table = if input.format == Format::Both {
        use crate::native_assignment_compare::{comparison_ids, ranks};
        let a_rank = ranks(&a.assignments)?;
        let b_rank = ranks(&b.assignments)?;
        let mut comparison = try_compare_plans(&comparison_ids(a.assignments, &a_rank), &comparison_ids(b.assignments, &b_rank))
            .map_err(|e| e.to_string())?;
        comparison.plan_a.label = format!("./{}", input.plan_a.name);
        comparison.plan_b.label = format!("./{}", input.plan_b.name);
        Some(format_comparison_table(&comparison))
    } else { None };
    let diff_evidence = json!({"changed_units":diff.tracts_changed,"district_ids":diff.districts_with_changes.iter().map(u64::to_string).collect::<Vec<_>>()});
    let report = ComparisonReport::from_loaded(a.side, b.side, None, diff);
    let cfg = NarrativeConfig { leaning_threshold: input.leaning_threshold, close_call_band: input.close_call_band,
        approved_by, ..Default::default() };
    let producer = ComparisonProducer {version:env!("CARGO_PKG_VERSION").to_string(),build_commit:String::new(),rustc_version:String::new()};
    let result = render_native_comparison(&report, &cfg, producer, epoch, input.format == Format::Html)
        .map_err(|e| e.to_string())?;
    let mut files = BTreeMap::from([("narrative.md",result.markdown),("narrative_manifest.json",result.manifest_json)]);
    if let Some(html) = result.html { files.insert("comparison.html",html); }
    if let Some(table) = table { files.insert("comparison.txt",table); }
    let hashes = files.iter().map(|(&name, text)| (name, sha256_hex(text.as_bytes()))).collect::<BTreeMap<_,_>>();
    Ok(json!({"schema_version":"bisect-native-comparison-narrative-v1","files":files,"file_sha256":hashes,
        "source_sha256":{"plan_a":a_hashes,"plan_b":b_hashes},"manifest_years":[a.year,b.year],
        "literal_cross_year":cross_year,"assignment_difference":diff_evidence,"named_signoff":cfg.approved_by.is_some(),
        "scope":"Native comparison of selected files. Sign-off is a supplied name, not authenticated identity or certification. Population change is unavailable from assignments alone. Build fields do not identify a desktop executable."}))
}
