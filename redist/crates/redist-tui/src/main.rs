use std::io;
use std::io::Stdout;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

use redist_tui::{plans, session};
use redist_tui::app::{App, DoctorState, RunPhase, RunProgress, RunResult, RunState, Screen};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // --version / --help (no terminal setup needed)
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("redist-tui {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Usage: redist-tui [--no-session] [--label LABEL] [--configure] [--config PATH] [--version]");
        println!("  --no-session        Start with clean state (no saved config)");
        println!("  --label LABEL       Pre-scope to a specific label (skips label picker)");
        println!("  --configure         Start directly in the compositor wizard");
        println!("  --config PATH       Path to algorithm config YAML (used with --configure)");
        return Ok(());
    }

    // Parse --no-session flag
    let no_session = args.iter().any(|a| a == "--no-session");

    // Parse --label LABEL (next arg after --label)
    let startup_label: Option<String> = args.windows(2)
        .find(|w| w[0] == "--label")
        .map(|w| w[1].clone());

    // Parse --configure flag
    let startup_configure = args.iter().any(|a| a == "--configure");

    // Parse --config PATH (next arg after --config)
    let startup_config: Option<String> = args.windows(2)
        .find(|w| w[0] == "--config")
        .map(|w| w[1].clone());

    // Install panic hook to restore terminal state on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Always try to restore terminal — ignore errors
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stderr(),
            crossterm::terminal::LeaveAlternateScreen
        );
        original_hook(panic_info);
    }));

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App initialisation
    let mut app = App::default();
    app.no_session = no_session;

    // Store startup flags for downstream use
    app.startup_label = startup_label.clone();
    app.startup_configure = startup_configure;
    app.startup_config = startup_config;

    // Load session (applies saved location/sort preferences)
    let sess = if !no_session {
        session::load_session()
    } else {
        session::Session::default()
    };

    // Discover plans using session settings
    app.plans = plans::discover_plans(&sess.output_base, &sess.version, &sess.year);

    // Pre-populate run form from session
    app.default_run_form = redist_tui::app::RunForm {
        location: sess.location.clone(),
        chamber: sess.chamber.clone(),
        year: sess.year.clone(),
        resolution: sess.resolution.clone(),
        version: sess.version.clone(),
        ..Default::default()
    };

    // If --label was given, pre-select that label in the plan list
    if let Some(ref lbl) = startup_label {
        if let Some(idx) = app.plans.iter().position(|p| p.label == *lbl) {
            app.selected_plan = idx;
        }
    }

    // Run main event loop
    let result = run_app(&mut terminal, &mut app, &sess);

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    _sess: &session::Session,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| {
            let area = f.area();

            // Reserve bottom 1 line for status bar
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(area);

            let main_area = layout[0];
            let status_area = layout[1];

            // Render current screen
            match &app.screen {
                Screen::Home => {
                    redist_tui::screens::home::render(f, main_area, app);
                }
                Screen::Run(state) => {
                    let state = state.clone();
                    redist_tui::screens::run::render(f, main_area, &state);
                }
                Screen::Compare(state) => {
                    let state = state.clone();
                    redist_tui::screens::compare::render(f, main_area, &state);
                }
                Screen::Verify(state) => {
                    let state = state.clone();
                    redist_tui::screens::verify::render(f, main_area, &state);
                }
                Screen::Doctor(state) => {
                    let state = state.clone();
                    redist_tui::screens::doctor::render(f, main_area, &state);
                }
            }

            // Always-visible status bar
            redist_tui::widgets::status_bar::render(f, status_area, app);

            // Overlays (rendered on top in priority order)
            if app.show_glossary {
                redist_tui::widgets::glossary::render(f, area, app);
            }
            if app.show_palette {
                redist_tui::widgets::command_palette::render(f, area, app);
            }
            if app.error.is_some() {
                redist_tui::widgets::error_banner::render(f, area, app);
            }
        })?;

        // Drain subprocess log into RunState each tick
        if let Screen::Run(ref mut run_state) = app.screen {
            if matches!(run_state.phase, RunPhase::Running) {
                // Drain log lines
                {
                    let mut log = app.subprocess_log.lock().unwrap();
                    for line in log.drain(..) {
                        // Parse STATUS: lines for progress
                        if let Some(msg) = redist_tui::runner::parse_status_line(&line) {
                            redist_tui::runner::update_progress_from_message(&mut run_state.progress, &msg);
                            run_state.log_lines.push(msg);
                        } else {
                            run_state.log_lines.push(line);
                        }
                        // Keep only last 50 log lines
                        if run_state.log_lines.len() > 50 {
                            run_state.log_lines.drain(0..run_state.log_lines.len() - 50);
                        }
                    }
                }
                // Update elapsed time from wall clock (fixes saturating_add(0) bug)
                if let Some(started) = app.run_started_at {
                    run_state.progress.elapsed_secs = started.elapsed().as_secs();
                }

                // Check if subprocess finished
                let done = *app.subprocess_done.lock().unwrap();
                if let Some(success) = done {
                    let elapsed = run_state.progress.elapsed_secs;
                    run_state.phase = RunPhase::Complete(RunResult {
                        success,
                        elapsed_secs: elapsed,
                        ..Default::default()
                    });
                    // Clear start time and done flag
                    app.run_started_at = None;
                    *app.subprocess_done.lock().unwrap() = None;
                }
            }
        }

        // Poll for events with 50ms timeout
        if !event::poll(Duration::from_millis(50))? {
            continue;
        }

        let evt = event::read()?;

        // Global overlays first
        if app.show_palette {
            if let Event::Key(key) = evt {
                match key.code {
                    KeyCode::Esc => {
                        app.show_palette = false;
                        app.palette_input.clear();
                    }
                    KeyCode::Backspace => {
                        app.palette_input.pop();
                    }
                    KeyCode::Enter => {
                        // Execute command
                        let cmd = app.palette_input.clone();
                        app.show_palette = false;
                        app.palette_input.clear();
                        execute_palette_command(app, &cmd);
                    }
                    KeyCode::Char(c) => {
                        app.palette_input.push(c);
                    }
                    _ => {}
                }
            }
            continue;
        }

        if app.show_glossary {
            if let Event::Key(_) = evt {
                app.show_glossary = false;
            }
            continue;
        }

        if app.error.is_some() {
            if let Event::Key(key) = evt {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('c') => {
                        app.clear_error();
                    }
                    _ => {}
                }
            }
            continue;
        }

        // Screen-specific key dispatch
        if let Event::Key(key) = evt {
            // Run form editing: intercept Tab/BackTab/Backspace/Char before generic dispatch
            let is_run_form = matches!(&app.screen, Screen::Run(s) if matches!(s.phase, RunPhase::Form));
            if is_run_form {
                match key.code {
                    KeyCode::Esc => {
                        app.navigate_back();
                        continue;
                    }
                    KeyCode::Enter => {
                        // Fall through to Enter handling below
                    }
                    KeyCode::Tab => {
                        if let Screen::Run(ref mut s) = app.screen {
                            s.form.active_field = (s.form.active_field + 1) % 8;
                        }
                        continue;
                    }
                    KeyCode::BackTab => {
                        if let Screen::Run(ref mut s) = app.screen {
                            s.form.active_field = s.form.active_field.saturating_sub(1);
                        }
                        continue;
                    }
                    KeyCode::Backspace => {
                        if let Screen::Run(ref mut s) = app.screen {
                            edit_active_field(&mut s.form, |f| { f.pop(); });
                        }
                        continue;
                    }
                    KeyCode::Char(c) => {
                        if let Screen::Run(ref mut s) = app.screen {
                            edit_active_field(&mut s.form, |f| { f.push(c); });
                        }
                        continue;
                    }
                    _ => {
                        continue;
                    }
                }
            }

            match key.code {
                KeyCode::Enter => {
                    // Run screen: Enter on form → spawn subprocess
                    if let Screen::Run(ref mut state) = app.screen {
                        if matches!(state.phase, RunPhase::Form) {
                            // Reset shared state
                            *app.subprocess_log.lock().unwrap() = Vec::new();
                            *app.subprocess_done.lock().unwrap() = None;

                            // Transition to Running phase
                            state.phase = RunPhase::Running;
                            state.log_lines.clear();
                            state.progress = RunProgress {
                                districts_total: state.form.label.parse().unwrap_or(1),
                                ..Default::default()
                            };
                            // Record wall-clock start time for elapsed tracking
                            app.run_started_at = Some(std::time::Instant::now());

                            // Spawn subprocess
                            let form_clone = state.form.clone();
                            let log_arc = app.subprocess_log.clone();
                            let done_arc = app.subprocess_done.clone();
                            redist_tui::runner::spawn_redist_state(&form_clone, log_arc, done_arc);
                        }
                    }
                    // Verify screen: Enter with non-empty path → run verify
                    if let Screen::Verify(ref mut state) = app.screen {
                        if !state.manifest_path.is_empty() && !state.running {
                            state.running = true;
                            state.result = None;
                            let path = state.manifest_path.clone();
                            let result = redist_tui::runner::run_verify_subprocess(&path);
                            state.result = Some(result);
                            state.running = false;
                        }
                    }
                }
                _ => {
                    match &app.screen {
                        Screen::Home => {
                            match (key.code, key.modifiers) {
                                (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => return Ok(()),
                                (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                                (KeyCode::Char('r'), _) => {
                                    // Pre-fill from session defaults
                                    let form = app.default_run_form.clone();
                                    app.navigate(Screen::Run(RunState {
                                        form,
                                        ..Default::default()
                                    }));
                                }
                                (KeyCode::Char('c'), _) => {
                                    use redist_tui::app::CompareState;
                                    let plan_a = app
                                        .visible_plans()
                                        .get(app.selected_plan)
                                        .map(|p| p.label.clone())
                                        .unwrap_or_default();
                                    app.navigate(Screen::Compare(CompareState {
                                        plan_a,
                                        ..Default::default()
                                    }));
                                }
                                (KeyCode::Char('v'), _) => {
                                    use redist_tui::app::VerifyState;
                                    app.navigate(Screen::Verify(VerifyState::default()));
                                }
                                (KeyCode::Char('d'), _) => {
                                    app.navigate(Screen::Doctor(DoctorState {
                                        location: app.default_run_form.location.clone(),
                                        chamber: app.default_run_form.chamber.clone(),
                                        year: "2020".to_string(),
                                        checks: Vec::new(),
                                    }));
                                }
                                (KeyCode::Up, _) => {
                                    if app.selected_plan > 0 {
                                        app.selected_plan -= 1;
                                    }
                                }
                                (KeyCode::Down, _) => {
                                    let count = app.visible_plans().len();
                                    if count > 0 && app.selected_plan + 1 < count {
                                        app.selected_plan += 1;
                                    }
                                }
                                (KeyCode::Char('s'), _) => {
                                    app.cycle_sort();
                                }
                                (KeyCode::Char(':'), _) => {
                                    app.show_palette = true;
                                }
                                (KeyCode::Char('?'), _) => {
                                    app.show_glossary = !app.show_glossary;
                                }
                                _ => {}
                            }
                        }
                        Screen::Run(_) => {
                            match key.code {
                                KeyCode::Esc => app.navigate_back(),
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char(':') => app.show_palette = true,
                                KeyCode::Char('?') => app.show_glossary = !app.show_glossary,
                                _ => {}
                            }
                        }
                        Screen::Compare(_) => {
                            match key.code {
                                KeyCode::Esc => app.navigate_back(),
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char(':') => app.show_palette = true,
                                KeyCode::Char('?') => app.show_glossary = !app.show_glossary,
                                KeyCode::Backspace => {
                                    if let Screen::Compare(ref mut s) = app.screen {
                                        s.plan_b_input.pop();
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if let Screen::Compare(ref mut s) = app.screen {
                                        s.plan_b_input.push(c);
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Screen::Compare(ref mut s) = app.screen {
                                        if !s.plan_b_input.is_empty() {
                                            let base = std::path::PathBuf::from("outputs")
                                                .join("v1").join("2020").join("plans");
                                            let plan_a_dir = base.join(&s.plan_a);
                                            let plan_b_dir = base.join(&s.plan_b_input);
                                            if plan_a_dir.exists() && plan_b_dir.exists() {
                                                let label_a = s.plan_a.clone();
                                                let label_b = s.plan_b_input.clone();
                                                let result = redist_tui::screens::compare::compute_compare_result(
                                                    &plan_a_dir, &plan_b_dir, &label_a, &label_b,
                                                );
                                                s.result = Some(result);
                                            } else {
                                                let plan_a = s.plan_a.clone();
                                                let plan_b = s.plan_b_input.clone();
                                                drop(s);
                                                app.set_error(format!(
                                                    "Plan not found: '{}' or '{}'. Check label spelling.",
                                                    plan_a, plan_b
                                                ));
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        Screen::Verify(_) => {
                            // Text input for manifest path
                            match key.code {
                                KeyCode::Esc => app.navigate_back(),
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char(':') => app.show_palette = true,
                                KeyCode::Char('?') => app.show_glossary = !app.show_glossary,
                                KeyCode::Backspace => {
                                    if let Screen::Verify(ref mut state) = app.screen {
                                        state.manifest_path.pop();
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if let Screen::Verify(ref mut state) = app.screen {
                                        state.manifest_path.push(c);
                                    }
                                }
                                _ => {}
                            }
                        }
                        Screen::Doctor(_) => {
                            match key.code {
                                KeyCode::Esc => app.navigate_back(),
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char('r') => {
                                    app.navigate(Screen::Run(RunState::default()));
                                }
                                KeyCode::Char(':') => app.show_palette = true,
                                KeyCode::Char('?') => app.show_glossary = !app.show_glossary,
                                KeyCode::Backspace => {
                                    if let Screen::Doctor(ref mut state) = app.screen {
                                        state.location.pop();
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if let Screen::Doctor(ref mut state) = app.screen {
                                        state.location.push(c.to_uppercase().next().unwrap_or(c));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn edit_active_field(form: &mut redist_tui::app::RunForm, edit: impl FnOnce(&mut String)) {
    match form.active_field {
        0 => edit(&mut form.location),
        1 => edit(&mut form.chamber),
        2 => edit(&mut form.year),
        3 => edit(&mut form.resolution),
        4 => edit(&mut form.seed),
        5 => edit(&mut form.label),
        6 => edit(&mut form.version),
        7 => edit(&mut form.balance_tol),
        _ => {}
    }
}

fn execute_palette_command(app: &mut App, cmd: &str) {
    match cmd.trim() {
        "run" | "r" => {
            app.navigate(Screen::Run(RunState::default()));
        }
        "compare" | "c" => {
            use redist_tui::app::CompareState;
            app.navigate(Screen::Compare(CompareState::default()));
        }
        "verify" | "v" => {
            use redist_tui::app::VerifyState;
            app.navigate(Screen::Verify(VerifyState::default()));
        }
        "doctor" | "d" => {
            app.navigate(Screen::Doctor(DoctorState::default()));
        }
        "quit" | "q" => {
            // Signal quit by navigating to home then... just clear state
            // (actual quit happens via 'q' key; palette can't return from here cleanly)
        }
        _ => {
            app.set_error(format!("Unknown command: '{}'", cmd));
        }
    }
}
