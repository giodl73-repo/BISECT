//! Submodule defining the [`Checkable`] trait for a rule and implementing the trait for all [`VeriPBRule`]s.

mod assertion;
mod comment;
pub mod conclusion;
mod core;
pub mod cutting_planes;
mod define_order;
pub mod deletion;
mod dom_strengthening;
pub mod end_proof;
mod end_subproof;
mod end_time;
mod equal_objective;
mod equal_preserved;
mod equals;
mod fail;
mod formula;
mod header;
mod implication;
mod is_deleted;
mod load_order;
mod objective_update;
mod order_aux_vars;
mod order_def;
mod order_def_constraint;
mod order_fresh_aux1;
mod order_fresh_aux2;
mod order_fresh_right_vars;
mod order_left_vars;
mod order_proof;
mod order_reflexivity;
mod order_right_vars;
mod order_specification;
mod order_transitivity;
mod order_vars;
mod output;
mod preserved_add;
mod preserved_rm;
mod proof_by_contradiction;
mod proofgoal;
mod red_strengthening;
pub mod rup;
mod scope;
mod set_level;
pub mod solution_logging;
mod start_time;
mod strengthening_to_core;
mod unimplemented;

use spire_enum::prelude::{delegate_impl, delegated_enum};
use veripb_formula::prelude::DBConstraint;
use veripb_propagator::propagation_engine::PropagationEngine;

pub use veripb_rules::VeriPBRule;
use veripb_rules::elaboration::Elaboratable;
pub use veripb_rules::prelude::*;

use crate::{
    context::Context,
    database::Database,
    deletion_sequence::DeletionSequenceEnum,
    error::CheckingError,
    rules::{
        conclusion::ConclusionChecker, deletion::DeletionChecker, end_subproof::EndSubproofChecker,
        load_order::LoadOrderChecker, proofgoal::ProofgoalChecker,
        solution_logging::SolutionChecker,
    },
};

use std::{fmt::Debug, rc::Rc};

/// Trait for a [`VeriPBRule`] that is [`Checkable`] by the VeriPB [`ForwardsChecker`](crate::checker::ForwardsChecker).
pub trait Checkable: Debug + Elaboratable {
    /// Check the conditions for the rule and return the derived constraints.
    fn compute(
        &mut self,
        _context: &mut Context,
        _database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError>;

    /// Return the ids of constraints deleted by the rule.
    #[inline]
    fn get_deleted<'a>(
        &'a self,
        _database: &Database,
    ) -> Result<Option<DeletionSequenceEnum<'a>>, CheckingError> {
        Ok(None)
    }

    /// Get the constraint ID returned by this constraint. This is currently mainly used for `i` and `e` to get the constraint ID of the corresponding constraint.
    #[inline]
    fn get_returned_id(&self) -> Option<isize> {
        None
    }

    /// Return `true` if resulting constraints are added to the core set instead of the derived set by default.
    #[inline]
    fn add_constraints_to_core(&self, _context: &Context) -> bool {
        false
    }

    /// Returns `true` if the rule swaps out the currently considered database.
    #[inline]
    fn swaps_database(&self) -> bool {
        false
    }

    /// Get the new database to be swapped out.
    fn get_new_database(&mut self) -> (Database, PropagationEngine) {
        unreachable!()
    }

    /// Give the old database to the rule to handle it.
    fn handle_old_database(
        &mut self,
        _context: &mut Context,
        _database: Database,
        _prop_engine: PropagationEngine,
    ) {
        unreachable!()
    }
}

/// Generate enum for [`Checkable`] rules and provide function to cast a [`VeriPBRule`] to a [`CheckableRule`].
macro_rules! define_rule_checker {
    ($($variant:ident),* $(,)?) => {
        #[delegated_enum]
        #[derive(Debug)]
        pub enum CheckableRule {
            Deletion(DeletionChecker),
            Conclusion(ConclusionChecker),
            EndSubproof(EndSubproofChecker),
            LoadOrder(LoadOrderChecker),
            Solution(SolutionChecker),
            Proofgoal(ProofgoalChecker),
            $($variant($variant),)*
        }

        impl From<VeriPBRule> for CheckableRule {
            fn from(rule: VeriPBRule) -> Self {
                match rule {
                    VeriPBRule::Deletion(inner) => CheckableRule::Deletion(DeletionChecker::from(inner)),
                    VeriPBRule::ConclusionRule(inner) => CheckableRule::Conclusion(ConclusionChecker::from(inner)),
                    VeriPBRule::EndSubproof(inner) => CheckableRule::EndSubproof(EndSubproofChecker::from(inner)),
                    VeriPBRule::LoadOrderRule(inner) => CheckableRule::LoadOrder(LoadOrderChecker::from(inner)),
                    VeriPBRule::SolutionRule(inner) => CheckableRule::Solution(SolutionChecker::from(inner)),
                    VeriPBRule::ProofgoalRule(inner) => CheckableRule::Proofgoal(ProofgoalChecker::from(inner)),
                    $(VeriPBRule::$variant(inner) => CheckableRule::$variant(inner),)*
                }
            }
        }
    }
}

define_rule_checker! {
    AssertionRule,
    Comment,
    DefineOrderRule,
    DominanceBasedStrengtheningRule,
    EndProof,
    EndTimerRule,
    EqualsObjectiveRule,
    EqualsRule,
    FailRule,
    FormulaCheck,
    HeaderRule,
    ImpliesRule,
    IsDeletedCheck,
    MoveToCoreRule,
    ObjectiveUpdateRule,
    ObjectiveValueRule,
    OrderAuxVariablesRule,
    OrderDefConstraintRule,
    OrderDefRule,
    OrderFreshAux1Rule,
    OrderFreshAux2Rule,
    OrderFreshRightVariablesRule,
    OrderLeftVariablesRule,
    OrderProofRule,
    OrderReflexivityRule,
    OrderRightVariablesRule,
    OrderSpecificationRule,
    OrderTransitivityRule,
    OrderVariablesRule,
    OutputRule,
    PolRule,
    PreservedAddRule,
    PreservedEqualsRule,
    PreservedRemoveRule,
    ProofByContradiction,
    RUPRule,
    RedundanceBasedStrengtheningRule,
    ScopeRule,
    SetLevelRule,
    StartTimerRule,
    StrengtheningToCoreRule,
    UnimplementedRule,
}

#[delegate_impl]
impl Elaboratable for CheckableRule {
    fn elaborate<
        C: veripb_rules::elaboration::ConstraintLookup,
        V: veripb_formula::prelude::VarNameLookup,
    >(
        &self,
        _elaborator: &mut veripb_rules::elaboration::Elaborator,
        _database: &C,
        _var_name_lookup: &V,
    ) -> Result<(), veripb_rules::elaboration::ElaborationError>;
}

#[delegate_impl]
impl Checkable for CheckableRule {
    fn compute(
        &mut self,
        context: &mut Context,
        database: &mut Database,
    ) -> Result<Vec<Rc<DBConstraint>>, CheckingError>;
    fn get_deleted<'a>(
        &'a self,
        database: &Database,
    ) -> Result<Option<DeletionSequenceEnum<'a>>, CheckingError>;
    fn get_returned_id(&self) -> Option<isize>;
    fn add_constraints_to_core(&self, context: &Context) -> bool;
    fn swaps_database(&self) -> bool;
    fn get_new_database(&mut self) -> (Database, PropagationEngine);
    fn handle_old_database(
        &mut self,
        context: &mut Context,
        database: Database,
        prop_engine: PropagationEngine,
    );
}
