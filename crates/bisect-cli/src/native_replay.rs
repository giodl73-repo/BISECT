//! Native IO adapter for resolved replay recipes. Merely loading JSON never runs it.
use std::path::{Path, PathBuf};
use bisect_runner::native_replay_recipe::{Recipe, ResolvedState, SourceIdentity, SCHEMA};
use crate::runner::{StateConfig,SplitStrategy,SeedCompositor};
use crate::adjacency_loader::LoadedGraph;

#[derive(Debug, clap::Args)]
pub struct ReplayNativeArgs {
    #[arg(long)] pub manifest: PathBuf,
    #[arg(long)] pub output_dir: PathBuf,
    #[arg(long)] pub label: String,
}

fn absolute(path: &Path, cwd: &Path) -> PathBuf {
    if path.is_absolute() { path.to_owned() } else { cwd.join(path) }
}
fn utf8(path: &Path) -> Result<String, String> {
    path.to_str().map(str::to_owned).ok_or_else(|| "Native replay source paths must be UTF-8.".into())
}

pub(crate) use bisect_runner::native_replay_recipe::graph_digest;

fn identity(role: &str, path: &Path, cwd: &Path) -> Result<SourceIdentity,String> {
    let path = absolute(path,cwd);
    let sha256 = if path.is_file() {
        Some(format!("sha256:{}",bisect_report::sha256_file(&path).map_err(|e|e.to_string())?))
    } else if path.exists() { return Err(format!("Native replay source is not a file: {}",path.display())); }
    else {None};
    Ok(SourceIdentity{role:role.into(),path:utf8(&path)?,sha256})
}

pub(crate) fn capture(cfg: &StateConfig, graph: &LoadedGraph, adjacency: &Path, geometry: Option<&Path>, tolerance: f64) -> Result<Option<serde_json::Value>,String> {
    capture_with_secondary(cfg,graph,adjacency,geometry,tolerance,None,None)
}

pub(crate) fn capture_with_secondary(cfg: &StateConfig, graph: &LoadedGraph, adjacency: &Path, geometry: Option<&Path>, tolerance: f64, secondary: Option<&Path>, execution:Option<bisect_runner::native_replay_recipe::NativeExecution>) -> Result<Option<serde_json::Value>,String> {
    let cwd = cfg.replay_auxiliary_root.clone().map(Ok).unwrap_or_else(std::env::current_dir).map_err(|e|e.to_string())?;
    let adjacency = absolute(adjacency,&cwd);
    let text = utf8(&adjacency)?;
    let (binary,stem) = if let Some(stem)=text.strip_suffix(".adj.bin") {(adjacency.clone(),stem.to_owned())}
        else {(adjacency.with_extension("adj.bin"),utf8(&adjacency.with_extension(""))?)};
    // Do not turn a legacy pickle claim into implicit pickle execution on replay.
    if !binary.is_file() { eprintln!("WARNING: no native binary graph; resolved replay recipe not recorded."); return Ok(None); }
    let mut sources=vec![identity("adjacency-request",&adjacency,&cwd)?,identity("adjacency-binary",&binary,&cwd)?,
        identity("geoid-companion",Path::new(&format!("{stem}_geoids.json")),&cwd)?,
        identity("centroid-companion",Path::new(&format!("{stem}_centroids.json")),&cwd)?];
    if let Some(path)=secondary {
        let requested=absolute(path,&cwd);let text=utf8(&requested)?;
        let (binary,stem)=if let Some(stem)=text.strip_suffix(".adj.bin"){(requested.clone(),stem.to_owned())}else{(requested.with_extension("adj.bin"),utf8(&requested.with_extension(""))?)};
        if !binary.is_file(){eprintln!("WARNING: no binary multiscale fine graph; replay recipe not recorded.");return Ok(None);}
        sources.push(identity("multiscale-fine-request",&requested,&cwd)?);
        sources.push(identity("multiscale-fine-binary",&binary,&cwd)?);
        sources.push(identity("multiscale-fine-geoid-companion",Path::new(&format!("{stem}_geoids.json")),&cwd)?);
        sources.push(identity("multiscale-fine-centroid-companion",Path::new(&format!("{stem}_centroids.json")),&cwd)?);
    }
    if cfg.algo.vertex_constraints.contains(&bisect_runner::state_config::VertexConstraintKind::Area)||matches!(cfg.algo.split,SplitStrategy::CompactBisect{..}){
        let relative=bisect_runner::native_tiger_geometry::relative_path(&cfg.state_code,&cfg.year).ok_or("Native algorithm TIGER path unavailable")?;
        let path=cwd.join(relative);
        for ext in ["shp","dbf","shx","prj"]{sources.push(identity(&format!("algorithm-tiger-{ext}"),&path.with_extension(ext),&cwd)?);}
    }
    if let Some(path)=geometry { for suffix in ["shp","dbf","shx","prj"] {sources.push(identity(&format!("geometry-{suffix}"),&path.with_extension(suffix),&cwd)?);} }
    if let Some(path)=&cfg.coi_weights {sources.push(identity("explicit-coi",path,&cwd)?);}
    if let Some(path)=&cfg.algo.weights.partisan_shares {sources.push(identity("explicit-partisan",path,&cwd)?);}
    let auxiliary_root=cfg.replay_auxiliary_root.as_deref().unwrap_or(&cwd);
    for(kind,active)in [("economic",cfg.algo.weights.economic_character),("housing",cfg.algo.weights.housing_character)]{
        if active{let relative=bisect_core::native_input_paths::character_relative_path(&cfg.state_name,&cfg.year,kind).unwrap();sources.push(identity(&format!("implicit-{kind}"),&auxiliary_root.join(relative),&cwd)?);}
    }
    if cfg.algo.weights.minority_weighting||matches!(cfg.algo.split,bisect_runner::state_config::SplitStrategy::VraSection{..})||matches!(cfg.algo.seeds,bisect_runner::state_config::SeedCompositor::VraRecom{..}){
        let relative=bisect_core::native_input_paths::demographic_relative_path(&cfg.state_name,&cfg.year);sources.push(identity("implicit-demographics",&auxiliary_root.join(relative),&cwd)?);
    }
    let recipe=Recipe{recursive_metis_backend:Some(bisect_runner::native_replay_recipe::native_compiled_recursive_backend()),schema_version:if execution.is_some(){bisect_runner::native_replay_recipe::EXECUTION_SCHEMA.into()}else{SCHEMA.into()},execution,original_working_directory:utf8(&cwd)?,adjacency_path:utf8(&adjacency)?,
        source_graph_sha256:graph_digest(graph)?,sources,
        source_binding_scope:"Observed native binary graph/companions, explicit auxiliaries, configured character-discovery paths and selected geometry. Single-district configured character sources are observed but not consumed. Other implicit data discovery retains native behavior; no complete atomic filesystem snapshot or upstream authenticity claim.".into(),
        state:ResolvedState{state_code:cfg.state_code.clone(),state_name:cfg.state_name.clone(),year:cfg.year.clone(),districts:cfg.effective_num_districts(),
            chamber:cfg.chamber.clone(),resolution:cfg.resolution.clone(),plan_resolution:cfg.plan_resolution.clone(),population_source:cfg.population_source.clone(),
            seats_per_district:cfg.seats_per_district,total_seats:cfg.total_seats,balance_tolerance_fraction:tolerance,algorithm:cfg.algo.clone(),coi_weights:cfg.coi_weights.clone(),
            multiscale_fine:cfg.multiscale_fine.clone(),multiscale_coarse:cfg.multiscale_coarse.clone(),smc_resample_threshold:cfg.smc_resample_threshold}};
    recipe.validate()?;
    let value=serde_json::to_value(recipe).map_err(|e|e.to_string())?;
    // JSON cannot represent all parsed native f64 values. Do not publish a
    // recipe that the typed reader cannot consume (or activate unused fields).
    Recipe::<usize>::from_value(value.clone())?;
    Ok(Some(value))
}

pub(crate) fn recipe_digest(value: serde_json::Value) -> Result<String,String> { Recipe::<usize>::from_value(value)?.digest() }

fn selected_sidecar(root:&Path,relative:&str)->Result<PathBuf,String>{
    let p=Path::new(relative);
    if p.is_absolute() || p.components().any(|c|!matches!(c,std::path::Component::Normal(_))){return Err("Native replay context must be a relative selected sidecar.".into());}
    Ok(root.join(p))
}

pub(crate) fn load(manifest:&bisect_report::PlanManifest,root:&Path,check_sources:bool)->Result<Option<Recipe>,String>{
    let Some(value)=manifest.native_replay_recipe.clone() else {
        // A new bound record must not be downgraded by stripping the recipe.
        if let Some(relative)=manifest.rctx_path.as_deref() {
            let path=selected_sidecar(root,relative)?;
            if path.is_file() {
                let context=rplan_io::read_rctx_str(&std::fs::read_to_string(path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
                if context.source_hashes.entries.contains_key("native_replay_recipe") {
                    return Err("Native replay recipe is missing from a bound manifest.".into());
                }
            }
        }
        return Ok(None)
    };
    let recipe=Recipe::from_value(value)?;let state=&recipe.state;
    let seed=manifest.engine_seed.or_else(||manifest.seed.and_then(|s|u64::try_from(s).ok()));
    if state.state_code!=manifest.state_code || state.year!=manifest.year || state.chamber!=manifest.chamber || state.districts!=manifest.num_districts
        || recipe.manifest_seed()!=seed || state.balance_tolerance_fraction*100.0!=manifest.balance_tolerance_pct {
        return Err("Native replay recipe differs from recorded manifest scope, seed or tolerance.".into());
    }
    let relative=manifest.rctx_path.as_deref().ok_or("Native replay recipe requires its original context sidecar.")?;
    let context=rplan_io::read_rctx_str(&std::fs::read_to_string(selected_sidecar(root,relative)?).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let actual=context.compute_context_hash().map_err(|e|e.to_string())?;
    if actual!=context.context_hash || manifest.context_hash.as_deref()!=Some(actual.as_str())
        || context.source_hashes.entries.get("native_replay_recipe")!=Some(&recipe.digest()?) {
        return Err("Native replay recipe is not bound to the recorded context.".into());
    }
    ensure_replay_backend(&recipe,&manifest.gpmetis_version)?;
    if check_sources { check_source_identities(&recipe)?; }
    Ok(Some(recipe))
}

fn ensure_replay_backend(recipe:&Recipe,native_version:&str)->Result<(),String>{
    if bisect_runner::native_replay_recipe::state_uses_multiscale(&recipe.state){
        let observed=recipe.execution.as_ref().and_then(|x|x.multiscale_sampler).ok_or("Recorded multiscale sampler profile is unknown; no native pointer-width profile is invented")?;
        if observed!=bisect_runner::native_replay_recipe::native_multiscale_sampler(){return Err("Recorded native multiscale sampler differs from this binary's pointer-width profile".into());}
    }
    let resolution=recipe.resolve_replay_backend(native_version)?;let engine=resolution.engine;
    if !resolution.bound_to_recipe{eprintln!("WARNING: Effective backend follows the recorded legacy native version claim, which is not bound by the v1 recipe digest.");}
    if recipe.requires_recursive_backend() && resolution.recursive_backend!=Some(bisect_runner::native_replay_recipe::native_compiled_recursive_backend()){
        return Err("Recorded effective recursive METIS backend differs from this native binary; no backend is substituted.".into());
    }
    #[cfg(not(feature="c-ffi-engine"))]
    if recipe.state.districts>1 && (recipe.requires_recursive_backend()||matches!(recipe.state.algorithm.split,bisect_runner::state_config::SplitStrategy::NWay|bisect_runner::state_config::SplitStrategy::ApportionRegions)) && engine==bisect_apportion::split::MetisEngine::CFfi{return Err("Recorded C METIS backend is not available in this native binary.".into());}
    let _=engine;Ok(())
}

fn check_source_identities(recipe:&Recipe)->Result<(),String>{
    if bisect_runner::native_tiger_geometry::consumed(recipe){bisect_runner::native_tiger_geometry::records(recipe)?;}
    let dispatched=matches!(recipe.state.algorithm.split,SplitStrategy::Bisect)||(matches!(recipe.state.algorithm.split,SplitStrategy::NWay)&&recipe.state.districts<=1);
    let active=dispatched&&matches!(recipe.state.algorithm.seeds,SeedCompositor::MultiScale{..}|SeedCompositor::MultiScaleAdaptive{..});
    if active&&matches!(recipe.state.multiscale_fine.as_str(),"bg"|"block_group"){
        let selected=recipe.sources.iter().find(|r|r.role=="multiscale-fine-request").ok_or("Missing recorded multiscale fine input; current discovery is not historical evidence")?;
        let request=Path::new(&selected.path);let text=utf8(request)?;
        let(binary,stem)=if let Some(stem)=text.strip_suffix(".adj.bin"){(request.to_path_buf(),stem.to_owned())}else{(request.with_extension("adj.bin"),utf8(&request.with_extension(""))?)};
        for(role,path,required)in[("multiscale-fine-binary",binary,true),("multiscale-fine-geoid-companion",PathBuf::from(format!("{stem}_geoids.json")),true),("multiscale-fine-centroid-companion",PathBuf::from(format!("{stem}_centroids.json")),false)]{
            let source=recipe.sources.iter().find(|r|r.role==role).ok_or_else(||format!("Missing multiscale source identity: {role}"))?;
            if source.path!=utf8(&path)?||required&&source.sha256.is_none(){return Err(format!("Multiscale source path linkage or required input differs: {role}"));}
        }
    }
    // Check path linkage before reading any explicit configured auxiliary.
    let cwd=Path::new(&recipe.original_working_directory);
    for(role,path)in[("explicit-coi",recipe.state.coi_weights.as_deref()),("explicit-partisan",recipe.state.algorithm.weights.partisan_shares.as_deref())]{
        if let Some(path)=path{
            let expected=utf8(&absolute(path,cwd))?;
            let recorded=recipe.sources.iter().find(|s|s.role==role).ok_or_else(||format!("Native recipe has no explicit auxiliary source identity: {role}"))?;
            if expected!=recorded.path{return Err(format!("Configured native auxiliary path differs from recorded source identity: {role}"));}
        }
    }

    let composing=bisect_runner::native_weighting::route(&recipe.state.algorithm.weights,recipe.state.districts)==bisect_runner::native_weighting::Route::Compose;
    for(kind,active)in [("economic",recipe.state.algorithm.weights.economic_character),("housing",recipe.state.algorithm.weights.housing_character)]{
        if active{
            let role=format!("implicit-{kind}");let relative=bisect_core::native_input_paths::character_relative_path(&recipe.state.state_name,&recipe.state.year,kind).unwrap();let expected=utf8(&cwd.join(relative))?;
            if let Some(record)=recipe.sources.iter().find(|r|r.role==role){if record.path!=expected{return Err(format!("Native character-discovery path differs from source identity: {role}"));}}
            else if composing{return Err(format!("Missing recorded native character-discovery role: {role}"));}
        }
    }
    if recipe.state.algorithm.weights.minority_weighting||matches!(recipe.state.algorithm.split,bisect_runner::state_config::SplitStrategy::VraSection{..})||matches!(recipe.state.algorithm.seeds,bisect_runner::state_config::SeedCompositor::VraRecom{..}){
        let relative=bisect_core::native_input_paths::demographic_relative_path(&recipe.state.state_name,&recipe.state.year);let expected=utf8(&cwd.join(relative))?;
        if let Some(record)=recipe.sources.iter().find(|s|s.role=="implicit-demographics"){if record.path!=expected{return Err("Native demographic-discovery path differs from recorded source identity".into());}}
        else if composing&&recipe.state.algorithm.weights.minority_weighting||recipe.state.districts>1&&(matches!(recipe.state.algorithm.split,bisect_runner::state_config::SplitStrategy::VraSection{..})||matches!(recipe.state.algorithm.seeds,bisect_runner::state_config::SeedCompositor::VraRecom{..})){return Err("Missing recorded native demographic-discovery identity".into());}
    }
    for source in &recipe.sources {
        let path=Path::new(&source.path);
        if !path.is_absolute(){return Err("Native replay source identity must resolve to an absolute native path.".into());}
        let current=identity(&source.role,path,Path::new(&recipe.original_working_directory))?;
        if current.sha256!=source.sha256 {return Err(format!("Native replay source changed or is unavailable: {}",source.role));}
    }
    Ok(())
}

fn state_config(recipe:&Recipe,output_dir:PathBuf,label:String)->StateConfig{
    let s=&recipe.state;let cwd=Path::new(&recipe.original_working_directory);let mut algo=s.algorithm.clone();
    algo.weights.partisan_shares=algo.weights.partisan_shares.as_ref().map(|p|absolute(p,cwd));
    StateConfig{state_code:s.state_code.clone(),state_name:s.state_name.clone(),num_districts:s.districts,year:s.year.clone(),version:"verify".into(),output_dir,
        algo,balance_tolerance:Some(s.balance_tolerance_fraction),coi_weights:s.coi_weights.as_ref().map(|p|absolute(p,cwd)),position:0,debug:false,reset:false,reprocess:false,time_partition:false,
        num_districts_override:Some(s.districts),chamber:s.chamber.clone(),label:Some(label),population_source:s.population_source.clone(),write_manifest:true,force:true,
        resolution:s.resolution.clone(),plan_resolution:s.plan_resolution.clone(),seats_per_district:s.seats_per_district,total_seats:s.total_seats,
        adjacency_override:Some(PathBuf::from(&recipe.adjacency_path)),replay_auxiliary_root:Some(cwd.to_owned()),replay_multiscale_fine_path:recipe.sources.iter().find(|r|r.role=="multiscale-fine-request").map(|r|PathBuf::from(&r.path)),multiscale_fine:s.multiscale_fine.clone(),multiscale_coarse:s.multiscale_coarse.clone(),smc_resample_threshold:s.smc_resample_threshold}
}

pub fn run(args:ReplayNativeArgs)->anyhow::Result<()>{
    let manifest:bisect_report::PlanManifest=serde_json::from_str(&std::fs::read_to_string(&args.manifest)?)?;
    let root=args.manifest.parent().ok_or_else(||anyhow::anyhow!("Missing native replay plan root"))?;
    crate::verify::verify_manifest_ilp_audit_summary(&manifest,root)?;
    crate::verify::verify_manifest_rplan_audit_certificate(&manifest,root)?;
    let recipe=load(&manifest,root,true).map_err(anyhow::Error::msg)?.ok_or_else(||anyhow::anyhow!("A resolved native replay recipe is required."))?;
    let graph=crate::adjacency_loader::load_adjacency_pkl(Path::new(&recipe.adjacency_path)).map_err(anyhow::Error::msg)?;
    if graph_digest(&graph).map_err(anyhow::Error::msg)?!=recipe.source_graph_sha256 {anyhow::bail!("Native replay source graph differs from the recorded graph.");}
    let config=state_config(&recipe,args.output_dir,args.label);
    crate::runner::run_replay_state(&config,graph).map_err(anyhow::Error::msg)
}
