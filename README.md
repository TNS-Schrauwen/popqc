# PopQC

<div align="center">
  <img src="img/popqc_logo.png" alt="PopQC Logo" width="200" />
</div>

**Population-level Quality Control and QC Outlier Detection for Large Cohort Genomics Datasets**

[![CI](https://github.com/TNS-Schrauwen/popqc/actions/workflows/ci.yml/badge.svg)](https://github.com/TNS-Schrauwen/popqc/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

PopQC is a fast, scalable QC reporting tool that generates interactive HTML reports for quality control analysis of large genomics datasets. It can process QC logs from **any genomics workflow** including RNA-seq, WGS, WES, variant calling, single-cell, long-read sequencing, and more.

PopQC auto-discovers QC outputs from common bioinformatics tools (e.g. MultiQC) and produces a single self-contained HTML report with interactive exploration capabilities designed for cohorts of 100 to 10,000+ samples.

---

## Motivation

Existing QC reporting tools (e.g., MultiQC) become visually overwhelming and difficult to navigate when cohort sizes exceed ~100 samples. Researchers working with large population cohorts face major issues:

1. MultiQC Reports become cluttered and hard to interpret for sample size 100+
2. Identifying failing or outlier samples requires manual inspection
3. No interactive filtering, clustering, or group comparison capabilities
4. Static reports do not support exploratory analysis
5. No built-in support for cohort-wide QC distribution tracking

PopQC solves these problems by generating interactive, self-contained HTML reports with **dynamic filtering, PCA visualization, multi-metric threshold exploration, QC outlier analysis, and group comparison** — all computed in the browser with no server required.

---

## Installation

### Pre-built Binaries (Recommended)

Download the latest release for your platform from the [Releases page](https://github.com/TNS-Schrauwen/popqc/releases):

```bash
# Linux (x86_64)
curl -LO https://github.com/popqc/popqc/releases/latest/download/popqc-linux-x86_64.tar.gz
tar -xzf popqc-linux-x86_64.tar.gz
sudo mv popqc /usr/local/bin/

# Verify installation
popqc --version
```


# Install on HPC

### For HPC environments where you do not have sudo access:

```bash
# Download the Linux binary
curl -LO https://github.com/TNS-Schrauwen/popqc/releases/latest/download/popqc-linux-x86_64.tar.gz
tar -xzf popqc-linux-x86_64.tar.gz

# Move to a directory in your PATH (e.g., ~/bin)
mkdir -p ~/bin
mv popqc ~/bin/

# Add to PATH if not already (add to ~/.bashrc for persistence)
export PATH="$HOME/bin:$PATH"
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc

# Verify
popqc --version
```

### Install from Source (requires Rust 1.85+)

```bash
# Install Rust (no root required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Clone and build
git clone https://github.com/TNS-Schrauwen/popqc.git
cd popqc
cargo build --release

# The binary is at: target/release/popqc
# Optionally copy to your PATH:
cp target/release/popqc ~/bin/
```

### Install via Cargo

```bash
cargo install --git https://github.com/TNS-Schrauwen/popqc.git
```

### Conda / Bioconda (coming soon)

```bash
conda install -c bioconda popqc
```

---

## Quick Start

```bash
# Generate a report from a MultiQC data directory
popqc run path/to//multiqc/data/ -o qc_report.html

# With sample metadata
popqc run path/to/multiqc/data/ --metadata path/to/metadata/samples.tsv -o qc_report.html

```

---

# Usage

## Command: popqc run

Auto-discover QC files and generate an interactive report.

```bash
Usage: popqc run [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  Path(s) to search for QC files

Options:
  -o, --output <OUTPUT>          Output HTML report path [default: popqc_report.html]
  -m, --metadata <METADATA>      Sample metadata file (TSV or CSV). Optional.
                                  First column is used as sample identifier by default.
                                  All other columns become metadata fields in the report.
      --sample-col <SAMPLE_COL>  Column name in metadata file containing sample IDs.
                                  If not specified, the first column is used.
      --title <TITLE>            Report title [default: "PopQC Report"]
      --pipeline <PIPELINE>      Pipeline name (e.g., "nf-core/rnaseq v3.14.0") [default: ""]
      --genome <GENOME>          Reference genome (e.g., "GRCh38") [default: ""]
      --annotation <ANNOTATION>  Annotation (e.g., "GENCODE v44") [default: ""]
      --max-depth <MAX_DEPTH>    Maximum directory traversal depth [default: 10]
  -v, --verbose                  Verbose output
  -h, --help                     Print help
```

## Command: popqc parsers

List all available built-in QC tool parsers:

```bash
popqc parsers
```

## Full Usage Example

```bash
popqc run \
    path/to/multiqc/data/ \
    --metadata path/to/metadata/samples.tsv \
    --sample-col "sample_id" \
    --title "My Cohort RNA-seq QC Report" \
    --pipeline "nf-core/rnaseq v3.14.0" \
    --genome "GRCh38" \
    --annotation "GENCODE v44" \
    --max-depth 10 \
    --output path/to/output/dir/cohort_qc_report.html \
    --verbose
```

## Multiple Input Directories

```bash
popqc run path/to/batch1/multiqc_data/ path/to/batch2/multiqc_data/ path/to/batch3/multiqc_data/ \
    --metadata path/to/metadata/all_samples.tsv \
    --title "Multi-Batch QC Report" \
    -o path/to/output/dir/multi_batch_report.html
```


## Documentation

For detailed usage instructions, please refer to our documentation:

- [**User Guide**](docs/user_guide.md): Detailed information on input files, metadata formats, CLI options, report features, and examples.
- [**Contribution Guide**](docs/contribution.md): Instructions for building from source, running tests, and adding new custom parsers.

---

## License

MIT License. See [LICENSE](LICENSE) for details.




