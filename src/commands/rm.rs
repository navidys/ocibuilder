use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct RemoveContainer {
    containers: Vec<String>,
}

impl RemoveContainer {
    pub fn new(containers: Vec<String>) -> Self {
        Self { containers }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        builder.rm(&self.containers)?;

        Ok(())
    }
}
