use std::rc::Rc;

use ahash::AHashSet;
use itertools::Itertools;
use veripb_formula::prelude::*;
use veripb_parser::{
    opb_parser::parse_opb_from_file_given_var_manager,
    wcnf_parser::parse_wcnf_from_file_given_var_manager,
};
use veripb_rules::keywords::{DERIVABLE, EQUIENUM, EQUIOPT, EQUISAT};
use veripb_rules::{OutputGuarantee, OutputRule, OutputType};

use crate::prelude::*;

#[inline]
fn parse_output_file(
    context: &mut Context,
) -> Result<(Formula, Option<AHashSet<VarIdx>>), CheckingError> {
    if let Some(output_formula_path) = &context.config.output_formula {
        match FormulaFormat::resolve(context.config.formula_format_override, output_formula_path) {
            FormulaFormat::CNF => unimplemented!("CNF output formulas are not supported"),
            FormulaFormat::WCNF => Ok((
                parse_wcnf_from_file_given_var_manager(
                    output_formula_path,
                    &mut context.var_names,
                )?,
                None,
            )),
            FormulaFormat::OPB => {
                let (formula, _, preserved_vars) = parse_opb_from_file_given_var_manager(
                    output_formula_path,
                    &mut context.var_names,
                )?;
                Ok((formula, preserved_vars))
            }
        }
    } else {
        Err(CheckingError::NoOutputFormulaGiven)
    }
}

#[inline]
fn check_file(
    context: &mut Context,
    database: &mut Database,
    check_preserved_vars: bool,
) -> Result<(), CheckingError> {
    // Parse the output formula.
    let (output_formula, preserved_vars) = parse_output_file(context)?;

    // Check that the objectives are equivalent.
    match (&context.objective, output_formula.objective) {
        (None, None) => {}
        (Some(current_objective), Some(output_objective)) => {
            if *current_objective != output_objective {
                if current_objective.constant != output_objective.constant {
                    return Err(CheckingError::OutputObjectiveConstantMismatch(
                        current_objective.constant.clone(),
                        output_objective.constant,
                    ));
                }
                let (current_extra, output_extra) = current_objective.difference(&output_objective);
                return Err(CheckingError::OutputObjectiveMismatch(
                    current_extra
                        .iter()
                        .map(|t| t.to_pretty_string(&context.var_names))
                        .join(", "),
                    output_extra
                        .iter()
                        .map(|t| t.to_pretty_string(&context.var_names))
                        .join(", "),
                ));
            }
        }
        (None, Some(_)) => return Err(CheckingError::OutputObjectiveButNoProofObjective),
        (Some(_), None) => return Err(CheckingError::OutputNoObjectiveButProofObjective),
    }

    // Check if every constraint in output formula is in core constraints.
    database.update_unique_index(&mut context.propagation_engine)?;
    for output_constraint in output_formula.constraints {
        let output_constraint = Rc::new(output_constraint.into());
        if let Some(database_constraint) = database.lookup(&output_constraint) {
            if database_constraint.is_core_constraint() {
                // Mark constraints for check that every core constraint is in output formula.
                database_constraint.header.borrow_mut().is_in_output_formula = true;
            } else {
                return Err(CheckingError::OutputConstraintNotInCore(
                    database_constraint.get_some_id(),
                ));
            }
        } else {
            return Err(CheckingError::OutputConstraintNotInDatabase(
                output_constraint.to_pretty_string(&context.var_names),
            ));
        }
    }

    // Check if every core constraint was marked by a output formula constraint.
    for constraint in database.unique_constraints.iter() {
        if constraint.is_core_and_not_in_output_constraint() {
            return Err(CheckingError::OutputCoreConstraintNotInOutput(
                constraint.get_some_id(),
            ));
        }
    }

    // Check if preserved sets are equal when guarantee is `EQUIENUMERABLE`.
    if check_preserved_vars {
        match (&context.preserved_variables, preserved_vars) {
            (None, None) => {}
            (Some(current_vars), Some(output_vars)) => {
                if *current_vars != output_vars {
                    let missing_vars = current_vars
                        .difference(&output_vars)
                        .map(|v| v.to_pretty_string(&context.var_names))
                        .join(", ");
                    let additional_vars = output_vars
                        .difference(current_vars)
                        .map(|v| v.to_pretty_string(&context.var_names))
                        .join(", ");
                    return Err(CheckingError::OutputPreservedVarsMismatch(
                        missing_vars,
                        additional_vars,
                    ));
                }
            }
            (None, Some(_)) => return Err(CheckingError::OutputCurrentHasNoPreservedVars),
            (Some(_), None) => return Err(CheckingError::OutputFormulaHasNoPreservedVars),
        }
    }

    Ok(())
}

impl Checkable for OutputRule {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        if context.has_output {
            return Err(CheckingError::DoubleOutput);
        }
        if context.has_conclusion || context.has_end_proof {
            return Err(CheckingError::WrongFooterOrder("output"));
        }
        if !context.subcontexts.is_empty() {
            return Err(CheckingError::OutputWhileOpenSubcontext);
        }

        // Check for checked deletion condition.
        if !context.strong_solution_guarantees
            && (self.guarantee == OutputGuarantee::Equioptimal
                || self.guarantee == OutputGuarantee::Equisatisfiable
                || self.guarantee == OutputGuarantee::Equienumerable)
        {
            return Err(CheckingError::ExpectedStrongerGuarantees);
        }

        // Check conditions for `EQUIENUMERABLE`.
        if self.guarantee == OutputGuarantee::Equienumerable {
            if context.preserved_variables.is_none() && context.witnesses_used {
                return Err(CheckingError::OutputEquienumWitnessUsed);
            }
            if context.solution_logged {
                return Err(CheckingError::OutputEquienumSolutionLogged);
            }
        }

        // Check that right guarantee was used.
        match self.guarantee {
            OutputGuarantee::Equioptimal if context.objective.is_none() => {
                return Err(CheckingError::OutputEquioptimalWithoutObjective);
            }
            OutputGuarantee::Equioptimal => {}
            OutputGuarantee::Equisatisfiable if context.objective.is_some() => {
                return Err(CheckingError::OutputEquisatisfiableWithObjective);
            }
            _ => {}
        }

        // Check that the output is equivalent to the current problem.
        match self.output_type {
            OutputType::None => {}
            OutputType::Implicit => {}
            OutputType::File => check_file(
                context,
                database,
                self.guarantee == OutputGuarantee::Equienumerable,
            )?,
            _ => {
                unimplemented!()
            }
        }

        // Warn the user that proof used unchecked assertions before printing `s` status lines.
        if context.assertion_used {
            log::warn!("The proof used unchecked assertions.");
            if context.elaborator.is_some() {
                log::warn!(
                    "Unchecked assertions (unsupported in the kernel format) have been written to the elaborated proof."
                );
            }
        }

        if context.config.print_verification_result && self.guarantee != OutputGuarantee::None {
            let output_status = match self.guarantee {
                OutputGuarantee::Derivable => format!("OUTPUT {DERIVABLE}"),
                OutputGuarantee::Equioptimal => {
                    if let Some(best_value) = &context.best_objective_value {
                        format!("OUTPUT {EQUIOPT} FOR obj < {best_value}")
                    } else {
                        format!("OUTPUT {EQUIOPT}")
                    }
                }
                OutputGuarantee::Equisatisfiable => format!("OUTPUT {EQUISAT}"),
                OutputGuarantee::Equienumerable => format!("OUTPUT {EQUIENUM}"),
                OutputGuarantee::None => String::new(),
            };

            if context.assertion_used {
                println!("s UNDER ASSERTIONS {output_status}");
            } else {
                println!("s VERIFIED {output_status}");
            }
        }

        context.has_output = true;
        Ok(vec![])
    }
}
