//! Client for interacting with DOIs via the Crossref API

use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;

use crate::{Author, Paper, PaperError, Source};

#[derive(Debug, Deserialize)]
struct CrossrefResponse {
  message: CrossrefWork,
}

#[derive(Debug, Deserialize)]
struct CrossrefWork {
  title:            Vec<String>,
  author:           Vec<CrossrefAuthor>,
  #[serde(rename = "abstract")]
  paper_abstract:   Option<String>,
  published_print:  Option<CrossrefDate>,
  published_online: Option<CrossrefDate>,
  #[serde(rename = "URL")]
  url:              Option<String>,
  #[serde(rename = "DOI")]
  doi:              String,
}

#[derive(Debug, Deserialize)]
struct CrossrefAuthor {
  given:       Option<String>,
  family:      Option<String>,
  affiliation: Vec<CrossrefAffiliation>,
}

#[derive(Debug, Deserialize)]
struct CrossrefAffiliation {
  name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CrossrefDate {
  #[serde(rename = "date-parts")]
  date_parts: Vec<Vec<i32>>,
}

pub struct DOIClient {
  client:   reqwest::Client,
  base_url: String,
}

impl DOIClient {
  pub fn new() -> Self {
    Self {
      client:   reqwest::Client::builder()
                .user_agent("YourApp/1.0 (mailto:your@email.com)")  // Required by Crossref
                .build()
                .unwrap(),
      base_url: "https://api.crossref.org/works".to_string(),
    }
  }

  pub async fn fetch_paper(&self, doi: &str) -> Result<Paper, PaperError> {
    let url = format!("{}/{}", self.base_url, doi);

    let response: CrossrefResponse =
      self.client.get(&url).send().await?.error_for_status()?.json().await?;

    let work = response.message;

    // Get the first title or return an error
    let title =
      work.title.first().ok_or_else(|| PaperError::ApiError("No title found".into()))?.clone();

    // Convert Crossref authors to our Author type
    let authors = work
      .author
      .into_iter()
      .map(|author| {
        let name = match (author.given, author.family) {
          (Some(given), Some(family)) => format!("{} {}", given, family),
          (Some(given), None) => given,
          (None, Some(family)) => family,
          (None, None) => "Unknown".to_string(),
        };

        let affiliation = author.affiliation.first().and_then(|aff| aff.name.clone());

        Author { name, affiliation, email: None }
      })
      .collect();

    // Try to get publication date, preferring print date over online date
    let publication_date = work
      .published_print
      .or(work.published_online)
      .and_then(|date| {
        let parts = date.date_parts.first()?.clone();
        let year = parts.get(0).copied()?;
        let month = parts.get(1).copied().unwrap_or(1);
        let day = parts.get(2).copied().unwrap_or(1);

        Utc.with_ymd_and_hms(year, month as u32, day as u32, 0, 0, 0).single()
      })
      .ok_or_else(|| PaperError::ApiError("No valid publication date found".into()))?;

    Ok(Paper {
      title,
      authors,
      abstract_text: work.paper_abstract.unwrap_or_default(),
      publication_date,
      source: Source::DOI,
      source_identifier: doi.to_string(),
      pdf_url: work.url, // Note: This might not always be a PDF URL
      doi: Some(work.doi),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_crossref_parse() -> anyhow::Result<()> {
    let doi = "10.1145/1327452.1327492"; // Example paper
    let client = DOIClient::new();
    let paper = client.fetch_paper(doi).await?;

    assert!(!paper.title.is_empty());
    assert!(!paper.authors.is_empty());
    assert_eq!(paper.source, Source::DOI);
    assert_eq!(paper.source_identifier, doi);

    Ok(())
  }
}
