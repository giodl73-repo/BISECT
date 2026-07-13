use crate::certified_split::canonical_hash;
use crate::{
    certified_split_children_connected, evaluate_certified_split_objective, CertifiedSplitError,
    CertifiedSplitInstance, CertifiedSplitObjective, ConnectivityCut,
    CERTIFIED_SPLIT_ENUMERATION_LIMIT,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const CERTIFIED_DISCOVERY_SCHEMA_VERSION: &str = "certified-split-discovery-v1";
pub const CERTIFIED_PROOF_REQUEST_SCHEMA_VERSION: &str = "certified-split-proof-request-v5";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedSplitDiscovery {
    pub schema_version: String,
    pub discovery_id: String,
    pub instance_hash: String,
    pub solver_name: String,
    pub solver_version: Option<String>,
    pub method: String,
    pub objective: CertifiedSplitObjective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CertifiedDecisionStage {
    PopulationLowerBound,
    BoundaryLowerBound,
    CanonicalTieBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CertifiedDecisionStatus {
    UnsatProofRequired,
    SatCounterexampleExists,
    ProofRequiredUnclassified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedProofRequest {
    pub schema_version: String,
    pub request_id: String,
    pub instance_hash: String,
    pub discovery_id: String,
    pub stage: CertifiedDecisionStage,
    pub connectivity_encoding: String,
    pub exact_right_population: Option<i64>,
    pub status: CertifiedDecisionStatus,
    pub opb_sha256: String,
    pub variable_count: usize,
    pub constraint_count: usize,
    pub proof_format: String,
    pub proof_status: String,
    pub solver_command_template: String,
    pub claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertifiedOpbArtifact {
    pub request: CertifiedProofRequest,
    pub opb: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RegionalPopulationRelation {
    Equal,
    AtLeast,
    AtMost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionalPopulationConstraint {
    pub region_id: String,
    pub units: Vec<usize>,
    pub relation: RegionalPopulationRelation,
    pub population: i64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProofBackendError {
    #[error(transparent)]
    Split(#[from] CertifiedSplitError),
    #[error("unsupported discovery schema: {0}")]
    DiscoverySchema(String),
    #[error("discovery instance hash mismatch")]
    DiscoveryInstanceHash,
    #[error("discovery id mismatch: expected {expected}, found {found}")]
    DiscoveryIdMismatch { expected: String, found: String },
    #[error("discovery assignment or objective does not match the split instance")]
    DiscoveryMismatch,
    #[error("invalid connectivity cut: {0}")]
    ConnectivityCut(String),
}

impl CertifiedSplitDiscovery {
    pub fn compute_id(&self) -> Result<String, CertifiedSplitError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            schema_version: &'a str,
            instance_hash: &'a str,
            solver_name: &'a str,
            solver_version: &'a Option<String>,
            method: &'a str,
            objective: &'a CertifiedSplitObjective,
        }
        canonical_hash(&Projection {
            schema_version: &self.schema_version,
            instance_hash: &self.instance_hash,
            solver_name: &self.solver_name,
            solver_version: &self.solver_version,
            method: &self.method,
            objective: &self.objective,
        })
    }
}

impl CertifiedProofRequest {
    pub fn compute_id(&self) -> Result<String, CertifiedSplitError> {
        #[derive(Serialize)]
        struct Projection<'a> {
            schema_version: &'a str,
            instance_hash: &'a str,
            discovery_id: &'a str,
            stage: CertifiedDecisionStage,
            connectivity_encoding: &'a str,
            exact_right_population: &'a Option<i64>,
            status: CertifiedDecisionStatus,
            opb_sha256: &'a str,
            variable_count: usize,
            constraint_count: usize,
            proof_format: &'a str,
            proof_status: &'a str,
            solver_command_template: &'a str,
            claim: &'a str,
        }
        canonical_hash(&Projection {
            schema_version: &self.schema_version,
            instance_hash: &self.instance_hash,
            discovery_id: &self.discovery_id,
            stage: self.stage,
            connectivity_encoding: &self.connectivity_encoding,
            exact_right_population: &self.exact_right_population,
            status: self.status,
            opb_sha256: &self.opb_sha256,
            variable_count: self.variable_count,
            constraint_count: self.constraint_count,
            proof_format: &self.proof_format,
            proof_status: &self.proof_status,
            solver_command_template: &self.solver_command_template,
            claim: &self.claim,
        })
    }
}

pub fn certified_split_discovery(
    instance: &CertifiedSplitInstance,
    solver_name: impl Into<String>,
    solver_version: Option<String>,
    method: impl Into<String>,
    assignment: Vec<u8>,
) -> Result<CertifiedSplitDiscovery, ProofBackendError> {
    instance.validate()?;
    if !certified_split_children_connected(instance, &assignment)? {
        return Err(ProofBackendError::DiscoveryMismatch);
    }
    let primary = evaluate_certified_split_objective(instance, &assignment)?;
    let mut discovery = CertifiedSplitDiscovery {
        schema_version: CERTIFIED_DISCOVERY_SCHEMA_VERSION.to_string(),
        discovery_id: String::new(),
        instance_hash: instance.hash()?,
        solver_name: solver_name.into(),
        solver_version,
        method: method.into(),
        objective: CertifiedSplitObjective {
            primary,
            canonical_assignment: assignment,
        },
    };
    discovery.discovery_id = discovery.compute_id()?;
    Ok(discovery)
}

pub fn compile_certified_split_proof_requests(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
) -> Result<Vec<CertifiedOpbArtifact>, ProofBackendError> {
    validate_discovery(instance, discovery)?;
    if instance.unit_ids.len() > CERTIFIED_SPLIT_ENUMERATION_LIMIT {
        return Err(CertifiedSplitError::EnumerationLimit {
            found: instance.unit_ids.len(),
            limit: CERTIFIED_SPLIT_ENUMERATION_LIMIT,
        }
        .into());
    }
    compile_requests(instance, discovery, false)
}

pub fn compile_certified_split_compact_proof_requests(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
) -> Result<Vec<CertifiedOpbArtifact>, ProofBackendError> {
    validate_discovery(instance, discovery)?;
    compile_requests(instance, discovery, true)
}

pub fn compile_certified_split_compact_proof_request(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    stage: CertifiedDecisionStage,
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    validate_discovery(instance, discovery)?;
    compile_stage(instance, discovery, stage, true, None)
}

pub fn compile_certified_split_compact_boundary_branch(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    exact_right_population: i64,
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    validate_discovery(instance, discovery)?;
    validate_boundary_branch(instance, discovery, exact_right_population)?;
    compile_stage(
        instance,
        discovery,
        CertifiedDecisionStage::BoundaryLowerBound,
        true,
        Some(exact_right_population),
    )
}

pub fn compile_certified_split_cutset_boundary_branch(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    exact_right_population: i64,
    cuts: &[ConnectivityCut],
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    compile_certified_split_cutset_boundary_branch_with_fixes(
        instance,
        discovery,
        exact_right_population,
        cuts,
        &vec![None; instance.unit_ids.len()],
    )
}

pub fn compile_certified_split_cutset_boundary_branch_with_fixes(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    exact_right_population: i64,
    cuts: &[ConnectivityCut],
    fixed_assignments: &[Option<u8>],
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    validate_discovery(instance, discovery)?;
    validate_boundary_branch(instance, discovery, exact_right_population)?;
    if fixed_assignments.len() != instance.unit_ids.len()
        || fixed_assignments.iter().flatten().any(|&label| label > 1)
    {
        return Err(ProofBackendError::ConnectivityCut(
            "fixed assignments must contain one optional binary label per unit".to_string(),
        ));
    }
    let variables = CutsetVariables::new(instance);
    let mut constraints = cutset_base_constraints(instance, &variables);
    for (unit, label) in fixed_assignments.iter().enumerate() {
        if let Some(label) = label {
            constraints.push(linear_constraint([(1, x(unit))], "=", i128::from(*label)));
        }
    }
    for cut in cuts {
        add_connectivity_cut(instance, &variables, cut, &mut constraints)?;
    }
    add_exact_right_population(instance, exact_right_population, &mut constraints);
    if discovery.objective.primary.weighted_boundary_cut == 0 {
        constraints.push(contradiction());
    } else {
        add_cut_threshold(
            instance,
            discovery.objective.primary.weighted_boundary_cut - 1,
            &mut constraints,
        );
    }
    finalize_artifact(
        instance,
        discovery,
        CertifiedDecisionStage::BoundaryLowerBound,
        format!(
            "cutset-v1;cuts={};fixed={}",
            cuts.len(),
            fixed_assignments.iter().flatten().count()
        ),
        Some(exact_right_population),
        variables.variable_count,
        constraints,
        CertifiedDecisionStatus::ProofRequiredUnclassified,
    )
}

pub fn compile_certified_split_reduced_cutset_boundary_branch(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    exact_right_population: i64,
    cuts: &[ConnectivityCut],
    fixed_assignments: &[Option<u8>],
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    validate_discovery(instance, discovery)?;
    validate_boundary_branch(instance, discovery, exact_right_population)?;
    validate_fixed_cores(instance, fixed_assignments)?;
    let variables = ReducedCutsetVariables::new(instance, fixed_assignments);
    let (mut constraints, fixed_cut) =
        reduced_cutset_base_constraints(instance, fixed_assignments, &variables)?;
    for cut in cuts {
        add_reduced_connectivity_cut(
            instance,
            fixed_assignments,
            &variables,
            cut,
            &mut constraints,
        )?;
    }
    add_reduced_exact_right_population(
        instance,
        fixed_assignments,
        &variables,
        exact_right_population,
        &mut constraints,
    )?;
    let threshold = discovery
        .objective
        .primary
        .weighted_boundary_cut
        .checked_sub(1)
        .ok_or(ProofBackendError::DiscoveryMismatch)?;
    if fixed_cut > threshold {
        constraints.push(contradiction());
    } else {
        add_reduced_cut_threshold(
            instance,
            &variables,
            threshold - fixed_cut,
            &mut constraints,
        );
    }
    finalize_artifact(
        instance,
        discovery,
        CertifiedDecisionStage::BoundaryLowerBound,
        format!(
            "cutset-reduced-v1;active={};cuts={};fixed={}",
            variables.active_units.len(),
            cuts.len(),
            fixed_assignments.iter().flatten().count()
        ),
        Some(exact_right_population),
        variables.variable_count,
        constraints,
        CertifiedDecisionStatus::ProofRequiredUnclassified,
    )
}

pub fn compile_certified_split_reduced_boundary_relaxation(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    exact_right_population: i64,
    fixed_assignments: &[Option<u8>],
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    validate_discovery(instance, discovery)?;
    validate_boundary_branch(instance, discovery, exact_right_population)?;
    if fixed_assignments.len() != instance.unit_ids.len()
        || fixed_assignments.iter().flatten().any(|&label| label > 1)
        || (instance.k_left == instance.k_right && fixed_assignments[0] == Some(1))
    {
        return Err(ProofBackendError::ConnectivityCut(
            "invalid reduced relaxation fixed assignments".to_string(),
        ));
    }
    let variables = ReducedCutsetVariables::new(instance, fixed_assignments);
    let (mut constraints, fixed_cut) =
        reduced_cutset_base_constraints(instance, fixed_assignments, &variables)?;
    if instance.k_left == instance.k_right && fixed_assignments[0].is_none() {
        constraints.push(linear_constraint(
            [(
                1,
                variables.assignment(0).ok_or_else(|| {
                    ProofBackendError::ConnectivityCut(
                        "missing canonical assignment variable".to_string(),
                    )
                })?,
            )],
            "=",
            0,
        ));
    }
    add_reduced_exact_right_population(
        instance,
        fixed_assignments,
        &variables,
        exact_right_population,
        &mut constraints,
    )?;
    let threshold = discovery
        .objective
        .primary
        .weighted_boundary_cut
        .checked_sub(1)
        .ok_or(ProofBackendError::DiscoveryMismatch)?;
    if fixed_cut > threshold {
        constraints.push(contradiction());
    } else {
        add_reduced_cut_threshold(
            instance,
            &variables,
            threshold - fixed_cut,
            &mut constraints,
        );
    }
    finalize_artifact(
        instance,
        discovery,
        CertifiedDecisionStage::BoundaryLowerBound,
        format!(
            "connectivity-relaxation-reduced-v1;active={};fixed={}",
            variables.active_units.len(),
            fixed_assignments.iter().flatten().count()
        ),
        Some(exact_right_population),
        variables.variable_count,
        constraints,
        CertifiedDecisionStatus::ProofRequiredUnclassified,
    )
}

pub fn compile_certified_split_boundary_relaxation(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    exact_right_population: i64,
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    validate_discovery(instance, discovery)?;
    validate_boundary_branch(instance, discovery, exact_right_population)?;
    let mut constraints = assignment_and_cut_constraints(instance);
    add_exact_right_population(instance, exact_right_population, &mut constraints);
    if discovery.objective.primary.weighted_boundary_cut == 0 {
        constraints.push(contradiction());
    } else {
        add_cut_threshold(
            instance,
            discovery.objective.primary.weighted_boundary_cut - 1,
            &mut constraints,
        );
    }
    finalize_artifact(
        instance,
        discovery,
        CertifiedDecisionStage::BoundaryLowerBound,
        "connectivity-relaxation-v1".to_string(),
        Some(exact_right_population),
        instance.unit_ids.len() + instance.edges.len(),
        constraints,
        CertifiedDecisionStatus::ProofRequiredUnclassified,
    )
}

pub fn compile_certified_split_boundary_relaxation_outside_core(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    exact_right_population: i64,
    fixed_assignments: &[Option<u8>],
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    validate_discovery(instance, discovery)?;
    validate_boundary_branch(instance, discovery, exact_right_population)?;
    if fixed_assignments.len() != instance.unit_ids.len()
        || fixed_assignments.iter().flatten().any(|&label| label > 1)
    {
        return Err(ProofBackendError::ConnectivityCut(
            "core assignments must contain one optional binary label per unit".to_string(),
        ));
    }

    let mut constraints = assignment_and_cut_constraints(instance);
    add_exact_right_population(instance, exact_right_population, &mut constraints);
    add_cut_threshold(
        instance,
        discovery
            .objective
            .primary
            .weighted_boundary_cut
            .checked_sub(1)
            .ok_or(ProofBackendError::DiscoveryMismatch)?,
        &mut constraints,
    );
    let fixed_right_count = fixed_assignments
        .iter()
        .filter(|&&label| label == Some(1))
        .count() as i128;
    constraints.push(linear_constraint(
        fixed_assignments
            .iter()
            .enumerate()
            .filter_map(|(unit, &label)| match label {
                Some(0) => Some((1, x(unit))),
                Some(1) => Some((-1, x(unit))),
                None => None,
                _ => unreachable!(),
            }),
        ">=",
        1 - fixed_right_count,
    ));
    finalize_artifact(
        instance,
        discovery,
        CertifiedDecisionStage::BoundaryLowerBound,
        format!(
            "connectivity-relaxation-outside-core-v1;fixed={}",
            fixed_assignments.iter().flatten().count()
        ),
        Some(exact_right_population),
        instance.unit_ids.len() + instance.edges.len(),
        constraints,
        CertifiedDecisionStatus::ProofRequiredUnclassified,
    )
}

pub fn compile_certified_split_regional_boundary_relaxation(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    exact_right_population: i64,
    regional_constraints: &[RegionalPopulationConstraint],
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    validate_discovery(instance, discovery)?;
    validate_boundary_branch(instance, discovery, exact_right_population)?;
    let mut constraints = assignment_and_cut_constraints(instance);
    add_exact_right_population(instance, exact_right_population, &mut constraints);
    add_cut_threshold(
        instance,
        discovery
            .objective
            .primary
            .weighted_boundary_cut
            .checked_sub(1)
            .ok_or(ProofBackendError::DiscoveryMismatch)?,
        &mut constraints,
    );
    for regional in regional_constraints {
        if regional.population < 0 {
            return Err(ProofBackendError::ConnectivityCut(format!(
                "region {} has negative population bound",
                regional.region_id
            )));
        }
        let units = regional.units.iter().copied().collect::<BTreeSet<_>>();
        if units.is_empty()
            || units.len() != regional.units.len()
            || units.iter().any(|&unit| unit >= instance.unit_ids.len())
        {
            return Err(ProofBackendError::ConnectivityCut(format!(
                "region {} has invalid unit membership",
                regional.region_id
            )));
        }
        let relation = match regional.relation {
            RegionalPopulationRelation::Equal => "=",
            RegionalPopulationRelation::AtLeast => ">=",
            RegionalPopulationRelation::AtMost => "<=",
        };
        constraints.push(linear_constraint(
            units
                .into_iter()
                .map(|unit| (i128::from(instance.populations[unit]), x(unit))),
            relation,
            i128::from(regional.population),
        ));
    }
    finalize_artifact(
        instance,
        discovery,
        CertifiedDecisionStage::BoundaryLowerBound,
        format!(
            "connectivity-relaxation-regional-v1;constraints={}",
            regional_constraints.len()
        ),
        Some(exact_right_population),
        instance.unit_ids.len() + instance.edges.len(),
        constraints,
        CertifiedDecisionStatus::ProofRequiredUnclassified,
    )
}

fn validate_boundary_branch(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    exact_right_population: i64,
) -> Result<(), ProofBackendError> {
    let total_population = instance.populations.iter().sum::<i64>();
    let branch_deviation = (instance.k_parent as i128 * i128::from(exact_right_population)
        - instance.k_right as i128 * i128::from(total_population))
    .unsigned_abs();
    if exact_right_population < 0
        || exact_right_population > total_population
        || branch_deviation
            > u128::from(discovery.objective.primary.max_population_deviation_scaled)
    {
        return Err(ProofBackendError::DiscoveryMismatch);
    }
    Ok(())
}

fn compile_requests(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    compact_connectivity: bool,
) -> Result<Vec<CertifiedOpbArtifact>, ProofBackendError> {
    [
        CertifiedDecisionStage::PopulationLowerBound,
        CertifiedDecisionStage::BoundaryLowerBound,
        CertifiedDecisionStage::CanonicalTieBreak,
    ]
    .into_iter()
    .map(|stage| compile_stage(instance, discovery, stage, compact_connectivity, None))
    .collect()
}

fn validate_discovery(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
) -> Result<(), ProofBackendError> {
    instance.validate()?;
    if discovery.schema_version != CERTIFIED_DISCOVERY_SCHEMA_VERSION {
        return Err(ProofBackendError::DiscoverySchema(
            discovery.schema_version.clone(),
        ));
    }
    if discovery.instance_hash != instance.hash()? {
        return Err(ProofBackendError::DiscoveryInstanceHash);
    }
    let expected_id = discovery.compute_id()?;
    if discovery.discovery_id != expected_id {
        return Err(ProofBackendError::DiscoveryIdMismatch {
            expected: expected_id,
            found: discovery.discovery_id.clone(),
        });
    }
    if discovery.objective.canonical_assignment.len() != instance.unit_ids.len()
        || !certified_split_children_connected(instance, &discovery.objective.canonical_assignment)?
        || evaluate_certified_split_objective(instance, &discovery.objective.canonical_assignment)?
            != discovery.objective.primary
    {
        return Err(ProofBackendError::DiscoveryMismatch);
    }
    Ok(())
}

fn compile_stage(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    stage: CertifiedDecisionStage,
    compact_connectivity: bool,
    exact_right_population: Option<i64>,
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    let (mut constraints, mut variable_count, connectivity_encoding) = if compact_connectivity {
        let variables = CompactVariables::new(instance);
        (
            compact_base_constraints(instance, &variables),
            variables.variable_count,
            "parent-depth-v3",
        )
    } else {
        (
            base_constraints(instance),
            instance.unit_ids.len() + instance.edges.len(),
            "static-nogoods-v1",
        )
    };
    let incumbent = &discovery.objective;
    match stage {
        CertifiedDecisionStage::PopulationLowerBound => {
            if incumbent.primary.max_population_deviation_scaled == 0 {
                constraints.push(contradiction());
            } else {
                add_population_threshold(
                    instance,
                    incumbent.primary.max_population_deviation_scaled - 1,
                    &mut constraints,
                );
            }
        }
        CertifiedDecisionStage::BoundaryLowerBound => {
            if let Some(population) = exact_right_population {
                add_exact_right_population(instance, population, &mut constraints);
            } else {
                add_population_threshold(
                    instance,
                    incumbent.primary.max_population_deviation_scaled,
                    &mut constraints,
                );
            }
            if incumbent.primary.weighted_boundary_cut == 0 {
                constraints.push(contradiction());
            } else {
                add_cut_threshold(
                    instance,
                    incumbent.primary.weighted_boundary_cut - 1,
                    &mut constraints,
                );
            }
        }
        CertifiedDecisionStage::CanonicalTieBreak => {
            add_population_threshold(
                instance,
                incumbent.primary.max_population_deviation_scaled,
                &mut constraints,
            );
            add_cut_threshold(
                instance,
                incumbent.primary.weighted_boundary_cut,
                &mut constraints,
            );
            if compact_connectivity {
                add_compact_lex_predecessor(
                    &incumbent.canonical_assignment,
                    &mut constraints,
                    &mut variable_count,
                );
            } else {
                add_lex_predecessor(&incumbent.canonical_assignment, &mut constraints);
            }
        }
    }
    let status = if compact_connectivity {
        CertifiedDecisionStatus::ProofRequiredUnclassified
    } else {
        let has_counterexample = decision_has_counterexample(instance, incumbent, stage)?;
        if has_counterexample {
            CertifiedDecisionStatus::SatCounterexampleExists
        } else {
            CertifiedDecisionStatus::UnsatProofRequired
        }
    };
    finalize_artifact(
        instance,
        discovery,
        stage,
        connectivity_encoding.to_string(),
        exact_right_population,
        variable_count,
        constraints,
        status,
    )
}

#[allow(clippy::too_many_arguments)]
fn finalize_artifact(
    instance: &CertifiedSplitInstance,
    discovery: &CertifiedSplitDiscovery,
    stage: CertifiedDecisionStage,
    connectivity_encoding: String,
    exact_right_population: Option<i64>,
    variable_count: usize,
    constraints: Vec<String>,
    status: CertifiedDecisionStatus,
) -> Result<CertifiedOpbArtifact, ProofBackendError> {
    let incumbent = &discovery.objective;
    let equal_count = constraints
        .iter()
        .filter(|constraint| constraint.contains(" = "))
        .count();
    let mut opb = format!(
        "* #variable= {variable_count} #constraint= {} #equal= {equal_count} intsize= 128\n",
        constraints.len(),
    );
    opb.push_str(&format!(
        "* instance_hash={} stage={stage:?}\n",
        instance.hash()?
    ));
    for constraint in constraints {
        opb.push_str(&constraint);
        opb.push('\n');
    }
    let opb_sha256 = format!("sha256:{:x}", Sha256::digest(opb.as_bytes()));
    let claim = stage_claim(incumbent, stage);
    let mut request = CertifiedProofRequest {
        schema_version: CERTIFIED_PROOF_REQUEST_SCHEMA_VERSION.to_string(),
        request_id: String::new(),
        instance_hash: instance.hash()?,
        discovery_id: discovery.discovery_id.clone(),
        stage,
        connectivity_encoding,
        exact_right_population,
        status,
        opb_sha256,
        variable_count,
        constraint_count: opb.lines().filter(|line| line.ends_with(';')).count(),
        proof_format: "veripb".to_string(),
        proof_status: "not-generated".to_string(),
        solver_command_template: "roundingsat --proof-log={proof} {opb}".to_string(),
        claim,
    };
    request.request_id = request.compute_id()?;
    Ok(CertifiedOpbArtifact { request, opb })
}

fn base_constraints(instance: &CertifiedSplitInstance) -> Vec<String> {
    let mut constraints = assignment_and_cut_constraints(instance);
    add_connectivity_nogoods(instance, &mut constraints);
    constraints
}

fn assignment_and_cut_constraints(instance: &CertifiedSplitInstance) -> Vec<String> {
    let unit_count = instance.unit_ids.len();
    let mut constraints = Vec::new();
    constraints.push(linear_constraint(
        (0..unit_count).map(|unit| (1_i128, x(unit))),
        ">=",
        instance.k_right as i128,
    ));
    constraints.push(linear_constraint(
        (0..unit_count).map(|unit| (-1_i128, x(unit))),
        ">=",
        instance.k_left as i128 - unit_count as i128,
    ));
    if instance.k_left == instance.k_right {
        constraints.push(linear_constraint([(1, x(0))], "=", 0));
    }
    for (edge_index, edge) in instance.edges.iter().enumerate() {
        constraints.push(linear_constraint(
            [
                (1, y(edge_index, unit_count)),
                (-1, x(edge.left)),
                (1, x(edge.right)),
            ],
            ">=",
            0,
        ));
        constraints.push(linear_constraint(
            [
                (-1, y(edge_index, unit_count)),
                (1, x(edge.left)),
                (1, x(edge.right)),
            ],
            ">=",
            0,
        ));
        constraints.push(linear_constraint(
            [
                (-1, y(edge_index, unit_count)),
                (-1, x(edge.left)),
                (-1, x(edge.right)),
            ],
            ">=",
            -2,
        ));
        constraints.push(linear_constraint(
            [
                (1, y(edge_index, unit_count)),
                (1, x(edge.left)),
                (-1, x(edge.right)),
            ],
            ">=",
            0,
        ));
    }
    constraints
}

struct ReducedCutsetVariables {
    active_units: Vec<usize>,
    assignment_index: Vec<Option<usize>>,
    active_edges: Vec<usize>,
    edge_index: Vec<Option<usize>>,
    variable_count: usize,
}

impl ReducedCutsetVariables {
    fn new(instance: &CertifiedSplitInstance, fixed_assignments: &[Option<u8>]) -> Self {
        let active_units = fixed_assignments
            .iter()
            .enumerate()
            .filter_map(|(unit, label)| label.is_none().then_some(unit))
            .collect::<Vec<_>>();
        let mut assignment_index = vec![None; instance.unit_ids.len()];
        for (index, &unit) in active_units.iter().enumerate() {
            assignment_index[unit] = Some(index);
        }
        let active_edges = instance
            .edges
            .iter()
            .enumerate()
            .filter_map(|(edge, value)| {
                (fixed_assignments[value.left].is_none()
                    || fixed_assignments[value.right].is_none())
                .then_some(edge)
            })
            .collect::<Vec<_>>();
        let mut edge_index = vec![None; instance.edges.len()];
        for (index, &edge) in active_edges.iter().enumerate() {
            edge_index[edge] = Some(active_units.len() + index);
        }
        let variable_count = active_units.len() + active_edges.len();
        Self {
            active_units,
            assignment_index,
            active_edges,
            edge_index,
            variable_count,
        }
    }

    fn assignment(&self, unit: usize) -> Option<String> {
        self.assignment_index[unit].map(variable)
    }

    fn cut(&self, edge: usize) -> Option<String> {
        self.edge_index[edge].map(variable)
    }
}

fn validate_fixed_cores(
    instance: &CertifiedSplitInstance,
    fixed_assignments: &[Option<u8>],
) -> Result<(), ProofBackendError> {
    if fixed_assignments.len() != instance.unit_ids.len()
        || fixed_assignments.iter().flatten().any(|&label| label > 1)
    {
        return Err(ProofBackendError::ConnectivityCut(
            "fixed assignments must contain one optional binary label per unit".to_string(),
        ));
    }
    if instance.k_left == instance.k_right && fixed_assignments[0] != Some(0) {
        return Err(ProofBackendError::ConnectivityCut(
            "equal-seat reduced branches must fix canonical unit 0 left".to_string(),
        ));
    }
    for label in [0_u8, 1_u8] {
        let core = fixed_assignments
            .iter()
            .enumerate()
            .filter_map(|(unit, value)| (*value == Some(label)).then_some(unit))
            .collect::<Vec<_>>();
        if core.is_empty() {
            return Err(ProofBackendError::ConnectivityCut(format!(
                "child {label} has no fixed core"
            )));
        }
    }
    Ok(())
}

fn reduced_cutset_base_constraints(
    instance: &CertifiedSplitInstance,
    fixed_assignments: &[Option<u8>],
    variables: &ReducedCutsetVariables,
) -> Result<(Vec<String>, u64), ProofBackendError> {
    let mut constraints = Vec::new();
    let mut fixed_cut = 0_u64;
    for (edge_index, edge) in instance.edges.iter().enumerate() {
        match (fixed_assignments[edge.left], fixed_assignments[edge.right]) {
            (Some(left), Some(right)) => {
                if left != right {
                    fixed_cut = fixed_cut
                        .checked_add(edge.weight)
                        .ok_or(ProofBackendError::DiscoveryMismatch)?;
                }
            }
            (None, None) => {
                let cut = variables.cut(edge_index).ok_or_else(|| {
                    ProofBackendError::ConnectivityCut("missing active edge variable".to_string())
                })?;
                let left = variables.assignment(edge.left).ok_or_else(|| {
                    ProofBackendError::ConnectivityCut(
                        "missing active assignment variable".to_string(),
                    )
                })?;
                let right = variables.assignment(edge.right).ok_or_else(|| {
                    ProofBackendError::ConnectivityCut(
                        "missing active assignment variable".to_string(),
                    )
                })?;
                constraints.push(linear_constraint(
                    [(1, cut.clone()), (-1, left.clone()), (1, right.clone())],
                    ">=",
                    0,
                ));
                constraints.push(linear_constraint(
                    [(-1, cut.clone()), (1, left.clone()), (1, right.clone())],
                    ">=",
                    0,
                ));
                constraints.push(linear_constraint(
                    [(-1, cut.clone()), (-1, left.clone()), (-1, right.clone())],
                    ">=",
                    -2,
                ));
                constraints.push(linear_constraint(
                    [(1, cut), (1, left), (-1, right)],
                    ">=",
                    0,
                ));
            }
            (None, Some(label)) => {
                let cut = variables.cut(edge_index).ok_or_else(|| {
                    ProofBackendError::ConnectivityCut("missing active edge variable".to_string())
                })?;
                let active = variables.assignment(edge.left).ok_or_else(|| {
                    ProofBackendError::ConnectivityCut(
                        "missing active assignment variable".to_string(),
                    )
                })?;
                constraints.push(if label == 0 {
                    linear_constraint([(1, cut), (-1, active)], "=", 0)
                } else {
                    linear_constraint([(1, cut), (1, active)], "=", 1)
                });
            }
            (Some(label), None) => {
                let cut = variables.cut(edge_index).ok_or_else(|| {
                    ProofBackendError::ConnectivityCut("missing active edge variable".to_string())
                })?;
                let active = variables.assignment(edge.right).ok_or_else(|| {
                    ProofBackendError::ConnectivityCut(
                        "missing active assignment variable".to_string(),
                    )
                })?;
                constraints.push(if label == 0 {
                    linear_constraint([(1, cut), (-1, active)], "=", 0)
                } else {
                    linear_constraint([(1, cut), (1, active)], "=", 1)
                });
            }
        }
    }
    Ok((constraints, fixed_cut))
}

fn add_reduced_exact_right_population(
    instance: &CertifiedSplitInstance,
    fixed_assignments: &[Option<u8>],
    variables: &ReducedCutsetVariables,
    exact_right_population: i64,
    constraints: &mut Vec<String>,
) -> Result<(), ProofBackendError> {
    let fixed_right_population = instance
        .populations
        .iter()
        .zip(fixed_assignments)
        .filter_map(|(&population, &label)| (label == Some(1)).then_some(population))
        .sum::<i64>();
    let remaining = exact_right_population - fixed_right_population;
    if remaining < 0 {
        constraints.push(contradiction());
        return Ok(());
    }
    let terms = variables
        .active_units
        .iter()
        .map(|&unit| {
            Ok((
                i128::from(instance.populations[unit]),
                variables.assignment(unit).ok_or_else(|| {
                    ProofBackendError::ConnectivityCut(
                        "missing active assignment variable".to_string(),
                    )
                })?,
            ))
        })
        .collect::<Result<Vec<_>, ProofBackendError>>()?;
    constraints.push(linear_constraint(terms, "=", i128::from(remaining)));
    Ok(())
}

fn add_reduced_cut_threshold(
    instance: &CertifiedSplitInstance,
    variables: &ReducedCutsetVariables,
    threshold: u64,
    constraints: &mut Vec<String>,
) {
    constraints.push(linear_constraint(
        variables.active_edges.iter().map(|&edge| {
            (
                i128::from(instance.edges[edge].weight),
                variables.cut(edge).expect("active edge variable"),
            )
        }),
        "<=",
        i128::from(threshold),
    ));
}

fn add_reduced_connectivity_cut(
    instance: &CertifiedSplitInstance,
    fixed_assignments: &[Option<u8>],
    variables: &ReducedCutsetVariables,
    cut: &ConnectivityCut,
    constraints: &mut Vec<String>,
) -> Result<(), ProofBackendError> {
    validate_connectivity_cut_graph(instance, cut)?;
    let label = cut.district_id as u8;
    let anchor = fixed_assignments
        .iter()
        .position(|&value| value == Some(label))
        .ok_or_else(|| {
            ProofBackendError::ConnectivityCut(format!("child {label} has no fixed anchor"))
        })?;
    if cut.component.contains(&anchor) {
        return Err(ProofBackendError::ConnectivityCut(
            "reduced cut component contains its anchor core".to_string(),
        ));
    }
    let contains_fixed_core = cut
        .component
        .iter()
        .any(|&unit| fixed_assignments[unit] == Some(label));
    let active_boundary = cut
        .outside_neighbors
        .iter()
        .copied()
        .filter(|&unit| fixed_assignments[unit].is_none())
        .collect::<Vec<_>>();
    if cut
        .outside_neighbors
        .iter()
        .any(|&unit| fixed_assignments[unit] == Some(label))
    {
        return Err(ProofBackendError::ConnectivityCut(
            "reduced cut boundary already touches its fixed core".to_string(),
        ));
    }
    let mut terms = active_boundary
        .iter()
        .map(|&unit| {
            variables
                .assignment(unit)
                .map(|value| (if label == 1 { 1 } else { -1 }, value))
                .ok_or_else(|| {
                    ProofBackendError::ConnectivityCut(
                        "missing reduced boundary variable".to_string(),
                    )
                })
        })
        .collect::<Result<Vec<_>, ProofBackendError>>()?;
    if !contains_fixed_core {
        let representative = cut
            .component
            .iter()
            .copied()
            .find(|&unit| fixed_assignments[unit].is_none())
            .ok_or_else(|| {
                ProofBackendError::ConnectivityCut(
                    "reduced cut has no active representative".to_string(),
                )
            })?;
        let representative_variable = variables.assignment(representative).ok_or_else(|| {
            ProofBackendError::ConnectivityCut(
                "missing reduced representative variable".to_string(),
            )
        })?;
        terms.push((if label == 1 { -1 } else { 1 }, representative_variable));
    }
    constraints.push(linear_constraint(
        terms,
        ">=",
        if label == 1 && contains_fixed_core {
            1
        } else if label == 1 {
            0
        } else {
            1 - active_boundary.len() as i128
        },
    ));
    Ok(())
}

struct CutsetVariables {
    unit_count: usize,
    root_offset: usize,
    seen_offset: usize,
    variable_count: usize,
}

impl CutsetVariables {
    fn new(instance: &CertifiedSplitInstance) -> Self {
        let unit_count = instance.unit_ids.len();
        let root_offset = unit_count + instance.edges.len();
        let seen_offset = root_offset + 2 * unit_count;
        let variable_count = seen_offset + 2 * unit_count;
        Self {
            unit_count,
            root_offset,
            seen_offset,
            variable_count,
        }
    }

    fn root(&self, child: usize, unit: usize) -> String {
        variable(self.root_offset + child * self.unit_count + unit)
    }

    fn seen(&self, child: usize, unit: usize) -> String {
        variable(self.seen_offset + child * self.unit_count + unit)
    }
}

fn cutset_base_constraints(
    instance: &CertifiedSplitInstance,
    variables: &CutsetVariables,
) -> Vec<String> {
    let mut constraints = assignment_and_cut_constraints(instance);
    let unit_count = instance.unit_ids.len();
    for child in 0..2 {
        constraints.push(linear_constraint(
            (0..unit_count).map(|unit| (1, variables.root(child, unit))),
            "=",
            1,
        ));
        for unit in 0..unit_count {
            let root = variables.root(child, unit);
            let seen = variables.seen(child, unit);
            if unit == 0 {
                constraints.push(if child == 1 {
                    linear_constraint([(1, seen.clone()), (-1, x(unit))], "=", 0)
                } else {
                    linear_constraint([(1, seen.clone()), (1, x(unit))], "=", 1)
                });
                constraints.push(linear_constraint([(1, root.clone()), (-1, seen)], "=", 0));
            } else {
                let previous_seen = variables.seen(child, unit - 1);
                constraints.push(linear_constraint(
                    [(1, seen.clone()), (-1, previous_seen.clone())],
                    ">=",
                    0,
                ));
                constraints.push(if child == 1 {
                    linear_constraint([(1, seen.clone()), (-1, x(unit))], ">=", 0)
                } else {
                    linear_constraint([(1, seen.clone()), (1, x(unit))], ">=", 1)
                });
                constraints.push(if child == 1 {
                    linear_constraint(
                        [(1, x(unit)), (1, previous_seen.clone()), (-1, seen.clone())],
                        ">=",
                        0,
                    )
                } else {
                    linear_constraint(
                        [
                            (1, previous_seen.clone()),
                            (-1, seen.clone()),
                            (-1, x(unit)),
                        ],
                        ">=",
                        -1,
                    )
                });
                constraints.push(linear_constraint(
                    [(1, root.clone()), (-1, seen), (1, previous_seen)],
                    "=",
                    0,
                ));
            }
            constraints.push(if child == 1 {
                linear_constraint([(-1, root), (1, x(unit))], ">=", 0)
            } else {
                linear_constraint([(-1, root), (-1, x(unit))], ">=", -1)
            });
        }
    }
    constraints
}

fn add_connectivity_cut(
    instance: &CertifiedSplitInstance,
    variables: &CutsetVariables,
    cut: &ConnectivityCut,
    constraints: &mut Vec<String>,
) -> Result<(), ProofBackendError> {
    let component = validate_connectivity_cut_graph(instance, cut)?;
    let representative = *component
        .first()
        .ok_or_else(|| ProofBackendError::ConnectivityCut("empty component".to_string()))?;
    let mut terms = component
        .iter()
        .map(|&unit| (1, variables.root(cut.district_id, unit)))
        .collect::<Vec<_>>();
    if cut.district_id == 1 {
        terms.extend(cut.outside_neighbors.iter().map(|&unit| (1, x(unit))));
        terms.push((-1, x(representative)));
        constraints.push(linear_constraint(terms, ">=", 0));
    } else {
        terms.extend(cut.outside_neighbors.iter().map(|&unit| (-1, x(unit))));
        terms.push((1, x(representative)));
        constraints.push(linear_constraint(
            terms,
            ">=",
            1 - cut.outside_neighbors.len() as i128,
        ));
    }
    Ok(())
}

fn validate_connectivity_cut_graph(
    instance: &CertifiedSplitInstance,
    cut: &ConnectivityCut,
) -> Result<BTreeSet<usize>, ProofBackendError> {
    if cut.district_id > 1 {
        return Err(ProofBackendError::ConnectivityCut(format!(
            "district {} is not a split child",
            cut.district_id
        )));
    }
    let component = cut.component.iter().copied().collect::<BTreeSet<_>>();
    if component.is_empty() || component.len() != cut.component.len() {
        return Err(ProofBackendError::ConnectivityCut(
            "component must contain unique units".to_string(),
        ));
    }
    if component
        .iter()
        .chain(cut.outside_neighbors.iter())
        .any(|&unit| unit >= instance.unit_ids.len())
    {
        return Err(ProofBackendError::ConnectivityCut(
            "unit index is out of range".to_string(),
        ));
    }
    let mut adjacency = vec![Vec::new(); instance.unit_ids.len()];
    for edge in &instance.edges {
        adjacency[edge.left].push(edge.right);
        adjacency[edge.right].push(edge.left);
    }
    let expected_boundary = component
        .iter()
        .flat_map(|&unit| adjacency[unit].iter().copied())
        .filter(|unit| !component.contains(unit))
        .collect::<BTreeSet<_>>();
    let submitted_boundary = cut
        .outside_neighbors
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if submitted_boundary.len() != cut.outside_neighbors.len()
        || submitted_boundary != expected_boundary
    {
        return Err(ProofBackendError::ConnectivityCut(
            "outside-neighbor boundary does not match the graph".to_string(),
        ));
    }
    Ok(component)
}

struct CompactVariables {
    unit_count: usize,
    arcs: Vec<(usize, usize)>,
    depth_bits: usize,
    root_offset: usize,
    seen_offset: usize,
    parent_offset: usize,
    depth_offset: usize,
    variable_count: usize,
}

impl CompactVariables {
    fn new(instance: &CertifiedSplitInstance) -> Self {
        let unit_count = instance.unit_ids.len();
        let mut arcs = Vec::with_capacity(instance.edges.len() * 2);
        for edge in &instance.edges {
            arcs.push((edge.left, edge.right));
            arcs.push((edge.right, edge.left));
        }
        let depth_bits = usize::BITS as usize - (unit_count - 1).leading_zeros() as usize;
        let cut_offset = unit_count;
        let root_offset = cut_offset + instance.edges.len();
        let seen_offset = root_offset + 2 * unit_count;
        let parent_offset = seen_offset + 2 * unit_count;
        let depth_offset = parent_offset + 2 * arcs.len();
        let variable_count = depth_offset + 2 * unit_count * depth_bits;
        Self {
            unit_count,
            arcs,
            depth_bits,
            root_offset,
            seen_offset,
            parent_offset,
            depth_offset,
            variable_count,
        }
    }

    fn root(&self, child: usize, unit: usize) -> String {
        variable(self.root_offset + child * self.unit_count + unit)
    }

    fn parent(&self, child: usize, arc: usize) -> String {
        variable(self.parent_offset + child * self.arcs.len() + arc)
    }

    fn seen(&self, child: usize, unit: usize) -> String {
        variable(self.seen_offset + child * self.unit_count + unit)
    }

    fn depth(&self, child: usize, unit: usize, bit: usize) -> String {
        variable(
            self.depth_offset
                + child * self.unit_count * self.depth_bits
                + unit * self.depth_bits
                + bit,
        )
    }

    fn depth_terms(&self, child: usize, unit: usize, sign: i128) -> Vec<(i128, String)> {
        (0..self.depth_bits)
            .map(|bit| (sign * (1_i128 << bit), self.depth(child, unit, bit)))
            .collect()
    }
}

fn compact_base_constraints(
    instance: &CertifiedSplitInstance,
    variables: &CompactVariables,
) -> Vec<String> {
    let mut constraints = assignment_and_cut_constraints(instance);
    let unit_count = instance.unit_ids.len();
    let big_m = unit_count as i128;
    let mut incoming = vec![Vec::new(); unit_count];
    for (arc, &(_, to)) in variables.arcs.iter().enumerate() {
        incoming[to].push(arc);
    }
    for child in 0..2 {
        constraints.push(linear_constraint(
            (0..unit_count).map(|unit| (1, variables.root(child, unit))),
            "=",
            1,
        ));
        for unit in 0..unit_count {
            let root = variables.root(child, unit);
            let seen = variables.seen(child, unit);
            if unit == 0 {
                constraints.push(if child == 1 {
                    linear_constraint([(1, seen.clone()), (-1, x(unit))], "=", 0)
                } else {
                    linear_constraint([(1, seen.clone()), (1, x(unit))], "=", 1)
                });
                constraints.push(linear_constraint([(1, root.clone()), (-1, seen)], "=", 0));
            } else {
                let previous_seen = variables.seen(child, unit - 1);
                constraints.push(linear_constraint(
                    [(1, seen.clone()), (-1, previous_seen.clone())],
                    ">=",
                    0,
                ));
                constraints.push(if child == 1 {
                    linear_constraint([(1, seen.clone()), (-1, x(unit))], ">=", 0)
                } else {
                    linear_constraint([(1, seen.clone()), (1, x(unit))], ">=", 1)
                });
                constraints.push(if child == 1 {
                    linear_constraint(
                        [(1, x(unit)), (1, previous_seen.clone()), (-1, seen.clone())],
                        ">=",
                        0,
                    )
                } else {
                    linear_constraint(
                        [
                            (1, previous_seen.clone()),
                            (-1, seen.clone()),
                            (-1, x(unit)),
                        ],
                        ">=",
                        -1,
                    )
                });
                constraints.push(linear_constraint(
                    [(1, root.clone()), (-1, seen), (1, previous_seen)],
                    "=",
                    0,
                ));
            }
            constraints.push(if child == 1 {
                linear_constraint([(-1, root.clone()), (1, x(unit))], ">=", 0)
            } else {
                linear_constraint([(-1, root.clone()), (-1, x(unit))], ">=", -1)
            });
            let mut parent_equation = incoming[unit]
                .iter()
                .map(|&arc| (1, variables.parent(child, arc)))
                .collect::<Vec<_>>();
            parent_equation.push((1, root.clone()));
            parent_equation.push(if child == 1 {
                (-1, x(unit))
            } else {
                (1, x(unit))
            });
            constraints.push(linear_constraint(
                parent_equation,
                "=",
                if child == 1 { 0 } else { 1 },
            ));
            let mut assigned_depth = variables.depth_terms(child, unit, 1);
            assigned_depth.push(if child == 1 {
                (-big_m, x(unit))
            } else {
                (big_m, x(unit))
            });
            constraints.push(linear_constraint(
                assigned_depth,
                "<=",
                if child == 1 { 0 } else { big_m },
            ));
            let mut root_depth = variables.depth_terms(child, unit, -1);
            root_depth.push((-big_m, root));
            constraints.push(linear_constraint(root_depth, ">=", -big_m));
        }
        for (arc, &(from, to)) in variables.arcs.iter().enumerate() {
            let parent = variables.parent(child, arc);
            for endpoint in [from, to] {
                constraints.push(if child == 1 {
                    linear_constraint([(-1, parent.clone()), (1, x(endpoint))], ">=", 0)
                } else {
                    linear_constraint([(-1, parent.clone()), (-1, x(endpoint))], ">=", -1)
                });
            }
            let mut depth_order = variables.depth_terms(child, to, 1);
            depth_order.extend(variables.depth_terms(child, from, -1));
            depth_order.push((-big_m, parent));
            constraints.push(linear_constraint(depth_order, ">=", 1 - big_m));
        }
    }
    constraints
}

fn add_population_threshold(
    instance: &CertifiedSplitInstance,
    threshold: u64,
    constraints: &mut Vec<String>,
) {
    let parent_population = instance
        .populations
        .iter()
        .map(|&value| i128::from(value))
        .sum::<i128>();
    let target = instance.k_right as i128 * parent_population;
    let terms = instance
        .populations
        .iter()
        .enumerate()
        .map(|(unit, &population)| (instance.k_parent as i128 * i128::from(population), x(unit)))
        .collect::<Vec<_>>();
    constraints.push(linear_constraint(
        terms.iter().cloned(),
        "<=",
        target + i128::from(threshold),
    ));
    constraints.push(linear_constraint(
        terms,
        ">=",
        target - i128::from(threshold),
    ));
}

fn add_cut_threshold(
    instance: &CertifiedSplitInstance,
    threshold: u64,
    constraints: &mut Vec<String>,
) {
    constraints.push(linear_constraint(
        instance
            .edges
            .iter()
            .enumerate()
            .map(|(edge, value)| (i128::from(value.weight), y(edge, instance.unit_ids.len()))),
        "<=",
        i128::from(threshold),
    ));
}

fn add_exact_right_population(
    instance: &CertifiedSplitInstance,
    population: i64,
    constraints: &mut Vec<String>,
) {
    constraints.push(linear_constraint(
        instance
            .populations
            .iter()
            .enumerate()
            .map(|(unit, &value)| (i128::from(value), x(unit))),
        "=",
        i128::from(population),
    ));
}

fn add_lex_predecessor(incumbent: &[u8], constraints: &mut Vec<String>) {
    let terms = incumbent
        .iter()
        .enumerate()
        .map(|(unit, _)| (1_i128 << (incumbent.len() - unit - 1), x(unit)))
        .collect::<Vec<_>>();
    let incumbent_value = incumbent
        .iter()
        .fold(0_i128, |value, &bit| (value << 1) | i128::from(bit));
    if incumbent_value == 0 {
        constraints.push(contradiction());
    } else {
        constraints.push(linear_constraint(terms, "<=", incumbent_value - 1));
    }
}

fn add_compact_lex_predecessor(
    incumbent: &[u8],
    constraints: &mut Vec<String>,
    variable_count: &mut usize,
) {
    let prefix_start = *variable_count;
    *variable_count += incumbent.len() + 1;
    let prefix = |index: usize| variable(prefix_start + index);
    constraints.push(linear_constraint([(1, prefix(0))], "=", 1));
    let mut witnesses = Vec::new();
    for (unit, &bit) in incumbent.iter().enumerate() {
        let current = prefix(unit);
        let next = prefix(unit + 1);
        constraints.push(linear_constraint(
            [(-1, next.clone()), (1, current.clone())],
            ">=",
            0,
        ));
        if bit == 1 {
            constraints.push(linear_constraint(
                [(-1, next.clone()), (1, x(unit))],
                ">=",
                0,
            ));
            constraints.push(linear_constraint(
                [(1, next), (-1, current.clone()), (-1, x(unit))],
                ">=",
                -1,
            ));
            let witness = variable(*variable_count);
            *variable_count += 1;
            constraints.push(linear_constraint(
                [(-1, witness.clone()), (1, current.clone())],
                ">=",
                0,
            ));
            constraints.push(linear_constraint(
                [(-1, witness.clone()), (-1, x(unit))],
                ">=",
                -1,
            ));
            constraints.push(linear_constraint(
                [(1, witness.clone()), (-1, current), (1, x(unit))],
                ">=",
                0,
            ));
            witnesses.push(witness);
        } else {
            constraints.push(linear_constraint(
                [(-1, next.clone()), (-1, x(unit))],
                ">=",
                -1,
            ));
            constraints.push(linear_constraint(
                [(1, next), (-1, current), (1, x(unit))],
                ">=",
                0,
            ));
        }
    }
    constraints.push(linear_constraint(
        witnesses.into_iter().map(|witness| (1, witness)),
        ">=",
        1,
    ));
}

fn add_connectivity_nogoods(instance: &CertifiedSplitInstance, constraints: &mut Vec<String>) {
    let unit_count = instance.unit_ids.len();
    let end = 1_u64 << unit_count;
    for mask in 1..(end - 1) {
        let assignment = (0..unit_count)
            .map(|unit| ((mask >> unit) & 1) as u8)
            .collect::<Vec<_>>();
        if instance.k_left == instance.k_right && assignment[0] != 0 {
            continue;
        }
        let left_units = assignment.iter().filter(|&&label| label == 0).count();
        if left_units < instance.k_left || unit_count - left_units < instance.k_right {
            continue;
        }
        if certified_split_children_connected(instance, &assignment).unwrap_or(false) {
            continue;
        }
        let ones = assignment.iter().filter(|&&bit| bit == 1).count() as i128;
        constraints.push(linear_constraint(
            assignment.iter().enumerate().map(|(unit, &bit)| {
                if bit == 0 {
                    (1, x(unit))
                } else {
                    (-1, x(unit))
                }
            }),
            ">=",
            1 - ones,
        ));
    }
}

fn decision_has_counterexample(
    instance: &CertifiedSplitInstance,
    incumbent: &CertifiedSplitObjective,
    stage: CertifiedDecisionStage,
) -> Result<bool, CertifiedSplitError> {
    let unit_count = instance.unit_ids.len();
    let end = 1_u64 << unit_count;
    for mask in 1..(end - 1) {
        let assignment = (0..unit_count)
            .map(|unit| ((mask >> unit) & 1) as u8)
            .collect::<Vec<_>>();
        if instance.k_left == instance.k_right && assignment[0] != 0 {
            continue;
        }
        let Ok(primary) = evaluate_certified_split_objective(instance, &assignment) else {
            continue;
        };
        if !certified_split_children_connected(instance, &assignment)? {
            continue;
        }
        let counterexample = match stage {
            CertifiedDecisionStage::PopulationLowerBound => {
                primary.max_population_deviation_scaled
                    < incumbent.primary.max_population_deviation_scaled
            }
            CertifiedDecisionStage::BoundaryLowerBound => {
                primary.max_population_deviation_scaled
                    <= incumbent.primary.max_population_deviation_scaled
                    && primary.weighted_boundary_cut < incumbent.primary.weighted_boundary_cut
            }
            CertifiedDecisionStage::CanonicalTieBreak => {
                primary.max_population_deviation_scaled
                    <= incumbent.primary.max_population_deviation_scaled
                    && primary.weighted_boundary_cut <= incumbent.primary.weighted_boundary_cut
                    && assignment < incumbent.canonical_assignment
            }
        };
        if counterexample {
            return Ok(true);
        }
    }
    Ok(false)
}

fn stage_claim(incumbent: &CertifiedSplitObjective, stage: CertifiedDecisionStage) -> String {
    match stage {
        CertifiedDecisionStage::PopulationLowerBound => format!(
            "no connected canonical split has max scaled population deviation below {}",
            incumbent.primary.max_population_deviation_scaled
        ),
        CertifiedDecisionStage::BoundaryLowerBound => format!(
            "at population deviation {}, no connected split has weighted cut below {}",
            incumbent.primary.max_population_deviation_scaled,
            incumbent.primary.weighted_boundary_cut
        ),
        CertifiedDecisionStage::CanonicalTieBreak => {
            format!(
                "at population deviation {} and weighted cut {}, no lexicographically smaller assignment exists",
                incumbent.primary.max_population_deviation_scaled,
                incumbent.primary.weighted_boundary_cut
            )
        }
    }
}

fn x(unit: usize) -> String {
    variable(unit)
}

fn y(edge: usize, unit_count: usize) -> String {
    variable(unit_count + edge)
}

fn variable(zero_based_index: usize) -> String {
    format!("x{}", zero_based_index + 1)
}

fn linear_constraint(
    terms: impl IntoIterator<Item = (i128, String)>,
    relation: &str,
    rhs: i128,
) -> String {
    let (terms, relation, rhs) = if relation == "<=" {
        (
            terms
                .into_iter()
                .map(|(coefficient, variable)| (-coefficient, variable))
                .collect::<Vec<_>>(),
            ">=",
            -rhs,
        )
    } else {
        (terms.into_iter().collect::<Vec<_>>(), relation, rhs)
    };
    let mut combined = BTreeMap::<String, i128>::new();
    for (coefficient, variable) in terms {
        *combined.entry(variable).or_default() += coefficient;
    }
    let expression = combined
        .into_iter()
        .filter(|(_, coefficient)| *coefficient != 0)
        .map(|(variable, coefficient)| {
            if coefficient > 0 {
                format!("+{coefficient} {variable}")
            } else {
                format!("{coefficient} {variable}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if expression.is_empty() {
        let satisfied = match relation {
            ">=" => 0 >= rhs,
            "=" => rhs == 0,
            _ => false,
        };
        if satisfied {
            "+1 x1 >= 0 ;".to_string()
        } else {
            contradiction()
        }
    } else {
        format!("{expression} {relation} {rhs} ;")
    }
}

fn contradiction() -> String {
    "+1 x1 >= 2 ;".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        canonical_orientation_rule, canonical_seat_split, certified_split_unit_universe_hash,
        solve_certified_split_bounded, ExactEdge, CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION,
        CERTIFIED_SPLIT_MODEL_ID,
    };

    fn path8() -> CertifiedSplitInstance {
        let unit_ids = (0..8).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let (k_left, k_right) = canonical_seat_split(4).unwrap();
        CertifiedSplitInstance {
            schema_version: CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![100; 8],
            edges: (0..7)
                .map(|left| ExactEdge {
                    left,
                    right: left + 1,
                    weight: 1,
                })
                .collect(),
            k_parent: 4,
            k_left,
            k_right,
            orientation_rule: canonical_orientation_rule(k_left, k_right),
        }
    }

    #[test]
    fn optimal_path8_compiles_three_unsat_decision_requests() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let assignment = artifacts.proof.canonical_assignment.clone().unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            Some("0.1.0".to_string()),
            "test-discovery",
            assignment,
        )
        .unwrap();
        let requests = compile_certified_split_proof_requests(&instance, &discovery).unwrap();
        assert_eq!(requests.len(), 3);
        assert!(requests.iter().all(|artifact| {
            artifact.request.status == CertifiedDecisionStatus::UnsatProofRequired
                && artifact.request.opb_sha256
                    == format!("sha256:{:x}", Sha256::digest(artifact.opb.as_bytes()))
        }));
        assert!(requests[1].opb.contains("x9"));
        assert!(requests[2].opb.contains("x1"));
    }

    #[test]
    fn suboptimal_discovery_exposes_sat_counterexample() {
        let instance = path8();
        let discovery = certified_split_discovery(
            &instance,
            "mock",
            None,
            "suboptimal",
            vec![0, 0, 0, 1, 1, 1, 1, 1],
        )
        .unwrap();
        let requests = compile_certified_split_proof_requests(&instance, &discovery).unwrap();
        assert_eq!(
            requests[0].request.status,
            CertifiedDecisionStatus::SatCounterexampleExists
        );
    }

    #[test]
    fn opb_compilation_is_deterministic() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "determinism",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let first = compile_certified_split_proof_requests(&instance, &discovery).unwrap();
        let second = compile_certified_split_proof_requests(&instance, &discovery).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn nonzero_population_optimum_compiles_nontrivial_unsat_bound() {
        let mut instance = path8();
        instance.populations = vec![1, 1, 1, 2, 1, 1, 1, 1];
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "nonzero-population-bound",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        assert!(discovery.objective.primary.max_population_deviation_scaled > 0);
        let requests = compile_certified_split_proof_requests(&instance, &discovery).unwrap();
        assert_eq!(
            requests[0].request.status,
            CertifiedDecisionStatus::UnsatProofRequired
        );
        assert!(!requests[0].opb.contains("+1 x1 >= 2 ;"));
    }

    #[test]
    fn compact_parent_depth_encoding_is_polynomial_and_unsat_on_path8() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "compact-connectivity",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let requests =
            compile_certified_split_compact_proof_requests(&instance, &discovery).unwrap();
        assert_eq!(requests.len(), 3);
        assert!(requests.iter().all(|artifact| {
            artifact.request.connectivity_encoding == "parent-depth-v3"
                && artifact.request.status == CertifiedDecisionStatus::ProofRequiredUnclassified
        }));
        assert_eq!(requests[0].request.variable_count, 123);
        assert_eq!(requests[1].request.variable_count, 123);
        assert!(requests[2].request.variable_count > 123);
        assert!(requests[1].opb.contains("#variable= 123"));
        assert!(requests[1].opb.contains("+1 x1 +1 x32 = 1 ;"));
        assert!(requests[1].opb.contains("+1 x16 -1 x32 = 0 ;"));
        assert!(requests[1]
            .opb
            .contains("-8 x1 -1 x76 -2 x77 -4 x78 >= -8 ;"));
        assert!(requests[1]
            .opb
            .contains("+8 x1 -1 x100 -2 x101 -4 x102 >= 0 ;"));
    }

    #[test]
    fn compact_canonical_lex_encoding_scales_beyond_i128_bits() {
        let unit_count = 130;
        let unit_ids = (0..unit_count)
            .map(|unit| format!("u{unit:03}"))
            .collect::<Vec<_>>();
        let instance = CertifiedSplitInstance {
            schema_version: CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1; unit_count],
            edges: (0..unit_count - 1)
                .map(|left| ExactEdge {
                    left,
                    right: left + 1,
                    weight: 1,
                })
                .collect(),
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: canonical_orientation_rule(1, 1),
        };
        let assignment = (0..unit_count)
            .map(|unit| if unit < unit_count / 2 { 0 } else { 1 })
            .collect::<Vec<_>>();
        let discovery =
            certified_split_discovery(&instance, "fixture", None, "large-lex", assignment).unwrap();
        let artifact = compile_certified_split_compact_proof_request(
            &instance,
            &discovery,
            CertifiedDecisionStage::CanonicalTieBreak,
        )
        .unwrap();
        assert_eq!(
            artifact.request.status,
            CertifiedDecisionStatus::ProofRequiredUnclassified
        );
        assert!(artifact.request.variable_count > 3_000);
        assert!(artifact.opb.starts_with("* #variable="));
    }

    #[test]
    fn compact_boundary_population_branches_cover_floor_and_ceiling() {
        let mut instance = path8();
        instance.populations = vec![1, 1, 1, 2, 1, 1, 1, 1];
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "population-branches",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let low =
            compile_certified_split_compact_boundary_branch(&instance, &discovery, 4).unwrap();
        let high =
            compile_certified_split_compact_boundary_branch(&instance, &discovery, 5).unwrap();
        assert_eq!(low.request.exact_right_population, Some(4));
        assert_eq!(high.request.exact_right_population, Some(5));
        assert_eq!(low.request.connectivity_encoding, "parent-depth-v3");
        assert_eq!(high.request.connectivity_encoding, "parent-depth-v3");
    }

    #[test]
    fn cutset_boundary_compiles_small_relaxation_and_violated_cuts() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "cutset-relaxation",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let relaxed =
            compile_certified_split_cutset_boundary_branch(&instance, &discovery, 400, &[])
                .unwrap();
        assert_eq!(relaxed.request.variable_count, 47);
        assert_eq!(
            relaxed.request.connectivity_encoding,
            "cutset-v1;cuts=0;fixed=0"
        );

        let adjacency = vec![
            vec![1],
            vec![0, 2],
            vec![1, 3],
            vec![2, 4],
            vec![3, 5],
            vec![4, 6],
            vec![5, 7],
            vec![6],
        ];
        let cuts = crate::separate_connectivity_cuts(&adjacency, &[0, 1, 0, 0, 1, 1, 1, 0], 2);
        let strengthened =
            compile_certified_split_cutset_boundary_branch(&instance, &discovery, 400, &cuts)
                .unwrap();
        assert_eq!(
            strengthened.request.connectivity_encoding,
            format!("cutset-v1;cuts={};fixed=0", cuts.len())
        );
        assert_eq!(
            strengthened.request.constraint_count,
            relaxed.request.constraint_count + cuts.len()
        );
    }

    #[test]
    fn cutset_boundary_rejects_tampered_graph_boundary() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "cutset-tamper",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let cut = ConnectivityCut {
            district_id: 0,
            component: vec![0],
            outside_neighbors: vec![2],
        };
        let error =
            compile_certified_split_cutset_boundary_branch(&instance, &discovery, 400, &[cut])
                .unwrap_err();
        assert!(matches!(error, ProofBackendError::ConnectivityCut(_)));
    }

    #[test]
    fn cutset_fixed_core_adds_exact_assignment_constraints() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "cutset-fixed-core",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let fixes = vec![Some(0), Some(0), None, None, None, None, Some(1), Some(1)];
        let artifact = compile_certified_split_cutset_boundary_branch_with_fixes(
            &instance,
            &discovery,
            400,
            &[],
            &fixes,
        )
        .unwrap();
        assert_eq!(
            artifact.request.connectivity_encoding,
            "cutset-v1;cuts=0;fixed=4"
        );
        assert!(artifact.opb.contains("+1 x1 = 0 ;"));
        assert!(artifact.opb.contains("+1 x8 = 1 ;"));
    }

    #[test]
    fn reduced_cutset_eliminates_fixed_core_variables() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "reduced-cutset",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let fixes = vec![Some(0), Some(0), None, None, None, None, Some(1), Some(1)];
        let artifact = compile_certified_split_reduced_cutset_boundary_branch(
            &instance,
            &discovery,
            400,
            &[],
            &fixes,
        )
        .unwrap();
        assert_eq!(artifact.request.variable_count, 9);
        assert_eq!(
            artifact.request.connectivity_encoding,
            "cutset-reduced-v1;active=4;cuts=0;fixed=4"
        );
        assert!(artifact.opb.contains("#variable= 9"));
    }

    #[test]
    fn reduced_cutset_accepts_multiple_fixed_core_components() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "reduced-cutset-disconnected-core",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let fixes = vec![Some(0), None, Some(0), None, None, Some(1), None, Some(1)];
        let artifact = compile_certified_split_reduced_cutset_boundary_branch(
            &instance,
            &discovery,
            400,
            &[],
            &fixes,
        )
        .unwrap();
        assert_eq!(artifact.request.variable_count, 11);
    }

    #[test]
    fn boundary_relaxation_omits_all_connectivity_witnesses() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "boundary-relaxation",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let artifact =
            compile_certified_split_boundary_relaxation(&instance, &discovery, 400).unwrap();
        assert_eq!(artifact.request.variable_count, 15);
        assert_eq!(
            artifact.request.connectivity_encoding,
            "connectivity-relaxation-v1"
        );
        assert!(!artifact.opb.contains("x16"));
    }

    #[test]
    fn outside_core_relaxation_requires_a_fixed_label_change() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "outside-core-relaxation",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let fixes = vec![Some(0), Some(0), None, None, None, None, Some(1), Some(1)];
        let artifact = compile_certified_split_boundary_relaxation_outside_core(
            &instance, &discovery, 400, &fixes,
        )
        .unwrap();
        assert_eq!(
            artifact.request.connectivity_encoding,
            "connectivity-relaxation-outside-core-v1;fixed=4"
        );
        assert!(artifact.opb.contains("+1 x1 +1 x2 -1 x7 -1 x8 >= -1 ;"));
    }

    #[test]
    fn regional_relaxation_adds_exact_population_branch() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "regional-relaxation",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let regional = vec![RegionalPopulationConstraint {
            region_id: "west".to_string(),
            units: vec![0, 1, 2, 3],
            relation: RegionalPopulationRelation::Equal,
            population: 0,
        }];
        let artifact = compile_certified_split_regional_boundary_relaxation(
            &instance, &discovery, 400, &regional,
        )
        .unwrap();
        assert_eq!(
            artifact.request.connectivity_encoding,
            "connectivity-relaxation-regional-v1;constraints=1"
        );
        assert!(artifact
            .opb
            .contains("+100 x1 +100 x2 +100 x3 +100 x4 = 0 ;"));
    }

    #[test]
    fn reduced_relaxation_eliminates_one_sided_fixed_units() {
        let instance = path8();
        let artifacts = solve_certified_split_bounded(&instance).unwrap();
        let discovery = certified_split_discovery(
            &instance,
            "bounded-oracle",
            None,
            "reduced-relaxation",
            artifacts.proof.canonical_assignment.unwrap(),
        )
        .unwrap();
        let fixes = vec![Some(0), Some(0), None, None, None, None, None, None];
        let artifact =
            compile_certified_split_reduced_boundary_relaxation(&instance, &discovery, 400, &fixes)
                .unwrap();
        assert_eq!(artifact.request.variable_count, 12);
        assert_eq!(
            artifact.request.connectivity_encoding,
            "connectivity-relaxation-reduced-v1;active=6;fixed=2"
        );
    }
}
