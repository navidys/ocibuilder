use std::fs;

use crate::error::{BuilderError, BuilderResult};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn reset(&self) -> BuilderResult<()> {
        self.lock()?;

        // remove tmp directory content
        let tmp_path = self.tmp_dir();
        match fs::remove_dir_all(tmp_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(tmp_path.clone(), err)),
        }

        self.unlock()?;

        Ok(())
    }
}
