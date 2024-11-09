use std::fs::{self, File};

use log::debug;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::{containers::Container, store::ContainerStore};

impl ContainerStore {
    pub fn remove(&self, dg: &digest::Digest) -> BuilderResult<()> {
        // remove from containers config
        let mut containers: Vec<Container> = Vec::new();
        let cnt_list = self.containers()?;
        for cnt in cnt_list {
            if cnt.id()[..12] == dg.encoded[..12] {
                continue;
            }

            containers.push(cnt);
        }

        let cnt_file_path = self.containers_path();
        let cnt_file = match File::create(&cnt_file_path) {
            Ok(f) => f,
            Err(err) => {
                return Err(BuilderError::ContainerStoreError(format!(
                    "{:?}: {:?}",
                    cnt_file_path,
                    err.to_string(),
                )));
            }
        };

        match serde_json::to_writer(cnt_file, &containers) {
            Ok(_) => {}
            Err(err) => {
                return Err(BuilderError::ContainerStoreError(format!(
                    "{:?}: {:?}",
                    cnt_file_path,
                    err.to_string(),
                )));
            }
        }

        // remove container layer
        let mut cnt_path = self.cstore_path().clone();
        cnt_path.push(&dg.encoded);
        debug!("remove overlay container directory: {:?}", cnt_path);

        if cnt_path.exists() {
            match fs::remove_dir_all(&cnt_path) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(cnt_path, err)),
            }
        }

        Ok(())
    }
}
