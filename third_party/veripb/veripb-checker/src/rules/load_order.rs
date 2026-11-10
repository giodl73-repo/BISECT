use std::rc::Rc;

use veripb_formula::prelude::*;
use veripb_rules::{
    LoadOrderRule,
    elaboration::{ConstraintLookup, Elaboratable, ElaborationError, Elaborator},
};

use crate::{order::ActiveOrder, prelude::*};

#[derive(Debug)]
pub struct LoadOrderChecker {
    pub inner: LoadOrderRule,
    pub elaboration_required: bool,
}

impl LoadOrderChecker {
    pub fn new(inner: LoadOrderRule) -> Self {
        Self {
            inner,
            elaboration_required: true,
        }
    }
}

impl Checkable for LoadOrderChecker {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        match &self.inner.name {
            Some(name) => match context.orders.get(name) {
                Some(order) => {
                    // Load order.
                    if order.left_vars.len() != self.inner.lits.len() {
                        return Err(CheckingError::OrderWrongNumberVariables(
                            order.left_vars.len(),
                            self.inner.lits.len(),
                        ));
                    }
                    database.move_to_core_all(&mut context.propagation_engine)?;

                    context.active_order = Some(ActiveOrder::new(order, self.inner.lits.clone()));
                }
                None => return Err(CheckingError::OrderNotDefined(name.to_owned())),
            },
            None => {
                // Unload order.
                if context.active_order.is_none() {
                    self.elaboration_required = false;
                }
                context.active_order = None;
            }
        }
        Ok(vec![])
    }
}

impl From<LoadOrderRule> for LoadOrderChecker {
    fn from(inner: LoadOrderRule) -> Self {
        Self::new(inner)
    }
}

impl Elaboratable for LoadOrderChecker {
    fn elaborate<C: ConstraintLookup, V: VarNameLookup>(
        &self,
        elaborator: &mut Elaborator,
        _database: &C,
        var_lookup: &V,
    ) -> Result<(), ElaborationError> {
        if self.elaboration_required {
            elaborator.write_load_order(self.inner.name.as_deref(), &self.inner.lits, var_lookup);
        }
        Ok(())
    }
}
