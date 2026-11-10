use bisect_local_search::{improve_one_move, ImproveError, LocalSearchConfig};

#[test]
fn invalid_inputs_return_errors_before_calculation() {
    let graph = vec![vec![1], vec![0]];
    let run = |g: &[Vec<usize>], p: &[i64], a: &[usize], k, tolerance| {
        assert!(matches!(improve_one_move(g, p, a, LocalSearchConfig::one_move(k, tolerance)), Err(ImproveError::InvalidInput(_))));
    };
    for tolerance in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1] {
        run(&graph, &[1, 1], &[0, 1], 2, tolerance);
    }
    for p in [vec![0, 0], vec![-1, 2], vec![i64::MAX, 1]] {
        run(&graph, &p, &[0, 1], 2, 1.0);
    }
    run(&graph, &[1, 1], &[0, 1], usize::MAX, 1.0);
    run(&[], &[], &[], 1, 1.0);
    for g in [vec![vec![1], vec![]], vec![vec![1, 1], vec![0]], vec![vec![0, 1], vec![0]], vec![vec![2], vec![0]]] {
        run(&g, &[1, 1], &[0, 1], 2, 1.0);
    }
}

fn feasible(g: &[Vec<usize>], p: &[i64], a: &[usize], tolerance: f64) -> bool {
    let total: i128 = p.iter().map(|&x| i128::from(x)).sum();
    for district in 0..2 {
        let members: Vec<_> = (0..a.len()).filter(|&i| a[i] == district).collect();
        if members.is_empty() { return false; }
        let pop: i128 = members.iter().map(|&i| i128::from(p[i])).sum();
        if (2 * pop - total).abs() as f64 / total as f64 > tolerance { return false; }
        let mut seen = vec![false; a.len()];
        seen[members[0]] = true;
        // Independent reachability by repeated edge relaxation.
        for _ in 0..a.len() {
            for &i in &members {
                if seen[i] { for &j in &g[i] { if a[j] == district { seen[j] = true; } } }
            }
        }
        if members.iter().any(|&i| !seen[i]) { return false; }
    }
    true
}

fn cut(g: &[Vec<usize>], a: &[usize]) -> usize {
    (0..a.len()).flat_map(|i| g[i].iter().map(move |&j| (i, j)))
        .filter(|&(i, j)| i < j && a[i] != a[j]).count()
}

#[test]
fn every_four_unit_graph_matches_independent_best_feasible_move() {
    let edges = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    let mut checked = 0;
    for mask in 0..64 {
        let mut g = vec![vec![]; 4];
        for (bit, &(i, j)) in edges.iter().enumerate() {
            if mask & (1 << bit) != 0 { g[i].push(j); g[j].push(i); }
        }
        for p in [vec![1, 1, 1, 1], vec![0, 1, 2, 3], vec![3, 2, 5, 4]] {
            for tolerance in [0.0, 0.5, 1.0] {
                for labels in 0..16 {
                    let a: Vec<_> = (0..4).map(|i| (labels >> i) & 1).collect();
                    if !feasible(&g, &p, &a, tolerance) { continue; }
                    let mut candidates = vec![];
                    for i in 0..4 {
                        let to = 1 - a[i];
                        if !g[i].iter().any(|&j| a[j] == to) { continue; }
                        let mut b = a.clone(); b[i] = to;
                        if feasible(&g, &p, &b, tolerance) && cut(&g, &b) < cut(&g, &a) {
                            candidates.push(((cut(&g, &b), i, a[i], to), b));
                        }
                    }
                    candidates.sort_by_key(|x| x.0);
                    let result = improve_one_move(&g, &p, &a, LocalSearchConfig::one_move(2, tolerance)).unwrap();
                    assert_eq!(result.assignment, candidates.first().map(|x| x.1.clone()).unwrap_or(a));
                    assert_eq!(result.moved_vertex, candidates.first().map(|x| x.0.1));
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 1_000, "only {checked} feasible cases checked");
}

#[test]
fn maximum_population_and_zero_population_unit_are_supported() {
    let result = improve_one_move(&[vec![1], vec![0]], &[i64::MAX, 0], &[0, 0], LocalSearchConfig::one_move(1, 0.0)).unwrap();
    assert_eq!(result.assignment, vec![0, 0]);
    assert_eq!(result.summary.final_population_deviation, 0.0);
}
