fn main() -> anyhow::Result<()> {
    let adjacency = vec![
        vec![1],
        vec![0, 2],
        vec![1, 3],
        vec![2, 4],
        vec![3, 5],
        vec![4],
    ];
    let weights = vec![100; 6];
    let config = bisect_apportion::SpectralConfig {
        max_iters: 200,
        tolerance: 0.05,
        target_fraction: 0.5,
    };
    let result = bisect_apportion::spectral_bisect(&adjacency, &weights, config)?;
    let payload = serde_json::json!({
        "schema_version": "method-transcript-v1",
        "package_id": "T.14+spectral-generated-synthetic",
        "package_tier": "method-produced-fixture",
        "paper": "T.14+spectral-partitioning",
        "producer_crate": "bisect-apportion",
        "producer_version": env!("CARGO_PKG_VERSION"),
        "method": "spectral",
        "source_workflow": "cargo run -p bisect-apportion --no-default-features --example spectral_path6",
        "status": "partitioned",
        "seed": null,
        "input_fixture": "path6-synthetic",
        "adjacency": adjacency,
        "populations": weights,
        "assignment": result.assignment,
        "vector": result.vector,
        "summary": result.summary,
        "scope_note": "Real crate-generated spectral bisection over a deterministic six-unit path fixture; validates spectral package generation and lineage, not real-data construction quality."
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
