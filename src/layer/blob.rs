use std::{fs, path::PathBuf};

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::store::LayerStore;

impl LayerStore {
    pub fn write_blob(&self, digest: &digest::Digest, blobs: &Vec<u8>) -> BuilderResult<()> {
        let mut blob_dir = self.lstore_path().clone();
        blob_dir.push(&digest.algorithm);

        let blob_file = self.blob_path(digest);

        match fs::create_dir_all(blob_dir) {
            Ok(_) => match fs::write(blob_file, blobs) {
                Ok(_) => {}
                Err(err) => {
                    return Err(BuilderError::LayerStoreError(format!(
                        "blob write: {}",
                        err,
                    )))
                }
            },
            Err(err) => {
                return Err(BuilderError::LayerStoreError(format!(
                    "blob directory: {}",
                    err,
                )))
            }
        }

        Ok(())
    }

    pub fn blob_path(&self, digest: &digest::Digest) -> PathBuf {
        let mut blob_file = self.lstore_path().clone();
        blob_file.push(&digest.algorithm);
        blob_file.push(&digest.encoded);

        blob_file
    }
}
