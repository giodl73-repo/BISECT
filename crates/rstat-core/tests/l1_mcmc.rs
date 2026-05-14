use rstat_core::mcmc::{
    effective_sample_size, gelman_rubin_rhat, hamming_autocorrelation,
    integrated_autocorrelation_time, DiagnosticsError,
};

#[test]
fn l1_mcmc_diagnostics_separate_converged_and_stuck_traces() {
    let c1 = vec![0.49, 0.50, 0.51, 0.50, 0.49, 0.51];
    let c2 = vec![0.50, 0.49, 0.51, 0.50, 0.51, 0.49];
    let c3 = vec![0.51, 0.50, 0.49, 0.50, 0.49, 0.51];
    let c4 = vec![0.50, 0.51, 0.49, 0.50, 0.51, 0.49];
    let chains = vec![c1.as_slice(), c2.as_slice(), c3.as_slice(), c4.as_slice()];

    let rhat = gelman_rubin_rhat(&chains).unwrap();
    let mixed_ess = effective_sample_size(&[0.0, 1.0, -1.0, 0.5, -0.5, 1.5, -1.5, 0.25]);
    let stuck_ess = effective_sample_size(&[1.0; 8]);

    assert!(rhat < 1.05);
    assert!(mixed_ess > 0.0);
    assert_eq!(stuck_ess, 8.0);
}

#[test]
fn l1_hamming_tau_tracks_mixing_speed() {
    let slow = vec![
        vec![1, 1, 2, 2],
        vec![1, 1, 2, 2],
        vec![1, 2, 2, 2],
        vec![2, 2, 2, 1],
    ];
    let fast = vec![
        vec![1, 1, 1, 1],
        vec![2, 2, 2, 2],
        vec![1, 1, 1, 1],
        vec![2, 2, 2, 2],
    ];

    let slow_h = hamming_autocorrelation(&slow, 3).unwrap();
    let fast_h = hamming_autocorrelation(&fast, 3).unwrap();

    assert!(integrated_autocorrelation_time(&slow_h) > 1.0);
    assert!(integrated_autocorrelation_time(&fast_h) <= integrated_autocorrelation_time(&slow_h));
}

#[test]
fn l1_hamming_rejects_empty_partition_vectors() {
    let partitions = vec![Vec::new(), Vec::new(), Vec::new()];

    assert_eq!(
        hamming_autocorrelation(&partitions, 2),
        Err(DiagnosticsError::EmptyPartition(0))
    );
}
