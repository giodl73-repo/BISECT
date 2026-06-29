use super::*;
use rcount_core::{
    synthetic_athena_boundary_package, synthetic_awaire_boundary_package,
    synthetic_bad_california_rla_package, synthetic_bad_colorado_rla_package,
    synthetic_bad_cvr_summary_package, synthetic_bad_lineage_package,
    synthetic_bad_manual_audit_package, synthetic_bad_rla_discrepancy_package,
    synthetic_bad_rla_margin_package, synthetic_bad_rla_replay_package,
    synthetic_bad_rla_statistical_package, synthetic_bad_rla_stopping_package,
    synthetic_bad_selection_sum_package, synthetic_batch_comparison_algorithm_package,
    synthetic_batch_comparison_package, synthetic_batch_size_drift_comparison_package,
    synthetic_bayesian_tabulation_boundary_package, synthetic_california_rla_package,
    synthetic_canvass_correction_package, synthetic_choice_bearing_proof_package,
    synthetic_colorado_rla_package, synthetic_cvr_summary_package,
    synthetic_kaplan_markov_macro_package, synthetic_mail_batch_added_package,
    synthetic_manual_audit_package, synthetic_minerva_multi_round_package,
    synthetic_minerva_round_one_package, synthetic_missing_batch_package,
    synthetic_missing_hand_tally_batch_comparison_package,
    synthetic_precinct_split_lineage_package, synthetic_privacy_inclusion_package,
    synthetic_raire_boundary_package, synthetic_rla_discrepancy_package,
    synthetic_rla_margin_package, synthetic_rla_replay_package, synthetic_rla_statistical_package,
    synthetic_rla_stopping_package, synthetic_soba_observable_ballot_boundary_package,
    synthetic_stratified_hybrid_package, synthetic_summary_basic_package,
    synthetic_summary_basic_package_with_base_references, RctxReference, RhistReference,
    SYN_RCTX_L0_CROSSWALK_HASH, SYN_RHIST_L2_PACKAGE_HASH,
};

#[test]
fn round_trips_synthetic_summary_basic_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_summary_basic_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (decoded_manifest, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_manifest.content_hash, manifest.content_hash);
    assert_eq!(decoded_package.summaries.len(), 3);
    assert_eq!(verify_source_index(tmp.path()).unwrap().len(), 1);
    verify_summary_basic_dir(tmp.path()).unwrap();
}

#[test]
fn round_trips_synthetic_canvass_correction_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_canvass_correction_package();
    let manifest = synthetic_canvass_correction_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.summaries.len(), 6);
    assert_eq!(decoded_package.status_events.len(), 2);
    assert_eq!(verify_source_index(tmp.path()).unwrap().len(), 1);
}

#[test]
fn round_trips_synthetic_minerva_round_one_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_minerva_round_one_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.audit_algorithm_runs.len(), 1);
    assert_eq!(
        decoded_package.audit_algorithm_runs[0].method_id,
        MINERVA_BALLOT_POLLING_METHOD_ID
    );
    assert_eq!(
        decoded_package.audit_algorithm_runs[0].sample_steps.len(),
        6
    );
}

#[test]
fn round_trips_synthetic_minerva_multi_round_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_minerva_multi_round_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    let steps = &decoded_package.audit_algorithm_runs[0].sample_steps;
    assert_eq!(steps.len(), 6);
    assert_eq!(steps[4].round_index, Some(0));
    assert_eq!(steps[5].round_index, Some(1));
}

#[test]
fn round_trips_synthetic_athena_boundary_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_athena_boundary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.audit_algorithm_runs.len(), 1);
    assert_eq!(
        decoded_package.audit_algorithm_runs[0].method_id,
        ATHENA_BALLOT_POLLING_METHOD_ID
    );
    assert_eq!(
        decoded_package.audit_algorithm_runs[0].decision,
        rcount_core::AuditAlgorithmDecision::Boundary
    );
}

#[test]
fn round_trips_synthetic_stratified_hybrid_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_stratified_hybrid_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    let run = decoded_package
        .audit_algorithm_runs
        .iter()
        .find(|run| run.method_id == rcount_core::STRATIFIED_HYBRID_RLA_METHOD_ID)
        .expect("stratified run must round-trip");
    assert_eq!(run.strata.len(), 2);
    assert_eq!(
        run.combining_rule_id.as_deref(),
        Some("suite-nuisance-boundary-v1")
    );
    assert_eq!(
        run.nuisance_parameter,
        Some(rcount_core::RationalValue {
            numerator: 1,
            denominator: 2,
        })
    );
    assert_eq!(run.strata[0].allocation_ppm, Some(500_000));
    assert_eq!(run.strata[1].allocation_ppm, Some(500_000));
}

#[test]
fn round_trips_synthetic_raire_boundary_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_raire_boundary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    let run = &decoded_package.audit_algorithm_runs[0];
    assert_eq!(run.method_id, rcount_core::RAIRE_IRV_METHOD_ID);
    assert_eq!(run.rcv_elimination_order.len(), 3);
    assert_eq!(run.sample_steps[0].ranked_choices[0], "cand-a");
}

#[test]
fn round_trips_synthetic_awaire_boundary_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_awaire_boundary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.audit_algorithm_runs[0].method_id,
        rcount_core::AWAIRE_IRV_METHOD_ID
    );
}

#[test]
fn round_trips_synthetic_bayesian_tabulation_boundary_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bayesian_tabulation_boundary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    let run = &decoded_package.audit_algorithm_runs[0];
    assert_eq!(
        run.method_id,
        rcount_core::BAYESIAN_TABULATION_AUDIT_METHOD_ID
    );
    assert_eq!(
        run.bayesian_prior_id.as_deref(),
        Some("dirichlet-multinomial-toy-prior-v1")
    );
    assert_eq!(run.posterior_risk_ppm, Some(42_000));
}

#[test]
fn round_trips_synthetic_soba_observable_ballot_boundary_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_soba_observable_ballot_boundary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    let run = &decoded_package.audit_algorithm_runs[0];
    assert_eq!(
        run.method_id,
        rcount_core::SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID
    );
    assert_eq!(decoded_package.inclusion_proofs.len(), 1);
    assert_eq!(
        run.sample_steps[0].sample_unit_id,
        "proof:accepted-token-001"
    );
}

#[test]
fn imports_statement_csv_and_preserves_source_hash() {
    let tmp = tempfile::tempdir().unwrap();
    let csv_path = tmp.path().join("statement.csv");
    std::fs::write(
            &csv_path,
            concat!(
                "contest_id,contest_title,vote_for,selection_id,selection_label,selection_kind,reporting_unit_id,reporting_unit_kind,parent_jurisdiction,status,votes,undervotes,overvotes,blank_contests,counted_ballots\n",
                "syn-2024-mayor,Synthetic Mayor,1,cand-a,Candidate A,candidate,syn:precinct:P-001,precinct,syn-county-1,canvassed,40,3,1,0,80\n",
                "syn-2024-mayor,Synthetic Mayor,1,cand-b,Candidate B,candidate,syn:precinct:P-001,precinct,syn-county-1,canvassed,35,3,1,0,80\n",
                "syn-2024-mayor,Synthetic Mayor,1,write-in,Write-in,write-in-bucket,syn:precinct:P-001,precinct,syn-county-1,canvassed,1,3,1,0,80\n",
                "syn-2024-mayor,Synthetic Mayor,1,cand-a,Candidate A,candidate,syn:precinct:P-002,precinct,syn-county-1,canvassed,25,4,0,1,60\n",
                "syn-2024-mayor,Synthetic Mayor,1,cand-b,Candidate B,candidate,syn:precinct:P-002,precinct,syn-county-1,canvassed,30,4,0,1,60\n",
                "syn-2024-mayor,Synthetic Mayor,1,write-in,Write-in,write-in-bucket,syn:precinct:P-002,precinct,syn-county-1,canvassed,0,4,0,1,60\n",
                "syn-2024-mayor,Synthetic Mayor,1,cand-a,Candidate A,candidate,syn:jurisdiction:SYN,jurisdiction-total,syn,canvassed,65,7,1,1,140\n",
                "syn-2024-mayor,Synthetic Mayor,1,cand-b,Candidate B,candidate,syn:jurisdiction:SYN,jurisdiction-total,syn,canvassed,65,7,1,1,140\n",
                "syn-2024-mayor,Synthetic Mayor,1,write-in,Write-in,write-in-bucket,syn:jurisdiction:SYN,jurisdiction-total,syn,canvassed,1,7,1,1,140\n",
            ),
        )
        .unwrap();

    let package = import_statement_csv(&csv_path).unwrap();
    verify_package(&package).unwrap();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    let package_dir = tmp.path().join("package");
    write_statement_csv_package_dir(&package_dir, &csv_path, &manifest, &package).unwrap();

    let (_, decoded_package) = read_package_dir(&package_dir).unwrap();
    verify_package(&decoded_package).unwrap();
    let checks = verify_source_index(&package_dir).unwrap();
    assert_eq!(checks[0].source_id, "source:statement-csv");
    assert!(package_dir.join("sources/statement-of-votes.csv").exists());
    assert!(!package_dir
        .join("sources/synthetic-summary-export.json")
        .exists());
}

#[test]
fn imports_nist_cdf_json_and_preserves_source_hash() {
    let tmp = tempfile::tempdir().unwrap();
    let json_path = tmp.path().join("cdf.json");
    std::fs::write(
            &json_path,
            r#"{
  "ElectionReport": {
    "ResultsStatus": "canvassed",
    "GpUnit": [
      {"@id": "syn:precinct:P-001", "Type": "precinct", "Name": {"Text": [{"Value": "P-001"}]}},
      {"@id": "syn:precinct:P-002", "Type": "precinct", "Name": {"Text": [{"Value": "P-002"}]}},
      {"@id": "syn:jurisdiction:SYN", "Type": "county", "Name": {"Text": [{"Value": "SYN County"}]}}
    ],
    "Election": [{
      "Contest": [{
        "@id": "syn-2024-mayor",
        "Name": {"Text": [{"Value": "Synthetic Mayor"}]},
        "NumberElected": 1,
        "ContestSelection": [
          {"@id": "cand-a", "Name": {"Text": [{"Value": "Candidate A"}]}, "VoteCounts": [
            {"GpUnitId": "syn:precinct:P-001", "Count": 40},
            {"GpUnitId": "syn:precinct:P-002", "Count": 25},
            {"GpUnitId": "syn:jurisdiction:SYN", "Count": 65}
          ]},
          {"@id": "cand-b", "Name": {"Text": [{"Value": "Candidate B"}]}, "VoteCounts": [
            {"GpUnitId": "syn:precinct:P-001", "Count": 35},
            {"GpUnitId": "syn:precinct:P-002", "Count": 30},
            {"GpUnitId": "syn:jurisdiction:SYN", "Count": 65}
          ]},
          {"@id": "write-in", "Name": {"Text": [{"Value": "Write-in"}]}, "IsWriteIn": true, "VoteCounts": [
            {"GpUnitId": "syn:precinct:P-001", "Count": 1},
            {"GpUnitId": "syn:precinct:P-002", "Count": 0},
            {"GpUnitId": "syn:jurisdiction:SYN", "Count": 1}
          ]}
        ],
        "OtherCounts": [
          {"GpUnitId": "syn:precinct:P-001", "Undervotes": 3, "Overvotes": 1, "BlankVotes": 0},
          {"GpUnitId": "syn:precinct:P-002", "Undervotes": 4, "Overvotes": 0, "BlankVotes": 1},
          {"GpUnitId": "syn:jurisdiction:SYN", "Undervotes": 7, "Overvotes": 1, "BlankVotes": 1}
        ]
      }]
    }]
  }
}"#,
        )
        .unwrap();

    let package = import_nist_cdf_json(&json_path).unwrap();
    verify_package(&package).unwrap();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    let package_dir = tmp.path().join("package");
    write_nist_cdf_package_dir(&package_dir, &json_path, &manifest, &package).unwrap();

    let (_, decoded_package) = read_package_dir(&package_dir).unwrap();
    verify_package(&decoded_package).unwrap();
    let checks = verify_source_index(&package_dir).unwrap();
    assert_eq!(checks[0].source_id, "source:nist-cdf-json");
    assert!(package_dir.join("sources/nist-cdf-results.json").exists());
}

#[test]
fn imports_ri_2024_rep28_rla_sources_and_manifest_batches() {
    let tmp = tempfile::tempdir().unwrap();
    let audit_path = tmp.path().join("audit-report.csv");
    let manifest_path = tmp.path().join("manifest.csv");
    let retrieval_path = tmp.path().join("retrieval.csv");
    std::fs::write(
            &audit_path,
            concat!(
                "######## ELECTION INFO ########,,,,,,,,\n",
                "Organization,Election Name,State,,,,,,\n",
                "Rhode Island,RI General Election 2024 Rep 28,RI,,,,,,\n",
                ",,,,,,,,\n",
                "######## CONTESTS ########,,,,,,,,\n",
                "Contest Name,Targeted?,Number of Winners,Votes Allowed,Total Ballots Cast,Vote Totals,,,\n",
                "Representative 28,Targeted,1,1,13136,Scott Guthrie: 3418; George A. Nardone: 4589; Write-in: 12,,,\n",
                ",,,,,,,,\n",
                "######## AUDIT SETTINGS ########,,,,,,,,\n",
                "Audit Name,Audit Type,Audit Math Type,Risk Limit,Random Seed,Online Data Entry?,,,\n",
                "11-5-24 Representative 28 Ballot Polling Audit,BALLOT_POLLING,MINERVA,9%,34053800000000000000,No,,,\n",
                ",,,,,,,,\n",
                "######## ROUNDS ########,,,,,,,,\n",
                "Round Number,Status,Started At,Sample Size,Risk Measurements,,,\n",
                "1,Ended,2024-11-20 21:52:00+00:00,1,George A. Nardone / Scott Guthrie: 0.054,,,\n",
                ",,,,,,,,\n",
                "######## SAMPLED BALLOTS ########,,,,,,,,\n",
                "Draw Number,Container,Tabulator,Batch Name,Ballot Position,Ticket Numbers,Audit Result,,\n",
                "1,0600,0315412524,EV Coventry,39,Round 1: 0.028518425968401157,George A. Nardone,,\n",
            ),
        )
        .unwrap();
    std::fs::write(
        &manifest_path,
        concat!(
            "Batch Name,Number of Ballots,Container,Tabulator\n",
            "EV Coventry,6751,0600,0315412524\n",
            "Coventry 0602,872,0602,0315412769\n",
            "Coventry 0603,612,0603,0315411890\n",
            "Coventry 0604,1055,0604,0315411395\n",
            "Coventry 0611,953,0611,0315412441\n",
            "Coventry 0612,294,0612,0315412579\n",
            "Coventry 0613,538,0613,0315412728\n",
            "Coventry 0614,368,0614,0315412655\n",
            "MB Coventry 1,628,C0017,8520060462\n",
            "MB Coventry 2,671,B0028,8516020237\n",
            "MB Coventry 3,205,A0025,8516020236\n",
            "MB Coventry 4,112,A0039,8516020236\n",
            "MB Coventry 5,65,B0064,8516020237\n",
            "MB Coventry 7,12,B0112,8516020237\n",
        ),
    )
    .unwrap();
    std::fs::write(
            &retrieval_path,
            concat!(
                "Container,Tabulator,Batch Name,Ballot Number,Ticket Numbers,Already Audited,Audit Board\n",
                "0600,0315412524,EV Coventry,39,0.028518425968401157,N,Audit Board #1\n",
            ),
        )
        .unwrap();

    let package =
        import_ri_2024_rep28_ballot_polling_audit(&audit_path, &manifest_path, &retrieval_path)
            .unwrap();
    verify_package(&package).unwrap();
    assert_eq!(package.batches.len(), 14);
    assert_eq!(package.summaries[0].blank_contests, 5117);
    assert_eq!(package.audit_algorithm_runs.len(), 1);
    assert_eq!(
        package.audit_algorithm_runs[0].method_id,
        rcount_core::MINERVA_BALLOT_POLLING_METHOD_ID
    );
    assert_eq!(
        package.audit_algorithm_runs[0].decision,
        rcount_core::AuditAlgorithmDecision::Boundary
    );
    assert_eq!(package.audit_algorithm_runs[0].risk_limit_ppm, Some(90_000));
    assert_eq!(
        package.audit_algorithm_runs[0].reported_winner_votes,
        Some(4589)
    );
    assert_eq!(
        package.audit_algorithm_runs[0].reported_loser_votes,
        Some(3418)
    );

    let manifest = ri_2024_rep28_manifest(&package).unwrap();
    let package_dir = tmp.path().join("ri-package");
    write_ri_2024_rep28_package_dir(
        &package_dir,
        &audit_path,
        &manifest_path,
        &retrieval_path,
        &manifest,
        &package,
    )
    .unwrap();
    let checks = verify_source_index(&package_dir).unwrap();
    assert_eq!(checks.len(), 3);
    assert!(package_dir
        .join("sources/ri-2024-rep28-ballot-retrieval.csv")
        .exists());
    let source_summary =
        std::fs::read_to_string(package_dir.join("transcripts/ri-2024-rep28-source-summary.json"))
            .unwrap();
    assert!(source_summary.contains(r#""sampled_ballot_rows": 1"#));
    assert!(source_summary.contains(r#""retrieval_rows": 1"#));
    assert!(!package_dir
        .join("sources/synthetic-summary-export.json")
        .exists());
}

#[test]
fn ri_2024_rep28_import_rejects_manifest_total_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let audit_path = tmp.path().join("audit-report.csv");
    let manifest_path = tmp.path().join("manifest.csv");
    let retrieval_path = tmp.path().join("retrieval.csv");
    std::fs::write(
            &audit_path,
            concat!(
                "######## CONTESTS ########,,,,,,,,\n",
                "Contest Name,Targeted?,Number of Winners,Votes Allowed,Total Ballots Cast,Vote Totals,,,\n",
                "Representative 28,Targeted,1,1,10,Scott Guthrie: 4; George A. Nardone: 3; Write-in: 0,,,\n",
                ",,,,,,,,\n",
                "######## AUDIT SETTINGS ########,,,,,,,,\n",
                "Audit Name,Audit Type,Audit Math Type,Risk Limit,Random Seed,Online Data Entry?,,,\n",
                "11-5-24 Representative 28 Ballot Polling Audit,BALLOT_POLLING,MINERVA,9%,34053800000000000000,No,,,\n",
                ",,,,,,,,\n",
                "######## ROUNDS ########,,,,,,,,\n",
                "Round Number,Status,Started At,Sample Size,Risk Measurements,,,\n",
                "1,Ended,2024-11-20 21:52:00+00:00,1,George A. Nardone / Scott Guthrie: 0.054,,,\n",
                ",,,,,,,,\n",
                "######## SAMPLED BALLOTS ########,,,,,,,,\n",
                "Draw Number,Container,Tabulator,Batch Name,Ballot Position,Ticket Numbers,Audit Result,,\n",
                "1,0600,0315412524,EV Coventry,1,Round 1: 0.1,George A. Nardone,,\n",
            ),
        )
        .unwrap();
    std::fs::write(
        &manifest_path,
        concat!(
            "Batch Name,Number of Ballots,Container,Tabulator\n",
            "EV Coventry,9,0600,0315412524\n",
        ),
    )
    .unwrap();
    std::fs::write(
            &retrieval_path,
            concat!(
                "Container,Tabulator,Batch Name,Ballot Number,Ticket Numbers,Already Audited,Audit Board\n",
                "0600,0315412524,EV Coventry,1,0.1,N,Audit Board #1\n",
            ),
        )
        .unwrap();

    assert!(matches!(
        import_ri_2024_rep28_ballot_polling_audit(&audit_path, &manifest_path, &retrieval_path),
        Err(RcountIoError::InvalidRhodeIslandRlaField { .. })
    ));
}

#[test]
fn round_trips_synthetic_mail_batch_added_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_mail_batch_added_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.batches.len(), 3);
    assert_eq!(
        decoded_package
            .summaries
            .iter()
            .filter(|summary| summary.batch_id.is_some())
            .count(),
        3
    );
}

#[test]
fn round_trips_synthetic_missing_batch_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_missing_batch_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.batches.len(), 2);
    assert_eq!(
        decoded_package
            .summaries
            .iter()
            .filter(|summary| summary.batch_id.as_deref() == Some("batch:P-001:late-mail"))
            .count(),
        1
    );
}

#[test]
fn round_trips_synthetic_precinct_split_lineage_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_precinct_split_lineage_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.lineage.len(), 2);
    assert!(decoded_package
        .lineage
        .iter()
        .any(|event| event.lineage_id == "lineage:P-004-split"));
}

#[test]
fn round_trips_rhist_references() {
    let tmp = tempfile::tempdir().unwrap();
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
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();

    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    assert!(tmp.path().join("normalized/rhist-refs.ndjson").is_file());

    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.rhist_refs.len(), 1);
    assert_eq!(
        decoded_package.rhist_refs[0].package_hash,
        "sha256:ccbddf423aa4ac08b0d45c4ac0b9db411293ea41fef3ac8fa93f9de9e85f66bb"
    );
    verify_package(&decoded_package).unwrap();
}

#[test]
fn round_trips_rctx_references() {
    let tmp = tempfile::tempdir().unwrap();
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
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();

    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    assert!(tmp.path().join("normalized/rctx-refs.ndjson").is_file());

    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.rctx_refs.len(), 1);
    assert_eq!(
        decoded_package.rctx_refs[0].crosswalk_hash.as_deref(),
        Some("sha256:2222222222222222222222222222222222222222222222222222222222222222")
    );
    verify_package(&decoded_package).unwrap();
}

#[test]
fn round_trips_shared_rctx_rhist_base_references() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_summary_basic_package_with_base_references();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();

    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    assert!(tmp.path().join("normalized/rctx-refs.ndjson").is_file());
    assert!(tmp.path().join("normalized/rhist-refs.ndjson").is_file());

    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.rctx_refs[0].crosswalk_hash.as_deref(),
        Some(SYN_RCTX_L0_CROSSWALK_HASH)
    );
    assert_eq!(
        decoded_package.rhist_refs[0].package_hash,
        SYN_RHIST_L2_PACKAGE_HASH
    );
    verify_package(&decoded_package).unwrap();
}

#[test]
fn round_trips_synthetic_bad_lineage_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_lineage_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.lineage.len(), 2);
    assert!(decoded_package.lineage[0]
        .current_reporting_unit_ids
        .contains(&"syn:precinct:P-004C".to_string()));
}

#[test]
fn round_trips_synthetic_privacy_inclusion_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_privacy_inclusion_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.inclusion_proofs.len(), 1);
    assert!(decoded_package.inclusion_proofs[0]
        .candidate_selections
        .is_empty());
}

#[test]
fn round_trips_synthetic_choice_bearing_proof_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_choice_bearing_proof_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.inclusion_proofs[0].candidate_selections,
        vec!["cand-a".to_string()]
    );
}

#[test]
fn round_trips_synthetic_cvr_summary_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_cvr_summary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.cvr.len(), 140);
    assert!(decoded_package
        .cvr
        .iter()
        .any(|row| row.cvr_id == "cvr:P-001:001"));
}

#[test]
fn round_trips_synthetic_bad_cvr_summary_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_cvr_summary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.cvr.len(), 140);
    assert!(verify_source_index(tmp.path()).is_ok());
}

#[test]
fn round_trips_synthetic_rla_replay_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_rla_replay_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.rla_audits.len(), 1);
    assert_eq!(decoded_package.rla_audits[0].sample_draws.len(), 12);
}

#[test]
fn round_trips_synthetic_bad_rla_replay_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_rla_replay_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.rla_audits[0].sample_draws[0].cvr_id,
        "cvr:P-999:999"
    );
}

#[test]
fn round_trips_synthetic_rla_stopping_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_rla_stopping_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.rla_audits[0].observations.len(), 12);
    assert_eq!(
        decoded_package.rla_audits[0].stopping_rule_id.as_deref(),
        Some("zero-discrepancy-threshold-v1")
    );
}

#[test]
fn round_trips_synthetic_bad_rla_stopping_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_rla_stopping_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.rla_audits[0].observations[0].observed_selection_ids,
        vec!["cand-b".to_string()]
    );
}

#[test]
fn round_trips_synthetic_rla_discrepancy_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_rla_discrepancy_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.rla_audits[0].discrepancies.len(), 1);
}

#[test]
fn round_trips_synthetic_bad_rla_discrepancy_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_rla_discrepancy_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.rla_audits[0].discrepancies.len(), 1);
}

#[test]
fn round_trips_synthetic_rla_margin_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_rla_margin_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.rla_audits[0]
            .margin
            .as_ref()
            .unwrap()
            .reported_margin,
        64
    );
}

#[test]
fn round_trips_synthetic_bad_rla_margin_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_rla_margin_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.rla_audits[0]
            .margin
            .as_ref()
            .unwrap()
            .reported_margin,
        65
    );
}

#[test]
fn round_trips_synthetic_rla_statistical_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_rla_statistical_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.rla_audits[0].declared_risk_ppm, Some(1303));
}

#[test]
fn round_trips_synthetic_bad_rla_statistical_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_rla_statistical_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.rla_audits[0].declared_risk_ppm, Some(1304));
}

#[test]
fn round_trips_synthetic_colorado_rla_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_colorado_rla_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.rla_audits[0]
            .jurisdiction_method_id
            .as_deref(),
        Some("colorado-rule-25-comparison-v1")
    );
}

#[test]
fn round_trips_synthetic_bad_colorado_rla_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_colorado_rla_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.rla_audits[0].public_seed,
        "3141592653589793238X"
    );
}

#[test]
fn round_trips_synthetic_california_rla_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_california_rla_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.rla_audits[0]
            .jurisdiction_method_id
            .as_deref(),
        Some("california-public-rla-v1")
    );
}

#[test]
fn round_trips_synthetic_bad_california_rla_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_california_rla_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.rla_audits[0]
            .audit_software_source_url
            .as_deref(),
        Some("synthetic-election-audit/rcount-open-rla-synthetic-v1")
    );
}

#[test]
fn round_trips_synthetic_manual_audit_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_manual_audit_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.manual_audits.len(), 1);
    assert_eq!(decoded_package.manual_audits[0].tolerance_votes, 0);
}

#[test]
fn round_trips_synthetic_bad_manual_audit_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_manual_audit_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.manual_audits[0].hand_totals[1].votes, 36);
}

#[test]
fn round_trips_synthetic_batch_comparison_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_batch_comparison_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(decoded_package.batch_comparison_audits.len(), 1);
    assert_eq!(
        decoded_package.batch_comparison_audits[0].declared_overstatement,
        2
    );
    assert!(tmp.path().join("audits/batch-comparison.ndjson").exists());
}

#[test]
fn round_trips_synthetic_batch_comparison_algorithm_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_batch_comparison_algorithm_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    let package_hashes: PackageHashes =
        read_json(&tmp.path().join("proofs/package-hashes.json")).unwrap();

    assert_eq!(package_hashes.audit_algorithm_run_count, 1);
    assert_eq!(package_hashes.batch_comparison_audit_count, 1);
    assert_eq!(decoded_package.audit_algorithm_runs.len(), 1);
    assert_eq!(decoded_package.batch_comparison_audits.len(), 1);
    assert_eq!(
        decoded_package.audit_algorithm_runs[0].sample_steps[0].sample_unit_id,
        "batch:P-001:election-day"
    );
    assert_eq!(
        decoded_package.audit_algorithm_runs[0].sample_steps[0]
            .assorter_value
            .denominator,
        5
    );
    assert!(tmp.path().join("audits/algorithm-runs.ndjson").exists());
    assert!(tmp.path().join("audits/batch-comparison.ndjson").exists());
}

#[test]
fn round_trips_synthetic_kaplan_markov_macro_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_kaplan_markov_macro_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    let run = &decoded_package.audit_algorithm_runs[0];

    assert_eq!(run.macro_ballot_count, Some(100));
    assert_eq!(run.macro_reported_margin, Some(10));
    assert_eq!(
        run.macro_gamma,
        Some(rcount_core::RationalValue {
            numerator: 11,
            denominator: 10,
        })
    );
    assert_eq!(run.sample_steps.len(), 16);
    assert!(tmp.path().join("audits/algorithm-runs.ndjson").exists());
}

#[test]
fn round_trips_synthetic_missing_hand_tally_batch_comparison_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_missing_hand_tally_batch_comparison_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.batch_comparison_audits[0].hand_totals.len(),
        1
    );
}

#[test]
fn round_trips_synthetic_batch_size_drift_comparison_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_batch_size_drift_comparison_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.batch_comparison_audits[0].declared_batch_ballots,
        69
    );
}

#[test]
fn rejects_manifest_content_hash_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_summary_basic_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let manifest_path = tmp.path().join("manifest.json");
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    raw["content_hash"] = serde_json::Value::String("sha256:bad".to_string());
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    assert!(matches!(
        read_package_dir(tmp.path()),
        Err(RcountIoError::ContentHashMismatch { .. })
    ));
}

#[test]
fn round_trips_synthetic_bad_selection_sum_package() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_bad_selection_sum_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
    assert_eq!(
        decoded_package.summaries[0].counted_ballots,
        synthetic_summary_basic_package().summaries[0].counted_ballots + 1
    );
    assert!(verify_source_index(tmp.path()).is_ok());
}

#[test]
fn rejects_tampered_source_file() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_summary_basic_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    std::fs::write(
        tmp.path()
            .join("sources")
            .join("synthetic-summary-export.json"),
        br#"{"tampered":true}"#,
    )
    .unwrap();

    assert!(matches!(
        verify_source_index(tmp.path()),
        Err(RcountIoError::SourceHashMismatch { .. })
    ));
}

#[test]
fn rejects_empty_source_index() {
    let tmp = tempfile::tempdir().unwrap();
    let package = synthetic_summary_basic_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();
    std::fs::write(
        tmp.path().join("sources").join("source-index.json"),
        br#"{"sources":[]}"#,
    )
    .unwrap();

    assert!(matches!(
        verify_source_index(tmp.path()),
        Err(RcountIoError::EmptySourceIndex)
    ));
}

#[test]
fn docs_summary_basic_fixture_verifies_when_present() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("summary-basic");
    if dir.exists() {
        verify_summary_basic_dir(&dir).unwrap();
    }
}
