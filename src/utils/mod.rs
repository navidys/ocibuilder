pub mod digest;

use std::{
    ffi::OsString,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use crate::error::{BuilderError, BuilderResult};
use libcontainer::utils::create_dir_all_with_mode;
use log::debug;
use nix::sys::stat::Mode;
use rand::Rng;
use sha2::{Digest, Sha256};

pub fn get_run_dir(run_dir: &Option<OsString>) -> BuilderResult<PathBuf> {
    let user_uid = nix::unistd::geteuid().as_raw();
    match run_dir {
        Some(run) => Ok(PathBuf::from(run)),
        None => {
            let run_path = if user_uid == 0 {
                PathBuf::from("/run/ocibuilder")
            } else {
                PathBuf::from(format!("/run/user/{}/youki", user_uid))
            };
            debug!("create runtime directory: {:?}", &run_path);
            match create_dir_all_with_mode(&run_path, user_uid, Mode::S_IRWXU) {
                Ok(_) => {}
                Err(err) => return Err(BuilderError::AnyError(err.to_string())),
            }

            Ok(run_path)
        }
    }
}

pub fn get_root_dir(root_dir: Option<OsString>) -> PathBuf {
    match root_dir {
        Some(root) => PathBuf::from(root),
        None => {
            let default_sub_dir = Path::new(".local/share/ocibuilder/");
            let mut home_path = match home::home_dir() {
                Some(hpath) => hpath,
                None => PathBuf::from("/tmp"),
            };

            home_path.push(default_sub_dir);

            home_path
        }
    }
}

pub fn new_digest_id() -> BuilderResult<digest::Digest> {
    let rand_value: String = (0..100)
        .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
        .collect();

    let hash = Sha256::digest(rand_value.as_bytes());

    digest::Digest::new(&format!("sha256:{}", base16ct::lower::encode_string(&hash)))
}

pub fn compute_sha256_hash_of_file(src_file: &Path) -> BuilderResult<String> {
    // Open the file
    let mut file = match File::open(src_file) {
        Ok(f) => f,
        Err(err) => return Err(BuilderError::IoError(src_file.to_owned(), err)),
    };

    // Create a SHA-256 "hasher"
    let mut hasher = Sha256::new();

    // Read the file in 4KB chunks and feed them to the hasher
    let mut buffer = [0; 4096];
    loop {
        let bytes_read = match file.read(&mut buffer) {
            Ok(n) => n,
            Err(err) => return Err(BuilderError::IoError(src_file.to_owned(), err)),
        };

        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    // Finalize the hash and get the result as a byte array
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

pub fn file_size(src: &Path) -> BuilderResult<i64> {
    match fs::metadata(src) {
        Ok(m) => Ok(m.len() as i64),
        Err(err) => Err(BuilderError::IoError(src.to_owned(), err)),
    }
}
