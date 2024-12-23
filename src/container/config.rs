use std::{fs::File, path::PathBuf};

use log::debug;
use oci_client::config::ConfigFile;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::store::ContainerStore;

const BUILDER_FILENAME: &str = "builder.json";

impl ContainerStore {
    pub fn write_builder_config(
        &self,
        cnt_id: &digest::Digest,
        img_cfg: &ConfigFile,
    ) -> BuilderResult<()> {
        debug!("write builder config: {}", cnt_id);

        let config_file_path = self.builder_config_path(cnt_id);

        let config_file = match File::create(&config_file_path) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(config_file_path, err)),
        };

        match serde_json::to_writer(config_file, img_cfg) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        }

        Ok(())
    }

    pub fn get_builder_config(&self, cnt_id: &digest::Digest) -> BuilderResult<ConfigFile> {
        debug!("get builder config: {}", cnt_id);

        let config_file_path = self.builder_config_path(cnt_id);
        let config_file = match File::open(&config_file_path) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(config_file_path, err)),
        };

        let builder_config: ConfigFile = match serde_json::from_reader(config_file) {
            Ok(m) => m,
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        };

        Ok(builder_config)
    }

    pub fn builder_config_path(&self, digest: &digest::Digest) -> PathBuf {
        let mut cpath = self.cstore_path().clone();
        cpath.push(&digest.encoded);
        cpath.push(BUILDER_FILENAME);

        cpath
    }
}
