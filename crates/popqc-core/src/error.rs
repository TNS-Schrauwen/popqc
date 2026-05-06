//! Error types for `PopQC`

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PopQCError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV parsing error: {0}")]
    Csv(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
    #[error("Parser error for '{parser}': {message}")]
    Parser { parser: String, message: String },
    #[error("No parseable files found in: {0}")]
    NoFilesFound(String),
    #[error("Sample name conflict: {0}")]
    SampleConflict(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Missing required column: {0}")]
    MissingColumn(String),
    #[error("Report generation failed: {0}")]
    Report(String),
}

pub type Result<T> = std::result::Result<T, PopQCError>;
