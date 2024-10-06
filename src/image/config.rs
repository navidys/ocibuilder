use std::{fs, path::PathBuf};

use log::debug;
use oci_spec::image::ImageConfiguration;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::store::ImageStore;

const CONFIG_FILENAME: &str = "config.json";

impl ImageStore {
    pub fn write_config(
        &self,
        digest: &digest::Digest,
        config: &ImageConfiguration,
    ) -> BuilderResult<()> {
        debug!("write image config: {}", digest);

        let mut config_dir = self.istore_path().clone();
        config_dir.push(&digest.encoded);

        match fs::create_dir_all(config_dir) {
            Ok(_) => {
                let config_file = self.config_path(digest);
                match config.to_file(config_file) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(BuilderError::ImageStoreError(format!(
                        "write config: {}",
                        err,
                    ))),
                }
            }
            Err(err) => Err(BuilderError::ImageStoreError(format!(
                "blob directory: {}",
                err,
            ))),
        }
    }

    pub fn get_config(&self, digest: &digest::Digest) -> BuilderResult<ImageConfiguration> {
        debug!("get image config: {}", digest);

        let config_file = self.config_path(digest);

        let image_config = match ImageConfiguration::from_file(config_file) {
            Ok(config) => config,
            Err(err) => return Err(BuilderError::OciSpecError(err)),
        };

        Ok(image_config)
    }

    pub fn config_size(&self, digest: &digest::Digest) -> BuilderResult<i64> {
        debug!("get image config size: {}", digest);

        let config_file = self.config_path(digest);
        match fs::metadata(&config_file) {
            Ok(m) => Ok(m.len() as i64),
            Err(err) => Err(BuilderError::IoError(config_file, err)),
        }
    }

    pub fn config_path(&self, digest: &digest::Digest) -> PathBuf {
        let mut cpath = self.istore_path().clone();
        cpath.push(&digest.encoded);
        cpath.push(CONFIG_FILENAME);

        cpath
    }
}
