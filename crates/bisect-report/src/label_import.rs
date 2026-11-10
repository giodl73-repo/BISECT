//! Shared native external label assignment import kernels.
use sha2::{Digest, Sha256};
use shapefile::dbase::{FieldValue, Record};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
/// Align external native labels with an independently supplied, unchanged context.
/// This records conversion provenance, never engine generation or certification.
pub fn align_context(assignments: &BTreeMap<String,u64>, context: &rplan_core::RplanContext,
    state_name: &str, label: &str, year: &str, created: &str, basis: &str,
    bindings: serde_json::Value) -> Result<rplan_io::RplanDocument,String> {
    use rplan_core::*;
    use rplan_io::*;
    context.validate().map_err(|e|e.to_string())?;
    if context.context_hash != context.compute_context_hash().map_err(|e|e.to_string())?
        || context.units.unit_universe_hash != context.units.compute_unit_universe_hash().map_err(|e|e.to_string())? {
        return Err("Context or unit universe hash does not match supplied content.".into());
    }
    let policy:serde_json::Value=serde_json::from_str(include_str!("../../../data/location_policy.json")).map_err(|e|e.to_string())?;
    let locations=bisect_analysis::location_registry::PolicyRegistry::new(&policy);
    let state=context.units.state.as_deref().ok_or("Context state is required.")?;
    let resolved=locations.state_name(state).ok_or("Unknown context state code.")?.to_lowercase().replace(' ',"_");
    if resolved!=state_name || context.units.year.ok_or("Context year is required.")?.to_string()!=year {
        return Err("Imported state and year must match the supplied context.".into());
    }
    if assignments.is_empty() || assignments.len()!=context.units.unit_ids.len()
        || assignments.len()>100_000 {
        return Err("Assignments must cover exactly the context, up to 100,000 units.".into());
    }
    let offset=match basis {"one-based"=>1,"zero-based"=>0,_=>return Err("Choose one-based or zero-based district labels.".into())};
    let assignment=context.units.unit_ids.iter().map(|id| {
        assignments.get(id).ok_or("Imported assignments do not match the exact context unit set.")?
            .checked_sub(offset).filter(|d|*d<500).map(|d|d as u32)
            .ok_or("District labels must form a complete range of at most 500 districts in the selected basis.")
    }).collect::<Result<Vec<_>,_>>()?;
    let k=assignment.iter().max().copied().unwrap() as usize+1;
    let plan=DistrictPlan {schema_version:DISTRICT_PLAN_SCHEMA_VERSION.into(),units:context.units.clone(),assignment,k,
        display_labels:(0..k).map(|d|(d as u64+offset).to_string()).collect(),allow_empty_districts:false};
    plan.validate().map_err(|e|e.to_string())?;
    let mut hashes=context.source_hashes.entries.clone();
    // Namespaced conversion identities cannot replace a supplied context identity.
    for (key,value) in bindings.as_object().ok_or("Import bindings must be an object.")? {
        if key.ends_with("_sha256") {
            let digest=value.as_str().ok_or("Import binding digest must be text.")?;
            if digest.len()!=64 || !digest.bytes().all(|b|b.is_ascii_hexdigit()) {return Err("Invalid import binding digest.".into());}
            let name=format!("bisect.label-import.{key}");
            if hashes.insert(name,digest.into()).is_some() {return Err("Context uses a reserved import provenance key.".into());}
        }
    }
    let document=RplanDocument {rplan_version:RPLAN_V02.into(),plan,
        metadata:RplanMetadataV02 {label:label.into(),jurisdiction:state.into(),chamber:"imported".into(),created_at:created.into(),description:Some("External assignments aligned to a supplied context; generation, source truth and certification are unverified.".into())},
        provenance:RplanProvenance {producer:BTreeMap::from([("method".into(),serde_json::json!("external-label-import-context-alignment"))]),source_hashes:hashes,conversion_lineage:vec![serde_json::json!({"operation":"external-label-import-context-alignment","district_basis":basis})]},geometry:None,
        extensions:BTreeMap::from([("org.bisect.label-import".into(),bindings)])};
    read_rplan_str(&write_rplan_string(&document).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    Ok(document)
}
#[cfg(test)]
mod context_tests {
    use super::*;
    #[test]
    fn all_native_unit_kinds_preserve_context_order_and_reject_missing_districts() {
        use rplan_core::UnitKind;
        for kind in [UnitKind::Block,UnitKind::BlockGroup,UnitKind::Tract,UnitKind::County,UnitKind::Precinct,UnitKind::Imported] {
            let mut context=rplan_io::read_rctx_str(include_str!("../../rplan-audit/fixtures/grid3x3.rctx")).unwrap();
            context.units.unit_kind=kind;
            context.units.unit_ids=(1..=9).rev().map(|i|match kind {
                UnitKind::Block=>format!("53001000100{i:04}"),UnitKind::BlockGroup=>format!("53001000{i:04}"),
                UnitKind::Tract=>format!("5300100{i:04}"),UnitKind::County=>format!("53{i:03}"),
                _=>format!("53:unit-{i}")}).collect();
            context.units.unit_universe_hash=context.units.compute_unit_universe_hash().unwrap();
            context.context_hash=context.compute_context_hash().unwrap();
            let values=context.units.unit_ids.iter().enumerate().map(|(i,g)|(g.clone(),(i/3+1) as u64)).collect();
            let plan=align_context(&values,&context,"washington","external","2020","recorded","one-based",serde_json::json!({})).unwrap();
            assert_eq!(plan.plan.units,context.units);
            assert_eq!(plan.plan.assignment,vec![0,0,0,1,1,1,2,2,2]);
            assert_eq!(plan.plan.k,3);
            assert!(!plan.provenance.producer.contains_key("engine_options"));
            let zero=values.iter().map(|(g,d)|(g.clone(),d-1)).collect();
            assert_eq!(align_context(&zero,&context,"washington","external","2020","recorded","zero-based",serde_json::json!({})).unwrap().plan.assignment,plan.plan.assignment);
            assert!(align_context(&values,&context,"washington","external","2020","recorded","zero-based",serde_json::json!({})).is_err());
            let mut missing=values.clone();missing.remove(&context.units.unit_ids[0]);
            assert!(align_context(&missing,&context,"washington","external","2020","recorded","one-based",serde_json::json!({})).is_err());
            let mut replaced=values.clone();let value=replaced.remove(&context.units.unit_ids[0]).unwrap();replaced.insert("53:extra-unit".into(),value);
            assert!(align_context(&replaced,&context,"washington","external","2020","recorded","one-based",serde_json::json!({})).is_err());
            assert!(align_context(&values,&context,"vermont","external","2020","recorded","one-based",serde_json::json!({})).is_err());
            assert!(align_context(&values,&context,"washington","external","2010","recorded","one-based",serde_json::json!({})).is_err());
            let holes=values.iter().map(|(g,d)|(g.clone(),if *d==2 {3}else{*d})).collect();
            assert!(align_context(&holes,&context,"washington","external","2020","recorded","one-based",serde_json::json!({})).is_err());
        }
    }
}
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
        _ => None,
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
        Some("csv") => Some("csv"),
        Some("geojson") | Some("json") => Some("geojson"),
        Some("shp") => Some("shapefile"),
        Some("rplan") => Some("rplan"),
        _ => None,
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
pub fn parse_csv_assignments(csv_str: &str) -> Result<HashMap<String, u64>, String> {
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
        let h0 = headers
            .first()
            .map(|h| h.trim().to_uppercase())
            .unwrap_or_default();
        let h1 = headers
            .get(1)
            .map(|h| h.trim().to_uppercase())
            .unwrap_or_default();
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

    let mut assignments: HashMap<String, u64> = HashMap::new();
    for (lineno, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        let geoid_raw = fields.get(geoid_col).map(|f| f.trim()).unwrap_or("");
        let dist_raw = fields.get(district_col).map(|f| f.trim()).unwrap_or("");

        // Auto-swap: if geoid_raw is small (<5 chars) and dist_raw looks like a GEOID
        let (geoid, dist_str) = if geoid_raw.len() < 5
            && dist_raw.len() >= 10
            && dist_raw.chars().all(|c| c.is_ascii_digit())
        {
            (dist_raw, geoid_raw)
        } else {
            (geoid_raw, dist_raw)
        };

        let district: u64 = dist_str.parse().map_err(|_| {
            format!(
                "[INPUT] CSV line {}: cannot parse district '{}' as integer",
                lineno + 1,
                dist_str
            )
        })?;
        assignments.insert(geoid.to_string(), district);
    }

    if assignments.is_empty() {
        return Err(
            "[INPUT] CSV produced no GEOID→district assignments — check file format".to_string(),
        );
    }
    Ok(assignments)
}

// ── RPLAN parsing ─────────────────────────────────────────────────────────────

/// Parse an RPLAN document into a `GEOID -> district` assignment map.
///
/// RPLAN stores canonical district ids as zero-based integers.  BISECT run
/// assignment files use one-based numeric district ids, so import normalizes
/// `0..k` to `1..=k`.
pub fn parse_rplan_assignments(rplan_str: &str) -> Result<HashMap<String, u64>, String> {
    let document =
        rplan_io::read_rplan_str(rplan_str).map_err(|e| format!("[INPUT] invalid RPLAN: {e}"))?;
    let units = &document.plan.units.unit_ids;
    let assignment = &document.plan.assignment;
    if units.len() != assignment.len() {
        return Err(format!(
            "[INPUT] RPLAN assignment length {} does not match unit count {}",
            assignment.len(),
            units.len()
        ));
    }

    let mut assignments = HashMap::with_capacity(units.len());
    for (unit_id, district_id) in units.iter().zip(assignment.iter()) {
        assignments.insert(unit_id.clone(), (*district_id as u64) + 1);
    }

    if assignments.is_empty() {
        return Err("[INPUT] RPLAN contains no unit assignments".to_string());
    }
    Ok(assignments)
}

// ── GeoJSON parsing ───────────────────────────────────────────────────────────

/// Parse a GeoJSON FeatureCollection into a `GEOID → district` assignment map.
///
/// Each feature must have a `district_id` (or `district`) property and a
/// `GEOID` (or `geoid`) property in its `properties` object.
pub fn parse_geojson_assignments(geojson_str: &str) -> Result<HashMap<String, u64>, String> {
    let v: serde_json::Value =
        serde_json::from_str(geojson_str).map_err(|e| format!("[INPUT] invalid JSON: {e}"))?;

    let features = v["features"]
        .as_array()
        .ok_or_else(|| "[INPUT] GeoJSON missing 'features' array".to_string())?;

    let mut assignments: HashMap<String, u64> = HashMap::new();
    for feature in features {
        let props = &feature["properties"];

        // Resolve GEOID: try "GEOID" then "geoid"
        let geoid = props["GEOID"].as_str().or_else(|| props["geoid"].as_str());

        // Resolve district: try "district_id" then "district"
        let district = props["district_id"]
            .as_u64()
            .or_else(|| props["district"].as_u64());

        if let (Some(g), Some(d)) = (geoid, district) {
            assignments.insert(g.to_string(), d);
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

// ── Shapefile parsing ─────────────────────────────────────────────────────────

const GEOID_FIELD_CANDIDATES: &[&str] = &["GEOID", "GEOID20", "GEOID10", "GEOID00", "GEO_ID"];
const DISTRICT_FIELD_CANDIDATES: &[&str] = &[
    "DISTRICT",
    "DISTRICTID",
    "DISTRICT_I",
    "DISTRICT_",
    "DIST_ID",
    "DIST",
    "DIST_NUM",
    "CD",
];

/// Parse shapefile DBF attributes into a `GEOID -> district` assignment map.
///
/// The importer reads attributes from the `.dbf` sidecar through the shapefile
/// reader and ignores geometry after the record can be read. Field names are
/// detected case-insensitively from common plan-export names.
#[cfg(feature = "filesystem")]
pub fn parse_shapefile_assignments(path: &Path) -> Result<HashMap<String, u64>, String> {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("shp"))
        .unwrap_or(false)
    {
        let dbf_path = path.with_extension("dbf");
        if !dbf_path.exists() {
            return Err(format!(
                "[INPUT] shapefile sidecar DBF is missing: '{}'",
                dbf_path.display()
            ));
        }
    }

    let reader = shapefile::Reader::from_path(path)
        .map_err(|e| format!("[INPUT] cannot read shapefile '{}': {e}", path.display()))?;
    parse_shapefile_reader(reader, &path.display().to_string())
}

pub fn parse_shapefile_reader<
    S: std::io::Read + std::io::Seek,
    D: std::io::Read + std::io::Seek,
>(
    mut reader: shapefile::Reader<S, D>,
    source_name: &str,
) -> Result<HashMap<String, u64>, String> {
    let mut assignments = HashMap::new();
    let mut geoid_field: Option<String> = None;
    let mut district_field: Option<String> = None;

    for (idx, shape_record) in reader.iter_shapes_and_records().enumerate() {
        let (_shape, record) = shape_record.map_err(|e| {
            format!(
                "[INPUT] cannot read shapefile record {} from '{}': {e}",
                idx + 1,
                source_name
            )
        })?;

        if geoid_field.is_none() {
            geoid_field = detect_dbf_field(&record, GEOID_FIELD_CANDIDATES);
            district_field = detect_dbf_field(&record, DISTRICT_FIELD_CANDIDATES);
            if geoid_field.is_none() || district_field.is_none() {
                let fields = available_dbf_fields(&record);
                return Err(format!(
                    "[INPUT] shapefile attributes must include GEOID and district fields. \
                     GEOID candidates: {}; district candidates: {}; available fields: {}",
                    GEOID_FIELD_CANDIDATES.join(", "),
                    DISTRICT_FIELD_CANDIDATES.join(", "),
                    fields
                ));
            }
        }

        let geoid_name = geoid_field.as_deref().unwrap();
        let district_name = district_field.as_deref().unwrap();
        let geoid = parse_geoid_value(record.get(geoid_name)).ok_or_else(|| {
            format!(
                "[INPUT] shapefile record {} has empty or invalid GEOID field '{}'",
                idx + 1,
                geoid_name
            )
        })?;
        let district = parse_district_value(record.get(district_name)).ok_or_else(|| {
            format!(
                "[INPUT] shapefile record {} has empty or non-integer district field '{}'",
                idx + 1,
                district_name
            )
        })?;

        assignments.insert(geoid, district);
    }

    if assignments.is_empty() {
        return Err(
            "[INPUT] shapefile produced no GEOID->district assignments - check file attributes"
                .to_string(),
        );
    }

    Ok(assignments)
}

fn detect_dbf_field(record: &Record, candidates: &[&str]) -> Option<String> {
    let fields = record.as_ref();
    candidates.iter().find_map(|candidate| {
        fields
            .keys()
            .find(|field| field.eq_ignore_ascii_case(candidate))
            .cloned()
    })
}

fn available_dbf_fields(record: &Record) -> String {
    let mut fields: Vec<&str> = record.as_ref().keys().map(String::as_str).collect();
    fields.sort_unstable();
    if fields.is_empty() {
        "<none>".to_string()
    } else {
        fields.join(", ")
    }
}

fn parse_geoid_value(value: Option<&FieldValue>) -> Option<String> {
    match value {
        Some(FieldValue::Character(Some(s))) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Some(FieldValue::Numeric(Some(v))) if v.fract() == 0.0 => Some(format!("{v:.0}")),
        Some(FieldValue::Integer(v)) => Some(v.to_string()),
        _ => None,
    }
}

fn parse_district_value(value: Option<&FieldValue>) -> Option<u64> {
    match value {
        Some(FieldValue::Character(Some(s))) => s.trim().parse::<u64>().ok(),
        Some(FieldValue::Numeric(Some(v))) if *v >= 0.0 && v.fract() == 0.0 => Some(*v as u64),
        Some(FieldValue::Integer(v)) if *v >= 0 => Some(*v as u64),
        Some(FieldValue::Double(v)) if *v >= 0.0 && v.fract() == 0.0 => Some(*v as u64),
        _ => None,
    }
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

pub type StateAssignments = BTreeMap<String, BTreeMap<String, u64>>;
pub fn group(assignments: HashMap<String, u64>) -> Result<(StateAssignments, Vec<String>), String> {
    let mut by_state: StateAssignments = Default::default();
    let mut unknown = Vec::new();
    for (geoid, district) in assignments {
        if let Some(state) = geoid.get(..2).and_then(fips_to_state_name) {
            by_state
                .entry(state.into())
                .or_default()
                .insert(geoid, district);
        } else {
            unknown.push(geoid);
        }
    }
    unknown.sort();
    if !unknown.is_empty() && by_state.is_empty() {
        return Err(format!("[INPUT] no GEOIDs could be mapped to a known state FIPS prefix. First unrecognised GEOID: '{}'. Ensure GEOIDs start with a valid 2-digit US state FIPS code.",unknown[0]));
    }
    Ok((by_state, unknown))
}
pub fn build_index(
    label: &str,
    year: &str,
    source: &str,
    format: &str,
    sha: &str,
    by_state: &StateAssignments,
    created: &str,
) -> serde_json::Value {
    serde_json::json!({"label":label,"year":year,"created":created,"version":env!("CARGO_PKG_VERSION"),"algorithm":{"structure":"external","source":sha,"format":format,"source_file":source},"states":by_state.iter().map(|(name,a)|(name.clone(),serde_json::json!({"status":"ok","tracts":a.len()}))).collect::<serde_json::Map<String,serde_json::Value>>(),"summary":{"total":by_state.values().map(|a|a.len()).sum::<usize>(),"states":by_state.len()}})
}

pub fn parse_shapefile_bytes(
    shp: &[u8],
    dbf: &[u8],
    shx: Option<&[u8]>,
    source_name: &str,
) -> Result<HashMap<String, u64>, String> {
    use std::io::Cursor;
    let shapes = match shx {
        Some(index) => shapefile::ShapeReader::with_shx(Cursor::new(shp), Cursor::new(index)),
        None => shapefile::ShapeReader::new(Cursor::new(shp)),
    }
    .map_err(|e| format!("[INPUT] cannot read shapefile '{source_name}': {e}"))?;
    let attributes = shapefile::dbase::Reader::new(Cursor::new(dbf))
        .map_err(|e| format!("[INPUT] cannot read shapefile '{source_name}': {e}"))?;
    parse_shapefile_reader(shapefile::Reader::new(shapes, attributes), source_name)
}
