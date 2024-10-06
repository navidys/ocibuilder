use std::{fs, path::PathBuf};

use crate::{
    error::{BuilderError, BuilderResult},
    utils::digest,
};

pub struct LayerStore {
    lstore_path: PathBuf,
    overlay_path: PathBuf,
}

impl LayerStore {
    pub fn new(root_dir: &PathBuf) -> BuilderResult<Self> {
        let mut lstore_path = PathBuf::from(&root_dir);
        lstore_path.push("overlay-layers/");

        match fs::create_dir_all(&lstore_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(lstore_path, err)),
        }

        let mut overlay_path = PathBuf::from(&root_dir);
        overlay_path.push("overlay/");

        match fs::create_dir_all(&overlay_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(overlay_path, err)),
        }

        Ok(Self {
            lstore_path,
            overlay_path,
        })
    }

    pub fn lstore_path(&self) -> &PathBuf {
        &self.lstore_path
    }

    pub fn overlay_path(&self) -> &PathBuf {
        &self.overlay_path
    }

    pub fn overlay_diff_path(&self, dg: &digest::Digest) -> PathBuf {
        let mut diff_path = self.overlay_path.clone();
        diff_path.push(&dg.encoded);
        diff_path.push("diff");
        diff_path
    }

    pub fn overlay_merged_path(&self, dg: &digest::Digest) -> PathBuf {
        let mut merged_path = self.overlay_path.clone();
        merged_path.push(&dg.encoded);
        merged_path.push("merged");
        merged_path
    }

    pub fn overlay_work_path(&self, dg: &digest::Digest) -> PathBuf {
        let mut work_path = self.overlay_path.clone();
        work_path.push(&dg.encoded);
        work_path.push("work");
        work_path
    }
}
