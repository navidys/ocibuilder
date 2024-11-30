use crate::{
    error::{BuilderError, BuilderResult},
    utils,
};

use log::debug;
use oci_client::manifest::OciDescriptor;

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn calculate_image_layers_size(&self, layers: Vec<OciDescriptor>) -> BuilderResult<i64> {
        let mut est_layers_size: u64 = 0;

        for layer in layers {
            let layer_id = utils::digest::Digest::new(&layer.digest)?;
            let layer_id_path = self.layer_store().overlay_dir_path(&layer_id);

            let layer_size = match utils::common::dir_size(&layer_id_path) {
                Ok(s) => s,
                Err(err) => return Err(BuilderError::AnyError(err.to_string())),
            };

            debug!("dir {:?} size: {}", layer_id_path, layer_size);

            est_layers_size += layer_size
        }

        let layers_dir_size = i64::from_ne_bytes(est_layers_size.to_ne_bytes());

        Ok(layers_dir_size)
    }
}
