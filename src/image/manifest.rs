use std::{fs, path::PathBuf};

use log::debug;
use oci_spec::image::ImageManifest;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::{self, digest},
};

use super::store::ImageStore;

const MANIFEST_FILENAME: &str = "manifest.json";

impl ImageStore {
    pub fn write_manifest(
        &self,
        digest: &digest::Digest,
        manifest: ImageManifest,
    ) -> BuilderResult<()> {
        let manifest_file = self.manifest_path(digest);

        debug!("write manifest: {:?}", manifest_file);

        match manifest.to_file(manifest_file) {
            Ok(_) => Ok(()),
            Err(err) => Err(BuilderError::ImageStoreError(format!(
                "write {}: {}",
                MANIFEST_FILENAME, err,
            ))),
        }
    }

    pub fn get_manifest(&self, image_id: &str) -> BuilderResult<ImageManifest> {
        let images = self.images()?;

        for img in images {
            if img.id() == image_id {
                let img_digest = utils::digest::Digest::new(image_id)?;
                let manifest_file = self.manifest_path(&img_digest);

                match ImageManifest::from_file(manifest_file) {
                    Ok(manifest) => return Ok(manifest),
                    Err(err) => return Err(BuilderError::OciSpecError(err)),
                }
            }
        }

        Err(BuilderError::ImageNotFound(image_id.to_string()))
    }

    pub fn manifest_size(&self, digest: &digest::Digest) -> BuilderResult<i64> {
        debug!("get image manifest size: {}", digest);

        let manifest_file = self.manifest_path(digest);

        match fs::metadata(&manifest_file) {
            Ok(m) => Ok(m.len() as i64),
            Err(err) => Err(BuilderError::IoError(manifest_file, err)),
        }
    }

    pub fn manifest_path(&self, digest: &digest::Digest) -> PathBuf {
        let mut manifest_file = self.istore_path().clone();
        manifest_file.push(digest.to_string());
        manifest_file.push(MANIFEST_FILENAME);

        manifest_file
    }
}
