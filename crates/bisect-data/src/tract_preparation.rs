//! Shared experimental lab-tract-v1 construction from decoded Census sources.
use crate::{AdjacencyGraph,TractRecord,Pl94BlockPopulation};
use geo::{Centroid,Simplify};
use std::collections::HashMap;

pub struct TractPreparation {
 pub graph:AdjacencyGraph,
 pub geoids:Vec<String>,
 pub centroids:Vec<(f64,f64)>,
 pub geographic_centroids:Vec<(f64,f64)>,
 pub bridges:Vec<(usize,usize)>,
 pub bridge_weight:f64,
 pub input_population:i64,
}
#[derive(Clone,Copy,Debug,serde::Serialize,serde::Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum CensusResolution{County,Tract,BlockGroup,Block}
impl CensusResolution{pub fn width(self)->usize{match self{Self::County=>5,Self::Tract=>11,Self::BlockGroup=>12,Self::Block=>15}}}
pub fn prepare_tract_graph(records:&[TractRecord],blocks:&[Pl94BlockPopulation])->Result<TractPreparation,String>{
 prepare_graph(records,blocks,11,false)
}
pub fn prepare_census_graph(records:&[TractRecord],blocks:&[Pl94BlockPopulation],resolution:CensusResolution)->Result<TractPreparation,String>{
 prepare_graph(records,blocks,resolution.width(),true)
}
fn prepare_graph(records:&[TractRecord],blocks:&[Pl94BlockPopulation],width:usize,strict:bool)->Result<TractPreparation,String>{
 if strict {
  let mut ids=std::collections::HashSet::new();
  if records.is_empty()||records.iter().any(|r|r.geoid.len()!=width||!r.geoid.bytes().all(|c|c.is_ascii_digit())||!ids.insert(&r.geoid)){return Err("Census geometry must contain unique IDs at the selected resolution".into());}
  let mut ids=std::collections::HashSet::new();
  if blocks.iter().any(|r|r.geoid.len()!=15||!r.geoid.bytes().all(|c|c.is_ascii_digit())||r.population<0||!ids.insert(&r.geoid)){return Err("PL population must contain unique nonnegative fifteen-digit block rows".into());}
 }
 let mut populations=HashMap::<String,i64>::new();
 for block in blocks{
  let tract=block.geoid.get(..width).ok_or("PL block GEOID is shorter than a tract")?;
  let count=populations.entry(tract.into()).or_default();
  *count=count.checked_add(block.population).ok_or("Tract population overflow")?;
 }
 let input_population=populations.values().try_fold(0i64,|sum,p|sum.checked_add(*p)).ok_or("State population overflow")?;
 let geographic=records.iter().enumerate().map(|(i,r)|crate::adjacency::parse_wkb_multipolygon(&r.geometry_wkb,i).map_err(|e|e.to_string())).collect::<Result<Vec<_>,_>>()?;
 let projected:Vec<_>=geographic.iter().map(crate::tiger::project_epsg5070).collect();
 let wkbs:Vec<_>=projected.iter().map(crate::tiger::geo_to_wkb_multipolygon).collect();
 let mut graph=crate::build_adjacency_graph(&wkbs,10.).map_err(|e|e.to_string())?;
 graph.vertex_weights=records.iter().map(|r|populations.get(&r.geoid).copied().unwrap_or(0)).collect();
 let joined=graph.vertex_weights.iter().try_fold(0i64,|sum,p|sum.checked_add(*p)).ok_or("Joined population overflow")?;
 if input_population<=0||joined!=input_population{return Err("TIGER/population join did not retain the full state population.".into());}
 graph.set_areas(records.iter().map(|r|r.aland as f64).collect());
 let centers=|polys:&[geo_types::MultiPolygon<f64>]|polys.iter().map(|g|g.centroid().map(|c|(c.x(),c.y())).ok_or_else(||"Empty tract geometry.".to_string())).collect::<Result<Vec<_>,_>>();
 let centroids=centers(&projected)?;let geographic_centroids=centers(&geographic)?;
 let geoids:Vec<_>=records.iter().map(|r|r.geoid.clone()).collect();
 let mut bridges=crate::connect_island_components(&graph.adjacency,&centroids,&geoids);bridges.sort();
 let mut lengths:Vec<_>=graph.edge_weights.values().copied().collect();lengths.sort_by(f64::total_cmp);
 let bridge_weight=lengths.get(lengths.len()/2).copied().unwrap_or(1.);
 for &(u,v) in &bridges{graph.adjacency[u].push(v);graph.adjacency[v].push(u);graph.edge_weights.insert((u.min(v),u.max(v)),bridge_weight);}
 for ns in &mut graph.adjacency{ns.sort();ns.dedup();}graph.n_edges=graph.edge_weights.len();
 Ok(TractPreparation{graph,geoids,centroids,geographic_centroids,bridges,bridge_weight,input_population})
}

/// The native laboratory's geographic display profile; analytic WKB is unsimplified.
pub fn tract_display_geometry(records:&[TractRecord],state:&str,year:&str)->Result<serde_json::Value,String>{
 census_geometry(records,state,year,true,None)
}
/// Exact geographic coordinates decoded from the selected source WKB.
/// No simplification, snapping, reprojection or topology repair is performed.
pub fn census_source_geometry(records:&[TractRecord],state:&str,year:&str)->Result<serde_json::Value,String>{
 census_geometry(records,state,year,false,None)
}
/// Enforce a source-position cap while decoding, before materializing oversized output.
pub fn census_source_geometry_bounded(records:&[TractRecord],state:&str,year:&str,max_positions:usize)->Result<serde_json::Value,String>{
 census_geometry(records,state,year,false,Some(max_positions))
}
/// Conservative shared-boundary approximation; original analytic WKB is untouched.
pub fn census_topology_display_geometry(records:&[TractRecord],state:&str,year:&str)->Result<serde_json::Value,String>{
 let source=records.iter().enumerate().map(|(i,r)|crate::adjacency::parse_wkb_multipolygon(&r.geometry_wkb,i).map_err(|e|e.to_string())).collect::<Result<Vec<_>,_>>()?;
 let simplified=crate::topology_simplify::simplify_shared(&source,0.0002)?;
 let features=records.iter().zip(simplified).map(|(r,polygons)|{
  let coordinates=polygons.0.iter().map(|p|std::iter::once(p.exterior()).chain(p.interiors()).map(|ring|ring.0.iter().map(|c|[c.x,c.y]).collect::<Vec<_>>()).collect::<Vec<_>>()).collect::<Vec<_>>();
  Ok(serde_json::json!({"type":"Feature","properties":{"geoid":r.geoid,"state":state,"county":r.geoid.get(..5).ok_or("Short Census GEOID")?},"geometry":{"type":"MultiPolygon","coordinates":coordinates}}))
 }).collect::<Result<Vec<_>,String>>()?;
 Ok(serde_json::json!({"type":"FeatureCollection","state":state,"year":year,"geometry_profile":"display-topology","display_simplification_degrees":0.0002,"display_simplification_method":"shared-boundary-conservative-v1","coordinate_basis":"nad83-degrees","crs_label":"EPSG:4269","features":features}))
}
fn census_geometry(records:&[TractRecord],state:&str,year:&str,simplified:bool,max_positions:Option<usize>)->Result<serde_json::Value,String>{
 let mut source_positions=0usize;
 let features=records.iter().enumerate().map(|(i,r)|{
  let polygons=crate::adjacency::parse_wkb_multipolygon(&r.geometry_wkb,i).map_err(|e|e.to_string())?;
  if let Some(limit)=max_positions {
   for p in &polygons.0{for ring in std::iter::once(p.exterior()).chain(p.interiors()){
    source_positions=source_positions.checked_add(ring.0.len()).ok_or("Exact source position overflow")?;
    if source_positions>limit{return Err(format!("Exact source geometry exceeds the {limit}-position analysis limit. Select display geometry for visualization; analysis-quality shared-boundary simplification is still required for larger sources."));}
   }}
  }
  let coordinates:Vec<_>=polygons.0.iter().map(|p|{let p=if simplified{p.simplify(&0.0002)}else{p.clone()};std::iter::once(p.exterior()).chain(p.interiors()).map(|ring|ring.0.iter().map(|c|[c.x,c.y]).collect::<Vec<_>>()).collect::<Vec<_>>()}).collect();
  Ok(serde_json::json!({"type":"Feature","properties":{"geoid":r.geoid,"state":state,"county":r.geoid.get(..5).ok_or("Short tract GEOID")?},"geometry":{"type":"MultiPolygon","coordinates":coordinates}}))
 }).collect::<Result<Vec<_>,String>>()?;
 let mut result=serde_json::json!({"type":"FeatureCollection","state":state,"year":year,"display_simplification_degrees":if simplified{0.0002}else{0.0},"features":features});
 if !simplified{result["geometry_profile"]=serde_json::json!("source-exact");}Ok(result)
}

#[cfg(test)]
mod census_tests {
 use super::*;
 fn record(id:&str,index:usize)->TractRecord {
  let x=-71.0+index as f64*0.01;
  let polygon=geo_types::Polygon::new(geo_types::LineString::from(vec![(x,41.0),(x+0.01,41.0),(x+0.01,41.01),(x,41.01),(x,41.0)]),vec![]);
  TractRecord{geoid:id.into(),geometry_wkb:crate::tiger::geo_to_wkb_multipolygon(&geo_types::MultiPolygon(vec![polygon])),aland:1000,awater:0,population:0}
 }
 #[test]
 fn source_coordinate_bound_counts_closing_vertices_and_all_units(){
  let records=vec![record("44001",0),record("44003",1)];
  assert!(census_source_geometry_bounded(&records,"RI","2020",9).is_err());
  let exact=census_source_geometry_bounded(&records,"RI","2020",10).unwrap();
  assert_eq!(exact,census_source_geometry(&records,"RI","2020").unwrap());
  assert_eq!(exact["display_simplification_degrees"],0.0);
 }
 #[test]
 fn selected_resolution_joins_real_block_prefixes_without_losing_population(){
  let blocks=vec![Pl94BlockPopulation{geoid:"440010001001001".into(),population:11},Pl94BlockPopulation{geoid:"440010001001002".into(),population:7},Pl94BlockPopulation{geoid:"440030002002001".into(),population:3}];
  for (resolution,ids,expected) in [(CensusResolution::County,vec!["44001","44003"],vec![18,3]),(CensusResolution::Tract,vec!["44001000100","44003000200"],vec![18,3]),(CensusResolution::BlockGroup,vec!["440010001001","440030002002"],vec![18,3]),(CensusResolution::Block,vec!["440010001001001","440010001001002","440030002002001"],vec![11,7,3])]{
   let records=ids.iter().enumerate().map(|(i,id)|record(id,i)).collect::<Vec<_>>();
   let prepared=prepare_census_graph(&records,&blocks,resolution).unwrap();assert_eq!(prepared.input_population,21);assert_eq!(prepared.graph.vertex_weights,expected);
   assert!(prepare_census_graph(&records[..records.len()-1],&blocks,resolution).is_err());
   let mut duplicate=blocks.clone();duplicate.push(blocks[0].clone());assert!(prepare_census_graph(&records,&duplicate,resolution).is_err());
   let mut negative=blocks.clone();negative[0].population=-1;assert!(prepare_census_graph(&records,&negative,resolution).is_err());
  }
 }
}
