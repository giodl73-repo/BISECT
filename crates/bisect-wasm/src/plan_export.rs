//! Lossless prepared-graph/assignment bridge into the native practitioner formats.
use crate::engine::{validate, Request};
use rplan_core::*;
use rplan_io::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Export supplied fine assignments without executing generation. Native unit
/// identity and context hashes bind the exact ordered graph used for auditing.
pub(crate) fn export_multiscale(input: crate::multiscale_input::MultiscaleRequest, fine_edges: Option<Vec<(usize,usize,f64)>>, assignments: BTreeMap<String,u32>, label:String, chamber:String, created_at:String) -> Result<Value,String> {
    crate::multiscale_input::validate(&input)?;
    for field in [&label,&chamber,&created_at] {
        if field.trim().is_empty() || field.len()>200 {return Err("Export metadata must be nonempty and at most 200 bytes.".into());}
    }
    let g=&input.request.graph;
    let zone=crate::zone_input::for_engine(g,&input.request.options,input.request.zone_membership.as_ref())?;
    let o=&input.request.options;
    let bg=input.block_groups.as_ref();
    let ids=bg.map_or(&g.geoids,|b|&b.geoids);
    let populations=bg.map_or(&g.population,|b|&b.population);
    let adjacency=bg.map_or(&g.adjacency,|b|&b.adjacency);
    let edges=match (bg,&fine_edges) {
        (Some(_),Some(edges))=>edges,
        (Some(_),None)=>return Err("Fine physical boundary edges are required for block-group export.".into()),
        (None,Some(_))=>return Err("Tract export uses the prepared graph boundaries; fine edges are unused.".into()),
        (None,None)=>&g.edges,
    };
    let mut weights=BTreeMap::new(); let mut total=0.;
    for &(u,v,length) in edges {
        if u>=v || v>=ids.len() || !length.is_finite() || length<0. || !adjacency[u].contains(&v) || weights.insert((u,v),length).is_some() {return Err("Invalid fine physical boundary edge.".into());}
        total+=length;
    }
    if !total.is_finite() || adjacency.iter().map(Vec::len).sum::<usize>()!=weights.len()*2 {return Err("Fine physical boundaries must cover adjacency exactly.".into());}
    if assignments.len()!=ids.len() {return Err("Assignments must cover exactly the fine graph.".into());}
    let assignment=ids.iter().map(|id| {
        let d=*assignments.get(id).ok_or("Missing fine graph assignment.")?;
        if d==0 || d as usize>o.districts {return Err("Engine district labels must be in 1..=k.");}
        Ok(d-1)
    }).collect::<Result<Vec<_>,_>>()?;
    let mut units=PlanUnitIndex {unit_kind:if bg.is_some(){UnitKind::BlockGroup}else{UnitKind::Tract},state:Some(g.state.clone()),year:Some(g.year.parse().map_err(|_|"Invalid graph year.")?),canonical_order:CanonicalOrder::ExplicitUnitIds,unit_ids:ids.clone(),unit_universe_hash:String::new(),source_id:Some("bisect-multiscale-fine-graph-v1".into())};
    units.unit_universe_hash=units.compute_unit_universe_hash().map_err(|e|e.to_string())?;
    let plan=DistrictPlan {schema_version:DISTRICT_PLAN_SCHEMA_VERSION.into(),units:units.clone(),assignment,k:o.districts,display_labels:(1..=o.districts).map(|d|d.to_string()).collect(),allow_empty_districts:false};
    plan.validate().map_err(|e|e.to_string())?;
    let input_value=serde_json::to_value(&input).map_err(|e|e.to_string())?;
    let input_hash=canonical_sha256(&input_value).map_err(|e|e.to_string())?;
    let boundary_hash=canonical_sha256(&serde_json::to_value(edges).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let mut context=RplanContext {rctx_version:RCTX_VERSION.into(),context_hash:String::new(),units,graph:Some(UnitGraph {edge_semantics:EdgeSemantics::Undirected,adjacency:adjacency.iter().enumerate().map(|(u,ns)|ns.iter().map(|&v|UnitEdge {to:v as u32,kind:EdgeKind::Custom,weight:Some(weights[&(u.min(v),u.max(v))])}).collect()).collect()}),populations:Some(populations.clone()),subdivisions:Some(SubdivisionContext {county_ids:Some(ids.iter().map(|id|Some(id[..5].into())).collect()),municipal_ids:None}),demographics:None,geometry:None,source_hashes:SourceHashes {entries:BTreeMap::from([("bisect.multiscale-input".into(),input_hash.clone()),("bisect.fine-boundaries".into(),boundary_hash)])}};
    if let Some(coi) = &input.request.coi {
        context.source_hashes.entries.insert("bisect.coi-weights-json".into(), crate::coi_input::hash(coi));
    }
    if let Some(z)=&zone {context.source_hashes.entries.insert("bisect.zone-membership-json".into(),z.sha256.clone());}
    if let Some(settings)=&input.request.electoral {settings.resolve(o.districts)?;context.source_hashes.entries.insert("bisect.electoral-settings".into(),settings.hash()?);}
    if let Some(computed)=crate::native_character_weights::for_engine(g,o,input.request.native_character_source.as_ref(),input.request.character.is_some())?{context.source_hashes.entries.insert("bisect.native-character-source".into(),computed.report["source_sha256"].as_str().unwrap().into());context.source_hashes.entries.insert("bisect.native-character-target".into(),computed.report["target_sha256"].as_str().unwrap().into());}
    context.validate().map_err(|e|e.to_string())?;
    context.context_hash=context.compute_context_hash().map_err(|e|e.to_string())?;
    let document=RplanDocument {rplan_version:RPLAN_V02.into(),plan,metadata:RplanMetadataV02 {label,jurisdiction:g.state.clone(),chamber,created_at,description:Some("Supplied multiscale fine assignments exported for independent native audit; generation and optimality are not certified.".into())},provenance:RplanProvenance {producer:BTreeMap::from([("multiscale_input".into(),input_value)]),source_hashes:context.source_hashes.entries.clone(),conversion_lineage:vec![json!({"operation":"multiscale-fine-graph-export","edge_kind":"custom","reason":"Input adjacency and physical boundary weights retained; original edge classification and map geometry are not exported."})]},geometry:None,extensions:BTreeMap::new()};
    read_rplan_str(&write_rplan_string(&document).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let context_json=crate::plan_audit_text::context_json_for_export(&context)?;
    read_rctx_str(&context_json).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-multiscale-plan-export-v1","plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?,"context_hash":context.context_hash,"multiscale_input_hash":input_hash,"document_json":write_rplan_string(&document).map_err(|e|e.to_string())?,"context_json":context_json,"document":document,"context":context,"source_boundary":"Unsigned supplied fine assignments; export does not execute generation or assert balance. Fine unit order, populations, adjacency and supplied physical weights are retained for independent audit. Edge classification and map geometry are unavailable in this export; retain the laboratory project for its map."}))
}

pub(crate) fn export(request: Request, assignments: BTreeMap<String, u32>, label: String, chamber: String, created_at: String) -> Result<Value, String> {
    if request.options.vra_absent_file_fallback==Some(true)&&request.demographics.is_some(){return Err("Absent-file fallback export cannot contain demographic input.".into());}
    crate::native_composed::validate_request(&request)?;
    let g = &request.graph;
    validate(g, &request.options)?;
    let zone=crate::zone_input::for_engine(g,&request.options,request.zone_membership.as_ref())?;
    let recorded_character=if request.native_weighting.is_some(){crate::native_auxiliary::CharacterResult{computed:None,evidence:None}}else{crate::native_auxiliary::character(request.native_auxiliary.as_ref(),g,&request.options,request.native_character_source.is_some()||request.character.is_some())?};
    let has_recorded_character=request.native_auxiliary.as_ref().is_some_and(|i|i.character.is_some());
    let native_character=if has_recorded_character{recorded_character.computed}else{crate::native_character_weights::for_engine(g,&request.options,request.native_character_source.as_ref(),request.character.is_some())?};
    let character_hash=if native_character.is_some(){None}else{match (["economic-character","housing-character"].contains(&request.options.weights.as_str()),request.character.as_ref()) {
        (true,Some(input))=>{
            if input.state!=g.state||input.year!=g.year {return Err("Character export scope must match the graph.".into());}
            let weights=crate::character_input::build_weights(input,&g.geoids,&g.edges,request.options.character_alpha.unwrap())?;
            if format!("{}-character",weights.kind)!=request.options.weights {return Err("Character export kind must match the weights.".into());}
            Some(weights.character_hash)
        },
        (true,None) if request.options.character_absent_file_fallback==Some(true)=>None,
        (true,None)=>return Err("Character export requires complete supplied observations.".into()),
        (false,Some(_))=>return Err("Character input has no effect on these export options.".into()),
        _=>None,
    }};
    let native_partisan=if request.native_weighting.is_some(){crate::native_auxiliary::Partisan{data:None,evidence:None}}else{crate::native_auxiliary::partisan(request.native_auxiliary.as_ref(),g,request.options.weights=="partisan",request.options.districts)?};
    let skip_native_partisan=request.options.districts==1&&request.native_auxiliary.as_ref().is_some_and(|i|i.partisan.is_some());
    crate::native_auxiliary::validate_selection(request.native_auxiliary.as_ref(),request.partisan.is_some(),request.coi.is_some())?;
    let partisan_hash=if native_partisan.data.is_some()||skip_native_partisan{native_partisan.data.map(|(_,hash)|hash)}else{match (request.options.weights=="partisan",request.partisan.as_ref()) {
        (true,Some(input))=>Some(crate::engine::partisan_shares(g,input)?.1),
        (true,None)=>return Err("Partisan export requires complete supplied shares.".into()),
        (false,Some(_))=>return Err("Partisan shares have no effect on these export options.".into()),
        _=>None,
    }};
    let election_hash=match (["proportional-bisect","proportional-section"].contains(&request.options.structure.as_str()),request.elections.as_ref()) {
        (true,Some(input))=>Some(crate::election_input::validate(g,&request.options,input)?.sha256),
        (true,None)=>return Err("Proportional export requires complete election counts.".into()),
        (false,Some(_))=>return Err("Election counts have no effect on these export options.".into()),
        _=>None,
    };
    for field in [&label, &chamber, &created_at] {
        if field.trim().is_empty() || field.len() > 200 { return Err("Export metadata must be nonempty and at most 200 bytes.".into()); }
    }
    if g.state.len()!=2 || !g.state.bytes().all(|c| c.is_ascii_uppercase()) || g.year.len()!=4 { return Err("Explicit state and four-digit year required.".into()); }
    let year = g.year.parse::<u16>().map_err(|_| "Invalid graph year.")?;
    if assignments.len()!=g.geoids.len() { return Err("Assignments must cover exactly the prepared graph.".into()); }
    let assignment = g.geoids.iter().map(|id| {
        let district = *assignments.get(id).ok_or("Missing graph assignment.")?;
        if district==0 || district as usize>request.options.districts {return Err("Engine district labels must be in 1..=k.");}
        Ok(district-1)
    }).collect::<Result<Vec<_>,_>>()?;
    let graph_hash = canonical_sha256(&serde_json::to_value(g).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let mut units=PlanUnitIndex {unit_kind:g.effective_unit_kind(),state:Some(g.state.clone()),year:Some(year),canonical_order:CanonicalOrder::ExplicitUnitIds,unit_ids:g.geoids.clone(),unit_universe_hash:String::new(),source_id:Some("bisect-prepared-graph-v1".into())};
    units.unit_universe_hash=units.compute_unit_universe_hash().map_err(|e|e.to_string())?;
    let plan=DistrictPlan {schema_version:DISTRICT_PLAN_SCHEMA_VERSION.into(),units:units.clone(),assignment,k:request.options.districts,display_labels:(1..=request.options.districts).map(|d|d.to_string()).collect(),allow_empty_districts:false};
    plan.validate().map_err(|e|e.to_string())?;
    let weights:BTreeMap<_,_>=g.edges.iter().map(|&(u,v,w)|((u,v),w)).collect();
    let county_ids=(0..g.geoids.len()).map(|i|g.report_county_id(i).map(str::to_owned)).collect::<Vec<_>>();
    let subdivisions=county_ids.iter().any(Option::is_some).then_some(SubdivisionContext{county_ids:Some(county_ids),municipal_ids:None});
    let mut context=RplanContext {rctx_version:RCTX_VERSION.into(),context_hash:String::new(),units,graph:Some(UnitGraph {edge_semantics:EdgeSemantics::Undirected,adjacency:g.adjacency.iter().enumerate().map(|(u,neighbors)|neighbors.iter().map(|&v|UnitEdge {to:v as u32,kind:EdgeKind::Custom,weight:Some(weights[&(u.min(v),u.max(v))])}).collect()).collect()}),populations:Some(g.population.clone()),subdivisions,demographics:None,geometry:None,source_hashes:SourceHashes {entries:BTreeMap::from([("bisect.prepared-graph".into(),graph_hash.clone())])}};
    if let Some(z)=&zone {context.source_hashes.entries.insert("bisect.zone-membership-json".into(),z.sha256.clone());}
    if let Some(settings)=&request.electoral {settings.resolve(request.options.districts)?;context.source_hashes.entries.insert("bisect.electoral-settings".into(),settings.hash()?);}
    if let Some(coi)=&request.coi{crate::coi_input::validate(coi)?;context.source_hashes.entries.insert("bisect.coi-weights-json".into(),crate::coi_input::hash(coi));}
    if let Some(aux)=&request.native_auxiliary{context.source_hashes.entries.extend(crate::native_auxiliary::source_hashes(aux)?);}
    if let Some(spec)=&request.native_weighting{context.source_hashes.entries.insert("bisect.native-weighting-spec".into(),canonical_sha256(&serde_json::to_value(spec).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?);}
    if let Some(digest)=&partisan_hash {context.source_hashes.entries.insert("bisect.partisan-shares".into(),digest.clone());}
    if let Some(digest)=&election_hash {context.source_hashes.entries.insert("bisect.election-counts".into(),digest.clone());}
    if let Some(computed)=&native_character{context.source_hashes.entries.insert("bisect.native-character-source".into(),computed.report["source_sha256"].as_str().unwrap().into());context.source_hashes.entries.insert("bisect.native-character-target".into(),computed.report["target_sha256"].as_str().unwrap().into());}
    if let Some(digest)=&character_hash {context.source_hashes.entries.insert("bisect.character-input".into(),digest.clone());}
    context.validate().map_err(|e|e.to_string())?;
    if let Some(input)=&request.demographics {
        crate::engine::demographic_fractions(g,input)?;
        context.source_hashes.entries.insert("bisect.demographic-input".into(),canonical_sha256(&serde_json::to_value(input).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?);
        if input.basis!="total-population" {
            if let Some(counts)=&input.counts {
                context.demographics=Some(DemographicContext {total_vap:Some(g.geoids.iter().map(|id|counts[id].total).collect()),minority_vap:Some(g.geoids.iter().map(|id|counts[id].minority).collect())});
            }
        }
        context.validate().map_err(|e|e.to_string())?;
    }
    context.context_hash=context.compute_context_hash().map_err(|e|e.to_string())?;
    let mut producer=BTreeMap::from([("engine_options".into(),serde_json::to_value(&request.options).map_err(|e|e.to_string())?)]);
    if let Some(settings)=&request.electoral {producer.insert("electoral_settings".into(),serde_json::to_value(settings).map_err(|e|e.to_string())?);}
    if let Some(input)=&request.zone_membership {producer.insert("zone_membership_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    if let Some(input)=&request.coi {producer.insert("coi_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    if let Some(input)=&request.native_character_source {producer.insert("native_character_source".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    if let Some(input)=&request.character {producer.insert("character_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    if let Some(input)=&request.partisan {producer.insert("partisan_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    if let Some(input)=&request.native_auxiliary {producer.insert("native_auxiliary_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    if let Some(spec)=&request.native_weighting{producer.insert("native_weighting_spec".into(),serde_json::to_value(spec).map_err(|e|e.to_string())?);}
    if let Some(input)=&request.elections {producer.insert("election_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    let document=RplanDocument {rplan_version:RPLAN_V02.into(),plan,metadata:RplanMetadataV02 {label,jurisdiction:g.state.clone(),chamber,created_at,description:Some("Prepared graph export; engine district labels converted from one-based to native zero-based IDs. Generation and optimality are not certified.".into())},provenance:RplanProvenance {producer,source_hashes:context.source_hashes.entries.clone(),conversion_lineage:vec![json!({"operation":"prepared-graph-export","edge_kind":"custom","reason":"Prepared graph does not retain bridge or physical edge classification."})]},geometry:None,extensions:BTreeMap::new()};
    // Exercise the same readers used by native file consumers before emitting anything.
    read_rplan_str(&write_rplan_string(&document).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let context_json=crate::plan_audit_text::context_json_for_export(&context)?;
    read_rctx_str(&context_json).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-engine-plan-export-v1","document_json":write_rplan_string(&document).map_err(|e|e.to_string())?,"context_json":context_json,"demographic_basis":request.demographics.as_ref().map(|input|&input.basis),"plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?,"context_hash":context.context_hash,"prepared_graph_hash":graph_hash,"document":document,"context":context,"source_boundary":"Unsigned supplied assignments; export validates shape and hashes but does not replay generation. Custom graph edges preserve prepared adjacency and physical weights; original edge types and geometry are unavailable. Explicit VAP/CVAP counts are retained in native demographic fields; fractions and total-population proxies do not supply VAP counts."}))
}
