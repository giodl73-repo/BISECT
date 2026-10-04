//! Lossless prepared-graph/assignment bridge into the native practitioner formats.
use crate::engine::{validate, Request};
use rplan_core::*;
use rplan_io::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub(crate) fn export(request: Request, assignments: BTreeMap<String, u32>, label: String, chamber: String, created_at: String) -> Result<Value, String> {
    let g = &request.graph;
    validate(g, &request.options)?;
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
    if let Some(input)=&request.partisan {producer.insert("partisan_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    if let Some(input)=&request.elections {producer.insert("election_input".into(),serde_json::to_value(input).map_err(|e|e.to_string())?);}
    let document=RplanDocument {rplan_version:RPLAN_V02.into(),plan,metadata:RplanMetadataV02 {label,jurisdiction:g.state.clone(),chamber,created_at,description:Some("Prepared graph export; engine district labels converted from one-based to native zero-based IDs. Generation and optimality are not certified.".into())},provenance:RplanProvenance {producer,source_hashes:context.source_hashes.entries.clone(),conversion_lineage:vec![json!({"operation":"prepared-graph-export","edge_kind":"custom","reason":"Prepared graph does not retain bridge or physical edge classification."})]},geometry:None,extensions:BTreeMap::new()};
    // Exercise the same readers used by native file consumers before emitting anything.
    read_rplan_str(&write_rplan_string(&document).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    read_rctx_str(&write_rctx_string(&context).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    Ok(json!({"schema_version":"bisect-engine-plan-export-v1","demographic_basis":request.demographics.as_ref().map(|input|&input.basis),"plan_hash":document.plan.plan_hash().map_err(|e|e.to_string())?,"context_hash":context.context_hash,"prepared_graph_hash":graph_hash,"document":document,"context":context,"source_boundary":"Unsigned supplied assignments; export validates shape and hashes but does not replay generation. Custom graph edges preserve prepared adjacency and physical weights; original edge types and geometry are unavailable. Explicit VAP/CVAP counts are retained in native demographic fields; fractions and total-population proxies do not supply VAP counts."}))
}
