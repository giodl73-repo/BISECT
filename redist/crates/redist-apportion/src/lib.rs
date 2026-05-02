//! `redist-apportion` — Huntington-Hill apportionment and prime-factor redistricting.
//!
//! Three things in one crate:
//!
//! 1. **`huntington_hill`** — allocates congressional seats to states using the
//!    geometric-mean priority rule (2 U.S.C. §2a).
//!
//! 2. **`prime_factor_sequence`** — returns the canonical prime factorization
//!    sequence of a seat count (smallest prime first, with repetition).
//!
//! 3. **`PfrCompositor`** — hierarchically applies a pluggable `SplitStrategy`
//!    to a census-tract graph, following the prime factorization of `n`.
//!    The compositor is the algorithmic core of Paper B.11 (Prime-Factored Maps).
//!
//! The `SplitStrategy` trait lets callers plug in different partitioning
//! algorithms: min-edge-cut, compactness-maximising, proportional, etc.
//! The default implementation `MetisKwaySplit` uses the METIS k-way solver
//! with population vertex weights and TIGER boundary-length edge weights.

pub mod huntington_hill;
pub mod prime;
pub mod graph;
pub mod split;
pub mod compositor;

pub use huntington_hill::huntington_hill;
pub use prime::prime_factor_sequence;
pub use graph::SubGraph;
pub use split::{SplitError, SplitStrategy, MetisKwaySplit};
pub use compositor::PfrCompositor;
