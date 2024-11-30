use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Add {
    /// Add an entry for this operation to the image's history.
    #[clap(long, required = false)]
    pub add_history: bool,

    /// container name or ID
    container: String,

    /// source file, directory, archive,...
    source: String,

    /// destination file or directory
    destination: String,
}

impl Add {
    pub fn new(container: String, source: String, destination: String, add_history: bool) -> Self {
        Self {
            container,
            source,
            destination,
            add_history,
        }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        builder.add(
            &self.container,
            &self.source,
            &self.destination,
            &self.add_history,
        )?;

        Ok(())
    }
}
