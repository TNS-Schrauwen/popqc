//! `PopQC` Parsers - Built-in parsers for common QC tools

pub mod multiqc_table;
pub mod registry;
pub mod star;

pub use registry::ParserRegistry;
