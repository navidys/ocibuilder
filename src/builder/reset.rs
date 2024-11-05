use std::{
    fs::{self, File, Permissions},
    os::unix::fs::PermissionsExt,
};
use walkdir::WalkDir;

use crate::error::{BuilderError, BuilderResult};

use super::oci::OCIBuilder;

impl OCIBuilder {
    pub fn reset(&self) -> BuilderResult<()> {
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

        // remove overlay content
        let overlay_path = self.layer_store().overlay_path();

        // first required to fixed the permissions on sub directories
        let perm = Permissions::from_mode(0o755);
        for entry in WalkDir::new(overlay_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let dir_metadata = match entry.metadata() {
                Ok(m) => m,
                Err(err) => return Err(BuilderError::WalkDirError(entry.into_path(), err)),
            };

            if dir_metadata.is_symlink() || dir_metadata.is_file() || entry.path_is_symlink() {
                continue;
            }

            let dir_file = match File::open(entry.clone().into_path()) {
                Ok(f) => f,
                Err(err) => return Err(BuilderError::IoError(entry.into_path(), err)),
            };

            match dir_file.set_permissions(perm.to_owned()) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(entry.into_path(), err)),
            }
        }

        match fs::remove_dir_all(overlay_path) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(overlay_path.clone(), err)),
        }

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
