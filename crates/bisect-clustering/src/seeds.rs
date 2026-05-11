use std::collections::VecDeque;

pub fn farthest_point_seeds(adjacency: &[Vec<usize>], k: usize) -> Result<Vec<usize>, String> {
    if k == 0 {
        return Err("k must be greater than zero".to_string());
    }
    if k > adjacency.len() {
        return Err(format!("k={} exceeds graph size {}", k, adjacency.len()));
    }
    if adjacency.is_empty() {
        return Err("graph must contain at least one unit".to_string());
    }

    let mut seeds = vec![0usize];
    while seeds.len() < k {
        let distances = distances_to_any_seed(adjacency, &seeds);
        let next = distances
            .iter()
            .enumerate()
            .filter(|(idx, _)| !seeds.contains(idx))
            .max_by_key(|(idx, distance)| (*distance, std::cmp::Reverse(*idx)))
            .map(|(idx, _)| idx)
            .ok_or_else(|| "could not select next seed".to_string())?;
        seeds.push(next);
    }
    Ok(seeds)
}

pub fn distances_from(adjacency: &[Vec<usize>], start: usize) -> Vec<Option<usize>> {
    let mut distances = vec![None; adjacency.len()];
    if start >= adjacency.len() {
        return distances;
    }
    let mut queue = VecDeque::new();
    distances[start] = Some(0);
    queue.push_back(start);
    while let Some(node) = queue.pop_front() {
        let next_distance = distances[node].unwrap() + 1;
        for &neighbor in &adjacency[node] {
            if neighbor < adjacency.len() && distances[neighbor].is_none() {
                distances[neighbor] = Some(next_distance);
                queue.push_back(neighbor);
            }
        }
    }
    distances
}

fn distances_to_any_seed(adjacency: &[Vec<usize>], seeds: &[usize]) -> Vec<usize> {
    let mut out = vec![usize::MAX; adjacency.len()];
    for &seed in seeds {
        for (idx, distance) in distances_from(adjacency, seed).into_iter().enumerate() {
            if let Some(distance) = distance {
                out[idx] = out[idx].min(distance);
            }
        }
    }
    out
}
