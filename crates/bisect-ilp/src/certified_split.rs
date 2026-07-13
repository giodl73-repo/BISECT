use crate::ExactEdge;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

pub const CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION: &str =
    "certified-recursive-bisection-split-instance-v1";
pub const CERTIFIED_SPLIT_CERTIFICATE_SCHEMA_VERSION: &str =
    "certified-recursive-bisection-split-certificate-v1";
pub const CERTIFIED_SPLIT_PROOF_SCHEMA_VERSION: &str =
    "certified-recursive-bisection-split-proof-v1";
pub const CERTIFIED_SPLIT_MODEL_ID: &str = "certified-standard-bisect-split-v1";
pub const CERTIFIED_SPLIT_ENUMERATION_LIMIT: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SplitOrientationRule {
    EqualSeatsUnitZeroLeft,
    SeatOrderedFloorLeftCeilRight,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedSplitInstance {
    pub schema_version: String,
    pub model_id: String,
    /// Existing `BisectionTree` path: empty root, then `0` left and `1` right.
    pub node_path: String,
    pub parent_certificate_id: Option<String>,
    pub unit_universe_hash: String,
    pub unit_ids: Vec<String>,
    pub populations: Vec<i64>,
    pub edges: Vec<ExactEdge>,
    pub k_parent: usize,
    pub k_left: usize,
    pub k_right: usize,
    pub orientation_rule: SplitOrientationRule,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CertifiedSplitPrimaryObjective {
    pub max_population_deviation_scaled: u64,
    pub total_population_deviation_scaled: u64,
    pub weighted_boundary_cut: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedSplitObjective {
    pub primary: CertifiedSplitPrimaryObjective,
    /// `0` selects the `k_left` child and `1` selects the `k_right` child.
    pub canonical_assignment: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum CertifiedSplitResult {
    Optimal {
        assignment: Vec<u8>,
        objective: CertifiedSplitObjective,
    },
    Infeasible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedSplitProofSummary {
    pub proof_kind: String,
    /// Counts the enumerated canonical space; equal-seat label-swap twins are excluded.
    pub feasible_assignments: Option<u64>,
    /// Counts primary-optimal assignments in that same canonical space.
    pub primary_objective_ties: Option<u64>,
    pub lower_bound: Option<CertifiedSplitPrimaryObjective>,
    pub proof_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedSplitProof {
    pub schema_version: String,
    pub proof_id: String,
    pub instance_hash: String,
    pub model_id: String,
    pub enumeration_order: String,
    pub candidate_count: u64,
    /// Counts the enumerated canonical space; equal-seat label-swap twins are excluded.
    pub feasible_count: u64,
    /// Counts primary-optimal assignments in that same canonical space.
    pub primary_objective_ties: u64,
    pub lower_bound: Option<CertifiedSplitPrimaryObjective>,
    pub canonical_assignment: Option<Vec<u8>>,
    pub search_commitment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedSplitArtifacts {
    pub instance: CertifiedSplitInstance,
    pub certificate: CertifiedSplitCertificate,
    pub proof: CertifiedSplitProof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedSplitCertificate {
    pub schema_version: String,
    pub certificate_id: String,
    pub instance_hash: String,
    pub model_id: String,
    pub result: CertifiedSplitResult,
    pub proof: CertifiedSplitProofSummary,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CertifiedSplitError {
    #[error("unsupported certified split instance schema: {0}")]
    InstanceSchema(String),
    #[error("unsupported certified split model: {0}")]
    Model(String),
    #[error("unsupported certified split certificate schema: {0}")]
    CertificateSchema(String),
    #[error("unsupported certified split proof schema: {0}")]
    ProofSchema(String),
    #[error("node path must be empty for root or contain only 0 and 1")]
    InvalidNodePath,
    #[error("root must not declare a parent certificate; child nodes must declare one")]
    ParentCertificateLink,
    #[error("parent certificate id must use the sha256:<64 lowercase hex> shape")]
    InvalidParentCertificateId,
    #[error("unit universe hash mismatch: expected {expected}, found {found}")]
    UnitUniverseHashMismatch { expected: String, found: String },
    #[error("certified recursive bisection requires k_parent >= 2")]
    InvalidParentSeatCount,
    #[error(
        "noncanonical seat split: expected {expected_left}/{expected_right}, found {found_left}/{found_right}"
    )]
    NonCanonicalSeatSplit {
        expected_left: usize,
        expected_right: usize,
        found_left: usize,
        found_right: usize,
    },
    #[error("orientation rule does not match the canonical seat split")]
    OrientationRuleMismatch,
    #[error("unit ids and populations have different lengths or fewer than two units")]
    UnitPopulationLength,
    #[error("unit ids must be nonempty, unique, and ascending")]
    NonCanonicalUnitIds,
    #[error("populations must be nonnegative")]
    NegativePopulation,
    #[error("invalid or duplicate edge ({left}, {right})")]
    InvalidEdge { left: usize, right: usize },
    #[error("edge weights must be positive")]
    ZeroEdgeWeight,
    #[error("population or edge arithmetic exceeds the split model numeric range")]
    NumericOverflow,
    #[error("bounded split oracle has {found} units; limit is {limit}")]
    EnumerationLimit { found: usize, limit: usize },
    #[error("assignment length does not match the instance")]
    AssignmentLength,
    #[error("assignments must contain only 0 and 1")]
    InvalidAssignmentLabel,
    #[error("both split children must be nonempty")]
    EmptyChild,
    #[error(
        "split children need at least their seat counts in units: left {left_units}/{k_left}, right {right_units}/{k_right}"
    )]
    InsufficientChildUnits {
        left_units: usize,
        right_units: usize,
        k_left: usize,
        k_right: usize,
    },
    #[error("equal-seat splits require canonical unit 0 in the left child")]
    EqualSeatOrientation,
    #[error("canonical serialization failed: {0}")]
    Canonicalization(String),
    #[error("instance hash mismatch: expected {expected}, found {found}")]
    InstanceHashMismatch { expected: String, found: String },
    #[error("certificate id mismatch: expected {expected}, found {found}")]
    CertificateIdMismatch { expected: String, found: String },
    #[error("proof id mismatch: expected {expected}, found {found}")]
    ProofIdMismatch { expected: String, found: String },
    #[error("certificate result differs from bounded exhaustive verification")]
    ResultMismatch,
    #[error("certificate proof summary differs from bounded exhaustive verification")]
    ProofSummaryMismatch,
    #[error("proof transcript differs from bounded exhaustive verification")]
    ProofMismatch,
}

impl CertifiedSplitInstance {
    pub fn validate(&self) -> Result<(), CertifiedSplitError> {
        if self.schema_version != CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION {
            return Err(CertifiedSplitError::InstanceSchema(
                self.schema_version.clone(),
            ));
        }
        if self.model_id != CERTIFIED_SPLIT_MODEL_ID {
            return Err(CertifiedSplitError::Model(self.model_id.clone()));
        }
        if !self
            .node_path
            .bytes()
            .all(|byte| matches!(byte, b'0' | b'1'))
        {
            return Err(CertifiedSplitError::InvalidNodePath);
        }
        if self.node_path.is_empty() != self.parent_certificate_id.is_none() {
            return Err(CertifiedSplitError::ParentCertificateLink);
        }
        if self
            .parent_certificate_id
            .as_ref()
            .is_some_and(|identifier| !valid_sha256_id(identifier))
        {
            return Err(CertifiedSplitError::InvalidParentCertificateId);
        }
        let (expected_left, expected_right) = canonical_seat_split(self.k_parent)?;
        if (self.k_left, self.k_right) != (expected_left, expected_right) {
            return Err(CertifiedSplitError::NonCanonicalSeatSplit {
                expected_left,
                expected_right,
                found_left: self.k_left,
                found_right: self.k_right,
            });
        }
        if self.orientation_rule != canonical_orientation_rule(self.k_left, self.k_right) {
            return Err(CertifiedSplitError::OrientationRuleMismatch);
        }
        let unit_count = self.unit_ids.len();
        if unit_count < 2 || self.populations.len() != unit_count {
            return Err(CertifiedSplitError::UnitPopulationLength);
        }
        if self
            .unit_ids
            .iter()
            .any(|unit_id| unit_id.trim().is_empty())
            || self.unit_ids.iter().collect::<BTreeSet<_>>().len() != unit_count
            || self.unit_ids.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(CertifiedSplitError::NonCanonicalUnitIds);
        }
        let expected_unit_hash = certified_split_unit_universe_hash(&self.unit_ids)?;
        if self.unit_universe_hash != expected_unit_hash {
            return Err(CertifiedSplitError::UnitUniverseHashMismatch {
                expected: expected_unit_hash,
                found: self.unit_universe_hash.clone(),
            });
        }
        if self.populations.iter().any(|&population| population < 0) {
            return Err(CertifiedSplitError::NegativePopulation);
        }
        let total_population = self
            .populations
            .iter()
            .try_fold(0_i128, |sum, &population| {
                sum.checked_add(i128::from(population))
            })
            .ok_or(CertifiedSplitError::NumericOverflow)?;
        let Some(max_scaled_population) = total_population.checked_mul(self.k_parent as i128)
        else {
            return Err(CertifiedSplitError::NumericOverflow);
        };
        if max_scaled_population > i128::from(u64::MAX / 2) {
            return Err(CertifiedSplitError::NumericOverflow);
        }
        let mut seen = BTreeSet::new();
        for edge in &self.edges {
            if edge.left >= unit_count
                || edge.right >= unit_count
                || edge.left >= edge.right
                || !seen.insert((edge.left, edge.right))
            {
                return Err(CertifiedSplitError::InvalidEdge {
                    left: edge.left,
                    right: edge.right,
                });
            }
            if edge.weight == 0 {
                return Err(CertifiedSplitError::ZeroEdgeWeight);
            }
        }
        if self
            .edges
            .iter()
            .try_fold(0_u64, |sum, edge| sum.checked_add(edge.weight))
            .is_none()
        {
            return Err(CertifiedSplitError::NumericOverflow);
        }
        Ok(())
    }

    pub fn hash(&self) -> Result<String, CertifiedSplitError> {
        canonical_hash(self)
    }
}

impl CertifiedSplitCertificate {
    pub fn compute_id(&self) -> Result<String, CertifiedSplitError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            schema_version: &'a str,
            instance_hash: &'a str,
            model_id: &'a str,
            result: &'a CertifiedSplitResult,
            proof: &'a CertifiedSplitProofSummary,
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

impl CertifiedSplitProof {
    pub fn compute_id(&self) -> Result<String, CertifiedSplitError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            schema_version: &'a str,
            instance_hash: &'a str,
            model_id: &'a str,
            enumeration_order: &'a str,
            candidate_count: u64,
            feasible_count: u64,
            primary_objective_ties: u64,
            lower_bound: &'a Option<CertifiedSplitPrimaryObjective>,
            canonical_assignment: &'a Option<Vec<u8>>,
            search_commitment: &'a str,
        }
        canonical_hash(&Projection {
            schema_version: &self.schema_version,
            instance_hash: &self.instance_hash,
            model_id: &self.model_id,
            enumeration_order: &self.enumeration_order,
            candidate_count: self.candidate_count,
            feasible_count: self.feasible_count,
            primary_objective_ties: self.primary_objective_ties,
            lower_bound: &self.lower_bound,
            canonical_assignment: &self.canonical_assignment,
            search_commitment: &self.search_commitment,
        })
    }
}

struct CertifiedSplitSearch {
    enumeration_order: String,
    candidate_count: u64,
    feasible_count: u64,
    primary_objective_ties: u64,
    best_primary: Option<CertifiedSplitPrimaryObjective>,
    best_assignment: Option<Vec<u8>>,
    search_commitment: String,
}

pub fn solve_certified_split_bounded(
    instance: &CertifiedSplitInstance,
) -> Result<CertifiedSplitArtifacts, CertifiedSplitError> {
    instance.validate()?;
    if instance.unit_ids.len() > CERTIFIED_SPLIT_ENUMERATION_LIMIT {
        return Err(CertifiedSplitError::EnumerationLimit {
            found: instance.unit_ids.len(),
            limit: CERTIFIED_SPLIT_ENUMERATION_LIMIT,
        });
    }
    let instance_hash = instance.hash()?;
    let search = enumerate_certified_split(instance, &instance_hash);
    artifacts_from_search(instance, search)
}

pub fn verify_certified_split_bounded(
    instance: &CertifiedSplitInstance,
    certificate: &CertifiedSplitCertificate,
    proof: &CertifiedSplitProof,
) -> Result<(), CertifiedSplitError> {
    instance.validate()?;
    if instance.unit_ids.len() > CERTIFIED_SPLIT_ENUMERATION_LIMIT {
        return Err(CertifiedSplitError::EnumerationLimit {
            found: instance.unit_ids.len(),
            limit: CERTIFIED_SPLIT_ENUMERATION_LIMIT,
        });
    }
    if certificate.schema_version != CERTIFIED_SPLIT_CERTIFICATE_SCHEMA_VERSION {
        return Err(CertifiedSplitError::CertificateSchema(
            certificate.schema_version.clone(),
        ));
    }
    if certificate.model_id != CERTIFIED_SPLIT_MODEL_ID {
        return Err(CertifiedSplitError::Model(certificate.model_id.clone()));
    }
    let instance_hash = instance.hash()?;
    if certificate.instance_hash != instance_hash {
        return Err(CertifiedSplitError::InstanceHashMismatch {
            expected: instance_hash,
            found: certificate.instance_hash.clone(),
        });
    }
    let certificate_id = certificate.compute_id()?;
    if certificate.certificate_id != certificate_id {
        return Err(CertifiedSplitError::CertificateIdMismatch {
            expected: certificate_id,
            found: certificate.certificate_id.clone(),
        });
    }
    if proof.schema_version != CERTIFIED_SPLIT_PROOF_SCHEMA_VERSION {
        return Err(CertifiedSplitError::ProofSchema(
            proof.schema_version.clone(),
        ));
    }
    if proof.model_id != CERTIFIED_SPLIT_MODEL_ID || proof.instance_hash != instance.hash()? {
        return Err(CertifiedSplitError::ProofMismatch);
    }
    let proof_id = proof.compute_id()?;
    if proof.proof_id != proof_id {
        return Err(CertifiedSplitError::ProofIdMismatch {
            expected: proof_id,
            found: proof.proof_id.clone(),
        });
    }
    if certificate.proof.proof_id.as_deref() != Some(proof.proof_id.as_str()) {
        return Err(CertifiedSplitError::ProofMismatch);
    }

    let expected = artifacts_from_search(
        instance,
        enumerate_certified_split(instance, &instance_hash),
    )?;
    if certificate.result != expected.certificate.result {
        return Err(CertifiedSplitError::ResultMismatch);
    }
    if certificate.proof != expected.certificate.proof {
        return Err(CertifiedSplitError::ProofSummaryMismatch);
    }
    if *proof != expected.proof {
        return Err(CertifiedSplitError::ProofMismatch);
    }
    Ok(())
}

fn enumerate_certified_split(
    instance: &CertifiedSplitInstance,
    instance_hash: &str,
) -> CertifiedSplitSearch {
    let unit_count = instance.unit_ids.len();
    let equal_seats = instance.k_left == instance.k_right;
    let (candidate_count, mask_end_exclusive, enumeration_order) = if equal_seats {
        let candidates = (1_u64 << (unit_count - 1)) - 1;
        (
            candidates,
            candidates + 1,
            "equal-seats; unit-0-fixed-left; nonzero reduced masks ascending".to_string(),
        )
    } else {
        let all_assignments = 1_u64 << unit_count;
        (
            all_assignments - 2,
            all_assignments - 1,
            "unequal-seats; full masks ascending; empty children excluded".to_string(),
        )
    };
    let adjacency = adjacency(instance);
    let mut transcript = Sha256::new();
    transcript.update(b"CERTIFIED_RECURSIVE_BISECTION_SPLIT_TRANSCRIPT_V1\0");
    transcript.update(instance_hash.as_bytes());
    transcript.update([0_u8]);
    let mut search = CertifiedSplitSearch {
        enumeration_order,
        candidate_count,
        feasible_count: 0,
        primary_objective_ties: 0,
        best_primary: None,
        best_assignment: None,
        search_commitment: String::new(),
    };

    for mask in 1..mask_end_exclusive {
        let assignment = if equal_seats {
            let mut assignment = vec![0_u8; unit_count];
            for unit in 1..unit_count {
                assignment[unit] = ((mask >> (unit - 1)) & 1) as u8;
            }
            assignment
        } else {
            (0..unit_count)
                .map(|unit| ((mask >> unit) & 1) as u8)
                .collect()
        };
        transcript.update(mask.to_le_bytes());
        if !children_have_enough_units(instance, &assignment) {
            transcript.update([0_u8]);
            continue;
        }
        if !children_connected_unchecked(&assignment, &adjacency) {
            transcript.update([0_u8]);
            continue;
        }
        search.feasible_count += 1;
        let primary = objective_unchecked(instance, &assignment);
        transcript.update([1_u8]);
        transcript.update(primary.max_population_deviation_scaled.to_le_bytes());
        transcript.update(primary.total_population_deviation_scaled.to_le_bytes());
        transcript.update(primary.weighted_boundary_cut.to_le_bytes());
        transcript.update(&assignment);
        match search.best_primary.as_ref() {
            None => {
                search.best_primary = Some(primary);
                search.best_assignment = Some(assignment);
                search.primary_objective_ties = 1;
            }
            Some(best) if primary < *best => {
                search.best_primary = Some(primary);
                search.best_assignment = Some(assignment);
                search.primary_objective_ties = 1;
            }
            Some(best) if primary == *best => {
                search.primary_objective_ties += 1;
                if search
                    .best_assignment
                    .as_ref()
                    .is_none_or(|current| assignment < *current)
                {
                    search.best_assignment = Some(assignment);
                }
            }
            _ => {}
        }
    }
    search.search_commitment = format!("sha256:{:x}", transcript.finalize());
    search
}

fn artifacts_from_search(
    instance: &CertifiedSplitInstance,
    search: CertifiedSplitSearch,
) -> Result<CertifiedSplitArtifacts, CertifiedSplitError> {
    let instance_hash = instance.hash()?;
    let mut proof = CertifiedSplitProof {
        schema_version: CERTIFIED_SPLIT_PROOF_SCHEMA_VERSION.to_string(),
        proof_id: String::new(),
        instance_hash: instance_hash.clone(),
        model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
        enumeration_order: search.enumeration_order,
        candidate_count: search.candidate_count,
        feasible_count: search.feasible_count,
        primary_objective_ties: search.primary_objective_ties,
        lower_bound: search.best_primary.clone(),
        canonical_assignment: search.best_assignment.clone(),
        search_commitment: search.search_commitment,
    };
    proof.proof_id = proof.compute_id()?;
    let result = match (&search.best_primary, &search.best_assignment) {
        (Some(primary), Some(assignment)) => CertifiedSplitResult::Optimal {
            assignment: assignment.clone(),
            objective: CertifiedSplitObjective {
                primary: primary.clone(),
                canonical_assignment: assignment.clone(),
            },
        },
        _ => CertifiedSplitResult::Infeasible,
    };
    let mut certificate = CertifiedSplitCertificate {
        schema_version: CERTIFIED_SPLIT_CERTIFICATE_SCHEMA_VERSION.to_string(),
        certificate_id: String::new(),
        instance_hash,
        model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
        result,
        proof: CertifiedSplitProofSummary {
            proof_kind: "bounded-exhaustive-enumeration".to_string(),
            feasible_assignments: Some(search.feasible_count),
            primary_objective_ties: Some(search.primary_objective_ties),
            lower_bound: search.best_primary,
            proof_id: Some(proof.proof_id.clone()),
        },
    };
    certificate.certificate_id = certificate.compute_id()?;
    Ok(CertifiedSplitArtifacts {
        instance: instance.clone(),
        certificate,
        proof,
    })
}

pub fn canonical_seat_split(k_parent: usize) -> Result<(usize, usize), CertifiedSplitError> {
    if k_parent < 2 {
        return Err(CertifiedSplitError::InvalidParentSeatCount);
    }
    let k_left = k_parent / 2;
    Ok((k_left, k_parent - k_left))
}

pub fn certified_split_unit_universe_hash(
    unit_ids: &[String],
) -> Result<String, CertifiedSplitError> {
    #[derive(Serialize)]
    struct Projection<'a> {
        unit_ids: &'a [String],
    }
    canonical_hash(&Projection { unit_ids })
}

pub fn canonical_orientation_rule(k_left: usize, k_right: usize) -> SplitOrientationRule {
    if k_left == k_right {
        SplitOrientationRule::EqualSeatsUnitZeroLeft
    } else {
        SplitOrientationRule::SeatOrderedFloorLeftCeilRight
    }
}

pub fn evaluate_certified_split_objective(
    instance: &CertifiedSplitInstance,
    assignment: &[u8],
) -> Result<CertifiedSplitPrimaryObjective, CertifiedSplitError> {
    // Connectivity is a feasibility condition checked separately by the exact oracle.
    instance.validate()?;
    validate_assignment(instance, assignment)?;
    Ok(objective_unchecked(instance, assignment))
}

fn objective_unchecked(
    instance: &CertifiedSplitInstance,
    assignment: &[u8],
) -> CertifiedSplitPrimaryObjective {
    let mut child_populations = [0_i128; 2];
    for (unit, &child) in assignment.iter().enumerate() {
        child_populations[child as usize] += i128::from(instance.populations[unit]);
    }
    let parent_population = child_populations[0] + child_populations[1];
    let left_deviation = (instance.k_parent as i128 * child_populations[0]
        - instance.k_left as i128 * parent_population)
        .unsigned_abs();
    let right_deviation = (instance.k_parent as i128 * child_populations[1]
        - instance.k_right as i128 * parent_population)
        .unsigned_abs();
    let weighted_boundary_cut = instance
        .edges
        .iter()
        .filter(|edge| assignment[edge.left] != assignment[edge.right])
        .map(|edge| edge.weight)
        .sum();
    CertifiedSplitPrimaryObjective {
        max_population_deviation_scaled: u64::try_from(left_deviation.max(right_deviation))
            .expect("validated split deviation must fit u64"),
        total_population_deviation_scaled: u64::try_from(left_deviation + right_deviation)
            .expect("validated total split deviation must fit u64"),
        weighted_boundary_cut,
    }
}

pub fn certified_split_children_connected(
    instance: &CertifiedSplitInstance,
    assignment: &[u8],
) -> Result<bool, CertifiedSplitError> {
    instance.validate()?;
    validate_assignment(instance, assignment)?;
    Ok(children_connected_unchecked(
        assignment,
        &adjacency(instance),
    ))
}

fn adjacency(instance: &CertifiedSplitInstance) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); instance.unit_ids.len()];
    for edge in &instance.edges {
        adjacency[edge.left].push(edge.right);
        adjacency[edge.right].push(edge.left);
    }
    adjacency
}

fn children_connected_unchecked(assignment: &[u8], adjacency: &[Vec<usize>]) -> bool {
    [0_u8, 1_u8].into_iter().all(|child| {
        let units = assignment
            .iter()
            .enumerate()
            .filter_map(|(unit, &label)| (label == child).then_some(unit))
            .collect::<Vec<_>>();
        let allowed = units.iter().copied().collect::<BTreeSet<_>>();
        let mut visited = BTreeSet::from([units[0]]);
        let mut queue = VecDeque::from([units[0]]);
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

fn validate_assignment(
    instance: &CertifiedSplitInstance,
    assignment: &[u8],
) -> Result<(), CertifiedSplitError> {
    if assignment.len() != instance.unit_ids.len() {
        return Err(CertifiedSplitError::AssignmentLength);
    }
    if assignment.iter().any(|&label| label > 1) {
        return Err(CertifiedSplitError::InvalidAssignmentLabel);
    }
    if !assignment.contains(&0) || !assignment.contains(&1) {
        return Err(CertifiedSplitError::EmptyChild);
    }
    let left_units = assignment.iter().filter(|&&label| label == 0).count();
    let right_units = assignment.len() - left_units;
    if left_units < instance.k_left || right_units < instance.k_right {
        return Err(CertifiedSplitError::InsufficientChildUnits {
            left_units,
            right_units,
            k_left: instance.k_left,
            k_right: instance.k_right,
        });
    }
    if instance.orientation_rule == SplitOrientationRule::EqualSeatsUnitZeroLeft
        && assignment[0] != 0
    {
        return Err(CertifiedSplitError::EqualSeatOrientation);
    }
    Ok(())
}

fn children_have_enough_units(instance: &CertifiedSplitInstance, assignment: &[u8]) -> bool {
    let left_units = assignment.iter().filter(|&&label| label == 0).count();
    left_units >= instance.k_left && assignment.len() - left_units >= instance.k_right
}

fn valid_sha256_id(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

pub(crate) fn canonical_hash<T: Serialize>(value: &T) -> Result<String, CertifiedSplitError> {
    let value = serde_json::to_value(value)
        .map_err(|error| CertifiedSplitError::Canonicalization(error.to_string()))?;
    let bytes = serde_json::to_vec(&canonicalize(value))
        .map_err(|error| CertifiedSplitError::Canonicalization(error.to_string()))?;
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

    fn split_instance(k_parent: usize, populations: Vec<i64>) -> CertifiedSplitInstance {
        let (k_left, k_right) = canonical_seat_split(k_parent).unwrap();
        let unit_ids = (0..populations.len())
            .map(|unit| format!("u{unit}"))
            .collect::<Vec<_>>();
        CertifiedSplitInstance {
            schema_version: CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations,
            edges: vec![ExactEdge {
                left: 0,
                right: 1,
                weight: 7,
            }],
            k_parent,
            k_left,
            k_right,
            orientation_rule: canonical_orientation_rule(k_left, k_right),
        }
    }

    fn complete_edges(unit_count: usize) -> Vec<ExactEdge> {
        (0..unit_count)
            .flat_map(|left| {
                ((left + 1)..unit_count).map(move |right| ExactEdge {
                    left,
                    right,
                    weight: 1,
                })
            })
            .collect()
    }

    #[test]
    fn california_schedule_uses_26_26_then_13_13_then_6_7() {
        assert_eq!(canonical_seat_split(52), Ok((26, 26)));
        assert_eq!(canonical_seat_split(26), Ok((13, 13)));
        assert_eq!(canonical_seat_split(13), Ok((6, 7)));
    }

    #[test]
    fn ratio_objective_is_zero_for_exact_two_to_three_split() {
        let mut instance = split_instance(5, vec![1, 1, 1, 1, 1]);
        instance.edges = complete_edges(5);
        let objective = evaluate_certified_split_objective(&instance, &[0, 0, 1, 1, 1]).unwrap();
        assert_eq!(objective.max_population_deviation_scaled, 0);
        assert_eq!(objective.total_population_deviation_scaled, 0);
        assert_eq!(objective.weighted_boundary_cut, 6);
    }

    #[test]
    fn odd_split_orientation_allows_unit_zero_on_larger_child() {
        let mut instance = split_instance(5, vec![4, 1, 3, 1, 1]);
        instance.edges = complete_edges(5);
        let objective = evaluate_certified_split_objective(&instance, &[1, 0, 0, 1, 1]).unwrap();
        assert_eq!(objective.max_population_deviation_scaled, 0);
    }

    #[test]
    fn equal_split_requires_unit_zero_left() {
        let instance = split_instance(2, vec![1, 1]);
        assert_eq!(
            evaluate_certified_split_objective(&instance, &[1, 0]),
            Err(CertifiedSplitError::EqualSeatOrientation)
        );
    }

    #[test]
    fn rejects_noncanonical_seat_split() {
        let mut instance = split_instance(5, vec![2, 3]);
        instance.k_left = 1;
        instance.k_right = 4;
        assert!(matches!(
            instance.validate(),
            Err(CertifiedSplitError::NonCanonicalSeatSplit { .. })
        ));
    }

    #[test]
    fn validates_root_child_link_and_unit_universe_identity() {
        let mut instance = split_instance(5, vec![2, 3]);
        instance.unit_universe_hash = format!("sha256:{}", "0".repeat(64));
        assert!(matches!(
            instance.validate(),
            Err(CertifiedSplitError::UnitUniverseHashMismatch { .. })
        ));

        let mut child = split_instance(3, vec![1, 2]);
        child.node_path = "0".to_string();
        assert_eq!(
            child.validate(),
            Err(CertifiedSplitError::ParentCertificateLink)
        );
        child.parent_certificate_id = Some(format!("sha256:{}", "a".repeat(64)));
        assert_eq!(child.validate(), Ok(()));
    }

    #[test]
    fn rejects_population_objective_overflow() {
        let instance = split_instance(2, vec![i64::MAX, i64::MAX]);
        assert_eq!(
            instance.validate(),
            Err(CertifiedSplitError::NumericOverflow)
        );
    }

    #[test]
    fn connectivity_checks_both_children() {
        let mut instance = split_instance(2, vec![1, 1, 1, 1]);
        instance.edges = vec![
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
        ];
        assert_eq!(
            certified_split_children_connected(&instance, &[0, 1, 0, 1]),
            Ok(false)
        );
        assert_eq!(
            certified_split_children_connected(&instance, &[0, 0, 1, 1]),
            Ok(true)
        );
    }

    #[test]
    fn instance_and_certificate_ids_are_stable() {
        let instance = split_instance(5, vec![1, 1, 1, 1, 1]);
        assert_eq!(instance.hash().unwrap(), instance.hash().unwrap());
        let mut certificate = CertifiedSplitCertificate {
            schema_version: CERTIFIED_SPLIT_CERTIFICATE_SCHEMA_VERSION.to_string(),
            certificate_id: String::new(),
            instance_hash: instance.hash().unwrap(),
            model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
            result: CertifiedSplitResult::Infeasible,
            proof: CertifiedSplitProofSummary {
                proof_kind: "contract-fixture".to_string(),
                feasible_assignments: None,
                primary_objective_ties: None,
                lower_bound: None,
                proof_id: None,
            },
        };
        certificate.certificate_id = certificate.compute_id().unwrap();
        assert_eq!(
            certificate.certificate_id,
            certificate.compute_id().unwrap()
        );
    }

    #[test]
    fn bounded_oracle_selects_ratio_correct_odd_orientation() {
        let mut instance = split_instance(5, vec![4, 1, 3, 1, 1]);
        instance.edges = complete_edges(5);
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let CertifiedSplitResult::Optimal {
            assignment,
            objective,
        } = &artifacts.certificate.result
        else {
            panic!("two connected units must be feasible");
        };
        assert_eq!(assignment, &[1, 0, 0, 1, 1]);
        assert_eq!(objective.primary.max_population_deviation_scaled, 0);
        assert_eq!(artifacts.proof.candidate_count, 30);
        assert_eq!(
            verify_certified_split_bounded(&instance, &artifacts.certificate, &artifacts.proof),
            Ok(())
        );
    }

    #[test]
    fn bounded_equal_split_uses_reduced_label_space() {
        let mut instance = split_instance(2, vec![1, 1, 1, 1]);
        instance.edges = vec![
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
        ];
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        assert_eq!(artifacts.proof.candidate_count, 7);
        assert_eq!(artifacts.proof.feasible_count, 3);
        let CertifiedSplitResult::Optimal { assignment, .. } = artifacts.certificate.result else {
            panic!("path split must be feasible");
        };
        assert_eq!(assignment, vec![0, 0, 1, 1]);
    }

    #[test]
    fn bounded_oracle_emits_exact_infeasibility() {
        let mut instance = split_instance(5, vec![2, 3, 0]);
        instance.edges.clear();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        assert_eq!(
            artifacts.certificate.result,
            CertifiedSplitResult::Infeasible
        );
        assert_eq!(artifacts.proof.candidate_count, 6);
        assert_eq!(artifacts.proof.feasible_count, 0);
        assert_eq!(
            verify_certified_split_bounded(&instance, &artifacts.certificate, &artifacts.proof),
            Ok(())
        );
    }

    #[test]
    fn bounded_verifier_rejects_false_ratio_optimum() {
        let mut instance = split_instance(5, vec![4, 1, 3, 1, 1]);
        instance.edges = complete_edges(5);
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let mut certificate = artifacts.certificate;
        if let CertifiedSplitResult::Optimal { objective, .. } = &mut certificate.result {
            objective.primary.max_population_deviation_scaled += 1;
        }
        certificate.certificate_id = certificate.compute_id().unwrap();
        assert_eq!(
            verify_certified_split_bounded(&instance, &certificate, &artifacts.proof),
            Err(CertifiedSplitError::ResultMismatch)
        );
    }

    #[test]
    fn bounded_verifier_rejects_transcript_tamper() {
        let mut instance = split_instance(5, vec![4, 1, 3, 1, 1]);
        instance.edges = complete_edges(5);
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let mut proof = artifacts.proof;
        proof.search_commitment = format!("sha256:{}", "0".repeat(64));
        proof.proof_id = proof.compute_id().unwrap();
        assert_eq!(
            verify_certified_split_bounded(&instance, &artifacts.certificate, &proof),
            Err(CertifiedSplitError::ProofMismatch)
        );
    }
}
