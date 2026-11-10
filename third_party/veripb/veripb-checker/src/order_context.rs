//! [`Subcontext`] used by [`ForwardsChecker`] for checking the definition of an [`Order`].

use itertools::izip;
use veripb_formula::prelude::*;
use veripb_propagator::propagation_engine::PropagationEngine;
use veripb_rules::elaboration::Elaborator;

use crate::{order::Order, prelude::*};

/// Enum for deterimining if a variable is a left/right-hand side variable, an auxiliary variable, or neither (external).
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OrderVariableKind {
    /// Variable not used for this order.
    External = 0u8,
    /// Left-hand or right-hand side variable of order.
    Mapped = 1u8,
    /// Auxiliary variable of order.
    Auxiliary = 2u8,
}

impl OrderVariableKind {
    /// Check if this variable is external.
    pub fn is_external(self) -> bool {
        self as u8 == 0u8
    }

    /// Check if this variable is used for this order.
    pub fn is_mapped_or_auxiliary(self) -> bool {
        self as u8 >= 1u8
    }

    /// Check if this variables is auxiliary for this order.
    pub fn is_auxiliary(self) -> bool {
        self as u8 == 2u8
    }
}

/// Context for the order definition subcontext.
#[derive(Debug, Default)]
pub struct OrderContext {
    /// Name of the order.
    pub name: String,
    /// If inside the `vars` environment of the order definition.
    pub inside_vars: bool,
    /// If the left-hand side variables have been defined.
    pub left_vars_defined: bool,
    /// If the right-hand side variables have been defined.
    pub right_vars_defined: bool,
    /// If the auxiliary variables have been defined.
    pub aux_vars_defined: bool,
    /// If inside the `def` environment of the order definition.
    pub inside_def: bool,
    /// If reflexivity has been proven for this order.
    pub reflexivity_proven: bool,
    /// If transitivity has been proven for this order.
    pub transitivity_proven: bool,
    /// The constructed order.
    pub constructed_order: Order,
    /// Temporary storage of the main database.
    pub stored_database: Box<Database>,
    /// Temporary storage of the main propagation engine.
    pub stored_prop_engine: Box<PropagationEngine>,
    /// Temporary storage of the main elaborator constraint ID.
    pub stored_elaborator_id: usize,
    /// Temporary storage of the current active order.
    pub stored_active_order: Box<Option<ActiveOrder>>,
}

impl OrderContext {
    /// Construct a new [`OrderContext`].
    #[inline]
    pub fn new(
        name: String,
        database: Database,
        prop_engine: PropagationEngine,
        active_order: Option<ActiveOrder>,
    ) -> Self {
        OrderContext {
            name,
            stored_database: Box::new(database),
            stored_prop_engine: Box::new(prop_engine),
            stored_active_order: Box::new(active_order),
            ..Default::default()
        }
    }

    /// Get the substitution for the reflexivity proofgoals.
    #[inline]
    pub fn get_reflexivity_substitution(&self) -> Substitution {
        let mut substitution = Substitution::with_size(self.constructed_order.left_vars.len());
        for (&left, &right) in izip!(
            self.constructed_order.left_vars.iter(),
            self.constructed_order.right_vars.iter(),
        ) {
            substitution.set(right, SubstitutionValue::lit(Lit::from_var(left, false)));
        }
        substitution
    }

    /// Get the substitutions for the transitivity proofgoals.
    ///
    /// Returns a tuple of the 2 substitutions that are non-trivial. The first substitution maps from left variables to right variables and right variables to the fresh right variables (required for premise). The second substitution maps from left variables to left variables and right variables to the fresh right variables (required for proofgoal).
    ///
    /// So speaking in terms of x, y, z and x R y R z, then transitivity means that x R z. So the tuple is the substitutions for (y R z, x R z).
    #[inline]
    pub fn get_transitivity_substitution(
        &self,
        transitivity: &TransitivityContext,
    ) -> (Substitution, Substitution) {
        let mut substitution_y_z = Substitution::with_size(
            2 * self.constructed_order.left_vars.len() + self.constructed_order.aux_vars.len(),
        );
        let mut substitution_x_z = Substitution::with_size(
            self.constructed_order.left_vars.len() + self.constructed_order.aux_vars.len(),
        );
        for (&left, &right, &fresh) in izip!(
            self.constructed_order.left_vars.iter(),
            self.constructed_order.right_vars.iter(),
            transitivity.fresh_right.iter(),
        ) {
            substitution_y_z.set(right, SubstitutionValue::lit(Lit::from_var(fresh, false)));
            substitution_y_z.set(left, SubstitutionValue::lit(Lit::from_var(right, false)));
            substitution_x_z.set(right, SubstitutionValue::lit(Lit::from_var(fresh, false)));
        }
        for (&aux, &fresh_aux_1, &fresh_aux_2) in izip!(
            self.constructed_order.aux_vars.iter(),
            transitivity.fresh_aux_1.iter(),
            transitivity.fresh_aux_2.iter()
        ) {
            substitution_y_z.set(
                aux,
                SubstitutionValue::lit(Lit::from_var(fresh_aux_1, false)),
            );
            substitution_x_z.set(
                aux,
                SubstitutionValue::lit(Lit::from_var(fresh_aux_2, false)),
            );
        }

        (substitution_y_z, substitution_x_z)
    }

    /// Try to autoprove the reflexivity of the order.
    ///
    /// Currently, autoproving of reflexivity only checks if all proofgoals are trivial.
    pub fn autoprove_reflexivity(&self, elaborator: &mut Option<Elaborator>) -> bool {
        let reflexivity_substitution = self.get_reflexivity_substitution();
        for constraint in self.constructed_order.definition.iter() {
            if !constraint
                .substitute(&reflexivity_substitution)
                .is_trivial()
            {
                return false;
            }
        }
        if let Some(elaborator) = elaborator {
            elaborator.write_trivial_reflexivity_subproof(self.constructed_order.definition.len());
        }
        true
    }

    /// Try to autoprove the transitivity of the order.
    ///
    /// Currently, autoproving of transitivity is not implemented.
    pub fn autoprove_transitivity(&self) -> bool {
        todo!()
    }
}

/// Context for the transitivity proof subcontext.
#[derive(Debug, Default)]
pub struct TransitivityContext {
    pub inside_vars: bool,
    pub is_proven: bool,
    pub fresh_right: Vec<VarIdx>,
    pub fresh_aux_1: Vec<VarIdx>,
    pub fresh_aux_2: Vec<VarIdx>,
}

/// Context for the transitivity proof subcontext.
#[derive(Debug, Default)]
pub struct ReflexivityContext {
    pub is_proven: bool,
}

/// Context for the specification derivation inside the `spec` environment of the order.
#[derive(Debug, Default)]
pub struct SpecificationContext {
    /// Temporary storage of the database for the order definition.
    pub stored_database: Box<Database>,
    /// Temporary storage of the propagation engine for the order definition.
    pub stored_prop_engine: Box<PropagationEngine>,
    /// Temporary storage of the current elaborator constraint ID for the order definition.
    pub stored_elaborator_id: usize,
}

impl SpecificationContext {
    /// Construct a new [`SpecificationContext`] for a give `database` and `prop_engine`.
    #[inline]
    pub fn new(database: Database, prop_engine: PropagationEngine) -> SpecificationContext {
        SpecificationContext {
            stored_database: Box::new(database),
            stored_prop_engine: Box::new(prop_engine),
            ..Default::default()
        }
    }
}
