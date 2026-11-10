//! Native filesystem CSV reader probe. Run only in an isolated fixture folder.
use std::collections::BTreeMap;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let kind=std::env::args().nth(1).ok_or("Kind required")?;
 let result=if kind=="economic" {bisect_cli::lodes::load_lodes_wac_tract("fixture","2020").map(|m|m.into_iter().map(|(id,c)|(id,vec![c.commercial_intensity,c.industrial_fraction,c.jobs_per_resident])).collect::<BTreeMap<_,_>>())}else{bisect_cli::housing::load_acs_housing_tract("fixture","2020").map(|m|m.into_iter().map(|(id,c)|(id,vec![c.pct_single_family,c.pct_multifamily,c.pct_owner,c.housing_vintage])).collect::<BTreeMap<_,_>>())};
 let value=match result{Ok(m)=>serde_json::json!({"native_status":if m.is_empty(){"empty"}else{"loaded"},"records":m.into_iter().map(|(id,v)|serde_json::json!({"geoid":id,"values":v,"finite":v.iter().all(|x|x.is_finite())})).collect::<Vec<_>>()}),Err(e)=>serde_json::json!({"native_status":"error","native_error":e})};println!("{value}");Ok(())
}
