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
