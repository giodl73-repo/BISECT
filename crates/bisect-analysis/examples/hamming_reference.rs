//! Independent direct native kernel probe; no WASM wrapper or label normalization.
fn main()->Result<(),Box<dyn std::error::Error>>{
 if usize::BITS!=64{return Err("Full-u64 native probe requires a 64-bit host".into());}
 let args:Vec<_>=std::env::args().collect();let v:serde_json::Value=serde_json::from_slice(&std::fs::read(&args[1])?)?;let lag:usize=args[2].parse()?;
 let chains:Vec<Vec<Vec<usize>>>=v["chains"].as_array().ok_or("chains")?.iter().map(|chain|chain.as_array().ok_or("chain")?.iter().map(|row|row.as_array().ok_or("row")?.iter().map(|label|{let n=if let Some(n)=label.as_u64(){n}else{label.as_str().ok_or("label")?.parse::<u64>()?};Ok(n as usize)}).collect::<Result<Vec<_>,Box<dyn std::error::Error>>>()).collect::<Result<Vec<_>,Box<dyn std::error::Error>>>()).collect::<Result<Vec<_>,Box<dyn std::error::Error>>>()?;
 println!("{}",serde_json::to_string_pretty(&bisect_analysis::ensemble_diagnostics::hamming_records(&chains,lag)?)?);Ok(())
}
