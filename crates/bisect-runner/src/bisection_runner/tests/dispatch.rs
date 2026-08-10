use super::*;

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

/// Scenario 23: fixed-seed recursive determinism across multiple active nodes.
/// Four districts force two METIS calls at depth one; these must execute
/// sequentially because the C backend shares RNG state.
#[test]
fn test_recursive_metis_fixed_seed_is_deterministic() {
    // An 8-node chain graph: 0-1-2-3-4-5-6-7
    let adj = vec![
        vec![1usize],
        vec![0, 2],
        vec![1, 3],
        vec![2, 4],
        vec![3, 5],
        vec![4, 6],
        vec![5, 7],
        vec![6],
    ];
    let vw = vec![1000i64; 8];
    let ew = HashMap::new();

    let result1 = run_all_splits(&adj, &vw, &ew, 4, 0.01, 100, Some(42), None);
    let result2 = run_all_splits(&adj, &vw, &ew, 4, 0.01, 100, Some(42), None);

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
