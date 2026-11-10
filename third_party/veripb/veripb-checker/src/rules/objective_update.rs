use std::rc::Rc;

use veripb_formula::prelude::DBConstraint;
use veripb_rules::ObjectiveUpdateRule;

use crate::prelude::*;

impl Checkable for ObjectiveUpdateRule {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        if context.objective.is_none() {
            return Err(CheckingError::ObjectiveUpdateNoObjective);
        }

        // We need to elaborate the obju rule line here, as autoproving might generate extra lines.
        if let Some(elaborator) = context.elaborator.as_mut() {
            elaborator.write_objective_update(
                self.update_type,
                &self.objective,
                &context.var_names,
            );
        }

        // Set up subproof subcontext.
        let mut subproof_context = SubproofContext::new(database.len());
        context.only_core = true;

        subproof_context.add_all_objective_update_proofgoals(
            self.update_type,
            &self.objective,
            context,
        );

        // Start subproof and change to new objective.
        if self.has_subproof {
            subproof_context.objective_update = Some(std::mem::take(&mut self.objective));
            subproof_context.objective_update_type = self.update_type;
            context
                .subcontexts
                .push(Subcontext::Subproof(subproof_context));
        } else {
            subproof_context.finalize(context, database, &[])?;
            context.update_objective(std::mem::take(&mut self.objective), self.update_type);
            context.only_core = false;
        }

        Ok(vec![])
    }
}
