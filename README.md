<div align="center">

# learner

*A Rust-powered academic research management system*

[![Crates.io](https://img.shields.io/crates/v/learner)](https://crates.io/crates/learner)
[![docs.rs](https://img.shields.io/docsrs/learner)](https://docs.rs/learner)
[![CI](https://github.com/autoparallel/learner/actions/workflows/check.yaml/badge.svg)](https://github.com/autoparallel/learner/actions/workflows/check.yaml)
[![License](https://img.shields.io/crates/l/learner)](LICENSE)

<img src="assets/header.svg" alt="learner header" width="600px">

</div>

## Features

- 📚 Academic Paper Management
  - Extract metadata from multiple sources (arXiv, IACR, DOI)
  - Support for both URLs and direct identifiers
  - Automatic source detection
  - Full paper metadata including authors, abstracts, and publication dates

- 🔍 Local Database Management
  - SQLite-based storage for offline access
  - Full-text search capabilities
  - Case-insensitive title search
  - Duplicate detection and handling
  - Platform-specific default locations

- 🚀 Command Line Interface (`learnerd`)
  - Interactive database management
  - Paper addition and retrieval
  - Search functionality
  - Beautiful, colored output
  - Detailed logging options

## Installation

### Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
learner = "0.1"
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
    // Initialize database
    let db = Database::open("papers.db").await?;
    
    // Add papers from various sources
    let paper = Paper::new("https://arxiv.org/abs/2301.07041").await?;
    paper.save(&db).await?;
    
    let paper = Paper::new("10.1145/1327452.1327492").await?;  // From DOI
    paper.save(&db).await?;
    
    let paper = Paper::new("2023/123").await?;  // From IACR
    paper.save(&db).await?;
    
    Ok(())
}
```

### CLI Usage

```bash
# Initialize a new database
learnerd init

# Add a paper (auto-detects source)
learnerd add 2301.07041
learnerd add "https://arxiv.org/abs/2301.07041"
learnerd add "10.1145/1327452.1327492"

# Retrieve a paper
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
   - Source-specific clients (arXiv, IACR, DOI)
   - Error handling

2. `learnerd` - CLI application offering:
   - User-friendly interface
   - Interactive confirmations
   - Colored output
   - Logging and debugging capabilities

## Roadmap

### Phase 1: Core Improvements ⏳
- [ ] Paper removal functionality
- [ ] Batch paper operations
- [ ] Export capabilities
- [ ] Enhanced search features
- [ ] Custom metadata fields

### Phase 2: Advanced Features 🔮
- [ ] PDF content extraction
- [ ] LLM-powered paper analysis
- [ ] Citation graph analysis
- [ ] Automated paper discovery
- [ ] Web interface

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Before making major changes, please open an issue first to discuss what you would like to change.

### Development Setup

1. Clone the repository
2. Install dependencies:
   ```bash
   cargo build
   ```
3. Run tests:
   ```bash
   cargo test
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