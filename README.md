<div align="center">

# learner
*A Rust-powered academic research management system*

[![Library](https://img.shields.io/badge/lib-learner-blue)](https://crates.io/crates/learner)
[![Crates.io](https://img.shields.io/crates/v/learner?color=orange)](https://crates.io/crates/learner)
[![docs.rs](https://img.shields.io/docsrs/learner)](https://docs.rs/learner)
[![Downloads](https://img.shields.io/crates/d/learner.svg)](https://crates.io/crates/learner)
&nbsp;&nbsp;|&nbsp;&nbsp;
[![CLI](https://img.shields.io/badge/cli-learnerd-blue)](https://crates.io/crates/learnerd)
[![Crates.io](https://img.shields.io/crates/v/learnerd?color=orange)](https://crates.io/crates/learnerd)
[![Downloads](https://img.shields.io/crates/d/learnerd.svg)](https://crates.io/crates/learnerd)

[![CI](https://github.com/autoparallel/learner/actions/workflows/check.yaml/badge.svg)](https://github.com/autoparallel/learner/actions/workflows/check.yaml)
[![License](https://img.shields.io/crates/l/learner)](LICENSE)

<img src="assets/header.svg" alt="learner header" width="600px">

</div>

## Features

- Academic Paper Management
  - Extract metadata from multiple sources (arXiv, IACR, DOI)
  - Support for both URLs and direct identifiers
  - Automatic source detection
  - Full paper metadata including authors, abstracts, and publication dates

- Local Database Management
  - SQLite-based storage for offline access
  - Full-text search capabilities
  - Case-insensitive title search
  - Duplicate detection and handling
  - Platform-specific default locations
  - PDF management with configurable storage location

- Command Line Interface (`learnerd`)
  - Interactive database management
  - Paper addition and retrieval
  - Search functionality
  - PDF downloading and management
  - Beautiful, colored output
  - Detailed logging options

## Installation

### Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
learner = "0.2"  # Core library
```

### CLI Tool

```bash
cargo install learnerd
```

## Usage

### Library Usage

```rust
use learner::{Paper, Database};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database with default paths
    let db = Database::open(Database::default_path()).await?;
    
    // Add papers from various sources
    let paper = Paper::new("https://arxiv.org/abs/2301.07041").await?;
    paper.save(&db).await?;
    
    // Download PDF if available
    let pdf_dir = Database::default_pdf_path();
    paper.download_pdf(pdf_dir).await?;
    
    // Add papers from other sources
    let paper = Paper::new("10.1145/1327452.1327492").await?;  // From DOI
    let paper = Paper::new("2023/123").await?;                 // From IACR
    
    Ok(())
}
```

### CLI Usage

```bash
# Initialize a new database (interactive)
learnerd init

# Add a paper (auto-detects source)
learnerd add 2301.07041
learnerd add "https://arxiv.org/abs/2301.07041"
learnerd add "10.1145/1327452.1327492"

# Skip PDF download
learnerd add 2301.07041 --no-pdf

# Download PDF for existing paper
learnerd download arxiv 2301.07041

# Retrieve paper details
learnerd get arxiv 2301.07041

# Search papers
learnerd search "neural networks"

# Verbose output for debugging
learnerd -v add 2301.07041

# Clean up database (with confirmation)
learnerd clean
```

## Project Structure

The project consists of two main components:

1. `learner` - Core library providing:
   - Paper metadata extraction
   - Database management
   - PDF download capabilities
   - Source-specific clients (arXiv, IACR, DOI)
   - Error handling

2. `learnerd` - CLI application offering:
   - User-friendly interface
   - PDF management
   - Interactive confirmations
   - Colored output
   - Logging and debugging capabilities

## Roadmap

### Phase 1: Core Improvements 
- [x] PDF management
- [ ] PDF content extraction
- [ ] DB/Paper removal functionality
- [ ] Batch paper operations
- [ ] Export capabilities
- [ ] Enhanced search features
- [ ] Custom metadata fields

### Phase 2: Advanced Features 
- [ ] LLM-powered paper analysis
- [ ] PDF daemon for paper versioning and annotations
- [ ] Automated paper discovery
- [ ] Citation graph analysis
- [ ] Web interface

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Before making major changes, please open an issue first to discuss what you would like to change.

### Continuous Integration
The project maintains code quality through automated CI workflows:

- Code Formatting
  - rustfmt: Enforces consistent Rust code formatting
  - taplo: Ensures TOML files (like Cargo.toml) follow consistent style

- Code Quality
  - clippy: Rust's official linter for catching common mistakes and enforcing best practices
  - cargo-udeps: Identifies unused dependencies to keep the project lean


- Testing
  - Runs the full test suite across all workspace members
  - [ ] TODO: Check cross-platform 

- Release Safety
  - cargo-semver-checks: Verifies that version bumps follow semantic versioning rules
  - Prevents accidental breaking changes in minor/patch releases

All CI checks must pass before merging pull requests, maintaining consistent quality across contributions.

### Development Setup

1. Clone the repository
2. Install dependencies:
   ```bash
   cargo build
   ```
3. Run tests:
   ```bash
   cargo test --workspace
   ```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [arXiv API](https://arxiv.org/help/api/index) for paper metadata
- [IACR](https://eprint.iacr.org/) for cryptography papers
- [CrossRef](https://www.crossref.org/) for DOI resolution
- [SQLite](https://www.sqlite.org/) for local database support

---

<div align="center">
Made for making learning sh*t less annoying.
</div>