//! Configuration types for `PopQC`

use crate::metrics::Thresholds;
use crate::outlier::OutlierMethod;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopQCConfig {
    pub project: ProjectConfig,
    pub discovery: DiscoveryConfig,
    pub samples: SamplesConfig,
    pub thresholds: HashMap<String, Thresholds>,
    pub report: ReportConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub description: String,
    pub pipeline: String,
    pub genome: String,
    pub annotation: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "PopQC Report".to_string(),
            description: String::new(),
            pipeline: String::new(),
            genome: String::new(),
            annotation: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub search_paths: Vec<PathBuf>,
    pub max_depth: usize,
    pub exclude_patterns: Vec<String>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![PathBuf::from(".")],
            max_depth: 10,
            exclude_patterns: vec![
                "work".to_string(),
                ".git".to_string(),
                "tmp".to_string(),
                ".nextflow".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SamplesConfig {
    pub metadata_file: Option<PathBuf>,
    pub name_column: Option<String>,
    pub group_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub title: String,
    pub output: PathBuf,
    pub embed_data: bool,
    pub max_inline_samples: usize,
    pub outlier_method: OutlierMethod,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            title: "PopQC Report".to_string(),
            output: PathBuf::from("popqc_report.html"),
            embed_data: true,
            max_inline_samples: 5000,
            outlier_method: OutlierMethod::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub threads: usize,
    pub memory_limit_mb: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            threads: 0,
            memory_limit_mb: 8192,
        }
    }
}
