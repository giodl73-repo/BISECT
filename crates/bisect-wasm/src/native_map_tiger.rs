//! Population-free, exact-coordinate geometry for legacy native map inputs.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Cursor;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    state: String,
    tiger: crate::tiger_source::Input,
    geometry_profile: Profile,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Profile { SourceExact, SourceExactLarge }

pub(crate) fn prepare(input: Input) -> Result<Value, String> {
    use crate::tiger_source as tiger;
    let block_groups=matches!(input.tiger.kind,tiger::Kind::BlockGroups);
    if !matches!(input.tiger.kind, tiger::Kind::Tracts|tiger::Kind::BlockGroups) || ![2000, 2010, 2020].contains(&input.tiger.year) {
        return Err("Native map geometry requires tracts or block groups and Census year 2000, 2010 or 2020".into());
    }
    let manifest: bisect_core::fetch_plan::Manifest = serde_json::from_str(include_str!("../../../data/manifest.json")).map_err(|e| e.to_string())?;
    let state = manifest.states.get(&input.state).ok_or("Select a known state code")?;
    let shp_raw = tiger::bytes(&input.tiger.shp)?;
    let dbf_raw = tiger::bytes(&input.tiger.dbf)?;
    let shx_raw = input.tiger.shx.as_ref().map(tiger::bytes).transpose()?;
    let shp = tiger::decoded_part(&shp_raw)?;
    let dbf = tiger::decoded_part(&dbf_raw)?;
    let shx = shx_raw.as_deref().map(tiger::decoded_part).transpose()?;
    tiger::preflight(&shp, shx.as_deref())?;
    let records = bisect_data::tiger::read_tiger_units_from_streams(Cursor::new(shp.as_ref()), Cursor::new(dbf.as_ref()), shx.as_deref().map(Cursor::new), block_groups).map_err(|e| e.to_string())?;
    let mut ids = std::collections::BTreeSet::new();
    if records.is_empty() || records.len() > 100_000 || records.iter().any(|r| r.geoid.len() != if block_groups {12} else {11} || !r.geoid.bytes().all(|b| b.is_ascii_digit()) || !r.geoid.starts_with(&state.fips) || !ids.insert(r.geoid.clone())) {
        return Err("Tract universe must be nonempty, unique, numeric and in the selected state".into());
    }
    let large = matches!(input.geometry_profile, Profile::SourceExactLarge);
    let mut geometry = bisect_data::tract_preparation::census_source_geometry_bounded(&records, &input.state, &input.tiger.year.to_string(), if large {2_000_000} else {500_000})?;
    if block_groups {geometry["unit_kind"]=json!("block-group");}
    if large { geometry["geometry_profile"] = json!("source-exact-large"); }
    let geometry_json = serde_json::to_string(&geometry).map_err(|e| e.to_string())?;
    if geometry_json.len() > if large {64*1024*1024} else {16*1024*1024} { return Err("Exact native map geometry exceeds the selected profile size limit".into()); }
    let identity = |source: &tiger::Source, bytes: &[u8]| json!({"name":source.name,"byte_length":bytes.len(),"sha256":format!("{:x}",Sha256::digest(bytes))});
    let mut output=json!({"schema_version":"bisect-native-map-tiger-v1","status":"prepared","state":input.state,"year":input.tiger.year,"units":records.len(),"geometry_profile":if large {"source-exact-large"}else{"source-exact"},"geometry_json":geometry_json,"geometry_sha256":format!("{:x}",Sha256::digest(geometry_json.as_bytes())),"shp":identity(&input.tiger.shp,&shp_raw),"dbf":identity(&input.tiger.dbf,&dbf_raw),"shx":input.tiger.shx.as_ref().zip(shx_raw.as_ref()).map(|(s,b)|identity(s,b)),"shp_decoded":tiger::decoded_identity(&shp_raw,&shp),"dbf_decoded":tiger::decoded_identity(&dbf_raw,&dbf),"shx_decoded":shx_raw.as_ref().zip(shx.as_ref()).map(|(r,d)|tiger::decoded_identity(r,d)),"execution_performed":false,"scope":"Shared native tract reader and exact decoded source coordinates, retaining holes and islands. No simplification, population join, graph preparation, partitioning or RPLAN creation. Selected state/vintage and geographic coordinate system are source claims; no PRJ or publisher authentication was checked. Native map geometry validation and dissolve work limits apply on rendering. Explicit profile limits do not establish all-state capacity."});
    if block_groups {output["unit_kind"]=json!("block-group");output["scope"]=json!("Selected block-group geometry through the native twelve-digit reader, preserving exact coordinates and native reader order. No population join, partitioning, simplification, authentication or certification. Selected state/vintage and coordinate system remain source claims; existing source/output limits apply.");}
    Ok(output)
}
