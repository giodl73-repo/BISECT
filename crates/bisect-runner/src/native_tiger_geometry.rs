//! Native State's algorithm-specific TIGER source and area alignment.
//! Paths are labels here. Filesystem and selected-byte adapters supply observations.
use std::collections::HashMap;
use crate::native_replay_recipe::{CountDomain,Recipe,SourceIdentity};
use crate::state_config::{SplitStrategy,VertexConstraintKind};

pub fn relative_path(state:&str,year:&str)->Option<String>{
 let fips=bisect_core::state_code_to_fips(&state.to_uppercase())?;
 let stem=format!("tl_{year}_{fips}_tract");
 Some(format!("data/{year}/tiger/tracts/{stem}/{stem}.shp"))
}
pub fn consumed<C:CountDomain>(recipe:&Recipe<C>)->bool{
 recipe.state.algorithm.vertex_constraints.contains(&VertexConstraintKind::Area)
 ||matches!(recipe.state.algorithm.split,SplitStrategy::CompactBisect{..})
}
/// New explicit algorithm identities, or old geometry identities only when their
/// path exactly equals the historical algorithm wrapper's single candidate.
pub fn records<C:CountDomain>(recipe:&Recipe<C>)->Result<[&SourceIdentity;4],String>{
 let normalize=|s:&str|s.replace('\\',"/");
 let relative=relative_path(&recipe.state.state_code,&recipe.state.year).ok_or("Native algorithm TIGER path is unavailable for this state")?;
 let expected=format!("{}/{}",normalize(&recipe.original_working_directory).trim_end_matches('/'),relative);
 let prefix=if recipe.sources.iter().any(|s|s.role=="algorithm-tiger-shp"){"algorithm-tiger"}else{"geometry"};
 let find=|ext:&str|->Result<&SourceIdentity,String>{
  let role=format!("{prefix}-{ext}");let record=recipe.sources.iter().find(|s|s.role==role).ok_or("Recorded native algorithm TIGER observation is unknown; current geometry or filesystem discovery is not historical evidence")?;
  let path=format!("{}.{ext}",expected.strip_suffix(".shp").unwrap());
  if normalize(&record.path)!=path{return Err(format!("Native algorithm TIGER source path differs from its recorded working-directory candidate: {role}"));}
  if recipe.sources.iter().any(|other|normalize(&other.path)==normalize(&record.path)&&other.sha256!=record.sha256){return Err(format!("Contradictory native algorithm TIGER observations for the same path: {role}"));}
  Ok(record)
 };
 Ok([find("shp")?,find("dbf")?,find("shx")?,find("prj")?])
}

/// The original native wrapper's exact GEOID -> ALAND alignment. Duplicate
/// native tract IDs retain its last-record behavior; missing graph IDs get zero.
pub fn align(records:&[bisect_data::TractRecord],index_to_geoid:&HashMap<usize,String>,adjacency:&[Vec<usize>],edges:&HashMap<(usize,usize),f64>)->(Vec<f64>,Vec<f64>){
 let geoid_to_aland:HashMap<String,f64>=records.iter().map(|r|(r.geoid.clone(),r.aland as f64)).collect();
 let n=adjacency.len();let mut areas=vec![0.0;n];let mut perimeters=vec![0.0;n];
 for(&idx,geoid)in index_to_geoid{
  if idx>=n{continue;}
  let area=*geoid_to_aland.get(geoid).unwrap_or(&0.0);areas[idx]=area;
  perimeters[idx]=crate::compact_geometry::native_state_exterior_perimeter(idx,area,&adjacency[idx],edges);
 }
 (areas,perimeters)
}
