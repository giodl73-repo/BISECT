/// Command palette overlay — floating input with suggestions list.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

/// All available commands.
static ALL_COMMANDS: &[&str] = &[
    "run",
    "compare",
    "verify",
    "doctor",
    "sort",
    "filter",
    "export",
    "analyze",
    "map",
    "fetch",
    "help",
    "quit",
];

/// Return commands matching the given input (prefix match, case-insensitive).
/// Empty input returns empty (no suggestions until the user types).
pub fn suggestions(input: &str) -> Vec<&'static str> {
    if input.is_empty() {
        return Vec::new();
    }
    let lower = input.to_lowercase();
    ALL_COMMANDS
        .iter()
        .copied()
        .filter(|cmd| cmd.starts_with(lower.as_str()))
        .collect()
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    if !app.show_palette {
        return;
    }

    // Centre the palette: 50% width, upper-third vertically
    let palette_width = (area.width * 50 / 100).max(40).min(area.width);
    let palette_height = 12u16;
    let x = area.x + (area.width.saturating_sub(palette_width)) / 2;
    let y = area.y + area.height / 4;
    let palette_area = Rect::new(x, y, palette_width, palette_height.min(area.height));

    // Clear the area behind the palette
    f.render_widget(Clear, palette_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Command Palette ")
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(palette_area);
    f.render_widget(block, palette_area);

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    // Input line with ": " prefix and cursor
    let input_line = Line::from(vec![
        Span::styled(": ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(&app.palette_input),
        Span::styled("_", Style::default().fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(input_line), parts[0]);

    // Suggestions list
    let cmds = suggestions(&app.palette_input);
    let suggestion_lines: Vec<Line> = cmds
        .iter()
        .map(|cmd| {
            Line::from(Span::styled(
                format!("  {}", cmd),
                Style::default().fg(Color::White),
            ))
        })
        .collect();

    let suggestions_para = Paragraph::new(suggestion_lines);
    f.render_widget(suggestions_para, parts[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestions_run_matches_run() {
        let result = suggestions("run");
        assert!(result.contains(&"run"), "suggestions('run') must include 'run'");
    }

    #[test]
    fn test_suggestions_empty_returns_empty() {
        let result = suggestions("");
        assert!(result.is_empty(), "empty input must return empty suggestions");
    }

    #[test]
    fn test_suggestions_doc_matches_doctor() {
        let result = suggestions("doc");
        assert!(result.contains(&"doctor"), "suggestions('doc') must include 'doctor'");
    }

    #[test]
    fn test_suggestions_unknown_returns_empty() {
        let result = suggestions("zzz");
        assert!(result.is_empty(), "no commands start with 'zzz'");
    }
}
