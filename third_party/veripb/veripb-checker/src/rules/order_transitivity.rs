use std::rc::Rc;

use veripb_formula::prelude::*;
use veripb_rules::OrderTransitivityRule;

use crate::{order_context::TransitivityContext, prelude::*};

impl Checkable for OrderTransitivityRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        context
            .subcontexts
            .push(Subcontext::Transitivity(TransitivityContext::default()));
        Ok(vec![])
    }
}
