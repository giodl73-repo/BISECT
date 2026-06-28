//! RCOUNT core: canonical count-ledger types, hashing, verification,
//! and synthetic fixtures. Split from a single 5.3k-line lib.rs into
//! cohesive modules, all re-exported here to preserve the public API.

pub(crate) use rayon::prelude::*;
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use serde_json::{Map, Value};
pub(crate) use sha2::{Digest, Sha256};
pub(crate) use std::collections::{BTreeMap, BTreeSet};
pub(crate) use thiserror::Error;

pub const RCOUNT_VERSION: &str = "0.1-draft";
pub const SOURCE_HASH_PREFIX: &[u8] = b"RCOUNT_SOURCE_V1\0";
pub const RECORD_HASH_PREFIX: &[u8] = b"RCOUNT_RECORD_V1\0";
pub const FILE_HASH_PREFIX: &[u8] = b"RCOUNT_FILE_V1\0";
pub const PACKAGE_HASH_PREFIX: &[u8] = b"RCOUNT_PACKAGE_V1\0";
pub const EVENT_HASH_PREFIX: &[u8] = b"RCOUNT_EVENT_V1\0";
pub const PROOF_HASH_PREFIX: &[u8] = b"RCOUNT_PROOF_V1\0";
pub const RLA_MANIFEST_HASH_PREFIX: &[u8] = b"RCOUNT_RLA_MANIFEST_V1\0";
pub const RLA_SAMPLE_PREFIX: &[u8] = b"RCOUNT_RLA_SAMPLE_V1\0";
pub const RLA_SAMPLING_ALGORITHM_ID: &str = "rcount-sha256-modulo-v1";
pub const COLORADO_RLA_METHOD_ID: &str = "colorado-rule-25-comparison-v1";
pub const CALIFORNIA_RLA_METHOD_ID: &str = "california-public-rla-v1";
pub const CALIFORNIA_BALLOT_MANIFEST_FORMAT_ID: &str =
    "ca-post-election-rla-ballot-manifest-2019-10-15";
pub const BRAVO_BALLOT_POLLING_METHOD_ID: &str = "bravo-ballot-polling-v1";
pub const MINERVA_BALLOT_POLLING_METHOD_ID: &str = "minerva-ballot-polling-v1";
pub const ATHENA_BALLOT_POLLING_METHOD_ID: &str = "athena-ballot-polling-v1";
pub const KAPLAN_MARKOV_COMPARISON_METHOD_ID: &str = "kaplan-markov-comparison-v1";
pub const ALPHA_MARTINGALE_METHOD_ID: &str = "alpha-martingale-v1";
pub const SHANGRLA_ASSORTER_METHOD_ID: &str = "shangrla-assorter-v1";
pub const STRATIFIED_HYBRID_RLA_METHOD_ID: &str = "stratified-hybrid-rla-v1";
pub const BATCH_COMPARISON_METHOD_ID: &str = "batch-comparison-v1";
pub const RAIRE_IRV_METHOD_ID: &str = "raire-irv-v1";
pub const AWAIRE_IRV_METHOD_ID: &str = "awaire-irv-v1";
pub const BAYESIAN_TABULATION_AUDIT_METHOD_ID: &str = "bayesian-tabulation-audit-v1";
pub const SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID: &str = "soba-observable-ballot-audit-v1";
pub const SYN_RCTX_L0_PACKAGE_HASH: &str =
    "sha256:bf552e9d9753d3376155ca9c4b21db6b1930e37919a58bcb9096cd563653d532";
pub const SYN_RCTX_L0_CONTEXT_HASH: &str =
    "sha256:b11f1eabcaf33e2d2691ddbe498c650830cffb9b0fb62820292d4ca0166c0bb7";
pub const SYN_RCTX_L0_CROSSWALK_HASH: &str =
    "sha256:906054d087e8c006047448c821d79e75a81bb1bbeb1d9349ab7b5d025029d9bb";
pub const SYN_RHIST_L2_PACKAGE_HASH: &str =
    "sha256:2c391099d7b61ba0c27fd231376391aadec81de62a387e291a043ed18d69db0b";

mod error;
mod fixtures;
mod model;
mod validation;
mod verify;
mod verify_audit;

pub use error::*;
pub use fixtures::*;
pub use model::*;
pub use validation::*;
pub use verify::*;
pub use verify_audit::*;

#[cfg(test)]
mod tests;
