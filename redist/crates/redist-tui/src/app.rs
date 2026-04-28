// App state — all shared state for the TUI, screen routing, navigation history.

// ── Screens ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Home,
    Run(RunState),
    Compare(CompareState),
    Verify(VerifyState),
    Doctor(DoctorState),
}

// ── Run screen state ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunState {
    pub form: RunForm,
    pub phase: RunPhase,
    pub progress: RunProgress,
    pub log_lines: Vec<String>,
    pub show_full_log: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunForm {
    pub location: String,
    pub chamber: String,
    pub year: String,
    pub resolution: String,
    pub seed: String,
    pub label: String,
    pub version: String,
    pub balance_tol: String,
    pub focused_field: usize,
    pub doctor_warnings: Vec<String>,
    pub doctor_errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum RunPhase {
    #[default]
    Form,
    Running,
    Complete(RunResult),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunProgress {
    /// (completed, total) per bisection depth
    pub depths: Vec<(usize, usize)>,
    pub districts_assigned: usize,
    pub districts_total: usize,
    pub elapsed_secs: u64,
    pub balance_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunResult {
    pub success: bool,
    pub elapsed_secs: u64,
    pub mean_pp: Option<f64>,
    pub max_deviation_pct: Option<f64>,
    pub county_splits: Option<usize>,
    pub all_contiguous: Option<bool>,
    pub error: Option<String>,
}

// ── Compare screen state ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompareState {
    pub plan_a: String,
    pub plan_b_input: String,
    pub result: Option<CompareResult>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompareResult {
    pub jaccard: f64,
    pub mean_pp_a: f64,
    pub mean_pp_b: f64,
    pub max_dev_a: f64,
    pub max_dev_b: f64,
    pub splits_a: usize,
    pub splits_b: usize,
    pub contiguous_a: bool,
    pub contiguous_b: bool,
    pub most_changed: Vec<(usize, f64)>,  // (district_id, pct_moved)
}

// ── Verify screen state ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VerifyState {
    pub manifest_path: String,
    pub result: Option<VerifyResult>,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VerifyResult {
    pub passed: bool,
    pub jaccard: f64,
    pub label: String,
    pub state_code: String,
    pub year: String,
    pub binary_match: bool,
    pub metis_version: String,
    pub adjacency_match: bool,
    pub tiger_match: bool,
    pub seed_recorded: bool,
    pub fail_reason: Option<String>,
    pub likely_causes: Vec<String>,
}

// ── Doctor screen state ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DoctorState {
    pub location: String,
    pub chamber: String,
    pub year: String,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoctorCheck {
    pub status: CheckStatus,
    pub message: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Info,
}

// ── Supporting types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlanSummary {
    pub label: String,
    pub state_code: String,
    pub state_name: String,
    pub chamber: String,
    pub year: String,
    pub num_districts: usize,
    pub mean_pp: Option<f64>,
    pub max_deviation_pct: Option<f64>,
    pub county_splits: Option<usize>,
    pub all_contiguous: Option<bool>,
    pub plan_dir: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SortColumn {
    #[default]
    Label,
    Splits,
    Pp,
    Deviation,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub summary: String,
    pub raw: Option<String>,
    pub show_raw: bool,
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    pub screen: Screen,
    pub screen_history: Vec<Screen>,
    pub plans: Vec<PlanSummary>,
    pub selected_plan: usize,
    pub filter: String,
    pub sort: SortColumn,
    pub sort_dir: SortDir,
    pub error: Option<AppError>,
    pub show_palette: bool,
    pub palette_input: String,
    pub palette_history: Vec<String>,
    pub show_glossary: bool,
    pub show_policy_panel: bool,
    pub no_session: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Home,
            screen_history: Vec::new(),
            plans: Vec::new(),
            selected_plan: 0,
            filter: String::new(),
            sort: SortColumn::Label,
            sort_dir: SortDir::Asc,
            error: None,
            show_palette: false,
            palette_input: String::new(),
            palette_history: Vec::new(),
            show_glossary: false,
            show_policy_panel: false,
            no_session: false,
        }
    }
}

impl App {
    pub fn navigate(&mut self, screen: Screen) {
        let prev = std::mem::replace(&mut self.screen, screen);
        self.screen_history.push(prev);
    }

    pub fn navigate_back(&mut self) {
        if let Some(prev) = self.screen_history.pop() {
            self.screen = prev;
        } else {
            self.screen = Screen::Home;
        }
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error = Some(AppError {
            summary: msg.into(),
            raw: None,
            show_raw: false,
        });
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Filtered + sorted plans for display.
    pub fn visible_plans(&self) -> Vec<&PlanSummary> {
        let mut plans: Vec<&PlanSummary> = self.plans.iter()
            .filter(|p| {
                if self.filter.is_empty() { return true; }
                let f = self.filter.to_lowercase();
                p.label.to_lowercase().contains(&f)
                    || p.state_code.to_lowercase().contains(&f)
                    || p.chamber.to_lowercase().contains(&f)
            })
            .collect();

        match (&self.sort, &self.sort_dir) {
            (SortColumn::Label, SortDir::Asc) => plans.sort_by(|a, b| a.label.cmp(&b.label)),
            (SortColumn::Label, SortDir::Desc) => plans.sort_by(|a, b| b.label.cmp(&a.label)),
            (SortColumn::Splits, SortDir::Asc) => plans.sort_by_key(|p| p.county_splits.unwrap_or(usize::MAX)),
            (SortColumn::Splits, SortDir::Desc) => plans.sort_by_key(|p| std::cmp::Reverse(p.county_splits.unwrap_or(0))),
            (SortColumn::Pp, SortDir::Asc) => plans.sort_by(|a, b| a.mean_pp.partial_cmp(&b.mean_pp).unwrap_or(std::cmp::Ordering::Equal)),
            (SortColumn::Pp, SortDir::Desc) => plans.sort_by(|a, b| b.mean_pp.partial_cmp(&a.mean_pp).unwrap_or(std::cmp::Ordering::Equal)),
            (SortColumn::Deviation, SortDir::Asc) => plans.sort_by(|a, b| a.max_deviation_pct.partial_cmp(&b.max_deviation_pct).unwrap_or(std::cmp::Ordering::Equal)),
            (SortColumn::Deviation, SortDir::Desc) => plans.sort_by(|a, b| b.max_deviation_pct.partial_cmp(&a.max_deviation_pct).unwrap_or(std::cmp::Ordering::Equal)),
        }
        plans
    }

    /// Cycle to next sort column/direction.
    pub fn cycle_sort(&mut self) {
        self.sort = match (&self.sort, &self.sort_dir) {
            (SortColumn::Label, SortDir::Asc) => { self.sort_dir = SortDir::Desc; SortColumn::Label }
            (SortColumn::Label, SortDir::Desc) => { self.sort_dir = SortDir::Asc; SortColumn::Splits }
            (SortColumn::Splits, SortDir::Asc) => { self.sort_dir = SortDir::Desc; SortColumn::Splits }
            (SortColumn::Splits, SortDir::Desc) => { self.sort_dir = SortDir::Asc; SortColumn::Pp }
            (SortColumn::Pp, SortDir::Asc) => { self.sort_dir = SortDir::Desc; SortColumn::Pp }
            (SortColumn::Pp, SortDir::Desc) => { self.sort_dir = SortDir::Asc; SortColumn::Deviation }
            _ => { self.sort_dir = SortDir::Asc; SortColumn::Label }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_starts_on_home_screen() {
        let app = App::default();
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn test_navigate_to_run_screen() {
        let mut app = App::default();
        app.navigate(Screen::Run(RunState::default()));
        assert!(matches!(app.screen, Screen::Run(_)));
    }

    #[test]
    fn test_navigate_back_returns_to_home() {
        let mut app = App::default();
        app.navigate(Screen::Run(RunState::default()));
        app.navigate_back();
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn test_navigate_back_from_home_stays_home() {
        let mut app = App::default();
        app.navigate_back();
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn test_set_and_clear_error() {
        let mut app = App::default();
        app.set_error("something went wrong");
        assert!(app.error.is_some());
        app.clear_error();
        assert!(app.error.is_none());
    }

    #[test]
    fn test_visible_plans_filter_by_state_code() {
        let mut app = App::default();
        app.plans = vec![
            PlanSummary { label: "wa_house".into(), state_code: "WA".into(), ..Default::default() },
            PlanSummary { label: "ca_cong".into(), state_code: "CA".into(), ..Default::default() },
        ];
        app.filter = "WA".to_string();
        let visible = app.visible_plans();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].state_code, "WA");
    }

    #[test]
    fn test_cycle_sort_changes_column() {
        let mut app = App::default();
        assert!(matches!(app.sort, SortColumn::Label));
        app.cycle_sort(); // Label Asc -> Label Desc
        assert!(matches!(app.sort, SortColumn::Label));
        assert!(matches!(app.sort_dir, SortDir::Desc));
        app.cycle_sort(); // Label Desc -> Splits Asc
        assert!(matches!(app.sort, SortColumn::Splits));
    }
}
