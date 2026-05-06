//! `PopQC` CLI

#![allow(clippy::large_enum_variant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::uninlined_format_args)]

use anyhow::Context;
use clap::{Parser, Subcommand};
use popqc_core::config::PopQCConfig;
use popqc_core::model::CohortFrame;
use popqc_core::normalize::SampleNameNormalizer;
use popqc_discovery::DiscoveryEngine;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "popqc")]
#[command(version)]
#[command(
    about = "Fast, scalable qualtiy control outlier analysis tool for large cohort genomics datasets."
)]
#[command(
    long_about = "PopQC generates interactive HTML reports for quality control \
    analysis of any large cohort genomics dataset (RNAseq, WGS, WES, variant calling, etc.). \
    It auto-discovers QC outputs from such as STAR, MultiQC, FastQC, Salmon, Picard, \
    samtools, featureCounts, RSeQC, Mosdepth, Qualimap, bcftools, and more."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[arg(short, long, default_value = "popqc_report.html")]
        output: PathBuf,
        #[arg(short, long)]
        metadata: Option<PathBuf>,
        #[arg(long)]
        sample_col: Option<String>,
        #[arg(long, default_value = "PopQC Report")]
        title: String,
        #[arg(long, default_value = "")]
        pipeline: String,
        #[arg(long, default_value = "")]
        genome: String,
        #[arg(long, default_value = "")]
        annotation: String,
        #[arg(long, default_value = "10")]
        max_depth: usize,
        #[arg(short, long)]
        verbose: bool,
    },

    Parsers,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            paths,
            output,
            metadata,
            sample_col,
            title,
            pipeline,
            genome,
            annotation,
            max_depth,
            verbose,
        } => {
            let filter = if verbose {
                EnvFilter::new("debug")
            } else {
                EnvFilter::new("info")
            };
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_target(false)
                .init();

            run_report(
                paths, output, metadata, sample_col, title, pipeline, genome, annotation, max_depth,
            )?;
        }
        Commands::Parsers => {
            list_parsers();
        }
    }

    Ok(())
}

fn run_report(
    paths: Vec<PathBuf>,
    output: PathBuf,
    metadata: Option<PathBuf>,
    sample_col: Option<String>,
    title: String,
    pipeline: String,
    genome: String,
    annotation: String,
    max_depth: usize,
) -> anyhow::Result<()> {
    let start = Instant::now();

    println!("╔══════════════════════════════════════════╗");
    println!("║        PopQC Report Generator            ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    info!("Step 1: Discovering QC files...");
    let engine = DiscoveryEngine::new().with_max_depth(max_depth);
    let frame = engine
        .run(&paths)
        .context("Failed to discover and parse QC files")?;

    info!(
        "  Found {} samples with {} metrics",
        frame.num_samples(),
        frame.num_metrics()
    );

    let frame = if let Some(meta_path) = metadata {
        info!("Step 2: Loading metadata from {}...", meta_path.display());
        load_metadata(frame, &meta_path, sample_col.as_deref())?
    } else {
        info!("Step 2: No metadata provided (optional). Skipping.");
        frame
    };

    info!("Step 3: Generating HTML report...");
    let config = PopQCConfig {
        project: popqc_core::config::ProjectConfig {
            name: title.clone(),
            description: String::new(),
            pipeline,
            genome,
            annotation,
        },
        report: popqc_core::config::ReportConfig {
            title,
            output: output.clone(),
            ..Default::default()
        },
        ..Default::default()
    };

    popqc_report::generate_report(&frame, &config, &output).context("Failed to generate report")?;

    let elapsed = start.elapsed();
    println!();
    println!("✓ Report generated: {}", output.display());
    println!("  Samples: {}", frame.num_samples());
    println!("  Metrics: {}", frame.num_metrics());
    println!("  Metadata fields: {}", frame.num_metadata_fields());
    println!("  Time: {:.2}s", elapsed.as_secs_f64());

    Ok(())
}

fn load_metadata(
    mut frame: CohortFrame,
    path: &PathBuf,
    sample_col: Option<&str>,
) -> anyhow::Result<CohortFrame> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read metadata file: {}", path.display()))?;
    let first_line = content.lines().next().context("Empty metadata file")?;
    let delimiter = if first_line.contains('\t') { '\t' } else { ',' };
    let mut lines = content.lines();
    let header = lines.next().context("Empty metadata file")?;
    let columns: Vec<&str> = header.split(delimiter).map(str::trim).collect();
    if columns.is_empty() {
        anyhow::bail!("No columns in metadata file");
    }
    let id_col_idx = if let Some(col_name) = sample_col {
        columns
            .iter()
            .position(|&c| c.eq_ignore_ascii_case(col_name))
            .with_context(|| {
                format!(
                    "Sample column '{}' not found in metadata. Available columns: {:?}",
                    col_name, columns
                )
            })?
    } else {
        let common_names = [
            "sample_id",
            "sampleid",
            "sample",
            "sample_name",
            "samplename",
            "name",
            "id",
        ];
        let mut found_idx = None;
        for &name in &common_names {
            if let Some(pos) = columns.iter().position(|&c| c.eq_ignore_ascii_case(name)) {
                found_idx = Some(pos);
                break;
            }
        }
        found_idx.unwrap_or(0)
    };
    info!(
        "  Using column '{}' (index {}) as sample identifier",
        columns[id_col_idx], id_col_idx
    );
    let normalizer = SampleNameNormalizer::default();
    let qc_sample_names: Vec<String> = frame.sample_names().iter().map(ToString::to_string).collect();
    let mut exact_index: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut normalized_index: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let _prefix_index: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for name in &qc_sample_names {
        exact_index.insert(name.clone(), name.clone());
        let norm = normalizer.normalize(name);
        normalized_index.insert(norm.clone(), name.clone());
    }

    let mut matched = 0usize;
    let mut total = 0usize;

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        total += 1;
        let fields: Vec<&str> = line.split(delimiter).map(str::trim).collect();
        if fields.len() <= id_col_idx {
            continue;
        }

        let raw_id = fields[id_col_idx].trim();
        if raw_id.is_empty() {
            continue;
        }

        let normalized_id = normalizer.normalize(raw_id);
        let mut metadata = std::collections::HashMap::new();
        for (i, &col_name) in columns.iter().enumerate() {
            if i == id_col_idx {
                continue;
            }
            if i < fields.len() {
                let val = fields[i].trim();
                if !val.is_empty() && val != "NA" && val != "N/A" && val != "." {
                    let clean_col = col_name
                        .replace(['(', ')'], "")
                        .replace(['-','_'], "")
                        .replace('/', "_")
                        .trim()
                        .to_string();
                    metadata.insert(clean_col, val.to_string());
                }
            }
        }

        if metadata.is_empty() {
            continue;
        }

        let matched_name = find_matching_sample(
            raw_id,
            &normalized_id,
            &qc_sample_names,
            &exact_index,
            &normalized_index,
            &normalizer,
        );

        if let Some(sample_name) = matched_name {
            let record = popqc_core::model::SampleRecord {
                sample: sample_name,
                metadata,
                metrics: std::collections::HashMap::new(),
            };
            frame.upsert_sample(record);
            matched += 1;
        }
    }

    let meta_col_names: Vec<&str> = columns
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != id_col_idx)
        .map(|(_, c)| *c)
        .collect();

    info!(
        "  Metadata: {}/{} rows matched to samples ({} columns: {})",
        matched,
        total,
        meta_col_names.len(),
        meta_col_names.join(", ")
    );

    if matched == 0 {
        warn!(
            "  WARNING: No metadata rows matched! Check that your metadata sample IDs match QC sample names."
        );
        warn!(
            "  Metadata IDs (first 5): {:?}",
            content
                .lines()
                .skip(1)
                .take(5)
                .map(|l| l.split(delimiter).nth(id_col_idx).unwrap_or(""))
                .collect::<Vec<_>>()
        );
        warn!(
            "  QC sample names (first 5): {:?}",
            qc_sample_names.iter().take(5).collect::<Vec<_>>()
        );
    }

    Ok(frame)
}

fn find_matching_sample(
    raw_id: &str,
    normalized_id: &str,
    qc_sample_names: &[String],
    exact_index: &std::collections::HashMap<String, String>,
    normalized_index: &std::collections::HashMap<String, String>,
    normalizer: &SampleNameNormalizer,
) -> Option<String> {
    if let Some(name) = exact_index.get(raw_id) {
        return Some(name.clone());
    }
    if let Some(name) = normalized_index.get(normalized_id) {
        return Some(name.clone());
    }
    if let Some(name) = exact_index.get(normalized_id) {
        return Some(name.clone());
    }
    let clean_id = raw_id.replace(['-', '_'], "").to_lowercase();
    let norm_clean = normalized_id.replace(['-', '_'], "").to_lowercase();

    for qc_name in qc_sample_names {
        let qc_clean = qc_name.replace(['-', '_'], "").to_lowercase();
        if qc_clean.starts_with(&clean_id) || qc_clean.starts_with(&norm_clean) {
            return Some(qc_name.clone());
        }

        let qc_lower = qc_name.to_lowercase();
        let raw_lower = raw_id.to_lowercase();
        if qc_lower.starts_with(&raw_lower) {
            return Some(qc_name.clone());
        }

        let qc_norm = normalizer.normalize(qc_name).to_lowercase();
        if qc_norm.starts_with(&normalized_id.to_lowercase()) {
            return Some(qc_name.clone());
        }
    }

    for qc_name in qc_sample_names {
        let qc_lower = qc_name.to_lowercase();
        let raw_lower = raw_id.to_lowercase();
        if raw_lower.len() >= 4 && qc_lower.contains(&raw_lower) {
            return Some(qc_name.clone());
        }
    }

    None
}

fn list_parsers() {
    let registry = popqc_parsers::ParserRegistry::with_defaults();
    println!("Available PopQC Parsers ({}):", registry.len());
    println!("{:-<60}", "");
    println!("{:<25} {:<}", "Parser ID", "Description");
    println!("{:-<60}", "");
    for parser in registry.all_parsers() {
        println!("  {:<23} {}", parser.id(), parser.description());
    }
    println!();
    println!("PopQC auto-discovers files matching these parsers.");
    println!("Works with: RNAseq, WGS, WES, variant calling, and more.");
}
