//! File-based native oracle; integer populations are emitted as exact CSV text.
fn main()->Result<(),String>{
 let args:Vec<_>=std::env::args().collect();if args.len()!=4{return Err("Expected year, geography and population paths".into());}
 let blocks=bisect_data::read_pl94_block_populations_for_year(&args[2],&args[3],args[1].parse().map_err(|_|"Invalid year")?).map_err(|e|e.to_string())?;
 let mut csv=String::from("geoid,population\n");for block in blocks{csv.push_str(&format!("{},{}\n",block.geoid,block.population));}
 print!("{csv}");Ok(())
}
