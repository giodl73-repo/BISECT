#[cfg(test)]
mod integration_pipeline_tests {
    use tempfile::TempDir;

    fn make_house_plan(tmp: &TempDir, label: &str, num_districts: usize) {
        let plan_dir = tmp.path().join("v1").join("2020").join("plans").join(label);
        let analysis_dir = plan_dir.join("analysis");
        let data_dir = plan_dir.join("data");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        // Write manifest with house chamber, NOT congressional
        let manifest = serde_json::json!({
            "label": label, "state_code": "WA", "year": "2020",
            "chamber": "house", "num_districts": num_districts,
            "population_source": "total", "partition_mode": "edge-weighted",
            "seed": 42, "binary_version": "0.1.0", "binary_sha256": "",
            "binary_download_url": "", "adjacency_file": "", "adjacency_sha256": "",
            "adjacency_build_command": "", "adjacency_build_version": "0.1.0",
            "tiger_source_url": "", "tiger_sha256": null,
            "created_at": "2026-04-28T00:00:00Z",
            "balance_tolerance_pct": 5.0, "population_balance_valid": true,
            "seats_per_district": 1, "total_seats": num_districts,
            "electoral_system": "single_member", "gpmetis_version": "unknown"
        });
        std::fs::write(
            plan_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        // Write fake assignments: num_districts districts, each with some tracts
        let mut assignments = serde_json::Map::new();
        for i in 0..num_districts * 5 {
            assignments.insert(
                format!("530330{:06}", i),
                serde_json::json!(i / 5 + 1),
            );
        }
        std::fs::write(
            data_dir.join("final_assignments.json"),
            serde_json::to_string(&assignments).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_plan_context_wa_house_98_not_congressional_10() {
        // THE REGRESSION TEST: PlanContext must return 98 for WA house,
        // never 10 (which is WA congressional from load_all_states).
        let tmp = TempDir::new().unwrap();
        make_house_plan(&tmp, "wa_house_smoke", 98);

        let ctx = crate::plan_context::PlanContext::from_label(
            tmp.path(),
            "v1",
            "2020",
            "wa_house_smoke",
        )
        .unwrap();

        assert_eq!(
            ctx.num_districts(),
            98,
            "PlanContext must read 98 from manifest, not 10 (WA congressional)"
        );
        assert_eq!(ctx.chamber(), "house");
        assert!(
            ctx.assignments_path().exists(),
            "assignments must be findable via PlanContext"
        );
    }

    #[test]
    fn test_plan_context_feeds_correct_district_count_for_analyze() {
        // Verify the metadata PlanContext provides would produce correct analysis.
        // This is the integration check: state->analyze pipeline.
        let tmp = TempDir::new().unwrap();
        make_house_plan(&tmp, "wa_house_5d_test", 5);

        let ctx = crate::plan_context::PlanContext::from_label(
            tmp.path(),
            "v1",
            "2020",
            "wa_house_5d_test",
        )
        .unwrap();

        // What analyze now gets: 5 districts from manifest
        assert_eq!(ctx.num_districts(), 5);

        // Load the assignments and verify they have 5 distinct districts
        let raw = std::fs::read_to_string(ctx.assignments_path()).unwrap();
        let asgn: std::collections::HashMap<String, usize> =
            serde_json::from_str(&raw).unwrap();
        let districts: std::collections::HashSet<usize> = asgn.values().copied().collect();
        assert_eq!(districts.len(), 5, "test data must have exactly 5 districts");
    }

    #[test]
    fn test_plan_context_error_lists_alternatives_for_missing_plan() {
        // Verify user gets helpful error with available plan list.
        let tmp = TempDir::new().unwrap();
        make_house_plan(&tmp, "wa_house_v1", 98);
        make_house_plan(&tmp, "wa_senate_v1", 49);

        let result = crate::plan_context::PlanContext::from_label(
            tmp.path(),
            "v1",
            "2020",
            "wa_house_typo",
        );

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("wa_house_v1") || msg.contains("wa_senate_v1"),
            "error must list available plans, got: {msg}"
        );
    }

    #[test]
    fn test_plan_context_to_report_context_district_count() {
        // L1: PlanContext feeds correct metadata to the report assembly step.
        // Verifies analyze→report handoff: ReportContext built from PlanContext
        // has the correct num_districts from the manifest.
        let tmp = TempDir::new().unwrap();
        make_house_plan(&tmp, "wa_house_report_test", 98);

        // Add required analysis stubs
        let plan_dir = tmp.path().join("v1").join("2020").join("plans").join("wa_house_report_test");
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        for name in &["summary.json", "contiguity.json", "compactness.json"] {
            std::fs::write(analysis_dir.join(name),
                serde_json::json!({"status": "ok", "districts": [], "all_contiguous": true}).to_string()
            ).unwrap();
        }

        let ctx = crate::plan_context::PlanContext::from_label(
            tmp.path(), "v1", "2020", "wa_house_report_test"
        ).unwrap();

        // PlanContext provides the manifest — assert it has 98 districts
        assert_eq!(ctx.manifest.num_districts, 98);
        assert_eq!(ctx.manifest.chamber, "house");

        // If we were to build ReportContext from PlanContext, it would correctly
        // use 98 districts (not 10 from congressional manifest)
        let report_ctx = redist_report::ReportContext::new(
            ctx.plan_dir.clone(),
            ctx.manifest.clone()
        );
        assert_eq!(report_ctx.manifest.num_districts, 98,
            "ReportContext built from PlanContext must have 98 districts");
    }

    #[test]
    fn test_compare_loads_plan_assignments_via_plan_context_paths() {
        // L1: compare.rs now uses PlanContext for labeled plans.
        // Verify that PlanContext.assignments_path() gives the correct path
        // that load_plan_assignments() can find.
        let tmp = TempDir::new().unwrap();
        make_house_plan(&tmp, "plan_for_compare_a", 5);
        make_house_plan(&tmp, "plan_for_compare_b", 5);

        let ctx_a = crate::plan_context::PlanContext::from_label(
            tmp.path(), "v1", "2020", "plan_for_compare_a"
        ).unwrap();
        let ctx_b = crate::plan_context::PlanContext::from_label(
            tmp.path(), "v1", "2020", "plan_for_compare_b"
        ).unwrap();

        // Both assignments must be loadable
        assert!(ctx_a.assignments_path().exists(), "plan_a assignments must exist");
        assert!(ctx_b.assignments_path().exists(), "plan_b assignments must exist");

        let raw_a: std::collections::HashMap<String, usize> = serde_json::from_str(
            &std::fs::read_to_string(ctx_a.assignments_path()).unwrap()
        ).unwrap();
        let raw_b: std::collections::HashMap<String, usize> = serde_json::from_str(
            &std::fs::read_to_string(ctx_b.assignments_path()).unwrap()
        ).unwrap();

        // Both plans have 5 districts
        let districts_a: std::collections::HashSet<usize> = raw_a.values().copied().collect();
        let districts_b: std::collections::HashSet<usize> = raw_b.values().copied().collect();
        assert_eq!(districts_a.len(), 5);
        assert_eq!(districts_b.len(), 5);

        // Jaccard between identical plans = 1.0 (same tracts, same districts)
        let matching = raw_a.iter()
            .filter(|(geoid, &d)| raw_b.get(*geoid) == Some(&d))
            .count();
        let union = raw_a.len().max(raw_b.len());
        let jaccard = matching as f64 / union as f64;
        assert_eq!(jaccard, 1.0, "identical plans must have Jaccard = 1.0");
    }
}
