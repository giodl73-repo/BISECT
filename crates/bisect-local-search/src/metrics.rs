use std::collections::{HashSet, VecDeque};

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
    (0..k).all(|district| district_connected(adjacency, assignment, district))
}

fn district_connected(adjacency: &[Vec<usize>], assignment: &[usize], district: usize) -> bool {
    let members: Vec<usize> = assignment
        .iter()
        .enumerate()
        .filter_map(|(idx, &assigned)| (assigned == district).then_some(idx))
        .collect();
    if members.is_empty() {
        return false;
    }
    let member_set: HashSet<usize> = members.iter().copied().collect();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(members[0]);
    queue.push_back(members[0]);
    while let Some(node) = queue.pop_front() {
        for &neighbor in &adjacency[node] {
            if member_set.contains(&neighbor) && visited.insert(neighbor) {
                queue.push_back(neighbor);
            }
        }
    }
    visited.len() == members.len()
}
