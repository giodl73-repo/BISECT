/// Error banner — 4-line Red-bordered overlay at bottom when app.error is set.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let Some(error) = &app.error else {
        return;
    };

    // 4-line banner at the bottom of the given area
    let banner_height = 4u16;
    let y = area.y + area.height.saturating_sub(banner_height);
    let banner_area = Rect::new(area.x, y, area.width, banner_height.min(area.height));

    f.render_widget(Clear, banner_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" Error  [e] full log  [c] copy  [Esc] dismiss ");

    let inner = block.inner(banner_area);
    f.render_widget(block, banner_area);

    let summary_line = Line::from(Span::styled(
        &error.summary,
        Style::default().fg(Color::White),
    ));
    let banner_para = Paragraph::new(summary_line);
    f.render_widget(banner_para, inner);
}
