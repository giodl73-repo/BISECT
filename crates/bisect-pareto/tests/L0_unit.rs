//! L0 unit tests for bisect-pareto.
//!
//! Tests dominance, crowding distance, and seed derivation.
//! All tests run unconditionally (no #[ignore]).
//! Per spec §12 (L0 invariants).

use bisect_pareto::dominance::{crowding_distance, dominates, fast_non_dominated_sort};
use bisect_pareto::objectives::Objectives;
use bisect_pareto::seeds::{cross_seed, init_seed, mut_seed};

fn obj(ec: f64, d: f64, vra: f64) -> Objectives {
    Objectives {
        ec,
        d_seats: d,
        vra_deficit: vra,
    }
}

// ── Dominance tests ───────────────────────────────────────────────────────────

#[test]
fn dominates_strict() {
    // A = (1,1,1), B = (2,2,2): A strictly better on all → A dominates B
    let a = obj(1.0, 1.0, 1.0);
    let b = obj(2.0, 2.0, 2.0);
    assert!(dominates(&a, &b), "A=(1,1,1) must dominate B=(2,2,2)");
    assert!(!dominates(&b, &a), "B=(2,2,2) must not dominate A=(1,1,1)");
}

#[test]
fn dominates_equal_not() {
    // A = B = (1,1,1): equal on all objectives → neither dominates
    let a = obj(1.0, 1.0, 1.0);
    let b = obj(1.0, 1.0, 1.0);
    assert!(
        !dominates(&a, &b),
        "equal plans must not dominate each other"
    );
    assert!(
        !dominates(&b, &a),
        "equal plans must not dominate each other (B vs A)"
    );
}

#[test]
fn dominates_mixed_not() {
    // A = (1,2,1), B = (2,1,2): A wins ec+vra, B wins d_seats → neither dominates
    let a = obj(1.0, 2.0, 1.0);
    let b = obj(2.0, 1.0, 2.0);
    assert!(!dominates(&a, &b), "A=(1,2,1) must not dominate B=(2,1,2)");
    assert!(!dominates(&b, &a), "B=(2,1,2) must not dominate A=(1,2,1)");
}

#[test]
fn dominates_one_objective_strict_rest_equal() {
    // A = (1,1,0), B = (2,1,0): A better on ec only, equal on rest → A dominates B
    let a = obj(1.0, 1.0, 0.0);
    let b = obj(2.0, 1.0, 0.0);
    assert!(
        dominates(&a, &b),
        "A must dominate B (strictly better on ec)"
    );
    assert!(!dominates(&b, &a));
}

// ── Non-dominated sort ────────────────────────────────────────────────────────

#[test]
fn non_dominated_sort_basic() {
    // A dominates B and C; B and C are mutually non-dominated
    let objectives = vec![
        obj(1.0, 1.0, 1.0), // 0: best — dominates both
        obj(2.0, 2.0, 2.0), // 1: dominated by 0
        obj(2.0, 3.0, 1.5), // 2: dominated by 0, not by 1
    ];
    let fronts = fast_non_dominated_sort(&objectives);
    assert!(!fronts.is_empty(), "fronts must not be empty");
    assert!(fronts[0].contains(&0), "plan 0 (best) must be in front 0");
    assert!(fronts.len() >= 2, "must have at least 2 fronts");
    assert!(fronts[1].contains(&1), "plan 1 must be in front 1");
    assert!(fronts[1].contains(&2), "plan 2 must be in front 1");
}

#[test]
fn non_dominated_sort_single_plan() {
    let objectives = vec![obj(5.0, 3.0, 0.1)];
    let fronts = fast_non_dominated_sort(&objectives);
    assert_eq!(fronts.len(), 1);
    assert_eq!(fronts[0], vec![0]);
}

#[test]
fn non_dominated_sort_all_equal() {
    let objectives = vec![obj(1.0, 1.0, 1.0); 4];
    let fronts = fast_non_dominated_sort(&objectives);
    // All equal → none dominate each other → all in front 0
    assert_eq!(fronts.len(), 1, "all equal → single front");
    assert_eq!(fronts[0].len(), 4, "all 4 in front 0");
}

#[test]
fn non_dominated_sort_chain() {
    // A dominates B, B dominates C → three separate fronts
    let objectives = vec![
        obj(1.0, 1.0, 1.0), // 0: front 0
        obj(2.0, 2.0, 2.0), // 1: front 1
        obj(3.0, 3.0, 3.0), // 2: front 2
    ];
    let fronts = fast_non_dominated_sort(&objectives);
    assert_eq!(fronts.len(), 3, "chain of dominance -> 3 separate fronts");
    assert_eq!(fronts[0], vec![0]);
    assert_eq!(fronts[1], vec![1]);
    assert_eq!(fronts[2], vec![2]);
}

// ── Crowding distance ──────────────────────────────────────────────────────────

#[test]
fn crowding_extremes_infinite() {
    // Plans at extreme of any objective get infinity
    let front = vec![0, 1, 2];
    let objectives = vec![
        obj(1.0, 5.0, 3.0), // min EC → infinite for EC
        obj(5.0, 1.0, 6.0), // min D_seats, max VRA → both infinite
        obj(3.0, 8.0, 0.0), // max D_seats, min VRA → both infinite
    ];
    let dist = crowding_distance(&front, &objectives);
    assert_eq!(dist.len(), 3);
    // Index 0: min EC → infinite
    assert!(dist[0].is_infinite(), "index 0 (min EC) must be infinite");
    // Index 1: min D_seats → infinite; max VRA → infinite
    assert!(dist[1].is_infinite(), "index 1 (min D) must be infinite");
    // Index 2: max D_seats → infinite; min VRA → infinite
    assert!(dist[2].is_infinite(), "index 2 (max D) must be infinite");
}

#[test]
fn crowding_middle_plan_finite() {
    // 4 plans: 2 extremes + 1 middle — middle plan gets finite distance
    let front = vec![0, 1, 2, 3];
    let objectives = vec![
        obj(1.0, 1.0, 0.0), // extreme min
        obj(2.0, 2.0, 0.5), // middle
        obj(3.0, 3.0, 1.0), // middle
        obj(4.0, 4.0, 2.0), // extreme max
    ];
    let dist = crowding_distance(&front, &objectives);
    assert_eq!(dist.len(), 4);
    // Extremes (index 0 and 3) should be infinite
    assert!(dist[0].is_infinite());
    assert!(dist[3].is_infinite());
    // Middle plans (index 1 and 2) should be finite
    assert!(
        dist[1].is_finite(),
        "middle plan must have finite crowding distance"
    );
    assert!(
        dist[2].is_finite(),
        "middle plan must have finite crowding distance"
    );
}

#[test]
fn crowding_two_plans_both_infinite() {
    let front = vec![0, 1];
    let objectives = vec![obj(1.0, 1.0, 0.0), obj(2.0, 2.0, 1.0)];
    let dist = crowding_distance(&front, &objectives);
    assert_eq!(dist.len(), 2);
    assert!(dist[0].is_infinite());
    assert!(dist[1].is_infinite());
}

// ── Seed derivation ───────────────────────────────────────────────────────────

#[test]
fn init_seeds_distinct() {
    let s = 42u64;
    let s0 = init_seed(s, 0);
    let s1 = init_seed(s, 1);
    let s2 = init_seed(s, 2);
    assert_ne!(s0, s1, "init_seed(s,0) != init_seed(s,1)");
    assert_ne!(s0, s2, "init_seed(s,0) != init_seed(s,2)");
    assert_ne!(s1, s2, "init_seed(s,1) != init_seed(s,2)");
}

#[test]
fn cross_mut_seeds_distinct() {
    let s = 42u64;
    let gen = 5u32;
    let i = 3u32;
    let cs = cross_seed(s, gen, i);
    let ms = mut_seed(s, gen, i);
    assert_ne!(cs, ms, "cross_seed != mut_seed for same inputs");
}

#[test]
fn seed_prefixes_distinct() {
    // PARETO_INIT_ (12b) != PARETO_CROSS_ (13b) != PARETO_MUT_ (11b)
    // With same "effective" gen/i/base, all three must differ
    let s = 0u64;
    let a = init_seed(s, 0); // PARETO_INIT_ prefix
    let b = cross_seed(s, 0, 0); // PARETO_CROSS_ prefix
    let c = mut_seed(s, 0, 0); // PARETO_MUT_ prefix
    assert_ne!(a, b, "PARETO_INIT_ != PARETO_CROSS_");
    assert_ne!(a, c, "PARETO_INIT_ != PARETO_MUT_");
    assert_ne!(b, c, "PARETO_CROSS_ != PARETO_MUT_");
}

#[test]
fn init_seed_deterministic() {
    let s = 99u64;
    assert_eq!(
        init_seed(s, 7),
        init_seed(s, 7),
        "same inputs -> same output"
    );
    assert_eq!(
        cross_seed(s, 3, 7),
        cross_seed(s, 3, 7),
        "cross deterministic"
    );
    assert_eq!(mut_seed(s, 3, 7), mut_seed(s, 3, 7), "mut deterministic");
}

#[test]
fn cross_seeds_vary_by_gen() {
    let s = 42u64;
    assert_ne!(cross_seed(s, 0, 0), cross_seed(s, 1, 0));
    assert_ne!(mut_seed(s, 0, 0), mut_seed(s, 1, 0));
}
