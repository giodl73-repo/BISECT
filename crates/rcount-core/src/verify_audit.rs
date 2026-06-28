use crate::*;

pub fn verify_rla_sampler_replay(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for audit in &package.rla_audits {
        if !seen.insert(audit.audit_id.as_str()) {
            return Err(RcountCoreError::DuplicateRlaAuditId {
                audit_id: audit.audit_id.clone(),
            });
        }
        if audit.risk_limit_ppm == 0 || audit.risk_limit_ppm >= 1_000_000 {
            return Err(RcountCoreError::InvalidRlaRiskLimit {
                audit_id: audit.audit_id.clone(),
                risk_limit_ppm: audit.risk_limit_ppm,
            });
        }
        if audit.sample_size == 0 || audit.sample_draws.len() != audit.sample_size as usize {
            return Err(RcountCoreError::InvalidRlaSampleSize {
                audit_id: audit.audit_id.clone(),
                sample_size: audit.sample_size,
            });
        }
        if audit.sampling_algorithm_id != RLA_SAMPLING_ALGORITHM_ID {
            return Err(RcountCoreError::UnsupportedRlaSamplingAlgorithm {
                audit_id: audit.audit_id.clone(),
                sampling_algorithm_id: audit.sampling_algorithm_id.clone(),
            });
        }
        let computed_manifest_hash =
            rla_contest_manifest_hash_for_audit(package, &audit.contest_id, &audit.audit_id)?;
        if audit.manifest_hash != computed_manifest_hash {
            return Err(RcountCoreError::RlaManifestHashMismatch {
                audit_id: audit.audit_id.clone(),
                declared: audit.manifest_hash.clone(),
                computed: computed_manifest_hash,
            });
        }
        let expected = replay_rla_sample(package, audit)?;
        for (declared, computed) in audit.sample_draws.iter().zip(expected.iter()) {
            if declared.draw_index != computed.draw_index || declared.cvr_id != computed.cvr_id {
                return Err(RcountCoreError::RlaSampleMismatch {
                    audit_id: audit.audit_id.clone(),
                    draw_index: computed.draw_index,
                    declared_cvr_id: declared.cvr_id.clone(),
                    computed_cvr_id: computed.cvr_id.clone(),
                });
            }
        }
        passes.push(EquationPass {
            equation_id: "rla_sampler_replay".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.audit_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_audit_algorithm_runs(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen_runs = BTreeSet::new();
    for run in &package.audit_algorithm_runs {
        if !seen_runs.insert(run.run_id.as_str()) {
            return Err(RcountCoreError::DuplicateAuditAlgorithmRunId {
                run_id: run.run_id.clone(),
            });
        }
    }

    let mut passes = Vec::new();
    for run in &package.audit_algorithm_runs {
        if !is_supported_audit_algorithm_method(&run.method_id) {
            return Err(RcountCoreError::UnsupportedAuditAlgorithmMethod {
                run_id: run.run_id.clone(),
                method_id: run.method_id.clone(),
            });
        }
        if let Some(risk_limit_ppm) = run.risk_limit_ppm {
            if risk_limit_ppm == 0 || risk_limit_ppm >= 1_000_000 {
                return Err(RcountCoreError::InvalidAuditAlgorithmRiskLimit {
                    run_id: run.run_id.clone(),
                    risk_limit_ppm,
                });
            }
        }
        verify_audit_macro_design(run)?;
        verify_stratified_hybrid_design(run, &seen_runs)?;
        verify_ranked_choice_audit_design(run)?;
        verify_bayesian_audit_design(run)?;
        verify_soba_observable_ballot_design(package, run)?;

        let mut assertions = BTreeMap::new();
        for assertion in &run.assertions {
            if assertions
                .insert(assertion.assertion_id.as_str(), assertion)
                .is_some()
            {
                return Err(RcountCoreError::DuplicateAuditAssertion {
                    run_id: run.run_id.clone(),
                    assertion_id: assertion.assertion_id.clone(),
                });
            }
            if assertion.assorter_id.trim().is_empty()
                || !is_positive_rational(assertion.assorter_upper_bound)
            {
                return Err(RcountCoreError::InvalidAuditAssorterBound {
                    run_id: run.run_id.clone(),
                    assertion_id: assertion.assertion_id.clone(),
                });
            }
        }

        let mut seen_steps = BTreeSet::new();
        for step in &run.sample_steps {
            let Some(assertion) = assertions.get(step.assertion_id.as_str()) else {
                return Err(RcountCoreError::MissingAuditAssertion {
                    run_id: run.run_id.clone(),
                    step_index: step.step_index,
                    assertion_id: step.assertion_id.clone(),
                });
            };
            if !seen_steps.insert((step.assertion_id.as_str(), step.step_index)) {
                return Err(RcountCoreError::DuplicateAuditSampleStep {
                    run_id: run.run_id.clone(),
                    assertion_id: step.assertion_id.clone(),
                    step_index: step.step_index,
                });
            }
            if !is_non_negative_rational(step.assorter_value)
                || rational_gt(step.assorter_value, assertion.assorter_upper_bound)
                || step.bet.is_some_and(|bet| !has_positive_denominator(bet))
                || step
                    .statistic
                    .is_some_and(|statistic| !is_non_negative_rational(statistic))
            {
                return Err(RcountCoreError::InvalidAuditAssorterValue {
                    run_id: run.run_id.clone(),
                    step_index: step.step_index,
                });
            }
            if step.p_value_ppm.is_some_and(|p_value| p_value > 1_000_000) {
                return Err(RcountCoreError::InvalidAuditPValue {
                    run_id: run.run_id.clone(),
                    step_index: step.step_index,
                    p_value_ppm: step.p_value_ppm.unwrap(),
                });
            }
        }

        passes.push(EquationPass {
            equation_id: "audit_algorithm_transcript".to_string(),
            contest_id: run.contest_id.clone(),
            reporting_unit_id: run.run_id.clone(),
        });
        if run.method_id == BATCH_COMPARISON_METHOD_ID
            && package
                .batch_comparison_audits
                .iter()
                .any(|audit| audit.contest_id == run.contest_id)
        {
            verify_batch_comparison_algorithm_linkage(package, run)?;
            passes.push(EquationPass {
                equation_id: "batch_comparison_algorithm_linkage".to_string(),
                contest_id: run.contest_id.clone(),
                reporting_unit_id: run.run_id.clone(),
            });
        }
    }
    Ok(passes)
}

pub(crate) fn verify_audit_macro_design(run: &AuditAlgorithmRun) -> Result<(), RcountCoreError> {
    match (
        run.macro_ballot_count,
        run.macro_reported_margin,
        run.macro_gamma,
    ) {
        (None, None, None) => Ok(()),
        (Some(ballot_count), Some(reported_margin), Some(gamma))
            if ballot_count > 0
                && reported_margin > 0
                && gamma.denominator > 0
                && rational_gt(
                    gamma,
                    RationalValue {
                        numerator: 1,
                        denominator: 1,
                    },
                ) =>
        {
            Ok(())
        }
        _ => Err(RcountCoreError::InvalidAuditMacroDesign {
            run_id: run.run_id.clone(),
        }),
    }
}

pub(crate) fn verify_stratified_hybrid_design(
    run: &AuditAlgorithmRun,
    run_ids: &BTreeSet<&str>,
) -> Result<(), RcountCoreError> {
    if run.method_id != STRATIFIED_HYBRID_RLA_METHOD_ID {
        if run.combining_rule_id.is_some()
            || run.nuisance_parameter.is_some()
            || !run.strata.is_empty()
        {
            return Err(RcountCoreError::InvalidStratifiedHybridDesign {
                run_id: run.run_id.clone(),
            });
        }
        return Ok(());
    }

    if run
        .combining_rule_id
        .as_deref()
        .is_none_or(|rule| rule.trim().is_empty())
        || run
            .nuisance_parameter
            .is_none_or(|parameter| !has_positive_denominator(parameter))
        || run.strata.len() < 2
        || !run.assertions.is_empty()
        || !run.sample_steps.is_empty()
    {
        return Err(RcountCoreError::InvalidStratifiedHybridDesign {
            run_id: run.run_id.clone(),
        });
    }

    let mut seen_strata = BTreeSet::new();
    let mut allocation_sum = 0_u32;
    for stratum in &run.strata {
        let Some(allocation_ppm) = stratum.allocation_ppm else {
            return Err(RcountCoreError::InvalidStratifiedHybridDesign {
                run_id: run.run_id.clone(),
            });
        };
        if stratum.stratum_id.trim().is_empty()
            || !seen_strata.insert(stratum.stratum_id.as_str())
            || stratum.component_run_id == run.run_id
            || !is_supported_audit_algorithm_method(&stratum.method_id)
            || stratum.method_id == STRATIFIED_HYBRID_RLA_METHOD_ID
            || stratum.ballot_count.is_some_and(|ballots| ballots == 0)
            || allocation_ppm == 0
            || allocation_ppm > 1_000_000
        {
            return Err(RcountCoreError::InvalidStratifiedHybridDesign {
                run_id: run.run_id.clone(),
            });
        }
        allocation_sum = allocation_sum.saturating_add(allocation_ppm);
        if !run_ids.contains(stratum.component_run_id.as_str()) {
            return Err(RcountCoreError::MissingStratifiedHybridComponent {
                run_id: run.run_id.clone(),
                component_run_id: stratum.component_run_id.clone(),
            });
        }
    }
    if allocation_sum != 1_000_000 {
        return Err(RcountCoreError::InvalidStratifiedHybridDesign {
            run_id: run.run_id.clone(),
        });
    }
    Ok(())
}

pub(crate) fn verify_ranked_choice_audit_design(
    run: &AuditAlgorithmRun,
) -> Result<(), RcountCoreError> {
    let is_ranked_method = matches!(
        run.method_id.as_str(),
        RAIRE_IRV_METHOD_ID | AWAIRE_IRV_METHOD_ID
    );
    if !is_ranked_method {
        if !run.rcv_elimination_order.is_empty()
            || run
                .sample_steps
                .iter()
                .any(|step| !step.ranked_choices.is_empty())
        {
            return Err(RcountCoreError::InvalidRankedChoiceAuditDesign {
                run_id: run.run_id.clone(),
            });
        }
        return Ok(());
    }

    if run.rcv_elimination_order.len() < 2
        || run.assertions.is_empty()
        || run
            .assertions
            .iter()
            .any(|assertion| assertion.kind != AuditAssertionKind::RankedChoiceAssertion)
    {
        return Err(RcountCoreError::InvalidRankedChoiceAuditDesign {
            run_id: run.run_id.clone(),
        });
    }

    let candidates = run
        .rcv_elimination_order
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if candidates.len() != run.rcv_elimination_order.len() {
        return Err(RcountCoreError::InvalidRankedChoiceAuditDesign {
            run_id: run.run_id.clone(),
        });
    }

    for step in &run.sample_steps {
        if step.ranked_choices.is_empty() {
            return Err(RcountCoreError::InvalidRankedChoiceSample {
                run_id: run.run_id.clone(),
                step_index: step.step_index,
            });
        }
        let mut seen_choices = BTreeSet::new();
        for choice in &step.ranked_choices {
            if !candidates.contains(choice.as_str()) || !seen_choices.insert(choice.as_str()) {
                return Err(RcountCoreError::InvalidRankedChoiceSample {
                    run_id: run.run_id.clone(),
                    step_index: step.step_index,
                });
            }
        }
    }

    Ok(())
}

pub(crate) fn verify_bayesian_audit_design(run: &AuditAlgorithmRun) -> Result<(), RcountCoreError> {
    if run.method_id != BAYESIAN_TABULATION_AUDIT_METHOD_ID {
        if run.bayesian_prior_id.is_some()
            || run.bayesian_likelihood_id.is_some()
            || run.posterior_winner_probability_ppm.is_some()
            || run.posterior_risk_ppm.is_some()
            || run.simulation_seed.is_some()
            || run.posterior_draws.is_some()
            || run.calibrated_risk_limit_ppm.is_some()
        {
            return Err(RcountCoreError::InvalidBayesianAuditDesign {
                run_id: run.run_id.clone(),
            });
        }
        return Ok(());
    }

    if run
        .bayesian_prior_id
        .as_deref()
        .is_none_or(|prior| prior.trim().is_empty())
        || run
            .bayesian_likelihood_id
            .as_deref()
            .is_none_or(|likelihood| likelihood.trim().is_empty())
        || run
            .posterior_winner_probability_ppm
            .is_none_or(|probability| probability > 1_000_000)
        || run.posterior_risk_ppm.is_none_or(|risk| risk > 1_000_000)
        || run.posterior_draws.is_some_and(|draws| draws == 0)
        || run
            .calibrated_risk_limit_ppm
            .is_some_and(|limit| limit == 0 || limit >= 1_000_000)
        || run.assertions.is_empty()
        || run
            .assertions
            .iter()
            .any(|assertion| assertion.kind != AuditAssertionKind::BayesianOutcome)
    {
        return Err(RcountCoreError::InvalidBayesianAuditDesign {
            run_id: run.run_id.clone(),
        });
    }

    Ok(())
}

pub(crate) fn verify_soba_observable_ballot_design(
    package: &RcountPackage,
    run: &AuditAlgorithmRun,
) -> Result<(), RcountCoreError> {
    if run.method_id != SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID {
        if run
            .assertions
            .iter()
            .any(|assertion| assertion.kind == AuditAssertionKind::ObservableBallotLinkage)
        {
            return Err(RcountCoreError::InvalidObservableBallotAuditDesign {
                run_id: run.run_id.clone(),
            });
        }
        return Ok(());
    }

    if run.assertions.is_empty()
        || run.sample_steps.is_empty()
        || run
            .assertions
            .iter()
            .any(|assertion| assertion.kind != AuditAssertionKind::ObservableBallotLinkage)
    {
        return Err(RcountCoreError::InvalidObservableBallotAuditDesign {
            run_id: run.run_id.clone(),
        });
    }

    let proofs = package
        .inclusion_proofs
        .iter()
        .map(|proof| (proof.proof_id.as_str(), proof))
        .collect::<BTreeMap<_, _>>();
    for step in &run.sample_steps {
        let proof = proofs.get(step.sample_unit_id.as_str()).ok_or_else(|| {
            RcountCoreError::MissingObservableBallotOpening {
                run_id: run.run_id.clone(),
                step_index: step.step_index,
                proof_id: step.sample_unit_id.clone(),
            }
        })?;
        if !proof.candidate_selections.is_empty() || proof.voter_id.is_some() {
            return Err(RcountCoreError::InvalidObservableBallotAuditDesign {
                run_id: run.run_id.clone(),
            });
        }
    }

    Ok(())
}

pub(crate) fn verify_batch_comparison_algorithm_linkage(
    package: &RcountPackage,
    run: &AuditAlgorithmRun,
) -> Result<(), RcountCoreError> {
    let audits_by_batch = package
        .batch_comparison_audits
        .iter()
        .filter(|audit| audit.contest_id == run.contest_id)
        .map(|audit| (audit.batch_id.as_str(), audit))
        .collect::<BTreeMap<_, _>>();

    for step in &run.sample_steps {
        let audit = audits_by_batch
            .get(step.sample_unit_id.as_str())
            .ok_or_else(
                || RcountCoreError::MissingBatchComparisonAlgorithmEvidence {
                    run_id: run.run_id.clone(),
                    step_index: step.step_index,
                    batch_id: step.sample_unit_id.clone(),
                },
            )?;
        let computed = RationalValue {
            numerator: audit.declared_overstatement,
            denominator: audit.declared_reported_margin,
        };
        if !rational_eq(step.assorter_value, computed) {
            return Err(RcountCoreError::BatchComparisonAlgorithmTaintMismatch {
                run_id: run.run_id.clone(),
                step_index: step.step_index,
                declared: step.assorter_value,
                computed,
            });
        }
    }

    Ok(())
}

pub fn derive_batch_comparison_algorithm_run(
    package: &RcountPackage,
    run_id: &str,
    contest_id: &str,
    risk_limit_ppm: u32,
    sampled_batch_ids: &[String],
    decision: AuditAlgorithmDecision,
) -> Result<AuditAlgorithmRun, RcountCoreError> {
    if sampled_batch_ids.is_empty() {
        return Err(RcountCoreError::EmptyBatchComparisonAlgorithmSample {
            run_id: run_id.to_string(),
        });
    }
    if risk_limit_ppm == 0 || risk_limit_ppm >= 1_000_000 {
        return Err(RcountCoreError::InvalidAuditAlgorithmRiskLimit {
            run_id: run_id.to_string(),
            risk_limit_ppm,
        });
    }

    verify_batch_comparison_audits(package)?;

    let audits_by_batch = package
        .batch_comparison_audits
        .iter()
        .filter(|audit| audit.contest_id == contest_id)
        .map(|audit| (audit.batch_id.as_str(), audit))
        .collect::<BTreeMap<_, _>>();

    let first_audit = audits_by_batch
        .get(sampled_batch_ids[0].as_str())
        .ok_or_else(
            || RcountCoreError::MissingBatchComparisonAlgorithmEvidence {
                run_id: run_id.to_string(),
                step_index: 0,
                batch_id: sampled_batch_ids[0].clone(),
            },
        )?;
    let winner_selection_id = first_audit.winner_selection_id.clone();
    let loser_selection_id = first_audit.loser_selection_id.clone();
    let assertion_id = format!("assertion:{winner_selection_id}-over-{loser_selection_id}");

    let mut run_source_refs = BTreeSet::new();
    let mut sample_steps = Vec::with_capacity(sampled_batch_ids.len());
    for (step_index, batch_id) in sampled_batch_ids.iter().enumerate() {
        let audit = audits_by_batch.get(batch_id.as_str()).ok_or_else(|| {
            RcountCoreError::MissingBatchComparisonAlgorithmEvidence {
                run_id: run_id.to_string(),
                step_index: step_index as u32,
                batch_id: batch_id.clone(),
            }
        })?;
        if audit.winner_selection_id != winner_selection_id
            || audit.loser_selection_id != loser_selection_id
        {
            return Err(RcountCoreError::BatchComparisonAlgorithmAssertionMismatch {
                run_id: run_id.to_string(),
                audit_id: audit.audit_id.clone(),
            });
        }
        if audit.declared_reported_margin <= 0 {
            return Err(RcountCoreError::InvalidBatchComparisonAlgorithmMargin {
                run_id: run_id.to_string(),
                audit_id: audit.audit_id.clone(),
                reported_margin: audit.declared_reported_margin,
            });
        }

        let mut step_source_refs = vec![audit.audit_id.clone()];
        step_source_refs.extend(audit.source_refs.clone());
        run_source_refs.extend(step_source_refs.iter().cloned());
        sample_steps.push(AuditSampleStep {
            step_index: step_index as u32,
            round_index: None,
            assertion_id: assertion_id.clone(),
            sample_unit_id: batch_id.clone(),
            assorter_value: RationalValue {
                numerator: audit.declared_overstatement,
                denominator: audit.declared_reported_margin,
            },
            bet: None,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: step_source_refs,
        });
    }

    Ok(AuditAlgorithmRun {
        run_id: run_id.to_string(),
        contest_id: contest_id.to_string(),
        method_id: BATCH_COMPARISON_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::Batch,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(risk_limit_ppm),
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
            assertion_id,
            kind: AuditAssertionKind::ComparisonOverstatement,
            assorter_id: "batch-plurality-overstatement-taint-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some(winner_selection_id),
            loser_selection_id: Some(loser_selection_id),
        }],
        sample_steps,
        decision,
        source_refs: run_source_refs.into_iter().collect(),
    })
}

pub fn rla_contest_manifest_hash(
    package: &RcountPackage,
    contest_id: &str,
) -> Result<String, RcountCoreError> {
    rla_contest_manifest_hash_for_audit(package, contest_id, "<manifest-hash>")
}

pub(crate) fn rla_contest_manifest_hash_for_audit(
    package: &RcountPackage,
    contest_id: &str,
    audit_id: &str,
) -> Result<String, RcountCoreError> {
    let population = rla_population(package, contest_id);
    if population.is_empty() {
        return Err(RcountCoreError::MissingRlaPopulation {
            audit_id: audit_id.to_string(),
            contest_id: contest_id.to_string(),
        });
    }
    let value = serde_json::json!({
        "contest_id": contest_id,
        "cvr_ids": population,
    });
    canonical_hash(RLA_MANIFEST_HASH_PREFIX, &value)
}

pub fn replay_rla_sample(
    package: &RcountPackage,
    audit: &RiskLimitAudit,
) -> Result<Vec<RlaSampleDraw>, RcountCoreError> {
    let population = rla_population(package, &audit.contest_id);
    if population.is_empty() {
        return Err(RcountCoreError::MissingRlaPopulation {
            audit_id: audit.audit_id.clone(),
            contest_id: audit.contest_id.clone(),
        });
    }

    let mut draws = Vec::with_capacity(audit.sample_size as usize);
    for draw_index in 0..audit.sample_size {
        let mut h = Sha256::new();
        h.update(RLA_SAMPLE_PREFIX);
        h.update(audit.manifest_hash.as_bytes());
        h.update(b"\0");
        h.update(audit.public_seed.as_bytes());
        h.update(b"\0");
        h.update(audit.contest_id.as_bytes());
        h.update(b"\0");
        h.update(audit.risk_limit_ppm.to_le_bytes());
        h.update(draw_index.to_le_bytes());
        h.update(audit.sampling_algorithm_id.as_bytes());
        let digest = h.finalize();
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        let selected = u64::from_le_bytes(bytes) as usize % population.len();
        draws.push(RlaSampleDraw {
            draw_index,
            cvr_id: population[selected].clone(),
        });
    }
    Ok(draws)
}

pub fn verify_rla_margin_metadata(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut passes = Vec::new();
    for audit in &package.rla_audits {
        let Some(margin) = &audit.margin else {
            continue;
        };
        let summary = package
            .summaries
            .iter()
            .find(|summary| {
                summary.contest_id == audit.contest_id
                    && summary.batch_id.is_none()
                    && summary.status == CountStatus::Canvassed
                    && package.reporting_units.iter().any(|unit| {
                        unit.reporting_unit_id == summary.reporting_unit_id
                            && unit.kind == ReportingUnitKind::JurisdictionTotal
                    })
            })
            .ok_or_else(|| RcountCoreError::MissingJurisdictionTotal {
                contest_id: audit.contest_id.clone(),
                jurisdiction_reporting_unit_id: "<jurisdiction-total>".to_string(),
            })?;
        let totals: BTreeMap<&str, i64> = summary
            .totals
            .iter()
            .map(|total| (total.selection_id.as_str(), total.votes))
            .collect();
        let winner_votes = totals
            .get(margin.winner_selection_id.as_str())
            .copied()
            .ok_or_else(|| RcountCoreError::MissingRlaMarginSelection {
                audit_id: audit.audit_id.clone(),
                selection_id: margin.winner_selection_id.clone(),
            })?;
        let loser_votes = totals
            .get(margin.loser_selection_id.as_str())
            .copied()
            .ok_or_else(|| RcountCoreError::MissingRlaMarginSelection {
                audit_id: audit.audit_id.clone(),
                selection_id: margin.loser_selection_id.clone(),
            })?;
        if margin.reported_winner_votes != winner_votes {
            return Err(RcountCoreError::RlaWinnerVotesMismatch {
                audit_id: audit.audit_id.clone(),
                selection_id: margin.winner_selection_id.clone(),
                declared: margin.reported_winner_votes,
                summary: winner_votes,
            });
        }
        if margin.reported_loser_votes != loser_votes {
            return Err(RcountCoreError::RlaLoserVotesMismatch {
                audit_id: audit.audit_id.clone(),
                selection_id: margin.loser_selection_id.clone(),
                declared: margin.reported_loser_votes,
                summary: loser_votes,
            });
        }
        let computed_margin = winner_votes - loser_votes;
        if computed_margin <= 0 {
            return Err(RcountCoreError::InvalidRlaReportedMargin {
                audit_id: audit.audit_id.clone(),
                margin: computed_margin,
            });
        }
        if margin.reported_margin != computed_margin {
            return Err(RcountCoreError::RlaReportedMarginMismatch {
                audit_id: audit.audit_id.clone(),
                declared: margin.reported_margin,
                summary: computed_margin,
            });
        }
        if margin.diluted_margin_denominator != summary.counted_ballots {
            return Err(RcountCoreError::RlaDilutedMarginDenominatorMismatch {
                audit_id: audit.audit_id.clone(),
                declared: margin.diluted_margin_denominator,
                summary: summary.counted_ballots,
            });
        }
        passes.push(EquationPass {
            equation_id: "rla_margin_metadata".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.audit_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_rla_stopping_rules(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut passes = Vec::new();
    for audit in &package.rla_audits {
        if audit.observations.is_empty()
            && audit.stopping_rule_id.is_none()
            && audit.max_discrepancies.is_none()
            && audit.declared_status.is_none()
            && audit.declared_risk_ppm.is_none()
        {
            continue;
        }
        let stopping_rule_id = audit.stopping_rule_id.as_deref().ok_or_else(|| {
            RcountCoreError::MissingRlaStoppingRule {
                audit_id: audit.audit_id.clone(),
            }
        })?;
        if !matches!(
            stopping_rule_id,
            "zero-discrepancy-threshold-v1" | "comparison-margin-threshold-v1"
        ) || audit.max_discrepancies.is_none()
            || audit.declared_status.is_none()
        {
            return Err(RcountCoreError::MissingRlaStoppingRule {
                audit_id: audit.audit_id.clone(),
            });
        }
        if stopping_rule_id == "comparison-margin-threshold-v1"
            && (audit.margin.is_none() || audit.declared_risk_ppm.is_none())
        {
            return Err(RcountCoreError::MissingRlaRiskEstimate {
                audit_id: audit.audit_id.clone(),
            });
        }

        let cvr_by_id: BTreeMap<&str, &CvrContestRecord> = package
            .cvr
            .iter()
            .filter(|row| row.contest_id == audit.contest_id)
            .map(|row| (row.cvr_id.as_str(), row))
            .collect();
        let mut observations = BTreeMap::new();
        for observation in &audit.observations {
            if observations
                .insert(observation.draw_index, observation)
                .is_some()
            {
                return Err(RcountCoreError::DuplicateRlaObservation {
                    audit_id: audit.audit_id.clone(),
                    draw_index: observation.draw_index,
                });
            }
        }

        let mut computed_discrepancies = Vec::new();
        for draw in &audit.sample_draws {
            let observation = observations.get(&draw.draw_index).ok_or_else(|| {
                RcountCoreError::MissingRlaObservation {
                    audit_id: audit.audit_id.clone(),
                    draw_index: draw.draw_index,
                }
            })?;
            if observation.cvr_id != draw.cvr_id {
                let discrepancy = RlaDiscrepancy {
                    draw_index: draw.draw_index,
                    cvr_id: draw.cvr_id.clone(),
                    kind: RlaDiscrepancyKind::WrongCvrObserved,
                };
                computed_discrepancies.push(discrepancy);
                continue;
            }
            let cvr = cvr_by_id.get(draw.cvr_id.as_str()).ok_or_else(|| {
                RcountCoreError::MissingRlaPopulation {
                    audit_id: audit.audit_id.clone(),
                    contest_id: audit.contest_id.clone(),
                }
            })?;
            if let Some(kind) = classify_rla_discrepancy(observation, cvr) {
                computed_discrepancies.push(RlaDiscrepancy {
                    draw_index: draw.draw_index,
                    cvr_id: draw.cvr_id.clone(),
                    kind,
                });
            }
        }

        verify_declared_rla_discrepancies(audit, &computed_discrepancies)?;

        let computed_risk_ppm = if stopping_rule_id == "comparison-margin-threshold-v1" {
            let computed = comparison_margin_risk_ppm(audit);
            let declared = audit.declared_risk_ppm.unwrap();
            if declared != computed {
                return Err(RcountCoreError::RlaRiskEstimateMismatch {
                    audit_id: audit.audit_id.clone(),
                    declared_ppm: declared,
                    computed_ppm: computed,
                });
            }
            Some(computed)
        } else {
            None
        };

        let computed = if computed_discrepancies.len() as u32 <= audit.max_discrepancies.unwrap()
            && computed_risk_ppm.map_or(true, |risk| risk <= audit.risk_limit_ppm)
        {
            RlaStoppingStatus::Pass
        } else {
            RlaStoppingStatus::Escalate
        };
        let declared = audit.declared_status.unwrap();
        if declared != computed {
            return Err(RcountCoreError::RlaStoppingStatusMismatch {
                audit_id: audit.audit_id.clone(),
                declared,
                computed,
            });
        }
        passes.push(EquationPass {
            equation_id: "rla_stopping_rule".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.audit_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_rla_jurisdiction_adapters(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut passes = Vec::new();
    for audit in &package.rla_audits {
        let Some(jurisdiction_method_id) = audit.jurisdiction_method_id.as_deref() else {
            continue;
        };
        match jurisdiction_method_id {
            COLORADO_RLA_METHOD_ID => verify_colorado_rla_adapter(audit)?,
            CALIFORNIA_RLA_METHOD_ID => verify_california_rla_adapter(audit)?,
            other => {
                return Err(RcountCoreError::UnsupportedRlaJurisdictionMethod {
                    audit_id: audit.audit_id.clone(),
                    jurisdiction_method_id: other.to_string(),
                });
            }
        }
        passes.push(EquationPass {
            equation_id: "rla_jurisdiction_adapter".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.audit_id.clone(),
        });
    }
    Ok(passes)
}

pub(crate) fn verify_colorado_rla_adapter(audit: &RiskLimitAudit) -> Result<(), RcountCoreError> {
    if audit.public_seed.len() != 20 || !audit.public_seed.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(RcountCoreError::InvalidColoradoRlaSeed {
            audit_id: audit.audit_id.clone(),
            public_seed: audit.public_seed.clone(),
        });
    }
    if audit.sampling_algorithm_id != RLA_SAMPLING_ALGORITHM_ID
        || audit.margin.is_none()
        || audit.stopping_rule_id.as_deref() != Some("comparison-margin-threshold-v1")
        || audit.declared_risk_ppm.is_none()
        || audit.declared_status.is_none()
    {
        return Err(RcountCoreError::MissingColoradoRlaComparisonFields {
            audit_id: audit.audit_id.clone(),
        });
    }
    Ok(())
}

pub(crate) fn verify_california_rla_adapter(audit: &RiskLimitAudit) -> Result<(), RcountCoreError> {
    let Some(ballot_manifest_format_id) = audit.ballot_manifest_format_id.as_deref() else {
        return Err(RcountCoreError::MissingCaliforniaRlaPublicToolFields {
            audit_id: audit.audit_id.clone(),
        });
    };
    if ballot_manifest_format_id != CALIFORNIA_BALLOT_MANIFEST_FORMAT_ID {
        return Err(RcountCoreError::InvalidCaliforniaRlaManifestFormat {
            audit_id: audit.audit_id.clone(),
            ballot_manifest_format_id: ballot_manifest_format_id.to_string(),
        });
    }
    if audit.audit_software_id.as_deref().is_none_or(str::is_empty)
        || audit
            .audit_software_source_url
            .as_deref()
            .is_none_or(str::is_empty)
        || audit.margin.is_none()
        || audit.declared_status.is_none()
    {
        return Err(RcountCoreError::MissingCaliforniaRlaPublicToolFields {
            audit_id: audit.audit_id.clone(),
        });
    }
    let source_url = audit.audit_software_source_url.as_deref().unwrap();
    if !(source_url.starts_with("https://") || source_url.starts_with("http://")) {
        return Err(RcountCoreError::InvalidRlaSoftwareSourceUrl {
            audit_id: audit.audit_id.clone(),
            source_url: source_url.to_string(),
        });
    }
    Ok(())
}

pub fn verify_manual_audits(package: &RcountPackage) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for audit in &package.manual_audits {
        if !seen.insert(audit.audit_id.as_str()) {
            return Err(RcountCoreError::DuplicateManualAuditId {
                audit_id: audit.audit_id.clone(),
            });
        }
        let summary = package
            .summaries
            .iter()
            .find(|summary| {
                summary.contest_id == audit.contest_id
                    && summary.reporting_unit_id == audit.reporting_unit_id
                    && summary.status == CountStatus::Canvassed
                    && summary.batch_id.is_none()
            })
            .ok_or_else(|| RcountCoreError::MissingManualAuditSummary {
                audit_id: audit.audit_id.clone(),
                contest_id: audit.contest_id.clone(),
                reporting_unit_id: audit.reporting_unit_id.clone(),
            })?;
        let summary_totals: BTreeMap<&str, i64> = summary
            .totals
            .iter()
            .map(|total| (total.selection_id.as_str(), total.votes))
            .collect();
        let machine_totals: BTreeMap<&str, i64> = audit
            .machine_totals
            .iter()
            .map(|total| (total.selection_id.as_str(), total.votes))
            .collect();
        for (selection_id, summary_votes) in &summary_totals {
            let declared = machine_totals.get(selection_id).copied().ok_or_else(|| {
                RcountCoreError::ManualAuditMachineTotalMismatch {
                    audit_id: audit.audit_id.clone(),
                    selection_id: (*selection_id).to_string(),
                    declared: 0,
                    summary: *summary_votes,
                }
            })?;
            if declared != *summary_votes {
                return Err(RcountCoreError::ManualAuditMachineTotalMismatch {
                    audit_id: audit.audit_id.clone(),
                    selection_id: (*selection_id).to_string(),
                    declared,
                    summary: *summary_votes,
                });
            }
        }
        let hand_totals: BTreeMap<&str, i64> = audit
            .hand_totals
            .iter()
            .map(|total| (total.selection_id.as_str(), total.votes))
            .collect();
        let computed = if summary_totals.iter().all(|(selection_id, machine_votes)| {
            hand_totals.get(selection_id).is_some_and(|hand_votes| {
                (*hand_votes - *machine_votes).abs() <= audit.tolerance_votes
            })
        }) {
            ManualAuditStatus::Pass
        } else {
            ManualAuditStatus::Escalate
        };
        if audit.declared_status != computed {
            return Err(RcountCoreError::ManualAuditStatusMismatch {
                audit_id: audit.audit_id.clone(),
                declared: audit.declared_status,
                computed,
            });
        }
        passes.push(EquationPass {
            equation_id: "manual_audit_reconciliation".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.reporting_unit_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_batch_comparison_audits(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for audit in &package.batch_comparison_audits {
        if !seen.insert(audit.audit_id.as_str()) {
            return Err(RcountCoreError::DuplicateBatchComparisonAuditId {
                audit_id: audit.audit_id.clone(),
            });
        }
        let batch = package
            .batches
            .iter()
            .find(|batch| batch.batch_id == audit.batch_id)
            .ok_or_else(|| RcountCoreError::MissingBatchComparisonBatch {
                audit_id: audit.audit_id.clone(),
                batch_id: audit.batch_id.clone(),
            })?;
        if audit.declared_batch_ballots != batch.counted_ballots {
            return Err(RcountCoreError::BatchComparisonBatchSizeMismatch {
                audit_id: audit.audit_id.clone(),
                batch_id: audit.batch_id.clone(),
                declared: audit.declared_batch_ballots,
                manifest: batch.counted_ballots,
            });
        }
        let summary = package
            .summaries
            .iter()
            .find(|summary| {
                summary.contest_id == audit.contest_id
                    && summary.batch_id.as_deref() == Some(audit.batch_id.as_str())
                    && summary.status == CountStatus::Canvassed
            })
            .ok_or_else(|| RcountCoreError::MissingBatchComparisonSummary {
                audit_id: audit.audit_id.clone(),
                contest_id: audit.contest_id.clone(),
                batch_id: audit.batch_id.clone(),
            })?;
        let summary_totals = totals_by_selection(&summary.totals);
        let reported_totals = totals_by_selection(&audit.reported_totals);
        check_reported_batch_total(
            &audit.audit_id,
            &audit.winner_selection_id,
            &summary_totals,
            &reported_totals,
        )?;
        check_reported_batch_total(
            &audit.audit_id,
            &audit.loser_selection_id,
            &summary_totals,
            &reported_totals,
        )?;

        let hand_totals = totals_by_selection(&audit.hand_totals);
        let reported_winner = required_total(&audit.winner_selection_id, &reported_totals);
        let reported_loser = required_total(&audit.loser_selection_id, &reported_totals);
        let hand_winner =
            required_hand_total(&audit.audit_id, &audit.winner_selection_id, &hand_totals)?;
        let hand_loser =
            required_hand_total(&audit.audit_id, &audit.loser_selection_id, &hand_totals)?;
        let reported_margin = reported_winner - reported_loser;
        let hand_margin = hand_winner - hand_loser;
        let overstatement = reported_margin - hand_margin;
        if audit.declared_reported_margin != reported_margin {
            return Err(RcountCoreError::BatchComparisonReportedMarginMismatch {
                audit_id: audit.audit_id.clone(),
                declared: audit.declared_reported_margin,
                computed: reported_margin,
            });
        }
        if audit.declared_hand_margin != hand_margin {
            return Err(RcountCoreError::BatchComparisonHandMarginMismatch {
                audit_id: audit.audit_id.clone(),
                declared: audit.declared_hand_margin,
                computed: hand_margin,
            });
        }
        if audit.declared_overstatement != overstatement {
            return Err(RcountCoreError::BatchComparisonOverstatementMismatch {
                audit_id: audit.audit_id.clone(),
                declared: audit.declared_overstatement,
                computed: overstatement,
            });
        }
        passes.push(EquationPass {
            equation_id: "batch_comparison_overstatement".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.batch_id.clone(),
        });
    }
    Ok(passes)
}

pub(crate) fn check_reported_batch_total(
    audit_id: &str,
    selection_id: &str,
    summary_totals: &BTreeMap<&str, i64>,
    reported_totals: &BTreeMap<&str, i64>,
) -> Result<(), RcountCoreError> {
    let summary = required_total(selection_id, summary_totals);
    let declared = required_total(selection_id, reported_totals);
    if declared != summary {
        return Err(RcountCoreError::BatchComparisonReportedTotalMismatch {
            audit_id: audit_id.to_string(),
            selection_id: selection_id.to_string(),
            declared,
            summary,
        });
    }
    Ok(())
}

pub(crate) fn totals_by_selection(totals: &[SelectionTotal]) -> BTreeMap<&str, i64> {
    totals
        .iter()
        .map(|total| (total.selection_id.as_str(), total.votes))
        .collect()
}

pub(crate) fn required_total(selection_id: &str, totals: &BTreeMap<&str, i64>) -> i64 {
    totals.get(selection_id).copied().unwrap_or(0)
}

pub(crate) fn required_hand_total(
    audit_id: &str,
    selection_id: &str,
    totals: &BTreeMap<&str, i64>,
) -> Result<i64, RcountCoreError> {
    totals.get(selection_id).copied().ok_or_else(|| {
        RcountCoreError::MissingBatchComparisonHandTally {
            audit_id: audit_id.to_string(),
            selection_id: selection_id.to_string(),
        }
    })
}

pub(crate) fn verify_declared_rla_discrepancies(
    audit: &RiskLimitAudit,
    computed: &[RlaDiscrepancy],
) -> Result<(), RcountCoreError> {
    if audit.discrepancies.is_empty() && computed.is_empty() {
        return Ok(());
    }
    let mut declared = audit.discrepancies.clone();
    declared.sort_by_key(|item| (item.draw_index, item.cvr_id.clone(), item.kind));
    let mut computed = computed.to_vec();
    computed.sort_by_key(|item| (item.draw_index, item.cvr_id.clone(), item.kind));
    if declared.len() != computed.len() {
        return Err(RcountCoreError::RlaDiscrepancyCountMismatch {
            audit_id: audit.audit_id.clone(),
            declared: declared.len(),
            computed: computed.len(),
        });
    }
    for (declared, computed) in declared.iter().zip(computed.iter()) {
        if declared.draw_index != computed.draw_index
            || declared.cvr_id != computed.cvr_id
            || declared.kind != computed.kind
        {
            return Err(RcountCoreError::RlaDiscrepancyMismatch {
                audit_id: audit.audit_id.clone(),
                draw_index: computed.draw_index,
                declared: declared.kind,
                computed: computed.kind,
            });
        }
    }
    Ok(())
}

pub(crate) fn classify_rla_discrepancy(
    observation: &RlaSampleObservation,
    cvr: &CvrContestRecord,
) -> Option<RlaDiscrepancyKind> {
    let mut observed = observation.observed_selection_ids.clone();
    observed.sort();
    let mut expected = cvr.selection_ids.clone();
    expected.sort();
    let selection_mismatch = observed != expected;
    let residual_mismatch = observation.undervote != cvr.undervote
        || observation.overvote != cvr.overvote
        || observation.blank_contest != cvr.blank_contest;
    match (selection_mismatch, residual_mismatch) {
        (true, true) => Some(RlaDiscrepancyKind::SelectionAndResidualMismatch),
        (true, false) => Some(RlaDiscrepancyKind::SelectionMismatch),
        (false, true) => Some(RlaDiscrepancyKind::ResidualMismatch),
        (false, false) => None,
    }
}

pub(crate) fn comparison_margin_risk_ppm(audit: &RiskLimitAudit) -> u32 {
    let margin = audit
        .margin
        .as_ref()
        .expect("comparison margin verifier requires margin metadata");
    let sample_margin_product =
        (audit.sample_size as u128).saturating_mul(margin.reported_margin.max(1) as u128);
    let denominator = sample_margin_product.max(1);
    let base = (1_000_000u128 + denominator - 1) / denominator;
    let discrepancy_penalty = (audit.discrepancies.len() as u128).saturating_mul(250_000);
    base.saturating_add(discrepancy_penalty).min(1_000_000) as u32
}

pub(crate) fn rla_population(package: &RcountPackage, contest_id: &str) -> Vec<String> {
    let mut population: Vec<String> = package
        .cvr
        .iter()
        .filter(|row| row.contest_id == contest_id)
        .map(|row| row.cvr_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    population.sort();
    population
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CvrAggregateKey {
    pub(crate) contest_id: String,
    pub(crate) reporting_unit_id: String,
    pub(crate) batch_id: Option<String>,
    pub(crate) status: CountStatus,
}

impl From<&CvrContestRecord> for CvrAggregateKey {
    fn from(row: &CvrContestRecord) -> Self {
        Self {
            contest_id: row.contest_id.clone(),
            reporting_unit_id: row.reporting_unit_id.clone(),
            batch_id: row.batch_id.clone(),
            status: row.status,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct CvrAggregate {
    pub(crate) selection_votes: BTreeMap<String, i64>,
    pub(crate) undervotes: i64,
    pub(crate) overvotes: i64,
    pub(crate) blank_contests: i64,
    pub(crate) counted_ballots: i64,
}
