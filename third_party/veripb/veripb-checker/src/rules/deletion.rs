use std::rc::Rc;

use colored::Colorize;
use veripb_formula::prelude::*;
use veripb_propagator::propagation_engine::PropagationEngine;
use veripb_rules::{
    Deletion, DeletionOption, DeletionOrigin,
    elaboration::{ConstraintLookup, Elaboratable, ElaborationError, Elaborator},
};

use super::Checkable;

use crate::{
    deletion_sequence::{DeletionSequence, DeletionSequenceEnum},
    prelude::*,
    proofgoal::ProofTechnique,
};

#[derive(Debug)]
pub struct DeletionChecker {
    inner: Deletion,
    core_ids: Vec<usize>,
    derived_ids: Vec<usize>,
}

impl DeletionChecker {
    pub fn new(inner: Deletion) -> Self {
        Self {
            inner,
            core_ids: Vec::new(),
            derived_ids: Vec::new(),
        }
    }

    /// Get the actual deletions due to this deletion command.
    ///
    /// This is required, as we allow deletion by specification and deletion by ID in the same proof. Hence, we follow the following convention for deletions.
    ///
    /// - If a constraint is deleted by specification and there are more valid constraint IDs for this constraints than the number of times this constraint has been deleted by specification, then all constraint IDs stay valid. Otherwise, all constraint IDs become invalid and the constraint is deleted.
    /// - If a constraint is deleted by ID, then this ID is no longer valid for this constraint. If this causes that the number of valid IDs is at least the number deletions by specification, then all constraint IDs become invalid and the constraint is deleted.
    ///
    /// This function returns `true`, if at least one constraint ID that is actually deleted is in the core set.
    #[inline]
    fn get_actual_deletions(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<(), CheckingError> {
        (self.core_ids, self.derived_ids) = resolve_actual_deletions(
            &self.inner.identifier,
            &mut context.level_ids,
            &mut context.propagation_engine,
            database,
        )?;
        Ok(())
    }

    /// Check if deletion from core and deletion from derived set only deletes constraints from their respective set.
    #[inline]
    fn check_deletion_origin(&self) -> Result<(), CheckingError> {
        validate_deletion_origin(&self.inner.origin, &self.core_ids, &self.derived_ids)
    }

    /// Checked deletion check. We need to construct the proof goals and use heuristics similar to redundance-based strengthening.
    #[inline]
    fn check_checked_deletion(
        &mut self,
        constraint_id: usize,
        database: &mut Database,
        context: &mut Context,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        context.check_witness(&self.inner.witness)?;

        let deleted_constraint = database.get_entry_usize(constraint_id)?.clone();
        let out_id = deleted_constraint.get_out_id(constraint_id);

        // Remove constraint from database and from propagator.
        database.delete_constraint(context, constraint_id)?;

        // Maybe the deletion is actually just implied by RUP. We only do this check if we had enough success autoproving the subproof at top-level without using the witness.
        let shortcut = try_shortcuts(
            &deleted_constraint,
            context,
            database,
            self.inner.has_subproof,
            &self.inner.witness,
            &[try_db_lookup, try_rup],
        )?;

        if let Some(shortcut) = shortcut {
            maybe_elaborate_deletion_shortcut(&shortcut, out_id, context);
            return Ok(vec![]);
        }

        let mut subproof_context = SubproofContext::new(database.len());

        // We need to elaborate the delc rule line here, as autoproving might generate extra lines.
        if let Some(elaborator) = &mut context.elaborator {
            elaborator.write_delc_subproof_header(
                out_id.unwrap(),
                Some(&self.inner.witness),
                Some(&context.var_names),
            );
        }

        if context.is_strengthening_to_core {
            subproof_context.proof_by_contradiction_subproof = true;
        }
        if !context.is_strengthening_to_core || context.major_version == Some(2) {
            subproof_context.add_all_red_strengthening_proof_goals(
                &deleted_constraint,
                &self.inner.witness,
                database,
                context,
                false,
                true,
            )?;
        }

        let negated = Rc::new(deleted_constraint.negate());

        if self.inner.has_subproof {
            subproof_context.additional_hint = Some(database.len());
            context.inside_strengthening_subproof = true;
            subproof_context.witness = std::mem::take(&mut self.inner.witness);
            context
                .subcontexts
                .push(Subcontext::Subproof(subproof_context));
            Ok(vec![negated])
        } else {
            if let Some(elaborator) = context.elaborator.as_mut() {
                negated.set_out_id(0, elaborator.inc_id());
            }

            subproof_context
                .finalize(context, database, &[negated])
                .map_err(|e| match e {
                    CheckingError::AutoprovingFailed(goal_id) => {
                        CheckingError::CheckedDeletionAutoprovingFailed(goal_id, out_id)
                    }
                    e => e,
                })?;
            Ok(vec![])
        }
    }
}

/// Resolve a deletion command to the concrete (core, derived) constraint ids it deletes.
///
/// We allow deletion by specification and deletion by ID in the same proof, following this
/// convention:
/// - If a constraint is deleted by specification and there are more valid constraint IDs than the
///   number of times it has been deleted by specification, all IDs stay valid; otherwise all IDs
///   become invalid and the constraint is deleted.
/// - If a constraint is deleted by ID, that ID is no longer valid; if this brings the valid-ID
///   count to at most the deletions-by-specification count, all IDs become invalid.
pub fn resolve_actual_deletions(
    identifier: &DeletionOption,
    level_ids: &mut [Vec<usize>],
    propagation_engine: &mut PropagationEngine,
    database: &mut Database,
) -> Result<(Vec<usize>, Vec<usize>), CheckingError> {
    let mut core_ids = Vec::new();
    let mut derived_ids = Vec::new();
    match identifier {
        DeletionOption::Id(constraint_ids) => {
            for &id in constraint_ids {
                let index = database.normalize_id(id) as usize;
                let (core, derived) = database.get_entry_usize(index)?.get_del_by_id(index);
                core_ids.extend(core);
                derived_ids.extend(derived);
            }
        }
        DeletionOption::Range((start, end)) => {
            let start_index = database.normalize_id(*start) as usize;
            let end_index = database.normalize_id(*end) as usize;
            for index_result in database.get_undeleted(start_index..end_index) {
                let index = index_result?;
                let (core, derived) = database.get_entry_usize(index)?.get_del_by_id(index);
                core_ids.extend(core);
                derived_ids.extend(derived);
            }
        }
        DeletionOption::Spec(constraint) => {
            database.update_unique_index(propagation_engine)?;
            match database.lookup(constraint) {
                Some(db_constraint) => {
                    let (core, derived) = db_constraint.get_del_by_spec();
                    core_ids.extend(core);
                    derived_ids.extend(derived);
                }
                None => return Err(CheckingError::not_found(constraint, false)),
            }
        }
        DeletionOption::Wipe(level) => {
            let number_levels = level_ids.len();
            for ids in &mut level_ids[*level..number_levels] {
                for index in ids.drain(..) {
                    if let Some(constraint) = database.get_entry_optionally_deleted_usize(index)? {
                        let (core, derived) = constraint.get_del_by_id(index);
                        core_ids.extend(core);
                        derived_ids.extend(derived);
                    }
                }
            }
        }
    }
    // Remove duplicates from the actually deleted IDs.
    core_ids.sort_unstable();
    core_ids.dedup();
    derived_ids.sort_unstable();
    derived_ids.dedup();
    Ok((core_ids, derived_ids))
}

/// Enforce a deletion command's origin: a `Core` command must delete no derived ids, and a
/// `Derived` command must delete no core ids.
pub fn validate_deletion_origin(
    origin: &DeletionOrigin,
    core_ids: &[usize],
    derived_ids: &[usize],
) -> Result<(), CheckingError> {
    match origin {
        DeletionOrigin::Core => {
            if let Some(&id) = derived_ids.first() {
                return Err(CheckingError::DeletionFromCoreDeletesDerived(id));
            }
        }
        DeletionOrigin::Derived => {
            if let Some(&id) = core_ids.first() {
                return Err(CheckingError::DeletionFromDerivedDeletesCore(id));
            }
        }
        DeletionOrigin::Unknown => {}
    }
    Ok(())
}

/// Ensure that the deletion step has the right shape for checked deletion if necessary.
pub fn validate_deletion_preconditions(
    has_subproof: bool,
    witness: &Substitution,
    core_ids: &[usize],
    derived_ids: &[usize],
    context: &Context,
) -> Result<(), CheckingError> {
    // Check preconditions:
    // If there is a subproof, then there should be exactly one core constraint.
    let must_delete_one_core = has_subproof || !witness.is_empty();
    let deleting_one_core = core_ids.len() == 1 && derived_ids.is_empty();

    if must_delete_one_core && !deleting_one_core {
        return Err(CheckingError::DeletionSubproofNotOneCoreConstraint);
    }

    // If strengthening to core is enabled, then checked deletion cannot use a witness.
    if context.is_strengthening_to_core && !witness.is_empty() {
        return Err(CheckingError::DeletionWithWitnessWhileStrengtheningToCore);
    }

    Ok(())
}

/// Determine when checked deletion must be used in the given context.
pub fn should_use_checked_deletion(has_subproof: bool, context: &Context) -> bool {
    context.strong_solution_guarantees
        || has_subproof
        || context.active_order.is_some()
        || context.is_strengthening_to_core
}

/// Elaborate a top-level dispatch of the deletion check due it it being RUP or
/// existing elsewhere in the database.
pub fn maybe_elaborate_deletion_shortcut(
    shortcut: &ProofTechnique,
    out_id: Option<usize>,
    context: &mut Context,
) {
    let Some(elaborator) = &mut context.elaborator else {
        return;
    };
    match shortcut {
        ProofTechnique::IsRUP => {
            elaborator
                .write_rup_checked_deletion(out_id.unwrap(), &context.propagation_engine.rup_hints);
        }
        ProofTechnique::DBLookup(_, found_constraint) => {
            elaborator.write_existing_constraint_checked_deletion(
                out_id.unwrap(),
                found_constraint.get_some_id(),
                found_constraint,
            );
        }
        other => unreachable!(
            "only the RUP and database-lookup shortcuts are attempted here, got {other}"
        ),
    }
}

/// Unchecked deletion can be used if all of the following conditions are met:
/// (0. No explicit subproof is specified, but this might only be a temporary condition.)
/// 1. If an order is loaded or strengthening-to-core is enabled, then check if all constraints are in the core set of constraints.
#[inline]
pub fn check_unchecked_deletion(
    has_subproof: bool,
    database: &mut Database,
    context: &mut Context,
) -> Result<(), CheckingError> {
    if has_subproof {
        return Err(CheckingError::UncheckedWithSubproof);
    }

    if (context.active_order.is_some() || context.is_strengthening_to_core)
        && !database.all_constraints_in_core()
    {
        return Err(CheckingError::UncheckedDeletionDangerous);
    }

    Ok(())
}

impl Checkable for DeletionChecker {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        // Get what constraints and IDs are actually deleted by this rule.
        self.get_actual_deletions(context, database)?;
        // Get which deleted IDs are from the core and which are from the derived set.
        self.check_deletion_origin()?;

        // Check that the checked deletion conditions are legal
        validate_deletion_preconditions(
            self.inner.has_subproof,
            &self.inner.witness,
            &self.core_ids,
            &self.derived_ids,
            context,
        )?;

        // We may not need to use checked deletion if no core ids or if checked deletion conditions
        // are not active.
        if self.core_ids.is_empty()
            || !should_use_checked_deletion(self.inner.has_subproof, context)
        {
            return Ok(vec![]);
        }

        // Proceed with checked deletion
        context.only_core = true;
        while let Some(id) = self.core_ids.pop() {
            if context.trace_enabled {
                println!("  Checked deletion of ID: {}", id.to_string().yellow());
            }

            if self.inner.has_subproof {
                return self.check_checked_deletion(id, database, context);
            }

            if let Some(elaborator) = &mut context.elaborator {
                elaborator.enable_buffered_proof();
            }

            match self.check_checked_deletion(id, database, context) {
                Ok(_) => {
                    if let Some(elaborator) = &mut context.elaborator {
                        elaborator.write_buffered_proof();
                    }
                }
                Err(CheckingError::CheckedDeletionAutoprovingFailed(goal_id, out_id)) => {
                    if context.config.force_checked_deletion {
                        return Err(CheckingError::ForceCheckedDeletionFailed(goal_id));
                    }

                    if context.strong_solution_guarantees {
                        log::warn!(
                            "Switching from stronger to weaker guarantee using unchecked deletion. This means that any solution after this deletion is not necessarily a solution for the original problem."
                        );
                    }

                    if let Some(elaborator) = &mut context.elaborator {
                        elaborator.forget_buffered_proof();
                        elaborator.write_unchecked_deletion(out_id.unwrap());
                    }

                    context.strong_solution_guarantees = false;
                    check_unchecked_deletion(self.inner.has_subproof, database, context)?;
                    break;
                }
                e => return e,
            }
        }

        context.only_core = false;
        Ok(vec![])
    }

    fn get_deleted(
        &self,
        _database: &Database,
    ) -> Result<Option<DeletionSequenceEnum<'_>>, CheckingError> {
        Ok(Some(DeletionSequenceEnum::Deletion(self)))
    }
}

impl From<Deletion> for DeletionChecker {
    fn from(inner: Deletion) -> Self {
        Self::new(inner)
    }
}

impl DeletionSequence for DeletionChecker {
    #[inline]
    fn get_deleted_ids(&self) -> impl Iterator<Item = &usize> {
        self.core_ids.iter().chain(self.derived_ids.iter())
    }
}

impl Elaboratable for DeletionChecker {
    fn elaborate<C: ConstraintLookup, V: VarNameLookup>(
        &self,
        elaborator: &mut Elaborator,
        database: &C,
        _var_names: &V,
    ) -> Result<(), ElaborationError> {
        if !self.derived_ids.is_empty() {
            elaborator.write_derived_deletions(&self.derived_ids, database);
        }

        if !self.core_ids.is_empty() {
            elaborator.write_core_deletions_no_subproof(&self.core_ids, database)
        }

        Ok(())
    }
}
