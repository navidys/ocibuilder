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

        for dir_name in ["diff", "work", "merged"] {
            let mut ovpath_subdir = ovpath.clone();
            ovpath_subdir.push(dir_name);

            match fs::create_dir(&ovpath_subdir) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(ovpath_subdir, err)),
            }
        }

        Ok(())
    }
}
