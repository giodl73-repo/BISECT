use std::rc::Rc;

use veripb_formula::prelude::*;
use veripb_rules::SetLevelRule;

use crate::prelude::*;

impl Checkable for SetLevelRule {
    #[inline]
    fn compute(
        &mut self,
        context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        if self.level >= context.level_ids.len() {
            context.level_ids.resize(self.level + 1, Default::default());
        }
        context.current_level = Some(self.level);
        Ok(vec![])
    }
}
