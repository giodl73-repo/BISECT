//! Explicit zone assignments for the shared native co-membership weighter.
//! This boundary performs no acquisition, spatial join or zone inference.
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{HashMap,HashSet};
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {pub source_name:String,pub assignments_json:String,pub alpha:f64}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {schema_version:u32,state:String,year:String,unit_ids:Vec<String>,zone_types:Vec<String>,assignments:Vec<Vec<Option<String>>>}
pub(crate) struct Computed {pub weights:HashMap<(usize,usize),f64>,pub sha256:String,source_name:String,alpha:f64,zone_types:Vec<String>,units:usize,joint:usize,shared:usize,changed:usize}
impl Computed {
 pub(crate) fn evidence(&self,scope:&str)->Value {json!({"method":"native-zone-membership-v1","effective_weights":"zone-membership","zone_data_applied":true,"alpha":self.alpha,"zone_types":self.zone_types,"units":self.units,"source_name":self.source_name,"source_sha256":self.sha256,"jointly_observed_edges":self.joint,"shared_edges":self.shared,"changed_edges":self.changed,"formula":"boundary-times-one-plus-alpha-times-shared-over-both-available","scope":scope})}
}
pub(crate) fn for_engine(g:&crate::engine::PreparedGraph,o:&crate::engine::Options,input:Option<&Input>)->Result<Option<Computed>,String>{
 if o.weights!="zone-membership" {return if input.is_some(){Err("Zone assignments have no effect without zone-membership weights.".into())}else{Ok(None)};}
 let Some(input)=input else{return Ok(None)};
 if input.source_name.is_empty()||input.source_name.len()>200||input.source_name.chars().any(char::is_control)||input.assignments_json.is_empty()||input.assignments_json.len()>8*1024*1024||!input.alpha.is_finite()||input.alpha.is_sign_negative()||input.alpha>100.0{return Err("Zone input requires a bounded source name, JSON up to 8 MiB and finite alpha in [0,100].".into());}
 let d:Document=serde_json::from_str(&input.assignments_json).map_err(|e|format!("Zone assignments: {e}"))?;
 let valid=|s:&str,empty:bool|(empty||!s.is_empty())&&s.len()<=200&&!s.chars().any(char::is_control);
 if d.schema_version!=1||d.state!=g.state||d.year!=g.year||d.unit_ids!=g.geoids||d.assignments.len()!=g.geoids.len()||d.zone_types.len()>32||d.zone_types.iter().any(|s|!valid(s,false))||d.zone_types.iter().collect::<HashSet<_>>().len()!=d.zone_types.len()||d.assignments.iter().any(|row|row.len()>d.zone_types.len()||row.iter().flatten().any(|s|!valid(s,true))){return Err("Zone scope, exact unit order, unique zone types or bounded assignment rows differ from the graph. Use null for unknown observations; ragged rows remain unfilled.".into());}
 let pairs=g.edges.iter().map(|&(u,v,_)|(u,v)).collect::<Vec<_>>();
 let weighter=bisect_core::zone_membership::ZoneMembershipWeighter::from_zone_assignments(&d.assignments,&pairs,input.alpha);
 let weights=weighter.apply(g.edges.iter().map(|&(u,v,w)|((u,v),w)).collect());
 let mut total=0.;let mut joint=0;let mut shared=0;let mut changed=0;
 for &(u,v,w) in &g.edges {
  let adjusted=weights[&(u,v)];total+=adjusted;
  if !adjusted.is_finite()||adjusted<0.||!total.is_finite(){return Err("Zone adjusted weights exceed the native finite nonnegative range.".into());}
  if adjusted!=w{changed+=1;}
  let both=d.assignments[u].iter().zip(&d.assignments[v]).filter_map(|(a,b)|a.as_ref().zip(b.as_ref())).collect::<Vec<_>>();
  if !both.is_empty(){joint+=1;}if both.iter().any(|(a,b)|a==b){shared+=1;}
 }
 Ok(Some(Computed {weights,sha256:format!("{:x}",Sha256::digest(input.assignments_json.as_bytes())),source_name:input.source_name.clone(),alpha:input.alpha,zone_types:d.zone_types,units:g.geoids.len(),joint,shared,changed}))
}
