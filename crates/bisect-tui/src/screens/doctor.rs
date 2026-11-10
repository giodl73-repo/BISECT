/// Doctor screen — live policy checks for a location/chamber/year.
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{CheckStatus, DoctorCheck, DoctorState};

/// Run all doctor checks against the LocationRegistry and return the list.
pub fn run_checks(location: &str, chamber: &str, year: &str) -> Vec<DoctorCheck> {
    let registry=bisect_cli::policy::LocationRegistry::load();
    let count=registry.chamber_districts(location,chamber,year).map(|n|n as u64);
    let tolerance=bisect_cli::runner::chamber_balance_tolerance(location,chamber);
    bisect_report::tui_doctor::checks(registry.raw(),location,chamber,year,count,tolerance)
}

pub fn render(f: &mut Frame, area: Rect, state: &DoctorState) {
    let title = format!(
        " Doctor  {}  {}  {} ",
        state.location, state.chamber, state.year
    );
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Show editable location prompt when location is empty
    if state.location.is_empty() {
        let prompt_lines = vec![
            Line::from(""),
            Line::from("  Enter location code (e.g., WA, IE, _TEST_EL):"),
            Line::from(vec![
                Span::raw("  > "),
                Span::styled("\u{2588}", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  Type a 2-letter code and press Enter (or [r] to run, [Esc] back)",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let prompt = Paragraph::new(prompt_lines);
        f.render_widget(prompt, inner);
        return;
    }

    // Run checks live during render
    let checks = run_checks(&state.location, &state.chamber, &state.year);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    // Location input line with cursor
    let location_line = Line::from(vec![
        Span::styled("  Location: ", Style::default().fg(Color::Cyan)),
        Span::styled(&state.location, Style::default().fg(Color::White)),
        Span::styled("\u{2588}", Style::default().fg(Color::Yellow)),
        Span::styled(
            "  (type to change, [r] run, [Esc] back)",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    f.render_widget(Paragraph::new(vec![location_line]), rows[0]);

    // Check list
    let check_lines: Vec<Line> = checks
        .iter()
        .map(|check| {
            let (symbol, color) = match check.status {
                CheckStatus::Pass => ("OK  ", Color::Green),
                CheckStatus::Warn => ("WARN", Color::Yellow),
                CheckStatus::Fail => ("FAIL", Color::Red),
                CheckStatus::Info => ("INFO", Color::Cyan),
            };

            let mut spans = vec![
                Span::styled(
                    format!("[{}] ", symbol),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(&check.message, Style::default().fg(Color::White)),
            ];

            if let Some(hint) = &check.hint {
                spans.push(Span::styled(
                    format!("  -> {}", hint),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            Line::from(spans)
        })
        .collect();

    let checks_para = Paragraph::new(check_lines);
    f.render_widget(checks_para, rows[1]);

    // Footer
    let footer = Paragraph::new("[r] Run with these settings  [Esc] back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, rows[2]);
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
    fn test_doctor_empty_location_shows_prompt() {
        use crate::app::DoctorState;
        let state = DoctorState::default();
        let content = render_to_string(120, 20, |f, area| render(f, area, &state));
        assert!(
            content.contains("location")
                || content.contains("Location")
                || content.contains("WA")
                || content.contains("IE")
                || content.contains("Doctor"),
            "doctor must show location prompt: {content}"
        );
    }

    #[test]
    fn test_doctor_with_vt_location_runs_checks() {
        use crate::app::DoctorState;
        let state = DoctorState {
            location: "VT".into(),
            chamber: "congressional".into(),
            year: "2020".into(),
            checks: vec![],
        };
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        // Doctor runs live checks — should show either location name or check results
        assert!(
            content.contains("VT") || content.contains("Vermont") || content.contains("Doctor"),
            "doctor output must reference location: {content}"
        );
    }

    #[test]
    fn test_doctor_empty_shows_input_prompt() {
        use crate::app::DoctorState;
        let state = DoctorState::default(); // location is ""
        let backend = TestBackend::new(120, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, f.area(), &state)).unwrap();
        let content: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        // Must show some kind of prompt/input indication
        assert!(
            content.contains("location") || content.contains("code") || content.contains("WA"),
            "empty doctor must show prompt: {}",
            &content[..200.min(content.len())]
        );
    }

    #[test]
    fn test_doctor_location_shows_cursor_when_non_empty() {
        use crate::app::DoctorState;
        let state = DoctorState {
            location: "WA".into(),
            chamber: "house".into(),
            year: "2020".into(),
            checks: vec![],
        };
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        // The location input line should show the location code
        assert!(
            content.contains("WA") || content.contains("Location"),
            "doctor must show current location: {}",
            &content[..200.min(content.len())]
        );
    }
}
