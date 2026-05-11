#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RepairStatus {
    NotNeeded,
    Needed,
    NotAttempted,
}
