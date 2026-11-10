use std::rc::Rc;

use veripb_formula::prelude::DBConstraint;
use veripb_rules::AssertionRule;

use crate::prelude::*;

impl Checkable for AssertionRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        context.assertion_used = true;
        Ok(vec![self.constraint.clone()])
    }
}
