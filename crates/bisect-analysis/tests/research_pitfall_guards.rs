use std::fs;
use std::path::{Path, PathBuf};

const D_SERIES_MAIN_FILES: &[&str] = &[
    "research/tracks/D-vra-legal/D.0+vra-compliance/main.tex",
    "research/tracks/D-vra-legal/D.1+threshold-analysis/main.tex",
    "research/tracks/D-vra-legal/D.2+nway-vs-recursive-vra/main.tex",
    "research/tracks/D-vra-legal/D.3+compactness-tradeoff/main.tex",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate lives under crates/bisect-analysis")
        .to_path_buf()
}

#[test]
fn d_series_papers_declare_population_metric_for_vra_claims() {
    let root = repo_root();
    for rel in D_SERIES_MAIN_FILES {
        let content =
            fs::read_to_string(root.join(rel)).expect("paper main.tex should be readable");
        assert!(
            content.contains("Population metric declaration"),
            "{rel} must explicitly declare the population metric used for MM percentages"
        );
        assert!(
            content.contains("total resident population")
                && content.contains("voting-age population (VAP)")
                && content.contains("citizen voting-age population (CVAP)"),
            "{rel} must distinguish total population, VAP, and CVAP"
        );
    }
}

#[test]
fn d_series_threshold_claims_are_bounded_not_point_results() {
    let root = repo_root();
    for rel in D_SERIES_MAIN_FILES {
        let content =
            fs::read_to_string(root.join(rel)).expect("paper main.tex should be readable");
        assert!(
            content.contains("Threshold sensitivity declaration"),
            "{rel} must include a threshold sensitivity declaration"
        );
        assert!(
            content.contains("bounded empirical transition")
                && content.contains("point legal rule")
                && content.contains("Mississippi")
                && content.contains("sensitivity bands"),
            "{rel} must frame threshold results as sensitivity-bounded claims"
        );
    }
}
