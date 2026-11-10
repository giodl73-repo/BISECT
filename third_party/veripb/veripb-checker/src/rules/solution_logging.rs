use std::rc::Rc;

use ahash::AHashSet;

use colored::Colorize;
use malachite_bigint::BigInt;
use num_traits::{One, Zero};
use veripb_formula::{
    pb_constraint::{constraint_from_terms, constraint_from_terms_and_coeff_sum},
    prelude::*,
};
use veripb_rules::{
    ObjectiveValueRule, SolutionRule, SolutionRuleOutput,
    elaboration::{ConstraintLookup, Elaboratable, ElaborationError, Elaborator},
};

use crate::prelude::*;

/// Build the solution-improving constraint asserting the objective is better than `value`.
pub fn improving_constraint(objective: &PBObjective, value: &BigInt) -> Rc<DBConstraint> {
    let mut negated_terms: Vec<_> = objective.terms.values().cloned().collect();
    for term in negated_terms.iter_mut() {
        term.coeff = -term.coeff.clone();
    }
    let constraint =
        constraint_from_terms(negated_terms, BigInt::from(1) + &objective.constant - value)
            .into_smallest_type();
    Rc::new(DBConstraint::from(constraint))
}

/// Build the solution-excluding constraint blocking `assignment` on the preserved variables.
///
/// Preserved variables in `shrunk` are left out of the constraint, so that it excludes all
/// solutions that agree with `assignment` on the non-shrunk preserved variables.
pub fn excluding_constraint(
    preserved_vars: &AHashSet<VarIdx>,
    assignment: &Assignment<BooleanVar>,
    shrunk: &AHashSet<VarIdx>,
    var_names: &VarNameManager,
) -> Result<Rc<DBConstraint>, CheckingError> {
    let mut terms = Vec::with_capacity(preserved_vars.len());
    for &var in preserved_vars {
        if shrunk.contains(&var) {
            continue;
        }
        match assignment.get_value(var) {
            BoolValue::Unassigned => {
                return Err(CheckingError::SolutionExcludingPreservedVarUnassigned(
                    var.to_pretty_string(var_names).bold(),
                ));
            }
            BoolValue::Assigned(true) => {
                // The current solution sets the variable to true, so the solution excluding clause gets the negated literal.
                terms.push(GeneralPBTerm::new(1, Lit::from_var(var, true)));
            }
            BoolValue::Assigned(false) => {
                // The current solution sets the variable to false, so the solution excluding clause gets the positive literal.
                terms.push(GeneralPBTerm::new(1, Lit::from_var(var, false)));
            }
        }
    }
    let coeff_sum = terms.len() as i64;
    // Construct the clause that says that at least one literal must disagree with the current solution.
    let constraint = constraint_from_terms_and_coeff_sum(terms, 1_i64, coeff_sum);
    Ok(Rc::new(DBConstraint::from(constraint)))
}

#[derive(Debug)]
pub struct SolutionChecker {
    pub inner: SolutionRule,
    pub propagated_solution: Assignment<BooleanVar>,
    /// The variables explicitly marked as shrunk in the solution.
    pub shrunk: AHashSet<VarIdx>,
}

impl SolutionChecker {
    fn new(inner: SolutionRule) -> Self {
        let shrunk = inner.shrunk.iter().copied().collect();
        Self {
            inner,
            propagated_solution: Assignment::default(),
            shrunk,
        }
    }
}

impl Checkable for SolutionChecker {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        // Set flag that we introduced a solution-improving or solution-excluding constraint.
        match self.inner.rule_output {
            SolutionRuleOutput::Improving | SolutionRuleOutput::Excluding => {
                context.solution_logged = true
            }
            SolutionRuleOutput::None => {}
        }
        if self.inner.rule_output == SolutionRuleOutput::Improving && context.objective.is_none() {
            return Err(CheckingError::SolutionImprovingRequiresObjective);
        }
        if self.inner.rule_output == SolutionRuleOutput::Excluding {
            if context.objective.is_some() {
                return Err(CheckingError::SolutionExcludingWithObjective);
            }
            if !context.strong_solution_guarantees {
                return Err(CheckingError::SolutionExcludingWithoutStrongGuarantees);
            }
            if let Some(preserved_vars) = &context.preserved_variables
                && let Some(&var) = self.shrunk.iter().find(|var| !preserved_vars.contains(var))
            {
                return Err(CheckingError::SolutionShrunkVarNotPreserved(
                    var.to_pretty_string(&context.var_names).bold(),
                ));
            }
        }

        let trace_trail = context.get_trace_trail_setting(true);
        let assignment = check_solution(
            &context.var_names,
            context.objective.as_ref(),
            context.preserved_variables.as_ref(),
            &mut context.propagation_engine,
            database,
            &self.inner,
            &self.shrunk,
            trace_trail,
        )?;

        // Keep track of currently best solution.
        let value = if let Some(objective) = &context.objective {
            objective
                .evaluate(&assignment)
                .ok_or(CheckingError::ObjectiveUnassigned)?
        } else {
            BigInt::zero()
        };

        // Check if claimed objective value matches computed objective value.
        if context.objective.is_none() && self.inner.objective_value.is_some() {
            return Err(CheckingError::SolutionValueHintNoObjective);
        }
        if self
            .inner
            .objective_value
            .as_ref()
            .is_some_and(|claimed_value| *claimed_value != value)
        {
            return Err(CheckingError::SolutionIncorrectValue(
                self.inner.objective_value.clone().unwrap(),
                value,
            ));
        }
        if context
            .best_objective_value
            .as_ref()
            .is_none_or(|best_value| value < *best_value)
        {
            context.best_objective_value = Some(value.clone());
            if context.strong_solution_guarantees {
                context.best_valid_objective_value = Some(value.clone())
            }
        }

        // Store propagated solution for potential elaboration.
        self.propagated_solution = assignment;

        // Return constraint according to the output type of the rule.
        match self.inner.rule_output {
            SolutionRuleOutput::None => Ok(vec![]),
            SolutionRuleOutput::Excluding => {
                if context.preserved_set_changed {
                    return Err(CheckingError::SolutionExcludingPreservedVarChanged);
                }
                let Some(preserved_vars) = &context.preserved_variables else {
                    return Err(CheckingError::NoPreservedForSolutionExcluding);
                };
                // Every shrunk (preserved) variable doubles the number of excluded solutions.
                context.num_excluded_solutions += BigInt::one() << self.shrunk.len();
                Ok(vec![excluding_constraint(
                    preserved_vars,
                    &self.propagated_solution,
                    &self.shrunk,
                    &context.var_names,
                )?])
            }
            SolutionRuleOutput::Improving => Ok(vec![improving_constraint(
                context
                    .objective
                    .as_ref()
                    .expect("checked at the start of compute that Improving has an objective"),
                &value,
            )]),
        }
    }

    #[inline]
    fn add_constraints_to_core(&self, _context: &Context) -> bool {
        true
    }
}

impl Checkable for ObjectiveValueRule {
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        if context.objective.is_none() {
            return Err(CheckingError::ObjectiveImprovingRequiresObjective);
        }
        if context
            .best_objective_value
            .as_ref()
            .is_none_or(|best_value| self.value < *best_value)
        {
            context.best_objective_value = Some(self.value.clone());
        }
        let objective = context
            .objective
            .as_ref()
            .expect("objective presence is checked above");
        Ok(vec![improving_constraint(objective, &self.value)])
    }

    #[inline]
    fn add_constraints_to_core(&self, _context: &Context) -> bool {
        true
    }
}

impl From<SolutionRule> for SolutionChecker {
    fn from(inner: SolutionRule) -> Self {
        Self::new(inner)
    }
}

impl Elaboratable for SolutionChecker {
    fn elaborate<C: ConstraintLookup, V: VarNameLookup>(
        &self,
        elaborator: &mut Elaborator,
        _database: &C,
        var_lookup: &V,
    ) -> Result<(), ElaborationError> {
        elaborator.write_solution(
            &self.inner.rule_output,
            &self.propagated_solution,
            &self.inner.shrunk,
            self.inner.objective_value.as_ref(),
            var_lookup,
        );
        Ok(())
    }
}
