use bisect_analysis::comparison::{jaccard, try_compare_plans};
use std::collections::{BTreeMap, HashMap, HashSet};

fn sets(assignments: &HashMap<String, usize>) -> BTreeMap<usize, HashSet<String>> {
    let mut result = BTreeMap::<usize, HashSet<String>>::new();
    for (unit, &district) in assignments {
        result.entry(district).or_default().insert(unit.clone());
    }
    result
}

fn all_pairs(a: &BTreeMap<usize, HashSet<String>>, b: &BTreeMap<usize, HashSet<String>>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let best: Vec<_> = a.values().filter_map(|x| {
        let value = b.values().map(|y| jaccard(x, y)).fold(f64::NEG_INFINITY, f64::max);
        value.is_finite().then_some(value)
    }).collect();
    if best.is_empty() { 0.0 } else { best.iter().fold(0.0, |a, b| a + b) / best.len() as f64 }
}

fn deviation(plan: &BTreeMap<usize, HashSet<String>>) -> f64 {
    if plan.is_empty() { return 0.0; }
    let mean = plan.values().map(|set| set.len() as f64).sum::<f64>() / plan.len() as f64;
    plan.values().map(|set| (set.len() as f64 - mean).abs() / mean).fold(0.0, f64::max)
}

#[test]
fn sparse_overlap_preserves_ordered_set_metrics_bit_for_bit() {
    let mut state = 42u64;
    let mut draw = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (state >> 32) as usize
    };
    for trial in 0..900 {
        let mut a = HashMap::new();
        let mut b = HashMap::new();
        for unit in 0..83 {
            if trial % 13 != 0 && draw() % 5 < 4 { a.insert(unit.to_string(), 1 + (draw() % (trial % 17 + 1)) * 17); }
            if trial % 11 != 0 && draw() % 7 < 5 { b.insert((unit + trial % 89).to_string(), 1 + (draw() % (trial % 13 + 1)) * 23); }
        }
        for (a,b) in [(&a,&b),(&b,&a)] {
            let output = try_compare_plans(a,b).unwrap();
            let set_a = sets(a);
            let set_b = sets(b);
            assert_eq!(output.jaccard_similarity.to_bits(), all_pairs(&set_a,&set_b).to_bits(), "trial {trial}");
            assert_eq!(output.population.plan_a_max_dev.to_bits(), deviation(&set_a).to_bits(), "trial {trial}");
            assert_eq!(output.population.plan_b_max_dev.to_bits(), deviation(&set_b).to_bits(), "trial {trial}");
            assert_eq!(output.plan_a.num_districts, set_a.len());
            assert_eq!(output.plan_b.num_districts, set_b.len());
        }
    }
}
