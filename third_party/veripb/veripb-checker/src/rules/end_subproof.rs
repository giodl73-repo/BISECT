use std::{ops::Range, rc::Rc};

use veripb_formula::prelude::*;
use veripb_propagator::propagation_engine::PropagationEngine;
use veripb_rules::{
    EndSubproof,
    elaboration::{ConstraintLookup, Elaboratable, ElaborationError, Elaborator},
};

use crate::{
    deletion_sequence::DeletionSequenceEnum, prelude::*, proofgoal_context::ProofgoalContext,
};

#[derive(Debug)]
pub struct EndSubproofChecker {
    inner: EndSubproof,
    is_elaborated: bool,
    subproof_id_range: Range<usize>,
    replacement_database: Option<Box<Database>>,
    replacement_prop_engine: Option<Box<PropagationEngine>>,
    qed_subcontext: bool,
    add_to_core: bool,
}

impl EndSubproofChecker {
    pub fn new(inner: EndSubproof) -> Self {
        Self {
            inner,
            is_elaborated: false,
            subproof_id_range: 0..0,
            replacement_database: None,
            replacement_prop_engine: None,
            qed_subcontext: false,
            add_to_core: false,
        }
    }

    /// End a subproof subcontext.
    #[inline]
    fn end_subproof(
        &mut self,
        context: &mut Context,
        database: &mut Database,
        subproof_context: SubproofContext,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        self.subproof_id_range = subproof_context.subproof_start_id..database.len();
        let result = if let Some(hint) = &mut self.inner.optional_hint {
            *hint = database.normalize_id(*hint);
            if !database.get_entry_usize(*hint as usize)?.is_contradicting() {
                return Err(CheckingError::HintNoContradiction(*hint));
            }
            if let Some(active_order) = &mut context.active_order {
                active_order.substitution_left_set = false;
                active_order.substitution_right_set = false;
            }
            subproof_context.finalize_unchecked(context)
        } else {
            // `finalize` will already elaborate the `qed` and write the final `qed` for the subproof.
            self.is_elaborated = true;
            // Need to check that all remaining proofgoals can be proven.
            subproof_context.finalize(context, database, &[])
        };

        // If the subcontext below is not a `Subproof` or `Proofgoal`, then we definitely end the only core subproof.
        match context.subcontexts.last() {
            Some(Subcontext::Proofgoal(_)) | Some(Subcontext::Subproof(_)) => (),
            _ => context.only_core = false,
        }

        result
    }

    /// End a proofgoal subcontext.
    #[inline]
    fn end_proofgoal(
        &mut self,
        context: &mut Context,
        database: &mut Database,
        proofgoal_context: ProofgoalContext,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        self.subproof_id_range = proofgoal_context.subproof_start_id..database.len();
        if let Some(hint) = &mut self.inner.optional_hint {
            *hint = database.normalize_id(*hint);
            if !database.get_entry_usize(*hint as usize)?.is_contradicting() {
                return Err(CheckingError::HintNoContradiction(*hint));
            }
            // Fix that version 2.0 `red` steps with empty witness are elaborated to `pbc` correctly.
            if let Some(Subcontext::Subproof(subsubcontext)) = context.subcontexts.last_mut()
                && subsubcontext.proof_by_contradiction_subproof
            {
                self.is_elaborated = true;
                subsubcontext.contradiction_proven = true;
                if context.elaborator.is_some() {
                    subsubcontext.contradiction_output_id = database
                        .get_entry_usize(*hint as usize)?
                        .get_out_id(*hint as usize)
                        .expect("Constraint should have output ID.");
                }
            }
            Ok(vec![])
        } else {
            // Need to check that contradiction has been derived.
            if database
                .get_entry_usize(database.len() - 1)?
                .is_contradicting()
            {
                self.inner.optional_hint = Some(database.len() as isize - 1);
            } else if let Some(id) = database.contains_contradiction() {
                self.inner.optional_hint = Some(id as isize);
            } else {
                return Err(CheckingError::ProofgoalEndContradicitionNotObvious);
            }
            // Fix that version 2.0 `red` steps with empty witness are elaborated to `pbc` correctly.
            if let Some(Subcontext::Subproof(subsubcontext)) = context.subcontexts.last_mut()
                && subsubcontext.proof_by_contradiction_subproof
            {
                let hint = self.inner.optional_hint.unwrap() as usize;
                self.is_elaborated = true;
                subsubcontext.contradiction_proven = true;
                if context.elaborator.is_some() {
                    subsubcontext.contradiction_output_id = database
                        .get_entry_usize(hint)?
                        .get_out_id(hint)
                        .expect("Constraint should have output ID.");
                }
            }
            Ok(vec![])
        }
    }
}

impl From<EndSubproof> for EndSubproofChecker {
    fn from(inner: EndSubproof) -> Self {
        Self::new(inner)
    }
}

impl Checkable for EndSubproofChecker {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        if let Some(subcontext) = context.subcontexts.pop() {
            match subcontext {
                Subcontext::Subproof(mut subproof_context) => {
                    if subproof_context.current_scope.is_some() {
                        subproof_context.current_scope = None;
                        self.subproof_id_range = subproof_context.scope_start_id..database.len();
                        context
                            .subcontexts
                            .push(Subcontext::Subproof(subproof_context));
                        return Ok(vec![]);
                    }
                    self.qed_subcontext = true;
                    self.add_to_core = subproof_context.add_to_core;
                    self.end_subproof(context, database, subproof_context)
                }
                Subcontext::Proofgoal(proofgoal_context) => {
                    self.qed_subcontext = true;
                    self.end_proofgoal(context, database, proofgoal_context)
                }
                Subcontext::Order(mut order_context) => {
                    if self.inner.optional_hint.is_some() {
                        return Err(CheckingError::EndNoHintAllowed);
                    }
                    if order_context.inside_vars {
                        order_context.inside_vars = false;
                        if !order_context.left_vars_defined {
                            return Err(CheckingError::LeftVarsUndefined);
                        }
                        if !order_context.right_vars_defined {
                            return Err(CheckingError::RightVarsUndefined);
                        }
                        if !order_context.aux_vars_defined {
                            if context.major_version >= Some(3) {
                                if let Some(elaborator) = context.elaborator.as_mut() {
                                    elaborator.write_empty_aux();
                                }
                            } else {
                                return Err(CheckingError::AuxVarsUndefined);
                            }
                        }

                        context.subcontexts.push(Subcontext::Order(order_context));
                        return Ok(vec![]);
                    }
                    if order_context.inside_def {
                        order_context.inside_def = false;
                        context.subcontexts.push(Subcontext::Order(order_context));
                        return Ok(vec![]);
                    }

                    // Check if reflexivity and transitivity have been proven.
                    if !order_context.reflexivity_proven
                        && !order_context.autoprove_reflexivity(&mut context.elaborator)
                    {
                        return Err(CheckingError::ReflexivityProofFailed);
                    }
                    if !order_context.transitivity_proven && !order_context.autoprove_transitivity()
                    {
                        return Err(CheckingError::TransitivityProofFailed);
                    }

                    // Restore database.
                    self.replacement_database = Some(order_context.stored_database);
                    self.replacement_prop_engine = Some(order_context.stored_prop_engine);

                    // Add successfully proven order to context.
                    context
                        .orders
                        .insert(order_context.name, order_context.constructed_order);

                    // Recover old context state.
                    if let Some(elaborator) = context.elaborator.as_mut() {
                        elaborator.current_id = order_context.stored_elaborator_id;
                    }
                    context.active_order = *order_context.stored_active_order;

                    Ok(vec![])
                }
                Subcontext::Reflexivity(reflexivity_context) => {
                    // Check if a proof for reflexivity has been provided.
                    if !reflexivity_context.is_proven {
                        return Err(CheckingError::ReflexivityProofMissing);
                    }
                    let order_context = match context.subcontexts.last_mut().unwrap() {
                        Subcontext::Order(order_context) => order_context,
                        _ => unreachable!(),
                    };
                    order_context.reflexivity_proven = true;
                    Ok(vec![])
                }
                Subcontext::Transitivity(mut transitivity_context) => {
                    if transitivity_context.inside_vars {
                        transitivity_context.inside_vars = false;
                        context
                            .subcontexts
                            .push(Subcontext::Transitivity(transitivity_context));
                        return Ok(vec![]);
                    }
                    // Check if a proof for transitivity has been provided.
                    if !transitivity_context.is_proven {
                        return Err(CheckingError::TransitivityProofMissing);
                    }
                    let order_context = match context.subcontexts.last_mut().unwrap() {
                        Subcontext::Order(order_context) => order_context,
                        _ => unreachable!(),
                    };
                    order_context.transitivity_proven = true;
                    Ok(vec![])
                }
                Subcontext::Specification(spec_context) => {
                    let order_context = match context.subcontexts.last_mut().unwrap() {
                        Subcontext::Order(order_context) => order_context,
                        _ => unreachable!(),
                    };

                    for constraint in database.entries.iter().flatten() {
                        order_context
                            .constructed_order
                            .specification
                            .push(constraint.clone());
                    }

                    // Restore database.
                    self.replacement_database = Some(spec_context.stored_database);
                    self.replacement_prop_engine = Some(spec_context.stored_prop_engine);
                    if let Some(elaborator) = context.elaborator.as_mut() {
                        elaborator.current_id = spec_context.stored_elaborator_id;
                    }

                    Ok(vec![])
                }
            }
        } else {
            Err(CheckingError::NoOpenSubcontext)
        }
    }

    #[inline]
    fn get_deleted<'a>(
        &'a self,
        database: &Database,
    ) -> Result<Option<DeletionSequenceEnum<'a>>, CheckingError> {
        let mut deleted_ids = Vec::new();
        database
            .get_undeleted(self.subproof_id_range.clone())
            .try_for_each(|result_index| {
                let index = result_index?;
                let constraint = database.get_entry_usize(index).unwrap();
                let mut constraint_ids = constraint.get_del_by_id(index);
                deleted_ids.append(&mut constraint_ids.0);
                deleted_ids.append(&mut constraint_ids.1);
                Result::<_, CheckingError>::Ok(())
            })?;
        // Remove duplicates from deleted constraint IDs.
        deleted_ids.sort_unstable();
        deleted_ids.dedup();
        Ok(Some(DeletionSequenceEnum::Vec(deleted_ids)))
    }

    #[inline]
    fn add_constraints_to_core(&self, _context: &Context) -> bool {
        self.add_to_core
    }

    #[inline]
    fn swaps_database(&self) -> bool {
        self.replacement_database.is_some()
    }

    #[inline]
    fn get_new_database(&mut self) -> (Database, PropagationEngine) {
        (
            *std::mem::take(&mut self.replacement_database).unwrap(),
            *std::mem::take(&mut self.replacement_prop_engine).unwrap(),
        )
    }

    #[inline]
    fn handle_old_database(
        &mut self,
        _context: &mut Context,
        _database: Database,
        _prop_engine: PropagationEngine,
    ) {
    }
}

impl Elaboratable for EndSubproofChecker {
    fn elaborate<C: ConstraintLookup, V: VarNameLookup>(
        &self,
        elaborator: &mut Elaborator,
        database: &C,
        _var_lookup: &V,
    ) -> Result<(), ElaborationError> {
        if self.is_elaborated {
            return Ok(());
        }
        elaborator.write_subcontext_end(self.qed_subcontext, self.inner.optional_hint, database);
        Ok(())
    }
}
