//! Keeping track of variables and their names.

use ahash::AHashMap;
use smartstring::alias::String;

use crate::var_type::VarIdx;

/// Read-only access to variable names by [`VarIdx`]. Used for pretty printing and tracing.
pub trait VarNameLookup {
    /// Name of the variable with index `idx`.
    fn get_name(&self, idx: VarIdx) -> &str;

    /// Number of variables currently tracked.
    fn len(&self) -> usize;

    /// Whether no variables are currently tracked.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Records variable names and assigns each a stable [`VarIdx`]. Used by parsers.
pub trait VarNameRecorder {
    /// Return the [`VarIdx`] for `name`, allocating a new one the first time the name is seen.
    fn add_by_name(&mut self, name: &str) -> VarIdx;
}

/// Structure to manage variables in the formula.
#[derive(Clone, Debug, Default)]
pub struct VarNameManager {
    /// Map from variable name to variable index in the vector `vars`.
    name_to_idx: AHashMap<String, VarIdx>,
    /// Vector holding assignment for this set of variables.
    idx_to_name: Vec<String>,
}

impl VarNameManager {
    /// Construct a new variable set with initial capacity `capacity`.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        VarNameManager {
            name_to_idx: AHashMap::with_capacity(capacity),
            idx_to_name: Vec::with_capacity(capacity),
        }
    }

    /// Add variable by name to the [`VarNameManager`] of the formula and return the variable ID.
    #[inline]
    pub fn add_by_name(&mut self, name: &str) -> VarIdx {
        match self.name_to_idx.get(name) {
            Some(idx) => *idx,
            None => {
                let idx = self.idx_to_name.len();
                self.name_to_idx.insert(name.into(), idx);
                self.idx_to_name.push(name.into());
                idx
            }
        }
    }

    /// Get the variable index by the name of the variable.
    ///
    /// Returns `Some(VarIdx)` if the variable exists and `None` if the variable does not exist.
    #[inline]
    pub fn get_idx(&self, name: &str) -> Option<VarIdx> {
        self.name_to_idx.get(name).copied()
    }

    /// Get the name of a variable in the [`VarNameManager`].
    #[inline]
    pub fn get_name(&self, idx: VarIdx) -> &str {
        &self.idx_to_name[idx]
    }

    /// The number of variables in the [`VarNameManager`].
    #[inline]
    pub fn len(&self) -> usize {
        self.idx_to_name.len()
    }

    /// Check if the contains any variables.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.idx_to_name.is_empty()
    }

    /// Return the currently allocated capacity of the [`VarNameManager`].
    #[inline]
    pub fn capacity(&self) -> usize {
        self.idx_to_name.capacity()
    }

    /// Reserve capacity for exactly `additional` many more variables in the [`VarNameManager`].
    ///
    /// This function can be used to avoid reallocation of the memory inside the [`VarNameManager`] when the number of additional variables is known.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.idx_to_name.reserve_exact(additional);
    }
}

impl VarNameLookup for VarNameManager {
    #[inline]
    fn get_name(&self, idx: VarIdx) -> &str {
        VarNameManager::get_name(self, idx)
    }

    #[inline]
    fn len(&self) -> usize {
        VarNameManager::len(self)
    }
}

impl VarNameRecorder for VarNameManager {
    #[inline]
    fn add_by_name(&mut self, name: &str) -> VarIdx {
        VarNameManager::add_by_name(self, name)
    }
}
