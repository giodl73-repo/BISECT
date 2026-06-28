//! Engine unit tests, split by algorithm to mirror the production submodules.
//! Shared graph-construction helpers live here so every test submodule can
//! reach them (and the re-exported engine API) via `use super::*;`.

pub(crate) use super::*;

/// Helper: compute the VRASection alignment score the same way run_geosection does.
/// alignment = |MVAP_frac(left) - MVAP_frac(right)| normalised to [0, 1]
/// = |mvap_left/mvap_total - (1 - mvap_left/mvap_total)| = |2*mvap_left/mvap_total - 1|
pub(crate) fn vra_alignment(mvap: &[f64], left: &[usize]) -> f64 {
    let mvap_total: f64 = mvap.iter().sum();
    if mvap_total == 0.0 {
        return 0.0;
    }
    let mvap_left: f64 = left.iter().map(|&v| mvap[v]).sum();
    (mvap_left / mvap_total - 0.5).abs() * 2.0
}

pub(crate) fn small_grid(rows: usize, cols: usize) -> (Vec<Vec<usize>>, Vec<i64>) {
    let n = rows * cols;
    let mut adj = vec![vec![]; n];
    for r in 0..rows {
        for c in 0..cols {
            let v = r * cols + c;
            if c + 1 < cols {
                adj[v].push(v + 1);
                adj[v + 1].push(v);
            }
            if r + 1 < rows {
                adj[v].push(v + cols);
                adj[v + cols].push(v);
            }
        }
    }
    let pop = vec![1000i64; n];
    (adj, pop)
}

pub(crate) fn grid4x4_centroids() -> Vec<(f64, f64)> {
    let mut c = Vec::with_capacity(16);
    for row in 0..4 {
        for col in 0..4 {
            // lon: -100.0 + col*1.0, lat: 37.0 + row*1.0
            c.push((-100.0 + col as f64, 37.0 + row as f64));
        }
    }
    c
}

/// 4×2 grid helper: 8 nodes, 2 districts, uniform pop=1000. k=2.
pub(crate) fn grid8_adj() -> Vec<Vec<usize>> {
    // 0-1-2-3 top row, 4-5-6-7 bottom row; vertical edges 0-4, 1-5, 2-6, 3-7.
    vec![
        vec![1, 4],    // 0
        vec![0, 2, 5], // 1
        vec![1, 3, 6], // 2
        vec![2, 7],    // 3
        vec![0, 5],    // 4
        vec![4, 1, 6], // 5
        vec![5, 2, 7], // 6
        vec![6, 3],    // 7
    ]
}

pub(crate) fn grid8_pop() -> Vec<i64> {
    vec![1000i64; 8]
}

/// Build a 4x4 grid adjacency for flip tests.
pub(crate) fn grid_4x4() -> (Vec<Vec<usize>>, Vec<i64>) {
    let (adj, pop) = small_grid(4, 4);
    (adj, pop)
}

/// Build a synthetic geoid map for an n-tract grid split into two counties.
/// Tracts 0..(n/2) -> county "37001", tracts (n/2)..n -> county "37003".
pub(crate) fn synthetic_geoids(n: usize) -> std::collections::HashMap<usize, String> {
    (0..n)
        .map(|i| {
            let county = if i < n / 2 { "37001" } else { "37003" };
            let tract_num = i % (n / 2);
            let geoid = format!("{county}{tract_num:06}");
            (i, geoid)
        })
        .collect()
}

/// Helper: build synthetic (lon, lat) centroids on a 4x4 grid spaced 0.01° apart.
/// Global index = row * cols + col, origin at (-96.05, 37.45).
pub(crate) fn synthetic_centroids(rows: usize, cols: usize) -> Vec<(f64, f64)> {
    let mut c = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for col in 0..cols {
            let lon = -96.05 + col as f64 * 0.01;
            let lat = 37.45 + r as f64 * 0.01;
            c.push((lon, lat));
        }
    }
    c
}

mod adaptive_multiscale;
mod bfs;
mod bisection_ensemble;
mod core;
mod cvd;
mod dispatch;
mod flip;
mod forest_recom;
mod ilp;
mod merge_split;
mod mka;
mod multiscale;
mod nway_geo;
mod percentile;
mod short_burst;
mod simulated_annealing;
mod smc;
