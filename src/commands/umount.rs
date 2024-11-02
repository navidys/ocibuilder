use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Umount {
    /// container name or ID
    container: String,
}

impl Umount {
    pub fn new(container: String) -> Self {
        Self { container }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        builder.umount(&self.container)?;

        Ok(())
    }
}
