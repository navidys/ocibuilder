use std::{
    fs::{self, File},
    path::PathBuf,
};

use log::debug;
use oci_client::manifest::OciImageManifest;
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
        manifest: &OciImageManifest,
    ) -> BuilderResult<()> {
        let manifest_file_path = self.manifest_path(digest);

        debug!("write manifest: {:?}", manifest_file_path);

        let mut image_manifest_dir = self.istore_path().clone();
        image_manifest_dir.push(digest.encoded.to_string());

        match fs::create_dir_all(&image_manifest_dir) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(image_manifest_dir, err)),
        }

        let manifest_file = match File::create(&manifest_file_path) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::IoError(manifest_file_path, err)),
        };

        match serde_json::to_writer(manifest_file, manifest) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        }

        Ok(())
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
        manifest_file.push(digest.encoded.to_string());
        manifest_file.push(MANIFEST_FILENAME);

        manifest_file
    }
}
