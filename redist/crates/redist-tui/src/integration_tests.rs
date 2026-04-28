// L1 integration: discover_plans → home screen → verify plan appears in rendered buffer

#[cfg(test)]
mod integration_tests {
    use crate::{app::App, plans, screens};
    use ratatui::{backend::TestBackend, Terminal};
    use tempfile::TempDir;

    fn make_plan_dir(tmp: &TempDir, label: &str) -> std::path::PathBuf {
        let plan_dir = tmp.path()
            .join("v1").join("2020").join("plans").join(label);
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();

        std::fs::write(plan_dir.join("manifest.json"), serde_json::json!({
            "label": label,
            "state_code": "WA",
            "chamber": "house",
            "year": "2020",
            "num_districts": 98,
        }).to_string()).unwrap();

        // Use keys matching what plans::read_plan expects
        std::fs::write(analysis_dir.join("compactness.json"), serde_json::json!({
            "mean_polsby_popper": 0.31
        }).to_string()).unwrap();

        std::fs::write(analysis_dir.join("summary.json"), serde_json::json!({
            "max_deviation_pct": 3.2
        }).to_string()).unwrap();

        std::fs::write(analysis_dir.join("splits.json"), serde_json::json!({
            "split_count": 8
        }).to_string()).unwrap();

        std::fs::write(analysis_dir.join("contiguity.json"), serde_json::json!({
            "all_contiguous": true
        }).to_string()).unwrap();

        plan_dir
    }

    #[test]
    fn test_discover_then_render_home_shows_plan() {
        let tmp = TempDir::new().unwrap();
        make_plan_dir(&tmp, "wa_house_integration_test");

        let plans = plans::discover_plans(
            tmp.path().to_str().unwrap(), "v1", "2020"
        );
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].label, "wa_house_integration_test");
        assert!((plans[0].mean_pp.unwrap_or(0.0) - 0.31).abs() < 0.01);
        assert_eq!(plans[0].county_splits, Some(8));
        assert_eq!(plans[0].all_contiguous, Some(true));

        // Render home screen and verify plan label appears
        let mut app = App::default();
        app.plans = plans;

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| {
            let area = f.area();
            screens::home::render(f, area, &app);
        }).unwrap();

        let content: String = terminal.backend().buffer()
            .content().iter()
            .map(|c| c.symbol().to_string())
            .collect();

        assert!(
            content.contains("wa_house_integration_test"),
            "plan label must appear in rendered home screen. Content: {}",
            &content[..200.min(content.len())]
        );
    }

    #[test]
    fn test_discover_multiple_plans_sorted_in_render() {
        let tmp = TempDir::new().unwrap();
        make_plan_dir(&tmp, "z_plan");
        make_plan_dir(&tmp, "a_plan");
        make_plan_dir(&tmp, "m_plan");

        let plans = plans::discover_plans(
            tmp.path().to_str().unwrap(), "v1", "2020"
        );
        assert_eq!(plans.len(), 3);
        assert_eq!(plans[0].label, "a_plan");  // sorted alphabetically

        let mut app = App::default();
        app.plans = plans;

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| {
            screens::home::render(f, f.area(), &app);
        }).unwrap();

        let content: String = terminal.backend().buffer()
            .content().iter()
            .map(|c| c.symbol().to_string())
            .collect();

        // a_plan should appear before z_plan in the sorted list
        let a_pos = content.find("a_plan").unwrap_or(usize::MAX);
        let z_pos = content.find("z_plan").unwrap_or(usize::MAX);
        assert!(a_pos < z_pos, "a_plan should appear before z_plan when sorted ascending");
    }

    #[test]
    fn test_compare_data_loading_from_plan_dirs() {
        use crate::screens::compare::{compute_compare_result, load_mean_pp, load_max_dev};

        let tmp_a = TempDir::new().unwrap();
        let tmp_b = TempDir::new().unwrap();

        // Set up plan A with analysis files
        let analysis_a = tmp_a.path().join("analysis");
        std::fs::create_dir_all(&analysis_a).unwrap();
        std::fs::write(
            analysis_a.join("compactness.json"),
            serde_json::json!({
                "districts": [{"district": 1, "polsby_popper": 0.42}]
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            analysis_a.join("summary.json"),
            serde_json::json!({"max_deviation_pct": 3.2}).to_string(),
        )
        .unwrap();
        std::fs::write(
            analysis_a.join("splits.json"),
            serde_json::json!({"split": 8}).to_string(),
        )
        .unwrap();
        std::fs::write(
            analysis_a.join("contiguity.json"),
            serde_json::json!({"all_contiguous": true}).to_string(),
        )
        .unwrap();

        // Set up plan B with different values
        let analysis_b = tmp_b.path().join("analysis");
        std::fs::create_dir_all(&analysis_b).unwrap();
        std::fs::write(
            analysis_b.join("compactness.json"),
            serde_json::json!({
                "districts": [{"district": 1, "polsby_popper": 0.35}]
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            analysis_b.join("summary.json"),
            serde_json::json!({"max_deviation_pct": 4.1}).to_string(),
        )
        .unwrap();
        std::fs::write(
            analysis_b.join("splits.json"),
            serde_json::json!({"split": 12}).to_string(),
        )
        .unwrap();
        std::fs::write(
            analysis_b.join("contiguity.json"),
            serde_json::json!({"all_contiguous": false}).to_string(),
        )
        .unwrap();

        // Verify individual loaders
        let pp_a = load_mean_pp(tmp_a.path());
        assert!(pp_a.is_some());
        assert!((pp_a.unwrap() - 0.42).abs() < 0.001, "plan A mean PP must match");

        let dev_b = load_max_dev(tmp_b.path());
        assert!(dev_b.is_some());
        assert!((dev_b.unwrap() - 4.1).abs() < 0.001, "plan B max dev must match");

        // Verify compute_compare_result
        let result =
            compute_compare_result(tmp_a.path(), tmp_b.path(), "plan_a", "plan_b");

        // No assignments → jaccard 0
        assert_eq!(result.jaccard, 0.0, "jaccard must be 0 with no assignment files");
        assert!((result.mean_pp_a - 0.42).abs() < 0.001, "mean_pp_a must be 0.42");
        assert!((result.mean_pp_b - 0.35).abs() < 0.001, "mean_pp_b must be 0.35");
        assert_eq!(result.splits_a, 8, "splits_a must be 8");
        assert_eq!(result.splits_b, 12, "splits_b must be 12");
        assert!(result.contiguous_a, "plan A must be contiguous");
        assert!(!result.contiguous_b, "plan B must not be contiguous");
    }

    #[test]
    fn test_status_parser_integration_with_progress() {
        use crate::runner::{parse_status_line, update_progress_from_message};
        use crate::app::RunProgress;

        let mut progress = RunProgress::default();

        // Simulate receiving STATUS lines from a real run
        let lines = vec![
            "STATUS:1:WA: loading adjacency",
            "STATUS:1:WA: recursive bisection into 98 districts",
            "STATUS:1:WA: balance OK, writing outputs",
            "STATUS:1:WA: complete (98D, 154000ms)",
        ];

        for line in lines {
            if let Some(msg) = parse_status_line(line) {
                update_progress_from_message(&mut progress, &msg);
            }
        }

        // After "balance OK" message, balance_ok should be true
        assert!(progress.balance_ok, "balance_ok must be set after balance OK message");
    }
}
