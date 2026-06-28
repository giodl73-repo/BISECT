use crate::*;

pub fn synthetic_summary_basic_package() -> RcountPackage {
    let contest = Contest {
        contest_id: "syn-2024-mayor".to_string(),
        title: "Synthetic Mayor".to_string(),
        vote_for: 1,
        selections: vec![
            Selection {
                selection_id: "cand-a".to_string(),
                kind: SelectionKind::Candidate,
                label: "Candidate A".to_string(),
            },
            Selection {
                selection_id: "cand-b".to_string(),
                kind: SelectionKind::Candidate,
                label: "Candidate B".to_string(),
            },
            Selection {
                selection_id: "write-in".to_string(),
                kind: SelectionKind::WriteInBucket,
                label: "Write-in".to_string(),
            },
        ],
    };
    let reporting_units = vec![
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-001".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-001".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: None,
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-002".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-002".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: None,
        },
        ReportingUnit {
            reporting_unit_id: "syn:jurisdiction:SYN".to_string(),
            kind: ReportingUnitKind::JurisdictionTotal,
            parent_jurisdiction: "syn".to_string(),
            source_ids: vec!["SYN".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: None,
        },
    ];
    let summaries = vec![
        summary("syn:precinct:P-001", 40, 35, 1, 3, 1, 0),
        summary("syn:precinct:P-002", 25, 30, 0, 4, 0, 1),
        summary("syn:jurisdiction:SYN", 65, 65, 1, 7, 1, 1),
    ];
    RcountPackage {
        rcount_version: RCOUNT_VERSION.to_string(),
        contests: vec![contest],
        reporting_units,
        batches: vec![],
        lineage: vec![],
        rhist_refs: vec![],
        rctx_refs: vec![],
        inclusion_proofs: vec![],
        cvr: vec![],
        audit_algorithm_runs: vec![],
        rla_audits: vec![],
        manual_audits: vec![],
        batch_comparison_audits: vec![],
        summaries,
        status_events: vec![],
    }
}

pub fn synthetic_summary_basic_package_with_base_references() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.rctx_refs = vec![RctxReference {
        reference_id: "rctx:syn-l0-shared-context".to_string(),
        context_hash: SYN_RCTX_L0_CONTEXT_HASH.to_string(),
        context_path: Some("docs/fixtures/rctx/l0-shared-context".to_string()),
        crosswalk_hash: Some(SYN_RCTX_L0_CROSSWALK_HASH.to_string()),
        crosswalk_path: Some("docs/fixtures/rctx/l0-shared-context/units/crosswalks.ndjson".to_string()),
        role: "aggregation-crosswalk".to_string(),
        note: Some(format!(
            "References RCTX fixture package {SYN_RCTX_L0_PACKAGE_HASH} by stable context and crosswalk hashes."
        )),
    }];
    package.rhist_refs = vec![RhistReference {
        reference_id: "rhist:syn-l2-three-cycle".to_string(),
        package_hash: SYN_RHIST_L2_PACKAGE_HASH.to_string(),
        package_path: Some("docs/fixtures/rhist/l2-three-cycle".to_string()),
        cycle_ids: vec![
            "syn-2024-general".to_string(),
            "syn-2026-general".to_string(),
            "syn-2028-general".to_string(),
        ],
        role: "unit-lineage".to_string(),
        note: Some("References RHIST rename/split/merge fixture by package hash.".to_string()),
    }];
    package
}

pub fn synthetic_canvass_correction_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    let unofficial = vec![
        summary_with_status(
            "syn:precinct:P-001",
            CountStatus::Unofficial,
            40,
            34,
            1,
            3,
            1,
            0,
        ),
        summary_with_status(
            "syn:precinct:P-002",
            CountStatus::Unofficial,
            25,
            30,
            0,
            4,
            0,
            1,
        ),
        summary_with_status(
            "syn:jurisdiction:SYN",
            CountStatus::Unofficial,
            65,
            64,
            1,
            7,
            1,
            1,
        ),
    ];
    let canvassed = vec![
        summary_with_status(
            "syn:precinct:P-001",
            CountStatus::Canvassed,
            40,
            35,
            1,
            3,
            1,
            0,
        ),
        summary_with_status(
            "syn:precinct:P-002",
            CountStatus::Canvassed,
            25,
            30,
            0,
            4,
            0,
            1,
        ),
        summary_with_status(
            "syn:jurisdiction:SYN",
            CountStatus::Canvassed,
            65,
            65,
            1,
            7,
            1,
            1,
        ),
    ];
    package.summaries = unofficial.into_iter().chain(canvassed).collect();
    package.status_events = vec![
        StatusEvent {
            event_id: "event-0001".to_string(),
            event_type: StatusEventType::InitialUnofficialReport,
            status_before: CountStatus::Unofficial,
            status_after: CountStatus::Unofficial,
            effective_at: "2024-11-05T23:00:00Z".to_string(),
            authority: "SYN County Election Office".to_string(),
            source_refs: vec!["source:unofficial-election-night-export".to_string()],
            explanation: "Election-night unofficial report loaded from the first public export.".to_string(),
        },
        StatusEvent {
            event_id: "event-0002".to_string(),
            event_type: StatusEventType::Correction,
            status_before: CountStatus::Unofficial,
            status_after: CountStatus::Canvassed,
            effective_at: "2024-11-12T18:22:00Z".to_string(),
            authority: "SYN County Canvassing Board".to_string(),
            source_refs: vec!["source:canvass-minutes-2024-11-12".to_string()],
            explanation: "Canvass correction added one Candidate B vote in P-001 after write-in adjudication review.".to_string(),
        },
    ];
    package
}

pub fn synthetic_bad_selection_sum_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.summaries[0].counted_ballots += 1;
    package
}

pub fn synthetic_mail_batch_added_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.batches = vec![
        BatchManifest {
            batch_id: "batch:P-001:election-day".to_string(),
            reporting_unit_id: "syn:precinct:P-001".to_string(),
            kind: BatchKind::ElectionDay,
            status: CountStatus::Canvassed,
            accepted_ballots: 70,
            counted_ballots: 70,
            rejected_ballots: 0,
            source_refs: vec!["source:synthetic-summary-export".to_string()],
        },
        BatchManifest {
            batch_id: "batch:P-001:late-mail".to_string(),
            reporting_unit_id: "syn:precinct:P-001".to_string(),
            kind: BatchKind::Mail,
            status: CountStatus::Canvassed,
            accepted_ballots: 10,
            counted_ballots: 10,
            rejected_ballots: 0,
            source_refs: vec!["source:synthetic-summary-export".to_string()],
        },
        BatchManifest {
            batch_id: "batch:P-002:election-day".to_string(),
            reporting_unit_id: "syn:precinct:P-002".to_string(),
            kind: BatchKind::ElectionDay,
            status: CountStatus::Canvassed,
            accepted_ballots: 60,
            counted_ballots: 60,
            rejected_ballots: 0,
            source_refs: vec!["source:synthetic-summary-export".to_string()],
        },
    ];
    package.summaries = vec![
        summary_with_status_and_batch(
            "syn:precinct:P-001",
            CountStatus::Canvassed,
            Some("batch:P-001:election-day"),
            35,
            30,
            1,
            3,
            1,
            0,
        ),
        summary_with_status_and_batch(
            "syn:precinct:P-001",
            CountStatus::Canvassed,
            Some("batch:P-001:late-mail"),
            5,
            5,
            0,
            0,
            0,
            0,
        ),
        summary_with_status_and_batch(
            "syn:precinct:P-002",
            CountStatus::Canvassed,
            Some("batch:P-002:election-day"),
            25,
            30,
            0,
            4,
            0,
            1,
        ),
        summary("syn:jurisdiction:SYN", 65, 65, 1, 7, 1, 1),
    ];
    package.status_events = vec![StatusEvent {
        event_id: "event-0003".to_string(),
        event_type: StatusEventType::LateMailBatchAdded,
        status_before: CountStatus::Unofficial,
        status_after: CountStatus::Canvassed,
        effective_at: "2024-11-08T17:00:00Z".to_string(),
        authority: "SYN County Election Office".to_string(),
        source_refs: vec!["source:synthetic-summary-export".to_string()],
        explanation: "Late-arriving mail batch for P-001 was accepted before canvass.".to_string(),
    }];
    package
}

pub fn synthetic_missing_batch_package() -> RcountPackage {
    let mut package = synthetic_mail_batch_added_package();
    package
        .batches
        .retain(|batch| batch.batch_id != "batch:P-001:late-mail");
    package
}

pub fn synthetic_batch_comparison_package() -> RcountPackage {
    let mut package = synthetic_mail_batch_added_package();
    package.batch_comparison_audits = vec![BatchComparisonAudit {
        audit_id: "batch-comparison:P-001-election-day".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        batch_id: "batch:P-001:election-day".to_string(),
        declared_batch_ballots: 70,
        winner_selection_id: "cand-a".to_string(),
        loser_selection_id: "cand-b".to_string(),
        reported_totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: 35,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: 30,
            },
        ],
        hand_totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: 34,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: 31,
            },
        ],
        declared_reported_margin: 5,
        declared_hand_margin: 3,
        declared_overstatement: 2,
        source_refs: vec!["source:synthetic-batch-hand-tally".to_string()],
    }];
    package
}

pub fn synthetic_batch_comparison_algorithm_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_package();
    package.audit_algorithm_runs = vec![derive_batch_comparison_algorithm_run(
        &package,
        "audit-run:batch-comparison-taint-linkage",
        "syn-2024-mayor",
        300_000,
        &["batch:P-001:election-day".to_string()],
        AuditAlgorithmDecision::Continue,
    )
    .expect("synthetic batch comparison algorithm run must derive")];
    package
}

pub fn synthetic_kaplan_markov_macro_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:kaplan-markov-macro-pass".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: KAPLAN_MARKOV_COMPARISON_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithoutReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(500_000),
        reported_winner_votes: Some(65),
        reported_loser_votes: Some(65),
        macro_ballot_count: Some(100),
        macro_reported_margin: Some(10),
        macro_gamma: Some(RationalValue {
            numerator: 11,
            denominator: 10,
        }),
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
            kind: AuditAssertionKind::ComparisonOverstatement,
            assorter_id: "macro-overstatement-category-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 2,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: (0..16)
            .map(|step_index| AuditSampleStep {
                step_index,
                round_index: None,
                assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                sample_unit_id: format!("ballot:macro:{step_index}"),
                assorter_value: RationalValue {
                    numerator: 0,
                    denominator: 1,
                },
                bet: None,
                statistic: None,
                p_value_ppm: None,
                ranked_choices: Vec::new(),
                source_refs: vec![format!("source:macro-ballot:{step_index}")],
            })
            .collect(),
        decision: AuditAlgorithmDecision::Pass,
        source_refs: vec!["source:synthetic-macro-comparison-audit".to_string()],
    }];
    package
}

pub fn synthetic_minerva_round_one_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:minerva-round-one-pass".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: MINERVA_BALLOT_POLLING_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(100_000),
        reported_winner_votes: Some(3),
        reported_loser_votes: Some(1),
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
        sample_steps: (0..6)
            .map(|step_index| AuditSampleStep {
                step_index,
                round_index: None,
                assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                sample_unit_id: format!("ballot:minerva:{step_index}"),
                assorter_value: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                bet: None,
                statistic: None,
                p_value_ppm: None,
                ranked_choices: Vec::new(),
                source_refs: vec![format!("source:minerva-ballot:{step_index}")],
            })
            .collect(),
        decision: AuditAlgorithmDecision::Pass,
        source_refs: vec!["source:synthetic-minerva-round-one-audit".to_string()],
    }];
    package
}

pub fn synthetic_minerva_multi_round_package() -> RcountPackage {
    let mut package = synthetic_minerva_round_one_package();
    package.audit_algorithm_runs[0].run_id = "audit-run:minerva-multi-round-pass".to_string();
    for step in &mut package.audit_algorithm_runs[0].sample_steps {
        step.round_index = if step.step_index < 5 {
            Some(0)
        } else {
            Some(1)
        };
    }
    package
}

pub fn synthetic_athena_boundary_package() -> RcountPackage {
    let mut package = synthetic_minerva_multi_round_package();
    let run = &mut package.audit_algorithm_runs[0];
    run.run_id = "audit-run:athena-boundary".to_string();
    run.method_id = ATHENA_BALLOT_POLLING_METHOD_ID.to_string();
    run.decision = AuditAlgorithmDecision::Boundary;
    run.source_refs = vec!["source:synthetic-athena-boundary-audit".to_string()];
    package
}

pub fn synthetic_stratified_hybrid_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_algorithm_package();
    let minerva_run = synthetic_minerva_multi_round_package()
        .audit_algorithm_runs
        .into_iter()
        .next()
        .expect("synthetic Minerva package must contain one algorithm run");
    let batch_run = package.audit_algorithm_runs[0].clone();
    package.audit_algorithm_runs.push(minerva_run);
    package.audit_algorithm_runs.push(AuditAlgorithmRun {
        run_id: "audit-run:stratified-hybrid-boundary".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: STRATIFIED_HYBRID_RLA_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::BoundaryOnly,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(100_000),
        reported_winner_votes: None,
        reported_loser_votes: None,
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: Some("suite-nuisance-boundary-v1".to_string()),
        nuisance_parameter: Some(RationalValue {
            numerator: 1,
            denominator: 2,
        }),
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: vec![
            AuditStratum {
                stratum_id: "stratum:batch-comparison".to_string(),
                method_id: batch_run.method_id,
                component_run_id: batch_run.run_id,
                ballot_count: Some(80),
                allocation_ppm: Some(500_000),
                source_refs: vec!["source:synthetic-batch-hand-tally".to_string()],
            },
            AuditStratum {
                stratum_id: "stratum:ballot-polling".to_string(),
                method_id: MINERVA_BALLOT_POLLING_METHOD_ID.to_string(),
                component_run_id: "audit-run:minerva-multi-round-pass".to_string(),
                ballot_count: Some(60),
                allocation_ppm: Some(500_000),
                source_refs: vec!["source:synthetic-minerva-round-one-audit".to_string()],
            },
        ],
        assertions: Vec::new(),
        sample_steps: Vec::new(),
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec![
            "audit-run:batch-comparison-taint-linkage".to_string(),
            "audit-run:minerva-multi-round-pass".to_string(),
        ],
    });
    package
}

pub fn synthetic_bad_stratified_hybrid_package() -> RcountPackage {
    let mut package = synthetic_stratified_hybrid_package();
    package.audit_algorithm_runs[2].strata[1].component_run_id =
        "audit-run:missing-stratum".to_string();
    package
}

pub fn synthetic_flattened_stratified_hybrid_package() -> RcountPackage {
    let mut package = synthetic_stratified_hybrid_package();
    package.audit_algorithm_runs[2].strata.truncate(1);
    package.audit_algorithm_runs[2].strata[0].allocation_ppm = Some(1_000_000);
    package
}

pub fn synthetic_raire_boundary_package() -> RcountPackage {
    synthetic_ranked_choice_boundary_package(
        RAIRE_IRV_METHOD_ID,
        "audit-run:raire-irv-boundary",
        "raire-neb-not-eliminated-before-v1",
    )
}

pub fn synthetic_awaire_boundary_package() -> RcountPackage {
    synthetic_ranked_choice_boundary_package(
        AWAIRE_IRV_METHOD_ID,
        "audit-run:awaire-irv-boundary",
        "awaire-adaptive-irv-v1",
    )
}

pub fn synthetic_bad_raire_boundary_package() -> RcountPackage {
    let mut package = synthetic_raire_boundary_package();
    package.audit_algorithm_runs[0].sample_steps[0]
        .ranked_choices
        .push("cand-a".to_string());
    package
}

pub fn synthetic_bayesian_tabulation_boundary_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:bayesian-tabulation-boundary".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: BAYESIAN_TABULATION_AUDIT_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::BoundaryOnly,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: None,
        reported_winner_votes: None,
        reported_loser_votes: None,
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: Some("dirichlet-multinomial-toy-prior-v1".to_string()),
        bayesian_likelihood_id: Some("sample-counts-without-replacement-v1".to_string()),
        posterior_winner_probability_ppm: Some(958_000),
        posterior_risk_ppm: Some(42_000),
        simulation_seed: Some(20_240_513),
        posterior_draws: Some(10_000),
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:bayesian-cand-a-outcome".to_string(),
            kind: AuditAssertionKind::BayesianOutcome,
            assorter_id: "bayesian-posterior-winner-probability-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: Vec::new(),
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec!["source:synthetic-bayesian-tabulation-audit".to_string()],
    }];
    package
}

pub fn synthetic_bad_bayesian_tabulation_boundary_package() -> RcountPackage {
    let mut package = synthetic_bayesian_tabulation_boundary_package();
    package.audit_algorithm_runs[0].posterior_risk_ppm = Some(1_000_001);
    package
}

pub(crate) fn synthetic_ranked_choice_boundary_package(
    method_id: &str,
    run_id: &str,
    assorter_id: &str,
) -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: run_id.to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: method_id.to_string(),
        sampling_mode: AuditSamplingMode::WithoutReplacement,
        rcv_elimination_order: vec![
            "cand-c".to_string(),
            "cand-b".to_string(),
            "cand-a".to_string(),
        ],
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
            assertion_id: "assertion:irv-cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::RankedChoiceAssertion,
            assorter_id: assorter_id.to_string(),
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
                assertion_id: "assertion:irv-cand-a-over-cand-b".to_string(),
                sample_unit_id: "ranked-ballot:0".to_string(),
                assorter_value: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                bet: None,
                statistic: None,
                p_value_ppm: None,
                ranked_choices: vec![
                    "cand-a".to_string(),
                    "cand-b".to_string(),
                    "cand-c".to_string(),
                ],
                source_refs: vec!["source:synthetic-ranked-cvr:0".to_string()],
            },
            AuditSampleStep {
                step_index: 1,
                round_index: None,
                assertion_id: "assertion:irv-cand-a-over-cand-b".to_string(),
                sample_unit_id: "ranked-ballot:1".to_string(),
                assorter_value: RationalValue {
                    numerator: 0,
                    denominator: 1,
                },
                bet: None,
                statistic: None,
                p_value_ppm: None,
                ranked_choices: vec![
                    "cand-b".to_string(),
                    "cand-a".to_string(),
                    "cand-c".to_string(),
                ],
                source_refs: vec!["source:synthetic-ranked-cvr:1".to_string()],
            },
        ],
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec!["source:synthetic-ranked-choice-audit".to_string()],
    }];
    package
}

pub fn synthetic_bad_batch_comparison_algorithm_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_algorithm_package();
    package.audit_algorithm_runs[0].sample_steps[0].assorter_value = RationalValue {
        numerator: 1,
        denominator: 5,
    };
    package
}

pub fn synthetic_bad_batch_comparison_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_package();
    package.batch_comparison_audits[0].declared_overstatement = 0;
    package
}

pub fn synthetic_missing_hand_tally_batch_comparison_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_package();
    package.batch_comparison_audits[0]
        .hand_totals
        .retain(|total| total.selection_id != "cand-b");
    package
}

pub fn synthetic_batch_size_drift_comparison_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_package();
    package.batch_comparison_audits[0].declared_batch_ballots = 69;
    package
}

pub fn synthetic_precinct_split_lineage_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.reporting_units.extend([
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-004".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-004".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: Some("2028-11-07".to_string()),
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-004A".to_string(),
            kind: ReportingUnitKind::SplitPrecinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-004A".to_string()],
            valid_from: Some("2028-11-07".to_string()),
            valid_to: None,
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-004B".to_string(),
            kind: ReportingUnitKind::SplitPrecinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-004B".to_string()],
            valid_from: Some("2028-11-07".to_string()),
            valid_to: None,
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-007".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-007".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: Some("2028-11-07".to_string()),
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-008".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-008".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: Some("2028-11-07".to_string()),
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-078".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-078".to_string()],
            valid_from: Some("2028-11-07".to_string()),
            valid_to: None,
        },
    ]);
    package.lineage = vec![
        ReportingUnitLineage {
            lineage_id: "lineage:P-004-split".to_string(),
            kind: LineageKind::Split,
            prior_cycle: "SYN-2024-general".to_string(),
            current_cycle: "SYN-2028-general".to_string(),
            prior_reporting_unit_ids: vec!["syn:precinct:P-004".to_string()],
            current_reporting_unit_ids: vec![
                "syn:precinct:P-004A".to_string(),
                "syn:precinct:P-004B".to_string(),
            ],
            authority: "SYN County Election Office".to_string(),
            explanation: "P-004 split into two precincts after municipal growth.".to_string(),
        },
        ReportingUnitLineage {
            lineage_id: "lineage:P-007-P-008-merge".to_string(),
            kind: LineageKind::Merge,
            prior_cycle: "SYN-2024-general".to_string(),
            current_cycle: "SYN-2028-general".to_string(),
            prior_reporting_unit_ids: vec![
                "syn:precinct:P-007".to_string(),
                "syn:precinct:P-008".to_string(),
            ],
            current_reporting_unit_ids: vec!["syn:precinct:P-078".to_string()],
            authority: "SYN County Election Office".to_string(),
            explanation: "P-007 and P-008 merged into P-078 during precinct consolidation."
                .to_string(),
        },
    ];
    package
}

pub fn synthetic_bad_lineage_package() -> RcountPackage {
    let mut package = synthetic_precinct_split_lineage_package();
    package.lineage[0]
        .current_reporting_unit_ids
        .push("syn:precinct:P-004C".to_string());
    package
}

pub fn synthetic_privacy_inclusion_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.inclusion_proofs = vec![InclusionProof {
        proof_id: "proof:accepted-token-001".to_string(),
        kind: InclusionProofKind::AnonymizedAcceptedBallotToken,
        token_hash: format!("sha256:{}", "a".repeat(64)),
        reporting_unit_id: "syn:precinct:P-001".to_string(),
        candidate_selections: vec![],
        voter_id: None,
        ballot_style: None,
        issued_at: None,
    }];
    package
}

pub fn synthetic_choice_bearing_proof_package() -> RcountPackage {
    let mut package = synthetic_privacy_inclusion_package();
    package.inclusion_proofs[0].candidate_selections = vec!["cand-a".to_string()];
    package
}

pub fn synthetic_soba_observable_ballot_boundary_package() -> RcountPackage {
    let mut package = synthetic_privacy_inclusion_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:soba-observable-ballot-boundary".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::BoundaryOnly,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: None,
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
            assertion_id: "assertion:observable-ballot-opening".to_string(),
            kind: AuditAssertionKind::ObservableBallotLinkage,
            assorter_id: "soba-commitment-opening-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: None,
            loser_selection_id: None,
        }],
        sample_steps: vec![AuditSampleStep {
            step_index: 0,
            round_index: None,
            assertion_id: "assertion:observable-ballot-opening".to_string(),
            sample_unit_id: "proof:accepted-token-001".to_string(),
            assorter_value: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            bet: None,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: vec!["source:synthetic-soba-opening".to_string()],
        }],
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec!["source:synthetic-soba-observable-ballot-audit".to_string()],
    }];
    package
}

pub fn synthetic_missing_soba_opening_package() -> RcountPackage {
    let mut package = synthetic_soba_observable_ballot_boundary_package();
    package.audit_algorithm_runs[0].sample_steps[0].sample_unit_id =
        "proof:missing-token".to_string();
    package
}

pub fn synthetic_cvr_summary_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.cvr = vec![];
    append_cvr_rows(
        &mut package.cvr,
        "P-001",
        "syn:precinct:P-001",
        40,
        35,
        1,
        3,
        1,
        0,
    );
    append_cvr_rows(
        &mut package.cvr,
        "P-002",
        "syn:precinct:P-002",
        25,
        30,
        0,
        4,
        0,
        1,
    );
    package
}

pub fn synthetic_bad_cvr_summary_package() -> RcountPackage {
    let mut package = synthetic_cvr_summary_package();
    let row = package
        .cvr
        .iter_mut()
        .find(|row| {
            row.reporting_unit_id == "syn:precinct:P-001"
                && row.selection_ids.len() == 1
                && row.selection_ids[0] == "cand-a"
        })
        .expect("synthetic CVR package must contain a Candidate A row");
    row.selection_ids = vec!["cand-b".to_string()];
    package
}

pub fn synthetic_rla_replay_package() -> RcountPackage {
    let mut package = synthetic_cvr_summary_package();
    let manifest_hash = rla_contest_manifest_hash(&package, "syn-2024-mayor")
        .expect("synthetic CVR package must have an RLA population");
    let mut audit = RiskLimitAudit {
        audit_id: "rla:syn-2024-mayor:round-1".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        jurisdiction_method_id: None,
        ballot_manifest_format_id: None,
        audit_software_id: None,
        audit_software_source_url: None,
        risk_limit_ppm: 50_000,
        public_seed: "31415926535897932384".to_string(),
        sampling_algorithm_id: RLA_SAMPLING_ALGORITHM_ID.to_string(),
        manifest_hash,
        sample_size: 12,
        sample_draws: vec![],
        observations: vec![],
        discrepancies: vec![],
        margin: None,
        stopping_rule_id: None,
        max_discrepancies: None,
        declared_status: None,
        declared_risk_ppm: None,
    };
    audit.sample_draws =
        replay_rla_sample(&package, &audit).expect("synthetic RLA sample must replay");
    package.rla_audits = vec![audit];
    package
}

pub fn synthetic_bad_rla_replay_package() -> RcountPackage {
    let mut package = synthetic_rla_replay_package();
    package.rla_audits[0].sample_draws[0].cvr_id = "cvr:P-999:999".to_string();
    package
}

pub fn synthetic_rla_stopping_package() -> RcountPackage {
    let mut package = synthetic_rla_replay_package();
    let observations = rla_observations_from_sample(&package, &package.rla_audits[0])
        .expect("synthetic RLA observations must match sample");
    let audit = &mut package.rla_audits[0];
    audit.observations = observations;
    audit.stopping_rule_id = Some("zero-discrepancy-threshold-v1".to_string());
    audit.max_discrepancies = Some(0);
    audit.declared_status = Some(RlaStoppingStatus::Pass);
    package
}

pub fn synthetic_rla_margin_package() -> RcountPackage {
    let mut package = synthetic_rla_stopping_package();
    package.rla_audits[0].margin = Some(RlaMarginMetadata {
        winner_selection_id: "cand-a".to_string(),
        loser_selection_id: "write-in".to_string(),
        reported_winner_votes: 65,
        reported_loser_votes: 1,
        reported_margin: 64,
        diluted_margin_denominator: 140,
    });
    package
}

pub fn synthetic_bad_rla_margin_package() -> RcountPackage {
    let mut package = synthetic_rla_margin_package();
    package.rla_audits[0]
        .margin
        .as_mut()
        .expect("synthetic RLA margin package must contain margin")
        .reported_margin += 1;
    package
}

pub fn synthetic_rla_statistical_package() -> RcountPackage {
    let mut package = synthetic_rla_margin_package();
    let risk_ppm = comparison_margin_risk_ppm(&package.rla_audits[0]);
    let audit = &mut package.rla_audits[0];
    audit.stopping_rule_id = Some("comparison-margin-threshold-v1".to_string());
    audit.max_discrepancies = Some(0);
    audit.declared_status = Some(RlaStoppingStatus::Pass);
    audit.declared_risk_ppm = Some(risk_ppm);
    package
}

pub fn synthetic_bad_rla_statistical_package() -> RcountPackage {
    let mut package = synthetic_rla_statistical_package();
    package.rla_audits[0].declared_risk_ppm = Some(
        package.rla_audits[0]
            .declared_risk_ppm
            .expect("synthetic statistical package must contain risk")
            + 1,
    );
    package
}

pub fn synthetic_colorado_rla_package() -> RcountPackage {
    let mut package = synthetic_rla_statistical_package();
    package.rla_audits[0].jurisdiction_method_id = Some(COLORADO_RLA_METHOD_ID.to_string());
    package
}

pub fn synthetic_bad_colorado_rla_package() -> RcountPackage {
    let mut package = synthetic_colorado_rla_package();
    package.rla_audits[0].public_seed = "3141592653589793238X".to_string();
    package.rla_audits[0].sample_draws =
        replay_rla_sample(&package, &package.rla_audits[0]).expect("bad seed still replays");
    package.rla_audits[0].observations =
        rla_observations_from_sample(&package, &package.rla_audits[0])
            .expect("bad Colorado seed package must still have matching observations");
    package
}

pub fn synthetic_california_rla_package() -> RcountPackage {
    let mut package = synthetic_rla_statistical_package();
    let audit = &mut package.rla_audits[0];
    audit.jurisdiction_method_id = Some(CALIFORNIA_RLA_METHOD_ID.to_string());
    audit.ballot_manifest_format_id = Some(CALIFORNIA_BALLOT_MANIFEST_FORMAT_ID.to_string());
    audit.audit_software_id = Some("rcount-open-rla-synthetic-v1".to_string());
    audit.audit_software_source_url = Some(
        "https://github.com/synthetic-election-audit/rcount-open-rla-synthetic-v1".to_string(),
    );
    package
}

pub fn synthetic_bad_california_rla_package() -> RcountPackage {
    let mut package = synthetic_california_rla_package();
    package.rla_audits[0].audit_software_source_url =
        Some("synthetic-election-audit/rcount-open-rla-synthetic-v1".to_string());
    package
}

pub fn synthetic_manual_audit_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.manual_audits = vec![ManualAudit {
        audit_id: "manual-audit:syn-2024-mayor:P-001".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        reporting_unit_id: "syn:precinct:P-001".to_string(),
        authority: "SYN County Canvassing Board".to_string(),
        audited_batch_ids: vec![],
        tolerance_votes: 0,
        machine_totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: 40,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: 35,
            },
            SelectionTotal {
                selection_id: "write-in".to_string(),
                votes: 1,
            },
        ],
        hand_totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: 40,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: 35,
            },
            SelectionTotal {
                selection_id: "write-in".to_string(),
                votes: 1,
            },
        ],
        declared_status: ManualAuditStatus::Pass,
    }];
    package
}

pub fn synthetic_bad_manual_audit_package() -> RcountPackage {
    let mut package = synthetic_manual_audit_package();
    package.manual_audits[0].hand_totals[1].votes += 1;
    package
}

pub fn synthetic_bad_rla_stopping_package() -> RcountPackage {
    let mut package = synthetic_rla_stopping_package();
    package.rla_audits[0].observations[0].observed_selection_ids = vec!["cand-b".to_string()];
    package.rla_audits[0].discrepancies = vec![RlaDiscrepancy {
        draw_index: package.rla_audits[0].sample_draws[0].draw_index,
        cvr_id: package.rla_audits[0].sample_draws[0].cvr_id.clone(),
        kind: RlaDiscrepancyKind::SelectionMismatch,
    }];
    package
}

pub fn synthetic_rla_discrepancy_package() -> RcountPackage {
    let mut package = synthetic_bad_rla_stopping_package();
    package.rla_audits[0].declared_status = Some(RlaStoppingStatus::Escalate);
    package
}

pub fn synthetic_bad_rla_discrepancy_package() -> RcountPackage {
    let mut package = synthetic_rla_discrepancy_package();
    package.rla_audits[0].discrepancies[0].kind = RlaDiscrepancyKind::ResidualMismatch;
    package
}

pub(crate) fn rla_observations_from_sample(
    package: &RcountPackage,
    audit: &RiskLimitAudit,
) -> Result<Vec<RlaSampleObservation>, RcountCoreError> {
    let cvr_by_id: BTreeMap<&str, &CvrContestRecord> = package
        .cvr
        .iter()
        .filter(|row| row.contest_id == audit.contest_id)
        .map(|row| (row.cvr_id.as_str(), row))
        .collect();
    let mut observations = Vec::with_capacity(audit.sample_draws.len());
    for draw in &audit.sample_draws {
        let cvr = cvr_by_id.get(draw.cvr_id.as_str()).ok_or_else(|| {
            RcountCoreError::MissingRlaPopulation {
                audit_id: audit.audit_id.clone(),
                contest_id: audit.contest_id.clone(),
            }
        })?;
        observations.push(RlaSampleObservation {
            draw_index: draw.draw_index,
            cvr_id: draw.cvr_id.clone(),
            observed_selection_ids: cvr.selection_ids.clone(),
            undervote: cvr.undervote,
            overvote: cvr.overvote,
            blank_contest: cvr.blank_contest,
        });
    }
    Ok(observations)
}

pub(crate) fn summary(
    reporting_unit_id: &str,
    cand_a: i64,
    cand_b: i64,
    write_in: i64,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
) -> Summary {
    summary_with_status_and_batch(
        reporting_unit_id,
        CountStatus::Canvassed,
        None,
        cand_a,
        cand_b,
        write_in,
        undervotes,
        overvotes,
        blank_contests,
    )
}

pub(crate) fn summary_with_status(
    reporting_unit_id: &str,
    status: CountStatus,
    cand_a: i64,
    cand_b: i64,
    write_in: i64,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
) -> Summary {
    summary_with_status_and_batch(
        reporting_unit_id,
        status,
        None,
        cand_a,
        cand_b,
        write_in,
        undervotes,
        overvotes,
        blank_contests,
    )
}

pub(crate) fn summary_with_status_and_batch(
    reporting_unit_id: &str,
    status: CountStatus,
    batch_id: Option<&str>,
    cand_a: i64,
    cand_b: i64,
    write_in: i64,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
) -> Summary {
    Summary {
        contest_id: "syn-2024-mayor".to_string(),
        reporting_unit_id: reporting_unit_id.to_string(),
        batch_id: batch_id.map(str::to_string),
        status,
        totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: cand_a,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: cand_b,
            },
            SelectionTotal {
                selection_id: "write-in".to_string(),
                votes: write_in,
            },
        ],
        undervotes,
        overvotes,
        blank_contests,
        counted_ballots: cand_a + cand_b + write_in + undervotes + overvotes + blank_contests,
    }
}

pub(crate) fn append_cvr_rows(
    rows: &mut Vec<CvrContestRecord>,
    id_prefix: &str,
    reporting_unit_id: &str,
    cand_a: i64,
    cand_b: i64,
    write_in: i64,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
) {
    let mut ordinal = 1usize;
    for (selection_id, count) in [
        ("cand-a", cand_a),
        ("cand-b", cand_b),
        ("write-in", write_in),
    ] {
        for _ in 0..count {
            rows.push(cvr_selection_row(
                id_prefix,
                ordinal,
                reporting_unit_id,
                selection_id,
            ));
            ordinal += 1;
        }
    }
    for _ in 0..undervotes {
        rows.push(cvr_residual_row(
            id_prefix,
            ordinal,
            reporting_unit_id,
            "undervote",
        ));
        ordinal += 1;
    }
    for _ in 0..overvotes {
        rows.push(cvr_residual_row(
            id_prefix,
            ordinal,
            reporting_unit_id,
            "overvote",
        ));
        ordinal += 1;
    }
    for _ in 0..blank_contests {
        rows.push(cvr_residual_row(
            id_prefix,
            ordinal,
            reporting_unit_id,
            "blank",
        ));
        ordinal += 1;
    }
}

pub(crate) fn cvr_selection_row(
    id_prefix: &str,
    ordinal: usize,
    reporting_unit_id: &str,
    selection_id: &str,
) -> CvrContestRecord {
    CvrContestRecord {
        cvr_id: format!("cvr:{id_prefix}:{ordinal:03}"),
        contest_id: "syn-2024-mayor".to_string(),
        reporting_unit_id: reporting_unit_id.to_string(),
        batch_id: None,
        status: CountStatus::Canvassed,
        selection_ids: vec![selection_id.to_string()],
        undervote: false,
        overvote: false,
        blank_contest: false,
        source_refs: vec!["source:synthetic-summary-export".to_string()],
    }
}

pub(crate) fn cvr_residual_row(
    id_prefix: &str,
    ordinal: usize,
    reporting_unit_id: &str,
    residual: &str,
) -> CvrContestRecord {
    CvrContestRecord {
        cvr_id: format!("cvr:{id_prefix}:{ordinal:03}"),
        contest_id: "syn-2024-mayor".to_string(),
        reporting_unit_id: reporting_unit_id.to_string(),
        batch_id: None,
        status: CountStatus::Canvassed,
        selection_ids: vec![],
        undervote: residual == "undervote",
        overvote: residual == "overvote",
        blank_contest: residual == "blank",
        source_refs: vec!["source:synthetic-summary-export".to_string()],
    }
}
