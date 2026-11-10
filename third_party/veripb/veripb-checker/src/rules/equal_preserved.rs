use std::rc::Rc;

use ahash::AHashSet;
use itertools::Itertools;
use veripb_formula::prelude::*;
use veripb_rules::PreservedEqualsRule;

use crate::prelude::*;

impl Checkable for PreservedEqualsRule {
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        if let Some(preserved_vars) = &context.preserved_variables {
            let vars_set = AHashSet::from_iter(self.vars.iter().cloned());
            if vars_set == *preserved_vars {
                Ok(vec![])
            } else {
                let missing_vars = preserved_vars
                    .difference(&vars_set)
                    .map(|v| v.to_pretty_string(&context.var_names))
                    .join(", ");
                let additional_vars = vars_set
                    .difference(preserved_vars)
                    .map(|v| v.to_pretty_string(&context.var_names))
                    .join(", ");
                Err(CheckingError::PreservedEqualsSetMismatch(
                    missing_vars,
                    additional_vars,
                ))
            }
        } else {
            Err(CheckingError::PreservedEqualsNoPreservedSetKnown)
        }
    }
}
