//! Shared native candidate-race package construction. No filesystem or clock access.
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;
use crate::race_of_candidate::AnnotationSet;
use sha2::{Digest,Sha256};
/// Manifest schema for `BISECT civic add-candidate-race` outputs. Schema-versioned
/// separately from `civic-coi v1` because the columns + provenance shape differ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateRaceManifest {
    pub schema_version: String,
    pub label: String,
    pub year: String,
    pub state: String,
    pub submitter: String,
    pub submitted_at: String,
    pub ingested_at: String,
    /// Number of candidate annotations parsed.
    pub n_annotations: u64,
    /// Number of distinct curators across the annotation set.
    pub n_curators: u64,
    /// True iff every annotation row had `independently_verified=true`.
    pub annotations_independently_verified: bool,
    /// Number of (candidate_name, party) entries with conflicting curator opinions
    /// (parser's `disputes` map).
    pub n_disputes: u64,
    /// Files staged into the output dir (relative names).
    pub original_csv: String,
    pub original_csv_sha256: String,
    pub attestation_doc: String,
    pub attestation_doc_sha256: String,
    /// SHA-256 of the rendered annotations.json (BD-R2 chain link).
    pub annotations_sha256: String,
    /// SHA-256 of the rendered provenance.json.
    pub provenance_sha256: String,
    pub version: String,
    pub build_commit: String,
    pub build_commit_short: String,
    pub rustc_version: String,
}


pub struct Metadata{pub label:String,pub year:String,pub state:String,pub submitter:String,pub submitted_at:String,pub ingested_at:String,pub version:String,pub build_commit:String,pub rustc_version:String}
pub struct Package{pub manifest:CandidateRaceManifest,pub files:BTreeMap<String,String>}
pub fn prepare(meta:Metadata,set:&AnnotationSet,csv:&[u8],attestation_name:&str,attestation:&[u8])->anyhow::Result<Package>{
 if ["candidate_race.csv","annotations.json","provenance.json","manifest.json"].iter().any(|n|attestation_name.eq_ignore_ascii_case(n)){anyhow::bail!("Attestation filename collides with a generated candidate-race package file.");}
 let annotations=serde_json::to_string_pretty(&set.annotations)?;let provenance=serde_json::to_string_pretty(&set.provenance)?;
 let hash=|b:&[u8]|format!("{:x}",Sha256::digest(b));
 let manifest=CandidateRaceManifest{schema_version:"candidate-race v1".into(),label:meta.label,year:meta.year,state:meta.state,submitter:meta.submitter,submitted_at:meta.submitted_at,ingested_at:meta.ingested_at,n_annotations:set.annotations.len() as u64,n_curators:set.provenance.curators.len() as u64,annotations_independently_verified:set.provenance.annotations_independently_verified,n_disputes:set.disputes.len() as u64,original_csv:"candidate_race.csv".into(),original_csv_sha256:hash(csv),attestation_doc:attestation_name.into(),attestation_doc_sha256:hash(attestation),annotations_sha256:hash(annotations.as_bytes()),provenance_sha256:hash(provenance.as_bytes()),version:meta.version,build_commit_short:meta.build_commit.chars().take(8).collect(),build_commit:meta.build_commit,rustc_version:meta.rustc_version};
 let files=BTreeMap::from([("annotations.json".into(),annotations),("provenance.json".into(),provenance),("manifest.json".into(),serde_json::to_string_pretty(&manifest)?)]);Ok(Package{manifest,files})
}
