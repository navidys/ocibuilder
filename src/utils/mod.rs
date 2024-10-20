pub mod digest;

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use rand::Rng;
use sha2::{Digest, Sha256};

use crate::error::BuilderResult;

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
