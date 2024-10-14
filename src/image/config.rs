use std::{fs, path::PathBuf};

use log::debug;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::store::ImageStore;

const CONFIG_FILENAME: &str = "config.json";

impl ImageStore {
    pub fn write_config(&self, digest: &digest::Digest, config: &str) -> BuilderResult<()> {
        debug!("write image config: {}", digest);

        let mut config_dir = self.istore_path().clone();
        config_dir.push(&digest.encoded);

        match fs::create_dir_all(config_dir) {
            Ok(_) => {
                let config_file = self.config_path(digest);
                match fs::write(&config_file, config) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(BuilderError::IoError(config_file, err)),
                }
            }
            Err(err) => Err(BuilderError::ImageStoreError(format!(
                "blob directory: {}",
                err,
            ))),
        }
    }

    pub fn config_path(&self, digest: &digest::Digest) -> PathBuf {
        let mut cpath = self.istore_path().clone();
        cpath.push(&digest.encoded);
        cpath.push(CONFIG_FILENAME);

        cpath
    }
}
