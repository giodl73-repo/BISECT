//! Native commission partisan file schema and in-memory assembly.
use serde::Serialize;
use crate::{DistrictElection,PartisanMetrics};
#[derive(Debug, Serialize)]
pub struct PartisanDistrictOutput {
    pub district: usize,
    pub dem_votes: f64,
    pub rep_votes: f64,
    pub dem_pct: f64,
    pub margin: f64,
    pub is_competitive: bool,
    pub is_uncontested: bool,
}

#[derive(Debug, Serialize)]
pub struct PartisanJsonOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap:Option<serde_json::Value>,
    pub analyzer: &'static str,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub election: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub election_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methodology_warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statewide: Option<StatewideStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricsBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub districts: Option<Vec<PartisanDistrictOutput>>,
}

#[derive(Debug, Serialize)]
pub struct StatewideStats {
    pub dem_vote_share: f64,
    pub dem_seat_share: f64,
    pub total_votes: f64,
}

#[derive(Debug, Serialize)]
pub struct MetricsBlock {
    pub efficiency_gap: crate::MetricWithCI,
    pub mean_median: crate::MetricWithCI,
    pub partisan_bias: crate::MetricWithCI,
}


pub fn assemble_partisan_report(districts:&[DistrictElection],metrics:&PartisanMetrics,election:String,election_file:String,methodology_warning:Option<String>)->PartisanJsonOutput {
 let methodology_warning=if (metrics.partisan_bias.value-metrics.seats_votes.bias).abs()>=0.01 {
  let warning=format!("Native consistency warning: partisan bias ({:.4}) and seats-votes bias ({:.4}) disagree by {:.4} (expected below 0.01). Possible numerical inconsistency; inspect metric conventions before interpreting these values.",metrics.partisan_bias.value,metrics.seats_votes.bias,(metrics.partisan_bias.value-metrics.seats_votes.bias).abs());
  Some(methodology_warning.map(|text|format!("{text} {warning}")).unwrap_or(warning))
 }else{methodology_warning};
 let district_outputs=districts.iter().map(|d|{
  let margin=(d.dem_votes-d.rep_votes).abs()/d.total().max(1.0);
  PartisanDistrictOutput{district:d.district,dem_votes:d.dem_votes,rep_votes:d.rep_votes,dem_pct:d.dem_pct(),margin,is_competitive:margin<0.05,is_uncontested:d.dem_votes==0.0||d.rep_votes==0.0}
 }).collect();
 PartisanJsonOutput{bootstrap:Some(serde_json::json!({"protocol":metrics.bootstrap_protocol,"seed":metrics.bootstrap_seed,"replicates":metrics.bootstrap_replicates})),analyzer:"partisan",available:true,unavailable_reason:None,required_file:None,election:Some(election),election_file:Some(election_file),methodology_warning,
 statewide:Some(StatewideStats{dem_vote_share:metrics.statewide_dem_vote_share,dem_seat_share:metrics.statewide_dem_seat_share,total_votes:districts.iter().map(DistrictElection::total).sum()}),
 metrics:Some(MetricsBlock{efficiency_gap:metrics.efficiency_gap.clone(),mean_median:metrics.mean_median.clone(),partisan_bias:metrics.partisan_bias.clone()}),districts:Some(district_outputs)}
}
