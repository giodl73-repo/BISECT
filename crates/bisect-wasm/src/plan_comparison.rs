//! Two-plan descriptive comparison through shared native analysis kernels.
use rplan_audit::measurements::{measure_plan, PlanMeasurements};
use rplan_core::DistrictPlan;
use serde_json::{json, Value};
use std::collections::HashMap;
use sha2::{Digest,Sha256};

pub(crate) fn compare(a: Value, b: Value, context: Option<Value>) -> Result<Value, String> {
    compare_native_text(&a.to_string(),&b.to_string(),context.as_ref().map(Value::to_string).as_deref())
}
pub(crate) fn compare_text(a:&str,b:&str,context:Option<&str>)->Result<Value,String>{
    if a.len()>3*1024*1024||b.len()>3*1024*1024||context.is_some_and(|c|c.len()>8*1024*1024){return Err("Comparison text exceeds the plan/context byte limits.".into());}
    let mut result=compare_native_text(a,b,context)?;
    let hash=|text:&str|format!("{:x}",Sha256::digest(text.as_bytes()));
    result["source_sha256"]=json!({"plan_a":hash(a),"plan_b":hash(b),"context":context.map(hash)});
    Ok(result)
}
fn compare_native_text(a:&str,b:&str,context:Option<&str>)->Result<Value,String>{
    let a = rplan_io::read_rplan_str(a).map_err(|e| e.to_string())?;
    let b = rplan_io::read_rplan_str(b).map_err(|e| e.to_string())?;
    let ua = &a.plan.units;
    let ub = &b.plan.units;
    if ua.unit_ids != ub.unit_ids
        || ua.unit_kind != ub.unit_kind
        || ua.state != ub.state
        || ua.year != ub.year
        || ua.unit_universe_hash != ub.unit_universe_hash
    {
        return Err("Comparison requires identical ordered units, unit kind, state and Census year. Cross-cycle comparison requires an explicit crosswalk.".into());
    }
    let n = ua.unit_ids.len();
    if n > 100_000
        || a.plan.k > 500
        || b.plan.k > 500
        || n.checked_mul(a.plan.k + b.plan.k)
            .is_none_or(|v| v > 50_000_000)
    {
        return Err("Comparison exceeds the portable unit/district work limit.".into());
    }
    let context = context
        .map(|c| rplan_io::read_rctx_str(c).map_err(|e| e.to_string()))
        .transpose()?;
    if context
        .as_ref()
        .and_then(|c| c.graph.as_ref())
        .is_some_and(|g| g.adjacency.iter().map(Vec::len).sum::<usize>() > 2_000_000)
    {
        return Err("Comparison graph exceeds the portable edge limit.".into());
    }
    let ma = measure_plan(&a.plan, context.as_ref())?;
    let mb = measure_plan(&b.plan, context.as_ref())?;
    let assignments = |p: &DistrictPlan| {
        p.units
            .unit_ids
            .iter()
            .cloned()
            .zip(p.assignment.iter().map(|&d| d as usize + 1))
            .collect::<HashMap<_, _>>()
    };
    let aa = assignments(&a.plan);
    let ab = assignments(&b.plan);
    let forward = bisect_analysis::try_compare_plans(&aa, &ab).map_err(|e| e.to_string())?;
    let reverse = bisect_analysis::try_compare_plans(&ab, &aa).map_err(|e| e.to_string())?;
    let changes = bisect_analysis::comparison::assignment_changes(
        aa.iter().map(|(unit, &district)| (unit.as_str(), district)),
        ab.iter().map(|(unit, &district)| (unit.as_str(), district)),
    );
    let changed_population =
        context
            .as_ref()
            .and_then(|c| c.populations.as_ref())
            .map(|populations| {
                a.plan
                    .assignment
                    .iter()
                    .zip(&b.plan.assignment)
                    .zip(populations)
                    .filter_map(|((&da, &db), &population)| (da != db).then_some(population))
                    .sum::<i64>()
                    .to_string()
            });
    let assignment_change = json!({
        "common_units": changes.common_units,
        "changed_units": changes.changed_units.len(),
        "population_in_changed_units": changed_population,
        "district_ids_with_changes": changes.districts_with_changes.iter().map(|d| d-1).collect::<Vec<_>>(),
        "method": "Label-sensitive assignment difference on identical ordered units. Pure district renumbering counts as change; no best matching or population movement is implied. District IDs are zero based; touched IDs combine both plans."
    });
    let max_deviation = |m: &PlanMeasurements| {
        m.districts
            .iter()
            .filter_map(|d| d.population.as_ref().map(|p| p.percent_deviation))
            .reduce(f64::max)
    };
    let population = if ma.total_population.is_some_and(|p| p > 0) {
        let pa = max_deviation(&ma).unwrap();
        let pb = max_deviation(&mb).unwrap();
        json!({"status":"measured","plan_a_max_deviation_percent":pa,"plan_b_max_deviation_percent":pb,"lower":if (pa-pb).abs()<1e-9{"equal"}else if pa<pb{"plan_a"}else{"plan_b"},"difference_percentage_points":(pa-pb).abs()})
    } else {
        json!({"status":"not-evaluated","reason":if ma.total_population==Some(0){"Total population is zero; relative deviation is undefined."}else{"A matching context with populations is required."}})
    };
    let summary = |doc: &rplan_io::RplanDocument, m: PlanMeasurements| json!({"label":doc.metadata.label,"plan_hash":doc.plan.plan_hash().map_err(|e|e.to_string()).unwrap(),"units":n,"districts":doc.plan.k,"allow_empty_districts":doc.plan.allow_empty_districts,"total_population":m.total_population.map(|v|v.to_string()),"district_measurements":m.districts.iter().map(|d|json!({"district_id":d.district_id,"label":doc.plan.display_labels.get(d.district_id as usize),"units":d.units,"population":d.population.as_ref().map(|p|p.population.to_string()),"deviation_percent":if m.total_population==Some(0){None}else{d.population.as_ref().map(|p|p.percent_deviation)},"components":d.component_count})).collect::<Vec<_>>(),"contiguity":if m.districts.iter().all(|d|d.component_count.is_some()){json!({"status":"measured","all_nonempty_and_connected":m.districts.iter().all(|d|d.component_count==Some(1)),"scope":"Supplied graph connectivity includes every edge classification, including declared bridges."})}else{json!({"status":"not-evaluated","reason":"A matching context graph is required."})},"county":m.county,"municipal":m.municipal});
    Ok(
        json!({"schema_version":"bisect-plan-comparison-v1","plan_a":summary(&a,ma),"plan_b":summary(&b,mb),"context_hash":context.as_ref().map(|c|&c.context_hash),"unit_kind":ua.unit_kind,"state":ua.state,"year":ua.year,"assignment_change":assignment_change,"similarity":{"a_to_b":forward.jaccard_similarity,"b_to_a":reverse.jaccard_similarity,"method":"Mean independently best district Jaccard; directional, unweighted by population, B districts may be reused. Empty allowed districts have no unit sets and are omitted from this metric."},"population":population,"unit_count_proxy":{"plan_a_max_deviation_percent":forward.population.plan_a_max_dev*100.,"plan_b_max_deviation_percent":forward.population.plan_b_max_dev*100.,"scope":"Native comparison proxy uses unit counts, not population; empty districts are omitted."},"compactness":{"status":"not-evaluated","reason":"No projected district polygon measurement inputs are supplied; context geometry hashes and display maps are insufficient."},"partisan":{"status":"not-evaluated","reason":"Election observations are not supplied to this operation."},"source_boundary":"Descriptive comparison of supplied plans and optional context. No legal profile, audit certificate, authenticated provenance, engine replay or optimality claim is implied."}),
    )
}
