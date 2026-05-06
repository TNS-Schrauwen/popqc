//! Generic parser for `MultiQC` tab-separated aggregate tables

#![allow(clippy::collapsible_if)]
#![allow(clippy::manual_strip)]
#![allow(clippy::unnecessary_literal_bound)]

use popqc_core::PopQCError;
use popqc_core::error::Result;
use popqc_core::normalize::SampleNameNormalizer;
use popqc_core::traits::{ParseResult, QCParser};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

pub struct MultiQCTableParser {
    tool_id: String,
    patterns: Vec<String>,
    normalizer: SampleNameNormalizer,
}

impl MultiQCTableParser {
    #[must_use]
    pub fn new(tool_id: &str, patterns: &[&str]) -> Self {
        Self {
            tool_id: tool_id.to_string(),
            patterns: patterns.iter().map(|p| (*p).to_string()).collect(),
            normalizer: SampleNameNormalizer::default(),
        }
    }

    fn parse_tsv(&self, path: &Path) -> Result<Vec<ParseResult>> {
        let content = fs::read_to_string(path).map_err(PopQCError::Io)?;
        let mut lines = content.lines();
        let header_line = lines.next().ok_or_else(|| PopQCError::Parser {
            parser: self.tool_id.clone(),
            message: "Empty file".to_string(),
        })?;

        let headers: Vec<&str> = header_line.split('\t').collect();
        if headers.is_empty() {
            return Err(PopQCError::Parser {
                parser: self.tool_id.clone(),
                message: "No columns found in header".to_string(),
            });
        }

        let mut merged: HashMap<String, HashMap<String, f64>> = HashMap::new();

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            if fields.is_empty() {
                continue;
            }

            let raw_name = fields[0].trim();
            let sample_name = self.normalizer.normalize(raw_name);

            if sample_name.is_empty() {
                warn!("Empty sample name after normalization: '{}'", raw_name);
                continue;
            }

            let entry = merged.entry(sample_name.clone()).or_default();

            for (i, &header) in headers.iter().enumerate().skip(1) {
                if i >= fields.len() {
                    break;
                }

                let value_str = fields[i].trim();
                if value_str.is_empty()
                    || value_str == "NA"
                    || value_str == "None"
                    || value_str == "N/A"
                    || value_str == "."
                {
                    continue;
                }

                if let Ok(value) = value_str.parse::<f64>() {
                    if value.is_finite() {
                        let metric_id = format!("{}_{}", self.tool_id, header.trim());
                        entry.entry(metric_id).or_insert(value);
                    }
                }
            }
        }

        let mut results = Vec::new();
        for (sample_name, metrics) in merged {
            if !metrics.is_empty() {
                debug!(
                    "Parsed sample '{}' with {} metrics from {}",
                    sample_name,
                    metrics.len(),
                    self.tool_id
                );
                results.push(ParseResult {
                    sample_name,
                    metrics,
                    warnings: Vec::new(),
                });
            }
        }

        Ok(results)
    }
}

impl QCParser for MultiQCTableParser {
    fn id(&self) -> &str {
        &self.tool_id
    }

    fn name(&self) -> &str {
        &self.tool_id
    }

    fn description(&self) -> &str {
        "MultiQC aggregate table parser"
    }

    fn file_patterns(&self) -> &[&str] {
        &[]
    }

    fn can_parse(&self, path: &Path) -> bool {
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        self.patterns.iter().any(|p| {
            if p.starts_with('*') {
                filename.ends_with(&p[1..])
            } else {
                filename == p.as_str()
            }
        })
    }

    fn parse(&self, path: &Path) -> Result<Vec<ParseResult>> {
        self.parse_tsv(path)
    }
}
