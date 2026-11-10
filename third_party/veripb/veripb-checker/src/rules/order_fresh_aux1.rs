use veripb_formula::prelude::*;
use veripb_rules::OrderFreshAux1Rule;

use crate::prelude::*;

impl Checkable for OrderFreshAux1Rule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<std::rc::Rc<DBConstraint>>, CheckingError> {
        if let Some(Subcontext::Transitivity(transitivity_context)) = context.subcontexts.last_mut()
        {
            transitivity_context.fresh_aux_1 = self.vars.clone();
            Ok(vec![])
        } else {
            Err(CheckingError::RuleOnlyAllowedInOrderVars)
        }
    }
}
