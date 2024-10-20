use std::{fs::File, path::PathBuf};

use log::debug;
use oci_client::manifest::OciImageManifest;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
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

    pub fn get_manifest(&self, image_id: &digest::Digest) -> BuilderResult<OciImageManifest> {
        let images = self.images()?;

        for img in images {
            if img.id() == image_id.encoded {
                let manifest_file_path = self.manifest_path(image_id);
                let manifest_file = match File::open(&manifest_file_path) {
                    Ok(f) => f,
                    Err(err) => return Err(BuilderError::IoError(manifest_file_path, err)),
                };

                let img_manifest: OciImageManifest = match serde_json::from_reader(manifest_file) {
                    Ok(m) => m,
                    Err(err) => return Err(BuilderError::SerdeJsonError(err)),
                };

                return Ok(img_manifest);
            }
        }

        Err(BuilderError::ImageManifestNotFound(image_id.to_string()))
    }

    pub fn manifest_path(&self, digest: &digest::Digest) -> PathBuf {
        let mut manifest_file = self.istore_path().clone();
        manifest_file.push(&digest.encoded);
        manifest_file.push(MANIFEST_FILENAME);

        manifest_file
    }
}
