//! Core traits for all parsers

#![allow(clippy::missing_errors_doc)]

use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub sample_name: String,
    pub metrics: HashMap<String, f64>,
    pub warnings: Vec<String>,
}

pub trait QCParser: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn file_patterns(&self) -> &[&str];
    fn can_parse(&self, path: &Path) -> bool {
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        self.file_patterns().iter().any(|pattern| {
            let pattern = pattern.trim_start_matches('*');
            filename.ends_with(pattern)
        })
    }

    fn parse(&self, path: &Path) -> Result<Vec<ParseResult>>;
    fn estimated_memory(&self, file_size: u64) -> u64 {
        file_size * 2
    }
}
