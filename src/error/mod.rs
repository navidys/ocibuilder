use std::{io, path::PathBuf};

use oci_spec::OciSpecError;
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

    #[error("oci spec error: {0}")]
    OciSpecError(OciSpecError),

    #[error("image store error: {0}")]
    ImageStoreError(String),

    #[error("image not found: {0}")]
    ImageNotFound(String),

    #[error("invalid image name: {0}")]
    InvalidImageName(String),

    #[error("invalid image reference: {0}")]
    InvalidImageReference(String),

    #[error("distribution error: {0}")]
    DistributionError(String),

    #[error("layer store error: {0}")]
    LayerStoreError(String),

    #[error("invalid digest: {0}")]
    InvalidDigest(String),
}
