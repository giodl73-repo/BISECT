impl Options {
 pub(crate) fn effective_balance_tolerance(&self)->f64{self.balance_tolerance_fraction.unwrap_or(self.balance_tolerance/100.0)}
}
use bisect_runner::bisection_runner::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};

// Matches native SubdivisionWeighter::county_only, including its disabled-signal threshold.
pub(crate) fn county_weight(boundary: f64, alpha: f64) -> f64 {
    if alpha > 1e-10 { boundary * (1.0 + alpha) } else { boundary }
}

// Native State currently warns and retains physical geographic weights: its
// TIGER/EIA spatial join is not implemented. Do not invent zone observations.
pub(crate) fn zone_fallback_evidence(scope:&str)->Value {
    json!({"method":"native-zone-membership-fallback-v1","native_status":"spatial-join-not-implemented","effective_weights":"geographic","zone_data_applied":false,"scope":scope})
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedGraph {
    pub schema_version: u32,
    pub state: String,
    pub year: String,
    /// Omission preserves the original tract ABI and graph identities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_kind: Option<rplan_core::UnitKind>,
    /// Complete explicit county identity for precinct/imported units only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub county_ids: Option<Vec<String>>,
    /// Native unit/report identity, including ordinal fallback and optional source IDs.
    /// This is a computational identity policy, never an authenticity assertion.
    #[serde(default, skip_serializing_if = "is_false")]
    pub native_source_identity: bool,
    /// Present native GEOIDs only; absent entries use ordinal plan IDs, never source IDs.
    /// None retains the complete legacy index; Some(empty) records an absent index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_geoid_index: Option<BTreeMap<String,String>>,
    pub geoids: Vec<String>,
    pub adjacency: Vec<Vec<usize>>,
    pub population: Vec<i64>,
    /// Canonical undirected edges, finite nonnegative physical length in metres.
    /// Zero-valued native edges remain topological connections; no length is inferred.
    pub edges: Vec<(usize, usize, f64)>,
    pub areas: Vec<f64>,
    pub exterior_perimeters: Vec<f64>,
    pub centroids: Vec<(f64, f64)>,
}

pub(crate) fn validate_unit_identity(state:&str,year:&str,unit_kind:Option<rplan_core::UnitKind>,county_ids:Option<&Vec<String>>,ids:&[String])->Result<(),String>{
    let n=ids.len();
    let mut geoids = HashSet::new();
    let custom = matches!(unit_kind.unwrap_or(rplan_core::UnitKind::Tract),rplan_core::UnitKind::Precinct|rplan_core::UnitKind::Imported);
    if custom {
        if state.len()!=2||!state.bytes().all(|b|b.is_ascii_uppercase()){return Err("Custom graphs require an explicit two-letter uppercase jurisdiction code.".into());}
        if year.len()!=4||!year.bytes().all(|b|b.is_ascii_digit()){return Err("Precinct/imported graphs require an explicit four-digit unit year.".into());}
        let counties=county_ids.filter(|ids|ids.len()==n).ok_or("Precinct/imported graphs require one explicit county ID per unit.")?;
        let valid=|id:&str|!id.is_empty()&&id.len()<=200&&!id.chars().any(char::is_control)&&!["__proto__","constructor","prototype"].contains(&id);
        if ids.iter().any(|id|!valid(id)||!geoids.insert(id))||counties.iter().any(|id|!valid(id)) {
            return Err("Precinct/imported unit IDs must be unique, and unit/county IDs must contain 1-200 UTF-8 bytes without controls or reserved names.".into());
        }
    } else {
        if county_ids.is_some(){return Err("Census graphs use their canonical five-digit county prefixes; explicit county IDs apply only to precinct/imported graphs.".into());}
        let width=match unit_kind.unwrap_or(rplan_core::UnitKind::Tract){rplan_core::UnitKind::County=>5,rplan_core::UnitKind::Tract=>11,rplan_core::UnitKind::BlockGroup=>12,rplan_core::UnitKind::Block=>15,_=>unreachable!()};
        if ids.iter().any(|id|id.len()!=width||!id.bytes().all(|b|b.is_ascii_digit())||!geoids.insert(id)) {
            return Err(format!("GEOIDs must be unique {width}-digit strings matching the declared unit kind."));
        }
        if unit_kind.is_some(){
            let fips=crate::tract_input::import_scope(state,year,"declared Census graph")?;
            if ids.iter().any(|id|!id.starts_with(fips)){return Err("Declared Census graph contains foreign-state GEOIDs.".into());}
        }
    }
    Ok(())
}

fn is_false(value:&bool)->bool{!*value}
fn validate_native_source_identity(g:&PreparedGraph)->Result<(),String>{
 if g.county_ids.is_some(){return Err("Native source identity retains native report prefixes without explicit county overrides.".into());}
 if g.state.len()!=2||!g.state.bytes().all(|b|b.is_ascii_uppercase())||g.year.len()!=4||!g.year.bytes().all(|b|b.is_ascii_digit()){return Err("Native source identity requires explicit state and four-digit year.".into());}
 let units=rplan_core::PlanUnitIndex{unit_kind:g.effective_unit_kind(),state:Some(g.state.clone()),year:g.year.parse().ok(),canonical_order:rplan_core::CanonicalOrder::ExplicitUnitIds,unit_ids:g.geoids.clone(),unit_universe_hash:String::new(),source_id:None};
 units.validate().map_err(|error|format!("Native unit format: {error}"))?;
 // Native adjacency overrides do not assert a GEOID's membership in the state label.
 if let Some(index)=&g.native_geoid_index {
  if index.keys().any(|i|i.parse::<u64>().ok().is_none_or(|n|n>=g.geoids.len() as u64||n.to_string()!=*i)){return Err("Native GEOID source index contains an out-of-range vertex.".into());}
  for (i,id) in g.geoids.iter().enumerate(){
   let expected=index.get(&i.to_string()).cloned().unwrap_or_else(||i.to_string());
   if id!=&expected{return Err("Native plan IDs must retain the original source index or the native ordinal fallback.".into());}
  }
 }
 let mut ids=HashSet::new();
 if g.geoids.iter().any(|id|id.is_empty()||id.len()>200||id.chars().any(char::is_control)||["__proto__","constructor","prototype"].contains(&id.as_str())||!ids.insert(id)){return Err("Native imported unit IDs must be unique, bounded safe strings.".into());}Ok(())
}
impl PreparedGraph {
    pub fn native_source_geoid(&self,index:usize)->Option<&str>{
        match &self.native_geoid_index {Some(ids)=>ids.get(&index.to_string()).map(String::as_str),None=>Some(self.geoids[index].as_str())}
    }
    pub fn native_source_index(&self)->HashMap<usize,String>{
        (0..self.geoids.len()).filter_map(|i|self.native_source_geoid(i).map(|id|(i,id.to_owned()))).collect()
    }
    pub fn report_county_id(&self,index:usize)->Option<&str>{
        if self.native_source_identity {
            let id=self.native_source_geoid(index)?;
            id.as_bytes().get(..5).filter(|prefix|prefix.iter().all(u8::is_ascii_digit)).map(|_|&id[..5])
        }else{Some(self.county_id(index))}
    }
    pub fn county_id(&self, index: usize) -> &str { self.county_ids.as_ref().map(|ids|ids[index].as_str()).unwrap_or_else(||&self.geoids[index][..5]) }
    pub fn effective_unit_kind(&self) -> rplan_core::UnitKind { self.unit_kind.unwrap_or(rplan_core::UnitKind::Tract) }
}

// Preserve legacy numeric seeds where JS can represent them exactly.
fn seed_json(seed:u64)->Value {
    if seed<=9_007_199_254_740_991 {json!(seed)} else {json!(seed.to_string())}
}
mod seed_serde {
    use serde::{de::{self,Visitor},Deserializer,Serialize,Serializer};
    use std::fmt;
    pub fn serialize<S:Serializer>(seed:&Option<u64>,serializer:S)->Result<S::Ok,S::Error>{
        seed.map(super::seed_json).serialize(serializer)
    }
    pub fn deserialize<'de,D:Deserializer<'de>>(deserializer:D)->Result<Option<u64>,D::Error>{
        struct Seed;
        impl<'de> Visitor<'de> for Seed {
            type Value=Option<u64>;
            fn expecting(&self,f:&mut fmt::Formatter)->fmt::Result{f.write_str("a u64 integer or canonical decimal u64 string")}
            fn visit_unit<E:de::Error>(self)->Result<Option<u64>,E>{Ok(None)}
            fn visit_none<E:de::Error>(self)->Result<Option<u64>,E>{Ok(None)}
            fn visit_u64<E:de::Error>(self,value:u64)->Result<Option<u64>,E>{Ok(Some(value))}
            fn visit_str<E:de::Error>(self,value:&str)->Result<Option<u64>,E>{
                if value.is_empty() || value.len()>20 || !value.bytes().all(|b|b.is_ascii_digit()) || (value.len()>1 && value.starts_with('0')) {
                    return Err(E::custom("Seed must be a canonical decimal u64."));
                }
                value.parse().map(Some).map_err(|_|E::custom("Seed exceeds u64 range."))
            }
        }
        deserializer.deserialize_any(Seed)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    /// Explicit backend; omission retains metis-core.
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub metis_engine:Option<String>,
    /// C sidecar runtime identity; required for c-ffi, absent for metis-core.
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub metis_runtime:Option<String>,
    /// Optional native county composition; independent of the legacy county weight selector.
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub county_augmentation:Option<f64>,
    /// Native State/States configuration filter; omission retains open behavior.
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub profile:Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metis_ufactor: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub capture_rounds:Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub capture_ilp_reports:Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub convergence_unlimited: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_alpha: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_absent_file_fallback: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proportional_eta: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metis_objective: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dem_threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rep_threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metis_trials: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub w_vra: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vra_absent_file_fallback: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vra_threshold: Option<f64>,
    pub structure: String,
    pub weights: String,
    pub search: String,
    pub districts: usize,
    #[serde(default,with="seed_serde")]
    pub seed: Option<u64>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub ilp_method:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub ilp_fallback:Option<String>,
    #[serde(default,with="seed_serde",skip_serializing_if="Option::is_none")]
    pub ilp_time_limit:Option<u64>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub ilp_gap:Option<f64>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub ilp_max_tracts:Option<usize>,
    pub seeds: usize,
    pub steps: usize,
    pub percentile: f64,
    pub alpha_county: f64,
    pub balance_tolerance: f64,
    /// Optional original native fraction; percent remains the display value.
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub balance_tolerance_fraction: Option<f64>,
    pub area_swing: f64,
    pub iterations: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sa_steps_per_tract: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sa_t0_factor: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sa_t_final: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_repair: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smc_particles: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smc_resample_threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cvd_iters: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cvd_metric: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mka_orientations: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mka_metric: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact_epsilon: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact_geometry: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_init: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub burst_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_bursts: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pt_replicas: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pt_swap_interval: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pt_cold_tol: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pt_hot_tol: Option<f64>,
}

/// Explicit per-tract fractions; no missing-tract or absent-file zero fill.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DemographicInput {
    pub schema_version: String,
    pub state: String,
    pub year: String,
    pub basis: String,
    pub source_label: String,
    pub minority_fractions: BTreeMap<String, f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counts: Option<BTreeMap<String, bisect_data::demographics::DemographicCounts>>,
}

pub(crate) fn demographic_fractions(g: &PreparedGraph, input: &DemographicInput) -> Result<(Vec<f64>, String), String> {
    if !["bisect-demographic-fractions-v1", "bisect-demographic-counts-v2"].contains(&input.schema_version.as_str()) || input.state != g.state || input.year != g.year
        || input.state.len() != 2 || !input.state.bytes().all(|b| b.is_ascii_uppercase()) || input.year.len() != 4 || !input.year.bytes().all(|b| b.is_ascii_digit())
        || !["total-population", "voting-age-population", "citizen-voting-age-population"].contains(&input.basis.as_str())
        || input.source_label.is_empty() || input.source_label.len() > 200 || input.minority_fractions.len() != g.geoids.len()
        || input.minority_fractions.values().any(|v| !v.is_finite() || v.is_sign_negative() || *v > 1.0) {
        return Err("Invalid demographic schema, scope, basis, label, coverage or fractions.".into());
    }
    match (&input.counts,input.schema_version.as_str()) {
        (None,"bisect-demographic-fractions-v1") => {},
        (Some(counts),"bisect-demographic-counts-v2") if counts.len()==input.minority_fractions.len() => {
            for (id,fraction) in &input.minority_fractions {
                let c=counts.get(id).ok_or("Demographic count coverage mismatch.")?;
                if !c.total.is_finite() || !c.minority.is_finite() || c.total.is_sign_negative() || c.minority.is_sign_negative() || c.total>9_007_199_254_740_991.0 || c.minority>c.total || *fraction!=if c.total==0.0 {0.0} else {c.minority/c.total} {return Err("Demographic counts and fractions disagree.".into());}
            }
        },
        _ => return Err("Demographic counts require the counts-v2 schema and complete coverage.".into()),
    }
    let fractions = g.geoids.iter().map(|id| input.minority_fractions.get(id).copied().ok_or_else(|| "Demographic input omits a graph tract.".to_string())).collect::<Result<Vec<_>, _>>()?;
    // The operational identity is independent of JSON float spelling and map
    // insertion order. Source labels remain user-provided descriptions.
    let mut h = Sha256::new();h.update(b"BISECT_DEMOGRAPHICS_V1\0");
    h.update(input.state.as_bytes());h.update([0]);h.update(input.year.as_bytes());h.update([0]);h.update(input.basis.as_bytes());h.update([0]);
    for (id, value) in &input.minority_fractions { h.update(id.as_bytes());h.update(value.to_le_bytes()); }
    Ok((fractions, format!("{:x}", h.finalize())))
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PartisanInput {
    pub schema_version: String,
    pub state: String,
    pub year: String,
    pub source_label: String,
    pub dem_shares: BTreeMap<String,f64>,
}

pub(crate) fn partisan_shares(g:&PreparedGraph,input:&PartisanInput)->Result<(Vec<f64>,String),String>{
    if input.schema_version!="bisect-partisan-shares-v1" || input.state!=g.state || input.year!=g.year
        || input.state.len()!=2 || !input.state.bytes().all(|b|b.is_ascii_uppercase())
        || input.year.len()!=4 || !input.year.bytes().all(|b|b.is_ascii_digit())
        || input.source_label.is_empty() || input.source_label.len()>200 || input.dem_shares.len()!=g.geoids.len()
        || input.dem_shares.values().any(|v|!v.is_finite()||v.is_sign_negative()||*v>1.0){
        return Err("Invalid partisan schema, scope, source label, shares or tract coverage.".into());
    }
    let shares=g.geoids.iter().map(|id|input.dem_shares.get(id).copied().ok_or_else(||"Partisan input omits a graph tract.".to_string())).collect::<Result<Vec<_>,_>>()?;
    let mut h=Sha256::new();h.update(b"BISECT_PARTISAN_SHARES_V1\0");h.update(input.state.as_bytes());h.update([0]);h.update(input.year.as_bytes());h.update([0]);
    for(id,value)in &input.dem_shares {h.update(id.as_bytes());h.update(value.to_le_bytes());}
    Ok((shares,format!("{:x}",h.finalize())))
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(default,skip_serializing_if="Option::is_none")]pub native_weighting:Option<bisect_runner::state_config::WeightSpec>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub(crate) native_auxiliary:Option<crate::native_auxiliary::Input>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub zone_membership:Option<crate::zone_input::Input>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub native_character_source:Option<crate::native_character_source::Input>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub electoral:Option<crate::electoral_input::Input>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub coi:Option<crate::coi_input::Input>,
    #[serde(default)]
    pub character: Option<crate::character_input::CharacterInput>,
    #[serde(default)]
    pub elections: Option<crate::election_input::ElectionInput>,
    #[serde(default)]
    pub partisan: Option<PartisanInput>,
    #[serde(default)]
    pub demographics: Option<DemographicInput>,
    pub graph: PreparedGraph,
    pub options: Options,
}

pub(crate) fn validate_profile(o:&Options,partisan:bool,search:Option<&str>)->Result<(),String>{
 let profile=o.profile.as_deref().unwrap_or("open");
 if !matches!(profile,"open"|"court"|"bulletproof"){return Err("Run profile must be open or court (alias bulletproof).".into());}
 bisect_runner::run_profile::validate(profile!="open",if o.weights=="partisan"{"partisan-weighted"}else{"edge-weighted"},false,search.or(Some(&o.search)),partisan)
}
pub(crate) fn validate(g: &PreparedGraph, o: &Options) -> Result<(), String> {
    crate::metis_backend::validate(o)?;
    validate_profile(o,false,None)?;
    if o.character_absent_file_fallback.is_some() && (o.character_absent_file_fallback!=Some(true) || !["economic-character","housing-character"].contains(&o.weights.as_str())) {return Err("Character absent-source fallback applies only to character weights and must be explicitly true.".into());}
    if o.vra_absent_file_fallback.is_some() && ((o.structure!="ratio-optimal-vra"&&o.search!="vra-recom") || o.weights=="vra-aligned" || o.vra_absent_file_fallback!=Some(true)) {return Err("Native absent-file fallback requires VRASection or VRA ReCom, a non-demographic objective and an explicit true choice.".into());}
    if o.convergence_unlimited.is_some() && o.search != "convergence" {
        return Err("Unlimited convergence applies only to convergence search.".into());
    }
    let character=["economic-character","housing-character"].contains(&o.weights.as_str());
    if character {
        if !o.character_alpha.is_some_and(|v|v.is_finite()&&!v.is_sign_negative()&&v<=1.) {return Err("Character weights require a finite blend alpha in [0,1].".into());}
    } else if o.character_alpha.is_some() {return Err("Character blend alpha has no effect without character weights.".into());}
    if o.structure=="proportional-section" {
        if o.proportional_eta.is_some_and(|v|!v.is_finite()||!(1.0..=2.0).contains(&v)){return Err("ProportionalSection eta must be finite in [1,2].".into());}
    } else if o.proportional_eta.is_some(){return Err("Vote constraint eta applies only to ProportionalSection.".into());}
    if o.metis_ufactor.is_some() && o.structure!="nway" {return Err("Separate METIS imbalance factor is supported only by native direct nway partitioning.".into());}
    if o.metis_objective.is_some() || o.metis_trials.is_some() {
        if !(o.structure=="nway" || (["ratio-optimal","ratio-optimal-area","ratio-optimal-vra"].contains(&o.structure.as_str())&&["single","multi"].contains(&o.search.as_str())) || (o.structure=="standard-bisect"&&["single","multi","percentile","convergence","bisection-ensemble"].contains(&o.search.as_str()))) || o.metis_objective.as_ref().is_some_and(|v| !["cut","volume"].contains(&v.as_str())) || o.metis_trials.is_some_and(|v| !(1..=100).contains(&v)) {return Err("METIS objective/trials require nway, Geo/Area/VRASection single/multi, or standard single/multi/percentile/convergence/ensemble bisection, cut or volume, and 1..=100 trials.".into());}
    }
    if o.weights=="partisan" {
        if !(o.structure=="standard-bisect"&&["single","multi","percentile","convergence","bisection-ensemble","flip","forest-recom","merge-split","short-burst","short-burst-forest","short-burst-merge-split","parallel-tempering","smc-percentile","vra-recom"].contains(&o.search.as_str())||["ratio-optimal","ratio-optimal-area","ratio-optimal-vra","compact-polsby","proportional-section"].contains(&o.structure.as_str())&&["single","multi"].contains(&o.search.as_str())||["spectral","nway","simulated-annealing","bfs-growth","centroidal-voronoi","moving-knife","capacity-clustering","regionalization","flow-construction","prime-factor","ilp","proportional-bisect"].contains(&o.structure.as_str())&&o.search=="single") || o.dem_threshold.is_none() || o.rep_threshold.is_none()
            || !o.dem_threshold.is_some_and(|v|v.is_finite()&&(0.0..=1.0).contains(&v)) || !o.rep_threshold.is_some_and(|v|v.is_finite()&&(0.0..=1.0).contains(&v)) || o.dem_threshold<o.rep_threshold {
            return Err("Partisan weights require standard Single/Multi/percentile/convergence/local-ensemble/Flip/forest-ReCom/merge-split/short-burst/parallel-tempering/SMC/VRA-ReCom searches, GeoSection/AreaSection/VRASection/Compact Polsby/ProportionalSection Single/Multi, or Spectral/NWay/simulated-annealing/BFS/CVD/moving-knife/clustering/regionalization/flow/prime-factor/ILP/Proportional Bisect Single and finite ordered thresholds in [0,1].".into());
        }
    } else if o.dem_threshold.is_some() || o.rep_threshold.is_some() {return Err("Partisan thresholds have no effect without partisan weights.".into());}
    crate::county_augmentation::validate(o)?;
    let n = g.geoids.len();
    if g.schema_version != 1
        || n == 0
        || n > 1_000_000
        || g.population.len() != n
        || g.adjacency.len() != n
    {
        return Err("Invalid prepared graph schema or vertex dimensions.".into());
    }
    if g.native_geoid_index.is_some()&&!g.native_source_identity{return Err("Native source indexes require the explicit native source identity policy.".into());}
    if g.native_source_identity {
        validate_native_source_identity(g)?;
        if o.weights!="native-composed"||o.county_augmentation.is_some(){return Err("Native source identity requires the shared native weighting compositor without prepared county overrides.".into());}
    }else{validate_unit_identity(&g.state,&g.year,g.unit_kind,g.county_ids.as_ref(),&g.geoids)?;}
    if g.population.iter().any(|&p| p < 0 || p > i32::MAX as i64)
        || g.population.iter().sum::<i64>() == 0
    {
        return Err("Population weights are invalid.".into());
    }
    let mut arcs = HashSet::new();
    for (u, neighbors) in g.adjacency.iter().enumerate() {
        for &v in neighbors {
            if v >= n || u == v || !arcs.insert((u, v)) {
                return Err("Invalid adjacency endpoint or duplicate.".into());
            }
        }
    }
    if arcs.iter().any(|&(u, v)| !arcs.contains(&(v, u))) {
        return Err("Adjacency must be symmetric.".into());
    }
    let mut edges = HashSet::new();
    for &(u, v, w) in &g.edges {
        if u >= v || !arcs.contains(&(u, v)) || !edges.insert((u, v)) || !w.is_finite() || w < 0.0
        {
            return Err("Invalid physical boundary edge.".into());
        }
    }
    if edges.len() * 2 != arcs.len() {
        return Err("Every adjacency edge needs a physical weight.".into());
    }
    for values in [&g.areas, &g.exterior_perimeters] {
        if (!values.is_empty() && values.len() != n)
            || values.iter().any(|v| !v.is_finite() || *v < 0.0)
            || !values.iter().sum::<f64>().is_finite()
        {
            return Err("Invalid graph geometry attributes.".into());
        }
    }
    if !g
        .edges
        .iter()
        .map(|&(u, v, w)| {
            if o.weights == "county" && g.county_id(u) == g.county_id(v) {
                county_weight(w, o.alpha_county)
            } else {
                w
            }
        })
        .sum::<f64>()
        .is_finite()
    {
        return Err("Physical/weighted boundary totals exceed finite numeric range.".into());
    }
    if (!g.centroids.is_empty() && g.centroids.len() != n)
        || g.centroids
            .iter()
            .any(|&(x, y)| !x.is_finite() || !y.is_finite() || x.abs() > 180.0 || y.abs() > 90.0)
    {
        return Err("Invalid longitude/latitude centroids.".into());
    }
    if o.structure=="ratio-optimal-area" && g.areas.len()!=n {
        return Err("AreaSection requires complete land areas; native inputs without geometry cannot run this method.".into());
    }
    if (o.structure=="moving-knife" || o.structure=="centroidal-voronoi" && o.cvd_metric.as_deref()!=Some("graph-distance")) && g.centroids.len()!=n {
        return Err("This method requires complete longitude/latitude centroids; choose a method not requiring them or select the companion source.".into());
    }
    if !(1..=n.min(500)).contains(&o.districts)
        || !((if o.search == "parallel-tempering" && o.pt_replicas.is_some() {0} else {1})..=10000).contains(&o.seeds)
        || !((if ["forest-recom", "merge-split", "flip", "bisection-ensemble", "vra-recom"].contains(&o.search.as_str()) {0} else {1})..=100000).contains(&o.steps)
        || !(1..=1000).contains(&o.iterations)
        || !o.balance_tolerance.is_finite()
        || !(0.01..=50.0).contains(&o.balance_tolerance)
        || o.balance_tolerance_fraction.is_some_and(|t| !t.is_finite() || !(0.0001..=0.5).contains(&t) || (t*100.0).to_bits()!=o.balance_tolerance.to_bits())
        || !o.percentile.is_finite()
        || !(0.0..=1.0).contains(&o.percentile)
        || !o.alpha_county.is_finite()
        || !(0.0..=100.0).contains(&o.alpha_county)
        || !o.area_swing.is_finite()
        || !(1.0..=2.0).contains(&o.area_swing)
    {
        return Err("Invalid engine option bounds.".into());
    }
    if !["geographic", "unweighted", "county", "partisan", "vra-aligned", "economic-character", "housing-character", "zone-membership", "native-composed"].contains(&o.weights.as_str())
        || (o.weights != "county" && o.alpha_county != 0.0)
    {
        return Err("Invalid boundary weights.".into());
    }
    if o.capture_ilp_reports==Some(true)&&o.structure!="ilp" {return Err("ILP report capture requires native ILP structure.".into());}
    if o.structure=="ilp" {
        if o.ilp_method.as_ref().is_some_and(|v|!["formulation-only","branch-and-cut","iterative-separation"].contains(&v.as_str()))||o.ilp_fallback.as_ref().is_some_and(|v|!["metis","error"].contains(&v.as_str()))||o.ilp_gap.is_some_and(|v|!v.is_finite()||v<0.)||o.ilp_max_tracts.is_some_and(|n|n>1_000_000){return Err("Invalid native ILP dispatch settings.".into());}
    }else if o.ilp_method.is_some()||o.ilp_fallback.is_some()||o.ilp_time_limit.is_some()||o.ilp_gap.is_some()||o.ilp_max_tracts.is_some(){return Err("ILP settings apply only to native ILP dispatch.".into());}
    if o.structure == "simulated-annealing" {
        if o.sa_steps_per_tract.is_some_and(|steps| steps > 10000)
            || o.sa_t0_factor.is_some_and(|v| !v.is_finite() || !(0.0..=1000.0).contains(&v))
            || o.sa_t_final.is_some_and(|v| !v.is_finite() || !(1e-15..=1e6).contains(&v)) {
            return Err("Invalid simulated annealing parameters.".into());
        }
    } else if o.sa_steps_per_tract.is_some() || o.sa_t0_factor.is_some() || o.sa_t_final.is_some() {
        return Err("Annealing parameters apply only to simulated annealing.".into());
    }
    if o.structure == "centroidal-voronoi" {
        if o.cvd_iters.is_some_and(|n| n>10000) || o.cvd_metric.as_ref().is_some_and(|metric| !["geographic","graph-distance"].contains(&metric.as_str())) {
            return Err("Invalid CVD iteration budget or distance metric.".into());
        }
    } else if o.cvd_iters.is_some() || o.cvd_metric.is_some() {
        return Err("CVD parameters apply only to centroidal Voronoi.".into());
    }
    if o.structure == "moving-knife" {
        if o.mka_orientations.is_some_and(|n| n>10000) || o.mka_metric.as_ref().is_some_and(|metric| !["reock","polsby"].contains(&metric.as_str())) {
            return Err("Invalid moving-knife orientation budget or scoring metric.".into());
        }
    } else if o.mka_orientations.is_some() || o.mka_metric.is_some() {
        return Err("Moving-knife parameters apply only to moving knife.".into());
    }
    if o.pt_replicas.is_some() || o.pt_swap_interval.is_some() || o.pt_cold_tol.is_some() || o.pt_hot_tol.is_some() {
        if o.structure != "standard-bisect" || o.search != "parallel-tempering"
            || !o.pt_replicas.is_some_and(|n| (1..=32).contains(&n))
            || !o.pt_swap_interval.is_some_and(|n| (1..=100000).contains(&n))
            || !o.pt_cold_tol.is_some_and(|v| v.is_finite() && (0.0001..=1.0).contains(&v))
            || !o.pt_hot_tol.is_some_and(|v| v.is_finite() && (o.pt_cold_tol.unwrap_or(1.0)..=1.0).contains(&v))
            || o.pt_replicas.unwrap().checked_mul(o.seeds).is_none_or(|n| n>100000) {
            return Err("Invalid parallel-tempering replicas, swap interval or tolerance ladder.".into());
        }
    }
    if o.burst_length.is_some() || o.n_bursts.is_some() {
        if o.structure != "standard-bisect" || !["short-burst", "short-burst-forest", "short-burst-merge-split"].contains(&o.search.as_str())
            || !o.burst_length.is_some_and(|n| n <= 100000) || !o.n_bursts.is_some_and(|n| n <= 10000)
            || o.burst_length.unwrap().checked_mul(o.n_bursts.unwrap()).is_none_or(|n| n > 100000) {
            return Err("Invalid short-burst length, count or total work budget.".into());
        }
    }
    if let Some(init) = &o.area_init {
        if o.structure != "ratio-optimal-area" || !["ratio-optimal", "moving-knife"].contains(&init.as_str()) {
            return Err("Invalid AreaSection initialization strategy.".into());
        }
    }
    if o.structure == "compact-polsby" {
        if o.compact_geometry.as_ref().is_some_and(|v| !["prepared", "native-state-circular"].contains(&v.as_str())) {
            return Err("Choose prepared or native-state-circular CompactBisect geometry.".into());
        }
        if o.compact_epsilon.is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v)) {
            return Err("Invalid CompactBisect edge-cut slack.".into());
        }
    } else if o.compact_epsilon.is_some() || o.compact_geometry.is_some() {
        return Err("CompactBisect slack applies only to compact-polsby.".into());
    }
    if let Some(repair) = &o.flow_repair {
        if o.structure != "flow-construction" || !["none", "bfs"].contains(&repair.as_str()) {
            return Err("Flow repair applies only to flow construction and must be none or bfs.".into());
        }
    }
    if o.search == "smc-percentile" {
        if !o.smc_particles.is_some_and(|n| (1..=10000).contains(&n))
            || !o.smc_resample_threshold.is_some_and(|v| v.is_finite() && (0.0..=1.0).contains(&v))
             {
            return Err("SMC requires valid particle and resampling parameters.".into());
        }
    } else if o.smc_particles.is_some() || o.smc_resample_threshold.is_some() {
        return Err("SMC parameters apply only to SMC percentile search.".into());
    }
    if o.search == "vra-recom" {
        if !o.vra_threshold.is_some_and(|v| v.is_finite() && (0.0..=1.0).contains(&v)) {
            return Err("VRA ReCom requires a valid fraction threshold.".into());
        }
    } else if o.vra_threshold.is_some() { return Err("VRA threshold applies only to VRA ReCom.".into()); }
    if o.structure=="ratio-optimal-vra" {
        if !o.w_vra.is_some_and(|v|v.is_finite()&&(0.0..=1.0).contains(&v)){return Err("VRASection requires an alignment weight between zero and one.".into());}
    } else if o.w_vra.is_some(){return Err("Alignment weight applies only to VRASection.".into());}
    let supported = match o.structure.as_str() {
        "standard-bisect" => [
            "vra-recom",
            "smc-percentile",
            "convergence",
            "single",
            "multi",
            "percentile",
            "bisection-ensemble",
            "short-burst",
            "short-burst-forest",
            "short-burst-merge-split",
            "flip",
            "forest-recom",
            "merge-split",
            "parallel-tempering",
        ]
        .as_slice(),
        "proportional-section" | "ratio-optimal" | "ratio-optimal-area" | "ratio-optimal-vra" | "compact-polsby" => ["single", "multi"].as_slice(),
        "ilp" | "proportional-bisect" | "simulated-annealing" | "capacity-clustering" | "regionalization" | "flow-construction" | "spectral" | "prime-factor" | "nway" | "bfs-growth" | "centroidal-voronoi" | "moving-knife" => ["single"].as_slice(),
        _ => return Err("Structure has not yet been wired into the browser boundary.".into()),
    };
    if !supported.contains(&o.search.as_str()) {
        return Err("Structure does not execute this search.".into());
    }
    Ok(())
}

pub fn execute(request:Request)->Result<Value,String>{
    if request.options.capture_ilp_reports==Some(true){return crate::ilp_inspection::execute_captured(request);}
    execute_observed(request,None)
}
pub(crate) fn emit_round_monitor(event:&bisect_runner::bisection_runner::RoundMonitorEvent)->Result<(),String>{
    crate::progress::emit(&json!({"schema_version":"bisect-native-round-monitor-v1","phase":event.phase,"name":event.name,"index":event.index.to_string(),"notified":event.notified.to_string(),"omitted":(event.index-event.notified).to_string(),"units":event.units,"assigned_units":event.assigned_units,"invalid_entries":event.invalid_entries.to_string(),"invalid_name":event.invalid_name,"complete_universe":event.complete_universe})).map_err(|_|"Host could not retain native execution progress.".into())
}
pub(crate) fn emit_round_progress(name:&str,index:usize,assigned_units:usize,units:usize)->Result<(),String>{
    crate::progress::emit(&json!({"schema_version":"bisect-native-round-progress-v1","name":name,"index":index,"assigned_units":assigned_units,"units":units,"complete_universe":assigned_units==units})).map_err(|_|"Host could not retain native round progress.".into())
}
pub(crate) fn execute_observed(request:Request,observer:Option<&IlpNodeObserver<'_>>)->Result<Value,String>{
    let _backend=crate::metis_backend::enter(&request.options)?;
    let enabled=request.options.capture_rounds==Some(true);
    let capture=if enabled{Some(RoundCapture::start_observed(request.graph.geoids.len(),Some(emit_round_progress))?)}else{None};
    let monitor=if !enabled{Some(bisect_runner::bisection_runner::RoundMonitor::start(request.graph.geoids.len(),emit_round_monitor)?)}else{None};
    let geoids=if enabled{request.graph.geoids.clone()}else{vec![]};
    let mut result=execute_inner(request,observer)?;
    if let Some(monitor)=monitor{monitor.finish()?;}
    if let Some(capture)=capture {
        finish_round_capture(capture,&geoids,&mut result,None)?;
    }
    Ok(result)
}
// Multiscale native callsites describe the tract initializer, even for BG output.
// Compare its projection to every returned BG; never collapse split tracts.
pub(crate) fn finish_round_capture(capture:RoundCapture,geoids:&[String],result:&mut Value,projected_units:Option<&[String]>)->Result<(),String>{
        let unit_index:BTreeMap<_,_>=geoids.iter().enumerate().map(|(i,id)|(i.to_string(),id.clone())).collect();
        let parent_indices:HashMap<_,_>=geoids.iter().enumerate().map(|(i,id)|(id.as_str(),i)).collect();
        let frames=capture.finish()?;
        let index_json=serde_json::to_string(&unit_index).map_err(|e|e.to_string())?;
        let mut events=vec![];
        for (sequence,frame) in frames.into_iter().enumerate(){
            let assignments_json=serde_json::to_string(&frame.assignments).map_err(|e|e.to_string())?;
            let terminal_matches=frame.complete_universe&&if let Some(units)=projected_units {
                units.iter().all(|id|id.get(..11).and_then(|parent|parent_indices.get(parent)).and_then(|i|frame.assignments.get(i)).is_some_and(|region|result["assignments"][id].as_u64()==Some(*region as u64)))
            }else{frame.assignments.iter().all(|(i,region)|result["assignments"][&geoids[*i]].as_u64()==Some(*region as u64))};
            events.push(json!({"sequence":sequence,"name":frame.name,"complete_universe":frame.complete_universe,"units":frame.assignments.len(),"regions":frame.assignments.values().collect::<HashSet<_>>().len(),"matches_returned_assignment":terminal_matches,"assignments_json":assignments_json,"assignments_sha256":format!("{:x}",Sha256::digest(assignments_json.as_bytes()))}));
        }
        let terminal_matches=events.last().is_some_and(|e|e["matches_returned_assignment"]==true);
        result["round_capture"]=json!({"schema_version":"bisect-native-round-capture-v1","status":if events.is_empty(){"no-events-emitted"}else{"captured"},"unit_index_json":index_json,"unit_index_sha256":format!("{:x}",Sha256::digest(index_json.as_bytes())),"events":events,"terminal_matches_returned_assignment":terminal_matches,"scope":"Ordered snapshots emitted at shared native intermediate-round callsites during this engine execution. No filesystem operations are used for capture. Events may include search candidates or partial coverage; matches_returned_assignment compares exact one-based labels and does not certify the full sequence or lineage. Repeated native names are retained as separate events. Empty capture means this execution emitted no native round snapshots; it does not imply a complete one-round history. Index companion uses the exact prepared graph unit order. Bounds: 128 events and two million total assignment entries. Capture failures reject this execution without returning a truncated trace."});
    Ok(())
}
fn execute_inner(request: Request,observer:Option<&IlpNodeObserver<'_>>) -> Result<Value, String> {
    crate::native_composed::validate_request(&request)?;

    let Request {
        graph: g,
        options: o,
        demographics,
        partisan,
        elections,
        character,
        native_character_source,
        native_auxiliary,
        native_weighting,
        zone_membership,
        coi,
        electoral,
    } = request;
    crate::native_auxiliary::validate_selection(native_auxiliary.as_ref(),partisan.is_some(),coi.is_some())?;
    validate_profile(&o,partisan.is_some()||native_auxiliary.as_ref().is_some_and(|i|i.partisan.is_some()),None)?;
    validate(&g, &o)?;
    let county_pre=crate::county_augmentation::prepare(&g,&o,"edge-weighting")?;
    let weight_graph=county_pre.as_ref().map(|p|&p.graph).unwrap_or(&g);
    let composed=native_weighting.as_ref().map(|spec|crate::native_composed::build(spec,native_auxiliary.as_ref(),&g,&o)).transpose()?;
    let recorded_character=if composed.is_some(){crate::native_auxiliary::CharacterResult{computed:None,evidence:None}}else{crate::native_auxiliary::character(native_auxiliary.as_ref(),weight_graph,&o,native_character_source.is_some()||character.is_some())?};
    let has_recorded_character=native_auxiliary.as_ref().is_some_and(|i|i.character.is_some());
    let native_character=if has_recorded_character{recorded_character.computed}else{crate::native_character_weights::for_engine(weight_graph,&o,native_character_source.as_ref(),character.is_some())?};
    let uses_elections=["proportional-bisect","proportional-section"].contains(&o.structure.as_str());
    let election_data=match (uses_elections,elections.as_ref()) {
        (true,Some(input))=>Some(crate::election_input::validate(&g,&o,input)?),
        (true,None)=>return Err("Proportional methods require complete explicit election counts.".into()),
        (false,Some(_))=>return Err("Election counts have no effect on this engine path.".into()),
        _=>None,
    };
    let uses_demographics=o.search=="vra-recom"||o.structure=="ratio-optimal-vra"||o.weights=="vra-aligned";
    let demographic = if let Some(c)=&composed{c.demographic.clone()}else{match (uses_demographics, demographics.as_ref()) {
        (true, Some(input)) => {
            if o.vra_absent_file_fallback==Some(true){return Err("Absent-file fallback cannot include demographic data.".into());}
            if (o.structure=="ratio-optimal-vra"||o.weights=="vra-aligned")&&input.basis!="total-population"{return Err("Native VRASection and VRA boundary weights require total-population fractions.".into());}
            Some(demographic_fractions(&g, input)?)
        },
        (true, None) if o.vra_absent_file_fallback==Some(true) => None,
        (true, None) => return Err("This engine path requires explicit complete demographic fractions.".into()),
        (false, Some(_)) => return Err("Demographic fractions have no effect on this engine path.".into()),
        _ => None,
    }};
    let native_partisan=if composed.is_some(){crate::native_auxiliary::Partisan{data:None,evidence:None}}else{crate::native_auxiliary::partisan(native_auxiliary.as_ref(),&g,o.weights=="partisan",o.districts)?};
    let skip_native_partisan=o.districts==1&&native_auxiliary.as_ref().is_some_and(|i|i.partisan.is_some());
    let partisan_data=if native_partisan.data.is_some()||skip_native_partisan{native_partisan.data}else{match (o.weights=="partisan",partisan.as_ref()) {
        (true,Some(input))=>Some(partisan_shares(&g,input)?),
        (true,None)=>return Err("Partisan weights require explicit complete tract shares.".into()),
        (false,Some(_))=>return Err("Partisan shares have no effect on this engine path.".into()),
        _=>None,
    }};
    let boosts=partisan_data.as_ref().map(|(shares,_)|bisect_core::build_partisan_weights(&g.edges.iter().map(|&(u,v,_)|(u,v)).collect::<Vec<_>>(),shares,o.dem_threshold.unwrap(),o.rep_threshold.unwrap()));
    let mut weighting_evidence=partisan_data.as_ref().map(|(shares,digest)|{
        let strong=shares.iter().filter(|&&v|v>=o.dem_threshold.unwrap()||v<=o.rep_threshold.unwrap()).count();
        json!({"method":"partisan-adaptive-boost","dispatch_representation":"native-sparse-boost-map","native_candidate_objective":"explicit-boosted-edges-only-when-nonempty","reported_weighted_boundary":"all-cut-edges-with-implicit-unit-default","dem_threshold":o.dem_threshold,"rep_threshold":o.rep_threshold,"strong_tracts":strong,"tracts":shares.len(),"alpha":3.0_f64.max(10.0*(1.0-0.7*(strong as f64/shares.len() as f64))),"baseline":"unit-edge-weight","boost":"same-strong-lean","shares_sha256":digest,"scope":"edge-weighting"})
    });
    let minority_weights=if o.weights=="vra-aligned" {
        let (fractions,digest)=demographic.as_ref().unwrap();
        let high=fractions.iter().filter(|&&f|f>=0.40).count();
        let alpha=3.0_f64.max(10.0*(1.0-0.7*(high as f64/fractions.len() as f64)));
        weighting_evidence=Some(json!({"method":"native-minority-override-v1","dispatch_representation":"native-sparse-boost-map","native_candidate_objective":"explicit-boosted-edges-only-when-nonempty","reported_weighted_boundary":"all-cut-edges-with-implicit-unit-default","threshold":0.40,"high_tracts":high,"tracts":fractions.len(),"alpha":alpha,"baseline":"unit-edge-weight","boost":"both-endpoints-at-or-above-threshold","basis":"total-population","demographics_sha256":digest,"scope":"edge-weighting"}));
        Some(bisect_core::build_vra_edge_weights(&g.edges.iter().map(|&(u,v,_)|(u,v)).collect::<Vec<_>>(),fractions,0.40))
    } else {None};
    if o.weights=="zone-membership" {weighting_evidence=Some(zone_fallback_evidence("edge-weighting"));}
    let character_weights=if let Some(computed)=native_character{weighting_evidence=Some(crate::native_character_weights::evidence(&computed.report,"edge-weighting"));Some(computed.weights.into_iter().map(|(u,v,w)|((u,v),w)).collect::<HashMap<_,_>>())}else{match (["economic-character","housing-character"].contains(&o.weights.as_str()),character.as_ref()) {
        (true,Some(input))=>{
            if input.state!=g.state || input.year!=g.year {return Err("Character input scope must match the prepared graph.".into());}
            let weights=crate::character_input::build_weights(input,&g.geoids,&weight_graph.edges,o.character_alpha.unwrap())?;
            if format!("{}-character",weights.kind)!=o.weights {return Err("Character input kind must match the selected weights.".into());}
            weighting_evidence=Some(json!({"method":"character-cosine-blend","kind":weights.kind,"alpha":weights.alpha,"character_sha256":weights.character_hash,"formula":weights.formula,"zero_policy":weights.zero_policy,"data_year":input.data_year,"baseline":"geographic-boundary-m","tracts":g.geoids.len(),"scope":"edge-weighting"}));
            Some(weights.edges.into_iter().map(|(u,v,w)|((u,v),w)).collect::<HashMap<_,_>>())
        },
        (true,None) if o.character_absent_file_fallback==Some(true)=>{weighting_evidence=Some(crate::character_input::absent_evidence(&o.weights,g.geoids.len(),"edge-weighting"));Some(weight_graph.edges.iter().map(|&(u,v,w)|((u,v),w)).collect::<HashMap<_,_>>())},
        (true,None)=>return Err("Character weights require explicit complete tract observations.".into()),
        (false,Some(_))=>return Err("Character input has no effect without character weights.".into()),
        _=>None,
    }};
    let zone=crate::zone_input::for_engine(weight_graph,&o,zone_membership.as_ref())?;
    if let Some(computed)=&zone {weighting_evidence=Some(computed.evidence("edge-weighting"));}
    let a = &g.adjacency;
    let p = &g.population;
    let k = o.districts;
    let t = o.effective_balance_tolerance();
    let it = o.iterations;
    let seed = o.seed.unwrap_or(if ["ratio-optimal","ratio-optimal-area","ratio-optimal-vra"].contains(&o.structure.as_str()) {1} else {0});
    let q = o.percentile;
    let e: HashMap<_, _> = if let Some(c)=&composed{c.weights.clone()}else if skip_native_partisan||o.districts==1&&has_recorded_character {g.edges.iter().map(|&(u,v,w)|((u,v),w)).collect()} else if o.weights=="unweighted" {HashMap::new()} else if o.weights=="vra-aligned" {minority_weights.as_ref().unwrap().clone()} else if o.weights=="partisan" {boosts.as_ref().unwrap().clone()} else if let Some(computed)=&zone {computed.weights.clone()} else {weight_graph
        .edges
        .iter()
        .map(|&(u, v, w)| {
            let weight = match o.weights.as_str() {
                "unweighted" => 1.0,
                "partisan" => boosts.as_ref().unwrap().get(&(u,v)).copied().unwrap_or(1.0),
                "vra-aligned" => minority_weights.as_ref().unwrap().get(&(u,v)).copied().unwrap_or(1.0),
                "economic-character" | "housing-character" => character_weights.as_ref().unwrap()[&(u,v)],
                "county" if g.county_id(u) == g.county_id(v) => county_weight(w, o.alpha_county),
                _ => w,
            };
            ((u, v), weight)
        })
        .collect()};
    let (e,county_evidence)=if let Some(pre)=&county_pre{(e,Some(pre.evidence.clone()))}else{crate::county_augmentation::apply(&g,&o,e,"edge-weighting")?};
    if let Some(settings)=&electoral {settings.resolve(o.districts)?;}
    let (e,coi_evidence)=if let Some(input)=&coi{let (adjusted,evidence)=crate::coi_input::apply(input,&g.geoids,e,o.weights!="unweighted")?;(adjusted,Some(evidence))}else{(e,None)};
    let (e,native_coi_evidence)=crate::native_auxiliary::coi(native_auxiliary.as_ref(),&g,e)?;
    let mut root_split = None;
    let mut structure_evidence = None;
    if o.search.starts_with("short-burst") && o.burst_length.is_some() && k > 1 {
        structure_evidence=Some(json!({"method":"short-burst", "chain":o.search,
            "burst_length":o.burst_length.unwrap(),"n_bursts":o.n_bursts.unwrap(),
            "total_proposals":o.burst_length.unwrap()*o.n_bursts.unwrap(),
            "selection":"endpoint-unweighted-cut-percentile", "rng":"chacha12-u64-v1"}));
    }
    // Native State dispatch writes its spectral summary even for one district.
    // Keep the native validation/summary call instead of the generic shortcut.
    let partition_timer=crate::partition_timing::Attempt::start(&g.state,&g.year,p.len(),k,o.seed);
    let plan = if o.structure=="spectral" {
        let (assignments,summary)=bisect_apportion::spectral::run_spectral_recursive(a,p,k,t,o.steps)?;
        structure_evidence=Some(summary);
        assignments.into_iter().enumerate().map(|(vertex,district)|(vertex,district+1)).collect()
    } else if k == 1 && !["capacity-clustering","regionalization","flow-construction"].contains(&o.structure.as_str()) {
        (0..p.len()).map(|i| (i, 1)).collect()
    } else {
        match o.structure.as_str() {
            "proportional-bisect" => {
                let counts=election_data.as_ref().unwrap();
                let (assignments,splits)=run_all_splits_proportional_recorded(a,p,&e,&counts.dem,k,t,it,o.seed,None)?;
                structure_evidence=Some(json!({"method":"proportional-bisect","ratio_basis":"democratic-votes-over-census-population","seat_rounding":"nearest-clamped-one-through-k-minus-one","seed_policy":"requested-seed-at-every-node","seed":seed_json(seed),"requested_seed":o.seed.map(seed_json),"split_schedule":"chosen-child-seat-counts","splits":splits}));
                assignments
            }
            "proportional-section" => {
                let counts=election_data.as_ref().unwrap();let seeds=if o.search=="multi"{o.seeds}else{1};let eta=o.proportional_eta.unwrap_or(1.1);
                let (assignments,left,right,cut,d)=run_proportional_section(a,p,&counts.dem,&counts.two_party,&e,k,t,it,seeds,eta,None)?;
                let left_dem:f64=(0..p.len()).filter(|i|assignments[i]<=left).map(|i|counts.dem[i]).sum();
                let root_population_left:i64=(0..p.len()).filter(|i|assignments[i]<=left).map(|i|p[i]).sum();
                let population_fraction=root_population_left as f64/p.iter().sum::<i64>() as f64;
                let dem_fraction=left_dem/counts.total_dem;
                let dem_right_target=(right as f64/(2.0*d*k as f64)).clamp(0.01,0.99);
                let population_ok=population_fraction<=1.001*(left as f64/k as f64)+1e-9 && 1.0-population_fraction<=1.001*(right as f64/k as f64)+1e-9;
                let democratic_ok=dem_fraction<=eta*(1.0-dem_right_target)+1e-9 && 1.0-dem_fraction<=eta*dem_right_target+1e-9;
                structure_evidence=Some(json!({"method":"proportional-section","ratio_basis":"statewide-two-party-share","statewide_democratic_share":d,"share_clamp":[0.01,0.99],"left_seats":left,"right_seats":right,"seat_rounding":"geometric-mean-democratic-quota-complement-positive-children","eta":eta,"root_population_multiplier":1.001,"root_population_fraction_left":population_fraction,"root_population_within_requested_multiplier":population_ok,"root_democratic_within_requested_multiplier":democratic_ok,"root_constraint_check_basis":"census-population-and-supplied-vote-counts","root_cut":cut,"root_democratic_fraction_left":left_dem/counts.total_dem,"root_democratic_target_right":(right as f64/(2.0*d*k as f64)).clamp(0.01,0.99),"seed_policy":"fixed-one-through-budget","seeds_per_root":seeds,"descendant_seeds_per_ratio":seeds.min(50),"descendants":"population-only-geosection"}));
                assignments
            }
            "simulated-annealing" => {
                let assignments = run_all_splits_sa(a,p,&e,k,t,it,o.seed,None,
                    o.sa_steps_per_tract.unwrap_or(10),o.sa_t0_factor.unwrap_or(0.01),o.sa_t_final.unwrap_or(0.0001),seed)?;
                structure_evidence=Some(json!({"method":"simulated-annealing","rng":"chacha12-u64-v1","objective":"unweighted-edge-cut",
                    "steps_per_tract":o.sa_steps_per_tract.unwrap_or(10),"t0_factor":o.sa_t0_factor.unwrap_or(0.01),"t_final":o.sa_t_final.unwrap_or(0.0001),
                    "initial_refinement_iterations":100,"population_targets":"floor-ceil-seat-ratio",
                    "temperature_schedule":"geometric; T0=max(1,factor*initial-cut); final clamped to [1e-12,T0]"}));
                assignments
            }
            "capacity-clustering" => {
                let result = bisect_clustering::capacity_cluster_repaired(a, p, bisect_clustering::ClusterConfig { k, tolerance: t })
                    .map_err(|error| format!("capacity-clustering failed: {error}"))?;
                if result.status != bisect_clustering::ClusterStatus::Valid {
                    return Err(format!("[ALGO] capacity-clustering did not produce a valid plan: {:?}", result.status));
                }
                structure_evidence = Some(serde_json::to_value(&result.summary).map_err(|error| error.to_string())?);
                result.assignment.into_iter().enumerate().map(|(vertex, district)| (vertex, district + 1)).collect()
            }
            "regionalization" => {
                let result = bisect_clustering::regionalize(a, p, bisect_clustering::ClusterConfig { k, tolerance: t })
                    .map_err(|error| format!("regionalization failed: {error}"))?;
                if result.status != bisect_clustering::ClusterStatus::Valid {
                    return Err(format!("[ALGO] regionalization did not produce a valid plan: {:?}", result.status));
                }
                structure_evidence = Some(json!({"method":"regionalization", "summary":result.summary,"merge_log":result.merge_log}));
                result.assignment.into_iter().enumerate().map(|(vertex, district)| (vertex, district + 1)).collect()
            }
            "flow-construction" => {
                let mut flow_config = bisect_flow::FlowConfig::new(k, t);
                if o.flow_repair.as_deref() == Some("none") {
                    flow_config.repair_method = bisect_flow::FlowRepairMethod::None;
                }
                let result = bisect_flow::construct_flow(a, p, flow_config)
                    .map_err(|error| format!("flow construction failed: {error}"))?;
                if result.status != bisect_flow::FlowStatus::Valid {
                    return Err(format!("[ALGO] flow construction did not produce a valid plan: {:?}", result.status));
                }
                structure_evidence = Some(serde_json::to_value(&result.summary).map_err(|error| error.to_string())?);
                result.assignment.into_iter().enumerate().map(|(vertex, district)| (vertex, district + 1)).collect()
            }
            "prime-factor" => {
                use bisect_apportion::{pfr_tree_depth, prime_factor_sequence, MetisPartitioner, PfrCompositor};
                let depth = pfr_tree_depth(k as u32).max(1);
                let per_level_tolerance = (t / (depth + 1) as f64).max(0.001);
                let compositor = PfrCompositor::new(MetisPartitioner {
                    balance_tolerance: per_level_tolerance,
                    niter: it as i32,
                    engine: if o.metis_engine.as_deref()==Some("c-ffi"){bisect_apportion::split::MetisEngine::CFfi}else{bisect_apportion::split::MetisEngine::RedistMetis},
                });
                let result = compositor.compose(a, p, &e, k as u32, o.seed)
                    .map_err(|error| format!("apportion-regions failed: {error}"))?;
                structure_evidence = Some(json!({
                    "method":"apportion-regions", "factor_sequence":prime_factor_sequence(k as u32),
                    "tree_depth":depth, "per_level_tolerance":per_level_tolerance,
                    "research_balance_limit_percent":3.0, "total_edge_cut_integer":result.total_edge_cut,
                    "cache_hits":result.cache_hits, "seed":o.seed.map(seed_json),
                    "split_prescription":"largest-prime-first; prime > 3 uses floor/ceil binary"
                }));
                result.assignment.into_iter().enumerate().map(|(vertex,district)|(vertex,district as usize+1)).collect()
            }
            "ilp" => {
                use bisect_runner::args::{IlpMethod,IlpFallback};
                let method=match o.ilp_method.as_deref().unwrap_or("formulation-only"){"branch-and-cut"=>IlpMethod::BranchAndCut,"iterative-separation"=>IlpMethod::IterativeSeparation,_=>IlpMethod::FormulationOnly};
                let fallback=if o.ilp_fallback.as_deref()==Some("error"){IlpFallback::Error}else{IlpFallback::Metis};
                structure_evidence=Some(json!({"method":"native-ilp-dispatch","ilp_method":o.ilp_method.as_deref().unwrap_or("formulation-only"),"fallback_policy":o.ilp_fallback.as_deref().unwrap_or("metis"),"time_limit":seed_json(o.ilp_time_limit.unwrap_or(300)),"gap":o.ilp_gap.unwrap_or(0.01),"max_tracts":o.ilp_max_tracts.unwrap_or(500),"native_solver_behavior":"shared-native-solver; per-node reports not captured","construction_objective":"unweighted-edges","fallback_refinement_iterations":10,"seed_policy":"backend-default; requested seed ignored","proof_status":"not-certified","report_delivery":"not-captured"}));
                run_all_splits_ilp_with_observer(a,p,&e,k,t,method,fallback,o.ilp_time_limit.unwrap_or(300),o.ilp_gap.unwrap_or(0.01),o.ilp_max_tracts.unwrap_or(500),None,observer)?
            },
            "nway" => {
                if o.metis_objective.is_some() || o.metis_trials.is_some() || o.metis_ufactor.is_some() {
                    structure_evidence=Some(json!({"method":"nway-metis","refinement_objective":o.metis_objective.as_deref().unwrap_or("cut"),"internal_trials":o.metis_trials.unwrap_or(1),"refinement_iterations":it,"trial_selection":"population-excess-then-edge-cut","edge_weight_scaling":"metres-times-100-truncated-minimum-1","contiguity":true,"min_connectivity":true}));
                }
                if let Some(factor)=o.metis_ufactor {
                    structure_evidence.as_mut().unwrap()["imbalance_factor"]=json!({"requested":factor,"effective":factor.clamp(1,1000),"native_clamp_min":1,"native_clamp_max":1000,"population_balance_tolerance_percent":o.balance_tolerance});
                }
                run_nway_partition_tuned(a,p,&e,k,o.metis_ufactor.map(|factor|1.0+factor as f64/1000.0).unwrap_or(1.0+t),it,o.seed,o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1))?
            },
            "bfs-growth" => run_all_splits_bfs(a, p, k, t, None, seed)?,
            "centroidal-voronoi" => {
                structure_evidence=Some(json!({"method":"centroidal-voronoi","distance_metric":o.cvd_metric.as_deref().unwrap_or("geographic"),"iterations_per_split":o.cvd_iters.unwrap_or(50).max(1),"rng":"chacha12-u64-v1","seed_index":"full-u64-modulo-v1"}));
                run_all_splits_cvd(
                a,
                p,
                k,
                t,
                None,
                o.cvd_iters.unwrap_or(50),
                seed,
                if o.cvd_metric.as_deref()==Some("graph-distance") { VoronoiMetric::GraphDistance } else { VoronoiMetric::Geographic },
                &g.centroids,
            )?
            },
            "moving-knife" => {
                structure_evidence=Some(json!({"method":"moving-knife","requested_metric":o.mka_metric.as_deref().unwrap_or("reock"),"effective_metric":"reock","orientations_per_split":o.mka_orientations.unwrap_or(36).max(1),"scoring_boundary":"Native Polsby option currently falls back to Reock; no perimeter scoring is claimed."}));
                run_all_splits_mka(a, p, k, t, None, o.mka_orientations.unwrap_or(36), if o.mka_metric.as_deref()==Some("polsby") {MkaMetric::PolsbyPopper} else {MkaMetric::Reock}, seed, &g.centroids)?
            }
            "compact-polsby" => {
                structure_evidence=Some(json!({"method":"compact-polsby","epsilon":o.compact_epsilon.unwrap_or(0.05),"seeds_per_split":if o.search=="multi" {o.seeds} else {1},"seed_policy":"fixed-1-through-budget","selection":"geometric-mean-polsby-among-near-minimum-weighted-cut"}));
                let native_perimeters;
                let perimeters=if o.compact_geometry.as_deref()==Some("native-state-circular") {
                    native_perimeters=g.areas.iter().enumerate().map(|(i,&area)|bisect_runner::compact_geometry::native_state_exterior_perimeter(i,area,&g.adjacency[i],&e)).collect::<Vec<_>>();
                    &native_perimeters
                } else {&g.exterior_perimeters};
                if let Some(profile)=&o.compact_geometry {structure_evidence.as_mut().unwrap()["geometry_inputs"]=json!(profile);}
                if g.areas.is_empty() || perimeters.is_empty() {
                    structure_evidence.as_mut().unwrap()["selection"]=json!("minimum-weighted-cut-native-missing-geometry");
                    structure_evidence.as_mut().unwrap()["geometry_available"]=json!(false);
                }
                run_all_splits_compact(
                a,
                p,
                &e,
                &g.areas,
                perimeters,
                k,
                t,
                it,
                None,
                &CompactBisectOpts {
                    seeds_per_level: if o.search == "multi" { o.seeds } else { 1 },
                    epsilon: o.compact_epsilon.unwrap_or(0.05),
                },
                None,
            )?
            },
            "ratio-optimal" | "ratio-optimal-area" | "ratio-optimal-vra" => {
                let minority_mass:Option<Vec<f64>>=if o.structure=="ratio-optimal-vra"&&o.vra_absent_file_fallback!=Some(true){Some(demographic.as_ref().unwrap().0.iter().zip(p).map(|(fraction,population)|fraction*(*population as f64)).collect())}else{None};
                let moving_init = o.area_init.as_deref() == Some("moving-knife") && !g.centroids.is_empty();
                let centroids: HashMap<usize, (f64, f64)> = if moving_init {
                    g.centroids.iter().copied().enumerate().collect()
                } else { HashMap::new() };
                let theta = if moving_init {
                    Some(split_subgraph_mka_direction(&(0..g.geoids.len()).collect(), &g.centroids, 180))
                } else { None };
                if o.structure == "ratio-optimal-area" && (o.area_init.is_some() || o.metis_objective.is_some() || o.metis_trials.is_some()) {
                    structure_evidence = Some(json!({"method":"areasection-initialization", "initialization":o.area_init.as_deref().unwrap_or("ratio-optimal"),
                        "orientations":if moving_init {180} else {0}, "direction_radians":theta,
                        "directional_lambda":if moving_init {1.0} else {0.0}, "scope":"root-only"}));
                }
                if o.structure=="ratio-optimal-area" && o.area_init.as_deref()==Some("moving-knife") && !moving_init {
                    structure_evidence.as_mut().unwrap()["initialization_fallback"]=json!("missing-centroids-native-ratio-optimal");
                }
                let (assignments, left_k, right_k, native_cut) = run_geosection_seeded_tuned(
                    a,
                    p,
                    &e,
                    k,
                    t,
                    it,
                    if o.search == "multi" { o.seeds } else { 1 },
                    None,
                    &centroids,
                    if moving_init {1.0} else {0.0},
                    if o.structure == "ratio-optimal-area" {
                        Some(&g.areas)
                    } else {
                        None
                    },
                    o.area_swing,
                    minority_mass.as_deref(),
                    o.w_vra.unwrap_or(0.0),
                    theta,
                    seed,
                    o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1),
                )?;
                if o.structure=="ratio-optimal" && (o.metis_objective.is_some() || o.metis_trials.is_some()) {
                    structure_evidence=Some(json!({"method":"geosection-metis","refinement_objective":o.metis_objective.as_deref().unwrap_or("cut"),"internal_trials_per_candidate":o.metis_trials.unwrap_or(1),"refinement_iterations":it,"scope":"each-recursive-ratio-search-and-two-seat-shortcut","internal_trial_selection":"population-excess-then-edge-cut","candidate_selection":"minimum-original-weighted-cut-per-ratio","ratio_selection":"weighted-cut-divided-by-sqrt-min-child-seats","seeds_per_ratio":if o.search=="multi"{o.seeds}else{1},"seed_schedule":"base-plus-candidate-index-wrapping-u64-at-each-node","post_refinement":"native-population-rebalance","partial_assignment_fallback":"legacy-standard-cut-one-trial"}));
                }
                let mut root_edges=g.edges.iter().collect::<Vec<_>>();root_edges.sort_unstable_by_key(|(u,v,_)|(*u,*v));
                let cut = if o.structure=="ratio-optimal-vra" {native_cut} else {root_edges.into_iter().filter(|(u,v,_)| (assignments[u] <= left_k) != (assignments[v] <= left_k)).map(|(u,v,_)| e.get(&(*u,*v)).copied().unwrap_or(1.0)).sum::<f64>()};
                if let Some(mass)=minority_mass.as_ref(){
                    let total=mass.iter().sum::<f64>();
                    let left=(0..mass.len()).filter(|i|assignments[i]<=left_k).map(|i|mass[i]).sum::<f64>();
                    let alignment=if total>0.0{(left/total-0.5).abs()*2.0}else{0.0};
                    let normalised=cut/(left_k.min(right_k) as f64).sqrt();
                    structure_evidence=Some(json!({"method":"vra-section","scope":"root-only","basis":demographics.as_ref().unwrap().basis,
                        "aggregation":"fraction-times-graph-population","w_vra":o.w_vra,"minority_mass_total":total,"minority_mass_left":left,
                        "minority_share_left":if total>0.0{Some(left/total)}else{None},"alignment":alignment,"normalised_cut":normalised,
                        "selection_score":normalised-o.w_vra.unwrap()*alignment*normalised.max(1.0),
                        "selection_policy":"minimum-weighted-cut-per-ratio-then-alignment-adjusted-ratio","ratio_count":k/2,
                        "root_shortcut":k==2&&o.search=="single","tie_policy":"strict-first-minimum","demographics_sha256":demographic.as_ref().unwrap().1}));
                }
                if o.vra_absent_file_fallback==Some(true){structure_evidence=Some(json!({"method":"vra-section-absent-file-fallback","source_policy":"explicit-native-absent-file","resolved_structure":"ratio-optimal","requested_w_vra":o.w_vra,"alignment_bonus_used":false}));}
                if ["ratio-optimal-area","ratio-optimal-vra"].contains(&o.structure.as_str()) && (o.metis_objective.is_some() || o.metis_trials.is_some()) {
                    structure_evidence.as_mut().unwrap()["metis_refinement"]=json!({"objective":o.metis_objective.as_deref().unwrap_or("cut"),"internal_trials":o.metis_trials.unwrap_or(1),"iterations":it,"scope":"each-recursive-ratio-search-and-two-seat-shortcut","internal_trial_selection":"population-excess-then-edge-cut","candidate_selection":"minimum-original-weighted-cut-per-ratio","seeds_per_ratio":if o.search=="multi"{o.seeds}else{1},"seed_schedule":"base-plus-candidate-index-wrapping-u64-at-each-node","post_refinement":"native-population-rebalance","partial_assignment_fallback":"legacy-standard-cut-one-trial"});
                }
                let total_area = (!g.areas.is_empty()).then(||g.areas.iter().sum::<f64>());
                let left_area = (!g.areas.is_empty()).then(||(0..g.geoids.len())
                    .filter(|v| assignments[v] <= left_k)
                    .map(|v| g.areas[v])
                    .sum::<f64>());
                let left_pop = (0..g.geoids.len())
                    .filter(|v| assignments[v] <= left_k)
                    .map(|v| p[v])
                    .sum::<i64>();
                let area_active = o.structure == "ratio-optimal-area";
                root_split = Some(
                    json!({"left_districts":left_k,"right_districts":right_k,"weighted_boundary":cut,
                    "population_left":left_pop,"population_total":p.iter().sum::<i64>(),"population_target_left":left_k as f64/k as f64,
                    "area_left_m2":left_area,"area_total_m2":total_area,"area_fraction_left":match (left_area,total_area){(Some(left),Some(total)) if total>0.0=>Some(left/total),_=>None},
                    "area_constraint_active":area_active,"area_swing":if area_active{Some(o.area_swing)}else{None},
                    "population_multiplier":if area_active{Some(1.001)}else{None},"constraint_scope":"root-only",
                    "base_seed":seed_json(seed),"seeds_per_ratio":if o.search=="multi"{o.seeds}else{1},
                    "area_within_requested_swing":if area_active{match (left_area,total_area){(Some(left),Some(total))=>Some(left<=total*0.5*o.area_swing+1e-6&&total-left<=total*0.5*o.area_swing+1e-6),_=>None}}else{None}}),
                );
                if native_character_source.is_some()||has_recorded_character{root_split.as_mut().unwrap()["weighted_boundary_class"]=json!(crate::native_character_weights::class(cut));}
                if ["ratio-optimal","ratio-optimal-area","ratio-optimal-vra"].contains(&o.structure.as_str())&&o.weights=="partisan" {
                    root_split.as_mut().unwrap()["native_candidate_cut"]=json!(native_cut);
                    if o.structure=="ratio-optimal-vra" {
                        let reported_cut=g.edges.iter().filter(|(u,v,_)|(assignments[u]<=left_k)!=(assignments[v]<=left_k)).map(|(u,v,_)|e.get(&(*u,*v)).copied().unwrap_or(1.0)).fold(0.0,|sum,w|sum+w);
                        root_split.as_mut().unwrap()["reported_weighted_boundary"]=json!(reported_cut);
                    }
                }
                assignments
            }
            "standard-bisect" => match o.search.as_str() {
                "smc-percentile" => {
                    let (plan,summary)=run_smc_percentile_with_evidence(a,p,k,seed,o.smc_particles.unwrap(),q,o.smc_resample_threshold.unwrap())?;
                    structure_evidence=Some(summary);
                    plan
                }
                "convergence" => {
                    let (plan,summary)=if o.metis_objective.is_some() || o.metis_trials.is_some() {
                        run_all_splits_convergence_tuned(a,p,&e,k,t,it,seed,o.seeds,if o.convergence_unlimited==Some(true) {None}else{Some(o.steps)},o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1))?
                    } else {run_all_splits_convergence(a,p,&e,k,t,it,seed,o.seeds,if o.convergence_unlimited==Some(true) {None}else{Some(o.steps)})?};
                    structure_evidence=Some(serde_json::to_value(summary).map_err(|e|e.to_string())?);
                    plan
                }
                "single" | "multi" if o.metis_objective.is_some() || o.metis_trials.is_some() => {
                    structure_evidence=Some(json!({"method":"recursive-metis","refinement_objective":o.metis_objective.as_deref().unwrap_or("cut"),"internal_trials_per_candidate":o.metis_trials.unwrap_or(1),"refinement_iterations":it,"scope":"each-floor-ceil-tree-node","internal_trial_selection":"population-excess-then-edge-cut","candidate_selection":if o.search=="multi"{"minimum-weighted-cut-among-balanced-contiguous"}else{"single-candidate"},"candidates_per_node":if o.search=="multi"{o.seeds}else{1},"seed_schedule":if o.seed.is_none()&&o.search=="single"{"backend-default-at-each-node"}else{"base-plus-candidate-index-wrapping-u64"},"post_refinement":"native-population-rebalance"}));
                    run_all_splits_tuned(a,p,&e,k,t,it,o.seed,if o.search=="multi"{Some(o.seeds)}else{None},o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1))?
                },
                "single" => run_all_splits(a, p, &e, k, t, it, o.seed, None)?,
                "multi" => run_all_splits_multi(a, p, &e, k, t, it, seed, o.seeds, None)?,
                "percentile" => {
                    if o.metis_objective.is_some() || o.metis_trials.is_some() {
                        let (plan,evidence)=run_all_splits_percentile_tuned(a,p,&e,k,t,it,seed,o.seeds,q,o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1))?;
                        structure_evidence=evidence;
                        plan
                    }else{run_all_splits_percentile(a, p, &e, k, t, it, seed, o.seeds, q, None)?}
                }
                "vra-recom" => {
                    let zeros=vec![0.0;p.len()];
                    let fractions=demographic.as_ref().map(|(f,_)|f.as_slice()).unwrap_or(&zeros);
                    let identity=demographic.as_ref().map(|(_,identity)|identity);
                    let (plan, report) = run_vra_recom_detailed(a,p,&e,k,it,seed,o.steps,q,o.vra_threshold.unwrap(),fractions)?;
                    structure_evidence=Some(json!({"method":"vra-recom","rng":"chacha12-u64-v1","steps":o.steps,"threshold":o.vra_threshold,
                        "basis":demographics.as_ref().map(|d|d.basis.as_str()).unwrap_or("total-population"),"minority_aggregation":"unweighted-tract-mean",
                        "population_tolerance_percent":0.5,"selection":"initial-and-accepted-unweighted-cut-percentile",
                        "rank_rule":"floor(p*record-count), clamped","demographics_sha256":identity,"run":report}));
                    if o.vra_absent_file_fallback==Some(true){structure_evidence.as_mut().unwrap()["source_policy"]=json!("explicit-native-absent-file-zero-fractions");}
                    plan
                },
                "bisection-ensemble" => {
                    structure_evidence=Some(json!({"method":"local-bisection-ensemble","rng":"chacha12-u64-v1","steps_per_split":o.steps,"percentile":q,
                        "selection":"initial-and-accepted-unweighted-cut-percentile","rank_rule":"floor(p*record-count), clamped",
                        "target_policy":"floor-ceil-seat-ratio","small_region_fallback":"at-most-four-tracts-single-metis",
                        "scope":"each-recursive-split","tree_sampler":"wilson"}));
                    if o.metis_objective.is_some() || o.metis_trials.is_some() {
                        structure_evidence.as_mut().unwrap()["metis_initialization"]=json!({"objective":o.metis_objective.as_deref().unwrap_or("cut"),"internal_trials":o.metis_trials.unwrap_or(1),"iterations":it,"scope":"initial-partition-and-small-region-fallback","internal_trial_selection":"population-excess-then-edge-cut","post_refinement":"native-population-rebalance"});
                        run_all_splits_ensemble_optional_seed_tuned(a,p,&e,k,t,it,o.seed,o.steps,q,o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1))?
                    } else {
                    run_all_splits_with_search(
                    a,
                    p,
                    &e,
                    k,
                    t,
                    it,
                    o.seed,
                    None,
                    Some((q, o.steps)),
                )?
                    }
                },
                "flip" => {
                    let (plan,records,rank)=run_flip_chain(a,p,&e,k,t,o.steps,seed,q)?;
                    structure_evidence=Some(json!({"method":"boundary-flip","rng":"chacha12-u64-v1","steps":o.steps,"percentile":q,
                        "selection":"initial-and-accepted-unweighted-cut-percentile","rank_rule":"floor(p*record-count), clamped",
                        "seed_domain":"FLIP_CHAIN_","initial_refinement_iterations":100,"population_allowance":"fraction-of-ideal-district",
                        "nonempty_districts":true,"record_count":records,"selected_rank":rank}));
                    plan
                },
                "forest-recom" => {
                    structure_evidence=Some(json!({"method":"forest-recom","rng":"chacha12-u64-v1","steps":o.steps,"percentile":q,
                        "selection":"initial-and-accepted-unweighted-cut-percentile","rank_rule":"floor(p*record-count), clamped","seed_domains":["FR_FORWARD_","FR_REVERSE_"]}));
                    run_forest_recom(a, p, &e, k, t, it, seed, o.steps, q)?
                },
                "merge-split" => {
                    structure_evidence=Some(json!({"method":"merge-split","rng":"chacha12-u64-v1","steps":o.steps,"percentile":q,
                        "selection":"initial-and-accepted-unweighted-cut-percentile","rank_rule":"floor(p*record-count), clamped","seed_domains":["MS_STEP_","MS_REVERSE_"]}));
                    run_merge_split(a, p, &e, k, t, it, seed, o.steps, q)?
                },
                "short-burst" => {
                    let length=o.burst_length.unwrap_or(20);let bursts=o.n_bursts.unwrap_or(o.steps.div_ceil(20));
                    let (plan,seeds,index)=run_short_burst(a,p,&e,k,t,it,seed,length,bursts,q)?;
                    structure_evidence=Some(json!({"method":"short-burst","chain":o.search,"burst_length":length,"n_bursts":bursts,
                        "total_proposals":length*bursts,"selection":"endpoint-unweighted-cut-percentile","rng":"chacha12-u64-v1",
                        "native_burst_ledger":{"burst_seeds":seeds.iter().map(|s|s.to_string()).collect::<Vec<_>>(),"selected_burst_index":index}}));
                    plan
                }
                "short-burst-forest" => {
                    run_short_burst_forest(a, p, &e, k, t, it, seed, o.burst_length.unwrap_or(20), o.n_bursts.unwrap_or(o.steps.div_ceil(20)), q)?
                }
                "short-burst-merge-split" => run_short_burst_merge_split(
                    a,
                    p,
                    &e,
                    k,
                    t,
                    it,
                    seed,
                    o.burst_length.unwrap_or(20),
                    o.n_bursts.unwrap_or(o.steps.div_ceil(20)),
                    q,
                )?,
                "parallel-tempering" => {
                    {
                        if o.pt_replicas.is_some() {
                            structure_evidence=Some(json!({"method":"parallel-tempering","replicas":o.pt_replicas,"swap_interval":o.pt_swap_interval,
                                "cold_tolerance":o.pt_cold_tol,"hot_tolerance":o.pt_hot_tol,"steps":o.seeds,
                                "selection":"cold-record-unweighted-cut-percentile","rng":"chacha12-u64-v1","swap_support":"both-receiving-tolerances"}));
                        }
                        run_parallel_tempering(a, p, &e, k, it, seed, o.pt_replicas.unwrap_or(4), o.pt_swap_interval.unwrap_or(10), o.pt_cold_tol.unwrap_or(t), o.pt_hot_tol.unwrap_or(t*4.0), o.seeds, q)?
                    }
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    };
    partition_timer.finish()?;
    if plan.len() != p.len()
        || (0..p.len()).any(|i| !plan.contains_key(&i) || !(1..=k).contains(&plan[&i]))
    {
        return Err("Engine returned incomplete assignments.".into());
    }
    let total = p.iter().sum::<i64>();
    let ideal = total as f64 / k as f64;
    let mut districts = Vec::new();
    let mut max_dev = 0.0f64;
    let mut max_native_fraction = 0.0f64;
    let mut connected = true;
    for d in 1..=k {
        let vertices: HashSet<_> = (0..p.len()).filter(|i| plan[i] == d).collect();
        let pop = vertices.iter().map(|&i| p[i]).sum::<i64>();
        let components = connected_components_of(a, &vertices).len();
        if components != 1 {
            connected = false;
        }
        let deviation = (pop as f64 / ideal - 1.0) * 100.0;
        max_native_fraction = max_native_fraction.max((pop as f64-ideal).abs()/ideal);
        max_dev = max_dev.max(deviation.abs());
        districts.push(json!({"district":d,"population":pop,"units":vertices.len(),"deviation_percent":deviation,"components":components}));
    }
    if o.structure == "prime-factor" && max_dev > 3.0 {
        return Err(format!("apportion-regions balance {:.1}% exceeds 3% research limit", max_dev));
    }
    // An empty cut is positive zero in both targets. Iterator Sum uses a
    // negative-zero identity whose optimized representation can differ by target.
    let boundary = g
        .edges
        .iter()
        .filter(|&&(u, v, _)| plan[&u] != plan[&v])
        .map(|&(_, _, w)| w)
        .fold(0.0_f64, |total, weight| total + weight);
    // Use the prepared edge order in both targets. HashMap iteration changes
    // floating-point summation order even when the assignments are identical.
    let weighted = g.edges
        .iter()
        .filter(|&&(u, v, _)| plan[&u] != plan[&v])
        .map(|&(u, v, _)| e.get(&(u,v)).copied().unwrap_or(1.0))
        .fold(0.0_f64, |total, weight| total + weight);
    let mut counties: HashMap<&str, HashSet<usize>> = HashMap::new();
    let assignments: BTreeMap<_, _> = g
        .geoids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            if let Some(county)=g.report_county_id(i){counties.entry(county).or_default().insert(plan[&i]);}
            (id.clone(), plan[&i])
        })
        .collect();
    let mut result=json!({"schema_version":1,"state":g.state,"year":g.year,"backend":if o.metis_engine.as_deref()==Some("c-ffi"){"c-metis-msvcrt-v1"}else{"metis-core-rust"},"options":o,
        "assignments":assignments,"metrics":{"districts":k,"district_metrics":districts,"units":p.len(),"population":total,
        "max_deviation_percent":max_dev,"graph_boundary_m":boundary,"weighted_boundary":weighted,
        "split_counties":counties.values().filter(|ds|ds.len()>1).count(),"contiguous":connected,
        "within_requested_tolerance":if o.balance_tolerance_fraction.is_some(){max_native_fraction<=o.effective_balance_tolerance()}else{max_dev<=o.balance_tolerance},"root_split":root_split,"structure_evidence":structure_evidence,"execution":"heuristic","optimality":"unproved"}});
    if g.areas.is_empty() || g.exterior_perimeters.is_empty() || g.centroids.is_empty() {
        result["metrics"]["input_availability"]=json!({"areas_m2":!g.areas.is_empty(),"exterior_perimeters_m":!g.exterior_perimeters.is_empty(),"centroids_wgs84":!g.centroids.is_empty()});
    }
    if let Some(settings)=electoral { result["metrics"]["electoral_evidence"]=settings.evidence(o.districts,g.population.iter().sum())?; }
    if let Some(evidence)=county_evidence {result["metrics"]["county_augmentation_evidence"]=evidence;}
    // Native State emits this descriptive file only for multi-district NWay
    // with minority weighting. Use the same total-population mass and zero
    // subgroup inputs, independently of the search objective or county layer.
    if o.structure=="nway" && (o.weights=="vra-aligned"||native_weighting.as_ref().is_some_and(|w|w.minority_weighting)) && k>1 {
        let (fractions,digest)=demographic.as_ref().unwrap();
        let minority:Vec<f64>=fractions.iter().zip(p).map(|(fraction,&population)|fraction*population as f64).collect();
        let zeros=vec![0.0;p.len()];
        let analysis=bisect_analysis::try_analyze_mm_districts(&plan,p,&minority,&zeros,&zeros,0.50).map_err(|e|format!("Native State VRA summary failed: {e}"))?;
        let analysis_json=serde_json::to_string_pretty(&analysis).map_err(|e|e.to_string())?;
        result["metrics"]["native_state_vra"]=json!({"schema_version":"bisect-native-state-vra-v1","basis":"total-population","minority_mass":"fraction-times-prepared-unit-population","threshold":0.5,"comparison":"strictly-greater","subgroup_inputs":"native-zero-placeholders","demographics_sha256":digest,"analysis":analysis,"analysis_json":analysis_json});
    }
    if let Some(evidence)=coi_evidence {result["metrics"]["coi_weighting_evidence"]=evidence;}
    if let Some(evidence)=native_partisan.evidence{result["metrics"]["native_partisan_source_evidence"]=evidence;}
    if let Some(evidence)=native_coi_evidence{result["metrics"]["native_coi_source_evidence"]=evidence;}
    if let Some(evidence)=recorded_character.evidence{result["metrics"]["native_character_source_evidence"]=evidence;}
    if let Some(c)=composed{for(key,value)in c.metrics{result["metrics"][key]=value;}}
    if let Some(counts)=election_data {
        result["metrics"]["election_input_evidence"]=json!({"schema_version":"bisect-election-evidence-v1","counts_sha256":counts.sha256,"election_year":elections.as_ref().unwrap().election_year,"democratic_total":counts.total_dem,"two_party_total":counts.total_two_party});
    }
    if native_character_source.is_some()||has_recorded_character{result["metrics"]["weighted_boundary_class"]=json!(crate::native_character_weights::class(weighted));}
    if let Some(evidence)=weighting_evidence {result["metrics"]["weighting_evidence"]=evidence;}
    if o.weights=="county" {result["metrics"]["weighting_evidence"]=json!({"method":"native-county-additive-v1","alpha":o.alpha_county,"within_county_multiplier":if o.alpha_county>1e-10 {1.0+o.alpha_county}else{1.0},"baseline":"physical-geographic-boundary","county_identity":if g.county_ids.is_some(){"explicit-prepared-county-ids"}else{"first-five-geoid-digits"},"disabled_at_or_below":1e-10});}
    Ok(result)
}
