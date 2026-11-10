use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub seats_per_district: u64,
}

fn default_location() -> String {
    "VT".into()
}
fn default_chamber() -> String {
    "congressional".into()
}
fn default_year() -> String {
    "2020".into()
}
fn default_version() -> String {
    "v1".into()
}
fn default_output_base() -> String {
    "outputs".into()
}
fn default_resolution() -> String {
    "tract".into()
}
fn default_sort_column() -> String {
    "label".into()
}
fn default_sort_dir() -> String {
    "asc".into()
}
fn default_seats() -> u64 {
    1
}

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

#[derive(Debug, Clone, Serialize)]
pub struct DecodedSession {
    pub session: Session,
    pub status: &'static str,
    pub parse_error: Option<String>,
}
pub fn read(source: Option<&str>) -> DecodedSession {
    match source {
        None => DecodedSession {
            session: Session::default(),
            status: "defaulted-absent",
            parse_error: None,
        },
        Some(source) => match toml::from_str::<Session>(source) {
            Ok(session) => DecodedSession {
                session,
                status: "parsed",
                parse_error: None,
            },
            Err(error) => DecodedSession {
                session: Session::default(),
                status: "defaulted-invalid",
                parse_error: Some(error.to_string()),
            },
        },
    }
}
pub fn write(session: &Session) -> Result<String, String> {
    toml::to_string_pretty(session).map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_fallback_and_defaults() {
        assert_eq!(read(None).session, Session::default());
        assert_eq!(read(Some("broken = [")).status, "defaulted-invalid");
        assert_eq!(read(Some("year = 2020")).session, Session::default());
        let s = read(Some("location='RI'\nunknown='ignored'"));
        assert_eq!(s.status, "parsed");
        assert_eq!(s.session.location, "RI");
        assert_eq!(s.session.chamber, "congressional");
    }
    #[test]
    fn toml_roundtrip_exact_large_count() {
        let mut s = Session::default();
        s.seats_per_district = i64::MAX as u64;
        s.output_base = "C:\\local\\outputs".into();
        assert_eq!(read(Some(&write(&s).unwrap())).session, s);
        s.seats_per_district = u64::MAX;
        assert!(write(&s).is_err());
    }
}
