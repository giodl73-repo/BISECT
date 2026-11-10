//! Existing native assignment exporters, with bounded RPLAN v0.1/v0.2 input.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImportSettings {
    pub format:String,pub state:String,pub year:u16,pub label:String,pub created_at:String,
    pub source_name:String,pub as_civic_counter_proposal:bool,pub submitted_by:Option<String>,pub submitted_at:Option<String>,
}

pub(crate) fn import(text:&str,centroids_text:Option<&str>,context_text:Option<&str>,s:ImportSettings)->Result<Value,String>{
    if text.len()>3*1024*1024{return Err("Plan import source exceeds 3 MiB.".into());}
    if s.state.len()!=2||!s.state.bytes().all(|b|b.is_ascii_uppercase())||!(1900..=2100).contains(&s.year){return Err("Supply a two-letter uppercase state and year 1900–2100.".into());}
    for value in [s.label.as_str(),s.created_at.as_str(),s.source_name.as_str()].into_iter().chain(s.submitted_by.as_deref()).chain(s.submitted_at.as_deref()){
        if value.trim().is_empty()||value.len()>200{return Err("Import labels, source name and recorded metadata must be nonempty and at most 200 bytes.".into());}
    }
    if s.as_civic_counter_proposal&&(s.submitted_by.is_none()||s.submitted_at.is_none()){return Err("Civic counter-proposal requires submitter and recorded submission timestamp.".into());}
    let mut polygon_diagnostic=None;
    let assignments=match s.format.as_str(){
        "gerrychain"=>bisect_report::import_gerrychain_to_assignments(text),
        "dra-csv"=>bisect_report::assignment_import::import_assignments_from_dra_csv(text),
        "geojson"=>bisect_report::assignment_import::import_assignments_from_geojson_properties(text),
        "geojson-polygons"=>{
            let centroid_source=centroids_text.ok_or("District polygons require an explicit tract centroid JSON file.")?;
            let detailed=polygon_import(text,centroid_source)?;
            polygon_diagnostic=Some(json!({"nearest_centroid_units":detailed.nearest_centroid_units,"nearest_centroid_count":detailed.nearest_centroid_units.len(),"point_in_polygon_count":detailed.assignments.len()-detailed.nearest_centroid_units.len(),"centroids_sha256":format!("{:x}",Sha256::digest(centroid_source.as_bytes())),"scope":"Native planar point-in-polygon; overlapping polygons use source order. Border/outside/hole points use nearest polygon centroid in squared longitude/latitude degrees, not geodesic distance. Fallback is reported, not evidence of geographic or legal correctness."}));
            Ok(detailed.assignments)
        },
        _=>return Err("Supported formats: gerrychain, dra-csv, geojson assignment properties.".into()),
    }.map_err(|e|e.to_string())?;
    if assignments.is_empty()||assignments.len()>100_000{return Err("Import requires 1–100,000 tract assignments.".into());}
    bisect_report::validate_geoid_format_batch(&assignments).map_err(|e|e.to_string())?;
    let k=*assignments.values().max().unwrap();if k>500{return Err("Import supports district IDs 1–500.".into());}
    let source_hash=format!("{:x}",Sha256::digest(text.as_bytes()));
    let manifest=bisect_report::PlanManifest{label:s.label.clone(),state_code:s.state.clone(),year:s.year.to_string(),chamber:"imported".into(),num_districts:k,
        created_at:s.created_at.clone(),submission_type:if s.as_civic_counter_proposal{"civic_counter_proposal"}else{"authoritative"}.into(),
        submitted_by:s.submitted_by.clone(),submitted_at:s.submitted_at.clone(),source_tool:Some(s.format.clone()),..Default::default()};
    let legacy=bisect_report::RplanFile{rplan_version:"0.1".into(),metadata:bisect_report::RplanMetadata{
        label:s.label,state_code:s.state,year:s.year.to_string(),chamber:"imported".into(),num_districts:k,population_source:"unknown".into(),created_at:s.created_at,
        created_by:"bisect import".into(),notes:Some(format!("imported from {}",s.source_name)),..Default::default()},assignments,geometry:None};
    let legacy_json=serde_json::to_string_pretty(&legacy).map_err(|e|e.to_string())?;
    let mut document=rplan_io::read_rplan_str(&legacy_json).map_err(|e|e.to_string())?;
    let context=context_text.map(|text|{
        if text.len()>8*1024*1024{return Err("Import context exceeds 8 MiB.".to_string());}
        rplan_io::read_rctx_str(text).map_err(|e|e.to_string())
    }).transpose()?;
    if let Some(ctx)=&context {
        if ctx.units.unit_kind!=rplan_core::UnitKind::Tract||ctx.units.state!=document.plan.units.state||ctx.units.year!=document.plan.units.year||ctx.units.unit_ids.len()!=legacy.assignments.len(){return Err("Import context jurisdiction/year/unit set does not match source assignments.".into());}
        if ctx.units.unit_universe_hash!=ctx.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{return Err("Import context unit-universe hash mismatch.".into());}
        let assignment=ctx.units.unit_ids.iter().map(|id|legacy.assignments.get(id).map(|d|(*d-1) as u32).ok_or_else(||format!("Import context unit {id} has no assignment"))).collect::<Result<Vec<_>,_>>()?;
        document.plan.units=ctx.units.clone();document.plan.assignment=assignment;
        rplan_audit::measurements::measure_plan(&document.plan,Some(ctx))?;
    }
    document.provenance.producer.insert("method".into(),json!("interchange-import"));
    document.provenance.producer.insert("source_format".into(),json!(s.format));
    document.provenance.source_hashes.insert("import_source".into(),format!("sha256:{source_hash}"));
    if let Some(d)=&polygon_diagnostic {document.provenance.source_hashes.insert("import_centroids".into(),format!("sha256:{}",d["centroids_sha256"].as_str().unwrap()));}
    document.extensions.insert("org.bisect.interchange-import".into(),json!({"source_name":s.source_name,"source_sha256":source_hash,"submission_type":manifest.submission_type,"submitted_by":manifest.submitted_by,"submitted_at":manifest.submitted_at}));
    let document_json=rplan_io::write_rplan_string(&document).map_err(|e|e.to_string())?;
    let files=std::collections::BTreeMap::from([("imported.rplan",document_json.clone()),("legacy.rplan",legacy_json),
        ("final_assignments.json",serde_json::to_string_pretty(&legacy.assignments.iter().collect::<std::collections::BTreeMap<_,_>>()).map_err(|e|e.to_string())?),
        ("manifest.json",serde_json::to_string_pretty(&manifest).map_err(|e|e.to_string())?)]);
    let hashes:std::collections::BTreeMap<_,_>=files.iter().map(|(name,text)|(*name,format!("{:x}",Sha256::digest(text.as_bytes())))).collect();
    Ok(json!({"schema_version":"bisect-plan-import-v1","document_json":document_json,"files":files,"file_sha256":hashes,"source_sha256":source_hash,
        "plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?,"units":document.plan.assignment.len(),"districts":k,
        "context_json":context.as_ref().map(rplan_io::write_rctx_string).transpose().map_err(|e|e.to_string())?,"polygon_diagnostic":polygon_diagnostic,
        "scope":"Shared native GerryChain, DRA CSV, tract-property GeoJSON or explicit-centroid district-polygon import. Positive one-based districts become zero-based RPLAN IDs; source bytes and declared submission metadata are retained. Requires 11-digit tract IDs and a complete contiguous district-label range. Optional hash-checked RCTX must match every source unit, state and year; modern plan order is explicitly aligned to that context. Polygon imports report native nearest-centroid fallbacks. Geometric boundaries are not embedded or recomputed. No population, contiguity or legal audit has run. Source authenticity and the native manifest's authoritative submission tag are declarations, not verified authority. Supply matching RCTX and a legal profile for auditing; display geometry is separate."}))
}

fn polygon_import(text:&str,centroids_text:&str)->Result<bisect_report::GeojsonAssignmentResult,String>{
    let centroids=crate::certified_exact::parse::<UniqueCentroids>(centroids_text)?.0;
    if centroids.is_empty()||centroids.len()>100_000{return Err("Supply 1–100,000 explicit tract centroids.".into());}
    for (id,(lon,lat)) in &centroids{
        bisect_report::validate_geoid_format(id).map_err(|e|e.to_string())?;
        if !lon.is_finite()||!lat.is_finite()||lon.abs()>180.0||lat.abs()>90.0{return Err("Centroid coordinates must be finite WGS84 longitude/latitude.".into());}
    }
    let raw:Value=crate::certified_exact::parse(text)?;
    let features=raw["features"].as_array().ok_or("Supply polygon FeatureCollection.")?;
    if features.len()>5000{return Err("Polygon import supports at most 5,000 features.".into());}
    let mut vertices=0usize;let mut polygons=0usize;
    for f in features{
        if !f["properties"]["district_id"].as_u64().is_some_and(|d|(1..=500).contains(&d)){return Err("Polygon district IDs must be integers 1–500.".into());}
        let g=&f["geometry"];if g.is_null(){continue;}
        let shapes=match g["type"].as_str(){
            Some("Polygon")=>vec![&g["coordinates"]],
            Some("MultiPolygon")=>g["coordinates"].as_array().ok_or("Malformed MultiPolygon.")?.iter().collect(),
            _=>return Err("District geometry must be Polygon or MultiPolygon.".into()),
        };
        for shape in shapes{polygons+=1;let rings=shape.as_array().ok_or("Malformed polygon rings.")?;
            if rings.is_empty(){return Err("Polygon needs an exterior ring.".into());}
            for ring in rings{let positions=ring.as_array().ok_or("Malformed ring.")?;if positions.len()<4||positions.first()!=positions.last(){return Err("Polygon rings must be closed and have at least four positions.".into());}
                vertices+=positions.len();for p in positions{let a=p.as_array().ok_or("Malformed position.")?;
                    let lon=a.first().and_then(Value::as_f64).ok_or("Missing longitude.")?;let lat=a.get(1).and_then(Value::as_f64).ok_or("Missing latitude.")?;
                    if !lon.is_finite()||!lat.is_finite()||lon.abs()>180.0||lat.abs()>90.0{return Err("Polygon coordinates must be finite WGS84 longitude/latitude.".into());}
                }
            }
        }
    }
    if vertices.saturating_mul(centroids.len())>50_000_000||polygons.saturating_mul(centroids.len())>2_000_000{return Err("Polygon import exceeds browser work limits.".into());}
    bisect_report::import_geojson_plan_detailed(text,&centroids).map_err(|e|e.to_string())
}

struct UniqueCentroids(std::collections::HashMap<String,(f64,f64)>);
impl<'de> Deserialize<'de> for UniqueCentroids{
    fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{
        struct Unique;
        impl<'de> serde::de::Visitor<'de> for Unique{
            type Value=UniqueCentroids;
            fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("unique tract IDs mapped to coordinate pairs")}
            fn visit_map<M:serde::de::MapAccess<'de>>(self,mut m:M)->Result<Self::Value,M::Error>{
                let mut out=std::collections::HashMap::new();
                while let Some((id,point))=m.next_entry::<String,(f64,f64)>()?{if out.insert(id.clone(),point).is_some(){return Err(serde::de::Error::custom(format!("Duplicate centroid unit {id}")));}}
                Ok(UniqueCentroids(out))
            }
        }
        d.deserialize_map(Unique)
    }
}

pub(crate) fn export(text:&str)->Result<Value,String>{
    let plan=crate::plan_audit_text::plan(text)?;
    if plan.units.unit_universe_hash!=plan.units.compute_unit_universe_hash().map_err(|e|e.to_string())?{
        return Err("Plan unit-universe hash does not match its ordered unit IDs.".into());
    }
    let document=rplan_io::read_rplan_str(text).map_err(|e|e.to_string())?;
    let raw:Value=crate::certified_exact::parse(text)?;
    // Keep legacy metadata exactly as the CLI exporters receive it. Modern plans
    // cross the same one-based interchange boundary as legacy RPLAN.
    let legacy:bisect_report::RplanFile=if raw["rplan_version"]=="0.1" {
        serde_json::from_value(raw).map_err(|e|e.to_string())?
    }else{
        bisect_report::RplanFile{
            rplan_version:"0.1".into(),
            metadata:bisect_report::RplanMetadata{
                label:document.metadata.label.clone(),state_code:plan.units.state.clone().unwrap_or_else(||document.metadata.jurisdiction.clone()),
                year:plan.units.year.map(|y|y.to_string()).unwrap_or_default(),chamber:document.metadata.chamber.clone(),num_districts:plan.k,
                created_at:document.metadata.created_at.clone(),..Default::default()
            },
            assignments:plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect(),
            geometry:document.geometry.clone(),
        }
    };
    let geometry_supplied=legacy.geometry.is_some();
    let files=std::collections::BTreeMap::from([
        ("assignments.csv",bisect_report::export_tracts_csv(&legacy).map_err(|e|e.to_string())?),
        ("gerrychain.json",bisect_report::export_gerrychain_v23(&legacy).map_err(|e|e.to_string())?),
        ("plan.geojson",bisect_report::export_geojson(&legacy).map_err(|e|e.to_string())?),
    ]);
    let hashes:std::collections::BTreeMap<_,_>=files.iter().map(|(name,text)|(*name,format!("{:x}",Sha256::digest(text.as_bytes())))).collect();
    Ok(json!({"schema_version":"bisect-plan-interchange-v1","files":files,"file_sha256":hashes,
        "document_json":rplan_io::write_rplan_string(&document).map_err(|e|e.to_string())?,
        "plan_hash":plan.plan_hash().map_err(|e|e.to_string())?,"units":plan.assignment.len(),"districts":plan.k,"geometry_supplied":geometry_supplied,
        "scope":"Shared native CSV, GerryChain v2.3 and GeoJSON exporters. Assignment exports use one-based numeric district IDs, independent of display labels. CSV units are sorted by ID. Native GeoJSON reuses embedded geometry without dissolving or recomputing it; absent geometry produces null-geometry district features. Separate display GeoJSON is not merged into this export. Shape validation is not a population, contiguity, geometry authenticity or legal audit. Reproducibility-package export remains separate."}))
}

pub(crate) fn reproducibility(text:&str,manifest_text:Option<&str>,analysis:&std::collections::BTreeMap<String,String>,label:&str)->Result<Value,String>{
    let bytes=text.len().saturating_add(manifest_text.map(str::len).unwrap_or(0)).saturating_add(analysis.iter().map(|(k,v)|k.len().saturating_add(v.len())).sum::<usize>());
    if bytes>3*1024*1024||analysis.len()>64{return Err("Reproducibility inputs must total at most 3 MiB and 64 analysis files.".into());}
    // Reuse the same exact-plan validation and native legacy conversion as interchange export.
    let exported=export(text)?;
    let document=rplan_io::read_rplan_str(exported["document_json"].as_str().ok_or("Missing validated plan")?).map_err(|e|e.to_string())?;
    let raw:Value=crate::certified_exact::parse(text)?;
    let legacy:bisect_report::RplanFile=if raw["rplan_version"]=="0.1"{serde_json::from_value(raw).map_err(|e|e.to_string())?}else{
        bisect_report::RplanFile {rplan_version:"0.1".into(),metadata:bisect_report::RplanMetadata {
            label:document.metadata.label.clone(),state_code:document.plan.units.state.clone().unwrap_or_else(||document.metadata.jurisdiction.clone()),
            year:document.plan.units.year.map(|y|y.to_string()).unwrap_or_default(),chamber:document.metadata.chamber.clone(),num_districts:document.plan.k,
            created_at:document.metadata.created_at.clone(),..Default::default()},
            assignments:document.plan.units.unit_ids.iter().cloned().zip(document.plan.assignment.iter().map(|d|*d as usize+1)).collect(),geometry:document.geometry.clone()}
    };
    if let Some(source)=manifest_text{let _:bisect_report::manifest_schema::ReadOnlyPlanManifest=crate::certified_exact::parse(source)?;}
    for source in analysis.values(){let _:Value=crate::certified_exact::parse(source)?;}
    let files=bisect_report::reproducibility::build_reproducibility_package(label,&legacy,manifest_text,analysis).map_err(|e|e.to_string())?;
    let hashes:std::collections::BTreeMap<_,_>=files.iter().map(|(name,source)|(name.clone(),format!("{:x}",Sha256::digest(source.as_bytes())))).collect();
    Ok(json!({"schema_version":"bisect-reproducibility-package-v1","files":files,"file_sha256":hashes,
        "source_plan_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"manifest_supplied":manifest_text.is_some(),"analysis_files":analysis.len(),
        "scope":"Native reproducibility-package files built on this device. Supplied manifest and analysis bytes are copied, not authenticated or recomputed. audit.json records manifest declarations; it is not an audit certificate. verify.sh contains quoted native CLI instructions and is never executed here. Missing manifest or seed makes verification instructions unavailable. Package does not include the native executable, Census sources, adjacency or external analyzers; those must be obtained separately. Legacy RPLAN output uses one-based district IDs; no automatic population, contiguity or legal audit."}))
}
