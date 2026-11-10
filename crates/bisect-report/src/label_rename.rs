//! Shared label mutation kernels; callers own filesystem access and transactions.
use std::collections::BTreeMap;
use crate::label_registry::LabelEntry;

pub fn validate_name(label: &str) -> Result<(), String> {
    if label.is_empty() { return Err("label must not be empty".into()); }
    if label.starts_with('.') { return Err(format!("label '{label}' must not start with '.'")); }
    if let Some(bad) = label.chars().find(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '-') {
        return Err(format!("label '{label}' contains invalid character '{bad}'; only [a-zA-Z0-9_-] are allowed"));
    }
    if ["runs", "analysis", "reports", ".bisect", "configs"].contains(&label) {
        return Err(format!("'{label}' is a reserved name and cannot be used as a label"));
    }
    Ok(())
}

pub fn distinct_names(src: &str, dst: &str) -> Result<(), String> {
    if src == dst { return Err("[CONFIG] mv: source and destination labels must differ".into()); }
    // On case-insensitive filesystems both paths name the same directory.
    // Reject before a forced destination removal can delete the source itself.
    if src.eq_ignore_ascii_case(dst) {
        return Err("[CONFIG] mv: case-only label renames are not portable across filesystems".into());
    }
    Ok(())
}

/// Native behavior reserializes every top-level index, even if its label differs.
pub fn patch_index(text: &str, old: &str, new: &str) -> Result<String, String> {
    let mut value: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if let Some(obj) = value.as_object_mut() {
        if obj.get("label").and_then(|v| v.as_str()) == Some(old) {
            obj.insert("label".into(), serde_json::Value::String(new.into()));
        }
    }
    serde_json::to_string_pretty(&value).map_err(|e| e.to_string())
}

/// In-memory equivalent of successful native directory moves. Produces a new
/// package; errors leave the caller's source intact. Empty directories must be
/// supplied explicitly because browser file pickers cannot discover them.
pub fn rename_package(
    registry: &BTreeMap<String, LabelEntry>, files: &BTreeMap<String, Vec<u8>>,
    directories: &[String], src: &str, dst: &str, force: bool,
) -> Result<(BTreeMap<String, LabelEntry>, BTreeMap<String, Vec<u8>>, Vec<String>), String> {
    validate_name(dst)?;
    validate_name(src)?;
    distinct_names(src, dst)?;
    let entry = registry.get(src).ok_or_else(|| format!("[CONFIG] mv: label '{src}' does not exist in registry.\nRun: bisect ls    to see available labels."))?;
    if registry.contains_key(dst) && !force {
        return Err(format!("[CONFIG] mv: label '{dst}' already exists in registry. Use --force to overwrite."));
    }
    let mut out = files.clone();
    let mut dirs = directories.to_vec();
    for stage in ["runs", "analysis", "reports"] {
        let from = format!("{stage}/{src}/");
        let to = format!("{stage}/{dst}/");
        let source_exists = files.keys().any(|p| p.starts_with(&from)) || directories.iter().any(|p| p.starts_with(&from));
        if !source_exists { continue; }
        let destination_exists = out.keys().any(|p| p.starts_with(&to)) || dirs.iter().any(|p| p.starts_with(&to));
        if destination_exists && !force {
            return Err(format!("[CONFIG] mv: destination directory '{stage}/{dst}' already exists. Use --force to overwrite."));
        }
        out.retain(|p, _| !p.starts_with(&from) && !p.starts_with(&to));
        dirs.retain(|p| !p.starts_with(&from) && !p.starts_with(&to));
        for (path, bytes) in files.iter().filter(|(p, _)| p.starts_with(&from)) {
            out.insert(format!("{to}{}", &path[from.len()..]), bytes.clone());
        }
        for path in directories.iter().filter(|p| p.starts_with(&from)) {
            dirs.push(format!("{to}{}", &path[from.len()..]));
        }
        let index = format!("{to}index.json");
        if let Some(bytes) = out.get(&index) {
            let text = std::str::from_utf8(bytes).map_err(|e| format!("patch_label: failed to read '{index}': {e}"))?;
            let patched = patch_index(text, src, dst).map_err(|e| format!("patch_label: failed to parse '{index}': {e}"))?;
            out.insert(index, patched.into_bytes());
        }
    }
    let mut updated = registry.clone();
    updated.remove(src);
    updated.insert(dst.into(), entry.clone());
    dirs.sort(); dirs.dedup();
    Ok((updated, out, dirs))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn case_only_names_are_refused_before_filesystem_effects() {
        for (source, destination) in [("alpha", "Alpha"), ("ALPHA", "alpha")] {
            let error = distinct_names(source, destination).unwrap_err();
            assert!(error.contains("case-only"));
        }
        assert!(distinct_names("alpha", "beta").is_ok());
    }

    #[test]
    fn forced_self_rename_is_rejected_before_mutation() {
        let registry = BTreeMap::from([("a".into(), LabelEntry::default())]);
        let files = BTreeMap::from([("runs/a/data.bin".into(), vec![0, 255])]);
        assert!(rename_package(&registry, &files, &[], "a", "a", true).is_err());
        assert_eq!(files["runs/a/data.bin"], [0,255]);
    }
    #[test]
    fn moves_binary_and_patches_only_top_index() {
        let registry = BTreeMap::from([("a".into(), LabelEntry {built:vec!["2020".into(),"2020".into()], ..Default::default()})]);
        let files = BTreeMap::from([
            ("runs/a/index.json".into(), br#"{"label":"a","config_path":"configs/a.yml"}"#.to_vec()),
            ("runs/a/2020/index.json".into(), br#"{"label":"a"}"#.to_vec()),
            ("analysis/a/data.bin".into(), vec![0,255]),
            ("configs/a.yml".into(), vec![1]),
        ]);
        let (reg,out,dirs) = rename_package(&registry,&files,&["reports/a/empty/".into()],"a","b",false).unwrap();
        assert_eq!(reg["b"].built, ["2020", "2020"]);
        assert!(!reg.contains_key("a"));
        assert_eq!(out["analysis/b/data.bin"], [0,255]);
        assert_eq!(out["runs/b/2020/index.json"], files["runs/a/2020/index.json"]);
        let top:serde_json::Value=serde_json::from_slice(&out["runs/b/index.json"]).unwrap();
        assert_eq!(top["label"], "b"); assert_eq!(top["config_path"], "configs/a.yml");
        assert_eq!(dirs, ["reports/b/empty/"]);
    }
    #[test]
    fn force_replaces_only_stages_with_a_source() {
        let reg=BTreeMap::from([("a".into(),LabelEntry::default()),("b".into(),LabelEntry::default())]);
        let files=BTreeMap::from([("runs/a/new".into(),vec![1]),("runs/b/old".into(),vec![2]),("analysis/b/keep".into(),vec![3])]);
        assert!(rename_package(&reg,&files,&[],"a","b",false).is_err());
        let (_,out,_)=rename_package(&reg,&files,&[],"a","b",true).unwrap();
        assert!(!out.contains_key("runs/b/old")); assert_eq!(out["runs/b/new"],[1]); assert_eq!(out["analysis/b/keep"],[3]);
    }
    #[test]
    fn malformed_index_fails_without_mutating_source() {
        let reg=BTreeMap::from([("a".into(),LabelEntry::default())]);
        let files=BTreeMap::from([("runs/a/index.json".into(),b"invalid".to_vec())]);
        assert!(rename_package(&reg,&files,&[],"a","b",false).is_err());
        assert!(files.contains_key("runs/a/index.json"));
    }
}
