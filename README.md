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

### Daemon Management

The `learnerd` daemon can run in the background to handle tasks like paper monitoring and updates. It can be run directly or installed as a system service.

#### Basic Usage

```bash
# Start the daemon
learnerd daemon start

# Check daemon status
learnerd daemon status

# Stop the daemon
learnerd daemon stop

# Restart the daemon
learnerd daemon restart
```

### System Service Installation

#### Linux (systemd)
```bash
# Install the service
sudo learnerd daemon install

# Enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable learnerd
sudo systemctl start learnerd

# Verify it's running
sudo systemctl status learnerd

# View logs
sudo journalctl -u learnerd

# Service management
sudo systemctl stop learnerd     # Stop the service
sudo systemctl start learnerd    # Start the service
sudo systemctl restart learnerd  # Restart the service

# Remove the service
sudo systemctl stop learnerd
sudo learnerd daemon uninstall
sudo systemctl daemon-reload
```

#### macOS (launchd)
```bash
# Install service files
sudo learnerd daemon install

# Load and start the service
sudo launchctl load /Library/LaunchDaemons/learnerd.daemon.plist

# Verify it's running
sudo launchctl list | grep learnerd

# Service management
sudo launchctl bootout system/learnerd.daemon        # Stop
sudo launchctl bootstrap system /Library/LaunchDaemons/learnerd.daemon.plist  # Start
sudo launchctl kickstart -k system/learnerd.daemon   # Restart

# Remove the service completely
sudo pkill learnerd && sudo launchctl bootout system/learnerd.daemon # stop and remove from bootlist
sudo learnerd daemon uninstall # remove service file all together
```

Important Notes:
- The service is not automatically started after installation
- You must explicitly `load` the service to activate it
- Once loaded, it will start automatically on system boot
- If you `bootout` the service, you must `load` again to reactivate it
- The `kickstart` command only works if the service is currently loaded

#### Log Files

The daemon writes logs to the following locations:

- Linux: `/var/log/learnerd/`
- macOS: `/Library/Logs/learnerd/`

Log files include:
- `learnerd.log` - Main application log with rotation
- `stdout.log` - Standard output
- `stderr.log` - Standard error

View the logs:
```bash
# View main log
tail -f /var/log/learnerd/learnerd.log   # Linux
tail -f /Library/Logs/learnerd/learnerd.log   # macOS

# View last 100 lines with timestamps
tail -n 100 /var/log/learnerd/learnerd.log
```

#### Common Issues

1. **Permission Denied**
   ```bash
   # Ensure correct permissions on log directory
   sudo chown -R root:root /var/log/learnerd   # Linux
   sudo chown -R root:wheel /Library/Logs/learnerd   # macOS
   ```

2. **Service Won't Start**
   ```bash
   # Check system logs
   sudo journalctl -u learnerd.service   # Linux
   sudo log show --predicate 'processImagePath contains "learnerd"'   # macOS
   ```

3. **PID File Issues**
   ```bash
   # If daemon won't start due to stale PID file
   sudo rm /var/run/learnerd.pid   # Linux
   sudo rm "/Library/Application Support/learnerd/learnerd.pid"   # macOS
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

## Development

This project uses [just](https://github.com/casey/just) as a command runner. Install it with:

```bash
cargo install just
```

### Common Commands

```bash
# Setup development environment (install dependencies and targets)
just setup

# Build all targets
just build-all

# Run tests
just test

# Format code
just fmt

# Run linter
just lint

# Run all CI checks locally
just ci

# Show available commands
just
```

### Building for Specific Targets

```bash
# Build for x86_64 Linux
just build-x86-linux

# Build for ARM64 macOS
just build-arm-mac
```

### System Requirements

The setup command will attempt to install required system dependencies, but if you need to install them manually:

#### Linux (Debian/Ubuntu)
```bash
sudo apt-get install pkg-config libssl-dev
```

#### macOS
```bash
brew install openssl@3
export OPENSSL_DIR=$(brew --prefix openssl@3)  # Add to your shell profile
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