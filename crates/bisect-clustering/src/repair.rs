#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RepairStatus {
    NotNeeded,
    Needed,
    Repaired,
    NotAttempted,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepairResult {
    pub assignment: Vec<usize>,
    pub changed_units: usize,
}

pub fn repair_to_valid_small(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    assignment: &[usize],
    k: usize,
    tolerance: f64,
) -> Option<RepairResult> {
    if assignment.len() != weights.len()
        || assignment.len() != adjacency.len()
        || assignment.len() > 16
    {
        return None;
    }

    let mut current = vec![0usize; assignment.len()];
    let mut best: Option<(usize, usize, Vec<usize>)> = None;
    enumerate_assignments(
        0,
        &mut current,
        adjacency,
        weights,
        assignment,
        k,
        tolerance,
        &mut best,
    );
    best.map(|(changed_units, _, assignment)| RepairResult {
        assignment,
        changed_units,
    })
}

fn enumerate_assignments(
    idx: usize,
    current: &mut [usize],
    adjacency: &[Vec<usize>],
    weights: &[i64],
    original: &[usize],
    k: usize,
    tolerance: f64,
    best: &mut Option<(usize, usize, Vec<usize>)>,
) {
    if idx == current.len() {
        if !crate::metrics::all_clusters_connected(adjacency, current, k) {
            return;
        }
        if crate::metrics::population_deviation(weights, current, k) > tolerance {
            return;
        }
        let changed = current
            .iter()
            .zip(original.iter())
            .filter(|(a, b)| a != b)
            .count();
        let cut = crate::metrics::edge_cut(adjacency, current);
        let candidate = (changed, cut, current.to_vec());
        if best.as_ref().map_or(true, |existing| &candidate < existing) {
            *best = Some(candidate);
        }
        return;
    }

    for district in 0..k {
        current[idx] = district;
        enumerate_assignments(
            idx + 1,
            current,
            adjacency,
            weights,
            original,
            k,
            tolerance,
            best,
        );
    }
}
