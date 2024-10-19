use std::fs::File;

use log::debug;
use oci_client::manifest::OciDescriptor;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::{self, digest},
};

use super::store::LayerStore;

const LAYERS_FILENAME: &str = "layers.json";

impl LayerStore {
    pub fn add_layer_desc(&self, desc: &OciDescriptor) -> BuilderResult<()> {
        debug!("write layer desc: {}", desc.digest);

        let leyer_digest = utils::digest::Digest::new(&desc.digest)?;

        // layer already exist and will not add
        if self.get_layer_desc(&leyer_digest).is_ok() {
            return Ok(());
        }

        let mut all_layers = self.get_all_layers_desc()?;
        all_layers.push(desc.to_owned());

        let mut layers_path = self.lstore_path().clone();
        layers_path.push(LAYERS_FILENAME);

        let layers_file = match File::create(&layers_path) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::LayerStoreError(err.to_string())),
        };

        match serde_json::to_writer(layers_file, &all_layers) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        }

        Ok(())
    }

    pub fn get_layer_desc(&self, dg: &digest::Digest) -> BuilderResult<OciDescriptor> {
        let layers = self.get_all_layers_desc()?;

        for layer in layers {
            if layer.digest == dg.to_string() {
                return Ok(layer);
            }
        }

        Err(BuilderError::LayerNotFound(dg.to_string()))
    }

    pub fn get_all_layers_desc(&self) -> BuilderResult<Vec<OciDescriptor>> {
        let mut layers_path = self.lstore_path().clone();
        layers_path.push(LAYERS_FILENAME);

        let layers_file = match File::open(&layers_path) {
            Ok(f) => f,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Ok(Vec::new());
                }

                return Err(BuilderError::LayerStoreError(err.to_string()));
            }
        };

        let layers: Vec<OciDescriptor> = match serde_json::from_reader(layers_file) {
            Ok(c) => c,
            Err(err) => return Err(BuilderError::SerdeJsonError(err)),
        };

        Ok(layers)
    }
}
