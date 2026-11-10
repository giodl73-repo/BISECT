//! Optional native subdivision step; physical graph/context edges remain unchanged.
use crate::engine::{Options,PreparedGraph};
use serde_json::{json,Value};
use std::collections::HashMap;
type EdgeMap=HashMap<(usize,usize),f64>;
pub(crate) struct Prepared{pub graph:PreparedGraph,pub evidence:Value}
pub(crate) fn validate(o:&Options)->Result<(),String>{
 if let Some(alpha)=o.county_augmentation {if !alpha.is_finite()||alpha.is_sign_negative()||alpha>100.||o.weights=="county"{return Err("County augmentation requires finite alpha in [0,100] and a non-county base weight. Use the existing county strength for county weights.".into());}}
 Ok(())
}
fn transform(g:&PreparedGraph,o:&Options,base:EdgeMap,scope:&str,order:&str)->Result<(EdgeMap,Option<Value>),String>{
 validate(o)?;let Some(alpha)=o.county_augmentation else{return Ok((base,None))};
 let county=(0..g.geoids.len()).map(|i|Some(g.county_id(i).to_owned())).collect::<Vec<_>>();let empty=vec![None;g.geoids.len()];
 let weighter=bisect_core::subdivision_weights::SubdivisionWeighter::full(county,empty.clone(),empty.clone(),empty,alpha,0.,0.,0.);
 let eligible=base.keys().filter(|&&(u,v)|g.county_id(u)==g.county_id(v)).count();let entries=base.len();
 let adjusted=weighter.apply(base.clone());let mut changed=0;let mut total=0.;
 for (&edge,&value) in &adjusted {total+=value;if !value.is_finite()||value<0.||!total.is_finite(){return Err("County augmentation exceeds the finite nonnegative weight range.".into());}if value!=base[&edge]{changed+=1;}}
 let evidence=json!({"method":"native-county-augmentation-v1","alpha":alpha,"within_county_multiplier":if alpha>1e-10{1.+alpha}else{1.},"disabled_at_or_below":1e-10,"county_identity":if g.county_ids.is_some(){"explicit-prepared-county-ids"}else{"first-five-geoid-digits"},"map_entries":entries,"eligible_edges":eligible,"changed_edges":changed,"implicit_weights":"not-materialized","order":order,"status":if alpha<=1e-10{"disabled"}else if entries==0{"no-effect-empty-map"}else if changed==0{"applied-no-edge-change"}else{"applied"},"scope":scope});
 Ok((adjusted,Some(evidence)))
}
pub(crate) fn prepare(g:&PreparedGraph,o:&Options,scope:&str)->Result<Option<Prepared>,String>{
 validate(o)?;if o.county_augmentation.is_none()||!["economic-character","housing-character","zone-membership"].contains(&o.weights.as_str()){return Ok(None);}
 let (weights,evidence)=transform(g,o,g.edges.iter().map(|&(u,v,w)|((u,v),w)).collect(),scope,"before-character-or-zone;before-coi")?;
 let mut graph=g.clone();for edge in &mut graph.edges{edge.2=weights[&(edge.0,edge.1)];}
 Ok(Some(Prepared{graph,evidence:evidence.unwrap()}))
}
pub(crate) fn apply(g:&PreparedGraph,o:&Options,base:EdgeMap,scope:&str)->Result<(EdgeMap,Option<Value>),String>{transform(g,o,base,scope,"after-base-override;before-coi")}
