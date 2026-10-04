use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const STRUCTURES: &[&str] = &[
    "standard-bisect",
    "nway",
    "ratio-optimal",
    "ratio-optimal-area",
    "prime-factor",
    "compact-polsby",
    "bfs-growth",
    "centroidal-voronoi",
    "moving-knife",
    "spectral",
];
pub const WEIGHTS: &[&str] = &["geographic", "unweighted", "county"];
pub const SEARCHES: &[&str] = &[
    "single",
    "multi",
    "percentile",
    "bisection-ensemble",
    "short-burst",
    "short-burst-forest",
    "short-burst-merge-split",
    "flip",
    "forest-recom",
    "merge-split",
    "parallel-tempering",
];

/// Matches the actual dispatch arms in bisect-cli/src/runner/mod.rs.
/// A parsed flag is not evidence that a structure executes that search.
pub fn supported_searches(structure: &str) -> Vec<&'static str> {
    match structure {
        "standard-bisect" => SEARCHES.iter().copied().filter(|s| *s != "multi").collect(),
        "ratio-optimal" | "ratio-optimal-area" | "compact-polsby" => vec!["single", "multi"],
        _ => vec!["single"],
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    pub name: String,
    pub mode: String,
    pub states: Vec<String>,
    pub year: String,
    pub chamber: String,
    pub districts: Option<usize>,
    pub structure: String,
    pub weights: String,
    pub search: String,
    pub seed: u64,
    pub seeds: usize,
    pub steps: usize,
    pub percentile: f64,
    pub alpha_county: f64,
    /// Percent, matching the CLI: 0.5 means half of one percent.
    pub balance_tolerance: f64,
    pub area_swing: f64,
    pub iterations: usize,
    pub timeout_seconds: u64,
}

impl Experiment {
    pub fn validate(&self, codes: &[String]) -> Result<()> {
        ensure!(
            !self.name.trim().is_empty() && self.name.len() <= 100,
            "Give the experiment a name of at most 100 characters."
        );
        ensure!(
            ["state", "national"].contains(&self.mode.as_str()),
            "Unknown experiment mode."
        );
        ensure!(
            !self.states.is_empty() && self.states.len() <= 50,
            "Choose between 1 and 50 states."
        );
        ensure!(
            self.mode != "state" || self.states.len() == 1,
            "State mode requires one state."
        );
        let mut seen = std::collections::HashSet::new();
        for code in &self.states {
            ensure!(
                codes.contains(code) && seen.insert(code),
                "Unknown or duplicate state: {code}"
            );
        }
        ensure!(
            ["2000", "2010", "2020"].contains(&self.year.as_str()),
            "Unsupported census year."
        );
        ensure!(
            ["congressional", "house", "senate"].contains(&self.chamber.as_str()),
            "Unknown chamber."
        );
        ensure!(
            STRUCTURES.contains(&self.structure.as_str()),
            "Unsupported structure."
        );
        ensure!(
            WEIGHTS.contains(&self.weights.as_str()),
            "Unsupported weight signal."
        );
        ensure!(
            SEARCHES.contains(&self.search.as_str()),
            "Unsupported search method."
        );
        ensure!(supported_searches(&self.structure).contains(&self.search.as_str()),
            "This structure does not execute the requested search. Choose a supported search to avoid a silently ignored setting.");
        ensure!(
            (1..=10000).contains(&self.seeds),
            "Seed budget must be 1–10,000."
        );
        ensure!(
            (1..=100000).contains(&self.steps),
            "Step budget must be 1–100,000."
        );
        ensure!(
            self.seed <= i32::MAX as u64,
            "Seed must fit the engine's positive 32-bit range."
        );
        ensure!(
            self.percentile.is_finite() && (0.0..=1.0).contains(&self.percentile),
            "Percentile must be between zero and one."
        );
        ensure!(
            self.alpha_county.is_finite() && (0.0..=100.0).contains(&self.alpha_county),
            "County weight must be 0–100."
        );
        ensure!(
            self.weights == "county" || self.alpha_county == 0.0,
            "County strength requires County weights."
        );
        ensure!(
            self.balance_tolerance.is_finite() && (0.01..=25.0).contains(&self.balance_tolerance),
            "Balance tolerance must be 0.01–25 percent."
        );
        ensure!(
            self.area_swing.is_finite() && (1.01..=2.0).contains(&self.area_swing),
            "Area swing must be 1.01–2.0."
        );
        ensure!(
            (1..=1000).contains(&self.iterations),
            "Refinement iterations must be 1–1,000."
        );
        ensure!(
            (5..=86400).contains(&self.timeout_seconds),
            "State time limit must be 5–86,400 seconds."
        );
        if let Some(k) = self.districts {
            ensure!(
                self.mode == "state" && (1..=500).contains(&k),
                "Custom district count is available in state mode, from 1 to 500."
            );
        }
        Ok(())
    }

    pub fn cli_args(
        &self,
        code: &str,
        k: usize,
        adjacency: &str,
        output: &str,
        label: &str,
    ) -> Vec<String> {
        let pairs = [
            ("state", code.to_owned()),
            ("year", self.year.clone()),
            ("chamber", self.chamber.clone()),
            ("resolution", "tract".into()),
            ("districts", k.to_string()),
            ("adjacency", adjacency.into()),
            ("output-dir", output.into()),
            ("label", label.into()),
            ("structure", self.structure.clone()),
            ("weights-override", self.weights.clone()),
            ("search", self.search.clone()),
            ("seed", self.seed.to_string()),
            ("seeds", self.seeds.to_string()),
            ("convergence-threshold", self.seeds.to_string()),
            ("ensemble-steps", self.steps.to_string()),
            ("flip-steps", self.steps.to_string()),
            ("forest-steps", self.steps.to_string()),
            ("merge-split-steps", self.steps.to_string()),
            ("burst-length", "20".into()),
            ("n-bursts", self.steps.div_ceil(20).to_string()),
            ("percentile", self.percentile.to_string()),
            ("alpha-county", self.alpha_county.to_string()),
            ("balance-tolerance", self.balance_tolerance.to_string()),
            (
                "ufactor",
                ((self.balance_tolerance * 10.0).ceil() as u32)
                    .max(1)
                    .to_string(),
            ),
            ("area-swing", self.area_swing.to_string()),
            ("niter", self.iterations.to_string()),
            ("metis-engine", "c-ffi".into()),
        ];
        let mut args = vec!["state".into()];
        for (flag, value) in pairs {
            args.extend([format!("--{flag}"), value]);
        }
        args.push("--time-partition".into());
        args
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateRun {
    pub code: String,
    pub status: String,
    pub error: Option<String>,
    pub elapsed_seconds: f64,
    pub command: Vec<String>,
    pub metrics: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub config: Experiment,
    pub status: String,
    pub created_unix: u64,
    pub states: Vec<StateRun>,
    pub logs: Vec<String>,
}

#[cfg(test)]
pub fn fixture() -> Experiment {
    Experiment {
        name: "Test".into(),
        mode: "state".into(),
        states: vec!["RI".into()],
        year: "2020".into(),
        chamber: "congressional".into(),
        districts: None,
        structure: "standard-bisect".into(),
        weights: "geographic".into(),
        search: "single".into(),
        seed: 42,
        seeds: 4,
        steps: 100,
        percentile: 0.0,
        alpha_county: 0.0,
        balance_tolerance: 0.5,
        area_swing: 1.1,
        iterations: 20,
        timeout_seconds: 60,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[test]
    fn laboratory_command_is_accepted_by_the_real_cli() {
        let args = fixture().cli_args("RI", 2, "test.adj.bin", "out", "lab_ri");
        let cli =
            bisect_cli::args::Cli::try_parse_from(std::iter::once("bisect".to_owned()).chain(args))
                .unwrap();
        let bisect_cli::args::Commands::State(state) = cli.command else {
            panic!()
        };
        assert_eq!(state.balance_tolerance, Some(0.5));
        assert_eq!(state.districts, Some(2));
        assert_eq!(state.seed, Some(42));
        assert!(matches!(
            state.metis_engine,
            Some(bisect_cli::args::MetisEngineArg::CFfi)
        ));
    }
    #[test]
    fn all_exposed_dimension_values_parse_in_the_engine() {
        for structure in STRUCTURES {
            for search in SEARCHES {
                for weights in WEIGHTS {
                    let mut config = fixture();
                    config.structure = (*structure).into();
                    config.search = (*search).into();
                    config.weights = (*weights).into();
                    if *weights == "county" {
                        config.alpha_county = 2.0;
                    }
                    if config.validate(&["RI".into()]).is_err() {
                        continue;
                    }
                    let args = config.cli_args("RI", 2, "test.adj.bin", "out", "lab_ri");
                    let cli = bisect_cli::args::Cli::try_parse_from(
                        std::iter::once("bisect".to_owned()).chain(args),
                    )
                    .unwrap();
                    let bisect_cli::args::Commands::State(state) = cli.command else {
                        panic!()
                    };
                    assert!(state.structure.is_some());
                    assert!(state.search.is_some());
                    assert!(state.weights_override.is_some());
                }
            }
        }
    }
    #[test]
    fn rejects_unsupported_and_ignored_combinations() {
        let codes = vec!["RI".into()];
        let mut config = fixture();
        config.states = vec!["../../data".into()];
        assert!(config.validate(&codes).is_err());
        config = fixture();
        config.structure = "bfs-growth".into();
        config.search = "multi".into();
        assert!(config.validate(&codes).is_err());
        config = fixture();
        config.search = "multi".into();
        assert!(config.validate(&codes).is_err());
        config.structure = "ratio-optimal".into();
        assert!(config.validate(&codes).is_ok());
        config.structure = "prime-factor".into();
        assert!(config.validate(&codes).is_err());
        config.structure = "ratio-optimal".into();
        config.search = "flip".into();
        assert!(config.validate(&codes).is_err());
        config.search = "convergence".into();
        assert!(config.validate(&codes).is_err());
        config = fixture();
        config.alpha_county = 2.0;
        assert!(config.validate(&codes).is_err());
        config = fixture();
        config.states.push("RI".into());
        config.mode = "national".into();
        assert!(config.validate(&codes).is_err());
    }
}
