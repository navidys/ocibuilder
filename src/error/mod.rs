use std::{io, path::PathBuf};

use thiserror::Error;

pub type BuilderResult<T> = std::result::Result<T, BuilderError>;

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("io error {0}: {1}")]
    IoError(PathBuf, io::Error),

    #[error("builder lock error: {0}")]
    BuilderLockError(io::Error),
}
