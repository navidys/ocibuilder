use std::fs;

use log::debug;

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
}
