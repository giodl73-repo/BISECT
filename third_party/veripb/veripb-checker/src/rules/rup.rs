use std::{num::NonZeroUsize, rc::Rc};

use colored::Colorize;
use veripb_formula::{prelude::*, propagation::RUPHint};
use veripb_propagator::propagation_engine::TraceTrail;
use veripb_rules::RUPRule;

use crate::prelude::*;

/// Pretty print the current trail, given a set of hints.
fn maybe_trace_trail_with_hints(
    hint: &[isize],
    database: &Database,
    context: &mut Context,
    negated: DBConstraint,
    is_failure: bool,
    trace_trail: TraceTrail,
) {
    if trace_trail.should_skip(is_failure) {
        return;
    }
    context.annotated_rup_assignment.reset();

    if is_failure {
        println!("Propagation check failed! The propagation had the following trail:");
    } else {
        println!("Propagation check succeeded. The propagation had the following trail:");
    }
    println!("  propagations in format: <assignment> (<reason constraint>)");

    loop {
        let mut unchanged = true;
        for id in hint.iter() {
            let id = database.normalize_id(*id);
            let reason = if id == 0 {
                &negated
            } else {
                database.get_entry_usize(id as usize).unwrap()
            };
            if reason.is_falsified(&context.annotated_rup_assignment) {
                println!(
                    "    {} ({})",
                    "*conflict*".red(),
                    reason.to_pretty_string(&context.var_names).cyan(),
                );
                return;
            }
            for lit in reason.traced_propagate(&mut context.annotated_rup_assignment) {
                unchanged = false;
                println!(
                    "    {} ({})",
                    lit.to_pretty_string(&context.var_names).purple(),
                    reason.to_pretty_string(&context.var_names).cyan(),
                )
            }
        }
        if unchanged {
            return;
        }
    }
}

impl Checkable for RUPRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        match &self.hint {
            None => {
                // Do unit propagation check to see if constraint is implied.
                database.update_propagation_index(&mut context.propagation_engine)?;
                if context
                    .propagation_engine
                    .reverse_unit_propagation_check(
                        &context.var_names,
                        &[],
                        Some(&self.constraint),
                        context.only_core,
                        context.elaborator.is_some(),
                        context.get_trace_trail_setting(true),
                    )?
                    .is_conflict()
                {
                    if let Some(elaborator) = context.elaborator.as_mut() {
                        elaborator.write_rup_step(
                            &self.constraint.constraint,
                            &context.propagation_engine.rup_hints,
                            &context.var_names,
                        );
                    }
                    Ok(vec![self.constraint.clone()])
                } else {
                    Err(CheckingError::NotRUP(context.only_core))
                }
            }
            Some(hint) => {
                hinted_rup_propagation(
                    &self.constraint,
                    context,
                    database,
                    hint,
                    context.elaborator.is_some(),
                )?;
                if let Some(elaborator) = context.elaborator.as_mut() {
                    elaborator.write_rup_step(
                        &self.constraint.constraint,
                        &context.annotated_rup_hints,
                        &context.var_names,
                    );
                }
                Ok(vec![self.constraint.clone()])
            }
        }
    }
}

// Do propagation based on the constraint IDs given as hint, saving the annotated
// RUP hints for elaboration.
pub fn hinted_rup_propagation(
    constraint: &Rc<DBConstraint>,
    context: &mut Context,
    database: &mut Database,
    given_hints: &Vec<isize>,
    collect_hints: bool,
) -> Result<(), CheckingError> {
    context.annotated_rup_assignment.reset();
    context
        .annotated_rup_assignment
        .resize(context.var_names.len());
    context.annotated_rup_hints.clear();
    let negated = constraint.negate();
    let conflict = 'propagate: loop {
        let mut unchanged = true;
        for id in given_hints {
            let id = database.normalize_id(*id);
            if id == 0 {
                match negated.propagate(&mut context.annotated_rup_assignment) {
                    ConstraintPropagationResult::Conflict => {
                        if collect_hints {
                            context.annotated_rup_hints.push(RUPHint::NegatedPremise);
                        }
                        break 'propagate true;
                    }
                    ConstraintPropagationResult::NoPropagation => {}
                    ConstraintPropagationResult::Propagated => {
                        unchanged = false;
                        if collect_hints {
                            context.annotated_rup_hints.push(RUPHint::NegatedPremise);
                        }
                    }
                }
            } else {
                let constraint = database.get_entry_usize(id as usize)?;
                if context.only_core && !constraint.is_core_constraint_id(id as usize) {
                    return Err(CheckingError::CoreSubproofUsingNonCoreConstraint(id));
                }
                match constraint.propagate(&mut context.annotated_rup_assignment) {
                    ConstraintPropagationResult::Conflict => {
                        if collect_hints {
                            context.annotated_rup_hints.push(RUPHint::ConstraintId(
                                NonZeroUsize::new(constraint.output_or_else_orig_id())
                                    .expect("should be non-zero"),
                            ));
                        }
                        break 'propagate true;
                    }
                    ConstraintPropagationResult::NoPropagation => {}
                    ConstraintPropagationResult::Propagated => {
                        unchanged = false;
                        if collect_hints {
                            context.annotated_rup_hints.push(RUPHint::ConstraintId(
                                NonZeroUsize::new(constraint.output_or_else_orig_id())
                                    .expect("should be non-zero"),
                            ))
                        }
                    }
                }
            };
        }

        if unchanged {
            break 'propagate false;
        }
    };

    let trace_trail = context.get_trace_trail_setting(true);
    maybe_trace_trail_with_hints(
        given_hints,
        database,
        context,
        negated,
        !conflict,
        trace_trail,
    );

    if conflict {
        Ok(())
    } else {
        Err(CheckingError::NotRUP(context.only_core))
    }
}
