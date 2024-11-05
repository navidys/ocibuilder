use std::{fs::File, path::PathBuf};

use log::debug;
use oci_client::Reference;
use serde::{Deserialize, Serialize};

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::store::ImageStore;

const IMAGES_FILENAME: &str = "images.json";
pub const SCRATCH_IMAGE_NAME: &str = "scratch";

#[derive(Debug, Deserialize, Serialize)]
pub struct Image {
    repository: String,
    tag: String,
    id: String,
}

impl Image {
    pub fn repository(&self) -> String {
        self.repository.clone()
    }

    pub fn tag(&self) -> String {
        self.tag.clone()
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }
}

impl ImageStore {
    pub fn images(&self) -> BuilderResult<Vec<Image>> {
        let images_path = self.images_path();

        let images_file = match File::open(&images_path) {
            Ok(f) => f,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Ok(Vec::new());
                }

                return Err(BuilderError::ImageStoreError(format!(
                    "{:?}: {:?}",
                    images_path,
                    err.to_string(),
                )));
            }
        };

        let images: Vec<Image> = match serde_json::from_reader(images_file) {
            Ok(i) => i,
            Err(err) => return Err(BuilderError::ImageStoreError(err.to_string())),
        };

        Ok(images)
    }

    pub fn image_digest(&self, name_or_id: &str) -> BuilderResult<digest::Digest> {
        let images = self.images()?;

        for img in images {
            let input_id = name_or_id.to_string();
            let img_name: String = format!("{}:{}", img.repository, img.tag);
            if img_name == input_id || (input_id.len() >= 12 && img.id[..12] == input_id[..12]) {
                let img_digest = digest::Digest::new(&format!("sha256:{}", img.id))?;
                return Ok(img_digest);
            }
        }

        Err(BuilderError::ImageNotFound(name_or_id.to_string()))
    }

    pub fn image_reference(&self, img_digest: &digest::Digest) -> BuilderResult<Reference> {
        let images = self.images()?;

        for img in images {
            if img.id == img_digest.encoded {
                let image_name = format!("{}:{}", img.repository, img.tag);
                let reference: Reference = match image_name.parse() {
                    Ok(img_ref) => img_ref,
                    Err(err) => return Err(BuilderError::InvalidImageName(img.repository, err)),
                };

                return Ok(reference);
            }
        }

        Err(BuilderError::ImageNotFound(img_digest.to_string()))
    }

    pub fn write_images(&self, img_ref: &Reference, dg: &digest::Digest) -> BuilderResult<()> {
        debug!("write images: {}", dg);

        let mut images = self.images()?;

        let img_repo = format!("{}/{}", img_ref.registry(), img_ref.repository());

        images.push(Image {
            repository: img_repo,
            tag: img_ref.tag().unwrap_or_default().to_string(),
            id: dg.encoded.to_owned(),
        });

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

        Ok(())
    }

    pub fn images_path(&self) -> PathBuf {
        let mut images_file = self.istore_path().clone();
        images_file.push(IMAGES_FILENAME);

        images_file
    }
}
