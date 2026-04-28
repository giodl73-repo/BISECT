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
}
