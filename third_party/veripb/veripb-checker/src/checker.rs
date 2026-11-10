//! Implementation of main [`ForwardsChecker`] proof processor.

use std::{
    fs::File,
    io::{BufWriter, Error, Write},
    path::Path,
};

use crate::{
    config::RangeSet,
    error::ForwardsCheckerError,
    order::ActiveOrder,
    rules::{Checkable, CheckableRule, VeriPBRule},
};
use colored::Colorize;
use malachite_bigint::BigInt;
use veripb_formula::prelude::*;
use veripb_parser::pbp_parser::proof_processor::ProofProcessor;
use veripb_rules::elaboration::{Elaboratable, Elaborator};

use crate::{context::Context, database::Database, error::CheckingError};

/// Main data structure of the VeriPB checker.
///
/// This data structure maintains the invariants of the proof.
///
/// To initalize the data structure, create a new [`ForwardsChecker`] using [`ForwardsChecker::new()`].
///
/// A rule can be checked using [`ForwardsChecker::execute_rule()`]
pub struct ForwardsChecker {
    pub context: Context,
    pub database: Database,
    returned_constraint_id: Option<isize>,
}

impl VarNameRecorder for ForwardsChecker {
    fn add_by_name(&mut self, name: &str) -> VarIdx {
        self.context.var_names.add_by_name(name)
    }
}

/// If an `active_order` is [`Some`], ensure the `var_id` is a valid auxiliary variable with respect to the order.
pub fn validate_aux_var(
    var_id: VarIdx,
    active_order: Option<&ActiveOrder>,
) -> Result<(), CheckingError> {
    if !active_order.is_none_or(|order| order.order.is_auxiliary_variable(var_id)) {
        return Err(CheckingError::InvalidAuxVar);
    }
    Ok(())
}

impl ProofProcessor for ForwardsChecker {
    type Error = ForwardsCheckerError;

    fn last_returned_constraint_id(&mut self) -> Option<isize> {
        self.get_returned_constraint_id()
    }

    fn process_version(&mut self, major: i64, minor: i64) {
        // Carefully update this if a major/minor version larger than 255 gets allowed!
        self.context.major_version = Some(major as u8);
        self.context.minor_version = Some(minor as u8);
    }

    fn process_var_name(
        &mut self,
        line: usize,
        name: &str,
        is_aux_var: bool,
    ) -> Result<usize, ForwardsCheckerError> {
        let var_id = self.context.var_names.add_by_name(name);
        if is_aux_var {
            validate_aux_var(var_id, self.context.active_order.as_ref())
                .map_err(|e| e.add_context(self.context.config.derivation_path.clone(), line))?;
        }
        Ok(var_id)
    }

    fn process_rule(&mut self, line: usize, rule: VeriPBRule) -> Result<(), ForwardsCheckerError> {
        self.execute_rule(line, rule)
    }

    fn should_trace(&self, line: usize) -> bool {
        self.context
            .should_trace(line, self.database.len() as isize)
    }
}

impl ForwardsChecker {
    /// Construct a new `ForwardsChecker` from an existing [`Context`] and a [`Formula`].
    ///
    /// The `formula` is used to initialize the database. The elaborator is initialized based on the elaborated proof file path given in the `config` of `context`.
    pub fn new(mut context: Context, formula: Formula) -> Result<Self, Error> {
        // Initialize elaborator if needed.
        if let Some(output_file) = context.config.elaboration_path.clone() {
            let mut elaborator = Elaborator::new(output_file)?;
            elaborator.write_load_formula(formula.len());
            context.elaborator = Some(elaborator);
        }

        // Initialize database.
        let mut database = Database::from_formula(formula.constraints.clone());
        context.objective = formula.objective.clone();

        // Set up output constraint IDs for original constraints.
        if let Some(elaborator) = context.elaborator.as_mut() {
            for (orig_id, constraint) in database.entries.iter_mut().flatten().enumerate() {
                constraint.set_out_id(orig_id + 1, elaborator.inc_id());
            }
        }

        // Keep original constraints and objective.
        context.original_constraints = formula.constraints;
        context.original_objective = formula.objective;

        let verifier = ForwardsChecker {
            context,
            database,
            returned_constraint_id: None,
        };

        // The formula constraints are assigned IDs starting at 1, so print them
        // for full tracing or when specific IDs are selected with `--trace-ids`.
        if verifier.context.config.trace || verifier.context.config.trace_ids.is_some() {
            print_formula(
                verifier.context.config.trace,
                verifier.context.config.trace_ids.as_ref(),
                verifier.context.objective.as_ref(),
                &verifier.database,
                &verifier.context.var_names,
            );
        }

        Ok(verifier)
    }

    /// Get the last returned constraint ID.
    ///
    /// This ID is usually the ID of the constraint that was just derived. For the `e` (equals) rule this is the ID of the constraint to which the given constraint is equal.
    pub fn get_returned_constraint_id(&mut self) -> Option<isize> {
        let result = self.returned_constraint_id;
        self.returned_constraint_id = None;
        result
    }

    /// Execute a rule.
    ///
    /// This performs the following checking in order:
    /// 1. Preliminary checks that this rule can be used.
    /// 2. Check that the rule can be applied and calculated the derived constraint.
    /// 3. Add the derived constraints to the database.
    /// 4. Elaborate the rule.
    /// 5. Delete constraints deleted by this rule.
    /// 6. Change the database.
    pub fn execute_rule(
        &mut self,
        line: usize,
        rule: VeriPBRule,
    ) -> Result<(), ForwardsCheckerError> {
        // Determine whether this rule should be traced. The next constraint ID
        // a rule introduces is the current database length.
        self.context.trace_enabled = self
            .context
            .should_trace(line, self.database.len() as isize);

        // Check if we are in a subproof and if the rule can be used within a subproof.
        let last_subcontext = self.context.subcontexts.last();
        if last_subcontext.is_some()
            && last_subcontext.unwrap().is_subproof()
            && !rule.is_subproof_friendly()
        {
            return Err(CheckingError::RuleNotSubproofFriendly(
                std::any::type_name_of_val(&rule).to_string(),
            )
            .add_context(self.context.config.derivation_path.clone(), line)
            .into());
        }

        // Construct a CheckableRule from the VeriPBRule (may have extra members needed for checking)
        let mut rule: CheckableRule = rule.into();

        // Do the checks for the rule.
        let new_constraints = rule
            .compute(&mut self.context, &mut self.database)
            .map_err(|e| e.add_context(self.context.config.derivation_path.clone(), line))?;
        let num_new_constraints = new_constraints.len();

        // Add the new constraints.
        for constraint in new_constraints {
            if self.context.trace_enabled {
                println!(
                    "  ConstraintId {}: {}",
                    if rule.add_constraints_to_core(&self.context) {
                        self.database.len().to_string().yellow()
                    } else {
                        self.database.len().to_string().bright_green()
                    },
                    constraint
                        .constraint
                        .to_pretty_string(&self.context.var_names)
                        .blue()
                )
            }
            // Add constraint ID to current level.
            if let Some(level) = self.context.current_level {
                unsafe {
                    self.context
                        .level_ids
                        .get_unchecked_mut(level)
                        .push(self.database.len())
                };
            }
            // Add constraint to database.
            self.database.add_constraint(
                constraint,
                self.context.only_core || rule.add_constraints_to_core(&self.context),
            );
        }

        // Get ID of returned constraint.
        self.returned_constraint_id = rule
            .get_returned_id()
            .or_else(|| Some(self.database.last_constraint_id()));

        // Elaborate the rule.
        if let Some(elaborator) = self.context.elaborator.as_mut() {
            // Add output constraint ID for elaborated proof.
            let start_id_new_constraints = self.database.len() - num_new_constraints;
            for (constraint_id, constraint) in (start_id_new_constraints..).zip(
                self.database.entries[start_id_new_constraints..]
                    .iter()
                    .flatten(),
            ) {
                constraint.set_out_id(constraint_id, elaborator.inc_id());
            }

            rule.elaborate(elaborator, &self.database, &self.context.var_names)?;
        }

        // Delete constraints also implicit through ending subproofs.
        if let Some(deletion_sequence) = rule
            .get_deleted(&self.database)
            .map_err(|e| e.add_context(self.context.config.derivation_path.clone(), line))?
        {
            if self.context.trace_enabled {
                let deletion_string = deletion_sequence.trace_deletions(&self.database);
                if !deletion_string.is_empty() {
                    println!("  Deleting IDs: {deletion_string}");
                }
            }

            deletion_sequence
                .delete_constraints(&mut self.database, &mut self.context)
                .map_err(|e| e.add_context(self.context.config.derivation_path.clone(), line))?
        }

        // Change the currently considered database.
        if rule.swaps_database() {
            let (database, prop_engine) = rule.get_new_database();
            let old_database = std::mem::replace(&mut self.database, database);
            let old_prop_engine =
                std::mem::replace(&mut self.context.propagation_engine, prop_engine);
            rule.handle_old_database(&mut self.context, old_database, old_prop_engine);
        }

        Ok(())
    }
}

/// Print the formula to the trace.
///
/// Can either trace everything, or a just selected range of ids (`trace_all` takes precedence).
pub fn print_formula(
    trace_all: bool,
    trace_ids: Option<&RangeSet>,
    objective: Option<&PBObjective>,
    database: &Database,
    var_names: &impl VarNameLookup,
) {
    if trace_all && let Some(objective) = objective {
        println!(
            "  {}: {}",
            "Objective".bright_green(),
            ("min ".to_owned() + objective.to_pretty_string(var_names).as_str()).green()
        )
    }
    for (idx, constraint) in database.entries.iter().enumerate() {
        if let Some(constraint) = constraint
            && (trace_all || trace_ids.is_some_and(|ids| ids.contains(idx)))
        {
            println!(
                "  ConstraintId {}: {}",
                idx.to_string().yellow(),
                constraint.constraint.to_pretty_string(var_names).blue()
            )
        }
    }
}

/// Write a dump of the final constraint database to `path`.
///
/// Every live (non-deleted) constraint is written on its own line, in ascending order of
/// constraint ID. An ID is absent if the constraint was deleted, or if the proof line
/// that would have introduced it was never reached because an earlier line failed to
/// check -- there is no explicit marker for a missing ID.
///
/// # Format
///
/// ```text
/// pseudo-Boolean database dump version 1
/// <id> <core|derived> <constraint> ;
/// <id> <core|derived> <constraint> ;
/// ...
/// end pseudo-Boolean database dump;
/// ```
///
/// For example:
///
/// ```text
/// pseudo-Boolean database dump version 1
/// 1 core 1 x1 1 x2 >= 1 ;
/// 2 core 1 ~x1 1 x2 >= 1 ;
/// 3 core 1 ~x2 >= 1 ;
/// 4 derived 1 x2 >= 1 ;
/// 5 derived >= 1 ;
/// end pseudo-Boolean database dump;
/// ```
pub fn dump_database(
    path: &Path,
    database: &Database,
    var_names: &impl VarNameLookup,
) -> Result<(), Error> {
    let mut out = BufWriter::new(File::create(path)?);
    writeln!(out, "pseudo-Boolean database dump version 1")?;
    for (id, constraint) in database.entries.iter().enumerate() {
        if let Some(constraint) = constraint {
            let kind = if constraint.is_core_constraint_id(id) {
                "core"
            } else {
                "derived"
            };
            writeln!(
                out,
                "{id} {kind} {} ;",
                constraint.constraint.to_pretty_string(var_names)
            )?;
        }
    }
    writeln!(out, "end pseudo-Boolean database dump;")?;
    Ok(())
}

/// Write a dump of the objective function and best known objective values to `path`.
///
/// Writes the current objective function (reflecting any `obju` update rules applied so
/// far in the proof), the best objective value logged overall, and the best objective
/// value logged while checked deletion was in effect (i.e. still valid under the proof's
/// checked guarantees). The objective is `none` if the problem has no objective; either
/// value is `none` if no (valid) improving solution was logged.
///
/// # Format
///
/// ```text
/// pseudo-Boolean objective dump version 1
/// objective: min <coeff> <lit> ... <constant> | none ;
/// best_objective_value: <value|none> ;
/// best_valid_objective_value: <value|none> ;
/// end pseudo-Boolean objective dump;
/// ```
///
/// For example:
///
/// ```text
/// pseudo-Boolean objective dump version 1
/// objective: min 2 x1 0 ;
/// best_objective_value: 2 ;
/// best_valid_objective_value: 2 ;
/// end pseudo-Boolean objective dump;
/// ```
pub fn dump_objective(
    path: &Path,
    objective: Option<&PBObjective>,
    best_objective_value: Option<&BigInt>,
    best_valid_objective_value: Option<&BigInt>,
    var_names: &impl VarNameLookup,
) -> Result<(), Error> {
    let mut out = BufWriter::new(File::create(path)?);
    writeln!(out, "pseudo-Boolean objective dump version 1")?;
    writeln!(
        out,
        "objective: {} ;",
        objective
            .map(|o| format!("min {}", o.to_pretty_string(var_names)))
            .unwrap_or_else(|| "none".to_string())
    )?;
    writeln!(
        out,
        "best_objective_value: {} ;",
        best_objective_value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_string())
    )?;
    writeln!(
        out,
        "best_valid_objective_value: {} ;",
        best_valid_objective_value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_string())
    )?;
    writeln!(out, "end pseudo-Boolean objective dump;")?;
    Ok(())
}
