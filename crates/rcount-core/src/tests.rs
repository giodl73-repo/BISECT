
use super::*;

fn synthetic_shangrla_algorithm_run() -> AuditAlgorithmRun {
    AuditAlgorithmRun {
        run_id: "audit-run:shangrla-toy".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: SHANGRLA_ASSORTER_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithoutReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(100_000),
        reported_winner_votes: None,
        reported_loser_votes: None,
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::PluralityWinnerLoser,
            assorter_id: "plurality-winner-loser-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: vec![
            AuditSampleStep {
                step_index: 0,
                round_index: None,
                assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                sample_unit_id: "cvr:P-001:001".to_string(),
                assorter_value: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                bet: None,
                statistic: Some(RationalValue {
                    numerator: 2,
                    denominator: 1,
                }),
                p_value_ppm: Some(80_000),
                ranked_choices: Vec::new(),
                source_refs: vec!["source:synthetic-audit".to_string()],
            },
            AuditSampleStep {
                step_index: 1,
                round_index: None,
                assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                sample_unit_id: "cvr:P-001:002".to_string(),
                assorter_value: RationalValue {
                    numerator: 1,
                    denominator: 2,
                },
                bet: None,
                statistic: Some(RationalValue {
                    numerator: 3,
                    denominator: 1,
                }),
                p_value_ppm: Some(50_000),
                ranked_choices: Vec::new(),
                source_refs: vec!["source:synthetic-audit".to_string()],
            },
        ],
        decision: AuditAlgorithmDecision::Pass,
        source_refs: vec!["source:synthetic-audit".to_string()],
    }
}

#[test]
fn synthetic_summary_basic_verifies_selection_sums() {
    let package = synthetic_summary_basic_package();
    let report = verify_package(&package).expect("synthetic summary package must verify");
    assert_eq!(report.passed.len(), 3);
    assert!(report.failed.is_empty());
}

#[test]
fn parallel_verifier_matches_serial_reports() {
    let packages = vec![
        synthetic_summary_basic_package(),
        synthetic_canvass_correction_package(),
        synthetic_mail_batch_added_package(),
        synthetic_precinct_split_lineage_package(),
        synthetic_privacy_inclusion_package(),
        synthetic_cvr_summary_package(),
        synthetic_rla_replay_package(),
        synthetic_rla_stopping_package(),
        synthetic_rla_margin_package(),
        synthetic_rla_statistical_package(),
        synthetic_colorado_rla_package(),
        synthetic_california_rla_package(),
        synthetic_manual_audit_package(),
    ];

    for package in packages {
        let serial = verify_package(&package).expect("serial verifier must accept fixture");
        let parallel =
            verify_package_parallel(&package).expect("parallel verifier must accept fixture");
        assert_eq!(parallel, serial);
    }
}

#[test]
fn parallel_verifier_matches_serial_error_for_bad_selection_sum() {
    let package = synthetic_bad_selection_sum_package();
    assert_eq!(
        verify_package_parallel(&package).expect_err("parallel verifier must fail"),
        verify_package(&package).expect_err("serial verifier must fail")
    );
}

#[test]
fn audit_algorithm_transcript_accepts_shangrla_assorter_steps() {
    let mut package = synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![synthetic_shangrla_algorithm_run()];

    let report = verify_package(&package).expect("audit algorithm transcript must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "audit_algorithm_transcript"
            && pass.reporting_unit_id == "audit-run:shangrla-toy"
    }));
}

#[test]
fn audit_algorithm_transcript_rejects_missing_assertion_step() {
    let mut package = synthetic_summary_basic_package();
    let mut run = synthetic_shangrla_algorithm_run();
    run.sample_steps[0].assertion_id = "assertion:missing".to_string();
    package.audit_algorithm_runs = vec![run];

    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::MissingAuditAssertion { .. })
    ));
}

#[test]
fn audit_algorithm_transcript_rejects_out_of_bound_assorter_value() {
    let mut package = synthetic_summary_basic_package();
    let mut run = synthetic_shangrla_algorithm_run();
    run.sample_steps[0].assorter_value = RationalValue {
        numerator: 3,
        denominator: 2,
    };
    package.audit_algorithm_runs = vec![run];

    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::InvalidAuditAssorterValue { .. })
    ));
}

#[test]
fn audit_algorithm_transcript_rejects_partial_macro_design() {
    let mut package = synthetic_summary_basic_package();
    let mut run = synthetic_shangrla_algorithm_run();
    run.method_id = KAPLAN_MARKOV_COMPARISON_METHOD_ID.to_string();
    run.macro_ballot_count = Some(100);
    package.audit_algorithm_runs = vec![run];

    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::InvalidAuditMacroDesign { .. })
    ));
}

#[test]
fn audit_algorithm_transcript_rejects_invalid_macro_gamma() {
    let mut package = synthetic_summary_basic_package();
    let mut run = synthetic_shangrla_algorithm_run();
    run.method_id = KAPLAN_MARKOV_COMPARISON_METHOD_ID.to_string();
    run.macro_ballot_count = Some(100);
    run.macro_reported_margin = Some(10);
    run.macro_gamma = Some(RationalValue {
        numerator: 1,
        denominator: 1,
    });
    package.audit_algorithm_runs = vec![run];

    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::InvalidAuditMacroDesign { .. })
    ));
}

#[test]
fn kaplan_markov_macro_package_verifies_algorithm_transcript() {
    let package = synthetic_kaplan_markov_macro_package();
    let report = verify_package(&package).expect("MACRO package must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "audit_algorithm_transcript"
            && pass.reporting_unit_id == "audit-run:kaplan-markov-macro-pass"
    }));
}

#[test]
fn minerva_round_one_package_verifies_algorithm_transcript() {
    let package = synthetic_minerva_round_one_package();
    let report = verify_package(&package).expect("Minerva package must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "audit_algorithm_transcript"
            && pass.reporting_unit_id == "audit-run:minerva-round-one-pass"
    }));
}

#[test]
fn minerva_multi_round_package_verifies_algorithm_transcript() {
    let package = synthetic_minerva_multi_round_package();
    let report = verify_package(&package).expect("multi-round Minerva package must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "audit_algorithm_transcript"
            && pass.reporting_unit_id == "audit-run:minerva-multi-round-pass"
    }));
    assert_eq!(
        package.audit_algorithm_runs[0].sample_steps[4].round_index,
        Some(0)
    );
    assert_eq!(
        package.audit_algorithm_runs[0].sample_steps[5].round_index,
        Some(1)
    );
}

#[test]
fn athena_boundary_package_verifies_algorithm_transcript() {
    let package = synthetic_athena_boundary_package();
    let report = verify_package(&package).expect("Athena boundary package must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "audit_algorithm_transcript"
            && pass.reporting_unit_id == "audit-run:athena-boundary"
    }));
    assert_eq!(
        package.audit_algorithm_runs[0].method_id,
        ATHENA_BALLOT_POLLING_METHOD_ID
    );
}

#[test]
fn stratified_hybrid_package_verifies_component_references() {
    let package = synthetic_stratified_hybrid_package();
    let report = verify_package(&package).expect("stratified package must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "audit_algorithm_transcript"
            && pass.reporting_unit_id == "audit-run:stratified-hybrid-boundary"
    }));
    let run = package
        .audit_algorithm_runs
        .iter()
        .find(|run| run.method_id == STRATIFIED_HYBRID_RLA_METHOD_ID)
        .expect("stratified run must be present");
    assert_eq!(run.strata.len(), 2);
    assert_eq!(
        run.combining_rule_id.as_deref(),
        Some("suite-nuisance-boundary-v1")
    );
    assert_eq!(
        run.nuisance_parameter,
        Some(RationalValue {
            numerator: 1,
            denominator: 2,
        })
    );
    assert_eq!(run.strata[0].allocation_ppm, Some(500_000));
    assert_eq!(run.strata[1].allocation_ppm, Some(500_000));
}

#[test]
fn stratified_hybrid_package_rejects_missing_component() {
    let package = synthetic_bad_stratified_hybrid_package();
    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::MissingStratifiedHybridComponent { .. })
    ));
}

#[test]
fn stratified_hybrid_package_rejects_flattened_stratum() {
    let package = synthetic_flattened_stratified_hybrid_package();
    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::InvalidStratifiedHybridDesign { .. })
    ));
}

#[test]
fn raire_boundary_package_verifies_ranked_choice_surface() {
    let package = synthetic_raire_boundary_package();
    let report = verify_package(&package).expect("RAIRE boundary package must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "audit_algorithm_transcript"
            && pass.reporting_unit_id == "audit-run:raire-irv-boundary"
    }));
    assert_eq!(
        package.audit_algorithm_runs[0].rcv_elimination_order,
        vec![
            "cand-c".to_string(),
            "cand-b".to_string(),
            "cand-a".to_string(),
        ]
    );
}

#[test]
fn awaire_boundary_package_verifies_ranked_choice_surface() {
    let package = synthetic_awaire_boundary_package();
    let report = verify_package(&package).expect("AWAIRE boundary package must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "audit_algorithm_transcript"
            && pass.reporting_unit_id == "audit-run:awaire-irv-boundary"
    }));
    assert_eq!(
        package.audit_algorithm_runs[0].method_id,
        AWAIRE_IRV_METHOD_ID
    );
}

#[test]
fn ranked_choice_boundary_package_rejects_duplicate_ranked_choice() {
    let package = synthetic_bad_raire_boundary_package();
    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::InvalidRankedChoiceSample { .. })
    ));
}

#[test]
fn bayesian_tabulation_boundary_package_verifies_analytic_surface() {
    let package = synthetic_bayesian_tabulation_boundary_package();
    let report =
        verify_package(&package).expect("Bayesian tabulation boundary package must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "audit_algorithm_transcript"
            && pass.reporting_unit_id == "audit-run:bayesian-tabulation-boundary"
    }));
    assert_eq!(
        package.audit_algorithm_runs[0].posterior_winner_probability_ppm,
        Some(958_000)
    );
    assert_eq!(
        package.audit_algorithm_runs[0].posterior_risk_ppm,
        Some(42_000)
    );
}

#[test]
fn bayesian_tabulation_boundary_package_rejects_invalid_posterior_risk() {
    let package = synthetic_bad_bayesian_tabulation_boundary_package();
    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::InvalidBayesianAuditDesign { .. })
    ));
}

#[test]
fn soba_observable_ballot_boundary_package_verifies_opening_linkage() {
    let package = synthetic_soba_observable_ballot_boundary_package();
    let report = verify_package(&package).expect("SOBA boundary package must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "audit_algorithm_transcript"
            && pass.reporting_unit_id == "audit-run:soba-observable-ballot-boundary"
    }));
    assert!(package.inclusion_proofs[0].candidate_selections.is_empty());
    assert_eq!(
        package.audit_algorithm_runs[0].assertions[0].kind,
        AuditAssertionKind::ObservableBallotLinkage
    );
}

#[test]
fn soba_observable_ballot_boundary_package_rejects_missing_opening() {
    let package = synthetic_missing_soba_opening_package();
    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::MissingObservableBallotOpening { .. })
    ));
}

#[test]
fn synthetic_summary_basic_verifies_jurisdiction_total() {
    let package = synthetic_summary_basic_package();
    let passes =
        verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
            .expect("jurisdiction total must verify");
    assert_eq!(passes[0].equation_id, "jurisdiction_contest_total");
}

#[test]
fn synthetic_canvass_correction_verifies_both_status_snapshots() {
    let package = synthetic_canvass_correction_package();
    let report = verify_package(&package).expect("canvass correction package must verify");
    assert_eq!(
        report
            .passed
            .iter()
            .filter(|pass| pass.equation_id == "contest_selection_sum")
            .count(),
        6
    );
    let jurisdiction_passes =
        verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
            .expect("both status snapshots must reconcile");
    assert_eq!(jurisdiction_passes.len(), 2);
}

#[test]
fn synthetic_mail_batch_added_verifies_batch_summaries() {
    let package = synthetic_mail_batch_added_package();
    let report = verify_package(&package).expect("mail batch package must verify");
    assert_eq!(
        report
            .passed
            .iter()
            .filter(|pass| pass.equation_id == "batch_summary_total")
            .count(),
        3
    );
    assert_eq!(
        report
            .passed
            .iter()
            .filter(|pass| pass.equation_id == "accepted_ballots")
            .count(),
        3
    );
    let jurisdiction_passes =
        verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
            .expect("batched summaries must roll up");
    assert_eq!(jurisdiction_passes.len(), 1);
}

#[test]
fn batch_summary_total_catches_missing_batch() {
    let package = synthetic_missing_batch_package();
    let err = verify_batch_summary_totals(&package).expect_err("missing batch must fail");
    assert!(matches!(err, RcountCoreError::MissingBatch { .. }));
}

#[test]
fn synthetic_precinct_split_lineage_verifies_split_and_merge() {
    let package = synthetic_precinct_split_lineage_package();
    let report = verify_package(&package).expect("lineage package must verify");
    assert_eq!(
        report
            .passed
            .iter()
            .filter(|pass| pass.equation_id == "lineage_conservation")
            .count(),
        2
    );
}

#[test]
fn rhist_reference_declared_verifies() {
    let mut package = synthetic_summary_basic_package();
    package.rhist_refs = vec![RhistReference {
        reference_id: "rhist:real-ri-tract-unchanged".to_string(),
        package_hash: "sha256:ccbddf423aa4ac08b0d45c4ac0b9db411293ea41fef3ac8fa93f9de9e85f66bb"
            .to_string(),
        package_path: Some("docs/fixtures/rhist/real-ri-tract-unchanged".to_string()),
        cycle_ids: vec![
            "ri-2000-census".to_string(),
            "ri-2010-census".to_string(),
            "ri-2020-census".to_string(),
        ],
        role: "unit-lineage".to_string(),
        note: Some("Real-source RHIST pressure fixture.".to_string()),
    }];

    let report = verify_package(&package).expect("RHIST reference must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "rhist_reference_declared"
            && pass.reporting_unit_id == "rhist:real-ri-tract-unchanged"
    }));
}

#[test]
fn rhist_reference_consumes_split_merge_fixture_package_hash() {
    let mut package = synthetic_summary_basic_package();
    let package_hash = rhist_fixture_package_hash("l2-three-cycle");
    package.rhist_refs = vec![RhistReference {
        reference_id: "rhist:syn-l2-three-cycle".to_string(),
        package_hash: package_hash.clone(),
        package_path: Some("docs/fixtures/rhist/l2-three-cycle".to_string()),
        cycle_ids: vec![
            "syn-2024-general".to_string(),
            "syn-2026-general".to_string(),
            "syn-2028-general".to_string(),
        ],
        role: "unit-lineage".to_string(),
        note: Some("References RHIST rename/split/merge fixture by package hash.".to_string()),
    }];

    let report = verify_package(&package).expect("RHIST fixture reference must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "rhist_reference_declared"
            && pass.reporting_unit_id == "rhist:syn-l2-three-cycle"
    }));
    assert_eq!(
        package.rhist_refs[0].package_hash,
        SYN_RHIST_L2_PACKAGE_HASH
    );
}

#[test]
fn synthetic_summary_basic_base_references_verify_together() {
    let package = synthetic_summary_basic_package_with_base_references();
    let report = verify_package(&package).expect("base references must verify");

    assert_eq!(package.rctx_refs[0].context_hash, SYN_RCTX_L0_CONTEXT_HASH);
    assert_eq!(
        package.rctx_refs[0].crosswalk_hash.as_deref(),
        Some(SYN_RCTX_L0_CROSSWALK_HASH)
    );
    assert_eq!(
        package.rhist_refs[0].package_hash,
        SYN_RHIST_L2_PACKAGE_HASH
    );
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "rctx_reference_declared"));
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "rhist_reference_declared"));
}

#[test]
fn rhist_reference_rejects_bad_hash() {
    let mut package = synthetic_summary_basic_package();
    package.rhist_refs = vec![RhistReference {
        reference_id: "rhist:bad-hash".to_string(),
        package_hash: "not-a-hash".to_string(),
        package_path: None,
        cycle_ids: vec!["cycle:one".to_string()],
        role: "unit-lineage".to_string(),
        note: None,
    }];

    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::InvalidRhistPackageHash { .. })
    ));
}

#[test]
fn rhist_reference_requires_cycles_and_supported_role() {
    let mut package = synthetic_summary_basic_package();
    package.rhist_refs = vec![RhistReference {
        reference_id: "rhist:no-cycles".to_string(),
        package_hash: "sha256:ccbddf423aa4ac08b0d45c4ac0b9db411293ea41fef3ac8fa93f9de9e85f66bb"
            .to_string(),
        package_path: None,
        cycle_ids: vec![],
        role: "unit-lineage".to_string(),
        note: None,
    }];
    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::EmptyRhistCycleRefs { .. })
    ));

    package.rhist_refs[0].cycle_ids = vec!["cycle:one".to_string()];
    package.rhist_refs[0].role = "freeform-history".to_string();
    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::UnsupportedRhistReferenceRole { .. })
    ));
}

fn rhist_fixture_package_hash(name: &str) -> String {
    let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("fixtures")
        .join("rhist")
        .join(name)
        .join("manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
    manifest["package_content_hash"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn rctx_reference_declared_verifies() {
    let mut package = synthetic_summary_basic_package();
    package.rctx_refs = vec![RctxReference {
        reference_id: "rctx:summary-basic-context".to_string(),
        context_hash: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
            .to_string(),
        context_path: Some("context.rctx".to_string()),
        crosswalk_hash: Some(
            "sha256:2222222222222222222222222222222222222222222222222222222222222222".to_string(),
        ),
        crosswalk_path: Some("crosswalks/summary-basic-to-plan.ndjson".to_string()),
        role: "aggregation-crosswalk".to_string(),
        note: Some("Synthetic RCTX aggregation binding.".to_string()),
    }];

    let report = verify_package(&package).expect("RCTX reference must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "rctx_reference_declared"
            && pass.reporting_unit_id == "rctx:summary-basic-context"
    }));
}

#[test]
fn rctx_reference_rejects_bad_hashes_and_role() {
    let mut package = synthetic_summary_basic_package();
    package.rctx_refs = vec![RctxReference {
        reference_id: "rctx:bad-context".to_string(),
        context_hash: "not-a-hash".to_string(),
        context_path: None,
        crosswalk_hash: None,
        crosswalk_path: None,
        role: "unit-context".to_string(),
        note: None,
    }];
    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::InvalidRctxContextHash { .. })
    ));

    package.rctx_refs[0].context_hash =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111".to_string();
    package.rctx_refs[0].crosswalk_hash = Some("not-a-hash".to_string());
    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::InvalidRctxCrosswalkHash { .. })
    ));

    package.rctx_refs[0].crosswalk_hash = None;
    package.rctx_refs[0].role = "map-render".to_string();
    assert!(matches!(
        verify_package(&package),
        Err(RcountCoreError::UnsupportedRctxReferenceRole { .. })
    ));
}

#[test]
fn lineage_conservation_catches_missing_current_unit() {
    let package = synthetic_bad_lineage_package();
    let err = verify_lineage_conservation(&package).expect_err("bad lineage must fail");
    assert!(matches!(
        err,
        RcountCoreError::MissingCurrentLineageUnit { .. }
    ));
}

#[test]
fn synthetic_privacy_inclusion_proof_verifies() {
    let package = synthetic_privacy_inclusion_package();
    let report = verify_package(&package).expect("privacy inclusion proof must verify");
    assert_eq!(
        report
            .passed
            .iter()
            .filter(|pass| pass.equation_id == "proof_privacy_gate")
            .count(),
        1
    );
}

#[test]
fn choice_bearing_proof_fails_privacy_gate() {
    let package = synthetic_choice_bearing_proof_package();
    let err = verify_proof_privacy(&package).expect_err("choice-bearing proof must fail");
    assert!(matches!(err, RcountCoreError::ChoiceBearingProof { .. }));
}

#[test]
fn synthetic_cvr_summary_verifies_against_summaries() {
    let package = synthetic_cvr_summary_package();
    let report = verify_package(&package).expect("CVR summary package must verify");
    assert_eq!(
        report
            .passed
            .iter()
            .filter(|pass| pass.equation_id == "cvr_summary_total")
            .count(),
        2
    );
}

#[test]
fn bad_cvr_summary_fails_cvr_reconciliation() {
    let package = synthetic_bad_cvr_summary_package();
    let err =
        verify_cvr_summary_reconciliation(&package).expect_err("bad CVR summary package must fail");
    assert!(matches!(err, RcountCoreError::CvrSummaryMismatch { .. }));
}

#[test]
fn rla_replay_package_verifies_sample() {
    let package = synthetic_rla_replay_package();
    let report = verify_package(&package).expect("RLA replay package must verify");
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "rla_sampler_replay"));
    assert_eq!(package.rla_audits[0].sample_draws.len(), 12);
}

#[test]
fn rla_replay_fails_on_tampered_sample_draw() {
    let package = synthetic_bad_rla_replay_package();
    let err = verify_rla_sampler_replay(&package)
        .expect_err("bad RLA replay package must fail sample replay");
    assert!(matches!(err, RcountCoreError::RlaSampleMismatch { .. }));
}

#[test]
fn rla_stopping_package_verifies_observations() {
    let package = synthetic_rla_stopping_package();
    let report = verify_package(&package).expect("RLA stopping package must verify");
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "rla_stopping_rule"));
}

#[test]
fn rla_stopping_fails_when_declared_pass_has_discrepancy() {
    let package = synthetic_bad_rla_stopping_package();
    let err = verify_rla_stopping_rules(&package)
        .expect_err("bad RLA stopping package must fail stopping rule");
    assert!(matches!(
        err,
        RcountCoreError::RlaStoppingStatusMismatch { .. }
    ));
}

#[test]
fn rla_discrepancy_package_verifies_declared_taxonomy() {
    let package = synthetic_rla_discrepancy_package();
    let report = verify_package(&package).expect("RLA discrepancy package must verify");
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "rla_stopping_rule"));
}

#[test]
fn rla_discrepancy_fails_when_declared_kind_is_wrong() {
    let package = synthetic_bad_rla_discrepancy_package();
    let err = verify_rla_stopping_rules(&package)
        .expect_err("bad RLA discrepancy package must fail taxonomy check");
    assert!(matches!(
        err,
        RcountCoreError::RlaDiscrepancyMismatch { .. }
    ));
}

#[test]
fn rla_margin_package_verifies_reported_margin_metadata() {
    let package = synthetic_rla_margin_package();
    let report = verify_package(&package).expect("RLA margin package must verify");
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "rla_margin_metadata"));
}

#[test]
fn rla_margin_fails_when_declared_margin_drifts() {
    let package = synthetic_bad_rla_margin_package();
    let err = verify_rla_margin_metadata(&package)
        .expect_err("bad RLA margin package must fail margin metadata");
    assert!(matches!(
        err,
        RcountCoreError::RlaReportedMarginMismatch { .. }
    ));
}

#[test]
fn rla_statistical_package_verifies_risk_estimate() {
    let package = synthetic_rla_statistical_package();
    let report = verify_package(&package).expect("RLA statistical package must verify");
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "rla_stopping_rule"));
    assert_eq!(package.rla_audits[0].declared_risk_ppm, Some(1303));
}

#[test]
fn rla_statistical_fails_when_declared_risk_drifts() {
    let package = synthetic_bad_rla_statistical_package();
    let err = verify_rla_stopping_rules(&package)
        .expect_err("bad RLA statistical package must fail risk estimate");
    assert!(matches!(
        err,
        RcountCoreError::RlaRiskEstimateMismatch { .. }
    ));
}

#[test]
fn colorado_rla_package_verifies_jurisdiction_adapter() {
    let package = synthetic_colorado_rla_package();
    let report = verify_package(&package).expect("Colorado-style RLA package must verify");
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "rla_jurisdiction_adapter"));
    assert_eq!(
        package.rla_audits[0].jurisdiction_method_id.as_deref(),
        Some(COLORADO_RLA_METHOD_ID)
    );
}

#[test]
fn colorado_rla_fails_when_seed_is_not_twenty_digits() {
    let package = synthetic_bad_colorado_rla_package();
    let err = verify_rla_jurisdiction_adapters(&package)
        .expect_err("bad Colorado-style RLA package must fail jurisdiction adapter");
    assert!(matches!(
        err,
        RcountCoreError::InvalidColoradoRlaSeed { .. }
    ));
}

#[test]
fn california_rla_package_verifies_public_tool_adapter() {
    let package = synthetic_california_rla_package();
    let report = verify_package(&package).expect("California-style RLA package must verify");
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "rla_jurisdiction_adapter"));
    assert_eq!(
        package.rla_audits[0].ballot_manifest_format_id.as_deref(),
        Some(CALIFORNIA_BALLOT_MANIFEST_FORMAT_ID)
    );
}

#[test]
fn california_rla_fails_when_source_url_is_not_public_url() {
    let package = synthetic_bad_california_rla_package();
    let err = verify_rla_jurisdiction_adapters(&package)
        .expect_err("bad California-style RLA package must fail jurisdiction adapter");
    assert!(matches!(
        err,
        RcountCoreError::InvalidRlaSoftwareSourceUrl { .. }
    ));
}

#[test]
fn manual_audit_package_verifies_hand_count_totals() {
    let package = synthetic_manual_audit_package();
    let report = verify_package(&package).expect("manual audit package must verify");
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "manual_audit_reconciliation"));
}

#[test]
fn manual_audit_fails_when_hand_count_exceeds_tolerance() {
    let package = synthetic_bad_manual_audit_package();
    let err = verify_manual_audits(&package)
        .expect_err("bad manual audit package must fail reconciliation");
    assert!(matches!(
        err,
        RcountCoreError::ManualAuditStatusMismatch { .. }
    ));
}

#[test]
fn batch_comparison_package_verifies_overstatement() {
    let package = synthetic_batch_comparison_package();
    let report = verify_package(&package).expect("batch comparison package must verify");
    assert!(report
        .passed
        .iter()
        .any(|pass| pass.equation_id == "batch_comparison_overstatement"
            && pass.reporting_unit_id == "batch:P-001:election-day"));
}

#[test]
fn batch_comparison_algorithm_links_to_verified_overstatement() {
    let package = synthetic_batch_comparison_algorithm_package();
    let report = verify_package(&package).expect("batch comparison algorithm package must verify");
    assert!(report.passed.iter().any(|pass| {
        pass.equation_id == "batch_comparison_algorithm_linkage"
            && pass.reporting_unit_id == "audit-run:batch-comparison-taint-linkage"
    }));
}

#[test]
fn derives_batch_comparison_algorithm_run_from_sample_order() {
    let package = synthetic_batch_comparison_package();
    let run = derive_batch_comparison_algorithm_run(
        &package,
        "audit-run:batch-comparison-derived",
        "syn-2024-mayor",
        300_000,
        &["batch:P-001:election-day".to_string()],
        AuditAlgorithmDecision::Continue,
    )
    .expect("batch comparison run must derive from package audits");

    assert_eq!(run.method_id, BATCH_COMPARISON_METHOD_ID);
    assert_eq!(
        run.sample_steps[0].sample_unit_id,
        "batch:P-001:election-day"
    );
    assert_eq!(
        run.sample_steps[0].assorter_value,
        RationalValue {
            numerator: 2,
            denominator: 5
        }
    );
    assert!(run.sample_steps[0]
        .source_refs
        .contains(&"batch-comparison:P-001-election-day".to_string()));
    assert!(run
        .source_refs
        .contains(&"source:synthetic-batch-hand-tally".to_string()));
}

#[test]
fn batch_comparison_algorithm_derivation_rejects_missing_sampled_batch() {
    let package = synthetic_batch_comparison_package();
    let err = derive_batch_comparison_algorithm_run(
        &package,
        "audit-run:batch-comparison-derived",
        "syn-2024-mayor",
        300_000,
        &["batch:P-404".to_string()],
        AuditAlgorithmDecision::Continue,
    )
    .expect_err("missing sampled batch must fail derivation");

    assert!(matches!(
        err,
        RcountCoreError::MissingBatchComparisonAlgorithmEvidence { .. }
    ));
}

#[test]
fn batch_comparison_algorithm_fails_when_taint_drifts() {
    let package = synthetic_bad_batch_comparison_algorithm_package();
    let err = verify_audit_algorithm_runs(&package)
        .expect_err("bad batch comparison algorithm taint must fail linkage");
    assert!(matches!(
        err,
        RcountCoreError::BatchComparisonAlgorithmTaintMismatch { .. }
    ));
}

#[test]
fn batch_comparison_fails_when_overstatement_drifts() {
    let package = synthetic_bad_batch_comparison_package();
    let err = verify_batch_comparison_audits(&package)
        .expect_err("bad batch comparison package must fail overstatement check");
    assert!(matches!(
        err,
        RcountCoreError::BatchComparisonOverstatementMismatch { .. }
    ));
}

#[test]
fn batch_comparison_fails_when_hand_tally_is_missing() {
    let package = synthetic_missing_hand_tally_batch_comparison_package();
    let err = verify_batch_comparison_audits(&package)
        .expect_err("missing hand tally must fail batch comparison check");
    assert!(matches!(
        err,
        RcountCoreError::MissingBatchComparisonHandTally { .. }
    ));
}

#[test]
fn batch_comparison_fails_when_batch_size_drifts() {
    let package = synthetic_batch_size_drift_comparison_package();
    let err = verify_batch_comparison_audits(&package)
        .expect_err("batch size drift must fail before overstatement check");
    assert!(matches!(
        err,
        RcountCoreError::BatchComparisonBatchSizeMismatch { .. }
    ));
}

#[test]
fn canvass_correction_requires_public_event_and_snapshots() {
    let mut package = synthetic_canvass_correction_package();
    let pass = verify_canvass_correction_event(&package).unwrap();
    assert_eq!(pass.equation_id, "canvass_correction_event");

    package.status_events.clear();
    let err =
        verify_canvass_correction_event(&package).expect_err("missing correction event must fail");
    assert!(matches!(
        err,
        RcountCoreError::MissingCanvassCorrectionEvent
    ));
}

#[test]
fn bad_arithmetic_fails_with_specific_equation_error() {
    let package = synthetic_bad_selection_sum_package();
    let err = verify_package(&package).expect_err("bad counted ballot total must fail");
    assert!(matches!(
        err,
        RcountCoreError::ContestSelectionSumMismatch { .. }
    ));
}

#[test]
fn tampered_jurisdiction_total_fails() {
    let mut package = synthetic_summary_basic_package();
    let total = package
        .summaries
        .iter_mut()
        .find(|summary| summary.reporting_unit_id == "syn:jurisdiction:SYN")
        .unwrap();
    total.totals[0].votes += 1;
    let err =
        verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
            .expect_err("tampered jurisdiction total must fail");
    assert!(matches!(
        err,
        RcountCoreError::JurisdictionSelectionMismatch { .. }
    ));
}

#[test]
fn record_hash_is_stable_for_equivalent_json_key_order() {
    let a = serde_json::json!({"b": 2, "a": {"d": 4, "c": 3}});
    let b = serde_json::json!({"a": {"c": 3, "d": 4}, "b": 2});
    assert_eq!(
        canonical_hash(RECORD_HASH_PREFIX, &a).unwrap(),
        canonical_hash(RECORD_HASH_PREFIX, &b).unwrap()
    );
}

#[test]
fn package_content_hash_has_rcount_prefix() {
    let package = synthetic_summary_basic_package();
    let hash = package_content_hash(&package).unwrap();
    assert!(hash.starts_with("sha256:"));
    assert_eq!(hash.len(), "sha256:".len() + 64);
}
