//! Trait definitions for pseudo-Boolean constraints.

use std::{
    fmt::{Debug, Display},
    ops::Mul,
    str::FromStr,
};

use malachite_bigint::BigInt;
use num_integer::Integer;
use num_traits::{
    CheckedAdd, CheckedMul, CheckedSub, NumAssign, NumAssignOps, NumAssignRef, NumOps, NumRef, One,
    Signed,
};
use spire_enum::prelude::{delegate_impl, delegated_enum};

use crate::{prelude::*, substitution::Substitutable};

/// Direction of the reification
pub enum ReificationDir {
    /// Variables implies constraint.
    VarImplIneq,
    /// Constraint implies variable.
    IneqImplVar,
}

pub const WEAKEN_ALL_THRESHOLD: usize = 10;

/// Enum to differentiate pseudo-Boolean constraints of different types.
#[delegated_enum(impl_conversions)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PBConstraintEnum {
    Clause(Clause),
    Cardinality(Cardinality),
    GeneralPBI64(GeneralPBConstraint<i64>),
    GeneralPBI128(GeneralPBConstraint<i128>),
    GeneralPBBigInt(GeneralPBConstraint<BigInt>),
}

impl PBConstraintEnum {
    /// Get a reference to the internal [`Clause`] of the enum.
    ///
    /// # Panics
    /// If the enum variant is not [`PBConstraintEnum::Clause`], this function will panic.
    #[inline]
    pub fn as_clause(&self) -> &Clause {
        if let PBConstraintEnum::Clause(clause) = self {
            return clause;
        }
        unreachable!()
    }

    /// Get a reference to the internal [`Cardinality`] of the enum.
    ///
    /// # Panics
    /// If the enum variant is not [`PBConstraintEnum::Cardinality`], this function will panic.
    #[inline]
    pub fn as_card(&self) -> &Cardinality {
        if let PBConstraintEnum::Cardinality(card) = self {
            return card;
        }
        unreachable!()
    }

    /// Get a reference to the internal [`GeneralPBConstraint<N>`] of the enum.
    ///
    /// # Panics
    /// If the enum variant is not [`PBConstraintEnum::GeneralPBI64`], [`PBConstraintEnum::GeneralPBI128`], or [`PBConstraintEnum::GeneralPBBigInt`], this function will panic.
    ///
    /// # Safety
    /// If the type of the constraint in the enum does not match the generic integer type `N`, this function will result in undefined behaviour.
    #[inline]
    pub fn as_general_pb<N: Int>(&self) -> &GeneralPBConstraint<N> {
        match self {
            Self::GeneralPBI64(constraint) => unsafe {
                std::mem::transmute::<&GeneralPBConstraint<i64>, &GeneralPBConstraint<N>>(
                    constraint,
                )
            },
            Self::GeneralPBI128(constraint) => unsafe {
                std::mem::transmute::<&GeneralPBConstraint<i128>, &GeneralPBConstraint<N>>(
                    constraint,
                )
            },
            Self::GeneralPBBigInt(constraint) => unsafe {
                std::mem::transmute::<&GeneralPBConstraint<BigInt>, &GeneralPBConstraint<N>>(
                    constraint,
                )
            },
            _ => unreachable!(),
        }
    }

    /// Add a summand to the constraint.
    ///
    /// This function unpacks `summand` to its specific type and calls the corresponding [PBConstraint::add()].
    #[inline]
    pub fn add(&mut self, summand: &PBConstraintEnum) -> Option<PBConstraintEnum> {
        match summand {
            PBConstraintEnum::Clause(clause) => PBConstraint::add(self, clause),
            PBConstraintEnum::Cardinality(cardinality) => PBConstraint::add(self, cardinality),
            PBConstraintEnum::GeneralPBI64(constraint) => PBConstraint::add(self, constraint),
            PBConstraintEnum::GeneralPBI128(constraint) => PBConstraint::add(self, constraint),
            PBConstraintEnum::GeneralPBBigInt(constraint) => PBConstraint::add(self, constraint),
        }
    }

    /// Check if this constraint (ordinarily) syntactically implies the `target` constraint.
    ///
    /// This function unpacks `target` and itself to its specific type and calls the corresponding [PBConstraintGetter::implies()].
    #[inline]
    pub fn implies(&self, target: &PBConstraintEnum) -> bool {
        match target {
            PBConstraintEnum::Clause(clause) => self.implies_impl(clause),
            PBConstraintEnum::Cardinality(cardinality) => self.implies_impl(cardinality),
            PBConstraintEnum::GeneralPBI64(constraint) => self.implies_impl(constraint),
            PBConstraintEnum::GeneralPBI128(constraint) => self.implies_impl(constraint),
            PBConstraintEnum::GeneralPBBigInt(constraint) => self.implies_impl(constraint),
        }
    }

    /// Check if this constraint (ordinarily) syntactically implies the `target` constraint.
    ///
    /// This function unpacks itself to its specific type and calls the corresponding [PBConstraintGetter::implies()].
    #[inline]
    fn implies_impl<C: PBConstraintGetter>(&self, target: &C) -> bool {
        match self {
            PBConstraintEnum::Clause(clause) => PBConstraintGetter::implies(clause, target),
            PBConstraintEnum::Cardinality(cardinality) => {
                PBConstraintGetter::implies(cardinality, target)
            }
            PBConstraintEnum::GeneralPBI64(constraint) => {
                PBConstraintGetter::implies(constraint, target)
            }
            PBConstraintEnum::GeneralPBI128(constraint) => {
                PBConstraintGetter::implies(constraint, target)
            }
            PBConstraintEnum::GeneralPBBigInt(constraint) => {
                PBConstraintGetter::implies(constraint, target)
            }
        }
    }
}

/// Construct a new constraint from its terms, degree, and coeff_sum.
///
/// This function normalizes the constraint and returns the most efficient data structure to represent this pseudo-Boolean constraint.
pub fn constraint_from_terms_and_coeff_sum<N>(
    terms: Vec<GeneralPBTerm<N>>,
    degree: N,
    coeff_sum: N,
) -> PBConstraintEnum
where
    N: Int,
    PBConstraintEnum: From<GeneralPBConstraint<N>>,
{
    // Creating a constraint takes care of normalizing the constraint.
    GeneralPBConstraint::from_terms(terms, coeff_sum, degree).into_smallest_type()
}

/// Construct a new constraint from its terms and degree.
///
/// This function normalizes the constraint and returns the most efficient data structure to represent this pseudo-Boolean constraint.
pub fn constraint_from_terms<N>(terms: Vec<GeneralPBTerm<N>>, degree: N) -> PBConstraintEnum
where
    N: Int,
    PBConstraintEnum: From<GeneralPBConstraint<N>>,
{
    let mut coeff_sum = N::zero();
    for term in terms.iter() {
        coeff_sum += &term.coeff.abs();
    }

    constraint_from_terms_and_coeff_sum(terms, degree, coeff_sum)
}

/// Getter functions for pseudo-Boolean constraints. These constraints cannot be implemented for the `PBConstraintEnum`, as the return value of them depends on an associated type that is not the same for all types of constraints.
pub trait PBConstraintGetter
where
    Self: PBConstraint,
{
    type CoeffType: Int;
    type TermType: PBTerm<CoeffType = Self::CoeffType>;

    /// Get the degree (right-hand side) of the pseudo-Boolean constraint.
    fn get_degree(&self) -> &Self::CoeffType;

    /// Get the terms (left-hand side) of the pseudo-Boolean constraint.
    fn get_terms(&self) -> &Vec<Self::TermType>;

    /// Get the sum of the coefficients of all the terms in normalized form.
    fn get_coeff_sum(&self) -> Self::CoeffType;

    /// Get the literals of the constraint.
    fn get_lits(&self) -> impl Iterator<Item = &Lit>;

    /// Check if this constraint (`self`) (strongly) syntactically implies the constraint `target`.
    ///
    /// Strong syntactic implication checks if it is possible to add literal axioms, then saturate the result and finally add more literal axioms to `self`, in such a way that we can derive `target`.
    #[inline]
    fn implies<C: PBConstraintGetter>(&self, target: &C) -> bool {
        // If target constraint is trivial, then it is always implied.
        if target.is_trivial() {
            return true;
        }

        // If source constraint is a contradiction, then the target is always implied.
        if self.is_contradicting() {
            return true;
        }

        // The `weaken_cost` represents the potential how much we lowered our degree due to necessary weakening steps, i.e., if it is positive then we need more weakening than the difference between source_degree and target_degree.
        let mut weaken_cost = target.get_degree().clone().into();
        weaken_cost -= self.get_degree().clone().into();
        if weaken_cost.is_positive() {
            return false;
        }
        let mut target_terms_iter = target.get_terms().iter();
        let mut cur_target_term = target_terms_iter.next();
        for source_term in self.get_terms().iter() {
            let lit = source_term.get_lit();
            let source_coeff = source_term.get_coeff().clone().into();

            // Get the next target term to compare with. Since terms are sorted by literal integer, we just have to iterate until target term literal is at least the source term lit.
            while cur_target_term.is_some_and(|t| t.get_lit() < lit) {
                cur_target_term = target_terms_iter.next();
            }

            match cur_target_term {
                // Source term literal in target constraint.
                Some(target_term) if target_term.get_lit() == lit => {
                    let target_coeff = target_term.get_coeff().to_owned().into();
                    if source_coeff > target_coeff && target_term.get_coeff() < target.get_degree()
                    {
                        // Source term coeff is larger target term coeff. We need to lower the coeff and since target_coeff < target_degree, saturation does not help.
                        let diff = source_coeff - target_coeff;
                        weaken_cost += &diff;
                        if weaken_cost.is_positive() {
                            return false;
                        }
                    }
                }
                // Source term variable not in target constraint.
                None | Some(_) => {
                    weaken_cost += source_coeff;
                    if weaken_cost.is_positive() {
                        return false;
                    }
                }
            }
        }

        debug_assert!(!weaken_cost.is_positive());
        true
    }
}

/// A pseudo-Boolean constraint is an integer linear inequality over literals. A pseudo-Boolean constraint is always viewed in normalized form, i.e., all coefficients are positive integers, the right-hand side is a non-negative integer, the terms are over distinct variables, and the comparison operator is `>=`.
///
/// # Invariants
/// 1. Terms inside a constraint are sorted by literal number (odd/even encoding for negation) in ascending order.
/// 2. All constraints are stored with terms over distinct variables and all coefficients are positive.
pub trait PBConstraint {
    /// The number of terms in the constraint.
    fn len(&self) -> usize;

    /// Returns `true` if and only if the constraints contains no terms.
    fn is_empty(&self) -> bool;

    /// Get the `index`-th constraint literal.
    fn get_lit(&self, index: usize) -> Option<&Lit>;

    /// Get the `index`-th term in the constraint as a BigInt term.
    fn get_term(&self, index: usize) -> Option<GeneralPBTerm<BigInt>>;

    /// Saturate the pseudo-Boolean constraint.
    fn saturate(&mut self);

    /// Weaken the variable with `var_idx` in the pseudo-Boolean constraint
    fn weaken(&mut self, var_idx: VarIdx) -> Option<PBConstraintEnum>;

    /// Weaken all variables in the vector `var_idxs` in the pseudo-Boolean constraint
    fn weaken_all(&mut self, var_idxs: Vec<VarIdx>) -> Option<PBConstraintEnum>;

    /// Divide the constraint by the `divisor` using cutting planes division in normalized form.
    fn normalized_form_div(&mut self, divisor: &BigInt);

    /// Divide the constraint in variable form by the `divisor` using the Chvatal-Gomory cut rule.
    fn variable_form_div(&mut self, divisor: &BigInt) -> Option<PBConstraintEnum>;

    /// Mixed integer rounding (MIR) cut dividing the constraint by `divisor` in normalized form.
    fn normalized_form_mir(&mut self, divisor: &BigInt) -> Option<PBConstraintEnum>;

    /// Mixed integer rounding (MIR) cut dividing the constraint by `divisor` in variable form.
    fn variable_form_mir(&mut self, divisor: &BigInt) -> Option<PBConstraintEnum>;

    /// Multiply the constraint by `factor`.
    fn multiply(&mut self, factor: &BigInt) -> Option<PBConstraintEnum>;

    /// Add the constraint `summand` to this constraint.
    fn add<C: PBConstraintGetter>(&mut self, summand: &C) -> Option<PBConstraintEnum>;

    /// Lower the right-hand side of the constraint by `amount`.
    fn lower_rhs(&mut self, amount: &BigInt) -> Option<PBConstraintEnum>;

    /// Get the negation of this constraint.
    fn negate(&self) -> PBConstraintEnum;

    /// Get the substituted constraint of this constraint under the given `substitution`.
    fn substitute(&self, substitution: &impl Substitutable) -> PBConstraintEnum;

    /// Get the maximum coefficient of the constraint.
    #[inline]
    fn get_max_coeff(&self) -> BigInt {
        BigInt::one()
    }

    /// Check if the constraint is a contradiction, i.e., the constraint is always falsified.
    fn is_contradicting(&self) -> bool;

    /// Returns true if the constraints is trivialized by an [`Assignment<BooleanVar>`]. This means that it is not required that all literals have to be assigned.
    fn is_satisfied(&self, assignment: &Assignment<BooleanVar>) -> bool;

    /// Returns true if the constraints is falsified by an [`Assignment<BooleanVar>`]. This means that it is not required that all literals have to be assigned.
    fn is_falsified(&self, assignment: &Assignment<BooleanVar>) -> bool;

    /// Check if the constraint is trivial, i.e., the constraint is always satisfied.
    fn is_trivial(&self) -> bool;

    /// Get the propgations of this constraint with respect to the given `assignment`.
    ///
    /// Used for annotated RUP checks.
    fn propagate(&self, assignment: &mut Assignment<BooleanVar>) -> ConstraintPropagationResult;

    /// Trace the propagation of the current assignment with respect to the given `assignment`.
    ///
    /// Used for annotated RUP checks when propagation fails to print propagation trail.
    fn traced_propagate(&self, assignment: &mut Assignment<BooleanVar>) -> Vec<Lit>;

    /// Assign the literals in this constraint which are falsified by the `assignment` to false in the `marking`.
    fn mark_negated_lits(
        &self,
        assignment: &Assignment<BooleanVar>,
        marking: &mut Assignment<BooleanVar>,
    );

    /// Get [`PBConstraintEnum`] turning the constraint to the most restrictive type.
    ///
    /// From most restrictive to least restrictive, the types are:
    /// 1. [`Clause`]
    /// 2. [`Cardinality`]
    /// 3. [`GeneralPBConstraint<i64>`]
    /// 4. [`GeneralPBConstraint<i128>`]
    /// 5. [`GeneralPBConstraint<BigInt>`]
    fn into_smallest_type(self) -> PBConstraintEnum;

    /// Get the reification constraint for this constraint with respect to `var` and reification `direction`. This function assumes that `var` is not a variable in the constraint.
    fn get_reification(&self, var: VarIdx, direction: ReificationDir) -> PBConstraintEnum;
}

#[delegate_impl]
impl PBConstraint for PBConstraintEnum {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_lit(&self, index: usize) -> Option<&Lit>;
    fn get_term(&self, index: usize) -> Option<GeneralPBTerm<BigInt>>;
    fn saturate(&mut self);
    fn weaken(&mut self, var_idx: VarIdx) -> Option<PBConstraintEnum>;
    fn weaken_all(&mut self, var_idxs: Vec<VarIdx>) -> Option<PBConstraintEnum>;
    fn normalized_form_div(&mut self, divisor: &BigInt);
    fn variable_form_div(&mut self, divisor: &BigInt) -> Option<PBConstraintEnum>;
    fn normalized_form_mir(&mut self, divisor: &BigInt) -> Option<PBConstraintEnum>;
    fn variable_form_mir(&mut self, divisor: &BigInt) -> Option<PBConstraintEnum>;
    fn multiply(&mut self, factor: &BigInt) -> Option<PBConstraintEnum>;
    fn add<C: PBConstraintGetter>(&mut self, summand: &C) -> Option<PBConstraintEnum>;
    fn lower_rhs(&mut self, amount: &BigInt) -> Option<PBConstraintEnum>;
    fn negate(&self) -> PBConstraintEnum;
    fn substitute(&self, substitution: &impl Substitutable) -> PBConstraintEnum;
    fn get_max_coeff(&self) -> BigInt;
    fn is_contradicting(&self) -> bool;
    fn is_satisfied(&self, assignment: &Assignment<BooleanVar>) -> bool;
    fn is_falsified(&self, assignment: &Assignment<BooleanVar>) -> bool;
    fn is_trivial(&self) -> bool;
    fn propagate(&self, assignment: &mut Assignment<BooleanVar>) -> ConstraintPropagationResult;
    fn traced_propagate(&self, assignment: &mut Assignment<BooleanVar>) -> Vec<Lit>;
    fn mark_negated_lits(
        &self,
        assignment: &Assignment<BooleanVar>,
        marking: &mut Assignment<BooleanVar>,
    );
    fn into_smallest_type(self) -> PBConstraintEnum;
    fn get_reification(&self, var: VarIdx, direction: ReificationDir) -> PBConstraintEnum;
}

#[delegate_impl]
impl ToPrettyString for PBConstraintEnum {
    fn to_pretty_string<V: VarNameLookup>(&self, var_names: &V) -> String;
}

/// Super trait for numbers used in pseudo-Boolean constraints.
pub trait Int:
    Display
    + NumAssignOps
    + NumAssign
    + NumRef
    + NumOps
    + NumAssignRef
    + Clone
    + Debug
    + ToString
    + From<i64>
    + TryFrom<i64>
    + TryFrom<i128>
    + TryFrom<BigInt>
    + Into<BigInt>
    + TryInto<BigInt>
    + TryInto<i128>
    + TryInto<i64>
    + Signed
    + FromStr
    + CheckedAdd
    + CheckedSub
    + CheckedMul
    + Integer
    + Mul<BigInt, Output = BigInt>
    + 'static
{
}
impl Int for i64 {}
impl Int for i128 {}
impl Int for BigInt {}

/// Check whether a constraint with the given coefficient sum and degree may be stored with coefficients of type `N`.
///
/// The coefficient sum and degree fitting in `N` is not enough: the propagator computes the slack `coeff_sum - degree`, and negation and reification compute `coeff_sum - degree + 1`, all in `N`. A constraint with a large negative degree (which is trivially satisfied) can have both values in range while its slack is not, and the slack then wrapped round and the constraint was reported as conflicting. For [`BigInt`] this is always true.
#[inline]
pub fn slack_fits<N: Int>(coeff_sum: &N, degree: &N) -> bool {
    coeff_sum
        .checked_sub(degree)
        .and_then(|slack| slack.checked_add(&N::one()))
        .is_some()
}
