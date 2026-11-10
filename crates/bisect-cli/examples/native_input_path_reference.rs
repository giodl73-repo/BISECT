fn main(){
 let args:Vec<_>=std::env::args().skip(1).collect();assert!(args.len()==4);
 std::env::set_var("BISECT_MANIFEST",&args[0]);
 match bisect_cli::runner::resolve_adjacency_path(&args[1],&args[2],&args[3]){
  Ok((path,resolution))=>println!("{}",serde_json::json!({"path":path,"resolution":resolution})),
  Err(error)=>{eprintln!("{error}");std::process::exit(1);}
 }
}
