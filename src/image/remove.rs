use std::fs::{self, File};

use log::debug;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::{images::Image, store::ImageStore};

impl ImageStore {
    pub fn remove(&self, dg: &digest::Digest) -> BuilderResult<()> {
        // remove image config
        let mut images: Vec<Image> = Vec::new();
        let image_list = self.images()?;
        for img in image_list {
            if img.id() == dg.encoded {
                continue;
            }

            images.push(img);
        }

        let images_path = self.images_path();
        let images_file = match File::create(&images_path) {
            Ok(f) => f,
            Err(err) => {
                return Err(BuilderError::ImageStoreError(format!(
                    "{:?}: {:?}",
                    images_path,
                    err.to_string(),
                )));
            }
        };

        match serde_json::to_writer(images_file, &images) {
            Ok(_) => {}
            Err(err) => {
                return Err(BuilderError::ImageStoreError(format!(
                    "{:?}: {:?}",
                    images_path,
                    err.to_string(),
                )));
            }
        }

        // remove overlay-images
        let mut image_dir = self.istore_path().clone();
        image_dir.push(&dg.encoded);
        debug!("remove overlay image directory: {:?}", image_dir);

        if image_dir.exists() {
            match fs::remove_dir_all(&image_dir) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(image_dir, err)),
            }
        }

        Ok(())
    }
}
