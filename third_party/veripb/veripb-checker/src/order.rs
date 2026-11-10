//! Implementation of a reflexive and transitive [`Order`] defined over pseudo-Boolean constraints.
//!
//! The [`Order`] is used for defining an order over assignments to a subset of literals, which is used in VeriPB for redundance- and dominance-based strengthening.

use std::rc::Rc;

use ahash::{HashSet, RandomState};
use itertools::izip;
use veripb_formula::prelude::*;

use crate::{order_context::OrderVariableKind, prelude::*, rules::ScopeId};

/// Order to be stored in the list of defined orders.
#[derive(Debug, Default, Clone)]
pub struct Order {
    /// Left-hand side variables of the order.
    pub left_vars: Vec<VarIdx>,
    /// Right-hand side variables of the order.
    pub right_vars: Vec<VarIdx>,
    /// Additional variables used by the order.
    pub aux_vars: Vec<VarIdx>,
    /// Constraints that define the order. All of these constraints have to be true for the order to hold.
    pub definition: Vec<Rc<DBConstraint>>,
    /// Specification constraints of the order.
    pub specification: Vec<Rc<DBConstraint>>,
    /// A vector to keep track of auxiliary variables for this order.
    pub vars_table: Vec<OrderVariableKind>,
}

impl Order {
    /// Check that all variables in the left-hand and right-hand side variables are distinct.
    pub fn check_distinct_left_and_right_variables(&self) -> bool {
        let mut vars = {
            let mut vec = self.left_vars.clone();
            vec.extend_from_slice(self.right_vars.as_slice());
            vec
        };
        vars.sort();
        vars.as_slice()
            .windows(2usize)
            .all(|a| a[0usize] != a[1usize])
    }

    /// Check that there are the same number of left-hand and right-hand side variables.
    pub fn check_symmetric_left_and_right_variables(&self) -> bool {
        self.left_vars.len() == self.right_vars.len()
    }

    /// Check that all auxiliary variables are distinct.
    pub fn check_distinct_aux_variables(&self) -> bool {
        let mut vars = self.aux_vars.clone();
        vars.sort();
        vars.as_slice()
            .windows(2usize)
            .all(|a| a[0usize] != a[1usize])
    }

    /// Set the variables `vars` to the kind `kind` in the variables lookup table.
    ///
    /// Returns true if any of the variables was already set to either `Mapped` or `Auxiliary`,
    /// including duplicates in `vars`.
    ///
    /// Don't call this with `kind` == `External`, the semantics are then undefined but safe.
    pub fn set_variables(&mut self, kind: OrderVariableKind, vars: &[VarIdx]) -> bool {
        vars.iter().any(|var| {
            if var >= &self.vars_table.len() {
                self.vars_table.resize(*var, OrderVariableKind::External);
                self.vars_table.push(kind);
                false
            } else {
                let slot = self.vars_table.get_mut(*var).unwrap();
                if slot.is_external() {
                    *slot = kind;
                    false
                } else {
                    !kind.is_external()
                }
            }
        })
    }

    /// Check if the variable with index `var` is an auxiliary variable for this [`Order`].
    pub fn is_auxiliary_variable(&self, var: VarIdx) -> bool {
        self.vars_table
            .get(var)
            .is_some_and(|kind| kind.is_auxiliary())
    }

    /// Check if the variable with index `var` is a variable used in this [`Order`].
    pub fn is_order_variable(&self, var: VarIdx) -> bool {
        self.vars_table
            .get(var)
            .is_some_and(|kind| kind.is_mapped_or_auxiliary())
    }
}

/// The currently loaded [`Order`] with the set of literals the order is loaded over and some cached data.
#[derive(Debug)]
pub struct ActiveOrder {
    /// Is the substitution of left-hand side variables to the literals the order is loaded over and right-hand side variables to the current substitution set.
    pub substitution_left_set: bool,
    /// Is the substitution of right-hand side variables to the literals the order is loaded over and left-hand side variables to the current substitution set.
    pub substitution_right_set: bool,
    /// Cached substitution of left-hand side variables to the literals the order is loaded over.
    substitution_left: Substitution,
    /// Cached substitution of right-hand side variables to the literals the order is loaded over.
    substitution_right: Substitution,
    /// The loaded order.
    pub order: Order,
    /// The literals the order is loaded over.
    lits: Vec<Lit>,
    /// Variables of the literals the order is loaded over.
    var_set: HashSet<VarIdx>,
}

impl ActiveOrder {
    /// Construct a new active order from a defined order.
    pub fn new(order: &Order, lits: Vec<Lit>) -> Self {
        let mut substitution_left = Substitution::default();
        let mut substitution_right = Substitution::default();
        let mut var_set = HashSet::with_hasher(RandomState::with_seeds(42, 42, 42, 42));
        for (&left_var, &right_var, &target_lit) in
            izip!(order.left_vars.iter(), order.right_vars.iter(), lits.iter())
        {
            substitution_left.set(left_var, SubstitutionValue::lit(target_lit));
            substitution_right.set(right_var, SubstitutionValue::lit(target_lit));
            var_set.insert(target_lit.get_var());
        }
        Self {
            substitution_left_set: false,
            substitution_right_set: false,
            substitution_left,
            substitution_right,
            order: order.clone(),
            lits,
            var_set,
        }
    }

    /// Get the proofgoals that the active order is not worsening under the `substitution`. This is used for redundance-based strengthening.
    pub fn get_proofgoals(&mut self, substitution: &Substitution) -> Vec<Proofgoal> {
        let mut proofgoals = Vec::new();

        // Optimization: if the constraint introduced is a reification, and the witness hence only touches fresh variables,
        // then the order proofgoal follows directly from reflexivity. This is such a common case that in this case we do
        // not even want to go through the order variables, since this could be a major overhead.
        if substitution.len() == 1
            && !self
                .var_set
                .contains(substitution.syntactic_support().next().unwrap())
        {
            for constraint in self.order.definition.iter() {
                proofgoals.push(Proofgoal::mk_single_constraint(
                    constraint.clone(),
                    Some(ScopeId::LessEqual),
                    true,
                ));
            }
            return proofgoals;
        }

        let mut witness_touching_order_vars = false;
        for (&left_var, &target_lit) in izip!(self.order.left_vars.iter(), self.lits.iter()) {
            match substitution.get(target_lit.get_var()) {
                Some(value) => {
                    witness_touching_order_vars = true;
                    if target_lit.is_negated() {
                        self.substitution_right.set(left_var, value.into_negation())
                    } else {
                        self.substitution_right.set(left_var, value)
                    }
                }
                None => self
                    .substitution_right
                    .set(left_var, SubstitutionValue::lit(target_lit)),
            };
        }
        self.substitution_right_set = true;

        for constraint in self.order.definition.iter() {
            proofgoals.push(Proofgoal::mk_single_constraint(
                Rc::new(constraint.substitute(&self.substitution_right)),
                Some(ScopeId::LessEqual),
                !witness_touching_order_vars,
            ));
        }
        proofgoals
    }

    /// Get the proofgoals that the active order is strictly improving under the `substitution`. This is used for dominance-based strengthening.
    pub fn get_proofgoals_strict(&mut self, substitution: &Substitution) -> Vec<Proofgoal> {
        let mut proofgoals = self.get_proofgoals(substitution);
        for (&right_var, &target_lit) in izip!(self.order.right_vars.iter(), self.lits.iter()) {
            match substitution.get(target_lit.get_var()) {
                Some(value) => {
                    if target_lit.is_negated() {
                        self.substitution_left.set(right_var, value.into_negation())
                    } else {
                        self.substitution_left.set(right_var, value)
                    }
                }
                None => self
                    .substitution_left
                    .set(right_var, SubstitutionValue::lit(target_lit)),
            };
        }
        self.substitution_left_set = true;

        let mut constraints = Vec::new();
        for constraint in self.order.definition.iter() {
            constraints.push(Rc::new(constraint.substitute(&self.substitution_left)));
        }
        proofgoals.push(Proofgoal::mk_multi_constraint(
            constraints,
            Some(ScopeId::GreaterEqual),
        ));

        proofgoals
    }

    /// Get the proofgoals that the active order is not worsening under the `substitution`. This is used for redundance-based and dominance-based strengthening.
    pub fn get_specification_less_equal(
        &mut self,
        substitution: &Substitution,
    ) -> Vec<Rc<DBConstraint>> {
        // Set up substitution if it is not already done so.
        if !self.substitution_right_set {
            for (&left_var, &target_lit) in izip!(self.order.left_vars.iter(), self.lits.iter()) {
                match substitution.get(target_lit.get_var()) {
                    Some(value) => {
                        if target_lit.is_negated() {
                            self.substitution_right.set(left_var, value.into_negation())
                        } else {
                            self.substitution_right.set(left_var, value)
                        }
                    }
                    None => self
                        .substitution_right
                        .set(left_var, SubstitutionValue::lit(target_lit)),
                };
            }
            self.substitution_right_set = true;
        }

        let mut premises = Vec::new();
        for constraint in self.order.specification.iter() {
            premises.push(Rc::new(constraint.substitute(&self.substitution_right)));
        }
        premises
    }

    /// Get the proofgoal that the active order is not improving under the `substitution`. This is used for dominance-based strengthening.
    pub fn get_specification_greater_equal(
        &mut self,
        substitution: &Substitution,
    ) -> Vec<Rc<DBConstraint>> {
        // Set up substitution if it is not already done so.
        if !self.substitution_left_set {
            for (&right_var, &target_lit) in izip!(self.order.right_vars.iter(), self.lits.iter()) {
                match substitution.get(target_lit.get_var()) {
                    Some(value) => {
                        if target_lit.is_negated() {
                            self.substitution_left.set(right_var, value.into_negation())
                        } else {
                            self.substitution_left.set(right_var, value)
                        }
                    }
                    None => self
                        .substitution_left
                        .set(right_var, SubstitutionValue::lit(target_lit)),
                };
            }
            self.substitution_left_set = true;
        }

        let mut premises = Vec::new();
        for constraint in self.order.specification.iter() {
            premises.push(Rc::new(constraint.substitute(&self.substitution_left)));
        }
        premises
    }
}
