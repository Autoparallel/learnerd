use chrono::{TimeZone, Utc};
use serde::Deserialize;

use super::*;

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
  // Add created field as fallback
  created:          Option<CrossrefDate>,
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

  fn parse_date(&self, date: &CrossrefDate) -> Option<DateTime<Utc>> {
    let parts = date.date_parts.first()?;
    debug!("Date parts: {:?}", parts);

    let year = *parts.first()?;
    let month = parts.get(1).copied().unwrap_or(1);
    let day = parts.get(2).copied().unwrap_or(1);

    debug!("Parsed year: {}, month: {}, day: {}", year, month, day);

    Utc.with_ymd_and_hms(year, month as u32, day as u32, 0, 0, 0).single()
  }

  pub async fn fetch_paper(&self, doi: &str) -> Result<Paper, LearnerError> {
    let url = format!("{}/{}", self.base_url, doi);
    debug!("Fetching from Crossref via: {}", url);

    let response = self.client.get(&url).send().await?;
    let status = response.status();
    debug!("Crossref response status: {}", status);

    let text = response.text().await?;
    debug!("Crossref response: {}", text);

    let response: CrossrefResponse = serde_json::from_str(&text)
      .map_err(|e| LearnerError::ApiError(format!("Failed to parse JSON: {}", e)))?;

    let work = response.message;

    debug!("Published print: {:?}", work.published_print);
    debug!("Published online: {:?}", work.published_online);
    debug!("Created: {:?}", work.created);

    // Get the first title or return an error
    let title =
      work.title.first().ok_or_else(|| LearnerError::ApiError("No title found".into()))?.clone();

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

    // Try to get publication date, with multiple fallbacks
    let publication_date = work
      .published_print
      .as_ref()
      .and_then(|d| self.parse_date(d))
      .or_else(|| work.published_online.as_ref().and_then(|d| self.parse_date(d)))
      .or_else(|| work.created.as_ref().and_then(|d| self.parse_date(d)))
      .ok_or_else(|| {
        LearnerError::ApiError(format!(
          "No valid publication date found. Print: {:?}, Online: {:?}, Created: {:?}",
          work.published_print, work.published_online, work.created
        ))
      })?;

    Ok(Paper {
      title,
      authors,
      abstract_text: work.paper_abstract.unwrap_or_default(),
      publication_date,
      source: Source::DOI,
      source_identifier: doi.to_string(),
      pdf_url: work.url,
      doi: Some(work.doi),
    })
  }
}

impl Default for DOIClient {
  fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
  use tracing_test::traced_test;

  use super::*;

  #[traced_test]
  #[tokio::test]
  async fn test_crossref_parse() -> anyhow::Result<()> {
    let doi = "10.1145/1327452.1327492";
    let client = DOIClient::new();
    let paper = client.fetch_paper(doi).await.unwrap();

    dbg!(&paper);

    assert!(!paper.title.is_empty());
    assert!(!paper.authors.is_empty());
    assert_eq!(paper.source, Source::DOI);
    assert_eq!(paper.source_identifier, doi);

    Ok(())
  }
}
