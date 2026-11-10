use std::rc::Rc;

use veripb_formula::prelude::*;
use veripb_rules::DominanceBasedStrengtheningRule;

use crate::prelude::*;

use super::ScopeId;

impl Checkable for DominanceBasedStrengtheningRule {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        context.check_witness(&self.witness)?;

        // We need to elaborate the dom rule line here, as autoproving might generate extra lines.
        if let Some(elaborator) = context.elaborator.as_mut() {
            elaborator.write_dom_header(&self.constraint, &self.witness, &context.var_names);
        }

        let mut subproof_context = SubproofContext::new(database.len());
        let mut subproof_context_strict = SubproofContext::new(database.len());

        if context.objective.is_none() && context.active_order.is_none() {
            return Err(CheckingError::DominanceNoObjectiveOrOrder);
        }

        // Proofgoals from order.
        if let Some(active_order) = &mut context.active_order {
            let mut order_proofgoals = active_order.get_proofgoals_strict(&self.witness);
            if active_order.order.specification.is_empty() {
                for proofgoal in order_proofgoals {
                    subproof_context.add_internal_proofgoal(proofgoal);
                }
                if context.trace_enabled {
                    println!("  ** proofgoals from order **");
                    for id in 1..subproof_context.internal_proofgoal_len() {
                        subproof_context.trace_internal_proofgoal(id, &context.var_names);
                    }
                }
            } else {
                if !self.has_subproof {
                    subproof_context_strict.resize_internal(order_proofgoals.len());
                    subproof_context_strict.add_internal_proofgoal(order_proofgoals.pop().unwrap());
                }
                for proofgoal in order_proofgoals {
                    subproof_context.add_internal_proofgoal(proofgoal);
                }
            }
        }

        // Proofgoal from the objective.
        if let Some(objective) = &context.objective {
            if context.active_order.is_none() {
                subproof_context
                    .add_single_proofgoal(objective.get_proofgoal_strict(&self.witness), None);
            } else {
                subproof_context.add_single_proofgoal(
                    objective.get_proofgoal(&self.witness),
                    Some(ScopeId::LessEqual),
                );
            }
            if context.trace_enabled {
                println!("  ** proofgoal from objective **");
                subproof_context.trace_internal_proofgoal_back(&context.var_names);
            }
        }

        // Get database proofgoals.
        subproof_context.add_database_proofgoals(&self.witness, database, false, false, context)?;

        let negated = Rc::new(self.constraint.negate());

        if self.has_subproof {
            subproof_context.additional_hint = Some(database.len());
            context.inside_strengthening_subproof = true;

            subproof_context.to_add.push(self.constraint.clone());
            subproof_context.witness = std::mem::take(&mut self.witness);
            subproof_context.add_to_core = context.is_strengthening_to_core;
            context
                .subcontexts
                .push(Subcontext::Subproof(subproof_context));

            Ok(vec![negated])
        } else {
            // No subproof, so we have to autoprove the proofgoals.
            if let Some(elaborator) = context.elaborator.as_mut() {
                negated.set_out_id(0, elaborator.inc_id());
            }
            if context.active_order.is_some()
                && !context
                    .active_order
                    .as_ref()
                    .unwrap()
                    .order
                    .specification
                    .is_empty()
            {
                subproof_context.finalize(context, database, std::slice::from_ref(&negated))?;

                subproof_context_strict.finalize(context, database, &[negated])?;
            } else {
                subproof_context.finalize(context, database, &[negated])?;
            }

            Ok(vec![self.constraint.clone()])
        }
    }

    #[inline]
    fn add_constraints_to_core(&self, context: &Context) -> bool {
        context.is_strengthening_to_core
    }
}
