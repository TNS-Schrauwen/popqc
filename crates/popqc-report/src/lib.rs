//! `PopQC` Report - Generate interactive HTML reports

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cast_precision_loss)]

use popqc_core::PopQCError;
use popqc_core::config::PopQCConfig;
use popqc_core::error::Result;
use popqc_core::model::CohortFrame;
use std::fs;
use std::path::Path;
use tracing::info;

mod html;

pub fn generate_report(
    frame: &CohortFrame,
    config: &PopQCConfig,
    output_path: &Path,
) -> Result<()> {
    info!(
        "Generating report: {} samples, {} metrics",
        frame.num_samples(),
        frame.num_metrics()
    );

    let html_content = html::build_html(frame, config)?;

    fs::write(output_path, &html_content).map_err(|e| PopQCError::Report(e.to_string()))?;

    let size_mb = html_content.len() as f64 / 1024.0 / 1024.0;
    info!(
        "Report written to {} ({:.1} MB)",
        output_path.display(),
        size_mb
    );

    Ok(())
}
