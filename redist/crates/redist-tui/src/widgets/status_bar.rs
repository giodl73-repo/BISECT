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
