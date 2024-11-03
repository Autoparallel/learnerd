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

- Extract metadata from academic papers
- Support for multiple sources:
  - arXiv papers
  - IACR ePrints
  - DOI-based papers
- Handle both URLs and direct identifiers

## 📦 Installation 

Add this to your `Cargo.toml`:

```toml
[dependencies]
learner = "0.1.0"
```

## Quick Start

```rust
use learner::Paper;

#[tokio::main]
async fn main() {
    // From arXiv URL
    let paper = Paper::new("https://arxiv.org/abs/2301.07041").await?;
    println!("Title: {}", paper.title);
    
    // From arXiv ID
    let paper = Paper::new("2301.07041").await?;
}
```

> **Note**
> The `Paper::new()` method automatically detects the source type from the input string, handling both URLs and direct identifiers.

## Roadmap

### Phase 1: Core Functionality
- PDF content extraction
- Local database for paper management
- Filesystem daemon for monitoring new papers
- REST API for paper submission and retrieval

> **Warning**
> The PDF extraction feature is experimental and may not work with all paper formats.

### Phase 2: Advanced Features
- LLM-powered paper analysis and summarization
- RSS feed integration for automated paper discovery
- Citation graph analysis
- Automatic tagging and categorization

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request, but please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [arXiv API](https://arxiv.org/help/api/index) for paper metadata
- [IACR](https://eprint.iacr.org/) for cryptography papers
- [CrossRef](https://www.crossref.org/) for DOI resolution

---

<div align="center">
Made for making learning sh*t less annoying.
</div>
