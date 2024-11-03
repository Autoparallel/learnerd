use std::path::{Path, PathBuf};

use rusqlite::params;
use tokio_rusqlite::Connection;

use super::*;

/// Database handle for learner
pub struct Database {
  conn: Connection,
}

impl Database {
  /// Open or create a database at the specified path
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

  /// Get default database path in user's data directory
  pub fn default_path() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("learner").join("learner.db")
  }

  /// Save a paper to the database
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

  /// Get a paper by its source and identifier
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
            source:            serde_json::from_str(&row.get::<_, String>(4)?).unwrap(),
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

  /// Search papers using FTS
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
            source:            serde_json::from_str(&row.get::<_, String>(4)?).unwrap(),
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
  use std::fs;

  use chrono::TimeZone;
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
  async fn test_save_and_retrieve_paper() -> Result<(), LearnerError> {
    let (db, _dir) = setup_test_db().await;
    let paper = create_test_paper();

    // Save paper
    let paper_id = db.save_paper(&paper).await?;
    assert!(paper_id > 0);

    // Retrieve paper
    let retrieved = db
      .get_paper_by_source_id(&paper.source, &paper.source_identifier)
      .await?
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

    Ok(())
  }

  #[tokio::test]
  async fn test_get_nonexistent_paper() -> Result<(), LearnerError> {
    let (db, _dir) = setup_test_db().await;

    let result = db.get_paper_by_source_id(&Source::Arxiv, "nonexistent").await?;

    assert!(result.is_none());
    Ok(())
  }

  #[tokio::test]
  async fn test_full_text_search() -> Result<(), LearnerError> {
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

    db.save_paper(&paper1).await?;
    db.save_paper(&paper2).await?;

    // Search for papers
    let results = db.search_papers("neural").await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, paper1.title);

    let results = db.search_papers("learning").await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].source_identifier, paper1.source_identifier);

    let results = db.search_papers("algorithms").await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, paper2.title);

    Ok(())
  }

  #[tokio::test]
  async fn test_duplicate_paper_handling() -> Result<(), LearnerError> {
    let (db, _dir) = setup_test_db().await;
    let paper = create_test_paper();

    // Save paper first time
    let result1 = db.save_paper(&paper).await;
    assert!(result1.is_ok());

    // Try to save the same paper again
    let result2 = db.save_paper(&paper).await;
    assert!(result2.is_err()); // Should fail due to UNIQUE constraint

    Ok(())
  }
}
