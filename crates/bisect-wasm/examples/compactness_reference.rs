//! File-backed native reference for the same report used by bisect analyze --type compactness.
use bisect_analysis::district_geometry::{compactness_report,read_unit_geometries_with_method,CoordinateBasis,DissolveMethod};
fn main()->Result<(),Box<dyn std::error::Error>>{
    let args:Vec<_>=std::env::args().collect();
    let request:serde_json::Value=serde_json::from_slice(&std::fs::read(args.get(1).ok_or("request path required")?)?)?;
    let plan=rplan_io::read_rplan_str(request["document_json"].as_str().ok_or("plan required")?)?.plan;
    let assignments=plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect();
    let basis:CoordinateBasis=serde_json::from_value(request["coordinate_basis"].clone())?;
    let method:DissolveMethod=request.get("dissolve_method").map(|m|serde_json::from_value(m.clone())).transpose()?.unwrap_or_default();
    let (districts,_)=read_unit_geometries_with_method(request["unit_geometry_json"].as_str().ok_or("geometry required")?,&assignments,basis,method)?;
    let mut report=compactness_report(&districts,plan.units.state.as_deref().unwrap_or("unknown"),&plan.units.year.map(|y|y.to_string()).unwrap_or_else(||"unknown".into()),basis,request["crs_label"].as_str().ok_or("CRS required")?);
    if method==DissolveMethod::Balanced {report["dissolve_method"]=serde_json::json!("balanced-input-order-pairs-indexed-work-v1");}
    if method==DissolveMethod::SpatialBalanced {report["dissolve_method"]=serde_json::json!("balanced-morton16-disjoint-envelopes-v1");}
    if method==DissolveMethod::Batch {report["dissolve_method"]=serde_json::json!("validated-partition-single-sweep-v1");}
    if request["include_exact_reock"].as_bool().unwrap_or(false){bisect_analysis::district_geometry::add_exact_reock(&mut report,&districts,Some(50_000_000))?;}
    if let Some(source)=request["population_centroids_csv"].as_str(){bisect_analysis::population_geometry::add_population_compactness(&mut report,&districts,&assignments,source,basis)?;}
    std::fs::write(args.get(2).ok_or("report path required")?,serde_json::to_string_pretty(&report)?)?;Ok(())
}
