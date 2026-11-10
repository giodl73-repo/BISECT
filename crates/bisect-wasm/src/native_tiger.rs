//! Algorithm-specific native TIGER observations from selected bytes only.
use bisect_runner::native_replay_recipe::ReadOnlyRecipe;
use bisect_runner::adjacency_loader::LoadedGraph;
use std::{collections::{BTreeMap,HashSet},io::Cursor};
use serde_json::{json,Value};
pub(crate) struct Prepared{pub areas:Vec<f64>,pub perimeters:Vec<f64>,pub evidence:Value}
pub(crate) fn prepare(recipe:&ReadOnlyRecipe,graph:&LoadedGraph,selected:&BTreeMap<String,Vec<u8>>)->Result<Option<Prepared>,String>{
 if !bisect_runner::native_tiger_geometry::consumed(recipe){return Ok(None)}
 let records=bisect_runner::native_tiger_geometry::records(recipe)?;
 let bytes=|i:usize|->Result<Option<&[u8]>,String>{records[i].sha256.as_ref().map(|h|selected.get(h).map(Vec::as_slice).ok_or_else(||format!("Recorded algorithm TIGER bytes not selected: {}",records[i].role))).transpose()};
 let shp=bytes(0)?;let dbf=bytes(1)?;let shx=bytes(2)?;
 let decoded=if let Some(shp)=shp{Some((||{
  let dbf=dbf.ok_or_else(||"Native algorithm TIGER DBF is recorded absent".to_owned())?;
  bisect_data::tiger::read_tiger_units_from_streams(Cursor::new(shp),Cursor::new(dbf),shx.map(Cursor::new),false).map_err(|e|e.to_string())
 })())}else{None};
 let (areas,perimeters,status,error,missing,duplicates,record_count)=match decoded{
  None=>(vec![],vec![],"native-absent-fallback",None,vec![],0,0),
  Some(Err(error))=>(vec![],vec![],"native-warning-fallback",Some(error),vec![],0,0),
  Some(Ok(rows))=>{
   let (areas,perimeters)=bisect_runner::native_tiger_geometry::align(&rows,&graph.index_to_geoid,&graph.adjacency,&graph.edge_weights);
   let ids:HashSet<_>=rows.iter().map(|r|r.geoid.as_str()).collect();
   let missing=(0..graph.n_vertices).filter_map(|i|graph.index_to_geoid.get(&i).filter(|id|!ids.contains(id.as_str())).cloned()).collect::<Vec<_>>();
   let duplicates=rows.len()-ids.len();let count=rows.len();(areas,perimeters,"native-reader-aligned",None,missing,duplicates,count)
  }
 };
 let ordered_ids=(0..graph.n_vertices).map(|i|graph.index_to_geoid.get(&i).cloned()).collect::<Option<Vec<_>>>().ok_or("Algorithm TIGER alignment requires complete graph GEOIDs")?;
 let evidence=json!({"status":status,"error":error,"source_roles":records.iter().map(|r|&r.role).collect::<Vec<_>>(),"source_sha256":records.iter().map(|r|&r.sha256).collect::<Vec<_>>(),"observations":record_count,"duplicate_records":duplicates,"missing_unit_ids":missing,"unit_ids":ordered_ids,"land_areas_m2":areas,"native_exterior_perimeters_m":perimeters,"path_opened":false,"scope":"Selected raw native algorithm TIGER streams; native ALAND integer conversion and exact graph-GEOID alignment, missing IDs zero, duplicate tract IDs last record, missing/unreadable source warning fallback. Perimeters use the historical circular approximation and physical graph weights; CompactBisect recalculates with resolved algorithm edge weights. These observations are not measured polygon perimeters, geometry validity, upstream authenticity or a legal finding."});
 Ok(Some(Prepared{areas,perimeters,evidence}))
}
