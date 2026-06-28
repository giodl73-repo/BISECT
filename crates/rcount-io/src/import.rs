use crate::*;

#[derive(Debug, Deserialize)]
pub(crate) struct StatementCsvRow {
    contest_id: String,
    contest_title: String,
    vote_for: String,
    selection_id: String,
    selection_label: String,
    selection_kind: String,
    reporting_unit_id: String,
    reporting_unit_kind: String,
    parent_jurisdiction: String,
    status: String,
    votes: String,
    undervotes: String,
    overvotes: String,
    blank_contests: String,
    counted_ballots: String,
}

#[derive(Debug, Clone)]
pub(crate) struct SummaryAccumulator {
    contest_id: String,
    reporting_unit_id: String,
    status: CountStatus,
    totals: Vec<SelectionTotal>,
    seen_selection_ids: BTreeSet<String>,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
    counted_ballots: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RhodeIslandManifestRow {
    #[serde(rename = "Batch Name")]
    batch_name: String,
    #[serde(rename = "Number of Ballots")]
    number_of_ballots: String,
    #[serde(rename = "Container")]
    container: String,
    #[serde(rename = "Tabulator")]
    tabulator: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RhodeIslandRetrievalRow {
    #[serde(rename = "Container")]
    pub(crate) container: String,
    #[serde(rename = "Tabulator")]
    pub(crate) tabulator: String,
    #[serde(rename = "Batch Name")]
    pub(crate) batch_name: String,
    #[serde(rename = "Ballot Number")]
    pub(crate) ballot_number: String,
    #[serde(rename = "Ticket Numbers")]
    pub(crate) ticket_numbers: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RhodeIslandRlaSourceSummary {
    pub adapter_id: String,
    pub contest_id: String,
    pub audit_method: String,
    pub risk_limit_ppm: u32,
    pub public_seed: String,
    pub declared_sample_size: u32,
    pub sampled_ballot_rows: usize,
    pub retrieval_rows: usize,
    pub claim_boundary: Vec<String>,
}

/// Imports a deliberately small statement-of-votes CSV into the neutral RCOUNT
/// model. This adapter is the V.9 fixture surface: one row per
/// contest/reporting-unit/selection total, plus repeated residual columns.
pub fn import_statement_csv(path: &Path) -> Result<RcountPackage, RcountIoError> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut contests: BTreeMap<String, Contest> = BTreeMap::new();
    let mut reporting_units: BTreeMap<String, ReportingUnit> = BTreeMap::new();
    let mut summaries: BTreeMap<(String, String, CountStatus), SummaryAccumulator> =
        BTreeMap::new();

    for (index, row) in reader.deserialize::<StatementCsvRow>().enumerate() {
        let row_number = index + 2;
        let row = row?;
        let contest_id = required(row_number, "contest_id", row.contest_id)?;
        let contest_title = required(row_number, "contest_title", row.contest_title)?;
        let vote_for = parse_u32(row_number, "vote_for", row.vote_for)?;
        let selection_id = required(row_number, "selection_id", row.selection_id)?;
        let selection_label = required(row_number, "selection_label", row.selection_label)?;
        let selection_kind = parse_selection_kind(row_number, row.selection_kind)?;
        let reporting_unit_id = required(row_number, "reporting_unit_id", row.reporting_unit_id)?;
        let reporting_unit_kind = parse_reporting_unit_kind(row_number, row.reporting_unit_kind)?;
        let parent_jurisdiction =
            required(row_number, "parent_jurisdiction", row.parent_jurisdiction)?;
        let status = parse_count_status(row_number, row.status)?;
        let votes = parse_i64(row_number, "votes", row.votes)?;
        let undervotes = parse_i64(row_number, "undervotes", row.undervotes)?;
        let overvotes = parse_i64(row_number, "overvotes", row.overvotes)?;
        let blank_contests = parse_i64(row_number, "blank_contests", row.blank_contests)?;
        let counted_ballots = parse_i64(row_number, "counted_ballots", row.counted_ballots)?;

        let contest = contests.entry(contest_id.clone()).or_insert(Contest {
            contest_id: contest_id.clone(),
            title: contest_title.clone(),
            vote_for,
            selections: Vec::new(),
        });
        require_same(
            row_number,
            &contest_id,
            "contest_title",
            &contest.title,
            &contest_title,
        )?;
        require_same(
            row_number,
            &contest_id,
            "vote_for",
            &contest.vote_for.to_string(),
            &vote_for.to_string(),
        )?;
        if let Some(selection) = contest
            .selections
            .iter()
            .find(|selection| selection.selection_id == selection_id)
        {
            require_same(
                row_number,
                &selection_id,
                "selection_label",
                &selection.label,
                &selection_label,
            )?;
            if selection.kind != selection_kind {
                return Err(RcountIoError::ConflictingStatementCsvField {
                    row: row_number,
                    id: selection_id.clone(),
                    field: "selection_kind".to_string(),
                    prior: format!("{:?}", selection.kind),
                    value: format!("{selection_kind:?}"),
                });
            }
        } else {
            contest.selections.push(Selection {
                selection_id: selection_id.clone(),
                kind: selection_kind,
                label: selection_label,
            });
        }

        let unit = reporting_units
            .entry(reporting_unit_id.clone())
            .or_insert(ReportingUnit {
                reporting_unit_id: reporting_unit_id.clone(),
                kind: reporting_unit_kind.clone(),
                parent_jurisdiction: parent_jurisdiction.clone(),
                source_ids: vec!["source:statement-csv".to_string()],
                valid_from: None,
                valid_to: None,
            });
        if unit.kind != reporting_unit_kind {
            return Err(RcountIoError::ConflictingStatementCsvField {
                row: row_number,
                id: reporting_unit_id,
                field: "reporting_unit_kind".to_string(),
                prior: format!("{:?}", unit.kind),
                value: format!("{reporting_unit_kind:?}"),
            });
        }
        require_same(
            row_number,
            &unit.reporting_unit_id,
            "parent_jurisdiction",
            &unit.parent_jurisdiction,
            &parent_jurisdiction,
        )?;

        let key = (contest_id.clone(), reporting_unit_id.clone(), status);
        let summary = summaries.entry(key).or_insert(SummaryAccumulator {
            contest_id,
            reporting_unit_id,
            status,
            totals: Vec::new(),
            seen_selection_ids: BTreeSet::new(),
            undervotes,
            overvotes,
            blank_contests,
            counted_ballots,
        });
        require_same(
            row_number,
            &summary.reporting_unit_id,
            "undervotes",
            &summary.undervotes.to_string(),
            &undervotes.to_string(),
        )?;
        require_same(
            row_number,
            &summary.reporting_unit_id,
            "overvotes",
            &summary.overvotes.to_string(),
            &overvotes.to_string(),
        )?;
        require_same(
            row_number,
            &summary.reporting_unit_id,
            "blank_contests",
            &summary.blank_contests.to_string(),
            &blank_contests.to_string(),
        )?;
        require_same(
            row_number,
            &summary.reporting_unit_id,
            "counted_ballots",
            &summary.counted_ballots.to_string(),
            &counted_ballots.to_string(),
        )?;
        if summary.seen_selection_ids.insert(selection_id.clone()) {
            summary.totals.push(SelectionTotal {
                selection_id,
                votes,
            });
        } else {
            return Err(RcountIoError::ConflictingStatementCsvField {
                row: row_number,
                id: summary.reporting_unit_id.clone(),
                field: "selection_id".to_string(),
                prior: "already present".to_string(),
                value: selection_id,
            });
        }
    }

    Ok(RcountPackage {
        rcount_version: RCOUNT_VERSION.to_string(),
        contests: contests.into_values().collect(),
        reporting_units: reporting_units.into_values().collect(),
        batches: Vec::new(),
        lineage: Vec::new(),
        rhist_refs: Vec::new(),
        rctx_refs: Vec::new(),
        inclusion_proofs: Vec::new(),
        cvr: Vec::new(),
        audit_algorithm_runs: Vec::new(),
        rla_audits: Vec::new(),
        manual_audits: Vec::new(),
        batch_comparison_audits: Vec::new(),
        summaries: summaries
            .into_values()
            .map(|summary| Summary {
                contest_id: summary.contest_id,
                reporting_unit_id: summary.reporting_unit_id,
                batch_id: None,
                status: summary.status,
                totals: summary.totals,
                undervotes: summary.undervotes,
                overvotes: summary.overvotes,
                blank_contests: summary.blank_contests,
                counted_ballots: summary.counted_ballots,
            })
            .collect(),
        status_events: Vec::<StatusEvent>::new(),
    })
}

pub fn write_statement_csv_package_dir(
    dir: &Path,
    csv_path: &Path,
    manifest: &RcountManifest,
    package: &RcountPackage,
) -> Result<(), RcountIoError> {
    write_package_dir(dir, manifest, package)?;
    let source_path = PathBuf::from("sources").join("statement-of-votes.csv");
    let bytes = fs::read(csv_path)?;
    fs::write(dir.join(&source_path), &bytes)?;
    let synthetic = dir.join("sources").join("synthetic-summary-export.json");
    if synthetic.exists() {
        fs::remove_file(synthetic)?;
    }
    write_json_pretty(
        &dir.join("sources").join("source-index.json"),
        &SourceIndex {
            sources: vec![SourceEntry {
                source_id: "source:statement-csv".to_string(),
                path: source_path.to_string_lossy().replace('\\', "/"),
                sha256: source_bytes_hash(&bytes),
            }],
        },
    )?;
    Ok(())
}

/// Imports a small NIST Election Results Reporting CDF-style JSON fixture into
/// RCOUNT. This is a first adapter slice, not a complete CDF implementation.
pub fn import_nist_cdf_json(path: &Path) -> Result<RcountPackage, RcountIoError> {
    let value: Value = serde_json::from_slice(&fs::read(path)?)?;
    let report = value.get("ElectionReport").unwrap_or(&value);
    let status = parse_nist_status(
        report
            .get("ResultsStatus")
            .and_then(Value::as_str)
            .unwrap_or("canvassed"),
    )?;

    let mut reporting_units = BTreeMap::new();
    for unit in array_field(report, "GpUnit")? {
        let id = nist_id(unit, "GpUnit")?;
        let kind = unit
            .get("Type")
            .and_then(Value::as_str)
            .map(parse_nist_reporting_unit_kind)
            .transpose()?
            .unwrap_or(ReportingUnitKind::Precinct);
        reporting_units.insert(
            id.clone(),
            ReportingUnit {
                reporting_unit_id: id,
                kind,
                parent_jurisdiction: "nist-cdf".to_string(),
                source_ids: vec!["source:nist-cdf-json".to_string()],
                valid_from: None,
                valid_to: None,
            },
        );
    }

    let elections = array_field(report, "Election")?;
    let mut contests = BTreeMap::new();
    let mut summaries: BTreeMap<(String, String, CountStatus), SummaryAccumulator> =
        BTreeMap::new();

    for election in elections {
        for contest_value in array_field(election, "Contest")? {
            let contest_id = nist_id(contest_value, "Contest")?;
            let contest_title = nist_text(contest_value.get("Name")).unwrap_or(contest_id.clone());
            let vote_for = contest_value
                .get("NumberElected")
                .or_else(|| contest_value.get("VotesAllowed"))
                .and_then(Value::as_u64)
                .unwrap_or(1) as u32;
            let contest = contests.entry(contest_id.clone()).or_insert(Contest {
                contest_id: contest_id.clone(),
                title: contest_title.clone(),
                vote_for,
                selections: Vec::new(),
            });
            require_same(
                0,
                &contest_id,
                "contest_title",
                &contest.title,
                &contest_title,
            )?;

            for selection_value in array_field(contest_value, "ContestSelection")? {
                let selection_id = nist_id(selection_value, "ContestSelection")?;
                let selection_label =
                    nist_text(selection_value.get("Name")).unwrap_or(selection_id.clone());
                let selection_kind = if selection_value
                    .get("IsWriteIn")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    SelectionKind::WriteInBucket
                } else {
                    SelectionKind::Candidate
                };
                if !contest
                    .selections
                    .iter()
                    .any(|selection| selection.selection_id == selection_id)
                {
                    contest.selections.push(Selection {
                        selection_id: selection_id.clone(),
                        kind: selection_kind,
                        label: selection_label,
                    });
                }
                for count in array_field(selection_value, "VoteCounts")? {
                    let reporting_unit_id = nist_gp_unit_ref(count)?;
                    ensure_nist_unit(&mut reporting_units, &reporting_unit_id);
                    let votes = nist_count(count, "Count")?;
                    let summary = summaries
                        .entry((contest_id.clone(), reporting_unit_id.clone(), status))
                        .or_insert(SummaryAccumulator {
                            contest_id: contest_id.clone(),
                            reporting_unit_id,
                            status,
                            totals: Vec::new(),
                            seen_selection_ids: BTreeSet::new(),
                            undervotes: 0,
                            overvotes: 0,
                            blank_contests: 0,
                            counted_ballots: 0,
                        });
                    if summary.seen_selection_ids.insert(selection_id.clone()) {
                        summary.totals.push(SelectionTotal {
                            selection_id: selection_id.clone(),
                            votes,
                        });
                    }
                }
            }

            for other_count in optional_array_field(contest_value, "OtherCounts") {
                let reporting_unit_id = nist_gp_unit_ref(other_count)?;
                ensure_nist_unit(&mut reporting_units, &reporting_unit_id);
                let summary = summaries
                    .entry((contest_id.clone(), reporting_unit_id.clone(), status))
                    .or_insert(SummaryAccumulator {
                        contest_id: contest_id.clone(),
                        reporting_unit_id,
                        status,
                        totals: Vec::new(),
                        seen_selection_ids: BTreeSet::new(),
                        undervotes: 0,
                        overvotes: 0,
                        blank_contests: 0,
                        counted_ballots: 0,
                    });
                summary.undervotes += optional_nist_count(other_count, "Undervotes")?;
                summary.overvotes += optional_nist_count(other_count, "Overvotes")?;
                summary.blank_contests += optional_nist_count(other_count, "BlankVotes")?;
            }
        }
    }

    Ok(RcountPackage {
        rcount_version: RCOUNT_VERSION.to_string(),
        contests: contests.into_values().collect(),
        reporting_units: reporting_units.into_values().collect(),
        batches: Vec::new(),
        lineage: Vec::new(),
        rhist_refs: Vec::new(),
        rctx_refs: Vec::new(),
        inclusion_proofs: Vec::new(),
        cvr: Vec::new(),
        audit_algorithm_runs: Vec::new(),
        rla_audits: Vec::new(),
        manual_audits: Vec::new(),
        batch_comparison_audits: Vec::new(),
        summaries: summaries
            .into_values()
            .map(|mut summary| {
                summary.counted_ballots =
                    summary.totals.iter().map(|total| total.votes).sum::<i64>()
                        + summary.undervotes
                        + summary.overvotes
                        + summary.blank_contests;
                Summary {
                    contest_id: summary.contest_id,
                    reporting_unit_id: summary.reporting_unit_id,
                    batch_id: None,
                    status: summary.status,
                    totals: summary.totals,
                    undervotes: summary.undervotes,
                    overvotes: summary.overvotes,
                    blank_contests: summary.blank_contests,
                    counted_ballots: summary.counted_ballots,
                }
            })
            .collect(),
        status_events: Vec::<StatusEvent>::new(),
    })
}

pub fn write_nist_cdf_package_dir(
    dir: &Path,
    json_path: &Path,
    manifest: &RcountManifest,
    package: &RcountPackage,
) -> Result<(), RcountIoError> {
    write_package_dir(dir, manifest, package)?;
    let source_path = PathBuf::from("sources").join("nist-cdf-results.json");
    let bytes = fs::read(json_path)?;
    fs::write(dir.join(&source_path), &bytes)?;
    let synthetic = dir.join("sources").join("synthetic-summary-export.json");
    if synthetic.exists() {
        fs::remove_file(synthetic)?;
    }
    write_json_pretty(
        &dir.join("sources").join("source-index.json"),
        &SourceIndex {
            sources: vec![SourceEntry {
                source_id: "source:nist-cdf-json".to_string(),
                path: source_path.to_string_lossy().replace('\\', "/"),
                sha256: source_bytes_hash(&bytes),
            }],
        },
    )?;
    Ok(())
}

pub fn import_ri_2024_rep28_ballot_polling_audit(
    audit_report_csv: &Path,
    ballot_manifest_csv: &Path,
    ballot_retrieval_csv: &Path,
) -> Result<RcountPackage, RcountIoError> {
    let report_rows = read_csv_rows(audit_report_csv)?;
    let contest_row = section_data_row(&report_rows, "######## CONTESTS ########")?;
    let settings_row = section_data_row(&report_rows, "######## AUDIT SETTINGS ########")?;
    let rounds_row = section_data_row(&report_rows, "######## ROUNDS ########")?;

    let contest_title = ri_field(contest_row, 0, "contest name")?;
    let vote_for = ri_field(contest_row, 3, "votes allowed")?
        .parse::<u32>()
        .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
            field: "votes allowed".to_string(),
            value: contest_row[3].clone(),
        })?;
    let counted_ballots = ri_i64(contest_row, 4, "total ballots cast")?;
    let vote_totals = parse_ri_vote_totals(ri_field(contest_row, 5, "vote totals")?)?;
    let contest_id = "ri-2024-rep-28".to_string();
    let jurisdiction_unit_id = "ri:state:representative-28".to_string();
    let residual_ballots =
        counted_ballots - vote_totals.iter().map(|(_, votes)| votes).sum::<i64>();
    if residual_ballots < 0 {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "vote totals".to_string(),
            value: contest_row[5].clone(),
        });
    }

    let risk_limit_ppm = parse_percent_ppm(ri_field(settings_row, 3, "risk limit")?)?;
    let public_seed = normalize_seed(ri_field(settings_row, 4, "random seed")?)?;
    let audit_method = ri_field(settings_row, 2, "audit math type")?;
    if risk_limit_ppm == 0 {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "risk limit".to_string(),
            value: settings_row[3].clone(),
        });
    }
    validate_ri_sample_sources(&report_rows, rounds_row, ballot_retrieval_csv)?;

    let mut reporting_units = vec![ReportingUnit {
        reporting_unit_id: jurisdiction_unit_id.clone(),
        kind: ReportingUnitKind::DistrictTotal,
        parent_jurisdiction: "Rhode Island".to_string(),
        source_ids: vec![
            "source:ri-rla-audit-report".to_string(),
            "source:ri-rla-ballot-manifest".to_string(),
        ],
        valid_from: None,
        valid_to: None,
    }];

    let mut batches = Vec::new();
    let mut manifest_reader = csv::Reader::from_path(ballot_manifest_csv)?;
    for (index, row) in manifest_reader
        .deserialize::<RhodeIslandManifestRow>()
        .enumerate()
    {
        let row_number = index + 2;
        let row = row?;
        let batch_name = required_ri_string(row_number, "Batch Name", row.batch_name)?;
        let batch_id = format!("ri:batch:{}", slug_id(&batch_name));
        let ballots = parse_ri_i64_string(row_number, "Number of Ballots", row.number_of_ballots)?;
        let kind = if batch_name.to_ascii_lowercase().starts_with("mb ") {
            BatchKind::Mail
        } else {
            BatchKind::ElectionDay
        };
        let reporting_unit_kind = match kind {
            BatchKind::Mail => ReportingUnitKind::MailBatch,
            _ => ReportingUnitKind::CentralCountBatch,
        };
        reporting_units.push(ReportingUnit {
            reporting_unit_id: batch_id.clone(),
            kind: reporting_unit_kind,
            parent_jurisdiction: "Rhode Island".to_string(),
            source_ids: vec!["source:ri-rla-ballot-manifest".to_string()],
            valid_from: None,
            valid_to: None,
        });
        batches.push(BatchManifest {
            batch_id: batch_id.clone(),
            reporting_unit_id: batch_id,
            kind,
            status: CountStatus::Canvassed,
            accepted_ballots: ballots,
            counted_ballots: ballots,
            rejected_ballots: 0,
            source_refs: vec![
                "source:ri-rla-ballot-manifest".to_string(),
                format!("container:{}", row.container.trim()),
                format!("tabulator:{}", row.tabulator.trim()),
            ],
        });
    }
    let manifest_ballots = batches
        .iter()
        .map(|batch| batch.counted_ballots)
        .sum::<i64>();
    if manifest_ballots != counted_ballots {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "manifest ballot total".to_string(),
            value: manifest_ballots.to_string(),
        });
    }

    Ok(RcountPackage {
        rcount_version: RCOUNT_VERSION.to_string(),
        contests: vec![Contest {
            contest_id: contest_id.clone(),
            title: contest_title.to_string(),
            vote_for,
            selections: vote_totals
                .iter()
                .map(|(label, _)| Selection {
                    selection_id: format!("ri-2024-rep28:{}", slug_id(label)),
                    kind: if label.eq_ignore_ascii_case("write-in") {
                        SelectionKind::WriteInBucket
                    } else {
                        SelectionKind::Candidate
                    },
                    label: label.clone(),
                })
                .collect(),
        }],
        reporting_units,
        batches,
        lineage: Vec::new(),
        rhist_refs: Vec::new(),
        rctx_refs: Vec::new(),
        inclusion_proofs: Vec::new(),
        cvr: Vec::new(),
        audit_algorithm_runs: ri_ballot_polling_algorithm_runs(
            &contest_id,
            audit_method,
            risk_limit_ppm,
            &public_seed,
            &vote_totals,
        ),
        rla_audits: Vec::new(),
        manual_audits: Vec::new(),
        batch_comparison_audits: Vec::new(),
        summaries: vec![Summary {
            contest_id,
            reporting_unit_id: jurisdiction_unit_id,
            batch_id: None,
            status: CountStatus::Canvassed,
            totals: vote_totals
                .into_iter()
                .map(|(label, votes)| SelectionTotal {
                    selection_id: format!("ri-2024-rep28:{}", slug_id(&label)),
                    votes,
                })
                .collect(),
            undervotes: 0,
            overvotes: 0,
            blank_contests: residual_ballots,
            counted_ballots,
        }],
        status_events: Vec::<StatusEvent>::new(),
    })
}

pub(crate) fn ri_ballot_polling_algorithm_runs(
    contest_id: &str,
    audit_method: &str,
    risk_limit_ppm: u32,
    public_seed: &str,
    vote_totals: &[(String, i64)],
) -> Vec<AuditAlgorithmRun> {
    let Some(method_id) = ri_ballot_polling_method_id(audit_method) else {
        return Vec::new();
    };
    let mut ranked = vote_totals
        .iter()
        .filter(|(label, votes)| !label.eq_ignore_ascii_case("write-in") && *votes > 0)
        .collect::<Vec<_>>();
    ranked.sort_by(|(_, left), (_, right)| right.cmp(left));
    let assertions = if ranked.len() >= 2 {
        vec![AuditAssertion {
            assertion_id: "assertion:ri-2024-rep28-top-two".to_string(),
            kind: AuditAssertionKind::PluralityWinnerLoser,
            assorter_id: "minerva-ballot-polling-top-two-v1".to_string(),
            assorter_upper_bound: rcount_core::RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some(format!("ri-2024-rep28:{}", slug_id(&ranked[0].0))),
            loser_selection_id: Some(format!("ri-2024-rep28:{}", slug_id(&ranked[1].0))),
        }]
    } else {
        Vec::new()
    };

    Vec::from([AuditAlgorithmRun {
        run_id: "audit-run:ri-2024-rep28-minerva".to_string(),
        contest_id: contest_id.to_string(),
        method_id: method_id.to_string(),
        sampling_mode: AuditSamplingMode::WithReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(risk_limit_ppm),
        reported_winner_votes: ranked.first().map(|(_, votes)| *votes as u64),
        reported_loser_votes: ranked.get(1).map(|(_, votes)| *votes as u64),
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
        assertions,
        sample_steps: Vec::new(),
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec![
            "source:ri-rla-audit-report".to_string(),
            "source:ri-rla-ballot-retrieval".to_string(),
            format!("public_seed:{public_seed}"),
        ],
    }])
}

pub(crate) fn ri_ballot_polling_method_id(audit_method: &str) -> Option<&'static str> {
    match audit_method.trim().to_ascii_uppercase().as_str() {
        "MINERVA" => Some(MINERVA_BALLOT_POLLING_METHOD_ID),
        "ATHENA" => Some(ATHENA_BALLOT_POLLING_METHOD_ID),
        _ => None,
    }
}

pub fn ri_2024_rep28_manifest(package: &RcountPackage) -> Result<RcountManifest, RcountIoError> {
    Ok(RcountManifest {
        rcount_version: RCOUNT_VERSION.to_string(),
        jurisdiction: Jurisdiction {
            country: "US".to_string(),
            state: "RI".to_string(),
            county: "statewide".to_string(),
        },
        election: Election {
            date: "2024-11-05".to_string(),
            election_type: "general".to_string(),
            scope: "state-representative-district-28".to_string(),
        },
        status: "canvassed".to_string(),
        hash_algorithm: "sha256".to_string(),
        content_hash: package_content_hash(package)?,
        created_by: CreatedBy {
            tool: "rcount-io-ri-rla-adapter".to_string(),
            version: RCOUNT_VERSION.to_string(),
        },
    })
}

pub fn ri_2024_rep28_source_summary(
    audit_report_csv: &Path,
    ballot_retrieval_csv: &Path,
) -> Result<RhodeIslandRlaSourceSummary, RcountIoError> {
    let report_rows = read_csv_rows(audit_report_csv)?;
    let settings_row = section_data_row(&report_rows, "######## AUDIT SETTINGS ########")?;
    let rounds_row = section_data_row(&report_rows, "######## ROUNDS ########")?;
    let sampled_ballots = ri_sampled_ballot_keys(&report_rows)?;
    let retrieval_ballots = ri_retrieval_ballot_keys(ballot_retrieval_csv)?;
    Ok(RhodeIslandRlaSourceSummary {
        adapter_id: "ri-2024-rep28-ballot-polling-v1".to_string(),
        contest_id: "ri-2024-rep-28".to_string(),
        audit_method: ri_field(settings_row, 2, "audit math type")?.to_string(),
        risk_limit_ppm: parse_percent_ppm(ri_field(settings_row, 3, "risk limit")?)?,
        public_seed: normalize_seed(ri_field(settings_row, 4, "random seed")?)?,
        declared_sample_size: ri_u32(rounds_row, 3, "sample size")?,
        sampled_ballot_rows: sampled_ballots.len(),
        retrieval_rows: retrieval_ballots.len(),
        claim_boundary: vec![
            "source rows are preserved and hashed".to_string(),
            "audit report sampled-ballot rows match retrieval CSV rows by ballot key".to_string(),
            "retrieval row count does not exceed declared sample size".to_string(),
            "Minerva risk calculation is recorded but not replayed".to_string(),
            "ballot-level human observations are not independently verified".to_string(),
        ],
    })
}

pub fn write_ri_2024_rep28_package_dir(
    dir: &Path,
    audit_report_csv: &Path,
    ballot_manifest_csv: &Path,
    ballot_retrieval_csv: &Path,
    manifest: &RcountManifest,
    package: &RcountPackage,
) -> Result<(), RcountIoError> {
    write_package_dir(dir, manifest, package)?;
    let synthetic = dir.join("sources").join("synthetic-summary-export.json");
    if synthetic.exists() {
        fs::remove_file(synthetic)?;
    }
    let sources = [
        (
            "source:ri-rla-audit-report",
            "ri-2024-rep28-audit-report.csv",
            audit_report_csv,
        ),
        (
            "source:ri-rla-ballot-manifest",
            "ri-2024-rep28-ballot-manifest.csv",
            ballot_manifest_csv,
        ),
        (
            "source:ri-rla-ballot-retrieval",
            "ri-2024-rep28-ballot-retrieval.csv",
            ballot_retrieval_csv,
        ),
    ];
    let mut entries = Vec::new();
    for (source_id, file_name, path) in sources {
        let source_path = PathBuf::from("sources").join(file_name);
        let bytes = fs::read(path)?;
        fs::write(dir.join(&source_path), &bytes)?;
        entries.push(SourceEntry {
            source_id: source_id.to_string(),
            path: source_path.to_string_lossy().replace('\\', "/"),
            sha256: source_bytes_hash(&bytes),
        });
    }
    write_json_pretty(
        &dir.join("sources").join("source-index.json"),
        &SourceIndex { sources: entries },
    )?;
    write_json_pretty(
        &dir.join("transcripts")
            .join("ri-2024-rep28-source-summary.json"),
        &ri_2024_rep28_source_summary(audit_report_csv, ballot_retrieval_csv)?,
    )?;
    Ok(())
}
