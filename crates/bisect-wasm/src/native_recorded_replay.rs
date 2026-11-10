//! Explicit selected-source replay through the existing State compositor/engine.
use serde::Deserialize;
use base64::{engine::general_purpose::STANDARD,Engine};
use serde_json::{json,Value};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
 pub sources:crate::native_recipe_sources::Input,
 pub label:String,
 pub created_at:String,
}

/// A selected original context is not reconstructed from coarse seed units.
/// Admit it only after the ordinary manifest/recipe byte checks and an exact
/// comparison with the complete fine graph and original native plan index.
fn checked_native_fine_context(bytes:&[u8],units:&rplan_core::PlanUnitIndex,graph:&bisect_runner::adjacency_loader::LoadedGraph)->Result<String,String>{
 let text=std::str::from_utf8(bytes).map_err(|e|e.to_string())?;
 let context=rplan_io::read_rctx_str(text).map_err(|e|e.to_string())?;
 if &context.units!=units || context.units.unit_kind!=rplan_core::UnitKind::BlockGroup || context.units.unit_ids.len()!=graph.n_vertices || context.units.unit_ids.iter().enumerate().any(|(i,id)|graph.index_to_geoid.get(&i)!=Some(id)) || context.populations.as_deref()!=Some(graph.vertex_weights.as_slice()) {
  return Err("Original native fine context differs from the complete original plan/source graph units or populations.".into());
 }
 let original=context.graph.as_ref().ok_or("Original native fine context has no physical graph.")?;
 if original.edge_semantics!=rplan_core::EdgeSemantics::Undirected || original.adjacency.len()!=graph.n_vertices || original.adjacency.iter().enumerate().any(|(from,edges)|edges.len()!=graph.adjacency[from].len() || edges.iter().zip(&graph.adjacency[from]).any(|(edge,&to)|edge.to as usize!=to || edge.kind!=rplan_core::EdgeKind::Boundary || edge.weight!=graph.edge_weights.get(&(from.min(to),from.max(to))).copied())) {
  return Err("Original native fine context differs from native ordered adjacency, boundary classification or physical weights.".into());
 }
 let subdivisions=rplan_core::SubdivisionContext{county_ids:Some(context.units.unit_ids.iter().map(|id|Some(id[..5].to_owned())).collect()),municipal_ids:None};
 if context.subdivisions.as_ref()!=Some(&subdivisions) || context.geometry.is_some(){return Err("Original native fine context differs from native county membership or inherits unsupported seed geometry.".into());}
 Ok(text.to_owned())
}

/// Preserve exact selected RCTX bytes and identity after the fine graph checks.
/// New replay labels/timestamps remain new RPLAN metadata, never original bytes.
fn bind_native_fine_context(exported:&Value,context_json:&str)->Result<Value,String>{
 use sha2::{Digest,Sha256};
 let context=rplan_io::read_rctx_str(context_json).map_err(|e|e.to_string())?;
 let mut document=rplan_io::read_rplan_str(exported["document_json"].as_str().ok_or("Missing fine replay export document.")?).map_err(|e|e.to_string())?;
 let old_context=rplan_io::read_rctx_str(exported["context_json"].as_str().ok_or("Missing fine replay export context.")?).map_err(|e|e.to_string())?;
 let a=&document.plan.units;let b=&context.units;
 if a.unit_kind!=b.unit_kind || a.state!=b.state || a.year!=b.year || a.canonical_order!=b.canonical_order || a.unit_ids!=b.unit_ids || old_context.populations!=context.populations {
  return Err("Executed fine export differs from the selected original fine context scope or populations.".into());
 }
 let a=old_context.graph.as_ref().ok_or("Executed fine export has no graph.")?;let b=context.graph.as_ref().ok_or("Selected original fine context has no graph.")?;
 if a.edge_semantics!=b.edge_semantics || a.adjacency.len()!=b.adjacency.len() || a.adjacency.iter().zip(&b.adjacency).any(|(x,y)|x.len()!=y.len() || x.iter().zip(y).any(|(x,y)|x.to!=y.to || x.weight!=y.weight)){
  return Err("Executed fine export differs from the selected original physical graph.".into());
 }
 let raw_sha256=format!("{:x}",Sha256::digest(context_json.as_bytes()));
 document.plan.units=context.units.clone();document.plan.validate().map_err(|e|e.to_string())?;
 document.provenance.source_hashes=context.source_hashes.entries.clone();
 document.provenance.producer.insert("native_fine_context_binding".into(),json!({"selected_context_sha256":raw_sha256,"context_hash":context.context_hash,"intermediate_export_context_hash":old_context.context_hash}));
 document.provenance.conversion_lineage=vec![json!({"operation":"recorded-native-fine-context-bind","original_edge_classification_retained":true,"selected_context_sha256":raw_sha256,"scope":"Original selected native context passed exact ordered fine graph/unit/population/physical weight checks before execution. Demographic and other retained context fields remain selected source claims, not regenerated or authenticated evidence."})];
 let document_json=rplan_io::write_rplan_string(&document).map_err(|e|e.to_string())?;rplan_io::read_rplan_str(&document_json).map_err(|e|e.to_string())?;
 let mut result=exported.clone();
 result["plan_hash"]=json!(document.plan.plan_hash().map_err(|e|e.to_string())?);result["context_hash"]=json!(context.context_hash);
 result["document"]=serde_json::to_value(&document).map_err(|e|e.to_string())?;result["document_json"]=json!(document_json);
 result["context"]=serde_json::from_str(context_json).map_err(|e|e.to_string())?;result["context_json"]=json!(context_json);
 result["original_context_source_sha256"]=json!(raw_sha256);
 result["source_boundary"]=json!("Exact selected original native fine RCTX bytes and unit identity retained after complete source/plan/fine-graph consistency checks. New replay RPLAN metadata remains new. Selected context auxiliary metadata, coordinates, publisher and original execution are not authenticated; no certification or full workflow/capacity claim.");
 Ok(result)
}


/// Preserve a selected original ordinary context only after all physical fields
/// match the independently selected native graph. Auxiliary context fields are
/// retained source claims, not freshly regenerated or authenticated observations.
fn checked_native_ordinary_context(bytes:&[u8],units:&rplan_core::PlanUnitIndex,graph:&bisect_runner::adjacency_loader::LoadedGraph)->Result<String,String>{
 let text=std::str::from_utf8(bytes).map_err(|e|e.to_string())?;
 let context=rplan_io::read_rctx_str(text).map_err(|e|e.to_string())?;
 if &context.units!=units || context.units.unit_ids.len()!=graph.n_vertices || context.units.unit_ids.iter().enumerate().any(|(i,id)|graph.index_to_geoid.get(&i).cloned().unwrap_or_else(||i.to_string())!=*id) || context.populations.as_deref()!=Some(graph.vertex_weights.as_slice()) {
  return Err("Original native ordinary context differs from the complete original plan/source graph units or populations.".into());
 }
 let original=context.graph.as_ref().ok_or("Original native ordinary context has no physical graph.")?;
 if original.edge_semantics!=rplan_core::EdgeSemantics::Undirected || original.adjacency.len()!=graph.n_vertices || original.adjacency.iter().enumerate().any(|(from,edges)|edges.len()!=graph.adjacency[from].len() || edges.iter().zip(&graph.adjacency[from]).any(|(edge,&to)|edge.to as usize!=to || edge.kind!=rplan_core::EdgeKind::Boundary || edge.weight!=graph.edge_weights.get(&(from.min(to),from.max(to))).copied())) {
  return Err("Original native ordinary context differs from native ordered adjacency, boundary classification or physical weights.".into());
 }
 let counties:Vec<_>=(0..graph.n_vertices).map(|i|graph.index_to_geoid.get(&i).and_then(|id|if id.len()>=5&&id.as_bytes()[..5].iter().all(u8::is_ascii_digit){Some(id[..5].to_owned())}else{None})).collect();
 let subdivisions=if counties.iter().any(Option::is_some){Some(rplan_core::SubdivisionContext{county_ids:Some(counties),municipal_ids:None})}else{None};
 if context.subdivisions!=subdivisions{return Err("Original native ordinary context differs from native report county membership.".into());}
 Ok(text.to_owned())
}

fn bind_native_ordinary_context(exported:&Value,context_json:&str)->Result<Value,String>{
 // The existing binder checks executed unit/population/physical graph equality;
 // it is independent of unit resolution. Keep the fine branch byte-identical.
 let mut result=bind_native_fine_context(exported,context_json)?;
 let mut document=rplan_io::read_rplan_str(result["document_json"].as_str().ok_or("Missing ordinary replay document.")?).map_err(|e|e.to_string())?;
 let binding=document.provenance.producer.remove("native_fine_context_binding").ok_or("Missing checked original context binding.")?;
 document.provenance.producer.insert("native_original_context_binding".into(),binding);
 document.provenance.conversion_lineage[0]["operation"]=json!("recorded-native-context-bind");
 document.provenance.conversion_lineage[0]["scope"]=json!("Original selected native context passed exact ordered ordinary graph/unit/population/physical weight and native report-county checks before execution. Retained demographic, geometry-hash and other auxiliary context fields remain selected source claims, not regenerated or authenticated evidence.");
 let text=rplan_io::write_rplan_string(&document).map_err(|e|e.to_string())?;rplan_io::read_rplan_str(&text).map_err(|e|e.to_string())?;
 result["document"]=serde_json::to_value(&document).map_err(|e|e.to_string())?;result["document_json"]=json!(text);
 result["source_boundary"]=json!("Exact selected original native ordinary RCTX bytes and unit identity retained after complete source/plan/physical-graph consistency checks. New replay metadata remains new. Retained auxiliary geometry hashes/demographics and original source/execution are not authenticated; no certification, fresh direct-State context parity or full workflow/capacity claim.");
 Ok(result)
}

pub(crate) fn replay(input:Input,execute:bool)->Result<Value,String>{
 // These are selected package-relative byte lookups, never native IO paths.
 let files=crate::package_verification::selected_bytes(input.sources.prechecks.files_base64.clone())?;
 let manifest:bisect_report::manifest_schema::ReadOnlyPlanManifest=serde_json::from_str(&input.sources.prechecks.manifest_json).map_err(|e|e.to_string())?;
 let manifest_path=input.sources.prechecks.manifest_path.clone();
 let (source_check,checked)=crate::native_recipe_sources::inspect_checked(input.sources)?;
 let mut output=json!({"schema_version":"bisect-native-recorded-replay-v1","status":"source-check-failed","source_check":source_check,"prepared":null,"replay":null,"comparison":null,"error":source_check["error"],"execution_performed":false,"recorded_paths_opened":false,"scope":"Explicit selected original source-bound replay. Native recipe, manifest/context and existing source bytes/graph are checked before preparation. Shared native State translation and existing population retries use the recorded exact seed, algorithm/search and fractional tolerance. Literal GEOID/district equality against the selected original RPLAN does not authenticate historical execution, unrecorded implicit inputs, upstream sources, optimality or legal validity. Recorded output/deletion controls and native paths are not executed."});
 let Some(checked)=checked else{return Ok(output)};
 let recorded_execution=checked.recipe.execution.clone();let requested_seed=checked.recipe.state.algorithm.metis.seed;
 output["backend_resolution"]=Value::Null;
 let mut original_fine_context=None;
 let mut original_ordinary_context=None;
 let prepared=(||->Result<(Value,BTreeMap<String,u64>),String>{
  let s=&checked.recipe.state;
  let backend=checked.recipe.resolve_replay_backend(&manifest.gpmetis_version)?;output["backend_resolution"]=serde_json::to_value(backend).map_err(|e|e.to_string())?;
  use bisect_runner::state_config::{VertexConstraintKind as V,SplitStrategy as S};
  let area_section=matches!(s.algorithm.split,S::AreaSection{..});
  let has_area=s.algorithm.vertex_constraints.contains(&V::Area);
  if area_section&&!has_area{return Err("Native AreaSection requires an Area constraint to load its computational TIGER data; recorded graph/report areas do not replace that native precondition.".into());}
  // AreaSection uses the Area presence flag and original population vector;
  // its computed interleaved constraint matrix is not passed to the partitioner.
  let native_area=area_section&&has_area;
  if !s.algorithm.vertex_constraints.is_empty()&&!matches!(s.algorithm.vertex_constraints.as_slice(),[V::Population])&&!native_area{return Err("Recorded non-population constraints need their selected native input translation before replay.".into());}
  let multiscale=bisect_runner::native_replay_recipe::state_uses_multiscale(s);
  let native_sampler=if multiscale{
   let sampler=checked.recipe.execution.as_ref().and_then(|e|e.multiscale_sampler).ok_or("Recorded multiscale sampler profile is unknown; historical native pointer width is not invented")?;
   Some(sampler)
  }else{None};
  let kind=match s.resolution.as_str(){"tract"=>rplan_core::UnitKind::Tract,"block-group"|"block_group"|"bg"=>rplan_core::UnitKind::BlockGroup,"block"=>rplan_core::UnitKind::Block,_=>return Err("Recorded resolution needs an explicit matching graph unit preparation.".into())};
  let selected_source=|role:&str,name:&std::path::Path|->Result<crate::native_auxiliary::Source,String>{
   let record=checked.recipe.sources.iter().find(|r|r.role==role).ok_or_else(||format!("Recipe does not record required native auxiliary role: {role}"))?;
   let bytes=record.sha256.as_ref().map(|h|checked.selected_by_hash.get(h).ok_or_else(||format!("Recorded native auxiliary bytes not selected: {role}"))).transpose()?;
   Ok(crate::native_auxiliary::Source{source_name:name.to_str().ok_or("Recorded native auxiliary name is not UTF-8")?.into(),bytes_base64:bytes.map(|b|STANDARD.encode(b))})
  };
  let w=&s.algorithm.weights;let route=bisect_runner::native_weighting::route(w,if s.districts==1{1}else{2});
  let composing=route==bisect_runner::native_weighting::Route::Compose;
  let implicit=|role:&str,relative:String,required:bool|->Result<Option<crate::native_auxiliary::Source>,String>{
   let Some(record)=checked.recipe.sources.iter().find(|r|r.role==role)else{if required{return Err(format!("Recipe does not record required native discovery role: {role}"));}return Ok(None)};
   let normalize=|p:&str|p.replace('\\',"/");let expected=format!("{}/{}",normalize(&checked.recipe.original_working_directory).trim_end_matches('/'),relative);
   if normalize(&record.path)!=expected{return Err(format!("Recorded native discovery path differs from original scope: {role}"));}
   selected_source(role,std::path::Path::new(&relative)).map(Some)
  };
  let economic=if w.economic_character{implicit("implicit-economic",bisect_core::native_input_paths::character_relative_path(&s.state_name,&s.year,"economic").unwrap(),composing)?}else{None};
  let housing=if w.housing_character{implicit("implicit-housing",bisect_core::native_input_paths::character_relative_path(&s.state_name,&s.year,"housing").unwrap(),composing)?}else{None};
  let needs_demo=w.minority_weighting||matches!(s.algorithm.split,bisect_runner::state_config::SplitStrategy::VraSection{..})||matches!(s.algorithm.seeds,bisect_runner::state_config::SeedCompositor::VraRecom{..});
  let demographics=if needs_demo{implicit("implicit-demographics",bisect_core::native_input_paths::demographic_relative_path(&s.state_name,&s.year),s.districts>1)?}else{None};
  // Preserve the existing single-character wire channel. Two active character
  // stages use separate selected sources, never a substituted single mode.
  let (character,economic,housing)=if w.economic_character&&w.housing_character{(None,economic,housing)}else if let Some(source)=economic{(Some(crate::native_auxiliary::Character{kind:"economic".into(),source}),None,None)}else if let Some(source)=housing{(Some(crate::native_auxiliary::Character{kind:"housing".into(),source}),None,None)}else{(None,None,None)};
  let native_auxiliary=crate::native_auxiliary::Input{partisan:w.partisan_shares.as_deref().map(|p|selected_source("explicit-partisan",p)).transpose()?,coi:s.coi_weights.as_deref().map(|p|selected_source("explicit-coi",p)).transpose()?,character,demographics,economic,housing};
  let mut auxiliary=serde_json::Map::new();auxiliary.insert("native_weighting".into(),serde_json::to_value(w).map_err(|e|e.to_string())?);
  if native_auxiliary.has_sources(){crate::native_auxiliary::validate(&native_auxiliary)?;auxiliary.insert("native_auxiliary".into(),serde_json::to_value(native_auxiliary).map_err(|e|e.to_string())?);}
  let auxiliary_json=Some(serde_json::to_string(&auxiliary).map_err(|e|e.to_string())?);
  let g=checked.graph;let mut geoids=Vec::with_capacity(g.n_vertices);
  for i in 0..g.n_vertices{geoids.push(g.index_to_geoid.get(&i).cloned().unwrap_or_else(||i.to_string()));}
  let fine=checked.fine_graph;
  let fine_ids=if let Some(f)=&fine{(0..f.n_vertices).map(|i|f.index_to_geoid.get(&i).cloned().ok_or("Fine graph needs complete GEOIDs")).collect::<Result<Vec<_>,_>>()?}else{geoids.clone()};
  let relative=manifest.rplan_path.as_deref().ok_or("Select the original RPLAN reference before recorded replay.")?;
  if !crate::package_verification::safe(relative){return Err("Unsafe selected replay plan reference.".into());}
  let root=manifest_path.rsplit_once('/').map_or("",|(p,_)|p);let path=if root.is_empty(){relative.to_owned()}else{format!("{root}/{relative}")};
  let reference=files.get(&path).ok_or("Original replay RPLAN reference is not selected.")?;
  let plan=rplan_io::read_rplan_str(std::str::from_utf8(reference).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?.plan;
  // Match the actual native runner's output policy, including its current
  // block/block_group -> Tract -> Imported fallback. The graph resolution is separate.
  let output_kind=if fine.is_some(){rplan_core::UnitKind::BlockGroup}else{match s.plan_resolution.as_str(){"bg"=>rplan_core::UnitKind::BlockGroup,"county"=>rplan_core::UnitKind::County,_=>rplan_core::UnitKind::Tract}};
  let mut expected_units=rplan_core::PlanUnitIndex{unit_kind:output_kind,state:Some(s.state_code.clone()),year:s.year.parse().ok(),canonical_order:rplan_core::CanonicalOrder::ExplicitUnitIds,unit_ids:fine_ids.clone(),unit_universe_hash:String::new(),source_id:Some(format!("bisect-adjacency-{}-{}",s.state_code,s.year))};
  if expected_units.validate().is_err(){expected_units.unit_kind=rplan_core::UnitKind::Imported;}
  expected_units.unit_universe_hash=expected_units.compute_unit_universe_hash().map_err(|e|e.to_string())?;
  expected_units.validate().map_err(|e|e.to_string())?;
  let native_imported_identity=fine.is_none()&&expected_units.unit_kind==rplan_core::UnitKind::Imported;
  if plan.k as u64!=s.districts||plan.units!=expected_units||plan.display_labels!=(1..=s.districts).map(|d|d.to_string()).collect::<Vec<_>>(){return Err("Original native RPLAN scope or literal district labels differ from the replay recipe.".into());}
  if let Some(fine_graph)=&fine {
   let relative=manifest.rctx_path.as_deref().ok_or("Select the original native fine RCTX before replay.")?;
   if !crate::package_verification::safe(relative){return Err("Unsafe selected fine context reference.".into());}
   let path=if root.is_empty(){relative.to_owned()}else{format!("{root}/{relative}")};
   let bytes=files.get(&path).ok_or("Original native fine RCTX is not selected.")?;
   original_fine_context=Some(checked_native_fine_context(bytes,&plan.units,fine_graph)?);
  } else {
   let relative=manifest.rctx_path.as_deref().ok_or("Select the original native ordinary RCTX before replay.")?;
   if !crate::package_verification::safe(relative){return Err("Unsafe selected ordinary context reference.".into());}
   let path=if root.is_empty(){relative.to_owned()}else{format!("{root}/{relative}")};
   let bytes=files.get(&path).ok_or("Original native ordinary RCTX is not selected.")?;
   original_ordinary_context=Some(checked_native_ordinary_context(bytes,&plan.units,&g)?);
  }
  let reference:BTreeMap<_,_>=plan.units.unit_ids.into_iter().zip(plan.assignment.into_iter().map(|d|u64::from(d)+1)).collect();
  if reference.len()!=fine_ids.len()||fine_ids.iter().any(|id|!reference.contains_key(id)){return Err("Original RPLAN and recorded graph have different complete GEOID sets.".into());}
  let mut edges=g.edge_weights.into_iter().map(|((a,b),w)|(a,b,w)).collect::<Vec<_>>();edges.sort_by_key(|e|(e.0,e.1));
  let native_geoid_index=if (g.index_to_geoid.len()!=g.n_vertices||(0..g.n_vertices).any(|i|!g.index_to_geoid.contains_key(&i))){Some(g.index_to_geoid.iter().map(|(&i,id)|(i.to_string(),id.clone())).collect())}else{None};
  let graph=crate::engine::PreparedGraph{native_source_identity:true,native_geoid_index,schema_version:1,state:s.state_code.clone(),year:s.year.clone(),unit_kind:Some(if native_imported_identity{rplan_core::UnitKind::Imported}else{kind}),county_ids:None,geoids,adjacency:g.adjacency,population:g.vertex_weights,edges,areas:checked.areas,exterior_perimeters:checked.perimeters,centroids:g.tract_centroids};
  let mut args=vec!["--state".into(),s.state_code.clone(),"--year".into(),s.year.clone(),"--districts".into(),s.districts.to_string(),"--chamber".into(),s.chamber.clone(),"--resolution".into(),if matches!(s.resolution.as_str(),"block-group"|"bg"){"block_group".into()}else{s.resolution.clone()},"--seats-per-district".into(),s.seats_per_district.to_string(),"--total-seats".into(),s.total_seats.to_string(),"--balance-tolerance".into(),(s.balance_tolerance_fraction*100.0).to_string(),"--smc-resample-threshold".into(),s.smc_resample_threshold.to_string()];
  args.extend(["--multiscale-fine".into(),s.multiscale_fine.clone(),"--multiscale-coarse".into(),s.multiscale_coarse.clone()]);
  if let Some(path)=&s.coi_weights{args.extend(["--coi-weights".into(),path.to_str().ok_or("Recorded COI name is not UTF-8")?.into()]);}
  let prepared=if multiscale{
   let mut request=json!({"graph":graph,"options":{"structure":"standard-bisect","search":"single","weights":"geographic","districts":1,"seed":null,"seeds":1,"steps":200,"percentile":0.0,"alpha_county":0.0,"balance_tolerance":0.5,"area_swing":1.10,"iterations":100}});
   for(key,value)in auxiliary{request[key]=value;}
   let block_groups=fine.as_ref().map(|f|crate::multiscale_input::BlockGroupGraph{schema_version:"bisect-block-group-graph-v1".into(),state:s.state_code.clone(),year:s.year.clone(),geoids:fine_ids.clone(),adjacency:f.adjacency.clone(),population:f.vertex_weights.clone()});
   let fine_edges=fine.as_ref().map(|f|{let mut edges=f.edge_weights.iter().map(|(&(a,b),&w)|(a,b,w)).collect::<Vec<_>>();edges.sort_by_key(|e|(e.0,e.1));edges});
   let template=json!({"operation":"run-multiscale-build","input":{"request":request,"fine_level":if block_groups.is_some(){"bg"}else{"tract"},"coarse_level":s.multiscale_coarse,"total_steps":0,"alpha":0.3,"percentile":0.0,"block_groups":block_groups,"adaptive":null,"native_sampler":native_sampler},"fine_edges":fine_edges});
   crate::native_state_run::prepare_recorded_multiscale(crate::native_state_run::Input{arguments:args,prepared:crate::prepared_engine::Input{request_json:serde_json::to_string(&template).map_err(|e|e.to_string())?,inspect_only:true,label:input.label.clone(),chamber:s.chamber.clone(),created_at:input.created_at.clone(),context_source:None}},&checked.recipe,&manifest.gpmetis_version)?
  }else{
   crate::native_state_run::prepare_recorded_graph(crate::native_state_run::GraphInput{arguments:args,graph_json:serde_json::to_string(&graph).map_err(|e|e.to_string())?,graph_source_name:"checked-recorded-native-graph.json".into(),auxiliary_source_name:auxiliary_json.as_ref().map(|_|"checked-native-auxiliary.json".into()),auxiliary_json,context_source:None,label:input.label.clone(),chamber:s.chamber.clone(),created_at:input.created_at.clone()},&checked.recipe,&manifest.gpmetis_version)?
  };
  Ok((prepared,reference))
 })();
 let (prepared,reference)=match prepared{Ok(v)=>v,Err(error)=>{output["status"]=json!("preparation-failed");output["error"]=json!(error);return Ok(output)}};
 output["prepared"]=prepared.clone();output["error"]=Value::Null;output["status"]=json!("prepared-replay-not-run");
 if !execute{return Ok(output)}
 let request_json=prepared["prepared_request_json"].as_str().ok_or("Missing prepared recorded request")?.to_owned();
 let chamber=manifest.chamber;
 match crate::prepared_engine::run(crate::prepared_engine::Input{context_source:None,inspect_only:false,request_json,label:input.label,chamber,created_at:input.created_at}){
  Err(error)=>{output["status"]=json!("replay-failed");output["error"]=json!(error);},
  Ok(mut replay)=>{
   if let Some(context_json)=&original_fine_context {
    if !replay["export"].is_null() && replay["export_error"].is_null(){
     match bind_native_fine_context(&replay["export"],context_json){Ok(exported)=>replay["export"]=exported,Err(error)=>replay["export_error"]=json!(error)}
    }
   }
   if let Some(context_json)=&original_ordinary_context {
    if !replay["export"].is_null() && replay["export_error"].is_null(){
     match bind_native_ordinary_context(&replay["export"],context_json){Ok(exported)=>replay["export"]=exported,Err(error)=>replay["export_error"]=json!(error)}
    }
   }
   let actual:BTreeMap<String,u64>=serde_json::from_value(replay["result"]["assignments"].clone()).map_err(|e|e.to_string())?;
   let mut matched=actual==reference;
   if let Some(expected)=&recorded_execution{
    let expected_json=serde_json::to_value(expected).map_err(|e|e.to_string())?;
    let report=&replay["result"]["build_report"];
    let attempts=report["attempts"].as_array().ok_or("Missing v3 execution retry report")?;
    let requested=json!(requested_seed.map(|v|v.to_string()));
    let requested_equal=attempts.first().is_some_and(|v|v["seed"]==requested);
    let accepted_equal=attempts.last().is_some_and(|v|v["seed"]==expected_json["accepted_seed"]);
    let attempt_equal=report["accepted_attempt"].as_u64()==Some(u64::from(expected.balance_retry_attempt));
    let sampler_equal=expected.multiscale_sampler.is_none()||replay["result"]["evidence"]["rng"]==expected_json["multiscale_sampler"];
    let equal=requested_equal&&accepted_equal&&attempt_equal&&sampler_equal;
    output["execution_comparison"]=json!({"bound_to_recipe":true,"requested_seed_equal":requested_equal,"accepted_seed_equal":accepted_equal,"retry_attempt_equal":attempt_equal,"sampler_equal":sampler_equal,"equal":equal,"recorded_execution":expected_json,"original_requested_seed":requested});
    matched=matched&&equal;
   }
   output["status"]=json!(if matched{"match"}else{"different"});output["comparison"]=json!({"literal_assignments_equal":matched,"reference_assignments":reference,"regenerated_assignments":actual,"district_relabelling_performed":false});output["execution_performed"]=json!(true);output["replay"]=replay;
  }
 }
 Ok(output)
}

#[cfg(test)]
mod native_fine_context_tests {
 use super::*;
 fn fixture()->(Value,Vec<u8>,rplan_core::PlanUnitIndex,bisect_runner::adjacency_loader::LoadedGraph){
  let fixture:Value=serde_json::from_str(include_str!("../../../scripts/web/fixtures/native-recorded-replay/core-native64-bg.json")).unwrap();
  let request:Value=serde_json::from_str(fixture["request_json"].as_str().unwrap()).unwrap();
  let context=STANDARD.decode(request["input"]["sources"]["prechecks"]["files_base64"]["plans/original/context.rctx"].as_str().unwrap()).unwrap();
  let units=rplan_io::read_rplan_str(fixture["native_reference"]["rplan_json"].as_str().unwrap()).unwrap().plan.units;
  let input:crate::native_recipe_sources::Input=serde_json::from_value(request["input"]["sources"].clone()).unwrap();
  let (_,checked)=crate::native_recipe_sources::inspect_checked(input).unwrap();
  (request,context,units,checked.unwrap().fine_graph.unwrap())
 }
 #[test]
 fn native_fine_context_preserves_original_bytes_and_full_index(){
  let (_,bytes,units,graph)=fixture();
  assert_eq!(checked_native_fine_context(&bytes,&units,&graph).unwrap().as_bytes(),bytes);
 }
 #[test]
 fn native_fine_context_rehashed_physical_mutations_are_refused(){
  let (_,bytes,units,graph)=fixture();let original=rplan_io::read_rctx_str(std::str::from_utf8(&bytes).unwrap()).unwrap();
  for mutation in 0..7 {
   let mut context=original.clone();
   match mutation {
    0=>context.populations.as_mut().unwrap()[0]+=1,
    1=>context.graph.as_mut().unwrap().adjacency[0][0].weight=Some(99.),
    2=>context.graph.as_mut().unwrap().adjacency[0][0].kind=rplan_core::EdgeKind::Custom,
    3=>context.subdivisions.as_mut().unwrap().county_ids.as_mut().unwrap()[0]=Some("01999".into()),
    4=>context.graph=None,
    5=>{context.units.source_id=Some("changed-native-identity".into());context.units.unit_universe_hash=context.units.compute_unit_universe_hash().unwrap();},
    _=>{context.units.unit_ids.swap(0,1);context.units.unit_universe_hash=context.units.compute_unit_universe_hash().unwrap();},
   }
   context.context_hash=context.compute_context_hash().unwrap();
   let text=rplan_io::write_rctx_string(&context).unwrap();
   assert!(rplan_io::read_rctx_str(&text).is_ok(),"mutation must reach semantic guard: {mutation}");
   assert!(checked_native_fine_context(text.as_bytes(),&units,&graph).is_err(),"mutation accepted: {mutation}");
  }
 }
 #[test]
 fn native_fine_context_replay_export_matches_original_context_and_plan(){
  let (request,bytes,units,_)=fixture();let input:Input=serde_json::from_value(request["input"].clone()).unwrap();
  let reply=replay(input,true).unwrap();assert_eq!(reply["status"],"match");assert!(reply["replay"]["export_error"].is_null());
  let exported=&reply["replay"]["export"];
  assert_eq!(exported["context_json"].as_str().unwrap().as_bytes(),bytes);
  assert_eq!(exported["document"]["plan"]["units"],serde_json::to_value(units).unwrap());
  assert!(exported["document"]["provenance"]["conversion_lineage"][0]["original_edge_classification_retained"].as_bool().unwrap());
 }
}


#[cfg(test)]
mod native_ordinary_context_tests {
 use super::*;
 fn fixture()->Value{serde_json::from_str(include_str!("../../../scripts/web/fixtures/native-ordinary-recorded-replay.json")).unwrap()}
 fn selected()->(String,rplan_core::PlanUnitIndex,bisect_runner::adjacency_loader::LoadedGraph){
  let f=fixture();let input:Input=serde_json::from_value(f["request"]["input"].clone()).unwrap();
  let files=crate::package_verification::selected_bytes(input.sources.prechecks.files_base64.clone()).unwrap();
  let checked=crate::native_recipe_sources::inspect_checked(input.sources).unwrap().1.unwrap();
  let context=String::from_utf8(files["plans/original/context.rctx"].clone()).unwrap();
  let plan=rplan_io::read_rplan_str(std::str::from_utf8(&files["plans/original/plan.rplan"]).unwrap()).unwrap().plan;
  (context,plan.units,checked.graph)
 }
 #[test]
 fn native_ordinary_context_preserves_selected_bytes_and_index(){
  let(text,units,graph)=selected();assert_eq!(checked_native_ordinary_context(text.as_bytes(),&units,&graph).unwrap(),text);
 }
 #[test]
 fn native_ordinary_context_rehashed_semantic_mutations_refuse(){
  let(text,units,graph)=selected();let context=rplan_io::read_rctx_str(&text).unwrap();
  for case in 0..10{
   let mut c=context.clone();
   match case{
    0=>c.populations.as_mut().unwrap()[0]+=1,
    1=>{for (from,edges) in c.graph.as_mut().unwrap().adjacency.iter_mut().enumerate(){for edge in edges{if from==0&&edge.to==1||from==1&&edge.to==0{edge.weight=Some(edge.weight.unwrap()+1.);}}}},
    2=>{for (from,edges) in c.graph.as_mut().unwrap().adjacency.iter_mut().enumerate(){for edge in edges{if from==0&&edge.to==1||from==1&&edge.to==0{edge.kind=rplan_core::EdgeKind::Bridge;}}}},
    3=>c.graph.as_mut().unwrap().adjacency[1].reverse(),
    4=>c.subdivisions.as_mut().unwrap().county_ids.as_mut().unwrap()[0]=Some("99999".into()),
    5=>{c.units.source_id=Some("other-selected-identity".into());},
    6=>{c.units.unit_ids.reverse();},
    7=>c.graph=None,
    8=>c.populations=None,
    9=>c.subdivisions=None,
    _=>unreachable!(),
   }
   c.units.unit_universe_hash=c.units.compute_unit_universe_hash().unwrap();c.context_hash=c.compute_context_hash().unwrap();
   let changed=serde_json::to_string(&c).unwrap();assert!(rplan_io::read_rctx_str(&changed).is_ok(),"mutation {case} must remain a valid independently rehashed context");
   assert!(checked_native_ordinary_context(changed.as_bytes(),&units,&graph).is_err(),"mutation {case}");
  }
 }
 #[test]
 fn native_ordinary_context_export_matches_original_context_and_plan_body(){
  let f=fixture();let input:Input=serde_json::from_value(f["request"]["input"].clone()).unwrap();let output=replay(input,true).unwrap();
  assert_eq!(output["status"],"match");assert!(output["replay"]["export_error"].is_null());
  let(text,_,_)=selected();assert_eq!(output["replay"]["export"]["context_json"],text);
  let original=rplan_io::read_rplan_str(f["reference"]["document_json"].as_str().unwrap()).unwrap();
  let actual=rplan_io::read_rplan_str(output["replay"]["export"]["document_json"].as_str().unwrap()).unwrap();
  assert_eq!(actual.plan,original.plan);assert_eq!(actual.metadata.label,f["request"]["input"]["label"].as_str().unwrap());
  assert!(actual.provenance.producer.contains_key("native_original_context_binding"));assert!(!actual.provenance.producer.contains_key("native_fine_context_binding"));
 }
}

#[cfg(test)]
mod native_imported_context_tests {
 use super::*;
 fn fixtures()->Vec<Value>{serde_json::from_str(include_str!("../../../scripts/web/fixtures/native-imported-recorded-replay.json")).unwrap()}
 #[test]
 fn native_imported_context_replays_all_optional_county_shapes(){
  for case in fixtures(){
   let output=crate::toolkit::execute(case["request"].clone()).unwrap();
   assert_eq!(output["status"],"match","{}: {output}",case["name"]);
   assert!(output["replay"]["export_error"].is_null(),"{output}");
   assert_eq!(output["replay"]["export"]["context_json"],case["context_json"]);
   let original=rplan_io::read_rplan_str(case["document_json"].as_str().unwrap()).unwrap();
   let actual=rplan_io::read_rplan_str(output["replay"]["export"]["document_json"].as_str().unwrap()).unwrap();
   assert_eq!(actual.plan,original.plan);
   assert_eq!(output["replay"]["executed_request"]["graph"]["geoids"],serde_json::to_value(&original.plan.units.unit_ids).unwrap());
  }
 }
 #[test]
 fn native_imported_context_preserves_missing_counties_without_invention(){
  for case in fixtures(){
   let output=crate::toolkit::execute(case["request"].clone()).unwrap();
   assert_eq!(output["status"],"match","{output}");
   let context:rplan_core::RplanContext=rplan_io::read_rctx_str(case["context_json"].as_str().unwrap()).unwrap();
   let exported:&Value=&output["replay"]["export"]["context"];
   assert_eq!(exported["subdivisions"],serde_json::to_value(context.subdivisions).unwrap());
  }
 }
 #[test]
 fn native_imported_context_policy_guards_are_explicit(){
  let case=fixtures().remove(0);
  let output=crate::toolkit::execute(case["request"].clone()).unwrap();
  assert_eq!(output["status"],"match","{output}");
  let original=output["replay"]["executed_request"].clone();
  for mutation in 0..5 {
   let mut value=original.clone();
   match mutation {
    0=>value["graph"]["county_ids"]=json!(["44001","44001","44003","44003"]),
    1=>value["graph"]["unit_kind"]=json!("tract"),
    2=>value["graph"]["geoids"][0]=value["graph"]["geoids"][1].clone(),
    3=>value["graph"]["geoids"][0]=json!("__proto__"),
    _=>{value["native_weighting"]=Value::Null;},
   }
   let request:crate::engine::Request=serde_json::from_value(value).unwrap();
   assert!(crate::native_composed::validate_request(&request).and_then(|_|crate::engine::validate(&request.graph,&request.options)).is_err(),"mutation {mutation}");
  }
 }

 #[test]
 fn native_imported_context_source_index_does_not_turn_ordinal_ids_into_coi(){
  for (name,expected_changes) in [("absent-index-coi",0),("sparse-index-coi",1)]{
   let case=fixtures().into_iter().find(|case|case["name"]==name).unwrap();
   let output=crate::toolkit::execute(case["request"].clone()).unwrap();assert_eq!(output["status"],"match","{output}");
   let request:crate::engine::Request=serde_json::from_value(output["replay"]["executed_request"].clone()).unwrap();
   assert!(request.graph.native_geoid_index.is_some());
   assert_eq!(output["replay"]["result"]["metrics"]["native_coi_source_evidence"]["changed_edges"],expected_changes);
   let original_ids:std::collections::HashMap<usize,String>=if case["source_index"].is_null(){std::collections::HashMap::new()}else{serde_json::from_value(case["source_index"].clone()).unwrap()};
   let weights=bisect_runner::coi_weights::parse(&case["coi"].to_string()).unwrap();
   let base=request.graph.edges.iter().map(|&(u,v,w)|((u,v),w)).collect();
   let reference=bisect_runner::coi_weights::apply(base,&weights,&original_ids).unwrap();
   let (actual,_)=crate::native_auxiliary::coi(request.native_auxiliary.as_ref(),&request.graph,request.graph.edges.iter().map(|&(u,v,w)|((u,v),w)).collect()).unwrap();
   assert_eq!(actual,reference);
   if name=="absent-index-coi"{
    assert!(request.graph.native_source_index().is_empty());
    // Same plan labels with a declared complete source index are different data.
    let mut false_source=request.graph.clone();false_source.native_geoid_index=None;
    let (changed,_)=crate::native_auxiliary::coi(request.native_auxiliary.as_ref(),&false_source,request.graph.edges.iter().map(|&(u,v,w)|((u,v),w)).collect()).unwrap();
    assert_ne!(actual,changed);assert_eq!(actual,request.graph.edges.iter().map(|&(u,v,w)|((u,v),w)).collect::<std::collections::HashMap<_,_>>());assert!(changed.values().all(|w|*w==0.0));
   }
  }
 }
 #[test]
 fn native_imported_context_source_index_guards_and_large_ordinal_counties(){
  let case=fixtures().into_iter().find(|case|case["name"]=="absent-index").unwrap();
  let output=crate::toolkit::execute(case["request"].clone()).unwrap();assert_eq!(output["status"],"match","{output}");
  let original=output["replay"]["executed_request"].clone();
  for mutation in 0..4 {
   let mut value=original.clone();
   match mutation {
    0=>value["graph"]["native_geoid_index"]=json!({"18446744073709551615":"outside"}),
    1=>value["graph"]["native_geoid_index"]=json!({"0":"changed-source-id"}),
    2=>value["graph"]["native_source_identity"]=json!(false),
    _=>value["graph"]["county_ids"]=json!(["made-up","made-up","made-up","made-up"]),
   }
   let request:crate::engine::Request=serde_json::from_value(value).unwrap();
   assert!(crate::engine::validate(&request.graph,&request.options).is_err(),"source index mutation {mutation}");
  }
  // Pure metadata convention check, not a generation/capacity claim.
  let mut graph:crate::engine::PreparedGraph=serde_json::from_value(original["graph"].clone()).unwrap();
  graph.geoids=(0..=100000).map(|i|i.to_string()).collect();
  assert_eq!(graph.report_county_id(100000),None);
 }

}
