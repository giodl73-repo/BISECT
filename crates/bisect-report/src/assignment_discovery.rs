//! Native Verify search order with storage supplied by the host.
use std::path::{Path, PathBuf};

/// Locate the original assignment source. A valid label manifest gates explicit
/// candidates; the final manifest-label fallback intentionally has no such gate.
/// Errors reading the first existing assignment propagate without falling back.
pub fn original_source<E>(
    output_base: &Path,
    year: &str,
    manifest_label: &str,
    explicit_label: Option<&str>,
    mut context_usable: impl FnMut(&Path) -> bool,
    mut exists: impl FnMut(&Path) -> bool,
    mut read: impl FnMut(&Path) -> Result<Vec<u8>, E>,
) -> Result<Option<(PathBuf, Vec<u8>)>, E> {
    if let Some(label) = explicit_label {
        for version in ["v1", "verify", "international"] {
            let directory = output_base.join(version).join(year).join("plans").join(label);
            if context_usable(&directory) {
                let path = directory.join("data/final_assignments.json");
                if exists(&path) {
                    return read(&path).map(|bytes| Some((path, bytes)));
                }
            }
        }
    }
    let path = output_base.join("v1").join(year).join("plans")
        .join(manifest_label).join("data/final_assignments.json");
    if exists(&path) { read(&path).map(|bytes| Some((path, bytes))) } else { Ok(None) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn ordered_explicit_candidates_and_ungated_fallback() {
        let mut files = BTreeMap::new();
        for version in ["v1", "verify", "international"] {
            files.insert(PathBuf::from(format!("{version}/2020/plans/explicit/data/final_assignments.json")), version.as_bytes().to_vec());
        }
        files.insert(PathBuf::from("v1/2020/plans/fallback/data/final_assignments.json"), b"fallback".to_vec());
        for valid in [vec!["v1", "verify"], vec!["verify"], vec!["international"], vec![]] {
            let found = original_source(Path::new(""), "2020", "fallback", Some("explicit"),
                |p| valid.iter().any(|v| p.starts_with(v)), |p| files.contains_key(p),
                |p| Ok::<_, String>(files[p].clone())).unwrap().unwrap();
            assert_eq!(found.1, valid.first().copied().unwrap_or("fallback").as_bytes());
        }
        let fallback = original_source(Path::new(""), "2020", "fallback", None,
            |_| panic!("fallback must not load a label context"), |p| files.contains_key(p),
            |p| Ok::<_, String>(files[p].clone())).unwrap().unwrap();
        assert_eq!(fallback.1, b"fallback");
    }

    #[test]
    fn missing_returns_none_and_read_errors_stop_search() {
        let absent = original_source(Path::new(""), "2020", "missing", Some("explicit"),
            |_| true, |_| false, |_| Ok::<_, String>(vec![])).unwrap();
        assert!(absent.is_none());
        let error = original_source(Path::new(""), "2020", "fallback", Some("explicit"),
            |_| true, |_| true, |_| Err::<Vec<u8>, _>("first source unreadable"));
        assert_eq!(error.unwrap_err(), "first source unreadable");
    }
}
