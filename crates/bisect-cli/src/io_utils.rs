use std::path::Path;

/// Write a serializable value to `path` atomically via a .tmp.json rename.
/// On success, path exists with the full content. On failure, no partial file.
pub(crate) fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let tmp = path.with_extension("tmp.json");
    std::fs::write(&tmp, serde_json::to_string_pretty(value)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
