fn main() {
    if std::env::args().any(|arg| arg == "--matrix") {
        matrix();
        return;
    }
    let result = bisect_wasm_cffi_probe::partition_probe();
    let disconnected = bisect_wasm_cffi_probe::disconnected_probe();
    let after_error = bisect_wasm_cffi_probe::partition_probe();
    println!(
        "{{\"cycle\":{result},\"disconnected\":{disconnected},\"after_error\":{after_error}}}"
    );
    if result < 0 || disconnected >= 0 || after_error != result {
        std::process::exit(1);
    }
}

extern "C" {
    fn srand(seed: u32);
    fn rand() -> i32;
}

fn matrix() {
    // Probe the actual linked host C runtime, not a Rust reimplementation.
    let rng_seeds = [0_u32, 1, 7, 2147483647, 4294967295];
    let rng: Vec<Vec<i32>> = rng_seeds
        .iter()
        .map(|&seed| unsafe {
            srand(seed);
            (0..64).map(|_| rand()).collect()
        })
        .collect();
    let mut cases = Vec::new();
    for (width, height) in [(6_i32, 5_i32), (20, 15)] {
        let n = width * height;
        for weighted in 0..3 {
            let ncon = if weighted == 2 { 2 } else { 1 };
            let mut xadj = vec![0];
            let mut adjncy = Vec::new();
            let mut edge_weights = Vec::new();
            let mut vertex_weights = Vec::new();
            for i in 0_i32..n {
                let mut neighbors = Vec::new();
                if i >= width {
                    neighbors.push(i - width);
                }
                if i % width > 0 {
                    neighbors.push(i - 1);
                }
                if i % width < width - 1 {
                    neighbors.push(i + 1);
                }
                if i < n - width {
                    neighbors.push(i + width);
                }
                for j in neighbors {
                    adjncy.push(j);
                    edge_weights.push(1 + i.min(j) % 7);
                }
                xadj.push(adjncy.len() as i32);
                vertex_weights.push(100 + i % 5 * 10);
                if ncon == 2 {
                    vertex_weights.push(50 + i % 3 * 5);
                }
            }
            let tpwgts: Vec<f32> = if ncon == 2 {
                vec![0.4, 0.4, 0.35, 0.35, 0.25, 0.25]
            } else {
                vec![0.4, 0.35, 0.25]
            };
            for seed in [1, 7, 42, 2147483647] {
                let mut part = vec![0; n as usize];
                let mut graph = metis::Graph::new(ncon, 3, &xadj, &adjncy)
                    .unwrap()
                    .set_tpwgts(&tpwgts)
                    .set_option(metis::option::Seed(seed))
                    .set_option(metis::option::UFactor(30))
                    .set_option(metis::option::NIter(10))
                    .set_option(metis::option::Contig(true))
                    .set_option(metis::option::MinConn(true));
                if weighted != 0 {
                    graph = graph.set_vwgt(&vertex_weights).set_adjwgt(&edge_weights);
                }
                let outcome = match graph.part_kway(&mut part) {
                    Ok(cut) => format!("\"code\":1,\"cut\":{cut},\"part\":{part:?}"),
                    Err(metis::Error::Input) => "\"code\":-2".into(),
                    Err(metis::Error::Memory) => "\"code\":-3".into(),
                    Err(metis::Error::Other) => "\"code\":-4".into(),
                };
                let vwgt = if weighted == 0 {
                    &[][..]
                } else {
                    &vertex_weights[..]
                };
                let adjwgt = if weighted == 0 {
                    &[][..]
                } else {
                    &edge_weights[..]
                };
                cases.push(format!("{{\"id\":\"grid-{width}x{height}-{weighted}-seed-{seed}\",\"n\":{n},\"ncon\":{ncon},\"k\":3,\"seed\":{seed},\"xadj\":{xadj:?},\"adjncy\":{adjncy:?},\"vwgt\":{vwgt:?},\"adjwgt\":{adjwgt:?},\"tpwgts\":{tpwgts:?},{outcome}}}"));
            }
        }
    }
    let cycle = bisect_wasm_cffi_probe::partition_probe();
    let disconnected = bisect_wasm_cffi_probe::disconnected_probe();
    let after_error = bisect_wasm_cffi_probe::partition_probe();
    let invalid_option = bisect_wasm_cffi_probe::invalid_option_probe();
    let after_invalid = bisect_wasm_cffi_probe::partition_probe();
    println!("{{\"rng_seeds\":{rng_seeds:?},\"rng\":{rng:?},\"cases\":[{}],\"cycle\":{cycle},\"disconnected\":{disconnected},\"after_error\":{after_error},\"invalid_option\":{invalid_option},\"after_invalid\":{after_invalid}}}", cases.join(","));
    if cycle < 0
        || disconnected != -4
        || cycle != after_error
        || invalid_option != -2
        || cycle != after_invalid
    {
        std::process::exit(1);
    }
}
