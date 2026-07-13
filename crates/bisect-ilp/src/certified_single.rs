use crate::{certified_split_unit_universe_hash, ExactEdge};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, VecDeque};
use thiserror::Error;

pub const CERTIFIED_SINGLE_INSTANCE_SCHEMA_VERSION: &str = "certified-single-district-instance-v1";
pub const CERTIFIED_SINGLE_CERTIFICATE_SCHEMA_VERSION: &str =
    "certified-single-district-certificate-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedSingleDistrictInstance {
    pub schema_version: String,
    pub unit_universe_hash: String,
    pub unit_ids: Vec<String>,
    pub populations: Vec<i64>,
    pub edges: Vec<ExactEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedSingleDistrictCertificate {
    pub schema_version: String,
    pub certificate_id: String,
    pub instance_hash: String,
    pub unit_universe_hash: String,
    pub assignment: Vec<u8>,
    pub unit_count: usize,
    pub population_total: i64,
    pub weighted_boundary_cut: u64,
    pub connected: bool,
    pub proof_kind: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CertifiedSingleDistrictError {
    #[error("unsupported single-district instance schema: {0}")]
    InstanceSchema(String),
    #[error("unsupported single-district certificate schema: {0}")]
    CertificateSchema(String),
    #[error("unit ids must be nonempty, unique, and ascending")]
    NonCanonicalUnitIds,
    #[error("unit ids and populations have different lengths")]
    UnitPopulationLength,
    #[error("populations must be nonnegative")]
    NegativePopulation,
    #[error("unit universe hash mismatch")]
    UnitUniverseHash,
    #[error("invalid or duplicate graph edge")]
    InvalidEdge,
    #[error("single-district graph must be connected")]
    Disconnected,
    #[error("single-district assignment must contain every unit exactly once in district 0")]
    Assignment,
    #[error("instance hash mismatch")]
    InstanceHash,
    #[error("certificate id mismatch")]
    CertificateId,
    #[error("certificate summary mismatch")]
    Summary,
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("numeric overflow")]
    NumericOverflow,
}

impl CertifiedSingleDistrictInstance {
    pub fn validate(&self) -> Result<(), CertifiedSingleDistrictError> {
        if self.schema_version != CERTIFIED_SINGLE_INSTANCE_SCHEMA_VERSION {
            return Err(CertifiedSingleDistrictError::InstanceSchema(
                self.schema_version.clone(),
            ));
        }
        if self.unit_ids.is_empty()
            || self.unit_ids.len() != self.populations.len()
            || self.unit_ids.iter().any(String::is_empty)
        {
            return Err(CertifiedSingleDistrictError::UnitPopulationLength);
        }
        if self.unit_ids.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(CertifiedSingleDistrictError::NonCanonicalUnitIds);
        }
        if self.populations.iter().any(|&population| population < 0) {
            return Err(CertifiedSingleDistrictError::NegativePopulation);
        }
        if self.unit_universe_hash != certified_split_unit_universe_hash(&self.unit_ids)? {
            return Err(CertifiedSingleDistrictError::UnitUniverseHash);
        }
        let mut seen = BTreeSet::new();
        let mut adjacency = vec![Vec::new(); self.unit_ids.len()];
        for edge in &self.edges {
            if edge.left >= edge.right
                || edge.right >= self.unit_ids.len()
                || edge.weight == 0
                || !seen.insert((edge.left, edge.right))
            {
                return Err(CertifiedSingleDistrictError::InvalidEdge);
            }
            adjacency[edge.left].push(edge.right);
            adjacency[edge.right].push(edge.left);
        }
        let mut reached = BTreeSet::from([0_usize]);
        let mut queue = VecDeque::from([0_usize]);
        while let Some(unit) = queue.pop_front() {
            for &neighbor in &adjacency[unit] {
                if reached.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
        if reached.len() != self.unit_ids.len() {
            return Err(CertifiedSingleDistrictError::Disconnected);
        }
        Ok(())
    }

    pub fn hash(&self) -> Result<String, CertifiedSingleDistrictError> {
        self.validate()?;
        hash_value(self)
    }
}

impl CertifiedSingleDistrictCertificate {
    fn compute_id(&self) -> Result<String, CertifiedSingleDistrictError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            schema_version: &'a str,
            instance_hash: &'a str,
            unit_universe_hash: &'a str,
            assignment: &'a [u8],
            unit_count: usize,
            population_total: i64,
            weighted_boundary_cut: u64,
            connected: bool,
            proof_kind: &'a str,
        }
        hash_value(&Projection {
            schema_version: &self.schema_version,
            instance_hash: &self.instance_hash,
            unit_universe_hash: &self.unit_universe_hash,
            assignment: &self.assignment,
            unit_count: self.unit_count,
            population_total: self.population_total,
            weighted_boundary_cut: self.weighted_boundary_cut,
            connected: self.connected,
            proof_kind: &self.proof_kind,
        })
    }
}

pub fn build_certified_single_district(
    instance: &CertifiedSingleDistrictInstance,
) -> Result<CertifiedSingleDistrictCertificate, CertifiedSingleDistrictError> {
    instance.validate()?;
    let population_total = instance
        .populations
        .iter()
        .try_fold(0_i64, |sum, &value| sum.checked_add(value))
        .ok_or(CertifiedSingleDistrictError::NumericOverflow)?;
    let mut certificate = CertifiedSingleDistrictCertificate {
        schema_version: CERTIFIED_SINGLE_CERTIFICATE_SCHEMA_VERSION.to_string(),
        certificate_id: String::new(),
        instance_hash: instance.hash()?,
        unit_universe_hash: instance.unit_universe_hash.clone(),
        assignment: vec![0; instance.unit_ids.len()],
        unit_count: instance.unit_ids.len(),
        population_total,
        weighted_boundary_cut: 0,
        connected: true,
        proof_kind: "trivial-wall-to-wall-k1".to_string(),
    };
    certificate.certificate_id = certificate.compute_id()?;
    Ok(certificate)
}

pub fn verify_certified_single_district(
    instance: &CertifiedSingleDistrictInstance,
    certificate: &CertifiedSingleDistrictCertificate,
) -> Result<(), CertifiedSingleDistrictError> {
    instance.validate()?;
    if certificate.schema_version != CERTIFIED_SINGLE_CERTIFICATE_SCHEMA_VERSION {
        return Err(CertifiedSingleDistrictError::CertificateSchema(
            certificate.schema_version.clone(),
        ));
    }
    if certificate.instance_hash != instance.hash()? {
        return Err(CertifiedSingleDistrictError::InstanceHash);
    }
    if certificate.certificate_id != certificate.compute_id()? {
        return Err(CertifiedSingleDistrictError::CertificateId);
    }
    if certificate.assignment.len() != instance.unit_ids.len()
        || certificate.assignment.iter().any(|&label| label != 0)
    {
        return Err(CertifiedSingleDistrictError::Assignment);
    }
    let expected = build_certified_single_district(instance)?;
    if certificate.unit_universe_hash != expected.unit_universe_hash
        || certificate.unit_count != expected.unit_count
        || certificate.population_total != expected.population_total
        || certificate.weighted_boundary_cut != 0
        || !certificate.connected
        || certificate.proof_kind != expected.proof_kind
    {
        return Err(CertifiedSingleDistrictError::Summary);
    }
    Ok(())
}

fn hash_value<T: Serialize>(value: &T) -> Result<String, CertifiedSingleDistrictError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| CertifiedSingleDistrictError::Serialization(error.to_string()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

impl From<crate::CertifiedSplitError> for CertifiedSingleDistrictError {
    fn from(error: crate::CertifiedSplitError) -> Self {
        CertifiedSingleDistrictError::Serialization(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path4() -> CertifiedSingleDistrictInstance {
        let unit_ids = (0..4).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        CertifiedSingleDistrictInstance {
            schema_version: CERTIFIED_SINGLE_INSTANCE_SCHEMA_VERSION.to_string(),
            unit_universe_hash: certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![10, 20, 30, 40],
            edges: (0..3)
                .map(|left| ExactEdge {
                    left,
                    right: left + 1,
                    weight: 1,
                })
                .collect(),
        }
    }

    #[test]
    fn path4_single_district_verifies() {
        let instance = path4();
        let certificate = build_certified_single_district(&instance).unwrap();
        verify_certified_single_district(&instance, &certificate).unwrap();
        assert_eq!(certificate.population_total, 100);
        assert_eq!(certificate.weighted_boundary_cut, 0);
    }

    #[test]
    fn omitted_assignment_is_rejected() {
        let instance = path4();
        let mut certificate = build_certified_single_district(&instance).unwrap();
        certificate.assignment.pop();
        certificate.certificate_id = certificate.compute_id().unwrap();
        assert_eq!(
            verify_certified_single_district(&instance, &certificate),
            Err(CertifiedSingleDistrictError::Assignment)
        );
    }

    #[test]
    fn nonzero_assignment_is_rejected() {
        let instance = path4();
        let mut certificate = build_certified_single_district(&instance).unwrap();
        certificate.assignment[2] = 1;
        certificate.certificate_id = certificate.compute_id().unwrap();
        assert_eq!(
            verify_certified_single_district(&instance, &certificate),
            Err(CertifiedSingleDistrictError::Assignment)
        );
    }

    #[test]
    fn disconnected_instance_is_rejected() {
        let mut instance = path4();
        instance.edges.pop();
        assert_eq!(
            build_certified_single_district(&instance),
            Err(CertifiedSingleDistrictError::Disconnected)
        );
    }
}
