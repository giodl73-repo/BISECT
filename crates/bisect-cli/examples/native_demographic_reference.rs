//! Independent file-adapter oracle for browser source inspection.
use std::collections::BTreeMap;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=std::env::args().collect();let file=std::path::Path::new(args.get(1).ok_or("CSV required")?);let basis=args.get(2).ok_or("Basis required")?;
 let records=if basis=="total-population"{
  let values:BTreeMap<_,_>=bisect_cli::demographics::load_demographics(file)?.into_iter().collect();
  values.into_iter().map(|(geoid,f)|serde_json::json!({"geoid":geoid,"fraction":if f.is_finite(){Some(f)}else{None},"non_finite_fraction":!f.is_finite()})).collect::<Vec<_>>()
 }else{
  let values:BTreeMap<_,_>=bisect_cli::demographics::load_vap_demographics(file)?.into_iter().collect();
  values.into_iter().map(|(geoid,c)|serde_json::json!({"geoid":geoid,"total":c.total_vap,"minority":c.minority_vap,"fraction":if c.total_vap==0.0{0.0}else{c.minority_vap/c.total_vap},"non_finite_fraction":false})).collect()
 };
 if let Some(graph_file)=args.get(3){
  let g:serde_json::Value=serde_json::from_slice(&std::fs::read(graph_file)?)?;
  let ids:Vec<String>=serde_json::from_value(g["geoids"].clone())?;
  let index:std::collections::HashMap<_,_>=ids.iter().cloned().enumerate().collect();
  let fractions=if basis=="total-population"{bisect_cli::demographics::align_demographics_to_adjacency(&bisect_cli::demographics::load_demographics(file)?,&index,ids.len())}else{let aligned=bisect_cli::demographics::align_vap_demographics_to_adjacency(&bisect_cli::demographics::load_vap_demographics(file)?,&index,ids.len());aligned.total_vap.as_ref().unwrap().iter().zip(aligned.minority_vap.as_ref().unwrap()).map(|(&t,&m)|if t==0.0{0.0}else{m/t}).collect()};
  println!("{}",serde_json::json!({"records":records,"fractions":fractions}));
 }else{println!("{}",serde_json::to_string(&records)?);}
 Ok(())
}
