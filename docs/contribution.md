# Contributing to PopQC

We welcome contributions! Whether you are fixing bugs, improving documentation, or adding new parsers, your help is appreciated. Please see the guidelines below.

## Building from Source

### Requirements

- Rust 1.85+ (install via [rustup](https://rustup.rs/))
- Git

### Build

```bash
git clone https://github.com/popqc/popqc.git
cd popqc
cargo build --release
```

The binary will be at `target/release/popqc`.

### Run Tests

```bash
cargo test --workspace
```

### Development Build (faster compilation, slower execution)

```bash
cargo build
./target/debug/popqc --help
```

## Adding a New Parser

1. Add a new entry in `crates/popqc-parsers/src/registry.rs`
2. If it's a MultiQC table format, use `MultiQCTableParser::new("tool_name", &["filename_pattern.txt"])`
3. For custom formats, implement the `QCParser` trait in a new file under `crates/popqc-parsers/src/`
4. Add tests with fixture data