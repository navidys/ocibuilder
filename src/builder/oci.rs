use fs2::FileExt;

use std::{
    fs::{self, File},
    path::PathBuf,
};

use log::debug;

use crate::{
    container::store::ContainerStore,
    error::{BuilderError, BuilderResult},
    image::store::ImageStore,
    layer::store::LayerStore,
};

pub struct OCIBuilder {
    image_store: ImageStore,
    container_store: ContainerStore,
    layer_store: LayerStore,
    root_dir: PathBuf,
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

        let image_store = ImageStore::new(&root_dir)?;
        let container_store = ContainerStore::new(&root_dir)?;
        let layer_store = LayerStore::new(&root_dir)?;

        Ok(Self {
            image_store,
            container_store,
            layer_store,
            lock_file,
            tmp_dir,
            root_dir,
        })
    }

    pub fn image_store(&self) -> &ImageStore {
        &self.image_store
    }

    pub fn container_store(&self) -> &ContainerStore {
        &self.container_store
    }

    pub fn layer_store(&self) -> &LayerStore {
        &self.layer_store
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
