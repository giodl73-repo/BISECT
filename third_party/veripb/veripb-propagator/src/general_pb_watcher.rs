//! Watcher for [`GeneralPBConstraint`] constraints.

use std::{cell::RefCell, rc::Rc};

use ahash::{HashMap, HashMapExt};
use veripb_formula::prelude::*;

use crate::{
    trail::{Propagation, Reason},
    watcher::{WatchInit, WatchUpdate, Watchable},
};

/// [`Watchable`] for general pseudo-Boolean constraints.
///
/// The basic idea for watching general PB constraints is that the sum of the coefficients of the watched terms is at least the right-hand side + largest coefficient to detect all propagations. The idea of the watcher is that during one propagation, we only need to look at each literal at most twice, once for checking if it could be a watch and once for checking if it proapgates. This is achieved by storing where we stopped iterating for during the watch update.
///
/// # Invariants
/// In addition to the invariants of [`Watchable`], there are some further invariants maintained by [`GeneralPBWatcher`].
/// 1. All literals in `sorted_terms` with a smaller position than `next_pos` are watched or falsifed.
/// 2. All literals in `sorted_terms` with a smaller position than `next_pos_prop` are falsifed or have been returned by [`Watchable::update_watch()`] as `propagated` in since the [`Watchable::reset_watch()`] .
#[derive(Debug)]
pub struct GeneralPBWatcher<N: Int> {
    /// Reference to the constraint that is watched.
    constraint: Rc<DBConstraint>,
    /// All terms of the constraint sorted by coefficient in descending order.
    sorted_terms: Vec<GeneralPBTerm<N>>,
    /// Mapping from watched literals which are contributing the current slack to their coefficient.
    pub watches: HashMap<Lit, N>,
    /// List of watched literals (paired with their coefficient) which are not contributing to the current slack.
    pub watched_but_not_contributing: Vec<(Lit, N)>,
    /// Upper bound on the current slack of the watched literals. This is only an upper bound since falsified literals might be watched and we have not updated the slack for them.
    pub watches_slack: N,
    /// Largest coefficient of the constraint.
    pub max_coeff: N,
    /// The start position in the `sorted_terms` vector where to look next for new watches.
    pub next_pos: usize,
    /// The start position in the `sorted_terms` vector where to look next for propagated literals.
    next_pos_prop: usize,
    /// Cache for detecting if constraint is propagating all its literals independent of an assignment.
    is_fully_propagated_cached: Option<bool>,
    /// If [`true`], then some watches `watched_but_not_contributing` might now unassigned or assigned to true. So they are contributing to the slack of the constraint.
    is_reset_not_contributing: bool,
}

impl<N: Int> GeneralPBWatcher<N>
where
    PBConstraintEnum: From<GeneralPBConstraint<N>>,
{
    /// Construct a watcher for the general pseudo-Boolean constraint `constraint`.
    pub fn from_constraint(constraint: GeneralPBConstraint<N>) -> Self {
        let mut sorted_terms = constraint.get_terms().to_owned();
        sorted_terms.sort_unstable_by(|term1, term2| term2.get_coeff().cmp(term1.get_coeff()));
        let max_coeff = if let Some(term) = sorted_terms.first() {
            term.get_coeff().clone()
        } else {
            N::zero()
        };
        let slack_init = -constraint.get_degree().clone();
        GeneralPBWatcher {
            constraint: Rc::new(DBConstraint {
                header: RefCell::new(DBHeader::default()),
                constraint: constraint.into(),
            }),
            sorted_terms,
            watches: HashMap::new(),
            watched_but_not_contributing: Vec::new(),
            watches_slack: slack_init.clone(),
            max_coeff,
            next_pos: 0,
            next_pos_prop: 0,
            is_fully_propagated_cached: None,
            is_reset_not_contributing: true,
        }
    }
}

impl<N: Int> GeneralPBWatcher<N> {}

impl<N: Int> Watchable for GeneralPBWatcher<N>
where
    PBConstraintEnum: From<GeneralPBConstraint<N>>,
{
    #[inline]
    fn from_db_constraint(constraint: &Rc<DBConstraint>) -> Self {
        let pb_constraint = constraint.constraint.as_general_pb::<N>();
        let mut sorted_terms = pb_constraint.get_terms().to_owned();
        sorted_terms.sort_unstable_by(|term1, term2| term2.get_coeff().cmp(term1.get_coeff()));
        if let Some(max_term) = pb_constraint
            .get_terms()
            .iter()
            .max_by(|term1, term2| term1.get_coeff().cmp(term2.get_coeff()))
        {
            GeneralPBWatcher {
                constraint: constraint.clone(),
                sorted_terms,
                watches: HashMap::new(),
                watched_but_not_contributing: Vec::new(),
                watches_slack: -pb_constraint.get_degree().clone(),
                max_coeff: max_term.get_coeff().to_owned(),
                next_pos: 0,
                next_pos_prop: 0,
                is_fully_propagated_cached: None,
                is_reset_not_contributing: true,
            }
        } else {
            GeneralPBWatcher {
                constraint: constraint.clone(),
                sorted_terms,
                watches: HashMap::new(),
                watched_but_not_contributing: Vec::new(),
                watches_slack: -pb_constraint.get_degree().clone(),
                max_coeff: N::zero(),
                next_pos: 0,
                next_pos_prop: 0,
                is_fully_propagated_cached: None,
                is_reset_not_contributing: true,
            }
        }
    }
    #[inline]
    fn get_assignment_independent_propagations(&self) -> Option<Vec<Propagation>> {
        let pb_constraint = self.constraint.constraint.as_general_pb::<N>();

        let slack = pb_constraint.get_coeff_sum() - pb_constraint.get_degree();
        let mut propagations = Vec::new();
        for term in self.sorted_terms.iter() {
            if term.get_coeff() > &slack {
                propagations.push(Propagation::new(
                    term.get_lit(),
                    Reason::Constraint(self.constraint.clone()),
                ));
            } else {
                // terms are sorted by coefficient in decreasing order, so no further propagations can occur
                break;
            }
        }

        if propagations.is_empty() {
            None
        } else {
            Some(propagations)
        }
    }

    #[inline]
    fn no_watches_required(&mut self) -> bool {
        if let Some(cached_result) = self.is_fully_propagated_cached {
            return cached_result;
        }
        let pb_constraint = self.constraint.constraint.as_general_pb::<N>();

        let slack = pb_constraint.get_coeff_sum() - pb_constraint.get_degree();
        if !slack.is_positive() || self.constraint.is_trivial() {
            self.is_fully_propagated_cached = Some(true);
            return true;
        }
        // If the minimum coefficient is larger than the slack, then all literals propagate.
        // Otherwise, at least the literals with the minimum coefficient do not propagate.
        // Note: all cases where the constraint has no terms are already covered, so there is at least one term.
        if self.sorted_terms.last().unwrap().get_coeff() <= &slack {
            self.is_fully_propagated_cached = Some(false);
            return false;
        }

        self.is_fully_propagated_cached = Some(true);
        true
    }

    #[inline]
    fn is_propagating_independent_of_assignment(&self) -> bool {
        let pb_constraint = self.constraint.constraint.as_general_pb::<N>();

        let slack = pb_constraint.get_coeff_sum() - pb_constraint.get_degree();
        if self.max_coeff > slack {
            return true;
        }

        false
    }

    #[inline]
    fn init_watches(&mut self, assign: &Assignment<BooleanVar>) -> WatchInit {
        if !self.watches.is_empty() {
            // Watches already defined.
            return WatchInit::Watching {
                propagated: vec![],
                watches: vec![],
            };
        }
        let pb_constraint = self.constraint.constraint.as_general_pb::<N>();
        if pb_constraint.is_trivial() {
            return WatchInit::Watching {
                propagated: vec![],
                watches: vec![],
            };
        }

        for (pos, term) in self.sorted_terms.iter().enumerate() {
            let value = unsafe { assign.get_lit_value_unchecked(term.get_lit()) };
            match value {
                BoolValue::Unassigned | BoolValue::Assigned(true) => {
                    self.watches.insert(term.get_lit(), term.coeff.clone());
                    self.watches_slack += term.get_coeff();
                    if self.watches_slack >= self.max_coeff {
                        self.next_pos = pos + 1;
                        return WatchInit::Watching {
                            propagated: vec![],
                            watches: self.watches.iter().map(|(&l, _)| l).collect(),
                        };
                    }
                }
                BoolValue::Assigned(false) => {}
            }
        }

        // If slack is negative, then constraint is conflicting.
        if self.watches_slack.is_negative() {
            self.watches.clear();
            self.watches_slack = -pb_constraint.get_degree().clone();
            return WatchInit::Conflict;
        }

        // Check which literals are propagated.
        let mut propagated = Vec::new();
        for term in self.sorted_terms[self.next_pos_prop..].iter() {
            self.next_pos_prop += 1;
            let lit = term.get_lit();
            if unsafe { assign.is_unassigned_unchecked(lit) } {
                if term.get_coeff() > &self.watches_slack {
                    propagated.push(lit);
                } else {
                    self.next_pos_prop -= 1;
                    break;
                }
            }
        }

        let mut watches: Vec<_> = self.watches.iter().map(|(&l, _)| l).collect();

        // Find enough literals to watch if we reset the propagator.
        let mut temp_slack = self.watches_slack.clone();
        for term in self.sorted_terms.iter() {
            let value = unsafe { assign.get_lit_value_unchecked(term.get_lit()) };
            match value {
                BoolValue::Unassigned | BoolValue::Assigned(true) => {}
                BoolValue::Assigned(false) => {
                    self.watched_but_not_contributing
                        .push((term.get_lit(), term.get_coeff().clone()));
                    watches.push(term.get_lit());
                    temp_slack += term.get_coeff();
                    if temp_slack >= self.max_coeff {
                        break;
                    }
                }
            }
        }

        WatchInit::Watching {
            propagated,
            watches,
        }
    }

    #[inline]
    fn update_watch(&mut self, old_watch: Lit, assign: &Assignment<BooleanVar>) -> WatchUpdate {
        if !self.is_reset_not_contributing {
            self.reset_not_contributing(assign);
        }

        let pb_constraint = self.constraint.constraint.as_general_pb::<N>();

        let original_slack = self.watches_slack.clone();

        // Remove old watch.
        let coeff_old_watch;
        if let Some(coeff) = self.watches.remove(&old_watch) {
            coeff_old_watch = coeff;
            self.watches_slack -= &coeff_old_watch;
        } else {
            let index = self
                .watched_but_not_contributing
                .iter()
                .position(|&(lit, _)| lit == old_watch)
                .expect("literal should be watched");
            (_, coeff_old_watch) = self.watched_but_not_contributing.swap_remove(index);
        }

        // Maybe we still watch enough literals.
        if self.watches_slack >= self.max_coeff {
            return WatchUpdate::None;
        }

        // Find new watches.
        let mut added_watch_coeff_pairs = Vec::new();
        for term in self.sorted_terms[self.next_pos..].iter() {
            self.next_pos += 1;
            let lit = term.get_lit();
            let value = unsafe { assign.get_lit_value_unchecked(lit) };
            match value {
                BoolValue::Unassigned | BoolValue::Assigned(true) => {
                    if !self.watches.contains_key(&lit) {
                        added_watch_coeff_pairs.push((term.lit, term.coeff.clone()));
                        self.watches_slack += term.get_coeff().to_owned();
                        if self.watches_slack >= self.max_coeff {
                            let added_watches =
                                added_watch_coeff_pairs.iter().map(|&(l, _)| l).collect();
                            self.watches.extend(added_watch_coeff_pairs);
                            return WatchUpdate::Unknown {
                                propagated: vec![],
                                new_watches: added_watches,
                            };
                        }
                    }
                }
                BoolValue::Assigned(false) => {}
            }
        }
        self.next_pos = pb_constraint.len();

        // Detect conflict.
        if self.watches_slack.is_negative() {
            self.watches_slack = original_slack;
            self.watches.insert(old_watch, coeff_old_watch);
            return WatchUpdate::Conflict;
        }

        // Check for propagations.
        let mut added_watches: Vec<_> = added_watch_coeff_pairs.iter().map(|&(l, _)| l).collect();
        self.watches.extend(added_watch_coeff_pairs);
        let mut propagated = Vec::new();
        for term in self.sorted_terms[self.next_pos_prop..].iter() {
            self.next_pos_prop += 1;
            let lit = term.get_lit();
            if unsafe { assign.is_unassigned_unchecked(lit) } {
                if term.get_coeff() > &self.watches_slack {
                    propagated.push(lit);
                } else {
                    self.next_pos_prop -= 1;
                    break;
                }
            }
        }

        // Could not find enough watches so that the watch slack is at least the largest coefficient. Keep old watch to watch enough after reset.
        added_watches.push(old_watch);
        // The old watch is no longer contributing to the slack of the watches and should be recovered later.
        self.watched_but_not_contributing
            .push((old_watch, coeff_old_watch));

        WatchUpdate::Unknown {
            propagated,
            new_watches: added_watches,
        }
    }

    #[inline]
    fn get_watches(&self) -> impl Iterator<Item = &Lit> {
        self.watches
            .keys()
            .chain(self.watched_but_not_contributing.iter().map(|(l, _)| l))
    }

    #[inline]
    fn get_constraint(&self) -> &Rc<DBConstraint> {
        &self.constraint
    }

    #[inline]
    fn reset_watch(&mut self) {
        self.next_pos = 0;
        self.next_pos_prop = 0;
        self.is_reset_not_contributing = false;
    }

    #[inline]
    fn reset_not_contributing(&mut self, assign: &Assignment<BooleanVar>) {
        let mut watched_but_not_contributing_new = Vec::new();
        for (lit, coeff) in self.watched_but_not_contributing.iter() {
            let value = unsafe { assign.get_lit_value_unchecked(*lit) };
            match value {
                BoolValue::Unassigned | BoolValue::Assigned(true) => {
                    // watch is actually contributing to the watchslack, so move it to "watches"
                    self.watches_slack += coeff;
                    self.watches.insert(*lit, coeff.clone());
                }
                BoolValue::Assigned(false) => {
                    // watch is not contributing, so put it back
                    watched_but_not_contributing_new.push((*lit, coeff.clone()));
                }
            }
        }
        self.watched_but_not_contributing = watched_but_not_contributing_new;
        self.is_reset_not_contributing = true;
    }

    fn remove_watches(&self, watchlist: &mut Vec<Vec<usize>>, watch_index: usize) {
        for lit in self
            .watches
            .keys()
            .chain(self.watched_but_not_contributing.iter().map(|(l, _)| l))
        {
            let list = watchlist.get_mut(lit.get_lit_data()).unwrap();
            let index = list.iter().position(|w| *w == watch_index).unwrap();
            list.swap_remove(index);
        }
    }
}
