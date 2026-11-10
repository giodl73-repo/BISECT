use std::rc::Rc;

use veripb_formula::prelude::*;
use veripb_rules::RedundanceBasedStrengtheningRule;

use crate::{prelude::*, proofgoal::ProofTechnique};

/// Elaborate a top-level dispatch of the red check due to the constraint being RUP or
/// existing already in the database.
pub fn maybe_elaborate_red_shortcut(
    shortcut: &ProofTechnique,
    constraint: &Rc<DBConstraint>,
    context: &mut Context,
) {
    let Some(elaborator) = &mut context.elaborator else {
        return;
    };

    match shortcut {
        ProofTechnique::IsRUP => {
            elaborator.write_rup_step(
                &constraint.constraint,
                &context.propagation_engine.rup_hints,
                &context.var_names,
            );
        }
        ProofTechnique::DBLookup(_, found_constraint) => {
            elaborator.write_pol_single_id(
                found_constraint
                    .get_out_id(found_constraint.get_some_id())
                    .unwrap(),
            );
        }
        other => unreachable!(
            "only the RUP and database-lookup shortcuts are attempted here, got {other}"
        ),
    }
}

impl Checkable for RedundanceBasedStrengtheningRule {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        context.check_witness(&self.witness)?;

        let subcontexts = context.subcontexts.as_slice().last_chunk::<2usize>();
        if let Some(
            [
                Subcontext::Order(order_context),
                Subcontext::Specification(_),
            ],
        ) = subcontexts
        {
            let contains_variable_out_of_order = match &self.constraint.constraint {
                PBConstraintEnum::Clause(clause) => clause.get_lits().any(|lit| {
                    !order_context
                        .constructed_order
                        .is_order_variable(lit.get_var())
                }),
                PBConstraintEnum::Cardinality(cardinality) => cardinality.get_lits().any(|lit| {
                    !order_context
                        .constructed_order
                        .is_order_variable(lit.get_var())
                }),
                PBConstraintEnum::GeneralPBI64(constraint) => constraint.get_lits().any(|lit| {
                    !order_context
                        .constructed_order
                        .is_order_variable(lit.get_var())
                }),
                PBConstraintEnum::GeneralPBI128(constraint) => constraint.get_lits().any(|lit| {
                    !order_context
                        .constructed_order
                        .is_order_variable(lit.get_var())
                }),
                PBConstraintEnum::GeneralPBBigInt(constraint) => constraint.get_lits().any(|lit| {
                    !order_context
                        .constructed_order
                        .is_order_variable(lit.get_var())
                }),
            };
            if contains_variable_out_of_order {
                Err(CheckingError::UndeclaredVariablesInOrder)?
            } else if self
                .witness
                .syntactic_support()
                .any(|var| !order_context.constructed_order.is_auxiliary_variable(*var))
            {
                Err(CheckingError::SpecWitnessMapsNonAuxVariable)?
            }
        }

        // Maybe the strengthening statements in the proof are actually just RUP constraints.
        // We only do these checks if we had enough success autoproving the subproof at top-level without using the witness.
        let shortcut = try_shortcuts(
            &self.constraint,
            context,
            database,
            self.has_subproof,
            &self.witness,
            &[try_rup, try_db_lookup],
        )?;

        if let Some(shortcut) = shortcut {
            maybe_elaborate_red_shortcut(&shortcut, &self.constraint, context);
            return Ok(vec![self.constraint.clone()]);
        }

        // Initialise the subproof context
        let mut subproof_context = SubproofContext::new(database.len());
        let mut add_to_core = false;

        let add_derived_goals = !context.is_strengthening_to_core;
        subproof_context.add_all_red_strengthening_proof_goals(
            &self.constraint,
            &self.witness,
            database,
            context,
            add_derived_goals,
            false,
        )?;

        // We need to elaborate the red rule line here, as autoproving might generate extra lines.
        // In version 2, redundance-based strengthening with an empty witness is treated as proof by
        // contradiction.
        if context.major_version <= Some(2) && self.witness.is_empty() {
            subproof_context.proof_by_contradiction_subproof = true;
            if let Some(elaborator) = context.elaborator.as_mut() {
                elaborator.write_pbc_header(&self.constraint, &context.var_names);
            }
        } else {
            add_to_core = context.is_strengthening_to_core;
            if let Some(elaborator) = context.elaborator.as_mut() {
                elaborator.write_red_header(&self.constraint, &self.witness, &context.var_names);
            }
        }

        let negated = Rc::new(self.constraint.negate());

        // Either wait for the provided subproof or else finalize the proof obligations now.
        if self.has_subproof {
            subproof_context.additional_hint = Some(database.len());
            subproof_context.to_add.push(self.constraint.clone());
            context.inside_strengthening_subproof = true;
            subproof_context.witness = std::mem::take(&mut self.witness);
            subproof_context.add_to_core = add_to_core;
            context
                .subcontexts
                .push(Subcontext::Subproof(subproof_context));
            Ok(vec![negated])
        } else {
            if let Some(elaborator) = context.elaborator.as_mut() {
                negated.set_out_id(0, elaborator.inc_id());
            }

            subproof_context.finalize(context, database, &[negated])?;
            Ok(vec![self.constraint.clone()])
        }
    }
}
