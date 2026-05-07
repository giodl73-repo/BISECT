/// Verify screen — chain-of-custody audit with PASS/FAIL result box.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::VerifyState;

pub fn render(f: &mut Frame, area: Rect, state: &VerifyState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Verify — Chain of Custody ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // manifest path input
            Constraint::Length(5), // PASS/FAIL box
            Constraint::Length(7), // chain of custody table
            Constraint::Min(2),    // likely causes (if FAIL)
            Constraint::Length(1), // footer
        ])
        .split(inner);

    // Manifest path input
    let path_line = ratatui::text::Line::from(vec![
        ratatui::text::Span::styled("Manifest: ", Style::default().fg(Color::Cyan)),
        ratatui::text::Span::raw(&state.manifest_path),
        ratatui::text::Span::styled("_", Style::default().fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(path_line), rows[0]);

    // PASS/FAIL box
    if let Some(result) = &state.result {
        let (pass_color, pass_text, jaccard_text) = if result.passed {
            (
                Color::Green,
                "  PASS  ",
                format!("Jaccard: {:.4}", result.jaccard),
            )
        } else {
            (
                Color::Red,
                "  FAIL  ",
                format!("Jaccard: {:.4}", result.jaccard),
            )
        };

        let label_text = format!("{}  {}  {}", result.label, result.state_code, result.year);

        let pass_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(pass_color))
            .title(format!(" {} ", pass_text.trim()))
            .title_alignment(Alignment::Center);

        let pass_inner = pass_block.inner(rows[1]);
        f.render_widget(pass_block, rows[1]);

        let pass_content = Paragraph::new(vec![
            ratatui::text::Line::from(ratatui::text::Span::styled(
                pass_text,
                Style::default()
                    .fg(pass_color)
                    .add_modifier(Modifier::BOLD),
            )),
            ratatui::text::Line::from(jaccard_text),
            ratatui::text::Line::from(label_text),
        ])
        .alignment(Alignment::Center);
        f.render_widget(pass_content, pass_inner);

        // Chain of custody table
        let checks = [
            ("Binary", result.binary_match),
            ("METIS", !result.metis_version.is_empty()),
            ("Adjacency", result.adjacency_match),
            ("Census geometry", result.tiger_match),
            ("Seed", result.seed_recorded),
        ];

        let header = Row::new(["Item", "Status"].iter().map(|h| {
            Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD))
        }));

        let check_rows: Vec<Row> = checks
            .iter()
            .map(|(name, ok)| {
                let (symbol, color) = if *ok {
                    ("OK", Color::Green)
                } else {
                    ("FAIL", Color::Red)
                };
                Row::new(vec![
                    Cell::from(*name),
                    Cell::from(symbol).style(Style::default().fg(color)),
                ])
            })
            .collect();

        let table = Table::new(
            check_rows,
            [Constraint::Min(20), Constraint::Length(10)],
        )
        .header(header)
        .block(
            Block::default()
                .title(" Chain of Custody ")
                .borders(Borders::TOP),
        );
        f.render_widget(table, rows[2]);

        // Likely causes (only on FAIL)
        if !result.passed && !result.likely_causes.is_empty() {
            let mut lines = vec![ratatui::text::Line::from(ratatui::text::Span::styled(
                "Likely causes:",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))];
            for cause in &result.likely_causes {
                lines.push(ratatui::text::Line::from(format!("  - {}", cause)));
            }
            let causes_para = Paragraph::new(lines);
            f.render_widget(causes_para, rows[3]);
        }
    } else {
        // No result yet
        let placeholder = Paragraph::new(if state.running {
            "Verifying..."
        } else {
            "Enter manifest path and press [Enter] to verify"
        })
        .style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, rows[1]);
    }

    // Footer
    let footer = Paragraph::new("[p] Export PDF  [x] Export audit.json  [Esc] back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, rows[4]);
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
    fn test_verify_pass_shows_pass_text() {
        use crate::app::{VerifyState, VerifyResult};
        let mut state = VerifyState::default();
        state.result = Some(VerifyResult {
            passed: true,
            jaccard: 0.9987,
            label: "wa_house_2020".into(),
            state_code: "WA".into(),
            year: "2020".into(),
            ..Default::default()
        });
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        assert!(content.contains("PASS") || content.contains("pass"), "PASS must appear: {content}");
    }

    #[test]
    fn test_verify_fail_shows_fail_text() {
        use crate::app::{VerifyState, VerifyResult};
        let mut state = VerifyState::default();
        state.result = Some(VerifyResult {
            passed: false,
            jaccard: 0.741,
            ..Default::default()
        });
        let content = render_to_string(120, 30, |f, area| render(f, area, &state));
        assert!(content.contains("FAIL") || content.contains("fail"), "FAIL must appear: {content}");
    }

    #[test]
    fn test_verify_empty_state_shows_input_prompt() {
        let state = VerifyState::default();
        let content = render_to_string(120, 20, |f, area| render(f, area, &state));
        assert!(
            content.contains("Manifest") || content.contains("manifest") || content.contains("path"),
            "manifest input must show: {content}"
        );
    }
}
