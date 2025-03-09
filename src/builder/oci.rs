use std::{
    fs::{self, File},
    path::PathBuf,
};

use fs2::FileExt;

use log::debug;

use crate::error::{BuilderError, BuilderResult};

pub struct OCIBuilder {
    // image_store: ImageStore,
    // container_store: ContainerStore,
    // layer_store: LayerStore,
    tmp_dir: PathBuf,
    lock_file: File,
}

impl OCIBuilder {
    pub fn new(root_dir: PathBuf) -> BuilderResult<Self> {
        debug!("root directory: {:?}", root_dir);

        // builder tmp directory
        let mut tmp_dir = PathBuf::from(&root_dir);
        tmp_dir.push("tmp/");

        match fs::create_dir_all(&tmp_dir) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(tmp_dir, err)),
        }

        // builder lock file
        let mut lfile = PathBuf::from(&root_dir);
        lfile.push("builder.lock");

        let lock_file = match File::create(&lfile) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(lfile, err)),
        };

        Ok(Self { lock_file, tmp_dir })
    }

    pub fn tmp_dir(&self) -> &PathBuf {
        &self.tmp_dir
    }

    pub fn lock(&self) -> BuilderResult<()> {
        match self.lock_file.lock_exclusive() {
            Ok(_) => Ok(()),
            Err(err) => Err(BuilderError::BuilderLockError(err)),
        }
    }

    pub fn unlock(&self) -> BuilderResult<()> {
        match self.lock_file.unlock() {
            Ok(_) => Ok(()),
            Err(err) => Err(BuilderError::BuilderLockError(err)),
        }
    }
}
