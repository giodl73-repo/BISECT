/// Glossary overlay — floating block with 7 metric definitions.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

/// All 7 glossary entries: (term, definition)
pub const GLOSSARY_ENTRIES: &[(&str, &str)] = &[
    (
        "Jaccard",
        "Fraction of census units shared between two plans. Range 0-1; >0.95 = near-identical.",
    ),
    (
        "PP / Polsby-Popper",
        "Compactness metric: 4pi*Area/Perimeter^2. Range 0-1; 1 = perfect circle, higher is more compact.",
    ),
    (
        "Max dev%",
        "Maximum population deviation from ideal district size, as a percent. Congressional standard: ±0.5%.",
    ),
    (
        "County splits",
        "Number of counties divided between two or more districts. Lower is better for community cohesion.",
    ),
    (
        "Contiguous",
        "All tracts in each district are geographically connected with no isolated islands.",
    ),
    (
        "VRA districts",
        "Majority-minority districts drawn to comply with Voting Rights Act Section 2 (minority opportunity districts).",
    ),
    (
        "Bisection depth",
        "Level of recursive binary splitting (depth 1 = 2 halves, depth N = 2^N parts). Determines granularity of METIS partitioning.",
    ),
];

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    if !app.show_glossary {
        return;
    }

    // Centre the glossary: 70% width, centred vertically
    let glos_width = (area.width * 70 / 100).max(50).min(area.width);
    let glos_height = (GLOSSARY_ENTRIES.len() as u16 * 2 + 3).min(area.height);
    let x = area.x + (area.width.saturating_sub(glos_width)) / 2;
    let y = area.y + (area.height.saturating_sub(glos_height)) / 2;
    let glos_area = Rect::new(x, y, glos_width, glos_height);

    f.render_widget(Clear, glos_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Metric Glossary  [?] close ");

    let inner = block.inner(glos_area);
    f.render_widget(block, glos_area);

    let mut lines: Vec<Line> = Vec::new();
    for (term, definition) in GLOSSARY_ENTRIES {
        lines.push(Line::from(Span::styled(
            *term,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(format!("  {}", definition)));
    }

    let glos_para = Paragraph::new(lines);
    f.render_widget(glos_para, inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossary_has_jaccard_entry() {
        let has_jaccard = GLOSSARY_ENTRIES
            .iter()
            .any(|(term, _)| term.contains("Jaccard"));
        assert!(has_jaccard, "glossary must have a Jaccard entry");
    }

    #[test]
    fn test_glossary_has_polsby_popper_entry() {
        let has_pp = GLOSSARY_ENTRIES
            .iter()
            .any(|(term, _)| term.contains("PP") || term.contains("Polsby-Popper") || term.contains("Polsby"));
        assert!(has_pp, "glossary must have a Polsby-Popper entry");
    }
}
