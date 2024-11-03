//! Local SQLite database management for storing and retrieving papers.
//!
//! This module provides functionality to persist paper metadata in a local SQLite database.
//! It supports:
//! - Paper metadata storage and retrieval
//! - Author information management
//! - Full-text search across papers
//! - Source-specific identifier lookups
//!
//! The database schema is automatically initialized when opening a database, and includes
//! tables for papers, authors, and full-text search indexes.
//!
//! # Examples
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Open or create a database
//! let db = learner::database::Database::open("papers.db").await?;
//!
//! // Fetch and save a paper
//! let paper = learner::paper::Paper::new("2301.07041").await?;
//! let id = db.save_paper(&paper).await?;
//!
//! // Search for papers
//! let results = db.search_papers("neural networks").await?;
//! for paper in results {
//!   println!("Found: {}", paper.title);
//! }
//! # Ok(())
//! # }
//! ```

use std::path::Path;

use rusqlite::params;
use tokio_rusqlite::Connection;

use super::*;

/// Handle for interacting with the paper database.
///
/// This struct manages an async connection to a SQLite database and provides
/// methods for storing and retrieving paper metadata. It uses SQLite's full-text
/// search capabilities for efficient paper discovery.
///
/// The database is automatically initialized with the required schema when opened.
/// If the database file doesn't exist, it will be created.
pub struct Database {
  /// Async SQLite connection handle
  conn: Connection,
}

impl Database {
  /// Opens an existing database or creates a new one at the specified path.
  ///
  /// This method will:
  /// 1. Create the database file if it doesn't exist
  /// 2. Initialize the schema using migrations
  /// 3. Set up full-text search indexes
  ///
  /// # Arguments
  ///
  /// * `path` - Path where the database file should be created or opened
  ///
  /// # Returns
  ///
  /// Returns a [`Result`] containing either:
  /// - A [`Database`] handle for database operations
  /// - A [`LearnerError`] if database creation or initialization fails
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use learner::database::Database;
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// // Open in a specific location
  /// let db = Database::open("papers.db").await?;
  ///
  /// // Or use the default location
  /// let db = Database::open(Database::default_path()).await?;
  /// # Ok(())
  /// # }
  /// ```
  pub async fn open(path: impl AsRef<Path>) -> Result<Self, LearnerError> {
    let conn = Connection::open(path.as_ref()).await?;

    // Initialize schema
    conn
      .call(|conn| {
        conn.execute_batch(include_str!(concat!(
          env!("CARGO_MANIFEST_DIR"),
          "/migrations/init.sql"
        )))?;
        Ok(())
      })
      .await?;

    Ok(Self { conn })
  }

  /// Returns the default path for the database file.
  ///
  /// The path is constructed as follows:
  /// - On Unix: `~/.local/share/learner/learner.db`
  /// - On macOS: `~/Library/Application Support/learner/learner.db`
  /// - On Windows: `%APPDATA%\learner\learner.db`
  /// - Fallback: `./learner.db` in the current directory
  ///
  /// # Examples
  ///
  /// ```no_run
  /// let path = learner::database::Database::default_path();
  /// println!("Database will be stored at: {}", path.display());
  /// ```
  pub fn default_path() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("learner").join("learner.db")
  }

  /// Saves a paper and its authors to the database.
  ///
  /// This method will:
  /// 1. Insert the paper's metadata into the papers table
  /// 2. Insert all authors into the authors table
  /// 3. Update the full-text search index
  ///
  /// The operation is performed in a transaction to ensure data consistency.
  ///
  /// # Arguments
  ///
  /// * `paper` - The paper to save
  ///
  /// # Returns
  ///
  /// Returns a [`Result`] containing either:
  /// - The database ID of the saved paper
  /// - A [`LearnerError`] if the save operation fails
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use learner::{database::Database, paper::Paper};
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// let db = Database::open("papers.db").await?;
  /// let paper = Paper::new("2301.07041").await?;
  /// let id = db.save_paper(&paper).await?;
  /// println!("Saved paper with ID: {}", id);
  /// # Ok(())
  /// # }
  /// ```
  pub async fn save_paper(&self, paper: &Paper) -> Result<i64, LearnerError> {
    let paper = paper.clone();
    self
      .conn
      .call(move |conn| {
        let tx = conn.transaction()?;

        // Insert paper
        let paper_id = {
          let mut stmt = tx.prepare_cached(
            "INSERT INTO papers (
                            title, abstract_text, publication_date, 
                            source, source_identifier, pdf_url, doi
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                        RETURNING id",
          )?;

          stmt.query_row(
            params![
              &paper.title,
              &paper.abstract_text,
              &paper.publication_date,
              paper.source.to_string(),
              &paper.source_identifier,
              &paper.pdf_url,
              &paper.doi,
            ],
            |row| row.get::<_, i64>(0),
          )?
        };

        // Insert authors
        {
          let mut stmt = tx.prepare_cached(
            "INSERT INTO authors (paper_id, name, affiliation, email)
                         VALUES (?1, ?2, ?3, ?4)",
          )?;

          for author in &paper.authors {
            stmt.execute(params![paper_id, &author.name, &author.affiliation, &author.email,])?;
          }
        }

        tx.commit()?;
        Ok(paper_id)
      })
      .await
      .map_err(LearnerError::from)
  }

  /// Retrieves a paper using its source and identifier.
  ///
  /// This method looks up a paper based on its origin (e.g., arXiv, DOI)
  /// and its source-specific identifier. It also fetches all associated
  /// author information.
  ///
  /// # Arguments
  ///
  /// * `source` - The paper's source system (arXiv, IACR, DOI)
  /// * `source_id` - The source-specific identifier
  ///
  /// # Returns
  ///
  /// Returns a [`Result`] containing either:
  /// - `Some(Paper)` if found
  /// - `None` if no matching paper exists
  /// - A [`LearnerError`] if the query fails
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use learner::{database::Database, paper::Source};
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// let db = Database::open("papers.db").await?;
  /// if let Some(paper) = db.get_paper_by_source_id(&Source::Arxiv, "2301.07041").await? {
  ///   println!("Found paper: {}", paper.title);
  /// }
  /// # Ok(())
  /// # }
  /// ```
  pub async fn get_paper_by_source_id(
    &self,
    source: &Source,
    source_id: &str,
  ) -> Result<Option<Paper>, LearnerError> {
    // Clone the values before moving into the async closure
    let source = source.to_string();
    let source_id = source_id.to_string();

    self
      .conn
      .call(move |conn| {
        let mut paper_stmt = conn.prepare_cached(
          "SELECT id, title, abstract_text, publication_date, source,
                            source_identifier, pdf_url, doi
                     FROM papers 
                     WHERE source = ?1 AND source_identifier = ?2",
        )?;

        let mut author_stmt = conn.prepare_cached(
          "SELECT name, affiliation, email
                     FROM authors
                     WHERE paper_id = ?",
        )?;

        let paper_result = paper_stmt.query_row(params![source, source_id], |row| {
          Ok(Paper {
            title:             row.get(1)?,
            abstract_text:     row.get(2)?,
            publication_date:  row.get(3)?,
            source:            Source::from_str(&row.get::<_, String>(4)?).map_err(|e| {
              rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
            })?,
            source_identifier: row.get(5)?,
            pdf_url:           row.get(6)?,
            doi:               row.get(7)?,
            authors:           Vec::new(), // Filled in below
          })
        });

        match paper_result {
          Ok(mut paper) => {
            let paper_id: i64 =
              paper_stmt.query_row(params![source, source_id], |row| row.get(0))?;

            let authors = author_stmt.query_map([paper_id], |row| {
              Ok(Author {
                name:        row.get(0)?,
                affiliation: row.get(1)?,
                email:       row.get(2)?,
              })
            })?;

            paper.authors = authors.collect::<Result<Vec<_>, _>>()?;
            Ok(Some(paper))
          },
          Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e.into()),
        }
      })
      .await
      .map_err(LearnerError::from)
  }

  /// Searches for papers using full-text search.
  ///
  /// This method uses SQLite's FTS5 module to perform full-text search across:
  /// - Paper titles
  /// - Paper abstracts
  ///
  /// Results are ordered by relevance using FTS5's built-in ranking algorithm.
  ///
  /// # Arguments
  ///
  /// * `query` - The search query using FTS5 syntax
  ///
  /// # Returns
  ///
  /// Returns a [`Result`] containing either:
  /// - A vector of matching papers
  /// - A [`LearnerError`] if the search fails
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// let db = learner::database::Database::open("papers.db").await?;
  ///
  /// // Simple word search
  /// let papers = db.search_papers("quantum").await?;
  ///
  /// // Phrase search
  /// let papers = db.search_papers("\"neural networks\"").await?;
  ///
  /// // Complex query
  /// let papers = db.search_papers("machine learning NOT regression").await?;
  /// # Ok(())
  /// # }
  /// ```
  pub async fn search_papers(&self, query: &str) -> Result<Vec<Paper>, LearnerError> {
    // Clone the query before moving into the async closure
    let query = query.to_string();

    self
      .conn
      .call(move |conn| {
        let mut stmt = conn.prepare_cached(
          "SELECT p.id, p.title, p.abstract_text, p.publication_date,
                            p.source, p.source_identifier, p.pdf_url, p.doi
                     FROM papers p
                     JOIN papers_fts f ON p.id = f.rowid
                     WHERE papers_fts MATCH ?1
                     ORDER BY rank",
        )?;

        let papers = stmt.query_map([query], |row| {
          Ok(Paper {
            title:             row.get(1)?,
            abstract_text:     row.get(2)?,
            publication_date:  row.get(3)?,
            source:            Source::from_str(&row.get::<_, String>(4)?).map_err(|e| {
              rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
            })?,
            source_identifier: row.get(5)?,
            pdf_url:           row.get(6)?,
            doi:               row.get(7)?,
            authors:           Vec::new(), // We'll fill this in below
          })
        })?;

        let mut result = Vec::new();
        for paper in papers {
          result.push(paper?);
        }
        Ok(result)
      })
      .await
      .map_err(LearnerError::from)
  }
}

#[cfg(test)]
mod tests {

  use tempfile::tempdir;

  use super::*;

  /// Helper function to create a test paper
  fn create_test_paper() -> Paper {
    Paper {
      title:             "Test Paper".to_string(),
      abstract_text:     "This is a test abstract".to_string(),
      publication_date:  Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
      source:            Source::Arxiv,
      source_identifier: "2401.00000".to_string(),
      pdf_url:           Some("https://arxiv.org/pdf/2401.00000".to_string()),
      doi:               Some("10.1000/test.123".to_string()),
      authors:           vec![
        Author {
          name:        "John Doe".to_string(),
          affiliation: Some("Test University".to_string()),
          email:       Some("john@test.edu".to_string()),
        },
        Author { name: "Jane Smith".to_string(), affiliation: None, email: None },
      ],
    }
  }

  /// Helper function to set up a test database
  async fn setup_test_db() -> (Database, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).await.unwrap();
    (db, dir)
  }

  #[tokio::test]
  async fn test_database_creation() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Create database
    let _db = Database::open(&db_path).await.unwrap();

    // Check that file exists
    assert!(db_path.exists());
  }

  #[tokio::test]
  async fn test_save_and_retrieve_paper() {
    let (db, _dir) = setup_test_db().await;
    let paper = create_test_paper();

    // Save paper
    let paper_id = db.save_paper(&paper).await.unwrap();
    assert!(paper_id > 0);

    // Retrieve paper
    let retrieved = db
      .get_paper_by_source_id(&paper.source, &paper.source_identifier)
      .await
      .unwrap()
      .expect("Paper should exist");

    // Verify paper data
    assert_eq!(retrieved.title, paper.title);
    assert_eq!(retrieved.abstract_text, paper.abstract_text);
    assert_eq!(retrieved.publication_date, paper.publication_date);
    assert_eq!(retrieved.source, paper.source);
    assert_eq!(retrieved.source_identifier, paper.source_identifier);
    assert_eq!(retrieved.pdf_url, paper.pdf_url);
    assert_eq!(retrieved.doi, paper.doi);

    // Verify authors
    assert_eq!(retrieved.authors.len(), paper.authors.len());
    assert_eq!(retrieved.authors[0].name, paper.authors[0].name);
    assert_eq!(retrieved.authors[0].affiliation, paper.authors[0].affiliation);
    assert_eq!(retrieved.authors[0].email, paper.authors[0].email);
    assert_eq!(retrieved.authors[1].name, paper.authors[1].name);
    assert_eq!(retrieved.authors[1].affiliation, None);
    assert_eq!(retrieved.authors[1].email, None);
  }

  #[tokio::test]
  async fn test_get_nonexistent_paper() {
    let (db, _dir) = setup_test_db().await;

    let result = db.get_paper_by_source_id(&Source::Arxiv, "nonexistent").await.unwrap();

    assert!(result.is_none());
  }

  #[tokio::test]
  async fn test_full_text_search() {
    let (db, _dir) = setup_test_db().await;

    // Save a few papers
    let mut paper1 = create_test_paper();
    paper1.title = "Neural Networks in Machine Learning".to_string();
    paper1.abstract_text = "This paper discusses deep learning".to_string();
    paper1.source_identifier = "2401.00001".to_string();

    let mut paper2 = create_test_paper();
    paper2.title = "Advanced Algorithms".to_string();
    paper2.abstract_text = "Classical computer science topics".to_string();
    paper2.source_identifier = "2401.00002".to_string();

    db.save_paper(&paper1).await.unwrap();
    db.save_paper(&paper2).await.unwrap();

    // Search for papers
    let results = db.search_papers("neural").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, paper1.title);

    let results = db.search_papers("learning").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].source_identifier, paper1.source_identifier);

    let results = db.search_papers("algorithms").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, paper2.title);
  }

  #[tokio::test]
  async fn test_duplicate_paper_handling() {
    let (db, _dir) = setup_test_db().await;
    let paper = create_test_paper();

    // Save paper first time
    let result1 = db.save_paper(&paper).await;
    assert!(result1.is_ok());

    // Try to save the same paper again
    let result2 = db.save_paper(&paper).await;
    assert!(result2.is_err()); // Should fail due to UNIQUE constraint
  }
}
