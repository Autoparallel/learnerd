use lazy_static::lazy_static;
use regex::Regex;
use url::Url;

use super::*;

/// The source of an academic paper
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Source {
  Arxiv,
  IACR,
  DOI,
}

impl std::fmt::Display for Source {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Source::Arxiv => write!(f, "Arxiv"),
      Source::IACR => write!(f, "IACR"),
      Source::DOI => write!(f, "DOI"),
    }
  }
}

impl std::str::FromStr for Source {
  type Err = LearnerError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Arxiv" => Ok(Source::Arxiv),
      "IACR" => Ok(Source::IACR),
      "DOI" => Ok(Source::DOI),
      s => Err(LearnerError::InvalidSource(s.to_owned())),
    }
  }
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
  ///   - An IACR URL (e.g., "https://eprint.iacr.org/2016/260")
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
  pub async fn new(input: &str) -> Result<Self, LearnerError> {
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
        Some("eprint.iacr.org") => {
          let id = extract_iacr_id(&url)?;
          IACRClient::new().fetch_paper(&id).await
        },
        Some("doi.org") => {
          let doi = extract_doi(&url)?;
          DOIClient::new().fetch_paper(&doi).await
        },
        _ => Err(LearnerError::InvalidIdentifier),
      };
    }

    // If not a URL, try to match against known patterns
    match input {
      // arXiv patterns
      id if ARXIV_NEW.is_match(id) || ARXIV_OLD.is_match(id) =>
        ArxivClient::new().fetch_paper(id).await,

      // IACR pattern
      id if IACR.is_match(id) => IACRClient::new().fetch_paper(id).await,

      // DOI pattern
      id if DOI.is_match(id) => DOIClient::new().fetch_paper(id).await,

      // No pattern matched
      _ => Err(LearnerError::InvalidIdentifier),
    }
  }

  /// Download the paper's PDF to a file
  ///
  /// # Arguments
  /// * `path` - The path where the PDF should be saved
  pub async fn download_pdf(&self, path: PathBuf) -> Result<(), LearnerError> {
    let Some(pdf_url) = &self.pdf_url else {
      return Err(LearnerError::ApiError("No PDF URL available".into()));
    };

    let response = reqwest::get(pdf_url).await?;
    let bytes = response.bytes().await?;
    // TODO: Replace this with a nicer error.
    std::fs::write(path, bytes).unwrap();
    Ok(())
  }

  pub async fn save(&self, db: &Database) -> Result<i64, LearnerError> { db.save_paper(self).await }
}

// Helper functions for URL parsing
fn extract_arxiv_id(url: &Url) -> Result<String, LearnerError> {
  let path = url.path();
  let re = regex::Regex::new(r"abs/([^/]+)$").unwrap();
  re.captures(path)
    .and_then(|cap| cap.get(1))
    .map(|m| m.as_str().to_string())
    .ok_or(LearnerError::InvalidIdentifier)
}

fn extract_iacr_id(url: &Url) -> Result<String, LearnerError> {
  let path = url.path();
  let re = regex::Regex::new(r"(\d{4}/\d+)$").unwrap();
  re.captures(path)
    .and_then(|cap| cap.get(1))
    .map(|m| m.as_str().to_string())
    .ok_or(LearnerError::InvalidIdentifier)
}

fn extract_doi(url: &Url) -> Result<String, LearnerError> {
  url.path().strip_prefix('/').map(|s| s.to_string()).ok_or(LearnerError::InvalidIdentifier)
}

#[cfg(test)]
mod tests {
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

  #[tokio::test]
  async fn test_iacr_paper_from_id() -> anyhow::Result<()> {
    let paper = Paper::new("2016/260").await?;
    assert!(!paper.title.is_empty());
    assert!(!paper.authors.is_empty());
    assert_eq!(paper.source, Source::IACR);
    Ok(())
  }

  #[tokio::test]
  async fn test_iacr_paper_from_url() -> anyhow::Result<()> {
    let paper = Paper::new("https://eprint.iacr.org/2016/260").await?;
    assert!(!paper.title.is_empty());
    assert!(!paper.authors.is_empty());
    assert_eq!(paper.source, Source::IACR);
    Ok(())
  }

  #[tokio::test]
  async fn test_doi_paper_from_id() -> anyhow::Result<()> {
    let paper = Paper::new("10.1145/1327452.1327492").await?;
    assert!(!paper.title.is_empty());
    assert!(!paper.authors.is_empty());
    assert_eq!(paper.source, Source::DOI);
    Ok(())
  }

  #[tokio::test]
  async fn test_doi_paper_from_url() -> anyhow::Result<()> {
    let paper = Paper::new("https://doi.org/10.1145/1327452.1327492").await?;
    assert!(!paper.title.is_empty());
    assert!(!paper.authors.is_empty());
    assert_eq!(paper.source, Source::DOI);
    Ok(())
  }
}
