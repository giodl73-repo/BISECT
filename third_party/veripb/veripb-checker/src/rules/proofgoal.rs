use std::rc::Rc;

use veripb_formula::prelude::*;
use veripb_rules::{
    ProofgoalID, ProofgoalRule,
    elaboration::{ConstraintLookup, Elaboratable, ElaborationError, Elaborator},
};

use crate::{prelude::*, proofgoal_context::ProofgoalContext};

#[derive(Debug)]
pub struct ProofgoalChecker {
    inner: ProofgoalRule,
    pbc_out_id_from_hint: Option<(usize, usize)>,
}
impl ProofgoalChecker {
    fn new(inner: ProofgoalRule) -> Self {
        Self {
            inner,
            pbc_out_id_from_hint: None,
        }
    }
}

impl Checkable for ProofgoalChecker {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError> {
        if let Some(Subcontext::Subproof(subproof_context)) = context.subcontexts.last_mut() {
            let proofgoal = match &mut self.inner.proofgoal_id {
                ProofgoalID::Internal(index) => subproof_context.pop_internal_goal(*index)?,
                ProofgoalID::Database(index) => {
                    *index = database.normalize_id(*index);
                    subproof_context.pop_database_goal(*index as usize, database)?
                }
            };
            let constraints = proofgoal.into_counterexample();
            let proofgoal_context = Subcontext::Proofgoal(ProofgoalContext::new(database.len()));
            context.subcontexts.push(proofgoal_context);

            if context.elaborator.is_none() {
                return Ok(constraints);
            }

            let Some(Subcontext::Subproof(subproof_context)) =
                context.subcontexts.get(context.subcontexts.len() - 2)
            else {
                return Ok(constraints);
            };

            if !subproof_context.proof_by_contradiction_subproof {
                return Ok(constraints);
            }

            let hint = subproof_context.additional_hint.unwrap();
            let counterexample_id = database.len() + constraints.len() - 1;
            self.pbc_out_id_from_hint = Some((counterexample_id, hint));

            Ok(constraints)
        } else {
            Err(CheckingError::ProofgoalOutsideSubproof)
        }
    }
}

impl From<ProofgoalRule> for ProofgoalChecker {
    fn from(inner: ProofgoalRule) -> Self {
        Self::new(inner)
    }
}

impl Elaboratable for ProofgoalChecker {
    fn elaborate<C: ConstraintLookup, V: VarNameLookup>(
        &self,
        elaborator: &mut Elaborator,
        database: &C,
        _var_lookup: &V,
    ) -> Result<(), ElaborationError> {
        let Some((counterexample_id, hint)) = self.pbc_out_id_from_hint else {
            elaborator.write_proofgoal(&self.inner.proofgoal_id, Some(database));
            return Ok(());
        };

        elaborator.dec_id();
        let hint_out_id = database
            .get_entry_usize(hint)
            .expect("Proof by contradiction should have additional hint")
            .get_out_id(hint)
            .expect("Constraint should have output ID.");
        database
            .get_entry_usize(counterexample_id)
            .expect("Counterexample constraint should be in the database")
            .set_out_id(counterexample_id, hint_out_id);
        Ok(())
    }
}
