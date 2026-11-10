use std::rc::Rc;

use veripb_formula::prelude::DBConstraint;
use veripb_rules::{ScopeId, ScopeRule};

use crate::prelude::*;

impl Checkable for ScopeRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        if let Some(Subcontext::Subproof(subproof_context)) = context.subcontexts.last_mut() {
            let premises = if let Some(active_order) = &mut context.active_order {
                match self.id {
                    ScopeId::LessEqual => {
                        active_order.get_specification_less_equal(&subproof_context.witness)
                    }
                    ScopeId::GreaterEqual => {
                        active_order.get_specification_greater_equal(&subproof_context.witness)
                    }
                }
            } else {
                vec![]
            };

            if subproof_context.current_scope.is_some() {
                return Err(CheckingError::StartScopeWhileOtherScopeOpen);
            }
            subproof_context.scope_start_id = database.len();
            subproof_context.current_scope = Some(self.id);

            Ok(premises)
        } else {
            Err(CheckingError::ScopeOutsideSubproof)
        }
    }
}
