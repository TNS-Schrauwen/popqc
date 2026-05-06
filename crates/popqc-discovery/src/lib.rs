//! `PopQC` Discovery: Auto-discover and parse QC files

#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use popqc_core::PopQCError;
use popqc_core::error::Result;
use popqc_core::model::{CohortFrame, SampleRecord};
use popqc_parsers::ParserRegistry;
use rayon::prelude::*;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

pub struct DiscoveryEngine {
    registry: ParserRegistry,
    max_depth: usize,
    exclude_patterns: Vec<String>,
}

impl DiscoveryEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: ParserRegistry::with_defaults(),
            max_depth: 10,
            exclude_patterns: vec!["work".to_string(), ".git".to_string(), "tmp".to_string()],
        }
    }

    #[must_use]
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    #[must_use]
    pub fn with_excludes(mut self, excludes: Vec<String>) -> Self {
        self.exclude_patterns = excludes;
        self
    }

    pub fn run(&self, search_paths: &[PathBuf]) -> Result<CohortFrame> {
        info!(
            "Starting discovery across {} path(s) with {} parsers",
            search_paths.len(),
            self.registry.len()
        );

        let candidates = self.find_candidates(search_paths);
        info!("Found {} candidate files", candidates.len());

        if candidates.is_empty() {
            return Err(PopQCError::NoFilesFound(
                search_paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        let frame = Mutex::new(CohortFrame::new());
        let parse_count = Mutex::new(0usize);

        candidates.par_iter().for_each(|(path, parser_id)| {
            let parsers = self.registry.find_parsers(path);
            for parser in parsers {
                if parser.id() != parser_id {
                    continue;
                }
                match parser.parse(path) {
                    Ok(results) => {
                        let mut f = frame.lock().unwrap();
                        for result in results {
                            let record = SampleRecord {
                                sample: result.sample_name,
                                metadata: std::collections::HashMap::new(),
                                metrics: result.metrics,
                            };
                            f.upsert_sample(record);
                        }
                        *parse_count.lock().unwrap() += 1;
                    }
                    Err(e) => {
                        warn!("Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        });

        let frame = frame.into_inner().unwrap();
        let count = *parse_count.lock().unwrap();
        info!(
            "Discovery complete: {} files parsed, {} samples, {} metrics",
            count,
            frame.num_samples(),
            frame.num_metrics()
        );

        Ok(frame)
    }

    fn find_candidates(&self, search_paths: &[PathBuf]) -> Vec<(PathBuf, String)> {
        let mut candidates = Vec::new();

        for search_path in search_paths {
            if !search_path.exists() {
                warn!("Search path does not exist: {}", search_path.display());
                continue;
            }

            let walker = WalkDir::new(search_path)
                .max_depth(self.max_depth)
                .follow_links(true)
                .into_iter()
                .filter_entry(|e| !self.is_excluded(e.path()));

            for entry in walker.flatten() {
                if !entry.file_type().is_file() {
                    continue;
                }

                let path = entry.path().to_path_buf();
                let parsers = self.registry.find_parsers(&path);
                for parser in parsers {
                    debug!("Matched {} -> parser '{}'", path.display(), parser.id());
                    candidates.push((path.clone(), parser.id().to_string()));
                }
            }
        }

        candidates
    }

    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns
            .iter()
            .any(|pat| path_str.contains(pat.as_str()))
    }
}

impl Default for DiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}
