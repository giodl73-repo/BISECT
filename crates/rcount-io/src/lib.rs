//! rcount-io: RCOUNT package directory read/write plus CSV/NIST/RI
//! importers. Split into modules, all re-exported here to preserve the
//! public API consumed by rcount-audit/district/cli.

pub(crate) use rcount_core::{
    package_content_hash, verify_jurisdiction_total, verify_package, AuditAlgorithmDecision,
    AuditAlgorithmRun, AuditAssertion, AuditAssertionKind, AuditSamplingMode, BatchKind,
    BatchManifest, Contest, CountStatus, RcountPackage, ReportingUnit, ReportingUnitKind,
    Selection, SelectionKind, SelectionTotal, StatusEvent, Summary,
    ATHENA_BALLOT_POLLING_METHOD_ID, MINERVA_BALLOT_POLLING_METHOD_ID, RCOUNT_VERSION,
    SOURCE_HASH_PREFIX,
};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use serde_json::Value;
pub(crate) use sha2::{Digest, Sha256};
pub(crate) use std::collections::{BTreeMap, BTreeSet};
pub(crate) use std::fs::{self, File};
pub(crate) use std::io::{BufRead, BufReader, Write};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use thiserror::Error;

mod docs_dirs;
mod error;
mod helpers;
mod import;
mod manifest;
mod package;

pub use docs_dirs::*;
pub use error::*;
pub(crate) use helpers::*;
pub use import::*;
pub use manifest::*;
pub use package::*;

#[cfg(test)]
mod tests;
