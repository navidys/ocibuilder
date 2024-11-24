use std::{fs, io, path::PathBuf};

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

            let layer_size = match dir_size(&layer_id_path) {
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

fn dir_size(path: impl Into<PathBuf>) -> io::Result<u64> {
    fn dir_size(mut dir: fs::ReadDir) -> io::Result<u64> {
        dir.try_fold(0, |acc, file| {
            let file = file?;
            let size = match file.metadata()? {
                data if data.is_dir() => dir_size(fs::read_dir(file.path())?)?,
                data => data.len(),
            };
            Ok(acc + size)
        })
    }

    dir_size(fs::read_dir(path.into())?)
}
