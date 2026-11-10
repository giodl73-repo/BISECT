//! Filesystem TIGER oracle and deterministic binary fixtures.
use serde_json::{json,Value};
fn fixture(root:&std::path::Path,name:&str,field:&str,area_suffix:&str,ids:&[&str],point:bool)->Result<(),String>{
 let dir=root.join(name);std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
 let land=format!("ALAND{area_suffix}");let water=format!("AWATER{area_suffix}");
 let fields=shapefile::dbase::TableWriterBuilder::new().add_character_field(field.try_into().map_err(|e|format!("{e}"))?,15).add_numeric_field(land.as_str().try_into().map_err(|e|format!("{e}"))?,18,0).add_numeric_field(water.as_str().try_into().map_err(|e|format!("{e}"))?,18,0);
 let mut writer=shapefile::Writer::from_path(dir.join("units.shp"),fields).map_err(|e|e.to_string())?;
 let ring=|x:f64,y:f64,size:f64|vec![shapefile::Point::new(x,y),shapefile::Point::new(x,y+size),shapefile::Point::new(x+size,y+size),shapefile::Point::new(x+size,y),shapefile::Point::new(x,y)];
 for id in ids {
  let mut record=shapefile::dbase::Record::default();record.insert(field.into(),shapefile::dbase::FieldValue::Character(Some((*id).into())));record.insert(land.clone(),shapefile::dbase::FieldValue::Numeric(Some(1234.)));record.insert(water.clone(),shapefile::dbase::FieldValue::Numeric(Some(56.)));
  if point {writer.write_shape_and_record(&shapefile::Point::new(-71.,41.),&record)}else if name.ends_with("-z"){let ring_z=|x,y,size|ring(x,y,size).into_iter().map(|p|shapefile::PointZ::new(p.x,p.y,123.,456.)).collect();let poly=shapefile::PolygonZ::with_rings(vec![shapefile::PolygonRing::Outer(ring_z(-71.,41.,0.1)),shapefile::PolygonRing::Inner(ring_z(-70.98,41.02,0.02)),shapefile::PolygonRing::Outer(ring_z(-70.8,41.2,0.05))]);writer.write_shape_and_record(&poly,&record)}else {let poly=shapefile::Polygon::with_rings(vec![shapefile::PolygonRing::Outer(ring(-71.,41.,0.1)),shapefile::PolygonRing::Inner(ring(-70.98,41.02,0.02)),shapefile::PolygonRing::Outer(ring(-70.8,41.2,0.05))]);writer.write_shape_and_record(&poly,&record)}.map_err(|e|e.to_string())?;
 }drop(writer);Ok(())
}
fn main()->Result<(),String>{
 let args:Vec<_>=std::env::args().collect();
 if args.len()==3&&args[1]=="fixtures"{
  let root=std::path::Path::new(&args[2]);
  for (name,field,suffix,ids,point) in [
   ("tracts","GEOID","",vec!["44001000200","44001000100"],false),
   ("tracts-z","GEOID","",vec!["44001000100"],false),
   ("tracts-2010","GEOID10","10",vec!["44001000100"],false),
   ("tracts-2000","CTIDFP00","00",vec!["44001000100"],false),
   ("tracts-nonnumeric","GEOID","",vec!["abcdefghijk"],false),
   ("groups","GEOID","",vec!["440010002001","440010001001"],false),
   ("groups-duplicate","GEOID","",vec!["440010001001","440010001001"],false),
   ("groups-invalid","GEOID","",vec!["44001000100x"],false),
   ("tracts-invalid","GEOID","",vec!["4400100010"],false),
   ("blocks-2020","GEOID20","20",vec!["440010002001001","440010001001001"],false),
   ("blocks-z","GEOID20","20",vec!["440010001001001"],false),
   ("blocks-2010","GEOID10","10",vec!["440010001001001"],false),
   ("blocks-2000","GEOID00","00",vec!["440010001001001"],false),
   ("blocks-fallback","BLKIDFP00","00",vec!["440010001001001"],false),
   ("blocks-invalid","GEOID20","20",vec!["44001000100100x"],false),
   ("points","GEOID","",vec!["44001000100"],true),
  ]{fixture(root,name,field,suffix,&ids,point)?;}return Ok(());
 }
 if args.len()!=4{return Err("Expected fixtures root, or kind year shapefile path".into());}
 let year=args[2].parse().map_err(|_|"Invalid year")?;let path=&args[3];
 let records:Vec<Value>=match args[1].as_str(){
  "tracts"|"block-groups"=>{let units=if args[1]=="tracts"{bisect_data::read_tiger_tracts(path)}else{bisect_data::tiger::read_tiger_block_groups(path)}.map_err(|e|e.to_string())?;units.into_iter().map(|r|json!({"geoid":r.geoid,"geometry_wkb_bytes":r.geometry_wkb,"aland":r.aland.to_string(),"awater":r.awater.to_string(),"population":r.population.to_string()})).collect()},
  "blocks"=>bisect_data::read_tiger_blocks_projected_for_year(path,year).map_err(|e|e.to_string())?.into_iter().map(|r|json!({"geoid":r.geoid,"geometry_wkb_bytes":r.geometry_wkb,"centroid":r.centroid,"aland":r.aland.to_string(),"awater":r.awater.to_string()})).collect(),
  "block-centroids"=>bisect_data::read_tiger_block_centroids_projected_for_year(path,year).map_err(|e|e.to_string())?.into_iter().map(|(geoid,centroid)|json!({"geoid":geoid,"centroid":centroid})).collect(),
  _=>return Err("Unsupported kind".into()),
 };println!("{}",json!(records));Ok(())
}
