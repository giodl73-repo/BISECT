//! Native label SHA-chain assessment with explicit caller-owned file access.
use std::path::PathBuf;
use serde_json::Value;
use sha2::{Digest,Sha256};
/// One link in the SHA chain verification result.
#[derive(Debug, serde::Serialize)]
pub struct ChainLink {
    pub description: String,
    /// The file that was hashed or the consumer index path.
    /// Kept for debug output (`{link:?}`) and future diagnostic extension.
    #[allow(dead_code)]
    pub path: PathBuf,
    pub sha_recorded: Option<String>,
    pub sha_actual: Option<String>,
    /// True if the path or the recorded SHA is absent. Native is_ok returns false
    /// for incomplete links, so a selected year fails verification.
    pub missing: bool,
}

impl ChainLink {
    pub fn is_ok(&self) -> bool {
        !self.missing && self.sha_recorded.is_some() && self.sha_recorded == self.sha_actual
    }
}

/// Check that `runs/{label}/{year}/index.json` has a `config_sha256` field that
/// matches the SHA-256 of the referenced config file.
pub fn check_config_sha(label: &str, run_index_path: &PathBuf, exists: &impl Fn(&std::path::Path)->bool, read: &impl Fn(&std::path::Path)->Option<Vec<u8>>) -> ChainLink {
    let description = "Config SHA".to_string();

    // Build index must exist
    if !exists(run_index_path) {
        return ChainLink {
            description,
            path: run_index_path.clone(),
            sha_recorded: None,
            sha_actual: None,
            missing: true,
        };
    }

    // Parse the build index
    let content = match read_text(run_index_path, read) {
        Ok(c) => c,
        Err(_) => {
            return ChainLink {
                description,
                path: run_index_path.clone(),
                sha_recorded: None,
                sha_actual: None,
                missing: true,
            }
        }
    };
    let index: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            return ChainLink {
                description,
                path: run_index_path.clone(),
                sha_recorded: None,
                sha_actual: None,
                missing: true,
            }
        }
    };

    // Extract recorded config_sha256
    let recorded_sha = index
        .get("config_sha256")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if recorded_sha.is_none() {
        return ChainLink {
            description: format!("{description} (no config_sha256 in build index)"),
            path: run_index_path.clone(),
            sha_recorded: None,
            sha_actual: None,
            missing: true,
        };
    }

    // Find the config file path: prefer "config_path" field, else fall back to
    // the label-named config in configs/
    let config_path = index
        .get("config_path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("configs").join(format!("{label}.yml")));

    let actual_sha = sha256_of_file(&config_path, read);

    ChainLink {
        description: format!(
            "{description} ({}) vs {}",
            config_path.display(),
            run_index_path.display()
        ),
        path: config_path,
        sha_recorded: recorded_sha,
        sha_actual: actual_sha,
        missing: false,
    }
}

/// Check that `consumer_index` has a field `recorded_field` whose value matches
/// the SHA-256 of `producer_index`.
pub fn check_cross_sha(
    label: &str,
    producer_path: &PathBuf,
    consumer_path: &PathBuf,
    recorded_field: &str,
    exists: &impl Fn(&std::path::Path)->bool,
    read: &impl Fn(&std::path::Path)->Option<Vec<u8>>,
) -> ChainLink {
    let description = format!("{label} SHA");

    // Consumer index must exist (if it doesn't, the stage hasn't run — that's
    // not a chain failure for stages that haven't been executed)
    if !exists(consumer_path) {
        return ChainLink {
            description: format!("{description} (consumer not found)"),
            path: consumer_path.clone(),
            sha_recorded: None,
            sha_actual: None,
            missing: true,
        };
    }

    // Parse consumer index
    let content = match read_text(consumer_path, read) {
        Ok(c) => c,
        Err(_) => {
            return ChainLink {
                description,
                path: consumer_path.clone(),
                sha_recorded: None,
                sha_actual: None,
                missing: true,
            }
        }
    };
    let consumer: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            return ChainLink {
                description,
                path: consumer_path.clone(),
                sha_recorded: None,
                sha_actual: None,
                missing: true,
            }
        }
    };

    let recorded_sha = consumer
        .get(recorded_field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if recorded_sha.is_none() {
        return ChainLink {
            description: format!(
                "{description} (no '{recorded_field}' in {})",
                consumer_path.display()
            ),
            path: consumer_path.clone(),
            sha_recorded: None,
            sha_actual: None,
            missing: true,
        };
    }

    // Compute actual SHA of the producer index
    let actual_sha = if exists(producer_path) {
        sha256_of_file(producer_path, read)
    } else {
        None
    };

    ChainLink {
        description: format!(
            "{description} ({} -> {})",
            producer_path.display(),
            consumer_path.display()
        ),
        path: producer_path.clone(),
        sha_recorded: recorded_sha,
        sha_actual: actual_sha,
        missing: false,
    }
}

/// Compute the SHA-256 of a file, returning `None` if the file cannot be read.
pub fn sha256_of_file(path: &PathBuf, read: &impl Fn(&std::path::Path)->Option<Vec<u8>>) -> Option<String> {
    let bytes = read(path)?;
    let digest = Sha256::digest(&bytes);
    Some(format!("{digest:x}"))
}

/// Render native console output without slicing untrusted UTF-8 hashes.
pub fn format_link(link: &ChainLink)->String {
    if link.missing {return format!("  {}: [MISSING or INCOMPLETE]\n",link.description);}
    let recorded=link.sha_recorded.as_deref().unwrap_or("(none)");
    let short:String=recorded.chars().take(16).collect();
    let status=if link.is_ok(){"MATCH"}else{"MISMATCH"};
    let mut text=format!("  {}: sha256={short}... {status}\n",link.description);
    if !link.is_ok(){let actual=link.sha_actual.as_deref().unwrap_or("(unreadable)");let actual_short:String=actual.chars().take(16).collect();text.push_str(&format!("    recorded: {recorded}\n    actual:   {actual_short}...\n"));}
    text
}
fn read_text(path:&std::path::Path,read:&impl Fn(&std::path::Path)->Option<Vec<u8>>)->Result<String,()> {String::from_utf8(read(path).ok_or(())?).map_err(|_|())}
