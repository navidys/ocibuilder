use std::fs::{self, File};

use log::debug;
use oci_client::manifest::OciDescriptor;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::store::LayerStore;

impl LayerStore {
    pub fn remove_layer_overlay(&self, dg: &digest::Digest) -> BuilderResult<()> {
        let mut ovpath = self.overlay_path().clone();
        ovpath.push(&dg.encoded);

        debug!("remove layer overlay directory: {:?}", ovpath);
        if ovpath.exists() {
            match fs::remove_dir_all(&ovpath) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(ovpath, err)),
            }
        }

        Ok(())
    }

    pub fn remove_blob(&self, dg: &digest::Digest) -> BuilderResult<()> {
        let mut all_layers: Vec<OciDescriptor> = Vec::new();
        let layers_desc = self.get_all_layers_desc()?;

        for layer in layers_desc {
            if layer.digest == dg.to_string() {
                continue;
            }

            all_layers.push(layer);
        }

        let layers_path = self.layers_path();
        let layers_file = match File::create(&layers_path) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::LayerStoreError(err.to_string())),
        };

        match serde_json::to_writer(layers_file, &all_layers) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        }

        // remove layer blob
        let blob_file = self.blob_path(dg);
        if blob_file.exists() {
            match fs::remove_file(&blob_file) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(blob_file, err)),
            }
        }

        Ok(())
    }
}
