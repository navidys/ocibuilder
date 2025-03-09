use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

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
