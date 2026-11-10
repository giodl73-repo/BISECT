//! Filesystem adapter oracle for portable ACS response preparation.
fn main()->Result<(),String>{
    let args:Vec<_>=std::env::args().collect();
    if args.len()!=3{return Err("Expected response JSON and destination CSV paths.".into());}
    let body=std::fs::read_to_string(&args[1]).map_err(|e|e.to_string())?;
    bisect_cli::fetch::write_acs_housing_body(&body,std::path::Path::new(&args[2]))
}
