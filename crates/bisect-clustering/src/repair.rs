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
        0,
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
    changed: usize,
) {
    // The primary objective is changed units. A prefix cannot reduce that
    // count, so this excludes only candidates strictly worse than the
    // incumbent. Equal counts still compete on cut and lexicographic labels.
    if best.as_ref().is_some_and(|existing| changed > existing.0) {
        return;
    }
    if idx == current.len() {
        if !crate::metrics::all_clusters_connected(adjacency, current, k) {
            return;
        }
        if crate::metrics::population_deviation(weights, current, k) > tolerance {
            return;
        }
        let cut = crate::metrics::edge_cut(adjacency, current);
        let candidate = (changed, cut, current.to_vec());
        if best.as_ref().map_or(true, |existing| &candidate < existing) {
            *best = Some(candidate);
        }
        return;
    }

    // Try the original label first to obtain a useful incumbent early.
    // Candidate comparison retains the original global tuple ordering.
    let original_label = original[idx];
    let labels = (original_label < k).then_some(original_label).into_iter()
        .chain((0..k).filter(|&district| district != original_label));
    for district in labels {
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
            changed + usize::from(district != original_label),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exhaustive(adjacency: &[Vec<usize>], weights: &[i64], original: &[usize], k: usize, tolerance: f64) -> Option<RepairResult> {
        let mut best = None;
        for code in 0..k.pow(original.len() as u32) {
            let mut code = code;
            let candidate: Vec<_> = (0..original.len()).map(|_| {let d = code % k; code /= k; d}).collect();
            if !crate::metrics::all_clusters_connected(adjacency, &candidate, k)
                || crate::metrics::population_deviation(weights, &candidate, k) > tolerance {continue;}
            let changed = candidate.iter().zip(original).filter(|(a,b)| a != b).count();
            let tuple = (changed, crate::metrics::edge_cut(adjacency, &candidate), candidate);
            if best.as_ref().map_or(true, |existing| &tuple < existing) {best = Some(tuple);}
        }
        best.map(|(changed_units, _, assignment)| RepairResult {changed_units, assignment})
    }

    #[test]
    fn pruning_preserves_exhaustive_objective_and_label_ties() {
        for n in 2usize..=7 {
            let line: Vec<Vec<usize>> = (0..n).map(|i| [i.checked_sub(1), (i+1<n).then_some(i+1)].into_iter().flatten().collect()).collect();
            let mut ring = line.clone();
            if n > 2 {ring[0].push(n-1); ring[n-1].push(0);}
            for adjacency in [&line, &ring] {
                for k in 1..=3.min(n) {
                    for weights in [vec![100; n], (0..n).map(|i| if i % 3 == 0 {0} else {100+i as i64}).collect()] {
                        for original in [vec![0; n], (0..n).map(|i|i%k).collect(), (0..n).map(|i|(n-1-i)%k).collect(), vec![k; n]] {
                            for tolerance in [0.0, 0.25, 0.5] {
                                assert_eq!(repair_to_valid_small(adjacency, &weights, &original, k, tolerance), exhaustive(adjacency, &weights, &original, k, tolerance));
                            }
                        }
                    }
                }
            }
        }
    }
}
