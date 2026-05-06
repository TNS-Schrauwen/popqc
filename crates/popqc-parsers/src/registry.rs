//! Parser registry - manages all available parsers
use crate::multiqc_table::MultiQCTableParser;
use crate::star::StarLogParser;
use popqc_core::traits::QCParser;
use std::path::Path;
use std::sync::Arc;
pub struct ParserRegistry {
    parsers: Vec<Arc<dyn QCParser>>,
}

impl ParserRegistry {
    #[must_use]
    pub fn with_defaults() -> Self {
        let parsers: Vec<Arc<dyn QCParser>> = vec![
            Arc::new(StarLogParser::new()),
            Arc::new(MultiQCTableParser::new("star", &["multiqc_star.txt"])),
            Arc::new(MultiQCTableParser::new("salmon", &["multiqc_salmon.txt"])),
            Arc::new(MultiQCTableParser::new(
                "featurecounts",
                &["multiqc_featureCounts.txt"],
            )),
            Arc::new(MultiQCTableParser::new("fastqc", &["multiqc_fastqc.txt"])),
            Arc::new(MultiQCTableParser::new(
                "picard_dups",
                &["multiqc_picard_dups.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "picard_insertsize",
                &["multiqc_picard_insertSize.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "picard_alignment",
                &["multiqc_picard_AlignmentSummaryMetrics.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "picard_wgsmetrics",
                &["multiqc_picard_wgsmetrics.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "picard_hsmetrics",
                &["multiqc_picard_HsMetrics.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "flagstat",
                &["multiqc_samtools_flagstat.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "samstats",
                &["multiqc_samtools_stats.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "samtools_idxstats",
                &["multiqc_samtools_idxstats.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "mosdepth",
                &["multiqc_mosdepth.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "qualimap",
                &["multiqc_qualimap_bamqc_genome_results.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "bcftools_stats",
                &["multiqc_bcftools_stats.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "vcftools",
                &["multiqc_vcftools.txt"],
            )),
            Arc::new(MultiQCTableParser::new("snpeff", &["multiqc_snpeff.txt"])),
            Arc::new(MultiQCTableParser::new(
                "rseqc_tin",
                &["multiqc_rseqc_tin.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "rseqc_infer",
                &["multiqc_rseqc_infer_experiment.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "rseqc_bamstat",
                &["multiqc_rseqc_bam_stat.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "rseqc_readdist",
                &["multiqc_rseqc_read_distribution.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "rseqc_junction",
                &["multiqc_rseqc_junction_annotation.txt"],
            )),
            Arc::new(MultiQCTableParser::new(
                "general",
                &["multiqc_general_stats.txt"],
            )),
            Arc::new(MultiQCTableParser::new("fastp", &["multiqc_fastp.txt"])),
            Arc::new(MultiQCTableParser::new(
                "trimgalore",
                &["multiqc_trimgalore.txt", "multiqc_cutadapt.txt"],
            )),
        ];

        Self { parsers }
    }

    #[must_use]
    pub fn find_parsers(&self, path: &Path) -> Vec<Arc<dyn QCParser>> {
        self.parsers
            .iter()
            .filter(|p| p.can_parse(path))
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn all_parsers(&self) -> &[Arc<dyn QCParser>] {
        &self.parsers
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.parsers.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.parsers.is_empty()
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
