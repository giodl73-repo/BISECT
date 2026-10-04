use bisect_runner::bisection_runner::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedGraph {
    pub schema_version: u32,
    pub state: String,
    pub year: String,
    pub geoids: Vec<String>,
    pub adjacency: Vec<Vec<usize>>,
    pub population: Vec<i64>,
    /// Canonical undirected edges, physical length in metres. Bridges remain explicit.
    pub edges: Vec<(usize, usize, f64)>,
    pub areas: Vec<f64>,
    pub exterior_perimeters: Vec<f64>,
    pub centroids: Vec<(f64, f64)>,
}

// Preserve legacy numeric seeds where JS can represent them exactly.
fn seed_json(seed:u64)->Value {
    if seed<=9_007_199_254_740_991 {json!(seed)} else {json!(seed.to_string())}
}
mod seed_serde {
    use serde::{de::{self,Visitor},Deserializer,Serialize,Serializer};
    use std::fmt;
    pub fn serialize<S:Serializer>(seed:&u64,serializer:S)->Result<S::Ok,S::Error>{
        super::seed_json(*seed).serialize(serializer)
    }
    pub fn deserialize<'de,D:Deserializer<'de>>(deserializer:D)->Result<u64,D::Error>{
        struct Seed;
        impl<'de> Visitor<'de> for Seed {
            type Value=u64;
            fn expecting(&self,f:&mut fmt::Formatter)->fmt::Result{f.write_str("a u64 integer or canonical decimal u64 string")}
            fn visit_u64<E:de::Error>(self,value:u64)->Result<u64,E>{Ok(value)}
            fn visit_str<E:de::Error>(self,value:&str)->Result<u64,E>{
                if value.is_empty() || value.len()>20 || !value.bytes().all(|b|b.is_ascii_digit()) || (value.len()>1 && value.starts_with('0')) {
                    return Err(E::custom("Seed must be a canonical decimal u64."));
                }
                value.parse().map_err(|_|E::custom("Seed exceeds u64 range."))
            }
        }
        deserializer.deserialize_any(Seed)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_alpha: Option<f64>,
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
    pub vra_threshold: Option<f64>,
    pub structure: String,
    pub weights: String,
    pub search: String,
    pub districts: usize,
    #[serde(with="seed_serde")]
    pub seed: u64,
    pub seeds: usize,
    pub steps: usize,
    pub percentile: f64,
    pub alpha_county: f64,
    pub balance_tolerance: f64,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
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

pub(crate) fn validate(g: &PreparedGraph, o: &Options) -> Result<(), String> {
    let character=["economic-character","housing-character"].contains(&o.weights.as_str());
    if character {
        if !o.character_alpha.is_some_and(|v|v.is_finite()&&!v.is_sign_negative()&&v<=1.) {return Err("Character weights require a finite blend alpha in [0,1].".into());}
    } else if o.character_alpha.is_some() {return Err("Character blend alpha has no effect without character weights.".into());}
    if o.structure=="proportional-section" {
        if o.proportional_eta.is_some_and(|v|!v.is_finite()||!(1.0..=2.0).contains(&v)){return Err("ProportionalSection eta must be finite in [1,2].".into());}
    } else if o.proportional_eta.is_some(){return Err("Vote constraint eta applies only to ProportionalSection.".into());}
    if o.metis_objective.is_some() || o.metis_trials.is_some() {
        if !(o.structure=="nway" || (["ratio-optimal","ratio-optimal-area","ratio-optimal-vra"].contains(&o.structure.as_str())&&["single","multi"].contains(&o.search.as_str())) || (o.structure=="standard-bisect"&&["single","multi","percentile","convergence","bisection-ensemble"].contains(&o.search.as_str()))) || o.metis_objective.as_ref().is_some_and(|v| !["cut","volume"].contains(&v.as_str())) || o.metis_trials.is_some_and(|v| !(1..=100).contains(&v)) {return Err("METIS objective/trials require nway, Geo/Area/VRASection single/multi, or standard single/multi/percentile/convergence/ensemble bisection, cut or volume, and 1..=100 trials.".into());}
    }
    if o.weights=="partisan" {
        if o.structure!="standard-bisect" || !["single","multi","percentile","convergence","bisection-ensemble"].contains(&o.search.as_str()) || o.dem_threshold.is_none() || o.rep_threshold.is_none()
            || !o.dem_threshold.is_some_and(|v|v.is_finite()&&(0.0..=1.0).contains(&v)) || !o.rep_threshold.is_some_and(|v|v.is_finite()&&(0.0..=1.0).contains(&v)) || o.dem_threshold<o.rep_threshold {
            return Err("Partisan weights require standard Single/Multi/percentile/convergence/local-ensemble and finite ordered thresholds in [0,1].".into());
        }
    } else if o.dem_threshold.is_some() || o.rep_threshold.is_some() {return Err("Partisan thresholds have no effect without partisan weights.".into());}
    let n = g.geoids.len();
    if g.schema_version != 1
        || n == 0
        || n > 1_000_000
        || g.population.len() != n
        || g.adjacency.len() != n
    {
        return Err("Invalid prepared graph schema or vertex dimensions.".into());
    }
    let mut geoids = HashSet::new();
    if g.geoids
        .iter()
        .any(|id| id.len() != 11 || !id.bytes().all(|b| b.is_ascii_digit()) || !geoids.insert(id))
    {
        return Err("GEOIDs must be unique eleven-digit strings.".into());
    }
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
        if u >= v || !arcs.contains(&(u, v)) || !edges.insert((u, v)) || !w.is_finite() || w <= 0.0
        {
            return Err("Invalid physical boundary edge.".into());
        }
    }
    if edges.len() * 2 != arcs.len() {
        return Err("Every adjacency edge needs a physical weight.".into());
    }
    for values in [&g.areas, &g.exterior_perimeters] {
        if values.len() != n
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
            if o.weights == "county" && g.geoids[u][..5] == g.geoids[v][..5] {
                w * o.alpha_county.max(1.0)
            } else {
                w
            }
        })
        .sum::<f64>()
        .is_finite()
    {
        return Err("Physical/weighted boundary totals exceed finite numeric range.".into());
    }
    if g.centroids.len() != n
        || g.centroids
            .iter()
            .any(|&(x, y)| !x.is_finite() || !y.is_finite() || x.abs() > 180.0 || y.abs() > 90.0)
    {
        return Err("Invalid longitude/latitude centroids.".into());
    }
    if !(1..=n.min(500)).contains(&o.districts)
        || !((if o.search == "parallel-tempering" && o.pt_replicas.is_some() {0} else {1})..=10000).contains(&o.seeds)
        || !((if ["forest-recom", "merge-split", "flip", "bisection-ensemble", "vra-recom"].contains(&o.search.as_str()) {0} else {1})..=100000).contains(&o.steps)
        || !(1..=1000).contains(&o.iterations)
        || !o.balance_tolerance.is_finite()
        || !(0.01..=25.0).contains(&o.balance_tolerance)
        || !o.percentile.is_finite()
        || !(0.0..=1.0).contains(&o.percentile)
        || !o.alpha_county.is_finite()
        || !(0.0..=100.0).contains(&o.alpha_county)
        || !o.area_swing.is_finite()
        || !(1.01..=2.0).contains(&o.area_swing)
    {
        return Err("Invalid engine option bounds.".into());
    }
    if !["geographic", "unweighted", "county", "partisan", "economic-character", "housing-character"].contains(&o.weights.as_str())
        || (o.weights != "county" && o.alpha_county != 0.0)
    {
        return Err("Invalid boundary weights.".into());
    }
    if o.structure == "simulated-annealing" {
        if !o.sa_steps_per_tract.is_some_and(|steps| steps <= 10000)
            || !o.sa_t0_factor.is_some_and(|v| v.is_finite() && (0.0..=1000.0).contains(&v))
            || !o.sa_t_final.is_some_and(|v| v.is_finite() && (1e-15..=1e6).contains(&v)) {
            return Err("Missing or invalid simulated annealing parameters.".into());
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
            || !o.pt_cold_tol.is_some_and(|v| v.is_finite() && (0.0001..=0.25).contains(&v) && (v*100.0-o.balance_tolerance).abs()<1e-9)
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
        if o.compact_epsilon.is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v)) {
            return Err("Invalid CompactBisect edge-cut slack.".into());
        }
    } else if o.compact_epsilon.is_some() {
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
            || o.balance_tolerance != 0.5 {
            return Err("SMC requires particle/resampling parameters and the native fixed 0.5 percent tolerance setting.".into());
        }
    } else if o.smc_particles.is_some() || o.smc_resample_threshold.is_some() {
        return Err("SMC parameters apply only to SMC percentile search.".into());
    }
    if o.search == "vra-recom" {
        if !o.vra_threshold.is_some_and(|v| v.is_finite() && (0.0..=1.0).contains(&v)) || o.balance_tolerance != 0.5 {
            return Err("VRA ReCom requires a fraction threshold and native fixed 0.5 percent population tolerance.".into());
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
        "proportional-bisect" | "simulated-annealing" | "capacity-clustering" | "regionalization" | "flow-construction" | "spectral" | "prime-factor" | "nway" | "bfs-growth" | "centroidal-voronoi" | "moving-knife" => ["single"].as_slice(),
        _ => return Err("Structure has not yet been wired into the browser boundary.".into()),
    };
    if !supported.contains(&o.search.as_str()) {
        return Err("Structure does not execute this search.".into());
    }
    Ok(())
}

pub fn execute(request: Request) -> Result<Value, String> {
    let Request {
        graph: g,
        options: o,
        demographics,
        partisan,
        elections,
        character,
    } = request;
    validate(&g, &o)?;
    let uses_elections=["proportional-bisect","proportional-section"].contains(&o.structure.as_str());
    let election_data=match (uses_elections,elections.as_ref()) {
        (true,Some(input))=>Some(crate::election_input::validate(&g,&o,input)?),
        (true,None)=>return Err("Proportional methods require complete explicit election counts.".into()),
        (false,Some(_))=>return Err("Election counts have no effect on this engine path.".into()),
        _=>None,
    };
    let uses_demographics=o.search=="vra-recom"||o.structure=="ratio-optimal-vra";
    let demographic = match (uses_demographics, demographics.as_ref()) {
        (true, Some(input)) => {
            if o.structure=="ratio-optimal-vra"&&input.basis!="total-population"{return Err("VRASection's native proxy requires total-population fractions.".into());}
            Some(demographic_fractions(&g, input)?)
        },
        (true, None) => return Err("This engine path requires explicit complete demographic fractions.".into()),
        (false, Some(_)) => return Err("Demographic fractions have no effect on this engine path.".into()),
        _ => None,
    };
    let partisan_data=match (o.weights=="partisan",partisan.as_ref()) {
        (true,Some(input))=>Some(partisan_shares(&g,input)?),
        (true,None)=>return Err("Partisan weights require explicit complete tract shares.".into()),
        (false,Some(_))=>return Err("Partisan shares have no effect on this engine path.".into()),
        _=>None,
    };
    let boosts=partisan_data.as_ref().map(|(shares,_)|bisect_core::build_partisan_weights(&g.edges.iter().map(|&(u,v,_)|(u,v)).collect::<Vec<_>>(),shares,o.dem_threshold.unwrap(),o.rep_threshold.unwrap()));
    let mut weighting_evidence=partisan_data.as_ref().map(|(shares,digest)|{
        let strong=shares.iter().filter(|&&v|v>=o.dem_threshold.unwrap()||v<=o.rep_threshold.unwrap()).count();
        json!({"method":"partisan-adaptive-boost","dem_threshold":o.dem_threshold,"rep_threshold":o.rep_threshold,"strong_tracts":strong,"tracts":shares.len(),"alpha":3.0_f64.max(10.0*(1.0-0.7*(strong as f64/shares.len() as f64))),"baseline":"unit-edge-weight","boost":"same-strong-lean","shares_sha256":digest,"scope":"edge-weighting"})
    });
    let character_weights=match (["economic-character","housing-character"].contains(&o.weights.as_str()),character.as_ref()) {
        (true,Some(input))=>{
            if input.state!=g.state || input.year!=g.year {return Err("Character input scope must match the prepared graph.".into());}
            let weights=crate::character_input::build_weights(input,&g.geoids,&g.edges,o.character_alpha.unwrap())?;
            if format!("{}-character",weights.kind)!=o.weights {return Err("Character input kind must match the selected weights.".into());}
            weighting_evidence=Some(json!({"method":"character-cosine-blend","kind":weights.kind,"alpha":weights.alpha,"character_sha256":weights.character_hash,"formula":weights.formula,"zero_policy":weights.zero_policy,"data_year":input.data_year,"baseline":"geographic-boundary-m","tracts":g.geoids.len(),"scope":"edge-weighting"}));
            Some(weights.edges.into_iter().map(|(u,v,w)|((u,v),w)).collect::<HashMap<_,_>>())
        },
        (true,None)=>return Err("Character weights require explicit complete tract observations.".into()),
        (false,Some(_))=>return Err("Character input has no effect without character weights.".into()),
        _=>None,
    };
    let a = &g.adjacency;
    let p = &g.population;
    let k = o.districts;
    let t = o.balance_tolerance / 100.0;
    let it = o.iterations;
    let seed = o.seed;
    let q = o.percentile;
    let e: HashMap<_, _> = g
        .edges
        .iter()
        .map(|&(u, v, w)| {
            let weight = match o.weights.as_str() {
                "unweighted" => 1.0,
                "partisan" => boosts.as_ref().unwrap().get(&(u,v)).copied().unwrap_or(1.0),
                "economic-character" | "housing-character" => character_weights.as_ref().unwrap()[&(u,v)],
                "county" if g.geoids[u][..5] == g.geoids[v][..5] => w * o.alpha_county.max(1.0),
                _ => w,
            };
            ((u, v), weight)
        })
        .collect();
    let mut root_split = None;
    let mut structure_evidence = None;
    if o.search.starts_with("short-burst") && o.burst_length.is_some() && k > 1 {
        structure_evidence=Some(json!({"method":"short-burst", "chain":o.search,
            "burst_length":o.burst_length.unwrap(),"n_bursts":o.n_bursts.unwrap(),
            "total_proposals":o.burst_length.unwrap()*o.n_bursts.unwrap(),
            "selection":"endpoint-unweighted-cut-percentile", "rng":"chacha12-u64-v1"}));
    }
    let plan = if k == 1 {
        (0..p.len()).map(|i| (i, 1)).collect()
    } else {
        match o.structure.as_str() {
            "proportional-bisect" => {
                let counts=election_data.as_ref().unwrap();
                let (assignments,splits)=run_all_splits_proportional_recorded(a,p,&e,&counts.dem,k,t,it,Some(seed),None)?;
                structure_evidence=Some(json!({"method":"proportional-bisect","ratio_basis":"democratic-votes-over-census-population","seat_rounding":"nearest-clamped-one-through-k-minus-one","seed_policy":"requested-seed-at-every-node","seed":seed_json(seed),"split_schedule":"chosen-child-seat-counts","splits":splits}));
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
                let assignments = run_all_splits_sa(a,p,&e,k,t,it,Some(seed),None,
                    o.sa_steps_per_tract.unwrap(),o.sa_t0_factor.unwrap(),o.sa_t_final.unwrap(),seed)?;
                structure_evidence=Some(json!({"method":"simulated-annealing","rng":"chacha12-u64-v1","objective":"unweighted-edge-cut",
                    "steps_per_tract":o.sa_steps_per_tract,"t0_factor":o.sa_t0_factor,"t_final":o.sa_t_final,
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
            "spectral" => {
                let (assignments,summary)=bisect_apportion::spectral::run_spectral_recursive(a,p,k,t,o.steps)?;
                structure_evidence=Some(summary);
                assignments.into_iter().enumerate().map(|(vertex,district)|(vertex,district+1)).collect()
            }
            "prime-factor" => {
                use bisect_apportion::{pfr_tree_depth, prime_factor_sequence, MetisPartitioner, PfrCompositor};
                let depth = pfr_tree_depth(k as u32).max(1);
                let per_level_tolerance = (t / (depth + 1) as f64).max(0.001);
                let compositor = PfrCompositor::new(MetisPartitioner {
                    balance_tolerance: per_level_tolerance,
                    niter: it as i32,
                    engine: bisect_apportion::split::MetisEngine::RedistMetis,
                });
                let result = compositor.compose(a, p, &e, k as u32, Some(seed))
                    .map_err(|error| format!("apportion-regions failed: {error}"))?;
                structure_evidence = Some(json!({
                    "method":"apportion-regions", "factor_sequence":prime_factor_sequence(k as u32),
                    "tree_depth":depth, "per_level_tolerance":per_level_tolerance,
                    "research_balance_limit_percent":3.0, "total_edge_cut_integer":result.total_edge_cut,
                    "cache_hits":result.cache_hits, "seed":seed_json(seed),
                    "split_prescription":"largest-prime-first; prime > 3 uses floor/ceil binary"
                }));
                result.assignment.into_iter().enumerate().map(|(vertex,district)|(vertex,district as usize+1)).collect()
            }
            "nway" => {
                if o.metis_objective.is_some() || o.metis_trials.is_some() {
                    structure_evidence=Some(json!({"method":"nway-metis","refinement_objective":o.metis_objective.as_deref().unwrap_or("cut"),"internal_trials":o.metis_trials.unwrap_or(1),"refinement_iterations":it,"trial_selection":"population-excess-then-edge-cut","edge_weight_scaling":"metres-times-100-truncated-minimum-1","contiguity":true,"min_connectivity":true}));
                }
                run_nway_partition_tuned(a,p,&e,k,1.0+t,it,Some(seed),o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1))?
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
                run_all_splits_compact(
                a,
                p,
                &e,
                &g.areas,
                &g.exterior_perimeters,
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
                let minority_mass:Option<Vec<f64>>=if o.structure=="ratio-optimal-vra"{Some(demographic.as_ref().unwrap().0.iter().zip(p).map(|(fraction,population)|fraction*(*population as f64)).collect())}else{None};
                let moving_init = o.area_init.as_deref() == Some("moving-knife");
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
                let (assignments, left_k, right_k, _selection_cut) = run_geosection_seeded_tuned(
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
                let cut = root_edges.into_iter().filter(|(u,v,_)| (assignments[u] <= left_k) != (assignments[v] <= left_k)).map(|(u,v,_)| e[&(*u,*v)]).sum::<f64>();
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
                if ["ratio-optimal-area","ratio-optimal-vra"].contains(&o.structure.as_str()) && (o.metis_objective.is_some() || o.metis_trials.is_some()) {
                    structure_evidence.as_mut().unwrap()["metis_refinement"]=json!({"objective":o.metis_objective.as_deref().unwrap_or("cut"),"internal_trials":o.metis_trials.unwrap_or(1),"iterations":it,"scope":"each-recursive-ratio-search-and-two-seat-shortcut","internal_trial_selection":"population-excess-then-edge-cut","candidate_selection":"minimum-original-weighted-cut-per-ratio","seeds_per_ratio":if o.search=="multi"{o.seeds}else{1},"seed_schedule":"base-plus-candidate-index-wrapping-u64-at-each-node","post_refinement":"native-population-rebalance","partial_assignment_fallback":"legacy-standard-cut-one-trial"});
                }
                let total_area = g.areas.iter().sum::<f64>();
                let left_area = (0..g.geoids.len())
                    .filter(|v| assignments[v] <= left_k)
                    .map(|v| g.areas[v])
                    .sum::<f64>();
                let left_pop = (0..g.geoids.len())
                    .filter(|v| assignments[v] <= left_k)
                    .map(|v| p[v])
                    .sum::<i64>();
                let area_active = o.structure == "ratio-optimal-area";
                root_split = Some(
                    json!({"left_districts":left_k,"right_districts":right_k,"weighted_boundary":cut,
                    "population_left":left_pop,"population_total":p.iter().sum::<i64>(),"population_target_left":left_k as f64/k as f64,
                    "area_left_m2":left_area,"area_total_m2":total_area,"area_fraction_left":if total_area>0.0{Some(left_area/total_area)}else{None},
                    "area_constraint_active":area_active,"area_swing":if area_active{Some(o.area_swing)}else{None},
                    "population_multiplier":if area_active{Some(1.001)}else{None},"constraint_scope":"root-only",
                    "base_seed":seed_json(seed),"seeds_per_ratio":if o.search=="multi"{o.seeds}else{1},
                    "area_within_requested_swing":if area_active{Some(left_area<=total_area*0.5*o.area_swing+1e-6&&total_area-left_area<=total_area*0.5*o.area_swing+1e-6)}else{None}}),
                );
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
                        run_all_splits_convergence_tuned(a,p,&e,k,t,it,seed,o.seeds,Some(o.steps),o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1))?
                    } else {run_all_splits_convergence(a,p,&e,k,t,it,seed,o.seeds,Some(o.steps))?};
                    structure_evidence=Some(serde_json::to_value(summary).map_err(|e|e.to_string())?);
                    plan
                }
                "single" | "multi" if o.metis_objective.is_some() || o.metis_trials.is_some() => {
                    structure_evidence=Some(json!({"method":"recursive-metis","refinement_objective":o.metis_objective.as_deref().unwrap_or("cut"),"internal_trials_per_candidate":o.metis_trials.unwrap_or(1),"refinement_iterations":it,"scope":"each-floor-ceil-tree-node","internal_trial_selection":"population-excess-then-edge-cut","candidate_selection":if o.search=="multi"{"minimum-weighted-cut-among-balanced-contiguous"}else{"single-candidate"},"candidates_per_node":if o.search=="multi"{o.seeds}else{1},"seed_schedule":"base-plus-candidate-index-wrapping-u64","post_refinement":"native-population-rebalance"}));
                    run_all_splits_tuned(a,p,&e,k,t,it,Some(seed),if o.search=="multi"{Some(o.seeds)}else{None},o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1))?
                },
                "single" => run_all_splits(a, p, &e, k, t, it, Some(seed), None)?,
                "multi" => run_all_splits_multi(a, p, &e, k, t, it, seed, o.seeds, None)?,
                "percentile" => {
                    if o.metis_objective.is_some() || o.metis_trials.is_some() {
                        let (plan,evidence)=run_all_splits_percentile_tuned(a,p,&e,k,t,it,seed,o.seeds,q,o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1))?;
                        structure_evidence=evidence;
                        plan
                    }else{run_all_splits_percentile(a, p, &e, k, t, it, seed, o.seeds, q, None)?}
                }
                "vra-recom" => {
                    let (fractions, identity) = demographic.as_ref().unwrap();
                    let (plan, report) = run_vra_recom_detailed(a,p,&e,k,it,seed,o.steps,q,o.vra_threshold.unwrap(),fractions)?;
                    structure_evidence=Some(json!({"method":"vra-recom","rng":"chacha12-u64-v1","steps":o.steps,"threshold":o.vra_threshold,
                        "basis":demographics.as_ref().unwrap().basis,"minority_aggregation":"unweighted-tract-mean",
                        "population_tolerance_percent":0.5,"selection":"initial-and-accepted-unweighted-cut-percentile",
                        "rank_rule":"floor(p*record-count), clamped","demographics_sha256":identity,"run":report}));
                    plan
                },
                "bisection-ensemble" => {
                    structure_evidence=Some(json!({"method":"local-bisection-ensemble","rng":"chacha12-u64-v1","steps_per_split":o.steps,"percentile":q,
                        "selection":"initial-and-accepted-unweighted-cut-percentile","rank_rule":"floor(p*record-count), clamped",
                        "target_policy":"floor-ceil-seat-ratio","small_region_fallback":"at-most-four-tracts-single-metis",
                        "scope":"each-recursive-split","tree_sampler":"wilson"}));
                    if o.metis_objective.is_some() || o.metis_trials.is_some() {
                        structure_evidence.as_mut().unwrap()["metis_initialization"]=json!({"objective":o.metis_objective.as_deref().unwrap_or("cut"),"internal_trials":o.metis_trials.unwrap_or(1),"iterations":it,"scope":"initial-partition-and-small-region-fallback","internal_trial_selection":"population-excess-then-edge-cut","post_refinement":"native-population-rebalance"});
                        run_all_splits_ensemble_tuned(a,p,&e,k,t,it,seed,o.steps,q,o.metis_objective.as_deref().unwrap_or("cut"),o.metis_trials.unwrap_or(1))?
                    } else {
                    run_all_splits_with_search(
                    a,
                    p,
                    &e,
                    k,
                    t,
                    it,
                    Some(seed),
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
                    run_short_burst(a, p, &e, k, t, it, seed, o.burst_length.unwrap_or(20), o.n_bursts.unwrap_or(o.steps.div_ceil(20)), q)?.0
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
    if plan.len() != p.len()
        || (0..p.len()).any(|i| !plan.contains_key(&i) || !(1..=k).contains(&plan[&i]))
    {
        return Err("Engine returned incomplete assignments.".into());
    }
    let total = p.iter().sum::<i64>();
    let ideal = total as f64 / k as f64;
    let mut districts = Vec::new();
    let mut max_dev = 0.0f64;
    let mut connected = true;
    for d in 1..=k {
        let vertices: HashSet<_> = (0..p.len()).filter(|i| plan[i] == d).collect();
        let pop = vertices.iter().map(|&i| p[i]).sum::<i64>();
        let components = connected_components_of(a, &vertices).len();
        if components != 1 {
            connected = false;
        }
        let deviation = (pop as f64 / ideal - 1.0) * 100.0;
        max_dev = max_dev.max(deviation.abs());
        districts.push(json!({"district":d,"population":pop,"units":vertices.len(),"deviation_percent":deviation,"components":components}));
    }
    if o.structure == "prime-factor" && max_dev > 3.0 {
        return Err(format!("apportion-regions balance {:.1}% exceeds 3% research limit", max_dev));
    }
    let boundary = g
        .edges
        .iter()
        .filter(|&&(u, v, _)| plan[&u] != plan[&v])
        .map(|&(_, _, w)| w)
        .sum::<f64>();
    // Use the prepared edge order in both targets. HashMap iteration changes
    // floating-point summation order even when the assignments are identical.
    let weighted = g.edges
        .iter()
        .filter(|&&(u, v, _)| plan[&u] != plan[&v])
        .map(|&(u, v, _)| e[&(u, v)])
        .sum::<f64>();
    let mut counties: HashMap<&str, HashSet<usize>> = HashMap::new();
    let assignments: BTreeMap<_, _> = g
        .geoids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            counties.entry(&id[..5]).or_default().insert(plan[&i]);
            (id.clone(), plan[&i])
        })
        .collect();
    let mut result=json!({"schema_version":1,"state":g.state,"year":g.year,"backend":"metis-core-rust","options":o,
        "assignments":assignments,"metrics":{"districts":k,"district_metrics":districts,"units":p.len(),"population":total,
        "max_deviation_percent":max_dev,"graph_boundary_m":boundary,"weighted_boundary":weighted,
        "split_counties":counties.values().filter(|ds|ds.len()>1).count(),"contiguous":connected,
        "within_requested_tolerance":max_dev<=o.balance_tolerance,"root_split":root_split,"structure_evidence":structure_evidence,"execution":"heuristic","optimality":"unproved"}});
    if let Some(counts)=election_data {
        result["metrics"]["election_input_evidence"]=json!({"schema_version":"bisect-election-evidence-v1","counts_sha256":counts.sha256,"election_year":elections.as_ref().unwrap().election_year,"democratic_total":counts.total_dem,"two_party_total":counts.total_two_party});
    }
    if let Some(evidence)=weighting_evidence {result["metrics"]["weighting_evidence"]=evidence;}
    Ok(result)
}
