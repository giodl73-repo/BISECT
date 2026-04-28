/// Compare screen — side-by-side plan comparison with Jaccard similarity.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
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
            Span::styled("Plan A: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&state.plan_a),
        ]),
        Line::from(vec![
            Span::styled("Plan B: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
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
                (result.splits_b as i64 - result.splits_a as i64) as f64,
                false, // lower is better for splits
            ),
            (
                "Contiguous",
                if result.contiguous_a { "Yes".to_string() } else { "No".to_string() },
                if result.contiguous_b { "Yes".to_string() } else { "No".to_string() },
                (result.contiguous_b as i8 - result.contiguous_a as i8) as f64,
                true, // higher is better (true > false)
            ),
        ];

        let rows_data: Vec<Row> = metrics
            .iter()
            .map(|(metric, a, b, delta, higher_is_better)| {
                let (delta_text, delta_style) = if delta.abs() < 1e-9 {
                    ("-- same".to_string(), Style::default().fg(Color::DarkGray))
                } else if (*higher_is_better && *delta > 0.0) || (!*higher_is_better && *delta < 0.0) {
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
        let changed_block = Block::default().title(" Most-changed Districts ").borders(Borders::TOP);
        let changed_inner = changed_block.inner(rows[3]);
        f.render_widget(changed_block, rows[3]);

        let changed_lines: Vec<Line> = result
            .most_changed
            .iter()
            .take(3)
            .map(|(district_id, pct)| {
                let bar_len = ((*pct * 20.0) as usize).clamp(0, 20);
                let bar = format!("{}{}", "█".repeat(bar_len), "░".repeat(20 - bar_len));
                Line::from(format!("  D{:>3}  {}  {:.0}% tracts moved", district_id, bar, pct * 100.0))
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
