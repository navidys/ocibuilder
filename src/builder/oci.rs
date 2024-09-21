use std::{
    fs::{self, File},
    path::PathBuf,
};

use log::debug;

use crate::{
    container,
    error::{BuilderError, BuilderResult},
    image, layer,
};

pub struct OCIBuilder {
    pub image_store: image::store::ImageStore,
    pub container_store: container::store::ContainerStore,
    pub layer_store: layer::store::LayerStore,

    lock_file: File,
    root_dir: PathBuf,
    tmp_dir: PathBuf,
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

        let image_store = image::store::ImageStore::new(&root_dir)?;
        let container_store = container::store::ContainerStore::new(&root_dir)?;
        let layer_store = layer::store::LayerStore::new(&root_dir)?;

        Ok(Self {
            image_store,
            container_store,
            layer_store,
            lock_file,
            tmp_dir,
            root_dir,
        })
    }
}
