//! Explicit scoped election counts; no filesystem loading or missing-tract fill.
use crate::engine::{Options,PreparedGraph};
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};
use std::collections::BTreeMap;
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct ElectionCounts {pub democratic:f64,pub two_party:f64}
#[derive(Clone,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct ElectionInput {
    pub schema_version:String,pub state:String,pub year:String,pub election_year:String,
    pub source_label:String,pub counts:BTreeMap<String,ElectionCounts>,
}
pub(crate) struct ElectionData {pub dem:Vec<f64>,pub two_party:Vec<f64>,pub total_dem:f64,pub total_two_party:f64,pub sha256:String}
pub(crate) fn validate(g:&PreparedGraph,o:&Options,input:&ElectionInput)->Result<ElectionData,String>{
    if input.schema_version!="bisect-election-counts-v1" || input.state!=g.state || input.year!=g.year
        || input.state.len()!=2 || !input.state.bytes().all(|b|b.is_ascii_uppercase())
        || !["2000","2010","2020"].contains(&input.year.as_str())
        || input.election_year.len()!=4 || !input.election_year.bytes().all(|b|b.is_ascii_digit())
        || input.source_label.trim().is_empty() || input.source_label.len()>200 || input.counts.len()!=g.geoids.len()
        || input.counts.values().any(|c|[c.democratic,c.two_party].iter().any(|v|!v.is_finite()||v.is_sign_negative()||*v>9_007_199_254_740_991.0)||c.democratic>c.two_party) {
        return Err("Invalid election schema, scope, label, counts or complete tract coverage.".into());
    }
    let aligned=g.geoids.iter().map(|id|input.counts.get(id).ok_or_else(||"Election input omits a graph tract.".to_string())).collect::<Result<Vec<_>,_>>()?;
    let dem:Vec<_>=aligned.iter().map(|c|c.democratic).collect();
    let two_party:Vec<_>=aligned.iter().map(|c|c.two_party).collect();
    let total_dem:f64=dem.iter().sum();let total_two_party:f64=two_party.iter().sum();
    if !total_dem.is_finite() || !total_two_party.is_finite() || total_two_party<=0.0 || total_two_party>9_007_199_254_740_991.0
        || (o.structure=="proportional-section" && total_dem<=0.0)
        || (o.structure=="proportional-bisect" && dem.iter().zip(&g.population).any(|(&d,&p)|d>p as f64)) {
        return Err("Election totals must be positive finite two-party counts; Section requires positive Democratic counts; recursive votes cannot exceed census population.".into());
    }
    let mut h=Sha256::new();h.update(b"BISECT_ELECTION_COUNTS_V1\0");
    for field in [&input.state,&input.year,&input.election_year] {h.update(field.as_bytes());h.update([0]);}
    for(id,c)in &input.counts {h.update(id.as_bytes());h.update(c.democratic.to_le_bytes());h.update(c.two_party.to_le_bytes());}
    Ok(ElectionData {dem,two_party,total_dem,total_two_party,sha256:format!("{:x}",h.finalize())})
}
