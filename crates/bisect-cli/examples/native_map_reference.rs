use std::collections::HashMap;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().skip(1).collect();
 if args.first().is_some_and(|v|v=="--capture-bfs")&&args.len()==3{
  let graph:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&args[1])?)?;
  let adjacency:Vec<Vec<usize>>=serde_json::from_value(graph["adjacency"].clone())?;let population:Vec<i64>=serde_json::from_value(graph["population"].clone())?;
  let root=std::path::Path::new(&args[2]);std::fs::create_dir_all(root)?;
  bisect_runner::bisection_runner::run_all_splits_bfs(&adjacency,&population,2,0.5,Some(root),42)?;
  let geoids:Vec<String>=serde_json::from_value(graph["geoids"].clone())?;
  let companion:std::collections::BTreeMap<_,_>=geoids.into_iter().enumerate().map(|(i,id)|(i.to_string(),id)).collect();std::fs::write(root.join("geoids.json"),serde_json::to_string(&companion)?)?;return Ok(());
 }
 if args.first().is_some_and(|v|v=="--write-tiger")&&args.len()==3 {
  let source:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&args[1])?)?;
  let fields=shapefile::dbase::TableWriterBuilder::new().add_character_field("GEOID".try_into()?,11);
  let mut writer=shapefile::Writer::from_path(&args[2],fields)?;
  for feature in source["features"].as_array().ok_or("Missing fixture features")?{
   let geometry=&feature["geometry"];let mut rings=vec![];
   let polys=if geometry["type"]=="Polygon"{vec![&geometry["coordinates"]]}else{geometry["coordinates"].as_array().ok_or("Missing polygons")?.iter().collect()};
   for poly in polys{for (i,ring) in poly.as_array().ok_or("Missing rings")?.iter().enumerate(){let points=ring.as_array().ok_or("Missing positions")?.iter().map(|p|Ok(shapefile::Point::new(p[0].as_f64().ok_or("Missing x")?,p[1].as_f64().ok_or("Missing y")?))).collect::<Result<Vec<_>,Box<dyn std::error::Error>>>()?;rings.push(if i==0{shapefile::PolygonRing::Outer(points)}else{shapefile::PolygonRing::Inner(points)});}}
   let shape=shapefile::Polygon::with_rings(rings);let mut record=shapefile::dbase::Record::default();record.insert("GEOID".into(),shapefile::dbase::FieldValue::Character(Some(feature["properties"]["GEOID"].as_str().ok_or("Missing GEOID")?.to_string())));writer.write_shape_and_record(&shape,&record)?;
  }return Ok(());
 }
 if args.first().is_some_and(|v|v=="--tiger")&&args.len()==3 {
  let records=bisect_data::tiger::read_tiger_tracts(&args[1])?;
  let ring=|line:&geo_types::LineString<f64>|line.0.iter().map(|c|[c.x,c.y]).collect::<Vec<_>>();
  let polygon=|p:&geo_types::Polygon<f64>|std::iter::once(ring(p.exterior())).chain(p.interiors().iter().map(ring)).collect::<Vec<_>>();
  let mut features=vec![];
  for record in records {
   let shape=bisect_map::wkb_to_geometry(&record.geometry_wkb)?;
   let geometry=match shape {
    geo_types::Geometry::Polygon(p)=>serde_json::json!({"type":"Polygon","coordinates":polygon(&p)}),
    geo_types::Geometry::MultiPolygon(p)=>serde_json::json!({"type":"MultiPolygon","coordinates":p.0.iter().map(polygon).collect::<Vec<_>>()}),
    _=>return Err("Unexpected native TIGER geometry".into()),
   };
   features.push(serde_json::json!({"type":"Feature","properties":{"GEOID":record.geoid},"geometry":geometry}));
  }
  std::fs::write(&args[2],serde_json::to_string(&serde_json::json!({"type":"FeatureCollection","state":"RI","year":2020,"geometry_policy":"unsimplified-native-source-vertices","source_crs":"NAD83 geographic positions used directly for native display; no datum conversion","features":features}))?)?;return Ok(());
 }
if args.len()!=4&&args.len()!=5&&args.len()!=7&&args.len()!=9{return Err("Expected plan, geometry, DPI, output stem".into());}
 let document=rplan_io::read_rplan_str(&std::fs::read_to_string(&args[0])?)?;
 let mut geometry:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(&args[1])?)?;
 for feature in geometry["features"].as_array_mut().ok_or("Missing features")?{
  let aliases=[feature["properties"]["GEOID"].as_str(),feature["properties"]["geoid"].as_str(),feature["id"].as_str()].into_iter().flatten().collect::<Vec<_>>();
  let id=aliases.first().ok_or("Missing ID")?.to_string();
  if aliases.iter().any(|alias|*alias!=id){return Err("Conflicting geometry ID aliases".into());}
  if feature["properties"].is_null(){feature["properties"]=serde_json::json!({});}
  let properties=feature["properties"].as_object_mut().ok_or("Properties must be an object")?;
  properties.insert("GEOID".into(),serde_json::json!(id));properties.remove("geoid");
 }
 let mut assignments:HashMap<_,_>=document.plan.units.unit_ids.iter().cloned().zip(document.plan.assignment.iter().map(|d|*d as usize+1)).collect();
 if args.get(5).is_some_and(|v|v=="rounds"){
  let raw:HashMap<String,usize>=serde_json::from_str(&std::fs::read_to_string(args.get(6).ok_or("Round assignments")?)?)?;
  let basis=args.get(7).map(String::as_str).unwrap_or("unit-id");
  let index:HashMap<String,String>=if basis=="native-index-map"{serde_json::from_str(&std::fs::read_to_string(args.get(8).ok_or("Index companion")?)?)?}else{HashMap::new()};
  assignments=raw.into_iter().map(|(key,value)|{
   let unit=match basis {"unit-id"=>key,"native-index-map"=>index.get(&key).ok_or("Missing index")?.clone(),"plan-order-index"=>document.plan.units.unit_ids.get(key.parse::<usize>()?).ok_or("Index range")?.clone(),_=>return Err("Unsupported index basis".into())};Ok((unit,value))
  }).collect::<Result<HashMap<_,_>,Box<dyn std::error::Error>>>()?;
 }
 if args.get(5).is_some_and(|v|v=="rounds"){for id in &document.plan.units.unit_ids{assignments.entry(id.clone()).or_insert(0);}}
 let method:bisect_analysis::district_geometry::DissolveMethod=serde_json::from_value(serde_json::json!(args.get(4).map(String::as_str).unwrap_or("sequential")))?;
 let (districts,_)=bisect_analysis::district_geometry::read_unit_geometries_with_method(&serde_json::to_string(&geometry)?,&assignments,bisect_analysis::district_geometry::CoordinateBasis::Wgs84Degrees,method)?;
 let map_type=args.get(5).map(String::as_str).unwrap_or("districts");
 let analysis=if let Some(file)=args.get(6).filter(|_|map_type!="rounds"){serde_json::from_str(&std::fs::read_to_string(file)?)?}else{serde_json::Value::Null};
 let count=if map_type=="rounds"{districts.len()}else{document.plan.k};
 let (svg,w,h)=bisect_map::renderer::state_map_svg(districts,count,args[2].parse()?,map_type,&analysis)?;
 let png=bisect_map::svg_to_png(&svg,&bisect_map::default_font_db())?;std::fs::write(format!("{}.svg",args[3]),svg)?;std::fs::write(format!("{}.png",args[3]),png)?;println!("{w} {h}");Ok(())
}
