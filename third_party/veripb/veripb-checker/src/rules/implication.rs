use std::rc::Rc;

use veripb_formula::prelude::*;
use veripb_rules::ImpliesRule;

use crate::prelude::*;

impl Checkable for ImpliesRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        self.hint = Some(check_implication(
            context.only_core,
            &mut context.propagation_engine,
            database,
            &self.constraint,
            self.hint,
        )?);

        if self.add_result {
            Ok(vec![self.constraint.clone()])
        } else {
            Ok(vec![])
        }
    }

    #[inline]
    fn get_returned_id(&self) -> Option<isize> {
        if self.add_result { None } else { self.hint }
    }
}
