use std::{io, path::PathBuf};

use oci_client::{errors::OciDistributionError, ParseError};
use thiserror::Error;

pub type BuilderResult<T> = std::result::Result<T, BuilderError>;

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("tab writer: {0}")]
    TabWriterError(String),

    #[error("io error {0}: {1}")]
    IoError(PathBuf, io::Error),

    #[error("json error: {0}")]
    SerdeJsonError(serde_json::Error),

    #[error("builder lock error: {0}")]
    BuilderLockError(io::Error),

    // general builder errors
    #[error("oci distribution error: {0}")]
    OciDistError(OciDistributionError),

    #[error("distribution error: {0}")]
    DistributionError(String),

    #[error("invalid digest: {0}")]
    InvalidDigest(String),

    // image store errors
    #[error("image store error: {0}")]
    ImageStoreError(String),

    #[error("image not found: {0}")]
    ImageNotFound(String),

    #[error("invalid image name {0}: {1}")]
    InvalidImageName(String, ParseError),

    #[error("invalid image reference: {0}")]
    InvalidImageReference(String),

    // layers store error
    #[error("layer store error: {0}")]
    LayerStoreError(String),

    #[error("layer not found: {0}")]
    LayerNotFound(String),
}
