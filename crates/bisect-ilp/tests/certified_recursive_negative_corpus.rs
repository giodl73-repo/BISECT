use bisect_ilp::{
    verify_certified_bisection_tree_bounded, CertifiedBisectionTree, CertifiedTreeError,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Expected {
    case: String,
    expected_error: String,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/examples/certified-recursive/negative-corpus")
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn committed_recursive_hostile_corpus_is_rejected() {
    let mut cases = std::fs::read_dir(root())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    cases.sort();
    assert_eq!(cases.len(), 5);

    for case in cases {
        let expected: Expected = read_json(&case.join("expected.json"));
        assert_eq!(case.file_name().unwrap().to_string_lossy(), expected.case);
        let tree: CertifiedBisectionTree = read_json(&case.join("certified-bisection-tree.json"));
        let error = verify_certified_bisection_tree_bounded(&tree)
            .expect_err("hostile recursive tree unexpectedly verified");
        match expected.expected_error.as_str() {
            "tree-id-mismatch" => {
                assert!(matches!(error, CertifiedTreeError::TreeIdMismatch { .. }))
            }
            "leaf-set-mismatch" => assert_eq!(error, CertifiedTreeError::LeafSetMismatch),
            "node-schedule-mismatch" => {
                assert_eq!(error, CertifiedTreeError::NodeScheduleMismatch)
            }
            "leaf-mismatch" => assert!(matches!(error, CertifiedTreeError::LeafMismatch(_))),
            "split-result-mismatch" => assert!(matches!(
                error,
                CertifiedTreeError::Split(bisect_ilp::CertifiedSplitError::ResultMismatch)
            )),
            other => panic!("unsupported expected recursive error {other}"),
        }
    }
}
