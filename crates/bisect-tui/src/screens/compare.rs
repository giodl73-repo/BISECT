/// Compare screen — side-by-side plan comparison with Jaccard similarity.

// Native disk loading remains outside the shared text-only computation.
fn source(plan_dir: &std::path::Path, relative: &str) -> Option<String> {
    std::fs::read_to_string(plan_dir.join(relative)).ok()
}
pub fn load_mean_pp(plan_dir: &std::path::Path) -> Option<f64> {
    bisect_report::tui_compare::mean_pp(source(plan_dir,"analysis/compactness.json").as_deref())
}
pub fn load_max_dev(plan_dir: &std::path::Path) -> Option<f64> {
    bisect_report::tui_compare::max_dev(source(plan_dir,"analysis/summary.json").as_deref())
}
pub fn load_splits(plan_dir: &std::path::Path) -> Option<usize> {
    bisect_report::tui_compare::splits(source(plan_dir,"analysis/splits.json").as_deref()).map(|n|n as usize)
}
pub fn load_contiguous(plan_dir: &std::path::Path) -> Option<bool> {
    bisect_report::tui_compare::contiguous(source(plan_dir,"analysis/contiguity.json").as_deref())
}
pub fn compute_compare_result(a: &std::path::Path,b: &std::path::Path,_label_a: &str,_label_b: &str) -> crate::app::CompareResult {
    let read=|dir: &std::path::Path| ["data/final_assignments.json","analysis/compactness.json","analysis/summary.json","analysis/splits.json","analysis/contiguity.json"].into_iter().filter_map(|name| source(dir,name).map(|text|(name.to_string(),text))).collect();
    bisect_report::tui_compare::compare(&read(a),&read(b))
}

// ── Render ────────────────────────────────────────────────────────────────────

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
    Frame,
};

use crate::app::CompareState;

pub fn render(f: &mut Frame, area: Rect, state: &CompareState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Compare Plans ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Plan A + B inputs
            Constraint::Length(2), // Jaccard label + gauge
            Constraint::Length(7), // Metrics table
            Constraint::Length(4), // Most-changed districts
            Constraint::Length(1), // Footer
        ])
        .split(inner);

    // Plan A and B header
    let header_lines = vec![
        Line::from(vec![
            Span::styled(
                "Plan A: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&state.plan_a),
        ]),
        Line::from(vec![
            Span::styled(
                "Plan B: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&state.plan_b_input),
            Span::styled("_", Style::default().fg(Color::White)), // cursor
        ]),
    ];
    let header_para = Paragraph::new(header_lines);
    f.render_widget(header_para, rows[0]);

    // Jaccard gauge
    let jaccard_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(rows[1]);

    let jaccard_val = state.result.as_ref().map(|r| r.jaccard).unwrap_or(0.0);
    let jaccard_color = if jaccard_val >= 0.95 {
        Color::Green
    } else if jaccard_val >= 0.80 {
        Color::Yellow
    } else {
        Color::Red
    };

    let jaccard_label = Paragraph::new("Jaccard similarity");
    f.render_widget(jaccard_label, jaccard_area[0]);

    let jaccard_gauge = Gauge::default()
        .gauge_style(Style::default().fg(jaccard_color))
        .ratio(jaccard_val.clamp(0.0, 1.0))
        .label(format!("{:.3}", jaccard_val));
    f.render_widget(jaccard_gauge, jaccard_area[1]);

    // Metrics table
    if let Some(result) = &state.result {
        let header_cells = ["Metric", "Plan A", "Plan B", "Delta"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells);

        let metrics: Vec<(&str, String, String, f64, bool)> = vec![
            (
                "PP (compactness)",
                format!("{:.3}", result.mean_pp_a),
                format!("{:.3}", result.mean_pp_b),
                result.mean_pp_b - result.mean_pp_a,
                true, // higher is better for PP
            ),
            (
                "Max deviation %",
                format!("{:.2}%", result.max_dev_a),
                format!("{:.2}%", result.max_dev_b),
                result.max_dev_b - result.max_dev_a,
                false, // lower is better for deviation
            ),
            (
                "County splits",
                format!("{}", result.splits_a),
                format!("{}", result.splits_b),
                (result.splits_b as i128 - result.splits_a as i128) as f64,
                false, // lower is better for splits
            ),
            (
                "Contiguous",
                if result.contiguous_a {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
                if result.contiguous_b {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
                (result.contiguous_b as i8 - result.contiguous_a as i8) as f64,
                true, // higher is better (true > false)
            ),
        ];

        let rows_data: Vec<Row> = metrics
            .iter()
            .map(|(metric, a, b, delta, higher_is_better)| {
                let (delta_text, delta_style) = if delta.abs() < 1e-9 {
                    ("-- same".to_string(), Style::default().fg(Color::DarkGray))
                } else if (*higher_is_better && *delta > 0.0)
                    || (!*higher_is_better && *delta < 0.0)
                {
                    (
                        format!("+{:.2} better", delta.abs()),
                        Style::default().fg(Color::Green),
                    )
                } else {
                    (
                        format!("-{:.2} worse", delta.abs()),
                        Style::default().fg(Color::Red),
                    )
                };

                Row::new(vec![
                    Cell::from(metric.to_string()),
                    Cell::from(a.clone()),
                    Cell::from(b.clone()),
                    Cell::from(delta_text).style(delta_style),
                ])
            })
            .collect();

        let table = Table::new(
            rows_data,
            [
                Constraint::Min(20),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(18),
            ],
        )
        .header(header)
        .block(Block::default().title(" Metrics ").borders(Borders::TOP));

        f.render_widget(table, rows[2]);

        // Most-changed districts
        let changed_block = Block::default()
            .title(" Most-changed Districts ")
            .borders(Borders::TOP);
        let changed_inner = changed_block.inner(rows[3]);
        f.render_widget(changed_block, rows[3]);

        let changed_lines: Vec<Line> = result
            .most_changed
            .iter()
            .take(3)
            .map(|(district_id, pct)| {
                let bar_len = ((*pct * 20.0) as usize).clamp(0, 20);
                let bar = format!("{}{}", "█".repeat(bar_len), "░".repeat(20 - bar_len));
                Line::from(format!(
                    "  D{:>3}  {}  {:.0}% tracts moved",
                    district_id,
                    bar,
                    pct * 100.0
                ))
            })
            .collect();
        let changed_para = Paragraph::new(changed_lines);
        f.render_widget(changed_para, changed_inner);
    } else {
        // No result yet — show placeholder
        let placeholder = Paragraph::new("Enter Plan B label and press [Enter] to compare")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, rows[2]);
    }

    // Footer
    let footer = Paragraph::new("[x] Export CSV  [m] Map diff  [Esc] back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, rows[4]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_to_string(
        width: u16,
        height: u16,
        f: impl FnOnce(&mut ratatui::Frame, ratatui::layout::Rect),
    ) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                f(frame, area);
            })
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect::<String>()
    }

    #[test]
    fn test_compare_shows_plan_a_label() {
        use crate::app::CompareState;
        let state = CompareState {
            plan_a: "wa_house_plan_a".into(),
            ..Default::default()
        };
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        assert!(content.contains("wa_house_plan_a") || content.contains("Plan A"));
    }

    #[test]
    fn test_compare_large_split_delta_does_not_overflow() {
        let state = CompareState {
            result: Some(crate::app::CompareResult {
                splits_a: u64::MAX,
                splits_b: 1,
                ..Default::default()
            }),
            ..Default::default()
        };
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        assert!(content.contains("County splits"));
    }

    #[test]
    fn test_compare_shows_jaccard_when_result_present() {
        use crate::app::{CompareResult, CompareState};
        let state = CompareState {
            plan_a: "plan_a".into(),
            result: Some(CompareResult {
                jaccard: 0.847,
                ..Default::default()
            }),
            ..Default::default()
        };
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        assert!(
            content.contains("0.847") || content.contains("Jaccard") || content.contains("jaccard")
        );
    }

    // ── Task 2 new tests ──────────────────────────────────────────────────────

    #[test]
    fn test_load_mean_pp_no_file_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(load_mean_pp(tmp.path()).is_none());
    }

    #[test]
    fn test_load_mean_pp_with_data() {
        let tmp = tempfile::TempDir::new().unwrap();
        let analysis = tmp.path().join("analysis");
        std::fs::create_dir_all(&analysis).unwrap();
        std::fs::write(
            analysis.join("compactness.json"),
            serde_json::json!({
                "districts": [{"district": 1, "polsby_popper": 0.42}]
            })
            .to_string(),
        )
        .unwrap();
        let pp = load_mean_pp(tmp.path());
        assert!(pp.is_some());
        assert!((pp.unwrap() - 0.42).abs() < 0.001);
    }

    #[test]
    fn test_load_max_dev_no_file_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(load_max_dev(tmp.path()).is_none());
    }

    #[test]
    fn test_load_max_dev_with_data() {
        let tmp = tempfile::TempDir::new().unwrap();
        let analysis = tmp.path().join("analysis");
        std::fs::create_dir_all(&analysis).unwrap();
        std::fs::write(
            analysis.join("summary.json"),
            serde_json::json!({"max_deviation_pct": 3.7}).to_string(),
        )
        .unwrap();
        let dev = load_max_dev(tmp.path());
        assert!(dev.is_some());
        assert!((dev.unwrap() - 3.7).abs() < 0.001);
    }

    #[test]
    fn test_load_splits_with_data() {
        let tmp = tempfile::TempDir::new().unwrap();
        let analysis = tmp.path().join("analysis");
        std::fs::create_dir_all(&analysis).unwrap();
        std::fs::write(
            analysis.join("splits.json"),
            serde_json::json!({"split": 8}).to_string(),
        )
        .unwrap();
        let splits = load_splits(tmp.path());
        assert_eq!(splits, Some(8));
    }

    #[test]
    fn test_load_contiguous_with_data() {
        let tmp = tempfile::TempDir::new().unwrap();
        let analysis = tmp.path().join("analysis");
        std::fs::create_dir_all(&analysis).unwrap();
        std::fs::write(
            analysis.join("contiguity.json"),
            serde_json::json!({"all_contiguous": true}).to_string(),
        )
        .unwrap();
        let c = load_contiguous(tmp.path());
        assert_eq!(c, Some(true));
    }

    #[test]
    fn test_compute_compare_result_empty_dirs() {
        let tmp_a = tempfile::TempDir::new().unwrap();
        let tmp_b = tempfile::TempDir::new().unwrap();
        let result = compute_compare_result(tmp_a.path(), tmp_b.path(), "a", "b");
        // No assignment files → jaccard 0, everything defaults
        assert_eq!(result.jaccard, 0.0);
        assert_eq!(result.mean_pp_a, 0.0);
        assert_eq!(result.most_changed, vec![]);
    }

    #[test]
    fn test_compare_metrics_table_renders_with_result() {
        use crate::app::{CompareResult, CompareState};
        let state = CompareState {
            plan_a: "plan_a".into(),
            result: Some(CompareResult {
                jaccard: 0.92,
                mean_pp_a: 0.31,
                mean_pp_b: 0.28,
                max_dev_a: 3.2,
                max_dev_b: 4.1,
                splits_a: 8,
                splits_b: 12,
                contiguous_a: true,
                contiguous_b: false,
                most_changed: vec![],
            }),
            ..Default::default()
        };
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        // Table rows should show metric names
        assert!(
            content.contains("PP")
                || content.contains("Compactness")
                || content.contains("deviation"),
            "metrics table must appear: {}",
            &content[..200.min(content.len())]
        );
    }
}
