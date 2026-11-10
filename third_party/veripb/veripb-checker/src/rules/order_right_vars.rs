use veripb_formula::prelude::*;
use veripb_rules::OrderRightVariablesRule;

use crate::prelude::*;

impl Checkable for OrderRightVariablesRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<std::rc::Rc<DBConstraint>>, CheckingError> {
        if let Some(Subcontext::Order(order_context)) = context.subcontexts.last_mut() {
            order_context.right_vars_defined = true;
            order_context.constructed_order.right_vars = self.vars.clone();

            if order_context.constructed_order.set_variables(
                crate::order_context::OrderVariableKind::Mapped,
                self.vars.as_slice(),
            ) {
                Err(CheckingError::OrderVariablesNonDistinct)
            } else if !order_context
                .constructed_order
                .check_symmetric_left_and_right_variables()
            {
                Err(CheckingError::AsymmetricOrderVariables)
            } else {
                Ok(vec![])
            }
        } else {
            Err(CheckingError::RuleOnlyAllowedInOrderVars)
        }
    }
}
