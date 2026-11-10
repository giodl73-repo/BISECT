//! Independent exact-coordinate export from locally selected Census geometry.
fn main()->Result<(),String>{
 let args=std::env::args().collect::<Vec<_>>();if args.len()!=6{return Err("Expected SHP, GEOID width, state FIPS, state code and year".into());}
 let path=std::path::Path::new(&args[1]);let width=args[2].parse().map_err(|_|"Invalid GEOID width")?;
 let mut records=bisect_data::tiger::read_tiger_census_units_from_streams(std::fs::File::open(path).map_err(|e|e.to_string())?,std::fs::File::open(path.with_extension("dbf")).map_err(|e|e.to_string())?,Some(std::fs::File::open(path.with_extension("shx")).map_err(|e|e.to_string())?),width).map_err(|e|e.to_string())?;
 records.retain(|r|r.geoid.starts_with(&args[3]));println!("{}",bisect_data::tract_preparation::census_source_geometry(&records,&args[4],&args[5])?);Ok(())
}
