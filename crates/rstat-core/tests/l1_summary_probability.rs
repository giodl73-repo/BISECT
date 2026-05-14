use rstat_core::probability::regularized_incomplete_beta;
use rstat_core::summary::{percentile_interval_sorted_copy, quantile_sorted_copy, summary_stats};

#[test]
fn l1_summary_quantiles_are_order_invariant() {
    let ordered = [1.0, 2.0, 3.0, 4.0, 5.0];
    let shuffled = [5.0, 1.0, 4.0, 2.0, 3.0];

    assert_eq!(
        quantile_sorted_copy(&ordered, 0.25).unwrap(),
        quantile_sorted_copy(&shuffled, 0.25).unwrap()
    );
    assert_eq!(
        percentile_interval_sorted_copy(&ordered, 0.025, 0.975).unwrap(),
        percentile_interval_sorted_copy(&shuffled, 0.025, 0.975).unwrap()
    );
}

#[test]
fn l1_summary_and_probability_compose_for_interval_report() {
    let samples = [0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80];

    let stats = summary_stats(&samples).unwrap();
    let (lo, hi) = percentile_interval_sorted_copy(&samples, 0.25, 0.75).unwrap();
    let beta_mid = regularized_incomplete_beta(0.5, 2.0, 2.0);

    assert_eq!(stats.count, 8);
    assert!((stats.mean - 0.45).abs() < 1e-12);
    assert!(lo < stats.mean && stats.mean < hi);
    assert!((beta_mid - 0.5).abs() < 0.01);
}
