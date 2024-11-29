pub mod common;
pub mod copy;
pub mod digest;

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::error::{BuilderError, BuilderResult};
use libcontainer::utils::create_dir_all_with_mode;
use log::debug;
use nix::sys::stat::Mode;

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
