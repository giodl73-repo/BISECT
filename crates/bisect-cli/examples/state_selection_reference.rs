use std::collections::HashSet;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut args=std::env::args().skip(1);let year=args.next().ok_or("Expected Census year")?;
 let filter:HashSet<_>=args.map(|s|s.to_uppercase()).collect();
 let selected:Vec<_>=bisect_cli::runner::load_all_states(&year)?.into_iter().filter(|r|filter.is_empty()||filter.contains(&r.0)).collect();
 print!("{}",serde_json::to_string(&selected)?);Ok(())
}
