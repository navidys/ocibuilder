use std::ffi::OsString;

use clap::Parser;

use crate::{builder, error::BuilderResult, utils};

#[derive(Parser, Debug)]
pub struct Commit {
    /// container name or ID
    container: String,

    /// image name
    name: Option<String>,
}

impl Commit {
    pub fn new(container: String, name: Option<String>) -> Self {
        Self { container, name }
    }

    pub fn exec(&self, root_dir: Option<OsString>) -> BuilderResult<()> {
        let root_dir_path = utils::get_root_dir(root_dir);
        let builder = builder::oci::OCIBuilder::new(root_dir_path)?;

        let image_id = builder.commit(&self.container, self.name.to_owned())?;
        println!("{}", image_id.encoded);

        Ok(())
    }
}
