//! rgraph-core: generic, domain-agnostic graph kernels (shortest paths,
//! edge cut, connectivity, components/bridges/articulation/betweenness).
//! Split into modules, all re-exported here to preserve the public API.

pub(crate) use std::cmp::Ordering;
pub(crate) use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
pub(crate) use std::fmt::Debug;
pub(crate) use std::hash::Hash;
pub(crate) use thiserror::Error;

mod components;
mod connectivity;
mod cut;
mod shortest_path;
mod types;

pub use components::*;
pub use connectivity::*;
pub use cut::*;
pub use shortest_path::*;
pub use types::*;

#[cfg(test)]
mod tests;
