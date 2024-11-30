use std::{fs, path::Path};

use crate::error::{BuilderError, BuilderResult};

use super::common;

fn copy_file(src: &Path, dest: &Path) -> BuilderResult<String> {
    let mut file_dest = dest.to_path_buf();
    if dest.exists() && dest.is_dir() {
        let src_filename = src.file_name().unwrap_or_default();
        file_dest.push(src_filename);
    }

    match fs::copy(src, file_dest) {
        Ok(_) => {}
        Err(err) => return Err(BuilderError::IoError(dest.to_path_buf(), err)),
    }

    common::compute_sha256_hash_of_file(src)
}

pub fn copy_content(src: &Path, dest: &Path) -> BuilderResult<String> {
    let metadata = match fs::metadata(src) {
        Ok(m) => m,
        Err(err) => return Err(BuilderError::IoError(dest.to_path_buf(), err)),
    };

    if metadata.is_dir() {
        let dir_data: fs::ReadDir = match fs::read_dir(src) {
            Ok(d) => d,
            Err(err) => return Err(BuilderError::IoError(src.to_path_buf(), err)),
        };

        if !dest.exists() {
            match fs::create_dir_all(dest) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::IoError(dest.to_path_buf(), err)),
            }
        }

        let mut copy_id = String::new();
        for entry in dir_data {
            match entry {
                Ok(e) => copy_id = copy_content(&e.path(), &dest.join(e.file_name()))?,
                Err(err) => return Err(BuilderError::IoError(src.to_path_buf(), err)),
            };
        }

        match fs::set_permissions(dest, metadata.permissions()) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(dest.to_path_buf(), err)),
        }

        return Ok(copy_id);
    }

    copy_file(src, dest)
}
