# PopQC User Guide

This guide covers everything you need to know to use PopQC, from required inputs to advanced command-line options and pipeline integrations.

## Input

### QC Data Directory

PopQC accepts **one or more directories** as input. It recursively searches these directories for QC output files from supported tools. No specific directory structure is required. PopQC will find and parse any supported files regardless of how they are organized.

**What PopQC looks for:**

PopQC auto-discovers files by matching filenames against known patterns. The following are examples of files PopQC can parse:

| File Pattern | Source Tool | Workflow |
|---|---|---|
| `multiqc_star.txt` | STAR (via MultiQC) | RNA-seq |
| `multiqc_fastqc.txt` | FastQC (via MultiQC) | Any |
| `multiqc_picard_dups.txt` | Picard MarkDuplicates | Any |
| `multiqc_picard_HsMetrics.txt` | Picard HsMetrics | WES |
| `multiqc_picard_wgsmetrics.txt` | Picard WgsMetrics | WGS |
| `multiqc_salmon.txt` | Salmon | RNA-seq |
| `multiqc_samtools_flagstat.txt` | samtools flagstat | Any |
| `multiqc_samtools_stats.txt` | samtools stats | Any |
| `multiqc_featureCounts.txt` | featureCounts | RNA-seq |
| `multiqc_mosdepth.txt` | Mosdepth | WGS/WES |
| `multiqc_bcftools_stats.txt` | bcftools stats | Variant calling |
| `multiqc_fastp.txt` | fastp | Any |
| `multiqc_rseqc_*.txt` | RSeQC suite | RNA-seq |
| `multiqc_general_stats.txt` | MultiQC general | Any |
| `*.Log.final.out` | STAR (direct) | RNA-seq |

---

**Typical directory structures that work:**
```
results/
├── multiqc/
│   └── multiqc_data/       ← point PopQC here
│       ├── multiqc_star.txt
│       ├── multiqc_fastqc.txt
│       ├── multiqc_picard_dups.txt
│       ├── multiqc_samtools_flagstat.txt
│       ├── multiqc_samtools_stats.txt
│       └── multiqc_general_stats.txt
└── pipeline_info/
```

Or point PopQC at the entire results directory:

```bash
popqc run path/to/results/ -o report.html
```

PopQC will search up to 10 levels deep (configurable with `--max-depth`).

### Metadata File (Optional)

Metadata is **completely optional**. If provided, it enhances the report with:
- Metadata columns visible in the sample table
- Ability to filter samples by any metadata field
- Color PCA plots by any metadata category
- Group samples by metadata for comparison

**Format:** Tab-separated (TSV) or comma-separated (CSV) — auto-detected.

**Requirements:**
- First row must be column headers
- One column must contain sample identifiers (first column by default, or specify with `--sample-col`)
- All other columns are treated as metadata fields
- Any column names work. There are no hardcoded expectations

**Example metadata files:**

*RNA-seq cohort:*
```tsv
sample_id	condition	tissue	batch	sex	age
Sample_001	treated	liver	batch1	M	45
Sample_002	control	liver	batch1	F	38
Sample_003	treated	kidney	batch2	M	52
Sample_004	control	kidney	batch2	F	41
```

*WGS rare disease cohort:*
```tsv
sample	family_id	relationship	affected_status	sex	ethnicity
NA12878	FAM001	proband	affected	female	European
NA12891	FAM001	father	unaffected	male	American
NA12892	FAM001	mother	unaffected	female	Asian
HG002	FAM002	proband	affected	male	Ashkenazi
```

*WES cancer cohort:*
```tsv
SampleName	PatientID	SampleType	Tissue	Stage	TreatmentArm
T001	P001	tumor	breast	III	armA
N001	P001	normal	blood	NA	armA
T002	P002	tumor	lung	II	armB
N002	P002	normal	blood	NA	armB
```

*Single-cell experiment:*
```tsv
library_id	donor	timepoint	stimulation	10x_chemistry
LIB_01	donor1	0h	unstimulated	v3.1
LIB_02	donor1	4h	LPS	v3.1
LIB_03	donor2	0h	unstimulated	v3.1
LIB_04	donor2	4h	LPS	v3.1
```

*Variant calling cohort (no metadata):*
```bash
# No --metadata flag needed. PopQC works without it.
popqc run ./multiqc_data/ -o variant_qc.html
```

**Key points:**
- Column names can be anything — PopQC renders them dynamically
- Values can be any string — they appear as filterable tags in the report
- Missing values (`NA`, `N/A`, `.`, or empty) are treated as absent
- Sample IDs in metadata are matched to QC data using intelligent name normalization (strips common suffixes like `.bam`, `.sorted`, `.markdup`, etc.)

---

## Usage

### Basic Usage

```bash
popqc run <PATH(S)> [OPTIONS]
```

### With Metadata

```bash
popqc run ./results/multiqc/multiqc_data/ \
    --metadata samples.tsv \
    -o cohort_qc_report.html
```

### Full Options

```bash
popqc run ./path/to/qc/files/ \
    --output report.html \
    --metadata metadata.tsv \
    --sample-col "sample_id" \
    --title "My Project QC Report" \
    --pipeline "nf-core/sarek v3.4.0" \
    --genome "GRCh38" \
    --annotation "Ensembl 110" \
    --max-depth 15 \
    --verbose
```

### Command Reference

| Argument | Required | Default | Description |
|---|---|---|---|
| `<PATHS>` | **Yes** | — | One or more directories to search for QC files |
| `-o, --output` | No | `popqc_report.html` | Output HTML report path |
| `-m, --metadata` | No | — | Sample metadata file (TSV or CSV) |
| `--sample-col` | No | First column | Column name in metadata containing sample IDs |
| `--title` | No | `"PopQC Report"` | Report title displayed in the header |
| `--pipeline` | No | — | Pipeline name/version shown in report badge |
| `--genome` | No | — | Reference genome shown in report badge |
| `--annotation` | No | — | Gene annotation shown in report badge |
| `--max-depth` | No | `10` | Maximum directory traversal depth |
| `-v, --verbose` | No | `false` | Enable debug-level logging |

### List Available Parsers

```bash
popqc parsers
```

Output:
```
Available PopQC Parsers (26):

------------------------------------------------------------
Parser ID                 Description
------------------------------------------------------------
  star_log                Parser for STAR aligner Log.final.out summary files
  star                    MultiQC aggregate table parser
  salmon                  MultiQC aggregate table parser
  featurecounts           MultiQC aggregate table parser
  fastqc                  MultiQC aggregate table parser
  picard_dups             MultiQC aggregate table parser
  picard_insertsize       MultiQC aggregate table parser
  picard_alignment        MultiQC aggregate table parser
  picard_wgsmetrics       MultiQC aggregate table parser
  picard_hsmetrics        MultiQC aggregate table parser
  flagstat                MultiQC aggregate table parser
  samstats                MultiQC aggregate table parser
  samtools_idxstats       MultiQC aggregate table parser
  mosdepth                MultiQC aggregate table parser
  qualimap                MultiQC aggregate table parser
  bcftools_stats          MultiQC aggregate table parser
  vcftools                MultiQC aggregate table parser
  snpeff                  MultiQC aggregate table parser
  rseqc_tin               MultiQC aggregate table parser
  rseqc_infer             MultiQC aggregate table parser
  rseqc_bamstat           MultiQC aggregate table parser
  rseqc_readdist          MultiQC aggregate table parser
  rseqc_junction          MultiQC aggregate table parser
  general                 MultiQC aggregate table parser
  fastp                   MultiQC aggregate table parser
  trimgalore              MultiQC aggregate table parser

```

---

## Supported Tools

PopQC supports QC outputs from the following tools, organized by workflow type:

### Universal (Any Workflow)
| Tool | Metrics | Notes |
|---|---|---|
| FastQC | Total sequences, GC%, sequence length, duplication | Pre/post trimming |
| fastp | Filtering stats, adapter content, quality | Trimming QC |
| Picard MarkDuplicates | Duplication rate, library size | PCR duplicate assessment |
| samtools flagstat | Mapped reads, properly paired, singletons | Alignment summary |
| samtools stats | Error rate, insert size, base quality | Detailed alignment stats |
| samtools idxstats | Per-chromosome read counts | Coverage distribution |

### RNA-seq
| Tool | Metrics |
|---|---|
| STAR | Uniquely mapped %, multi-mapped %, unmapped %, splice junctions |
| Salmon | Percent mapped, library type, fragment length |
| featureCounts | Assigned reads, unassigned categories |
| RSeQC TIN | Transcript Integrity Number |
| RSeQC infer_experiment | Strandedness fractions |
| RSeQC bam_stat | Splice reads, proper pairs, MAPQ |
| RSeQC read_distribution | CDS, UTR, intron, intergenic fractions |
| RSeQC junction_annotation | Known/novel splice junctions |

### WGS / WES
| Tool | Metrics |
|---|---|
| Picard WgsMetrics | Mean coverage, % bases ≥ 10x/20x/30x |
| Picard HsMetrics | On-target %, mean target coverage, fold enrichment |
| Mosdepth | Coverage distribution, per-chromosome depth |
| Qualimap | Genome coverage, GC bias, insert size |

### Variant Calling
| Tool | Metrics |
|---|---|
| bcftools stats | SNP count, Ti/Tv ratio, indel distribution |
| VCFtools | Ts/Tv by quality, allele frequencies |
| snpEff | Variant effects, functional impact |

### General
| Tool | Metrics |
|---|---|
| MultiQC general_stats | Aggregated metrics from any MultiQC run |

---

## Report Features

The PopQC HTML report is a single self-contained file that works in any modern web browser. No internet connection is required after the initial page load. All data is embedded directly in the HTML.

The report contains four interactive tabs:

### 1. Sample Table Tab

A fully interactive data table showing all samples and their QC metrics.

**Features:**
- **Search** — Type to filter samples by name
- **Metadata filter** — If metadata is provided, filter by any metadata column/value combination
- **Sort** — Click any column header to sort ascending/descending
- **Select** — Click rows or use checkboxes to select samples
- **Select all** — Checkbox in the header selects all visible (filtered) samples
- **Color coding** — Automatic red/yellow/green highlighting for key metrics:
  - Mapping rates (red < 50%, yellow < 70%, green ≥ 70%)
  - Duplication rates (red > 80%, yellow > 60%, green ≤ 60%)
  - Assignment rates (red < 30%, yellow < 50%, green ≥ 50%)
- **Export CSV** — Download all data or only selected samples as CSV

### 2. Multi-Filter Explore Tab

Apply multiple QC thresholds simultaneously to identify problematic samples in a large cohort

**Features:**
- **Add filters** — Create multiple filter rules, each with a metric, min threshold, and/or max threshold
- **Run Filters** — Identify all samples failing ANY of the defined thresholds
- **Per-metric plots** — Each filter generates its own scatter plot in a responsive grid:
  - Blue dots = passing samples
  - Red dots = flagged samples
  - Red dashed lines = threshold boundaries
  - Green shaded region = acceptable range
- **Fullscreen view** — Click the ⛶ button on any plot to view it fullscreen with:
  - Full sample labels on X-axis
  - Larger markers
  - Full Plotly interactivity (zoom, pan, hover)
  - Close with ✕, clicking backdrop, or pressing Escape
- **Summary statistics** — Shows total flagged count and per-sample failure details
- **Failure table** — Lists all flagged samples with:
  - Number of filters failed (with visual progress bar)
  - Specific failure details (which metric, what value)
- **Select flagged** — Add all flagged samples to the table selection for CSV export

### 3. PCA Tab

Principal Component Analysis for detecting batch effects, outliers, and sample clustering.

**Features:**
- **Feature selection** — Choose which QC metrics to include in PCA (all selected by default)
- **Select All / None** — Quickly toggle all features
- **Color by** — If metadata is provided, color points by any metadata field to visualize:
  - Batch effects
  - Condition separation
  - Tissue/cell type clustering
  - Any other grouping variable
- **Computation** — PCA is computed entirely in the browser using Jacobi eigenvalue decomposition of the covariance matrix
- **Variance explained** — Shows PC1 and PC2 variance percentages
- **Sample count** — Reports how many samples had complete data for selected features
- **Interactive plot** — Hover for sample names, zoom, pan

### 4. Group Comparison Tab

Define custom sample groups and compare their QC metric distributions.

**Features:**
- **Create groups** — Name groups anything (e.g., "Good", "Bad", "Batch1", "Batch2", "Treatment", "Control")
- **Multi-select dropdown** — Click to open a searchable dropdown with checkmarks:
  - Check/uncheck samples to add/remove from group
  - Search by sample name within the dropdown
  - Dropdown stays open while selecting multiple samples
  - Selected samples shown as removable tags below
- **Comparison modes:**
  - **Group (Box Plot)** — Box plots showing distribution of a metric across groups, with all individual points overlaid
  - **Sample (Bar Chart)** — Per-sample bar chart for direct 1-vs-1 or few-vs-few comparison
- **Metric selection** — Compare any QC metric across groups
- **Interactive plots** — Full Plotly interactivity (hover, zoom, export PNG)

---

## Output

PopQC generates a single self-contained HTML file:

```
popqc_report.html     # Interactive HTML report (typically 2-100 MB depending on cohort size)
```


---

## Configuration

### Sample Name Normalization

PopQC automatically strips common file suffixes to match sample names across different QC tools. For example, all of these are normalized to `Sample_001`:

```
Sample_001.Aligned.sortedByCoord.out.bam
Sample_001.markdup.sorted.bam
Sample_001.sorted.cram
Sample_001_R1_001.fastq.gz
Sample_001.Log.final.out
Sample_001.flagstat
Sample_001.CollectWgsMetrics.coverage_metrics
Sample_001_mosdepth.global.dist
```

The full list of stripped suffixes includes patterns from RNA-seq, WGS, WES, and variant calling pipelines.

### Metadata Sample Matching

When metadata is provided, PopQC matches metadata rows to QC samples by:
1. Normalizing the metadata sample ID column using the same suffix-stripping rules
2. Attempting exact match first
3. If no match, trying the raw (un-normalized) metadata value

This means your metadata file can use short sample names (e.g., `Sample_001`) even if the QC files use long names (e.g., `Sample_001.markdup.sorted.bam`).

---

## Pipeline Integration

### Nextflow

```nextflow
process POPQC {
    publishDir "${params.outdir}/popqc", mode: 'copy'

    input:
    path multiqc_data
    path metadata

    output:
    path "popqc_report.html"

    script:
    def meta_arg = metadata ? "--metadata ${metadata}" : ""
    """
    popqc run ${multiqc_data} \
        ${meta_arg} \
        --title "${params.project_name} QC" \
        --pipeline "${workflow.manifest.name} ${workflow.manifest.version}" \
        --genome "${params.genome}" \
        -o popqc_report.html
    """
}
```

### Snakemake

```python
rule popqc:
    input:
        qc_dir="results/multiqc/multiqc_data",
        metadata="config/samples.tsv"
    output:
        "results/popqc/qc_report.html"
    shell:
        """
        popqc run {input.qc_dir} \
            --metadata {input.metadata} \
            --title "Project QC Report" \
            -o {output}
        """
```
