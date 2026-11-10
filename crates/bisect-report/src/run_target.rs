//! Native State guards shared by filesystem and selected-folder adapters.
use std::path::Path;
/// Same native collision decision for selected in-memory folder evidence.
/// Existence remains distinct from readable or valid manifest contents.
pub fn check_plan_collision_evidence(plan_dir:&Path,force:bool,manifest_exists:bool,manifest_text:Option<&str>)->anyhow::Result<()>{
    if !force && manifest_exists {
        // Read created_at from existing manifest for a helpful error message
        let created_at = manifest_text
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v["created_at"].as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        let label = plan_dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| plan_dir.display().to_string());

        anyhow::bail!(
            "ERROR: plan '{}' already exists at: {}\n\
             Existing manifest created_at: {}\n\
             Use --force to overwrite, or choose a different --label.",
            label,
            plan_dir.display(),
            created_at,
        );
    }
    Ok(())
}

/// Native incomplete-target guard on an explicit selected existence record.
pub fn check_incomplete_plan_evidence(label:&str,manifest_tmp_exists:bool)->anyhow::Result<()>{
    if manifest_tmp_exists {
        anyhow::bail!(
            "ERROR: Incomplete plan '{}' detected (manifest.tmp exists). \
             Delete the plan directory and re-run.",
            label
        );
    }
    Ok(())
}

