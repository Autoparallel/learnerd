use chrono::{DateTime, Utc};
use quick_xml::de::from_str;
use serde::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
struct Feed {
  #[serde(rename = "entry")]
  entries: Vec<Entry>,
}

// TODO: Note there are more things we get in a typical response which are probably useful honestly.
// I think we should capture those and also potentially put all of this in the `Source` enum
// variants so that the `Paper` struct contains all the relevant metadata.
#[derive(Debug, Deserialize)]
struct Entry {
  title:     String,
  #[serde(rename = "author")]
  authors:   Vec<Author>,
  summary:   String,
  published: DateTime<Utc>,
  #[serde(rename = "id")]
  arxiv_url: String,
}

#[derive(Debug, Deserialize)]
struct Author {
  name: String,
}

pub struct ArxivClient {
  client: reqwest::Client,
}

impl ArxivClient {
  /// Creates a new [`ArxivClient`].
  pub fn new() -> Self { Self { client: reqwest::Client::new() } }

  /// .
  ///
  /// # Errors
  ///
  /// This function will return an error if .
  pub async fn fetch_paper(&self, identifier: &str) -> Result<Paper, PaperError> {
    let url = format!("http://export.arxiv.org/api/query?id_list={}&max_results=1", identifier);

    debug!("Fetching from arXiv via: {url}");

    let response = self.client.get(&url).send().await?.text().await?;

    debug!("arXiv response: {response}");

    let feed: Feed = from_str(&response)
      .map_err(|e| PaperError::ApiError(format!("Failed to parse XML: {}", e)))?;

    let entry = feed.entries.first().ok_or(PaperError::NotFound)?;

    // Convert arXiv URL to PDF URL (just need to change /abs/ to /pdf/ and add .pdf)
    let pdf_url = entry.arxiv_url.replace("/abs/", "/pdf/") + ".pdf";

    Ok(Paper {
      title:             entry.title.clone(),
      authors:           entry
        .authors
        .iter()
        .map(|author| crate::Author {
          name:        author.name.clone(),
          affiliation: None,
          email:       None,
        })
        .collect(),
      abstract_text:     entry.summary.clone(),
      publication_date:  entry.published,
      source:            Source::Arxiv,
      source_identifier: identifier.to_string(),
      pdf_url:           Some(pdf_url),
      doi:               None, // We can add DOI extraction if needed
    })
  }
}

impl Default for ArxivClient {
  fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[tokio::test]
  async fn test_arxiv_entry_fetch() {
    let client = ArxivClient::new();
    let paper = client.fetch_paper("2301.07041").await.unwrap();

    println!("Title: {}", paper.title);
    println!("Authors: {:?}", paper.authors);

    assert!(!paper.title.is_empty());
    assert!(!paper.authors.is_empty());
    assert_eq!(paper.source, Source::Arxiv);
    assert_eq!(paper.source_identifier, "2301.07041");
  }
}
