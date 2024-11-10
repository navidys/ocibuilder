use std::fs;

use log::debug;

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

use super::store::LayerStore;

impl LayerStore {
    pub fn create_layer_overlay_dir(&self, dg: &digest::Digest) -> BuilderResult<()> {
        let mut ovpath = self.overlay_path().clone();
        ovpath.push(&dg.encoded);

        debug!("create layer overlay directories: {:?}", ovpath);

        match fs::create_dir_all(&ovpath) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(ovpath, err)),
        }

        let overlays_subdir = [
            &self.overlay_diff_path(dg),
            &self.overlay_rootfs_path(dg),
            &self.overlay_work_path(dg),
        ];

        for dir_path in overlays_subdir {
            match fs::create_dir(dir_path) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(dir_path.to_owned(), err)),
            }
        }

        Ok(())
    }
}
