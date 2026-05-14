pub fn edge_cut(adjacency: &[Vec<usize>], assignment: &[usize]) -> usize {
    rgraph_core::undirected_edge_cut(adjacency, assignment)
        .expect("validated local-search adjacency and assignment")
}

pub fn population_deviation(weights: &[i64], assignment: &[usize], k: usize) -> f64 {
    if k == 0 {
        return f64::INFINITY;
    }
    let total: i64 = weights.iter().sum();
    let ideal = total as f64 / k as f64;
    let mut district_pop = vec![0i64; k];
    for (&district, &weight) in assignment.iter().zip(weights.iter()) {
        if district < k {
            district_pop[district] += weight;
        }
    }
    district_pop
        .into_iter()
        .map(|pop| (pop as f64 - ideal).abs() / ideal)
        .fold(0.0, f64::max)
}

pub fn all_districts_connected(adjacency: &[Vec<usize>], assignment: &[usize], k: usize) -> bool {
    rgraph_core::assignment_labels_connected(adjacency, assignment, 0..k)
        .expect("validated local-search adjacency and assignment")
}
