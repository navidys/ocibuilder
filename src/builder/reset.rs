use std::fs;

use crate::error::{BuilderError, BuilderResult};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub async fn reset(&self) -> BuilderResult<()> {
        self.lock()?;

        // remove image store directory content
        let istore_path = self.image_store().istore_path();
        match fs::remove_dir_all(istore_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(istore_path.clone(), err)),
        }

        // remove container store directory content
        let cstore_path = self.container_store().cstore_path();
        match fs::remove_dir_all(cstore_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(cstore_path.clone(), err)),
        }

        // remove layer store directory content
        let lstore_path = self.layer_store().lstore_path();
        match fs::remove_dir_all(lstore_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(lstore_path.clone(), err)),
        }

        self.unlock()?;

        Ok(())
    }
}
