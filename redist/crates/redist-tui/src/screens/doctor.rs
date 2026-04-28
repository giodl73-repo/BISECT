/// Doctor screen — live policy checks for a location/chamber/year.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{CheckStatus, DoctorCheck, DoctorState};

/// Run all doctor checks against the LocationRegistry and return the list.
pub fn run_checks(location: &str, chamber: &str, year: &str) -> Vec<DoctorCheck> {
    let registry = redist_cli::policy::LocationRegistry::load();
    let mut checks = Vec::new();

    // 1. has_location
    if registry.has_location(location) {
        let name = registry.state_name(location)
            .unwrap_or_else(|| location.to_string());
        checks.push(DoctorCheck {
            status: CheckStatus::Pass,
            message: format!("Location '{location}' found: {name}"),
            hint: None,
        });
    } else {
        checks.push(DoctorCheck {
            status: CheckStatus::Fail,
            message: format!("Location '{location}' not found in policy database"),
            hint: Some("Check REDIST_LOCATION_POLICY or add the location to location_policy.json".to_string()),
        });
        // Early return — remaining checks are meaningless without a valid location
        return checks;
    }

    // 2. validate_year
    match registry.validate_year(location, year) {
        Ok(_) => {
            checks.push(DoctorCheck {
                status: CheckStatus::Pass,
                message: format!("Year '{year}' is available for '{location}'"),
                hint: None,
            });
        }
        Err(msg) => {
            checks.push(DoctorCheck {
                status: CheckStatus::Fail,
                message: msg.clone(),
                hint: Some(format!("Available years: {}", registry.available_years(location).join(", "))),
            });
        }
    }

    // 3. chamber_districts
    match registry.chamber_districts(location, chamber, year) {
        Some(0) => {
            checks.push(DoctorCheck {
                status: CheckStatus::Warn,
                message: format!("Chamber '{chamber}' has 0 districts (unicameral or no chamber?)"),
                hint: Some("Use --chamber house for unicameral legislatures (e.g. NE)".to_string()),
            });
        }
        Some(n) => {
            checks.push(DoctorCheck {
                status: CheckStatus::Pass,
                message: format!("Chamber '{chamber}' has {n} districts"),
                hint: None,
            });
        }
        None => {
            checks.push(DoctorCheck {
                status: CheckStatus::Warn,
                message: format!("Chamber '{chamber}' district count not found for '{location}'"),
                hint: Some("Try 'congressional', 'house', or 'senate'".to_string()),
            });
        }
    }

    // 4. chamber_balance_tolerance
    let tol = redist_cli::runner::chamber_balance_tolerance(location, chamber);
    checks.push(DoctorCheck {
        status: CheckStatus::Info,
        message: format!("Balance tolerance for '{chamber}': {:.1}%", tol * 100.0),
        hint: None,
    });

    // 5. granularity_warning
    match registry.granularity_warning(location, year, chamber, "tract") {
        Some(warn) => {
            checks.push(DoctorCheck {
                status: CheckStatus::Warn,
                message: warn,
                hint: Some("Use --resolution block_group for better balance".to_string()),
            });
        }
        None => {
            checks.push(DoctorCheck {
                status: CheckStatus::Pass,
                message: "Tract resolution is sufficient for this chamber".to_string(),
                hint: None,
            });
        }
    }

    // 6. compactness_standard from raw()
    let raw = registry.raw();
    let location_upper = location.to_uppercase();
    if let Some(entry) = raw.get(&location_upper).or_else(|| raw.get(location)) {
        if let Some(cs) = entry.get("compactness_standard").and_then(|v| v.as_str()) {
            checks.push(DoctorCheck {
                status: CheckStatus::Info,
                message: format!("Compactness standard: {cs}"),
                hint: None,
            });
        }

        // 7. nesting from raw()
        if let Some(nr) = entry.get("nesting_requirement").and_then(|v| v.as_str()) {
            let ratio = entry.get("nesting_ratio").and_then(|v| v.as_str()).unwrap_or("?");
            checks.push(DoctorCheck {
                status: CheckStatus::Info,
                message: format!("Nesting requirement: {nr} ({ratio})"),
                hint: None,
            });
        }
    }

    checks
}

pub fn render(f: &mut Frame, area: Rect, state: &DoctorState) {
    let title = format!(
        " Doctor  {}  {}  {} ",
        state.location, state.chamber, state.year
    );
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Run checks live during render
    let checks = run_checks(&state.location, &state.chamber, &state.year);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

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
    f.render_widget(checks_para, rows[0]);

    // Footer
    let footer = Paragraph::new("[r] Run with these settings  [Esc] back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, rows[1]);
}
