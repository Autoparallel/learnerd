//! A library for fetching academic papers and their metadata from various sources
//! including arXiv, IACR, and DOI-based repositories.
//!
//! # Example
//! ```rust,no_run
//! use learner::{Paper, Source};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!   // Fetch from arXiv
//!   let paper = Paper::new("2301.07041").await?;
//!   println!("Title: {}", paper.title);
//!
//!   Ok(())
//! }
//! ```

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

pub mod clients;

use clients::arxiv::ArxivClient;
use lazy_static::lazy_static;
use tracing::debug;
#[cfg(test)] use tracing_test::traced_test;

/// Errors that can occur when fetching papers
#[derive(Error, Debug)]
pub enum PaperError {
  #[error("Invalid identifier format")]
  InvalidIdentifier,
  #[error(transparent)]
  Network(#[from] reqwest::Error),
  #[error("Paper not found")]
  NotFound,
  #[error("API error: {0}")]
  ApiError(String),
  #[error(transparent)]
  InvalidUrl(#[from] url::ParseError),
}

/// The source of an academic paper
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Source {
  Arxiv,
  IACR,
  DOI,
}

/// Represents an author of a paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
  pub name:        String,
  pub affiliation: Option<String>,
  pub email:       Option<String>,
}

/// Represents a complete academic paper with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
  pub title:             String,
  pub authors:           Vec<Author>,
  pub abstract_text:     String,
  pub publication_date:  DateTime<Utc>,
  pub source:            Source,
  pub source_identifier: String,
  pub pdf_url:           Option<String>,
  pub doi:               Option<String>,
}

impl Paper {
  /// Create a new paper from either a URL, identifier, or DOI
  ///
  /// # Arguments
  /// * `input` - Can be:
  ///   - An arXiv URL (e.g., "https://arxiv.org/abs/2301.07041")
  ///   - An arXiv ID (e.g., "2301.07041" or "math.AG/0601001")
  ///   - An IACR URL (e.g., "https://eprint.iacr.org/2023/123")
  ///   - An IACR ID (e.g., "2023/123")
  ///   - A DOI URL (e.g., "https://doi.org/10.1145/1327452.1327492")
  ///   - A DOI (e.g., "10.1145/1327452.1327492")
  ///
  /// # Example
  /// ```rust,no_run
  /// # use learner::Paper;
  /// # async fn run() -> anyhow::Result<()> {
  /// // All of these are valid:
  /// let paper1 = Paper::new("https://arxiv.org/abs/2301.07041").await?;
  /// let paper2 = Paper::new("2301.07041").await?;
  /// let paper3 = Paper::new("10.1145/1327452.1327492").await?;
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Errors
  /// Returns `PaperError` if:
  /// - The input format is not recognized
  /// - The paper cannot be found
  /// - There are network issues
  /// - The API returns an error
  pub async fn new(input: &str) -> Result<Self, PaperError> {
    lazy_static! {
        // arXiv patterns
        static ref ARXIV_NEW: Regex = Regex::new(r"^(\d{4}\.\d{4,5})$").unwrap();
        static ref ARXIV_OLD: Regex = Regex::new(r"^([a-zA-Z-]+/\d{7})$").unwrap();

        // IACR pattern
        static ref IACR: Regex = Regex::new(r"^(\d{4}/\d+)$").unwrap();

        // DOI pattern
        static ref DOI: Regex = Regex::new(r"^10\.\d{4,9}/[-._;()/:\w]+$").unwrap();
    }

    // First try to parse as URL
    if let Ok(url) = Url::parse(input) {
      return match url.host_str() {
        Some("arxiv.org") => {
          let id = extract_arxiv_id(&url)?;
          ArxivClient::new().fetch_paper(&id).await
        },
        // Some("eprint.iacr.org") => {
        //     let id = extract_iacr_id(&url)?;
        //     Self::fetch_iacr(&id).await
        // }
        // Some("doi.org") => {
        //     let doi = extract_doi(&url)?;
        //     Self::fetch_doi(&doi).await
        // }
        _ => Err(PaperError::InvalidIdentifier),
      };
    }

    // If not a URL, try to match against known patterns
    match input {
      // arXiv patterns
      id if ARXIV_NEW.is_match(id) || ARXIV_OLD.is_match(id) =>
        ArxivClient::new().fetch_paper(id).await,

      // // IACR pattern
      // i if IACR.is_match(i) => Self::fetch_iacr(i).await,

      // // DOI pattern
      // i if DOI.is_match(i) => Self::fetch_doi(i).await,

      // No pattern matched
      _ => Err(PaperError::InvalidIdentifier),
    }
  }

  /// Download the paper's PDF to a file
  ///
  /// # Arguments
  /// * `path` - The path where the PDF should be saved
  pub async fn download_pdf(&self, path: PathBuf) -> Result<(), PaperError> {
    let Some(pdf_url) = &self.pdf_url else {
      return Err(PaperError::ApiError("No PDF URL available".into()));
    };

    let response = reqwest::get(pdf_url).await?;
    let bytes = response.bytes().await?;
    // TODO: Replace this with a nicer error.
    std::fs::write(path, bytes).unwrap();
    Ok(())
  }
}

// Helper functions for URL parsing
fn extract_arxiv_id(url: &Url) -> Result<String, PaperError> {
  let path = url.path();
  let re = regex::Regex::new(r"abs/([^/]+)$").unwrap();
  re.captures(path)
    .and_then(|cap| cap.get(1))
    .map(|m| m.as_str().to_string())
    .ok_or(PaperError::InvalidIdentifier)
}

#[allow(unused)]
fn extract_iacr_id(url: &Url) -> Result<String, PaperError> {
  let path = url.path();
  let re = regex::Regex::new(r"(\d{4}/\d+)$").unwrap();
  re.captures(path)
    .and_then(|cap| cap.get(1))
    .map(|m| m.as_str().to_string())
    .ok_or(PaperError::InvalidIdentifier)
}

#[allow(unused)]
fn extract_doi(url: &Url) -> Result<String, PaperError> {
  url.path().strip_prefix('/').map(|s| s.to_string()).ok_or(PaperError::InvalidIdentifier)
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;

  use super::*;

  #[traced_test]
  #[tokio::test]
  async fn test_arxiv_paper_from_id() {
    let paper = Paper::new("2301.07041").await.unwrap();
    assert!(!paper.title.is_empty());
    assert!(!paper.authors.is_empty());
    assert_eq!(paper.source, Source::Arxiv);
    dbg!(paper);
  }

  #[traced_test]
  #[tokio::test]
  async fn test_arxiv_paper_from_url() {
    let paper = Paper::new("https://arxiv.org/abs/2301.07041").await.unwrap();
    assert_eq!(paper.source, Source::Arxiv);
    assert_eq!(paper.source_identifier, "2301.07041");
  }

  // #[tokio::test]
  // async fn test_iacr_paper() -> anyhow::Result<()> {
  //     let paper = Paper::from_iacr("2023/123").await?;
  //     assert!(paper.title.len() > 0);
  //     assert!(!paper.authors.is_empty());
  //     assert_eq!(paper.source, Source::IACR);
  //     Ok(())
  // }

  // #[tokio::test]
  // async fn test_doi_paper() -> anyhow::Result<()> {
  //     let paper = Paper::from_doi("10.1145/1327452.1327492").await?;
  //     assert!(paper.title.len() > 0);
  //     assert!(!paper.authors.is_empty());
  //     assert_eq!(paper.source, Source::DOI);
  //     Ok(())
  // }
}
