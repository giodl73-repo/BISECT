use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

pub const EXACT_INSTANCE_SCHEMA_VERSION: &str = "exact-canonical-instance-v1";
pub const EXACT_CERTIFICATE_SCHEMA_VERSION: &str = "exact-canonical-certificate-v1";
pub const EXACT_PROOF_SCHEMA_VERSION: &str = "exact-canonical-proof-v1";
pub const EXACT_MODEL_ID: &str = "exact-canonical-k2-exhaustive-v1";
pub const EXACT_ENUMERATION_LIMIT: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactEdge {
    pub left: usize,
    pub right: usize,
    pub weight: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactCanonicalInstance {
    pub schema_version: String,
    pub model_id: String,
    pub unit_ids: Vec<String>,
    pub populations: Vec<i64>,
    pub edges: Vec<ExactEdge>,
    pub k: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PrimaryObjective {
    /// `abs(k * district_population - total_population)`, maximum over districts.
    pub max_population_deviation_scaled: u64,
    /// Sum of the scaled absolute district deviations.
    pub total_population_deviation_scaled: u64,
    /// Sum of weights for edges whose endpoints have different district labels.
    pub weighted_boundary_cut: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactObjective {
    pub primary: PrimaryObjective,
    /// Canonical district labels in the instance's unit order.
    pub canonical_assignment: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum ExactCertificateResult {
    Optimal {
        assignment: Vec<u8>,
        objective: ExactObjective,
    },
    Infeasible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExhaustiveProof {
    pub proof_kind: String,
    pub fixed_label_unit: usize,
    pub enumerated_assignments: u64,
    pub feasible_assignments: u64,
    pub primary_objective_ties: u64,
    pub lower_bound: Option<PrimaryObjective>,
    pub transcript_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactProofTranscript {
    pub schema_version: String,
    pub transcript_id: String,
    pub instance_hash: String,
    pub model_id: String,
    pub enumeration_order: String,
    pub fixed_label_unit: usize,
    pub candidate_count: u64,
    pub feasible_count: u64,
    pub primary_objective_ties: u64,
    pub lower_bound: Option<PrimaryObjective>,
    pub canonical_assignment: Option<Vec<u8>>,
    /// SHA-256 commitment to every candidate in enumeration order, including
    /// feasibility, objective values, and assignment bytes.
    pub search_commitment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactCanonicalArtifacts {
    pub certificate: ExactCanonicalCertificate,
    pub proof: ExactProofTranscript,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactCanonicalCertificate {
    pub schema_version: String,
    pub certificate_id: String,
    pub instance_hash: String,
    pub model_id: String,
    pub result: ExactCertificateResult,
    pub proof: ExhaustiveProof,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExactCertificateError {
    #[error("unsupported instance schema: {0}")]
    InstanceSchema(String),
    #[error("unsupported certificate schema: {0}")]
    CertificateSchema(String),
    #[error("unsupported proof schema: {0}")]
    ProofSchema(String),
    #[error("unsupported exact model: {0}")]
    Model(String),
    #[error("exact reference solver supports k=2, found k={0}")]
    UnsupportedDistrictCount(usize),
    #[error("exact reference instance has {found} units; supported range is 2..={limit}")]
    InstanceSize { found: usize, limit: usize },
    #[error("unit ids and populations have different lengths")]
    UnitPopulationLength,
    #[error("unit ids must be non-empty and unique")]
    InvalidUnitIds,
    #[error("unit ids must be in ascending canonical order")]
    NonCanonicalUnitOrder,
    #[error("populations must be non-negative")]
    NegativePopulation,
    #[error("population or edge-weight totals exceed the exact reference numeric range")]
    NumericOverflow,
    #[error("invalid or duplicate edge ({left}, {right})")]
    InvalidEdge { left: usize, right: usize },
    #[error("edge weights must be positive")]
    ZeroEdgeWeight,
    #[error("canonical serialization failed: {0}")]
    Canonicalization(String),
    #[error("certificate id mismatch: expected {expected}, found {found}")]
    CertificateIdMismatch { expected: String, found: String },
    #[error("instance hash mismatch: expected {expected}, found {found}")]
    InstanceHashMismatch { expected: String, found: String },
    #[error("certificate result does not match exhaustive verification")]
    ResultMismatch,
    #[error("certificate proof statistics do not match exhaustive verification")]
    ProofMismatch,
    #[error("proof transcript id mismatch: expected {expected}, found {found}")]
    ProofIdMismatch { expected: String, found: String },
    #[error("proof transcript does not match exhaustive verification")]
    TranscriptMismatch,
}

impl ExactCanonicalInstance {
    pub fn validate(&self) -> Result<(), ExactCertificateError> {
        if self.schema_version != EXACT_INSTANCE_SCHEMA_VERSION {
            return Err(ExactCertificateError::InstanceSchema(
                self.schema_version.clone(),
            ));
        }
        if self.model_id != EXACT_MODEL_ID {
            return Err(ExactCertificateError::Model(self.model_id.clone()));
        }
        if self.k != 2 {
            return Err(ExactCertificateError::UnsupportedDistrictCount(self.k));
        }
        let n = self.unit_ids.len();
        if !(2..=EXACT_ENUMERATION_LIMIT).contains(&n) {
            return Err(ExactCertificateError::InstanceSize {
                found: n,
                limit: EXACT_ENUMERATION_LIMIT,
            });
        }
        if self.populations.len() != n {
            return Err(ExactCertificateError::UnitPopulationLength);
        }
        if self.unit_ids.iter().any(|id| id.trim().is_empty())
            || self.unit_ids.iter().collect::<BTreeSet<_>>().len() != n
        {
            return Err(ExactCertificateError::InvalidUnitIds);
        }
        if self.unit_ids.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(ExactCertificateError::NonCanonicalUnitOrder);
        }
        if self.populations.iter().any(|&population| population < 0) {
            return Err(ExactCertificateError::NegativePopulation);
        }
        let total_population = self
            .populations
            .iter()
            .try_fold(0_i64, |sum, &population| sum.checked_add(population))
            .ok_or(ExactCertificateError::NumericOverflow)?;
        if total_population.checked_mul(self.k as i64).is_none() {
            return Err(ExactCertificateError::NumericOverflow);
        }
        let mut seen = BTreeSet::new();
        for edge in &self.edges {
            if edge.left >= n
                || edge.right >= n
                || edge.left >= edge.right
                || !seen.insert((edge.left, edge.right))
            {
                return Err(ExactCertificateError::InvalidEdge {
                    left: edge.left,
                    right: edge.right,
                });
            }
            if edge.weight == 0 {
                return Err(ExactCertificateError::ZeroEdgeWeight);
            }
        }
        if self
            .edges
            .iter()
            .try_fold(0_u64, |sum, edge| sum.checked_add(edge.weight))
            .is_none()
        {
            return Err(ExactCertificateError::NumericOverflow);
        }
        Ok(())
    }

    pub fn hash(&self) -> Result<String, ExactCertificateError> {
        canonical_hash(self)
    }
}

impl ExactCanonicalCertificate {
    pub fn compute_id(&self) -> Result<String, ExactCertificateError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            schema_version: &'a str,
            instance_hash: &'a str,
            model_id: &'a str,
            result: &'a ExactCertificateResult,
            proof: &'a ExhaustiveProof,
        }

        canonical_hash(&Projection {
            schema_version: &self.schema_version,
            instance_hash: &self.instance_hash,
            model_id: &self.model_id,
            result: &self.result,
            proof: &self.proof,
        })
    }
}

impl ExactProofTranscript {
    pub fn compute_id(&self) -> Result<String, ExactCertificateError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            schema_version: &'a str,
            instance_hash: &'a str,
            model_id: &'a str,
            enumeration_order: &'a str,
            fixed_label_unit: usize,
            candidate_count: u64,
            feasible_count: u64,
            primary_objective_ties: u64,
            lower_bound: &'a Option<PrimaryObjective>,
            canonical_assignment: &'a Option<Vec<u8>>,
            search_commitment: &'a str,
        }
        canonical_hash(&Projection {
            schema_version: &self.schema_version,
            instance_hash: &self.instance_hash,
            model_id: &self.model_id,
            enumeration_order: &self.enumeration_order,
            fixed_label_unit: self.fixed_label_unit,
            candidate_count: self.candidate_count,
            feasible_count: self.feasible_count,
            primary_objective_ties: self.primary_objective_ties,
            lower_bound: &self.lower_bound,
            canonical_assignment: &self.canonical_assignment,
            search_commitment: &self.search_commitment,
        })
    }
}

pub fn solve_exact_canonical(
    instance: &ExactCanonicalInstance,
) -> Result<ExactCanonicalCertificate, ExactCertificateError> {
    Ok(solve_exact_canonical_artifacts(instance)?.certificate)
}

pub fn solve_exact_canonical_artifacts(
    instance: &ExactCanonicalInstance,
) -> Result<ExactCanonicalArtifacts, ExactCertificateError> {
    instance.validate()?;
    let search = enumerate(instance);
    let proof = transcript_from_search(instance, &search)?;
    let certificate = certificate_from_search(instance, search, &proof)?;
    Ok(ExactCanonicalArtifacts { certificate, proof })
}

pub fn verify_exact_canonical_certificate(
    instance: &ExactCanonicalInstance,
    certificate: &ExactCanonicalCertificate,
) -> Result<(), ExactCertificateError> {
    instance.validate()?;
    if certificate.schema_version != EXACT_CERTIFICATE_SCHEMA_VERSION {
        return Err(ExactCertificateError::CertificateSchema(
            certificate.schema_version.clone(),
        ));
    }
    if certificate.model_id != EXACT_MODEL_ID {
        return Err(ExactCertificateError::Model(certificate.model_id.clone()));
    }
    let expected_instance_hash = instance.hash()?;
    if certificate.instance_hash != expected_instance_hash {
        return Err(ExactCertificateError::InstanceHashMismatch {
            expected: expected_instance_hash,
            found: certificate.instance_hash.clone(),
        });
    }
    let expected_id = certificate.compute_id()?;
    if certificate.certificate_id != expected_id {
        return Err(ExactCertificateError::CertificateIdMismatch {
            expected: expected_id,
            found: certificate.certificate_id.clone(),
        });
    }

    // Deliberately re-enumerate from the instance rather than trusting solver
    // bounds, search statistics, or the submitted assignment.
    let expected = solve_exact_canonical_artifacts(instance)?.certificate;
    if certificate.result != expected.result {
        return Err(ExactCertificateError::ResultMismatch);
    }
    if certificate.proof != expected.proof {
        return Err(ExactCertificateError::ProofMismatch);
    }
    Ok(())
}

pub fn verify_exact_canonical_artifacts(
    instance: &ExactCanonicalInstance,
    certificate: &ExactCanonicalCertificate,
    proof: &ExactProofTranscript,
) -> Result<(), ExactCertificateError> {
    verify_exact_canonical_certificate(instance, certificate)?;
    if proof.schema_version != EXACT_PROOF_SCHEMA_VERSION {
        return Err(ExactCertificateError::ProofSchema(
            proof.schema_version.clone(),
        ));
    }
    if proof.instance_hash != instance.hash()? || proof.model_id != EXACT_MODEL_ID {
        return Err(ExactCertificateError::TranscriptMismatch);
    }
    let expected_id = proof.compute_id()?;
    if proof.transcript_id != expected_id {
        return Err(ExactCertificateError::ProofIdMismatch {
            expected: expected_id,
            found: proof.transcript_id.clone(),
        });
    }
    if certificate.proof.transcript_id != proof.transcript_id {
        return Err(ExactCertificateError::TranscriptMismatch);
    }
    let expected = solve_exact_canonical_artifacts(instance)?;
    if *proof != expected.proof {
        return Err(ExactCertificateError::TranscriptMismatch);
    }
    Ok(())
}

struct SearchResult {
    enumerated_assignments: u64,
    feasible_assignments: u64,
    primary_objective_ties: u64,
    best_primary: Option<PrimaryObjective>,
    best_assignment: Option<Vec<u8>>,
    search_commitment: String,
}

fn enumerate(instance: &ExactCanonicalInstance) -> SearchResult {
    let n = instance.unit_ids.len();
    let candidate_count = (1_u64 << (n - 1)) - 1;
    let mut transcript = Sha256::new();
    transcript.update(b"EXACT_CANONICAL_TRANSCRIPT_V1\0");
    let mut result = SearchResult {
        enumerated_assignments: candidate_count,
        feasible_assignments: 0,
        primary_objective_ties: 0,
        best_primary: None,
        best_assignment: None,
        search_commitment: String::new(),
    };

    // Unit 0 is fixed to district 0 to remove district-label symmetry.
    for mask in 1..=candidate_count {
        let mut assignment = vec![0_u8; n];
        for unit in 1..n {
            assignment[unit] = ((mask >> (unit - 1)) & 1) as u8;
        }
        transcript.update((mask as u64).to_le_bytes());
        if !districts_connected(instance, &assignment) {
            transcript.update([0_u8]);
            continue;
        }
        result.feasible_assignments += 1;
        let primary = objective(instance, &assignment);
        transcript.update([1_u8]);
        transcript.update(primary.max_population_deviation_scaled.to_le_bytes());
        transcript.update(primary.total_population_deviation_scaled.to_le_bytes());
        transcript.update(primary.weighted_boundary_cut.to_le_bytes());
        transcript.update(&assignment);
        match result.best_primary.as_ref() {
            None => {
                result.best_primary = Some(primary);
                result.best_assignment = Some(assignment);
                result.primary_objective_ties = 1;
            }
            Some(best) if primary < *best => {
                result.best_primary = Some(primary);
                result.best_assignment = Some(assignment);
                result.primary_objective_ties = 1;
            }
            Some(best) if primary == *best => {
                result.primary_objective_ties += 1;
                if result
                    .best_assignment
                    .as_ref()
                    .is_none_or(|current| assignment < *current)
                {
                    result.best_assignment = Some(assignment);
                }
            }
            _ => {}
        }
    }
    result.search_commitment = format!("sha256:{:x}", transcript.finalize());
    result
}

fn certificate_from_search(
    instance: &ExactCanonicalInstance,
    search: SearchResult,
    transcript: &ExactProofTranscript,
) -> Result<ExactCanonicalCertificate, ExactCertificateError> {
    let result = match (&search.best_primary, &search.best_assignment) {
        (Some(primary), Some(assignment)) => ExactCertificateResult::Optimal {
            assignment: assignment.clone(),
            objective: ExactObjective {
                primary: primary.clone(),
                canonical_assignment: assignment.clone(),
            },
        },
        _ => ExactCertificateResult::Infeasible,
    };
    let proof = ExhaustiveProof {
        proof_kind: "label-fixed-exhaustive-enumeration".to_string(),
        fixed_label_unit: 0,
        enumerated_assignments: search.enumerated_assignments,
        feasible_assignments: search.feasible_assignments,
        primary_objective_ties: search.primary_objective_ties,
        lower_bound: search.best_primary,
        transcript_id: transcript.transcript_id.clone(),
    };
    let mut certificate = ExactCanonicalCertificate {
        schema_version: EXACT_CERTIFICATE_SCHEMA_VERSION.to_string(),
        certificate_id: String::new(),
        instance_hash: instance.hash()?,
        model_id: EXACT_MODEL_ID.to_string(),
        result,
        proof,
    };
    certificate.certificate_id = certificate.compute_id()?;
    Ok(certificate)
}

fn transcript_from_search(
    instance: &ExactCanonicalInstance,
    search: &SearchResult,
) -> Result<ExactProofTranscript, ExactCertificateError> {
    let mut transcript = ExactProofTranscript {
        schema_version: EXACT_PROOF_SCHEMA_VERSION.to_string(),
        transcript_id: String::new(),
        instance_hash: instance.hash()?,
        model_id: EXACT_MODEL_ID.to_string(),
        enumeration_order: "unit-0-fixed-label-0; nonzero masks ascending through 2^(n-1)-1"
            .to_string(),
        fixed_label_unit: 0,
        candidate_count: search.enumerated_assignments,
        feasible_count: search.feasible_assignments,
        primary_objective_ties: search.primary_objective_ties,
        lower_bound: search.best_primary.clone(),
        canonical_assignment: search.best_assignment.clone(),
        search_commitment: search.search_commitment.clone(),
    };
    transcript.transcript_id = transcript.compute_id()?;
    Ok(transcript)
}

fn objective(instance: &ExactCanonicalInstance, assignment: &[u8]) -> PrimaryObjective {
    let mut district_population = [0_i64; 2];
    for (unit, &district) in assignment.iter().enumerate() {
        district_population[district as usize] += instance.populations[unit];
    }
    let total_population = district_population.iter().sum::<i64>();
    let deviations = district_population
        .map(|population| (instance.k as i64 * population - total_population).unsigned_abs());
    let weighted_boundary_cut = instance
        .edges
        .iter()
        .filter(|edge| assignment[edge.left] != assignment[edge.right])
        .map(|edge| edge.weight)
        .sum();
    PrimaryObjective {
        max_population_deviation_scaled: deviations[0].max(deviations[1]),
        total_population_deviation_scaled: deviations.iter().sum(),
        weighted_boundary_cut,
    }
}

fn districts_connected(instance: &ExactCanonicalInstance, assignment: &[u8]) -> bool {
    let mut adjacency = vec![Vec::new(); assignment.len()];
    for edge in &instance.edges {
        adjacency[edge.left].push(edge.right);
        adjacency[edge.right].push(edge.left);
    }
    [0_u8, 1_u8].into_iter().all(|district| {
        let units = assignment
            .iter()
            .enumerate()
            .filter_map(|(unit, &label)| (label == district).then_some(unit))
            .collect::<Vec<_>>();
        let Some(&start) = units.first() else {
            return false;
        };
        let allowed = units.iter().copied().collect::<BTreeSet<_>>();
        let mut visited = BTreeSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some(unit) = queue.pop_front() {
            for &neighbor in &adjacency[unit] {
                if allowed.contains(&neighbor) && visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
        visited.len() == units.len()
    })
}

fn canonical_hash<T: Serialize>(value: &T) -> Result<String, ExactCertificateError> {
    let value = serde_json::to_value(value)
        .map_err(|error| ExactCertificateError::Canonicalization(error.to_string()))?;
    let canonical = canonicalize(value);
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| ExactCertificateError::Canonicalization(error.to_string()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn canonicalize(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .map(|(key, value)| (key, canonicalize(value)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonicalize).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path4() -> ExactCanonicalInstance {
        ExactCanonicalInstance {
            schema_version: EXACT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: EXACT_MODEL_ID.to_string(),
            unit_ids: ["u0", "u1", "u2", "u3"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            populations: vec![100, 100, 100, 100],
            edges: vec![
                ExactEdge {
                    left: 0,
                    right: 1,
                    weight: 1,
                },
                ExactEdge {
                    left: 1,
                    right: 2,
                    weight: 1,
                },
                ExactEdge {
                    left: 2,
                    right: 3,
                    weight: 1,
                },
            ],
            k: 2,
        }
    }

    fn cycle4() -> ExactCanonicalInstance {
        let mut instance = path4();
        instance.edges.push(ExactEdge {
            left: 0,
            right: 3,
            weight: 1,
        });
        instance
    }

    fn infeasible_three_islands() -> ExactCanonicalInstance {
        ExactCanonicalInstance {
            schema_version: EXACT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: EXACT_MODEL_ID.to_string(),
            unit_ids: ["a", "b", "c"].into_iter().map(str::to_string).collect(),
            populations: vec![1, 1, 1],
            edges: vec![],
            k: 2,
        }
    }

    #[test]
    fn canonical_path4_produces_unique_optimal_certificate() {
        let instance = path4();
        let artifacts = solve_exact_canonical_artifacts(&instance).unwrap();
        let certificate = artifacts.certificate;
        let ExactCertificateResult::Optimal {
            assignment,
            objective,
        } = &certificate.result
        else {
            panic!("path4 must be feasible");
        };
        assert_eq!(assignment, &[0, 0, 1, 1]);
        assert_eq!(objective.primary.max_population_deviation_scaled, 0);
        assert_eq!(objective.primary.total_population_deviation_scaled, 0);
        assert_eq!(objective.primary.weighted_boundary_cut, 1);
        assert_eq!(certificate.proof.feasible_assignments, 3);
        assert_eq!(
            verify_exact_canonical_certificate(&instance, &certificate),
            Ok(())
        );
        assert_eq!(
            verify_exact_canonical_artifacts(&instance, &certificate, &artifacts.proof),
            Ok(())
        );
        assert_eq!(
            certificate.proof.transcript_id,
            artifacts.proof.transcript_id
        );
    }

    #[test]
    fn canonical_cycle4_uses_lexicographic_tie_break() {
        let certificate = solve_exact_canonical(&cycle4()).unwrap();
        let ExactCertificateResult::Optimal { assignment, .. } = certificate.result else {
            panic!("cycle4 must be feasible");
        };
        assert_eq!(assignment, vec![0, 0, 1, 1]);
        assert_eq!(certificate.proof.primary_objective_ties, 2);
    }

    #[test]
    fn canonical_infeasibility_certificate_verifies() {
        let instance = infeasible_three_islands();
        let certificate = solve_exact_canonical(&instance).unwrap();
        assert_eq!(certificate.result, ExactCertificateResult::Infeasible);
        assert_eq!(certificate.proof.feasible_assignments, 0);
        assert_eq!(
            verify_exact_canonical_certificate(&instance, &certificate),
            Ok(())
        );
    }

    #[test]
    fn canonical_verifier_rejects_false_optimum() {
        let instance = path4();
        let mut certificate = solve_exact_canonical(&instance).unwrap();
        if let ExactCertificateResult::Optimal { objective, .. } = &mut certificate.result {
            objective.primary.weighted_boundary_cut += 1;
        }
        certificate.certificate_id = certificate.compute_id().unwrap();
        assert_eq!(
            verify_exact_canonical_certificate(&instance, &certificate),
            Err(ExactCertificateError::ResultMismatch)
        );
    }

    #[test]
    fn canonical_verifier_rejects_false_infeasibility() {
        let instance = path4();
        let mut certificate = solve_exact_canonical(&instance).unwrap();
        certificate.result = ExactCertificateResult::Infeasible;
        certificate.certificate_id = certificate.compute_id().unwrap();
        assert_eq!(
            verify_exact_canonical_certificate(&instance, &certificate),
            Err(ExactCertificateError::ResultMismatch)
        );
    }

    #[test]
    fn canonical_verifier_rejects_certificate_id_tamper() {
        let instance = path4();
        let mut certificate = solve_exact_canonical(&instance).unwrap();
        certificate.certificate_id = "sha256:bad".to_string();
        assert!(matches!(
            verify_exact_canonical_certificate(&instance, &certificate),
            Err(ExactCertificateError::CertificateIdMismatch { .. })
        ));
    }

    #[test]
    fn canonical_unbalanced_path_minimizes_population_before_cut() {
        let mut instance = path4();
        instance.populations = vec![100, 100, 100, 50];
        let certificate = solve_exact_canonical(&instance).unwrap();
        let ExactCertificateResult::Optimal {
            assignment,
            objective,
        } = certificate.result
        else {
            panic!("unbalanced path remains feasible");
        };
        assert_eq!(assignment, vec![0, 0, 1, 1]);
        assert_eq!(objective.primary.max_population_deviation_scaled, 50);
        assert_eq!(objective.primary.total_population_deviation_scaled, 100);
        assert_eq!(objective.primary.weighted_boundary_cut, 1);
    }

    #[test]
    fn canonical_instance_rejects_unsorted_unit_order() {
        let mut instance = path4();
        instance.unit_ids.swap(1, 2);
        assert_eq!(
            instance.validate(),
            Err(ExactCertificateError::NonCanonicalUnitOrder)
        );
    }

    #[test]
    fn canonical_instance_rejects_numeric_overflow() {
        let mut instance = path4();
        instance.populations = vec![i64::MAX, 1, 1, 1];
        assert_eq!(
            instance.validate(),
            Err(ExactCertificateError::NumericOverflow)
        );
    }

    #[test]
    fn canonical_instance_rejects_district_objective_overflow() {
        let mut instance = path4();
        instance.populations = vec![2_000_000_000_000_000_000; 4];
        assert_eq!(
            instance.validate(),
            Err(ExactCertificateError::NumericOverflow)
        );
    }

    #[test]
    fn canonical_proof_transcript_is_deterministic() {
        let first = solve_exact_canonical_artifacts(&cycle4()).unwrap();
        let second = solve_exact_canonical_artifacts(&cycle4()).unwrap();
        assert_eq!(first.proof, second.proof);
        assert_eq!(first.certificate, second.certificate);
    }

    #[test]
    fn canonical_verifier_rejects_proof_commitment_tamper() {
        let instance = path4();
        let artifacts = solve_exact_canonical_artifacts(&instance).unwrap();
        let mut proof = artifacts.proof;
        proof.search_commitment = format!("sha256:{}", "0".repeat(64));
        proof.transcript_id = proof.compute_id().unwrap();
        assert_eq!(
            verify_exact_canonical_artifacts(&instance, &artifacts.certificate, &proof),
            Err(ExactCertificateError::TranscriptMismatch)
        );
    }

    #[test]
    fn canonical_verifier_rejects_proof_id_tamper() {
        let instance = path4();
        let artifacts = solve_exact_canonical_artifacts(&instance).unwrap();
        let mut proof = artifacts.proof;
        proof.transcript_id = "sha256:bad".to_string();
        assert!(matches!(
            verify_exact_canonical_artifacts(&instance, &artifacts.certificate, &proof),
            Err(ExactCertificateError::ProofIdMismatch { .. })
        ));
    }
}
