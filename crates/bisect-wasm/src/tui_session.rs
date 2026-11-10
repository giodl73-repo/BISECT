use bisect_report::tui_session::{self, Session};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub location: String,
    pub chamber: String,
    pub year: String,
    pub version: String,
    pub output_base: String,
    pub resolution: String,
    pub sort_column: String,
    pub sort_direction: String,
    pub show_metric_glossary: bool,
    pub adjacency_override: String,
    pub seats_per_district: String,
}
fn validate(session: &Session) -> Result<(), String> {
    if [
        &session.location,
        &session.chamber,
        &session.year,
        &session.version,
        &session.output_base,
        &session.resolution,
        &session.sort_column,
        &session.sort_direction,
        &session.adjacency_override,
    ]
    .iter()
    .any(|s| s.len() > 512)
        || session.seats_per_district > i64::MAX as u64
    {
        return Err("Session strings support 512 UTF-8 bytes each and seats support the native TOML nonnegative i64 range.".into());
    }
    Ok(())
}
fn result(
    session: Session,
    status: &str,
    source_name: Option<String>,
    source: Option<String>,
    error: Option<String>,
) -> Result<Value, String> {
    validate(&session)?;
    let text = tui_session::write(&session)?;
    if text.len() > 8192 {
        return Err("Canonical session exceeds 8 KiB.".into());
    }
    let mut settings = serde_json::to_value(&session).map_err(|e| e.to_string())?;
    settings["seats_per_district"] = json!(session.seats_per_district.to_string());
    Ok(
        json!({"schema_version":"bisect-tui-session-v1","status":status,"settings":settings,"source_name":source_name,"source_toml":source,"source_sha256":source.as_ref().map(|s|format!("{:x}",Sha256::digest(s.as_bytes()))),"parse_error":error,"native_toml":text,"native_toml_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"execution_performed":false,"scope":"Shared native TUI session defaults, TOML decoder and pretty serializer. Absent sources use native defaults; malformed or wrong-typed TOML defaults the entire session, with an explicit diagnostic here. Unknown TOML fields retain native ignore semantics; original source bytes are retained separately. Paths, adjacency override, resolution and seat settings are recorded preferences, not opened, fetched or executed. Export does not run a plan or assert that all preferences are applied to browser/native workflows. Project Open is inert; preference application is separate and explicit."}),
    )
}
pub(crate) fn inspect(
    source_name: Option<String>,
    source: Option<String>,
) -> Result<Value, String> {
    if source.as_ref().is_some_and(|s| s.len() > 8192)
        || source_name.as_ref().is_some_and(|s| s.len() > 200)
        || source.is_some() != source_name.is_some()
    {
        return Err(
            "Supply one named session source up to 8 KiB, or neither for native defaults.".into(),
        );
    }
    let decoded = tui_session::read(source.as_deref());
    result(
        decoded.session,
        decoded.status,
        source_name,
        source,
        decoded.parse_error,
    )
}
pub(crate) fn export(settings: Settings) -> Result<Value, String> {
    if settings.seats_per_district.is_empty()
        || settings.seats_per_district.len() > 19
        || !settings
            .seats_per_district
            .bytes()
            .all(|b| b.is_ascii_digit())
        || settings.seats_per_district.len() > 1 && settings.seats_per_district.starts_with('0')
    {
        return Err("Seats must be a canonical nonnegative decimal integer.".into());
    }
    let session = Session {
        location: settings.location,
        chamber: settings.chamber,
        year: settings.year,
        version: settings.version,
        output_base: settings.output_base,
        resolution: settings.resolution,
        sort_column: settings.sort_column,
        sort_direction: settings.sort_direction,
        show_metric_glossary: settings.show_metric_glossary,
        adjacency_override: settings.adjacency_override,
        seats_per_district: settings
            .seats_per_district
            .parse::<u64>()
            .map_err(|e| e.to_string())?,
    };
    result(session, "exported", None, None, None)
}
