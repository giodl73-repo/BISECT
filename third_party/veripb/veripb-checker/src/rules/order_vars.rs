use std::rc::Rc;

use veripb_formula::prelude::*;
use veripb_rules::OrderVariablesRule;

use crate::prelude::*;

impl Checkable for OrderVariablesRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        match context.subcontexts.last_mut() {
            Some(Subcontext::Order(order_context)) => {
                order_context.inside_vars = true;
                Ok(vec![])
            }
            Some(Subcontext::Transitivity(transitivity_context)) => {
                transitivity_context.inside_vars = true;
                Ok(vec![])
            }
            _ => Err(CheckingError::VarsNotAllowedHere),
        }
    }
}
