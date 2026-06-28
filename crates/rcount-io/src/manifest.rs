use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RcountManifest {
    pub rcount_version: String,
    pub jurisdiction: Jurisdiction,
    pub election: Election,
    pub status: String,
    pub hash_algorithm: String,
    pub content_hash: String,
    pub created_by: CreatedBy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Jurisdiction {
    pub country: String,
    pub state: String,
    pub county: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Election {
    pub date: String,
    #[serde(rename = "type")]
    pub election_type: String,
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedBy {
    pub tool: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceIndex {
    pub sources: Vec<SourceEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEntry {
    pub source_id: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageHashes {
    pub package_content_hash: String,
    pub contest_count: usize,
    pub reporting_unit_count: usize,
    pub batch_count: usize,
    pub lineage_count: usize,
    #[serde(default)]
    pub rhist_ref_count: usize,
    #[serde(default)]
    pub rctx_ref_count: usize,
    pub inclusion_proof_count: usize,
    pub cvr_count: usize,
    #[serde(default)]
    pub audit_algorithm_run_count: usize,
    pub rla_audit_count: usize,
    pub manual_audit_count: usize,
    #[serde(default)]
    pub batch_comparison_audit_count: usize,
    pub summary_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCheck {
    pub source_id: String,
    pub path: String,
    pub sha256: String,
}

pub fn synthetic_summary_basic_manifest(
    package: &RcountPackage,
) -> Result<RcountManifest, RcountIoError> {
    synthetic_manifest(package, "canvassed")
}

pub fn synthetic_canvass_correction_manifest(
    package: &RcountPackage,
) -> Result<RcountManifest, RcountIoError> {
    synthetic_manifest(package, "canvassed")
}

pub(crate) fn synthetic_manifest(
    package: &RcountPackage,
    status: &str,
) -> Result<RcountManifest, RcountIoError> {
    Ok(RcountManifest {
        rcount_version: RCOUNT_VERSION.to_string(),
        jurisdiction: Jurisdiction {
            country: "US".to_string(),
            state: "SYN".to_string(),
            county: "SYN-COUNTY-1".to_string(),
        },
        election: Election {
            date: "2024-11-05".to_string(),
            election_type: "general".to_string(),
            scope: "synthetic-county".to_string(),
        },
        status: status.to_string(),
        hash_algorithm: "sha256".to_string(),
        content_hash: package_content_hash(package)?,
        created_by: CreatedBy {
            tool: "rcount-io-example".to_string(),
            version: RCOUNT_VERSION.to_string(),
        },
    })
}
