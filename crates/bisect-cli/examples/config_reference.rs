//! File-backed native config command/conversion reference for the browser gate.
use bisect_cli::algo_config::{
    validate_config, validate_config_court, AlgoYaml, NativeAlgorithmConfig,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("YAML path required")?;
    if std::env::args().nth(2).as_deref() == Some("court") {
        validate_config_court(std::path::Path::new(&path))?;
    }
    let yaml = AlgoYaml::from_file(std::path::Path::new(&path))?;
    let native = yaml.to_algorithm_config()?;
    println!(
        "{}",
        serde_json::json!({
            "summary":validate_config(std::path::Path::new(&path))?,
            "split":native.split.mode_name(),
            "geographic":native.weights.geographic,
            "minority_weighting":native.weights.minority_weighting,
            "alpha_county":native.weights.alpha_county,
            "vertex_constraints":native.vertex_constraints.iter().map(|v|format!("{v:?}")).collect::<Vec<_>>(),
            "search":format!("{:?}",native.seeds),
            "engine":format!("{:?}",native.metis.engine),
            "seed":native.metis.seed.map(|n|n.to_string()),
            "ufactor":native.metis.ufactor,"niter":native.metis.niter,
        })
    );
    Ok(())
}
