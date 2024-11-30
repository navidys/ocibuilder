use std::{
    fs::{self, File},
    path::Path,
};

use file_format::FileFormat;
use log::debug;

use crate::error::{BuilderError, BuilderResult};

use super::common;

pub fn add_content(src: &Path, dest: &Path) -> BuilderResult<String> {
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

        let mut add_id = String::new();
        for entry in dir_data {
            match entry {
                Ok(e) => add_id = add_content(&e.path(), &dest.join(e.file_name()))?,
                Err(err) => return Err(BuilderError::IoError(src.to_path_buf(), err)),
            };
        }

        match fs::set_permissions(dest, metadata.permissions()) {
            Ok(_) => {}
            Err(err) => return Err(BuilderError::IoError(dest.to_path_buf(), err)),
        }

        return Ok(add_id);
    } else if metadata.is_file() {
        let src_format = match FileFormat::from_file(src) {
            Ok(f) => f,
            Err(err) => return Err(BuilderError::AnyError(err.to_string())),
        };

        let src_format = src_format.media_type();
        debug!("src file format: {}", src_format);

        if src_format == "application/gzip" {
            let src_file = match File::open(src) {
                Ok(f) => f,
                Err(err) => return Err(BuilderError::IoError(src.to_path_buf(), err)),
            };

            let buf = flate2::read::GzDecoder::new(src_file);
            let mut blob_archive = tar::Archive::new(buf);
            blob_archive.set_preserve_ownerships(false);
            match blob_archive.unpack(dest) {
                Ok(_) => return common::compute_sha256_hash_of_file(src),
                Err(err) => return Err(BuilderError::ArchiveError(err.to_string())),
            }
        } else if src_format == "application/x-tar" {
            let src_file = match File::open(src) {
                Ok(f) => f,
                Err(err) => return Err(BuilderError::IoError(src.to_path_buf(), err)),
            };

            let mut blob_archive = tar::Archive::new(src_file);
            blob_archive.set_preserve_ownerships(false);
            match blob_archive.unpack(dest) {
                Ok(_) => return common::compute_sha256_hash_of_file(src),
                Err(err) => return Err(BuilderError::ArchiveError(err.to_string())),
            }
        }

        return copy_file(src, dest);
    }

    Err(BuilderError::AddError(
        "unsupported source file format".to_string(),
    ))
}

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
    } else if metadata.is_file() {
        return copy_file(src, dest);
    }

    Err(BuilderError::AddError(
        "unsupported source file format".to_string(),
    ))
}
