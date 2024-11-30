use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Copy {
    /// container name or ID
    container: String,

    /// source file, archive,...
    source: String,

    /// destination file or directory
    destination: String,
}

impl Copy {
    pub fn new(container: String, source: String, destination: String) -> Self {
        Self {
            container,
            source,
            destination,
        }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        builder.copy(&self.container, &self.source, &self.destination)?;

        Ok(())
    }
}
