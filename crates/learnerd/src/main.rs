use std::path::PathBuf;

use clap::{builder::ArgAction, Parser, Subcommand};
use console::{style, Emoji};
use learner::{
  database::Database,
  errors::LearnerError,
  paper::{Paper, Source},
};
use tracing::{debug, trace};
use tracing_subscriber::EnvFilter;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
static BOOKS: Emoji<'_, '_> = Emoji("📚 ", "");
static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");
static PAPER: Emoji<'_, '_> = Emoji("📄 ", "");
static SAVE: Emoji<'_, '_> = Emoji("💾 ", "");
static WARNING: Emoji<'_, '_> = Emoji("⚠️  ", "");
static SUCCESS: Emoji<'_, '_> = Emoji("✨ ", "");

#[derive(Parser)]
#[command(author, version, about = "Daemon and CLI for the learner paper management system")]
struct Cli {
  /// Verbose mode (-v, -vv, -vvv)
  #[arg(
        short,
        long,
        action = ArgAction::Count,
        global = true,
        help = "Increase logging verbosity"
    )]
  verbose: u8,

  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Initialize a new learner database
  Init {
    /// Path where the database should be created
    #[arg(long, short)]
    path: Option<PathBuf>,
  },
  /// Add a paper to the database
  Add {
    /// Paper identifier (arXiv ID, DOI, or IACR ID)
    identifier: String,
  },
  /// Remove a paper from the database
  Remove {
    /// Source system (arxiv, doi, iacr)
    #[arg(value_enum)]
    source:     Source,
    /// Paper identifier in the source system
    identifier: String,
  },
  /// Retrieve a paper from the database
  Get {
    /// Source system (arxiv, doi, iacr)
    #[arg(value_enum)]
    source:     Source,
    /// Paper identifier in the source system
    identifier: String,
  },
  /// Search papers in the database
  Search {
    /// Search query
    query: String,
  },
}

/// Setup logging with the specified verbosity level
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

#[tokio::main]
async fn main() -> Result<(), LearnerError> {
  let cli = Cli::parse();
  setup_logging(cli.verbose);

  match cli.command {
    Commands::Init { path } => {
      let path = path.unwrap_or_else(|| {
        let default_path = Database::default_path();
        println!(
          "{} Using default database path: {}",
          style(BOOKS).cyan(),
          style(default_path.display()).yellow()
        );
        default_path
      });

      if let Some(parent) = path.parent() {
        trace!("Creating parent directories: {}", parent.display());
        std::fs::create_dir_all(parent)?;
      }

      println!(
        "{} Initializing database at: {}",
        style(ROCKET).cyan(),
        style(path.display()).yellow()
      );

      Database::open(&path).await?;

      println!("{} Database initialized successfully!", style(SUCCESS).green());
      Ok(())
    },

    Commands::Add { identifier } => {
      let path = Database::default_path();
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

      let id = paper.save(&db).await?;
      println!("\n{} Saved paper with ID: {}", style(SAVE).green(), style(id).yellow());
      Ok(())
    },

    Commands::Remove { source, identifier } => {
      let path = Database::default_path();
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
      let path = Database::default_path();
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
      let path = Database::default_path();
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
  }
}
