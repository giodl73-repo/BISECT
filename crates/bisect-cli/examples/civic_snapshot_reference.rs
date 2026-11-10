//! Local controlled-server oracle for the native snapshot reader/writer.
fn main(){
 let args:Vec<_>=std::env::args().skip(1).collect();assert_eq!(args.len(),2);
 let client=reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(10)).redirect(reqwest::redirect::Policy::limited(3)).build().unwrap();
 std::fs::create_dir_all(&args[1]).unwrap();
 let record=bisect_cli::civic::snapshot_url_with_client(&args[0],std::path::Path::new(&args[1]),client).unwrap();
 println!("{}",serde_json::to_string(&record).unwrap());
}
