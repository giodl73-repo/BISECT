//! Native config.New dry-run bytes and explicit engine/seed extension reference.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("Template settings JSON required")?;
    let s: bisect_runner::config::TemplateConfig = serde_json::from_slice(&std::fs::read(path)?)?;
    let text = if s.engine.is_none() && s.seed.is_none() {
        bisect_cli::algo_config::write_template_config(
            &s.name,
            &s.structure,
            s.weights.as_deref(),
            s.alpha_county,
            s.search.as_deref(),
            s.convergence_threshold,
            s.seeds,
            s.balance_tolerance,
            &s.years,
            s.workers,
            None,
            false,
            true,
        )?
    } else {
        bisect_runner::config::render_template_config(&s)?
    };
    // Generated YAML must also pass the actual native parser and typed adapter.
    use bisect_cli::algo_config::NativeAlgorithmConfig;
    bisect_cli::algo_config::AlgoYaml::from_text(&text)?.to_algorithm_config()?;
    print!("{text}");
    Ok(())
}
