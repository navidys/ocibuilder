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

    pub fn manifest_path(&self, digest: &digest::Digest) -> PathBuf {
        let mut manifest_file = self.istore_path().clone();
        manifest_file.push(&digest.encoded);
        manifest_file.push(MANIFEST_FILENAME);

        manifest_file
    }
}
