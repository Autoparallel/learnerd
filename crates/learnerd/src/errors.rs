use thiserror::Error;

#[derive(Error, Debug)]
pub enum LearnerdErrors {
  #[error(transparent)]
  Dialoguer(#[from] dialoguer::Error),
  #[error(transparent)]
  Learner(#[from] learner::errors::LearnerError),
  #[error(transparent)]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  Glob(#[from] glob::PatternError),
}
