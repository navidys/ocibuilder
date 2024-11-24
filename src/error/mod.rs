use std::{io, path::PathBuf};

use oci_client::{errors::OciDistributionError, ParseError};
use oci_spec::OciSpecError;
use thiserror::Error;

pub type BuilderResult<T> = std::result::Result<T, BuilderError>;

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("tab writer: {0}")]
    TabWriterError(String),

    #[error("io error {0}: {1}")]
    IoError(PathBuf, io::Error),

    #[error("walkdir error {0}: {1}")]
    WalkDirError(PathBuf, walkdir::Error),

    #[error("json error: {0}")]
    SerdeJsonError(serde_json::Error),

    #[error("archive error: {0}")]
    ArchiveError(String),

    // general builder errors
    #[error("response error: {0}")]
    ResponseError(String),

    #[error("terminal multi progress error: {0}")]
    TerminalMultiProgressError(String),

    #[error("anyhow error: {0}")]
    AnyError(String),

    #[error("builder lock error: {0}")]
    BuilderLockError(io::Error),

    #[error("oci distribution error: {0}")]
    OciDistError(OciDistributionError),

    #[error("distribution error: {0}")]
    DistributionError(String),

    #[error("invalid digest: {0}")]
    InvalidDigest(String),

    #[error("mount/umount error: {0}")]
    MountUmountError(String),

    #[error("spawn: {0}")]
    SpawnError(String),

    #[error("image or container name/id not found: {0}")]
    ContainerOrImageNotFound(String),

    // yuki error
    #[error("yuki error: {0}")]
    YukiError(String),

    // OCI spec errors
    #[error("oci spec error: {0}")]
    OciSpecError(OciSpecError),

    // container store errors
    #[error("container store error: {0}")]
    ContainerStoreError(String),

    #[error("container with same name found: {0}")]
    ContainerWithSameName(String),

    #[error("container not found: {0}")]
    ContainerNotFound(String),

    // image store errors
    #[error("image store error: {0}")]
    ImageStoreError(String),

    #[error("image not found: {0}")]
    ImageNotFound(String),

    #[error("image manifest not found: {0}")]
    ImageManifestNotFound(String),

    #[error("invalid image name {0}: {1}")]
    InvalidImageName(String, ParseError),

    #[error("Image with same name found: {0}")]
    ImageWithSameName(String),

    #[error("invalid image reference: {0}")]
    InvalidImageReference(String),

    #[error("image archive file exists: {0}")]
    ImageArchiveExits(PathBuf),

    #[error("image used by a container: {0}")]
    ImageUsedByContainer(String),

    // layers store error
    #[error("layer store error: {0}")]
    LayerStoreError(String),

    #[error("layer not found: {0}")]
    LayerNotFound(String),
}
