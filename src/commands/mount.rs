use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Mount {
    /// container name or ID
    #[clap(short, long)]
    name: Option<String>,
}

impl Mount {
    pub fn new(name: Option<String>) -> Self {
        Self { name }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let _builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        Ok(())
    }
}
