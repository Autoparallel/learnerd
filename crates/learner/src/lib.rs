//! A library for fetching academic papers and their metadata from various sources
//! including arXiv, IACR, and DOI-based repositories.
//!
//! # Example
//! ```no_run
//! use learner::paper::{Paper, Source};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!   // Fetch from arXiv
//!   let paper = Paper::new("2301.07041").await?;
//!   println!("Title: {}", paper.title);
//!
//!   Ok(())
//! }
//! ```

#![warn(missing_docs, clippy::missing_docs_in_private_items)]
use std::path::PathBuf;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;
#[cfg(test)] use tracing_test::traced_test;

pub mod clients;
pub mod database;
pub mod errors;
pub mod paper;

use clients::{arxiv::ArxivClient, doi::DOIClient, iacr::IACRClient};
use database::Database;
use errors::LearnerError;
use paper::{Author, Paper, Source};
