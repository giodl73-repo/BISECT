use super::*;

// ── Group 2: run_nway_partition ──────────────────────────────────────────

#[test]
fn test_run_nway_partition_basic() {
    // 12 tracts into 3 districts — n-way partition
    let n = 12;
    let adj: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            let mut nbrs = vec![];
            if i > 0 {
                nbrs.push(i - 1);
            }
            if i < n - 1 {
                nbrs.push(i + 1);
            }
            nbrs
        })
        .collect();
    let vw = vec![1000i64; n];
    let ew = HashMap::new();
    let result = run_nway_partition(&adj, &vw, &ew, 3, 1.05, 100, Some(42));
    assert!(
        result.is_ok(),
        "n-way partition should succeed: {:?}",
        result.err()
    );
    let assignments = result.unwrap();
    assert_eq!(assignments.len(), n, "all tracts assigned");
    let districts: std::collections::HashSet<usize> = assignments.values().copied().collect();
    assert_eq!(districts.len(), 3, "exactly 3 districts");
    assert!(districts.contains(&1) && districts.contains(&2) && districts.contains(&3));
}

// Balance quality for n-way is only guaranteed by C METIS (c-ffi-engine).
// metis-core produces correct assignments but with looser balance on small
// graphs — this is a documented known gap.
#[test]
#[cfg(feature = "c-ffi-engine")]
fn test_run_nway_partition_balance() {
    // 20 equal-weight tracts into 4 districts — should be well-balanced
    let n = 20;
    let adj: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            let mut nbrs = vec![];
            if i > 0 {
                nbrs.push(i - 1);
            }
            if i < n - 1 {
                nbrs.push(i + 1);
            }
            nbrs
        })
        .collect();
    let vw = vec![1000i64; n];
    let ew = HashMap::new();
    let assignments = run_nway_partition(&adj, &vw, &ew, 4, 1.05, 100, Some(42)).unwrap();

    let mut district_pops = vec![0i64; 5];
    for (tract, &dist) in &assignments {
        district_pops[dist] += vw[*tract];
    }
    let ideal = 20 * 1000 / 4; // 5000
    for d in 1..=4 {
        let dev = (district_pops[d] - ideal as i64).abs() as f64 / ideal as f64;
        assert!(
            dev <= 0.1,
            "district {d} deviation {:.1}% exceeds 10%",
            dev * 100.0
        );
    }
}

#[test]
fn test_run_nway_partition_output_complete_and_valid() {
    let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
    let vw = vec![1000i64; 4];
    let ew = HashMap::new();
    let assignments = run_nway_partition(&adj, &vw, &ew, 2, 1.05, 100, Some(42)).unwrap();
    // Every tract assigned, district IDs 1-based
    assert_eq!(assignments.len(), 4);
    assert!(assignments.values().all(|&d| d >= 1 && d <= 2));
    let d1: Vec<_> = assignments.values().filter(|&&d| d == 1).collect();
    let d2: Vec<_> = assignments.values().filter(|&&d| d == 2).collect();
    assert!(
        !d1.is_empty() && !d2.is_empty(),
        "both districts must have tracts"
    );
}

// ── AreaSection / dual-constraint METIS (ncon=2) tests ──────────────────

/// Verify write_metis_graph_dual produces valid ncon=2 format.
/// The header line must contain ncon=2. Each vertex line must have two weights.
#[test]
fn test_write_metis_graph_dual_format() {
    use bisect_core::metis_format::write_metis_graph_dual;
    // 3-vertex path: 0-1-2
    let adj = vec![vec![1], vec![0, 2], vec![1]];
    let pop = vec![100i64, 200, 150];
    let area = vec![500i64, 800, 600];
    let mut ew = HashMap::new();
    ew.insert((0, 1), 1000.0f64);
    ew.insert((1, 2), 1500.0f64);

    let content = write_metis_graph_dual(&adj, &pop, &area, Some(&ew)).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Header: "3 2 011 2"
    assert_eq!(
        lines[0], "3 2 011 2",
        "header must be '3 2 011 2' for ncon=2"
    );

    // Vertex 0: "100 500 2 100000 ..." (pop area neighbor1 eweight1)
    assert!(
        lines[1].starts_with("100 500 "),
        "vertex 0 must start with pop area"
    );

    // Vertex 1 (degree 2): "200 800 1 100000 3 150000"
    assert!(
        lines[2].starts_with("200 800 "),
        "vertex 1 must start with pop area"
    );

    // Vertex 2: "150 600 2 150000"
    assert!(
        lines[3].starts_with("150 600 "),
        "vertex 2 must start with pop area"
    );
}

/// Verify tpwgts file format for ncon=2 uses "partition : constraint = weight" syntax.
#[test]
fn test_dual_tpwgts_format_ncon2() {
    let pop_left = 0.4286f64; // 6/14
    let area_left = 0.5f64;
    // Correct ncon=2 format: partition : constraint = weight
    // n-1 partition format: write only partition 0, METIS infers partition 1
    // (same as Python archive: write n-1 partitions, METIS infers the last)
    let content = format!("0 : 0 = {pop_left:.6}\n0 : 1 = {area_left:.6}\n");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "n-1 format: write 1 partition × 2 constraints = 2 lines"
    );
    assert!(
        lines[0].starts_with("0 : 0 = "),
        "line 0 must be 'p0:constraint0'"
    );
    assert!(
        lines[1].starts_with("0 : 1 = "),
        "line 1 must be 'p0:constraint1'"
    );
    let p0c0: f64 = lines[0][8..].trim().parse().unwrap();
    let p0c1: f64 = lines[1][8..].trim().parse().unwrap();
    assert!(
        (p0c0 - pop_left).abs() < 1e-5,
        "constraint 0 should be pop_left"
    );
    assert!(
        (p0c1 - area_left).abs() < 1e-5,
        "constraint 1 should be area_left"
    );
}

/// Integration test: call split_subgraph with ncon=2 (unified dual-constraint path).
/// Tests that the new unified split_subgraph handles ncon=2 correctly.
#[test]
fn test_split_subgraph_ncon2_small_graph() {
    // 8-vertex grid: 0-1-2-3 (top row), 4-5-6-7 (bottom row)
    // Edges: 0-1, 1-2, 2-3, 4-5, 5-6, 6-7, 0-4, 1-5, 2-6, 3-7
    let adj = vec![
        vec![1, 4],
        vec![0, 2, 5],
        vec![1, 3, 6],
        vec![2, 7],
        vec![0, 5],
        vec![4, 6, 1],
        vec![5, 7, 2],
        vec![6, 3],
    ];
    let pop = vec![100i64; 8]; // uniform population
                               // area in hectares (already scaled; each vertex = 100 ha)
    let area_ha = vec![100i64; 8];
    // interleaved vwgt for ncon=2: [pop_0, area_0, pop_1, area_1, ...]
    let vwgt_interleaved: Vec<i64> = pop
        .iter()
        .zip(area_ha.iter())
        .flat_map(|(&p, &a)| [p, a])
        .collect();
    let mut ew = HashMap::new();
    for (u, v) in [
        (0, 1),
        (1, 2),
        (2, 3),
        (4, 5),
        (5, 6),
        (6, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ] {
        ew.insert((u.min(v), u.max(v)), 1000.0f64);
    }
    let tracts: HashSet<usize> = (0..8).collect();

    // Bisect 50/50 by both population and area (ncon=2)
    // tpwgts=[0.5, 0.5, 0.5, 0.5]: both partitions get 50% pop and 50% area
    let result = split_subgraph(
        &adj,
        &vwgt_interleaved,
        2,
        &ew,
        &tracts,
        1.005,
        100,
        Some(42),
        Some(vec![0.5f32, 0.5f32, 0.5f32, 0.5f32]),
        Some(vec![1.001f32, 1.001f32]),
    );

    match result {
        Ok((left, right)) => {
            assert_eq!(left.len() + right.len(), 8, "all vertices assigned");
            assert!(!left.is_empty() && !right.is_empty(), "non-trivial split");
            // Both halves should have ~50% of population
            let pop_left: i64 = left.iter().map(|&v| pop[v]).sum();
            let pop_total: i64 = pop.iter().sum();
            let ratio = pop_left as f64 / pop_total as f64;
            assert!(
                (ratio - 0.5).abs() <= 0.001,
                "pop balance should be ~50%: got {:.1}%",
                ratio * 100.0
            );
            let area_left: i64 = left.iter().map(|&v| area_ha[v]).sum();
            assert_eq!(
                area_left, 400,
                "both area and population constraints must be met"
            );
            assert!(is_connected_subset(&adj, &left) && is_connected_subset(&adj, &right));
        }
        Err(e) => panic!("dual-constraint split must be supported: {e}"),
    }
}

#[test]
fn areasection_two_districts_enforces_area_and_honors_swing() {
    let adj = vec![
        vec![1, 4],
        vec![0, 2, 5],
        vec![1, 3, 6],
        vec![2, 7],
        vec![0, 5],
        vec![1, 4, 6],
        vec![2, 5, 7],
        vec![3, 6],
    ];
    let population = vec![100i64; 8];
    let areas = vec![
        9000000.0, 9000000.0, 1000000.0, 1000000.0, 9000000.0, 9000000.0, 1000000.0, 1000000.0,
    ];
    let mut edges = HashMap::new();
    for u in 0..8 {
        for &v in &adj[u] {
            if u < v {
                edges.insert(
                    (u, v),
                    if (u == 1 && v == 2) || (u == 5 && v == 6) {
                        1.0
                    } else if v - u == 4 {
                        100.0
                    } else {
                        1000.0
                    },
                );
            }
        }
    }
    let empty = crate::geosection_orientation::CentroidMap::new();
    let run = |swing| {
        run_geosection_seeded(
            &adj,
            &population,
            &edges,
            2,
            0.05,
            20,
            64,
            None,
            &empty,
            0.0,
            Some(&areas),
            swing,
            None,
            0.0,
            None,
            42,
        )
        .unwrap()
    };
    let (tight, _, _, tight_cost) = run(1.1);
    let left: HashSet<_> = tight
        .iter()
        .filter(|(_, d)| **d == 1)
        .map(|(&v, _)| v)
        .collect();
    assert_eq!(left.len(), 4);
    assert!(is_connected_subset(&adj, &left));
    let fraction = left.iter().map(|&v| areas[v]).sum::<f64>() / areas.iter().sum::<f64>();
    assert!(
        (fraction - 0.5).abs() < 1e-10,
        "two-district shortcut must not ignore area: {fraction}"
    );
    let (loose, _, _, loose_cost) = run(1.9);
    let loose_fraction = loose
        .iter()
        .filter(|(_, d)| **d == 1)
        .map(|(&v, _)| areas[v])
        .sum::<f64>()
        / areas.iter().sum::<f64>();
    assert!(
        (loose_fraction - 0.5).abs() > 0.3,
        "looser area budget should permit the cheap split"
    );
    assert!(loose_cost < tight_cost);
}

#[test]
fn areasection_infeasible_atomic_area_returns_error() {
    let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
    let empty = crate::geosection_orientation::CentroidMap::new();
    let result = run_geosection_seeded(
        &adj,
        &[100; 4],
        &HashMap::new(),
        2,
        0.05,
        10,
        8,
        None,
        &empty,
        0.0,
        Some(&[9000000.0, 10000.0, 10000.0, 10000.0]),
        1.1,
        None,
        0.0,
        None,
        42,
    );
    assert!(result
        .unwrap_err()
        .contains("no sampled feasible root split"));
}

// ── VRASection (T.7): alignment score unit tests ─────────────────────────

#[test]
fn vra_section_rejects_invalid_mass_and_weight() {
    let (adj,pop)=small_grid(4,4);
    let edges=HashMap::new();
    let centers=crate::geosection_orientation::CentroidMap::new();
    for mass in [vec![1.0;15],vec![-1.0;16],vec![f64::NAN;16],vec![f64::INFINITY;16]] {
        assert!(run_geosection_seeded(&adj,&pop,&edges,2,0.005,100,1,None,&centers,0.0,None,1.1,Some(&mass),0.4,None,42).is_err());
    }
    for weight in [-0.1,1.01,f64::NAN,f64::INFINITY] {
        assert!(run_geosection_seeded(&adj,&pop,&edges,2,0.005,100,1,None,&centers,0.0,None,1.1,Some(&[0.5;16]),weight,None,42).is_err());
    }
}

#[test]
fn test_vra_alignment_perfectly_concentrated() {
    // All minority population on the left side → alignment = 1.0
    let mvap = vec![100.0, 100.0, 0.0, 0.0];
    let left = vec![0, 1];
    let a = vra_alignment(&mvap, &left);
    assert!(
        (a - 1.0).abs() < 1e-9,
        "all minority on left should give alignment=1.0, got {a}"
    );
}

#[test]
fn test_vra_alignment_equal_split() {
    // Minority population equal on both sides → alignment = 0.0
    let mvap = vec![50.0, 50.0, 50.0, 50.0];
    let left = vec![0, 1];
    let a = vra_alignment(&mvap, &left);
    assert!(
        a < 1e-9,
        "equal minority split should give alignment=0.0, got {a}"
    );
}

#[test]
fn test_vra_alignment_partial_concentration() {
    // 3/4 of minority on left, 1/4 on right → alignment = |0.75 - 0.25| = 0.5
    let mvap = vec![75.0, 0.0, 25.0, 0.0]; // total=100
    let left = vec![0]; // mvap_left = 75
    let a = vra_alignment(&mvap, &left);
    assert!(
        (a - 0.5).abs() < 1e-9,
        "3/4 concentration should give alignment=0.5, got {a}"
    );
}

#[test]
fn test_vra_selection_score_prefers_concentrated_split() {
    // When minority_vap is provided, a split with high alignment should get a
    // LOWER selection score (preferred) vs a split with the same normalised EC
    // but lower alignment.
    let normalised = 1000.0_f64;
    let w_vra = 0.40_f64;

    // High alignment (0.8): score = 1000 - 0.4 * 0.8 * max(1000, 1) = 1000 - 320 = 680
    let alignment_high = 0.8_f64;
    let score_high = normalised - w_vra * alignment_high * normalised.max(1.0);
    assert!(
        (score_high - 680.0).abs() < 1e-9,
        "score with high alignment should be 680, got {score_high}"
    );

    // Low alignment (0.1): score = 1000 - 0.4 * 0.1 * 1000 = 1000 - 40 = 960
    let alignment_low = 0.1_f64;
    let score_low = normalised - w_vra * alignment_low * normalised.max(1.0);
    assert!(
        (score_low - 960.0).abs() < 1e-9,
        "score with low alignment should be 960, got {score_low}"
    );

    // High alignment split (lower score) should be preferred
    assert!(
        score_high < score_low,
        "high alignment ({score_high}) should beat low alignment ({score_low})"
    );
}

#[test]
fn test_vra_alignment_zero_mvap_returns_zero() {
    // If there is no minority population, alignment is 0 — no preference
    let mvap = vec![0.0, 0.0, 0.0];
    let left = vec![0, 1];
    let a = vra_alignment(&mvap, &left);
    assert_eq!(a, 0.0, "zero total minority VAP must give alignment=0");
}

// ── Group: VRASection alignment score (additional) ────────────────────────

#[test]
fn vra_score_improves_as_concentration_increases() {
    // Three increasingly concentrated left-side splits.
    // alignment grows: 0% → 50% → 100% concentration.
    // Alignment score must be monotone non-decreasing.
    let mvap_total = 100.0_f64;

    // Case 1: perfectly balanced (25/25/25/25 pop, left = {0,1})
    let mvap_balanced = vec![25.0, 25.0, 25.0, 25.0];
    let a1 = vra_alignment(&mvap_balanced, &[0, 1]);

    // Case 2: 75% on left, 25% on right
    let mvap_partial = vec![37.5, 37.5, 12.5, 12.5];
    let a2 = vra_alignment(&mvap_partial, &[0, 1]);

    // Case 3: 100% on left
    let mvap_concentrated = vec![50.0, 50.0, 0.0, 0.0];
    let a3 = vra_alignment(&mvap_concentrated, &[0, 1]);

    // Suppress "unused" warning for the total variable
    let _ = mvap_total;

    assert!(
        a1 <= a2,
        "alignment must grow with concentration: {a1} <= {a2}"
    );
    assert!(
        a2 <= a3,
        "alignment must grow with concentration: {a2} <= {a3}"
    );
}

#[test]
fn vra_alignment_large_state_many_tracts() {
    // 50-tract test — verify no integer overflow or precision issues.
    // Even-indexed tracts have high minority pop, odd-indexed have none.
    let mvap: Vec<f64> = (0..50)
        .map(|i| if i % 2 == 0 { 100.0 } else { 0.0 })
        .collect();
    let left: Vec<usize> = (0..25).collect();
    let a = vra_alignment(&mvap, &left);
    // left has tracts 0..25: even-indexed = 0,2,4,...,24 → 13 tracts × 100.0 = 1300
    // odd-indexed in left  = 1,3,5,...,23 → 12 tracts × 0.0 = 0
    // total mvap = 25 × 100.0 = 2500
    // mvap_left = 1300, mvap_frac = 1300/2500 = 0.52
    // alignment = |0.52 - 0.5| * 2 = 0.04
    assert!(
        a >= 0.0 && a <= 1.0,
        "alignment must be in [0,1] for 50-tract test, got {a}"
    );
    assert!(
        (a - 0.04).abs() < 1e-9,
        "expected alignment ~0.04, got {a:.6}"
    );
}

#[test]
fn vra_score_symmetric_around_half() {
    // Alignment is symmetric: A(left, right) == A(right, left).
    // i.e. swapping the two sides does not change the score.
    let mvap = vec![80.0, 20.0, 10.0, 90.0];
    let left_indices = vec![0, 1]; // mvap_left  = 100
    let right_indices = vec![2, 3]; // mvap_right = 100
    let a_fwd = vra_alignment(&mvap, &left_indices);
    let a_rev = vra_alignment(&mvap, &right_indices);
    assert!(
        (a_fwd - a_rev).abs() < 1e-9,
        "alignment must be symmetric: fwd={a_fwd:.6} rev={a_rev:.6}"
    );
}


#[test]
fn geosection_tuning_preserves_defaults_and_local_ratio_ranking() {
    let (adj, pop) = small_grid(10, 10);
    let edges: HashMap<(usize,usize),f64> = adj.iter().enumerate().flat_map(|(u,ns)|ns.iter().filter(move |&&v|v>u).map(move |&v|((u,v),(1+u%5) as f64))).collect();
    let centers = crate::geosection_orientation::CentroidMap::new();
    let tracts: HashSet<usize> = (0..pop.len()).collect();
    for k in [2, 3, 4, 6] {
        let legacy = run_geosection_seeded(&adj,&pop,&edges,k,0.1,10,2,None,&centers,0.0,None,1.1,None,0.0,None,u64::MAX).unwrap();
        let defaults = run_geosection_seeded_tuned(&adj,&pop,&edges,k,0.1,10,2,None,&centers,0.0,None,1.1,None,0.0,None,u64::MAX,"cut",1).unwrap();
        assert_eq!(legacy.0,defaults.0);
        assert_eq!((legacy.1,legacy.2),(defaults.1,defaults.2));
        for objective in ["cut","volume"] {
            for trials in [1,3] {
                let actual = run_geosection_seeded_tuned(&adj,&pop,&edges,k,0.1,10,2,None,&centers,0.0,None,1.1,None,0.0,None,u64::MAX,objective,trials).unwrap();
                let mut best = (f64::INFINITY,0usize);
                for left_k in 1..=k/2 {
                    let left = left_k as f32/k as f32;
                    let targets = if left_k==k-left_k {None} else {Some(vec![left,1.0-left])};
                    let mut cut = f64::INFINITY;
                    for index in 0..2 {
                        let (l,_) = split_subgraph_tuned(&adj,&pop,1,&edges,&tracts,1.0+0.1/k as f64,10,Some(u64::MAX.wrapping_add(index)),targets.clone(),None,objective,trials).unwrap();
                        cut=cut.min(edges.iter().filter(|((u,v),_)|l.contains(u)!=l.contains(v)).map(|(_,w)|w).sum());
                    }
                    let score=cut/(left_k as f64).sqrt();
                    if score<best.0 {best=(score,left_k);}
                }
                assert_eq!(actual.1,best.1,"ratio selection uses local original weighted cuts");
                assert!((actual.3/(actual.1 as f64).sqrt()-best.0).abs()<1e-8);
            }
        }
    }
    for (objective,trials) in [("invalid",1),("cut",0),("volume",101)] {
        assert!(run_geosection_seeded_tuned(&adj,&pop,&edges,4,0.1,10,1,None,&centers,0.0,None,1.1,None,0.0,None,42,objective,trials).is_err());
    }
}
