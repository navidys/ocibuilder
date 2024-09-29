use std::{fs, path::PathBuf};

use crate::error::{BuilderError, BuilderResult};

pub struct ContainerStore {
    cstore_path: PathBuf,
}

impl ContainerStore {
    pub fn new(root_dir: &PathBuf) -> BuilderResult<Self> {
        let mut cstore_path = PathBuf::from(&root_dir);
        cstore_path.push("overlay-containers/");

        match fs::create_dir_all(&cstore_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(cstore_path, err)),
        }

        Ok(Self { cstore_path })
    }

    pub fn cstore_path(&self) -> &PathBuf {
        &self.cstore_path
    }
}
