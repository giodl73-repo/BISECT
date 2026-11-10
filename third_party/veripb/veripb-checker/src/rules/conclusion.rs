use std::rc::Rc;

use malachite_bigint::BigInt;
use veripb_formula::{pb_constraint::constraint_from_terms, prelude::*};
use veripb_propagator::propagation_engine::PropagationEngine;
use veripb_rules::{
    Bound, ConclusionResult, ConclusionRule,
    elaboration::{ConstraintLookup, Elaboratable, ElaborationError, Elaborator},
};

use crate::prelude::*;

#[derive(Debug)]
pub struct ConclusionChecker {
    inner: ConclusionRule,
    propagated_solution: Option<Assignment<BooleanVar>>,
}

impl ConclusionChecker {
    pub fn new(inner: ConclusionRule) -> Self {
        Self {
            inner,
            propagated_solution: None,
        }
    }
}

impl From<ConclusionRule> for ConclusionChecker {
    fn from(inner: ConclusionRule) -> Self {
        Self::new(inner)
    }
}

impl Checkable for ConclusionChecker {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        validate_conclusion_rule_usage(
            context.has_conclusion,
            context.has_output,
            context.has_end_proof,
        )?;

        // Perform the necessary checks for each conclusion.
        match self.inner.result {
            ConclusionResult::None => {}
            ConclusionResult::Unsatisfiable => {
                self.inner.constraint_id_hint = Some(validate_unsat_conclusion(
                    self.inner.constraint_id_hint,
                    database,
                    context.best_objective_value.as_ref(),
                    context.objective.as_ref(),
                )?);
            }
            ConclusionResult::Satisfiable => {
                self.propagated_solution = validate_sat_conclusion(
                    self.inner.solution_hint.as_ref(),
                    context.best_objective_value.as_ref(),
                    context.objective.as_ref(),
                    &context.original_constraints,
                    context.var_names.len(),
                )?;
            }
            ConclusionResult::Bounds => {
                let lower_bound = self
                    .inner
                    .lower_bound_or_count
                    .as_ref()
                    .expect("Must have lower bound for conclusion BOUNDS");
                let upper_bound = self
                    .inner
                    .upper_bound
                    .as_ref()
                    .expect("Must have upper bound for conclusion BOUNDS");

                validate_bounds_compatibility(lower_bound, upper_bound)?;

                // Check lower bound.
                self.inner.constraint_id_hint = Some(validate_lower_bound(
                    lower_bound,
                    context.best_objective_value.as_ref(),
                    self.inner.constraint_id_hint,
                    context.objective.as_ref(),
                    database,
                    context.only_core,
                    &mut context.propagation_engine,
                )?);

                // Check upper bound.
                self.propagated_solution = validate_upper_bound(
                    upper_bound,
                    context.best_valid_objective_value.as_ref(),
                    self.inner.solution_hint.as_ref(),
                    context.original_objective.as_ref(),
                    &context.original_constraints,
                    context.var_names.len(),
                )?;
            }
            ConclusionResult::EnumerationComplete => {
                validate_enumerated_count(
                    self.inner.lower_bound_or_count.as_ref(),
                    &context.num_excluded_solutions,
                )?;
                self.inner.constraint_id_hint = Some(check_contradiction(
                    self.inner.constraint_id_hint,
                    database,
                )?);
            }
            ConclusionResult::EnumerationPartial => {
                validate_enumerated_count(
                    self.inner.lower_bound_or_count.as_ref(),
                    &context.num_excluded_solutions,
                )?;
            }
        }

        // Print the verification statement for each conclusion.
        if context.config.print_verification_result {
            print_conclusion_status(
                &self.inner.result,
                self.inner.lower_bound_or_count.as_ref(),
                self.inner.upper_bound.as_ref(),
                context.assertion_used,
            );
        }

        context.has_conclusion = true;
        Ok(vec![])
    }
}

/// Return an error when the given bool flags indicate an
/// invalid state at the point where a [`ConclusionRule`] is encountered.
pub fn validate_conclusion_rule_usage(
    second_conclusion: bool,
    output_before_conclusion: bool,
    end_before_conclusion: bool,
) -> Result<(), CheckingError> {
    if second_conclusion {
        return Err(CheckingError::DoubleConclusion);
    }
    if !output_before_conclusion || end_before_conclusion {
        return Err(CheckingError::WrongFooterOrder("conclusion"));
    }
    Ok(())
}

/// Checks if the database contains a contradiction.
pub fn check_contradiction(
    constraint_id_hint: Option<isize>,
    database: &Database,
) -> Result<isize, CheckingError> {
    match constraint_id_hint {
        None => {
            if let Some(contradicting_id) = database.contains_contradiction() {
                Ok(contradicting_id as isize)
            } else {
                Err(CheckingError::NoContradicitionInDB)
            }
        }
        Some(hint) => {
            let constraint = database.get_entry(hint)?;
            if constraint.is_contradicting() {
                return Ok(database.normalize_id(hint));
            }
            Err(CheckingError::HintNoContradiction(hint))
        }
    }
}

/// Check if the proof contains a solution or the hint is a correct solution.
///
/// Returns the propagated solution if a solution was given as a hint.
#[inline]
fn check_solution(
    conclusion_result: &ConclusionResult,
    solution_hint: Option<&Vec<Lit>>,
    num_vars: usize,
    original_constraints: &[PBConstraintEnum],
    best_valid_objective_value: Option<&BigInt>,
) -> Result<Option<Assignment<BooleanVar>>, CheckingError> {
    match solution_hint {
        Some(solution) => {
            // Do not propagate the assignment, as the hint given in the conclusion has to be a full assignment.
            let assignment = Assignment::from(solution, num_vars)
                .ok_or(CheckingError::ConclusionSolutionConflicting)?;
            for constraint in original_constraints.iter() {
                if !constraint.is_satisfied(&assignment) {
                    return Err(CheckingError::OriginalConstraintNotSatisfied);
                }
            }
            Ok(Some(assignment))
        }
        None => {
            if best_valid_objective_value.is_none() {
                return Err(CheckingError::SolutionMissing(
                    conclusion_result.to_string(),
                ));
            }
            Ok(None)
        }
    }
}

/// Check that the claimed number of enumerated solutions matches the number the proof excluded.
pub fn validate_enumerated_count(
    count: Option<&Bound>,
    num_excluded_solutions: &BigInt,
) -> Result<(), CheckingError> {
    let count = count.expect("Must have a solution count for an enumeration conclusion");
    let expected_sol = count.clone().unwrap();
    if expected_sol != *num_excluded_solutions {
        return Err(CheckingError::MismatchEnumeratedSolutions(
            expected_sol,
            num_excluded_solutions.clone(),
        ));
    }
    Ok(())
}

/// Ensure a SAT conclusion is allowed: return an error if `objective` is [`Some`].
///
/// Otherwise check the solution given in the conclusion, or that one was logged in the proof.
///
/// Returns the solution if one was given in the conclusion.
pub fn validate_sat_conclusion(
    solution_hint: Option<&Vec<Lit>>,
    best_objective_value: Option<&BigInt>,
    objective: Option<&PBObjective>,
    original_constraints: &[PBConstraintEnum],
    num_vars: usize,
) -> Result<Option<Assignment<BooleanVar>>, CheckingError> {
    if objective.is_some() {
        return Err(CheckingError::ConclusionSatWithObjective);
    }
    check_solution(
        &ConclusionResult::Satisfiable,
        solution_hint,
        num_vars,
        original_constraints,
        best_objective_value,
    )
}

/// Ensure an UNSAT conclusion is allowed: return an error if either `best_objective_value` or
/// `objective` is [`Some`].
///
/// Otherwise check the database for contradiction, returning the contradicting constraint id.
pub fn validate_unsat_conclusion(
    constraint_id_hint: Option<isize>,
    database: &mut Database,
    best_objective_value: Option<&BigInt>,
    objective: Option<&PBObjective>,
) -> Result<isize, CheckingError> {
    if best_objective_value.is_some() {
        return Err(CheckingError::ConclusionUnsatSolutionLogged);
    }
    if objective.is_some() {
        return Err(CheckingError::ConclusionUnsatObjective);
    }
    check_contradiction(constraint_id_hint, database)
}

/// Check that the two bounds of a `conclusion BOUNDS` are compatible with each other.
///
/// An unbounded lower bound claims infeasibility, so it admits no upper bound, and a bounded lower
/// bound may not exceed a bounded upper bound.
pub fn validate_bounds_compatibility(
    lower_bound: &Bound,
    upper_bound: &Bound,
) -> Result<(), CheckingError> {
    match (lower_bound, upper_bound) {
        (Bound::Unbounded, Bound::Bounded(_)) => {
            Err(CheckingError::ConclusionBoundsInfeasibleAndUpper)
        }
        (Bound::Bounded(lower), Bound::Bounded(upper)) if lower > upper => Err(
            CheckingError::ConclusionBoundsLowerGreaterUpper(lower.to_owned(), upper.to_owned()),
        ),
        _ => Ok(()),
    }
}

/// Check the lower bound of a `conclusion BOUNDS`.
///
/// `Unbounded` requires that no solution was logged and that the database contains a
/// contradiction. `Bounded` requires that the bound does not beat the best logged solution and
/// that the corresponding lower bounding constraint is implied.
///
/// Returns the id of the constraint justifying the bound.
pub fn validate_lower_bound(
    lower_bound: &Bound,
    best_objective_value: Option<&BigInt>,
    constraint_id_hint: Option<isize>,
    objective: Option<&PBObjective>,
    database: &mut Database,
    only_core: bool,
    prop_engine: &mut PropagationEngine,
) -> Result<isize, CheckingError> {
    let Some(objective) = objective else {
        return Err(CheckingError::ConclusionBoundsNoObjective);
    };

    match lower_bound {
        Bound::Unbounded => {
            if best_objective_value.is_some() {
                return Err(CheckingError::ConclusionBoundsInfeasibleAndSolutionLogged);
            }

            check_contradiction(constraint_id_hint, database)
        }
        Bound::Bounded(bound) => {
            // Check that lower bound is not better than best logged solution.
            if best_objective_value.is_some_and(|best| bound > best) {
                return Err(CheckingError::ConclusionBoundsLBLargerBestSolution);
            }
            // Construct lower bounding constraint.
            let constraint = constraint_from_terms::<BigInt>(
                objective.terms.values().cloned().collect(),
                bound.to_owned() - &objective.constant,
            );
            if let Some(contradicting_id) = database.contains_contradiction() {
                Ok(contradicting_id as isize)
            } else {
                let db_constraint = Rc::new(DBConstraint::from(constraint));
                check_implication(
                    only_core,
                    prop_engine,
                    database,
                    &db_constraint,
                    constraint_id_hint,
                )
            }
        }
    }
}

/// Check the upper bound of a `conclusion BOUNDS`.
///
/// `Unbounded` is always accepted. `Bounded` must match the objective value of the solution hint,
/// or the best recorded objective value when no hint is given.
///
/// Returns the propagated solution if a solution was given as a hint.
pub fn validate_upper_bound(
    upper_bound: &Bound,
    best_valid_objective_value: Option<&BigInt>,
    solution_hint: Option<&Vec<Lit>>,
    original_objective: Option<&PBObjective>,
    original_constraints: &[PBConstraintEnum],
    num_vars: usize,
) -> Result<Option<Assignment<BooleanVar>>, CheckingError> {
    let Bound::Bounded(bound) = upper_bound else {
        return Ok(None);
    };

    let solution = check_solution(
        &ConclusionResult::Bounds,
        solution_hint,
        num_vars,
        original_constraints,
        best_valid_objective_value,
    )?;
    match solution {
        Some(assignment) => {
            let value = original_objective
                .expect("objective should be present for conclusion BOUNDS")
                .evaluate(&assignment)
                .ok_or(CheckingError::ObjectiveUnassigned)?;

            if *bound != value {
                return Err(CheckingError::ConclusionBoundsUpperBoundMismatchHint(
                    bound.to_owned(),
                    value,
                ));
            }
            Ok(Some(assignment))
        }

        None => {
            let best_valid_objective_value =
                best_valid_objective_value.expect("error should be thrown by check_solution if best_valid_objective_value is not present");
            if bound != best_valid_objective_value {
                return Err(CheckingError::ConclusionBoundsUpperBoundMismatchRecorded(
                    bound.to_owned(),
                    best_valid_objective_value.clone(),
                ));
            }
            Ok(None)
        }
    }
}

/// Print the final `s VERIFIED ...` line (or `s UNDER ASSERTIONS ...` when an
/// assertion was used) summarising a completed conclusion.
pub fn print_conclusion_status(
    result: &ConclusionResult,
    lower_bound: Option<&Bound>,
    upper_bound: Option<&Bound>,
    assertion_used: bool,
) {
    let conclusion_status = match result {
        ConclusionResult::None => "NO CONCLUSION".to_string(),
        ConclusionResult::Unsatisfiable => "UNSATISFIABLE".to_string(),
        ConclusionResult::Satisfiable => "SATISFIABLE".to_string(),
        ConclusionResult::EnumerationComplete => {
            format!("COMPLETE ENUMERATION OF {} SOLUTIONS", lower_bound.unwrap())
        }
        ConclusionResult::EnumerationPartial => {
            format!("PARTIAL ENUMERATION OF {} SOLUTIONS", lower_bound.unwrap())
        }
        ConclusionResult::Bounds => format!(
            "BOUNDS {} <= obj <= {}",
            lower_bound.unwrap(),
            upper_bound.unwrap()
        ),
    };

    if assertion_used {
        println!("s UNDER ASSERTIONS {conclusion_status}");
    } else {
        println!("s VERIFIED {conclusion_status}");
    }
}

impl Elaboratable for ConclusionChecker {
    fn elaborate<C: ConstraintLookup, V: VarNameLookup>(
        &self,
        elaborator: &mut Elaborator,
        database: &C,
        var_names: &V,
    ) -> Result<(), ElaborationError> {
        let inner = &self.inner;
        match inner.result {
            ConclusionResult::None => elaborator.write_conclusion_none(),
            ConclusionResult::Satisfiable => {
                elaborator.write_conclusion_sat(var_names, self.propagated_solution.as_ref())
            }
            ConclusionResult::Unsatisfiable => elaborator
                .write_conclusion_unsat(database, inner.constraint_id_hint.unwrap() as usize),
            ConclusionResult::Bounds => elaborator.write_conclusion_bounds(
                database,
                var_names,
                inner.lower_bound_or_count.as_ref().unwrap(),
                inner.upper_bound.as_ref().unwrap(),
                inner.constraint_id_hint.unwrap() as usize,
                self.propagated_solution.as_ref(),
            ),
            ConclusionResult::EnumerationComplete => elaborator
                .write_conclusion_enumeration_complete(
                    database,
                    inner.lower_bound_or_count.as_ref().unwrap(),
                    inner.constraint_id_hint.unwrap() as usize,
                ),
            ConclusionResult::EnumerationPartial => elaborator
                .write_conclusion_enumeration_partial(inner.lower_bound_or_count.as_ref().unwrap()),
        }
        Ok(())
    }
}
