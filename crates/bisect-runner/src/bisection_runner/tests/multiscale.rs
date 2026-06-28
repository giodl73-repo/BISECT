use super::*;

// ── MultiScale tests ──────────────────────────────────────────────────────

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
