//! Command line interface and daemon for the learner paper management system.
//!
//! This crate provides a CLI tool for managing academic papers using the `learner` library.
//! It supports operations like:
//! - Database initialization and management
//! - Paper addition and retrieval
//! - Full-text search across papers
//! - Database maintenance and cleanup
//!
//! # Usage
//!
//! ```bash
//! # Initialize a new database
//! learnerd init
//!
//! # Add a paper by its identifier
//! learnerd add 2301.07041
//!
//! # Retrieve a paper
//! learnerd get arxiv 2301.07041
//!
//! # Search for papers
//! learnerd search "neural networks"
//!
//! # Clean up the database
//! learnerd clean
//! ```
//!
//! The CLI provides colored output and interactive confirmations for destructive
//! operations. It also supports various verbosity levels for debugging through
//! the `-v` flag.

#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use std::{path::PathBuf, str::FromStr};

use clap::{builder::ArgAction, Parser, Subcommand};
use console::{style, Emoji};
use errors::LearnerdErrors;
use learner::{
  database::Database,
  paper::{Paper, Source},
};
use tracing::{debug, trace};
use tracing_subscriber::EnvFilter;

pub mod errors;

// Emoji constants for prettier output
/// Search operation indicator
static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
/// Database/library operations indicator
static BOOKS: Emoji<'_, '_> = Emoji("📚 ", "");
/// Initialization/startup indicator
static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");
/// Paper details indicator
static PAPER: Emoji<'_, '_> = Emoji("📄 ", "");
/// Save operation indicator
static SAVE: Emoji<'_, '_> = Emoji("💾 ", "");
/// Warning indicator
static WARNING: Emoji<'_, '_> = Emoji("⚠️  ", "");
/// Success indicator
static SUCCESS: Emoji<'_, '_> = Emoji("✨ ", "");

/// Command line interface configuration and argument parsing
#[derive(Parser)]
#[command(author, version, about = "Daemon and CLI for the learner paper management system")]
struct Cli {
  /// Verbose mode (-v, -vv, -vvv) for different levels of logging detail
  #[arg(
        short,
        long,
        action = ArgAction::Count,
        global = true,
        help = "Increase logging verbosity"
    )]
  verbose: u8,

  /// Path to the database file. This is where the database will be created or referenced from. If
  /// not specified, uses the default platform-specific data directory.
  #[arg(long, short, global = true)]
  path: Option<PathBuf>,

  /// The subcommand to execute
  #[command(subcommand)]
  command: Commands,
}

/// Available commands for the CLI
#[derive(Subcommand)]
enum Commands {
  /// Initialize a new learner database
  Init,
  /// Add a paper to the database by its identifier
  Add {
    /// Paper identifier (arXiv ID, DOI, or IACR ID)
    /// Examples: "2301.07041", "10.1145/1327452.1327492"
    identifier: String,

    /// Skip PDF download prompt
    #[arg(long)]
    no_pdf: bool,
  },
  /// Remove a paper from the database by its source and identifier
  Remove {
    /// Source system (arxiv, doi, iacr)
    #[arg(value_enum)]
    source:     Source,
    /// Paper identifier in the source system
    identifier: String,
  },
  /// Retrieve and display a paper's details
  Get {
    /// Source system (arxiv, doi, iacr)
    #[arg(value_enum)]
    source:     Source,
    /// Paper identifier in the source system
    identifier: String,
  },
  /// Search papers in the database
  Search {
    /// Search query - supports full text search
    query: String,
  },
  /// Removes the entire database after confirmation
  Clean {
    /// Skip confirmation prompts
    #[arg(long, short)]
    force: bool,
  },
}

/// Configures the logging system based on the verbosity level
///
/// # Arguments
///
/// * `verbosity` - Number of times the verbose flag was used (0-3)
///
/// The verbosity levels are:
/// - 0: warn (default)
/// - 1: info
/// - 2: debug
/// - 3+: trace
fn setup_logging(verbosity: u8) {
  let filter = match verbosity {
    0 => "warn",
    1 => "info",
    2 => "debug",
    _ => "trace",
  };

  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));

  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_target(true)
    .init();
}

/// Entry point for the learnerd CLI application
///
/// Handles command line argument parsing, sets up logging, and executes
/// the requested command. All commands provide colored output and
/// interactive confirmations for destructive operations.
///
/// # Errors
///
/// Returns `LearnerdErrors` for various failure conditions including:
/// - Database operations failures
/// - Paper fetching failures
/// - File system errors
/// - User interaction errors
#[tokio::main]
async fn main() -> Result<(), LearnerdErrors> {
  let cli = Cli::parse();
  setup_logging(cli.verbose);

  match cli.command {
    Commands::Init => {
      let db_path = cli.path.unwrap_or_else(|| {
        let default_path = Database::default_path();
        println!(
          "{} Using default database path: {}",
          style(BOOKS).cyan(),
          style(default_path.display()).yellow()
        );
        default_path
      });

      if db_path.exists() {
        println!(
          "{} Database already exists at: {}",
          style(WARNING).yellow(),
          style(db_path.display()).yellow()
        );

        // First confirmation with proper prompt
        let confirm = dialoguer::Confirm::new()
          .with_prompt(
            "Do you want to reinitialize this database? This will erase all existing data",
          )
          .default(false)
          .interact()?;

        if !confirm {
          println!("{} Keeping existing database", style("ℹ").blue());
          return Ok(());
        }

        // Require typing INIT for final confirmation
        let input = dialoguer::Input::<String>::new()
          .with_prompt(&format!(
            "{} Type {} to confirm reinitialization",
            style("⚠️").red(),
            style("INIT").red().bold()
          ))
          .interact_text()?;

        if input != "INIT" {
          println!("{} Operation cancelled, keeping existing database", style("ℹ").blue());
          return Ok(());
        }

        // Remove existing database
        println!("{} Removing existing database", style(WARNING).yellow());
        std::fs::remove_file(&db_path)?;

        // Also remove any FTS auxiliary files
        let fts_files = glob::glob(&format!("{}*", db_path.display()))?;
        for file in fts_files.flatten() {
          std::fs::remove_file(file)?;
        }
      }

      // Create parent directories if they don't exist
      if let Some(parent) = db_path.parent() {
        trace!("Creating parent directories: {}", parent.display());
        std::fs::create_dir_all(parent)?;
      }

      println!(
        "{} Initializing database at: {}",
        style(ROCKET).cyan(),
        style(db_path.display()).yellow()
      );

      let db = Database::open(&db_path).await?;

      // Set up PDF directory
      let pdf_dir = Database::default_pdf_path();
      println!(
        "\n{} PDF files will be stored in: {}",
        style(PAPER).cyan(),
        style(pdf_dir.display()).yellow()
      );

      if dialoguer::Confirm::new()
        .with_prompt("Use this location for PDF storage?")
        .default(true)
        .interact()?
      {
        std::fs::create_dir_all(&pdf_dir)?;
        db.set_config("pdf_dir", &pdf_dir.to_string_lossy()).await?;
      } else {
        let pdf_dir: String =
          dialoguer::Input::new().with_prompt("Enter path for PDF storage").interact_text()?;
        let pdf_dir = PathBuf::from_str(&pdf_dir).unwrap(); // TODO (autoparallel): fix this unwrap
        std::fs::create_dir_all(&pdf_dir)?;
        db.set_config("pdf_dir", &pdf_dir.to_string_lossy()).await?;
      }

      println!("{} Database initialized successfully!", style(SUCCESS).green());
      Ok(())
    },

    Commands::Add { identifier, no_pdf } => {
      let path = cli.path.unwrap_or_else(|| {
        let default_path = Database::default_path();
        println!(
          "{} Using default database path: {}",
          style(BOOKS).cyan(),
          style(default_path.display()).yellow()
        );
        default_path
      });
      trace!("Using database at: {}", path.display());
      let db = Database::open(&path).await?;

      println!("{} Fetching paper: {}", style(LOOKING_GLASS).cyan(), style(&identifier).yellow());

      let paper = Paper::new(&identifier).await?;
      debug!("Paper details: {:?}", paper);

      println!("\n{} Found paper:", style(SUCCESS).green());
      println!("   {} {}", style("Title:").green().bold(), style(&paper.title).white());
      println!(
        "   {} {}",
        style("Authors:").green().bold(),
        style(paper.authors.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")).white()
      );

      // Save paper first
      match paper.save(&db).await {
        Ok(id) => {
          println!("\n{} Saved paper with ID: {}", style(SAVE).green(), style(id).yellow());
        },
        Err(e) if e.is_duplicate_error() => {
          println!("\n{} This paper is already in your database", style("ℹ").blue());
        },
        Err(e) => return Err(LearnerdErrors::Learner(e)),
      };

      // Handle PDF download if available
      if paper.pdf_url.is_some() {
        if !no_pdf
          && dialoguer::Confirm::new().with_prompt("Download PDF?").default(true).interact()?
        {
          println!("{} Downloading PDF...", style(LOOKING_GLASS).cyan());

          // Get PDF directory from database
          let pdf_dir = match db.get_config("pdf_dir").await? {
            Some(dir) => PathBuf::from(dir),
            None => {
              println!(
                "{} PDF directory not configured. Run {} first",
                style(WARNING).yellow(),
                style("learnerd init").cyan()
              );
              return Ok(());
            },
          };

          match paper.download_pdf(pdf_dir).await {
            Ok(_) => {
              println!("{} PDF downloaded successfully!", style(SUCCESS).green());
            },
            Err(e) => {
              println!(
                "{} Failed to download PDF: {}",
                style(WARNING).yellow(),
                style(e.to_string()).red()
              );
              println!(
                "   {} You can try downloading it later using: {} {} {} {}",
                style("Tip:").blue(),
                style("learnerd download").yellow(),
                style(&paper.source.to_string()).cyan(),
                style(&paper.source_identifier).yellow(),
                style("--force").dim()
              );
            },
          }
        }
      } else {
        println!("\n{} No PDF URL available for this paper", style(WARNING).yellow());
      }

      Ok(())
    },

    Commands::Remove { source, identifier } => {
      let path = cli.path.unwrap_or_else(|| {
        let default_path = Database::default_path();
        println!(
          "{} Using default database path: {}",
          style(BOOKS).cyan(),
          style(default_path.display()).yellow()
        );
        default_path
      });
      trace!("Using database at: {}", path.display());
      let _db = Database::open(&path).await?;

      println!("{} Remove functionality not yet implemented", style(WARNING).yellow());
      println!(
        "Would remove paper from {} with ID {}",
        style(source).cyan(),
        style(identifier).yellow()
      );
      Ok(())
    },

    Commands::Get { source, identifier } => {
      let path = cli.path.unwrap_or_else(|| {
        let default_path = Database::default_path();
        println!(
          "{} Using default database path: {}",
          style(BOOKS).cyan(),
          style(default_path.display()).yellow()
        );
        default_path
      });
      trace!("Using database at: {}", path.display());
      let db = Database::open(&path).await?;

      println!(
        "{} Fetching paper from {} with ID {}",
        style(LOOKING_GLASS).cyan(),
        style(&source).cyan(),
        style(&identifier).yellow()
      );

      match db.get_paper_by_source_id(&source, &identifier).await? {
        Some(paper) => {
          debug!("Found paper: {:?}", paper);
          println!("\n{} Paper details:", style(PAPER).green());
          println!("   {} {}", style("Title:").green().bold(), style(&paper.title).white());
          println!(
            "   {} {}",
            style("Authors:").green().bold(),
            style(paper.authors.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "))
              .white()
          );
          println!(
            "   {} {}",
            style("Abstract:").green().bold(),
            style(&paper.abstract_text).white()
          );
          println!(
            "   {} {}",
            style("Published:").green().bold(),
            style(&paper.publication_date).white()
          );
          if let Some(url) = &paper.pdf_url {
            println!("   {} {}", style("PDF URL:").green().bold(), style(url).blue().underlined());
          }
          if let Some(doi) = &paper.doi {
            println!("   {} {}", style("DOI:").green().bold(), style(doi).blue().underlined());
          }
        },
        None => {
          println!("{} Paper not found", style(WARNING).yellow());
        },
      }
      Ok(())
    },

    Commands::Search { query } => {
      let path = cli.path.unwrap_or_else(|| {
        let default_path = Database::default_path();
        println!(
          "{} Using default database path: {}",
          style(BOOKS).cyan(),
          style(default_path.display()).yellow()
        );
        default_path
      });
      trace!("Using database at: {}", path.display());
      let db = Database::open(&path).await?;

      println!("{} Searching for: {}", style(LOOKING_GLASS).cyan(), style(&query).yellow());

      // Modify query to use FTS5 syntax for better matching
      let search_query = query.split_whitespace().collect::<Vec<_>>().join(" OR ");
      debug!("Modified search query: {}", search_query);

      let papers = db.search_papers(&search_query).await?;
      if papers.is_empty() {
        println!(
          "{} No papers found matching: {}",
          style(WARNING).yellow(),
          style(&query).yellow()
        );
      } else {
        println!("\n{} Found {} papers:", style(SUCCESS).green(), style(papers.len()).yellow());

        for (i, paper) in papers.iter().enumerate() {
          debug!("Paper details: {:?}", paper);
          println!("\n{}. {}", style(i + 1).yellow(), style(&paper.title).white().bold());

          let authors = paper.authors.iter().map(|a| a.name.as_str()).collect::<Vec<_>>();

          let author_display = if authors.is_empty() {
            style("No authors listed").red().italic().to_string()
          } else {
            style(authors.join(", ")).white().to_string()
          };

          println!("   {} {}", style("Authors:").green(), author_display);

          if let Some(doi) = &paper.doi {
            println!("   {} {}", style("DOI:").green(), style(doi).blue().underlined());
          }

          println!(
            "   {} {} {}",
            style("Source:").green(),
            style(&paper.source).cyan(),
            style(&paper.source_identifier).yellow()
          );

          // Show a preview of the abstract
          if !paper.abstract_text.is_empty() {
            let preview = paper.abstract_text.chars().take(100).collect::<String>();
            let preview =
              if paper.abstract_text.len() > 100 { format!("{}...", preview) } else { preview };
            println!("   {} {}", style("Abstract:").green(), style(preview).white().italic());
          }
        }

        // If we have multiple results, show a tip about refining the search
        if papers.len() > 1 {
          println!(
            "\n{} Tip: Use quotes for exact phrases, e.g. {}",
            style("💡").yellow(),
            style("\"exact phrase\"").yellow().italic()
          );
        }
      }
      Ok(())
    },

    Commands::Clean { force } => {
      let path = cli.path.unwrap_or_else(|| {
        let default_path = Database::default_path();
        println!(
          "{} Using default database path: {}",
          style(BOOKS).cyan(),
          style(default_path.display()).yellow()
        );
        default_path
      });
      if path.exists() {
        println!(
          "{} Database found at: {}",
          style(WARNING).yellow(),
          style(path.display()).yellow()
        );

        // Skip confirmations if force flag is set
        if !force {
          // First confirmation
          if !dialoguer::Confirm::new()
            .with_prompt("Are you sure you want to delete this database?")
            .default(false)
            .wait_for_newline(true)
            .interact()?
          {
            println!("{} Operation cancelled", style("✖").red());
            return Ok(());
          }

          // Require typing DELETE for final confirmation
          let input = dialoguer::Input::<String>::new()
            .with_prompt(&format!(
              "{} Type {} to confirm deletion",
              style("⚠️").red(),
              style("DELETE").red().bold()
            ))
            .interact_text()?;

          if input != "DELETE" {
            println!("{} Operation cancelled", style("✖").red());
            return Ok(());
          }
        }

        // Proceed with deletion
        println!(
          "{} Removing database: {}",
          style(WARNING).yellow(),
          style(path.display()).yellow()
        );
        std::fs::remove_file(&path)?;

        // Also remove any FTS auxiliary files
        let fts_files = glob::glob(&format!("{}*", path.display()))?;
        for file in fts_files.flatten() {
          std::fs::remove_file(file)?;
        }
        println!("{} Database files cleaned", style(SUCCESS).green());
      } else {
        println!(
          "{} No database found at: {}",
          style(WARNING).yellow(),
          style(path.display()).yellow()
        );
      }
      Ok(())
    },
  }
}
