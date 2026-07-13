use bisect_ilp::{
    verify_exact_canonical_artifacts, ExactCanonicalCertificate, ExactCanonicalInstance,
    ExactCertificateError, ExactProofTranscript,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Expected {
    case: String,
    expected_error: String,
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> T {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/examples/exact-canonical/negative-corpus")
}

#[test]
fn committed_negative_corpus_is_rejected_as_declared() {
    let mut cases: Vec<_> = std::fs::read_dir(corpus_root())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    cases.sort();
    assert_eq!(cases.len(), 5);

    for case in cases {
        let expected: Expected = read_json(&case.join("expected.json"));
        assert_eq!(
            case.file_name().unwrap().to_string_lossy(),
            expected.case,
            "fixture directory and declared case differ"
        );
        let instance: ExactCanonicalInstance =
            read_json(&case.join("exact-canonical-instance.json"));
        let certificate: ExactCanonicalCertificate =
            read_json(&case.join("exact-canonical-certificate.json"));
        let proof: ExactProofTranscript = read_json(&case.join("exact-canonical-proof.json"));
        let error = verify_exact_canonical_artifacts(&instance, &certificate, &proof)
            .expect_err("negative fixture unexpectedly verified");
        match expected.expected_error.as_str() {
            "result-mismatch" => assert_eq!(error, ExactCertificateError::ResultMismatch),
            "certificate-id-mismatch" => assert!(matches!(
                error,
                ExactCertificateError::CertificateIdMismatch { .. }
            )),
            other => panic!("unsupported expected error {other}"),
        }
    }
}
