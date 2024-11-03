use chrono::{DateTime, TimeZone, Utc};
use quick_xml::de::from_str;
use serde::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(rename = "OAI-PMH")]
struct OAIPMHResponse {
  #[serde(rename = "GetRecord")]
  get_record: Option<GetRecord>,
  error:      Option<OAIError>,
}

#[derive(Debug, Deserialize)]
struct OAIError {
  #[serde(rename = "@code")]
  code:    String,
  #[serde(rename = "$text")]
  message: String,
}

#[derive(Debug, Deserialize)]
struct GetRecord {
  record: Record,
}

#[derive(Debug, Deserialize)]
struct Record {
  metadata: Metadata,
}

#[derive(Debug, Deserialize)]
struct Metadata {
  #[serde(rename = "dc")]
  dublin_core: DublinCore,
}

#[derive(Debug, Deserialize)]
struct DublinCore {
  #[serde(rename = "title")]
  title:       String,
  #[serde(rename = "creator")]
  creators:    Vec<String>,
  #[serde(rename = "description")]
  description: String,
  #[serde(rename = "date")]
  dates:       Vec<String>,
  #[serde(rename = "identifier")]
  identifiers: Vec<String>,
}

pub struct IACRClient {
  client:   reqwest::Client,
  base_url: String,
}

impl IACRClient {
  pub fn new() -> Self {
    Self { client: reqwest::Client::new(), base_url: "https://eprint.iacr.org/oai".to_string() }
  }

  pub async fn fetch_paper(&self, identifier: &str) -> Result<Paper, PaperError> {
    // IACR identifiers are in the format "YYYY/NNNN"
    let parts: Vec<&str> = identifier.split('/').collect();
    if parts.len() != 2 {
      return Err(PaperError::InvalidIdentifier);
    }

    let url = format!(
      "{}?verb=GetRecord&identifier=oai:eprint.iacr.org:{}&metadataPrefix=oai_dc",
      self.base_url, identifier
    );

    debug!("Fetching from IACR via OAI-PMH: {url}");

    let response = self.client.get(&url).send().await?;

    let text = response.text().await?;
    debug!("IACR OAI-PMH response: {}", text);

    // Clean up the XML to handle namespaces
    let text = text
            .replace("xmlns:oai_dc=\"http://www.openarchives.org/OAI/2.0/oai_dc/\"", "")
            .replace("xmlns:dc=\"http://purl.org/dc/elements/1.1/\"", "")
            .replace("xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"", "")
            .replace("xsi:schemaLocation=\"http://www.openarchives.org/OAI/2.0/oai_dc/ http://www.openarchives.org/OAI/2.0/oai_dc.xsd\"", "")
            .replace("oai_dc:", "")
            .replace("dc:", "");

    debug!("Cleaned XML: {}", text);

    let oai_response: OAIPMHResponse =
      from_str(&text).map_err(|e| PaperError::ApiError(format!("Failed to parse XML: {}", e)))?;

    if let Some(error) = oai_response.error {
      return Err(PaperError::ApiError(format!(
        "OAI-PMH error: {} - {}",
        error.code, error.message
      )));
    }

    let record = oai_response
      .get_record
      .ok_or_else(|| PaperError::ApiError("No record found".to_string()))?
      .record;

    let dc = record.metadata.dublin_core;

    // Try to find a URL-style identifier starting with https://eprint.iacr.org/
    let doi = dc.identifiers.iter().find(|id| id.starts_with("https://eprint.iacr.org/")).cloned();

    // Parse the earliest date (creation date)
    let publication_date = dc
      .dates
      .first()
      .and_then(|date_str| DateTime::parse_from_rfc3339(date_str).ok())
      .map(|dt| dt.with_timezone(&Utc))
      .ok_or_else(|| PaperError::ApiError("Invalid date format".to_string()))?;

    Ok(Paper {
      title: dc.title,
      authors: dc
        .creators
        .into_iter()
        .map(|name| Author { name, affiliation: None, email: None })
        .collect(),
      abstract_text: dc.description,
      publication_date,
      source: Source::IACR,
      source_identifier: identifier.to_string(),
      pdf_url: Some(format!("https://eprint.iacr.org/{}/{}.pdf", parts[0], parts[1])),
      doi,
    })
  }
}

impl Default for IACRClient {
  fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
  use tracing_test::traced_test;

  use super::*;

  #[traced_test]
  #[tokio::test]
  async fn test_iacr_entry_fetch() {
    let client = IACRClient::new();
    let paper = client.fetch_paper("2016/260").await.unwrap();

    dbg!(&paper);

    assert!(!paper.title.is_empty());
    assert!(!paper.authors.is_empty());
    assert_eq!(paper.source, Source::IACR);
    assert_eq!(paper.source_identifier, "2016/260");
  }
}
