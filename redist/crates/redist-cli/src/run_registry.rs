/// run_registry.rs — `.redist` run registry for label-based pipeline management.
///
/// The `.redist` file at the repository root (CWD) tracks what has been done
/// to each label (built, analyzed, reported).  It is never edited by the user;
/// only written by `redist` commands.
///
/// All mutating operations acquire an exclusive advisory lock on `.redist.lock`
/// before loading → modifying → atomically saving.  Read-only operations use a
/// shared lock.  This prevents lost-update races when parallel year builds call
/// `mark_built` concurrently.
///
/// Format:
/// ```json
/// {
///   "official_proposal": {
///     "built":    ["2020", "2010", "2000"],
///     "analyzed": ["2020"],
///     "reported": []
///   }
/// }
/// ```
///
/// Invariants (enforced by every mutating call):
///   - `analyzed` ⊆ `built`
///   - `reported` ⊆ `analyzed`
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;

use fs2::FileExt;
use serde::{Deserialize, Serialize};

// ── Data structures ────────────────────────────────────────────────────────────

/// Per-label stage completion state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LabelEntry {
    pub built: Vec<String>,
    pub analyzed: Vec<String>,
    pub reported: Vec<String>,
}

/// The full `.redist` registry — maps label name → stage completion.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Registry {
    #[serde(flatten)]
    pub labels: HashMap<String, LabelEntry>,
}

// ── Path helpers ───────────────────────────────────────────────────────────────

fn registry_path() -> PathBuf {
    PathBuf::from(".redist")
}

fn registry_tmp_path() -> PathBuf {
    PathBuf::from(".redist.tmp")
}

fn lock_path() -> PathBuf {
    PathBuf::from(".redist.lock")
}

// ── Core I/O (without locking — callers must hold lock) ───────────────────────

/// Load the registry from `.redist` in the current working directory.
/// Returns an empty registry if the file does not exist.
/// Does NOT acquire a lock — use `load()` (shared lock) for external callers
/// or call this only while holding the exclusive lock.
fn load_unlocked() -> Result<Registry, String> {
    let path = registry_path();
    if !path.exists() {
        return Ok(Registry::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read .redist: {e}"))?;
    if content.trim().is_empty() {
        return Ok(Registry::default());
    }
    serde_json::from_str::<Registry>(&content)
        .map_err(|e| format!("failed to parse .redist: {e}"))
}

/// Save the registry to `.redist` atomically (write `.redist.tmp` → rename).
/// Does NOT acquire a lock — callers must hold the exclusive lock.
fn save_unlocked(registry: &Registry) -> Result<(), String> {
    let json = serde_json::to_string_pretty(registry)
        .map_err(|e| format!("failed to serialize registry: {e}"))?;
    let tmp = registry_tmp_path();
    fs::write(&tmp, json).map_err(|e| format!("failed to write .redist.tmp: {e}"))?;
    fs::rename(&tmp, registry_path())
        .map_err(|e| format!("failed to rename .redist.tmp → .redist: {e}"))
}

// ── Public locking helpers ─────────────────────────────────────────────────────

/// Open (or create) the lock file and return a handle.
fn open_lock_file() -> Result<fs::File, String> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(lock_path())
        .map_err(|e| format!("failed to open .redist.lock: {e}"))
}

// ── Registry impl ──────────────────────────────────────────────────────────────

impl Registry {
    // ── Public constructors ────────────────────────────────────────────────────

    /// Load from `.redist` in the current working directory while holding a
    /// shared lock on `.redist.lock`.  Returns empty registry if file absent.
    pub fn load() -> Result<Self, String> {
        let lock = open_lock_file()?;
        lock.lock_shared()
            .map_err(|e| format!("failed to acquire shared lock: {e}"))?;
        let result = load_unlocked();
        lock.unlock()
            .map_err(|e| format!("failed to release shared lock: {e}"))?;
        result
    }

    /// Save to `.redist` atomically (write `.redist.tmp` → rename).
    ///
    /// The caller must hold the exclusive lock.  In normal use, call
    /// `with_lock` which handles this automatically.
    pub fn save(&self) -> Result<(), String> {
        save_unlocked(self)
    }

    // ── Mutation under exclusive lock ──────────────────────────────────────────

    /// Acquire an exclusive lock on `.redist.lock`, load the registry, call
    /// `f(&mut registry)`, save the result, then release the lock.
    ///
    /// All mutating operations use this pattern to prevent lost-update races.
    pub fn with_lock<T, F: FnOnce(&mut Registry) -> Result<T, String>>(f: F) -> Result<T, String> {
        let lock = open_lock_file()?;
        lock.lock_exclusive()
            .map_err(|e| format!("failed to acquire exclusive lock: {e}"))?;
        let mut reg = load_unlocked()?;
        let result = f(&mut reg);
        if result.is_ok() {
            save_unlocked(&reg)?;
        }
        lock.unlock()
            .map_err(|e| format!("failed to release exclusive lock: {e}"))?;
        result
    }

    // ── Stage marking ──────────────────────────────────────────────────────────

    /// Mark `year` as built for `label`.  Creates the label entry if needed.
    /// Idempotent: if the year is already in the built list, this is a no-op.
    pub fn mark_built(label: &str, year: &str) -> Result<(), String> {
        Self::with_lock(|reg| {
            let entry = reg.labels.entry(label.to_string()).or_default();
            if !entry.built.contains(&year.to_string()) {
                entry.built.push(year.to_string());
            }
            Ok(())
        })
    }

    /// Mark `year` as analyzed for `label`.
    ///
    /// Returns `Err` if `year` is not in the `built` list (invariant enforcement).
    pub fn mark_analyzed(label: &str, year: &str) -> Result<(), String> {
        Self::with_lock(|reg| {
            let entry = reg.labels.entry(label.to_string()).or_default();
            if !entry.built.contains(&year.to_string()) {
                return Err(format!(
                    "[CONFIG] analyze: '{label}' has not been built for year {year}.\n\
                     Run: redist build {label} --year {year}"
                ));
            }
            if !entry.analyzed.contains(&year.to_string()) {
                entry.analyzed.push(year.to_string());
            }
            Ok(())
        })
    }

    /// Mark `year` as reported for `label`.
    ///
    /// Returns `Err` if `year` is not in the `analyzed` list (invariant enforcement).
    pub fn mark_reported(label: &str, year: &str) -> Result<(), String> {
        Self::with_lock(|reg| {
            let entry = reg.labels.entry(label.to_string()).or_default();
            if !entry.analyzed.contains(&year.to_string()) {
                return Err(format!(
                    "[CONFIG] report: '{label}' has not been analyzed for year {year}.\n\
                     Run: redist analyze {label} --year {year}"
                ));
            }
            if !entry.reported.contains(&year.to_string()) {
                entry.reported.push(year.to_string());
            }
            Ok(())
        })
    }

    /// Remove `year` from the `analyzed` and `reported` lists (staleness cascade).
    ///
    /// Called when `--force` rebuild overwrites a year.  The `built` list is
    /// intentionally NOT modified — the new build will call `mark_built` again.
    pub fn invalidate_year(label: &str, year: &str) -> Result<(), String> {
        Self::with_lock(|reg| {
            if let Some(entry) = reg.labels.get_mut(label) {
                entry.analyzed.retain(|y| y != year);
                entry.reported.retain(|y| y != year);
            }
            Ok(())
        })
    }

    // ── Queries (shared lock) ──────────────────────────────────────────────────

    /// Return the `LabelEntry` for `label`, or `None` if not in the registry.
    pub fn get(label: &str) -> Result<Option<LabelEntry>, String> {
        let reg = Self::load()?;
        Ok(reg.labels.get(label).cloned())
    }

    /// List all labels sorted alphabetically, returning `(name, entry)` pairs.
    pub fn list_labels() -> Result<Vec<(String, LabelEntry)>, String> {
        let reg = Self::load()?;
        let mut pairs: Vec<(String, LabelEntry)> = reg.labels.into_iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(pairs)
    }

    // ── Structural mutations ───────────────────────────────────────────────────

    /// Remove `label` entirely from the registry.
    pub fn remove_label(label: &str) -> Result<(), String> {
        Self::with_lock(|reg| {
            reg.labels.remove(label);
            Ok(())
        })
    }

    /// Rename label `from` → `to`.
    ///
    /// - Returns `Err` if `to` already exists in the registry (unless `force` is true).
    /// - Returns `Err` if `from` does not exist in the registry.
    ///
    /// Note: This is a registry-only rename.  For a full filesystem + registry
    /// rename, use `redist mv` (which calls this after moving directories).
    pub fn rename_label(from: &str, to: &str, force: bool) -> Result<(), String> {
        Self::with_lock(|reg| {
            if !reg.labels.contains_key(from) {
                return Err(format!(
                    "[CONFIG] mv: label '{from}' does not exist in registry"
                ));
            }
            if reg.labels.contains_key(to) && !force {
                return Err(format!(
                    "[CONFIG] mv: label '{to}' already exists in registry. \
                     Use --force to overwrite."
                ));
            }
            let entry = reg.labels.remove(from).unwrap();
            reg.labels.insert(to.to_string(), entry);
            Ok(())
        })
    }

    // ── Pre-flight guards (shared lock) ────────────────────────────────────────

    /// Return `Err` with an actionable message if `year` has not been built for `label`.
    pub fn require_built(label: &str, year: &str) -> Result<(), String> {
        match Self::get(label)? {
            None => Err(format!(
                "[CONFIG] analyze: '{label}' has not been built for year {year}.\n\
                 Run: redist build {label} --year {year}"
            )),
            Some(entry) => {
                if entry.built.contains(&year.to_string()) {
                    Ok(())
                } else {
                    Err(format!(
                        "[CONFIG] analyze: '{label}' has not been built for year {year}.\n\
                         Run: redist build {label} --year {year}"
                    ))
                }
            }
        }
    }

    /// Return `Err` with an actionable message if `year` has not been analyzed for `label`.
    pub fn require_analyzed(label: &str, year: &str) -> Result<(), String> {
        match Self::get(label)? {
            None => Err(format!(
                "[CONFIG] report: '{label}' has not been analyzed for year {year}.\n\
                 Run: redist analyze {label} --year {year}"
            )),
            Some(entry) => {
                if entry.analyzed.contains(&year.to_string()) {
                    Ok(())
                } else {
                    Err(format!(
                        "[CONFIG] report: '{label}' has not been analyzed for year {year}.\n\
                         Run: redist analyze {label} --year {year}"
                    ))
                }
            }
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Set the current working directory to a temp dir for the duration of the
    /// closure, then restore the original directory afterward.
    ///
    /// IMPORTANT: `set_current_dir` is process-wide, so tests that use this
    /// helper must NOT be run in parallel with each other.  Mark test modules
    /// with `#[serial_test::serial]` in production, or run with `-- --test-threads=1`.
    /// For our L0 suite they each use a fresh TempDir, so serial execution
    /// is sufficient to keep them correct.
    fn with_tempdir<F: FnOnce()>(f: F) -> TempDir {
        let dir = TempDir::new().expect("tempdir");
        let original = std::env::current_dir().expect("current_dir");
        std::env::set_current_dir(dir.path()).expect("set_current_dir");
        f();
        std::env::set_current_dir(&original).expect("restore current_dir");
        dir
    }

    // ── 1. load returns empty registry when .redist is absent ─────────────────

    #[test]
    fn test_load_missing_returns_empty() {
        let _dir = with_tempdir(|| {
            let reg = Registry::load().expect("load must succeed");
            assert!(reg.labels.is_empty(), "expected empty registry when .redist absent");
        });
    }

    // ── 2. mark_built adds year to built list ──────────────────────────────────

    #[test]
    fn test_mark_built_adds_year() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("my_label", "2020").expect("mark_built");
            let entry = Registry::get("my_label").expect("get").expect("entry exists");
            assert!(entry.built.contains(&"2020".to_string()), "2020 must be in built");
            assert!(entry.analyzed.is_empty());
            assert!(entry.reported.is_empty());
        });
    }

    // ── 3. mark_built is idempotent ────────────────────────────────────────────

    #[test]
    fn test_mark_built_idempotent() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            Registry::mark_built("lbl", "2020").unwrap();
            let entry = Registry::get("lbl").unwrap().unwrap();
            assert_eq!(entry.built.iter().filter(|y| *y == "2020").count(), 1,
                "duplicate year must not be added");
        });
    }

    // ── 4. mark_analyzed errors when year not built ────────────────────────────

    #[test]
    fn test_mark_analyzed_errors_when_not_built() {
        let _dir = with_tempdir(|| {
            let result = Registry::mark_analyzed("lbl", "2020");
            assert!(result.is_err(), "must error when year not built");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "error must contain [CONFIG] prefix: {msg}");
            assert!(msg.contains("redist build"), "error must contain fix command: {msg}");
        });
    }

    // ── 5. mark_analyzed succeeds after mark_built ────────────────────────────

    #[test]
    fn test_mark_analyzed_succeeds_after_built() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            Registry::mark_analyzed("lbl", "2020").expect("must succeed");
            let entry = Registry::get("lbl").unwrap().unwrap();
            assert!(entry.analyzed.contains(&"2020".to_string()));
        });
    }

    // ── 6. mark_reported errors when year not analyzed ─────────────────────────

    #[test]
    fn test_mark_reported_errors_when_not_analyzed() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            let result = Registry::mark_reported("lbl", "2020");
            assert!(result.is_err(), "must error when year not analyzed");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "error must contain [CONFIG] prefix: {msg}");
            assert!(msg.contains("redist analyze"), "error must contain fix command: {msg}");
        });
    }

    // ── 7. mark_reported succeeds after mark_analyzed ─────────────────────────

    #[test]
    fn test_mark_reported_succeeds_after_analyzed() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            Registry::mark_analyzed("lbl", "2020").unwrap();
            Registry::mark_reported("lbl", "2020").expect("must succeed");
            let entry = Registry::get("lbl").unwrap().unwrap();
            assert!(entry.reported.contains(&"2020".to_string()));
        });
    }

    // ── 8. invalidate_year removes from analyzed+reported but not built ────────

    #[test]
    fn test_invalidate_year_removes_analyzed_and_reported() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            Registry::mark_analyzed("lbl", "2020").unwrap();
            Registry::mark_reported("lbl", "2020").unwrap();

            // Sanity: all three lists have 2020
            let before = Registry::get("lbl").unwrap().unwrap();
            assert!(before.built.contains(&"2020".to_string()));
            assert!(before.analyzed.contains(&"2020".to_string()));
            assert!(before.reported.contains(&"2020".to_string()));

            Registry::invalidate_year("lbl", "2020").expect("invalidate_year");
            let after = Registry::get("lbl").unwrap().unwrap();
            assert!(after.built.contains(&"2020".to_string()), "built must be preserved");
            assert!(!after.analyzed.contains(&"2020".to_string()), "analyzed must be cleared");
            assert!(!after.reported.contains(&"2020".to_string()), "reported must be cleared");
        });
    }

    // ── 9. rename_label updates key in registry ────────────────────────────────

    #[test]
    fn test_rename_label_updates_key() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("old_name", "2020").unwrap();
            Registry::rename_label("old_name", "new_name", false).expect("rename must succeed");

            assert!(Registry::get("old_name").unwrap().is_none(),
                "old label must be gone after rename");
            let entry = Registry::get("new_name").unwrap()
                .expect("new label must exist after rename");
            assert!(entry.built.contains(&"2020".to_string()),
                "renamed entry must preserve built list");
        });
    }

    // ── 10. rename_label errors when target exists (without --force) ───────────

    #[test]
    fn test_rename_label_errors_when_target_exists_no_force() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("alpha", "2020").unwrap();
            Registry::mark_built("beta", "2010").unwrap();

            let result = Registry::rename_label("alpha", "beta", false);
            assert!(result.is_err(), "must error when target already exists");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "error must be a [CONFIG] error: {msg}");
            assert!(msg.contains("--force"), "error must mention --force: {msg}");
        });
    }

    // ── 11. rename_label with --force overwrites existing target ──────────────

    #[test]
    fn test_rename_label_force_overwrites_target() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("alpha", "2020").unwrap();
            Registry::mark_built("beta", "2010").unwrap();

            // With --force the rename must succeed, replacing beta's data with alpha's
            Registry::rename_label("alpha", "beta", true).expect("forced rename must succeed");

            assert!(Registry::get("alpha").unwrap().is_none(), "alpha must be removed");
            let beta = Registry::get("beta").unwrap().expect("beta must exist");
            assert!(beta.built.contains(&"2020".to_string()),
                "beta must now have alpha's built years (2020), not its old data (2010)");
        });
    }

    // ── 12. list_labels returns alphabetically sorted results ─────────────────

    #[test]
    fn test_list_labels_sorted() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("zebra", "2020").unwrap();
            Registry::mark_built("apple", "2020").unwrap();
            Registry::mark_built("mango", "2020").unwrap();

            let labels = Registry::list_labels().expect("list_labels");
            let names: Vec<&str> = labels.iter().map(|(n, _)| n.as_str()).collect();
            assert_eq!(names, vec!["apple", "mango", "zebra"],
                "labels must be sorted alphabetically");
        });
    }

    // ── 13. remove_label deletes the entry ────────────────────────────────────

    #[test]
    fn test_remove_label_deletes_entry() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("to_remove", "2020").unwrap();
            assert!(Registry::get("to_remove").unwrap().is_some());

            Registry::remove_label("to_remove").expect("remove_label");
            assert!(Registry::get("to_remove").unwrap().is_none(),
                "label must be absent after remove");
        });
    }

    // ── 14. require_built produces actionable error message ───────────────────

    #[test]
    fn test_require_built_produces_actionable_message() {
        let _dir = with_tempdir(|| {
            let result = Registry::require_built("my_label", "2020");
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must have [CONFIG] prefix: {msg}");
            assert!(msg.contains("my_label"), "must name the label: {msg}");
            assert!(msg.contains("2020"), "must name the year: {msg}");
            assert!(msg.contains("redist build"), "must suggest fix: {msg}");
        });
    }

    // ── 15. require_analyzed produces actionable error message ────────────────

    #[test]
    fn test_require_analyzed_produces_actionable_message() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("my_label", "2020").unwrap();
            let result = Registry::require_analyzed("my_label", "2020");
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must have [CONFIG] prefix: {msg}");
            assert!(msg.contains("my_label"), "must name the label: {msg}");
            assert!(msg.contains("2020"), "must name the year: {msg}");
            assert!(msg.contains("redist analyze"), "must suggest fix: {msg}");
        });
    }

    // ── 16. save is atomic (tmp file renamed) ─────────────────────────────────

    #[test]
    fn test_save_is_atomic_no_tmp_file_left() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();

            // After a successful mark_built, .redist exists and .redist.tmp does not
            assert!(registry_path().exists(), ".redist must exist after save");
            assert!(!registry_tmp_path().exists(), ".redist.tmp must not exist after atomic rename");
        });
    }

    // ── 17. invariant: analyzed ⊆ built ───────────────────────────────────────

    #[test]
    fn test_invariant_analyzed_subset_of_built() {
        let _dir = with_tempdir(|| {
            // Cannot mark analyzed for year not in built — invariant enforced
            let r = Registry::mark_analyzed("lbl", "2010");
            assert!(r.is_err(), "invariant: analyzed must require built");
        });
    }

    // ── 18. invariant: reported ⊆ analyzed ────────────────────────────────────

    #[test]
    fn test_invariant_reported_subset_of_analyzed() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            // Skip mark_analyzed — should fail on mark_reported
            let r = Registry::mark_reported("lbl", "2020");
            assert!(r.is_err(), "invariant: reported must require analyzed");
        });
    }

    // ── 19. multiple labels coexist ────────────────────────────────────────────

    #[test]
    fn test_multiple_labels_coexist() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("label_a", "2020").unwrap();
            Registry::mark_built("label_b", "2010").unwrap();
            Registry::mark_built("label_b", "2020").unwrap();
            Registry::mark_analyzed("label_b", "2020").unwrap();

            let a = Registry::get("label_a").unwrap().unwrap();
            let b = Registry::get("label_b").unwrap().unwrap();

            assert_eq!(a.built, vec!["2020"]);
            assert!(a.analyzed.is_empty());
            assert!(b.built.contains(&"2010".to_string()));
            assert!(b.built.contains(&"2020".to_string()));
            assert!(b.analyzed.contains(&"2020".to_string()));
        });
    }

    // ── 20. rename_label errors when source does not exist ────────────────────

    #[test]
    fn test_rename_label_errors_when_source_missing() {
        let _dir = with_tempdir(|| {
            let result = Registry::rename_label("nonexistent", "target", false);
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "must be [CONFIG] error: {msg}");
            assert!(msg.contains("nonexistent"), "must name the missing label: {msg}");
        });
    }

    // ── 21. require_built succeeds when year IS built ─────────────────────────
    //
    // The happy path of require_built was never tested (only the error path was).

    #[test]
    fn test_require_built_succeeds_when_year_is_built() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            let result = Registry::require_built("lbl", "2020");
            assert!(result.is_ok(), "require_built must succeed when year is built: {:?}", result);
        });
    }

    // ── 22. require_analyzed succeeds when year IS analyzed ───────────────────

    #[test]
    fn test_require_analyzed_succeeds_when_year_is_analyzed() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            Registry::mark_analyzed("lbl", "2020").unwrap();
            let result = Registry::require_analyzed("lbl", "2020");
            assert!(result.is_ok(), "require_analyzed must succeed when year is analyzed: {:?}", result);
        });
    }

    // ── 23. mark_analyzed idempotent ─────────────────────────────────────────
    //
    // Calling mark_analyzed twice for the same year must not create a duplicate.

    #[test]
    fn test_mark_analyzed_idempotent() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            Registry::mark_analyzed("lbl", "2020").unwrap();
            Registry::mark_analyzed("lbl", "2020").unwrap();
            let entry = Registry::get("lbl").unwrap().unwrap();
            assert_eq!(
                entry.analyzed.iter().filter(|y| *y == "2020").count(),
                1,
                "analyzed list must not contain duplicate 2020"
            );
        });
    }

    // ── 24. mark_reported idempotent ─────────────────────────────────────────

    #[test]
    fn test_mark_reported_idempotent() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            Registry::mark_analyzed("lbl", "2020").unwrap();
            Registry::mark_reported("lbl", "2020").unwrap();
            Registry::mark_reported("lbl", "2020").unwrap();
            let entry = Registry::get("lbl").unwrap().unwrap();
            assert_eq!(
                entry.reported.iter().filter(|y| *y == "2020").count(),
                1,
                "reported list must not contain duplicate 2020"
            );
        });
    }

    // ── 25. require_built: label exists but year missing → [CONFIG] ───────────
    //
    // Distinct from test 14 where the label itself is absent; here the label
    // exists but is built for a DIFFERENT year.

    #[test]
    fn test_require_built_label_exists_but_wrong_year() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2010").unwrap();  // built for 2010, not 2020
            let result = Registry::require_built("lbl", "2020");
            assert!(result.is_err(), "must error when year not in built list");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"),  "[CONFIG] prefix required: {msg}");
            assert!(msg.contains("2020"),      "must name the missing year: {msg}");
            assert!(msg.contains("redist build"), "must suggest fix: {msg}");
        });
    }

    // ── 26. require_analyzed: label exists but year not analyzed → [CONFIG] ──

    #[test]
    fn test_require_analyzed_label_exists_but_wrong_year() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2010").unwrap();
            Registry::mark_analyzed("lbl", "2010").unwrap();
            // 2020 is built but not analyzed for lbl (actually not even built)
            let result = Registry::require_analyzed("lbl", "2020");
            assert!(result.is_err(), "must error when year not analyzed");
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"), "[CONFIG] prefix required: {msg}");
            assert!(msg.contains("redist analyze"), "must suggest fix: {msg}");
        });
    }

    // ── 27. mark_reported error message contains [CONFIG] and redist analyze ──
    //
    // Verify the exact format of the error for the report prerequisite guard.

    #[test]
    fn test_mark_reported_error_message_format() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("my_plan", "2020").unwrap();
            let result = Registry::mark_reported("my_plan", "2020");
            assert!(result.is_err());
            let msg = result.unwrap_err();
            assert!(msg.contains("[CONFIG]"),       "[CONFIG] prefix required: {msg}");
            assert!(msg.contains("my_plan"),        "must name the label: {msg}");
            assert!(msg.contains("2020"),           "must name the year: {msg}");
            assert!(msg.contains("redist analyze"), "must suggest the fix: {msg}");
        });
    }

    // ── 28. invalidate_year on label that does not exist is a no-op ───────────
    //
    // If the label is not in the registry, invalidate_year must succeed silently.

    #[test]
    fn test_invalidate_year_on_absent_label_is_noop() {
        let _dir = with_tempdir(|| {
            let result = Registry::invalidate_year("ghost_label", "2020");
            assert!(result.is_ok(), "invalidate_year on absent label must succeed: {:?}", result);
            // Registry should still be empty
            let labels = Registry::list_labels().unwrap();
            assert!(labels.is_empty(), "registry must remain empty: {labels:?}");
        });
    }

    // ── 29. mark_built multiple years for one label ───────────────────────────

    #[test]
    fn test_mark_built_multiple_years() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("lbl", "2020").unwrap();
            Registry::mark_built("lbl", "2010").unwrap();
            Registry::mark_built("lbl", "2000").unwrap();
            let entry = Registry::get("lbl").unwrap().unwrap();
            assert!(entry.built.contains(&"2020".to_string()), "2020 must be built");
            assert!(entry.built.contains(&"2010".to_string()), "2010 must be built");
            assert!(entry.built.contains(&"2000".to_string()), "2000 must be built");
        });
    }

    // ── 30. Registry round-trips through JSON: .redist is valid JSON ──────────

    #[test]
    fn test_registry_is_valid_json_after_mutations() {
        let _dir = with_tempdir(|| {
            Registry::mark_built("plan_x", "2020").unwrap();
            Registry::mark_analyzed("plan_x", "2020").unwrap();

            let content = std::fs::read_to_string(".redist")
                .expect(".redist must exist after mutations");
            let v: serde_json::Value = serde_json::from_str(&content)
                .expect(".redist must be valid JSON");
            assert!(v.is_object(), ".redist must be a JSON object");
            assert!(v.get("plan_x").is_some(), "plan_x must appear in JSON");
        });
    }
}
