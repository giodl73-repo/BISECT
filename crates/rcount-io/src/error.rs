use crate::*;

#[derive(Debug, Error)]
pub enum RcountIoError {
    #[error("core error: {0}")]
    Core(#[from] rcount_core::RcountCoreError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported RCOUNT version: {0}")]
    UnsupportedVersion(String),
    #[error("manifest content_hash mismatch: declared {declared}, computed {computed}")]
    ContentHashMismatch { declared: String, computed: String },
    #[error("source index is empty")]
    EmptySourceIndex,
    #[error("source path is not package-relative under sources/: {path}")]
    InvalidSourcePath { path: String },
    #[error("source file is missing: {path}")]
    MissingSourceFile { path: String },
    #[error("source hash mismatch for {source_id}: declared {declared}, computed {computed}")]
    SourceHashMismatch {
        source_id: String,
        declared: String,
        computed: String,
    },
    #[error("statement CSV row {row} is missing {field}")]
    MissingStatementCsvField { row: usize, field: String },
    #[error("statement CSV row {row} has invalid {field}: {value}")]
    InvalidStatementCsvField {
        row: usize,
        field: String,
        value: String,
    },
    #[error("statement CSV row {row} conflicts with prior {field} for {id}: {prior} vs {value}")]
    ConflictingStatementCsvField {
        row: usize,
        id: String,
        field: String,
        prior: String,
        value: String,
    },
    #[error("NIST CDF import is missing {field}")]
    MissingNistCdfField { field: String },
    #[error("NIST CDF import has invalid {field}: {value}")]
    InvalidNistCdfField { field: String, value: String },
    #[error("Rhode Island RLA import is missing section {section}")]
    MissingRhodeIslandRlaSection { section: String },
    #[error("Rhode Island RLA import is missing field {field}")]
    MissingRhodeIslandRlaField { field: String },
    #[error("Rhode Island RLA import has invalid field {field}: {value}")]
    InvalidRhodeIslandRlaField { field: String, value: String },
}
