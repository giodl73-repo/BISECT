//! Native hashed contexts for browser comparison parity gates.
use serde_json::{json, Value};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args()
        .nth(1)
        .ok_or("Specify a new fixture output directory.")?;
    std::fs::create_dir_all(&out)?;
    let a = rplan_io::read_rplan_str(include_str!(
        "../../rplan-audit/fixtures/grid3x3-valid.rplan"
    ))?;
    let mut context =
        rplan_io::read_rctx_str(include_str!("../../rplan-audit/fixtures/grid3x3.rctx"))?;
    context.subdivisions = Some(rplan_core::SubdivisionContext {
        county_ids: Some(vec![Some("synthetic-county".into()); 9]),
        municipal_ids: Some(
            (0..9)
                .map(|i| Some(format!("synthetic-place-{}", i / 2)))
                .collect(),
        ),
    });
    let mut contexts = serde_json::Map::new();
    for name in [
        "full",
        "population-only",
        "graph-only",
        "partial-subdivisions",
        "zero-population",
        "large-population",
        "missing-subdivisions",
        "no-graph",
    ] {
        let mut c = context.clone();
        match name {
            "population-only" => {
                c.graph = None;
                c.subdivisions = None;
            }
            "graph-only" => {
                c.populations = None;
                c.subdivisions = None;
            }
            "partial-subdivisions" => {
                let s = c.subdivisions.as_mut().unwrap();
                s.county_ids.as_mut().unwrap()[0] = None;
                s.municipal_ids = Some(vec![None; 9]);
            }
            "zero-population" => c.populations = Some(vec![0; 9]),
            "large-population" => c.populations = Some(vec![1i64 << 52; 9]),
            "missing-subdivisions" => c.subdivisions = None,
            "no-graph" => c.graph = None,
            _ => {}
        }
        c.context_hash = c.compute_context_hash()?;
        c.validate()?;
        contexts.insert(name.into(), serde_json::to_value(c)?);
    }
    let file = format!("{out}/contexts.json");
    std::fs::write(
        file,
        serde_json::to_vec_pretty(&json!({"plan":a,"contexts":Value::Object(contexts)}))?,
    )?;
    Ok(())
}
