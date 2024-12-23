use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Mount {
    /// container name or ID
    container: String,
}

impl Mount {
    pub fn new(container: String) -> Self {
        Self { container }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        let mount_point = builder.mount(&self.container)?;
        println!("mount point: {:?}", mount_point);

        Ok(())
    }
}
