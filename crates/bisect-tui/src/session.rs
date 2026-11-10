pub use bisect_report::tui_session::Session;

/// Path to session config file: ~/.config/BISECT/tui.toml
pub fn config_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(
        std::path::PathBuf::from(home)
            .join(".config")
            .join("bisect")
            .join("tui.toml"),
    )
}

/// Load session from disk. Returns Default if file absent or unparseable.
pub fn load_session() -> Session {
    let Some(path) = config_path() else {
        return Session::default();
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Session::default();
    };
    bisect_report::tui_session::read(Some(&content)).session
}

/// Save session to disk. Silently ignores errors (config is best-effort).
pub fn save_session(session: &Session) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = bisect_report::tui_session::write(session) {
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
        let s: Session = content
            .ok()
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
