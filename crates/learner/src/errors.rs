use thiserror::Error;

/// Errors that can occur when fetching papers
#[derive(Error, Debug)]
pub enum LearnerError {
  #[error("Invalid identifier format")]
  InvalidIdentifier,
  #[error("Invalid source type, see `learner::paper::Source`")]
  InvalidSource(String),
  #[error(transparent)]
  Network(#[from] reqwest::Error),
  #[error("Paper not found")]
  NotFound,
  #[error("API error: {0}")]
  ApiError(String),
  #[error(transparent)]
  InvalidUrl(#[from] url::ParseError),
  #[error(transparent)]
  Sqlite(#[from] rusqlite::Error),
  #[error(transparent)]
  AsyncSqlite(#[from] tokio_rusqlite::Error),
  #[error(transparent)]
  Path(#[from] std::io::Error),
  #[error("Database not initialized")]
  DatabaseNotInitialized,
  #[error(transparent)]
  ColumnOverflow(#[from] std::num::TryFromIntError),
}
