use std::{io, path::PathBuf};

use thiserror::Error;

pub type BuilderResult<T> = std::result::Result<T, BuilderError>;

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("url {0} parse error: because {1}")]
    UrlParseError(String, url::ParseError),

    #[error("client call error: {0}")]
    ClientCallError(String),

    #[error("io error {0}: {1}")]
    IoError(PathBuf, io::Error),

    #[error("builder lock error: {0}")]
    BuilderLockError(io::Error),
}
