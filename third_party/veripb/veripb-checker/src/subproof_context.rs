/// [`Subcontext`] for a top-level subproof.
use std::{collections::BTreeMap, rc::Rc};

use colored::Colorize;
use malachite_bigint::BigInt;
use num_traits::{Signed, Zero};
use veripb_formula::prelude::*;
use veripb_rules::{ObjectiveUpdateType, ProofgoalID};

use crate::{context::AUTOPROVING, prelude::*, rules::ScopeId};

/// The proofgoal elaboration threshold ratio controls if proofgoals that are equal to a constraint in the database are elaborated or not. The ratio we compare to is the number of proofgoals divided by the number of constraint in the formula.
///
/// # Why is this needed?
/// CakePB supports autoproving for proofgoals where the constraint is already in the database, but the initialization of the data structure required for this is slow and only pays off if there are many proof goals. Through experimentation it was determined that `0.05` is a good ratio for this.
const PROOFGOAL_ELABORATION_THRESHOLD: f64 = 0.05;

/// Top-level subproof [`Subcontext`] storing information that is only needed inside that subproof and what should be changed after the subproof is completed.
#[derive(Debug, Default)]
pub struct SubproofContext {
    /// Constraints to be added after the subproof has been completed.
    pub to_add: Vec<Rc<DBConstraint>>,
    /// Add the constraints in the `to_add` vector to the core set.
    pub add_to_core: bool,
    /// Proofgoals from the database that should be proven in the subproof.
    pub database_proofgoals: BTreeMap<usize, Proofgoal>,
    /// Internal proofgoals that should be proven in the subproof. Internal proofgoals are identified by an `#`.
    pub internal_proofgoals: Vec<Option<Proofgoal>>,
    /// Objective that should be set after the subproof is completed. This can update
    pub objective_update: Option<PBObjective>,
    /// The type of the objective update.
    pub objective_update_type: ObjectiveUpdateType,
    /// The additional premise is usually the negated constraint.
    pub additional_hint: Option<usize>,
    /// Flag if proof by contradiction subproof.
    pub proof_by_contradiction_subproof: bool,
    /// Start constraint ID of the subproof.
    pub subproof_start_id: usize,
    /// Current scope the subproof is in.
    pub current_scope: Option<ScopeId>,
    /// Starting constraint ID for the scope.
    pub scope_start_id: usize,
    /// Witness used for this subproof.
    pub witness: Substitution,
    /// Contradiction proven.
    pub contradiction_proven: bool,
    /// Contradiction output hint.
    pub contradiction_output_id: usize,
}

impl SubproofContext {
    /// Constructs a new subproof context with internal proofgoals containing dummy 0th element.
    #[inline]
    pub fn new(start_id: usize) -> Self {
        SubproofContext {
            internal_proofgoals: vec![None],
            subproof_start_id: start_id,
            ..Default::default()
        }
    }

    /// Add a single constraint internal proofgoal to the subproof context.
    #[inline]
    pub fn add_single_proofgoal(
        &mut self,
        constraint: Rc<DBConstraint>,
        scope_restriction: Option<ScopeId>,
    ) {
        self.internal_proofgoals
            .push(Some(Proofgoal::mk_single_constraint(
                constraint,
                scope_restriction,
                false,
            )));
    }

    /// Add a proofgoal as an internal proofgoal.
    #[inline]
    pub fn add_internal_proofgoal(&mut self, proofgoal: Proofgoal) {
        self.internal_proofgoals.push(Some(proofgoal));
    }

    /// Add dummy internal proofgoals to increment the IDs.
    pub fn resize_internal(&mut self, new_size: usize) {
        self.internal_proofgoals.resize_with(new_size, || None);
    }

    /// Add a multi constraint internal proofgoal to the subproof context.
    #[inline]
    pub fn add_multi_proofgoal(
        &mut self,
        constraints: Vec<Rc<DBConstraint>>,
        scope_restriction: Option<ScopeId>,
    ) {
        self.internal_proofgoals
            .push(Some(Proofgoal::mk_multi_constraint(
                constraints,
                scope_restriction,
            )));
    }

    /// Add a single constraint proofgoal from the database to the subproof context.
    ///
    /// The constraint is added in such a way that it can be easily referenced by the constraint ID of any duplicate of the constraint. However, when the proofgoal is proven, this fact should directly propagate to the other duplicates.
    #[inline]
    pub fn add_database_proofgoal(
        &mut self,
        constraint: Rc<DBConstraint>,
        add_derived_goals: bool,
        scope_restriction: Option<ScopeId>,
    ) {
        for &id in constraint.header.borrow().core_ids.iter() {
            self.database_proofgoals.insert(
                id,
                Proofgoal::mk_single_constraint(constraint.clone(), scope_restriction, false),
            );
        }
        if add_derived_goals {
            for &id in constraint.header.borrow().derived_ids.iter() {
                self.database_proofgoals.insert(
                    id,
                    Proofgoal::mk_single_constraint(constraint.clone(), scope_restriction, false),
                );
            }
        }
    }

    /// Collect the database proofgoals of a strengthening step under `witness`.
    ///
    /// Shared by redundance- and dominance-based strengthening, which differ only in
    /// `add_derived_goals` and `only_core`.
    pub fn add_database_proofgoals(
        &mut self,
        witness: &Substitution,
        database: &mut Database,
        add_derived_goals: bool,
        only_core: bool,
        context: &mut Context,
    ) -> Result<(), CheckingError> {
        database.update_unique_index(&mut context.propagation_engine)?;
        for constraint in database.get_proofgoals(
            witness,
            add_derived_goals,
            context.elaborator.is_some(),
            only_core,
        ) {
            self.add_database_proofgoal(constraint, add_derived_goals, Some(ScopeId::LessEqual));
        }
        if context.trace_enabled {
            self.trace_database_proofgoals(&context.var_names);
        }
        Ok(())
    }

    /// Collect every proofgoal a redundance-based strengthening step must discharge:
    /// - the added constraint under the witness (`#1`)
    /// - the active order definition constraints (`#1`-`#(|order|+1)`)
    /// - objective goals (`#(|order|+2)`)
    /// - and then the affected database constraints (database ids).
    pub fn add_all_red_strengthening_proof_goals(
        &mut self,
        constraint: &Rc<DBConstraint>,
        witness: &Substitution,
        database: &mut Database,
        context: &mut Context,
        add_derived_goals: bool,
        for_deletion: bool,
    ) -> Result<(), CheckingError> {
        // Start with the constraint to be introduced.
        self.add_single_proofgoal(
            Rc::new(constraint.substitute(witness)),
            Some(ScopeId::LessEqual),
        );

        if context.trace_enabled {
            println!(
                "  ** proofgoal from satisfying {} constraint **",
                if for_deletion { "deleted" } else { "added" }
            );
            self.trace_internal_proofgoal_back(&context.var_names);
        }

        // Proofgoals from order.
        if let Some(active_order) = context.active_order.as_mut() {
            for proofgoal in active_order.get_proofgoals(witness) {
                self.add_internal_proofgoal(proofgoal);
            }
            if context.trace_enabled {
                println!("  ** proofgoals from order **");
                // Start at 2 since the first proofgoal is the constraint to be derived (under the witness).
                for id in 2..self.internal_proofgoal_len() {
                    self.trace_internal_proofgoal(id, &context.var_names);
                }
            }
        }

        // Proofgoal from the objective.
        if let Some(objective) = &context.objective {
            self.add_single_proofgoal(objective.get_proofgoal(witness), Some(ScopeId::LessEqual));
            if context.trace_enabled {
                println!("  ** proofgoal from objective **");
                self.trace_internal_proofgoal_back(&context.var_names);
            }
        }

        // Get database proofgoals.
        self.add_database_proofgoals(
            witness,
            database,
            add_derived_goals,
            context.only_core,
            context,
        )
    }

    /// Collect the proofgoals needed for and objective update rule.
    pub fn add_all_objective_update_proofgoals(
        &mut self,
        update_type: ObjectiveUpdateType,
        objective: &PBObjective,
        context: &Context,
    ) {
        let (new_geq_old_constraint, old_geq_new_constraint) = match update_type {
            ObjectiveUpdateType::New => {
                // Compute f_new >= f_old first and for f_old >= f_new the degree and the coefficients are just negated.
                let old_objective = context.objective.as_ref().unwrap();
                let new_geq_old_degree = old_objective.constant.clone() - &objective.constant;
                let old_geq_new_degree = -new_geq_old_degree.clone();
                let mut new_geq_old_terms = objective.terms.values().cloned().collect::<Vec<_>>();
                new_geq_old_terms.extend(
                    old_objective
                        .terms
                        .values()
                        .map(|term| GeneralPBTerm::new(-term.coeff.clone(), term.lit)),
                );
                let old_geq_new_terms = new_geq_old_terms
                    .iter()
                    .map(|term| GeneralPBTerm::new(-term.coeff.clone(), term.lit))
                    .collect::<Vec<_>>();
                let coeff_sum = new_geq_old_terms
                    .iter()
                    .fold(Zero::zero(), |sum: BigInt, term| sum + &term.coeff.abs());
                let new_geq_old_constraint = constraint_from_terms_and_coeff_sum(
                    new_geq_old_terms,
                    new_geq_old_degree,
                    coeff_sum.clone(),
                );
                let old_geq_new_constraint = constraint_from_terms_and_coeff_sum(
                    old_geq_new_terms,
                    old_geq_new_degree,
                    coeff_sum,
                );

                (
                    Rc::new(new_geq_old_constraint.into()),
                    Rc::new(old_geq_new_constraint.into()),
                )
            }
            ObjectiveUpdateType::Diff => {
                let coeff_abs_sum = objective
                    .terms
                    .values()
                    .fold(Zero::zero(), |sum: BigInt, term| sum + &term.coeff.abs());

                // The proofgoal f_new >= f_old is just `diff_terms >= -diff_constant`.
                let new_geq_old_constraint = constraint_from_terms_and_coeff_sum(
                    objective.terms.values().cloned().collect(),
                    -objective.constant.clone(),
                    coeff_abs_sum.clone(),
                );

                // The proofgoal f_old >= f_new is just `-diff_terms >= diff_constant`.
                let old_geq_new_constraint = constraint_from_terms_and_coeff_sum(
                    objective
                        .terms
                        .values()
                        .map(|term| GeneralPBTerm::new(-term.coeff.clone(), term.lit))
                        .collect(),
                    objective.constant.clone(),
                    coeff_abs_sum,
                );

                (
                    Rc::new(new_geq_old_constraint.into()),
                    Rc::new(old_geq_new_constraint.into()),
                )
            }
        };

        self.add_single_proofgoal(new_geq_old_constraint, None);
        if context.trace_enabled {
            println!("  ** proofgoal from new objective >= old objective **");
            self.trace_internal_proofgoal_back(&context.var_names);
        }

        self.add_single_proofgoal(old_geq_new_constraint, None);
        if context.trace_enabled {
            println!("  ** proofgoal from old objective >= new objective **");
            self.trace_internal_proofgoal_back(&context.var_names);
        }
    }

    /// Get an internal proof goal.
    #[inline]
    pub fn pop_internal_goal(&mut self, index: usize) -> Result<Proofgoal, CheckingError> {
        if let Some(optional_goal) = self.internal_proofgoals.get_mut(index) {
            if let Some(goal) = optional_goal.take() {
                if let Some(scope) = self.current_scope
                    && !goal.is_in_scope(scope)
                {
                    return Err(CheckingError::ProofgoalNotInScope(
                        ProofgoalID::Internal(index),
                        scope,
                    ));
                }
                return Ok(goal);
            }
            return Err(CheckingError::InternalProofgoalAlreadyProven(index));
        }
        Err(CheckingError::InternalProofgoalNotExisting(index))
    }

    /// Get a database proofgoal.
    #[inline]
    pub fn pop_database_goal(
        &mut self,
        index: usize,
        database: &Database,
    ) -> Result<Proofgoal, CheckingError> {
        if let Some(goal) = self.database_proofgoals.remove(&index) {
            if let Some(scope) = self.current_scope
                && !goal.is_in_scope(scope)
            {
                return Err(CheckingError::ProofgoalNotInScope(
                    ProofgoalID::Database(index as isize),
                    scope,
                ));
            }
            Ok(goal)
        } else {
            let constraint = database.get_entry_usize(index)?;
            Ok(Proofgoal::mk_single_constraint(
                Rc::new(constraint.substitute(&self.witness)),
                None,
                false,
            ))
        }
    }

    /// Prints the last added internal proofgoal.
    #[inline]
    pub fn trace_internal_proofgoal_back(&self, var_names: &VarNameManager) {
        self.trace_internal_proofgoal(self.internal_proofgoals.len() - 1, var_names);
    }

    /// Prints the internal proofgoal with the given `id`.
    #[inline]
    pub fn trace_internal_proofgoal(&self, id: usize, var_names: &VarNameManager) {
        if let Some(Some(proofgoal)) = self.internal_proofgoals.get(id) {
            let id_string = String::from("#") + id.to_string().as_str();
            proofgoal.trace(id_string.as_str(), var_names)
        }
    }

    /// Print all the internal proofgoals.
    #[inline]
    pub fn trace_all_internal_proofgoals(&self, var_names: &VarNameManager) {
        for (goal_id, goal) in self.internal_proofgoals.iter().enumerate() {
            let id_string = String::from("#") + goal_id.to_string().as_str();
            if let Some(proofgoal) = goal {
                proofgoal.trace(id_string.as_str(), var_names)
            }
        }
    }

    /// Print the proofgoals from the database according to their constraint ID.
    #[inline]
    pub fn trace_database_proofgoals(&self, var_names: &VarNameManager) {
        if self.database_proofgoals.is_empty() {
            return;
        }
        println!("  ** proofgoal from formula **");
        for (goal_id, proofgoal) in self.database_proofgoals.iter() {
            println!(
                "  proofgoal {}: {}",
                goal_id.to_string().bright_green(),
                proofgoal
                    .unwrapped_conclusion()
                    .to_pretty_string(var_names)
                    .blue()
            );
        }
    }

    /// Finalizes the subproof by checking for unproven proofgoals and autoproving them if possible.
    ///
    /// This function can be used to conclude a top-level subproof (i.e., not a proofgoal subproof).
    /// It tries to autoprove the remaining proofgoals. If all proofgoals are proven, then this function updates the objective and return the constraints to be added after the subproof is successful.
    ///
    /// This function consumes the `SubproofContext`, as it is no longer required and saves reallocating memory for the output constraints that should be added.
    pub fn finalize(
        mut self,
        context: &mut Context,
        database: &mut Database,
        additional_premises: &[Rc<DBConstraint>],
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        // First check if all proofgoals have been proven already.
        if self.has_no_outstanding_obligations() {
            if let Some(elaborator) = context.elaborator.as_mut() {
                elaborator.write_qed();
            }
            return self.finalize_unchecked(context);
        }

        // Check if contradiction has already been proven
        if self.contradiction_proven {
            if let Some(elaborator) = context.elaborator.as_mut() {
                elaborator.write_qed_with_id(self.contradiction_output_id);
            }
            return self.finalize_unchecked(context);
        }

        if context.trace_enabled {
            println!("  autoproving remaining proofgoals:")
        }

        let has_premises = !additional_premises.is_empty();

        // Check if we can derive contradiction in the top-level subproof.
        // If so, all proofgoals can use this contradiction to show the proofgoals.
        if self.maybe_toplevel_rup_check(context, database, additional_premises)? {
            return self.finalize_via_toplevel_rup(context, has_premises);
        }

        // Proof by contradiction must be RUP at top level if no explicit subproof given.
        if self.proof_by_contradiction_subproof {
            return Err(CheckingError::ProofByContradictionNotRUP);
        }

        // Determine the hint to use. If there are additional premises, take the first constraint as
        // hint, as this is the negated constraint for redundance- and dominance-based
        // strengthening.
        let hint = match (additional_premises.first(), self.additional_hint) {
            (Some(premise), _) => Some(premise.clone()),
            (None, Some(p)) => Some(database.get_entry_usize(p)?.clone()),
            (None, None) => None,
        };

        // Add any additional premises to the propagation engine for the autoproving.
        for constraint in additional_premises {
            context.propagation_engine.attach(AUTOPROVING, constraint)?;
        }

        if has_premises {
            context.propagation_engine.enable_front(AUTOPROVING);
        }

        // Attempt autoproving on the remaining goals and then clear the autoproving set
        // (both on success and on failure).
        let result = self.autoprove_remaining_goals(context, database, &hint);
        if has_premises {
            context.propagation_engine.disable(AUTOPROVING);
            context.propagation_engine.clear_set(AUTOPROVING);
        }
        result?;

        // All goals have been successfully proven.
        self.finalize_unchecked(context)
    }

    /// Finalize the subproof without checking for unproven proofgoals.
    ///
    /// This function updates the objective and return the constraints to be added after the subproof is successful.
    #[inline]
    pub fn finalize_unchecked(
        self,
        context: &mut Context,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        // Change the objective if it should be updated.
        if let Some(objective_update) = self.objective_update {
            context.update_objective(objective_update, self.objective_update_type);
        }

        // Add the derived constraints.
        Ok(self.to_add)
    }

    /// The maximum ID for an internal proofgoal.
    #[inline]
    pub fn internal_proofgoal_len(&self) -> usize {
        self.internal_proofgoals.len()
    }

    /// The number of database proofgoals.
    #[inline]
    pub fn database_proofgoals_len(&self) -> usize {
        self.database_proofgoals.len()
    }

    /// Whether all proofgoals have been proven already
    #[inline]
    pub fn has_no_outstanding_obligations(&self) -> bool {
        !self.proof_by_contradiction_subproof
            && self.internal_proofgoals.iter().all(|g| g.is_none())
            && self.database_proofgoals.is_empty()
    }

    /// Apply heuristic determine whether to elaborate proofgoals induced by
    /// database constraints (see PROOFGOAL_ELABORATION_THRESHOLD comment above).
    pub fn should_elaborate_db_proofgoals(&self, database: &Database) -> bool {
        (self.database_proofgoals.len() as f64 / database.len() as f64)
            <= PROOFGOAL_ELABORATION_THRESHOLD
    }

    /// Autoprove and elaborate every remaining internal and database proofgoal, then write the
    /// concluding QED.
    fn autoprove_remaining_goals(
        &mut self,
        context: &mut Context,
        database: &mut Database,
        hint: &Option<Rc<DBConstraint>>,
    ) -> Result<(), CheckingError> {
        // Internal proofgoals.
        for (id, proofgoal) in self.internal_proofgoals.iter_mut().enumerate() {
            let Some(proofgoal) = proofgoal else {
                continue;
            };
            let goal_id = ProofgoalID::Internal(id);
            proofgoal.allocate_premise_output_ids(context);
            let technique = proofgoal.autoprove(context, database, hint, goal_id)?;
            proofgoal.elaborate(&technique, context, database, goal_id, true)?;
            if context.trace_enabled {
                proofgoal.trace_autoproven(format!("{goal_id}").purple(), technique);
            }
        }

        // Database proofgoals.
        let elaborate_db_proofgoals = self.should_elaborate_db_proofgoals(database);
        while let Some((id, mut proofgoal)) = self.database_proofgoals.pop_first() {
            let goal_id = ProofgoalID::Database(id as isize);
            proofgoal.allocate_premise_output_ids(context);
            let technique = proofgoal.autoprove(context, database, hint, goal_id)?;
            proofgoal.elaborate(
                &technique,
                context,
                database,
                goal_id,
                elaborate_db_proofgoals,
            )?;
            if context.trace_enabled {
                proofgoal.trace_autoproven(format!("{goal_id}").bright_green(), technique);
            }
        }

        // Conclude the subproof elaboration.
        if let Some(elaborator) = context.elaborator.as_mut() {
            elaborator.write_qed();
        }
        Ok(())
    }

    // Heuristic to check if we should use the top-level RUP check to detect if all proofgoals follow by one RUP check without using the proofgoals at all.
    // Usually this is the case for redundance-based strengthening steps that could be RUP steps.
    // Such steps occur in proofs produced by SAT solvers directly translated from the RAT rule.
    //
    // Such RAT steps always have a witness of length 1, so we require that the witness has length at most 1.
    // In addition, we require that there is at least one internal proofgoal (which are offset by one) and at least three database proofgoals.
    pub fn use_toplevel_rup_check(&self) -> bool {
        self.internal_proofgoal_len() >= 2
            && self.database_proofgoals.len() >= 3
            && self.witness.len() <= 1
    }

    /// Run a top-level rup check, if permitted. Returns true if the RUP check succeeded and false if
    /// it failed or was not attempted.
    pub fn maybe_toplevel_rup_check(
        &self,
        context: &mut Context,
        database: &mut Database,
        additional_premises: &[Rc<DBConstraint>],
    ) -> Result<bool, CheckingError> {
        // Determine if we will even attempt the top level rup check.
        if !self.proof_by_contradiction_subproof && !self.use_toplevel_rup_check() {
            return Ok(false);
        }

        database.update_propagation_index(&mut context.propagation_engine)?;
        let trace_trail = context.get_trace_trail_setting(self.proof_by_contradiction_subproof);
        Ok(context
            .propagation_engine
            .reverse_unit_propagation_check(
                &context.var_names,
                additional_premises,
                None,
                context.only_core,
                context.elaborator.is_some(),
                trace_trail,
            )?
            .is_conflict())
    }

    /// Finalizes the subproof for the case where a top-level rup check succeeded, updating
    /// the RUP streak for potential future shortcutting.
    pub fn finalize_via_toplevel_rup(
        self,
        context: &mut Context,
        has_premises: bool,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        if context.trace_enabled {
            let msg_sub = if has_premises {
                "premises"
            } else {
                "constraints"
            };
            println!("  * database with subproof {msg_sub} is propagating to conflict",);
        }
        context.rup_streak += 1;
        if let Some(elaborator) = context.elaborator.as_mut() {
            elaborator.write_rup_contradiction_subproof_tail(
                false,
                &context.propagation_engine.rup_hints,
            );
        }
        self.finalize_unchecked(context)
    }
}
