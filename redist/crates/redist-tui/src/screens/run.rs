/// Run screen — form, running progress, and completion summary.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, LineGauge, Paragraph},
};

use crate::app::{RunPhase, RunState};

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    match &state.phase {
        RunPhase::Form => render_form(f, area, state),
        RunPhase::Running => render_running(f, area, state),
        RunPhase::Complete(result) => {
            let result = result.clone();
            render_complete(f, area, state, &result);
        }
    }
}

fn render_form(f: &mut Frame, area: Rect, state: &RunState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Run — Configure ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let form = &state.form;
    let fields = [
        ("Location", form.location.as_str()),
        ("Chamber", form.chamber.as_str()),
        ("Year", form.year.as_str()),
        ("Resolution", form.resolution.as_str()),
        ("Seed", form.seed.as_str()),
        ("Label", form.label.as_str()),
        ("Version", form.version.as_str()),
        ("Balance tol %", form.balance_tol.as_str()),
    ];

    let mut lines: Vec<Line> = fields
        .iter()
        .enumerate()
        .map(|(i, (label, value))| {
            let style = if i == form.focused_field {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::from(vec![
                Span::styled(format!("  {:<16}", label), style),
                Span::styled(format!(" {}", value), Style::default().fg(Color::White)),
            ])
        })
        .collect();

    // Doctor warnings in Yellow
    for warn in &form.doctor_warnings {
        lines.push(Line::from(Span::styled(
            format!("  WARN: {}", warn),
            Style::default().fg(Color::Yellow),
        )));
    }

    // Doctor errors in Red
    for err in &form.doctor_errors {
        lines.push(Line::from(Span::styled(
            format!("  ERR:  {}", err),
            Style::default().fg(Color::Red),
        )));
    }

    // Command preview footer
    let cmd = format!(
        "redist state --state {} --chamber {} --year {} --version {} --label {}",
        form.location, form.chamber, form.year, form.version, form.label
    );
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  > {}", cmd),
        Style::default().fg(Color::DarkGray),
    )));

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);
}

fn render_running(f: &mut Frame, area: Rect, state: &RunState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Run — In Progress ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let progress = &state.progress;

    // Count dynamic rows: one per depth + assigned + elapsed + balance + log preview
    let depth_count = progress.depths.len().max(1);
    let mut constraints = Vec::new();
    for _ in 0..depth_count {
        constraints.push(Constraint::Length(2)); // label + gauge
    }
    constraints.push(Constraint::Length(1)); // assigned LineGauge
    constraints.push(Constraint::Length(1)); // elapsed
    constraints.push(Constraint::Length(1)); // balance
    constraints.push(Constraint::Min(3));    // log preview

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut row_idx = 0;

    // Bisection depth gauges
    for (depth_i, &(done, total)) in progress.depths.iter().enumerate() {
        if row_idx + 1 >= rows.len() { break; }
        let ratio = if total > 0 { (done as f64 / total as f64).clamp(0.0, 1.0) } else { 0.0 };
        let label_area = rows[row_idx];
        let gauge_area = rows[row_idx + 1];

        let label = Paragraph::new(format!("Depth {} ({}/{})", depth_i + 1, done, total));
        f.render_widget(label, label_area);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(ratio)
            .label(format!("{:.0}%", ratio * 100.0));
        f.render_widget(gauge, gauge_area);
        row_idx += 2;
    }

    // If no depths yet, show placeholder
    if progress.depths.is_empty() && row_idx + 1 < rows.len() {
        let label = Paragraph::new("Bisection: waiting...").style(Style::default().fg(Color::DarkGray));
        f.render_widget(label, rows[row_idx]);
        row_idx += 2;
    }

    // Assigned LineGauge
    if row_idx < rows.len() {
        let total = progress.districts_total.max(1);
        let ratio = (progress.districts_assigned as f64 / total as f64).clamp(0.0, 1.0);
        let line_gauge = LineGauge::default()
            .filled_style(Style::default().fg(Color::Green))
            .ratio(ratio)
            .label(format!(
                "Assigned {}/{}",
                progress.districts_assigned, progress.districts_total
            ));
        f.render_widget(line_gauge, rows[row_idx]);
        row_idx += 1;
    }

    // Elapsed
    if row_idx < rows.len() {
        let elapsed = Paragraph::new(format!("Elapsed: {}s", progress.elapsed_secs));
        f.render_widget(elapsed, rows[row_idx]);
        row_idx += 1;
    }

    // Balance
    if row_idx < rows.len() {
        let (balance_text, color) = if progress.balance_ok {
            ("Balance: OK", Color::Green)
        } else {
            ("Balance: checking...", Color::Yellow)
        };
        let balance = Paragraph::new(balance_text).style(Style::default().fg(color));
        f.render_widget(balance, rows[row_idx]);
        row_idx += 1;
    }

    // Log preview (last 3 lines)
    if row_idx < rows.len() {
        let log_lines: Vec<Line> = state.log_lines
            .iter()
            .rev()
            .take(3)
            .rev()
            .map(|l| Line::from(Span::styled(l.as_str(), Style::default().fg(Color::DarkGray))))
            .collect();
        let log_block = Block::default().title(" Log ").borders(Borders::TOP);
        let log_para = Paragraph::new(log_lines).block(log_block);
        f.render_widget(log_para, rows[row_idx]);
    }
}

fn render_complete(f: &mut Frame, area: Rect, _state: &RunState, result: &crate::app::RunResult) {
    let (title_color, status_text) = if result.success {
        (Color::Green, "Run complete — SUCCESS")
    } else {
        (Color::Red, "Run complete — FAILED")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Run — {} ", status_text))
        .border_style(Style::default().fg(title_color));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status
            Constraint::Length(1), // PP label
            Constraint::Length(1), // PP gauge
            Constraint::Length(1), // deviation label
            Constraint::Length(1), // deviation gauge
            Constraint::Length(1), // splits label
            Constraint::Length(1), // splits gauge
            Constraint::Min(1),    // actions
        ])
        .split(inner);

    // Status line
    let elapsed_text = format!("Elapsed: {}s", result.elapsed_secs);
    let status_para = Paragraph::new(elapsed_text);
    f.render_widget(status_para, rows[0]);

    // PP gauge
    let pp_val = result.mean_pp.unwrap_or(0.0).clamp(0.0, 1.0);
    let pp_label = Paragraph::new("Polsby-Popper");
    f.render_widget(pp_label, rows[1]);
    let pp_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(pp_val)
        .label(format!("{:.3}", pp_val));
    f.render_widget(pp_gauge, rows[2]);

    // Deviation gauge
    let dev_pct = result.max_deviation_pct.unwrap_or(0.0);
    let dev_ratio = (dev_pct / 25.0).clamp(0.0, 1.0);
    let dev_label = Paragraph::new("Max deviation %");
    f.render_widget(dev_label, rows[3]);
    let dev_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(dev_ratio)
        .label(format!("{:.2}%", dev_pct));
    f.render_widget(dev_gauge, rows[4]);

    // Splits gauge
    let splits = result.county_splits.unwrap_or(0);
    let splits_ratio = (splits as f64 / 50.0).clamp(0.0, 1.0);
    let splits_label = Paragraph::new("County splits");
    f.render_widget(splits_label, rows[5]);
    let splits_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Magenta))
        .ratio(splits_ratio)
        .label(format!("{}", splits));
    f.render_widget(splits_gauge, rows[6]);

    // Actions
    let actions = Paragraph::new("[a] analyze  [r] re-run  [c] compare  [Enter] home")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(actions, rows[7]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_to_string(width: u16, height: u16, f: impl FnOnce(&mut ratatui::Frame, ratatui::layout::Rect)) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| {
            let area = frame.area();
            f(frame, area);
        }).unwrap();
        terminal.backend().buffer().content().iter()
            .map(|c| c.symbol().to_string())
            .collect::<String>()
    }

    #[test]
    fn test_run_form_renders_fields() {
        use crate::app::{RunState, RunForm};
        let mut state = RunState::default();
        state.form = RunForm {
            location: "WA".into(),
            chamber: "house".into(),
            year: "2020".into(),
            ..Default::default()
        };
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        assert!(content.contains("WA") || content.contains("house"), "form fields must appear");
    }

    #[test]
    fn test_run_progress_shows_depth_bars() {
        use crate::app::{RunState, RunPhase, RunProgress};
        let mut state = RunState::default();
        state.phase = RunPhase::Running;
        state.progress = RunProgress {
            depths: vec![(2, 2), (3, 4), (0, 8)],
            districts_assigned: 22,
            districts_total: 98,
            balance_ok: true,
            ..Default::default()
        };
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        assert!(content.contains("Depth") || content.contains("depth") || content.contains("2 / 2") || content.contains("22"));
    }

    #[test]
    fn test_run_completion_shows_pass_indicators() {
        use crate::app::{RunState, RunPhase, RunResult};
        let mut state = RunState::default();
        state.phase = RunPhase::Complete(RunResult {
            success: true,
            elapsed_secs: 154,
            mean_pp: Some(0.31),
            ..Default::default()
        });
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        assert!(content.contains("Done") || content.contains("Complete") || content.contains("154") || content.contains("SUCCESS"));
    }
}
