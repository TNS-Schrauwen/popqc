//! STAR aligner Log.final.out parser

#![allow(clippy::collapsible_if)]
#![allow(clippy::unnecessary_literal_bound)]

use popqc_core::PopQCError;
use popqc_core::error::Result;
use popqc_core::normalize::SampleNameNormalizer;
use popqc_core::traits::{ParseResult, QCParser};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct StarLogParser {
    normalizer: SampleNameNormalizer,
}

impl StarLogParser {
    #[must_use]
    pub fn new() -> Self {
        Self {
            normalizer: SampleNameNormalizer::default(),
        }
    }

    fn extract_value(line: &str) -> Option<f64> {
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() != 2 {
            return None;
        }
        let value_str = parts[1].trim().trim_end_matches('%');
        value_str.parse::<f64>().ok()
    }
}

impl Default for StarLogParser {
    fn default() -> Self {
        Self::new()
    }
}

impl QCParser for StarLogParser {
    fn id(&self) -> &str {
        "star_log"
    }

    fn name(&self) -> &str {
        "STAR Log.final.out"
    }

    fn description(&self) -> &str {
        "Parser for STAR aligner Log.final.out summary files"
    }

    fn file_patterns(&self) -> &[&str] {
        &["*Log.final.out"]
    }

    fn parse(&self, path: &Path) -> Result<Vec<ParseResult>> {
        let content = fs::read_to_string(path).map_err(PopQCError::Io)?;
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("unknown");
        let sample_name = self.normalizer.normalize(filename);

        let mut metrics = HashMap::new();
        let metric_map: &[(&str, &str)] = &[
            ("Number of input reads", "star_total_reads"),
            ("Uniquely mapped reads %", "star_uniquely_mapped_percent"),
            ("Uniquely mapped reads number", "star_uniquely_mapped"),
            ("Average mapped length", "star_avg_mapped_read_length"),
            ("Number of splices: Total", "star_num_splices"),
            ("Mismatch rate per base, %", "star_mismatch_rate"),
            (
                "% of reads mapped to multiple loci",
                "star_multimapped_percent",
            ),
            (
                "% of reads unmapped: too short",
                "star_unmapped_tooshort_percent",
            ),
            ("% of reads unmapped: other", "star_unmapped_other_percent"),
            ("Insertion rate per base, %", "star_insertion_rate"),
            ("Deletion rate per base, %", "star_deletion_rate"),
        ];

        for line in content.lines() {
            let trimmed = line.trim();
            for &(pattern, metric_id) in metric_map {
                if trimmed.contains(pattern) {
                    if let Some(value) = Self::extract_value(trimmed) {
                        metrics.insert(metric_id.to_string(), value);
                    }
                }
            }
        }

        if metrics.is_empty() {
            return Err(PopQCError::Parser {
                parser: "star_log".to_string(),
                message: format!("No metrics extracted from {}", path.display()),
            });
        }

        Ok(vec![ParseResult {
            sample_name,
            metrics,
            warnings: Vec::new(),
        }])
    }
}
