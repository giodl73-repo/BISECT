//! `BISECT-apportion` — Huntington-Hill apportionment and prime-factor redistricting.
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
//!    The compositor is the algorithmic core of Paper T.4 (Prime-Factored Maps).
//!
//! The `SplitStrategy` trait lets callers plug in different partitioning
//! algorithms: min-edge-cut, compactness-maximising, proportional, etc.
//! The default implementation `MetisKwaySplit` uses the METIS k-way solver
//! with population vertex weights and TIGER boundary-length edge weights.

pub mod compositor;
pub mod divisor_methods;
pub mod evidence_manifest;
pub mod graph;
pub mod huntington_hill;
pub mod nest;
pub mod paradoxes;
pub mod prime;
pub mod spectral;
pub mod split;

pub use compositor::PfrCompositor;
pub use divisor_methods::{apportionment_divisor, RoundingRule};
pub use graph::SubGraph;
pub use huntington_hill::huntington_hill;
pub use nest::{
    compatible_spines, spine_compatibility_score, us_state_compatibility_table, StateCompatibility,
};
pub use paradoxes::check_alabama_paradox;
pub use prime::{
    pfr_tree_depth, prime_factor_sequence, split_prescription, SplitStep, MAX_DIRECT_SPLIT,
};
pub use spectral::{spectral_bisect, SpectralConfig, SpectralError, SpectralResult};
pub use split::{MetisPartitioner, Partitioner, SplitError};
