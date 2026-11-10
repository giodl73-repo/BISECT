//! Proof context for the [`ForwardsChecker`] storing the state of the checker.

use std::{
    num::Saturating,
    time::{Duration, Instant},
};

use ahash::{AHashMap, AHashSet};
use colored::Colorize;
use itertools::Itertools;
use log::warn;
use malachite_bigint::BigInt;
use num_traits::Zero;
use veripb_formula::{prelude::*, propagation::RUPHint};
use veripb_propagator::propagation_engine::{PropagationEngine, TraceTrail};
use veripb_rules::{ObjectiveUpdateType, elaboration::Elaborator};

use crate::{
    order_context::{OrderContext, ReflexivityContext, SpecificationContext, TransitivityContext},
    prelude::*,
    proofgoal_context::ProofgoalContext,
};

/// Index of temporary propagation sets for autoproving.
pub const AUTOPROVING: usize = 4;

/// Constant for heuristic trying RUP for `red`-rule or deletion before generating proofgoals.
pub const REQUIRED_RUP_STREAK: Saturating<u8> = Saturating(5);

/// A [`Subcontext`] stores information that is specific to an environment in the proof.
///
/// The meaning and usage of each variant of [`Subcontext`] are described below.
#[derive(Debug)]
pub enum Subcontext {
    /// A top-level subproof for a proof rule started with the keyword `subproof`.
    Subproof(SubproofContext),
    /// A subproof for a proofgoal inside a top-level subproof. `Proofgoal` subproofs are started by the `proofgoal`-rule.
    Proofgoal(ProofgoalContext),
    /// Subcontext for defining an order.
    Order(OrderContext),
    /// Subcontext for showing that an order is transitive.
    Transitivity(TransitivityContext),
    /// Subcontext for showing that an order is reflexive.
    Reflexivity(ReflexivityContext),
    /// Subcontext for deriving the auxiliary specification for and order.
    Specification(SpecificationContext),
}

impl Subcontext {
    /// Check if the subcontext is a top-level subproof or proofgoal subproof.
    #[inline]
    pub fn is_subproof(&self) -> bool {
        matches!(self, Subcontext::Subproof(_) | Subcontext::Proofgoal(_))
    }
}

/// Context to store the state of the [`ForwardsChecker`].
///
/// This can be thought of as the configuration maintained by the checker.
#[derive(Debug, Default)]
pub struct Context {
    /// Static configuration options (e.g. from command-line arguments)
    pub config: CheckerConfig,
    /// Stronger guarantees that solutions to current problem are solutions to original problem.
    pub strong_solution_guarantees: bool,
    /// Storage of variable names and used variables.
    pub var_names: VarNameManager,
    /// Original formula.
    pub original_constraints: Vec<PBConstraintEnum>,
    /// Original objective.
    pub original_objective: Option<PBObjective>,
    /// Current objective.
    pub objective: Option<PBObjective>,
    /// The currently best objective value logged while checked deletion was enabled.
    pub best_valid_objective_value: Option<BigInt>,
    /// The currently best objective value over all.
    pub best_objective_value: Option<BigInt>,
    /// Stack of currently used subproof contexts.
    pub subcontexts: Vec<Subcontext>,
    /// A flag to determine if we are inside a strengthening subproof, where some rules are not allowed.
    pub inside_strengthening_subproof: bool,
    /// Propagation engine for checks by propagation.
    pub propagation_engine: PropagationEngine,
    /// All orders known to checker.
    pub orders: AHashMap<String, Order>,
    /// The currently active order.
    pub active_order: Option<ActiveOrder>,
    /// Current streak autoproving red rules by the constraint being RUP.
    pub rup_streak: Saturating<u8>,
    /// Stored assignment for annotated RUP steps.
    pub annotated_rup_assignment: Assignment<BooleanVar>,
    /// Stored hints for annotated RUP steps.
    pub annotated_rup_hints: Vec<RUPHint>,
    /// Proof major version.
    pub major_version: Option<u8>,
    /// Proof minor version.
    pub minor_version: Option<u8>,
    /// Output has been checked.
    pub has_output: bool,
    /// Conclusion has been checked
    pub has_conclusion: bool,
    /// End of proof has been encountered
    pub has_end_proof: bool,
    /// An assertion rule has been used in the proof.
    pub assertion_used: bool,
    /// Elaborator
    pub elaborator: Option<Elaborator>,
    /// The current level that is set
    pub current_level: Option<usize>,
    /// The storage of constraint IDs for each level
    pub level_ids: Vec<Vec<usize>>,
    /// Store if only core constraints can be used for proof.
    pub only_core: bool,
    /// Stores if the strengthening to core is more is enabled.
    pub is_strengthening_to_core: bool,
    /// The current set of preserved variables.
    pub preserved_variables: Option<AHashSet<VarIdx>>,
    /// If witnesses have been used before the preserved set has been initialized.
    pub witnesses_used: bool,
    /// A solution has been logged, which introduced an excluding constraint.
    pub solution_logged: bool,
    /// The preserved set has been changed.
    pub preserved_set_changed: bool,
    /// The number of solutions excluded by `solx`.
    ///
    /// A `solx` with `k` shrunk preserved variables excludes `2^k` solutions.
    pub num_excluded_solutions: BigInt,
    /// Whether tracing is currently active for the rule being processed.
    ///
    /// This is recomputed for each rule (see [`Context::should_trace`]) so that
    /// the `--trace-lines` and `--trace-ids` options can enable tracing only for
    /// selected lines or constraint IDs.
    pub trace_enabled: bool,
    /// Mapping from [`String`] of a running timer to the [`Instant`] they were started.
    pub running_timers: AHashMap<String, Instant>,
    /// Mapping from [`String`] of a timer to its total elapsed duration.
    pub timer_totals: AHashMap<String, Duration>,
}

impl Context {
    /// Construct a new [`Context`] from the checker `config` and the mapping of labels to IDs in `var_names`.
    pub fn new(
        config: impl Into<CheckerConfig>,
        var_names: VarNameManager,
        preserved_variables: Option<AHashSet<VarIdx>>,
    ) -> Context {
        let config: CheckerConfig = config.into();
        let strong_solution_guarantees = !config.unchecked_deletion;
        Context {
            config,
            strong_solution_guarantees,
            var_names,
            preserved_variables,
            propagation_engine: new_veripb_propagation_engine(),
            rup_streak: REQUIRED_RUP_STREAK - Saturating(1),
            ..Default::default()
        }
    }

    /// Check if the current `line` or constraint ID `next_id` should be traced.
    pub fn should_trace(&self, line: usize, next_id: isize) -> bool {
        should_trace(
            self.config.trace,
            self.config.trace_lines.as_ref(),
            self.config.trace_ids.as_ref(),
            line,
            next_id,
        )
    }

    /// When the propagation trail of the rule being checked should be printed, where
    /// `on_failure` lets a caller override trace_failed.
    pub fn get_trace_trail_setting(&self, on_failure: bool) -> TraceTrail {
        let unfiltered = self.config.trace_lines.is_none() && self.config.trace_ids.is_none();
        TraceTrail::from_config(
            on_failure && self.config.trace_failed,
            self.config.trace_trail && (unfiltered || self.trace_enabled),
        )
    }

    /// Update the objective to a new objective.
    ///
    /// The `update_type` specifies how the objective should be updated and `objective_update` contains the data of the objective updaet.
    pub fn update_objective(
        &mut self,
        mut objective_update: PBObjective,
        update_type: ObjectiveUpdateType,
    ) {
        match update_type {
            ObjectiveUpdateType::New => self.objective = Some(objective_update),
            ObjectiveUpdateType::Diff => {
                // Change the objective by the diff.
                let objective = self.objective.as_mut().unwrap();
                objective.constant += objective_update.constant;
                while let Some((var, term)) = objective_update.terms.pop_first() {
                    if let Some(existing_term) = objective.terms.get_mut(&var) {
                        objective.constant += existing_term.add_with(term);
                        if existing_term.coeff.is_zero() {
                            objective.terms.remove(&var);
                        }
                    } else {
                        objective.terms.insert(var, term);
                    }
                }
            }
        }

        // Trace objective change.
        if self.trace_enabled {
            println!(
                "  {} updated to: {}",
                "Objective".bright_green(),
                self.objective
                    .as_ref()
                    .unwrap()
                    .to_pretty_string(&self.var_names)
                    .green()
            );
        }
    }

    /// Pretty print the current preserved set.
    #[inline]
    pub fn print_preserved_set(&self) {
        println!(
            "{}: {}",
            "Preserved set".magenta(),
            self.preserved_variables
                .as_ref()
                .unwrap()
                .iter()
                .map(|v| v.to_pretty_string(&self.var_names))
                .join(" ")
                .blue()
        );
    }

    /// Check if the witness satisfies the following conditions:
    /// - If a preserved set is loaded, then the domain of the witness can not contain any preserved variables.
    #[inline]
    pub fn check_witness(&mut self, witness: &Substitution) -> Result<(), CheckingError> {
        if witness.is_empty() {
            return Ok(());
        }
        if let Some(preserved_vars) = &self.preserved_variables {
            for var in witness.syntactic_support() {
                if preserved_vars.contains(var) {
                    return Err(CheckingError::WitnessContainsPreservedVar(
                        var.to_pretty_string(&self.var_names).bold(),
                    ));
                }
            }
        } else {
            self.witnesses_used = true;
        }
        Ok(())
    }

    /// Print total time of custom timers in a formatted form.
    pub fn print_custom_timers(&self) {
        // Print times sorted by name of timer.
        for (name, total_time) in self.timer_totals.iter().sorted_unstable_by_key(|t| t.0) {
            println!(
                "c statistic: custom timer '{name}': {} s",
                total_time.as_secs_f64()
            );
        }

        // Report warnings for unstopped timers.
        for name in self.running_timers.keys() {
            warn!("The timer with name '{name}' is still running!");
        }
    }
}

/// Determine whether the rule on (0-based) proof file line `line` that will
/// derive constraints starting at ID `next_id` should be traced.
pub fn should_trace(
    trace_all: bool,
    trace_lines: Option<&RangeSet>,
    trace_ids: Option<&RangeSet>,
    line: usize,
    next_id: isize,
) -> bool {
    if trace_all {
        return true;
    }
    if let Some(lines) = trace_lines {
        // Proof file lines are presented to the user as 1-based.
        if lines.contains(line + 1) {
            return true;
        }
    }
    if next_id >= 0
        && let Some(ids) = trace_ids
        && ids.contains(next_id as usize)
    {
        return true;
    }
    false
}
