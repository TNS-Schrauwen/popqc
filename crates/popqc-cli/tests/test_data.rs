//! Integration test: Run with test QC 500-sample data

use popqc_core::config::PopQCConfig;
use popqc_core::model::CohortFrame;
use popqc_discovery::DiscoveryEngine;
use std::collections::HashSet;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("test_data")
}

fn multiqc_dir() -> PathBuf {
    fixtures_dir().join("multiqc_data")
}

fn metadata_path() -> PathBuf {
    fixtures_dir().join("metadata.tsv")
}

#[test]
fn test_discovery_finds_all_synthetic_files() {
    let qc_dir = multiqc_dir();
    if !qc_dir.exists() {
        eprintln!("Skipping: synthetic fixtures not found at {:?}", qc_dir);
        eprintln!("Run: python generate_synthetic_data.py --output-dir tests/fixtures/test_data");
        return;
    }

    let engine = DiscoveryEngine::new().with_max_depth(3);
    let frame = engine.run(&[qc_dir]).unwrap();

    // Should find exactly 500 unique samples
    assert_eq!(
        frame.num_samples(),
        500,
        "Expected 500 unique samples, got {}",
        frame.num_samples()
    );

    // Should find metrics from multiple tools
    assert!(
        frame.num_metrics() > 10,
        "Expected >10 metrics, got {}",
        frame.num_metrics()
    );

    println!(
        "Discovery OK: {} samples, {} metrics",
        frame.num_samples(),
        frame.num_metrics()
    );
}

#[test]
fn test_sample_names_are_correct() {
    let qc_dir = multiqc_dir();
    if !qc_dir.exists() {
        return;
    }

    let engine = DiscoveryEngine::new();
    let frame = engine.run(&[qc_dir]).unwrap();

    let names: Vec<&str> = frame.sample_names();

    for name in &names {
        assert!(
            name.starts_with("sample"),
            "Unexpected sample name: '{}'",
            name
        );
    }

    let unique: HashSet<&&str> = names.iter().collect();
    assert_eq!(
        unique.len(),
        names.len(),
        "Found duplicate sample names! {} unique vs {} total",
        unique.len(),
        names.len()
    );

    // Verify specific samples exist
    assert!(frame.get_sample("sample1").is_some());
    assert!(frame.get_sample("sample250").is_some());
    assert!(frame.get_sample("sample500").is_some());
    assert!(frame.get_sample("sample501").is_none());
}

#[test]
fn test_metrics_have_expected_ranges() {
    let qc_dir = multiqc_dir();
    if !qc_dir.exists() {
        return;
    }

    let engine = DiscoveryEngine::new();
    let frame = engine.run(&[qc_dir]).unwrap();
    let star_metric = frame
        .metric_ids()
        .iter()
        .find(|m| m.contains("uniquely_mapped_percent"))
        .cloned();

    if let Some(metric_id) = star_metric {
        let values = frame.metric_values_non_null(&metric_id);
        assert!(!values.is_empty(), "No STAR alignment values found");
        for &v in &values {
            assert!(v >= 0.0 && v <= 100.0, "STAR alignment {} out of range", v);
        }
        let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        println!("STAR alignment range: {:.1} - {:.1}", min_val, max_val);
        // Based on our synthetic distribution: should span from ~60 to ~99
        assert!(min_val < 75.0, "Min STAR should be <75, got {}", min_val);
        assert!(max_val > 88.0, "Max STAR should be >88, got {}", max_val);
    }

    // Check GC content
    let gc_metric = frame
        .metric_ids()
        .iter()
        .find(|m| m.contains("percent_gc") || m.contains("gc"))
        .cloned();

    if let Some(metric_id) = gc_metric {
        let values = frame.metric_values_non_null(&metric_id);
        assert!(!values.is_empty(), "No GC values found");
        let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        println!("GC content range: {:.1} - {:.1}", min_val, max_val);
        assert!(
            min_val >= 25.0 && max_val <= 80.0,
            "GC out of expected range"
        );
    }
}

#[test]
fn test_metadata_loading() {
    let qc_dir = multiqc_dir();
    let meta = metadata_path();
    if !qc_dir.exists() || !meta.exists() {
        return;
    }

    let engine = DiscoveryEngine::new();
    let mut frame = engine.run(&[qc_dir]).unwrap();

    // Load metadata
    let content = std::fs::read_to_string(&meta).unwrap();
    let normalizer = popqc_core::normalize::SampleNameNormalizer::default();

    let mut lines = content.lines();
    let header = lines.next().unwrap();
    let columns: Vec<&str> = header.split('\t').collect();

    let mut matched = 0;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let sample_id = fields[0].trim();

        if frame.get_sample(sample_id).is_some() {
            let mut metadata = std::collections::HashMap::new();
            for (i, &col) in columns.iter().enumerate().skip(1) {
                if i < fields.len() && !fields[i].trim().is_empty() {
                    metadata.insert(col.to_string(), fields[i].trim().to_string());
                }
            }
            let record = popqc_core::model::SampleRecord {
                sample: sample_id.to_string(),
                metadata,
                metrics: std::collections::HashMap::new(),
            };
            frame.upsert_sample(record);
            matched += 1;
        }
    }

    assert_eq!(
        matched, 500,
        "Expected 500 metadata matches, got {}",
        matched
    );

    assert!(
        frame.num_metadata_fields() > 0,
        "No metadata fields found after loading"
    );

    let sample1 = frame.get_sample("sample1").unwrap();
    assert!(
        sample1.metadata.contains_key("condition"),
        "sample1 should have 'condition' metadata"
    );
    assert!(
        sample1.metadata.contains_key("sex"),
        "sample1 should have 'sex' metadata"
    );
    assert!(
        sample1.metadata.contains_key("age"),
        "sample1 should have 'age' metadata"
    );

    println!(
        "Metadata OK: {} fields loaded for {} samples",
        frame.num_metadata_fields(),
        matched
    );
}

#[test]
fn test_report_generation_with_synthetic_data() {
    let qc_dir = multiqc_dir();
    if !qc_dir.exists() {
        return;
    }

    let engine = DiscoveryEngine::new();
    let frame = engine.run(&[qc_dir]).unwrap();

    let output = std::env::temp_dir().join("popqc_synthetic_test_report.html");

    let config = PopQCConfig {
        project: popqc_core::config::ProjectConfig {
            name: "Synthetic Test".to_string(),
            pipeline: "Test Pipeline".to_string(),
            genome: "GRCh38".to_string(),
            annotation: "Test".to_string(),
            ..Default::default()
        },
        report: popqc_core::config::ReportConfig {
            title: "Integration Test Report".to_string(),
            output: output.clone(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Generate the report
    popqc_report::generate_report(&frame, &config, &output).unwrap();

    // Verify report was created
    assert!(output.exists(), "Report file should exist");

    let content = std::fs::read_to_string(&output).unwrap();
    let size = content.len();

    assert!(
        content.contains("plotly"),
        "Report should include Plotly.js"
    );
    assert!(
        content.contains("PopQC"),
        "Report should have PopQC branding"
    );
    assert!(
        content.contains("sample1"),
        "Report should contain sample data"
    );
    assert!(
        content.contains("Sample Table"),
        "Report should have Sample Table tab"
    );
    assert!(
        content.contains("Explore"),
        "Report should have Explore tab"
    );
    assert!(content.contains("PCA"), "Report should have PCA tab");
    assert!(
        content.contains("Compare"),
        "Report should have Compare tab"
    );

    assert!(size > 100_000, "Report too small: {} bytes", size);
    assert!(size < 100_000_000, "Report too large: {} bytes", size);

    println!(
        "Report generated: {} ({:.1} MB)",
        output.display(),
        size as f64 / 1_048_576.0
    );

    // Cleanup
    std::fs::remove_file(&output).ok();
}
