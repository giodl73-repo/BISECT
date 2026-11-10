use std::collections::HashMap;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().skip(1).collect();if args.len()!=2{return Err("Expected national bundle JSON path and output stem".into());}
 let bundle:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&args[0])?)?;let dpi=bundle["dpi"].as_u64().ok_or("DPI")? as u32;let kind=bundle["map_type"].as_str().ok_or("Map type")?;
 let manifest:bisect_core::fetch_plan::Manifest=serde_json::from_str(include_str!("../../../data/manifest.json"))?;
 let mut states:Vec<_>=bundle["states"].as_object().ok_or("States")?.iter().collect();states.sort_by_key(|(code,_)|manifest.states.get(*code).map(|s|s.name.to_lowercase().replace(' ',"_")).unwrap_or_else(||(*code).clone()));
 let mut all=vec![];
 for (_,source) in states {
  let document=rplan_io::read_rplan_str(source["document_json"].as_str().ok_or("Plan")?)?;
  let assignments:HashMap<_,_>=document.plan.units.unit_ids.iter().cloned().zip(document.plan.assignment.iter().map(|d|*d as usize+1)).collect();
  let method:bisect_analysis::district_geometry::DissolveMethod=serde_json::from_value(source.get("dissolve_method").cloned().unwrap_or(serde_json::json!("sequential")))?;
  let (districts,_)=bisect_analysis::district_geometry::read_unit_geometries_with_method(source["unit_geometry_json"].as_str().ok_or("Geometry")?,&assignments,bisect_analysis::district_geometry::CoordinateBasis::Wgs84Degrees,method)?;
  let analysis=source["analysis_json"].as_str().map(serde_json::from_str).transpose()?.unwrap_or(serde_json::Value::Null);
  all.extend(bisect_map::renderer::state_map_items(districts,document.plan.k,kind,&analysis)?);
 }
 let (w,h)=bisect_map::canvas_size_from_dpi(dpi,16.0,1.6);let projection=bisect_map::InsetProjection::us_national(w,h);
 let svg=bisect_map::renderer::build_national_svg(&all,&projection,w,h);let png=bisect_map::svg_to_png(&svg,&bisect_map::default_font_db())?;
 std::fs::write(format!("{}.svg",args[1]),svg)?;std::fs::write(format!("{}.png",args[1]),png)?;println!("{w} {h}");Ok(())
}
