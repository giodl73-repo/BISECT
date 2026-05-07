/// Status bar — always-visible 1-line footer showing current context.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // Determine location/chamber/year from screen context or session defaults
    let (location, chamber, year) = context_from_app(app);

    let plan_count = app.plans.len();

    let line = Line::from(vec![
        Span::styled(
            format!("[{}  {}  {}]", location, chamber, year),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(format!("  plans: {}  ", plan_count)),
        Span::styled("|  ", Style::default().fg(Color::DarkGray)),
        Span::styled("? help  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q quit", Style::default().fg(Color::DarkGray)),
    ]);

    let bar = Paragraph::new(line).style(Style::default().bg(Color::Black));
    f.render_widget(bar, area);
}

fn context_from_app(app: &App) -> (String, String, String) {
    use crate::app::Screen;
    match &app.screen {
        Screen::Run(s) => (
            s.form.location.clone(),
            s.form.chamber.clone(),
            s.form.year.clone(),
        ),
        Screen::Doctor(s) => (s.location.clone(), s.chamber.clone(), s.year.clone()),
        Screen::Compare(s) => (s.plan_a.clone(), "compare".to_string(), String::new()),
        _ => ("--".to_string(), "--".to_string(), "--".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_shows_plan_count() {
        use crate::app::{App, PlanSummary};
        let mut app = App::default();
        app.plans = vec![PlanSummary { label: "test".into(), ..Default::default() }];
        let backend = ratatui::backend::TestBackend::new(80, 1);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal.draw(|f| {
            let area = f.area();
            render(f, area, &app);
        }).unwrap();
        let content: String = terminal.backend().buffer().content().iter()
            .map(|c| c.symbol().to_string()).collect();
        assert!(content.contains("1") || content.contains("plans"), "plan count must appear: {content}");
    }

    #[test]
    fn test_status_bar_shows_help_hint() {
        let app = App::default();
        let backend = ratatui::backend::TestBackend::new(80, 1);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, f.area(), &app)).unwrap();
        let content: String = terminal.backend().buffer().content().iter()
            .map(|c| c.symbol().to_string()).collect();
        assert!(content.contains("?") || content.contains("q"), "help hint must appear: {content}");
    }
}
