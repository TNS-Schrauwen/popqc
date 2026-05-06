//! `PopQC` Core - Data types, traits, and shared infrastructure

pub mod config;
pub mod error;
pub mod metrics;
pub mod model;
pub mod normalize;
pub mod outlier;
pub mod traits;

pub use config::PopQCConfig;
pub use error::{PopQCError, Result};
pub use metrics::{Metric, MetricCatalog, MetricType, ThresholdLevel, Thresholds};
pub use model::CohortFrame;
pub use normalize::SampleNameNormalizer;
pub use traits::{ParseResult, QCParser};
