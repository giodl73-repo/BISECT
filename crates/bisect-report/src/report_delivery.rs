//! Bounded native HTML/JSON report filenames and companion audit delivery.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CommissionFormat {
    Html,
    Json,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportDeliverySettings {
    pub label: String,
    pub formats: Vec<CommissionFormat>,
    pub audit_with_report: bool,
}
impl ReportDeliverySettings {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.label.is_empty()
            || self.label.len() > 100
            || !self
                .label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            || self.formats.is_empty()
            || self.formats.len() > 8
        {
            anyhow::bail!("Choose a safe output label and HTML/JSON formats.");
        }
        Ok(())
    }
    pub fn wants_html(&self) -> bool {
        self.formats.contains(&CommissionFormat::Html)
    }
    pub fn wants_json(&self) -> bool {
        self.formats.contains(&CommissionFormat::Json)
    }
}
pub fn report_delivery_files(
    settings: &ReportDeliverySettings,
    report_json: &str,
    report_html: Option<&str>,
    audit_json: &str,
) -> anyhow::Result<BTreeMap<String, String>> {
    settings.validate()?;
    let mut files = BTreeMap::new();
    if settings.wants_json() {
        files.insert(
            format!("{}_report.json", settings.label),
            report_json.into(),
        );
    }
    if settings.wants_html() {
        files.insert(
            format!("{}_report.html", settings.label),
            report_html
                .ok_or_else(|| anyhow::anyhow!("Requested HTML report is missing."))?
                .into(),
        );
    }
    if settings.audit_with_report {
        files.insert("audit.json".into(), audit_json.into());
    }
    Ok(files)
}
