/// Home screen — two-panel plan browser with table and detail panel.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // Quick-action bar (1 line) + main content
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    // Quick-action bar
    let action_bar = Paragraph::new(Line::from(vec![
        Span::styled("[r] Run  ", Style::default().fg(Color::Cyan)),
        Span::styled("[c] Compare  ", Style::default().fg(Color::Green)),
        Span::styled("[v] Verify  ", Style::default().fg(Color::Yellow)),
        Span::styled("[d] Doctor  ", Style::default().fg(Color::Magenta)),
        Span::styled("[s] Sort  ", Style::default().fg(Color::White)),
        Span::styled("[?] Help  ", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(action_bar, outer[0]);

    // Main content: left (65%) + right (35%)
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(outer[1]);

    render_plan_list(f, panels[0], app);
    render_detail_panel(f, panels[1], app);
}

fn render_plan_list(f: &mut Frame, area: Rect, app: &App) {
    let plans = app.visible_plans();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Plans ({}) ", plans.len()));

    if plans.is_empty() {
        let msg = Paragraph::new("No plans found. Press [r] to run...")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, area);
        return;
    }

    let header_cells = ["Label", "St", "Chamber", "Yr", "D", "Sp", "C"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = plans
        .iter()
        .enumerate()
        .map(|(i, plan)| {
            let contiguous_str = match plan.all_contiguous {
                Some(true) => "Y",
                Some(false) => "N",
                None => "?",
            };
            let splits_str = plan
                .county_splits
                .map(|s| s.to_string())
                .unwrap_or_else(|| "?".to_string());

            let cells = vec![
                Cell::from(plan.label.as_str()),
                Cell::from(plan.state_code.as_str()),
                Cell::from(plan.chamber.as_str()),
                Cell::from(plan.year.as_str()),
                Cell::from(plan.num_districts.to_string()),
                Cell::from(splits_str),
                Cell::from(contiguous_str),
            ];

            // Determine row style
            let is_selected = i == app.selected_plan;
            let base_style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else if plan.all_contiguous == Some(false) {
                Style::default().fg(Color::Red)
            } else if plan.county_splits.map(|s| s > 10).unwrap_or(false) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            Row::new(cells).style(base_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),     // Label
            Constraint::Length(4),   // St
            Constraint::Length(12),  // Chamber
            Constraint::Length(5),   // Yr
            Constraint::Length(5),   // D
            Constraint::Length(5),   // Sp
            Constraint::Length(3),   // C
        ],
    )
    .header(header)
    .block(block);

    f.render_widget(table, area);
}

fn render_detail_panel(f: &mut Frame, area: Rect, app: &App) {
    let plans = app.visible_plans();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Detail ");

    if plans.is_empty() || app.selected_plan >= plans.len() {
        f.render_widget(block, area);
        return;
    }

    let plan = plans[app.selected_plan];

    // Split detail panel: header text + gauges + footer actions
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // name + state/chamber/year
            Constraint::Length(1),  // PP gauge label
            Constraint::Length(1),  // PP gauge
            Constraint::Length(1),  // deviation gauge label
            Constraint::Length(1),  // deviation gauge
            Constraint::Length(1),  // splits gauge label
            Constraint::Length(1),  // splits gauge
            Constraint::Length(1),  // contiguous indicator
            Constraint::Min(1),     // actions
        ])
        .split(inner);

    // Plan name and metadata
    let header_text = Paragraph::new(vec![
        Line::from(Span::styled(
            &plan.label,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "{}  {}  {}",
            plan.state_name, plan.chamber, plan.year
        )),
        Line::from(format!("{} districts", plan.num_districts)),
    ]);
    f.render_widget(header_text, rows[0]);

    // PP gauge (0–1)
    let pp_val = plan.mean_pp.unwrap_or(0.0).clamp(0.0, 1.0);
    let pp_label = Paragraph::new("Polsby-Popper (compactness)");
    f.render_widget(pp_label, rows[1]);
    let pp_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(pp_val)
        .label(format!("{:.3}", pp_val));
    f.render_widget(pp_gauge, rows[2]);

    // Deviation gauge (0–25%)
    let dev_pct = plan.max_deviation_pct.unwrap_or(0.0);
    let dev_ratio = (dev_pct / 25.0).clamp(0.0, 1.0);
    let dev_label = Paragraph::new("Max deviation %");
    f.render_widget(dev_label, rows[3]);
    let dev_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(dev_ratio)
        .label(format!("{:.2}%", dev_pct));
    f.render_widget(dev_gauge, rows[4]);

    // Splits gauge (0–50)
    let splits = plan.county_splits.unwrap_or(0);
    let splits_ratio = (splits as f64 / 50.0).clamp(0.0, 1.0);
    let splits_label = Paragraph::new("County splits");
    f.render_widget(splits_label, rows[5]);
    let splits_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Magenta))
        .ratio(splits_ratio)
        .label(format!("{}", splits));
    f.render_widget(splits_gauge, rows[6]);

    // Contiguous indicator
    let (contiguous_text, contiguous_color) = match plan.all_contiguous {
        Some(true) => ("Contiguous: YES", Color::Green),
        Some(false) => ("Contiguous: NO", Color::Red),
        None => ("Contiguous: ?", Color::DarkGray),
    };
    let contiguous_widget = Paragraph::new(contiguous_text)
        .style(Style::default().fg(contiguous_color));
    f.render_widget(contiguous_widget, rows[7]);

    // Actions
    let actions = Paragraph::new("[Enter] open  [a] analyze  [c] compare  [x] export")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Left);
    f.render_widget(actions, rows[8]);
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
    fn test_home_renders_plan_label() {
        use crate::app::PlanSummary;
        let mut app = crate::app::App::default();
        app.plans = vec![PlanSummary {
            label: "wa_house_test".into(),
            state_code: "WA".into(),
            chamber: "house".into(),
            year: "2020".into(),
            num_districts: 98,
            ..Default::default()
        }];
        let content = render_to_string(120, 30, |f, area| render(f, area, &app));
        assert!(content.contains("wa_house_test"), "plan label must appear: {content}");
    }

    #[test]
    fn test_home_empty_state_shown_when_no_plans() {
        let app = crate::app::App::default();
        let content = render_to_string(120, 20, |f, area| render(f, area, &app));
        assert!(
            content.contains("No plans") || content.contains("no plans") || content.contains("[r]"),
            "empty state must show: {content}"
        );
    }

    #[test]
    fn test_home_quick_action_bar_visible() {
        let app = crate::app::App::default();
        let content = render_to_string(120, 20, |f, area| render(f, area, &app));
        assert!(content.contains("[r]"), "quick action bar must show [r] Run");
        assert!(content.contains("[v]"), "quick action bar must show [v] Verify");
    }
}
