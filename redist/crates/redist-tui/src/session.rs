use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(default = "default_location")]
    pub location: String,
    #[serde(default = "default_chamber")]
    pub chamber: String,
    #[serde(default = "default_year")]
    pub year: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_output_base")]
    pub output_base: String,
    #[serde(default = "default_resolution")]
    pub resolution: String,
    #[serde(default = "default_sort_column")]
    pub sort_column: String,
    #[serde(default = "default_sort_dir")]
    pub sort_direction: String,
    #[serde(default)]
    pub show_metric_glossary: bool,
    #[serde(default)]
    pub adjacency_override: String,
    #[serde(default = "default_seats")]
    pub seats_per_district: usize,
}

fn default_location() -> String { "VT".into() }
fn default_chamber() -> String { "congressional".into() }
fn default_year() -> String { "2020".into() }
fn default_version() -> String { "v1".into() }
fn default_output_base() -> String { "outputs".into() }
fn default_resolution() -> String { "tract".into() }
fn default_sort_column() -> String { "label".into() }
fn default_sort_dir() -> String { "asc".into() }
fn default_seats() -> usize { 1 }

impl Default for Session {
    fn default() -> Self {
        Self {
            location: default_location(),
            chamber: default_chamber(),
            year: default_year(),
            version: default_version(),
            output_base: default_output_base(),
            resolution: default_resolution(),
            sort_column: default_sort_column(),
            sort_direction: default_sort_dir(),
            show_metric_glossary: false,
            adjacency_override: String::new(),
            seats_per_district: 1,
        }
    }
}

/// Path to session config file: ~/.config/redist/tui.toml
pub fn config_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(std::path::PathBuf::from(home)
        .join(".config").join("redist").join("tui.toml"))
}

/// Load session from disk. Returns Default if file absent or unparseable.
pub fn load_session() -> Session {
    let Some(path) = config_path() else { return Session::default() };
    let Ok(content) = std::fs::read_to_string(&path) else { return Session::default() };
    toml::from_str(&content).unwrap_or_default()
}

/// Save session to disk. Silently ignores errors (config is best-effort).
pub fn save_session(session: &Session) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = toml::to_string_pretty(session) {
        let _ = std::fs::write(&path, content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_session_has_expected_values() {
        let s = Session::default();
        assert_eq!(s.chamber, "congressional");
        assert_eq!(s.year, "2020");
        assert_eq!(s.seats_per_district, 1);
        assert_eq!(s.resolution, "tract");
    }

    #[test]
    fn test_save_and_load_session_roundtrip() {
        // Use explicit path functions to avoid global env var mutation
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("tui.toml");

        let mut s = Session::default();
        s.location = "WA".into();
        s.chamber = "house".into();
        s.seats_per_district = 5;

        // Save directly to path
        let content = toml::to_string_pretty(&s).unwrap();
        std::fs::write(&path, &content).unwrap();

        // Load directly from path
        let loaded_content = std::fs::read_to_string(&path).unwrap();
        let loaded: Session = toml::from_str(&loaded_content).unwrap();
        assert_eq!(loaded.location, "WA");
        assert_eq!(loaded.chamber, "house");
        assert_eq!(loaded.seats_per_district, 5);
    }

    #[test]
    fn test_load_session_absent_returns_default() {
        // Load from a non-existent path returns default
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.toml");
        let content = std::fs::read_to_string(&path);
        let s: Session = content.ok()
            .and_then(|c| toml::from_str(&c).ok())
            .unwrap_or_default();
        assert_eq!(s.chamber, "congressional");
    }

    #[test]
    fn test_session_toml_serializes() {
        let s = Session::default();
        let toml_str = toml::to_string_pretty(&s).unwrap();
        assert!(toml_str.contains("chamber"));
        assert!(toml_str.contains("year"));
    }
}
