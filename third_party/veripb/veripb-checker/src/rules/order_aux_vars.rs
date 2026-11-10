use veripb_formula::prelude::*;
use veripb_rules::OrderAuxVariablesRule;

use crate::prelude::*;

impl Checkable for OrderAuxVariablesRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<std::rc::Rc<DBConstraint>>, CheckingError> {
        if let Some(Subcontext::Order(order_context)) = context.subcontexts.last_mut() {
            order_context.aux_vars_defined = true;
            order_context.constructed_order.aux_vars = self.vars.clone();

            if order_context.constructed_order.set_variables(
                crate::order_context::OrderVariableKind::Auxiliary,
                self.vars.as_slice(),
            ) {
                Err(CheckingError::OrderVariablesNonDistinct)
            } else {
                Ok(vec![])
            }
        } else {
            Err(CheckingError::RuleOnlyAllowedInOrderVars)
        }
    }
}
