use std::{fs, path::PathBuf};

use crate::error::{BuilderError, BuilderResult};

pub struct ImageStore {
    istore_path: PathBuf,
}

impl ImageStore {
    pub fn new(root_dir: &PathBuf) -> BuilderResult<Self> {
        let mut istore_path = PathBuf::from(&root_dir);
        istore_path.push("overlay-images/");

        match fs::create_dir_all(&istore_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(istore_path, err)),
        }

        Ok(Self { istore_path })
    }

    pub fn istore_path(&self) -> &PathBuf {
        &self.istore_path
    }
}
