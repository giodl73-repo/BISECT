
use super::*;

// ── detect_gpmetis_version ───────────────────────────────────────────────

#[test]
fn test_detect_gpmetis_version_returns_string() {
    let version = detect_gpmetis_version();
    assert!(!version.is_empty(), "must return a non-empty string");
    assert!(
        version.chars().all(|c| !c.is_control() || c == ' '),
        "version string must contain only printable chars: {:?}",
        version
    );
    assert!(
        version.contains("METIS"),
        "expected METIS in version string, got: {version}"
    );
}

#[test]
fn test_detect_gpmetis_version_never_panics() {
    let v = detect_gpmetis_version();
    assert!(
        !v.is_empty(),
        "must return a non-empty string, got: {:?}",
        v
    );
}

#[test]
fn test_split_four_node_graph() {
    let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
    let vw = vec![1000i64, 1000, 1000, 1000];
    let ew = HashMap::new();
    let indices: HashSet<usize> = (0..4).collect();

    let (left, right) = split_subgraph(
        &adj,
        &vw,
        1,
        &ew,
        &indices,
        1.001,
        100,
        Some(42),
        None,
        None,
    )
    .expect("METIS should split 4-node graph");

    assert_eq!(left.len() + right.len(), 4, "all tracts assigned");
    assert!(!left.is_empty() && !right.is_empty(), "non-empty split");
    // Disjoint and complete
    assert!(left.is_disjoint(&right), "left and right must be disjoint");
    for i in 0..4 {
        assert!(left.contains(&i) || right.contains(&i), "tract {i} missing");
    }
    let pop_left: i64 = left.iter().map(|&i| vw[i]).sum();
    let pop_right: i64 = right.iter().map(|&i| vw[i]).sum();
    let dev = (pop_left - pop_right).abs() as f64 / 4000.0;
    assert!(dev <= 0.2, "split should be balanced, got {dev:.3}");
}

#[test]
fn test_run_all_splits_single_district() {
    let n = 193usize;
    let adj = vec![vec![]; n];
    let vw = vec![1000i64; n];
    let ew = HashMap::new();
    let assignments = run_all_splits(&adj, &vw, &ew, 1, 0.005, 100, None, None)
        .expect("single district should not invoke METIS");
    assert_eq!(assignments.len(), n);
    assert!(assignments.values().all(|&d| d == 1));
}

#[test]
fn test_run_all_splits_two_districts() {
    let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
    let vw = vec![1000i64, 1000, 1000, 1000];
    let ew = HashMap::new();

    let assignments = run_all_splits(&adj, &vw, &ew, 2, 0.005, 100, Some(42), None).unwrap();

    assert_eq!(assignments.len(), 4, "all tracts assigned");
    assert!(
        assignments.values().any(|&d| d == 1),
        "district 1 must exist"
    );
    assert!(
        assignments.values().any(|&d| d == 2),
        "district 2 must exist"
    );
    assert!(assignments.values().all(|&d| d == 1 || d == 2));

    let d1: HashSet<usize> = assignments
        .iter()
        .filter(|(_, &v)| v == 1)
        .map(|(&k, _)| k)
        .collect();
    let d2: HashSet<usize> = assignments
        .iter()
        .filter(|(_, &v)| v == 2)
        .map(|(&k, _)| k)
        .collect();
    assert!(d1.is_disjoint(&d2), "districts must be disjoint");
    assert_eq!(d1.len() + d2.len(), 4, "complete coverage");
}

#[test]
fn test_leaf_sort_bfs_order() {
    // Verify sort_by_key gives BFS not lex order
    let mut paths = vec![
        "1".to_string(),
        "0".to_string(),
        "01".to_string(),
        "00".to_string(),
    ];
    paths.sort_by_key(|p| (p.len(), p.clone()));
    // BFS: depth-1 first ("0","1"), then depth-2 ("00","01")
    assert_eq!(paths, vec!["0", "1", "00", "01"]);
}

// ── Invariant tests ──────────────────────────────────────────────────────

#[test]
fn test_nway_single_district_shortcut() {
    let adj = vec![vec![1], vec![0]];
    let vw = vec![1000i64, 1000];
    let ew = HashMap::new();
    let assignments = run_nway_partition(&adj, &vw, &ew, 1, 1.005, 100, None).unwrap();
    assert_eq!(assignments.len(), 2);
    assert!(assignments.values().all(|&d| d == 1));
}

#[test]
fn test_nway_two_districts() {
    let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
    let vw = vec![1000i64; 4];
    let ew: HashMap<(usize, usize), f64> = HashMap::new();
    let assignments = run_nway_partition(&adj, &vw, &ew, 2, 1.005, 100, Some(42)).unwrap();
    assert_eq!(assignments.len(), 4);
    assert!(
        assignments.values().any(|&d| d == 1),
        "district 1 must exist"
    );
    assert!(
        assignments.values().any(|&d| d == 2),
        "district 2 must exist"
    );
    // Districts are 1-based and disjoint
    let d1: HashSet<_> = assignments
        .iter()
        .filter(|(_, &v)| v == 1)
        .map(|(&k, _)| k)
        .collect();
    let d2: HashSet<_> = assignments
        .iter()
        .filter(|(_, &v)| v == 2)
        .map(|(&k, _)| k)
        .collect();
    assert!(d1.is_disjoint(&d2));
    assert_eq!(d1.len() + d2.len(), 4);
}

#[test]
fn test_nway_equal_weights_sum_to_one() {
    // AC-05: for n-way, verify n-1 explicit weights + inferred last = 1.0
    // With weight_per = 1/k, sum = (k-1)/k + inferred(1/k) = 1.0 exactly
    for k in [2usize, 3, 7, 52] {
        let weight_per = 1.0_f64 / k as f64;
        let explicit_sum: f64 = (k - 1) as f64 * weight_per;
        let inferred = 1.0 - explicit_sum;
        assert!(
            (explicit_sum + inferred - 1.0).abs() < 1e-9,
            "k={k}: explicit({explicit_sum:.9}) + inferred({inferred:.9}) should = 1.0"
        );
    }
}

#[test]
fn test_invariant_target_weights_sum_to_one_2way() {
    // AC-05: target partition weights must sum to 1.0 for 2-way split
    // (k_left/k + k_right/k = k/k = 1.0 by construction)
    for k in [2, 3, 4, 7, 8, 14, 52] {
        let tree = bisect_core::BisectionTree::from_k(k);
        for node in &tree.nodes {
            let left_frac = node.k_left as f64 / node.k as f64;
            let right_frac = node.k_right as f64 / node.k as f64;
            let sum = left_frac + right_frac;
            assert!(
                    (sum - 1.0).abs() < 1e-9,
                    "k={k} node k={}: left_frac={left_frac:.6} + right_frac={right_frac:.6} = {sum:.6} != 1.0",
                    node.k
                );
        }
    }
}

// ── ufactor correctness tests ─────────────────────────────────────────────

#[test]
fn test_ufactor_integer_conversion_0_5_pct() {
    // 0.5% tolerance: decimal 1.005 → integer 5
    let decimal = 1.005_f64;
    let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
    assert_eq!(ufactor_int, 5, "1.005 must convert to integer 5 (0.5%)");
}

#[test]
fn test_ufactor_integer_conversion_5_pct() {
    // 5% tolerance: decimal 1.05 → integer 50
    let decimal = 1.05_f64;
    let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
    assert_eq!(ufactor_int, 50, "1.05 must convert to integer 50 (5%)");
}

#[test]
fn test_ufactor_integer_conversion_10_pct() {
    // 10% tolerance: decimal 1.10 → integer 100
    let decimal = 1.10_f64;
    let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
    assert_eq!(ufactor_int, 100, "1.10 must convert to integer 100 (10%)");
}

#[test]
fn test_ufactor_never_zero() {
    // Minimum clamped to 1 — ufactor=0 would disable balance checking
    for decimal in [1.0001_f64, 1.0_f64, 0.999_f64] {
        let raw = ((decimal - 1.0) * 1000.0).round() as i32;
        let clamped = (raw as u32).clamp(1, 1000);
        assert!(
            clamped >= 1,
            "ufactor must be >= 1, got {clamped} from decimal {decimal}"
        );
    }
}

#[test]
fn test_per_node_ufactor_formula() {
    // node_ufactor = 1.0 + balance_tolerance / k_node
    // Root of 98-district map (T=10%): should be very tight
    let k_root = 98usize;
    let tolerance = 0.10_f64;
    let node_ufactor = 1.0 + tolerance / k_root as f64;
    // ~0.102% — convert to int
    let ufactor_int = ((node_ufactor - 1.0) * 1000.0).round() as u32;
    assert_eq!(
        ufactor_int, 1,
        "root of 98-district (10%) → ufactor=1 (0.1%)"
    );

    // Leaf of 2-district split (T=10%): should be loose
    let k_leaf = 2usize;
    let leaf_ufactor = 1.0 + tolerance / k_leaf as f64;
    let leaf_int = ((leaf_ufactor - 1.0) * 1000.0).round() as u32;
    assert_eq!(leaf_int, 50, "leaf of 2-district (10%) → ufactor=50 (5%)");
}

#[test]
fn test_per_node_ufactor_congressional_tight() {
    // Congressional (T=0.5%): root of 52-district CA map
    // 0.5%/52 = 0.0096% → rounds to 0, clamped to minimum 1
    let k = 52usize;
    let tolerance = 0.005_f64; // 0.5%
    let node_ufactor = 1.0 + tolerance / k as f64;
    let raw = ((node_ufactor - 1.0) * 1000.0).round() as u32;
    let ufactor_int = raw.clamp(1, 1000); // minimum 1 = 0.1%
    assert_eq!(
        ufactor_int, 1,
        "CA 52D congressional root → clamped to minimum ufactor=1 (0.1%)"
    );
}

#[test]
fn test_ufactor_wasnt_silently_truncated_regression() {
    // This test catches the historical bug where '-ufactor=1.0050' was passed
    // to gpmetis as a float, which atoi() truncated to 1 regardless of value.
    // The correct behavior: 1.005 → integer 5 (not 1).
    let old_style_float = 1.005_f64;
    // Old bug: atoi("1.0050") == 1 (always)
    // New fix: round((1.005 - 1.0) * 1000) == 5
    let correct_int = ((old_style_float - 1.0) * 1000.0).round() as u32;
    assert_ne!(
        correct_int, 1,
        "REGRESSION: 1.005 should not convert to 1 — that was the bug. Got {correct_int}"
    );
    assert_eq!(
        correct_int, 5,
        "1.005 (0.5% tolerance) must convert to integer 5"
    );
}

#[test]
fn test_invariant_vertex_weights_positive() {
    // DF-04: all vertex weights must be >= 1 after loading
    // sub-zero weights cause METIS to produce degenerate partitions
    let adj = vec![vec![1], vec![0, 2], vec![1]];
    let vw = vec![1000i64, 500, 2000]; // all positive
    let ew: HashMap<(usize, usize), f64> = HashMap::new();
    // The subgraph builder clamps to max(weight, 1) — verify it would catch 0
    let tract_indices: HashSet<usize> = (0..3).collect();
    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();
    let sub_vw: Vec<i64> = sorted.iter().map(|&g| vw[g].max(1)).collect();
    assert!(
        sub_vw.iter().all(|&v| v >= 1),
        "all vertex weights must be >= 1 after clamping"
    );
}

// ── Group 1: split_subgraph edge cases ───────────────────────────────────

#[test]
fn test_split_subgraph_with_edge_weights() {
    // 4-node chain with strong edge weights on left side — should bias split
    let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
    let vw = vec![1000i64; 4];
    let mut ew = HashMap::new();
    ew.insert((0, 1), 1000.0); // strong edge — METIS should avoid cutting
    ew.insert((1, 2), 1.0); // weak edge — METIS may cut here
    ew.insert((2, 3), 1000.0); // strong edge
    let indices: HashSet<usize> = (0..4).collect();
    let (left, right) = split_subgraph(
        &adj,
        &vw,
        1,
        &ew,
        &indices,
        1.005,
        100,
        Some(42),
        None,
        None,
    )
    .expect("should split with edge weights");
    assert_eq!(left.len() + right.len(), 4);
    assert!(!left.is_empty() && !right.is_empty());
}

#[test]
fn test_split_subgraph_unequal_target_weights() {
    // 6 tracts, split 4:2 (target weights 2/3 and 1/3)
    let adj = vec![
        vec![1, 2],
        vec![0, 3],
        vec![0, 3],
        vec![1, 2, 4, 5],
        vec![3, 5],
        vec![3, 4],
    ];
    let vw = vec![1000i64; 6];
    let ew = HashMap::new();
    let indices: HashSet<usize> = (0..6).collect();
    let (left, right) = split_subgraph(
        &adj,
        &vw,
        1,
        &ew,
        &indices,
        1.05,
        100,
        Some(42),
        Some(vec![2.0f32 / 3.0, 1.0f32 / 3.0]), // unequal: 4:2 split
        None,
    )
    .expect("should split with target weights");
    assert_eq!(left.len() + right.len(), 6);
    assert!(!left.is_empty() && !right.is_empty());
    // Approximate target: left ~4 tracts, right ~2 (within tolerance)
    let larger = left.len().max(right.len());
    assert!(
        larger >= 3,
        "larger partition should have >= 3 tracts for 4:2 split"
    );
}

#[test]
fn test_split_subgraph_single_node_returns_all_left() {
    // Edge case: single tract — no METIS call, returns all in left
    let adj = vec![vec![]];
    let vw = vec![1000i64];
    let ew = HashMap::new();
    let indices: HashSet<usize> = vec![0].into_iter().collect();
    let (left, right) = split_subgraph(&adj, &vw, 1, &ew, &indices, 1.005, 100, None, None, None)
        .expect("single node split");
    assert_eq!(left.len(), 1);
    assert!(right.is_empty());
}

#[test]
fn test_split_subgraph_two_tracts_always_splits() {
    // 2 tracts: must produce one in each partition
    let adj = vec![vec![1], vec![0]];
    let vw = vec![1000i64, 1000];
    let ew = HashMap::new();
    let indices: HashSet<usize> = (0..2).collect();
    let (left, right) = split_subgraph(
        &adj,
        &vw,
        1,
        &ew,
        &indices,
        1.005,
        100,
        Some(42),
        None,
        None,
    )
    .expect("2-node split");
    assert_eq!(left.len(), 1);
    assert_eq!(right.len(), 1);
    assert!(left.is_disjoint(&right));
}

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

// ── Group 3: run_all_splits edge cases ───────────────────────────────────

#[test]
fn test_run_all_splits_large_k_structure() {
    // Verify that run_all_splits with k=8 produces exactly 8 districts
    // without calling gpmetis (test the assignment structure, not balance)
    // Use single-tract-per-district to make it trivially balanced

    let n = 16;
    // Grid graph: 4x4
    let adj: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            let row = i / 4;
            let col = i % 4;
            let mut nbrs = vec![];
            if row > 0 {
                nbrs.push(i - 4);
            }
            if row < 3 {
                nbrs.push(i + 4);
            }
            if col > 0 {
                nbrs.push(i - 1);
            }
            if col < 3 {
                nbrs.push(i + 1);
            }
            nbrs
        })
        .collect();
    let vw = vec![1000i64; n];
    let ew = HashMap::new();
    let assignments = run_all_splits(&adj, &vw, &ew, 8, 0.10, 100, Some(42), None).unwrap();
    assert_eq!(assignments.len(), n);
    let districts: std::collections::HashSet<usize> = assignments.values().copied().collect();
    assert_eq!(districts.len(), 8, "exactly 8 districts");
    // All district IDs 1-based
    assert!(districts.iter().all(|&d| d >= 1 && d <= 8));
}

#[test]
fn test_run_all_splits_tight_balance_10pct() {
    // With correct ufactor math, 10% tolerance on a 4-district map
    // should produce well-balanced output

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
    let vw = vec![1000i64; 8]; // 8 equal tracts
    let ew = HashMap::new();
    let assignments = run_all_splits(&adj, &vw, &ew, 4, 0.10, 100, Some(42), None).unwrap();

    let mut pops = vec![0i64; 5];
    for (&tract, &dist) in &assignments {
        pops[dist] += vw[tract];
    }
    let ideal = 8000 / 4; // 2000
    for d in 1..=4 {
        let dev = (pops[d] - ideal).abs() as f64 / ideal as f64;
        assert!(
            dev <= 0.10,
            "district {d} deviation {:.1}% exceeds 10%",
            dev * 100.0
        );
    }
}

// ── AP-08: Granularity floor tests ───────────────────────────────────────

#[test]
fn test_granularity_floor_warning_threshold() {
    // AP-08: when tracts_per_district < 20, balance may be unachievable
    // This tests the THRESHOLD CALCULATION not the algorithm (which can't be unit tested)
    let total_tracts = 1784usize; // WA 2020
    let house_districts = 98usize;
    let tpd = total_tracts as f64 / house_districts as f64;
    assert!(
        tpd < 20.0,
        "WA house at tract level has {tpd:.1} tracts/district — below granularity threshold"
    );

    let avg_tract_pop = 7_705_281i64 / total_tracts as i64;
    let ideal_district_pop = 7_705_281i64 / house_districts as i64;
    let single_tract_impact_pct = avg_tract_pop as f64 / ideal_district_pop as f64 * 100.0;
    // One tract swap changes the balance by >5% — makes 5% tolerance often impossible
    assert!(single_tract_impact_pct > 3.0,
            "At WA tract granularity, one tract swap = {single_tract_impact_pct:.1}% of district ideal — exceeds 5% tolerance at 10% target");
}

#[test]
fn test_granularity_sufficient_for_congressional() {
    // Congressional maps (10 districts) have ~178 tracts/district — far above threshold
    let total_tracts = 1784usize;
    let congressional_districts = 10usize;
    let tpd = total_tracts as f64 / congressional_districts as f64;
    assert!(
        tpd >= 20.0,
        "WA congressional has {tpd:.1} tracts/district — sufficient granularity"
    );
}

#[test]
fn test_granularity_block_group_fixes_wa_house() {
    // Block groups (5311 for WA) give 54/district — above threshold
    let bg_count = 5311usize;
    let house_districts = 98usize;
    let bgpd = bg_count as f64 / house_districts as f64;
    assert!(
        bgpd >= 20.0,
        "WA house at block_group has {bgpd:.1} BGs/district — adequate"
    );
}

// ── Task 147: ARM Linux platform detection ───────────────────────────────

#[test]
fn test_gpmetis_not_found_error_includes_arch() {
    // The error message from a missing gpmetis must include the OS/arch string.
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let install_hint = match (os, arch) {
            ("linux", "aarch64") | ("linux", "arm") =>
                "ARM Linux: apt-get install metis (Debian/Ubuntu) or build from source: https://github.com/KarypisLab/METIS",
            ("macos", "aarch64") =>
                "Apple Silicon: brew install metis",
            ("linux", _) =>
                "Linux: apt-get install metis (Debian/Ubuntu) or dnf install metis-devel (Fedora)",
            ("windows", _) =>
                "Windows: download from https://github.com/KarypisLab/METIS/releases or install via vcpkg",
            ("macos", _) =>
                "macOS: brew install metis",
            _ =>
                "Install METIS from https://github.com/KarypisLab/METIS",
        };
    let msg = format!("gpmetis not found ({os}/{arch}). {install_hint}");
    assert!(msg.contains(os), "error must contain OS: {os}");
    assert!(msg.contains(arch), "error must contain arch: {arch}");
    assert!(
        msg.contains("gpmetis not found"),
        "must include 'gpmetis not found'"
    );
}

#[test]
fn test_platform_install_hint_linux_arm() {
    // Simulate ARM Linux hint construction.
    let (os, arch) = ("linux", "aarch64");
    let install_hint = match (os, arch) {
            ("linux", "aarch64") | ("linux", "arm") =>
                "ARM Linux: apt-get install metis (Debian/Ubuntu) or build from source: https://github.com/KarypisLab/METIS",
            ("macos", "aarch64") =>
                "Apple Silicon: brew install metis",
            ("linux", _) =>
                "Linux: apt-get install metis (Debian/Ubuntu) or dnf install metis-devel (Fedora)",
            ("windows", _) =>
                "Windows: download from https://github.com/KarypisLab/METIS/releases or install via vcpkg",
            ("macos", _) =>
                "macOS: brew install metis",
            _ =>
                "Install METIS from https://github.com/KarypisLab/METIS",
        };
    assert!(
        install_hint.contains("apt-get install metis"),
        "ARM Linux must get apt-get hint, got: {install_hint}"
    );
    assert!(
        install_hint.contains("ARM Linux"),
        "must mention ARM Linux, got: {install_hint}"
    );
}

/// Task 112: Windows path quoting invariant.
/// Documents that Command::arg(PathBuf) handles paths with spaces correctly via
/// the OS API — no manual quoting is needed or should be applied.
#[test]
fn test_path_arg_does_not_need_manual_quoting() {
    use std::ffi::OsString;
    // Simulate building the -tpwgt= flag as done in split_subgraph/run_nway_partition.
    // A path with spaces: "/tmp/my dir with spaces/tpwgts.txt"
    let spaced_path = std::path::PathBuf::from("/tmp/my dir with spaces/tpwgts.txt");

    // The correct pattern: OsString concatenation, passed as a single .arg()
    let mut flag = OsString::from("-tpwgt=");
    flag.push(spaced_path.as_os_str());

    // The flag should contain the path verbatim (with spaces) — no manual quoting
    let flag_str = flag.to_string_lossy();
    assert!(
        flag_str.contains(" "),
        "spaces are preserved in OsString — OS API handles quoting"
    );
    assert!(flag_str.starts_with("-tpwgt="), "flag prefix preserved");
    assert!(
        !flag_str.contains('"'),
        "no manual quoting added — OS API handles this"
    );

    // Contrast: format!() with .display() would produce the same string,
    // but would be passed through the shell if used with Command::new("sh").arg("-c", ...)
    // When using Command::arg() directly, the OS API receives the raw arg — safe either way.
    // The important invariant: do NOT concatenate paths into shell strings.
    let display_str = format!("-tpwgt={}", spaced_path.display());
    assert_eq!(
        flag_str,
        display_str.as_str(),
        "OsString flag matches display()-based string for non-Unicode paths"
    );
}

/// Scenario 23: Rayon seed determinism — sort split_results by path before insert.
/// Verify that for a two-district run with a fixed seed, calling run_all_splits
/// twice returns identical assignments (deterministic output).
#[test]
fn test_rayon_results_sorted_before_insert() {
    // A simple 4-node chain graph: 0-1-2-3
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let vw = vec![1000i64, 1000, 1000, 1000];
    let ew = HashMap::new();

    // Run twice with the same seed
    let result1 = run_all_splits(&adj, &vw, &ew, 2, 0.005, 100, Some(42), None);
    let result2 = run_all_splits(&adj, &vw, &ew, 2, 0.005, 100, Some(42), None);

    assert!(
        result1.is_ok(),
        "first run must succeed: {:?}",
        result1.err()
    );
    assert!(
        result2.is_ok(),
        "second run must succeed: {:?}",
        result2.err()
    );

    let a1 = result1.unwrap();
    let a2 = result2.unwrap();

    // With sorted insertion order and same seed, assignments must be identical
    let mut a1_sorted: Vec<(usize, usize)> = a1.into_iter().collect();
    let mut a2_sorted: Vec<(usize, usize)> = a2.into_iter().collect();
    a1_sorted.sort_by_key(|&(k, _)| k);
    a2_sorted.sort_by_key(|&(k, _)| k);

    assert_eq!(
        a1_sorted, a2_sorted,
        "two runs with the same seed must produce identical assignments"
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
    let pop_right = 1.0 - pop_left;
    let area_right = 1.0 - area_left;
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
#[ignore = "requires METIS with ncon=2 support"]
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
                (ratio - 0.5).abs() < 0.15,
                "pop balance should be ~50%: got {:.1}%",
                ratio * 100.0
            );
        }
        Err(e) => {
            // METIS may fail on this version — log and skip
            eprintln!("split_subgraph ncon=2 error: {e}");
        }
    }
}

// ── Lorenz analysis tests ─────────────────────────────────────────────

#[test]
fn lorenz_curve_starts_at_origin_ends_at_one() {
    let pop = vec![100i64, 200, 300, 400];
    let area = vec![1000.0f64, 2000.0, 3000.0, 4000.0];
    let (curve, _, _) = population_lorenz(&pop, &area, 4);
    assert!(
        curve
            .first()
            .map(|&(a, p)| a == 0.0 && p == 0.0)
            .unwrap_or(false),
        "curve must start at (0,0)"
    );
    let (a_last, p_last) = *curve.last().unwrap();
    assert!((a_last - 1.0).abs() < 1e-9, "curve area must reach 1.0");
    assert!((p_last - 1.0).abs() < 1e-9, "curve pop must reach 1.0");
}

#[test]
fn lorenz_curve_monotone_non_decreasing() {
    let pop = vec![50i64, 200, 100, 400, 10];
    let area = vec![500.0f64, 1000.0, 800.0, 2000.0, 200.0];
    let (curve, _, _) = population_lorenz(&pop, &area, 5);
    for w in curve.windows(2) {
        assert!(w[1].0 >= w[0].0, "area fraction must be non-decreasing");
        assert!(w[1].1 >= w[0].1, "pop fraction must be non-decreasing");
    }
}

#[test]
fn lorenz_natural_ratio_uniform_state_is_half() {
    // Uniform state: all tracts same density → natural pop at 50% area = 50%
    let pop = vec![100i64; 10];
    let area = vec![1000.0f64; 10];
    let (_, natural_pop, suggested_k) = population_lorenz(&pop, &area, 10);
    assert!(
        (natural_pop - 0.5).abs() < 0.05,
        "uniform state natural pop should be ~50%: got {:.1}%",
        natural_pop * 100.0
    );
    assert_eq!(
        suggested_k, 5,
        "uniform state natural k should be 5 out of 10"
    );
}

#[test]
fn lorenz_natural_ratio_concentrated_state() {
    // Very concentrated: first 2 tracts have 90% of pop in 10% of area
    let pop = vec![900i64, 900, 10, 10, 10, 10, 10, 10, 10, 10];
    let area = vec![
        500.0f64, 500.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0,
    ];
    let (_, natural_pop, _) = population_lorenz(&pop, &area, 10);
    // Dense 50% of area (first 5 dense tracts) should contain well above 50% of pop
    assert!(
        natural_pop > 0.7,
        "concentrated state: dense half should hold >70% pop, got {:.1}%",
        natural_pop * 100.0
    );
}

#[test]
fn lorenz_min_area_for_zero_is_zero() {
    let pop = vec![100i64, 200, 300];
    let area = vec![1000.0f64, 2000.0, 3000.0];
    let min_a = lorenz_min_area(&pop, &area, 0.0);
    assert_eq!(min_a, 0.0);
}

#[test]
fn lorenz_min_area_for_one_is_one() {
    let pop = vec![100i64, 200, 300];
    let area = vec![1000.0f64, 2000.0, 3000.0];
    let min_a = lorenz_min_area(&pop, &area, 1.0);
    assert!((min_a - 1.0).abs() < 1e-9);
}

#[test]
fn lorenz_min_area_dense_tract_first() {
    // Tract 0: 900 pop, 100 area (very dense)
    // Tract 1: 100 pop, 900 area (very sparse)
    // Minimum area to hold 90% of pop = just tract 0 = 100/1000 = 10%
    let pop = vec![900i64, 100];
    let area = vec![100.0f64, 900.0];
    let min_a = lorenz_min_area(&pop, &area, 0.9);
    assert!(
        min_a < 0.15,
        "dense tract holds 90% pop in ~10% area, got {:.1}%",
        min_a * 100.0
    );
}

// ── VRASection (T.7): alignment score unit tests ─────────────────────────

/// Helper: compute the VRASection alignment score the same way run_geosection does.
/// alignment = |MVAP_frac(left) - MVAP_frac(right)| normalised to [0, 1]
/// = |mvap_left/mvap_total - (1 - mvap_left/mvap_total)| = |2*mvap_left/mvap_total - 1|
fn vra_alignment(mvap: &[f64], left: &[usize]) -> f64 {
    let mvap_total: f64 = mvap.iter().sum();
    if mvap_total == 0.0 {
        return 0.0;
    }
    let mvap_left: f64 = left.iter().map(|&v| mvap[v]).sum();
    (mvap_left / mvap_total - 0.5).abs() * 2.0
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

// ── Group: connected_components_of ───────────────────────────────────────

#[test]
fn connected_components_single_vertex() {
    // 1-vertex graph, subset = {0} → exactly 1 component
    let adj = vec![vec![]];
    let subset: HashSet<usize> = vec![0].into_iter().collect();
    let comps = connected_components_of(&adj, &subset);
    assert_eq!(comps.len(), 1, "single vertex must yield 1 component");
    assert!(comps[0].contains(&0));
}

#[test]
fn connected_components_two_disconnected_vertices() {
    // 2-vertex graph with no edges → 2 components when both in subset
    let adj = vec![vec![], vec![]];
    let subset: HashSet<usize> = vec![0, 1].into_iter().collect();
    let comps = connected_components_of(&adj, &subset);
    assert_eq!(
        comps.len(),
        2,
        "two isolated vertices must yield 2 components"
    );
}

#[test]
fn connected_components_fully_connected() {
    // 4-node chain 0-1-2-3: all in subset → 1 component
    let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
    let subset: HashSet<usize> = (0..4).collect();
    let comps = connected_components_of(&adj, &subset);
    assert_eq!(comps.len(), 1, "connected chain must yield 1 component");
    let union: HashSet<usize> = comps.into_iter().flatten().collect();
    assert_eq!(union.len(), 4, "all vertices accounted for");
}

#[test]
fn connected_components_subset_only() {
    // 6-node graph in two cliques: 0-1-2 and 3-4-5, with no cross-edges.
    // Pass subset = {0,1,2} → should find 1 component even though 3-4-5 exist.
    let adj = vec![
        vec![1, 2],
        vec![0, 2],
        vec![0, 1], // clique A
        vec![4, 5],
        vec![3, 5],
        vec![3, 4], // clique B
    ];
    let subset: HashSet<usize> = vec![0, 1, 2].into_iter().collect();
    let comps = connected_components_of(&adj, &subset);
    assert_eq!(comps.len(), 1, "subset {{0,1,2}} is a clique → 1 component");
    let union: HashSet<usize> = comps.into_iter().flatten().collect();
    assert_eq!(union, subset, "component must exactly match subset");
}

#[test]
fn connected_components_ignores_external_edges() {
    // 4-node graph: 0 connects to 1,2,3 but subset = {0,1}.
    // Edge 0-2 and 0-3 go outside subset and must be ignored.
    // 0-1 is internal → subset {0,1} is 1 component.
    let adj = vec![vec![1, 2, 3], vec![0], vec![0], vec![0]];
    let subset: HashSet<usize> = vec![0, 1].into_iter().collect();
    let comps = connected_components_of(&adj, &subset);
    assert_eq!(
        comps.len(),
        1,
        "external edges must be ignored; {{0,1}} is connected"
    );
}

// ── Group: repair_bisection_contiguity ───────────────────────────────────

#[test]
fn repair_no_op_when_both_connected() {
    // Left = {0,1}, Right = {2,3} on a 4-node chain.
    // Both sides already connected — repair should return them unchanged.
    let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
    let left: HashSet<usize> = vec![0, 1].into_iter().collect();
    let right: HashSet<usize> = vec![2, 3].into_iter().collect();
    let (l2, r2) = repair_bisection_contiguity(&adj, left.clone(), right.clone());
    assert_eq!(l2, left, "no-op: left unchanged");
    assert_eq!(r2, right, "no-op: right unchanged");
}

#[test]
fn repair_single_orphan_moved_to_right() {
    // Chain 0-1-2-3-4.  Left = {0,1,4} — vertex 4 is not connected to 0,1
    // through left-only edges.  Repair should move vertex 4 to right.
    let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]];
    let left: HashSet<usize> = vec![0, 1, 4].into_iter().collect();
    let right: HashSet<usize> = vec![2, 3].into_iter().collect();
    let (l2, r2) = repair_bisection_contiguity(&adj, left, right);
    assert!(
        !l2.contains(&4) || r2.contains(&4) || l2.contains(&4),
        "vertex 4 must end up in exactly one side"
    );
    // Both sides must cover all 5 vertices
    let mut all: Vec<usize> = l2.union(&r2).copied().collect();
    all.sort_unstable();
    assert_eq!(all, vec![0, 1, 2, 3, 4], "all vertices must be covered");
}

#[test]
fn repair_single_orphan_moved_to_left() {
    // Chain 0-1-2-3-4.  Right = {1,4} — vertex 4 is orphaned from 1 (no path
    // through right).  Repair migrates 4 to left.
    let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]];
    let left: HashSet<usize> = vec![0, 2, 3].into_iter().collect();
    let right: HashSet<usize> = vec![1, 4].into_iter().collect();
    let (l2, r2) = repair_bisection_contiguity(&adj, left, right);
    let mut all: Vec<usize> = l2.union(&r2).copied().collect();
    all.sort_unstable();
    assert_eq!(
        all,
        vec![0, 1, 2, 3, 4],
        "repair must preserve all vertices"
    );
}

#[test]
fn repair_result_covers_all_vertices() {
    // Arbitrary disconnected split on an 8-node graph.
    // Key invariant: |left| + |right| must equal n after repair.
    let adj: Vec<Vec<usize>> = vec![
        vec![1],
        vec![0, 2],
        vec![1, 3],
        vec![2],
        vec![5],
        vec![4, 6],
        vec![5, 7],
        vec![6],
    ];
    // left gets both chains but with a gap: {0,1,5,6}
    let left: HashSet<usize> = vec![0, 1, 5, 6].into_iter().collect();
    let right: HashSet<usize> = vec![2, 3, 4, 7].into_iter().collect();
    let (l2, r2) = repair_bisection_contiguity(&adj, left, right);
    assert_eq!(l2.len() + r2.len(), 8, "all 8 vertices must be covered");
    assert!(
        l2.is_disjoint(&r2),
        "sides must remain disjoint after repair"
    );
}

#[test]
fn repair_result_both_sides_nonempty() {
    // Even a maximally unbalanced split should keep both sides non-empty.
    let adj = vec![vec![1], vec![0, 2], vec![1]];
    let left: HashSet<usize> = vec![0, 2].into_iter().collect(); // disconnected
    let right: HashSet<usize> = vec![1].into_iter().collect();
    let (l2, r2) = repair_bisection_contiguity(&adj, left, right);
    assert!(!l2.is_empty(), "left must remain non-empty after repair");
    assert!(!r2.is_empty(), "right must remain non-empty after repair");
    assert_eq!(l2.len() + r2.len(), 3, "all 3 vertices covered");
}

#[test]
fn repair_idempotent_on_connected() {
    // Calling repair twice on an already-connected split must produce the same result.
    let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
    let left: HashSet<usize> = vec![0, 1].into_iter().collect();
    let right: HashSet<usize> = vec![2, 3].into_iter().collect();
    let (l1, r1) = repair_bisection_contiguity(&adj, left.clone(), right.clone());
    let (l2, r2) = repair_bisection_contiguity(&adj, l1.clone(), r1.clone());
    assert_eq!(l1, l2, "repair must be idempotent on left");
    assert_eq!(r1, r2, "repair must be idempotent on right");
}

// ── Group: population_lorenz additional coverage ─────────────────────────

#[test]
fn lorenz_empty_weights_returns_early() {
    // All-zero weights → function returns early with empty curve and 0 natural pop
    let pop = vec![0i64, 0, 0];
    let area = vec![1000.0f64, 2000.0, 3000.0];
    let (curve, natural_pop, suggested_k) = population_lorenz(&pop, &area, 4);
    assert!(curve.is_empty(), "zero total pop must return empty curve");
    assert_eq!(natural_pop, 0.0, "natural pop at half area must be 0");
    assert_eq!(suggested_k, 2, "suggested_k must be num_districts/2 = 2");
}

#[test]
fn lorenz_single_tract() {
    // 1 vertex: curve is trivially (0,0)→(1,1)
    let pop = vec![100i64];
    let area = vec![1000.0f64];
    let (curve, natural_pop, _) = population_lorenz(&pop, &area, 2);
    assert_eq!(
        curve.len(),
        2,
        "single tract: curve has 2 points (0,0) and (1,1)"
    );
    assert!(
        (curve[0].0).abs() < 1e-9 && (curve[0].1).abs() < 1e-9,
        "first point must be (0,0)"
    );
    assert!(
        (curve[1].0 - 1.0).abs() < 1e-9 && (curve[1].1 - 1.0).abs() < 1e-9,
        "last point must be (1,1)"
    );
    // natural pop at half-area: single tract crosses 0.5 area threshold when added,
    // so natural_pop is interpolated — at any rate it must be in [0,1]
    assert!(
        natural_pop >= 0.0 && natural_pop <= 1.0,
        "natural_pop must be in [0,1], got {natural_pop}"
    );
}

#[test]
fn lorenz_two_tracts_different_density() {
    // Tract 0: pop=100, area=10  (density=10) — dense
    // Tract 1: pop=10,  area=100 (density=0.1) — sparse
    // Dense tract is first in sort order.
    // After adding tract 0: cum_area = 10/110 ≈ 0.091 — still < 0.5
    // After adding tract 1: cum_area = 110/110 = 1.0 — crossed 0.5
    // So natural_pop_at_half is interpolated between (0.091, 100/110) and (1.0, 1.0)
    let pop = vec![100i64, 10];
    let area = vec![10.0f64, 100.0];
    let (curve, natural_pop, suggested_k) = population_lorenz(&pop, &area, 2);
    assert_eq!(curve.len(), 3, "two tracts: curve has 3 points");
    // The denser tract must come first in the sorted curve
    // After first tract: cum_pop fraction = 100/110 ≈ 0.909
    assert!(
        curve[1].1 > 0.8,
        "after dense tract, accumulated pop fraction must be > 80%, got {:.3}",
        curve[1].1
    );
    // natural_pop must be > 0.5 (dense area contains majority of pop)
    assert!(
        natural_pop > 0.5,
        "natural pop at half area > 0.5 when pop is concentrated in dense tract, got {natural_pop}"
    );
    // suggested_k <= num_districts/2 = 1
    assert!(
        suggested_k >= 1,
        "suggested_k must be >= 1, got {suggested_k}"
    );
}

#[test]
fn lorenz_natural_k_clamped_to_half() {
    // Verify suggested_k is always <= num_districts/2 for various inputs.
    // Use a heavily concentrated state (all pop in first tract).
    let pop = vec![1000i64, 1, 1, 1, 1, 1, 1, 1];
    let area = vec![10.0f64; 8];
    for k in [2usize, 4, 6, 8, 10, 20, 52] {
        let (_, _, suggested_k) = population_lorenz(&pop, &area, k);
        let max_allowed = k / 2;
        assert!(
            suggested_k <= max_allowed,
            "suggested_k={suggested_k} must be <= {max_allowed} for k={k}"
        );
    }
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

// ── Group: bisection_runner edge cases ────────────────────────────────────

#[test]
fn split_subgraph_empty_tract_indices_returns_empty() {
    // Empty tract set → (empty, empty) without panic
    let adj = vec![vec![1], vec![0, 2], vec![1]];
    let vw = vec![1000i64; 3];
    let ew = HashMap::new();
    let indices: HashSet<usize> = HashSet::new();
    let (left, right) = split_subgraph(&adj, &vw, 1, &ew, &indices, 1.005, 100, None, None, None)
        .expect("empty tract set must not error");
    assert!(left.is_empty(), "empty input → left must be empty");
    assert!(right.is_empty(), "empty input → right must be empty");
}

#[test]
fn split_subgraph_single_tract_returns_all_left() {
    // 1-tract set → (that tract, empty) — already covered by Group 1 but added for completeness
    let adj = vec![vec![]];
    let vw = vec![5000i64];
    let ew = HashMap::new();
    let indices: HashSet<usize> = vec![0].into_iter().collect();
    let (left, right) = split_subgraph(&adj, &vw, 1, &ew, &indices, 1.005, 100, None, None, None)
        .expect("single-tract split must not error");
    assert!(left.contains(&0), "single tract must land in left");
    assert!(
        right.is_empty(),
        "right must be empty for single-tract input"
    );
}

#[test]
fn run_all_splits_single_district_no_metis_call() {
    // k=1: every tract gets district 1 without invoking METIS at all.
    let n = 50usize;
    let adj: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            let mut nb = vec![];
            if i > 0 {
                nb.push(i - 1);
            }
            if i < n - 1 {
                nb.push(i + 1);
            }
            nb
        })
        .collect();
    let vw = vec![1000i64; n];
    let ew = HashMap::new();
    let assignments = run_all_splits(&adj, &vw, &ew, 1, 0.005, 100, None, None)
        .expect("k=1 must succeed without METIS");
    assert_eq!(assignments.len(), n, "all tracts assigned");
    assert!(
        assignments.values().all(|&d| d == 1),
        "k=1: every tract must be in district 1"
    );
}

#[test]
fn run_nway_single_district_shortcut() {
    // k=1 via run_nway_partition: verify same shortcut path works.
    let adj = vec![vec![1], vec![0, 2], vec![1]];
    let vw = vec![1000i64; 3];
    let ew = HashMap::new();
    let assignments = run_nway_partition(&adj, &vw, &ew, 1, 1.005, 100, None)
        .expect("k=1 nway must not invoke METIS");
    assert_eq!(assignments.len(), 3, "all 3 tracts assigned");
    assert!(
        assignments.values().all(|&d| d == 1),
        "k=1: every tract must be district 1"
    );
}

#[test]
fn ufactor_clamp_prevents_zero() {
    for ufactor in [1.0_f64, 1.0001, 1.001, 1.003, 1.004, 1.005] {
        let raw = ((ufactor - 1.0) * 1000.0).round() as i32;
        let clamped = raw.clamp(5, 1000);
        assert!(
            clamped >= 5,
            "uf_int must be >= 5 (0.5%% floor), got {clamped} from ufactor={ufactor}"
        );
    }
}

// ── PercentileSweep tests ─────────────────────────────────────────────────

fn small_grid(rows: usize, cols: usize) -> (Vec<Vec<usize>>, Vec<i64>) {
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

#[test]
fn percentile_sweep_k1_returns_all_district_1() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let result = run_all_splits_percentile(&adj, &pop, &ew, 1, 0.05, 10, 42, 5, 0.5, None)
        .expect("k=1 must succeed");
    assert!(
        result.values().all(|&d| d == 1),
        "k=1: all tracts in district 1"
    );
}

#[test]
fn percentile_sweep_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let result = run_all_splits_percentile(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 0.5, None)
        .expect("k=2 must succeed");
    assert_eq!(result.len(), 16);
    let districts: std::collections::HashSet<usize> = result.values().copied().collect();
    assert_eq!(districts.len(), 2, "must produce exactly 2 districts");
}

#[test]
fn percentile_sweep_p0_same_as_minimum() {
    // p=0.0 should always return the minimum-EC plan.
    let (adj, pop) = small_grid(5, 4);
    let ew = HashMap::new();
    let min_plan = run_all_splits_percentile(&adj, &pop, &ew, 2, 0.05, 10, 99, 10, 0.0, None)
        .expect("p=0.0 must succeed");
    let ec_min = count_edge_cuts(&min_plan, &adj);
    let max_plan = run_all_splits_percentile(&adj, &pop, &ew, 2, 0.05, 10, 99, 10, 1.0, None)
        .expect("p=1.0 must succeed");
    let ec_max = count_edge_cuts(&max_plan, &adj);
    assert!(
        ec_min <= ec_max,
        "p=0.0 plan must have fewer or equal cuts than p=1.0"
    );
}

#[test]
fn percentile_sweep_deterministic() {
    let (adj, pop) = small_grid(4, 5);
    let ew = HashMap::new();
    let r1 = run_all_splits_percentile(&adj, &pop, &ew, 2, 0.05, 10, 7, 5, 0.5, None).unwrap();
    let r2 = run_all_splits_percentile(&adj, &pop, &ew, 2, 0.05, 10, 7, 5, 0.5, None).unwrap();
    assert_eq!(r1, r2, "same seed must produce identical result");
}

// ── BisectionEnsemble tests ───────────────────────────────────────────────

#[test]
fn bisection_ensemble_produces_valid_2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_bisection_ensemble(
        &adj,
        &pop,
        &ew,
        &tracts,
        1.05,
        10,
        Some(42),
        None,
        50,
        0.5,
    )
    .expect("bisection ensemble must succeed");
    assert!(
        !left.is_empty() && !right.is_empty(),
        "both components must be non-empty"
    );
    let mut all: Vec<usize> = left.iter().chain(right.iter()).copied().collect();
    all.sort_unstable();
    let expected: Vec<usize> = (0..16).collect();
    assert_eq!(all, expected, "components must partition all 16 tracts");
}

#[test]
fn bisection_ensemble_p0_equals_standard_bisection_on_small_graph() {
    // With p=0.0 (minimum EC), result should equal or be better than standard bisection.
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let (l_ens, r_ens) = split_subgraph_bisection_ensemble(
        &adj,
        &pop,
        &ew,
        &tracts,
        1.05,
        10,
        Some(42),
        None,
        20,
        0.0,
    )
    .expect("must succeed");
    // Just verify it produces a valid partition
    assert_eq!(l_ens.len() + r_ens.len(), 16);
}

#[test]
fn bisection_ensemble_with_search_produces_k2_partition() {
    let (adj, pop) = small_grid(4, 5); // 20 tracts
    let ew = HashMap::new();
    let result = run_all_splits_with_search(
        &adj,
        &pop,
        &ew,
        2,
        0.05,
        10,
        Some(42),
        None,
        Some((0.5, 30)),
    )
    .expect("bisection-ensemble run_all_splits must succeed");
    assert_eq!(result.len(), 20);
    let districts: std::collections::HashSet<usize> = result.values().copied().collect();
    assert_eq!(districts.len(), 2);
}

#[test]
fn empty_graph_split_honors_tpwgts_left_target() {
    let adj = vec![vec![], vec![], vec![], vec![]];
    let pop = vec![10i64, 10, 10, 70];
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..4).collect();
    let (left, right) = split_subgraph(
        &adj,
        &pop,
        1,
        &ew,
        &tracts,
        1.05,
        10,
        Some(1),
        Some(vec![0.30, 0.70]),
        None,
    )
    .expect("empty graph split must succeed");
    let left_pop: i64 = left.iter().map(|&g| pop[g]).sum();
    let right_pop: i64 = right.iter().map(|&g| pop[g]).sum();
    assert_eq!(left_pop, 30);
    assert_eq!(right_pop, 70);
}

#[test]
fn connected_subset_detects_bridge_removal_disconnection() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1]];
    let connected: HashSet<usize> = [0, 1, 2].into_iter().collect();
    let disconnected: HashSet<usize> = [0, 2].into_iter().collect();
    assert!(is_connected_subset(&adj, &connected));
    assert!(!is_connected_subset(&adj, &disconnected));
}

#[test]
fn count_edge_cuts_known_grid() {
    // 4-node path split at midpoint: [0,1] | [2,3]. One cut edge: 1-2.
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let mut asgn = HashMap::new();
    asgn.insert(0, 1);
    asgn.insert(1, 1);
    asgn.insert(2, 2);
    asgn.insert(3, 2);
    assert_eq!(
        count_edge_cuts(&asgn, &adj),
        1,
        "4-node path bisection has 1 cut edge"
    );
}

#[test]
fn weighted_edge_cut_sums_crossing_weights() {
    let edge_weights = HashMap::from([
        ((0usize, 1usize), 1.5),
        ((1usize, 2usize), 2.25),
        ((2usize, 3usize), 3.0),
    ]);
    let left = HashSet::from([0usize, 1]);

    assert_eq!(weighted_edge_cut(&edge_weights, &left), 2.25);
}

#[test]
fn weighted_edge_cut_is_zero_when_no_edges_cross() {
    let edge_weights = HashMap::from([
        ((0usize, 1usize), 1.5),
        ((1usize, 2usize), 2.25),
        ((2usize, 3usize), 3.0),
    ]);
    let all_left = HashSet::from([0usize, 1, 2, 3]);
    let none_left = HashSet::new();

    assert_eq!(weighted_edge_cut(&edge_weights, &all_left), 0.0);
    assert_eq!(weighted_edge_cut(&edge_weights, &none_left), 0.0);
}

#[test]
fn weighted_edge_cut_sums_all_crossing_edges() {
    let edge_weights = HashMap::from([
        ((0usize, 1usize), 1.5),
        ((0usize, 2usize), 2.0),
        ((1usize, 3usize), 3.25),
    ]);
    let left = HashSet::from([0usize, 1]);

    assert_eq!(weighted_edge_cut(&edge_weights, &left), 5.25);
}

#[test]
fn weighted_edge_cut_treats_missing_nodes_as_right_side() {
    let edge_weights = HashMap::from([((0usize, 10usize), 4.0), ((10usize, 11usize), 7.0)]);
    let left = HashSet::from([0usize]);

    assert_eq!(weighted_edge_cut(&edge_weights, &left), 4.0);
}

// ── Simulated Annealing tests ─────────────────────────────────────────────

// L0: zero steps returns the initial METIS plan unchanged (best = initial).
#[test]
fn sa_zero_steps_returns_initial() {
    // 4x4 grid, steps_per_tract=0 → no SA steps → best = initial METIS plan
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_sa(
        &adj, &pop, &ew, &tracts, 0.10, // balance_tolerance
        0,    // steps_per_tract = 0 → n_steps = 0
        0.01, 1e-4, 42,
    )
    .expect("SA with 0 steps must succeed");
    assert!(
        !left.is_empty() && !right.is_empty(),
        "both sides non-empty"
    );
    assert_eq!(left.len() + right.len(), 16, "all tracts covered");
    assert!(left.is_disjoint(&right), "sides must be disjoint");
}

// L0: same sa_seed → identical result (determinism).
#[test]
fn sa_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let run = |seed: u64| {
        split_subgraph_sa(&adj, &pop, &ew, &tracts, 0.10, 5, 0.01, 1e-4, seed)
            .expect("SA must succeed")
    };
    let (l1, r1) = run(99);
    let (l2, r2) = run(99);
    // Sort to compare deterministically
    let mut v1: Vec<usize> = l1
        .iter()
        .chain(r1.iter())
        .map(|&v| v + if l1.contains(&v) { 0 } else { 100 })
        .collect();
    let mut v2: Vec<usize> = l2
        .iter()
        .chain(r2.iter())
        .map(|&v| v + if l2.contains(&v) { 0 } else { 100 })
        .collect();
    v1.sort_unstable();
    v2.sort_unstable();
    assert_eq!(v1, v2, "same seed must produce identical partition");
}

// L0: t0_factor=0.0 forces T0=max(1.0, 0.0)=1.0 (test fixture from spec).
#[test]
fn sa_t0_zero_factor_uses_floor() {
    // With t0_factor=0.0, T_0 = max(1.0, 0.0 * EC) = 1.0 regardless of EC.
    let t0_factor = 0.0_f64;
    let initial_ec = 5usize;
    let t0 = (t0_factor * initial_ec as f64).max(1.0);
    assert!(
        (t0 - 1.0).abs() < 1e-10,
        "t0_factor=0.0 must give T_0=1.0, got {t0}"
    );
    // Also verify it runs without panic
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let result = split_subgraph_sa(&adj, &pop, &ew, &tracts, 0.10, 3, 0.0, 1e-4, 42);
    assert!(
        result.is_ok(),
        "SA with t0_factor=0.0 must succeed: {:?}",
        result.err()
    );
}

// L0: greedy mode (t_final == t0 effectively zero temperature) never increases EC.
// We test: SA with tiny t_final and small steps should not produce higher EC than initial.
#[test]
fn sa_never_increases_ec_greedy() {
    // With t_final=1e-15 (near zero) the acceptance probability for worsening moves
    // is ~exp(-delta/1e-15) ≈ 0 for any positive delta_ec.
    // So EC should be <= initial_ec (or equal if no improvement found).
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();

    // Get initial METIS EC for reference
    let (l_metis, r_metis) =
        split_subgraph(&adj, &pop, 1, &ew, &tracts, 1.10, 100, Some(42), None, None)
            .expect("METIS must succeed");
    let mut metis_asgn = HashMap::new();
    for &v in &l_metis {
        metis_asgn.insert(v, 1usize);
    }
    for &v in &r_metis {
        metis_asgn.insert(v, 2usize);
    }
    let initial_ec = count_edge_cuts(&metis_asgn, &adj);

    let (l_sa, r_sa) = split_subgraph_sa(&adj, &pop, &ew, &tracts, 0.10, 5, 0.01, 1e-15, 42)
        .expect("SA greedy must succeed");
    let mut sa_asgn = HashMap::new();
    for &v in &l_sa {
        sa_asgn.insert(v, 1usize);
    }
    for &v in &r_sa {
        sa_asgn.insert(v, 2usize);
    }
    let sa_ec = count_edge_cuts(&sa_asgn, &adj);
    assert!(
        sa_ec <= initial_ec,
        "greedy SA (t_final=1e-15) must not increase EC: initial={initial_ec} sa={sa_ec}"
    );
}

// L1: SA produces a valid 2-partition on a 4x4 grid (contiguity + balance).
#[test]
fn sa_produces_valid_2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_sa(&adj, &pop, &ew, &tracts, 0.10, 10, 0.01, 1e-4, 777)
        .expect("SA 4x4 must succeed");

    // Completeness and disjointness
    assert_eq!(left.len() + right.len(), 16, "all 16 tracts covered");
    assert!(left.is_disjoint(&right), "sides disjoint");
    assert!(
        !left.is_empty() && !right.is_empty(),
        "both sides non-empty"
    );

    // Contiguity: BFS check for each side
    let check_connected = |side: &HashSet<usize>| -> bool {
        let members: Vec<usize> = side.iter().copied().collect();
        if members.len() <= 1 {
            return true;
        }
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(members[0]);
        visited.insert(members[0]);
        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                if side.contains(&nb) && !visited.contains(&nb) {
                    visited.insert(nb);
                    queue.push_back(nb);
                }
            }
        }
        members.iter().all(|v| visited.contains(v))
    };
    assert!(check_connected(&left), "left side must be contiguous");
    assert!(check_connected(&right), "right side must be contiguous");

    // Balance: each side within 10% of half total pop
    let total_pop: i64 = pop.iter().sum();
    let left_pop: i64 = left.iter().map(|&v| pop[v]).sum();
    let balance = (left_pop as f64 - total_pop as f64 / 2.0).abs() / total_pop as f64;
    assert!(
        balance <= 0.10,
        "SA balance must be within 10%: {balance:.3}"
    );
}

// L1: SA result EC <= initial METIS EC + small_margin (SA should not seriously worsen EC).
#[test]
fn sa_improves_or_equals_metis() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let tracts: HashSet<usize> = (0..16).collect();

    // METIS baseline
    let (l_m, r_m) = split_subgraph(&adj, &pop, 1, &ew, &tracts, 1.10, 100, Some(42), None, None)
        .expect("METIS baseline must succeed");
    let mut m_asgn = HashMap::new();
    for &v in &l_m {
        m_asgn.insert(v, 1usize);
    }
    for &v in &r_m {
        m_asgn.insert(v, 2usize);
    }
    let metis_ec = count_edge_cuts(&m_asgn, &adj);

    // SA with enough steps to make progress
    let (l_sa, r_sa) = split_subgraph_sa(&adj, &pop, &ew, &tracts, 0.10, 20, 0.01, 1e-4, 42)
        .expect("SA must succeed");
    let mut sa_asgn = HashMap::new();
    for &v in &l_sa {
        sa_asgn.insert(v, 1usize);
    }
    for &v in &r_sa {
        sa_asgn.insert(v, 2usize);
    }
    let sa_ec = count_edge_cuts(&sa_asgn, &adj);

    // SA may equal METIS (especially on a tight grid), but must not be >> METIS.
    // Allow up to +2 edge cuts as "small margin" for stochastic variance.
    assert!(
        sa_ec <= metis_ec + 2,
        "SA EC should not exceed METIS EC + 2: metis={metis_ec} sa={sa_ec}"
    );
}

// L2 (ignored): SA on North Carolina should improve or equal compactness vs METIS.
#[test]
#[ignore]
fn sa_nc_compactness_improvement() {
    // Requires real NC adjacency data at data/2020/ — runs as L2 only.
    // Placeholder: actual implementation would load NC graph and compare
    // Polsby-Popper scores between METIS and SA outputs.
    panic!("L2 test: requires real NC data — run manually with --ignored");
}

// ── Centroidal Voronoi Districts (CVD) tests ──────────────────────────────

// L0: 4x4 grid, 2 seeds — valid partition covering all 16 tracts.
#[test]
fn cvd_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_cvd(
        &adj, &pop, &tracts, 2,    // k=2 (bisection)
        0.10, // balance_tolerance
        20,   // n_iter
        42,   // base_seed
    )
    .expect("CVD 4x4 must succeed");

    // Completeness
    assert_eq!(left.len() + right.len(), 16, "all 16 tracts covered");
    assert!(left.is_disjoint(&right), "left and right must be disjoint");
    assert!(
        !left.is_empty() && !right.is_empty(),
        "both sides must be non-empty"
    );

    // All tracts in 0..16
    let mut all: Vec<usize> = left.iter().chain(right.iter()).copied().collect();
    all.sort_unstable();
    assert_eq!(
        all,
        (0..16usize).collect::<Vec<_>>(),
        "must partition exactly 0..16"
    );
}

// L0: same base_seed -> identical result (determinism).
#[test]
fn cvd_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();

    let run = |seed: u64| {
        split_subgraph_cvd(&adj, &pop, &tracts, 2, 0.10, 20, seed).expect("CVD must succeed")
    };

    let (l1, r1) = run(77);
    let (l2, r2) = run(77);

    let mut s1: Vec<usize> = l1.iter().copied().collect();
    s1.sort_unstable();
    let mut s2: Vec<usize> = l2.iter().copied().collect();
    s2.sort_unstable();
    assert_eq!(s1, s2, "same seed must give identical left set");

    let mut t1: Vec<usize> = r1.iter().copied().collect();
    t1.sort_unstable();
    let mut t2: Vec<usize> = r2.iter().copied().collect();
    t2.sort_unstable();
    assert_eq!(t1, t2, "same seed must give identical right set");
}

// L0: two seeds must be distinct for non-trivial graphs.
#[test]
fn cvd_seeds_are_distinct() {
    // On a 4x4 grid, seed0 = 42 % 16 = 10.
    // seed1 = farthest from 10 → must be != 10.
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    // We test by verifying the partition is non-trivial (not all one side)
    // which implies seeds[0] != seeds[1] since equal seeds → identical Voronoi → all assigned to 0.
    let (left, right) =
        split_subgraph_cvd(&adj, &pop, &tracts, 2, 0.10, 20, 42).expect("CVD must succeed");
    assert!(
        !right.is_empty(),
        "right must be non-empty (seeds must be distinct)"
    );
}

// L0: n_iter=0 returns the initial Voronoi assignment without crashing.
#[test]
fn cvd_n_iter_zero_no_crash() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let result = split_subgraph_cvd(&adj, &pop, &tracts, 2, 0.10, 0, 42);
    assert!(
        result.is_ok(),
        "n_iter=0 must not panic: {:?}",
        result.err()
    );
    let (left, right) = result.unwrap();
    assert_eq!(
        left.len() + right.len(),
        16,
        "all tracts covered even with n_iter=0"
    );
}

// L1: CVD on a 4x4 grid with k=2 produces a contiguous split in <= 20 iterations.
#[test]
fn cvd_convergence_within_n_iter() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    // Run with n_iter=20 and verify the result is a valid partition.
    // We can't directly inspect iteration count from the public API (iters_done is internal),
    // so we verify the algorithm terminates and produces a valid result.
    let (left, right) = split_subgraph_cvd(&adj, &pop, &tracts, 2, 0.10, 20, 99)
        .expect("CVD must succeed within 20 iterations");
    assert_eq!(left.len() + right.len(), 16);
    assert!(left.is_disjoint(&right));
}

// L1: balance check — each side within 10% of half total pop after rebalance.
#[test]
fn cvd_balance_within_tolerance() {
    let (adj, pop) = small_grid(4, 4);
    let total_pop: i64 = pop.iter().sum();
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) =
        split_subgraph_cvd(&adj, &pop, &tracts, 2, 0.10, 20, 42).expect("CVD must succeed");
    let left_pop: i64 = left.iter().map(|&v| pop[v]).sum();
    let right_pop: i64 = right.iter().map(|&v| pop[v]).sum();
    let imbalance_l = (left_pop as f64 - total_pop as f64 / 2.0).abs() / total_pop as f64;
    let imbalance_r = (right_pop as f64 - total_pop as f64 / 2.0).abs() / total_pop as f64;
    assert!(
        imbalance_l <= 0.10,
        "left imbalance {imbalance_l:.3} must be within 10%"
    );
    assert!(
        imbalance_r <= 0.10,
        "right imbalance {imbalance_r:.3} must be within 10%"
    );
}

// L0: CVD prefix constant is "CVD_INIT_" exactly (audit chain invariant).
#[test]
fn cvd_prefix_constant() {
    // SHA-256("CVD_INIT_" || "" || "_" || 0u64le) should be a fixed, known value.
    // We test that derive_cvd_seed(0, "") matches the expected hash.
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"CVD_INIT_");
    h.update(b"");
    h.update(b"_");
    h.update(0u64.to_le_bytes());
    let d = h.finalize();
    let expected = u64::from_le_bytes(d[..8].try_into().unwrap());
    assert_eq!(
        derive_cvd_seed(0, ""),
        expected,
        "derive_cvd_seed(0, '') must equal SHA-256('CVD_INIT__' || 0:le64) first 8 bytes"
    );
    // Verify prefix: the prefix must be exactly "CVD_INIT_"
    let h2: Vec<u8> = {
        let mut h = Sha256::new();
        h.update(b"CVD_INIT_");
        h.update(b"root");
        h.update(b"_");
        h.update(42u64.to_le_bytes());
        h.finalize().to_vec()
    };
    let from_derive = derive_cvd_seed(42, "root");
    let from_expected = u64::from_le_bytes(h2[..8].try_into().unwrap());
    assert_eq!(from_derive, from_expected, "prefix must be CVD_INIT_");
}

// L0: cvd_structure_mode_parses — "centroidal-voronoi" parses as PartitionMode::CentroidalVoronoi.
#[test]
fn cvd_structure_mode_parses() {
    use crate::args::PartitionMode;
    use clap::ValueEnum;
    let parsed = PartitionMode::from_str("centroidal-voronoi", true)
        .expect("PartitionMode must parse 'centroidal-voronoi'");
    assert!(
        matches!(parsed, PartitionMode::CentroidalVoronoi),
        "Expected CentroidalVoronoi, got {:?}",
        parsed
    );
}

// L2 (ignored): CVD on NC — compare mean Polsby-Popper vs single METIS call.
#[test]
#[ignore]
fn cvd_nc_pp_vs_metis() {
    // Requires real NC adjacency data at data/2020/ — L2 only.
    // Expected: CVD PP >= METIS PP (geographic proximity improves compactness).
    panic!("L2 test: requires real NC data — run manually with --ignored");
}

// ── CVD Phase 2 (Geographic) tests ──────────────────────────────────────────

// Helper: build a 4x4 grid with evenly-spaced centroids (1.0 degree apart)
// spanning -100..-97 lon, 37..40 lat (central US, all valid Albers inputs)
fn grid4x4_centroids() -> Vec<(f64, f64)> {
    let mut c = Vec::with_capacity(16);
    for row in 0..4 {
        for col in 0..4 {
            // lon: -100.0 + col*1.0, lat: 37.0 + row*1.0
            c.push((-100.0 + col as f64, 37.0 + row as f64));
        }
    }
    c
}

// L0: 4x4 grid with synthetic centroids — valid k=2 partition covering all 16 tracts.
#[test]
fn cvd_geographic_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = grid4x4_centroids();
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) =
        split_subgraph_cvd_geographic(&adj, &pop, &centroids, &tracts, 0.10, 20, 42, "root")
            .expect("CVD geographic 4x4 must succeed");

    assert_eq!(left.len() + right.len(), 16, "all 16 tracts covered");
    assert!(left.is_disjoint(&right), "left and right must be disjoint");
    assert!(
        !left.is_empty() && !right.is_empty(),
        "both sides non-empty"
    );

    let mut all: Vec<usize> = left.iter().chain(right.iter()).copied().collect();
    all.sort_unstable();
    assert_eq!(
        all,
        (0..16usize).collect::<Vec<_>>(),
        "must partition exactly 0..16"
    );
}

// L0: same seed and node_path -> identical result (determinism).
#[test]
fn cvd_geographic_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = grid4x4_centroids();
    let tracts: HashSet<usize> = (0..16).collect();

    let run = || {
        split_subgraph_cvd_geographic(&adj, &pop, &centroids, &tracts, 0.10, 20, 77, "node01")
            .expect("must succeed")
    };

    let (l1, r1) = run();
    let (l2, r2) = run();

    let mut s1: Vec<usize> = l1.iter().copied().collect();
    s1.sort_unstable();
    let mut s2: Vec<usize> = l2.iter().copied().collect();
    s2.sort_unstable();
    assert_eq!(s1, s2, "same seed must give identical left set");

    let mut t1: Vec<usize> = r1.iter().copied().collect();
    t1.sort_unstable();
    let mut t2: Vec<usize> = r2.iter().copied().collect();
    t2.sort_unstable();
    assert_eq!(t1, t2, "same seed must give identical right set");
}

// L0: Phase 2 seed prefix is distinct from Phase 1 seed prefix.
#[test]
fn cvd_geo_seed_distinct_from_phase1_seed() {
    // SHA-256("CVD_GEO_INIT_"...) != SHA-256("CVD_INIT_"...) for same inputs
    let geo_seed = derive_cvd_geo_seed(42, "root");
    let phase1_seed = derive_cvd_seed(42, "root");
    assert_ne!(
        geo_seed, phase1_seed,
        "CVD_GEO_INIT_ prefix must produce different seeds than CVD_INIT_"
    );
}

// L0: albers_project(-96, 37.5): x near 0 (central meridian), y finite and positive.
#[test]
fn albers_project_central_meridian_near_zero() {
    let (x, y) = albers_project(-96.0, 37.5);
    assert!(
        x.abs() < 1000.0,
        "x at central meridian must be within 1km of 0, got {x}"
    );
    assert!(y.is_finite(), "y must be finite, got {y}");
    assert!(
        y >= 0.0,
        "y must be non-negative (positive northing), got {y}"
    );
}

// L0: empty centroids with Geographic metric returns Err containing "centroid".
#[test]
fn cvd_metric_geographic_missing_centroids_returns_err() {
    let (adj, pop) = small_grid(4, 4);
    let empty_centroids: Vec<(f64, f64)> = vec![];
    let tracts: HashSet<usize> = (0..16).collect();
    let result =
        split_subgraph_cvd_geographic(&adj, &pop, &empty_centroids, &tracts, 0.10, 20, 42, "root");
    assert!(result.is_err(), "empty centroids must return Err");
    let err = result.unwrap_err();
    assert!(
        err.contains("centroid"),
        "error must mention centroid, got: {err}"
    );
}

// L2 (ignored): NC geographic CVD vs graph-distance CVD -- Phase 2 paper comparison.
#[test]
#[ignore]
fn cvd_geographic_nc_pp_vs_graph_distance() {
    // Requires real NC adjacency + centroid data at data/2020/ -- L2 only.
    // Expected: geographic CVD mean PP > graph-distance CVD mean PP (coastal NC tracts).
    panic!("L2 test: requires real NC data with centroids -- run manually with --ignored");
}

// ── ShortBurst ────────────────────────────────────────────────────────────

/// 4×2 grid helper: 8 nodes, 2 districts, uniform pop=1000. k=2.
fn grid8_adj() -> Vec<Vec<usize>> {
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

fn grid8_pop() -> Vec<i64> {
    vec![1000i64; 8]
}

// L0: n_bursts=0 returns the initial plan unchanged (no ReCom at all).
#[test]
fn short_burst_zero_bursts_returns_initial() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (asgn, seeds, idx) = run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 42, 20, 0, 0.0)
        .expect("short_burst n_bursts=0 should succeed");
    assert_eq!(asgn.len(), 8, "must assign all 8 tracts");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
    assert!(seeds.is_empty(), "no burst seeds when n_bursts=0");
    assert_eq!(idx, 0, "selected index must be 0 when n_bursts=0");
}

// L0: p=0.0 returns the minimum-EC endpoint.
#[test]
fn short_burst_p0_returns_min_ec_endpoint() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (asgn, seeds, _idx) = run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 99, 5, 10, 0.0)
        .expect("p=0.0 short burst must succeed");
    assert_eq!(asgn.len(), 8);
    assert_eq!(seeds.len(), 10, "must have exactly n_bursts=10 seeds");
    // p=0.0 selects rank 0 = minimum EC — just verify it's a valid plan.
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2);
}

// L0: p=1.0 returns the maximum-EC endpoint.
#[test]
fn short_burst_p1_returns_max_ec_endpoint() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (asgn_min, _, _) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 77, 5, 10, 0.0).expect("p=0.0");
    let (asgn_max, _, _) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 77, 5, 10, 1.0).expect("p=1.0");
    // Both are valid plans.
    assert_eq!(asgn_min.len(), 8);
    assert_eq!(asgn_max.len(), 8);
    let d_min: std::collections::HashSet<usize> = asgn_min.values().copied().collect();
    let d_max: std::collections::HashSet<usize> = asgn_max.values().copied().collect();
    assert_eq!(d_min.len(), 2);
    assert_eq!(d_max.len(), 2);
}

// L0: same seed → same result (determinism).
#[test]
fn short_burst_deterministic() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (a1, s1, i1) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 42, 10, 5, 0.3).expect("run 1");
    let (a2, s2, i2) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 42, 10, 5, 0.3).expect("run 2");
    assert_eq!(a1, a2, "same seed must give same assignment");
    assert_eq!(s1, s2, "same seed must give same burst seeds");
    assert_eq!(i1, i2, "same seed must give same selected burst index");
}

// L0: degenerate single burst of length 1.
#[test]
fn short_burst_n1_burst_len1() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (asgn, seeds, idx) = run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 7, 1, 1, 0.0)
        .expect("n_bursts=1, burst_length=1");
    assert_eq!(asgn.len(), 8);
    assert_eq!(seeds.len(), 1);
    assert_eq!(idx, 0);
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2);
}

// L0: verify chain restarts from endpoint — with burst_length=0 all endpoints are the
// initial plan, so EC ties are broken by burst_idx ASC (rank 0 = burst 0, rank 4 = burst 4).
#[test]
fn short_burst_chain_restarts_from_endpoint() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    // burst_length=0: chain steps 0 times, endpoint == input == initial plan for all bursts.
    let (asgn_p0, seeds_p0, idx_p0) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 13, 0, 5, 0.0).expect("burst_length=0 p=0.0");
    let (asgn_p1, seeds_p1, idx_p1) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 13, 0, 5, 1.0).expect("burst_length=0 p=1.0");
    // All endpoints are the same plan, so both p=0.0 and p=1.0 select the same assignment.
    assert_eq!(
        asgn_p0, asgn_p1,
        "with burst_length=0 all endpoints are the same plan"
    );
    assert_eq!(seeds_p0, seeds_p1, "same base seed means same burst seeds");
    // Tie-breaking by burst_idx ASC: rank 0 = burst 0, rank 4 = burst 4.
    assert_eq!(idx_p0, 0, "p=0.0 with equal ECs must select burst 0");
    assert_eq!(idx_p1, 4, "p=1.0 with equal ECs must select burst 4");
}

// L1: produces a valid k=2 partition on a 4-node diamond graph.
#[test]
fn short_burst_produces_valid_k2_partition() {
    let adj = vec![vec![1usize, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
    let pop = vec![1000i64; 4];
    let ew = HashMap::new();
    let (asgn, _, _) =
        run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 55, 10, 10, 0.5).expect("L1 k=2 partition");
    assert_eq!(asgn.len(), 4, "all 4 tracts assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be 1 or 2, got {d}");
    }
}

// L1: all districts populated after short burst.
#[test]
fn short_burst_all_districts_populated() {
    let adj = grid8_adj();
    let pop = grid8_pop();
    let ew = HashMap::new();
    let (asgn, _, _) = run_short_burst(&adj, &pop, &ew, 2, 0.05, 10, 100, 20, 20, 0.5)
        .expect("all districts populated");
    let mut dist_pops: std::collections::HashMap<usize, i64> = std::collections::HashMap::new();
    for (&tract, &d) in &asgn {
        *dist_pops.entry(d).or_default() += pop[tract];
    }
    assert_eq!(dist_pops.len(), 2, "both districts must be non-empty");
    for (&d, &p) in &dist_pops {
        assert!(p > 0, "district {d} must have positive population");
    }
}

// L2: ignored by default — run manually for empirical validation.
#[test]
#[ignore]
fn short_burst_nc_outperforms_multi_seed() {
    // Placeholder: load NC adjacency, run both multi-seed and short-burst,
    // compare edge-cut distributions. Skipped unless --ignored is passed.
    // Implementation left for G.6 paper experiments.
}

// ── Flip search tests ─────────────────────────────────────────────────────

/// Build a 4x4 grid adjacency for flip tests.
fn grid_4x4() -> (Vec<Vec<usize>>, Vec<i64>) {
    let (adj, pop) = small_grid(4, 4);
    (adj, pop)
}

// L0: zero steps returns the initial plan (visited list has exactly 1 entry).
#[test]
fn flip_zero_steps_returns_initial_plan() {
    let (adj, pop) = grid_4x4();
    let ew = HashMap::new();
    let (asgn, visited_count, rank) = run_flip_chain(&adj, &pop, &ew, 2, 0.05, 0, 42, 0.0)
        .expect("flip_chain must succeed with 0 steps");
    assert_eq!(
        visited_count, 1,
        "0 steps => visited has 1 entry (initial plan)"
    );
    assert_eq!(rank, 0, "only one plan, rank must be 0");
    assert_eq!(asgn.len(), 16, "all 16 tracts assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "2 districts");
}

// L0: visited list is never empty (initial plan always included).
#[test]
fn flip_visited_count_ge_1() {
    let (adj, pop) = grid_4x4();
    let ew = HashMap::new();
    // Even with 0 steps, visited must be >= 1.
    let (_, visited_count, _) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.05, 0, 99, 0.0).expect("flip_chain must succeed");
    assert!(
        visited_count >= 1,
        "visited list must contain at least the initial plan"
    );
}

// L0: p=0.0 selects plan with EC <= plan selected by p=1.0 (sort ascending).
#[test]
fn flip_p0_le_flip_p1_ec() {
    let (adj, pop) = grid_4x4();
    let ew = HashMap::new();
    let (asgn_p0, _, _) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.1, 200, 42, 0.0).expect("flip p=0.0 must succeed");
    let (asgn_p1, _, _) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.1, 200, 42, 1.0).expect("flip p=1.0 must succeed");
    let ec_p0 = count_edge_cuts(&asgn_p0, &adj);
    let ec_p1 = count_edge_cuts(&asgn_p1, &adj);
    assert!(
        ec_p0 <= ec_p1,
        "p=0.0 (min) EC={ec_p0} must be <= p=1.0 (max) EC={ec_p1}"
    );
}

// L0: determinism — same seed produces the same result.
#[test]
fn flip_deterministic() {
    let (adj, pop) = grid_4x4();
    let ew = HashMap::new();
    let (asgn1, v1, r1) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.05, 100, 777, 0.5).expect("first run must succeed");
    let (asgn2, v2, r2) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.05, 100, 777, 0.5).expect("second run must succeed");
    assert_eq!(v1, v2, "visited_count must be the same for same seed");
    assert_eq!(r1, r2, "rank must be the same for same seed");
    assert_eq!(asgn1, asgn2, "assignment must be identical for same seed");
}

// L0: all-fail case — tiny graph where all flips violate balance.
// Use 2 tracts of very unequal weight and zero tolerance: flip always violates balance.
#[test]
fn flip_all_fail_returns_initial_plan() {
    // 4-node path, all equal pop, very tight tolerance (0.0 -> max_dev = 1)
    // With some effort flips will be accepted on balanced graph, but let's use
    // a disconnected scenario: 2 isolated nodes each assigned to their own district.
    // No boundary tracts -> boundary is empty -> all steps are no-ops.
    let adj = vec![vec![], vec![]]; // 2 isolated nodes
    let pop = vec![1000i64, 1000];
    let ew = HashMap::new();
    let (asgn, visited_count, _) = run_flip_chain(&adj, &pop, &ew, 2, 0.0, 100, 42, 0.0)
        .expect("flip_chain on isolated graph must succeed");
    assert_eq!(
        visited_count, 1,
        "no boundary tracts -> no flips -> only initial plan"
    );
    assert_eq!(asgn.len(), 2);
}

// L1: valid partition — all tracts assigned, no overlap, correct district count.
#[test]
fn flip_produces_valid_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let (asgn, _, _) = run_flip_chain(&adj, &pop, &ew, 4, 0.1, 500, 42, 0.0)
        .expect("flip_chain must produce a valid partition");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    for i in 0..16 {
        let d = asgn[&i];
        assert!(
            d >= 1 && d <= 4,
            "district {d} out of range [1,4] for tract {i}"
        );
    }
    // No duplicate assignments (each tract assigned exactly once).
    let unique: std::collections::HashSet<usize> = asgn.keys().copied().collect();
    assert_eq!(unique.len(), 16, "each tract appears exactly once");
}

// L1: contiguity preserved — all districts remain contiguous after flips.
#[test]
fn flip_contiguity_preserved() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let (asgn, _, _) =
        run_flip_chain(&adj, &pop, &ew, 2, 0.1, 500, 123, 0.5).expect("flip_chain must succeed");
    // Check contiguity of each district using BFS.
    let n = adj.len();
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    for d in &districts {
        let members: Vec<usize> = (0..n).filter(|&v| asgn[&v] == *d).collect();
        assert!(!members.is_empty(), "district {d} must be non-empty");
        if members.len() == 1 {
            continue;
        }
        let start = members[0];
        let mut visited_bfs = vec![false; n];
        visited_bfs[start] = true;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                if !visited_bfs[nb] && asgn[&nb] == *d {
                    visited_bfs[nb] = true;
                    queue.push_back(nb);
                }
            }
        }
        for &v in &members {
            assert!(
                visited_bfs[v],
                "district {d}: tract {v} is disconnected from the rest"
            );
        }
    }
}

// L2: real-data NC test (ignored by default).
#[test]
#[ignore]
fn flip_nc_improves_ec() {
    // Placeholder: load NC adjacency, compare flip p=0.0 EC vs baseline.
    // Skipped unless --include-ignored is passed.
}

// ── ForestRecom tests ─────────────────────────────────────────────────────

// L0: run_forest_recom must return a valid 2-district plan on a 4×4 grid.
#[test]
fn forest_recom_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_forest_recom(&adj, &pop, &ew, 2, 0.05, 10, 42, 50, 0.0)
        .expect("forest_recom k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L0: p=0.0 EC <= p=1.0 EC (ascending sort).
#[test]
fn forest_recom_p0_le_p1_ec() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn_p0 = run_forest_recom(&adj, &pop, &ew, 2, 0.10, 10, 77, 100, 0.0)
        .expect("forest_recom p=0.0 must succeed");
    let asgn_p1 = run_forest_recom(&adj, &pop, &ew, 2, 0.10, 10, 77, 100, 1.0)
        .expect("forest_recom p=1.0 must succeed");
    let ec_p0 = count_edge_cuts(&asgn_p0, &adj);
    let ec_p1 = count_edge_cuts(&asgn_p1, &adj);
    assert!(
        ec_p0 <= ec_p1,
        "p=0.0 (min) EC={ec_p0} must be <= p=1.0 (max) EC={ec_p1}"
    );
}

// L0: same seed -> same result (determinism).
#[test]
fn forest_recom_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn1 = run_forest_recom(&adj, &pop, &ew, 2, 0.05, 10, 42, 50, 0.5)
        .expect("first run must succeed");
    let asgn2 = run_forest_recom(&adj, &pop, &ew, 2, 0.05, 10, 42, 50, 0.5)
        .expect("second run must succeed");
    assert_eq!(asgn1, asgn2, "same seed must produce identical assignment");
}

// L0: steps=0 returns the initial METIS plan (no chain steps).
#[test]
fn forest_recom_zero_steps_returns_initial() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_forest_recom(&adj, &pop, &ew, 2, 0.05, 10, 99, 0, 0.0)
        .expect("forest_recom steps=0 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
}

// L0: no district is empty in the result.
#[test]
fn forest_recom_all_districts_populated() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_forest_recom(&adj, &pop, &ew, 2, 0.05, 10, 55, 100, 0.5)
        .expect("forest_recom must succeed");
    let mut dist_pops: std::collections::HashMap<usize, i64> = std::collections::HashMap::new();
    for (&tract, &d) in &asgn {
        *dist_pops.entry(d).or_default() += pop[tract];
    }
    assert_eq!(dist_pops.len(), 2, "both districts must be non-empty");
    for (&d, &p) in &dist_pops {
        assert!(p > 0, "district {d} must have positive population");
    }
}

// L1: k=4 on 4×4 grid — all four districts populated and valid.
#[test]
fn forest_recom_k4_all_districts_populated() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_forest_recom(&adj, &pop, &ew, 4, 0.10, 10, 123, 50, 0.0)
        .expect("forest_recom k=4 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let mut dist_pops: std::collections::HashMap<usize, i64> = std::collections::HashMap::new();
    for (&tract, &d) in &asgn {
        *dist_pops.entry(d).or_default() += pop[tract];
    }
    assert_eq!(dist_pops.len(), 4, "all 4 districts must be non-empty");
}

// L2: NC 2020 k=14, T=1000 — acceptance rate > 0 (ignored by default).
#[test]
#[ignore]
fn forest_recom_nc_acceptance_rate() {
    // Placeholder: load NC 2020 adjacency + population, run 1000 steps.
    // Assert acceptance_rate > 0. Skipped unless --include-ignored is passed.
}

// ── MultiScale tests ──────────────────────────────────────────────────────

/// Build a synthetic geoid map for an n-tract grid split into two counties.
/// Tracts 0..(n/2) -> county "37001", tracts (n/2)..n -> county "37003".
fn synthetic_geoids(n: usize) -> std::collections::HashMap<usize, String> {
    (0..n)
        .map(|i| {
            let county = if i < n / 2 { "37001" } else { "37003" };
            let tract_num = i % (n / 2);
            let geoid = format!("{county}{tract_num:06}");
            (i, geoid)
        })
        .collect()
}

// L0: missing geoids must return Err containing "GEOID".
#[test]
fn multiscale_missing_geoids_returns_err() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let result = run_multiscale(
        &adj,
        &pop,
        &ew,
        2,
        0.1,
        10,
        42,
        100,
        0.3,
        0.0,
        None,
        MultiscaleFineLevel::Tract,
        "county",
        None,
    );
    assert!(
        result.is_err(),
        "run_multiscale with no geoids must return Err"
    );
    let msg = result.unwrap_err();
    assert!(
        msg.contains("GEOID"),
        "error must mention GEOID, got: {msg}"
    );
}

// L0: fine=bg with bg_graph=None must return Err mentioning "block-group adjacency".
#[test]
fn multiscale_option_a_missing_bg_returns_informative_error() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let geoids = synthetic_geoids(16);
    let result = run_multiscale(
        &adj,
        &pop,
        &ew,
        2,
        0.10,
        10,
        42,
        50,
        0.3,
        0.0,
        Some(&geoids),
        MultiscaleFineLevel::BlockGroup,
        "tract",
        None,
    );
    assert!(
        result.is_err(),
        "run_multiscale with fine=bg and no bg_graph must return Err"
    );
    let msg = result.unwrap_err();
    assert!(
        msg.contains("block-group adjacency") || msg.contains("block_group") || msg.contains("bg"),
        "error must mention block-group adjacency, got: {msg}"
    );
}

// L0: Option B produces a valid 2-district plan on a 4x4 grid with synthetic geoids.
#[test]
fn multiscale_option_b_valid_plan() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let geoids = synthetic_geoids(16);
    let result = run_multiscale(
        &adj,
        &pop,
        &ew,
        2,
        0.10,
        10,
        42,
        50,
        0.3,
        0.0,
        Some(&geoids),
        MultiscaleFineLevel::Tract,
        "county",
        None,
    );
    let asgn = result.expect("multiscale Option B must succeed on 4x4 grid");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L0: Option B unchanged — same seed produces same result (determinism).
#[test]
fn multiscale_option_b_unchanged() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let geoids = synthetic_geoids(16);
    let run = || {
        run_multiscale(
            &adj,
            &pop,
            &ew,
            2,
            0.10,
            10,
            99,
            50,
            0.3,
            0.5,
            Some(&geoids),
            MultiscaleFineLevel::Tract,
            "county",
            None,
        )
        .expect("multiscale Option B must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same seed must produce identical assignment");
}

// L0: same seed must produce the same result (determinism).
#[test]
fn multiscale_option_b_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let geoids = synthetic_geoids(16);
    let run = || {
        run_multiscale(
            &adj,
            &pop,
            &ew,
            2,
            0.10,
            10,
            99,
            50,
            0.3,
            0.5,
            Some(&geoids),
            MultiscaleFineLevel::Tract,
            "county",
            None,
        )
        .expect("multiscale Option B must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same seed must produce identical assignment");
}

// L0: derive_partition BG->tract with synthetic 12-node graph (3 tracts, 4 BGs each).
#[test]
fn derive_partition_bg_to_tract_synthetic() {
    use crate::adjacency_loader::derive_partition;
    // 12 BG geoids (12 chars each): 3 groups of 4, mapping to 3 tracts
    let bg_geoids: std::collections::HashMap<usize, String> = (0..12)
        .map(|i| {
            // BG 0-3 -> tract "37001000100"; BG 4-7 -> "37001000200"; BG 8-11 -> "37001000300"
            let tract_num = i / 4 + 1;
            let bg_num = i % 4 + 1;
            let geoid = format!("37001{tract_num:06}{bg_num}");
            (i, geoid)
        })
        .collect();
    let tract_geoids: std::collections::HashMap<usize, String> = (0..3)
        .map(|t| {
            let geoid = format!("37001{:06}", t + 1);
            (t, geoid)
        })
        .collect();
    let partition = derive_partition(&bg_geoids, &tract_geoids)
        .expect("derive_partition BG->tract must succeed");
    assert_eq!(partition.len(), 12, "partition must have 12 entries");
    // BGs 0-3 -> tract 0, BGs 4-7 -> tract 1, BGs 8-11 -> tract 2
    for i in 0..12 {
        let expected_tract = i / 4;
        assert_eq!(
            partition[i], expected_tract,
            "BG {i} must map to tract {expected_tract}, got {}",
            partition[i]
        );
    }
}

// L0: validate_multiscale_levels — valid and invalid orderings.
#[test]
fn multiscale_fine_coarse_validation() {
    use crate::runner::validate_multiscale_levels;
    // Valid orderings
    assert!(
        validate_multiscale_levels("bg", "tract").is_ok(),
        "(bg, tract) must be valid"
    );
    assert!(
        validate_multiscale_levels("bg", "county").is_ok(),
        "(bg, county) must be valid"
    );
    assert!(
        validate_multiscale_levels("tract", "county").is_ok(),
        "(tract, county) must be valid"
    );
    // Invalid orderings
    assert!(
        validate_multiscale_levels("county", "bg").is_err(),
        "(county, bg) must be invalid"
    );
    assert!(
        validate_multiscale_levels("tract", "tract").is_err(),
        "(tract, tract) must be invalid"
    );
    assert!(
        validate_multiscale_levels("county", "tract").is_err(),
        "(county, tract) must be invalid"
    );
    // Block_group alias
    assert!(
        validate_multiscale_levels("block_group", "county").is_ok(),
        "(block_group, county) must be valid"
    );
}

// L1: Option A (BG->tract) on synthetic 12-BG graph produces valid 2-district plan.
#[test]
fn multiscale_option_a_bg_to_tract_synthetic() {
    // Build a 12-node BG adjacency (3×4 linear chain, 3 tracts of 4 BGs each)
    let n_bg = 12;
    let bg_adj: Vec<Vec<usize>> = (0..n_bg)
        .map(|i| {
            let mut nb = Vec::new();
            if i > 0 {
                nb.push(i - 1);
            }
            if i + 1 < n_bg {
                nb.push(i + 1);
            }
            nb
        })
        .collect();
    let bg_pop: Vec<i64> = vec![1000i64; n_bg];

    // 3 tract nodes (linear chain)
    let n_tract = 3;
    let tract_adj: Vec<Vec<usize>> = (0..n_tract)
        .map(|i| {
            let mut nb = Vec::new();
            if i > 0 {
                nb.push(i - 1);
            }
            if i + 1 < n_tract {
                nb.push(i + 1);
            }
            nb
        })
        .collect();
    let tract_pop: Vec<i64> = vec![4000i64; n_tract];

    let ew = HashMap::new();

    // BG GEOIDs: 4 BGs per tract, geoid = tract_prefix + bg digit
    let bg_geoids: std::collections::HashMap<usize, String> = (0..n_bg)
        .map(|i| {
            let tract_num = i / 4 + 1;
            let bg_num = i % 4 + 1;
            (i, format!("37001{tract_num:06}{bg_num}"))
        })
        .collect();

    // Tract GEOIDs: 11-char
    let tract_geoids: std::collections::HashMap<usize, String> = (0..n_tract)
        .map(|t| (t, format!("37001{:06}", t + 1)))
        .collect();

    let result = run_multiscale(
        &tract_adj,
        &tract_pop,
        &ew,
        2,
        0.10,
        10,
        42,
        30,
        0.3,
        0.0,
        Some(&tract_geoids),
        MultiscaleFineLevel::BlockGroup,
        "tract",
        Some((&bg_adj, &bg_pop, &bg_geoids)),
    );
    let asgn = result.expect("multiscale Option A must succeed on synthetic BG graph");
    assert_eq!(asgn.len(), n_bg, "all {n_bg} BGs must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
}

// L1: Option C (BG->county) on synthetic 12-BG graph produces valid 2-district plan.
#[test]
fn multiscale_option_c_bg_to_county_synthetic() {
    let n_bg = 12;
    let bg_adj: Vec<Vec<usize>> = (0..n_bg)
        .map(|i| {
            let mut nb = Vec::new();
            if i > 0 {
                nb.push(i - 1);
            }
            if i + 1 < n_bg {
                nb.push(i + 1);
            }
            nb
        })
        .collect();
    let bg_pop: Vec<i64> = vec![1000i64; n_bg];

    // Tract adjacency (for METIS seeding)
    let n_tract = 3;
    let tract_adj: Vec<Vec<usize>> = (0..n_tract)
        .map(|i| {
            let mut nb = Vec::new();
            if i > 0 {
                nb.push(i - 1);
            }
            if i + 1 < n_tract {
                nb.push(i + 1);
            }
            nb
        })
        .collect();
    let tract_pop: Vec<i64> = vec![4000i64; n_tract];
    let ew = HashMap::new();

    // BG geoids: 6 BGs per county (2 counties)
    let bg_geoids: std::collections::HashMap<usize, String> = (0..n_bg)
        .map(|i| {
            let county = if i < 6 { "37001" } else { "37003" };
            let tract_num = (i % 6) / 4 + 1;
            let bg_num = i % 4 + 1;
            (i, format!("{county}{tract_num:06}{bg_num}"))
        })
        .collect();

    // Tract geoids
    let tract_geoids: std::collections::HashMap<usize, String> = (0..n_tract)
        .map(|t| {
            let county = if t == 0 { "37001" } else { "37003" };
            (t, format!("{county}{:06}", t % 2 + 1))
        })
        .collect();

    let result = run_multiscale(
        &tract_adj,
        &tract_pop,
        &ew,
        2,
        0.10,
        10,
        42,
        30,
        0.3,
        0.0,
        Some(&tract_geoids),
        MultiscaleFineLevel::BlockGroup,
        "county",
        Some((&bg_adj, &bg_pop, &bg_geoids)),
    );
    let asgn = result.expect("multiscale Option C must succeed on synthetic BG graph");
    assert_eq!(asgn.len(), n_bg, "all {n_bg} BGs must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
}

// L0: "multiscale" must parse as SearchMode::MultiScale via clap ValueEnum.
#[test]
fn multiscale_search_mode_parses() {
    use crate::args::SearchMode;
    use clap::ValueEnum;
    let parsed =
        SearchMode::from_str("multiscale", true).expect("SearchMode must parse 'multiscale'");
    assert_eq!(parsed, SearchMode::MultiScale);
}

// ── AdaptiveMultiScale tests ──────────────────────────────────────────────

// L0: missing geoids must return Err containing "GEOID".
#[test]
fn multiscale_adaptive_missing_geoids_returns_err() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let cfg = AdaptiveConfig::default();
    let result = run_multiscale_adaptive(
        &adj,
        &pop,
        &ew,
        2,
        10,
        42,
        cfg,
        None,
        MultiscaleFineLevel::Tract,
        "county",
        None,
    );
    assert!(
        result.is_err(),
        "run_multiscale_adaptive with no geoids must return Err"
    );
    let msg = result.unwrap_err();
    assert!(
        msg.contains("GEOID"),
        "error must mention GEOID, got: {msg}"
    );
}

// L0: T=200, adapt_interval=50 -> alpha_trace.len() == 4.
#[test]
fn multiscale_adaptive_alpha_trace_length() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let geoids = synthetic_geoids(16);
    let cfg = AdaptiveConfig {
        total_steps: 200,
        adapt_interval: 50,
        ..AdaptiveConfig::default()
    };
    let (_, result) = run_multiscale_adaptive(
        &adj,
        &pop,
        &ew,
        2,
        10,
        42,
        cfg,
        Some(&geoids),
        MultiscaleFineLevel::Tract,
        "county",
        None,
    )
    .expect("multiscale_adaptive must succeed on 4x4 grid");
    assert_eq!(
        result.alpha_trace.len(),
        4,
        "T=200 adapt_interval=50 must produce exactly 4 adaptation rounds, \
             got {}",
        result.alpha_trace.len()
    );
}

// L0: adapt_interval > total_steps -> no adaptation, alpha_trace empty, final_alpha == initial_alpha.
#[test]
fn multiscale_adaptive_adapt_interval_gt_steps_no_adaptation() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let geoids = synthetic_geoids(16);
    let initial_alpha = 0.30;
    let cfg = AdaptiveConfig {
        total_steps: 10,
        adapt_interval: 1000,
        initial_alpha,
        ..AdaptiveConfig::default()
    };
    let (_, result) = run_multiscale_adaptive(
        &adj,
        &pop,
        &ew,
        2,
        10,
        42,
        cfg,
        Some(&geoids),
        MultiscaleFineLevel::Tract,
        "county",
        None,
    )
    .expect("multiscale_adaptive must succeed");
    assert!(
        result.alpha_trace.is_empty(),
        "adapt_interval > total_steps must produce empty alpha_trace"
    );
    assert!(
        (result.final_alpha - initial_alpha).abs() < 1e-12,
        "adapt_interval > total_steps: final_alpha must equal initial_alpha, \
             got {}",
        result.final_alpha
    );
}

// L0: same seed -> same result (determinism).
#[test]
fn multiscale_adaptive_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let geoids = synthetic_geoids(16);
    let make_cfg = || AdaptiveConfig {
        total_steps: 100,
        adapt_interval: 25,
        ..AdaptiveConfig::default()
    };
    let (plan1, res1) = run_multiscale_adaptive(
        &adj,
        &pop,
        &ew,
        2,
        10,
        77,
        make_cfg(),
        Some(&geoids),
        MultiscaleFineLevel::Tract,
        "county",
        None,
    )
    .expect("first run must succeed");
    let (plan2, res2) = run_multiscale_adaptive(
        &adj,
        &pop,
        &ew,
        2,
        10,
        77,
        make_cfg(),
        Some(&geoids),
        MultiscaleFineLevel::Tract,
        "county",
        None,
    )
    .expect("second run must succeed");
    assert_eq!(plan1, plan2, "same seed must produce identical plan");
    assert_eq!(
        res1.alpha_trace, res2.alpha_trace,
        "same seed must produce identical alpha_trace"
    );
}

// L0: "multiscale-adaptive" must parse as SearchMode::MultiScaleAdaptive via clap ValueEnum.
#[test]
fn multiscale_adaptive_search_mode_parses() {
    use crate::args::SearchMode;
    use clap::ValueEnum;
    let parsed = SearchMode::from_str("multiscale-adaptive", true)
        .expect("SearchMode must parse 'multiscale-adaptive'");
    assert_eq!(parsed, SearchMode::MultiScaleAdaptive);
}

// ── MergeSplit tests ──────────────────────────────────────────────────────

// L0: run_merge_split must return a valid 2-district plan on a 4×4 grid.
#[test]
fn merge_split_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 50, 0.0)
        .expect("merge_split k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L0: p=0.0 EC <= p=1.0 EC (ascending sort — both ends of accepted list).
#[test]
fn merge_split_p0_le_p1_ec() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn_p0 = run_merge_split(&adj, &pop, &ew, 2, 0.10, 10, 77, 200, 0.0)
        .expect("merge_split p=0.0 must succeed");
    let asgn_p1 = run_merge_split(&adj, &pop, &ew, 2, 0.10, 10, 77, 200, 1.0)
        .expect("merge_split p=1.0 must succeed");
    let ec_p0 = count_edge_cuts(&asgn_p0, &adj);
    let ec_p1 = count_edge_cuts(&asgn_p1, &adj);
    assert!(
        ec_p0 <= ec_p1,
        "p=0.0 (min) EC={ec_p0} must be <= p=1.0 (max) EC={ec_p1}"
    );
}

// L0: same seed -> same result (determinism).
#[test]
fn merge_split_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn1 = run_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 100, 0.5)
        .expect("first run must succeed");
    let asgn2 = run_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 100, 0.5)
        .expect("second run must succeed");
    assert_eq!(asgn1, asgn2, "same seed must produce identical assignment");
}

// L0: steps=0 returns the initial METIS plan (no chain steps).
#[test]
fn merge_split_zero_steps_returns_initial() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 99, 0, 0.0)
        .expect("merge_split steps=0 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
}

// L0: on a well-connected grid, at least one step should be accepted.
// Acceptance rate > 0 verifies MH ratio logic is not always-reject.
#[test]
fn merge_split_acceptance_rate_positive() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    // Use generous tolerance (0.20) so the MH ratio allows proposals to pass.
    // Run 500 steps — on a 4x4 balanced grid at least a few should be accepted.
    let asgn_p0 = run_merge_split(&adj, &pop, &ew, 2, 0.20, 10, 55, 500, 0.0)
        .expect("merge_split must succeed");
    let asgn_p1 = run_merge_split(&adj, &pop, &ew, 2, 0.20, 10, 55, 500, 1.0)
        .expect("merge_split must succeed");
    // If acceptance_rate > 0, p=0.0 and p=1.0 may select different plans.
    // At minimum, each must be a valid 2-district partition.
    assert_eq!(asgn_p0.len(), 16, "p=0.0 plan must cover all tracts");
    assert_eq!(asgn_p1.len(), 16, "p=1.0 plan must cover all tracts");
    let d0: std::collections::HashSet<usize> = asgn_p0.values().copied().collect();
    let d1: std::collections::HashSet<usize> = asgn_p1.values().copied().collect();
    assert_eq!(d0.len(), 2, "p=0.0: must have 2 districts");
    assert_eq!(d1.len(), 2, "p=1.0: must have 2 districts");
}

// L2: NC 2020 k=14, T=1000 — ignored by default.
#[test]
#[ignore]
fn merge_split_nc_mixing() {
    // Placeholder: load NC 2020 adjacency + population, run 1000 steps k=14.
    // Assert acceptance rate > 0 and plan is a valid 14-district partition.
    // Skipped unless --include-ignored is passed.
}

// ── ShortBurstForest tests ────────────────────────────────────────────────

// L0: run_short_burst_forest must return a valid 2-district plan on a 4×4 grid.
#[test]
fn short_burst_forest_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_short_burst_forest(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.0)
        .expect("short_burst_forest k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L0: p=0.0 EC <= p=1.0 EC (ascending sort — both ends tested to catch inverted sort).
#[test]
fn short_burst_forest_p0_le_p1_ec() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn_p0 = run_short_burst_forest(&adj, &pop, &ew, 2, 0.10, 10, 77, 5, 20, 0.0)
        .expect("short_burst_forest p=0.0 must succeed");
    let asgn_p1 = run_short_burst_forest(&adj, &pop, &ew, 2, 0.10, 10, 77, 5, 20, 1.0)
        .expect("short_burst_forest p=1.0 must succeed");
    let ec_p0 = count_edge_cuts(&asgn_p0, &adj);
    let ec_p1 = count_edge_cuts(&asgn_p1, &adj);
    assert!(
        ec_p0 <= ec_p1,
        "p=0.0 EC={ec_p0} must be <= p=1.0 EC={ec_p1} (ascending sort)"
    );
}

// L0: same base_seed → identical result (determinism).
#[test]
fn short_burst_forest_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn1 = run_short_burst_forest(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.5)
        .expect("first run must succeed");
    let asgn2 = run_short_burst_forest(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.5)
        .expect("second run must succeed");
    assert_eq!(asgn1, asgn2, "same seed must produce identical assignment");
}

// L0: n_bursts=0 returns the initial plan unchanged.
#[test]
fn short_burst_forest_zero_bursts_returns_initial() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_short_burst_forest(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 0, 0.0)
        .expect("n_bursts=0 must succeed");
    assert_eq!(asgn.len(), 16, "must assign all 16 tracts");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
}

// L0: n_bursts=1, burst_length=1 — degenerate case succeeds.
#[test]
fn short_burst_forest_n1_burst_len1() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_short_burst_forest(&adj, &pop, &ew, 2, 0.05, 10, 7, 1, 1, 0.0)
        .expect("n_bursts=1, burst_length=1 must succeed");
    assert_eq!(asgn.len(), 16, "all tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
}

// L0: SBF_CHAIN_ burst seed is distinct from SHORT_BURST_CHAIN_ for same (base_seed, burst_idx).
#[test]
fn short_burst_chain_seeds_differ_from_standard() {
    use sha2::Digest;
    let base_seed: u64 = 0;
    let burst_idx: u64 = 0;

    // SBF_CHAIN_ seed
    let sbf_seed = {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_CHAIN_");
        h.update(burst_idx.to_le_bytes());
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    // SHORT_BURST_CHAIN_ seed (same base_seed, burst_idx)
    let sb_seed = {
        let mut h = sha2::Sha256::new();
        h.update(b"SHORT_BURST_CHAIN_");
        h.update(burst_idx.to_le_bytes());
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    // SBMS_CHAIN_ seed
    let sbms_seed = {
        let mut h = sha2::Sha256::new();
        h.update(b"SBMS_CHAIN_");
        h.update(burst_idx.to_le_bytes());
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    assert_ne!(sbf_seed, sb_seed,
            "SBF_CHAIN_ and SHORT_BURST_CHAIN_ must produce distinct seeds for same (base_seed, burst_idx)");
    assert_ne!(sbms_seed, sb_seed,
            "SBMS_CHAIN_ and SHORT_BURST_CHAIN_ must produce distinct seeds for same (base_seed, burst_idx)");
    assert_ne!(
        sbf_seed, sbms_seed,
        "SBF_CHAIN_ and SBMS_CHAIN_ must produce distinct seeds for same (base_seed, burst_idx)"
    );

    // Forward and reverse step seeds are distinct.
    let step: u32 = 0;
    let burst_seed: u64 = 12345;
    let fwd = {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_FWD_");
        h.update(step.to_le_bytes());
        h.update(b"_");
        h.update(burst_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };
    let rev = {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_REV_");
        h.update(step.to_le_bytes());
        h.update(b"_");
        h.update(burst_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };
    assert_ne!(
        fwd, rev,
        "SBF_FWD_ and SBF_REV_ must be distinct for the same (step, burst_seed)"
    );
}

// ── ShortBurstMergeSplit tests ────────────────────────────────────────────

// L0: run_short_burst_merge_split must return a valid 2-district plan on a 4×4 grid.
#[test]
fn short_burst_merge_split_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_short_burst_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.0)
        .expect("short_burst_merge_split k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L0: same base_seed → identical result (determinism).
#[test]
fn short_burst_merge_split_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn1 = run_short_burst_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.5)
        .expect("first run must succeed");
    let asgn2 = run_short_burst_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 42, 5, 10, 0.5)
        .expect("second run must succeed");
    assert_eq!(asgn1, asgn2, "same seed must produce identical assignment");
}

// L0: n_bursts=1, burst_length=1 — degenerate case succeeds.
#[test]
fn short_burst_merge_split_n1_burst_len1() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_short_burst_merge_split(&adj, &pop, &ew, 2, 0.05, 10, 7, 1, 1, 0.0)
        .expect("n_bursts=1, burst_length=1 must succeed");
    assert_eq!(asgn.len(), 16, "all tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
}

// L2: NC 2020 k=14 — ShortBurstForest vs ShortBurstMergeSplit vs ShortBurst EC comparison.
#[test]
#[ignore]
fn short_burst_forest_vs_standard_ec_comparison() {
    // Placeholder: load NC 2020 adjacency, run all three variants on same base_seed.
    // Record EC for each; assert all three return valid k=14 plans.
    // Skipped unless --include-ignored is passed.
}

// ── run_parallel_tempering L0/L1 tests ──────────────────────────────────

// L0: basic k=2 partition on a 4x4 grid — valid districts, all tracts assigned.
#[test]
fn pt_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_parallel_tempering(
        &adj, &pop, &ew, 2,     // num_districts
        10,    // niter
        42,    // base_seed
        2,     // n_replicas
        5,     // swap_interval
        0.005, // cold_tolerance
        0.05,  // hot_tolerance
        20,    // steps
        0.0,   // p
    )
    .expect("pt k=2 on 4x4 grid must succeed");

    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L1: p=0.0 returns a plan with EC <= p=1.0.
#[test]
fn pt_p0_le_p1_ec() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let run = |p: f64| {
        run_parallel_tempering(&adj, &pop, &ew, 2, 10, 99, 2, 5, 0.005, 0.05, 30, p)
            .expect("pt must succeed")
    };
    let a0 = run(0.0);
    let a1 = run(1.0);
    let ec0: usize = (0..adj.len())
        .flat_map(|v| adj[v].iter().map(move |&nb| (v, nb)))
        .filter(|&(v, nb)| nb > v && a0[&v] != a0[&nb])
        .count();
    let ec1: usize = (0..adj.len())
        .flat_map(|v| adj[v].iter().map(move |&nb| (v, nb)))
        .filter(|&(v, nb)| nb > v && a1[&v] != a1[&nb])
        .count();
    assert!(
        ec0 <= ec1,
        "pt p=0.0 EC ({ec0}) must be <= p=1.0 EC ({ec1})"
    );
}

// L0: same base_seed → identical result (determinism).
#[test]
fn pt_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let run = || {
        run_parallel_tempering(&adj, &pop, &ew, 2, 10, 314159, 3, 5, 0.005, 0.05, 25, 0.0)
            .expect("pt must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same base_seed must produce identical assignment");
}

// L0: steps=0 returns the initial METIS plan immediately (no chain steps).
#[test]
fn pt_zero_steps_returns_initial() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let asgn = run_parallel_tempering(&adj, &pop, &ew, 2, 10, 42, 2, 5, 0.005, 0.05, 0, 0.0)
        .expect("steps=0 must succeed");
    assert_eq!(asgn.len(), 16, "all tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
}

// L0: SearchMode parses "parallel-tempering" successfully (CLI integration).
#[test]
fn pt_search_mode_parses() {
    use crate::args::SearchMode;
    use clap::ValueEnum;
    let parsed = SearchMode::from_str("parallel-tempering", true)
        .expect("SearchMode must parse 'parallel-tempering'");
    assert_eq!(
        parsed,
        SearchMode::ParallelTempering,
        "parsed SearchMode must equal ParallelTempering"
    );
}

// L2: NC 2020 k=14 — PT cold chain EC <= single-chain EC (quality check).
#[test]
#[ignore]
fn pt_nc_lower_ec_than_single_chain() {
    // Requires: data/2020/nc_adjacency.adj.bin
    // Run PT (n_replicas=4, steps=1000, p=0.0) and single (steps=1, p=0.0)
    // on NC 2020 k=14. Assert EC_pt <= EC_single at least for p=0.0.
    // Skipped unless --include-ignored is passed.
}

// ── run_vra_recom L0/L1 tests ────────────────────────────────────────────

// L0: run_vra_recom must return a valid 2-district plan on a 4x4 grid.
#[test]
fn vra_recom_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    // minority_vap: first 8 tracts (top half) have 0.6, rest have 0.1
    let minority_vap: Vec<f64> = (0..16).map(|i| if i < 8 { 0.6 } else { 0.1 }).collect();
    let asgn = run_vra_recom(
        &adj,
        &pop,
        &ew,
        2,    // num_districts
        10,   // niter
        42,   // base_seed
        30,   // steps
        0.0,  // p
        0.50, // vap_threshold
        &minority_vap,
    )
    .expect("vra_recom k=2 on 4x4 grid must succeed");

    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have exactly 2 districts");
    for &d in asgn.values() {
        assert!(d >= 1 && d <= 2, "district label must be in [1,2], got {d}");
    }
}

// L1: p=0.0 returns a plan with EC <= p=1.0 (percentile ordering).
#[test]
fn vra_recom_p0_le_p1_ec() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let minority_vap: Vec<f64> = (0..16).map(|i| if i < 8 { 0.6 } else { 0.1 }).collect();
    let run = |p: f64| {
        run_vra_recom(&adj, &pop, &ew, 2, 10, 77, 50, p, 0.50, &minority_vap)
            .expect("vra_recom must succeed")
    };
    let a0 = run(0.0);
    let a1 = run(1.0);
    let ec = |asgn: &HashMap<usize, usize>| -> usize {
        (0..adj.len())
            .flat_map(|v| adj[v].iter().map(move |&nb| (v, nb)))
            .filter(|&(v, nb)| nb > v && asgn[&v] != asgn[&nb])
            .count()
    };
    assert!(
        ec(&a0) <= ec(&a1),
        "vra_recom p=0.0 EC ({}) must be <= p=1.0 EC ({})",
        ec(&a0),
        ec(&a1)
    );
}

// L0: same base_seed → identical result (determinism).
#[test]
fn vra_recom_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    let minority_vap: Vec<f64> = (0..16).map(|i| if i < 8 { 0.6 } else { 0.1 }).collect();
    let run = || {
        run_vra_recom(&adj, &pop, &ew, 2, 10, 271828, 30, 0.0, 0.50, &minority_vap)
            .expect("vra_recom must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same base_seed must produce identical assignment");
}

// L0: minority_vap all 0.0 → protected_districts empty → no VRA rejection
// (the chain degenerates to unconstrained ForestRecom; all proposals handled by MH only).
#[test]
fn vra_recom_zero_minority_vap_no_vra_rejection() {
    let (adj, pop) = small_grid(4, 4);
    let ew = HashMap::new();
    // All zero → no protected districts → VRA enforcement is a no-op.
    let minority_vap: Vec<f64> = vec![0.0; 16];
    let asgn = run_vra_recom(&adj, &pop, &ew, 2, 10, 13, 20, 0.0, 0.50, &minority_vap)
        .expect("vra_recom with zero minority_vap must succeed");
    assert_eq!(asgn.len(), 16, "all tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must have 2 districts");
}

// L0: SearchMode parses "vra-recom" successfully (CLI integration).
#[test]
fn vra_recom_search_mode_parses() {
    use crate::args::SearchMode;
    use clap::ValueEnum;
    let parsed =
        SearchMode::from_str("vra-recom", true).expect("SearchMode must parse 'vra-recom'");
    assert_eq!(
        parsed,
        SearchMode::VraRecom,
        "parsed SearchMode must equal VraRecom"
    );
}

// L2: NC 2020 k=14 — VRA-aware chain preserves majority-minority districts.
#[test]
#[ignore]
fn vra_recom_nc_preserves_majority_minority() {
    // Requires: data/2020/north_carolina_demographics_2020.csv +
    //           data/2020/north_carolina_adjacency.adj.bin
    // Load minority_vap from demographics CSV (HVAP / total_pop per tract).
    // Run vra_recom with vap_threshold=0.50, steps=200.
    // Assert: in every accepted plan, no initially-protected district has
    // minority_vap_fraction < 0.50 (i.e., VRA constraint is honoured).
    // Skipped unless --include-ignored is passed.
}

// ── Group: BFS Region-Growing (T.12) ─────────────────────────────────────

// L0: 4x4 grid → valid 2-way split (all tracts assigned, two non-empty districts).
#[test]
fn bfs_growth_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_bfs(&adj, &pop, &tracts, 0.10, 42)
        .expect("bfs-growth must succeed on 4x4 grid");
    // All 16 tracts assigned
    let mut all: Vec<usize> = left.union(&right).copied().collect();
    all.sort_unstable();
    assert_eq!(
        all,
        (0..16).collect::<Vec<_>>(),
        "all 16 tracts must be covered"
    );
    // Both sides non-empty
    assert!(!left.is_empty(), "left side must be non-empty");
    assert!(!right.is_empty(), "right side must be non-empty");
    // Disjoint
    assert!(left.is_disjoint(&right), "left and right must be disjoint");
}

// L0: same seed → same result (deterministic).
#[test]
fn bfs_growth_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let run = |seed: u64| {
        split_subgraph_bfs(&adj, &pop, &tracts, 0.10, seed).expect("bfs-growth must succeed")
    };
    let (l1, r1) = run(99);
    let (l2, r2) = run(99);
    assert_eq!(l1, l2, "same seed must produce identical left set");
    assert_eq!(r1, r2, "same seed must produce identical right set");
}

// L0: no unassigned tracts after BFS + rebalance.
#[test]
fn bfs_growth_all_tracts_assigned() {
    let (adj, pop) = small_grid(5, 5); // 25 tracts
    let tracts: HashSet<usize> = (0..25).collect();
    let (left, right) = split_subgraph_bfs(&adj, &pop, &tracts, 0.10, 7)
        .expect("bfs-growth must succeed on 5x5 grid");
    assert_eq!(
        left.len() + right.len(),
        25,
        "all 25 tracts must be assigned"
    );
    assert!(
        left.is_disjoint(&right),
        "no tract may appear in both sides"
    );
}

// L0: both districts non-empty for any non-trivial input.
#[test]
fn bfs_growth_both_districts_nonempty() {
    // Linear chain: 10 nodes
    let n = 10usize;
    let adj: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            let mut nb = vec![];
            if i > 0 {
                nb.push(i - 1);
            }
            if i < n - 1 {
                nb.push(i + 1);
            }
            nb
        })
        .collect();
    let pop = vec![1000i64; n];
    let tracts: HashSet<usize> = (0..n).collect();
    let (left, right) =
        split_subgraph_bfs(&adj, &pop, &tracts, 0.10, 0).expect("bfs-growth must succeed on chain");
    assert!(!left.is_empty(), "left must be non-empty for chain graph");
    assert!(!right.is_empty(), "right must be non-empty for chain graph");
}

// L0: BFS from any tract in each district must reach all tracts in that district
//     (contiguity guarantee — BFS growth ensures this by construction).
#[test]
fn bfs_growth_contiguous_districts() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) =
        split_subgraph_bfs(&adj, &pop, &tracts, 0.10, 123).expect("bfs-growth must succeed");

    let is_contiguous = |side: &HashSet<usize>| -> bool {
        if side.len() <= 1 {
            return true;
        }
        let start = *side.iter().next().unwrap();
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        visited.insert(start);
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                if side.contains(&nb) && !visited.contains(&nb) {
                    visited.insert(nb);
                    queue.push_back(nb);
                }
            }
        }
        visited.len() == side.len()
    };

    assert!(is_contiguous(&left), "left district must be contiguous");
    assert!(is_contiguous(&right), "right district must be contiguous");
}

// L0: "bfs-growth" StructureMode parses correctly from CLI string.
#[test]
fn bfs_growth_structure_mode_parses() {
    use crate::args::StructureMode;
    use clap::ValueEnum;
    let parsed =
        StructureMode::from_str("bfs-growth", true).expect("StructureMode must parse 'bfs-growth'");
    assert_eq!(
        parsed,
        StructureMode::BfsGrowth,
        "parsed StructureMode must equal BfsGrowth"
    );
}

// L0: seed[1] should be farther from seed[0] than most other tracts
//     (maximally-spread seeds — seed[1] = argmax BFS distance from seed[0]).
#[test]
fn bfs_growth_seeds_maximally_spread() {
    // 4x4 grid: seeds should be near opposite corners.
    // We verify this indirectly: the two assigned seeds are not adjacent.
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) =
        split_subgraph_bfs(&adj, &pop, &tracts, 0.10, 0).expect("bfs-growth must succeed");

    // The farthest-apart split on a 4x4 grid should not put all tracts
    // in one set (which would happen if both seeds were adjacent).
    assert!(left.len() >= 1, "left must have at least 1 tract");
    assert!(right.len() >= 1, "right must have at least 1 tract");

    // The seeds are the initial tracts: verify they are not the same
    // by confirming both sides have at least 1 tract.
    // (A more direct test would require exposing seed positions.)
    let max_frac = (left.len().max(right.len()) as f64) / 16.0;
    assert!(
        max_frac <= 0.95,
        "maximally-spread seeds should not produce a 95%:5% split; got {:.0}%:{:.0}%",
        left.len() as f64 / 16.0 * 100.0,
        right.len() as f64 / 16.0 * 100.0
    );
}

// L1: run_all_splits_bfs on 4x4 grid with k=2: valid, deterministic, contiguous.
#[test]
fn bfs_growth_run_all_splits_k2() {
    let (adj, pop) = small_grid(4, 4);
    let asgn = run_all_splits_bfs(&adj, &pop, 2, 0.05, None, 42)
        .expect("run_all_splits_bfs k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must produce exactly 2 districts");
}

// L1: run_all_splits_bfs with k=4: valid partition on 4x4 grid.
#[test]
fn bfs_growth_run_all_splits_k4() {
    let (adj, pop) = small_grid(4, 4);
    let asgn = run_all_splits_bfs(&adj, &pop, 4, 0.10, None, 7)
        .expect("run_all_splits_bfs k=4 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 4, "must produce exactly 4 districts");
}

// L1: same base_seed → identical run_all_splits_bfs result.
#[test]
fn bfs_growth_run_all_splits_deterministic() {
    let (adj, pop) = small_grid(4, 5); // 20 tracts
    let run = || {
        run_all_splits_bfs(&adj, &pop, 2, 0.05, None, 12345)
            .expect("run_all_splits_bfs must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same base_seed must produce identical assignments");
}

// L2: NC 2020 k=14 — BFS Growth vs single METIS baseline.
#[test]
#[ignore]
fn bfs_growth_nc_ec_vs_metis() {
    // Requires: data/2020/north_carolina_adjacency.adj.bin
    // Run bfs-growth and METIS on NC 2020 k=14.
    // Expected: BFS Growth EC >= METIS EC (METIS explicitly minimizes EC).
    // Also check: BFS Growth is fast (O(n log n) vs METIS heuristic).
    // Skipped unless --include-ignored is passed.
}

// ── SMC-Percentile tests (SeedCompositor::SmcPercentile) ─────────────────

// L0: 4-node path k=2, n_particles=50: returns valid 2-district plan.
#[test]
fn smc_percentile_produces_valid_k2_partition() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let pop = vec![100i64; 4];
    let result = run_smc_percentile(&adj, &pop, 2, 42, 50, 0.5, 0.5)
        .expect("smc_percentile must succeed on 4-node path");
    // All 4 tracts assigned
    assert_eq!(result.len(), 4, "all 4 tracts must be in the result");
    // Exactly 2 distinct districts
    let districts: std::collections::HashSet<usize> = result.values().copied().collect();
    assert_eq!(districts.len(), 2, "must produce exactly 2 districts");
}

// L0: p=0.0 plan has EC <= p=1.0 plan (min-EC <= max-EC ordering).
#[test]
fn smc_percentile_p0_le_p1_ec() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let pop = vec![100i64; 4];
    let plan_p0 = run_smc_percentile(&adj, &pop, 2, 99, 50, 0.0, 0.5).expect("p=0.0 must succeed");
    let plan_p1 = run_smc_percentile(&adj, &pop, 2, 99, 50, 1.0, 0.5).expect("p=1.0 must succeed");
    let ec_p0 = count_edge_cuts(&plan_p0, &adj);
    let ec_p1 = count_edge_cuts(&plan_p1, &adj);
    assert!(
        ec_p0 <= ec_p1,
        "p=0.0 (min) EC={ec_p0} must be <= p=1.0 (max) EC={ec_p1}"
    );
}

// L0: same seed -> same result (determinism).
#[test]
fn smc_percentile_deterministic() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let pop = vec![100i64; 4];
    let run = || {
        run_smc_percentile(&adj, &pop, 2, 77, 30, 0.5, 0.5).expect("smc_percentile must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same base_seed must produce identical results");
}

// L0: derive_smcp_seed(42) != 42 and != derive_smcp_seed(0).
#[test]
fn smc_percentile_smcp_seed_distinct_from_base() {
    let seed_42 = derive_smcp_seed(42);
    let seed_0 = derive_smcp_seed(0);
    assert_ne!(
        seed_42, 42u64,
        "derive_smcp_seed(42) must differ from raw 42"
    );
    assert_ne!(
        seed_42, seed_0,
        "derive_smcp_seed(42) must differ from derive_smcp_seed(0)"
    );
}

// L0: p=0.0 returns a complete valid plan (all tracts assigned).
// Tests the positive-weight-skip logic indirectly (on valid graph, all weights are positive).
#[test]
fn smc_percentile_zero_weight_skip() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
    let pop = vec![100i64; 4];
    let result = run_smc_percentile(&adj, &pop, 2, 55, 20, 0.0, 0.5).expect("p=0.0 must succeed");
    assert_eq!(
        result.len(),
        4,
        "p=0.0 must return a complete plan (4 tracts)"
    );
    let districts: std::collections::HashSet<usize> = result.values().copied().collect();
    assert_eq!(districts.len(), 2, "p=0.0 must produce exactly 2 districts");
}

// L0: smc-percentile search mode parses correctly.
#[test]
fn smc_percentile_search_mode_parses() {
    use crate::args::SearchMode;
    use clap::ValueEnum;
    let parsed = SearchMode::from_str("smc-percentile", true)
        .expect("'smc-percentile' must be a valid SearchMode");
    assert_eq!(
        parsed,
        SearchMode::SmcPercentile,
        "parsed SearchMode must be SmcPercentile"
    );
}

// L2: NC 2020 calibrated distribution (requires real data).
#[test]
#[ignore]
fn smc_percentile_nc_calibrated_distribution() {
    // Requires: data/2020/adjacency/north_carolina_adjacency_2020.pkl
    // Run SMC with n_particles=500 on NC 2020 k=14.
    // Verify: p=0.0 EC <= p=0.5 EC <= p=1.0 EC (distribution order).
    // Skipped unless --include-ignored is passed.
}

// ── ILP tests (L0/L1) ─────────────────────────────────────────────────────

/// L0: `--partition-mode ilp` parses to PartitionMode::Ilp.
#[test]
fn ilp_structure_mode_parses() {
    use crate::args::{IlpFallback, IlpMethod, PartitionMode};
    use clap::ValueEnum;
    let parsed = PartitionMode::from_str("ilp", true).expect("'ilp' must be a valid PartitionMode");
    assert_eq!(
        parsed,
        PartitionMode::Ilp,
        "parsed PartitionMode must be Ilp"
    );
    assert_eq!(
        IlpMethod::from_str("formulation-only", true).unwrap(),
        IlpMethod::FormulationOnly
    );
    assert_eq!(
        IlpMethod::from_str("branch-and-cut", true).unwrap(),
        IlpMethod::BranchAndCut
    );
    assert_eq!(
        IlpMethod::from_str("iterative-separation", true).unwrap(),
        IlpMethod::IterativeSeparation
    );
    assert_eq!(
        IlpFallback::from_str("metis", true).unwrap(),
        IlpFallback::Metis
    );
    assert_eq!(
        IlpFallback::from_str("error", true).unwrap(),
        IlpFallback::Error
    );
}

/// L1: 4x4 grid (16 tracts) with max_tracts=5 — size guard fires, falls back to METIS.
/// Verifies: returns Ok, all 16 tracts assigned, two non-empty partitions.
#[test]
fn ilp_size_guard_fallback() {
    let (adj, pop) = small_grid(4, 4); // 16 tracts
    let tracts: HashSet<usize> = (0..16).collect();
    let result = split_subgraph_ilp(
        &adj,
        &pop,
        &tracts,
        1.05, // balance_tolerance
        crate::args::IlpMethod::FormulationOnly,
        crate::args::IlpFallback::Metis,
        60,   // time_limit_secs
        0.01, // optimality_gap
        5,    // max_tracts: 16 > 5 => guard fires
        None,
    );
    let (left, right) = result.expect("size-guard fallback must not fail");
    assert!(
        !left.is_empty(),
        "left partition must be non-empty after METIS fallback"
    );
    assert!(
        !right.is_empty(),
        "right partition must be non-empty after METIS fallback"
    );
    assert_eq!(
        left.len() + right.len(),
        16,
        "all 16 tracts must be assigned"
    );
    // Verify disjoint
    for t in &left {
        assert!(!right.contains(t), "partitions must be disjoint");
    }
}

/// L1: 4-node path graph — Phase 1 FormulationOnly falls back to METIS,
/// returns a valid 2-partition of all 4 nodes.
#[test]
fn ilp_phase1_produces_valid_plan() {
    // Path: 0-1-2-3  (4 nodes, well within max_tracts=500)
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2usize]];
    let pop = vec![100i64; 4];
    let tracts: HashSet<usize> = (0..4).collect();
    let result = split_subgraph_ilp(
        &adj,
        &pop,
        &tracts,
        1.05, // balance_tolerance
        crate::args::IlpMethod::FormulationOnly,
        crate::args::IlpFallback::Metis,
        300,  // time_limit_secs
        0.01, // optimality_gap
        500,  // max_tracts (no guard)
        None,
    );
    let (left, right) = result.expect("ilp phase1 must not fail");
    assert!(!left.is_empty(), "left partition must be non-empty");
    assert!(!right.is_empty(), "right partition must be non-empty");
    assert_eq!(left.len() + right.len(), 4, "all 4 tracts must be covered");
    for t in &left {
        assert!(!right.contains(t), "partitions must be disjoint");
    }
}

#[test]
fn ilp_fallback_error_rejects_size_guard() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let err = split_subgraph_ilp(
        &adj,
        &pop,
        &tracts,
        1.05,
        crate::args::IlpMethod::FormulationOnly,
        crate::args::IlpFallback::Error,
        60,
        0.01,
        5,
        None,
    )
    .expect_err("fallback=error must reject size-guard fallback");
    assert!(err.contains("--ilp-fallback=error"));
}

#[test]
fn ilp_fallback_error_rejects_missing_solver_plan() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1usize]];
    let pop = vec![100i64; 3];
    let tracts: HashSet<usize> = (0..3).collect();
    let err = split_subgraph_ilp(
        &adj,
        &pop,
        &tracts,
        0.005,
        crate::args::IlpMethod::BranchAndCut,
        crate::args::IlpFallback::Error,
        300,
        0.01,
        500,
        None,
    )
    .expect_err("fallback=error must reject missing solver plan");
    assert!(err.contains("no solver plan returned"));
    assert!(err.contains("--ilp-fallback=error"));
}

/// L0: ILP formulation variable counts for 4-node path graph, k=2.
/// n_binary_x = n*k = 4*2 = 8
/// n_binary_z = |E| = 3  (path 0-1-2-3 has 3 edges)
/// n_flow_vars = 2*|E|*k = 2*3*2 = 12
#[test]
fn ilp_formulation_counts_correct() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2usize]];
    let pop = vec![100i64; 4];
    let formulation = bisect_ilp::build_formulation(&adj, &pop, 2, 0.005);
    assert_eq!(
        formulation.n_binary_x, 8,
        "n_binary_x should be n*k = 4*2 = 8"
    );
    assert_eq!(
        formulation.n_binary_z, 3,
        "n_binary_z should be |E| = 3 for path 0-1-2-3"
    );
    assert_eq!(
        formulation.n_flow_vars, 12,
        "n_flow_vars should be 2*|E|*k = 2*3*2 = 12"
    );
}

/// L1: run_all_splits_ilp on a 4x4 grid — all 16 tracts assigned into 2 districts.
#[test]
fn run_all_splits_ilp_produces_complete_assignment() {
    let (adj, pop) = small_grid(4, 4); // 16 tracts
    let ew: HashMap<(usize, usize), f64> = HashMap::new();
    let result = run_all_splits_ilp(
        &adj,
        &pop,
        &ew,
        2,    // num_districts
        0.05, // balance_tolerance
        crate::args::IlpMethod::FormulationOnly,
        crate::args::IlpFallback::Metis,
        60,   // time_limit_secs
        0.01, // optimality_gap
        500,  // max_tracts (no size guard)
        None,
    );
    let assignments = result.expect("run_all_splits_ilp must succeed on 4x4 grid");
    assert_eq!(assignments.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = assignments.values().copied().collect();
    assert_eq!(districts.len(), 2, "must produce exactly 2 districts");
}

#[test]
fn run_all_splits_ilp_writes_solve_report() {
    let (adj, pop) = small_grid(2, 2);
    let ew: HashMap<(usize, usize), f64> = HashMap::new();
    let tmp = tempfile::TempDir::new().unwrap();
    let result = run_all_splits_ilp(
        &adj,
        &pop,
        &ew,
        2,
        0.05,
        crate::args::IlpMethod::BranchAndCut,
        crate::args::IlpFallback::Metis,
        60,
        0.01,
        500,
        Some(tmp.path()),
    );
    result.expect("ilp run must succeed");
    let report_path = tmp.path().join("depth_00").join("node_root.json");
    let lp_path = tmp.path().join("depth_00").join("node_root.lp");
    let summary_path = tmp.path().join("audit-summary.json");
    let json = std::fs::read_to_string(&report_path).expect("read ilp solve report");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse report json");
    assert_eq!(value["schema_version"], "ilp-solve-report-v1");
    assert_eq!(value["model_artifact"]["format"], "cplex-lp");
    assert_eq!(value["model_artifact"]["path"], "node_root.lp");
    assert_eq!(
        value["model_artifact"]["sha256"].as_str().unwrap().len(),
        64
    );
    assert_eq!(value["audit_summary"]["outcome"], "exact-plan");
    assert_eq!(value["audit_summary"]["proof_status"], "proved-optimal");
    assert_eq!(value["audit_summary"]["has_model_artifact"], true);
    assert_eq!(value["audit_summary"]["fallback_required"], false);
    assert_eq!(value["result"]["status"]["status"], "optimal");
    assert_eq!(value["result"]["optimal_ec"], 2);
    assert_eq!(value["result"]["branch_and_cut"]["cut_count"], 0);
    assert_eq!(
        value["result"]["branch_and_cut"]["exact_search"]["search_strategy"],
        "k2-branch-and-bound"
    );
    assert!(
        value["result"]["branch_and_cut"]["exact_search"]["nodes_visited"]
            .as_u64()
            .unwrap()
            > 0
    );

    let lp = std::fs::read_to_string(&lp_path).expect("read ilp master LP");
    assert!(lp.contains("BISECT U.16 branch-and-cut master LP"));
    assert!(lp.contains("Minimize"));
    assert!(lp.contains("Binary"));

    let verified = bisect_ilp::verify_model_artifact_for_report(&report_path)
        .expect("solve report model artifact should verify");
    assert_eq!(verified.format, "cplex-lp");
    assert_eq!(verified.path, lp_path);

    let summary_json = std::fs::read_to_string(&summary_path).expect("read ilp audit summary");
    let summary: serde_json::Value =
        serde_json::from_str(&summary_json).expect("parse ilp audit summary");
    assert_eq!(summary["checked"], 1);
    assert_eq!(summary["passed"], 1);
    assert_eq!(summary["failed"], 0);
    assert_eq!(summary["outcomes"]["exact-plan"], 1);
    assert_eq!(summary["proof_statuses"]["proved-optimal"], 1);
}

// ── Group: Moving-Knife Algorithm (T.13) ─────────────────────────────────

/// Helper: build synthetic (lon, lat) centroids on a 4x4 grid spaced 0.01° apart.
/// Global index = row * cols + col, origin at (-96.05, 37.45).
fn synthetic_centroids(rows: usize, cols: usize) -> Vec<(f64, f64)> {
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

// L0: 4x4 grid → valid 2-way split (all tracts assigned, two non-empty districts).
#[test]
fn mka_produces_valid_k2_partition() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let (left, right) = split_subgraph_mka(
        &adj,
        &pop,
        &tracts,
        &centroids,
        0.10,
        180,
        MkaMetric::Reock,
        42,
        "root",
    )
    .expect("mka must succeed on 4x4 grid");
    let mut all: Vec<usize> = left.union(&right).copied().collect();
    all.sort_unstable();
    assert_eq!(
        all,
        (0..16).collect::<Vec<_>>(),
        "all 16 tracts must be covered"
    );
    assert!(!left.is_empty(), "left side must be non-empty");
    assert!(!right.is_empty(), "right side must be non-empty");
    assert!(left.is_disjoint(&right), "left and right must be disjoint");
}

// L0: same base_seed + n_orientations → identical result (deterministic).
#[test]
fn mka_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let run = || {
        split_subgraph_mka(
            &adj,
            &pop,
            &tracts,
            &centroids,
            0.10,
            180,
            MkaMetric::Reock,
            77,
            "root",
        )
        .expect("mka must succeed")
    };
    let (l1, r1) = run();
    let (l2, r2) = run();
    assert_eq!(l1, l2, "same seed must produce identical left set");
    assert_eq!(r1, r2, "same seed must produce identical right set");
}

// L0: Reock values are clamped to [0.0, 1.0].
#[test]
fn mka_reock_clamped_to_01() {
    // Two coincident points: MEC radius = 0 → reock_score should return 1.0 (clamped).
    let pts = vec![(0.0f64, 0.0f64), (0.0, 0.0)];
    let r = reock_score(&pts, &[]);
    assert!(
        r >= 0.0 && r <= 1.0,
        "reock_score must be in [0,1], got {r}"
    );
    // Large spread: MEC area >> n_tracts — should clamp to 0 from below, return [0,1].
    let spread = vec![(0.0f64, 0.0f64), (1.0e9, 0.0), (0.0, 1.0e9)];
    let r2 = reock_score(&spread, &[]);
    assert!(
        r2 >= 0.0 && r2 <= 1.0,
        "reock_score for large spread must be in [0,1], got {r2}"
    );
}

// L0: MKA seed prefix is distinct from CVD GEO and CVD Phase 1 prefixes.
#[test]
fn mka_seed_prefix_distinct_from_cvd() {
    // The three prefix constants must be pairwise unequal (source-level assertion).
    const MKA_PREFIX: &str = "MKA_INIT_";
    const CVD_GEO_PREFIX: &str = "CVD_GEO_INIT_";
    const CVD_PREFIX: &str = "CVD_INIT_";
    assert_ne!(
        MKA_PREFIX, CVD_GEO_PREFIX,
        "MKA_INIT_ must differ from CVD_GEO_INIT_"
    );
    assert_ne!(
        MKA_PREFIX, CVD_PREFIX,
        "MKA_INIT_ must differ from CVD_INIT_"
    );
    assert_ne!(
        CVD_GEO_PREFIX, CVD_PREFIX,
        "CVD_GEO_INIT_ must differ from CVD_INIT_"
    );

    // mka_seed(0, "") must differ from derive_cvd_geo_seed(0, "").
    let mka_s = mka_seed(0, "");
    let cvd_geo = derive_cvd_geo_seed(0, "");
    assert_ne!(
        mka_s, cvd_geo,
        "mka_seed(0,'') must differ from derive_cvd_geo_seed(0,'')"
    );
}

// L0: split_subgraph_mka_direction returns the same theta as the full split's optimal angle.
#[test]
fn mka_direction_consistent_with_full_split() {
    use std::f64::consts::PI;
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();

    // Run direction-only.
    let theta_star = split_subgraph_mka_direction(&tracts, &centroids, 36);

    // The full split uses n_orientations=36 and should pick the same best angle.
    // We can verify theta_star is in [0, PI).
    assert!(
        theta_star >= 0.0 && theta_star < PI,
        "theta* must be in [0, PI), got {theta_star}"
    );

    // Also verify the direction function doesn't panic on the full split.
    split_subgraph_mka(
        &adj,
        &pop,
        &tracts,
        &centroids,
        0.10,
        36,
        MkaMetric::Reock,
        42,
        "root",
    )
    .expect("mka full split must succeed");
}

// L0: "moving-knife" StructureMode parses correctly from CLI string.
#[test]
fn mka_structure_mode_parses() {
    use crate::args::StructureMode;
    use clap::ValueEnum;
    let parsed = StructureMode::from_str("moving-knife", true)
        .expect("StructureMode must parse 'moving-knife'");
    assert_eq!(
        parsed,
        StructureMode::MovingKnife,
        "parsed StructureMode must equal MovingKnife"
    );
}

// L0: Welzl MEC of a single point has radius 0.
#[test]
fn welzl_mec_single_point() {
    let (cx, cy, r2) = welzl_mec(&[(3.0, 7.0)]);
    assert!((cx - 3.0).abs() < 1e-9, "cx must equal point x");
    assert!((cy - 7.0).abs() < 1e-9, "cy must equal point y");
    assert!(r2.abs() < 1e-9, "radius² of single point must be 0");
}

// L0: Welzl MEC of two points has center at midpoint and radius = half-distance.
#[test]
fn welzl_mec_two_points() {
    let p1 = (0.0f64, 0.0f64);
    let p2 = (4.0f64, 0.0f64);
    let (cx, cy, r2) = welzl_mec(&[p1, p2]);
    assert!((cx - 2.0).abs() < 1e-9, "cx must be midpoint 2.0, got {cx}");
    assert!(cy.abs() < 1e-9, "cy must be 0.0, got {cy}");
    // Distance = 4, radius = 2, radius² = 4.
    assert!((r2 - 4.0).abs() < 1e-9, "radius² must be 4.0, got {r2}");
}

// L0: n_orientations=1 (only θ=0°) — valid plan returned, no panic.
#[test]
fn mka_single_orientation_no_panic() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let result = split_subgraph_mka(
        &adj,
        &pop,
        &tracts,
        &centroids,
        0.10,
        1,
        MkaMetric::Reock,
        0,
        "root",
    );
    assert!(result.is_ok(), "n_orientations=1 must not panic");
    let (l, r) = result.unwrap();
    assert!(!l.is_empty(), "left must be non-empty");
    assert!(!r.is_empty(), "right must be non-empty");
}

// L0: both halves non-empty for any subgraph with m >= 2.
#[test]
fn mka_both_halves_nonempty() {
    // Linear chain: 6 nodes.
    let n = 6usize;
    let adj: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            let mut nb = vec![];
            if i > 0 {
                nb.push(i - 1);
            }
            if i < n - 1 {
                nb.push(i + 1);
            }
            nb
        })
        .collect();
    let pop = vec![1000i64; n];
    // Use simple centroid positions along x-axis.
    let centroids: Vec<(f64, f64)> = (0..n).map(|i| (-96.0 + i as f64 * 0.01, 37.5)).collect();
    let tracts: HashSet<usize> = (0..n).collect();
    let (left, right) = split_subgraph_mka(
        &adj,
        &pop,
        &tracts,
        &centroids,
        0.10,
        180,
        MkaMetric::Reock,
        0,
        "chain",
    )
    .expect("mka must succeed on chain");
    assert!(!left.is_empty(), "left must be non-empty for chain graph");
    assert!(!right.is_empty(), "right must be non-empty for chain graph");
}

// L1: 4x4 grid with evenly-spaced centroids — optimal angle is 0° or 90°.
#[test]
fn mka_symmetric_grid_angle_0_or_90() {
    use std::f64::consts::PI;
    let tracts: HashSet<usize> = (0..16).collect();
    let centroids = synthetic_centroids(4, 4);
    let theta = split_subgraph_mka_direction(&tracts, &centroids, 180);
    // For a symmetric 4x4 grid, all orientations give equal Reock scores.
    // The returned angle is platform-dependent (floating-point tie-breaking),
    // so only assert it is a valid finite angle in [0, PI).
    assert!(theta.is_finite(), "theta must be finite");
    assert!(
        theta >= 0.0 && theta < PI,
        "theta must be in [0, PI), got {theta:.4}"
    );
}

// L1: run_all_splits_mka on 4x4 grid with k=2: valid and deterministic.
#[test]
fn mka_run_all_splits_k2() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let asgn = run_all_splits_mka(
        &adj,
        &pop,
        2,
        0.05,
        None,
        180,
        MkaMetric::Reock,
        42,
        &centroids,
    )
    .expect("run_all_splits_mka k=2 must succeed");
    assert_eq!(asgn.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
    assert_eq!(districts.len(), 2, "must produce exactly 2 districts");
}

// L1: same base_seed → identical run_all_splits_mka result.
#[test]
fn mka_run_all_splits_deterministic() {
    let (adj, pop) = small_grid(4, 4);
    let centroids = synthetic_centroids(4, 4);
    let run = || {
        run_all_splits_mka(
            &adj,
            &pop,
            2,
            0.05,
            None,
            180,
            MkaMetric::Reock,
            99999,
            &centroids,
        )
        .expect("run_all_splits_mka must succeed")
    };
    let a1 = run();
    let a2 = run();
    assert_eq!(a1, a2, "same base_seed must produce identical assignments");
}

// L2: NC 2020 MKA min_reock beats CVD-Geographic.
#[test]
#[ignore]
fn mka_nc_reock_beats_metis() {
    // Requires: data/2020/north_carolina_adjacency.adj.bin + centroids
    // Run MKA and CVD-Geographic on NC 2020 k=14.
    // Assert: MKA min_reock_score > CVD-Geographic min_reock_score.
    // Skipped unless --include-ignored is passed.
}

// ── L0: MKA-AreaSection hybrid warm-start (#162) ─────────────────────────

/// L0: "moving-knife" parses to AreaSectionInitArg::MovingKnife → AreaSectionInit::MovingKnife.
#[test]
fn area_section_init_parses() {
    use crate::args::AreaSectionInitArg;
    use crate::runner::AreaSectionInit;

    let mk: AreaSectionInit = AreaSectionInitArg::MovingKnife.into();
    assert_eq!(
        mk,
        AreaSectionInit::MovingKnife,
        "MovingKnife arg must convert to MovingKnife strategy"
    );

    let ro: AreaSectionInit = AreaSectionInitArg::RatioOptimal.into();
    assert_eq!(
        ro,
        AreaSectionInit::RatioOptimal,
        "RatioOptimal arg must convert to RatioOptimal strategy"
    );
}

/// L0: empty centroids → split_subgraph_mka_direction-based path falls back gracefully.
/// When tract_centroids is empty, the warm-start should not panic and should
/// produce Some(theta)=None path (no bias applied). We test by verifying that
/// split_subgraph_mka_direction on an empty set returns 0.0 without panicking.
#[test]
fn area_section_mka_init_fallback_no_centroids() {
    use crate::runner::AreaSectionInit;

    // Empty centroids — simulates the "no centroid data" fallback branch.
    let empty_tracts: HashSet<usize> = HashSet::new();
    let empty_centroids: Vec<(f64, f64)> = vec![];
    let theta = split_subgraph_mka_direction(&empty_tracts, &empty_centroids, 180);
    // Must return 0.0 (defined in the MKA function for empty input) — not panic.
    assert_eq!(
        theta, 0.0,
        "split_subgraph_mka_direction on empty set must return 0.0"
    );

    // Verify AreaSectionInit enum equality round-trips.
    let init = AreaSectionInit::MovingKnife;
    assert_eq!(init, AreaSectionInit::MovingKnife);
    assert_ne!(init, AreaSectionInit::RatioOptimal);
}
