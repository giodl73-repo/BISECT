//! Independent native StateConfig electoral semantics oracle; no filesystem State run.
fn main()->Result<(),Box<dyn std::error::Error>>{
 let text=std::fs::read_to_string(std::env::args().nth(1).ok_or("Request path required")?)?;
 let v:serde_json::Value=serde_json::from_str(&text)?;
 let k=v["districts"].as_u64().ok_or("districts")? as usize;
 let population=v["population"].as_i64().ok_or("population")?;
 let settings=&v["electoral"];
 let seats=settings["seats_per_district"].as_u64().unwrap_or(1) as usize;
 let total=settings["total_seats"].as_u64().map(|n|n as usize).unwrap_or(seats*k);
 let mut cfg=bisect_cli::runner::StateConfig::new_bulk("IA".into(),"Iowa".into(),k,"2020".into(),"v1".into(),std::path::PathBuf::new(),0);
 cfg.seats_per_district=seats;cfg.total_seats=total;
 println!("{}",serde_json::json!({"seats_per_district":cfg.effective_seats_per_district(),"total_seats":cfg.total_seats,"electoral_system":if cfg.seats_per_district<=1{"single_member"}else{"multi_member_uniform"},"ideal_population_per_seat":cfg.ideal_pop_per_seat(population)}));Ok(())
}
