//! Direct native reader/render oracle: no property rewriting or source repair.
use std::collections::HashMap;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().skip(1).collect();
 if args.len()!=4{return Err("plan geometry dpi output-stem required".into());}
 let plan=rplan_io::read_rplan_str(&std::fs::read_to_string(&args[0])?)?.plan;
 let geometry=std::fs::read_to_string(&args[1])?;
 let assignments:HashMap<_,_>=plan.units.unit_ids.iter().cloned().zip(plan.assignment.iter().map(|d|*d as usize+1)).collect();
 let (districts,_)=bisect_analysis::district_geometry::read_unit_geometries_with_method(&geometry,&assignments,bisect_analysis::district_geometry::CoordinateBasis::Wgs84Degrees,bisect_analysis::district_geometry::DissolveMethod::Sequential)?;
 let (svg,w,h)=bisect_map::renderer::state_map_svg(districts,plan.k,args[2].parse()?,"districts",&serde_json::Value::Null)?;
 let png=bisect_map::svg_to_png(&svg,&bisect_map::default_font_db())?;
 std::fs::write(format!("{}.svg",args[3]),svg)?;std::fs::write(format!("{}.png",args[3]),png)?;println!("{w} {h}");Ok(())
}
