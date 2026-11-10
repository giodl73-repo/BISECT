fn main(){
 let args:Vec<_>=std::env::args().skip(1).collect();assert!(args.len()>=3);
 std::env::remove_var("BISECT_LOCATION_POLICY");std::env::remove_var("BISECT_STATE_POLICY");
 if let Some(path)=args.get(3){std::env::set_var("BISECT_LOCATION_POLICY",path);}
 println!("{}",bisect_cli::runner::chamber_district_count(&args[0],&args[1],args[2].parse().unwrap()));
}
