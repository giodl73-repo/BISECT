use crate::*;

pub fn canonical_hash(prefix: &[u8], value: &Value) -> Result<String, RcountCoreError> {
    let canonical = canonicalize_value(value);
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| RcountCoreError::CanonicalJson(err.to_string()))?;
    let mut h = Sha256::new();
    h.update(prefix);
    h.update(bytes);
    Ok(format!("sha256:{:x}", h.finalize()))
}

pub fn record_hash<T: Serialize>(record: &T) -> Result<String, RcountCoreError> {
    let value = serde_json::to_value(record)
        .map_err(|err| RcountCoreError::CanonicalJson(err.to_string()))?;
    canonical_hash(RECORD_HASH_PREFIX, &value)
}

pub fn package_content_hash(package: &RcountPackage) -> Result<String, RcountCoreError> {
    let value = serde_json::to_value(package)
        .map_err(|err| RcountCoreError::CanonicalJson(err.to_string()))?;
    canonical_hash(PACKAGE_HASH_PREFIX, &value)
}

pub fn verify_package(package: &RcountPackage) -> Result<VerificationReport, RcountCoreError> {
    let contests: BTreeMap<&str, &Contest> = package
        .contests
        .iter()
        .map(|contest| (contest.contest_id.as_str(), contest))
        .collect();
    for contest in package.contests.iter() {
        validate_contest(contest)?;
    }

    let mut report = VerificationReport::default();
    for summary in package.summaries.iter() {
        let contest = contests.get(summary.contest_id.as_str()).ok_or_else(|| {
            RcountCoreError::UnknownSelection {
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
                selection_id: "<contest-missing>".to_string(),
            }
        })?;
        verify_contest_selection_sum(contest, summary)?;
        report.passed.push(EquationPass {
            equation_id: "contest_selection_sum".to_string(),
            contest_id: summary.contest_id.clone(),
            reporting_unit_id: summary.reporting_unit_id.clone(),
        });
    }
    report.passed.extend(verify_status_events(package)?);
    report.passed.extend(verify_batch_summary_totals(package)?);
    report.passed.extend(verify_lineage_conservation(package)?);
    report.passed.extend(verify_rhist_references(package)?);
    report.passed.extend(verify_rctx_references(package)?);
    report.passed.extend(verify_proof_privacy(package)?);
    report
        .passed
        .extend(verify_cvr_summary_reconciliation(package)?);
    report
        .passed
        .extend(verify_batch_comparison_audits(package)?);
    report.passed.extend(verify_audit_algorithm_runs(package)?);
    report.passed.extend(verify_rla_sampler_replay(package)?);
    report.passed.extend(verify_rla_margin_metadata(package)?);
    report.passed.extend(verify_rla_stopping_rules(package)?);
    report
        .passed
        .extend(verify_rla_jurisdiction_adapters(package)?);
    report.passed.extend(verify_manual_audits(package)?);
    Ok(report)
}

pub fn verify_package_parallel(
    package: &RcountPackage,
) -> Result<VerificationReport, RcountCoreError> {
    let contests: BTreeMap<&str, &Contest> = package
        .contests
        .iter()
        .map(|contest| (contest.contest_id.as_str(), contest))
        .collect();
    for contest in package.contests.iter() {
        validate_contest(contest)?;
    }

    let summary_passes = package
        .summaries
        .par_iter()
        .map(|summary| {
            let contest = contests.get(summary.contest_id.as_str()).ok_or_else(|| {
                RcountCoreError::UnknownSelection {
                    contest_id: summary.contest_id.clone(),
                    reporting_unit_id: summary.reporting_unit_id.clone(),
                    selection_id: "<contest-missing>".to_string(),
                }
            })?;
            verify_contest_selection_sum(contest, summary)?;
            Ok(EquationPass {
                equation_id: "contest_selection_sum".to_string(),
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
            })
        })
        .collect::<Result<Vec<_>, RcountCoreError>>()?;

    let mut report = VerificationReport::default();
    report.passed.extend(summary_passes);
    report.passed.extend(verify_status_events(package)?);
    report.passed.extend(verify_batch_summary_totals(package)?);
    report.passed.extend(verify_lineage_conservation(package)?);
    report.passed.extend(verify_rhist_references(package)?);
    report.passed.extend(verify_rctx_references(package)?);
    report.passed.extend(verify_proof_privacy(package)?);
    report
        .passed
        .extend(verify_cvr_summary_reconciliation(package)?);
    report
        .passed
        .extend(verify_batch_comparison_audits(package)?);
    report.passed.extend(verify_audit_algorithm_runs(package)?);
    report.passed.extend(verify_rla_sampler_replay(package)?);
    report.passed.extend(verify_rla_margin_metadata(package)?);
    report.passed.extend(verify_rla_stopping_rules(package)?);
    report
        .passed
        .extend(verify_rla_jurisdiction_adapters(package)?);
    report.passed.extend(verify_manual_audits(package)?);
    Ok(report)
}

pub fn verify_contest_selection_sum(
    contest: &Contest,
    summary: &Summary,
) -> Result<(), RcountCoreError> {
    ensure_non_negative(summary.undervotes)?;
    ensure_non_negative(summary.overvotes)?;
    ensure_non_negative(summary.blank_contests)?;
    ensure_non_negative(summary.counted_ballots)?;

    let valid_selection_ids: BTreeSet<&str> = contest
        .selections
        .iter()
        .map(|selection| selection.selection_id.as_str())
        .collect();
    let mut seen = BTreeSet::new();
    let mut selection_votes = 0i64;
    for total in summary.totals.iter() {
        ensure_non_negative(total.votes)?;
        if !seen.insert(total.selection_id.as_str()) {
            return Err(RcountCoreError::DuplicateSummarySelection {
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
                selection_id: total.selection_id.clone(),
            });
        }
        if !valid_selection_ids.contains(total.selection_id.as_str()) {
            return Err(RcountCoreError::UnknownSelection {
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
                selection_id: total.selection_id.clone(),
            });
        }
        selection_votes += total.votes;
    }

    let computed =
        selection_votes + summary.undervotes + summary.overvotes + summary.blank_contests;
    if computed != summary.counted_ballots {
        return Err(RcountCoreError::ContestSelectionSumMismatch {
            contest_id: summary.contest_id.clone(),
            reporting_unit_id: summary.reporting_unit_id.clone(),
            declared_ballots: summary.counted_ballots,
            computed_ballots: computed,
        });
    }
    Ok(())
}

pub fn verify_jurisdiction_total(
    contest_id: &str,
    jurisdiction_reporting_unit_id: &str,
    summaries: &[Summary],
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let totals: Vec<&Summary> = summaries
        .iter()
        .filter(|summary| {
            summary.contest_id == contest_id
                && summary.reporting_unit_id == jurisdiction_reporting_unit_id
        })
        .collect();
    if totals.is_empty() {
        return Err(RcountCoreError::MissingJurisdictionTotal {
            contest_id: contest_id.to_string(),
            jurisdiction_reporting_unit_id: jurisdiction_reporting_unit_id.to_string(),
        });
    }

    let mut passes = Vec::new();
    for total in totals {
        verify_jurisdiction_total_for_status(
            contest_id,
            jurisdiction_reporting_unit_id,
            total,
            summaries,
        )?;
        passes.push(EquationPass {
            equation_id: "jurisdiction_contest_total".to_string(),
            contest_id: contest_id.to_string(),
            reporting_unit_id: jurisdiction_reporting_unit_id.to_string(),
        });
    }
    Ok(passes)
}

pub(crate) fn verify_jurisdiction_total_for_status(
    contest_id: &str,
    jurisdiction_reporting_unit_id: &str,
    total: &Summary,
    summaries: &[Summary],
) -> Result<(), RcountCoreError> {
    let mut selection_sums: BTreeMap<&str, i64> = BTreeMap::new();
    let mut undervotes = 0i64;
    let mut overvotes = 0i64;
    let mut blank_contests = 0i64;
    let mut counted_ballots = 0i64;

    for summary in summaries.iter().filter(|summary| {
        summary.contest_id == contest_id
            && summary.reporting_unit_id != jurisdiction_reporting_unit_id
            && summary.status == total.status
    }) {
        for selection in summary.totals.iter() {
            *selection_sums
                .entry(selection.selection_id.as_str())
                .or_default() += selection.votes;
        }
        undervotes += summary.undervotes;
        overvotes += summary.overvotes;
        blank_contests += summary.blank_contests;
        counted_ballots += summary.counted_ballots;
    }

    for total_selection in total.totals.iter() {
        let computed = selection_sums
            .get(total_selection.selection_id.as_str())
            .copied()
            .unwrap_or_default();
        if total_selection.votes != computed {
            return Err(RcountCoreError::JurisdictionSelectionMismatch {
                contest_id: contest_id.to_string(),
                selection_id: total_selection.selection_id.clone(),
                declared_votes: total_selection.votes,
                computed_votes: computed,
            });
        }
    }
    check_residual(contest_id, "undervotes", total.undervotes, undervotes)?;
    check_residual(contest_id, "overvotes", total.overvotes, overvotes)?;
    check_residual(
        contest_id,
        "blank_contests",
        total.blank_contests,
        blank_contests,
    )?;
    check_residual(
        contest_id,
        "counted_ballots",
        total.counted_ballots,
        counted_ballots,
    )?;

    Ok(())
}

pub fn verify_status_events(package: &RcountPackage) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for event in package.status_events.iter() {
        if !seen.insert(event.event_id.as_str()) {
            return Err(RcountCoreError::DuplicateStatusEventId {
                event_id: event.event_id.clone(),
            });
        }
        if event.status_before == event.status_after
            && event.event_type != StatusEventType::InitialUnofficialReport
        {
            return Err(RcountCoreError::NoStatusTransition {
                event_id: event.event_id.clone(),
            });
        }
        if event.authority.trim().is_empty() || event.explanation.trim().is_empty() {
            return Err(RcountCoreError::IncompleteStatusEvent {
                event_id: event.event_id.clone(),
            });
        }
        passes.push(EquationPass {
            equation_id: "status_event_declared".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: event.event_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_canvass_correction_event(
    package: &RcountPackage,
) -> Result<EquationPass, RcountCoreError> {
    let has_event = package.status_events.iter().any(|event| {
        event.event_type == StatusEventType::Correction
            && event.status_before == CountStatus::Unofficial
            && event.status_after == CountStatus::Canvassed
    });
    if !has_event {
        return Err(RcountCoreError::MissingCanvassCorrectionEvent);
    }
    for status in [CountStatus::Unofficial, CountStatus::Canvassed] {
        if !package
            .summaries
            .iter()
            .any(|summary| summary.status == status)
        {
            return Err(RcountCoreError::MissingStatusSummaries { status });
        }
    }
    Ok(EquationPass {
        equation_id: "canvass_correction_event".to_string(),
        contest_id: "*".to_string(),
        reporting_unit_id: "*".to_string(),
    })
}

pub fn verify_batch_summary_totals(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut batches: BTreeMap<&str, &BatchManifest> = BTreeMap::new();
    let mut passes = Vec::new();
    for batch in package.batches.iter() {
        ensure_non_negative(batch.accepted_ballots)?;
        ensure_non_negative(batch.counted_ballots)?;
        ensure_non_negative(batch.rejected_ballots)?;
        if batches.insert(batch.batch_id.as_str(), batch).is_some() {
            return Err(RcountCoreError::DuplicateBatchId {
                batch_id: batch.batch_id.clone(),
            });
        }
        let computed = batch.counted_ballots + batch.rejected_ballots;
        if batch.accepted_ballots != computed {
            return Err(RcountCoreError::AcceptedBallotsMismatch {
                batch_id: batch.batch_id.clone(),
                declared_ballots: batch.accepted_ballots,
                computed_ballots: computed,
            });
        }
        passes.push(EquationPass {
            equation_id: "accepted_ballots".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: batch.batch_id.clone(),
        });
    }

    for summary in package
        .summaries
        .iter()
        .filter(|summary| summary.batch_id.is_some())
    {
        let batch_id = summary
            .batch_id
            .as_ref()
            .expect("filtered to batch summaries");
        let batch =
            batches
                .get(batch_id.as_str())
                .ok_or_else(|| RcountCoreError::MissingBatch {
                    contest_id: summary.contest_id.clone(),
                    reporting_unit_id: summary.reporting_unit_id.clone(),
                    batch_id: batch_id.clone(),
                })?;
        if batch.counted_ballots != summary.counted_ballots {
            return Err(RcountCoreError::BatchSummaryTotalMismatch {
                batch_id: batch_id.clone(),
                declared_ballots: batch.counted_ballots,
                summary_ballots: summary.counted_ballots,
            });
        }
        passes.push(EquationPass {
            equation_id: "batch_summary_total".to_string(),
            contest_id: summary.contest_id.clone(),
            reporting_unit_id: batch_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_lineage_conservation(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let units: BTreeSet<&str> = package
        .reporting_units
        .iter()
        .map(|unit| unit.reporting_unit_id.as_str())
        .collect();
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();

    for event in package.lineage.iter() {
        if !seen.insert(event.lineage_id.as_str()) {
            return Err(RcountCoreError::DuplicateLineageId {
                lineage_id: event.lineage_id.clone(),
            });
        }
        for prior in event.prior_reporting_unit_ids.iter() {
            if !units.contains(prior.as_str()) {
                return Err(RcountCoreError::MissingPriorLineageUnit {
                    lineage_id: event.lineage_id.clone(),
                    reporting_unit_id: prior.clone(),
                });
            }
        }
        for current in event.current_reporting_unit_ids.iter() {
            if !units.contains(current.as_str()) {
                return Err(RcountCoreError::MissingCurrentLineageUnit {
                    lineage_id: event.lineage_id.clone(),
                    reporting_unit_id: current.clone(),
                });
            }
        }
        match event.kind {
            LineageKind::Unchanged => {
                if event.prior_reporting_unit_ids.len() != 1
                    || event.current_reporting_unit_ids.len() != 1
                {
                    return Err(RcountCoreError::InvalidSplitLineage {
                        lineage_id: event.lineage_id.clone(),
                    });
                }
            }
            LineageKind::Split => {
                if event.prior_reporting_unit_ids.len() != 1
                    || event.current_reporting_unit_ids.len() < 2
                {
                    return Err(RcountCoreError::InvalidSplitLineage {
                        lineage_id: event.lineage_id.clone(),
                    });
                }
            }
            LineageKind::Merge => {
                if event.prior_reporting_unit_ids.len() < 2
                    || event.current_reporting_unit_ids.len() != 1
                {
                    return Err(RcountCoreError::InvalidMergeLineage {
                        lineage_id: event.lineage_id.clone(),
                    });
                }
            }
        }
        passes.push(EquationPass {
            equation_id: "lineage_conservation".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: event.lineage_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_rhist_references(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for reference in &package.rhist_refs {
        if !seen.insert(reference.reference_id.as_str()) {
            return Err(RcountCoreError::DuplicateRhistReference {
                reference_id: reference.reference_id.clone(),
            });
        }
        if !is_sha256_hash(&reference.package_hash) {
            return Err(RcountCoreError::InvalidRhistPackageHash {
                reference_id: reference.reference_id.clone(),
                package_hash: reference.package_hash.clone(),
            });
        }
        if reference.cycle_ids.is_empty() {
            return Err(RcountCoreError::EmptyRhistCycleRefs {
                reference_id: reference.reference_id.clone(),
            });
        }
        if !matches!(
            reference.role.as_str(),
            "unit-lineage" | "aggregation-crosswalk" | "context-lineage"
        ) {
            return Err(RcountCoreError::UnsupportedRhistReferenceRole {
                reference_id: reference.reference_id.clone(),
                role: reference.role.clone(),
            });
        }
        passes.push(EquationPass {
            equation_id: "rhist_reference_declared".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: reference.reference_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_rctx_references(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for reference in &package.rctx_refs {
        if !seen.insert(reference.reference_id.as_str()) {
            return Err(RcountCoreError::DuplicateRctxReference {
                reference_id: reference.reference_id.clone(),
            });
        }
        if !is_sha256_hash(&reference.context_hash) {
            return Err(RcountCoreError::InvalidRctxContextHash {
                reference_id: reference.reference_id.clone(),
                context_hash: reference.context_hash.clone(),
            });
        }
        if let Some(crosswalk_hash) = &reference.crosswalk_hash {
            if !is_sha256_hash(crosswalk_hash) {
                return Err(RcountCoreError::InvalidRctxCrosswalkHash {
                    reference_id: reference.reference_id.clone(),
                    crosswalk_hash: crosswalk_hash.clone(),
                });
            }
        }
        if !matches!(
            reference.role.as_str(),
            "unit-context" | "aggregation-crosswalk" | "plan-context"
        ) {
            return Err(RcountCoreError::UnsupportedRctxReferenceRole {
                reference_id: reference.reference_id.clone(),
                role: reference.role.clone(),
            });
        }
        passes.push(EquationPass {
            equation_id: "rctx_reference_declared".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: reference.reference_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_proof_privacy(package: &RcountPackage) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for proof in package.inclusion_proofs.iter() {
        if !seen.insert(proof.proof_id.as_str()) {
            return Err(RcountCoreError::DuplicateProofId {
                proof_id: proof.proof_id.clone(),
            });
        }
        if !is_sha256_hash(&proof.token_hash) {
            return Err(RcountCoreError::InvalidProofTokenHash {
                proof_id: proof.proof_id.clone(),
                token_hash: proof.token_hash.clone(),
            });
        }
        if !proof.candidate_selections.is_empty() {
            return Err(RcountCoreError::ChoiceBearingProof {
                proof_id: proof.proof_id.clone(),
            });
        }
        if proof.voter_id.is_some() && proof.ballot_style.is_some() && proof.issued_at.is_some() {
            return Err(RcountCoreError::LinkableVoterProof {
                proof_id: proof.proof_id.clone(),
            });
        }
        passes.push(EquationPass {
            equation_id: "proof_privacy_gate".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: proof.proof_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_cvr_summary_reconciliation(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    if package.cvr.is_empty() {
        return Ok(Vec::new());
    }

    let contests: BTreeMap<&str, &Contest> = package
        .contests
        .iter()
        .map(|contest| (contest.contest_id.as_str(), contest))
        .collect();
    let mut seen = BTreeSet::new();
    let mut aggregates: BTreeMap<CvrAggregateKey, CvrAggregate> = BTreeMap::new();

    for row in &package.cvr {
        if !seen.insert((row.cvr_id.as_str(), row.contest_id.as_str())) {
            return Err(RcountCoreError::DuplicateCvrContest {
                cvr_id: row.cvr_id.clone(),
                contest_id: row.contest_id.clone(),
            });
        }
        let contest = contests.get(row.contest_id.as_str()).ok_or_else(|| {
            RcountCoreError::UnknownCvrSelection {
                cvr_id: row.cvr_id.clone(),
                contest_id: row.contest_id.clone(),
                selection_id: "<contest-missing>".to_string(),
            }
        })?;
        let valid_selection_ids: BTreeSet<&str> = contest
            .selections
            .iter()
            .map(|selection| selection.selection_id.as_str())
            .collect();
        let residual_count = row.undervote as u8 + row.overvote as u8 + row.blank_contest as u8;
        if residual_count > 1
            || (residual_count == 1 && !row.selection_ids.is_empty())
            || row.selection_ids.len() > contest.vote_for as usize
        {
            return Err(RcountCoreError::InvalidCvrContestCardinality {
                cvr_id: row.cvr_id.clone(),
                contest_id: row.contest_id.clone(),
            });
        }
        for selection_id in &row.selection_ids {
            if !valid_selection_ids.contains(selection_id.as_str()) {
                return Err(RcountCoreError::UnknownCvrSelection {
                    cvr_id: row.cvr_id.clone(),
                    contest_id: row.contest_id.clone(),
                    selection_id: selection_id.clone(),
                });
            }
        }

        let aggregate = aggregates.entry(CvrAggregateKey::from(row)).or_default();
        aggregate.counted_ballots += 1;
        for selection_id in &row.selection_ids {
            *aggregate
                .selection_votes
                .entry(selection_id.clone())
                .or_default() += 1;
        }
        aggregate.undervotes += row.undervote as i64;
        aggregate.overvotes += row.overvote as i64;
        aggregate.blank_contests += row.blank_contest as i64;
    }

    let mut passes = Vec::new();
    for (key, aggregate) in aggregates {
        let summary = package
            .summaries
            .iter()
            .find(|summary| {
                summary.contest_id == key.contest_id
                    && summary.reporting_unit_id == key.reporting_unit_id
                    && summary.batch_id == key.batch_id
                    && summary.status == key.status
            })
            .ok_or_else(|| RcountCoreError::MissingCvrSummary {
                contest_id: key.contest_id.clone(),
                reporting_unit_id: key.reporting_unit_id.clone(),
            })?;

        for total in &summary.totals {
            let cvr = aggregate
                .selection_votes
                .get(&total.selection_id)
                .copied()
                .unwrap_or_default();
            check_cvr_field(
                &key.contest_id,
                &key.reporting_unit_id,
                &format!("selection:{}", total.selection_id),
                total.votes,
                cvr,
            )?;
        }
        check_cvr_field(
            &key.contest_id,
            &key.reporting_unit_id,
            "undervotes",
            summary.undervotes,
            aggregate.undervotes,
        )?;
        check_cvr_field(
            &key.contest_id,
            &key.reporting_unit_id,
            "overvotes",
            summary.overvotes,
            aggregate.overvotes,
        )?;
        check_cvr_field(
            &key.contest_id,
            &key.reporting_unit_id,
            "blank_contests",
            summary.blank_contests,
            aggregate.blank_contests,
        )?;
        check_cvr_field(
            &key.contest_id,
            &key.reporting_unit_id,
            "counted_ballots",
            summary.counted_ballots,
            aggregate.counted_ballots,
        )?;
        passes.push(EquationPass {
            equation_id: "cvr_summary_total".to_string(),
            contest_id: key.contest_id,
            reporting_unit_id: key.reporting_unit_id,
        });
    }
    Ok(passes)
}
