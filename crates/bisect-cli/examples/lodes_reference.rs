//! Native filesystem adapter oracle for both LODES preparation kernels.
fn main()->Result<(),String>{
    let args:Vec<_>=std::env::args().collect();
    if args.len()!=4{return Err("Expected wac/od, CSV source and destination paths.".into());}
    let file=std::fs::File::open(&args[2]).map_err(|e|e.to_string())?;
    let reader=std::io::BufReader::new(file);let dest=std::path::Path::new(&args[3]);
    match args[1].as_str(){"wac"=>bisect_cli::fetch::aggregate_lodes_wac_reader(reader,dest),"od"=>bisect_cli::fetch::aggregate_lodes_od_reader(reader,dest),"wac-gzip"=>bisect_cli::fetch::aggregate_lodes_wac_gzip_file(std::path::Path::new(&args[2]),dest),"od-gzip"=>bisect_cli::fetch::aggregate_lodes_od_gzip_file(std::path::Path::new(&args[2]),dest),_=>Err("Unknown LODES kind".into())}
}
