use std::rc::Rc;

use veripb_formula::prelude::*;
use veripb_propagator::propagation_engine::PropagationEngine;
use veripb_rules::OrderSpecificationRule;

use crate::{order_context::SpecificationContext, prelude::*};

impl Checkable for OrderSpecificationRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        // Check that we are inside the order definition.
        if !matches!(context.subcontexts.last(), Some(Subcontext::Order(_))) {
            return Err(CheckingError::SpecNotAllowedHere);
        }
        Ok(vec![])
    }

    #[inline]
    fn swaps_database(&self) -> bool {
        true
    }

    #[inline]
    fn get_new_database(&mut self) -> (Database, PropagationEngine) {
        (Database::new(), new_veripb_propagation_engine())
    }

    #[inline]
    fn handle_old_database(
        &mut self,
        context: &mut Context,
        database: Database,
        prop_engine: PropagationEngine,
    ) {
        let mut specification_context = SpecificationContext::new(database, prop_engine);
        if let Some(elaborator) = context.elaborator.as_mut() {
            specification_context.stored_elaborator_id = elaborator.current_id;
            elaborator.current_id = 0;
        }
        context
            .subcontexts
            .push(Subcontext::Specification(specification_context));
    }
}
