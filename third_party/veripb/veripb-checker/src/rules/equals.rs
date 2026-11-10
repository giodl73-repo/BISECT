use std::rc::Rc;

use veripb_formula::prelude::DBConstraint;
use veripb_rules::EqualsRule;

use crate::prelude::*;

impl Checkable for EqualsRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        match self.hint.as_mut() {
            None => {
                database.update_unique_index(&mut context.propagation_engine)?;
                if let Some(constraint) = database.lookup(&self.constraint) {
                    if !context.only_core || constraint.is_core_constraint() {
                        self.hint = Some(constraint.get_some_id() as isize);
                    } else {
                        return Err(CheckingError::not_found(&self.constraint, true));
                    }
                } else {
                    return Err(CheckingError::not_found(&self.constraint, false));
                }
            }
            Some(index) => {
                *index = database.normalize_id(*index);
                let constraint = database.get_entry_usize(*index as usize)?;
                if context.only_core && !constraint.is_core_constraint_id(*index as usize) {
                    return Err(CheckingError::CoreSubproofUsingNonCoreConstraint(*index));
                }
                if *constraint != self.constraint {
                    return Err(CheckingError::not_equal(&self.constraint, constraint));
                }
            }
        }
        if self.add_result {
            Ok(vec![self.constraint.clone()])
        } else {
            Ok(vec![])
        }
    }

    #[inline]
    fn get_returned_id(&self) -> Option<isize> {
        self.hint
    }
}
