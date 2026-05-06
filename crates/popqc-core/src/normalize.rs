//! Sample name normalization strip common suffixes

#![allow(clippy::unnecessary_sort_by)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleNameNormalizer {
    strip_suffixes: Vec<String>,
}

impl Default for SampleNameNormalizer {
    fn default() -> Self {
        let mut suffixes = vec![
            "_raw Read 1",
            "_raw Read 2",
            "_raw",
            " Read 1",
            " Read 2",
            ".infer_experiment",
            ".read_distribution",
            ".bam_stat",
            ".junction_annotation",
            ".inner_distance",
            ".tin",
            ".markdup.sorted.bam",
            ".markdup.sorted",
            ".dedup.sorted.bam",
            ".dedup.sorted",
            ".Aligned.sortedByCoord.out.bam",
            ".Aligned.sortedByCoord.out",
            ".Aligned.toTranscriptome.out.bam",
            ".Aligned.toTranscriptome.out",
            ".Aligned.out.bam",
            ".Aligned.out",
            ".sorted.bam",
            ".sorted.cram",
            ".sorted",
            ".recal.bam",
            ".recal.cram",
            ".bqsr.bam",
            ".bam",
            ".cram",
            ".Log.final.out",
            ".flagstat",
            ".idxstats",
            ".stats",
            "_fastqc",
            "_qualimap",
            "_mosdepth",
            "_wgsmetrics",
            "_hsmetrics",
            "_multiplemetrics",
            "_tidditcov.wig",
            "_mosdepth.global.dist",
            "_mosdepth.per-base.d4",
            "_mosdepth.summary",
            "_val_1",
            "_val_2",
            "_trimmed",
            ".trimmed",
            "_paired",
            "_unpaired",
            "_R1_001",
            "_R2_001",
            "_R1",
            "_R2",
            "_1",
            "_2",
            ".fq.gz",
            ".fastq.gz",
            ".fq",
            ".fastq",
            ".vcf.gz",
            ".vcf",
            ".g.vcf.gz",
            ".g.vcf",
            ".bed.gz",
            ".bed",
            ".txt",
            ".csv",
            ".tsv",
            "_snv",
            "_sv_merge",
            "_repeat_expansion",
            ".CollectHsMetrics.coverage_metrics",
            ".CollectWgsMetrics.coverage_metrics",
            ".CollectMultipleMetrics.alignment_summary_metrics",
            ".CollectMultipleMetrics.insert_size_metrics",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        suffixes.sort_by(|a, b| b.len().cmp(&a.len()));
        Self {
            strip_suffixes: suffixes,
        }
    }
}

impl SampleNameNormalizer {
    #[must_use]
    pub fn with_additional_suffixes(additional: &[&str]) -> Self {
        let mut norm = Self::default();
        for s in additional {
            norm.strip_suffixes.push((*s).to_string());
        }
        norm.strip_suffixes.sort_by(|a, b| b.len().cmp(&a.len()));
        norm
    }

    #[must_use]
    pub fn with_suffixes_only(suffixes: Vec<String>) -> Self {
        let mut s = suffixes;
        s.sort_by(|a, b| b.len().cmp(&a.len()));
        Self { strip_suffixes: s }
    }

    #[must_use]
    pub fn normalize(&self, name: &str) -> String {
        let mut s = name.trim().to_string();
        if let Some(pos) = s.rfind('/') {
            s = s[pos + 1..].to_string();
        }
        if let Some(pos) = s.rfind('\\') {
            s = s[pos + 1..].to_string();
        }

        let mut changed = true;
        while changed {
            changed = false;
            for suffix in &self.strip_suffixes {
                if s.ends_with(suffix.as_str()) {
                    s.truncate(s.len() - suffix.len());
                    changed = true;
                    break;
                }
            }
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_star_output() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(
            norm.normalize("SampleA.Aligned.sortedByCoord.out.bam"),
            "SampleA"
        );
    }

    #[test]
    fn test_normalize_wgs_bam() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(norm.normalize("NA12878.markdup.sorted.bam"), "NA12878");
    }

    #[test]
    fn test_normalize_wes_dedup() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(norm.normalize("Patient01.dedup.sorted.bam"), "Patient01");
    }

    #[test]
    fn test_normalize_mosdepth() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(norm.normalize("HG001_mosdepth.global.dist"), "HG001");
    }

    #[test]
    fn test_normalize_picard_metrics() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(
            norm.normalize("Sample1.CollectWgsMetrics.coverage_metrics"),
            "Sample1"
        );
    }

    #[test]
    fn test_normalize_path_stripping() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(
            norm.normalize("/data/results/qc_bam/Patient01.flagstat"),
            "Patient01"
        );
    }

    #[test]
    fn test_normalize_cram() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(norm.normalize("Sample99.sorted.cram"), "Sample99");
    }

    #[test]
    fn test_normalize_multiqc_raw_read1() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(
            norm.normalize("F005P001-100m_515406902_raw Read 1"),
            "F005P001-100m_515406902"
        );
    }

    #[test]
    fn test_normalize_multiqc_raw_read2() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(
            norm.normalize("UHR_Control_TGEN_Box_6-100m-100m_548706175_raw Read 2"),
            "UHR_Control_TGEN_Box_6-100m-100m_548706175"
        );
    }

    #[test]
    fn test_normalize_multiqc_raw() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(
            norm.normalize("F005P001-100m_515406902_raw"),
            "F005P001-100m_515406902"
        );
    }

    #[test]
    fn test_normalize_rseqc_infer_experiment() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(
            norm.normalize("F005P001-100m_515406902.infer_experiment"),
            "F005P001-100m_515406902"
        );
    }

    #[test]
    fn test_normalize_rseqc_read_distribution() {
        let norm = SampleNameNormalizer::default();
        assert_eq!(norm.normalize("SampleX.read_distribution"), "SampleX");
    }
}
