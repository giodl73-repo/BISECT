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
    pub active_field: usize,
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

pub use bisect_report::tui_compare::CompareResult;

// ── Verify screen state ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VerifyState {
    pub manifest_path: String,
    pub result: Option<VerifyResult>,
    pub running: bool,
}

pub use bisect_report::tui_verify::VerifyResult;

// ── Doctor screen state ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DoctorState {
    pub location: String,
    pub chamber: String,
    pub year: String,
    pub checks: Vec<DoctorCheck>,
}

pub use bisect_report::tui_doctor::{DoctorCheck,CheckStatus};

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

pub use bisect_report::tui_library::{SortColumn,SortDir};
impl bisect_report::tui_library::PlanOrdering for PlanSummary {
 fn label(&self)->&str{&self.label}fn state_code(&self)->&str{&self.state_code}fn chamber(&self)->&str{&self.chamber}
 fn mean_pp(&self)->Option<f64>{self.mean_pp}fn max_deviation_pct(&self)->Option<f64>{self.max_deviation_pct}fn county_splits(&self)->Option<u64>{self.county_splits.map(|n|n as u64)}
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
    /// Default RunForm values pre-populated from session config.
    pub default_run_form: RunForm,
    /// Wall-clock start time for elapsed display (set when phase transitions to Running).
    pub run_started_at: Option<std::time::Instant>,
    /// Shared log lines from background subprocess during a run.
    pub subprocess_log: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    /// None = still running, Some(true) = success, Some(false) = failed.
    pub subprocess_done: std::sync::Arc<std::sync::Mutex<Option<bool>>>,
    /// Label passed via --label at startup (pre-scopes the plan list).
    pub startup_label: Option<String>,
    /// When true, the TUI was launched with --configure (opens compositor wizard on start).
    pub startup_configure: bool,
    /// Optional config path passed via --config at startup.
    pub startup_config: Option<String>,
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
            default_run_form: RunForm::default(),
            run_started_at: None,
            subprocess_log: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            subprocess_done: std::sync::Arc::new(std::sync::Mutex::new(None)),
            startup_label: None,
            startup_configure: false,
            startup_config: None,
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
        bisect_report::tui_library::visible_plans(&self.plans,&self.filter,&self.sort,&self.sort_dir)
    }

    /// Cycle to next sort column/direction.
    pub fn cycle_sort(&mut self) {
        self.sort = match (&self.sort, &self.sort_dir) {
            (SortColumn::Label, SortDir::Asc) => {
                self.sort_dir = SortDir::Desc;
                SortColumn::Label
            }
            (SortColumn::Label, SortDir::Desc) => {
                self.sort_dir = SortDir::Asc;
                SortColumn::Splits
            }
            (SortColumn::Splits, SortDir::Asc) => {
                self.sort_dir = SortDir::Desc;
                SortColumn::Splits
            }
            (SortColumn::Splits, SortDir::Desc) => {
                self.sort_dir = SortDir::Asc;
                SortColumn::Pp
            }
            (SortColumn::Pp, SortDir::Asc) => {
                self.sort_dir = SortDir::Desc;
                SortColumn::Pp
            }
            (SortColumn::Pp, SortDir::Desc) => {
                self.sort_dir = SortDir::Asc;
                SortColumn::Deviation
            }
            _ => {
                self.sort_dir = SortDir::Asc;
                SortColumn::Label
            }
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
            PlanSummary {
                label: "wa_house".into(),
                state_code: "WA".into(),
                ..Default::default()
            },
            PlanSummary {
                label: "ca_cong".into(),
                state_code: "CA".into(),
                ..Default::default()
            },
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

    // ── Additional app tests ──────────────────────────────────────────────────

    #[test]
    fn test_navigate_pushes_to_history() {
        let mut app = App::default();
        app.navigate(Screen::Run(RunState::default()));
        assert_eq!(app.screen_history.len(), 1);
        assert!(matches!(app.screen_history[0], Screen::Home));
    }

    #[test]
    fn test_navigate_back_multi_level() {
        let mut app = App::default();
        app.navigate(Screen::Run(RunState::default()));
        app.navigate(Screen::Doctor(DoctorState::default()));
        app.navigate_back(); // back to Run
        assert!(matches!(app.screen, Screen::Run(_)));
        app.navigate_back(); // back to Home
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn test_set_error_stores_summary() {
        let mut app = App::default();
        app.set_error("test error message");
        let err = app.error.as_ref().unwrap();
        assert_eq!(err.summary, "test error message");
        assert!(err.raw.is_none());
        assert!(!err.show_raw);
    }

    #[test]
    fn test_visible_plans_empty_filter_shows_all() {
        let mut app = App::default();
        app.plans = vec![
            PlanSummary {
                label: "wa_house".into(),
                state_code: "WA".into(),
                ..Default::default()
            },
            PlanSummary {
                label: "ca_cong".into(),
                state_code: "CA".into(),
                ..Default::default()
            },
            PlanSummary {
                label: "tx_senate".into(),
                state_code: "TX".into(),
                ..Default::default()
            },
        ];
        app.filter = String::new();
        assert_eq!(app.visible_plans().len(), 3);
    }

    #[test]
    fn test_visible_plans_filter_case_insensitive() {
        let mut app = App::default();
        app.plans = vec![
            PlanSummary {
                label: "WA_house".into(),
                state_code: "WA".into(),
                ..Default::default()
            },
            PlanSummary {
                label: "ca_cong".into(),
                state_code: "CA".into(),
                ..Default::default()
            },
        ];
        app.filter = "wa".to_string();
        let visible = app.visible_plans();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].state_code, "WA");
    }

    #[test]
    fn test_visible_plans_filter_by_chamber() {
        let mut app = App::default();
        app.plans = vec![
            PlanSummary {
                label: "wa_house".into(),
                state_code: "WA".into(),
                chamber: "house".into(),
                ..Default::default()
            },
            PlanSummary {
                label: "wa_senate".into(),
                state_code: "WA".into(),
                chamber: "senate".into(),
                ..Default::default()
            },
        ];
        app.filter = "senate".to_string();
        let visible = app.visible_plans();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].chamber, "senate");
    }

    #[test]
    fn test_visible_plans_sort_by_splits_asc() {
        let mut app = App::default();
        app.plans = vec![
            PlanSummary {
                label: "b".into(),
                county_splits: Some(10),
                ..Default::default()
            },
            PlanSummary {
                label: "a".into(),
                county_splits: Some(2),
                ..Default::default()
            },
            PlanSummary {
                label: "c".into(),
                county_splits: Some(5),
                ..Default::default()
            },
        ];
        app.sort = SortColumn::Splits;
        app.sort_dir = SortDir::Asc;
        let visible = app.visible_plans();
        assert_eq!(visible[0].county_splits, Some(2));
        assert_eq!(visible[1].county_splits, Some(5));
        assert_eq!(visible[2].county_splits, Some(10));
    }

    #[test]
    fn test_visible_plans_sort_by_splits_desc() {
        let mut app = App::default();
        app.plans = vec![
            PlanSummary {
                label: "b".into(),
                county_splits: Some(10),
                ..Default::default()
            },
            PlanSummary {
                label: "a".into(),
                county_splits: Some(2),
                ..Default::default()
            },
        ];
        app.sort = SortColumn::Splits;
        app.sort_dir = SortDir::Desc;
        let visible = app.visible_plans();
        assert_eq!(visible[0].county_splits, Some(10));
    }

    #[test]
    fn test_visible_plans_sort_by_pp_asc() {
        let mut app = App::default();
        app.plans = vec![
            PlanSummary {
                label: "b".into(),
                mean_pp: Some(0.9),
                ..Default::default()
            },
            PlanSummary {
                label: "a".into(),
                mean_pp: Some(0.1),
                ..Default::default()
            },
        ];
        app.sort = SortColumn::Pp;
        app.sort_dir = SortDir::Asc;
        let visible = app.visible_plans();
        assert!((visible[0].mean_pp.unwrap() - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_visible_plans_sort_by_deviation_desc() {
        let mut app = App::default();
        app.plans = vec![
            PlanSummary {
                label: "a".into(),
                max_deviation_pct: Some(0.5),
                ..Default::default()
            },
            PlanSummary {
                label: "b".into(),
                max_deviation_pct: Some(2.0),
                ..Default::default()
            },
        ];
        app.sort = SortColumn::Deviation;
        app.sort_dir = SortDir::Desc;
        let visible = app.visible_plans();
        assert!((visible[0].max_deviation_pct.unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_cycle_sort_full_cycle_returns_to_label_asc() {
        let mut app = App::default();
        // Label Asc(0) → Label Desc(1) → Splits Asc(2) → Splits Desc(3)
        // → Pp Asc(4) → Pp Desc(5) → Deviation Asc(6) → Label Asc(7)
        for _ in 0..7 {
            app.cycle_sort();
        }
        // After 7 cycles we should be back at Label Asc
        assert!(matches!(app.sort, SortColumn::Label));
        assert!(matches!(app.sort_dir, SortDir::Asc));
    }

    #[test]
    fn test_run_progress_default_values() {
        let p = RunProgress::default();
        assert_eq!(p.districts_assigned, 0);
        assert_eq!(p.districts_total, 0);
        assert!(!p.balance_ok);
        assert_eq!(p.elapsed_secs, 0);
        assert!(p.depths.is_empty());
    }

    #[test]
    fn test_run_result_default_values() {
        let r = RunResult::default();
        assert!(!r.success);
        assert!(r.mean_pp.is_none());
        assert!(r.error.is_none());
    }

    #[test]
    fn test_check_status_variants_debug() {
        // Ensure all CheckStatus variants can be formatted without panic
        let statuses = [
            CheckStatus::Pass,
            CheckStatus::Warn,
            CheckStatus::Fail,
            CheckStatus::Info,
        ];
        for s in &statuses {
            let _ = format!("{:?}", s);
        }
    }

    #[test]
    fn test_doctor_check_message_stored() {
        let check = DoctorCheck {
            status: CheckStatus::Warn,
            message: "adjacency file missing".into(),
            hint: Some("run bisect fetch".into()),
        };
        assert_eq!(check.message, "adjacency file missing");
        assert_eq!(check.hint.as_deref(), Some("run bisect fetch"));
    }

    #[test]
    fn test_compare_result_most_changed_empty_default() {
        let r = CompareResult::default();
        assert!(r.most_changed.is_empty());
        assert_eq!(r.jaccard, 0.0);
    }

    #[test]
    fn test_verify_result_default_not_passed() {
        let r = VerifyResult::default();
        assert!(!r.passed);
        assert!(r.fail_reason.is_none());
    }

    #[test]
    fn test_app_error_no_raw_by_default() {
        let mut app = App::default();
        app.set_error("oops");
        assert!(app.error.as_ref().unwrap().raw.is_none());
    }

    #[test]
    fn test_plan_summary_default_contiguous_is_none() {
        let p = PlanSummary::default();
        assert!(p.all_contiguous.is_none());
    }
}
