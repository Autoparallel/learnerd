//! A library for fetching academic papers and their metadata from various sources
//! including arXiv, IACR, and DOI-based repositories.
//!
//! # Example
//! ```rust,no_run
//! use learnerd::{Paper, Source};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Fetch from arXiv
//!     let paper = Paper::from_arxiv("2301.07041").await?;
//!     println!("Title: {}", paper.title);
//!
//!     Ok(())
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use url::Url;

pub mod clients;

use clients::arxiv;

use tracing::debug;

/// Errors that can occur when fetching papers
#[derive(Error, Debug)]
pub enum PaperError {
    #[error("Invalid identifier format")]
    InvalidIdentifier,
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Paper not found")]
    NotFound,
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Invalid URL: {0}")]
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
    pub name: String,
    pub affiliation: Option<String>,
    pub email: Option<String>,
}

/// Represents a complete academic paper with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub title: String,
    pub authors: Vec<Author>,
    pub abstract_text: String,
    pub publication_date: DateTime<Utc>,
    pub source: Source,
    pub source_identifier: String,
    pub pdf_url: Option<String>,
    pub doi: Option<String>,
}

impl Paper {
    /// Create a paper from an arXiv identifier
    ///
    /// # Arguments
    /// * `identifier` - An arXiv identifier (e.g., "2301.07041" or "math.AG/0601001")
    ///
    /// # Example
    /// ```rust,no_run
    /// # use academic_papers::Paper;
    /// # async fn run() -> anyhow::Result<()> {
    /// let paper = Paper::from_arxiv("2301.07041").await?;
    /// println!("Title: {}", paper.title);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_arxiv(identifier: &str) -> Result<Self, PaperError> {
        let client = arxiv::ArxivClient::new();
        client.fetch_paper(identifier).await
    }

    // /// Create a paper from an IACR identifier
    // ///
    // /// # Arguments
    // /// * `identifier` - An IACR ePrint identifier (e.g., "2023/123")
    // pub async fn from_iacr(identifier: &str) -> Result<Self, PaperError> {
    //     let client = iacr::IACRClient::new();
    //     client.fetch_paper(identifier).await
    // }

    // /// Create a paper from a DOI
    // ///
    // /// # Arguments
    // /// * `doi` - A DOI (e.g., "10.1145/1327452.1327492")
    // pub async fn from_doi(doi: &str) -> Result<Self, PaperError> {
    //     let client = doi::DOIClient::new();
    //     client.fetch_paper(doi).await
    // }

    /// Create a paper from a URL
    ///
    /// # Arguments
    /// * `url` - A URL to an arXiv, IACR, or DOI paper
    ///
    /// # Example
    /// ```rust,no_run
    /// # use learnerd::Paper;
    /// # async fn run() -> anyhow::Result<()> {
    /// let paper = Paper::from_url("https://arxiv.org/abs/2301.07041").await?;
    /// println!("Title: {}", paper.title);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_url(url: &str) -> Result<Self, PaperError> {
        let url = Url::parse(url)?;

        match url.host_str() {
            Some("arxiv.org") => {
                let id = extract_arxiv_id(&url)?;
                Self::from_arxiv(&id).await
            }
            // Some("eprint.iacr.org") => {
            //     let id = extract_iacr_id(&url)?;
            //     Self::from_iacr(&id).await
            // }
            // Some("doi.org") => {
            //     let doi = extract_doi(&url)?;
            //     Self::from_doi(&doi).await
            // }
            _ => Err(PaperError::InvalidUrl(url::ParseError::IdnaError)),
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

fn extract_iacr_id(url: &Url) -> Result<String, PaperError> {
    let path = url.path();
    let re = regex::Regex::new(r"(\d{4}/\d+)$").unwrap();
    re.captures(path)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or(PaperError::InvalidIdentifier)
}

fn extract_doi(url: &Url) -> Result<String, PaperError> {
    url.path()
        .strip_prefix('/')
        .map(|s| s.to_string())
        .ok_or(PaperError::InvalidIdentifier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // #[tokio::test]
    // async fn test_arxiv_paper() -> anyhow::Result<()> {
    //     let paper = Paper::from_arxiv("2301.07041").await?;
    //     assert!(paper.title.len() > 0);
    //     assert!(!paper.authors.is_empty());
    //     assert_eq!(paper.source, Source::Arxiv);
    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_arxiv_url() -> anyhow::Result<()> {
    //     let paper = Paper::from_url("https://arxiv.org/abs/2301.07041").await?;
    //     assert_eq!(paper.source, Source::Arxiv);
    //     assert_eq!(paper.source_identifier, "2301.07041");
    //     Ok(())
    // }

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
