//! Versioned resolved State settings. Paths here are records, never IO authority.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::state_config::AlgorithmConfig;

impl<'de,C:Deserialize<'de>> Deserialize<'de> for AlgorithmConfig<C> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self,D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Owned<C> {
            split: crate::state_config::SplitStrategy<C>,
            weights: crate::state_config::WeightSpec,
            vertex_constraints: Vec<crate::state_config::VertexConstraintKind>,
            seeds: crate::state_config::SeedCompositor<C>,
            metis: crate::state_config::MetisParams,
            mode_label: Option<String>,
        }
        let value=Owned::<C>::deserialize(d)?;
        let label=match value.mode_label.as_deref() {
            None=>None,Some("unweighted")=>Some("unweighted"),
            Some("partisan-weighted")=>Some("partisan-weighted"),Some("proportional")=>Some("proportional"),
            Some(label)=>return Err(serde::de::Error::custom(format!("Unknown native mode label: {label}"))),
        };
        Ok(Self{split:value.split,weights:value.weights,vertex_constraints:value.vertex_constraints,seeds:value.seeds,metis:value.metis,mode_label:label})
    }
}

/// Native metadata keeps its full unsigned count domain independently of
/// the pointer width used by an executable engine or browser allocator.
pub trait CountDomain:Copy+std::fmt::Debug+Serialize+serde::de::DeserializeOwned{
    fn to_u64(self)->u64;
}
impl CountDomain for usize{fn to_u64(self)->u64{self as u64}}
impl CountDomain for u64{fn to_u64(self)->u64{self}}
pub type ReadOnlyRecipe=Recipe<u64>;
pub type ReadOnlyResolvedState=ResolvedState<u64>;

pub const LEGACY_SCHEMA: &str = "bisect-native-state-replay-recipe-v1";
pub const SCHEMA: &str = "bisect-native-state-replay-recipe-v2";
pub const EXECUTION_SCHEMA:&str="bisect-native-state-replay-recipe-v3";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub role: String,
    pub path: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedState<C = usize> {
    pub state_code: String,
    pub state_name: String,
    pub year: String,
    pub districts: C,
    pub chamber: String,
    pub resolution: String,
    pub plan_resolution: String,
    pub population_source: String,
    pub seats_per_district: C,
    pub total_seats: C,
    pub balance_tolerance_fraction: f64,
    pub algorithm: AlgorithmConfig<C>,
    pub coi_weights: Option<std::path::PathBuf>,
    pub multiscale_fine: String,
    pub multiscale_coarse: String,
    pub smc_resample_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe<C = usize> {
    pub schema_version: String,
    pub state: ResolvedState<C>,
    pub original_working_directory: String,
    pub adjacency_path: String,
    pub source_graph_sha256: String,
    pub sources: Vec<SourceIdentity>,
    pub source_binding_scope: String,
    /// Native recursive dispatcher selection, distinct from configured engine.
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub recursive_metis_backend: Option<RecursiveBackend>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub execution:Option<NativeExecution>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeExecution{
    pub balance_retry_attempt:u32,
    #[serde(with="optional_seed")]
    pub accepted_seed:Option<u64>,
    pub multiscale_sampler:Option<NativeMultiscaleSampler>,
}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
pub enum NativeMultiscaleSampler{
    #[serde(rename="rand-0.8.6-smallrng-native64-wilson")]
    Native64,
    #[serde(rename="rand-0.8.6-smallrng-native32-wilson")]
    Native32,
}
pub fn native_multiscale_sampler()->NativeMultiscaleSampler{
    if usize::BITS==64{NativeMultiscaleSampler::Native64}else{NativeMultiscaleSampler::Native32}
}
pub fn state_uses_multiscale<C:CountDomain>(state:&ResolvedState<C>)->bool{
    let dispatched=matches!(state.algorithm.split,crate::state_config::SplitStrategy::Bisect)
        ||matches!(state.algorithm.split,crate::state_config::SplitStrategy::NWay)&&state.districts.to_u64()<=1;
    dispatched&&matches!(state.algorithm.seeds,crate::state_config::SeedCompositor::MultiScale{..}|crate::state_config::SeedCompositor::MultiScaleAdaptive{..})
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum RecursiveBackend { MetisCore, CFfi }
impl RecursiveBackend {
 pub fn engine(self)->bisect_apportion::split::MetisEngine{match self{Self::MetisCore=>bisect_apportion::split::MetisEngine::RedistMetis,Self::CFfi=>bisect_apportion::split::MetisEngine::CFfi}}
}
/// Native shared recursive dispatch has a compile-time choice. This does not
/// describe a WASM sidecar's dynamically selected execution scope.
pub fn native_compiled_recursive_backend()->RecursiveBackend{
 if cfg!(feature="c-ffi-engine"){RecursiveBackend::CFfi}else{RecursiveBackend::MetisCore}
}

#[derive(Debug,Clone,Serialize)]
pub struct BackendResolution {
 pub engine:bisect_apportion::split::MetisEngine,
 pub recursive_backend:Option<RecursiveBackend>,
 pub evidence:&'static str,
 pub bound_to_recipe:bool,
}
fn legacy_native_backend(version:&str)->Option<RecursiveBackend>{
 match version {
  "METIS 5.1.0 (vendored C FFI via metis-rs 0.2)"=>Some(RecursiveBackend::CFfi),
  "metis-core (pure Rust workspace engine)"=>Some(RecursiveBackend::MetisCore),
  _=>None,
 }
}

impl<C:CountDomain> Recipe<C> {
    pub fn manifest_seed(&self)->Option<u64>{self.execution.as_ref().map(|x|x.accepted_seed).unwrap_or(self.state.algorithm.metis.seed)}

    /// Legacy native version text is a recorded claim, outside the v1 digest.
    pub fn resolve_replay_backend(&self,native_version:&str)->Result<BackendResolution,String>{
        if !self.requires_recursive_backend(){return Ok(BackendResolution{engine:self.state.algorithm.metis.engine,recursive_backend:None,evidence:"configured-engine-or-unused-recursive-backend",bound_to_recipe:true});}
        let legacy=legacy_native_backend(native_version);
        if let Some(backend)=self.recursive_metis_backend {
            if legacy.is_some_and(|claim|claim!=backend){return Err("Effective recursive backend evidence contradicts the manifest native version claim.".into());}
            Ok(BackendResolution{engine:backend.engine(),recursive_backend:Some(backend),evidence:if self.schema_version==EXECUTION_SCHEMA{"recipe-v3"}else{"recipe-v2"},bound_to_recipe:true})
        }else if let Some(backend)=legacy {
            Ok(BackendResolution{engine:backend.engine(),recursive_backend:Some(backend),evidence:"legacy-native-version-claim",bound_to_recipe:false})
        }else{Err("Recorded recipe lacks effective recursive METIS backend evidence and an unambiguous native version claim; no historical backend is invented.".into())}
    }

    /// These native methods explicitly forward the configured engine or do not
    /// use the shared native dispatcher. The historical field/API name says
    /// recursive; NWay also uses that compile-time backend and requires evidence.
    pub fn requires_recursive_backend(&self)->bool{
        self.state.districts.to_u64()>1 && !matches!(self.state.algorithm.split,
            crate::state_config::SplitStrategy::ApportionRegions|
            crate::state_config::SplitStrategy::MovingKnife{..}|crate::state_config::SplitStrategy::Spectral{..}|
            crate::state_config::SplitStrategy::Regionalization|crate::state_config::SplitStrategy::FlowConstruction)
    }
    pub fn replay_metis_engine(&self)->Result<bisect_apportion::split::MetisEngine,String>{
        if self.requires_recursive_backend(){
            self.recursive_metis_backend.map(RecursiveBackend::engine).ok_or_else(||"Recorded recipe lacks effective recursive METIS backend evidence; historical configuration is insufficient for faithful replay.".into())
        }else{Ok(self.state.algorithm.metis.engine)}
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != SCHEMA && self.schema_version != LEGACY_SCHEMA && self.schema_version != EXECUTION_SCHEMA { return Err("Unsupported native replay recipe version.".into()); }
        if (self.schema_version!=LEGACY_SCHEMA)!=self.recursive_metis_backend.is_some(){return Err("Native recipe version and effective recursive backend evidence differ.".into());}
        if (self.schema_version==EXECUTION_SCHEMA)!=self.execution.is_some(){return Err("Native recipe version and execution evidence differ.".into());}
        if let Some(execution)=&self.execution{
            if execution.balance_retry_attempt>crate::balance_retry::MAX_BALANCE_RETRIES
                ||execution.accepted_seed!=crate::balance_retry::balance_retry_seed(self.state.algorithm.metis.seed,execution.balance_retry_attempt){
                return Err("Native accepted seed does not follow its recorded original seed and retry schedule.".into());
            }
            if state_uses_multiscale(&self.state)!=execution.multiscale_sampler.is_some(){return Err("Native multiscale sampler evidence differs from the consumed compositor.".into());}
        }
        let state = &self.state;
        if state.state_code.is_empty() || state.year.is_empty() || state.districts.to_u64() == 0
            || state.seats_per_district.to_u64() == 0 || state.total_seats.to_u64() == 0
            || !state.balance_tolerance_fraction.is_finite()
            || !(0.0001..=0.5).contains(&state.balance_tolerance_fraction)
            || !state.smc_resample_threshold.is_finite()
        { return Err("Invalid resolved native replay scope or numeric settings.".into()); }
        let safe = |s: &str| !s.is_empty() && s.len() <= 4096 && !s.contains('\0');
        if !safe(&self.adjacency_path) || !safe(&self.original_working_directory)
            || !is_hash(&self.source_graph_sha256) || self.sources.is_empty() || self.sources.len() > 64
        { return Err("Invalid native replay source records.".into()); }
        let mut roles = std::collections::BTreeSet::new();
        for source in &self.sources {
            if !safe(&source.path) || !safe(&source.role) || !roles.insert(&source.role)
                || source.sha256.as_ref().is_some_and(|s| !is_hash(s))
            { return Err("Invalid or duplicate native replay source identity.".into()); }
        }
        Ok(())
    }

    /// A typed versioned digest: struct field order and enum representations are fixed
    /// by this version. Seed transport is decimal text, including max-u64.
    pub fn digest(&self) -> Result<String, String> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
    }

    pub fn from_value(value: serde_json::Value) -> Result<Self, String> {
        let recipe: Self = serde_json::from_value(value).map_err(|e| format!("Invalid native replay recipe: {e}"))?;
        recipe.validate()?;
        Ok(recipe)
    }
}

fn is_hash(s: &str) -> bool {
    s.strip_prefix("sha256:").is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()))
}

pub mod optional_seed {
    use serde::{Deserialize, Serialize};
    pub fn serialize<S: serde::Serializer>(value: &Option<u64>, s: S) -> Result<S::Ok, S::Error> {
        value.map(|v| v.to_string()).serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Exact { Text(String), Integer(u64) }
        Option::<Exact>::deserialize(d)?.map(|value| match value {
            Exact::Integer(v) => Ok(v),
            Exact::Text(v) if !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()) => v.parse().map_err(serde::de::Error::custom),
            Exact::Text(_) => Err(serde::de::Error::custom("Native replay seed must be unsigned decimal text.")),
        }).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn example_recipe()->Recipe{
        let args=crate::state_command::parse(vec!["--state".into(),"RI".into(),"--search".into(),"single".into()]).unwrap();
        Recipe{schema_version:SCHEMA.into(),state:ResolvedState{state_code:"RI".into(),state_name:"rhode_island".into(),year:"2020".into(),districts:2,chamber:"congressional".into(),resolution:"tract".into(),plan_resolution:"tract".into(),population_source:"total".into(),seats_per_district:1,total_seats:2,balance_tolerance_fraction:0.005,algorithm:AlgorithmConfig::from_state_args(&args),coi_weights:None,multiscale_fine:"tract".into(),multiscale_coarse:"county".into(),smc_resample_threshold:0.5},original_working_directory:"C:/recorded".into(),adjacency_path:"C:/recorded/graph.adj.bin".into(),source_graph_sha256:format!("sha256:{}","0".repeat(64)),sources:vec![SourceIdentity{role:"adjacency-binary".into(),path:"C:/recorded/graph.adj.bin".into(),sha256:Some(format!("sha256:{}","1".repeat(64)))}],source_binding_scope:"Observed sources only".into(),recursive_metis_backend:Some(RecursiveBackend::CFfi),execution:None}
    }
    #[test]
    fn execution_v3_binds_retry_schedule_and_sampler_without_changing_legacy_wire(){
        let mut r=example_recipe();let v2=r.digest().unwrap();
        assert!(serde_json::to_value(&r).unwrap().get("execution").is_none());
        r.schema_version=EXECUTION_SCHEMA.into();assert!(r.validate().is_err());
        r.state.algorithm.metis.seed=Some(u64::MAX);
        r.execution=Some(NativeExecution{balance_retry_attempt:1,accepted_seed:Some(0),multiscale_sampler:None});
        r.validate().unwrap();assert_eq!(r.manifest_seed(),Some(0));
        let wire=serde_json::to_value(&r).unwrap();assert_eq!(wire["execution"]["accepted_seed"],"0");
        assert_eq!(r.digest().unwrap(),Recipe::<usize>::from_value(wire.clone()).unwrap().digest().unwrap());
        let bound=r.digest().unwrap();r.execution.as_mut().unwrap().accepted_seed=Some(1);assert!(r.validate().is_err());
        r.execution.as_mut().unwrap().accepted_seed=Some(0);r.execution.as_mut().unwrap().balance_retry_attempt=51;assert!(r.validate().is_err());
        r.execution.as_mut().unwrap().balance_retry_attempt=1;assert_eq!(r.digest().unwrap(),bound);
        r.state.algorithm.seeds=crate::state_config::SeedCompositor::MultiScale{total_steps:64,p:0.5,alpha:0.3};assert!(r.validate().is_err());
        r.execution.as_mut().unwrap().multiscale_sampler=Some(NativeMultiscaleSampler::Native64);r.validate().unwrap();
        assert_ne!(bound,r.digest().unwrap());
        let mut unknown=serde_json::to_value(&r).unwrap();unknown["execution"]["multiscale_sampler"]=serde_json::json!("unknown");assert!(Recipe::<usize>::from_value(unknown).is_err());
        r.schema_version=SCHEMA.into();assert!(r.validate().is_err());
        let fresh=example_recipe();assert_eq!(v2,fresh.digest().unwrap());
    }
    #[test]
    fn execution_v3_records_absent_seed_and_does_not_activate_unused_multiscale(){
        let mut r=example_recipe();r.schema_version=EXECUTION_SCHEMA.into();r.state.algorithm.metis.seed=None;
        r.state.algorithm.split=crate::state_config::SplitStrategy::NWay;
        r.state.algorithm.seeds=crate::state_config::SeedCompositor::MultiScaleAdaptive{total_steps:64,p:0.5,target_accept:0.6,adapt_interval:8};
        r.execution=Some(NativeExecution{balance_retry_attempt:0,accepted_seed:None,multiscale_sampler:None});r.validate().unwrap();
        assert!(serde_json::to_value(&r).unwrap()["execution"]["accepted_seed"].is_null());
        r.state.districts=1;assert!(r.validate().is_err());
        r.execution.as_mut().unwrap().multiscale_sampler=Some(NativeMultiscaleSampler::Native64);r.validate().unwrap();
    }
    #[test]
    fn effective_recursive_backend_is_distinct_from_configured_backend(){
        let mut recipe=example_recipe();recipe.state.algorithm.metis.engine=bisect_apportion::split::MetisEngine::RedistMetis;
        assert_eq!(recipe.replay_metis_engine().unwrap(),bisect_apportion::split::MetisEngine::CFfi);
        recipe.state.algorithm.split=crate::state_config::SplitStrategy::NWay;
        assert!(recipe.requires_recursive_backend());
        assert_eq!(recipe.replay_metis_engine().unwrap(),bisect_apportion::split::MetisEngine::CFfi);
        recipe.state.algorithm.split=crate::state_config::SplitStrategy::ApportionRegions;
        assert_eq!(recipe.replay_metis_engine().unwrap(),bisect_apportion::split::MetisEngine::RedistMetis);
    }
    #[test]
    fn legacy_recipe_does_not_invent_effective_recursive_dispatch(){
        let mut recipe=example_recipe();recipe.schema_version=LEGACY_SCHEMA.into();recipe.recursive_metis_backend=None;
        recipe.validate().unwrap();assert!(recipe.replay_metis_engine().unwrap_err().contains("lacks effective"));
        let wire=serde_json::to_value(&recipe).unwrap();assert!(wire.get("recursive_metis_backend").is_none());
        let restored=Recipe::<usize>::from_value(wire).unwrap();assert_eq!(restored.digest().unwrap(),recipe.digest().unwrap());
        recipe.state.districts=1;assert!(!recipe.requires_recursive_backend());assert!(recipe.replay_metis_engine().is_ok());
    }
    #[test]
    fn versioned_effective_backend_evidence_is_mandatory_and_bound(){
        let mut recipe=example_recipe();let original=recipe.digest().unwrap();
        assert_eq!(serde_json::to_value(&recipe).unwrap()["recursive_metis_backend"],"c-ffi");
        recipe.recursive_metis_backend=Some(RecursiveBackend::MetisCore);assert_ne!(original,recipe.digest().unwrap());
        recipe.recursive_metis_backend=None;assert!(recipe.validate().is_err());
        recipe.schema_version=LEGACY_SCHEMA.into();recipe.recursive_metis_backend=Some(RecursiveBackend::CFfi);assert!(recipe.validate().is_err());
    }
    #[test]
    fn unambiguous_legacy_version_is_a_separate_unbound_claim(){
        let mut recipe=example_recipe();recipe.schema_version=LEGACY_SCHEMA.into();recipe.recursive_metis_backend=None;
        let before=recipe.digest().unwrap();
        let c=recipe.resolve_replay_backend("METIS 5.1.0 (vendored C FFI via metis-rs 0.2)").unwrap();
        assert_eq!(c.engine,bisect_apportion::split::MetisEngine::CFfi);assert!(!c.bound_to_recipe);assert_eq!(c.evidence,"legacy-native-version-claim");
        let rust=recipe.resolve_replay_backend("metis-core (pure Rust workspace engine)").unwrap();assert_eq!(rust.engine,bisect_apportion::split::MetisEngine::RedistMetis);
        for missing in ["","METIS 5.1.0","unknown","metis-core"]{assert!(recipe.resolve_replay_backend(missing).is_err());}
        assert_eq!(recipe.digest().unwrap(),before);
    }
    #[test]
    fn bound_backend_and_known_native_version_cannot_contradict(){
        let recipe=example_recipe();assert!(recipe.resolve_replay_backend("metis-core (pure Rust workspace engine)").unwrap_err().contains("contradicts"));
        let exact=recipe.resolve_replay_backend("METIS 5.1.0 (vendored C FFI via metis-rs 0.2)").unwrap();assert!(exact.bound_to_recipe);assert_eq!(exact.evidence,"recipe-v2");
    }
    #[test]
    fn all_native_partition_presets_roundtrip_resolved_settings() {
        use clap::ValueEnum;
        for preset in crate::args::PartitionMode::value_variants() {
            let name=preset.to_possible_value().unwrap().get_name().to_owned();
            let args=crate::state_command::parse(vec!["--state".into(),"RI".into(),"--partition-mode".into(),name.clone(),"--seed".into(),"17".into()]).unwrap();
            let algorithm=AlgorithmConfig::from_state_args(&args);
            let json=serde_json::to_value(&algorithm).unwrap();
            let restored:AlgorithmConfig=serde_json::from_value(json).unwrap();
            assert_eq!(format!("{algorithm:?}"),format!("{restored:?}"),"{name}");
        }
    }
    #[test]
    fn settings_reject_unknown_controls_and_ambiguous_seeds() {
        let args=crate::state_command::parse(vec!["--state".into(),"RI".into()]).unwrap();
        let value=serde_json::to_value(AlgorithmConfig::from_state_args(&args)).unwrap();
        for key in ["reset","output_dir","run_program"] {
            let mut changed=value.clone();changed[key]=serde_json::json!(true);
            assert!(serde_json::from_value::<AlgorithmConfig>(changed).is_err());
        }
        for seed in [serde_json::json!(-1),serde_json::json!("-1"),serde_json::json!("18446744073709551616"),serde_json::json!("1e3")] {
            let mut changed=value.clone();changed["metis"]["seed"]=seed;
            assert!(serde_json::from_value::<AlgorithmConfig>(changed).is_err());
        }
    }
    #[test]
    fn all_search_compositors_roundtrip_with_exact_unsigned_seed() {
        let searches = ["single", "multi", "convergence", "percentile", "bisection-ensemble", "flip", "short-burst", "short-burst-forest", "short-burst-merge-split", "forest-recom", "multiscale", "merge-split", "multiscale-adaptive", "parallel-tempering", "vra-recom", "smc-percentile"];
        for search in searches {
            let args = crate::state_command::parse(vec!["--state".into(), "RI".into(), "--search".into(), search.into(), "--seed".into(), u64::MAX.to_string()]).unwrap();
            let algorithm = AlgorithmConfig::from_state_args(&args);
            let value = serde_json::to_value(&algorithm).unwrap();
            assert_eq!(value["metis"]["seed"], u64::MAX.to_string());
            let restored: AlgorithmConfig = serde_json::from_value(value).unwrap();
            assert_eq!(format!("{algorithm:?}"), format!("{restored:?}"));
        }
    }
}

pub fn graph_digest(graph: &crate::adjacency_loader::LoadedGraph) -> Result<String, String> {
    let mut edges: Vec<_> = graph.edge_weights.iter().map(|(&(a,b),&w)| (a,b,w)).collect();
    edges.sort_by_key(|&(a,b,_)| (a,b));
    let geoids: std::collections::BTreeMap<_,_> = graph.index_to_geoid.iter().map(|(&i,id)| (i.to_string(),id)).collect();
    let value = serde_json::json!({"schema_version":"bisect-loaded-native-graph-v1", "adjacency":graph.adjacency,
        "population":graph.vertex_weights,"edges":edges,"geoids":geoids,"n_vertices":graph.n_vertices,
        "n_edges":graph.n_edges,"centroids":graph.tract_centroids});
    // Explicit lexical keys avoid depending on serde_json's map feature flags.
    let ordered: std::collections::BTreeMap<_,_> = value.as_object().ok_or("Invalid native graph snapshot")?.iter().collect();
    Ok(format!("sha256:{:x}", Sha256::digest(serde_json::to_vec(&ordered).map_err(|e|e.to_string())?)))
}

