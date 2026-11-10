//! Storage of pseudo-Boolean constraints in the [`ForwardsChecker`](crate::checker::ForwardsChecker).

use std::{cell::RefCell, ops::Range, rc::Rc};

use ahash::{HashSet, RandomState};
use by_address::ByAddress;
use veripb_formula::prelude::*;
use veripb_propagator::propagation_engine::{
    CORE_CLAUSES_CARD, CORE_GENERAL, DERIVED_CLAUSES_CARD, DERIVED_GENERAL, PropagationEngine,
};
use veripb_rules::elaboration::ConstraintLookup;

use crate::{context::Context, error::CheckingError, occurrence_list::OccurrenceList};

const IGNORE_DATABASE_PROOFGOAL_THRESHOLD: f64 = 0.05;

/// Remove the constraint from the core propagator and add it to the derived propagator.
#[inline]
pub fn move_to_derived_propagator(
    prop_engine: &mut PropagationEngine,
    constraint: &Rc<DBConstraint>,
) -> Result<(), CheckingError> {
    prop_engine.detach_checker(true, constraint, prop_engine.only_core_trail)?;
    prop_engine.attach_checker(false, constraint)?;
    Ok(())
}

/// Storage for constraints.
#[derive(Debug, Default)]
pub struct Database {
    /// Constraint ID indexed vector to directly get a constraint from its index.
    pub entries: Vec<Option<Rc<DBConstraint>>>,
    /// Hash map of unique constraints identified by the constraint of the entry.
    pub unique_constraints: HashSet<Rc<DBConstraint>>,
    /// An occurrence list mapping from literals to constraints. This list is kept in sync with the `unique_constraints`.
    occurrences: OccurrenceList,
    /// Only constraint IDs that are less than `next_unique_index_id` have been added to `unique_constraints`.
    next_unique_index_id: usize,
    /// Only constraint IDs that are less than `next_occurrences_index_id` have been added to `occurrences`.
    next_occurrences_index_id: usize,
    /// Only constraint IDs that are less than `next_propagation_index_id` have been added to the propagation engine.
    pub(crate) next_propagation_index_id: usize,
    /// Only constraint IDs that are at least `first_derived_id_lower_bound` may be in the derived set.
    first_derived_id_lower_bound: usize,
    /// The maximum size of the database to elaborate all database proofgoals even if they are autoproven.
    ignore_database_proofgoal_size: usize,
}

impl Database {
    /// Construct a new empty database with dummy entry at position 0.
    pub fn new() -> Self {
        Database {
            entries: vec![None],
            ..Default::default()
        }
    }

    /// Initialize a database from the a `Formula`.
    pub fn from_formula(constraints: Vec<PBConstraintEnum>) -> Self {
        let mut entries = Vec::with_capacity(constraints.len() + 1);
        entries.push(None);
        let unique_constraints = HashSet::with_capacity_and_hasher(
            constraints.len(),
            RandomState::with_seeds(42, 42, 42, 42),
        );
        let occurrences = OccurrenceList::default();
        for constraint in constraints {
            let db_constraint = Rc::new(DBConstraint {
                header: RefCell::new(DBHeader::default()),
                constraint,
            });
            db_constraint.add_id(entries.len(), true);
            entries.push(Some(db_constraint));
        }
        let first_non_indexed_constraint_id = entries.len();
        let ignore_database_proofgoal_size =
            (IGNORE_DATABASE_PROOFGOAL_THRESHOLD * (entries.len() as f64)) as usize;

        Database {
            entries,
            unique_constraints,
            occurrences,
            next_unique_index_id: 1,
            next_occurrences_index_id: 1,
            next_propagation_index_id: 1,
            first_derived_id_lower_bound: first_non_indexed_constraint_id,
            ignore_database_proofgoal_size,
        }
    }

    /// Lazily adds a constraint to the database.
    ///
    /// If the constraint is already in the database, then it will be added lazily again.
    #[inline]
    pub fn add_constraint(&mut self, constraint: Rc<DBConstraint>, add_to_core: bool) {
        constraint.add_id(self.len(), add_to_core);
        self.entries.push(Some(constraint));
    }

    /// Remove a constraint from the unique-constraint and occurrences indices if it has
    /// been added to them.
    pub fn deindex_removed_constraint(&mut self, constraint: &Rc<DBConstraint>) {
        let mut header = constraint.header.borrow_mut();
        if header.is_in_unique {
            self.unique_constraints.remove(constraint.as_ref());
            header.is_in_unique = false;
        }

        if header.is_in_occurrences {
            self.occurrences.remove(constraint);
            header.is_in_occurrences = false;
        }
    }

    /// Re-index a constraint into whichever indices are already caught up to `len`
    /// (an index still behind will pick it up on its next `update_*`). Inverse of
    /// [`Database::deindex_removed_constraint`], will be used for the trimmer.
    pub fn reindex_reinserted_constraint(
        &mut self,
        constraint: &Rc<DBConstraint>,
        prop_engine: &mut PropagationEngine,
    ) -> Result<(), CheckingError> {
        let len = self.len();
        let mut header = constraint.header.borrow_mut();
        if self.next_unique_index_id == len
            && !header.is_in_unique
            && self.unique_constraints.insert(Rc::clone(constraint))
        {
            header.is_in_unique = true;
        }
        if self.next_occurrences_index_id == len && !header.is_in_occurrences {
            self.occurrences.add(constraint);
        }
        if self.next_propagation_index_id == len && header.attached_propagator_count == 0 {
            let add_to_core = constraint.is_core_constraint();
            prop_engine.attach_checker(add_to_core, constraint)?;
        }
        Ok(())
    }

    /// Check that the given id can be removed from the Database and `.take()` the entry if so.
    pub fn take_deletable_entry(&mut self, id: usize) -> Result<Rc<DBConstraint>, CheckingError> {
        match self.entries.get_mut(id) {
            None => Err(CheckingError::deletion(
                id,
                &format!(
                    "there are only {} constraints in the database",
                    self.entries.len()
                ),
            )),
            Some(None) => Err(CheckingError::deletion(
                id,
                "the constraint has already been deleted",
            )),
            Some(entry) => Ok(entry.take().expect("already checked None")),
        }
    }

    /// Delete a constraint from the database by their ID.
    ///
    /// Also deletes the constraint from indexing structures if it was added to them.
    #[inline]
    pub fn delete_constraint(
        &mut self,
        context: &mut Context,
        constraint_id: usize,
    ) -> Result<(), CheckingError> {
        let db_constraint = self.take_deletable_entry(constraint_id)?;
        let was_core = db_constraint.is_core_constraint();
        db_constraint.remove_id(constraint_id);
        if db_constraint.all_constraint_ids_empty(false) {
            self.deindex_removed_constraint(&db_constraint);
            context
                .propagation_engine
                .detach_checker(was_core, &db_constraint, true)?;
        } else if was_core && !db_constraint.is_core_constraint() {
            move_to_derived_propagator(&mut context.propagation_engine, &db_constraint)?;
        }
        Ok(())
    }
    /// Indexes all constraints that are not yet indexed in `unique_constraints`.
    ///
    /// In case a duplicate constraint is detected, then the duplicate constraint is merged into the first occurrence.
    /// In particular, this also removes duplicate constraints from `occurrences` and the propagation engine.
    ///
    /// Must be called before:
    /// - using `Database::unique_constraints` directly
    /// - calling `Database::lookup()`
    /// - calling `Database::get_proofgoals()`
    pub fn update_unique_index(
        &mut self,
        prop_engine: &mut PropagationEngine,
    ) -> Result<(), CheckingError> {
        for constraint_id in self.next_unique_index_id..self.len() {
            // First check that the constraint ID is not deleted.
            if let Some(Some(constraint)) = self.entries.get(constraint_id) {
                // Skip a constraint with no live id (needed for
                // trimming, where survivor constraints persist in the
                // database even after being removed).
                if constraint.all_constraint_ids_empty(false) {
                    continue;
                }
                let is_core_constraint = constraint.is_core_constraint();
                if self.unique_constraints.insert(constraint.clone()) {
                    constraint.header.borrow_mut().is_in_unique = true;
                } else {
                    // Merge this constraint with the existing constraint.
                    let entry = self.unique_constraints.get(constraint).unwrap();
                    let is_original_core_constraint = entry.is_core_constraint();
                    entry.add_id(constraint_id, is_core_constraint);
                    if let Some(out_id) = constraint.get_out_id(constraint_id) {
                        entry.set_out_id(constraint_id, out_id);
                    }
                    constraint.remove_id(constraint_id);
                    // Remove from occurrence list if the duplicate constraint was added.
                    {
                        let mut header = constraint.header.borrow_mut();
                        if header.is_in_occurrences {
                            self.occurrences.remove(constraint);
                            header.is_in_occurrences = false;
                        }
                    }

                    // Remove from propagator if it was added.
                    if constraint_id < self.next_propagation_index_id {
                        prop_engine.detach_checker(is_core_constraint, constraint, true)?;
                    }
                    // In case the original constraint is only in the derived set
                    // and the new is in the core set, then move it to the core.
                    let is_original_in_propagator =
                        entry.header.borrow().attached_propagator_count > 0;
                    if is_original_in_propagator
                        && !is_original_core_constraint
                        && is_core_constraint
                    {
                        prop_engine.move_constraint_checker(false, true, entry)?;
                    }
                    // Overwrite the duplicate constraint such that they both IDs point to the same constraint.
                    self.entries[constraint_id] = Some(Rc::clone(entry));
                }
            }
        }
        self.next_unique_index_id = self.len();
        Ok(())
    }

    /// Indexes all constraints that are not yet indexed in `occurrences`. Will be automatically called before occurrences are needed.
    pub fn update_occurrences_index(&mut self) {
        for constraint_id in self.next_occurrences_index_id..self.len() {
            // First check that the constraint ID is not deleted.
            if let Some(Some(constraint)) = self.entries.get(constraint_id) {
                // Skip a constraint with no live id (needed for trimming).
                if constraint.all_constraint_ids_empty(false) {
                    continue;
                }
                if !constraint.header.borrow().is_in_occurrences {
                    self.occurrences.add(constraint);
                }
            }
        }
        self.next_occurrences_index_id = self.len();
    }

    /// Indexes all constraints that are not yet added to the propagation engine.
    ///
    /// Must be called before:
    /// - calling `PropagationEngine::reverse_unit_propagation_check()`
    /// - calling `PropagationEngine::propagate_solution()`
    pub fn update_propagation_index(
        &mut self,
        prop_engine: &mut PropagationEngine,
    ) -> Result<(), CheckingError> {
        for constraint_id in self.next_propagation_index_id..self.len() {
            // First check that the constraint ID is not deleted.
            if let Some(Some(constraint)) = self.entries.get(constraint_id) {
                // Skip a constraint with no live id (needed for trimming).
                let attach = {
                    let header = constraint.header.borrow();
                    header.attached_propagator_count == 0
                        && !(header.core_ids.is_empty() && header.derived_ids.is_empty())
                };
                if attach {
                    let add_to_core = constraint.is_core_constraint();
                    prop_engine.attach_checker(add_to_core, constraint)?;
                }
            }
        }
        self.next_propagation_index_id = self.len();
        Ok(())
    }

    /// Check if `constraint` is contained in the `Database` and return a reference to the constraint in the database if it exists.
    #[inline]
    pub fn lookup(&self, constraint: &Rc<DBConstraint>) -> Option<&Rc<DBConstraint>> {
        debug_assert!(self.len() == self.next_unique_index_id);
        self.unique_constraints.get(constraint)
    }

    /// Check if the database contain a contradiction.
    #[inline]
    pub fn contains_contradiction(&self) -> Option<usize> {
        for constraint in self.unique_constraints.iter() {
            if constraint.is_contradicting() {
                return Some(constraint.get_some_id());
            }
        }
        for constraint_id in self.next_unique_index_id..self.len() {
            if let Some(Some(constraint)) = self.entries.get(constraint_id)
                && constraint.is_contradicting()
            {
                return Some(constraint_id);
            }
        }
        None
    }

    /// Normalize a constraint ID using the current number of constraints into an absolute constraint ID.
    #[inline]
    pub fn normalize_id(&self, index: isize) -> isize {
        if index < 0 {
            self.len() as isize + index
        } else {
            index
        }
    }

    /// Get the constraint at a specific absolute or relative `index` in the database.
    ///
    /// This function returns an error if there is no constraint stored at the `index`.
    #[inline]
    pub fn get_entry(&self, index: isize) -> Result<&Rc<DBConstraint>, CheckingError> {
        let entry = if index < 0 {
            match self
                .entries
                .get(((self.entries.len() as isize) + index) as usize)
            {
                Some(entry) => entry,
                None => {
                    return Err(CheckingError::out_of_bounds(
                        index,
                        -(self.entries.len() as isize) + 1,
                        (self.entries.len() as isize) - 1,
                    ));
                }
            }
        } else {
            match self.entries.get(index as usize) {
                Some(entry) => entry,
                None => {
                    return Err(CheckingError::out_of_bounds(
                        index,
                        -(self.entries.len() as isize),
                        (self.entries.len() as isize) - 1,
                    ));
                }
            }
        };

        match entry {
            Some(constraint) => Ok(constraint),
            None => Err(CheckingError::access_deleted(index)),
        }
    }

    /// Get the constraint at a specific absolute `index` in the database.
    ///
    /// This function returns an error if there is no constraint stored at the `index`.
    #[inline]
    pub fn get_entry_usize(&self, index: usize) -> Result<&Rc<DBConstraint>, CheckingError> {
        let entry = match self.entries.get(index) {
            Some(entry) => entry,
            None => {
                return Err(CheckingError::out_of_bounds(
                    index as isize,
                    -(self.entries.len() as isize),
                    (self.entries.len() as isize) - 1,
                ));
            }
        };

        match entry {
            Some(constraint) => Ok(constraint),
            None => Err(CheckingError::access_deleted(index as isize)),
        }
    }

    /// Get the constraint at a specific absolute `index` in the database.
    ///
    /// This functions returns [`None`] if the constraint has already been deleted.
    #[inline]
    pub fn get_entry_optionally_deleted_usize(
        &self,
        index: usize,
    ) -> Result<Option<&Rc<DBConstraint>>, CheckingError> {
        let entry = match self.entries.get(index) {
            Some(entry) => entry,
            None => {
                return Err(CheckingError::out_of_bounds(
                    index as isize,
                    -(self.entries.len() as isize),
                    (self.entries.len() as isize) - 1,
                ));
            }
        };

        match entry {
            Some(constraint) => Ok(Some(constraint)),
            None => Ok(None),
        }
    }

    /// Check if the constraint at the `index` is not deleted.
    #[inline]
    pub fn is_undeleted(&self, index: usize) -> Result<bool, CheckingError> {
        if let Some(entry) = self.entries.get(index) {
            return Ok(entry.is_some());
        }
        Err(CheckingError::out_of_bounds(
            index as isize,
            -(self.entries.len() as isize),
            (self.entries.len() as isize) - 1,
        ))
    }

    /// Move the constraint ID to the core set and add it to the core propagator if it is not already in the core.
    #[inline]
    pub fn move_to_core(
        &self,
        prop_engine: &mut PropagationEngine,
        index: usize,
    ) -> Result<(), CheckingError> {
        // Find the constraint and get its header.
        let constraint = self.get_entry_usize(index)?;

        // If the constraint was not in the core propagator before, it is moved there.
        if !constraint.is_core_constraint()
            && constraint.header.borrow().attached_propagator_count > 0
        {
            prop_engine.move_constraint_checker(false, true, constraint)?;
        }

        // Remove ID from derived set and add it to the core set of IDs.
        constraint.move_id_to_core(index);

        Ok(())
    }

    /// Move all constraints to the core set and add them to the core propagator.
    pub fn move_to_core_all(
        &mut self,
        prop_engine: &mut PropagationEngine,
    ) -> Result<(), CheckingError> {
        // Move all constraints in database to core set.

        for constraint_id in self.first_derived_id_lower_bound..self.len() {
            if let Some(Some(constraint)) = self.entries.get(constraint_id) {
                let header = &mut *constraint.header.borrow_mut();
                header.core_ids.append(&mut header.derived_ids);
            }
        }
        self.first_derived_id_lower_bound = self.len();

        // Move all constraints in the derived propagation set to the core propagation set.
        prop_engine.move_all_constraints(DERIVED_CLAUSES_CARD, CORE_CLAUSES_CARD)?;
        prop_engine.move_all_constraints(DERIVED_GENERAL, CORE_GENERAL)?;

        Ok(())
    }

    /// Check if all constraints in the database are in the core set. No constraint is moved by this operation.
    pub fn all_constraints_in_core(&mut self) -> bool {
        // We make use of the fact that constraints can only be moved from the derived to the core set. So once we checked if a constraint is in the core set, then it will stay there or it will be deleted.
        for constraint in self.entries[self.first_derived_id_lower_bound..]
            .iter()
            .flatten()
        {
            if !constraint.header.borrow().derived_ids.is_empty() {
                return false;
            }
        }
        // Store last constraint ID as start point for next check.
        self.first_derived_id_lower_bound = self.len();

        true
    }

    /// Get the variables actually used by the constraints in the database.
    #[inline]
    pub fn get_used_vars(&mut self) -> Vec<VarIdx> {
        self.update_occurrences_index();
        self.occurrences.get_used_vars()
    }

    /// Add constraints to `unique_substituted_constraints` that contain the literal `lit`.
    ///
    /// This function only adds constraints that are not obviously implied by the database already.
    #[inline]
    fn add_non_obvious_proofgoals(
        &self,
        substitution: &Substitution,
        unique_touched_constraint: HashSet<&ByAddress<Rc<DBConstraint>>>,
        add_derived_goals: bool,
        exclude_database_autoproving: bool,
        only_core_subproof: bool,
    ) -> HashSet<Rc<DBConstraint>> {
        let mut unique_substituted_constraints =
            HashSet::with_hasher(RandomState::with_seeds(42, 42, 42, 42));
        for constraint in unique_touched_constraint {
            if add_derived_goals || constraint.is_core_constraint() {
                let substituted_constraint = Rc::new(constraint.substitute(substitution));
                // The following conditions have to consider a proofgoal non-obvious:
                // 1. The proofgoal is not trivial (... >= 1)
                // 2. The original constraint does not imply the proofgoal.
                // 3. No other database constraint is syntactially equivalent to the proofgoal. This is ignored, if:
                //     1. `exclude_database_autoproving` is true.
                //     2. We can only use core constraints for the subrpoof and the database constraint is not in the core set.
                if !substituted_constraint.is_trivial()
                    && !constraint.implies(&substituted_constraint)
                    && (exclude_database_autoproving
                        || self
                            .lookup(&substituted_constraint)
                            .is_none_or(|c| only_core_subproof && !c.is_core_constraint()))
                {
                    // Copy ids from the original constraint.
                    substituted_constraint.copy_ids(constraint);
                    // Check if constraint is already contained in unique substituted constraints.
                    if let Some(prev_entry) =
                        unique_substituted_constraints.replace(substituted_constraint)
                    {
                        let entry = unique_substituted_constraints.get(&prev_entry).unwrap();
                        entry.append_ids(Rc::into_inner(prev_entry).unwrap());
                    }
                }
            }
        }
        unique_substituted_constraints
    }

    /// Get the proofgoals from the formula for strengthening rules with respect to the substitution.
    ///
    /// Proofgoals that are trivial, implied by the original constraint, or that can be found in the database are not added as proofgoals.
    #[inline]
    pub fn get_proofgoals(
        &mut self,
        substitution: &Substitution,
        add_derived_goals: bool,
        exclude_database_autoproving: bool,
        only_core_subproof: bool,
    ) -> HashSet<Rc<DBConstraint>> {
        debug_assert_eq!(self.len(), self.next_unique_index_id);

        self.update_occurrences_index();
        let mut unique_touched_constraints =
            HashSet::with_hasher(RandomState::with_seeds(42, 42, 42, 42));
        for &var_idx in substitution.syntactic_support() {
            // Constraints where the literal is mapped to true do not need to be considered as a proofgoal.
            match substitution.get(var_idx).unwrap() {
                SubstitutionValue::TRUE => {
                    if let Some(occurrences) = self
                        .occurrences
                        .get_constraints_for_lit(Lit::from_var(var_idx, true))
                    {
                        unique_touched_constraints.extend(occurrences.iter());
                    }
                }
                SubstitutionValue::FALSE => {
                    if let Some(occurrences) = self
                        .occurrences
                        .get_constraints_for_lit(Lit::from_var(var_idx, false))
                    {
                        unique_touched_constraints.extend(occurrences.iter());
                    }
                }
                _ => {
                    if let Some(occurrences) = self
                        .occurrences
                        .get_constraints_for_lit(Lit::from_var(var_idx, false))
                    {
                        unique_touched_constraints.extend(occurrences.iter());
                    }
                    if let Some(occurrences) = self
                        .occurrences
                        .get_constraints_for_lit(Lit::from_var(var_idx, true))
                    {
                        unique_touched_constraints.extend(occurrences.iter());
                    }
                }
            }
        }

        let exclude_database_autoproving = exclude_database_autoproving
            && unique_touched_constraints.len() < self.ignore_database_proofgoal_size;

        self.add_non_obvious_proofgoals(
            substitution,
            unique_touched_constraints,
            add_derived_goals,
            exclude_database_autoproving,
            only_core_subproof,
        )
    }

    /// Get an iterator over the constraint IDs for the constraints that have not been deleted in the given `range` in the order of the constraint IDs.
    #[inline]
    pub fn get_undeleted(
        &self,
        range: Range<usize>,
    ) -> impl '_ + Iterator<Item = Result<usize, CheckingError>> {
        range
            .into_iter()
            .filter_map(|id| match self.is_undeleted(id) {
                Ok(true) => Some(Ok(id)),
                Ok(false) => None,
                Err(err) => Some(Err(err)),
            })
    }

    /// Get an iterator over the constraint entries in the database that have not been deleted starting from the first constraint that has not been added to the unique constraints. The constraints are in order of the constraint ID.
    #[inline]
    pub fn get_undeleted_non_unique_indexed(&self) -> impl '_ + Iterator<Item = &Rc<DBConstraint>> {
        self.entries[self.next_unique_index_id..self.len()]
            .iter()
            .flatten()
    }

    /// The length of the database is the number of entries including deleted constraint and duplicates.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The ID of the most recently added constraint.
    pub fn last_constraint_id(&self) -> isize {
        std::cmp::max(self.entries.len(), 1) as isize - 1
    }

    /// Returns `true` if there are no entries in the database including deleted constraints.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The size of the database is the number of non-deleted (`Some(constraint)`) entries in the database.
    #[inline]
    pub fn size(&self) -> usize {
        self.entries.iter().flatten().count()
    }
}

impl ConstraintLookup for Database {
    type Error = CheckingError;

    fn get_entry_usize(&self, index: usize) -> Result<&Rc<DBConstraint>, Self::Error> {
        self.get_entry_usize(index)
    }
}
