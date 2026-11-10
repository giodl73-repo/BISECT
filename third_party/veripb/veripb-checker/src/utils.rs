//! This submodule contains useful helper functions that can be used in multiple places for the checker.

use std::{num::Saturating, rc::Rc};

use ahash::AHashSet;
use colored::Colorize;
use veripb_formula::prelude::*;
use veripb_propagator::propagation_engine::{
    CORE_CLAUSES_CARD, CORE_GENERAL, DERIVED_CLAUSES_CARD, DERIVED_GENERAL, PropagationEngine,
    TraceTrail,
};
use veripb_rules::{SolutionRule, SolutionRuleOutput};

use crate::{context::REQUIRED_RUP_STREAK, prelude::*, proofgoal::ProofTechnique};

/// Set up a new propagation engine for VeriPB.
#[inline]
pub fn new_veripb_propagation_engine() -> PropagationEngine {
    let mut propagation_engine = PropagationEngine::default();
    propagation_engine.add_set();
    propagation_engine.add_set();
    propagation_engine.add_set();
    propagation_engine.add_set();
    propagation_engine.add_set();
    propagation_engine.enable_back(CORE_CLAUSES_CARD);
    propagation_engine.enable_back(CORE_GENERAL);
    propagation_engine.enable_back(DERIVED_CLAUSES_CARD);
    propagation_engine.enable_back(DERIVED_GENERAL);
    propagation_engine
}

/// Build the assignment given by the solution literals of `sol_rule`, without checking it.
#[inline]
pub fn assignment_from_provided_solution(
    sol_rule: &SolutionRule,
    var_names: &VarNameManager,
) -> Result<Assignment<BooleanVar>, CheckingError> {
    let assignment = Assignment::from(&sol_rule.solution, var_names.len())
        .ok_or(CheckingError::SolutionIsConflicting)?;
    Ok(assignment)
}

/// Complete a partial solution by propagating its literals against the database.
#[inline]
pub fn complete_solution_by_propagation(
    sol_rule: &SolutionRule,
    var_names: &VarNameManager,
    prop_engine: &mut PropagationEngine,
    database: &mut Database,
    trace_trail: TraceTrail,
) -> Result<Assignment<BooleanVar>, CheckingError> {
    database.update_propagation_index(prop_engine)?;
    Ok(prop_engine.propagate_solution(&sol_rule.solution, var_names, trace_trail)?)
}

/// Check if the given solution or the propagated solution satisfies all core constraints and that
/// all necessary variables are assigned. Return the (propagated) solution as an [`Assignment`].
///
/// The variables in `shrunk` are explicitly left unassigned: they may neither be assigned by the
/// given solution nor by propagation, and they count as assigned when checking that all preserved
/// variables are assigned.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn check_solution(
    var_names: &VarNameManager,
    objective: Option<&PBObjective>,
    preserved_vars: Option<&AHashSet<VarIdx>>,
    prop_engine: &mut PropagationEngine,
    database: &mut Database,
    sol_rule: &SolutionRule,
    shrunk: &AHashSet<VarIdx>,
    trace_trail: TraceTrail,
) -> Result<Assignment<BooleanVar>, CheckingError> {
    let assignment = assignment_from_provided_solution(sol_rule, var_names)?;
    if let Some(&var) = shrunk.iter().find(|&&var| assignment.is_assigned(var)) {
        return Err(CheckingError::SolutionShrunkVarAssigned(
            var.to_pretty_string(var_names).bold(),
        ));
    }

    // Heuristic check if given solution already satisfies all core constraints and assigns the necessary variables.
    database.update_unique_index(prop_engine)?;
    if get_unsatisfied_core_constraint(database, &assignment)?.is_none()
        && solution_assigns_necessary_variables(
            var_names,
            objective,
            preserved_vars,
            &assignment,
            shrunk,
            &sol_rule.rule_output,
        )
        .is_ok()
    {
        return Ok(assignment);
    }

    // Propagate the solution.
    let assignment =
        complete_solution_by_propagation(sol_rule, var_names, prop_engine, database, trace_trail)?;

    // Check that no shrunk variable was propagated.
    if let Some(&var) = shrunk.iter().find(|&&var| assignment.is_assigned(var)) {
        return Err(CheckingError::SolutionShrunkVarPropagated(
            var.to_pretty_string(var_names).bold(),
        ));
    }

    // Check if the propagated solution satisfies all constraints.
    if let Some(id) = get_unsatisfied_core_constraint(database, &assignment)? {
        Err(CheckingError::SolutionNotSatisfiedConstraint(id))
    } else {
        solution_assigns_necessary_variables(
            var_names,
            objective,
            preserved_vars,
            &assignment,
            shrunk,
            &sol_rule.rule_output,
        )?;
        Ok(assignment)
    }
}

/// Check that every objective variable is assigned, as required to derive a solution-improving
/// constraint from an assignment.
#[inline]
pub fn objective_vars_assigned(
    var_names: &VarNameManager,
    objective: Option<&PBObjective>,
    assignment: &Assignment<BooleanVar>,
) -> Result<(), CheckingError> {
    let Some(objective) = objective else {
        return Err(CheckingError::SolutionImprovingRequiresObjective);
    };
    for &var in objective.terms.iter().map(|t| t.0) {
        if assignment.get_value(var) == BoolValue::Unassigned {
            return Err(CheckingError::SolutionVarUnassignedImproving(
                var.to_pretty_string(var_names),
            ));
        }
    }
    Ok(())
}

/// Check that every preserved variable is assigned or explicitly shrunk, as required to derive a
/// solution-excluding constraint.
#[inline]
pub fn preserved_vars_assigned(
    var_names: &VarNameManager,
    preserved_variables: Option<&AHashSet<VarIdx>>,
    assignment: &Assignment<BooleanVar>,
    shrunk: &AHashSet<VarIdx>,
) -> Result<(), CheckingError> {
    let Some(preserved) = preserved_variables else {
        return Err(CheckingError::NoPreservedForSolutionExcluding);
    };
    for &var in preserved.iter() {
        if assignment.get_value(var) == BoolValue::Unassigned && !shrunk.contains(&var) {
            return Err(CheckingError::SolutionVarUnassignedExcluding(
                var.to_pretty_string(var_names),
            ));
        }
    }
    Ok(())
}

/// Check if the requirements for the solution rule output are met.
///
/// Specifically, this means that all variables in the preserved set are assigned (or explicitly shrunk) if we want an excluding constraint and that all variables in the objective function are assigned if we want a solution-improving constraint.
#[inline]
fn solution_assigns_necessary_variables(
    var_names: &VarNameManager,
    objective: Option<&PBObjective>,
    preserved_variables: Option<&AHashSet<VarIdx>>,
    assignment: &Assignment<BooleanVar>,
    shrunk: &AHashSet<VarIdx>,
    solution_output: &SolutionRuleOutput,
) -> Result<(), CheckingError> {
    match solution_output {
        SolutionRuleOutput::None => Ok(()),
        SolutionRuleOutput::Improving => objective_vars_assigned(var_names, objective, assignment),
        SolutionRuleOutput::Excluding => {
            preserved_vars_assigned(var_names, preserved_variables, assignment, shrunk)
        }
    }
}

/// Check that the given solution satisfies all core constraints.
///
/// Returns the constraint ID of an unsatisfied constraints or `None` if all constraints are satisfied.
#[inline]
pub fn get_unsatisfied_core_constraint(
    database: &mut Database,
    assignment: &Assignment<BooleanVar>,
) -> Result<Option<usize>, CheckingError> {
    for constraint in database.unique_constraints.iter() {
        if constraint.is_core_constraint() && !constraint.is_satisfied(assignment) {
            // Core constraint is not satisfied.
            if constraint.is_falsified(assignment) {
                // Core constraint is falsified.
                return Err(CheckingError::SolutionFalsifyingConstraint(
                    constraint.get_some_id(),
                ));
            } else {
                // Core constraint is neither satisfied nor falsified.
                return Ok(Some(constraint.get_some_id()));
            }
        }
    }
    Ok(None)
}

/// Do the syntactic implication check with saturation.
///
/// This check can derive a constraint `target` from another constraint `source` by adding literal axioms to it, performing saturation and another round of adding literal axioms.
///
/// Already takes care to check if the `source` constraint is a core constraint in a core only subproof.
#[inline]
pub fn check_implication(
    only_core: bool,
    propagation_engine: &mut PropagationEngine,
    database: &mut Database,
    target: &Rc<DBConstraint>,
    hint: Option<isize>,
) -> Result<isize, CheckingError> {
    match hint {
        Some(hint) => {
            let index = database.normalize_id(hint);
            let entry = database.get_entry_usize(index as usize)?;
            if only_core && !entry.is_core_constraint_id(index as usize) {
                return Err(CheckingError::CoreSubproofUsingNonCoreConstraint(hint));
            }
            if !entry.implies(target) {
                return Err(CheckingError::not_implied(target, entry));
            }
            Ok(index)
        }
        None => {
            database.update_unique_index(propagation_engine)?;
            if let Some(constraint) = database.lookup(target)
                && (!only_core || constraint.is_core_constraint())
            {
                return Ok(constraint.get_some_id() as isize);
            }

            for constraint in database.unique_constraints.iter() {
                if (!only_core || constraint.is_core_constraint()) && constraint.implies(target) {
                    return Ok(constraint.get_some_id() as isize);
                }
            }

            Err(CheckingError::not_implied_db(target, only_core))
        }
    }
}

/// Do the syntactic implication check with saturation for the substituted database.
///
/// This check is stronger than the syntactic implication check, since it substitutes `source` and `target` constraints using the current unit propagation before checking if the `source` implies `target`.
///
/// Already takes care to check if the `source` constraint is a core constraint in a core only subproof.
#[inline]
pub fn check_substituted_implication(
    context: &mut Context,
    database: &mut Database,
    target: &Rc<DBConstraint>,
    hint: Option<isize>,
) -> Result<isize, CheckingError> {
    match hint {
        Some(hint) => {
            let propagated_assignment = context
                .propagation_engine
                .get_propagated_assignment(context.var_names.len(), context.only_core);
            let sub_target = &Rc::new(target.substitute(propagated_assignment));
            let index = database.normalize_id(hint);
            let entry = database.get_entry_usize(index as usize)?;
            let sub_source = &Rc::new(entry.substitute(propagated_assignment));
            if context.only_core && !entry.is_core_constraint_id(index as usize) {
                return Err(CheckingError::CoreSubproofUsingNonCoreConstraint(hint));
            }
            if !sub_source.implies(sub_target) {
                return Err(CheckingError::not_implied_substituted(sub_target, entry));
            }
            Ok(index)
        }
        None => {
            database.update_unique_index(&mut context.propagation_engine)?;
            let propagated_assignment = context
                .propagation_engine
                .get_propagated_assignment(context.var_names.len(), context.only_core);
            let sub_target = &Rc::new(target.substitute(propagated_assignment));
            if let Some(constraint) = database.lookup(target)
                && (!context.only_core || constraint.is_core_constraint())
            {
                return Ok(constraint.get_some_id() as isize);
            }
            if let Some(constraint) = database.lookup(sub_target)
                && (!context.only_core || constraint.is_core_constraint())
            {
                return Ok(constraint.get_some_id() as isize);
            }

            for constraint in database.unique_constraints.iter() {
                if !context.only_core || constraint.is_core_constraint() {
                    let sub_constraint = constraint.substitute(propagated_assignment);
                    if sub_constraint.implies(sub_target) {
                        return Ok(constraint.get_some_id() as isize);
                    }
                }
            }

            Err(CheckingError::not_implied_substituted_db(
                sub_target,
                context.only_core,
            ))
        }
    }
}

/// Attempt a top level autoproving check for a RUP constraint.
/// Returns None if unsuccessful.
pub fn try_rup(
    constraint: &Rc<DBConstraint>,
    context: &mut Context,
    database: &mut Database,
) -> Result<Option<ProofTechnique>, CheckingError> {
    database.update_propagation_index(&mut context.propagation_engine)?;
    if context
        .propagation_engine
        .reverse_unit_propagation_check(
            &context.var_names,
            &[],
            Some(constraint),
            context.only_core,
            context.elaborator.is_some(),
            TraceTrail::No,
        )?
        .is_conflict()
    {
        if context.trace_enabled {
            println!("* constraint is RUP *")
        }
        return Ok(Some(ProofTechnique::IsRUP));
    }
    Ok(None)
}

/// Attempt a top level database lookup check.
/// Returns None if unsuccessful.
pub fn try_db_lookup(
    constraint: &Rc<DBConstraint>,
    context: &mut Context,
    database: &mut Database,
) -> Result<Option<ProofTechnique>, CheckingError> {
    database.update_unique_index(&mut context.propagation_engine)?;
    if let Some(found_constraint) = database.lookup(constraint)
        && (!context.only_core || found_constraint.is_core_constraint())
    {
        if context.trace_enabled {
            println!("* constraint is in DB already")
        }

        return Ok(Some(ProofTechnique::DBLookup(
            constraint.clone(),
            found_constraint.clone(),
        )));
    }
    Ok(None)
}

type ShortcutCheck = fn(
    &Rc<DBConstraint>,
    &mut Context,
    &mut Database,
) -> Result<Option<ProofTechnique>, CheckingError>;

/// For deletion and redundance based strengthening, attempt to shortcut the full proof obligation
/// dispatch by trying RUP and/or database lookup.
///
/// NB: context.rup_streak records how often top level RUP was previously successful and will skip
/// the attempt if there has not been enough success yet.
pub fn try_shortcuts(
    constraint: &Rc<DBConstraint>,
    context: &mut Context,
    database: &mut Database,
    has_subproof: bool,
    witness: &Substitution,
    to_try: &[ShortcutCheck],
) -> Result<Option<ProofTechnique>, CheckingError> {
    if has_subproof || (!witness.is_empty() && context.rup_streak < REQUIRED_RUP_STREAK) {
        return Ok(None);
    }

    for try_fn in to_try {
        let technique = try_fn(constraint, context, database)?;
        if technique.is_some() {
            return Ok(technique);
        }
    }

    context.rup_streak = Saturating(0);

    Ok(None)
}
