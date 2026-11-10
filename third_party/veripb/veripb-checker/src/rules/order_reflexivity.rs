use std::rc::Rc;

use veripb_formula::prelude::*;
use veripb_rules::OrderReflexivityRule;

use crate::{order_context::ReflexivityContext, prelude::*};

impl Checkable for OrderReflexivityRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        context
            .subcontexts
            .push(Subcontext::Reflexivity(ReflexivityContext::default()));
        Ok(vec![])
    }
}
