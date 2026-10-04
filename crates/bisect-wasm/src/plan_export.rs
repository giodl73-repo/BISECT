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
        if u>=v || v>=ids.len() || !length.is_finite() || length<=0. || !adjacency[u].contains(&v) || weights.insert((u,v),length).is_some() {return Err("Invalid fine physical boundary edge.".into());}
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
    context.validate().map_err(|e|e.to_string())?;
    context.context_hash=context.compute_context_hash().map_err(|e|e.to_string())?;
    let document=RplanDocument {rplan_version:RPLAN_V02.into(),plan,metadata:RplanMetadataV02 {label,jurisdiction:g.state.clone(),chamber,created_at,description:Some("Supplied multiscale fine assignments exported for independent native audit; generation and optimality are not certified.".into())},provenance:RplanProvenance {producer:BTreeMap::from([("multiscale_input".into(),input_value)]),source_hashes:context.source_hashes.entries.clone(),conversion_lineage:vec![json!({"operation":"multiscale-fine-graph-export","edge_kind":"custom","reason":"Input adjacency and physical boundary weights retained; original edge classification and map geometry are not exported."})]},geometry:None,extensions:BTreeMap::new()};
    read_rplan_str(&write_rplan_string(&document).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    read_rctx_str(&write_rctx_string(&context).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-multiscale-plan-export-v1","plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?,"context_hash":context.context_hash,"multiscale_input_hash":input_hash,"document":document,"context":context,"source_boundary":"Unsigned supplied fine assignments; export does not execute generation or assert balance. Fine unit order, populations, adjacency and supplied physical weights are retained for independent audit. Edge classification and map geometry are unavailable in this export; retain the laboratory project for its map."}))
}

pub(crate) fn export(request: Request, assignments: BTreeMap<String, u32>, label: String, chamber: String, created_at: String) -> Result<Value, String> {
    let g = &request.graph;
    validate(g, &request.options)?;
    let character_hash=match (["economic-character","housing-character"].contains(&request.options.weights.as_str()),request.character.as_ref()) {
        (true,Some(input))=>{
            if input.state!=g.state||input.year!=g.year {return Err("Character export scope must match the graph.".into());}
            let weights=crate::character_input::build_weights(input,&g.geoids,&g.edges,request.options.character_alpha.unwrap())?;
            if format!("{}-character",weights.kind)!=request.options.weights {return Err("Character export kind must match the weights.".into());}
            Some(weights.character_hash)
        },
        (true,None)=>return Err("Character export requires complete supplied observations.".into()),
        (false,Some(_))=>return Err("Character input has no effect on these export options.".into()),
        _=>None,
    };
    let partisan_hash=match (request.options.weights=="partisan",request.partisan.as_ref()) {
        (true,Some(input))=>Some(crate::engine::partisan_shares(g,input)?.1),
        (true,None)=>return Err("Partisan export requires complete supplied shares.".into()),
        (false,Some(_))=>return Err("Partisan shares have no effect on these export options.".into()),
        _=>None,
    };
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
    let mut units=PlanUnitIndex {unit_kind:UnitKind::Tract,state:Some(g.state.clone()),year:Some(year),canonical_order:CanonicalOrder::ExplicitUnitIds,unit_ids:g.geoids.clone(),unit_universe_hash:String::new(),source_id:Some("bisect-prepared-graph-v1".into())};
    units.unit_universe_hash=units.compute_unit_universe_hash().map_err(|e|e.to_string())?;
    let plan=DistrictPlan {schema_version:DISTRICT_PLAN_SCHEMA_VERSION.into(),units:units.clone(),assignment,k:request.options.districts,display_labels:(1..=request.options.districts).map(|d|d.to_string()).collect(),allow_empty_districts:false};
    plan.validate().map_err(|e|e.to_string())?;
    let weights:BTreeMap<_,_>=g.edges.iter().map(|&(u,v,w)|((u,v),w)).collect();
    let mut context=RplanContext {rctx_version:RCTX_VERSION.into(),context_hash:String::new(),units,graph:Some(UnitGraph {edge_semantics:EdgeSemantics::Undirected,adjacency:g.adjacency.iter().enumerate().map(|(u,neighbors)|neighbors.iter().map(|&v|UnitEdge {to:v as u32,kind:EdgeKind::Custom,weight:Some(weights[&(u.min(v),u.max(v))])}).collect()).collect()}),populations:Some(g.population.clone()),subdivisions:Some(SubdivisionContext {county_ids:Some(g.geoids.iter().map(|id|Some(id[..5].into())).collect()),municipal_ids:None}),demographics:None,geometry:None,source_hashes:SourceHashes {entries:BTreeMap::from([("bisect.prepared-graph".into(),graph_hash.clone())])}};
    if let Some(digest)=&partisan_hash {context.source_hashes.entries.insert("bisect.partisan-shares".into(),digest.clone());}
    if let Some(digest)=&election_hash {context.source_hashes.entries.insert("bisect.election-counts".into(),digest.clone());}
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
    if let Some(input)=&request.character {producer.insert("character_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    if let Some(input)=&request.partisan {producer.insert("partisan_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    if let Some(input)=&request.elections {producer.insert("election_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    let document=RplanDocument {rplan_version:RPLAN_V02.into(),plan,metadata:RplanMetadataV02 {label,jurisdiction:g.state.clone(),chamber,created_at,description:Some("Prepared graph export; engine district labels converted from one-based to native zero-based IDs. Generation and optimality are not certified.".into())},provenance:RplanProvenance {producer,source_hashes:context.source_hashes.entries.clone(),conversion_lineage:vec![json!({"operation":"prepared-graph-export","edge_kind":"custom","reason":"Prepared graph does not retain bridge or physical edge classification."})]},geometry:None,extensions:BTreeMap::new()};
    // Exercise the same readers used by native file consumers before emitting anything.
    read_rplan_str(&write_rplan_string(&document).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    read_rctx_str(&write_rctx_string(&context).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-engine-plan-export-v1","demographic_basis":request.demographics.as_ref().map(|input|&input.basis),"plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?,"context_hash":context.context_hash,"prepared_graph_hash":graph_hash,"document":document,"context":context,"source_boundary":"Unsigned supplied assignments; export validates shape and hashes but does not replay generation. Custom graph edges preserve prepared adjacency and physical weights; original edge types and geometry are unavailable. Explicit VAP/CVAP counts are retained in native demographic fields; fractions and total-population proxies do not supply VAP counts."}))
}
